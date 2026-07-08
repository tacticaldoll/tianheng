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

# run-mode — the runtime dimension reacts as an event (exit 0, never a crash) and emits the reaction.
got=0
cargo run --quiet --bin runtime_demo "${PATCH[@]}" >/tmp/composed_runtime.txt 2>&1 || got=$?
expect "$got" 0 "composed run-mode is event-only (no crash)"
grep -q 'runtime reaction' /tmp/composed_runtime.txt || { echo "::error::runtime_demo did not emit the reaction"; exit 1; }
echo "ok  composed run-mode emitted the fail-closed reaction"

echo "all examples reacted as declared."
