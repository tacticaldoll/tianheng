use std::path::PathBuf;

use xuanji::{BoundaryKind, Polarity, RuleKey, Severity, Violation, ViolationId};

use crate::finding::SemanticFact;

pub(crate) struct SingleModuleViolationContext<'a> {
    pub(crate) module: &'a str,
    pub(crate) rule: &'a str,
    pub(crate) rule_key: RuleKey,
    pub(crate) reason: &'a str,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<&'a str>,
}

/// Add deny-style violations for a boundary whose findings all sit on one governed module seam.
/// Each finding carries the real file its own item's branch was resolved from (see
/// [`crate::module_resolve::resolve_module_items_with_files`]) — never a single, first-branch file
/// for the whole module, which would misattribute a finding produced by a non-first `#[cfg]`-split
/// branch (a real defect found on a round-5 adversarial review; see `PROJECT.md`'s Decisions).
/// Every capability supplies `(target, rule key, structured fact)` identity; presentation, file,
/// anchor, and polarity remain metadata.
pub(crate) fn push_single_module_violations(
    violations: &mut Vec<Violation>,
    context: SingleModuleViolationContext<'_>,
    findings: Vec<(SemanticFact, PathBuf)>,
) {
    let anchor = context.anchor.map(str::to_string);
    for (finding, file) in findings {
        let finding = finding.into_finding();
        let id = ViolationId::new(
            context.module,
            context.rule_key.clone(),
            finding.key().clone(),
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                id,
                context.rule,
                finding.text(),
                context.reason.to_string(),
                context.severity,
            )
            .with_file(Some(file.display().to_string()))
            .with_anchor(anchor.clone())
            .with_polarity(Polarity::DenyBreach),
        );
    }
}

pub(crate) struct MultiModuleViolationContext<'a> {
    /// The violation `target` — the boundary's anchored module, kept stable so identity
    /// `(target, rule, finding_key)` does not shift as the governed subtree grows.
    pub(crate) target: &'a str,
    pub(crate) rule: &'a str,
    pub(crate) rule_key: RuleKey,
    pub(crate) reason: &'a str,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<&'a str>,
    /// The finding's polarity metadata (deny-breach vs allowlist-gap). Not part of the violation
    /// identity, so each capability passes its own without shifting structured identity.
    pub(crate) polarity: Polarity,
}

/// Add violations for a boundary whose findings sit across many modules — the shared emitter for
/// every whole-crate-scan capability (forbidden-marker, trait-impl, unsafe-confinement, and the
/// async-exposure subtree branch), of either polarity: each caller supplies its own `polarity` via
/// the context. Each finding carries its enclosing module (metadata, never part of the identity)
/// AND the real file that module's own branch was resolved from, collected directly at the site
/// (`ImplSite`/`TypeDef`/`UnsafeSite`, or the subtree walker's own per-branch file) rather than
/// re-resolved afterward by module string — a re-resolution keyed only by the module string
/// misattributes a finding whenever two `#[cfg]`-split branches share one module path (the same
/// finding as [`push_single_module_violations`]'s doc, found one hop further downstream on a
/// round-5 adversarial review; see `PROJECT.md`'s Decisions). The violation `target` stays the
/// boundary's anchor, so a finding's structured identity is stable.
pub(crate) fn push_multi_module_violations(
    violations: &mut Vec<Violation>,
    context: MultiModuleViolationContext<'_>,
    findings: Vec<(SemanticFact, String, PathBuf)>,
) {
    let anchor = context.anchor.map(str::to_string);
    for (finding, _module, file) in findings {
        let finding = finding.into_finding();
        let id = ViolationId::new(
            context.target,
            context.rule_key.clone(),
            finding.key().clone(),
        );
        violations.push(
            Violation::new(
                BoundaryKind::Semantic,
                id,
                context.rule,
                finding.text(),
                context.reason.to_string(),
                context.severity,
            )
            .with_file(Some(file.display().to_string()))
            .with_anchor(anchor.clone())
            .with_polarity(context.polarity),
        );
    }
}
