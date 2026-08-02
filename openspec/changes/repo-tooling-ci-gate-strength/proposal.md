## Why

Two repository-tooling gates each currently make a green check mean less than it claims, both
reproduced directly:

- **`deny.toml` claims yanked crates are denied, but the unset `yanked` field defaults to `"warn"`.**
  Reproduced against a real yanked crate (`openssl-probe = "0.1.3"`, pinned into a lockfile via
  `cargo update --precise`): `cargo deny check` prints `warning[yanked]: detected yanked crate` and
  still reports `advisories ok`, exit 0 — the required `Supply chain (cargo-deny)` CI job goes green
  with a yanked dependency in the graph, directly contradicting `deny.toml`'s own stated intent
  ("Deny known-vulnerable and yanked crates").
- **`scripts/test_examples.sh` silently falls back to a published crates.io release when its
  `patch.crates-io` override is incompatible with a local family version.** Reproduced against a
  scratch copy of the workspace with `[workspace.package].version` bumped to `0.4.0` (the exact
  scenario `AGENTS.md` mandates for any pre-1.0 breaking change): every example still commits the
  adopter form `guibiao = "0.3"`, so the patch no longer satisfies that requirement; cargo prints
  `warning: patch ... was not used in the crate graph` and silently resolves the last-published
  `0.3.0` crate instead. Every assertion in the script still passes (the published crate reacts
  identically), so the dogfood gate — whose entire purpose is exercising the in-development tree
  (see its own top-of-file comment) — stays green while silently testing the wrong tree.

## What Changes

- `deny.toml`'s `[advisories]` table gains an explicit `yanked = "deny"`, matching what the section's
  own comment already claims.
- `scripts/test_examples.sh` gains an `assert_patched` check, run immediately after each example's
  `patch.crates-io` args are built and before they are used to run that example: `cargo tree -p
  <crate> "${PATCH[@]}" --depth 0` prints the resolved package's own source in parens for a real
  path/patch dependency (absent for a registry resolution), so its absence is a reliable signal the
  patch was silently dropped. Every example in the script is checked for every family crate it
  patches.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `governance-dogfood`: the "Isolated examples pass repository quality gates" requirement gains a
  scenario asserting that a silently-dropped `patch.crates-io` override fails the gate rather than
  passing against a stale published crate.

## Impact

- Affected code: `deny.toml`, `scripts/test_examples.sh`.
- No public API/DSL change, no baseline format change — this strengthens two CI gates so they
  actually enforce what they already claim to; neither the `Supply chain (cargo-deny)` nor the
  examples-dogfood job currently fails on real Tianheng workspace state (no yanked dependency is
  currently in the graph, and no example's family requirement is currently unsatisfied), so this
  fix has zero effect on the CURRENT green build — only on the failure modes it now actually catches.
- `deny.toml`'s own policy correctness is pure supply-chain tooling, outside any Tianheng-authored
  capability surface (see `crate-source-boundary`'s own note that resolved build-provenance is
  "supply-chain tooling's lane," not a Tianheng feature) — it carries no capability delta.
