## 1. Apply

- [x] 1.1 Extend `openspec/specs/adopter-surface/spec.md`'s `## Subject` to name
      `crates/guibiao/tests/adopter_surface.rs`, `crates/hunyi/tests/adopter_surface.rs`, and
      `crates/louke/tests/adopter_surface.rs`, alongside the existing three shell files.
- [x] 1.2 Confirmed: unclaimed count dropped from 109 to 106 (claimed 286 to 289), exactly three, and
      an independent adversarial review confirmed no other capability's spec already names any of the
      three files under its own `## Subject`.

## 2. Verify

- [x] 2.1 Full Definition of Done gate list run: green.
- [x] 2.2 `adopter-surface` validates individually. `openspec validate --specs --strict` shows one
      pre-existing failure (`repository-checks`, unrelated to this change — confirmed identical with
      this change stashed out) and is not this change's to fix.
