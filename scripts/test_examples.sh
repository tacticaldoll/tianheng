#!/usr/bin/env bash
# Run every example in examples/ in isolation and assert its reaction.
#
# Each example is its own workspace — its *deliberate* faults (a bad import, an API leak, a rogue
# runtime origin) must never be swept by Tianheng's workspace-wide gates. Each commits the
# adopter's real dependency form (`guibiao = "0.2"`), so we resolve the family to LOCAL source via
# `--config patch.crates-io.<crate>.path=...` — the same idiom the `packaged-selftest` CI job uses:
# the committed Cargo.toml stays copy-paste-honest while CI exercises the in-development tree.
#
# Assertions bind the STABLE contract — exit codes and the `--format json`/`sarif` fields — never
# the ANSI human render, which is presentation and free to change (see the render-polish track).
set -euo pipefail

WS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/lib/published_family_coverage.sh
source "$WS/scripts/lib/published_family_coverage.sh"
# shellcheck source=scripts/lib/example_quality.sh
source "$WS/scripts/lib/example_quality.sh"
# shellcheck source=scripts/lib/example_suite.sh
source "$WS/scripts/lib/example_suite.sh"

configure_example_suite "$WS/examples"
init_example_artifacts

# Prove that both ledger failure directions bite before trusting its positive result below.
bash "$WS/scripts/test_published_family_coverage.sh"
bash "$WS/scripts/test_example_suite.sh"
# Prove a real warning stops an example before its reaction could be accepted.
bash "$WS/scripts/test_example_quality_gate.sh"

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
quality_gates "guibiao-standalone" "${PATCH[@]}"
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "guibiao-standalone demo reacts"
fulfill_family guibiao/module
fulfill_example guibiao-standalone

# ---------------------------------------------------------------- hunyi-standalone
cd "$WS/examples/hunyi-standalone"
mapfile -d '' PATCH < <(patch hunyi xuanji xingbiao)
quality_gates "hunyi-standalone" "${PATCH[@]}"
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "hunyi-standalone demo reacts"
fulfill_family hunyi/signature
fulfill_family hunyi/visibility
fulfill_example hunyi-standalone

# ---------------------------------------------------------------- unsafe-confinement
# 渾儀's unsafe-confinement — the one capability the family cannot self-demo (every family crate is
# `#![forbid(unsafe_code)]`), so it needs a crate with real, confined `unsafe`.
cd "$WS/examples/unsafe-confinement"
mapfile -d '' PATCH < <(patch hunyi xuanji xingbiao)
quality_gates "unsafe-confinement" "${PATCH[@]}"
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin demo "${PATCH[@]}" >/dev/null 2>&1 || got=$?
expect "$got" 1 "unsafe-confinement demo reacts"
fulfill_family hunyi/unsafe
fulfill_example unsafe-confinement

# ---------------------------------------------------------------- capability-catalog
# Breadth-only contract coverage for published families that have no honest home in the
# focused teaching examples. Bind stable structured identity, never the human finding sentence.
cd "$WS/examples/capability-catalog"
mapfile -d '' PATCH < <(patch xuanji xingbiao guibiao hunyi louke tianheng)
quality_gates "capability-catalog" "${PATCH[@]}"
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format json \
    >"$EXAMPLE_ARTIFACT_ROOT/capability_catalog.json" 2>/dev/null || got=$?
expect "$got" 1 "capability-catalog shell reacts"
for identity in \
    '"rule": "restrict dependency sources to"' \
    '"type": "tianheng.fact/guibiao/external-importer"' \
    '"type": "tianheng.fact/hunyi/trait-impl-site"' \
    '"type": "tianheng.fact/hunyi/forbidden-marker-acquisition"' \
    '"type": "tianheng.fact/hunyi/dyn-trait-exposure"' \
    '"type": "tianheng.fact/hunyi/impl-trait-exposure"' \
    '"type": "tianheng.fact/hunyi/async-exposure"'
do
    grep -q "$identity" "$EXAMPLE_ARTIFACT_ROOT/capability_catalog.json" \
        || { echo "::error::capability catalog missing structured identity $identity"; exit 1; }
done
echo "ok  capability-catalog carries every assigned structured family identity"
fulfill_family guibiao/crate
fulfill_family guibiao/module
fulfill_family hunyi/trait-impl
fulfill_family hunyi/forbidden-marker
fulfill_family hunyi/dyn-trait
fulfill_family hunyi/impl-trait
fulfill_family hunyi/async-exposure
fulfill_family tianheng/no-existential-leak
fulfill_example capability-catalog

# ---------------------------------------------------------------- composed
cd "$WS/examples/composed"
mapfile -d '' PATCH < <(patch xuanji xingbiao guibiao hunyi louke tianheng)
quality_gates "composed" "${PATCH[@]}"
cargo test "${PATCH[@]}"

# check-mode — presentation ⊥ verdict: the exit code is identical across formats.
for fmt in "" "--format json" "--format sarif"; do
    got=0
    # shellcheck disable=SC2086
    cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml $fmt \
        >"$EXAMPLE_ARTIFACT_ROOT/composed_out.txt" 2>&1 || got=$?
    expect "$got" 1 "composed check-mode reacts (${fmt:-default})"
done

# Machine contracts (bound to the stable projection, not the human render).
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format json \
    >"$EXAMPLE_ARTIFACT_ROOT/composed.json" 2>/dev/null || true
grep -q '"exit_code": 1' "$EXAMPLE_ARTIFACT_ROOT/composed.json" || { echo "::error::composed json missing exit_code:1"; exit 1; }
echo "ok  composed json contract (exit_code: 1)"

# The funnel is real, not one dimension masquerading: `run()` must aggregate BOTH the static
# (圭表, a `module` boundary) fault AND the semantic (渾儀) fault. Asserting the exit code alone
# would still pass if one dimension silently stopped contributing — the static fault would keep
# exit == 1 on its own — a false negative in the composition the example exists to demonstrate.
grep -q '"kind": "module"' "$EXAMPLE_ARTIFACT_ROOT/composed.json" \
    || { echo "::error::composed json has no static (module) violation — the funnel dropped its 圭表 contribution"; exit 1; }
grep -q '"kind": "semantic"' "$EXAMPLE_ARTIFACT_ROOT/composed.json" \
    || { echo "::error::composed json has no semantic violation — the funnel dropped its 渾儀 contribution"; exit 1; }
echo "ok  composed funnel aggregates both dimensions (圭表 module + 渾儀 semantic)"

cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format sarif \
    >"$EXAMPLE_ARTIFACT_ROOT/composed.sarif" 2>/dev/null || true
grep -q '"version": "2.1.0"' "$EXAMPLE_ARTIFACT_ROOT/composed.sarif" || { echo "::error::composed sarif missing version 2.1.0"; exit 1; }
echo "ok  composed sarif contract (version 2.1.0)"

# Adoption path — drive the real shell, not only Baseline's library API. Recording the deliberate
# static + semantic faults is observation (exit 0); gating against that snapshot keeps the known
# drift green while preserving the violations as baselined machine output. The standalone example's
# reaction tests separately prove an identity absent from the baseline still exits 1.
BASELINE_WORK="$EXAMPLE_ARTIFACT_ROOT/baseline"
mkdir -p "$BASELINE_WORK"
BASELINE_PATH="$BASELINE_WORK/composed-baseline.json"
got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml \
    --write-baseline "$BASELINE_PATH" >"$EXAMPLE_ARTIFACT_ROOT/composed_baseline_write.txt" 2>&1 || got=$?
expect "$got" 0 "composed baseline write records existing drift"
grep -q '"format": "tianheng.baseline/structured-facts"' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing semantic format"; exit 1; }
grep -q '"fact"' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing structured facts"; exit 1; }
grep -q '"violations"' "$BASELINE_PATH" \
    || { echo "::error::composed baseline missing violations"; exit 1; }

got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml \
    --baseline "$BASELINE_PATH" --format json >"$EXAMPLE_ARTIFACT_ROOT/composed_baselined.json" 2>/dev/null || got=$?
expect "$got" 0 "composed baseline gates only new drift"
grep -q '"baselined": true' "$EXAMPLE_ARTIFACT_ROOT/composed_baselined.json" \
    || { echo "::error::composed gate did not project known drift as baselined"; exit 1; }

# run-mode — the runtime dimension reacts as an event (exit 0, never a crash) and emits the reaction.
got=0
cargo run --quiet --bin runtime_demo "${PATCH[@]}" >"$EXAMPLE_ARTIFACT_ROOT/composed_runtime.txt" 2>&1 || got=$?
expect "$got" 0 "composed run-mode is event-only (no crash)"
grep -q 'runtime reaction' "$EXAMPLE_ARTIFACT_ROOT/composed_runtime.txt" || { echo "::error::runtime_demo did not emit the reaction"; exit 1; }
echo "ok  composed run-mode emitted the fail-closed reaction"
fulfill_family louke/runtime
fulfill_example composed

# ---------------------------------------------------------------- sans-io-pure
# The 天衡 shell's `sans_io_pure` profile — folds a 圭表 clock boundary and a subtree-scoped 渾儀
# async boundary into one declaration; `run` projects both into one exit code.
cd "$WS/examples/sans-io-pure"
mapfile -d '' PATCH < <(patch xuanji xingbiao guibiao hunyi louke tianheng)
quality_gates "sans-io-pure" "${PATCH[@]}"
cargo test "${PATCH[@]}"
got=0
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml >/dev/null 2>&1 || got=$?
expect "$got" 1 "sans-io-pure check reacts (both axes folded)"

# The profile is real composition, not one axis: `run()` must aggregate BOTH the 圭表 clock (a
# `module` boundary) fault AND the 渾儀 async fault — asserting the exit code alone would pass if
# one axis silently stopped contributing.
cargo run --quiet --bin check "${PATCH[@]}" -- check --manifest-path Cargo.toml --format json \
    >"$EXAMPLE_ARTIFACT_ROOT/sans_io.json" 2>/dev/null || true
grep -q '"kind": "module"' "$EXAMPLE_ARTIFACT_ROOT/sans_io.json" \
    || { echo "::error::sans-io-pure json has no static (module) clock violation"; exit 1; }
grep -q '"kind": "semantic"' "$EXAMPLE_ARTIFACT_ROOT/sans_io.json" \
    || { echo "::error::sans-io-pure json has no semantic async violation — the subtree axis dropped"; exit 1; }
echo "ok  sans-io-pure folds both axes (圭表 clock + 渾儀 async subtree)"
fulfill_family hunyi/async-exposure
fulfill_family tianheng/sans-io-pure
fulfill_example sans-io-pure

verify_example_coverage
verify_family_coverage
echo "all examples reacted as declared."
