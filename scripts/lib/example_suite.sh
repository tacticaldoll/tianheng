#!/usr/bin/env bash
# Repository-only completeness and temporary-artifact lifecycle for the isolated example suite.

declare -A FULFILLED_EXAMPLES=()
EXAMPLES_ROOT=
EXAMPLE_ARTIFACT_ROOT=

configure_example_suite() {
    EXAMPLES_ROOT=$1
}

repository_examples() {
    local manifest
    while IFS= read -r manifest; do
        basename "$(dirname "$manifest")"
    done < <(find "$EXAMPLES_ROOT" -mindepth 2 -maxdepth 2 -name Cargo.toml -type f | sort)
}

is_repository_example() {
    local candidate=$1 example found=1
    while IFS= read -r example; do
        if [[ $candidate == "$example" ]]; then
            found=0
        fi
    done < <(repository_examples)
    return "$found"
}

fulfill_example() {
    local example=$1
    if ! is_repository_example "$example"; then
        echo "::error::unknown repository example claim: $example" >&2
        return 1
    fi
    FULFILLED_EXAMPLES["$example"]=1
}

verify_example_coverage() {
    local example missing=0
    while IFS= read -r example; do
        if [[ -z ${FULFILLED_EXAMPLES[$example]+fulfilled} ]]; then
            echo "::error::repository example has no fulfilled reaction owner: $example" >&2
            missing=1
        fi
    done < <(repository_examples)
    if [[ $missing -ne 0 ]]; then
        return 1
    fi

    echo "ok  every repository example has a fulfilled reaction owner"
}

cleanup_example_artifacts() {
    if [[ -n ${EXAMPLE_ARTIFACT_ROOT:-} && -d $EXAMPLE_ARTIFACT_ROOT ]]; then
        rm -rf -- "$EXAMPLE_ARTIFACT_ROOT"
    fi
}

init_example_artifacts() {
    EXAMPLE_ARTIFACT_ROOT=$(mktemp -d "${TMPDIR:-/tmp}/tianheng-examples.XXXXXX")
    trap cleanup_example_artifacts EXIT
}
