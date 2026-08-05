# Enumeration (tasks 1.1–1.4)

Working note, consumed by task 3 and pruned with this change. Recorded here rather than left in a
conversation, because a negative result — "this bound has no pinning test" — has no other resting place
until it becomes a register citation.

## Declared bound scenarios: 24

Was 25; `rule-model-surface`'s "A name living in two namespaces resolves to its module reading (stated
bound)" was retired in `4d7d93c` as a contradiction of the reacting scenario in its own requirement.

## Pinned, citation verified against the test's own fixture

| Bound | Pinning test |
|---|---|
| `inline-symbol-path-confinement` / receiver-method read | `inline_receiver_method_read_is_a_bound` |
| `inline-symbol-path-confinement` / path taken as a value | `inline_value_capture_is_a_bound_under_the_default` |
| `inline-symbol-path-confinement` / fully-qualified external call | `inline_strict_external_absent_fully_qualified_call_is_a_bound` |
| `inline-symbol-path-confinement` / extern-crate rename under strict-external | `inline_strict_external_extern_crate_rename_is_a_stated_bound` |
| `semantic-async-exposure-boundary` / body-nested module | `async_subtree_does_not_observe_a_body_nested_module` |
| `semantic-dyn-trait-operand-boundary` / unresolvable bare principal | `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound` |
| `semantic-forbidden-marker` / unresolvable hand-impl self-type | `an_unresolvable_glob_self_type_is_a_documented_bound` |
| `semantic-forbidden-marker` / impl nested one level further or static-wrapped | `an_impl_nested_one_level_further_stays_a_stated_bound` |
| `semantic-reexport-exposure` / sibling-root glob | `sibling_root_glob_does_not_react` |
| `semantic-reexport-exposure` / ancestor-root glob over a deeper prefix | `ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react` |
| `semantic-reexport-exposure` / facade hop re-exporting a privately-used bare name | `facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound` |
| `semantic-reexport-exposure` / non-forbidden-root external glob | `extern_glob_nonforbidden_root_is_a_stated_bound` |
| `semantic-reexport-exposure` / re-export renamed through a foreign module | `foreign_prelude_rename_is_a_stated_bound` |
| `semantic-reexport-exposure` / module-scoped extern-crate rename | `module_scoped_extern_crate_rename_is_a_stated_bound` |
| `semantic-signature-coupling` / invocation inside an impl body | `a_cfg_if_inside_an_impl_body_is_a_stated_bound` |
| `semantic-signature-coupling` / plain item nested the same way | `a_plain_fn_directly_in_a_const_body_stays_a_stated_bound` |
| `semantic-signature-coupling` / impl nested one level further or static-wrapped | `an_impl_nested_one_level_further_stays_a_stated_bound` |
| `semantic-trait-impl-locality` / impl nested one level further | `an_impl_nested_one_level_further_stays_a_stated_bound` |
| `semantic-trait-impl-locality` / static-wrapped impl | `a_static_wrapped_impl_stays_a_stated_bound` |
| `semantic-unsafe-confinement` / macro-generated unsafe | `unsafe_in_a_macro_body_is_a_stated_bound` |
| `semantic-visibility-boundary` / macro-generated item | `a_macro_invocation_pub_item_is_a_documented_bound` |
| `semantic-dyn-trait-boundary` / private alias hiding a dyn | needs re-check: the nearest candidate asserts the **opposite** direction (`..._reacts`) |

Two tests are each cited by more than one capability's bound —
`an_impl_nested_one_level_further_stays_a_stated_bound` by three — which settles design open question 1:
one behaviour of one dimension can bound several of its capabilities, and each is defined exactly once
(`crates/hunyi/src/tests/macro_and_body_nested.rs`), so the exactly-one-definition rule is satisfied.

## Unpinned — the register's opening debt

| Bound | Why nothing pins it |
|---|---|
| `inline-symbol-path-confinement` / `#[path]`-remapped file in the subtree | An *inherited* scanner bound. Its canonical statement is prose in `external-crate-confinement`'s requirement (`#[path]`-remapped modules … remain the scanner's stated out-of-scope bounds), and no test exercises the remap-in-subtree shape for this rule. |
| `inline-symbol-path-confinement` / external-crate re-export | No test found for a foreign crate re-exporting the confined path. |
| `semantic-impl-trait-operand-boundary` / unresolvable bare principal | Only the **dyn**-operand twin is pinned. The resolver is shared (`hunyi::resolve`), so the dyn test could be borrowed — deliberately not borrowed, because the register's value is telling the truth about which surface is exercised. The honest entry is unpinned plus a tracker for writing the impl-trait twin. |

## Prose-declared bounds the floor will surface, with their disposition

- `inline-symbol-path-confinement` UFCS-qualified call — prose only, but **already pinned** by
  `inline_ufcs_is_a_documented_bound_under_the_default`. Promoting it to a scenario is a clean win.
- `external-crate-confinement`'s four scanner out-of-scope bounds, stated in one sentence
  (`#[path]`-remapped modules, `cfg_attr`-wrapped path attributes, cfg-gated code, lib+bin
  conventional-path conflation). One sentence, four bounds: each needs its own scenario.
- `rule-model-surface`'s residual left after `4d7d93c` — a value arriving through a macro body or a
  re-export is unobserved, which directs the reaction to the module reading. Stated today only inside a
  struck-through `BACKLOG.md` entry, which is exactly why it was mis-reported as a false negative during
  the sweep that preceded this change. Pinnable from the sweep's own fixture.

## Task 1.3 — the floor's pattern, derived from what occurs

`(stated|documented) bounds?` over `openspec/specs/*`, counting an occurrence as cleared when it lies
inside a bound scenario. Measured: 55 matching lines, 24 of them declaring scenario headings, 4 requirement
headings, 29 prose or bare THEN clauses. Known misses of this pattern, to be registered as the register's
own residual: a bound worded as "out-of-scope", "not observed", or "does not claim to observe" without the
word *bound*.

## Task 1.4 — settled

A bound shared by two capabilities is registered **once per capability**, each citing the same test.
Evidence above: three capabilities declare the body-nested-impl-depth bound separately, and one test
defends all three. Registering once would leave two capabilities' specs silent about a bound they have.
