#!/usr/bin/env bash
#
# Every state and failure direction of `check_publish_source.sh`, each on a throwaway repository.
#
# A gate on an irreversible act has one chance to be right: by the time a wrong `.cargo_vcs_info.json`
# is on crates.io, no correction exists. So each refusal is proven here against a fixture built to
# trip exactly that one condition, and the pass path is proven too — a gate that only ever refuses
# would be discovered on release day.
#
# The fixtures sign with an ephemeral key generated per run, so the signed-tag path is exercised
# without depending on a maintainer's key being present (CI has none).
set -euo pipefail

script_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
check=$script_dir/check_publish_source.sh
version=0.4.0

command -v ssh-keygen >/dev/null 2>&1 \
    || { printf 'cannot judge: ssh-keygen is unavailable, so the signed-tag fixtures cannot be built\n' >&2; exit 2; }

fixture_root=$(mktemp -d)
trap 'rm -rf "$fixture_root"' EXIT

signing_key=$fixture_root/signing
ssh-keygen -q -t ed25519 -N '' -C 'publish-source-test' -f "$signing_key"

write_workspace() {
    local repo=$1 workspace_version=$2
    mkdir -p "$repo/crates/xuanji"
    printf '%s\n' \
        '[workspace]' \
        'members = ["crates/xuanji"]' \
        '' \
        '[workspace.package]' \
        "version = \"$workspace_version\"" \
        >"$repo/Cargo.toml"
    printf '%s\n' \
        '[package]' \
        'name = "xuanji"' \
        'version.workspace = true' \
        'edition = "2024"' \
        >"$repo/crates/xuanji/Cargo.toml"
}

# A repository in the exact shape a publish runs from: `main` pushed to a remote, its tip a
# `release: X.Y.Z` snapshot, tagged with a signed annotated tag, worktree clean.
new_release_repo() {
    local name=$1 repo origin
    repo=$fixture_root/$name
    origin=$fixture_root/$name-origin.git
    git init -q --bare "$origin"
    mkdir -p "$repo"
    git init -q -b main "$repo"
    git -C "$repo" config user.name 'Publish Source Test'
    git -C "$repo" config user.email 'publish-source@example.invalid'
    git -C "$repo" config gpg.format ssh
    git -C "$repo" config user.signingkey "$signing_key.pub"
    # Pinned, not inherited: a maintainer's global `tag.gpgSign = true` turns the lightweight-tag
    # fixture below into an annotated one, which then fails to build at all ("no tag message?"), so the
    # refusal that case exists to prove would never be reached — and it would pass on CI, which has no
    # such global. Signing is asked for explicitly with `-s` wherever a fixture wants it.
    git -C "$repo" config commit.gpgsign false
    git -C "$repo" config tag.gpgsign false
    git -C "$repo" remote add origin "$origin"

    write_workspace "$repo" "$version"
    git -C "$repo" add .
    git -C "$repo" commit -qm 'chore: groundwork'
    printf '%s\n' '# notes' >"$repo/NOTES.md"
    git -C "$repo" add .
    git -C "$repo" commit -qm "release: $version"
    git -C "$repo" tag -s "v$version" -m "release: $version"
    git -C "$repo" push -q origin main
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

# The expected exit CODE is asserted, not merely non-zero: the family's contract separates a wrong
# publish source (1) from a gate that cannot decide (2), and a gate that collapsed the two would
# report a misconfiguration as a clean refusal.
expect_fail() {
    local repo=$1 expected_status=$2 expected=$3 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] \
        || { printf 'expected exit %d containing %q, got exit %d: %s\n' "$expected_status" "$expected" "$status" "$output" >&2; exit 1; }
    grep -Fq "$expected" <<<"$output" \
        || { printf 'expected exit %d containing %q, got: %s\n' "$expected_status" "$expected" "$output" >&2; exit 1; }
}

publishable=$(new_release_repo publishable)
expect_pass "$publishable" "ok publish source"

modified=$(new_release_repo modified)
printf '%s\n' '# edited' >"$modified/NOTES.md"
expect_fail "$modified" 1 'worktree is not clean'

untracked=$(new_release_repo untracked)
printf '%s\n' 'stray' >"$untracked/STRAY.md"
expect_fail "$untracked" 1 'worktree is not clean'

# The 0.4.0 shape itself: a commit sitting on top of the release snapshot — a release branch's tip,
# whose tree may be identical and whose commit is not the one `main` released.
past_snapshot=$(new_release_repo past-snapshot)
printf '%s\n' '# prepared' >"$past_snapshot/PREPARE.md"
git -C "$past_snapshot" add .
git -C "$past_snapshot" commit -qm 'chore(release): prepare 0.4.0'
git -C "$past_snapshot" push -q origin main
expect_fail "$past_snapshot" 1 'HEAD is not this version'

untagged=$(new_release_repo untagged)
git -C "$untagged" tag -d "v$version" >/dev/null
expect_fail "$untagged" 1 "there is no tag v$version"

lightweight=$(new_release_repo lightweight)
git -C "$lightweight" tag -d "v$version" >/dev/null
git -C "$lightweight" tag "v$version"
expect_fail "$lightweight" 1 'is a lightweight tag'

unsigned=$(new_release_repo unsigned)
git -C "$unsigned" tag -d "v$version" >/dev/null
git -C "$unsigned" tag -a "v$version" -m "release: $version"
expect_fail "$unsigned" 1 'carries no signature'

# Tag and HEAD both plausible on their own, naming different commits. Caught before the remote check,
# so the diagnostic names the tag rather than the branch tip.
tag_elsewhere=$(new_release_repo tag-elsewhere)
git -C "$tag_elsewhere" tag -d "v$version" >/dev/null
git -C "$tag_elsewhere" tag -s "v$version" -m "release: $version" HEAD~1
expect_fail "$tag_elsewhere" 1 'but HEAD is'

# HEAD is the tagged release snapshot, yet the protected branch has moved on. Read live from the
# remote, so a stale `refs/remotes/origin/main` cannot make this pass.
behind_remote=$(new_release_repo behind-remote)
git -C "$behind_remote" push -q --force origin 'HEAD~1:refs/heads/main'
expect_fail "$behind_remote" 1 'is not the tip of origin/main'

unreachable_remote=$(new_release_repo unreachable-remote)
git -C "$unreachable_remote" remote set-url origin "$fixture_root/absent-origin.git"
expect_fail "$unreachable_remote" 2 'could not read refs/heads/main'

malformed_version=$(new_release_repo malformed-version)
write_workspace "$malformed_version" 'not-a-version'
git -C "$malformed_version" add .
git -C "$malformed_version" commit -qm 'chore: malform the version'
git -C "$malformed_version" push -q origin main
expect_fail "$malformed_version" 2 'workspace version is missing or malformed'

no_manifest=$fixture_root/no-manifest
mkdir -p "$no_manifest"
git init -q -b main "$no_manifest"
expect_fail "$no_manifest" 2 'has no Cargo.toml'

no_git=$fixture_root/no-git
mkdir -p "$no_git"
write_workspace "$no_git" "$version"
expect_fail "$no_git" 2 'is not a git worktree'

# The wrapper's own refusal. `publish.sh` resolves its own repository root — deliberately, since it must
# judge THIS tree — so it cannot be pointed at a fixture the way the gate can; this runs it against the
# real checkout and asserts the ARGUMENT refusal by its message. That distinction is load-bearing: a
# development checkout also fails the gate (its HEAD is no release snapshot), and both refusals exit 1,
# so a status-only assertion would pass with the guard deleted. Exit 1 also proves cargo was never
# reached, `exec cargo publish` being the only other way this script can terminate.
for relocate in --manifest-path '--manifest-path=/elsewhere/Cargo.toml'; do
    wrapper_status=0
    wrapper_output=$(bash "$script_dir/publish.sh" "$relocate" /elsewhere/Cargo.toml 2>&1) || wrapper_status=$?
    [[ $wrapper_status -eq 1 ]] \
        || { printf 'publish.sh must refuse %q with exit 1, got exit %d: %s\n' "$relocate" "$wrapper_status" "$wrapper_output" >&2; exit 1; }
    grep -Fq "moves cargo's workspace root away from the tree this gate judges" <<<"$wrapper_output" \
        || { printf 'publish.sh must refuse %q as an argument, not merely fail the gate: %s\n' "$relocate" "$wrapper_output" >&2; exit 1; }
done

# Read-only, on a fixture this gate has NOT already judged. Capturing `before` from a repository the gate had
# run over several times was blind by construction: a gate that writes the same file on every run leaves that
# file in `before` too, so the comparison held. Measured, not reasoned — a stray write injected into a sibling
# gate passed its read-only direction unnoticed until the fixture was made fresh.
untouched=$(new_release_repo untouched)
before_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
before_head=$(git -C "$untouched" rev-parse HEAD)
before_tags=$(git -C "$untouched" tag --list)
"$check" "$untouched" >/dev/null
after_tree=$(git -C "$untouched" status --porcelain=v1 --untracked-files=all)
after_head=$(git -C "$untouched" rev-parse HEAD)
after_tags=$(git -C "$untouched" tag --list)
[[ $before_tree == "$after_tree" && $before_head == "$after_head" && $before_tags == "$after_tags" ]] \
    || { printf 'publish source check mutated repository state\n' >&2; exit 1; }

# The same contract direction on the gate that stands before an irreversible act. Measured before the
# backstop: with `git status` stubbed to fail after the worktree check passed, this gate exited **131** with
# no output — a status the header's own contract does not define.
contract_stub=$fixture_root/contract-stub
mkdir -p "$contract_stub"
cat >"$contract_stub/git" <<STUB
#!/usr/bin/env bash
for arg in "\$@"; do
    [[ \$arg == status ]] && exit 131
done
exec "$(command -v git)" "\$@"
STUB
chmod +x "$contract_stub/git"

contract_status=0
contract_output=$(PATH="$contract_stub:$PATH" "$check" "$publishable" 2>&1) || contract_status=$?
[[ $contract_status -eq 2 ]] \
    || { printf 'an unhandled failure must exit 2, not the tool status, got %d: %s\n' "$contract_status" "$contract_output" >&2; exit 1; }
grep -Fq 'an unhandled command failed' <<<"$contract_output" \
    || { printf 'an unhandled failure must say so and name where, got: %s\n' "$contract_output" >&2; exit 1; }

# A clean run must print NOTHING on stderr. What this replaces grepped for the backstop's own
# `an unhandled command failed`, so any *other* line a gate printed on a clean run while exiting 0 still read
# as clean — and a matrix that names one diagnostic has to track that diagnostic's wording. Emptiness has no
# wording to keep in step. `test_whitespace_hygiene.sh` documents the `errtrace` misfire the property descends
# from and is the matrix whose clean run actually exercises it.
clean_stderr=$("$check" "$publishable" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] \
    || { printf 'a clean run must print nothing on stderr, got: %s\n' "$clean_stderr" >&2; exit 1; }

printf 'ok publish source state and failure matrix\n'
