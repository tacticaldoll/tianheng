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

/// Fold a Cargo package name to its Rust import identifier: `-` → `_` (`windows-sys` →
/// `windows_sys`). Cargo maps a hyphenated package name to an underscore identifier in source, and
/// a `use` path can never contain `-`, so every site matching a declared package name against an
/// observed import head needs this fold. The single home of it, so a dependency-name-matching site
/// and a confined-crate-name site cannot silently diverge on the rule.
pub(crate) fn package_name_to_import_ident(name: &str) -> String {
    name.replace('-', "_")
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

/// If an inline module declaration `mod <ident> {` begins at `i` (a standalone `mod` keyword whose
/// name is followed, after optional whitespace, by `{`), return `(name_start, name_end,
/// index_of_opening_brace)`; otherwise `None` — a `mod name;` with no body, or not a declaration.
/// Only an inline body encloses nested items. The single home of the inline-`mod` boundary test the
/// `use`-scan ([`super::use_scan`]) and symbol-scan ([`super::symbol_scan`]) walks share, so the two
/// cannot drift (the twin-drift bug class).
pub(super) fn inline_mod_at(bytes: &[u8], i: usize) -> Option<(usize, usize, usize)> {
    if !is_mod_declaration_keyword(bytes, i) {
        return None;
    }
    let mut j = i + 3;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let name_start = j;
    while j < bytes.len() && !bytes[j].is_ascii_whitespace() && bytes[j] != b';' && bytes[j] != b'{'
    {
        j += 1;
    }
    let name_end = j;
    if name_end == name_start {
        return None;
    }
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if bytes.get(j) == Some(&b'{') {
        Some((name_start, name_end, j))
    } else {
        None
    }
}

/// The module path enclosing a lexical position, formed from the file's `base` module and the names
/// of the inline `mod`s currently open around it (each `mod_stack` entry is `(name,
/// enclosing brace depth)`; only the names are joined). The shared home backing every walk that
/// attributes a `use` / item / call to its true inline submodule.
pub(super) fn effective_module(base: &str, mod_stack: &[(String, usize)]) -> String {
    let mut module = base.to_string();
    for (name, _) in mod_stack {
        module.push_str("::");
        module.push_str(name);
    }
    module
}

/// The one lexical walk shared by symbol readers that need the module enclosing a byte position.
/// `contexts[i]` is the inline-module path at byte `i`; `sites` records each inline `mod name {`
/// with the brace depth and enclosing module it had when encountered. Keeping the stack here makes
/// glob, path-occurrence, and definition readers agree on inline nesting instead of carrying three
/// copies of the same push/pop walk.
pub(super) struct InlineModuleScan {
    pub contexts: Vec<String>,
    pub module_tops: Vec<usize>,
    pub sites: Vec<InlineModuleSite>,
}

pub(super) struct InlineModuleSite {
    pub at: usize,
    pub name: String,
    pub brace: usize,
    pub module_top: usize,
    pub enclosing: String,
}

pub(super) fn scan_inline_modules(source: &str, base: &str) -> InlineModuleScan {
    let bytes = source.as_bytes();
    let mut contexts = vec![base.to_string(); bytes.len() + 1];
    let mut module_tops = vec![0usize; bytes.len() + 1];
    let mut sites = Vec::new();
    let mut i = 0;
    let mut depth = 0usize;
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    while i < bytes.len() {
        contexts[i] = effective_module(base, &mod_stack);
        module_tops[i] = mod_stack.last().map_or(0, |(_, d)| d + 1);
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            let enclosing = contexts[i].clone();
            let name = canonical_segment(&String::from_utf8_lossy(&bytes[name_start..name_end]))
                .to_string();
            sites.push(InlineModuleSite {
                at: i,
                name: name.clone(),
                brace,
                module_top: mod_stack.last().map_or(0, |(_, d)| d + 1),
                enclosing,
            });
            mod_stack.push((name, depth));
            i = brace;
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                while mod_stack.last().is_some_and(|(_, d)| *d == depth) {
                    mod_stack.pop();
                }
            }
            _ => {}
        }
        i += 1;
    }
    contexts[bytes.len()] = effective_module(base, &mod_stack);
    module_tops[bytes.len()] = mod_stack.last().map_or(0, |(_, d)| d + 1);
    InlineModuleScan {
        contexts,
        module_tops,
        sites,
    }
}

/// Whether a bare head names a crate-root module that **shadows** the extern prelude. Only at the
/// crate root itself (`current_module == "crate"`) is a sibling `mod` in scope, so a bare
/// `use foo::…` / path there resolves to the local `crate::foo`; in any submodule the same bare head
/// reaches only the extern prelude (an external crate). The single home of that shadow rule the
/// `use`-scan and symbol-scan share, so no copy can drift.
pub(super) fn is_crate_root_shadow(
    current_module: &str,
    head: &str,
    root_modules: &[String],
) -> bool {
    current_module == "crate" && root_modules.iter().any(|m| m == head)
}

/// Canonicalize and fold module path segments, resolving embedded `self` and `super`
/// segments anywhere in the path (e.g. `["crate", "a", "b", "super", "c"]` -> `["crate", "a", "c"]`).
/// Returns `None` if `super` over-pops past the `crate` root or if the path is not crate-rooted.
pub(super) fn fold_canonical_segments(segments: &[&str]) -> Option<String> {
    let mut stack: Vec<&str> = Vec::new();
    for &raw_seg in segments {
        let seg = canonical_segment(raw_seg.trim());
        if seg.is_empty() {
            continue;
        }
        match seg {
            "self" => continue,
            "super" => {
                if stack.last() == Some(&"crate") || stack.is_empty() {
                    return None;
                }
                stack.pop();
            }
            other => stack.push(other),
        }
    }
    if stack.first() == Some(&"crate") {
        Some(stack.join("::"))
    } else {
        None
    }
}

/// Resolve a `self::…` / `super::…` relative path against `current_module` into a crate-rooted
/// absolute path. `parts` is the already-canonicalized, `::`-split path whose first segment is
/// `self` or `super`. Returns `None` when a `super` chain **over-pops** past the crate root (more
/// `super`s than ancestors): the result would not be crate-rooted, names no internal module (and
/// the source does not compile), so it must never be mistaken for an outward edge. Any other head
/// (`parts[0]` not `self`/`super`, or empty) also returns `None` — the caller resolves those.
///
/// The single home of the `super`-pop loop and its over-pop guard, which the `use`-scan
/// ([`super::use_scan`]) and symbol-scan ([`super::symbol_scan`]) resolvers share — so a fix to that
/// subtle edge cannot silently diverge across them (the twin-drift bug class). guibiao-internal;
/// crosses no dimension boundary.
pub(super) fn resolve_self_super(current_module: &str, parts: &[&str]) -> Option<String> {
    let first = parts.first().copied()?;
    if first != "self" && first != "super" {
        return None;
    }
    let mut full: Vec<&str> = current_module
        .split("::")
        .filter(|s| !s.is_empty())
        .collect();
    full.extend(parts);
    fold_canonical_segments(&full)
}

/// Content inside the first `{ … }` of `s` (which must start with `{`), honoring nesting. The single
/// home of the brace-body extractor the `use`-scan ([`super::use_scan`]) and symbol-scan
/// ([`super::symbol_scan`]) use-tree parsers share, so the two cannot drift (the twin-drift bug
/// class).
pub(super) fn brace_content(s: &str) -> String {
    let mut depth = 0i32;
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '{' => {
                depth += 1;
                if depth == 1 {
                    continue;
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        out.push(ch);
    }
    out
}

/// Split on commas at brace depth 0 — the use-tree group splitter both scanners share (see
/// [`brace_content`]).
pub(super) fn split_top_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0i32;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_canonical_segments_resolves_embedded_super_and_self() {
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "b", "super", "c", "D"]),
            Some("crate::a::c::D".to_string())
        );
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "self", "b", "C"]),
            Some("crate::a::b::C".to_string())
        );
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "b", "super", "super", "secret"]),
            Some("crate::secret".to_string())
        );
    }

    #[test]
    fn fold_canonical_segments_over_pop_returns_none() {
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "super", "super"]),
            None
        );
        assert_eq!(fold_canonical_segments(&["super", "a"]), None);
    }
}
