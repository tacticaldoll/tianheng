# Observation bound extents

Where each declared **observation bound** stops the measure — not how far a scan walks (that is
`ScanDepth`, an adopter's knob), but where this family's own reaction deliberately stops.

**48 of 94 declared bounds are declared false negatives** — the reaction fires less than the truth, which is the one direction this family treats as a defect. That figure leads this document because a number in a footnote is not read, and each such bound names who must act:

- `external-crate-confinement/an-extern-crate-declaration-is-not-observed-a-stated-bound` — owner: engine
- `inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-bound` — owner: adopter
- `inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default` — owner: adopter
- `inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external` — owner: engine
- `inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default` — owner: adopter
- `observation-bound-register/what-code-executed-inside-the-checkout-does-outside-it-is-not-observed-a-stated-bound` — owner: engine
- `observation-bound-register/whether-a-citation-carrying-no-declared-mutation-is-defended-is-not-observed-a-stated-bound` — owner: engine
- `observation-bound-register/whether-a-citation-demonstrates-the-direction-its-bound-declares-a-stated-bound` — owner: engine
- `observation-bound-register/whether-a-cited-test-s-outcome-depends-on-its-run-count-is-not-observed-beyond-one-period-a-stated-bound` — owner: engine
- `observation-bound-register/whether-a-pin-gutted-but-not-committed-still-bites-is-not-observed-a-stated-bound` — owner: engine
- `observation-bound-register/whether-a-record-perturbs-the-check-or-the-pin-s-own-assertions-is-not-observed-a-stated-bound` — owner: engine
- `observation-bound-register/which-member-holds-a-check-is-a-judgement-a-stated-bound` — owner: engine
- `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound` — owner: engine
- `observer-protocol/what-a-subject-does-not-establish-a-stated-bound` — owner: engine
- `observer-protocol/whether-an-observer-s-own-verdict-is-correct-is-not-observed-a-stated-bound` — owner: adopter
- `observer-protocol/whether-the-shell-makes-an-independent-semantic-decision-is-not-observed-a-stated-bound` — owner: engine
- `projection-register/a-document-generated-by-an-unrecognized-mechanism-is-not-observed-a-stated-bound` — owner: engine
- `publish-source-integrity/the-tree-changing-after-the-gate-passed-is-not-observed-a-stated-bound` — owner: engine
- `publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound` — owner: inherited from the verification environment
- `reference-integrity/a-path-already-wrong-when-a-dated-record-was-written-is-not-observed-a-stated-bound` — owner: engine
- `reference-integrity/a-rust-identifier-named-in-prose-is-not-resolved-a-stated-bound` — owner: engine
- `release-coherence/a-dated-release-section-names-a-gate-a-stated-bound` — owner: engine
- `release-coherence/a-dependency-declared-under-a-quoted-cfg-target-is-not-observed-a-stated-bound` — owner: engine
- `release-coherence/a-directory-named-without-its-trailing-slash-a-stated-bound` — owner: engine
- `release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound` — owner: engine
- `release-coherence/a-name-reached-only-through-a-url-a-stated-bound` — owner: engine
- `release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound` — owner: engine
- `release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound` — owner: engine
- `repository-checks/a-census-written-outside-markdown-is-not-observed-a-stated-bound` — owner: engine
- `repository-checks/a-construction-shape-the-register-s-reader-does-not-model-a-stated-bound` — owner: engine
- `repository-checks/a-consumer-that-stops-early-is-not-on-the-reader-s-list-a-stated-bound` — owner: engine
- `repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound` — owner: engine
- `repository-checks/a-figure-written-in-words-at-one-hundred-or-above-is-not-matched-a-stated-bound` — owner: engine
- `repository-checks/a-gate-reached-without-the-wrapper-a-stated-bound` — owner: engine
- `repository-checks/a-refusal-constructed-outside-the-register-s-corpus-is-not-triaged-a-stated-bound` — owner: engine
- `repository-checks/a-title-edited-inside-the-re-read-itself-a-stated-bound` — owner: engine
- `repository-checks/a-tool-configuration-set-in-the-environment-is-not-observed-a-stated-bound` — owner: engine
- `repository-checks/a-whitespace-preceded-shell-marker-inside-quotes-is-cut-a-stated-bound` — owner: engine
- `repository-checks/files-no-capability-claims-a-stated-bound` — owner: engine
- `repository-checks/whether-a-mention-compiles-anything-is-not-observed-a-stated-bound` — owner: engine
- `runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound` — owner: inherited from the corpus entry point
- `runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound` — owner: engine
- `self-law-projection/a-reason-carrying-the-clause-while-negating-the-law-is-not-observed-a-stated-bound` — owner: engine
- `self-law-projection/a-workspace-dependency-allowlist-is-not-examined-a-stated-bound` — owner: engine
- `semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound` — owner: engine
- `semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound` — owner: engine
- `semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound` — owner: engine
- `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound` — owner: engine

Generated from each dimension's `observation_bounds()` by `crates/kanhe/tests/observation_bound_model.rs`. **Do not edit by hand** — regenerate with
`BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test observation_bound_model`.

**What this document does not claim.** The classification is *authored*: the type refuses a contradiction and derives what each bound's defence must demonstrate, but nothing verifies that a bound recorded as over-reacting really over-reacts rather than under-reacting. This capability declares further limits as bounds of its own — among them, an answer that depends on which corpus entry point observed it has no extent of its own and is recorded as under-reacting with the entry point as its owner, and a bound both out of reach and granularity-limited cannot be expressed at all. The sections below list every one of them; this paragraph deliberately does not, because a list typed here is a literal in a template and the freshness check compares that text with itself. This disclosure is authored rather than derived from the specification and held both ways; the backlog carries that.

**refuses to judge** and *out of reach* are kept distinct deliberately. The misclassification this model exists to prevent was exactly a confusion between them — a prediction of a silent false negative where the real behaviour was a fail-loud refusal — and a direction that cannot be named cannot be predicted with.

## as intended, granularity bounded (5)

### `observation-bound-model/an-answer-that-depends-on-the-corpus-entry-point-has-no-extent-of-its-own-a-stated-bound`

> a bound whose outcome differs by which corpus entry point observed it

- **because**: it is recorded as an under-reaction owned by the entry point rather than carrying a value of its own, so it shares that value with bounds whose answer does not depend on an entry point; one live instance does not earn a value every other member has several of
- **its defence must show**: collapses granularity
- **pinned by**: `an_entry_dependent_bound_is_declared_as_under_reacting`

### `runtime-origin-assertion/an-absolute-path-literal-s-target-outside-the-anchor-keeps-its-absolute-label-a-stated-bound`

> a module reached only through an absolute `#[path = "/…"]` outside the scanning anchor

- **because**: the literal has no textual relationship to the anchor, so the site is named with the raw absolute path — the violation is still emitted
- **its defence must show**: collapses granularity
- **pinned by**: `an_absolute_path_literal_outside_the_anchor_keeps_the_path_the_literal_wrote`

### `runtime-origin-assertion/identical-expression-repeated-in-the-same-function-collapses-to-one-finding-a-stated-bound`

> `assert_boundary!(SEAM, obj)` written twice verbatim in one function

- **because**: no further source content distinguishes the two occurrences, so they share one finding — the site still reacts
- **its defence must show**: collapses granularity
- **pinned by**: `identical_expression_repeated_in_the_same_function_collapses_to_one_violation`

### `semantic-dyn-trait-boundary/a-public-item-naming-such-an-alias-is-not-expanded-a-stated-bound`

> a public item whose signature names a public alias that itself holds a `dyn`

- **because**: the `dyn` is already caught at the alias declaration, so naming it again through the item would be a second finding for one shape
- **its defence must show**: collapses granularity
- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-boundary/an-unrenderable-sub-node-is-a-stated-bound`

> two trait objects differing only inside a sub-node that cannot be rendered

- **because**: a complex const-generic expression, a same-named macro, a `verbatim` type or a lifetime cannot be rendered stably, so the two share one subject and key — each still reacts on first occurrence, and only baseline-dedup granularity is bounded
- **its defence must show**: collapses granularity
- **pinned by**: `an_unrenderable_sub_node_is_a_stated_rendering_bound`

## declines to refuse (1)

### `semantic-trait-impl-locality/a-cfg-gated-module-with-an-absent-file-is-skipped-not-a-scan-error-a-stated-bound`

> a cfg-gated module declaration whose source file is absent from the checkout

- **because**: the whole-crate walk skips the module rather than failing the gate with a scan error, because an absent cfg-gated file is an ordinary checkout state and refusing to judge on it would make the gate unusable
- **its defence must show**: does not refuse
- **pinned by**: `hunyi::a_cfg_gated_module_with_no_file_is_skipped_not_errored`

## not a violation (3)

### `semantic-async-exposure-boundary/a-body-nested-module-is-a-stated-bound`

> `pub async fn` inside a `mod` declared in a function body

- **because**: a `mod` inside a fn body is not public API — it is unreachable as `crate::…`, so there is nothing exposed to react to
- **its defence must show**: does not react
- **pinned by**: `async_subtree_does_not_observe_a_body_nested_module`

### `semantic-reexport-exposure/an-underscore-rename-is-a-documented-bound`

> `pub use crate::infra::DbPool as _;` under a boundary forbidding that module

- **because**: `as _` binds no nameable path a consumer can reach, so nothing is exposed
- **its defence must show**: does not react
- **pinned by**: `restricted_and_private_and_underscore_reexports_do_not_react`

### `semantic-signature-coupling/a-plain-item-nested-the-same-way-stays-a-stated-bound`

> a plain item nested inside a body, where an `impl` in the same position is recovered

- **because**: a plain item there is genuinely scoped to that body and unreachable as `crate::…`, exactly like the body-nested-module bound
- **its defence must show**: does not react
- **pinned by**: `a_plain_fn_directly_in_a_const_body_stays_a_stated_bound`

## out of reach (25)

### `external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-bound`

> a confined-crate `use` written inside a string literal or a macro body

- **because**: comments, string literals and macro bodies are stripped before scanning
- **its defence must show**: does not react
- **pinned by**: `confine_ignores_a_use_inside_a_string_literal`

### `inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound`

> a read reached through a method call on a receiver

- **because**: no type inference is performed on the receiver, so the confined path is never resolved from the call site
- **its defence must show**: does not react
- **pinned by**: `inline_receiver_method_read_is_a_bound`

### `inline-symbol-path-confinement/an-external-crate-re-export-is-a-documented-bound`

> a confined path reached through an external crate's own re-export

- **because**: foreign ASTs are not scanned, so a re-export chain leaving this workspace is never followed
- **its defence must show**: does not react
- **pinned by**: `inline_foreign_reexport_of_the_confined_path_is_a_bound`

### `observation-bound-model/a-bound-both-out-of-reach-and-granularity-limited-cannot-be-expressed-a-stated-bound`

> a bound both invisible to the observation source and limited in the granularity of the fact it would have produced

- **because**: granularity is carried only by the as-intended extent, so the pair has no representation at all; no declared bound exhibits it, and offering granularity on every extent would invite a combination nothing shows while weakening the nesting that makes a contradiction unwritable
- **its defence must show**: does not react
- **pinned by**: `granularity_is_carried_only_by_the_as_intended_extent`

### `observation-bound-model/whether-a-declaration-s-stated-cause-is-the-real-cause-is-not-observed-a-stated-bound`

> a declaration whose rationale names a cause that is not why the reaction stops

- **because**: the extent is typed and checkable while the rationale is prose the model never reads; requiring the two to agree would trade a fact for a heuristic
- **its defence must show**: does not react
- **pinned by**: `a_rationale_that_contradicts_its_extent_is_a_stated_bound`

### `observer-protocol/a-trait-object-on-a-wrapped-signature-s-continuation-line-is-not-seen-a-stated-bound`

> a public signature spanning several lines that names a trait object on a line not beginning with `pub `

- **because**: the reaction reads this crate lexically, one line at a time, because 渾儀 governs no module of it and the `dyn`-trait DSL offers only forbid-all and forbid-named-operands, so a declared exposure would be a name with no reaction
- **its defence must show**: does not react
- **pinned by**: `a_trait_object_on_a_continuation_line_is_not_recognized`

### `observer-protocol/whether-an-observer-s-declared-bounds-are-complete-is-not-observed-a-stated-bound`

> an observer that declares some of its limits and omits others

- **because**: the trait compels a declaration and never a complete one; no reaction can enumerate the limits of a reaction it did not write, so an omission is invisible
- **its defence must show**: does not react
- **pinned by**: `an_observer_may_under_declare_its_bounds`

### `projection-register/whether-a-stated-regeneration-command-regenerates-its-document-is-not-observed-a-stated-bound`

> a generated document whose header names a command that no longer regenerates it

- **because**: the header is read and never evaluated; running the command would mean re-entering the `cargo test` harness already running, or — for the shell mechanism — writing the projection into the tree the check is judging, which every gate in this family is forbidden from doing
- **its defence must show**: does not react
- **pinned by**: `a_regeneration_command_is_registered_and_never_run`

### `repository-checks/a-check-that-should-distinguish-a-region-and-does-not-a-stated-bound`

> a check judging a property over executed text on unclassified text — no region decision written, or one a neighbouring scan of the same file contradicts

- **because**: an absence is not a shape and nothing can scan for a filter never written, while a disagreement between two scans is visible only to something that can already recognize a region decision — the reaction measured against this repository and rejected for refusing more legitimate sites than defects
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a check that never wrote a region decision is invisible*

### `repository-checks/a-hook-is-proposed-for-this-rule-a-stated-bound`

> a squash merge made anywhere but through the sanctioned wrapper

- **because**: a squash merge runs on GitHub's servers, so no local commit exists and no hook runs, and both values of the repository's squash-title setting append the serial; the check guards the sanctioned path to a merge, and a browser reaches no wrapper
- **its defence must show**: does not react
- **pinned by**: `a_merge_made_outside_the_wrapper_is_not_observed`

### `runtime-origin-assertion/source-outside-a-member-s-library-or-binary-target-subtree-is-out-of-scope-a-stated-bound`

> a probe or seam mention in `tests/`, `examples/`, or `build.rs`

- **because**: the audit's corpus is the member's library and binary targets, so it never reads those files at all
- **its defence must show**: does not react
- **pinned by**: `source_outside_lib_or_bin_target_subtree_is_out_of_scope_corpus_bound`

### `semantic-dyn-trait-boundary/a-macro-generated-dyn-is-a-documented-bound`

> a `dyn` appearing only in a macro's expansion, with no `dyn` token at the call site

- **because**: macros are not expanded — the universal 渾儀 macro-expansion bound — so the token never enters the observed AST
- **its defence must show**: does not react
- **pinned by**: `a_macro_generated_dyn_is_a_documented_coverage_bound`

### `semantic-dyn-trait-boundary/a-private-alias-hiding-a-dyn-in-a-public-position-is-a-stated-bound`

> a non-public `type` alias holding a `dyn`, named by a public signature

- **because**: the resolver does not expand `type` aliases, so the `dyn` is never seen from the public position that exposes it
- **its defence must show**: does not react
- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> `dyn Frobnicate` where the bare principal has no `use`, no dependency, and no local declaration

- **because**: the oracle does not over-reach a single bare segment, so a prelude or glob-imported trait resolves to nothing rather than to a guess
- **its defence must show**: does not react
- **pinned by**: `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound`

### `semantic-forbidden-marker/an-unresolvable-hand-impl-self-type-is-a-documented-bound`

> a hand-written impl whose self-type arrives through a glob import

- **because**: a glob's leaves are not enumerable, so the self-type cannot be resolved to its definition — the co-located, `use`-imported, re-export-spelled and alias cases do resolve and react
- **its defence must show**: does not react
- **pinned by**: `an_unresolvable_glob_self_type_is_a_documented_bound`

### `semantic-impl-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> `impl Frobnicate` where the bare principal has no `use`, no dependency, and no local declaration

- **because**: the same resolver limit as the `dyn` operand dimension — a single bare segment is not over-reached
- **its defence must show**: does not react
- **pinned by**: `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound`

### `semantic-reexport-exposure/a-non-forbidden-root-external-glob-is-a-documented-bound`

> `pub use worklane_core::spi::*;` under a boundary forbidding a different module of that crate

- **because**: an external glob's individual leaves are not enumerable, so none is observed
- **its defence must show**: does not react
- **pinned by**: `extern_glob_nonforbidden_root_is_a_stated_bound`

### `semantic-reexport-exposure/a-re-export-renamed-through-a-foreign-module-is-a-documented-bound`

> a re-export of a foreign prelude path that itself re-exports a forbidden module's type

- **because**: the foreign chain is not parsed, so only the written path is matched — the reaction never claims to have followed it
- **its defence must show**: does not react
- **pinned by**: `foreign_prelude_rename_is_a_stated_bound`

### `semantic-reexport-exposure/a-sibling-root-glob-is-a-documented-bound`

> `pub use crate::elsewhere::*;` where that module transitively re-exports a forbidden type

- **because**: the glob's leaves are not enumerable here, so the transitively re-exported leaf is never seen
- **its defence must show**: does not react
- **pinned by**: `sibling_root_glob_does_not_react`

### `semantic-reexport-exposure/an-ancestor-root-glob-spanning-a-deeper-forbidden-prefix-is-a-documented-bound`

> `pub use crate::infra::*;` where the forbidden prefix is deeper than the glob root

- **because**: whether the glob root publicly re-exports the deeper forbidden subtree cannot be enumerated — the sharper sub-case of the sibling glob, declared separately rather than lumped with it
- **its defence must show**: does not react
- **pinned by**: `ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react`

### `semantic-signature-coupling/a-macro-under-another-name-is-not-treated-as-transparent-a-stated-bound`

> an arbitrary macro invocation whose body holds item-shaped content

- **because**: the invocation is not transparent and its body is not read — extracting from it would read an `impl` body's braces as an arm and report an item the macro may never emit
- **its defence must show**: does not react
- **pinned by**: `an_arbitrary_macro_body_is_not_read_as_transparent_arms`

### `semantic-trait-impl-exposure/a-glob-imported-type-in-an-impl-position-is-a-documented-bound`

> an impl position naming a type that arrives through a glob import

- **because**: a glob's leaves are not enumerable, so the type in that position resolves to nothing
- **its defence must show**: does not react
- **pinned by**: `a_glob_imported_type_in_an_impl_position_is_a_documented_coverage_bound`

### `semantic-trait-impl-locality/a-macro-generated-impl-is-a-documented-bound`

> an `impl` appearing only in a macro's expansion

- **because**: macros are not expanded, so the impl never enters the observed AST
- **its defence must show**: does not react
- **pinned by**: `a_macro_generated_impl_is_a_documented_bound`

### `semantic-unsafe-confinement/macro-generated-unsafe-is-a-documented-bound`

> an `unsafe` block or item appearing only in a macro's expansion

- **because**: macros are not expanded, so the `unsafe` token never enters the observed AST
- **its defence must show**: does not react
- **pinned by**: `unsafe_in_a_macro_body_is_a_stated_bound`

### `semantic-visibility-boundary/a-macro-generated-item-is-a-documented-bound`

> a `pub` item appearing only in a macro's expansion

- **because**: macros are not expanded, so the item never enters the observed AST
- **its defence must show**: does not react
- **pinned by**: `a_macro_invocation_pub_item_is_a_documented_bound`

## over-reacts (12)

### `crate-dependency-boundary/an-optional-dependency-edge-is-observed-as-a-declared-one-a-stated-bound`

> a dependency edge declared `optional = true`, reachable only when a feature enables it

- **because**: the rules read the declared dependency set, and cargo reports an optional edge in it like any other, so an edge confined to a non-default feature is governed as though it were unconditional — no rule can express *only when that feature is on*
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `an_optional_dependency_edge_is_observed_as_a_declared_one`

### `crate-source-boundary/a-git-plus-version-dependency-is-flagged-though-it-would-publish-a-stated-bound`

> a dependency declaring both `git` and `version` under a registry-only allowlist

- **because**: the rule governs the declared source kind, not publish-eligibility, so a dependency that would `cargo publish` successfully is still classified `Git`
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist`

### `external-crate-confinement/cfg-gated-code-is-observed-as-written-a-stated-bound`

> a confined-crate import under a `#[cfg(...)]` the build would not enable

- **because**: the predicate is never evaluated, so a dead arm is observed as live — cfg-blindness inherited from the module scanner, which reacts wider than the build
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `confine_external_crate_is_cfg_blind_to_unenabled_cfg_arms`

### `release-coherence/a-basename-an-entry-writes-for-another-reason-a-stated-bound`

> an adopter-facing entry naming something of its own — a basename, or the directory itself — that the judged repository also tracks under `scripts/`

- **because**: a word is matched against basenames as well as paths, because the document cites both forms; narrowing it to full paths would lose every bare citation, and deciding which of two files a bare name means is a judgement about the sentence rather than about the reference
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_colliding_basename_is_a_stated_bound`

### `release-coherence/prose-about-the-marker-is-read-as-a-marker-a-stated-bound`

> a release section that discusses the breaking marker without marking anything

- **because**: the classifier reads the marker's presence rather than its position, so a section describing the marking rule is required to carry a migration it does not owe. The reach is kept deliberately: a positional matcher would stop observing a real break whose marker sits anywhere but an entry's first token, buying a false negative in the floor to remove a refusal an author can argue with
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `prose_about_the_marker_is_read_as_a_marker_a_stated_bound`

### `repository-checks/a-negative-value-cargo-documents-is-refused-by-the-shape-rule-a-stated-bound`

> a `--jobs` value cargo documents as a negative job count, refused by the wrapper's shape rule

- **because**: the shape rule asks one question of every value-taking arm, and a leading digit means a job count for one arm and nothing for `--package` or `--registry`. Admitting it means asking the shape question differently per arm, which is the arrangement one check exists to replace. The caller passes the count instead
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_refused_flag_cannot_sit_in_an_admitted_arguments_value_position`

### `repository-checks/a-shell-comment-opened-by-a-metacharacter-stays-in-the-executed-region-a-stated-bound`

> a shell comment marker written straight after an unquoted metacharacter, where bash opens a comment and the token-start rule does not cut

- **because**: the rule tests for whitespace or line start, so text bash discards survives into the executed region and commentary can satisfy a property about executed text
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_shell_marker_after_a_metacharacter_stays_in_the_region`

### `runtime-origin-assertion/a-composite-shape-yields-a-truncated-origin-a-stated-bound`

> a registered type that is a reference, tuple, array, pointer, or function pointer

- **because**: the derived origin is a truncated rendering matching no module name, so the crossing reacts fail-closed rather than being admitted through the wrapper
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `the_derived_origin_honors_its_stated_shape_bounds`

### `self-law-projection/a-comment-naming-every-member-for-another-reason-is-refused-a-stated-bound`

> one contiguous line-comment block naming every current allowlist member for a purpose other than copying the declaration

- **because**: the block check asks whether the members all appear and never why, and teaching it to read intent would be a heuristic over prose
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_comment_naming_every_member_for_another_reason_is_refused`

### `self-law-projection/a-doc-example-of-the-dependency-dsl-is-refused-a-stated-bound`

> a line comment under the shell naming `restrict_dependencies_to(` in order to teach the re-exported DSL

- **because**: the recognizer reads a comment's text and never its purpose, so a doc example of a DSL the shell publishes is refused exactly as a restatement of its own declaration would be
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_doc_example_of_the_dependency_dsl_is_refused`

### `self-law-projection/a-reason-that-paraphrases-the-law-is-refused-a-stated-bound`

> a dimension's `because` stating the mutual-independence law in different words, without the literal clause

- **because**: the check reads the `because` for the literal clause, so a reason that genuinely states the law in other words is refused; the direction is the safe one and closing it needs the check to decide two wordings state one law
- **its defence must show**: reacts on a harmless shape
- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

### `semantic-visibility-boundary/a-pub-in-narrow-path-item-may-over-react-under-a-tight-ceiling-a-stated-bound`

> `pub(in crate::a) fn` on an item already directly in `crate::a`, under a `Module` ceiling

- **because**: the conservative `Crate` rank exceeds the `Module` ceiling, so an effectively private item may react — never a silent pass
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_pub_in_narrow_path_over_reacts_under_a_module_ceiling`

## under-reacts (48)

### `external-crate-confinement/an-extern-crate-declaration-is-not-observed-a-stated-bound`

> `extern crate libc;` reaching a confined crate without a `use`

- **because**: the rule observes `use` imports only, so a crate reached through an `extern crate` declaration and fully-qualified paths is not seen
- **its defence must show**: does not react
- **pinned by**: `confine_ignores_an_extern_crate_declaration`

### `inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-bound`

> a read expressed through a verb outside the adopter's declared set

- **because**: the engine declines to guess which verbs are reads, so a verb the declaration omits is not observed
- **its defence must show**: does not react
- **pinned by**: `inline_a_verb_outside_the_declared_set_is_a_bound`

### `inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default`

> a confined path mentioned in value position rather than called

- **because**: value-position mentions are not observed under the default; the adopter's `strict_prefix_only()` reacts to them
- **its defence must show**: does not react
- **pinned by**: `inline_value_capture_is_a_bound_under_the_default`

### `inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external`

> a call reached through an `extern crate … as` alias head under strict-external

- **because**: the use-map is built from `use` declarations only, so an `extern crate` rename binds an alias the resolver does not know
- **its defence must show**: does not react
- **pinned by**: `inline_strict_external_extern_crate_rename_is_a_stated_bound`

### `inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default`

> a fully-qualified call into an external crate with no `use`

- **because**: the default observes `use`-rooted paths, leaving the un-`use`d fully-qualified spelling to the adopter's stricter opt-in
- **its defence must show**: does not react
- **pinned by**: `inline_strict_external_absent_fully_qualified_call_is_a_bound`

### `observation-bound-register/what-code-executed-inside-the-checkout-does-outside-it-is-not-observed-a-stated-bound`

> code run inside the checkout writing outside it, or replacing a checked path so the check's own write lands elsewhere

- **because**: running the cited test is the whole method, so code execution inside the checkout is granted unconditionally; the shared common directory is what makes a git-reading citation reachable at all, and re-checking a resolved path after the build would re-check the window that defeated it
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-citation-carrying-no-declared-mutation-is-defended-is-not-observed-a-stated-bound`

> a pinning citation for which no mutation is declared

- **because**: the gate runs the mutations it is given and nothing else, so a citation with no record is neither exercised nor refused; authoring a record that genuinely perturbs the pinned point is per-bound work, which is why coverage is disclosed on every clean run rather than implied
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-citation-demonstrates-the-direction-its-bound-declares-a-stated-bound`

> a declared bound citing a test that bites, while demonstrating a different direction from the one its extent predicts

- **because**: `demonstrates()` names the direction a defence must show and reaches the projection label and the contradiction classification, while no reader compares that prediction with what the cited test asserts. Deciding what a test demonstrates from its source is a judgement over code of the kind measured and rejected over prose, and unlike a citation that never runs or never bites there is no reaction here whose gap a fixture could exhibit
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a pin may defend a direction its bound does not declare*

### `observation-bound-register/whether-a-cited-test-s-outcome-depends-on-its-run-count-is-not-observed-beyond-one-period-a-stated-bound`

> a cited test passing and failing by a period the fixed run sequence does not break

- **because**: the check runs the test a fixed number of times and the number is readable in its own source, so a matching period escapes; closing it needs each run unable to observe how many times the test has run, whose cost grows with the coverage this capability exists to grow
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-pin-gutted-but-not-committed-still-bites-is-not-observed-a-stated-bound`

> a cited pin whose assertions are removed in the working directory and not committed

- **because**: the checkout under test is HEAD's content, because mutating the author's own checkout is what a separate checkout exists to avoid; the two properties are in tension and this one is given up deliberately
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-record-perturbs-the-check-or-the-pin-s-own-assertions-is-not-observed-a-stated-bound`

> a record naming the file its pin lives in and neutralising one of that pin's assertions

- **because**: a killed pin does not say what killed it, and refusing a record that edits its pin's own file would refuse this tree's first seeded record, which legitimately perturbs a recognizer sitting beside the pin that defends it
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/which-member-holds-a-check-is-a-judgement-a-stated-bound`

> which governance member a newly added check belongs to

- **because**: the split is by what a check judges, and two mechanical rules were each measured unreliable: a text scan reads a comment naming a governance document as governance while a check scanning every tracked file names nothing, and the workspace marker means both `this needs the repository as its subject` and `this needs a fixture`. Position is the declaration
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *which governance member a check belongs to is unobserved*

### `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound`

> a whole-line signature copy — commented, in a string literal, or otherwise — with the definition moved out of the inspected source

- **because**: the reader knows nothing of comments or literals, so one whole-line occurrence anchors whatever follows it; what passes is a second hand-maintained path that agrees today, since a divergent one is caught by observation-bound-model's bijection over Observer::bounds — measured both ways
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the bounds-method reader anchors on a whole-line occurrence that is not the definition*

### `observer-protocol/what-a-subject-does-not-establish-a-stated-bound`

> a participant reporting a subject larger than what it observed

- **because**: the constructor is public because an implementor must be able to return the outcome, so the type converts an omission into a commission and stops there; telling a reported subject from an observed one would need the engine to walk each dimension's corpus itself, which is the shared scanner 三儀 ⊥ 三儀 forbids
- **its defence must show**: does not react
- **pinned by**: `a_subject_is_declared_by_the_participant_and_not_verified`

### `observer-protocol/whether-an-observer-s-own-verdict-is-correct-is-not-observed-a-stated-bound`

> a composed observer returning an outcome that misjudges the workspace it read

- **because**: the fold composes verdicts and does not adjudicate them; second-guessing each participant would need a second implementation of every dimension
- **its defence must show**: does not react
- **pinned by**: `the_fold_does_not_adjudicate_a_participant_s_verdict`

### `observer-protocol/whether-the-shell-makes-an-independent-semantic-decision-is-not-observed-a-stated-bound`

> the shell's composition arm deciding semantic emptiness itself instead of leaving it to the observer it invokes

- **because**: a text reader over the composition body was defeated at every level it could be narrowed to — name resolution, the parameter's binding site, the identity of the definition, the caller frame, and execution, which no reading of text reaches — so invoking the observer made the two paths' EQUALITY construction-held and left this untouched, measured: a guard above that call compiles and passes every gate
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the shell's semantic delegation, held by construction*

### `projection-register/a-document-generated-by-an-unrecognized-mechanism-is-not-observed-a-stated-bound`

> a document generated by neither the shared Rust rule nor a `check_*` gate under `BLESS`, whose author also omitted the marker

- **because**: it is absent from both sides of the correspondence, so that correspondence holds over a surface missing a member and the register reports itself complete
- **its defence must show**: does not react
- **pinned by**: `a_third_generation_mechanism_is_not_recognized`

### `publish-source-integrity/the-tree-changing-after-the-gate-passed-is-not-observed-a-stated-bound`

> the repository altered between the source gate's single pass and `cargo publish` reading the tree

- **because**: the gate is one process and the act is another, and `cargo publish` takes no argument naming the commit it must package -- there is no `--match-head-commit` to pin what was judged, which is what closes the equivalent window on the merge path. `cargo publish` refuses a dirty worktree, which narrows it and is weaker than what the gate holds: a tree amended and committed is clean again
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the window the publish wrapper can only narrow*

### `publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound`

> a release tag carrying a cryptographically valid signature made by a key no maintainer authorized

- **because**: validity is verifiable with no configuration and attribution is not — it needs an allowed-signers file that exists on a maintainer's machine and not in CI, so requiring it would make the same tag judged differently by where the gate ran
- **its defence must show**: does not react
- **pinned by**: `a_valid_signature_from_an_unauthorized_key_is_accepted`

### `reference-integrity/a-path-already-wrong-when-a-dated-record-was-written-is-not-observed-a-stated-bound`

> a path inside a dated CHANGELOG section that resolved to nothing at the moment it was written

- **because**: the exemption is by section rather than by whether the path was once right, and separating the two needs the tree as it stood at that date — a per-section historical checkout, whose cost is not proportionate to a mistyped path in a record no one may rewrite
- **its defence must show**: does not react
- **pinned by**: `a_dated_changelog_section_keeps_its_paths_and_an_undated_one_does_not`

### `reference-integrity/a-rust-identifier-named-in-prose-is-not-resolved-a-stated-bound`

> a backticked snake_case name written in a doc comment's prose rather than as an intra-doc link

- **because**: no reader of text can tell a name that should resolve from one that should not: measured over every published crate's `src`, 859 of 2,373 such tokens match no declaration in the tree, and the most frequent of those are Rust keywords, attribute names and std method names. Separating them needs type information about a receiver, which `inline-symbol-path-confinement` already declares unobserved
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a Rust identifier named in prose is resolved by no reaction*

### `release-coherence/a-dated-release-section-names-a-gate-a-stated-bound`

> an entry in a dated `## [X.Y.Z] - DATE` section naming a path under `scripts/`

- **because**: a dated section records what was true at that release, so rewriting it to satisfy a rule written afterwards would falsify the record rather than repair it — the reason `docs/history/` is left alone. The leak is real and stays: an adopter reading `[0.4.0]` meets nine entries naming files they can never run, and closing it needs a form of repair that adds to the record instead of editing it
- **its defence must show**: does not react
- **pinned by**: `a_dated_section_naming_a_gate_is_a_stated_bound`

### `release-coherence/a-dependency-declared-under-a-quoted-cfg-target-is-not-observed-a-stated-bound`

> a family dependency declared under `[target.'cfg(…)'.dependencies]` or its `.NAME` form

- **because**: the reader decides which tables hold dependencies from the heading, and this heading's second key is a quoted cfg expression -- reading which configurations it selects is a grammar of its own rather than a context in front of a dependency table. The bare-triple sibling is read, because two bare TOML keys need nothing guessed
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a dependency declared under a quoted cfg target is not observed*

### `release-coherence/a-directory-named-without-its-trailing-slash-a-stated-bound`

> an adopter-facing entry naming a directory under `scripts/` without its trailing slash

- **because**: directories are derived slash-terminated, and stripping that slash leaves a word indistinguishable from ordinary prose — `scripts` is an English plural this document already uses as one. Admitting the unslashed form for deeper names only, where the collision is less likely, would make the check judge which of its own keys read as English
- **its defence must show**: does not react
- **pinned by**: `a_directory_named_without_its_slash_is_a_stated_bound`

### `release-coherence/a-heading-inside-a-fenced-code-block-a-stated-bound`

> a `### ` line inside a fenced code block, followed by entries that name machinery

- **because**: the check walks the document's line grammar and does not track fences, so such a line sets the heading in force and can name the one exempt heading; it is latent rather than live — this repository's changelog carries no fenced block — and closing it means a second, stateful reading of a document this gate reads once
- **its defence must show**: does not react
- **pinned by**: `a_heading_inside_a_fenced_block_is_a_stated_bound`

### `release-coherence/a-name-reached-only-through-a-url-a-stated-bound`

> an adopter-facing entry naming machinery only inside a URL

- **because**: a word is a maximal run of path characters, so a scheme and host fuse with the path into one run that equals no tracked name; splitting a URL into its path would make the check judge a foreign host's layout as though it were this repository's
- **its defence must show**: does not react
- **pinned by**: `a_name_reached_only_through_a_url_is_a_stated_bound`

### `release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound`

> an adopter-facing entry whose subject is this repository's own governance and which names no path under `scripts/`

- **because**: the rule reads an entry's REFERENCES, and this residual needs a judgement over its SUBJECT — the prose instrument this repository designed, measured three times and rejected. It is live rather than hypothetical: two entries of exactly this shape sit under adopter headings in the section this change edited
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*

### `release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound`

> an adopter-facing entry naming a file under `scripts/` that the judged repository does not track

- **because**: the enumeration is `git ls-files scripts/`, so an untracked `scripts/` reads as absent and a citation of it goes unseen; closing this means judging worktree content, which this repository's gates are held not to do — the larger error
- **its defence must show**: does not react
- **pinned by**: `machinery_tracked_by_nothing_is_a_stated_bound`

### `repository-checks/a-census-written-outside-markdown-is-not-observed-a-stated-bound`

> a declared census written with the wrong figures in a tracked file that is not Markdown

- **because**: the corpus is tracked Markdown, and widening it was measured rather than reasoned about: this repository's Rust sources carry census phrases as fixture input, where the figures are a parser's expected output and deliberately arbitrary, so admitting them would report a test asserting its own parser as a drifted document. The narrow corpus is what keeps every report actionable
- **its defence must show**: does not react
- **pinned by**: `a_census_outside_markdown_is_a_stated_bound`

### `repository-checks/a-construction-shape-the-register-s-reader-does-not-model-a-stated-bound`

> a bare reference to a registered or unregistered constructor's name, where whether it names the constructor taken by value or a local variable sharing its spelling is not decidable from syntax

- **because**: the register's reader used to be text over Rust and not exhaustive over the language, where a byte char literal, a raw string, or a two-line closure parameter list could desynchronise a character scan entirely -- invisible to both of its readings at once, the unsafe direction this bound named. Reading this repository's own Rust with a real parser instead of scanning it closed that floor; what remains is not lexical. Whether a bare reference names the constructor taken by value or a local sharing its spelling is not written down anywhere a parse tree carries, and answering it needs name resolution, which a reader of syntax alone does not have
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a bare reference to a registered constructor's name cannot be told from a local variable sharing its spelling without name resolution*

### `repository-checks/a-consumer-that-stops-early-is-not-on-the-reader-s-list-a-stated-bound`

> a pipeline whose last stage exits before its producer finishes under a program name this reader does not list

- **because**: the reader recognises `grep -q`, `grep -m` and `head`, which is what this shape looked like in the workflow's own history. The set of programs that exit early is not closed, and the question behind it -- does this stage read its input to EOF -- is not one a reader over shell text can answer. The three names close the door that was open rather than every door
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the early-exit consumers the pipeline reader names*

### `repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound`

> a figure about an enumerable set, written in a phrasing no census declares

- **because**: the declaration is the coverage — a census names the one sentence its figures are written in, and a count outside that sentence is unheld. Reaching it needs a judgement over prose, the instrument this repository designed, measured three times and rejected; `AGENTS.md` carries the other half as a rule with no check
- **its defence must show**: does not react
- **pinned by**: `a_count_in_an_undeclared_phrasing_is_a_stated_bound`

### `repository-checks/a-figure-written-in-words-at-one-hundred-or-above-is-not-matched-a-stated-bound`

> a declared census's figure spelled in words at one hundred or above

- **because**: the word reader covers the units, the tens, and one compound of the two, which stops at ninety-nine. The figures this repository writes in words are the small ones, and a set large enough to need three-digit words is one whose prose writes digits — so extending it upward buys nothing measurable, while a word reader that silently stops matching reads as covered
- **its defence must show**: does not react
- **pinned by**: `a_word_form_at_one_hundred_or_above_is_a_stated_bound`

### `repository-checks/a-gate-reached-without-the-wrapper-a-stated-bound`

> an act reaching cargo publish or a merge without going through its wrapper

- **because**: both assertions guard the sanctioned path -- the wrapper requiring its gate to report one passing test, and the check pinning the identifier it cites. Reaching further would mean observing the operator's shell or GitHub's servers rather than this repository
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*

### `repository-checks/a-refusal-constructed-outside-the-register-s-corpus-is-not-triaged-a-stated-bound`

> a refusal constructed by a gate implemented under `crates/kanhe/tests`, beside the directions over it

- **because**: the register reads `crates/kanhe/src`, where a construction is held by a named direction or declared unheld. A gate whose judgement and directions share a file has no answer to *which direction observes this branch*, because every direction in the file can see it -- so triaging it means first deciding where such gates belong, which is a question about their location rather than about this register
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *a gate that is its own test is outside the refusal register*

### `repository-checks/a-title-edited-inside-the-re-read-itself-a-stated-bound`

> a pull request title changing between the wrapper's post-gate re-read of it and `gh pr merge`

- **because**: the wrapper pins two of its three judged inputs by construction -- the body travels as the value the gate judged, and the commit set is pinned through `--match-head-commit`, which the server decides atomically. `gh` offers no equivalent for the title, so a re-read shrinks the exposure from a whole `cargo test` to one API call rather than closing it
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *the title race the wrapper can only narrow*

### `repository-checks/a-tool-configuration-set-in-the-environment-is-not-observed-a-stated-bound`

> a value a sanctioned wrapper refuses as an argument, exported into its environment instead

- **because**: the allowlist classifies ARGUMENTS, and cargo takes the same configuration from the environment -- measured on cargo 1.96.0, `--target not-a-real-triple` and `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical rustc-probe failure. Closing it is ordinary work here rather than another layer's, since the wrapper could scrub the environment before invoking cargo; it needs an allowlist over the environment, and legitimate setups export CARGO_HOME and CARGO_TARGET_DIR, so which set to admit is a decision this bound records instead of guessing
- **its defence must show**: does not react
- **pinned by**: `a_tool_configuration_set_in_the_environment_is_a_stated_bound`

### `repository-checks/a-whitespace-preceded-shell-marker-inside-quotes-is-cut-a-stated-bound`

> a shell comment marker preceded by whitespace inside a quoted string, where bash keeps it as string content and the token-start rule cuts it

- **because**: executed text is deleted, so a property about it is judged over less than the line carries — the direction the Core Contract forbids, and one a sentence in the classifier recorded as reaching the Rust region alone while both run the same rule
- **its defence must show**: does not react
- **pinned by**: `a_shell_marker_inside_quotes_is_cut_from_the_region`

### `repository-checks/files-no-capability-claims-a-stated-bound`

> a tracked file no capability's declared subject claims

- **because**: subjects are declared where a capability has something to say, and requiring them to tile the repository would buy coverage with a claim per capability that nobody could defend. The join reports how many tracked paths went unclaimed, so a clean verdict is not read as a complete one
- **its defence must show**: does not react
- **pinned by**: `files_no_capability_claims_are_reported_rather_than_implied_judged`

### `repository-checks/whether-a-mention-compiles-anything-is-not-observed-a-stated-bound`

> a promised prelude member the external contract names only in a comment

- **because**: the check asks whether the promise was noticed at all, and deciding that a mention is load-bearing is a judgement over text this repository has designed, measured and rejected. What makes a mention bite is the compiler; a comment-only mention still fails the reviewer reading the diff, which is the layer that owns it
- **its defence must show**: does not react
- **pinned by**: `a_member_named_only_in_a_comment_is_counted_as_named`

### `runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound`

> a seam whose only probe sits in a module reached by `#[path]` into a symlinked directory

- **because**: the root-file run reports the seam covered while the directory run reports it unprobed, so which entry point observed it decides the answer
- **its defence must show**: does not react
- **pinned by**: `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`

### `runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound`

> a seam whose only probe sits behind `#[cfg(test)]` or another non-production predicate

- **because**: the audit is cfg-blind and counts the probe as coverage, so a seam with no production probe reads as probed
- **its defence must show**: does not react
- **pinned by**: `production_probe_behind_non_production_cfg_is_counted_as_coverage`

### `self-law-projection/a-reason-carrying-the-clause-while-negating-the-law-is-not-observed-a-stated-bound`

> a `because` quoting 三儀 ⊥ 三儀 and then stating that it does not bind that dimension

- **because**: the check looks for the clause and not for what the sentence does with it, so the agent-loaded projection can carry the law's opposite while satisfying the check that exists to keep the law taught
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

### `self-law-projection/a-workspace-dependency-allowlist-is-not-examined-a-stated-bound`

> a dimension declaring the law through `restrict_workspace_dependencies_to` instead

- **because**: the filter admits one rule variant, while the variant it omits governs workspace-member edges specifically and is the more natural one for this law
- **its defence must show**: does not react
- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

### `semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound`

> a private import followed by a bare `pub use Foo;`, re-exported onward

- **because**: the closure captures inline `pub use` paths only, so the hop through a privately imported bare name is not followed
- **its defence must show**: does not react
- **pinned by**: `facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound`

### `semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound`

> `extern crate worklane_core as wc;` declared inside a module rather than at the crate root

- **because**: only crate-root renames are collected, since a module-scoped alias binds locally, so the alias head is not resolved to the crate it names
- **its defence must show**: does not react
- **pinned by**: `module_scoped_extern_crate_rename_is_a_stated_bound`

### `semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound`

> an `impl` nested one level deeper than the recovered position, or wrapped in a `static`

- **because**: only an `impl` directly in such a body is recovered, so a deeper or `static`-wrapped one exposes without being observed
- **its defence must show**: does not react
- **pinned by**: `an_impl_nested_one_level_further_stays_a_stated_bound`
- **pinned by**: `a_static_wrapped_impl_stays_a_stated_bound`

### `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound`

> a `cfg_if!` invocation inside an `impl` body exposing a forbidden type

- **because**: transparency covers item position, and an impl-body invocation's arms are impl items observed through different walkers — a declared gap rather than a claimed reaction
- **its defence must show**: does not react
- **pinned by**: `a_cfg_if_inside_an_impl_body_is_a_stated_bound`
