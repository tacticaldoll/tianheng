## MODIFIED Requirements

### Requirement: Every enumerated gate SHALL have a companion failure matrix holding five properties

Every property about executed gate or twin behavior SHALL be read with shell line-comment semantics: a line whose
trimmed start is `#` is prose, while Rust's `//` marker SHALL NOT be treated as a shell comment. Header and prose
properties SHALL continue to use their dedicated regions.

#### Scenario: A shell comment claims an executed property

- **WHEN** a required gate or twin form appears only on a line whose trimmed start is `#`
- **THEN** the reaction does not count it as executed shell text
