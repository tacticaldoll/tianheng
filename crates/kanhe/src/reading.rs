//! Refusing input a reader cannot understand, where the habit is to skip it.
//!
//! Sibling of [`crate::selection`], deliberately apart. That one answers *how many candidates are there*;
//! this one answers *could this be read at all*. Merging them would produce one instrument for two
//! mechanisms, which is the shape this repository removes on sight.
//!
//! **The bug is never "read it wrong". It is that not-readable was spelled the same as not-present.**
//! `filter_map(|part| part.parse().ok())` drops what it cannot parse and hands the survivors on, so a
//! destructure of three succeeds over an input that carried four — measured: `2028--4-30` read as
//! `2028-04-30`. `machinery_names` `continue`d on a failed prefix strip and enumerated 0 of 8 members.
//!
//! **This module binds only the call sites that use it.** Nothing enumerates the readers that should —
//! see `BACKLOG.md`'s entry on a reader's corpus being narrower than its claim, which owns that residue.

use crate::refusal::{Refusal, cannot_judge_at};

/// How a text is divided into fields.
///
/// The two do not differ by convenience, they differ by what an **empty** field means. Collapsing runs is
/// right for a declaration a human spaces freely; it is wrong for a delimiter whose repetition is a defect,
/// and reading `2028--4-30` as three fields is exactly that defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sep {
    /// Runs of whitespace, collapsed — so `24   2028` is two fields, not four.
    Whitespace,
    /// One character, **not** collapsed — so `2028--4-30` is four fields, not three.
    Char(char),
}

impl Sep {
    /// Every field, in order, without dropping an empty one.
    fn divide(self, text: &str) -> Vec<&str> {
        match self {
            Sep::Whitespace => text.split_whitespace().collect(),
            Sep::Char(separator) => text.split(separator).collect(),
        }
    }
}

/// Exactly `N` fields, or a refusal naming how many were found.
///
/// A **cannot-judge**: a field count the reader did not expect is a fact about the input, not a subject
/// disagreeing with what it is judged against.
///
/// **The count is the whole point.** `split(sep).filter_map(…)` answers *fewer* by dropping, and the
/// survivors then destructure as if nothing was lost — so a reader claiming to have read three fields
/// reports a verdict over an input that carried four. Asking for `N` and being told what arrived makes the
/// two states different again.
///
/// `what` names the thing being read, so the refusal says which reader met the input rather than only what
/// the input was. What to *write* instead belongs to the caller, which knows the form it wanted.
pub fn fields<'a, const N: usize>(
    what: &str,
    text: &'a str,
    sep: Sep,
) -> Result<[&'a str; N], Refusal> {
    let found = sep.divide(text);
    let count = found.len();
    found.try_into().map_err(|_| {
        cannot_judge_at(
            "repository-checks#fields-miscounted",
            format!(
                "the {what} reads `{text}`, which divides into {count} fields where this reader expects \
                 {N}; taking the ones it recognised would report a verdict over an input it did not read"
            ),
        )
    })
}
