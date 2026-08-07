## MODIFIED Requirements

### Requirement: The built-in path SHALL keep its behaviour, and the two paths SHALL be held equal

The reaction inspecting a Rust observer's `bounds()` body SHALL read it with Rust line-comment semantics. A `//`
line is prose and SHALL be excluded, while a Rust attribute beginning with `#` remains executed Rust text.

#### Scenario: A Rust attribute appears in an inspected body

- **WHEN** an inspected Rust body contains a line whose trimmed start is `#`
- **THEN** the reaction retains that line as Rust source rather than dropping it as a shell comment
