#!/usr/bin/env bash
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_release_coherence.sh
fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

write_workspace() {
    local repo=$1 version=$2
    mkdir -p "$repo/crates/xuanji" "$repo/crates/tianheng"
    printf '%s\n' \
        '[workspace]' \
        'members = ["crates/xuanji", "crates/tianheng"]' \
        '' \
        '[workspace.package]' \
        "version = \"$version\"" \
        '' \
        '[workspace.dependencies]' \
        "xuanji = { path = \"crates/xuanji\", version = \"$version\" }" \
        >"$repo/Cargo.toml"
    for package in xuanji tianheng; do
        printf '%s\n' \
            '[package]' \
            "name = \"$package\"" \
            'version.workspace = true' \
            'edition = "2024"' \
            >"$repo/crates/$package/Cargo.toml"
    done
    # An example carrying the adopter's published requirement, so the fixture has the shape
    # `require_example_pins` reads. Without one, that check's own vacuity guard fires here and every
    # state case in this matrix reports the missing-examples failure instead of what it is testing.
    mkdir -p "$repo/examples/adopter"
    printf '%s\n' \
        '[package]' \
        'name = "adopter"' \
        'version = "0.0.0"' \
        'edition = "2024"' \
        '' \
        '[dependencies]' \
        "xuanji = \"${version%.*}\"" \
        >"$repo/examples/adopter/Cargo.toml"
    printf '%s\n' \
        'version = 4' \
        '' \
        '[[package]]' \
        'name = "tianheng"' \
        "version = \"$version\"" \
        '' \
        '[[package]]' \
        'name = "xuanji"' \
        "version = \"$version\"" \
        >"$repo/Cargo.lock"
}

write_release_changelog() {
    local repo=$1 version=$2 previous=${3:-0.1.0}
    printf '%s\n' \
        '# Changelog' \
        '' \
        '## [Unreleased]' \
        '' \
        "## [$version] - 2026-07-20" \
        '' \
        '- Release notes.' \
        '' \
        "[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v$version...HEAD" \
        "[$version]: https://github.com/tacticaldoll/tianheng/compare/v$previous...v$version" \
        >"$repo/CHANGELOG.md"
}

write_development_changelog() {
    local repo=$1 version=$2 with_item=${3:-yes}
    {
        printf '%s\n' '# Changelog' '' '## [Unreleased]' ''
        if [[ $with_item == yes ]]; then
            printf '%s\n' '- An adopter-facing change.' ''
        fi
        printf '%s\n' "[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v$version...HEAD"
    } >"$repo/CHANGELOG.md"
}

new_repo() {
    local name=$1 version=${2:-0.2.0} repo
    repo=$fixture_root/$name
    mkdir -p "$repo"
    git -C "$repo" init -q
    git -C "$repo" config user.name 'Release Coherence Test'
    git -C "$repo" config user.email 'release-coherence@example.invalid'
    write_workspace "$repo" 0.1.0
    write_release_changelog "$repo" 0.1.0 0.0.0
    git -C "$repo" add .
    git -C "$repo" commit -qm 'release: 0.1.0'
    write_workspace "$repo" "$version"
    write_release_changelog "$repo" "$version" 0.1.0
    git -C "$repo" add .
    git -C "$repo" commit -qm "release: $version"
    printf '%s\n' "$repo"
}

commit_all() {
    local repo=$1 subject=$2
    git -C "$repo" add .
    git -C "$repo" commit -qm "$subject"
}

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

snapshot=$(new_repo snapshot)
expect_pass "$snapshot" 'snapshot: 0.2.0'

git -C "$snapshot" worktree add -q -b snapshot-worktree "$fixture_root/snapshot-worktree"
expect_pass "$fixture_root/snapshot-worktree" 'snapshot: 0.2.0'

development=$(new_repo development)
write_development_changelog "$development" 0.2.0
commit_all "$development" 'docs: describe pending work'
expect_pass "$development" 'development: 0.2.0'

ready=$(new_repo ready)
write_workspace "$ready" 0.2.1
write_release_changelog "$ready" 0.2.1 0.2.0
commit_all "$ready" 'chore: prepare release'
expect_pass "$ready" 'release-ready: 0.2.1'

missing_history=$fixture_root/missing-history
mkdir -p "$missing_history"
git -C "$missing_history" init -q
git -C "$missing_history" config user.name 'Release Coherence Test'
git -C "$missing_history" config user.email 'release-coherence@example.invalid'
write_workspace "$missing_history" 0.2.0
write_development_changelog "$missing_history" 0.2.0
commit_all "$missing_history" 'chore: initial import'
expect_fail "$missing_history" 2 'release history is unavailable'

malformed_history=$(new_repo malformed-history)
write_development_changelog "$malformed_history" 0.2.0
commit_all "$malformed_history" 'release: next'
expect_fail "$malformed_history" 1 'malformed release history subject: release: next'

regression=$(new_repo regression)
write_workspace "$regression" 0.1.9
write_development_changelog "$regression" 0.1.9
commit_all "$regression" 'chore: regress version'
expect_fail "$regression" 1 '0.1.9 is older than latest release 0.2.0'

empty_development=$(new_repo empty-development)
write_development_changelog "$empty_development" 0.2.0 no
commit_all "$empty_development" 'chore: omit release note'
expect_fail "$empty_development" 1 'requires adopter-facing release narrative'

stale_lock=$(new_repo stale-lock)
write_workspace "$stale_lock" 0.2.1
write_release_changelog "$stale_lock" 0.2.1 0.2.0
sed -i '0,/version = "0.2.1"/s//version = "0.2.0"/' "$stale_lock/Cargo.lock"
commit_all "$stale_lock" 'chore: leave stale lock'
expect_fail "$stale_lock" 1 'Cargo.lock package tianheng is 0.2.0; expected 0.2.1'

missing_notes=$(new_repo missing-notes)
write_workspace "$missing_notes" 0.2.1
write_development_changelog "$missing_notes" 0.2.1 no
commit_all "$missing_notes" 'chore: omit release section'
expect_fail "$missing_notes" 1 'missing dated release notes for 0.2.1'

missing_unreleased=$(new_repo missing-unreleased)
write_workspace "$missing_unreleased" 0.2.1
write_release_changelog "$missing_unreleased" 0.2.1 0.2.0
sed -i '/^## \[Unreleased\]$/d' "$missing_unreleased/CHANGELOG.md"
commit_all "$missing_unreleased" 'chore: omit unreleased section'
expect_fail "$missing_unreleased" 1 'exactly one [Unreleased] section'

invalid_link=$(new_repo invalid-link)
write_workspace "$invalid_link" 0.2.1
write_release_changelog "$invalid_link" 0.2.1 0.2.0
sed -i 's#compare/v0.2.0...v0.2.1#garbage#' "$invalid_link/CHANGELOG.md"
commit_all "$invalid_link" 'chore: break release comparison'
expect_fail "$invalid_link" 1 'comparison link for 0.2.1 must start at v0.2.0'

mismatched_snapshot=$fixture_root/mismatched-snapshot
mkdir -p "$mismatched_snapshot"
git -C "$mismatched_snapshot" init -q
git -C "$mismatched_snapshot" config user.name 'Release Coherence Test'
git -C "$mismatched_snapshot" config user.email 'release-coherence@example.invalid'
write_workspace "$mismatched_snapshot" 0.2.1
write_release_changelog "$mismatched_snapshot" 0.2.1 0.2.0
git -C "$mismatched_snapshot" add .
git -C "$mismatched_snapshot" commit -qm 'release: 0.2.0'
expect_fail "$mismatched_snapshot" 1 'subject is 0.2.0 but workspace version is 0.2.1'

# Failure branches of the manifest-and-pin checks — without these, a `require_workspace_manifests`
# or `require_internal_pins` that degraded to zero assertions (e.g. the crate glob silently emptied)
# would still pass the whole matrix. Each case makes exactly one of those checks the one that must fire.
missing_inheritance=$(new_repo missing-inheritance)
sed -i 's/^version\.workspace = true$/version = "0.2.0"/' "$missing_inheritance/crates/xuanji/Cargo.toml"
commit_all "$missing_inheritance" 'chore: pin a crate version literally'
expect_fail "$missing_inheritance" 1 'must inherit version.workspace = true'

mismatched_pin=$(new_repo mismatched-pin)
sed -i 's#version = "0.2.0" }#version = "0.1.0" }#' "$mismatched_pin/Cargo.toml"
commit_all "$mismatched_pin" 'chore: drift an internal pin'
expect_fail "$mismatched_pin" 1 'internal dependency xuanji is pinned to 0.1.0; expected 0.2.0'

# An example left behind by a release bump. This is the realistic release-prep slip: the workspace and
# the internal pins move together, and the examples' committed published requirement does not — after
# which Cargo silently drops their `patch.crates-io` override and they resolve the LAST PUBLISHED family
# from crates.io instead of the tree under development.
stale_example_pin=$(new_repo stale-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = "0.1"/' "$stale_example_pin/examples/adopter/Cargo.toml"
commit_all "$stale_example_pin" 'chore: leave an example on the previous minor'
expect_fail "$stale_example_pin" 1 'example adopter requires xuanji = "0.1"'

# And that check's own vacuity guards, so a renamed examples/ or a changed dependency form cannot make
# it pass with zero assertions.
missing_examples=$(new_repo missing-examples)
rm -rf "$missing_examples/examples"
commit_all "$missing_examples" 'chore: remove the examples directory'
expect_fail "$missing_examples" 2 'found no example manifests'

# The TABLE dependency form is read too, so an example using it is checked rather than skipped. Without
# this, one example moving to `{ version = "…" }` would go unverified while the set-level guard below
# stayed satisfied by its siblings — a silent hole exactly where this gate is supposed to be looking.
table_form_example_pin=$(new_repo table-form-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = { version = "0.1", features = ["audit"] }/' "$table_form_example_pin/examples/adopter/Cargo.toml"
commit_all "$table_form_example_pin" 'chore: stale table-form requirement in an example'
expect_fail "$table_form_example_pin" 1 'example adopter requires xuanji = "0.1"'

# The vacuity guard itself: an empty crate set (layout change / crates removed) must fail loud,
# not iterate the manifest and lock loops zero times and report coherent.
empty_crate_set=$(new_repo empty-crate-set)
find "$empty_crate_set/crates" -name Cargo.toml -delete
commit_all "$empty_crate_set" 'chore: remove crate manifests'
expect_fail "$empty_crate_set" 2 'found no workspace crate manifests'

# Read-only, on a fixture this gate has NOT already judged. Capturing `before` from a repository the gate had
# run over several times was blind by construction: a gate that writes the same file on every run leaves that
# file in `before` too, so the comparison held. Measured, not reasoned — a stray write injected into a sibling
# gate passed its read-only direction unnoticed until the fixture was made fresh.
untouched=$(new_repo untouched)
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

# The internal-pin loop's vacuity guard: it was the only loop in the gate without one, so a reformatted
# `[workspace.dependencies]` table iterated zero times and the direction passed having asserted nothing about
# any pin. The fixture keeps a real path dependency but in a form this line-oriented scan does not read.
vacuous_pins=$(new_repo vacuous-pins)
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
