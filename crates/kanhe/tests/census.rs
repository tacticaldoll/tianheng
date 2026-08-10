//! Repository check: every declared census agrees with the check that produces it.
//!
//! Each census below is **produced** by the enumeration it is about — the register's own parse — never by a
//! second reading of the same source. A second parse would let the census and
//! the check disagree, which is the drift this exists to end.

use kanhe::bound_register_parse as register;
use kanhe::census;

use census::{Census, sweep};
use kanhe::refusal::Kind;
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
    // What is NOT declared here, and why. `[Unreleased]`'s prose states how many entries named this
    // repository's machinery BEFORE the section was collapsed — a historical observation, not a live count,
    // and the check that enumerates the set today produces a different and equally correct figure. A
    // census holds a figure about the CURRENT tree; a figure about a past state is a record, and holding it
    // to today's enumeration would demand that the record change every time the tree does. That residual is
    // declared as a bound rather than approximated.
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

    let offences = sweep(&root, &files, &declared);
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
    );
    assert_eq!(
        offences.len(),
        1,
        "an unreadable tracked document must produce exactly one refusal, got {offences:?}"
    );
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

/// The sweep must be able to see a disagreement, or its silence says nothing.
///
/// Every census here asserts agreement, and agreement has more than one cause: a phrase nothing matches is
/// silent for a reason that has nothing to do with the figures. This runs one census whose figures are
/// deliberately wrong and requires the sweep to name it.
#[test]
fn the_sweep_names_a_disagreement_it_is_shown() {
    let Some(root) = workspace_root() else {
        return;
    };
    let files = tracked(&root);
    let bounds = parse_bounds(&root);
    let capabilities: BTreeSet<&str> = bounds.iter().map(|b| b.capability.as_str()).collect();

    let wrong = vec![Census {
        subject: "a control",
        phrase: "{} bounds across {} capabilities",
        // Far enough from any real figure that no document can accidentally agree with it. `len() + 1` was
        // the first draft, and editing `BACKLOG.md` to that very number disabled this control.
        figures: vec![bounds.len() + 10_000, capabilities.len()],
    }];
    let offences = sweep(&root, &files, &wrong);
    assert!(
        !offences.is_empty(),
        "the sweep found no disagreement against figures that are deliberately wrong, so its agreement above \
         is silence rather than a verdict"
    );
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
    std::fs::create_dir_all(&scratch).expect("the scratch root is writable");
    std::fs::write(scratch.join("control.md"), &control).expect("write");
    assert!(
        !sweep(&scratch, &["control.md".to_string()], &declared).is_empty(),
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
    let offences = sweep(&scratch, &["undeclared.md".to_string()], &declared);
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        offences.is_empty(),
        "the sweep must stay silent about a count written in an undeclared phrasing — that is the declared \
         bound, and reaching it needs the prose detector this repository rejected. Got: {offences:?}"
    );
}
