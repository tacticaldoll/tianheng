//! 繩墨 (Shéngmò) — the inked line.
//!
//! A carpenter snaps it to mark true. Everything is judged against it, and the line is not part of
//! the furniture. This crate is that line for its own repository: the **law** 天衡 declares over
//! itself, and the dogfood gates that run the delivered product's reactions against this workspace.
//!
//! It ships in **no package**. That is not incidental — it is the criterion the governance
//! capability gives for machinery being governance rather than product, and until this crate
//! existed the apparatus failed it: `cargo package --list -p tianheng` carried all 50 files under
//! `tests/`, so every repository check judging this repository's changelog, specs and scripts reached every
//! adopter, where it could only detect no workspace and return.
//!
//! # What lives where
//!
//! The same shape 圭表 and 渾儀 already carry, for the same reason:
//!
//! - **`src/`** — the law, its declared observation bounds and the judgements. A judgement is implementation,
//!   not a test.
//! - **`src/tests/`** — the failure matrices, as unit tests beside what they test.
//! - **`tests/`** — the dogfood gates: what runs the product reactions against the real repository.
//!
//! # Not one of the 三儀
//!
//! 璇璣, 星表, 圭表, 渾儀, 漏刻 and 天衡 are instruments, and they are the product. 繩墨 is a
//! measuring tool of a different kind and belongs to the workshop rather than to what leaves it.
//! The name is chosen to say that before a reader has to ask.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub mod bounds;
pub mod law;
pub mod workspace;
