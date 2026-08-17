//! Observation bounds declared by Kanhe-owned repository checks.
//!
//! These declarations are consumed by this repository's bound-model gate. They are not part of any published
//! catalog: the checks they qualify live in Kanhe and ship in no package.

use tianheng::{BoundDecl, BoundId, Extent, FactGranularity, Owner, Reached};

/// Every observation bound the Kanhe-owned repository checks declare.
pub fn observation_bounds() -> Vec<BoundDecl> {
    vec![
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/whether-a-citation-demonstrates-the-direction-its-bound-declares-a-stated-bound",
            ),
            "a declared bound citing a test that bites, while demonstrating a different direction from the one \
             its extent predicts",
            Extent::Reached(Reached::UnderReacts {
                because: "`demonstrates()` names the direction a defence must show and reaches the projection \
                          label and the contradiction classification, while no reader compares that prediction \
                          with what the cited test asserts. Deciding what a test demonstrates from its source is \
                          a judgement over code of the kind measured and rejected over prose, and unlike a \
                          citation that never runs or never bites there is no reaction here whose gap a fixture \
                          could exhibit"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *a pin may defend a direction its bound does not declare*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "repository-checks/a-gate-reached-without-the-wrapper-a-stated-bound",
            ),
            "an act reaching cargo publish or a merge without going through its wrapper",
            Extent::Reached(Reached::UnderReacts {
                because: "both assertions guard the sanctioned path -- the wrapper requiring its gate to \
                          report one passing test, and the check pinning the identifier it cites. \
                          Reaching further would mean observing the operator's shell or GitHub's servers \
                          rather than this repository"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "repository-checks/a-title-edited-inside-the-re-read-itself-a-stated-bound",
            ),
            "a pull request title changing between the wrapper's post-gate re-read of it and `gh pr merge`",
            Extent::Reached(Reached::UnderReacts {
                because: "the wrapper pins two of its three judged inputs by construction -- the body \
                          travels as the value the gate judged, and the commit set is pinned through \
                          `--match-head-commit`, which the server decides atomically. `gh` offers no \
                          equivalent for the title, so a re-read shrinks the exposure from a whole \
                          `cargo test` to one API call rather than closing it"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-tool-configuration-set-in-the-environment-is-not-observed-a-stated-bound",
            ),
            "a value a sanctioned wrapper refuses as an argument, exported into its environment instead",
            Extent::Reached(Reached::UnderReacts {
                because: "the allowlist classifies ARGUMENTS, and cargo takes the same configuration from the \
                          environment -- measured on cargo 1.96.0, `--target not-a-real-triple` and \
                          `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical rustc-probe failure. \
                          Closing it is ordinary work here rather than another layer's, since the wrapper could \
                          scrub the environment before invoking cargo; it needs an allowlist over the \
                          environment, and legitimate setups export CARGO_HOME and CARGO_TARGET_DIR, so which \
                          set to admit is a decision this bound records instead of guessing"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_tool_configuration_set_in_the_environment_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-figure-written-in-words-at-one-hundred-or-above-is-not-matched-a-stated-bound",
            ),
            "a declared census's figure spelled in words at one hundred or above",
            Extent::Reached(Reached::UnderReacts {
                because: "the word reader covers the units, the tens, and one compound of the two, which \
                          stops at ninety-nine. The figures this repository writes in words are the small \
                          ones, and a set large enough to need three-digit words is one whose prose writes \
                          digits — so extending it upward buys nothing measurable, while a word reader that \
                          silently stops matching reads as covered"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_word_form_at_one_hundred_or_above_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-census-written-outside-markdown-is-not-observed-a-stated-bound",
            ),
            "a declared census written with the wrong figures in a tracked file that is not Markdown",
            Extent::Reached(Reached::UnderReacts {
                because: "the corpus is tracked Markdown, and widening it was measured rather than reasoned \
                          about: this repository's Rust sources carry census phrases as fixture input, where \
                          the figures are a parser's expected output and deliberately arbitrary, so admitting \
                          them would report a test asserting its own parser as a drifted document. The narrow \
                          corpus is what keeps every report actionable"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_census_outside_markdown_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/whether-a-mention-compiles-anything-is-not-observed-a-stated-bound",
            ),
            "a promised prelude member the external contract names only in a comment",
            Extent::Reached(Reached::UnderReacts {
                because: "the check asks whether the promise was noticed at all, and deciding that a \
                          mention is load-bearing is a judgement over text this repository has designed, \
                          measured and rejected. What makes a mention bite is the compiler; a comment-only \
                          mention still fails the reviewer reading the diff, which is the layer that owns it"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_member_named_only_in_a_comment_is_counted_as_named",
        ),
        BoundDecl::unpinned(
            BoundId::new(
                "repository-checks/a-check-that-should-distinguish-a-region-and-does-not-a-stated-bound",
            ),
            "a check judging a property over executed text on unclassified text — no region decision \
             written, or one a neighbouring scan of the same file contradicts",
            Extent::OutOfReach {
                because: "an absence is not a shape and nothing can scan for a filter never written, while a \
                          disagreement between two scans is visible only to something that can already \
                          recognize a region decision — the reaction measured against this repository and \
                          rejected for refusing more legitimate sites than defects"
                    .into(),
            },
            "`BACKLOG.md` — *a check that never wrote a region decision is invisible*",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-shell-comment-opened-by-a-metacharacter-stays-in-the-executed-region-a-stated-bound",
            ),
            "a shell comment marker written straight after an unquoted metacharacter, where bash opens a \
             comment and the token-start rule does not cut",
            Extent::Reached(Reached::OverReacts {
                because: "the rule tests for whitespace or line start, so text bash discards survives into \
                          the executed region and commentary can satisfy a property about executed text"
                    .into(),
            }),
            "a_shell_marker_after_a_metacharacter_stays_in_the_region",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-whitespace-preceded-shell-marker-inside-quotes-is-cut-a-stated-bound",
            ),
            "a shell comment marker preceded by whitespace inside a quoted string, where bash keeps it as \
             string content and the token-start rule cuts it",
            Extent::Reached(Reached::UnderReacts {
                because: "executed text is deleted, so a property about it is judged over less than the line \
                          carries — the direction the Core Contract forbids, and one a sentence in the \
                          classifier recorded as reaching the Rust region alone while both run the same rule"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_shell_marker_inside_quotes_is_cut_from_the_region",
        ),
        BoundDecl::pinned(
            BoundId::new("repository-checks/files-no-capability-claims-a-stated-bound"),
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
                "repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound",
            ),
            "a figure about an enumerable set, written in a phrasing no census declares",
            Extent::Reached(Reached::UnderReacts {
                because: "the declaration is the coverage — a census names the one sentence its figures are \
                          written in, and a count outside that sentence is unheld. Reaching it needs a \
                          judgement over prose, the instrument this repository designed, measured three times \
                          and rejected; `AGENTS.md` carries the other half as a rule with no check"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_count_in_an_undeclared_phrasing_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "repository-checks/a-hook-is-proposed-for-this-rule-a-stated-bound",
            ),
            "a squash merge made anywhere but through the sanctioned wrapper",
            Extent::OutOfReach {
                because: "a squash merge runs on GitHub's servers, so no local commit exists and no hook \
                          runs, and both values of the repository's squash-title setting append the serial; \
                          the check guards the sanctioned path to a merge, and a browser reaches no \
                          wrapper"
                    .into(),
            },
            "a_merge_made_outside_the_wrapper_is_not_observed",
        ),
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
                "observer-protocol/what-a-subject-does-not-establish-a-stated-bound",
            ),
            "a participant reporting a subject larger than what it observed",
            Extent::Reached(Reached::UnderReacts {
                because: "the constructor is public because an implementor must be able to return the \
                          outcome, so the type converts an omission into a commission and stops there; \
                          telling a reported subject from an observed one would need the engine to walk each \
                          dimension's corpus itself, which is the shared scanner 三儀 ⊥ 三儀 forbids"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_subject_is_declared_by_the_participant_and_not_verified",
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
        // The check that read the composition body is retired, so what is declared is the obligation being
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
        // --- observation-bound-register ---
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/which-member-holds-a-check-is-a-judgement-a-stated-bound",
            ),
            "which governance member a newly added check belongs to",
            Extent::Reached(Reached::UnderReacts {
                because: "the split is by what a check judges, and two mechanical rules were each \
                          measured unreliable: a text scan reads a comment naming a governance document as \
                          governance while a check scanning every tracked file names nothing, and the \
                          workspace marker means both `this needs the repository as its subject` and `this \
                          needs a fixture`. Position is the declaration"
                    .into(),
                owner: Owner::Engine,
            }),
            "`BACKLOG.md` — *which governance member a check belongs to is unobserved*",
        ),
        //
        // The register's own bounds — the only ones this crate declares about the check
        // that produces the register rather than about a dimension. `crates/kanhe/tests/pin_bites.rs` decides that a
        // citation's pin *bites* only where a mutation is declared for it; where none is, nothing decides.
        BoundDecl::unpinned(
            BoundId::new(
                "observation-bound-register/what-code-executed-inside-the-checkout-does-outside-it-is-not-observed-a-stated-bound",
            ),
            "code run inside the checkout writing outside it, or replacing a checked path so the check's own write lands elsewhere",
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
                because: "the check runs the test a fixed number of times and the number is readable in \
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
                "observation-bound-register/whether-a-record-perturbs-the-check-or-the-pin-s-own-assertions-is-not-observed-a-stated-bound",
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
        // --- reference-integrity ---
        //
        // Its check is `tests/reference_integrity.rs`, so this crate owns it. The capability declared no bound
        // at all until this one, while carrying a blanket exemption for `docs/history/` that no specification
        // mentioned — and that exemption was hiding a live defect rather than a limit. Narrowing it to the
        // dated sections it was actually for leaves exactly one thing unobserved, and this is it.
        BoundDecl::pinned(
            BoundId::new(
                "reference-integrity/a-path-already-wrong-when-a-dated-record-was-written-is-not-observed-a-stated-bound",
            ),
            "a path inside a dated CHANGELOG section that resolved to nothing at the moment it was written",
            Extent::Reached(Reached::UnderReacts {
                because: "the exemption is by section rather than by whether the path was once right, and \
                          separating the two needs the tree as it stood at that date — a per-section \
                          historical checkout, whose cost is not proportionate to a mistyped path in a \
                          record no one may rewrite"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_dated_changelog_section_keeps_its_paths_and_an_undated_one_does_not",
        ),
        // --- publish-source-integrity ---
        //
        // Its check is a Rust gate invoked by shell, and `PINNED-BY` resolves only a harness-registered Rust function — so
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
        // Its check is `tests/projection_register.rs`, so this crate owns these too. The two sit on opposite
        // sides of the false-negative line, which is exactly what the retired adjective slot could not express:
        // one is a shape the check never evaluates, the other a shape it can read and does not flag.
        BoundDecl::pinned(
            BoundId::new(
                "projection-register/whether-a-stated-regeneration-command-regenerates-its-document-is-not-observed-a-stated-bound",
            ),
            "a generated document whose header names a command that no longer regenerates it",
            Extent::OutOfReach {
                because: "the header is read and never evaluated; running the command would mean re-entering the \
                          `cargo test` harness already running, or — for the shell mechanism — writing the \
                          projection into the tree the check is judging, which every gate in this family is \
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
                // Not out of reach: the third mechanism's source sits in the tree this check already reads,
                // so it is seen and not flagged. Recording it as out-of-reach would be the misclassification
                // this model exists to prevent — a silent false negative dressed as an invisible shape.
                owner: Owner::Engine,
            }),
            "a_third_generation_mechanism_is_not_recognized",
        ),
        // --- release-coherence: the adopter-narrative rule's limits ---
        //
        // How many there are is not written here. The block grew from four to seven across two review rounds
        // while a header saying "four" sat on top of it, which is the same typed census this capability's own
        // check was made to stop writing.
        //
        // Its check is a Rust gate invoked by shell, and `PINNED-BY` resolves only a harness-registered Rust function — so
        // all but one cite `tests/release_coherence.rs`, a file that exists for them. The twin defends every
        // one of those too, through the same fixture builder, and cannot be cited.
        //
        // Every extent below is read off a run of that limit's own WHEN. One has no mechanical WHEN to run and
        // is unpinned for that reason rather than deferred.
        //
        // One more was declared here and RETIRED in the same window: while the scan compared whole backticked
        // spans, a gate named as unquoted prose passed, and that was declared. Adversarial review reproduced
        // three false negatives against the span reading — a span carrying a command, a double-backtick span,
        // an inline span wrapped across a line — and the word-run scan that closes all three reaches unquoted
        // prose too. Its WHEN was rerun against the new tree and the check fires, which is what retires a
        // bound rather than an argument that it should have closed.
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/prose-about-the-marker-is-read-as-a-marker-a-stated-bound",
            ),
            "a release section that discusses the breaking marker without marking anything",
            Extent::Reached(Reached::OverReacts {
                because: "the classifier reads the marker's presence rather than its position, so a section \
                          describing the marking rule is required to carry a migration it does not owe. The \
                          reach is kept deliberately: a positional matcher would stop observing a real break \
                          whose marker sits anywhere but an entry's first token, buying a false negative in \
                          the floor to remove a refusal an author can argue with"
                    .into(),
            }),
            "prose_about_the_marker_is_read_as_a_marker_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-dated-release-section-names-a-gate-a-stated-bound",
            ),
            "an entry in a dated `## [X.Y.Z] - DATE` section naming a path under `scripts/`",
            // Under-reacting rather than not-a-violation, and the distinction was argued in review rather than
            // assumed. Both values derive the same defence — does not react — so no run can separate them, and
            // the first draft picked the wrong one. `NotAViolation` says the check is RIGHT because nothing
            // is wrong. Something is: nine entries in the released `[0.4.0]` name machinery an adopter reading
            // that section still meets, which is exactly the harm this rule exists to stop. What is refused is
            // the REPAIR, not the diagnosis — and a limit accepted for a policy reason is a declared false
            // negative with an owner, which is the value that carries one.
            Extent::Reached(Reached::UnderReacts {
                because: "a dated section records what was true at that release, so rewriting it to satisfy a \
                          rule written afterwards would falsify the record rather than repair it — the reason \
                          `docs/history/` is left alone. The leak is real and stays: an adopter reading \
                          `[0.4.0]` meets nine entries naming files they can never run, and closing it needs a \
                          form of repair that adds to the record instead of editing it"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_dated_section_naming_a_gate_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound",
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
                "release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound",
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
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-basename-an-entry-writes-for-another-reason-a-stated-bound",
            ),
            "an adopter-facing entry naming something of its own — a basename, or the directory itself — that \
             the judged repository also tracks under `scripts/`",
            Extent::Reached(Reached::OverReacts {
                because: "a word is matched against basenames as well as paths, because the document cites \
                          both forms; narrowing it to full paths would lose every bare citation, and deciding \
                          which of two files a bare name means is a judgement about the sentence rather than \
                          about the reference"
                    .into(),
            }),
            "a_colliding_basename_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-directory-named-without-its-trailing-slash-a-stated-bound",
            ),
            "an adopter-facing entry naming a directory under `scripts/` without its trailing slash",
            Extent::Reached(Reached::UnderReacts {
                because: "directories are derived slash-terminated, and stripping that slash leaves a word \
                          indistinguishable from ordinary prose — `scripts` is an English plural this document \
                          already uses as one. Admitting the unslashed form for deeper names only, where the \
                          collision is less likely, would make the check judge which of its own keys read \
                          as English"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_directory_named_without_its_slash_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-name-reached-only-through-a-url-a-stated-bound",
            ),
            "an adopter-facing entry naming machinery only inside a URL",
            Extent::Reached(Reached::UnderReacts {
                because: "a word is a maximal run of path characters, so a scheme and host fuse with the path \
                          into one run that equals no tracked name; splitting a URL into its path would make \
                          the check judge a foreign host's layout as though it were this repository's"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_name_reached_only_through_a_url_is_a_stated_bound",
        ),
        BoundDecl::pinned(
            BoundId::new(
                "release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound",
            ),
            "a `### ` line inside a fenced code block, followed by entries that name machinery",
            Extent::Reached(Reached::UnderReacts {
                because: "the check walks the document's line grammar and does not track fences, so such a \
                          line sets the heading in force and can name the one exempt heading; it is latent \
                          rather than live — this repository's changelog carries no fenced block — and closing \
                          it means a second, stateful reading of a document this gate reads once"
                    .into(),
                owner: Owner::Engine,
            }),
            "a_heading_inside_a_fenced_block_is_a_stated_bound",
        ),
    ]
}
