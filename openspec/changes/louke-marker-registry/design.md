# design: Louke Marker Registry Design

## Architectural Context

`louke` provides runtime origin assertions with two faces:
1. **Prod Face**: Lock-free, std-only `assert_boundary!` probe hot path (depends on `xuanji` only).
2. **CI Face**: `audit_probe_coverage` scanner compiled behind the `audit` feature flag.

The CI face reads workspace Rust sources using a zero-allocation byte-window scanner (`crates/louke/src/audit/scan.rs`), skipping comments, string literals, and macro definition bodies (`macro_rules!`) to avoid false-positive coverage assertions.

## Design Decisions

### 1. Marker Matching Abstraction (`MarkerSet`)
Instead of searching solely for `const NAME: &[u8] = b"assert_boundary"`, `scan.rs` will accept a slice of marker names `&[&str]` (or `HashSet<&str>`).
For each position in source code, `match_probe_marker` will attempt to match any registered marker string that satisfies:
- Valid left word boundary (`is_word_boundary`)
- Identifier equality with a registered marker name
- Valid right word boundary
- Followed by optional whitespace and `!` token (with `()` / `{}` / `[]` macro delimiters)

### 2. High-Performance Lexer Integrity
The scanner retains its `syn`-free, zero-allocation byte window design. Custom marker matching iterates through the configured marker slice without regex or AST parsing, ensuring `audit_probe_coverage` remains fast even across large codebases.

### 3. API Surface Layering
- `audit_probe_coverage(boundaries, roots)` preserves its signature, defaulting markers to `&["assert_boundary"]`.
- `audit_probe_coverage_with_markers(boundaries, roots, markers)` allows callers to pass explicit marker lists `&[&str]`.

## Adversarial Review Checklist

- **Minimalism**: Does this add heavy dependencies to `louke`? No, stays `syn`-free and byte-window based.
- **Independence**: Does this pull `guibiao` or `hunyi` into `louke`? No (`三儀 ⊥ 三儀` maintained).
- **False Negative Prevention**: Are custom marker probes checked with identical word boundary and macro body exclusion rules? Yes.
