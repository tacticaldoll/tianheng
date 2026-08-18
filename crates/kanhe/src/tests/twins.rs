//! The failure matrix for [`crate::twins`]: what the reader counts, and what it declines to.

use crate::refusal;
use crate::twins::{STATEMENTS, WINDOW, judge, twins};

/// A module carrying `body`, as the corpus is spelled.
fn module(path: &str, body: &str) -> (String, String) {
    (path.to_string(), body.to_string())
}

/// Four executed statements, enough to fill a window on their own.
const FOUR: &str = "let a = one();\nlet b = two(a);\nlet c = three(b);\nlet d = four(c);\n";

/// One implementation in two modules is refused, and the refusal names both sites.
#[test]
fn a_window_two_modules_share_is_a_violation() {
    let corpus = [module("a.rs", FOUR), module("b.rs", FOUR)];
    let refusal =
        judge(&corpus).expect_err("two modules carrying one window disagree with the rule");
    refusal::expect(
        "repository-checks#one-implementation-in-two-modules",
        &refusal,
    );
    assert!(
        refusal.message.contains("a.rs:1") && refusal.message.contains("b.rs:1"),
        "the refusal must name every site, got: {}",
        refusal.message
    );
}

/// The same window twice **inside one module** is not this check's subject.
///
/// It is the shape a reader would expect to be reported and is deliberately not: this check is about a rule
/// with two owners, and a repetition one module can see is a repetition it can remove without an extraction.
#[test]
fn a_window_repeated_inside_one_module_is_not_reported() {
    let corpus = [
        module("a.rs", &format!("{FOUR}let e = five();\n{FOUR}")),
        module("b.rs", "let z = other();\n"),
    ];
    assert!(
        judge(&corpus).is_ok(),
        "a repetition within one module is not two modules holding one implementation"
    );
}

/// Rust's own skeleton is written the same way in every module, and reporting it reports the language.
///
/// The corpus rule that closes this is *executed statements, not item declarations*. Measured on this
/// repository: without it the same window reports four extra findings — `#[cfg(test)] mod tests { use
/// super::*;` across four modules, and a closing assertion followed by `#[test]` across five.
#[test]
fn the_item_skeleton_two_modules_share_is_not_a_twin() {
    let skeleton = "#[cfg(test)]\nmod tests {\nuse super::*;\n#[test]\n";
    let corpus = [module("a.rs", skeleton), module("b.rs", skeleton)];
    assert!(
        judge(&corpus).is_ok(),
        "attributes, `mod` and `use` declare rather than execute, so a window of them is Rust's shape \
         rather than a shared implementation"
    );
}

/// A window whose only shared content is closers carries no implementation.
#[test]
fn a_window_of_closers_is_not_a_twin() {
    let closers = "let a = one();\n);\n}\n}\n";
    let corpus = [module("a.rs", closers), module("b.rs", closers)];
    assert!(
        judge(&corpus).is_ok(),
        "one statement among three closers is below the statement floor, so two modules ending a call the \
         same way is not a shared implementation"
    );
    assert_eq!(STATEMENTS, 2, "and that floor is what this direction reads");
}

/// **A comment is not executed text**, so two modules explaining themselves alike are not two implementations.
///
/// The corpus comes from `crate::region`, which `repository-checks` requires of a check deciding a property
/// over executed text — and this is the direction that holds it, rather than the requirement being satisfied
/// by the import.
#[test]
fn two_modules_carrying_one_comment_are_not_a_twin() {
    let commented = "// one\n// two\n// three\n// four\nlet a = one();\n";
    let corpus = [module("a.rs", commented), module("b.rs", commented)];
    assert!(
        judge(&corpus).is_ok(),
        "a shared comment is not a shared implementation; the corpus is executed text"
    );
}

/// Where rustfmt broke a line is not part of what a module implements.
#[test]
fn a_window_differing_only_in_line_wrapping_is_still_a_twin() {
    let wrapped = "let a = one();\nlet b =\n    two(a);\nlet c = three(b);\nlet d = four(c);\n";
    // The same statements, wrapped: collapsing whitespace does not join the two physical lines, so this
    // direction asserts what the reader actually does rather than what the sentence above might suggest.
    let corpus = [module("a.rs", FOUR), module("b.rs", wrapped)];
    let found = twins(&corpus);
    assert!(
        found.is_empty(),
        "collapsing whitespace normalises *within* a line and never across one, which is a limit of this \
         reader and is stated here rather than implied: {found:?}"
    );
}

/// A corpus that collapsed to nothing is refused rather than reported clean.
#[test]
fn a_corpus_that_is_not_a_set_cannot_be_judged() {
    let refusal = judge(&[]).expect_err("no modules is no question");
    refusal::expect("repository-checks#twin-corpus-is-not-a-set", &refusal);
    let one = judge(&[module("a.rs", FOUR)]).expect_err("one module cannot share with another");
    refusal::expect("repository-checks#twin-corpus-is-not-a-set", &one);
}

/// The window is long enough that four statements fill exactly one.
#[test]
fn the_window_is_the_length_the_directions_above_assume() {
    assert_eq!(WINDOW, 4);
    let corpus = [module("a.rs", FOUR), module("b.rs", FOUR)];
    assert_eq!(
        twins(&corpus).len(),
        1,
        "four shared statements are one window, so the matrix above reads one finding rather than several"
    );
}
