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
# What may reach `cargo publish` is an ALLOWLIST: an argument this script does not know — including one a
# future cargo adds — is refused by default, which is the property a list of what to forbid cannot have.
#
# Classified against `cargo publish --help` on cargo 1.96.0 by the questions `repository-checks` states for
# every wrapper in front of an irreversible act: what the argument moves, whether the tool honours it as the
# wrapper composes the invocation, and whether it performs a further act. Admitted are the arguments that
# change only whether and how the publish proceeds. Refused are the ones that move the source tree (`--manifest-path`),
# the set of crates (`--exclude`, and the `--workspace` this script supplies itself), what cargo
# verifies before uploading (`--no-verify`, the feature and target selectors), what gets packaged
# (`--allow-dirty`), and `--config`, which can become any of those and can name a whole configuration
# file besides.
#
# **Whether cargo HONOURS an argument beside what this script supplies is measured, not read from `--help`.**
# Measured on cargo 1.96.0: `--package` written after an unconditional `--workspace` is discarded, and the whole
# workspace is published.
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
# gate make. `--token` is not among them — cargo 1.96.0 answers it with `\`cargo publish --token\`
# is deprecated in favor of using \`cargo login\` and environment variables`, so the refusal points
# where cargo does.
#
# ONE spelling each, values as separate arguments. Parsing a tool's glued and equals forms would admit
# short forms whose long forms are refused; refusing them costs an argument's worth of typing and
# removes the parsing question entirely.
set -Eeuo pipefail
# The stream policy, before anything can write: `scripts/wrapper.sh`'s paragraph on `tell` says why it is here.
trap '' PIPE

WRAPPER_SUBJECT='publish source'
# The lifecycle this wrapper is built on: the two exit classes and the one helper that chooses them, the
# ERR trap, the verdict file's lifecycle, and the two guards over the gate's run. It lived twice and the
# copies agreed by maintenance; it is written once now, and what stays below is what only this wrapper
# decides — the allowlist, the selection, the gate, and the publish. Resolved through the tree the wrapper
# names, so a copy of this file planted outside the tree — a direction's fixture — answers *library not
# found* rather than judging under no lifecycle.
#
# **The one acquisition the shared machinery cannot guard is the one that loads it.** `cannot_judge` is the
# library's, so a `source` that fails has nowhere to delegate to, and unguarded under `set -e` it exits with
# `source`'s own status — `1`, the class that means a gate ran and refused. Measured on bash 5: the ERR trap
# does not fire for a failed `source` (it does for a bare failing command), so the guard prints the refusal
# itself rather than trapping. `a_wrapper_without_its_library_is_the_unjudged_class` holds the class by
# running this wrapper with the library removed.
if ! source "$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)/scripts/wrapper.sh"; then
    printf '%s: cannot read the shared wrapper library, so the tree the source gate judges and the one `cargo publish` would package cannot be located — which is not the same fact as a gate that ran and refused\n' "$WRAPPER_SUBJECT" >&2 || :
    exit 2
fi

# A value-taking flag's value is checked by the shared library's `value_refusal`, before the shift.
#
# **The sacrifice is `--jobs -N`, and it is named rather than left silent.** cargo documents a negative job
# count — *If negative, it sets the maximum number of parallel jobs to the number of logical CPUs plus
# provided value* — and measured, `cargo publish --jobs -1 --dry-run` packages and verifies normally. This
# script refuses it. Admitting it would need a per-arm rule, since a leading digit means nothing for
# `--package` or `--registry`, and a rule that differs per arm is the thing one shape check exists to avoid.
# The cost is one arithmetic step for the caller: pass the count. `repository-checks` carries it as a stated
# bound.
require_a_value() {
    local why
    why=$(value_refusal "$@") || cannot_judge "$why"
}

# This script's own root: the tree the gate judges, the manifest it is run from, and the directory
# `cargo publish` runs in. The helper in the shared library owns the acquisition and its guard; the message
# this wrapper adds is only what its own act would lose without it.
repo=$(wrapper_own_root) || cannot_judge \
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

# The ERR trap is installed BEFORE the parser, so an unguarded failure inside it is read as the
# cannot-judge it is rather than as the class a gate's refusal owns. The bootstrap guard above speaks for
# the one statement it covers, because an ERR trap does not fire for a failed `source`; this one covers
# everything after.
install_exit_class_trap

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
        require_a_value "$#" "$1" "${2-}"
        if [[ ${selection[0]} == --workspace ]]; then
            selection=()
        fi
        selection+=("$1" "$2")
        shift 2
        ;;
    --jobs | --color | --target-dir | --registry | --index)
        require_a_value "$#" "$1" "${2-}"
        forwarded+=("$1" "$2")
        shift 2
        ;;
    # The arms below decide nothing the catch-all would not — each is unlisted, so it is refused
    # either way. They exist to say WHY, because a refusal an operator cannot act on is a refusal they
    # work around. Each pattern covers cargo's glued and equals forms of the same flag.
    --manifest-path | --manifest-path=*)
        refuse "$1" "it moves cargo's workspace root away from the tree this gate judges, so the source gate would pass on $repo while cargo published something else. Publish from a checkout of the tagged \`chore(release): X.Y.Z\` commit on origin/main instead — cargo stamps the commit it ran on into every tarball, permanently"
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

open_verdict_file

# The source gate. It lives in Rust with the other repository gates and does not run in
# development — no development checkout is a release snapshot — so it is asked for explicitly here, the one
# moment it can answer. A failure aborts before `cargo publish`, which is the point: the act below is
# irreversible.

gate_output=$(TIANHENG_GATE_VERDICT=$verdict_file \
    TIANHENG_PUBLISH_SOURCE=1 TIANHENG_WORKSPACE_TESTS=1 \
    cargo test --manifest-path "$repo/Cargo.toml" -p kanhe --test publish_source \
    -- --exact the_publish_source_is_the_signed_release_snapshot 2>&1) || {
    exit_for_the_gates_refusal "$gate_output"
}
require_one_pass "$gate_output"
require_a_verdict

# Guarded like every acquisition, and for the sharper reason: this one runs **after** the gate has passed. A
# failed `cd` under `set -e` exits 1 — the class that means a gate ran and refused — so a wrapper that could
# not enter the tree would report a disagreement the gate never found, one line before `cargo publish`.
cd "$repo" || cannot_judge \
    "cannot enter $repo, the tree \`cargo publish\` would package, after the source gate had already passed \
— which is not the same fact as a gate that ran and refused"
# What the publish did, decided by `perform_the_act` rather than handed to cargo with `exec`: cargo's status is
# cargo's class, and it exits `1` on an argument it cannot parse — the class reserved for a gate that ran and
# refused. The allowlist above keeps every such argument away today; this is what keeps the class right without
# relying on that.
#
# Nothing is read back from the registry, so this account observes cargo's status and nothing else, and says so.
# A success adds no sentence of its own: cargo's output is all the record there is, and a line here would claim
# a reading this wrapper never made. A failure needs one, because a workspace publish uploads crate by crate —
# a run that stops may have published some of the crates it named, and which ones is unknown to this wrapper.
account_for_the_publish() {
    local status=$1
    if ((status != 0)); then
        cannot_judge "cargo publish exited $status without completing, after the source gate had agreed. Nothing was \
read back from the registry, so which of the crates it named were published is unknown — check crates.io before \
running this again, since a published version can be yanked but never replaced. This is not the same fact as a \
gate that ran and refused"
    fi
}

# `forwarded` may be empty, and `"${empty[@]}"` under `set -u` is an unbound variable before bash 4.4 —
# where it ends bash without the ERR trap and this wrapper would stop as a status no stop chose, a sentence
# about the wrong cause, on the argument-free invocation that is the ordinary one. `selection` is never empty
# and needs no guard. The `+` form is used rather than a version check, so no minimum has to be declared
# anywhere and kept in step.
perform_the_act account_for_the_publish cargo publish "${selection[@]}" ${forwarded[@]+"${forwarded[@]}"}
