# Change: gate 漏刻's audit-scoped declarations with the reaction they describe

## Why

`crates/louke/src/lib.rs` exports its bounds unconditionally:

```rust
mod bounds;
pub use bounds::observation_bounds;

// Gated with the audit face it delegates to: `audit_probe_coverage` and the 星表 dependency it derives
// its corpus from are both behind `audit`, so an audit-OFF build of this crate alone must not see this
// module. CI's isolated `cargo clippy -p louke` is what caught the ungated version …
#[cfg(feature = "audit")]
mod observer;
```

The reasoning above `mod observer` applies to `mod bounds` and was not applied to it. Measured: **five of the six**
declarations describe `audit_probe_coverage` — the scanner an audit-OFF build compiles none of — including two
`Reached::UnderReacts` declared false negatives owned by `Owner::Engine`. Only
`a-composite-shape-yields-a-truncated-origin` describes the always-present origin derivation on the hot path.

So an audit-OFF dependent read six declared bounds for a reaction the crate does not contain, and the accessor's
doc comment — *"Every observation bound 漏刻 declares"* — was false in that configuration. A bound is a property of
a **reaction**; a declaration whose reaction is absent is an unbacked claim, which is the one thing this model
exists to refuse. It arrived through the *export* rather than through the declaration, which is why no existing
reaction saw it: the bijection runs with `--all-features`.

## What Changes

- The five audit-scoped declarations move behind `#[cfg(feature = "audit")]`; the hot-path one stays, because
  `observation-bound-model` requires a crate that *has* a declaration to export it.
- `Owner` and `FactGranularity` are gated with them — only the audit-scoped declarations name either, so the
  isolated audit-OFF `cargo clippy -p louke` would report both unused. That pass reported it immediately, and its
  `unused_mut` on the accumulator too; the allow for that is scoped with `cfg_attr` to the configuration where the
  extend is compiled out, rather than blanket.
- The module and accessor documentation say the set depends on the feature, instead of claiming to be every bound.
- `observation-bound-model` gains the rule: **where a reaction is behind a Cargo feature, the declarations
  describing it are gated with it.**

## Impact

- Affected specs: `observation-bound-model`
- Affected code: `crates/louke/src/bounds.rs`
- **Adopter-visible under a non-default feature**: an audit-OFF dependent of `louke` sees one declared bound
  instead of six. That is the correction, not a regression — the five it no longer sees describe a scanner its build
  never had. The `tianheng` shell enables `audit`, so nothing changes for anyone using the composed entry.
- No API removed: `observation_bounds` keeps its signature in both configurations.
