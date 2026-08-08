#!/usr/bin/env bash
#
# Every declared mutation kills the pinning test it names: a citation defends a bound only if it can tell the
# reaction from a perturbation of it.
#
# `check_bound_register.sh` decides that a `PINNED-BY` citation names a test that RUNS — resolved to one
# definition under `crates/`, carrying `#[test]`, registered by the harness. It does not decide that the test
# BITES. Measured in this window rather than supposed: replacing a cited pin's entire body with a binding that
# asserts nothing left the suite at 16 passed and the register printing its citation count clean.
#
# Biting is not decidable from text, and the register already records why for the easier question one level
# down — a `cfg`-removed attribute, a definition trapped in an uninvoked macro, a definition inside a string or
# a comment. Whether a test WOULD fail under a different reaction is a question about running a program. So
# this gate runs the cited test against a mutated tree and reads its status.
#
# Four properties of the arrangement are load-bearing, each measured:
#
#   * The tree is a detached WORKTREE at HEAD, so the author's files are untouched whatever happens mid-run.
#     Note what that is NOT: the sibling gates enumerate tracked PATHS with `git ls-files` and then read the
#     worktree's content, deliberately — `check_whitespace_hygiene.sh`'s header calls reading anything else a
#     false negative. This gate is the only one judging HEAD's content, and it inherits the matching blind
#     spot: a pin gutted but not yet committed is invisible to it. Declared as a bound rather than left to be
#     discovered.
#   * The build gets its OWN target directory, because the gate's premise is that the binary under test was
#     built from the mutated tree — a shared directory has been seen to serve one that was not, and a verdict
#     over the wrong binary is not a verdict. That is the whole justification: attempts to reproduce a
#     PARTICULAR wrong verdict from sharing landed on cannot-judge (the previous run's mutated binary fails the
#     control) or on a correct clean run (cargo rebuilds), so no false-clean direction was found and none is
#     claimed. The matrix has no direction for this requirement.
#   * A mutation that breaks the BUILD is cannot-judge. `cargo test` exits non-zero for a compile error as
#     well as for a failing assertion, so the two are separated by building first. Note what this buys: the
#     exit CLASS is already 2 without it, because `ran_exactly_one` refuses a run that executed no test — the
#     separate build step buys the DIAGNOSTIC that names the compile error, which is the difference between an
#     author fixing the record in a minute and hunting for it.
#   * The records are parsed ONCE. Two readers over one file is how a set gets counted by one rule and
#     processed by another; that shape produced a false negative here and is written up at the parser itself.
#
# Each record is also run UNMUTATED first. Without that control a test that fails for its own reasons reads as
# a pin that bites, which is the `f() == f()` shape this repository refuses elsewhere.
#
# What a killed pin does NOT prove: that the record perturbed the REACTION rather than the pin's own
# assertions. A record naming the pin's file and neutralising one of its asserts kills it and counts as
# coverage, and no reaction here can tell the two apart — the first seeded record edits the very file its pin
# lives in, so a rule separating them would refuse the tree's own legitimate shape. Declared as a bound.
#
# Coverage is partial by construction and says so: a clean run prints how many distinct cited tests carry no
# mutation. Authoring a mutation that genuinely perturbs the pinned point is expert work per bound, and a
# mutation that misses reports a biting pin as a dead one — the safe direction, an author answers it with a
# better mutation. A gate that reported only the mutations it ran, and stayed silent about the rest, would be
# the reads-as-coverage failure this gate exists to end, one level up.
#
# Exit 0 every declared mutation killed its pin, 1 a record failed its half of the bargain, 2 cannot judge.
set -Eeuo pipefail
# The family's exit contract as a backstop — see `scripts/lib/exit_contract.sh` for what it catches, why it
# is a trap rather than per-command handling, and the measurements behind both.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
# One way to read an observation source — see `scripts/lib/capture.sh` for the two measured failures that shape it.
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/capture.sh"
exit_contract_backstop 'pin bites'

# The repository to judge, so the failure matrix can build throwaway fixtures rather than being able to test
# only this checkout — the same argument every sibling gate takes, for the same reason.
repo=${1:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}
cd "$repo"

RECORDS=scripts/lib/pin_mutations.tsv

fail() {
    printf 'pin bites: %s\n' "$1" >&2
    exit 1
}

cannot_judge() {
    printf 'pin bites: cannot judge: %s\n' "$1" >&2
    exit 2
}

git rev-parse --is-inside-work-tree >/dev/null 2>&1 \
    || cannot_judge "repository root $repo is not a git worktree; this gate judges tracked content"

command -v cargo >/dev/null 2>&1 \
    || cannot_judge "cargo is not on PATH; whether a pin bites is decided by running it, and nothing here can stand in for that"

git ls-files --error-unmatch "$RECORDS" >/dev/null 2>&1 \
    || cannot_judge "$RECORDS is not tracked; the declared mutations are the surface this gate judges"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

# Records first, so an empty or unreadable set refuses before the expensive build.
#
# Fields are TAB-separated: test name, tracked path, `from`, `to`. `\n` and `\t` in the two substrings are
# unescaped after the split, so a perturbation spanning lines is still one record. Blank lines and `#` lines
# are prose.
records=$work/records
capture_or_refuse "the declared mutations in $RECORDS" "$records" cannot_judge \
    -- git show "HEAD:$RECORDS"

# ONE parser, and the records are parsed ONCE into arrays the rest of this gate reads.
#
# Two readers over one file is how a set gets counted by one rule and processed by another. This gate had that
# shape and it produced a false negative, measured on this repository: the vacuity guard read lines with
# `IFS= read` while the record loop read them with `IFS=$'\t' read`, and TAB is IFS *whitespace*, so a
# comment line indented with one TAB was counted as a declared mutation by the first and skipped as prose by
# the second. A records file holding nothing to run reported `pin bites ok (0 declared mutations)` and exit 0 —
# the one outcome the Core Contract forbids, in the gate whose whole subject is a defence that is not
# defending. `check_whitespace_hygiene.sh` accepts a leading TAB, so nothing else stopped it being committed.
#
# The split is exact for the same reason: three TABs per record, no field collapsing, so a literal TAB inside a
# substring is a loud refusal rather than a silent shift of the fields after it.
#
# `|| [[ -n $line ]]` keeps a final record that carries no trailing newline. Dropping it silently would be a
# coverage loss reported as a clean run, which is this gate's own subject.
names=()
files=()
froms=()
tos=()
while IFS= read -r line || [[ -n $line ]]; do
    [[ -n $line && ${line:0:1} != '#' ]] || continue
    tabs=${line//[!$'\t']/}
    ((${#tabs} == 3)) \
        || cannot_judge "a record in $RECORDS carries ${#tabs} TAB(s) where a record is four TAB-separated fields (test, file, from, to). A literal TAB inside a substring must be written \\t, and a comment must open at column one — an indented \`#\` is a malformed record here, not prose, which is the safe reading of the two:
$line"
    rest=${line#*$'\t'}
    names+=("${line%%$'\t'*}")
    files+=("${rest%%$'\t'*}")
    rest=${rest#*$'\t'}
    froms+=("${rest%%$'\t'*}")
    tos+=("${rest#*$'\t'}")
done <"$records"
declared=${#names[@]}

((declared > 0)) \
    || cannot_judge "$RECORDS declares no mutation; every property of zero mutations holds, and reporting that as conformance is the vacuity direction this repository has re-opened most often"

# The citations this gate is allowed to speak about. `check_bound_register.sh` owns the authoritative parse and
# every other property of them; what is needed here is only the NAME SET, so that a mutation naming a test no
# bound cites is refused rather than passing as coverage of a citation that does not exist.
citations=$work/citations
capture_or_refuse 'the register PINNED-BY citations' "$citations" cannot_judge \
    -- git grep -h -E '^- \*\*PINNED-BY\*\* `[^`]+`' HEAD -- 'openspec/specs/*/spec.md'
cited_names() { sed -E 's/^- \*\*PINNED-BY\*\* `([^`]+)`.*/\1/; s/^.*:://' "$citations" | sort -u; }
capture_or_refuse 'the citation names' "$citations.names" cannot_judge -- cited_names
cited_total=$(wc -l <"$citations.names")
((cited_total > 0)) \
    || cannot_judge "no PINNED-BY citation was read from the specs; a gate about citations that found none would report every mutation valid against an empty set"

# The tree under test: tracked content at HEAD, as a **worktree** rather than an archive.
#
# `git archive | tar -x` was the first shape and it carries no `.git`, which makes some citations structurally
# unreachable rather than merely uncovered: a pin that reads the repository through git fails its own CONTROL
# run, so no record can ever exercise it. Measured — `units_outside_the_gate_pairing_are_outside_the_surface`
# is one such citation, and the BACKLOG entry claiming coverage "grows one considered record at a time" was
# false for it. A detached worktree is the same tracked content at HEAD with a working `.git`, and mutating it
# still touches none of the author's files.
tree=$work/tree
git worktree add --quiet --detach "$tree" HEAD \
    || cannot_judge "could not check out HEAD into a scratch worktree; nothing was observed"
# The worktree is registered in the judged repository's `.git`, so it is pruned even on an abort.
trap 'git worktree remove --force "$tree" >/dev/null 2>&1 || true; rm -rf "$work"' EXIT
export CARGO_TARGET_DIR=$work/target
export TIANHENG_WORKSPACE_TESTS=1

# Which package and target a citation runs in is DERIVED from where THE CITED TEST is defined — never from the
# file the mutation edits, and never declared beside the mutation. A record routinely perturbs a reaction in one
# file while the pin defending it lives in another, and deriving from the edited file then runs the wrong target:
# the fixture whose recognizer sits in `src/lib.rs` and whose pin sits in `tests/pin.rs` selected `--lib`, where
# the citation is not registered at all. A second spelling of the same fact would rot instead, which is why this
# is derived rather than a fifth field.
#
# It materializes the search and assigns to a caller-visible array, rather than refusing inside a process
# substitution: a refusal there exits that subshell and the parent reads nothing — the swallowed-status class
# `scripts/lib/capture.sh` was written for, and one this gate must not reintroduce in the helper that decides
# WHAT gets run.
selector=()
derive_selector() {
    local name=$1 pkg defined
    defining_files() { (cd "$tree" && grep -rlE "fn $name\\(" crates --include='*.rs' | sort); }
    capture_or_refuse "where \`$name\` is defined" "$work/defined" cannot_judge --ordinary-empty 1 -- defining_files
    mapfile -t defined <"$work/defined"
    ((${#defined[@]} == 1)) \
        || cannot_judge "\`$name\` is defined in ${#defined[@]} files under crates/; the target to run it in cannot be derived from a set"
    [[ ${defined[0]} =~ ^crates/([^/]+)/ ]] \
        || cannot_judge "${defined[0]} is not under crates/<package>/, so the package to run \`$name\` in cannot be derived"
    pkg=${BASH_REMATCH[1]}
    if [[ ${defined[0]} =~ ^crates/[^/]+/tests/([^/]+)\.rs$ ]]; then
        selector=(-p "$pkg" --test "${BASH_REMATCH[1]}")
    else
        selector=(-p "$pkg" --lib)
    fi
}

# The harness name, not the cited one. A lib test registers as `tests::<name>` while the citation is the bare
# identifier, and `--exact <bare>` then matches NOTHING — the harness runs zero tests and exits 0, which this
# gate would read as a pin surviving its mutation. Found by this gate's own first lib-target record, which is
# why the resolution is here rather than a `--exact` on the citation: the filter that silently matches nothing
# is the vacuity direction, and it arrived through the run that decides everything else.
resolve_test_name() {
    local name=$1 listed
    shift
    (cd "$tree" && cargo test --all-features "$@" -- --list) >"$work/list.log" 2>&1 \
        || cannot_judge "could not enumerate the tests of the target defining \`$name\`:
$(tail -20 "$work/list.log")"
    capture_or_refuse "the registered tests matching \`$name\`" "$work/listed" cannot_judge \
        -- awk -v n="$name" '/: test$/ { t = $0; sub(/: test$/, "", t); if (t == n || t ~ ("::" n "$")) print t }' "$work/list.log"
    mapfile -t listed <"$work/listed"
    ((${#listed[@]} == 1)) \
        || cannot_judge "\`$name\` resolves to ${#listed[@]} registered tests in that target; a filter matching none runs nothing and exits 0, and one matching several does not name the citation"
    resolved=${listed[0]}
}

# Was a test actually run? A filter matching nothing exits 0 with `0 passed`, which is indistinguishable from a
# passing pin by status alone. The count is read from the harness's own summary rather than assumed from the
# filter having been exact.
ran_exactly_one() {
    local passed failed
    passed=$(sed -n 's/^test result: [^.]*\. \([0-9]\+\) passed;.*/\1/p' "$work/run.log" | head -1)
    failed=$(sed -n 's/^test result: [^.]*\. [0-9]\+ passed; \([0-9]\+\) failed;.*/\1/p' "$work/run.log" | head -1)
    [[ -n $passed && -n $failed ]] && (( passed + failed == 1 ))
}

run_cited() {
    local name=$1 outcome=0
    shift
    (cd "$tree" && cargo test --all-features "$@" -- --exact "$name") >"$work/run.log" 2>&1 || outcome=$?
    return "$outcome"
}

build_cited() {
    local outcome=0
    (cd "$tree" && cargo test --no-run --all-features "$@") >"$work/build.log" 2>&1 || outcome=$?
    return "$outcome"
}

covered=$work/covered
: >"$covered"
for index in "${!names[@]}"; do
    name=${names[index]}
    file=${files[index]}
    from=${froms[index]}
    to=${tos[index]}
    # `to` may be empty — deleting the anchor is a legitimate perturbation. The other three may not.
    [[ -n $name && -n $file && -n $from ]] \
        || cannot_judge "a record carries an empty test name, path, or anchor: ${names[index]}"

    grep -Fxq "$name" "$citations.names" \
        || fail "the record for \`$name\` names a test no declared bound cites; a mutation is an assertion about a defence, and there is no defence here to assert about"

    # Tracked-ness, not reachability. `[[ -f $tree/$file ]]` accepts a `../` path that resolves OUTSIDE the
    # tree, and the mutation then rewrites a file this gate has no business touching — measured, and it
    # falsifies the interrupted-run property stated in this file's header. Asking git makes the check the one
    # the message already claimed.
    git ls-files --error-unmatch "$file" >/dev/null 2>&1 \
        || cannot_judge "the record for \`$name\` names $file, which HEAD does not track; a mutation edits tracked content of the tree under test and nothing else"

    derive_selector "$name"

    # The control. A test that fails for its own reasons would otherwise read as a pin that bites.
    build_cited "${selector[@]}" \
        || cannot_judge "the unmutated tree does not build for \`$name\`:
$(tail -20 "$work/build.log")"
    resolve_test_name "$name" "${selector[@]}"
    run_cited "$resolved" "${selector[@]}" \
        || cannot_judge "\`$name\` does not pass on the unmutated tree, so its failure under a mutation would say nothing:
$(tail -20 "$work/run.log")"
    ran_exactly_one \
        || cannot_judge "the control run for \`$name\` did not run exactly one test; a filter matching nothing exits 0 and would read as a pin surviving its mutation"

    cp "$tree/$file" "$work/original"
    # `from` must occur EXACTLY once. An anchor matching twice names a set rather than a site, and substituting
    # the first occurrence silently perturbs something other than what the record describes — the rule the
    # observer protocol's body reader reached the expensive way in the same window.
    applied=$(FROM=$from TO=$to python3 - "$tree/$file" <<'PY'
import os, pathlib, sys
path = pathlib.Path(sys.argv[1])
text = path.read_text()
frm = os.environ["FROM"].replace("\\n", "\n").replace("\\t", "\t")
to = os.environ["TO"].replace("\\n", "\n").replace("\\t", "\t")
count = text.count(frm)
if count != 1:
    print(count)
else:
    path.write_text(text.replace(frm, to))
    print(1)
PY
    ) || cannot_judge "could not apply the mutation for \`$name\` to $file"
    if [[ $applied != 1 ]]; then
        cp "$work/original" "$tree/$file"
        cannot_judge "the anchor for \`$name\` occurs $applied times in $file; zero and many are both a perturbation that was never applied, which is a different fact from the pin not biting"
    fi

    # A mutation that breaks the build observed nothing. The exit class here would be 2 in any case, since
    # `ran_exactly_one` refuses a run that executed no test; what this step buys is the diagnostic naming the
    # compile error rather than a report that nothing ran.
    if ! build_cited "${selector[@]}"; then
        cp "$work/original" "$tree/$file"
        cannot_judge "the mutation for \`$name\` does not compile, so the pin was never exercised:
$(tail -20 "$work/build.log")"
    fi

    survived=0
    run_cited "$resolved" "${selector[@]}" && survived=1
    ran_exactly_one || { cp "$work/original" "$tree/$file"; cannot_judge "the mutated run for \`$name\` did not run exactly one test, so nothing was observed either way"; }
    cp "$work/original" "$tree/$file"

    ((survived == 0)) \
        || fail "\`$name\` passes with its declared mutation applied to $file — a test that cannot tell the reaction from a perturbation of it defends nothing while occupying the place of a defence:
    $from
 -> $to"

    printf '%s\n' "$name" >>"$covered"
done

# The remainder is over DISTINCT CITED TESTS, and says so, because that is the population this gate can act on
# and it is not the register's. The register counts *citations* — one per `PINNED-BY` bullet — and blesses one
# test cited by two bounds in one capability, so the two figures differ by design; printing an unqualified
# "pinning citations" here made this gate a fifth answer to a question the register is the arbiter of, which
# `observation-bound-register` says in as many words. Both sides are counted over the same set now: a record
# names a test, several records may name one test, and covering a test twice cannot make the remainder
# negative — it did, measured, when a record count was subtracted from a name count.
covered_total=$(sort -u "$covered" | wc -l)
printf 'pin bites ok (%d declared mutations over %d of %d distinct cited tests)\n' \
    "$declared" "$covered_total" "$cited_total"
printf '  %d distinct cited tests carry no mutation — a clean run here is not every pin having been exercised\n' \
    "$((cited_total - covered_total))"
