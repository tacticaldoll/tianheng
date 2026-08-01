## Context

`DeclaredModule::has_bare_cfg` is set at one place — the file-form (`mod name;`) branch of
`declared_modules_in` (`declarations.rs:188`) — from `has_bare_cfg_attr_before_item`, which scans the
attribute prefix immediately before the `mod` keyword. Two tolerance sites read it, both in
`walk.rs`: the plain absent-conventional-file case (`resolve_plain_sources`, line 243) and the absent
`#[path]` remap target case (line 369). One flag, two outcomes.

Transparency scope is already tracked. `TopLevelTracker` pushes a `MacroScope` when
`transparent_macro_body_at` matches (`cfg_if!` only) and pops it past its closing delimiter, and the
existing `is_top_level` gate — `active.inherited_top_level && active.macro_depth == 1` — already
restricts `mod` observation inside a macro body to a declaration sitting **directly in an arm brace**,
excluding one nested in an item body. That gate is what the five 0.2.3 follow-up fixes built.

So the information needed is present at the assignment site; what is missing is that the assignment
consults only the item's own attribute prefix.

## Goals / Non-Goals

**Goals:**
- Make arm membership a second source of cfg-conditionality, so the two spellings of a per-platform
  shim agree.
- Cover both absence outcomes (plain file, `#[path]` target) without touching either site, by fixing
  the flag they share.
- Name the flag for what it now means.

**Non-Goals:**
- Changing what is observed. An arm module whose file exists is reached and governed exactly as today.
- Weakening the ambiguity reaction. Both conventional forms present stays a constitution error under
  every gate, arm membership included.
- Evaluating `cfg`. Arm membership is a syntactic fact; no predicate is examined.
- 渾儀's and 漏刻's own equivalents. 渾儀 does not yet observe arm declarations at all, so it has
  nothing to diverge on until it does; that is the sibling change this one sequences before.

## Decisions

### Decision 1: Arm membership is read from the tracker, not re-derived

`TopLevelTracker` gains `in_transparent_macro(&self) -> bool` — a non-empty `macro_scopes` stack. The
file-form branch then sets the flag from `has_bare_cfg_attr_before_item(bytes, i) ||
top_level.in_transparent_macro()`.

Re-deriving arm membership by scanning backward for a `cfg_if!` header would duplicate the scope model
those five fixes exist to own, and would have to re-solve arm-versus-item-body depth. The tracker
already answers it at exactly the byte offset the declaration sits at, because `advance` pops expired
scopes before returning for that index.

The `is_top_level` gate is left untouched and remains the guard that a declaration is directly in an
arm: `mod` is only observed when it holds, and inside a macro scope it requires `macro_depth == 1`.
The two conditions compose — "observed at all" stays the tracker's existing answer, and "conditionally
compiled" becomes this new one.

### Decision 2: Rename `has_bare_cfg` to `is_cfg_conditional`

The flag's meaning changes from a syntactic observation about one attribute to a semantic one about
the declaration's compilation, now reachable two ways. Its doc comment also claims to be "the same one
hunyi's `has_cfg_attr` checks"; that stops holding, and the rename is what makes the divergence
visible instead of leaving a name that quietly misdescribes its contents. Nine mechanical sites,
crate-internal (`pub(super)`), no public surface.

### Decision 3: The ambiguity path is deliberately not consulted

`resolve_plain_sources` tests `flat.is_file() && nested.is_file()` and returns `Err` **before** it
reads the tolerance flag. That ordering already gives the correct result for an arm declaration — two
present files are unresolvable under every predicate — so no change is needed there, and the spec
scenario pinning it exists to keep that ordering from being "simplified" later.

## Risks / Trade-offs

- **[Trade-off] A previously-refused compilable crate becomes judged.** That is the point, and the
  direction is toward judging rather than away, so it cannot introduce a false negative: the tolerated
  declaration has no file, therefore no code to observe. An adopter who was seeing exit 2 on this shape
  will now get a real verdict, which may be exit 1 if the crate has genuine violations elsewhere that
  the aborted walk never reached.
- **[Risk] Over-tolerance if `in_transparent_macro` were true outside an arm.** The tracker pushes a
  scope only for a `cfg_if!` invocation and pops it at the matching close delimiter, and `mod` is only
  observed under `is_top_level`. A `mod` between arms is not valid Rust, so "inside a transparent macro
  body and top-level" is arm membership. Pinned by the sibling scenario asserting an unconditional
  missing file outside any macro still errors. One shape does slip through and is accepted as a stated
  bound: a malformed `cfg_if! { mod x; }` with no `if` header has no real gate, yet would read as
  cfg-conditional. That source does not compile — `cfg_if!` requires the `if` form — so the over-
  tolerance can only apply to a crate that already cannot build.
- **[Trade-off] The rename inflates the diff.** A behavior change and a nine-site mechanical rename
  land together, which is harder to review than either alone. Accepted because the alternative leaves
  a name that is actively false — `has_bare_cfg` returning true for a declaration with no bare `#[cfg]`
  anywhere near it — and a wrong internal name is the seed of the next drift, in a codebase whose doc
  comments carry this much of the reasoning.
- **[Why this is not the P4 mistake]** A sibling proposal in this area was withdrawn for resting on an
  unevidenced population (other transparent macros, for which no adopter usage could be shown). The
  distinction: that proposal *added a general mechanism* to cover a hypothetical population, at a
  measured cost (an `impl` body's braces read as an arm, inventing items). This one *removes a
  contradiction* between two spellings of one intent, at no cost. A contradiction is self-evidencing —
  it does not need an adopter to report that two forms of the same declaration get opposite verdicts —
  and the drift law governs naming reactions that do not exist, not repairing reactions that disagree.
- **[Observation source] The failing shape was constructed, not reported.** No adopter reported it, and
  the common `cfg_if!` per-platform shim commits both arm files, which already worked (measured exit
  0). What justifies the change is not a predicted population but the removal of an inconsistency: two
  spellings of one intent, one accepted and one refused, with the refusal landing on source that
  compiles. The fix carries no offsetting cost — an absent file holds no code — so there is no
  trade-off to defer for evidence.
