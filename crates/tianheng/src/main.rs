//! Tianheng runner — the CI reaction.
//!
//! A thin caller of the library entry point [`tianheng::run`], over this repo's own
//! sample constitution. A real project declares one [`tianheng::Constitution`] carrying any
//! static, semantic, and runtime boundaries in Rust and calls `tianheng::run` the same way to
//! get the identical `check` contract across every dimension. This demo declares only a static
//! boundary (it governs a sample static crate); the other dimensions contribute nothing.
//!
//! Usage:
//!   tianheng check --manifest-path <path/to/Cargo.toml>
//!                [--baseline <file> | --write-baseline <file>] [--format text|json]
//!
//! Exits 0 (clean / warn-only / fully baselined), 1 (enforced violation), or
//! 2 (constitution/scan error, unreadable baseline, or a usage mistake).

use std::process::ExitCode;

mod constitution;
use constitution::constitution;

fn main() -> ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
