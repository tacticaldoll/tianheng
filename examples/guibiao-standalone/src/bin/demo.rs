//! What an adopter wires into CI: run 圭表 against this crate, render the reaction, and exit
//! with its code. The rendering is *presentation only* — the verdict is `outcome.exit_code()`,
//! and reads reason-first (the repair direction), then where, then what.
use std::path::Path;
use std::process::ExitCode;

use guibiao::{check, Outcome};
use hexagonal_demo::governance::constitution;

fn main() -> ExitCode {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let outcome = check(&constitution(), &manifest);
    match &outcome {
        Outcome::Clean => println!("✓ clean — no boundary drifted"),
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
