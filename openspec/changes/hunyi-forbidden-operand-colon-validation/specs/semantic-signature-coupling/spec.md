## ADDED Requirements

### Requirement: A malformed `::`-path forbidden operand is a constitution error

A forbidden operand given to `must_not_expose`/`and_not_expose` SHALL be rejected as a **constitution
error** (exit 2) when its `::`-delimited spelling has any empty segment — a leading `::`, a trailing
`::`, a doubled `::`, or the empty string itself. This is a restriction on the **DSL operand string**
the developer writes, distinct from the "Requirement: Name resolution scope and no false negative"
section's leading-`::` guidance for the **source path being scanned**: that guidance is about how to
write `-> ::serde::Value` in the governed module's own code so it resolves as an unambiguous extern
rather than a local shadow; this requirement is about the separate `must_not_expose("...")` string,
which the resolver never produces with a leading `::` regardless of how the source is spelled (the
resolved canonical path of any extern exposure is always the bare form, e.g. `serde::Value`, never
`::serde::Value`). A forbidden operand shaped with an empty segment can therefore never equal or
prefix-contain any canonical path this system resolves, so without this requirement the boundary
would silently and permanently never react to it — the one class of bug this capability's core
contract forbids everywhere else it can occur. There is no legitimate reason to write a leading `::`
in this operand: no canonical path this crate ever produces carries one, so the spelling is always
either inert or broken, never meaningfully different from the bare form.

#### Scenario: A leading-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("::serde")` against a module exposing `-> ::serde::Value`, with `serde` a real dependency
- **THEN** the system reports a constitution error (exit 2) naming the malformed operand, rather than silently reporting the boundary satisfied

#### Scenario: A trailing-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("serde::")` against the same module
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: A doubled-`::` forbidden operand is a constitution error

- **WHEN** a boundary declares `must_not_expose("::serde::")` against the same module
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: The bare-string spelling is unaffected

- **WHEN** a boundary declares `must_not_expose("serde")` against the same module
- **THEN** the system emits a violation naming `serde::Value`, exactly as before this requirement existed
