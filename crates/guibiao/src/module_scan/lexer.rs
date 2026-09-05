//! Lexical hygiene for the source scanner: strip comments, string/char literals, and
//! macro bodies to structural text, and the token-boundary primitives the `use`/`mod`
//! walks stand on (`keyword_starts_at`, `is_ident_byte`). Pure byte processing — no model
//! type, no path, only `std` — feeding the module-graph walk in the parent module.

/// Source reduced to its **declarations**: comments, string/char literals, and macro bodies stripped.
///
/// The one pipeline every declaration reader in this module composes, named once here rather than
/// re-spelled at each call site — `use_scan` twice, `symbol_scan` twice, and `module_check`'s
/// value-namespace query, which is where re-spelling it went wrong: that call passed the RAW file, so a
/// `fn foo` written in a comment, a string, or a macro body read as a declaration.
///
/// Stripping macro bodies is what makes "declared inside a macro body is not observed" a single stated
/// bound rather than a per-reader accident.
pub(crate) fn declaration_text(source: &str) -> String {
    strip_macro_bodies(&strip_comments_and_strings(source))
}

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
            // found across whitespace by `preceding_macro_name`. Transparent control-flow macros
            // (`cfg_if!`) wrap human-authored items without transforming identities; their bodies
            // are preserved so enclosed `use`, `mod`, and call expressions are observed statically.
            if is_transparent_macro_name(bytes, i) {
                out.push(bytes[i]);
                positions.push(input_positions[i]);
                i += 1;
            } else {
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
    let Some((start, word)) = word_before(bytes, bang) else {
        return false;
    };
    is_raw_ident_prefixed(bytes, start) || !is_rust_keyword(word)
}

/// The identifier word immediately before `at`, skipping optional ASCII whitespace.
///
/// Returns its start position as well as the word because callers need the position to distinguish
/// a raw identifier (`r#word`) from the same bare word. The returned slice excludes the `r#`
/// prefix, matching the scanner's canonical identifier vocabulary.
fn word_before(bytes: &[u8], at: usize) -> Option<(usize, &[u8])> {
    let mut end = at;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ident_byte(bytes[start - 1]) {
        start -= 1;
    }
    if start == end {
        return None;
    }
    Some((start, &bytes[start..end]))
}

/// Whether the macro invocation at `bang` is a **transparent control-flow macro** (specifically
/// `cfg_if!`), whose structural contents should be preserved during macro stripping.
fn is_transparent_macro_name(bytes: &[u8], bang: usize) -> bool {
    word_before(bytes, bang).is_some_and(|(_, word)| word == b"cfg_if")
}

/// If `bytes[bang]` is the `!` of a transparent control-flow macro invocation (`cfg_if!`),
/// return `Some((open_delim_pos, close_delim_pos))`; otherwise `None`. `open_delim_pos` is the
/// index of `{`, `(`, or `[`, and `close_delim_pos` is the index just past the matching closing
/// delimiter.
pub(super) fn transparent_macro_body_at(bytes: &[u8], bang: usize) -> Option<(usize, usize)> {
    if bytes.get(bang) != Some(&b'!')
        || !preceding_macro_name(bytes, bang)
        || !is_transparent_macro_name(bytes, bang)
    {
        return None;
    }
    let mut j = bang + 1;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let open_pos = j;
    if open_pos >= bytes.len() || !matches!(bytes[open_pos], b'{' | b'(' | b'[') {
        return None;
    }
    let close_pos = balanced_group_end(bytes, open_pos)?;
    Some((open_pos, close_pos))
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
/// Drop a `//` line comment: to end of line. Pushes nothing — the caller's separator-free
/// join is fine here since the `\n` it stops at already separates neighboring tokens.
fn skip_line_comment(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i] != b'\n' {
        i += 1;
    }
    i
}

/// Drop a `/* … */` block comment. Rust nests these, so track depth and drop through to the
/// `*/` that closes the outermost one — otherwise commented-out code that itself contains a
/// `/* */` would re-expose a `use` after the inner close.
fn skip_block_comment(bytes: &[u8], mut i: usize) -> usize {
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
    if depth > 0 {
        // Unterminated (Rust itself would reject this as a compile error, but observation
        // must still react 0/1/2, never panic or corrupt state): the loop above stops
        // peeking once fewer than two bytes remain, which can leave exactly one trailing
        // byte unconsumed — and that byte may be the orphaned tail of a multi-byte UTF-8
        // character whose lead byte(s) were already dropped inside the (still-open)
        // comment. Left in place, the outer loop would re-scan that lone byte as ordinary
        // code and push it into `out` on its own, an invalid UTF-8 fragment that
        // `String::from_utf8_lossy` below then *lengthens* (one byte becomes the 3-byte
        // U+FFFD), desynchronizing `positions` from the string it maps into and panicking
        // the next stage's indexing. An unterminated comment logically extends to EOF, so
        // consume through EOF rather than leaving anything dangling to be re-read as code.
        i = bytes.len();
    }
    i
}

/// Drop a raw string `r#*"…"#*`: no escapes; closed by `"` plus the same number of `#`.
fn skip_raw_string(bytes: &[u8], quote: usize, hashes: usize) -> usize {
    let mut i = quote + 1;
    while i < bytes.len() {
        if bytes[i] == b'"' && raw_closing_matches(bytes, i + 1, hashes) {
            i += 1 + hashes;
            break;
        }
        i += 1;
    }
    i
}

/// Drop a `"…"` string (or byte-string) literal, honoring `\"` and `\\`.
fn skip_string_literal(bytes: &[u8], mut i: usize) -> usize {
    i += 1;
    while i < bytes.len() && bytes[i] != b'"' {
        i += if bytes[i] == b'\\' { 2 } else { 1 };
    }
    i += 1;
    i
}

/// A char literal must be skipped whole so a quote it contains (`'"'`) cannot open a spurious
/// string. `None` for a lifetime (`'a`) or stray quote, which the caller emits as ordinary text.
fn skip_char_literal(bytes: &[u8], i: usize) -> Option<usize> {
    if i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
        // Escaped char literal (`'\n'`, `'\''`, `'\u{…}'`): skip the opening quote and the
        // backslash, then the escaped character itself (which may be a `'`, as in `'\''`), then
        // scan to the closing quote. Skipping the escaped character first is what keeps `'\''`
        // from ending on its own escaped quote and leaking the real closing quote.
        let mut j = i + 2;
        if j < bytes.len() {
            j += 1;
        }
        while j < bytes.len() && bytes[j] != b'\'' {
            j += 1;
        }
        j += 1;
        Some(j)
    } else {
        // Simple char literal (`'x'`, `'"'`, or a non-ASCII scalar like `'«'`/`'未'`): skip the
        // opening quote, the scalar's full UTF-8 encoding, and the closing quote. Measuring the
        // scalar's real byte length (rather than assuming exactly one, as `'x'` alone would
        // suggest) matters: a multi-byte scalar's closing quote sits further away, and
        // mis-locating it treats the opening quote as a lone stray quote instead — whereupon the
        // scalar's raw bytes leak into the cleaned text as ordinary code, and (if two such
        // literals sit adjacent, as in `['«','{']`) a *later* literal's own closing/opening quote
        // pair can then be misread as a fake 3-byte char literal, silently swallowing the
        // intervening comma and the next literal's opening quote — which unprotects that next
        // literal's payload, leaking a real `{`/`}` byte into the cleaned text as a spurious
        // structural brace. `None` here (no scalar found) means a lifetime or stray quote.
        simple_char_literal_scalar_len(bytes, i).map(|len| i + 1 + len + 1)
    }
}

pub(super) fn strip_comments_and_strings_tracked(source: &str) -> (String, Vec<usize>) {
    let bytes = source.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut positions: Vec<usize> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            i = skip_line_comment(bytes, i);
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i = skip_block_comment(bytes, i);
            // Emit a separator so a comment wedged between two tokens does not fuse them: without
            // it, `use/*c*/crate::X;` becomes `usecrate::X;` and the `use` keyword is no longer
            // recognized (its following byte is an identifier byte), silently dropping the import.
            // (A line comment leaves its `\n`, which already separates; `strip_macro_bodies` emits
            // the same separator space for the same reason.)
            out.push(b' ');
            positions.push(i);
        } else if let Some((hashes, quote)) = raw_string_prefix(bytes, i) {
            i = skip_raw_string(bytes, quote, hashes);
        } else if bytes[i] == b'"' {
            i = skip_string_literal(bytes, i);
        } else if bytes[i] == b'\'' {
            match skip_char_literal(bytes, i) {
                Some(next) => i = next,
                None => {
                    out.push(bytes[i]);
                    positions.push(i);
                    i += 1;
                }
            }
        } else {
            out.push(bytes[i]);
            positions.push(i);
            i += 1;
        }
    }
    (String::from_utf8_lossy(&out).into_owned(), positions)
}

/// If a simple (non-escaped) char literal begins at `i` — an opening `'` the caller has already
/// confirmed, not followed by a backslash — return the byte length of its single scalar value:
/// 1 for an ASCII character, more for a multi-byte UTF-8 scalar (`'«'` is 2, `'未'` is 3), only
/// when the byte immediately after that scalar's full encoding is the closing `'`. `None` for a
/// lifetime (`'a`), a stray quote, or anything malformed — the caller then falls through to
/// treating the `'` as ordinary text, exactly as it already did for those cases.
fn simple_char_literal_scalar_len(bytes: &[u8], i: usize) -> Option<usize> {
    let len = utf8_scalar_len(*bytes.get(i + 1)?)?;
    if bytes.get(i + 1 + len) == Some(&b'\'') {
        Some(len)
    } else {
        None
    }
}

/// The byte length of one UTF-8 scalar value's encoding, read from its lead byte — 1 for ASCII,
/// 2/3/4 for a valid multi-byte lead byte, `None` for a byte that cannot start a scalar (a bare
/// continuation byte, or a lead byte Rust's `char` never encodes to, e.g. an overlong or
/// surrogate-range form). Trusts the lead byte's class rather than validating the full encoding —
/// this only needs to *locate* the scalar's end to find the literal's closing quote, not confirm
/// the source is well-formed UTF-8 (a governed file already is, being real Rust source `rustc`
/// itself would have to accept).
fn utf8_scalar_len(lead: u8) -> Option<usize> {
    match lead {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
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

/// The outcome of scanning what follows a keyword-confirmed `use` at `bytes[i]`: a real
/// `use … ;` statement (its trimmed body, and the cursor just past the `;`), a `use<'a, T>`
/// precise-capturing type bound — not an import — (where to resume ordinary scanning), or an
/// unterminated statement (the caller scans to EOF).
pub(super) enum UseStatementScan {
    Statement { body: String, next: usize },
    NotAStatement { resume_at: usize },
    Unterminated,
}

/// Classify what follows a keyword-confirmed `use` at `bytes[i]` (see [`keyword_starts_at`]).
/// Shared by `use_statements` and `pub_use_statements` (symbol_scan.rs, flat glob detection and the
/// re-export closure's feed) and `use_trees_with_modules` (use_scan.rs's inline-module-aware walk) — the
/// one place all three interpret "what is a `use` statement's body" identically; each still owns its own
/// surrounding loop, since their module/brace and visibility tracking around this scan genuinely differ.
pub(super) fn scan_use_statement(bytes: &[u8], source: &str, i: usize) -> UseStatementScan {
    let start = i + 3;
    // A precise-capturing bound `-> impl Trait + use<'a, T>` (stable Rust) puts a `use` token
    // inside a type bound: it is followed by `<`, whereas a `use` *statement* is always followed
    // by a path (ident / `{` / `*` / `::` / `crate`/`self`/`super`). So a `<` here means this is
    // a bound, not an import — the caller resumes scanning from `start` (letting the `<…>` be
    // walked as ordinary bytes) rather than scanning to the next `;`, which would swallow the
    // following real `use` (a false negative). A comment between `use` and `<` is already
    // removed by the upstream comment/string strip.
    let mut p = start;
    while p < bytes.len() && bytes[p].is_ascii_whitespace() {
        p += 1;
    }
    if bytes.get(p) == Some(&b'<') {
        return UseStatementScan::NotAStatement { resume_at: start };
    }
    match source[start..].find(';') {
        Some(rel) => UseStatementScan::Statement {
            body: source[start..start + rel].trim().to_string(),
            next: start + rel + 1,
        },
        None => UseStatementScan::Unterminated,
    }
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
/// `\0`/`\'`/`\"`/`\xHH`/`\u{…}`/backslash-newline line continuation) — so a `#[path]` value read
/// from raw source matches what syn would give. An unrecognized escape yields `None` (fail-safe:
/// the caller treats the value as unreadable rather than guessing). Deliberately a standalone
/// copy, not shared with 漏刻's identical decoder — 三儀 ⊥ 三儀, each dimension's lexer stands on
/// its own.
fn decode_str_escapes(inner: &[u8]) -> Option<String> {
    let s = std::str::from_utf8(inner).ok()?;
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
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
            // A backslash immediately followed by a newline (`\n`, or `\r\n`) is a line
            // continuation: it and every subsequent leading whitespace character on the
            // continued line are stripped, contributing nothing to the decoded value — verified
            // against a real `rustc` build (`"a\` + newline + indentation + `b"` decodes to
            // `"ab"`). Never a literal `\r`/`\n` push; that would only apply to the plain `r`/`n`
            // letter escapes above.
            '\r' | '\n' => {
                while matches!(chars.peek(), Some(' ' | '\t' | '\n' | '\r')) {
                    chars.next();
                }
            }
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
