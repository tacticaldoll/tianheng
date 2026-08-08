#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_release_coherence.sh
fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# One construction of this capability's fixture, shared with the Rust file that cites its declared bounds.
# shellcheck source=lib/coherence_fixture.sh
source "$script_dir/lib/coherence_fixture.sh"


expect_pass() {
    local repo=$1 expected=$2 output status=0
    output=$("$check" "$repo") || status=$?
    [[ $status -eq 0 ]] \
        || { printf 'expected success (exit 0), got exit %d: %s\n' "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected success containing %q, got: %s\n' "$expected" "$output" >&2; exit 1; }
}

# The expected exit CODE, not merely non-zero. This matrix asserted `!= 0` and was blind to exactly the
# regression that followed: installing the shared exit-contract backstop turned this gate's `fail` — a
# `return 1` relying on `set -e` — into exit 2, so every genuine incoherence was reported as cannot-judge and
# CI stayed green. The family's contract separates a violation (1) from a gate that cannot decide (2)
# precisely so a consumer can act on the difference; a matrix that cannot see the difference cannot defend it.
expect_fail() {
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

snapshot=$(coherence_fixture_repo "$fixture_root" snapshot)
expect_pass "$snapshot" 'snapshot: 0.2.0'

git -C "$snapshot" worktree add -q -b snapshot-worktree "$fixture_root/snapshot-worktree"
expect_pass "$fixture_root/snapshot-worktree" 'snapshot: 0.2.0'

development=$(coherence_fixture_repo "$fixture_root" development)
coherence_fixture_development_changelog "$development" 0.2.0
coherence_fixture_commit "$development" 'docs: describe pending work'
expect_pass "$development" 'development: 0.2.0'

# `[Unreleased]` is adopter narrative, so it may name the planned release before the mechanical preparation
# advances the mutable surfaces this gate enumerates. Everything but this item's prose remains at 0.2.0.
intended_narrative=$(coherence_fixture_repo "$fixture_root" intended-narrative)
coherence_fixture_development_changelog "$intended_narrative" 0.2.0
sed -i 's/An adopter-facing change./Planned for 0.3.0: an adopter-facing change./' \
    "$intended_narrative/CHANGELOG.md"
coherence_fixture_commit "$intended_narrative" 'docs: describe intended release'
expect_pass "$intended_narrative" 'development: 0.2.0'

ready=$(coherence_fixture_repo "$fixture_root" ready)
coherence_fixture_workspace "$ready" 0.2.1
coherence_fixture_release_changelog "$ready" 0.2.1 0.2.0
coherence_fixture_commit "$ready" 'chore: prepare release'
expect_pass "$ready" 'release-ready: 0.2.1'

missing_history=$fixture_root/missing-history
mkdir -p "$missing_history"
git -C "$missing_history" init -q
git -C "$missing_history" config user.name 'Release Coherence Test'
git -C "$missing_history" config user.email 'release-coherence@example.invalid'
coherence_fixture_workspace "$missing_history" 0.2.0
coherence_fixture_development_changelog "$missing_history" 0.2.0
coherence_fixture_commit "$missing_history" 'chore: initial import'
expect_fail "$missing_history" 2 'release history is unavailable'

malformed_history=$(coherence_fixture_repo "$fixture_root" malformed-history)
coherence_fixture_development_changelog "$malformed_history" 0.2.0
coherence_fixture_commit "$malformed_history" 'release: next'
expect_fail "$malformed_history" 1 'malformed release history subject: release: next'

regression=$(coherence_fixture_repo "$fixture_root" regression)
coherence_fixture_workspace "$regression" 0.1.9
coherence_fixture_development_changelog "$regression" 0.1.9
coherence_fixture_commit "$regression" 'chore: regress version'
expect_fail "$regression" 1 '0.1.9 is older than latest release 0.2.0'

empty_development=$(coherence_fixture_repo "$fixture_root" empty-development)
coherence_fixture_development_changelog "$empty_development" 0.2.0 no
coherence_fixture_commit "$empty_development" 'chore: omit release note'
expect_fail "$empty_development" 1 'requires adopter-facing release narrative'

stale_lock=$(coherence_fixture_repo "$fixture_root" stale-lock)
coherence_fixture_workspace "$stale_lock" 0.2.1
coherence_fixture_release_changelog "$stale_lock" 0.2.1 0.2.0
sed -i '0,/version = "0.2.1"/s//version = "0.2.0"/' "$stale_lock/Cargo.lock"
coherence_fixture_commit "$stale_lock" 'chore: leave stale lock'
expect_fail "$stale_lock" 1 'Cargo.lock package tianheng is 0.2.0; expected 0.2.1'

# The lockfile direction must reach EVERY workspace package, not only the first. The case above stales
# `tianheng`, which the package enumeration yields first, so it is satisfied on the loop's first iteration and
# says nothing about the rest — every lockfile assertion this matrix made was about iteration one. This case
# stales the SECOND package, which is the shape a truncated package list hides: the loop ends early and a real
# disagreement goes unreported, a false negative rather than a wrong verdict.
stale_lock_second=$(coherence_fixture_repo "$fixture_root" stale-lock-second)
coherence_fixture_workspace "$stale_lock_second" 0.2.1
coherence_fixture_release_changelog "$stale_lock_second" 0.2.1 0.2.0
sed -i '/name = "xuanji"/{n;s/version = "0.2.1"/version = "0.2.0"/;}' "$stale_lock_second/Cargo.lock"
coherence_fixture_commit "$stale_lock_second" 'chore: leave the second package stale'
expect_fail "$stale_lock_second" 1 'Cargo.lock package xuanji is 0.2.0; expected 0.2.1'

missing_notes=$(coherence_fixture_repo "$fixture_root" missing-notes)
coherence_fixture_workspace "$missing_notes" 0.2.1
coherence_fixture_development_changelog "$missing_notes" 0.2.1 no
coherence_fixture_commit "$missing_notes" 'chore: omit release section'
expect_fail "$missing_notes" 1 'missing dated release notes for 0.2.1'

missing_unreleased=$(coherence_fixture_repo "$fixture_root" missing-unreleased)
coherence_fixture_workspace "$missing_unreleased" 0.2.1
coherence_fixture_release_changelog "$missing_unreleased" 0.2.1 0.2.0
sed -i '/^## \[Unreleased\]$/d' "$missing_unreleased/CHANGELOG.md"
coherence_fixture_commit "$missing_unreleased" 'chore: omit unreleased section'
expect_fail "$missing_unreleased" 1 'exactly one [Unreleased] section'

# --- the changelog's internal consistency ---
#
# Both directions were produced by the 0.5.0 window rather than imagined: an `[Unreleased]` grew a second
# `### Changed` heading three hundred lines from the first, and a prose claim about which releases carry a
# `### Migration` section was wrong under every reading. Neither was visible to anything until a mechanical
# sweep read the document's structure.

duplicate_heading=$(coherence_fixture_repo "$fixture_root" duplicate-heading)
coherence_fixture_workspace "$duplicate_heading" 0.2.0
coherence_fixture_development_changelog "$duplicate_heading" 0.2.0
python3 - "$duplicate_heading/CHANGELOG.md" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "- An adopter-facing change.\n",
    "### Changed\n- An adopter-facing change.\n\n### Changed\n- A second block of the same name.\n"))
EDIT
coherence_fixture_commit "$duplicate_heading" 'chore: split one section in two'
expect_fail "$duplicate_heading" 1 'repeats a heading'

breaking_without_migration=$(coherence_fixture_repo "$fixture_root" breaking-without-migration)
coherence_fixture_workspace "$breaking_without_migration" 0.2.0
coherence_fixture_development_changelog "$breaking_without_migration" 0.2.0
python3 - "$breaking_without_migration/CHANGELOG.md" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "- An adopter-facing change.\n",
    "### Changed\n- **BREAKING** an adopter-facing change with nowhere to read what to do.\n"))
EDIT
coherence_fixture_commit "$breaking_without_migration" 'chore: mark a break with no migration'
expect_fail "$breaking_without_migration" 1 'carries no `### Migration` section'

# The control for the direction above: the same break WITH the section is coherent, so the refusal is about the
# missing migration rather than about the marker.
breaking_with_migration=$(coherence_fixture_repo "$fixture_root" breaking-with-migration)
coherence_fixture_workspace "$breaking_with_migration" 0.2.0
coherence_fixture_development_changelog "$breaking_with_migration" 0.2.0
python3 - "$breaking_with_migration/CHANGELOG.md" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace(
    "- An adopter-facing change.\n",
    "### Changed\n- **BREAKING** an adopter-facing change.\n\n### Migration\n- Regenerate the baseline.\n"))
EDIT
coherence_fixture_commit "$breaking_with_migration" 'chore: mark a break and say what to do'
expect_pass "$breaking_with_migration" 'development: 0.2.0'

# --- adopter narrative names no self-governance machinery ---
#
# `CHANGELOG.md` is the adopter's document and offered no heading that was not an adopter's vocabulary, so
# nineteen entries name that machinery — ten in `[Unreleased]` and nine in the released `[0.4.0]` — every one
# of them under `### Added` / `### Changed` / `### Fixed`.
# Every direction below asserts the exit CODE, and the pair 3/4 is what holds the rule to the enumerator
# rather than to the `check_` prefix.

adopter_names_path=$(coherence_fixture_repo "$fixture_root" adopter-names-path)
coherence_fixture_development_changelog "$adopter_names_path" 0.2.0
coherence_fixture_machinery "$adopter_names_path"
coherence_fixture_unreleased_body "$adopter_names_path" '### Fixed
- A repair, described by naming `scripts/check_pin_bites.sh`.'
coherence_fixture_commit "$adopter_names_path" 'docs: name a gate under an adopter heading'
expect_fail "$adopter_names_path" 1 "names this repository's own machinery"

# The control: the SAME entry under the self-governance heading is coherent, so the refusal above is about the
# heading it sat under rather than about the path being named at all.
self_governance_heading=$(coherence_fixture_repo "$fixture_root" self-governance-heading)
coherence_fixture_development_changelog "$self_governance_heading" 0.2.0
coherence_fixture_machinery "$self_governance_heading"
coherence_fixture_unreleased_body "$self_governance_heading" '### Self-governance
- A repair, described by naming `scripts/check_pin_bites.sh`.'
coherence_fixture_commit "$self_governance_heading" 'docs: name a gate where it belongs'
expect_pass "$self_governance_heading" 'development: 0.2.0'

# The document cites both forms, so both are recognised.
adopter_names_basename=$(coherence_fixture_repo "$fixture_root" adopter-names-basename)
coherence_fixture_development_changelog "$adopter_names_basename" 0.2.0
coherence_fixture_machinery "$adopter_names_basename"
coherence_fixture_unreleased_body "$adopter_names_basename" '### Fixed
- A repair, described by naming `check_pin_bites.sh` with no directory.'
coherence_fixture_commit "$adopter_names_basename" 'docs: name a gate by basename'
expect_fail "$adopter_names_basename" 1 "names this repository's own machinery"

# Its control, and the direction that keeps the rule honest: a basename the enumerator does NOT resolve is not
# machinery, however much it looks like a gate. Without this, a matcher on the `check_`/`test_` prefix would
# pass the matrix while judging by a pattern rather than by what the repository tracks.
unresolved_basename=$(coherence_fixture_repo "$fixture_root" unresolved-basename)
coherence_fixture_development_changelog "$unresolved_basename" 0.2.0
coherence_fixture_machinery "$unresolved_basename"
coherence_fixture_unreleased_body "$unresolved_basename" '### Fixed
- A repair in an adopter tool named `check_something_the_repository_does_not_track.sh`.'
coherence_fixture_commit "$unresolved_basename" 'docs: name a file no scripts/ entry resolves'
expect_pass "$unresolved_basename" 'development: 0.2.0'

# The scope, pinned rather than inferred: a dated section records what was true at that release, and rewriting
# it to satisfy a rule written afterwards would falsify the record.
dated_names_path=$(coherence_fixture_repo "$fixture_root" dated-names-path)
coherence_fixture_machinery "$dated_names_path"
python3 - "$dated_names_path/CHANGELOG.md" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
text = p.read_text()
# The dated section names the gate; `[Unreleased]` carries an ordinary item, so the commit that follows the
# release leaves this in development state without itself citing anything.
text = text.replace("## [Unreleased]\n\n", "## [Unreleased]\n\n- An adopter-facing change.\n\n")
text = text.replace("- Release notes.\n",
                    "### Fixed\n- A repair, described by naming `scripts/check_pin_bites.sh`.\n")
p.write_text(text)
EDIT
coherence_fixture_commit "$dated_names_path" 'docs: a dated section names a gate'
expect_pass "$dated_names_path" 'development: 0.2.0'

# Recognition is by token. A bare substring matcher would fire on any sentence containing the characters,
# trading a declared blindness for an undeclared false-positive surface.
unquoted_prose=$(coherence_fixture_repo "$fixture_root" unquoted-prose)
coherence_fixture_development_changelog "$unquoted_prose" 0.2.0
coherence_fixture_machinery "$unquoted_prose"
coherence_fixture_unreleased_body "$unquoted_prose" '### Fixed
- A repair to the check_pin_bites.sh gate, written as prose rather than as a token.'
coherence_fixture_commit "$unquoted_prose" 'docs: name a gate as bare prose'
expect_pass "$unquoted_prose" 'development: 0.2.0'

# A repository tracking NO machinery has nothing an entry could leak, so it is clean — and it must reach that
# verdict by having nothing to match. Keyed on `NR == FNR`, an empty enumeration makes awk consume the
# changelog as its own enumerator, no section is ever emitted, and the gate refuses on the section vacuity
# guard. Run against that keying, this direction reports `expected success (exit 0), got exit 2` — measured,
# which is also how the first draft of that comment was corrected: the failure is a false refusal, not the
# silent pass it was written to claim.
no_tracked_machinery=$(coherence_fixture_repo "$fixture_root" no-tracked-machinery)
coherence_fixture_development_changelog "$no_tracked_machinery" 0.2.0
coherence_fixture_unreleased_body "$no_tracked_machinery" '### Fixed
- A repair, described by naming `scripts/check_pin_bites.sh`, in a repository tracking no such file.'
coherence_fixture_commit "$no_tracked_machinery" 'docs: name a gate the repository does not track'
expect_pass "$no_tracked_machinery" 'development: 0.2.0'

# The enumeration is the INDEX, not the worktree, which this repository's gates are held to generally. An
# untracked `scripts/` therefore reads as absent and the citation goes unseen — a declared bound rather than a
# defect, because reading worktree content here would break the rule the gate exists under.
untracked_machinery=$(coherence_fixture_repo "$fixture_root" untracked-machinery)
coherence_fixture_development_changelog "$untracked_machinery" 0.2.0
coherence_fixture_unreleased_body "$untracked_machinery" '### Fixed
- A repair, described by naming `scripts/check_pin_bites.sh`.'
coherence_fixture_commit "$untracked_machinery" 'docs: name a gate before it is tracked'
coherence_fixture_machinery "$untracked_machinery" # written, never added
expect_pass "$untracked_machinery" 'development: 0.2.0'

invalid_link=$(coherence_fixture_repo "$fixture_root" invalid-link)
coherence_fixture_workspace "$invalid_link" 0.2.1
coherence_fixture_release_changelog "$invalid_link" 0.2.1 0.2.0
sed -i 's#compare/v0.2.0...v0.2.1#garbage#' "$invalid_link/CHANGELOG.md"
coherence_fixture_commit "$invalid_link" 'chore: break release comparison'
expect_fail "$invalid_link" 1 'comparison link for 0.2.1 must start at v0.2.0'

mismatched_snapshot=$fixture_root/mismatched-snapshot
mkdir -p "$mismatched_snapshot"
git -C "$mismatched_snapshot" init -q
git -C "$mismatched_snapshot" config user.name 'Release Coherence Test'
git -C "$mismatched_snapshot" config user.email 'release-coherence@example.invalid'
coherence_fixture_workspace "$mismatched_snapshot" 0.2.1
coherence_fixture_release_changelog "$mismatched_snapshot" 0.2.1 0.2.0
git -C "$mismatched_snapshot" add .
git -C "$mismatched_snapshot" commit -qm 'release: 0.2.0'
expect_fail "$mismatched_snapshot" 1 'subject is 0.2.0 but workspace version is 0.2.1'

# Failure branches of the manifest-and-pin checks — without these, a `require_workspace_manifests`
# or `require_internal_pins` that degraded to zero assertions (e.g. the crate glob silently emptied)
# would still pass the whole matrix. Each case makes exactly one of those checks the one that must fire.
missing_inheritance=$(coherence_fixture_repo "$fixture_root" missing-inheritance)
sed -i 's/^version\.workspace = true$/version = "0.2.0"/' "$missing_inheritance/crates/xuanji/Cargo.toml"
coherence_fixture_commit "$missing_inheritance" 'chore: pin a crate version literally'
expect_fail "$missing_inheritance" 1 'must inherit version.workspace = true'

mismatched_pin=$(coherence_fixture_repo "$fixture_root" mismatched-pin)
sed -i 's#version = "0.2.0" }#version = "0.1.0" }#' "$mismatched_pin/Cargo.toml"
coherence_fixture_commit "$mismatched_pin" 'chore: drift an internal pin'
expect_fail "$mismatched_pin" 1 'internal dependency xuanji is pinned to 0.1.0; expected 0.2.0'

# An example left behind by a release bump. This is the realistic release-prep slip: the workspace and
# the internal pins move together, and the examples' committed published requirement does not — after
# which Cargo silently drops their `patch.crates-io` override and they resolve the LAST PUBLISHED family
# from crates.io instead of the tree under development.
stale_example_pin=$(coherence_fixture_repo "$fixture_root" stale-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = "0.1"/' "$stale_example_pin/examples/adopter/Cargo.toml"
coherence_fixture_commit "$stale_example_pin" 'chore: leave an example on the previous minor'
expect_fail "$stale_example_pin" 1 'example adopter requires xuanji = "0.1"'

# And that check's own vacuity guards, so a renamed examples/ or a changed dependency form cannot make
# it pass with zero assertions.
missing_examples=$(coherence_fixture_repo "$fixture_root" missing-examples)
rm -rf "$missing_examples/examples"
coherence_fixture_commit "$missing_examples" 'chore: remove the examples directory'
expect_fail "$missing_examples" 2 'found no example manifests'

# The TABLE dependency form is read too, so an example using it is checked rather than skipped. Without
# this, one example moving to `{ version = "…" }` would go unverified while the set-level guard below
# stayed satisfied by its siblings — a silent hole exactly where this gate is supposed to be looking.
table_form_example_pin=$(coherence_fixture_repo "$fixture_root" table-form-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = { version = "0.1", features = ["audit"] }/' "$table_form_example_pin/examples/adopter/Cargo.toml"
coherence_fixture_commit "$table_form_example_pin" 'chore: stale table-form requirement in an example'
expect_fail "$table_form_example_pin" 1 'example adopter requires xuanji = "0.1"'

# The vacuity guard itself: an empty crate set (layout change / crates removed) must fail loud,
# not iterate the manifest and lock loops zero times and report coherent.
empty_crate_set=$(coherence_fixture_repo "$fixture_root" empty-crate-set)
find "$empty_crate_set/crates" -name Cargo.toml -delete
coherence_fixture_commit "$empty_crate_set" 'chore: remove crate manifests'
expect_fail "$empty_crate_set" 2 'found no workspace crate manifests'

# Read-only, on a fixture this gate has NOT already judged. Capturing `before` from a repository the gate had
# run over several times was blind by construction: a gate that writes the same file on every run leaves that
# file in `before` too, so the comparison held. Measured, not reasoned — a stray write injected into a sibling
# gate passed its read-only direction unnoticed until the fixture was made fresh.
untouched=$(coherence_fixture_repo "$fixture_root" untouched)
before_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$untouched" rev-parse HEAD)
before_tags=$(git -C "$untouched" tag --list)
"$check" "$untouched" >/dev/null
after_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$untouched" rev-parse HEAD)
after_tags=$(git -C "$untouched" tag --list)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" && $before_tags == "$after_tags" ]] \
    || { printf 'release coherence check mutated repository state\n' >&2; exit 1; }

# An unhandled failure reports within the exit contract, not with the failing tool's own status. Measured
# before the backstop existed: with `git log` stubbed to fail, this gate exited **130** and printed nothing,
# which the contract does not define and an operator cannot read. `git log` is the injection point because it
# is what this gate is built on, and the stub passes every other git call through so the case proves the
# contract rather than "the gate needs git".
contract_stub=$fixture_root/contract-stub
mkdir -p "$contract_stub"
contract_real_git=$(command -v git)
cat >"$contract_stub/git" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    [[ \$arg == log ]] && exit 130
done
exec "$contract_real_git" "\$@"
STUB
chmod +x "$contract_stub/git"

contract_status=0
contract_output=$(PATH="$contract_stub:$PATH" "$check" "$development" 2>&1) || contract_status=$?
[[ $contract_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 2, not the tool status, got %d: %s\n' "$contract_status" "$contract_output" >&2; exit 1; }
grep -Fq 'an unhandled command failed' <<<"$contract_output" \
    || { printf 'an unhandled failure must say so and name where, got: %s\n' "$contract_output" >&2; exit 1; }

# A clean run must print NOTHING on stderr. What this replaces grepped for the backstop's own
# `an unhandled command failed`, so any *other* line a gate printed on a clean run while exiting 0 still read
# as clean — and a matrix that names one diagnostic has to track that diagnostic's wording. Emptiness has no
# wording to keep in step. `test_whitespace_hygiene.sh` documents the `errtrace` misfire the property descends
# from and is the matrix whose clean run actually exercises it.
clean_stderr=$("$check" "$development" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

# A partial read of the release history is not a shorter history. Before the shared capture rule this gate
# consumed `git log` through `< <(…)`, whose status the parent never sees: measured with a stub emitting one real
# release record and then exiting 7, it concluded the tree was in SNAPSHOT state and reported
# `[Unreleased] must be empty` — exit 1, a violation invented from truncated history, sending a maintainer to look
# for a problem that is not there. The other direction of the same class makes a gate report clean; this one makes
# it report a defect. Both are why the status is checked in the parent.
partial_history=$fixture_root/partial-history-stub
mkdir -p "$partial_history"
cat >"$partial_history/git" <<STUB
#!/usr/bin/env bash
for argument in "\$@"; do [[ \$argument == "--format=%H%x09%s" ]] && want=1; done
if [[ \${want:-0} -eq 1 ]]; then
    printf '%s\trelease: 0.2.0\n' "\$($(command -v git) -C "\$3" rev-parse HEAD 2>/dev/null || echo deadbeef)"
    exit 7
fi
exec $(command -v git) "\$@"
STUB
chmod +x "$partial_history/git"

partial_status=0
partial_output=$(PATH="$partial_history:$PATH" "$check" "$snapshot" 2>&1) || partial_status=$?
[[ $partial_status -eq 2 ]] \
    || { printf 'a truncated release history must exit 2, got %d: %s\n' "$partial_status" "$partial_output" >&2; exit 1; }
grep -Fq 'a failed read is not an empty result' <<<"$partial_output" \
    || { printf 'the refusal must name the partial read, got: %s\n' "$partial_output" >&2; exit 1; }

# The internal-pin loop's vacuity guard: it was the only loop in the gate without one, so a reformatted
# `[workspace.dependencies]` table iterated zero times and the direction passed having asserted nothing about
# any pin. The fixture keeps a real path dependency but in a form this line-oriented scan does not read.
vacuous_pins=$(coherence_fixture_repo "$fixture_root" vacuous-pins)
python3 - "$vacuous_pins" <<'PYEOF'
import pathlib, re, sys
p = pathlib.Path(sys.argv[1]) / "Cargo.toml"
t = p.read_text()
# same dependency, same path, split across lines — a form the single-line scan cannot see
t = re.sub(r'^\s*xuanji\s*=\s*\{[^}]*\}\s*$',
           'xuanji = {\n    path = "crates/xuanji",\n    version = "0.2.0",\n}', t, count=1, flags=re.M)
p.write_text(t)
PYEOF
git -C "$vacuous_pins" add -A
git -C "$vacuous_pins" commit -qm 'reformat the internal dependency table'
expect_fail "$vacuous_pins" 2 'found no internal path dependency'

printf 'ok release coherence state and failure matrix\n'
