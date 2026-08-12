## Why

`reference-integrity` already says *A reference SHALL name a thing, not a position*, and its reaction sweeps
every line-comment format. Markdown sits outside **by construction**, for a reason that holds: in a record a
positional *phrase* narrates a past state, and separating that from a live reference is a judgement over prose
this repository has designed, measured and declined.

That reasoning covers phrases. It does not cover a **structured coordinate** — a backticked `` `path:NNN` `` —
which is decidable by shape exactly as a bound id is, and needs no reading of the prose around it. Two live
ones exist, both in one `BACKLOG.md` clause, and both have rotted: they were correct when written on
2026-08-09 and now land mid-paragraph in unrelated entries. The `0.5.0` window pushed them further, so the
class is demonstrably load-bearing rather than theoretical.

A changelog entry in this same unreleased window claims line-number citations are *"gone from tracked
content"*. They are gone from tracked **source**, which is the scope the reaction holds; the two in Markdown
are what the claim walked past.

## What Changes

- The requirement gains the structured-coordinate form, refused in **every** tracked format, Markdown
  included. The prose-phrase exclusion for Markdown is untouched and restated as still standing.
- `crates/kanhe/tests/reference_integrity.rs` refuses a backticked `<tracked-path>:<line>` wherever it appears.
- The two `BACKLOG.md` citations name the entries they meant instead of coordinates.
- The changelog claim is narrowed from *tracked content* to *tracked source*, and says what it left.

Zero migration: after those two repairs the reaction holds an empty set, which this repository already treats
as a shape worth keeping rather than pruning.

## Capabilities

### New Capabilities

<!-- none: the requirement exists and is extended -->

### Modified Capabilities

- `reference-integrity`: *A reference SHALL name a thing, not a position* gains the structured-coordinate form
  and the format scope it is refused in, with the scenario for a coordinate in whole-document prose.

## Impact

- `crates/kanhe/tests/reference_integrity.rs` — one direction.
- `BACKLOG.md`, `CHANGELOG.md` — the two citations and the narrowed claim.
- `openspec/specs/reference-integrity/spec.md` — one requirement extended, one scenario added.
- No published crate, signature, wire format, exit class, baseline or manifest is touched.

### Capabilities touched without a requirement change

- `release-coherence`: `CHANGELOG.md` is its declared subject; the narrowed claim moves no requirement of it.
- `repository-checks`: `crates/kanhe/**/*.rs` is its declared subject; the direction added there moves none of
  its requirements.
