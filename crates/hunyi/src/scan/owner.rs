//! The owner an item nested in an `impl` is named by: the impl's self type, resolved to one
//! canonical path where the module's uses or local types name it, otherwise rendered as written.
//! Shared by every reader that qualifies a site by its enclosing impl — unsafe confinement and the
//! static-item boundary — so one self type is one owner in both.

use std::collections::HashSet;

use crate::resolve::*;

/// The canonical owner of an `impl` block's self type in `module`.
///
/// A path whose head the module can resolve — `crate`, `self`, `super`, a leading `::`, a `use`d
/// name or a local type — resolves to its one canonical target, with the last segment's generic
/// arguments rendered after it; any other self type is rendered as written. A head bound to more
/// than one target by mutually exclusive `#[cfg]` branches has no one owner and is refused, as is a
/// self type with no supported rendering: a label that names scan position is not an identity.
pub(crate) fn canonical_self_type_owner(
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
