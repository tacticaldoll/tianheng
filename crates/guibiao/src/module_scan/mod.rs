//! The source scanner: the functional core's observation source for module boundaries.
//! Given a crate's `src/`, it lists `.rs` files ([`fs_walk`]), walks the `mod`-declared
//! module graph reachable from the crate root ([`reachability`]),
//! extracts the `crate::…` module paths a file imports via `use` ([`use_scan`]), and judges inline
//! symbol paths ([`symbol_scan`]).
//!
//! The import scan and the inline scan read a file through modules in one order, each importing only modules before
//! it: the [`token_tree`], the one reader of a file's bytes; the shared path vocabulary ([`path_vocab`]:
//! raw-identifier canonicalization, `::`-containment, `self`/`super` folding); the item-header grammar
//! ([`item_head`]); the `use` trees ([`use_tree`]) and path [`occurrence`]s read over it; the [`scope_tree`]; the one
//! [`resolve`]r; and the [`glob_hazard`].
//! [`fs_walk`] lists the file system, [`reachability`] probes it for the files a declaration may name, and
//! [`source_texts`] reads it, each source path once per evaluation, for every reader of a file's text; every other
//! module is pure string, token, and path processing. It depends on no model type but the finding the inline scan
//! reports. Above those readers sits [`evaluation`]: one evaluation's scan of each compiled root,
//! built once and shared by every module boundary judged over that root.

mod evaluation;
mod fs_walk;
mod glob_hazard;
mod item_head;
mod occurrence;
mod path_vocab;
mod reachability;
mod resolve;
mod scope_tree;
mod source_texts;
mod symbol_scan;
mod token_tree;
mod use_scan;
mod use_tree;

pub(crate) use evaluation::{EvaluationScans, RootScan};
pub(crate) use fs_walk::rust_files;
pub(crate) use path_vocab::{
    PrefixRoot, SymbolPrefix, canonical_module_path, canonical_module_spelling,
    canonical_symbol_path_spelling, package_name_to_import_ident, path_within, sysroot_crate,
};
pub(crate) use reachability::{governed_files, names_crate_by_path_alone, reachable_modules};
#[cfg(test)]
pub(crate) use scope_tree::take_table_builds;
pub(crate) use symbol_scan::{InlineFinding, UnitScan};
pub(crate) use token_tree::Edition;
pub(crate) use use_scan::ImportedPath;

/// Cross-cutting tests assert invariants that span the readers the scanner is split into: the
/// `use`-scan ([`use_scan`]) and the declaration walk ([`reachability`]) read through one
/// [`token_tree`] and one canonicalization ([`path_vocab`]), and these hold them to one answer on what
/// a macro body, a raw identifier, or a Unicode identifier is.
#[cfg(test)]
mod tests {
    use super::path_vocab::canonical_segment;
    use super::reachability::declared_modules;
    use super::token_tree::{Edition, Kind, TokenTree};
    use super::use_scan::{imported_module_paths, imports_with_importers};
    use super::{ImportedPath, canonical_module_path};

    /// Truncated or malformed inputs must never panic.
    #[test]
    fn scanner_does_not_panic_on_odd_input() {
        for src in [
            "r#\"unterminated raw string",
            "\"unterminated string",
            "/* unterminated block",
            "'",
            "r",
            "use ",
            "use crate::",
            "mod ",
            "mod foo",
            "}",
            "",
        ] {
            let _ = imported_module_paths(src, "crate::kernel").unwrap();
            let _ = declared_modules(src);
        }
    }

    /// A `macro_rules!` with a raw-identifier name (`r#try`) is one macro node like any other,
    /// so the `use` or `mod` items its body holds are not observed.
    #[test]
    fn a_raw_identifier_macro_name_does_not_leak_its_body() {
        let with_use = r#"
            macro_rules! r#try {
                () => { use crate::ghost::Thing; };
            }
            use crate::real::A;
        "#;
        assert_eq!(
            imported_module_paths(with_use, "crate").unwrap(),
            vec![ImportedPath::plain("crate::real::A")],
            "a `use` inside a raw-identifier macro definition is not observed"
        );
        let with_mod = "macro_rules! r#try { () => { mod ghost; }; }\nmod real;";
        assert_eq!(
            declared_modules(with_mod),
            vec!["real".to_string()],
            "a `mod` inside a raw-identifier macro definition is not a declared module"
        );
    }

    /// Rust allows non-ASCII identifiers: `use貓` / `mod貓` are single identifiers, not keywords.
    #[test]
    fn keyword_detection_does_not_fire_inside_a_unicode_identifier() {
        let first = |source: &str| {
            let tree = TokenTree::lex(source, Edition::Rust2018);
            (tree.kind(0), tree.text(0).to_string())
        };
        assert_eq!(first("use貓;"), (Kind::Ident, "use貓".to_string()));
        assert_eq!(first("mod貓 {}"), (Kind::Ident, "mod貓".to_string()));
        assert_eq!(first("use 貓;"), (Kind::Keyword, "use".to_string()));
        assert!(
            imports_with_importers("fn use貓() {}", "crate", Edition::Rust2018)
                .unwrap()
                .is_empty()
        );
        assert!(declared_modules("fn mod貓() {}").is_empty());
    }

    /// `mod r#type;` compiles to `type.rs`, so `mod`, `use`, and file path all reduce to `type`.
    #[test]
    fn raw_identifiers_are_canonicalized() {
        assert_eq!(canonical_segment("r#type"), "type");
        assert_eq!(canonical_segment("type"), "type");
        assert_eq!(
            canonical_module_path("crate::r#type::r#mod"),
            "crate::type::mod"
        );
        assert_eq!(
            declared_modules("pub mod r#type;"),
            vec!["type".to_string()]
        );
        assert_eq!(
            imported_module_paths("use crate::r#type::Thing;", "crate").unwrap(),
            vec![ImportedPath::plain("crate::type::Thing")],
            "a raw-identifier use path is canonicalized to its plain form"
        );
    }
}
