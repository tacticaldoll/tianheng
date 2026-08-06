# Change: state where a lexical self-guard stops, and where polarity does not apply

## Why

Two claims in this window's review were left open pending measurement. Both are now measured, and neither turned
out to be what it looked like.

### 1. `composition_introduces_no_trait_object` reads less than it appears to

The reaction that keeps `tianheng`'s composed shell free of trait objects is lexical by necessity — recorded and
still true: no module of this crate is governed by a semantic boundary, and the `dyn`-trait DSL offers only
forbid-all and forbid-named-operands, so a declared exposure would have been a name with no reaction. But *how*
lexical was never stated. It reads only `src/*.rs` at the top level, and only lines that themselves begin with
`pub `.

Measured, both limits are real and neither is currently a false negative:

- **Subdirectories are never read.** `src/runner/` and `src/runner/projection/` hold eight files the reaction
  never opens. Injecting `pub fn probe_exposure() -> Option<Box<dyn Debug>>` into
  `src/runner/projection/document.rs` leaves the reaction passing. It is **not** an exposure, though: `mod
  runner;` is private and so is `mod projection;`, so nothing under them is reachable from outside the crate.
  The non-recursion is therefore *correct* — but it rests on an **unchecked premise**. Someone writing
  `pub mod runner;` would silently take eight files out of the reaction's reach with nothing to say so.
- **A wrapped signature's continuation is never seen.** The matcher requires `pub ` at the start of the same
  line that carries `dyn `. A `pub fn` whose return type sits on a later line is invisible. Multi-line `pub fn`
  signatures exist in this crate (`testing.rs:165`), so the shape is live even though no instance returns `dyn`.

The first is a premise to **check**, not a bound to declare. The second is genuinely out of reach for a
line-oriented matcher and belongs in the register as a declared bound.

### 2. Polarity has a reaction after all — the compiler's

The open question was whether `Violation::polarity` being `Option` hides rule kinds that should carry a repair
direction and do not. Measured across every production emission site:

| Path | Polarity |
| --- | --- |
| 圭表 crate rules | `Rule::polarity()` — an **exhaustive match** returning `Polarity`, not an `Option` |
| 圭表 module rules | `ModuleRule::polarity()` — likewise exhaustive |
| 渾儀 all findings | `EmitContext { polarity: Polarity }` — non-optional field |
| 漏刻 origin assertion | set explicitly |
| 漏刻 **probe audit** | none |

So for every dimension whose rules *have* a direction, a new rule variant cannot compile without declaring one —
by construction, which is a stronger guard than a reaction. The single path that emits none is the probe audit,
and that is correct rather than missing: `Polarity` distinguishes a deny-breach from an allowlist gap, and
"this declared seam has no probe" is neither. The repair is to probe it or drop the declaration.

What is missing is not a reaction but the **sentence**. Nothing says when `None` is the right answer, so a reader
meeting an `Option` with no stated rule assumes a gap — as this review did.

## What Changes

- The trait-object reaction **asserts its own premise**: every subdirectory of `src/` is reached through a
  non-`pub` `mod` declaration, so nothing beneath it can be publicly exposed. Making one public fails the
  reaction and demands recursion, instead of quietly removing eight files from its reach.
- Its line matcher becomes a **named recognizer**, so the residual can be pinned by feeding it text rather than
  by rewriting the crate.
- **One declared observation bound** for the residual: a `dyn` on a line that does not itself begin with `pub `.
- `Polarity`'s own documentation states **when `None` is correct**, and `runtime-origin-assertion` says that its
  audit findings carry none, with the reason.

## Impact

- Affected specs: `observer-protocol`, `runtime-origin-assertion`
- Affected code: `crates/tianheng/tests/observer_protocol.rs`, `crates/tianheng/src/bounds.rs`,
  `crates/xuanji/src/model.rs`
- No public API change. `Polarity` gains documentation, not a variant.
- The bound register gains one bound; the extent projection changes by exactly that entry.
