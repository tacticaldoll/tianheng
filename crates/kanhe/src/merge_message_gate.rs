//! The squash-message judgement, shared by the gate and by its failure matrix.
//!
//! `AGENTS.md` states these rules and nothing held them: nine subjects, counted when this gate was written, carry a
//! trailing `(#N)`, the most recent on the commit that landed a check for a requirement enforced by
//! nothing. The rule cannot be held where rules are usually held here — a squash merge runs on GitHub's
//! servers, so no local commit exists and no `commit-msg` hook runs, and both values of
//! `squash_merge_commit_title` append the serial. What remains is one string passed at merge time.
//!
//! So this stands in front of `gh pr merge` the way the publish-source gate stands in front of
//! `cargo publish`: a rule that was written, then missed, at the one moment nothing can be undone. A merged
//! squash on a release branch is a record, and amending it would decouple it from the pull request whose
//! merge record cites its hash.
//!
//! The verdict is the shared kinded [`Refusal`], so *the message disagrees* stays separate from *the title
//! could not be read*. The focused failure matrix asserts both observable outcomes and their actionable text.

use crate::refusal::{Refusal, cannot_judge_at, violation_at};

/// The Conventional Commit types `AGENTS.md` admits.
const TYPES: [&str; 9] = [
    "feat", "fix", "refactor", "docs", "test", "build", "ci", "perf", "chore",
];

/// The tool-authorship marks `AGENTS.md` names, matched **case-insensitively** and **by position**.
///
/// Three forms, and the rule they serve is wider than three: `AGENTS.md` forbids these "or any other
/// tool-authorship mark". That clause is not enumerable, so this check holds the named forms and the open one
/// stays a reviewer's obligation — stated here rather than implied by a list that looks complete.
///
/// **Case-insensitively, because the canonical spelling is not the one this listed.** Git writes the trailer
/// `Co-authored-by:` and GitHub renders it that way; the exact-case `contains` this replaced let
/// `co-authored-by: Claude` and `generated with Claude Code` straight through — measured, both were accepted.
///
/// **By position where the mark is a trailer, because a body that DISCUSSES one does not carry it.** A trailer is
/// a line of its own beginning with the key; prose naming it is inline, and GitHub honours only the line form.
/// Matching a bare substring refuses the commit message of any change about this rule — including the one that
/// widened it — which is the false refusal `repository-checks` already forbids this gate: refuse a shape for what
/// it is, not for what it resembles. Widening the case and narrowing to the line are one change, because either
/// alone trades one defect for the other.
///
/// **And not by position where it is a glyph.** Reading all three by position was the first draft and it would
/// have opened a false negative: `fix(x): 🤖 wrote this` was refused by the substring and would have passed,
/// because the glyph sits mid-line. [`Shape`] carries that difference beside each mark.
///
/// The subject arm matters only for the glyph. A subject that begins with a trailer key is already refused for not
/// being a Conventional Commit, one check earlier — measured, which is why no direction here asserts otherwise.
const ATTRIBUTION: [(&str, Shape); 3] = [
    ("co-authored-by", Shape::TrailerKey),
    ("generated with", Shape::Footer),
    ("🤖", Shape::Glyph),
];

/// How a mark is recognized. **Not every mark is the same kind of thing.**
///
/// A first draft read all three by position and would have introduced a false negative: `fix(x): 🤖 wrote this`
/// was refused by the substring it replaced and would have passed, because the glyph sits mid-line. Each kind
/// needs its own recognition, so it travels in the same array as the mark rather than in a second
/// rule beside it.
#[derive(Clone, Copy)]
enum Shape {
    /// A trailer KEY on a line of its own, so the mark must be followed by its `:`. Prose naming it is not
    /// it, and GitHub honours only the line form.
    ///
    /// **The colon is what bounds it, and without one the prefix ran on.** `starts_with("co-authored-by")`
    /// refuses a line beginning `Co-authored-bystander …` — a false refusal of a message carrying no
    /// attribution at all, in a gate whose own requirement exists to prevent exactly that: the line-start
    /// rule is there so *a body that names one inside a sentence is not carrying it*.
    TrailerKey,
    /// A footer PHRASE on a line of its own, bounded by a word boundary rather than by a colon.
    ///
    /// **Not every mark this array holds is a `Key: Value`, and treating them alike is what let the prefix
    /// run on.** `Generated with Claude Code` carries no colon, so demanding one would stop refusing the
    /// real mark; requiring the phrase to END — the next character is not one a word continues with —
    /// refuses that and admits `Generated withheld …`, which is not this mark.
    ///
    /// A review reported this as the gate failing to establish a `Key: Value` shape. It is not: the
    /// requirement asks for case-insensitive recognition at the start of a line, and this half of the array
    /// is a footer rather than a key — the spec's own word for it. What was wrong is the boundary, not the
    /// shape.
    Footer,
    /// A glyph with no legitimate use in this repository's commit messages, wherever it appears. Prose about the
    /// rule names it in words instead — which is what this repository's own prose does.
    Glyph,
}

impl Shape {
    /// The rule this shape refuses by, so a refusal states the recognition it was actually made under.
    ///
    /// **One sentence served both shapes and stated only one.** The refusal read *naming the mark inside a
    /// sentence is not carrying it; a line that begins with it is* — the trailer rule — while
    /// [`Shape::Glyph`] matches wherever the glyph sits, mid-line included, which [`Shape::Glyph`]'s own doc
    /// states. So an operator refused for `fix(x): 🤖 wrote this` was handed the rule that would
    /// have permitted it, in front of a record no rerun amends. The mark and its recognizer already travel
    /// in one array; the sentence that recognizer refuses by belongs beside them rather than in a single
    /// `format!` that cannot know which fired.
    const fn rule(self) -> &'static str {
        match self {
            Shape::TrailerKey => {
                "a line that begins with it and its `:` is carrying it; naming it inside a sentence is not, \
                 and a longer word that merely starts the same way is not it at all"
            }
            Shape::Footer => {
                "a line that begins with it, as a phrase that ends there, is carrying it; naming it inside a \
                 sentence is not, and a longer word that merely starts the same way is not it at all"
            }
            Shape::Glyph => {
                "this glyph has no legitimate use in a commit message here, wherever it appears — name the \
                 rule in words instead, as this repository's own prose does"
            }
        }
    }
}

/// Whether `text` carries `mark` in the way its shape defines.
///
/// **A prefix is not the mark, and this took `starts_with` alone until a review read it.** Both line shapes
/// now require the mark to END where it ends: a trailer key at its `:`, a footer phrase at a word boundary.
/// Without that, `Co-authored-bystander …` at a line start was refused — a false refusal, which is the
/// direction this requirement's own reason forbids rather than the false negative the gate exists for.
fn carries(text: &str, mark: &str, shape: Shape) -> bool {
    match shape {
        Shape::TrailerKey => text.lines().any(|line| {
            line.trim()
                .to_ascii_lowercase()
                .strip_prefix(mark)
                // Optional space before the colon, because git's own trailer reader accepts one.
                .is_some_and(|rest| rest.trim_start().starts_with(':'))
        }),
        Shape::Footer => text.lines().any(|line| {
            line.trim()
                .to_ascii_lowercase()
                .strip_prefix(mark)
                // The phrase ends here: end of line, or a character no word continues with. `-` and `_`
                // count as continuing, so a hyphenated compound is not this phrase either.
                .is_some_and(|rest| {
                    rest.chars()
                        .next()
                        .is_none_or(|c| !(c.is_alphanumeric() || c == '-' || c == '_'))
                })
        }),
        Shape::Glyph => text.contains(mark),
    }
}

/// Whether a subject ends in a pull request serial, in the form GitHub appends.
fn carries_a_serial(subject: &str) -> bool {
    let Some(open) = subject.rfind("(#") else {
        return false;
    };
    let inside = &subject[open + 2..];
    inside
        .strip_suffix(')')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// Whether a subject is `<type>(<scope>)!?: <summary>` with a lowercase admitted type.
fn is_conventional(subject: &str) -> bool {
    let Some((head, summary)) = subject.split_once(": ") else {
        return false;
    };
    if summary.trim().is_empty() {
        return false;
    }
    let head = head.strip_suffix('!').unwrap_or(head);
    let name = match head.split_once('(') {
        Some((name, rest)) => {
            let Some(scope) = rest.strip_suffix(')') else {
                return false;
            };
            if scope.is_empty() || scope != scope.to_ascii_lowercase() {
                return false;
            }
            name
        }
        None => head,
    };
    TYPES.contains(&name)
}

/// Whether the body is GitHub's concatenated commit list — every bullet one of **these** commits.
///
/// Recognised by what the bullets say, not by their shape. Refusing every all-bullet body refused a terse
/// self-contained one for its formatting, and tightening the shape instead — requiring a bullet to look like
/// a Conventional Commit — would refuse a hand-written `- fix: …` body while a branch carrying one
/// non-conventional subject slipped through. The exact question is *are these the commits*, and the wrapper
/// can answer it.
fn is_a_bare_commit_list(body: &str, commits: &[String]) -> bool {
    let mut saw_one = false;
    for line in body.lines().filter(|line| !line.trim().is_empty()) {
        let trimmed = line.trim_start();
        let Some(text) = trimmed
            .strip_prefix("* ")
            .or_else(|| trimmed.strip_prefix("- "))
        else {
            return false;
        };
        if !commits.iter().any(|subject| subject == text.trim()) {
            return false;
        }
        saw_one = true;
    }
    saw_one
}

/// Whether `subject` is the release snapshot's, the one subject whose body the ritual requires to be empty.
///
/// Exactly `release: X.Y.Z` with a well-formed version, because the exception is for that act and not for a
/// subject that merely begins with the word: the release-history reader in `release_coherence_gate` refuses a
/// malformed `release:` subject for the same reason, and the two would disagree about the same line otherwise.
/// The one message exception, identified by **where the squash lands** as well as by what it says.
///
/// `AGENTS.md` states it as the *release-branch-to-`main`* squash, and a subject is not a destination: a
/// message reading `release: 0.0.0` on any other base is an ordinary squash claiming the exception's shape.
/// Deciding on the subject alone made the exception's identity a spelling, which is what this repository
/// refuses everywhere it recognises something by a bare marker rather than by its position.
fn is_release_snapshot(subject: &str, base: &str) -> bool {
    base.trim() == "main"
        && subject
            .strip_prefix("release: ")
            .is_some_and(|rest| crate::manifest::semver(rest).is_some())
}

/// Judge a proposed squash message against the pull request it would record.
///
/// Ordered most-specific first. A subject carrying a serial also differs from its title and is also still
/// conventional-shaped; reporting the general fact for the specific one sends a reader to compare two strings
/// that differ by exactly the thing the rule already names.
pub fn judge(
    subject: &str,
    body: &str,
    title: &str,
    commits: &[String],
    base: &str,
) -> Result<String, Refusal> {
    if title.trim().is_empty() {
        return Err(cannot_judge_at(
            "repository-checks#squash-title-unavailable",
            "the pull request's title is unavailable, so whether the subject is that title cannot be \
             decided — which is not the same fact as a subject that disagrees",
        ));
    }
    if carries_a_serial(subject) {
        return Err(violation_at(
            "repository-checks#squash-subject-carries-a-serial",
            format!(
                "the squash subject ends in a pull request serial: {subject:?}. GitHub appends `(#N)` to the \
             default subject and neither value of the repository's squash-title setting suppresses it, so \
             the subject is passed explicitly — without the serial"
            ),
        ));
    }
    if subject != title {
        return Err(violation_at(
            "repository-checks#squash-subject-is-not-the-title",
            format!(
                "the squash subject is not the pull request's title.\n  subject: {subject:?}\n  title:   \
             {title:?}\nThe title is what review saw; a subject saying something else makes the record \
             disagree with what was approved"
            ),
        ));
    }
    // The same sole exception: `release: X.Y.Z` is not a Conventional Commit subject and `AGENTS.md` says so
    // — *the release-branch-to-`main` squash is the sole message exception*. This rule refused it too, so the
    // gate encoded every rule of the ritual except the one the ritual itself names.
    if !is_conventional(subject) && !is_release_snapshot(subject, base) {
        return Err(violation_at(
            "repository-checks#squash-subject-is-not-conventional",
            format!(
                "the squash subject is not a Conventional Commit: {subject:?}. Expected \
             `<type>(<scope>)!?: <summary>` with a lowercase type from {TYPES:?}"
            ),
        ));
    }
    // The head, not the whole subject: a summary may carry an exclamation mark for its own reasons, and the
    // shape check above already reads the head to strip a trailing `!` before matching the type.
    let head_is_breaking = subject
        .split_once(": ")
        .is_some_and(|(head, _)| head.ends_with('!'));
    if head_is_breaking && !body.contains("BREAKING CHANGE:") {
        return Err(violation_at(
            "repository-checks#squash-breaking-without-a-migration-footer",
            "the squash subject is marked breaking and the body names no `BREAKING CHANGE:` footer, so the \
             record announces a migration it does not describe",
        ));
    }
    for (mark, shape) in ATTRIBUTION {
        if carries(subject, mark, shape) || carries(body, mark, shape) {
            return Err(violation_at(
                "repository-checks#squash-message-carries-an-attribution",
                format!(
                    "the squash message carries the agent attribution {mark:?}, which this repository's commit \
                 messages and pull request descriptions do not carry. {}",
                    shape.rule()
                ),
            ));
        }
    }
    // **The release snapshot is the one subject whose body is required to be empty, and this rule refused
    // it.** `AGENTS.md` states the exception in its own words — the release-branch-to-`main` squash's subject
    // is `release: X.Y.Z` and *its body is deliberately empty* — so the wrapper could not perform the one
    // merge the ritual cares about most, and `release: 0.5.0` was merged around it. Measured: `0.4.0`'s
    // release commit carries an empty body too, so this has been true of every release.
    //
    // The exception is what lets that merge go *through* the wrapper, which is strictly more observation
    // than it had: the subject shape, the attribution marks and the title match are all still judged. An
    // empty body stays a violation for every other subject.
    if body.trim().is_empty() && !is_release_snapshot(subject, base) {
        return Err(violation_at(
            "repository-checks#squash-body-is-empty",
            "the squash body is empty; a commit body carries why the change exists and what contract it \
             preserves, and the branch's fine-grained commits are review provenance rather than this record",
        ));
    }
    if commits.is_empty() {
        return Err(cannot_judge_at(
            "repository-checks#squash-commits-unavailable",
            "the pull request's commit subjects are unavailable, so whether this body is the default \
             concatenation of them cannot be decided — falling back to refusing every bulleted body is the \
             false refusal this reads them to avoid",
        ));
    }
    if is_a_bare_commit_list(body, commits) {
        return Err(violation_at(
            "repository-checks#squash-body-is-a-bare-commit-list",
            "the squash body is a bare list of commit subjects, which is the default this rule exists to \
             replace with something self-contained",
        ));
    }
    Ok(format!("ok merge message ({subject})"))
}

/// The Conventional Commit types `AGENTS.md` admits, read from the contract rather than copied from it.
///
/// The gate carries its own list, and the contract states the same set in prose. Two lists that must agree
/// is the shape this repository closes wherever an enumerator exists — and here one does: the sentence naming
/// the narrowest honest type carries every admitted type as a backticked run.
///
/// **Refuses loudly when the anchor is gone.** A parse that silently found nothing would make the comparison
/// vacuous in the direction that matters: an empty contract set is a subset of anything, so the gate would keep
/// admitting whatever it already admits while reporting agreement. Returning `None` lets the caller say the
/// contract could not be read, which is a different fact from the two sides disagreeing.
///
/// **And exactly one anchor, or the contract could not be read.** `.nth(1)` took the text after the first
/// occurrence, so a second — the shape prose acquires the moment a rule is restated — was dropped in
/// silence, and the agreement direction then compared the gate's list against half its subject while
/// reporting agreement. That is the same silent-narrowing this function's own doc argues against for an
/// empty parse, one level up from it, and `crate::selection` is what answers *how many* rather than
/// inheriting one.
pub fn admitted_types(agents: &str) -> Result<Vec<String>, Refusal> {
    const ANCHOR: &str = "Use the narrowest honest type:";
    // The refusal travels, rather than collapsing to one absence. `the_only` names WHICH way the count was
    // wrong — found none, found N — and `.ok()?` threw that away, so a maintainer who restated the rule in
    // `AGENTS.md` was sent to look for a missing anchor while there were two. The sibling reader repaired in
    // this same window argued the opposite for the identical distinction: *none and several are different
    // facts*. One rule, two readers, and only one of them was following it.
    let clause = crate::selection::the_only("admitted-types anchor", agents.split(ANCHOR).skip(1))?;
    // The run ends at the sentence's period, so a later backticked word — `!`, `BREAKING CHANGE:` — is
    // outside.
    //
    // **`split_once`, because the `else` this replaced was a refusal nothing could produce.** `str::split`
    // always yields at least one item — `"".split(". ").next()` is `Some("")`, measured — so the branch
    // saying *the clause has no sentence after its anchor* was unreachable, and an anchor at the very end of
    // the contract falls where it belongs: it names no backticked type, which is what the refusal below
    // says and is true of it. A diagnostic that cannot be produced is not a distinction the reader draws.
    let (run, _) = clause.split_once(". ").unwrap_or((clause, ""));
    // **The pairing is decided before a pair is taken.** `split('`').skip(1).step_by(2)` paired markers as
    // they came, so an odd count shifted every pair after it: measured, `` `feat`, `fix` and `chore ``
    // — an unterminated trailing run — read as `["feat", "fix", "chore"]`, admitting the tail as a type. A
    // shifted pairing is readable, which is why neither this site nor its two siblings could report it.
    let types = crate::reading::backticked("admitted-types clause", run)?;
    if types.is_empty() {
        return Err(cannot_judge_at(
            "repository-checks#admitted-types-clause-names-no-type",
            "the admitted-types clause names no backticked type, so the contract states no list for the \
             gate's own to be compared against",
        ));
    }
    Ok(types)
}

/// The types this gate judges by, for a direction that holds them against the contract.
pub fn gate_types() -> Vec<String> {
    TYPES.iter().map(|t| (*t).to_string()).collect()
}
