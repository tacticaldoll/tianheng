#!/usr/bin/env bash
# Exit 0 coherent, 1 incoherent, 2 cannot judge — the family's own Core Contract, stated here because the
# gate now holds it on every path and a contract a reader cannot find is one they cannot rely on.
set -Eeuo pipefail
# The family's exit contract as a backstop — see `scripts/lib/exit_contract.sh` for what it catches, why it
# is a trap rather than per-command handling, and the measurements behind both.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
exit_contract_backstop 'release coherence'

repo=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}

fail() {
    printf 'release coherence: %s\n' "$*" >&2
    return 1
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

version_order() {
    local left=$1 right=$2 left_part right_part index
    IFS=. read -r -a left_parts <<<"$left"
    IFS=. read -r -a right_parts <<<"$right"
    for index in 0 1 2; do
        left_part=${left_parts[$index]}
        right_part=${right_parts[$index]}
        if ((${#left_part} < ${#right_part})) \
            || { ((${#left_part} == ${#right_part})) && [[ $left_part < $right_part ]]; }; then
            printf '%s\n' -1
            return
        fi
        if ((${#left_part} > ${#right_part})) \
            || { ((${#left_part} == ${#right_part})) && [[ $left_part > $right_part ]]; }; then
            printf '%s\n' 1
            return
        fi
    done
    printf '%s\n' 0
}

changelog_unreleased_has_item() {
    awk '
        /^## \[Unreleased\]/ { in_unreleased = 1; next }
        in_unreleased && /^## \[/ { exit }
        in_unreleased && /^[[:space:]]*-[[:space:]]+/ { found = 1 }
        END { exit !found }
    ' "$repo/CHANGELOG.md"
}

changelog_unreleased_is_empty() {
    ! changelog_unreleased_has_item
}

require_workspace_manifests() {
    local manifest package_name
    for manifest in "${workspace_manifest_files[@]}"; do
        package_name=$(awk -F '"' '/^[[:space:]]*name[[:space:]]*=/ { print $2; exit }' "$manifest")
        grep -Eq '^[[:space:]]*version\.workspace[[:space:]]*=[[:space:]]*true([[:space:]]*(#.*)?)?$' "$manifest" \
            || fail "workspace package ${package_name:-$manifest} must inherit version.workspace = true"
    done
}

require_internal_pins() {
    local line dependency pin
    while IFS= read -r line; do
        dependency=${line%%=*}
        dependency=${dependency//[[:space:]]/}
        pin=$(sed -n 's/.*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' <<<"$line")
        [[ -n $pin ]] || fail "internal dependency $dependency has no version pin"
        [[ $pin == "$workspace_version" ]] \
            || fail "internal dependency $dependency is pinned to $pin; expected $workspace_version"
    done < <(grep -E '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=.*path[[:space:]]*=[[:space:]]*"crates/' "$repo/Cargo.toml")
}

# Every example's committed family-crate requirement must be satisfiable by the workspace version.
#
# The examples commit the adopter's real published form (`guibiao = "0.3"`) and the examples gate
# resolves the family to local source with `--config patch.crates-io.<crate>.path=…`. Cargo *silently
# drops* a patch whose local version no longer satisfies the requirement, so the moment the workspace
# bumps to 0.4.0 every example would resolve the last published 0.3.x from crates.io instead. That
# failure IS caught — `test_examples.sh` asserts the patch took effect — but it is caught by the dogfood
# job rather than by the gate that claims workspace/dependency version alignment, and it surfaces as a
# resolution puzzle rather than as "the release bump left the examples behind". Named here, at the
# release surface, so a bump reports the one thing the author has to do about it.
#
# The family package names are read from the workspace rather than listed, so a seventh crate is covered
# the day it becomes a member.
require_example_pins() {
    local expected_minor manifest dependency pin family seen=0 manifests=0
    expected_minor=${workspace_version%.*}
    mapfile -t family < <(workspace_packages)
    for manifest in "$repo"/examples/*/Cargo.toml; do
        [[ -f $manifest ]] || continue
        manifests=$((manifests + 1))
        for dependency in "${family[@]}"; do
            # Both dependency forms, so one example switching to the table form is not silently
            # skipped while the set-level guard below stays satisfied by its siblings:
            #   plain  `xuanji = "0.3"`
            #   table  `xuanji = { version = "0.3", features = [...] }`
            while IFS= read -r pin; do
                seen=$((seen + 1))
                [[ $pin == "$expected_minor" || $pin == "$workspace_version" ]] \
                    || fail "example $(basename "$(dirname "$manifest")") requires $dependency = \"$pin\", which the workspace version $workspace_version does not satisfy; expected \"$expected_minor\" (a release bump must carry the examples with it, or their patch.crates-io override is silently dropped)"
            done < <(sed -n \
                -e "s/^[[:space:]]*$dependency[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\1/p" \
                -e "s/^[[:space:]]*$dependency[[:space:]]*=[[:space:]]*{.*version[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\1/p" \
                "$manifest")
        done
    done
    # Vacuity guards, mirroring the workspace-manifest one: a rename of examples/, or a shift to the
    # table form (`tianheng = { version = "…" }`) that this line-form parse does not read, would
    # otherwise iterate zero times and pass with zero assertions.
    [[ $manifests -gt 0 ]] \
        || fail "found no example manifests under $repo/examples — the layout changed or is absent, so example version coherence cannot be verified"
    [[ $seen -gt 0 ]] \
        || fail "read $manifests example manifest(s) but found no family dependency requirement in any of them — the dependency form changed and this gate would pass vacuously"
}

workspace_packages() {
    local manifest
    for manifest in "${workspace_manifest_files[@]}"; do
        awk -F '"' '/^[[:space:]]*name[[:space:]]*=/ { print $2; exit }' "$manifest"
    done
}

lock_version_for() {
    local wanted=$1
    awk -v wanted="$wanted" '
        /^\[\[package\]\]$/ { name = ""; version = "" }
        /^[[:space:]]*name[[:space:]]*=/ {
            value = $0
            sub(/^[^"]*"/, "", value)
            sub(/".*/, "", value)
            name = value
        }
        /^[[:space:]]*version[[:space:]]*=/ {
            value = $0
            sub(/^[^"]*"/, "", value)
            sub(/".*/, "", value)
            version = value
            if (name == wanted) { print version; exit }
        }
    ' "$repo/Cargo.lock"
}

require_release_surfaces() {
    local package lock_version expected_release_link
    [[ $(grep -Ec '^## \[Unreleased\]$' "$repo/CHANGELOG.md") -eq 1 ]] \
        || fail "CHANGELOG must contain exactly one [Unreleased] section"
    changelog_unreleased_is_empty \
        || fail "[Unreleased] must be empty in $state state"
    grep -Eq "^## \\[$workspace_version\\] - [0-9]{4}-[0-9]{2}-[0-9]{2}$" "$repo/CHANGELOG.md" \
        || fail "CHANGELOG is missing dated release notes for $workspace_version"
    if [[ $state == release-ready || -n $previous_release_version ]]; then
        if [[ $state == release-ready ]]; then
            expected_release_link=$release_version
        else
            expected_release_link=$previous_release_version
        fi
        grep -Fqx "[$workspace_version]: https://github.com/tacticaldoll/tianheng/compare/v$expected_release_link...v$workspace_version" "$repo/CHANGELOG.md" \
            || fail "CHANGELOG comparison link for $workspace_version must start at v$expected_release_link"
    else
        grep -Fqx "[$workspace_version]: https://github.com/tacticaldoll/tianheng/releases/tag/v$workspace_version" "$repo/CHANGELOG.md" \
            || fail "first release CHANGELOG link must target v$workspace_version"
    fi
    while IFS= read -r package; do
        lock_version=$(lock_version_for "$package")
        [[ -n $lock_version ]] || fail "Cargo.lock is missing workspace package $package"
        [[ $lock_version == "$workspace_version" ]] \
            || fail "Cargo.lock package $package is $lock_version; expected $workspace_version"
    done < <(workspace_packages)
}

[[ -f $repo/Cargo.toml ]] || fail "repository root $repo has no Cargo.toml"
[[ -f $repo/CHANGELOG.md ]] || fail "repository root $repo has no CHANGELOG.md"
git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || fail "repository root $repo has no git history"

workspace_version=$(read_workspace_version)
[[ $workspace_version =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
    || fail "workspace version is missing or malformed: ${workspace_version:-<missing>}"

malformed_release=$(git -C "$repo" log --format='%s' \
    | awk '$0 ~ /^release:/ && $0 !~ /^release: (0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/ { print }')
[[ -z $malformed_release ]] \
    || fail "malformed release history subject: $(head -n 1 <<<"$malformed_release")"
mapfile -t release_records < <(git -C "$repo" log --format='%H%x09%s' \
    | awk -F '\t' '$2 ~ /^release: (0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/ { print }')
[[ ${#release_records[@]} -gt 0 ]] \
    || fail "exact release history is unavailable; fetch full history containing release: X.Y.Z"
release_record=${release_records[0]}
release_commit=${release_record%%$'\t'*}
release_subject=${release_record#*$'\t'}
release_version=${release_subject#release: }
previous_release_version=
if [[ ${#release_records[@]} -gt 1 ]]; then
    previous_release_subject=${release_records[1]#*$'\t'}
    previous_release_version=${previous_release_subject#release: }
fi
head_commit=$(git -C "$repo" rev-parse HEAD)

if [[ $head_commit == "$release_commit" ]]; then
    state=snapshot
    [[ $workspace_version == "$release_version" ]] \
        || fail "release snapshot subject is $release_version but workspace version is $workspace_version"
else
    order=$(version_order "$workspace_version" "$release_version")
    case $order in
        -1) fail "workspace version $workspace_version is older than latest release $release_version" ;;
        0) state=development ;;
        1) state=release-ready ;;
    esac
fi

# Discover the workspace crate manifests once, in the main body — NOT inside a `< <(...)`
# subshell, where a `fail` exits only the subshell and is swallowed by the outer read loop.
# Guard the set non-empty: if the crate layout ever deepens (crates/<group>/<pkg>) or `crates/`
# is renamed/absent, the `find` yields nothing and every manifest-and-lock loop below would
# otherwise iterate zero times and pass with zero assertions — a coherent-looking but vacuous
# gate. Mirrors the release-spine emptiness guard above (`${#release_records[@]} -gt 0`).
mapfile -t workspace_manifest_files < <(find "$repo/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml -type f | sort)
[[ ${#workspace_manifest_files[@]} -gt 0 ]] \
    || fail "found no workspace crate manifests under $repo/crates — the crate layout changed or is absent, so manifest and lock coherence cannot be verified"

require_workspace_manifests
require_internal_pins
require_example_pins

case $state in
    development)
        changelog_unreleased_has_item \
            || fail "development requires adopter-facing release narrative under [Unreleased]"
        grep -Fqx "[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v$workspace_version...HEAD" "$repo/CHANGELOG.md" \
            || fail "[Unreleased] comparison link must start at v$workspace_version and end at HEAD"
        ;;
    release-ready|snapshot)
        require_release_surfaces
        ;;
esac

printf 'ok release coherence (%s: %s)\n' "$state" "$workspace_version"
