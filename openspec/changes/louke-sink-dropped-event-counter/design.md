## Context

`crates/louke/src/registry.rs`'s `emit` function reacts to every `Violation` under the default
`Event` posture, before the opt-in panic gate. When no custom sink is installed it falls back to a
shipped default sink that writes the violation as JSON to stderr and, by design, never panics if
that write fails — `let _ = writeln!(std::io::stderr(), …)` discards the `Result` outright. The
comment beside it is honest about *why* (a broken pipe must not crash production), but the crate
has no mechanism anywhere — no counter, no health check, no diagnostics endpoint — for an adopter
to learn that a write actually failed. Confirmed by inspection: 漏刻 exposes no metrics, health, or
introspection surface today beyond the `Violation` event stream itself.

## Goals / Non-Goals

**Goals:**
- Make a failed default-sink write observable from outside the process, without adding a
  dependency, a lock, or reopening the panic risk the silent-drop code exists to avoid.
- Keep the fix scoped to the exact verified defect — the shipped default sink's own write — not a
  general sink-health framework.
- Preserve std-light: the fix must compile and work in louke's default (`audit`-off) build, the
  configuration a production binary actually ships.

**Non-Goals:**
- A general metrics/health-check subsystem for 漏刻. No adoption pressure is recorded for one, and
  the drift law forbids speculative API surface; this stays a single counter closing a single
  verified gap.
- Counting or detecting a *custom* sink's failures. `set_sink` takes an opaque `Fn(&Violation)`
  that returns nothing, so the system cannot observe whether it succeeded — unchanged.
- Retrying the write or falling back to a second output channel (considered and rejected below).
- The separately-floated `Tracked::type_name()` enhancement for unregistered-crossing messages —
  a different, unverified-as-a-defect finding, out of scope here.

## Decisions

### Decision 1: A process-global `AtomicU64` counter with a public accessor — not a fallback write target

Two shapes were on the table:

- **A fallback write target** (retry once, or write to a second fd/file) adds a *second*
  failure-prone surface: the fallback itself can be just as broken as stderr was (a container with
  every standard fd closed, a sandbox with no writable path), which would only move the silent-drop
  bug one hop over rather than closing it, and it does not fit "std-light" any better — it still
  needs somewhere to fall back to that the crate cannot assume exists in every deployment.
- **A counter** is observable-but-passive: it does nothing unless something reads it. Because 漏刻
  ships no existing metrics/health/diagnostics mechanism today, a bare *private* counter would not
  actually close the observability gap — it would relocate it from "invisible inside the write
  path" to "invisible inside an unread static." The fix therefore has to ship the accessor
  (`dropped_sink_events()`) along with the counter, or it does not earn its keep.

Decision: `AtomicU64` plus `pub fn dropped_sink_events() -> u64`. An adopter who ships the default
sink polls this into whatever health check, periodic log line, or metrics scrape they already run —
the crate does not need to know what that destination is, consistent with `xuanji`'s own
dimension-agnostic scoping (it carries the reaction model, not an observation engine).

### Decision 2: Lives in the std-light prod face, not behind the `audit` feature

`PROJECT.md`'s 漏刻 identity-coherence decision draws the line at *what runs where*: "Prod face
(`assert_boundary!`) is std-light and fail-closed; CI face (`audit_probe_coverage`) is
feature-gated behind `audit`." The default sink and its write failure happen in a **running
production process**, not at CI/build time, so this is prod-face machinery by definition — gating
it behind `audit` would make the fix unavailable in exactly the configuration the finding was
reproduced in (the default, `audit`-off build). It adds no new dependency: `std::sync::atomic`
only.

### Decision 3: The increment is infallible under contention

`AtomicU64::fetch_add(1, Ordering::Relaxed)` cannot fail or panic — even under contention, and even
on overflow, where it silently wraps rather than panicking — and takes no lock, matching the
registry's existing no-lock posture (its `TypeId` map read is lock-free for the same reason).
`Relaxed` ordering is sufficient: the counter synchronizes no other memory, is read independently
via `load`, and an adopter is expected to poll it rather than treat a specific value as a
synchronization point.

### Decision 4: Test the counting logic without touching the real process stderr

`emit`'s default-sink write is extracted into `emit_default(w: impl std::io::Write, violation:
&Violation)` so a unit test can pin "a failing writer increments the counter by exactly one" with a
fake, always-erroring `Write` implementation — portable, no `unsafe`, no new dev-dependency. The
finding's *original* reproduction (closing real fd 2) is kept too, as a `#[cfg(unix)]`-gated
integration test, since it is the only test exercising the real `assert_boundary!` → `__react` →
`emit` → OS-broken-stderr path end to end. It uses a raw `extern "C" { fn close(fd: i32) -> i32;
}` declaration rather than a `libc` dev-dependency: the workspace carries no dev-dependencies
anywhere today, and CI runs exclusively on `ubuntu-latest`.

## Risks / Trade-offs

- The counter can in principle wrap at `u64::MAX` dropped writes — accepted; that is an
  implausible number of stderr failures for any real process lifetime, and wrapping (not panicking)
  is the correct failure mode for a passive counter.
- Nothing pushes an adopter to poll `dropped_sink_events()` — it is a floor, not a full metrics
  subsystem. Accepted per the stated non-goal: building a push-based mechanism has no recorded
  adoption pressure, and would be exactly the speculative API-surface growth the drift law forbids.
- The public API grows by one free function. Narrowly scoped to the exact verified defect
  (`registry.rs:224`), unlike the `Tracked::type_name()` idea floated during investigation, which
  was declined as speculative against an already-refuted finding.
