//! The squash-message judgement, shared by the gate and by its failure matrix.
//!
//! `AGENTS.md` states these rules and nothing held them: **9** subjects in this repository's history carry a
//! trailing `(#N)`, the most recent on the commit that landed a reaction for a requirement enforced by
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
//! could not be read* — and so these construction sites are enumerated and perturbed by `refusal_bites` like
//! every other.

#![allow(dead_code)]

use crate::refusal::{Refusal, cannot_judge, violation};

/// The Conventional Commit types `AGENTS.md` admits.
const TYPES: [&str; 9] = [
    "feat", "fix", "refactor", "docs", "test", "build", "ci", "perf", "chore",
];

/// Marks an agent wrote it, in any of the forms `AGENTS.md` names.
const ATTRIBUTION: [&str; 3] = ["Co-Authored-By", "Generated with", "🤖"];

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
        return Err(cannot_judge(
            "the pull request's title is unavailable, so whether the subject is that title cannot be \
             decided — which is not the same fact as a subject that disagrees",
        ));
    }
    if carries_a_serial(subject) {
        return Err(violation(format!(
            "the squash subject ends in a pull request serial: {subject:?}. GitHub appends `(#N)` to the \
             default subject and neither value of the repository's squash-title setting suppresses it, so \
             the subject is passed explicitly — without the serial"
        )));
    }
    if subject != title {
        return Err(violation(format!(
            "the squash subject is not the pull request's title.\n  subject: {subject:?}\n  title:   \
             {title:?}\nThe title is what review saw; a subject saying something else makes the record \
             disagree with what was approved"
        )));
    }
    if !is_conventional(subject) {
        return Err(violation(format!(
            "the squash subject is not a Conventional Commit: {subject:?}. Expected \
             `<type>(<scope>)!?: <summary>` with a lowercase type from {TYPES:?}"
        )));
    }
    // The head, not the whole subject: a summary may carry an exclamation mark for its own reasons, and the
    // shape check above already reads the head to strip a trailing `!` before matching the type.
    let head_is_breaking = subject
        .split_once(": ")
        .is_some_and(|(head, _)| head.ends_with('!'));
    if head_is_breaking && !body.contains("BREAKING CHANGE:") {
        return Err(violation(
            "the squash subject is marked breaking and the body names no `BREAKING CHANGE:` footer, so the \
             record announces a migration it does not describe",
        ));
    }
    for mark in ATTRIBUTION {
        if subject.contains(mark) || body.contains(mark) {
            return Err(violation(format!(
                "the squash message carries the agent attribution {mark:?}, which this repository's commit \
                 messages and pull request descriptions do not"
            )));
        }
    }
    if body.trim().is_empty() {
        return Err(violation(
            "the squash body is empty; a commit body carries why the change exists and what contract it \
             preserves, and the branch's fine-grained commits are review provenance rather than this record",
        ));
    }
    if commits.is_empty() {
        return Err(cannot_judge(
            "the pull request's commit subjects are unavailable, so whether this body is the default \
             concatenation of them cannot be decided — falling back to refusing every bulleted body is the \
             over-reaction this reads them to avoid",
        ));
    }
    if is_a_bare_commit_list(body, commits) {
        return Err(violation(
            "the squash body is a bare list of commit subjects, which is the default this rule exists to \
             replace with something self-contained",
        ));
    }
    Ok(format!("ok merge message ({subject})"))
}
