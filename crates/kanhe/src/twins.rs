//! One implementation living in two modules, found by running rather than by reading.
//!
//! **The class has a name here because two extractions each closed it and each left a sibling behind.**
//! [`crate::hermetic_git`]'s own header says its command builder "lived twice, byte-identical, in
//! `publish_source_gate` and `release_coherence_gate`"; [`crate::manifest`]'s says "two more twins were left
//! behind in that extraction — and unlike the pair that was taken, **these two had diverged**". Both name the
//! same pair of files. Both took the function they had come for and stopped: the corpus of an extraction was
//! *the function someone noticed*, never *what those two modules share*.
//!
//! A review found the remainder, four instances in one sweep. A review finds it again next time, which is the
//! property this replaces.
//!
//! # What it observes, and what that cost to calibrate
//!
//! A window of [`WINDOW`] consecutive executed lines carrying at least [`STATEMENTS`] executed **statements**,
//! appearing in more than one module. Over `crates/kanhe/src` and `crates/shengmo/src` that is **3 windows,
//! and all three are the live twins** — two of the `WorkspaceVersion` consumption, one of the fixture `run()`.
//!
//! **The corpus is executed statements, not item declarations**, and that single rule is what removed every
//! false positive. Without it the same window reports 7: the four extra are `#[cfg(test)] mod tests { use
//! super::*;` in four modules and a closing assertion followed by `#[test]` in five — Rust's own skeleton,
//! which no one should delete. Excluding items rather than listing those shapes is the same distinction
//! [`crate::region`] already draws for comments: what a language *declares* is not what it *executes*.
//!
//! # Two instruments measured and rejected, because each looked right
//!
//! **Normalized function-body similarity.** The live twin scores `0.62` and is the highest-scoring
//! cross-module pair in the corpus — but the noise floor is `0.56`, unrelated directions sharing an
//! `assert!`/`let` skeleton, so the usable margin is `0.06`. Restricting to functions of the same *name*
//! widens it to `0.20` (`0.62` against a next-highest `0.30`) and goes blind to the twin this check's own
//! doc calls the harder one: the `WorkspaceVersion` pair is a 25-line block inside two 200-line functions, so
//! function-level similarity dilutes it to `0.30` — under any threshold that survives the noise.
//!
//! **Windows over statements alone, with punctuation dropped from the stream.** It reports 1 window over the
//! whole corpus and finds neither twin: with the closers removed, the fixture `run()` pair's longest common
//! run is *two* statements, because the four lines that differ are spread through it.
//!
//! The first of those measurements was itself wrong before it was right, which is worth recording. A
//! brace-depth walker that counted `{` inside `panic!("cannot run {program} …")` cut function bodies at the
//! wrong line, and the corpus scan then reported no pair above `0.60` while the twin measured directly scored
//! `0.615`. The instrument said the signal was under the noise; the instrument was broken.
//!
//! # Residue, declared rather than approximated
//!
//! A twin whose two copies were **renamed** apart survives any of these, and so does a second spelling at
//! **token** level — a constant with one owner and a literal copy of its value elsewhere, which is the shape
//! the same review found for `TIANHENG_WORKSPACE_TESTS` and `Do not edit by hand`. Both are declared bounds
//! of `repository-checks` with their own trackers, not gaps this module implies away.

use crate::refusal::{Refusal, cannot_judge_at, violation_at};
use crate::region::Source;

/// How many consecutive executed lines make one window.
///
/// Measured rather than chosen: at 5 the fixture `run()` twin is no longer reported, because the differing
/// lines around its common run leave nothing five long. At 3 the corpus grows without adding a true finding.
pub const WINDOW: usize = 4;

/// How many of a window's lines must be executed statements for the window to be read at all.
///
/// At 3 both live twins disappear — each is a short common run inside otherwise-differing code — and at 1 the
/// corpus fills with single statements between closers.
pub const STATEMENTS: usize = 2;

/// One window of executed text that more than one module carries.
#[derive(Debug, PartialEq, Eq)]
pub struct Twin {
    /// The shared window, joined by newlines, so a diagnostic can show what is duplicated.
    pub window: String,
    /// Every `(path, one-based line)` the window opens at, in the order the corpus was read.
    pub sites: Vec<(String, usize)>,
}

/// Whether a line carries nothing but closers, separators and terminators.
///
/// Such a line occupies a window without contributing to it: two modules ending a call in `);` and `}` share
/// no implementation.
fn is_punctuation(line: &str) -> bool {
    !line.is_empty()
        && line
            .chars()
            .all(|c| c.is_whitespace() || matches!(c, ')' | '}' | ']' | ';' | ',' | '.' | '?'))
}

/// Whether a line **declares an item** rather than executing.
///
/// Rust's skeleton — an attribute, a `mod`, a `use`, the opening line of a `fn` or a type — is written the
/// same way in every module by construction, and reporting it is reporting the language. Recognised by the
/// keyword the declaration opens with, after any visibility and any `async`/`unsafe`, because that is where
/// the distinction actually sits.
fn opens_an_item(line: &str) -> bool {
    const OPENERS: [&str; 10] = [
        "mod ", "use ", "fn ", "struct ", "enum ", "trait ", "impl ", "type ", "const ", "static ",
    ];
    if line.starts_with("#[") || line.starts_with("#![") {
        return true;
    }
    let rest = match line.strip_prefix("pub(") {
        Some(rest) => rest.split_once(") ").map_or(rest, |(_, rest)| rest),
        None => line.strip_prefix("pub ").unwrap_or(line),
    };
    let rest = rest.strip_prefix("async ").unwrap_or(rest);
    let rest = rest.strip_prefix("unsafe ").unwrap_or(rest);
    OPENERS.iter().any(|opener| rest.starts_with(opener))
}

/// Whether a line is an executed statement — the corpus this check reads.
fn is_a_statement(line: &str) -> bool {
    !is_punctuation(line) && !opens_an_item(line)
}

/// The executed lines of one source, each with its one-based position, whitespace collapsed.
///
/// Collapsed so that a window is about what the code *is* rather than how rustfmt wrapped it: two modules
/// that differ only in where a line broke carry one implementation, and a comparison sensitive to the break
/// would say otherwise.
fn executed(text: &str) -> Vec<(usize, String)> {
    let source = Source::of(text);
    source
        .rust()
        .numbered_lines()
        .filter_map(|(number, line)| {
            let collapsed = line.split_whitespace().collect::<Vec<_>>().join(" ");
            (!collapsed.is_empty()).then_some((number, collapsed))
        })
        .collect()
}

/// Every window more than one of `sources` carries, where `sources` is `(path, text)`.
///
/// Windows are reported as they are found, overlaps included: one twin longer than [`WINDOW`] produces
/// several, and collapsing them would decide for a reader how much of the run is the duplicated thing.
pub fn twins(sources: &[(String, String)]) -> Vec<Twin> {
    let mut seen: std::collections::BTreeMap<String, Vec<(String, usize)>> =
        std::collections::BTreeMap::new();
    for (path, text) in sources {
        let lines = executed(text);
        for window in lines.windows(WINDOW) {
            if window
                .iter()
                .filter(|(_, line)| is_a_statement(line))
                .count()
                < STATEMENTS
            {
                continue;
            }
            let key = window
                .iter()
                .map(|(_, line)| line.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            seen.entry(key)
                .or_default()
                .push((path.clone(), window[0].0));
        }
    }
    seen.into_iter()
        .filter(|(_, sites)| {
            sites
                .iter()
                .map(|(path, _)| path)
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                > 1
        })
        .map(|(window, sites)| Twin { window, sites })
        .collect()
}

/// Judge a corpus of modules: no window of executed statements may live in two of them.
///
/// **An empty corpus refuses**, and that is not a formality. This check's whole subject is what a *set* of
/// modules shares, so a corpus that collapsed to nothing satisfies "no window appears twice" exactly as a
/// clean one does — and the enumeration that builds it is an input like any other.
pub fn judge(sources: &[(String, String)]) -> Result<String, Refusal> {
    if sources.len() < 2 {
        return Err(cannot_judge_at(
            "repository-checks#twin-corpus-is-not-a-set",
            format!(
                "read {} module(s), and whether one implementation lives in two of them is not a question a \
                 corpus this size can answer — a collapsed enumeration reports clean for the same reason a \
                 clean tree does",
                sources.len()
            ),
        ));
    }
    let found = twins(sources);
    if found.is_empty() {
        return Ok(format!(
            "ok twins (no window of {WINDOW} executed lines is shared by two of {} modules)",
            sources.len()
        ));
    }
    let mut said = String::new();
    for twin in &found {
        let sites: Vec<String> = twin
            .sites
            .iter()
            .map(|(path, line)| format!("{path}:{line}"))
            .collect();
        said.push_str(&format!("\n  {}\n", sites.join(" | ")));
        for line in twin.window.lines() {
            said.push_str(&format!("      {line}\n"));
        }
    }
    Err(violation_at(
        "repository-checks#one-implementation-in-two-modules",
        format!(
            "these windows of executed statements live in more than one module. Two extractions have already \
             closed this class in this crate and each left a sibling behind, because the corpus of an \
             extraction was the function someone noticed rather than what the two modules share. Delete one \
             copy and call the other:{said}"
        ),
    ))
}
