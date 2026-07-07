//! Shared path and keyword primitives for the source scanner — the small foundation the
//! `use`-scan ([`super::use_scan`]) and module-graph walk ([`super::reachability`]) both stand
//! on, so neither sibling depends laterally on the other. Path canonicalization (raw-identifier
//! reduction and `::`-delimited containment) and the `mod`-keyword boundary test; pure string /
//! byte processing over [`super::lexer`]'s token primitives, no model type.

use super::lexer::keyword_starts_at;

/// Canonicalize one path segment by stripping a leading raw-identifier marker
/// (`r#name` -> `name`). Rust resolves `mod r#type;` to the source file `type.rs`,
/// so the file-derived path, the `mod` declaration, and a `use r#type::…` path must
/// all reduce to the same module identity; this is the single place that reduction
/// lives. A segment with no `r#` prefix is returned unchanged.
pub(super) fn canonical_segment(segment: &str) -> &str {
    segment.strip_prefix("r#").unwrap_or(segment)
}

/// Canonicalize a whole `::`-joined module path segment-by-segment (see
/// [`canonical_segment`]), so a boundary's declared path and an observed path compare
/// in one vocabulary regardless of which uses the raw-identifier form.
pub(crate) fn canonical_module_path(path: &str) -> String {
    path.split("::")
        .map(canonical_segment)
        .collect::<Vec<_>>()
        .join("::")
}

/// Sibling-safe `::`-delimited path containment: `path` is `prefix` itself or lies strictly
/// beneath it (`crate::a` contains `crate::a::b`, never the prefix-colliding sibling
/// `crate::ab`). The single home of the containment rule every module boundary's inbound /
/// outbound predicate and the file selector share, so no copy can drift to a bare
/// `starts_with` — which would admit a sibling (a false positive on the allowed side) or,
/// inverted, miss a subtree (a false negative on the forbidden side). The 圭表 twin of 渾儀's
/// `path_within`; the two dimensions cannot share code (三儀 ⊥ 三儀), so they agree by using the
/// same rule, not the same function.
pub(crate) fn path_within(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
}

/// Whether a standalone `mod` keyword begins at `i` (bounded by non-identifier bytes) —
/// the head of a possible module declaration, not a substring like `module`.
pub(super) fn is_mod_declaration_keyword(bytes: &[u8], i: usize) -> bool {
    keyword_starts_at(bytes, i, b"mod")
}
