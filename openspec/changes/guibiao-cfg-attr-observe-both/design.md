## Context

In Rust cross-platform projects, module declarations are frequently written with conditional path attributes such as:
```rust
#[cfg_attr(unix, path = "os/unix.rs")]
#[cfg_attr(windows, path = "os/windows.rs")]
pub mod os;
```
`guibiao` is a static, compiler-free (and syn-free) architectural governance core. Previously, `guibiao` treated `cfg_attr(path)` as a conditional remap and marked the module as `Excluded`, which prevented guessing which platform path was active but left a static governance false-negative window on uncompiled platform files.

To close this false-negative gap, `guibiao` adopts Observe-Both (union-scan) semantics: scanning all candidate target files that physically exist on disk, combining their observed imports under the module's logical path.

## Goals / Non-Goals

**Goals:**
- Extend `guibiao::module_scan::reachability` internal attribute parsing to collect all candidate `cfg_attr(..., path = "...")` targets.
- Perform a union-scan across default and physically existing candidate files (`path.exists()`), treating missing files gracefully without triggering scan errors.
- Preserve unconditional `#[path = "..."]` precedence over conditional `cfg_attr` paths.
- Maintain zero external dependencies (`serde_json`-only) and 100% public API compatibility.

**Non-Goals:**
- Full `cfg(...)` predicate evaluation (remains syn-free and compiler-free).
- Modifying `hunyi` or `louke` (this is a `guibiao` module reachability enhancement).

## Decisions

### Decision 1: Extend internal `PathAttrKind` data structure
**Choice**: Update `PathAttrKind` in `crates/guibiao/src/module_scan/reachability.rs` to carry candidate conditional targets:
```rust
enum PathAttrKind {
    None,
    Direct(usize),
    ConditionalRemaps(Vec<PathRemapSpec>),
    Excluded,
}
```
**Rationale**: Keeps all changes internal to `guibiao::module_scan::reachability` without exposing new public traits or types.

### Decision 2: Multi-target collection over early return
**Choice**: `attr_prefix_path_kind` will scan the entire attribute prefix preceding a `mod` item, accumulating all `cfg_attr(..., path = "...")` occurrences into a list rather than returning after the first finding.
**Rationale**: Prevents missing subsequent conditional branches on the same module declaration.

### Decision 3: Graceful missing-file handling for conditional targets
**Choice**: During module reachability traversal, for each `PathRemapSpec` candidate target, check if `resolved_path.exists()`. If the file exists, include it in the reachability walk; if it does not exist, skip it without raising an `Exit 2` scan error.
**Rationale**: Conditional targets may legally be missing depending on source checkout or target-platform scope. In contrast, unconditional `#[path = "..."]` targets that are missing remain hard `Exit 2` scan errors because rustc itself rejects missing unconditional path references.

## Risks / Trade-offs

- **[Risk]** Two platform implementations (e.g. `unix.rs` and `win.rs`) might import different modules that violate an allowlist boundary.
  → **Mitigation**: This is the intended behavior of union scanning (over-approximation for architecture governance). If any target file contains a forbidden import, the governance reaction triggers.
- **[Risk]** Attribute scanning performance overhead on large attribute prefixes.
  → **Mitigation**: The scanner only inspects attribute bytes preceding `mod` declarations at brace depth 0 / inline module depth, matching the existing fast byte scanner.

## Open Questions

- None (all design decisions and depth policies resolved during explore phase).
