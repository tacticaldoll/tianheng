//! The `list` **projection** layer — rendering a declared `Constitution` to text, JSON, and
//! Markdown. Pure projection (PROJECT.md): it observes nothing and never reacts; the reaction
//! layer (`run`/`dispatch`/`gate`) lives in the parent `runner`. Split out so the projection
//! surface grows here as capabilities and formats are added, not inside the shell.

use guibiao::constitution_json;
use hunyi::{
    ASYNC_EXPOSURE_RULE, AsyncExposureBoundary, DYN_TRAIT_RULE, DynTraitBoundary,
    FORBIDDEN_MARKER_RULE, ForbiddenMarkerBoundary, IMPL_TRAIT_RULE, ImplTraitBoundary,
    SIGNATURE_RULE, SemanticBoundary, TRAIT_IMPL_RULE, TraitImplBoundary, VISIBILITY_RULE,
    VisibilityBoundary,
};
use louke::{RUNTIME_SEAM_RULE, RuntimeBoundary};
use serde_json::Value;

use crate::Constitution;

/// The text projection of the semantic boundaries, appended to the `list` output. Empty when
/// there are none, so a static-only project's `list` output is unchanged.
pub(super) fn semantic_text(boundaries: &[SemanticBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Semantic {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        // The opt-in deepening changes the reaction, so the projected law must show it.
        let opt_in = if boundary.including_trait_impls() {
            " (including trait impls)"
        } else {
            ""
        };
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}: {}{}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            SIGNATURE_RULE,
            boundary.forbidden().join(", "),
            opt_in,
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one semantic boundary, mirroring a static boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`) plus the `forbidden` set.
pub(super) fn semantic_boundary_json(boundary: &SemanticBoundary) -> Value {
    let mut object = serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": SIGNATURE_RULE,
        "severity": boundary.severity().as_str(),
        "forbidden": boundary.forbidden(),
        "reason": boundary.reason(),
    });
    // Emit the opt-in only when set, so a bare boundary's JSON (and the Markdown derived from it via
    // `boundary_params`) stays byte-unchanged; when set, Markdown surfaces it generically.
    if boundary.including_trait_impls() {
        object["including_trait_impls"] = serde_json::json!(true);
    }
    object
}

/// The text projection of the trait-impl-locality boundaries, appended to `list`. Empty when
/// there are none, so a project not using the dimension sees unchanged output.
pub(super) fn trait_impl_text(boundaries: &[TraitImplBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Trait-impl-locality {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] trait {} in {}\n  rule:   {} (declared: {})\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.trait_(),
            boundary.crate_package(),
            TRAIT_IMPL_RULE,
            boundary.allowed_locations().join(", "),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one trait-impl-locality boundary, mirroring the others' shape
/// (`kind`, `target` = the trait, `crate`, `rule`, `severity`, `reason`) plus the
/// `allowed_locations` set.
pub(super) fn trait_impl_boundary_json(boundary: &TraitImplBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.trait_(),
        "crate": boundary.crate_package(),
        "rule": TRAIT_IMPL_RULE,
        "severity": boundary.severity().as_str(),
        "allowed_locations": boundary.allowed_locations(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the visibility boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
pub(super) fn visibility_text(boundaries: &[VisibilityBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Visibility {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            VISIBILITY_RULE,
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one visibility boundary, mirroring the others' shape (`kind`,
/// `target` = the module, `crate`, `rule`, `severity`, `reason`).
pub(super) fn visibility_boundary_json(boundary: &VisibilityBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": VISIBILITY_RULE,
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the forbidden-marker boundaries, appended to `list`. Empty when
/// there are none, so a project not using the dimension sees unchanged output.
pub(super) fn forbidden_marker_text(boundaries: &[ForbiddenMarkerBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Forbidden-marker {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] subtree {} in {}\n  rule:   {}: {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            FORBIDDEN_MARKER_RULE,
            boundary.forbidden().join(", "),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one forbidden-marker boundary (`kind`, `target` = the subtree,
/// `crate`, `rule`, `severity`, `reason`) plus the `forbidden` trait set.
pub(super) fn forbidden_marker_boundary_json(boundary: &ForbiddenMarkerBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": FORBIDDEN_MARKER_RULE,
        "severity": boundary.severity().as_str(),
        "forbidden": boundary.forbidden(),
        "reason": boundary.reason(),
    })
}

/// The text projection of the dyn-trait boundaries, appended to `list`. Empty when there are
/// none, so a project not using the dimension sees unchanged output.
pub(super) fn dyn_trait_text(boundaries: &[DynTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Dyn-trait {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            shape_rule_text(DYN_TRAIT_RULE, boundary.forbidden_operands()),
            boundary.reason(),
        ));
    }
    out
}

/// The text rule line for a shape/existential boundary: the bare shape rule when shape-only, or
/// `… of: A, B` when operand-scoped — so `list --format text` surfaces the operand set the JSON
/// and markdown projections already carry (parity across the three `list` formats).
pub(super) fn shape_rule_text(rule: &str, operands: &[String]) -> String {
    if operands.is_empty() {
        rule.to_string()
    } else {
        format!("{rule} of: {}", operands.join(", "))
    }
}

pub(super) fn impl_trait_text(boundaries: &[ImplTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Impl-trait {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            shape_rule_text(IMPL_TRAIT_RULE, boundary.forbidden_operands()),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one dyn-trait boundary, mirroring a semantic boundary's shape (`kind`,
/// `target`, `crate`, `rule`, `severity`, `reason`). An operand-scoped boundary additionally
/// carries the `forbidden` operand set; a shape-only boundary (empty set) emits no such field.
pub(super) fn dyn_trait_boundary_json(boundary: &DynTraitBoundary) -> Value {
    let mut object = serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": DYN_TRAIT_RULE,
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    });
    // The operand set surfaces only for an operand-scoped boundary; a shape-only boundary
    // (empty set) projects unchanged, with no `forbidden` param.
    let operands = boundary.forbidden_operands();
    if !operands.is_empty() {
        object["forbidden"] = serde_json::json!(operands);
    }
    object
}

pub(super) fn impl_trait_boundary_json(boundary: &ImplTraitBoundary) -> Value {
    let mut object = serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": IMPL_TRAIT_RULE,
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    });
    // The operand set surfaces only for an operand-scoped boundary; a shape-only boundary
    // (empty set) projects unchanged, with no `forbidden` param.
    let operands = boundary.forbidden_operands();
    if !operands.is_empty() {
        object["forbidden"] = serde_json::json!(operands);
    }
    object
}

pub(super) fn async_exposure_text(boundaries: &[AsyncExposureBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Async-exposure {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            ASYNC_EXPOSURE_RULE,
            boundary.reason(),
        ));
    }
    out
}

pub(super) fn async_exposure_boundary_json(boundary: &AsyncExposureBoundary) -> Value {
    serde_json::json!({
        "kind": "semantic",
        "target": boundary.module(),
        "crate": boundary.crate_package(),
        "rule": ASYNC_EXPOSURE_RULE,
        "severity": boundary.severity().as_str(),
        "reason": boundary.reason(),
    })
}

/// The `list --format json` document: the static constitution's projection augmented with one
/// array per non-empty dimension, so the document covers every declared law and never silently
/// omits one. A dimension with no boundaries adds no key (a static-only project's document is
/// byte-identical to before the other dimensions existed).
pub(super) fn list_document(constitution: &Constitution) -> Value {
    let semantic = constitution.semantic_boundaries();
    let runtime = constitution.runtime_boundaries();
    let mut document: Value =
        serde_json::from_str(&constitution_json(constitution.static_boundaries()))
            .expect("constitution_json emits a valid document");
    if !semantic.signature.is_empty() {
        document["semantic_boundaries"] = Value::Array(
            semantic
                .signature
                .iter()
                .map(semantic_boundary_json)
                .collect(),
        );
    }
    if !semantic.trait_impl.is_empty() {
        document["trait_impl_boundaries"] = Value::Array(
            semantic
                .trait_impl
                .iter()
                .map(trait_impl_boundary_json)
                .collect(),
        );
    }
    if !semantic.visibility.is_empty() {
        document["visibility_boundaries"] = Value::Array(
            semantic
                .visibility
                .iter()
                .map(visibility_boundary_json)
                .collect(),
        );
    }
    if !semantic.forbidden_marker.is_empty() {
        document["forbidden_marker_boundaries"] = Value::Array(
            semantic
                .forbidden_marker
                .iter()
                .map(forbidden_marker_boundary_json)
                .collect(),
        );
    }
    if !semantic.dyn_trait.is_empty() {
        document["dyn_trait_boundaries"] = Value::Array(
            semantic
                .dyn_trait
                .iter()
                .map(dyn_trait_boundary_json)
                .collect(),
        );
    }
    if !semantic.impl_trait.is_empty() {
        document["impl_trait_boundaries"] = Value::Array(
            semantic
                .impl_trait
                .iter()
                .map(impl_trait_boundary_json)
                .collect(),
        );
    }
    if !semantic.async_exposure.is_empty() {
        document["async_exposure_boundaries"] = Value::Array(
            semantic
                .async_exposure
                .iter()
                .map(async_exposure_boundary_json)
                .collect(),
        );
    }
    if !runtime.is_empty() {
        document["runtime_boundaries"] =
            Value::Array(runtime.iter().map(runtime_boundary_json).collect());
    }
    document
}

/// The text projection of the runtime (漏刻) boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
///
/// #16 doc note: this seam-origin rule reacts at **runtime** (the prod `assert_boundary!` face),
/// NOT at `check` time. `check`'s runtime face is only the probe-coverage audit (does every
/// declared seam have a probe?) — it never observes a live crossing. So an agent reading this
/// `list` entry must not expect `check` to react to an origin crossing the seam; the origin
/// reaction happens in the running binary. The rule label is the canonical `RUNTIME_SEAM_RULE`
/// (the same const `check_crossing` renders), with the allowed-origin set as a per-boundary detail.
pub(super) fn runtime_text(boundaries: &[RuntimeBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let noun = if boundaries.len() == 1 {
        "boundary"
    } else {
        "boundaries"
    };
    let mut out = format!("Runtime {noun} ({}):\n", boundaries.len());
    for boundary in boundaries {
        // The rule label is driven from the canonical `RUNTIME_SEAM_RULE` const (shared with the
        // JSON projection and the prod reaction), so the text and JSON no longer drift; the
        // allowed-origin set is the per-boundary detail, like the dyn/impl operand set.
        out.push_str(&format!(
            "\n[{}] seam {} (reacts at runtime, not at check)\n  rule:    {} (only origins: {})\n  posture: {}\n  reason:  {}\n",
            boundary.severity().as_str(),
            boundary.seam(),
            RUNTIME_SEAM_RULE,
            boundary.allowed_origins().join(", "),
            boundary.posture().as_str(),
            boundary.reason(),
        ));
    }
    out
}

/// The JSON projection of one runtime boundary (`kind` = runtime, `target` = the seam, `rule`,
/// `severity`, `posture`, `reason`) plus the `allowed_origins` set. `posture` is projected so a
/// `panic_on_violation` boundary does not project identically to a default event-only one.
///
/// #16 doc note: the projected `rule` reacts at **runtime** (the prod `assert_boundary!` face),
/// not at `check` — `check`'s runtime face is only probe-coverage. The label is the canonical
/// `RUNTIME_SEAM_RULE` const, shared with the text projection and the prod `check_crossing`.
pub(super) fn runtime_boundary_json(boundary: &RuntimeBoundary) -> Value {
    serde_json::json!({
        "kind": "runtime",
        "target": boundary.seam(),
        "rule": RUNTIME_SEAM_RULE,
        "severity": boundary.severity().as_str(),
        "posture": boundary.posture().as_str(),
        "allowed_origins": boundary.allowed_origins(),
        "reason": boundary.reason(),
    })
}

/// Render a constitution as the human- and agent-readable Markdown summary of its declared law —
/// the same projection `list --format markdown` prints, returned as a `String` for library
/// callers (e.g. to generate an agent-context artifact). It composes the same internal projector,
/// so it carries no less than the JSON and never reacts; it adds nothing of its own (no preamble,
/// no trailing newline), so it equals the CLI output byte for byte.
///
/// **Format stability.** This Markdown layout is intended for display, review, and LLM context.
/// It is **not** a machine-stable contract and **may evolve in any compatible release** to improve
/// readability or imitability (e.g. foregrounding a boundary's `reason`). Consumers that need a
/// stable, machine-parseable projection MUST use the JSON projection (`list --format json`)
/// instead — depending on the exact Markdown shape is unsupported.
///
/// ```
/// use tianheng::prelude::*;
/// let c = Constitution::new("my-project").boundary(
///     CrateBoundary::crate_("my-core")
///         .deny_external_dependencies()
///         .because("my-core stays dependency-light"),
/// );
/// let md = tianheng::constitution_markdown(&c);
/// assert!(md.contains("# Constitution: my-project"));
/// assert!(md.contains("my-core stays dependency-light"));
/// // Write it where an agent will read it, e.g.:
/// // std::fs::write("AGENTS.my-project-law.md", md)?;
/// ```
pub fn constitution_markdown(constitution: &Constitution) -> String {
    list_markdown(&list_document(constitution))
}

/// The `list --format markdown` projection: an agent-readable summary of the *whole* declared
/// law. It is rendered from the very [`Value`] [`list_document`] emits, so it provably carries
/// no information absent from the JSON and covers exactly the same dimensions (the spec's
/// "no less than the JSON" guarantee holds by construction, not by parallel maintenance). Like
/// `list` as a whole it observes nothing and never reacts. A dimension with no declared
/// boundaries contributes no section, mirroring the text and JSON projections.
pub(super) fn list_markdown(document: &Value) -> String {
    let name = document
        .get("constitution")
        .and_then(Value::as_str)
        .unwrap_or("(unnamed)");
    let mut out = format!("# Constitution: {name}\n");
    // The dimension sections in projection order; each key matches `list_document`'s, and a
    // section absent or empty there is skipped here, so the two projections stay in lockstep.
    for (key, heading) in [
        ("boundaries", "Static boundaries"),
        (
            "semantic_boundaries",
            "Semantic boundaries (signature-coupling)",
        ),
        ("trait_impl_boundaries", "Trait-impl-locality boundaries"),
        ("visibility_boundaries", "Visibility boundaries"),
        ("forbidden_marker_boundaries", "Forbidden-marker boundaries"),
        ("dyn_trait_boundaries", "Dyn-trait boundaries"),
        ("impl_trait_boundaries", "Impl-trait boundaries"),
        ("async_exposure_boundaries", "Async-exposure boundaries"),
        ("runtime_boundaries", "Runtime boundaries"),
    ] {
        let Some(Value::Array(items)) = document.get(key) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("\n## {heading}\n"));
        for item in items {
            out.push_str(&boundary_markdown(item));
        }
    }
    out
}

/// One boundary as a Markdown block, with the declared `reason` **foregrounded**: the `target`
/// is the heading; then — when present — the `reason` as a leading blockquote (the block's
/// principle, set apart from the mechanical metadata); then the `rule` with its parameters (the
/// reaction's mechanical shape); then the kind/severity classification, and the owning crate for a
/// module boundary. Every field is read from the JSON projection, so an agent reads the same law
/// the JSON carries.
///
/// The reason leads deliberately (see PROJECT.md, 潛移): it is the gravity-bearing content a model
/// imitates and the repair hint on a violation. The only layout property pinned is this ordering
/// (reason → rule → classification); the exact rendering stays free to evolve under Contract B (see
/// [`constitution_markdown`]). A boundary with no reason emits no blockquote and no orphan blank line.
pub(super) fn boundary_markdown(boundary: &Value) -> String {
    let field = |key: &str| boundary.get(key).and_then(Value::as_str).unwrap_or("");
    let mut out = format!("\n### `{}`\n", field("target"));

    let reason = field("reason");
    if !reason.is_empty() {
        out.push_str(&format!("\n> {reason}\n\n"));
    }

    out.push_str(&format!("- **rule**: {}", field("rule")));
    let params = boundary_params(boundary);
    if !params.is_empty() {
        out.push_str(&format!(" ({params})"));
    }
    out.push('\n');

    let mut context = format!("- **kind**: {}", field("kind"));
    let severity = field("severity");
    if !severity.is_empty() {
        context.push_str(&format!(" · **severity**: {severity}"));
    }
    if let Some(krate) = boundary.get("crate").and_then(Value::as_str) {
        context.push_str(&format!(" · **crate**: {krate}"));
    }
    out.push_str(&context);
    out.push('\n');
    out
}

/// The rule parameters of a boundary — every JSON field that is not one of the structural keys
/// (kind/target/crate/rule/severity/reason) — rendered inline. This generically surfaces each
/// dimension's specifics (a forbidden set, allowed locations, allowed origins, a posture, a
/// dependency kind) without hard-coding any dimension, so a new dimension's parameters appear
/// in the Markdown the moment they appear in the JSON.
pub(super) fn boundary_params(boundary: &Value) -> String {
    const STRUCTURAL: [&str; 6] = ["kind", "target", "crate", "rule", "severity", "reason"];
    let Some(object) = boundary.as_object() else {
        return String::new();
    };
    object
        .iter()
        .filter(|(key, _)| !STRUCTURAL.contains(&key.as_str()))
        .map(|(key, value)| format!("{key}: {}", inline_value(value)))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Render a JSON value compactly for a Markdown parameter: a string as itself, an array as a
/// comma-joined list, a scalar via its display, an object via its JSON text. Each rendering is
/// a pure function of the value, so the projection is stable and diffable; within a boundary,
/// `boundary_params` walks the object in serde_json's default `Map` order — lexicographic by
/// key (a `BTreeMap`), not declaration order — which is likewise deterministic.
pub(super) fn inline_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(inline_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => value.to_string(),
    }
}
