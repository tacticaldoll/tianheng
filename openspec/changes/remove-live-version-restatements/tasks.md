## 1. Prose migration

- [x] 1.1 Inventory mutable version restatements separately from historical and manifest literals
- [x] 1.2 Remove numeric restatements from CI, release-coherence comments, and example documentation
- [x] 1.3 Correct observer-participant's published-surface claim

## 2. Verification

- [x] 2.1 Verify touched files retain no mutable numeric restatement and preserve required historical literals
- [x] 2.2 Run OpenSpec validation and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- Before migration, CI still named an old dependency form, release-coherence comments named an old/new version
  pair, and example prose repeated manifest requirements while observer-participant described APIs available only
  through the local patch as already published.
- After migration, the remaining version literals in the audited surfaces are actual manifest requirements,
  immutable history/provenance, or fixture values. OpenSpec validation, release-coherence directions, repository
  hygiene, and the complete Definition of Done pass.
