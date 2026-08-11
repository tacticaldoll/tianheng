#!/usr/bin/env bash
#
# The sanctioned merge path: the squash-message gate, then `gh pr merge --squash`.
#
# Why a wrapper rather than a documented rule. `AGENTS.md` already says the squash subject is the pull
# request's title with no auto-appended `(#N)`; nine subjects in this repository's history carry that serial
# anyway, the most recent on the commit that landed a check for a requirement enforced by nothing. The
# failure mode is not disagreement about the rule — it is one string typed at the one moment nothing can be
# undone. A merged squash cannot be repaired: amending it changes its hash, and the pull request's merge
# record cites that hash, so the two would name different things afterwards.
#
# Nothing here carries a verdict. The judgement is `crates/kanhe/tests/merge_message.rs`, a Rust repository check
# like every other one judging this repository; this script gathers the inputs and refuses to reach `gh`
# without it.
#
# What stays outside. WHETHER to merge, and whether CI is green, remain a human's call — this wrapper holds
# only what the merge is about to record. A merge made in the GitHub web UI reaches no wrapper at all; that is
# a declared bound, not an oversight.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

usage() {
    printf 'usage: %s <pr-number> --body-file <path> [--subject <text>] [gh args…]\n' "${0##*/}" >&2
    printf '  The subject defaults to the pull request title, which is what the rule requires anyway.\n' >&2
}

pr=${1:-}
if [[ -z $pr || $pr == -* ]]; then
    usage
    exit 2
fi
# A URL names its own repository, and this wrapper reads its evidence from several places. `gh pr view` and
# `gh pr merge` would follow the URL while the live-commits endpoint is built from a repository reference of its
# own — so a cross-repository URL has the gate judge one pull request and the merge record another, which is the
# same hole a `--repo` flag opened and this positional selector reopens. A number or a branch name names no
# repository and resolves against the one being pinned below, so both stay accepted.
if [[ $pr == http://* || $pr == https://* ]]; then
    printf 'merge message: %s\n' \
        "refusing a pull-request URL: it names its own repository, while this wrapper reads the live commit \
set from the repository it is run in. Pass the number, or run it from a checkout of that repository" >&2
    exit 2
fi
shift

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
# Chosen here rather than at each site because the split survived precisely by being spelled out twenty times.
cannot_judge() {
    printf 'merge message: %s\n' "$1" >&2
    exit 2
}

subject=""
body_file=""
passthrough=()
# A value-taking flag given no value is an OBSERVABLE misconfiguration, so it fails loud. Before this it failed
# silent: `shift 2` with one argument left returns non-zero, `set -e` took that as the exit, and the wrapper
# stopped with no output at all — while every other refusal below prints `merge message: …`. Reproduced by
# running the wrapper with `--subject` last: empty output, exit 1. Validating before shifting is what keeps the
# arithmetic from becoming the diagnostic.
require_value() {
    if (($1 < 2)); then
        printf 'merge message: %s\n' \
            "refusing \`$2\` with no value: this wrapper reads every value as the argument after its flag, so \
pass it that way or drop the flag" >&2
        usage
        exit 2
    fi
}

while (($#)); do
    case $1 in
    --subject)
        require_value "$#" "$1"
        subject=$2
        shift 2
        ;;
    --body-file)
        require_value "$#" "$1"
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
    # `--subject` and `--body-file` written below? The second question was missing for a window, and the sibling
    # publish wrapper paid for it: it admitted `--package` beside an unconditional `--workspace`, which cargo
    # silently maps to *all packages*. Here the same question refuses `--auto` and `--disable-auto`: one defers
    # the merge past the evidence the gate read, the other is not a merge at all.
    #
    # Forwarded are the flags that change whether the merge may proceed, never what it would record, and never
    # WHEN it happens relative to the evidence.
    #
    # ONE spelling each, values as separate arguments. Parsing gh's glued and equals forms is what let the
    # short forms through; refusing those costs an argument's worth of typing and removes the parsing question.
    # `--admin` bypasses the branch's required checks. That is consistent with what this wrapper already
    # declares: WHETHER to merge, and whether CI is green, stay a human's call — this holds only what the
    # merge is about to record.
    --admin | --delete-branch)
        passthrough+=("$1")
        shift
        ;;
    # The arms below decide nothing the catch-all would not — every one of these is unlisted, so it is refused
    # either way. They exist to say WHY, because a refusal an operator cannot act on is a refusal they work
    # around. Each pattern covers gh's glued and equals forms too: `-t`, `-t=x` and `-tx` are one flag.
    --subject=* | --body-file=* | --body | --body=* | -t* | -F* | -b*)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: the message this wrapper hands to the gate is the message the merge records, and \
gh takes the last spelling of a repeated flag — so this would have the gate judge one message and the merge \
write another. Pass the subject as \`--subject <text>\` and the body as \`--body-file <path>\`" >&2
        exit 2
        ;;
    --repo | --repo=* | -R*)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: this wrapper reads the title, the pull request number, the live commit subjects \
and the gate from the repository it is run in, so a repository selector would judge one pull request and merge \
another. Run it from a checkout of the repository whose pull request you are merging" >&2
        exit 2
        ;;
    --merge | --rebase | --squash | -m* | -r* | -s*)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: a development pull request lands on a release branch as one squash, and this \
gate judges that squash's message" >&2
        exit 2
        ;;
    --auto)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: it does not merge now, it merges LATER — gh: \"Automatically merge only after \
necessary requirements are met\". The gate judged this body against the pull request's live commit subjects as \
they are at this moment; a commit pushed before the deferred merge lands changes that set while the captured \
subject and body do not, so what gets recorded would no longer be what was judged. Merge when the requirements \
are met, and this wrapper will judge the set that exists then" >&2
        exit 2
        ;;
    --disable-auto)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: it is not a merge — it turns auto-merge off and returns. This wrapper would run \
the gate, reach gh, and exit 0 having merged nothing, reporting success for an act that did not happen. Run \
\`gh pr merge --disable-auto\` directly; there is no record for a gate to hold" >&2
        exit 2
        ;;
    --match-head-commit | --match-head-commit=*)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: this wrapper supplies it itself, pinning the head the gate actually read its \
evidence from, and gh takes the last spelling of a repeated flag — so a caller-supplied SHA would replace \
exactly the link this guard exists to make" >&2
        exit 2
        ;;
    --author-email | --author-email=* | -A*)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: this wrapper holds what the merge is about to record, and the author it records \
is part of that" >&2
        exit 2
        ;;
    *)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: this wrapper forwards only the flags that change whether the merge may proceed, \
never what it would record, and this is not one of them. An argument it does not know is refused rather than \
passed on, because the record it stands in front of cannot be repaired" >&2
        exit 2
        ;;
    esac
done

if [[ -z $body_file ]]; then
    usage
    exit 2
fi
if [[ ! -f $body_file ]]; then
    cannot_judge "cannot read the body file $body_file"
fi

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
title=$(gh pr view "$pr" --repo "$repository" --json title --jq .title) || cannot_judge \
    "cannot read pull request $pr's title, so whether the subject is that title cannot be decided"
: "${subject:=$title}"
pr_number=$(gh pr view "$pr" --repo "$repository" --json number --jq .number) || cannot_judge \
    "cannot read pull request $pr's number, so the live commits endpoint cannot be built from its canonical identity"
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
head=$(gh pr view "$pr" --repo "$repository" --json headRefOid --jq .headRefOid) || cannot_judge \
    "cannot read pull request $pr's head commit, so the merge cannot be pinned to the head the gate reads from"
if [[ ! $head =~ ^[0-9a-f]{7,40}$ ]]; then
    cannot_judge "cannot read the pull request's head commit, so the merge could not be pinned to the head \
this gate read its evidence from — and an unpinned merge may record a body that no longer matches the commits"
fi
if [[ ! $pr_number =~ ^[1-9][0-9]*$ ]]; then
    cannot_judge \
        "cannot resolve $pr to one pull request number; the live commits endpoint requires its canonical identity"
fi

# The gate. A failure aborts before the merge, which is the point: the record below cannot be amended.

# The token the gate renders for a disagreement, as opposed to an input it could not judge. Pinned against
# `refusal::Kind`'s own `Debug` rendering by a repository check, because a wrapper grepping for a string the
# gate prints is two places that must agree.
GATE_VIOLATION_TOKEN=Violation

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
    if ! printf '%s' "$output" | grep -qE 'test result: ok\. 1 passed'; then
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

gate_output=$(TIANHENG_MERGE_SUBJECT=$subject \
    TIANHENG_MERGE_TITLE=$title \
    TIANHENG_MERGE_COMMITS=$commits \
    TIANHENG_MERGE_BODY=$(cat -- "$body_file") \
    cargo test --manifest-path "$repo/Cargo.toml" -p kanhe --test merge_message \
    -- --exact the_squash_message_is_the_pull_request_it_records 2>&1) || {
    printf '%s\n' "$gate_output" >&2
    # The gate prints its own class — `merge message (Violation): …` or `(CannotJudge): …` — so the wrapper
    # reads it rather than guessing. Anything else is a run that reached no verdict at all, a compile error
    # included, and that is not a disagreement: the default is the unjudged class, which errs toward telling
    # the operator to look at the output rather than at their message.
    #
    # `GATE_VIOLATION_TOKEN` is held against `refusal::Kind`'s own rendering by
    # `crates/kanhe/tests/gate_exit_classes.rs`, so this matcher and the gate cannot drift apart.
    if printf '%s' "$gate_output" | grep -q "($GATE_VIOLATION_TOKEN)"; then
        exit 1
    fi
    exit 2
}
require_one_pass "$gate_output"

exec gh pr merge "$pr" --repo "$repository" --squash --subject "$subject" --body-file "$body_file" \
    --match-head-commit "$head" "${passthrough[@]}"
