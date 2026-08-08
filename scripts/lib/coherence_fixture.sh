# shellcheck shell=bash
#
# One builder for a repository in the shape `check_release_coherence.sh` judges, shared by
# `scripts/test_release_coherence.sh` and by the Rust tests that cite this capability's declared bounds.
#
# Why shared rather than copied, for the same reason `scripts/lib/release_fixture.sh` exists: `PINNED-BY`
# resolves only a harness-registered Rust function, so a bound belonging to a shell gate can be defended by a
# twin direction and cited by nothing. Pinning it from Rust means a second construction of "a repository with a
# changelog and some machinery" — and two constructions of one fixture is the twin-drift class this repository
# keeps closing. So there is one, here, and both callers use it.
#
# It is a library: sourced, never executed. The caller owns the temporary root and its cleanup, because a
# fixture builder that also decided lifetime would make the twin's single `trap` two.

# A workspace whose version is $2, written into $1, carrying the example pin `require_example_pins` reads —
# without one, that check's vacuity guard fires and every direction reports the missing-examples failure
# instead of what it is testing.
coherence_fixture_workspace() {
    local repo=$1 version=$2 package
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

# A changelog whose latest section is the dated release $2, previous $3.
coherence_fixture_release_changelog() {
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

# A changelog in development state; $3 `no` omits the single `[Unreleased]` item.
coherence_fixture_development_changelog() {
    local repo=$1 version=$2 with_item=${3:-yes}
    {
        printf '%s\n' '# Changelog' '' '## [Unreleased]' ''
        if [[ $with_item == yes ]]; then
            printf '%s\n' '- An adopter-facing change.' ''
        fi
        printf '%s\n' "[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v$version...HEAD"
    } >"$repo/CHANGELOG.md"
}

# A tracked file under `scripts/`, so the fixture has machinery for the gate's enumerator to find. Written but
# never added, when the caller wants the index to disagree with the worktree.
coherence_fixture_machinery() {
    mkdir -p "$1/scripts"
    printf '#!/usr/bin/env bash\nexit 0\n' >"$1/scripts/check_pin_bites.sh"
}

# Replace `[Unreleased]`'s single item with $2, so a caller can place a citation under a heading of its
# choosing.
coherence_fixture_unreleased_body() {
    python3 - "$1/CHANGELOG.md" "$2" <<'EDIT'
import pathlib, sys
p = pathlib.Path(sys.argv[1])
p.write_text(p.read_text().replace("- An adopter-facing change.\n", sys.argv[2] + "\n"))
EDIT
}

coherence_fixture_commit() {
    local repo=$1 subject=$2
    git -C "$repo" add .
    git -C "$repo" commit -qm "$subject"
}

# A repository under $1 named $2, released at ${3:-0.2.0} over a 0.1.0 predecessor. Prints its path.
coherence_fixture_repo() {
    local root=$1 name=$2 version=${3:-0.2.0} repo
    repo=$root/$name
    mkdir -p "$repo"
    git -C "$repo" init -q
    git -C "$repo" config user.name 'Release Coherence Test'
    git -C "$repo" config user.email 'release-coherence@example.invalid'
    coherence_fixture_workspace "$repo" 0.1.0
    coherence_fixture_release_changelog "$repo" 0.1.0 0.0.0
    git -C "$repo" add .
    git -C "$repo" commit -qm 'release: 0.1.0'
    coherence_fixture_workspace "$repo" "$version"
    coherence_fixture_release_changelog "$repo" "$version" 0.1.0
    git -C "$repo" add .
    git -C "$repo" commit -qm "release: $version"
    printf '%s\n' "$repo"
}
