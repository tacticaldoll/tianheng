## Context

The reaction builds a corpus from `git ls-files`, but filters it to `.md` and `.rs`, despite its module contract speaking about documents and comments. It then excludes every line of any path containing `/tests/`, even though later branches already carry narrower exclusions for illustrative script and example targets. This hides current stale claims in test comments. The inspected-file counter is also incremented both before and after exclusions.

Separately, the capability accurately states that cargo test does not preserve a three-way exit vocabulary, then later requires exit 0/1/2 and stdout/stderr shapes. The latter requirement cannot be implemented by this reaction.

## Goals / Non-Goals

**Goals:**

- Observe the tracked comment-bearing formats where current stale references exist.
- Make test-source exclusions specific to constructed fixture shapes rather than file-wide.
- Preserve fail-loud reads, active-plan exclusion, record-document exclusion, and read-only operation.
- State pass/fail behavior at the level cargo test actually exposes.

**Non-Goals:**

- Do not claim to parse every comment syntax or every repository file type.
- Do not add a general prose detector or a second reference reaction.
- Do not repair publish-source vocabulary or unrelated test comments in this change.
- Do not address the repository-wide predictable temporary-directory pattern.

## Decisions

### Declare one inspectable-source predicate

A single predicate will admit `.md`, `.rs`, `.toml`, and files named `.gitignore`. The corpus enumeration and its zero-source diagnostic will use the same predicate, and each admitted path increments `inspected` exactly once before the active-plan exclusion.

The alternative—declaring a bound for TOML and ignore comments—would preserve known stale references and contradict the reaction's purpose where a concrete tracked observation source already exists.

### Observe comment regions rather than test identity

Rust lines whose first non-whitespace token is `//`, including rustdoc forms, will be read regardless of whether the file is a test. Rust string literals and executed code are not repository prose claims and remain outside this reader. TOML and `.gitignore` likewise contribute lines beginning with their `#` marker after indentation, while Markdown contributes its document text. Existing local exclusions remain only at the reference shapes they justify: illustrative `scripts/` or `examples/` targets in fixture links, non-member crate shapes, and active OpenSpec plans. A current reference to a deleted real repository path will therefore react regardless of whether it sits in a test comment.

### Describe the cargo-test reaction in pass/fail terms

The read-only requirement will be renamed and rewritten. Clean references pass; stale references are aggregated into a failing assertion; missing prerequisites and observation failures fail loudly and name the read. No stdout or exact process-exit code is promised.

## Risks / Trade-offs

- **Widening the corpus exposes historical record prose** → Keep `docs/history/` and dated CHANGELOG sections excluded as records.
- **Fixture strings become false positives** → Retain only decidable shape-level exclusions and add a negative test showing a live stale path in `/tests/` still reacts.
- **Corpus wording outruns implementation again** → Name the admitted formats and comment regions explicitly in both predicate-backed scenarios and module prose; block comments and trailing inline comments are not claimed.
- **A widened run finds live stale comments** → Repair each toward the current Rust reaction or workflow path in the same apply increment.
