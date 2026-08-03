//! Unsafe-site traversal and code block scanning.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use syn::visit::{self, Visit};

use super::items::*;
use super::types::*;
use crate::collect::type_param_names;
use crate::crate_scope::local_type_namespace_names;
use crate::finding::UnsafeSiteFact;
use crate::module_resolve::read_parse;
use crate::resolve::*;

struct UnsafeSiteCollector<'a> {
    sites: Vec<UnsafeSiteFact>,
    error: Option<String>,
    module: &'a str,
    uses: &'a UseMap,
    local_types: &'a HashSet<String>,
    // The enclosing `impl`'s self-type / `trait`'s name during the recursion, so an `unsafe fn`
    // method is owner-qualified (`unsafe fn Foo::m`) — else two same-named `unsafe fn`s on
    // different owners in one module collapse to one finding and a baseline of the first masks the
    // second (a false negative), the same injectivity `unsafe impl` already guards.
    current_owner: Option<String>,
    current_trait: Option<String>,
    // The trait of the enclosing *trait `impl`* (`None` for an inherent impl), so a trait-impl
    // `unsafe fn` is qualified by `<trait for self>` — else `impl Foo { unsafe fn m }` and
    // `impl A for Foo { unsafe fn m }` (same self type), or `impl A for Foo` and `impl B for Foo`
    // (same self type, different trait), collapse to one `unsafe fn Foo::m` and a baseline of one
    // masks the other (a false negative). Self-type alone only separates *different* self types.
    current_impl_trait: Option<String>,
    current_impl_is_trait: bool,
}

impl<'a> UnsafeSiteCollector<'a> {
    fn new(module: &'a str, uses: &'a UseMap, local_types: &'a HashSet<String>) -> Self {
        Self {
            sites: Vec::new(),
            error: None,
            module,
            uses,
            local_types,
            current_owner: None,
            current_trait: None,
            current_impl_trait: None,
            current_impl_is_trait: false,
        }
    }

    fn unsupported(&mut self, role: &str) {
        if self.error.is_none() {
            self.error = Some(format!(
                "cannot identify unsafe {role} in {} without a positional fallback",
                self.module
            ));
        }
    }
}

fn canonical_unsafe_owner(
    self_ty: &syn::Type,
    uses: &UseMap,
    local_types: &HashSet<String>,
    module: &str,
    impl_type_params: &HashSet<String>,
) -> Option<String> {
    if let syn::Type::Path(tp) = self_ty {
        if tp.qself.is_none() && !is_shadowed_param_path(&tp.path, impl_type_params) {
            let head = tp
                .path
                .segments
                .first()
                .map(|segment| strip_raw(&segment.ident.to_string()));
            let should_resolve = tp.path.leading_colon.is_some()
                || matches!(head.as_deref(), Some("crate" | "self" | "super"))
                || head
                    .as_ref()
                    .is_some_and(|head| uses.contains_key(head) || local_types.contains(head));
            if should_resolve {
                let base = resolve_path(&tp.path, uses, module, BareFallback::CurrentModule)?;
                return Some(format!("{base}{}", render_last_segment_args(&tp.path)?));
            }
        }
    }
    type_to_string(self_ty)
}

impl<'ast> Visit<'ast> for UnsafeSiteCollector<'_> {
    fn visit_expr_unsafe(&mut self, node: &'ast syn::ExprUnsafe) {
        self.sites.push(UnsafeSiteFact::Block);
        visit::visit_expr_unsafe(self, node);
    }

    fn visit_item_fn(&mut self, node: &'ast syn::ItemFn) {
        if node.sig.unsafety.is_some() {
            self.sites.push(UnsafeSiteFact::FreeFn {
                name: strip_raw(&node.sig.ident.to_string()),
            });
        }
        visit::visit_item_fn(self, node);
    }

    fn visit_impl_item_fn(&mut self, node: &'ast syn::ImplItemFn) {
        if node.sig.unsafety.is_some() {
            let name = strip_raw(&node.sig.ident.to_string());
            // Qualify by the enclosing impl (set in `visit_item_impl`): a *trait* impl by
            // `<trait for self>`, an inherent impl by its self type alone. Self-type alone only
            // separates *different* self types, so a trait-impl method and an inherent (or
            // other-trait) method with the same name on the *same* self type would otherwise
            // collapse to one finding and a baseline of one mask the other (a false negative).
            match (
                self.current_impl_is_trait,
                &self.current_impl_trait,
                &self.current_owner,
            ) {
                (true, Some(trait_ref), Some(owner)) => {
                    self.sites.push(UnsafeSiteFact::TraitImplMethod {
                        trait_ref: trait_ref.clone(),
                        owner: owner.clone(),
                        name,
                    });
                }
                (false, _, Some(owner)) => self.sites.push(UnsafeSiteFact::InherentMethod {
                    owner: owner.clone(),
                    name,
                }),
                _ => self.unsupported("method owner"),
            }
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if node.sig.unsafety.is_some() {
            let name = strip_raw(&node.sig.ident.to_string());
            // Qualify by the declaring trait (set in `visit_item_trait`), so two traits each
            // declaring `unsafe fn m` in one module do not collapse to one finding.
            match &self.current_trait {
                Some(owner) => self.sites.push(UnsafeSiteFact::TraitMethod {
                    owner: owner.clone(),
                    name,
                }),
                None => self.unsupported("trait-method owner"),
            }
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        // Owner-qualify by the implemented-for type so `unsafe impl Send for Foo` and
        // `unsafe impl Send for Bar` in one module stay distinct findings — else a baseline of
        // the first silently masks the second (a false negative). Lexical (`type_to_string`, no
        // resolution — this is the light walk), mirroring the trait-path rendering above. If the
        // self type cannot be rendered, an observed unsafe site fails loud rather than publishing
        // traversal position as identity. The same owner also qualifies inner unsafe methods.
        let params = type_param_names(&node.generics);
        let owner = canonical_unsafe_owner(
            &node.self_ty,
            self.uses,
            self.local_types,
            self.module,
            &params,
        );
        // The implemented trait (if any), rendered once — reused for the `unsafe impl` label and to
        // qualify the impl's inner `unsafe fn` methods as `<trait for self>` (injectivity above).
        let impl_trait = node
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path_to_string(path));
        if node.unsafety.is_some() {
            match (&impl_trait, &owner, node.trait_.is_some()) {
                (Some(trait_ref), Some(owner), true) => {
                    self.sites.push(UnsafeSiteFact::TraitImpl {
                        trait_ref: trait_ref.clone(),
                        owner: owner.clone(),
                    });
                }
                (None, Some(owner), false) => self.sites.push(UnsafeSiteFact::InherentImpl {
                    owner: owner.clone(),
                }),
                (None, _, true) => self.unsupported("impl trait"),
                (_, None, _) => self.unsupported("impl self type"),
                _ => unreachable!("trait presence and rendered trait stay aligned"),
            }
        }
        let prev_owner = std::mem::replace(&mut self.current_owner, owner);
        let prev_trait = self.current_impl_trait.take();
        let prev_is_trait = self.current_impl_is_trait;
        self.current_impl_is_trait = node.trait_.is_some();
        self.current_impl_trait = impl_trait;
        visit::visit_item_impl(self, node);
        self.current_owner = prev_owner;
        self.current_impl_trait = prev_trait;
        self.current_impl_is_trait = prev_is_trait;
    }

    fn visit_item_trait(&mut self, node: &'ast syn::ItemTrait) {
        let name = strip_raw(&node.ident.to_string());
        if node.unsafety.is_some() {
            self.sites
                .push(UnsafeSiteFact::Trait { name: name.clone() });
        }
        let prev = self
            .current_trait
            .replace(format!("{}::{name}", self.module));
        visit::visit_item_trait(self, node);
        self.current_trait = prev;
    }

    fn visit_item_foreign_mod(&mut self, node: &'ast syn::ItemForeignMod) {
        if node.unsafety.is_some() {
            self.sites.push(UnsafeSiteFact::ExternBlock);
        }
        visit::visit_item_foreign_mod(self, node);
    }
}

/// Walk the whole crate from its root and collect every `unsafe` site with its enclosing module.
/// Mirrors [`scan_crate`]'s descent (file + inline modules, ancestor-path cycle guard → exit 2, an
/// unconditional `#[path]` followed / a `cfg_attr`-wrapped one skipped as a stated bound, a
/// non-`#[cfg]` missing module file → exit 2, a cfg-gated missing file tolerated). A separate,
/// lighter walk than `scan_crate` (no re-export/alias/type-def resolution).
pub(crate) fn scan_unsafe_sites(
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
) -> Result<Vec<UnsafeSite>, String> {
    let root = read_parse(root_file)?;
    let mut sites = Vec::new();
    let mut ancestors: HashSet<PathBuf> = HashSet::new();
    ancestors.insert(xingbiao::canonicalize_or_fail(root_file)?);
    walk_unsafe(
        root.items,
        "crate".to_string(),
        src_dir.to_path_buf(),
        // The crate root is mod-rs-like: its own directory is the `#[path]` base too.
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        crate_package,
        &ancestors,
        0,
        &mut sites,
    )?;
    Ok(sites)
}

#[allow(clippy::too_many_arguments)]
fn walk_unsafe(
    items: Vec<syn::Item>,
    module: String,
    child_dir: PathBuf,
    file_dir: PathBuf,
    current_file: PathBuf,
    crate_package: &str,
    ancestors: &HashSet<PathBuf>,
    depth: usize,
    sites: &mut Vec<UnsafeSite>,
) -> Result<(), String> {
    check_module_depth(depth, &module, crate_package)?;
    // Feed the collector this module's items minus top-level `mod`s (walk-owned); body-nested
    // `mod`s stay in and are caught by the collector's default `visit_item_mod` recursion. Arms
    // flattened first, so an `unsafe` site written inside a `cfg_if!` arm is confined like any
    // other — the collector visits items, and an unflattened `Item::Macro` is an opaque token
    // stream it cannot see into.
    let (items, flat) = flatten_for_walk(&items);
    let uses = collect_uses(&items);
    let local_types = local_type_namespace_names(&items);
    let mut collector = UnsafeSiteCollector::new(&module, &uses, &local_types);
    for item in &items {
        if matches!(item, syn::Item::Mod(_)) {
            continue;
        }
        collector.visit_item(item);
    }
    if let Some(error) = collector.error {
        return Err(error);
    }
    for site in collector.sites {
        sites.push(UnsafeSite {
            module: module.clone(),
            file: current_file.clone(),
            site,
        });
    }

    for (child_items, child_module, sub_dir, sub_file_dir, opened, child_file) in
        resolve_child_modules(
            &flat,
            &module,
            &child_dir,
            &file_dir,
            &current_file,
            crate_package,
            ancestors,
        )?
    {
        match opened {
            Some(canon) => {
                let mut child_ancestors = ancestors.clone();
                child_ancestors.insert(canon);
                walk_unsafe(
                    child_items,
                    child_module,
                    sub_dir,
                    sub_file_dir,
                    child_file,
                    crate_package,
                    &child_ancestors,
                    depth + 1,
                    sites,
                )?;
            }
            None => walk_unsafe(
                child_items,
                child_module,
                sub_dir,
                sub_file_dir,
                child_file,
                crate_package,
                ancestors,
                depth + 1,
                sites,
            )?,
        }
    }
    Ok(())
}
