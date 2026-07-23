#!/usr/bin/env bash
# Repository-only inventory for the public boundary-family dogfood contract.
#
# A family is added here only after OpenSpec/API review classifies a public insertion path as a
# family rather than a modifier, depth, or shorthand. This is test governance, never product API or
# report metadata. Call `fulfill_family` only after the owner's real evaluator and structured
# reaction assertions have succeeded; `verify_family_coverage` closes the ledger at the end.

PUBLISHED_BOUNDARY_FAMILIES=(
    guibiao/crate
    guibiao/module
    hunyi/signature
    hunyi/trait-impl
    hunyi/visibility
    hunyi/forbidden-marker
    hunyi/dyn-trait
    hunyi/impl-trait
    hunyi/async-exposure
    hunyi/unsafe
    tianheng/sans-io-pure
    tianheng/no-existential-leak
    louke/runtime
)

declare -A FULFILLED_BOUNDARY_FAMILIES=()

is_published_family() {
    local candidate=$1 family
    for family in "${PUBLISHED_BOUNDARY_FAMILIES[@]}"; do
        if [ "$candidate" = "$family" ]; then
            return 0
        fi
    done
    return 1
}

fulfill_family() {
    local family=$1
    if ! is_published_family "$family"; then
        echo "::error::unknown published boundary family claim: $family" >&2
        return 1
    fi
    FULFILLED_BOUNDARY_FAMILIES["$family"]=1
}

verify_family_coverage() {
    local family missing=0
    for family in "${PUBLISHED_BOUNDARY_FAMILIES[@]}"; do
        if [ -z "${FULFILLED_BOUNDARY_FAMILIES[$family]+fulfilled}" ]; then
            echo "::error::published boundary family has no fulfilled reaction owner: $family" >&2
            missing=1
        fi
    done
    if [ "$missing" -ne 0 ]; then
        return 1
    fi

    echo "ok  every published boundary family has a fulfilled reaction owner"
}
