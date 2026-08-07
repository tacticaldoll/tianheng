## Context

The current repository test scans every Rust line comment under the shell and rejects the authored DSL token,
then separately requires the ANSI rationale to point to the generated projection. It does not inspect the
boundary's live `allowed` values, so a prose-only copy of the same members remains invisible.

## Goals / Non-Goals

**Goals:**

- Select the unique shell dependency boundary once and reuse it for both comment and fixture reactions.
- Detect a full current-member copy across one contiguous line-comment block.
- Derive member tokens from the live rule and prove the copied-members form fails.

**Non-Goals:**

- Ban individual dependency names from comments or parse natural language.
- Scan block comments, Markdown, or executable Rust expressions.
- Amend the accepted shell dependency boundary.

## Decisions

Extract a helper returning the unique live shell dependency boundary. The comment reaction reads its
`RestrictDependenciesTo { allowed }` values, tokenizes each contiguous `//` comment block using Rust/crate-name
identifier characters, and fails when every live member appears in the block. Contiguous aggregation catches
the original multiline copy shape without treating separated architectural comments as one list.

Retain the existing literal DSL-token refusal: it detects a declaration copy even if the live allowlist is empty
or the copied call is incomplete. The two checks observe distinct authored forms and share the same source scan.

## Risks / Trade-offs

- A partial member list remains allowed because it may be ordinary dependency-specific rationale; the guard owns
  a full allowlist census, not prose semantics.
- A copy assembled through block comments is outside this line-comment contract. The scenario and test name stay
  explicit about that perimeter.
