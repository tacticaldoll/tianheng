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
# The functions below are written to that contract: each failure goes through `cannot_judge`, which
# `exit 2`s, and the trap's message covers anything else by construction.

# This library is a part of each wrapper, not a unit of its own — sourced, it must never be executed.
# The guard answers EX_USAGE (64), which is neither class a wrapper reserves: a library run as a command is a
# plain misuse, not a wrapper that stopped, so `1` would read as a gate that refused and `2` as a
# cannot-judge, and both are the wrong sentence for it. The number is a global rather than a literal so the
# answer has one owner; `a_library_run_as_a_command_stops_without_reaching_a_wrapper_s_classes` holds the
# direction by running the file, and `each_wrapper_chooses_its_exit_class_in_one_place` counts zero bare
# `exit 1`/`exit 2` sites in it.
#
# **Definitions only, above this line and below it.** A wrapper sources this file inside `if ! source …`,
# which runs it with errexit suppressed and the ERR trap out of reach — so a statement here that can fail
# would be silently ignored rather than refused. Everything at the top level is a definition or an
# assignment; the day that stops being true, the sourcing shape has to change first.
WRAPPER_USAGE=64
if [[ ${BASH_SOURCE[0]} == "$0" ]]; then
    printf '%s\n' \
        'wrapper.sh is a library the two wrappers source, not a wrapper itself. Run scripts/merge-pr.sh or scripts/publish.sh' \
        >&2
    exit "$WRAPPER_USAGE"
fi

# --- the two exit classes, chosen in one place ---------------------------------------------------------------
#
# `2` is everything a wrapper could not judge: a misconfigured invocation, and an input it could not read.
# `1` is a gate that ran and refused. The contract is this repository's own — `crates/shengmo/src/law.rs`:
# *0 clean, 1 violation, 2 constitution/usage error* — and each is chosen here, once, for both wrappers.
#
# **Five could-not-read conditions were split across both classes with no rule.** An unresolvable repository
# exited 2 while an unreadable body file, an unreadable head, an unresolvable pull-request number and an
# unreadable commit set exited 1. Two of those facts are ones the gate this family fronts types the other way:
# `merge_message_gate::judge` returns cannot-judge for an unavailable title and for unavailable commit
# subjects, because "which is not the same fact as a subject that disagrees". So the wrapper reported as a
# disagreement what its own gate calls unjudgeable — telling an operator, in the words of the publish gate,
# "to go looking for a disagreement that does not exist".
#
# The subject the refusal is about — `merge message` or `publish source` — is the one thing the two copies
# never shared, and the WRAPPER_SUBJECT global each wrapper sets before sourcing is its one owner: every
# helper here reads it, so a call site cannot spell one wrapper's prefix inside the other.
cannot_judge() {
    printf '%s: %s\n' "$WRAPPER_SUBJECT" "$1" >&2
    exit 2
}

# The refusal idiom, delegating the class to the function above rather than choosing it again.
#
# **Converging the `case` arms onto this helper left four sites behind, and two of them predated it.** The
# merge wrapper's positional selector and body-file guard exited through a bare `usage; exit 2` carrying none
# of the prefix above; the URL refusal hand-copied that function's body because both helpers were defined
# below it; and its value guard re-spelled it further down the file. Every stop in both wrappers now
# delegates, and what decides that is `each_wrapper_chooses_its_exit_class_in_one_place` rather than the next
# reader — a helper's existence was never the property, since three of those four sites were written with it
# in scope.
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
# exactly one such statement: the gate's own verdict arm.
#
# Measured on bash 5.x rather than reasoned about. A bare failure traps and exits 2, including a failed `cd`.
# A `||`-guarded command does not trap, so every existing guard still decides its own outcome. A failure in a
# condition — `if`, `while`, `!`, `&&` — does not trap, so the `grep -q` that checks the gate ran is
# unaffected. An explicit `exit 1` is not intercepted, so the gate's verdict still reaches the caller. `set -E`
# is required and is not optional: without it a failure inside a function exits 1 and the trap never sees it.
install_exit_class_trap() {
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
    read_verdict
    if [[ $verdict != "$GATE_CLEAN_CLASS" ]]; then
        cannot_judge \
            "the gate ran and passed without reaching a verdict — the channel carries ${verdict:-nothing}, and a run that judged nothing is not a run that agreed. This is the class a passing test cannot distinguish on its own, which is why it is read rather than inferred"
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
        printf '%s\n' "$output" >&2
        cannot_judge \
            "the gate did not run — its invocation selected no passing test, so the name in this script no longer names one. libtest exits 0 for a filter that matches nothing, which is why this is checked rather than trusted"
    fi
}

# The class the gate reported, read off the channel it was given. Absent, empty or anything else is a run that
# reached no verdict — a compile error included — and that is not a disagreement. The one read, shared by
# `require_a_verdict` on the passing path and each wrapper's verdict arm on the failing one; an arm is then
# one comparison, and the three hand-written copies the extraction briefly left are the shape this removes.
read_verdict() {
    verdict=""
    if [[ -f $verdict_file ]]; then
        verdict=$(cat -- "$verdict_file") || verdict=""
    fi
}

# The verdict file's lifecycle: created where the gate is about to run, removed where the act completes.
# An EXIT trap does not run when `exec` replaces the shell image — measured, `bash -c 'trap "echo T" EXIT;
# exec true'` prints nothing while the same script without `exec` prints `T`. So the trap fired on every
# path where nothing happened and was skipped on the one path that completes the act. The trap stays,
# because it is what covers the failure paths; the removal stays explicit, immediately before the `exec`.
open_verdict_file() {
    verdict_file=$(mktemp) || cannot_judge \
        "cannot open a file for the gate to report its refusal class on, so a failing gate could not be told \
from an input it could not read"
    trap 'rm -f "$verdict_file"' EXIT
}
