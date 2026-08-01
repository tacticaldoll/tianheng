# proposal: Guibiao cfg_if Arm Declaration Is Cfg-Conditional

## Why

圭表 observes `mod` declarations inside `cfg_if!` arms (the transparency requirement, shipped 0.2.3).
Its absent-file tolerance, however, is keyed on a **bare `#[cfg]` attribute preceding the item**
(`has_bare_cfg`) — and a `mod` written inside a `cfg_if!` arm carries no such attribute, because the
predicate lives in the macro's `if #[cfg(..)]` header. So the two spellings of one intent disagree:

| the same per-platform shim | 圭表 |
| --- | --- |
| `#[cfg(unix)] mod unix_impl;` / `#[cfg(windows)] mod windows_impl;`, only `unix_impl.rs` present | exit 0 — tolerated |
| `cfg_if! { if #[cfg(unix)] { mod unix_impl; } else { mod windows_impl; } }`, only `unix_impl.rs` present | **exit 2** |

Measured, with the error naming the absence as unconditional:

```
module 'crate::windows_impl' is declared (`mod windows_impl;`) but its source file
could not be located (expected '…/windows_impl.rs' or '…/windows_impl/mod.rs')
```

rustc strips the whole non-selected arm, so that source **compiles**. 圭表 refuses to judge a working
build, and refuses it only for one of two equivalent spellings.

The accurate characterization is not "圭表 has a bug" but **the 0.2.3 transparency carve-out is
incomplete**: it made arm bodies *observable* without making arm membership *confer the gate*. The
scanner is internally consistent on its own terms — it tolerates absence only where it can see a gate
on the item — and the carve-out simply never taught it that an arm's predicate is such a gate.

Every `cfg_if!` arm is conditionally compiled by construction — the `if` arm on its declared
predicate, the trailing `else` on that predicate's negation — so arm membership is itself the
"might legitimately be absent on this build" signal the flag exists to carry.

Completing the carve-out has no downside to weigh: a file that does not exist holds no code to miss,
so skipping it introduces no false negative, while erroring introduces a false positive on compilable
source. There is no offsetting cost to trade.

## What Changes

- Treat a file-form `mod name;` declared directly inside a transparent macro (`cfg_if!`) arm as
  cfg-conditional, so an absent conventional file is tolerated exactly as a bare-`#[cfg]`-gated one
  is. `TopLevelTracker` already knows this — a non-empty `macro_scopes` stack is precisely "inside a
  transparent macro body", and the existing `is_top_level` gate already restricts `mod` observation to
  a declaration sitting directly in an arm brace rather than nested in an item body.
- Rename `DeclaredModule::has_bare_cfg` to `is_cfg_conditional`. The flag no longer means "a bare
  `#[cfg]` precedes this item"; it means "this declaration may legitimately have no file in the
  current configuration", which now has two sources. The old name would misdescribe it, and its doc
  comment's claim of being "the same one hunyi's `has_cfg_attr` checks" stops holding.
- No change to what is *observed*: an arm module whose file exists is still reached and governed, and
  the ambiguity reaction (both conventional forms present) still fires regardless of any gate.

## Capabilities

### Modified Capabilities

- `module-boundary`: the absent-file tolerance in *A plain module declaration resolves to exactly one
  conventional file* gains its second source — membership in a `cfg_if!` arm — alongside the existing
  bare-`#[cfg]` attribute, with `#[cfg_attr]` still granting nothing.

## Impact

- `crates/guibiao/src/module_scan/reachability/declarations.rs`: the flag, its name, and its
  assignment at the file-form declaration branch.
- `crates/guibiao/src/module_scan/reachability/walk.rs`: the two tolerance sites that read the flag
  (plain absent file, and an absent `#[path]` remap target) — both keyed on the same flag, so one
  change covers both.
- `CHANGELOG.md`: `[Unreleased]` → `### Fixed`.
- Non-breaking: no public API, DSL, or wire-format change. The adopter-facing effect is that a
  previously-refused compilable crate is now judged.
