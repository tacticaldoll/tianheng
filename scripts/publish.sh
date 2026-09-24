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
# actually HONOURS it beside what this script supplies itself — and `--package` failed that one silently:
# written after an unconditional `--workspace`, cargo discarded it and published everything. Classify
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
    printf '%s: cannot read the shared wrapper library, so the tree the source gate judges and the one `cargo publish` would package cannot be located — which is not the same fact as a gate that ran and refused\n' "$WRAPPER_SUBJECT" >&2
    exit 2
fi

# A value-taking flag's value, checked for both ways it can be wrong: absent, and flag-shaped.
#
# **A value position is not a place a refused flag may sit.** The arms below checked only that *something*
# followed, and cargo does not consume a flag-shaped value — measured on cargo 1.96.0,
# `cargo publish --package --no-verify` packages **without verifying**, byte-identical to passing
# `--no-verify` alone, and exits 0 with no complaint about a package by that name. So every refusal this file
# argues for was reachable through the one selector it admits.
#
# Held for every value-taking arm rather than the one that leaked. cargo's own handling differs per flag —
# some consume the value and fail later, some are refused by clap — but that is a fact about cargo's error
# paths at one version, and a wrapper standing in front of an irreversible act does not rest on the tool
# failing correctly. The property is the script's own: an argument it did not name does not travel, wherever
# it was written.
#
# Checked by SHAPE rather than against the refusal list, because the list is not the property, and the refusal
# SAYS the shape rather than explaining cargo. The first form told the operator that cargo *reads the value as
# an argument of its own* — true of `--package --no-verify`, and false of most of what it stops: measured,
# `--jobs --allow-dirty` has cargo consume the value and fail later with `could not parse --allow-dirty`, and
# `--registry --config` is refused by clap with `a value is required for '--registry <REGISTRY>'`. Three
# mechanisms, one sentence, so the sentence was wrong twice. What is true of every arm is this script's own
# property: it does not accept a value beginning with `-`.
#
# **The sacrifice is `--jobs -N`, and it is named rather than left silent.** cargo documents a negative job
# count — *If negative, it sets the maximum number of parallel jobs to the number of logical CPUs plus
# provided value* — and measured, `cargo publish --jobs -1 --dry-run` packages and verifies normally. This
# script refuses it. Admitting it would need a per-arm rule, since a leading digit means nothing for
# `--package` or `--registry`, and a rule that differs per arm is the thing one shape check exists to avoid.
# The cost is one arithmetic step for the caller: pass the count. `repository-checks` carries it as a stated
# bound.
require_a_value() {
    if (($1 < 2)); then
        refuse "$2" "this script reads every value as the argument after its flag, so pass it that way or drop the flag"
    fi
    if [[ $3 == -* ]]; then
        refuse "$2" "its value is \`$3\`, and this script does not accept a value beginning with \`-\`. It does not read cargo's handling of a flag-shaped value, which differs by flag and by version, so it refuses the shape instead. Pass a value, or drop the flag"
    fi
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
