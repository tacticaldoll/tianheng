## Context

`crates/louke/src/audit.rs::audit_probe_coverage` scans source for `assert_boundary!` probes whose
seam argument is not a string literal ("un-auditable"). Today it reacts once **per file**:
`unauditable_files: Vec<&str>` collects `Probe::Unauditable { file }`, sorts, and dedups by file
alone before emitting one violation per file. `Probe::Unauditable` (`crates/louke/src/audit/scan.rs`)
carries only the file. `capture_probe` has three un-auditable branches: two (a malformed raw string,
and "anything else" — a const or expression) bail at the start position without parsing further;
the third (a plain string whose escapes fail to decode) already scans to the closing quote before
failing, so `start..j` is available there but currently discarded. None of the three currently
retains the expression text.

This file-level granularity is recorded in `BACKLOG.md` as accepted debt: two distinct un-auditable
expressions in one file collapse to one baseline identity, so baselining one can mask a different,
later-added one in the same file.

The 0.3.0 identity migration (`structured-violation-identity`) makes the fix affordable — every
fact's identity is now a set of named, content-derived fields — but it also **forecloses the naive
fix**: `structured-violation-identity/spec.md` explicitly forbids deriving identity from "traversal
position, ordinal, or collection index," and two other dimensions (`hunyi`'s async-exposure and
unsafe-confinement collectors) already enforce this by **failing loud** rather than falling back to
a positional identity when no structural discriminator exists. A raw byte offset is exactly the kind
of positional value this project's identity model rejects. `hunyi` also carries the precedent for
what a *sufficient* structural discriminator looks like for a same-named-item collision: it
qualifies by `owner` (the `Self` type) and, for a trait impl, additionally by `trait_ref` — never a
bare method name alone. This change follows that shape, not just the "fail loud" half of the
precedent.

## Goals / Non-Goals

**Goals:**
- Give each un-auditable probe an identity derived from source **content** (the offending
  expression's own text, and its owner-qualified enclosing item), not position.
- Close the false-negative risk where baselining one un-auditable probe masks a different one in
  the same file.
- Stay `syn`-free in 漏刻's audit scanner (its self-law: no new dependency beyond the existing
  `xingbiao` `audit`-feature exception).

**Non-Goals:**
- Fully disambiguating two byte-identical expressions in the same file and the same owner-qualified
  enclosing item — see Decisions below; this is a stated bound, not attempted.
- Any change to the prod-hot-path `assert_boundary!`/`install` behavior — this is CI-audit-only
  (`audit` feature).
- Line/column reporting for human presentation — out of scope for this change (could be a later,
  presentation-only addition; identity does not need it).

## Decisions

### 1. Identity discriminator is expression text + enclosing fn/impl name, not byte offset

**Why not byte offset:** forbidden by `structured-violation-identity/spec.md` ("no ... fallback
... derived solely from traversal position") and inconsistent with the precedent set by `hunyi`'s
async/unsafe collectors, which fail loud rather than use a positional identity.

**Why expression text:** it is the actual distinguishing content of the fact — two different
non-literal seam expressions are, in fact, two different problems. Capturing it requires parsing to
the end of the first macro argument (top-level comma or matching close-delimiter). **Correction
after adversarial review:** the reusable precedent for this is `foreign_macro_body_end`
(`scan.rs:703-748`), not `balanced_brace_end` — `balanced_brace_end` tracks `{`/`}` depth only, so a
seam expression containing a nested call or index (`assert_boundary!(some_fn(a, b), obj)`,
`assert_boundary!(TABLE[i], obj)`) would truncate at the wrong comma if built on it.
`foreign_macro_body_end`'s "one depth counter over all three delimiter kinds" loop, skipping
literals/comments via the shared `skip_literal_or_comment`, is the correct model; this change's new
helper differs only in stopping at a **top-level comma** as well as the matching close-delimiter
(finding the first argument, not the whole macro body) — that comma-stop is the one genuinely new
piece of parsing logic this change adds.

**Why also an owner-qualified enclosing item, not a bare name:** without full qualification, two
identical expressions in *different* impls of the same method name would still collide — a real
false negative, and in fact the *identical* collision class `hunyi`'s unsafe-confinement and
signature-coupling facts were built to close (see `PROJECT.md` Decisions and
`semantic-trait-impl-locality`): `hunyi` qualifies by `owner` (the `Self` type) and, for a trait
impl, additionally by `trait_ref` (`UnsafeSiteFact::InherentMethod{owner,name}` /
`TraitImplMethod{trait_ref,owner,name}` in `crates/hunyi/src/finding.rs`) — never by a bare method
name alone, because two types can share a method name. This change's enclosing-item tracker must
follow the same shape: for a free `fn`, the module path + fn name; for a method inside `impl Type`,
the `Self` type + method name; for a method inside `impl Trait for Type`, the trait path + `Self`
type + method name; for a trait's own default-body method, the trait name + method name. A bare
innermost name (the first draft of this design) is **not sufficient** and was corrected before
implementation — see the added scenario "Same-named method in two different impls stays distinct."
The scanner currently does no enclosing-scope tracking at all (it is a flat per-file linear scan);
this change adds a small brace-depth-tracked owner-chain tracker, in the same hand-rolled,
`syn`-free style as `foreign_macro_body_end`/`balanced_brace_end`.

**Alternative considered — a fixed per-file occurrence counter:** rejected outright; this is exactly
the "collection index" the spec forbids, and it is also *fragile*: inserting or removing an
unrelated probe earlier in the file would silently renumber every later occurrence, churning
baselines for code that did not change semantically.

### 2. Byte-identical expressions in the same file and the same owner-qualified enclosing item collapse to one finding

At that granularity — same file, same fully-qualified owner (module path, or `Self` type / trait
path + method name), same expression text — there is no more source content left to read: the two
occurrences are, in every content-derived sense, the same fact restated. This mirrors the existing
`module-boundary` precedent ("the same import repeated on multiple lines is one violation"): a
stated bound, recorded explicitly in the spec delta, not a silent gap. This is a materially
narrower bound than "same file" alone (the original, rejected framing) — it survives adversarial
review only because the enclosing item is now fully owner-qualified (Decision 1), not a bare name.

### 3. This ships as a patch (false-negative closure), following the v0.1.3 re-export-exposure precedent

The change can only ever **increase** the number of violations reported for a given codebase, never
decrease or reinterpret one — the defining shape of a false-negative closure under this project's
SemVer discipline (`AGENTS.md`; `BACKLOG.md`'s v0.1.3 re-export-exposure entry is the recorded
precedent for exactly this trade). No public API, wire format, or baseline document shape changes;
only the identity *values* for this one fact type change.

**Acknowledged divergence from BACKLOG's own prior sketch:** `BACKLOG.md`'s accepted-debt entry for
this exact gap recorded a remediation sketch — "qualify the finding by a per-probe locator (byte
offset / occurrence index)" — written before the 0.3.0 identity migration hardened the
no-positional-identity rule. This change deliberately does **not** follow that sketch; it is
superseded, not silently dropped. `tasks.md`'s BACKLOG-cleanup task says so explicitly rather than
just replacing the entry text.

## Risks / Trade-offs

- **[Risk]** An existing dirty codebase with multiple un-auditable probes in one file will see its
  violation count jump and its baseline go stale on this file's entries.
  **Mitigation:** this is the same adopter-facing ratchet-down flow every prior false-negative
  closure has used (`--write-baseline` regenerates; `COOKBOOK.md` already documents the flow) — not
  a new mechanism.
- **[Risk]** The new owner-qualified enclosing-item tracker is new scanning surface (more code = more
  chance of a scanning bug) in a crate whose own self-law demands a light, `syn`-free hot-path-adjacent
  scanner.
  **Mitigation:** the tracker only runs under the non-default `audit` feature (CI-only face), the
  same isolation every other audit-only capability already relies on; reuses the existing
  `foreign_macro_body_end`/`skip_literal_or_comment` primitives rather than adding a new parsing
  strategy, and mirrors `hunyi`'s already-proven owner-qualification shapes rather than inventing a
  new one.
- **[Trade-off]** Byte-identical duplicate probes in the same file, same owner-qualified scope, and
  same expression text remain a stated bound rather than fully resolved. Accepted: closing it would
  require positional identity, which the project's own identity model forbids, and the residual case
  is narrow (identical code, identical owner, identical file).

## Open Questions

- Exact BACKLOG.md classification for this change once shipped (READY-PATCH vs. direct maintenance
  PR) — likely moot once implemented; the accepted-debt entry should simply be resolved and removed
  at that point.
