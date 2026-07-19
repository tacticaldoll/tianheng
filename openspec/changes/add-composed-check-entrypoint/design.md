## Context

The runner already evaluates static boundaries with coverage, the full semantic bundle, and runtime
probe coverage in a deterministic order before applying baseline/presentation concerns. That
composition is private inside CLI dispatch. Consequently the composed example must reconstruct two
dimension checks merely to inspect an `Outcome`, and does not test the runtime CI face through the
same call.

The new adopter-surface contract distinguishes reaction inspection from process presentation. A
composed library check is the missing bridge; a macro testing harness remains a separate deferred
ergonomics decision.

## Goals / Non-Goals

**Goals:**

- Return one inspectable `Outcome` for the same unified `Constitution` accepted by `run`.
- Make CLI and library evaluation share dimension ordering, merge semantics, and error precedence.
- Continue auditing runtime probes even when no runtime boundaries are declared, catching orphans.
- Add the entrypoint without breaking static `check`, `run`, or adopter declarations.

**Non-Goals:**

- An inline fixture format, temp-workspace builder, or assertion macro.
- Baseline loading/writing, coverage warnings, formats, current-directory discovery, or output.
- Making filesystem/cargo-metadata observation "pure"; presentation-free means no shell effects,
  not no observation effects.
- Moving observation code between dimensions or changing any finding identity.

## Decisions

### Extract one evaluator beneath both entrypoints

Create a private runner evaluator returning `(Outcome, Option<Coverage>)`. Move the existing
static→semantic→runtime composition into it without changing order or guards. `run` consumes both
values for its existing coverage/baseline/presentation flow; public `check_constitution` returns the
raw Outcome and discards coverage.

Alternative considered: call CLI `run` with synthetic arguments and capture output. Rejected
because process presentation is exactly what library tests need to avoid, and `ExitCode` loses the
inspectable findings.

Alternative considered: independently call each public dimension check in the new function.
Rejected because it would create a second composition implementation that can drift from CLI
semantics.

### Require an explicit manifest path

The API accepts `&Path`, matching dimension checks. Nearest-manifest discovery remains CLI
convenience; library callers and tests must make the observed workspace explicit.

### Return raw, unbaselined Outcome

The function does not accept baseline or coverage flags. Baselines are a gate mode over an observed
Outcome, coverage is an advisory, and formats are projections. Keeping those in `run` leaves the
library reaction deterministic over `(Constitution, manifest workspace)` and avoids a second option
surface.

### Put the function in the inspection tier

Re-export `check_constitution` at the crate root and wildcard prelude. Retain prelude `check` as the
pure static-core entrypoint; the longer name prevents a breaking rename and makes the composed scope
explicit.

## Risks / Trade-offs

- **Runner verdict drifts during extraction.** → Move the existing block intact and add parity tests
  for merged violations, error precedence, runtime orphan probes, and CLI exit behavior.
- **Callers expect no I/O from a library function.** → Rustdoc states that it observes the manifest
  through cargo metadata and source scans; only presentation/process effects are absent.
- **The prelude becomes broader.** → One function closes a demonstrated gap and is compile-reacted
  by the adopter-surface test; no testing framework is pre-built around it.
- **Coverage is silently lost to callers.** → Document that this API returns reaction only and keep
  `run --warn-uncovered` as the coverage surface.

## Migration Plan

Additive: existing callers need no migration. Update the composed example to the unified function.
Rollback restores the private dispatch block and removes the new function/spec/example use; no data
is migrated.

## Open Questions

None. The existing runner path fixes the semantics and the example fixes the first consumer shape.
