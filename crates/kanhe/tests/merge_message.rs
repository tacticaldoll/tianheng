//! Repository check: the squash message a merge is about to record.
//!
//! It stands before a record. A merged squash on a release branch cannot be repaired: amending it changes its
//! hash, and the pull request's merge record cites that hash, so the two would name different things. The
//! nine subjects already carrying a serial stay as they are for exactly that reason.
//!
//! **The gate itself does not run in development.** There is no proposed message to judge until a merge is
//! being made, so `scripts/merge-pr.sh` supplies one and asks for the verdict at the one moment it can
//! answer. What runs in the ordinary suite is the failure matrix below, which holds the judgement to refusing
//! each shape — and to refusing it with its **own** message, so that no two sites can stand in for each
//! other.

use kanhe::refusal;

use kanhe::merge_message_gate as gate;

use gate::judge;
use refusal::Kind;

const OK_SUBJECT: &str = "feat(tianheng): hold the squash message to its pull request";
/// Commit subjects a body could be the concatenation of, for directions not about that question.
fn commits() -> Vec<String> {
    vec![
        "feat(x): one thing".to_string(),
        "fix(y): another".to_string(),
    ]
}

const OK_BODY: &str = "Why this exists and what contract it preserves.\n";

fn refuse(subject: &str, body: &str, title: &str, kind: Kind, needle: &str) {
    let refusal = judge(subject, body, title, &commits())
        .expect_err(&format!("expected a refusal containing {needle:?}"));
    assert_eq!(refusal.kind, kind, "{}", refusal.message);
    assert!(
        refusal.message.contains(needle),
        "expected a refusal containing {needle:?}, got: {}",
        refusal.message
    );
}

/// The gate, over the message a merge is about to record.
#[test]
fn the_squash_message_is_the_pull_request_it_records() {
    let Ok(subject) = std::env::var("TIANHENG_MERGE_SUBJECT") else {
        eprintln!(
            "merge message: not judged — there is no proposed message until a merge is being made. \
             `scripts/merge-pr.sh` supplies one and asks for this verdict at that moment."
        );
        return;
    };
    let body = std::env::var("TIANHENG_MERGE_BODY").unwrap_or_default();
    let title = std::env::var("TIANHENG_MERGE_TITLE").unwrap_or_default();
    // The pull request's own commit subjects, newline-separated. Absent, the judgement refuses rather than
    // falling back to refusing every bulleted body.
    let supplied: Vec<String> = std::env::var("TIANHENG_MERGE_COMMITS")
        .unwrap_or_default()
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect();
    match judge(&subject, &body, &title, &supplied) {
        Ok(report) => eprintln!("{report}"),
        Err(refusal) => {
            // The class travels on its own channel, written before this fails, so a wrapper reads a verdict
            // rather than searching this message for one. See `kanhe::verdict_channel`.
            kanhe::verdict_channel::report(refusal.kind);
            panic!("merge message ({:?}): {}", refusal.kind, refusal.message)
        }
    }
}

// --- the failure matrix ------------------------------------------------------------------------------------

/// The whole shape, accepted — so every refusal below is about the thing it names.
#[test]
fn a_subject_that_is_its_title_with_a_body_is_accepted() {
    let verdict = judge(OK_SUBJECT, OK_BODY, OK_SUBJECT, &commits());
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// The defect this check was built for, in the exact shape it took.
#[test]
fn a_subject_carrying_a_pull_request_serial_is_a_violation() {
    refuse(
        &format!("{OK_SUBJECT} (#447)"),
        OK_BODY,
        OK_SUBJECT,
        Kind::Violation,
        "ends in a pull request serial",
    );
}

/// A serial in the title too — still the serial's fault, not the comparison's.
#[test]
fn a_serial_in_both_the_subject_and_the_title_is_still_the_serial() {
    let with_serial = format!("{OK_SUBJECT} (#447)");
    refuse(
        &with_serial,
        OK_BODY,
        &with_serial,
        Kind::Violation,
        "ends in a pull request serial",
    );
}

#[test]
fn a_subject_that_is_not_the_title_is_a_violation() {
    refuse(
        "feat(tianheng): something else entirely",
        OK_BODY,
        OK_SUBJECT,
        Kind::Violation,
        "is not the pull request's title",
    );
}

#[test]
fn a_title_that_cannot_be_read_cannot_be_judged() {
    refuse(
        OK_SUBJECT,
        OK_BODY,
        "   ",
        Kind::CannotJudge,
        "title is unavailable",
    );
}

#[test]
fn a_subject_that_is_not_a_conventional_commit_is_a_violation() {
    for subject in [
        "update the refusal sweep",
        "Feat(tianheng): capitalised type",
        "feat(Tianheng): capitalised scope",
        "feat(tianheng):",
        "chore: ",
    ] {
        refuse(
            subject,
            OK_BODY,
            subject,
            Kind::Violation,
            "is not a Conventional Commit",
        );
    }
}

#[test]
fn a_breaking_subject_with_no_migration_footer_is_a_violation() {
    let subject = "refactor(tianheng)!: move the observer surface";
    refuse(
        subject,
        OK_BODY,
        subject,
        Kind::Violation,
        "names no `BREAKING CHANGE:` footer",
    );
    let with_footer = format!("{OK_BODY}\nBREAKING CHANGE: adopters regenerate their baseline.\n");
    assert!(judge(subject, &with_footer, subject, &commits()).is_ok());
}

#[test]
fn agent_attribution_anywhere_in_the_message_is_a_violation() {
    for body in [
        format!("{OK_BODY}\nCo-Authored-By: Someone <a@b.invalid>\n"),
        format!("{OK_BODY}\nGenerated with a tool\n"),
        format!("{OK_BODY}\n🤖 made this\n"),
    ] {
        refuse(
            OK_SUBJECT,
            &body,
            OK_SUBJECT,
            Kind::Violation,
            "carries the agent attribution",
        );
    }
}

/// A summary may carry an exclamation mark; only the head marks a migration.
#[test]
fn a_bang_in_the_summary_is_not_a_breaking_marker() {
    let subject = "fix(tianheng): preserve bang! in summaries";
    let verdict = judge(subject, OK_BODY, subject, &commits());
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// A terse body written as bullets none of which is a commit subject is self-contained.
#[test]
fn a_bullet_body_that_is_not_the_commit_subjects_is_accepted() {
    let verdict = judge(
        OK_SUBJECT,
        "- Why: the contract this preserves.\n- Contract: what it must not break.\n",
        OK_SUBJECT,
        &["feat(x): one".to_string(), "fix(y): another".to_string()],
    );
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

/// Without the commit subjects the judgement refuses rather than falling back to the shape, which is the
/// false refusal reading them removes.
#[test]
fn a_body_judged_without_the_commit_subjects_cannot_be_judged() {
    let refusal = judge(OK_SUBJECT, "- a bullet\n", OK_SUBJECT, &[])
        .expect_err("no commit subjects is a refusal to judge, not a fallback");
    assert_eq!(refusal.kind, Kind::CannotJudge);
    assert!(
        refusal.message.contains("commit subjects are unavailable"),
        "{}",
        refusal.message
    );
}

#[test]
fn an_empty_body_is_a_violation() {
    refuse(
        OK_SUBJECT,
        "  \n\n",
        OK_SUBJECT,
        Kind::Violation,
        "body is empty",
    );
}

#[test]
fn a_body_that_is_a_bare_commit_list_is_a_violation() {
    refuse(
        OK_SUBJECT,
        &format!("* {}\n* {}\n", commits()[0], commits()[1]),
        OK_SUBJECT,
        Kind::Violation,
        "bare list of commit subjects",
    );
}

/// `repository-checks/a-hook-is-proposed-for-this-rule-a-stated-bound`
///
/// `OutOfReach`, owned by the engine. This check guards the **sanctioned path** to a merge, not every path.
/// A merge made in the GitHub web UI reaches no wrapper, and neither a `commit-msg` hook nor the repository's
/// squash-title setting can hold the rule at all — the first because a squash merge creates no local commit,
/// the second because both of its values append the serial.
///
/// Measured rather than reasoned: this repository's settings report `squash_merge_commit_title` as
/// `COMMIT_OR_PR_TITLE`, and nine subjects in its history carry the serial that setting produced.
#[test]
fn a_merge_made_outside_the_wrapper_is_not_observed() {
    // What the check does hold: a message handed to it.
    assert!(judge(OK_SUBJECT, OK_BODY, OK_SUBJECT, &commits()).is_ok());
    assert!(
        judge(
            &format!("{OK_SUBJECT} (#1)"),
            OK_BODY,
            OK_SUBJECT,
            &commits()
        )
        .is_err()
    );

    // What it cannot: a merge that never hands it one. There is no input to this function representing a
    // merge made elsewhere, which is the bound — the judgement is over a message, and a browser supplies
    // none. Reaching further would mean observing GitHub's server, not this repository.
    let observed_without_a_message = judge("", "", "", &commits());
    assert_eq!(
        observed_without_a_message.err().map(|r| r.kind),
        Some(Kind::CannotJudge),
        "with no message at all the check can only refuse to judge, which is exactly what it can say \
         about a merge made outside the wrapper"
    );
}
