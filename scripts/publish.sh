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
# verification, and the bundled-license check all run before anyone arrives here. Extra arguments are
# passed through to cargo — which bounds what this wrapper claims: the gate judged THIS repository, so
# an argument that points cargo at another tree (`--manifest-path`) publishes something the gate never
# read. That is not guarded against, because the caller is the one human authorized to publish at all;
# it is written down so the guarantee is not read as wider than it is.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

bash "$repo/scripts/check_publish_source.sh" "$repo"

cd "$repo"
exec cargo publish --workspace "$@"
