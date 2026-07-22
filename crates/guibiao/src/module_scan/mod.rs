//! The source scanner: the functional core's observation source for module boundaries.
//! Given a crate's `src/`, it lists `.rs` files ([`fs_walk`]), walks the `mod`-declared
//! module graph reachable from the crate root ([`reachability`]), and extracts the
//! `crate::…` module paths a file imports via `use` ([`use_scan`]) — comments, string
//! literals, and macro bodies stripped ([`lexer`]), so only real, file-based, reachable
//! imports are observed (PROJECT.md). The shared path vocabulary (raw-identifier
//! canonicalization, `::`-containment, and the `mod`-keyword test) lives in [`path_vocab`],
//! the small foundation the `use`-scan and the module walk both stand on so neither depends
//! laterally on the other. Pure string, byte, and path processing: it depends on no model
//! type, only `std`.

mod fs_walk;
mod lexer;
mod path_vocab;
mod reachability;
mod symbol_scan;
mod use_scan;

pub(crate) use fs_walk::rust_files;
pub(crate) use path_vocab::{canonical_module_path, package_name_to_import_ident, path_within};
pub(crate) use reachability::{governed_files, reachable_modules};
pub(crate) use symbol_scan::{InlineFinding, inline_symbol_findings};
pub(crate) use use_scan::{
    external_imports_with_importers, imported_module_paths, imports_with_importers,
};

// Cross-cutting tests assert invariants that span the dimensions the scanner is split into:
// the `use`-scan ([`use_scan`]) and the declaration walk ([`reachability`]) must share exactly
// one lexical-hygiene pass ([`lexer`]) and one canonicalization ([`path_vocab`]), so both agree on
// what a macro body, a raw identifier, or a Unicode identifier is. A single-dimension test lives
// beside its own module; these deliberately reach across, so they stay at the seam.
#[cfg(test)]
mod tests {
    use super::lexer::{keyword_starts_at, strip_macro_bodies};
    use super::path_vocab::canonical_segment;
    use super::reachability::declared_modules;
    use super::{canonical_module_path, imported_module_paths};

    #[test]
    fn scanner_does_not_panic_on_odd_input() {
        // Truncated / malformed inputs must never panic (robustness over precision).
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
            let _ = imported_module_paths(src, "crate::kernel", &[]);
            let _ = declared_modules(src);
            let _ = strip_macro_bodies(src);
        }
    }

    #[test]
    fn a_raw_identifier_macro_name_does_not_leak_its_body() {
        // A `macro_rules!` with a raw-identifier name (`r#try`): `#` is not an identifier byte, so
        // the name scan must tolerate the `r#` prefix — otherwise it stops at `r`, fails to locate
        // the body, leaves it unstripped, and wrongly observes the `use`/`mod` inside the
        // never-invoked definition (a false positive). Both scans share the macro-body stripping.
        let with_use = r#"
            macro_rules! r#try {
                () => { use crate::ghost::Thing; };
            }
            use crate::real::A;
        "#;
        assert_eq!(
            imported_module_paths(with_use, "crate", &[]),
            vec!["crate::real::A".to_string()],
            "a `use` inside a raw-identifier macro definition is not observed"
        );
        let with_mod = "macro_rules! r#try { () => { mod ghost; }; }\nmod real;";
        assert_eq!(
            declared_modules(with_mod),
            vec!["real".to_string()],
            "a `mod` inside a raw-identifier macro definition is not a declared module"
        );
    }

    #[test]
    fn keyword_detection_does_not_fire_inside_a_unicode_identifier() {
        // Rust allows non-ASCII identifiers. `use貓` / `mod貓` are single identifiers, so
        // the leading `use` / `mod` is NOT a keyword: the byte after it (the lead byte of
        // `貓`, >= 0x80) is an identifier byte. A regression guard against an ASCII-only
        // `is_ident_byte` splitting the identifier and firing a false keyword.
        assert!(!keyword_starts_at("use貓;".as_bytes(), 0, b"use"));
        assert!(!keyword_starts_at("mod貓 {}".as_bytes(), 0, b"mod"));
        // The genuine keyword (followed by whitespace) is still detected.
        assert!(keyword_starts_at("use 貓;".as_bytes(), 0, b"use"));
        // And nothing is observed as an import or a declared module from the identifier.
        assert!(imported_module_paths("fn use貓() {}", "crate", &[]).is_empty());
        assert!(declared_modules("fn mod貓() {}").is_empty());
    }

    #[test]
    fn raw_identifiers_are_canonicalized() {
        // `mod r#type;` compiles to `type.rs`, so the `mod` token, the `use` path, and
        // the file path must all reduce to the same identity (`type`).
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
            imported_module_paths("use crate::r#type::Thing;", "crate", &[]),
            vec!["crate::type::Thing".to_string()],
            "a raw-identifier use path is canonicalized to its plain form"
        );
    }
}
