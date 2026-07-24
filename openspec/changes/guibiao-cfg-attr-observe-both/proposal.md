## Why

Currently, `guibiao` module reachability scanner recognizes `#[cfg_attr(..., path = "...")]` as a conditional module remap and marks it as `Excluded` (out of scope for automatic reachability following). While this prevents false assumptions about whether a conditional path is active on the current platform, it creates a governance false-negative gap: code residing in platform-conditional module paths (e.g., `os/windows.rs`, `os/unix.rs`) escapes static architecture governance unless explicitly compiled on that platform.

By introducing Observe-Both semantics (union-scan over default and all physically existing `cfg_attr` remapped files), `guibiao` closes this false-negative gap in a single CI run across all platforms without requiring platform-specific compilation or AST parsing.

## What Changes

- Update `guibiao::module_scan::reachability` internal `PathAttrKind` to support collecting candidate `cfg_attr(..., path = "...")` targets alongside default module paths.
- Perform a union reachability walk across all physically existing candidate files (`path.exists()`), filtering missing files gracefully without triggering scan errors.
- Preserve precedence for unconditional direct `#[path = "..."]` attributes over conditional `cfg_attr` paths according to rustc semantics.
- Ensure inline module bodies (`is_inline = true`) retain precedence over file-system resolution.

## Capabilities

### New Capabilities
- None

### Modified Capabilities
- `module-boundary`: Enhance module reachability to perform union scanning across default and physically existing `cfg_attr(path = "...")` remapped paths, closing false-negative observation gaps under conditional compilation.

## Impact

- **Affected code**: `crates/guibiao/src/module_scan/reachability.rs`
- **Dependencies**: No external dependencies added (remains `serde_json`-only).
- **APIs**: Zero public API changes (100% backward compatible internal enhancement).
