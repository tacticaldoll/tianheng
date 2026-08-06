//! A house rule 三儀 does not cover, added as a **participant** rather than as a patch to the family.
//!
//! The rule: every module file in a governed subtree opens with a `//!` header, so a reader learns what a
//! file is for before reading it. No dimension of 三儀 governs that — it is neither a dependency edge, nor an
//! exposed type, nor a runtime origin. The protocol's whole promise is that such a rule can join a run instead
//! of waiting for the family to grow a feature for it.
//!
//! Written against `tianheng::prelude` and nothing else.

use std::path::Path;

use tianheng::prelude::*;

/// The rule label and its repair direction, as a reader sees them in the report.
const RULE: &str = "every module file opens with a `//!` header";
const REASON: &str = "a reader opening a file learns what it is for before reading it";

/// Every `.rs` file directly in a governed subtree must open with a `//!` module header.
pub struct ModuleHeaderObserver {
    /// The subtrees this participant reads, relative to the crate root.
    ///
    /// **Configuration, not a constant** — and that is why this participant's bound ids cannot be literals.
    /// Which bounds it has depends on what it was told to read, so they are built with `format!` at the
    /// moment it is asked. `BoundId`'s owned-or-borrowed form is what makes that expressible at all.
    subtrees: Vec<String>,
}

impl ModuleHeaderObserver {
    /// A participant reading the given subtrees, each relative to the crate root.
    pub fn reading<I, S>(subtrees: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            subtrees: subtrees.into_iter().map(Into::into).collect(),
        }
    }
}

impl Observer for ModuleHeaderObserver {
    /// Read each configured subtree one level deep and report every file with no header.
    ///
    /// Anything this participant was told to read and **could not** is a constitution error — exit 2 — never a
    /// silent pass. A participant that reports clean because it failed to look is the one bug the family's
    /// contract forbids outright, and joining a run does not exempt an outsider from it.
    fn observe(&self, manifest_path: &Path) -> Outcome {
        let Some(root) = manifest_path.parent() else {
            return Outcome::ConstitutionError(format!(
                "manifest '{}' has no parent directory, so no subtree can be located",
                manifest_path.display()
            ));
        };
        let mut violations = Vec::new();
        for subtree in &self.subtrees {
            let directory = root.join(subtree);
            let entries = match std::fs::read_dir(&directory) {
                Ok(entries) => entries,
                Err(error) => {
                    return Outcome::ConstitutionError(format!(
                        "cannot read governed subtree '{}': {error}",
                        directory.display()
                    ));
                }
            };
            for entry in entries {
                let path = match entry {
                    Ok(entry) => entry.path(),
                    Err(error) => {
                        return Outcome::ConstitutionError(format!(
                            "cannot enumerate '{}': {error}",
                            directory.display()
                        ));
                    }
                };
                if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                    continue;
                }
                let text = match std::fs::read_to_string(&path) {
                    Ok(text) => text,
                    Err(error) => {
                        return Outcome::ConstitutionError(format!(
                            "cannot read '{}': {error}",
                            path.display()
                        ));
                    }
                };
                if text
                    .lines()
                    .next()
                    .is_some_and(|line| line.trim_start().starts_with("//!"))
                {
                    continue;
                }
                let label = path
                    .strip_prefix(root)
                    .unwrap_or(&path)
                    .display()
                    .to_string();
                violations.push(
                    Violation::new(
                        // Borrowed from 三儀's vocabulary, and the nearest honest fit rather than a right
                        // one: `BoundaryKind` has no value a participant owns, so an outsider's violation
                        // must claim one of the family's four. Recorded as a finding of this example in its
                        // README rather than worked around here.
                        BoundaryKind::Module,
                        ViolationId::new(
                            subtree.clone(),
                            RuleKey::of(
                                "house-rules.rule/module-header",
                                [("subtree", subtree.as_str())],
                            ),
                            StructuredFactIdentity::of(
                                "house-rules.fact/module-header",
                                "missing-header",
                                [("file", label.as_str())],
                            ),
                        ),
                        RULE,
                        label.clone(),
                        REASON.to_string(),
                        Severity::Enforce,
                    )
                    .with_file(Some(label)),
                );
            }
        }
        if violations.is_empty() {
            Outcome::Clean
        } else {
            Outcome::Violations(Report::new(violations))
        }
    }

    /// What this participant does **not** observe, computed from what it was configured to read.
    ///
    /// One bound per governed subtree, because the answer genuinely depends on configuration: a participant
    /// told to read two subtrees stops at two different places. Every string here is built at run time —
    /// the id, the shape, the reason and the pin — which is the case `BoundDecl`'s owned-or-borrowed strings
    /// were added for and which nothing in the family itself exercises, since every family declaration is a
    /// literal.
    fn bounds(&self) -> Vec<BoundDecl> {
        self.subtrees
            .iter()
            .flat_map(|subtree| {
                let slug = subtree.replace(['/', '-', '.'], "_");
                [
                    BoundDecl::new(
                        BoundId::new(format!(
                            "house-rules/a-file-nested-below-{subtree}-is-out-of-reach"
                        )),
                        format!("a `.rs` file in a directory below `{subtree}`"),
                        Extent::OutOfReach {
                            because: format!(
                                "this participant lists `{subtree}` one level deep and never descends, so a \
                                 nested module file is never read at all"
                            )
                            .into(),
                        },
                        format!("a_file_nested_below_{slug}_is_out_of_reach"),
                    ),
                    // The second extent, and the one that makes this example about the bound MODEL rather than
                    // about the call that declares a bound. The rule reads a file's FIRST line, so a real module
                    // header sitting below a licence comment is reported missing — correct against the rule as
                    // worded, wrong against the rule's reason, which such a reader is already served by.
                    //
                    // Declared rather than closed on purpose. Skipping a leading comment block would trade this
                    // edge for a `/* … */` header and an inner attribute above the doc comment, and would leave
                    // the rule's wording saying something other than what the code does.
                    BoundDecl::new(
                        BoundId::new(format!(
                            "house-rules/a-header-below-a-leading-comment-in-{subtree}-over-reacts"
                        )),
                        format!(
                            "a `.rs` file under `{subtree}` whose `//!` header sits below a leading comment"
                        ),
                        Extent::Reached(Reached::OverReacts {
                            because: format!(
                                "the rule reads the first line of a file under `{subtree}`, so a header below a \
                                 licence comment reads as absent though a reader of that file learns what it is \
                                 for — which is the reason the rule gives for itself"
                            )
                            .into(),
                        }),
                        format!("a_header_below_a_leading_comment_in_{slug}_over_reacts"),
                    ),
                ]
            })
            .collect()
    }
}
