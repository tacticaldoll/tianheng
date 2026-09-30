//! Extraction of top-level `mod` declarations and their cfg/path attributes, read from a file's [`TokenTree`].

use super::super::item_head::{
    GroupKind, attribute_path, cfg_attr_metas, classify_group, in_macro_body, is_outer_attribute,
    item_at_keyword, macro_group_kind,
};
use super::super::path_vocab::canonical_segment;
use super::super::token_tree::{Delimiter, Kind, Node, TokenTree};

/// One `mod` declared at the top level of a token range: its canonical name, and — for an inline declaration
/// (`{ … }`) — the token indices of its body's braces, `None` for a file declaration (`;`), so a caller can re-scan
/// just that body to find further declarations nested inside it.
pub(super) struct DeclaredModule {
    pub(super) name: String,
    /// The inline body's `{` and `}`, and `None` for a file declaration — one field, so an inline declaration
    /// without a body, or a file declaration with one, cannot be built.
    pub(super) body: Option<(usize, usize)>,
    /// An **unconditional** `#[path = …]` before the declaration: `Some(Some(value))` where its value is a string
    /// literal, `Some(None)` where it is written but its value cannot be read, which leaves the declaration no
    /// backing source rather than the conventional one; `None` where there is none.
    pub(super) direct_path: Option<Option<String>>,
    /// Whether a `cfg_attr(…, path = …)` is written before the direct `#[path]`, so the direct one is the first path
    /// attribute only where that predicate is false and its target may be absent on a build that compiles the
    /// candidate — which the walk grants only where some candidate's file exists, the test a lone `cfg_attr` path
    /// already meets, so a declaration no configuration backs still fails loud — measured against rustc 1.96.0, edition 2021, on unix: `#[cfg_attr(unix, path = "b.rs")]
    /// #[path = "a.rs"] mod m;` builds with only `b.rs` on disk.
    pub(super) direct_path_is_conditional: bool,
    /// Every readable `cfg_attr(…, path = "…")` value before the declaration, nested `cfg_attr`s included, in
    /// textual order.
    pub(super) conditional_paths: Vec<String>,
    /// Whether this declaration may legitimately have no source file in the current configuration —
    /// the "might legitimately be absent on this build" signal. Only meaningful for a non-inline
    /// (file-form) declaration with no resolvable file, where it decides between a tolerated skip and
    /// a constitution error, for a plain conventional file and for a `#[path]` remap target alike.
    ///
    /// Two sources, treated identically because they express one intent:
    /// - a BARE `#[cfg(...)]` attribute (never `cfg_attr`) precedes the item;
    /// - the declaration sits directly inside a transparent control-flow macro arm (`cfg_if!`), whose
    ///   predicate lives in the macro's `if #[cfg(..)]` header rather than on the item. Every arm is
    ///   conditionally compiled by construction, the trailing `else` on its predicate's negation.
    ///
    /// A `cfg_attr` counts only where it applies a `cfg`: unlike a bare `#[cfg(pred)]`, which removes the whole item
    /// when `pred` is false, `#[cfg_attr(pred, …)]` removes nothing of itself, so `#[cfg_attr(unix,
    /// allow(dead_code))] mod x;` with no backing file is a genuine compile error, E0583 (verified against a real
    /// `rustc` build), while `#[cfg_attr(all(), cfg(any()))] mod x;` applies a `cfg` that removes it and builds with
    /// no `x.rs`, measured against rustc 1.96.0, edition 2021.
    ///
    /// Deliberately NOT the same signal as 渾儀's `has_cfg_attr`, which reads only the item's own
    /// attributes: the semantic dimension does not observe arm declarations at all yet, so it has
    /// nothing here to agree or disagree with until it does.
    pub(super) is_cfg_conditional: bool,
    /// Whether a block declares it, so it names no conventional file — rustc refuses a file-form `mod` in a block
    /// with no path attribute that applies — and is read from its path attributes alone.
    pub(super) declared_in_block: bool,
}

/// What the attributes before one `mod` declare about where its source is.
#[derive(Default)]
struct PathAttributes {
    direct: Option<Option<String>>,
    direct_after_candidate: bool,
    conditional: Vec<(usize, String)>,
    bare_cfg: bool,
}

/// Scan the `mod` declarations at the top level of tokens `range` — a whole file, or an inline module's body
/// between its braces.
///
/// A group is passed over whole, so a `mod` nested inside another item declares no child of this range; an inline
/// body's contents are re-scanned by the caller only if the module turns out to be inline-only. A macro's group is
/// passed over too, except a `cfg_if!`'s, whose arms hold top-level declarations of the range, nested `cfg_if!`s
/// included. A `mod` is read by the item-header grammar every reader shares. Nothing recurses: arms wait on a
/// worklist.
pub(super) fn declared_modules_in(
    tree: &TokenTree,
    range: std::ops::Range<usize>,
) -> Vec<DeclaredModule> {
    let mut declared: Vec<(usize, DeclaredModule)> = Vec::new();
    let mut work: Vec<(usize, usize, bool)> = vec![(range.start, range.end.min(tree.len()), false)];
    while let Some((from, to, in_arm)) = work.pop() {
        let mut i = from;
        while i < to {
            let node = tree.node_at(i);
            if let Node::Macro { open, close, .. } = node {
                if macro_group_kind(tree, open) == Some(GroupKind::CfgIf) {
                    work.extend(
                        cfg_if_arms(tree, open, close).map(|(open, close)| (open + 1, close, true)),
                    );
                }
                i = close + 1;
                continue;
            }
            let head = (tree.kind(i) == Kind::Keyword && tree.text(i) == "mod")
                .then(|| item_at_keyword(tree, i))
                .flatten();
            let Some((head, name)) = head.and_then(|head| head.name.map(|name| (head, name)))
            else {
                i = node.last() + 1;
                continue;
            };
            let body = head.body.map(|open| (open, tree.partner(open)));
            if body.is_none() && !tree.is(name + 1, ";") {
                i = node.last() + 1;
                continue;
            }
            let attributes = attributes_before(tree, head.start);
            i = body.map_or(name + 2, |(_, close)| close + 1);
            declared.push((
                head.start,
                DeclaredModule {
                    name: canonical_segment(tree.text(name)).to_string(),
                    body,
                    direct_path: attributes.direct,
                    direct_path_is_conditional: attributes.direct_after_candidate,
                    conditional_paths: attributes
                        .conditional
                        .into_iter()
                        .map(|(_, value)| value)
                        .collect(),
                    is_cfg_conditional: body.is_none() && (attributes.bare_cfg || in_arm),
                    declared_in_block: false,
                },
            ));
        }
    }
    declared.sort_by_key(|(at, _)| *at);
    declared.into_iter().map(|(_, module)| module).collect()
}

/// The arms of the `cfg_if!` whose group is `open..=close`: the brace groups directly inside it that the group
/// classifier reads as arms.
fn cfg_if_arms<'t>(
    tree: &'t TokenTree,
    open: usize,
    close: usize,
) -> impl Iterator<Item = (usize, usize)> + 't {
    let mut k = open + 1;
    std::iter::from_fn(move || {
        while k < close {
            let child = tree.node_at(k);
            k = child.last() + 1;
            if let Node::Group { open, close } = child {
                if classify_group(tree, open, Some(&GroupKind::CfgIf)) == GroupKind::CfgArm {
                    return Some((open, close));
                }
            }
        }
        None
    })
}

/// Every file-form `mod` a block within tokens `range` declares with a path attribute, direct or `cfg_attr` — `fn f()
/// { #[path = "x.rs"] mod m; }` and `fn f() { #[cfg_attr(unix, path = "x.rs")] mod m; }`, which rustc compiles on unix,
/// measured against rustc 1.96.0, edition 2021 — named as the module it is governed
/// as: `{block}::m`, and `{block N}::m` for the Nth module named `m` a block of the range declares, inline or not, in
/// source order, so two are two modules. A block names nothing a path outside it can write, so the name is the
/// block's readable form rather than a path; rustc refuses a file-form `mod` in a block with no path attribute, so
/// none is read. A `mod` inside a macro's group, or inside an inline module the range holds — whose own scan reads it — is
/// not this range's, and a `cfg_if!` arm is no block: [`declared_modules_in`] reads its declarations. The walk out from a
/// `mod` ends at the first module body it meets, which decides it, so a range reads a `mod` its inline modules hold in
/// time the blocks between the `mod` and that body bound, rather than once per group enclosing it.
pub(super) fn block_path_modules(
    tree: &TokenTree,
    range: std::ops::Range<usize>,
) -> Vec<DeclaredModule> {
    let mut seen: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut declared = Vec::new();
    let end = range.end.min(tree.len());
    for i in range.start..end {
        if tree.kind(i) != Kind::Keyword || tree.text(i) != "mod" {
            continue;
        }
        let mut in_block = false;
        let mut in_nested_module = false;
        let mut at = tree.enclosing(i);
        while let Some(open) = at.filter(|&open| open >= range.start) {
            let arm = tree
                .enclosing(open)
                .is_some_and(|outer| macro_group_kind(tree, outer) == Some(GroupKind::CfgIf));
            match classify_group(tree, open, None) {
                GroupKind::ModuleBody(_) => {
                    in_nested_module = true;
                    break;
                }
                GroupKind::CfgIf => {}
                _ if arm => {}
                _ => in_block = true,
            }
            at = tree.enclosing(open);
        }
        if !in_block || in_nested_module || in_macro_body(tree, i) {
            continue;
        }
        let Some((head, name)) =
            item_at_keyword(tree, i).and_then(|head| head.name.map(|name| (head, name)))
        else {
            continue;
        };
        let name = canonical_segment(tree.text(name)).to_string();
        let count = seen.entry(name.clone()).or_default();
        *count += 1;
        if head.body.is_some() || !tree.is(head.name.map_or(i, |n| n + 1), ";") {
            continue;
        }
        let attributes = attributes_before(tree, head.start);
        if attributes.direct.is_none() && attributes.conditional.is_empty() {
            continue;
        }
        let block = if *count == 1 {
            "{block}".to_string()
        } else {
            format!("{{block {count}}}")
        };
        declared.push(DeclaredModule {
            name: format!("{block}::{name}"),
            body: None,
            direct_path: attributes.direct,
            direct_path_is_conditional: attributes.direct_after_candidate,
            conditional_paths: attributes
                .conditional
                .into_iter()
                .map(|(_, value)| value)
                .collect(),
            is_cfg_conditional: attributes.bare_cfg,
            declared_in_block: true,
        });
    }
    declared
}

/// The `#[path]`, `cfg_attr(…, path = …)` and bare `#[cfg]` attributes of the item whose header starts at `start`:
/// the outer attribute nodes standing directly before it. An inner attribute, `#![…]`, belongs to the item it is
/// written in, so it ends the walk.
///
/// The static scanner intentionally does not read attributes in general, but `path` is a stated coverage concern
/// either way: an unconditional, direct `#[path = "…"]` is followed, and a `cfg_attr`-wrapped one written before it
/// is a conditional candidate beside it. Every candidate that may be compiled is unioned, because this scanner is
/// deliberately cfg-blind and the active predicate may not silently remove governed source. An
/// attribute's name is one segment, so a raw spelling names the built-in: `#[r#path = "…"]` IS the remap,
/// measured against rustc 1.96.0, edition 2021, which compiles the remapped file for it.
///
/// rustc compiles the first path attribute written and reports every later one as unused, so which attributes are
/// read is decided by position, and only a `cfg_attr` one's predicate is left to the build. The first direct
/// `#[path]` is the remap; a `cfg_attr` path written before it is a candidate, since where its predicate holds it is
/// the first; every path written after it, direct or `cfg_attr`, is never compiled and is not read. Measured
/// against rustc 1.96.0, edition 2021: `#[path = "a.rs"] #[path = "b.rs"] mod m;` and
/// `#[path = "a.rs"] #[cfg_attr(unix, path = "b.rs")] mod m;` compile `a.rs` with no `b.rs` on disk and warn that
/// the second attribute is unused, while `#[cfg_attr(unix, path = "b.rs")] #[path = "a.rs"] mod m;` compiles `b.rs`
/// on unix.
fn attributes_before(tree: &TokenTree, start: usize) -> PathAttributes {
    let mut attributes = Vec::new();
    let mut k = start;
    while let Some(node @ Node::Attribute { open, close, .. }) = tree.node_before(k) {
        if !is_outer_attribute(node) {
            break;
        }
        attributes.push((open + 1, close));
        k = node.first();
    }
    let mut found = PathAttributes::default();
    let mut direct_at = None;
    let mut metas: Vec<(usize, usize, bool)> = attributes
        .into_iter()
        .map(|(start, end)| (start, end, false))
        .collect();
    while let Some((start, end, applied)) = metas.pop() {
        match attribute_path(tree, start, end) {
            (Some("path"), after) if tree.is(after, "=") && !applied => {
                if direct_at.is_none() {
                    direct_at = Some(start);
                    found.direct = Some(tree.string_value(after + 1));
                }
            }
            (Some("path"), after) if tree.is(after, "=") => {
                if let Some(value) = tree.string_value(after + 1) {
                    found.conditional.push((start, value));
                }
            }
            (Some("cfg"), _) => found.bare_cfg = true,
            (Some("cfg_attr"), after) if tree.kind(after) == Kind::Open(Delimiter::Parenthesis) => {
                metas.extend(
                    cfg_attr_metas(tree, after)
                        .into_iter()
                        .rev()
                        .map(|(meta, end)| (meta, end, true)),
                );
            }
            _ => {}
        }
    }
    if let Some(direct_at) = direct_at {
        found.conditional.retain(|(at, _)| *at < direct_at);
        found.direct_after_candidate = !found.conditional.is_empty();
    }
    found.conditional.sort_by_key(|(at, _)| *at);
    found
}

/// The declared module names at the top level of `source`, each paired with whether it is inline.
#[cfg(test)]
fn declared_modules_with_kind(source: &str) -> Vec<(String, bool)> {
    let tree = TokenTree::lex(source, super::super::token_tree::Edition::Rust2018);
    declared_modules_in(&tree, 0..tree.len())
        .into_iter()
        .map(|declared| (declared.name, declared.body.is_some()))
        .collect()
}

/// The declared module names only, discarding the inline/file kind.
#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declared_modules_with_kind(source)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::super::super::token_tree::Edition;
    use super::*;

    fn attributes(source: &str) -> PathAttributes {
        let tree = TokenTree::lex(source, Edition::Rust2018);
        let at = (0..tree.len())
            .rev()
            .find(|&i| tree.is(i, "mod"))
            .expect("a mod");
        attributes_before(&tree, at)
    }

    /// The `path` and the bare `cfg` reader take the attribute's name from one position, so they agree about the
    /// same source whatever its spacing or raw spelling.
    #[test]
    fn both_readers_take_the_attribute_name_from_one_position() {
        for (prefix, remaps, bare_cfg) in [
            ("#[path = \"x.rs\"]", true, false),
            ("#[cfg(unix)]", false, true),
            ("# [ path = \"x.rs\"]", true, false),
            ("# [ cfg(unix)]", false, true),
            ("#[r#path = \"x.rs\"]", true, false),
            ("#[r#cfg(unix)]", false, true),
            ("#[r#cfg_attr(unix, path = \"x.rs\")]", true, false),
            ("#[path]", false, false),
            ("#[path(\"x.rs\")]", false, false),
            ("##[path = \"x.rs\"]", true, false),
            ("#![cfg(unix)]", false, false),
            ("#![path = \"x.rs\"]", false, false),
            ("mod tests { #![cfg(test)]", false, false),
        ] {
            let found = attributes(&format!("{prefix} mod m;"));
            let remapped = found.direct.is_some() || !found.conditional.is_empty();
            assert_eq!(remapped, remaps, "the path reader's verdict on {prefix}");
            assert_eq!(
                found.bare_cfg, bare_cfg,
                "the cfg reader's verdict on {prefix}"
            );
        }
    }

    #[test]
    fn deeply_nested_cfg_attr_paths_use_a_bounded_native_stack() {
        const DEPTH: usize = 4096;
        let mut nested = String::from("#[cfg_attr");
        for _ in 0..DEPTH {
            nested.push_str("(predicate, cfg_attr");
        }
        nested.push_str("(predicate, path = \"target.rs\")");
        for _ in 0..=DEPTH {
            nested.push(')');
        }
        nested.push_str("] mod m;");
        let found = attributes(&nested);
        assert_eq!(
            found.conditional.len(),
            1,
            "the deepest path candidate remains observable"
        );
        assert_eq!(found.conditional[0].1, "target.rs");
    }

    #[test]
    fn iterative_cfg_attr_collection_preserves_textual_candidate_order() {
        let found = attributes(
            "#[cfg_attr(a, cfg_attr(b, path = \"first.rs\"), path = \"second.rs\", cfg_attr(c, path = \"third.rs\"))] mod m;",
        );
        let values: Vec<&str> = found.conditional.iter().map(|(_, v)| v.as_str()).collect();
        assert_eq!(values, ["first.rs", "second.rs", "third.rs"]);
    }

    /// The attribute admitting applied metas is the BUILT-IN `cfg_attr`, whose path is exactly one segment. A
    /// qualified look-alike ends in the same word while being somebody else's attribute, so descending into it
    /// would collect a target no build compiles; an unqualified one, raw-spelled or not, keeps its metas.
    #[test]
    fn a_qualified_look_alike_is_not_the_built_in_cfg_attr() {
        for (label, attr, expected) in [
            (
                "plain",
                "#[cfg_attr(any(), foo::cfg_attr(a, path = \"bogus\"), path = \"real.rs\")]",
                1,
            ),
            (
                "raw identifier",
                "#[cfg_attr(any(), foo::r#cfg_attr(a, path = \"bogus\"), path = \"real.rs\")]",
                1,
            ),
            (
                "nested plain",
                "#[cfg_attr(any(), cfg_attr(a, path = \"nested.rs\"))]",
                1,
            ),
            (
                "nested raw",
                "#[cfg_attr(any(), r#cfg_attr(a, path = \"nested.rs\"))]",
                1,
            ),
        ] {
            let found = attributes(&format!("{attr} mod m;"));
            assert_eq!(
                found.conditional.len(),
                expected,
                "{label}: {:?}",
                found.conditional
            );
            assert!(
                found.conditional.iter().all(|(_, v)| v != "bogus"),
                "{label}"
            );
        }
    }

    /// A `#[path]` whose value is no string literal is written but unreadable, which is not the same fact as no
    /// remap: the declaration then has no backing source.
    #[test]
    fn an_unreadable_path_value_is_not_an_absent_path() {
        assert_eq!(
            attributes("#[path = concat!(\"a\")] mod m;").direct,
            Some(None)
        );
        assert_eq!(attributes("#[path = \"a\\q\"] mod m;").direct, Some(None));
        assert_eq!(
            attributes("#[path = r#\"a.rs\"#] mod m;").direct,
            Some(Some("a.rs".into()))
        );
        assert_eq!(
            attributes("#[path = \"a\\\n   b.rs\"] mod m;").direct,
            Some(Some("ab.rs".into()))
        );
        assert_eq!(attributes("mod m;").direct, None);
    }
}
