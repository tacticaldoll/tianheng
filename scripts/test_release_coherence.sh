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

expect_fail() {
    local repo=$1 expected=$2 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -ne 0 ]] || { printf 'expected failure containing %q\n' "$expected" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected failure containing %q, got: %s\n' "$expected" "$output" >&2; exit 1; }
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
expect_fail "$missing_history" 'release history is unavailable'

malformed_history=$(new_repo malformed-history)
write_development_changelog "$malformed_history" 0.2.0
commit_all "$malformed_history" 'release: next'
expect_fail "$malformed_history" 'malformed release history subject: release: next'

regression=$(new_repo regression)
write_workspace "$regression" 0.1.9
write_development_changelog "$regression" 0.1.9
commit_all "$regression" 'chore: regress version'
expect_fail "$regression" '0.1.9 is older than latest release 0.2.0'

empty_development=$(new_repo empty-development)
write_development_changelog "$empty_development" 0.2.0 no
commit_all "$empty_development" 'chore: omit release note'
expect_fail "$empty_development" 'requires adopter-facing release narrative'

stale_lock=$(new_repo stale-lock)
write_workspace "$stale_lock" 0.2.1
write_release_changelog "$stale_lock" 0.2.1 0.2.0
sed -i '0,/version = "0.2.1"/s//version = "0.2.0"/' "$stale_lock/Cargo.lock"
commit_all "$stale_lock" 'chore: leave stale lock'
expect_fail "$stale_lock" 'Cargo.lock package tianheng is 0.2.0; expected 0.2.1'

missing_notes=$(new_repo missing-notes)
write_workspace "$missing_notes" 0.2.1
write_development_changelog "$missing_notes" 0.2.1 no
commit_all "$missing_notes" 'chore: omit release section'
expect_fail "$missing_notes" 'missing dated release notes for 0.2.1'

missing_unreleased=$(new_repo missing-unreleased)
write_workspace "$missing_unreleased" 0.2.1
write_release_changelog "$missing_unreleased" 0.2.1 0.2.0
sed -i '/^## \[Unreleased\]$/d' "$missing_unreleased/CHANGELOG.md"
commit_all "$missing_unreleased" 'chore: omit unreleased section'
expect_fail "$missing_unreleased" 'exactly one [Unreleased] section'

invalid_link=$(new_repo invalid-link)
write_workspace "$invalid_link" 0.2.1
write_release_changelog "$invalid_link" 0.2.1 0.2.0
sed -i 's#compare/v0.2.0...v0.2.1#garbage#' "$invalid_link/CHANGELOG.md"
commit_all "$invalid_link" 'chore: break release comparison'
expect_fail "$invalid_link" 'comparison link for 0.2.1 must start at v0.2.0'

mismatched_snapshot=$fixture_root/mismatched-snapshot
mkdir -p "$mismatched_snapshot"
git -C "$mismatched_snapshot" init -q
git -C "$mismatched_snapshot" config user.name 'Release Coherence Test'
git -C "$mismatched_snapshot" config user.email 'release-coherence@example.invalid'
write_workspace "$mismatched_snapshot" 0.2.1
write_release_changelog "$mismatched_snapshot" 0.2.1 0.2.0
git -C "$mismatched_snapshot" add .
git -C "$mismatched_snapshot" commit -qm 'release: 0.2.0'
expect_fail "$mismatched_snapshot" 'subject is 0.2.0 but workspace version is 0.2.1'

# Failure branches of the manifest-and-pin checks — without these, a `require_workspace_manifests`
# or `require_internal_pins` that degraded to zero assertions (e.g. the crate glob silently emptied)
# would still pass the whole matrix. Each case makes exactly one of those checks the one that must fire.
missing_inheritance=$(new_repo missing-inheritance)
sed -i 's/^version\.workspace = true$/version = "0.2.0"/' "$missing_inheritance/crates/xuanji/Cargo.toml"
commit_all "$missing_inheritance" 'chore: pin a crate version literally'
expect_fail "$missing_inheritance" 'must inherit version.workspace = true'

mismatched_pin=$(new_repo mismatched-pin)
sed -i 's#version = "0.2.0" }#version = "0.1.0" }#' "$mismatched_pin/Cargo.toml"
commit_all "$mismatched_pin" 'chore: drift an internal pin'
expect_fail "$mismatched_pin" 'internal dependency xuanji is pinned to 0.1.0; expected 0.2.0'

# An example left behind by a release bump. This is the realistic release-prep slip: the workspace and
# the internal pins move together, and the examples' committed published requirement does not — after
# which Cargo silently drops their `patch.crates-io` override and they resolve the LAST PUBLISHED family
# from crates.io instead of the tree under development.
stale_example_pin=$(new_repo stale-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = "0.1"/' "$stale_example_pin/examples/adopter/Cargo.toml"
commit_all "$stale_example_pin" 'chore: leave an example on the previous minor'
expect_fail "$stale_example_pin" 'example adopter requires xuanji = "0.1"'

# And that check's own vacuity guards, so a renamed examples/ or a changed dependency form cannot make
# it pass with zero assertions.
missing_examples=$(new_repo missing-examples)
rm -rf "$missing_examples/examples"
commit_all "$missing_examples" 'chore: remove the examples directory'
expect_fail "$missing_examples" 'found no example manifests'

# The TABLE dependency form is read too, so an example using it is checked rather than skipped. Without
# this, one example moving to `{ version = "…" }` would go unverified while the set-level guard below
# stayed satisfied by its siblings — a silent hole exactly where this gate is supposed to be looking.
table_form_example_pin=$(new_repo table-form-example-pin)
sed -i 's/^xuanji = "0.2"$/xuanji = { version = "0.1", features = ["audit"] }/' "$table_form_example_pin/examples/adopter/Cargo.toml"
commit_all "$table_form_example_pin" 'chore: stale table-form requirement in an example'
expect_fail "$table_form_example_pin" 'example adopter requires xuanji = "0.1"'

# The vacuity guard itself: an empty crate set (layout change / crates removed) must fail loud,
# not iterate the manifest and lock loops zero times and report coherent.
empty_crate_set=$(new_repo empty-crate-set)
find "$empty_crate_set/crates" -name Cargo.toml -delete
commit_all "$empty_crate_set" 'chore: remove crate manifests'
expect_fail "$empty_crate_set" 'found no workspace crate manifests'

before_tree=$(git -C "$development" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$development" rev-parse HEAD)
before_tags=$(git -C "$development" tag --list)
"$check" "$development" >/dev/null
after_tree=$(git -C "$development" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$development" rev-parse HEAD)
after_tags=$(git -C "$development" tag --list)
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

# A PASSING run must print no backstop diagnostic. The assertion exists because installing the shared `ERR`
# trap produced exactly that failure: `errtrace` propagates it into process substitutions, where a
# legitimately-failing command is routine, so a clean run emitted the cannot-judge line once per file while
# still exiting 0 — invisible to every check that reads only the exit code.
#
# What it does and does not hold, stated rather than implied: this fixture's clean run does not exercise a
# failing command inside a process substitution, so removing the backstop's subshell guard does NOT fail this
# assertion. The gate that misfired is `check_whitespace_hygiene.sh`, whose clean run does, and which has no
# companion matrix — filed in `BACKLOG.md`. This pins the property here, where a future change could break
# it, and the measurement is what covers the gate that has no fixture.
clean_noise=$("$check" "$development" 2>&1 >/dev/null || true)
grep -Fq 'an unhandled command failed' <<<"$clean_noise" \
    && { printf 'a passing run must print no backstop diagnostic, got: %s\n' "$clean_noise" >&2; exit 1; }

printf 'ok release coherence state and failure matrix\n'
