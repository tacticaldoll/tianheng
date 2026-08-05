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
#   * A `PINNED-BY` name resolves to exactly ONE function definition under `crates/`, and that definition
#     is a TEST. Zero fails: a renamed or deleted test leaves a citation that reads as coverage while
#     defending nothing. Two also fails: a name defined twice makes the citation name a set rather than a
#     reaction. Resolving to a function that never runs as a test fails for the same reason as zero — a
#     helper of the right name defends nothing while occupying the place of the defence. Matching is on the
#     definition form, never a bare mention, so a comment or a doc link cannot satisfy it.
#   * An `UNPINNED` tracker names a path this repository TRACKS. That is the checkable part of "names an
#     owner"; whether the named section still describes the debt is prose no reaction can read, and
#     demanding it would trade a fact for a heuristic.
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

# Whether a citation is even well formed, checked BEFORE it is resolved. Two directions were silent passes
# until this existed, both measured on a fixture rather than argued:
#
#   * The name is interpolated into the search pattern above, so a regular-expression metacharacter resolves a
#     citation to a DIFFERENTLY NAMED function — `a_probe_bound_is_pinne.` found `a_probe_bound_is_pinned` and
#     the gate reported clean. That defeats the renamed-or-deleted direction this whole gate was built for: a
#     citation for a test that does not exist passed.
#   * The crate qualifier is joined to a filesystem path, so `../outside::a_fn` resolved against a function
#     OUTSIDE `crates/` — the boundary the requirement declares — and the gate reported clean.
#
# Validation rather than escaping, deliberately. Escaping `a_probe.` would make the gate search for a literal
# dot, find nothing, and report "no function under crates/ defines it" — right exit code, wrong diagnosis,
# since the citation is malformed and not stale. Validation names the actual defect, and the same rule on the
# qualifier closes the traversal direction for free, a crate-directory name holding neither `/` nor `.`.
#
# Measured before it was written: all 36 cited names are plain Rust identifiers and every directory under
# `crates/` is a plain name, so this refuses nothing that exists. A raw identifier (`r#type`) would be
# refused; none is cited, and the refusal is loud.
# A raw identifier is a Rust identifier and this register imposes no naming convention of its own, so `r#name`
# is accepted; `#` is not an ERE metacharacter, so nothing downstream changes. Non-ASCII identifiers stay
# refused and the requirement says so rather than implying Rust's full grammar: the search pattern is
# byte-oriented, no cited name needs otherwise, and the refusal is loud where an unreliable match would not be.
citation_is_well_formed() {
    local citation=$1 qualifier name
    case $citation in
    *::*::*) return 1 ;;
    *::*)
        qualifier=${citation%%::*}
        name=${citation##*::}
        [[ $qualifier =~ ^[A-Za-z0-9_-]+$ ]] || return 1
        ;;
    *) name=$citation ;;
    esac
    [[ $name =~ ^(r#)?[A-Za-z_][A-Za-z0-9_]*$ ]]
}

# The tests the HARNESS registers, per workspace package. This is the authority on "does this citation name a
# test that runs", and the text scan below is not.
#
# Three reviews produced falsifiers against deciding that from source text, the third producing three at once:
# a `#[test]` neutralised by `#[cfg(any())]`, a `#[test] fn` inside an uninvoked `macro_rules!` body, and a
# definition inside a raw string — every one accepted with exit 0, none of them a test that runs. Enumerating
# those sub-cases in a text scan is unbounded (cfg, cfg_attr, feature gates, a cfg-gated `mod`, comments,
# strings, macros); the previous version of this gate declared one of them as a residual and three more arrived
# in the next review.
#
# The enumeration was rejected TWICE in this file's own comments, on an unmeasured premise: that it needs a
# compiled workspace while the failure matrix builds repositories holding one `lib.rs` and no manifest. A
# throwaway repository can carry a six-line manifest, and it enumerates COLD in 107ms — the cost was estimated
# from inside the code instead of measured, which is the failure `BACKLOG.md`'s own governance preamble warns
# about, and it cost a wrong residual declaration that stood for one change.
#
# PER PACKAGE, not per workspace, because `--list` prints `module::path::name` with no crate label while a
# citation may be crate-qualified. That is not theoretical here:
# `a_cfg_gated_module_with_no_file_is_skipped_not_errored` is registered in BOTH 渾儀 and 漏刻 and the register
# cites the 渾儀 one, so a workspace-wide leaf match would let a citation qualified to a crate whose test had
# been cfg-disabled be satisfied by the other crate's live test — the very hole this direction closes,
# reintroduced by the shortcut. All six packages enumerate in 746ms warm.
declare -A harness_packages_of=()
harness_state=unknown
build_harness_index() {
    [[ $harness_state != unknown ]] && return 0
    if [[ ! -f $repo/Cargo.toml ]]; then
        harness_state=absent
        # Said out loud. A gate that silently drops its strongest direction reports a weaker clean than the
        # one it claims, and the failure matrix builds manifest-less repositories deliberately — most of this
        # register's directions have nothing to do with Rust.
        printf 'bound register: no root Cargo.toml — citation test-ness decided by the source-text fallback, not the test harness\n'
        return 0
    fi
    # The packages are the directories under `crates/`, which is checkable and checked: `cargo metadata`'s
    # JSON would need parsing, and a naive `grep '"name"'` over it collects target and dependency names too
    # (26 strings where six packages exist). If a directory is not a package name, `cargo test -p` fails and
    # this becomes `error` — loud, never a quiet skip.
    local members member listed leaf
    mapfile -t members < <(cd "$repo/crates" 2>/dev/null && find . -mindepth 1 -maxdepth 1 -type d -printf '%f\n' | sort)
    if [[ ${#members[@]} -eq 0 ]]; then
        harness_state=error
        return 0
    fi
    for member in "${members[@]}"; do
        listed=$(cd "$repo" && cargo test -p "$member" --all-features -- --list 2>/dev/null) || {
            harness_state=error
            return 0
        }
        while IFS= read -r leaf; do
            [[ -n $leaf ]] || continue
            harness_packages_of[$leaf]="${harness_packages_of[$leaf]:-,}$member,"
        done < <(printf '%s\n' "$listed" | sed -n 's/: test$//p' | sed 's/.*:://' | sort -u)
    done
    harness_state=ready
    printf 'bound register: citation test-ness decided by the test harness (%d registered test names across %d package(s))\n' \
        "${#harness_packages_of[@]}" "${#members[@]}"
}

# Whether the harness registers this citation, honouring its crate qualifier when it carries one.
harness_registers() {
    local citation=$1
    local qualifier=""
    local leaf=$citation
    case $citation in
    *::*)
        qualifier=${citation%%::*}
        leaf=${citation##*::}
        ;;
    esac
    local packages=${harness_packages_of[$leaf]:-}
    [[ -n $packages ]] || return 1
    [[ -z $qualifier ]] && return 0
    [[ $packages == *",$qualifier,"* ]]
}

# Whether the definition at `file:line` is a TEST. Read from the ATTRIBUTE RUN above the definition rather
# than from the line before it: `#[test]` / `#[should_panic(…)]` / `fn` is a shape this tree already carries
# in three places, so a single-line read would refuse a real test. That error direction matters — a refused
# genuine citation is a false positive an author argues with, and arguing with a gate is how a gate gets
# turned off.
#
# Requiring the cited function to be a test is not a naming convention imposed on a suite this register does
# not own; it is what the citation already means. Nothing here reads the test's NAME, which is what lets the
# bound-pinning tests keep their three naming variants.
#
# The walk stops at a line ending `{`, `}`, or `;` — the previous item's end — and at a BLANK line, so a
# `#[test]` above one function can never be read as covering a plain function beneath it. Stopping at a blank
# refuses `#[test]`, blank, `fn`: legal Rust nobody writes, and refusing it is loud where leaking an
# attribute across items would be the silent false coverage this gate exists to refuse. Line comments are
# walked past; `// #[test]` is not a marking, since it must be the attribute and not a mention.
#
# It also stops at a BLOCK-COMMENT DELIMITER, which is how `/*` `#[test]` `*/` `pub fn cited()` — a non-test
# satisfying a citation with exit 0 — is refused. The delimiter is treated as a boundary and never
# interpreted, because the two alternatives do not work here:
#
#   * Tracking comment state while walking UPWARD is impossible in principle. Whether a line sits inside a
#     block comment is a property of everything BEFORE it, and this walk moves backwards with no knowledge of
#     the file above.
#   * Stripping comments needs to know which `/*` opens one and which is text inside a string literal, and
#     this tree makes that concrete: 49 `/*` occurrences live INSIDE string literals, several of them nested,
#     because louke's lexer and the lexical-conformance suite test exactly that
#     (`crates/tianheng/tests/lexical_conformance.rs:72`, `crates/louke/src/audit/tests.rs:673`). A
#     delimiter-counting stripper would open a phantom comment at the first of them and swallow every
#     definition until the next `*/`, so the gate would begin refusing real citations here on its first run.
#
# Verified before adopting: no `#[test]` run in this tree contains a block comment, and none of the 36 cited
# tests is affected. The error direction is loud — a run that genuinely contains one is refused, not accepted.
#
# There is NO line cap. A 12-line window was a backstop against walking to the top of a file, but the stop
# conditions already are that boundary, so the cap only ever removed correct behaviour: a `#[test]` above 13
# further attributes was refused, and no attribute-run length is declared anywhere. The preceding lines are
# read once rather than one `sed` per line, which is also cheaper.
#
# This walk is the FALLBACK, not the authority. `cargo test -- --list` decides test-ness wherever a manifest
# exists (`build_harness_index` above), and this comment used to say that source was "rejected: it needs a
# compiled workspace, and the whole failure matrix is throwaway repositories holding one `lib.rs` and no
# manifest." Both halves were wrong, and the second was never measured: a fixture crate carrying a six-line
# manifest enumerates cold in ~107ms, so a throwaway repository can carry one. The estimate was made from
# inside this file instead of measured, which is why the text scan spent three review rounds defending a
# claim it could not hold.
#
# What the walk still owns is the manifest-less repository — most of this register's directions have nothing
# to do with Rust — plus the definition SITE, which the harness's leaf names do not carry.
#
# In that fallback the form-matching limit is live and is not stated as a bound of the register: a definition
# inside a block comment or a raw string satisfies a citation, because this reads a line's FORM and not its
# comment or string state. The harness closes it wherever one exists, since neither shape compiles into a
# registered test, and the fallback announces itself in the gate's own output so a clean result names which
# direction decided it.
definition_is_test() {
    local file=$1 line=$2
    local n=0 trimmed
    local above=()
    ((line > 1)) || return 1
    mapfile -t above < <(sed -n "1,$((line - 1))p" -- "$file")
    for ((n = ${#above[@]}; n >= 1; n--)); do
        trimmed=${above[n - 1]}
        trimmed=${trimmed#"${trimmed%%[![:space:]]*}"}
        trimmed=${trimmed%"${trimmed##*[![:space:]]}"}
        case $trimmed in
        '#[test]'*) return 0 ;;
        '') return 1 ;;
        *'/*'* | *'*/'*) return 1 ;;
        *'{' | *'}' | *';') return 1 ;;
        esac
    done
    return 1
}

# The paths this repository tracks, read once. `-z` because `git ls-files` quotes a non-ASCII path by
# default, and a quoted path would match nothing — the same checkout-dependence the whitespace and identity
# gates were fixed for.
declare -A tracked_paths=()
tracked_path_index_built=0
build_tracked_path_index() {
    [[ $tracked_path_index_built == 1 ]] && return 0
    local path
    while IFS= read -r -d '' path; do
        tracked_paths[$path]=1
    done < <(git -C "$repo" ls-files -z)
    tracked_path_index_built=1
}

# Whether an `UNPINNED` tracker names a path this repository tracks. This is the checkable part of "names an
# owner": `BACKLOG.md READY-PATCH "declared bounds with no pinning test"` names one, and `no test exists`
# names none — the citation the requirement forbids, which until now passed because any non-empty text was
# accepted. Which SECTION of that document owns the debt is deliberately not checked: that is prose, and a
# gate guessing at prose produces the false positives that get gates disabled.
#
# Split with `read -ra` rather than an unquoted expansion, so a tracker containing `*` cannot glob against
# the working directory.
tracker_names_a_tracked_path() {
    local text=$1 token parts=()
    build_tracked_path_index
    read -ra parts <<<"$text"
    for token in "${parts[@]}"; do
        token=${token#[\"\'\`(\[]}
        token=${token%[\"\'\`)\].,;:]}
        [[ -n $token && -n ${tracked_paths[$token]:-} ]] && return 0
    done
    return 1
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
            if [[ $c == "<empty>" ]]; then
                fail "$id ($file:$line) is UNPINNED with no tracker; untracked debt is indistinguishable from an oversight"
            elif ! tracker_names_a_tracked_path "$c"; then
                fail "$id ($file:$line) is UNPINNED with \"$c\", which names no path this repository tracks; a tracker that cannot be read is anonymous debt wearing an owner's name"
            fi
            continue
        fi
        while IFS= read -r name; do
            [[ -n $name ]] || continue
            # Before resolution, never after: an ill-formed citation must be named as ill-formed rather than
            # searched for and reported stale.
            if ! citation_is_well_formed "$name"; then
                fail "$id ($file:$line) is PINNED-BY \`$name\`, which is not a citation this reaction can resolve; a name must be a Rust identifier and a crate qualifier a crate-directory name, with at most one \`::\` — otherwise a metacharacter resolves the citation to a differently-named function, or a path component resolves it outside crates/"
                continue
            fi
            build_harness_index
            [[ $harness_state == error ]] \
                && cannot_judge "the test harness could not be enumerated under $repo/crates — a citation's test-ness is undecided rather than weakly decided, so this gate refuses to judge instead of falling back"
            if [[ $harness_state == ready ]] && ! harness_registers "$name"; then
                fail "$id ($file:$line) is PINNED-BY \`$name\`, which the test harness does not register for that crate; a citation names what defends the bound, and a definition removed by a cfg, trapped in an uninvoked macro, or written inside a string or comment runs nothing"
                continue
            fi
            mapfile -t sites < <(definitions_of "$name")
            case ${#sites[@]} in
            0)
                # With the harness authoritative, "registered but not located" is a DISAGREEMENT about a form,
                # not about existence, so it is named as one. The scan requires `fn` and the name on one line;
                # a definition split across lines was previously reported as an absent test.
                if [[ $harness_state == ready ]]; then
                    fail "$id ($file:$line) is PINNED-BY \`$name\`, which the harness registers but no line under crates/ matches the definition form; this scan requires \`fn\` and the name on one source line, so the site cannot be reported"
                else
                    fail "$id ($file:$line) is PINNED-BY \`$name\`, which no function under crates/ defines; a renamed or deleted test must not read as coverage"
                fi
                ;;
            1)
                site_file=${sites[0]%%:*}
                site_line=${sites[0]#*:}
                site_line=${site_line%%:*}
                # The attribute walk is the FALLBACK only. Where the harness answered, consulting it again
                # would produce disagreement noise on shapes the enumeration already excludes.
                # Reported repo-relative, so the site is readable in a fixture run and copy-pasteable in a
                # real one; the grep that found it had to be absolute.
                [[ $harness_state == ready ]] || definition_is_test "$site_file" "$site_line" \
                    || fail "$id ($file:$line) is PINNED-BY \`$name\`, whose only definition at ${site_file#"$repo"/}:$site_line carries no \`#[test]\` in the attribute run above it; a function that never runs as a test defends nothing while occupying the place of the defence"
                ;;
            *)
                site_paths=()
                for site in "${sites[@]}"; do
                    site=${site%%:*}
                    site_paths+=("${site#"$repo"/}")
                done
                fail "$id ($file:$line) is PINNED-BY \`$name\`, defined ${#sites[@]} times — the citation names a set rather than a reaction:
$(printf '           %s\n' "${site_paths[@]}")"
                ;;
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
# Keyed on the cited test rather than on statement text: text similarity would be a heuristic where a shared
# citation is a fact. Repetition WITHIN one capability is not a restatement — a bound covering two shapes
# cites two tests, and one capability may cite one test from two bounds — so the direction fires only across
# capabilities.
#
# This direction is a FLOOR, and the projection says so: two declarations of one behaviour citing two
# DIFFERENT tests are invisible to it. Telling those apart from two genuine bounds over sibling shapes is a
# semantic judgment, and the evidence that a similarity key would be wrong rather than merely imprecise is in
# the tree: `semantic-dyn-trait-operand-boundary` and `semantic-impl-trait-operand-boundary` both declare
# `A genuinely unresolvable bare principal is a documented bound`, with distinct WHEN clauses (`dyn` versus
# `impl Trait`) and distinct pinning tests, and 三儀 ⊥ 三儀 requires each dimension to declare its own. A key
# over heading text or statement similarity would fire on that pair and the only repair it would accept is
# dissolving a symmetry the constitution requires.
#
# Of the two restatements that motivated this capability, note which direction reaches which: the
# `#[path]`-remap bound was PROSE in `external-crate-confinement` and a scenario in
# `inline-symbol-path-confinement`, so the undeclared-prose direction is what reaches that shape — not this
# one. Crediting this direction with it would be the overclaim the register exists to end.
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
        '' \
        'The second floor is the same shape. A bound declared twice is caught only when both declarations cite' \
        'the **same pinning test**, which is a fact rather than a heuristic; two declarations of one behaviour' \
        'citing two different tests are invisible. Telling those apart from two genuine bounds over sibling' \
        'shapes is a semantic judgment — two operand dimensions here declare identically-worded bounds over' \
        '`dyn` and `impl Trait`, each defended by its own test, and each must declare its own — so nothing' \
        'observes it and no bound of the register capability claims it.' \
        '' \
        'A third floor was stated here for one change and is **retired**: a `pinned by` line could be satisfied' \
        'by a definition that never ran — commented out, inside a string, removed by a `cfg`, or trapped in an' \
        'uninvoked macro — because the scan read only the form of a line. Test-ness is now decided by the test' \
        'harness enumeration, which registers none of those. The weakness survives only in the source-text' \
        'fallback used where no manifest exists, which the register spec describes.' \
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

# Cannot-judge precedes WRITING, not merely judging. With no declared bound parsed the heading form has
# changed and the register is not there to be projected, so rendering first would leave behind a `0 of 0`
# document that reads as the complete register of a repository holding no bounds — the flattering direction a
# gate whose subject is absence makes easy.
[[ $declared -gt 0 ]] \
    || cannot_judge "parsed 0 declared bounds across $scanned spec file(s) — the heading form changed, so this gate would pass vacuously"

if [[ ${BLESS:-} == 1 ]]; then
    # The directory is part of what blessing generates, so a fresh checkout — or a test fixture — can be
    # blessed without being prepared for it first.
    mkdir -p "$(dirname "$repo/$PROJECTION")"
    render_projection "$unpinned_count" >"$repo/$PROJECTION"
    # Blessing WRITES and then falls into the same verdict as any other run. It used to exit 0 here, which
    # made regeneration report the family's "clean" over a register whose offenses it had just printed:
    # "the document was rewritten" and "the register it describes is valid" are different claims, and one
    # exit code cannot carry both. The projection is written either way, deliberately — with the register
    # invalid, seeing what it now says is how the author repairs it.
    if [[ $offenses -gt 0 ]]; then
        printf 'blessed %s (%d declared bounds, %d unpinned) — the register it describes is NOT valid\n' \
            "$PROJECTION" "$declared" "$unpinned_count"
    else
        printf 'blessed %s (%d declared bounds, %d unpinned)\n' "$PROJECTION" "$declared" "$unpinned_count"
    fi
elif [[ -f $repo/$PROJECTION ]]; then
    rendered=$(mktemp)
    render_projection "$unpinned_count" >"$rendered"
    if ! diff -q "$rendered" "$repo/$PROJECTION" >/dev/null 2>&1; then
        fail "$PROJECTION no longer matches the specs; regenerate it with BLESS=1 bash scripts/check_bound_register.sh"
    fi
    rm -f "$rendered"
else
    fail "$PROJECTION is missing; generate it with BLESS=1 bash scripts/check_bound_register.sh"
fi

if [[ $offenses -gt 0 ]]; then
    printf '\nbound register: %d offense(s) across %d declared bound(s) in %d spec file(s)\n' \
        "$offenses" "$declared" "$scanned" >&2
    printf 'remedy: give each declared bound one PINNED-BY naming a test that exists, or one UNPINNED\n' >&2
    printf '        naming its tracker; and declare a bound stated in prose as a bound-marked scenario\n' >&2
    exit 1
fi

printf 'bound register ok (%d declared bounds across %d spec files)\n' "$declared" "$scanned"
