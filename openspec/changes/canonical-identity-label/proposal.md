# Canonical identity label

## Why

Two identity labels landed in the 0.4.0 window — 圭表/渾儀's compilation unit
(`xingbiao::compilation_unit_label`) and 漏刻's observed file (`audit::scan::probes::labeled`) — and
each derives its value by stripping a prefix from an observed path. Both then hand that stripped path
to a *different* rule about what a path is, and neither rule is complete:

1. **The separator is the platform's, not the label's.** Both `to_str()` / encode the stripped path
   verbatim, so one commit yields `unit: "src/lib.rs"` on Linux and `unit: "src\\lib.rs"` on Windows.
   A baseline recorded by CI matches nothing for a Windows contributor, and every entry re-fires as
   new. This is the checkout-dependence class this window closed five times, arriving along an axis
   none of those five covered. The crates do target Windows — `sync_parent_dir` carries a
   `cfg(not(unix))` arm and `encoded` documents WTF-8 — so it is not out of scope by construction.

2. **A non-UTF-8 path is refused by one dimension and judged by the other.**
   `compilation_unit_label` returns `None` for two unrelated causes — the root is not under the
   manifest directory, and the stripped path is not valid UTF-8 — and both callers turn `None` into
   `out_of_package_root_error`, whose text asserts "which is not under the package's manifest
   directory". So a package whose root path contains a non-UTF-8 byte *inside* its own directory is
   refused with a diagnostic naming a cause that is factually false, and the message renders the
   offending bytes through `display()`, lossily. Meanwhile 漏刻 already solved this exact input:
   `probes::encoded` percent-escapes it and keeps the label injective. Two dimensions disagreeing on
   what an input *is* is not 三儀 ⊥ 三儀 — that principle is about independent implementations of one
   rule, not about divergent scope.

Both are one missing thing: there is no stated, shared answer to "what is the canonical label for an
observed path". Each site invented half of one.

## What Changes

- 星表 gains `path_label`, the single canonical answer: a path rendered with `/` as its only
  separator and every byte preserved injectively.
- `compilation_unit_label` is built on it. Its `None` then carries exactly one meaning — the root is
  not under the manifest directory — so the constitution error it triggers is true whenever it fires.
  A non-UTF-8 root inside the package directory becomes **governed** rather than refused.
- 漏刻's private `encoded` is retired in favour of the shared function, so the injectivity rule it
  already stated is the one 圭表 and 渾儀 now hold too, written once.
- `structured-violation-identity` gains the rule the coordinate derivation was missing: a label that
  is identity must not vary with the platform that produced it.

## Impact

- Affected specs: `structured-violation-identity`, `module-boundary`, `runtime-origin-assertion`
- Affected code: `crates/xingbiao/src/lib.rs`, `crates/guibiao/src/errors.rs`,
  `crates/hunyi/src/errors.rs`, `crates/louke/src/audit/scan/probes.rs`
- **Not breaking for existing baselines on unix**: measured, a realistic relative path
  (`src/lib.rs`, `src/bin/x.rs`, `tools/outside.rs`, a `%`-bearing path, a non-UTF-8 path) yields a
  label byte-identical to today's. What changes is Windows (where today's label is wrong) and a
  non-UTF-8 root inside the package (today refused, now judged) — both strictly corrections.
