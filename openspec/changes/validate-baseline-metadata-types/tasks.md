## 1. Parser Reaction

- [ ] 1.1 Add focused parser tests for omitted, null, string, and every wrong JSON metadata type in both baseline versions
- [ ] 1.2 Make `Baseline::from_json` distinguish absent/null metadata from wrong-typed values with a field-specific error
- [ ] 1.3 Verify canonical serialization still omits absent and explicit-null metadata

## 2. Shell Reaction

- [ ] 2.1 Add a subprocess CLI test proving `--baseline` with wrong-typed metadata reports the field and exits 2 without suppressing findings
- [ ] 2.2 Add a subprocess CLI test proving `--write-baseline` emits the existing metadata-loss warning before producing a fresh snapshot

## 3. Governance And Verification

- [ ] 3.1 Mark the baseline metadata strictness backlog candidate built while preserving its compatibility boundary
- [ ] 3.2 Perform apply-stage adversarial review against fail-loud minimalism and the explicit snapshot contract
- [ ] 3.3 Run strict OpenSpec validation and the complete Definition of Done
