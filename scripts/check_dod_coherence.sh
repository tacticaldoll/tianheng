#!/usr/bin/env bash
# The local Definition of Done must be a SUBSET of what CI runs, and example dogfood keeps one authored order.
#
# `AGENTS.md` declares its Definition of Done block "the single source for the local pre-flight gate
# list ... CI runs a superset of it". That is a checkable claim, and it drifted: both CI `cargo doc`
# invocations gained `--document-private-items` while the local list kept the flagless form, so the
# gate a contributor actually runs could not see the very class (a broken intra-doc link among
# crate-private items) that the CI change existed to catch. A contributor whose local gates pass then
# learns about it from CI — which is exactly what the local list exists to prevent.
#
# So the agreement is a reaction rather than a promise: every command in the block must appear
# verbatim in `.github/workflows/ci.yml`. Verbatim, not "equivalent" — a differing flag IS the drift.
# The same parsed surfaces hold the focused-example shape: its failure matrices form one contiguous sequence
# before the positive driver in both places, and that driver's non-comment source lines directly name none of
# them. Membership alone cannot see a reorder or nested rerun while every command remains present.
#
# The comparison is whole-line, never substring: a substring match cannot see a MISSING TRAILING FLAG,
# which is precisely the drift above (`… --all-features` is a substring of `… --all-features
# --document-private-items`). A guard blind to the case that motivated it is not a guard.
# Exit 0 coherent, 1 incoherent, 2 cannot judge — the family's own Core Contract, stated here for the same
# reason: this gate already refused with 2 in two places without ever declaring what its codes mean.
set -Eeuo pipefail
# The family's exit contract as a backstop — see `scripts/lib/exit_contract.sh` for what it catches, why it
# is a trap rather than per-command handling, and the measurements behind both.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
exit_contract_backstop 'dod coherence'

# The repository to judge, so the failure matrix can build throwaway fixtures rather than being able to test
# only this checkout — the same argument the other four gates take, for the same reason: a gate that cannot be
# pointed at a fixture cannot have its refusals proven.
cd "${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"

agents="AGENTS.md"
ci=".github/workflows/ci.yml"
example_driver="scripts/test_examples.sh"
focused_example_matrices=(
  "scripts/test_published_family_coverage.sh"
  "scripts/test_example_quality_gate.sh"
  "scripts/test_example_suite.sh"
)
[ -s "$agents" ] || { echo "error: $agents is missing or empty"; exit 2; }
[ -s "$ci" ] || { echo "error: $ci is missing or empty"; exit 2; }
[ -s "$example_driver" ] || { echo "error: $example_driver is missing or empty"; exit 2; }

# Commands the local list runs that CI runs by another mechanism. Each needs a reason, and the
# reason has to be about the MECHANISM — never about a flag difference, which is the drift itself.
is_exempt() {
  case "$1" in
    # CI runs cargo-deny through EmbarkStudios/cargo-deny-action@v2 (its own job), not this CLI form.
    "cargo deny check") return 0 ;;
    *) return 1 ;;
  esac
}

# The fenced ```bash block that immediately follows the "## Definition of Done" heading.
block="$(awk '
  /^## Definition of Done$/ { in_section = 1; next }
  in_section && /^```bash$/ { in_block = 1; next }
  in_block && /^```$/       { exit }
  in_block                  { print }
' "$agents")"

[ -n "$block" ] || {
  echo "error: no fenced bash block found under '## Definition of Done' in $agents — the gate list"
  echo "       moved or changed shape, so this reaction can no longer read it (never a silent pass)"
  exit 2
}

# CI's own command lines, normalized to bare commands: leading indentation dropped, a YAML `- `
# sequence dash and a `run: ` key dropped (a one-line `run:` step is still a command), trailing
# whitespace dropped. Compared for EQUALITY against each local command below.
ci_commands="$(sed -e 's/^[[:space:]]*//' -e 's/^- //' -e 's/^run:[[:space:]]*//' -e 's/[[:space:]]*$//' "$ci")"

missing=0
found=0
dod_commands=""
while IFS= read -r line; do
  # Drop trailing inline comments, then trim. A pure comment or blank line carries no command.
  command="${line%%#*}"
  command="$(printf '%s' "$command" | sed -e 's/[[:space:]]*$//' -e 's/^[[:space:]]*//')"
  [ -n "$command" ] || continue
  found=$((found + 1))
  dod_commands+="${dod_commands:+$'\n'}$command"
  if is_exempt "$command"; then
    continue
  fi
  if ! grep -qxF -- "$command" <<< "$ci_commands"; then
    echo "error: the local Definition of Done runs a command CI does not run identically:"
    echo "         $command"
    echo "       $agents calls that block the single source for the local gate list and states CI"
    echo "       runs a superset. Either add it to $ci, or make the local line match CI exactly —"
    echo "       a flag that differs is the drift, not a detail."
    missing=1
  fi
done <<< "$block"

[ "$found" -gt 0 ] || {
  echo "error: the Definition of Done block parsed to zero commands — a shape change would make this"
  echo "       reaction vacuously pass, so it fails loud instead"
  exit 2
}

example_dogfood_sequence() {
  local matrix
  for matrix in "${focused_example_matrices[@]}"; do
    printf 'bash %s\n' "$matrix"
  done
  printf 'bash %s\n' "$example_driver"
}

contains_contiguous_sequence() {
  local commands=$1 sequence=$2
  awk -v sequence="$sequence" '
    BEGIN {
      count = split(sequence, expected, "\n")
      next_expected = 1
    }
    /^[[:space:]]*($|#)/ { next }
    {
      if ($0 == expected[next_expected]) {
        next_expected++
        if (next_expected > count) {
          found = 1
          exit
        }
      } else if ($0 == expected[1]) {
        next_expected = 2
      } else {
        next_expected = 1
      }
    }
    END { exit(found ? 0 : 1) }
  ' <<<"$commands"
}

dogfood_sequence="$(example_dogfood_sequence)"
if ! contains_contiguous_sequence "$dod_commands" "$dogfood_sequence"; then
  echo "error: local Definition of Done lacks the required contiguous example dogfood sequence"
  missing=1
fi
if ! contains_contiguous_sequence "$ci_commands" "$dogfood_sequence"; then
  echo "error: CI lacks the required contiguous example dogfood sequence"
  missing=1
fi

# This is deliberately an authored-form reaction, not a shell call-graph claim. Full-line shell comments are
# prose; every other source line may not directly name a focused matrix basename.
driver_non_comment="$(sed '/^[[:space:]]*#/d' "$example_driver")"
for matrix in "${focused_example_matrices[@]}"; do
  matrix_name=${matrix##*/}
  if grep -Fq -- "$matrix_name" <<<"$driver_non_comment"; then
    echo "error: $example_driver directly names nested matrix $matrix_name"
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "ok: every local Definition of Done command ($found parsed) is run by CI; example dogfood orchestration is ordered and non-recursive by authored shape"
