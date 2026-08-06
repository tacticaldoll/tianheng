# Observation bound extents

Where each declared **observation bound** stops the measure — not how far a scan walks (that is
`ScanDepth`, an adopter's knob), but where this family's own reaction deliberately stops.

**11 of 42 declared bounds are declared false negatives** — the reaction fires less than the truth, which is the one direction this family treats as a defect. That figure leads this document because a number in a footnote is not read, and each such bound names who must act:

- `external-crate-confinement/an-extern-crate-declaration-is-not-observed-a-stated-bound` — owner: engine
- `inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-bound` — owner: adopter
- `inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default` — owner: adopter
- `inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external` — owner: engine
- `inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default` — owner: adopter
- `runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound` — owner: inherited from the corpus entry point
- `runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound` — owner: engine
- `semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound` — owner: engine
- `semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound` — owner: engine
- `semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound` — owner: engine
- `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound` — owner: engine

Generated from each dimension's `observation_bounds()` by `crates/tianheng/tests/observation_bound_model.rs`. **Do not edit by hand** — regenerate with
`BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test observation_bound_model`.

**What this document does not claim.** The classification is *authored*: the type refuses a contradiction and derives what each bound's defence must demonstrate, but nothing verifies that a bound recorded as over-reacting really over-reacts rather than under-reacting. Two further limits are declared as bounds of `observation-bound-model` itself: an answer that depends on which corpus entry point observed it has no extent of its own and is recorded as under-reacting with the entry point as its owner, and a bound both out of reach and granularity-limited cannot be expressed at all.

One value carries no bound today and is kept deliberately: **refuses to judge**. The misclassification this model exists to prevent was exactly a confusion between that and *out of reach* — a prediction of a silent false negative where the real behaviour was a fail-loud refusal — and a direction that cannot be named cannot be predicted with.

## as intended, granularity bounded (5)

### `external-crate-confinement/the-lib-and-bin-conventional-path-conflation-is-a-stated-bound`

> a package whose library and binary conventional source paths coincide

- **because**: the two targets' module graphs are not told apart, so a finding names one compilation unit where two share a path — the reaction still fires
- **its defence must show**: collapses granularity
- **pinned by**: `confine_external_crate_conflates_coincident_lib_and_bin_conventional_paths`

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

## out of reach (18)

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

## over-reacts (4)

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

### `runtime-origin-assertion/a-composite-shape-yields-a-truncated-origin-a-stated-bound`

> a registered type that is a reference, tuple, array, pointer, or function pointer

- **because**: the derived origin is a truncated rendering matching no module name, so the crossing reacts fail-closed rather than being admitted through the wrapper
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `the_derived_origin_honors_its_stated_shape_bounds`

### `semantic-visibility-boundary/a-pub-in-narrow-path-item-may-over-react-under-a-tight-ceiling-a-stated-bound`

> `pub(in crate::a) fn` on an item already directly in `crate::a`, under a `Module` ceiling

- **because**: the conservative `Crate` rank exceeds the `Module` ceiling, so an effectively private item may react — never a silent pass
- **its defence must show**: reacts on a harmless shape
- **pinned by**: `a_pub_in_narrow_path_over_reacts_under_a_module_ceiling`

## under-reacts (11)

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
- **pinned by**: `a_static_wrapped_impl_stays_a_stated_bound`

### `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound`

> a `cfg_if!` invocation inside an `impl` body exposing a forbidden type

- **because**: transparency covers item position, and an impl-body invocation's arms are impl items observed through different walkers — a declared gap rather than a claimed reaction
- **its defence must show**: does not react
- **pinned by**: `a_cfg_if_inside_an_impl_body_is_a_stated_bound`
