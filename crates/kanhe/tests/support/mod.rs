//! What the repository checks that read `.github/workflows/ci.yml` share, compiled into each that uses it.
//!
//! An integration test is its own crate, so a reader two of them need lives here rather than in `src/`: the
//! workflow grammar is parsed with a dev-dependency, and `src/` sees only `kanhe`'s normal ones. Each test
//! target uses the part it needs, which is why dead code is allowed at this one boundary.
#![allow(dead_code)]

pub mod shell;
pub mod workflow;
