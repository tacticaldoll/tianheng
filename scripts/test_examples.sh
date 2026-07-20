#!/usr/bin/env bash
# Run every example in examples/ in isolation and assert its reaction.
#
# Each example is its own workspace — its *deliberate* faults (a bad import, an API leak, a rogue
# runtime origin) must never be swept by Tianheng's workspace-wide gates. Each commits the
# adopter's real dependency form (`guibiao = "0.1"`), so we resolve the family to LOCAL source via
# `--config patch.crates-io.<crate>.path=...` — the same idiom the `packaged-selftest` CI job uses:
# the committed Cargo.toml stays copy-paste-honest while CI exercises the in-development tree.
#
# Assertions bind the STABLE contract — exit codes and the `--format json`/`sarif` fields — never
# the ANSI human render, which is presentation and free to change (see the render-polish track).
set -euo pipefail

WS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

# Emit null-delimited `--config patch...` args for the named family crates.
patch() {
    local out=() c
    for c in "$@"; do
        out+=(--config "patch.crates-io.$c.path=\"$WS/crates/$c\"")
    done
    printf '%s\0' "${out[@]}"
}

expect() { # expect <got> <want> <label>
    if [ "$1" -ne "$2" ]; then
        echo "::error::$3: expected exit $2, got $1"
        exit 1
    fi
    echo "ok  $3 (exit $1)"
}

# ---------------------------------------------------------------- guibiao-standalone
cd "$WS/examples/guibiao-standalone"
mapfile -d '' PATCH < <(patch guibiao xuanji xingbiao)
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "guibiao-standalone demo reacts"

# ---------------------------------------------------------------- hunyi-standalone
cd "$WS/examples/hunyi-standalone"
mapfile -d '' PATCH < <(patch hunyi xuanji xingbiao)
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "hunyi-standalone demo reacts"

# ---------------------------------------------------------------- unsafe-confinement
# 渾儀's unsafe-confinement — the one capability the family cannot self-demo (every family crate is
# `#![forbid(unsafe_code)]`), so it needs a crate with real, confined `unsafe`.
cd "$WS/examples/unsafe-confinement"
mapfile -d '' PATCH < <(patch hunyi xuanji xingbiao)
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "unsafe-confinement demo reacts"

# ---------------------------------------------------------------- composed
cd "$WS/examples/composed"
mapfile -d '' PATCH < <(patch xuanji xingbiao guibiao hunyi louke tianheng)
cargo test "${PATCH[@]}"

# check-mode — presentation ⊥ verdict: the exit code is identical across formats.
for fmt in "" "--format json" "--format sarif"; do
    got=0
    # shellcheck disable=SC2086
    cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml $fmt \
        >/tmp/composed_out.txt 2>&1 || got=$?
    expect "$got" 1 "composed check-mode reacts (${fmt:-default})"
done

# Machine contracts (bound to the stable projection, not the human render).
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format json \
    >/tmp/composed.json 2>/dev/null || true
grep -q '"exit_code": 1' /tmp/composed.json || { echo "::error::composed json missing exit_code:1"; exit 1; }
echo "ok  composed json contract (exit_code: 1)"

# The funnel is real, not one dimension masquerading: `run()` must aggregate BOTH the static
# (圭表, a `module` boundary) fault AND the semantic (渾儀) fault. Asserting the exit code alone
# would still pass if one dimension silently stopped contributing — the static fault would keep
# exit == 1 on its own — a false negative in the composition the example exists to demonstrate.
grep -q '"kind": "module"' /tmp/composed.json \
    || { echo "::error::composed json has no static (module) violation — the funnel dropped its 圭表 contribution"; exit 1; }
grep -q '"kind": "semantic"' /tmp/composed.json \
    || { echo "::error::composed json has no semantic violation — the funnel dropped its 渾儀 contribution"; exit 1; }
echo "ok  composed funnel aggregates both dimensions (圭表 module + 渾儀 semantic)"

cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format sarif \
    >/tmp/composed.sarif 2>/dev/null || true
grep -q '"version": "2.1.0"' /tmp/composed.sarif || { echo "::error::composed sarif missing version 2.1.0"; exit 1; }
echo "ok  composed sarif contract (version 2.1.0)"

# Adoption path — drive the real shell, not only Baseline's library API. Recording the deliberate
# static + semantic faults is observation (exit 0); gating against that snapshot keeps the known
# drift green while preserving the violations as baselined machine output. The standalone example's
# reaction tests separately prove an identity absent from the baseline still exits 1.
BASELINE_WORK="$(mktemp -d)"
BASELINE_PATH="$BASELINE_WORK/composed-baseline.json"
trap 'rm -rf -- "$BASELINE_WORK"' EXIT
got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml \
    --write-baseline "$BASELINE_PATH" >/tmp/composed_baseline_write.txt 2>&1 || got=$?
expect "$got" 0 "composed baseline write records existing drift"
grep -q '"version": 2' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing version 2"; exit 1; }
grep -q '"finding_key"' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing structured finding keys"; exit 1; }
grep -q '"violations"' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing violations"; exit 1; }

got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml \
    --baseline "$BASELINE_PATH" --format json >/tmp/composed_baselined.json 2>/dev/null || got=$?
expect "$got" 0 "composed baseline gates only new drift"
grep -q '"baselined": true' /tmp/composed_baselined.json \
    || { echo "::error::composed gate did not project known drift as baselined"; exit 1; }
rm -rf -- "$BASELINE_WORK"
trap - EXIT

# run-mode — the runtime dimension reacts as an event (exit 0, never a crash) and emits the reaction.
got=0
cargo run --quiet --bin runtime_demo "${PATCH[@]}" >/tmp/composed_runtime.txt 2>&1 || got=$?
expect "$got" 0 "composed run-mode is event-only (no crash)"
grep -q 'runtime reaction' /tmp/composed_runtime.txt || { echo "::error::runtime_demo did not emit the reaction"; exit 1; }
echo "ok  composed run-mode emitted the fail-closed reaction"

# ---------------------------------------------------------------- sans-io-pure
# The 天衡 shell's `sans_io_pure` profile — folds a 圭表 clock boundary and a subtree-scoped 渾儀
# async boundary into one declaration; `run` projects both into one exit code.
cd "$WS/examples/sans-io-pure"
mapfile -d '' PATCH < <(patch xuanji xingbiao guibiao hunyi louke tianheng)
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml >/dev/null 2>&1 || got=$?
expect "$got" 1 "sans-io-pure check reacts (both axes folded)"

# The profile is real composition, not one axis: `run()` must aggregate BOTH the 圭表 clock (a
# `module` boundary) fault AND the 渾儀 async fault — asserting the exit code alone would pass if
# one axis silently stopped contributing.
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format json \
    >/tmp/sans_io.json 2>/dev/null || true
grep -q '"kind": "module"' /tmp/sans_io.json \
    || { echo "::error::sans-io-pure json has no static (module) clock violation"; exit 1; }
grep -q '"kind": "semantic"' /tmp/sans_io.json \
    || { echo "::error::sans-io-pure json has no semantic async violation — the subtree axis dropped"; exit 1; }
echo "ok  sans-io-pure folds both axes (圭表 clock + 渾儀 async subtree)"

echo "all examples reacted as declared."
