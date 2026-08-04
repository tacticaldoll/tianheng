# Canonical identity label

## Why

Two identity labels landed in the 0.4.0 window — 圭表/渾儀's compilation unit
(`xingbiao::compilation_unit_label`) and 漏刻's observed file (`audit::scan::probes::labeled`) — and
each derives its value by stripping a prefix from an observed path. Each then applies its own,
different rule for turning that stripped path into a string, and one of those rules is wrong:

**The separator is the platform's, not the label's.** Both render the stripped path verbatim, so one
commit yields `unit: "src/lib.rs"` on Linux and `unit: "src\\lib.rs"` on Windows. A baseline recorded
by CI matches nothing for a Windows contributor and every entry re-fires as new. This is the
checkout-dependence class this window closed five times, arriving along an axis none of those five
covered: not *where* the repository sits, but *which platform read it*. The crates do target Windows —
`sync_parent_dir` carries a `cfg(not(unix))` arm and `encoded` documents WTF-8 — so it is not out of
scope by construction.

There is no shared, stated answer to "what is the canonical label for an observed path", so the two
sites answer differently. 漏刻's answer is the more complete one — `probes::encoded` keeps the label
injective across bytes that are not valid UTF-8 — and it is private to 漏刻.

### What this change does NOT fix, having measured that it is not broken

Review also reported that `compilation_unit_label` conflates two `None` causes — "not under the
manifest directory" and "not valid UTF-8" — and so refuses a non-UTF-8 root inside the package with a
diagnostic naming a cause that is factually false. **The second cause is unreachable**, measured four
ways:

- `cargo metadata` run anywhere under a non-UTF-8 directory fails outright: `error: path contains
  invalid UTF-8 characters`, exit 101. Cargo will not operate there at all.
- An auto-discovered target whose *file name* is not valid UTF-8 (`src/bin/ba\xFFd.rs`) is silently
  omitted from `cargo metadata`'s target list, so it never becomes a root.
- A `Cargo.toml` is UTF-8, so a `[[bin]] path` literal cannot spell a non-UTF-8 path.
- Decisively: `src_path` and `manifest_path` reach 星表 as JSON **strings**, so every path built from
  them is valid UTF-8 by construction, and `to_str()` on a component-boundary suffix of one cannot
  return `None`.

So that half of the finding is refuted rather than fixed, and the reason is recorded here so it is not
re-raised. The `None`-means-one-thing property still arrives — as a free consequence of the shared
primitive being infallible, not as a fix for a live misdiagnosis.

## What Changes

- 星表 gains `path_label`: a path rendered with `/` as its only separator and every byte preserved
  injectively. One canonical answer, in the substrate all three dimensions already read.
- `compilation_unit_label` is built on it, so a Windows-produced label matches a Linux-produced one.
  Its `None` then has exactly one possible cause, structurally.
- 漏刻's private `encoded` is retired in favour of the shared function, so the injectivity rule 漏刻
  already stated is the one 圭表 and 渾儀 now hold too, written once. 漏刻 is where the non-UTF-8 case
  is genuinely reachable — its labels come from filesystem walks, not from Cargo's JSON — so that rule
  keeps its reason for existing.
- `structured-violation-identity` gains the rule the coordinate derivation was missing: a label that
  is identity must not vary with the platform that produced it.

## Impact

- Affected specs: `structured-violation-identity`
- Affected code: `crates/xingbiao/src/lib.rs`, `crates/louke/src/audit/scan/probes.rs`
- **No baseline re-keys on unix**: measured, every shape that actually occurs — `src/lib.rs`,
  `src/bin/x.rs`, an absolute path, `tools/outside.rs`, a `%`-bearing path, a non-UTF-8 path, and a
  unix backslash inside one file name — yields a label byte-identical to today's. What changes is
  Windows, where today's label is wrong.
