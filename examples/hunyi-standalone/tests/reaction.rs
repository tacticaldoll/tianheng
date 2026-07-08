//! 渾儀-standalone reactions, asserted as runnable proof: the public-API exposure reaction, and
//! its precision (a forbidden type that is *not* exposed does not react).
use std::path::{Path, PathBuf};

use api_hygiene::governance::constitution;
use hunyi::{check, SemanticBoundary};

fn manifest() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The core reaction: `api::connection` returns `infra::DbPool`, a leak → exit 1.
#[test]
fn the_api_leak_reacts_with_exit_1() {
    assert_eq!(check(&constitution(), &manifest()).exit_code(), 1);
}

/// Precision: forbidding a type that is *not* named on any public surface does not react — the
/// dimension reacts to real exposure, not to the mere existence of the type.
#[test]
fn a_type_that_is_not_exposed_does_not_react() {
    let boundary = vec![SemanticBoundary::in_crate("api_hygiene")
        .module("crate::api")
        .must_not_expose("crate::infra::Secret")
        .because("a type never exposed must not react")];
    assert_eq!(check(&boundary, &manifest()).exit_code(), 0);
}
