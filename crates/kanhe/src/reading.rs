//! Refusing input a reader cannot understand, where the habit is to skip it.
//!
//! Sibling of [`crate::selection`], deliberately apart. That one answers *how many candidates are there*;
//! this one answers *could this be read at all*. Merging them would produce one instrument for two
//! mechanisms, which is the shape this repository removes on sight.
//!
//! **The bug is never "read it wrong". It is that not-readable was spelled the same as not-present.**
//! `filter_map(|part| part.parse().ok())` drops what it cannot parse and hands the survivors on, so a
//! destructure of three succeeds over an input that carried four — measured: `2028--4-30` read as
//! `2028-04-30`. `machinery_names` `continue`d on a failed prefix strip and enumerated 0 of 8 members.
//!
//! **This module binds only the call sites that use it.** Nothing enumerates the readers that should —
//! see `BACKLOG.md`'s entry on a reader's corpus being narrower than its claim, which owns that residue.

use crate::refusal::{Refusal, cannot_judge_at};

/// How a text is divided into fields.
///
/// The two do not differ by convenience, they differ by what an **empty** field means. Collapsing runs is
/// right for a declaration a human spaces freely; it is wrong for a delimiter whose repetition is a defect,
/// and reading `2028--4-30` as three fields is exactly that defect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sep {
    /// Runs of whitespace, collapsed — so `24   2028` is two fields, not four.
    Whitespace,
    /// One character, **not** collapsed — so `2028--4-30` is four fields, not three.
    Char(char),
}

impl Sep {
    /// Every field, in order, without dropping an empty one.
    fn divide(self, text: &str) -> Vec<&str> {
        match self {
            Sep::Whitespace => text.split_whitespace().collect(),
            Sep::Char(separator) => text.split(separator).collect(),
        }
    }
}

/// Exactly `N` fields, or a refusal naming how many were found.
///
/// A **cannot-judge**: a field count the reader did not expect is a fact about the input, not a subject
/// disagreeing with what it is judged against.
///
/// **The count is the whole point.** `split(sep).filter_map(…)` answers *fewer* by dropping, and the
/// survivors then destructure as if nothing was lost — so a reader claiming to have read three fields
/// reports a verdict over an input that carried four. Asking for `N` and being told what arrived makes the
/// two states different again.
///
/// `what` names the thing being read, so the refusal says which reader met the input rather than only what
/// the input was. What to *write* instead belongs to the caller, which knows the form it wanted.
pub fn fields<'a, const N: usize>(
    what: &str,
    text: &'a str,
    sep: Sep,
) -> Result<[&'a str; N], Refusal> {
    let found = sep.divide(text);
    let count = found.len();
    found.try_into().map_err(|_| {
        cannot_judge_at(
            "repository-checks#fields-miscounted",
            format!(
                "the {what} reads `{text}`, which divides into {count} fields where this reader expects \
                 {N}; taking the ones it recognised would report a verdict over an input it did not read"
            ),
        )
    })
}

/// A civil date, and its distance from the epoch.
///
/// **The fields are private so [`date`] is the only way in.** A struct literal would build a `Civil` that
/// no calendar has — `2028-02-31` — and then answer `days_from_epoch` for it, which is the defect this type
/// exists to make unconstructible rather than to catch. Same argument as [`Refusal`]'s own private `site`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Civil {
    year: i64,
    month: i64,
    day: i64,
}

impl Civil {
    /// Days from 1970-01-01, proleptic Gregorian, by Howard Hinnant's civil-calendar algorithm.
    ///
    /// Arithmetic rather than a dependency: this crate takes `serde_json` for cargo's message stream and
    /// nothing else, and a date library would be one added for a comparison. The algorithm is closed-form,
    /// and the values that make a careless transcription wrong — a leap day, a century that is not a leap
    /// year, the epoch itself — are asserted beside it.
    pub const fn days_from_epoch(self) -> i64 {
        let year = if self.month <= 2 {
            self.year - 1
        } else {
            self.year
        };
        let era = if year >= 0 { year } else { year - 399 } / 400;
        let year_of_era = year - era * 400;
        let shifted_month = (self.month + 9) % 12;
        let day_of_year = (153 * shifted_month + 2) / 5 + self.day - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        era * 146_097 + day_of_era - 719_468
    }
}

/// How many days a month has, leap years included.
const fn days_in_month(year: i64, month: i64) -> i64 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        _ => {
            if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) {
                29
            } else {
                28
            }
        }
    }
}

/// Whether every character is an ASCII digit, and there are exactly `width` of them.
fn digits(text: &str, width: usize) -> bool {
    text.len() == width && text.bytes().all(|byte| byte.is_ascii_digit())
}

/// The value of a run of ASCII digits, which [`digits`] establishes before this is called.
///
/// **No `Result`, because there is no failure.** Two or four ASCII digits cannot overflow an `i64`, and a
/// non-digit cannot reach here.
fn digit_value(text: &str) -> i64 {
    text.bytes()
        .fold(0i64, |value, byte| value * 10 + i64::from(byte - b'0'))
}

/// A `YYYY-MM-DD` date the calendar actually has, or a refusal saying which way it was not one.
///
/// **Two mechanisms made a date wrong here, and one repair closes both.** A component that could not be
/// parsed was *dropped* — `filter_map(|part| part.parse().ok())` over `2028--4-30` yielded three values from
/// four fields, so a destructure of three succeeded and the date read as `2028-04-30`. And a component in
/// range was not checked against the calendar — `1..=12` with `1..=31` admits `2028-02-31`, which
/// [`Civil::days_from_epoch`] then answers for as the following March. A reader whose refusal said *names no
/// day* did neither.
///
/// The field count is [`fields`]'s, with `Sep::Char('-')` **not** collapsing, so a repeated delimiter is the
/// extra field it is rather than an absence. The width is checked because `YYYY-MM-DD` is the declared form
/// and `2028-4-30` is not it: admitting it would make the reader accept two spellings of one date while its
/// own message names one.
///
/// Every refusal is a **cannot-judge**: a date this reader cannot read is a fact about the input, not a
/// subject disagreeing with what it is judged against.
pub fn date(what: &str, text: &str) -> Result<Civil, Refusal> {
    let [year, month, day] = fields::<3>(what, text, Sep::Char('-'))?;
    if !digits(year, 4) || !digits(month, 2) || !digits(day, 2) {
        return Err(cannot_judge_at(
            "repository-checks#date-not-the-declared-shape",
            format!(
                "the {what} reads `{text}`, and this reads `YYYY-MM-DD` — four digits, two, then two, so \
                 one date has one spelling here"
            ),
        ));
    }
    // **Read rather than parsed, so there is no failure arm to answer for.** `parse::<i64>()` hands back a
    // `Result` these three cannot take: the widths above have already established each is a run of two or
    // four ASCII digits. An arm for it would be a fail-loud over an impossible state, which the minimalism
    // bound forbids — and worse, the refusal register would then hold a registered site no direction can
    // reach, which is a declared gap where there is no gap.
    let (year, month, day) = (digit_value(year), digit_value(month), digit_value(day));
    // The month is answered before the day, because `days_in_month` is only defined against a real month —
    // asked about a thirteenth it falls to its February arm and the refusal would say a thirteenth month has
    // 28 days, which is a sentence about nothing.
    if !(1..=12).contains(&month) {
        return Err(cannot_judge_at(
            "repository-checks#date-names-no-month",
            format!("the {what} reads `{text}`, and the calendar has no month {month}"),
        ));
    }
    if day < 1 || day > days_in_month(year, month) {
        return Err(cannot_judge_at(
            "repository-checks#date-names-no-day",
            format!(
                "the {what} reads `{text}`, and the calendar has no such day — {year}-{month:02} has {} \
                 days. A date past its month's end is not a later date, it is no date",
                days_in_month(year, month)
            ),
        ));
    }
    Ok(Civil { year, month, day })
}
