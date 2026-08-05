## Why

A second adversarial review of the observation-bound register found three ways a `PINNED-BY` citation still
resolves to something that is not the test it names. Each was reproduced before being accepted:

- **A citation containing a regex metacharacter resolves to a differently-named function.** The cited name is
  interpolated into an ERE, so `- **PINNED-BY** \`a_probe_bound_is_pinne.\`` resolves to
  `a_probe_bound_is_pinned` and the gate reports clean. This defeats the register's original direction — a
  renamed or deleted test must not read as coverage — because a citation for a test that does not exist
  passes.
- **A crate qualifier is concatenated into a path, so `../` leaves `crates/`.**
  `- **PINNED-BY** \`../outside::a_fn\`` resolves against a function outside the boundary the requirement
  declares, and the gate reports clean.
- **A `#[test]` inside a block comment satisfies the attribute run.** `/*`, `#[test]`, `*/`, `pub fn …` is
  accepted, so a function that never runs occupies the place of the defence. The matrix covered `// #[test]`
  and not this.

A fourth defect runs the other way: the attribute walk is capped at 12 lines, so a legitimate test with 13
interleaved attributes is **refused**. No length limit is declared anywhere; the cap is arbitrary.

Reproducing the block-comment case exposed a wider instance the review did not reach: a function definition
that is *itself* inside a block comment also satisfies the citation, and always has. That one cannot be
closed here, for a reason this tree measures rather than asserts — see below.

The first two are not a security exposure and are not treated as one. The input is repository-controlled
prose in a tracked spec, so anyone who can write it can edit the gate beside it; no privilege boundary is
crossed. They are **false coverage**, which is the class this capability exists to end, and which makes them
more urgent than a security framing implies rather than less.

## What Changes

- A cited name SHALL be validated as a Rust identifier and a crate qualifier as a crate-directory name,
  with at most one `::`. This closes the metacharacter and traversal directions **by construction** rather
  than by escaping: an invalid citation is refused with its own diagnostic. All 36 citations in the tree are
  already plain identifiers and all crate directories are plain names, so nothing is migrated.
- The attribute walk SHALL stop at a block-comment delimiter (`/*` or `*/`) rather than interpret it. Not
  stripping and not tracking: comment state is a forward property of a file that an upward walk cannot know,
  and stripping needs string-literal lexing — this tree's own lexer fixtures carry 49 `/*` occurrences
  **inside string literals**, several nested, so a delimiter-counting stripper would manufacture phantom
  comments and swallow real definitions. No real `#[test]` run in the tree contains a block comment, and
  none of the 36 cited tests is affected.
- The attribute walk SHALL run to the real item boundary instead of an arbitrary line cap.
- **A definition inside a block comment is declared a bound of this capability**, pinned by a fixture, with
  the measured reason recorded — and the requirement's claim that a citation "cannot be satisfied by a
  comment" is narrowed to the mention forms it actually refuses. This is the third claim in this capability
  found wider than its reaction; recording the residual is the treatment its own rules prescribe.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: citation syntax is validated before resolution; test recognition stops at a
  block-comment delimiter and runs to the item boundary rather than a line cap; the definition-form claim is
  narrowed and its residual declared as this capability's first bound.

## Impact

- `scripts/check_bound_register.sh` — citation validation, the attribute walk.
- `scripts/test_bound_register.sh` — a fixture per new refusal, plus the long-attribute-run and
  commented-definition directions.
- `openspec/specs/observation-bound-register/spec.md` — the resolution requirement, and its first declared
  bound.
- `docs/observation-bounds.md` — regenerated (one more declared bound).
- No crate, no public API, no adopter action.
