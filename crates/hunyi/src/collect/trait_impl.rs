use super::exposure::*;
use crate::finding::*;
use crate::resolve::*;
/// Collect the type paths exposed by one **trait `impl` block**'s impl-site-authored positions
/// (`semantic-trait-impl-exposure`, opt-in). Only fires for `impl Trait for Type` (inherent impls
/// are `collect_item_exposures`'s job). See that spec's "Impl-site-authored positions govern
/// trait-impl exposure" requirement for the full position list (`trait-arg`, `self`, `assoc
/// {name}`, `where {bounded-type}`, `method {name} return`) and its "Position-qualified seam
/// identity prevents baseline masking" requirement for why each is seam-qualified. The pushed
/// [`PathExposure`]s flow through the same resolve → canonicalize → match → `{type} exposed by
/// {seam}` pipeline as signature-coupling, with `BareFallback::Ignore` parity.
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
    // granularity choice — its generic args distinguish `From<Vec<X>>` from `From<Box<X>>`). An
    // unrenderable path carries an internal sentinel rejected before fact emission.
    let trait_label = path_to_string(trait_path).unwrap_or_else(|| format!("trait_#{ordinal}"));
    // The impl block's own generic type parameters are in scope in every position below; shadow
    // them so a bare parameter use is not misresolved through a same-named `use … as <param>` alias
    // to a forbidden type (parity with the inherent-impl / signature-coupling collector) — including
    // in the Self label itself, computed next.
    let params = type_param_names(&item.generics);
    let self_label = canonical_self_owner(&item.self_ty, uses, module, ordinal, &params);
    let seam = |position: TraitImplPosition| PublicSeam::TraitImpl {
        trait_ref: trait_label.clone(),
        owner: self_label.clone(),
        position,
    };

    // 1. trait-arg — the trait ref's generic arguments (not the trait base path).
    if let Some(syn::PathArguments::AngleBracketed(args)) =
        trait_path.segments.last().map(|s| &s.arguments)
    {
        let seam = seam(TraitImplPosition::TraitArg);
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
        &seam(TraitImplPosition::SelfType),
    ));

    // 3. where — impl generic-param bounds and the `where`-clause, keyed by the bounded type so
    //    two distinct bounds exposing the same type never collapse under the baseline — including
    //    when the bounded type cannot be rendered, where the where-predicate loop below fails
    //    loud instead of falling back to a key two such bounds could share.
    for param in &item.generics.params {
        match param {
            syn::GenericParam::Type(tp) => {
                let key = strip_raw(&tp.ident.to_string());
                let seam = seam(TraitImplPosition::Where(key));
                out.extend(tag_paths(
                    paths_in_bounds_scoped(&tp.bounds, &params),
                    &seam,
                ));
            }
            // A const-param's *type* annotation (`impl<const N: crate::infra::X>`) is impl-site-
            // authored, so this walk observes it too.
            syn::GenericParam::Const(cp) => {
                let key = strip_raw(&cp.ident.to_string());
                let seam = seam(TraitImplPosition::Where(key));
                out.extend(tag_paths(paths_in_type_scoped(&cp.ty, &params), &seam));
            }
            syn::GenericParam::Lifetime(_) => {}
        }
    }
    if let Some(where_clause) = &item.generics.where_clause {
        for (bound_ordinal, predicate) in where_clause.predicates.iter().enumerate() {
            if let syn::WherePredicate::Type(pt) = predicate {
                // A bounded type that cannot be rendered (a complex const-generic argument, e.g.
                // `Arr<{ N + 1 }>` — `path_to_string`'s generic-argument rendering is all-or-
                // nothing, so one unrenderable argument fails the whole path) MUST NOT fall back
                // to the bare literal `_`: two such bounds in ONE impl block would then share
                // that key, and their facts — identical kind, subject, AND seam — would collapse
                // to one, silently losing the second bound's violation (the identity-collision
                // this position's "never collapse" guarantee forbids). Mirror the sibling
                // `trait_label` fallback above and `canonical_self_owner`'s own unrenderable case:
                // an internal positional sentinel, composed of the item's own `ordinal` (unique
                // per impl block, continuous across the module) and this predicate's own position
                // within THIS impl block's where-clause (`bound_ordinal`, so two unrenderable
                // bounds in the SAME impl block never share a sentinel either). The sentinel is
                // never published: every public observation path routes it through the shared
                // `reject_positional_identity` gate, so unsupported syntax fails loud instead of
                // silently colliding.
                let key = type_to_string(&pt.bounded_ty)
                    .unwrap_or_else(|| format!("_#{ordinal}.{bound_ordinal}"));
                let seam = seam(TraitImplPosition::Where(key));
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
                let seam = seam(TraitImplPosition::Assoc(strip_raw(
                    &assoc.ident.to_string(),
                )));
                out.extend(tag_paths(paths_in_type_scoped(&assoc.ty, &params), &seam));
            }
            syn::ImplItem::Const(assoc) => {
                let seam = seam(TraitImplPosition::Assoc(strip_raw(
                    &assoc.ident.to_string(),
                )));
                out.extend(tag_paths(paths_in_type_scoped(&assoc.ty, &params), &seam));
            }
            // 5. method {name} return — the written return type only (never params/receiver).
            //    Shadow the impl's params AND the method's own generics (`fn f<U>() -> U`).
            syn::ImplItem::Fn(method) => {
                let seam = seam(TraitImplPosition::MethodReturn(strip_raw(
                    &method.sig.ident.to_string(),
                )));
                out.extend(tag_paths(
                    paths_in_return_scoped(&method.sig, &params),
                    &seam,
                ));
            }
            _ => {}
        }
    }
}
