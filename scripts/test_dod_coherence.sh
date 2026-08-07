#!/usr/bin/env bash
#
# Every state and failure direction of `check_dod_coherence.sh`, each on a throwaway repository.
#
# One of the two `check_*` gates that had no companion matrix. Its subject is a claim `AGENTS.md` makes about itself —
# that its Definition of Done block is the single source for the local gate list and that CI runs a superset —
# so a gate that could not be observed refusing would leave that claim resting on nothing, which is the shape
# this repository keeps closing elsewhere.
#
# Every assertion names the expected exit CODE rather than merely non-zero: this family separates a violation
# (1) from a gate that cannot decide (2), and a matrix blind to that difference cannot defend it — measured,
# not supposed, since exactly that blindness let a 1-into-2 collapse ride green through CI in the sibling
# release-coherence gate.
set -Eeuo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_dod_coherence.sh

dogfood_sequence='bash scripts/test_published_family_coverage.sh
bash scripts/test_example_quality_gate.sh
bash scripts/test_example_suite.sh
bash scripts/test_examples.sh'
reordered_dogfood_sequence='bash scripts/test_example_quality_gate.sh
bash scripts/test_published_family_coverage.sh
bash scripts/test_example_suite.sh
bash scripts/test_examples.sh'

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# A repository whose Definition of Done block and CI workflow agree. Only those documents and the positive
# driver matter, so the fixture carries nothing else — the gate reads text, not a workspace.
new_repo() {
    local name=$1 dod=$2 ci_run=$3
    local dod_dogfood=${4-$dogfood_sequence} ci_dogfood=${5-$dogfood_sequence} repo
    repo=$fixture_root/$name
    mkdir -p "$repo/.github/workflows" "$repo/scripts"
    {
        printf '# AGENTS\n\n## Definition of Done\n\n'
        printf '```bash\n%s\n%s\n```\n' "$dod" "$dod_dogfood"
    } >"$repo/AGENTS.md"
    {
        printf 'name: ci\njobs:\n  dod:\n    steps:\n      - name: gates\n        run: |\n'
        printf '%s\n%s\n' "$ci_run" "$ci_dogfood" | sed 's/^/          /'
    } >"$repo/.github/workflows/ci.yml"
    printf '#!/usr/bin/env bash\nset -euo pipefail\n' >"$repo/scripts/test_examples.sh"
    printf '%s\n' "$repo"
}

expect_pass() {
    local repo=$1 expected=$2 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq 0 ]] \
        || { printf 'expected exit 0, got %d: %s\n' "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected success containing %q, got: %s\n' "$expected" "$output" >&2; exit 1; }
}

expect_fail() {
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

# --- the passing direction first: a gate that only ever refuses is not a working gate ---

agreed=$(new_repo agreed 'cargo fmt --all --check
bash scripts/check_thing.sh' 'cargo fmt --all --check
bash scripts/check_thing.sh')
expect_pass "$agreed" 'is run by CI'

# A clean run must print nothing on stderr. The exit-contract backstop is installed here too, and a trap that
# fires on an ordinary non-zero return would print once per clean run while the code stayed 0 — invisible to
# every check that reads only the code, which is how it went unnoticed in a sibling gate.
clean_stderr=$("$check" "$agreed" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

# CI running MORE than the local list is the declared relationship, not drift: the Definition of Done says CI
# runs a superset of it.
superset=$(new_repo superset 'cargo fmt --all --check' 'cargo fmt --all --check
cargo deny check')
expect_pass "$superset" 'is run by CI'

# The four commands can all remain present while their required source shape drifts. Membership alone cannot
# distinguish either order failure, so each side is perturbed independently.
reordered_local=$(new_repo reordered-local 'cargo fmt --all --check' 'cargo fmt --all --check' \
    "$reordered_dogfood_sequence" "$dogfood_sequence")
expect_fail "$reordered_local" 1 'local Definition of Done lacks the required contiguous example dogfood sequence'

reordered_ci=$(new_repo reordered-ci 'cargo fmt --all --check' 'cargo fmt --all --check' \
    "$dogfood_sequence" "$reordered_dogfood_sequence")
expect_fail "$reordered_ci" 1 'CI lacks the required contiguous example dogfood sequence'

# A full-line comment remains prose and may point a reader at the focused matrix without becoming a nested call.
comment_only=$(new_repo comment-only 'cargo fmt --all --check' 'cargo fmt --all --check')
printf '# Kept separate from test_example_suite.sh.\n' >>"$comment_only/scripts/test_examples.sh"
expect_pass "$comment_only" 'example dogfood orchestration is ordered and non-recursive by authored shape'

# The positive driver's own text is the observable non-recursion perimeter. A direct nested call must fail even
# though the top-level command streams are still perfectly ordered.
nested_matrix=$(new_repo nested-matrix 'cargo fmt --all --check' 'cargo fmt --all --check')
printf 'bash scripts/test_example_suite.sh\n' >>"$nested_matrix/scripts/test_examples.sh"
expect_fail "$nested_matrix" 1 'scripts/test_examples.sh directly names nested matrix test_example_suite.sh'

missing_driver=$(new_repo missing-driver 'cargo fmt --all --check' 'cargo fmt --all --check')
rm -f "$missing_driver/scripts/test_examples.sh"
expect_fail "$missing_driver" 2 'scripts/test_examples.sh is missing or empty'

# --- the violation direction (exit 1) ---

# The class this gate exists for, and the one it was built after: a local command CI does not run.
absent=$(new_repo absent 'cargo fmt --all --check
bash scripts/check_thing.sh' 'cargo fmt --all --check')
expect_fail "$absent" 1 'runs a command CI does not run identically'

# A flag that differs is the drift, not a detail — the exact shape that motivated the gate, where both sides
# ran `cargo doc` and only CI carried `--document-private-items`.
flag_drift=$(new_repo flag-drift 'cargo doc --workspace --no-deps' 'cargo doc --workspace --no-deps --document-private-items')
expect_fail "$flag_drift" 1 'runs a command CI does not run identically'

# --- the cannot-judge directions (exit 2) ---

missing_agents=$(new_repo missing-agents 'cargo fmt --all --check' 'cargo fmt --all --check')
rm -f "$missing_agents/AGENTS.md"
expect_fail "$missing_agents" 2 'AGENTS.md is missing or empty'

missing_ci=$(new_repo missing-ci 'cargo fmt --all --check' 'cargo fmt --all --check')
rm -f "$missing_ci/.github/workflows/ci.yml"
expect_fail "$missing_ci" 2 'ci.yml is missing or empty'

# The block is located by its heading and fence. A shape change must fail loud rather than parse to nothing
# and pass vacuously — the flattering direction for a gate whose subject is a list.
renamed_heading=$(new_repo renamed-heading 'cargo fmt --all --check' 'cargo fmt --all --check')
python3 - "$renamed_heading" <<'PY'
import pathlib, sys
p = pathlib.Path(sys.argv[1]) / "AGENTS.md"
p.write_text(p.read_text().replace("## Definition of Done", "## Pre-flight checks"))
PY
expect_fail "$renamed_heading" 2 'Definition of Done'

# A block that exists but holds no command — a comment-only fence — parses to zero commands, which must also
# refuse rather than report every zero of zero commands present in CI.
empty_block=$(new_repo empty-block '# just a comment' 'cargo fmt --all --check' '' "$dogfood_sequence")
expect_fail "$empty_block" 2 'parsed to zero commands'

# And any unhandled failure at all, which the shared backstop owns: `mktemp` is unwrapped and runs before the
# documents are read.
mktemp_stub=$fixture_root/mktemp-stub
mkdir -p "$mktemp_stub"
printf '#!/usr/bin/env bash\nexit 7\n' >"$mktemp_stub/mktemp"
chmod +x "$mktemp_stub/mktemp"

unhandled_status=0
unhandled_output=$(PATH="$mktemp_stub:$PATH" "$check" "$agreed" 2>&1) || unhandled_status=$?
[[ $unhandled_status -eq 0 || $unhandled_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 0 or 2, never a utility status, got %d: %s\n' "$unhandled_status" "$unhandled_output" >&2; exit 1; }

# --- read-only, like every gate in the family ---

# On a fixture this gate has NOT already judged. Capturing `before` from a repository the gate had run over
# several times was blind by construction: a gate that writes the same file on every run leaves that file in
# `before` too, so the comparison held. Measured, not reasoned — a stray write injected into a sibling gate
# passed its read-only direction unnoticed until the fixture was made fresh.
untouched=$(new_repo untouched 'cargo fmt --all --check' 'cargo fmt --all --check')
before=$(cd "$untouched" && find . -type f | sort && cat AGENTS.md .github/workflows/ci.yml)
"$check" "$untouched" >/dev/null
after=$(cd "$untouched" && find . -type f | sort && cat AGENTS.md .github/workflows/ci.yml)
[[ $before == "$after" ]] \
    || { printf 'dod coherence check mutated the repository it judged\n' >&2; exit 1; }

printf 'ok dod coherence state and failure matrix\n'
