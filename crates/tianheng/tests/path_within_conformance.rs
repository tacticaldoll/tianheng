//! Cross-dimension conformance for `::`-boundary-safe containment — 圭表 (`guibiao`) and 渾儀
//! (`hunyi`) each independently hand-roll a `path_within` predicate (`path == prefix ||
//! path.starts_with("{prefix}::")`, guarding against a bare `starts_with` that would admit a
//! namesake sibling: forbidding `crate::domain` must never also match `crate::domainish`). 三儀 ⊥
//! 三儀 means they cannot share the function itself — 圭表's own copy states outright that the two
//! dimensions "agree by using the same rule, not the same function" — but nothing had previously
//! fed the SAME descendant-vs-sibling shape to both and asserted they agree, unlike the sibling
//! lexical-hygiene ledger (`lexical_conformance.rs`) this mirrors.
//!
//! Each dimension exercises the property through its own real capability (圭表's
//! `must_not_import`, 渾儀's `UnsafeBoundary::only_under`) rather than the private `path_within`
//! internals, matching `lexical_conformance.rs`'s black-box-through-the-public-surface style.

use std::path::{Path, PathBuf};

use guibiao::{Constitution as GnomonConstitution, ModuleBoundary, Outcome as GnomonOutcome};
use hunyi::{Outcome as HunyiOutcome, UnsafeBoundary, check_unsafe_confinement};

/// Write a minimal, dependency-free crate (so `cargo metadata --no-deps` never touches the
/// network) with `lib.rs` set to `body`, and return its manifest path.
fn write_fixture(name: &str, body: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "tianheng-path-within-conformance-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    std::fs::create_dir_all(&src).expect("create temp src");
    std::fs::write(
        dir.join("Cargo.toml"),
        format!("[package]\nname = \"{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n"),
    )
    .expect("write Cargo.toml");
    std::fs::write(src.join("lib.rs"), body).expect("write lib.rs");
    dir.join("Cargo.toml")
}

fn guibiao_forbids_domain(package: &str, manifest: &Path) -> GnomonOutcome {
    let constitution = GnomonConstitution::new(package).boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_import("crate::domain")
            .because("conformance: containment must not treat a namesake sibling as contained"),
    );
    guibiao::check(&constitution, manifest)
}

fn hunyi_confines_unsafe_to_domain(package: &str, manifest: &Path) -> HunyiOutcome {
    let boundary = UnsafeBoundary::in_crate(package)
        .only_under(["crate::domain"])
        .because("conformance: containment must not treat a namesake sibling as contained");
    check_unsafe_confinement(&[boundary], manifest)
}

#[test]
fn guibiao_and_hunyi_agree_a_descendant_of_the_governed_subtree_is_contained() {
    let manifest = write_fixture(
        "path-within-descendant-guibiao",
        "pub mod domain { pub mod inner { pub struct Thing; } }\nuse crate::domain::inner::Thing;\n",
    );
    let outcome = guibiao_forbids_domain("path-within-descendant-guibiao", &manifest);
    let _ = std::fs::remove_dir_all(manifest.parent().expect("fixture has a parent"));
    assert_eq!(
        outcome.exit_code(),
        1,
        "圭表: a descendant of the forbidden subtree must react: {outcome:?}"
    );

    let manifest = write_fixture(
        "path-within-descendant-hunyi",
        "pub mod domain { pub mod inner { pub fn f() { unsafe {} } } }\n",
    );
    let outcome = hunyi_confines_unsafe_to_domain("path-within-descendant-hunyi", &manifest);
    let _ = std::fs::remove_dir_all(manifest.parent().expect("fixture has a parent"));
    assert_eq!(
        outcome.exit_code(),
        0,
        "渾儀: unsafe inside the allowed subtree's descendant must NOT react: {outcome:?}"
    );
}

#[test]
fn guibiao_and_hunyi_agree_a_namesake_sibling_is_not_contained() {
    let manifest = write_fixture(
        "path-within-sibling-guibiao",
        "pub mod domainish { pub struct Thing; }\nuse crate::domainish::Thing;\n",
    );
    let outcome = guibiao_forbids_domain("path-within-sibling-guibiao", &manifest);
    let _ = std::fs::remove_dir_all(manifest.parent().expect("fixture has a parent"));
    assert_eq!(
        outcome.exit_code(),
        0,
        "圭表: importing a namesake sibling of the forbidden subtree must NOT react: {outcome:?}"
    );

    let manifest = write_fixture(
        "path-within-sibling-hunyi",
        "pub mod domainish { pub fn f() { unsafe {} } }\n",
    );
    let outcome = hunyi_confines_unsafe_to_domain("path-within-sibling-hunyi", &manifest);
    let _ = std::fs::remove_dir_all(manifest.parent().expect("fixture has a parent"));
    assert_eq!(
        outcome.exit_code(),
        1,
        "渾儀: unsafe in a namesake sibling of the allowed subtree must react: {outcome:?}"
    );
}
