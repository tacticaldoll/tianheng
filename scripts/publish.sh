#!/usr/bin/env bash
#
# The sanctioned publish path: the source gate, then `cargo publish --workspace`.
#
# Why a wrapper rather than two documented steps. "Publish the tagged `main` commit, not the release
# branch" was already said in the 0.4.0 window, before 0.4.0 was published from the release branch
# anyway. The failure mode is not disagreement about the rule, it is a separate step skipped by habit
# at the one moment nothing can be undone — `.cargo_vcs_info.json` is stamped with the commit
# `cargo publish` ran on, and a published version can never be re-uploaded. Behind this script the
# gate stops being a step to remember: it is the only way to reach `cargo publish`, so skipping it
# means not publishing at all.
#
# Whether to publish stays outside. The publish is irreversible (a version is yankable, never
# deletable) and remains confirm-first with a human, and the Definition of Done, the packaged-tarball
# verification, and the bundled-license check all run before anyone arrives here.
#
# What may reach `cargo publish` is an ALLOWLIST. This script used to forward everything except
# `--manifest-path`, and its sibling `scripts/merge-pr.sh` learned what that costs: naming what may
# not pass leaked three times there, most sharply through spellings of flags whose long forms were
# already named. Enumerating what may pass means an argument this script does not know — including one
# a future cargo adds — is refused by default, which is the property a denylist cannot have. This
# family argues it in its own law: an allowlist is always stricter than a denylist.
#
# Classified against `cargo publish --help` on cargo 1.96.0 by one question: does the argument move
# what the gate judged, or what the act records? Admitted are the arguments that change only whether
# and how the publish proceeds. Refused are the ones that move the source tree (`--manifest-path`),
# the set of crates (`--exclude`, and the `--workspace` this script supplies itself), what cargo
# verifies before uploading (`--no-verify`, the feature and target selectors), what gets packaged
# (`--allow-dirty`), and `--config`, which can become any of those and can name a whole configuration
# file besides.
#
# **Admitting an argument takes TWO questions, not one.** The first is above. The second is whether cargo
# actually HONOURS it beside what this script supplies itself — and `--package` failed that one silently for a
# window: written after an unconditional `--workspace`, cargo discarded it and published everything. Classify
# against the tool's real behaviour at a named version, not against its `--help` alone.
#
# Two classifications are worth their sentence. `--package` narrows by NAMING, which a partly
# completed publish genuinely needs — crates.io accepts the six one at a time and a resumed run must
# say which — and the command then records what it did; `--exclude` narrows by SUBTRACTION under the
# `--workspace` this script would otherwise supply, so the invocation reads as the whole workspace while
# publishing less. And `--allow-dirty` was forwarded before, on the ground that the source gate
# refuses a dirty tree upstream anyway: that makes it inert rather than safe, and inert-by-someone-
# else is not how this script holds anything.
#
# `--registry` and `--index` stay admitted, keeping the reasoning that admitted them: they change the
# publish's DESTINATION, not its source, which is a different claim from the one this wrapper and its
# gate make. `--token` no longer joins them — cargo 1.96.0 answers it with `\`cargo publish --token\`
# is deprecated in favor of using \`cargo login\` and environment variables`, so the refusal points
# where cargo does.
#
# ONE spelling each, values as separate arguments. Parsing a tool's glued and equals forms is exactly
# what let the short forms through the sibling wrapper; refusing them costs an argument's worth of
# typing and removes the parsing question entirely.
set -Eeuo pipefail

# Before the gate, not after: a refusal must not depend on the gate's own verdict, and reading the
# arguments is the cheapest of the two. A misconfigured invocation exits 2, the usage-error class this
# repository's own runner contract already uses, rather than 1 — which is what a gate that ran and
# refused would exit.
refuse() {
    printf 'publish source: %s\n' "refusing \`$1\`: $2" >&2
    exit 2
}

# The same two classes its sibling `scripts/merge-pr.sh` states, for the same reason: `2` is everything this
# script could not judge — a misconfigured invocation, and an input it could not read — and `1` is a gate that
# ran and refused. A gate that did NOT run belongs to the first class, however loudly its own message says so.
cannot_judge() {
    printf 'publish source: %s\n' "$1" >&2
    exit 2
}

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

# This script's own root: the tree the gate judges, the manifest it is run from, and the directory
# `cargo publish` runs in. Acquired after `cannot_judge` rather than at the top of the file, because it is an
# acquisition like any other and must report the class that function defines. Unguarded it was the one
# statement `set -e` answered for: a failed `cd` exits 1, so a script that never found the tree it publishes
# would have reported the class that means the gate ran and refused, in front of an irreversible act.
repo=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd) || cannot_judge \
    "cannot resolve this script's own root from ${BASH_SOURCE[0]}, so neither the tree the source gate judges \
nor the one \`cargo publish\` would package can be located — which is not the same fact as a gate that ran \
and refused"

# The package selection, held separately from everything else forwarded.
#
# `--workspace` is this script's DEFAULT selection, not a constant it writes over whatever the caller asked
# for. Written unconditionally it silently voided the one selector this script admits: measured on cargo 1.96.0
# with the identical selection flags, `--workspace --package xuanji` selects 8 packages and `--package xuanji`
# selects 1, and cargo says nothing — it maps (`--workspace`, no `--exclude`, any `--package`) to *all*. So
# `publish.sh --package xuanji` published the entire workspace while the comment beside the arm explained that
# `--package` is how a partly completed publish resumes, in front of the one act that cannot be undone.
selection=(--workspace)

forwarded=()
while (($#)); do
    case $1 in
    # Whether and how the publish proceeds — never its source, its set, or what cargo verifies.
    --dry-run | --keep-going | --locked | --offline | --frozen | --verbose | --quiet)
        forwarded+=("$1")
        shift
        ;;
    # The one admitted selector. It REPLACES the default rather than joining it, and may be repeated —
    # measured, two `--package` flags select two packages.
    --package)
        if (($# < 2)); then
            refuse "$1" "this script reads every value as the argument after its flag, so pass it that way or drop the flag"
        fi
        if [[ ${selection[0]} == --workspace ]]; then
            selection=()
        fi
        selection+=("$1" "$2")
        shift 2
        ;;
    --jobs | --color | --target-dir | --registry | --index)
        if (($# < 2)); then
            refuse "$1" "this script reads every value as the argument after its flag, so pass it that way or drop the flag"
        fi
        forwarded+=("$1" "$2")
        shift 2
        ;;
    # The arms below decide nothing the catch-all would not — each is unlisted, so it is refused
    # either way. They exist to say WHY, because a refusal an operator cannot act on is a refusal they
    # work around. Each pattern covers cargo's glued and equals forms of the same flag.
    --manifest-path | --manifest-path=*)
        refuse "$1" "it moves cargo's workspace root away from the tree this gate judges, so the source gate would pass on $repo while cargo published something else. Publish from a checkout of the tagged \`release: X.Y.Z\` commit on origin/main instead — cargo stamps the commit it ran on into every tarball, permanently"
        ;;
    --exclude | --exclude=* | --workspace)
        refuse "$1" "this script publishes the workspace and writes \`--workspace\` itself, so an argument that removes a crate from that set would have the invocation read as the whole workspace while publishing less. To publish part of it, name the parts with \`--package <spec>\`, which records what it did"
        ;;
    --no-verify)
        refuse "$1" "it drops cargo's own build of the packaged tarballs at the one moment nothing can be undone; a version is yankable, never replaceable"
        ;;
    --allow-dirty)
        refuse "$1" "it packages content no commit holds, and \`.cargo_vcs_info.json\` would still name a commit that does not contain what was uploaded. The source gate refuses a dirty tree upstream, which makes this inert rather than safe"
        ;;
    --config | --config=* | -Z* | --features | --features=* | -F* | --all-features | --no-default-features | --target | --target=*)
        refuse "$1" "it changes what cargo evaluates or configures on the way to an irreversible upload — \`--config\` can name a whole configuration file and reach every other refusal here. Set what you need in the tree the gate judges"
        ;;
    --token | --token=*)
        refuse "$1" "cargo 1.96.0 answers it with \`\`cargo publish --token\` is deprecated in favor of using \`cargo login\` and environment variables\`, so this script points where cargo does rather than forwarding a flag on its way out"
        ;;
    -p* | -n | -j* | -v | -vv | -q)
        refuse "$1" "this script admits one spelling of each argument it forwards, with values as separate arguments, because parsing a tool's short and glued forms is what a denylist has to get exhaustively right. Use the long form"
        ;;
    *)
        refuse "$1" "this script forwards only the arguments that change whether and how the publish proceeds, never its source, its set of crates, or what cargo verifies, and this is not one of them. An argument it does not know is refused rather than passed on, because the upload it stands in front of can be yanked but never replaced"
        ;;
    esac
done

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

verdict_file=$(mktemp) || cannot_judge \
    "cannot open a file for the gate to report its refusal class on, so a failing gate could not be told from \
an input it could not read"
trap 'rm -f "$verdict_file"' EXIT

# The source gate. It lives in Rust with the other repository gates and does not run in
# development — no development checkout is a release snapshot — so it is asked for explicitly here, the one
# moment it can answer. A failure aborts before `cargo publish`, which is the point: the act below is
# irreversible.
# `libtest` exits 0 when `--exact` selects no test — measured, an unknown name reports `0 passed` and exits 0,
# and an `#[ignore]`d one reports `0 passed; 1 ignored` and exits 0 too. So the exit status answers *did the
# selected tests pass* while the question here is *did the gate judge this act*, and those differ exactly when
# a rename has quietly happened. Require the run to say it judged one thing.
#
# Asserted here rather than inside the gate: a renamed or silenced test cannot report that it did not run.
require_one_pass() {
    local output=$1
    if ! printf '%s' "$output" | grep -qE 'test result: ok\. 1 passed'; then
        printf '%s\n' "$output" >&2
        cannot_judge \
            "the gate did not run — its invocation selected no passing test, so the name in this script no longer names one. libtest exits 0 for a filter that matches nothing, which is why this is checked rather than trusted"
    fi
}

gate_output=$(TIANHENG_GATE_VERDICT=$verdict_file \
    TIANHENG_PUBLISH_SOURCE=1 TIANHENG_WORKSPACE_TESTS=1 \
    cargo test --manifest-path "$repo/Cargo.toml" -p kanhe --test publish_source \
    -- --exact the_publish_source_is_the_signed_release_snapshot 2>&1) || {
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

# Guarded like every acquisition, and for the sharper reason: this one runs **after** the gate has passed. A
# failed `cd` under `set -e` exits 1 — the class that means a gate ran and refused — so a wrapper that could
# not enter the tree would report a disagreement the gate never found, one line before `cargo publish`.
cd "$repo" || cannot_judge \
    "cannot enter $repo, the tree \`cargo publish\` would package, after the source gate had already passed \
— which is not the same fact as a gate that ran and refused"
# Removed here, not left to the trap. An EXIT trap does not run when `exec` replaces the shell image —
# measured, `bash -c 'trap "echo T" EXIT; exec true'` prints nothing while the same script without `exec` prints
# `T`. So the trap fired on every path where nothing happened and was skipped on the one path that completes the
# act: three successful runs left three empty files in `$TMPDIR`, measured against an isolated one. The trap
# stays, because it is what covers the failure paths; `exec` stays, because the tool's exit status becoming this
# script's is deliberate.
rm -f "$verdict_file"

# `forwarded` may be empty, and `"${empty[@]}"` under `set -u` is an unbound variable before bash 4.4 —
# where this wrapper would abort through the ERR trap reporting "an unguarded command failed", a sentence
# about the wrong cause, on the argument-free invocation that is the ordinary one. `selection` is never empty
# and needs no guard. The `+` form is used rather than a version check, so no minimum has to be declared
# anywhere and kept in step.
exec cargo publish "${selection[@]}" ${forwarded[@]+"${forwarded[@]}"}
