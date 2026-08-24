use crate::reading::{Sep, fields};
use crate::refusal::Kind;

/// Both ways [`fields`] refuses, and the one way it answers.
///
/// Each refusal is a **cannot-judge**: a field count this reader did not expect is a fact about the input,
/// not a subject that disagrees with what it is judged against.
#[test]
fn a_field_count_this_reader_did_not_expect_is_refused_either_way() {
    let few = fields::<2>("support window", "24", Sep::Whitespace).expect_err("one is not two");
    assert_eq!(few.kind, Kind::CannotJudge, "{}", few.message);
    crate::refusal::expect("repository-checks#fields-miscounted", &few);
    assert!(
        few.message.contains("1 fields"),
        "the refusal must say how many arrived, since none and one are different facts: {}",
        few.message
    );

    let many = fields::<2>("support window", "24 2028-04-30 extra", Sep::Whitespace)
        .expect_err("three is not two");
    crate::refusal::expect("repository-checks#fields-miscounted", &many);
    assert!(
        many.message.contains("3 fields"),
        "the refusal must say how many arrived: {}",
        many.message
    );

    assert_eq!(
        fields::<2>("support window", "24 2028-04-30", Sep::Whitespace).map_err(|r| r.message),
        Ok(["24", "2028-04-30"])
    );
}

/// The separators differ by what an **empty** field means, which is the reason there are two.
///
/// The `Sep::Char` direction is the one that matters: a repeated delimiter is a defect, and a reader that
/// collapses it reads `2028--4-30` as a well-formed date. Give it the candidate a collapsing reader would
/// have dropped, and assert the dropped one is what refuses.
#[test]
fn a_character_separator_keeps_the_empty_field_a_collapsing_reader_would_drop() {
    assert_eq!(
        fields::<2>("support window", "24   2028-04-30", Sep::Whitespace).map_err(|r| r.message),
        Ok(["24", "2028-04-30"]),
        "whitespace collapses runs, so a freely spaced declaration still reads as two fields"
    );

    let doubled = fields::<3>("date", "2028--4-30", Sep::Char('-'))
        .expect_err("a repeated delimiter divides into four fields, not three");
    crate::refusal::expect("repository-checks#fields-miscounted", &doubled);
    assert!(
        doubled.message.contains("4 fields"),
        "the refusal must name the field the collapsing reader dropped: {}",
        doubled.message
    );

    assert_eq!(
        fields::<3>("date", "2028-04-30", Sep::Char('-')).map_err(|r| r.message),
        Ok(["2028", "04", "30"])
    );
}
