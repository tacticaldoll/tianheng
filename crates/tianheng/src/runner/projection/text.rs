use hunyi::{
    ASYNC_EXPOSURE_RULE, AsyncExposureBoundary, DYN_TRAIT_RULE, DynTraitBoundary,
    FORBIDDEN_MARKER_RULE, ForbiddenMarkerBoundary, IMPL_TRAIT_RULE, ImplTraitBoundary,
    SIGNATURE_RULE, SemanticBoundary, TRAIT_IMPL_RULE, TraitImplBoundary, UNSAFE_CONFINEMENT_RULE,
    UnsafeBoundary, VisibilityBoundary,
};
use louke::{RuntimeBoundary, runtime_seam_rule_line};

/// Start a text projection section. Empty boundary sets return an empty string, so projects not
/// using a dimension keep byte-identical `list` output.
fn text_section(title: &str, count: usize) -> String {
    if count == 0 {
        return String::new();
    }
    let noun = if count == 1 { "boundary" } else { "boundaries" };
    format!("{title} {noun} ({count}):\n")
}
/// A trailing `anchor:` line for a boundary's text block — emitted only when the boundary carries a
/// durable governance anchor, so a boundary without one keeps byte-identical `list` text. Closes the
/// three-format parity gap where the anchor appeared in the JSON and Markdown projections but not the
/// text one (the same emit-when-set discipline they use).
fn anchor_line(anchor: Option<&str>) -> String {
    match anchor {
        Some(anchor) => format!("  anchor: {anchor}\n"),
        None => String::new(),
    }
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
        out.push_str(&anchor_line(boundary.anchor()));
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
        out.push_str(&anchor_line(boundary.anchor()));
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
            boundary.ceiling().rule(),
            boundary.reason(),
        ));
        out.push_str(&anchor_line(boundary.anchor()));
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
        out.push_str(&anchor_line(boundary.anchor()));
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
        out.push_str(&anchor_line(boundary.anchor()));
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
        out.push_str(&anchor_line(boundary.anchor()));
    }
    out
}
pub(in crate::runner) fn async_exposure_text(boundaries: &[AsyncExposureBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Async-exposure", boundaries.len());
    for boundary in boundaries {
        // The subtree opt-in changes the reaction, so the projected law shows it (parity with the
        // JSON/Markdown projections); a bare boundary's text stays byte-identical.
        let scope = if boundary.including_submodules() {
            " (including submodules)"
        } else {
            ""
        };
        out.push_str(&format!(
            "\n[{}] module {} in {}\n  rule:   {}{}\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.module(),
            boundary.crate_package(),
            ASYNC_EXPOSURE_RULE,
            scope,
            boundary.reason(),
        ));
        out.push_str(&anchor_line(boundary.anchor()));
    }
    out
}
/// The text projection of the unsafe-confinement boundaries.
pub(in crate::runner) fn unsafe_text(boundaries: &[UnsafeBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Unsafe-confinement", boundaries.len());
    for boundary in boundaries {
        out.push_str(&format!(
            "\n[{}] crate {}\n  rule:   {} (allowed: {})\n  reason: {}\n",
            boundary.severity().as_str(),
            boundary.crate_package(),
            UNSAFE_CONFINEMENT_RULE,
            boundary.allowed_locations().join(", "),
            boundary.reason(),
        ));
        out.push_str(&anchor_line(boundary.anchor()));
    }
    out
}
/// The text projection of the runtime (漏刻) boundaries, appended to `list`. Empty when there
/// are none, so a project not using the dimension sees unchanged output.
///
/// This seam-origin rule reacts at **runtime** (the prod `assert_boundary!` face),
/// NOT at `check` time. `check`'s runtime face is only the probe-coverage audit (does every
/// declared seam have a probe?) — it never observes a live crossing. So an agent reading this
/// `list` entry must not expect `check` to react to an origin crossing the seam; the origin
/// reaction happens in the running binary. The rule line comes from `louke::runtime_seam_rule_line`
/// — the same formatter `check_crossing` renders — so the text projection and the prod reaction
/// share one folded rule string (label + allowed-origin detail), never a hand-copied twin.
pub(in crate::runner) fn runtime_text(boundaries: &[RuntimeBoundary]) -> String {
    if boundaries.is_empty() {
        return String::new();
    }
    let mut out = text_section("Runtime", boundaries.len());
    for boundary in boundaries {
        // The full rule line comes from `louke::runtime_seam_rule_line` — the SAME formatter the
        // prod reaction (`check_crossing`) uses — so the folded `… (only origins: …)` wording is
        // written once, never hand-copied here (the twin-drift bug class). The folded style matches
        // the dyn/impl text projection; the JSON projection keeps the label bare with origins as a
        // field.
        out.push_str(&format!(
            "\n[{}] seam {} (reacts at runtime, not at check)\n  rule:    {}\n  posture: {}\n  reason:  {}\n",
            boundary.severity().as_str(),
            boundary.seam(),
            runtime_seam_rule_line(boundary.allowed_origins()),
            boundary.posture().as_str(),
            boundary.reason(),
        ));
        out.push_str(&anchor_line(boundary.anchor()));
    }
    out
}
