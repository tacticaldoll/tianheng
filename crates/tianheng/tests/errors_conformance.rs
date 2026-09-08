//! Cross-dimension conformance for the "cannot judge" (exit-2) constitution-error wordings —
//! 圭表 (`guibiao`) and 渾儀 (`hunyi`) each carry their own `errors.rs`, with several builder pairs
//! whose doc comments claim byte-identical or deliberately-parallel agreement (`unreadable_workspace_error`
//! and `crate_not_found_error` "MUST stay byte-identical"; `unknown_module_error` a declared
//! "parallel twin"), but neither module is reachable from outside its own crate (`pub(crate)`), so
//! nothing had previously fed the same failure to both through their real public surfaces and
//! asserted the claim holds — unlike the sibling lexical-hygiene ledger (`lexical_conformance.rs`)
//! this mirrors.
//!
//! Stated bound, not a silent gap: `missing_src_error`'s declared parallel twin is NOT pinned
//! here. Constructing a fixture that reaches it (a package `cargo metadata` accepts but whose
//! `crate_root_file` target search comes up empty) kept landing on a different, unrelated
//! constitution error specific to each dimension's own module-target resolution rather than the
//! shared "no src" spine; closing that gap needs a fixture shape neither dimension's public
//! surface makes easy to construct, not a reason to assert a false pass.

use std::path::{Path, PathBuf};

use guibiao::{
    Constitution as GnomonConstitution, CrateBoundary, ModuleBoundary, Outcome as GnomonOutcome,
};
use hunyi::{AsyncExposureBoundary, Outcome as HunyiOutcome, UnsafeBoundary, check_async_exposure};

#[path = "support/mod.rs"]
mod support;
use support::TempFixture;

fn gnomon_error(outcome: GnomonOutcome) -> String {
    match outcome {
        GnomonOutcome::ConstitutionError(message) => message,
        other => panic!("expected a constitution error, got {other:?}"),
    }
}

fn hunyi_error(outcome: HunyiOutcome) -> String {
    match outcome {
        HunyiOutcome::ConstitutionError(message) => message,
        other => panic!("expected a constitution error, got {other:?}"),
    }
}

#[test]
fn guibiao_and_hunyi_agree_verbatim_on_an_unreadable_workspace() {
    // Neither dimension names a boundary here — reading the workspace is the first gate either
    // opens with, before any boundary is even inspected (see hunyi::driver::read_metadata).
    let manifest = PathBuf::from("/tianheng-conformance-nonexistent/Cargo.toml");

    let guibiao_message = gnomon_error(guibiao::check(
        &GnomonConstitution::new("conformance"),
        &manifest,
    ));
    let hunyi_message = hunyi_error(check_async_exposure(&[], &manifest));

    assert_eq!(
        guibiao_message, hunyi_message,
        "an unreadable workspace's message MUST stay byte-identical across dimensions"
    );
}

#[test]
fn guibiao_and_hunyi_agree_verbatim_on_a_crate_not_in_the_workspace() {
    let fixture = TempFixture::new("errors-crate-not-found", "pub fn f() {}\n");
    let manifest = fixture.manifest();

    let guibiao_message = gnomon_error(guibiao::check(
        &GnomonConstitution::new("errors-crate-not-found").boundary(
            CrateBoundary::crate_("nonexistent-crate")
                .restrict_dependencies_to(["serde_json"])
                .because("conformance: a nonexistent crate target must fail loud identically"),
        ),
        manifest,
    ));
    let hunyi_message = hunyi_error(check_unsafe_confinement_on("nonexistent-crate", manifest));

    assert_eq!(
        guibiao_message, hunyi_message,
        "a crate absent from the workspace's message MUST stay byte-identical across dimensions"
    );
}

fn check_unsafe_confinement_on(package: &str, manifest: &Path) -> HunyiOutcome {
    let boundary = UnsafeBoundary::in_crate(package)
        .only_under(["crate::somewhere"])
        .because("conformance: a nonexistent crate target must fail loud identically");
    hunyi::check_unsafe_confinement(&[boundary], manifest)
}

#[test]
fn guibiao_and_hunyi_agree_on_the_parallel_unknown_module_wording() {
    let fixture = TempFixture::new("errors-unknown-module", "pub fn f() {}\n");
    let manifest = fixture.manifest();

    let guibiao_message = gnomon_error(guibiao::check(
        &GnomonConstitution::new("errors-unknown-module").boundary(
            ModuleBoundary::in_crate("errors-unknown-module")
                .module("crate::does_not_exist")
                .must_not_import("crate::y")
                .because("conformance: an unreachable module target must fail loud identically"),
        ),
        manifest,
    ));
    let hunyi_message = hunyi_error(check_async_exposure(
        &[AsyncExposureBoundary::in_crate("errors-unknown-module")
            .module("crate::does_not_exist")
            .must_not_expose_async_fn()
            .because("conformance: an unreachable module target must fail loud identically")],
        manifest,
    ));

    // Declared a *parallel*, not verbatim, twin: same principle preamble and "check the path"
    // tail, differing only in the dimension-accurate detail. Pin the shared spine, not the whole
    // string, so the declared, honest wording difference does not itself fail this gate.
    for shared in [
        "a boundary must anchor to a real module or it silently never reacts: module",
        "'crate::does_not_exist'",
        "check the path",
    ] {
        assert!(
            guibiao_message.contains(shared),
            "圭表's unknown-module message dropped the shared spine {shared:?}: {guibiao_message:?}"
        );
        assert!(
            hunyi_message.contains(shared),
            "渾儀's unknown-module message dropped the shared spine {shared:?}: {hunyi_message:?}"
        );
    }
}
