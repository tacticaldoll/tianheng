#!/usr/bin/env bash
# Exit 0 coherent, 1 incoherent, 2 cannot judge — the family's own Core Contract, stated here because the
# gate now holds it on every path and a contract a reader cannot find is one they cannot rely on.
set -Eeuo pipefail
# The family's exit contract as a backstop — see `scripts/lib/exit_contract.sh` for what it catches, why it
# is a trap rather than per-command handling, and the measurements behind both.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
# One way to read an observation source: materialize, check the status HERE in the parent, then consume. See
# `scripts/lib/capture.sh`. Measured on this gate: a `git log` that emitted one release record and then failed made
# it conclude the tree was in snapshot state and report `[Unreleased] must be empty` — exit 1, a violation invented
# from a partial read. Every producer whose output decides a verdict now goes through the shared rule.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/capture.sh"
exit_contract_backstop 'release coherence'

repo=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}

# Exits rather than `return 1`-and-let-`set -e`-do-it. That indirection was live and wrong: with the shared
# exit-contract backstop installed, the `ERR` trap fired on the returned 1 and converted every genuine
# incoherence into `2` — cannot judge — which is the one collapse the family contract forbids. Measured on a
# fixture whose internal pin disagreed: the gate printed the right diagnosis and then exited 2. The matrix
# could not see it, asserting only a non-zero status; that is fixed too.
fail() {
    printf 'release coherence: %s\n' "$*" >&2
    exit 1
}

# The other half of the contract this gate's header claims and never held: an input it cannot read is not an
# incoherence. A shallow clone with no release spine, an absent manifest, a layout that moved — none of those
# say "the release surfaces disagree", and reporting them as `1` tells a consumer to go looking for a
# disagreement that does not exist. Every sibling gate separates these two; this one collapsed them.
release_capture=$(mktemp)
# The lockfile read gets a buffer of its own, and that is load-bearing rather than tidy. The package list is
# read from `$release_capture` by a `while` loop that asks the lockfile about each package INSIDE it, so a
# capture into the shared buffer would truncate the list mid-read: the loop would end after the first package
# and every later disagreement would go unreported — a false negative, the one direction the Core Contract
# forbids, produced by nothing more than reusing a filename. Every other capture here is sequential and may
# share; a nested one may not.
lock_capture=$(mktemp)
# The changelog's own structure, read once and asked several questions. Sequential like `$release_capture`,
# so it may share nothing with the nested lockfile read above.
changelog_capture=$(mktemp)
# The enumeration of this repository's own machinery, read from `git ls-files` rather than written beside the
# rule. A list of gate names kept next to its enumerator lets a new script be added and never measured, which
# is the register's own prohibition; the enumerator is the only authority.
machinery_capture=$(mktemp)
trap 'rm -f "$release_capture" "$lock_capture" "$changelog_capture" "$machinery_capture"' EXIT

cannot_judge() {
    printf 'release coherence: cannot judge: %s\n' "$*" >&2
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
    local line dependency pin pins=0
    # `grep` exits 1 on a clean miss, which the vacuity guard below is what reacts to — not this capture.
    capture_or_refuse 'the internal path dependencies' "$release_capture" cannot_judge --ordinary-empty 1 -- \
        grep -E '^[[:space:]]*[A-Za-z0-9_-]+[[:space:]]*=.*path[[:space:]]*=[[:space:]]*"crates/' "$repo/Cargo.toml"
    while IFS= read -r line; do
        pins=$((pins + 1))
        dependency=${line%%=*}
        dependency=${dependency//[[:space:]]/}
        pin=$(sed -n 's/.*version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' <<<"$line")
        [[ -n $pin ]] || fail "internal dependency $dependency has no version pin"
        [[ $pin == "$workspace_version" ]] \
            || fail "internal dependency $dependency is pinned to $pin; expected $workspace_version"
    done <"$release_capture"
    # The vacuity guard every other loop in this file already had, and this one did not: a reformatted
    # `[workspace.dependencies]` table, or a `grep` that could not read the manifest, iterates zero times and
    # the direction passes having asserted nothing about any pin.
    [[ $pins -gt 0 ]] \
        || cannot_judge "found no internal path dependency in $repo/Cargo.toml — the declaration form changed, so pin coherence cannot be verified"
}

# Every example's committed family-crate requirement must be satisfiable by the workspace version.
#
# The examples commit the adopter's real published-version requirement form and the examples gate
# resolves the family to local source with `--config patch.crates-io.<crate>.path=…`. Cargo *silently
# drops* a patch whose local version no longer satisfies the requirement, so the moment the workspace
# version advances an example left behind may resolve the last compatible published crate instead. That
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
    capture_or_refuse 'the workspace package list' "$release_capture" cannot_judge -- workspace_packages
    mapfile -t family <"$release_capture"
    for manifest in "$repo"/examples/*/Cargo.toml; do
        [[ -f $manifest ]] || continue
        manifests=$((manifests + 1))
        for dependency in "${family[@]}"; do
            # Both dependency forms, so one example switching to the table form is not silently
            # skipped while the set-level guard below stays satisfied by its siblings:
            #   plain  `xuanji = "<requirement>"`
            #   table  `xuanji = { version = "<requirement>", features = [...] }`
            capture_or_refuse "example $dependency pins" "$release_capture" cannot_judge -- sed -n \
                -e "s/^[[:space:]]*$dependency[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\\1/p" \
                -e "s/^[[:space:]]*$dependency[[:space:]]*=[[:space:]]*{.*version[[:space:]]*=[[:space:]]*\"\([^\"]*\)\".*/\\1/p" \
                "$manifest"
            while IFS= read -r pin; do
                seen=$((seen + 1))
                [[ $pin == "$expected_minor" || $pin == "$workspace_version" ]] \
                    || fail "example $(basename "$(dirname "$manifest")") requires $dependency = \"$pin\", which the workspace version $workspace_version does not satisfy; expected \"$expected_minor\" (a release bump must carry the examples with it, or their patch.crates-io override is silently dropped)"
            done <"$release_capture"
        done
    done
    # Vacuity guards, mirroring the workspace-manifest one: a rename of examples/, or a shift to the
    # table form (`tianheng = { version = "…" }`) that this line-form parse does not read, would
    # otherwise iterate zero times and pass with zero assertions.
    [[ $manifests -gt 0 ]] \
        || cannot_judge "found no example manifests under $repo/examples — the layout changed or is absent, so example version coherence cannot be verified"
    [[ $seen -gt 0 ]] \
        || cannot_judge "read $manifests example manifest(s) but found no family dependency requirement in any of them — the dependency form changed and this gate would pass vacuously"
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
    capture_or_refuse 'the workspace package list' "$release_capture" cannot_judge -- workspace_packages
    while IFS= read -r package; do
        # Migrated with the class every sibling producer here was: an `awk` over `Cargo.lock` is a read of an
        # observation source, and a failed read is not an absent package. Unmigrated it reached the shared
        # backstop, which can only say which line aborted; named, it says which lockfile entry could not be
        # read. Into `$lock_capture` and never `$release_capture` — see that buffer's own comment for the false
        # negative the shared name would produce here.
        capture_or_refuse "the lockfile entry for $package" "$lock_capture" cannot_judge -- \
            lock_version_for "$package"
        lock_version=$(<"$lock_capture")
        [[ -n $lock_version ]] || fail "Cargo.lock is missing workspace package $package"
        [[ $lock_version == "$workspace_version" ]] \
            || fail "Cargo.lock package $package is $lock_version; expected $workspace_version"
    done <"$release_capture"
}

[[ -f $repo/Cargo.toml ]] || cannot_judge "repository root $repo has no Cargo.toml"
[[ -f $repo/CHANGELOG.md ]] || cannot_judge "repository root $repo has no CHANGELOG.md"
git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || cannot_judge "repository root $repo has no git history"

workspace_version=$(read_workspace_version)
[[ $workspace_version =~ ^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$ ]] \
    || cannot_judge "workspace version is missing or malformed: ${workspace_version:-<missing>}"

malformed_release=$(git -C "$repo" log --format='%s' \
    | awk '$0 ~ /^release:/ && $0 !~ /^release: (0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/ { print }')
[[ -z $malformed_release ]] \
    || fail "malformed release history subject: $(head -n 1 <<<"$malformed_release")"
# A pipeline cannot be handed to `capture_or_refuse "$@"`, so it becomes a function — which also puts the `git log`
# status where `pipefail` can carry it out. Its failure is the measured one: truncated history made this gate
# conclude snapshot state and invent `[Unreleased] must be empty`.
release_history() {
    git -C "$repo" log --format='%H%x09%s' \
        | awk -F '\t' '$2 ~ /^release: (0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)$/ { print }'
}
capture_or_refuse 'the release history' "$release_capture" cannot_judge -- release_history
mapfile -t release_records <"$release_capture"
[[ ${#release_records[@]} -gt 0 ]] \
    || cannot_judge "exact release history is unavailable; fetch full history containing release: X.Y.Z — a shallow clone cannot see the release spine, which is not the same as surfaces that disagree"
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
capture_or_refuse 'the workspace crate manifests' "$release_capture" cannot_judge -- \
    find "$repo/crates" -mindepth 2 -maxdepth 2 -name Cargo.toml -type f
sort -o "$release_capture" "$release_capture" \
    || cannot_judge "could not sort the workspace crate manifests"
mapfile -t workspace_manifest_files <"$release_capture"
[[ ${#workspace_manifest_files[@]} -gt 0 ]] \
    || cannot_judge "found no workspace crate manifests under $repo/crates — the crate layout changed or is absent, so manifest and lock coherence cannot be verified"

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

# --- the CHANGELOG's own internal consistency ---
#
# Everything above reads the changelog's STATE — which version, which sections exist, whether the link is
# right. Nothing read whether a section is coherent with itself, and the closing review of the 0.5.0 window
# found two defects of exactly that shape: an `[Unreleased]` that had grown a SECOND `### Changed` heading, and
# a prose claim about which prior releases carry a `### Migration` section that was wrong under every reading.
#
# Both are decidable, and neither needs the prose detector this repository measured and rejected three times:
# the changelog has a grammar, this gate already walks it, and these are properties of that grammar. What is
# NOT decidable stays out — whether an entry is accurate, whether "no adopter action" is true, whether the
# wording is right. The line is between the document's structure and its content, and only the first is here.
changelog_sections() {
    awk '
        # First input: the machinery enumeration. Both the full path and the bare basename are recognised,
        # because the document cites both forms — `scripts/check_publish_source.sh` and `check_pin_bites.sh`.
        # No count of that enumeration is written here: a census is produced, never typed, and the first draft
        # of this comment stated one that the very commit adding it made stale.
        #
        # A basename colliding with a file an entry names for another reason would make this fire on that
        # citation — a false positive, the safe direction, and declared as a bound rather than left implicit.
        # Keyed on FILENAME rather than on `NR == FNR`, and that is load-bearing. With an EMPTY enumeration
        # file, `NR == FNR` holds for every line of the *changelog* — awk consumes the document as its own
        # enumerator and emits no record at all. Measured against that keying rather than argued: the gate then
        # exits **2** on the section vacuity guard, so a repository that tracks no machinery is refused instead
        # of judged. A repository with none has nothing an entry could leak and is legitimately clean; it must
        # reach that verdict by having nothing to match, never by the parser losing its second input.
        FILENAME == ARGV[1] {
            paths[$0] = 1
            base = $0
            sub(/.*\//, "", base)
            bases[base] = 1
            next
        }
        /^## \[/ { section = $0; sub(/ - .*/, "", section); heading = ""; printf "SECTION\t%s\n", section; next }
        section == "" { next }
        /^### / { heading = substr($0, 5); printf "HEADING\t%s\t%s\n", section, heading }
        /\*\*BREAKING\*\*/ { printf "BREAKING\t%s\n", section }
        # Recognition is by WORD — a maximal run of path characters — and each run must EQUAL a tracked path or
        # basename. That is exact matching of a lexical token, not substring matching: no sentence merely
        # containing the characters can match, because the run is delimited by the first character a path
        # cannot hold.
        #
        # It reads a run rather than a whole backticked span, and that was measured rather than preferred. The
        # span rule was written first and adversarial review reproduced three false negatives against it, every
        # one of them a shape this document already uses: a span carrying anything besides the bare path
        # (`bash scripts/check_pin_bites.sh`, `scripts/check_pin_bites.sh --fix`, `./scripts/…`) compared
        # unequal and passed; a double-backtick span — the section already holds four — mispaired the regex and
        # swallowed the path; and an inline span wrapped across a source line left its continuation unscanned,
        # a shape live on three lines of the governed section. Reading runs closes all three at once and
        # reaches a markdown link target as well, which the span rule never could.
        #
        # Attribution is line -> heading in force -> section, which is the document grammar this gate already
        # walks: every line of a list item sits under the same heading as its first, so item boundaries buy
        # nothing the heading does not already give.
        {
            rest = $0
            while (match(rest, /[A-Za-z0-9_.\/-]+/)) {
                token = substr(rest, RSTART, RLENGTH)
                rest = substr(rest, RSTART + RLENGTH)
                # A leading `./` and trailing sentence punctuation belong to the prose, not to the name.
                sub(/^\.\//, "", token)
                sub(/\.+$/, "", token)
                if (token in paths || token in bases)
                    printf "CITATION\t%s\t%s\t%s\n", section, heading, token
            }
        }
    ' "$machinery_capture" "$repo/CHANGELOG.md"
}

# A failed read refuses; an EMPTY successful read does not. Those are different facts: `git ls-files` exiting
# non-zero says the enumeration was never obtained, while an empty result says the judged repository tracks no
# machinery — and a repository with none has nothing an entry could leak. What stays outside is an UNTRACKED
# `scripts/`, which this reads as absent; judging worktree content a gate's own law says to read from the index
# would be the larger error, so that blindness is declared as a bound.
capture_or_refuse "the tracked files under scripts/" "$machinery_capture" cannot_judge \
    -- git -C "$repo" ls-files scripts/

changelog_shape=$changelog_capture
capture_or_refuse "the CHANGELOG's section structure" "$changelog_shape" cannot_judge \
    --ordinary-empty 1 -- changelog_sections

# The vacuity guard is over SECTIONS, not headings: a changelog whose sections carry bullets directly and no
# `###` sub-headings is an ordinary small changelog, and refusing it would refuse this repository's own early
# releases. A changelog with no `## [` section at all is the undecidable one.
grep -q '^SECTION' "$changelog_shape" \
    || cannot_judge "no \`## [\` section was read from CHANGELOG.md; a document with no release sections cannot be judged coherent, and reporting that as coherent is the vacuity direction"

# A heading twice in one release section splits what belongs together, and a reader of the second half never
# learns the first exists. Measured: an `[Unreleased]` carried two `### Changed` blocks 330 lines apart, each
# describing the same window.
duplicate_headings=$(awk -F'\t' '$1 == "HEADING" { seen[$2 "\t" $3]++ } END { for (k in seen) if (seen[k] > 1) print k }' "$changelog_shape")
[[ -z $duplicate_headings ]] \
    || fail "a CHANGELOG release section repeats a heading, so entries that belong together are split:
$duplicate_headings"

# A section marking a change **BREAKING** owes its reader the migration in one place. `[0.4.0]` established
# that and `[Unreleased]` follows it; the direction is one-way, because a section may carry a migration for a
# break marked some other way — `[0.3.0]` does.
missing_migration=$(awk -F'\t' '
    $1 == "BREAKING" { breaks[$2] = 1 }
    $1 == "HEADING" && $3 == "Migration" { migration[$2] = 1 }
    END { for (s in breaks) if (!(s in migration)) print s }
' "$changelog_shape")
[[ -z $missing_migration ]] \
    || fail "a CHANGELOG section marks a change **BREAKING** and carries no \`### Migration\` section, so what an adopter must do is scattered through the entries or absent:
$missing_migration"

# `CHANGELOG.md` is the adopter's document, and every heading it offered — Added, Changed, Fixed, Migration —
# is an adopter's vocabulary. It offered no heading that was not, so every change to this repository's own
# machinery was written into one of them: nineteen entries name it — ten in `[Unreleased]` and nine in the
# released `[0.4.0]` — for a directory that ships in zero packages. `### Self-governance` is that missing
# heading, and this refuses the leak back into the others.
#
# Adopter-facing is defined as the COMPLEMENT of that one heading rather than as a list of the four. A heading
# nobody anticipated is then adopter-facing, which is the direction that reacts; enumerating the adopter set
# would make every future heading exempt by default.
#
# Scope is `[Unreleased]`. A dated section records what was true at that release, and rewriting it to satisfy a
# rule written afterwards would falsify the record — the same reason `docs/history/` is left alone. That
# blindness is declared as an observation bound rather than left to be inferred from this condition.
adopter_cited_machinery=$(awk -F'\t' '
    $1 == "CITATION" && $2 == "## [Unreleased]" && $3 != "Self-governance" {
        printf "  %s under `### %s` names %s\n", $2, ($3 == "" ? "(no heading)" : $3), $4
    }
' "$changelog_shape" | sort -u)
[[ -z $adopter_cited_machinery ]] \
    || fail "an adopter-facing CHANGELOG entry names this repository's own machinery, which ships in no package and which an adopter can never run — move it under \`### Self-governance\`, or, where the adopter-relevant fact is genuinely there, state the guarantee and drop the filename:
$adopter_cited_machinery"

printf 'ok release coherence (%s: %s)\n' "$state" "$workspace_version"
