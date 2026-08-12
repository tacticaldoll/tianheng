//! Reading the observation-bound register out of the tracked specs.
//!
//! Shared by the register check and by the census sweep, because a census is produced by the check that
//! enumerates the set — a second parse would let the two disagree, which is the drift the census rule exists
//! to end.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("openspec/specs").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

/// Run a search whose *ordinary* no-match answer is a non-zero status, and return the matching lines.
///
/// `grep` exits 1 on a clean miss. Treating that as a failure was found the hard way in this repository's
/// shell era and is recorded in the library that replaced it: a producer's contract has to be named per call
/// site rather than inferred, because the alternative — treating every non-zero as empty — turns a failed
/// read into a clean verdict, which is the one direction the Core Contract forbids.
pub fn search(root: &Path, what: &str, args: &[&str]) -> Vec<String> {
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
pub fn must(root: &Path, what: &str, args: &[&str]) -> String {
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
pub fn slug_of(heading: &str) -> String {
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
pub fn marks_a_bound(heading: &str) -> bool {
    ["a stated bound", "a documented bound"]
        .into_iter()
        .any(|marker| contains_words(heading, marker))
}

fn contains_words(text: &str, words: &str) -> bool {
    text.match_indices(words).any(|(start, matched)| {
        let before = text[..start].chars().next_back();
        let after = text[start + matched.len()..].chars().next();
        before.is_none_or(|ch| !ch.is_alphanumeric())
            && after.is_none_or(|ch| !ch.is_alphanumeric())
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Citation {
    /// Every test cited. Several are legal: `observation-bound-model` declares that one bound may be
    /// defended by more than one test, and `BoundDecl::pinned_by_many` is its typed counterpart. The
    /// register's "exactly one citation" is about the two exclusive FORMS — pinned or tracked — not about
    /// how many tests a defence names. A first draft of this check read it as a bullet count, refused
    /// the one live instance, and split the scenario in two; the model's own bijection caught it.
    PinnedBy(Vec<String>),
    /// The whole remainder of the `UNPINNED` line, which names the tracker and says what it owns.
    Unpinned(String),
    UnpinnedWithoutTracker,
    /// More than one `UNPINNED`. Several trackers are several **owners of one gap**, which is two answers to
    /// the question a citation exists to answer — unlike several `PINNED-BY`, which are several defences of
    /// one bound. The declaration holds one tracker, so keeping one of them silently records a bound whose
    /// owner is whichever line happened to be last.
    RepeatedUnpinned,
    Both,
    Neither,
}

#[derive(Debug, Clone)]
pub struct Bound {
    pub id: String,
    pub capability: String,
    pub spec: String,
    pub line: usize,
    /// The `THEN` bullet, continuation lines joined — what the projection quotes.
    pub body: String,
    pub citation: Citation,
}

/// Every tracked capability spec, as `(capability, repo-relative path)`.
pub fn tracked_specs(root: &Path) -> Vec<(String, String)> {
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
        "`git ls-files` matched no openspec/specs/*/spec.md — this check would report clean without \
         reading anything, which is the vacuity direction"
    );
    specs
}

/// Read every declared bound out of the tracked specs.
/// Every declared bound in **one** spec's text.
///
/// Split out so a direction can be shown a shape rather than only this repository's own specs. A second
/// implementation of the parse would be the twin-drift class this repository keeps closing, and a control
/// pinned against a copy of the parse would say nothing about the parse.
pub fn bounds_in(capability: &str, spec: &str, text: &str) -> Vec<Bound> {
    let mut bounds = Vec::new();
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
        // projected the FIRST and moved on; rebuilding this check surfaced the one live instance.
        let tracked = !unpinned.is_empty() || unpinned_bare;
        let citation = if !pinned.is_empty() && tracked {
            Citation::Both
        } else if !pinned.is_empty() {
            Citation::PinnedBy(pinned)
        } else if unpinned.len() > 1 {
            Citation::RepeatedUnpinned
        } else if let Some(tracker) = unpinned.pop() {
            Citation::Unpinned(tracker)
        } else if unpinned_bare {
            Citation::UnpinnedWithoutTracker
        } else {
            Citation::Neither
        };

        bounds.push(Bound {
            id: format!("{capability}/{}", slug_of(heading)),
            capability: capability.to_string(),
            spec: spec.to_string(),
            line: index + 1,
            body,
            citation,
        });
    }
    bounds
}

pub fn parse_bounds(root: &Path) -> Vec<Bound> {
    let mut bounds = Vec::new();

    for (capability, spec) in tracked_specs(root) {
        let text = std::fs::read_to_string(root.join(&spec)).unwrap_or_else(|err| {
            panic!(
                "could not read the declared bounds from {spec}: {err} — a spec this check cannot parse \
                 leaves the register undecided rather than clean"
            )
        });
        bounds.extend(bounds_in(&capability, &spec, &text));
    }

    assert!(
        !bounds.is_empty(),
        "parsed 0 declared bounds across the tracked specs — the heading form changed, so this check \
         cannot judge rather than reporting a register of nothing as clean"
    );
    bounds
}

/// Whether a character can appear inside a path-like word.
///
/// The run is what makes a path safe from being mistaken for a reference: reading maximal runs, the token in
/// `openspec/specs/repository-checks/spec.md` is the whole path and carries three slashes, so it is not a
/// `<capability>/<slug>` pair and is excluded by construction rather than by an exception list. A substring
/// search would find `repository-checks/spec.md` inside it and refuse a path for resembling a reference — the
/// false refusal this repository forbids its gates.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-')
}

/// Whether `slug` is the kebab-case form a derived bound id carries.
fn is_kebab(slug: &str) -> bool {
    !slug.is_empty()
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Every **bare** `<capability>/<slug>` this text carries, as `(line number, id)`.
///
/// The `(bound: …)` form clears prose; this resolves the **id**, which is no less a reference for being
/// written without the wrapper. Both defects that motivated it sat in a doc comment above the very test
/// defending the bound, where the bijection cannot look — it compares the two declaration sides and a doc
/// comment is neither.
///
/// `capabilities` is **enumerated by the caller** from the tracked specs, never listed here: a capability
/// added later must be recognized without this function being touched, which is the register's own
/// prohibition against a hand-kept membership beside its enumerator.
///
/// This resolves nothing by itself — it reports what looks like a reference, and the caller holds those
/// against the produced id set. Recognition and resolution are kept apart so the bare form cannot grow a
/// second opinion about which ids exist.
pub fn bare_references(capabilities: &BTreeSet<String>, text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(start) = rest.find(is_word_char) {
            let tail = &rest[start..];
            let end = tail.find(|c| !is_word_char(c)).unwrap_or(tail.len());
            let (word, remainder) = tail.split_at(end);
            rest = remainder;
            let Some((capability, slug)) = word.split_once('/') else {
                continue;
            };
            if slug.contains('/') || !capabilities.contains(capability) || !is_kebab(slug) {
                continue;
            }
            found.push((index + 1, word.to_string()));
        }
    }
    found
}
