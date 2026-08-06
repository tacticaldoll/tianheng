# shellcheck shell=bash
#
# One way to read an observation source: MATERIALIZE it, check its status in the parent shell, then consume the
# file. Never `while … done < <(producer)`, whose status the parent never sees.
#
# The class this exists to end, measured on two gates rather than reasoned about:
#
#   * `check_whitespace_hygiene.sh` — a `git ls-files --eol` that emitted one clean row and then exited 7 left the
#     gate reporting `whitespace hygiene ok (1 tracked text files)` and **exit 0**, over a repository it had read
#     one file of. The printed count fell from 2 to 1 and nothing reacted to it: the evidence was in the output.
#   * `check_release_coherence.sh` — a `git log` truncated the same way made the gate conclude the tree was in
#     snapshot state and report `[Unreleased] must be empty`, **exit 1**. A violation invented from a partial read,
#     sending a maintainer to look for a problem that is not there.
#
# So the class fails in BOTH directions, which is why a vacuity guard cannot cover it: `inspected > 0` was built
# for zero rows, and a partial read gives one or more. The guard and this helper answer different questions.
#
# `BACKLOG.md` recorded a swallowed subshell status as the window's most-recurring class — nine mentions — and
# `check_bound_register.sh` already had the right shape locally in `read_tracked_files`. It was bespoke, which is
# why that same gate still carried two unchecked producers. This is that shape, shared.

# Run a producer, capturing its output into the file named by $2 and refusing in the parent shell if it failed.
#
#   capture_or_refuse <what> <destination-file> <refusal-function> -- cmd...
#
# `<what>` names the observation source for the diagnostic. `<refusal-function>` is the caller's own
# cannot-judge — this library never decides a gate's exit contract for it, because the family's three-way contract
# belongs to the gate.
capture_or_refuse() {
    local what=$1 destination=$2 refuse=$3
    shift 3
    [[ $1 == -- ]] && shift
    "$@" >"$destination" \
        || "$refuse" "reading $what failed (exit $?), and a failed read is not an empty result — treating it as one reports a verdict over content that was never read"
}

# The same, into an array named by $2, for a NUL-separated producer.
#
#   capture_nul_or_refuse <what> <array-name> <refusal-function> -- cmd...
#
# `-z` rather than newlines wherever git is the producer: `git ls-files` quotes a non-ASCII path by default, so a
# quoted path names no file on disk.
capture_nul_or_refuse() {
    local what=$1 refuse=$3
    local -n _dest=$2
    shift 3
    [[ $1 == -- ]] && shift
    local scratch
    scratch=$(mktemp) || "$refuse" "could not create a temporary file to capture $what"
    _dest=()
    if ! "$@" >"$scratch"; then
        local status=$?
        rm -f "$scratch"
        "$refuse" "reading $what failed (exit $status), and a failed enumeration is not an empty repository — treating it as one reports a verdict over content that was never read"
    fi
    mapfile -d '' -t _dest <"$scratch"
    rm -f "$scratch"
}
