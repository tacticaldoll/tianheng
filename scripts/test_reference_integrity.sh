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
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

expect_fail_fixture_policy() {
    local gov_docs=$1 repo=$2 expected_status=$3 expected=$4 output status=0
    output=$("$check" "$repo" --fixture-governance-documents "$gov_docs" 2>&1) || status=$?
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

# Ambient state cannot weaken the required set used for a real run.
poisoned_status=0
poisoned_output=$(GOVERNANCE_DOCUMENTS='Cargo.toml deny.toml' "$check" "$repo_missing_gov" 2>&1) \
    || poisoned_status=$?
[[ $poisoned_status -eq 2 && $poisoned_output == *"'PROJECT.md' is named"* ]] \
    || { printf 'ambient GOVERNANCE_DOCUMENTS changed policy: exit %d: %s\n' "$poisoned_status" "$poisoned_output" >&2; exit 1; }

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
expect_fail_fixture_policy "Cargo.toml deny.toml" "$repo_no_inspected" 2 "cannot judge: inspected 0 files"

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

# 10. The reference extraction cannot fail quietly (exit 2)
#
# The normalization ran inside a process substitution, where a failing `sed` or `sort` reports nothing
# to the parent: `set -o pipefail` covers a pipeline the shell runs, not one in a subshell whose status
# no one reads. The stream came back empty, the loop body never ran, and every reference in that file
# went unexamined while the file still counted as inspected.
#
# The fixture is built on the STALE-PROSE repository deliberately, so the direction is discriminating
# rather than merely loud: this repository must refuse. With the extraction unchecked the run reports
# `reference integrity ok` over a reference it never read — a silent pass on a tree known to be bad.
#
# `sed` is stubbed to fail only for the extraction's own expression, not unconditionally: `is_tracked`
# escapes each reference with `sed` too, and a blanket failure would refuse somewhere else and prove a
# different thing.
repo_broken_extraction=$(new_valid_repo broken_extraction)
printf '# PROJECT\nSee [docs/nonexistent.md](docs/nonexistent.md).\n' >"$repo_broken_extraction/PROJECT.md"
git -C "$repo_broken_extraction" add PROJECT.md
git -C "$repo_broken_extraction" commit -qm 'add stale prose reference'
expect_fail "$repo_broken_extraction" 1 "references 'docs/nonexistent.md', which is not tracked in this repository"

stub_bin=$fixture_root/stub-bin
mkdir -p "$stub_bin"
real_sed=$(command -v sed)
cat >"$stub_bin/sed" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    case \$arg in
    *'s#^\]\\('*) exit 3 ;;
    esac
done
exec "$real_sed" "\$@"
STUB
chmod +x "$stub_bin/sed"

broken_status=0
broken_output=$(PATH="$stub_bin:$PATH" "$check" "$repo_broken_extraction" 2>&1) || broken_status=$?
[[ $broken_status -eq 2 ]] \
    || { printf 'a failed reference extraction must exit 2, got %d: %s\n' "$broken_status" "$broken_output" >&2; exit 1; }
grep -Fq 'could not normalize the references extracted from' <<<"$broken_output" \
    || { printf 'a failed reference extraction must name itself, got: %s\n' "$broken_output" >&2; exit 1; }

# 11. The tracked-path index cannot fail silently or with a foreign status (exit 2)
#
# This is the FIRST enumeration and the one every `is_tracked` answer below reads. Measured before the
# repair: with `git ls-files` stubbed to fail, the gate exited **3** — git's own status — and printed nothing,
# which the contract does not define and an operator cannot read. Asserted on the exit CODE, since the wrong
# answer here is also non-zero.
index_git_stub=$fixture_root/index-git-stub
mkdir -p "$index_git_stub"
real_git_bin=$(command -v git)
cat >"$index_git_stub/git" <<STUB
#!/usr/bin/env bash
[[ \$1 == ls-files && \$# -eq 1 ]] && exit 3
exec "$real_git_bin" "\$@"
STUB
chmod +x "$index_git_stub/git"

index_status=0
index_output=$(PATH="$index_git_stub:$PATH" "$check" "$repo_clean" 2>&1) || index_status=$?
[[ $index_status -eq 2 ]] \
    || { printf 'a failed tracked-path index must exit 2, got %d: %s\n' "$index_status" "$index_output" >&2; exit 1; }
grep -Fq 'could not build the tracked-path index' <<<"$index_output" \
    || { printf 'a failed tracked-path index must name itself, got: %s\n' "$index_output" >&2; exit 1; }

# 12. And any unhandled failure at all, which is what the ERR trap is for: the sites nobody wrapped. `mktemp`
# is unwrapped and runs before anything else, so it is the honest injection point.
mktemp_stub=$fixture_root/mktemp-stub
mkdir -p "$mktemp_stub"
printf '#!/usr/bin/env bash\nexit 7\n' >"$mktemp_stub/mktemp"
chmod +x "$mktemp_stub/mktemp"

unhandled_status=0
unhandled_output=$(PATH="$mktemp_stub:$PATH" "$check" "$repo_clean" 2>&1) || unhandled_status=$?
[[ $unhandled_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 2, not the utility status, got %d: %s\n' "$unhandled_status" "$unhandled_output" >&2; exit 1; }
grep -Fq 'an unhandled command failed' <<<"$unhandled_output" \
    || { printf 'an unhandled failure must say so and name where, got: %s\n' "$unhandled_output" >&2; exit 1; }

# 13. Read-only, like every gate in the family. This was the one gate twin with no such direction while five
# siblings had one — and the gate reads a tracked-path index into a temporary file, so "it only reads" was an
# assumption about where that file lands rather than an observation. A gate that edits what it judges makes
# its own next verdict unreproducible.
#
# On a fixture this gate has NOT already judged. Written first against `$repo_clean`, and a stray write
# injected into the gate passed unnoticed: the gate had already run over that repository several times, so a
# file it writes on every run sits in `before` as well as in `after`. Every sibling twin carried the same
# blindness and was corrected with it.
untouched=$(new_valid_repo untouched)
before_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$untouched" rev-parse HEAD)
"$check" "$untouched" >/dev/null
[[ $(git -C "$untouched" status --porcelain=v1 --untracked-files=all) == "$before_tree" \
    && $(git -C "$untouched" rev-parse HEAD) == "$before_head" ]] \
    || { printf 'reference integrity check mutated repository state\n' >&2; exit 1; }

# A clean run must print NOTHING on stderr. What this replaces grepped for the backstop's own
# `an unhandled command failed`, so any *other* line a gate printed on a clean run while exiting 0 still read
# as clean — and a matrix that names one diagnostic has to track that diagnostic's wording. Emptiness has no
# wording to keep in step. `test_whitespace_hygiene.sh` documents the `errtrace` misfire the property descends
# from and is the matrix whose clean run actually exercises it.
clean_stderr=$("$check" "$repo_clean" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

echo "all reference integrity test matrix directions passed"
