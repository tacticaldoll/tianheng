//! 校讎 (Jiàochóu) — collation.
//!
//! Two texts laid side by side to find where they disagree. Every reaction in this crate does that:
//! `AGENTS.md` against `.github/workflows/ci.yml`, `CHANGELOG.md` against the tree it describes, a
//! spec's declared bound against the test that defends it, a generated document against the
//! generator its own header names.
//!
//! It is **not** self-governance. 繩墨 holds the law 天衡 declares over itself and the reactions that
//! run the delivered product against this workspace; what lives here judges the repository's
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
//! - **`tests/`** — the reactions: what runs against the real repository.

#![forbid(unsafe_code)]

pub mod bound_register_parse;
pub mod census;
pub mod merge_message_gate;
pub mod publish_source_gate;
pub mod refusal;
pub mod refusal_exemptions;
pub mod refusal_sites;
pub mod region;
pub mod release_coherence_gate;
