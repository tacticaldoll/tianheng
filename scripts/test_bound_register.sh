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

# A fixture that is a real cargo workspace, so the HARNESS direction can be proven. The manifest-less
# repositories above are deliberate — most of the register's directions have nothing to do with Rust — but
# test-ness is decided by `cargo test -- --list`, and a direction that cannot be proven on a fixture is not a
# direction. Measured: a crate this small enumerates COLD in ~107ms, which is why the premise that the matrix
# could not carry a manifest was wrong.
new_cargo_repo() {
    local name=$1 spec=$2 rust=$3
    local repo=$fixture_root/$name
    mkdir -p "$repo/openspec/specs/probe-capability" "$repo/crates/probe/src"
    git init -q "$repo"
    git -C "$repo" config user.name 'Bound Register Test'
    git -C "$repo" config user.email 'bound-register@example.invalid'
    printf '%s' "$spec" >"$repo/openspec/specs/probe-capability/spec.md"
    printf '%s' "$rust" >"$repo/crates/probe/src/lib.rs"
    printf 'probe debt\n' >"$repo/BACKLOG.md"
    printf '%s\n' '[workspace]' 'members = ["crates/*"]' 'resolver = "2"' >"$repo/Cargo.toml"
    printf '%s\n' '[package]' 'name = "probe"' 'version = "0.0.0"' 'edition = "2021"' \
        '' '[lib]' 'path = "src/lib.rs"' >"$repo/crates/probe/Cargo.toml"
    printf 'target/\n' >"$repo/.gitignore"
    git -C "$repo" add -A
    git -C "$repo" commit -qm 'probe cargo fixture'
    local bless_output bless_status=0
    bless_output=$(BLESS=1 "$check" "$repo" 2>&1) || bless_status=$?
    [[ -f $repo/docs/observation-bounds.md ]] \
        || { printf 'blessing the cargo fixture wrote nothing (exit %d): %s\n' "$bless_status" "$bless_output" >&2; exit 1; }
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

# A whole definition inside a block comment satisfies a citation on the FALLBACK path, and this fixture pins
# that: the source-text walk reads the form of a line, not its comment state. It is a property of the declared
# fallback and not of the register's judgment where a manifest exists — the cargo fixtures below prove the
# harness refuses the same shape. Closing it in the fallback would need string-literal lexing, which this tree
# defeats with 49 in-string `/*` occurrences.
commented_definition=$(new_repo commented-definition "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '/*
#[test]
fn a_probe_bound_is_pinned() {}
*/
')
expect_pass "$commented_definition" 'bound register ok (1 declared bounds'

# The fallback must SAY it is the fallback. A gate that silently drops its strongest direction reports a weaker
# clean than the one it claims.
expect_pass "$commented_definition" 'no root Cargo.toml — citation test-ness decided by the source-text fallback'

# A raw identifier is a Rust identifier, and the register imposes no naming convention of its own.
raw_identifier=$(new_repo raw-identifier "$(spec_with '- **PINNED-BY** `r#type`')" \
    '#[test]
fn r#type() {}
')
expect_pass "$raw_identifier" 'bound register ok (1 declared bounds'

# --- the citation search reads TRACKED content, not the worktree ---

# An untracked `.rs` file holding a same-named definition must not decide a citation. A `grep -r` walk
# counted it, so a scratch copy of a test file — the likeliest such artifact, tests being what citations
# name — resolved the name twice and the gate refused with "defined 2 times" on a developer's machine
# while CI's clean checkout passed. The fixture keeps the tracked definition legitimate and adds the
# untracked twin AFTER the commit, so only trackedness distinguishes them.
untracked_twin=$(new_repo untracked-twin "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf '%s' "$DEFAULT_RUST" >"$untracked_twin/crates/probe/src/scratch_copy.rs"
expect_pass "$untracked_twin" 'bound register ok (1 declared bounds'

# The same file, tracked, IS a second definition and must still refuse — otherwise the direction above
# would have been bought by blinding the search rather than by scoping it.
tracked_twin=$(new_repo tracked-twin "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf '%s' "$DEFAULT_RUST" >"$tracked_twin/crates/probe/src/second_definition.rs"
git -C "$tracked_twin" add -A
git -C "$tracked_twin" commit -qm 'a second tracked definition'
expect_fail "$tracked_twin" 1 'the citation names a set rather than a reaction'

# --- an enumeration that FAILED is not an empty repository ---

# `mapfile -d '' -t arr < <(git ls-files -z …)` reads a subshell whose status no one sees, and `pipefail`
# does not reach it, so a failed enumeration returned exactly what a repository holding nothing returns.
# The census direction is the one that reports CLEAN on that reading, which is why the fixture is built to
# hold a stale census: without the status check the run exits 0 over a document it never examined. The
# tracker and citation directions fail the other way — refusing every bound in the register for a `git`
# failure — and share the one enumerator this proves.
#
# `git` is stubbed to fail for the CENSUS enumeration alone (`ls-files -z -- *.md`) and to pass everything
# else through, so the case proves that enumeration's own direction rather than "the gate needs git": the
# worktree check, the spec-file enumeration, and the tracked-path index all still succeed.
failed_enumeration=$(new_repo failed-enumeration "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'This register holds 99 bounds across 7 capabilities.\n' >>"$failed_enumeration/BACKLOG.md"
git -C "$failed_enumeration" add -A
git -C "$failed_enumeration" commit -qm 'a stale written census'
expect_fail "$failed_enumeration" 1 'writes "99 bounds across 7 capabilities"'

git_stub=$fixture_root/git-stub
mkdir -p "$git_stub"
real_git=$(command -v git)
cat >"$git_stub/git" <<STUB
#!/usr/bin/env bash
census=0
for arg in "\$@"; do
    [[ \$arg == '*.md' ]] && census=1
done
if [[ \$census == 1 ]]; then
    for arg in "\$@"; do
        [[ \$arg == ls-files ]] && exit 3
    done
fi
exec "$real_git" "\$@"
STUB
chmod +x "$git_stub/git"

enumeration_status=0
enumeration_output=$(PATH="$git_stub:$PATH" "$check" "$failed_enumeration" 2>&1) || enumeration_status=$?
[[ $enumeration_status -eq 2 ]] \
    || { printf 'a failed tracked-file enumeration must exit 2, got %d: %s\n' "$enumeration_status" "$enumeration_output" >&2; exit 1; }
grep -Fq 'failed enumerating' <<<"$enumeration_output" \
    || { printf 'a failed enumeration must name itself, got: %s\n' "$enumeration_output" >&2; exit 1; }

# The CITATION enumeration, which reaches the same enumerator down a different path — and the path that had
# to be straightened for its refusal to survive. The scan runs inside `definitions_into`, which was consumed
# through a process substitution: a `cannot_judge` there exited only that subshell, the parent read an empty
# site list, and the gate reported "no function under crates/ defines it" — a violation invented from a `git`
# failure. Asserting the exit CODE distinguishes the two: the false answer is 1, the honest one is 2.
citation_git_stub=$fixture_root/citation-git-stub
mkdir -p "$citation_git_stub"
cat >"$citation_git_stub/git" <<STUB
#!/usr/bin/env bash
rs=0
for arg in "\$@"; do
    [[ \$arg == 'crates/*.rs' ]] && rs=1
done
if [[ \$rs == 1 ]]; then
    for arg in "\$@"; do
        [[ \$arg == ls-files ]] && exit 3
    done
fi
exec "$real_git" "\$@"
STUB
chmod +x "$citation_git_stub/git"

citation_status=0
citation_output=$(PATH="$citation_git_stub:$PATH" "$check" "$pinned" 2>&1) || citation_status=$?
[[ $citation_status -eq 2 ]] \
    || { printf 'a failed citation enumeration must exit 2, got %d: %s\n' "$citation_status" "$citation_output" >&2; exit 1; }
grep -Fq 'failed enumerating crates/*.rs' <<<"$citation_output" \
    || { printf 'a failed citation enumeration must name itself, got: %s\n' "$citation_output" >&2; exit 1; }

# And `grep` failing to READ, which is a different claim from finding no match. Exit 1 is the zero-sites
# answer the register is written for; exit >1 used to be discarded with `|| true`, so a citation was reported
# as defined nowhere because a file could not be opened.
grep_stub=$fixture_root/grep-stub
mkdir -p "$grep_stub"
cat >"$grep_stub/grep" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    case \$arg in
    *'fn[[:space:]]+'*) exit 2 ;;
    esac
done
exec "$(command -v grep)" "\$@"
STUB
chmod +x "$grep_stub/grep"

unreadable_status=0
unreadable_output=$(PATH="$grep_stub:$PATH" "$check" "$pinned" 2>&1) || unreadable_status=$?
[[ $unreadable_status -eq 2 ]] \
    || { printf 'grep failing to read must exit 2, got %d: %s\n' "$unreadable_status" "$unreadable_output" >&2; exit 1; }
grep -Fq 'could not read the tracked Rust files' <<<"$unreadable_output" \
    || { printf 'an unreadable definition scan must name itself, got: %s\n' "$unreadable_output" >&2; exit 1; }

# Every remaining READ of the observation source, each stubbed to fail on its own call. These are not
# enumerations, which is why they outlived the enumeration repair: a read whose status is discarded produces
# the same two wrong answers — a clean report over content never examined, or a violation invented from an
# IO failure — and the direction is what earns the check, not the probability.

# The CENSUS read. `|| true` swallowed `grep`'s exit >1, so a tracked document the direction claims to cover
# went unexamined behind a clean report. The fixture holds a stale census, so the false answer is exit 0.
census_grep_stub=$fixture_root/census-grep-stub
mkdir -p "$census_grep_stub"
cat >"$census_grep_stub/grep" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    case \$arg in
    *'bounds across'*) exit 2 ;;
    esac
done
exec "$(command -v grep)" "\$@"
STUB
chmod +x "$census_grep_stub/grep"

census_read_status=0
census_read_output=$(PATH="$census_grep_stub:$PATH" "$check" "$failed_enumeration" 2>&1) || census_read_status=$?
[[ $census_read_status -eq 2 ]] \
    || { printf 'an unreadable census scan must exit 2, got %d: %s\n' "$census_read_status" "$census_read_output" >&2; exit 1; }
grep -Fq 'could not read the tracked Markdown' <<<"$census_read_output" \
    || { printf 'an unreadable census scan must name itself, got: %s\n' "$census_read_output" >&2; exit 1; }

# The ATTRIBUTE-RUN read, which runs only in the manifest-less fallback: without its status checked, a real
# test reads as carrying no `#[test]` — exit 1. The stub matches the `1,<N>p` form alone, so the spec parse's
# own `sed` still runs.
attribute_sed_stub=$fixture_root/attribute-sed-stub
mkdir -p "$attribute_sed_stub"
cat >"$attribute_sed_stub/sed" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    [[ \$arg =~ ^1,[0-9]+p$ ]] && exit 5
done
exec "$(command -v sed)" "\$@"
STUB
chmod +x "$attribute_sed_stub/sed"

attribute_read_status=0
attribute_read_output=$(PATH="$attribute_sed_stub:$PATH" "$check" "$pinned" 2>&1) || attribute_read_status=$?
[[ $attribute_read_status -eq 2 ]] \
    || { printf 'an unreadable attribute run must exit 2, got %d: %s\n' "$attribute_read_status" "$attribute_read_output" >&2; exit 1; }
grep -Fq 'while checking whether the definition at line' <<<"$attribute_read_output" \
    || { printf 'an unreadable attribute run must name itself, got: %s\n' "$attribute_read_output" >&2; exit 1; }

# --- the exit contract binds every path, including one nobody wrapped ---

# `set -e` with `pipefail` carries a failing utility's status out of the process: measured before this was
# fixed, a stubbed `sed` made the gate exit 4 with no output at all. A status the contract does not define is
# one no consumer can act on, so the assertion is on the CODE — 2, not "non-zero".
#
# `mktemp` is the injection point because nothing wraps it: it is the first thing the run does after parsing
# arguments, which is precisely the kind of site an ERR trap exists for and a per-command wrapper misses.
mktemp_stub=$fixture_root/mktemp-stub
mkdir -p "$mktemp_stub"
cat >"$mktemp_stub/mktemp" <<'STUB'
#!/usr/bin/env bash
exit 7
STUB
chmod +x "$mktemp_stub/mktemp"

unhandled_status=0
unhandled_output=$(PATH="$mktemp_stub:$PATH" "$check" "$pinned" 2>&1) || unhandled_status=$?
[[ $unhandled_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 2, not the utility status, got %d: %s\n' "$unhandled_status" "$unhandled_output" >&2; exit 1; }
grep -Fq 'an unhandled command failed' <<<"$unhandled_output" \
    || { printf 'an unhandled failure must say so and name where, got: %s\n' "$unhandled_output" >&2; exit 1; }

# A spec that cannot be READ keeps its own diagnosis rather than falling to the backstop: "an unhandled
# command failed at line N" is a worse answer than the spec, and this read has a name worth giving.
spec_sed_stub=$fixture_root/spec-sed-stub
mkdir -p "$spec_sed_stub"
cat >"$spec_sed_stub/sed" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    case \$arg in
    *spec.md) exit 6 ;;
    esac
done
exec "$(command -v sed)" "\$@"
STUB
chmod +x "$spec_sed_stub/sed"

unreadable_spec_status=0
unreadable_spec_output=$(PATH="$spec_sed_stub:$PATH" "$check" "$pinned" 2>&1) || unreadable_spec_status=$?
[[ $unreadable_spec_status -eq 2 ]] \
    || { printf 'an unreadable spec must exit 2, got %d: %s\n' "$unreadable_spec_status" "$unreadable_spec_output" >&2; exit 1; }
grep -Fq 'could not read the declared bounds from' <<<"$unreadable_spec_output" \
    || { printf 'an unreadable spec must name itself rather than fall to the backstop, got: %s\n' "$unreadable_spec_output" >&2; exit 1; }

# --- a tracked spec absent from the worktree cannot be judged ---

# `git ls-files` lists it and the worktree does not hold it. Skipping it dropped its bounds from the
# verdicts AND from the projection, so the two agreed with each other about a register neither had read.
#
# TWO tracked specs, and the SECOND is the one removed — deliberately, because a single-spec fixture
# cannot show the defect: with nothing left to parse the vacuity guard fires and the run exits 2 anyway,
# on a different diagnosis.
#
# Two directions, and what each is worth. JUDGING was never a silent pass in a repository whose
# projection is committed: the document still described both specs, so the old `continue` surfaced as a
# stale-projection failure — exit 1 blaming the projection for a tree that could not be read. The gain
# there is the diagnosis, and that is all it is claimed to be.
#
# BLESSING is where the silence was. The write happens after this loop, so a skipped spec produced a
# projection rewritten to describe a partial register, exit 0, and a document that reads as the complete
# register of a repository holding fewer bounds than it does. Refusing before the write is what closes it,
# so the projection is asserted UNCHANGED as well as the exit code.
absent_spec=$(new_repo absent-spec "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" '' nobless)
mkdir -p "$absent_spec/openspec/specs/second-capability"
spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' >"$absent_spec/openspec/specs/second-capability/spec.md"
git -C "$absent_spec" add -A
git -C "$absent_spec" commit -qm 'a second tracked spec'
BLESS=1 "$check" "$absent_spec" >/dev/null 2>&1 || true
projection_before=$(cat "$absent_spec/docs/observation-bounds.md")
rm -f "$absent_spec/openspec/specs/second-capability/spec.md"
expect_fail "$absent_spec" 2 'absent from the worktree'

absent_spec_bless_status=0
absent_spec_bless_output=$(BLESS=1 "$check" "$absent_spec" 2>&1) || absent_spec_bless_status=$?
[[ $absent_spec_bless_status -eq 2 ]] \
    || { printf 'blessing over an absent tracked spec must exit 2, got %d: %s\n' "$absent_spec_bless_status" "$absent_spec_bless_output" >&2; exit 1; }
[[ $(cat "$absent_spec/docs/observation-bounds.md") == "$projection_before" ]] \
    || { printf 'blessing over an absent tracked spec rewrote the projection to a partial register\n' >&2; exit 1; }

# --- the harness is the authority, proven on real cargo workspaces ---

# The passing direction first: a real registered test resolves, and the gate says the harness decided.
harness_pass=$(new_cargo_repo harness-pass "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[cfg(test)]
mod tests {
    #[test]
    fn a_probe_bound_is_pinned() {}
}
')
expect_pass "$harness_pass" 'citation test-ness decided by the test harness'
expect_pass "$harness_pass" 'bound register ok (1 declared bounds'

# The HARNESS leaf parse, on a real workspace: without its status checked, a package whose test names could
# not be read contributes none, and every citation qualified to it is reported as unregistered — exit 1.
harness_sed_stub=$fixture_root/harness-sed-stub
mkdir -p "$harness_sed_stub"
cat >"$harness_sed_stub/sed" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    case \$arg in
    *': test$'*) exit 4 ;;
    esac
done
exec "$(command -v sed)" "\$@"
STUB
chmod +x "$harness_sed_stub/sed"

harness_parse_status=0
harness_parse_output=$(PATH="$harness_sed_stub:$PATH" "$check" "$harness_pass" 2>&1) || harness_parse_status=$?
[[ $harness_parse_status -eq 2 ]] \
    || { printf 'an unparsable harness listing must exit 2, got %d: %s\n' "$harness_parse_status" "$harness_parse_output" >&2; exit 1; }
grep -Fq 'could not parse the test names' <<<"$harness_parse_output" \
    || { printf 'an unparsable harness listing must name itself, got: %s\n' "$harness_parse_output" >&2; exit 1; }

# A PARTIAL package enumeration, which is the shape the previous guard missed: it caught a totally empty
# result, so an enumeration that emitted some entries and then failed left a short list reading as
# authoritative — the gate judged citations against a harness index built from one package out of six. The
# stub emits one manifest and then fails, so only the status distinguishes it from a one-package workspace.
partial_git_stub=$fixture_root/partial-git-stub
mkdir -p "$partial_git_stub"
cat >"$partial_git_stub/git" <<STUB
#!/usr/bin/env bash
manifests=0
for arg in "\$@"; do
    [[ \$arg == 'crates/*/Cargo.toml' ]] && manifests=1
done
if [[ \$manifests == 1 ]]; then
    for arg in "\$@"; do
        if [[ \$arg == ls-files ]]; then
            printf 'crates/probe/Cargo.toml\0'
            exit 3
        fi
    done
fi
exec "$real_git" "\$@"
STUB
chmod +x "$partial_git_stub/git"

partial_status=0
partial_output=$(PATH="$partial_git_stub:$PATH" "$check" "$harness_pass" 2>&1) || partial_status=$?
[[ $partial_status -eq 2 ]] \
    || { printf 'a partial package enumeration must exit 2, got %d: %s\n' "$partial_status" "$partial_output" >&2; exit 1; }
grep -Fq 'failed enumerating crates/*/Cargo.toml' <<<"$partial_output" \
    || { printf 'a partial package enumeration must name itself, got: %s\n' "$partial_output" >&2; exit 1; }

# A `#[test]` the build removes. The attribute run says test; nothing registers. The source-text fallback
# accepted this with exit 0, which is what moved the authority to the harness.
cfg_disabled=$(new_cargo_repo cfg-disabled "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[cfg(test)]
mod tests {
    #[test]
    #[cfg(any())]
    fn a_probe_bound_is_pinned() {}
}
')
expect_fail "$cfg_disabled" 1 'which the test harness does not register for that crate'

# Real `#[test] fn` tokens inside a macro nothing invokes expand nowhere, so they register nothing.
macro_body=$(new_cargo_repo macro-body "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    'macro_rules! never_invoked {
    () => {
        #[test]
        fn a_probe_bound_is_pinned() {}
    };
}
')
expect_fail "$macro_body" 1 'which the test harness does not register for that crate'

# A definition inside a multi-line string literal: the definition scan matches the line, the harness does not.
raw_string=$(new_cargo_repo raw-string "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    'pub const SRC: &str = r#"
#[test]
fn a_probe_bound_is_pinned() {}
"#;
')
expect_fail "$raw_string" 1 'which the test harness does not register for that crate'

# And the block-commented definition, which the previous change declared a residual and this one retires: the
# harness refuses it where the fallback above accepts it.
block_commented_definition=$(new_cargo_repo block-commented-definition "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '/*
#[test]
fn a_probe_bound_is_pinned() {}
*/
')
expect_fail "$block_commented_definition" 1 'which the test harness does not register for that crate'

# A registered test the definition scan cannot locate is a disagreement about a FORM, not about existence, so
# the reaction names the line shape it requires instead of reporting the test absent.
split_definition=$(new_cargo_repo split-definition "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" \
    '#[cfg(test)]
mod tests {
    #[test]
    pub fn
        a_probe_bound_is_pinned() {}
}
')
expect_fail "$split_definition" 1 'requires `fn` and the name on one source line'

# The direction that justifies enumerating PER PACKAGE rather than per workspace, and it is not hypothetical:
# `cargo test -- --list` prints `module::path::name` with no crate label, and this repository already has one
# test name registered in two crates. Here `alpha`'s test is cfg-disabled while `beta`'s is live, and the
# citation is qualified to `alpha`. A workspace-wide leaf match would find beta's test and pass — the hole this
# whole direction closes, reintroduced by the shortcut. Built by hand because it needs two crates.
crate_precision=$fixture_root/crate-precision
mkdir -p "$crate_precision/openspec/specs/probe-capability" \
    "$crate_precision/crates/alpha/src" "$crate_precision/crates/beta/src"
git init -q "$crate_precision"
git -C "$crate_precision" config user.name 'Bound Register Test'
git -C "$crate_precision" config user.email 'bound-register@example.invalid'
spec_with '- **PINNED-BY** `alpha::a_shared_test_name`' >"$crate_precision/openspec/specs/probe-capability/spec.md"
printf 'probe debt\n' >"$crate_precision/BACKLOG.md"
printf 'target/\n' >"$crate_precision/.gitignore"
printf '%s\n' '[workspace]' 'members = ["crates/*"]' 'resolver = "2"' >"$crate_precision/Cargo.toml"
for member in alpha beta; do
    printf '%s\n' '[package]' "name = \"$member\"" 'version = "0.0.0"' 'edition = "2021"' \
        '' '[lib]' 'path = "src/lib.rs"' >"$crate_precision/crates/$member/Cargo.toml"
done
printf '%s\n' '#[cfg(test)]' 'mod tests {' '    #[test]' '    #[cfg(any())]' \
    '    fn a_shared_test_name() {}' '}' >"$crate_precision/crates/alpha/src/lib.rs"
printf '%s\n' '#[cfg(test)]' 'mod tests {' '    #[test]' \
    '    fn a_shared_test_name() {}' '}' >"$crate_precision/crates/beta/src/lib.rs"
git -C "$crate_precision" add -A
git -C "$crate_precision" commit -qm 'crate precision fixture'
BLESS=1 "$check" "$crate_precision" >/dev/null 2>&1 || true
[[ -f $crate_precision/docs/observation-bounds.md ]] \
    || { printf 'blessing the crate-precision fixture wrote no projection\n' >&2; exit 1; }
expect_fail "$crate_precision" 1 'which the test harness does not register for that crate'

# And its counterpart, so the refusal above is precision rather than blanket refusal of a qualified citation:
# the same shape with the citation qualified to the crate whose test is live passes.
sed -i.bak 's/`alpha::a_shared_test_name`/`beta::a_shared_test_name`/' \
    "$crate_precision/openspec/specs/probe-capability/spec.md"
rm -f "$crate_precision/openspec/specs/probe-capability/spec.md.bak"
git -C "$crate_precision" commit -qam 'cite the live crate'
BLESS=1 "$check" "$crate_precision" >/dev/null 2>&1
expect_pass "$crate_precision" 'bound register ok (1 declared bounds'

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

# --- the marker admits no qualifier, and detection deliberately still does ---

# The classification a qualifier used to carry now lives in `xuanji::Extent`, so the marker is adjacent. This
# direction is refused EXPLICITLY rather than by non-match: an unmatched heading is not read as a bound at all,
# so it would surface through the prose floor below with a misleading message and the wrong repair.
# The fixture carries a properly-marked bound as WELL as the qualified one, deliberately. A repository whose
# only bound is refused parses zero bounds, and the vacuity guard then exits 2 before this direction can be
# seen — measured on the first version of this case, which conflated the qualifier refusal with the
# heading-form-changed guard and asserted the wrong code.
qualified_heading=$(new_repo qualified-heading "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
#### Scenario: Another probed shape is a stated coverage bound
- **WHEN** the probe meets that shape
- **THEN** it declines to look
- **PINNED-BY** `a_probe_bound_is_pinned`')")
expect_fail "$qualified_heading" 1 'marks a bound with a qualifier'

# And the other half of that change, which is the one a later reader is most likely to "fix" into consistency:
# the PROSE recognizer still admits the qualified wording the heading rule forbids. It is the register's
# detection floor, not a requirement on authored form, so narrowing it in step would stop it reporting a bound
# stated in prose with a qualifier — a false negative in the direction that stops the register being completed
# by declaring only the convenient bounds.
qualified_prose=$(new_repo qualified-prose "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
Some paragraph that calls something a stated coverage bound without declaring it.')")
expect_fail "$qualified_prose" 1 'states a bound outside any declared bound scenario'

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

# EVERY reference on the line is resolved, not just one of them. The extraction was a greedy
# `sed -n 's/.*(bound:…).*/\1/p'`, so `.*` swallowed everything up to the LAST reference and only that one was
# ever checked — an EARLIER dangling reference passed while the line reported clean. Measured in both
# directions, because "only one is checked" says nothing about which: an earlier dangler was accepted (exit 0)
# and a later one was already caught, so this fixture must dangle the FIRST reference to discriminate.
#
# This does NOT close the residual that let a retired `#[path]` bound survive: there the reference resolved
# and the sentence stated four other bounds. That one is in the projection header, because closing it means
# reading which bounds a sentence lists.
earlier_reference_dangling=$(new_repo earlier-reference-dangling "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' \
    '
Two shapes are stated bounds (bound: probe-capability/no-such-first-bound) and (bound: probe-capability/a-probed-shape-is-a-stated-bound).')")
expect_fail "$earlier_reference_dangling" 1 'no-such-first-bound`, which no declared bound produces'

# A reference is resolved wherever it sits, not only on a line that also says the trigger words. Reaching
# references only through a prose record meant rewording a sentence silently un-checked them — and a repair
# did exactly that here, turning `the module scanner's stated bounds` into `the module scanner's bounds, one
# reference per bound` and leaving the two references it added unresolved from then on. Fixture: a Purpose
# paragraph, which no trigger words appear in.
reference_off_a_triggering_line=$(new_repo reference-off-trigger "$(printf '%s\n' \
    '# probe-capability Specification' '' '## Purpose' '' \
    'The probe inherits bounds, one reference per bound: (bound: probe-capability/no-such-bound-at-all).' '' \
    '## Requirements' '### Requirement: The probe observes something' '' \
    'The probe SHALL observe the shape it claims.' '' \
    '#### Scenario: A probed shape is a stated bound' \
    '- **WHEN** the probe meets the shape' \
    '- **THEN** it does not claim to observe it' \
    '- **PINNED-BY** `a_probe_bound_is_pinned`')")
expect_fail "$reference_off_a_triggering_line" 1 'no-such-bound-at-all`, which no declared bound produces'

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

# The projection's CONTENT, not only its freshness. Byte-for-byte staleness checking proves the document and
# the reaction agree; it can never prove either is right, because both come from one renderer. A mangled
# apostrophe — `author\s:` where `author's:` was meant — sat in the tracked document through a full review
# because nothing ever asserted what the header says, only that it matched what the renderer would say again.
#
# So the disclosures a register reader depends on are asserted literally. Each of these is a claim the spec
# requires the header to make, and each is now a fixture rather than a hope.
projection=$headline/docs/observation-bounds.md
while IFS= read -r required; do
    [[ -n $required ]] || continue
    grep -Fq "$required" "$projection" \
        || { printf 'the projection header must state: %s\n' "$required" >&2; exit 1; }
# Each entry is a claim the spec REQUIRES the header to make — read from the document rather than
# remembered, because writing this list from memory is how the first attempt asserted a paragraph the
# header does not have. Historical notes in the header are deliberately absent: they are retired when the
# window ships, and a fixture must not pin what is meant to go away.
done <<'REQUIRED'
1. **Unrecognized wording.**
2. **The scan is line-oriented.**
3. **A reference clears more than it names.**
The **exemption**:
The second floor is the same shape.
declared bounds have no pinning test.
REQUIRED

# And nothing rendered may carry a backslash: none of this document's prose wants one, so a lone `\` is a
# quoting artifact of the renderer rather than content — which is exactly how `author\s:` reached the tree.
! grep -q '\\' "$projection" \
    || { printf 'the projection carries a backslash, which its prose never wants: %s\n' \
        "$(grep -n '\\' "$projection" | head -3)" >&2; exit 1; }

# --- the census direction ---

# A hand-written count of a set the reaction enumerates has no reaction, and this one went stale three times
# in one release window — the third time inside the CHANGELOG entry recording that the first two had. Counting
# by hand is not the failure: four independent careful counts of this tree produced four different answers for
# "citations". So the one shape that must live in prose is reacted to, and the rest were deleted from prose.
census_stale=$(new_repo census-stale "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'The register holds 9 bounds across 4 capabilities.\n' >>"$census_stale/BACKLOG.md"
git -C "$census_stale" add -A
git -C "$census_stale" commit -qm 'a stale census'
expect_fail "$census_stale" 1 'where the register holds 1 across 1'

# And a census that agrees passes, so the direction is not simply refusing the shape.
census_true=$(new_repo census-true "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'The register holds 1 bounds across 1 capabilities.\n' >>"$census_true/BACKLOG.md"
git -C "$census_true" add -A
git -C "$census_true" commit -qm 'a true census'
expect_pass "$census_true" 'bound register ok (1 declared bounds'

# A figure at the START of a line, which markdown reflow produces routinely. The first matcher guarded the
# number with `[^0-9]` to avoid reading `142 bounds` as `42`, and that guard cannot match at position zero —
# so a line-initial census was silently skipped while the identical figure mid-line was caught. The fixture
# above is mid-line, which is exactly why the gap was invisible to this matrix.
census_line_start=$(new_repo census-line-start "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf '9 bounds across 4 capabilities is what this document claims.\n' >>"$census_line_start/BACKLOG.md"
git -C "$census_line_start" add -A
git -C "$census_line_start" commit -qm 'a line-initial stale census'
expect_fail "$census_line_start" 1 'where the register holds 1 across 1'

# The guard the `[^0-9]` prefix was there for, kept now that the prefix is gone: `grep -o` is
# leftmost-longest, so a longer number is read whole rather than sliced into a false agreement. Without this,
# `21 bounds across 1 capabilities` in a one-bound register could match the trailing `1 bounds across 1` and
# pass.
census_longer_number=$(new_repo census-longer-number "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'The register holds 21 bounds across 1 capabilities.\n' >>"$census_longer_number/BACKLOG.md"
git -C "$census_longer_number" add -A
git -C "$census_longer_number" commit -qm 'a stale census whose count shares a suffix with the truth'
expect_fail "$census_longer_number" 1 'writes "21 bounds across 1 capabilities"'

# EVERY figure on the line, not the last one. A greedy `.*` examined only the final match, so an earlier
# stale figure passed while the line reported clean — the same partial check the reference direction was
# already repaired for, which is why it is proven here in the direction that discriminates: the FIRST figure
# is the stale one.
census_two_on_a_line=$(new_repo census-two-on-a-line "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'It listed 9 bounds across 4 capabilities and now holds 1 bounds across 1 capabilities.\n' \
    >>"$census_two_on_a_line/BACKLOG.md"
git -C "$census_two_on_a_line" add -A
git -C "$census_two_on_a_line" commit -qm 'two censuses on one line, the first stale'
expect_fail "$census_two_on_a_line" 1 'writes "9 bounds across 4 capabilities"'

# The direction's own residual, recorded as accepted behaviour rather than left to be rediscovered: the scan
# is line-oriented, so a figure reflowed across a line break is invisible to it. Measured, not reasoned — the
# same figure on one line is the control above and fails. Closing it would mean joining lines before matching,
# which costs the line number the diagnostic needs and would match across a paragraph boundary, so the spec
# states it instead. This fixture exists so a change that silently widens or closes the residual is visible.
census_reflowed=$(new_repo census-reflowed "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'The register holds 9 bounds across 4\ncapabilities today.\n' >>"$census_reflowed/BACKLOG.md"
git -C "$census_reflowed" add -A
git -C "$census_reflowed" commit -qm 'a stale census reflowed across a line break'
expect_pass "$census_reflowed" 'bound register ok (1 declared bounds'

# TRACKED content only. A filesystem walk judged the worktree, so an untracked scratch note failed the gate
# — a local file breaking a developer's run while CI's clean checkout passed. This gate's own header says it
# judges tracked content, and every other direction in it reads `git ls-files`.
census_untracked=$(new_repo census-untracked "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'Scratch: 9 bounds across 4 capabilities.\n' >"$census_untracked/scratch-notes.md"
expect_pass "$census_untracked" 'bound register ok (1 declared bounds'

# And the same for a path the repository ignores, which is the shape a vendored or generated tree takes. The
# walk excluded exactly one directory by name (`target/`), so every other ignored path was judged.
census_ignored=$(new_repo census-ignored "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')")
printf 'vendor/\n' >"$census_ignored/.gitignore"
mkdir -p "$census_ignored/vendor"
printf 'Vendored: 9 bounds across 4 capabilities.\n' >"$census_ignored/vendor/README.md"
git -C "$census_ignored" add .gitignore
git -C "$census_ignored" commit -qm 'ignore the vendor tree'
expect_pass "$census_ignored" 'bound register ok (1 declared bounds'

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

# A manifest is present, so the harness is the authority — and it cannot be enumerated. That is `cannot judge`,
# never the source-text fallback: a citation's test-ness is then UNDECIDED rather than weakly decided, and
# quietly dropping to the weaker direction would report a clean the gate cannot claim. Both triggers get their
# own fixture, because they are two ways into one exit and the matrix proves directions rather than exits.
#
# These two were the register's only unproven refusal, found by counting `cannot_judge` call sites against the
# matrix's `expect_fail … 2` assertions — 4 against 3. The gate's own requirement is that each failure
# direction is proven by a companion test, so an unproven one is that requirement violated a level up.

# Trigger one: `crates/<member>` is not a package, so `cargo test -p <member>` fails.
harness_unenumerable=$fixture_root/harness-unenumerable
mkdir -p "$harness_unenumerable/openspec/specs/probe-capability" "$harness_unenumerable/crates/probe/src"
git init -q "$harness_unenumerable"
git -C "$harness_unenumerable" config user.name 'Bound Register Test'
git -C "$harness_unenumerable" config user.email 'bound-register@example.invalid'
spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' >"$harness_unenumerable/openspec/specs/probe-capability/spec.md"
printf '%s' "$DEFAULT_RUST" >"$harness_unenumerable/crates/probe/src/lib.rs"
printf 'probe debt\n' >"$harness_unenumerable/BACKLOG.md"
# A root manifest declaring no members, and no manifest under `crates/probe` — so the directory looks like a
# package to this gate and is not one.
printf '%s\n' '[workspace]' 'members = []' 'resolver = "2"' >"$harness_unenumerable/Cargo.toml"
printf 'target/\n' >"$harness_unenumerable/.gitignore"
git -C "$harness_unenumerable" add -A
git -C "$harness_unenumerable" commit -qm 'unenumerable harness fixture'
expect_fail "$harness_unenumerable" 2 'the test harness could not be enumerated'

# Trigger two: a root manifest with no package directories at all under `crates/`, so there is nothing to
# enumerate and the index cannot be built either.
harness_no_members=$fixture_root/harness-no-members
mkdir -p "$harness_no_members/openspec/specs/probe-capability" "$harness_no_members/crates"
git init -q "$harness_no_members"
git -C "$harness_no_members" config user.name 'Bound Register Test'
git -C "$harness_no_members" config user.email 'bound-register@example.invalid'
spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`' >"$harness_no_members/openspec/specs/probe-capability/spec.md"
printf 'probe debt\n' >"$harness_no_members/BACKLOG.md"
printf '%s\n' '[workspace]' 'members = []' 'resolver = "2"' >"$harness_no_members/Cargo.toml"
printf 'target/\n' >"$harness_no_members/.gitignore"
git -C "$harness_no_members" add -A
git -C "$harness_no_members" commit -qm 'memberless harness fixture'
expect_fail "$harness_no_members" 2 'the test harness could not be enumerated'

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

# On a fixture this gate has NEVER judged, which for this twin means `no-bless`: `new_repo` blesses by
# default and blessing *runs the gate*, so a fixture built the usual way already carries whatever that run
# wrote. The earlier form captured `before` from exactly such a repository, and a stray write injected into
# the gate passed it unnoticed — a file written on every run sits in `before` as well as in `after`. Every
# sibling twin carried the same blindness in its own form and was corrected with it.
#
# The run then refuses (exit 1) because the projection it wants was never generated. That is the right
# fixture rather than a compromise: a refusing run must be read-only too, and this is the only path here
# where the gate is one flag away from writing a document.
untouched=$(new_repo untouched "$(spec_with '- **PINNED-BY** `a_probe_bound_is_pinned`')" '' no-bless)
before_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$untouched" rev-parse HEAD)
untouched_status=0
untouched_output=$("$check" "$untouched" 2>&1) || untouched_status=$?
# Asserted so the direction cannot pass by the gate exiting before it read anything.
[[ $untouched_status -eq 1 ]] \
    || { printf 'the read-only fixture must still reach a verdict, got exit %d: %s\n' "$untouched_status" "$untouched_output" >&2; exit 1; }
after_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$untouched" rev-parse HEAD)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" ]] \
    || { printf 'bound register check mutated repository state\n' >&2; exit 1; }

# A clean run must print NOTHING on stderr. What this replaces grepped for the backstop's own
# `an unhandled command failed`, so any *other* line a gate printed on a clean run while exiting 0 still read
# as clean — and a matrix that names one diagnostic has to track that diagnostic's wording. Emptiness has no
# wording to keep in step. `test_whitespace_hygiene.sh` documents the `errtrace` misfire the property descends
# from and is the matrix whose clean run actually exercises it.
clean_stderr=$("$check" "$pinned" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

printf 'ok bound register state and failure matrix\n'
