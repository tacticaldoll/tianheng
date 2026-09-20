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
    /// The enclosing `impl`'s self-type or `trait`'s name during recursion for owner qualification.
    current_owner: Option<Result<String, OwnerUnnameable>>,
    current_trait: Option<String>,
    /// The trait of the enclosing trait `impl` (`None` for an inherent impl) for `<trait for self>` qualification.
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

    /// Refuse to name an owner, saying what was met rather than only what is not invented.
    ///
    /// The three causes reached one sentence — *cannot identify … without a positional fallback* — which
    /// names the policy and not the fact. An adopter grepping for their own case found nothing to match.
    fn unsupported(&mut self, role: &str, why: OwnerUnnameable) {
        if self.error.is_none() {
            self.error = Some(format!(
                "cannot identify unsafe {role} in {} — {}; no positional fallback is invented for it, \
                 because a label that names a traversal position is not an identity",
                self.module,
                why.cause()
            ));
        }
    }

    /// The one refusal here that is NOT about an owner: an impl block whose TRAIT could not be named.
    /// It keeps its own sentence, because [`OwnerUnnameable`]'s causes are about a self type.
    fn unsupported_trait(&mut self, role: &str) {
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
) -> Result<String, OwnerUnnameable> {
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
                let mut candidates =
                    resolve_path_all(&tp.path, uses, module, BareFallback::CurrentModule);
                candidates.sort();
                candidates.dedup();
                let [base] = candidates.as_slice() else {
                    return Err(OwnerUnnameable::AmbiguousAlias);
                };
                let args =
                    render_last_segment_args(&tp.path).ok_or(OwnerUnnameable::Unrenderable)?;
                return Ok(format!("{base}{args}"));
            }
        }
    }
    type_to_string(self_ty).ok_or(OwnerUnnameable::Unrenderable)
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
            match (
                self.current_impl_is_trait,
                &self.current_impl_trait,
                &self.current_owner,
            ) {
                (true, Some(trait_ref), Some(Ok(owner))) => {
                    self.sites.push(UnsafeSiteFact::TraitImplMethod {
                        trait_ref: trait_ref.clone(),
                        owner: owner.clone(),
                        name,
                    });
                }
                (false, _, Some(Ok(owner))) => self.sites.push(UnsafeSiteFact::InherentMethod {
                    owner: owner.clone(),
                    name,
                }),
                (_, _, Some(Err(why))) => {
                    let why = *why;
                    self.unsupported("method owner", why);
                }
                _ => self.unsupported_trait("method owner's trait"),
            }
        }
        visit::visit_impl_item_fn(self, node);
    }

    fn visit_trait_item_fn(&mut self, node: &'ast syn::TraitItemFn) {
        if node.sig.unsafety.is_some() {
            let name = strip_raw(&node.sig.ident.to_string());
            match &self.current_trait {
                Some(owner) => self.sites.push(UnsafeSiteFact::TraitMethod {
                    owner: owner.clone(),
                    name,
                }),
                None => self.unsupported_trait("trait-method owner"),
            }
        }
        visit::visit_trait_item_fn(self, node);
    }

    fn visit_item_impl(&mut self, node: &'ast syn::ItemImpl) {
        let params = type_param_names(&node.generics);
        let owner = canonical_unsafe_owner(
            &node.self_ty,
            self.uses,
            self.local_types,
            self.module,
            &params,
        );
        let impl_trait = node
            .trait_
            .as_ref()
            .and_then(|(_, path, _)| path_to_string(path));
        if node.unsafety.is_some() {
            match (&impl_trait, &owner, node.trait_.is_some()) {
                (Some(trait_ref), Ok(owner), true) => {
                    self.sites.push(UnsafeSiteFact::TraitImpl {
                        trait_ref: trait_ref.clone(),
                        owner: owner.clone(),
                    });
                }
                (None, Ok(owner), false) => self.sites.push(UnsafeSiteFact::InherentImpl {
                    owner: owner.clone(),
                }),
                (None, _, true) => self.unsupported_trait("impl trait"),
                (_, Err(why), _) => {
                    let why = *why;
                    self.unsupported("impl self type", why);
                }
                _ => unreachable!("trait presence and rendered trait stay aligned"),
            }
        }
        let prev_owner = self.current_owner.replace(owner);
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
