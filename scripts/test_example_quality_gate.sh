#!/usr/bin/env bash
# Prove that a real isolated-workspace warning stops the gate before reaction acceptance.
set -euo pipefail

WS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/lib/example_quality.sh
source "$WS/scripts/lib/example_quality.sh"

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT
mkdir -p "$WORK/src"

printf '%s\n' \
    '[package]' \
    'name = "quality_warning"' \
    'version = "0.0.0"' \
    'edition = "2021"' \
    'publish = false' \
    '' \
    '[workspace]' \
    >"$WORK/Cargo.toml"
printf '%s\n' \
    'pub fn has_one(values: &[u8]) -> bool {' \
    '    values.iter().any(|value| *value == 1)' \
    '}' \
    >"$WORK/src/lib.rs"

SENTINEL="$WORK/reaction-accepted"
set +e
(
    set -e
    cd "$WORK"
    quality_gates "warning fixture"
    touch "$SENTINEL"
) >"$WORK/quality.out" 2>&1
got=$?
set -e
if [ "$got" -eq 0 ]; then
    echo "::error::a Clippy warning did not stop isolated-example acceptance" >&2
    exit 1
fi
if [ -e "$SENTINEL" ]; then
    echo "::error::reaction acceptance ran after the quality gate failed" >&2
    exit 1
fi
grep -q "manual_contains" "$WORK/quality.out" \
    || { echo "::error::fixture did not fail on its declared Clippy warning" >&2; exit 1; }
echo "ok  isolated-example warning fails before reaction acceptance"
