use crate::reading::{Sep, date, fields};
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

/// The date arithmetic, on the values that make a careless transcription wrong.
///
/// Moved here with the arithmetic it tests. It sat in `interpreter_support_window` and caught a real
/// off-by-one there — the `2100-03-01` row, a century that is **not** a leap year, which is exactly where a
/// careless transcription goes wrong.
#[test]
fn the_calendar_arithmetic_holds_at_its_awkward_days() {
    for (text, expected) in [
        ("1970-01-01", 0),
        ("1970-01-02", 1),
        ("1972-02-29", 789),
        ("2000-02-29", 11016),
        ("2100-03-01", 47541),
        ("2028-04-30", 21304),
    ] {
        let civil = date("date", text).expect("a real day");
        assert_eq!(
            civil.days_from_epoch(),
            expected,
            "{text} is not {expected} days from the epoch"
        );
    }
}

/// Every shape [`date`] refuses, and the site each carries.
///
/// **The two mechanisms are given separate rows because they are separate mechanisms.** A dropped component
/// and an unchecked calendar both produced a wrong date here, and a repair closing one would have read as
/// closing both.
#[test]
fn a_date_this_reader_cannot_read_is_refused_and_says_which_way() {
    // Mechanism one: a component a `filter_map` would have dropped. The empty field between the doubled
    // delimiter is the candidate, and the field count is what refuses.
    let dropped = date("support window's date", "2028--4-30").expect_err("four fields, not three");
    assert_eq!(dropped.kind, Kind::CannotJudge, "{}", dropped.message);
    crate::refusal::expect("repository-checks#fields-miscounted", &dropped);

    // Mechanism two: every component parses and is four-two-two digits, and the calendar still has no such
    // day. `2028-02-31` is the measured instance — it was accepted, and read as the following March.
    for text in ["2028-02-31", "2028-04-31", "2027-02-29", "2028-04-00"] {
        let no_day = date("support window's date", text).expect_err(
            "the calendar has no such day, so this refuses rather than reading a later one",
        );
        assert_eq!(no_day.kind, Kind::CannotJudge, "{}", no_day.message);
        crate::refusal::expect("repository-checks#date-names-no-day", &no_day);
    }

    // A month the calendar does not have takes its OWN site, because one identity names one branch: a
    // direction citing a shared one would vouch for a branch it never reached.
    for text in ["2028-13-01", "2028-00-10"] {
        let no_month =
            date("support window's date", text).expect_err("the calendar has no such month");
        crate::refusal::expect("repository-checks#date-names-no-month", &no_month);
    }

    // The leap rule in all three directions, so the day check is not simply `31`.
    assert!(
        date("date", "2028-02-29").is_ok(),
        "2028 is divisible by 4, so the 29th is a day"
    );
    assert!(
        date("date", "2100-02-29").is_err(),
        "2100 is divisible by 100 and not by 400, so it is not a leap year"
    );
    assert!(
        date("date", "2000-02-29").is_ok(),
        "2000 is divisible by 400, so it is"
    );
}

/// The declared spelling is one spelling, and the widths are what make it one.
#[test]
fn a_date_outside_the_declared_spelling_is_refused() {
    // Three fields, wrong widths — the declared spelling is what refuses.
    for text in ["2028-4-30", "2028-04-3", "28-04-30", "2028-0a-30"] {
        let refusal = date("support window's date", text)
            .expect_err("only `YYYY-MM-DD` is the declared spelling");
        assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
        crate::refusal::expect("repository-checks#date-not-the-declared-shape", &refusal);
    }

    // Not three fields at all — the count refuses first, and that is the right site rather than a spelling
    // complaint about an input that never divided.
    for text in ["2028-04-30-01", "April 2028"] {
        let refusal = date("support window's date", text).expect_err("not three fields");
        crate::refusal::expect("repository-checks#fields-miscounted", &refusal);
    }
    assert!(date("support window's date", "2028-04-30").is_ok());
}
