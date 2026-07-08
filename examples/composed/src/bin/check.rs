//! The composed CI entry: hand the full constitution to the 天衡 shell and let it project all
//! 三儀 into one exit code. This is the real adopter CLI — `check`, `list`, `--format json|sarif`
//! are all dispatched by `tianheng::run`.
//!
//!     cargo run --bin check -- check --manifest-path .
//!     cargo run --bin check -- check --manifest-path . --format json
//!
//! The `--format` flag changes the *presentation*, never the verdict (the exit code is identical
//! across formats — `scripts/test_examples.sh` asserts exactly that).
use std::process::ExitCode;

use composed_app::governance::constitution;

fn main() -> ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
