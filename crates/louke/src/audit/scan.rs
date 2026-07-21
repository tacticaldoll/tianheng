use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// What the source scan found for a probe occurrence (`assert_boundary!`).
#[derive(Debug)]
pub(super) enum Probe {
    /// A probe whose seam is a string literal (auditable, plain or raw): the seam value.
    Literal(String),
    /// A probe whose seam argument is NOT a string literal (a const or expression): the CI
    /// face cannot trace it to a declared seam, so it reacts rather than skipping. Carries the
    /// source file so the reaction is actionable (and the baseline identity stable).
    Unauditable { file: String },
}

pub(super) fn collect_probes(input: &Path, probes: &mut Vec<Probe>) -> Result<(), String> {
    if input.is_file() {
        return collect_reachable_probes(input, probes);
    }
    collect_directory_probes(input, probes)
}

fn collect_directory_probes(dir: &Path, probes: &mut Vec<Probe>) -> Result<(), String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    // Sort entries so the scan order — and thus the violation order in the report — is
    // deterministic across runs (read_dir order is OS/filesystem-dependent and unsorted).
    let mut paths = Vec::new();
    for entry in read {
        let entry =
            entry.map_err(|e| format!("cannot read a dir entry under {}: {e}", dir.display()))?;
        // file_type() does NOT follow symlinks, so a symlinked directory does not recurse —
        // avoiding an infinite loop on a cyclic symlink (fail safe, not stack-overflow loud).
        let file_type = entry
            .file_type()
            .map_err(|e| format!("cannot stat {}: {e}", entry.path().display()))?;
        paths.push((file_type.is_dir(), entry.path()));
    }
    paths.sort();
    for (is_dir, path) in paths {
        if is_dir {
            collect_directory_probes(&path, probes)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            let source = std::fs::read_to_string(&path)
                .map_err(|e| format!("cannot read source {}: {e}", path.display()))?;
            scan_source(&source, &path.display().to_string(), probes);
        }
    }
    Ok(())
}

fn collect_reachable_probes(root: &Path, probes: &mut Vec<Probe>) -> Result<(), String> {
    let root_parent = root
        .parent()
        .ok_or_else(|| format!("source root has no parent: {}", root.display()))?;
    let mut pending = vec![(root.to_path_buf(), root_parent.to_path_buf())];
    let mut visited = BTreeSet::new();
    while let Some((file, child_base)) = pending.pop() {
        if !visited.insert(file.clone()) {
            continue;
        }
        let source = std::fs::read_to_string(&file)
            .map_err(|e| format!("cannot read source {}: {e}", file.display()))?;
        scan_source(&source, &file.display().to_string(), probes);
        let mut children = external_module_files(&source, &child_base)?;
        children.sort();
        children.reverse();
        pending.extend(children);
    }
    Ok(())
}

fn external_module_files(
    source: &str,
    child_base: &Path,
) -> Result<Vec<(PathBuf, PathBuf)>, String> {
    let mut modules = Vec::new();
    collect_scope_modules(source.as_bytes(), 0, source.len(), child_base, &mut modules)?;
    Ok(modules)
}

fn collect_scope_modules(
    bytes: &[u8],
    start: usize,
    end: usize,
    child_base: &Path,
    modules: &mut Vec<(PathBuf, PathBuf)>,
) -> Result<(), String> {
    let mut i = start;
    while i < end {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(end);
            continue;
        }
        if bytes[i] == b'!' && preceding_token_is_ident(bytes, i) {
            if let Some(next) = foreign_macro_body_end(bytes, i) {
                i = next.min(end);
                continue;
            }
        }
        if is_mod_keyword(bytes, i) {
            let mut cursor = skip_ascii_space(bytes, i + 3);
            let name_start = cursor;
            if bytes.get(cursor..cursor + 2) == Some(b"r#") {
                cursor += 2;
            }
            while cursor < end && is_ident_byte(bytes[cursor]) {
                cursor += 1;
            }
            if cursor == name_start
                || (cursor == name_start + 2 && &bytes[name_start..cursor] == b"r#")
            {
                i += 3;
                continue;
            }
            let raw_name = &bytes[name_start..cursor];
            let name = if raw_name.starts_with(b"r#") {
                &raw_name[2..]
            } else {
                raw_name
            };
            let name = std::str::from_utf8(name).map_err(|e| e.to_string())?;
            cursor = skip_ascii_space(bytes, cursor);
            match bytes.get(cursor) {
                Some(b';') => {
                    let attrs = mod_preamble_attrs(bytes, i);
                    if !attrs.path {
                        match resolve_external_module(child_base, name)? {
                            Some(resolved) => modules.push(resolved),
                            // No conventional file. A `#[cfg]`-gated declaration may legitimately
                            // have none in this configuration (an off feature / another platform),
                            // so tolerate it — it compiles no probes here, so skipping it cannot
                            // silently cover a seam. A non-cfg missing module is a real broken
                            // reference: fail loud (exit 2), never a silent skip.
                            None if attrs.cfg => {}
                            None => {
                                return Err(format!(
                                    "cannot resolve reachable module `{name}` under {}",
                                    child_base.display()
                                ));
                            }
                        }
                    }
                    i = cursor + 1;
                    continue;
                }
                Some(b'{') => {
                    let close = balanced_brace_end(bytes, cursor, end);
                    let inline_base = child_base.join(name);
                    collect_scope_modules(
                        bytes,
                        cursor + 1,
                        close.saturating_sub(1),
                        &inline_base,
                        modules,
                    )?;
                    i = close;
                    continue;
                }
                _ => {}
            }
        }
        if bytes[i] == b'{' {
            i = balanced_brace_end(bytes, i, end);
            continue;
        }
        i += 1;
    }
    Ok(())
}

/// Resolve a `mod name;` to its conventional file and the base directory for its own children:
/// `Ok(Some(..))` for `<base>/name.rs` or `<base>/name/mod.rs`, `Ok(None)` when neither exists (the
/// caller decides whether an absent file is a legitimate `#[cfg]`-gated skip or a hard error), and
/// `Err` only for a genuine ambiguity (both files present).
fn resolve_external_module(base: &Path, name: &str) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let flat = base.join(format!("{name}.rs"));
    let nested = base.join(name).join("mod.rs");
    let file = match (flat.is_file(), nested.is_file()) {
        (true, false) => flat,
        (false, true) => nested,
        (true, true) => {
            return Err(format!(
                "module `{name}` resolves to both '{}' and '{}'",
                flat.display(),
                nested.display()
            ));
        }
        (false, false) => return Ok(None),
    };
    let next_base = if file.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
        file.parent().unwrap_or(base).to_path_buf()
    } else {
        file.parent().unwrap_or(base).join(name)
    };
    Ok(Some((file, next_base)))
}

fn is_mod_keyword(bytes: &[u8], i: usize) -> bool {
    bytes.get(i..i + 3) == Some(b"mod")
        && (i == 0 || !is_ident_byte(bytes[i - 1]))
        && bytes.get(i + 3).is_none_or(|b| !is_ident_byte(*b))
}

fn preceding_token_is_ident(bytes: &[u8], bang: usize) -> bool {
    let mut end = bang;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end > 0 && is_ident_byte(bytes[end - 1])
}

fn skip_ascii_space(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

fn balanced_brace_end(bytes: &[u8], open: usize, limit: usize) -> usize {
    let mut depth = 0usize;
    let mut i = open;
    while i < limit {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(limit);
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    limit
}

/// Outer attributes on the `mod name;` at `mod_index` that steer the walker.
struct ModPreambleAttrs {
    /// A `#[path = "..."]` relocation: the module lives at an author-chosen file, so the by-name
    /// resolver must not try (and fail) to resolve it — a stated scan bound.
    path: bool,
    /// A `#[cfg(...)]` / `#[cfg_attr(...)]` gate: the module may legitimately have no file in the
    /// current configuration (an off feature / another platform), so an absent file is tolerated
    /// rather than a scan error — the same cfg-tolerance 渾儀 applies, reimplemented louke-locally
    /// (三儀 ⊥ 三儀). This is not `cfg` evaluation: a resolvable cfg-gated module is still scanned
    /// and its probes still counted; only an *absent* file for a cfg-gated declaration is tolerated.
    cfg: bool,
}

/// Scan a `mod name;`'s preamble (the bytes since the previous item boundary) for the outer
/// attributes that steer the walker. Detection is **structural, not a raw substring**: comments and
/// string literals are skipped, and only an *outer attribute whose meta name is exactly* `path`
/// (followed by `=`), `cfg`, or `cfg_attr` matches. A comment or unrelated attribute that merely
/// contains the text (`// fast path`, `#[cfg(feature = "fastpath")]`) MUST NOT be read as a `path`
/// relocation — a false match would drop a reachable module and every probe under it (a silent
/// coverage false negative, the worst outcome under FN-first). A `#[cfg_attr(.., path = ..)]`
/// conditional relocation reads as `cfg` (its meta name is `cfg_attr`, not `path`), so an absent
/// target is tolerated rather than errored.
fn mod_preamble_attrs(bytes: &[u8], mod_index: usize) -> ModPreambleAttrs {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    let mut attrs = ModPreambleAttrs {
        path: false,
        cfg: false,
    };
    let mut i = start;
    while i < mod_index {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(mod_index);
            continue;
        }
        if bytes[i] == b'#' {
            let mut open = i + 1;
            if bytes.get(open) == Some(&b'!') {
                open += 1;
            }
            if bytes.get(open) == Some(&b'[') {
                // The attribute's meta name is the first identifier inside the brackets.
                let name_start = skip_preamble_trivia(bytes, open + 1, mod_index);
                let mut name_end = name_start;
                while name_end < mod_index && is_ident_byte(bytes[name_end]) {
                    name_end += 1;
                }
                match &bytes[name_start..name_end] {
                    b"path" => {
                        let eq = skip_preamble_trivia(bytes, name_end, mod_index);
                        if bytes.get(eq) == Some(&b'=') {
                            attrs.path = true;
                        }
                    }
                    b"cfg" | b"cfg_attr" => attrs.cfg = true,
                    _ => {}
                }
                i = attr_group_end(bytes, open, mod_index);
                continue;
            }
        }
        i += 1;
    }
    attrs
}

/// Advance past whitespace, comments, and string/char literals to the next significant byte
/// (bounded by `end`). Shared by the attribute walk so a comment or literal inside a preamble
/// never derails the meta-name match.
fn skip_preamble_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
    while i < end {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(end);
            continue;
        }
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        break;
    }
    i
}

/// Index just past the `]` closing the attribute-bracket group opened at `open` (which indexes the
/// `[`), tracking nested `[]` and skipping string/char literals and comments so a `]` inside a
/// `#[path = "a]b.rs"]` literal does not close the group early. Mirrors [`balanced_brace_end`].
fn attr_group_end(bytes: &[u8], open: usize, limit: usize) -> usize {
    let mut depth = 0usize;
    let mut i = open;
    while i < limit {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(limit);
            continue;
        }
        match bytes[i] {
            b'[' => depth += 1,
            b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return i + 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    limit
}

/// Skip a (possibly nested) block comment whose opening `/*` is at `i`, returning the index just
/// past its outermost `*/`. Rust block comments nest, so depth is tracked; an unterminated comment
/// runs to EOF. Shared by [`scan_source`] and [`skip_trivia`] so the two cannot drift — the
/// original non-nested bug existed in *both* precisely because they were independent copies.
fn skip_block_comment(b: &[u8], mut i: usize) -> usize {
    let mut depth = 1usize;
    i += 2; // past the opening `/*`
    while i + 1 < b.len() && depth > 0 {
        if b[i] == b'/' && b[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if b[i] == b'*' && b[i + 1] == b'/' {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    if depth > 0 { b.len() } else { i }
}

/// Walk source skipping comments / string & char literals, and when the `assert_boundary!`
/// probe marker appears in code, record whether its seam argument is a string literal
/// (auditable) or not (un-auditable). Declarations come from the passed `RuntimeBoundary` objects.
/// `file` labels an un-auditable probe so the reaction is actionable.
pub(super) fn scan_source(source: &str, file: &str, probes: &mut Vec<Probe>) {
    let b = source.as_bytes();
    let mut i = 0;
    while i < b.len() {
        // Comments and string/char literals are skipped whole (one shared definition below), so
        // a marker or delimiter inside them is never mis-read.
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        // A left word boundary: `my_assert_boundary!` / `xassert_boundary!` are unrelated user
        // macros, not our probe. Require the preceding byte to be a non-identifier char so a
        // marker embedded in a longer identifier is not mis-counted as a probe.
        let left_boundary = i == 0 || !is_ident_byte(b[i - 1]);
        if left_boundary {
            if let Some(rest) = match_probe_marker(b, i) {
                let (probe, next) = capture_probe(b, rest, file);
                if let Some(probe) = probe {
                    probes.push(probe);
                }
                i = next;
                continue;
            }
        }
        // A foreign macro invocation / `macro_rules!` definition body is macro-generated or dead
        // code: a probe lexically inside it must not count as coverage (the 圭表 strip_macro_bodies
        // rule, reimplemented louke-locally — 三儀 ⊥ 三儀 forbids importing it). `assert_boundary!`'s
        // own `!` is consumed by the marker branch above (and `capture_probe` advances past it), so
        // a `!`-preceded-by-identifier reached here is always a FOREIGN macro; skip its balanced
        // body (and any probe nested in it) in one jump.
        if b[i] == b'!' {
            // A foreign macro's `!` may be separated from its name by whitespace (`some_macro !(…)`
            // is valid Rust), mirroring the probe marker's own gap tolerance — so look back past
            // whitespace for the name's last identifier byte before deciding this opens a macro
            // body. (A comment between the name and `!` stays a documented bound: rustfmt removes
            // it, and scanning back over a block comment is not worth the cost.)
            let mut name_end = i;
            while name_end > 0 && b[name_end - 1].is_ascii_whitespace() {
                name_end -= 1;
            }
            let mut name_start = name_end;
            while name_start > 0 && is_ident_byte(b[name_start - 1]) {
                name_start -= 1;
            }
            // A raw identifier `r#keyword` (e.g. a macro named `r#async`) escapes the keyword and IS
            // a valid macro name — its body must still be skipped. The ident-run stops at the `#`
            // (not an ident byte), so detect a preceding `r#` at a word boundary and exempt it from
            // the keyword test below.
            let is_raw_ident = name_start >= 2
                && b[name_start - 1] == b'#'
                && b[name_start - 2] == b'r'
                && (name_start == 2 || !is_ident_byte(b[name_start - 3]));
            // Otherwise the name before `!` must be a real identifier that is NOT a keyword. A
            // keyword there is unary negation in expression position (`return !(x)`, `if !(cond) {…}`,
            // `match !(x)`), never a macro — treating its parenthesized operand as a macro body would
            // skip real code (and drop any probe inside it). `macro_rules` is not a keyword, so it
            // still reaches `foreign_macro_body_end`'s name-skip.
            if name_start < name_end && (is_raw_ident || !is_rust_keyword(&b[name_start..name_end]))
            {
                if let Some(end) = foreign_macro_body_end(b, i) {
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// If `i` begins a comment or a string/char literal, return the index just past it; else `None`.
/// One shared definition for the main scan and the macro-body skip, so their literal/comment
/// handling can never drift apart (the independent-copy drift `skip_block_comment` warns about).
/// Raw/byte strings are tested before plain strings (an inner `"` would otherwise desync), and a
/// lifetime (`'a`) is deliberately NOT a literal (left to be walked as code).
fn skip_literal_or_comment(b: &[u8], i: usize) -> Option<usize> {
    // line comment
    if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
        let mut j = i;
        while j < b.len() && b[j] != b'\n' {
            j += 1;
        }
        return Some(j);
    }
    // block comment (nesting + drift rationale in `skip_block_comment`)
    if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
        return Some(skip_block_comment(b, i));
    }
    // raw / byte string literal (r"…", r#"…"#, b"…", br#"…"#) — before the plain-string case
    if let Some(end) = raw_or_byte_string_end(b, i) {
        return Some(end);
    }
    // plain string literal
    if b[i] == b'"' {
        let mut j = i + 1;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    // char literal vs lifetime: only a clear char ('x' or '\n'); a lifetime ('a) is not a literal.
    if b[i] == b'\'' {
        let is_char =
            (i + 1 < b.len() && b[i + 1] == b'\\') || (i + 2 < b.len() && b[i + 2] == b'\'');
        if is_char {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'\'' {
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            return Some((j + 1).min(b.len()));
        }
    }
    None
}

/// The identifier run ending immediately before `end` equals `target`. Used to recognize a
/// `macro_rules` keyword before its `!` (the only stable form taking a `name` between `!` and the
/// body delimiter) without a false match on `my_macro_rules` (the maximal run differs).
fn preceding_ident_is(b: &[u8], end: usize, target: &[u8]) -> bool {
    let mut start = end;
    while start > 0 && is_ident_byte(b[start - 1]) {
        start -= 1;
    }
    &b[start..end] == target
}

/// Given `bang` where `b[bang] == b'!'` and the preceding byte is an identifier byte, return the
/// index past a foreign macro's balanced body, or `None` when this `!` does not open one — `!=`,
/// unary `!expr`, or a keyword glued to `!` (`if!cond {…}` / `while!x {…}` / `match!x {…}`), none of
/// which is a macro. `macro_rules! name {…}` is the sole form with an identifier between `!` and the
/// delimiter, so the name-skip is gated on the preceding identifier being exactly `macro_rules`;
/// treating any `ident! ident {` as a macro would swallow a real `if`/`while`/`match` block and drop
/// a probe inside it (a reintroduced false negative). The balanced walk reuses
/// `skip_literal_or_comment`, so a delimiter inside a string/char/comment never closes early; an
/// unterminated body at EOF returns `Some(len)`.
fn foreign_macro_body_end(b: &[u8], bang: usize) -> Option<usize> {
    let mut i = skip_trivia(b, bang + 1);
    // The name may be separated from `!` by whitespace (`macro_rules ! foo {…}` is valid Rust),
    // exactly as the caller tolerates when deciding this `!` opens a macro. Skip back over that
    // whitespace before the keyword test — anchoring at `bang` would miss the spaced form, leaving
    // the body (and any probe inside it) unskipped and wrongly counted as coverage (a false negative).
    let mut name_end = bang;
    while name_end > 0 && b[name_end - 1].is_ascii_whitespace() {
        name_end -= 1;
    }
    if preceding_ident_is(b, name_end, b"macro_rules") {
        let name_start = i;
        while i < b.len() && (is_ident_byte(b[i]) || b[i] == b'#') {
            i += 1;
        }
        if i == name_start {
            return None; // `macro_rules!` with no name — malformed, not a body to skip
        }
        i = skip_trivia(b, i);
    }
    if !matches!(b.get(i), Some(b'{') | Some(b'(') | Some(b'[')) {
        return None;
    }
    // One depth counter over all three delimiter kinds: correct because the audit scans compilable
    // Rust, whose token trees are properly nested (a `)` never closes a `{`). Literals/comments are
    // skipped first each iteration, so a delimiter inside a string/char never perturbs the count.
    let mut depth = 0usize;
    while i < b.len() {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        match b[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Some(b.len())
}

/// Detect a raw or byte string literal starting at `i` (`r"…"`, `r#"…"#`, `b"…"`,
/// `br"…"`, `br#"…"#`) and return the index past its end, or `None` if `i` is not such a
/// literal. Rust syntax guarantees `r`/`b` immediately before `"`/`#` is a literal prefix
/// (no identifier can precede a string), so no token-boundary check is needed.
fn raw_or_byte_string_end(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    let byte = j < b.len() && b[j] == b'b';
    if byte {
        j += 1;
    }
    let raw = j < b.len() && b[j] == b'r';
    if raw {
        j += 1;
        let mut hashes = 0;
        while j < b.len() && b[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j >= b.len() || b[j] != b'"' {
            return None;
        }
        j += 1;
        // scan to the closing `"` followed by `hashes` `#`s
        while j < b.len() {
            if b[j] == b'"' {
                let mut k = j + 1;
                let mut h = 0;
                while k < b.len() && h < hashes && b[k] == b'#' {
                    k += 1;
                    h += 1;
                }
                if h == hashes {
                    return Some(k);
                }
            }
            j += 1;
        }
        return Some(b.len());
    }
    // a `b"…"` byte string (escaped like a normal string) — only when a `b` prefix was
    // consumed and a quote immediately follows.
    if byte && j < b.len() && b[j] == b'"' {
        j += 1;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    None
}

/// Match the probe marker at `i`: the identifier `assert_boundary` at a word boundary, then — as
/// `ident ! (…)` with whitespace/comments between the name and `!` is valid Rust (`println !("x")`
/// compiles) — its `!`. Returns the index just past the `!`, whence [`capture_probe`] skips trivia
/// to the opening delimiter; `None` otherwise. The right word boundary rejects a longer identifier
/// like `assert_boundaryx`; the caller checks the left boundary. Tolerating the gap closes a false
/// negative: a probe written `assert_boundary !("seam")` was silently dropped by a contiguous match.
fn match_probe_marker(b: &[u8], i: usize) -> Option<usize> {
    const NAME: &[u8] = b"assert_boundary";
    if i + NAME.len() > b.len() || &b[i..i + NAME.len()] != NAME {
        return None;
    }
    let after_name = i + NAME.len();
    // Right word boundary: `assert_boundaryx` / `assert_boundary_probe` is a different identifier.
    if b.get(after_name).is_some_and(|&c| is_ident_byte(c)) {
        return None;
    }
    let bang = skip_trivia(b, after_name);
    if b.get(bang) != Some(&b'!') {
        return None;
    }
    Some(bang + 1)
}

/// An identifier byte — ASCII `[A-Za-z0-9_]` or any UTF-8 non-ASCII byte (`>= 0x80`). Used for the
/// marker's word boundary: a multi-byte Unicode identifier char (`Ω` in `Ωassert_boundary`) is XID
/// and must keep the boundary, so a foreign macro whose name merely *ends* in `assert_boundary` is
/// not mis-read as our probe. ASCII-only would treat the `Ω` continuation bytes as a boundary and
/// falsely match (a false coverage / fabricated probed-but-undeclared reaction).
fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

/// Whether the identifier run `word` is a Rust keyword (strict or reserved). A macro name is a real
/// identifier and never a keyword, so a keyword immediately before `!` is unary negation
/// (`return !(x)`, `if !(cond) {…}`), not a macro invocation — its operand must not be skipped as a
/// macro body. `macro_rules` is deliberately absent (it is not a keyword and must reach the
/// name-skip). A non-ASCII / non-UTF-8 run is never a keyword.
fn is_rust_keyword(word: &[u8]) -> bool {
    let Ok(word) = std::str::from_utf8(word) else {
        return false;
    };
    matches!(
        word,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            // reserved / edition keywords
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
            | "gen"
    )
}

/// Skip ASCII whitespace and `//` / `/* */` comments, returning the next code index. Mirrors
/// the comment handling in [`scan_source`] so a comment between the `!` and `(`, or before the
/// seam argument, does not desync probe capture (which would silently drop a real probe).
fn skip_trivia(b: &[u8], mut i: usize) -> usize {
    loop {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'*') {
            i = skip_block_comment(b, i);
            continue;
        }
        return i;
    }
}

/// After the `assert_boundary!` marker, classify the probe by its first argument and return
/// `(probe, next_index)`. Skip trivia, expect a macro opening delimiter (`(`, `{`, or `[`),
/// skip trivia; a plain or raw string first argument is an auditable [`Probe::Literal`] (its
/// value); any other first token (a `const`, an expression, a byte string) is
/// [`Probe::Unauditable`] — never a silent skip. `None` (with `next` past the marker) only when
/// the marker is not actually a probe call (no opening delimiter follows).
fn capture_probe(b: &[u8], i: usize, file: &str) -> (Option<Probe>, usize) {
    let i = skip_trivia(b, i);
    // Rust macros accept `( )`, `{ }`, or `[ ]` interchangeably; a probe written
    // `assert_boundary!{"s", o}` or `["s", o]` is a real probe. Accept any of the three
    // opening delimiters so a non-`()` probe is not silently dropped — a silent drop would let
    // a typo'd seam escape the undeclared-seam check, a false negative.
    if !matches!(b.get(i), Some(&b'(') | Some(&b'{') | Some(&b'[')) {
        return (None, i);
    }
    let i = skip_trivia(b, i + 1);
    if i >= b.len() {
        return (None, i);
    }
    // A raw string `r"…"` / `r#"…"#` is a traceable literal — parse its value rather than
    // rejecting it as un-auditable (which would mis-flag a legitimate probe and double-report).
    if b[i] == b'r' && matches!(b.get(i + 1), Some(b'"') | Some(b'#')) {
        if let Some((seam, next)) = raw_string_value(b, i) {
            return (Some(Probe::Literal(seam)), next);
        }
        return (
            Some(Probe::Unauditable {
                file: file.to_string(),
            }),
            i,
        );
    }
    // A plain string literal. Find its end (the `\\`-skip only keeps a `\"` from ending the
    // string early), then DECODE its escapes to the value the compiler produces — the declared
    // seam set is compiler-decoded (`RuntimeBoundary::seam()`), so comparing the raw source bytes
    // would let an escape-bearing seam diverge between the two faces (a false pair of reactions,
    // and a false negative when two spellings decode to the same bytes). An escape the decoder
    // cannot reproduce exactly reacts as un-auditable (loud), never a silently mismatched literal.
    if b[i] == b'"' {
        let mut j = i + 1;
        let start = j;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        if j >= b.len() {
            return (None, j);
        }
        return match decode_str_escapes(&b[start..j]) {
            Some(seam) => (Some(Probe::Literal(seam)), j + 1),
            None => (
                Some(Probe::Unauditable {
                    file: file.to_string(),
                }),
                j + 1,
            ),
        };
    }
    // Anything else (a const, an expression, a byte string) cannot be traced to a declared seam.
    (
        Some(Probe::Unauditable {
            file: file.to_string(),
        }),
        i,
    )
}

/// Parse a raw string literal `r"…"` / `r#…"…"#…` starting at `i`, returning `(value, next)`.
/// `None` if it is not a well-formed raw string.
fn raw_string_value(b: &[u8], i: usize) -> Option<(String, usize)> {
    let mut j = i + 1; // past `r`
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    let start = j;
    while j < b.len() {
        if b[j] == b'"' {
            let mut k = j + 1;
            let mut h = 0;
            while h < hashes && b.get(k) == Some(&b'#') {
                k += 1;
                h += 1;
            }
            if h == hashes {
                return Some((String::from_utf8_lossy(&b[start..j]).into_owned(), k));
            }
        }
        j += 1;
    }
    None
}

/// Decode a plain-string literal's inner bytes (between the quotes, escapes still present) to the
/// exact `&str` value the Rust compiler produces, so a probe seam matches the compiler-decoded
/// declared seam (`RuntimeBoundary::seam()`) rather than the raw source bytes. Returns `None` on any
/// escape the decoder does not reproduce exactly — a malformed or unrecognized escape, an
/// out-of-range `\x`, an invalid `\u{…}`, or a backslash-newline **line continuation** (deliberately
/// not decoded: it *strips* characters, the one escape class that could yield a wrong non-`None`
/// value and reintroduce a false negative, and no real seam name spans lines). The caller routes
/// `None` to an un-auditable probe (a loud reaction), never a silent mismatch. The escape set is the
/// `&str` string-literal set only; byte-string-only escapes never reach here (byte strings are
/// already un-auditable).
fn decode_str_escapes(inner: &[u8]) -> Option<String> {
    // The surrounding source compiled, so it is valid UTF-8; escapes are all ASCII, so iterating
    // by `char` reconstructs any multi-byte content faithfully.
    let s = std::str::from_utf8(inner).ok()?;
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '\\' => out.push('\\'),
            '0' => out.push('\0'),
            '\'' => out.push('\''),
            '"' => out.push('"'),
            // `\xHH`: exactly two hex digits, and (for a `&str`) a value in `0x00..=0x7F`.
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                let v = hi * 16 + lo;
                if v > 0x7F {
                    return None;
                }
                out.push(char::from_u32(v)?);
            }
            // `\u{ H..H }`: 1..=6 hex digits (`_` permitted as separators), a valid `char`.
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut value: u32 = 0;
                let mut digits = 0;
                loop {
                    match chars.next()? {
                        '}' => break,
                        // A leading `_` is "invalid start of unicode escape" in rustc; only
                        // internal/trailing separators are legal, so match rustc exactly here.
                        '_' if digits == 0 => return None,
                        '_' => continue,
                        d => {
                            let hd = d.to_digit(16)?;
                            digits += 1;
                            if digits > 6 {
                                return None;
                            }
                            value = value * 16 + hd;
                        }
                    }
                }
                if digits == 0 {
                    return None;
                }
                out.push(char::from_u32(value)?);
            }
            // An unrecognized escape or a backslash-newline line continuation: react loud.
            _ => return None,
        }
    }
    Some(out)
}
