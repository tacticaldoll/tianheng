## 1. deny.toml yanked policy

- [x] 1.1 Reproduced against a real yanked crate (`openssl-probe = "0.1.3"`, pinned via
      `cargo update --precise`): `cargo deny check` warns but exits 0 (`advisories ok`) with the
      unset `yanked` field.
- [x] 1.2 Added `yanked = "deny"` to `deny.toml`'s `[advisories]` table, with a comment recording
      why the unset default is insufficient.
- [x] 1.3 Non-vacuous verification: re-ran the same repro with `yanked = "deny"` present — now
      `error[yanked]`, `advisories FAILED`, exit 1. Reverted to confirm it warns-and-passes again,
      restored.
- [x] 1.4 Confirmed the real workspace's own `cargo deny check` still passes clean (no yanked
      dependency currently in the graph) — this closes a latent gap without touching the present
      green build.

## 2. test_examples.sh patch-drop detection

- [x] 2.1 Reproduced against a scratch copy of the workspace with `[workspace.package].version`
      bumped to `0.4.0` (`AGENTS.md`'s own pre-1.0 breaking-change rule): every example's committed
      `= "0.3"` family requirement no longer accepts the patch; `cargo tree` shows
      `warning: patch ... was not used in the crate graph` and resolves the crate from crates.io
      instead (no local path in the tree's root line).
- [x] 2.2 Added `assert_patched` to `scripts/test_examples.sh`: `cargo tree -p <crate>
      "${PATCH[@]}" --depth 0`, checked against the expected `<crate> v<ver>
      ($WS/crates/<crate>)` pattern.
- [x] 2.3 Wired `assert_patched` into all six examples (guibiao-standalone, hunyi-standalone,
      unsafe-confinement, capability-catalog, composed, sans-io-pure), immediately after each
      example's own `PATCH` array is built.
- [x] 2.4 Non-vacuous verification: ran the full script against the real (un-bumped) workspace
      (passes, `assert_patched` succeeds for every example) and against the scratch, version-bumped
      copy (fails loud on the very first example, naming the exact crate and the fallback version).
- [x] 2.5 Added a `governance-dogfood` spec delta scenario for this behavior.

## 3. Documentation

- [x] 3.1 Added a CHANGELOG `[Unreleased] ### Fixed` entry describing both fixes. No **BREAKING**
      marker — CI-gate strengthening only, no product surface change.

## 4. Definition of Done

- [x] 4.1 Run the full local gate list from `AGENTS.md` (build, three clippy passes, fmt, full test
      suite, both doc passes, `cargo deny check`, release-coherence scripts, `test_examples.sh`).
- [x] 4.2 Adversarial apply-stage review: confirm the declared reaction still bites, not a taste
      call.

## 5. Adversarial review follow-up

- [x] 5.1 Review found `assert_patched`'s `resolved="$(cargo tree ... | tail -1)"` runs under the
      script's own `set -euo pipefail` with no failure guard: if `cargo tree -p <crate>` itself
      exits non-zero (e.g. an ambiguous `-p` match, or the crate not resolving at all), `pipefail`
      propagates that exit code through the assignment and `errexit` kills the whole script right
      there — the function's own diagnostic `case` branch, the entire point of `assert_patched`,
      is never reached, and cargo's own error was discarded (`2>/dev/null`). Reproduced directly: a
      nonexistent crate name silently exited 101 with zero output under the original code.
- [x] 5.2 Fixed: capture merged stdout+stderr (`2>&1`) with `|| true` guarding the assignment so a
      `cargo tree` failure can't abort the script, then `grep` for the crate's own line (rather than
      blindly trusting the last line, tolerating a warning landing after the tree line in the merged
      stream) so a genuine failure's real message reaches the reported diagnostic.
- [x] 5.3 Non-vacuous verification: reproduced the exact silent-abort (nonexistent crate name,
      original code: exit 101, zero output) and confirmed the fix instead reports a real diagnostic
      and exits 1 cleanly. Re-ran the full `test_examples.sh` against the real workspace — still
      green.
