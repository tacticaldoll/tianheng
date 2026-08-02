## ADDED Requirements

### Requirement: Lexical hygiene never panics on malformed source

The system SHALL react 0/1/2 on any governed source file, including one whose lexical structure is
malformed in a way rustc itself would reject (an unterminated block comment, or any other unclosed
construct reaching end-of-file) — never panicking or otherwise aborting the process. An unterminated
block comment SHALL be treated as extending through end-of-file: every byte within it, including a
trailing byte that would otherwise be the orphaned tail of a multi-byte character, is consumed as
part of the comment rather than re-scanned as code.

#### Scenario: An unterminated block comment swallowing a multi-byte character does not panic

- **WHEN** a governed source file ends in an unterminated block comment (no closing `*/`) whose
  dropped content includes a multi-byte UTF-8 character with no trailing newline after it
- **THEN** the system reacts with a normal violation or clean outcome instead of panicking

#### Scenario: An unterminated block comment at end of file does not panic

- **WHEN** a governed source file ends in an unterminated block comment with no trailing newline,
  regardless of what precedes it
- **THEN** the system reacts 0/1/2 instead of panicking, and every module declared before the
  comment remains observable
