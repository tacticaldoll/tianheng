//! Self-governance reaction: every declared census agrees with the reaction that produces it.
//!
//! Each census below is **produced** by the enumeration it is about — the register's own parse — never by a
//! second reading of the same source. A second parse would let the census and
//! the reaction disagree, which is the drift this exists to end.

#[path = "support/census.rs"]
mod census;
#[path = "support/refusal_exemptions.rs"]
mod exemptions;
#[path = "support/refusal.rs"]
mod refusal;
#[path = "support/bound_register_parse.rs"]
mod register;
#[path = "support/refusal_sites.rs"]
mod sites;

use census::{Census, sweep};
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
    let corpus = sites::build(&root);
    assert!(
        corpus.offences.is_empty(),
        "the refusal-site enumeration this census is produced from is not sound:\n{}",
        corpus.offences.join("\n")
    );
    let refusal_sites = corpus.sites;
    let out_of_reach = refusal_sites
        .iter()
        .filter(|s| s.declares_out_of_reach())
        .count();

    // What is NOT declared here, and why. `[Unreleased]`'s prose states how many entries named this
    // repository's machinery BEFORE the section was collapsed — a historical observation, not a live count,
    // and the reaction that enumerates the set today produces a different and equally correct figure. A
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
        Census {
            subject: "refusal sites declared out of reach",
            phrase: "{} of {} refusal sites are declared out of reach",
            // Produced by the same enumeration `refusal_bites` perturbs, not by a second reading of the same
            // source: a census and the reaction it is about must not be able to disagree.
            figures: vec![out_of_reach, refusal_sites.len()],
        },
    ];

    let offences = sweep(&root, &files, &declared);
    assert!(
        offences.is_empty(),
        "a hand-written census disagrees with the reaction that enumerates its set:\n{}",
        offences.join("\n")
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

/// `rust-self-governance-gates/a-count-written-in-a-sentence-no-census-declares-a-stated-bound`
///
/// `UnderReacts`, owned by the engine. The declaration is the coverage: a figure written in a phrasing no
/// census names is unheld. Reaching it needs a judgement over prose, the instrument this repository designed,
/// measured three times and rejected — and `AGENTS.md` carries the other half as a rule with no reaction.
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
