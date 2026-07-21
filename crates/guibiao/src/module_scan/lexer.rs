//! Lexical hygiene for the source scanner: strip comments, string/char literals, and
//! macro bodies to structural text, and the token-boundary primitives the `use`/`mod`
//! walks stand on (`keyword_starts_at`, `is_ident_byte`). Pure byte processing — no model
//! type, no path, only `std` — feeding the module-graph walk in the parent module.

/// Remove macro bodies so a `use` written inside a macro — a macro-generated import,
/// out of scope per the module-boundary spec — is not mistaken for a real import. Two
/// forms are stripped: a `macro_rules! name <delim>…<delim>` **definition** (name and
/// balanced body), and a macro **invocation** `ident! <delim>…<delim>` (the balanced
/// body; the `ident!` head is kept, harmlessly). Runs on already comment/string-stripped
/// text, so every delimiter is structural and a `macro`/`!` inside a comment or string is
/// not matched. A real `use` is never inside a macro body, so nothing real is dropped.
/// The body delimiter may be `{}`, `()`, or `[]`. Never panics on malformed input.
pub(super) fn strip_macro_bodies(source: &str) -> String {
    let identity: Vec<usize> = (0..source.len()).collect();
    strip_macro_bodies_tracked(source, &identity).0
}

/// [`strip_macro_bodies`], additionally returning a position map like
/// [`strip_comments_and_strings_tracked`]: `positions[k]` is `input_positions[j]`, where `j` is
/// the index in `source` that produced `out[k]` — so a caller chaining this after
/// [`strip_comments_and_strings_tracked`] gets positions all the way back to the true original
/// source, not just this stage's input. `input_positions` must be at least as long as `source`.
pub(super) fn strip_macro_bodies_tracked(
    source: &str,
    input_positions: &[usize],
) -> (String, Vec<usize>) {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut positions: Vec<usize> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = macro_rules_body_end(bytes, i) {
            // `macro_rules! name <delim>…<delim>` — drop the name and the body.
            out.push(b' ');
            positions.push(input_positions[i]);
            i = end;
        } else if bytes[i] == b'!' && preceding_macro_name(bytes, i) {
            // A macro invocation `path ! <delim>…<delim>`: keep the `!`, drop the body. Rust allows
            // whitespace between the macro path and its `!` (`cfg_if ! { … }`), so the macro name is
            // found across whitespace by `preceding_macro_name`. The `!` of `macro_rules!` never
            // reaches here — the definition arm above consumes it. `!=` / unary `!expr` are not
            // invocations: the byte after `!` is not an opening delimiter, so
            // `macro_invocation_body_end` returns `None`; and a unary `!` on a parenthesized or block
            // expression after a keyword (`return !(x)`, `break !{ … }`) is excluded by
            // `preceding_macro_name` (a keyword is not a macro name), so a governed `use` inside such
            // a real block is never wrongly stripped.
            match macro_invocation_body_end(bytes, i) {
                Some(end) => {
                    out.push(b'!');
                    positions.push(input_positions[i]);
                    out.push(b' ');
                    positions.push(input_positions[i]);
                    i = end;
                }
                None => {
                    out.push(bytes[i]);
                    positions.push(input_positions[i]);
                    i += 1;
                }
            }
        } else {
            out.push(bytes[i]);
            positions.push(input_positions[i]);
            i += 1;
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), positions)
}

/// If a `macro_rules! name <delim>…<delim>` definition begins at `i`, return the index
/// just past its balanced closing delimiter; otherwise `None`. `macro_rules` must be a
/// standalone word, followed by `!`, a macro name, and an opening `{`/`(`/`[`.
fn macro_rules_body_end(bytes: &[u8], i: usize) -> Option<usize> {
    const KW: &[u8] = b"macro_rules";
    if !bytes[i..].starts_with(KW) || (i > 0 && is_ident_byte(bytes[i - 1])) {
        return None;
    }
    let skip_ws = |mut j: usize| {
        while j < bytes.len() && bytes[j].is_ascii_whitespace() {
            j += 1;
        }
        j
    };
    let mut j = skip_ws(i + KW.len());
    if bytes.get(j) != Some(&b'!') {
        return None;
    }
    j = skip_ws(j + 1);
    // The macro name — identifier bytes, tolerating a raw-identifier prefix
    // (`macro_rules! r#try`): `#` is not an identifier byte, so a plain ident scan would stop at
    // `r`, `balanced_group_end` would then decline at `#`, and the definition body would be left
    // unstripped — wrongly observing a `use`/`mod` inside a never-invoked macro definition.
    if bytes[j..].starts_with(b"r#") {
        j += 2;
    }
    let name_start = j;
    while j < bytes.len() && is_ident_byte(bytes[j]) {
        j += 1;
    }
    if j == name_start {
        return None;
    }
    balanced_group_end(bytes, skip_ws(j))
}

/// If `bytes[i]` is the `!` of a macro invocation `path ! <delim>…<delim>` (the caller
/// has checked a macro name precedes via [`preceding_macro_name`]), return the index past the
/// balanced body; otherwise `None`. The opening delimiter may follow whitespace.
fn macro_invocation_body_end(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    balanced_group_end(bytes, j)
}

/// Whether the `!` at `bang` is preceded — across optional whitespace — by a **macro name**: a
/// non-keyword identifier, tolerating a raw-identifier prefix (`r#foo ! { … }`). Rust permits
/// whitespace between a macro path and its `!` (`cfg_if ! { … }`), so the name is found by skipping
/// whitespace back to the identifier word. A **keyword** before the `!` (`return !(x)`,
/// `break !{ … }`, `in !(y)`) means the `!` is a unary negation of the following expression/block —
/// not a macro invocation — so that `(…)`/`{…}`/`[…]` is real code (which may contain a governed
/// `use`) and must not be stripped. No preceding identifier (`!x`, a leading `!`) is likewise not an
/// invocation. A raw identifier is always a name (never a keyword), so `r#try ! { … }` strips.
fn preceding_macro_name(bytes: &[u8], bang: usize) -> bool {
    let mut end = bang;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return false; // no identifier word precedes the `!`
    }
    is_raw_ident_prefixed(bytes, start) || !is_rust_keyword(&bytes[start..end])
}

/// Whether `word` is a Rust keyword — a word that, before a `!`, marks a unary negation rather than
/// a macro name (see [`preceding_macro_name`]). Mirrors the 漏刻 audit scanner's own keyword guard;
/// 三儀 ⊥ 三儀 forbids sharing it, so the two scanners keep parallel copies until the deferred
/// judgment-neutral-scanner extraction unifies them. `macro_rules` is deliberately absent (its
/// definition is consumed by [`macro_rules_body_end`] before this is reached).
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

/// Index just past the balanced delimiter group opening at `j` (which must be `{`, `(`,
/// or `[`), or `None` if `j` is not an opening delimiter. Strings and comments are
/// already stripped, so every delimiter is structural and same-delimiter groups nest
/// correctly. An unterminated group (malformed input) ends at end of input, not a panic.
pub(super) fn balanced_group_end(bytes: &[u8], j: usize) -> Option<usize> {
    let (open, close) = match bytes.get(j) {
        Some(b'{') => (b'{', b'}'),
        Some(b'(') => (b'(', b')'),
        Some(b'[') => (b'[', b']'),
        _ => return None,
    };
    let mut depth = 0usize;
    let mut k = j;
    while k < bytes.len() {
        if bytes[k] == open {
            depth += 1;
        } else if bytes[k] == close {
            depth -= 1;
            if depth == 0 {
                return Some(k + 1);
            }
        }
        k += 1;
    }
    Some(bytes.len())
}

/// Remove comments and string literals — line (`//`), block (`/* */`), normal,
/// byte, and C-strings (`"…"`, `b"…"`, `c"…"`, honoring `\"`/`\\`), and raw strings
/// (`r"…"`, `r#"…"#`, `br#"…"#`, `cr#"…"#`, any number of hashes) — so their contents can never be
/// mistaken for a `use` declaration: a `//` or a `use …;` written inside any of them
/// is ignored. Char literals are recognized minimally so a quote-bearing one (`'"'`)
/// does not open a spurious string; a lifetime (`'a`) is emitted as ordinary text.
/// Bare path expressions and macro-generated imports remain out of scope (PROJECT.md).
/// UTF-8 is preserved: kept bytes are decoded once and never split, because every
/// region boundary cut on is ASCII.
pub(super) fn strip_comments_and_strings(source: &str) -> String {
    strip_comments_and_strings_tracked(source).0
}

/// [`strip_comments_and_strings`], additionally returning a same-length position map:
/// `positions[k]` is the byte index in `source` that produced `out[k]`. A synthetic separator
/// (the block-comment case below) has no single source byte, so it is stamped with the position
/// immediately after the comment it replaces — a value real content is never found at, since a
/// caller only looks up a *kept* byte's original position (e.g. an `=` sign) to resolve a `#[path
/// = "…"]` value from the untouched original source, never a separator's.
pub(super) fn strip_comments_and_strings_tracked(source: &str) -> (String, Vec<usize>) {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut positions: Vec<usize> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            // Line comment: drop to end of line.
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            // Block comment: Rust nests these, so track depth and drop through to the
            // `*/` that closes the outermost one — otherwise commented-out code that
            // itself contains a `/* */` would re-expose a `use` after the inner close.
            i += 2;
            let mut depth = 1usize;
            while i + 1 < bytes.len() && depth > 0 {
                if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                    depth += 1;
                    i += 2;
                } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            // Emit a separator so a comment wedged between two tokens does not fuse them: without
            // it, `use/*c*/crate::X;` becomes `usecrate::X;` and the `use` keyword is no longer
            // recognized (its following byte is an identifier byte), silently dropping the import.
            // (A line comment leaves its `\n`, which already separates; `strip_macro_bodies` emits
            // the same separator space for the same reason.)
            out.push(b' ');
            positions.push(i);
        } else if let Some((hashes, quote)) = raw_string_prefix(bytes, i) {
            // Raw string `r#*"…"#*`: no escapes; closed by `"` plus the same number
            // of `#`. Drop the whole literal so its text is never scanned.
            i = quote + 1;
            while i < bytes.len() {
                if bytes[i] == b'"' && raw_closing_matches(bytes, i + 1, hashes) {
                    i += 1 + hashes;
                    break;
                }
                i += 1;
            }
        } else if bytes[i] == b'"' {
            // String (or byte-string) literal: drop it, honoring `\"` and `\\`.
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                i += if bytes[i] == b'\\' { 2 } else { 1 };
            }
            i += 1;
        } else if bytes[i] == b'\'' {
            // A char literal must be skipped whole so a quote it contains (`'"'`)
            // cannot open a spurious string. A lifetime (`'a`) has no closing quote
            // and is emitted as ordinary text.
            if i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                // Escaped char literal (`'\n'`, `'\''`, `'\u{…}'`): skip the opening
                // quote and the backslash, then the escaped character itself (which may
                // be a `'`, as in `'\''`), then scan to the closing quote. Skipping the
                // escaped character first is what keeps `'\''` from ending on its own
                // escaped quote and leaking the real closing quote.
                i += 2;
                if i < bytes.len() {
                    i += 1;
                }
                while i < bytes.len() && bytes[i] != b'\'' {
                    i += 1;
                }
                i += 1;
            } else if i + 2 < bytes.len() && bytes[i + 2] == b'\'' {
                // Simple char literal (`'x'`, `'"'`).
                i += 3;
            } else {
                // A lifetime or stray quote.
                out.push(bytes[i]);
                positions.push(i);
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            positions.push(i);
            i += 1;
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), positions)
}

/// If a raw string literal begins at `i` — `r`, `br`, or `cr` at a token boundary, then any
/// number of `#`, then `"` — return `(hash_count, index_of_opening_quote)`. A leading
/// `r`/`b`/`c` that is the tail of an identifier is not a prefix. The `cr`/`cr#` form is the raw
/// **C-string** literal (stable since Rust 1.79): without recognizing it, the `cr#"…"#` body is
/// scanned as code plus plain strings, and an **odd** number of inner unescaped `"` (raw strings
/// do not escape) leaves a final `"` that opens an unterminated plain string running to EOF,
/// swallowing a following `use` — a false negative. A non-raw `c"…"` / `b"…"` needs no handling
/// here — its `"` opens a plain string with ordinary escaping, which the plain-string branch
/// already strips correctly (the `c`/`b` prefix byte is emitted as harmless code).
fn raw_string_prefix(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    if i > 0 && is_ident_byte(bytes[i - 1]) {
        return None;
    }
    let mut j = i;
    // An optional single byte-string (`b`) or C-string (`c`) prefix before the raw `r` — Rust has
    // no `bc`/`cb` combination, so at most one applies.
    if matches!(bytes.get(j), Some(&b'b') | Some(&b'c')) {
        j += 1;
    }
    if bytes.get(j) != Some(&b'r') {
        return None;
    }
    j += 1;
    let mut hashes = 0;
    while bytes.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if bytes.get(j) == Some(&b'"') {
        Some((hashes, j))
    } else {
        None
    }
}

/// Whether `hashes` `#` characters start at `at` — the closing delimiter that, with
/// the preceding `"`, terminates a raw string opened with the same number of hashes.
fn raw_closing_matches(bytes: &[u8], at: usize, hashes: usize) -> bool {
    (0..hashes).all(|k| bytes.get(at + k) == Some(&b'#'))
}

/// Whether `keyword` appears as a standalone word starting exactly at `i` (bounded by
/// non-identifier bytes on both sides), and is **not** a raw identifier `r#keyword`.
///
/// The raw-identifier guard matters: `#` is not an identifier byte, so a bare
/// "preceding byte is not an ident byte" test would treat the `use` inside `r#use` (a valid raw
/// identifier — e.g. a field `r#use: bool`) as the `use` keyword, and the `use`-walk would then scan
/// to the next `;` and swallow the following real `use` declaration (a false negative that silently
/// disables the import boundary). So a `keyword` immediately preceded by `r#` (with a word boundary
/// before the `r`) is a raw identifier, not the keyword — the same raw-ident handling
/// `macro_rules_body_end` already applies to a macro name.
pub(super) fn keyword_starts_at(bytes: &[u8], i: usize, keyword: &[u8]) -> bool {
    if !bytes[i..].starts_with(keyword) {
        return false;
    }
    let before_ok = !is_raw_ident_prefixed(bytes, i) && (i == 0 || !is_ident_byte(bytes[i - 1]));
    let after = i + keyword.len();
    let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
    before_ok && after_ok
}

/// Whether the word beginning at `pos` is a raw identifier (`r#word`) — i.e. immediately preceded by
/// `r#` with a word boundary before the `r`. The single home of the `r#`-prefix test shared by the
/// keyword boundary check ([`keyword_starts_at`]) and the macro-name check ([`preceding_macro_name`]),
/// so the two cannot drift on the subtle `pos == 2` boundary case.
fn is_raw_ident_prefixed(bytes: &[u8], pos: usize) -> bool {
    pos >= 2
        && bytes[pos - 1] == b'#'
        && bytes[pos - 2] == b'r'
        && (pos == 2 || !is_ident_byte(bytes[pos - 3]))
}

pub(super) fn is_ident_byte(byte: u8) -> bool {
    // Any non-ASCII byte (>= 0x80) is a UTF-8 lead/continuation byte of a Unicode
    // identifier character (Rust allows non-ASCII identifiers, e.g. `use貓`). Treating
    // it as an identifier byte keeps keyword detection (`use`, `mod`) from firing inside
    // a Unicode identifier: `keyword_at("use貓;", …, "use")` must be `None`, since `use貓`
    // is one identifier, not the `use` keyword.
    byte == b'_' || byte.is_ascii_alphanumeric() || byte >= 0x80
}

/// [`strip_macro_bodies`] composed after [`strip_comments_and_strings`] — the pipeline every
/// scanner in this module already runs — with the position map chained all the way back to
/// `source`, so a caller holding a byte index into the returned string can recover exactly which
/// original byte produced it (used to re-read a `#[path = "…"]` value's real quoted text, which
/// cleaning has already dropped by the time a `mod` declaration is found).
pub(super) fn clean_with_positions(source: &str) -> (String, Vec<usize>) {
    let (stripped, positions) = strip_comments_and_strings_tracked(source);
    strip_macro_bodies_tracked(&stripped, &positions)
}

/// Read a `#[path = <value>]` attribute's string value from the **original, untouched** source
/// bytes, starting at `start` (immediately after the `=`) and bounded by `end`. Callers may pass
/// the enclosing item's own position for a tight bound, or (as guibiao's sole caller does) the
/// end of the file — safe either way, because a well-formed string literal's closing quote always
/// arrives long before that, and a malformed/unterminated one correctly yields `None` regardless
/// of how generous `end` is. Skips leading whitespace and comments (an attribute may be written
/// `path /* … */ = /* … */ "…"`),
/// then parses a plain or raw string literal, decoding escapes through [`decode_str_escapes`] —
/// the same set rustc and syn accept — so this matches 渾儀's `syn`-derived value and 漏刻's own
/// `read_path_string` on the same input (three-instrument agreement). Returns `None` for anything
/// that is not a string literal here, or a literal whose escapes do not decode — fail-safe: the
/// caller then treats the module as not directly relocated rather than mis-reading a value.
pub(super) fn read_path_string(bytes: &[u8], start: usize, end: usize) -> Option<String> {
    let mut i = start;
    while i < end {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && matches!(bytes.get(i + 1), Some(&b'/') | Some(&b'*')) {
            if bytes[i + 1] == b'/' {
                while i < end && bytes[i] != b'\n' {
                    i += 1;
                }
            } else {
                i += 2;
                let mut depth = 1usize;
                while i + 1 < end && depth > 0 {
                    if bytes[i] == b'/' && bytes[i + 1] == b'*' {
                        depth += 1;
                        i += 2;
                    } else if bytes[i] == b'*' && bytes[i + 1] == b'/' {
                        depth -= 1;
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
            }
            continue;
        }
        break;
    }
    if bytes.get(i) == Some(&b'r') {
        // Raw string `r#*"…"#*`: no escapes; the closing is `"` then the same `#` count.
        let mut hashes = 0usize;
        let mut j = i + 1;
        while bytes.get(j) == Some(&b'#') {
            hashes += 1;
            j += 1;
        }
        if bytes.get(j) != Some(&b'"') {
            return None;
        }
        j += 1;
        let content_start = j;
        while j < end {
            if bytes[j] == b'"' {
                let mut k = j + 1;
                let mut seen = 0usize;
                while seen < hashes && bytes.get(k) == Some(&b'#') {
                    k += 1;
                    seen += 1;
                }
                if seen == hashes {
                    return String::from_utf8(bytes[content_start..j].to_vec()).ok();
                }
            }
            j += 1;
        }
        return None;
    }
    if bytes.get(i) != Some(&b'"') {
        return None;
    }
    i += 1;
    let content_start = i;
    while i < end {
        match bytes[i] {
            b'"' => return decode_str_escapes(&bytes[content_start..i]),
            // Skip the escaped byte so an escaped quote `\"` (or `\\`) does not end the literal
            // early.
            b'\\' => i += 2,
            _ => i += 1,
        }
    }
    None
}

/// Decode a plain string literal's escapes — the set rustc and syn accept (`\n`/`\r`/`\t`/`\\`/
/// `\0`/`\'`/`\"`/`\xHH`/`\u{…}`) — so a `#[path]` value read from raw source matches what syn
/// would give. An unrecognized escape or a backslash-newline line continuation yields `None`
/// (fail-safe: the caller treats the value as unreadable rather than guessing). Deliberately a
/// standalone copy, not shared with 漏刻's identical decoder — 三儀 ⊥ 三儀, each dimension's lexer
/// stands on its own.
fn decode_str_escapes(inner: &[u8]) -> Option<String> {
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
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                let v = hi * 16 + lo;
                if v > 0x7F {
                    return None;
                }
                out.push(char::from_u32(v)?);
            }
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut value: u32 = 0;
                let mut digits = 0;
                loop {
                    match chars.next()? {
                        '}' => break,
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
            _ => return None,
        }
    }
    Some(out)
}
