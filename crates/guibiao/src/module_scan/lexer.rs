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
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if let Some(end) = macro_rules_body_end(bytes, i) {
            // `macro_rules! name <delim>…<delim>` — drop the name and the body.
            out.push(b' ');
            i = end;
        } else if bytes[i] == b'!' && i > 0 && is_ident_byte(bytes[i - 1]) {
            // A macro invocation `ident! <delim>…<delim>`: keep the `!`, drop the body.
            // The `!` of `macro_rules!` is never reached here — the definition arm above
            // consumes it. `!=` / unary `!expr` are not invocations: the byte after `!`
            // is not an opening delimiter, so `macro_invocation_body_end` returns `None`.
            match macro_invocation_body_end(bytes, i) {
                Some(end) => {
                    out.push(b'!');
                    out.push(b' ');
                    i = end;
                }
                None => {
                    out.push(bytes[i]);
                    i += 1;
                }
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
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

/// If `bytes[i]` is the `!` of a macro invocation `ident! <delim>…<delim>` (the caller
/// has checked an identifier byte immediately precedes), return the index past the
/// balanced body; otherwise `None`. The opening delimiter may follow whitespace.
fn macro_invocation_body_end(bytes: &[u8], i: usize) -> Option<usize> {
    let mut j = i + 1;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    balanced_group_end(bytes, j)
}

/// Index just past the balanced delimiter group opening at `j` (which must be `{`, `(`,
/// or `[`), or `None` if `j` is not an opening delimiter. Strings and comments are
/// already stripped, so every delimiter is structural and same-delimiter groups nest
/// correctly. An unterminated group (malformed input) ends at end of input, not a panic.
fn balanced_group_end(bytes: &[u8], j: usize) -> Option<usize> {
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
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
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
                i += 1;
            }
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8_lossy(&out).into_owned()
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
/// non-identifier bytes on both sides).
pub(super) fn keyword_starts_at(bytes: &[u8], i: usize, keyword: &[u8]) -> bool {
    if !bytes[i..].starts_with(keyword) {
        return false;
    }
    let before_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
    let after = i + keyword.len();
    let after_ok = after >= bytes.len() || !is_ident_byte(bytes[after]);
    before_ok && after_ok
}

pub(super) fn is_ident_byte(byte: u8) -> bool {
    // Any non-ASCII byte (>= 0x80) is a UTF-8 lead/continuation byte of a Unicode
    // identifier character (Rust allows non-ASCII identifiers, e.g. `use貓`). Treating
    // it as an identifier byte keeps keyword detection (`use`, `mod`) from firing inside
    // a Unicode identifier: `keyword_at("use貓;", …, "use")` must be `None`, since `use貓`
    // is one identifier, not the `use` keyword.
    byte == b'_' || byte.is_ascii_alphanumeric() || byte >= 0x80
}
