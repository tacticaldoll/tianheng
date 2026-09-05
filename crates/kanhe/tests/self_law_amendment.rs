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

use std::path::PathBuf;

/// One boundary as the projection renders it: everything a steward reads when weighing an amendment.
///
/// **Every rendered field, verbatim, rather than a chosen few.** Two earlier forms each omitted the field the
/// next relaxation moved. The first carried the heading and the rule: lowering a boundary from `enforce` to
/// `warn` moved neither, turning a run-failing violation into an advisory — run against that form with 璇璣
/// lowered, all nine self-governance assertions and this check passed. The second named heading, reason, rule
/// and kind: the renderer also emits `- **anchor**:`, which it did not read, so an anchor could be added or
/// changed with nothing moving.
///
/// So `fields` carries the `- **…**:` lines as they are written, in order. A field the renderer gains later
/// enters this identity by itself, which is what stops a third form from omitting a third thing.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
struct Boundary {
    heading: &'static str,
    reason: &'static str,
    fields: &'static [&'static str],
}

/// Every boundary the law declares, as the projection renders it.
///
/// **Hand-written on purpose, and the only such list this crate holds.** Everywhere else here a corpus is
/// derived, because a typed list drifts. Here the drift *is* the reaction: this list is the second artifact an
/// amendment has to produce, so it must be the thing that has to be edited, and a derived one would agree with
/// the law by construction and observe nothing.
const DECLARED: [Boundary; 13] = [
    Boundary {
        heading: "`xuanji` (crate)",
        reason: "璇璣 is the dimension-agnostic reaction model: it must not depend on any workspace member; serde_json only",
        fields: &[
            "- **rule**: restrict dependencies to (only: serde_json)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`xingbiao` (crate)",
        reason: "星表 is the shared metadata substrate: it depends on no workspace member at all; serde_json only",
        fields: &[
            "- **rule**: restrict dependencies to (only: serde_json)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`guibiao` (crate)",
        reason: "the 圭表 static core stays dependency-light: serde_json, xuanji (reaction model), and xingbiao (metadata substrate) only. functional core ⊥ imperative shell: 圭表 must not depend on the 天衡 shell. 三儀 ⊥ 三儀: it names no sibling dimension",
        fields: &[
            "- **rule**: restrict dependencies to (only: serde_json, xuanji, xingbiao)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`hunyi` (crate)",
        reason: "渾儀 is the semantic AST dimension: it depends on 璇璣, 星表, serde_json and syn only. 三儀 ⊥ 三儀: it names no sibling dimension and never the 天衡 shell (functional dimension ⊥ imperative shell)",
        fields: &[
            "- **rule**: restrict dependencies to (only: xuanji, xingbiao, serde_json, syn)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`louke` (crate)",
        reason: "漏刻 is the runtime dimension: it depends on 璇璣 and 星表 only. 三儀 ⊥ 三儀: naming no sibling dimension and never the 天衡 shell",
        fields: &[
            "- **rule**: restrict dependencies to (only: xuanji, xingbiao)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`tianheng` (crate)",
        reason: "the 天衡 shell's direct normal edges end at the observation dimensions and at projection serialization, never at the lower reaction model or metadata substrate",
        fields: &[
            "- **rule**: restrict dependencies to (only: guibiao, hunyi, louke, serde_json)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`shengmo` (crate)",
        reason: "繩墨 depends on 天衡 and serde_json only: no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist",
        fields: &[
            "- **rule**: restrict dependencies to (only: tianheng, serde_json)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`kanhe` (crate)",
        reason: "勘合 depends on 繩墨, 天衡, serde_json and toml_edit only: no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist",
        fields: &[
            "- **rule**: restrict dependencies to (only: shengmo, tianheng, serde_json, toml_edit)",
            "- **kind**: crate · **severity**: enforce",
        ],
    },
    Boundary {
        heading: "`xuanji::crate` (module)",
        reason: "璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself",
        fields: &[
            "- **rule**: inline symbol path confined to module (confined_prefix: std::time; ending_with: now)",
            "- **kind**: module · **severity**: enforce · **crate**: xuanji",
        ],
    },
    Boundary {
        heading: "`guibiao::crate` (module)",
        reason: "path canonicalization and cycle/dedup guards in guibiao must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling",
        fields: &[
            "- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
            "- **kind**: module · **severity**: enforce · **crate**: guibiao",
        ],
    },
    Boundary {
        heading: "`hunyi::crate` (module)",
        reason: "path canonicalization and cycle/dedup guards in hunyi must resolve through `xingbiao::canonicalize_or_fail` or `try_visit` for unified failure handling",
        fields: &[
            "- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
            "- **kind**: module · **severity**: enforce · **crate**: hunyi",
        ],
    },
    Boundary {
        heading: "`louke::crate` (module)",
        reason: "path canonicalization and cycle/dedup guards in louke must resolve through `xingbiao::try_visit` for unified failure handling",
        fields: &[
            "- **rule**: inline symbol path confined to module (confined_prefix: std::fs; ending_with: canonicalize)",
            "- **kind**: module · **severity**: enforce · **crate**: louke",
        ],
    },
    Boundary {
        heading: "`xuanji::crate` (semantic)",
        reason: "璇璣 is the measure-only reaction model: it reads no ambient clock inline and exposes no async surface — time and effects enter only through the dimensions above it, never the model itself",
        fields: &[
            "- **rule**: must not expose async fn (including_submodules: true; scan_depth: subtree)",
            "- **kind**: semantic · **severity**: enforce · **crate**: xuanji",
        ],
    },
];

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("AGENTS.self-law.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// What the projection renders, or why it could not be read.
///
/// Typed apart rather than collapsed into an empty set: a projection this reader cannot parse is not a law with no
/// boundaries, and an empty answer compared two-way against an empty declaration would pass over nothing.
#[derive(Debug, PartialEq, Eq)]
enum Projected {
    Read(Vec<Rendered>),
    Unreadable(String),
}

/// The owned form of [`Boundary`], for what a run reads rather than what this file declares.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
struct Rendered {
    heading: String,
    reason: String,
    fields: Vec<String>,
}

impl Rendered {
    fn of(declared: &Boundary) -> Self {
        Self {
            heading: declared.heading.to_string(),
            reason: declared.reason.to_string(),
            fields: declared.fields.iter().map(|f| (*f).to_string()).collect(),
        }
    }
}

/// Each `### ` section: its heading, its whole reason, and every `- **…**:` line beneath it.
///
/// **The reason is read to its end, not to its first line.** The renderer writes it as `\n> {reason}\n\n`,
/// so only the first line of a reason carrying a newline is marked — `because` places no restriction on
/// newlines. A reader that took the marked line alone would hold half a sentence and call the rest unnamed,
/// which is the same defect as omitting a field: the amendment surface is smaller than the identity claims.
/// Continuation lines run to the blank line the renderer writes before the fields.
///
/// A `Vec` rather than a set, so the caller can see a repeated boundary before any comparison folds it away.
fn projected(text: &str) -> Projected {
    /// A section whose heading has been read and whose body is still arriving.
    struct Open {
        heading: String,
        reason: Vec<String>,
        in_reason: bool,
        fields: Vec<String>,
    }

    fn close(open: Option<Open>, read: &mut Vec<Rendered>) -> Result<(), String> {
        let Some(open) = open else {
            return Ok(());
        };
        let mut missing = Vec::new();
        if open.reason.is_empty() {
            missing.push("a `> ` reason");
        }
        if open.fields.is_empty() {
            missing.push("a `- **…**:` field");
        }
        if !missing.is_empty() {
            return Err(format!(
                "the section `{}` carries no {}, so this reader cannot say what that boundary declares",
                open.heading,
                missing.join(" and no ")
            ));
        }
        read.push(Rendered {
            heading: open.heading,
            reason: open.reason.join("\n"),
            fields: open.fields,
        });
        Ok(())
    }

    let mut read: Vec<Rendered> = Vec::new();
    let mut open: Option<Open> = None;

    for line in text.lines() {
        if let Some(rest) = line.strip_prefix("### ") {
            if let Err(why) = close(open.take(), &mut read) {
                return Projected::Unreadable(why);
            }
            open = Some(Open {
                heading: rest.trim().to_string(),
                reason: Vec::new(),
                in_reason: false,
                fields: Vec::new(),
            });
            continue;
        }
        if line.starts_with('#') {
            // Any other heading closes the open section: the projection groups boundaries under `## `
            // headings, so a heading is a boundary's end and not prose inside it.
            if let Err(why) = close(open.take(), &mut read) {
                return Projected::Unreadable(why);
            }
            continue;
        }
        let Some(section) = open.as_mut() else {
            if line.starts_with("> ") || line.starts_with("- **") {
                return Projected::Unreadable(format!(
                    "a boundary field appears under no `### ` heading: {line}"
                ));
            }
            continue;
        };
        let quoted = if line == ">" {
            Some("")
        } else {
            line.strip_prefix("> ")
        };
        if let Some(rest) = quoted {
            // The reason is one unbroken run of quoted lines. A second run after it is a second reason,
            // which is a section this reader cannot attribute rather than one it should join up.
            if !section.reason.is_empty() && !section.in_reason {
                return Projected::Unreadable(format!(
                    "the section `{}` opens a second quoted run, `{rest}`, so this reader cannot say which \
                     one is the boundary's reason",
                    section.heading
                ));
            }
            section.reason.push(rest.trim_end().to_string());
            section.in_reason = true;
            continue;
        }
        section.in_reason = false;
        if line.starts_with("- **") {
            section.fields.push(line.trim_end().to_string());
        } else if !line.trim().is_empty() {
            // **Nothing unquoted belongs to a boundary.** Every reason line is quoted by the renderer, so
            // unquoted prose inside a section is text this reader can attribute neither to the boundary nor
            // to the document — and guessing, which an earlier form did by blank line and by `- **` prefix,
            // is what silently dropped a reason's second paragraph.
            return Projected::Unreadable(format!(
                "the section `{}` carries unquoted prose, `{line}`, which this reader cannot attribute to \
                 the boundary or to the document",
                section.heading
            ));
        }
    }
    if let Err(why) = close(open, &mut read) {
        return Projected::Unreadable(why);
    }
    if read.is_empty() {
        return Projected::Unreadable(
            "the projection renders no boundary at all, which is a document this reader cannot be about"
                .to_string(),
        );
    }
    Projected::Read(read)
}

/// The declared boundary list equals the projected one, both directions.
///
/// The two directions catch different things and neither implies the other: a boundary in the projection and
/// not here is an amendment nobody named, and one here and not in the projection is a declaration that
/// outlived its boundary — the second is what a one-directional comparison misses, and it is how a list comes
/// to certify a law that no longer says what it says.
///
/// **What this establishes, and what it does not.** It forces a structural change to the law to produce a
/// *second explicit artifact*, in a file `.github/CODEOWNERS` routes to the steward, so a delta cannot arrive
/// inside a regenerated projection unremarked. It does **not** establish that anyone accepted it: one actor
/// can change the law, re-bless the projection and edit this list in a single commit, and everything here
/// passes. That is a judgement boundary rather than a gap this check can close — a single-steward repository
/// has no mechanical second party, since a pull request's author cannot approve their own.
#[test]
fn the_law_declares_no_boundary_this_repository_has_not_named() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — the same repo-only discipline every reaction here keeps
    };
    let path = root.join("AGENTS.self-law.md");
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));

    let projected = match projected(&text) {
        Projected::Read(read) => read,
        Projected::Unreadable(why) => panic!(
            "the self-law projection could not be read, which is not the same fact as a law with no \
             boundaries: {why}"
        ),
    };
    let declared: Vec<Rendered> = DECLARED.iter().map(Rendered::of).collect();

    // Both sides are checked for repeats before either is compared. A duplicate on either side would be
    // folded away by a set comparison, and the equality would then hold over a law neither side fully saw.
    for (label, list) in [("this file", &declared), ("the projection", &projected)] {
        let mut sorted = list.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(
            sorted.len(),
            list.len(),
            "{label} carries the same boundary twice, so a comparison over it would be over fewer \
             boundaries than it holds"
        );
    }

    let mut declared_sorted = declared;
    let mut projected_sorted = projected;
    declared_sorted.sort();
    projected_sorted.sort();
    assert_eq!(
        declared_sorted, projected_sorted,
        "the law's boundaries differ from what this repository has named. Every field is part of the \
         identity: a boundary added or removed, an allowlist widened, a severity lowered from enforce to \
         warn, or a reason rewritten all move it. Name the amendment by editing DECLARED in this file — \
         never `AGENTS.self-law.md`, which is generated."
    );
}

/// The reader answers *unreadable* rather than *no boundaries* for each shape that produces neither.
///
/// Without this the check's own failure mode is the silent one it exists to refuse: a projection whose
/// headings this reader stopped recognizing would render an empty list, and a two-way comparison against an
/// empty declaration is an assertion over nothing.
#[test]
fn a_projection_this_reader_cannot_parse_is_not_a_law_with_no_boundaries() {
    for (label, text) in [
        (
            "no boundary section at all",
            "# prose only\n\nnothing here.\n",
        ),
        (
            "a section with no reason",
            "### `a` (crate)\n\n- **rule**: r\n- **kind**: k\n",
        ),
        (
            "a section with no field at all",
            "### `a` (crate)\n\n> why\n",
        ),
        (
            "a field under no heading",
            "- **rule**: a rule under no heading\n",
        ),
        (
            "a section opening a second reason",
            "### `a` (crate)\n\n> why\n\n> and again\n- **rule**: r\n",
        ),
        (
            "unquoted prose after the fields",
            "### `a` (crate)\n\n> why\n- **rule**: r\n\nloose prose\n",
        ),
        (
            "an unquoted continuation, which the renderer no longer writes",
            "### `a` (crate)\n\n> first line\nsecond line\n\n- **rule**: r\n",
        ),
    ] {
        assert!(
            matches!(projected(text), Projected::Unreadable(_)),
            "{label}: must be refused rather than read as a law with fewer boundaries"
        );
    }

    // A whole section is read, so each refusal above is about its own shape rather than about a reader that
    // refuses everything.
    assert_eq!(
        projected(
            "### `a` (crate)\n\n> why\n\n- **rule**: r\n- **kind**: crate · **severity**: enforce\n"
        ),
        Projected::Read(vec![Rendered {
            heading: "`a` (crate)".to_string(),
            reason: "why".to_string(),
            fields: vec![
                "- **rule**: r".to_string(),
                "- **kind**: crate · **severity**: enforce".to_string(),
            ],
        }])
    );

    // **A reason spanning paragraphs is read whole, including a line shaped like a field.** The renderer
    // quotes every line — a blank one as a bare `>` — so the run is unbroken and nothing has to be guessed
    // from a blank line or from a `- **` prefix, which is how an earlier form dropped a second paragraph
    // and misread a `- **note**:` line as a boundary field.
    assert_eq!(
        projected(
            "### `a` (crate)\n\n> first paragraph\n>\n> - **note**: still the reason\n\n- **rule**: r\n"
        ),
        Projected::Read(vec![Rendered {
            heading: "`a` (crate)".to_string(),
            reason: "first paragraph\n\n- **note**: still the reason".to_string(),
            fields: vec!["- **rule**: r".to_string()],
        }])
    );

    // **Every rendered field is carried, including one this reader was never told about.** `- **anchor**:` is
    // emitted by the renderer and was absent from two earlier identities; nothing here names it, and it is in
    // the identity because the fields are taken as written rather than chosen.
    assert_eq!(
        projected(
            "### `a` (crate)\n\n> why\n\n- **rule**: r\n- **anchor**: src/lib.rs\n- **kind**: k\n"
        ),
        Projected::Read(vec![Rendered {
            heading: "`a` (crate)".to_string(),
            reason: "why".to_string(),
            fields: vec![
                "- **rule**: r".to_string(),
                "- **anchor**: src/lib.rs".to_string(),
                "- **kind**: k".to_string(),
            ],
        }])
    );
}
