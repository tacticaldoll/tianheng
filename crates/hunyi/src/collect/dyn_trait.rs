use syn::visit::Visit;

use super::exposure::*;
use crate::finding::*;
use crate::resolve::*;
use crate::syn_util::*;
pub(crate) fn dyns_in_signature(sig: &syn::Signature) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_signature(sig);
    c.exposures
}

pub(crate) fn dyns_in_type(ty: &syn::Type) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_type(ty);
    c.exposures
}

pub(crate) fn dyns_in_generics(generics: &syn::Generics) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    c.visit_generics(generics);
    c.exposures
}

/// The `dyn` trait-object shapes within a bound list (a trait's supertraits, or a public
/// associated type's `: Bound`s). The bound HEAD is a trait position (never a `dyn`), but a `dyn`
/// legally appears inside a bound's **generic argument** (`Facade: AsRef<Box<dyn crate::Port>>`),
/// so the walk must descend the bounds — the dyn-shape analogue of [`paths_in_bounds`].
pub(crate) fn dyns_in_bounds(
    bounds: &syn::punctuated::Punctuated<syn::TypeParamBound, syn::token::Plus>,
) -> Vec<ShapeExposure> {
    let mut c = DynCollector::default();
    for bound in bounds {
        c.visit_type_param_bound(bound);
    }
    c.exposures
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
                &item_seam(ItemKind::Struct, module, &item.ident),
            ));
            collect_named_field_exposures(
                item.fields.iter(),
                MemberKind::Field,
                module,
                &name,
                |field| is_public(&field.vis),
                |ty, seam| stamp_seam(dyns_in_type(ty), seam),
                out,
            );
        }
        syn::Item::Enum(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam(ItemKind::Enum, module, &item.ident),
            ));
            // Enum variants and their fields are as public as the enum itself; per-member seam
            // for the same injectivity guarantee as the type-exposure collector above.
            for variant in &item.variants {
                let owner = format!("{name}::{}", strip_raw(&variant.ident.to_string()));
                collect_named_field_exposures(
                    variant.fields.iter(),
                    MemberKind::Variant,
                    module,
                    &owner,
                    |_| true,
                    |ty, seam| stamp_seam(dyns_in_type(ty), seam),
                    out,
                );
            }
        }
        syn::Item::Union(item) if is_public(&item.vis) => {
            let name = strip_raw(&item.ident.to_string());
            out.extend(stamp_seam(
                dyns_in_generics(&item.generics),
                &item_seam(ItemKind::Union, module, &item.ident),
            ));
            collect_named_field_exposures(
                item.fields.named.iter(),
                MemberKind::Field,
                module,
                &name,
                |field| is_public(&field.vis),
                |ty, seam| stamp_seam(dyns_in_type(ty), seam),
                out,
            );
        }
        syn::Item::Type(item) if is_public(&item.vis) => {
            let seam = item_seam(ItemKind::Type, module, &item.ident);
            out.extend(stamp_seam(dyns_in_generics(&item.generics), &seam));
            // A public type-alias target writing `dyn` is exposed at the alias item itself; a
            // public item that merely *names* this alias is not expanded (the resolver does
            // not expand `type` aliases — a stated bound).
            out.extend(stamp_seam(dyns_in_type(&item.ty), &seam));
        }
        syn::Item::Const(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam(ItemKind::Const, module, &item.ident),
            ));
        }
        syn::Item::Static(item) if is_public(&item.vis) => {
            out.extend(stamp_seam(
                dyns_in_type(&item.ty),
                &item_seam(ItemKind::Static, module, &item.ident),
            ));
        }
        syn::Item::Trait(item) if is_public(&item.vis) => {
            let trait_name = strip_raw(&item.ident.to_string());
            let trait_seam = item_seam(ItemKind::Trait, module, &item.ident);
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
                        let seam =
                            trait_assoc_seam(AssocKind::Type, module, &trait_name, &assoc.ident);
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
                        let seam =
                            trait_assoc_seam(AssocKind::Const, module, &trait_name, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    _ => {}
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
            // A `dyn` written in the impl block's own generic-param bound or where-clause
            // (`impl<T: AsRef<Box<dyn crate::Port>>> Foo<T>`) is exposed on the inherent API — the
            // sibling path collector observes this position (via paths_in_generics_scoped), so the dyn rule
            // must not lag it. Parallel to the struct/enum/trait arms, which already walk generics.
            // Module-qualified for the same reason the sibling method seam below is: two inherent
            // impl blocks for one owner may be written in two different modules.
            // Per-position, keyed by the bounded thing — the same shared walk the sibling path
            // collector uses, for the same identity reason (see `PublicSeam::InherentGenerics`).
            for (bound, positions) in impl_generics_positions(&item.generics, ordinal) {
                let seam = PublicSeam::InherentGenerics {
                    module: module.to_string(),
                    owner: owner.clone(),
                    bound,
                };
                for position in positions {
                    let dyns = match position {
                        GenericsPosition::Bounds(bounds) => dyns_in_bounds(bounds),
                        GenericsPosition::Type(ty) => dyns_in_type(ty),
                    };
                    out.extend(stamp_seam(dyns, &seam));
                }
            }
            for impl_item in &item.items {
                match impl_item {
                    syn::ImplItem::Fn(method) if is_public(&method.vis) => {
                        let seam = inherent_method_seam(module, &owner, &method.sig.ident);
                        out.extend(stamp_seam(dyns_in_signature(&method.sig), &seam));
                    }
                    // A public associated `const`/`type` declares a public-API type position, so a
                    // `dyn` written there is exposed — the same positions the signature-coupling
                    // collector observes (`collect_item_exposures`); the dyn rule must not lag it.
                    syn::ImplItem::Const(assoc) if is_public(&assoc.vis) => {
                        let seam =
                            inherent_assoc_seam(AssocKind::Const, module, &owner, &assoc.ident);
                        out.extend(stamp_seam(dyns_in_type(&assoc.ty), &seam));
                    }
                    syn::ImplItem::Type(assoc) if is_public(&assoc.vis) => {
                        let seam =
                            inherent_assoc_seam(AssocKind::Type, module, &owner, &assoc.ident);
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
    use crate::resolve::{BareFallback, resolve_path_all};

    // Resolve every exposure path a public item produces, via the same segment-ident resolver the
    // query uses (`BareFallback::Ignore`), so a test can assert whether a forbidden `crate::…` type
    // is observed by the collector. Every candidate is kept, exactly as the query's own matching
    // does — these fixtures declare no aliases, so each path yields at most one.
    fn resolved(item_src: &str, module: &str) -> Vec<String> {
        let item: syn::Item = syn::parse_str(item_src).unwrap();
        let uses = UseMap::new();
        let mut out = Vec::new();
        collect_item_exposures(&item, module, &uses, 0, &mut out);
        out.iter()
            .flat_map(|e| resolve_path_all(&e.path, &uses, module, BareFallback::Ignore))
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
