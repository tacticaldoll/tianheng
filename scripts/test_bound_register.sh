#!/usr/bin/env bash
#
# Every state and failure direction of `check_bound_register.sh`, each on a throwaway repository.
#
# A gate over a coverage claim is the easiest kind to get wrong in the flattering direction: it can refuse
# nothing and still read as protection, because its subject is absence. So each refusal is proven against a
# fixture built to trip exactly one condition, the passing directions are proven too, and every assertion
# names the expected exit CODE rather than merely non-zero — the family's contract separates a violation (1)
# from a gate that cannot decide (2), and collapsing them would report a misconfiguration as a clean refusal.
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_bound_register.sh

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# A repository with one capability spec and one crate, both tracked — the gate judges tracked content, so an
# untracked fixture would be invisible to it and every case would pass vacuously.
new_repo() {
    local name=$1 spec=$2 rust=${3:-} repo
    repo=$fixture_root/$name
    mkdir -p "$repo/openspec/specs/probe-capability" "$repo/crates/probe/src"
    git init -q "$repo"
    git -C "$repo" config user.name 'Bound Register Test'
    git -C "$repo" config user.email 'bound-register@example.invalid'
    printf '%s' "$spec" >"$repo/openspec/specs/probe-capability/spec.md"
    printf '%s' "${rust:-$DEFAULT_RUST}" >"$repo/crates/probe/src/lib.rs"
    git -C "$repo" add -A
    git -C "$repo" commit -qm 'probe fixture'
    printf '%s\n' "$repo"
}

DEFAULT_RUST='pub fn a_probe_bound_is_pinned() {}
'

# A spec whose single declared bound carries `citation`. `extra` appends further content, used by the prose
# and reference cases.
spec_with() {
    local citation=$1 extra=${2:-}
    printf '%s\n' \
        '# probe-capability Specification' \
        '' \
        '## Purpose' \
        '' \
        'Probe capability.' \
        '## Requirements' \
        '### Requirement: The probe observes something' \
        '' \
        'The probe SHALL observe the shape it claims.' \
        '' \
        '#### Scenario: A probed shape is a stated bound' \
        '- **WHEN** the probe meets the shape' \
        '- **THEN** it does not claim to observe it' \
        "$citation" \
        '' \
        '#### Scenario: The probe reacts on a real shape' \
        '- **WHEN** the probe meets a real shape' \
        '- **THEN** it reacts' \
        "$extra"
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

# --- the passing directions, first: a gate that only ever refuses is not a working gate ---

pinned=$(new_repo pinned "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
expect_pass "$pinned" 'bound register ok (1 declared bounds across 1 spec files)'

tracked=$(new_repo tracked "$(spec_with '- **UNPINNED** BACKLOG.md "probe debt"')")
expect_pass "$tracked" 'bound register ok (1 declared bounds'

# --- the citation directions ---

no_citation=$(new_repo no-citation "$(spec_with '')")
expect_fail "$no_citation" 1 'carries neither PINNED-BY nor UNPINNED'

both=$(new_repo both "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`
- **UNPINNED** BACKLOG.md "probe debt"')")
expect_fail "$both" 1 'carries both PINNED-BY and UNPINNED'

untracked_debt=$(new_repo untracked-debt "$(spec_with '- **UNPINNED**')")
expect_fail "$untracked_debt" 1 'is UNPINNED with no tracker'

# --- the pinning-test resolution directions ---

absent=$(new_repo absent "$(spec_with '- **PINNED-BY** `a_test_that_was_renamed_away`')")
expect_fail "$absent" 1 'which no function under crates/ defines'

twice=$(new_repo twice "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    'pub fn a_probe_bound_is_pinned() {}

mod second {
    pub fn a_probe_bound_is_pinned() {}
}
')
expect_fail "$twice" 1 'the citation names a set rather than a reaction'

# A citation must not be satisfiable by a MENTION. Without the definition-form match, a doc comment naming
# the test would read as coverage — the exact silent pass the register opposes.
mention_only=$(new_repo mention-only "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '// See a_probe_bound_is_pinned( for the bound this module claims.
pub fn something_else() {}
')
expect_fail "$mention_only" 1 'which no function under crates/ defines'

# --- the prose floor, and the reference that clears it ---

stray_prose=$(new_repo stray-prose "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
Some paragraph that calls something a stated bound without declaring it.')")
expect_fail "$stray_prose" 1 'states a bound outside any declared bound scenario'

# The reference paragraph deliberately follows a NON-bound scenario: prose inside a bound scenario is part
# of the declaration and is not scanned, which is what keeps a bound's own WHEN/THEN from flagging itself.
referenced=$(new_repo referenced "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
That shape is a stated bound (bound: probe-capability/a-probed-shape-is-a-stated-bound), stated once.')")
expect_pass "$referenced" 'bound register ok (1 declared bounds'

# A mention whose negation lands on the bound NOUN denies the bound, so demanding its declaration would
# demand a declaration of what the sentence says does not exist.
negated=$(new_repo negated "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
Both files are read, a cfg-blind union rather than a skip bound, so nothing is skipped here.')")
expect_pass "$negated" 'bound register ok (1 declared bounds'

# And the direction that matters more, because the first attempt at the rule above broke it: a real
# declaration whose sentence carries a negation on a DIFFERENT verb must still be caught. Allowing the
# negation anywhere before the phrase hid three real declarations in this repository and caught none of the
# intended cases.
negation_elsewhere=$(new_repo negation-elsewhere "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
Type aliases are not expanded (a stated bound), so the shape stays unobserved.')")
expect_fail "$negation_elsewhere" 1 'states a bound outside any declared bound scenario'

dangling=$(new_repo dangling "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
That shape is a stated bound (bound: probe-capability/no-such-bound-was-ever-declared).')")
expect_fail "$dangling" 1 'which no declared bound produces'

# Two declared bounds whose headings differ only in punctuation collapse to one slug, so a reference to it
# names a set. This is what checks the derived id is injective rather than assuming it.
ambiguous_spec="$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
#### Scenario: A probed shape is a stated bound!
- **WHEN** the probe meets the shape again
- **THEN** it does not claim to observe it
- **PINNED-BY** `a_probe_bound_is_pinned`

#### Scenario: A plain scenario, so the paragraph below is not inside a bound
- **WHEN** anything
- **THEN** anything

Referring to it: a stated bound (bound: probe-capability/a-probed-shape-is-a-stated-bound).')"
ambiguous=$(new_repo ambiguous "$ambiguous_spec")
expect_fail "$ambiguous" 1 'which two declared bounds produce'

# --- cannot-judge directions ---

not_git=$fixture_root/not-git
mkdir -p "$not_git"
expect_fail "$not_git" 2 'is not a git worktree'

no_specs=$fixture_root/no-specs
mkdir -p "$no_specs"
git init -q "$no_specs"
git -C "$no_specs" config user.name 'Bound Register Test'
git -C "$no_specs" config user.email 'bound-register@example.invalid'
printf 'placeholder\n' >"$no_specs/README.md"
git -C "$no_specs" add -A
git -C "$no_specs" commit -qm 'no specs'
expect_fail "$no_specs" 2 'matched no openspec/specs'

# A spec present but declaring no bound at all: the heading form may have changed, so the gate must refuse
# to judge rather than report clean over a register it could not find.
no_bounds=$(new_repo no-bounds "$(printf '%s\n' \
    '# probe-capability Specification' \
    '' \
    '## Purpose' \
    '' \
    'Probe capability.' \
    '## Requirements' \
    '### Requirement: The probe observes something' \
    '' \
    'The probe SHALL observe the shape it claims.' \
    '' \
    '#### Scenario: The probe reacts' \
    '- **WHEN** the probe meets a real shape' \
    '- **THEN** it reacts')")
expect_fail "$no_bounds" 2 'parsed 0 declared bounds'

# --- read-only ---

before_tree=$(git -C "$pinned" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$pinned" rev-parse HEAD)
"$check" "$pinned" >/dev/null
after_tree=$(git -C "$pinned" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$pinned" rev-parse HEAD)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" ]] \
    || { printf 'bound register check mutated repository state\n' >&2; exit 1; }

printf 'ok bound register state and failure matrix\n'
