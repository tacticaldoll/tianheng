# Change: the observer-protocol equality reaction covers every arm

## Why

`observer-protocol` promises that its two composition paths cannot disagree silently. Measured, its reaction
proves that for **one** of the three dimensions.

Three defects, each measured rather than reasoned:

1. **Two of three arms are vacuous.** The equality fixture declares a single violated `CrateBoundary`. Its
   semantic and runtime boundary sets are therefore empty, and an empty declaration is *Clean* on this
   workspace — measured: `check_constitution(&Constitution::new("probe"), &workspace)` returns `Clean`. So
   both the semantic and the runtime arms contribute nothing to either side of the comparison, and either one
   could be short-circuited to `Clean` without the reaction noticing. Measured directly: replacing
   `SemanticObserver::observe`'s body with `Outcome::Clean` leaves the suite passing.

   The existing fixture already carries the *right* argument — its doc comment explains that a clean fixture
   makes the comparison vacuous, and asserts the outcome is a violation for exactly that reason. The argument
   was applied once, to the whole verdict, when it is a property of each dimension separately.

2. **The bound-set comparison compares a function with itself.** The requirement says an observer declares
   *exactly the bound set* its dimension exports, and the reaction asserts
   `observer.bounds() == dimension::observation_bounds()`. But all three `bounds()` implementations *are*
   `observation_bounds()` — one line each, nothing else — so the assertion is `f() == f()` and no change to any
   declaration can fail it. Measured: drifting a declaration's extent with its id untouched left this suite at
   10 passed.

   Noticed while trying to demonstrate a smaller defect. The first reading was that the comparison lowers each
   `BoundDecl` to its id and so admits a drifted `shape`/`extent`/`pinned_by`; that reading is true and
   irrelevant, because comparing whole declarations would have been a better comparison of two identical
   things. The real defect is one level up.

   **This is a name without a reaction, not a false negative.** What the requirement fears — an observer
   satisfying its obligation with a second, divergent list — is prevented by construction: there is one list and
   one function. And the declarations' *content* is held elsewhere: the same extent perturbation fails
   `observation_bound_model`'s `the_extent_projection_is_fresh`, verified. So nothing is currently unguarded;
   what exists is an assertion that cannot fail, sitting where a reader takes it for the guarantee.

3. **The runtime arm is a hand-copied twin.** `evaluate_constitution`'s runtime arm and
   `RuntimeObserver::observe` hold the same three statements, including the same
   `cannot read workspace '{}': {message}` literal in two places. Equality between the two paths for this
   dimension currently depends on nobody editing one of the copies — the precise failure mode the reaction is
   supposed to end, sitting inside the thing being compared.

## What Changes

- The equality fixture declares a **deliberately violated boundary in each of the three dimensions**, and the
  reaction asserts each dimension actually reacted. A fixture that goes vacuous — because the workspace
  changed under it — fails loudly instead of quietly proving less.
- Both reactions are driven from **one array of three dimension entries**. An entry carries how that dimension
  declares its violating boundary, how it enters the fold, which violation kind proves it reacted, its
  observer's `bounds()`, and its dimension's exported declarations. One array, so the fixture and the fold
  cannot describe different dimension sets.
- The bound-set comparison is replaced by a reaction over **the shape of each `bounds()` body**: exactly one
  executed statement, and that statement the delegation. A second list is something written in a body, so the
  body is where the property lives and where it can fail. The declarations' content stays held by the extent
  projection, which already reacts to it.
- `evaluate_constitution`'s runtime arm **delegates to `RuntimeObserver`** rather than restating it. Not a new
  shared helper: the built-in path *is* the observer for this dimension, which is a stronger statement than two
  callers of a third function, and it adds no public surface to `louke`. Where the two paths share an
  implementation, equality holds *by construction*; the requirement says so, so a reader does not take a
  constructed equality for an observed one.

## Impact

- Affected specs: `observer-protocol`
- Affected code: `crates/tianheng/tests/observer_protocol.rs`, `crates/tianheng/src/runner.rs`
- **No public API change at all** — the delegation reuses `RuntimeObserver`, already public. The observable
  behaviour of both paths is unchanged, and the reaction is what proves it.
- Cost of the delegation: one `to_vec` of the declared runtime seams per run, in exchange for deleting a
  hand-copied twin. Paid deliberately.
