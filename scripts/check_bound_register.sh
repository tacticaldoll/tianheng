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

# The repository to judge, so the failure matrix can build throwaway fixtures rather than being able to
# test only this checkout. A gate that cannot be pointed at a fixture cannot have its refusals proven.
repo=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}

# The marking, and why it admits one interposed word. `(stated|documented) bound` adjacent is precise but
# brittle: `An underscore rename is a documented non-observed bound` declares a bound and would be missed,
# so a spec would have to be reworded to suit the tool. One optional word between admits that heading and
# `a stated coverage bound` while still refusing Rust's own sense of the word — `assoc type bound`,
# `supertrait bound` — which a bare "contains bound" rule would sweep in wholesale.
BOUND_HEADING='^#### Scenario: .*(stated|documented)( [A-Za-z-]+)? bounds?'
BOUND_PROSE='(stated|documented)( [A-Za-z-]+)? bounds?'

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
    sed "s/${cr}\$//" -- "$repo/$file" | awk -v file="$file" -v heading="$BOUND_HEADING" -v prose="$BOUND_PROSE" '
        # `<none>` rather than an empty field: TAB is IFS whitespace, so the reading shell collapses
        # consecutive tabs and an empty citation would slide the next field into its place — which it did,
        # reading an UNPINNED tracker as a PINNED-BY test name until the failure matrix caught it.
        function flush() {
            if (open != "") {
                printf "BOUND\t%s\t%d\t%s\t%s\t%s\t%s\n", file, open_line, open,
                    (pinned == "" ? "<none>" : pinned), (unpinned == "" ? "<none>" : unpinned),
                    (statement == "" ? "<none>" : statement)
            }
            open = ""; pinned = ""; unpinned = ""; statement = ""
        }
        # A requirement or section heading closes any open bound and is never itself scanned: it names the
        # requirement whose scenarios declare the bounds.
        #
        # A requirement whose own heading names bounds DECLARES them, and several do it as a numbered prose
        # list — `Observation bounds are stated, not silent` enumerates seven. Requiring each item to become
        # its own scenario would restructure three requirements and read worse, so the prose of such a
        # requirement is exempt. The exemption is not free: the requirement must hold at least one declared bound
        # scenario, or its prose list would have no reaction anywhere, which is the state this whole register
        # opposes. That obligation is emitted as a record and checked by the caller.
        /^#{1,3} / {
            flush()
            if (req != "" && req_is_bounds && req_stated) {
                printf "REQBOUNDS\t%s\t%d\t%s\t%s\n", file, req_line, req, (req_declared ? "yes" : "no")
            }
            in_scenario = 0
            req = ""; req_is_bounds = 0; req_declared = 0; req_stated = 0
            if ($0 ~ /^### Requirement:/) {
                req = substr($0, length("### Requirement: ") + 1)
                req_line = NR
                req_is_bounds = (tolower(req) ~ /bounds?([^a-z]|$)/)
            }
            next
        }
        /^#### / {
            flush()
            in_scenario = 1
            if ($0 ~ heading) {
                open = substr($0, length("#### Scenario: ") + 1)
                open_line = NR
                req_declared = 1
            }
            next
        }
        # The bound as stated: its first THEN, which is the sentence a reader needs and the projection
        # carries. Tabs are squashed so the record cannot be split by one.
        open != "" && statement == "" && /^[[:space:]]*-[[:space:]]+\*\*THEN\*\*/ {
            line = $0
            sub(/^[[:space:]]*-[[:space:]]+\*\*THEN\*\*[[:space:]]*/, "", line)
            gsub(/\t/, " ", line)
            statement = line
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
        # A mention whose negation applies to the BOUND NOUN ITSELF is not a declaration:
        # `a cfg-blind union rather than a skip bound` says the shape is not a bound, so demanding a
        # declaration of it would demand a declaration of something the sentence denies.
        #
        # The adjacency is load-bearing and was measured, not reasoned. A first attempt allowed the negation
        # anywhere within 60 characters before the phrase, and it hid three REAL declarations while catching
        # none of the intended cases: `type aliases are not expanded (a stated bound)`,
        # `the invocation is not transparent, so its body stays a stated coverage bound`, and
        # `a production probe must not live behind a non-production cfg — a stated bound` all carry a
        # negation on a different verb. Only `(rather than|not|never) a <word> bound` — the negation directly
        # on the noun — is skipped.
        function negated(text) {
            return match(text, /(rather than|not|never) an?( [A-Za-z-]+)? bounds?/) > 0
        }
        # Prose stating a bound outside any declared bound scenario.
        # A bounds-heading requirement is answerable only for prose that actually states a bound. Its
        # heading naming bounds is not itself an obligation: the register capability describes the
        # mechanism in requirements whose headings say "bound" while stating none.
        open == "" && req_is_bounds && $0 ~ prose && !negated($0) { req_stated = 1; next }
        open == "" && !req_is_bounds && $0 ~ prose && !negated($0) {
            line = $0
            gsub(/\t/, " ", line)
            printf "PROSE\t%s\t%d\t%s\n", file, NR, line
        }
        END {
            flush()
            if (req != "" && req_is_bounds && req_stated) {
                printf "REQBOUNDS\t%s\t%d\t%s\t%s\n", file, req_line, req, (req_declared ? "yes" : "no")
            }
        }
    '
}

# Definitions of a cited test name, as `file:line` records. The definition FORM only: a bare mention in a
# comment, a string, or a doc link defends nothing, so it must not satisfy a citation.
#
# A citation may be written `<crate>::<name>` to disambiguate, and it has to be: two dimensions legitimately
# give the same-shaped bound the same test name — `a_cfg_gated_module_with_no_file_is_skipped_not_errored`
# exists in both 渾儀 and 漏刻 — and the alternative would be renaming a pre-existing test to suit this
# register, which is the one thing it must not require of a suite it does not own.
definitions_of() {
    local name=$1 root=$repo/crates
    case $name in
    *::*)
        root=$repo/crates/${name%%::*}
        name=${name##*::}
        # Emit NOTHING for an absent crate, so the caller counts zero sites and refuses. An earlier
        # attempt printed a placeholder, which the caller counted as one site — an absent crate qualifier
        # then read as coverage, the silent pass this whole gate opposes. Caught by the matrix.
        [[ -d $root ]] || return 0
        ;;
    esac
    grep -rnE "^[[:space:]]*(pub([[:space:]]*\([^)]*\))?[[:space:]]+)?(async[[:space:]]+)?fn[[:space:]]+$name[[:space:]]*\(" \
        "$root" --include='*.rs' 2>/dev/null || true
}

git -C "$repo" rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || cannot_judge "repository root $repo is not a git worktree; this gate judges tracked content"
mapfile -t spec_files < <(git -C "$repo" ls-files 'openspec/specs/*/spec.md' | sort)
[[ ${#spec_files[@]} -gt 0 ]] \
    || cannot_judge "git ls-files matched no openspec/specs/*/spec.md — this gate would report clean without reading a spec"

records=$(mktemp)
ids=$(mktemp)
trap 'rm -f "$records" "$ids"' EXIT

for spec in "${spec_files[@]}"; do
    [[ -f $repo/$spec ]] || continue
    scanned=$((scanned + 1))
    parse_spec "$spec" >>"$records"
done

# A bound's id is derived from where it sits: `<capability>/<slug>`, the slug being the heading lowercased
# with each run of non-alphanumerics collapsed to one hyphen. Nothing allocates it, so no ledger exists to
# fall out of step — and the reference direction below checks the derivation is injective rather than
# assuming it. Character classes only, never `\+` or `\{1,\}`, so BSD and GNU sed agree.
slug_of() {
    printf '%s' "$1" | tr '[:upper:]' '[:lower:]' \
        | sed -e 's/[^a-z0-9][^a-z0-9]*/-/g' -e 's/^-//' -e 's/-$//'
}

# Pass 1 — the id table, built from every spec before any prose is judged, because a reference may point
# at a bound declared in a different capability's file.
while IFS=$'\t' read -r kind file line heading pinned unpinned statement; do
    [[ $kind == BOUND ]] || continue
    declared=$((declared + 1))
    capability="${file#openspec/specs/}"
    capability="${capability%/spec.md}"
    printf '%s\t%s:%s\n' "$capability/$(slug_of "$heading")" "$file" "$line" >>"$ids"
done <"$records"

# Pass 2 — the verdicts.
while IFS=$'\t' read -r kind file line a b c d; do
    case $kind in
    BOUND)
        capability="${file#openspec/specs/}"
        capability="${capability%/spec.md}"
        id="$capability/$(slug_of "$a")"
        [[ $b == "<none>" ]] && b=""
        [[ $c == "<none>" ]] && c=""
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
        while IFS= read -r name; do
            [[ -n $name ]] || continue
            mapfile -t sites < <(definitions_of "$name")
            case ${#sites[@]} in
            0) fail "$id ($file:$line) is PINNED-BY \`$name\`, which no function under crates/ defines; a renamed or deleted test must not read as coverage" ;;
            1) : ;;
            *) fail "$id ($file:$line) is PINNED-BY \`$name\`, defined ${#sites[@]} times — the citation names a set rather than a reaction:
$(printf '           %s\n' "${sites[@]%%:*}")" ;;
            esac
        done < <(printf '%s\n' "${b//|/$'\n'}")
        ;;
    REQBOUNDS)
        [[ $b == yes ]] \
            || fail "$file:$line — the requirement \"$a\" names bounds, so its prose may state them, but it declares no bound scenario; a prose list with no reaction anywhere is the state this register opposes"
        ;;
    PROSE)
        # A reference is the third option between rewriting prose that is doing its job and restating a
        # bound that already exists elsewhere — the restatement being the drift this register exists to end.
        reference=$(printf '%s' "$a" | sed -n 's/.*(bound:[[:space:]]*\([A-Za-z0-9_./-]*\)).*/\1/p' | head -n 1)
        if [[ -z $reference ]]; then
            fail "$file:$line states a bound outside any declared bound scenario, so it is absent from the register:
           $(printf '%s' "$a" | cut -c1-108)"
            continue
        fi
        mapfile -t targets < <(awk -F'\t' -v want="$reference" '$1 == want { print $2 }' "$ids")
        case ${#targets[@]} in
        0) fail "$file:$line references bound \`$reference\`, which no declared bound produces; a dangling reference is indistinguishable from an undeclared bound" ;;
        1) : ;;
        *) fail "$file:$line references bound \`$reference\`, which two declared bounds produce — a derived id must be unique:
$(printf '           %s\n' "${targets[@]}")" ;;
        esac
        ;;
    esac
done <"$records"

# One behaviour has one defence, so a test cited by declared bounds in more than one capability means the
# same bound is declared twice. That restatement has cost this repository twice already — the `#[path]`-remap
# bound was stale in two capabilities at once, and a sync left a contradicting bound beside its own reacting
# scenario — so one behaviour change must not be able to leave several specs stale.
#
# Keyed on the cited test rather than on statement text: two declarations of one behaviour will not have
# identical prose, and text similarity would be a heuristic where a shared citation is a fact. Repetition
# WITHIN one capability is not a restatement — a bound covering two shapes cites two tests, and one
# capability may cite one test from two bounds — so the direction fires only across capabilities.
#
# The reaction names the capabilities and demands a choice rather than computing ownership, which would mean
# modelling which capability a test exercises: the judgment the drift law keeps out of a reaction.
while IFS=$'\t' read -r test caps; do
    fail "the test \`$test\` is cited by declared bounds in ${caps//,/, } — one behaviour has one defence, so one capability declares the bound and the others reference it with (bound: …)"
done < <(awk -F'\t' '
    $1 == "BOUND" && $5 != "<none>" {
        cap = $2
        sub(/^openspec\/specs\//, "", cap)
        sub(/\/spec\.md$/, "", cap)
        n = split($5, tests, "|")
        for (i = 1; i <= n; i++) {
            if (index("," seen[tests[i]] ",", "," cap ",") == 0) {
                seen[tests[i]] = (seen[tests[i]] == "" ? cap : seen[tests[i]] "," cap)
                count[tests[i]]++
            }
        }
    }
    END { for (t in count) if (count[t] > 1) printf "%s\t%s\n", t, seen[t] }
' "$records" | sort)

# The projection, built from the SAME records the verdicts came from rather than a second parse, so the
# document and the reaction cannot disagree about what a bound is.
#
# Generated and staleness-checked, the discipline `AGENTS.self-law.md` already follows: a hand-maintained
# structural document drifts from what it describes, and a register of a hundred claims would drift fastest.
# `BLESS=1` rewrites it; any other run compares and fails.
PROJECTION=docs/observation-bounds.md

render_projection() {
    local unpinned_total=$1
    printf '%s\n' \
        '# Observation bounds' \
        '' \
        'Every **observation bound** this family declares: a claim that a reaction deliberately stops at a' \
        'named shape, so that shape is governed policy rather than a defect.' \
        '' \
        "**$unpinned_total of $declared declared bounds have no pinning test.** That figure is the register's" \
        'audit backlog and leads the document because a number in a footnote is not read. Each such bound names' \
        'the tracker that owns closing it.' \
        '' \
        'Generated from `openspec/specs/*/spec.md` by `scripts/check_bound_register.sh`. **Do not edit by hand** —' \
        'regenerate with `BLESS=1 bash scripts/check_bound_register.sh`. A stale projection fails that gate.' \
        '' \
        '**What this document does not claim.** It lists the bounds the specs *state in a recognizable form*: a' \
        'scenario whose heading marks it a bound. A bound worded outside that form — "out-of-scope", "does not' \
        'claim to observe" — is invisible to the scan that assembles this, so the list is a floor rather than a' \
        'proof of completeness. A register that implied otherwise would mislead exactly where it is most' \
        'trusted.' \
        ''
    local last=''
    while IFS=$'\t' read -r kind file line heading pinned unpinned statement; do
        [[ $kind == BOUND ]] || continue
        local cap="${file#openspec/specs/}"
        cap="${cap%/spec.md}"
        if [[ $cap != "$last" ]]; then
            printf '\n## %s\n' "$cap"
            last=$cap
        fi
        printf '\n### `%s/%s`\n\n' "$cap" "$(slug_of "$heading")"
        [[ $statement == "<none>" ]] || printf '> %s\n\n' "$statement"
        if [[ $pinned != "<none>" ]]; then
            local one
            while IFS= read -r one; do
                [[ -n $one ]] && printf -- '- **pinned by**: `%s`\n' "$one"
            done < <(printf '%s\n' "${pinned//|/$'\n'}")
        else
            printf -- '- **unpinned**, tracked by: %s\n' "$unpinned"
        fi
    done <"$records"
}

unpinned_count=$(awk -F'\t' '$1 == "BOUND" && $5 == "<none>" { n++ } END { print n + 0 }' "$records")

if [[ ${BLESS:-} == 1 ]]; then
    # The directory is part of what blessing generates, so a fresh checkout — or a test fixture — can be
    # blessed without being prepared for it first.
    mkdir -p "$(dirname "$repo/$PROJECTION")"
    render_projection "$unpinned_count" >"$repo/$PROJECTION"
    printf 'blessed %s (%d declared bounds, %d unpinned)\n' "$PROJECTION" "$declared" "$unpinned_count"
    exit 0
fi

if [[ -f $repo/$PROJECTION ]]; then
    rendered=$(mktemp)
    render_projection "$unpinned_count" >"$rendered"
    if ! diff -q "$rendered" "$repo/$PROJECTION" >/dev/null 2>&1; then
        fail "$PROJECTION no longer matches the specs; regenerate it with BLESS=1 bash scripts/check_bound_register.sh"
    fi
    rm -f "$rendered"
else
    fail "$PROJECTION is missing; generate it with BLESS=1 bash scripts/check_bound_register.sh"
fi

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
