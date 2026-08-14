//! Repository check: the observation bound register.
//!
//! Every observation bound this family declares lives as a scenario in `openspec/specs/*/spec.md` whose
//! heading marks it one. This check holds each of them to carrying exactly one citation — a `PINNED-BY`
//! naming a test the harness registers, or an `UNPINNED` naming a tracker the repository tracks — resolves
//! every reference, refuses a bound stated in prose and declared nowhere, and generates
//! `docs/observation-bounds.md` from what it read.
//!
//! **Whether a cited name is a test that runs is decided by the test harness, not by the source text.**
//! `observation-bound-register` makes that normative, and it is the discipline this repository adopted after
//! measuring and rejecting the text route three times: a `pinned by` line could otherwise be satisfied by a
//! definition commented out, inside a string literal, removed by a `cfg`, or trapped in an uninvoked macro.
//! So this check runs `cargo test -p <member> -- --list` per package — per package rather than per
//! workspace, because the enumeration carries no crate label while a citation may be crate-qualified, and
//! this repository already has one test name registered in two crates.
//!
//! Running cargo from inside a test cargo launched was measured rather than assumed: the outer build lock is
//! released before the test binary runs, so the inner enumeration neither blocks nor rebuilds, and it shares
//! the warm target directory instead of paying for one of its own.

use kanhe::bound_register_parse as parse;

use parse::{
    Bound, Citation, bounds_in, must, parse_bounds, pinning_citations, search, workspace_root,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

// --- citations ------------------------------------------------------------------------------------------

#[test]
fn every_declared_bound_carries_exactly_one_citation() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut offences = Vec::new();
    for bound in parse_bounds(&root) {
        let complaint = match bound.citation {
            Citation::Both => Some(
                "carries both PINNED-BY and UNPINNED; a bound is either defended or tracked, and claiming \
                 both hides which — the two are exclusive answers to one question",
            ),
            Citation::Neither => Some(
                "carries neither PINNED-BY nor UNPINNED; a bound with no recorded defence is indistinguishable \
                 from an oversight",
            ),
            Citation::RepeatedUnpinned => Some(
                "carries more than one UNPINNED; several trackers are several owners of one gap, which is \
                 two answers to one question — and the declaration holds one, so keeping the last records a \
                 bound whose owner is whichever line came last",
            ),
            Citation::UnpinnedWithoutTracker => Some(
                "is UNPINNED with no tracker; untracked debt is indistinguishable from debt nobody owns",
            ),
            _ => None,
        };
        if let Some(why) = complaint {
            offences.push(format!(
                "  {} ({}:{}) {why}",
                bound.id, bound.spec, bound.line
            ));
        }
    }
    assert!(
        offences.is_empty(),
        "declared bounds without exactly one citation:\n{}",
        offences.join("\n")
    );
}

/// Every workspace member's registered test names, keyed by package.
///
/// Per package rather than per workspace: the enumeration carries no crate label while a citation may be
/// crate-qualified, and this repository already has one test name registered in two crates, so a
/// workspace-wide match would let a citation qualified to one crate be satisfied by the other's test.
fn registered_tests(root: &Path) -> BTreeMap<String, BTreeSet<String>> {
    // Tracked content, not the working directory, and refusing rather than shortening. A listing that emits
    // some entries and then fails leaves a short list that reads as authoritative, and every citation in a
    // package never enumerated is then reported as one the harness does not register — a filesystem failure
    // charged to the register. This capability's own requirement says so; nothing held it until now.
    let manifests = must(
        root,
        "`git ls-files -- crates/*/Cargo.toml`",
        &["git", "ls-files", "--", "crates/*/Cargo.toml"],
    );
    // git's pathspec `*` crosses directory separators, unlike the shell's — measured, the glob above also
    // matched fixture manifests nested under a member's tests. A workspace member is one segment deep.
    let members: Vec<String> = manifests
        .lines()
        .filter_map(|path| path.strip_prefix("crates/"))
        .filter_map(|rest| rest.strip_suffix("/Cargo.toml"))
        .filter(|member| !member.contains('/'))
        .map(str::to_string)
        .collect();
    assert!(
        !members.is_empty(),
        "no workspace member was found under crates/ — a citation's test-ness is undecidable without the \
         harness, and reporting every citation resolved against an empty enumeration is the vacuity direction"
    );

    let mut by_package = BTreeMap::new();
    for member in members {
        let listing = must(
            root,
            &format!("`cargo test -p {member} --all-features -- --list`"),
            &[
                "cargo",
                "test",
                "-q",
                "-p",
                &member,
                "--all-features",
                "--",
                "--list",
            ],
        );
        let names: BTreeSet<String> = listing
            .lines()
            .filter_map(|line| line.strip_suffix(": test"))
            .map(|name| name.rsplit("::").next().unwrap_or(name).to_string())
            .collect();
        by_package.insert(member, names);
    }
    by_package
}

#[test]
fn every_pinning_citation_resolves_to_one_registered_test() {
    let Some(root) = workspace_root() else {
        return;
    };
    // **Every citation in the tracked specs, not only those under a bound heading.** The register's own
    // corpus is bound-gated and rightly so — a citation under an ordinary scenario declares no bound — but
    // resolution is a different question, and the marker means one thing in both places. Measured: 70 of 75
    // were resolved here and 5 were parsed by nothing, and renaming one of those five left the whole gate
    // suite green while its spec cited a function that no longer existed.
    let citations = pinning_citations(&root);
    let harness = registered_tests(&root);

    let mut offences = Vec::new();
    {
        for record in &citations {
            let citation = &record.name;
            let at = match &record.bound {
                Some(id) => format!("{id} ({}:{})", record.spec, record.line),
                None => format!("{}:{}", record.spec, record.line),
            };

            // Syntax first, and by construction rather than by escaping: the name is used as a search key and a
            // path component, so a metacharacter or a `..` would resolve a citation for a test that does not
            // exist against something else entirely.
            let (qualifier, name) = match citation.split_once("::") {
                Some((q, n)) => (Some(q), n),
                None => (None, citation.as_str()),
            };
            let ascii_ident = |s: &str| {
                let s = s.strip_prefix("r#").unwrap_or(s);
                !s.is_empty()
                    && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
                    && !s.starts_with(|c: char| c.is_ascii_digit())
            };
            if !ascii_ident(name) || citation.matches("::").count() > 1 {
                offences.push(format!(
                "  {at} is PINNED-BY `{citation}`, which is not a citation this check can resolve"
            ));
                continue;
            }

            let packages: Vec<&String> = match qualifier {
                Some(q) => {
                    if !harness.contains_key(q) {
                        offences.push(format!(
                        "  {at} is PINNED-BY `{citation}`, whose crate qualifier names no workspace member"
                    ));
                        continue;
                    }
                    harness.keys().filter(|k| k.as_str() == q).collect()
                }
                None => harness.keys().collect(),
            };

            let registering: Vec<&String> = packages
                .into_iter()
                .filter(|p| harness[*p].contains(name))
                .collect();
            if registering.is_empty() {
                offences.push(format!(
                "  {at} is PINNED-BY `{citation}`, which the test harness does not register — a renamed or \
                 deleted test leaves this citation naming nothing"
            ));
                continue;
            }

            // The harness decides test-ness; the source decides uniqueness. A name registered once but defined
            // twice names a set rather than a defence.
            let sites = search(
                &root,
                "`git grep` locating the cited definition",
                &[
                    "git",
                    "grep",
                    "-n",
                    "-E",
                    // POSIX ERE, which is what `git grep -E` speaks: `\\s` and `\\b` are PCRE and match
                    // nothing here — measured, they reported every citation defined zero times.
                    // `pub(super) fn` and friends: a visibility qualifier may carry a parenthesised scope, and
                    // this repository's dimension tests use exactly that. Measured — without it, every citation
                    // in `guibiao`'s test modules reported zero definitions.
                    &format!(
                        "^[[:space:]]*(pub([(][^)]*[)])?[[:space:]]+)?(async[[:space:]]+)?(const[[:space:]]+)?(unsafe[[:space:]]+)?fn {name}[[:space:]]*[(<]"
                    ),
                    "--",
                    // Scoped to the cited crate when the citation is qualified. That qualifier exists precisely
                    // because one test name is registered in two crates here, so searching all of `crates/`
                    // would report the citation ambiguous for the reason it was disambiguated.
                    &qualifier.map_or("crates/".to_string(), |q| format!("crates/{q}/")),
                ],
            );
            if sites.len() != 1 {
                offences.push(format!(
                "  {at} is PINNED-BY `{citation}`, defined {} times under crates/ — a citation names one \
                 defence, not a set",
                sites.len()
            ));
            }
        }
    }
    assert!(
        offences.is_empty(),
        "pinning citations that do not resolve to one registered test:\n{}",
        offences.join("\n")
    );
}

#[test]
fn every_unpinned_bound_names_a_tracked_tracker() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked: BTreeSet<String> = must(&root, "`git ls-files`", &["git", "ls-files"])
        .lines()
        .map(str::to_string)
        .collect();

    let mut offences = Vec::new();
    for bound in parse_bounds(&root) {
        let Citation::Unpinned(tracker) = &bound.citation else {
            continue;
        };
        let named = tracker
            .split('`')
            .nth(1)
            .map(str::to_string)
            .unwrap_or_default();
        if named.is_empty() || !tracked.contains(&named) {
            offences.push(format!(
                "  {} ({}:{}) is UNPINNED against `{named}`, which this repository does not track — debt \
                 filed where nobody looks is debt nobody owns",
                bound.id, bound.spec, bound.line
            ));
        }
    }
    assert!(
        offences.is_empty(),
        "unpinned bounds whose tracker is not tracked:\n{}",
        offences.join("\n")
    );
}

#[test]
fn no_test_is_cited_by_bounds_of_two_capabilities() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut by_test: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for bound in parse_bounds(&root) {
        if let Citation::PinnedBy(names) = &bound.citation {
            for name in names {
                by_test
                    .entry(name.clone())
                    .or_default()
                    .insert(bound.capability.clone());
            }
        }
    }
    let shared: Vec<String> = by_test
        .iter()
        .filter(|(_, caps)| caps.len() > 1)
        .map(|(test, caps)| {
            format!(
                "  `{test}` is cited by declared bounds in {} — one behaviour has one defence, so a test \
                 cited across capabilities means one of them is a restatement",
                caps.iter().cloned().collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    assert!(shared.is_empty(), "restated bounds:\n{}", shared.join("\n"));
}

// --- the projection -------------------------------------------------------------------------------------

/// The projection's preamble, with the two figures it states **computed** rather than written.
///
/// A count typed into a generated document is the hand-written census this family refuses: the generator
/// would compare its own literal against itself and never notice the register moving underneath it.
fn preamble(unpinned: usize, total: usize) -> String {
    format!(
        r#"# Observation bounds

Every **observation bound** this family declares: a claim that a reaction deliberately stops at a
named shape, so that shape is governed policy rather than a defect.

**{unpinned} of {total} declared bounds have no pinning test.** That figure is the register's
audit backlog and leads the document because a number in a footnote is not read. Each such bound names
the tracker that owns closing it.

Generated from `openspec/specs/*/spec.md` by `crates/kanhe/tests/bound_register.rs`. **Do not edit by hand** —
regenerate with `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test bound_register`. A stale projection fails that gate.

**What this document does not claim.** It lists the bounds the specs *state in a recognizable form*: a
scenario whose heading marks it a bound. The undeclared-prose direction that keeps this list honest has known
residuals and a deliberate exemption, each stated in this document rather than left in the check's
comments, because a residual a reader cannot see is one the register is lying about:

1. **Unrecognized wording.** A bound worded outside the scanned form — "out-of-scope", "does not claim
   to observe", "a stated, inherited bound" — is invisible to the scan.
2. **The scan is line-oriented.** A statement whose bound names continue onto the next line is examined
   only on the line carrying the trigger words.
3. **A reference clears more than it names.** `(bound: …)` clears the prose it sits with regardless of
   how many bounds that prose states, or whether the bound it names is one of them. This is how a
   retired `#[path]` bound survived two sweeps inside a sentence listing four inherited bounds behind
   one reference to a fifth. The discipline is one reference per stated bound, and it is the author's:
   closing it would mean reading which bounds a sentence lists, which no repository check can do. Scanning
   paragraphs instead of lines was measured against that defect and would not have caught it, because
   the paragraph carries the same clearing reference.

The **exemption**: prose under a requirement whose heading names bounds is not reported, because several
such requirements state their bounds as a numbered list, and requiring each item to become its own
scenario would restructure them and read worse. Its price
is charged — such a requirement must declare at least one bound scenario — but the other items of its
list are unregistered, which is why this list is a floor rather than a proof of completeness.

The second floor is the same shape. A bound declared twice is caught only when both declarations cite
the **same pinning test**, which is a fact rather than a heuristic; two declarations of one behaviour
citing two different tests are invisible. Telling those apart from two genuine bounds over sibling
shapes is a semantic judgment — two operand dimensions here declare identically-worded bounds over
`dyn` and `impl Trait`, each defended by its own test, and each must declare its own — so nothing
observes it and no bound of the register capability claims it.

A third floor was stated here for one change and is **retired**: a `pinned by` line could be satisfied
by a definition that never ran — commented out, inside a string, removed by a `cfg`, or trapped in an
uninvoked macro — because the scan read only the form of a line. Test-ness is now decided by the test
harness enumeration, which registers none of those. The weakness survives only in the source-text
fallback used where no manifest exists, which the register spec describes.

"#
    )
}

fn render_projection(bounds: &[Bound]) -> String {
    let total = bounds.len();
    let unpinned = bounds
        .iter()
        .filter(|b| !matches!(b.citation, Citation::PinnedBy(_)))
        .count();

    let mut by_capability: BTreeMap<&str, Vec<&Bound>> = BTreeMap::new();
    for bound in bounds {
        by_capability
            .entry(bound.capability.as_str())
            .or_default()
            .push(bound);
    }

    let mut out = preamble(unpinned, total);
    // Capabilities in path order, bounds in DOCUMENT order within each — a reader following the projection
    // back into a spec meets them in the order that spec states them.
    for (capability, entries) in by_capability {
        out.push_str(&format!("\n## {capability}\n"));
        for bound in entries {
            out.push_str(&format!("\n### `{}`\n\n> {}\n\n", bound.id, bound.body));
            match &bound.citation {
                Citation::PinnedBy(names) => {
                    // One bullet per defence, which is how this document already reads where a bound cites
                    // two — a reader scanning for a test name finds it at the start of a line either way.
                    for name in names {
                        out.push_str(&format!("- **pinned by**: `{name}`\n"));
                    }
                }
                Citation::Unpinned(tracker) => {
                    out.push_str(&format!("- **unpinned**, tracked by: {tracker}\n"));
                }
                other => panic!(
                    "bound {} reached the projection with an invalid citation {other:?} — the citation \
                     check must refuse it before this one renders it",
                    bound.id
                ),
            }
        }
    }
    out
}

#[test]
fn the_register_projection_is_generated_and_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    // No `if is_file` escape: a deleted projection is a stale projection, and skipping on absence lets the
    // document be removed without anything noticing.
    let bounds = parse_bounds(&root);
    tianheng::testing::assert_projection_matches(
        &root,
        "docs/observation-bounds.md",
        &render_projection(&bounds),
    );
}

// The census this file once swept for lives in `census.rs`, declared beside every other, so one
// check holds them all. Two implementations of one comparison is what that file exists to end.

/// A citation answered twice fails — and the asymmetry with `PINNED-BY` is deliberate, so it has a control.
///
/// Several pinning tests are several **defences of one bound**; several trackers are several **owners of one
/// gap**. The declaration holds one tracker, so keeping the last silently recorded a bound whose owner was
/// whichever line came last. The comment beside the parse described this defect and the code implemented it
/// only for the `PINNED-BY`+`UNPINNED` pair.
#[test]
fn a_citation_answered_twice_fails_whichever_answer_is_repeated() {
    let scenario = |citations: &str| {
        format!(
            "#### Scenario: Something is not observed — a stated bound\n\n- **WHEN** a shape appears\n- **THEN** nothing reacts\n{citations}"
        )
    };

    let repeated_unpinned = bounds_in(
        "some-capability",
        "openspec/specs/some-capability/spec.md",
        &scenario("- **UNPINNED** BACKLOG: one owner\n- **UNPINNED** BACKLOG: another owner\n"),
    );
    assert_eq!(repeated_unpinned.len(), 1, "{repeated_unpinned:?}");
    assert_eq!(
        repeated_unpinned[0].citation,
        Citation::RepeatedUnpinned,
        "two trackers were reduced to one, so the bound records whichever line came last"
    );

    // **The two shapes a count catches and a flag does not.** The rule is *more than one `UNPINNED`*, and
    // the classification used to ask it of the tracker-bearing lines alone, with the bare ones behind a
    // boolean. So a mixed pair fell through to the single-tracker arm and read as a well-formed citation —
    // silently dropping the bare line — and two bare ones collapsed to one flag and read as a single
    // tracker-less citation. Neither is *more than one*, which is what the variant's own doc says it is.
    for (name, citations) in [
        (
            "one bare and one with a tracker",
            "- **UNPINNED**\n- **UNPINNED** BACKLOG: an owner\n",
        ),
        ("two bare", "- **UNPINNED**\n- **UNPINNED**\n"),
    ] {
        let mixed = bounds_in(
            "some-capability",
            "openspec/specs/some-capability/spec.md",
            &scenario(citations),
        );
        assert_eq!(mixed.len(), 1, "{name}: {mixed:?}");
        assert_eq!(
            mixed[0].citation,
            Citation::RepeatedUnpinned,
            "{name}: two UNPINNED lines are two owners of one gap, whichever form each takes"
        );
    }

    // The control. Flattening the two would break a live declaration: `observation-bound-model` states that
    // several `PINNED-BY` lines are all retained, and a first draft of this check read the rule as a
    // bullet count and split the one live instance in two.
    let repeated_pinned = bounds_in(
        "some-capability",
        "openspec/specs/some-capability/spec.md",
        &scenario("- **PINNED-BY** `first_test`\n- **PINNED-BY** `second_test`\n"),
    );
    assert_eq!(
        repeated_pinned[0].citation,
        Citation::PinnedBy(vec!["first_test".into(), "second_test".into()]),
        "several tests defending one bound is not two answers to one question"
    );

    // And one tracker still reads as one tracker.
    let single = bounds_in(
        "some-capability",
        "openspec/specs/some-capability/spec.md",
        &scenario("- **UNPINNED** BACKLOG: the one owner\n"),
    );
    assert_eq!(
        single[0].citation,
        Citation::Unpinned("BACKLOG: the one owner".into())
    );
}

/// Every **bare** bound id in tracked Rust and Markdown resolves to a declared bound.
///
/// The `(bound: …)` form clears prose; this resolves the **id**, which is no less a reference for being
/// written without the wrapper. Both defects that motivated it sat in a doc comment above the very test
/// defending the bound — where the bijection cannot look, since it compares the two *declaration* sides and a
/// doc comment is neither — and each had been derived from a requirement's prose rather than from the
/// declaring scenario's heading.
///
/// This is reference resolution and not the detector over prose this repository has designed, measured three
/// times and rejected: a bound id has a fixed shape, and the set it must land in is *produced* by the
/// declarations. Nothing here decides what a sentence means.
///
/// The capability set is enumerated from the tracked specs, so a capability added later is recognized without
/// this direction being edited.
#[test]
fn every_bare_bound_reference_resolves_to_a_declared_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let capabilities: BTreeSet<String> = parse::tracked_specs(&root)
        .into_iter()
        .map(|(capability, _)| capability)
        .collect();
    let declared: BTreeSet<String> = parse_bounds(&root)
        .into_iter()
        .map(|bound| bound.id)
        .collect();
    assert!(
        !declared.is_empty(),
        "no declared bound was read, so every reference would fail for a reason that is not about the \
         reference — a corpus that never arrived is not one in which nothing resolves"
    );

    let listing = must(&root, "`git ls-files`", &["git", "ls-files"]);
    let corpus: Vec<&str> = listing
        .lines()
        .filter(|path| path.ends_with(".rs") || path.ends_with(".md"))
        .collect();
    assert!(
        !corpus.is_empty(),
        "no tracked Rust or Markdown was enumerated, so this direction would report clean over nothing"
    );

    let mut dangling = Vec::new();
    for path in corpus {
        // Skipping an unreadable file would report every bare reference in it as resolving, since a
        // reference this sweep never read cannot be dangling. That is clean-over-unread, which this
        // repository refuses wherever it enumerates tracked content.
        let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        for (line, id) in parse::bare_references(&capabilities, &text) {
            if !declared.contains(&id) {
                dangling.push(format!("{path}:{line}: `{id}` names no declared bound"));
            }
        }
    }
    assert!(
        dangling.is_empty(),
        "a bare bound reference resolves to nothing:\n{}\nAn id that points nowhere is indistinguishable \
         from an undeclared bound, and resolution belongs to the id rather than to the syntax around it.",
        dangling.join("\n")
    );
}
