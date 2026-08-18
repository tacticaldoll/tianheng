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
    ("co-authored-by", Shape::Trailer),
    ("generated with", Shape::Trailer),
    ("🤖", Shape::Glyph),
];

/// How a mark is recognized. **Not every mark is the same kind of thing.**
///
/// A first draft read all three by position and would have introduced a false negative: `fix(x): 🤖 wrote this`
/// was refused by the substring it replaced and would have passed, because the glyph sits mid-line. The two kinds
/// need different recognition, so the recognition travels in the same array as the mark rather than in a second
/// rule beside it.
#[derive(Clone, Copy)]
enum Shape {
    /// A key on a line of its own. Prose naming it is not it, and GitHub honours only the line form.
    Trailer,
    /// A glyph with no legitimate use in this repository's commit messages, wherever it appears. Prose about the
    /// rule names it in words instead — which is what this repository's own prose does.
    Glyph,
}

/// Whether `text` carries `mark` in the way its shape defines.
fn carries(text: &str, mark: &str, shape: Shape) -> bool {
    match shape {
        Shape::Trailer => text
            .lines()
            .any(|line| line.trim().to_ascii_lowercase().starts_with(mark)),
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
    if !is_conventional(subject) {
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
                    "a line of the squash message is the agent attribution {mark:?}, which this repository's commit \
                 messages and pull request descriptions do not carry. Naming the mark inside a sentence is not \
                 carrying it; a line that begins with it is"
                ),
            ));
        }
    }
    if body.trim().is_empty() {
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
    let types: Vec<String> = run
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect();
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
