## Why

漏刻's CI-face audit scanner (`crates/louke/src/audit/scan.rs`) is a hand-rolled byte scanner —
independent from 圭表's and 渾儀's `syn`-based walkers by design (三儀 ⊥ 三儀) — that resolves a
target's reachable module graph to find every `assert_boundary!` probe. Three independent gaps in
that resolution, each reproduced directly:

- **A comment between `mod` and its name (or between the name and its terminator) drops the whole
  module.** `pub mod /* relocated */ child;` is unremarkable, trivia-only Rust to rustc, but the
  scanner's name-skip only advanced past ASCII whitespace, not comments — so the identifier scan
  found nothing at the comment's leading `/` and the declaration was never recognized as a `mod` at
  all. Not a graceful skip: the module and its entire subtree, and every probe beneath it, silently
  vanished from the corpus (exit 0 Clean on source 圭表 and 渾儀 both react to).
- **The catch-all brace skip drops the only legal non-inline module form inside a function/block
  body**, `#[path] mod name;` (a bare `mod name;` with no established file-path convention there
  does not compile without one). Every brace the walker didn't specifically recognize (a fn body, a
  bare block, …) was treated as one opaque, unwalked unit — so this legal, compiling form's probes
  were never counted at all, with no loud signal either (unlike the missing-module path, which fails
  loud).
- **`mod_preamble_attrs` documented a `cfg_attr(path)` tolerance the code never implemented.** The
  attribute-matching pass checked for the exact identifier `cfg`; `cfg_attr` is a different
  identifier and matched neither the `path` arm nor the bare-`cfg` arm. A module stacking two
  `cfg_attr`-wrapped `#[path]` declarations that together cover every platform (both targets
  present, compiling cleanly everywhere) was reported a hard constitution error instead of being
  scanned — a false positive blocking CI on entirely valid code, and the exact contradiction the
  finding names between the doc's claim and the implementation.

## What Changes

- A new `skip_space_and_comments` (mirroring the existing `skip_ascii_space`) is used at both
  positions between `mod` and its terminator, so a comment there is trivia, never a corpus drop.
- The catch-all `{…}` handler in `collect_scope_modules` now descends into any unrecognized brace
  scope (a fn/const/static body, a bare block, a match arm, …) instead of skipping it as opaque —
  `is_mod_keyword`'s own whole-word match means this costs nothing on ordinary code (no real `mod`
  token exists inside a struct literal or match arm to misfire on), and finds a legal `#[path] mod`
  wherever Rust permits one. A `mod` found this way adds no directory component of its own (unlike a
  NAMED inline `mod x { … }`), so the enclosing bases thread through unchanged; arm membership is
  inherited into it.
- `ModPreambleAttrs` gains `cfg_attr_paths: Vec<String>` — every `path = "…"` value found inside a
  `#[cfg_attr(<pred>, …, path = "…")]` wrapper on the declaration (a module may stack more than one,
  one per platform predicate). The consumer unions every existing candidate — each `cfg_attr` target
  that resolves, plus the conventional file — as separate sources, matching the crate-wide walk's
  own per-platform-pair handling. Absence is tolerated only when NEITHER any `cfg_attr` target NOR
  the conventional file resolves anywhere, and the declaration carries no other cfg-conditional gate
  (a bare `#[cfg]` or transparent-arm membership) — a genuinely broken reference on every
  configuration, so it still fails loud.
- The identical `cfg_attr(path)` union also applies to an **inline** `mod x { … }` (a body, not a
  `;`-terminated external declaration) — adversarial review found the first cut wired
  `cfg_attr_paths` into only the external-module consumer, leaving the inline-module consumer still
  reading `attrs.path` alone. Here the union governs which **base directory** x's own nested items
  resolve from (not a file to read — the body is already present in source), so each candidate is
  descended only when it exists as a directory; if none does, the conventional base is used anyway so
  a nested reference genuinely broken on every platform still fails loud (unchanged from before this
  fix existed).
- A doubly-**nested** `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is a stated, undetected bound here (a
  hand-rolled byte scanner, unlike `hunyi`'s `syn`-based recursive walk) — not attempted, and
  documented as such rather than silently claimed.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `runtime-origin-assertion`: the "Root-aware audit excludes unreachable source files" requirement's
  own stated `#[path]`/`cfg_attr` resolution rules are corrected and extended with the union rule
  above; new scenarios for the comment-skip and block-scoped-path-mod fixes.

## Impact

- Affected code: `crates/louke/src/audit/scan.rs`, `crates/louke/src/audit.rs` (public doc only).
- No public API/DSL change, no baseline format change (this fixes false negatives and one false
  positive, not an identity shape — an adopter's existing baseline is unaffected either way).
- Out of scope, named explicitly rather than silently left: a doubly-nested `cfg_attr(cfg_attr(path))`
  remains undetected by this hand-rolled scanner (a stated bound), unlike `hunyi`'s `syn`-based
  recursive handling of the identical shape.
