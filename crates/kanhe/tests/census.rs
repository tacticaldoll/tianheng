//! Repository check: every declared census agrees with the check that produces it.
//!
//! Each census below is **produced** by the enumeration it is about — the register's own parse — never by a
//! second reading of the same source. A second parse would let the census and
//! the check disagree, which is the drift this exists to end.

use kanhe::bound_register_parse as register;
use kanhe::census;

use census::{Census, Sweep, sweep};
use kanhe::refusal;
use refusal::Kind;
use register::{Citation, parse_bounds, workspace_root};
use std::collections::BTreeSet;

fn tracked(root: &std::path::Path) -> Vec<String> {
    let out = std::process::Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("run git ls-files");
    assert!(
        out.status.success(),
        "`git ls-files` failed; a failed enumeration is not a repository holding no documents"
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn every_declared_census_agrees_with_what_produces_it() {
    let Some(root) = workspace_root() else {
        return;
    };
    let files = tracked(&root);

    let bounds = parse_bounds(&root);
    let capabilities: BTreeSet<&str> = bounds.iter().map(|b| b.capability.as_str()).collect();
    let unpinned = bounds
        .iter()
        .filter(|b| !matches!(b.citation, Citation::PinnedBy(_)))
        .count();
    // **A figure about a past state is a record, and it is now CLOSED rather than declared.** This comment
    // used to name `[Unreleased]`'s prose as the instance and say the residual was declared as a bound. Both
    // halves had stopped being true: that section is empty, so the instance is gone, and no bound of this
    // family ever declared the record case — the ones that exist are about words at a hundred and above, a
    // census outside Markdown, and a count in a phrasing no census declares. Meanwhile the residual was
    // live: a figure inside a dated `CHANGELOG.md` section was refused for disagreeing with today's
    // enumeration, escaping only because its two numbers straddled a line break. `record` now cuts records
    // out of this sweep's corpus, which is what the comment claimed a bound was doing.
    let declared = vec![
        Census {
            subject: "declared observation bounds and the capabilities declaring them",
            phrase: "{} bounds across {} capabilities",
            figures: vec![bounds.len(), capabilities.len()],
        },
        Census {
            subject: "declared observation bounds with no pinning test",
            phrase: "{} of {} declared bounds have no pinning test",
            figures: vec![unpinned, bounds.len()],
        },
    ];

    let Sweep { offences, stating } = sweep(&root, &files, &declared);
    // **Printed because the alternative is assuming it, and the assumption was wrong twice.** The rule this
    // serves says a count of a live set is not written, and it was twice concluded from that that nothing
    // states a census at all — by a grep for one of the declared phrasings, over a corpus that skipped the
    // records. One document does: a generated projection whose figure the renderer computes, which is the one
    // place a figure belongs. So this sweep is not armed-and-idle; it is what makes *produced* checkable.
    //
    // The figure comes from the sweep's own pass. A first version walked the corpus again with its own
    // filter, which excluded whole record documents but not a record's dated sections — so a correct
    // historical sentence could raise it, and the message below would have read that as a hand-written count
    // arriving. One enumerator, one record cut.
    eprintln!(
        "census ok ({stating} tracked document(s) state a declared census, over {} declared) — a figure \
         belongs only where it is produced, so the documents this finds should be the generated projections \
         and nothing else; a rise here is a hand-written count arriving",
        declared.len()
    );
    assert!(
        offences.is_empty(),
        "a hand-written census disagrees with the check that enumerates its set, or a tracked document could \
         not be read:\n{}",
        offences
            .iter()
            .map(|refusal| format!("{:?}: {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// A tracked document the sweep cannot read is reported as **cannot judge**, not skipped.
///
/// The distinction is the whole point of the typed result: skipping would report clean over a corpus the sweep
/// never examined, and a clean verdict that rests on an unread file is the shape its sibling reference gate
/// refuses outright. Shown rather than asserted about — the corpus names a document that is not there, which
/// is what an unreadable tracked path looks like from inside the read.
#[test]
fn a_tracked_document_the_sweep_cannot_read_is_a_cannot_judge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let declared = vec![Census {
        subject: "a control",
        phrase: "{} bounds across {} capabilities",
        figures: vec![1, 1],
    }];
    let offences = sweep(
        &root,
        &["zzz_absent_census_probe.md".to_string()],
        &declared,
    )
    .offences;
    assert_eq!(
        offences.len(),
        1,
        "an unreadable tracked document must produce exactly one refusal, got {offences:?}"
    );
    refusal::expect("repository-checks#census-document-unreadable", &offences[0]);
    assert_eq!(
        offences[0].kind,
        Kind::CannotJudge,
        "an unread document is not a document without a census, so it is not a violation"
    );
    assert!(
        offences[0].message.contains("zzz_absent_census_probe.md"),
        "the refusal must name the document it could not read, got {:?}",
        offences[0].message
    );
}

/// A figure this sweep cannot represent is reported as **cannot judge**, not read as no figure at all.
///
/// **`parse().ok()?` spelled *unreadable* the same as *absent*.** A document writing a count past `usize` was
/// compared against nothing, so the sweep reported clean over exactly the sentence it exists for — the
/// conflation `reading`'s module doc names as the one bug this repository forbids, in the module whose whole
/// subject is a declared figure disagreeing with a produced one. Shown rather than asserted about: the tree
/// carries no such figure, so this writes one.
#[test]
fn a_figure_the_sweep_cannot_represent_is_a_cannot_judge() {
    let declared = vec![Census {
        subject: "a control",
        phrase: "{} bounds across {} capabilities",
        figures: vec![1, 1],
    }];
    let scratch = std::env::temp_dir().join(format!("tianheng-census-wide-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
    std::fs::write(
        scratch.join("wide.md"),
        "  a line writing 99999999999999999999999999 bounds across 1 capabilities
",
    )
    .expect("write");
    let offences = sweep(&scratch, &["wide.md".to_string()], &declared).offences;
    // The control, in the same directory: the phrase and the sweep are both alive, so the refusal above is
    // about the width of the figure and not about either of them.
    std::fs::write(
        scratch.join("narrow.md"),
        "  a line writing 2 bounds across 1 capabilities
",
    )
    .expect("write");
    let control = sweep(&scratch, &["narrow.md".to_string()], &declared).offences;
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(
        offences.len(),
        1,
        "a figure past `usize` must produce exactly one refusal, got {offences:?}"
    );
    refusal::expect("repository-checks#census-figure-unreadable", &offences[0]);
    assert_eq!(
        offences[0].kind,
        Kind::CannotJudge,
        "a figure this sweep cannot hold is a fact about the sweep, not a document disagreeing with it"
    );
    assert!(
        offences[0].message.contains("99999999999999999999999999"),
        "the refusal must quote the run so the sentence can be found, got {:?}",
        offences[0].message
    );
    assert_eq!(
        control.len(),
        1,
        "the control must disagree, or the refusal above could be the phrase matching nothing: {control:?}"
    );
    refusal::expect("repository-checks#census-figure-disagrees", &control[0]);
}

/// The sweep must be able to see a disagreement, or its silence says nothing.
///
/// Every census here asserts agreement, and agreement has more than one cause: a phrase nothing matches is
/// silent for a reason that has nothing to do with the figures. This runs one census whose figures are
/// deliberately wrong and requires the sweep to name it.
///
/// **Shown against a written document rather than against the tree, because the tree states no census.** This
/// control used to sweep the real workspace with figures far from any real one, which worked only while a
/// governance document carried the phrase. That document's figure was the last hand-written count in the
/// repository and it is gone: the rule is now that a count of a live set is not written at all, so a control
/// that needs one would be asking the tree to keep the very thing the rule removed. The subject is written
/// here instead — which is what every other direction in this file already does, and what makes this one
/// independent of whether any document happens to state a census.
#[test]
fn the_sweep_names_a_disagreement_it_is_shown() {
    let declared = vec![Census {
        subject: "a control",
        phrase: "{} bounds across {} capabilities",
        figures: vec![2, 1],
    }];
    let scratch =
        std::env::temp_dir().join(format!("tianheng-census-control-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
    // A document stating the phrase with figures that disagree, and beside it one that agrees. Without the
    // second, the refusal could be the phrase matching nothing rather than the figures differing.
    std::fs::write(
        scratch.join("disagrees.md"),
        "  a line writing 7 bounds across 1 capabilities\n",
    )
    .expect("write");
    std::fs::write(
        scratch.join("agrees.md"),
        "  a line writing 2 bounds across 1 capabilities\n",
    )
    .expect("write");
    let offences = sweep(&scratch, &["disagrees.md".to_string()], &declared).offences;
    let control = sweep(&scratch, &["agrees.md".to_string()], &declared).offences;
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        control.is_empty(),
        "the agreeing document must be silent, or the refusal below is about the phrase rather than the \
         figures: {control:?}"
    );
    assert!(
        !offences.is_empty(),
        "the sweep found no disagreement against figures that are deliberately wrong, so its agreement \
         elsewhere is silence rather than a verdict"
    );
    refusal::expect("repository-checks#census-figure-disagrees", &offences[0]);
}

/// `repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The declaration is the coverage: a figure written in a phrasing no
/// census names is unheld. Reaching it needs a judgement over prose, the instrument this repository designed,
/// measured three times and rejected — and `AGENTS.md` carries the other half as a rule with no check.
#[test]
fn a_count_in_an_undeclared_phrasing_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let bounds = parse_bounds(&root);
    let capabilities: BTreeSet<&str> = bounds.iter().map(|b| b.capability.as_str()).collect();
    let declared = vec![Census {
        subject: "declared observation bounds and the capabilities declaring them",
        phrase: "{} bounds across {} capabilities",
        figures: vec![bounds.len(), capabilities.len()],
    }];

    // The control: the sweep is alive, and reads this document.
    let control = format!(
        "  a line writing {} bounds across {} capabilities\n",
        bounds.len() + 1,
        capabilities.len()
    );
    let scratch = std::env::temp_dir().join(format!("tianheng-census-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
    std::fs::write(scratch.join("control.md"), &control).expect("write");
    assert!(
        !sweep(&scratch, &["control.md".to_string()], &declared)
            .offences
            .is_empty(),
        "the sweep must see a disagreement it is shown, or the silence below says nothing"
    );

    // The bound: the same count, in a sentence no census declares.
    std::fs::write(
        scratch.join("undeclared.md"),
        format!(
            "  the register holds {} declared limits over {} subjects\n",
            bounds.len() + 1,
            capabilities.len()
        ),
    )
    .expect("write");
    let offences = sweep(&scratch, &["undeclared.md".to_string()], &declared).offences;
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        offences.is_empty(),
        "the sweep must stay silent about a count written in an undeclared phrasing — that is the declared \
         bound, and reaching it needs the prose detector this repository rejected. Got: {offences:?}"
    );
}

/// `repository-checks/a-figure-written-in-words-at-one-hundred-or-above-is-not-matched-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. `number_at` reads the units, the tens, and one compound of the two,
/// which stops at ninety-nine. Both directions on one body, differing only in how the figure is **spelled**:
/// a bound whose silence is not contrasted with a reaction on the same shape is indistinguishable from a
/// sweep that reads nothing.
///
/// Declared with fixed figures rather than the register's own, because the property under test is the word
/// reader's reach and not the size of any set this repository happens to hold today.
#[test]
fn a_word_form_at_one_hundred_or_above_is_a_stated_bound() {
    let declared = vec![Census {
        subject: "a control",
        phrase: "{} bounds across {} capabilities",
        figures: vec![99, 24],
    }];
    let scratch =
        std::env::temp_dir().join(format!("tianheng-census-words-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    // The control: a disagreeing figure written in words the reader reaches.
    std::fs::write(
        scratch.join("below.md"),
        "  a line writing ninety-eight bounds across 24 capabilities\n",
    )
    .expect("write");
    assert!(
        !sweep(&scratch, &["below.md".to_string()], &declared)
            .offences
            .is_empty(),
        "the sweep must see a disagreement spelled in words below one hundred, or the silence below says \
         nothing about the ceiling"
    );

    // The bound: the same disagreement, spelled at one hundred.
    std::fs::write(
        scratch.join("above.md"),
        "  a line writing one hundred bounds across 24 capabilities\n",
    )
    .expect("write");
    let offences = sweep(&scratch, &["above.md".to_string()], &declared).offences;
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        offences.is_empty(),
        "the sweep must stay silent about a figure spelled at one hundred or above — that is the declared \
         bound, and a word reader that silently stops matching is why it is stated rather than left to be \
         discovered. Got: {offences:?}"
    );
}

/// `repository-checks/a-census-written-outside-markdown-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The corpus is tracked Markdown, and the narrowing was measured rather
/// than reasoned about: this repository's Rust sources carry census phrases **as fixture input** — the figures
/// above in this very file are a parser's expected output and deliberately arbitrary — so admitting `.rs` would
/// report a test asserting its own parser as a drifted document. Both directions on one body, differing only in
/// the extension, because a bound whose silence is not contrasted with a reaction is indistinguishable from a
/// sweep that reads nothing.
#[test]
fn a_census_outside_markdown_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let bounds = parse_bounds(&root);
    let capabilities: BTreeSet<&str> = bounds.iter().map(|b| b.capability.as_str()).collect();
    let declared = vec![Census {
        subject: "declared observation bounds and the capabilities declaring them",
        phrase: "{} bounds across {} capabilities",
        figures: vec![bounds.len(), capabilities.len()],
    }];
    let wrong = format!(
        "  a line writing {} bounds across {} capabilities\n",
        bounds.len() + 1,
        capabilities.len()
    );

    let scratch =
        std::env::temp_dir().join(format!("tianheng-census-corpus-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
    std::fs::write(scratch.join("held.md"), &wrong).expect("write the Markdown control");
    std::fs::write(scratch.join("unheld.rs"), &wrong).expect("write the Rust subject");

    // The control: the same figures, in Markdown, are reported. Without it the silence below is satisfiable by
    // a sweep that reads nothing at all.
    assert!(
        !sweep(&scratch, &["held.md".to_string()], &declared)
            .offences
            .is_empty(),
        "the sweep must report a disagreement it is shown in Markdown, or the bound below proves nothing"
    );
    // The bound: the same figures, in a Rust source, are not.
    let unheld = sweep(&scratch, &["unheld.rs".to_string()], &declared).offences;
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        unheld.is_empty(),
        "the corpus is tracked Markdown, so a census outside it is a stated bound rather than a finding, got \
         {unheld:?}"
    );
}
