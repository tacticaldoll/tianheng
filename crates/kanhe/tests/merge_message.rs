//! Repository check: the squash message a merge is about to record.
//!
//! It stands before a record. A merged squash on a release branch cannot be repaired: amending it changes its
//! hash, and the pull request's merge record cites that hash, so the two would name different things. The
//! subjects already carrying a serial when this gate was written stay as they are for exactly that reason.
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

/// A line that *is* an attribution mark is refused, whatever its case.
///
/// The exact-case `contains` this replaced let the canonical spellings straight through — git writes the trailer
/// `Co-authored-by:` and GitHub renders it that way, so the one form most likely to appear was the one form not
/// caught. Measured before the widening: `co-authored-by: Claude` and `generated with Claude Code` were both
/// accepted.
#[test]
fn a_line_that_is_an_agent_attribution_is_a_violation() {
    for line in [
        // The form the old check caught, and the three it did not.
        "Co-Authored-By: Someone <a@b.invalid>",
        "Co-authored-by: Someone <a@b.invalid>",
        "co-authored-by: someone <a@b.invalid>",
        "CO-AUTHORED-BY: SOMEONE",
        // Indented, because a trailer git honours may carry leading space and a reader would still read it as one.
        "   Co-authored-by: Someone <a@b.invalid>",
        "Generated with a tool",
        "generated with a tool",
        "🤖 made this",
    ] {
        refuse(
            OK_SUBJECT,
            &format!("{OK_BODY}\n{line}\n"),
            OK_SUBJECT,
            Kind::Violation,
            "is the agent attribution",
        );
    }
    // The glyph is refused wherever it sits, including mid-subject: it has no legitimate use here, and reading it
    // by position would have let this exact shape through — the false negative the first draft opened.
    refuse(
        "fix(kanhe): 🤖 wrote this",
        OK_BODY,
        "fix(kanhe): 🤖 wrote this",
        Kind::Violation,
        "is the agent attribution",
    );
}

/// A body that NAMES a mark inside a sentence is not a body that carries one.
///
/// The load-bearing half. `repository-checks` forbids this gate to refuse a shape for what it resembles, and a
/// bare substring refuses the commit message of any change about this rule — including the one that widened it,
/// whose own body names every form. Widening the case without narrowing to the line would have traded one defect
/// for the other, so they are one change.
#[test]
fn a_sentence_naming_an_attribution_mark_is_not_carrying_one() {
    for line in [
        "This change removes the `co-authored-by: Claude` trailer the old rule missed.",
        "The forms are Co-Authored-By, Generated with, and the robot glyph.",
        "The third form is the robot glyph, named in words here because pasting it would be carrying it.",
    ] {
        let body = format!("{OK_BODY}\n{line}\n");
        assert!(
            judge(OK_SUBJECT, &body, OK_SUBJECT, &commits()).is_ok(),
            "naming a mark in prose must not be refused: {line}"
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
/// `COMMIT_OR_PR_TITLE`, and subjects in its history carry the serial that setting produced.
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

/// The types this gate judges by are the types the contract admits, in both directions.
///
/// `TYPES` was documented as *"the Conventional Commit types `AGENTS.md` admits"* — a second copy of a list the
/// contract states in prose, with nothing holding them equal. Diverge them and the gate refuses a subject the
/// contract admits, or admits one it forbids, and either way the wrapper standing in front of an unamendable
/// record enforces something other than the rule.
///
/// Both directions, because the two failures differ: a type in the contract and not the gate is a subject
/// wrongly refused, and a type in the gate and not the contract is one wrongly admitted.
///
/// A contract that cannot be parsed is a **cannot-judge**, not an empty set. An empty set is a subset of
/// anything, so a silent parse failure would report agreement while the gate went on admitting whatever it
/// already admits.
#[test]
fn the_gate_admits_exactly_the_types_the_contract_does() {
    let Some(root) = shengmo::workspace::locate(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("AGENTS.md").is_file(),
        shengmo::workspace::marker_set(),
    ) else {
        return;
    };
    let agents = std::fs::read_to_string(root.join("AGENTS.md"))
        .expect("AGENTS.md is the contract this gate answers to and must be readable");
    let contract = kanhe::merge_message_gate::admitted_types(&agents).unwrap_or_else(|| {
        panic!(
            "cannot read the admitted Conventional Commit types from AGENTS.md — the clause naming the \
             narrowest honest type is the anchor, and an unparsed contract is not an empty one"
        )
    });
    let contract: std::collections::BTreeSet<String> = contract.into_iter().collect();
    let gate: std::collections::BTreeSet<String> = kanhe::merge_message_gate::gate_types()
        .into_iter()
        .collect();

    let refused_but_admitted: Vec<&String> = contract.difference(&gate).collect();
    assert!(
        refused_but_admitted.is_empty(),
        "AGENTS.md admits these types and the gate would refuse a subject using them: {refused_but_admitted:?}"
    );
    let admitted_but_unstated: Vec<&String> = gate.difference(&contract).collect();
    assert!(
        admitted_but_unstated.is_empty(),
        "the gate admits these types and AGENTS.md does not state them: {admitted_but_unstated:?}"
    );
}

/// Two anchors mean the contract could not be read, never the first one's answer.
///
/// The agreement direction above compares the gate's list against the contract's. Reading past a second
/// anchor — the shape prose acquires the moment a rule is restated, quoted, or given an example — would have
/// it compare against half its subject while reporting agreement, which is the silent narrowing
/// `admitted_types`' own doc argues against for an empty parse, one level up from it.
///
/// Negative run: with `.nth(1)`, the two-anchor input answers `Some(["feat", "fix"])` — the first anchor's
/// list, with the second's `refactor` dropped and nothing said.
#[test]
fn two_admitted_type_anchors_cannot_be_read() {
    const ANCHOR: &str = "Use the narrowest honest type:";
    let one = format!("prose. {ANCHOR} `feat`, `fix`. more prose");
    assert_eq!(
        kanhe::merge_message_gate::admitted_types(&one),
        Some(vec!["feat".to_string(), "fix".to_string()]),
        "the control: one anchor reads its own run"
    );

    let two = format!("{one}\n\nrestated: {ANCHOR} `refactor`. and on");
    assert_eq!(
        kanhe::merge_message_gate::admitted_types(&two),
        None,
        "two anchors are a contract this reader may not choose between — not the first one's list"
    );

    assert_eq!(
        kanhe::merge_message_gate::admitted_types("no anchor at all"),
        None,
        "and no anchor is still unreadable, as it was"
    );
}
