#!/usr/bin/env bash
#
# The sanctioned merge path: the squash-message gate, then `gh pr merge --squash`.
#
# Why a wrapper rather than a documented rule. `AGENTS.md` already says the squash subject is the pull
# request's title with no auto-appended `(#N)`; nine subjects in this repository's history carry that serial
# anyway, the most recent on the commit that landed a reaction for a requirement enforced by nothing. The
# failure mode is not disagreement about the rule — it is one string typed at the one moment nothing can be
# undone. A merged squash cannot be repaired: amending it changes its hash, and the pull request's merge
# record cites that hash, so the two would name different things afterwards.
#
# Nothing here carries a verdict. The judgement is `crates/kanhe/tests/merge_message.rs`, a Rust reaction
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
shift

subject=""
body_file=""
passthrough=()
while (($#)); do
    case $1 in
    --subject)
        subject=${2:-}
        shift 2
        ;;
    --body-file)
        body_file=${2:-}
        shift 2
        ;;
    # Both spellings, because `--subject=TEXT` and `--subject TEXT` reach `gh` identically and a guard
    # catching one would be a guard catching neither.
    --subject=* | --body-file=*)
        printf 'merge message: %s\n' \
            "refusing \`${1%%=*}=…\`: pass it as two arguments so this wrapper reads the same value \`gh\` would" >&2
        exit 1
        ;;
    --body | --body=*)
        printf 'merge message: %s\n' \
            "refusing \`--body\`: the body is judged before the merge, so it is read from a file this wrapper can hand to the gate" >&2
        exit 1
        ;;
    --merge | --rebase)
        printf 'merge message: %s\n' \
            "refusing \`$1\`: a development pull request lands on a release branch as one squash, and this gate judges that squash's message" >&2
        exit 1
        ;;
    *)
        passthrough+=("$1")
        shift
        ;;
    esac
done

if [[ -z $body_file ]]; then
    usage
    exit 2
fi
if [[ ! -f $body_file ]]; then
    printf 'merge message: %s\n' "cannot read the body file $body_file" >&2
    exit 1
fi

title=$(gh pr view "$pr" --json title --jq .title)
: "${subject:=$title}"
pr_number=$(gh pr view "$pr" --json number --jq .number)
if [[ ! $pr_number =~ ^[1-9][0-9]*$ ]]; then
    printf 'merge message: %s\n' \
        "cannot resolve $pr to one pull request number; the live commits endpoint requires its canonical identity" >&2
    exit 1
fi

# The gate. A failure aborts before the merge, which is the point: the record below cannot be amended.

# `libtest` exits 0 when `--exact` selects no test — measured, an unknown name reports `0 passed` and exits 0,
# and an `#[ignore]`d one reports `0 passed; 1 ignored` and exits 0 too. So the exit status answers *did the
# selected tests pass* while the question here is *did the gate judge this act*, and those differ exactly when
# a rename has quietly happened. Require the run to say it judged one thing.
#
# Asserted here rather than inside the gate: a renamed or silenced test cannot report that it did not run.
require_one_pass() {
    local what=$1 output=$2
    if ! printf '%s' "$output" | grep -qE 'test result: ok\. 1 passed'; then
        printf '%s\n' "$output" >&2
        printf '%s: %s\n' "$what" \
            "the gate did not run — its invocation selected no passing test, so the name in this script no longer names one. libtest exits 0 for a filter that matches nothing, which is why this is checked rather than trusted" >&2
        exit 1
    fi
}

# The pull request's own LIVE commit subjects, so the gate can ask whether this body *is* their concatenation
# rather than whether it looks like one. Local remote-tracking refs can lag the pull request or carry no fork
# head at all, and a stale subset makes a default body containing the missing subjects look unrelated and pass.
# The commits endpoint returns the full message; take its first line rather than `messageHeadline`, which
# truncates long subjects. `--paginate` keeps a large pull request one set rather than its first page.
commits=$(gh api --paginate "repos/{owner}/{repo}/pulls/$pr_number/commits" \
    --jq '.[].commit.message | split("\n")[0]')
if [[ -z ${commits//[[:space:]]/} ]]; then
    printf 'merge message: %s\n' \
        "cannot read any commit subjects from pull request $pr_number; an empty live set is not evidence about its body" >&2
    exit 1
fi

gate_output=$(TIANHENG_MERGE_SUBJECT=$subject \
    TIANHENG_MERGE_TITLE=$title \
    TIANHENG_MERGE_COMMITS=$commits \
    TIANHENG_MERGE_BODY=$(cat -- "$body_file") \
    cargo test --manifest-path "$repo/Cargo.toml" -p kanhe --test merge_message \
    -- --exact the_squash_message_is_the_pull_request_it_records 2>&1) || {
    printf '%s\n' "$gate_output" >&2
    exit 1
}
require_one_pass 'merge message' "$gate_output"

exec gh pr merge "$pr" --squash --subject "$subject" --body-file "$body_file" "${passthrough[@]}"
