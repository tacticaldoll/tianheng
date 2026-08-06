//! The shell's own declared bounds — `observation-bound-model`'s, `observer-protocol`'s and
//! `gate-shape-contract`'s, typed in the model they describe.
//!
//! The capability that classifies where every other reaction stops must classify where **it** stops, or its
//! leading figure — the count of declared false negatives — would be a number the counter had exempted itself
//! from. Its reaction lives in this crate (`tests/observation_bound_model.rs`, the only place that sees all
//! three dimensions), so this crate owns the declarations.

use crate::{BoundDecl, BoundId, Extent, FactGranularity, Owner, Reached};

/// Every observation bound `observation-bound-model` declares.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        BoundDecl::new(
            BoundId::new(
                "observation-bound-model/whether-a-declaration-s-stated-cause-is-the-real-cause-is-not-observed-a-stated-bound",
            ),
            "a declaration whose rationale names a cause that is not why the reaction stops",
            Extent::OutOfReach {
                because: "the extent is typed and checkable while the rationale is prose the model never \
                          reads; requiring the two to agree would trade a fact for a heuristic".into(),
            },
            "a_rationale_that_contradicts_its_extent_is_a_stated_bound",
        ),
        BoundDecl::new(
            BoundId::new(
                "observation-bound-model/an-answer-that-depends-on-the-corpus-entry-point-has-no-extent-of-its-own-a-stated-bound",
            ),
            "a bound whose outcome differs by which corpus entry point observed it",
            Extent::Reached(Reached::AsIntended {
                bounded: FactGranularity::Identity,
                // The classification is right — the direction that matters, a seam reported covered when it is
                // not, is recorded either way — but two distinct situations share one value, which is an
                // identity granularity limit rather than a limit on the classification's correctness.
                because: "it is recorded as an under-reaction owned by the entry point rather than carrying a \
                          value of its own, so it shares that value with bounds whose answer does not depend \
                          on an entry point; one live instance does not earn a value every other member has \
                          several of".into(),
            }),
            "an_entry_dependent_bound_is_declared_as_under_reacting",
        ),
        BoundDecl::new(
            BoundId::new(
                "observation-bound-model/a-bound-both-out-of-reach-and-granularity-limited-cannot-be-expressed-a-stated-bound",
            ),
            "a bound both invisible to the observation source and limited in the granularity of the fact it \
             would have produced",
            Extent::OutOfReach {
                because: "granularity is carried only by the as-intended extent, so the pair has no \
                          representation at all; no declared bound exhibits it, and offering granularity on \
                          every extent would invite a combination nothing shows while weakening the nesting \
                          that makes a contradiction unwritable".into(),
            },
            "granularity_is_carried_only_by_the_as_intended_extent",
        ),
        BoundDecl::new(
            BoundId::new(
                "observer-protocol/whether-an-observer-s-declared-bounds-are-complete-is-not-observed-a-stated-bound",
            ),
            "an observer that declares some of its limits and omits others",
            Extent::OutOfReach {
                because: "the trait compels a declaration and never a complete one; no reaction can enumerate \
                          the limits of a reaction it did not write, so an omission is invisible".into(),
            },
            "an_observer_may_under_declare_its_bounds",
        ),
        BoundDecl::new(
            BoundId::new(
                "observer-protocol/whether-an-observer-s-own-verdict-is-correct-is-not-observed-a-stated-bound",
            ),
            "a composed observer returning an outcome that misjudges the workspace it read",
            Extent::Reached(Reached::UnderReacts {
                because: "the fold composes verdicts and does not adjudicate them; second-guessing each \
                          participant would need a second implementation of every dimension".into(),
                owner: Owner::Adopter,
            }),
            "the_fold_does_not_adjudicate_a_participant_s_verdict",
        ),
        BoundDecl::new(
            BoundId::new(
                "observer-protocol/a-trait-object-on-a-wrapped-signature-s-continuation-line-is-not-seen-a-stated-bound",
            ),
            "a public signature spanning several lines that names a trait object on a line not beginning with \
             `pub `",
            Extent::OutOfReach {
                // Out of reach rather than under-reacting: the recognizer is handed one line at a time, so the
                // continuation is never a candidate it declined — it is text the observation never presents.
                because: "the reaction reads this crate lexically, one line at a time, because 渾儀 governs no \
                          module of it and the `dyn`-trait DSL offers only forbid-all and forbid-named-operands, \
                          so a declared exposure would be a name with no reaction".into(),
            },
            "a_trait_object_on_a_continuation_line_is_not_recognized",
        ),
        // --- gate-shape-contract ---
        //
        // Its reaction is `tests/gate_shape_contract.rs`, so this crate owns these too. The four are read out
        // of each scenario's WHEN and THEN rather than out of a shared adjective: three name shapes the
        // reaction never looks at, and one names a shape it looks straight at and declines to judge.
        BoundDecl::new(
            BoundId::new(
                "gate-shape-contract/whether-an-enumeration-carries-a-vacuity-guard-is-not-observed-a-stated-bound",
            ),
            "a gate iterating an enumeration with no guard against zero iterations",
            Extent::OutOfReach {
                because: "the reaction reads a gate's text for the properties it requires and models none of \
                          its control flow, so a loop that iterates nothing and reports clean is not a shape \
                          it examines".into(),
            },
            "a_missing_vacuity_guard_is_a_stated_semantic_bound",
        ),
        BoundDecl::new(
            BoundId::new(
                "gate-shape-contract/whether-a-read-s-status-is-checked-in-the-parent-shell-is-not-observed-a-stated-bound",
            ),
            "a gate reading a command's output through a command substitution whose status nobody inspects, \
             or through a pipeline whose non-final stage fails",
            Extent::OutOfReach {
                // NARROWED, not rewritten: the process-substitution shape IS now observed, by the
                // one-checked-capture property. What remains needs control flow rather than text — whether a
                // caller inspects `$?` after a `$(…)` is not a property of the source. The heading and therefore
                // the id are untouched, because renaming would break this declaration and the citation in one
                // edit for a reason unrelated to the bound's content.
                because: "the process-substitution construct is now refused by its own property, so what is left \
                          is a status swallowed where detecting it would mean modelling whether the caller reads \
                          `$?` afterwards; the backstop the reaction also requires narrows the damage without \
                          detecting either shape".into(),
            },
            "an_unchecked_read_status_is_a_stated_semantic_bound",
        ),
        BoundDecl::new(
            BoundId::new(
                "gate-shape-contract/whether-a-gate-s-1-versus-2-assignment-is-correct-is-not-observed-a-stated-bound",
            ),
            "a gate reporting a genuine violation as cannot-judge, or a misconfiguration as a violation",
            Extent::Reached(Reached::UnderReacts {
                because: "the reaction requires the twin to assert an expected code and declines to judge \
                          whether the code the gate assigned is the right one; that judgment is what let a \
                          `fail` returning instead of exiting turn every violation into cannot-judge and ride \
                          green".into(),
                // The engine's own, not an adopter's: this reaction sees the codes and stops there, and no
                // declaration by an adopter would change what it does with them.
                owner: Owner::Engine,
            }),
            "a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound",
        ),
        BoundDecl::new(
            BoundId::new(
                "gate-shape-contract/shell-units-that-are-not-a-gate-or-its-twin-are-outside-the-surface-a-stated-bound",
            ),
            "a sourced function library, a matrix over one, the example runner, or the publish tool",
            Extent::OutOfReach {
                because: "the enumeration is the `check_*` gate and the twin its basename names, so no other \
                          shell unit is judged on any of the properties it holds them to; the one thing asserted about \
                          them is that none carries the shared exit contract, which keeps the exclusion from \
                          being a hiding place rather than making it coverage".into(),
            },
            "units_outside_the_gate_pairing_are_outside_the_surface",
        ),
        // --- publish-source-integrity ---
        //
        // Its reaction is a shell gate, and `PINNED-BY` resolves only a harness-registered Rust function — so
        // its citation is `tests/publish_source_integrity.rs`, a file that exists for this one bound. The shell
        // gate defends it too, and cannot be cited.
        BoundDecl::new(
            BoundId::new(
                "publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound",
            ),
            "a release tag carrying a cryptographically valid signature made by a key no maintainer authorized",
            Extent::Reached(Reached::UnderReacts {
                because: "validity is verifiable with no configuration and attribution is not — it needs an \
                          allowed-signers file that exists on a maintainer's machine and not in CI, so \
                          requiring it would make the same tag judged differently by where the gate ran"
                    .into(),
                // The layer is the verification environment, not this engine: no change to the gate closes
                // this, because the missing input is a configuration rather than a check. Giving CI an
                // allowed-signers file is what would — a repository decision, so naming the environment is
                // what makes the owner actionable.
                owner: Owner::Inherited {
                    from: "the verification environment".into(),
                },
            }),
            "a_valid_signature_from_an_unauthorized_key_is_accepted",
        ),
        // --- projection-register ---
        //
        // Its reaction is `tests/projection_register.rs`, so this crate owns these too. The two sit on opposite
        // sides of the false-negative line, which is exactly what the retired adjective slot could not express:
        // one is a shape the reaction never evaluates, the other a shape it can read and does not react to.
        BoundDecl::new(
            BoundId::new(
                "projection-register/whether-a-stated-regeneration-command-regenerates-its-document-is-not-observed-a-stated-bound",
            ),
            "a generated document whose header names a command that no longer regenerates it",
            Extent::OutOfReach {
                because: "the header is read and never evaluated; running the command would mean re-entering the \
                          `cargo test` harness already running, or — for the shell mechanism — writing the \
                          projection into the tree the reaction is judging, which every gate in this family is \
                          forbidden from doing".into(),
            },
            "a_regeneration_command_is_registered_and_never_run",
        ),
        BoundDecl::new(
            BoundId::new(
                "projection-register/a-document-generated-by-an-unrecognized-mechanism-is-not-observed-a-stated-bound",
            ),
            "a document generated by neither the shared Rust rule nor a `check_*` gate under `BLESS`, whose \
             author also omitted the marker",
            Extent::Reached(Reached::UnderReacts {
                because: "it is absent from both sides of the correspondence, so that correspondence holds over \
                          a surface missing a member and the register reports itself complete".into(),
                // Not out of reach: the third mechanism's source sits in the tree this reaction already reads,
                // so it is seen and not reacted to. Recording it as out-of-reach would be the misclassification
                // this model exists to prevent — a silent false negative dressed as an invisible shape.
                owner: Owner::Engine,
            }),
            "a_third_generation_mechanism_is_not_recognized",
        ),
    ]
}
