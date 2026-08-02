## Context

`module_findings` (exposure.rs) and `scan.rs`'s crate-wide `collect_reexports` closure both resolve
a bare `pub use dep::X;` re-export head against the crate's external-crate name set, with the
governed/defining module's own child-**module** names subtracted first (`externs − child_mods`) —
rustc really does let a local `mod dep` shadow the extern prelude's `dep` inside that module. The
same two call sites also resolve a bare head against a crate-root `extern crate X as Y;` rename map
first (before the externs-set fallback), with the identical per-module child-module subtraction
(`renames − child_mods`) — the spec's own text ties the two together as one exclusion applied to
"both" sets, in both places.

`change/hunyi-cfg-branch-use-reexport-merging` (PR #149) already fixed the case where the child-mod
subtraction was computed once over the UNION of two mutually-exclusive **branches** (two competing
resolutions of the governed module ITSELF — an inline-vs-file-form split, or two inline siblings of
the same name) — grouping by branch index instead closed that. It explicitly left one line out of
scope: two mutually-exclusive **items within the same branch's own file** (a `#[cfg(unix)] mod
serde;` beside a `#[cfg(not(unix))] pub use serde::Value;`, or the two arms of one `cfg_if!`) still
share the identical branch and file, so the existing branch-level fix is a no-op for them — the
child-module-name set is still computed cfg-blind across BOTH items, and the `mod` suppresses the
`pub use` even though the two can never compile together.

**Revision note (round 2 of this change):** the first round fixed only the external-crate-name half
of the exclusion (`externs_reexport`/`reexport_externs_for`) and explicitly named the rename-map
half (`renames_bare`) a Non-Goal, on the assumption it needed independent justification and
reproduction before claiming in scope. An independent adversarial review of that first round
reproduced the identical false-negative shape through the rename-map half instead
(`extern crate serde as wc;` + `#[cfg(unix)] mod wc;` + `#[cfg(not(unix))] pub use wc::Value;`) and
found it sharper than under-shadowing: `extern_verbatim_renamed` checks the rename map **before**
the externs-set fallback, and an alias spelling is never itself a member of the externs set, so a
cfg-blindly-shadowed rename alias drops the resolution outright rather than merely leaving it
shadowed. The review also found the round-1 doc trail had drifted: `exposure.rs:157`'s own comment
pointed at "the module doc" for a rationale no module doc carried, and the synced spec text's
antecedent read as covering both halves while its operative sentence named only the extern set. This
document folds the rename-map fix into the SAME change rather than deferring it as a fresh residual
finding — the mechanism (`provably_mutually_exclusive`) already exists and applies to the rename map
exactly as it does to the extern-name set, so deferral would only relabel a known, reproduced,
directly-fixable gap as tracked debt.

## Goals / Non-Goals

**Goals:**
- The child-module shadow (both at the direct head and inside the crate-wide re-export closure)
  stops suppressing a `pub use` when the shadowing `mod` is provably never compiled alongside it —
  on **both** halves the spec ties together: the external-crate-name set and the crate-root
  rename-alias map.
- Cover the two shapes the audit trigger and the spec's own rustc rationale actually name: a bare
  `#[cfg(P)]` / `#[cfg(not(P))]` negation pair, and two arms of one `cfg_if!` invocation.
- Keep every other cfg-branch resolution (the `use`-map, `externs_type`, and `renames_bare` **as
  applied to a type-position head**) untouched — this change is scoped to the re-export head's own
  shadow (both halves) and its crate-wide-closure sibling, not a general cfg-satisfiability engine
  and not the type-position shadow.

**Non-Goals:**
- Proving exclusion between two *unrelated* predicates (`cfg(windows)` vs `cfg(target_os =
  "macos")`), between arms of two *different* `cfg_if!` invocations, or across more than one bare
  `#[cfg]` attribute stacked on an item. These stay the pre-existing cfg-blind "may coexist"
  default — a stated residual bound (mirroring the spec's other documented residual bounds), not a
  guess. A general cfg-predicate SAT solver is out of scope for this dependency-light dimension.
- Extending the SAME cfg-mutual-exclusion awareness to `externs_type`/a type-position head's own
  `renames_bare` use (`semantic-signature-coupling`'s type-position shadow). Not named in the
  audited finding or its rename-map sibling, each would need its own reproduction, and is left as a
  candidate follow-up if ever independently found — unlike the rename-map **re-export** shadow this
  revision folds in, a type-position head's shadow was never claimed fixed by this change's own
  spec text (the "External-crate re-exports are observed by default" requirement governs re-exports
  only), so there is no matching broken-doc-trail risk in leaving it be.

## Decisions

- **Mutual exclusion is proven, not guessed, via two syntactic shapes only.** `cfg_if!`'s arms are
  exclusive by construction (only one predicate in the chain is ever true); an exact `not(P)`/`P`
  bare-attribute pair is the one general syntactic negation cheap to detect without a SAT solver.
  Detecting this requires comparing two parsed `cfg` predicates structurally — `syn::Meta` carries
  no `PartialEq` for its `Meta::List` token-stream payload, so a small recursive `meta_eq` (Path /
  List / NameValue) is added, reusing the existing `cfg_attr_metas` parser rather than a second one.
  Alternative considered: comparing predicates as rendered strings — rejected, since it would treat
  `not(unix)` and `not( unix )` as different literals for no reason `cfg` itself cares about.
- **`cfg_if!` arm identity, not just membership, must be threaded through.** The pre-existing
  `FlatItem.in_transparent_arm: bool` only recorded "reached through some arm," not which one — not
  enough to tell two arms of one invocation apart from two arms of two unrelated invocations. Added
  a private `ArmKey { invocation, arm }` (a monotonic per-invocation counter assigned during
  flattening) alongside the existing boolean, set together by the same constructor so the two can
  never drift apart. A nested `cfg_if!`'s own innermost key wins over an enclosing arm's, since an
  item's most specific arm membership is the one its true sibling-exclusivity check needs; comparing
  across nesting levels (an outer arm's item vs. a doubly-nested inner arm's item) is not attempted
  and stays a residual bound — narrower than the audited trigger, so left unclaimed rather than
  guessed at.
- **The shadow becomes per-*exposure-item*, not per-branch — for BOTH the externs set and the
  rename map.** `externs_reexport` was a single `HashSet<String>` computed once per branch; it is
  replaced with `reexport_externs_for(externs, mod_decls, use_flat)`, called once per re-export
  exposure with that exposure's OWN originating item's tag. The rename map gets the identical
  treatment via a new `reexport_renames_for(renames, mod_decls, use_flat)` — structurally the same
  function, over `HashMap<String, String>` instead of `HashSet<String>`, since `ExternRenameMap` is
  a plain type alias for that map (no wrapping type, so no conversion needed at call sites).
  `mod_decls: Vec<(String, FlatItem)>` (built by `child_module_decls`) replaces the flat
  `child_mods: HashSet<String>`/`HashSet<&str>` wherever either shadow is computed, so a same-named
  module declared under two different cfg-gatings is tested individually rather than collapsed to
  one name. `module_findings` now sources its items via a new `resolve_module_items_with_cfg_tags`
  (returning `FlatItem`, not a bare `syn::Item`) instead of `resolve_module_items_with_files` — a
  parallel function, not a signature change to the existing one, since six other capabilities
  consume `resolve_module_items_with_files` and have no need for arm identity. A type-position
  head's `renames_bare` keeps the OLD, branch-wide, cfg-blind computation unchanged (an explicit
  Non-Goal, matching `externs_type`'s own scope) — the `Cow<HashMap<String, String>>` selecting
  between the two (owned per-item for a re-export, borrowed branch-wide otherwise) mirrors the
  existing `Cow<HashSet<String>>` already used for `externs_type`/`externs_reexport`.
- **The crate-wide closure (`scan.rs`/`collect_reexports`) gets the identical treatment**, not left
  cfg-blind, on both halves. The spec's own existing text already requires the child-module
  exclusion applied "both to the direct re-export head resolution and inside the crate-wide
  re-export closure," and "to both the external-crate set ... and the crate-root rename map ...
  exactly as the direct head does" — reproducing a facade-chain variant of each trigger (a governed
  module's `pub use` reaching the mutually-exclusive pair through another module) confirmed the
  closure has the identical gap for both. `collect_reexports` now takes `items: &[FlatItem]` and
  `child_mods: &[(String, FlatItem)]` instead of `&[syn::Item]` / `&HashSet<String>`, and computes
  BOTH `externs_bare` and `renames_bare` per use-item (dropping the old once-per-call
  `renames_shadowed` computation entirely, since every consumer of the rename map inside this
  function is now per-item); `walk_module` already computes `flat: Vec<FlatItem>` for the
  child-resolution absence check, so it is threaded into `collect_reexports` too instead of the
  flattened, cfg-blind plain list.
- **`resolve/mod.rs` gains its first dependency on `syn_util`.** `resolve/mod.rs` previously
  imported nothing from a sibling `hunyi` module (a `syn`-based but otherwise self-contained
  resolution layer); it now imports `FlatItem`/`reexport_externs_for`/`reexport_renames_for`.
  `syn_util.rs` already imports `crate::resolve::strip_raw`, so this is a two-way `use` relationship
  between sibling modules within one crate — legal and unproblematic in Rust (unlike the crate-level
  `guibiao ⊥ tianheng` boundary this project's self-governance actually enforces), and the more
  contained alternative (re-deriving the per-item exclusion inside `scan.rs` before calling an
  unchanged `collect_reexports`) would have needed `collect_reexports` restructured to accept two
  pre-resolved per-use-item maps anyway, without avoiding the dependency.
- **Doc-trail hygiene: no more cross-references to a "module doc" that doesn't carry the claim.**
  Round 1 left a comment in `exposure.rs` reading "see the module doc for why that stays a stated
  bound here" for the (then cfg-blind) rename-map case — a dangling cross-reference once the
  claimed rationale is not written down anywhere else, flagged by the same review. The comment is
  rewritten in place to state directly why the type-position `renames_bare` stays cfg-blind (an
  explicit, narrower Non-Goal than the round-1 text implied) rather than pointing elsewhere for a
  rationale that was never actually recorded.

## Risks / Trade-offs

- [Stacked bare `#[cfg]` attributes, or predicates related only semantically (`cfg(unix)` vs
  `cfg(target_family = "unix")`) are not proven exclusive] → conservative default (unchanged,
  cfg-blind shadow) preserves the pre-existing behavior exactly; no NEW false negative is
  introduced by under-covering these, only the pre-existing one for the two now-covered shapes.
- [`FlatItem` cloning per branch/mod-decl adds allocation over the previous flat `HashSet<String>`]
  → this dimension is an offline CI-time analysis tool, not a hot path; correctness and clarity are
  prioritized over the micro-cost of cloning a handful of `syn::Item`s per module.
- [A future capability adding its own child-module shadow (e.g. hardening the type-position
  `renames_bare`/`externs_type` the same way) must remember this same per-item pattern rather than
  the old per-branch one] → the `provably_mutually_exclusive`/`reexport_externs_for`/
  `reexport_renames_for`/`child_module_decls` quartet in `syn_util.rs` is written to be reused, not
  `exposure.rs`-private, precisely so a future fix does not re-derive it.
- [A spec's antecedent ("this exclusion ... applies to both X and Y") can silently drift ahead of
  or behind its own operative sentences as a change is revised in rounds] → this revision's spec
  delta states each halved case explicitly in its own sentence rather than relying on a shared
  antecedent to carry both, so a future partial fix cannot leave the same ambiguity again.
