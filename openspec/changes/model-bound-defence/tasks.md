## 1. Typed defence model

- [ ] 1.1 Add `Defence`, explicit `BoundDecl` constructors, and evidence accessors in `xuanji`
- [ ] 1.2 Prove pinned, multiply-pinned, unpinned, borrowed, and computed-string states with unit tests

## 2. Family migration

- [ ] 2.1 Migrate every family and example declaration from `BoundDecl::new` to an explicit defence constructor
- [ ] 2.2 Update the cookbook and `[Unreleased]` migration narrative for the breaking API

## 3. Composed reaction

- [ ] 3.1 Parse and compare every ordered `PINNED-BY` citation and the mutually exclusive `UNPINNED` tracker
- [ ] 3.2 Regenerate the extent projection and prove the existing multiply-pinned scenario retains both citations

## 4. Verification

- [ ] 4.1 Record negative evidence that the old model cannot express unpinned state and loses a second live pin
- [ ] 4.2 Run focused model tests, projection freshness checks, formatting, and repository hygiene gates
- [ ] 4.3 Run the complete repository Definition of Done
