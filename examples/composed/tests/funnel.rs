//! The funnel, stage by stage: each instrument reacts to its own fault. Asserted through the
//! per-dimension checks (which return an inspectable `Outcome`), so each stage's exit code is
//! provable — the composed CLI (`bin/check`) then projects all of them into one code.
use std::path::{Path, PathBuf};

use tianheng::{check, check_semantic, GnomonConstitution, ModuleBoundary, SemanticBoundary};

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// Stage 1 — 圭表: the `domain → infra` import reacts (exit 1).
#[test]
fn the_static_instrument_reacts() {
    let c = GnomonConstitution::new("composed_app").boundary(
        ModuleBoundary::in_crate("composed_app")
            .module("crate::domain")
            .must_not_import("crate::infra")
            .because("the domain stays pure"),
    );
    assert_eq!(check(&c, &manifest()).exit_code(), 1);
}

/// Stage 2 — 渾儀: the `api` leak of `infra::DbPool` reacts (exit 1).
#[test]
fn the_semantic_instrument_reacts() {
    let boundaries = vec![SemanticBoundary::in_crate("composed_app")
        .module("crate::api")
        .must_not_expose("crate::infra::DbPool")
        .because("the public API must not leak the internal database pool")];
    assert_eq!(check_semantic(&boundaries, &manifest()).exit_code(), 1);
}
