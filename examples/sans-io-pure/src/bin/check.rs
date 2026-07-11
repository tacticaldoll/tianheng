//! The adopter CI entry: hand the `sans_io_pure` constitution to the 天衡 shell and let it project
//! both folded axes (圭表 clock + 渾儀 async) into one exit code.
//!
//!     cargo run --bin check -- check --manifest-path .
//!     cargo run --bin check -- check --manifest-path . --format json
//!
//! `--format` changes the *presentation*, never the verdict — the exit code is identical across
//! formats (`scripts/test_examples.sh` asserts exactly that).
use std::process::ExitCode;

use sans_io_kernel::governance::constitution;

fn main() -> ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
