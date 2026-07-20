#!/usr/bin/env bash
# Focused failure directions for example ownership and invocation-local artifact cleanup.
set -euo pipefail

WS=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
# shellcheck source=scripts/lib/example_suite.sh
source "$WS/scripts/lib/example_suite.sh"

WORK=$(mktemp -d)
trap 'rm -rf -- "$WORK"' EXIT
mkdir -p "$WORK/examples/alpha" "$WORK/examples/beta"
touch "$WORK/examples/alpha/Cargo.toml" "$WORK/examples/beta/Cargo.toml"
configure_example_suite "$WORK/examples"

if fulfill_example gamma >"$WORK/unknown.out" 2>&1; then
    echo "::error::unknown example claim passed" >&2
    exit 1
fi
grep -q 'unknown repository example claim: gamma' "$WORK/unknown.out" \
    || { echo "::error::unknown example failure did not name gamma" >&2; exit 1; }
echo "ok  unknown repository example claim fails loud"

fulfill_example alpha
if verify_example_coverage >"$WORK/missing.out" 2>&1; then
    echo "::error::unfulfilled repository example passed" >&2
    exit 1
fi
grep -q 'repository example has no fulfilled reaction owner: beta' "$WORK/missing.out" \
    || { echo "::error::missing owner failure did not name beta" >&2; exit 1; }
echo "ok  unfulfilled repository example fails loud"

fulfill_example beta
verify_example_coverage >/dev/null

run_waiting_child() {
    local record=$1
    bash -c '
        set -euo pipefail
        source "$1"
        init_example_artifacts
        printf "%s\n" "$EXAMPLE_ARTIFACT_ROOT" >"$2"
        while [[ ! -e $3 ]]; do
            sleep 0.01
        done
        exit 9
    ' _ "$WS/scripts/lib/example_suite.sh" "$record" "$WORK/release-children"
}

run_waiting_child "$WORK/first-root" &
first_pid=$!
run_waiting_child "$WORK/second-root" &
second_pid=$!
while [[ ! -s $WORK/first-root || ! -s $WORK/second-root ]]; do
    sleep 0.01
done
first_root=$(<"$WORK/first-root")
second_root=$(<"$WORK/second-root")
[[ $first_root != "$second_root" ]] \
    || { echo "::error::two invocations shared one artifact root" >&2; exit 1; }
[[ -d $first_root && -d $second_root ]] \
    || { echo "::error::concurrent artifact roots did not overlap" >&2; exit 1; }
touch "$WORK/release-children"
first_status=0
second_status=0
wait "$first_pid" || first_status=$?
wait "$second_pid" || second_status=$?
[[ $first_status -eq 9 && $second_status -eq 9 ]] \
    || { echo "::error::artifact children exited $first_status/$second_status, expected 9/9" >&2; exit 1; }
[[ ! -e $first_root && ! -e $second_root ]] \
    || { echo "::error::failed invocation left its artifact root behind" >&2; exit 1; }
echo "ok  concurrent example artifact roots are unique and clean after failure"
