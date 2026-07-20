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
    local repo=$1 expected=$2 output
    output=$("$check" "$repo")
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

before_tree=$(git -C "$development" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$development" rev-parse HEAD)
before_tags=$(git -C "$development" tag --list)
"$check" "$development" >/dev/null
after_tree=$(git -C "$development" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$development" rev-parse HEAD)
after_tags=$(git -C "$development" tag --list)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" && $before_tags == "$after_tags" ]] \
    || { printf 'release coherence check mutated repository state\n' >&2; exit 1; }

printf 'ok release coherence state and failure matrix\n'
