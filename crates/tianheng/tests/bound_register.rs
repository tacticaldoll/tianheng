//! Self-governance reaction: the observation bound register.
//!
//! Every observation bound this family declares lives as a scenario in `openspec/specs/*/spec.md` whose
//! heading marks it one. This reaction holds each of them to carrying exactly one citation — a `PINNED-BY`
//! naming a test the harness registers, or an `UNPINNED` naming a tracker the repository tracks — resolves
//! every reference, refuses a bound stated in prose and declared nowhere, and generates
//! `docs/observation-bounds.md` from what it read.
//!
//! **Whether a cited name is a test that runs is decided by the test harness, not by the source text.**
//! `observation-bound-register` makes that normative, and it is the discipline this repository adopted after
//! measuring and rejecting the text route three times: a `pinned by` line could otherwise be satisfied by a
//! definition commented out, inside a string literal, removed by a `cfg`, or trapped in an uninvoked macro.
//! So this reaction runs `cargo test -p <member> -- --list` per package — per package rather than per
//! workspace, because the enumeration carries no crate label while a citation may be crate-qualified, and
//! this repository already has one test name registered in two crates.
//!
//! Running cargo from inside a test cargo launched was measured rather than assumed: the outer build lock is
//! released before the test binary runs, so the inner enumeration neither blocks nor rebuilds, and it shares
//! the warm target directory instead of paying for one of its own.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("openspec/specs").is_dir() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "openspec/specs expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — a \
         governance reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

/// Run a search whose *ordinary* no-match answer is a non-zero status, and return the matching lines.
///
/// `grep` exits 1 on a clean miss. Treating that as a failure was found the hard way in this repository's
/// shell era and is recorded in the library that replaced it: a producer's contract has to be named per call
/// site rather than inferred, because the alternative — treating every non-zero as empty — turns a failed
/// read into a clean verdict, which is the one direction the Core Contract forbids.
fn search(root: &Path, what: &str, args: &[&str]) -> Vec<String> {
    let output = Command::new(args[0])
        .args(&args[1..])
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {what}: {err}"));
    match output.status.code() {
        Some(0) => String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
        Some(1) => Vec::new(),
        other => panic!(
            "{what} failed (exit {other:?}); a failed read is not an empty result: {}",
            String::from_utf8_lossy(&output.stderr)
        ),
    }
}

/// Run a command in `root`, requiring success, and return its stdout.
///
/// A failed read is not an empty result: reporting one as the other would report a verdict over content that
/// was never read, which is the vacuity direction the Core Contract forbids.
fn must(root: &Path, what: &str, args: &[&str]) -> String {
    let output = Command::new(args[0])
        .args(&args[1..])
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {what}: {err}"));
    assert!(
        output.status.success(),
        "{what} failed ({}); a failed read is not an empty result: {}{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The slug rule, applied to a scenario heading to derive a bound's id.
///
/// `observation_bound_model.rs` derives these independently and compares its set against the projection this
/// file generates. That is the only guard against the two rules drifting, so the duplication is deliberate:
/// unifying them would make that comparison `f() == f()`.
fn slug_of(heading: &str) -> String {
    let mut out = String::with_capacity(heading.len());
    let mut pending_hyphen = false;
    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }
    out
}

/// Whether a scenario heading marks itself a bound, in the one form the register admits: the marker word
/// adjacent to `bound`, with no interposed qualifier. Measured: admitting one interposed word let the phrase
/// "stated and not yet declared as bounds" read as a declaration.
fn marks_a_bound(heading: &str) -> bool {
    heading.contains("a stated bound") || heading.contains("a documented bound")
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum Citation {
    /// Every test cited. Several are legal: `observation-bound-model` declares that one bound may be
    /// defended by more than one test, and `BoundDecl::pinned_by_many` is its typed counterpart. The
    /// register's "exactly one citation" is about the two exclusive FORMS — pinned or tracked — not about
    /// how many tests a defence names. A first draft of this reaction read it as a bullet count, refused
    /// the one live instance, and split the scenario in two; the model's own bijection caught it.
    PinnedBy(Vec<String>),
    /// The whole remainder of the `UNPINNED` line, which names the tracker and says what it owns.
    Unpinned(String),
    UnpinnedWithoutTracker,
    Both,
    Neither,
}

#[derive(Debug, Clone)]
struct Bound {
    id: String,
    capability: String,
    spec: String,
    line: usize,
    /// The `THEN` bullet, continuation lines joined — what the projection quotes.
    body: String,
    citation: Citation,
}

/// Every tracked capability spec, as `(capability, repo-relative path)`.
fn tracked_specs(root: &Path) -> Vec<(String, String)> {
    let listing = must(
        root,
        "`git ls-files openspec/specs`",
        &["git", "ls-files", "openspec/specs"],
    );
    let specs: Vec<(String, String)> = listing
        .lines()
        .filter(|p| p.ends_with("/spec.md"))
        .filter_map(|p| {
            p.strip_prefix("openspec/specs/")
                .and_then(|rest| rest.strip_suffix("/spec.md"))
                .map(|cap| (cap.to_string(), p.to_string()))
        })
        .collect();
    assert!(
        !specs.is_empty(),
        "`git ls-files` matched no openspec/specs/*/spec.md — this reaction would report clean without \
         reading anything, which is the vacuity direction"
    );
    specs
}

/// Read every declared bound out of the tracked specs.
fn parse_bounds(root: &Path) -> Vec<Bound> {
    let mut bounds = Vec::new();

    for (capability, spec) in tracked_specs(root) {
        let text = std::fs::read_to_string(root.join(&spec)).unwrap_or_else(|err| {
            panic!(
                "could not read the declared bounds from {spec}: {err} — a spec this reaction cannot parse \
                 leaves the register undecided rather than clean"
            )
        });
        let lines: Vec<&str> = text.lines().collect();

        for (index, raw) in lines.iter().enumerate() {
            let Some(heading) = raw.strip_prefix("#### Scenario:") else {
                continue;
            };
            let heading = heading.trim();
            if !marks_a_bound(heading) {
                continue;
            }

            let mut body = String::new();
            let mut pinned: Vec<String> = Vec::new();
            let mut unpinned: Vec<String> = Vec::new();
            let mut unpinned_bare = false;
            let mut in_then = false;

            for line in lines.iter().skip(index + 1) {
                let trimmed = line.trim();
                if trimmed.starts_with("#### ")
                    || trimmed.starts_with("### ")
                    || trimmed.starts_with("## ")
                {
                    break;
                }
                if let Some(rest) = trimmed.strip_prefix("- **THEN** ") {
                    body.push_str(rest);
                    in_then = true;
                    continue;
                }
                if let Some(rest) = trimmed.strip_prefix("- **PINNED-BY** ") {
                    pinned.push(rest.trim().trim_matches('`').to_string());
                    in_then = false;
                    continue;
                }
                if trimmed == "- **UNPINNED**" {
                    unpinned_bare = true;
                    in_then = false;
                    continue;
                }
                if let Some(rest) = trimmed.strip_prefix("- **UNPINNED** ") {
                    let rest = rest.trim();
                    if rest.is_empty() {
                        unpinned_bare = true;
                    } else {
                        unpinned.push(rest.to_string());
                    }
                    in_then = false;
                    continue;
                }
                if trimmed.starts_with("- ") {
                    in_then = false;
                    continue;
                }
                if in_then && !trimmed.is_empty() {
                    body.push(' ');
                    body.push_str(trimmed);
                }
            }

            // A scenario carrying two citations declares two bounds behind one heading, so the register
            // holds one of them and the other is defended by a test nothing points at. The old shell gate
            // projected the FIRST and moved on; rebuilding this reaction surfaced the one live instance.
            let tracked = !unpinned.is_empty() || unpinned_bare;
            let citation = if !pinned.is_empty() && tracked {
                Citation::Both
            } else if !pinned.is_empty() {
                Citation::PinnedBy(pinned)
            } else if let Some(tracker) = unpinned.pop() {
                Citation::Unpinned(tracker)
            } else if unpinned_bare {
                Citation::UnpinnedWithoutTracker
            } else {
                Citation::Neither
            };

            bounds.push(Bound {
                id: format!("{capability}/{}", slug_of(heading)),
                capability: capability.clone(),
                spec: spec.clone(),
                line: index + 1,
                body,
                citation,
            });
        }
    }

    assert!(
        !bounds.is_empty(),
        "parsed 0 declared bounds across the tracked specs — the heading form changed, so this reaction \
         cannot judge rather than reporting a register of nothing as clean"
    );
    bounds
}

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
    let members: Vec<String> = std::fs::read_dir(root.join("crates"))
        .expect("crates/ is readable")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("Cargo.toml").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
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
    let bounds = parse_bounds(&root);
    let harness = registered_tests(&root);

    let mut offences = Vec::new();
    for bound in &bounds {
        let Citation::PinnedBy(citations) = &bound.citation else {
            continue;
        };
        for citation in citations {
            let at = format!("{} ({}:{})", bound.id, bound.spec, bound.line);

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
                "  {at} is PINNED-BY `{citation}`, which is not a citation this reaction can resolve"
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
                 deleted test leaves the bound defended by nothing"
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

Generated from `openspec/specs/*/spec.md` by `crates/tianheng/tests/bound_register.rs`. **Do not edit by hand** —
regenerate with `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test bound_register`. A stale projection fails that gate.

**What this document does not claim.** It lists the bounds the specs *state in a recognizable form*: a
scenario whose heading marks it a bound. The undeclared-prose direction that keeps this list honest has
three known residuals and one deliberate exemption, all four enumerated here rather than left in the
reaction's comments, because a residual a reader cannot see is one the register is lying about:

1. **Unrecognized wording.** A bound worded outside the scanned form — "out-of-scope", "does not claim
   to observe", "a stated, inherited bound" — is invisible to the scan.
2. **The scan is line-oriented.** A statement whose bound names continue onto the next line is examined
   only on the line carrying the trigger words.
3. **A reference clears more than it names.** `(bound: …)` clears the prose it sits with regardless of
   how many bounds that prose states, or whether the bound it names is one of them. This is how a
   retired `#[path]` bound survived two sweeps inside a sentence listing four inherited bounds behind
   one reference to a fifth. The discipline is one reference per stated bound, and it is the author's:
   closing it would mean reading which bounds a sentence lists, which no reaction can do. Scanning
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
                     reaction must refuse it before this one renders it",
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

#[test]
fn no_hand_written_census_disagrees_with_the_register() {
    let Some(root) = workspace_root() else {
        return;
    };
    let bounds = parse_bounds(&root);
    let total = bounds.len();
    let capabilities: BTreeSet<&str> = bounds.iter().map(|b| b.capability.as_str()).collect();

    let tracked = must(&root, "`git ls-files`", &["git", "ls-files"]);
    let mut offences = Vec::new();
    let pattern = regex_lite_census();
    for path in tracked.lines().filter(|p| p.ends_with(".md")) {
        let Ok(text) = std::fs::read_to_string(root.join(path)) else {
            continue;
        };
        for (index, line) in text.lines().enumerate() {
            if let Some((written_total, written_caps)) = pattern(line)
                && (written_total != total || written_caps != capabilities.len())
            {
                offences.push(format!(
                    "  {path}:{} writes \"{written_total} bounds across {written_caps} capabilities\" where \
                     the register holds {total} across {} — a hand-written census of a set this reaction \
                     enumerates goes stale silently",
                    index + 1,
                    capabilities.len()
                ));
            }
        }
    }
    assert!(
        offences.is_empty(),
        "hand-written censuses disagreeing with the register:\n{}",
        offences.join("\n")
    );
}

/// Recognise `<n> bounds across <m> capabilities` without pulling in a regex dependency.
fn regex_lite_census() -> impl Fn(&str) -> Option<(usize, usize)> {
    |line: &str| {
        let at = line.find(" bounds across ")?;
        let before = line[..at].rsplit(|c: char| !c.is_ascii_digit()).next()?;
        let rest = &line[at + " bounds across ".len()..];
        let after: String = rest.chars().take_while(char::is_ascii_digit).collect();
        if !rest[after.len()..].trim_start().starts_with("capabilit") {
            return None;
        }
        Some((before.parse().ok()?, after.parse().ok()?))
    }
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-bound-register-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
