//! Unsafe-confinement reactions, asserted as runnable proof: the stray `unsafe` in `crate::net`
//! reacts, the confined `unsafe` in `crate::ffi` does not, and the confinement-only scope is a
//! constitution error (not a silent no-op) when asked to ban `unsafe` crate-wide.
use std::path::{Path, PathBuf};

use hunyi::{check_unsafe_confinement, Outcome, UnsafeBoundary};
use unsafe_confinement::governance::constitution;

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The core reaction: `net::peek` opens an `unsafe` block outside `crate::ffi` → exit 1, while the
/// confined `unsafe` in `crate::ffi` does not react (confinement, not a ban).
#[test]
fn the_stray_unsafe_reacts_with_exit_1() {
    let outcome = check_unsafe_confinement(&constitution(), &manifest());
    assert_eq!(outcome.exit_code(), 1);
    // Self-sufficiently non-vacuous: the violation names the stray site in `crate::net`, never the
    // confined `crate::ffi` one — the confinement reacts on the right module.
    if let Outcome::Violations(report) = &outcome {
        assert!(report.violations.iter().any(|violation| {
            violation.finding_key().namespace() == "hunyi"
                && violation.finding_key().code() == "unsafe_site"
        }));
        let findings: Vec<&str> = report.violations.iter().map(|v| v.finding.as_str()).collect();
        assert!(findings.iter().any(|f| f.contains("crate::net")), "{findings:?}");
        assert!(
            !findings.iter().any(|f| f.contains("crate::ffi")),
            "the confined ffi unsafe must not react: {findings:?}"
        );
    }
}

/// Precision: widening the allowed subtree to include `crate::net` makes the crate clean — the
/// `ffi` unsafe was never a violation, so nothing else reacts (exit 0).
#[test]
fn confining_both_modules_is_clean() {
    let boundary = vec![
        UnsafeBoundary::in_crate("unsafe_confinement")
            .only_under(["crate::ffi", "crate::net"])
            .because("both modules may hold unsafe"),
    ];
    assert_eq!(check_unsafe_confinement(&boundary, &manifest()).exit_code(), 0);
}

/// Confinement-only, enforced loud: an empty allowed set is NOT "ban unsafe crate-wide" (that is
/// `#![forbid(unsafe_code)]`'s stronger, compile-time job) — it is a constitution error (exit 2),
/// never a silent no-op that would pass a crate full of `unsafe`.
#[test]
fn an_empty_allowed_set_is_a_constitution_error() {
    let boundary = vec![
        UnsafeBoundary::in_crate("unsafe_confinement")
            .only_under(Vec::<&str>::new())
            .because("this would be a crate-wide ban — use #![forbid(unsafe_code)] instead"),
    ];
    assert_eq!(check_unsafe_confinement(&boundary, &manifest()).exit_code(), 2);
}
