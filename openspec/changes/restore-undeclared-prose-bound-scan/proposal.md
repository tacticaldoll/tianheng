## Why

`openspec/specs/observation-bound-register/spec.md` already states, in full detail, a requirement that
"the reaction SHALL scan `openspec/specs/*` for bound-declaring prose and SHALL fail on any occurrence
outside a declared bound scenario" — with four worked scenarios and three named residuals. This reaction
existed in the shell era (`scripts/check_bound_register.sh`'s `BOUND_PROSE` scan) and was silently dropped
when `64ed18c` migrated shell check gates to Rust self-governance reactions; nothing reimplemented it. A
sentence like "the observation stops here and does not observe X" added to spec prose today, with no
wrapping scenario, produces no failure anywhere in `cargo test --workspace --all-features`. Found by the
v0.4.0..HEAD adversarial contract-audit review, which constructed exactly that falsifier.

The requirement's own residuals section reads as though a real scan exists; it does not. This closes that
gap by giving the requirement its reaction, not by changing what the requirement says.

## What Changes

- Adds the missing reaction to `observation-bound-register`: a scan of every tracked `openspec/specs/*/spec.md`
  for bound-declaring prose (`stated`/`documented`, at most one interposed word, then `bound`/`bounds`,
  word-boundary aware — the shell era's own `BOUND_PROSE` tolerance, ported directly rather than reusing
  `marks_a_bound`) sitting **outside** a declared bound scenario. `marks_a_bound` is a different, stricter
  mechanism: it derives a bound's identity from its heading and was deliberately narrowed to admit **no**
  interposed word (see `observation-bound-register/spec.md`'s "no qualifier" requirement) precisely because
  a qualifier there would double as an unclosed classification. The prose scan matches nothing that feeds an
  id — it is a trigger heuristic, not a declaration — so that reason for tightening does not carry over.
- A match is cleared by either (a) sitting inside a declared bound scenario, (b) sitting under a
  requirement whose own heading names bounds (which then owes at least one declared bound scenario of its
  own), or (c) carrying a resolvable bare `<capability>/<slug>` reference (reusing the existing
  `bare_references` helper).
- A negation directly on the bound noun (`rather than`/`not`/`never` immediately before `a`/`an` [+ one
  word] `bound`/`bounds`) is not a declaration and is excluded, matching the shell era's own measured
  tolerance.
- No dependency added: `kanhe` is dependency-restricted to `(shengmo, tianheng, serde_json)` by this
  family's own self-law, so the trigger/negation matching is hand-written string scanning, not a regex
  crate.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `observation-bound-register`: no requirement wording changes. The requirement ("A bound stated in prose
  but not declared as a scenario SHALL fail") and its four scenarios are already fully specified; this
  change gives it the reaction it currently has none of. The delta spec restates the existing requirement
  unchanged, so sync can confirm text parity rather than merge new prose.

## Impact

- `crates/kanhe/src/bound_register_parse.rs`: new prose-scanning functions (trigger match, negation
  exclusion, requirement/scenario state tracking), reusing existing `marks_a_bound` and `bare_references`.
- `crates/kanhe/tests/bound_register.rs`: a new test wiring the scan into the ordinary suite, plus
  synthetic-text tests for each of the requirement's four scenarios.
- Must scan the live `openspec/specs/*` corpus (34+ files) cleanly — no new false positives. If the real
  corpus contains genuine prior undeclared-prose bounds once the scan runs, those need declaring as proper
  scenarios or excluding by reference, which may touch spec files beyond `observation-bound-register`
  itself (found only once the scan actually runs).
- Repository-internal (`kanhe` is `publish = false`); no adopter-facing API or manifest changes.
