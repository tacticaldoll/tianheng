//! Repository check: every member `tianheng::prelude` promises is named by the external compilation contract.
//!
//! The judgement is [`kanhe::prelude_promise::judge`]; this file supplies the two tracked inputs and the
//! failure matrix. The matrix matters more than usual here, because the real direction's verdict over a
//! repaired tree is that it found nothing — and a verdict of that shape survives the recognizer being
//! deleted, which is the trap the sibling reference gate records paying for.

use std::path::PathBuf;

use kanhe::prelude_promise::{Promise, judge, mentioned_identifiers, promised_members};

const LIB: &str = "crates/tianheng/src/lib.rs";
const CONTRACT: &str = "crates/tianheng/tests/adopter_surface.rs";

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

#[test]
fn every_prelude_member_is_named_by_the_external_contract() {
    let Some(root) = workspace_root() else {
        return;
    };
    let lib = std::fs::read_to_string(root.join(LIB)).expect("the shell's lib source is tracked");
    let contract = std::fs::read_to_string(root.join(CONTRACT))
        .expect("the external compilation contract is tracked");

    match judge(&lib, &contract) {
        Promise::Kept => {}
        Promise::Unnamed(names) => panic!(
            "{} promises {} name(s) that {CONTRACT} never mentions — an adopter reaches them through the \
             wildcard prelude, so a promise no external crate compiles against has never been reached the \
             way they reach it. Name each one in the form its kind admits:\n{}",
            LIB,
            names.len(),
            names
                .iter()
                .map(|name| format!("  {name}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
        Promise::CannotJudge(why) => panic!("prelude promise (cannot judge): {why}"),
    }
}

/// The promise is read from the prelude's own block, not from any `pub use super::{…}` in the file.
///
/// This repository's shell carries several of the latter. Keying on the `pub use` alone was the first shape
/// written and it collected the whole re-export surface — a different set under a different contract — so the
/// distinction is asserted rather than left to the reader of the parser.
#[test]
fn the_promise_is_the_prelude_block_and_not_a_sibling_reexport() {
    let elsewhere =
        "pub use super::{NotPromised};\n\npub mod prelude {\n    pub use super::{Promised};\n}\n";
    let members = promised_members(elsewhere);
    assert!(
        members.contains("Promised"),
        "the prelude block's own member must be read, got {members:?}"
    );
    assert!(
        !members.contains("NotPromised"),
        "a sibling `pub use super::{{…}}` is not the promise, got {members:?}"
    );
}

/// Each refusal direction, and each seen to refuse rather than assumed to.
#[test]
fn an_input_that_cannot_be_read_is_refused_rather_than_reported_clean() {
    let promise = "pub mod prelude {\n    pub use super::{Alpha, Beta};\n}\n";

    assert_eq!(
        judge(promise, "fn t() { let _ = (Alpha, Beta); }"),
        Promise::Kept,
        "a contract mentioning every promised member keeps the promise"
    );
    assert_eq!(
        judge(promise, "fn t() { let _ = Alpha; }"),
        Promise::Unnamed(vec!["Beta".to_string()]),
        "a promised member the contract never mentions is the disagreement this check exists for"
    );
    match judge("fn unrelated() {}\n", "fn t() { let _ = Alpha; }") {
        Promise::CannotJudge(why) => assert!(
            why.contains("no member"),
            "a file with no prelude block must refuse for that reason, got: {why}"
        ),
        other => panic!("a promise of nothing must be refused, not judged: {other:?}"),
    }
    match judge(promise, "") {
        Promise::CannotJudge(why) => assert!(
            why.contains("no identifier"),
            "an unread contract must refuse for that reason, got: {why}"
        ),
        other => panic!("an unread contract must be refused, not judged: {other:?}"),
    }
}

/// A mention is an identifier, so a substring of one is not a mention.
///
/// `Run` inside `RuntimeObserver` would satisfy a substring reader while the contract never names `Run`, and
/// the promise carries both — which is exactly the pair that would have hidden.
#[test]
fn a_longer_identifier_containing_a_promised_name_is_not_a_mention() {
    let mentioned = mentioned_identifiers("fn t() { let _ = RuntimeObserver; }");
    assert!(
        mentioned.contains("RuntimeObserver"),
        "the identifier itself is mentioned, got {mentioned:?}"
    );
    assert!(
        !mentioned.contains("Run"),
        "a promised name embedded in a longer identifier is not a mention, got {mentioned:?}"
    );
}
