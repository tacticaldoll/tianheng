//! A census is **declared**, produced by the check that enumerates the set, and swept for.
//!
//! `AGENTS.md` has said *a census is produced, never typed* since the 0.5.0 window, and one check has
//! enforced it — for exactly one sentence, `N bounds across M capabilities`. Every other figure about a set
//! this repository enumerates was outside anything, and adversarial review found **eight** of them wrong in a
//! single change: an entry population stated as eleven, then nineteen, then twenty, each corrected by the very
//! commit that broke the next one; a block header saying "four limits, three pinned" over seven and six; a
//! coupling ratio that drifted inside its own pull request; and a count of files that the commit typing it had
//! just made stale.
//!
//! The shape that ends the class is not a wider detector — a judgement over prose is the instrument this
//! repository designed, measured three times and rejected. It is a **declaration**: a check that enumerates
//! a set declares the one sentence its figures are written in, and one sweep holds every tracked document to
//! it. Adding a census means declaring it, which makes it enumerable; a figure written in an undeclared
//! sentence stays outside, and that residual is declared as a bound rather than approximated.

use std::path::Path;

use crate::refusal::{Refusal, cannot_judge_at, violation_at};

/// One census: a set some check enumerates, and the sentence its figures are written in.
pub struct Census {
    /// What is counted, for the diagnostic.
    pub subject: &'static str,
    /// The sentence, with `{}` where each figure goes. Everything outside the placeholders is matched
    /// literally, so a document may phrase the surrounding prose freely and still be held to the figures.
    pub phrase: &'static str,
    /// The figures, in the order the placeholders appear, **produced** by the enumerating check.
    pub figures: Vec<usize>,
}

/// Read every match this line carries of this census's phrasing, left to right.
///
/// Matching is literal between placeholders and a run of digits at each, tried from every offset — so
/// `73 bounds across 22 capabilities` is recognised wherever a sentence carries it, which is where a census
/// actually gets written. Anchoring at offset zero was the first draft and it matched nothing: a phrase
/// beginning with a placeholder has an empty leading literal, and the digits then had to be the line's first
/// characters.
///
/// **Every occurrence on the line is collected, not only the first.** Returning on the first match left a
/// line stating the correct figures first, followed later by a stale earlier draft's, silently unexamined —
/// measured directly: `"73 bounds across 22 capabilities, earlier drafts said 12 bounds across 5
/// capabilities"` against declared `[73, 22]` returned only the leading, correct match under the previous
/// first-match version, so the trailing stale figures produced no offence.
///
/// **A start offset landing inside an already-matched occurrence is skipped**, advancing past the whole
/// match instead of retrying one byte later. Trying every offset means a start inside `73` reads `3` as its
/// own number, and a start inside the compound word `seventy-three` reads its tail `three` as its own number
/// (value 3) — both then re-matching the same phrase's remaining tail again, reusing the second figure that
/// belonged to the first occurrence. Measured directly: without this, `"73 bounds across 22 capabilities"`
/// reported both `[73, 22]` and the spurious `[3, 22]`, and `"seventy-three bounds across twenty-two
/// capabilities"` reported both `[73, 22]` and the spurious `[3, 22]` from `three`. Neither is the residual
/// the header already discloses (a figure reflowed across a line break): each is a false occurrence sharing
/// text with one already read.
///
/// **This applies to every placeholder's figure, not only the first.** A phrase of two or more placeholders
/// with a short or empty literal after a middle or final one has the identical exposure one placeholder
/// later: `"{} of {}"` against `"3 of 53 of 9"` returned both `[3, 53]` and the spurious `[53, 9]` under a
/// version of this function that only skipped past the first placeholder's own token — the second figure's
/// digits, `53`, were still available to start a fresh match. Skipping to the end of the **whole** matched
/// span rather than only its first number's token closes both at once, because every figure the match
/// consumed — first, middle, or last — sits inside that span.
pub fn figures_in(line: &str, phrase: &str) -> Result<Vec<Vec<usize>>, String> {
    let parts: Vec<&str> = phrase.split("{}").collect();
    if parts.len() < 2 {
        return Ok(Vec::new());
    }
    let mut found = Vec::new();
    let mut start = 0usize;
    while start <= line.len() {
        if !line.is_char_boundary(start) {
            start += 1;
            continue;
        }
        match match_from(&line[start..], &parts)? {
            Some((figures, matched_len)) => {
                found.push(figures);
                start += matched_len.max(1);
            }
            None => start += 1,
        }
    }
    Ok(found)
}

/// One attempt, anchored at the front of `rest`. Returns the figures alongside how many bytes of `rest` the
/// whole match consumed, so [`figures_in`] can skip past it entirely rather than re-reading any of its
/// figures — first, middle, or last — as the start of a fresh, shorter match.
fn match_from(rest: &str, parts: &[&str]) -> Result<Option<(Vec<usize>, usize)>, String> {
    let Some(mut tail_rest) = rest.strip_prefix(parts[0]) else {
        return Ok(None);
    };
    let mut found = Vec::with_capacity(parts.len() - 1);
    for tail in &parts[1..] {
        let Some((value, consumed)) = number_at(tail_rest)? else {
            return Ok(None);
        };
        tail_rest = &tail_rest[consumed..];
        found.push(value);
        if !tail.is_empty() {
            let Some(next) = tail_rest.strip_prefix(*tail) else {
                return Ok(None);
            };
            tail_rest = next;
        }
    }
    Ok(Some((found, rest.len() - tail_rest.len())))
}

/// The count written at the front of `rest`, in digits **or in words**, and how many bytes it took.
///
/// Reading digits only was the first draft, and it left censuses **then declared** silent against the very
/// documents they are for. No count of them is written here: how many were declared at that moment is a figure
/// about a past state that nothing produces, and two attempts at it have already been wrong — the first said
/// four, and its repair implied the set had once held four and shrunk. Measured across the whole history of the
/// declaring test, the live set has never held more than three.
///
/// **The example a reader can still follow.** `{} of them in` was declared here and retired for matching an
/// unrelated sentence — the phrase-specificity assertion in [`sweep`] records that retirement — and it was one
/// of the phrasings that stayed silent. The document sentence it was for survives: `BACKLOG.md` and
/// `CHANGELOG.md` both write *twenty entries named that machinery*, a count spelled as a word, which a
/// digit-only matcher cannot see.
///
/// The need outlives every instance. This repository's prose writes counts as words, so a document stating a
/// declared census that way is invisible to a digit reader while being exactly the sentence it declares.
fn number_at(rest: &str) -> Result<Option<(usize, usize)>, String> {
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if !digits.is_empty() {
        // **A number this sweep cannot represent is not the absence of a number, and the old answer was
        // worse than absence.** `parse().ok()?` sent an overflowing run to the same answer as *there is
        // nothing numeric here* — and [`figures_in`] advances one byte and retries, so the next attempt saw
        // the same run one digit shorter, and eventually one that fits. Measured: a 26-digit run read as
        // `9999999999999999999`, a figure the document never wrote, and that was compared against the
        // declared census. Not silence — a fabricated figure. It refuses instead, and the refusal quotes the
        // run so the sentence can be found.
        return match digits.parse() {
            Ok(value) => Ok(Some((value, digits.len()))),
            Err(_) => Err(digits),
        };
    }
    const UNITS: [(&str, usize); 21] = [
        ("zero", 0),
        ("one", 1),
        ("two", 2),
        ("three", 3),
        ("four", 4),
        ("five", 5),
        ("six", 6),
        ("seven", 7),
        ("eight", 8),
        ("nine", 9),
        ("ten", 10),
        ("eleven", 11),
        ("twelve", 12),
        ("thirteen", 13),
        ("fourteen", 14),
        ("fifteen", 15),
        ("sixteen", 16),
        ("seventeen", 17),
        ("eighteen", 18),
        ("nineteen", 19),
        ("twenty", 20),
    ];
    const TENS: [(&str, usize); 8] = [
        ("twenty", 20),
        ("thirty", 30),
        ("forty", 40),
        ("fifty", 50),
        ("sixty", 60),
        ("seventy", 70),
        ("eighty", 80),
        ("ninety", 90),
    ];
    let lower = rest.to_ascii_lowercase();
    // A compound first — `twenty-two` must not read as `twenty`.
    for (tens_word, tens) in TENS {
        for separator in ["-", " "] {
            let prefix = format!("{tens_word}{separator}");
            if let Some(after) = lower.strip_prefix(&prefix) {
                for (unit_word, unit) in UNITS.iter().take(10).skip(1) {
                    if after.starts_with(unit_word) {
                        return Ok(Some((tens + unit, prefix.len() + unit_word.len())));
                    }
                }
            }
        }
    }
    // Longest word first, so `nineteen` is never read as `nine`.
    let mut best: Option<(usize, usize)> = None;
    for (word, value) in UNITS.iter().chain(TENS.iter()) {
        if lower.starts_with(word) && best.is_none_or(|(_, taken)| word.len() > taken) {
            best = Some((*value, word.len()));
        }
    }
    Ok(best)
}

/// Every tracked document stating a declared census with the wrong figures, and every one it could not read.
///
/// The two are different facts and the return type says so. A tracked document this sweep cannot read is not a
/// document with no census in it: skipping it silently would report clean over a corpus the sweep never
/// examined, which is the direction its sibling reference gate already refuses outright — *a file this check
/// claims to have inspected must have been read*.
pub fn sweep(root: &Path, tracked: &[String], declared: &[Census]) -> Vec<Refusal> {
    assert!(
        !declared.is_empty(),
        "no census was declared, so this sweep would report clean without comparing anything — the vacuity \
         direction"
    );
    for census in declared {
        // A phrase carrying a newline can never match, because the sweep reads a line at a time — it would be
        // declared, enumerable, and silent. One of the first four censuses written here was exactly that, and
        // it passed. A census that cannot match is worse than an undeclared one: it reads as covered.
        assert!(
            !census.phrase.contains('\n'),
            "the census for {} declares a phrase spanning lines, which this sweep can never match — it would \
             be silent while reading as declared",
            census.subject
        );
        // A phrase must be specific enough to name its own set. `{} of them in` was declared here and
        // matched an unrelated sentence in a specification — a census that fires on prose about something
        // else is a false positive the author then learns to ignore.
        let longest = census
            .phrase
            .split("{}")
            .map(str::len)
            .max()
            .unwrap_or_default();
        assert!(
            longest >= 12,
            "the census for {} declares a phrase whose longest literal is {longest} characters, which is not \
             enough to name the set it counts",
            census.subject
        );
        assert!(
            census.phrase.matches("{}").count() == census.figures.len(),
            "the census for {} declares {} placeholder(s) and {} figure(s)",
            census.subject,
            census.phrase.matches("{}").count(),
            census.figures.len()
        );
    }
    let mut offences = Vec::new();
    for path in tracked.iter().filter(|p| p.ends_with(".md")) {
        // **A record's figure is outside this sweep, the way it is outside the citation reader.** This read
        // every tracked `.md` with no exemption, so a figure inside a dated `CHANGELOG.md` section — a
        // measurement of its own moment, correct as written — was refused for disagreeing with today's
        // enumeration. One live instance escaped only because its two figures straddled a line break, which
        // means a reflow of that paragraph turned a green tree red over correct text: a false refusal, and one
        // edit away rather than latent. `record` owns the distinction; both readers ask it.
        if crate::record::is_record_document(path) {
            continue;
        }
        let text = match std::fs::read_to_string(root.join(path)) {
            Ok(text) => text,
            Err(err) => {
                offences.push(cannot_judge_at(
                    "repository-checks#census-document-unreadable",
                    format!(
                    "  {path} is tracked and could not be read ({err}), so its censuses were never compared \
                     — an unread document is not a document without one"
                )));
                continue;
            }
        };
        // A record's dated sections, so a figure inside one is skipped by line while the live sections of the
        // same document are read normally. `record` cuts them; this only asks.
        let records = crate::record::record_lines(path, &text);
        for (index, line) in text.lines().enumerate() {
            if records.contains(&(index + 1)) {
                continue;
            }
            for census in declared {
                let written_here = match figures_in(line, census.phrase) {
                    Ok(written) => written,
                    // A figure past `usize` is a sentence this sweep cannot compare, not a sentence with no
                    // census in it. Naming the run is what lets a reader find it.
                    Err(digits) => {
                        offences.push(cannot_judge_at(
                            "repository-checks#census-figure-unreadable",
                            format!(
                                "  {path}:{} writes `{digits}` where {} declares a count, and this sweep \
                                 cannot represent it — an unreadable figure is not an absent one",
                                index + 1,
                                census.subject
                            ),
                        ));
                        continue;
                    }
                };
                for written in written_here {
                    if written != census.figures {
                        offences.push(violation_at(
                            "repository-checks#census-figure-disagrees",
                            format!(
                            "  {path}:{} writes {written:?} for {} where the check that enumerates it \
                             produces {:?} — a census is produced, never typed",
                            index + 1,
                            census.subject,
                            census.figures
                        )));
                    }
                }
            }
        }
    }
    offences
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every figure the phrase reads on this line, where the line carries no number this sweep cannot hold.
    ///
    /// The refusing path has its own direction below; this keeps the reading rows about reading.
    fn read(line: &str, phrase: &str) -> Vec<Vec<usize>> {
        figures_in(line, phrase).expect("this line writes no figure past `usize`")
    }

    #[test]
    fn a_phrase_reads_its_figures_wherever_the_sentence_carries_it() {
        let phrase = "{} bounds across {} capabilities";
        assert_eq!(
            read(
                "the register currently holds **73 bounds across 22 capabilities** today",
                phrase
            ),
            vec![vec![73, 22]]
        );
        assert_eq!(read("no figures here", phrase), Vec::<Vec<usize>>::new());
        // Words, which is how this repository's prose actually writes a count.
        assert_eq!(
            read(
                "seventy-three bounds across twenty-two capabilities",
                phrase
            ),
            vec![vec![73, 22]]
        );
        assert_eq!(
            read("nineteen bounds across nine capabilities", phrase),
            vec![vec![19, 9]]
        );
        assert_eq!(
            read("73 bounds across many capabilities", phrase),
            Vec::<Vec<usize>>::new()
        );
    }

    /// A figure this sweep cannot represent refuses, where it used to read as no figure at all.
    ///
    /// **`parse().ok()?` spelled *unreadable* the same as *absent*.** A document writing a count past `usize`
    /// was compared against nothing and the sweep reported clean over the sentence it exists for — the
    /// conflation `reading`'s module doc names as the one bug this repository forbids, in the module whose
    /// whole subject is a declared figure disagreeing with a produced one.
    #[test]
    fn a_figure_this_sweep_cannot_represent_refuses_rather_than_reading_as_absent() {
        let phrase = "{} bounds across {} capabilities";
        let refused = figures_in(
            "99999999999999999999999999 bounds across 22 capabilities",
            phrase,
        )
        .expect_err("a run of digits past `usize` is not the absence of a number");
        assert_eq!(
            refused, "99999999999999999999999999",
            "the refusal quotes the run, so the sentence can be found"
        );
        // Measured with the old answer restored: this same line read `[[9999999999999999999, 22]]` — the
        // sweep retried one byte along, the run fitted at nineteen digits, and a figure the document never
        // wrote was compared against the census. Truncation, not silence.
        // The same sentence with a figure this sweep can hold reads, so the refusal is about the width and
        // not about the phrase.
        assert_eq!(read("73 bounds across 22 capabilities", phrase), [[73, 22]]);
        // And the second figure's position refuses too — the reader does not stop caring after the first.
        assert!(
            figures_in(
                "73 bounds across 99999999999999999999999999 capabilities",
                phrase
            )
            .is_err(),
            "a figure past `usize` anywhere in the phrase is one this sweep cannot compare"
        );
    }

    /// The direction this repository's own review measured missing: a line stating the correct figures
    /// first and a stale earlier draft's later must surface **both** matches, not only the leading one.
    #[test]
    fn every_occurrence_on_one_line_is_read_not_only_the_first() {
        let phrase = "{} bounds across {} capabilities";
        assert_eq!(
            read(
                "73 bounds across 22 capabilities, earlier drafts said 12 bounds across 5 capabilities",
                phrase
            ),
            vec![vec![73, 22], vec![12, 5]]
        );
        // The reverse order: the stale figure first, the correct one trailing — already caught before this
        // fix, kept as the control.
        assert_eq!(
            read(
                "earlier drafts said 12 bounds across 5 capabilities, corrected to 73 bounds across 22 \
                 capabilities",
                phrase
            ),
            vec![vec![12, 5], vec![73, 22]]
        );
    }

    /// A short or empty literal after a non-first placeholder must not let that figure's own digits start a
    /// second, spurious match — the same overlap [`every_occurrence_on_one_line_is_read_not_only_the_first`]
    /// closed for the first placeholder, one placeholder later. `"{} bounds across {} capabilities"` has a
    /// long literal after its second placeholder, so this never showed there; a phrase like `"{} of {}"`,
    /// whose final placeholder has no literal after it at all, has the identical exposure.
    #[test]
    fn a_later_placeholder_s_figure_does_not_start_a_second_spurious_match() {
        let phrase = "{} of {}";
        assert_eq!(read("3 of 53 of 9", phrase), vec![vec![3, 53]]);
    }
}
