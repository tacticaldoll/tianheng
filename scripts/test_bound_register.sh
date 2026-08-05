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
    local name=$1 spec=$2 rust=${3:-} bless=${4:-bless} repo
    repo=$fixture_root/$name
    mkdir -p "$repo/openspec/specs/probe-capability" "$repo/crates/probe/src"
    git init -q "$repo"
    git -C "$repo" config user.name 'Bound Register Test'
    git -C "$repo" config user.email 'bound-register@example.invalid'
    printf '%s' "$spec" >"$repo/openspec/specs/probe-capability/spec.md"
    printf '%s' "${rust:-$DEFAULT_RUST}" >"$repo/crates/probe/src/lib.rs"
    # A tracked tracker document, because an `UNPINNED` citation must name a path the repository tracks. A
    # fixture without one would make every tracked-debt direction fail for the wrong reason.
    printf 'probe debt\n' >"$repo/BACKLOG.md"
    git -C "$repo" add -A
    git -C "$repo" commit -qm 'probe fixture'
    # Bless the projection so each case tests the condition it was built for rather than the absent
    # document. The projection directions below delete or edit it deliberately.
    #
    # Blessing a deliberately-broken fixture WRITES the projection and then fails on that fixture's own
    # offenses, so what this asserts is that the document exists — not that the exit code was 0. Asserting
    # the code would re-couple the matrix to the confusion the gate was changed to end: "regenerated" and
    # "valid" are different claims. The output is captured so the matrix stays readable and surfaced only
    # when nothing was written, which is the one failure this line can still produce.
    if [[ $bless == bless ]]; then
        local bless_output bless_status=0
        bless_output=$(BLESS=1 "$check" "$repo" 2>&1) || bless_status=$?
        [[ -f $repo/docs/observation-bounds.md ]] \
            || { printf 'blessing the fixture projection wrote nothing (exit %d): %s\n' "$bless_status" "$bless_output" >&2; exit 1; }
    fi
    printf '%s\n' "$repo"
}

# The default pinning test is a TEST. It was a plain `pub fn` until the citation direction was tightened to
# require one, and that fixture is exactly the hole the tightening closes: it proved the passing direction
# while demonstrating that a non-test satisfied a citation.
DEFAULT_RUST='#[test]
fn a_probe_bound_is_pinned() {}
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

# A tracker must NAME an owner, and any non-empty text used to satisfy that. A sentence restating that no
# test exists records the gap and gives it to nobody — which is the citation the requirement forbids in the
# same paragraph that permits `UNPINNED` at all.
asserts_absence=$(new_repo asserts-absence "$(spec_with '- **UNPINNED** no test exists')")
expect_fail "$asserts_absence" 1 'names no path this repository tracks'

# And a tracker naming a document the repository does not track is anonymous debt wearing an owner's name:
# the pointed-at file cannot be read, so the citation is the same class as a `PINNED-BY` naming a deleted
# test.
absent_tracker=$(new_repo absent-tracker "$(spec_with '- **UNPINNED** NOSUCH.md READY-PATCH "probe debt"')")
expect_fail "$absent_tracker" 1 'names no path this repository tracks'

# --- the pinning-test resolution directions ---

absent=$(new_repo absent "$(spec_with '- **PINNED-BY** `a_test_that_was_renamed_away`')")
expect_fail "$absent" 1 'which no function under crates/ defines'

twice=$(new_repo twice "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[test]
fn a_probe_bound_is_pinned() {}

mod second {
    #[test]
    fn a_probe_bound_is_pinned() {}
}
')
expect_fail "$twice" 1 'the citation names a set rather than a reaction'

# A citation must resolve to a TEST. A helper or production function of the right name defends nothing while
# occupying the place of the defence — the same silent coverage as an absent test, and the shape the matrix's
# own default fixture used to have.
not_a_test=$(new_repo not-a-test "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    'pub fn a_probe_bound_is_pinned() {}
')
expect_fail "$not_a_test" 1 'carries no `#[test]` in the attribute run above it'

# Test-ness is read from the attribute RUN, not the line before the definition: `#[test]` then
# `#[should_panic]` then `fn` exists in this tree three times, so a single-line read would refuse a real test.
interleaved=$(new_repo interleaved "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[test]
#[should_panic(expected = "probe")]
fn a_probe_bound_is_pinned() {}
')
expect_pass "$interleaved" 'bound register ok (1 declared bounds'

# And the leak the walk must not allow: a `#[test]` above one function must not read as covering the plain
# function beneath it.
attribute_leak=$(new_repo attribute-leak "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[test]
fn a_real_test() {}
pub fn a_probe_bound_is_pinned() {}
')
expect_fail "$attribute_leak" 1 'carries no `#[test]` in the attribute run above it'

# A commented-out attribute is a mention, not a marking — the same rule the definition match already follows.
commented_attribute=$(new_repo commented-attribute "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '// #[test]
pub fn a_probe_bound_is_pinned() {}
')
expect_fail "$commented_attribute" 1 'carries no `#[test]` in the attribute run above it'

# And the same in a BLOCK comment, which the line-comment rule above does not reach: the walk stops at the
# delimiter rather than reading commented text as an attribute. It cannot strip or track comments — comment
# state is a forward property an upward walk cannot know, and stripping needs string-literal lexing, which
# this tree's 49 in-string `/*` occurrences would defeat.
block_commented_attribute=$(new_repo block-commented-attribute "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '/*
#[test]
*/
pub fn a_probe_bound_is_pinned() {}
')
expect_fail "$block_commented_attribute" 1 'carries no `#[test]` in the attribute run above it'

# The walk has no line cap: the stop conditions are the boundary, so a run longer than any window still
# resolves. A 12-line cap refused this exact shape.
long_attribute_run=$(new_repo long-attribute-run "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[test]
#[allow(clippy::assertions_on_constants)] // 1
#[allow(clippy::assertions_on_constants)] // 2
#[allow(clippy::assertions_on_constants)] // 3
#[allow(clippy::assertions_on_constants)] // 4
#[allow(clippy::assertions_on_constants)] // 5
#[allow(clippy::assertions_on_constants)] // 6
#[allow(clippy::assertions_on_constants)] // 7
#[allow(clippy::assertions_on_constants)] // 8
#[allow(clippy::assertions_on_constants)] // 9
#[allow(clippy::assertions_on_constants)] // 10
#[allow(clippy::assertions_on_constants)] // 11
#[allow(clippy::assertions_on_constants)] // 12
#[allow(clippy::assertions_on_constants)] // 13
fn a_probe_bound_is_pinned() {}
')
expect_pass "$long_attribute_run" 'bound register ok (1 declared bounds'

# --- the citation must be well formed before it is resolved ---

# The cited name is interpolated into the search pattern, so a metacharacter resolved a citation for a test
# that does not exist to a differently-named function — defeating the renamed-or-deleted direction this gate
# was built for. Validated rather than escaped: escaping would report the citation stale when it is malformed.
metacharacter_name=$(new_repo metacharacter-name "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinne.`')")
expect_fail "$metacharacter_name" 1 'which is not a citation this reaction can resolve'

# The crate qualifier is joined to a filesystem path, so a traversal resolved a citation against a function
# outside the `crates/` boundary this reaction declares.
traversing_qualifier=$(new_repo traversing-qualifier "$(spec_with '- **PINNED-BY** `../outside::a_probe_bound_is_pinned`')")
expect_fail "$traversing_qualifier" 1 'which is not a citation this reaction can resolve'

# One `::` disambiguates a crate; a second names something this reaction does not resolve, so it is refused
# rather than silently read as a crate plus a leftover.
nested_qualifier=$(new_repo nested-qualifier "$(spec_with '- **PINNED-BY** `probe::inner::a_probe_bound_is_pinned`')")
expect_fail "$nested_qualifier" 1 'which is not a citation this reaction can resolve'

# --- the stated residual of matching a line's form ---

# A whole definition inside a block comment satisfies a citation. This fixture RECORDS that accepted residual
# rather than endorsing it: closing it needs the string-literal lexing the walk's comment rule rejects, and
# `docs/observation-bounds.md` states it as the register's third floor. If a later change closes it, this
# fixture fails, which is the point — the residual cannot be repaired silently.
commented_definition=$(new_repo commented-definition "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '/*
#[test]
fn a_probe_bound_is_pinned() {}
*/
')
expect_pass "$commented_definition" 'bound register ok (1 declared bounds'

# A citation must not be satisfiable by a MENTION. Without the definition-form match, a doc comment naming
# the test would read as coverage — the exact silent pass the register opposes.
mention_only=$(new_repo mention-only "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '// See a_probe_bound_is_pinned( for the bound this module claims.
pub fn something_else() {}
')
expect_fail "$mention_only" 1 'which no function under crates/ defines'

# A citation may be crate-qualified, and it has to be: the same test name legitimately exists in two
# dimensions for the same-shaped bound, and renaming a pre-existing test to suit this register is the one
# thing it must not require.
qualified=$(new_repo qualified "$(spec_with '- **PINNED-BY** `probe::a_probe_bound_is_pinned`')")
expect_pass "$qualified" 'bound register ok (1 declared bounds'

qualified_absent=$(new_repo qualified-absent "$(spec_with '- **PINNED-BY** `nosuchcrate::a_probe_bound_is_pinned`')")
expect_fail "$qualified_absent" 1 'which no function under crates/ defines'

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

# A requirement whose heading names bounds may state them in prose — several do it as a numbered list — but
# it must then declare at least one bound scenario, or the list has no reaction anywhere.
bounds_req_ok=$(new_repo bounds-req-ok "$(printf '%s\n' \
    '# probe-capability Specification' '' '## Purpose' '' 'Probe capability.' '## Requirements' \
    '### Requirement: Observation bounds are stated, not silent' '' \
    'The following are OUT OF SCOPE as stated coverage bounds: (1) one thing; (2) another.' '' \
    '#### Scenario: A probed shape is a stated bound' \
    '- **WHEN** the probe meets the shape' \
    '- **THEN** it does not claim to observe it' \
    '- **PINNED-BY** `a_probe_bound_is_pinned`')")
expect_pass "$bounds_req_ok" 'bound register ok (1 declared bounds'

bounds_req_bare=$(new_repo bounds-req-bare "$(printf '%s\n' \
    '# probe-capability Specification' '' '## Purpose' '' 'Probe capability.' '## Requirements' \
    '### Requirement: Observation bounds are stated, not silent' '' \
    'The following are OUT OF SCOPE as stated coverage bounds: (1) one thing; (2) another.' '' \
    '#### Scenario: The probe reacts' \
    '- **WHEN** the probe meets a real shape' \
    '- **THEN** it reacts' '' \
    '### Requirement: A second requirement' '' \
    'It SHALL exist.' '' \
    '#### Scenario: A probed shape is a stated bound' \
    '- **WHEN** the probe meets the shape' \
    '- **THEN** it does not claim to observe it' \
    '- **PINNED-BY** `a_probe_bound_is_pinned`')")
expect_fail "$bounds_req_bare" 1 'names bounds, so its prose may state them, but it declares no bound scenario'

# --- restatement across capabilities ---

# One behaviour has one defence, so the same test cited by two capabilities means one bound declared twice.
# Needs a second capability, so this fixture is built by hand rather than through spec_with.
restated=$fixture_root/restated
mkdir -p "$restated/openspec/specs/cap-one" "$restated/openspec/specs/cap-two" "$restated/crates/probe/src"
git init -q "$restated"
git -C "$restated" config user.name 'Bound Register Test'
git -C "$restated" config user.email 'bound-register@example.invalid'
printf '%s' "$DEFAULT_RUST" >"$restated/crates/probe/src/lib.rs"
for cap in cap-one cap-two; do
    printf '%s\n' \
        "# $cap Specification" '' '## Purpose' '' 'Probe capability.' '## Requirements' \
        '### Requirement: The probe observes something' '' \
        'The probe SHALL observe the shape it claims.' '' \
        '#### Scenario: A probed shape is a stated bound' \
        '- **WHEN** the probe meets the shape' \
        '- **THEN** it does not claim to observe it' \
        '- **PINNED-BY** `a_probe_bound_is_pinned`' >"$restated/openspec/specs/$cap/spec.md"
done
git -C "$restated" add -A
git -C "$restated" commit -qm 'restatement fixture'
# Blessing this fixture now exits 1 on the restatement it was built to carry, and still writes the
# projection. Asserting the document exists is what this line needs; asserting the exit code would assert
# the conflation the gate was changed to end.
BLESS=1 "$check" "$restated" >/dev/null 2>&1 || true
[[ -f $restated/docs/observation-bounds.md ]] \
    || { printf 'blessing the restatement fixture wrote no projection\n' >&2; exit 1; }
expect_fail "$restated" 1 'one behaviour has one defence'

# Repetition WITHIN one capability is not a restatement: a bound covering two shapes cites two tests, and
# one capability may cite one test from two bounds.
within=$(new_repo within "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`
- **PINNED-BY** `a_second_probe_bound_is_pinned`' \
    '
#### Scenario: A second probed shape is a stated bound
- **WHEN** the probe meets another shape
- **THEN** it does not claim to observe it
- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[test]
fn a_probe_bound_is_pinned() {}

#[test]
fn a_second_probe_bound_is_pinned() {}
')
expect_pass "$within" 'bound register ok (2 declared bounds'

# --- the projection ---

# Stale: the specs moved and the document did not. This is the direction that keeps a generated register from
# becoming a hand-maintained one, which is the drift it exists to avoid.
stale=$(new_repo stale "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf '\nhand edit\n' >>"$stale/docs/observation-bounds.md"
expect_fail "$stale" 1 'no longer matches the specs'

# And regenerating clears it, so the failure names a repair rather than a dead end.
BLESS=1 "$check" "$stale" >/dev/null
expect_pass "$stale" 'bound register ok'

missing=$(new_repo missing "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
rm -f "$missing/docs/observation-bounds.md"
expect_fail "$missing" 1 'is missing; generate it'

# The headline figure is the unpinned count, not a footnote — asserted on a fixture whose single bound is
# tracked rather than pinned.
headline=$(new_repo headline "$(spec_with '- **UNPINNED** BACKLOG.md "probe debt"')")
grep -Fq '**1 of 1 declared bounds have no pinning test.**' "$headline/docs/observation-bounds.md" \
    || { printf 'the projection must lead with the unpinned count\n' >&2; exit 1; }

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
# to judge rather than report clean over a register it could not find. Built WITHOUT blessing, because
# cannot-judge now precedes the projection write — this fixture is the direction that proves it.
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
    '- **THEN** it reacts')" '' no-bless)
expect_fail "$no_bounds" 2 'parsed 0 declared bounds'

# --- regeneration carries the exit contract ---

# Regeneration used to exit 0 here, which made it report the family's "clean" over a register whose offenses
# it had just printed. It writes the projection AND fails: the document is what the author needs in order to
# repair the register, and the exit code is what CI reads.
bless_offense=$(new_repo bless-offense "$(spec_with '')" '' no-bless)
bless_output=$(BLESS=1 "$check" "$bless_offense" 2>&1) && bless_status=0 || bless_status=$?
[[ $bless_status -eq 1 ]] \
    || { printf 'blessing a register with an offense must exit 1, got %d: %s\n' "$bless_status" "$bless_output" >&2; exit 1; }
[[ -f $bless_offense/docs/observation-bounds.md ]] \
    || { printf 'blessing must still write the projection it regenerated\n' >&2; exit 1; }
grep -Fq 'the register it describes is NOT valid' <<<"$bless_output" \
    || { printf 'blessing an invalid register must say so: %s\n' "$bless_output" >&2; exit 1; }

# And cannot-judge precedes the write, so a register whose declarations the gate could not find leaves behind
# no document that reads as a complete register of a repository holding no bounds.
bless_vacuous=$(new_repo bless-vacuous "$(printf '%s\n' \
    '# probe-capability Specification' '' '## Purpose' '' 'Probe capability.' '## Requirements' \
    '### Requirement: The probe observes something' '' \
    'The probe SHALL observe the shape it claims.' '' \
    '#### Scenario: The probe reacts' \
    '- **WHEN** the probe meets a real shape' \
    '- **THEN** it reacts')" '' no-bless)
bless_output=$(BLESS=1 "$check" "$bless_vacuous" 2>&1) && bless_status=0 || bless_status=$?
[[ $bless_status -eq 2 ]] \
    || { printf 'blessing a vacuous register must exit 2, got %d: %s\n' "$bless_status" "$bless_output" >&2; exit 1; }
[[ ! -e $bless_vacuous/docs/observation-bounds.md ]] \
    || { printf 'a register the gate cannot judge must leave no projection behind\n' >&2; exit 1; }

# --- read-only ---

before_tree=$(git -C "$pinned" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$pinned" rev-parse HEAD)
"$check" "$pinned" >/dev/null
after_tree=$(git -C "$pinned" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$pinned" rev-parse HEAD)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" ]] \
    || { printf 'bound register check mutated repository state\n' >&2; exit 1; }

printf 'ok bound register state and failure matrix\n'
