//! What an adopter wires into CI: run 渾儀 against this crate, render the reaction reason-first,
//! and exit with its code. The rendering is *presentation only* — the verdict is
//! `outcome.exit_code()`.
use std::path::Path;
use std::process::ExitCode;

use api_hygiene::governance::constitution;
use hunyi::{check, Outcome};

fn main() -> ExitCode {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let outcome = check(&constitution(), &manifest);
    match &outcome {
        Outcome::Clean => println!("✓ clean — no forbidden type is exposed"),
        Outcome::Violations(report) => {
            for v in &report.violations {
                println!(
                    "⛒ {}\n    {} · {}\n    at: {}",
                    v.reason,
                    v.target,
                    v.finding,
                    v.file.as_deref().unwrap_or("(no single file)")
                );
            }
        }
        Outcome::ConstitutionError(msg) => eprintln!("constitution error: {msg}"),
        // `Outcome` is `#[non_exhaustive]` (it lives in 璇璣, shared across dimensions).
        _ => {}
    }
    ExitCode::from(outcome.exit_code())
}
