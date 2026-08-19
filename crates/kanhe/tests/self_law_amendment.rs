//! Repository check: the law's boundary set is declared, so an amendment cannot land unnamed.
//!
//! `.github/CODEOWNERS` states the intended reaction in its own words — *changing the law is an amendment
//! and requires steward review*, and *the review requirement is the reaction: a merge cannot relax the law
//! without a human accepting it*. Its last paragraph then says what that is worth: designation alone only
//! auto-requests review, and making review required needs branch protection the admin has to enable.
//!
//! **Measured, it is not enabled**: `main`'s protection answers `require_code_owner_reviews: false` and
//! `required_approving_review_count: 0`. And enabling it would not close this, because GitHub does not let a
//! pull request's author approve their own — so for a repository whose steward and author are one person the
//! rule cannot fire at all. A prose prescription with no backstop is the shape this repository's own reason
//! rule forbids everywhere else, and here it sat on the law itself.
//!
//! **What it cost.** Two crate boundaries reached `AGENTS.self-law.md` under a commit body reading *the law
//! itself did not change: the regenerated projection differs by exactly three lines, all of them the
//! preamble's own self-reference*. The projection gained nineteen lines, fourteen of them two new boundary
//! entries with their own targets, rules and severities. Nothing refused it, and it was found by reading the
//! history months later rather than by a gate.
//!
//! So the boundary set is declared here and held against the projection **in both directions**, which is what
//! `repository-checks` already requires of any constant a check judges by. Adding a boundary, removing one,
//! or **widening an allowlist** all move the projection, and each then fails until the declaration below is
//! edited to match — and that edit is the amendment artifact a squash body has to describe.
//!
//! **Why the rule text and not just the target.** Relaxing the law rarely deletes a boundary; it widens one.
//! Keying on the heading alone would pass a `xuanji` that had quietly gained `guibiao`. The existing
//! self-governance assertions do not reach that either: `dimension_boundaries_declare_the_mutual_independence_law`
//! reads only the three dimension crates' allowlists, and `every_workspace_member_is_self_governed` asks
//! whether a member has a boundary, never what that boundary permits. Widening 璇璣's allowlist was invisible
//! to every check in this workspace before this one.
//!
//! **Why the projection and not `constitution()`.** `kanhe` can reach `shengmo::law` and calling it would
//! compare the law against itself — `f() == f()`, an assertion that cannot fail. The subject is the tracked
//! text an agent and a steward both read.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// Every boundary the law declares, as the projection renders it: its heading and its rule.
///
/// **Hand-written on purpose, and the only such list this check holds.** Everywhere else in this crate a
/// corpus is derived, because a typed list drifts. Here the drift *is* the reaction: this list is the human
/// side of the amendment, so it must be the thing that has to be edited, and a derived one would agree with
/// the law by construction and observe nothing.
const DECLARED: [(&str, &str); 13] = [
    (
        "`xuanji` (crate)",
        "restrict dependencies to (only: serde_json)",
    ),
    (
        "`xingbiao` (crate)",
        "restrict dependencies to (only: serde_json)",
    ),
    (
        "`guibiao` (crate)",
        "restrict dependencies to (only: serde_json, xuanji, xingbiao)",
    ),
    (
        "`hunyi` (crate)",
        "restrict dependencies to (only: xuanji, xingbiao, serde_json, syn)",
    ),
    (
        "`louke` (crate)",
        "restrict dependencies to (only: xuanji, xingbiao)",
    ),
    (
        "`tianheng` (crate)",
        "restrict dependencies to (only: guibiao, hunyi, louke, serde_json)",
    ),
    (
        "`shengmo` (crate)",
        "restrict dependencies to (only: tianheng, serde_json)",
    ),
    (
        "`kanhe` (crate)",
        "restrict dependencies to (only: shengmo, tianheng, serde_json)",
    ),
    (
        "`xuanji::crate` (module)",
        "inline symbol path confined to module (confined_prefix: std::time; ending_with: now)",
    ),
    (
        "`guibiao::crate` (module)",
        "inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
    ),
    (
        "`hunyi::crate` (module)",
        "inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
    ),
    (
        "`louke::crate` (module)",
        "inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
    ),
    (
        "`xuanji::crate` (semantic)",
        "must not expose async fn (including_submodules: true; scan_depth: subtree)",
    ),
];

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("AGENTS.self-law.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// What the projection renders, as `(heading, rule)` pairs, or why it could not be read.
///
/// Three states rather than an empty set: a projection this reader cannot parse is not a law with no
/// boundaries, and an empty answer compared two-way against an empty declaration would pass over nothing.
#[derive(Debug, PartialEq, Eq)]
enum Projected {
    Read(BTreeSet<(String, String)>),
    Unreadable(String),
}

fn projected(text: &str) -> Projected {
    let mut pairs = BTreeSet::new();
    let mut heading: Option<String> = None;
    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("### ") {
            if let Some(orphan) = heading.replace(rest.trim().to_string()) {
                return Projected::Unreadable(format!(
                    "the section `{orphan}` is followed by another heading before any `- **rule**:` line, \
                     so this reader cannot say what that boundary declares"
                ));
            }
        } else if let Some(rule) = line.strip_prefix("- **rule**: ") {
            match heading.take() {
                Some(head) => {
                    pairs.insert((head, rule.trim().to_string()));
                }
                None => {
                    return Projected::Unreadable(format!(
                        "a `- **rule**:` line appears under no `### ` heading: {rule}"
                    ));
                }
            }
        }
    }
    if let Some(orphan) = heading {
        return Projected::Unreadable(format!(
            "the section `{orphan}` carries no `- **rule**:` line, so this reader cannot say what that \
             boundary declares"
        ));
    }
    if pairs.is_empty() {
        return Projected::Unreadable(
            "the projection renders no boundary at all, which is a document this reader cannot be about"
                .to_string(),
        );
    }
    Projected::Read(pairs)
}

/// The declared boundary set equals the projected one, both directions.
///
/// The two directions catch different things and neither implies the other: a boundary in the projection and
/// not here is an amendment nobody named, and one here and not in the projection is a declaration that
/// outlived its boundary — the second is what a one-directional comparison misses, and it is how a list comes
/// to certify a law that no longer says what it says.
#[test]
fn the_law_declares_no_boundary_this_repository_has_not_named() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — the same repo-only discipline every reaction here keeps
    };
    let path = root.join("AGENTS.self-law.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));

    let projected = match projected(&text) {
        Projected::Read(pairs) => pairs,
        Projected::Unreadable(why) => panic!(
            "the self-law projection could not be read, which is not the same fact as a law with no \
             boundaries: {why}"
        ),
    };
    let declared: BTreeSet<(String, String)> = DECLARED
        .iter()
        .map(|(head, rule)| ((*head).to_string(), (*rule).to_string()))
        .collect();

    assert_eq!(
        declared.len(),
        DECLARED.len(),
        "two entries in DECLARED are identical, so the set comparison below is over fewer boundaries than \
         the list claims"
    );
    assert_eq!(
        declared, projected,
        "the law's boundary set differs from the set this repository has named. An amendment — a boundary \
         added or removed, or an allowlist widened — is accepted by editing DECLARED in this file, which is \
         the artifact a steward reviews and a squash body describes. Never edit `AGENTS.self-law.md`, which \
         is generated."
    );
}

/// The reader answers *unreadable* rather than *no boundaries* for each shape that produces neither.
///
/// Without this the check's own failure mode is the silent one it exists to refuse: a projection whose
/// headings this reader stopped recognizing would render an empty set, and a two-way comparison against an
/// empty declaration is an assertion over nothing.
#[test]
fn a_projection_this_reader_cannot_parse_is_not_a_law_with_no_boundaries() {
    assert!(matches!(
        projected("# a document with no boundary sections\n\nprose only.\n"),
        Projected::Unreadable(_)
    ));
    assert!(matches!(
        projected("### `a` (crate)\n\nprose, and no rule line.\n"),
        Projected::Unreadable(_)
    ));
    assert!(matches!(
        projected("### `a` (crate)\n\n### `b` (crate)\n\n- **rule**: r\n"),
        Projected::Unreadable(_)
    ));
    assert!(matches!(
        projected("- **rule**: a rule under no heading\n"),
        Projected::Unreadable(_)
    ));
    // And a well-formed pair is read, so each `Projected::Unreadable` this direction asserts is about its
    // own shape rather than about the reader refusing everything.
    assert_eq!(
        projected("### `a` (crate)\n\n> why\n\n- **rule**: r\n- **kind**: crate\n"),
        Projected::Read(BTreeSet::from([(
            "`a` (crate)".to_string(),
            "r".to_string()
        )]))
    );
}
