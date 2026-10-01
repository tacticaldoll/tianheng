//! The `use`-import scan: each `use` leaf of a file classified as an internal `crate::…` path, an external crate, or
//! neither — grouped and glob forms expanded, raw identifiers canonicalized, and every head read through the one
//! resolver the inline scan reads, so what a bare head names is the scope's answer rather than a rule of its own. A
//! `::*` glob is observed at its **base** module (`use a::b::*;` → `a::b`) and retains its glob shape, so
//! `must_not_import` can react fail-closed when that base is equal to or an ancestor of a forbidden module, and a
//! `use` inside an inline `mod name { … }` is attributed to that module. Pure token and path processing, no model
//! type.

use super::path_vocab::{PathSite, identity_modules, readable_module};
use super::resolve::{CrateScopes, Named, Namespace};
use super::scope_tree::ScopeTable;
#[cfg(test)]
use super::token_tree::Edition;
#[cfg(test)]
use super::token_tree::TokenTree;
use super::use_tree::{UseLeaf, UseStatement};

/// One normalized internal import path, retaining **which form** the source wrote: a glob so boundary
/// evaluation can distinguish a direct import from an ancestor-glob hazard, and a `{self}` leaf so it
/// can tell an import of a module from an import of something in it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ImportedPath {
    pub path: String,
    pub is_glob: bool,
    /// The source wrote the path as a `{self}` leaf — `use m::foo::{self};`, or `{self as f}`.
    ///
    /// Recorded because the normalized path cannot show it: a `{self}` leaf normalizes to its prefix
    /// module, so `use m::foo::{self};` and a bare `use m::foo;` are byte-identical afterward while
    /// binding different things.
    pub is_self_leaf: bool,
}

impl ImportedPath {
    /// Whether this import form can bind a **value** declared in the path's parent module.
    ///
    /// Only a plain leaf can. Both other forms are ruled out by the language, not by likelihood:
    ///
    /// - a **glob** imports the *contents* of the named module and never the name itself, so with
    ///   `mod foo` and `fn foo` both declared, `use m::foo::*;` then calling `foo()` is
    ///   `error[E0425]: cannot find function 'foo' in this scope`;
    /// - a **`{self}` leaf** imports the named module, so the same declarations give
    ///   `error[E0423]: expected function, found module 'foo'` while `foo::INSIDE` compiles.
    ///
    /// One question with two source facts behind it, named here so a reader of the inbound
    /// value-namespace reaction meets the language rule rather than two ad-hoc conditions — the second
    /// of which was missed when the first was added.
    pub(crate) fn can_bind_a_value(&self) -> bool {
        !self.is_glob && !self.is_self_leaf
    }

    #[cfg(test)]
    pub(crate) fn plain(path: impl Into<String>) -> Self {
        ImportedPath {
            path: path.into(),
            is_glob: false,
            is_self_leaf: false,
        }
    }

    /// A `{self}` leaf: the same path a plain import of this module would normalize to, with the form
    /// recorded — which is the whole distinction, since the paths are otherwise identical.
    #[cfg(test)]
    pub(crate) fn self_leaf(path: impl Into<String>) -> Self {
        ImportedPath {
            path: path.into(),
            is_glob: false,
            is_self_leaf: true,
        }
    }

    #[cfg(test)]
    pub(crate) fn glob(path: impl Into<String>) -> Self {
        ImportedPath {
            path: path.into(),
            is_glob: true,
            is_self_leaf: false,
        }
    }
}

impl std::ops::Deref for ImportedPath {
    type Target = str;
    fn deref(&self) -> &str {
        &self.path
    }
}

/// One file's scan read alone: its scope table and the resolver over it, which is all the file-alone entries below
/// read. A name another file's module declares, or a glob of it brings, is not in it, which is why production reads
/// every file of a compilation unit through [`super::symbol_scan::UnitScan`] instead.
#[cfg(test)]
fn file_alone(
    source: &str,
    current_module: &str,
    edition: Edition,
) -> Result<Vec<(String, UseTarget, bool, bool)>, String> {
    let tree = TokenTree::lex(source, edition);
    let table = ScopeTable::build(&tree, current_module, 0);
    let uses = file_uses(super::use_tree::use_statements(&tree), &table);
    classify_uses(&CrateScopes::new(vec![table], edition), 0, &uses)
}

/// Each internal import of one file read alone, paired with the **module that actually declares it** — inline-aware,
/// so a `use` inside an inline `mod inner { … }` is attributed to `{current_module}::inner`. Sorted and deduped by
/// `(importer, import)`.
#[cfg(test)]
pub(super) fn imports_with_importers(
    source: &str,
    current_module: &str,
    edition: Edition,
) -> Result<Vec<(String, ImportedPath)>, String> {
    Ok(internal_imports(file_alone(
        source,
        current_module,
        edition,
    )?))
}

/// The import PATHS of one file read alone, deduplicated and sorted — the paths-only reading the normalization
/// assertions of this module and of [`super`] are written against, derived from [`imports_with_importers`].
#[cfg(test)]
pub(super) fn imported_module_paths(
    source: &str,
    current_module: &str,
) -> Result<Vec<ImportedPath>, String> {
    let mut paths: Vec<ImportedPath> =
        imports_with_importers(source, current_module, Edition::Rust2021)?
            .into_iter()
            .map(|(_importer, import)| import)
            .collect();
    paths.sort();
    paths.dedup();
    Ok(paths)
}

/// Each importer module of one file read alone paired with the **external** crate it imports — the mirror of
/// [`imports_with_importers`]. Sorted and deduped by `(importer, external crate)`.
#[cfg(test)]
fn external_imports_with_importers(
    source: &str,
    current_module: &str,
    edition: Edition,
) -> Result<Vec<(String, String)>, String> {
    Ok(external_imports(file_alone(
        source,
        current_module,
        edition,
    )?))
}

/// What a `use` leaf's path names: a module path of this crate, or an external crate by its name. A path that
/// names neither — a `super` past the crate root, `::self`, a block-local item — is no target at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum UseTarget {
    Internal(String),
    External(String),
}

/// Classify a `use` leaf's path `written` in scope `scope` of table `t` through the one resolver the inline scan
/// reads, so an import and a call agree on what a head names. A crate-rooted, `self` or `super` path is internal;
/// a bare head names what the scope binds or declares under it — a sibling or child `mod`, an item, another import,
/// a name a glob brings — before the extern prelude, as a uniform path does in edition 2018 and later, and in
/// edition 2015 a `use` path is read from the crate root. Only the head is read through a binding: a path is
/// internal as the head names it, and a re-export the rest runs through is not followed, since an import of
/// `crate::support::X` imports `crate::support` whatever `X` re-exports. A head nothing binds is an external crate,
/// as is a `::`-rooted one in edition 2018 and later.
fn classify(
    scopes: &CrateScopes,
    t: usize,
    scope: u32,
    written: &str,
    ns: Namespace,
) -> Result<Vec<UseTarget>, String> {
    let head_of = |path: &str| {
        let path = path.trim_start_matches("::");
        path.split_once("::")
            .map_or(path, |(head, _)| head)
            .to_string()
    };
    Ok(
        match scopes.head_names(t, scope, written, PathSite::Use, ns) {
            Named::Paths(paths) => paths
                .into_iter()
                .map(|path| {
                    if path == "crate" || path.starts_with("crate::") {
                        UseTarget::Internal(readable_module(&path))
                    } else {
                        UseTarget::External(head_of(&path))
                    }
                })
                .collect(),
            Named::External(path) => vec![UseTarget::External(head_of(&path))],
            Named::Unbound { head, .. } => vec![UseTarget::External(head)],
            Named::Local | Named::Invalid => Vec::new(),
            Named::PastCap(refusal) => return Err(refusal),
        },
    )
}

/// Each leaf of one file's `use` statements, classified: the importer's identity, what the leaf names, and whether
/// it is a glob base or a `{self}` leaf.
pub(super) fn classify_uses(
    scopes: &CrateScopes,
    t: usize,
    uses: &[FileUse],
) -> Result<Vec<(String, UseTarget, bool, bool)>, String> {
    let mut out = Vec::new();
    for file_use in uses {
        let leaves = file_use.leaves.as_ref().map_err(Clone::clone)?;
        for leaf in leaves {
            let (written, is_glob, is_self_leaf, ns) = match leaf {
                UseLeaf::Name { path, .. } => (path, false, false, Namespace::Either),
                UseLeaf::Glob(base) => (base, true, false, Namespace::Type),
                UseLeaf::SelfLeaf { module, .. } => (module, false, true, Namespace::Type),
                UseLeaf::Empty(_) => continue,
            };
            for target in classify(scopes, t, file_use.scope, written, ns)? {
                out.push((file_use.importer.clone(), target, is_glob, is_self_leaf));
            }
        }
    }
    Ok(out)
}

/// One `use` statement of a file: the identity of the module it is written in, the scope it stands in, and its
/// leaves, or the refusal of a tree nested past the parser's cap.
pub(super) struct FileUse {
    pub importer: String,
    pub scope: u32,
    pub leaves: Result<Vec<UseLeaf>, String>,
}

/// Every `use` statement of the file `tree` holds, read against its scope `table`.
pub(super) fn file_uses(statements: Vec<UseStatement>, table: &ScopeTable) -> Vec<FileUse> {
    let identities = identity_modules(table.scopes.iter().map(|scope| scope.module.as_str()));
    statements
        .into_iter()
        .map(|statement| {
            let scope = table.scope_at(statement.at);
            FileUse {
                importer: identities[&table.scopes[scope as usize].module].clone(),
                scope,
                leaves: statement.leaves,
            }
        })
        .collect()
}

/// Internal imports of classified leaves, paired with their importer, sorted and deduplicated.
pub(super) fn internal_imports(
    classified: Vec<(String, UseTarget, bool, bool)>,
) -> Vec<(String, ImportedPath)> {
    let mut pairs: Vec<(String, ImportedPath)> = classified
        .into_iter()
        .filter_map(|(importer, target, is_glob, is_self_leaf)| match target {
            UseTarget::Internal(path) => Some((
                importer,
                ImportedPath {
                    path,
                    is_glob,
                    is_self_leaf,
                },
            )),
            UseTarget::External(_) => None,
        })
        .collect();
    pairs.sort();
    pairs.dedup();
    pairs
}

/// External crates of classified leaves, paired with their importer, sorted and deduplicated.
pub(super) fn external_imports(
    classified: Vec<(String, UseTarget, bool, bool)>,
) -> Vec<(String, String)> {
    let mut pairs: Vec<(String, String)> = classified
        .into_iter()
        .filter_map(|(importer, target, ..)| match target {
            UseTarget::External(head) => Some((importer, head)),
            UseTarget::Internal(_) => None,
        })
        .collect();
    pairs.sort();
    pairs.dedup();
    pairs
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
        let imports = imported_module_paths(source, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::a::b")),
            "{imports:?}"
        );
        assert!(
            imports.contains(&ImportedPath::plain("crate::a::c::d")),
            "{imports:?}"
        );
        assert!(
            imports.contains(&ImportedPath::plain("crate::sibling::X")),
            "{imports:?}"
        );
        assert!(
            imports.contains(&ImportedPath::plain("crate::kernel::inner::Y")),
            "{imports:?}"
        );
        assert!(
            imports.contains(&ImportedPath::glob("crate::z")),
            "{imports:?}"
        );
        assert!(!imports.iter().any(|p| p.contains("serde")), "{imports:?}");
        assert!(
            !imports.iter().any(|p| p.contains("ignored")),
            "{imports:?}"
        );
    }

    /// A block comment wedged between the `use` keyword and its path does not fuse them: a comment
    /// is no token, so in `use/*re-export*/crate::secret::Thing;` `use` and `crate` stay two tokens
    /// and the import is observed.
    #[test]
    fn a_block_comment_between_use_and_its_path_does_not_fuse_the_keyword() {
        assert_eq!(
            imported_module_paths("use/*re-export*/crate::secret::Thing;", "crate").unwrap(),
            vec![ImportedPath::plain("crate::secret::Thing")],
            "a block comment after `use` must not swallow the import",
        );
    }

    /// `use crate::config::{self as cfg, Setting};` imports the
    /// module `crate::config` (under the alias) plus `crate::config::Setting`. The `self as cfg`
    /// form must resolve to the prefix module, not leave a phantom `crate::config::self` segment.
    ///
    /// The resolved PATH is what it always was; what is added is that the `{self}` form is now
    /// recorded on the import rather than erased by normalization. It has to be: a `{self}` leaf
    /// binds the module, a plain leaf can bind a value beside it, and the two normalize to the same
    /// string — so the inbound value-namespace reaction could not tell them apart and reacted to
    /// both (see `ImportedPath::can_bind_a_value`).
    #[test]
    fn a_self_alias_in_a_use_group_resolves_to_the_prefix_module() {
        let source = "use crate::config::{self as cfg, Setting};";
        let imports = imported_module_paths(source, "crate").unwrap();
        assert_eq!(
            imports,
            vec![
                ImportedPath::self_leaf("crate::config"),
                ImportedPath::plain("crate::config::Setting"),
            ],
            "a `self as alias` in a group resolves to the prefix module: {imports:?}"
        );
        assert_eq!(
            imported_module_paths("use crate::config::{self as cfg};", "crate").unwrap(),
            vec![ImportedPath::self_leaf("crate::config")],
        );
        assert!(
            !imports[1].is_self_leaf && imports[1].can_bind_a_value(),
            "a sibling leaf keeps the plain form: {imports:?}"
        );
    }

    /// `crate::a` has one ancestor (`crate`); `super::super` over-pops past the root.
    /// Such a path names no internal module (and does not compile), so it must not be
    /// observed — never a malformed root-less path like "other::X".
    #[test]
    fn super_past_the_crate_root_is_not_an_internal_module() {
        let over = "use super::super::other::X;\n";
        assert!(
            imported_module_paths(over, "crate::a").unwrap().is_empty(),
            "over-popped super must yield no import: {:?}",
            imported_module_paths(over, "crate::a").unwrap()
        );
        let ok = "use super::other::X;\n";
        assert_eq!(
            imported_module_paths(ok, "crate::a").unwrap(),
            vec![ImportedPath::plain("crate::other::X")],
        );
    }

    /// Scanner ignores comments, strings, char literals, and lifetimes without losing following imports.
    #[test]
    fn scanner_ignores_comments_and_string_literals() {
        let url = r#"let u = "http://example.com"; use crate::real::A;"#;
        let imports = imported_module_paths(url, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::real::A")),
            "{imports:?}"
        );

        let in_string = r#"let s = "use crate::ghost::Z;";"#;
        assert!(
            imported_module_paths(in_string, "crate::kernel")
                .unwrap()
                .is_empty(),
            "a use inside a string must not be observed"
        );

        let quote_char = r#"let q = '"'; use crate::real::B;"#;
        let imports = imported_module_paths(quote_char, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::real::B")),
            "{imports:?}"
        );

        let lifetime = "fn f<'a>(x: &'a str) {} use crate::a::b;";
        let imports = imported_module_paths(lifetime, "crate::kernel").unwrap();
        assert_eq!(
            imports,
            vec![ImportedPath::plain("crate::a::b")],
            "{imports:?}"
        );
    }

    /// Rust nests block comments. Commenting out code that itself contains a
    /// `/* */` must not let the inner `*/` re-expose the rest as live code: a
    /// `use` inside the (nested) comment must not be observed, while a real `use`
    /// after the outer close still is.
    #[test]
    fn scanner_handles_nested_block_comments() {
        let source = r#"
            /*
            fn old() {
                /* tweak later */
                use crate::legacy::Thing;
            }
            */
            use crate::current::A;
        "#;
        let imports = imported_module_paths(source, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::current::A")),
            "the real use after the nested comment must be observed: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("legacy")),
            "a use inside a nested block comment must not be observed: {imports:?}"
        );
    }

    /// A bare `use kernel::…` at the crate root, where `mod kernel;` is declared, names that module rather than an
    /// external crate, so it resolves to `crate::kernel::…`. A first segment the scope does not bind is external, and
    /// without the `mod kernel;` the same `use` is external too.
    #[test]
    fn scanner_resolves_root_relative_bare_use() {
        let source = "use kernel::Thing; use serde::Deserialize;";
        let imports = imported_module_paths(&format!("mod kernel;\n{source}"), "crate").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::kernel::Thing")),
            "a bare use of a crate-root module must resolve to crate::…: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("serde")),
            "an unknown first segment stays external: {imports:?}"
        );
        assert!(
            imported_module_paths(source, "crate")
                .unwrap()
                .iter()
                .all(|p| !p.contains("kernel")),
            "with no `mod kernel` in scope, the bare path is treated as external"
        );
    }

    /// `use ::serde::…` is an explicit external/global path: the leading `::` bypasses
    /// the local-module shadow, so it is external even when a crate-root module shares
    /// the name.
    #[test]
    fn scanner_treats_leading_colon_path_as_external() {
        assert!(
            imported_module_paths("mod serde;\nuse ::serde::Deserialize;", "crate")
                .unwrap()
                .is_empty(),
            "a leading-:: path must be external even when its head matches a root module"
        );
        assert!(
            imported_module_paths("mod serde;\nuse serde::Deserialize;", "crate")
                .unwrap()
                .contains(&ImportedPath::plain("crate::serde::Deserialize")),
            "a bare head matching a root module resolves locally (shadowing rule)"
        );
    }

    /// A bare head is read in the scope it is written in, as a uniform path is: a crate-root module binds it at the
    /// crate root, a submodule's own child binds it in that submodule, and a head a submodule's scope does not bind —
    /// `serde`, with no `mod serde;` there — reaches the extern prelude and is external. Reading that last one as
    /// `crate::serde::…` would be a false positive that fails a module boundary.
    #[test]
    fn scanner_resolves_a_bare_head_in_the_scope_it_is_written_in() {
        assert!(
            imported_module_paths("mod serde;\nuse serde::Value;", "crate")
                .unwrap()
                .contains(&ImportedPath::plain("crate::serde::Value")),
            "at the crate root, a bare crate-root-module path resolves locally"
        );
        assert!(
            imported_module_paths("use serde::Value;", "crate::sub")
                .unwrap()
                .is_empty(),
            "in a submodule, a bare first segment its scope does not bind is external"
        );
        assert_eq!(
            imported_module_paths("mod inner;\nuse inner::X;", "crate::sub").unwrap(),
            vec![ImportedPath::plain("crate::sub::inner::X")],
            "a submodule's own child is in its scope, so a bare head naming it is that module, as a uniform path is"
        );
    }

    /// A non-ASCII identifier is one token, so a non-ASCII module path is observed intact.
    #[test]
    fn scanner_preserves_non_ascii_module_paths() {
        let source = "use crate::café::Item;";
        let imports = imported_module_paths(source, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::café::Item")),
            "{imports:?}"
        );
    }

    /// A `use …;` inside a raw string (any hash count) is not an import.
    #[test]
    fn scanner_ignores_raw_and_byte_strings() {
        for src in [
            r##"let s = r"use crate::ghost::Z;";"##,
            r##"let s = r#"use crate::ghost::Z;"#;"##,
            r##"let s = br#"use crate::ghost::Z;"#;"##,
            r#"let s = b"use crate::ghost::Z;";"#,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel")
                    .unwrap()
                    .is_empty(),
                "a use inside a (raw/byte) string must not be observed: {src}"
            );
        }

        let tricky = r####"let s = r##"http://x "# inside"##; use crate::real::C;"####;
        let imports = imported_module_paths(tricky, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::real::C")),
            "{imports:?}"
        );

        let idents = "let r = 1; let b = 2; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel").unwrap(),
            vec![ImportedPath::plain("crate::real::D")]
        );
    }

    /// The desync guard: a raw C-string with an odd number of inner unescaped `"` (raw
    /// strings do not escape) must not swallow a following `use`.
    #[test]
    fn scanner_ignores_raw_c_strings() {
        for src in [
            r##"let s = cr"use crate::ghost::Z;";"##,
            r##"let s = cr#"use crate::ghost::Z;"#;"##,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel")
                    .unwrap()
                    .is_empty(),
                "a use inside a raw C-string must not be observed: {src}"
            );
        }

        let tricky = r##"let s = cr#"a"b"#; use crate::real::C;"##;
        let imports = imported_module_paths(tricky, "crate::kernel").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::real::C")),
            "a use after a raw C-string with an odd inner-quote count must be observed: {imports:?}"
        );

        let cstr = r#"let s = c"use crate::ghost::Z;"; use crate::real::E;"#;
        assert_eq!(
            imported_module_paths(cstr, "crate::kernel").unwrap(),
            vec![ImportedPath::plain("crate::real::E")]
        );

        let idents = "let c = 1; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel").unwrap(),
            vec![ImportedPath::plain("crate::real::D")]
        );
    }

    /// A `use` written inside a macro — a `macro_rules!` definition OR a macro
    /// invocation — is a macro-generated import (out of scope): it must not be
    /// observed. A real `use` outside the macro still is.
    #[test]
    fn use_inside_a_macro_body_is_not_observed() {
        let source = r#"
            macro_rules! m {
                () => { use crate::ghost::Thing; };
                ($x:tt) => {{ use crate::ghost::Other; }};
            }
            with_imports! { use crate::ghost::FromInvocation; }
            use crate::real::A;
        "#;
        let imports = imported_module_paths(source, "crate").unwrap();
        assert_eq!(
            imports,
            vec![ImportedPath::plain("crate::real::A")],
            "macro definition and invocation bodies are skipped; the real use is kept: {imports:?}"
        );
        for body in [
            "macro_rules! m { () => { use crate::ghost::T; }; }",
            "macro_rules! m ( () => { use crate::ghost::T; }; )",
            "macro_rules! m [ () => { use crate::ghost::T; }; ]",
            "some_macro! { use crate::ghost::T; }",
            "some_macro!( use crate::ghost::T; )",
            "some_macro![ use crate::ghost::T; ]",
        ] {
            assert!(
                imported_module_paths(body, "crate").unwrap().is_empty(),
                "no import observed from a macro body: {body}"
            );
        }
        let not_macros = "let _ = a != b; let _ = !flag; use crate::real::B;";
        assert!(
            imported_module_paths(not_macros, "crate")
                .unwrap()
                .contains(&ImportedPath::plain("crate::real::B")),
            "`!=` / unary `!` must not be treated as a macro invocation"
        );
    }

    /// `-> impl Trait + use<…>` (stable Rust) is a precise-capturing bound, not an import.
    /// The scanner must not treat it as a `use` statement and consume to the next `;`, which
    /// would swallow the real `use` that follows — a false negative that silently disables the
    /// module boundary. Cover the empty, parameterized, whitespace, and comment forms.
    #[test]
    fn a_precise_capturing_use_bound_does_not_swallow_the_following_use() {
        for header in [
            "fn iter() -> impl Iterator<Item = u8> + use<> { std::iter::empty() }",
            "fn iter<'a, T>() -> impl Iterator<Item = &'a T> + use<'a, T> { loop {} }",
            "fn iter() -> impl Iterator<Item = u8> + use <> { std::iter::empty() }",
            "fn iter() -> impl Iterator<Item = u8> + use /*c*/ <> { std::iter::empty() }",
        ] {
            let src = format!("{header}\nuse crate::forbidden::Thing;");
            assert_eq!(
                imported_module_paths(&src, "crate").unwrap(),
                vec![ImportedPath::plain("crate::forbidden::Thing")],
                "the `use<…>` bound must be skipped so the following real use is observed: {header:?}"
            );
        }
    }

    /// Control: a plain `use` is unaffected. And a `use<>` bound as the file's final token
    /// (no trailing `;`) must not panic (bounds-safe peek) and must not drop the real `use`
    /// that precedes it.
    #[test]
    fn a_use_bound_as_the_last_token_neither_panics_nor_drops_a_preceding_use() {
        assert_eq!(
            imported_module_paths("use crate::x::Y;", "crate").unwrap(),
            vec![ImportedPath::plain("crate::x::Y")],
            "a plain use is unaffected by the bound-skip"
        );
        assert_eq!(
            imported_module_paths("use crate::x::Y;\nfn f() -> impl Sized + use<>", "crate",)
                .unwrap(),
            vec![ImportedPath::plain("crate::x::Y")],
            "a trailing use<> bound must not drop the preceding real use or panic"
        );
    }

    /// A `self`/`super` import inside an inline `mod inner { … }` resolves against the
    /// inline submodule, not the file's module: `self` -> crate::a::inner::…, and
    /// `super` from crate::a::inner -> crate::a.
    ///
    /// A bare first segment inside an inline submodule is read in that submodule's scope, so a crate-root module's
    /// name there is external unless the submodule binds it, even when the file is the crate root (the enclosing
    /// module is crate::inner, not crate).
    #[test]
    fn use_inside_an_inline_module_is_attributed_to_that_module() {
        let source = "mod inner { use self::leaf::Thing; use super::sibling::X; }";
        let imports = imported_module_paths(source, "crate::a").unwrap();
        assert!(
            imports.contains(&ImportedPath::plain("crate::a::inner::leaf::Thing")),
            "self must resolve against the inline submodule: {imports:?}"
        );
        assert!(
            imports.contains(&ImportedPath::plain("crate::a::sibling::X")),
            "super from the inline submodule resolves to crate::a: {imports:?}"
        );
        let bare = imported_module_paths("mod kernel;\nmod inner { use kernel::Thing; }", "crate")
            .unwrap();
        assert!(
            bare.is_empty(),
            "a bare use of a crate-root module inside an inline submodule that binds no such name is external: {bare:?}"
        );
        assert_eq!(
            imported_module_paths("use self::leaf::Thing;", "crate::a").unwrap(),
            vec![ImportedPath::plain("crate::a::leaf::Thing")]
        );
    }

    /// `'\''` is one literal token, so its escaped quote opens no string: the following string's
    /// fake `use` stays inside that string's literal token and the real `use` after it is observed.
    #[test]
    fn escaped_quote_char_literal_is_consumed_whole() {
        let src = r#"let _q = '\''; let _s = "use crate::ghost::Z;"; use crate::real::A;"#;
        assert_eq!(
            imported_module_paths(src, "crate::kernel").unwrap(),
            vec![ImportedPath::plain("crate::real::A")],
            "an escaped-quote char literal must not leak and expose a fake use"
        );
    }

    /// The external scan is the exact mirror of the internal one: it captures a head
    /// precisely when the internal scan would drop it as external, using one resolution.
    /// In edition 2018, a bare first segment the importing scope does not bind is external, and a leading `::` is
    /// external whatever the scope binds. A bare first segment naming a crate-root module, at the crate root, is the
    /// internal module (the sibling `mod` shadows the extern prelude) — NOT external.
    #[test]
    fn external_scan_captures_external_heads_with_the_shared_resolution() {
        let pairs = external_imports_with_importers(
            "use libc::c_int;\nuse ::winapi::HANDLE;\nuse crate::domain::Thing;",
            "crate::service",
            Edition::Rust2018,
        )
        .unwrap();
        assert_eq!(
            pairs,
            vec![
                ("crate::service".to_string(), "libc".to_string()),
                ("crate::service".to_string(), "winapi".to_string()),
            ],
            "external heads captured, the internal `crate::…` import dropped: {pairs:?}"
        );
        let shadowed = external_imports_with_importers(
            "mod libc;\nuse libc::helper;",
            "crate",
            Edition::Rust2018,
        )
        .unwrap();
        assert!(
            shadowed.is_empty(),
            "a shadowed crate-root module is internal, not an external head: {shadowed:?}"
        );
        let internal = external_imports_with_importers(
            "use crate::a::B;\nuse self::x::Y;\nuse super::z::W;",
            "crate::m",
            Edition::Rust2018,
        )
        .unwrap();
        assert!(
            internal.is_empty(),
            "internal roots yield no external heads: {internal:?}"
        );
        let masked = external_imports_with_importers(
            "fn f() { let _s = \"use libc::c_int;\"; }\nmacro_rules! m { () => { use libc::c_void; }; }",
            "crate::service",
        Edition::Rust2018,
    ).unwrap();
        assert!(
            masked.is_empty(),
            "a use inside a string or macro body is not an observed external head: {masked:?}"
        );
    }

    /// Transparent control-flow macros (`cfg_if!`) wrap human-authored items without
    /// transforming identities; their bodies are preserved so enclosed `use` declarations
    /// are observed statically (closing the false-negative gap).
    #[test]
    fn cfg_if_macro_body_is_read_and_observed() {
        let src = r#"
            cfg_if::cfg_if! {
                if #[cfg(unix)] {
                    use crate::unix_module::UnixThing;
                } else {
                    use crate::win_module::WinThing;
                }
            }
        "#;
        let paths = imported_module_paths(src, "crate").unwrap();
        assert_eq!(
            paths,
            vec![
                ImportedPath::plain("crate::unix_module::UnixThing"),
                ImportedPath::plain("crate::win_module::WinThing"),
            ],
            "imports inside cfg_if! macro bodies are observed: {paths:?}"
        );
    }
}
