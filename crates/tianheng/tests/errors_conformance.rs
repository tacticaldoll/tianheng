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
//! here. Both dimensions reach it only through synthetic metadata — Cargo refuses a manifest
//! declaring no target before `cargo metadata` would report one, and every public surface these
//! directions feed takes a manifest, not a metadata value. The state that once shared this
//! bound's premise — `cargo metadata` accepted, the target search coming up empty — is the
//! example-only package pinned below, which each dimension answers with its
//! `no_compiled_root_error` twin.

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

/// A package whose every target is an example compiles no root: Cargo accepts the manifest and
/// reports the example as its one target, and both dimensions SHALL refuse it with the parallel
/// `no_compiled_root_error` wording. The fixture's `src/lib.rs` exists, so a "cannot locate the
/// crate root source" answer would be false of it — the divergence this direction forbids.
#[test]
fn guibiao_and_hunyi_agree_on_the_parallel_no_compiled_root_wording() {
    let fixture = TempFixture::with_manifest_extra(
        "errors-no-compiled-root",
        "autolib = false\nautobins = false\n\n[[example]]\nname = \"e\"\npath = \"examples/e.rs\"\n",
        "pub fn f() {}\n",
    );
    fixture.write("examples/e.rs", "fn main() {}\n");
    let manifest = fixture.manifest();

    let guibiao_message = gnomon_error(guibiao::check(
        &GnomonConstitution::new("errors-no-compiled-root").boundary(
            ModuleBoundary::in_crate("errors-no-compiled-root")
                .module("crate::seam")
                .must_not_import("crate::forbidden")
                .because("conformance: a package compiling no root must fail loud identically"),
        ),
        manifest,
    ));
    let hunyi_message = hunyi_error(check_async_exposure(
        &[AsyncExposureBoundary::in_crate("errors-no-compiled-root")
            .module("crate::seam")
            .must_not_expose_async_fn()
            .because("conformance: a package compiling no root must fail loud identically")],
        manifest,
    ));

    // Declared a *parallel*, not verbatim, twin: same principle and detail, differing only in the
    // dimension noun. Pin the shared spine, plus each side's own noun.
    for shared in [
        "boundary is observed from a compiled crate root",
        "'errors-no-compiled-root' has none: no target Cargo reports for it is a library or a binary",
        "so nothing its src directory holds is compiled into a root this boundary could govern",
    ] {
        assert!(
            guibiao_message.contains(shared),
            "圭表's no-compiled-root message dropped the shared spine {shared:?}: {guibiao_message:?}"
        );
        assert!(
            hunyi_message.contains(shared),
            "渾儀's no-compiled-root message dropped the shared spine {shared:?}: {hunyi_message:?}"
        );
    }
    assert!(
        guibiao_message.contains("a module boundary"),
        "圭表's refusal names its own dimension: {guibiao_message:?}"
    );
    assert!(
        hunyi_message.contains("a semantic boundary"),
        "渾儀's refusal names its own dimension: {hunyi_message:?}"
    );
}
