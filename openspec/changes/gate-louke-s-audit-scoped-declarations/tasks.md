# Tasks

- [ ] 1 The five audit-scoped declarations behind `#[cfg(feature = "audit")]`; the hot-path one ungated.
- [ ] 2 `Owner` and `FactGranularity` gated with them; the `unused_mut` allow scoped by `cfg_attr` rather than
      blanket.
- [ ] 3 Module and accessor docs state the feature dependence instead of claiming every bound.
- [ ] 4 **Verify by execution**: `observation_bounds().len()` is 1 audit-OFF and 6 audit-ON.
- [ ] 5 Both clippy passes clean — the isolated `cargo clippy -p louke` especially, since it is the pass that
      exists for this class.
- [ ] 6 The `observation-bound-model` delta, and `CHANGELOG.md`. No version bump.
- [ ] 7 Sync the delta and prune the dated archive copy.
