//! `use` statements and their trees: the one enumeration of `use` statements over a [`TokenTree`] and the one
//! use-tree parser, which the import scan and the scope tree both read.

use super::item_head::{Visibility, in_attribute, in_macro_body, visibility_before};
use super::path_vocab::canonical_module_path;
use super::token_tree::{Delimiter, Kind, TokenTree};

/// The cap on brace nesting in a `use` tree, so a pathologically nested `use` cannot overflow the parser's stack —
/// a backstop set far beyond any real or lint-clean source, where rustfmt-formatted `use`s nest a handful of levels.
/// The parser owns it, so every reader of one `use` statement is refused at the same depth.
const MAX_USE_TREE_NESTING: usize = 128;

/// One `use … ;` statement: its `use` keyword's token, the visibility its qualifier gives what it binds, and its
/// leaves as the one use-tree parser reads them from its tokens, or the refusal quoting the tree as written.
pub(super) struct UseStatement {
    /// The token index of the `use` keyword.
    pub at: usize,
    /// The statement's `;`.
    pub end: usize,
    /// The visibility the qualifier before `use` gives every name the statement binds.
    pub visibility: Visibility,
    /// Every leaf of the tree, or the refusal quoting the tree; a tree is never read in part.
    pub leaves: Result<Vec<UseLeaf>, String>,
}

/// Every `use … ;` statement in `tree`, in order: the one enumeration every reader of imports in this scanner
/// starts from. A precise-capturing `use<…>` bound is not a statement, a statement inside a macro's group other
/// than `cfg_if!`'s is not read, nor one inside an attribute, which is that attribute's input — rustc 1.96.0 compiles
/// `#[cfg_attr(any(), my_attr(use crate::forbidden::Thing;))] fn f() {}` with no such import — and one with no `;`
/// at its level is not a statement.
pub(super) fn use_statements(tree: &TokenTree) -> Vec<UseStatement> {
    statements_where(tree, false)
}

/// Every `use … ;` statement a macro's group other than `cfg_if!`'s holds, read as [`use_statements`] reads the rest:
/// what the macro makes of its tokens is not read, so no import rule reads them, while a reader of mentions reads them
/// as it reads every other token of a macro's group, conservatively.
pub(super) fn macro_use_statements(tree: &TokenTree) -> Vec<UseStatement> {
    statements_where(tree, true)
}

/// The `use` statements standing inside a macro's group where `in_macro` holds, and the rest where it does not.
fn statements_where(tree: &TokenTree, in_macro: bool) -> Vec<UseStatement> {
    let mut statements = Vec::new();
    for at in 0..tree.len() {
        if tree.kind(at) != Kind::Keyword
            || tree.text(at) != "use"
            || tree.is(at + 1, "<")
            || in_macro_body(tree, at) != in_macro
            || in_attribute(tree, at)
        {
            continue;
        }
        let Some(end) = statement_end(tree, at + 1) else {
            continue;
        };
        let body = render(tree, at + 1, end);
        let mut leaves = Vec::new();
        let read = read_tree(tree, at + 1, end, &Path::default(), 0, &mut leaves).map_err(
            |unread| match unread {
                Unread::PastCap(depth) => {
                    format!("cannot judge a `use` tree nested past {depth} brace levels: '{body}'")
                }
                Unread::Token(k) if k < end => format!(
                    "cannot judge a `use` tree holding `{}` where a path segment stands: '{body}'",
                    tree.written(k)
                ),
                Unread::Token(_) => {
                    format!("cannot judge a `use` tree whose path ends in `::`: '{body}'")
                }
            },
        );
        statements.push(UseStatement {
            at,
            end,
            visibility: visibility_before(tree, at),
            leaves: read.map(|()| leaves),
        });
    }
    statements
}

/// The `;` ending the statement whose body starts at `from`, read at the body's level.
fn statement_end(tree: &TokenTree, from: usize) -> Option<usize> {
    let mut k = from;
    while k < tree.len() {
        match tree.kind(k) {
            Kind::Open(_) => k = tree.partner(k) + 1,
            Kind::Close(_) => return None,
            _ if tree.is(k, ";") => return Some(k),
            _ => k += 1,
        }
    }
    None
}

/// Tokens `from..to` as text, for a refusal to quote: two words are separated by a space, and nothing else is.
fn render(tree: &TokenTree, from: usize, to: usize) -> String {
    let word = |k: usize| {
        matches!(
            tree.kind(k),
            Kind::Ident | Kind::RawIdent | Kind::Keyword | Kind::Literal | Kind::Lifetime
        )
    };
    let mut out = String::new();
    for k in from..to {
        if k > from && word(k) && word(k - 1) {
            out.push(' ');
        }
        out.push_str(tree.written(k));
    }
    out
}

/// One leaf of a use tree.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum UseLeaf {
    /// A path the tree imports, and the name it binds: its `as` alias — `_` for `as _`, which binds no name a path
    /// can head — or else its last segment.
    Name {
        /// The imported path, each segment as written, `$crate` read as `crate`, and led by `::` where the tree was.
        path: String,
        /// The bound name, canonical (no `r#`).
        binds: String,
    },
    /// A glob, by its base path: `a::b::*` and the `*` of `a::b::{*}` are both `a::b`, and a glob with no path before
    /// it — `::*`, `*`, `{*}` — is `::`, which names the crate root in edition 2015, where each of those spellings
    /// reads from it.
    Glob(String),
    /// A `{self}` leaf: the group's prefix module itself, and the name it binds — its `as` alias, or
    /// else the module path's last segment, so `use std::io::{self};` binds `io`.
    SelfLeaf {
        /// The group's prefix path, each segment as written, `$crate` read as `crate`, and led by `::` where the tree
        /// was.
        module: String,
        /// The bound name, canonical (no `r#`).
        binds: String,
    },
    /// An empty group, by the path before it: `use crate::a::{};` imports and binds nothing, and still names
    /// `crate::a`, which rustc resolves — `use crate::a::nothere::{};` is refused with E0432 — so a reader of
    /// mentions reads it while a reader of imports passes it over.
    Empty(String),
}

/// The path a use tree has read so far: its segments as written, and whether it began with `::`.
#[derive(Clone, Default)]
struct Path {
    /// Whether the tree began with `::`.
    global: bool,
    /// Each segment as its token is written, `r#` kept.
    segments: Vec<String>,
}

impl Path {
    /// The segments joined by `::`, led by `::` where the tree began with one: the path as written, not canonicalized.
    fn written(&self) -> String {
        let joined = self.segments.join("::");
        if self.global {
            format!("::{joined}")
        } else {
            joined
        }
    }

    /// The last segment as written, or `None` before any segment is read.
    fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }
}

/// Why a use tree is not read: nested past the cap, or holding at `Token`'s index — the end of the statement where a
/// path runs out after `::` — a token no path segment is, which rustc refuses, so the tree is refused rather than a
/// leaf dropped.
#[derive(Debug)]
enum Unread {
    /// Brace nesting went past the cap, which it carries for the refusal to name.
    PastCap(usize),
    /// The index of the token no path segment is; an index at the statement's end means the path ended in `::`.
    Token(usize),
}

/// Read the use tree in tokens `from..to` under `prefix` into `out` — the one use-tree parser every reader of
/// imports in this scanner shares, reading tokens rather than text, so `as _` is an alias and `::*` a glob by
/// what they are. A group recurses once per brace level, and past [`MAX_USE_TREE_NESTING`] levels the tree is
/// refused, answering the cap, rather than read partially: a real, compilable `use` nested that deep would
/// otherwise vanish from observation with no report — the false negative PROJECT.md's core contract forbids. A
/// `$crate` head, which only a `macro_rules!` body can write, is read as `crate`: it names the crate defining the
/// macro, and a macro this scanner reads is this crate's.
fn read_tree(
    tree: &TokenTree,
    from: usize,
    to: usize,
    prefix: &Path,
    depth: usize,
    out: &mut Vec<UseLeaf>,
) -> Result<(), Unread> {
    if depth > MAX_USE_TREE_NESTING {
        return Err(Unread::PastCap(MAX_USE_TREE_NESTING));
    }
    let mut path = prefix.clone();
    let mut k = from;
    if k < to && tree.is(k, "::") && path.segments.is_empty() {
        path.global = true;
        k += 1;
    } else if path.segments.is_empty() && tree.is(k, "$") && k + 1 < to && tree.is(k + 1, "crate") {
        k += 1;
    }
    while k < to {
        if tree.is(k, "*") {
            out.push(UseLeaf::Glob(if path.segments.is_empty() {
                "::".to_string()
            } else {
                path.written()
            }));
            return Ok(());
        }
        if tree.kind(k) == Kind::Open(Delimiter::Brace) {
            let close = tree.partner(k).min(to);
            if close == k + 1 {
                if !path.segments.is_empty() {
                    out.push(UseLeaf::Empty(path.written()));
                }
                return Ok(());
            }
            let mut part = k + 1;
            while part < close {
                let mut end = part;
                while end < close && !tree.is(end, ",") {
                    end = match tree.kind(end) {
                        Kind::Open(_) => tree.partner(end) + 1,
                        _ => end + 1,
                    };
                }
                read_part(tree, part, end.min(close), &path, depth + 1, out)?;
                part = end + 1;
            }
            return Ok(());
        }
        if !matches!(tree.kind(k), Kind::Ident | Kind::RawIdent | Kind::Keyword) {
            return Err(Unread::Token(k));
        }
        let segment = tree.written(k);
        path.segments.push(segment.to_string());
        k += 1;
        if k < to && tree.is(k, "::") {
            k += 1;
            continue;
        }
        let binds = if tree.is(k, "as") && k + 1 < to {
            tree.written(k + 1)
        } else {
            segment
        };
        out.push(UseLeaf::Name {
            path: path.written(),
            binds: canonical_module_path(binds),
        });
        return Ok(());
    }
    Err(Unread::Token(to))
}

/// Read one part of a group, tokens `from..to`, under the group's prefix: a `self` leaf names the prefix module
/// itself, and anything else is a use tree of its own.
fn read_part(
    tree: &TokenTree,
    from: usize,
    to: usize,
    prefix: &Path,
    depth: usize,
    out: &mut Vec<UseLeaf>,
) -> Result<(), Unread> {
    if from >= to {
        return Ok(());
    }
    if tree.is(from, "self") && (from + 1 == to || tree.is(from + 1, "as")) {
        let Some(last) = prefix.last() else {
            return Ok(());
        };
        let named = if tree.is(from + 1, "as") && from + 2 < to {
            tree.written(from + 2)
        } else {
            last
        };
        out.push(UseLeaf::SelfLeaf {
            module: prefix.written(),
            binds: canonical_module_path(named),
        });
        return Ok(());
    }
    read_tree(tree, from, to, prefix, depth, out)
}
