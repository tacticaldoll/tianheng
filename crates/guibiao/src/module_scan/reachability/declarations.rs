//! Lexical extraction of top-level `mod` declarations and their cfg/path attributes.

#[cfg(test)]
use super::super::lexer::clean_with_positions;
use super::super::lexer::{balanced_group_end, is_ident_byte, transparent_macro_body_at};
use super::super::path_vocab::{canonical_segment, is_mod_declaration_keyword};

/// One `mod` declared at the top level of a byte range within already-cleaned (comment/string/
/// macro-body-stripped) text: its canonical name, whether it is inline (`{ … }`, `true`) or file
/// (`;`, `false`), and — for an inline declaration — the byte range of its body's *content*
/// (excluding the enclosing braces), so a caller can re-scan just that span to find further
/// declarations nested inside it. `direct_path_eq` is the cleaned-text position of the `=` in an
/// **unconditional** `#[path = "…"]` preceding a FILE declaration — cleaning has already dropped
/// the quoted value itself, so a caller resolves it by mapping this position back to the
/// original source (see [`super::lexer::clean_with_positions`]) and reading from there.
pub(super) struct DeclaredModule {
    pub(super) name: String,
    pub(super) is_inline: bool,
    pub(super) body: Option<(usize, usize)>,
    pub(super) direct_path_eq: Option<usize>,
    pub(super) conditional_path_eqs: Vec<usize>,
    /// Whether a BARE `#[cfg(...)]` (never `cfg_attr`) precedes this declaration — see
    /// [`has_bare_cfg_attr_before_item`]. Only meaningful for a non-inline (file-form) declaration
    /// with no resolvable file: it is the "might legitimately be absent on this build" signal, the
    /// same one hunyi's `has_cfg_attr` checks.
    pub(super) has_bare_cfg: bool,
}

/// The test-only `declared_modules_with_kind` generalized to scan `cleaned[range]` instead of a
/// whole file, so it can be re-applied to an inline module's own body — the byte span between its
/// braces — to find the `mod` declarations nested inside it. `path_attr_before_item` scans backward
/// from a candidate unbounded by `range.start`, which stays correct here: the nearest preceding
/// `;`/`{`/`}` it finds is either an earlier sibling's terminator within the range or the range's
/// own enclosing `{`, never a byte outside the declaration it is checking.
pub(super) fn declared_modules_in(
    cleaned: &str,
    range: std::ops::Range<usize>,
) -> Vec<DeclaredModule> {
    let bytes = cleaned.as_bytes();
    let end = range.end.min(bytes.len());
    let mut declared = Vec::new();
    let mut i = range.start.min(end);

    struct MacroScope {
        open_pos: usize,
        close_pos: usize,
        macro_depth: usize,
        inherited_top_level: bool,
    }
    let mut macro_scopes: Vec<MacroScope> = Vec::new();
    let mut file_depth = 0usize;

    while i < end {
        // Pop any completed macro scopes
        while let Some(active) = macro_scopes.last() {
            if i >= active.close_pos {
                macro_scopes.pop();
            } else {
                break;
            }
        }

        // Check if `i` is the `!` of a transparent macro invocation (`cfg_if!`)
        if let Some((open_pos, close_pos)) = transparent_macro_body_at(bytes, i) {
            let inherited_top_level = if let Some(parent) = macro_scopes.last() {
                parent.inherited_top_level && (parent.macro_depth == 1)
            } else {
                file_depth == 0
            };
            macro_scopes.push(MacroScope {
                open_pos,
                close_pos,
                macro_depth: 0,
                inherited_top_level,
            });
            i += 1;
            continue;
        }

        let is_top_level = if let Some(active) = macro_scopes.last() {
            active.inherited_top_level && (active.macro_depth == 1)
        } else {
            file_depth == 0
        };

        match bytes[i] {
            b'{' | b'(' | b'[' => {
                if let Some(active) = macro_scopes.last_mut() {
                    if i > active.open_pos {
                        active.macro_depth += 1;
                    }
                } else if bytes[i] == b'{' {
                    file_depth += 1;
                }
                i += 1;
            }
            b'}' | b')' | b']' => {
                if let Some(active) = macro_scopes.last_mut() {
                    if i > active.open_pos {
                        active.macro_depth = active.macro_depth.saturating_sub(1);
                    }
                } else if bytes[i] == b'}' {
                    file_depth = file_depth.saturating_sub(1);
                }
                i += 1;
            }
            b'm' if is_top_level && is_mod_declaration_keyword(bytes, i) => {
                let mut j = i + 3;
                while j < end && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < end
                    && !bytes[j].is_ascii_whitespace()
                    && bytes[j] != b';'
                    && bytes[j] != b'{'
                {
                    j += 1;
                }
                let ident = cleaned[start..j].trim();
                let mut k = j;
                while k < end && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if !ident.is_empty() {
                    match bytes.get(k) {
                        Some(b'{') => {
                            // Skip the whole body in one jump — its content is re-scanned only if
                            // this module turns out to be inline-only, from `body` below. The
                            // module itself is always declared regardless of a preceding
                            // `#[path]` (rustc's `path` attribute never relocates an inline
                            // body's OWN content — the body already IS the module). It is NOT a
                            // no-op, though: it relocates the base directory THIS body's own
                            // file-form children resolve from (verified against a real rustc
                            // build: `#[path = "d"] mod x { mod y; }` compiles `y` at `d/y.rs`,
                            // never `<parent's child_base>/x/y.rs`) — an unconditional direct
                            // value is captured here (`direct_path_eq`) for exactly that reason;
                            // a `cfg_attr`-wrapped one stays the same stated, cfg-conditional skip
                            // bound as the file-form case (never followed cfg-blind).
                            let (direct_path_eq, conditional_path_eqs) =
                                match path_attr_before_item(bytes, i) {
                                    PathAttrKind::Remaps {
                                        direct,
                                        conditional,
                                    } => (direct, conditional),
                                    PathAttrKind::None | PathAttrKind::Excluded => {
                                        (None, Vec::new())
                                    }
                                };
                            let close = balanced_group_end(bytes, k).unwrap_or(bytes.len());
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: true,
                                body: Some((k + 1, close.saturating_sub(1))),
                                direct_path_eq,
                                conditional_path_eqs,
                                has_bare_cfg: false,
                            });
                            i = close;
                            continue;
                        }
                        Some(b';') => {
                            let has_bare_cfg = has_bare_cfg_attr_before_item(bytes, i);
                            let (direct_path_eq, conditional_path_eqs) =
                                match path_attr_before_item(bytes, i) {
                                    PathAttrKind::Remaps {
                                        direct,
                                        conditional,
                                    } => (direct, conditional),
                                    PathAttrKind::None | PathAttrKind::Excluded => {
                                        (None, Vec::new())
                                    }
                                };
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: false,
                                body: None,
                                direct_path_eq,
                                conditional_path_eqs,
                                has_bare_cfg,
                            });
                        }
                        _ => {}
                    }
                }
                i += 3;
            }
            _ => i += 1,
        }
    }
    declared
}

/// Names of modules declared at the top level (brace depth 0) of `source`, each paired with
/// whether it is an **inline** declaration (`mod name { … }`, `true`) or a **file** declaration
/// (`mod name;`, `false`) — the distinction [`reachable_modules`] needs to tell a real
/// file-backed module from an inline body whose same-named conventional file is an orphan.
/// Declared at any visibility (`pub mod`, `pub(crate) mod`, …). Comments, string/char literals,
/// and macro bodies are stripped first, so a commented-out, quoted, or macro-generated `mod` is
/// not counted; a `mod` nested inside another item (depth > 0) declares a child module, not a
/// crate-root one, and is skipped. Names are canonicalized (`r#name` -> `name`). Robust over
/// malformed input: it never panics (the same tolerance as the `use` scanner). Test-only: the
/// reachability walk itself calls [`declared_modules_in`] directly (over both whole files and
/// inline body spans), so production code no longer goes through this whole-file convenience.
#[cfg(test)]
fn declared_modules_with_kind(source: &str) -> Vec<(String, bool)> {
    // Strip macro bodies as well as comments/strings, the same hygiene the `use`
    // scanner applies: a `mod` written inside a macro body is macro-generated and out
    // of scope, so it must not be observed as a real declaration. (A `macro_rules!`
    // body is already excluded by brace depth; this also closes the `()`/`[]`-delimited
    // invocation gap, where `mod` would otherwise sit at brace depth 0.)
    let (cleaned, _positions) = clean_with_positions(source);
    let len = cleaned.len();
    declared_modules_in(&cleaned, 0..len)
        .into_iter()
        .map(|declared| (declared.name, declared.is_inline))
        .collect()
}

/// The declared module names only, discarding the inline/file kind — a test-only convenience
/// wrapping [`declared_modules_with_kind`] (itself test-only; see its doc).
#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declared_modules_with_kind(source)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// What the top-level item prefix before a `mod` keyword says about a `#[path]` remap. The
/// static scanner intentionally does not read attributes in general, but `path` is a stated
/// coverage concern either way: an unconditional, direct `#[path = "…"]` is now *followed*
/// (`Direct`, carrying the cleaned-text position of its `=`, so the real value can be read from
/// the untouched original source); a `cfg_attr`-wrapped one stays cfg-conditional and excluded,
/// same as a bare `path`-named attribute with no followable value — both `Excluded`, matching the
/// stated bound: a path-remapped module is not conventionally file-backed, so treating the `mod`
/// token as an ordinary file declaration would govern the wrong file (or a same-named orphan).
enum PathAttrKind {
    None,
    Remaps {
        direct: Option<usize>,
        conditional: Vec<usize>,
    },
    Excluded,
}

fn path_attr_before_item(bytes: &[u8], mod_index: usize) -> PathAttrKind {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    match attr_prefix_path_kind(&bytes[start..mod_index]) {
        PathAttrKind::Remaps {
            direct,
            conditional,
        } => PathAttrKind::Remaps {
            direct: direct.map(|rel| start + rel),
            conditional: conditional.into_iter().map(|rel| start + rel).collect(),
        },
        other => other,
    }
}

fn attr_prefix_path_kind(bytes: &[u8]) -> PathAttrKind {
    let mut i = 0;
    let mut excluded = false;
    let mut direct = None;
    let mut conditional_eqs = Vec::new();
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'[') {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes[i..].starts_with(b"path")
            && bytes.get(i + 4).is_none_or(|byte| !is_ident_byte(*byte))
        {
            // Retain an unconditional direct path alongside every cfg-conditional candidate.
            // rustc currently gives multiple path-bearing attributes textual precedence (and
            // warns that accepting the shape will become an error), while this scanner is
            // deliberately cfg-blind. Unioning every physically existing written candidate is
            // therefore the only false-negative-safe observation: neither attribute order nor
            // the active predicate may silently remove governed source.
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes.get(j) == Some(&b'=') {
                direct = Some(j);
                i = j + 1;
                continue;
            }
            // A bare `#[path]`/`#[path(...)]` (not valid remap syntax) excludes on its own, but
            // a later unconditional `#[path = "…"]` on the same item still wins — keep scanning
            // rather than returning.
            excluded = true;
            continue;
        }
        // The combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
        // `#[cfg(<pred>)] #[path = "…"]`) is a conditional remap. Collect candidate path = "..."
        // positions across all cfg_attr occurrences. An unconditional `#[path = "…"]` elsewhere
        // on the same item still wins (above), so this keeps scanning instead of returning immediately.
        if bytes[i..].starts_with(b"cfg_attr")
            && bytes.get(i + 8).is_none_or(|byte| !is_ident_byte(*byte))
        {
            cfg_attr_prefix_collect_path_eqs(&bytes[i + 8..], i + 8, &mut conditional_eqs);
            if conditional_eqs.is_empty() && cfg_attr_prefix_has_path(&bytes[i + 8..]) {
                excluded = true;
            }
            continue;
        }
    }
    if direct.is_some() || !conditional_eqs.is_empty() {
        PathAttrKind::Remaps {
            direct,
            conditional: conditional_eqs,
        }
    } else if excluded {
        PathAttrKind::Excluded
    } else {
        PathAttrKind::None
    }
}

fn cfg_attr_prefix_collect_path_eqs(bytes: &[u8], base_offset: usize, eqs: &mut Vec<usize>) {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return;
    }
    i += 1;
    let mut depth = 1usize;
    let mut past_predicate = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                if ident == b"path" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        eqs.push(base_offset + j);
                    }
                } else if ident == b"cfg_attr" {
                    cfg_attr_prefix_collect_path_eqs(&bytes[i..], base_offset + i, eqs);
                }
            }
            _ => i += 1,
        }
    }
}

/// Whether a BARE `#[cfg(...)]` attribute (never `cfg_attr`) is among the attribute prefix
/// immediately preceding an item — the same "might legitimately be absent on this build" signal
/// hunyi's `has_cfg_attr` checks via `syn` (`crate::syn_util::has_cfg_attr`), hand-rolled here for
/// this crate's syn-free scanner. Deliberately narrow: this detects mere PRESENCE of the `cfg`
/// identifier, never evaluates a predicate — the same syntactic-identifier-only shape already used
/// above to detect `path`/`cfg_attr`, not a new capability tier or a step toward general attribute
/// evaluation. `cfg_attr` is deliberately excluded (verified against a real `rustc` build): unlike
/// a bare `#[cfg(pred)]`, which removes the whole item when `pred` is false, `#[cfg_attr(pred, …)]`
/// never removes the item — it only conditionally applies its wrapped attribute — so it must never
/// grant this tolerance (`#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a
/// genuine compile error, E0583, on every platform).
fn has_bare_cfg_attr_before_item(bytes: &[u8], mod_index: usize) -> bool {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    attr_prefix_has_bare_cfg(&bytes[start..mod_index])
}

fn attr_prefix_has_bare_cfg(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'[') {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // The byte immediately after `cfg` must not continue the identifier (excludes `cfg_attr`,
        // whose next byte is `_`).
        if bytes[i..].starts_with(b"cfg")
            && bytes.get(i + 3).is_none_or(|byte| !is_ident_byte(*byte))
        {
            return true;
        }
        i += 1;
    }
    false
}

/// Whether a `cfg_attr(…)` attribute — `bytes` positioned just after the `cfg_attr` identifier —
/// carries a `path` meta among its **applied attributes**. `cfg_attr(<predicate>, <attr>, …)`: the
/// first meta is the cfg predicate (a condition, not an applied attribute), so it is **skipped**
/// before matching — mirroring hunyi's `is_path_remap` (`metas.iter().skip(1)`), so the two
/// dimensions agree. Scans the balanced parenthesis group and matches a depth-1 `path` identifier,
/// past the predicate, immediately followed by `=` (the `path = "…"` name-value form); it also
/// **recurses** into a nested applied `cfg_attr(…)`, so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
/// detected too. Conservative — a same-suffixed identifier (`target_path`), a `path` nested inside a
/// predicate group (`all(…)`), or a `path` in the predicate position is **not** matched — so a
/// non-remapping `cfg_attr` is never mistaken for a remap (which would drop a governed module — the
/// inverse false negative).
///
/// Input note: this runs on comment/string-stripped bytes (`declared_modules_with_kind` applies
/// `strip_comments_and_strings` first), so a `path` inside a string literal cannot reach here; the
/// `b'"'` arm below is defense-in-depth for that upstream invariant, not a live path.
fn cfg_attr_prefix_has_path(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return false;
    }
    i += 1;
    let mut depth = 1usize;
    // The first depth-1 meta is the cfg predicate, not an applied attribute; only match a `path`
    // meta AFTER the first depth-1 comma, so `#[cfg_attr(path = "…", …)]` (a `path` cfg key) is not
    // mistaken for a remap.
    let mut past_predicate = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'"' => {
                // Strings are stripped upstream (see doc); defense-in-depth for the invariant.
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                if ident == b"path" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        return true;
                    }
                } else if ident == b"cfg_attr" && cfg_attr_prefix_has_path(&bytes[i..]) {
                    // A nested `cfg_attr(<pred>, …)` applied meta: recurse into ITS group (which
                    // skips its own predicate), so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
                    // detected too — matching hunyi's recursive `is_path_remap`.
                    return true;
                }
            }
            _ => i += 1,
        }
    }
    false
}
