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

use crate::refusal::{Refusal, cannot_judge, violation};

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

/// Read the figures a line writes in this census's phrasing, if it writes them at all.
///
/// Matching is literal between placeholders and a run of digits at each, tried from every offset — so
/// `73 bounds across 22 capabilities` is recognised wherever a sentence carries it, which is where a census
/// actually gets written. Anchoring at offset zero was the first draft and it matched nothing: a phrase
/// beginning with a placeholder has an empty leading literal, and the digits then had to be the line's first
/// characters.
fn figures_in(line: &str, phrase: &str) -> Option<Vec<usize>> {
    let parts: Vec<&str> = phrase.split("{}").collect();
    if parts.len() < 2 {
        return None;
    }
    let bytes = line.len();
    for start in 0..=bytes {
        if !line.is_char_boundary(start) {
            continue;
        }
        if let Some(found) = match_from(&line[start..], &parts) {
            return Some(found);
        }
    }
    None
}

/// One attempt, anchored at the front of `rest`.
fn match_from(rest: &str, parts: &[&str]) -> Option<Vec<usize>> {
    let mut rest = rest.strip_prefix(parts[0])?;
    let mut found = Vec::with_capacity(parts.len() - 1);
    for tail in &parts[1..] {
        let (value, consumed) = number_at(rest)?;
        rest = &rest[consumed..];
        found.push(value);
        if !tail.is_empty() {
            rest = rest.strip_prefix(*tail)?;
        }
    }
    Some(found)
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
fn number_at(rest: &str) -> Option<(usize, usize)> {
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    if !digits.is_empty() {
        return Some((digits.parse().ok()?, digits.len()));
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
                        return Some((tens + unit, prefix.len() + unit_word.len()));
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
    best
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
        let text = match std::fs::read_to_string(root.join(path)) {
            Ok(text) => text,
            Err(err) => {
                offences.push(cannot_judge(format!(
                    "  {path} is tracked and could not be read ({err}), so its censuses were never compared \
                     — an unread document is not a document without one"
                )));
                continue;
            }
        };
        for (index, line) in text.lines().enumerate() {
            for census in declared {
                let Some(written) = figures_in(line, census.phrase) else {
                    continue;
                };
                if written != census.figures {
                    offences.push(violation(format!(
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
    offences
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_phrase_reads_its_figures_wherever_the_sentence_carries_it() {
        let phrase = "{} bounds across {} capabilities";
        assert_eq!(
            figures_in(
                "the register currently holds **73 bounds across 22 capabilities** today",
                phrase
            ),
            Some(vec![73, 22])
        );
        assert_eq!(figures_in("no figures here", phrase), None);
        // Words, which is how this repository's prose actually writes a count.
        assert_eq!(
            figures_in(
                "seventy-three bounds across twenty-two capabilities",
                phrase
            ),
            Some(vec![73, 22])
        );
        assert_eq!(
            figures_in("nineteen bounds across nine capabilities", phrase),
            Some(vec![19, 9])
        );
        assert_eq!(
            figures_in("73 bounds across many capabilities", phrase),
            None
        );
    }
}
