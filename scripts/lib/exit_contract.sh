# shellcheck shell=bash
#
# The family's exit contract, as a backstop every gate installs: 0 clean, 1 violation, 2 cannot judge, and
# nothing else.
#
# Why a backstop rather than per-command handling. `set -e` with `pipefail` carries a failing utility's own
# status out of the process, so an unhandled failure leaves a gate reporting a number the contract does not
# define — and, because the abort is the shell's rather than the gate's, printing nothing at all. Measured
# across the gates before this existed, each with one tool stubbed to fail:
#
#   check_publish_source.sh      `git status` fails mid-run  -> exit 131, no output
#   check_release_coherence.sh   `git log` fails             -> exit 130, no output
#   check_dod_coherence.sh       `awk` fails                 -> exit 9,   no output
#   check_whitespace_hygiene.sh  `mktemp` fails              -> exit 7,   no output
#   check_bound_register.sh      `sed` fails reading a spec  -> exit 4,   no output
#   check_reference_integrity.sh `git ls-files` fails        -> exit 3,   no output
#
# Two of those declare the contract in their own headers, and the first gates an irreversible act. Wrapping
# each command instead was tried twice in one release window and twice left a site behind; the count of
# unwrapped commands is not the property to manage, so the property is moved into the shell.
#
# It reports WHERE, never what. A trap cannot know what a command meant, and one that invented a cause would
# be worse than the raw status it replaces — so a read worth naming keeps its own refusal, and this is the
# floor beneath those, not a substitute for them.
#
# Safe over gates built on deliberate non-zero returns — `grep -q` misses, `[[ … ]] && continue`,
# `((status <= 1)) || cannot_judge`, captured pipelines with their own handlers — because that was measured
# rather than assumed: under `errtrace` a failure in any of those shapes does not fire an `ERR` trap, even
# inside a function. Each gate's own matrix, whose passing directions run through exactly those shapes, is
# what fails loudly if that ever stops holding.
#
# It returns immediately in a SUBSHELL, and that guard is not caution — it is a correction. `errtrace`
# propagates this trap into process substitutions, where a legitimately-failing command is routine: the
# whitespace gate reads `done < <(grep -n "[[:space:]]\$" … | cut -d: -f1)`, and `grep` exiting 1 on a file
# with no trailing whitespace is the ORDINARY case. Without the guard that fired the diagnostic once per
# clean file — hundreds of lines of "cannot judge" on a passing run, while the parent still exited 0, so an
# exit-code-only check hid it entirely.
#
# Returning is also the correct scope, not merely the quiet one: a subshell's status reaches the parent only
# where the parent reads it, and where the parent does not read it, forcing 2 there changes nothing the
# caller can see. A swallowed subshell status is the OTHER half of this class and no trap can fix it — it is
# closed by capturing the status where it can be acted on, which is why the gates do that explicitly.
#
# Attribution survives the indirection, also measured: `${BASH_SOURCE[0]}` and `$LINENO` expand at fire time
# in the failing gate's frame, so the diagnostic names the gate and line, never this file. The caller must
# have `set -E` (`errtrace`) for the trap to reach inside its functions.
exit_contract_backstop() {
    local prefix=$1
    trap 'unhandled=$?
          [[ ${BASH_SUBSHELL:-0} -eq 0 ]] || exit "$unhandled"
          printf "%s: cannot judge: an unhandled command failed (exit %d) at %s:%d — this gate reports 0 clean, 1 violation, 2 cannot judge, and nothing else\n" "'"$prefix"'" "$unhandled" "${BASH_SOURCE[0]}" "$LINENO" >&2
          exit 2' ERR
}
