//! Find line comments (`//`) in Rust source, distinguishing them from `//` inside
//! string literals, raw strings, char literals, and block comments.
//!
//! This is a **finder**, not a stripper: [`crate::comment_scan::line_comments`] returns
//! the positions and content of every `//` comment, rather than removing them. The skip
//! helpers below are copied from 圭表's `module_scan/lexer.rs` (whose copies are
//! `pub(super)` inside `module_scan` and so not reachable from 勘合, which may not depend
//! on 圭表); the two answer different questions through the same lexical rules — that one
//! strips, this one finds. Both recognize the same lexical forms, but this finder slices
//! skipped spans to count physical newlines. Its skip helpers return the first unread
//! byte or the end of input, never a position past it.

/// A line comment found in Rust source.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LineComment {
    /// 1-based line number in the original source.
    pub line: usize,
    /// The comment text, from `//` to end of line (exclusive of the newline).
    pub text: String,
}

/// Every `//` line comment in `source`, with its 1-based line number.
///
/// String literals (`"…"`, `b"…"`, `c"…"`), raw strings (`r"…"`, `r#"…"#`, `br#"…"#`,
/// `cr#"…"#`), char literals (`'x'`, `'\''`), and block comments (`/* … */`, including
/// nested) are skipped: a `//` inside any of them is not a comment. Lifetimes (`'a`)
/// are not char literals and do not suppress a following `//`.
///
/// Doc comments are out of scope: `///` and `//!` are excluded, and the exclusion is
/// **exact** — `////` is a plain comment to rustc (`struct X;` followed by `//// x`
/// compiles, where `/// x` errors `expected item after doc comment`), so a fourth
/// slash does not hide a comment from this sweep.
///
/// Never panics on malformed input — an unterminated literal or block comment runs to
/// end of input, which is the only sound answer when the source does not compile.
pub fn line_comments(source: &str) -> Vec<LineComment> {
    let bytes = source.as_bytes();
    let mut found = Vec::new();
    let mut line = 1usize;
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] == b'\n' {
            line += 1;
            i += 1;
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            // A `//` comment: from here to end of line. Doc comments are `///` exactly
            // (a fourth slash makes it plain) and `//!`.
            let is_doc = (bytes.get(i + 2) == Some(&b'/') && bytes.get(i + 3) != Some(&b'/'))
                || bytes.get(i + 2) == Some(&b'!');
            let start = i;
            while i < bytes.len() && bytes[i] != b'\n' {
                i += 1;
            }
            if !is_doc {
                let text = String::from_utf8_lossy(&bytes[start..i]).into_owned();
                found.push(LineComment { line, text });
            }
        } else if bytes[i] == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'*' {
            i = skip_block_comment(bytes, i, &mut line);
        } else if let Some((hashes, quote)) = raw_string_prefix(bytes, i) {
            i = skip_raw_string(bytes, quote, hashes, &mut line);
        } else if bytes[i] == b'"' {
            i = skip_string_literal(bytes, i, &mut line);
        } else if bytes[i] == b'\'' {
            match skip_char_literal(bytes, i) {
                Some(next) => {
                    line += bytes[i..next].iter().filter(|b| **b == b'\n').count();
                    i = next;
                }
                None => {
                    i += 1;
                }
            }
        } else {
            i += 1;
        }
    }

    found
}

/// Skip a `/* … */` block comment, counting every `\n` the skipped span carries into `line`.
/// The line counter is the caller's because this scanner's output is a position — a comment
/// reported at the wrong line sends a reader to the wrong line, which is the one thing a
/// `{path}:{line}` diagnostic must never do.
fn skip_block_comment(bytes: &[u8], mut i: usize, line: &mut usize) -> usize {
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
            if bytes[i] == b'\n' {
                *line += 1;
            }
            i += 1;
        }
    }
    if depth > 0 {
        *line += bytes[i..].iter().filter(|b| **b == b'\n').count();
        i = bytes.len();
    }
    i
}

/// Skip a raw string body, counting newlines as [`skip_block_comment`] does.
fn skip_raw_string(bytes: &[u8], quote: usize, hashes: usize, line: &mut usize) -> usize {
    let mut i = quote + 1;
    while i < bytes.len() {
        if bytes[i] == b'"' && raw_closing_matches(bytes, i + 1, hashes) {
            i += 1 + hashes;
            break;
        }
        if bytes[i] == b'\n' {
            *line += 1;
        }
        i += 1;
    }
    i
}

/// Skip a `"…"` string (or byte-string) literal, honoring `\"` and `\\`. Every newline
/// in the span is counted: a bare newline inside a string is malformed Rust, but this
/// scanner never panics on malformed input, and the physical line is real either way.
fn skip_string_literal(bytes: &[u8], mut i: usize, line: &mut usize) -> usize {
    i += 1;
    while i < bytes.len() && bytes[i] != b'"' {
        if bytes[i] == b'\\' {
            if bytes.get(i + 1) == Some(&b'\n') {
                *line += 1;
            }
            i = (i + 2).min(bytes.len());
        } else {
            if bytes[i] == b'\n' {
                *line += 1;
            }
            i += 1;
        }
    }
    (i + 1).min(bytes.len())
}

/// The returned cursor is always within `bytes`, including when the closing quote is absent.
fn skip_char_literal(bytes: &[u8], i: usize) -> Option<usize> {
    if i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
        let mut j = i + 2;
        if j < bytes.len() {
            j += 1;
        }
        while j < bytes.len() && bytes[j] != b'\'' {
            j += 1;
        }
        Some((j + 1).min(bytes.len()))
    } else {
        simple_char_literal_scalar_len(bytes, i).map(|len| i + 1 + len + 1)
    }
}

fn simple_char_literal_scalar_len(bytes: &[u8], i: usize) -> Option<usize> {
    let len = utf8_scalar_len(*bytes.get(i + 1)?)?;
    if bytes.get(i + 1 + len) == Some(&b'\'') {
        Some(len)
    } else {
        None
    }
}

fn utf8_scalar_len(lead: u8) -> Option<usize> {
    match lead {
        0x00..=0x7F => Some(1),
        0xC2..=0xDF => Some(2),
        0xE0..=0xEF => Some(3),
        0xF0..=0xF4 => Some(4),
        _ => None,
    }
}

fn raw_string_prefix(bytes: &[u8], i: usize) -> Option<(usize, usize)> {
    if i > 0 && is_ident_byte(bytes[i - 1]) {
        return None;
    }
    let mut j = i;
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

fn raw_closing_matches(bytes: &[u8], at: usize, hashes: usize) -> bool {
    (0..hashes).all(|k| bytes.get(at + k) == Some(&b'#'))
}

fn is_ident_byte(byte: u8) -> bool {
    // Any non-ASCII byte (>= 0x80) is a UTF-8 lead/continuation byte of a Unicode
    // identifier character (Rust allows non-ASCII identifiers, e.g. `貓`). Without this,
    // `raw_string_prefix` after a non-ASCII identifier reads `貓r"x"` as opening a raw
    // string, because the byte before `r` fails the identifier test it should pass.
    byte == b'_' || byte.is_ascii_alphanumeric() || byte >= 0x80
}

#[cfg(test)]
mod tests {
    use super::*;

    fn texts(comments: &[LineComment]) -> Vec<&str> {
        comments.iter().map(|c| c.text.as_str()).collect()
    }

    #[test]
    fn a_bare_line_comment_is_found() {
        let comments = line_comments("// hello\nfn main() {}\n");
        assert_eq!(texts(&comments), ["// hello"]);
        assert_eq!(comments[0].line, 1);
    }

    #[test]
    fn a_doc_comment_is_not_found() {
        let comments = line_comments("/// doc\n//! inner doc\nfn main() {}\n");
        assert!(comments.is_empty());
    }

    #[test]
    fn a_fourth_slash_makes_a_plain_comment() {
        // rustc: `struct X;` followed by `//// x` compiles; `/// x` errors
        // `expected item after doc comment`. A fourth slash is not a doc comment.
        let comments = line_comments("//// plain\n");
        assert_eq!(texts(&comments), ["//// plain"]);
        let comments = line_comments("///// also plain\n");
        assert_eq!(texts(&comments), ["///// also plain"]);
    }

    #[test]
    fn a_comment_after_a_multi_line_block_comment_carries_its_line() {
        let comments = line_comments("/* one\ntwo\nthree */\n// real\n");
        assert_eq!(texts(&comments), ["// real"]);
        assert_eq!(comments[0].line, 4);
    }

    #[test]
    fn a_comment_after_a_multi_line_raw_string_carries_its_line() {
        let comments = line_comments("let s = r#\"\n\n\"#;\n// real\n");
        assert_eq!(texts(&comments), ["// real"]);
        assert_eq!(comments[0].line, 4);
    }

    #[test]
    fn a_comment_after_a_line_continuation_carries_its_line() {
        let comments = line_comments("let s = \"one\\\ntwo\";\n// real\n");
        assert_eq!(texts(&comments), ["// real"]);
        assert_eq!(comments[0].line, 3);
    }

    #[test]
    fn a_comment_inside_a_string_is_not_found() {
        let comments = line_comments(r#"let s = "// not a comment";"#);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_raw_string_is_not_found() {
        let comments = line_comments(r##"let s = r#"// not a comment"#;"##);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_byte_string_is_not_found() {
        let comments = line_comments(r#"let s = b"// not a comment";"#);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_c_string_is_not_found() {
        let comments = line_comments(r#"let s = c"// not a comment";"#);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_raw_byte_string_is_not_found() {
        let comments = line_comments(r##"let s = br#"// not a comment"#;"##);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_raw_c_string_is_not_found() {
        let comments = line_comments(r##"let s = cr#"// not a comment"#;"##);
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_after_a_char_literal_is_found() {
        let comments = line_comments("'x' // real\n");
        assert_eq!(texts(&comments), ["// real"]);
    }

    #[test]
    fn a_quote_char_literal_does_not_swallow_a_comment() {
        let comments = line_comments("'\"' // real\n");
        assert_eq!(texts(&comments), ["// real"]);
    }

    #[test]
    fn an_escaped_quote_char_literal_does_not_swallow_a_comment() {
        let comments = line_comments("'\\'' // real\n");
        assert_eq!(texts(&comments), ["// real"]);
    }

    #[test]
    fn a_lifetime_does_not_suppress_a_comment() {
        let comments = line_comments("&'a str // real\n");
        assert_eq!(texts(&comments), ["// real"]);
    }

    #[test]
    fn a_comment_inside_a_block_comment_is_not_found() {
        let comments = line_comments("/* // not real */\n");
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_inside_a_nested_block_comment_is_not_found() {
        let comments = line_comments("/* outer /* inner // not real */ still outer */\n");
        assert!(comments.is_empty());
    }

    #[test]
    fn a_comment_after_a_block_comment_is_found() {
        let comments = line_comments("/* block */ // real\n");
        assert_eq!(texts(&comments), ["// real"]);
    }

    #[test]
    fn multiple_comments_on_different_lines() {
        let source = "// first\nfn main() {}\n// second\n";
        let comments = line_comments(source);
        assert_eq!(texts(&comments), ["// first", "// second"]);
        assert_eq!(comments[0].line, 1);
        assert_eq!(comments[1].line, 3);
    }

    #[test]
    fn a_trailing_comment_is_found() {
        let comments = line_comments("let x = 1; // trailing\n");
        assert_eq!(texts(&comments), ["// trailing"]);
    }

    #[test]
    fn an_empty_comment_is_found() {
        let comments = line_comments("//\n");
        assert_eq!(texts(&comments), ["//"]);
    }

    #[test]
    fn an_escaped_quote_in_a_string_does_not_end_it() {
        let comments = line_comments(r#"let s = "escaped \" // still in string";"#);
        assert!(comments.is_empty());
    }

    #[test]
    fn an_unterminated_string_runs_to_eof() {
        let comments = line_comments("let s = \"unterminated // not a comment");
        assert!(comments.is_empty());
    }

    #[test]
    fn an_unterminated_block_comment_runs_to_eof() {
        let comments = line_comments("/* unterminated // not a comment");
        assert!(comments.is_empty());
    }

    #[test]
    fn an_unterminated_escaped_char_literal_stays_within_input() {
        let source = "'\\";
        assert!(line_comments(source).is_empty());
        assert_eq!(skip_char_literal(source.as_bytes(), 0), Some(source.len()));
    }

    #[test]
    fn malformed_skip_spans_stop_at_end_of_input() {
        let string = b"\"unterminated\\";
        let block = b"/* unterminated";
        let raw = b"r#\"unterminated";
        let mut line = 1;
        assert_eq!(skip_string_literal(string, 0, &mut line), string.len());
        assert_eq!(skip_block_comment(block, 0, &mut line), block.len());
        assert_eq!(skip_raw_string(raw, 2, 1, &mut line), raw.len());
    }
}
