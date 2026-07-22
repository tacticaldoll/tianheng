//! Cross-dimension conformance for a plain `mod child;` backed by BOTH conventional file forms at
//! once (`child.rs` AND `child/mod.rs`) — a genuine rustc compile error (E0761) independent of any
//! `#[cfg]`, unlike a missing file. 漏刻's own `resolve_external_module`
//! (`crates/louke/src/audit/scan.rs`) already hard-errored on this shape; 圭表's `reachability.rs`
//! previously silently accepted both as separate sources — a pre-existing BACKLOG debt, closed
//! alongside this test. Pins that the two independently hand-written scanners (三儀 ⊥ 三儀: no
//! shared scanner code) now agree the shape is a genuine ambiguity, mirroring
//! `path_within_conformance.rs`'s style.

use std::path::Path;

use guibiao::{Constitution as GnomonConstitution, ModuleBoundary, Outcome as GnomonOutcome};
use louke::{Outcome as LoukeOutcome, RuntimeBoundary, audit_probe_coverage};

#[path = "support/mod.rs"]
mod support;
use support::TempFixture;

fn guibiao_scans(package: &str, manifest: &Path) -> GnomonOutcome {
    let constitution = GnomonConstitution::new(package).boundary(
        ModuleBoundary::in_crate(package)
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("conformance: a dual-backed module must fail loud identically"),
    );
    guibiao::check(&constitution, manifest)
}

fn louke_scans(root: &Path) -> LoukeOutcome {
    let boundary = RuntimeBoundary::at("conformance-seam")
        .only_origins(["o"])
        .because("conformance: a dual-backed module must fail loud identically");
    audit_probe_coverage(&[boundary], &[root.to_path_buf()])
}

#[test]
fn guibiao_and_louke_agree_a_dual_backed_module_is_a_scan_error() {
    let fixture = TempFixture::new("dual-backed-module", "pub mod child;\n");
    let src = fixture.lib().parent().expect("lib.rs has a parent");
    std::fs::write(src.join("child.rs"), "// flat form\n").expect("write child.rs");
    std::fs::create_dir_all(src.join("child")).expect("mkdir child");
    std::fs::write(src.join("child").join("mod.rs"), "// nested form\n").expect("write mod.rs");

    let guibiao_outcome = guibiao_scans("dual-backed-module", fixture.manifest());
    assert_eq!(
        guibiao_outcome.exit_code(),
        2,
        "圭表: a dual-backed module must be a constitution error (cannot judge): {guibiao_outcome:?}"
    );

    let louke_outcome = louke_scans(fixture.lib());
    assert_eq!(
        louke_outcome.exit_code(),
        2,
        "漏刻: a dual-backed module must be a constitution error (cannot judge): {louke_outcome:?}"
    );
}
