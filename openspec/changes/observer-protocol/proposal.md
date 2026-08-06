## Why

天衡's central promise is honesty about what it does **not** observe. `observation-bound-model` typed that
honesty — where each measure stops, who owns closing a declared false negative, what a bound's defence must
demonstrate — and holds the specs' declarations and the code's in a bijection. What it cannot do is *require*
anyone to make a declaration. The promise is still a convention this family keeps about itself.

Measured: the execution layer has **no seam at all**. `grep -c 'pub trait' crates/tianheng/src crates/xuanji/src`
returns zero, and `evaluate_constitution` in `runner.rs` hand-wires three concrete calls — `check_and_cover`,
`hunyi::check_all`, `audit_probe_coverage`. So a fourth observer cannot join a run, and if one could, nothing
would oblige it to state what it does not see.

Why now: the trigger recorded in `BACKLOG.md` fired when `observation-bound-model` landed, because
`Observer::bounds()` with no default body is what turns the promise into a compile error, and it now has
something to delegate to.

## What Changes

**璇璣 gains a published `Observer` trait, and the shell gains a composition entry that folds a set of them.**

- **No method carries a default body.** Adding a *stage* — a new question every observer must answer — then
  breaks every implementor, family and third-party alike. That is the enforcement, and it is the half an
  `enum` + hand-maintained `ALL` cannot give: there, adding a value breaks the *consumer's* match while every
  existing declaration keeps its old answer and nobody re-examines it. Adding an *answer* to an existing
  question must break nothing, and that is `#[non_exhaustive]` on the extent enums already.
- **`bounds()` is one of those methods.** An observer cannot join a run without declaring what it does not
  observe. The framework's promise stops being the family's diligence and becomes a property of the type.
- **Three methods, not five.** An earlier sketch split the lifecycle into corpus, facts and reaction. That is a
  fiction 三儀 ⊥ 三儀 forbids: each dimension implements its own lexical hygiene, *by design*, with no shared
  scanner, so no dimension exposes those stages separately and every implementor would collapse them. The
  honest seam is: identify yourself, observe a workspace, declare your limits.
- **The three dimensions implement it**, each delegating to the outcome-only face it already has, so the
  protocol is dogfooded rather than offered.
- **The fold is ordered with a cannot-judge short-circuit**, preserving `merge_outcomes`' existing invariant —
  a constitution error from any observer supersedes every violation, and evaluation stops. Assembly order is
  therefore **semantically observable** (it decides which cannot-judge is reported), so this change declares
  that rather than leaving it incidental.
- **There is no `dyn` anywhere**, because the fold is **eager**: assembly folds each observer as it arrives, so
  the heterogeneous collection never exists. `Run::over(..).observe(a).observe(b).verdict()` monomorphizes each
  call and carries only the accumulated outcome. This replaced a design that held `&[&dyn Observer]` and
  declared the trait-object exposure as a boundary of the shell — measured, that could not be honoured: no
  module of `tianheng` is governed by a semantic boundary today, and `DynTraitBoundary` offers `must_not_expose_dyn`
  (all) and `must_not_expose_dyn_of` (named operands) with **no allow-except form**, so the declaration would
  have been a name with no reaction. The eager fold removes the exposure instead of governing it.

## Capabilities

### New Capabilities

- `observer-protocol`: the fixed lifecycle every observation participant implements, the ordered fold that
  composes a set of them into one verdict, and the obligation — enforced by a method with no default — that an
  observer declare what it does not observe before it may join a run.

### Modified Capabilities

None. `observation-bound-model` supplies the declarations this protocol makes mandatory and is unchanged by it.

## Impact

- **New**: `Observer` in `xuanji`, re-exported through the dimensions and `tianheng::prelude`.
- **New**: an observer type per dimension, and `tianheng::Run` — an eager fold, so no trait object appears in
  any signature.
- **New**: a reaction asserting the trait-driven fold agrees with `check_constitution` on this workspace, and
  that each observer's `bounds()` is exactly its dimension's declared set.
- **Unchanged**: `check_constitution`, `run`, and the CLI keep their present path and behaviour, coverage
  included. The protocol is an additional composition entry, not a replacement — a rewrite of the built-in
  path would put a behaviour change under a change whose subject is a seam.
- Version class **MINOR** — additive published API, no adopter migration.
