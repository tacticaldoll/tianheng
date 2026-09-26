//! What more than one repository check shares, compiled into each that uses it: the one `bash` they run, the
//! readers of `.github/workflows/ci.yml`, the launcher the wrapper directions hand a broken stream with, and
//! the fixture those directions run under.
//!
//! An integration test is its own crate, so what two of them need lives here rather than in `src/`: the
//! workflow grammar is parsed with a dev-dependency, and `src/` sees only `kanhe`'s normal ones. A module
//! with a reader on both sides of that line does not live here — the shell lexer depends on `std` alone and
//! is read from `src/` as well, so it is `kanhe::shell`, shared rather than compiled twice. Each test target
//! uses the part it needs, which is why dead code is allowed at this one boundary.
#![allow(dead_code)]

pub mod bash;
pub mod fixture;
pub mod streams;
pub mod workflow;
