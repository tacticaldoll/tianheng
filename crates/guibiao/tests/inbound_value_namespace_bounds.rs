//! The inbound value-namespace reaction reacts to a **value binding** and to nothing else.
//!
//! `resolve_import_module` reads only the path, so `use m::foo;` where `m` declares both `mod foo` and
//! `fn foo` resolves to the descendant `m::foo` and would miss that the import also reaches `m` — the
//! false negative closed in `c4f64aa`. That closure consults the value namespace, and these are the
//! shapes it must NOT mistake for one. Each was a real false positive when this file was written.
use guibiao::{Constitution, ModuleBoundary, Outcome, check};
use std::path::{Path, PathBuf};

struct Probe(PathBuf);

impl Probe {
    fn new(label: &str, files: &[(&str, &str)]) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static N: AtomicU32 = AtomicU32::new(0);
        let dir = std::env::temp_dir().join(format!(
            "guibiao-inbound-value-{label}-{}-{}",
            std::process::id(),
            N.fetch_add(1, Ordering::Relaxed)
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("create src");
        std::fs::write(
            dir.join("Cargo.toml"),
            "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\n",
        )
        .expect("write manifest");
        for (relative, body) in files {
            let path = dir.join("src").join(relative);
            std::fs::create_dir_all(path.parent().expect("parent")).expect("create parent");
            std::fs::write(path, body).expect("write source");
        }
        Self(dir)
    }

    fn manifest(&self) -> PathBuf {
        self.0.join("Cargo.toml")
    }
}

impl Drop for Probe {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// An inbound `Shallow` boundary on `crate::protected`, the one cell where the two readings differ.
fn shallow_inbound() -> Constitution {
    Constitution::new("inbound-value").boundary(
        ModuleBoundary::in_crate("x")
            .module("crate::protected")
            .must_only_be_imported_by(["crate::facade"])
            .depth(xuanji::ScanDepth::Shallow)
            .because("only the facade may reach into protected"),
    )
}

fn findings(manifest: &Path) -> Vec<String> {
    match check(&shallow_inbound(), manifest) {
        Outcome::Violations(report) => report
            .violations
            .iter()
            .map(|v| v.finding.clone())
            .collect(),
        Outcome::Clean => Vec::new(),
        other => panic!("expected a judgement, got {other:?}"),
    }
}

/// The reaction the two bounds below must not disturb: a real value binding still reacts.
#[test]
fn a_real_value_binding_still_reacts() {
    let probe = Probe::new(
        "binding",
        &[
            ("lib.rs", "pub mod protected;\npub mod consumer;\n"),
            ("protected.rs", "pub mod foo;\npub fn foo() -> u8 { 7 }\n"),
            ("protected/foo.rs", "pub const INSIDE: u8 = 1;\n"),
            ("consumer.rs", "use crate::protected::foo;\n"),
        ],
    );
    assert_eq!(
        findings(&probe.manifest()),
        vec!["crate::consumer".to_string()],
        "`use protected::foo;` binds the `fn foo` declared in the protected module, so it reaches it"
    );
}

/// A **glob** cannot bind a value, so it must not react.
///
/// `use_scan` stores a glob at its base module with `::*` stripped, so `use protected::foo::*;` arrives
/// at the predicate byte-identical to a plain `use protected::foo;` — same path, same single-segment
/// leaf, same declared `fn foo`. All three original conditions passed and it reacted.
///
/// Verified against rustc rather than reasoned: with both `mod foo` and `pub fn foo` declared,
/// `use m::foo;` compiles with `foo() + foo::INSIDE` (both namespaces bound), while `use m::foo::*;`
/// fails `error[E0425]: cannot find function 'foo' in this scope` — a glob imports the *contents* of the
/// module `foo`, so the `fn foo` plays no part and the import reaches only the descendant.
#[test]
fn a_glob_import_does_not_bind_a_value_and_does_not_react() {
    let probe = Probe::new(
        "glob",
        &[
            ("lib.rs", "pub mod protected;\npub mod consumer;\n"),
            ("protected.rs", "pub mod foo;\npub fn foo() -> u8 { 7 }\n"),
            ("protected/foo.rs", "pub const INSIDE: u8 = 1;\n"),
            ("consumer.rs", "use crate::protected::foo::*;\n"),
        ],
    );
    assert!(
        findings(&probe.manifest()).is_empty(),
        "a glob reaches the descendant module's contents, never a value of the protected module"
    );
}

/// `use m::foo::{self};` binds the **module** `foo`, never the `fn foo` — so it must not react.
///
/// The same collapse the glob condition exists for, one import form over. `use_scan` records a
/// `{self}` leaf as its prefix module with `is_glob: false`, so it reaches the reaction byte-identical
/// to a bare `use m::foo;` — same path, same single-segment leaf, same declared `fn foo`.
///
/// Verified against rustc rather than reasoned. With both `mod foo` and `pub fn foo` declared,
/// `use m::foo::{self};` followed by `foo()` fails `error[E0423]: expected function, found module
/// 'foo'`, while `foo::INSIDE` compiles — so `{self}` reaches the module and no value of its parent.
///
/// All four spellings are asserted, because they take different paths through the use-tree expansion:
/// the bare `{self}`, the aliased `{self as f}`, one nested inside an outer brace group, and one
/// beside a sibling leaf.
#[test]
fn a_self_brace_import_binds_the_module_only_and_does_not_react() {
    for (label, consumer) in [
        ("bare", "use crate::protected::foo::{self};\n"),
        ("aliased", "use crate::protected::foo::{self as f};\n"),
        ("nested", "use crate::protected::{foo::{self}};\n"),
        (
            "with-sibling",
            "use crate::protected::foo::{self, INSIDE};\n",
        ),
    ] {
        let probe = Probe::new(
            label,
            &[
                ("lib.rs", "pub mod protected;\npub mod consumer;\n"),
                ("protected.rs", "pub mod foo;\npub fn foo() -> u8 { 7 }\n"),
                ("protected/foo.rs", "pub const INSIDE: u8 = 1;\n"),
                ("consumer.rs", consumer),
            ],
        );
        assert!(
            findings(&probe.manifest()).is_empty(),
            "`{label}`: a `{{self}}` leaf binds the module `foo`, not the `fn foo`, so it reaches only \
             the descendant"
        );
    }
}

/// A value name that is only *text* — a comment, a string literal, a macro body — declares nothing.
///
/// The collector's own precondition is declaration-cleaned source; it was handed the raw file, so any
/// of these three read as a declaration and made an ordinary `use protected::foo;` react even though
/// `protected` declares only the module. Each shape is asserted separately: one fixture covering all
/// three could pass while two of the strippings were missing.
#[test]
fn a_value_named_only_in_text_declares_nothing() {
    for (label, protected) in [
        ("comment", "pub mod foo;\n// pub fn foo() -> u8 { 7 }\n"),
        (
            "string",
            "pub mod foo;\npub const S: &str = \"pub fn foo() {}\";\n",
        ),
        (
            "macro-body",
            "pub mod foo;\nmacro_rules! never { () => { pub fn foo() {} }; }\n",
        ),
    ] {
        let probe = Probe::new(
            label,
            &[
                ("lib.rs", "pub mod protected;\npub mod consumer;\n"),
                ("protected.rs", protected),
                ("protected/foo.rs", "pub const INSIDE: u8 = 1;\n"),
                ("consumer.rs", "use crate::protected::foo;\n"),
            ],
        );
        assert!(
            findings(&probe.manifest()).is_empty(),
            "`{label}`: no value `foo` is declared, so the import reaches only the descendant module"
        );
    }
}
