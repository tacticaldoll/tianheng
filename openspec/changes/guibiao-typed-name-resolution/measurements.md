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
| R5 `async` prefix (2018) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `dyn` prefix (2018) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `try` prefix (2018) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `_` prefix (2018) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2018) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 `gen` prefix (2021) | Build 0; bare identifier resolves | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `async` prefix (2021) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `dyn` prefix (2021) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `try` prefix (2021) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `_` prefix (2021) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2021) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 `gen` prefix (2024) | Build 0; source uses `r#gen`; the tested boundary prefix is unraw `gen` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `async` prefix (2024) | Build 0; source uses `r#async` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `dyn` prefix (2024) | Build 0; source uses `r#dyn` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `try` prefix (2024) | Build 0; source uses `r#try` | 1/1 | 1/1 | 2/2 | 1/1 |
| R5 `_` prefix (2024) | Build 101; Rust rejects `_::f()` as a path | 0/0 | 0/0 | 0/0 | 2/2 |
| R5 `union` prefix (2024) | Build 0; weak-keyword path resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#gen` prefix (2024) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#async` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#dyn` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R5 raw `r#try` prefix (2021) | Build 0; raw identifier resolves | 1/1 | 1/1 | 1/1 | 1/1 |
| R6 `<W>::md5x()` after the angle close (2021) | Build 0; resolves as the associated function on `W`, not an external root | 0/1 | 0/1 | 1/1 | 0/0 |

The matrix contains 29 independent probe packages. The target codes are design decisions, not executed results. The R5 target column was revised after review: a boundary prefix is a name, not Rust source, so every head that can name a crate or module is accepted bare or raw and normalized, and only `_` changes answer (see *Revision round* below, which measures the raw prefixes in all three editions and the heads that can never name anything). `extern crate … as` alias calls remain unmeasured and keep their existing stated bound.

## R2 baseline identity experiment

On the R2 relative-prefix source, the runner first declared `::std::time` and wrote a baseline, then declared `std::time` and checked against it. Exact commands and JSON are under `R2-baseline-create` / `R2-baseline-cross-spelling` in the raw appendix.

- v0.7.1: creating the global-prefix baseline exits 0 and writes zero violations; comparing the relative spelling exits 1 with a violation.
- `be047d8b^`: global-prefix baseline creation exits 2 because that spelling is rejected; the follow-up cannot read the baseline and exits 2.
- release/0.8.0: global-prefix baseline creation exits 0 and writes one violation; the relative spelling exits 1. JSON shows a stale baseline `rule_key.prefix` of `::std::time` beside the current violation's `std::time`.

The proposal's target is one canonical identity, `std::time`, for both accepted root spellings. The existing release output demonstrates that merely emitting the same finding does not make the baseline identity the same.

## Measurement conditions and limits

Each source package was built with Cargo before checking it. The `_` cases are intentionally compiler-invalid; their `cargo build --tests --quiet` result is exit 101, and each checker still ran on the same source. All matrix check commands were serialized. Probe and checker builds used offline Cargo after the sandbox could not resolve crates.io during an initial online build attempt. The complete raw appendix includes every matrix command and result, including successful empty output, violations, constitution errors, and R2 baseline JSON.

## Revision round

Measured 2026-09-29 with `rustc 1.96.0` / `cargo 1.96.0`, against the same three revisions, by the script at the end of this section. The first round's scratch directory was deleted and its log does not record the checker's constitution, so this round records it: the script replaces `crates/tianheng/src/constitution.rs` in a `git archive` of each revision with a constitution that declares one `ModuleBoundary::in_crate(PROBE_PACKAGE).module(PROBE_MODULE).must_not_call_inline(PREFIX)`, adding `.strict_external()` when `MODE=strict`. Its `K-2021-async` row reproduces the first round's `R5 async prefix (2021)` cells (1/1, 1/1, 2/2), which is the check that the reconstruction observes what the first round observed.

Every command, its stdout, stderr and exit code are in [measurements.revision.raw.log](measurements.revision.raw.log) under `## {row} · rustc ({edition})` and `## {row} · {revision} · {mode}`. Cells are `default/strict` exit codes. **Rust** is the build exit plus the binding the source establishes. **Target** is a design decision, not an executed result.

### Boundary-prefix heads

K rows: the package depends on a path crate renamed to the head, and `crate::core` writes `use HEAD::f; pub fn g() { f(); }`, spelling `HEAD` raw exactly where the edition requires it (`async`, `dyn`, `try` from 2018; `gen` in 2024). The `-rawprefix` rows run the same source under the boundary prefix `r#HEAD`.

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| `gen`, `async`, `dyn`, `try` unraw prefix, each of 2018/2021/2024 (12 rows) | Build 0 | 1/1 | 1/1 | 2/2 | 1/1 |
| same heads, raw prefix `r#…`, each edition (12 rows) | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| `union` unraw and raw prefix, each edition (6 rows) | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| `_` prefix, `use _::f; f()` shape, each edition (3 rows) | Build 101 | 1/1 | 1/1 | 1/1 | 2/2 |
| `K-2021-_-direct`: `_` prefix, `_::f()` shape (the first round's shape) | Build 101 | 0/0 | 0/0 | 0/0 | 2/2 |

The two `_` shapes differ at v0.7.1 (1/1 when an import makes the scanner resolve `f` to `_::f`, 0/0 for a direct `_::f()`), and neither source compiles. Every revision accepts `_` as a prefix; none refuses it.

N rows: `crate::clock` defines `now`, `crate::core` calls `crate::clock::now()`, and the boundary prefix varies. `N-2021-source-*` build a crate whose source writes the raw spelling.

| Prefix | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| `crate::clock` | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| `r#crate::clock` | `r#crate::clock::now()` builds 101 | 1/1 | 2/2 | 2/2 | 2/2 |
| `::crate::clock` | — | 0/0 | 2/2 | 2/2 | 2/2 |
| `self::clock`, `super::clock` | — | 0/0 | 2/2 | 2/2 | 2/2 |
| `r#self::clock`, `r#super::clock` | `r#self::…`, `r#super::…` build 101 | 0/0 | 2/2 | 2/2 | 2/2 |
| `Self::clock` | — | 0/0 | 0/0 | 2/2 | 2/2 |
| `r#Self::clock` | `r#Self::…` builds 101 | 0/0 | 2/2 | 2/2 | 2/2 |
| `_::clock` | — | 0/0 | 0/0 | 0/0 | 2/2 |
| `r#_::clock` | `r#_::…` builds 101 | 0/0 | 2/2 | 2/2 | 2/2 |

Against release/0.8.0 the only head whose answer this design changes is `_`/`_::clock`, from accepted to refused; against v0.7.1 the other N-row changes were made by commits already on release/0.8.0 and are recorded in its `[Unreleased]` BREAKING entry.

### Block-local scope

Rust's answer for B3/B4 is one finding under each prefix: `h` calls `crate::a::X::fa`, `g` calls `crate::b::X::fb`. The `Found:` column is the checker's finding list for the `default` cell; strict lists the same.

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| A `fn g() { use std::process::Command; Command::new("x"); }`, prefix `std::process` | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| A2 same `use` inside a nested block | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| B module `use crate::a::X`; `fn g` has `use crate::b::X; X::f()`, prefix `crate::a` | Build 0; `g` returns `u16`, `b`'s type | 0/0 | 0/0 | 0/0 | 0/0 |
| B same, prefix `crate::b` | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| B3 module `use crate::a::X`, `h` calls `X::fa()`, then `g` has `use crate::b::X; X::fb()`, prefix `crate::a` | Build 0 | 0/0 | 0/0 | 0/0 | 1/1 (`fa`) |
| B3 prefix `crate::b` — Found: `crate::b::X::fa`, `crate::b::X::fb` at every revision | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 (`fb` only) |
| B4 `g`'s local `use` first, `h`, module `use crate::a::X` last, prefix `crate::a` — Found: `crate::a::X::fa`, `crate::a::X::fb` at every revision | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 (`fa` only) |
| B4 prefix `crate::b` | Build 0 | 0/0 | 0/0 | 0/0 | 1/1 (`fb`) |
| F module `use std::process::Command`; `fn g` declares `struct Command` and calls `Command::new("x")` | Build 0; resolves to the block's struct | 1/1 | 1/1 | 1/1 | 0/0 |
| G `Command::new("x")` written before the block's `use std::process::Command` | Build 0; a block `use` covers its whole block | 1/1 | 1/1 | 1/1 | 1/1 |

Every revision binds a name to the **last `use` of it written anywhere in the file**, block-local or not: B3 and B4 are the same source with the statements reordered and give opposite answers, each a false negative under one prefix and a false finding under the other. F is a false positive from the same cause.

### Qualified paths beginning with `<`

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| C1 `<std::time::SystemTime>::now()`, prefix `std::time` | Build 0 | 0/0 | 0/0 | 0/0 | 0/0, declared under-reach |
| C2 `<crate::clock::S as crate::clock::Clock>::now()`, prefix `crate::clock::Clock` | Build 0 | 0/0 | 0/0 | 0/0 | 0/0, declared under-reach |
| C2 same, prefix `crate::clock` | Build 0 | 0/0 | 0/0 | 0/0 | 0/0, declared under-reach |
| C3 `use crate::clock::{Clock, S}; <S as Clock>::now()`, both prefixes | Build 0 | 0/0 | 0/0 | 0/0 | 0/0, declared under-reach |
| C4 `<SystemTime as Clone>::clone(t)` after `use std::time::SystemTime`, prefix `std::time` | Build 0 | 0/0 | 0/0 | 0/0 | 0/0, declared under-reach |

No revision reports or refuses any of these; each is a call Rust makes into the prefix, so the answer is an under-reach, which the active spec already names in its first observation bound.

### Edition 2015

`D-2015-extern-crate-and-local` is an edition-2015 package with `extern crate md5x;` at its root. `crate::core` calls `::clock::now()`, `::md5x::compute()` and `md5x::compute()`; `crate::use2015` writes `use clock::now; now()`. `D-2018-control-use-without-crate` is the same `use clock::now` in 2018, which builds 101 with E0432, so the 2015 source resolves `clock` from the crate root.

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| prefix `crate::clock` over `crate::core` (`::clock::now()`) | Build 0; `::clock` is the crate-root module | 0/0 | 0/0 | 0/0 | 1/1 |
| prefix `md5x` over `crate::core` | Build 0; both calls reach the dependency | 0/1 | 0/1 | 1/1 | 0/1 |
| prefix `::md5x` over `crate::core` | Build 0 | 0/0 | 2/2 | 1/1 | 0/1 |
| prefix `crate::clock` over `crate::use2015` (`use clock::now`) | Build 0 | 0/0 | 0/0 | 0/0 | 1/1 |

### Two bindings for one name

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| E1 `use crate::a::*; use crate::b::*;` and `X::f()` | Build 101 with E0659 (`X` is ambiguous) | 1/1 (glob) | 1/1 | 1/1 | 1/1 |
| E2 same globs, `X` unused | Build 0 | 1/1 (glob) | 1/1 | 1/1 | 1/1 |
| E3 `use crate::a::*; use crate::b::X; X::f()`, prefix `crate::a` | Build 0; the named import wins | 1/1 (glob) | 1/1 | 1/1 | 1/1 (glob hazard, stated over-reaction) |
| E3 same, prefix `crate::b` | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |
| E4 `#[cfg(unix)] use crate::a::*; #[cfg(not(unix))] use crate::b::*;`, each prefix | Build 0 | 1/1 (glob) | 1/1 | 1/1 | 1/1 |
| E5 `crate::b` re-exports `crate::a::X`, both globbed, `X::f()` | Build 0; one item, no ambiguity | 1/1 (globs `a` and `b`) | 1/1 | 1/1 | 1/1 |
| E6 `#[cfg(unix)] use crate::a::X; #[cfg(not(unix))] use crate::b::X;`, prefix `crate::a` | Build 0 on Linux; `X` is `crate::a::X` | 0/0 | 0/0 | 0/0 | 1/1 |
| E6 same, prefix `crate::b` | Build 0 | 1/1 | 1/1 | 1/1 | 1/1 |

`(glob)` marks a cell whose finding is `glob crate::… in crate::core`, the existing glob-hazard reaction. Every glob row already reacts through it, so no glob shape needs a resolver answer to be judged. E6 is the named-import form: every revision keeps the last `use`, so on the platform that compiles the `unix` arm, the live binding is the one not observed.

### Braces that open no name scope, and a generic parameter

Measured 2026-09-29 with `rustc 1.96.0` / `cargo 1.96.0` against the same three revisions, by the script's `H` section: the script with its `K` through `E` sections removed (`awk '/^# --- K:/{skip=1} /^# --- H:/{skip=0} !skip'`), so the checker builds and helpers are the ones every other row used. Each row is one edition-2021 package; `crate::core` holds the source shown, the prefix is `std::process`, and the finding in every reacting cell is `std::process::Command::new in crate::core` (H1, H3) or `std::process::Command::default in crate::core` (H2). The sections are appended to [measurements.revision.raw.log](measurements.revision.raw.log) under the same `## {row} · …` headings.

| Row | Rust | v0.7.1 | `be047d8b^` | release/0.8.0 | Target |
|---|---|---:|---:|---:|---:|
| H1 `use std::process::Command; pub struct S; impl S { const Command: u8 = 0; pub fn f() { let _ = Command::new("x"); } }` | Build 0; an associated item is reached only through `Self::`, so the bare `Command` is the import | 1/1 | 1/1 | 1/1 | 1/1 |
| H2 `#[allow(unused_imports)] use std::process::Command; pub fn f<Command: Default>() -> Command { Command::default() }` | Build 0; `Command` is the generic parameter | 1/1 | 1/1 | 1/1 | 1/1, declared over-reaction (design g) |
| H3 `use std::process::Command; pub enum E { Command } pub fn f() { let _ = Command::new("x"); }` | Build 0; a variant is reached only through `E::` | 1/1 | 1/1 | 1/1 | 1/1 |

H1 and H3 are what a scope walk that opened a scope at every brace would get wrong: it would record `const Command` or the variant as a binding and lose a finding Rust says is real. H2 is the one row here whose target is not Rust's answer.

### Script

Run as `S=<scratch dir> REPO=<tianheng checkout> bash probes.sh`. It prints the summary rows above and writes the raw log.

```bash
#!/usr/bin/env bash
# Regenerate the revision-round probe matrix. Usage: S=<scratch dir> REPO=<tianheng checkout> bash probes.sh
# Writes $S/raw.log (every command, stdout, stderr, exit code) and prints one summary line per row.
set -u
: "${S:?}" "${REPO:?}"
P=$S/probes LOG=$S/raw.log
mkdir -p "$P" "$S/targets" "$S/src"
: > "$LOG"

# --- checkers: each revision's tree with crates/tianheng/src/constitution.rs replaced by the probe constitution
cat > "$S/constitution.rs" <<'EOF'
//! Probe constitution: one inline-call boundary read from the environment.
use tianheng::prelude::*;

pub fn constitution() -> Constitution {
    let var = |k: &str| std::env::var(k).unwrap_or_else(|_| panic!("{k} unset"));
    let draft = ModuleBoundary::in_crate(&var("PROBE_PACKAGE"))
        .module(&var("PROBE_MODULE"))
        .must_not_call_inline(&var("PREFIX"));
    let draft = if var("MODE") == "strict" { draft.strict_external() } else { draft };
    Constitution::new("probe").boundary(draft.because("probe inline path resolution"))
}
EOF
for pair in v071:c69171a77380780c79a7f39f48c17744f783bf50 pre:4e92d0f2626eb456ffd197de90d88ee008494751 rel:8dae20662188316c55fd104953632961ad4966c3; do
  name=${pair%%:*} rev=${pair#*:}
  if [ ! -x "$S/targets/$name/debug/tianheng" ]; then
    rm -rf "$S/src/$name"; mkdir -p "$S/src/$name"
    git -C "$REPO" archive "$rev" | tar -x -C "$S/src/$name"
    cp "$S/constitution.rs" "$S/src/$name/crates/tianheng/src/constitution.rs"
    (cd "$S/src/$name" && CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build -p tianheng --bin tianheng --target-dir "$S/targets/$name" --quiet)
  fi
done

sec() { # title, then an env-prefixed command; appends one section, echoes the exit code
  local title=$1 out code; shift
  { echo "## $title"; echo "\$ $*"; } >> "$LOG"
  out=$(env "$@" 2> "$S/.err"); code=$?
  { echo "[stdout]"; if [ -n "$out" ]; then echo "$out"; else echo "<empty>"; fi
    echo "[stderr]"; if [ -s "$S/.err" ]; then cat "$S/.err"; else echo "<empty>"; fi
    echo "[exit code] $code"; echo; } >> "$LOG"
  echo "$code"
}
build() { # probe edition
  sec "$1 · rustc ($2)" CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true cargo build --manifest-path "$P/$1/Cargo.toml" --target-dir "$S/targets/probes" --tests --quiet
}
row() { # label probe prefix module ; cells are default/strict per revision
  local label=$1 probe=$2 prefix=$3 module=$4 cells="" c name tag pair mode code
  for c in v071:v0.7.1 pre:be047d8b^ rel:release/0.8.0; do
    name=${c%%:*} tag=${c#*:} pair=""
    for mode in default strict; do
      code=$(sec "$label · $tag · $mode" CARGO_BUILD_JOBS=4 CARGO_NET_OFFLINE=true MODE=$mode PREFIX=$prefix PROBE_MODULE=$module PROBE_PACKAGE=probe "$S/targets/$name/debug/tianheng" check --manifest-path "$P/$probe/Cargo.toml")
      pair="$pair${pair:+/}$code"
    done
    cells="$cells | $pair"
  done
  echo "$label | build $(grep -A99 "^## $probe · rustc" "$LOG" | sed -n '/^\[exit code\]/{s/.* //p;q}') | prefix \`$prefix\` @ $module$cells"
}
new() { # probe edition [dependency keys...] ; each key is a path dependency renamed to that key
  local d=$P/$1 edition=$2 key pkg; rm -rf "$d"; mkdir -p "$d/src"; shift 2
  printf '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "%s"\n\n[workspace]\n\n[dependencies]\n' "$edition" > "$d/Cargo.toml"
  for key in "$@"; do
    pkg="dep_$(echo "$key" | tr -dc 'a-z0-9')"; mkdir -p "$d/$pkg/src"
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2015"\n' "$pkg" > "$d/$pkg/Cargo.toml"
    printf 'pub fn f() {}\npub fn compute() -> u32 { 0 }\n' > "$d/$pkg/src/lib.rs"
    echo "$key = { package = \"$pkg\", path = \"$pkg\" }" >> "$d/Cargo.toml"
  done
}
w() { mkdir -p "$(dirname "$P/$1/src/$2")"; printf '%s\n' "$3" > "$P/$1/src/$2"; }
ab() { # two modules with a same-named type X: a::X::fa / b::X::fb (or f/f when $2 = same)
  w "$1" lib.rs $'pub mod a;\npub mod b;\npub mod core;'
  if [ "${2:-}" = same ]; then
    w "$1" a.rs $'pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }'; w "$1" b.rs $'pub struct X;\nimpl X { pub fn f() -> u16 { 0 } }'
  else
    w "$1" a.rs $'pub struct X;\nimpl X { pub fn fa() -> u8 { 0 } }'; w "$1" b.rs $'pub struct X;\nimpl X { pub fn fb() -> u16 { 0 } }'
  fi
}

# --- K: boundary-prefix heads. A dependency renamed to the head, imported with the spelling the edition requires.
for ed in 2018 2021 2024; do for h in gen async dyn try union _; do
  p=K-$ed-$h
  if [ "$h" = _ ]; then new "$p" "$ed"; src='use _::f;'
  else new "$p" "$ed" "$h"
    case "$h:$ed" in async:*|dyn:*|try:*|gen:2024) src="use r#$h::f;" ;; *) src="use $h::f;" ;; esac
  fi
  w "$p" lib.rs 'pub mod core;'; w "$p" core.rs "$src"$'\npub fn g() { f(); }'
  build "$p" "$ed" > /dev/null
  row "$p" "$p" "$h" crate::core
  [ "$h" = _ ] || row "$p-rawprefix" "$p" "r#$h" crate::core
done; done
p=K-2021-_-direct; new $p 2021; w $p lib.rs 'pub mod core;'; w $p core.rs $'pub fn f() {}\npub fn g() { _::f(); }'
build $p 2021 > /dev/null; row $p $p _ crate::core

# --- N: heads that never name a crate or module, bare and raw; and rustc on each raw spelling in source
p=N-2021-rawroot; new $p 2021; w $p lib.rs $'pub mod clock;\npub mod core;'; w $p clock.rs 'pub fn now() -> u64 { 0 }'; w $p core.rs 'pub fn g() -> u64 { crate::clock::now() }'
build $p 2021 > /dev/null
for pre in crate::clock 'r#crate::clock' ::crate::clock self::clock 'r#self::clock' super::clock 'r#super::clock' Self::clock 'r#Self::clock' _::clock 'r#_::clock'; do
  row "$p[$pre]" $p "$pre" crate::core
done
for h in 'r#crate' 'r#self' 'r#super' 'r#Self' 'r#_'; do
  p="N-2021-source-$(echo "$h" | tr -d '#')"; new "$p" 2021
  w "$p" lib.rs "pub mod clock { pub fn now() -> u64 { 0 } }"$'\n'"pub fn g() -> u64 { $h::clock::now() }"
  echo "$p | source \`$h::clock::now()\` | build $(build "$p" 2021)"
done

# --- A: a `use` inside a function body, and inside a nested block
p=A-2021-fn-local-use; new $p 2021; w $p lib.rs 'pub mod core;'; w $p core.rs 'pub fn g() { use std::process::Command; let _ = Command::new("x"); }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core
p=A2-2021-block-local-use; new $p 2021; w $p lib.rs 'pub mod core;'; w $p core.rs 'pub fn g() { { use std::process::Command; let _ = Command::new("x"); } }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core

# --- B: a function-local `use` of a name the module also imports
p=B-2021-fn-use-shadows-module-use; new $p 2021; ab $p same
w $p core.rs $'#[allow(unused_imports)]\nuse crate::a::X;\npub fn g() -> u16 { use crate::b::X; X::f() }'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core
p=B3-2021-local-use-after-module-use; new $p 2021; ab $p
w $p core.rs $'use crate::a::X;\npub fn h() -> u8 { X::fa() }\npub fn g() -> u16 { use crate::b::X; X::fb() }'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core
p=B4-2021-local-use-before-module-use; new $p 2021; ab $p
w $p core.rs $'pub fn g() -> u16 { use crate::b::X; X::fb() }\npub fn h() -> u8 { X::fa() }\nuse crate::a::X;'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core
p=F-2021-block-item-shadows-module-use; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs $'#[allow(unused_imports)]\nuse std::process::Command;\npub fn g() { struct Command; impl Command { fn new(_: &str) -> Self { Command } } let _ = Command::new("x"); }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core
p=G-2021-use-visible-before-its-statement; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs 'pub fn g() { let _ = Command::new("x"); use std::process::Command; }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core

# --- C: qualified paths beginning with `<`
p=C1-2021-qualified-self-type; new $p 2021; w $p lib.rs 'pub mod core;'; w $p core.rs 'pub fn g() -> std::time::SystemTime { <std::time::SystemTime>::now() }'
build $p 2021 > /dev/null; row "$p" $p std::time crate::core
p=C2-2021-qualified-as-trait; new $p 2021; w $p lib.rs $'pub mod clock;\npub mod core;'
w $p clock.rs $'pub trait Clock { fn now() -> u64; }\npub struct S;\nimpl Clock for S { fn now() -> u64 { 0 } }'
w $p core.rs 'pub fn g() -> u64 { <crate::clock::S as crate::clock::Clock>::now() }'
build $p 2021 > /dev/null; row "$p[crate::clock::Clock]" $p crate::clock::Clock crate::core; row "$p[crate::clock]" $p crate::clock crate::core
p=C3-2021-qualified-used-names; new $p 2021; w $p lib.rs $'pub mod clock;\npub mod core;'
w $p clock.rs $'pub trait Clock { fn now() -> u64; }\npub struct S;\nimpl Clock for S { fn now() -> u64 { 0 } }'
w $p core.rs $'use crate::clock::{Clock, S};\npub fn g() -> u64 { <S as Clock>::now() }'
build $p 2021 > /dev/null; row "$p[crate::clock::Clock]" $p crate::clock::Clock crate::core; row "$p[crate::clock]" $p crate::clock crate::core
p=C4-2021-qualified-std-trait; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs $'use std::time::SystemTime;\npub fn g(t: &SystemTime) -> SystemTime { <SystemTime as Clone>::clone(t) }'
build $p 2021 > /dev/null; row "$p" $p std::time crate::core

# --- D: edition 2015 root-relative paths beside an `extern crate`
p=D-2015-extern-crate-and-local; new $p 2015 md5x
w $p lib.rs $'extern crate md5x;\npub mod clock;\npub mod core;\npub mod use2015;'
w $p clock.rs 'pub fn now() -> u64 { 0 }'
w $p core.rs $'pub fn g() -> u64 { ::clock::now() }\npub fn h() -> u32 { ::md5x::compute() }\npub fn i() -> u32 { md5x::compute() }'
w $p use2015.rs $'use clock::now;\npub fn j() -> u64 { now() }'
build $p 2015 > /dev/null
row "$p[crate::clock]" $p crate::clock crate::core
row "$p[md5x]" $p md5x crate::core
row "$p[::md5x]" $p ::md5x crate::core
row "$p[crate::clock]@use2015" $p crate::clock crate::use2015
p=D-2018-control-use-without-crate; new $p 2018; w $p lib.rs $'pub mod clock;\npub mod use2015;'; w $p clock.rs 'pub fn now() -> u64 { 0 }'; w $p use2015.rs $'use clock::now;\npub fn j() -> u64 { now() }'
echo "$p | build $(build $p 2018)"

# --- E: two globs bringing one name
p=E1-2021-two-globs-used-name; new $p 2021; ab $p same; w $p core.rs $'use crate::a::*;\nuse crate::b::*;\npub fn g() { let _ = X::f(); }'
build $p 2021 > /dev/null; row "$p" $p crate::a crate::core
p=E2-2021-two-globs-unused-name; new $p 2021; ab $p same; w $p core.rs $'#[allow(unused_imports)]\nuse crate::a::*;\n#[allow(unused_imports)]\nuse crate::b::*;\npub fn g() {}'
build $p 2021 > /dev/null; row "$p" $p crate::a crate::core
p=E3-2021-explicit-beats-glob; new $p 2021; ab $p same; w $p core.rs $'#[allow(unused_imports)]\nuse crate::a::*;\nuse crate::b::X;\npub fn g() -> u16 { X::f() }'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core
p=E4-2021-cfg-alternative-globs; new $p 2021; ab $p same; w $p core.rs $'#[cfg(unix)]\nuse crate::a::*;\n#[cfg(not(unix))]\nuse crate::b::*;\npub fn g() { let _ = X::f(); }'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core
p=E5-2021-same-item-two-globs; new $p 2021; w $p lib.rs $'pub mod a;\npub mod b;\npub mod core;'; w $p a.rs $'pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }'; w $p b.rs 'pub use crate::a::X;'
w $p core.rs $'use crate::a::*;\nuse crate::b::*;\npub fn g() -> u8 { X::f() }'
build $p 2021 > /dev/null; row "$p" $p crate::a crate::core
p=E6-2021-cfg-alternative-named-uses; new $p 2021; ab $p same; w $p core.rs $'#[cfg(unix)]\nuse crate::a::X;\n#[cfg(not(unix))]\nuse crate::b::X;\npub fn g() { let _ = X::f(); }'
build $p 2021 > /dev/null; row "$p[crate::a]" $p crate::a crate::core; row "$p[crate::b]" $p crate::b crate::core

# --- H: braces that open no name scope, and a generic parameter
p=H1-2021-associated-const-does-not-shadow; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs 'use std::process::Command; pub struct S; impl S { const Command: u8 = 0; pub fn f() { let _ = Command::new("x"); } }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core
p=H2-2021-generic-parameter-shadows-use; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs '#[allow(unused_imports)] use std::process::Command; pub fn f<Command: Default>() -> Command { Command::default() }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core
p=H3-2021-enum-variant-does-not-shadow; new $p 2021; w $p lib.rs 'pub mod core;'
w $p core.rs 'use std::process::Command; pub enum E { Command } pub fn f() { let _ = Command::new("x"); }'
build $p 2021 > /dev/null; row "$p" $p std::process crate::core
```
