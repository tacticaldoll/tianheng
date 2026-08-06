# shellcheck shell=bash
#
# One builder for a repository in the exact shape a publish runs from, shared by
# `scripts/test_publish_source.sh` and by the Rust test that pins this capability's declared bound.
#
# Why shared rather than copied. `PINNED-BY` resolves only a harness-registered Rust function, so a bound
# belonging to a shell gate can be defended by a twin direction and cited by nothing. Pinning it from Rust means
# a second construction of "a signed release repository" — and two constructions of one fixture is the twin-drift
# class this repository keeps closing. So there is one, here, and both callers use it.
#
# It is a library: sourced, never executed. The caller owns the temporary root and its cleanup, because a
# fixture builder that also decided lifetime would make the twin's single `trap` two.

# A minimal workspace whose version is $2, written into $1.
release_fixture_workspace() {
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

# `main` pushed to a bare remote, its tip a `release: $3` snapshot, tagged with a signed annotated tag, worktree
# clean. $1 is the directory to build under, $2 the fixture name, $3 the version, $4 the signing key's path (its
# `.pub` is the configured signer). Prints the repository path.
release_fixture_repo() {
    local root=$1 name=$2 version=$3 signing_key=$4
    local repo=$root/$name
    local origin=$root/$name-origin.git
    git init -q --bare "$origin"
    mkdir -p "$repo"
    git init -q -b main "$repo"
    git -C "$repo" config user.name 'Publish Source Test'
    git -C "$repo" config user.email 'publish-source@example.invalid'
    git -C "$repo" config gpg.format ssh
    git -C "$repo" config user.signingkey "$signing_key.pub"
    # Pinned, not inherited: a maintainer's global `tag.gpgSign = true` turns a lightweight-tag fixture into an
    # annotated one, which then fails to build at all ("no tag message?"), so the refusal that case exists to
    # prove would never be reached — and it would pass on CI, which has no such global. Signing is asked for
    # explicitly with `-s` wherever a fixture wants it.
    git -C "$repo" config commit.gpgsign false
    git -C "$repo" config tag.gpgsign false
    git -C "$repo" remote add origin "$origin"

    release_fixture_workspace "$repo" "$version"
    git -C "$repo" add .
    git -C "$repo" commit -qm 'chore: groundwork'
    printf '%s\n' '# notes' >"$repo/NOTES.md"
    git -C "$repo" add .
    git -C "$repo" commit -qm "release: $version"
    git -C "$repo" tag -s "v$version" -m "release: $version"
    git -C "$repo" push -q origin main
    printf '%s\n' "$repo"
}
