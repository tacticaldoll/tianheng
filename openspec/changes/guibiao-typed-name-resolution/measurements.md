# Typed-name-resolution probe matrix

Measured 2026-09-29 with `rustc 1.96.0` / `cargo 1.96.0`. Probe packages are real Cargo packages under the dedicated scratch directory `/tmp/tianheng-typed-name-resolution.AdH7r7`; builds used `CARGO_BUILD_JOBS=4`, offline mode, and `cargo build --tests`. Checker executables were built from these exact revisions:

| Label | Revision |
|---|---|
| v0.7.1 | `c69171a77380780c79a7f39f48c17744f783bf50` |
| `be047d8b^` | `4e92d0f2626eb456ffd197de90d88ee008494751` |
| release/0.8.0 | `8dae20662188316c55fd104953632961ad4966c3` |

Checker cells are `default/strict` exit codes in that order. The target cell gives the proposed exit codes, also `default/strict`. Exit `101` is Cargo rejecting a probe; all other build exits are `0`. For every row, [measurements.raw.log](measurements.raw.log) contains the executed Cargo command and complete stdout, stderr, and exit code under `## {scenario} · rustc ({edition})`; each checker cell has its own section named `## {scenario} · {revision} · {mode}`. Thus the codes below are summaries; the appendix is the per-cell raw evidence.

| Scenario | Rust/Cargo result and binding | v0.7.1 | `be047d8b^` | release/0.8.0 | Target answer |
|---|---|---:|---:|---:|---:|
| R1 parent private use through `super::*` (2021) | Build 0; nested test resolves inherited `SystemTime` to `std::time` | 1/1 | 1/1 | 0/0 | 1/1 |
| R2 relative prefix `std::time` (2021) | Build 0; sysroot path | 1/1 | 1/1 | 1/1 | 1/1 |
| R2 root prefix `::std::time` (2021) | Build 0; same sysroot path | 0/0 | 2/2 | 1/1 | 1/1 |
| R3 bare dependency head `md5x::compute()` (2021) | Build 0; dependency-only package resolves `md5x` externally | 0/1 | 0/1 | 0/1 | 0/1 |
| R3 root dependency head `::md5x::compute()` (2021) | Build 0; same external dependency | 0/1 | 0/1 | 1/1 | 0/1 |
| R4 local `mod md5x` shadows dependency, bare call (2021) | Build 0; call returns local `md5x::Local`, while dependency returns `u32`, so the successful type-check establishes local resolution | 0/0 | 2/2 | 0/0 | 0/0 |
| R5 `gen` prefix (2018) | Build 0; bare identifier resolves | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `async` prefix (2018) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `dyn` prefix (2018) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `try` prefix (2018) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `_` prefix (2018) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2018) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 `gen` prefix (2021) | Build 0; bare identifier resolves | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `async` prefix (2021) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `dyn` prefix (2021) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `try` prefix (2021) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `_` prefix (2021) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2021) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 `gen` prefix (2024) | Build 0; source uses `r#gen`; the tested boundary prefix is unraw `gen` | 1/1 | 1/1 | 2/2 | 2/2, suggest `r#gen` |
| R5 `async` prefix (2024) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `dyn` prefix (2024) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `try` prefix (2024) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 2/2 |
| R5 `_` prefix (2024) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2024) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#gen` prefix (2024) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#async` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#dyn` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#try` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R6 `<W>::md5x()` after the angle close (2021) | Build 0; resolves as the associated function on `W`, not an external root | 0/1 | 0/1 | 1/1 | 0/0 |

The matrix contains 29 independent probe packages. The target codes are design decisions, not executed results. Raw `async`/`dyn`/`try` probes were executed for edition 2021 only; their unraw prefixes are rejected in all three measured editions, and the Reference-backed expectation is that a raw prefix is the repair. Implementation acceptance must add raw probes for editions 2018 and 2024 before claiming those edition cases verified. Edition 2015 root behavior, ambiguous glob selection, `extern crate` alias calls, and block-local `use` items were not measured and are declared bounds in [design.md](design.md).

## R2 baseline identity experiment

On the R2 relative-prefix source, the runner first declared `::std::time` and wrote a baseline, then declared `std::time` and checked against it. Exact commands and JSON are under `R2-baseline-create` / `R2-baseline-cross-spelling` in the raw appendix.

- v0.7.1: creating the global-prefix baseline exits 0 and writes zero violations; comparing the relative spelling exits 1 with a violation.
- `be047d8b^`: global-prefix baseline creation exits 2 because that spelling is rejected; the follow-up cannot read the baseline and exits 2.
- release/0.8.0: global-prefix baseline creation exits 0 and writes one violation; the relative spelling exits 1. JSON shows a stale baseline `rule_key.prefix` of `::std::time` beside the current violation's `std::time`.

The proposal's target is one canonical identity, `std::time`, for both accepted root spellings. The existing release output demonstrates that merely emitting the same finding does not make the baseline identity the same.

## Measurement conditions and limits

Each source package was built with Cargo before checking it. The `_` cases are intentionally compiler-invalid; their `cargo build --tests --quiet` result is exit 101, and each checker still ran on the same source. All matrix check commands were serialized. Probe and checker builds used offline Cargo after the sandbox could not resolve crates.io during an initial online build attempt. The complete raw appendix includes every matrix command and result, including successful empty output, violations, constitution errors, and R2 baseline JSON.
