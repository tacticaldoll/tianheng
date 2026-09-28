//! Static-declaration traversal: every `static`, foreign `static` and `thread_local!` static a crate
//! declares, with the module that declares it and the named value items enclosing it.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};

use super::items::walk_subtree_direct_modules;
use super::owner::canonical_self_type_owner;
use crate::collect::type_param_names;
use crate::crate_scope::local_type_namespace_names;
use crate::errors::{
    static_owner_unnameable_error, thread_local_body_error, thread_local_rename_error,
    undecodable_foreign_item_error,
};
use crate::finding::StaticKind;
use crate::resolve::{UseMap, collect_uses, path_to_string, strip_raw};
use crate::syn_util::{ForeignDecl, decode_foreign_item, is_thread_local_macro};

/// One declared static: the module that declares it, the file that module's branch was read from,
/// and what its fact records.
pub(crate) struct StaticSite {
    pub(crate) module: String,
    pub(crate) file: PathBuf,
    pub(crate) kind: StaticKind,
    pub(crate) name: String,
    pub(crate) owner: String,
}

/// One segment of a static's owner chain: a named value item, or the reason an enclosing impl or
/// trait cannot be named. A refusal is recorded only when a static is found beneath it, so an
/// unnameable impl holding no static costs nothing.
type OwnerSegment = Result<String, String>;

struct StaticCollector<'a> {
    module: &'a str,
    file: &'a Path,
    uses: &'a UseMap,
    local_types: &'a HashSet<String>,
    owners: Vec<OwnerSegment>,
    /// The enclosing `impl` or `trait`, rendered as the prefix its items' names are qualified by.
    item_prefix: Option<OwnerSegment>,
    sites: Vec<StaticSite>,
    rename: Option<String>,
    error: Option<String>,
}

impl<'a> StaticCollector<'a> {
    fn new(
        module: &'a str,
        file: &'a Path,
        uses: &'a UseMap,
        local_types: &'a HashSet<String>,
    ) -> Self {
        Self {
            module,
            file,
            uses,
            local_types,
            owners: Vec::new(),
            item_prefix: None,
            sites: Vec::new(),
            rename: None,
            error: None,
        }
    }

    fn fail(&mut self, error: String) {
        if self.error.is_none() {
            self.error = Some(error);
        }
    }

    fn record(&mut self, kind: StaticKind, ident: &syn::Ident) {
        let name = strip_raw(&ident.to_string());
        let mut owner = Vec::new();
        for segment in &self.owners {
            match segment {
                Ok(segment) => owner.push(segment.clone()),
                Err(cause) => {
                    let error = static_owner_unnameable_error(&name, self.module, cause);
                    self.fail(error);
                    return;
                }
            }
        }
        self.sites.push(StaticSite {
            module: self.module.to_string(),
            file: self.file.to_path_buf(),
            kind,
            name,
            owner: owner.join("::"),
        });
    }

    /// Visit `inner` with `segment` closing the owner chain.
    fn within(&mut self, segment: OwnerSegment, inner: impl FnOnce(&mut Self)) {
        self.owners.push(segment);
        inner(self);
        self.owners.pop();
    }

    /// `segment` qualified by the enclosing impl or trait, or refused with it.
    fn qualified(&self, segment: String) -> OwnerSegment {
        match &self.item_prefix {
            Some(Ok(prefix)) => Ok(format!("{prefix}::{segment}")),
            Some(Err(cause)) => Err(cause.clone()),
            None => Ok(segment),
        }
    }

    /// The statics one `thread_local!` invocation declares, each recorded under the enclosing owner.
    fn thread_local(&mut self, mac: &syn::Macro) {
        let statics = match mac.parse_body_with(thread_local_statics) {
            Ok(statics) => statics,
            Err(error) => {
                let error = thread_local_body_error(self.module, self.file, &error.to_string());
                self.fail(error);
                return;
            }
        };
        for (ident, ty, expr) in &statics {
            self.record(StaticKind::ThreadLocal, ident);
            let segment = Ok(format!("static {}", strip_raw(&ident.to_string())));
            self.within(segment, |this| {
                this.visit_type(ty);
                this.visit_expr(expr);
            });
        }
    }

    fn note_rename(&mut self, tree: &syn::UseTree) {
        match tree {
            syn::UseTree::Path(path) => self.note_rename(&path.tree),
            syn::UseTree::Group(group) => {
                for tree in &group.items {
                    self.note_rename(tree);
                }
            }
            syn::UseTree::Rename(rename) => {
                let renamed = strip_raw(&rename.rename.to_string());
                if strip_raw(&rename.ident.to_string()) == "thread_local"
                    && renamed != "thread_local"
                    && renamed != "_"
                    && self.rename.is_none()
                {
                    self.rename = Some(renamed);
                }
            }
            syn::UseTree::Name(_) | syn::UseTree::Glob(_) => {}
        }
    }
}

impl<'ast> Visit<'ast> for StaticCollector<'_> {
    fn visit_item_static(&mut self, node: &'ast syn::ItemStatic) {
        let kind = match node.mutability {
            syn::StaticMutability::Mut(_) => StaticKind::StaticMut,
            _ => StaticKind::Static,
        };
        self.record(kind, &node.ident);
        let segment = Ok(format!("static {}", strip_raw(&node.ident.to_string())));
        self.within(segment, |this| visit::visit_item_static(this, node));
    }

    fn visit_item_const(&mut self, node: &'ast syn::ItemConst) {
        let segment = Ok(format!("const {}", strip_raw(&node.ident.to_string())));
        self.within(segment, |this| visit::visit_item_const(this, node));
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        let segment = Ok(strip_raw(&node.sig.ident.to_string()));
        let prefix = self.item_prefix.take();
        self.within(segment, |this| visit::visit_item_fn(this, node));
        self.item_prefix = prefix;
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let params = type_param_names(&node.generics);
        let owner = canonical_self_type_owner(
            &node.self_ty,
            self.uses,
            self.local_types,
            self.module,
            &params,
        )
        .map(|owner| {
            owner
                .strip_prefix(self.module)
                .and_then(|rest| rest.strip_prefix("::"))
                .map_or(owner.clone(), str::to_string)
        })
        .map_err(|why| format!("impl self type cannot be named: {}", why.cause()));
        let prefix = match (&node.trait_, owner) {
            (None, owner) => owner,
            (Some((_, path, _)), Ok(owner)) => match path_to_string(path) {
                Some(trait_ref) => Ok(format!("<{trait_ref} for {owner}>")),
                None => Err("impl trait cannot be named".to_string()),
            },
            (Some(_), Err(cause)) => Err(cause),
        };
        let previous = self.item_prefix.replace(prefix);
        visit::visit_item_impl(self, node);
        self.item_prefix = previous;
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let previous = self
            .item_prefix
            .replace(Ok(strip_raw(&node.ident.to_string())));
        visit::visit_item_trait(self, node);
        self.item_prefix = previous;
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        let segment = self.qualified(strip_raw(&node.sig.ident.to_string()));
        let prefix = self.item_prefix.take();
        self.within(segment, |this| visit::visit_impl_item_fn(this, node));
        self.item_prefix = prefix;
    }

    fn visit_impl_item_const(&mut self, node: &'ast syn::ImplItemConst) {
        let segment = self
            .qualified(strip_raw(&node.ident.to_string()))
            .map(|name| format!("const {name}"));
        let prefix = self.item_prefix.take();
        self.within(segment, |this| visit::visit_impl_item_const(this, node));
        self.item_prefix = prefix;
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        let segment = self.qualified(strip_raw(&node.sig.ident.to_string()));
        let prefix = self.item_prefix.take();
        self.within(segment, |this| visit::visit_trait_item_fn(this, node));
        self.item_prefix = prefix;
    }

    fn visit_trait_item_const(&mut self, node: &'ast syn::TraitItemConst) {
        let segment = self
            .qualified(strip_raw(&node.ident.to_string()))
            .map(|name| format!("const {name}"));
        let prefix = self.item_prefix.take();
        self.within(segment, |this| visit::visit_trait_item_const(this, node));
        self.item_prefix = prefix;
    }

    fn visit_foreign_item(&mut self, node: &'ast syn::ForeignItem) {
        match decode_foreign_item(node) {
            Ok(ForeignDecl::Static {
                ident, mutability, ..
            }) => {
                let kind = match mutability {
                    syn::StaticMutability::Mut(_) => StaticKind::ForeignStaticMut,
                    _ => StaticKind::ForeignStatic,
                };
                self.record(kind, &ident);
            }
            Ok(ForeignDecl::Fn { .. } | ForeignDecl::Type { .. } | ForeignDecl::Macro) => {}
            Err(undecodable) => {
                let error =
                    undecodable_foreign_item_error(self.module, self.file, &undecodable.seen);
                self.fail(error);
            }
        }
    }

    fn visit_item_macro(&mut self, node: &'ast syn::ItemMacro) {
        if node.ident.is_none() && is_thread_local_macro(&node.mac.path) {
            self.thread_local(&node.mac);
        }
    }

    fn visit_stmt_macro(&mut self, node: &'ast syn::StmtMacro) {
        if is_thread_local_macro(&node.mac.path) {
            self.thread_local(&node.mac);
        }
    }

    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        self.note_rename(&node.tree);
    }
}

/// The statics a `thread_local!` body declares, read by the macro's own grammar: attributed,
/// visibility-qualified `static NAME: T = init` declarations separated by `;`, where the last one's
/// `;` is optional.
///
/// std's `thread_local!` matches a declaration's initializer as `$init:expr $(; $($rest:tt)*)?`, and
/// its own documentation writes `thread_local!(static FOO: Cell<u32> = Cell::new(1));`, so a reader
/// requiring every declaration to end in `;` would refuse a body std accepts. Only `static`
/// declarations are read, because std's grammar admits nothing else.
fn thread_local_statics(
    input: syn::parse::ParseStream,
) -> syn::Result<Vec<(syn::Ident, syn::Type, syn::Expr)>> {
    let mut statics = Vec::new();
    while !input.is_empty() {
        input.call(syn::Attribute::parse_outer)?;
        input.parse::<syn::Visibility>()?;
        input.parse::<syn::Token![static]>()?;
        let ident = input.parse::<syn::Ident>()?;
        input.parse::<syn::Token![:]>()?;
        let ty = input.parse::<syn::Type>()?;
        input.parse::<syn::Token![=]>()?;
        let expr = input.parse::<syn::Expr>()?;
        statics.push((ident, ty, expr));
        if input.is_empty() {
            break;
        }
        input.parse::<syn::Token![;]>()?;
    }
    Ok(statics)
}

pub(crate) struct StaticSiteError {
    pub(crate) module: String,
    pub(crate) error: String,
}

/// What one compilation unit declares: every static site, and the first rename of `thread_local`
/// met anywhere in the unit, with the module that wrote it.
pub(crate) struct StaticScan {
    pub(crate) sites: Vec<StaticSite>,
    pub(crate) rename: Option<(String, String)>,
    pub(crate) errors: Vec<StaticSiteError>,
}

/// Walk the whole unit from its root and collect every declared static with its declaring module.
///
/// The descent is the subtree walk every module-anchored boundary resolves with, anchored at
/// `crate` and returning each module's direct items — so an `impl` recovered from a function body
/// is reached once, through the body, rather than a second time as a direct item. An inline child
/// `mod` is skipped as an item because the walk yields it as its own module. A crate-wide rename of
/// `thread_local` is found in the same pass, since an invocation under the new name could sit
/// anywhere the anchored subtree reaches.
pub(crate) fn scan_static_sites(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
) -> Result<StaticScan, String> {
    let modules = walk_subtree_direct_modules(src_dir, root_file, "crate", crate_package)?;
    let mut sites = Vec::new();
    let mut rename = None;
    let mut errors = Vec::new();
    for (module, items, file) in &modules {
        let uses = collect_uses(items);
        let local_types = local_type_namespace_names(items);
        let mut collector = StaticCollector::new(module, file, &uses, &local_types);
        for item in items {
            if matches!(item, syn::Item::Mod(_)) {
                continue;
            }
            collector.visit_item(item);
        }
        if let Some(error) = collector.error {
            errors.push(StaticSiteError {
                module: module.clone(),
                error,
            });
        }
        if rename.is_none() {
            rename = collector.rename.map(|to| (to, module.clone()));
        }
        sites.extend(collector.sites);
    }
    Ok(StaticScan {
        sites,
        rename,
        errors,
    })
}

/// The refusal a unit's `thread_local` rename earns, if it has one.
pub(crate) fn refuse_thread_local_rename(
    scan: &StaticScan,
    crate_package: &str,
) -> Result<(), String> {
    match &scan.rename {
        Some((renamed_to, module)) => {
            Err(thread_local_rename_error(renamed_to, module, crate_package))
        }
        None => Ok(()),
    }
}
