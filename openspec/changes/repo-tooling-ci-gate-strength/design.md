## Context

Two repository CI gates each currently pass green in a state their own stated purpose says they
should fail on. Neither is a Tianheng capability bug (no `Constitution`/`Boundary`/reaction code
changes) — both are the tooling that governs the workspace itself.

Reproduced directly:

```
# 1. deny.toml's unset `yanked` defaults to "warn".
$ cargo update -p openssl-probe --precise 0.1.3   # a real yanked version, pinned into the lockfile
warning: selected package `openssl-probe@0.1.3` was yanked by the author
$ cargo deny check
warning[yanked]: detected yanked crate (try `cargo update -p openssl-probe`)
advisories ok, bans ok, licenses ok, sources ok    # exit 0
```

```
# 2. test_examples.sh's patch.crates-io silently drops when incompatible.
$ sed -i 's/version = "0.3.0"/version = "0.4.0"/' Cargo.toml   # AGENTS.md's own pre-1.0 bump rule
$ cd examples/guibiao-standalone && cargo tree -p guibiao --config 'patch.crates-io.guibiao.path="..."' ...
warning: patch `guibiao v0.4.0 (...)` was not used in the crate graph
    Adding guibiao v0.3.0 (available: v0.4.0)     # silently resolved from crates.io instead
guibiao v0.3.0                                     # no local path in the tree output
```

## Goals / Non-Goals

**Goals:**
- `cargo deny check` fails on a yanked dependency in the graph, matching `deny.toml`'s own stated
  claim.
- `scripts/test_examples.sh` fails loud, identifying the exact crate and example, when a
  `patch.crates-io` override is silently dropped — rather than passing against a stale published
  crate with no signal at all.

**Non-Goals:**
- No change to any Tianheng capability, `Constitution`, boundary, or reaction — both fixes are
  entirely in repo-tooling configuration/scripts.
- No general-purpose "detect any silently-dropped Cargo patch" library — the `assert_patched` check
  is scoped to this script's own existing `PATCH` array construction, the same pattern the script
  already uses throughout (`quality_gates "${PATCH[@]}"`, `cargo test "${PATCH[@]}"`).

## Decisions

- **`yanked = "deny"` as an explicit field, not a version bump or schema change.** `deny.toml`'s
  `version = 2` schema already supports the field; the only gap was never setting it. Adding one
  line with a comment recording WHY the default alone is insufficient (an unset field silently
  reads as `"warn"`, and there is no lint that flags an unset security-relevant field) closes the
  gap with the smallest possible diff.
- **`cargo tree -p <crate> ... --depth 0`, not `cargo metadata` + JSON parsing.** No `jq` dependency
  exists anywhere in `scripts/` today, and `cargo metadata`'s JSON would require parsing to find the
  resolved package's `manifest_path`/`source` for a crate that may not even be a direct dependency
  of the example (family crates are pulled in transitively too). `cargo tree`'s own resolved-package
  header line already renders the local path in parens for a path/patch dependency (confirmed
  empirically, both for the success case and the silently-dropped-patch case above) with zero new
  dependencies and a one-line pattern match.
- **The check runs once per example, immediately after that example's own `PATCH` array is built,
  reusing the same array** — not a separate top-level pass — so a failure names the exact example
  and crate whose patch was dropped, and so it composes with the script's existing per-example
  structure (`mapfile -d '' PATCH < <(patch ...)` immediately followed by the checks that consume
  it) rather than introducing a second, disconnected pass over the same data.

## Risks / Trade-offs

- **[Risk] `cargo tree`'s output format is not a stable, versioned contract.** → **Mitigation**: the
  parenthesized source-path rendering for a path dependency is long-standing, widely-relied-upon
  `cargo tree` behavior (predates the `--depth` flag itself); `--depth 0` further limits the surface
  contract to exactly the root line. If a future cargo changes this rendering, the check fails loud
  (a pattern-match miss), not silently — an acceptable trade for zero new tooling dependencies.
- **[Risk] `yanked = "deny"` could newly fail CI on an ALREADY-yanked transitive dependency that was
  previously silently tolerated.** → **Mitigation**: `cargo deny check` currently passes clean
  (`advisories ok`) on the real workspace graph — verified before this change — so this closes a
  latent gap without breaking the present build; a future yank would (correctly) now need an
  explicit `ignore` entry with a recorded rationale, exactly as `deny.toml`'s own comment already
  demands for a known-vulnerable advisory.

## Migration Plan

1. Add `yanked = "deny"` to `deny.toml`'s `[advisories]` table.
2. Add `assert_patched` to `scripts/test_examples.sh`; call it for every example immediately after
   that example's `PATCH` array is built, over every family crate that example patches.
3. Non-vacuous verification: reproduced both failure modes against a real yanked crate and a
   scratch, version-bumped copy of the workspace; confirmed each fix's own check fails loud in
   exactly that scenario, then confirmed the real (un-bumped, no-yanked-dep) workspace still passes
   clean.
4. Added a `governance-dogfood` spec scenario for the `test_examples.sh` fix; `deny.toml`'s fix
   carries no capability delta (see Non-Goals).
5. CHANGELOG `[Unreleased]` entry. No **BREAKING** marker — strengthens two CI gates, no product
   surface changes. No version bump (campaign-wide constraint).

## Open Questions

None outstanding.
