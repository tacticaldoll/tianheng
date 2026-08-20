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

use std::path::Path;

use kanhe::refusal;

use kanhe::merge_message_gate as gate;
use kanhe::supplied::{self, Supplied};
use kanhe::verdict_channel::Verdict;

use gate::judge;
use refusal::Kind;
use refusal::Refusal;

const OK_SUBJECT: &str = "feat(tianheng): hold the squash message to its pull request";
/// Commit subjects a body could be the concatenation of, for directions not about that question.
fn commits() -> Vec<String> {
    vec![
        "feat(x): one thing".to_string(),
        "fix(y): another".to_string(),
    ]
}

const OK_BODY: &str = "Why this exists and what contract it preserves.\n";

/// A refusal from `site`, of `kind`, saying `needle`.
///
/// **The site, and not only the message.** A needle is a phrase inside a rendered message: it cannot tell a
/// branch that was never exercised from one whose wording moved, and it is what the refusal register
/// replaces with a citation the run compares. The needle stays because what the operator is told is the
/// whole of what a refusal delivers, and the site says which branch told them.
fn refuse(subject: &str, body: &str, title: &str, kind: Kind, needle: &str) -> Refusal {
    let refusal = judge(subject, body, title, &commits())
        .expect_err(&format!("expected a refusal containing {needle:?}"));
    assert_eq!(refusal.kind, kind, "{}", refusal.message);
    assert!(
        refusal.message.contains(needle),
        "expected a refusal containing {needle:?}, got: {}",
        refusal.message
    );
    refusal
}

/// The gate, over the message a merge is about to record.
///
/// **One exit.** The body below returns a [`Verdict`] for every path it can take, and `deliver` is what turns
/// that into a channel write and a failure. Before this the harness had three exits and each had to remember
/// to report its class; one of them did not, and a subject supplied as bytes this gate cannot read left
/// through it clean.
#[test]
fn the_squash_message_is_the_pull_request_it_records() {
    kanhe::verdict_channel::deliver("merge message", the_supplied_message());
}

/// The verdict over the message `scripts/merge-pr.sh` supplied, as a value.
fn the_supplied_message() -> Verdict {
    // **One reader for all four judged inputs.** `kanhe::supplied` answers *absent*, *the value*, and *set
    // to bytes this gate cannot read* as three states, and every input here goes through it.
    //
    // Three of the four already did. The fourth — the subject, whose absence means **no merge is being
    // made** — was read with `env::var`, which answers *not set* and *not UTF-8* with one `Err`. The wrapper
    // takes the subject from `argv`, where arbitrary bytes are expressible, so a subject it did supply took
    // the arm that returns clean: the run exited `0`, `require_one_pass` saw `1 passed`, and
    // `exec gh pr merge` recorded a subject no judgement had read. Two spellings of one rule is what let the
    // repair that closed the other three leave `TIANHENG_MERGE_SUBJECT` reading by the rule it replaced.
    let subject = match supplied::from_env("TIANHENG_MERGE_SUBJECT") {
        Supplied::Absent => {
            return Verdict::NotAsked(
                "there is no proposed message until a merge is being made. `scripts/merge-pr.sh` supplies \
                 one and asks for this verdict at that moment"
                    .to_string(),
            );
        }
        Supplied::Unreadable => {
            return Verdict::Refused(refusal::cannot_judge(
                "TIANHENG_MERGE_SUBJECT was supplied as bytes this gate cannot read, so the subject the \
                 merge would record was never judged. That is neither a subject that disagrees nor an \
                 absent one — a merge IS being made, and this gate could not read what it would record",
            ));
        }
        Supplied::Value(subject) => subject,
    };
    // A merge is being made once the subject is here, so a missing input is the wrapper supplying an
    // incomplete set, which is unjudgeable rather than untrue. An empty value that *was* supplied keeps its
    // own meaning: the gate answers an empty title and an empty commit list as cannot-judge and an empty body
    // as a violation, and those are its verdicts to reach.
    let (body, title, commits) = match (
        supplied::from_env("TIANHENG_MERGE_BODY"),
        supplied::from_env("TIANHENG_MERGE_TITLE"),
        supplied::from_env("TIANHENG_MERGE_COMMITS"),
    ) {
        (Supplied::Value(body), Supplied::Value(title), Supplied::Value(commits)) => {
            (body, title, commits)
        }
        (body, title, commits) => {
            let mut unsupplied = Vec::new();
            let mut unreadable = Vec::new();
            for (name, state) in [
                ("TIANHENG_MERGE_BODY", body),
                ("TIANHENG_MERGE_TITLE", title),
                ("TIANHENG_MERGE_COMMITS", commits),
            ] {
                match state {
                    Supplied::Value(_) => {}
                    Supplied::Absent => unsupplied.push(name),
                    Supplied::Unreadable => unreadable.push(name),
                }
            }
            let mut said = Vec::new();
            if !unsupplied.is_empty() {
                said.push(format!("{} not supplied", unsupplied.join(", ")));
            }
            if !unreadable.is_empty() {
                said.push(format!("{} is not UTF-8", unreadable.join(", ")));
            }
            return Verdict::Refused(refusal::cannot_judge(format!(
                "{}, so there is nothing to judge — a merge is being made, because the subject is here, and \
                 this is an incomplete set rather than a message that disagrees",
                said.join("; ")
            )));
        }
    };
    // The pull request's own commit subjects, newline-separated. Empty, the judgement refuses rather than
    // falling back to refusing every bulleted body.
    let supplied: Vec<String> = commits
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_string)
        .collect();
    match judge(&subject, &body, &title, &supplied) {
        Ok(report) => Verdict::Clean(report),
        Err(refusal) => Verdict::Refused(refusal),
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
    let refusal = refuse(
        &format!("{OK_SUBJECT} (#447)"),
        OK_BODY,
        OK_SUBJECT,
        Kind::Violation,
        "ends in a pull request serial",
    );
    refusal::expect(
        "repository-checks#squash-subject-carries-a-serial",
        &refusal,
    );
}

/// A serial in the title too — still the serial's fault, not the comparison's.
#[test]
fn a_serial_in_both_the_subject_and_the_title_is_still_the_serial() {
    let with_serial = format!("{OK_SUBJECT} (#447)");
    let refusal = refuse(
        &with_serial,
        OK_BODY,
        &with_serial,
        Kind::Violation,
        "ends in a pull request serial",
    );
    refusal::expect(
        "repository-checks#squash-subject-carries-a-serial",
        &refusal,
    );
}

#[test]
fn a_subject_that_is_not_the_title_is_a_violation() {
    let refusal = refuse(
        "feat(tianheng): something else entirely",
        OK_BODY,
        OK_SUBJECT,
        Kind::Violation,
        "is not the pull request's title",
    );
    refusal::expect(
        "repository-checks#squash-subject-is-not-the-title",
        &refusal,
    );
}

#[test]
fn a_title_that_cannot_be_read_cannot_be_judged() {
    let refusal = refuse(
        OK_SUBJECT,
        OK_BODY,
        "   ",
        Kind::CannotJudge,
        "title is unavailable",
    );
    refusal::expect("repository-checks#squash-title-unavailable", &refusal);
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
        let refusal = refuse(
            subject,
            OK_BODY,
            subject,
            Kind::Violation,
            "is not a Conventional Commit",
        );
        refusal::expect(
            "repository-checks#squash-subject-is-not-conventional",
            &refusal,
        );
    }
}

#[test]
fn a_breaking_subject_with_no_migration_footer_is_a_violation() {
    let subject = "refactor(tianheng)!: move the observer surface";
    let refusal = refuse(
        subject,
        OK_BODY,
        subject,
        Kind::Violation,
        "names no `BREAKING CHANGE:` footer",
    );
    refusal::expect(
        "repository-checks#squash-breaking-without-a-migration-footer",
        &refusal,
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
        let refusal = refuse(
            OK_SUBJECT,
            &format!("{OK_BODY}\n{line}\n"),
            OK_SUBJECT,
            Kind::Violation,
            "carries the agent attribution",
        );
        refusal::expect(
            "repository-checks#squash-message-carries-an-attribution",
            &refusal,
        );
    }
    // The glyph is refused wherever it sits, including mid-subject: it has no legitimate use here, and reading it
    // by position would have let this exact shape through — the false negative the first draft opened.
    let refusal = refuse(
        "fix(kanhe): 🤖 wrote this",
        OK_BODY,
        "fix(kanhe): 🤖 wrote this",
        Kind::Violation,
        "carries the agent attribution",
    );
    refusal::expect(
        "repository-checks#squash-message-carries-an-attribution",
        &refusal,
    );
}

/// The refusal states the rule of the shape that actually fired, not the other one.
///
/// **The half nothing held.** One `format!` served both recognition shapes and carried the trailer sentence:
/// *naming the mark inside a sentence is not carrying it; a line that begins with it is*. A glyph matches
/// wherever it sits, so an operator refused for `fix(x): 🤖 wrote this` was handed the rule that would have
/// permitted it — in front of a record no rerun amends. Both directions above assert only the shared half,
/// which cannot tell the two sentences apart; this asserts each shape's own.
#[test]
fn the_refusal_states_the_rule_of_the_shape_that_fired() {
    let trailer = refuse(
        OK_SUBJECT,
        &format!("{OK_BODY}\nCo-authored-by: Someone <a@b.invalid>\n"),
        OK_SUBJECT,
        Kind::Violation,
        "a line that begins with it is carrying it",
    );
    assert!(
        !trailer.message.contains("wherever it appears"),
        "a trailer refusal must not carry the glyph rule: {}",
        trailer.message
    );

    let glyph = refuse(
        "fix(kanhe): 🤖 wrote this",
        OK_BODY,
        "fix(kanhe): 🤖 wrote this",
        Kind::Violation,
        "wherever it appears",
    );
    assert!(
        !glyph.message.contains("a line that begins with it"),
        "a glyph refused mid-line must not be handed the rule that would have permitted it: {}",
        glyph.message
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
    refusal::expect("repository-checks#squash-commits-unavailable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge);
    assert!(
        refusal.message.contains("commit subjects are unavailable"),
        "{}",
        refusal.message
    );
}

#[test]
fn an_empty_body_is_a_violation() {
    let refusal = refuse(
        OK_SUBJECT,
        "  \n\n",
        OK_SUBJECT,
        Kind::Violation,
        "body is empty",
    );
    refusal::expect("repository-checks#squash-body-is-empty", &refusal);
}

#[test]
fn a_body_that_is_a_bare_commit_list_is_a_violation() {
    let refusal = refuse(
        OK_SUBJECT,
        &format!("* {}\n* {}\n", commits()[0], commits()[1]),
        OK_SUBJECT,
        Kind::Violation,
        "bare list of commit subjects",
    );
    refusal::expect(
        "repository-checks#squash-body-is-a-bare-commit-list",
        &refusal,
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
    // The refusal's own message travels into the panic: it names WHICH way the count was wrong, which is
    // what a maintainer acts on. A collapsed `None` sent one looking for a missing anchor while there were two.
    let contract = kanhe::merge_message_gate::admitted_types(&agents).unwrap_or_else(|refusal| {
        panic!(
            "cannot read the admitted Conventional Commit types from AGENTS.md: {} — the clause naming the \
             narrowest honest type is the anchor, and an unparsed contract is not an empty one",
            refusal.message
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
        kanhe::merge_message_gate::admitted_types(&one).expect("one anchor reads its own run"),
        vec!["feat".to_string(), "fix".to_string()],
        "the control: one anchor reads its own run"
    );

    // **The message, not only the absence.** Both counts were `None` before, so a maintainer who restated
    // the rule was told the anchor was missing while there were two of them — and pinning the collapse is
    // what would have kept the diagnostic from improving.
    let two = format!("{one}\n\nrestated: {ANCHOR} `refactor`. and on");
    let several = kanhe::merge_message_gate::admitted_types(&two)
        .expect_err("two anchors are a contract this reader may not choose between");
    assert!(
        several.message.contains("found 2"),
        "the refusal must say there were two, got: {}",
        several.message
    );

    // An anchor ending the contract: the clause after it is empty, which is the input the unreachable
    // *no sentence after its anchor* branch claimed. It names no backticked type, which is both true of it
    // and the only fact a reader can act on.
    let ends = format!("prose. {ANCHOR}");
    let ended = kanhe::merge_message_gate::admitted_types(&ends)
        .expect_err("an anchor with nothing after it states no list");
    refusal::expect(
        "repository-checks#admitted-types-clause-names-no-type",
        &ended,
    );
    assert!(
        ended.message.contains("names no backticked type"),
        "an anchor ending the contract must be reported as stating no list, got: {}",
        ended.message
    );

    let none = kanhe::merge_message_gate::admitted_types("no anchor at all")
        .expect_err("no anchor is still unreadable");
    assert!(
        none.message.contains("found none"),
        "and this one must say there were none, got: {}",
        none.message
    );
}

// --- the harness boundary, driven as the wrapper drives it -------------------------------------------------

/// The gate re-run in a child process with the four judged inputs supplied, and the class it reported.
///
/// **A direction over the harness cannot read the harness's own environment.** The four inputs arrive as
/// process environment, a parallel test run shares one, and `set_var` mutates it for every sibling — so the
/// only way to give this gate an input is to be a different process. The test binary re-executes itself with
/// `--exact`, which is the same selection `scripts/merge-pr.sh` uses and therefore the same code path.
///
/// Returns `(the child exited zero, what it wrote to the verdict channel)`. An absent channel file is an empty
/// string, which is what a wrapper reads as *no verdict was reached*.
fn gate_over(subject: &std::ffi::OsStr) -> (bool, String) {
    gate_over_channel(subject, None)
}

/// The same, with the channel pointed somewhere of the caller's choosing.
fn gate_over_channel(subject: &std::ffi::OsStr, channel: Option<&Path>) -> (bool, String) {
    // **One path, decided once.** The env and the read-back were two expressions: a caller supplying a
    // channel had the child write there while this read the default scratch file, so the returned class was
    // always empty. Its only caller passed an unwritable path, where empty is also the right answer — so the
    // mismatch was invisible, and the direction asserting `reported.is_empty()` passed for the wrong reason.
    let scratch = std::env::temp_dir().join(format!(
        "kanhe-merge-subject-{}-{}",
        std::process::id(),
        SUBJECT_PROBE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch)
        .expect("claim a scratch root for the child's verdict channel");
    let verdict = scratch.join("verdict");
    let channel_path = channel.unwrap_or(verdict.as_path());

    let out = std::process::Command::new(
        std::env::current_exe()
            .expect("this test binary's own path, to re-run one direction in a child"),
    )
    .args([
        "--exact",
        "the_squash_message_is_the_pull_request_it_records",
    ])
    .env(kanhe::verdict_channel::ENV, channel_path)
    .env("TIANHENG_MERGE_SUBJECT", subject)
    .env("TIANHENG_MERGE_TITLE", OK_SUBJECT)
    .env("TIANHENG_MERGE_BODY", OK_BODY)
    .env("TIANHENG_MERGE_COMMITS", commits().join("\n"))
    .output()
    .expect("re-run this binary's gate direction in a child process");

    let reported = std::fs::read_to_string(channel_path).unwrap_or_default();
    let _ = std::fs::remove_dir_all(&scratch);
    (out.status.success(), reported)
}

static SUBJECT_PROBE: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);

/// A subject the wrapper supplied but this gate cannot read is not a subject it was never given.
///
/// `scripts/merge-pr.sh` takes the subject from `argv` — `--subject <text>` — which on this platform carries
/// arbitrary bytes. Read with `env::var`, *not set* and *set but not UTF-8* are one `Err`, and the arm that
/// answers it means **no merge is being made**: the gate prints "not judged", returns, and the run exits `0`.
/// The wrapper's `require_one_pass` then sees `1 passed`, and `exec gh pr merge` records a subject no
/// judgement ever read — the one direction the Core Contract forbids, in front of a record that cannot be
/// amended.
///
/// The control below is what makes this a measurement rather than a coincidence: the same four inputs with a
/// readable subject reach a verdict, so the child is exercising the gate and the two runs differ **only** in
/// the subject's bytes.
#[test]
fn a_subject_supplied_as_bytes_this_gate_cannot_read_is_not_an_absent_subject() {
    use std::os::unix::ffi::OsStrExt;

    let (control_ok, control_verdict) = gate_over(std::ffi::OsStr::new(OK_SUBJECT));
    assert!(
        control_ok && control_verdict == kanhe::verdict_channel::CLEAN,
        "the control must reach a clean verdict and say so on the channel, or the two runs below differ for \
         a reason other than the subject's bytes: exited zero = {control_ok}, channel = {control_verdict:?}"
    );

    let unreadable = std::ffi::OsStr::from_bytes(b"fix(kanhe): \xff\xfe not utf-8");
    let (exited_zero, reported) = gate_over(unreadable);
    assert!(
        !exited_zero,
        "a subject supplied as bytes this gate cannot read must stop the run; it exited zero, so \
         `require_one_pass` would see a pass and the merge would record a subject nothing judged"
    );
    assert_eq!(
        reported,
        kanhe::verdict_channel::rendered(Kind::CannotJudge),
        "and it must report cannot-judge on the channel: the wrapper supplied this input, so it is not a \
         message that disagrees"
    );
}

/// A verdict reached and lost is not a verdict never reached.
///
/// **The channel write's outcome used to be discarded.** `deliver` called the writer and dropped its
/// `bool`, so a `Refused` whose write failed left the same absent file a run that judged nothing leaves —
/// and the wrapper, reading absence as unjudged, reported exit 2 where the gate had found exit 1. The
/// module and `repository-checks` both claim *absent means unjudged by construction*; with the outcome
/// dropped, absence had two causes and only one of them was that.
///
/// **The channel is made unwritable by naming a path under a directory that does not exist**, not by
/// permissions: `publish-source-integrity#signature-unwritable` is declared unheld precisely because a
/// permission-based fixture answers differently for root, which makes the direction's own result depend on
/// who runs it. A missing parent fails the same way for everyone.
#[test]
fn a_verdict_that_cannot_reach_the_channel_is_not_an_absent_one() {
    let unwritable = std::env::temp_dir()
        .join(format!("kanhe-no-such-dir-{}", std::process::id()))
        .join("verdict");
    assert!(
        !unwritable
            .parent()
            .expect("the probe path has a parent")
            .exists(),
        "the probe depends on the parent being absent, or it measures something else"
    );

    let (exited_zero, reported) =
        gate_over_channel(std::ffi::OsStr::new(OK_SUBJECT), Some(&unwritable));
    assert!(
        !exited_zero,
        "a verdict the gate reached and could not put on the channel must stop the run; silently leaving \
         the file absent is what makes `absent means unjudged` false"
    );
    assert!(
        reported.is_empty(),
        "and nothing may be on the channel, which is the state this refuses to let stand for a verdict: \
         {reported:?}"
    );
}

/// A **refused** verdict that cannot reach the channel, and what the wrapper still sees.
///
/// **This is the case the repair's own record over-claimed.** It said a `Refused` whose write failed reached
/// the wrapper as exit `2` where the gate had found exit `1`, and that the repair ended that collapse. It does
/// not: the channel is absent either way, so the wrapper reads unjudged either way. What changed is that the
/// gate now names the channel and the error instead of failing on the refusal alone — the operator is told
/// which of the two facts they have, and the exit class is not what carries it.
///
/// Held here because the direction that shipped with the repair used a clean verdict, so it proved the clean
/// arm and was cited for the refused one.
#[test]
fn a_refused_verdict_that_cannot_reach_the_channel_still_reads_as_unjudged() {
    let unwritable = std::env::temp_dir()
        .join(format!("kanhe-no-such-dir-refused-{}", std::process::id()))
        .join("verdict");

    // A subject that is not the title is a violation — the gate's own `Refused`.
    let (exited_zero, reported) = gate_over_channel(
        std::ffi::OsStr::new("fix(kanhe): a subject that is not the title"),
        Some(&unwritable),
    );
    assert!(!exited_zero, "a refused verdict must fail the run");
    assert!(
        reported.is_empty(),
        "and the channel carries nothing, which is what a wrapper reads as unjudged — the exit class it \
         then reports is 2, the same as before this was repaired: {reported:?}"
    );
}

/// The channel this helper names is the channel it reads.
///
/// **The falsifier for a mismatch its only caller could not see.** With the env and the read-back as two
/// expressions, a caller supplying a channel had the child write there while the helper read the default
/// scratch file — and every caller passed an unwritable path, where empty is also the correct answer. A
/// writable custom channel is the one input that tells the two apart.
#[test]
fn the_channel_this_helper_names_is_the_one_it_reads() {
    let scratch = std::env::temp_dir().join(format!(
        "kanhe-channel-roundtrip-{}-{}",
        std::process::id(),
        SUBJECT_PROBE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("claim a scratch root for a writable channel");
    let channel = scratch.join("elsewhere");

    let (exited_zero, reported) =
        gate_over_channel(std::ffi::OsStr::new(OK_SUBJECT), Some(&channel));
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(exited_zero, "a clean verdict does not fail the run");
    assert_eq!(
        reported,
        kanhe::verdict_channel::CLEAN,
        "the class must come back from the channel the caller named, not from the default one"
    );
}
