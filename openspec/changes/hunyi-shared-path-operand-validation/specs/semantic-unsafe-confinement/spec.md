## ADDED Requirements

### Requirement: A malformed `::`-path allowed-subtree entry is a constitution error

An allowed-subtree entry given to `only_under([...])` SHALL be rejected as a **constitution error**
(exit 2) when its `::`-delimited spelling has any empty segment — a leading `::`, a trailing `::`, a
doubled `::`, or the empty string itself — checked alongside the existing empty-set and crate-root
guards, before any crate scanning. This is the identical restriction
`semantic-signature-coupling`'s "A malformed `::`-path forbidden operand is a constitution error"
requirement already places on the forbidden-operand family, read at the allowed-subtree polarity:
`matches_allowed`'s `::`-delimited containment can never equal or prefix-contain a real module
location against an operand shaped this way, so without this requirement a malformed entry would
not silently pass the boundary — the containment check already fails loud, since a site outside
every (non-matching) allowed entry is reported as a violation — but it would silently misreport
every genuinely-confined `unsafe` site as a spurious violation, naming no cause, rather than a clear
constitution error identifying the actual typo. There is no legitimate reason to write this shape:
no canonical module path this system ever resolves carries an empty segment, so the spelling is
always either inert or broken, never meaningfully different from the bare form.

#### Scenario: A leading-`::` allowed-subtree entry is a constitution error

- **WHEN** a boundary declares `only_under(["::crate::ffi"])` and the crate confines all `unsafe` genuinely inside `crate::ffi`
- **THEN** the system reports a constitution error (exit 2) naming the malformed entry, rather than reporting the genuinely-confined site as a spurious violation

#### Scenario: A trailing-`::` allowed-subtree entry is a constitution error

- **WHEN** a boundary declares `only_under(["crate::ffi::"])` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: A doubled-`::` allowed-subtree entry is a constitution error

- **WHEN** a boundary declares `only_under(["crate::ffi::::raw"])` against the same crate
- **THEN** the system reports a constitution error (exit 2), for the identical reason

#### Scenario: The bare-string spelling is unaffected

- **WHEN** a boundary declares `only_under(["crate::ffi"])` against the same crate
- **THEN** the system reports no violation for the genuinely-confined site, exactly as before this requirement existed
