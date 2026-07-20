#!/usr/bin/env bash
# Focused failure-direction tests for the repository-only published-family ledger.
set -euo pipefail

WS="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
# shellcheck source=scripts/lib/published_family_coverage.sh
source "$WS/scripts/lib/published_family_coverage.sh"

WORK="$(mktemp -d)"
trap 'rm -rf -- "$WORK"' EXIT

if fulfill_family "hunyi/not-a-family" >"$WORK/unknown.out" 2>&1; then
    echo "::error::unknown published-family claim passed" >&2
    exit 1
fi
grep -q "unknown published boundary family claim: hunyi/not-a-family" "$WORK/unknown.out" \
    || { echo "::error::unknown claim failure did not name the family" >&2; exit 1; }
echo "ok  unknown published-family claim fails loud"

FULFILLED_BOUNDARY_FAMILIES=()
for family in "${PUBLISHED_BOUNDARY_FAMILIES[@]}"; do
    if [ "$family" != "louke/runtime" ]; then
        fulfill_family "$family"
    fi
done
if verify_family_coverage >"$WORK/missing.out" 2>&1; then
    echo "::error::unfulfilled published family passed" >&2
    exit 1
fi
grep -q "published boundary family has no fulfilled reaction owner: louke/runtime" \
    "$WORK/missing.out" \
    || { echo "::error::missing-owner failure did not name the family" >&2; exit 1; }
echo "ok  unfulfilled published family fails loud"
