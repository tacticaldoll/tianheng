# proposal: Louke's Default Sink Counts a Dropped Write

## Why

The 0.3.1 adversarial sweep found (`crates/louke/src/registry.rs:224`) that 漏刻's shipped default
sink discards a failed stderr write: `let _ = writeln!(std::io::stderr(), …)`. An adopter who never
calls `set_sink` — the exact adopter the default sink exists for — runs in a process whose stderr
write can fail for ordinary operational reasons: `myapp 2>&1 | consumer` after `consumer` exits
(EPIPE — Rust std ignores `SIGPIPE`), a daemon that closed its inherited fds, or a plain `myapp
2>&-` (EBADF). When that happens, an **enforce-severity runtime violation is silently and totally
lost** — no crash, no log line, no trace anywhere.

Reproduced directly: closed fd 2, installed one boundary, never called `set_sink`, then fired a
disallowed-origin crossing through the real `assert_boundary!` macro. The process did not crash (as
intended) but produced zero observable output and left no signal that a violation had occurred at
all.

The behavior is intentional at the code level — the comment beside it explains that `eprintln!`
would panic on a broken pipe, which would violate the crate's own no-panic-on-a-reaction invariant
(`Event` posture must never crash production) — but the trade-off itself is invisible: nothing in
the crate lets an adopter detect that it happened. 漏刻 ships no other diagnostics or health surface
today, so a dropped write is currently unobservable by any means.

This closes that gap with the smallest addition that makes the loss observable: a lock-free counter
that increments only when the default sink's write actually fails, plus a public accessor. It adds
no dependency, no lock, and — critically — cannot itself reopen the panic risk the silent-drop code
exists to avoid, since an atomic increment cannot fail.

## What Changes

- Extract the default sink's write into a small internal, writer-generic helper (`emit_default`) so
  the failure-counting logic is unit-testable against a fake always-failing writer, without touching
  the real process stderr.
- Add a process-global `AtomicU64` that counts a failed default-sink write, and a new public
  `louke::dropped_sink_events() -> u64` that exposes it. The increment is a single lock-free atomic
  add — infallible by construction.
- Scope stays narrow: only the *shipped default sink's own* write failure is counted. A custom
  sink's `Fn(&Violation)` returns nothing, so its success or failure is opaque to the system and
  stays uncounted, unchanged from today.
- Update `openspec/specs/runtime-origin-assertion/spec.md`'s "Default-safe reaction" requirement so
  the counter is a stated, tested capability, not just an implementation comment.
- Update `PROJECT.md`'s 漏刻 decision text so it no longer implies a dropped default-sink write is
  totally silent — it is now a discoverable, poll-based signal.

## Capabilities

### Modified Capabilities

- `runtime-origin-assertion`: the "Default-safe reaction — a Violation event, panic opt-in"
  requirement gains a paragraph and a scenario documenting that a failed default-sink write is
  counted (`dropped_sink_events`) rather than silently and permanently lost, while the no-panic
  guarantee is preserved.

## Impact

- `crates/louke/src/registry.rs`: `emit` delegates to a new `emit_default` helper; a new
  `DROPPED_SINK_EVENTS` static and public `dropped_sink_events()` accessor.
- `crates/louke/src/lib.rs`: re-export `dropped_sink_events`.
- `crates/louke/src/tests.rs`: a unit test pinning the counting logic with a fake always-failing
  writer.
- `crates/louke/tests/`: a new `#[cfg(unix)]`-gated integration test reproducing the original
  finding end-to-end — closed fd 2, no custom sink, one fired violation, counter up by exactly one,
  no panic.
- `openspec/specs/runtime-origin-assertion/spec.md`, `PROJECT.md`, `CHANGELOG.md`.
- Non-breaking and additive-only: the only public surface change is one new free function; no
  existing signature, behavior, or wire format changes shape.
