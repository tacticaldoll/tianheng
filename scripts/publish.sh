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
# Extra arguments are passed through to cargo, EXCEPT the one that would move the workspace root out
# from under the gate. `--manifest-path` points cargo at another tree, so the gate would judge this
# repository and cargo would publish a different one — the wrapper's whole claim, in a single
# argument. It was stated as an accepted bound here rather than refused, on the reasoning that the
# caller is the one human authorized to publish at all; the gate beside this one exists because
# "publish `main`, not the release branch" was likewise stated and then missed in the same window it
# was written, and a rule that has been stated and then missed needs a reaction rather than another
# sentence. Four lines is what it costs, and no legitimate publish passes it.
#
# What is still forwarded, deliberately: `-p`/`--package`, `--dry-run`, `--allow-dirty` (which the
# gate refuses upstream anyway), `--target-dir`, and the registry-side arguments `--registry`,
# `--index`, and `--token`. Those last three change the publish's DESTINATION, not its source, which
# is a different claim from the one this wrapper and its gate make — so they stay ungated, and that is
# written down rather than implied.
set -euo pipefail

repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)

# Before the gate, not after: a refusal must not depend on the gate's own verdict, and reading the
# arguments is the cheapest of the two. Both spellings, since `--manifest-path=PATH` and
# `--manifest-path PATH` reach cargo identically and a guard that caught one would be a guard that
# caught neither.
for arg in "$@"; do
    case $arg in
    --manifest-path | --manifest-path=*)
        printf 'publish source: %s\n' \
            "refusing \`$arg\`: it moves cargo's workspace root away from the tree this gate judges, so the source gate would pass on $repo while cargo published something else. Publish from a checkout of the tagged \`release: X.Y.Z\` commit on origin/main instead — cargo stamps the commit it ran on into every tarball, permanently" >&2
        exit 1
        ;;
    esac
done

# The source gate. It lives in Rust with every other self-governance reaction and does not run in
# development — no development checkout is a release snapshot — so it is asked for explicitly here, the one
# moment it can answer. A failure aborts before `cargo publish`, which is the point: the act below is
# irreversible.
TIANHENG_PUBLISH_SOURCE=1 TIANHENG_WORKSPACE_TESTS=1 \
    cargo test --manifest-path "$repo/Cargo.toml" -p tianheng --test publish_source \
    -- --exact the_publish_source_is_the_signed_release_snapshot

cd "$repo"
exec cargo publish --workspace "$@"
