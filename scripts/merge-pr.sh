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
            "refusing \`$2\` with no value: pass the value as the next argument, or drop the flag and let the \
subject default to the pull request title" >&2
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
    # Both spellings, because `--subject=TEXT` and `--subject TEXT` reach `gh` identically and a guard
    # catching one would be a guard catching neither.
    --subject=* | --body-file=*)
        printf 'merge message: %s\n' \
            "refusing \`${1%%=*}=…\`: pass it as two arguments so this wrapper reads the same value \`gh\` would" >&2
        exit 1
        ;;
    # A repository selector reaches only the final `gh pr merge`, while the title, the canonical number, the
    # live commit subjects and the gate are all read from the AMBIENT repository. Accepting one would let this
    # wrapper judge pull request N here and merge pull request N somewhere else — the gate's whole claim undone
    # by one argument, which is the sentence `scripts/publish.sh` already carries about `--manifest-path`. The
    # same distinction decides it: an argument that moves the judged SUBJECT is refused, while one that changes
    # where the result goes stays forwarded. Refusing beats threading the selector through every read, because
    # a refusal cannot be got subtly wrong and three reads agreeing by maintenance can.
    --repo | --repo=* | -R)
        printf 'merge message: %s\n' \
            "refusing \`${1%%=*}\`: this wrapper reads the title, the pull request number, the live commit \
subjects and the gate from the repository it is run in, so a repository selector would judge one pull request \
and merge another. Run it from a checkout of the repository whose pull request you are merging" >&2
        exit 2
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

# ONE repository identity, resolved once and passed to every call below.
#
# The endpoint already named a repository — implicitly, through a placeholder gh expands from the working
# directory — while the three `gh pr` calls named whichever the selector resolved to. Four references defaulting
# to the same place is agreement by circumstance; naming it once is agreement by construction, and it is the
# shape this wrapper's own contract asks for: the accepted selector, the live commit set and the merge must be
# one pull request.
repository=$(gh repo view --json nameWithOwner --jq .nameWithOwner) || {
    printf 'merge message: %s\n' \
        "cannot resolve which repository this checkout is, so the selector, the live commit set and the merge \
cannot be shown to name one pull request" >&2
    exit 2
}

title=$(gh pr view "$pr" --repo "$repository" --json title --jq .title)
: "${subject:=$title}"
pr_number=$(gh pr view "$pr" --repo "$repository" --json number --jq .number)
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
commits=$(gh api --paginate "repos/$repository/pulls/$pr_number/commits" \
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

exec gh pr merge "$pr" --repo "$repository" --squash --subject "$subject" --body-file "$body_file" \
    "${passthrough[@]}"
