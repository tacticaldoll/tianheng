//! The gate-identity judgement's failure matrix.

use crate::gate_identity::{citations, logical_lines, offences, registered_names};
use crate::refusal::Kind;

const LISTING: &str = "module::ident: test\nthe_gate: test\nother::the_gate: test\n";

fn lists(_pkg: &str, _target: &str) -> Result<String, String> {
    Ok("the_gate: test\nsomething_else: test\n".to_string())
}

fn invocation(identifier: &str) -> String {
    format!(
        "cargo test --manifest-path x -p kanhe --test merge_message \\\n    -- --exact {identifier}\n"
    )
}

#[test]
fn a_wrapped_invocation_is_one_logical_line() {
    let joined = logical_lines("a \\\n  b \\\n  c\nnext\n");
    assert_eq!(joined, vec!["a    b    c".to_string(), "next".to_string()]);
}

#[test]
fn an_identifier_is_bound_to_the_target_of_its_own_invocation() {
    let found = citations("scripts/w.sh", &invocation("the_gate"));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].identifier, "the_gate");
    assert_eq!(found[0].target.as_deref(), Some("merge_message"));
    assert_eq!(found[0].package.as_deref(), Some("kanhe"));
}

#[test]
fn a_commented_invocation_cites_nothing() {
    assert!(
        citations(
            "scripts/w.sh",
            "# cargo test -p k --test t -- --exact ghost\n"
        )
        .is_empty()
    );
}

#[test]
fn a_registered_name_is_its_last_segment() {
    assert_eq!(
        registered_names(LISTING),
        vec!["ident", "the_gate", "the_gate"]
    );
}

#[test]
fn a_gate_the_target_does_not_register_is_a_violation() {
    let refusals = offences(
        &citations("scripts/w.sh", &invocation("renamed_away")),
        lists,
    );
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::Violation);
    assert!(refusals[0].message.contains("does not register"));
}

#[test]
fn a_gate_registered_twice_is_a_violation() {
    let refusals = offences(
        &citations("scripts/w.sh", &invocation("the_gate")),
        |_, _| Ok(LISTING.to_string()),
    );
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::Violation);
    assert!(refusals[0].message.contains("registers 2 times"));
}

#[test]
fn an_identifier_with_no_target_cannot_be_judged() {
    let refusals = offences(
        &citations("scripts/w.sh", "cargo test -- --exact loose\n"),
        lists,
    );
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    assert!(refusals[0].message.contains("names no `--test <target>`"));
}

#[test]
fn a_listing_that_cannot_be_read_cannot_be_judged() {
    let refusals = offences(
        &citations("scripts/w.sh", &invocation("the_gate")),
        |_, _| Err("cargo exploded".to_string()),
    );
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    assert!(refusals[0].message.contains("could not list"));
}

#[test]
fn a_gate_registered_once_is_clean() {
    assert!(offences(&citations("scripts/w.sh", &invocation("the_gate")), lists).is_empty());
}
