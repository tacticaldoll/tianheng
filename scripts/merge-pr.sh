#!/usr/bin/env bash
#
# The sanctioned merge path: the squash-message gate, then `gh pr merge --squash`.
#
# Why a wrapper rather than a documented rule. `AGENTS.md` already says the squash subject is the pull
# request's title with no auto-appended `(#N)`; nine subjects, counted when this wrapper was written, carry that serial
# anyway, the most recent on the commit that landed a check for a requirement enforced by nothing. The
# failure mode is not disagreement about the rule — it is one string typed at the one moment nothing can be
# undone. A merged squash cannot be repaired: amending it changes its hash, and the pull request's merge
# record cites that hash, so the two would name different things afterwards.
#
# Nothing here carries a verdict. The judgement is `crates/kanhe/tests/merge_message.rs`, a Rust repository check
# like every other one judging this repository; this script gathers the inputs and refuses to reach `gh`
# without it.
#
# What stays outside, and what no longer does. WHETHER to merge remains a human's call. Whether CI AGREED
# does not: `require_ci_green` below reads the rollup and refuses, unconditionally, because the alternative
# was measured — this wrapper merged nineteen consecutive red runs on the difference between a local
# Definition of Done and the superset CI runs. So this holds what the merge is about to record AND that the
# suite agreed about it.
#
# This sentence said both were a human's call until 2026-08-21, and `require_ci_green` had landed 204 commits
# earlier. A premise its own new code had falsified, left standing where an operator reads it first — and the
# `--admin` arm below was reasoned from it, which is how a stale premise spreads rather than merely sits.
#
# A merge made in the GitHub web UI reaches no wrapper at all; that is a declared bound, not an oversight.
set -Eeuo pipefail

usage() {
    printf 'usage: %s <pr-number> --body-file <path> [--subject <text>] [gh args…]\n' "${0##*/}" >&2
    printf '  The subject defaults to the pull request title, which is what the rule requires anyway.\n' >&2
}

# --- the two exit classes, chosen in one place ------------------------------------------------------------
#
# `2` is everything this wrapper could not judge: a misconfigured invocation, and an input it could not read.
# `1` is a gate that ran and refused. The contract is this repository's own — `crates/shengmo/src/law.rs`:
# *0 clean, 1 violation, 2 constitution/usage error* — and its sibling `scripts/publish.sh` states the rule for
# arguments already.
#
# **Five could-not-read conditions were split across both classes with no rule.** An unresolvable repository
# exited 2 while an unreadable body file, an unreadable head, an unresolvable pull-request number and an
# unreadable commit set exited 1. Two of those facts are ones the gate this wrapper fronts types the other way:
# `merge_message_gate::judge` returns cannot-judge for an unavailable title and for unavailable commit subjects,
# because "which is not the same fact as a subject that disagrees". So the wrapper reported as a disagreement
# what its own gate calls unjudgeable — telling an operator, in the words of the sibling publish gate, "to go
# looking for a disagreement that does not exist".
#
cannot_judge() {
    printf 'merge message: %s\n' "$1" >&2
    exit 2
}

# The refusal idiom, delegating the class to the function above rather than choosing it again.
#
# **Converging the `case` arms onto this helper left four sites behind, and two of them predated it.** The
# positional selector and the body-file guard exited through a bare `usage; exit 2` carrying none of the
# prefix above; the URL refusal hand-copied that function's body because both helpers were defined below it;
# and `require_value` re-spelled it fifty-nine lines after it. Every stop now delegates, and what decides that
# is `each_wrapper_chooses_its_exit_class_in_one_place` rather than the next reader — a helper's existence was
# never the property, since three of those four sites were written with it in scope.
refuse() {
    cannot_judge "refusing \`$1\`: $2"
}

# A misconfigured invocation: the same class, plus the usage line, since what the operator needs here is the
# shape of the call rather than a fact about the pull request.
#
# `usage` runs BEFORE the message because the delegate exits; the reason is stated rather than left as an
# ordering someone later reads as arbitrary, and it puts the cause on the last line where a terminal shows it.
usage_error() {
    usage
    cannot_judge "$1"
}

pr=${1:-}
if [[ -z $pr ]]; then
    usage_error "this wrapper takes the pull request as its first positional argument, and none was given"
fi
if [[ $pr == -* ]]; then
    usage_error "the first argument is the pull request, not a flag; \`$pr\` reads as one"
fi
# A URL names its own repository, and this wrapper reads its evidence from several places. `gh pr view` and
# `gh pr merge` would follow the URL while the live-commits endpoint is built from a repository reference of its
# own — so a cross-repository URL has the gate judge one pull request and the merge record another, which is the
# same hole a `--repo` flag opened and this positional selector reopens. A number or a branch name names no
# repository and resolves against the one being pinned below, so both stay accepted.
if [[ $pr == http://* || $pr == https://* ]]; then
    refuse "$pr" \
        "a pull-request URL names its own repository, while this wrapper reads the live commit set from the \
repository it is run in. Pass the number, or run it from a checkout of that repository"
fi
shift

# **The class a wrapper exits is now decided by construction, not by a sweep that must be exhaustive.**
#
# Under `set -e` any unguarded failure exits with the TOOL's status, and this repository reserves `1` for a
# gate that ran and refused. Two sweeps were widened to catch that — first by tool name, then by command
# substitution — and a bare `cd` walked through both, because the axis was never *which shape the statement
# has*: it is *any statement whose failure can choose the class*. That is every command, which is why
# enumerating them is the wrong instrument. Enumerating what may exit `1` is the right one, and there is
# exactly one such statement: the gate's own verdict arm.
#
# Measured on bash 5.x rather than reasoned about. A bare failure traps and exits 2, including a failed `cd`.
# A `||`-guarded command does not trap, so every existing guard still decides its own outcome. A failure in a
# condition — `if`, `while`, `!`, `&&` — does not trap, so the `grep -q` that checks the gate ran is
# unaffected. An explicit `exit 1` is not intercepted, so the gate's verdict still reaches the caller. `set -E`
# is required and is not optional: without it a failure inside a function exits 1 and the trap never sees it.
trap 'cannot_judge "an unguarded command failed, so this wrapper stopped without reaching a verdict — which is not the same fact as a gate that ran and refused"' ERR

# This wrapper's own root, from which the gate is run — acquired after `cannot_judge` rather than at the top
# of the file, because it is an acquisition like any other and must report the class that function defines.
# Unguarded it was the one statement `set -e` answered for: a failed `cd` exits 1, so a wrapper that never
# found its gate would have reported the class that means the gate ran and refused.
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd) || cannot_judge \
    "cannot resolve this wrapper's own root from ${BASH_SOURCE[0]}, so the gate it must run before reaching \
\`gh pr merge\` cannot be located — which is not the same fact as a gate that ran and refused"

# The gate comes from `$repo`, this wrapper's own tree. Every piece of evidence — the title, the number, the
# head, the live commit subjects — comes from whatever `gh` resolves out of the WORKING DIRECTORY. Those are
# one tree whenever this is run the way its own refusals say to run it, and the `--repo` refusal enumerates
# them as one. Nothing held that. Invoked by absolute path from another checkout the two come apart silently,
# and this wrapper would judge one repository's pull request by another repository's law and then merge it.
#
# Compared through git rather than as strings: `$repo` is a logical path and a worktree root is a physical
# one, so two spellings of the same tree would refuse. Asking git for both puts them in one form.
gate_tree=$(git -C "$repo" rev-parse --show-toplevel) || cannot_judge \
    "cannot resolve the worktree holding this wrapper's gate at $repo, so which law it would apply cannot be \
decided"
evidence_tree=$(git rev-parse --show-toplevel) || cannot_judge \
    "cannot resolve the worktree this is being run from, so which repository the evidence would come from \
cannot be decided"
if [[ $gate_tree != "$evidence_tree" ]]; then
    cannot_judge \
        "this wrapper's gate lives in $gate_tree and its evidence would come from $evidence_tree. The gate is \
one repository's law and the pull request is another's, and merging the second under the first is not a \
judgement about either. Run it from a checkout of the repository whose pull request you are merging, using \
that checkout's own copy of this wrapper"
fi

subject=""
body_file=""
passthrough=()
# A value-taking flag given no value is an OBSERVABLE misconfiguration, so it fails loud. Before this it failed
# silent: `shift 2` with one argument left returns non-zero, `set -e` took that as the exit, and the wrapper
# stopped with no output at all — while every other refusal below prints `merge message: …`. Reproduced by
# running the wrapper with `--subject` last: empty output, exit 1. Validating before shifting is what keeps the
# arithmetic from becoming the diagnostic.
# A value-taking flag's value, checked for both ways it can be wrong: absent, and flag-shaped.
#
# **A value position is not a place a refused argument may sit, and this arm was missing while its sibling
# argued the point at length.** `scripts/publish.sh` refuses a value beginning with `-` and says why —
# measured on cargo 1.96.0, `--package --no-verify` packages WITHOUT verifying. The same door stood open here
# and had a different consequence: `--subject --admin` made the subject the literal string `--admin`, so the
# operator's flag never reached `gh` while the gate reported a subject disagreeing with the title. It fails
# closed and diagnoses the wrong thing, which is the class both wrappers spend paragraphs closing.
#
# Checked by SHAPE rather than against the refusal list, for the reason the sibling records: the list is not
# the property. What is true here is this wrapper's own — it does not accept a value beginning with `-`, and
# it does not read `gh`'s handling of a flag-shaped value, which differs by flag and by version.
require_value() {
    if (($1 < 2)); then
        usage_error "refusing \`$2\` with no value: this wrapper reads every value as the argument after its \
flag, so pass it that way or drop the flag"
    fi
    if [[ $3 == -* ]]; then
        usage_error "refusing \`$2\`: its value is \`$3\`, and this wrapper does not accept a value \
beginning with \`-\`. A refused argument does not become admitted by sitting in a value position, and an \
admitted one does not reach \`gh\` by being read as text. Pass a value, or drop the flag"
    fi
}

while (($#)); do
    case $1 in
    --subject)
        require_value "$#" "$1" "${2-}"
        subject=$2
        shift 2
        ;;
    --body-file)
        require_value "$#" "$1" "${2-}"
        body_file=$2
        shift 2
        ;;
    # --- What may reach `gh pr merge`, and nothing else -------------------------------------------------
    #
    # This was a DENYLIST and it leaked three times: a `--repo` flag, a positional pull-request URL, and every
    # SHORT spelling of the flags the long-form arms named. `gh` accepts `-t` for `--subject` and `-F` for
    # `--body-file`, this wrapper splices the passthrough AFTER its own flags, and `gh` takes the LAST
    # occurrence of a repeated flag — measured on gh 2.95.0, where `--body-file A -F B` and `-F A --body-file B`
    # both read B. So one unlisted spelling replaced the very message the gate had just approved.
    #
    # Enumerating what to forbid is the shape that failed. This enumerates what may pass, so a flag the wrapper
    # does not know — including one a future `gh` adds — is refused by default, which is the property a
    # denylist cannot have. This family already argues it in its own law: an allowlist is always stricter than
    # a denylist.
    #
    # Classified against `gh pr merge --help` on gh 2.95.0 by TWO questions. First: does it move what the gate
    # judged? Second: does gh honour it as this wrapper composes the invocation — beside the `--squash`,
    # `--subject` and `--body-file` written below? The second question was missing, and the sibling
    # publish wrapper paid for it: it admitted `--package` beside an unconditional `--workspace`, which cargo
    # silently maps to *all packages*. Here the same question refuses `--auto` and `--disable-auto`: one defers
    # the merge past the evidence the gate read, the other is not a merge at all.
    #
    # Forwarded are the flags that change whether the merge may proceed, never what it would record, and never
    # WHEN it happens relative to the evidence.
    #
    # ONE spelling each, values as separate arguments. Parsing gh's glued and equals forms is what let the
    # short forms through; refusing those costs an argument's worth of typing and removes the parsing question.
    # `--admin` is admitted for what it still does: bypass required **reviews**. In a single-steward
    # repository a pull request's author cannot approve their own, so `require_code_owner_reviews` cannot be
    # satisfied by the person merging — which `PROJECT.md` records as a judgement boundary rather than a
    # mechanism. That is a real and remaining use.
    #
    # **What it no longer does is bypass CI, and the arm used to be reasoned from that.** It said this was
    # consistent with *whether CI is green stays a human's call*; `require_ci_green` refuses a red or
    # unfinished rollup before `gh` is reached, so passing `--admin` to force a red merge through this path
    # does not work and is not meant to. Use the web UI for that, and meet no gate at all — which is the
    # declared bound the header names, not a loophole this arm opens.
    #
    # It is the only flag admitted here, and the criterion above is why. `--delete-branch` shared this arm
    # with no sentence of its own: it changes neither whether the merge proceeds, nor what it records, nor
    # when it happens — it is a **post-merge act**, and the one admitted argument with an irreversible side
    # effect. It is refused below.
    --admin)
        passthrough+=("$1")
        shift
        ;;
    # The arms below decide nothing the catch-all would not — every one of these is unlisted, so it is refused
    # either way. They exist to say WHY, because a refusal an operator cannot act on is a refusal they work
    # around. Each pattern covers gh's glued and equals forms too: `-t`, `-t=x` and `-tx` are one flag.
    --subject=* | --body-file=* | --body | --body=* | -t* | -F* | -b*)
        refuse "$1" \
            "the message this wrapper hands to the gate is the message the merge records, and \
gh takes the last spelling of a repeated flag — so this would have the gate judge one message and the merge \
write another. Pass the subject as \`--subject <text>\` and the body as \`--body-file <path>\`"
        ;;
    --repo | --repo=* | -R*)
        refuse "$1" \
            "this wrapper reads the title, the pull request number, the head it pins the merge \
to and the live commit subjects from the repository it is run in — and refuses to run at all unless its gate \
comes from that same worktree. A repository selector would move the evidence off the one tree all of it is \
required to share, so the gate would judge one pull request and the merge record another. Run it from a \
checkout of the repository whose pull request you are merging"
        ;;
    --merge | --rebase | --squash | -m* | -r* | -s*)
        refuse "$1" \
            "a development pull request lands on a release branch as one squash, and this \
gate judges that squash's message"
        ;;
    --delete-branch | -d)
        refuse "$1" \
            "it is not part of the merge, it is an act **after** it — and the one with an \
effect no rerun undoes. Deleting a branch another pull request targets auto-closes that pull request, and \
GitHub refuses to reopen it once the branch is gone and the head has moved; this repository has paid for that \
already. Every other admitted argument changes whether the merge proceeds; none changes what happens \
afterwards, which is the criterion this allowlist states. Delete the branch yourself once you can see nothing \
was stacked on it"
        ;;
    --auto)
        refuse "$1" \
            "it does not merge now, it merges LATER — gh: \"Automatically merge only after \
necessary requirements are met\". The gate judged this body against the pull request's live commit subjects as \
they are at this moment; a commit pushed before the deferred merge lands changes that set while the captured \
subject and body do not, so what gets recorded would no longer be what was judged. Merge when the requirements \
are met, and this wrapper will judge the set that exists then"
        ;;
    --disable-auto)
        refuse "$1" \
            "it is not a merge — it turns auto-merge off and returns. This wrapper would run \
the gate, reach gh, and exit 0 having merged nothing, reporting success for an act that did not happen. Run \
\`gh pr merge --disable-auto\` directly; there is no record for a gate to hold"
        ;;
    --match-head-commit | --match-head-commit=*)
        refuse "$1" \
            "this wrapper supplies it itself, pinning the head the gate actually read its \
evidence from, and gh takes the last spelling of a repeated flag — so a caller-supplied SHA would replace \
exactly the link this guard exists to make"
        ;;
    --author-email | --author-email=* | -A*)
        refuse "$1" \
            "this wrapper holds what the merge is about to record, and the author it records \
is part of that"
        ;;
    *)
        refuse "$1" \
            "this wrapper forwards only the flags that change whether the merge may proceed, \
never what it would record, and this is not one of them. An argument it does not know is refused rather than \
passed on, because the record it stands in front of cannot be repaired"
        ;;
    esac
done

if [[ -z $body_file ]]; then
    usage_error "the squash body is what the gate judges and what the merge records, so \`--body-file\` is \
required rather than defaulted"
fi
if [[ ! -f $body_file ]]; then
    cannot_judge "cannot read the body file $body_file"
fi
# Read ONCE, here, guarded — and hand the value to the gate rather than the path.
#
# `-f` says a regular file is there; it does not say this process may read it. The read used to happen inside the
# gate's own invocation as `TIANHENG_MERGE_BODY=$(cat -- "$body_file")`, unguarded: measured, an unreadable file
# left that variable EMPTY and the gate then judged an empty body, which it refuses as a violation — *the squash
# body is empty*. So a file this wrapper could not read was reported to the operator as a body they had written
# wrongly. Reading once also closes the window between the check and the use, in which the file could have gone.
body=$(cat -- "$body_file") || cannot_judge \
    "cannot read the body file $body_file, so whether this body is the record the merge should carry cannot be \
decided — which is not the same fact as a body that disagrees"


# ONE repository identity, resolved once and passed to every call below.
#
# The endpoint already named a repository — implicitly, through a placeholder gh expands from the working
# directory — while the three `gh pr` calls named whichever the selector resolved to. Four references defaulting
# to the same place is agreement by circumstance; naming it once is agreement by construction, and it is the
# shape this wrapper's own contract asks for: the accepted selector, the live commit set and the merge must be
# one pull request.
repository=$(gh repo view --json nameWithOwner --jq .nameWithOwner) || cannot_judge \
    "cannot resolve which repository this checkout is, so the selector, the live commit set and the merge \
cannot be shown to name one pull request"

# Every acquisition below is guarded. An unguarded `var=$(gh …)` under `set -e` exits with the TOOL's status and
# only the tool's stderr — measured, a failing commits read left this wrapper exiting 91 with nothing of its own
# said, so the class it reports was neither of the two it defines and the operator got gh's words for a fact
# about this wrapper. The same shape as a value-taking flag failing on its shift arithmetic, one layer out.
# ONE pull request identity, resolved once and passed to every call below.
#
# `$pr` is a SELECTOR — a number, a branch, or a URL — and each `gh pr` call resolved it again. Four
# resolutions landing on one pull request is agreement by circumstance, which is the shape this wrapper closed
# for the repository a few lines up and left open for the pull request itself. `--match-head-commit` does not
# close it: two open pull requests from one head branch to different bases carry the same head OID, so the pin
# cannot tell them apart. This call is the only one that may take the selector, because resolving it is what it
# is for.
pr_number=$(gh pr view "$pr" --repo "$repository" --json number --jq .number) || cannot_judge \
    "cannot read pull request $pr's number, so the accepted selector, the live commit set and the merge cannot \
be shown to name one pull request"
if [[ ! $pr_number =~ ^[1-9][0-9]*$ ]]; then
    cannot_judge \
        "cannot resolve $pr to one pull request number; the evidence reads and the merge require its canonical \
identity"
fi
title=$(gh pr view "$pr_number" --repo "$repository" --json title --jq .title) || cannot_judge \
    "cannot read pull request $pr_number's title, so whether the subject is that title cannot be decided"
: "${subject:=$title}"
# The base the squash lands on. `AGENTS.md` states the one message exception as the release-branch-to-`main`
# squash, and a subject is not a destination — deciding it on the subject alone made the exception's identity a
# spelling any branch could write. The gate takes the base as evidence, exactly as it takes the title.
base=$(gh pr view "$pr_number" --repo "$repository" --json baseRefName --jq .baseRefName) || cannot_judge \
    "cannot read pull request $pr_number's base branch, so whether this squash is the one message exception \
cannot be decided"
# And the branch it comes from. The contract names **both** endpoints — the release-branch-to-`main` squash —
# and taking only the destination left the exception reachable from any branch onto `main`.
head_branch=$(gh pr view "$pr_number" --repo "$repository" --json headRefName --jq .headRefName) || cannot_judge \
    "cannot read pull request $pr_number's head branch, so whether this squash is the one message exception \
cannot be decided"
# The head the gate is about to read its evidence from, captured BEFORE the commit set — the order is the guard.
#
# What this closes: the gate judges the body against the pull request's commit subjects as they are while it
# runs, and the merge happens afterwards. A commit pushed in between changes the set the body must equal, and
# nothing noticed. `--match-head-commit` is gh's answer — the merge is refused unless the head still matches —
# and this wrapper now supplies it rather than admitting it, so what gets pinned is the head the evidence came
# from and not a SHA a caller chose.
#
# **Read before the commits, not after.** Capture the head first and a push in between leaves the commit set
# ahead of the pinned head, so gh refuses: fails closed. Capture it after and the pinned head would include the
# new commit while the gate judged the older set, so the merge would proceed and record a body missing it: fails
# open. Same two calls, opposite guarantees.
head=$(gh pr view "$pr_number" --repo "$repository" --json headRefOid --jq .headRefOid) || cannot_judge \
    "cannot read pull request $pr_number's head commit, so the merge cannot be pinned to the head the gate \
reads from"
if [[ ! $head =~ ^[0-9a-f]{7,40}$ ]]; then
    cannot_judge "cannot read the pull request's head commit, so the merge could not be pinned to the head \
this gate read its evidence from — and an unpinned merge may record a body that no longer matches the commits"
fi

# The gate. A failure aborts before the merge, which is the point: the record below cannot be amended.

# The channel the gate reports its refusal class on, and the class that means a disagreement.
#
# Both are held against `kanhe::verdict_channel` by `crates/kanhe/tests/gate_exit_classes.rs`, so neither the
# variable name nor the class spelling can drift from the gate's side.
#
# **This replaced grepping the gate's output.** Searching stdout for `(Violation)` put the parentheses in this
# script and the variant name in Rust — two owners for one token — and measured, changing the gate's format
# string left every direction green while this pattern matched nothing, so every violation would have reported as
# unjudged. It also searched a stream carrying arbitrary tooling output, where a class could be read from text no
# judgement wrote. A file the gate writes only when it has a verdict makes *absent* mean unjudged by
# construction.
GATE_VIOLATION_CLASS=Violation
# The class a gate reports when it JUDGED AND AGREED, and the guard that requires it on the success path.
#
# **`require_one_pass` answers a different question and cannot cover this one.** It asks *did the selected
# test pass* — which a harness that returned without judging satisfies, and one did: a subject supplied as
# bytes the gate could not read took an arm that printed "not judged" and returned, so `1 passed` was true
# and nothing had been judged. The two guards catch different states and both stay: `require_one_pass` sees a
# renamed test (nothing ran), this sees a test that ran, passed, and reached no verdict.
#
# The gate now writes the channel on its clean arm too, so *absent on success* means unjudged by
# construction rather than by a wrapper remembering to check. Held against `kanhe::verdict_channel::CLEAN` by
# `crates/kanhe/tests/gate_exit_classes.rs`, so neither spelling can drift from the gate's side.
GATE_CLEAN_CLASS=Clean

require_a_verdict() {
    local reached=""
    if [[ -f $verdict_file ]]; then
        reached=$(cat -- "$verdict_file") || reached=""
    fi
    if [[ $reached != "$GATE_CLEAN_CLASS" ]]; then
        cannot_judge \
            "the gate ran and passed without reaching a verdict — the channel carries ${reached:-nothing}, and a run that judged nothing is not a run that agreed. This is the class a passing test cannot distinguish on its own, which is why it is read rather than inferred"
    fi
}

# The other suite: what CI said about this pull request's head.
#
# **Measured, this wrapper merged nineteen consecutive red runs.** Every local gate reported green each time —
# the Definition of Done is the LOCAL pre-flight list and says so, and CI runs a superset of it. One job in
# that superset, the MSRV build, is not in the local list because it installs a toolchain and rebuilds the
# workspace; a single `if let … && …` that the default toolchain accepts and 1.85 refuses was therefore red in
# CI and green here, through nineteen merges, until someone ran the job by hand.
#
# So the wrapper reads the same answer it already reads for its own gate: a verdict, not an inference. Three
# states, and the middle one is why a boolean would not do — a run still in flight is not a run that failed,
# and merging on "not success" would refuse a pull request whose checks simply have not finished.
#
# `--json` rather than the human table, because the table is a display this wrapper does not own. An empty
# conclusion is a check still running; a missing rollup is a head no workflow has claimed, which is its own
# cannot-judge rather than a pass — a pull request nothing has checked is not a pull request that checked out.
require_ci_green() {
    # **One read, three states derived from it.** The first form asked two independent jq filters about
    # `statusCheckRollup`, and a pull request with *no checks at all* is a value neither can produce: the
    # disagreement filter answers the empty string and the unfinished filter answers zero, so nothing refused
    # and the merge ran. That is the same false-negative direction as the nineteen red merges this guard was
    # written for, reintroduced by the guard — and unreachable by construction rather than by omission, which
    # is why the states are now read off one answer instead of asked for separately.
    #
    # `\t` between the conclusion and the name because a check name carries spaces (`MSRV (rust-version)`),
    # and stderr is left on the terminal rather than folded into the value: the first form captured it with
    # `2>&1` and then tested the value for emptiness, so a notice on a SUCCESSFUL call would have been
    # reported as `CI has not agreed about this pull request: <notice>` — a refusal naming a check that does
    # not exist. The pending read handled the same stream the opposite way, discarding it entirely, and that
    # disagreement between the two is what made the pair worth reading as one.
    local rollup
    # **The rollup is a UNION of two node shapes, and reading one of them is not reading it.** GitHub's
    # `StatusCheckRollupContext` is `CheckRun | StatusContext`: a `CheckRun` carries `.conclusion`/`.name`,
    # and a `StatusContext` — an external commit status — carries `.state`/`.context` and neither of the
    # first two. So the earlier filter answered `""` and `"?"` for every commit status, and a FAILED one was
    # classified as *unfinished* and reported as `these checks have not finished: ?` — a refusal naming a
    # check that does not exist. That is verbatim the defect `require_ci_green`'s stderr-capture note already
    # records fixing once, where folding a successful call's notice into the value produced `CI has not
    # agreed about this pull request: <notice>`; the same wrong sentence, reached through the node shape the
    # filter never read.
    #
    # Fail-closed either way, so this was latent rather than live: this repository runs GitHub Actions and
    # produces no commit statuses. Latent is not fixed — a check reported under a name nobody can find is
    # what sends an operator looking for the wrong thing, and one added integration changes the class.
    #
    # `.state` and `.context` are the fallbacks, so one filter reads both shapes and every classification
    # below is over one vocabulary.
    rollup=$(gh pr view "$pr_number" --repo "$repository" --json statusCheckRollup \
        -q '[.statusCheckRollup[]? | ((.conclusion // .state // "") + "\t" + (.name // .context // "?"))] | join("\n")') \
        || cannot_judge \
            "cannot read what CI said about this pull request, which is not the same fact as CI having agreed"

    if [[ -z ${rollup//[[:space:]]/} ]]; then
        cannot_judge \
            "no workflow has claimed this head, so nothing has checked this pull request — which is not the \
same fact as a suite that agreed"
    fi

    # Split by parameter expansion rather than by `read`'s field splitting: a tab is IFS whitespace, so
    # `IFS=$'\t' read -r conclusion name` strips the LEADING tab of an unfinished check's line and reads its
    # name as the conclusion — measured, the unfinished direction reported a disagreement.
    local disagreeing="" unfinished="" silent="" line conclusion name
    while IFS= read -r line; do
        [[ -z ${line//[[:space:]]/} ]] && continue
        conclusion=${line%%$'\t'*}
        name=${line#*$'\t'}
        # One vocabulary over both node shapes. `SUCCESS` is shared; `NEUTRAL`/`SKIPPED` are a `CheckRun`'s
        # conclusions; `PENDING`/`EXPECTED` are a `StatusContext`'s states, and an empty field is a
        # `CheckRun` still running. `FAILURE`, `ERROR`, `CANCELLED`, `TIMED_OUT` and anything unlisted fall
        # to the catch-all, which is the direction a class this script has not met must fall.
        #
        # **`EXPECTED` is unfinished, not agreeing.** GitHub's own meaning is *a status is expected* —
        # required and not yet posted — so reading it as agreement would merge past a required status that
        # never arrived, which is the false-negative direction this whole guard exists to close. Classified
        # with `PENDING`, whose operator action is identical: wait for it.
        #
        # **`NEUTRAL` and `SKIPPED` are their own class, and they agreed with nothing.** They sat beside
        # `SUCCESS` with no measurement while the arm above was reasoned about at length — and the same
        # argument covers them: a check that did not run produced no evidence, so reading it as agreement
        # merges past whatever it would have said. That holds whatever the workflow looks like, which is why
        # neither this arm nor its refusal says anything about the workflow's shape.
        #
        # **The refusal used to.** It told the operator that no job in this repository's workflow carries
        # `if:`, `needs:`, `paths:` or `continue-on-error:`, so a skip could only mean interference. True when
        # written, and a sentence that goes stale the moment someone adds one — at which point the wrapper is
        # telling an operator something false about the tree they are standing in. A diagnostic states what to
        # do about the state it met; a claim about the world needs something holding it, and this one bought
        # nothing the classification did not already have.
        #
        # Their own arm rather than the unfinished one, because the operator action differs: an unfinished
        # check is waited for, and a skipped one is investigated. When a job legitimately may skip — a
        # path filter, say — move it back beside `SUCCESS` and state which job and why that skip is
        # evidence, the way the `EXPECTED` paragraph above states its own.
        case $conclusion in
        SUCCESS) ;;
        "" | PENDING | EXPECTED) unfinished+="${unfinished:+, }${name}" ;;
        NEUTRAL | SKIPPED) silent+="${silent:+, }${name} (${conclusion})" ;;
        *) disagreeing+="${disagreeing:+, }${name} (${conclusion})" ;;
        esac
    done <<<"$rollup"

    if [[ -n $disagreeing ]]; then
        cannot_judge \
            "CI has not agreed about this pull request: $disagreeing. A local Definition of Done is the \
pre-flight list and CI runs a superset of it, so a green local run is not a green suite — measured, this \
wrapper merged nineteen consecutive red runs on exactly that difference"
    fi
    if [[ -n $silent ]]; then
        cannot_judge \
            "these checks produced no evidence: $silent. A check that did not run agreed with nothing, so \
this is not a suite that agreed. Look at why it did not run. If a job in this workflow may now legitimately \
skip, move that conclusion back beside \`SUCCESS\` in this script and record which job it is and why its skip \
is evidence — the way the \`EXPECTED\` classification beside it states its own reason"
    fi
    if [[ -n $unfinished ]]; then
        cannot_judge \
            "these checks have not finished: $unfinished. A run still in flight is not a run that agreed; \
wait for it rather than merging on its silence"
    fi
}

# **A pull request that changes no file records a message about work that is not in it.**
#
# Measured: this wrapper merged one. The content was committed onto the release branch itself, the branch the
# pull request named still pointed at an already-merged commit, and the pull request's diff was therefore
# empty — so every guard here was satisfied. The live commit set was non-empty, the gate judged the message
# against it, CI was green because nothing had changed, `--match-head-commit` pinned a head that was real,
# and the squash recorded a message asserting seven repairs across five files while carrying none of them.
# The only copy of the work was then discarded by a reset to origin.
#
# Nothing anywhere holds a merged squash's tree against its parent, and 勘合's own name is a document made in
# two halves proven genuine by fitting them together. This is the case where one half is empty, and it is
# read rather than inferred, like the gate's verdict and CI's beside it.
require_changed_files() {
    local changed
    changed=$(gh pr view "$pr_number" --repo "$repository" --json changedFiles -q '.changedFiles') \
        || cannot_judge \
            "cannot read how many files this pull request changes, which is not the same fact as a pull \
request that changes some"
    if [[ ! $changed =~ ^[0-9]+$ ]]; then
        cannot_judge \
            "the changed-file count read as \`${changed}\`, which is not a number — a count this wrapper \
cannot read is not a count of zero"
    fi
    if ((changed == 0)); then
        cannot_judge \
            "this pull request changes no file, so the message about to be recorded describes work that is \
not in it. Check that the branch you pushed is the branch holding the commits"
    fi
}

verdict_file=$(mktemp) || cannot_judge \
    "cannot open a file for the gate to report its refusal class on, so a failing gate could not be told from \
an input it could not read"
trap 'rm -f "$verdict_file"' EXIT

# `libtest` exits 0 when `--exact` selects no test — measured, an unknown name reports `0 passed` and exits 0,
# and an `#[ignore]`d one reports `0 passed; 1 ignored` and exits 0 too. So the exit status answers *did the
# selected tests pass* while the question here is *did the gate judge this act*, and those differ exactly when
# a rename has quietly happened. Require the run to say it judged one thing.
#
# Asserted here rather than inside the gate: a renamed or silenced test cannot report that it did not run.
#
# A gate that did not run is a cannot-judge, so this exits 2. It reads as the sharpest case of the class: the
# message says *the gate did not run* in so many words, and reporting that as a violation names a disagreement
# no judgement ever formed.
require_one_pass() {
    local output=$1
    # A here-string, not a pipe. `grep -q` exits at its first match, and under `set -o pipefail` the
    # `printf` upstream takes SIGPIPE and that becomes the pipeline's status — so this would report *the
    # gate did not run* for a closed pipe, immediately before an irreversible act. Measured: with the token
    # at the end of a 405 KB stream, which is where a `cargo test` summary sits, 0 of 8 runs returned
    # non-zero; with the same token near the start, 8 of 8 did. Both wrappers were holding by where the
    # token happened to sit, which nothing declares and nothing keeps true.
    if ! grep -qE 'test result: ok\. 1 passed' <<< "$output"; then
        printf '%s\n' "$output" >&2
        cannot_judge \
            "the gate did not run — its invocation selected no passing test, so the name in this script no longer names one. libtest exits 0 for a filter that matches nothing, which is why this is checked rather than trusted"
    fi
}

# The pull request's own LIVE commit subjects, so the gate can ask whether this body *is* their concatenation
# rather than whether it looks like one. Local remote-tracking refs can lag the pull request or carry no fork
# head at all, and a stale subset makes a default body containing the missing subjects look unrelated and pass.
# The commits endpoint returns the full message; take its first line rather than `messageHeadline`, which
# truncates long subjects. `--paginate` keeps a large pull request one set rather than its first page.
commits=$(gh api --paginate "repos/$repository/pulls/$pr_number/commits" \
    --jq '.[].commit.message | split("\n")[0]') || cannot_judge \
    "cannot read the commit subjects of pull request $pr_number, so whether this body is their concatenation \
cannot be decided"
if [[ -z ${commits//[[:space:]]/} ]]; then
    cannot_judge \
        "cannot read any commit subjects from pull request $pr_number; an empty live set is not evidence about its body"
fi

gate_output=$(TIANHENG_GATE_VERDICT=$verdict_file \
    TIANHENG_MERGE_SUBJECT=$subject \
    TIANHENG_MERGE_TITLE=$title \
    TIANHENG_MERGE_COMMITS=$commits \
    TIANHENG_MERGE_BODY=$body \
    TIANHENG_MERGE_BASE=$base \
    TIANHENG_MERGE_HEAD=$head_branch \
    cargo test --manifest-path "$repo/Cargo.toml" -p kanhe --test merge_message \
    -- --exact the_squash_message_is_the_pull_request_it_records 2>&1) || {
    printf '%s\n' "$gate_output" >&2
    # The class the gate reported, on the channel it was given. Absent, empty or anything else is a run that
    # reached no verdict — a compile error included — and that is not a disagreement.
    verdict=""
    if [[ -f $verdict_file ]]; then
        verdict=$(cat -- "$verdict_file") || verdict=""
    fi
    if [[ $verdict == "$GATE_VIOLATION_CLASS" ]]; then
        exit 1
    fi
    exit 2
}
require_one_pass "$gate_output"
require_a_verdict
require_ci_green
require_changed_files

# The title and the base are re-read, because each is the OTHER END OF A RELATION rather than a value being
# recorded.
#
# The wrapper's judged inputs divide by that one question, and the division is the criterion rather than a
# tally: an input the merge RECORDS travels as the value the gate saw, and an input the merge is JUDGED
# AGAINST has to still hold when the merge happens. The body is recorded, so it travels as a value and a
# rewrite between the two cannot reach the record. The commit set is one end of `body == their
# concatenation`, so a push in between makes the judged relation false and `--match-head-commit` refuses.
#
# `subject == title` is a relation, and so is the release exception's `base` — `gh pr merge` takes no base of
# its own and lands wherever the pull request points at merge time, so a base edited after the gate ran
# leaves an approved empty-body release message landing on a destination nothing judged, and carries the
# exception to a squash that is not one. Both were captured once and never looked at again. Sorted by the
# wrapper's own criterion, both were filed on the wrong side; the title was moved first and the base was left
# behind by the same reading that moved it.
#
# The HEAD BRANCH is the exception, and deliberately so. It is the exception's other endpoint, but GitHub
# offers no way to change an existing pull request's head, and `--match-head-commit` already pins the head
# object. A re-read of it could be made to refuse only against a fixture, never against the tool — a guard
# nothing has been seen to refuse, which `AGENTS.md` does not count as a guard.
#
# **This narrows the window; it does not close it, and the difference is the point.** `--match-head-commit`
# is decided by the server, atomically. `gh` offers no `--match-title`, so a client-side re-read shrinks the
# exposure from a whole `cargo test` — minutes on a cold target directory — to one API call, and a change
# inside that call still lands. The residue is a declared bound of `repository-checks`, beside the one for a
# merge made outside this wrapper, rather than a limit this comment implies away.
title_now=$(gh pr view "$pr_number" --repo "$repository" --json title --jq .title) || cannot_judge \
    "cannot re-read pull request $pr_number's title after the gate, so whether the subject the gate approved \
is still that title cannot be decided — which is not the same fact as a subject that disagrees"
# **A moved title is a cannot-judge, not a disagreement**, and the exit-class check refused the first draft
# of this guard for saying otherwise. The gate did not find the subject wrong: it found it right, against a
# title that no longer exists, so what this wrapper has is a verdict about a vanished input. That is the
# class `merge_message_gate::judge` already gives an unavailable title — "which is not the same fact as a
# subject that disagrees" — and the construction that reserves `1` for the gate's own verdict arm is what
# caught the misfiling.
if [[ $title_now != "$title" ]]; then
    cannot_judge "the pull request's title changed while the gate ran. It judged \"$title\", the title is \
now \"$title_now\", so the verdict in hand is about a title that no longer exists rather than about a \
subject that disagrees. Re-run this wrapper and it will judge the title that exists now"
fi

# The base, on the same terms and for the same reason. A read failing here is a cannot-judge on its own: not
# knowing where the squash lands is not the same fact as knowing it moved.
base_now=$(gh pr view "$pr_number" --repo "$repository" --json baseRefName --jq .baseRefName) || cannot_judge \
    "cannot re-read pull request $pr_number's base branch after the gate, so whether this squash is still \
the one the gate judged cannot be decided"
if [[ $base_now != "$base" ]]; then
    cannot_judge "the pull request's base branch changed while the gate ran. It judged a squash onto \
\"$base\", the base is now \"$base_now\", so the verdict in hand is about a destination this merge will not \
use — and the one message exception is named by where the squash lands, not by how its subject reads. \
Re-run this wrapper and it will judge the base that exists now"
fi

# Removed here, not left to the trap. An EXIT trap does not run when `exec` replaces the shell image —
# measured, `bash -c 'trap "echo T" EXIT; exec true'` prints nothing while the same script without `exec` prints
# `T`. So the trap fired on every path where nothing happened and was skipped on the one path that completes the
# act: three successful runs left three empty files in `$TMPDIR`, measured against an isolated one. The trap
# stays, because it is what covers the failure paths; `exec` stays, because the tool's exit status becoming this
# script's is deliberate.
rm -f "$verdict_file"

# The body travels as the VALUE the gate judged, never as the path it was read from.
#
# `--body-file` would have gh open the file again, after the gate has run — and what sits between the two is a
# whole `cargo test`, minutes of it on a cold target directory. A rewrite in that window is recorded by a merge
# that judged something else, permanently: the pull request's merge record cites the squash commit's hash, so
# amending it afterwards decouples the two.
#
# The other three judged inputs already travelled this way and nothing said they were one set: the subject is a
# value, the repository is resolved once and named on every call, the head is captured before the commit set and
# pinned with `--match-head-commit`, through which the live subjects are pinned too. This is the local half of
# that pin — a pull request that moved is refused, and an input that moved on disk is never read a second time.
#
# Safe against a later occurrence only because the allowlist refuses a CALLER's body flag in every spelling, so
# `passthrough` can never carry one: gh takes the last spelling of a repeated flag, and this argument is spliced
# before it. That safety belongs to the allowlist rather than to the order these are written in.
#
# `--body` over a wrapper-owned temporary file, which would close the same race: such a file must outlive the
# `exec` for gh to read it, so it could not be removed beforehand and no EXIT trap survives an `exec` — which is
# the leak closed by removing the file just before the `exec`, reintroduced to fix a different defect. A
# value in `argv` has an `ARG_MAX` ceiling a path does not, and that ceiling fails loud with `E2BIG` before the merge rather than
# recording something wrong.
#
# `passthrough` may be empty, and `"${empty[@]}"` under `set -u` is an unbound variable before bash 4.4 —
# where this wrapper would abort through the ERR trap reporting "an unguarded command failed", a sentence
# about the wrong cause, on the invocation with no passthrough flags that is the ordinary one. The `+` form
# is used rather than a version check, so no minimum has to be declared anywhere and kept in step.
exec gh pr merge "$pr_number" --repo "$repository" --squash --subject "$subject" --body "$body" \
    --match-head-commit "$head" ${passthrough[@]+"${passthrough[@]}"}
