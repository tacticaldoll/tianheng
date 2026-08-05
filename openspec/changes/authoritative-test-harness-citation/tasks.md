## 1. Measure before deciding

- [x] 1.1 Reproduce all three review findings on fixtures and record the exit codes.
- [x] 1.2 Measure `cargo test --list` over this workspace and confirm all 36 cited names appear.
- [x] 1.3 Measure per-package enumeration, and confirm the same-named-test-in-two-crates case is real.
- [x] 1.4 Measure a throwaway fixture crate's cold enumeration, and confirm a cfg-disabled and a macro-body
      test are absent from it.

## 2. The harness becomes the authority

- [ ] 2.1 Build a per-package index of registered tests, once, lazily.
- [ ] 2.2 Fail a citation whose name is absent from the cited crate's set (or the workspace's, unqualified).
- [ ] 2.3 Exit cannot-judge when a manifest exists but the enumeration cannot be produced.
- [ ] 2.4 Keep the definition scan for the site and the duplicate direction; stop consulting the attribute
      walk when the harness is available.
- [ ] 2.5 Report the line-shape limitation when the harness registers a test the scan cannot locate.

## 3. The fallback, declared and reported

- [ ] 3.1 Fall back to the attribute walk when the repository has no root manifest.
- [ ] 3.2 Print which direction decided test-ness, so a clean result names its own strength.

## 4. Citation grammar

- [ ] 4.1 Accept a raw identifier (`r#name`).
- [ ] 4.2 Leave non-ASCII refused, with the requirement stating the narrowing.

## 5. Retire the residual out loud

- [ ] 5.1 Remove the projection's third floor.
- [ ] 5.2 Invert the `commented-definition` fixture from a passing residual record to a refusal.
- [ ] 5.3 Move the `BACKLOG.md` entry to the closed records with its reproduction and the measurement that
      dissolved it.

## 6. Fixtures and ordering

- [ ] 6.1 Add manifest-bearing fixtures: a real test passes; a cfg-disabled test fails; a macro-body test
      fails; a raw-string definition fails.
- [ ] 6.2 Add the fixture asserting the degradation notice on a manifest-less repository.
- [ ] 6.3 Add the raw-identifier passing fixture and the line-shape diagnostic fixture.
- [ ] 6.4 Move CI's register step after its build step, and update the `AGENTS.md` comment on those lines.

## 7. Verification and lifecycle

- [ ] 7.1 Run every new guard against the code without the change and record what it accepted or refused.
- [ ] 7.2 Run the full Definition of Done.
- [ ] 7.3 Sync, validate, update `CHANGELOG.md`.
- [ ] 7.4 Archive, prune the dated copy, open one squash PR into `release/0.4.1`.
