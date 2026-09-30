//! `use` statements and their trees: the one enumeration of `use` statements over a [`TokenTree`] and the one
//! use-tree parser, which the import scan and the scope tree both read.

use super::item_head::{Visibility, in_macro_body, visibility_before};
use super::path_vocab::canonical_module_path;
use super::token_tree::{Delimiter, Kind, TokenTree};

/// The cap on brace nesting in a `use` tree, so a pathologically nested `use` cannot overflow the parser's stack —
/// a backstop set far beyond any real or lint-clean source, where rustfmt-formatted `use`s nest a handful of levels.
/// The parser owns it, so every reader of one `use` statement is refused at the same depth.
const MAX_USE_TREE_NESTING: usize = 128;

/// One `use … ;` statement: its `use` keyword's token, the visibility its qualifier gives what it binds, and its
/// leaves as the one use-tree parser reads them from its tokens, or the refusal quoting the tree as written.
pub(super) struct UseStatement {
    pub at: usize,
    pub visibility: Visibility,
    pub leaves: Result<Vec<UseLeaf>, String>,
}

/// Every `use … ;` statement in `tree`, in order: the one enumeration every reader of imports in this scanner
/// starts from. A precise-capturing `use<…>` bound is not a statement, a statement inside a macro's group other
/// than `cfg_if!`'s is not read, and one with no `;` at its level is not a statement.
pub(super) fn use_statements(tree: &TokenTree) -> Vec<UseStatement> {
    let mut statements = Vec::new();
    for at in 0..tree.len() {
        if tree.kind(at) != Kind::Keyword
            || tree.text(at) != "use"
            || tree.is(at + 1, "<")
            || in_macro_body(tree, at)
        {
            continue;
        }
        let Some(end) = statement_end(tree, at + 1) else {
            continue;
        };
        let body = render(tree, at + 1, end);
        let mut leaves = Vec::new();
        let read =
            read_tree(tree, at + 1, end, &Path::default(), 0, &mut leaves).map_err(|depth| {
                format!("cannot judge a `use` tree nested past {depth} brace levels: '{body}'")
            });
        statements.push(UseStatement {
            at,
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
    Name { path: String, binds: String },
    /// A glob, by its base path: `a::b::*` and the `*` of `a::b::{*}` are both `a::b`, and a glob with no path before
    /// it — `::*`, `*`, `{*}` — is `::`, which names the crate root in edition 2015, where each of those spellings
    /// reads from it.
    Glob(String),
    /// A `{self}` leaf: the group's prefix module itself, and the name it binds — its `as` alias, or
    /// else the module path's last segment, so `use std::io::{self};` binds `io`.
    SelfLeaf { module: String, binds: String },
}

/// The path a use tree has read so far: its segments as written, and whether it began with `::`.
#[derive(Clone, Default)]
struct Path {
    global: bool,
    segments: Vec<String>,
}

impl Path {
    fn written(&self) -> String {
        let joined = self.segments.join("::");
        if self.global {
            format!("::{joined}")
        } else {
            joined
        }
    }

    fn last(&self) -> Option<&str> {
        self.segments.last().map(String::as_str)
    }
}

/// Read the use tree in tokens `from..to` under `prefix` into `out` — the one use-tree parser every reader of
/// imports in this scanner shares, reading tokens rather than text, so `as _` is an alias and `::*` a glob by
/// what they are. A group recurses once per brace level, and past [`MAX_USE_TREE_NESTING`] levels the tree is
/// refused, answering the cap, rather than read partially: a real, compilable `use` nested that deep would
/// otherwise vanish from observation with no report — the false negative PROJECT.md's core contract forbids.
fn read_tree(
    tree: &TokenTree,
    from: usize,
    to: usize,
    prefix: &Path,
    depth: usize,
    out: &mut Vec<UseLeaf>,
) -> Result<(), usize> {
    if depth > MAX_USE_TREE_NESTING {
        return Err(MAX_USE_TREE_NESTING);
    }
    let mut path = prefix.clone();
    let mut k = from;
    if k < to && tree.is(k, "::") && path.segments.is_empty() {
        path.global = true;
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
            return Ok(());
        }
        path.segments.push(tree.written(k).to_string());
        k += 1;
        if k < to && tree.is(k, "::") {
            k += 1;
            continue;
        }
        let binds = if tree.is(k, "as") && k + 1 < to {
            tree.written(k + 1).to_string()
        } else {
            path.last().unwrap_or_default().to_string()
        };
        let binds = canonical_module_path(&binds);
        if !binds.is_empty() {
            out.push(UseLeaf::Name {
                path: path.written(),
                binds,
            });
        }
        return Ok(());
    }
    Ok(())
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
) -> Result<(), usize> {
    if from >= to {
        return Ok(());
    }
    if tree.is(from, "self") && (from + 1 == to || tree.is(from + 1, "as")) {
        if prefix.segments.is_empty() {
            return Ok(());
        }
        let named = if tree.is(from + 1, "as") && from + 2 < to {
            tree.written(from + 2)
        } else {
            prefix.last().unwrap_or_default()
        };
        out.push(UseLeaf::SelfLeaf {
            module: prefix.written(),
            binds: canonical_module_path(named),
        });
        return Ok(());
    }
    read_tree(tree, from, to, prefix, depth, out)
}
