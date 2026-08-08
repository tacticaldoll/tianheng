//! The shell's own declared bounds: every capability whose reaction lives in this crate, typed in the model
//! those reactions describe. Which capabilities those are is not listed here — the set is read from the
//! declarations below by `observation-bound-model`'s bijection, and a second copy in prose would be the
//! hand-typed census this family refuses.
//!
//! The capability that classifies where every other reaction stops must classify where **it** stops, or its
//! leading figure — the count of declared false negatives — would be a number the counter had exempted itself
//! from. Its reaction lives in this crate (`tests/observation_bound_model.rs`, the only place that sees all
//! three dimensions), so this crate owns the declarations.

use crate::{BoundDecl, BoundId, Extent, FactGranularity, Owner, Reached};

/// Every observation bound declared by a capability whose reaction lives in this crate.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        // Which side of the false-negative line a moved extent falls on is decided by the comparison reading
        // it, not by the extent. An exact one-statement equality cannot survive one and therefore over-reacts,
        // which is this bound. A second reader over the shell's composition body compared by count and
        // containment, which a truncated remainder satisfies, and was retired rather than narrowed a fifth
        // time; the distinction is kept here so the direction is not read as a property of the extent itself.
        BoundDecl::pinned(
            BoundId::new(
                "observer-protocol/a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent-a-stated-bound",
            ),
            "an inspected bounds-method body carrying `{` or `}` inside a block comment or a string literal",
            // Over-reacting rather than under-reacting, and that is read off this comparison rather than
            // preferred: the body is required to be one exact statement, which no brace-carrying construct
            // survives, so a moved extent refuses a conforming body instead of admitting a divergent one.
            Extent::Reached(Reached::OverReacts {
                because: "the extent is found by counting braces outside line comments only, and separating a \
                          brace in code from one inside a string literal needs the lexing this tree's own \
                          lexer suites defeat, their fixtures putting comment delimiters inside string \
                          literals".into(),
            }),
            "a_brace_in_a_block_comment_moves_the_body_extent",
        ),
        // The reaction that read the composition body is retired, so what is declared is the obligation being
        // unobserved rather than one family of escapes from a reader that no longer exists. Under-reacting with
        // the engine as owner, not out of reach: the deciding text was inside the file the reader loaded, so the
        // measure stopped where this repository chose to stop it, and closing it is ordinary work here.
        BoundDecl::unpinned(
            BoundId::new(
                "observer-protocol/whether-the-shell-makes-an-independent-semantic-decision-is-not-observed-a-stated-bound",
            ),
            "the shell's composition arm deciding semantic emptiness itself instead of leaving it to the observer it invokes",
            Extent::Reached(Reached::UnderReacts {
                because: "a text reader over the composition body was defeated at every level it could be \
                          narrowed to — name resolution, the parameter's binding site, the identity of the \
                          definition, the caller frame, and execution, which no reading of text reaches — so \
                          invoking the observer made the two paths' EQUALITY construction-held and left this untouched, measured: a \
                          guard above that call compiles and passes every gate".into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *the shell's semantic delegation, held by construction*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound",
            ),
            "a whole-line signature copy — commented, in a string literal, or otherwise — with the definition moved out of the inspected source",
            Extent::Reached(Reached::UnderReacts {
                because: "the reader knows nothing of comments or literals, so one whole-line occurrence \
                          anchors whatever follows it; what passes is a second hand-maintained path that \
                          agrees today, since a divergent one is caught by observation-bound-model's \
                          bijection over Observer::bounds — measured both ways"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *the bounds-method reader anchors on a whole-line occurrence that is not the definition*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observer-protocol/whether-the-stated-construction-held-list-matches-the-composition-path-is-not-observed-a-stated-bound",
            ),
            "the requirement's list of construction-held dimensions naming a different set than the built-in path invokes",
            Extent::Reached(Reached::UnderReacts {
                because: "the list is hand-maintained prose about a set the code enumerates, and falsifying \
                          it passes the whole suite and every gate; deciding it needs a perturbed build \
                          rather than a read, because the discriminator is which assertion fails when a \
                          dimension's observer is emptied"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *the construction-held list is hand-maintained prose*",
        ),
        // --- observation-bound-register ---
        //
        // The register's own bounds — the only ones this crate declares about the reaction
        // that produces the register rather than about a dimension. `check_pin_bites.sh` decides that a
        // citation's pin *bites* only where a mutation is declared for it; where none is, nothing decides.
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/what-code-executed-inside-the-checkout-does-outside-it-is-not-observed-a-stated-bound",
            ),
            "code run inside the checkout writing outside it, or replacing a checked path so the reaction's own write lands elsewhere",
            Extent::Reached(Reached::UnderReacts {
                because: "running the cited test is the whole method, so code execution inside the checkout \
                          is granted unconditionally; the shared common directory is what makes a \
                          git-reading citation reachable at all, and re-checking a resolved path after the \
                          build would re-check the window that defeated it"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *most pinning citations have never been seen to fail*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/whether-a-cited-test-s-outcome-depends-on-its-run-count-is-not-observed-beyond-one-period-a-stated-bound",
            ),
            "a cited test passing and failing by a period the fixed run sequence does not break",
            Extent::Reached(Reached::UnderReacts {
                because: "the reaction runs the test a fixed number of times and the number is readable in \
                          its own source, so a matching period escapes; closing it needs each run unable to \
                          observe how many times the test has run, whose cost grows with the coverage this \
                          capability exists to grow"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *most pinning citations have never been seen to fail*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/whether-a-pin-gutted-but-not-committed-still-bites-is-not-observed-a-stated-bound",
            ),
            "a cited pin whose assertions are removed in the working directory and not committed",
            Extent::Reached(Reached::UnderReacts {
                because: "the checkout under test is HEAD's content, because mutating the author's own \
                          checkout is what a separate checkout exists to avoid; the two properties are in \
                          tension and this one is given up deliberately"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *most pinning citations have never been seen to fail*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/whether-a-record-perturbs-the-reaction-or-the-pin-s-own-assertions-is-not-observed-a-stated-bound",
            ),
            "a record naming the file its pin lives in and neutralising one of that pin's assertions",
            Extent::Reached(Reached::UnderReacts {
                because: "a killed pin does not say what killed it, and refusing a record that edits its \
                          pin's own file would refuse this tree's first seeded record, which legitimately \
                          perturbs a recognizer sitting beside the pin that defends it"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *most pinning citations have never been seen to fail*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/whether-a-citation-carrying-no-declared-mutation-is-defended-is-not-observed-a-stated-bound",
            ),
            "a pinning citation for which no mutation is declared",
            Extent::Reached(Reached::UnderReacts {
                because: "the gate runs the mutations it is given and nothing else, so a citation with no \
                          record is neither exercised nor refused; authoring a record that genuinely perturbs \
                          the pinned point is per-bound work, which is why coverage is disclosed on every \
                          clean run rather than implied"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *most pinning citations have never been seen to fail*",
        ),
        // --- gate-shape-contract ---
        //
        // Its reaction is `tests/gate_shape_contract.rs`, so this crate owns these too. The four are read out
        // of each scenario's WHEN and THEN rather than out of a shared adjective: three name shapes the
        // reaction never looks at, and one names a shape it looks straight at and declines to judge.
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
            BoundId::new(
                "gate-shape-contract/whether-a-gate-s-1-versus-2-assignment-is-correct-is-not-observed-a-stated-bound",
            ),
            "a gate reporting a genuine violation as cannot-judge, or a misconfiguration as a violation",
            Extent::Reached(Reached::UnderReacts {
                // Narrowed with the scenario, and this is the third site the same claim was written at — the
                // spec's THEN, the projection derived from it, and here. It used to say this judgment is what
                // let a `fail` returning instead of exiting ride green; what let that ride green was the
                // matrix asserting a non-zero status rather than a code, which the `exit codes` property now
                // refuses.
                because: "the reaction requires the twin to assert an expected code and declines to judge \
                          whether the code the gate assigned is the right one, which needs each gate's meaning \
                          rather than its text".into(),
                // The engine's own, not an adopter's: this reaction sees the codes and stops there, and no
                // declaration by an adopter would change what it does with them.
                owner: Owner::Engine,
            }),
            "a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "gate-shape-contract/a-permitted-builtin-piped-into-an-external-command-is-still-permitted-a-stated-bound",
            ),
            "a gate's process substitution beginning with `printf` or `echo` and piped into an external command",
            Extent::Reached(Reached::UnderReacts {
                // The producer's text IS handed to the recognizer, which stops at its first word — so this is an
                // under-reaction rather than a shape out of reach. Owned by the engine because closing it is this
                // reaction's own to do; what makes it not ordinary work is that refusing a producer containing a
                // pipe was measured and false-positives on both live sites, whose pipe sits inside a parameter
                // expansion. Separating the two needs shell parsing rather than text.
                because: "the permission is granted on the producer's first word, so a builtin's exemption — \
                          having no I/O to fail at — reaches a pipeline stage that does; refusing a producer \
                          containing a pipe was measured and refuses the tree's own legitimate sites too".into(),
                owner: Owner::Engine,
            }),
            "a_builtin_piped_into_an_external_command_is_a_stated_bound",
        ),
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        BoundDecl::pinned(
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
        // --- self-law-projection: the mutual-independence reaction's four limits ---
        //
        // Each extent is read off a run of that limit's own WHEN, never off the argument for it: a draft
        // declared the first as a false NEGATIVE and one run showed the reaction fires there.
        BoundDecl::unpinned(
            BoundId::new(
                "self-law-projection/a-reason-that-paraphrases-the-law-is-refused-a-stated-bound",
            ),
            "a dimension's `because` stating the mutual-independence law in different words, without the literal clause",
            Extent::Reached(Reached::OverReacts {
                because: "the check reads the `because` for the literal clause, so a reason that genuinely \
                          states the law in other words is refused; the direction is the safe one and closing \
                          it needs the reaction to decide two wordings state one law"
                    .into(),
            }),
            "`BACKLOG.md` — *four limits of the mutual-independence reaction*",
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
            "`BACKLOG.md` — *four limits of the mutual-independence reaction*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "self-law-projection/a-dimension-absent-from-the-reaction-s-own-list-is-not-examined-a-stated-bound",
            ),
            "a dimension crate whose package name is not in the reaction's hand-kept list",
            Extent::Reached(Reached::UnderReacts {
                because: "the list is typed beside a set that enumerates itself, and the set-coverage \
                          assertion compares a set produced by filtering on that same list, so an omission \
                          is invisible to both halves"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *four limits of the mutual-independence reaction*",
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
            "`BACKLOG.md` — *four limits of the mutual-independence reaction*",
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
        // --- release-coherence: the adopter-narrative rule's four limits ---
        //
        // Its reaction is a shell gate, and `PINNED-BY` resolves only a harness-registered Rust function — so
        // three of these cite `tests/release_coherence.rs`, a file that exists for them. The twin defends each
        // of the three too, through the same fixture builder, and cannot be cited.
        //
        // Every extent below is read off a run of that limit's own WHEN. The fourth has no mechanical WHEN to
        // run, and is unpinned for that reason rather than deferred.
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-dated-release-section-naming-machinery-is-not-observed-a-stated-bound",
            ),
            "an entry in a dated `## [X.Y.Z] - DATE` section naming a path under `scripts/`",
            Extent::Reached(Reached::NotAViolation {
                because: "a dated section records what was true at that release, so rewriting it to satisfy a \
                          rule written afterwards would falsify the record rather than repair it — the reason \
                          `docs/history/` is left alone; five entries in the released `[0.4.0]` name a gate and \
                          are meant to keep doing so"
                    .into(),
            }),
            "a_dated_section_naming_a_gate_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-gate-named-as-bare-prose-is-not-observed-a-stated-bound",
            ),
            "an adopter-facing entry naming a gate as ordinary prose rather than as a backticked token",
            Extent::Reached(Reached::UnderReacts {
                because: "recognition is by token, so the leak the rule exists to stop passes unseen when the \
                          name is unquoted; widening to a bare substring would fire on any sentence carrying \
                          the characters, trading a declared blindness for an undeclared false-positive surface"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_gate_named_as_bare_prose_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/machinery-tracked-by-nothing-is-not-observed-a-stated-bound",
            ),
            "an adopter-facing entry naming a file under `scripts/` that the judged repository does not track",
            Extent::Reached(Reached::UnderReacts {
                because: "the enumeration is `git ls-files scripts/`, so an untracked `scripts/` reads as \
                          absent and a citation of it goes unseen; closing this means judging worktree content, \
                          which this repository's gates are held not to do — the larger error"
                    .into(),
                owner: Owner::Engine,
            }),
            "machinery_tracked_by_nothing_is_a_stated_bound",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "release-coherence/an-adopter-entry-about-self-governance-naming-no-machinery-is-not-observed-a-stated-bound",
            ),
            "an adopter-facing entry whose subject is this repository's own governance and which names no path \
             under `scripts/`",
            Extent::Reached(Reached::UnderReacts {
                because: "the rule reads an entry's REFERENCES, and this residual needs a judgement over its \
                          SUBJECT — the prose instrument this repository designed, measured three times and \
                          rejected. It is live rather than hypothetical: two entries of exactly this shape sit \
                          under adopter headings in the section this change edited"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*",
        ),
    ]
}
