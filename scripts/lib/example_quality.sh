#!/usr/bin/env bash
# Rust quality gates for the current isolated example workspace.
# Arguments after the label are Cargo `--config patch...` options shared with that example's tests.

quality_gates() {
    local label=$1
    shift

    cargo fmt --all --check || return $?
    cargo clippy --all-targets "$@" -- -D warnings || return $?
    RUSTDOCFLAGS="-D warnings" cargo doc --no-deps "$@" || return $?
    echo "ok  $label passes isolated fmt, Clippy, and rustdoc gates"
}
