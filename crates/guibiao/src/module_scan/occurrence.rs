//! Path occurrences, read from the [`TokenTree`] by the tokens beside them.
//!
//! An occurrence is a token run — `::`? *head* ( `::` *segment* | `::` `<…>` )* — whose head is an identifier, a
//! raw identifier or one of the path keywords `self`, `Self`, `super` and `crate`. Its role is read from its
//! neighbours alone: the name a `fn` item, a tuple struct or a tuple variant declares is a definition; a run followed
//! by a parenthesized group is a call; anything else is a mention. No expression or pattern grammar is read, so a
//! tuple-struct or tuple-variant pattern — written as a call is written — is read as a call: the declared bound
//! `inline-symbol-path-confinement/a-path-in-a-pattern-position-is-read-as-a-call-a-stated-bound`.
//!
//! `<…>` is counted in two places only: after `::`, where the `<` is always a turbofish, and at a `<` that
//! [`opens_a_qualified_path`] admits. Everywhere else a `<` is punctuation.

use std::collections::BTreeSet;

use super::item_head::{
    angle_group, angle_group_end, attribute_path, cfg_attr_metas, heads_a_path,
    opens_a_qualified_path, opens_an_angle_group, variant_positions,
};
use super::token_tree::{Delimiter, Kind, Node, TokenTree};

/// A path occurrence: the token its run starts at — the head, or the `::` rooting it — its `::`-joined segments,
/// `::`-prefixed where rooted, and whether it is applied as a call.
pub(super) struct Occurrence {
    pub at: usize,
    pub segments: String,
    pub is_call: bool,
}

/// A path run whose head is at `head`.
pub(super) struct PathRun {
    pub segments: Vec<String>,
    /// The index just past the run — past a trailing turbofish too.
    pub end: usize,
    /// The `<` or `<<` opening each of the run's turbofishes, whose contents — types and const arguments — hold
    /// paths of their own.
    pub turbofishes: Vec<usize>,
}

/// The path run whose head is at `head`: `head` ( `::` *segment* | `::` `<…>` )*.
pub(super) fn path_run(tree: &TokenTree, head: usize) -> PathRun {
    let mut segments = vec![tree.text(head).to_string()];
    let mut turbofishes = Vec::new();
    let mut j = head + 1;
    while tree.is(j, "::") {
        if heads_a_path(tree, j + 1) {
            segments.push(tree.text(j + 1).to_string());
            j += 2;
        } else if opens_an_angle_group(tree, j + 1) {
            match angle_group_end(tree, j + 1) {
                Some(close) => {
                    turbofishes.push(j + 1);
                    j = close + 1;
                }
                None => break,
            }
        } else {
            break;
        }
    }
    PathRun {
        segments,
        end: j,
        turbofishes,
    }
}

/// The words before a single identifier that make it a name being introduced rather than a path: an item's or a
/// binding's name, or a `for` loop's pattern. `mut` is not among them, since it also stands in `&mut` and `*mut`
/// before a path; [`binds_after_mut`] reads it apart.
const INTRODUCING_WORDS: [&str; 12] = [
    "fn", "struct", "enum", "union", "trait", "type", "mod", "const", "static", "let", "ref", "for",
];

/// Whether the token at `at` is the `mut` of a binding — `let mut x`, `ref mut x`, `(mut x, …)` — rather than of a
/// reference or a raw pointer, `&mut Clock` or `*mut T`, before which a path is written. No pattern grammar is read,
/// so the binding is told from the reference by the one token before the `mut`.
fn binds_after_mut(tree: &TokenTree, at: usize) -> bool {
    tree.is(at, "mut")
        && !at.checked_sub(1).is_some_and(|before| {
            tree.is(before, "&") || tree.is(before, "&&") || tree.is(before, "*")
        })
}

/// Whether the single identifier at `head`, ending before `end`, is a path mentioned rather than a name introduced: an
/// identifier, not a keyword segment, since a bare `self` is a receiver and not a path to its module; not after
/// [`INTRODUCING_WORDS`] or a binding's `mut`; and not followed by a lone `:`, which makes it a field, a parameter, a
/// binding or a generic parameter being declared or initialized, by `!`, which makes it a macro's name, or by `@`,
/// which binds a pattern. So under a strict confinement `let g: fn() = now;` and `&mut Clock` mention their paths,
/// while `pub now: u8`, `fn f(now: u8)`, `S { now: 1 }`, `const NOW: u8`, `let mut now` and `now!()` do not.
fn names_a_single_segment_path(tree: &TokenTree, head: usize, end: usize) -> bool {
    matches!(tree.kind(head), Kind::Ident | Kind::RawIdent)
        && !head.checked_sub(1).is_some_and(|before| {
            INTRODUCING_WORDS.contains(&tree.text(before)) || binds_after_mut(tree, before)
        })
        && !(end < tree.len() && (tree.is(end, ":") || tree.is(end, "!") || tree.is(end, "@")))
}

/// Every call and path mention in `tree`. An attribute's arguments are read past its own path and a `cfg` or
/// `cfg_attr` predicate, as [`attribute_arguments`] states. The identifier
/// after a `.` is a field or a method, not a path. The tail of a qualified path — `<T>::f`, `<T as Trait>::f` — has
/// no head the scanner can resolve without the type inference it does not perform, so it is left to the
/// receiver-method bound; the paths inside its `<…>` are read as any others are. A turbofish's contents — after a
/// path or after a method — are read the same way: counting its `<…>` says where the path ends, and what it holds
/// waits on a worklist to be read in turn. A `use` statement's paths are no occurrences: each is an import, read by the
/// one reader of use trees, whose leaves a strict confinement judges as `use` paths — a grouped `use crate::{a::b};`
/// holds no path `a::b`, and in edition 2015 a `use` path starts at the crate root where an expression's does not.
/// `statements` spans each such statement, `(use, ;)`, in source order.
pub(super) fn occurrences(tree: &TokenTree, statements: &[(usize, usize)]) -> Vec<Occurrence> {
    let mut scan = Scan {
        tree,
        variants: variant_positions(tree),
        qualified_tails: BTreeSet::new(),
        work: vec![(0, tree.len())],
        out: Vec::new(),
    };
    while let Some((from, to)) = scan.work.pop() {
        scan.read(from, to);
    }
    scan.out.retain(|occurrence| {
        let before = statements.partition_point(|&(at, _)| at <= occurrence.at);
        before == 0 || statements[before - 1].1 < occurrence.at
    });
    scan.out
}

/// What the scan of one tree carries between the ranges it reads.
struct Scan<'t, 's> {
    tree: &'t TokenTree<'s>,
    variants: BTreeSet<usize>,
    /// The `::` beginning each qualified path's tail, which the scan does not read as a path.
    qualified_tails: BTreeSet<usize>,
    /// Ranges waiting to be read: turbofish contents and attribute arguments.
    work: Vec<(usize, usize)>,
    out: Vec<Occurrence>,
}

/// The `::` after the `>` that closes the inner group a `<<` token opens, where the `<<` opens a turbofish or a
/// qualified path and its inner group closes before the outer one and is followed by `::`: `a::<<u8 as Tr>::X>()`
/// and `<<u8 as Tr>::X as Tr2>::Y`. Nothing here checks that the inner group is a qualified path's; the callers read
/// it only where the outer `<` is a path's, never after an operand, where `a << b > ::c()` compares a shift with a
/// rooted call. So a `<<` opening a generic list, `Vec<<u8 as Tr>::md5x>`, leaves its inner tail read as a rooted
/// path — a declared over-reaction (bound:
/// inline-symbol-path-confinement/a-qualified-path-a-shift-opens-in-a-generic-list-is-read-as-a-rooted-path-a-stated-bound).
fn inner_qualified_tail(tree: &TokenTree, shift: usize) -> Option<usize> {
    let group = angle_group(tree, shift)?;
    let inner = group.inner_close?;
    (inner != group.close && tree.is(inner + 1, "::")).then_some(inner + 1)
}

/// The token ranges of the attribute whose `[` is at `open` and `]` at `close` that are read for paths: what follows
/// each attribute's own path — the attribute's name is not a call even where parentheses follow it — except a
/// `cfg`'s predicate, and a `cfg_attr`'s, whose applied metas are attributes read the same way. An attribute macro
/// reads its arguments as its own input, so a call written there is a call. Nested `cfg_attr`s wait on a worklist.
fn attribute_arguments(tree: &TokenTree, open: usize, close: usize) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut metas = vec![(open + 1, close)];
    while let Some((start, end)) = metas.pop() {
        match attribute_path(tree, start, end) {
            (Some("cfg"), _) => {}
            (Some("cfg_attr"), after) if tree.kind(after) == Kind::Open(Delimiter::Parenthesis) => {
                metas.extend(cfg_attr_metas(tree, after));
            }
            (_, after) => ranges.push((after, end)),
        }
    }
    ranges
}

impl Scan<'_, '_> {
    /// Queue the contents of the turbofish opened at `open`, recording the tail of a qualified path a `<<` opens
    /// inside it, and return the token closing it.
    fn turbofish(&mut self, open: usize) -> Option<usize> {
        let close = angle_group_end(self.tree, open)?;
        if self.tree.is(open, "<<") {
            self.qualified_tails
                .extend(inner_qualified_tail(self.tree, open));
        }
        self.work.push((open + 1, close));
        Some(close)
    }

    /// Read tokens `from..to`.
    fn read(&mut self, from: usize, to: usize) {
        let tree = self.tree;
        let mut i = from;
        while i < to {
            if let Node::Attribute { open, close, .. } = tree.node_at(i) {
                self.work.extend(attribute_arguments(tree, open, close));
                i = close + 1;
                continue;
            }
            if tree.is(i, ".") {
                i += 1;
                if i < to && tree.kind(i) == Kind::Literal && tree.text(i).ends_with('.') {
                    i += 1;
                }
                if i < to && matches!(tree.kind(i), Kind::Ident | Kind::RawIdent | Kind::Keyword) {
                    i += 1;
                    if tree.is(i, "::") && opens_an_angle_group(tree, i + 1) {
                        if let Some(close) = self.turbofish(i + 1) {
                            i = close + 1;
                        }
                    }
                }
                continue;
            }
            if tree.is(i, "<") || tree.is(i, "<<") {
                if let Some(tail) = opens_a_qualified_path(tree, i) {
                    self.qualified_tails.insert(tail);
                    if tree.is(i, "<<") {
                        self.qualified_tails.extend(inner_qualified_tail(tree, i));
                    }
                }
                i += 1;
                continue;
            }
            let (start, head, rooted) = if tree.is(i, "::") && heads_a_path(tree, i + 1) {
                if self.qualified_tails.contains(&i) {
                    let tail = path_run(tree, i + 1);
                    for open in tail.turbofishes {
                        self.turbofish(open);
                    }
                    i = tail.end;
                    continue;
                }
                (i, i + 1, true)
            } else if heads_a_path(tree, i) {
                (i, i, false)
            } else {
                i += 1;
                continue;
            };
            let PathRun {
                segments,
                end,
                turbofishes,
            } = path_run(tree, head);
            for open in turbofishes {
                self.turbofish(open);
            }
            let defines = !rooted
                && (self.variants.contains(&head)
                    || head
                        .checked_sub(1)
                        .is_some_and(|p| tree.is(p, "fn") || tree.is(p, "struct")));
            let applied = end < to && tree.kind(end) == Kind::Open(Delimiter::Parenthesis);
            let is_call = applied && !defines;
            if is_call
                || (!defines
                    && (rooted
                        || segments.len() > 1
                        || names_a_single_segment_path(tree, head, end)))
            {
                let joined = segments.join("::");
                self.out.push(Occurrence {
                    at: start,
                    segments: if rooted {
                        format!("::{joined}")
                    } else {
                        joined
                    },
                    is_call,
                });
            }
            i = end;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::super::token_tree::Edition;
    use super::*;

    fn read(source: &str) -> Vec<(String, bool)> {
        let tree = TokenTree::lex(source, Edition::Rust2018);
        let spans: Vec<(usize, usize)> = super::super::use_tree::use_statements(&tree)
            .iter()
            .map(|statement| (statement.at, statement.end))
            .collect();
        occurrences(&tree, &spans)
            .into_iter()
            .map(|o| (o.segments, o.is_call))
            .collect()
    }

    /// The calls alone, for a direction whose subject is what reads as a call.
    fn calls(source: &str) -> Vec<String> {
        read(source)
            .into_iter()
            .filter(|(_, is_call)| *is_call)
            .map(|(path, _)| path)
            .collect()
    }

    #[test]
    fn a_role_is_read_from_the_tokens_beside_a_run() {
        assert_eq!(
            read("fn f() { a::b(); Vec::<u8>::new(); ::std::x::y; x.m(); P(1); r#match(x) }"),
            [
                ("a::b".to_string(), true),
                ("Vec::new".to_string(), true),
                ("::std::x::y".to_string(), false),
                ("x".to_string(), false),
                ("P".to_string(), true),
                ("match".to_string(), true),
                ("x".to_string(), false),
                ("u8".to_string(), false),
            ]
        );
    }

    /// A single identifier is a path mentioned where it names something, and not where it introduces a name: `now`
    /// read as a value and `Instant` as a type are mentions, while an item's name, a field's, a parameter's, a
    /// binding's, a `for` pattern's, a macro's and a bare `self` are not.
    #[test]
    fn a_single_identifier_is_a_mention_only_where_it_names_something() {
        assert_eq!(
            read(
                "fn f(p: Instant) { let g: fn() = now; let mut m = 1; let _ = &mut Clock; for k in v {} S { field: 1 }; m!(); self.x; }\n\
                 struct S { field: u8 } const C: u8 = 1; static D: u8 = 1; type T = U; mod n {} trait R {}"
            ),
            [
                ("Instant".to_string(), false),
                ("now".to_string(), false),
                ("Clock".to_string(), false),
                ("v".to_string(), false),
                ("S".to_string(), false),
                ("u8".to_string(), false),
                ("u8".to_string(), false),
                ("u8".to_string(), false),
                ("U".to_string(), false),
            ]
        );
    }

    #[test]
    fn a_definition_is_not_a_call() {
        let read = read("pub fn g() {} pub struct P(pub u8); enum E { A(u8), #[x] B(u8) }");
        assert!(
            read.iter().all(|(path, is_call)| path == "u8" && !is_call),
            "{read:?}"
        );
    }

    #[test]
    fn a_qualified_paths_tail_is_not_read_and_a_comparison_opens_no_group() {
        assert!(calls("fn g() { <W>::md5x(); return <W>::f(); }").is_empty());
        assert_eq!(
            calls(
                "fn g() { a < b && c > ::std::process::id(); x.await < b && c > ::std::process::id() }"
            ),
            ["::std::process::id", "::std::process::id"]
        );
        assert_eq!(
            calls("fn g() { let b = { 1 } < n && k > ::std::process::id(); }"),
            ["::std::process::id"]
        );
    }

    #[test]
    fn an_attributes_name_and_predicate_are_passed_over() {
        assert!(
            calls("#[cfg(any(unix, not(x)))] #![allow(y)] #[cfg_attr(a::b(), c::d(e))] fn g() {}")
                .is_empty()
        );
        assert_eq!(calls("#[m::attr(x = a::b())] fn g() {}"), ["a::b"]);
    }
}
