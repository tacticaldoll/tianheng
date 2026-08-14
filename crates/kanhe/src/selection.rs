//! Answering *how many*, where a reader has several candidates and needs some of them.
//!
//! This exists because of a measured class, not a preference. A reader that quietly takes one of several
//! candidates reports clean over a subject it never read, and it stays correct for exactly as long as a second
//! candidate happens not to exist. Live instances repaired in this repository: `split_once("pub use super::{")`
//! read the first of the prelude's re-export statements; `trim_end_matches("::*")` folded a glob's whole
//! re-export set into one identifier.
//!
//! **The bug is never "chose wrong". It is that no choice was made** — `split_once`, `.next()` and `.first()`
//! are reached for by habit, and each silently answers *one* where the domain answers *many*. So the step this
//! module asks for is the one that was skipped: **make the candidates a value first.** Once they are a
//! collection, both the temptation and the defect are gone, and what remains is a decision with two legible
//! spellings.
//!
//! Neither is a default, because neither answer is always right: a prelude may legitimately carry two
//! re-export statements, and a citation must resolve to exactly one test. Forcing an answer to *how many* is
//! the point; forcing a particular answer would only produce the deliberate detour that carved out the glob
//! special case in the first place.
//!
//! **This module binds only the call sites that use it.** Nothing enumerates the readers that should — see
//! `BACKLOG.md`'s entry on a reader's corpus being narrower than its claim, which owns that residue.

use crate::refusal::{Refusal, cannot_judge};

/// Exactly one candidate, or a refusal saying which way the count was wrong.
///
/// A **cannot-judge**, not a violation: none and several are both facts about the input this reader could not
/// reduce to one, which is a different thing from a subject that disagrees with what it is judged against.
pub fn the_only<T>(what: &str, candidates: impl IntoIterator<Item = T>) -> Result<T, Refusal> {
    let mut found = candidates.into_iter();
    let Some(first) = found.next() else {
        return Err(cannot_judge(format!(
            "expected exactly one {what} and found none, so there is nothing to judge rather than a \
             disagreement to report"
        )));
    };
    let extra = found.count();
    if extra > 0 {
        return Err(cannot_judge(format!(
            "expected exactly one {what} and found {}; taking the first would report a verdict over a subject \
             this reader did not read",
            extra + 1
        )));
    }
    Ok(first)
}

/// Every candidate.
///
/// **This performs no check** — it is `collect` under a name that records the decision, so a reader of the
/// call site can see that *all* was chosen rather than inherited from a habit. It is stated here so the name
/// cannot be read as protection: what it buys is legibility beside [`the_only`], nothing more.
pub fn all_of<T>(candidates: impl IntoIterator<Item = T>) -> Vec<T> {
    candidates.into_iter().collect()
}
