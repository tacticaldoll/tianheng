use crate::refusal::Kind;
use crate::selection::{all_of, the_only};

/// Both ways [`the_only`] refuses, and the one way it answers.
///
/// Each refusal is a **cannot-judge**: none and several are facts about an input this reader could not reduce
/// to one, which is not the same fact as a subject that disagrees with what it is judged against.
#[test]
fn the_only_refuses_none_and_several_and_says_which() {
    let none = the_only("widget", Vec::<u8>::new()).expect_err("none is not one");
    assert_eq!(none.kind, Kind::CannotJudge, "{}", none.message);
    crate::refusal::expect("repository-checks#the-only-found-none", &none);
    assert!(
        none.message.contains("found none"),
        "the refusal must say the count was zero: {}",
        none.message
    );

    let several = the_only("widget", vec![1u8, 2, 3]).expect_err("several is not one");
    assert_eq!(several.kind, Kind::CannotJudge, "{}", several.message);
    crate::refusal::expect("repository-checks#the-only-found-several", &several);
    assert!(
        several.message.contains("found 3"),
        "the refusal must say how many, since two and twenty are different facts: {}",
        several.message
    );

    assert_eq!(the_only("widget", vec![7u8]).map_err(|r| r.message), Ok(7));
}

/// [`all_of`] keeps every candidate, including the ones a first-only reader would drop.
///
/// Duplicates are kept too. Deduplication is the caller's decision and folding it in here would make the
/// count this module exists to preserve depend on values rather than on how many were found.
#[test]
fn all_of_keeps_every_candidate_including_repeats() {
    assert_eq!(all_of(vec![1u8, 2, 3]), vec![1, 2, 3]);
    assert_eq!(all_of(vec![4u8, 4]), vec![4, 4]);
    assert_eq!(all_of(Vec::<u8>::new()), Vec::<u8>::new());
}
