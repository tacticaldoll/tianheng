//! `publish-source-integrity`'s declared bound, demonstrated.
//!
//! It builds its fixture through `support/publish_source_gate.rs`, the same builder the failure matrix uses.
//! Two constructions of "a signed release repository" would be the twin-drift class this repository keeps
//! closing, and the whole reason the builder is shared.

use kanhe::publish_source_gate as gate;

use gate::{build_fixture, judge};
use std::path::PathBuf;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// `publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the verification environment. The gate verifies that the tag's signature is
/// cryptographically valid over the tag object — which needs no configuration — and does **not** verify that
/// the signing key is one a maintainer authorized, which needs an allowed-signers file that exists on a
/// maintainer's machine and not in CI.
///
/// Demonstrated rather than asserted in prose: the fixture is signed by an ephemeral key named in no
/// allowed-signers file anywhere, and every git invocation building it runs with `GIT_CONFIG_GLOBAL` and
/// `GIT_CONFIG_SYSTEM` at `/dev/null`, so no ambient configuration can name it either. The gate accepts it.
///
/// A previous form of this pin built such a fixture and then ran a placeholder test against **this**
/// repository, ignoring the fixture entirely. It passed for a reason that had nothing to do with the bound.
#[test]
fn a_valid_signature_from_an_unauthorized_key_is_accepted() {
    let Some(_) = workspace_root() else {
        return;
    };
    let root = std::env::temp_dir().join(format!(
        "tianheng-publish-source-integrity-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the fixture root is writable");

    let fixture = build_fixture(&root, "unauthorized", "9.9.9");

    // The premise, stated rather than trusted: nothing authorizes this key. If an allowed-signers file were
    // reachable, the acceptance below would say nothing about the bound.
    assert!(
        fixture.key.is_file(),
        "the fixture must hold its own ephemeral key, or it demonstrates nothing"
    );
    // What keeps the signer unauthorized is that every fixture command runs hermetically — asserting the
    // scratch root holds no allowed-signers file asserts what the two statements above already guarantee.

    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);

    assert!(
        verdict.is_ok(),
        "the gate must accept a cryptographically valid signature whose signer nothing authorized — that is \
         the declared bound, and a refusal here would mean the bound had closed and this citation should be \
         retired. Got: {:?}",
        verdict.err()
    );
}
