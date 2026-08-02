# tasks: Louke's Default Sink Counts a Dropped Write Implementation Plan

Design decisions and rationale carried from `design.md` — none of this needs re-deriving.

## 1. Prod-face counter

- [ ] 1.1 Add `use std::sync::atomic::{AtomicU64, Ordering};` and a `DROPPED_SINK_EVENTS: AtomicU64` static to `crates/louke/src/registry.rs`, beside `SINK`. <!-- id: 0 -->
- [ ] 1.2 Extract the default sink's write into `fn emit_default(w: impl std::io::Write, violation: &Violation)`, incrementing `DROPPED_SINK_EVENTS` on a write error; have `emit`'s `None` arm call it with `std::io::stderr()`. <!-- id: 1 -->
- [ ] 1.3 Add `pub fn dropped_sink_events() -> u64` reading the counter with `Ordering::Relaxed`, documented as scoped to the shipped default sink only (a custom sink's failures are opaque and stay uncounted). <!-- id: 2 -->
- [ ] 1.4 Re-export `dropped_sink_events` from `crates/louke/src/lib.rs` alongside `install`/`set_sink`. <!-- id: 3 -->

## 2. Regression coverage

- [ ] 2.1 Unit test in `crates/louke/src/tests.rs`: a fake always-erroring `Write` implementation drives `emit_default`, asserting `dropped_sink_events()` increases by exactly one and the call does not panic. <!-- id: 4 -->
- [ ] 2.2 Integration test (new file under `crates/louke/tests/`, `#[cfg(unix)]`-gated): close real fd 2 via a raw `extern "C" { fn close(fd: i32) -> i32; }` declaration, install a boundary with no custom sink, fire a disallowed-origin crossing through `assert_boundary!`, and assert `dropped_sink_events()` increased by exactly one and the process did not panic — the original finding's exact reproduction, now pinned. <!-- id: 5 -->
- [ ] 2.3 Confirm the existing `prod_face_end_to_end` / `sink_twice` / `integration.rs` tests are unaffected (a healthy stderr write still counts zero drops). <!-- id: 6 -->

## 3. Spec and documentation

- [ ] 3.1 MODIFY the `runtime-origin-assertion` "Default-safe reaction — a Violation event, panic opt-in" requirement: reproduce it verbatim and add the counter paragraph plus one new scenario, keeping the three existing scenarios intact. <!-- id: 7 -->
- [ ] 3.2 Update `PROJECT.md`'s 漏刻 identity-coherence decision text so it does not read as promising a dropped default-sink write is undetectable. <!-- id: 8 -->
- [ ] 3.3 Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry naming the closed silent-loss gap and the new `dropped_sink_events()` accessor. <!-- id: 9 -->
- [ ] 3.4 Sweep 漏刻's own prose (crate doc, README, nearby comments) for language implying the default sink's write failure is silent with no trace, so no file keeps stating the pre-change behavior. <!-- id: 10 -->

## 4. Verification

- [ ] 4.1 After `openspec archive`, verify the synced main spec against the delta (character count / scenario count) rather than trusting the merge blindly. <!-- id: 11 -->
- [ ] 4.2 Run the full Definition of Done from the workspace root and report actual output: `cargo build --workspace`; `cargo clippy --all-targets --all-features -- -D warnings`; `cargo clippy --workspace -- -D warnings`; `cargo clippy -p louke -- -D warnings`; `cargo fmt --all --check`; `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`; `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`; `cargo deny check`; `bash scripts/test_release_coherence.sh`; `bash scripts/check_release_coherence.sh`; `bash scripts/test_examples.sh`. <!-- id: 12 -->
