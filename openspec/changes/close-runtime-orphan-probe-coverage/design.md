## Context

Cargo metadata already supplies each library or binary target's exact `src_path`, but
`member_src_dirs` discards the filename. Once only a directory remains, a custom `[lib] path` cannot
be distinguished from orphan sibling files. Guessing `lib.rs`/`main.rs` would regress custom-layout
coverage; importing 圭表's private reachability code would breach 三儀 independence.

## Goals / Non-Goals

**Goals:**

- Exclude undeclared orphan files from the composed runtime audit corpus.
- Retain custom Cargo target roots and deterministic, fail-loud source reads.
- Keep the runtime production face dependency-light and existing callers source-compatible.

**Non-Goals:**

- Evaluating `cfg`, features, generated modules, or the full Cargo compilation matrix.
- Moving a walker into 星表 before a second dimension proves shared semantics.
- Reusing or depending on 圭表's module scanner, or adding `syn` to 漏刻.
- Re-keying any runtime finding.

## Decisions

### Preserve root files in 星表; interpret file inputs root-aware in 漏刻

Add a vocabulary-neutral `member_root_files` derivation beside `member_src_dirs`. 天衡 depends on
星表 directly and passes those files to the unchanged `audit_probe_coverage` signature. Louke treats
a file input as a crate root and a directory input as its compatibility-mode recursive corpus.
This avoids a second public audit verb and keeps direct callers compiling, while the composed path
becomes correct by construction.

### Walk reachability louke-locally

The audit-only walker reads a root, scans probes in every reachable file, and follows conventional
file-backed `mod name;` declarations. Inline module bodies are already part of their owning file and
their nested module declarations must resolve relative to the inline module's logical directory.
An inline-only `mod name { ... }` never causes a same-named sibling file to be scanned. Missing or
unreadable files selected by a reachable declaration are constitution errors rather than skips.

### Amend the shell self-law rather than route through 圭表

星表 is the shared declared-workspace-data substrate beneath the dimensions. A direct
`tianheng -> xingbiao` edge is downward and acyclic; routing root data through guibiao would make the
runtime composition depend semantically on a sibling instrument. The self-governance allowlist and
generated projection change together under this reviewed OpenSpec lifecycle.

## Risks / Trade-offs

- Directory-input callers retain legacy recursive behavior and therefore do not gain orphan
  exclusion until they pass root files; docs make the mode distinction explicit without breaking
  them.
- `cfg` remains lexical: mutually exclusive module declarations may both be followed. This can
  over-count a cfg-disabled probe, an already stated bound; it does not re-admit undeclared orphans.
- `#[path]` external-module remaps remain outside the first walker. Ignoring them can report a seam
  unprobed (false positive), never silently covered. The bound is explicit and can deepen later on
  the same source.

## Migration Plan

No adopter migration is required. Existing directory callers behave as before; callers wanting
orphan-safe coverage can pass exact target root files. Tianheng's composed runner does so by default.
