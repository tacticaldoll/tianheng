## Context

Two independent matching mechanisms sit downstream of a forbidden/allowed operand string, and both
silently miss a malformed one instead of erroring:

**Path-prefix containment** (`containment::path_within`, `matches_forbidden`, used by
signature-coupling and the dyn/impl-trait operand boundaries):

```rust
fn path_within(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
}
```

The `path` side is always a canonical string this crate resolved from a real `syn::Path` — and
`extern_verbatim_renamed`/`extern_verbatim_segs` (`resolve/mod.rs`) build that string purely from
`path.segments`, never consulting `leading_colon`:

```rust
let mut segs: Vec<String> = path.segments.iter().map(|s| strip_raw(&s.ident.to_string())).collect();
```

So `-> ::serde::Value` resolves to `"serde::Value"` — never `"::serde::Value"`. A forbidden entry of
`"::serde"` survives `canonical_path_str` unchanged (it strips a raw-identifier prefix per segment,
not empty segments), so `path_within("serde::Value", "::serde")` is unconditionally `false`: neither
equal, nor does `"serde::Value"` start with `"::serde::"`. A trailing (`"serde::"`) or doubled
(`"::serde::"`) form is unconditionally `false` for the identical reason — a real resolved canonical
path never contains an empty segment (rustc's own grammar forbids one), so an operand shaped this way
can never equal or prefix-contain one.

**Leaf-identifier matching** (`containment::leaf_of`, used only by forbidden-marker):

```rust
pub(crate) fn leaf_of(path: &str) -> &str {
    let leaf = path.rsplit("::").next().unwrap_or(path);
    leaf.strip_prefix("r#").unwrap_or(leaf)
}
```

This one is immune to a *leading* `::` (`leaf_of("::serde::Serialize")` is still `"Serialize"`) but
not to a *trailing* one: `leaf_of("serde::")` is `""`, and no real identifier is ever empty, so it
can never match.

## Measured reproduction

A throwaway regression probe (added to `crates/hunyi/src/tests.rs`, run, then reverted before this
change existed — not part of the audit investigation's final diff) against module `crate::api`
declaring `pub fn ext() -> ::serde::Value { unimplemented!() }`, `serde` a real dependency:

| `must_not_expose(...)` operand | outcome |
| --- | --- |
| `"serde"` | reacts: `serde::Value exposed by fn crate::api::ext` |
| `"::serde"` | `[]` — silent pass |
| `"serde::"` | `[]` — silent pass |
| `"::serde::"` | `[]` — silent pass |

## Decisions

### Decision 1: One shared predicate and error, not four capability-specific copies

`has_empty_path_segment` (`resolve/mod.rs`, beside `canonical_path_str`, which every affected call
site already imports) and `malformed_path_operand_error` (`errors.rs`, beside the other constitution-
error builders) are written once and called from all four affected sites. `containment.rs`'s own
module doc states the reason for this shape of consolidation directly: "the single home of the
containment rule, so no copy drifts to a bare `starts_with`" — the identical argument applies to a
copy of the malformed-operand check drifting out of sync across four call sites.

### Decision 2: Reject every empty `::`-segment, not only a leading one

The audit's trigger named the leading-`::` case; measuring it found the trailing and doubled cases
are equally broken for path-prefix containment, and forbidden-marker's leaf matching has the mirror
image (trailing breaks it, leading does not). Two options:

- A per-mechanism rule (reject leading `::` only where it breaks path containment; reject trailing
  `::` only where it breaks leaf matching) — accurate to each mechanism's own sensitivity, but it
  requires the caller to know which matcher its capability uses and forces two validation functions
  to stay in sync with two internal implementation details that are themselves subject to change.
- One uniform rule: reject *any* empty segment, everywhere a forbidden/allowed operand of this shape
  is accepted, regardless of which mechanism happens to be sensitive to which end. `"::serde"` is
  rejected for `must_not_acquire` even though leaf-matching alone would tolerate it.

Taking the uniform rule: it is simpler to state and simpler to keep correct if a capability's
matching mechanism ever changes, and rejecting a leading `::` on `must_not_acquire` costs nothing —
across this entire DSL, a leading `::` in an operand string is *never* semantically distinct from the
bare form (no resolved canonical path this crate produces ever carries one, by construction), so
"harmless when unrejected" and "meaningfully different when rejected" are never both true for the
same spelling. A single rule that is occasionally stricter than the narrowest mechanism requires is
worth the consistency; a rule that is exactly as strict as needed per mechanism is not worth tracking
four times.

### Decision 3: Constitution error (exit 2) at check-time, not a construction-time panic

Two candidate failure modes:

- **Panic (or `Result`) at DSL-construction time** — `must_not_expose(path: &str)` etc. reject the
  operand the instant it is written. Catches the mistake at the earliest possible moment, since no
  crate context is needed to know a `::`-path string has an empty segment.
- **`Result<_, String>` from the pure heart, surfaced as a constitution error (exit 2) at check
  time** — the mechanism every other "developer wrote something structurally wrong" case in this
  boundary type already uses: `unknown_module_error`, `unknown_trait_error`,
  `dual_backed_module_error`, and — the closest precedent, both textually and structurally —
  `unsafe_empty_allowed_error`/`unsafe_crate_root_allowed_error`, guarded in `unsafe_findings` before
  any scanning, for the identical shape of problem ("this configuration could never react").

Chosen: check-time constitution error, for two reasons. First, no builder method anywhere in this
DSL validates a string argument eagerly today — not even `because(reason)`, whose own spec text
requires "a non-empty reason." Introducing a panic for only this one case would be a new
error-reporting shape unlike anything else in the DSL, inconsistent with how every sibling
malformed-input class is handled. Second, and more binding: this project's entire failure surface for
"cannot judge" / "would silently never react" is the `Outcome`/`Violation` contract — the same one
`--format json` and `--format sarif` project — and every one of the precedents above deliberately
routes through it rather than a Rust-level panic at the call site, which would crash with a raw
backtrace instead of a formatted report an adopter's CI tooling already knows how to parse. A panic
would be earlier but incompatible with the one reporting surface this system is built around; a
constitution error is exactly as loud (still exit 2, never exit 0, never a violation misreported as
architectural drift) and stays inside it.

## Goals / Non-Goals

**Goals:**
- Close the silent-pass class for every forbidden/allowed-operand-shaped DSL method whose matching
  mechanism is provably sensitive to an empty `::`-segment.
- Keep the source-spelling advice (`::serde::Value` in scanned code disambiguates from a local
  shadow) and the operand-spelling restriction (the DSL string itself must have no empty segment)
  textually distinct in the specs, so a future reader cannot re-conflate them the way this finding's
  own framing did.

**Non-Goals:**
- `unsafe_confinement`'s and `trait_impl`'s `allowed_locations` (`matches_allowed`/`path_within`
  against a crate-relative module path, never extern-resolved). A malformed entry there makes every
  real site look disallowed — spurious violations, a fail-loud (if noisy) direction, not the silent
  pass this change closes. Worth tightening on its own terms eventually, but a different failure
  class deserves its own review rather than riding along on this one's scope.
- `TraitImplBoundary::trait_(...)`'s anchor. `trait_impl_findings` already returns
  `unknown_trait_error` when the canonicalized anchor matches no real local trait definition
  (including, trivially, a malformed spelling) — the anchor-resolution backstop every boundary
  already has, so no new check is needed there.
- A boundary's own module anchor (`.module(...)`) generally — resolved and fail-loud via
  `resolve_crate`/module resolution independently of this change.
