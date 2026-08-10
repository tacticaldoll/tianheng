//! Observation bounds declared by Kanhe-owned repository checks.
//!
//! These declarations are consumed by this repository's bound-model gate. They are not part of the published
//! Tianheng catalog: the checks they qualify live in Kanhe and ship in no package.

use tianheng::{BoundDecl, BoundId, Extent, Owner, Reached};

pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        BoundDecl::unpinned(
            BoundId::new(
                "rust-repository-reactions/a-gate-reached-without-the-wrapper-a-stated-bound",
            ),
            "an act reaching cargo publish or a merge without going through its wrapper",
            Extent::Reached(Reached::UnderReacts {
                because: "both assertions guard the sanctioned path -- the wrapper requiring its gate to \
                          report one passing test, and the reaction pinning the identifier it cites. \
                          Reaching further would mean observing the operator's shell or GitHub's servers \
                          rather than this repository"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*",
        ),
        BoundDecl::pinned(
            BoundId::new("rust-repository-reactions/files-no-capability-claims-a-stated-bound"),
            "a tracked file no capability's declared subject claims",
            Extent::Reached(Reached::UnderReacts {
                because: "subjects are declared where a capability has something to say, and requiring them \
                          to tile the repository would buy coverage with a claim per capability that nobody \
                          could defend. The join reports how many tracked paths went unclaimed, so a clean \
                          verdict is not read as a complete one"
                    .into(),
                owner: Owner::Engine,
            }),
            "files_no_capability_claims_are_reported_rather_than_implied_judged",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "rust-repository-reactions/a-count-written-in-a-sentence-no-census-declares-a-stated-bound",
            ),
            "a figure about an enumerable set, written in a phrasing no census declares",
            Extent::Reached(Reached::UnderReacts {
                because: "the declaration is the coverage — a census names the one sentence its figures are \
                          written in, and a count outside that sentence is unheld. Reaching it needs a \
                          judgement over prose, the instrument this repository designed, measured three times \
                          and rejected; `AGENTS.md` carries the other half as a rule with no reaction"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_count_in_an_undeclared_phrasing_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "rust-repository-reactions/a-hook-is-proposed-for-this-rule-a-stated-bound",
            ),
            "a squash merge made anywhere but through the sanctioned wrapper",
            Extent::OutOfReach {
                because: "a squash merge runs on GitHub's servers, so no local commit exists and no hook \
                          runs, and both values of the repository's squash-title setting append the serial; \
                          the reaction guards the sanctioned path to a merge, and a browser reaches no \
                          wrapper"
                    .into(),
            },
            "a_merge_made_outside_the_wrapper_is_not_observed",
        ),
    ]
}
