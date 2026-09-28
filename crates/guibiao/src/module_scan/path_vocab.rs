//! Shared path and keyword primitives for the source scanner — the small foundation the
//! `use`-scan ([`super::use_scan`]) and module-graph walk ([`super::reachability`]) both stand
//! on, so neither sibling depends laterally on the other. Path canonicalization (raw-identifier
//! reduction and `::`-delimited containment) and the `mod`-keyword boundary test; pure string /
//! byte processing over [`super::lexer`]'s token primitives, no model type.

use super::lexer::{is_ident_byte, keyword_starts_at};

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

/// Whether `segment` is exactly one identifier, written with nothing around it.
///
/// Read with the lexer's own [`is_ident_byte`], the byte test every scanner here names an
/// identifier with: a non-empty run of identifier bytes that does not start with a digit, behind at
/// most one `r#`. Behind `r#` the five names a raw identifier cannot spell — `crate`, `self`,
/// `super`, `Self` and `_` — are refused, as rustc refuses them. Every non-ASCII byte is an
/// identifier byte to the lexer, whatever character it belongs to, so a non-ASCII segment that is
/// not an identifier passes this layer; it names no module, and the existence check that follows
/// every accepted path is what refuses it. Keywords pass for the same reason.
fn is_identifier(segment: &str) -> bool {
    let (raw, name) = match segment.strip_prefix("r#") {
        Some(name) => (true, name),
        None => (false, segment),
    };
    let bytes = name.as_bytes();
    let lexes = bytes.first().is_some_and(|first| !first.is_ascii_digit())
        && bytes.iter().all(|&byte| is_ident_byte(byte));
    lexes && !(raw && matches!(name, "crate" | "self" | "super" | "Self" | "_"))
}

/// `crate`, or `crate::` followed by `::`-separated identifiers.
fn is_canonical_spelling(written: &str) -> bool {
    let mut segments = written.split("::");
    segments.next() == Some("crate") && segments.all(is_identifier)
}

/// The canonical spelling of a written module path, or the spelling it most plausibly meant.
///
/// A module path has exactly one accepted spelling — `crate`, or `crate::` followed by
/// `::`-separated identifiers — so that one module is one identity: a rule's module paths enter its
/// [`RuleKey`](xuanji::RuleKey), and a baseline entry recorded under one spelling would not suppress
/// the same finding declared under another. The one equivalence folded is rustc's own, `r#x` and `x`
/// being the same identifier, so the accepted form carries no raw prefix. Every other spelling is
/// refused rather than rewritten, and the refusal carries the suggestion: the written path's
/// non-empty, trimmed segments rooted at `crate`, or `None` when that is no canonical spelling
/// either, or when the path starts at `self` or `super` — relative to a module a declaration does
/// not have.
pub(crate) fn canonical_module_spelling(written: &str) -> Result<String, Option<String>> {
    if is_canonical_spelling(written) {
        return Ok(canonical_module_path(written));
    }
    let segments: Vec<&str> = written
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    let candidate = match segments.first().map(|head| canonical_segment(head)) {
        None => return Err(Some("crate".to_string())),
        Some("self" | "super") => return Err(None),
        Some("crate") => std::iter::once("crate")
            .chain(segments[1..].iter().copied())
            .collect::<Vec<_>>()
            .join("::"),
        Some(_) => format!("crate::{}", segments.join("::")),
    };
    Err(is_canonical_spelling(&candidate).then_some(candidate))
}

/// Whether a symbol path's first segment, as written, can never name a crate or module.
///
/// The set is the Rust Reference's identifier grammar rather than a keyword list: `_` is not an
/// identifier, so neither `_` nor `r#_` names anything; `crate`, `self`, `super` and `Self` cannot be
/// written raw; bare `self`, `super` and `Self` are relative to a module or type a declaration does not
/// have; and `crate` stands only at the start of a path, so after a leading `::` it names nothing. Bare
/// `crate` is the crate-root form. Every other head — a keyword in some edition or not — names the crate
/// or module of that name, written bare or raw.
fn is_disallowed_symbol_head(head: &str, is_global: bool) -> bool {
    match head.strip_prefix("r#") {
        Some(name) => matches!(name, "_" | "crate" | "self" | "super" | "Self"),
        None => matches!(head, "_" | "self" | "super" | "Self") || (is_global && head == "crate"),
    }
}

/// Whether `written` is `::`-separated identifiers (optionally starting with `::` for external
/// crates) whose first segment can name a crate or module ([`is_disallowed_symbol_head`]).
fn is_symbol_path_spelling(written: &str) -> bool {
    let is_global = written.starts_with("::");
    let raw = if is_global {
        let after = &written[2..];
        if after.is_empty() || after.starts_with(':') {
            return false;
        }
        after
    } else {
        written
    };
    if raw.ends_with(':') {
        return false;
    }
    let mut segments = raw.split("::");
    segments
        .next()
        .is_some_and(|head| is_identifier(head) && !is_disallowed_symbol_head(head, is_global))
        && segments.all(is_identifier)
}

/// Whether an inline-call prefix was written from the extern-crate root (`::std::time`) or bare
/// (`std::time`). Both name one crate, so the root form is not part of a prefix's identity; it is kept
/// for the readers that judge what the written first segment may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrefixRoot {
    Bare,
    Global,
}

/// An inline-call prefix in its one canonical form: `::`-separated segments with no leading `::` and
/// no `r#`, and the root form it was written in. `std::time` and `::std::time` are one `path`, which is
/// what a prefix's rule key, its violations' target and the call matcher all read, so the three cannot
/// disagree about which prefix a finding belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolPrefix {
    pub path: String,
    pub root: PrefixRoot,
}

impl SymbolPrefix {
    /// The canonical form of any written prefix. Total, so the model can key a rule on it before the
    /// spelling is judged; [`canonical_symbol_path_spelling`] is what refuses a spelling.
    pub(crate) fn of(written: &str) -> Self {
        match written.strip_prefix("::") {
            Some(rest) => SymbolPrefix {
                path: canonical_module_path(rest),
                root: PrefixRoot::Global,
            },
            None => SymbolPrefix {
                path: canonical_module_path(written),
                root: PrefixRoot::Bare,
            },
        }
    }
}

/// The canonical spelling of a written symbol path, or the spelling it most plausibly meant.
///
/// A symbol path — an inline-call prefix — is compared with resolved paths segment by segment, so it
/// has one accepted spelling: `::`-separated identifiers, optionally starting with `::` to explicitly
/// name an external crate, each read by the [`is_identifier`] a module path is read by, with a first
/// segment outside the finite set that can never name a crate or module
/// ([`is_disallowed_symbol_head`]). Which of the remaining first segments name something is the
/// caller's question, since it needs what the crate declares. `r#x` and `x` are one identifier, so the
/// accepted form is a [`SymbolPrefix`], raw prefixes and the leading `::` removed. Every other spelling
/// is refused, and the refusal carries the written path's non-empty, trimmed segments, raw prefixes
/// removed, as the suggestion — or `None` when that is no accepted spelling either.
pub(crate) fn canonical_symbol_path_spelling(
    written: &str,
) -> Result<SymbolPrefix, Option<String>> {
    if is_symbol_path_spelling(written) {
        return Ok(SymbolPrefix::of(written));
    }
    let is_global = written.trim().starts_with("::");
    let segments: Vec<&str> = written
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err(None);
    }
    let first = segments[0];
    let head = canonical_segment(first);
    if is_disallowed_symbol_head(head, is_global) {
        return Err(None);
    }
    let candidate_path = segments
        .iter()
        .map(|segment| canonical_segment(segment))
        .collect::<Vec<_>>()
        .join("::");
    let candidate = if is_global {
        format!("::{candidate_path}")
    } else {
        candidate_path
    };
    Err(is_symbol_path_spelling(&candidate).then_some(candidate))
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

/// The one lexical walk shared by symbol readers that need the module enclosing a byte position.
/// `contexts[i]` is an index into `modules` for the inline-module path at byte `i`; `sites` records
/// each inline `mod name {` with the brace depth and enclosing module it had when encountered.
/// Keeping the stack here makes glob, path-occurrence, and definition readers agree on inline
/// nesting instead of carrying three copies of the same push/pop walk. An index per byte avoids
/// cloning a module `String` for every source byte.
pub(super) struct InlineModuleScan {
    pub contexts: Vec<u32>,
    pub modules: Vec<String>,
    pub module_tops: Vec<usize>,
    pub sites: Vec<InlineModuleSite>,
}

pub(super) struct InlineModuleSite {
    pub at: usize,
    pub name: String,
    pub brace: usize,
    pub module_top: usize,
    pub enclosing: u32,
}

pub(super) fn scan_inline_modules(source: &str, base: &str) -> InlineModuleScan {
    let bytes = source.as_bytes();
    let mut contexts = vec![0u32; bytes.len() + 1];
    let mut module_tops = vec![0usize; bytes.len() + 1];
    let mut modules = vec![base.to_string()];
    let mut module_indices = std::collections::HashMap::from([(base.to_string(), 0u32)]);
    let mut sites = Vec::new();
    let mut i = 0;
    let mut depth = 0usize;
    let mut mod_stack: Vec<(u32, usize)> = Vec::new();
    let mut current = 0u32;
    while i < bytes.len() {
        contexts[i] = current;
        module_tops[i] = mod_stack.last().map_or(0, |(_, d)| d + 1);
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            let name = canonical_segment(&String::from_utf8_lossy(&bytes[name_start..name_end]))
                .to_string();
            let path = format!("{}::{name}", modules[current as usize]);
            let next = if let Some(&index) = module_indices.get(&path) {
                index
            } else {
                let index = u32::try_from(modules.len()).expect("inline module table exceeds u32");
                modules.push(path.clone());
                module_indices.insert(path, index);
                index
            };
            sites.push(InlineModuleSite {
                at: i,
                name: name.clone(),
                brace,
                module_top: mod_stack.last().map_or(0, |(_, d)| d + 1),
                enclosing: current,
            });
            mod_stack.push((next, depth));
            current = next;
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
                current = mod_stack.last().map_or(0, |(index, _)| *index);
            }
            _ => {}
        }
        i += 1;
    }
    contexts[bytes.len()] = current;
    module_tops[bytes.len()] = mod_stack.last().map_or(0, |(_, d)| d + 1);
    InlineModuleScan {
        contexts,
        modules,
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

/// Content inside the first `{ … }` of `s` (which must start with `{`), honoring nesting. The
/// brace-body extractor of the scanner's one use-tree parser, `scope_graph::use_tree_leaves`.
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

/// Split on commas at brace depth 0 — the use-tree group splitter of the one use-tree parser (see
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
