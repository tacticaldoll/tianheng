## MODIFIED Requirements

### Requirement: Default-safe reaction — a Violation event, panic opt-in

A reaction SHALL build a `xuanji::Violation` of kind **`Runtime`** (the shared measure: `target` = seam, `rule` = the allowlist rule, `finding` = offending origin + concrete type, with a severity) and by default **project it as a structured runtime event** (`Violation::to_json`) to a process-global **sink** the user can install (the system ships a default sink). A hard `panic` SHALL be **opt-in** (per boundary), never the default — a governance tool MUST NOT crash production on a false positive. A `warn`-severity boundary SHALL always be event-only.

The shipped default sink writes to stderr and MUST NOT itself panic if that write fails (a closed
or broken stderr — EPIPE on a closed pipe after its reader exits, EBADF on a closed fd) — the same
no-panic invariant the sink protects on its happy path. A failed default-sink write SHALL instead
increment a process-global, lock-free counter exposed as `dropped_sink_events() -> u64`, rather
than silently discarding all trace of the loss, so an adopter who relies on the default sink (has
never called `set_sink`) can detect from outside the process that an event went unobserved. The
counter increment itself SHALL be infallible — a single atomic add, never a lock, never able to
itself fail or panic — so closing this observability gap cannot reopen the panic risk it exists to
avoid. A custom sink's own success or failure is opaque to the system (`set_sink` takes a
`Fn(&Violation)` returning nothing) and is NOT counted — the counter is scoped to the shipped
default sink only.

#### Scenario: Default reaction emits an event, does not panic

- **WHEN** a boundary with default posture reacts
- **THEN** the system emits the `Violation` (kind `Runtime`) as json to the installed sink and the program continues (no panic)

#### Scenario: Panic is opt-in

- **WHEN** a boundary configured to panic on violation reacts
- **THEN** the system panics — only because panic was explicitly opted in

#### Scenario: A user-installed sink receives the event

- **WHEN** the user installs a custom sink and a boundary reacts
- **THEN** the custom sink receives the `Violation`, not the default sink

#### Scenario: A broken default-sink write is counted, not silently lost

- **WHEN** no custom sink is installed and the default sink's write to stderr fails (a closed or broken stderr)
- **THEN** the system does not panic, and `dropped_sink_events()` increases by exactly one for that violation
