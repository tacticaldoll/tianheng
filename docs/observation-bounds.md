# Observation bounds

Every **observation bound** this family declares: a claim that a reaction deliberately stops at a
named shape, so that shape is governed policy rather than a defect.

**8 of 41 declared bounds have no pinning test.** That figure is the register's
audit backlog and leads the document because a number in a footnote is not read. Each such bound names
the tracker that owns closing it.

Generated from `openspec/specs/*/spec.md` by `scripts/check_bound_register.sh`. **Do not edit by hand** —
regenerate with `BLESS=1 bash scripts/check_bound_register.sh`. A stale projection fails that gate.

**What this document does not claim.** It lists the bounds the specs *state in a recognizable form*: a
scenario whose heading marks it a bound. A bound worded outside that form — "out-of-scope", "does not
claim to observe" — is invisible to the scan that assembles this, so the list is a floor rather than a
proof of completeness. A register that implied otherwise would mislead exactly where it is most
trusted.


## crate-source-boundary

### `crate-source-boundary/a-git-plus-version-dependency-is-flagged-though-it-would-publish-a-stated-conservative-bound`

> the system classifies it as `Git` (its declared source is `git+…`) and emits a violation — even though such a dependency would `cargo publish` successfully — because the rule governs the declared source kind, a stated conservative bound, not publish-eligibility

- **pinned by**: `source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist`

## external-crate-confinement

### `external-crate-confinement/cfg-gated-code-is-observed-as-written-a-stated-scanner-bound`

> the system observes it as written rather than evaluating the predicate, so the reaction is cfg-blind — inherited from the module scanner and stated here, never a silent claim about which branch is live

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "declared bounds with no pinning test"

### `external-crate-confinement/the-lib-and-bin-conventional-path-conflation-is-a-stated-scanner-bound`

> the system does not distinguish their module graphs — the conflation inherited from the module scanner, stated rather than silently resolved

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "declared bounds with no pinning test"

### `external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-coverage-bound`

> the system reports no violation, because comments, string literals, and macro bodies are stripped before scanning, matching the scanner's stated bounds

- **pinned by**: `confine_ignores_a_use_inside_a_string_literal`

## inline-symbol-path-confinement

### `inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-adopter-owned-bound`

> the system does NOT react (a false negative the adopter owns by narrowing), rather than the engine silently guessing which verbs are reads

- **pinned by**: `inline_a_verb_outside_the_declared_set_is_a_bound`

### `inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound`

> the system does not claim to observe it (no type inference on the receiver) — a stated bound, not a silent assertion of cleanliness

- **pinned by**: `inline_receiver_method_read_is_a_bound`

### `inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default`

> the system does not react (value-position mention is a stated bound under the default; `.strict_prefix_only()` catches it) — declared, not silent

- **pinned by**: `inline_value_capture_is_a_bound_under_the_default`

### `inline-symbol-path-confinement/an-external-crate-re-export-is-a-documented-bound`

> the system does not claim to observe it (foreign AST is not scanned) — a stated bound

- **pinned by**: `inline_foreign_reexport_of_the_confined_path_is_a_bound`

### `inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external`

> the system does not claim to observe the call through the `chr` alias head (the use-map reads `use` only; the `extern crate … as` rename is a stated bound even under strict-external), never a silent assertion of cleanliness

- **pinned by**: `inline_strict_external_extern_crate_rename_is_a_stated_bound`

### `inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default`

> the system does NOT react (the fully-qualified un-`use`d external call is a stated non-observation under the default; behavior is unchanged from before this capability)

- **pinned by**: `inline_strict_external_absent_fully_qualified_call_is_a_bound`

## runtime-origin-assertion

### `runtime-origin-assertion/source-outside-a-member-s-library-or-binary-target-subtree-is-out-of-scope-a-stated-corpus-bound`

> the audit does not read it, its corpus being the member's library and binary targets — a stated bound shared with the semantic dimension, never a silent claim of coverage

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "declared bounds with no pinning test"

### `runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-cfg-blind-bound`

> the audit counts it as coverage, being cfg-blind, so a seam whose production probe lives there is reported as probed — a stated bound, never a silent pass

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "declared bounds with no pinning test"

### `runtime-origin-assertion/identical-expression-repeated-in-the-same-function-collapses-to-one-finding-a-stated-identity-bound`

> `audit_probe_coverage` emits one un-auditable-probe violation for that site — a stated bound, since no further source content distinguishes the two occurrences

- **pinned by**: `identical_expression_repeated_in_the_same_function_collapses_to_one_violation`

### `runtime-origin-assertion/an-absolute-path-literal-s-target-outside-the-anchor-keeps-its-absolute-label-a-stated-label-bound`

> `audit_probe_coverage` still emits the un-auditable-probe violation, naming the site with the raw absolute path — a stated bound, since the literal has no textual relationship to the anchor

- **pinned by**: `an_absolute_path_literal_outside_the_anchor_keeps_the_path_the_literal_wrote`

### `runtime-origin-assertion/a-composite-shape-yields-a-truncated-origin-a-stated-shape-bound`

> the derived origin is a truncated rendering that equals no module name, so it matches no allowlist entry and in particular never the wrapped type's own defining module; the crossing reacts fail-closed rather than being admitted through the wrapper

- **pinned by**: `the_derived_origin_honors_its_stated_shape_bounds`

### `runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-corpus-bound`

> the root-file run reports the seam covered, while the directory run reports it unprobed — the

- **pinned by**: `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`

## semantic-async-exposure-boundary

### `semantic-async-exposure-boundary/a-body-nested-module-is-a-stated-bound`

> the system does not observe `hidden` — a `mod` inside a fn body is not public API (not reachable as `crate::…`), a stated bound, never a silent claim about it

- **pinned by**: `async_subtree_does_not_observe_a_body_nested_module`

## semantic-dyn-trait-boundary

### `semantic-dyn-trait-boundary/a-public-item-naming-such-an-alias-is-not-expanded-a-stated-coverage-bound`

> the system reacts at the alias declaration but emits **no additional** reaction for `make` via alias expansion — the `dyn` is already caught at the alias, and `type` aliases are not expanded (a stated bound)

- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-boundary/a-private-alias-hiding-a-dyn-in-a-public-position-is-a-stated-bound`

> the system does not claim to observe the hidden `dyn` (a stated coverage bound — the resolver does not expand `type` aliases), rather than silently asserting the boundary is clean

- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-boundary/a-macro-generated-dyn-is-a-documented-coverage-bound`

> the system does not claim to observe it (the universal 渾儀 macro-expansion bound), rather than silently asserting the boundary is clean

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "three declared bounds have no pinning test"

### `semantic-dyn-trait-boundary/an-unrenderable-sub-node-is-a-stated-rendering-bound`

> the system does not claim to distinguish them: they share a canonical `subject` field and key at the same seam (each still *reacts* on first occurrence; only baseline-dedup granularity is bounded). This is a **stated subject-rendering bound** — the same `(target, rule_key, fact)` granularity bound `semantic-trait-impl-locality`'s `(impl for <self_ty>)` fact carries — declared here, never a silent claim of cleanliness

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "three declared bounds have no pinning test"

## semantic-dyn-trait-operand-boundary

### `semantic-dyn-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> the system does not resolve the principal and reports no violation — a stated resolver-coverage bound (the oracle does not over-reach a single bare segment), never a silent claim of cleanliness over a resolvable operand

- **pinned by**: `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound`

## semantic-forbidden-marker

### `semantic-forbidden-marker/an-unresolvable-hand-impl-self-type-is-a-documented-bound`

> the system does not claim to observe it (a stated coverage bound), rather than silently asserting cleanliness — the co-located, `use`-imported, re-export-spelled, and type-alias cases (the common ones) do resolve and react

- **pinned by**: `an_unresolvable_glob_self_type_is_a_documented_bound`

## semantic-impl-trait-operand-boundary

### `semantic-impl-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> the system does not resolve the principal and reports no violation — a stated resolver-coverage bound, never a silent claim over a resolvable operand

- **pinned by**: `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound`

## semantic-reexport-exposure

### `semantic-reexport-exposure/an-underscore-rename-is-a-documented-non-observed-bound`

> the system does not react — `as _` binds no nameable path a consumer can reach — and this is a stated bound, not a silent claim of cleanliness

- **pinned by**: `restricted_and_private_and_underscore_reexports_do_not_react`

### `semantic-reexport-exposure/a-sibling-root-glob-is-a-documented-bound`

> the system does not claim to observe the transitively re-exported leaf (the inherited glob bound — the glob's leaves are not enumerable here), rather than silently asserting the boundary is clean

- **pinned by**: `sibling_root_glob_does_not_react`

### `semantic-reexport-exposure/an-ancestor-root-glob-spanning-a-deeper-forbidden-prefix-is-a-documented-bound`

> the system treats it as a stated bound (it cannot enumerate whether `crate::infra` publicly re-exports the forbidden `db` subtree), documented as the sharper ancestor-root sub-case rather than lumped with the innocent sibling glob or silently claimed clean

- **pinned by**: `ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react`

### `semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound`

> the system does not follow that hop (the closure captures only inline `pub use` paths) and this is a documented inherited bound, not silently claimed clean

- **pinned by**: `facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound`

### `semantic-reexport-exposure/a-non-forbidden-root-external-glob-is-a-documented-bound`

> the system does not claim to observe the glob's individual leaves, rather than silently asserting the boundary is clean

- **pinned by**: `extern_glob_nonforbidden_root_is_a_stated_bound`

### `semantic-reexport-exposure/a-re-export-renamed-through-a-foreign-module-is-a-documented-bound`

> the system matches only the written path (`worklane_core::prelude::Foo`, not in/under the forbidden set) and does not silently claim to have followed the foreign chain

- **pinned by**: `foreign_prelude_rename_is_a_stated_bound`

### `semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound`

> the system does not resolve `wc` to `worklane_core` (only crate-root renames are collected, since a module-scoped alias binds only locally) and this is a documented bound, distinct from the handled crate-root rename

- **pinned by**: `module_scoped_extern_crate_rename_is_a_stated_bound`

## semantic-signature-coupling

### `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound`

> the system reports no exposure — transparency covers item position, where an arm's contents are items; an `impl`-body invocation's arms are impl items, observed through different walkers, and that remains a declared gap rather than a claimed reaction

- **pinned by**: `a_cfg_if_inside_an_impl_body_is_a_stated_bound`

### `semantic-signature-coupling/a-macro-under-another-name-is-not-treated-as-transparent-a-stated-coverage-bound`

> the system reports no exposure — the invocation is not transparent, so its body stays a stated coverage bound; extracting from it would read the `impl` body's braces as an arm and report an item the macro may never emit

- **pinned by**: `an_arbitrary_macro_body_is_not_read_as_transparent_arms`

### `semantic-signature-coupling/a-plain-item-nested-the-same-way-stays-a-stated-bound`

> the system reports no exposure — only an `impl` block is recovered from a body this way; a plain item nested the same way is genuinely scoped to that body and unreachable as `crate::…`, exactly like the existing body-nested-module bound, so it stays unobserved rather than a new, unaudited claim

- **pinned by**: `a_plain_fn_directly_in_a_const_body_stays_a_stated_bound`

### `semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound`

> the system reports no exposure for that impl — a stated coverage bound rather than a silent claim of cleanliness

- **pinned by**: `an_impl_nested_one_level_further_stays_a_stated_bound`
- **pinned by**: `a_static_wrapped_impl_stays_a_stated_bound`

## semantic-trait-impl-exposure

### `semantic-trait-impl-exposure/a-glob-imported-type-in-an-impl-position-is-a-documented-coverage-bound`

> the system does not claim to observe it (inherited glob bound), rather than silently asserting the boundary is clean

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "three declared bounds have no pinning test"

## semantic-trait-impl-locality

### `semantic-trait-impl-locality/a-macro-generated-impl-is-a-documented-coverage-bound`

> the system does not claim to observe it (out of scope, the same nature as the existing macro bound), rather than silently asserting the boundary is clean

- **pinned by**: `a_macro_generated_impl_is_a_documented_bound`

### `semantic-trait-impl-locality/a-cfg-gated-module-with-an-absent-file-is-skipped-not-a-scan-error-a-stated-coverage-bound`

> the whole-crate walk skips the module (a stated coverage bound) rather than failing the gate with a scan error (exit 2)

- **pinned by**: `hunyi::a_cfg_gated_module_with_no_file_is_skipped_not_errored`

## semantic-unsafe-confinement

### `semantic-unsafe-confinement/macro-generated-unsafe-is-a-documented-bound`

> the system does not claim to observe it (out of scope, the dimension's macro bound), rather than silently asserting the module is unsafe-free

- **pinned by**: `unsafe_in_a_macro_body_is_a_stated_bound`

## semantic-visibility-boundary

### `semantic-visibility-boundary/a-macro-generated-item-is-a-documented-bound`

> the system does not claim to observe it (out of scope, the same nature as the dimension's existing macro bound), rather than silently asserting the module is clean

- **pinned by**: `a_macro_invocation_pub_item_is_a_documented_bound`

### `semantic-visibility-boundary/a-pub-in-narrow-path-item-may-over-react-under-a-tight-ceiling-a-stated-over-reaction-bound`

> the system MAY react (the conservative `Crate` rank exceeds the `Module` ceiling), a stated over-reaction bound, never a silent pass

- **unpinned**, tracked by: BACKLOG.md READY-PATCH "three declared bounds have no pinning test"
