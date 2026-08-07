//! What an adopter wires into CI: one composed run, one exit code, covering a family dimension and this
//! crate's own participant alike. The rendering is presentation only — the verdict is `outcome.exit_code()`.

use std::process::ExitCode;

use house_rules::governance::{participant, verdict};
use tianheng::prelude::*;

fn main() -> ExitCode {
    let outcome = verdict();
    match &outcome {
        Outcome::Clean => println!("✓ clean — every boundary and every house rule holds"),
        Outcome::Violations(report) => {
            for violation in &report.violations {
                println!(
                    "⛒ {}\n    {} · {}\n    at: {}",
                    violation.reason,
                    violation.rule,
                    violation.finding,
                    violation.file.as_deref().unwrap_or("(no single file)")
                );
            }
        }
        Outcome::ConstitutionError(message) => eprintln!("constitution error: {message}"),
        // `Outcome` is `#[non_exhaustive]`: it lives in 璇璣 and is shared across dimensions, so an
        // outside participant matches it the same way a family crate does.
        _ => {}
    }

    // What the participant declines to see, printed beside the verdict rather than left in its source. A
    // reader deciding whether to trust a clean run needs the limits of the thing that produced it.
    println!("\nthis participant's declared bounds:");
    for bound in participant().bounds() {
        println!("  · {} — {}", bound.id().as_str(), bound.shape());
        // The pin is printed too, and not only for a reader: it is what lets the dogfood gate resolve every
        // citation this participant computes against its own test harness. Left in the source, a computed pin
        // naming a test that does not exist reads as coverage while defending nothing — which is what the
        // second of these two was doing.
        if let Some(tests) = bound.defence().pinning_tests() {
            for test in tests {
                println!("      pinned by: {test}");
            }
        }
    }

    ExitCode::from(outcome.exit_code())
}
