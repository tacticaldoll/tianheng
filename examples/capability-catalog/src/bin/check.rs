//! Drive the catalog through the same public shell an adopter uses.

use std::process::ExitCode;

use capability_catalog::governance::constitution;

fn main() -> ExitCode {
    tianheng::run(&constitution(), std::env::args())
}
