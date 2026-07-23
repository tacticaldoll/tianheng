## 1. Scanner primitives (crates/louke/src/audit/scan.rs)

- [ ] 1.1 Add a helper that finds the end of the first macro argument from a given start position
      (top-level comma or matching close-delimiter over all three delimiter kinds, `(`/`{`/`[`).
      Model this on `foreign_macro_body_end`'s depth-counter loop (NOT `balanced_brace_end`, which
      is curly-only and would truncate a seam expression containing a nested call or index, e.g.
      `assert_boundary!(some_fn(a, b), obj)`), reusing `skip_literal_or_comment` so nested
      strings/comments don't confuse the scan.
- [ ] 1.2 Thread this helper into all three `capture_probe` un-auditable branches (raw-string decode
      failure, plain-string decode failure, non-string-literal first token) to capture the trimmed
      expression text.
- [ ] 1.3 Add an **owner-qualified enclosing-item** tracker to the source walk (brace-depth tracked,
      `syn`-free, reusing existing delimiter/comment-skip primitives) and thread it through to each
      `Probe::Unauditable` construction site. This must NOT be a bare innermost name — mirror
      `hunyi`'s owner/`trait_ref` qualification shapes:
      - free `fn`: module path + fn name
      - method inside `impl Type { … }`: `Self` type + method name
      - method inside `impl Trait for Type { … }`: trait path + `Self` type + method name
      - trait's own default-body method: trait name + method name
- [ ] 1.4 Extend `Probe::Unauditable` with the two new fields (owner-qualified enclosing item,
      expression text) alongside the existing `file`.

## 2. Identity wiring

- [ ] 2.1 Extend `RuntimeFact::UnauditableProbe` (`crates/louke/src/finding.rs`) with the two new
      named identity fields; update its `key(...)` call and human finding text.
- [ ] 2.2 Change `audit.rs::audit_probe_coverage`'s un-auditable-probe collection/dedup from
      `Vec<&str>` (file only) to a 3-tuple `(file, enclosing item, expression text)`, sorted and
      deduped the same way.
- [ ] 2.3 Confirm the emitted violation's human finding text still names the file (via
      `.with_file(...)`) and now also surfaces the enclosing item / expression for legibility.

## 3. Tests

- [ ] 3.1 Two distinct non-literal expressions in the same file, same function → two violations,
      baselining one does not suppress the other.
- [ ] 3.2 Same expression text in two different free functions in the same file → two violations.
- [ ] 3.3 Same-named method in two different `impl Type` blocks, identical expression text → two
      violations distinguished by owner (the collision the original bare-name design missed).
- [ ] 3.4 Same-named method in two different `impl Trait for Type` blocks on the same `Self` type,
      identical expression text → two violations distinguished by trait.
- [ ] 3.5 Identical expression repeated verbatim in the same function/method → one violation
      (stated bound scenario).
- [ ] 3.6 Same expression text in two different files → two violations, distinguished by file.
- [ ] 3.7 Existing un-auditable-probe tests (raw-string decode failure, plain-string decode
      failure, non-literal-token cases) still pass with the extended identity shape.
- [ ] 3.8 `cargo test -p louke --all-features` (the `audit` feature gates this code path).

## 4. Spec and documentation sync

- [ ] 4.1 Sync `openspec/specs/runtime-origin-assertion/spec.md` with this change's ADDED
      requirement (via `openspec-sync-specs` / archive flow, not a hand copy).
- [ ] 4.2 Resolve and remove `BACKLOG.md`'s "Un-auditable-probe finding identity is file-granular"
      accepted-debt entry now that it is closed — explicitly note that its recorded remediation
      sketch ("byte offset / occurrence index") was superseded by the 0.3.0 no-positional-identity
      rule, not silently replaced.
- [ ] 4.3 Add a `CHANGELOG.md` `[Unreleased]` entry describing the false-negative closure (patch,
      per the v0.1.3 re-export-exposure precedent — not a breaking change).

## 5. Verification (Definition of Done, per AGENTS.md)

- [ ] 5.1 `cargo build --workspace`
- [ ] 5.2 `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] 5.3 `cargo clippy --workspace -- -D warnings`
- [ ] 5.4 `cargo clippy -p louke -- -D warnings` (audit-OFF, prod-light build)
- [ ] 5.5 `cargo fmt --all --check`
- [ ] 5.6 `TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features`
- [ ] 5.7 `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features`
- [ ] 5.8 `cargo deny check`
- [ ] 5.9 `bash scripts/test_release_coherence.sh` / `bash scripts/check_release_coherence.sh`
- [ ] 5.10 `bash scripts/test_examples.sh`
