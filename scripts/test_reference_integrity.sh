#!/usr/bin/env bash
#
# Every state and failure direction of `check_reference_integrity.sh`, each on a throwaway repository.
#
# Proven against fixtures built to trip exactly one condition, separating a reference violation (exit 1)
# from a gate that cannot decide (exit 2).
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_reference_integrity.sh

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

# A minimal valid repository containing all required governance documents and one workspace member.
new_valid_repo() {
    local name=$1 repo
    repo=$fixture_root/$name
    mkdir -p "$repo/crates/probe/src" "$repo/docs"
    git init -q "$repo"
    git -C "$repo" config user.name 'Reference Integrity Test'
    git -C "$repo" config user.email 'reference-integrity@example.invalid'

    # Required governance documents
    printf '# AGENTS\nSee [PROJECT.md](PROJECT.md).\n' >"$repo/AGENTS.md"
    printf '# AGENTS self-law\n' >"$repo/AGENTS.self-law.md"
    printf '# BACKLOG\n' >"$repo/BACKLOG.md"
    printf '# CHANGELOG\n' >"$repo/CHANGELOG.md"
    printf '# COOKBOOK\n' >"$repo/COOKBOOK.md"
    printf '# PROJECT\n' >"$repo/PROJECT.md"
    printf '# README\n' >"$repo/README.md"
    printf '[workspace]\nmembers = ["crates/probe"]\n' >"$repo/Cargo.toml"
    printf '[bans]\n' >"$repo/deny.toml"

    # Workspace member
    printf '[package]\nname = "probe"\nversion = "0.1.0"\nedition = "2021"\n' >"$repo/crates/probe/Cargo.toml"
    printf '// Probe crate root\n' >"$repo/crates/probe/src/lib.rs"

    # Track content
    git -C "$repo" add -A
    git -C "$repo" commit -qm 'initial fixture commit'
    printf '%s\n' "$repo"
}

expect_pass() {
    local repo=$1 expected=$2 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq 0 ]] \
        || { printf 'expected success (exit 0), got exit %d: %s\n' "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected success containing %q, got: %s\n' "$expected" "$output" >&2; exit 1; }
}

expect_fail() {
    expect_fail_env "" "$@"
}

expect_fail_env() {
    local gov_docs=$1 repo=$2 expected_status=$3 expected=$4 output status=0
    if [ -n "$gov_docs" ]; then
        output=$(GOVERNANCE_DOCUMENTS="$gov_docs" "$check" "$repo" 2>&1) || status=$?
    else
        output=$("$check" "$repo" 2>&1) || status=$?
    fi
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

# 1. Clean valid repository
repo_clean=$(new_valid_repo clean)
expect_pass "$repo_clean" "reference integrity ok"

# 2. Missing governance document (exit 2)
repo_missing_gov=$(new_valid_repo missing_gov)
git -C "$repo_missing_gov" rm -q PROJECT.md
git -C "$repo_missing_gov" commit -qm 'remove PROJECT.md'
expect_fail "$repo_missing_gov" 2 "cannot judge: 'PROJECT.md' is named as one of this repository's governance documents"

# 3. No workspace member crates (exit 2)
repo_no_members=$(new_valid_repo no_members)
git -C "$repo_no_members" rm -rf -q crates/
printf '[workspace]\nmembers = []\n' >"$repo_no_members/Cargo.toml"
git -C "$repo_no_members" add -A
git -C "$repo_no_members" commit -qm 'remove member crates'
expect_fail "$repo_no_members" 2 "cannot judge: found no workspace member under crates/"

# 4. Zero inspected tracked .md or .rs files (exit 2)
repo_no_inspected=$(new_valid_repo no_inspected)
git -C "$repo_no_inspected" rm -rf -q '*.md' crates/probe/src/lib.rs
git -C "$repo_no_inspected" commit -qm 'remove all md and rs files'
expect_fail_env "Cargo.toml deny.toml" "$repo_no_inspected" 2 "cannot judge: inspected 0 files"

# 5. Stale prose path reference (exit 1)
repo_stale_prose=$(new_valid_repo stale_prose)
printf '# README\nSee docs/nonexistent.md for details.\n' >"$repo_stale_prose/README.md"
git -C "$repo_stale_prose" add README.md
git -C "$repo_stale_prose" commit -qm 'add stale prose reference'
expect_fail "$repo_stale_prose" 1 "references 'docs/nonexistent.md', which is not tracked in this repository"

# 6. Stale markdown link target (exit 1)
repo_stale_link=$(new_valid_repo stale_link)
printf '# README\nSee [missing link](docs/missing.md).\n' >"$repo_stale_link/README.md"
git -C "$repo_stale_link" add README.md
git -C "$repo_stale_link" commit -qm 'add stale markdown link'
expect_fail "$repo_stale_link" 1 "references 'docs/missing.md', which is not tracked in this repository"

# 7. Stale tests/ reference in member crate (exit 1)
repo_stale_test=$(new_valid_repo stale_test)
printf '// See tests/nonexistent_test.rs\n' >"$repo_stale_test/crates/probe/src/lib.rs"
git -C "$repo_stale_test" add crates/probe/src/lib.rs
git -C "$repo_stale_test" commit -qm 'add stale tests reference'
expect_fail "$repo_stale_test" 1 "references 'tests/nonexistent_test.rs', which is tracked under no workspace member"

# 8. Valid ignored path reference (exit 0)
repo_ignored=$(new_valid_repo ignored_ref)
printf 'ignored_dir/\n' >"$repo_ignored/.gitignore"
printf '# README\nReferencing ignored_dir/sample.txt here.\n' >"$repo_ignored/README.md"
git -C "$repo_ignored" add .gitignore README.md
git -C "$repo_ignored" commit -qm 'add ignored reference'
expect_pass "$repo_ignored" "reference integrity ok"

# 9. OpenSpec change directory exemption (exit 0)
repo_openspec=$(new_valid_repo openspec_exempt)
mkdir -p "$repo_openspec/openspec/changes/my-change"
printf '# Proposal\nWill create docs/future.md.\n' >"$repo_openspec/openspec/changes/my-change/proposal.md"
git -C "$repo_openspec" add openspec/changes/my-change/proposal.md
git -C "$repo_openspec" commit -qm 'add openspec change proposal'
expect_pass "$repo_openspec" "reference integrity ok"

echo "all reference integrity test matrix directions passed"
