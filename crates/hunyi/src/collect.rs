//! The AST-collector cluster — the pure syntax-tree walkers that turn one parsed `syn::Item`
//! into the exposure findings each semantic rule reacts to. Every collector observes only the
//! public surface (via [`crate::syn_util::is_public`]), stamps each finding with its seam, and returns; the
//! reaction/decision lives in `lib.rs`. This module holds no state and makes no I/O.

use syn::visit::Visit;

use crate::finding::{
    PathExposure, SemanticFinding, field_seam, fn_seam, inherent_assoc_seam, inherent_method_seam,
    item_seam, member_label, render_sig_tail, tag_paths, trait_assoc_seam, trait_method_seam,
};
use crate::resolve::{
    DynCollector, ImplTraitCollector, PathCollector, ShapeExposure, UseMap, canonical_self_owner,
    path_to_string, stamp_seam, strip_raw, type_to_string,
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
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
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
fn impl_traits_in_return(sig: &syn::Signature) -> Vec<ShapeExposure> {
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
    ordinal: usize,
    out: &mut Vec<String>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            if item.sig.asyncness.is_some() {
                out.push(
                    SemanticFinding::AsyncFreeFn {
                        module: module.to_string(),
                        name: strip_raw(&item.sig.ident.to_string()),
                        tail: render_sig_tail(&item.sig),
                    }
                    .to_string(),
                );
            }
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            for trait_item in &item.items {
                if let syn::TraitItem::Fn(method) = trait_item {
                    if method.sig.asyncness.is_some() {
                        out.push(
                            SemanticFinding::AsyncTraitMethod {
                                module: module.to_string(),
                                trait_name: trait_name.clone(),
                                name: strip_raw(&method.sig.ident.to_string()),
                                tail: render_sig_tail(&method.sig),
                            }
                            .to_string(),
                        );
                    }
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            // Owner-qualify by the impl's canonical self type (via the shared `canonical_self_owner`,
            // as the other three collectors do) so `impl A`/`impl B` async methods of the same name
            // never collide under the (target, rule, finding) baseline (a false negative). Generics
            // stay distinct (`Foo<u8>` vs `Foo<u16>`); a self type with an unrenderable const-generic
            // expression is disambiguated by the impl's position, never collapsed.
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            for impl_item in &item.items {
                if let syn::ImplItem::Fn(method) = impl_item {
                    if is_public(&method.vis) && method.sig.asyncness.is_some() {
                        out.push(
                            SemanticFinding::AsyncInherentMethod {
                                owner: owner.clone(),
                                name: strip_raw(&method.sig.ident.to_string()),
                                tail: render_sig_tail(&method.sig),
                            }
                            .to_string(),
                        );
                    }
                }
            }
        }
        _ => {}
    }
}

/// The generic **type-parameter** names declared by `generics` — the names that, used bare, are
/// parameters rather than nominal types (so a same-named `type` alias must not resolve them).
fn type_param_names(generics: &syn::Generics) -> std::collections::HashSet<String> {
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
fn paths_in_signature(sig: &syn::Signature) -> Vec<syn::Path> {
    paths_in_signature_scoped(sig, &std::collections::HashSet::new())
}

/// Like [`paths_in_signature`] but also shadowing the **enclosing** item's generic type parameters
/// (an inherent-impl / trait's `<T>` is in scope inside its methods), so a method parameter named
/// like an enclosing param — or a same-module alias — is not misresolved.
fn paths_in_signature_scoped(
    sig: &syn::Signature,
    enclosing: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut shadowed = enclosing.clone();
    shadowed.extend(type_param_names(&sig.generics));
    let mut c = PathCollector::shadowing(shadowed);
    c.visit_signature(sig);
    c.paths
}

fn paths_in_type(ty: &syn::Type) -> Vec<syn::Path> {
    let mut c = PathCollector::default();
    c.visit_type(ty);
    c.paths
}

/// Paths in a type, shadowing the given in-scope generic type parameters — used where a type
/// position (a field, an alias target, an assoc item) sits inside a generic item whose params must
/// not be mistaken for nominal types.
fn paths_in_type_scoped(
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
fn paths_in_generics_scoped(
    generics: &syn::Generics,
    params: &std::collections::HashSet<String>,
) -> Vec<syn::Path> {
    let mut c = PathCollector::shadowing(params.clone());
    c.visit_generics(generics);
    c.paths
}

fn dyns_in_signature(sig: &syn::Signature) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_signature(sig);
    c.exposures
}

fn dyns_in_type(ty: &syn::Type) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_type(ty);
    c.exposures
}

fn dyns_in_generics(generics: &syn::Generics) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_generics(generics);
    c.exposures
}

/// The `dyn` trait-object shapes within a bound list (a trait's supertraits, or a public
/// associated type's `: Bound`s). The bound HEAD is a trait position (never a `dyn`), but a `dyn`
/// legally appears inside a bound's **generic argument** (`Facade: AsRef<Box<dyn crate::Port>>`),
/// so the walk must descend the bounds — the dyn-shape analogue of [`paths_in_bounds`].
fn dyns_in_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    for bound in bounds {
        c.visit_type_param_bound(bound);
    }
    c.exposures
}

/// Collect the type paths exposed by one item's public surface. Only `pub` items
/// contribute; `pub(crate)`/`pub(in …)`/private are internal, not exposed. Trait `impl`
/// blocks are skipped (out of scope — their shape is the trait's, not the impl site's).
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
                &item_seam("struct", module, &item.ident),
            ));
            for (index, field) in item.fields.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type_scoped(&field.ty, &params), &seam));
                }
            }
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &item_seam("enum", module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself. Each field
            // carries a per-member seam (`variant {Enum}::{Variant}::{index|name}`), mirroring
            // struct/union fields, so two forbidden fields of one variant stay distinct findings
            // — never collapsing to one `(target, rule, finding)` and masking a new leak.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                for (index, field) in variant.fields.iter().enumerate() {
                    let seam = field_seam("variant", module, &owner, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type_scoped(&field.ty, &params), &seam));
                }
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            let params = type_param_names(&item.generics);
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &params),
                &item_seam("union", module, &item.ident),
            ));
            for (index, field) in item.fields.named.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(tag_paths(paths_in_type_scoped(&field.ty, &params), &seam));
                }
            }
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam("type", module, &item.ident);
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
                &item_seam("const", module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(tag_paths(
                paths_in_type(&item.ty),
                &item_seam("static", module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            let trait_seam = item_seam("trait", module, &item.ident);
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
                        let seam = trait_assoc_seam("type", module, &trait_name, &assoc.ident);
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
                        let seam = trait_assoc_seam("const", module, &trait_name, &assoc.ident);
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
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            let impl_params = type_param_names(&item.generics);
            // The impl block's own generic-param bounds and where-clause are impl-site-authored
            // public contract for the inherent API (`impl<T: crate::infra::Secret> Foo<T> { … }`),
            // observed like a struct/enum/type def's generics (paths_in_generics_scoped) and the
            // trait-impl collector's where-walk. Owner-qualified so it stays distinct from the
            // block's methods / assoc items and from another block's generics.
            out.extend(tag_paths(
                paths_in_generics_scoped(&item.generics, &impl_params),
                &format!("impl <{owner}> (generics)"),
            ));
            for impl_item in &item.items {
                match impl_item {
                    // A public method's signature. The impl's own `<T>` is in scope inside it, so
                    // shadow it (plus the method's own params) to keep a param use from resolving
                    // through a same-named alias.
                    syn::ImplItem::Fn(method) if is_public(&method.vis) => {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
                        out.extend(tag_paths(
                            paths_in_signature_scoped(&method.sig, &impl_params),
                            &seam,
                        ));
                    }
                    // A public associated `const`'s declared type is public API (`Foo::K`).
                    syn::ImplItem::Const(assoc) if is_public(&assoc.vis) => {
                        let seam = inherent_assoc_seam("const", &owner, &assoc.ident);
                        out.extend(tag_paths(
                            paths_in_type_scoped(&assoc.ty, &impl_params),
                            &seam,
                        ));
                    }
                    // A public associated `type`'s target is public API (`Foo::T`).
                    syn::ImplItem::Type(assoc) if is_public(&assoc.vis) => {
                        let seam = inherent_assoc_seam("type", &owner, &assoc.ident);
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
                seam: format!("pub extern crate {name}"),
                path: syn::Path::from(item.ident.clone()),
                is_reexport: true,
            });
        }
        _ => {}
    }
}

/// Whether an ident is the `self` keyword-segment of a `use` tree (`{self, X}` / `self as alias`),
/// meaning "the prefix module itself". `self` is a keyword and never a raw identifier, so a string
/// compare is exact.
fn is_self_segment(ident: &syn::Ident) -> bool {
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
fn walk_reexport_tree(
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
fn seg_name(ident: &syn::Ident) -> String {
    strip_raw(&ident.to_string())
}

/// Push a re-export exposure. The `syn::Path` is built **directly from the segment idents** (never
/// re-parsed from a string), so a raw-identifier segment (`pub use crate::r#type::X;`) is preserved
/// and matches correctly — `resolve_path`/`matches_forbidden` normalize raw idents downstream. The
/// seam is `pub use {module}::{exported}`. An empty segment list is skipped (a `self` under no
/// prefix cannot arise from a legal re-export).
fn push_reexport(
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
        seam: format!("pub use {module}::{exported}"),
        is_reexport: true,
    });
}

/// The paths named across a set of trait-bounds — each bound's trait path *and* any type nested
/// in its generic arguments (`T: From<crate::infra::Secret>` yields both `From` and
/// `crate::infra::Secret`). Used for the impl-site `where` position.
fn paths_in_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<syn::Path> {
    paths_in_bounds_scoped(bounds, &std::collections::HashSet::new())
}

/// Like [`paths_in_bounds`] but shadowing in-scope generic type parameters (see [`paths_in_type_scoped`]).
fn paths_in_bounds_scoped(
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
fn paths_in_return_scoped(
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

/// Collect the type paths exposed by one **trait `impl` block**'s impl-site-authored positions
/// (`semantic-trait-impl-exposure`, opt-in). Only fires for `impl Trait for Type` (inherent impls
/// are `collect_item_exposures`'s job). The observed positions — each seam-qualified so two of them
/// exposing the same forbidden type stay distinct findings (the one forbidden bug) — are:
/// `trait-arg` (the trait ref's generic arguments, NOT the trait path itself: implementing a
/// forbidden *trait* is `must_not_acquire`/locality's concern), `self` (the Self type, bare and
/// nested), `assoc {name}` (associated type/value bindings), `where {bounded-type}` (the impl's own
/// generics + `where`-clause, keyed by the bounded type so two bounds never collapse), and
/// `method {name} return` (the written return type only — params/receiver are trait-dictated). The
/// pushed [`PathExposure`]s flow through the same resolve → canonicalize → match → `{type} exposed
/// by {seam}` pipeline as signature-coupling, with `BareFallback::Ignore` parity.
pub(crate) fn collect_trait_impl_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<PathExposure>,
) {
    let syn::Item::Impl(item) = item else { return };
    let Some((_, trait_path, _)) = &item.trait_ else {
        return; // inherent impl — governed by `collect_item_exposures`
    };
    // Seam prefix `impl {Trait} for {SelfTy}`. The Self label is canonicalized (parity with the
    // inherent-impl / locality seam owner); the trait label is the written path (a rendering-
    // granularity choice — its generic args distinguish `From<Vec<X>>` from `From<Box<X>>`).
    let trait_label = path_to_string(trait_path).unwrap_or_else(|| format!("trait_#{ordinal}"));
    let self_label = canonical_self_owner(&item.self_ty, uses, module, ordinal);
    let prefix = format!("impl {trait_label} for {self_label}");
    // The impl block's own generic type parameters are in scope in every position below; shadow
    // them so a bare parameter use is not misresolved through a same-named `use … as <param>` alias
    // to a forbidden type (parity with the inherent-impl / signature-coupling collector).
    let params = type_param_names(&item.generics);

    // 1. trait-arg — the trait ref's generic arguments (not the trait base path).
    if let Some(syn::PathArguments::AngleBracketed(args)) =
        trait_path.segments.last().map(|s| &s.arguments)
    {
        let seam = format!("{prefix} (trait-arg)");
        for arg in &args.args {
            match arg {
                syn::GenericArgument::Type(ty) => {
                    out.extend(tag_paths(paths_in_type_scoped(ty, &params), &seam))
                }
                syn::GenericArgument::AssocType(at) => {
                    out.extend(tag_paths(paths_in_type_scoped(&at.ty, &params), &seam))
                }
                _ => {}
            }
        }
    }

    // 2. self — the Self type, bare (`impl T for infra::Forbidden`) and nested
    //    (`impl T for Vec<infra::Forbidden>`). A bare `Self`/`Self::X` in a return (position 5)
    //    does not resolve and cannot double-fire here.
    out.extend(tag_paths(
        paths_in_type_scoped(&item.self_ty, &params),
        &format!("{prefix} (self)"),
    ));

    // 3. where — impl generic-param bounds and the `where`-clause, keyed by the bounded type so
    //    two distinct bounds exposing the same type never collapse under the baseline.
    for param in &item.generics.params {
        match param {
            syn::GenericParam::Type(tp) => {
                let key = strip_raw(&tp.ident.to_string());
                let seam = format!("{prefix} (where {key})");
                out.extend(tag_paths(
                    paths_in_bounds_scoped(&tp.bounds, &params),
                    &seam,
                ));
            }
            // A const-param's *type* annotation (`impl<const N: crate::infra::X>`) is impl-site-
            // authored, so this walk observes it too.
            syn::GenericParam::Const(cp) => {
                let key = strip_raw(&cp.ident.to_string());
                let seam = format!("{prefix} (where {key})");
                out.extend(tag_paths(paths_in_type_scoped(&cp.ty, &params), &seam));
            }
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    if let Some(where_clause) = &item.generics.where_clause {
        for predicate in &where_clause.predicates {
            if let syn::WherePredicate::Type(pt) = predicate {
                let key = type_to_string(&pt.bounded_ty).unwrap_or_else(|| "_".to_string());
                let seam = format!("{prefix} (where {key})");
                // Both sides are impl-site-authored: a forbidden type in the bounded (LHS) type
                // (`where crate::infra::X: Clone`) leaks as surely as one in the bound (RHS), so
                // the walk observes both.
                out.extend(tag_paths(
                    paths_in_type_scoped(&pt.bounded_ty, &params),
                    &seam,
                ));
                out.extend(tag_paths(
                    paths_in_bounds_scoped(&pt.bounds, &params),
                    &seam,
                ));
            }
        }
    }

    for impl_item in &item.items {
        match impl_item {
            // 4. assoc {name} — associated type/value bindings authored in the impl. Both an
            //    associated `type X = …` and an associated `const X: … ` carry an impl-site type.
            syn::ImplItem::Type(assoc) => {
                let seam = format!("{prefix} (assoc {})", strip_raw(&assoc.ident.to_string()));
                out.extend(tag_paths(paths_in_type_scoped(&assoc.ty, &params), &seam));
            }
            syn::ImplItem::Const(assoc) => {
                let seam = format!("{prefix} (assoc {})", strip_raw(&assoc.ident.to_string()));
                out.extend(tag_paths(paths_in_type_scoped(&assoc.ty, &params), &seam));
            }
            // 5. method {name} return — the written return type only (never params/receiver).
            //    Shadow the impl's params AND the method's own generics (`fn f<U>() -> U`).
            syn::ImplItem::Fn(method) => {
                let seam = format!(
                    "{prefix} (method {} return)",
                    strip_raw(&method.sig.ident.to_string())
                );
                out.extend(tag_paths(
                    paths_in_return_scoped(&method.sig, &params),
                    &seam,
                ));
            }
            _ => {}
        }
    }
}

/// Collect the `dyn` trait-object shapes exposed by one item's public surface — the
/// dyn-shape complement of [`collect_item_exposures`], over the same governed positions.
/// Kept **deliberately parallel, not merged**: signature-coupling pushes bare supertrait /
/// associated-bound *paths* (whose collected paths a shared visitor would change), and this
/// walk additionally observes associated-type **defaults** (`type T = Box<dyn …>;`), a
/// position exposure-governance does not cover. A bound's HEAD is a trait position (never a
/// `dyn`), but a `dyn` legally appears inside a bound's generic argument
/// (`Facade: AsRef<Box<dyn crate::Port>>`), so supertraits and associated-type bounds ARE walked
/// (via [`dyns_in_bounds`]), matching the sibling path collector.
pub(crate) fn collect_item_dyn_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<ShapeExposure>,
) {
    match item {
        syn::Item::Fn(item) if is_public(&item.vis) => {
            let seam = fn_seam(module, &item.sig.ident);
            out.extend(stamp_seam(dyns_in_signature(&item.sig), &seam));
        }
        syn::Item::Struct(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("struct", module, &item.ident),
            ));
            for (index, field) in item.fields.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("enum", module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself; per-member seam
            // for the same injectivity guarantee as the type-exposure collector above.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                for (index, field) in variant.fields.iter().enumerate() {
                    let seam = field_seam("variant", module, &owner, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam("union", module, &item.ident),
            ));
            for (index, field) in item.fields.named.iter().enumerate() {
                if is_public(&field.vis) {
                    let seam = field_seam("field", module, &name, &member_label(index, field));
                    out.extend(stamp_seam(dyns_in_type(&field.ty), &seam));
                }
            }
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam("type", module, &item.ident);
            out.extend(stamp_seam(dyns_in_generics(&item.generics), &seam));
            // A public type-alias target writing `dyn` is exposed at the alias item itself; a
            // public item that merely *names* this alias is not expanded (the resolver does
            // not expand `type` aliases — a stated bound).
            out.extend(stamp_seam(dyns_in_type(&item.ty), &seam));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam("const", module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam("static", module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            let trait_seam = item_seam("trait", module, &item.ident);
            out.extend(stamp_seam(dyns_in_generics(&item.generics), &trait_seam));
            // Supertraits are part of the trait's public contract. Their bound HEAD is a trait
            // position (never a `dyn`), but a `dyn` legally appears inside a supertrait bound's
            // generic argument (`Facade: AsRef<Box<dyn crate::Port>>`) — a real exposed trait-object
            // the sibling path collector already walks via paths_in_bounds. Match it here.
            out.extend(stamp_seam(dyns_in_bounds(&item.supertraits), &trait_seam));
            for trait_item in &item.items {
                match trait_item {
                    syn::TraitItem::Fn(method) => {
                        let seam = trait_method_seam(module, &trait_name, &method.sig.ident);
                        out.extend(stamp_seam(dyns_in_signature(&method.sig), &seam));
                    }
                    syn::TraitItem::Type(assoc) => {
                        let seam = trait_assoc_seam("type", module, &trait_name, &assoc.ident);
                        // A public associated type's `: Bound`s and GAT generics carry the same
                        // dyn-in-generic-argument exposure as a supertrait; its **default**
                        // (`type T = Box<dyn …>;`) is a plain exposed type position. All three are
                        // walked by the sibling path collector, so the dyn rule must not lag them.
                        out.extend(stamp_seam(dyns_in_bounds(&assoc.bounds), &seam));
                        out.extend(stamp_seam(dyns_in_generics(&assoc.generics), &seam));
                        if let Some((_, default)) = &assoc.default {
                            out.extend(stamp_seam(dyns_in_type(default), &seam));
                        }
                    }
                    syn::TraitItem::Const(assoc) => {
                        let seam = trait_assoc_seam("const", module, &trait_name, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    _ => {}
                }
            }
        }
        syn::Item::Impl(item) if item.trait_.is_none() => {
            let owner = canonical_self_owner(&item.self_ty, uses, module, ordinal);
            // A `dyn` written in the impl block's own generic-param bound or where-clause
            // (`impl<T: AsRef<Box<dyn crate::Port>>> Foo<T>`) is exposed on the inherent API — the
            // sibling path collector observes this position (via paths_in_generics_scoped), so the dyn rule
            // must not lag it. Parallel to the struct/enum/trait arms, which already walk generics.
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &format!("impl <{owner}> (generics)"),
            ));
            for impl_item in &item.items {
                match impl_item {
                    syn::ImplItem::Fn(method) if is_public(&method.vis) => {
                        let seam = inherent_method_seam(&owner, &method.sig.ident);
                        out.extend(stamp_seam(dyns_in_signature(&method.sig), &seam));
                    }
                    // A public associated `const`/`type` declares a public-API type position, so a
                    // `dyn` written there is exposed — the same positions the signature-coupling
                    // collector observes (`collect_item_exposures`); the dyn rule must not lag it.
                    syn::ImplItem::Const(assoc) if is_public(&assoc.vis) => {
                        let seam = inherent_assoc_seam("const", &owner, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    syn::ImplItem::Type(assoc) if is_public(&assoc.vis) => {
                        let seam = inherent_assoc_seam("type", &owner, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::{BareFallback, resolve_path};

    // Resolve every exposure path a public item produces, via the same segment-ident resolver the
    // query uses (`BareFallback::Ignore`), so a test can assert whether a forbidden `crate::…` type
    // is observed by the collector.
    fn resolved(item_src: &str, module: &str) -> Vec<String> {
        let item: syn::Item = syn::parse_str(item_src).unwrap();
        let uses = UseMap::new();
        let mut out = Vec::new();
        collect_item_exposures(&item, module, &uses, 0, &mut out);
        out.iter()
            .filter_map(|e| resolve_path(&e.path, &uses, module, BareFallback::Ignore))
            .collect()
    }

    fn exposes(item_src: &str, needle: &str) -> bool {
        resolved(item_src, "crate::domain")
            .iter()
            .any(|p| p == needle)
    }

    #[test]
    fn an_inherent_impl_public_assoc_const_and_type_are_observed() {
        // A forbidden type in a public inherent-impl associated `const`'s type or
        // `type` alias's target is now observed (was skipped — only methods were).
        assert!(
            exposes(
                "impl Foo { pub const K: crate::infra::Secret = todo!(); }",
                "crate::infra::Secret"
            ),
            "an inherent-impl pub const's type must expose crate::infra::Secret"
        );
        assert!(
            exposes(
                "impl Foo { pub type T = crate::infra::Secret; }",
                "crate::infra::Secret"
            ),
            "an inherent-impl pub type's target must expose crate::infra::Secret"
        );
    }

    #[test]
    fn a_non_public_inherent_assoc_item_is_not_exposed_but_a_pub_method_still_is() {
        // Only `pub` inherent assoc items are exposed; a private const/type is internal.
        assert!(
            !resolved(
                "impl Foo { const K: crate::infra::Secret = todo!(); type T = crate::infra::Secret; }",
                "crate::domain"
            )
            .iter()
            .any(|p| p.contains("crate::infra")),
            "a non-pub inherent assoc const/type must not be exposed"
        );
        // A public method's signature is still observed (the arm is unchanged).
        assert!(
            exposes(
                "impl Foo { pub fn make() -> crate::infra::Secret { todo!() } }",
                "crate::infra::Secret"
            ),
            "a pub inherent method signature is still observed"
        );
    }

    #[test]
    fn an_inherent_impl_generic_bound_is_observed() {
        // A forbidden type appearing only on the inherent impl's own generic-param bound
        // or where-clause is now observed — parity with the trait-impl collector's where-walk and
        // the struct/enum/type defs' `paths_in_generics_scoped` (both already observe this position).
        assert!(
            exposes(
                "impl<T: crate::infra::Secret> Foo<T> { pub fn m(&self) {} }",
                "crate::infra::Secret"
            ),
            "an inherent-impl generic-param bound must expose crate::infra::Secret"
        );
        assert!(
            exposes(
                "impl<T> Foo<T> where T: crate::infra::Secret { pub fn m(&self) {} }",
                "crate::infra::Secret"
            ),
            "an inherent-impl where-clause bound must expose crate::infra::Secret"
        );
    }

    #[test]
    fn a_supertrait_generic_argument_is_observed() {
        // Control: a struct field's generic arg was already observed.
        assert!(
            exposes(
                "pub struct S { pub f: Vec<crate::infra::Secret> }",
                "crate::infra::Secret"
            ),
            "control: a field generic arg must expose crate::infra::Secret"
        );
        // The fix: a supertrait bound's generic arg is now observed too (was silently dropped).
        assert!(
            exposes(
                "pub trait Facade: AsRef<crate::infra::Secret> {}",
                "crate::infra::Secret"
            ),
            "a supertrait bound's generic arg must expose crate::infra::Secret"
        );
    }

    #[test]
    fn an_assoc_type_bound_gat_param_and_default_are_observed() {
        assert!(
            exposes(
                "pub trait F { type Bar: Into<crate::infra::Secret>; }",
                "crate::infra::Secret"
            ),
            "an associated-type bound's generic arg must be observed"
        );
        assert!(
            exposes(
                "pub trait F { type Gat<T: crate::infra::Marker>; }",
                "crate::infra::Marker"
            ),
            "a GAT generic-parameter bound must be observed"
        );
        assert!(
            exposes(
                "pub trait F { type Bar = crate::infra::Secret; }",
                "crate::infra::Secret"
            ),
            "an associated-type default target must be observed"
        );
    }

    #[test]
    fn a_forbidden_supertrait_head_still_reacts_and_a_std_bound_does_not() {
        // No regression: a forbidden supertrait *head itself* is still observed.
        assert!(
            exposes(
                "pub trait Facade: crate::infra::SecretTrait {}",
                "crate::infra::SecretTrait"
            ),
            "a forbidden supertrait head must still react"
        );
        // An escape-free / std bound exposes no crate::infra.
        assert!(
            !resolved("pub trait Facade: Send + Sync {}", "crate::domain")
                .iter()
                .any(|p| p.contains("crate::infra")),
            "a std supertrait must not expose crate::infra"
        );
    }
}
