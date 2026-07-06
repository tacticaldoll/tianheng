use hunyi::{
    ASYNC_EXPOSURE_RULE, AsyncExposureBoundary, DYN_TRAIT_RULE, DynTraitBoundary,
    FORBIDDEN_MARKER_RULE, ForbiddenMarkerBoundary, IMPL_TRAIT_RULE, ImplTraitBoundary,
    SIGNATURE_RULE, SemanticBoundary, TRAIT_IMPL_RULE, TraitImplBoundary, VISIBILITY_RULE,
    VisibilityBoundary,
};
use louke::{RUNTIME_SEAM_RULE, RuntimeBoundary};

/// Start a text projection section. Empty boundary sets return an empty string, so projects not
/// using a dimension keep byte-identical `list` output.
fn text_section(title: &str, count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    let noun = if count == 1 { "boundary" } else { "boundaries" };
    format!("{title} {noun} ({count}):\n")
}
/// The text projection of the semantic boundaries.
pub(in crate::runner) fn semantic_text(boundaries: &[SemanticBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Semantic", boundaries.len());
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
/// The text projection of the trait-impl-locality boundaries.
pub(in crate::runner) fn trait_impl_text(boundaries: &[TraitImplBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Trait-impl-locality", boundaries.len());
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
/// The text projection of the visibility boundaries.
pub(in crate::runner) fn visibility_text(boundaries: &[VisibilityBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Visibility", boundaries.len());
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
/// The text projection of the forbidden-marker boundaries.
pub(in crate::runner) fn forbidden_marker_text(boundaries: &[ForbiddenMarkerBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Forbidden-marker", boundaries.len());
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
/// The text projection of the dyn-trait boundaries.
pub(in crate::runner) fn dyn_trait_text(boundaries: &[DynTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Dyn-trait", boundaries.len());
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
pub(in crate::runner) fn shape_rule_text(rule: &str, operands: &[String]) -> String {
    if operands.is_empty() {
        rule.to_string()
    } else {
        format!("{rule} of: {}", operands.join(", "))
    }
}
pub(in crate::runner) fn impl_trait_text(boundaries: &[ImplTraitBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Impl-trait", boundaries.len());
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
pub(in crate::runner) fn async_exposure_text(boundaries: &[AsyncExposureBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Async-exposure", boundaries.len());
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
/// The text projection of the runtime (漏刻) boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
///
/// #16 doc note: this seam-origin rule reacts at **runtime** (the prod `assert_boundary!` face),
/// NOT at `check` time. `check`'s runtime face is only the probe-coverage audit (does every
/// declared seam have a probe?) — it never observes a live crossing. So an agent reading this
/// `list` entry must not expect `check` to react to an origin crossing the seam; the origin
/// reaction happens in the running binary. The rule label is the canonical `RUNTIME_SEAM_RULE`
/// (the same const `check_crossing` renders), with the allowed-origin set as a per-boundary detail.
pub(in crate::runner) fn runtime_text(boundaries: &[RuntimeBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Runtime", boundaries.len());
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
