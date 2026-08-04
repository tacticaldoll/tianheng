use super::exposure::*;
use crate::finding::*;
use crate::resolve::*;
use crate::syn_util::{GenericsPosition, impl_generics_positions};
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
    //    two distinct bounds exposing the same type never collapse under the baseline. The positions
    //    and their keys come from the shared `impl_generics_positions`, which the inherent-impl
    //    generics seams also use: three collectors keying one syntax shape must not each carry their
    //    own copy of where the positions are or how they are named. An unrenderable bounded type's
    //    positional sentinel lives there too, so it cannot drift between them either.
    for (key, positions) in impl_generics_positions(&item.generics, ordinal) {
        let seam = seam(TraitImplPosition::Where(key));
        for position in positions {
            let paths = match position {
                GenericsPosition::Bounds(bounds) => paths_in_bounds_scoped(bounds, &params),
                GenericsPosition::Type(ty) => paths_in_type_scoped(ty, &params),
            };
            out.extend(tag_paths(paths, &seam));
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
