#!/usr/bin/env bash
#
# The observation-bound register's reaction: every declared bound names what defends it, and every bound
# stated in spec prose is declared.
#
# An observation bound is a claim that a reaction deliberately STOPS at a named shape. It is the one claim
# class no reaction defends, and this repository carries roughly a hundred of them, which is how two came
# to outlive the behaviour they described: `inline-symbol-path-confinement` and
# `external-crate-confinement` both declared `#[path]`-remapped modules unobserved for two releases after
# the scanner began following them, and `rule-model-surface` carried a bound scenario contradicting a
# reacting scenario in its own requirement. A stale bound is worse than ordinary stale prose in the one
# direction that matters: it reads as PERMISSION, telling a future auditor that a real escape is governed
# policy.
#
# What a bound declaration is. A `#### Scenario:` whose heading marks it a bound, sitting under the
# requirement it qualifies — 21 of the 24 declared today sit that way, and hoisting them into a common
# section would separate each bound from the reaction it limits. The `Observation bounds` requirement
# three specs carry is a place bounds are gathered, never the definition of one.
#
# Requiring the heading convention is legitimate where requiring a TEST-name convention is not, and the
# difference is ownership: a scenario heading is authored in the spec, so this gate may require its form,
# while a test name pre-exists the register and belongs to its suite. (Measured: the bound-pinning tests
# follow at least three naming variants — `_is_a_stated_bound`, `_is_a_documented_bound`, `_is_a_bound…` —
# and some carry no "bound" in the name at all, so a test-name-keyed register would have reported pinned
# bounds as unpinned.)
#
# The two directions, and what each refuses:
#
#   * Every declared bound carries exactly one citation — `PINNED-BY` a test, or `UNPINNED` with a
#     tracker. Neither fails, because a bound with no recorded answer to "what defends this" is the
#     unbacked claim the register exists to end. Both fails, because a bound is either defended or
#     tracked and the declaration must say which.
#   * A `PINNED-BY` name resolves to exactly ONE function definition under `crates/`. Zero fails: a
#     renamed or deleted test leaves a citation that reads as coverage while defending nothing. Two also
#     fails: a name defined twice makes the citation name a set rather than a reaction. Matching is on the
#     definition form, never a bare mention, so a comment or a doc link cannot satisfy it.
#   * A bound stated in prose outside a declared bound scenario fails, which is what stops the register
#     being completed by declaring only the convenient bounds.
#
# The prose direction is a FLOOR, not a proof, and the projection says so where a reader will see it: a
# bound worded without the pattern below — "out-of-scope", "does not claim to observe" — is invisible to
# it. Claiming otherwise would be the register lying exactly where it is most trusted.
#
# A requirement heading naming bounds is not itself a declaration and is not scanned: it points at the
# bounds its scenarios declare (`External resolution has stated residual bounds`, and the glob-re-export
# requirement, are both that shape).
#
# Exit 0 clean, 1 violation, 2 cannot judge — the family's own Core Contract, so this reads the same way
# as the reactions it sits beside. Read-only: it never edits a spec or writes a projection.
set -euo pipefail

cd "$(dirname "$0")/.."

BOUND_HEADING='^#### Scenario: .*(stated|documented) bound'
BOUND_PROSE='(stated|documented) bounds?'

fail() {
    printf 'bound register: %s\n' "$*" >&2
    offenses=$((offenses + 1))
}

cannot_judge() {
    printf 'bound register: cannot judge: %s\n' "$*" >&2
    exit 2
}

offenses=0
declared=0
scanned=0

# One tracked spec file's declared bounds and unregistered prose, emitted as TAB-separated records for the
# loop below. Written in awk rather than a shell read-loop because the parse is stateful: a citation
# belongs to the bound heading above it, and a prose line's verdict depends on whether a bound heading is
# currently open.
#
# The line-terminating CR is stripped first. A CRLF checkout would otherwise carry it into the extracted
# test name, and every citation would resolve to nothing — the same checkout-dependence the whitespace and
# identity gates were fixed for. Written as a literal byte through `printf`, because BSD `sed` does not
# read `\r` on the left-hand side.
cr=$(printf '\r')
parse_spec() {
    local file=$1
    sed "s/${cr}\$//" -- "$file" | awk -v file="$file" -v heading="$BOUND_HEADING" -v prose="$BOUND_PROSE" '
        function flush() {
            if (open != "") {
                printf "BOUND\t%s\t%d\t%s\t%s\t%s\n", file, open_line, open, pinned, unpinned
            }
            open = ""; pinned = ""; unpinned = ""
        }
        # A requirement or section heading closes any open bound and is never itself scanned: it names the
        # requirement whose scenarios declare the bounds.
        /^#{1,3} / { flush(); in_scenario = 0; next }
        /^#### / {
            flush()
            in_scenario = 1
            if ($0 ~ heading) {
                open = substr($0, length("#### Scenario: ") + 1)
                open_line = NR
            }
            next
        }
        open != "" && /^[[:space:]]*-[[:space:]]+\*\*PINNED-BY\*\*/ {
            line = $0
            sub(/^[^`]*`/, "", line)
            sub(/`.*$/, "", line)
            pinned = (pinned == "" ? line : pinned "|" line)
            next
        }
        open != "" && /^[[:space:]]*-[[:space:]]+\*\*UNPINNED\*\*/ {
            line = $0
            sub(/^[[:space:]]*-[[:space:]]+\*\*UNPINNED\*\*[[:space:]]*/, "", line)
            gsub(/[[:space:]]+$/, "", line)
            unpinned = (unpinned == "" ? (line == "" ? "<empty>" : line) : unpinned "|" line)
            next
        }
        # Prose stating a bound outside any declared bound scenario.
        open == "" && $0 ~ prose { printf "PROSE\t%s\t%d\t%s\n", file, NR, $0 }
        END { flush() }
    '
}

# Definitions of a cited test name, as `file:line` records. The definition FORM only: a bare mention in a
# comment, a string, or a doc link defends nothing, so it must not satisfy a citation.
definitions_of() {
    grep -rnE "^[[:space:]]*(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+$1[[:space:]]*\(" \
        crates/ --include='*.rs' 2>/dev/null || true
}

mapfile -t spec_files < <(git ls-files 'openspec/specs/*/spec.md' | sort)
[[ ${#spec_files[@]} -gt 0 ]] \
    || cannot_judge "git ls-files matched no openspec/specs/*/spec.md — this gate would report clean without reading a spec"

for spec in "${spec_files[@]}"; do
    [[ -f $spec ]] || continue
    scanned=$((scanned + 1))
    while IFS=$'\t' read -r kind file line a b c; do
        case $kind in
        BOUND)
            declared=$((declared + 1))
            id="${file#openspec/specs/}"
            id="${id%/spec.md}/$a"
            if [[ -n $b && -n $c ]]; then
                fail "$id ($file:$line) carries both PINNED-BY and UNPINNED; a bound is either defended or tracked, and the declaration must say which"
                continue
            fi
            if [[ -z $b && -z $c ]]; then
                fail "$id ($file:$line) carries neither PINNED-BY nor UNPINNED; a bound with no recorded defence is the unbacked claim this register exists to end"
                continue
            fi
            if [[ -n $c ]]; then
                [[ $c == "<empty>" ]] \
                    && fail "$id ($file:$line) is UNPINNED with no tracker; untracked debt is indistinguishable from an oversight"
                continue
            fi
            # PINNED-BY may repeat only if the bound genuinely has two defenders; each must resolve.
            while IFS= read -r name; do
                [[ -n $name ]] || continue
                mapfile -t sites < <(definitions_of "$name")
                case ${#sites[@]} in
                0) fail "$id ($file:$line) is PINNED-BY \`$name\`, which no function under crates/ defines; a renamed or deleted test must not read as coverage" ;;
                1) : ;;
                *) fail "$id ($file:$line) is PINNED-BY \`$name\`, defined $(( ${#sites[@]} )) times — the citation names a set rather than a reaction:
$(printf '           %s\n' "${sites[@]%%:*}")" ;;
                esac
            done < <(printf '%s\n' "${b//|/$'\n'}")
            ;;
        PROSE)
            fail "$file:$line states a bound outside any declared bound scenario, so it is absent from the register:
           $(printf '%s' "$a" | cut -c1-108)"
            ;;
        esac
    done < <(parse_spec "$spec")
done

[[ $declared -gt 0 ]] \
    || cannot_judge "parsed 0 declared bounds across $scanned spec file(s) — the heading form changed, so this gate would pass vacuously"

if [[ $offenses -gt 0 ]]; then
    printf '\nbound register: %d offense(s) across %d declared bound(s) in %d spec file(s)\n' \
        "$offenses" "$declared" "$scanned" >&2
    printf 'remedy: give each declared bound one PINNED-BY naming a test that exists, or one UNPINNED\n' >&2
    printf '        naming its tracker; and declare a bound stated in prose as a bound-marked scenario\n' >&2
    exit 1
fi

printf 'bound register ok (%d declared bounds across %d spec files)\n' "$declared" "$scanned"
