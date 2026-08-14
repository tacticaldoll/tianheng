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
/// A reader keyed on the marker alone agrees with this one exactly while no sibling re-export of that form
/// exists, and measured when this was written none did — so running it against the real shell could not have
/// told the two apart. The fixture supplies the sibling the shell does not have, which is the only way to
/// assert the distinction before it costs something.
#[test]
fn the_promise_is_the_prelude_block_and_not_a_sibling_reexport() {
    let elsewhere =
        "pub use super::{NotPromised};\n\npub mod prelude {\n    pub use super::{Promised};\n}\n";
    let members = promised_members(elsewhere).expect("both members are plain identifiers");
    assert!(
        members.contains("Promised"),
        "the prelude block's own member must be read, got {members:?}"
    );
    assert!(
        !members.contains("NotPromised"),
        "a sibling `pub use super::{{…}}` is not the promise, got {members:?}"
    );
}

/// Every re-export statement in the block, with the second being the one the old reader dropped.
///
/// Two of the thing is not enough on its own: the falsifier has to be the candidate the reader would have
/// discarded, asserted present in the result. Here that is `Second`, in a statement after the first — which
/// `split_once("pub use super::{")` never reached, so every member of it became a promised member no external
/// contract had to name.
///
/// `First` is the control: it holds whether the second statement is read at all apart from whether the first
/// still is. The negative run needs no edit to this test — restore `split_once` in place of the split over
/// every statement and `Second` disappears.
///
/// Read rather than refused, deliberately: a prelude split across two `pub use super::{…}` is legal and
/// ordinary Rust, so refusing it would be a false refusal over a well-stated promise. Refusal is this
/// module's answer for forms it *cannot* read — a path, a rename, a nested group, a glob — and a second
/// statement of a form it already reads is not one of those.
#[test]
fn every_reexport_statement_in_the_prelude_block_is_read() {
    let two = "pub mod prelude {\n    pub use super::{First};\n    pub use super::{Second};\n}\n";
    let members = promised_members(two).expect("both members are plain identifiers");
    assert!(
        members.contains("First"),
        "the first statement must still be read, got {members:?}"
    );
    assert!(
        members.contains("Second"),
        "the second statement is the one the first-only reader dropped; a member missing here is a promised \
         member no contract must name: {members:?}"
    );

    // The sibling distinction still holds with several statements in the block — a re-export outside the
    // module is not the promise, however many the block itself carries.
    let with_sibling = "pub use super::{Outside};\n\npub mod prelude {\n    pub use super::{First};\n    \
                        pub use super::{Second};\n}\n";
    let members = promised_members(with_sibling).expect("all three are plain identifiers");
    assert!(
        !members.contains("Outside"),
        "widening to every statement must not widen past the prelude block, got {members:?}"
    );
}

/// Two `pub mod prelude {` markers are reported, not resolved by position.
///
/// The sibling of the statement case above, and the opposite answer: statements inside one block are unioned
/// because a split promise is still one promise, while two blocks are two promises and picking either by file
/// order would decide the question silently. The second marker here is inside a doc comment, which is the
/// reachable form — a nested `pub mod prelude` is legal too, but a comment needs no code to appear.
#[test]
fn several_prelude_markers_are_reported_rather_than_decided_by_position() {
    let two_blocks = "/// see `pub mod prelude {` for the promise\npub mod prelude {\n    \
                      pub use super::{Real};\n}\n";
    let verdict = judge(two_blocks, "fn contract(_: Real) {}");
    match verdict {
        Promise::CannotJudge(why) => assert!(
            why.contains("2") && why.contains("decided by whichever comes first"),
            "the refusal must say how many and why it is not resolved: {why}"
        ),
        other => panic!("two prelude markers must not be judged by position, got {other:?}"),
    }
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

/// Each unreadable member form, refused rather than read as something smaller.
///
/// The **glob** is the row that was missing, and it was the one that mattered. A `trim_end_matches("::*")`
/// turned `runner::*` into the plain identifier `runner`, which passes the member test and enters the promise
/// as one member — so every name behind the glob went unchecked while the promised set read as complete. One
/// glob re-export could have emptied this check of most of its subject, in the file whose entire purpose is
/// catching a promise that narrowed unobserved.
///
/// The three forms the doc already named are here as controls: without them this row would pass for a reader
/// that refuses everything, which is a different defect in the same place.
#[test]
fn a_member_form_this_reader_cannot_understand_is_refused_rather_than_narrowed() {
    for (form, member) in [
        ("runner::*", "a glob"),
        ("runner::Format", "a path"),
        ("Foo as Bar", "a rename"),
    ] {
        let promise = format!("pub mod prelude {{\n    pub use super::{{Alpha, {form}}};\n}}\n");
        match judge(
            &promise,
            "fn t() { let _ = (Alpha, runner, Format, Foo, Bar); }",
        ) {
            Promise::CannotJudge(why) => assert!(
                why.contains(form),
                "{member} must be refused by name so its author can act on it, got: {why}"
            ),
            other => panic!(
                "{member} `{form}` was read as a member instead of refused, which narrows the promise by \
                 exactly what the reader could not parse: {other:?}"
            ),
        }
    }
}

/// The bound: a promised member named **only in a comment** is counted as named.
///
/// `repository-checks/whether-a-mention-compiles-anything-is-not-observed-a-stated-bound`, `UnderReacts`, owned
/// by the engine. The check asks whether the promise was noticed at all; deciding that a mention is
/// load-bearing is a judgement over text this repository has designed, measured and rejected, and what makes a
/// mention bite is the compiler.
///
/// Both directions on one contract, differing only by the comment line. Without the control the silence is
/// satisfiable by a judgement that never reports anything, and the direction this bound declares is *silence* —
/// which is exactly the shape that cannot be pinned by a test that reacts.
#[test]
fn a_member_named_only_in_a_comment_is_counted_as_named() {
    let promise = "pub mod prelude {\n    pub use super::{Alpha, Beta};\n}\n";

    // The control: with `Beta` nowhere in the contract, the promise is not kept.
    assert_eq!(
        judge(promise, "fn t() { let _ = Alpha; }\n"),
        Promise::Unnamed(vec!["Beta".to_string()]),
        "a promised member the contract never mentions must be reported, or the bound below proves nothing"
    );

    // The bound: the only thing added is a comment, and the promise now reads as kept.
    assert_eq!(
        judge(
            promise,
            "fn t() { let _ = Alpha; }\n// Beta is named here and nowhere else.\n"
        ),
        Promise::Kept,
        "a mention inside a comment counts as named, which is the declared stop"
    );
}

/// A promised member this check cannot read is **refused**, not dropped.
///
/// Measured before the repair, on a mixed list: `{Alpha, runner::Format, Foo as Bar, a::{B, C}, Beta}` parsed
/// to `{Alpha, Beta}` — three of five members gone, silently, and the promise narrowed to whatever the parser
/// happened to understand. The prelude is a flat list of identifiers today, so nothing was wrong; what was
/// wrong is that nothing would have said so.
///
/// Neither declared as a bound nor closed by widening. Declaring it would put a **false negative** in a check
/// whose whole subject is a promise narrowing unobserved, and widening means new extraction rules — last path
/// segment, post-`as` name — in a hand-rolled reader, with no pressure asking for them. Refusing needs no rule
/// at all and cannot narrow anything: if the prelude ever grows one of these forms, its author meets a refusal
/// naming the member instead of silence.
#[test]
fn a_promised_member_the_parser_cannot_read_is_refused() {
    let contract = "fn t() { let _ = (Alpha, Beta); }";
    for form in ["runner::Format", "Foo as Bar", "a::{B", "C}"] {
        let promise =
            format!("pub mod prelude {{\n    pub use super::{{Alpha, {form}, Beta}};\n}}\n");
        match judge(&promise, contract) {
            Promise::CannotJudge(why) => assert!(
                why.contains(form),
                "the refusal must name the member it could not read, got {why:?}"
            ),
            other => panic!("`{form}` must be refused rather than dropped, got {other:?}"),
        }
    }

    // The control: a promise this check can read is still judged, so the refusals above are about the member
    // rather than about a parser that has stopped answering.
    let plain = "pub mod prelude {\n    pub use super::{Alpha, Beta};\n}\n";
    assert_eq!(
        judge(plain, contract),
        Promise::Kept,
        "a flat identifier list must still be judged"
    );
}

/// A mention is an identifier, so a substring of one is not a mention.
///
/// The promise carries both `Run` and `RuntimeObserver`, and the shorter sits inside the longer — so a
/// substring reader reports `Run` named by any mention of the observer. That is what it would have hidden when
/// the contract named neither, and the fixture keeps the distinction asserted now that it names both.
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
