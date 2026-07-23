## 1. Scanner primitives (crates/louke/src/audit/scan.rs)

- [x] 1.1 Add a helper that finds the end of the first macro argument from a given start position
      (top-level comma or matching close-delimiter over all three delimiter kinds, `(`/`{`/`[`).
      Model this on `foreign_macro_body_end`'s depth-counter loop (NOT `balanced_brace_end`, which
      is curly-only and would truncate a seam expression containing a nested call or index, e.g.
      `assert_boundary!(some_fn(a, b), obj)`), reusing `skip_literal_or_comment` so nested
      strings/comments don't confuse the scan.
- [x] 1.2 Thread this helper into all three `capture_probe` un-auditable branches (raw-string decode
      failure, plain-string decode failure, non-string-literal first token) to capture the trimmed
      expression text.
- [x] 1.3 Add an **owner-qualified enclosing-item** tracker to the source walk (brace-depth tracked,
      `syn`-free, reusing existing delimiter/comment-skip primitives) and thread it through to each
      `Probe::Unauditable` construction site. This must NOT be a bare innermost name — mirror
      `hunyi`'s owner/`trait_ref` qualification shapes:
      - free `fn`: module path + fn name
      - method inside `impl Type { … }`: `Self` type + method name
      - method inside `impl Trait for Type { … }`: trait path + `Self` type + method name
      - trait's own default-body method: trait name + method name
- [x] 1.4 Extend `Probe::Unauditable` with the two new fields (owner-qualified enclosing item,
      expression text) alongside the existing `file`.

## 2. Identity wiring

- [x] 2.1 Extend `RuntimeFact::UnauditableProbe` (`crates/louke/src/finding.rs`) with the two new
      named identity fields; update its `key(...)` call and human finding text.
- [x] 2.2 Change `audit.rs::audit_probe_coverage`'s un-auditable-probe collection/dedup from
      `Vec<&str>` (file only) to a 3-tuple `(file, enclosing item, expression text)`, sorted and
      deduped the same way.
- [x] 2.3 Confirm the emitted violation's human finding text still names the file (via
      `.with_file(...)`) and now also surfaces the enclosing item / expression for legibility.

## 3. Tests

- [x] 3.1 Two distinct non-literal expressions in the same file, same function → two violations,
      baselining one does not suppress the other.
- [x] 3.2 Same expression text in two different free functions in the same file → two violations.
- [x] 3.3 Same-named method in two different `impl Type` blocks, identical expression text → two
      violations distinguished by owner (the collision the original bare-name design missed).
- [x] 3.4 Same-named method in two different `impl Trait for Type` blocks on the same `Self` type,
      identical expression text → two violations distinguished by trait.
- [x] 3.5 Identical expression repeated verbatim in the same function/method → one violation
      (stated bound scenario).
- [x] 3.6 Same expression text in two different files → two violations, distinguished by file.
- [x] 3.7 Existing un-auditable-probe tests (raw-string decode failure, plain-string decode
      failure, non-literal-token cases) still pass with the extended identity shape.
- [x] 3.8 `cargo test -p louke --all-features` (the `audit` feature gates this code path).
- [x] 3.9 (found during apply-phase adversarial review) Same-named free `fn` in two different
      inline `mod` blocks of the same file → two violations, distinguished by module path. The
      first cut of the owner tracker qualified free fns by a bare name only, missing the module
      path task 1.3 already called for; caught empirically by an independent review, fixed by
      threading an inline-`mod`-nesting stack alongside the existing impl/trait context stack.

## 4. Spec and documentation sync

- [x] 4.1 Sync `openspec/specs/runtime-origin-assertion/spec.md` with this change's ADDED
      requirement (via `openspec-sync-specs` / archive flow, not a hand copy).
- [x] 4.2 Resolve and remove `BACKLOG.md`'s "Un-auditable-probe finding identity is file-granular"
      accepted-debt entry now that it is closed — explicitly note that its recorded remediation
      sketch ("byte offset / occurrence index") was superseded by the 0.3.0 no-positional-identity
      rule, not silently replaced.
- [x] 4.3 Add a `CHANGELOG.md` `[Unreleased]` entry describing the false-negative closure (patch,
      per the v0.1.3 re-export-exposure precedent — not a breaking change).

## 5. Verification (Definition of Done, per AGENTS.md)

- [x] 5.1 `cargo build --workspace`
- [x] 5.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [x] 5.3 `cargo clippy --workspace -- -D warnings`
- [x] 5.4 `cargo clippy -p louke -- -D warnings` (audit-OFF, prod-light build)
- [x] 5.5 `cargo fmt --all --check`
- [x] 5.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [x] 5.7 `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [x] 5.8 `cargo deny check`
- [x] 5.9 `bash scripts/test_release_coherence.sh` / `bash scripts/check_release_coherence.sh`
- [x] 5.10 `bash scripts/test_examples.sh`
