## Context

`guibiao` is Tianheng's static observation core. When parsing grouped `use` trees like `use crate::a::b::{super::secret::X}` or inline symbol calls like `crate::a::b::super::secret::helper()`, `use_scan` and `symbol_scan` previously concatenated prefixes and leaves into un-collapsed paths containing embedded `super` or `self` segments.

Previously, `normalize_module_path`, `resolve_head`, and `resolve_written_path` checked if the first segment was `crate`, and if so, immediately joined all segments with `::` without resolving embedded `super` or `self` segments. This left `"super"` in the string identity, breaking `path_within` checks and creating false-negative gaps across both module boundaries and inline symbol path confinements.

## Goals / Non-Goals

**Goals:**
- Provide single-source segment folding in `guibiao::path_vocab::fold_canonical_segments` that resolves embedded `self` (no-op) and `super` (pop parent segment) segments.
- Enforce over-pop protection: if `super` attempts to pop past the `crate` root or an empty stack, return `None` (invalid path).
- Unify path resolution across `use_scan`, `reachability`, and `symbol_scan` (`resolve_head` and `resolve_written_path`) so all scanner paths resolve to normalized, fully collapsed `crate::...` paths.

**Non-Goals:**
- Adding `syn` or AST parser dependencies to `guibiao` (maintains `serde_json`-only invariant).
- Changing existing leading `self::` or `super::` resolution rules for top-level paths.

## Decisions

### Decision 1: Stack-Based Segment Folding Pattern
- **Rationale**: A stack-based walk over path segments (`["crate", "a", "b", "super", "c"]`) cleanly handles arbitrary nesting and combinations of `self` and `super` in one pass.
- **Implementation**: In `path_vocab.rs`, implement `fold_canonical_segments(segments: &[&str]) -> Option<String>`.

### Decision 2: Over-Pop Past Crate Protection
- **Rationale**: Popping past `crate` root (e.g. `crate::super`) produces an un-rooted path that cannot compile in Rust and must not be treated as a valid internal module edge or symbol path.
- **Implementation**: Return `None` if popping occurs when the stack top is `"crate"` or empty.

### Decision 3: Shared Resolver Wiring across Scanner Dimensions
- **Rationale**: Route `use_scan`, `reachability`, and `symbol_scan` (`resolve_head` and `resolve_written_path`) path normalizations through `fold_canonical_segments` to avoid twin-drift bugs.
