//! The token tree: the one reader of a source file's bytes for the inline scan and the import scan.
//!
//! One forward pass lexes the original source — comments dropped, every literal one token, punctuation read by
//! maximal munch — and pairs `(`, `[` and `{` with their closers before any reader runs, so a reader asks its
//! question of whole tokens and paired groups and never of a byte. A macro call — a path's last segment, its `!` and
//! its group, the segments before it being tokens of their own — and an attribute are each one [`Node`], so what
//! stands before an item is read from structure. Positions are token indices.
//!
//! Nothing here recurses, so no nesting depth is refused. The module names no type of the crate it sits in: it
//! reads Rust's lexical grammar, and what the tokens mean is each reader's own question.

use std::borrow::Cow;
use std::cell::OnceCell;
use std::ops::Range;

use unicode_normalization::UnicodeNormalization;

/// The editions whose grammar differs in a way a reader of paths can see: in 2015 `async`, `await`, `dyn` and `try`
/// are identifiers, and a `use` path or a `::`-rooted path starts at the crate root; before 2021 a `c` before a string
/// literal is an identifier of its own, since C string literals arrive in 2021.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Edition {
    /// The edition where `async`, `await`, `dyn` and `try` are identifiers, and a `use` path or a `::`-rooted path
    /// starts at the crate root.
    Rust2015,
    /// The edition where those words are keywords, while a `c` before a string literal is still an identifier of its
    /// own.
    Rust2018,
    /// 2021 and every edition after it.
    Rust2021,
}

impl Edition {
    /// The edition a package's manifest names, as `cargo metadata` reports it: `2015`, `2018`, or any later one.
    pub(crate) fn of(manifest_edition: Option<&str>) -> Self {
        match manifest_edition {
            Some("2015") => Edition::Rust2015,
            Some("2018") => Edition::Rust2018,
            _ => Edition::Rust2021,
        }
    }
}

/// The bracket a group is written with: an opener pairs only a closer of its own delimiter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Delimiter {
    /// `(` and `)`.
    Parenthesis,
    /// `[` and `]`.
    Bracket,
    /// `{` and `}`.
    Brace,
}

/// A token's lexical class, decided once by the lexer; the tree's edition decides a word between [`Kind::Keyword`]
/// and [`Kind::Ident`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Kind {
    /// A word other than `_` that the edition does not reserve, a contextual keyword such as `union` or `macro_rules`
    /// among them.
    Ident,
    /// `r#name`; its [`TokenTree::text`] is `name`.
    RawIdent,
    /// A word [`is_keyword`] reserves in the tree's edition, `self`, `super`, `crate` and `Self` among them.
    Keyword,
    /// A lifetime or a label: `'a`, `'static`.
    Lifetime,
    /// Any literal: string, raw string, byte or C string, character, byte character, number.
    Literal,
    /// Punctuation read by maximal munch over [`MULTI_BYTE_PUNCTUATION`]; also `_`, a `'` that opens no lifetime or
    /// literal, and a closer that pairs no opener.
    Punct,
    /// An opener; [`TokenTree::partner`] names its closer.
    Open(Delimiter),
    /// A closer paired with its opener. One the source never wrote, closing an opener left unclosed, stands past the
    /// last written token with an empty span.
    Close(Delimiter),
    /// Past the last token: what [`TokenTree::kind`] answers there, so a reader stepping off the end of a cut-off file
    /// reads a token that is none of the others. No token has it.
    End,
}

/// One lexed token: what it is, and the bytes of the source it was read from.
#[derive(Clone, Debug)]
pub(super) struct Token {
    /// What the token is. Once the tree is built, every [`Kind::Open`] and [`Kind::Close`] token is paired with its
    /// partner.
    pub kind: Kind,
    /// The bytes of the source the token was read from. A closer the source never wrote, pairing an unclosed
    /// opener at the end of the text, has an empty span there.
    pub span: Range<usize>,
}

/// What a reader steps over: one token, a paired group, a macro call or an attribute, each named by the token
/// indices that bound it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Node {
    /// A token that starts none of the other nodes, such as a `#` with no `[` after it or a name with no `!` and group.
    Token(usize),
    /// A `(…)`, `[…]` or `{…}` group, from its opener to its closer.
    Group {
        /// The opener.
        open: usize,
        /// The closer paired with `open`.
        close: usize,
    },
    /// `name ! group`, or `macro_rules ! name group`, with `name` the first token of the node.
    Macro {
        /// The node's first token: the macro's name, or `macro_rules`.
        name: usize,
        /// The `!` straight after `name`.
        bang: usize,
        /// The opener of the group the macro is called with, past the defined name for `macro_rules`.
        open: usize,
        /// The closer paired with `open`, the node's last token.
        close: usize,
    },
    /// `# [ … ]` or `# ! [ … ]`.
    Attribute {
        /// The `#`.
        hash: usize,
        /// The `[`, past the `!` of an inner attribute.
        open: usize,
        /// The `]` paired with `open`.
        close: usize,
    },
}

impl Node {
    /// The first token of the node.
    pub(super) fn first(self) -> usize {
        match self {
            Node::Token(i) => i,
            Node::Group { open, .. } => open,
            Node::Macro { name, .. } => name,
            Node::Attribute { hash, .. } => hash,
        }
    }

    /// The last token of the node.
    pub(super) fn last(self) -> usize {
        match self {
            Node::Token(i) => i,
            Node::Group { close, .. }
            | Node::Macro { close, .. }
            | Node::Attribute { close, .. } => close,
        }
    }
}

/// One source file as tokens, each `(`, `[` and `{` paired with its closer: a closer no opener pairs reads as
/// [`Kind::Punct`], and an opener the source leaves unclosed is closed past its last token.
pub(super) struct TokenTree<'s> {
    /// The source the tokens' spans index: as written, or with its identifiers in NFC ([`identifiers_in_nfc`]).
    source: Cow<'s, str>,
    /// Every token in source order, with a closer for each opener the source left unclosed appended after the last;
    /// a token's index here is its position in every question the tree answers.
    tokens: Vec<Token>,
    /// For an opener, its closer; for a closer, its opener; for any other token, itself.
    partner: Vec<usize>,
    /// The opener of the innermost group holding each token, `None` at the top level.
    parent: Vec<Option<usize>>,
    /// What the reader of `<…>` groups found, by token, once it has read the tree: this module pairs only `(`, `[`
    /// and `{`, and holds the slot so the tree is paired once rather than once per question.
    angles: OnceCell<Vec<AngleSlot>>,
}

/// One token's entry in the `<…>` pairing a reader fills once per tree: for a token opening such a group, the token
/// closing it and, for a `<<`, the one closing its inner group; for a `(`, `[` or `{`, whether a `<…>` group of its
/// level holds it. What the entries mean is that reader's question; this module only keeps them.
#[derive(Clone, Copy, Default)]
pub(super) struct AngleSlot {
    /// For a `<` or `<<` that opens a group, the token closing it; `None` for every other token, and for an opener a
    /// `;` or the end of its level reaches first.
    pub(super) close: Option<usize>,
    /// For a `<<` that opens a group, the token closing the inner group its second `<` opens — the same token as
    /// `close` where one `>>` closes both.
    pub(super) inner_close: Option<usize>,
    /// For a `(`, `[` or `{`, whether a `<…>` group among its sibling nodes spans it; `false` for every other token.
    pub(super) enclosed: bool,
}

/// Every word a keyword table names in some edition: the strict and reserved keywords of the Rust Reference's
/// *Keywords* chapter. `macro_rules`, `union`, `auto`, `default`, `safe` and `raw` are contextual and read as
/// identifiers.
const RESERVED_WORDS: [&str; 52] = [
    "as", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn", "for",
    "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "pub", "ref", "return",
    "self", "Self", "static", "struct", "super", "trait", "true", "type", "unsafe", "use", "where",
    "while", "async", "await", "dyn", "abstract", "become", "box", "do", "final", "macro",
    "override", "priv", "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// Whether `word` is any edition's keyword: the one table [`is_keyword`] reads.
fn is_reserved_word(word: &str) -> bool {
    RESERVED_WORDS.contains(&word)
}

/// Whether `word`, written without `r#`, is a keyword of `edition`. `gen`, reserved from 2024, is read as an
/// identifier in every edition: a bare `gen` does not compile where it is reserved, so no answer depends on it.
pub(super) fn is_keyword(word: &str, edition: Edition) -> bool {
    is_reserved_word(word)
        && word != "gen"
        && !(edition == Edition::Rust2015 && matches!(word, "async" | "await" | "dyn" | "try"))
}

/// Every multi-byte punctuation token, longest first, so the first member a position starts with is the token
/// maximal munch reads there. Copied from the Rust Reference's *Tokens → Punctuation* table
/// (<https://doc.rust-lang.org/reference/tokens.html#punctuation>, read 2026-09-29; the page names no version), and
/// measured against rustc 1.96.0, which reads each member as one token and no other join of two or three
/// punctuation bytes as one: `macro_rules! one { ($t:tt) => {}; }` accepts `one!(…)` for exactly these. The test
/// module holds the table as copied and compares the two both ways.
pub(super) const MULTI_BYTE_PUNCTUATION: [&str; 25] = [
    "...", "..=", "<<=", ">>=", "!=", "%=", "&&", "&=", "*=", "+=", "-=", "->", "..", "/=", "::",
    "<-", "<<", "<=", "==", "=>", ">=", ">>", "^=", "|=", "||",
];

/// Where the source's tokens start: past a UTF-8 byte-order mark, which rustc strips first, and then past a shebang
/// line. rustc 1.96.0 builds a crate root opening with the mark above `pub mod m { … }`.
fn source_start(bytes: &[u8]) -> usize {
    let bom = if bytes.starts_with(&[0xEF, 0xBB, 0xBF]) {
        3
    } else {
        0
    };
    bom + shebang_end(&bytes[bom..])
}

/// Where a source with no byte-order mark starts its tokens: past a shebang line, which a file opening with `#!` holds
/// unless the first token after the `#!`, past whitespace and comments — a block comment read with the lexer's own
/// nesting — is a `[`: that is an inner attribute, `#![…]`. The Reference states the rule under *Input format →
/// Shebang removal*, and rustc 1.96.0 builds `#!/usr/bin/env run` above `pub mod core;` with the module declared.
fn shebang_end(bytes: &[u8]) -> usize {
    if !bytes.starts_with(b"#!") {
        return 0;
    }
    let mut k = 2;
    loop {
        match bytes.get(k) {
            Some(_) if white_space_len(bytes, k) > 0 => k += white_space_len(bytes, k),
            Some(b'/') if bytes.get(k + 1) == Some(&b'/') => {
                while k < bytes.len() && bytes[k] != b'\n' {
                    k += 1;
                }
            }
            Some(b'/') if bytes.get(k + 1) == Some(&b'*') => k = block_comment_end(bytes, k),
            Some(b'[') => return 0,
            _ => break,
        }
    }
    bytes
        .iter()
        .position(|&b| b == b'\n')
        .map_or(bytes.len(), |at| at + 1)
}

/// The length of the whitespace character at `bytes[i]`, or `0` where none stands there: the Reference's
/// *Whitespace* is Unicode's `Pattern_White_Space`, which holds a vertical tab `u8::is_ascii_whitespace` does not and
/// five characters past ASCII. Measured against rustc 1.96.0, edition 2021: `use crate::forbidden::{\u{b}Thing};`
/// compiles, and so does each of the others in that place.
pub(super) fn white_space_len(bytes: &[u8], i: usize) -> usize {
    const WIDE: [&[u8]; 5] = [
        "\u{85}".as_bytes(),
        "\u{200e}".as_bytes(),
        "\u{200f}".as_bytes(),
        "\u{2028}".as_bytes(),
        "\u{2029}".as_bytes(),
    ];
    match bytes.get(i) {
        Some(b'\t' | b'\n' | b'\x0b' | b'\x0c' | b'\r' | b' ') => 1,
        Some(_) => WIDE
            .iter()
            .find(|wide| bytes[i..].starts_with(wide))
            .map_or(0, |wide| wide.len()),
        None => 0,
    }
}

/// Whether `byte` continues an identifier. Any byte at or above 0x80 does, so a Unicode identifier is one word;
/// [`word_end`] stops at a whitespace character past ASCII, which is no part of a word.
pub(super) fn is_ident_byte(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphanumeric() || byte >= 0x80
}

/// Whether `byte` may begin an identifier: what [`is_ident_byte`] accepts, less the ASCII digits.
fn is_ident_start(byte: u8) -> bool {
    byte == b'_' || byte.is_ascii_alphabetic() || byte >= 0x80
}

impl<'s> TokenTree<'s> {
    /// Lex `source` in `edition`. Never fails: input rustc refuses gets an answer, never a panic.
    pub(super) fn lex(source: &'s str, edition: Edition) -> Self {
        let bytes = source.as_bytes();
        let mut tokens: Vec<Token> = Vec::new();
        let mut i = source_start(bytes);
        while i < bytes.len() {
            let b = bytes[i];
            let space = white_space_len(bytes, i);
            if space > 0 {
                i += space;
                continue;
            }
            if b == b'/' && bytes.get(i + 1) == Some(&b'/') {
                while i < bytes.len() && bytes[i] != b'\n' {
                    i += 1;
                }
                continue;
            }
            if b == b'/' && bytes.get(i + 1) == Some(&b'*') {
                i = block_comment_end(bytes, i);
                continue;
            }
            let (kind, end) = if let Some(end) = literal_end(bytes, i, edition) {
                (Kind::Literal, end)
            } else if b == b'r'
                && bytes.get(i + 1) == Some(&b'#')
                && bytes.get(i + 2).is_some_and(|&c| is_ident_start(c))
            {
                (Kind::RawIdent, word_end(bytes, i + 2))
            } else if b == b'\'' {
                match lifetime_end(bytes, i) {
                    Some(end) => (Kind::Lifetime, end),
                    None => (Kind::Punct, i + 1),
                }
            } else if b.is_ascii_digit() {
                (Kind::Literal, number_end(bytes, i))
            } else if is_ident_start(b) {
                let end = word_end(bytes, i);
                let word = &source[i..end];
                let kind = if word == "_" {
                    Kind::Punct
                } else if is_keyword(word, edition) {
                    Kind::Keyword
                } else {
                    Kind::Ident
                };
                (kind, end)
            } else {
                let kind = match b {
                    b'(' => Kind::Open(Delimiter::Parenthesis),
                    b'[' => Kind::Open(Delimiter::Bracket),
                    b'{' => Kind::Open(Delimiter::Brace),
                    b')' => Kind::Close(Delimiter::Parenthesis),
                    b']' => Kind::Close(Delimiter::Bracket),
                    b'}' => Kind::Close(Delimiter::Brace),
                    _ => Kind::Punct,
                };
                let len = if kind == Kind::Punct {
                    MULTI_BYTE_PUNCTUATION
                        .iter()
                        .find(|p| bytes[i..].starts_with(p.as_bytes()))
                        .map_or(1, |p| p.len())
                } else {
                    1
                };
                (kind, i + len)
            };
            tokens.push(Token {
                kind,
                span: i..end.min(bytes.len()),
            });
            i = end.max(i + 1);
        }

        let mut partner: Vec<usize> = (0..tokens.len()).collect();
        let mut stack: Vec<(usize, Delimiter)> = Vec::new();
        for k in 0..tokens.len() {
            match tokens[k].kind {
                Kind::Open(d) => stack.push((k, d)),
                Kind::Close(d) => match stack.last() {
                    Some(&(open, top)) if top == d => {
                        stack.pop();
                        partner[open] = k;
                        partner[k] = open;
                    }
                    _ => tokens[k].kind = Kind::Punct,
                },
                _ => {}
            }
        }
        while let Some((open, d)) = stack.pop() {
            let k = tokens.len();
            tokens.push(Token {
                kind: Kind::Close(d),
                span: bytes.len()..bytes.len(),
            });
            partner.push(open);
            partner[open] = k;
        }
        let mut parent = vec![None; tokens.len()];
        let mut open_stack: Vec<usize> = Vec::new();
        for k in 0..tokens.len() {
            if let Kind::Close(_) = tokens[k].kind {
                open_stack.pop();
            }
            parent[k] = open_stack.last().copied();
            if let Kind::Open(_) = tokens[k].kind {
                open_stack.push(k);
            }
        }
        TokenTree {
            source: identifiers_in_nfc(source, &mut tokens),
            tokens,
            partner,
            parent,
            angles: OnceCell::new(),
        }
    }

    /// The `<…>` pairing of this tree, read by `read` the first time it is asked for and kept.
    pub(super) fn angle_slots(&self, read: impl FnOnce() -> Vec<AngleSlot>) -> &[AngleSlot] {
        self.angles.get_or_init(read)
    }

    /// How many tokens the tree holds, the closers appended for unclosed openers included; [`TokenTree::kind`] answers
    /// [`Kind::End`] from this index on.
    pub(super) fn len(&self) -> usize {
        self.tokens.len()
    }

    /// The token's kind, or [`Kind::End`] past the last token.
    pub(super) fn kind(&self, i: usize) -> Kind {
        self.tokens.get(i).map_or(Kind::End, |token| token.kind)
    }

    /// The token's text; a raw identifier's without its `r#`.
    pub(super) fn text(&self, i: usize) -> &str {
        let span = self.tokens[i].span.clone();
        let text = &self.source[span];
        if self.tokens[i].kind == Kind::RawIdent {
            &text[2..]
        } else {
            text
        }
    }

    /// The token's text as the source wrote it, a raw identifier's `r#` included.
    pub(super) fn written(&self, i: usize) -> &str {
        &self.source[self.tokens[i].span.clone()]
    }

    /// Whether token `i` exists and is punctuation, a keyword or an identifier spelled `text`.
    pub(super) fn is(&self, i: usize, text: &str) -> bool {
        i < self.tokens.len()
            && matches!(
                self.tokens[i].kind,
                Kind::Punct | Kind::Keyword | Kind::Ident
            )
            && self.text(i) == text
    }

    /// Whether a `macro_rules!` definition starts at token `m`: `macro_rules`, `!` and the macro's name, the one
    /// reading of that header the node reader and the item-header grammar share.
    pub(super) fn opens_a_macro_rules(&self, m: usize) -> bool {
        self.is(m, "macro_rules") && self.is(m + 1, "!") && self.is_word(m + 2)
    }

    /// Whether token `i` exists and is an identifier or a raw identifier.
    pub(super) fn is_word(&self, i: usize) -> bool {
        i < self.tokens.len() && matches!(self.tokens[i].kind, Kind::Ident | Kind::RawIdent)
    }

    /// The byte of the source token `i` starts at.
    #[cfg(test)]
    pub(super) fn start(&self, i: usize) -> usize {
        self.tokens[i].span.start
    }

    /// The value of the string literal at token `i` — a plain `"…"` with its escapes decoded, or a raw `r#"…"#` —
    /// or `None` for any other token, a byte or C string among them, or a plain string whose escapes rustc would
    /// refuse.
    pub(super) fn string_value(&self, i: usize) -> Option<String> {
        if self.tokens.get(i)?.kind != Kind::Literal {
            return None;
        }
        let text = self.written(i);
        if let Some(raw) = text.strip_prefix('r') {
            let hashes = raw.len() - raw.trim_start_matches('#').len();
            let inner = raw[hashes..].strip_prefix('"')?;
            return inner
                .strip_suffix(&format!("\"{}", "#".repeat(hashes)))
                .map(str::to_string);
        }
        let inner = text.strip_prefix('"')?.strip_suffix('"')?;
        decode_str_escapes(inner.as_bytes())
    }

    /// An opener's closer, a closer's opener, any other token itself.
    pub(super) fn partner(&self, i: usize) -> usize {
        self.partner[i]
    }

    /// The opener of the innermost group holding token `i`.
    pub(super) fn enclosing(&self, i: usize) -> Option<usize> {
        self.parent[i]
    }

    /// The node starting at token `i`.
    pub(super) fn node_at(&self, i: usize) -> Node {
        match self.kind(i) {
            Kind::Open(_) => Node::Group {
                open: i,
                close: self.partner[i],
            },
            Kind::Punct if self.text(i) == "#" => {
                let open = if self.is(i + 1, "!") { i + 2 } else { i + 1 };
                if self.kind(open) == Kind::Open(Delimiter::Bracket) {
                    Node::Attribute {
                        hash: i,
                        open,
                        close: self.partner[open],
                    }
                } else {
                    Node::Token(i)
                }
            }
            Kind::Ident | Kind::RawIdent => {
                let open = if self.opens_a_macro_rules(i) {
                    i + 3
                } else {
                    i + 2
                };
                if self.is(i + 1, "!")
                    && open < self.len()
                    && matches!(self.kind(open), Kind::Open(_))
                {
                    Node::Macro {
                        name: i,
                        bang: i + 1,
                        open,
                        close: self.partner[open],
                    }
                } else {
                    Node::Token(i)
                }
            }
            _ => Node::Token(i),
        }
    }

    /// The sibling node ending just before token `i`, or `None` where `i` starts its group or the text.
    pub(super) fn node_before(&self, i: usize) -> Option<Node> {
        if i == 0 {
            return None;
        }
        let j = i - 1;
        if let Kind::Open(_) = self.kind(j) {
            return None;
        }
        let Kind::Close(_) = self.kind(j) else {
            return Some(Node::Token(j));
        };
        let open = self.partner[j];
        let before = |k: usize| open.checked_sub(k);
        if self.kind(open) == Kind::Open(Delimiter::Bracket) {
            if let Some(h) = before(1).filter(|&h| self.is(h, "#")) {
                return Some(Node::Attribute {
                    hash: h,
                    open,
                    close: j,
                });
            }
            if let Some(h) = before(2).filter(|&h| self.is(h + 1, "!") && self.is(h, "#")) {
                return Some(Node::Attribute {
                    hash: h,
                    open,
                    close: j,
                });
            }
        }
        if let Some(bang) = before(1).filter(|&b| self.is(b, "!")) {
            if let Some(name) = bang.checked_sub(1).filter(|&n| self.is_word(n)) {
                return Some(Node::Macro {
                    name,
                    bang,
                    open,
                    close: j,
                });
            }
        }
        if let Some(m) = before(3).filter(|&m| self.opens_a_macro_rules(m)) {
            return Some(Node::Macro {
                name: m,
                bang: m + 1,
                open,
                close: j,
            });
        }
        Some(Node::Group { open, close: j })
    }
}

/// `source` with every identifier token in Unicode Normalization Form C, and each token's span moved to the text it
/// now covers; everything else, a literal's contents included, is kept byte for byte. rustc normalizes an identifier
/// to NFC before comparing it, so `se` + U+0301 + `cret` and `s` + U+00E9 + `cret` are one name to it and one name to
/// every reader of this tree. Measured against rustc 1.96.0, edition 2021: `pub mod sécret` written precomposed
/// beside a call `crate::se\u{301}cret::go()` written decomposed builds. A source whose identifiers are already in NFC,
/// every ASCII one among them, is kept as it is.
fn identifiers_in_nfc<'s>(source: &'s str, tokens: &mut [Token]) -> Cow<'s, str> {
    let is_identifier = |token: &Token| matches!(token.kind, Kind::Ident | Kind::RawIdent);
    let unnormalized = |token: &Token| {
        is_identifier(token) && {
            let text = &source[token.span.clone()];
            !text.is_ascii() && !unicode_normalization::is_nfc(text)
        }
    };
    if !tokens.iter().any(unnormalized) {
        return Cow::Borrowed(source);
    }
    let mut normalized = String::with_capacity(source.len());
    let mut copied = 0;
    for token in tokens {
        let span = token.span.clone();
        normalized.push_str(&source[copied..span.start]);
        let start = normalized.len();
        if is_identifier(token) {
            normalized.extend(source[span.clone()].nfc());
        } else {
            normalized.push_str(&source[span.clone()]);
        }
        token.span = start..normalized.len();
        copied = span.end;
    }
    normalized.push_str(&source[copied..]);
    Cow::Owned(normalized)
}

/// Decode a plain string literal's escapes — the set rustc and syn accept (`\n`/`\r`/`\t`/`\\`/
/// `\0`/`\'`/`\"`/`\xHH`/`\u{…}`/backslash-newline line continuation) — so a `#[path]` value read
/// from source matches what syn would give. An unrecognized escape yields `None` (fail-safe: the
/// caller treats the value as unreadable rather than guessing). A standalone copy, not shared with
/// 漏刻's identical decoder — 三儀 ⊥ 三儀, each dimension's lexer stands on its own.
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

/// The byte length of the UTF-8 character `lead` begins, read from its high bits; a continuation byte or one no
/// character begins with counts as one.
fn utf8_len(lead: u8) -> usize {
    match lead {
        0xC0..=0xDF => 2,
        0xE0..=0xEF => 3,
        0xF0..=0xF7 => 4,
        _ => 1,
    }
}

/// Past the run of bytes [`is_ident_byte`] accepts starting at `i`, stopping before a whitespace character past
/// ASCII; a number's digits and suffix are read as such a run too.
fn word_end(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && is_ident_byte(bytes[i]) && white_space_len(bytes, i) == 0 {
        i += 1;
    }
    i
}

/// Past a nested `/* … */`; an unterminated comment runs to the end of the text.
fn block_comment_end(bytes: &[u8], mut i: usize) -> usize {
    i += 2;
    let mut depth = 1usize;
    while i < bytes.len() && depth > 0 {
        if bytes[i] == b'/' && bytes.get(i + 1) == Some(&b'*') {
            depth += 1;
            i += 2;
        } else if bytes[i] == b'*' && bytes.get(i + 1) == Some(&b'/') {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    i.min(bytes.len())
}

/// Past the string or character literal starting at `i` — raw, byte and, from 2021, C forms included — or `None`.
/// Before 2021 a `c` is an identifier, so `cr#"x"` there is `cr`, `#` and the string `"x"`: measured against rustc
/// 1.96.0, `m!(cr#"x"); …; m!("#");` compiles in edition 2018 with the code between read as code, and is refused in
/// edition 2021.
fn literal_end(bytes: &[u8], i: usize, edition: Edition) -> Option<usize> {
    let mut j = i;
    if bytes.get(j) == Some(&b'b') || (bytes.get(j) == Some(&b'c') && edition == Edition::Rust2021)
    {
        j += 1;
    }
    if bytes.get(j) == Some(&b'r') {
        let mut k = j + 1;
        let mut hashes = 0;
        while bytes.get(k) == Some(&b'#') {
            hashes += 1;
            k += 1;
        }
        if bytes.get(k) == Some(&b'"') {
            let mut m = k + 1;
            while m < bytes.len() {
                if bytes[m] == b'"' && (0..hashes).all(|h| bytes.get(m + 1 + h) == Some(&b'#')) {
                    return Some(m + 1 + hashes);
                }
                m += 1;
            }
            return Some(bytes.len());
        }
    }
    match bytes.get(j) {
        Some(b'"') => {
            let mut m = j + 1;
            while m < bytes.len() && bytes[m] != b'"' {
                m += if bytes[m] == b'\\' { 2 } else { 1 };
            }
            Some((m + 1).min(bytes.len()))
        }
        Some(b'\'') if j == i || bytes[i] == b'b' => char_literal_end(bytes, j),
        _ => None,
    }
}

/// Past the character literal whose quote is at `q`, or `None` where the quote opens a lifetime.
fn char_literal_end(bytes: &[u8], q: usize) -> Option<usize> {
    if bytes.get(q + 1) == Some(&b'\\') {
        let mut m = q + 3;
        while m < bytes.len() && bytes[m] != b'\'' {
            m += 1;
        }
        return Some((m + 1).min(bytes.len()));
    }
    let lead = *bytes.get(q + 1)?;
    let len = utf8_len(lead);
    (bytes.get(q + 1 + len) == Some(&b'\'')).then_some(q + 2 + len)
}

/// Past the lifetime or label whose quote is at `q`, or `None` for a stray quote.
fn lifetime_end(bytes: &[u8], q: usize) -> Option<usize> {
    let start = if bytes.get(q + 1) == Some(&b'r') && bytes.get(q + 2) == Some(&b'#') {
        q + 3
    } else {
        q + 1
    };
    bytes
        .get(start)
        .is_some_and(|&b| is_ident_start(b))
        .then(|| word_end(bytes, start))
}

/// Past the numeric literal starting at `i`: digits, a suffix, one `.` and the digits after it, and a signed exponent
/// in a decimal literal. A `.` belongs to the literal only straight after its digits, and only where no other `.`,
/// `_` or identifier follows it, as rustc reads it: `1.` and `1.0` are literals, while `1..2` is three tokens,
/// `1.max(2)` a literal and a `.`, and `1u8.` a literal with a suffix before a `.`.
fn number_end(bytes: &[u8], i: usize) -> usize {
    let prefixed = bytes[i] == b'0' && matches!(bytes.get(i + 1), Some(b'x' | b'o' | b'b'));
    let mut j = word_end(bytes, i);
    let exponent_sign = |j: usize| {
        !prefixed
            && matches!(bytes[j - 1], b'e' | b'E')
            && matches!(bytes.get(j), Some(b'+' | b'-'))
            && bytes.get(j + 1).is_some_and(u8::is_ascii_digit)
    };
    if exponent_sign(j) {
        j = word_end(bytes, j + 1);
    }
    let digits_only = bytes[i..j].iter().all(|&b| b.is_ascii_digit() || b == b'_');
    if !prefixed
        && digits_only
        && bytes.get(j) == Some(&b'.')
        && bytes
            .get(j + 1)
            .is_none_or(|&next| next != b'.' && !is_ident_start(next))
    {
        j = if bytes.get(j + 1).is_some_and(u8::is_ascii_digit) {
            word_end(bytes, j + 1)
        } else {
            j + 1
        };
        if exponent_sign(j) {
            j = word_end(bytes, j + 1);
        }
    }
    j
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(source: &str) -> Vec<(Kind, String)> {
        let tree = TokenTree::lex(source, Edition::Rust2021);
        (0..tree.len())
            .map(|i| (tree.kind(i), tree.written(i).to_string()))
            .collect()
    }

    /// An identifier is read in NFC, as rustc compares one, and the spans after it follow the text it now holds, while a
    /// literal keeps the bytes it was written with.
    #[test]
    fn an_identifier_is_read_in_nfc_and_a_literal_as_written() {
        let decomposed = "crate::se\u{301}cret::go(\"se\u{301}\"); r#se\u{301}cret";
        assert_eq!(
            kinds(decomposed),
            vec![
                (Kind::Keyword, "crate".into()),
                (Kind::Punct, "::".into()),
                (Kind::Ident, "s\u{e9}cret".into()),
                (Kind::Punct, "::".into()),
                (Kind::Ident, "go".into()),
                (Kind::Open(Delimiter::Parenthesis), "(".into()),
                (Kind::Literal, "\"se\u{301}\"".into()),
                (Kind::Close(Delimiter::Parenthesis), ")".into()),
                (Kind::Punct, ";".into()),
                (Kind::RawIdent, "r#s\u{e9}cret".into()),
            ]
        );
    }

    /// A shebang line and a leading byte-order mark are no token, while `#!` followed — past whitespace and comments,
    /// nested ones included — by `[` opens an inner attribute and is lexed.
    #[test]
    fn a_shebang_line_is_no_token_and_an_inner_attribute_is_one() {
        let tree = TokenTree::lex("#!/usr/bin/env run\nmod core;", Edition::Rust2018);
        assert_eq!(tree.text(0), "mod", "the shebang line is skipped");
        let tree = TokenTree::lex("\u{feff}mod core;", Edition::Rust2018);
        assert_eq!(
            tree.text(0),
            "mod",
            "a byte-order mark is no part of the first token"
        );
        for source in [
            "#![allow(dead_code)]\nmod core;",
            "#! /* note */ [allow(dead_code)]\nmod core;",
            "#! /* /* nested */ */ [allow(dead_code)] mod core;",
        ] {
            let tree = TokenTree::lex(source, Edition::Rust2018);
            assert_eq!(
                tree.text(0),
                "#",
                "{source:?} opens with an inner attribute, no shebang"
            );
        }
    }

    /// The Reference's multi-byte punctuation table as copied, compared with [`MULTI_BYTE_PUNCTUATION`] both ways, and
    /// its order held to longest first so the first member a position starts with is the maximal munch.
    #[test]
    fn the_punctuation_set_is_the_references_both_ways() {
        let reference = [
            "+=", "-=", "*=", "/=", "%=", "^=", "&=", "|=", "<<=", ">>=", "==", "!=", ">=", "<=",
            "&&", "||", "<<", ">>", "..", "...", "..=", "::", "->", "=>", "<-",
        ];
        for token in reference {
            assert!(MULTI_BYTE_PUNCTUATION.contains(&token), "{token} missing");
        }
        for token in MULTI_BYTE_PUNCTUATION {
            assert!(
                reference.contains(&token),
                "{token} is not in the Reference's table"
            );
        }
        for (i, a) in MULTI_BYTE_PUNCTUATION.iter().enumerate() {
            for b in &MULTI_BYTE_PUNCTUATION[i + 1..] {
                assert!(
                    !(b.starts_with(a) && b.len() > a.len()),
                    "{b} is shadowed by {a}"
                );
            }
        }
    }

    #[test]
    fn maximal_munch_reads_each_operator_whole() {
        assert_eq!(
            kinds("a<<=b..=c->d"),
            [
                (Kind::Ident, "a".into()),
                (Kind::Punct, "<<=".into()),
                (Kind::Ident, "b".into()),
                (Kind::Punct, "..=".into()),
                (Kind::Ident, "c".into()),
                (Kind::Punct, "->".into()),
                (Kind::Ident, "d".into()),
            ]
        );
    }

    #[test]
    fn a_char_literal_is_one_literal_token() {
        for source in [
            "'a'",
            "'\\n'",
            "'\\u{1F600}'",
            "'未'",
            "b'x'",
            "'\"'",
            "'}'",
        ] {
            assert_eq!(
                kinds(source),
                [(Kind::Literal, source.to_string())],
                "{source}"
            );
        }
        assert_eq!(
            kinds("'a 'static"),
            [
                (Kind::Lifetime, "'a".into()),
                (Kind::Lifetime, "'static".into())
            ]
        );
    }

    #[test]
    fn every_string_form_is_one_literal_token() {
        for source in [
            "\"a { // \\\" b\"",
            "r\"x\"",
            "r#\"a \" } \"#",
            "br##\"x\"##",
            "cr#\"}\"#",
            "b\"x\"",
            "c\"x\"",
        ] {
            assert_eq!(
                kinds(source),
                [(Kind::Literal, source.to_string())],
                "{source}"
            );
        }
    }

    /// Before edition 2021 a `c` is an identifier, so `c"x"` is two tokens there.
    #[test]
    fn a_c_before_a_string_is_an_identifier_before_2021() {
        let tree = TokenTree::lex("c\"x\"", Edition::Rust2018);
        assert_eq!(
            (0..tree.len()).map(|i| tree.kind(i)).collect::<Vec<_>>(),
            [Kind::Ident, Kind::Literal]
        );
    }

    #[test]
    fn a_number_is_one_literal_token() {
        assert_eq!(kinds("1u8"), [(Kind::Literal, "1u8".into())]);
        assert_eq!(kinds("1.5e-3f64"), [(Kind::Literal, "1.5e-3f64".into())]);
        assert_eq!(kinds("0x1e"), [(Kind::Literal, "0x1e".into())]);
        assert_eq!(
            kinds("1..2"),
            [
                (Kind::Literal, "1".into()),
                (Kind::Punct, "..".into()),
                (Kind::Literal, "2".into())
            ]
        );
        assert_eq!(
            kinds("1.max"),
            [
                (Kind::Literal, "1".into()),
                (Kind::Punct, ".".into()),
                (Kind::Ident, "max".into())
            ]
        );
    }

    #[test]
    fn comments_nest_and_leave_no_token() {
        assert_eq!(
            kinds("a /* b /* c */ d */ e // f\ng"),
            [
                (Kind::Ident, "a".into()),
                (Kind::Ident, "e".into()),
                (Kind::Ident, "g".into()),
            ]
        );
    }

    #[test]
    fn a_keyword_is_read_by_edition() {
        let tree = TokenTree::lex("async dyn r#match gen _", Edition::Rust2015);
        assert_eq!(
            (0..tree.len()).map(|i| tree.kind(i)).collect::<Vec<_>>(),
            [
                Kind::Ident,
                Kind::Ident,
                Kind::RawIdent,
                Kind::Ident,
                Kind::Punct
            ]
        );
        assert_eq!(tree.text(2), "match");
        let tree = TokenTree::lex("async dyn", Edition::Rust2018);
        assert_eq!(tree.kind(0), Kind::Keyword);
        assert_eq!(tree.kind(1), Kind::Keyword);
    }

    #[test]
    fn partners_pair_only_matching_delimiters() {
        let tree = TokenTree::lex("( [ ) ] }", Edition::Rust2018);
        assert_eq!(tree.partner(1), 3, "`[` closes at `]`");
        assert_eq!(
            tree.kind(2),
            Kind::Punct,
            "a `)` inside `[…]` closes nothing"
        );
        assert_eq!(
            tree.kind(4),
            Kind::Punct,
            "a stray closing brace closes nothing"
        );
        assert_eq!(
            tree.kind(tree.partner(0)),
            Kind::Close(Delimiter::Parenthesis)
        );
        assert_eq!(
            tree.partner(0),
            tree.len() - 1,
            "an unclosed `(` closes at the end of the text"
        );
    }

    #[test]
    fn a_mismatched_closer_closes_nothing() {
        let tree = TokenTree::lex("{ ) }", Edition::Rust2018);
        assert_eq!(tree.partner(0), 2);
        assert_eq!(tree.kind(1), Kind::Punct);
    }

    #[test]
    fn a_macro_call_and_an_attribute_are_one_node() {
        let tree = TokenTree::lex(
            "thread_local! { x } #![a] #[b] mod m {} return !(x) macro_rules! m { }",
            Edition::Rust2018,
        );
        assert!(matches!(
            tree.node_at(0),
            Node::Macro {
                name: 0,
                bang: 1,
                open: 2,
                close: 4
            }
        ));
        assert!(matches!(
            tree.node_at(5),
            Node::Attribute {
                hash: 5,
                open: 7,
                ..
            }
        ));
        assert!(matches!(
            tree.node_before(14),
            Some(Node::Attribute { hash: 10, .. })
        ));
        assert!(matches!(
            tree.node_before(10),
            Some(Node::Attribute { hash: 5, .. })
        ));
        assert!(matches!(
            tree.node_before(5),
            Some(Node::Macro { name: 0, .. })
        ));
        let ret = (0..tree.len()).find(|&i| tree.is(i, "return")).unwrap();
        assert!(
            matches!(tree.node_at(ret + 1), Node::Token(_)),
            "a keyword before `!` is a negation"
        );
        let mr = (0..tree.len())
            .find(|&i| tree.is(i, "macro_rules"))
            .unwrap();
        assert!(matches!(tree.node_at(mr), Node::Macro { .. }));
        assert!(
            matches!(tree.node_before(tree.len()), Some(Node::Macro { name, .. }) if name == mr)
        );
    }

    #[test]
    fn odd_input_is_read_without_panicking() {
        for source in [
            "r#\"", "\"", "/*", "'", "r", "b'", "c\"", "0x", "1e+", "'r#", "#", "#!", "a!", "}",
            "(", "",
        ] {
            let tree = TokenTree::lex(source, Edition::Rust2018);
            for i in 0..tree.len() {
                let _ = (tree.node_at(i), tree.node_before(i), tree.text(i));
            }
        }
    }
}
