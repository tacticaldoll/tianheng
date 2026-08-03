use syn::visit::Visit;

// The AST-collector cluster — the pure syntax-tree walkers that turn one parsed `syn::Item`
// into the exposure findings each semantic rule reacts to. Every collector observes only the
// public surface (via [`crate::syn_util::is_public`]), stamps each finding with its seam, and returns; the
// reaction/decision lives in `lib.rs`. This module holds no state and makes no I/O.

use crate::finding::{
    AssocKind, ItemKind, MemberKind, PathExposure, PublicSeam, SemanticFact, field_seam, fn_seam,
    inherent_assoc_seam, inherent_method_seam, item_seam, member_label, render_sig_tail, tag_paths,
    trait_assoc_seam, trait_method_seam,
};
use crate::resolve::{
    ImplTraitCollector, PathCollector, ShapeExposure, UseMap, canonical_self_owner,
    canonical_self_owner_without_fallback, stamp_seam, strip_raw,
};
use crate::syn_util::is_public;

/// Collect the returned-`impl Trait` [`ShapeExposure`]s in the **return type** of a public item's
/// functions/methods only (the existential positions). Never visits argument positions (APIT is
/// universal, not a leak) nor trait-*impl* methods (their return shape is dictated by the trait).
pub(crate) fn collect_item_return_impl_traits(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<ShapeExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(stamp_seam(impl_traits_in_return(&item.sig), &seam));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            // A trait method's return is part of the public trait API (trait items carry no
            // individual visibility); the trait DECLARES any RPIT here.
            let trait_name = strip_raw(&item.ident.to_string());
            for trait_item in &item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                    out.extend(stamp_seam(impl_traits_in_return(&method.sig), &seam));
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let owner = canonical_self_owner(
                &item.self_ty,
                uses,
                module,
                ordinal,
                &type_param_names(&item.generics),
            );
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        let seam = inherent_method_seam(module, &owner, &method.sig.ident);
                        out.extend(stamp_seam(impl_traits_in_return(&method.sig), &seam));
                    }
                }
            }
        }
        _ => {}
    }
}

/// The returned-`impl Trait` [`ShapeExposure`]s in a signature's **return type** (at any depth).
/// Visits `sig.output` ONLY — never `sig.inputs`, so argument-position `impl Trait` (APIT) is
/// excluded.
pub(crate) fn impl_traits_in_return(sig: &syn::Signature) -> Vec<ShapeExposure> {
    let mut collector = ImplTraitCollector::default();
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        collector.visit_type(ty);
    }
    collector.exposures
}

pub(crate) fn collect_item_async_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    _ordinal: usize,
    out: &mut Vec<SemanticFact>,
) -> Result<(), String> {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            if item.sig.asyncness.is_some() {
                out.push(SemanticFact::AsyncFreeFn {
                    module: module.to_string(),
                    name: strip_raw(&item.sig.ident.to_string()),
                    tail: render_sig_tail(&item.sig),
                });
            }
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            for trait_item in &item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    if method.sig.asyncness.is_some() {
                        out.push(SemanticFact::AsyncTraitMethod {
                            module: module.to_string(),
                            trait_name: trait_name.clone(),
                            name: strip_raw(&method.sig.ident.to_string()),
                            tail: render_sig_tail(&method.sig),
                        });
                    }
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let async_methods: Vec<&syn::ImplItemFn> = item
                .items
                .iter()
                .filter_map(|impl_item| match impl_item {
                    syn::ImplItem::Fn(method)
                        if is_public(&method.vis) && method.sig.asyncness.is_some() =>
                    {
                        Some(method)
                    }
                    _ => None,
                })
                .collect();
            if async_methods.is_empty() {
                return Ok(());
            }
            let owner = canonical_self_owner_without_fallback(
                &item.self_ty,
                uses,
                module,
                &type_param_names(&item.generics),
            )
            .ok_or_else(|| {
                format!(
                    "cannot identify public async method owner in {module} without a positional fallback"
                )
            })?;
            for method in async_methods {
                out.push(SemanticFact::AsyncInherentMethod {
                    module: module.to_string(),
                    owner: owner.clone(),
                    name: strip_raw(&method.sig.ident.to_string()),
                    tail: render_sig_tail(&method.sig),
                });
            }
        }
        _ => {}
    }
    Ok(())
}

/// The generic **type-parameter** names declared by `generics` — the names that, used bare, are
/// parameters rather than nominal types (so a same-named `type` alias must not resolve them).
pub(crate) fn type_param_names(generics: &syn::Generics) -> std::collections::HashSet<String> {
    generics
        .params
        .iter()
        .filter_map(|p| match p {
            syn::GenericParam::Type(tp) => Some(strip_raw(&tp.ident.to_string())),
            _ => None,
        })
        .collect()
}

/// Paths in a signature, shadowing the signature's OWN generic type parameters (`fn f<T>(x: T)` —
/// `T` is a param use, not a nominal type). A signature always carries its own generics, so this is
/// the base every fn/method exposure walk uses.
pub(crate) fn paths_in_signature(sig: &syn::Signature) -> Vec<syn::Path> {
    paths_in_signature_scoped(sig, &std::collections::HashSet::new())
}

/// Like [`paths_in_signature`] but also shadowing the **enclosing** item's generic type parameters
/// (an inherent-impl / trait's `<T>` is in scope inside its methods), so a method parameter named
/// like an enclosing param — or a same-module alias — is not misresolved.
pub(crate) fn paths_in_signature_scoped(
    sig: &syn::Signature,
    enclosing: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut shadowed = enclosing.clone();
    shadowed.extend(type_param_names(&sig.generics));
    let mut c = PathCollector::shadowing(shadowed);
    c.visit_signature(sig);
    c.paths
}

pub(crate) fn paths_in_type(ty: &syn::Type) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    c.visit_type(ty);
    c.paths
}

/// Paths in a type, shadowing the given in-scope generic type parameters — used where a type
/// position (a field, an alias target, an assoc item) sits inside a generic item whose params must
/// not be mistaken for nominal types.
pub(crate) fn paths_in_type_scoped(
    ty: &syn::Type,
    params: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut c = PathCollector::shadowing(params.clone());
    c.visit_type(ty);
    c.paths
}

/// Paths in an item's generics (its param bounds and where-clause), shadowing the given in-scope
/// generic type parameters. A def/impl generic param used bare inside its own bounds
/// (`struct S<T, U> where U: AsRef<T>` — `T` is a parameter, not a nominal type) must be shadowed;
/// otherwise a same-named module use-alias (`use crate::infra::Secret as T;`) misresolves the bare
/// `T` to the aliased type and emits a spurious exposure — the exact false positive the
/// [`PathCollector`] shadowing was built to prevent. A multi-segment forbidden path is never
/// shadowed, so real leaks in bounds are still observed.
pub(crate) fn paths_in_generics_scoped(
    generics: &syn::Generics,
    params: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut c = PathCollector::shadowing(params.clone());
    c.visit_generics(generics);
    c.paths
}

/// Collect the type paths exposed by one item's public surface. Only `pub` items
/// contribute; `pub(crate)`/`pub(in …)`/private are internal, not exposed. Trait `impl`
/// blocks are skipped (out of scope — their shape is the trait's, not the impl site's).
/// Collect a struct-like member list's (a `struct`'s or `union`'s named fields, or one enum
/// variant's fields) governed-field exposures, tagging each with its per-field member seam — the
/// field-iteration shape shared by the struct/union/enum-variant arms of both
/// [`collect_item_exposures`] and [`collect_item_dyn_exposures`]. `is_governed` decides which
/// fields react: struct/union fields carry their own visibility, so callers pass
/// `is_public(&field.vis)`; an enum variant's fields are as public as the enum itself regardless
/// of `field.vis` (always `Inherited` there — never independently `pub`), so that caller passes
/// an always-true predicate instead — folding `is_public` into every call site here would silently
/// drop every enum-variant field exposure, a real false negative. `extract` turns one field's type
/// into its raw, seam-tagged exposures; the two collectors differ only in what it does: path
/// exposures scoped against the owner's generic params (`paths_in_type_scoped` + `tag_paths`) for
/// the former, dyn exposures via `dyns_in_type` + `stamp_seam` for the latter — unscoped, since a
/// `dyn` node is a shape, not a resolvable path, so it carries nothing for a generic parameter to
/// shadow.
pub(crate) fn collect_named_field_exposures<'f, E>(
    fields: impl Iterator<Item = &'f syn::Field>,
    kind: MemberKind,
    module: &str,
    owner: &str,
    is_governed: impl Fn(&syn::Field) -> bool,
    mut extract: impl FnMut(&syn::Type, &PublicSeam) -> Vec<E>,
    out: &mut Vec<E>,
) {
    for (index, field) in fields.enumerate() {
        if is_governed(field) {
            let seam = field_seam(kind, module, owner, &member_label(index, field));
            out.extend(extract(&field.ty, &seam));
        }
    }
}

pub(crate) fn collect_item_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<PathExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(tag_paths(paths_in_signature(&item.sig), &seam));
        }
        syn::Item::Struct(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &item_seam(ItemKind::Struct, module, &item.ident),
            ));
            collect_named_field_exposures(
                item.fields.iter(),
                MemberKind::Field,
                module,
                &name,
                |field| is_public(&field.vis),
                |ty, seam| tag_paths(paths_in_type_scoped(ty, &params), seam),
                out,
            );
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &item_seam(ItemKind::Enum, module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself. Each field
            // carries a per-member seam (`variant {Enum}::{Variant}::{index|name}`), mirroring
            // struct/union fields, so two forbidden fields of one variant stay distinct findings
            // — never collapsing to one `(target, rule, finding)` and masking a new leak.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                collect_named_field_exposures(
                    variant.fields.iter(),
                    MemberKind::Variant,
                    module,
                    &owner,
                    |_| true,
                    |ty, seam| tag_paths(paths_in_type_scoped(ty, &params), seam),
                    out,
                );
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &item_seam(ItemKind::Union, module, &item.ident),
            ));
            collect_named_field_exposures(
                item.fields.named.iter(),
                MemberKind::Field,
                module,
                &name,
                |field| is_public(&field.vis),
                |ty, seam| tag_paths(paths_in_type_scoped(ty, &params), seam),
                out,
            );
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam(ItemKind::Type, module, &item.ident);
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &seam,
            ));
            out.extend(tag_paths(paths_in_type_scoped(&item.ty, &params), &seam));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(tag_paths(
                paths_in_type(&item.ty),
                &item_seam(ItemKind::Const, module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(tag_paths(
                paths_in_type(&item.ty),
                &item_seam(ItemKind::Static, module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            let trait_seam = item_seam(ItemKind::Trait, module, &item.ident);
            let trait_params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &trait_params),
                &trait_seam,
            ));
            // Supertraits are part of the trait's public contract; walk them with the same
            // full recursion (paths_in_bounds → PathCollector) every other position uses, so a
            // forbidden type in a bound's generic argument (`Facade: AsRef<crate::infra::Secret>`)
            // is observed too — not only the bound's head trait (which paths_in_bounds still pushes,
            // preserving forbidden-supertrait-head detection).
            out.extend(tag_paths(paths_in_bounds(&item.supertraits), &trait_seam));
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(method) => {
                        let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                        out.extend(tag_paths(
                            paths_in_signature_scoped(&method.sig, &trait_params),
                            &seam,
                        ));
                    }
                    syn::TraitItem::Type(assoc) => {
                        let seam =
                            trait_assoc_seam(AssocKind::Type, module, &trait_name, &assoc.ident);
                        // Full-recursion coverage for every bound position of a public associated
                        // type: its own bounds (`: Into<crate::infra::Secret>`), its generic
                        // parameters (GAT `<T: crate::infra::Marker>` + where-clause), and its
                        // default target (`= crate::infra::Secret`, an observed type position the
                        // `dyn` collector already walks) — so a forbidden generic argument here is
                        // not silently dropped.
                        // The trait's params AND the GAT's own params are in scope inside the GAT's
                        // bounds/where-clause, so shadow both — a bare param there is a parameter,
                        // not a nominal type reachable through a same-named alias.
                        let mut assoc_params = trait_params.clone();
                        assoc_params.extend(type_param_names(&assoc.generics));
                        out.extend(tag_paths(
                            paths_in_bounds_scoped(&assoc.bounds, &assoc_params),
                            &seam,
                        ));
                        out.extend(tag_paths(
                            paths_in_generics_scoped(&assoc.generics, &assoc_params),
                            &seam,
                        ));
                        if let Some((_, ty)) = &assoc.default {
                            // The trait's and the GAT's own type params are in scope in the default.
                            out.extend(tag_paths(paths_in_type_scoped(ty, &assoc_params), &seam));
                        }
                    }
                    syn::TraitItem::Const(assoc) => {
                        let seam =
                            trait_assoc_seam(AssocKind::Const, module, &trait_name, &assoc.ident);
                        out.extend(tag_paths(
                            paths_in_type_scoped(&assoc.ty, &trait_params),
                            &seam,
                        ));
                    }
                    _ => {}
                }
            }
        }
        // Inherent `impl Type { … }` (no trait): its `pub` methods are public API the module
        // authored. Trait impls (`impl Trait for Type`) carry `trait_` and are out of scope.
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let impl_params = type_param_names(&item.generics);
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal, &impl_params);
            // The impl block's own generic-param bounds and where-clause are impl-site-authored
            // public contract for the inherent API (`impl<T: crate::infra::Secret> Foo<T> { … }`),
            // observed like a struct/enum/type def's generics (paths_in_generics_scoped) and the
            // trait-impl collector's where-walk. Owner-qualified so it stays distinct from the
            // block's methods / assoc items, and module-qualified — like the sibling method /
            // assoc seams below — so two blocks for the SAME owner written in two different
            // modules stay distinct too: an owner names what the impl is for, never where it is
            // written, and inherent impls carry no coherence exclusion to lean on.
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &impl_params),
                &PublicSeam::InherentGenerics {
                    module: module.to_string(),
                    owner: owner.clone(),
                },
            ));
            for impl_item in &item.items {
                match impl_item {
                    // A public method's signature. The impl's own `<T>` is in scope inside it, so
                    // shadow it (plus the method's own params) to keep a param use from resolving
                    // through a same-named alias.
                    syn::ImplItem::Fn(method) if is_public(&method.vis) => {
                        let seam = inherent_method_seam(module, &owner, &method.sig.ident);
                        out.extend(tag_paths(
                            paths_in_signature_scoped(&method.sig, &impl_params),
                            &seam,
                        ));
                    }
                    // A public associated `const`'s declared type is public API (`Foo::K`).
                    syn::ImplItem::Const(assoc) if is_public(&assoc.vis) => {
                        let seam =
                            inherent_assoc_seam(AssocKind::Const, module, &owner, &assoc.ident);
                        out.extend(tag_paths(
                            paths_in_type_scoped(&assoc.ty, &impl_params),
                            &seam,
                        ));
                    }
                    // A public associated `type`'s target is public API (`Foo::T`).
                    syn::ImplItem::Type(assoc) if is_public(&assoc.vis) => {
                        let seam =
                            inherent_assoc_seam(AssocKind::Type, module, &owner, &assoc.ident);
                        out.extend(tag_paths(
                            paths_in_type_scoped(&assoc.ty, &impl_params),
                            &seam,
                        ));
                    }
                    _ => {}
                }
            }
        }
        // A bare `pub use` republishes what it names on the module's public surface — the most
        // direct exposure (`semantic-reexport-exposure`). Restricted-visibility re-exports are
        // internal, like a private field. The walked path flows through the same resolve →
        // canonicalize → match pipeline as any exposed type.
        syn::Item::Use(item) if is_public(&item.vis) => {
            walk_reexport_tree(
                &item.tree,
                Vec::new(),
                module,
                item.leading_colon.is_some(),
                out,
            );
        }
        // A `pub extern crate X [as Y];` republishes the external crate root `X` on the module's
        // public surface — like `pub use ::X;`. The exposure names the **real** crate `X` (not the
        // `as`-rename), a bare extern head (raw external set, `is_reexport`). `extern crate self`
        // renames the current crate, not an external exposure.
        syn::Item::ExternCrate(item) if is_public(&item.vis) && item.ident != "self" => {
            let name = strip_raw(&item.ident.to_string());
            out.push(PathExposure {
                seam: PublicSeam::ExternCrate {
                    module: module.to_string(),
                    name,
                },
                path: syn::Path::from(item.ident.clone()),
                is_reexport: true,
            });
        }
        // A `pub fn`/`pub static` inside an `extern` block is a real item in this module's own
        // namespace — the FFI declaration, not a definition, but still exactly as public as a
        // same-shaped ordinary `Item::Fn`/`Item::Static` (Rust cannot even declare both under one
        // name in the same module, so there is no identity collision to avoid by treating them
        // differently). Reused verbatim so a forbidden type named only in an `extern` block's
        // signature is not invisible to this query merely because it has no body.
        syn::Item::ForeignMod(item) => {
            for foreign_item in &item.items {
                match foreign_item {
                    syn::ForeignItem::Fn(f) if is_public(&f.vis) => {
                        let seam = fn_seam(module, &f.sig.ident);
                        out.extend(tag_paths(paths_in_signature(&f.sig), &seam));
                    }
                    syn::ForeignItem::Static(s) if is_public(&s.vis) => {
                        out.extend(tag_paths(
                            paths_in_type(&s.ty),
                            &item_seam(ItemKind::Static, module, &s.ident),
                        ));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

/// Whether an ident is the `self` keyword-segment of a `use` tree (`{self, X}` / `self as alias`),
/// meaning "the prefix module itself". `self` is a keyword and never a raw identifier, so a string
/// compare is exact.
pub(crate) fn is_self_segment(ident: &syn::Ident) -> bool {
    ident == "self"
}

/// Walk a `pub use` tree, pushing one [`PathExposure`] per re-exported leaf (and the root of a
/// glob), seam-qualified by the **exported** path so two aliases of the same forbidden type stay
/// distinct findings. Handles: named/renamed leaves; grouped re-exports (per leaf); a whole-module
/// re-export (`pub use crate::infra as fs` — the leaf path is a module, matched like any path); a
/// `self` group member (`{self, X}` — re-exports the prefix module, keyed by the prefix's final
/// segment, never the literal `self`); a glob (the root prefix, which reacts iff it resolves
/// in/under the forbidden set). `as _` binds no nameable path — a stated non-observed bound.
/// A `self` group member and a renamed `self` both mean "the prefix module itself" — collapse to
/// the prefix, keyed by the prefix's final segment (or the alias).
pub(crate) fn walk_reexport_tree(
    tree: &syn::UseTree,
    prefix: Vec<syn::Ident>,
    module: &str,
    leading_colon: bool,
    out: &mut Vec<PathExposure>,
) {
    match tree {
        syn::UseTree::Path(path) => {
            let mut segs = prefix;
            segs.push(path.ident.clone());
            walk_reexport_tree(&path.tree, segs, module, leading_colon, out);
        }
        syn::UseTree::Name(name) => {
            if is_self_segment(&name.ident) {
                // `pub use crate::infra::{self, …}` re-exports the prefix module, bound under the
                // prefix's final segment (never the literal `self`).
                let exported = prefix.last().map(seg_name);
                push_reexport(&prefix, exported.as_deref(), module, leading_colon, out);
            } else {
                let exported = seg_name(&name.ident);
                let mut segs = prefix;
                segs.push(name.ident.clone());
                push_reexport(&segs, Some(&exported), module, leading_colon, out);
            }
        }
        syn::UseTree::Rename(rename) => {
            let alias = seg_name(&rename.rename);
            if alias == "_" {
                return; // `as _` binds no nameable path — a stated non-observed bound
            }
            if is_self_segment(&rename.ident) {
                // `pub use crate::infra::{self as fs}` — the prefix module, renamed.
                push_reexport(&prefix, Some(&alias), module, leading_colon, out);
            } else {
                let mut segs = prefix;
                segs.push(rename.ident.clone());
                push_reexport(&segs, Some(&alias), module, leading_colon, out);
            }
        }
        syn::UseTree::Glob(_) => {
            // The glob root: reacts iff it resolves in/under the forbidden set (the pipeline
            // decides). A sibling/ancestor root simply does not match — a stated glob bound.
            push_reexport(&prefix, Some("*"), module, leading_colon, out);
        }
        syn::UseTree::Group(group) => {
            for item in &group.items {
                walk_reexport_tree(item, prefix.clone(), module, leading_colon, out);
            }
        }
    }
}

/// A path segment's display name, raw-identifier prefix stripped (`r#type` → `type`), for the
/// human-facing exported name in the seam.
pub(crate) fn seg_name(ident: &syn::Ident) -> String {
    strip_raw(&ident.to_string())
}

/// Push a re-export exposure. The `syn::Path` is built **directly from the segment idents** (never
/// re-parsed from a string), so a raw-identifier segment (`pub use crate::r#type::X;`) is preserved
/// and matches correctly — `resolve_path`/`matches_forbidden` normalize raw idents downstream. The
/// seam is `pub use {module}::{exported}`. An empty segment list is skipped (a `self` under no
/// prefix cannot arise from a legal re-export).
pub(crate) fn push_reexport(
    segs: &[syn::Ident],
    exported: Option<&str>,
    module: &str,
    leading_colon: bool,
    out: &mut Vec<PathExposure>,
) {
    let (Some(exported), false) = (exported, segs.is_empty()) else {
        return;
    };
    let segments = segs
        .iter()
        .map(|ident| syn::PathSegment {
            ident: ident.clone(),
            arguments: syn::PathArguments::None,
        })
        .collect();
    out.push(PathExposure {
        // Preserve the `use` item's leading `::`: `pub use ::dep::X;` is an unambiguous extern
        // (resolved against the raw extern set by the query's leading-`::` branch), so it must stay
        // distinguishable from a bare `pub use dep::X;` — the latter is shadowed by a same-named
        // child `mod dep`, the former is not.
        path: syn::Path {
            leading_colon: leading_colon.then(<syn::Token![::]>::default),
            segments,
        },
        seam: PublicSeam::Reexport {
            module: module.to_string(),
            exported: exported.to_string(),
        },
        is_reexport: true,
    });
}

/// The paths named across a set of trait-bounds — each bound's trait path *and* any type nested
/// in its generic arguments (`T: From<crate::infra::Secret>` yields both `From` and
/// `crate::infra::Secret`). Used for the impl-site `where` position.
pub(crate) fn paths_in_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<syn::Path> {
    paths_in_bounds_scoped(bounds, &std::collections::HashSet::new())
}

/// Like [`paths_in_bounds`] but shadowing in-scope generic type parameters (see [`paths_in_type_scoped`]).
pub(crate) fn paths_in_bounds_scoped(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
    params: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut c = PathCollector::shadowing(params.clone());
    for bound in bounds {
        c.visit_type_param_bound(bound);
    }
    c.paths
}

/// The type paths in a signature's **return type** only (`sig.output`, never `sig.inputs`),
/// shadowing the enclosing item's + the signature's own generic type parameters so a bare return of
/// a parameter (`-> T`) is not misresolved through a same-named `use … as T` alias to a forbidden
/// type. A trait-impl method's params/receiver are trait-dictated (not refinable), but its return
/// MAY be refined at the impl site, so a concretely-written return can expose an impl-authored type.
pub(crate) fn paths_in_return_scoped(
    sig: &syn::Signature,
    enclosing: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut shadowed = enclosing.clone();
    shadowed.extend(type_param_names(&sig.generics));
    let mut c = PathCollector::shadowing(shadowed);
    if let syn::ReturnType::Type(_, ty) = &sig.output {
        c.visit_type(ty);
    }
    c.paths
}
