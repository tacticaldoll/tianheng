use super::exposure::*;
use crate::finding::*;
use crate::resolve::*;
use crate::syn_util::{GenericsPosition, impl_generics_positions};
/// Collect the type paths exposed by one **trait `impl` block**'s impl-site-authored positions
/// (`semantic-trait-impl-exposure`, opt-in). Only fires for `impl Trait for Type` (inherent impls
/// are `collect_item_exposures`'s job).
///
/// Impl positions collected:
/// 1. `trait-arg` — generic arguments of the trait reference.
/// 2. `self` — the Self type (bare or nested).
/// 3. `where` — impl generic-param bounds and where-clauses keyed by bounded type via `impl_generics_positions`.
/// 4. `assoc` — associated type/const bindings authored in the impl.
/// 5. `method return` — written return types of methods authored in the impl (shadowing method generics).
///
/// Impl generic type parameters are shadowed so bare parameter uses are not misresolved through in-scope aliases.
/// The pushed [`PathExposure`]s flow through the same resolve → canonicalize → match pipeline as signature-coupling.
pub(crate) fn collect_trait_impl_exposures(
    item: &syn::Item,
    module: &str,
    uses: &UseMap,
    ordinal: usize,
    out: &mut Vec<PathExposure>,
) {
    let syn::Item::Impl(item) = item else { return };
    let Some((_, trait_path, _)) = &item.trait_ else {
        return;
    };
    let trait_label = path_to_string(trait_path).unwrap_or_else(|| format!("trait_#{ordinal}"));
    let params = type_param_names(&item.generics);
    let self_label = canonical_self_owner(&item.self_ty, uses, module, ordinal, &params);
    let seam = |position: TraitImplPosition| PublicSeam::TraitImpl {
        trait_ref: trait_label.clone(),
        owner: self_label.clone(),
        position,
    };

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

    out.extend(tag_paths(
        paths_in_type_scoped(&item.self_ty, &params),
        &seam(TraitImplPosition::SelfType),
    ));

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
