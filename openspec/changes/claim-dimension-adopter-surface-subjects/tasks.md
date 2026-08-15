## 1. Apply

- [ ] 1.1 Extend `openspec/specs/adopter-surface/spec.md`'s `## Subject` to name
      `crates/guibiao/tests/adopter_surface.rs`, `crates/hunyi/tests/adopter_surface.rs`, and
      `crates/louke/tests/adopter_surface.rs`, alongside the existing three shell files.
- [ ] 1.2 Confirm `crates/kanhe/tests/capability_subjects.rs`'s unclaimed-file count drops by exactly
      three on a clean run, and that no other capability's subject already claims any of the three
      files (which would make this an overlap rather than a genuine gap close).

## 2. Verify

- [ ] 2.1 Run the full Definition of Done gate list.
- [ ] 2.2 Confirm `openspec validate --strict` passes over every spec, including the modified
      `adopter-surface` spec.
