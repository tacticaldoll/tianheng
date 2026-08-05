#!/usr/bin/env bash
#
# The pre-publish gate: `cargo publish` may only run from the tagged `main` release commit.
#
# Why a reaction rather than a reminder. `cargo publish` stamps `.cargo_vcs_info.json` inside every
# tarball with the sha1 of whatever `HEAD` it ran on, and a published version can never be
# re-uploaded — so that pointer is permanent from the moment it lands. The 0.4.0 family carries
# `f1dba52`, the tip of `release/0.4.0`, instead of `e645a549`, the `release: 0.4.0` commit on `main`
# that `v0.4.0` tags. Nothing about the shipped content is wrong (the two trees are byte-identical,
# same tree hash, every shipped file matching), which is exactly what makes this class so easy to
# miss and impossible to correct afterwards: cargo records the COMMIT, not the content, and an
# identical tree does not save you. The commit it recorded is a staging branch's tip, and the release
# ritual archives that branch once it squash-merges — after which the published artifact's only
# provenance pointer names nothing reachable. Eleven 0.1.x releases and 0.2.2 already sit in that
# state.
#
# "Publish `main`, not the release branch" had already been said once, in that same release window,
# before 0.4.0 went out from the release branch anyway. A rule that has been stated and then missed
# is the definition of one that needs a reaction instead of another sentence.
#
# What it asserts, all of committed state, none of it about content:
#   * the worktree is clean, so `HEAD` describes what would be packaged;
#   * `HEAD` is a `release: X.Y.Z` snapshot commit whose version is the workspace version;
#   * `vX.Y.Z` exists, is an annotated tag, carries a signature, and points at `HEAD`;
#   * `HEAD` is the tip of the remote's `main` — the protected branch, read live, never from a
#     possibly-stale `refs/remotes/`.
#
# One bound, stated rather than implied: the signature check asserts that the tag object CARRIES a
# signature, not that the signature verifies. Verification needs an allowed-signers configuration that
# exists on a maintainer's machine and not in CI, so requiring it would make the same tag judged
# differently by where the gate ran. Cryptographic validity is GitHub's verified badge and `git tag -v`;
# this gate refuses the lightweight and unsigned shapes, which are the ones a hurried release produces.
#
# Exit 0 publishable, 1 wrong source, 2 cannot judge — the family's own Core Contract, so this script
# reads the same way as the reactions it sits beside. It is read-only: it never fetches, commits,
# tags, or publishes.
set -euo pipefail

repo=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}
remote=${2:-origin}

fail() {
    printf 'publish source: %s\n' "$*" >&2
    exit 1
}

cannot_judge() {
    printf 'publish source: cannot judge: %s\n' "$*" >&2
    exit 2
}

read_workspace_version() {
    awk '
        /^\[workspace\.package\]$/ { in_package = 1; next }
        /^\[/ { in_package = 0 }
        in_package && /^[[:space:]]*version[[:space:]]*=/ {
            if (match($0, /"[^"]+"/)) {
                print substr($0, RSTART + 1, RLENGTH - 2)
                exit
            }
        }
    ' "$repo/Cargo.toml"
}

[[ -f $repo/Cargo.toml ]] || cannot_judge "repository root $repo has no Cargo.toml"
git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || cannot_judge "repository root $repo is not a git worktree"

workspace_version=$(read_workspace_version)
[[ $workspace_version =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
    || cannot_judge "workspace version is missing or malformed: ${workspace_version:-<missing>}"

tag=v$workspace_version

# Cleanliness first: every check below reads committed state, and `cargo publish` would package the
# worktree. A dirty tree makes `HEAD` a description of something other than what would ship.
worktree_state=$(git -C "$repo" status --porcelain=v1 --untracked-files=all)
[[ -z $worktree_state ]] \
    || fail "worktree is not clean, so HEAD does not describe what would be packaged:
$(sed 's/^/         /' <<<"$worktree_state")"

head_commit=$(git -C "$repo" rev-parse HEAD)
head_subject=$(git -C "$repo" log -1 --format='%s' HEAD)

[[ $head_subject == "release: $workspace_version" ]] \
    || fail "HEAD is not this version's release snapshot: its subject is \"$head_subject\", expected \"release: $workspace_version\". Publish the \`release: X.Y.Z\` commit on $remote/main, never a release branch's tip — cargo stamps the commit it ran on into every tarball, permanently"

git -C "$repo" rev-parse --verify "refs/tags/$tag" >/dev/null 2>&1 \
    || fail "there is no tag $tag; the release snapshot is tagged before it is published"

[[ $(git -C "$repo" cat-file -t "refs/tags/$tag") == tag ]] \
    || fail "$tag is a lightweight tag; the release tags are annotated (\`git tag -s\`)"

git -C "$repo" cat-file tag "refs/tags/$tag" | grep -q '^-----BEGIN .* SIGNATURE-----$' \
    || fail "$tag carries no signature; the release tags are signed (\`git tag -s\`)"

tag_commit=$(git -C "$repo" rev-list -n 1 "$tag")
[[ $tag_commit == "$head_commit" ]] \
    || fail "$tag points at $tag_commit but HEAD is $head_commit; publish the commit the tag names"

# The protected branch is read LIVE. A `refs/remotes/$remote/main` left behind by the last fetch
# would let a local branch that merely contains an identical tree pass as the released snapshot —
# which is the shape this gate exists to refuse.
remote_main=$(git -C "$repo" ls-remote "$remote" refs/heads/main 2>/dev/null | awk 'NR == 1 { print $1 }') \
    || remote_main=
[[ -n $remote_main ]] \
    || cannot_judge "could not read refs/heads/main from remote \"$remote\", so whether HEAD is the released snapshot cannot be decided (never a silent pass)"
[[ $remote_main == "$head_commit" ]] \
    || fail "HEAD $head_commit is not the tip of $remote/main ($remote_main); \`main\` is the release-only branch every publish comes from"

printf 'ok publish source (%s at %s, tagged %s)\n' "$remote/main" "$head_commit" "$tag"
