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

/// Projection data for one module-scoped boundary text block. The `target` field encodes the
/// kind-specific prefix ("module X in Y", "trait X in Y", "subtree X in Y", "crate X"), so a
/// single `render_section` skeleton handles all eight boundary types — the format string is
/// written once, not eight times.
struct ModuleBlockSpec<'a> {
    severity: &'a str,
    target: String,
    rule_line: String,
    reason: &'a str,
    anchor: Option<&'a str>,
}

/// The single render skeleton shared by all module-scoped boundary projections. `runtime_text`
/// is deliberately excluded — its extra `posture:` field makes it a permanent exception.
fn render_section(title: &str, blocks: &[ModuleBlockSpec<'_>]) -> String {
    if blocks.is_empty() {
        return String::new();
    }
    let mut out = text_section(title, blocks.len());
    for b in blocks {
        out.push_str(&format!(
            "\n[{}] {}\n  rule:   {}\n  reason: {}\n",
            b.severity, b.target, b.rule_line, b.reason
        ));
        out.push_str(&anchor_line(b.anchor));
    }
    out
}

/// The text projection of the semantic boundaries.
pub(in crate::runner) fn semantic_text(boundaries: &[SemanticBoundary]) -> String {
    render_section(
        "Semantic",
        &boundaries
            .iter()
            .map(|b| {
                let opt_in = if b.including_trait_impls() {
                    " (including trait impls)"
                } else {
                    ""
                };
                ModuleBlockSpec {
                    severity: b.severity().as_str(),
                    target: format!("module {} in {}", b.module(), b.crate_package()),
                    rule_line: format!(
                        "{}: {}{}",
                        SIGNATURE_RULE,
                        b.forbidden().join(", "),
                        opt_in
                    ),
                    reason: b.reason(),
                    anchor: b.anchor(),
                }
            })
            .collect::<Vec<_>>(),
    )
}
/// The text projection of the trait-impl-locality boundaries.
pub(in crate::runner) fn trait_impl_text(boundaries: &[TraitImplBoundary]) -> String {
    render_section(
        "Trait-impl-locality",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("trait {} in {}", b.trait_(), b.crate_package()),
                rule_line: format!(
                    "{} (declared: {})",
                    TRAIT_IMPL_RULE,
                    b.allowed_locations().join(", ")
                ),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
}
/// The text projection of the visibility boundaries.
pub(in crate::runner) fn visibility_text(boundaries: &[VisibilityBoundary]) -> String {
    render_section(
        "Visibility",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("module {} in {}", b.module(), b.crate_package()),
                rule_line: b.ceiling().rule().to_string(),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
}
/// The text projection of the forbidden-marker boundaries.
pub(in crate::runner) fn forbidden_marker_text(boundaries: &[ForbiddenMarkerBoundary]) -> String {
    render_section(
        "Forbidden-marker",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("subtree {} in {}", b.module(), b.crate_package()),
                rule_line: format!("{}: {}", FORBIDDEN_MARKER_RULE, b.forbidden().join(", ")),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
}
/// The text projection of the dyn-trait boundaries.
pub(in crate::runner) fn dyn_trait_text(boundaries: &[DynTraitBoundary]) -> String {
    render_section(
        "Dyn-trait",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("module {} in {}", b.module(), b.crate_package()),
                rule_line: shape_rule_text(DYN_TRAIT_RULE, b.forbidden_operands()),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
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
    render_section(
        "Impl-trait",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("module {} in {}", b.module(), b.crate_package()),
                rule_line: shape_rule_text(IMPL_TRAIT_RULE, b.forbidden_operands()),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
}
pub(in crate::runner) fn async_exposure_text(boundaries: &[AsyncExposureBoundary]) -> String {
    render_section(
        "Async-exposure",
        &boundaries
            .iter()
            .map(|b| {
                // The subtree opt-in changes the reaction, so the projected law shows it (parity
                // with the JSON/Markdown projections); a bare boundary's text stays byte-identical.
                let scope = if b.including_submodules() {
                    " (including submodules)"
                } else {
                    ""
                };
                ModuleBlockSpec {
                    severity: b.severity().as_str(),
                    target: format!("module {} in {}", b.module(), b.crate_package()),
                    rule_line: format!("{}{}", ASYNC_EXPOSURE_RULE, scope),
                    reason: b.reason(),
                    anchor: b.anchor(),
                }
            })
            .collect::<Vec<_>>(),
    )
}
/// The text projection of the unsafe-confinement boundaries.
pub(in crate::runner) fn unsafe_text(boundaries: &[UnsafeBoundary]) -> String {
    render_section(
        "Unsafe-confinement",
        &boundaries
            .iter()
            .map(|b| ModuleBlockSpec {
                severity: b.severity().as_str(),
                target: format!("crate {}", b.crate_package()),
                rule_line: format!(
                    "{} (allowed: {})",
                    UNSAFE_CONFINEMENT_RULE,
                    b.allowed_locations().join(", ")
                ),
                reason: b.reason(),
                anchor: b.anchor(),
            })
            .collect::<Vec<_>>(),
    )
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
