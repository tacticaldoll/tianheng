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
#   * `vX.Y.Z` exists, is an annotated tag, carries a signature that VERIFIES, and points at `HEAD`;
#   * `HEAD` is the tip of the remote's `main` — the protected branch, read live, never from a
#     possibly-stale `refs/remotes/`.
#
# One bound, stated rather than implied: the signature check asserts that the signature is cryptographically
# VALID over the tag object, not that its signer is AUTHORIZED. Attribution needs an allowed-signers
# configuration that exists on a maintainer's machine and not in CI, so requiring it would make the same tag
# judged differently by where the gate ran. Validity needs no configuration at all.
#
# That sentence used to say the opposite — that verification itself needs allowed-signers, so this gate could
# only match a shape. It was wrong, and being wrong is why the gate accepted an unsigned tag whose message
# quoted a signature block for as long as it did: a stated cause is what a later author reasons from, and this
# one said "you cannot check this", so nobody did. Measured, `ssh-keygen -Y check-novalidate` verifies validity
# with no configuration; only attribution needs it. Corrected here rather than only in the specification,
# because this is where the next author reads.
#
# Exit 0 publishable, 1 wrong source, 2 cannot judge — the family's own Core Contract, so this script
# reads the same way as the reactions it sits beside. It is read-only: it never fetches, commits,
# tags, or publishes.
set -Eeuo pipefail
# The family's exit contract as a backstop — see `scripts/lib/exit_contract.sh` for what it catches, why it
# is a trap rather than per-command handling, and the measurements behind both.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
exit_contract_backstop 'publish source'

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

# The signature is VERIFIED, not shape-matched. What this replaced grepped the whole tag object — message
# included — so an annotated-but-unsigned tag whose message quoted a signature block satisfied it. Measured on a
# fixture: the grep passed while `git verify-tag` reported `Couldn't decode signature: invalid format`. That is a
# silent pass on the one path with no correction afterwards.
#
# Four mechanisms were measured with NO allowed-signers file anywhere, and three fail:
#   * this grep — accepts the quoted block;
#   * `%(contents:signature)` merely non-empty — also accepts it, returning the quoted text, so the extraction is
#     right and the assertion is not;
#   * `git verify-tag` — exit 1 with an identical `allowedSignersFile needs to be configured` for a genuinely
#     signed tag AND an unsigned one, `--raw` the same, so a gate built on it always reports cannot-judge in CI:
#     the check disabled while looking strengthened.
# `ssh-keygen -Y check-novalidate` accepts the signed fixture and refuses the quoted one with no configuration.
command -v ssh-keygen >/dev/null 2>&1 \
    || cannot_judge "ssh-keygen is unavailable, so $tag's signature cannot be verified"

signature_dir=$(mktemp -d)
trap 'rm -rf "$signature_dir"' EXIT

# The mechanism is exercised on a signature of its own making BEFORE its verdict on the tag is trusted, because
# `ssh-keygen`'s exit status cannot say whether a check failed or the tool did. Measured: an invalid signature
# exits 255 and an unreadable signature file also exits 255, while bad arguments exit 1 — so a non-zero is
# ambiguous unless the tool is already known to work. Without this round-trip a broken or stubbed `ssh-keygen`
# reports the tag as a wrong source: a 1-versus-2 collapse telling a maintainer their tag is bad when the tool is,
# on the one path with no correction afterwards. Found by this gate's own twin, not by reading.
ssh-keygen -q -t ed25519 -N '' -f "$signature_dir/probe" >/dev/null 2>&1 \
    && printf 'probe' | ssh-keygen -Y sign -n git -f "$signature_dir/probe" >"$signature_dir/probe.sig" 2>/dev/null \
    && printf 'probe' | ssh-keygen -Y check-novalidate -n git -s "$signature_dir/probe.sig" >/dev/null 2>&1 \
    || cannot_judge "the signature mechanism failed its own round-trip, so no verdict on $tag's signature would be reliable"

tag_object=$(git -C "$repo" cat-file tag "refs/tags/$tag") \
    || cannot_judge "could not read the tag object for $tag"
tag_signature=$(git -C "$repo" for-each-ref --format='%(contents:signature)' "refs/tags/$tag") \
    || cannot_judge "could not read $tag's signature block"

[[ -n $tag_signature ]] \
    || fail "$tag carries no signature; the release tags are signed (\`git tag -s\`)"

# The KIND is read from the block's own armor header rather than from a tool's diagnostic text. A signature this
# mechanism cannot read is cannot-judge, never a wrong source: a false refusal before an irreversible act costs a
# release, and this family signs with SSH while a GPG-signed tag stays expressible.
[[ $tag_signature == '-----BEGIN SSH SIGNATURE-----'* ]] \
    || cannot_judge "$tag carries a signature this gate cannot verify — it reads SSH signatures, and this block begins ${tag_signature%%$'\n'*}"

[[ $tag_object == *"$tag_signature" ]] \
    || cannot_judge "$tag's extracted signature is not the tag object's suffix, so the signed payload cannot be reconstructed reliably"

# The signed payload is the tag object with the signature block removed as a SUFFIX, never by stripping from the
# first line that resembles a signature header. Measured on a genuinely signed tag whose message ALSO quotes a
# verification log: stripping from the first such line truncates the payload and refuses a real signature — a
# false refusal introduced by the hardening itself. Suffix removal keeps a quoted block inside the payload.
printf '%s\n' "$tag_signature" >"$signature_dir/tag.sig"
printf '%s' "${tag_object%"$tag_signature"}" \
    | ssh-keygen -Y check-novalidate -n git -s "$signature_dir/tag.sig" >/dev/null 2>&1 \
    || fail "$tag's signature does not verify over the tag object; a signature block quoted in a tag message is text, not a signature"

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
