//! The `use`-import scan: given a file's source and its module, extract the internal
//! `crate::…` module paths it imports via `use` — grouped/glob forms expanded, raw
//! identifiers canonicalized, external crates and out-of-scope forms (bare path
//! expressions, macro-generated imports) dropped. A `::*` glob is observed at its **base**
//! module only (`use a::b::*;` → `a::b`): a glob of an *ancestor* of a forbidden module does
//! not surface the forbidden descendant as an edge — a declared partial-coverage bound of the
//! denylist rule (`must_not_import`), documented on its builder. Inline `mod name { … }` nesting is
//! tracked so `self`/`super` resolve against the real enclosing module. Depends downward
//! on [`super::lexer`] (hygiene / token boundaries) and [`super::path_vocab`] (segment
//! canonicalization, the `mod`-keyword test); pure string processing, no model type.

use super::lexer::{keyword_starts_at, strip_comments_and_strings, strip_macro_bodies};
use super::path_vocab::{
    brace_content, canonical_segment, effective_module, inline_mod_at, is_crate_root_shadow,
    resolve_self_super, split_top_commas,
};

/// Internal module paths imported by `source`, normalized to absolute `crate::…`
/// form. `current_module` is the importing file's module; a `use` inside an inline
/// `mod name { … }` is attributed to that submodule, so `self`/`super` resolve against
/// the real enclosing module, not the file's. Only `use` declarations are observed;
/// grouped and glob forms are expanded; raw identifiers (`r#name`) are canonicalized;
/// paths whose first segment is an external crate are ignored. Bare path expressions
/// and macro-generated imports are out of scope (PROJECT.md): comments and string
/// literals are stripped, and macro bodies are removed, so a `use` written inside one
/// is a macro-generated import and is not observed. Returns sorted, de-duplicated paths.
pub(crate) fn imported_module_paths(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> Vec<String> {
    let mut paths: Vec<String> = imports_with_importers(source, current_module, root_modules)
        .into_iter()
        .map(|(_importer, import)| import)
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Each internal import paired with the **module that actually declares it** — inline-aware, so a
/// `use` inside an inline `mod inner { … }` is attributed to `{current_module}::inner`, not the
/// file's module. The inbound rules (`MustNotBeImportedBy` / `MustOnlyBeImportedBy`) test the
/// *importer's* identity, so they need this pair; [`imported_module_paths`] keeps only the absolute
/// import path (right for the outbound rules, which test the import), discarding the importer an
/// inbound rule would otherwise mis-attribute to the file's module. Sorted + deduped by
/// `(importer, import)`.
pub(crate) fn imports_with_importers(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> Vec<(String, String)> {
    let cleaned = strip_macro_bodies(&strip_comments_and_strings(source));
    let mut pairs = Vec::new();
    for (module, tree) in use_trees_with_modules(&cleaned, current_module) {
        for leaf in expand_use_tree(&tree) {
            if let Some(absolute) = normalize_module_path(&leaf, &module, root_modules) {
                pairs.push((module.clone(), absolute));
            }
        }
    }
    pairs.sort();
    pairs.dedup();
    pairs
}

/// Each importer module paired with the **external** crate it imports — the mirror of
/// [`imports_with_importers`] for the one rule that observes external imports (confinement),
/// instead of dropping them. Same lexical pipeline (comment/string/macro strip, inline-`mod`
/// attribution, group/glob expansion); the only difference is [`external_crate_head`] in place
/// of `normalize_module_path`, so an external head is captured **exactly when** the internal
/// scan would have discarded it as external. Sorted + deduped by `(importer, external crate)`.
pub(crate) fn external_imports_with_importers(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> Vec<(String, String)> {
    let cleaned = strip_macro_bodies(&strip_comments_and_strings(source));
    let mut pairs = Vec::new();
    for (module, tree) in use_trees_with_modules(&cleaned, current_module) {
        for leaf in expand_use_tree(&tree) {
            if let Some(head) = external_crate_head(&leaf, &module, root_modules) {
                pairs.push((module.clone(), head));
            }
        }
    }
    pairs.sort();
    pairs.dedup();
    pairs
}

/// Each `use … ;` statement paired with the module that lexically encloses it. The
/// walk tracks inline `mod name { … }` nesting by brace depth, so a `use` inside an
/// inline submodule is attributed to that submodule (e.g. `crate::a::inner`) rather
/// than the file's module (`crate::a`); `self`/`super` then resolve against the real
/// enclosing module, and a bare first segment inside an inline submodule is external
/// even when the file is the crate root. A `mod name;` with no inline body encloses
/// nothing. The text is already comment/string/macro-stripped, so every brace is
/// structural; a `use … ;` is consumed whole, so its own group braces
/// (`use a::{b, c};`) never perturb the depth.
fn use_trees_with_modules(source: &str, base_module: &str) -> Vec<(String, String)> {
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    // (inline module name, enclosing brace depth).
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if keyword_starts_at(bytes, i, b"use") {
            let start = i + 3;
            // A precise-capturing bound `-> impl Trait + use<'a, T>` (stable Rust) puts a `use`
            // token inside a type bound: it is followed by `<`, whereas a `use` *statement* is
            // always followed by a path (ident / `{` / `*` / `::` / `crate`/`self`/`super`).
            // So a `<` here means this is a bound, not an import — skip past the `use` token and
            // continue (letting the `<…>` be walked as ordinary bytes) rather than scanning to the
            // next `;`, which would swallow the following real `use` (a false negative). A comment
            // between `use` and `<` is already removed by the upstream comment/string strip.
            let mut p = start;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            if bytes.get(p) == Some(&b'<') {
                i = start;
                continue;
            }
            match source[start..].find(';') {
                Some(rel) => {
                    trees.push((
                        effective_module(base_module, &mod_stack),
                        source[start..start + rel].trim().to_string(),
                    ));
                    i = start + rel + 1;
                    continue;
                }
                None => break,
            }
        }
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            mod_stack.push((
                canonical_segment(source[name_start..name_end].trim()).to_string(),
                depth,
            ));
            i = brace; // let the `{` arm below increment the depth
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                while mod_stack.last().is_some_and(|(_, d)| *d == depth) {
                    mod_stack.pop();
                }
            }
            _ => {}
        }
        i += 1;
    }
    trees
}

/// Expand a use tree into leaf paths: `a::{b, c::d}` -> `a::b`, `a::c::d`; drop
/// `::*` and ` as alias`; `{self}` resolves to the prefix module.
fn expand_use_tree(tree: &str) -> Vec<String> {
    expand_use_tree_depth(tree, 0)
}

/// A brace-nesting depth cap so a pathologically nested `use a::{b::{c::{ … }}}` cannot overflow the
/// stack — a DoS backstop set far beyond any real or lint-clean source (rustfmt-formatted `use`s
/// nest a handful of levels). Past the cap the sub-tree is not expanded; this is the only place the
/// scanner bounds coverage, so it is logged as a stated limit, not a silent hole for real code.
const MAX_USE_NEST_DEPTH: usize = 128;

fn expand_use_tree_depth(tree: &str, depth: usize) -> Vec<String> {
    if depth >= MAX_USE_NEST_DEPTH {
        return Vec::new();
    }
    let tree = tree.trim();
    match tree.find('{') {
        Some(open) => {
            let prefix = tree[..open].trim();
            let inner = brace_content(&tree[open..]);
            let mut out = Vec::new();
            for part in split_top_commas(&inner) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                // `self` — bare or aliased (`self as x`) — names the prefix module itself, not a
                // child. Strip a trailing ` as <alias>` before the check so `{self as cfg}` resolves
                // to the prefix module rather than falling through to the leaf branch and leaving a
                // phantom `…::self` segment.
                let head = match part.find(" as ") {
                    Some(idx) => part[..idx].trim(),
                    None => part,
                };
                if head == "self" {
                    let module = prefix.trim_end_matches(':').trim();
                    if !module.is_empty() {
                        out.push(module.to_string());
                    }
                } else {
                    out.extend(expand_use_tree_depth(&format!("{prefix}{part}"), depth + 1));
                }
            }
            out
        }
        None => {
            let leaf = match tree.find(" as ") {
                Some(idx) => &tree[..idx],
                None => tree,
            };
            let leaf = leaf.trim().strip_suffix("::*").unwrap_or(leaf.trim());
            let leaf = leaf.trim_end_matches(':');
            if leaf.is_empty() {
                Vec::new()
            } else {
                vec![leaf.to_string()]
            }
        }
    }
}

/// Resolve a use path to an absolute `crate::…` module path, or `None` if it refers
/// to an external crate. A first segment of `crate`/`self`/`super` resolves as usual.
/// A first segment that names a crate-root module (`root_modules`) resolves to
/// `crate::…` **only when the importing file is the crate root** (`current_module ==
/// "crate"`): there a sibling `mod` is in scope and shadows the extern prelude, so a
/// bare `use foo::…` is the local module. In a submodule a bare first segment reaches
/// only the extern prelude (a bare crate-root-module path there is either an external
/// crate or a compile error), so it is external. A path written with a leading `::`
/// (`use ::foo::…`) is explicitly the external/global crate and is always external. Any
/// other first segment is external (`None`).
/// Split `path` on `::` into canonicalized (raw-identifier-stripped, trimmed), non-empty
/// segments — the shared pipeline [`normalize_module_path`] and [`external_crate_head`] both
/// start from, so a canonicalization fix cannot land in one and not the other.
fn split_canonical_segments(path: &str) -> Vec<&str> {
    path.split("::")
        .map(|segment| canonical_segment(segment.trim()))
        .filter(|segment| !segment.is_empty())
        .collect()
}

fn normalize_module_path(
    path: &str,
    current_module: &str,
    root_modules: &[String],
) -> Option<String> {
    // A leading `::` is an explicit external/global path (`use ::serde::…`): it bypasses
    // the local-module shadow, so it is external even if a crate-root module shares the
    // name. Checked before segments are split so the marker is not lost.
    if path.trim_start().starts_with("::") {
        return None;
    }
    let segments = split_canonical_segments(path);
    let (first, _rest) = segments.split_first()?;
    match *first {
        "crate" => Some(segments.join("::")),
        // `self`/`super` relative resolution — incl. the `super` over-pop guard — lives once in
        // `path_vocab::resolve_self_super`, shared with the symbol-scan resolvers.
        "self" | "super" => resolve_self_super(current_module, &segments),
        other => {
            // A bare first segment names a crate-root module only at the crate root,
            // where a sibling `mod` is in scope and shadows the extern prelude — there a
            // bare `use foo::…` is the local module, so resolve to `crate::…`. In a
            // submodule the same bare path reaches only the extern prelude (it is an
            // external crate, or a compile error), so it is external. Gating on
            // `current_module == "crate"` keeps a submodule's `use serde::…` external
            // even when a `mod serde;` exists at the crate root. (Explicit external is
            // written `::name`, handled above.)
            if is_crate_root_shadow(current_module, other, root_modules) {
                let mut out = vec!["crate"];
                out.extend(segments.iter().copied());
                Some(out.join("::"))
            } else {
                None
            }
        }
    }
}

/// The **external** crate a use path names, or `None` if the path is internal or degenerate.
/// The precise inverse of [`normalize_module_path`]'s external branch: it returns `Some(name)`
/// exactly when that function returns `None` *because the head is an external crate* — and
/// `None` for the non-external cases (`crate`/`self`/`super`, a crate-root module reached bare
/// at the crate root, an empty path, an over-popped `super`). It reuses the identical
/// leading-`::`, `crate`/`self`/`super`, and crate-root-module-shadowing resolution, so
/// "external" has one definition shared with the internal scan, never a divergent one.
fn external_crate_head(
    path: &str,
    current_module: &str,
    root_modules: &[String],
) -> Option<String> {
    // A leading `::` is the explicit external/global form (`use ::libc::…`): the head is the
    // first real segment, external even when a crate-root module shares the name — the exact
    // mirror of the early-return `normalize_module_path` makes for the same marker.
    if path.trim_start().starts_with("::") {
        return split_canonical_segments(path)
            .into_iter()
            .next()
            .map(str::to_string);
    }
    let segments = split_canonical_segments(path);
    let (first, _rest) = segments.split_first()?;
    match *first {
        // Internal roots (or a degenerate/over-popped `super`) — never an external crate.
        "crate" | "self" | "super" => None,
        other => {
            // A bare first segment naming a crate-root module, at the crate root, is the
            // internal module (the sibling `mod` shadows the extern prelude) — not external,
            // exactly as `normalize_module_path` resolves it to `crate::…`. Everywhere else a
            // bare first segment reaches the extern prelude: it is the external crate.
            if is_crate_root_shadow(current_module, other, root_modules) {
                None
            } else {
                Some(other.to_string())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_expands_groups_and_resolves_relative_imports() {
        let source = r#"
            // a line comment mentioning use crate::ignored::me;
            use crate::a::{b, c::d};
            use super::sibling::X;
            use self::inner::Y;
            use serde::Deserialize;
            use crate::z::*;
        "#;
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(imports.contains(&"crate::a::b".to_string()), "{imports:?}");
        assert!(
            imports.contains(&"crate::a::c::d".to_string()),
            "{imports:?}"
        );
        // `super` from crate::kernel resolves to crate.
        assert!(
            imports.contains(&"crate::sibling::X".to_string()),
            "{imports:?}"
        );
        // `self` resolves against the current module.
        assert!(
            imports.contains(&"crate::kernel::inner::Y".to_string()),
            "{imports:?}"
        );
        // glob keeps the module prefix.
        assert!(imports.contains(&"crate::z".to_string()), "{imports:?}");
        // external first segment is ignored, and commented-out imports are not seen.
        assert!(!imports.iter().any(|p| p.contains("serde")), "{imports:?}");
        assert!(
            !imports.iter().any(|p| p.contains("ignored")),
            "{imports:?}"
        );
    }

    #[test]
    fn a_block_comment_between_use_and_its_path_does_not_fuse_the_keyword() {
        // A block comment wedged between the `use` keyword and its path must not fuse them —
        // `use/*re-export*/crate::secret::Thing;` stripped to `usecrate::secret::Thing;` would
        // leave `use` unrecognized and the import dropped.
        assert_eq!(
            imported_module_paths("use/*re-export*/crate::secret::Thing;", "crate", &[]),
            vec!["crate::secret::Thing".to_string()],
            "a block comment after `use` must not swallow the import",
        );
    }

    #[test]
    fn a_self_alias_in_a_use_group_resolves_to_the_prefix_module() {
        // `use crate::config::{self as cfg, Setting};` imports the
        // module `crate::config` (under the alias) plus `crate::config::Setting`. The `self as cfg`
        // form must resolve to the prefix module, not leave a phantom `crate::config::self` segment.
        let source = "use crate::config::{self as cfg, Setting};";
        let imports = imported_module_paths(source, "crate", &[]);
        assert_eq!(
            imports,
            vec![
                "crate::config".to_string(),
                "crate::config::Setting".to_string(),
            ],
            "a `self as alias` in a group resolves to the prefix module: {imports:?}"
        );
        // A lone `{self as x}` likewise resolves to the prefix module, never `…::self`.
        assert_eq!(
            imported_module_paths("use crate::config::{self as cfg};", "crate", &[]),
            vec!["crate::config".to_string()],
        );
    }

    #[test]
    fn super_past_the_crate_root_is_not_an_internal_module() {
        // `crate::a` has one ancestor (`crate`); `super::super` over-pops past the root.
        // Such a path names no internal module (and does not compile), so it must not be
        // observed — never a malformed root-less path like "other::X".
        let over = "use super::super::other::X;\n";
        assert!(
            imported_module_paths(over, "crate::a", &[]).is_empty(),
            "over-popped super must yield no import: {:?}",
            imported_module_paths(over, "crate::a", &[])
        );
        // A single `super` from `crate::a` still resolves to `crate::other::X`.
        let ok = "use super::other::X;\n";
        assert_eq!(
            imported_module_paths(ok, "crate::a", &[]),
            vec!["crate::other::X".to_string()],
        );
    }

    #[test]
    fn scanner_ignores_comments_and_string_literals() {
        // A `//` inside a string must not eat a real `use` later on the same line.
        let url = r#"let u = "http://example.com"; use crate::real::A;"#;
        let imports = imported_module_paths(url, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::A".to_string()),
            "{imports:?}"
        );

        // A `use …;` written inside a string is not a real import.
        let in_string = r#"let s = "use crate::ghost::Z;";"#;
        assert!(
            imported_module_paths(in_string, "crate::kernel", &[]).is_empty(),
            "a use inside a string must not be observed"
        );

        // A quote-bearing char literal must not open a spurious string and swallow code.
        let quote_char = r#"let q = '"'; use crate::real::B;"#;
        let imports = imported_module_paths(quote_char, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::B".to_string()),
            "{imports:?}"
        );

        // A lifetime must not break use detection or produce a spurious path.
        let lifetime = "fn f<'a>(x: &'a str) {} use crate::a::b;";
        let imports = imported_module_paths(lifetime, "crate::kernel", &[]);
        assert_eq!(imports, vec!["crate::a::b".to_string()], "{imports:?}");
    }

    #[test]
    fn scanner_handles_nested_block_comments() {
        // Rust nests block comments. Commenting out code that itself contains a
        // `/* */` must not let the inner `*/` re-expose the rest as live code: a
        // `use` inside the (nested) comment must not be observed, while a real `use`
        // after the outer close still is.
        let source = r#"
            /*
            fn old() {
                /* tweak later */
                use crate::legacy::Thing;
            }
            */
            use crate::current::A;
        "#;
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::current::A".to_string()),
            "the real use after the nested comment must be observed: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("legacy")),
            "a use inside a nested block comment must not be observed: {imports:?}"
        );
    }

    #[test]
    fn scanner_resolves_root_relative_bare_use() {
        // A root-relative bare `use kernel::…` (legal only at the crate root) names a
        // crate-root module, not an external crate, so it resolves to `crate::kernel::…`.
        // An unknown first segment is still external. With no root modules known, the
        // bare path stays external (the conservative behavior).
        let source = "use kernel::Thing; use serde::Deserialize;";
        let roots = vec!["kernel".to_string()];
        let imports = imported_module_paths(source, "crate", &roots);
        assert!(
            imports.contains(&"crate::kernel::Thing".to_string()),
            "a bare use of a crate-root module must resolve to crate::…: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("serde")),
            "an unknown first segment stays external: {imports:?}"
        );
        assert!(
            imported_module_paths(source, "crate", &[])
                .iter()
                .all(|p| !p.contains("kernel")),
            "with no root modules known, the bare path is treated as external"
        );
    }

    #[test]
    fn scanner_treats_leading_colon_path_as_external() {
        // `use ::serde::…` is an explicit external/global path: the leading `::` bypasses
        // the local-module shadow, so it is external even when a crate-root module shares
        // the name. (Dropping the leading `::` would mis-resolve it as the internal
        // `crate::serde::…`.)
        let roots = vec!["serde".to_string()];
        assert!(
            imported_module_paths("use ::serde::Deserialize;", "crate", &roots).is_empty(),
            "a leading-:: path must be external even when its head matches a root module"
        );
        // Sanity: without the leading `::`, the same head IS the local module, because a
        // crate-root module shadows the extern prelude in a bare `use` path.
        assert!(
            imported_module_paths("use serde::Deserialize;", "crate", &roots)
                .contains(&"crate::serde::Deserialize".to_string()),
            "a bare head matching a root module resolves locally (shadowing rule)"
        );
    }

    #[test]
    fn scanner_resolves_root_relative_use_only_at_crate_root() {
        // The crate-root-module shadow holds ONLY at the crate root. In a submodule a
        // bare `use serde::…` reaches the extern prelude (external), even when `serde`
        // is a crate-root module — matching the compiler. Resolving it to
        // `crate::serde::…` would be a false positive that fails a module boundary.
        let roots = vec!["serde".to_string()];
        assert!(
            imported_module_paths("use serde::Value;", "crate", &roots)
                .contains(&"crate::serde::Value".to_string()),
            "at the crate root, a bare crate-root-module path resolves locally"
        );
        assert!(
            imported_module_paths("use serde::Value;", "crate::sub", &roots).is_empty(),
            "in a submodule, a bare first segment is external even if it matches a root module"
        );
    }

    #[test]
    fn scanner_preserves_non_ascii_module_paths() {
        // strip is UTF-8 safe: a non-ASCII module path survives stripping intact.
        let source = "use crate::café::Item;";
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::café::Item".to_string()),
            "{imports:?}"
        );
    }

    #[test]
    fn scanner_ignores_raw_and_byte_strings() {
        // A `use …;` inside a raw string (any hash count) is not an import.
        for src in [
            r##"let s = r"use crate::ghost::Z;";"##,
            r##"let s = r#"use crate::ghost::Z;"#;"##,
            r##"let s = br#"use crate::ghost::Z;"#;"##,
            r#"let s = b"use crate::ghost::Z;";"#,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel", &[]).is_empty(),
                "a use inside a (raw/byte) string must not be observed: {src}"
            );
        }

        // A `//` and an inner `"#` inside a raw string must not eat a following use.
        // (Two outer hashes so the inner `"#` does not close it.)
        let tricky = r####"let s = r##"http://x "# inside"##; use crate::real::C;"####;
        let imports = imported_module_paths(tricky, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::C".to_string()),
            "{imports:?}"
        );

        // `r` / `b` as ordinary identifiers (not raw-string prefixes) are unaffected.
        let idents = "let r = 1; let b = 2; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel", &[]),
            vec!["crate::real::D".to_string()]
        );
    }

    #[test]
    fn scanner_ignores_raw_c_strings() {
        // A `use …;` inside a raw C-string (`cr"…"` / `cr#"…"#`, stable Rust 1.79) is not observed.
        for src in [
            r##"let s = cr"use crate::ghost::Z;";"##,
            r##"let s = cr#"use crate::ghost::Z;"#;"##,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel", &[]).is_empty(),
                "a use inside a raw C-string must not be observed: {src}"
            );
        }

        // The desync guard: a raw C-string with an **odd** number of inner unescaped `"` (raw
        // strings do not escape) must not swallow a following `use`. Before the `cr#` prefix was
        // recognized, the leading `c` made `raw_string_prefix` decline (and the following `r` sat
        // after an identifier byte, so it declined too), so `#"a"b"#` was scanned as code + plain
        // strings: `"a"` closed as one plain string, then the `"` before `#` opened an unterminated
        // one that ran to EOF, silently dropping the import (a false negative in FFI-flavoured
        // code). An odd inner-quote count is required to leave that final `"` unpaired; an even
        // count re-pairs into balanced plain strings and does not desync — so the test uses an odd
        // count (one inner `"` here) to actually fail on unfixed code, not merely pass on it.
        let tricky = r##"let s = cr#"a"b"#; use crate::real::C;"##;
        let imports = imported_module_paths(tricky, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::C".to_string()),
            "a use after a raw C-string with an odd inner-quote count must be observed: {imports:?}"
        );

        // A non-raw `c"…"` C-string (ordinary escaping) also hides an inner use and does not
        // desync — handled by the plain-string branch, the `c` prefix byte emitted as code.
        let cstr = r#"let s = c"use crate::ghost::Z;"; use crate::real::E;"#;
        assert_eq!(
            imported_module_paths(cstr, "crate::kernel", &[]),
            vec!["crate::real::E".to_string()]
        );

        // `c` as an ordinary identifier (not a C-string prefix) is unaffected.
        let idents = "let c = 1; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel", &[]),
            vec!["crate::real::D".to_string()]
        );
    }

    #[test]
    fn use_inside_a_macro_body_is_not_observed() {
        // A `use` written inside a macro — a `macro_rules!` definition OR a macro
        // invocation — is a macro-generated import (out of scope): it must not be
        // observed. A real `use` outside the macro still is.
        let source = r#"
            macro_rules! m {
                () => { use crate::ghost::Thing; };
                ($x:tt) => {{ use crate::ghost::Other; }};
            }
            with_imports! { use crate::ghost::FromInvocation; }
            use crate::real::A;
        "#;
        let imports = imported_module_paths(source, "crate", &[]);
        assert_eq!(
            imports,
            vec!["crate::real::A".to_string()],
            "macro definition and invocation bodies are skipped; the real use is kept: {imports:?}"
        );
        // Every body delimiter form ({}, (), []) is skipped, for both definitions and
        // invocations.
        for body in [
            "macro_rules! m { () => { use crate::ghost::T; }; }",
            "macro_rules! m ( () => { use crate::ghost::T; }; )",
            "macro_rules! m [ () => { use crate::ghost::T; }; ]",
            "some_macro! { use crate::ghost::T; }",
            "some_macro!( use crate::ghost::T; )",
            "some_macro![ use crate::ghost::T; ]",
        ] {
            assert!(
                imported_module_paths(body, "crate", &[]).is_empty(),
                "no import observed from a macro body: {body}"
            );
        }
        // `!=` and unary `!` are not macro invocations — a following real use is kept.
        let not_macros = "let _ = a != b; let _ = !flag; use crate::real::B;";
        assert!(
            imported_module_paths(not_macros, "crate", &[]).contains(&"crate::real::B".to_string()),
            "`!=` / unary `!` must not be treated as a macro invocation"
        );
    }

    #[test]
    fn a_precise_capturing_use_bound_does_not_swallow_the_following_use() {
        // `-> impl Trait + use<…>` (stable Rust) is a precise-capturing bound, not an import.
        // The scanner must not treat it as a `use` statement and consume to the next `;`, which
        // would swallow the real `use` that follows — a false negative that silently disables the
        // module boundary. Cover the empty, parameterized, whitespace, and comment forms.
        for header in [
            "fn iter() -> impl Iterator<Item = u8> + use<> { std::iter::empty() }",
            "fn iter<'a, T>() -> impl Iterator<Item = &'a T> + use<'a, T> { loop {} }",
            "fn iter() -> impl Iterator<Item = u8> + use <> { std::iter::empty() }",
            "fn iter() -> impl Iterator<Item = u8> + use /*c*/ <> { std::iter::empty() }",
        ] {
            let src = format!("{header}\nuse crate::forbidden::Thing;");
            assert_eq!(
                imported_module_paths(&src, "crate", &[]),
                vec!["crate::forbidden::Thing".to_string()],
                "the `use<…>` bound must be skipped so the following real use is observed: {header:?}"
            );
        }
    }

    #[test]
    fn a_use_bound_as_the_last_token_neither_panics_nor_drops_a_preceding_use() {
        // Control: a plain `use` is unaffected. And a `use<>` bound as the file's final token
        // (no trailing `;`) must not panic (bounds-safe peek) and must not drop the real `use`
        // that precedes it.
        assert_eq!(
            imported_module_paths("use crate::x::Y;", "crate", &[]),
            vec!["crate::x::Y".to_string()],
            "a plain use is unaffected by the bound-skip"
        );
        assert_eq!(
            imported_module_paths(
                "use crate::x::Y;\nfn f() -> impl Sized + use<>",
                "crate",
                &[],
            ),
            vec!["crate::x::Y".to_string()],
            "a trailing use<> bound must not drop the preceding real use or panic"
        );
    }

    #[test]
    fn use_inside_an_inline_module_is_attributed_to_that_module() {
        // A `self`/`super` import inside an inline `mod inner { … }` resolves against the
        // inline submodule, not the file's module: `self` -> crate::a::inner::…, and
        // `super` from crate::a::inner -> crate::a.
        let source = "mod inner { use self::leaf::Thing; use super::sibling::X; }";
        let imports = imported_module_paths(source, "crate::a", &[]);
        assert!(
            imports.contains(&"crate::a::inner::leaf::Thing".to_string()),
            "self must resolve against the inline submodule: {imports:?}"
        );
        assert!(
            imports.contains(&"crate::a::sibling::X".to_string()),
            "super from the inline submodule resolves to crate::a: {imports:?}"
        );
        // A bare first segment inside an inline submodule is external even when the file
        // IS the crate root (the enclosing module is crate::inner, not crate).
        let bare = imported_module_paths(
            "mod inner { use kernel::Thing; }",
            "crate",
            &["kernel".to_string()],
        );
        assert!(
            bare.is_empty(),
            "a bare use inside an inline submodule is external: {bare:?}"
        );
        // A top-level use (no inline module) is unchanged.
        assert_eq!(
            imported_module_paths("use self::leaf::Thing;", "crate::a", &[]),
            vec!["crate::a::leaf::Thing".to_string()]
        );
    }

    #[test]
    fn escaped_quote_char_literal_is_consumed_whole() {
        // `'\''` must be skipped as a whole so the following string's fake `use` is still
        // stripped and the real `use` after it is observed (a regression guard for the
        // escaped-char skip against leaking the closing quote).
        let src = r#"let _q = '\''; let _s = "use crate::ghost::Z;"; use crate::real::A;"#;
        assert_eq!(
            imported_module_paths(src, "crate::kernel", &[]),
            vec!["crate::real::A".to_string()],
            "an escaped-quote char literal must not leak and expose a fake use"
        );
    }

    #[test]
    fn external_scan_captures_external_heads_with_the_shared_resolution() {
        // The external scan is the exact mirror of the internal one: it captures a head
        // precisely when the internal scan would drop it as external, using one resolution.
        // A submodule's bare first segment is external; a leading `::` is explicitly external.
        let pairs = external_imports_with_importers(
            "use libc::c_int;\nuse ::winapi::HANDLE;\nuse crate::domain::Thing;",
            "crate::service",
            &[],
        );
        assert_eq!(
            pairs,
            vec![
                ("crate::service".to_string(), "libc".to_string()),
                ("crate::service".to_string(), "winapi".to_string()),
            ],
            "external heads captured, the internal `crate::…` import dropped: {pairs:?}"
        );
        // A bare first segment naming a crate-root module, at the crate root, is the internal
        // module (the sibling `mod` shadows the extern prelude) — NOT external. This is the
        // no-false-positive guarantee: the external scan must not observe it either.
        let shadowed =
            external_imports_with_importers("use libc::helper;", "crate", &["libc".to_string()]);
        assert!(
            shadowed.is_empty(),
            "a shadowed crate-root module is internal, not an external head: {shadowed:?}"
        );
        // `crate`/`self`/`super` are internal roots, never external heads.
        let internal = external_imports_with_importers(
            "use crate::a::B;\nuse self::x::Y;\nuse super::z::W;",
            "crate::m",
            &[],
        );
        assert!(
            internal.is_empty(),
            "internal roots yield no external heads: {internal:?}"
        );
        // A `use` inside a string literal or a macro body is stripped before scanning.
        let masked = external_imports_with_importers(
            "fn f() { let _s = \"use libc::c_int;\"; }\nmacro_rules! m { () => { use libc::c_void; }; }",
            "crate::service",
            &[],
        );
        assert!(
            masked.is_empty(),
            "a use inside a string or macro body is not an observed external head: {masked:?}"
        );
    }
}
