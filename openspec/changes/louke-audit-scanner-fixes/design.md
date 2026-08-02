## Context

`crates/louke/src/audit/scan.rs` is a deliberately separate, hand-rolled byte scanner backing the
CI-face `audit_probe_coverage` — independent from 圭表's and 渾儀's `syn`-based module walkers by
project design (三儀 ⊥ 三儀: the same rule stated three times, never a shared implementation, so a
bug in one dimension's walker cannot silently propagate to the others). It resolves `mod`
declarations from a target root, descending the reachable module graph to find every
`assert_boundary!` probe.

Reproduced directly, all three from the audit sweep:

```rust
// 1. Comment between `mod` and its name drops the whole module and every probe beneath it.
pub mod /* relocated */ child;
pub fn p(o: u8) { assert_boundary!("seam", o); }
// child.rs: pub fn q(o: u8) { assert_boundary!("seaam", o); }  // typo, never seen
```
Measured: `Outcome::Clean` (exit 0) — `child.rs`'s typo'd seam never reacts, though 圭表 and 渾儀 both
react on the identical shape.

```rust
// 2. #[path] mod inside a function body — the only legal non-inline block-scoped form — is dropped.
pub fn f() { #[path = "inner.rs"] mod inner; }
pub fn p(o: u8) { assert_boundary!("seam", o); }
// inner.rs: pub fn q(o: u8) { assert_boundary!("seaam", o); }  // typo, never seen
```
Measured: `Outcome::Clean` (exit 0) — no loud signal either, unlike a missing module.

```rust
// 3. Two cfg_attr(path) declarations together covering every platform: valid on every build.
#[cfg_attr(unix, path = "u.rs")]
#[cfg_attr(not(unix), path = "w.rs")]
pub mod plat;
pub fn p(o: u8) { assert_boundary!("seam", o); }
// u.rs and w.rs both present, both containing a probe.
```
Measured (pre-fix): `Outcome::ConstitutionError("cannot resolve reachable module 'plat' ...")` — a
false positive hard-failing CI on source that compiles cleanly on every configuration, and the exact
contradiction the audit finding names: `mod_preamble_attrs`'s own doc claimed a `cfg_attr(path)`
tolerance ("reads as `cfg`... an absent target is tolerated"), but the attribute-matching `match`
checked for the literal identifier `cfg`, and `cfg_attr` is a different identifier — it matched
neither the `path` arm nor the `cfg` arm, so the doc's claim was never implemented at all.

## Goals / Non-Goals

**Goals:**
- A comment between `mod` and its name, or between the name and its terminator, is trivia — never a
  reason the declaration goes unrecognized.
- Any block scope (fn/const/static body, bare block, match arm, …) is descended for a nested `mod`
  item — the only way the legal block-scoped `#[path] mod` form can ever be found.
- `cfg_attr`-wrapped `#[path]` targets are extracted (one or more per declaration) and unioned with
  the conventional file, exactly mirroring the crate-wide walk's own established per-platform-pair
  handling — never silently preferred over, or excluded in favor of, the other.
- Absence tolerance stays additive, not broadened: neither ANY candidate resolving, nor another
  cfg-conditional gate present, remains a genuine, fail-loud constitution error.

**Non-Goals:**
- A doubly-nested `#[cfg_attr(a, cfg_attr(b, path = "…"))]` — `hunyi`'s `syn`-based resolver handles
  this recursively; extending this hand-rolled byte scanner to the identical recursive depth was
  judged not worth the added complexity for a shape this rare, and is a stated, documented bound,
  not a silent claim of coverage.
- Any change to 圭表's or 渾儀's own walkers — this change is louke-local; the three dimensions'
  module-resolution walkers are deliberately independent (三儀 ⊥ 三儀) and this fix does not touch the
  other two.

## Decisions

- **A dedicated `skip_space_and_comments`, not a broadened `skip_ascii_space`.** `skip_ascii_space`
  has exactly two callers, both in the `mod`-name-skip position; rather than changing its own
  semantics (which could surprise a future caller expecting a plain whitespace-only skip), a new,
  precisely-named function makes the comment-tolerance explicit at the call site.
- **The catch-all brace-skip is generalized to always descend, not conditionally.** `is_mod_keyword`
  is already a precise, whole-word match (`bytes.get(i+3).is_none_or(|b| !is_ident_byte(*b))`), so
  descending into an ordinary struct-literal/match-arm/expression body costs nothing — no real `mod`
  token exists there to misfire on. This is simpler and more robust than trying to special-case "is
  this brace a fn/const/static body" (which would need duplicating `syn`'s own item-vs-expression
  grammar knowledge in a byte scanner) — always descending is correct precisely because the false
  match rate is already zero by construction.
- **`ModPreambleAttrs` gains a new `Vec` field rather than repurposing `cfg`.** Mirrors `hunyi`'s own
  `cfg_attr_path_values` precedent (a `Vec`, since a module may stack more than one `cfg_attr`-wrapped
  `#[path]`), keeping the bare-`#[cfg]` absence-tolerance flag (`cfg`) semantically pure — it is
  never conflated with the additive, path-carrying tolerance `cfg_attr_paths` provides.
- **Nested `cfg_attr(path)` is a stated non-goal, not chased for parity with `hunyi`.** `hunyi`
  recurses through `syn::Meta::List` for free, since it already parses the full attribute AST; this
  scanner would need to hand-roll a second level of paren-balanced sub-scanning for a shape none of
  the audit findings measured. Documented as a bound rather than silently unhandled.

## Risks / Trade-offs

- **[Risk] Always descending into every brace scope regressively re-reads content already covered
  by a more specific handler (e.g. an inline `mod x { … }`'s own body).** → **Mitigation**: the
  specific handlers (`is_mod_keyword` branches, transparent-macro arms) are checked and consumed
  BEFORE the generic catch-all is ever reached in the same position, so there is no double-descent —
  confirmed by the full existing regression suite staying green (109 pre-existing tests unaffected).
- **[Risk] The `cfg_attr_paths` union pushes a duplicate `(file, base)` pair when a `cfg_attr` target
  happens to canonicalize to the same file as another candidate.** → **Mitigation**: the crate-wide
  walk's own top-level `visited: HashSet<PathBuf>` (in `collect_reachable_probes`, keyed on
  canonical path via `xingbiao::try_visit`) already deduplicates every pending module regardless of
  how many times it was pushed — pre-existing infrastructure, unchanged by this fix.

## Migration Plan

1. Add `skip_space_and_comments`; use it at both `mod`-name-skip positions.
2. Generalize the catch-all brace handler to recurse via `collect_scope_modules`.
3. Add `cfg_attr_paths` to `ModPreambleAttrs`; extract via a new `paren_group_end` +
   `find_path_meta_value`; union in the consumer with the conventional-file resolution.
4. Regression: comment-before-name, comment-after-name, block-scoped `#[path] mod` (direct fn body
   and one bare block deeper), two-`cfg_attr`-covering-every-platform (both the reacting-typo and the
   Clean case), and a missing `cfg_attr` target tolerated when the conventional file backs the module.
5. Non-vacuous verification per fix layer: each of the three independently reverted, confirmed its
   own regression tests fail in the predicted way, restored.
6. Updated the stale doc comments (`ModPreambleAttrs`'s own fields, `mod_preamble_attrs`'s function
   doc, and `audit_probe_coverage_with_markers`'s public doc in `audit.rs`) to state the union rule
   instead of the unimplemented "reads as `cfg`" claim.
7. CHANGELOG `[Unreleased]` entry. No **BREAKING** marker — closes false negatives and one false
   positive, not an identity shape; no baseline is invalidated. No version bump (campaign-wide
   constraint).

## Open Questions

None outstanding. Nested `cfg_attr(path)` is an explicit non-goal, not an open question.
