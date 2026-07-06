//! The `list` **projection** layer — rendering a declared `Constitution` to text, JSON, and
//! Markdown. Pure projection (PROJECT.md): it observes nothing and never reacts; the reaction
//! layer (`run`/`dispatch`/`gate`) lives in the parent `runner`. Split out so the projection
//! surface grows here as capabilities and formats are added, not inside the shell.

use crate::Constitution;

mod document;
mod gate;
mod markdown;
mod text;

pub(super) use document::list_document;
pub use gate::projection_gate;
pub(super) use markdown::list_markdown;
pub(super) use text::*;

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
