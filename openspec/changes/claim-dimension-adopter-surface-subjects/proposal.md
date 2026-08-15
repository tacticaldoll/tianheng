## Why

`observation_bounds()` reached the public surface of `guibiao`, `hunyi`, and `louke` in the `0.5.0` window,
each pinned by that dimension's own `tests/adopter_surface.rs`, advertised as the way an adopter reads a
dimension's declared bounds without composing a run. No capability's declared `## Subject` claims those three
files: `adopter-surface`'s own subject names only the shell's three files
(`crates/tianheng/src/lib.rs`, `crates/tianheng/src/sans_io.rs`, `crates/tianheng/tests/adopter_surface.rs`).
`crates/kanhe/tests/capability_subjects.rs` reports every unclaimed file on a clean run rather than failing
over it — a declared bound (`repository-checks/files-no-capability-claims-a-stated-bound`), whose reason
(requiring subjects to tile the repository would buy coverage with a claim per capability nobody could
defend) still holds. This is not that: it is one existing capability's subject failing to name three files
that are plainly its own — a standalone-dimension's adopter surface belongs to `adopter-surface` by the same
argument that put the shell's own file there, and the promotion trigger (a published-surface member landing
in an unclaimed file) already fired once for `observation_bounds()` itself.

## What Changes

- Extend `adopter-surface`'s declared `## Subject` to also name `crates/guibiao/tests/adopter_surface.rs`,
  `crates/hunyi/tests/adopter_surface.rs`, and `crates/louke/tests/adopter_surface.rs` — the three
  per-dimension standalone-adoption compile checks, the same role `crates/tianheng/tests/adopter_surface.rs`
  already holds for the composed shell.
- No requirement text changes: the capability's `Requirements` continue to describe the composed prelude and
  the standalone per-dimension surfaces exactly as they do today. This is a subject-only correction — the
  reaction that files a changed file under the capability whose subject claims it.

### Capabilities

#### Modified Capabilities

- `adopter-surface`: its declared `## Subject` gains the three dimension standalone-surface files described
  above. No `Requirements` change.

## Impact

- `openspec/specs/adopter-surface/spec.md` — `## Subject` list only.
- `crates/kanhe/tests/capability_subjects.rs`'s unclaimed-file count (printed, not asserted) drops by three on
  the next clean run.
- No code, public API, or CI behavior changes. `publish = false` on every crate whose subject declaration this
  touches has no bearing here, since a capability's `## Subject` is repository-internal bookkeeping, not
  shipped surface.
