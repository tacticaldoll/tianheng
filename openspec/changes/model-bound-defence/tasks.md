## 1. Typed defence model

- [x] 1.1 Add `Defence`, explicit `BoundDecl` constructors, and evidence accessors in `xuanji`
- [x] 1.2 Prove pinned, multiply-pinned, unpinned, borrowed, and computed-string states with unit tests

## 2. Family migration

- [x] 2.1 Migrate every family and example declaration from `BoundDecl::new` to an explicit defence constructor
- [x] 2.2 Update the cookbook and `[Unreleased]` migration narrative for the breaking API

## 3. Composed reaction

- [x] 3.1 Parse and compare every ordered `PINNED-BY` citation and the mutually exclusive `UNPINNED` tracker
- [x] 3.2 Regenerate the extent projection and prove the existing multiply-pinned scenario retains both citations

## 4. Verification

- [x] 4.1 Record negative evidence that the old model cannot express unpinned state and loses a second live pin
- [x] 4.2 Run focused model tests, projection freshness checks, formatting, and repository hygiene gates
- [x] 4.3 Run the complete repository Definition of Done

### Verification evidence

- Removing `BoundDecl::unpinned` made `cargo test -p xuanji an_unpinned_declaration_carries_a_tracker_and_no_test`
  exit 101 with `no associated function or constant named unpinned`.
- Replacing the live multiply-pinned declaration with a single pin made the exact composed-model comparison exit
  101 and name both the two spec citations and the one typed citation.
- The xuanji suite, observation-bound model with and without `BLESS`, formatting, whitespace, reference integrity,
  and bound-register checks passed; the register reported 57 pinning citations for 56 declarations.
- Every command in the current `AGENTS.md` Definition of Done passed, including all build, Clippy, test, rustdoc,
  cargo-deny, gate-matrix, release-coherence, bound-register, and example-family commands.
