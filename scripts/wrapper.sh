# The one lifecycle both irreversible-act wrappers are built on — sourced, never executed.
#
# This file exists because the lifecycle lived twice, byte-identical across most of it, and the copies
# agreed only because a reviewer kept them so: `merge-pr.sh` spelled `printf … >&2; exit 2` inline at
# four sites while `publish.sh` had the helper, and the divergence was found by reading the pair, not by
# anything running. What the two wrappers share is now written once, here; what differs — the argument
# allowlist, the evidence, the gate, the act — stays in each wrapper, which is the half that makes each
# of them a law unto its own act.
#
# The split this file is NOT: a verdict carried in shell. Every judgement still lives in `crates/kanhe`
# and each wrapper still names its own gate by `--exact`; this library orders the act around that
# verdict and renders none of its own.
#
# Sourced rather than executed, so every function here runs inside the sourcing wrapper's
# `set -Eeuo pipefail` and answers to its ERR trap; nothing below may leave a command whose failure
# chooses the exit class — that is the property the ERR trap exists to take away from every statement.
# The functions below are written to that contract: each failure goes through `cannot_judge`, and the trap's
# message covers anything else by construction.

# This library is a part of each wrapper, not a unit of its own — sourced, it must never be executed.
#
# **Definitions only at the top level.** A wrapper sources this file inside `if ! source …`, which runs it
# with errexit suppressed and the ERR trap out of reach, so a top-level statement that can fail would be
# silently ignored rather than refused. The top level holds assignments, function definitions and the
# execution guard below, whose test cannot fail; anything else needs the sourcing shape changed first.

# --- the exit codes, declared once -------------------------------------------------------------------------
#
# Spelled in `kanhe::verdict_channel` and read here, never typed twice: `wrapper_exit` owns the two classes and
# `LIBRARY_MISUSE` the guard's answer, and `each_wrapper_uses_the_channel_the_gates_report_on` holds these
# three against them. Every `exit` in this file and in both wrappers names one of them; the one literal is
# each wrapper's bootstrap guard, which runs before this file is loaded.
#
# `2` is everything a wrapper could not judge: a misconfigured invocation, and an input it could not read.
# `1` is a gate that ran and refused. The contract is this repository's own — `crates/shengmo/src/law.rs`:
# *0 clean, 1 violation, 2 constitution/usage error*.
#
# **Five could-not-read conditions were split across both classes with no rule.** An unresolvable repository
# exited 2 while an unreadable body file, an unreadable head, an unresolvable pull-request number and an
# unreadable commit set exited 1. Two of those facts are ones the gate this family fronts types the other way:
# `merge_message_gate::judge` returns cannot-judge for an unavailable title and for unavailable commit
# subjects, because "which is not the same fact as a subject that disagrees". So the wrapper reported as a
# disagreement what its own gate calls unjudgeable — telling an operator, in the words of the publish gate,
# "to go looking for a disagreement that does not exist".
WRAPPER_EXIT_VIOLATION=1
WRAPPER_EXIT_UNJUDGED=2
WRAPPER_EXIT_MISUSE=64

# Executed rather than sourced is a plain misuse, answered outside both classes a wrapper reserves: `1` would
# read as a gate that refused and `2` as a wrapper that could not judge.
# `a_library_run_as_a_command_stops_without_reaching_a_wrapper_s_classes` runs the file.
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    printf '%s\n' \
        'wrapper.sh is a library the two wrappers source, not a wrapper itself. Run scripts/merge-pr.sh or scripts/publish.sh' \
        >&2 || :
    exit "$WRAPPER_EXIT_MISUSE"
fi

# **A write never chooses the class.** Every line a wrapper prints goes through `tell` or `say`, and neither can
# fail: a write to a closed or broken stream is dropped rather than let reach `set -e`. Measured on bash 5.2
# before these existed, with the stream closed: `cannot_judge` exited `1` — its `printf` failed, the ERR trap
# re-entered it, and the second failure left through errexit with printf's own status, the class reserved for a
# gate that ran and refused — and a report printed after a completed act exited `2`, calling the merge it had
# just made a run that reached no verdict. The class is the one fact a closed terminal must not move; the text is
# what an operator loses by closing it. `install_exit_class_trap` ignores SIGPIPE so a broken pipe is such a
# failed write rather than a signal that ends the shell with `141`, which is neither class.
#
# `tell` is for the operator's diagnostics and goes to stderr; `say` is the one line of result a completed act
# reports, and goes to stdout, where a caller capturing the result reads it.
tell() {
    printf '%s\n' "$@" >&2 || :
}

say() {
    printf '%s: %s\n' "$WRAPPER_SUBJECT" "$1" || :
}

# The subject the refusal is about — `merge message` or `publish source` — is the one thing the two copies
# never shared, and the WRAPPER_SUBJECT global each wrapper sets before sourcing is its one owner: every
# helper here reads it, so a call site cannot spell one wrapper's prefix inside the other.
cannot_judge() {
    tell "$WRAPPER_SUBJECT: $1"
    exit "$WRAPPER_EXIT_UNJUDGED"
}

# The refusal idiom, delegating the class to the function above rather than choosing it again.
#
# Every stop in both wrappers delegates here or to `cannot_judge`; a stop that exits on its own is refused by
# `each_wrapper_chooses_its_exit_class_in_one_place`.
refuse() {
    cannot_judge "refusing \`$1\`: $2"
}

# **The class a wrapper exits is decided by construction, not by a sweep that must be exhaustive.**
#
# Under `set -e` any unguarded failure exits with the TOOL's status, and this repository reserves `1` for a
# gate that ran and refused. Two sweeps were widened to catch that — first by tool name, then by command
# substitution — and a bare `cd` walked through both, because the axis was never *which shape the statement
# has*: it is *any statement whose failure can choose the class*. That is every command, which is why
# enumerating them is the wrong instrument. Enumerating what may exit `1` is the right one, and there is
# exactly one such statement: `exit_for_the_gates_refusal`, below.
#
# Measured on bash 5.x rather than reasoned about. A bare failure traps and exits 2, including a failed `cd`.
# A `||`-guarded command does not trap, so every existing guard still decides its own outcome. A failure in a
# condition — `if`, `while`, `!`, `&&` — does not trap, so the `grep -q` that checks the gate ran is
# unaffected. An explicit `exit` is not intercepted, so the gate's verdict still reaches the caller. `set -E`
# is required and is not optional: without it a failure inside a function exits 1 and the trap never sees it.
install_exit_class_trap() {
    trap '' PIPE
    trap 'cannot_judge "an unguarded command failed, so this wrapper stopped without reaching a verdict — which is not the same fact as a gate that ran and refused"' ERR
}

# A wrapper's own root: the tree its gate is run from. Acquired here rather than at the top of each wrapper,
# because it is an acquisition like any other and must report the class `cannot_judge` defines. Unguarded it
# was the one statement `set -e` answered for: a failed `cd` exits 1, so a wrapper that never found its gate
# would have reported the class that means the gate ran and refused.
wrapper_own_root() {
    cd -- "$(dirname -- "${BASH_SOURCE[1]}")/.." && pwd
}

# The channel the gate reports its refusal class on, and the class that means a disagreement.
#
# Both are held against `kanhe::verdict_channel` by `crates/kanhe/tests/gate_exit_classes.rs`, so neither the
# variable name nor the class spelling can drift from the gate's side.
#
# **This replaced grepping the gate's output.** Searching stdout for `(Violation)` put the parentheses in
# this script and the variant name in Rust — two owners for one token — and measured, changing the gate's
# format string left every direction green while this pattern matched nothing, so every violation would have
# reported as unjudged. It also searched a stream carrying arbitrary tooling output, where a class could be
# read from text no judgement wrote. A file the gate writes only when it has a verdict makes *absent* mean
# unjudged by construction.
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
    local reached
    reached=$(verdict_on_channel) || reached=""
    if [[ $reached != "$GATE_CLEAN_CLASS" ]]; then
        cannot_judge \
            "the gate ran and passed without reaching a verdict — the channel carries ${reached:-nothing}, and a run that judged nothing is not a run that agreed. This is the class a passing test cannot distinguish on its own, which is why it is read rather than inferred"
    fi
}

# `libtest` exits 0 when `--exact` selects no test — measured, an unknown name reports `0 passed` and exits 0,
# and an `#[ignore]`d one reports `0 passed; 1 ignored` and exits 0 too. So the exit status answers *did the
# selected tests pass* while the question here is *did the gate judge this act*, and those differ exactly when
# a rename has quietly happened. Require the run to say it judged one thing.
#
# Asserted here rather than inside the gate: a renamed or silenced test cannot report that it did not run.
require_one_pass() {
    local output=$1
    # A here-string, not a pipe. `grep -q` exits at its first match, and under `set -o pipefail` the
    # `printf` upstream takes SIGPIPE and that becomes the pipeline's status — so this would report *the
    # gate did not run* for a closed pipe, immediately before an irreversible act. Measured: with the token
    # at the end of a 405 KB stream, which is where a `cargo test` summary sits, 0 of 8 runs returned
    # non-zero; with the same token near the start, 8 of 8 did. Both wrappers were holding by where the
    # token happened to sit, which nothing declares and nothing keeps true.
    if ! grep -qE 'test result: ok\. 1 passed' <<< "$output"; then
        tell "$output"
        cannot_judge \
            "the gate did not run — its invocation selected no passing test, so the name in this script no longer names one. libtest exits 0 for a filter that matches nothing, which is why this is checked rather than trusted"
    fi
}

# The class the gate reported, read off the channel it was given. Absent, empty or anything else is a run that
# reached no verdict — a compile error included — and that is not a disagreement. Printed rather than left in
# a global, so each caller holds its own reading.
verdict_on_channel() {
    if [[ -f $verdict_file ]]; then
        cat -- "$verdict_file" || printf ''
    fi
}

# The failing path of a gate's run, identical for both wrappers and so written once: the gate's output for
# the operator, then the class the channel carries. Only a `Violation` the gate itself wrote leaves as the
# violation class, which makes *only the gate's verdict exits 1* a property of this function and of the
# channel rather than of where a wrapper calls it.
exit_for_the_gates_refusal() {
    local output=$1 reached
    tell "$output"
    reached=$(verdict_on_channel) || reached=""
    if [[ $reached == "$GATE_VIOLATION_CLASS" ]]; then
        exit "$WRAPPER_EXIT_VIOLATION"
    fi
    cannot_judge \
        "the gate failed without reporting a disagreement — its channel carries ${reached:-nothing} — so this stops as a run that could not judge, not as one that refused"
}

# The verdict file's lifecycle: created where the gate is about to run, removed by the EXIT trap on every path.
# That holds because no wrapper `exec`s — `perform_the_act` below runs the act — and an EXIT trap does not run
# when `exec` replaces the shell image: measured, `bash -c 'trap "echo T" EXIT; exec true'` prints nothing while
# the same script without `exec` prints `T`.
open_verdict_file() {
    verdict_file=$(mktemp) || cannot_judge \
        "cannot open a file for the gate to report its refusal class on, so a failing gate could not be told \
from an input it could not read"
    trap 'rm -f "$verdict_file"' EXIT
}

# **The act, run and accounted for in one place.** Both wrappers end here: `perform_the_act <account> <command…>`
# runs the irreversible command — never `exec`s it — and hands its exit status to the wrapper's own `<account>`,
# which decides the class from what it can **observe** of the act rather than from that status.
#
# Two defects had one cause, and this is the shape that removes it. A tool's status is the tool's class, not this
# repository's: `gh pr merge` exits `1` when it does not merge and `cargo publish` exits `1` on an argument it
# cannot parse, and `1` is reserved for a gate that ran and refused — so an `exec`d act reported its own failure
# as a disagreement the gate never found. And a status is not an observation of the remote side: a client can
# exit non-zero after the server acted, when the response is what was lost, so a sentence built from the status
# alone can tell an operator an irreversible act did not happen when it did. The account reads what it can and
# says which half it could not.
#
# An account returns for a completed act, or leaves through `cannot_judge`; it never exits on its own, which
# `each_wrapper_chooses_its_exit_class_in_one_place` holds.
perform_the_act() {
    local account=$1 status=0
    shift
    "$@" || status=$?
    "$account" "$status"
}
