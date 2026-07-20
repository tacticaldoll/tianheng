## Why

漏刻's CI audit recursively scans every `.rs` file below each target source directory. An
unreachable orphan file can therefore contain the only `assert_boundary!` for a seam and make the
audit pass even though no compiled runtime path enforces that boundary—the project's forbidden
false negative.

## What Changes

- Preserve Cargo's exact library/binary target root files through the shared 星表 substrate instead
  of reducing them to parent directories before the runtime audit.
- Teach the louke-local audit walker to scan only source files reachable from those roots through
  Rust module declarations, excluding undeclared orphan files and inline-module shadow files.
- Keep `audit_probe_coverage`'s public Rust signature source-compatible: file paths select the new
  root-aware walk, while directory inputs retain the legacy recursive corpus for direct callers.
- Make 天衡's composed check use exact root-file inputs, so its default reaction closes the false
  coverage path for conventional and custom Cargo target roots.
- Preserve 三儀 ⊥ 三儀: 漏刻 does not import 圭表's walker or `syn`; 天衡 reads root metadata directly
  from the shared 星表 substrate and the louke audit remains feature-gated.
- Close the orphan-only false coverage in root-aware and composed checks; document that legacy
  directory inputs retain their recursive compatibility behavior.
- Record `cfg` evaluation and path-remapped external modules as explicit lexical bounds rather than
  attempting a Cargo compilation matrix.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `runtime-origin-assertion`: probe coverage is determined from target-root-reachable source, so an
  orphan `.rs` file cannot satisfy a declared seam.
- `composed-library-check`: the shell supplies exact Cargo target roots to the runtime audit while
  preserving reaction precedence.

## Impact

Affected areas are `xingbiao` root selection, louke's audit-only scanner, Tianheng composition,
self-governance declaration/projection, tests/specs, and backlog. `tianheng` gains a direct workspace dependency
on `xingbiao`; no heavy dependency enters louke's default production face. Existing public function
signatures, finding identity, manifests' package versions, and baseline wire remain unchanged.
