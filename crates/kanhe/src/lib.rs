//! 勘合 (Kānhé) — the split tally.
//!
//! A 勘合 is one document made in two halves, kept apart, and proven genuine by fitting them back
//! together. Every repository check in this crate does that:
//! `AGENTS.md` against `.github/workflows/ci.yml`, `CHANGELOG.md` against the tree it describes, a
//! spec's declared bound against the test that defends it, a generated document against the
//! generator its own header names.
//!
//! It is **not** 校讎. That word is already spent: 校讎 is one of the 三司 and names the *amendment
//! flow* — the steward routing, and the amendment itself — while this crate collates a record. A first
//! draft of this crate took the name anyway, which is the misnaming its own siblings exist to end.
//!
//! It is **not** self-governance either. 繩墨 holds the law 天衡 declares over itself and the dogfood gates that
//! run the delivered product's reactions against this workspace; what lives here judges the repository's
//! *record*, and reaches no product contract at all. Keeping the two apart is the point: a claim
//! about one was read as a claim about both for as long as they shared a directory.
//!
//! Like 繩墨 it ships in no package, and for the same reason.
//!
//! # What lives where
//!
//! - **`src/`** — the judgements: what a squash message must be, what a release section must say,
//!   where a refusal may be constructed.
//! - **`src/tests/`** — their failure matrices, as unit tests beside what they test.
//! - **`tests/`** — the repository checks: what runs against the real repository.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod bound_register_parse;
pub mod bounds;
pub mod capability_subjects;
pub mod census;
pub mod gate_identity;
pub mod hermetic_git;
pub mod manifest;
pub mod merge_message_gate;
pub mod prelude_promise;
pub mod publish_source_gate;
pub mod refusal;
pub mod refusal_bounds;
pub mod region;
pub mod release_coherence_gate;
pub mod restatement;
pub mod selection;
pub mod supplied;
pub mod verdict_channel;

#[cfg(test)]
mod tests;
