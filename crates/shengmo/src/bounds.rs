//! Observation bounds declared by Shengmo's self-governance dogfood and declaration checks.
//!
//! The declarations qualify this repository's dogfood and are consumed only by its combined bound-model gate.
//! Shengmo ships in no package, so they do not enter the published Tianheng surface.

use tianheng::{BoundDecl, BoundId, Extent, Owner, Reached};

pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        // --- self-law-projection ---
        BoundDecl::pinned(
            BoundId::new(
                "self-law-projection/a-doc-example-of-the-dependency-dsl-is-refused-a-stated-bound",
            ),
            "a line comment under the shell naming `restrict_dependencies_to(` in order to teach the \
             re-exported DSL",
            Extent::Reached(Reached::OverReacts {
                because: "the recognizer reads a comment's text and never its purpose, so a doc example of a \
                          DSL the shell publishes is refused exactly as a restatement of its own declaration \
                          would be".into(),
            }),
            "a_doc_example_of_the_dependency_dsl_is_refused",
        ),
        // --- self-law-projection: the mutual-independence check's limits ---
        //
        // Each extent is read off a run of that limit's own WHEN, never off the argument for it: a draft
        // declared the first as a false NEGATIVE and one run showed the check fires there.
        BoundDecl::unpinned(
            BoundId::new(
                "self-law-projection/a-reason-that-paraphrases-the-law-is-refused-a-stated-bound",
            ),
            "a dimension's `because` stating the mutual-independence law in different words, without the literal clause",
            Extent::Reached(Reached::OverReacts {
                because: "the check reads the `because` for the literal clause, so a reason that genuinely \
                          states the law in other words is refused; the direction is the safe one and closing \
                          it needs the check to decide two wordings state one law"
                    .into(),
            }),
            "`BACKLOG.md` — *four limits of the mutual-independence check*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "self-law-projection/a-reason-carrying-the-clause-while-negating-the-law-is-not-observed-a-stated-bound",
            ),
            "a `because` quoting 三儀 ⊥ 三儀 and then stating that it does not bind that dimension",
            Extent::Reached(Reached::UnderReacts {
                because: "the check looks for the clause and not for what the sentence does with it, so the \
                          agent-loaded projection can carry the law's opposite while satisfying the check that \
                          exists to keep the law taught"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *four limits of the mutual-independence check*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "self-law-projection/a-workspace-dependency-allowlist-is-not-examined-a-stated-bound",
            ),
            "a dimension declaring the law through `restrict_workspace_dependencies_to` instead",
            Extent::Reached(Reached::UnderReacts {
                because: "the filter admits one rule variant, while the variant it omits governs \
                          workspace-member edges specifically and is the more natural one for this law"
                    .into(),
            owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *four limits of the mutual-independence check*",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "self-law-projection/a-comment-naming-every-member-for-another-reason-is-refused-a-stated-bound",
            ),
            "one contiguous line-comment block naming every current allowlist member for a purpose other than \
             copying the declaration",
            Extent::Reached(Reached::OverReacts {
                because: "the block check asks whether the members all appear and never why, and teaching it \
                          to read intent would be a heuristic over prose".into(),
            }),
            "a_comment_naming_every_member_for_another_reason_is_refused",
        ),
    ]
}
