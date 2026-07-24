## Context

The release branch combines a breaking structured-identity migration with additive scanner,
projection, and testing-harness work. Adversarial review found that two new identities can still
collapse or miss compiled source, while several compatibility promises are not executable or are
described inaccurately. The implementation must preserve Tianheng's false-negative intolerance,
crate dependency direction, and 0.2.3 pre-release version state.

## Goals / Non-Goals

**Goals:**

- Make every reviewed contract executable through focused regression tests.
- Keep observation conservative when written conditional path attributes create several possible
  physical module sources.
- Give runtime audit identities a stable lexical owner path without positional ordinals.
- Finish the intentional 0.3.0 public identity break in this branch.
- Make self-law freshness use the same public harness and BLESS semantics taught to adopters.
- Restore release-note and backlog provenance.

**Non-Goals:**

- Evaluate cfg predicates or reproduce rustc's target configuration.
- Add a parser dependency to 圭表.
- Change machine identity formats beyond the already-planned 0.3.0 contract.
- Bump manifests, lockfiles, or package versions.
- Introduce baseline debt-ratchet behavior.

## Decisions

1. **Mixed path attributes use conservative candidate union.** The scanner will collect both direct
   and conditional `path = "…"` candidates in written order, canonicalize/deduplicate resolved
   files, and scan every candidate that physically exists. This matches the established cfg-blind
   union policy and avoids encoding rustc's warned, transitional multiple-path precedence.
   A declaration with no usable candidate continues to fail according to existing resolution rules.

2. **Runtime owners use a lexical chain.** `render_owner` will include the enclosing function owner
   before the nested function/local impl context. Names and structural context are used; byte
   offsets and traversal ordinals remain forbidden identity inputs.

3. **All `ViolationId` components are encapsulated.** `Violation::target` becomes private and gains
   a `target()` accessor. In-tree field reads migrate mechanically; JSON and comparison output stay
   unchanged.

4. **Projection encodes only non-legacy depth.** A legacy/default `Subtree` module boundary omits
   `scan_depth`; explicit `Shallow` emits `"shallow"`. Focused JSON and Markdown tests pin both
   directions.

5. **The public harness owns environment parsing.** `GovernanceTest` exposes the projection
   freshness behavior used by self-governance, recognizes only `BLESS=1` or case-insensitive
   `BLESS=true`, and is tested through isolated temporary artifacts. The lower-level
   `projection_gate` remains a pure bool-driven primitive.

6. **Documentation records, rather than obscures, the shipped surface.** CHANGELOG lists the
   harness, depth toggle, path observation, and hardening fixes. BACKLOG restores the debt-ratchet
   WATCH entry without promoting it.

## Risks / Trade-offs

- **Union-scanning may inspect a candidate not compiled in the current configuration.** → This is
  the deliberate cfg-blind, false-negative-safe trade-off already used for mutually exclusive
  module branches.
- **Lexical owner strings may become longer.** → They are semantic identity fields, not primary
  diagnostics; tests pin stability across unrelated insertion/reordering.
- **Private `target` breaks direct field readers.** → The release is already the coordinated
  breaking identity window; an accessor provides a mechanical migration.
- **Environment-variable tests share process state.** → Serialize them with a test-local mutex and
  restore prior state, or test a pure parser directly if extracted.

