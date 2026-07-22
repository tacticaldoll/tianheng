use guibiao::constitution_json;
use hunyi::{
    ASYNC_EXPOSURE_RULE, AsyncExposureBoundary, DYN_TRAIT_RULE, DynTraitBoundary,
    FORBIDDEN_MARKER_RULE, ForbiddenMarkerBoundary, IMPL_TRAIT_RULE, ImplTraitBoundary,
    SIGNATURE_RULE, SemanticBoundary, TRAIT_IMPL_RULE, TraitImplBoundary, UNSAFE_CONFINEMENT_RULE,
    UnsafeBoundary, VisibilityBoundary,
};
use louke::{RUNTIME_SEAM_RULE, RuntimeBoundary};
use serde_json::Value;

use crate::Constitution;

/// Attach a boundary's durable governance `anchor` to its projected JSON, only when set — so a
/// boundary without one keeps byte-identical projection (and the Markdown derived from it via
/// `boundary_params`), the same discipline the operand / `including_trait_impls` params already use.
fn anchored(mut object: Value, anchor: Option<&str>) -> Value {
    if let Some(anchor) = anchor {
        object["anchor"] = serde_json::json!(anchor);
    }
    object
}
fn boundary_json_base(
    kind: &str,
    target: &str,
    krate: Option<&str>,
    rule: &str,
    severity: &str,
    reason: &str,
    anchor: Option<&str>,
) -> Value {
    let mut object = serde_json::json!({
        "kind": kind,
        "target": target,
        "rule": rule,
        "severity": severity,
        "reason": reason,
    });
    if let Some(krate) = krate {
        object["crate"] = serde_json::json!(krate);
    }
    anchored(object, anchor)
}
fn semantic_module_json(
    module: &str,
    krate: &str,
    rule: &str,
    severity: &str,
    reason: &str,
    anchor: Option<&str>,
) -> Value {
    boundary_json_base(
        "semantic",
        module,
        Some(krate),
        rule,
        severity,
        reason,
        anchor,
    )
}
/// The JSON projection of one semantic boundary, mirroring a static boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`) plus the `forbidden` set.
pub(in crate::runner) fn semantic_boundary_json(boundary: &SemanticBoundary) -> Value {
    let mut object = semantic_module_json(
        boundary.module(),
        boundary.crate_package(),
        SIGNATURE_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    object["forbidden"] = serde_json::json!(boundary.forbidden());
    // Emit the opt-in only when set, so a bare boundary's JSON (and the Markdown derived from it via
    // `boundary_params`) stays byte-unchanged; when set, Markdown surfaces it generically.
    if boundary.including_trait_impls() {
        object["including_trait_impls"] = serde_json::json!(true);
    }
    object
}
/// The JSON projection of one trait-impl-locality boundary, mirroring the others' shape
/// (`kind`, `target` = the trait, `crate`, `rule`, `severity`, `reason`) plus the
/// `allowed_locations` set.
pub(in crate::runner) fn trait_impl_boundary_json(boundary: &TraitImplBoundary) -> Value {
    let mut object = boundary_json_base(
        "semantic",
        boundary.trait_(),
        Some(boundary.crate_package()),
        TRAIT_IMPL_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    object["allowed_locations"] = serde_json::json!(boundary.allowed_locations());
    object
}
/// The JSON projection of one visibility boundary, mirroring the others' shape (`kind`,
/// `target` = the module, `crate`, `rule`, `severity`, `reason`).
pub(in crate::runner) fn visibility_boundary_json(boundary: &VisibilityBoundary) -> Value {
    semantic_module_json(
        boundary.module(),
        boundary.crate_package(),
        boundary.ceiling().rule(),
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    )
}
/// The JSON projection of one forbidden-marker boundary (`kind`, `target` = the subtree,
/// `crate`, `rule`, `severity`, `reason`) plus the `forbidden` trait set.
pub(in crate::runner) fn forbidden_marker_boundary_json(
    boundary: &ForbiddenMarkerBoundary,
) -> Value {
    let mut object = semantic_module_json(
        boundary.module(),
        boundary.crate_package(),
        FORBIDDEN_MARKER_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    object["forbidden"] = serde_json::json!(boundary.forbidden());
    object
}
/// Shared body for an operand-scoped shape boundary (dyn-trait or impl-trait): builds the
/// `semantic_module_json` base and emits `"forbidden"` only when the operand set is non-empty —
/// a shape-only boundary (empty set) projects unchanged, with no `forbidden` param.
fn shape_operand_boundary_json(
    module: &str,
    krate: &str,
    rule: &str,
    severity: &str,
    reason: &str,
    anchor: Option<&str>,
    operands: &[String],
) -> Value {
    let mut object = semantic_module_json(module, krate, rule, severity, reason, anchor);
    if !operands.is_empty() {
        object["forbidden"] = serde_json::json!(operands);
    }
    object
}
/// The JSON projection of one dyn-trait boundary, mirroring a semantic boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`). An operand-scoped boundary additionally
/// carries the `forbidden` operand set; a shape-only boundary (empty set) emits no such field.
pub(in crate::runner) fn dyn_trait_boundary_json(boundary: &DynTraitBoundary) -> Value {
    shape_operand_boundary_json(
        boundary.module(),
        boundary.crate_package(),
        DYN_TRAIT_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
        boundary.forbidden_operands(),
    )
}
pub(in crate::runner) fn impl_trait_boundary_json(boundary: &ImplTraitBoundary) -> Value {
    shape_operand_boundary_json(
        boundary.module(),
        boundary.crate_package(),
        IMPL_TRAIT_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
        boundary.forbidden_operands(),
    )
}
pub(in crate::runner) fn async_exposure_boundary_json(boundary: &AsyncExposureBoundary) -> Value {
    let mut object = semantic_module_json(
        boundary.module(),
        boundary.crate_package(),
        ASYNC_EXPOSURE_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    // The subtree opt-in changes the reaction (whole subtree vs the anchored seam), so the
    // projected law must show it. Emitted only when set, so a bare boundary's JSON (and the
    // Markdown derived from it) stays byte-identical.
    if boundary.including_submodules() {
        object["including_submodules"] = serde_json::json!(true);
    }
    object
}
/// The JSON projection of one unsafe-confinement boundary (`kind`, `target` = the confined crate,
/// `crate`, `rule`, `severity`, `reason`) plus the `allowed_locations` subtree set.
pub(in crate::runner) fn unsafe_boundary_json(boundary: &UnsafeBoundary) -> Value {
    let mut object = boundary_json_base(
        "semantic",
        boundary.crate_package(),
        Some(boundary.crate_package()),
        UNSAFE_CONFINEMENT_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    object["allowed_locations"] = serde_json::json!(boundary.allowed_locations());
    object
}
fn append_array<T>(document: &mut Value, key: &str, items: &[T], project: impl Fn(&T) -> Value) {
    if !items.is_empty() {
        document[key] = Value::Array(items.iter().map(project).collect());
    }
}
/// The `list --format json` document: the static constitution's projection augmented with one
/// array per non-empty dimension, so the document covers every declared law and never silently
/// omits one. A dimension with no boundaries adds no key (a static-only project's document is
/// byte-identical to before the other dimensions existed).
pub(in crate::runner) fn list_document(constitution: &Constitution) -> Value {
    let semantic = constitution.semantic_boundaries();
    let runtime = constitution.runtime_boundaries();
    let mut document: Value =
        serde_json::from_str(&constitution_json(constitution.static_boundaries()))
            .expect("constitution_json emits a valid document");
    append_array(
        &mut document,
        "semantic_boundaries",
        &semantic.signature,
        semantic_boundary_json,
    );
    append_array(
        &mut document,
        "trait_impl_boundaries",
        &semantic.trait_impl,
        trait_impl_boundary_json,
    );
    append_array(
        &mut document,
        "visibility_boundaries",
        &semantic.visibility,
        visibility_boundary_json,
    );
    append_array(
        &mut document,
        "forbidden_marker_boundaries",
        &semantic.forbidden_marker,
        forbidden_marker_boundary_json,
    );
    append_array(
        &mut document,
        "dyn_trait_boundaries",
        &semantic.dyn_trait,
        dyn_trait_boundary_json,
    );
    append_array(
        &mut document,
        "impl_trait_boundaries",
        &semantic.impl_trait,
        impl_trait_boundary_json,
    );
    append_array(
        &mut document,
        "async_exposure_boundaries",
        &semantic.async_exposure,
        async_exposure_boundary_json,
    );
    append_array(
        &mut document,
        "unsafe_confinement_boundaries",
        &semantic.unsafe_confinement,
        unsafe_boundary_json,
    );
    append_array(
        &mut document,
        "runtime_boundaries",
        runtime,
        runtime_boundary_json,
    );
    document
}
/// The JSON projection of one runtime boundary (`kind` = runtime, `target` = the seam, `rule`,
/// `severity`, `posture`, `reason`) plus the `allowed_origins` set. `posture` is projected so a
/// `panic_on_violation` boundary does not project identically to a default event-only one.
///
/// The projected `rule` reacts at **runtime** (the prod `assert_boundary!` face),
/// not at `check` — `check`'s runtime face is only probe-coverage. The label is the canonical
/// `RUNTIME_SEAM_RULE` const, shared with the text projection and the prod `check_crossing`.
pub(in crate::runner) fn runtime_boundary_json(boundary: &RuntimeBoundary) -> Value {
    let mut object = boundary_json_base(
        "runtime",
        boundary.seam(),
        None,
        RUNTIME_SEAM_RULE,
        boundary.severity().as_str(),
        boundary.reason(),
        boundary.anchor(),
    );
    object["posture"] = serde_json::json!(boundary.posture().as_str());
    object["allowed_origins"] = serde_json::json!(boundary.allowed_origins());
    object
}
