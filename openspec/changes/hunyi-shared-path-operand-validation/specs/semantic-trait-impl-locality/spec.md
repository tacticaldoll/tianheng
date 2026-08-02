## ADDED Requirements

### Requirement: A malformed `::`-path allowed-location entry is a constitution error

An allowed-location entry given to `only_implemented_in`/`and_in` SHALL be rejected as a
**constitution error** (exit 2) when its `::`-delimited spelling has any empty segment — a leading
`::`, a trailing `::`, a doubled `::`, or the empty string itself — checked before any crate
scanning. This is the identical restriction `semantic-signature-coupling`'s "A malformed `::`-path
forbidden operand is a constitution error" requirement already places on `must_not_expose`'s
operand, read at the allowed-location polarity: `matches_allowed`'s `::`-delimited containment
(equality or a `prefix::`-boundary match) can never equal or prefix-contain a real module location
against an operand shaped this way, so without this requirement a malformed entry would not
silently pass the boundary — the containment check already fails loud, since a location outside
every (non-matching) allowed entry is reported as a violation — but it would silently misreport
every genuinely-in-place impl as a spurious violation, naming no cause, rather than a clear
constitution error identifying the actual typo. There is no legitimate reason to write this shape:
no canonical module path this system ever resolves carries an empty segment, so the spelling is
always either inert or broken, never meaningfully different from the bare form.

#### Scenario: A leading-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("::crate::commands")` and the crate defines `impl Command for Foo` genuinely inside `crate::commands`
- **THEN** the system reports a constitution error (exit 2) naming the malformed entry, rather than reporting the genuinely-in-place impl as a spurious violation

#### Scenario: A trailing-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("crate::commands::")` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: A doubled-`::` allowed-location entry is a constitution error

- **WHEN** a boundary declares `only_implemented_in("crate::commands::::sub")` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: The bare-string spelling is unaffected

- **WHEN** a boundary declares `only_implemented_in("crate::commands")` against the same crate
- **THEN** the system reports no violation for the genuinely-in-place impl, exactly as before this requirement existed
