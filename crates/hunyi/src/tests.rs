use super::*;

use std::path::{Path, PathBuf};

use crate::containment::leaf_of;
use crate::crate_scope::dependency_names;
use crate::errors::{missing_module_file_error, unknown_module_error, unknown_trait_error};
use crate::module_resolve::resolve_module_file;

/// A unique, self-cleaning temp `src/` tree: write source files (and, where needed, a symlink),
/// then hand its root/src paths to a pure entrypoint under test — replaces the hand-rolled
/// `temp_dir().join(format!(...))` + manual `remove_dir_all` at both ends that this file's many
/// fixture-building helpers otherwise each repeat.
struct TempSrcTree {
    dir: PathBuf,
    src: PathBuf,
}

impl TempSrcTree {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hunyi-{label}-{}", std::process::id()));
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        Self { dir, src }
    }

    /// Write a source file at `rel` (relative to `src/`), creating parent dirs as needed.
    /// Returns the file's absolute path.
    fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
        path
    }

    /// Write every `(relative path, contents)` pair under `src/`.
    fn write_all(&self, files: &[(&str, &str)]) {
        for (rel, contents) in files {
            self.write(rel, contents);
        }
    }

    fn src(&self) -> &Path {
        &self.src
    }

    fn root(&self) -> PathBuf {
        self.src.join("lib.rs")
    }

    #[cfg(unix)]
    fn symlink(&self, target: impl AsRef<Path>, link_rel_to_src: &str) -> &Self {
        std::os::unix::fs::symlink(target, self.src.join(link_rel_to_src)).expect("create symlink");
        self
    }

    /// The `cargo metadata`-shaped JSON some private `check_*_boundary` reads (needed only by the
    /// `fixture_metadata` call sites), for a single package "x" whose lib root is this tree's
    /// `src/lib.rs`.
    fn metadata(&self) -> Value {
        serde_json::json!({
            "packages": [{
                "name": "x",
                "dependencies": [],
                "targets": [{ "kind": ["lib"], "src_path": self.root().to_string_lossy().into_owned() }],
            }],
        })
    }
}

impl Drop for TempSrcTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Write `files` (each `(relative path, contents)`) under a unique temp `src` dir, then
/// return the findings for `module` against `forbidden`. Exercises the whole evaluator
/// (module resolution → exposure → use-resolution → match) without spawning `cargo`.
fn findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(name);
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let result = module_findings(
        tree.src(),
        &tree.root(),
        module,
        &forbidden,
        "x",
        false,
        &[],
    );
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

/// Like [`findings`] but with a declared **dependency-name set** (already `-`→`_`
/// normalized, as `dependency_names` produces), so an external-crate exposure resolves.
fn findings_with_deps(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(name);
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|s| s.to_string()).collect();
    let result = module_findings(
        tree.src(),
        &tree.root(),
        module,
        &forbidden,
        "x",
        false,
        &deps,
    );
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

/// Like [`findings`] but with the `semantic-trait-impl-exposure` opt-in enabled, so a trait
/// `impl` block's impl-site-authored positions are also observed.
fn findings_including_trait_impls(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(name);
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let result = module_findings(tree.src(), &tree.root(), module, &forbidden, "x", true, &[]);
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

// --- extern-path exposure (the external-crate name set) -------------------

#[test]
fn hyphenated_dependency_name_is_normalized() {
    let package = serde_json::json!({
        "dependencies": [
            { "name": "async-trait", "rename": null },
            { "name": "serde_json", "rename": "pkg" },
        ]
    });
    let mut names = dependency_names(&package);
    names.sort();
    assert_eq!(names, vec!["async_trait".to_string(), "pkg".to_string()]);
}

#[test]
fn duplicate_semantic_violations_collapse_keeping_the_more_severe() {
    // Two boundaries of one capability on one module can emit the same ViolationId; the outcome
    // fold collapses them by id and keeps the more-severe reaction, so a warn duplicate never masks
    // an enforce one and the fact is reported once (parity with the 圭表 static dimension's dedup).
    let mk = |sev| {
        let finding = crate::finding::SemanticFact::Exposed {
            kind: crate::finding::ExposureKind::Signature,
            subject: "crate::infra::Db".to_string(),
            seam: crate::finding::PublicSeam::FreeFn {
                module: "crate::m".to_string(),
                name: "f".to_string(),
            },
        }
        .into_finding();
        Violation::new(
            BoundaryKind::Semantic,
            ViolationId::new(
                "crate::m",
                RuleKey::of(
                    "tianheng.rule/hunyi/signature-exposure",
                    [
                        ("forbidden", "[\"crate::infra::Db\"]"),
                        ("including_trait_impls", "false"),
                    ],
                ),
                finding.key().clone(),
            ),
            SIGNATURE_RULE,
            finding.text(),
            "reason".to_string(),
            sev,
        )
    };
    match outcome_from(vec![mk(Severity::Warn), mk(Severity::Enforce)]) {
        Outcome::Violations(report) => {
            assert_eq!(
                report.violations.len(),
                1,
                "the duplicate id collapses to one: {:?}",
                report.violations
            );
            assert_eq!(
                report.violations[0].severity,
                Severity::Enforce,
                "the more-severe reaction is kept"
            );
        }
        other => panic!("expected Violations, got {other:?}"),
    }
}

#[test]
fn leaf_of_strips_a_raw_identifier() {
    // Declared marker leaf compares raw-canonicalized, symmetric with the observed `path_leaf`.
    assert_eq!(leaf_of("crate::a::r#Trait"), "Trait");
    assert_eq!(leaf_of("Plain"), "Plain");
}

#[test]
fn bare_extern_reexport_reacts() {
    let out = findings_with_deps(
        "ext-bare",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::spi::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by pub use crate::domain::Foo"]
    );
}

#[test]
fn sysroot_reexport_reacts_without_a_declared_dependency() {
    // `std` is never in `dependencies`, yet is a valid extern head — the set adds it.
    let out = findings(
        "ext-sysroot",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use std::sync::Mutex;\n"),
        ],
        "crate::domain",
        &["std::sync"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["std::sync::Mutex exposed by pub use crate::domain::Mutex"]
    );
}

#[test]
fn hyphenated_dependency_reexport_reacts_under_the_underscore_spelling() {
    let out = findings_with_deps(
        "ext-hyphen",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use async_trait::Thing;\n"),
        ],
        "crate::domain",
        &["async_trait"],
        &["async_trait"], // as `dependency_names` normalizes `async-trait`
    )
    .unwrap();
    assert_eq!(
        out,
        ["async_trait::Thing exposed by pub use crate::domain::Thing"]
    );
}

#[test]
fn aliased_extern_reexport_is_keyed_by_its_alias() {
    let out = findings_with_deps(
        "ext-alias",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::spi::Foo as Bar;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by pub use crate::domain::Bar"]
    );
}

#[test]
fn grouped_extern_reexport_reacts_per_leaf() {
    let out = findings_with_deps(
        "ext-group",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::spi::{Foo, Bar};\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "worklane_core::spi::Bar exposed by pub use crate::domain::Bar",
            "worklane_core::spi::Foo exposed by pub use crate::domain::Foo",
        ]
    );
}

#[test]
fn single_segment_crate_root_reexport_reacts() {
    let out = findings_with_deps(
        "ext-single",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core exposed by pub use crate::domain::worklane_core"]
    );
}

#[test]
fn subtree_extern_reexport_reacts_despite_a_crate_root_module_of_the_same_name() {
    // A crate-root `mod worklane_core` shadows the extern prelude only in the ROOT module; in
    // the child `crate::domain`, a bare `pub use worklane_core::Foo;` is the external crate by
    // edition-2018+ grammar and MUST react. The shadow is per-module (domain has no such
    // child), and a re-export head uses the raw set — so this real extern leak is not dropped.
    let out = findings_with_deps(
        "ext-subtree-reexport",
        &[
            (
                "lib.rs",
                "pub mod worklane_core { pub struct Foo; }\npub mod domain;\n",
            ),
            ("domain.rs", "pub use worklane_core::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::Foo exposed by pub use crate::domain::Foo"]
    );
}

#[test]
fn signature_child_module_shadowing_a_dependency_is_no_false_positive() {
    // The governed module declares its OWN `mod worklane_core`, so a type-position
    // `-> worklane_core::Foo` denotes the local child module, not the dependency — the
    // per-module shadow excludes it from the type-position set, so no false positive.
    let out = findings_with_deps(
        "ext-sig-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod worklane_core { pub struct Foo; }\npub fn make() -> worklane_core::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn inline_extern_field_type_reacts() {
    let out = findings_with_deps(
        "ext-field",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Handle { pub inner: worklane_core::spi::Conn }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Conn exposed by field crate::domain::Handle::inner"]
    );
}

#[test]
fn inline_extern_signature_return_reacts() {
    let out = findings_with_deps(
        "ext-sig",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn make() -> worklane_core::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn signature_child_module_path_is_no_false_positive() {
    // A bare child-module path in a signature (`child` not a dependency) stays unresolved
    // under `Ignore` — folding in extern resolution introduces no child-module leak.
    let out = findings_with_deps(
        "ext-child",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod child { pub struct Local; }\npub fn make() -> child::Local { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["child"],
        &[],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn facade_chain_of_inline_reexports_to_an_extern_type_reacts() {
    let out = findings_with_deps(
        "ext-facade",
        &[
            ("lib.rs", "pub mod facade;\npub mod domain;\n"),
            ("facade.rs", "pub use worklane_core::spi::Foo;\n"),
            ("domain.rs", "pub use crate::facade::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by pub use crate::domain::Foo"]
    );
}

#[test]
fn facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound() {
    // `facade: use …::Foo; pub use Foo;` — the closure captures only inline `pub use`
    // paths, so this hop is not followed. An inherited v0.1.3 bound, asserted explicit.
    let out = findings_with_deps(
        "ext-facade-priv",
        &[
            ("lib.rs", "pub mod facade;\npub mod domain;\n"),
            ("facade.rs", "use worklane_core::spi::Foo;\npub use Foo;\n"),
            ("domain.rs", "pub use crate::facade::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

// --- facade-closure re-export head-shadow (the sibling of the direct-head FP) -

#[test]
fn facade_reaching_a_child_shadowed_extern_head_does_not_react() {
    // `crate::a` re-exports `dep::spi::Foo` but declares a child
    // `mod dep`, so rustc resolves the bare head to the local module — the target is local, not the
    // dependency. A facade `crate::b`'s `pub use crate::a::Foo;` must NOT react: the crate-wide
    // re-export closure now excludes `crate::a`'s own child modules when collecting its re-exports,
    // so it no longer records `crate::a::Foo → dep::spi::Foo`.
    let out = findings_with_deps(
        "facade-child-shadow-extern",
        &[
            ("lib.rs", "pub mod a;\npub mod b;\n"),
            (
                "a.rs",
                "pub mod dep { pub mod spi { pub struct Foo; } }\npub use dep::spi::Foo;\n",
            ),
            ("b.rs", "pub use crate::a::Foo;\n"),
        ],
        "crate::b",
        &["dep::spi"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn facade_reaching_a_child_shadowed_rename_alias_head_does_not_react() {
    // A crate-root `extern crate worklane_core as wc;`, but
    // `crate::a` declares a child `mod wc` that shadows the bare alias head within `crate::a` (a
    // submodule `mod wc` does not conflict with the crate-root rename), so `pub use wc::spi::Foo;`
    // is local. A facade `crate::b` must NOT react — the closure's rename map is child-excluded for
    // `crate::a`'s bare heads, so it no longer rewrites `wc` to `worklane_core`.
    let out = findings_with_deps(
        "facade-child-shadow-rename",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod a;\npub mod b;\n",
            ),
            (
                "a.rs",
                "pub mod wc { pub mod spi { pub struct Foo; } }\npub use wc::spi::Foo;\n",
            ),
            ("b.rs", "pub use crate::a::Foo;\n"),
        ],
        "crate::b",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn leading_colon_facade_hop_reacts_through_the_closure_despite_a_child_module() {
    // No FN (the escape hatch through a facade): `crate::a`'s `pub use ::dep::spi::Foo;` is an
    // unambiguous extern (leading `::`), unshadowed by the child `mod dep`. A facade `crate::b`
    // must STILL react — the closure honors the `use` item's leading colon and keeps the raw extern
    // set for that head, so it records `crate::a::Foo → dep::spi::Foo`.
    let out = findings_with_deps(
        "facade-leading-colon",
        &[
            ("lib.rs", "pub mod a;\npub mod b;\n"),
            (
                "a.rs",
                "pub mod dep { pub mod spi { pub struct Foo; } }\npub use ::dep::spi::Foo;\n",
            ),
            ("b.rs", "pub use crate::a::Foo;\n"),
        ],
        "crate::b",
        &["dep::spi"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dep::spi::Foo exposed by pub use crate::b::Foo"]);
}

#[test]
fn crate_root_mod_does_not_suppress_a_child_facade_reexport_through_the_closure() {
    // No FN (per-defining-module scope): a crate-root `mod dep` does not shadow a bare
    // `pub use dep::Foo;` in a *child* module `crate::a` (there bare `dep` reaches only the extern
    // prelude — the crate-root module is `crate::dep`), so the closure still records the extern hop
    // and a facade `crate::b` reacts. The subtraction is scoped to each defining module's own items.
    let out = findings_with_deps(
        "facade-crate-root-mod",
        &[
            (
                "lib.rs",
                "pub mod dep { pub struct Foo; }\npub mod a;\npub mod b;\n",
            ),
            ("a.rs", "pub use dep::Foo;\n"),
            ("b.rs", "pub use crate::a::Foo;\n"),
        ],
        "crate::b",
        &["dep"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dep::Foo exposed by pub use crate::b::Foo"]);
}

// --- type-alias exposure (P1.1: resolvable-nominal-path aliases) -------------

#[test]
fn private_alias_in_a_public_seam_reacts() {
    // `type H = crate::infra::Db;` (private) hidden behind `pub fn make() -> H` was a
    // silent pass; the alias is now followed to its target.
    let out = findings(
        "alias-private",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = crate::infra::Db;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn a_generic_param_shadowing_a_same_module_alias_is_not_a_finding() {
    // A generic type parameter named identically to a same-module
    // `type` alias is a parameter *use*, not the alias, so it must not resolve through the alias to
    // its forbidden target. (Rust lets the param shadow the alias inside the item.)
    let out = findings(
        "param-shadows-alias",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "type Secret = crate::infra::Real;\npub fn f<Secret>(x: Secret) {}\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a generic param shadowing a same-module alias must not react: {out:?}"
    );
    // Control: WITHOUT the shadowing param, the same bare `Secret` IS the alias — it resolves to the
    // forbidden target and reacts. (Proves the fix only suppresses the param use, not the alias.)
    let out = findings(
        "alias-used-as-type",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "type Secret = crate::infra::Real;\npub fn g(x: Secret) {}\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Real exposed by fn crate::api::g"],
        "the alias used as a real type still reacts: {out:?}"
    );
}

#[test]
fn a_def_site_generic_param_shadowing_a_use_alias_is_not_a_finding() {
    // A struct's own generic parameter used bare inside its own
    // where-clause (`struct S<T, U> where U: AsRef<T>`) is a parameter, not a nominal type, so it
    // must not resolve through a same-named `use … as T` alias to a forbidden type. The def-site
    // generics walk previously ran UNSHADOWED (unlike every sibling member walk); it now shadows the
    // item's own params.
    let out = findings(
        "def-generics-param-shadows-alias",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "use crate::infra::Secret as T;\npub struct S<T, U> where U: AsRef<T> { pub f: U }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a def-site generic param shadowing a use-alias must not react: {out:?}"
    );
    // Control: a genuine multi-segment forbidden path in the where-clause is never shadowed and
    // still reacts — proving the fix suppresses only the bare param use, not real leaks.
    let out = findings(
        "def-generics-real-leak",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "pub struct S<U> where U: AsRef<crate::infra::Secret> { pub f: U }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.iter().any(|f| f.contains("crate::infra::Secret")),
        "a real forbidden bound in the def-site where-clause still reacts: {out:?}"
    );
}

#[test]
fn an_assoc_type_projection_off_a_shadowing_param_is_not_a_finding() {
    // An associated-type projection off a generic parameter
    // (`T::Item`) is a *parameter* projection, not a nominal type. When the module also declares a
    // same-named import alias (`use crate::infra::Secret as T;` — legal, the fn's `<T>` only
    // lexically shadows it), the projection previously escaped the param shadow (two segments, while
    // the shadow only covered the bare single-segment form) and was misresolved through the alias to
    // `crate::infra::Secret::Item`, reacting on code exposing nothing.
    let out = findings(
        "assoc-projection-shadows-alias",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "use crate::infra::Secret as T;\npub fn f<T: Iterator>() -> T::Item { unimplemented!() }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "an assoc-type projection off a shadowing param must not react: {out:?}"
    );
    // Control: a genuine multi-segment forbidden path in the same return position (head is NOT a
    // param) still reacts — proving the fix suppresses only the param projection, not real leaks.
    let out = findings(
        "assoc-projection-real-leak",
        &[
            ("lib.rs", "pub mod api;\npub mod infra;\n"),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "pub fn g() -> crate::infra::Secret { unimplemented!() }\n",
            ),
        ],
        "crate::api",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.iter().any(|f| f.contains("crate::infra::Secret")),
        "a real forbidden return type still reacts: {out:?}"
    );
}

#[test]
fn cross_module_alias_reached_via_use_reacts() {
    // The alias lives in another module and is reached via `use`; crate-wide collection
    // keys it by `crate::other::H`, which the exposure's resolved path canonicalizes through.
    let out = findings(
        "alias-cross",
        &[
            ("lib.rs", "pub mod domain;\npub mod other;\n"),
            ("other.rs", "pub type H = crate::infra::Db;\n"),
            (
                "domain.rs",
                "use crate::other::H;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn alias_through_a_reexport_chain_reacts() {
    // `type H = crate::facade::Db;` where `crate::facade` re-exports `crate::infra::Db` —
    // the alias and re-export hops are followed together to a fixpoint.
    let out = findings(
        "alias-reexport-chain",
        &[
            ("lib.rs", "pub mod domain;\npub mod facade;\n"),
            ("facade.rs", "pub use crate::infra::Db;\n"),
            (
                "domain.rs",
                "type H = crate::facade::Db;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn a_type_reached_through_a_reexported_module_facade_reacts() {
    // A `pub use crate::real::sub;` re-exports a MODULE; a member
    // reached through it (`crate::facade::sub::Foo`) must canonicalize (longest-prefix) to its
    // defining path `crate::real::sub::Foo` and react. Whole-key-only canonicalization missed it.
    let out = findings(
        "module-facade",
        &[
            (
                "lib.rs",
                "pub mod real;\npub mod facade;\npub mod domain;\n",
            ),
            ("real.rs", "pub mod sub { pub struct Foo; }\n"),
            ("facade.rs", "pub use crate::real::sub;\n"),
            (
                "domain.rs",
                "use crate::facade::sub;\npub fn f() -> sub::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::real::sub"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::real::sub::Foo exposed by fn crate::domain::f"],
        "a type reached through a re-exported module facade must canonicalize and react: {out:?}"
    );
}

#[test]
fn a_reexport_whose_key_prefixes_its_value_does_not_diverge() {
    // Termination guaranteed: a reexport map entry whose alias key is a strict
    // `::`-prefix of its own value — the shape a same-name nested re-export (`pub use self::x::x;`)
    // yields — made `rewrite_longest_prefix` re-fire on its own monotonically-growing output; the
    // exact-repeat `seen` guard never fires on a never-repeating sequence, so the tool hung / OOMed.
    // The hop cap now bounds the fixpoint regardless of map contents (this exercises the cap
    // directly, bypassing the build-time guard).
    use crate::resolve::{ReexportMap, canonicalize_through_reexports};
    let mut map = ReexportMap::new();
    map.insert("crate::a".to_string(), "crate::a::b".to_string());
    // Before the fix this never returned; the assertion is simply that it TERMINATES.
    let out = canonicalize_through_reexports("crate::a::foo", &map);
    assert!(
        !out.is_empty(),
        "canonicalization must terminate on a key⊂value reexport entry: {out:?}"
    );
}

#[test]
fn resolve_self_type_does_not_diverge_on_a_reexport_whose_key_prefixes_its_value() {
    // The sibling of the reexports test above, at `resolve_self_type`'s own resolver: before it
    // was routed through the shared, hop-capped `canonicalize_through_aliases`, its hand-rolled
    // outer loop re-ran the (already-capped) inner `canonicalize_through_reexports` call every
    // iteration, so a key⊂value reexport entry made the outer `landing` grow by a bounded amount
    // each iteration, never exactly repeating — the outer exact-repeat `seen` guard alone could
    // not catch that. The assertion is simply that this terminates.
    use crate::containment::resolve_self_type;
    use crate::resolve::{AliasMap, ReexportMap, UseMap};
    use std::collections::HashSet;

    let self_ty: syn::Type = syn::parse_str("Foo").unwrap();
    let uses = UseMap::new();
    let aliases = AliasMap::new();
    let mut reexports = ReexportMap::new();
    reexports.insert("crate::a::Foo".to_string(), "crate::a::Foo::b".to_string());
    let landing = resolve_self_type(
        &self_ty,
        &uses,
        "crate::a",
        &aliases,
        &reexports,
        &HashSet::new(),
    );
    assert!(
        landing.is_some(),
        "canonicalization must terminate on a key⊂value reexport entry: {landing:?}"
    );
}

#[test]
fn a_self_similar_reexport_is_dropped_and_the_real_type_still_reacts() {
    // Build-time guard: `pub use self::sub::sub;` re-exports the value `sub` from
    // a same-named child module, yielding a `crate::sub -> crate::sub::sub` map entry (key ⊂ value).
    // `collect_reexports` now refuses it — it is meaningless for type-path canonicalization and would
    // hang the fixpoint. The real type under `crate::sub` must still canonicalize to its own path
    // (never a fabricated `crate::sub::sub::Thing`) and react.
    let out = findings(
        "self-similar-reexport",
        &[
            (
                "lib.rs",
                "pub mod sub;\npub mod domain;\npub use self::sub::sub;\n",
            ),
            ("sub.rs", "pub fn sub() {}\npub struct Thing;\n"),
            ("domain.rs", "pub fn f(_x: crate::sub::Thing) {}\n"),
        ],
        "crate::domain",
        &["crate::sub"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::sub::Thing exposed by fn crate::domain::f"],
        "the real type under crate::sub reacts at its own path, never a fabricated one: {out:?}"
    );
}

#[test]
fn alias_of_an_alias_reacts() {
    // `type A = crate::infra::Db; type H = crate::domain::A;` — an alias→alias hop.
    let out = findings(
        "alias-of-alias",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type A = crate::infra::Db;\ntype H = crate::domain::A;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn alias_to_an_extern_path_reacts() {
    // `type H = worklane_core::spi::Foo;` — the alias target resolves via the extern oracle.
    let out = findings_with_deps(
        "alias-extern",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = worklane_core::spi::Foo;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn public_same_module_alias_still_reacts() {
    // Regression: a `pub type` alias's target is a walked exposed position (pre-existing),
    // unaffected by alias-map resolution.
    let out = findings(
        "alias-public-target",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub type H = crate::infra::Db;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by type crate::domain::H"]);
}

#[test]
fn complex_target_alias_is_a_stated_bound() {
    // `type H = Vec<crate::infra::Db>;` — a non-nominal target is not collected, so the
    // alias-hidden form stays a bound; the SAME `Vec<…>` written directly still reacts.
    let out = findings(
        "alias-complex-target",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = Vec<crate::infra::Db>;\npub fn hidden() -> H { unimplemented!() }\npub fn direct() -> Vec<crate::infra::Db> { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    // Only the directly-written Vec reacts; the alias-hidden Vec is the stated bound.
    assert_eq!(
        out,
        ["crate::infra::Db exposed by fn crate::domain::direct"]
    );
}

#[test]
fn generic_alias_is_a_stated_bound() {
    // `type H<T> = crate::infra::Db;` — a generic alias is skipped even with a nominal
    // target, and its parameterized use `H<u8>` is not a bare-alias site.
    let out = findings(
        "alias-generic",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H<T> = crate::infra::Db;\npub fn make() -> H<u8> { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn a_local_module_shadows_a_dependency_in_an_alias_target() {
    // `mod serde { … }` + `type H = serde::Foo;` — the target is the local child module,
    // not the dependency, so the per-module shadow leaves the alias uncollected (no FP).
    let out = findings_with_deps(
        "alias-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod serde { pub struct Foo; }\ntype H = serde::Foo;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn alias_to_a_nonforbidden_path_is_clean() {
    let out = findings(
        "alias-clean",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = crate::safe::Thing;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn alias_hidden_and_direct_exposures_share_the_canonical_type() {
    // The alias resolves to the same canonical type the direct spelling names, so baseline
    // identity is spelling-independent (the finding names `crate::infra::Db`, never `H`);
    // the two distinct seams stay distinct findings.
    let out = findings(
        "alias-identity",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = crate::infra::Db;\npub fn viaalias() -> H { unimplemented!() }\npub fn direct() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Db exposed by fn crate::domain::direct",
            "crate::infra::Db exposed by fn crate::domain::viaalias",
        ]
    );
}

#[test]
fn a_single_segment_alias_named_like_a_dependency_resolves_to_the_local_alias() {
    // `type serde = crate::infra::Db;` collides with the `serde` dependency name. The
    // bare-local-alias fallback fires before the extern oracle, so `-> serde` resolves to
    // the local alias's target `crate::infra::Db`, not the extern crate (Rust's shadowing).
    let out = findings_with_deps(
        "alias-dep-collision",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type serde = crate::infra::Db;\npub fn make() -> serde { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn alias_target_reached_via_use_reacts() {
    // The alias target is a bare name resolved through the module's own `use`-map
    // (`use crate::infra::Db; type H = Db;`), the same resolution an exposure gets.
    let out = findings(
        "alias-use-target",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra::Db;\ntype H = Db;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn alias_in_a_trait_impl_position_reacts_under_the_opt_in() {
    // Parity: `semantic-trait-impl-exposure` reuses signature-coupling's resolver, so an
    // alias in an impl-site-authored position resolves the same way under the opt-in.
    let out = findings_including_trait_impls(
        "alias-trait-impl",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = crate::infra::DbPool;\npub struct Service;\nimpl From<H> for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by impl From<H> for crate::domain::Service (trait-arg)"]
    );
}

#[test]
fn extern_glob_forbidden_root_reacts() {
    let out = findings_with_deps(
        "ext-glob-hit",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::spi::*;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi exposed by pub use crate::domain::*"]
    );
}

#[test]
fn extern_glob_nonforbidden_root_is_a_stated_bound() {
    let out = findings_with_deps(
        "ext-glob-miss",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::spi::*;\n"),
        ],
        "crate::domain",
        &["worklane_core::other"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn foreign_prelude_rename_is_a_stated_bound() {
    // Following `worklane_core::prelude::Foo` into the foreign crate needs its AST; the
    // written path is matched as-is and does not prefix-match the forbidden module.
    let out = findings_with_deps(
        "ext-prelude",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use worklane_core::prelude::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

// --- extern-crate exposure (P1.3) -----------------------------------------

#[test]
fn source_level_crate_root_extern_crate_rename_reacts() {
    // `extern crate worklane_core as wc;` at the crate root binds `wc` crate-wide (the extern
    // prelude); read from the local AST, `wc::spi::Foo` resolves to the real crate.
    let out = findings_with_deps(
        "ext-externcrate-rename",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            ("domain.rs", "pub use wc::spi::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by pub use crate::domain::Foo"]
    );
}

#[test]
fn source_level_extern_crate_rename_in_a_type_position_reacts() {
    let out = findings_with_deps(
        "ext-externcrate-rename-type",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub fn make() -> wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn private_use_of_a_crate_root_extern_rename_reacts() {
    // A forbidden type imported by a PRIVATE `use wc::spi::Foo;` (wc = a crate-root
    // `extern crate worklane_core as wc;` rename) resolves through the use-map to `wc::spi::Foo`
    // verbatim — the use-map never consults the rename map. `apply_bare_alias_rename` rewrites the
    // bare alias head to the real crate, so it now matches the forbidden real name, exactly as the
    // direct `-> wc::spi::Foo` type-position spelling already did.
    let out = findings_with_deps(
        "ext-private-use-rename",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "use wc::spi::Foo;\npub fn make() -> Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn private_use_of_a_child_shadowed_rename_alias_does_not_react() {
    // FP guard on the #2 fix: a governed module with its own child `mod wc` shadows the crate-root
    // alias, so `renames_bare` excludes `wc` and the bare-head rewrite does not fire — the imported
    // `Foo` stays local (`crate::domain::wc::spi::Foo`) and is not mistaken for the forbidden dep.
    let out = findings_with_deps(
        "ext-private-use-shadowed",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub mod wc { pub mod spi { pub struct Foo; } }\nuse wc::spi::Foo;\npub fn make() -> Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn module_scoped_extern_crate_rename_is_a_stated_bound() {
    // A rename declared inside `mod domain` binds only locally, so it is NOT collected into the
    // crate-wide map (collecting it would over-apply). A documented bound, not a silent claim.
    let out = findings_with_deps(
        "ext-externcrate-rename-modscoped",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "extern crate worklane_core as wc;\npub fn make() -> wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn extern_crate_rename_to_a_nonforbidden_crate_is_clean() {
    let out = findings_with_deps(
        "ext-externcrate-rename-clean",
        &[
            ("lib.rs", "extern crate serde as s;\npub mod domain;\n"),
            ("domain.rs", "pub use s::Value;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["serde", "worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn pub_extern_crate_reacts_as_an_exposure() {
    let out = findings_with_deps(
        "ext-pub-externcrate",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub extern crate worklane_core;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core exposed by pub extern crate worklane_core"]
    );
}

#[test]
fn pub_extern_crate_rename_names_the_real_crate() {
    // The exposure names the real crate `worklane_core`, not the `as`-rename `wc`.
    let out = findings_with_deps(
        "ext-pub-externcrate-rename",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub extern crate worklane_core as wc;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core exposed by pub extern crate worklane_core"]
    );
}

#[test]
fn private_extern_crate_is_not_an_exposure() {
    let out = findings_with_deps(
        "ext-priv-externcrate",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "extern crate worklane_core;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn pub_extern_crate_outside_the_forbidden_set_is_clean() {
    let out = findings_with_deps(
        "ext-pub-externcrate-clean",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub extern crate serde;\n"),
        ],
        "crate::domain",
        &["worklane_core"],
        &["serde", "worklane_core"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn a_bare_std_prelude_alias_target_is_not_mis_recorded() {
    // Guard for the name-gated collection fallback: `type H = String` (bare std prelude, not a
    // local alias) must NOT be recorded as `crate::domain::String`. Probed under a degenerate
    // self-forbidding boundary (the only set a mis-record would match) — must stay clean.
    let out = findings(
        "parity-nofp-std",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type H = String;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::domain"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn a_bare_alias_to_a_complex_local_alias_stays_bounded() {
    // `type Inner = Vec<crate::infra::Db>` (complex, not collected) then `type Public = Inner`
    // (bare). Public records `crate::domain::Inner`; the fixpoint stops there (Inner not in the
    // alias map) — the complex alias stays a stated bound, no react, no infinite loop.
    let out = findings(
        "parity-complex-intermediate",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type Inner = Vec<crate::infra::Db>;\ntype Public = Inner;\npub fn make() -> Public { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

// --- resolver collection↔query parity (FN1–FN3 + facade rename) -----------

#[test]
fn bare_alias_of_an_alias_reacts() {
    // FN1: `type Public = Inner` (bare intermediate). Collection records
    // Public → crate::domain::Inner (CurrentModule); the query fixpoint chains to infra::Db.
    let out = findings(
        "parity-fn1",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type Inner = crate::infra::Db;\ntype Public = Inner;\npub fn make() -> Public { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn bare_alias_of_an_alias_reacts_in_reverse_source_order() {
    // Same as above but the intermediate is declared AFTER the public alias — the query-time
    // fixpoint is order-independent (both aliases recorded with canonical names).
    let out = findings(
        "parity-fn1-rev",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type Public = Inner;\ntype Inner = crate::infra::Db;\npub fn make() -> Public { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra::Db exposed by fn crate::domain::make"]);
}

#[test]
fn alias_target_through_a_crate_root_extern_rename_reacts() {
    // FN2: alias target uses a source `extern crate … as` rename; collection now applies
    // extern_verbatim_renamed with the pre-collected rename map.
    let out = findings_with_deps(
        "parity-fn2",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "type H = wc::spi::Foo;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn alias_target_through_extern_rename_reacts_when_alias_precedes_extern_crate() {
    // FN2 root-forward-ref: the `type H` at the crate root is declared BEFORE the
    // `extern crate … as wc` — the pre-collection of renames makes it order-independent.
    let out = findings_with_deps(
        "parity-fn2-fwd",
        &[
            (
                "lib.rs",
                "type H = wc::spi::Foo;\nextern crate worklane_core as wc;\npub fn make() -> H { unimplemented!() }\n",
            ),
        ],
        "crate",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(out, ["worklane_core::spi::Foo exposed by fn crate::make"]);
}

#[test]
fn renamed_head_is_not_suppressed_by_a_same_named_child_module_shadow() {
    // FN3: a child `mod worklane_core` shadows the extern name in type positions, but the
    // as-written head is `wc` (a rename), not the child — the renamed head resolves directly.
    let out = findings_with_deps(
        "parity-fn3",
        &[
            ("lib.rs", "extern crate worklane_core as wc;\npub mod domain;\n"),
            (
                "domain.rs",
                "pub mod worklane_core { pub struct Local; }\npub fn make() -> wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn facade_reexport_through_an_extern_rename_reacts() {
    // FN2 sibling: a facade `pub use wc::spi::Foo` (rename) re-exported onward — the rename is
    // now threaded into the re-export closure.
    let out = findings_with_deps(
        "parity-facade-rename",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod facade;\npub mod domain;\n",
            ),
            ("facade.rs", "pub use wc::spi::Foo;\n"),
            ("domain.rs", "pub use crate::facade::Foo;\n"),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by pub use crate::domain::Foo"]
    );
}

#[test]
fn a_bare_alias_to_a_nonforbidden_local_type_is_clean() {
    // No false positive from the CurrentModule fallback: an alias to a same-module local type
    // resolves to crate::domain::Local, which matches no (sane) forbidden set.
    let out = findings(
        "parity-nofp",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Local;\ntype Public = Local;\npub fn make() -> Public { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

// --- semantic-trait-impl-exposure (opt-in depth) --------------------------

#[test]
fn trait_impl_exposure_reacts_at_the_trait_arg_position() {
    let out = findings_including_trait_impls(
        "ti-trait-arg",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl From<crate::infra::DbPool> for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by impl From<crate::infra::DbPool> for crate::domain::Service (trait-arg)"
        ]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_the_self_position_bare() {
    // F3a: the Self type IS the forbidden type — exposure, like a `pub fn` parameter.
    let out = findings_including_trait_impls(
        "ti-self-bare",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub trait Loc {}\nimpl Loc for crate::infra::Forbidden {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Forbidden exposed by impl Loc for crate::infra::Forbidden (self)"]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_the_self_position_nested() {
    // A forbidden type nested inside the Self type (`impl T for Vec<Forbidden>`).
    let out = findings_including_trait_impls(
        "ti-self-nested",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub trait Loc {}\nimpl Loc for Vec<crate::infra::DbPool> {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out.len(), 1, "one self-position finding expected: {out:?}");
    assert!(
        out[0].starts_with("crate::infra::DbPool exposed by impl Loc for")
            && out[0].ends_with("(self)"),
        "nested Self finding shape: {out:?}"
    );
}

#[test]
fn trait_impl_exposure_reacts_at_the_assoc_position() {
    let out = findings_including_trait_impls(
        "ti-assoc",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl Iterator for Service { type Item = crate::infra::Secret; }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Secret exposed by impl Iterator for crate::domain::Service (assoc Item)"]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_the_where_position() {
    let out = findings_including_trait_impls(
        "ti-where",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl<T: crate::infra::Secret> Loc for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Secret exposed by impl Loc for crate::domain::Service (where T)"]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_an_associated_const_type() {
    // Parity with the v1 trait-def walk (which observes assoc-const types): an impl-authored
    // associated const's type is impl-site-authored and must react.
    let out = findings_including_trait_impls(
        "ti-assoc-const",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl Marker for Service { const MAX: crate::infra::Limit = 0; }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Limit exposed by impl Marker for crate::domain::Service (assoc MAX)"]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_a_where_clause_bounded_type() {
    // The forbidden type on the LHS of a where-predicate (`where crate::infra::X: Clone`) is
    // impl-site-authored — must react, not just the RHS bound.
    let out = findings_including_trait_impls(
        "ti-where-lhs",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl Loc for Service where crate::infra::Assoc: Clone {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Assoc exposed by impl Loc for crate::domain::Service (where crate::infra::Assoc)"
        ]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_a_const_generic_param_type() {
    // The const-param's *type* annotation is impl-site-authored (position 4). The struct's own
    // param uses a plain `usize`, so the forbidden path appears ONLY on the impl block — a
    // signature-coupling finding cannot mask the trait-impl one.
    let out = findings_including_trait_impls(
        "ti-const-param",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service<const N: usize>;\nimpl<const N: crate::infra::Forbidden> Loc for Service<N> {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Forbidden exposed by impl Loc for crate::domain::Service<N> (where N)"]
    );
}

#[test]
fn trait_impl_exposure_reacts_at_a_refined_rpitit_return() {
    // The blocking review finding: a trait declares an opaque return, the impl refines it to a
    // concrete forbidden type at the impl site — must react (else the one forbidden bug).
    let out = findings_including_trait_impls(
        "ti-rpitit",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl Port for Service { fn items(&self) -> crate::infra::Iter { todo!() } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Iter exposed by impl Port for crate::domain::Service (method items return)"
        ]
    );
}

#[test]
fn a_trait_impl_generic_param_shadowing_an_alias_is_not_exposed() {
    // Round-2 fix (parallel to fix #6): an impl generic parameter named identically to a same-module
    // `use … as <param>` alias is a parameter use, not the aliased type — the trait-impl-exposure
    // collector now shadows the impl's params, so it must NOT resolve `T` through `as T` to the
    // forbidden type (a false positive the inherent-impl collector already avoids).
    let out = findings_including_trait_impls(
        "ti-param-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra::Forbidden as T;\npub struct Local;\npub trait SomeTrait<X> {}\nimpl<T> SomeTrait<T> for Local {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "an impl generic param must not resolve through a same-named `use … as` alias: {out:?}"
    );
}

#[test]
fn trait_impl_method_parameter_is_not_observed_but_the_return_is() {
    // Params/receiver are trait-dictated (invariant), so the parameter `crate::infra::DbPool`
    // does NOT react; the impl-refined return `crate::infra::Iter` DOES.
    let out = findings_including_trait_impls(
        "ti-param-vs-return",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl Sink for Service { fn put(&self, x: crate::infra::DbPool) -> crate::infra::Iter { todo!() } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Iter exposed by impl Sink for crate::domain::Service (method put return)"]
    );
}

#[test]
fn implementing_a_forbidden_trait_is_a_non_goal() {
    // F3b: the forbidden path is the trait being IMPLEMENTED, not a type it exposes —
    // that is `must_not_acquire`/locality's concern, not exposure. No finding.
    let out = findings_including_trait_impls(
        "ti-forbidden-trait",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl crate::infra::Sealed for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "implementing a forbidden trait must not react: {out:?}"
    );
}

#[test]
fn a_bare_boundary_ignores_trait_impls() {
    // Without `.including_trait_impls()`, the v1 signature-coupling surface is preserved.
    let out = findings(
        "ti-bare-off",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl From<crate::infra::DbPool> for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a bare boundary must not observe trait impls: {out:?}"
    );
}

#[test]
fn two_where_bounds_exposing_the_same_type_stay_distinct() {
    // F2 false-negative guard: distinct bounds keyed by their bounded type never collapse.
    let out = findings_including_trait_impls(
        "ti-where-distinct",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl<T, U> Loc for Service where T: crate::infra::Secret, U: crate::infra::Secret {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Secret exposed by impl Loc for crate::domain::Service (where T)",
            "crate::infra::Secret exposed by impl Loc for crate::domain::Service (where U)",
        ]
    );
}

#[test]
fn two_positions_exposing_the_same_type_stay_distinct() {
    // The one forbidden bug: same type at trait-arg and self must be two findings.
    let out = findings_including_trait_impls(
        "ti-two-positions",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "impl From<crate::infra::DbPool> for crate::infra::DbPool {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by impl From<crate::infra::DbPool> for crate::infra::DbPool (self)",
            "crate::infra::DbPool exposed by impl From<crate::infra::DbPool> for crate::infra::DbPool (trait-arg)",
        ]
    );
}

#[test]
fn a_reexported_type_in_a_trait_impl_position_resolves_and_reacts() {
    // Resolver reuse: a `pub use` facade path canonicalizes to its defining path before matching.
    let out = findings_including_trait_impls(
        "ti-reexport",
        &[
            ("lib.rs", "pub mod domain;\npub mod facade;\n"),
            ("facade.rs", "pub use crate::infra::DbPool;\n"),
            (
                "domain.rs",
                "use crate::facade::DbPool;\npub struct Service;\nimpl From<DbPool> for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by impl From<DbPool> for crate::domain::Service (trait-arg)"
        ]
    );
}

#[test]
fn a_bare_name_in_a_trait_impl_position_is_not_a_false_positive() {
    // F6: BareFallback::Ignore parity — a bare local name is not resolved against the current
    // module, so a boundary forbidding the module's own path does not fire on it.
    let out = findings_including_trait_impls(
        "ti-bare-name",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service;\nimpl From<DbPool> for Service {}\n",
            ),
        ],
        "crate::domain",
        &["crate::domain"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a bare name must not resolve against the current module: {out:?}"
    );
}

// --- semantic-reexport-exposure (default-on) ------------------------------

#[test]
fn reexport_of_a_forbidden_type_reacts_by_default() {
    let out = findings(
        "rx-named",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::DbPool;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by pub use crate::domain::DbPool"]
    );
}

#[test]
fn aliased_reexport_is_keyed_by_the_alias() {
    let out = findings(
        "rx-alias",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::DbPool as Pool;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by pub use crate::domain::Pool"]
    );
}

#[test]
fn two_aliases_of_the_same_type_stay_distinct_findings() {
    let out = findings(
        "rx-two-alias",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub use crate::infra::DbPool;\npub use crate::infra::DbPool as Pool;\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by pub use crate::domain::DbPool",
            "crate::infra::DbPool exposed by pub use crate::domain::Pool",
        ]
    );
}

#[test]
fn grouped_reexport_reacts_per_leaf() {
    let out = findings(
        "rx-group",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::{DbPool, Config};\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Config exposed by pub use crate::domain::Config",
            "crate::infra::DbPool exposed by pub use crate::domain::DbPool",
        ]
    );
}

#[test]
fn reexport_through_a_facade_chain_reacts() {
    let out = findings(
        "rx-facade",
        &[
            ("lib.rs", "pub mod domain;\npub mod facade;\n"),
            ("facade.rs", "pub use crate::infra::DbPool;\n"),
            ("domain.rs", "pub use crate::facade::DbPool;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by pub use crate::domain::DbPool"]
    );
}

#[test]
fn reexport_through_a_self_group_facade_chain_reacts() {
    // The facade republishes the whole forbidden module via `{self}`; the governed module then
    // re-exports that republished module. The closure must collapse the facade's trailing
    // `self` (key it by the prefix's final segment, target the prefix module) or the chain does
    // not canonicalize back to `crate::infra` and the leak passes silently — a false negative.
    let out = findings(
        "rx-self-facade",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod facade;\npub mod domain;\n",
            ),
            ("infra.rs", "pub struct DbPool;\n"),
            ("facade.rs", "pub use crate::infra::{self};\n"),
            ("domain.rs", "pub use crate::facade::infra;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra exposed by pub use crate::domain::infra"]
    );
}

#[test]
fn reexport_through_a_renamed_self_facade_chain_reacts_cleanly() {
    // The MAJOR companion: `{self as fs}` in the facade. Before the closure collapse this
    // reacted only by accident, emitting a malformed `crate::infra::self` canonical. It must
    // now canonicalize to a clean `crate::infra`.
    let out = findings(
        "rx-renamed-self-facade",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod facade;\npub mod domain;\n",
            ),
            ("infra.rs", "pub struct DbPool;\n"),
            ("facade.rs", "pub use crate::infra::{self as fs};\n"),
            ("domain.rs", "pub use crate::facade::fs;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra exposed by pub use crate::domain::fs"]);
}

#[test]
fn named_whole_module_reexport_reacts() {
    let out = findings(
        "rx-module",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra as fs;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra exposed by pub use crate::domain::fs"]);
}

#[test]
fn self_group_module_reexport_reacts_keyed_by_module_name() {
    let out = findings(
        "rx-self-group",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::{self, DbPool};\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra exposed by pub use crate::domain::infra",
            "crate::infra::DbPool exposed by pub use crate::domain::DbPool",
        ]
    );
}

#[test]
fn reexport_with_raw_identifier_segment_reacts() {
    // A raw-identifier (keyword) segment must not be dropped — the syn::Path is built from the
    // idents, not re-parsed from a stripped string, so `r#type` matches forbidden `crate::type`.
    let out = findings(
        "rx-raw",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::r#type::DbPool;\n"),
        ],
        "crate::domain",
        &["crate::type"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::type::DbPool exposed by pub use crate::domain::DbPool"]
    );
}

#[test]
fn renamed_self_module_reexport_reacts_with_correct_type() {
    // `{self as fs}` is a Rename node, not a Name — it must still collapse to the prefix module.
    let out = findings(
        "rx-self-rename",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::{self as fs};\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra exposed by pub use crate::domain::fs"]);
}

#[test]
fn glob_reexport_with_forbidden_root_reacts() {
    let out = findings(
        "rx-glob-root",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::*;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(out, ["crate::infra exposed by pub use crate::domain::*"]);
}

#[test]
fn glob_reexport_with_root_deeper_than_forbidden_prefix_reacts() {
    let out = findings(
        "rx-glob-deep",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::db::*;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::db exposed by pub use crate::domain::*"]
    );
}

#[test]
fn sibling_root_glob_does_not_react() {
    let out = findings(
        "rx-glob-sibling",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::elsewhere::*;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "sibling-root glob is a stated bound: {out:?}"
    );
}

#[test]
fn ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react() {
    // `pub use crate::infra::*` under a DEEPER forbidden prefix — a stated bound (can't
    // enumerate whether infra publicly re-exports the forbidden db subtree).
    let out = findings(
        "rx-glob-ancestor",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub use crate::infra::*;\n"),
        ],
        "crate::domain",
        &["crate::infra::db"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "ancestor-root glob is a stated bound: {out:?}"
    );
}

#[test]
fn restricted_and_private_and_underscore_reexports_do_not_react() {
    let out = findings(
        "rx-nonpublic",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub(crate) use crate::infra::DbPool;\nuse crate::infra::Config;\npub use crate::infra::Trait as _;\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "pub(crate)/private/`as _` re-exports are not public exposure: {out:?}"
    );
}

#[test]
fn forbidden_type_in_a_public_return_is_a_finding() {
    let out = findings(
        "return",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn pool() -> crate::infra::DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

#[test]
fn a_type_used_only_internally_is_not_a_finding() {
    // Imported and used in a private fn body / private item — never in a public
    // signature. This is the exposure-vs-import distinction: a static import boundary
    // would flag the import; semantic correctly says clean.
    let out = findings(
        "internal-only",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra::DbPool;\nfn helper() -> DbPool { todo!() }\nstruct Private { p: DbPool }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(out.is_empty(), "internal use is not exposure: {out:?}");
}

#[test]
fn forbidden_type_in_a_public_field_is_a_finding() {
    let out = findings(
        "field",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service { pub pool: crate::infra::DbPool, secret: u8 }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by field crate::domain::Service::pool"]
    );
}

#[test]
fn a_private_field_does_not_expose() {
    let out = findings(
        "private-field",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Service { pool: crate::infra::DbPool }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(out.is_empty(), "a private field is not public API: {out:?}");
}

#[test]
fn inherent_impl_public_method_exposes() {
    let out = findings(
        "inherent-impl",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct S;\nimpl S { pub fn pool(&self) -> crate::infra::DbPool { todo!() } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn <crate::domain::S>::pool"]
    );
}

#[test]
fn trait_impl_is_out_of_scope() {
    let out = findings(
        "trait-impl",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct S;\nimpl From<crate::infra::DbPool> for S { fn from(_: crate::infra::DbPool) -> S { S } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "trait impls are a documented bound: {out:?}"
    );
}

#[test]
fn a_renamed_import_resolves_and_reacts() {
    let out = findings(
        "renamed",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra::DbPool as Pool;\npub fn pool() -> Pool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

#[test]
fn a_use_imported_type_resolves_via_its_head() {
    let out = findings(
        "use-head",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra;\npub fn pool() -> infra::DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

#[test]
fn a_glob_import_is_a_documented_bound() {
    let out = findings(
        "glob",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::infra::*;\npub fn pool() -> DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "glob is out of scope, not silently matched: {out:?}"
    );
}

#[test]
fn a_forbidden_trait_in_a_generic_bound_is_a_finding() {
    let out = findings(
        "bound",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn run<T: crate::infra::Pooled>(_: T) {}\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Pooled exposed by fn crate::domain::run"]
    );
}

#[test]
fn a_module_prefix_matches_beneath_but_not_a_sibling() {
    let out = findings(
        "prefix",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn a() -> crate::infra::db::Pool { todo!() }\npub fn b() -> crate::infrastructure::Helper { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::db::Pool exposed by fn crate::domain::a"],
        "sibling must not match: {out:?}"
    );
}

#[test]
fn a_nested_generic_argument_is_observed() {
    let out = findings(
        "nested",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn pools() -> Vec<crate::infra::DbPool> { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pools"]
    );
}

#[test]
fn an_unknown_module_is_a_constitution_error() {
    let err = findings(
        "unknown",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "// nothing\n"),
        ],
        "crate::ghost",
        &["crate::infra"],
    )
    .unwrap_err();
    assert_eq!(err, unknown_module_error("crate::ghost", "x"));
}

#[test]
fn a_mod_rs_backed_module_resolves() {
    let out = findings(
        "modrs",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain/mod.rs",
                "pub fn pool() -> crate::infra::DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

#[test]
fn an_inline_module_resolves() {
    let out = findings(
        "inline",
        &[(
            "lib.rs",
            "pub mod domain { pub fn pool() -> crate::infra::DbPool { todo!() } }\n",
        )],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

// --- signature-coupling re-export back-fill (S1) -------------------------

#[test]
fn a_forbidden_type_via_a_pub_use_facade_resolves_and_reacts() {
    // The closed false negative: domain imports the type via a facade that re-exports
    // it; resolution must follow the `pub use` chain to the forbidden defining path.
    let out = findings(
        "reexport-exposure",
        &[
            ("lib.rs", "pub mod domain;\npub mod facade;\n"),
            ("facade.rs", "pub use crate::infra::DbPool;\n"),
            (
                "domain.rs",
                "use crate::facade::DbPool;\npub fn pool() -> DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"],
        "a forbidden type reached through a pub use facade must react"
    );
}

#[test]
fn a_forbidden_type_via_a_super_relative_use_resolves_and_reacts() {
    // The same relative-use canonicalization fix applies to exposure-governance: a
    // forbidden type imported via `use super::infra::DbPool` must resolve to its
    // canonical path, not be silently passed.
    let out = findings(
        "super-exposure",
        &[
            ("lib.rs", "pub mod domain;\npub mod infra;\n"),
            ("infra.rs", "pub struct DbPool;\n"),
            (
                "domain.rs",
                "use super::infra::DbPool;\npub fn pool() -> DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::domain::pool"]
    );
}

// --- trait-impl-locality ------------------------------------------------

fn locality_findings(
    name: &str,
    files: &[(&str, &str)],
    trait_path: &str,
    allowed: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("loc-{name}"));
    tree.write_all(files);
    let allowed: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let result = trait_impl_findings(tree.src(), &tree.root(), trait_path, &allowed, "x");
    // The pure-heart tests assert on findings only; drop the per-finding module/file here.
    result.map(|v| {
        v.into_iter()
            .map(|(finding, _module, _file)| finding.to_string())
            .collect()
    })
}

#[test]
fn an_impl_outside_the_allowed_location_is_a_finding() {
    let out = locality_findings(
        "outside",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn two_misplaced_impls_do_not_dedup_collapse_when_a_blanket_impls_param_shadows_an_alias() {
    // Round-10 finding: `canonical_self_owner` never received round 9's impl_type_params shadow at
    // all -- unlike resolve_self_type (containment.rs), it unconditionally resolved any bare self
    // type via resolve_path. This is not merely a cosmetic label: the `owner` it renders is part of
    // `SemanticFact::MisplacedImpl`'s finding IDENTITY, deduplicated by exact equality. A module
    // declaring `use Foo as T;` alongside BOTH a blanket `impl<T> Command for T {}` (T is the
    // impl's own generic parameter) AND a genuine direct `impl Command for Foo {}` had the blanket
    // impl's bare `T` incorrectly resolve through the alias to the SAME canonical owner string as
    // the direct impl's own (correctly resolved) owner -- two textually and semantically distinct
    // misplaced-impl violations collapsed into one reported finding, a real false negative (one
    // genuine violation silently vanishing), not just a wrong display string. Fixed by giving
    // `canonical_self_owner` the same `impl_type_params` shadow `resolve_self_type` already has.
    let out = locality_findings(
        "owner-collapse-blanket-and-direct",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\nuse crate::domain::sub::Foo as T;\npub mod sub { pub struct Foo; }\nimpl<T> Command for T {}\nimpl Command for crate::domain::sub::Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        2,
        "both the blanket impl (its own param T) and the direct impl on Foo are genuinely distinct \
         misplaced-impl violations and must not dedup-collapse into one: {out:?}"
    );
}

#[test]
fn a_cfg_dual_declared_module_backed_by_one_file_does_not_duplicate_its_impl_finding() {
    // Round-6 finding: resolve_child_modules (scan.rs, backing the whole-crate scan) had no
    // canonical-file dedup for two mutually-exclusive #[cfg] arms plainly declaring the IDENTICAL
    // name resolving to the ONE real file -- unlike module_resolve.rs's descend(), which gained
    // exactly this dedup in 0.2.2. A renderable const-generic owner keeps this test focused on the
    // cfg/file de-duplication contract; unrenderable identity is covered separately by a fail-loud
    // reaction. Verified against real rustc: both `cargo check --features u`
    // and `--features w` compile cleanly with exactly one `impl Command for Arr<2>`.
    let out = locality_findings(
        "cfg-dual-same-file",
        &[
            (
                "lib.rs",
                "pub trait Command {}\npub struct Arr<const N: usize>;\n\
                 #[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\npub mod foo;\n",
            ),
            ("foo.rs", "impl crate::Command for crate::Arr<2> {}\n"),
        ],
        "crate::Command",
        &["crate::allowed_elsewhere"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "one real impl, backed by one real file under either #[cfg] arm, must be one finding: {out:?}"
    );
}

#[test]
fn an_impl_inside_the_allowed_location_is_clean() {
    let out = locality_findings(
        "inside",
        &[
            ("lib.rs", "pub mod command;\npub mod commands;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "commands.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "an impl in the allowed location is clean: {out:?}"
    );
}

#[test]
fn a_nested_module_beneath_the_allowed_prefix_is_clean() {
    let out = locality_findings(
        "nested-allowed",
        &[
            ("lib.rs", "pub mod command;\npub mod commands;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("commands.rs", "pub mod greet;\n"),
            (
                "commands/greet.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "beneath an allowed prefix is clean: {out:?}"
    );
}

#[test]
fn a_prefix_colliding_sibling_location_is_not_allowed() {
    let out = locality_findings(
        "sibling",
        &[
            ("lib.rs", "pub mod command;\npub mod commandeer;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "commandeer.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::commandeer (impl crate::command::Command for crate::commandeer::Foo)"],
        "a sibling of the allowed prefix is not allowed"
    );
}

#[test]
fn an_impl_in_any_of_several_allowed_locations_is_clean() {
    let out = locality_findings(
        "multi-allowed",
        &[
            ("lib.rs", "pub mod command;\npub mod builtins;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "builtins.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands", "crate::builtins"],
    )
    .unwrap();
    assert!(out.is_empty(), "any one allowed location suffices: {out:?}");
}

#[test]
fn a_bare_same_module_trait_name_reacts() {
    // B1: the impl is in the trait's own (disallowed) module, with a bare `Command`
    // and no `use`. Resolving the bare name against the current module is required —
    // leaving it unresolved would silently pass a real misplaced impl.
    let out = locality_findings(
        "bare-same-module",
        &[
            ("lib.rs", "pub mod command;\n"),
            (
                "command.rs",
                "pub trait Command {}\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::command (impl crate::command::Command for crate::command::Foo)"]
    );
}

#[test]
fn a_renamed_trait_import_reacts() {
    let out = locality_findings(
        "renamed-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command as Cmd;\npub struct Foo;\nimpl Cmd for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn a_super_relative_trait_import_reacts() {
    // The relative-use false negative: `use super::command::Command` populates the
    // use-map with the relative string; resolution must canonicalize it against the
    // module before matching the anchor, or a real misplaced impl silently passes.
    let out = locality_findings(
        "super-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use super::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn a_cfg_gated_module_with_no_file_is_skipped_not_errored() {
    // A `#[cfg(feature = "x")] mod optional;` with no `optional.rs` (the feature is off)
    // is legal Rust; the whole-crate walk must skip it, never fail the gate (exit 2).
    let out = locality_findings(
        "cfg-absent-mod",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg(feature = \"never\")]\npub mod optional;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a cfg-gated absent module is skipped: {out:?}"
    );
}

#[test]
fn a_reexported_trait_path_reacts() {
    // S1: the impl reaches the trait through a facade re-export; resolution must
    // follow the pub use chain to match the anchor.
    let out = locality_findings(
        "reexport-impl",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod facade;\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            ("facade.rs", "pub use crate::command::Command;\n"),
            (
                "domain.rs",
                "use crate::facade::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn an_anchor_named_at_a_reexport_path_resolves_not_a_constitution_error() {
    // B2: the boundary names the trait at its facade path; this must resolve to the
    // real local trait (not a false exit-2) and still react to misplaced impls.
    let out = locality_findings(
        "reexport-anchor",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod facade;\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            ("facade.rs", "pub use crate::command::Command;\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::facade::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn an_unresolvable_trait_anchor_is_a_constitution_error() {
    let err = locality_findings(
        "ghost-trait",
        &[
            ("lib.rs", "pub mod command;\n"),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Ghost",
        &["crate::commands"],
    )
    .unwrap_err();
    assert_eq!(err, unknown_trait_error("crate::command::Ghost", "x"));
}

#[test]
fn a_non_anchored_traits_impl_is_ignored() {
    let out = locality_findings(
        "other-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\npub trait Other {}\n"),
            (
                "domain.rs",
                "use crate::command::Other;\npub struct Foo;\nimpl Other for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(out.is_empty(), "only the anchored trait reacts: {out:?}");
}

#[test]
fn an_inline_module_impl_is_located() {
    let out = locality_findings(
        "inline-impl",
        &[
            (
                "lib.rs",
                "pub mod command;\npub mod domain { use crate::command::Command; pub struct Foo; impl Command for Foo {} }\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn a_glob_imported_trait_is_a_documented_bound() {
    let out = locality_findings(
        "glob-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::*;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a glob-imported trait is out of scope, not silently matched: {out:?}"
    );
}

#[test]
fn an_unconditional_path_remapped_module_is_followed_and_its_impl_reacts() {
    // An unconditional `#[path = "weird.rs"] mod domain;` is now *followed* to weird.rs: a
    // disallowed impl there reacts, attributed to the module `crate::domain` (its declared path,
    // regardless of the file it lives in). Previously the module was skipped — a false negative
    // (a disallowed impl in a relocated module passing unobserved).
    let out = locality_findings(
        "path-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[path = \"weird.rs\"]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"],
        "the impl in the #[path]-relocated module is followed and reacts: {out:?}"
    );
}

#[test]
fn a_cfg_attr_remapped_module_is_a_documented_bound() {
    // `#[cfg_attr(<pred>, path = "…")]` is recognized as a remap (== the separate
    // `#[cfg(<pred>)] #[path = "…"]`), so the module is out of scope — not scanned against a
    // wrong/absent conventional file, and NOT a spurious exit-2. Mirrors the direct-#[path] bound.
    let out = locality_findings(
        "cfg-attr-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(windows, path = \"weird.rs\")]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a cfg_attr-remapped module is out of scope, same as a direct #[path]: {out:?}"
    );

    // A NESTED cfg_attr remap is recognized too, so hunyi stays
    // consistent with guibiao (both treat it as the #[path] bound) rather than diverging.
    let nested = locality_findings(
        "cfg-attr-nested-remapped",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(a, cfg_attr(b, path = \"weird.rs\"))]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "weird.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        nested.is_empty(),
        "a nested cfg_attr remap is out of scope: {nested:?}"
    );
}

#[test]
fn a_cfg_attr_without_a_path_meta_is_scanned_normally() {
    // The inverse false negative: a cfg_attr carrying NO `path` meta is a normal file module and
    // must be scanned, or its violations would silently vanish.
    let out = locality_findings(
        "cfg-attr-no-path",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(test, allow(dead_code))]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        !out.is_empty(),
        "a cfg_attr without a path meta is a normal module and must be scanned: {out:?}"
    );

    // Twin alignment: only a `path = "…"` NAME-VALUE is a remap. A bare `path` meta (not a valid
    // `#[path]`) is NOT a remap — so the module is scanned, matching guibiao's byte scanner (which
    // requires `path =`). Previously hunyi over-matched any `path`-named meta.
    let bare = locality_findings(
        "cfg-attr-bare-path",
        &[
            (
                "lib.rs",
                "pub mod command;\n#[cfg_attr(test, path)]\npub mod domain;\n",
            ),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        !bare.is_empty(),
        "a bare `path` meta (not `path = \"…\"`) is not a remap; the module is scanned: {bare:?}"
    );
}

#[test]
fn two_impls_in_one_module_are_distinct_findings_by_self_type() {
    let out = locality_findings(
        "distinct-self",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct A;\npub struct B;\nimpl Command for A {}\nimpl Command for B {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Command for crate::domain::A)",
            "crate::domain (impl crate::command::Command for crate::domain::B)"
        ]
    );
}

#[test]
fn const_generic_expr_self_types_fail_loud_without_positional_identity() {
    // The ordinary owner renderer cannot distinguish these complex const expressions. Publishing
    // scan position would make identity drift under reorder/insertion, so observation must fail.
    let error = findings(
        "const-generic-expr",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Arr<const N: usize>(u8);\n\
                 impl Arr<{ 1 + 1 }> { pub fn a(&self) -> crate::infra::T { todo!() } }\n\
                 impl Arr<{ 2 + 2 }> { pub fn a(&self) -> crate::infra::T { todo!() } }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(error.contains("stable structural label"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
fn owner_is_canonical_across_written_forms() {
    // The same self type written two ways — a bare `impl Foo` and a fully-qualified
    // `impl crate::m::Foo` — must render to the IDENTICAL canonical owner
    // `crate::m::Foo`, so the token form never over-splits a single type into two owners.
    let out = findings(
        "canonical-forms",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub struct Foo;\n\
                 impl Foo { pub fn a(&self) -> crate::infra::T { todo!() } }\n\
                 impl crate::m::Foo { pub fn b(&self) -> crate::infra::T { todo!() } }\n",
            ),
        ],
        "crate::m",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::T exposed by fn <crate::m::Foo>::a",
            "crate::infra::T exposed by fn <crate::m::Foo>::b",
        ],
        "both written forms of the same self type render the identical canonical owner",
    );
}

#[test]
fn a_cfg_gated_impl_is_observed_as_written() {
    // `#[cfg]` is not evaluated: syn parses every branch, so a misplaced impl behind a
    // disabled feature is still observed (a deliberate, documented over-approximation).
    let out = locality_findings(
        "cfg-gated",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct Foo;\n#[cfg(feature = \"never\")]\nimpl Command for Foo {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::domain (impl crate::command::Command for crate::domain::Foo)"]
    );
}

#[test]
fn a_macro_generated_impl_is_a_documented_bound() {
    // A `make_impl!(…)` invocation is an `Item::Macro`, not an `Item::Impl` — syn does
    // not expand it, so the impl it would generate is out of scope, not silently matched.
    let out = locality_findings(
        "macro-impl",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("domain.rs", "make_impl!(Foo);\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a macro-generated impl is out of scope, not silently matched: {out:?}"
    );
}

#[test]
fn the_builder_carries_severity() {
    // Severity (and thus baseline/exit-code parity via the shared 璇璣 model) is plumbed
    // from the builder into each Violation by `check_trait_impl_boundary`.
    let warn = TraitImplBoundary::in_crate("app")
        .trait_("crate::command::Command")
        .only_implemented_in("crate::commands")
        .warn()
        .because("advisory first");
    assert_eq!(warn.severity(), Severity::Warn);

    let enforce = TraitImplBoundary::in_crate("app")
        .trait_("crate::command::Command")
        .only_implemented_in("crate::commands")
        .because("enforced");
    assert_eq!(enforce.severity(), Severity::Enforce);
}

#[test]
fn every_hunyi_rule_family_has_exact_semantic_identity() {
    fn assert_rule(rule: xuanji::RuleKey, expected_type: &str, expected_fields: &[(&str, &str)]) {
        assert_eq!(rule.rule_type(), expected_type);
        assert_eq!(rule.fields().collect::<Vec<_>>(), expected_fields);
    }

    let signature = SemanticBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("r#crate::infra")
        .and_not_expose("crate::storage")
        .including_trait_impls()
        .because("presentation only");
    assert_rule(
        signature.rule_key(),
        "tianheng.rule/hunyi/signature-exposure",
        &[
            ("forbidden", "[\"crate::infra\",\"crate::storage\"]"),
            ("including_trait_impls", "true"),
        ],
    );

    let dyn_trait = DynTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_dyn_of(["crate::Port", "crate::Other"])
        .because("r");
    assert_rule(
        dyn_trait.rule_key(),
        "tianheng.rule/hunyi/dyn-trait-exposure",
        &[("forbidden_operands", "[\"crate::Other\",\"crate::Port\"]")],
    );

    let impl_trait = ImplTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_impl_trait_of(["crate::Port"])
        .including_submodules()
        .because("r");
    assert_rule(
        impl_trait.rule_key(),
        "tianheng.rule/hunyi/impl-trait-exposure",
        &[
            ("forbidden_operands", "[\"crate::Port\"]"),
            ("including_submodules", "true"),
        ],
    );

    let locality = TraitImplBoundary::in_crate("x")
        .trait_("r#crate::Port")
        .only_implemented_in("crate::adapter")
        .and_in("crate::infra")
        .because("r");
    assert_rule(
        locality.rule_key(),
        "tianheng.rule/hunyi/trait-impl-locality",
        &[
            ("allowed_locations", "[\"crate::adapter\",\"crate::infra\"]"),
            ("trait", "crate::Port"),
        ],
    );

    let marker = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_acquire("serde::Serialize")
        .and_not_acquire("serde::Deserialize")
        .because("r");
    assert_rule(
        marker.rule_key(),
        "tianheng.rule/hunyi/forbidden-marker",
        &[("forbidden", "[\"serde::Deserialize\",\"serde::Serialize\"]")],
    );

    let visibility = VisibilityBoundary::in_crate("x")
        .module("crate::internal")
        .max_visibility(VisibilityCeiling::Super)
        .because("r");
    assert_rule(
        visibility.rule_key(),
        "tianheng.rule/hunyi/visibility-ceiling",
        &[("ceiling", "super")],
    );

    let async_exposure = AsyncExposureBoundary::in_crate("x")
        .module("crate::core")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("r");
    assert_rule(
        async_exposure.rule_key(),
        "tianheng.rule/hunyi/async-exposure",
        &[("including_submodules", "true")],
    );

    let unsafe_confinement = UnsafeBoundary::in_crate("x")
        .only_under(["crate::ffi", "crate::platform"])
        .because("r");
    assert_rule(
        unsafe_confinement.rule_key(),
        "tianheng.rule/hunyi/unsafe-confinement",
        &[("allowed", "[\"crate::ffi\",\"crate::platform\"]")],
    );
}

#[test]
fn hunyi_rule_identity_is_set_order_stable_and_parameter_sensitive() {
    let left = SemanticBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .because("first wording");
    let reordered = SemanticBoundary::in_crate("other")
        .module("crate::elsewhere")
        .must_not_expose("crate::storage")
        .and_not_expose("crate::infra")
        .and_not_expose("crate::infra")
        .warn()
        .because("different wording")
        .with_anchor("GOV-1");
    let expanded = SemanticBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .and_not_expose("crate::transport")
        .because("first wording");
    let deeper = SemanticBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose("crate::infra")
        .and_not_expose("crate::storage")
        .including_trait_impls()
        .because("first wording");

    assert_eq!(left.rule_key(), reordered.rule_key());
    assert_ne!(left.rule_key(), expanded.rule_key());
    assert_ne!(left.rule_key(), deeper.rule_key());
}

// --- unsafe confinement --------------------------------------------------

fn unsafe_labels(
    name: &str,
    files: &[(&str, &str)],
    allowed: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("unsafe-{name}"));
    tree.write_all(files);
    let allowed: Vec<String> = allowed.iter().map(|a| a.to_string()).collect();
    unsafe_findings(tree.src(), &tree.root(), &allowed, "x").map(|fs| {
        fs.into_iter()
            .map(|(finding, _, _)| finding.to_string())
            .collect()
    })
}

fn unsafe_keys(name: &str, source: &str) -> Result<Vec<StructuredFactIdentity>, String> {
    let tree = TempSrcTree::new(&format!("unsafe-keys-{name}"));
    tree.write_all(&[("lib.rs", "pub mod net;\n"), ("net.rs", source)]);
    unsafe_findings(tree.src(), &tree.root(), &["crate::ffi".to_string()], "x").map(|findings| {
        findings
            .into_iter()
            .map(|(fact, _, _)| fact.into_finding().key().clone())
            .collect()
    })
}

#[test]
fn unsafe_identity_survives_reorder_and_unrelated_insertion() {
    let before = unsafe_keys(
        "reorder-before",
        "pub struct Api;\nunsafe impl Send for Api {}\n",
    )
    .unwrap();
    let after = unsafe_keys(
        "reorder-after",
        "pub const UNRELATED: usize = 1;\npub struct Api;\nunsafe impl Send for Api {}\n",
    )
    .unwrap();
    assert_eq!(before, after);
}

#[test]
fn unrenderable_unsafe_owner_fails_loud_without_an_ordinal_identity() {
    let error = unsafe_keys(
        "unrenderable-owner",
        "pub struct Arr<const N: usize>;\npub const N: usize = 1;\nunsafe impl Send for Arr<{ N + 1 }> {}\n",
    )
    .unwrap_err();
    assert!(error.contains("without a positional fallback"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
fn unsafe_production_violation_separates_target_rule_and_fact_roles() {
    let (metadata, _fixture) = fixture_metadata(
        "unsafe-identity",
        &[
            ("lib.rs", "pub mod net;\npub mod ffi;\n"),
            ("net.rs", "pub unsafe fn decode() {}\n"),
            ("ffi.rs", ""),
        ],
    );
    let boundary = UnsafeBoundary::in_crate("x")
        .only_under(["crate::raw", "crate::ffi"])
        .because("unsafe stays behind the audited adapter");
    let mut violations = Vec::new();
    check_unsafe_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);

    let id = violations[0].id();
    assert_eq!(id.target(), "x");
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/unsafe-confinement");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("allowed", "[\"crate::ffi\",\"crate::raw\"]")]
    );
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/hunyi/unsafe-site");
    assert_eq!(fact.shape(), "unsafe-free-function");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![("module", "crate::net"), ("name", "decode")]
    );
}

#[test]
fn unsafe_block_outside_subtree_reacts() {
    let out = unsafe_labels(
        "block",
        &[
            ("lib.rs", "pub mod ffi;\npub mod net;\n"),
            (
                "ffi.rs",
                "pub fn ok() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
            (
                "net.rs",
                "pub fn f() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe block in crate::net"],
        "a block outside the subtree reacts; one under it is clean: {out:?}"
    );
}

#[test]
fn unsafe_fn_impl_trait_extern_outside_react() {
    let out = unsafe_labels(
        "kinds",
        &[
            ("lib.rs", "pub mod ffi;\npub mod net;\n"),
            ("ffi.rs", "\n"),
            (
                "net.rs",
                "pub unsafe trait Zeroable {}\npub unsafe fn decode() {}\nunsafe impl Zeroable for u8 {}\nunsafe extern \"C\" { fn c(); }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe extern block in crate::net",
            "unsafe fn decode in crate::net",
            "unsafe impl Zeroable for u8 in crate::net",
            "unsafe trait Zeroable in crate::net",
        ],
        "every unsafe-keyword site outside the subtree reacts: {out:?}"
    );
}

#[test]
fn unsafe_under_the_subtree_is_clean() {
    let out = unsafe_labels(
        "clean",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "pub mod raw;\npub unsafe fn a() {}\n"),
            (
                "ffi/raw.rs",
                "pub fn b() { unsafe { core::ptr::null::<u8>(); } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "unsafe at the subtree and beneath it is clean: {out:?}"
    );
}

#[test]
fn empty_allowed_set_is_a_constitution_error() {
    let err = unsafe_labels("empty", &[("lib.rs", "pub fn f() { unsafe {} }\n")], &[]).unwrap_err();
    assert!(
        err.contains("forbid(unsafe_code)"),
        "empty only_under points at #![forbid(unsafe_code)]: {err}"
    );
}

#[test]
fn crate_root_allowed_set_is_a_constitution_error() {
    let err = unsafe_labels("root", &[("lib.rs", "pub fn f() {}\n")], &["crate"]).unwrap_err();
    assert!(err.contains("crate root"), "{err}");
}

#[test]
fn unsafe_blocks_dedup_per_module() {
    let out = unsafe_labels(
        "dedup",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub fn f() { unsafe {} unsafe {} }\npub fn g() { unsafe {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe block in crate::net"],
        "N blocks in one module dedup to one stable finding: {out:?}"
    );
}

#[test]
fn two_unsafe_impls_of_different_traits_stay_distinct() {
    let out = unsafe_labels(
        "impls",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\nunsafe impl Send for Foo {}\nunsafe impl Sync for Foo {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe impl Send for Foo in crate::net",
            "unsafe impl Sync for Foo in crate::net",
        ],
        "the trait is in the finding, so two unsafe impls do not collapse: {out:?}"
    );
}

#[test]
fn two_unsafe_impls_of_one_trait_for_different_types_stay_distinct() {
    // Same trait, different self type: the finding is owner-qualified, so neither masks the other.
    // Were the self type omitted, a baseline of the first would silently accept the second — a
    // false negative (a new out-of-subtree `unsafe` site passing unobserved).
    let out = unsafe_labels(
        "impls-same-trait",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub struct Bar;\nunsafe impl Send for Foo {}\nunsafe impl Send for Bar {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe impl Send for Bar in crate::net",
            "unsafe impl Send for Foo in crate::net",
        ],
        "the self type is in the finding, so same-trait impls for different types do not collapse: {out:?}"
    );
}

#[test]
fn two_same_named_unsafe_fns_on_different_owners_stay_distinct() {
    // Same method name, different inherent-impl self type: the finding must be owner-qualified,
    // else a baseline of the first silently accepts the second — a false negative (a new
    // out-of-subtree `unsafe` site passing unobserved). The unsafe-fn analogue of
    // `two_unsafe_impls_of_one_trait_for_different_types_stay_distinct`.
    let out = unsafe_labels(
        "unsafe-fns-same-name",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub struct Bar;\nimpl Foo { unsafe fn m(&self) {} }\nimpl Bar { unsafe fn m(&self) {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn Bar::m in crate::net",
            "unsafe fn Foo::m in crate::net",
        ],
        "same-named unsafe fns on different owners must not collapse: {out:?}"
    );
}

#[test]
fn two_same_named_unsafe_trait_fns_stay_distinct() {
    // Two traits in one module each declaring `unsafe fn m` must stay distinct findings, qualified
    // by the declaring trait — else a baseline of the first masks the second (a false negative).
    let out = unsafe_labels(
        "unsafe-trait-fns",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub trait A { unsafe fn m(&self); }\npub trait B { unsafe fn m(&self); }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn A::m in crate::net",
            "unsafe fn B::m in crate::net"
        ],
        "trait-declared unsafe fns must be qualified by their trait: {out:?}"
    );
}

#[test]
fn trait_impl_unsafe_fn_stays_distinct_from_inherent_and_other_traits() {
    // A trait-impl `unsafe fn` is qualified by `<trait for self>`, not the self type alone: on ONE
    // self type, an inherent `unsafe fn m`, `impl A for Foo { unsafe fn m }`, and
    // `impl B for Foo { unsafe fn m }` are three distinct `unsafe` sites and MUST stay three
    // findings — else a baseline of the inherent (or one trait-impl) silently accepts a later-added
    // trait-impl `unsafe fn` on a *safe* trait (no independent `unsafe impl` finding): a new
    // out-of-subtree `unsafe` site passing unobserved, the forbidden false negative. Self-type-only
    // qualification (`unsafe fn Foo::m` for all three) collapsed them; this pins the fix.
    let out = unsafe_labels(
        "unsafe-fns-trait-impl",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub struct Foo;\npub trait A { fn m(&self); }\npub trait B { fn m(&self); }\n\
                 impl Foo { unsafe fn m(&self) {} }\n\
                 impl A for Foo { unsafe fn m(&self) {} }\n\
                 impl B for Foo { unsafe fn m(&self) {} }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "unsafe fn <A for Foo>::m in crate::net",
            "unsafe fn <B for Foo>::m in crate::net",
            "unsafe fn Foo::m in crate::net",
        ],
        "a trait-impl unsafe fn must be qualified by <trait for self>, distinct from the inherent \
         method and other trait impls on the same type: {out:?}"
    );
}

#[test]
fn unsafe_in_an_unconditional_path_remapped_module_reacts() {
    // An unconditional `#[path = "relocated.rs"] mod net;` is followed to relocated.rs (there is no
    // conventional net.rs, so this only resolves by following the remap); its `unsafe fn`, outside
    // the allowed subtree, reacts attributed to the declared module path `crate::net`. Previously
    // the relocated module was skipped — a false negative (relocated `unsafe` passing unobserved).
    let out = unsafe_labels(
        "path-remap-unsafe",
        &[
            ("lib.rs", "#[path = \"relocated.rs\"]\npub mod net;\n"),
            ("relocated.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::net"],
        "unsafe in an unconditional #[path] module is followed and reacts: {out:?}"
    );
}

#[test]
fn path_in_a_non_mod_rs_file_resolves_from_the_containing_files_own_dir() {
    // rustc 1.x ground truth: a non-inline `#[path="bar.rs"]` written INSIDE src/foo.rs (reached via
    // `mod foo;`) resolves to src/bar.rs — the CONTAINING file's own directory — NOT src/foo/bar.rs.
    // The real unsafe fn lives at the rustc-correct src/bar.rs; a decoy sits at the wrong src/foo/bar.rs
    // the earlier (buggy) child_dir base would have read. Resolving from child_dir reads the decoy and
    // drops the real unsafe (Ok([]) — the forbidden false negative); this pins the corrected base.
    let out = unsafe_labels(
        "path-nonmodrs",
        &[
            ("lib.rs", "pub mod foo;\n"),
            ("foo.rs", "#[path = \"bar.rs\"]\npub mod bar;\n"),
            ("bar.rs", "pub unsafe fn poke() {}\n"),
            ("foo/bar.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::foo::bar"],
        "a #[path] inside a non-mod.rs file resolves from that file's own dir (src/bar.rs), not \
         src/foo/bar.rs: {out:?}"
    );
}

#[test]
fn path_nested_in_an_inline_block_resolves_from_the_accumulated_dir() {
    // rustc ground truth (verified against rustc 1.96.0): a `#[path="other.rs"]` written INSIDE an
    // inline `mod inline { … }` at the crate root resolves to src/inline/other.rs — rustc accumulates
    // the inline-module name as a directory component onto the file's own dir. The real unsafe lives
    // at the rustc-correct src/inline/other.rs; a decoy sits at src/other.rs, which threading the
    // enclosing file_dir UNCHANGED through inline descent would have read (dropping the real unsafe,
    // Ok([]) — the forbidden false negative). Pins the accumulated inline base.
    let out = unsafe_labels(
        "path-inline-modrs",
        &[
            (
                "lib.rs",
                "pub mod inline { #[path = \"other.rs\"] pub mod inner; }\n",
            ),
            ("inline/other.rs", "pub unsafe fn poke() {}\n"),
            ("other.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::inline::inner"],
        "a #[path] nested in an inline block resolves from <file_dir>/inline (src/inline/other.rs), \
         not the src/other.rs orphan: {out:?}"
    );
}

#[test]
fn path_nested_in_an_inline_block_in_a_non_mod_rs_file_accumulates_both_components() {
    // rustc ground truth (rustc 1.96.0): src/bar.rs (reached via `mod bar;`, a non-mod-rs file) with
    // `pub mod inline { #[path="p.rs"] pub mod inner; }` resolves inner to src/bar/inline/p.rs — the
    // base accumulates BOTH the non-mod-rs conventional-child dir (bar/) AND the inline name (inline/).
    // Real unsafe at the rustc-correct src/bar/inline/p.rs; a decoy at src/p.rs (the enclosing
    // file_dir base). Confirms the two components compose.
    let out = unsafe_labels(
        "path-inline-nonmodrs",
        &[
            ("lib.rs", "pub mod bar;\n"),
            (
                "bar.rs",
                "pub mod inline { #[path = \"p.rs\"] pub mod inner; }\n",
            ),
            ("bar/inline/p.rs", "pub unsafe fn poke() {}\n"),
            ("p.rs", "pub fn decoy() {}\n"),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::bar::inline::inner"],
        "the #[path] base accumulates bar/ and inline/ (src/bar/inline/p.rs), not src/p.rs: {out:?}"
    );
}

#[test]
fn two_modules_sharing_one_path_target_are_not_a_false_cycle() {
    // rustc ground truth (rustc 1.96.0): `#[path="shared.rs"] pub mod a; #[path="shared.rs"] pub mod
    // b;` compiles cleanly — two sibling declarations legitimately resolving to one file is NOT a
    // cycle. A monotonic whole-tree visited set misreported the second reach as a "symlink loop"
    // (exit 2) on rustc-compilable input — a false positive and a 三儀 ⊥ 三儀 divergence (漏刻 accepts
    // it). The ancestor-path guard must accept it: the unsafe in shared.rs reacts under BOTH paths.
    let out = unsafe_labels(
        "path-shared-target",
        &[
            (
                "lib.rs",
                "#[path = \"shared.rs\"]\npub mod a;\n#[path = \"shared.rs\"]\npub mod b;\n",
            ),
            ("shared.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .expect("two modules sharing one #[path] target is not a cycle (rustc compiles it)");
    assert_eq!(
        out,
        ["unsafe fn poke in crate::a", "unsafe fn poke in crate::b",],
        "a file shared by two #[path] declarations reacts under both module paths, no false cycle: \
         {out:?}"
    );
}

#[test]
fn a_conventional_module_and_a_path_alias_to_it_are_not_a_false_cycle() {
    // rustc ground truth (rustc 1.96.0): `pub mod foo; #[path="foo.rs"] pub mod bar;` compiles — one
    // file (src/foo.rs) reached by a conventional decl and a #[path] alias is not a cycle. Pins the
    // second, conventional-branch face of the ancestor-guard fix.
    let out = unsafe_labels(
        "path-alias-conventional",
        &[
            (
                "lib.rs",
                "pub mod foo;\n#[path = \"foo.rs\"]\npub mod bar;\n",
            ),
            ("foo.rs", "pub unsafe fn poke() {}\n"),
        ],
        &["crate::ffi"],
    )
    .expect("a conventional module and a #[path] alias to the same file is not a cycle");
    assert_eq!(
        out,
        [
            "unsafe fn poke in crate::bar",
            "unsafe fn poke in crate::foo",
        ],
        "one file reached conventionally and via a #[path] alias reacts under both paths: {out:?}"
    );
}

#[test]
fn unsafe_in_a_body_nested_mod_reacts() {
    // The propose-review false-negative guard: a `mod` inside a fn body is not descended by the
    // top-level walk; the collector's default recursion must still catch its unsafe.
    let out = unsafe_labels(
        "body-nested",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "pub fn f() { mod raw { pub unsafe fn poke() {} } }\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["unsafe fn poke in crate::net"],
        "unsafe in a body-nested mod is attributed to the enclosing module, never dropped: {out:?}"
    );
}

#[test]
fn unsafe_in_a_macro_body_is_a_stated_bound() {
    // Macro bodies are unexpanded (the dimension's inherited macro bound): the unsafe inside a
    // never-invoked macro definition is not observed — stated, not a silent claim.
    let out = unsafe_labels(
        "macro",
        &[
            ("lib.rs", "pub mod net;\n"),
            (
                "net.rs",
                "macro_rules! m { () => { unsafe {} }; }\npub fn f() {}\n",
            ),
        ],
        &["crate::ffi"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "unsafe in a macro body is not observed: {out:?}"
    );
}

// --- visibility boundary -------------------------------------------------

fn vis_findings(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<String>, String> {
    // The Crate ceiling (rank 2) — the `must_not_declare_pub` case existing tests assert.
    vis_findings_at(name, files, module, VisibilityCeiling::Crate.rank())
}

fn vis_findings_at(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    ceiling_rank: u8,
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("vis-{name}"));
    tree.write_all(files);
    let result = visibility_findings(tree.src(), &tree.root(), module, "x", ceiling_rank);
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

#[test]
fn visibility_rank_is_false_negative_safe_for_every_form() {
    use crate::syn_util::visibility_rank;
    let rank = |vis: &str| {
        let src = format!("{vis} fn f() {{}}");
        visibility_rank(&syn::parse_str::<syn::ItemFn>(&src).expect("parse vis").vis)
    };
    assert_eq!(rank("pub"), 3);
    assert_eq!(rank("pub(crate)"), 2);
    assert_eq!(rank("pub(super)"), 1);
    assert_eq!(rank("pub(self)"), 0);
    assert_eq!(rank(""), 0, "inherited/private");
    assert_eq!(rank("pub(in crate)"), 2);
    assert_eq!(rank("pub(in super)"), 1);
    assert_eq!(rank("pub(in self)"), 0);
    assert_eq!(
        rank("pub(in crate::a::b)"),
        2,
        "in-crate path is at most crate-visible"
    );
    // The load-bearing false-negative guard: pub(in super::super) reaches the grandparent's whole
    // subtree — broader than pub(super) — so it must rank Crate (2), never Super (1). A first-segment
    // match ("super"->1) would silently pass it under a Super ceiling (the one forbidden bug).
    assert_eq!(rank("pub(in super::super)"), 2);
}

#[test]
fn super_ceiling_reacts_on_pub_and_pub_crate_only() {
    let out = vis_findings_at(
        "super-ceiling",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub fn a() {}\npub(crate) fn b() {}\npub(super) fn c() {}\nfn d() {}\n",
            ),
        ],
        "crate::m",
        VisibilityCeiling::Super.rank(),
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub fn a", "pub(crate) fn b"],
        "Super ceiling reacts on pub + pub(crate), not pub(super)/private: {out:?}"
    );
}

#[test]
fn module_ceiling_reacts_on_pub_super_but_not_private() {
    let out = vis_findings_at(
        "module-ceiling",
        &[
            ("lib.rs", "pub mod m;\n"),
            ("m.rs", "pub(super) fn c() {}\nfn d() {}\n"),
        ],
        "crate::m",
        VisibilityCeiling::Module.rank(),
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub(super) fn c"],
        "Module ceiling reacts on pub(super), not private: {out:?}"
    );
}

#[test]
fn pub_in_crate_path_is_clean_under_crate_ceiling() {
    let out = vis_findings_at(
        "pub-in-crate-path",
        &[
            ("lib.rs", "pub mod m;\n"),
            ("m.rs", "pub(in crate::a::b) fn f() {}\n"),
        ],
        "crate::m",
        VisibilityCeiling::Crate.rank(),
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "pub(in crate path) is at most crate-visible, clean under a Crate ceiling: {out:?}"
    );
}

#[test]
fn pub_in_super_super_reacts_under_super_ceiling() {
    // The conservative upper bound in action: pub(in super::super) ranks Crate (2), which exceeds a
    // Super (1) ceiling, so it reacts — never silently passed as if it were pub(super).
    let out = vis_findings_at(
        "pub-in-super-super",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "pub mod b;\n"),
            ("a/b.rs", "pub(in super::super) fn f() {}\n"),
        ],
        "crate::a::b",
        VisibilityCeiling::Super.rank(),
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub(in super::super) fn f"],
        "multi-segment pub(in super::super) ranks Crate and reacts under Super: {out:?}"
    );
}

#[test]
fn max_visibility_and_the_sugar_carry_the_ceiling() {
    let sugar = VisibilityBoundary::in_crate("app")
        .module("crate::m")
        .must_not_declare_pub()
        .because("r");
    assert_eq!(sugar.ceiling(), VisibilityCeiling::Crate);
    assert_eq!(sugar.ceiling().rule(), VISIBILITY_RULE);

    let sup = VisibilityBoundary::in_crate("app")
        .module("crate::m")
        .max_visibility(VisibilityCeiling::Super)
        .because("r");
    assert_eq!(sup.ceiling(), VisibilityCeiling::Super);
}

#[test]
fn ceiling_rule_strings_are_distinct_across_the_semantic_family() {
    // Crate keeps the legacy string byte-for-byte (baseline stability); all three are distinct from
    // every other rule so (target, rule, finding) stays injective family-wide.
    assert_eq!(
        VisibilityCeiling::Crate.rule(),
        "must not declare pub items"
    );
    let all = [
        VISIBILITY_RULE,
        VISIBILITY_SUPER_RULE,
        VISIBILITY_MODULE_RULE,
        SIGNATURE_RULE,
        DYN_TRAIT_RULE,
        IMPL_TRAIT_RULE,
        ASYNC_EXPOSURE_RULE,
        TRAIT_IMPL_RULE,
        FORBIDDEN_MARKER_RULE,
    ];
    let set: std::collections::HashSet<&str> = all.iter().copied().collect();
    assert_eq!(
        set.len(),
        all.len(),
        "all semantic rule strings are distinct"
    );
}

#[test]
fn pub_items_react_and_non_pub_items_are_clean() {
    let out = vis_findings(
        "pub-mix",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "pub fn a() {}\npub struct B;\npub trait C {}\npub(crate) fn d() {}\npub(super) fn e() {}\nfn f() {}\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub fn a", "pub struct B", "pub trait C"],
        "only bare-pub items react: {out:?}"
    );
}

#[test]
fn a_pub_use_and_glob_react() {
    let out = vis_findings(
        "pub-use",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "pub use crate::db::Handle;\npub use crate::db::*;\npub(crate) use crate::db::Hidden;\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub use crate::db::*", "pub use crate::db::Handle"]);
}

#[test]
fn a_pub_submodule_reacts() {
    let out = vis_findings(
        "pub-mod",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "pub mod sub;\nmod hidden;\n"),
            ("internal/sub.rs", "\n"),
            ("internal/hidden.rs", "\n"),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub mod sub"]);
}

#[test]
fn a_bare_pub_item_in_a_non_pub_module_still_reacts() {
    let out = vis_findings(
        "pub-in-crate-mod",
        &[
            ("lib.rs", "pub(crate) mod internal;\n"),
            ("internal.rs", "pub fn helper() {}\n"),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub fn helper"],
        "the rule governs the declared pub keyword, not crate-reachability"
    );
}

#[test]
fn a_pub_extern_crate_and_pub_trait_alias_react() {
    // Bare-`pub` item kinds beyond the common set: a public crate re-export and a
    // public trait alias are observable bare-`pub` declarations and must react.
    let out = vis_findings(
        "extern-and-alias",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "pub extern crate serde;\npub trait Alias = Clone;\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub extern crate serde", "pub trait Alias (alias)"]);
}

#[test]
fn a_leading_colon_pub_use_is_rendered_and_distinct() {
    // `::external::X` and `external::X` are distinct declarations; the leading colon
    // must be rendered so they do not collide under dedup.
    let out = vis_findings(
        "leading-colon",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "pub use ::external::X;\npub use external::X;\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub use ::external::X", "pub use external::X"]);
}

#[test]
fn a_macro_export_macro_is_out_of_scope() {
    let out = vis_findings(
        "macro-export",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "#[macro_export]\nmacro_rules! m { () => {} }\npub(crate) fn helper() {}\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a #[macro_export] macro carries no pub keyword — out of declared scope: {out:?}"
    );
}

#[test]
fn a_macro_invocation_pub_item_is_a_documented_bound() {
    let out = vis_findings(
        "macro-gen",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "make_public!();\n"),
        ],
        "crate::internal",
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a macro-generated item is out of scope, not silently claimed: {out:?}"
    );
}

#[test]
fn a_cfg_gated_pub_item_is_observed_as_written() {
    let out = vis_findings(
        "cfg-pub",
        &[
            ("lib.rs", "pub mod internal;\n"),
            (
                "internal.rs",
                "#[cfg(feature = \"never\")]\npub fn gated() {}\n",
            ),
        ],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub fn gated"], "cfg is observed as-written");
}

#[test]
fn an_unknown_visibility_module_is_a_constitution_error() {
    let err = vis_findings(
        "vis-unknown",
        &[("lib.rs", "pub mod internal;\n"), ("internal.rs", "\n")],
        "crate::ghost",
    )
    .unwrap_err();
    assert_eq!(err, unknown_module_error("crate::ghost", "x"));
}

#[test]
fn an_inline_visibility_module_is_scanned() {
    let out = vis_findings(
        "vis-inline",
        &[("lib.rs", "pub mod internal { pub fn a() {} fn b() {} }\n")],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub fn a"]);
}

#[test]
fn the_visibility_builder_carries_severity() {
    let warn = VisibilityBoundary::in_crate("app")
        .module("crate::internal")
        .must_not_declare_pub()
        .warn()
        .because("advisory first");
    assert_eq!(warn.severity(), Severity::Warn);

    let enforce = VisibilityBoundary::in_crate("app")
        .module("crate::internal")
        .must_not_declare_pub()
        .because("enforced");
    assert_eq!(enforce.severity(), Severity::Enforce);
}

#[test]
fn a_generic_self_type_is_rendered_distinctly() {
    let out = locality_findings(
        "generic-self",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Command {}\n"),
            (
                "domain.rs",
                "use crate::command::Command;\npub struct W<T>(T);\nimpl Command for W<u8> {}\nimpl Command for W<u16> {}\n",
            ),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Command for crate::domain::W<u16>)",
            "crate::domain (impl crate::command::Command for crate::domain::W<u8>)"
        ]
    );
}

#[test]
fn distinct_trait_instantiations_for_one_self_type_stay_distinct_findings() {
    // `impl Convert<u8> for Foo` and
    // `impl Convert<u16> for Foo` are two distinct, coherent misplaced impls. The finding now
    // carries the anchor WITH its written generic args, so they stay two findings — previously both
    // collapsed to `crate::domain (impl for crate::domain::Foo)` and a baseline masked the second.
    let out = locality_findings(
        "generic-trait",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Convert<T> {}\n"),
            (
                "domain.rs",
                "use crate::command::Convert;\npub struct Foo;\nimpl Convert<u8> for Foo {}\nimpl Convert<u16> for Foo {}\n",
            ),
        ],
        "crate::command::Convert",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Convert<u16> for crate::domain::Foo)",
            "crate::domain (impl crate::command::Convert<u8> for crate::domain::Foo)"
        ],
        "two distinct trait instantiations for one self type must stay distinct: {out:?}"
    );
}

#[test]
fn array_length_differing_trait_instantiations_stay_distinct() {
    // Round-2 fix: the type renderer now includes an array length (`[u8; 4]` vs `[u8; 8]`), so
    // instantiations differing only in a const array length stay distinct findings (the renderer
    // previously emitted `[u8; _]`, collapsing them).
    let out = locality_findings(
        "array-arg",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Convert<T> {}\n"),
            (
                "domain.rs",
                "use crate::command::Convert;\npub struct Foo;\nimpl Convert<[u8; 4]> for Foo {}\nimpl Convert<[u8; 8]> for Foo {}\n",
            ),
        ],
        "crate::command::Convert",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Convert<[u8; 4]> for crate::domain::Foo)",
            "crate::domain (impl crate::command::Convert<[u8; 8]> for crate::domain::Foo)"
        ],
        "array-length-differing instantiations must stay distinct: {out:?}"
    );
}

#[test]
fn complex_length_arrays_of_different_element_types_stay_distinct() {
    // When an array length is an unrenderable const
    // expression (`N + 1`), the renderer must keep the ELEMENT type and mark only the length `_`
    // (`[u8; _]` / `[u16; _]`), never propagate `None` for the whole array. Round 2's Array arm
    // propagated `None`, routing both arrays into the caller's single shared `_` bucket — collapsing
    // even distinct element types into one finding so a baseline could mask the second exposure.
    let out = locality_findings(
        "complex-array-arg",
        &[
            ("lib.rs", "pub mod command;\npub mod domain;\n"),
            ("command.rs", "pub trait Convert<T> {}\n"),
            (
                "domain.rs",
                "use crate::command::Convert;\npub struct Foo;\nimpl<const N: usize> Convert<[u8; N + 1]> for Foo {}\nimpl<const N: usize> Convert<[u16; N + 1]> for Foo {}\n",
            ),
        ],
        "crate::command::Convert",
        &["crate::commands"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::domain (impl crate::command::Convert<[u16; _]> for crate::domain::Foo)",
            "crate::domain (impl crate::command::Convert<[u8; _]> for crate::domain::Foo)"
        ],
        "complex-length arrays of different element types must stay distinct, not collapse to one `_`: {out:?}"
    );
}

// --- forbidden-marker ----------------------------------------------------

fn marker_findings(
    name: &str,
    files: &[(&str, &str)],
    subtree: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("mark-{name}"));
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let result = forbidden_marker_findings(tree.src(), &tree.root(), subtree, &forbidden, "x");
    // The pure-heart tests assert on findings only; drop the per-finding module/file here.
    result.map(|v| {
        v.into_iter()
            .map(|(finding, _module, _file)| finding.to_string())
            .collect()
    })
}

#[test]
fn a_forbidden_derive_on_a_subtree_type_reacts_and_a_clean_type_does_not() {
    let out = marker_findings(
        "derive",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[derive(serde::Serialize)]\npub struct Order;\n#[derive(Clone, Debug)]\npub struct Plain;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, ["derive serde::Serialize on crate::domain::Order"]);
}

#[test]
fn a_serde_derive_path_and_cfg_attr_derive_react_by_leaf() {
    let out = marker_findings(
        "leaf-and-cfgattr",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[derive(serde_derive::Serialize)]\npub struct A;\n#[cfg_attr(feature = \"serde\", derive(serde::Serialize))]\npub struct B;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "derive serde::Serialize on crate::domain::B",
            "derive serde_derive::Serialize on crate::domain::A"
        ],
        "serde_derive path (leaf) and cfg_attr-wrapped derive both react, each rendered by its own \
         written derive path (so two same-leaf derives stay distinct): {out:?}"
    );
}

#[test]
fn a_hand_impl_outside_the_subtree_reacts_via_the_self_type() {
    let out = marker_findings(
        "hand-impl",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "impl serde::Serialize for crate::domain::Order {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::domain::Order in crate::wire"],
        "a hand impl written outside the subtree, for a subtree type, reacts: {out:?}"
    );
}

#[test]
fn a_foreign_or_prelude_self_type_is_not_a_governed_subtree_type() {
    // `impl Marker for Vec<u8>` (a local trait on a std type, orphan-
    // legal) must NOT react — Vec is not a type the crate defines, even though the bare `Vec` head
    // would fabricate `crate::domain::Vec` via the CurrentModule fallback. Cross-checking the self
    // type against the crate's actual type definitions rejects the fabrication.
    let out = marker_findings(
        "foreign-self",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Order;\npub trait Marker {}\nimpl Marker for Vec<u8> {}\nimpl Marker for Box<Order> {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a marker acquired by a foreign/prelude self type (Vec/Box) is not a subtree type: {out:?}"
    );
    // Control: the SAME marker on the real subtree type still reacts.
    let out = marker_findings(
        "foreign-self-control",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Order;\npub trait Marker {}\nimpl Marker for Order {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl Marker for crate::domain::Order in crate::domain"]
    );
}

#[test]
fn distinct_generic_marker_instantiations_stay_distinct_findings() {
    // `impl Marker<u8> for Order` and
    // `impl Marker<u16> for Order` are two distinct, coherent acquisitions. The finding now carries
    // the written trait path WITH its generic args (and the impl-site module), so they stay two
    // findings — a baseline accepting one cannot mask the other (previously both collapsed to
    // `impl Marker for crate::domain::Order`).
    let out = marker_findings(
        "generic-marker",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Order;\npub trait Marker<T> {}\nimpl Marker<u8> for Order {}\nimpl Marker<u16> for Order {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "impl Marker<u16> for crate::domain::Order in crate::domain",
            "impl Marker<u8> for crate::domain::Order in crate::domain",
        ],
        "two distinct generic instantiations must stay distinct findings: {out:?}"
    );
}

#[test]
fn unrenderable_generic_marker_instantiations_fail_loud_without_positional_identity() {
    // The ordinary trait renderer cannot distinguish these const expressions. Failing loud keeps
    // either acquisition from being hidden behind scan-order-derived public identity.
    let error = marker_findings(
        "const-marker",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Foo;\npub trait Marker<const M: usize> {}\nimpl Marker<{ 1 + 1 }> for Foo {}\nimpl Marker<{ 2 + 2 }> for Foo {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap_err();
    assert!(error.contains("stable structural label"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
fn a_forbidden_marker_on_a_local_type_alias_reacts() {
    // Round-2 fix (regression closed): a marker impl'd on a local type alias resolves through the
    // alias closure to the underlying defined subtree type, so it still reacts — the round-1
    // type-defs cross-check alone (aliases are not in type_defs) had silently dropped it.
    let out = marker_findings(
        "alias-self",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Real;\ntype Bar = Real;\npub trait Marker {}\nimpl Marker for Bar {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(out, ["impl Marker for crate::domain::Bar in crate::domain"]);
    // Chain: `type Bar = A; type A = Real` — the marker on `Bar` lands on the struct `Real` through
    // two alias hops, so it must still react (the landing check chases the alias chain to a defined
    // type). Guards against under-reacting on an alias-of-an-alias to a real subtree type.
    let out = marker_findings(
        "alias-of-alias-self",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Real;\ntype A = Real;\ntype Bar = A;\npub trait Marker {}\nimpl Marker for Bar {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(out, ["impl Marker for crate::domain::Bar in crate::domain"]);
}

#[test]
fn a_blanket_impls_own_generic_param_is_not_resolved_through_a_same_named_alias() {
    // Round-9 finding: resolve_self_type (containment.rs) resolved a bare self type exactly like
    // any other path reference, with no awareness that the identifier might be the impl's OWN
    // declared generic type parameter rather than a nominal type. `impl<T> Marker for T {}` (a
    // blanket impl — T is a parameter use, not a type) in a module that also happens to declare an
    // unrelated `use ... as T` alias resolved the self type through that alias, fabricating a
    // marker-acquisition finding on the aliased type even though the source never writes `impl
    // Marker for` it at all. The sibling exposure collectors already shadow an impl's own generic
    // params for every OTHER position (collect.rs::type_param_names); the marker gate's self-type
    // check lacked the identical shadowing. Fixed by threading each ImplSite's own
    // `type_params` (impl<T, ..>'s declared names) into resolve_self_type, which now drops a bare
    // self type matching one of them before any resolution is attempted.
    let out = marker_findings(
        "blanket-impl-generic-param-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::domain::sub::Innocent as T;\npub mod sub { pub struct Innocent; }\npub trait Marker {}\nimpl<T> Marker for T {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a blanket impl's own generic param T must not resolve through the unrelated `use ... as T` \
         alias in scope in that module — the source never impls Marker for Innocent: {out:?}"
    );
}

#[test]
fn a_blanket_impls_generic_param_is_shadowed_even_through_a_multi_segment_projection() {
    // Round-10 finding: round 9's fix (resolve_self_type) only recognized a BARE single-segment
    // self type as the impl's own generic parameter (via `Path::get_ident()`, which returns `None`
    // for anything with more than one segment). `impl<T> Marker for T::Assoc {}` -- T::Assoc is a
    // projection off the impl's own parameter, never a nominal type, exactly like the sibling
    // exposure collector's own `is_shadowed_param_path` already treats `T::Item` -- was therefore
    // never shadowed and still resolved the leading `T` through an unrelated same-named alias,
    // fabricating a marker-acquisition finding one segment deeper than round 9 closed. Fixed by
    // sharing `is_shadowed_param_path` (the leading-segment check, regardless of further segments)
    // between the exposure collector and resolve_self_type instead of a narrower private copy.
    let out = marker_findings(
        "blanket-impl-multi-segment-generic-param-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::domain::sub as T;\npub mod sub { pub struct Assoc; }\npub trait Marker {}\nimpl<T> Marker for T::Assoc {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a blanket impl's own generic param T must stay shadowed even in a projection T::Assoc, \
         never resolving through the unrelated `use ... as T` module alias: {out:?}"
    );
}

#[test]
fn a_qualified_path_self_type_off_the_impls_own_generic_param_is_not_resolved_through_an_alias() {
    // Round-11 finding: `resolve_self_type` had no `qself.is_none()` guard at all, unlike its
    // sibling `canonical_self_owner` (which excludes a qself'd self type from resolution
    // entirely). A QUALIFIED-path self type (`<T>::Item`) stores its own dependent type (`T`, the
    // impl's own generic parameter) in `qself.ty`, entirely OUTSIDE `path.segments` -- so
    // `is_shadowed_param_path`, which only ever inspects `path`, can never see it. The trailing
    // segments (`Item`) were resolved as an ordinary bare path instead, silently bypassing the
    // round-9/10 shadow through a third syntactic vector. `impl<T: HasItem> Marker<T> for <T>::Item
    // {}` is real, compiling Rust (the `Marker<T>` trait argument satisfies rustc's E0207
    // unconstrained-type-parameter check). Fixed by dropping any qself'd self type before the
    // shadow check even runs -- the same "not a placeable nominal path" treatment already given to
    // every other non-resolvable self-type shape.
    let out = marker_findings(
        "qself-bracket-projection-shadow-gap",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use crate::domain::sub::Innocent as Item;\npub mod sub { pub struct Innocent; }\npub trait HasItem { type Item; }\npub trait Marker<X> {}\nimpl<T: HasItem> Marker<T> for <T>::Item {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a qself'd self type dependent on the impl's own generic param T must not resolve its \
         trailing segment through the unrelated `use ... as Item` module alias: {out:?}"
    );
}

#[test]
fn a_forbidden_marker_on_an_alias_to_a_foreign_type_is_clean() {
    // A `type` alias defines no new type — coherence sees through it —
    // so a marker impl'd on an alias to a FOREIGN/prelude type governs no subtree type and must NOT
    // react, exactly like the byte-identical impl on the target itself. Round 2 over-broadened the
    // acceptance to every local alias name; the landing-type check restores the foreign-self principle.
    let out = marker_findings(
        "foreign-alias-self",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "type Baz = Vec<u8>;\ntype Named = String;\npub trait Marker {}\nimpl Marker for Baz {}\nimpl Marker for Named {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a marker on an alias to a foreign type (Vec<u8>/String) lands off the subtree — no finding: {out:?}"
    );
    // Control: an alias to a real subtree struct still reacts (the round-2 behavior preserved).
    let out = marker_findings(
        "local-alias-self-control",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Real;\ntype Bar = Real;\npub trait Marker {}\nimpl Marker for Bar {}\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(out, ["impl Marker for crate::domain::Bar in crate::domain"]);
}

#[test]
fn two_same_leaf_derives_on_one_type_stay_distinct() {
    // Round-2 fix (derive-form identity): `#[derive(a::Marker, b::Marker)]` — two distinct forbidden
    // derives sharing a leaf on one type — stay distinct findings, rendered by their written paths.
    let out = marker_findings(
        "dual-derive",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[derive(a::Marker, b::Marker)]\npub struct T;\n",
            ),
        ],
        "crate::domain",
        &["Marker"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "derive a::Marker on crate::domain::T",
            "derive b::Marker on crate::domain::T"
        ],
        "two same-leaf derives must stay distinct findings: {out:?}"
    );
}

#[test]
fn a_submodule_type_is_governed_and_a_sibling_is_not() {
    let out = marker_findings(
        "subtree",
        &[
            ("lib.rs", "pub mod domain;\npub mod domainx;\n"),
            ("domain.rs", "pub mod order;\n"),
            (
                "domain/order.rs",
                "#[derive(serde::Serialize)]\npub struct Order;\n",
            ),
            (
                "domainx.rs",
                "#[derive(serde::Serialize)]\npub struct Other;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["derive serde::Serialize on crate::domain::order::Order"],
        "a submodule type is governed; the prefix-colliding sibling crate::domainx is not: {out:?}"
    );
}

#[test]
fn a_same_leaf_different_trait_is_a_documented_false_positive() {
    let out = marker_findings(
        "leaf-fp",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[derive(rkyv::Serialize)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["derive rkyv::Serialize on crate::domain::Order"],
        "leaf-match reacts (accepted false positive; the finding now shows the written derive path, \
         rkyv::Serialize, making the leaf-only match visible)"
    );
}

#[test]
fn an_unresolvable_glob_self_type_is_a_documented_bound() {
    let out = marker_findings(
        "glob-self",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "use crate::domain::*;\nimpl serde::Serialize for Order {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a glob-imported self-type cannot be placed in the subtree — a stated bound: {out:?}"
    );
}

#[test]
fn a_nested_cfg_attr_derive_reacts() {
    // The review's blocker: `cfg_attr(a, cfg_attr(b, derive(X)))` must still yield X.
    let out = marker_findings(
        "nested-cfgattr",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[cfg_attr(all(), cfg_attr(all(), derive(serde::Serialize)))]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, ["derive serde::Serialize on crate::domain::Order"]);
}

#[test]
fn a_malformed_derive_is_a_scan_error_not_a_silent_pass() {
    // `syn::parse_file` tokenizes attribute arguments lazily, so a struct whose `#[derive(...)]`
    // holds non-paths (a bare literal) parses as a *file* but cannot be read as a derive-path
    // list. "Cannot judge" is not "nothing to judge": the scan must surface an Err (which the
    // shell maps to exit 2), never swallow it and report the subtree clean — a silent pass here
    // would be the one forbidden bug (a forbidden derive could hide behind an unreadable one).
    let result = marker_findings(
        "malformed-derive",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "#[derive(0, \"nope\")]\npub struct Order;\n"),
        ],
        "crate::domain",
        &["serde::Serialize"],
    );
    let err =
        result.expect_err("a derive whose args are not paths must be a scan error, not clean");
    assert!(
        err.contains("cannot parse derive"),
        "the error must name the parse failure it could not judge: {err}"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_module_cycle_is_a_scan_error_not_a_stack_overflow() {
    // A cyclic symlinked module directory
    // (`src/foo/foo -> src/foo`) makes the file-backed `mod` walk revisit the same canonical file
    // forever. The scan must stop with a scan error ("cannot judge", exit 2), never recurse into a
    // stack overflow (SIGABRT). Driven through `forbidden_marker_findings`, which runs `scan_crate`
    // (the whole-crate walk) first.
    let tree = TempSrcTree::new("symcycle");
    tree.write("lib.rs", "pub mod foo;\n");
    tree.write("foo/mod.rs", "pub mod foo;\n");
    // src/foo/foo -> src/foo : crate::foo::foo resolves back through the symlink to foo/mod.rs.
    tree.symlink("../foo", "foo/foo");
    let result = forbidden_marker_findings(tree.src(), &tree.root(), "crate", &[], "x");
    let err =
        result.expect_err("a symlinked module cycle must be a scan error, not a hang/overflow");
    assert!(
        err.contains("module cycle") || err.contains("symlink"),
        "the error must name the cycle it could not judge: {err}"
    );
}

#[test]
fn two_same_named_types_in_different_submodules_stay_distinct() {
    // The review's baseline-collapse blocker: the finding must use the canonical path so
    // two `Order`s don't dedup into one (baselining one would else suppress the other).
    let out = marker_findings(
        "same-name",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub mod a;\npub mod b;\n"),
            (
                "domain/a.rs",
                "#[derive(serde::Serialize)]\npub struct Order;\n",
            ),
            (
                "domain/b.rs",
                "#[derive(serde::Serialize)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "derive serde::Serialize on crate::domain::a::Order",
            "derive serde::Serialize on crate::domain::b::Order"
        ],
        "two same-named types must stay distinct findings: {out:?}"
    );
}

#[test]
fn a_cfg_dual_declared_module_backed_by_one_file_does_not_duplicate_its_marker_finding() {
    // The forbidden-marker impl form shares resolve_child_modules/scan.impls with trait-impl-
    // locality, so it has the identical round-6 duplication hazard: two mutually-exclusive
    // #[cfg] arms declaring the same name resolving to one real file used to inflate one real
    // marker acquisition into two findings. Keep the owner renderable so this test isolates cfg
    // de-duplication; positional fallback rejection has its own reaction.
    let out = marker_findings(
        "cfg-dual-same-file",
        &[
            (
                "lib.rs",
                "pub struct Arr<const N: usize>;\n\
                 #[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\npub mod foo;\n",
            ),
            ("foo.rs", "impl crate::Marker for crate::Arr<2> {}\n"),
        ],
        "crate",
        &["crate::Marker"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "one real impl, backed by one real file under either #[cfg] arm, must be one finding: {out:?}"
    );
}

#[test]
fn the_forbidden_marker_builder_carries_severity() {
    let b = ForbiddenMarkerBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_acquire("serde::Serialize")
        .and_not_acquire("serde::Deserialize")
        .warn()
        .because("r");
    assert_eq!(b.forbidden(), &["serde::Serialize", "serde::Deserialize"]);
    assert_eq!(b.severity(), Severity::Warn);
}

// --- dyn-trait-boundary ---------------------------------------------------

/// Like [`findings`] but for the dyn-trait capability: write `files`, return the rendered
/// `dyn` shapes exposed by `module`. Shape-only, so it takes no forbidden set.
fn dyn_findings(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("dyn-{name}"));
    tree.write_all(files);
    let result = dyn_module_findings(tree.src(), &tree.root(), module, "x");
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

fn dyn_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    dyn_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

/// Like [`dyn_findings`] but for the operand-scoped rule: write `files`, return the rendered
/// `dyn` shapes whose principal trait resolves into `forbidden`.
fn dyn_operand_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("dynop-{name}"));
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|f| f.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|d| d.to_string()).collect();
    let result =
        dyn_operand_module_findings(tree.src(), &tree.root(), module, &forbidden, "x", &deps);
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

fn dyn_operand_mod(name: &str, body: &str, forbidden: &[&str]) -> Result<Vec<String>, String> {
    dyn_operand_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
        forbidden,
        &[],
    )
}

#[test]
fn a_dyn_in_a_supertrait_or_assoc_type_bound_is_observed() {
    // A `dyn` inside a supertrait's generic argument, or inside a
    // public associated type's `: Bound`, is a real exposed trait-object in the trait's public
    // contract. The sibling signature-coupling collector already walks these bound positions
    // (paths_in_bounds); the dyn collector now matches it — previously it skipped supertraits and
    // associated-type bounds entirely, silently dropping the dyn (a false negative → exit 0).
    assert!(
        dyn_mod(
            "supertrait-dyn",
            "pub trait Facade: AsRef<Box<dyn crate::ports::Port>> {}\n",
        )
        .unwrap()
        .contains(&"dyn crate::ports::Port exposed by trait crate::m::Facade".to_string()),
        "a dyn in a supertrait generic argument must be observed",
    );
    assert!(
        dyn_mod(
            "assoc-bound-dyn",
            "pub trait F { type Bar: AsRef<Box<dyn crate::ports::Port>>; }\n",
        )
        .unwrap()
        .contains(&"dyn crate::ports::Port exposed by type trait crate::m::F::Bar".to_string()),
        "a dyn in an associated-type bound must be observed",
    );
}

#[test]
fn a_dyn_in_an_inherent_impl_generic_bound_is_observed() {
    // Round-2 fix: a `dyn` in an inherent impl's own generic-param bound is exposed on the inherent
    // API; the dyn collector's inherent-impl arm now walks the impl generics (parity with the path
    // collector's fix #9 and with the struct/enum/trait arms).
    let out = dyn_mod(
        "dyn-impl-generics",
        "pub struct Foo<T>(T);\nimpl<T: AsRef<Box<dyn crate::ports::Port>>> Foo<T> { pub fn m(&self) {} }\n",
    )
    .unwrap();
    assert!(
        out.iter()
            .any(|f| f.contains("dyn crate::ports::Port") && f.contains("(generics)")),
        "a dyn in an inherent-impl generic bound must be observed: {out:?}"
    );
}

#[test]
fn dyn_operand_flags_a_named_trait_and_passes_others() {
    // A dyn of the listed trait is flagged; a dyn of an unlisted trait passes.
    assert_eq!(
        dyn_operand_mod(
            "named",
            "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
    assert!(
        dyn_operand_mod(
            "other",
            "pub fn e() -> Box<dyn std::error::Error> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "a dyn of an unlisted trait passes",
    );
}

#[test]
fn dyn_operand_honors_a_module_prefix() {
    // A module-prefix operand forbids any dyn of a trait under it (exact-or-`::` prefix).
    assert_eq!(
        dyn_operand_mod(
            "prefix",
            "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n",
            &["crate::ports"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
fn dyn_operand_matches_a_reexported_trait_by_its_defining_path() {
    // The trait is defined at crate::ports::Port and re-exported as crate::Port; the module
    // exposes `dyn crate::Port`. Forbidding either path matches — both canonicalize through
    // the re-export closure to the defining path.
    let files = &[
        (
            "lib.rs",
            "pub mod ports;\npub use crate::ports::Port;\npub mod m;\n",
        ),
        ("ports.rs", "pub trait Port {}\n"),
        ("m.rs", "pub fn c() -> Box<dyn crate::Port> { todo!() }\n"),
    ];
    // Forbid by the DEFINING path — the exposed facade `crate::Port` canonicalizes to it.
    assert_eq!(
        dyn_operand_findings(
            "reexport-defining",
            files,
            "crate::m",
            &["crate::ports::Port"],
            &[],
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::c"],
        "a dyn written through a re-export facade matches the forbidden defining path",
    );
}

#[test]
fn a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_principal() {
    // Round-7 finding: extern_resolution computed externs_type/renames_bare ONCE over the
    // flattened union of every #[cfg] branch's items (feeding operand_module_findings, backing
    // dyn-trait/impl-trait operand-scoped boundaries) -- the identical conflation round 6 fixed
    // for signature-coupling's use-map, left unfixed here too. The "u" branch (platform.rs)
    // declares a LOCAL `mod traits { .. }`; the mutually-exclusive "w" branch (win_platform.rs)
    // has no local `mod traits` at all and its own `dyn traits::Marker` genuinely names the real
    // extern crate `traits`. Before the fix, the "u" branch's local `mod traits` silently
    // suppressed the "w" branch's own genuine extern dyn-principal match.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")] pub mod platform;\n\
             #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
        ),
        (
            "platform.rs",
            "pub mod traits { pub trait Marker {} }\npub fn open() -> u8 { 0 }\n",
        ),
        (
            "win_platform.rs",
            "pub fn f() -> Box<dyn traits::Marker> { todo!() }\n",
        ),
    ];
    assert_eq!(
        dyn_operand_findings(
            "cfg-sibling-childmod-shadow",
            files,
            "crate::platform",
            &["traits::Marker"],
            &["traits"],
        )
        .unwrap(),
        ["dyn traits::Marker exposed by fn crate::platform::f"],
        "the w branch's own genuine extern dyn-principal must react, regardless of the u \
         branch's own local mod traits",
    );
}

#[test]
fn a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_own_extern_principal()
 {
    // Round-8 finding, the operand-scoped (`shape_scan.rs`/`crate_scope.rs`) analogue of
    // `a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_extern_reexport`
    // above: `operand_module_findings` groups its per-branch `FileExternScope` (and `uses_by_branch`)
    // by branch index too, not just by file — two INLINE `#[cfg]` siblings share the identical
    // enclosing lib.rs, so a file-keyed group would let the "u" arm's local `mod traits` suppress
    // the "w" arm's genuine extern `dyn traits::Marker`, the identical conflation the file-form
    // version above exercises, but with both arms declared inline in one shared file.
    let files = &[(
        "lib.rs",
        "#[cfg(feature = \"u\")] pub mod platform {\n\
         pub mod traits { pub trait Marker {} }\n\
         pub fn open() -> u8 { 0 }\n}\n\
         #[cfg(feature = \"w\")] pub mod platform {\n\
         pub fn f() -> Box<dyn traits::Marker> { todo!() }\n}\n",
    )];
    assert_eq!(
        dyn_operand_findings(
            "cfg-split-inline-inline-childmod-shadow",
            files,
            "crate::platform",
            &["traits::Marker"],
            &["traits"],
        )
        .unwrap(),
        ["dyn traits::Marker exposed by fn crate::platform::f"],
        "the w arm's own genuine extern dyn-principal must react, regardless of the u arm's own \
         local mod traits, even though both arms are inline and share lib.rs",
    );
}

#[test]
fn dyn_operand_ignores_auto_trait_markers() {
    // `dyn Port + Send`: the sole non-auto trait is Port. Forbidding Port flags it; forbidding
    // only the Send marker flags nothing (Send is an auto trait, never an operand, and a bare Send
    // does not resolve).
    assert_eq!(
        dyn_operand_mod(
            "marker-port",
            "pub fn c() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port + Send exposed by fn crate::m::c"],
    );
    assert!(
        dyn_operand_mod(
            "marker-send",
            "pub fn c() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
            &["Send"],
        )
        .unwrap()
        .is_empty(),
        "the trailing Send marker is not the operand",
    );
}

#[test]
fn dyn_operand_matches_when_an_auto_trait_is_written_before_the_principal() {
    // `dyn Send + crate::ports::Port` — the auto trait is written FIRST. Rust allows this (only
    // lifetimes are order-constrained), so the principal is not "the first trait bound"; skipping
    // auto traits, Port is the operand and forbidding it must flag the exposure. Taking the first
    // trait bound (Send) would silently pass a forbidden operand — a false negative.
    assert_eq!(
        dyn_operand_mod(
            "auto-first",
            "pub fn c() -> Box<dyn Send + crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn Send + crate::ports::Port exposed by fn crate::m::c"],
    );
    // Two auto traits before the principal is still resolved.
    assert_eq!(
        dyn_operand_mod(
            "auto-first-2",
            "pub fn c() -> Box<dyn Send + Sync + crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn Send + Sync + crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
fn dyn_operand_matches_a_dyn_nested_deep() {
    // Nested inside Vec<Box<dyn …>> — still matched by its principal trait.
    assert_eq!(
        dyn_operand_mod(
            "nested",
            "pub fn c() -> Vec<Box<dyn crate::ports::Port>> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
fn dyn_operand_empty_set_degenerates_to_any() {
    // An empty forbidden set reacts to any dyn — identical to shape-only, never a no-op.
    let body = "pub fn c() -> Box<dyn crate::ports::Port> { todo!() }\n";
    assert_eq!(
        dyn_operand_mod("empty", body, &[]).unwrap(),
        dyn_mod("empty-shape", body).unwrap(),
        "must_not_expose_dyn_of([]) matches exactly what shape-only must_not_expose_dyn does",
    );
    assert_eq!(
        dyn_operand_mod("empty2", body, &[]).unwrap(),
        ["dyn crate::ports::Port exposed by fn crate::m::c"],
    );
}

#[test]
fn dyn_operand_boundary_carries_its_operands_and_severity() {
    let b = DynTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_dyn_of(["crate::ports::Port"])
        .warn()
        .because("the core seam must not leak a dyn Port");
    assert_eq!(b.forbidden_operands(), ["crate::ports::Port"]);
    assert_eq!(b.severity(), Severity::Warn);
    // Shape-only still constructs an empty operand set (regression guard).
    let shape = DynTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_dyn()
        .because("no dyn at all");
    assert!(shape.forbidden_operands().is_empty());
}

// --- impl-trait-boundary (existential exposure) ---------------------------

/// Like [`dyn_findings`] but for the impl-trait capability: write `files`, return the rendered
/// `impl …` shapes returned by `module`'s public API.
fn impl_trait_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("impl-{name}"));
    tree.write_all(files);
    let result = impl_trait_module_findings(tree.src(), &tree.root(), module, "x");
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

fn impl_trait_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    impl_trait_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

#[test]
fn impl_trait_flags_a_returned_impl_trait() {
    assert_eq!(
        impl_trait_mod("ret", "pub fn make() -> impl crate::Port { todo!() }\n").unwrap(),
        ["impl crate::Port exposed by fn crate::m::make"],
    );
}

#[test]
fn impl_trait_flags_a_nested_returned_impl_trait() {
    assert_eq!(
        impl_trait_mod(
            "nested",
            "pub fn maybe() -> Option<impl crate::Port> { todo!() }\n"
        )
        .unwrap(),
        ["impl crate::Port exposed by fn crate::m::maybe"],
        "an impl Trait at depth in the return type is existential and reacts",
    );
}

#[test]
fn impl_trait_flags_a_trait_method_rpit() {
    assert_eq!(
        impl_trait_mod(
            "rpitit",
            "pub trait T { fn make(&self) -> impl crate::Port; }\n"
        )
        .unwrap(),
        ["impl crate::Port exposed by fn trait crate::m::T::make"],
        "a trait method's declared RPIT is the existential, governed at the declaration",
    );
}

#[test]
fn impl_trait_does_not_flag_an_argument_position() {
    // APIT is universal (a caller-chosen generic), not an existential leak.
    assert!(
        impl_trait_mod("apit", "pub fn drive(p: impl crate::Port) { let _ = p; }\n")
            .unwrap()
            .is_empty(),
        "argument-position impl Trait is not governed",
    );
}

#[test]
fn impl_trait_does_not_flag_an_async_fn() {
    // async fn leaks a compiler-inserted `impl Future`, not a written `impl Trait` — a
    // distinct, out-of-scope existential form (stated bound).
    assert!(
        impl_trait_mod("async", "pub async fn connect() -> u8 { 0 }\n")
            .unwrap()
            .is_empty(),
        "async fn's implicit impl Future is out of scope",
    );
}

#[test]
fn impl_trait_does_not_flag_a_private_fn_or_a_trait_impl_method() {
    // Private fn: not public API.
    assert!(
        impl_trait_mod("priv", "fn make() -> impl crate::Port { todo!() }\n")
            .unwrap()
            .is_empty(),
        "a private fn's RPIT is not public API",
    );
    // Trait-impl method: return shape dictated by the trait declaration (governed there).
    assert!(
        impl_trait_mod(
            "traitimpl",
            "pub struct S; impl crate::T for S { fn make(&self) -> impl crate::Port { todo!() } }\n"
        )
        .unwrap()
        .is_empty(),
        "a trait-impl method's return is not double-counted",
    );
}

#[test]
fn impl_trait_renders_iterator_and_fn_shapes_distinctly() {
    assert_eq!(
        impl_trait_mod(
            "iter",
            "pub fn it() -> impl Iterator<Item = u8> { todo!() }\n"
        )
        .unwrap(),
        ["impl Iterator<Item = u8> exposed by fn crate::m::it"],
    );
    assert_eq!(
        impl_trait_mod("clo", "pub fn f() -> impl Fn(i32) -> i32 { todo!() }\n").unwrap(),
        ["impl Fn(i32) -> i32 exposed by fn crate::m::f"],
    );
}

#[test]
fn impl_trait_boundary_carries_anchor_and_severity() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .warn()
        .because("the core seam must return named types");
    assert_eq!(b.crate_package(), "core");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
}

// --- operand-scoped impl-trait --------------------------------------------

fn impl_trait_operand_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("implop-{name}"));
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|f| f.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|d| d.to_string()).collect();
    let result = impl_trait_operand_module_findings(
        tree.src(),
        &tree.root(),
        module,
        &forbidden,
        "x",
        &deps,
    );
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

fn impl_trait_operand_mod(
    name: &str,
    body: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    impl_trait_operand_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
        forbidden,
        &[],
    )
}

#[test]
fn impl_trait_operand_flags_a_named_trait_and_passes_others() {
    assert_eq!(
        impl_trait_operand_mod(
            "named",
            "pub fn make() -> impl crate::ports::Port { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::make"],
    );
    // A returned impl Iterator (ergonomic existential) passes when only a domain port is forbidden.
    assert!(
        impl_trait_operand_mod(
            "iter",
            "pub fn it() -> impl Iterator<Item = u8> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "a returned impl of an unlisted (and bare-std) trait passes",
    );
}

#[test]
fn impl_trait_operand_honors_a_module_prefix() {
    assert_eq!(
        impl_trait_operand_mod(
            "prefix",
            "pub fn make() -> impl crate::ports::Port { todo!() }\n",
            &["crate::ports"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::make"],
    );
}

#[test]
fn impl_trait_operand_matches_a_reexported_trait_by_its_defining_path() {
    let files = &[
        (
            "lib.rs",
            "pub mod ports;\npub use crate::ports::Port;\npub mod m;\n",
        ),
        ("ports.rs", "pub trait Port {}\n"),
        ("m.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    assert_eq!(
        impl_trait_operand_findings("reexport", files, "crate::m", &["crate::ports::Port"], &[])
            .unwrap(),
        ["impl crate::Port exposed by fn crate::m::make"],
    );
}

#[test]
fn impl_trait_operand_ignores_auto_trait_markers() {
    assert_eq!(
        impl_trait_operand_mod(
            "marker-port",
            "pub fn make() -> impl crate::ports::Port + Send { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port + Send exposed by fn crate::m::make"],
    );
    assert!(
        impl_trait_operand_mod(
            "marker-send",
            "pub fn make() -> impl crate::ports::Port + Send { todo!() }\n",
            &["Send"],
        )
        .unwrap()
        .is_empty(),
        "the trailing Send marker is not the operand",
    );
}

#[test]
fn impl_trait_operand_matches_an_auto_trait_written_before_the_principal() {
    // `impl Send + crate::ports::Port` — auto trait first (valid Rust; impl-Trait bounds are an
    // unordered set). Skipping auto traits, Port is the operand and forbidding it must flag it.
    assert_eq!(
        impl_trait_operand_mod(
            "auto-first",
            "pub fn make() -> impl Send + crate::ports::Port { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl Send + crate::ports::Port exposed by fn crate::m::make"],
    );
}

#[test]
fn impl_trait_operand_matches_a_second_non_auto_trait() {
    // `impl crate::ports::Port + crate::ports::Sink` — a returned `impl Trait` may name several
    // non-auto traits. Forbidding the SECOND one must flag it: the returned type genuinely is a
    // Sink. Matching only the first non-auto trait would silently pass it (a false negative).
    assert_eq!(
        impl_trait_operand_mod(
            "second-trait",
            "pub fn make() -> impl crate::ports::Port + crate::ports::Sink { todo!() }\n",
            &["crate::ports::Sink"],
        )
        .unwrap(),
        ["impl crate::ports::Port + crate::ports::Sink exposed by fn crate::m::make"],
    );
}

#[test]
fn impl_trait_operand_matches_a_nested_returned_impl() {
    assert_eq!(
        impl_trait_operand_mod(
            "nested",
            "pub fn maybe() -> Option<impl crate::ports::Port> { todo!() }\n",
            &["crate::ports::Port"],
        )
        .unwrap(),
        ["impl crate::ports::Port exposed by fn crate::m::maybe"],
    );
}

#[test]
fn impl_trait_operand_empty_set_degenerates_to_any() {
    let body = "pub fn make() -> impl crate::ports::Port { todo!() }\n";
    assert_eq!(
        impl_trait_operand_mod("empty", body, &[]).unwrap(),
        impl_trait_mod("empty-shape", body).unwrap(),
        "must_not_expose_impl_trait_of([]) matches exactly what shape-only does",
    );
}

#[test]
fn impl_trait_operand_inherits_return_position_scoping() {
    // APIT and async fn stay out of scope under the operand variant too.
    assert!(
        impl_trait_operand_mod(
            "apit",
            "pub fn drive(p: impl crate::ports::Port) { let _ = p; }\n",
            &["crate::ports::Port"],
        )
        .unwrap()
        .is_empty(),
        "argument-position impl Trait is not governed even with a matching operand",
    );
    assert!(
        impl_trait_operand_mod(
            "async",
            "pub async fn c() -> u8 { 0 }\n",
            &["crate::ports::Port"]
        )
        .unwrap()
        .is_empty(),
    );
}

#[test]
fn impl_trait_operand_boundary_carries_operands_and_severity() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait_of(["crate::ports::Port"])
        .warn()
        .because("the core seam must not return an existential Port");
    assert_eq!(b.forbidden_operands(), ["crate::ports::Port"]);
    assert_eq!(b.severity(), Severity::Warn);
    let shape = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .because("no existential at all");
    assert!(shape.forbidden_operands().is_empty());
}

#[test]
fn impl_trait_boundary_carries_anchor_and_including_submodules() {
    let b = ImplTraitBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .warn()
        .because("the core seam must return named types, not an existential");
    assert_eq!(b.severity(), Severity::Warn);
    // The subtree opt-in defaults off and threads through `.because`.
    assert!(!b.including_submodules());
    let sub = ImplTraitBoundary::in_crate("core")
        .module("crate")
        .must_not_expose_impl_trait()
        .including_submodules()
        .because("no existential anywhere under the kernel");
    assert!(sub.including_submodules());
}

// --- impl-trait: subtree scope (`including_submodules`) -------------------

fn impl_trait_subtree(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<(String, String)>, String> {
    let tree = TempSrcTree::new(&format!("impl-sub-{name}"));
    tree.write_all(files);
    let result = impl_trait_subtree_findings(tree.src(), &tree.root(), module, "x");
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, module, _file)| (fact.to_string(), module))
            .collect()
    })
}

/// Just the finding strings, sorted — for cases where the module attribution rides inside the
/// finding string anyway.
fn impl_trait_subtree_labels(name: &str, files: &[(&str, &str)], module: &str) -> Vec<String> {
    impl_trait_subtree(name, files, module)
        .unwrap()
        .into_iter()
        .map(|(finding, _module)| finding)
        .collect()
}

#[test]
fn impl_trait_subtree_reacts_to_a_submodule_return_the_seam_scope_misses() {
    // The crux this opt-in exists for, mirroring async-exposure's own: a returned `impl Trait` in a
    // *submodule* is invisible to the default seam scope (anchored at `crate`, it sees only
    // crate-root items) — the gap the `no_existential_leak` composed profile's own honesty
    // requires closed for its impl-trait half. The subtree scope catches it.
    let files = &[
        ("lib.rs", "pub mod net;\n"),
        ("net.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    // Default seam scope at `crate` misses it entirely…
    assert_eq!(
        impl_trait_findings("seam-misses-sub", files, "crate").unwrap(),
        Vec::<String>::new(),
    );
    // …the subtree scope reacts, attributing it to the submodule.
    let subtree = impl_trait_subtree("sub-reacts", files, "crate").unwrap();
    assert_eq!(subtree.len(), 1);
    assert_eq!(subtree[0].1, "crate::net");
    assert!(subtree[0].0.contains("impl crate::Port"), "{:?}", subtree);
}

#[test]
fn impl_trait_subtree_includes_the_anchor_modules_own_seam_byte_identically() {
    // The anchor module's own returned `impl Trait` is still caught, and its finding string is
    // byte-identical to the single-module path — so enabling the opt-in on a seam-only boundary
    // adds deeper findings without re-identifying the seam ones (baseline stability).
    let files = &[
        ("lib.rs", "pub mod m;\n"),
        (
            "m.rs",
            "pub fn own() -> impl crate::Port { todo!() }\npub mod deep;\n",
        ),
        (
            "m/deep.rs",
            "pub fn nested() -> impl crate::Port { todo!() }\n",
        ),
    ];
    let seam = impl_trait_findings("seam-parity", files, "crate::m").unwrap();
    assert_eq!(seam.len(), 1);
    let subtree = impl_trait_subtree_labels("subtree-parity", files, "crate::m");
    assert_eq!(subtree.len(), 2);
    // The seam finding appears verbatim in the subtree result.
    assert!(subtree.contains(&seam[0]));
}

#[test]
fn impl_trait_subtree_scopes_to_the_anchored_subtree_not_the_whole_crate() {
    let files = &[
        ("lib.rs", "pub mod a;\npub mod c;\n"),
        (
            "a.rs",
            "pub mod b;\npub fn make() -> impl crate::Port { todo!() }\n",
        ),
        ("a/b.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
        ("c.rs", "pub fn make() -> impl crate::Port { todo!() }\n"),
    ];
    let subtree = impl_trait_subtree("bounded", files, "crate::a").unwrap();
    let modules: Vec<&str> = subtree.iter().map(|(_, m)| m.as_str()).collect();
    assert!(modules.contains(&"crate::a"));
    assert!(modules.contains(&"crate::a::b"));
    assert!(!modules.contains(&"crate::c"), "{:?}", modules);
}

#[test]
fn impl_trait_subtree_tolerates_a_cfg_gated_fileless_submodule() {
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"never\")]\npub mod optional;\npub mod present;\n",
        ),
        (
            "present.rs",
            "pub fn make() -> impl crate::Port { todo!() }\n",
        ),
    ];
    let subtree = impl_trait_subtree("cfg-fileless", files, "crate").unwrap();
    assert_eq!(subtree.len(), 1);
    assert_eq!(subtree[0].1, "crate::present");
}

#[test]
fn impl_trait_subtree_errors_on_a_non_cfg_missing_submodule() {
    let files = &[("lib.rs", "pub mod missing;\n")];
    let err = impl_trait_subtree("non-cfg-missing", files, "crate").unwrap_err();
    assert!(err.contains("missing"), "{err}");
}

#[test]
fn impl_trait_subtree_does_not_observe_a_body_nested_module() {
    let files = &[(
        "lib.rs",
        "pub fn outer() { mod inner { pub fn hidden() -> impl crate::Port { todo!() } } }\n",
    )];
    let subtree = impl_trait_subtree("body-nested", files, "crate").unwrap();
    assert!(subtree.is_empty(), "{:?}", subtree);
}

#[test]
fn impl_trait_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner() {
    // Mirrors `async_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner` exactly: impl-trait's
    // owner resolution (`canonical_self_owner`) produces an internal positional sentinel for a
    // genuinely unrenderable self type, caught by the shared `reject_positional_identity` gate
    // (invoked via `sort_attributed_facts`/`sort_faceted_facts`) — never published as identity,
    // under either scope.
    let files = &[(
        "lib.rs",
        "pub struct Arr<const N: usize>;\npub struct Marker;\nimpl Marker { pub fn before() -> impl crate::Port { todo!() } }\nimpl<const N: usize> Arr<{ N + 1 }> { pub fn unrenderable() -> impl crate::Port { todo!() } }\n",
    )];
    let seam = impl_trait_findings("const-generic-owner-parity-seam", files, "crate").unwrap_err();
    let subtree =
        impl_trait_subtree("const-generic-owner-parity-subtree", files, "crate").unwrap_err();
    assert!(seam.contains("without a stable structural label"), "{seam}");
    assert!(
        subtree.contains("without a stable structural label"),
        "{subtree}"
    );
    assert!(!seam.contains("_#") && !subtree.contains("_#"));
}

#[test]
fn impl_trait_subtree_cfg_branches_never_share_an_unrenderable_owner_fallback() {
    // Mirrors `async_cfg_branches_never_share_an_unrenderable_owner_fallback` exactly: two
    // mutually-exclusive `#[cfg]` branches of the same module each declare a same-named type with
    // an unrenderable const-generic self-type argument (`Arr<{ N + 1 }>` vs `Arr<{ N + 2 }>`). This
    // is the actual case task 1.3a's continuous-ordinal threading protects: a hardcoded or
    // reset-per-module ordinal would let the two branches' sentinels collide into one internal
    // value before `reject_positional_identity` ever runs. The gate still fails loud either way, so
    // this proves the ordinal is threaded correctly, not merely that the gate exists (the single-site
    // test above already proves the gate; this one proves the counter feeding it).
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod m;\n#[cfg(feature = \"w\")]\n#[path = \"m_w.rs\"]\npub mod m;\n",
        ),
        (
            "m.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 1 }> { pub fn run() -> impl crate::Port { todo!() } }\n",
        ),
        (
            "m_w.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 2 }> { pub fn run() -> impl crate::Port { todo!() } }\n",
        ),
    ];
    let error =
        impl_trait_subtree("cfg-split-owner-fallback-collision", files, "crate::m").unwrap_err();
    assert!(
        error.contains("without a stable structural label"),
        "{error}"
    );
    assert!(!error.contains("_#"), "{error}");
}

#[test]
fn impl_trait_operand_scoped_boundary_rejects_subtree_scope() {
    // A stated bound (not a silent gap): operand-scoping's per-branch principal-resolution
    // machinery is proven only over a single module, so combining it with subtree scope fails
    // loud with an actionable message rather than silently under- or mis-reacting.
    let (metadata, _fixture) = fixture_metadata(
        "impltrait-operand-subtree",
        &[("lib.rs", "pub fn make() -> impl crate::Port { todo!() }\n")],
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate")
        .must_not_expose_impl_trait_of(["crate::Port"])
        .including_submodules()
        .because("r");
    let mut violations = Vec::new();
    let err = check_impl_trait_boundary(&metadata, &boundary, &mut violations).unwrap_err();
    assert!(err.contains("not yet supported"), "{err}");
    assert!(violations.is_empty());
}

// --- async-exposure -------------------------------------------------------

fn async_findings(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("async-{name}"));
    tree.write_all(files);
    let result = async_exposure_module_findings(tree.src(), &tree.root(), module, "x");
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

fn async_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    async_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
}

fn async_observations(
    name: &str,
    body: &str,
) -> Result<Vec<(StructuredFactIdentity, String)>, String> {
    let tree = TempSrcTree::new(&format!("async-observation-{name}"));
    tree.write_all(&[("lib.rs", "pub mod registry;\n"), ("registry.rs", body)]);
    async_exposure_module_findings(tree.src(), &tree.root(), "crate::registry", "x").map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _)| {
                let finding = fact.into_finding();
                (finding.key().clone(), finding.text().to_string())
            })
            .collect()
    })
}

#[test]
fn pacta_shaped_registry_signature_changes_preserve_async_seam_identity() {
    let first = async_observations(
        "pacta-v1",
        "pub struct Registry;\npub struct Contract;\nimpl Registry { pub async fn register(&self, contract: Contract) {} }\n",
    )
    .unwrap();
    let second = async_observations(
        "pacta-v2",
        "pub struct Registry;\npub struct Receipt;\nimpl Registry { pub async fn register(&mut self, name: &str, version: u64) -> Receipt { Receipt } }\n",
    )
    .unwrap();
    assert_eq!(first.len(), 1);
    assert_eq!(second.len(), 1);
    assert_eq!(first[0].0, second[0].0);
    assert_ne!(first[0].1, second[0].1);
}

#[test]
fn async_production_violation_separates_target_rule_and_seam() {
    let (metadata, _fixture) = fixture_metadata(
        "async-identity",
        &[
            ("lib.rs", "pub mod registry;\n"),
            ("registry.rs", "pub async fn register(name: &str) {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::registry")
        .must_not_expose_async_fn()
        .because("registry operations keep a synchronous seam");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);

    let id = violations[0].id();
    assert_eq!(id.target(), "crate::registry");
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/async-exposure");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("including_submodules", "false")]
    );
    let fact = id.fact();
    assert_eq!(fact.fact_type(), "tianheng.fact/hunyi/async-exposure");
    assert_eq!(fact.shape(), "async-free-function");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![
            ("module", "crate::registry"),
            ("name", "register"),
            ("owner", "crate::registry"),
            ("owner_kind", "module"),
        ]
    );
}

#[test]
fn async_exposure_flags_a_public_async_free_fn() {
    assert_eq!(
        async_mod("free", "pub async fn connect() -> u8 { 0 }\n").unwrap(),
        ["async fn crate::m::connect() -> u8"],
    );
}

#[test]
fn async_exposure_flags_a_public_inherent_async_method() {
    assert_eq!(
        async_mod(
            "inherent",
            "pub struct Service; impl Service { pub async fn run(&self) {} }\n"
        )
        .unwrap(),
        ["async fn <crate::m::Service>::run(&self)"],
    );
}

#[test]
fn async_exposure_flags_a_public_trait_async_method_declaration() {
    assert_eq!(
        async_mod("trait", "pub trait Port { async fn fetch(&self) -> u8; }\n").unwrap(),
        ["async fn trait crate::m::Port::fetch(&self) -> u8"],
    );
}

#[test]
fn async_exposure_does_not_flag_trait_impl_private_or_nonasync() {
    // Trait-impl async method: dictated by the trait declaration — not double-counted.
    assert!(
        async_mod(
            "traitimpl",
            "pub struct S; impl crate::T for S { async fn run(&self) {} }\n"
        )
        .unwrap()
        .is_empty(),
    );
    // Private async fn: not public API.
    assert!(
        async_mod("priv", "async fn helper() {}\n")
            .unwrap()
            .is_empty(),
    );
    // Non-async public fn: not async.
    assert!(
        async_mod("sync", "pub fn ready() -> u8 { 0 }\n")
            .unwrap()
            .is_empty(),
    );
}

#[test]
fn async_exposure_finding_is_injective_across_same_named_owners() {
    // The crux: two same-named async methods across two inherent impls must NOT collide, or a
    // baselined one would mask the other (a false negative).
    let two_impls = async_mod(
        "two-impls",
        "pub struct A; pub struct B;\n\
         impl A { pub async fn run(&self) {} }\n\
         impl B { pub async fn run(&self) {} }\n",
    )
    .unwrap();
    assert_eq!(
        two_impls,
        [
            "async fn <crate::m::A>::run(&self)".to_string(),
            "async fn <crate::m::B>::run(&self)".to_string(),
        ],
        "same-named async methods across two impls yield two distinct owner-qualified findings",
    );
    // And two same-named async methods across two traits.
    let two_traits = async_mod(
        "two-traits",
        "pub trait T { async fn run(&self); }\npub trait U { async fn run(&self); }\n",
    )
    .unwrap();
    assert_eq!(
        two_traits,
        [
            "async fn trait crate::m::T::run(&self)".to_string(),
            "async fn trait crate::m::U::run(&self)".to_string(),
        ],
    );
}

#[test]
fn async_exposure_boundary_carries_anchor_and_severity() {
    let b = AsyncExposureBoundary::in_crate("core")
        .module("crate::core")
        .must_not_expose_async_fn()
        .warn()
        .because("the core seam is synchronous");
    assert_eq!(b.crate_package(), "core");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
    // The subtree opt-in defaults off and threads through `.because`.
    assert!(!b.including_submodules());
    let sub = AsyncExposureBoundary::in_crate("core")
        .module("crate")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("no async anywhere under the kernel");
    assert!(sub.including_submodules());
}

// --- async-exposure: subtree scope (`including_submodules`) ----------------

fn async_subtree(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<(String, String)>, String> {
    let tree = TempSrcTree::new(&format!("async-sub-{name}"));
    tree.write_all(files);
    let result = async_exposure_subtree_findings(tree.src(), &tree.root(), module, "x");
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, module, _file)| (fact.to_string(), module))
            .collect()
    })
}

/// Just the finding strings, sorted — for cases where the module attribution rides inside the
/// finding string anyway.
fn async_subtree_labels(name: &str, files: &[(&str, &str)], module: &str) -> Vec<String> {
    async_subtree(name, files, module)
        .unwrap()
        .into_iter()
        .map(|(finding, _module)| finding)
        .collect()
}

#[test]
fn async_subtree_reacts_to_a_submodule_async_fn_the_seam_scope_misses() {
    // The crux this whole opt-in exists for. A `pub async fn` in a *submodule* is invisible to the
    // default seam scope (anchored at `crate`, it sees only crate-root items) — the latent false
    // negative dogfooding `sans_io_pure` on 璇璣 surfaced. The subtree scope catches it.
    let files = &[
        ("lib.rs", "pub mod net;\n"),
        ("net.rs", "pub async fn connect() {}\n"),
    ];
    // Default seam scope at `crate` misses it entirely…
    assert_eq!(
        async_findings("seam-misses-sub", files, "crate").unwrap(),
        Vec::<String>::new(),
    );
    // …the subtree scope reacts, attributing it to the submodule.
    assert_eq!(
        async_subtree("sub-reacts", files, "crate").unwrap(),
        [(
            "async fn crate::net::connect()".to_string(),
            "crate::net".to_string()
        )],
    );
}

#[test]
fn async_subtree_includes_the_anchor_modules_own_seam_byte_identically() {
    // The anchor module's own async fn is still caught, and its finding string is byte-identical to
    // the single-module path — so enabling the opt-in on a seam-only boundary adds deeper findings
    // without re-identifying the seam ones (baseline stability).
    let files = &[
        ("lib.rs", "pub mod m;\n"),
        ("m.rs", "pub async fn own() {}\npub mod deep;\n"),
        ("m/deep.rs", "pub async fn nested() {}\n"),
    ];
    let seam = async_findings("seam-parity", files, "crate::m").unwrap();
    assert_eq!(seam, ["async fn crate::m::own()"]);
    let subtree = async_subtree_labels("subtree-parity", files, "crate::m");
    assert_eq!(
        subtree,
        [
            "async fn crate::m::deep::nested()",
            "async fn crate::m::own()",
        ],
    );
    // The seam finding appears verbatim in the subtree result.
    assert!(subtree.contains(&seam[0]));
}

#[test]
fn async_subtree_and_seam_both_fail_loud_on_an_unrenderable_owner() {
    let files = &[(
        "lib.rs",
        "pub struct Arr<const N: usize>;\npub struct Marker;\nimpl Marker { pub async fn before() {} }\nimpl<const N: usize> Arr<{ N + 1 }> { pub async fn unrenderable() {} }\n",
    )];
    let seam = async_findings("const-generic-owner-parity-seam", files, "crate").unwrap_err();
    let subtree = async_subtree("const-generic-owner-parity-subtree", files, "crate").unwrap_err();
    assert!(seam.contains("without a positional fallback"), "{seam}");
    assert!(
        subtree.contains("without a positional fallback"),
        "{subtree}"
    );
    assert!(!seam.contains("_#") && !subtree.contains("_#"));
}

#[test]
fn async_cfg_branches_never_share_an_unrenderable_owner_fallback() {
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod m;\n#[cfg(feature = \"w\")]\n#[path = \"m_w.rs\"]\npub mod m;\n",
        ),
        (
            "m.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 1 }> { pub async fn run() {} }\n",
        ),
        (
            "m_w.rs",
            "pub struct Arr<const N: usize>;\nimpl<const N: usize> Arr<{ N + 2 }> { pub async fn run() {} }\n",
        ),
    ];
    let error = async_subtree("cfg-split-owner-fallback-collision", files, "crate::m").unwrap_err();
    assert!(error.contains("without a positional fallback"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
fn async_subtree_reacts_through_inline_and_nested_modules() {
    // Inline `mod`, file `mod`, and a grandchild all react, each attributed to its own module.
    let files = &[
        (
            "lib.rs",
            "pub mod outer { pub async fn a() {} pub mod middle; }\n",
        ),
        ("outer/middle.rs", "pub async fn b() {}\npub mod leaf;\n"),
        ("outer/middle/leaf.rs", "pub async fn c() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("nested", files, "crate"),
        [
            "async fn crate::outer::a()",
            "async fn crate::outer::middle::b()",
            "async fn crate::outer::middle::leaf::c()",
        ],
    );
}

#[test]
fn async_subtree_anchored_at_an_inline_module_follows_its_own_further_path_child() {
    // rustc ground truth (verified with a real rustc build): `#[path = "moved/leaf.rs"]` written
    // inside an INLINE `mod outer { … }` accumulates outer's own directory as the base — the file
    // actually compiles at `outer/moved/leaf.rs`, never `moved/leaf.rs` (which would sit beside
    // lib.rs itself). `walk_subtree_modules` used to re-derive the anchor's own `#[path]`-base as
    // `file.parent()` — correct for a file-form anchor, but wrong for an INLINE anchor (the inline
    // body stays in the *enclosing* file, whose own directory is not the inline module's
    // accumulated one) — silently substituting the wrong base for anything the subtree walk
    // itself needs to resolve a further `#[path]` from. `resolve_module_root`'s own returned
    // `path_base` (this fix) is used directly instead of being re-derived.
    let files = &[
        (
            "lib.rs",
            "pub mod outer {\n    #[path = \"moved/leaf.rs\"]\n    pub mod leaf;\n}\n",
        ),
        ("outer/moved/leaf.rs", "pub async fn seam() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("inline-anchor-path-child", files, "crate::outer"),
        ["async fn crate::outer::leaf::seam()"],
    );
}

#[test]
fn async_subtree_walks_every_branch_of_a_cfg_split_anchor_not_just_the_first() {
    // rustc ground truth (verified with a real rustc build under either single-feature config):
    // `#[cfg(feature = "u")] pub mod foo;` (flat, own directory src/) paired with
    // `#[cfg(feature = "w")] #[path = "win/foo.rs"] pub mod foo;` (own directory src/win/) is the
    // standard per-platform shim — each arm plainly declares its OWN `pub mod bar;`, resolving to
    // a DIFFERENT real file (src/foo/bar.rs vs src/win/bar.rs). `resolve_module_root` correctly
    // unions both arms' items, but `walk_subtree_modules` used to thread only the FIRST arm's own
    // directory pair through to resolve those unioned items' own children — so the second arm's
    // `bar` silently resolved against the wrong directory and its own async fn was never observed.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\n#[path = \"win/foo.rs\"]\npub mod foo;\n",
        ),
        ("foo.rs", "pub mod bar;\n"),
        ("foo/bar.rs", "pub async fn unix_leaf() {}\n"),
        ("win/foo.rs", "pub mod bar;\n"),
        ("win/bar.rs", "pub async fn win_leaf() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("cfg-split-anchor-both-branches", files, "crate::foo"),
        [
            "async fn crate::foo::bar::unix_leaf()",
            "async fn crate::foo::bar::win_leaf()",
        ],
    );
}

#[test]
fn async_subtree_violations_name_each_branchs_own_file_not_a_shared_module_string_cache() {
    // Round-5 finding: async_exposure_subtree_findings correctly emits one finding per branch
    // (fixed above), both tagged with the identical module string "crate::foo::bar" (a legitimate
    // cfg-split: unix_leaf lives in foo/bar.rs, win_leaf in win/bar.rs). Before this redesign,
    // push_multi_module_violations resolved each finding's file via per_finding_file, a cache
    // keyed ONLY by that module string — so the first finding processed populated the cache with
    // one branch's file, and the second finding (from the OTHER branch) silently reused it. Every
    // multi-module finding now pairs with the real file its own branch was resolved from (from
    // the subtree walker itself), so each violation's file must name its own real branch.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-split-anchor-file-attribution",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\n#[path = \"win/foo.rs\"]\npub mod foo;\n",
            ),
            ("foo.rs", "pub mod bar;\n"),
            ("foo/bar.rs", "pub async fn unix_leaf() {}\n"),
            ("win/foo.rs", "pub mod bar;\n"),
            ("win/bar.rs", "pub async fn win_leaf() {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::foo")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("each branch's finding must name its own real file");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 2, "{violations:?}");
    let mut by_finding: std::collections::BTreeMap<String, &str> = Default::default();
    for v in &violations {
        by_finding.insert(
            v.finding.clone(),
            v.file
                .as_deref()
                .expect("a subtree finding carries its file"),
        );
    }
    assert!(
        by_finding["async fn crate::foo::bar::unix_leaf()"].ends_with("foo/bar.rs"),
        "unix_leaf must name foo/bar.rs: {by_finding:?}"
    );
    assert!(
        by_finding["async fn crate::foo::bar::win_leaf()"].ends_with("win/bar.rs"),
        "win_leaf must name win/bar.rs, never foo/bar.rs (a shared-cache misattribution): {by_finding:?}"
    );
}

#[test]
fn async_subtree_does_not_duplicate_a_file_shared_by_two_plain_cfg_siblings() {
    // rustc ground truth: `#[cfg(feature = "u")] pub mod foo;` and `#[cfg(feature = "w")] pub mod
    // foo;` (both PLAIN, no #[path]) are two mutually-exclusive declarations of the SAME name that
    // resolve to the IDENTICAL real file (foo.rs) — neither build ever compiles it twice.
    // descend()'s file-form search used to push one branch per matching declaration regardless of
    // whether they resolved to the same file, so foo.rs's own async fn was observed (and reported)
    // twice.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"u\")]\npub mod foo;\n#[cfg(feature = \"w\")]\npub mod foo;\n",
        ),
        ("foo.rs", "pub async fn seam() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("plain-cfg-siblings-one-file", files, "crate::foo"),
        ["async fn crate::foo::seam()"],
    );
}

#[test]
fn async_subtree_scopes_to_the_anchored_subtree_not_the_whole_crate() {
    // Anchored at `crate::a`, an async fn under `crate::a` reacts; a sibling `crate::c` does not —
    // the subtree is bounded by the anchor, not the crate.
    let files = &[
        ("lib.rs", "pub mod a;\npub mod c;\n"),
        ("a.rs", "pub async fn af() {}\npub mod b;\n"),
        ("a/b.rs", "pub async fn bf() {}\n"),
        ("c.rs", "pub async fn cf() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("bounded", files, "crate::a"),
        ["async fn crate::a::af()", "async fn crate::a::b::bf()"],
    );
}

#[test]
fn async_subtree_tolerates_a_cfg_gated_fileless_submodule() {
    // A `#[cfg]`-gated module with no file when the feature is off is tolerated (a stated bound),
    // not a scan error; the present modules still react.
    let files = &[
        (
            "lib.rs",
            "#[cfg(feature = \"absent\")]\npub mod gated;\npub mod present;\n",
        ),
        ("present.rs", "pub async fn here() {}\n"),
    ];
    assert_eq!(
        async_subtree_labels("cfg-tolerated", files, "crate"),
        ["async fn crate::present::here()"],
    );
}

#[test]
fn async_subtree_errors_on_a_non_cfg_missing_submodule() {
    // A non-`#[cfg]` `mod x;` with no file is a scan error (exit 2) — "cannot judge", never a
    // silent pass that would under-react.
    let files = &[("lib.rs", "pub mod gone;\n")];
    assert!(async_subtree("non-cfg-missing", files, "crate").is_err());
}

#[test]
fn async_subtree_distinguishes_same_named_async_methods_across_modules() {
    // Cross-module dedup safety (the invariant `push_multi_module_violations` rests on): it flattens
    // findings to identity `(anchor, rule, finding)`, discarding the enclosing module — so two
    // same-named inherent async methods in *different* submodules stay distinct ONLY because the
    // finding string carries the module-qualified owner. If that owner ever lost its module prefix,
    // baselining one would mask the other (a false negative). This pins it.
    let files = &[
        ("lib.rs", "pub mod a;\npub mod b;\n"),
        (
            "a.rs",
            "pub struct S;\nimpl S { pub async fn run(&self) {} }\n",
        ),
        (
            "b.rs",
            "pub struct S;\nimpl S { pub async fn run(&self) {} }\n",
        ),
    ];
    assert_eq!(
        async_subtree_labels("cross-mod-owners", files, "crate"),
        [
            "async fn <crate::a::S>::run(&self)",
            "async fn <crate::b::S>::run(&self)",
        ],
    );
}

#[test]
fn async_subtree_does_not_observe_a_body_nested_module() {
    // A `mod` declared inside a fn body is not part of the public module tree (its items are not
    // reachable as `crate::…`), so the subtree walk — which descends the public module tree, not fn
    // bodies — does not observe it. A stated bound: it is not public API, so async-exposure (which
    // governs the *public* seam) makes no claim about it, rather than silently asserting cleanliness.
    let files = &[(
        "lib.rs",
        "pub fn outer() { mod inner { pub async fn hidden() {} } }\n",
    )];
    assert_eq!(
        async_subtree_labels("body-nested", files, "crate"),
        Vec::<String>::new(),
    );
}

#[test]
fn dyn_in_public_return_param_and_field_react() {
    assert_eq!(
        dyn_mod(
            "ret",
            "pub fn connect() -> Box<dyn crate::Port> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::connect"]
    );
    assert_eq!(
        dyn_mod(
            "param",
            "pub fn drive(x: &dyn crate::Port) { let _ = x; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::drive"]
    );
    assert_eq!(
        dyn_mod("field", "pub struct S { pub p: Box<dyn crate::Port> }\n").unwrap(),
        ["dyn crate::Port exposed by field crate::m::S::p"]
    );
}

#[test]
fn dyn_reacts_at_any_nesting_depth() {
    assert_eq!(
        dyn_mod(
            "vec",
            "pub fn all() -> Vec<Box<dyn crate::Port>> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::all"]
    );
    assert_eq!(
        dyn_mod(
            "opt",
            "pub fn maybe(x: Option<&dyn crate::Port>) { let _ = x; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::maybe"]
    );
    // Nested inside an otherwise-static `impl Trait` return — still exposed to the caller.
    assert_eq!(
        dyn_mod(
            "impl-iter",
            "pub fn ports() -> impl Iterator<Item = Box<dyn crate::Port>> { std::iter::empty() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::ports"]
    );
}

#[test]
fn impl_trait_with_no_dyn_node_is_clean() {
    let out = dyn_mod(
        "impl-trait",
        "pub fn port() -> impl crate::Port { todo!() }\n",
    )
    .unwrap();
    assert!(out.is_empty(), "impl Trait carries no dyn node: {out:?}");
}

#[test]
fn dyn_in_const_static_trait_method_assoc_default_and_where_react() {
    assert_eq!(
        dyn_mod("const", "pub const C: &dyn crate::Port = todo!();\n").unwrap(),
        ["dyn crate::Port exposed by const crate::m::C"]
    );
    assert_eq!(
        dyn_mod("static", "pub static S: &dyn crate::Port = todo!();\n").unwrap(),
        ["dyn crate::Port exposed by static crate::m::S"]
    );
    assert_eq!(
        dyn_mod(
            "trait-method",
            "pub trait Service { fn port(&self) -> Box<dyn crate::Port>; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn trait crate::m::Service::port"]
    );
    assert_eq!(
        dyn_mod(
            "assoc-default",
            "pub trait Service { type Out = Box<dyn crate::Port>; }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by type trait crate::m::Service::Out"]
    );
    assert_eq!(
        dyn_mod(
            "where",
            "pub fn run<T>() where Box<dyn crate::Port>: Into<T> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port exposed by fn crate::m::run"]
    );
}

#[test]
fn dyn_in_an_inherent_impl_public_assoc_const_reacts() {
    // The dyn collector's inherent-impl arm now observes public associated `const`/`type`
    // positions (parity with the signature-coupling collector, which gained them this release), so a
    // `dyn` written in an inherent-impl `pub const` type reacts — it did not before.
    assert_eq!(
        dyn_mod(
            "inherent-assoc-const",
            "pub struct Config;\nimpl Config { pub const DEFAULT: &dyn crate::Port = todo!(); }\n",
        )
        .unwrap(),
        ["dyn crate::Port exposed by const <crate::m::Config>::DEFAULT"]
    );
}

#[test]
fn public_alias_target_reacts_but_named_alias_is_not_expanded() {
    // The public alias item's own target exposes dyn → reacts at the alias.
    assert_eq!(
        dyn_mod("alias-item", "pub type Handler = Box<dyn crate::Port>;\n").unwrap(),
        ["dyn crate::Port exposed by type crate::m::Handler"]
    );
    // A public fn naming a *private* alias: the alias is not expanded (stated bound), and a
    // private alias is not itself exposed — so the dyn escapes (the documented bound), the
    // only finding being none.
    let out = dyn_mod(
        "alias-named",
        "type Handler = Box<dyn crate::Port>;\npub fn make() -> Handler { todo!() }\n",
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "named private alias is not expanded: {out:?}"
    );
}

#[test]
fn internal_dyn_is_structurally_clean() {
    let out = dyn_mod(
        "internal",
        "fn helper() -> Box<dyn crate::Port> { todo!() }\nstruct Private { p: Box<dyn crate::Port> }\n",
    )
    .unwrap();
    assert!(out.is_empty(), "internal dyn is never exposed: {out:?}");
}

#[test]
fn dyn_with_multiple_bounds_renders_stably() {
    assert_eq!(
        dyn_mod(
            "bounds",
            "pub fn f() -> Box<dyn crate::Port + Send> { todo!() }\n"
        )
        .unwrap(),
        ["dyn crate::Port + Send exposed by fn crate::m::f"]
    );
}

#[test]
fn distinct_closures_and_nested_dyns_do_not_collide_into_one_finding() {
    // The boxed-closure family must render its full shape, not a degenerate placeholder —
    // else two distinct exposed `dyn` collapse to one finding and a new one is masked by a
    // baselined one (the one forbidden bug). `Fn`/`FnMut` differ, so two findings.
    let out = dyn_mod(
        "closures",
        "pub fn a(cb: Box<dyn Fn(i32) -> i32>) { let _ = cb; }\n\
         pub fn b(cb: Box<dyn FnMut(String) -> bool>) { let _ = cb; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn Fn(i32) -> i32 exposed by fn crate::m::a",
            "dyn FnMut(String) -> bool exposed by fn crate::m::b"
        ]
    );
    // A dyn nested inside another dyn's generic argument: BOTH are exposed dynamic
    // dispatch, so both react (any-depth node presence) — distinct, non-colliding findings.
    assert_eq!(
        dyn_mod(
            "nested",
            "pub fn f() -> Box<dyn crate::Foo<Box<dyn crate::Bar>>> { todo!() }\n"
        )
        .unwrap(),
        [
            "dyn crate::Bar exposed by fn crate::m::f",
            "dyn crate::Foo<Box<dyn crate::Bar>> exposed by fn crate::m::f"
        ]
    );
    // Associated-type bindings (`Iterator<Item = …>`, the most common assoc-bound dyn) keep
    // their payload — distinct item types stay distinct findings, not `dyn Iterator<_>`.
    let out = dyn_mod(
        "assoc",
        "pub fn a(x: Box<dyn Iterator<Item = u8>>) { let _ = x; }\n\
         pub fn b(x: Box<dyn Iterator<Item = u16>>) { let _ = x; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn Iterator<Item = u16> exposed by fn crate::m::b",
            "dyn Iterator<Item = u8> exposed by fn crate::m::a"
        ]
    );
    // Macro-typed and fn-pointer generic args render by name/shape, not a shared `dyn _`.
    let out = dyn_mod(
        "macro-fnptr",
        "pub fn a(x: Box<dyn crate::Foo<fn(i32)>>) { let _ = x; }\n\
         pub fn b(x: Box<dyn crate::Foo<fn(u8)>>) { let _ = x; }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn crate::Foo<fn(i32)> exposed by fn crate::m::a",
            "dyn crate::Foo<fn(u8)> exposed by fn crate::m::b"
        ]
    );
}

#[test]
fn same_shape_at_two_seams_stays_two_findings() {
    // The closed collision false-negative: two distinct public seams exposing the SAME dyn
    // shape must stay two findings, not collapse to one — else a new leak is masked by a
    // baselined one. Seam-qualification keeps them distinct.
    let out = dyn_mod(
        "two-seams",
        "pub fn a() -> Box<dyn crate::infra::Port> { todo!() }\n\
         pub fn b() -> Box<dyn crate::infra::Port> { todo!() }\n",
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "dyn crate::infra::Port exposed by fn crate::m::a",
            "dyn crate::infra::Port exposed by fn crate::m::b"
        ],
        "the same dyn shape at two seams must not collapse to one finding",
    );
    // The same guarantee for signature-coupling: two fns exposing the SAME forbidden type
    // stay two findings, one per seam.
    let out = findings(
        "two-seams-sig",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub fn a() -> crate::infra::DbPool { todo!() }\n\
                 pub fn b() -> crate::infra::DbPool { todo!() }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::DbPool exposed by fn crate::domain::a",
            "crate::infra::DbPool exposed by fn crate::domain::b"
        ],
        "the same forbidden type at two seams must not collapse to one finding",
    );
}

#[test]
fn the_dyn_trait_builder_carries_anchor_and_severity() {
    let b = DynTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_dyn()
        .warn()
        .because("the core seam is statically dispatched");
    assert_eq!(b.crate_package(), "app");
    assert_eq!(b.module(), "crate::core");
    assert_eq!(b.severity(), Severity::Warn);
    assert_eq!(b.reason(), "the core seam is statically dispatched");
}

#[test]
fn dyn_unknown_module_is_a_constitution_error() {
    let err = dyn_findings(
        "unknown",
        &[("lib.rs", "pub mod m;\n"), ("m.rs", "// nothing\n")],
        "crate::ghost",
    )
    .unwrap_err();
    assert_eq!(err, unknown_module_error("crate::ghost", "x"));
}

// --- semantic finding source file (the reaction-layer `file`) --------------

/// Write `files` under a unique temp `src`, resolve the governed `module`'s source file
/// (the file a single-module semantic violation reports), and return it. Cleans up; the
/// returned path is asserted by suffix, not existence.
fn resolve_file(name: &str, files: &[(&str, &str)], module: &str) -> Result<PathBuf, String> {
    let tree = TempSrcTree::new(&format!("file-{name}"));
    tree.write_all(files);
    resolve_module_file(tree.src(), &tree.root(), module, "x")
}

#[test]
fn module_file_is_the_crate_root_for_the_root_module() {
    let file = resolve_file("root", &[("lib.rs", "pub struct A;\n")], "crate").unwrap();
    assert!(file.ends_with("src/lib.rs"), "got {}", file.display());
}

#[test]
fn module_file_is_the_file_module_source() {
    let file = resolve_file(
        "filemod",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub struct A;\n"),
        ],
        "crate::domain",
    )
    .unwrap();
    assert!(file.ends_with("domain.rs"), "got {}", file.display());
}

/// A mutually-exclusive `#[cfg]` per-platform shim — an inline arm plus a file-form sibling
/// arm whose file is absent on this build — now resolves via the inline arm instead of hard
/// erroring on the sibling's missing file, aligning `descend` with `scan::resolve_child_modules`'s
/// identical `#[cfg]`-tolerance for a missing plain module file (previously the two walkers
/// silently disagreed on this exact shape — the 0.2.2 lesson).
#[test]
fn descend_tolerates_a_cfg_gated_missing_sibling_when_an_inline_arm_resolves() {
    let file = resolve_file(
        "cfg-shim",
        &[(
            "lib.rs",
            "#[cfg(unix)]\npub mod shared { pub struct A; }\n\
             #[cfg(windows)]\npub mod shared;\n",
        )],
        "crate::shared",
    )
    .expect("the inline arm resolves even though the windows-only file-form sibling has no file");
    assert!(file.ends_with("lib.rs"), "got {}", file.display());
}

/// When EVERY declaration for the anchored module is `#[cfg]`-gated and none resolves (no inline
/// sibling to fall back on), resolution still fails loud — never a silent, vacuous "zero items"
/// pass. `descend`'s own `next_branches.is_empty()` guard (which already existed for the ordinary
/// "no branch survived this segment" case) catches this for free: cfg-tolerance only ever removes
/// candidates, so an entirely-eliminated segment reads the same as an always-had genuinely unknown
/// one.
#[test]
fn descend_still_errors_when_every_candidate_for_a_module_is_cfg_gated_missing() {
    let err = resolve_file(
        "cfg-only-missing",
        &[("lib.rs", "#[cfg(feature = \"absent\")]\npub mod gated;\n")],
        "crate::gated",
    )
    .expect_err("a module with no surviving branch must be a scan error, never a vacuous pass");
    assert_eq!(err, unknown_module_error("crate::gated", "x"));
}

/// A BARE `#[cfg]`-gated missing file is tolerated (the sibling test above), but a
/// `#[cfg_attr(pred, …)]`-decorated one is NOT: unlike a bare `#[cfg]`, `cfg_attr` never removes
/// the `mod` item itself — it only conditionally applies its wrapped attribute — so the file must
/// always exist regardless of the predicate. Verified against a real `rustc` build: this exact
/// shape (`#[cfg_attr(unix, allow(dead_code))] mod gated;` with no `gated.rs`) is E0583 on every
/// platform. `has_cfg_attr` deliberately does not match `cfg_attr` for this reason.
#[test]
fn descend_does_not_tolerate_a_cfg_attr_decorated_missing_file_only_bare_cfg() {
    let err = resolve_file(
        "cfg-attr-not-tolerated",
        &[(
            "lib.rs",
            "#[cfg_attr(unix, allow(dead_code))]\npub mod gated;\n",
        )],
        "crate::gated",
    )
    .expect_err("a cfg_attr-decorated (not cfg-gated) missing file must still be a scan error");
    assert_eq!(err, missing_module_file_error("crate::gated", "x"));
}

/// A BARE `#[cfg(pred)]` co-occurring with an unconditional `#[path = "…"]` on the SAME item
/// removes the whole item, `#[path]` included, when `pred` is false — a standard per-platform
/// shim (`#[cfg(windows)] #[path = "windows_impl.rs"] mod imp;`) that must not hard-error `descend`
/// merely because this platform's target file was never written. Verified against a real `rustc`
/// build: this compiles cleanly with the target entirely absent. The mutually-exclusive inline
/// sibling arm (always present, no file needed) still resolves.
#[test]
fn descend_tolerates_a_cfg_gated_unconditional_path_target_when_missing() {
    let file = resolve_file(
        "cfg-path-shim",
        &[(
            "lib.rs",
            "#[cfg(unix)]\npub mod shared { pub struct A; }\n\
             #[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod shared;\n",
        )],
        "crate::shared",
    )
    .expect("the inline arm resolves even though the windows-only #[path] target has no file");
    assert!(file.ends_with("lib.rs"), "got {}", file.display());
}

/// The crate-wide walker (`scan::resolve_child_modules`, backing `semantic-unsafe-confinement`,
/// which has no single-module anchor mode) must tolerate the identical shape: a cfg-gated
/// unconditional `#[path]` target with no file must not fail the whole scan, so an unrelated
/// module's real `unsafe` site is still observed.
#[test]
fn resolve_child_modules_tolerates_a_cfg_gated_unconditional_path_target_when_missing() {
    let out = unsafe_labels(
        "cfg-path-shim-crate-wide",
        &[
            (
                "lib.rs",
                "#[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod imp;\npub mod live;\n",
            ),
            ("live.rs", "unsafe fn f() {}\n"),
        ],
        &["crate::allowed_elsewhere"],
    )
    .expect("a cfg-gated #[path] target with no file must not fail the crate-wide scan");
    assert_eq!(out, ["unsafe fn f in crate::live"]);
}

#[test]
fn module_file_is_mod_rs_for_a_nested_module() {
    let file = resolve_file(
        "nested",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain/mod.rs", "pub struct A;\n"),
        ],
        "crate::domain",
    )
    .unwrap();
    assert!(file.ends_with("domain/mod.rs"), "got {}", file.display());
}

#[test]
fn module_file_is_the_enclosing_file_for_an_inline_submodule() {
    // `crate::inner` is inline in lib.rs, so its file is lib.rs — never a (non-existent)
    // inner.rs. This is the case the naive "module name → <name>.rs" guess gets wrong.
    let file = resolve_file(
        "inline",
        &[("lib.rs", "pub mod inner { pub struct A; }\n")],
        "crate::inner",
    )
    .unwrap();
    assert!(file.ends_with("src/lib.rs"), "got {}", file.display());
}

#[test]
fn module_file_descends_a_deep_file_module() {
    let file = resolve_file(
        "deep",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "pub mod b;\n"),
            ("a/b.rs", "pub struct A;\n"),
        ],
        "crate::a::b",
    )
    .unwrap();
    assert!(file.ends_with("a/b.rs"), "got {}", file.display());
}

#[test]
fn module_file_follows_an_unconditional_path_on_an_inline_module_to_its_relocated_child() {
    // rustc ground truth (verified with a real `cargo check`): `#[path = "thread_files"] pub mod
    // thread { pub mod local_data; }` compiles `thread_files/local_data.rs` as
    // `crate::thread::local_data`, with no `src/thread/` directory at all — the naive
    // (non-relocated) location does not even exist. Before the fix, `descend`'s inline-collection
    // loop skipped ANY `#[path]`-bearing mod (inline or not) before ever checking its content,
    // and the file-form loop then also skipped it (assuming it was "already collected above"),
    // so the item vanished from both loops — `crate::thread` itself failed with a spurious
    // "module not found" error, even though it demonstrably exists and compiles.
    let file = resolve_file(
        "inline-path-relocate",
        &[
            (
                "lib.rs",
                "#[path = \"thread_files\"]\npub mod thread {\n    pub mod local_data;\n}\n",
            ),
            ("thread_files/local_data.rs", "pub struct A;\n"),
        ],
        "crate::thread::local_data",
    )
    .unwrap();
    assert!(
        file.ends_with("thread_files/local_data.rs")
            || file.ends_with("thread_files\\local_data.rs"),
        "got {}",
        file.display()
    );
}

/// Build fixtures under a temp `src` plus synthetic `cargo metadata --no-deps` for a single
/// crate `x` whose lib root is that `src/lib.rs`, so a private `check_*_boundary` can run
/// without spawning `cargo`. Returns `(metadata, tree)`; the tree's `Drop` removes the fixtures
/// once the caller drops it — hold it alive until after the check (the check reads the fixtures
/// from disk).
fn fixture_metadata(name: &str, files: &[(&str, &str)]) -> (Value, TempSrcTree) {
    let tree = TempSrcTree::new(&format!("meta-{name}"));
    tree.write_all(files);
    (tree.metadata(), tree)
}

#[test]
fn semantic_violation_carries_the_governed_module_file_not_the_types_file() {
    // The forbidden type `crate::infra::Db` is *defined* in infra.rs; the exposing seam is in
    // domain.rs. The reported `file` is the seam's module (domain.rs), the actionable one.
    let (metadata, _fixture) = fixture_metadata(
        "seam",
        &[
            ("lib.rs", "pub mod infra;\npub mod domain;\n"),
            ("infra.rs", "pub struct Db;\n"),
            (
                "domain.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("domain must not expose infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "one exposure violation");
    assert_eq!(violations[0].target, "crate::domain");
    assert_eq!(violations[0].rule, SIGNATURE_RULE);
    let id = violations[0].id();
    let key = id.fact();
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/signature-exposure");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![
            ("forbidden", "[\"crate::infra\"]"),
            ("including_trait_impls", "false"),
        ]
    );
    assert_eq!(key.fact_type(), "tianheng.fact/hunyi/signature-exposure");
    assert_eq!(key.shape(), "public-seam");
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![
            ("seam_kind", "free_fn"),
            ("seam_module", "crate::domain"),
            ("seam_name", "leak"),
            ("subject", "crate::infra::Db"),
        ]
    );
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(
        file.ends_with("domain.rs"),
        "the file is the seam's module (domain.rs), not the type's file (infra.rs): got {file}"
    );
}

#[test]
fn the_semantic_file_is_not_part_of_the_baseline_identity() {
    let (metadata, _fixture) = fixture_metadata(
        "baseline",
        &[
            ("lib.rs", "pub mod infra;\npub mod domain;\n"),
            ("infra.rs", "pub struct Db;\n"),
            (
                "domain.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("r");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    let v = &violations[0];
    assert!(v.file.is_some(), "the violation now carries a file");
    // `file` is metadata, not identity: a violation baselined while `file` was null still
    // matches once populated, so populating it never re-baselines or changes the count.
    assert_eq!(v.id(), v.clone().with_file(None).id());
}

#[test]
fn cfg_duplicated_inline_modules_are_all_governed() {
    // Two `#[cfg(..)] mod platform {..}` variants parse as separate inline modules (syn does not
    // evaluate cfg). A signature-coupling boundary anchored on `crate::platform` must observe BOTH:
    // resolving only the source-first variant let a forbidden exposure in the other pass unobserved
    // (exit 0) — a mod-resolution divergence, the forbidden false-negative class. Matches the
    // crate-wide scan's observe-all policy for same-named modules.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-dup-platform",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(unix)] pub mod platform { pub fn open() -> u8 { 0 } }\n\
                 #[cfg(windows)] pub mod platform { pub fn open() -> crate::infra::Db { unimplemented!() } }\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in any cfg variant");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the non-source-first cfg variant's exposure must react: {violations:?}"
    );
}

#[test]
fn cfg_mixed_inline_and_file_form_siblings_are_both_governed() {
    // rustc ground truth (verified with a real rustc build under EITHER single-feature config):
    // `#[cfg(feature = "a")] pub mod platform { .. }` (inline) and `#[cfg(feature = "b")] pub mod
    // platform;` (file-form, backed by platform.rs) is the standard per-platform shim pairing an
    // inline variant with a file-form one — valid, common Rust, not a name collision. `descend`
    // used to return as soon as it found ANY inline variant, never reading the file-form sibling
    // at all: a boundary anchored on `crate::platform` observed only the inline arm's exposures,
    // silently missing the file-form arm's — a real false negative (the resolver never even
    // opened platform.rs). Both must react now, matching the crate-wide scan's own cfg-blind,
    // observe-all policy for same-named children.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-inline-file",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform { pub fn open() -> u8 { 0 } }\n\
                 #[cfg(feature = \"b\")] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in any cfg variant, inline or file-form");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the file-form sibling's exposure must react even though an inline variant exists: {violations:?}"
    );
}

#[test]
fn a_semantic_boundary_anchored_at_an_inline_module_with_an_unconditional_path_reacts_instead_of_erroring()
 {
    // Before the fix, ANY single-module-anchored capability (signature-coupling-exposure,
    // dyn-trait-boundary, impl-trait-boundary, visibility-boundary, and async-exposure's
    // non-subtree seam) hard-failed with a spurious "module not found" (exit 2) when anchored at
    // an inline module carrying an unconditional `#[path]` — or any of its descendants — even
    // though hunyi's own crate-wide walker (`walk_subtree_modules`/`resolve_child_modules`)
    // resolved the identical layout without trouble. This asserts the single-module path now
    // agrees with the crate-wide one: the boundary must react on the real exposure, not error.
    let (metadata, _fixture) = fixture_metadata(
        "inline-path-boundary",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[path = \"thread_files\"]\npub mod thread {\n    pub mod local_data;\n}\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "thread_files/local_data.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::thread::local_data")
        .must_not_expose("crate::infra")
        .because("an inline module's own #[path] must not make its children unresolvable");
    let mut violations = Vec::new();
    let result = check_boundary(&metadata, &boundary, &mut violations);
    result.expect("crate::thread::local_data must resolve, not hard-error as an unknown module");
    assert_eq!(
        violations.len(),
        1,
        "the relocated child's exposure must still be observed: {violations:?}"
    );
}

#[test]
fn a_further_segment_beneath_a_flat_file_form_cfg_sibling_resolves_from_its_own_directory() {
    // rustc ground truth (verified with a real rustc build under the "b" feature): a flat
    // (non-`mod.rs`) file-form cfg sibling's OWN `#[path]` resolves relative to ITS OWN
    // containing directory, not `<child_dir>/<its own name>/` — the same rule an ordinary flat
    // file always follows, regardless of whether it also happens to pair with a mutually-
    // exclusive `#[cfg]` inline sibling. Before the fix, descend()'s merged-branch case
    // unconditionally continued a further segment from the INLINE sibling's accumulated
    // directory, which only coincides with a `mod.rs`-style file-form sibling's own directory —
    // silently misresolving (or hard-erroring on) the real target for a flat one instead.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-flat-further-segment",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod plat { pub struct Marker; }\n\
                 #[cfg(feature = \"b\")] #[path = \"moved/plat_moved.rs\"] pub mod plat;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "moved/plat_moved.rs",
                "#[path = \"elsewhere.rs\"]\npub mod target;\n",
            ),
            (
                "moved/elsewhere.rs",
                "pub fn get() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::plat::target")
        .must_not_expose("crate::infra")
        .because(
            "plat::target must not expose infra even through a flat cfg-sibling's own #[path]",
        );
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the flat file-form sibling's own #[path] target (moved/elsewhere.rs, a SIBLING of \
         plat_moved.rs, not a child of a plat/ subdirectory) must be read and react: {violations:?}"
    );
}

#[test]
fn a_plain_child_of_a_path_remapped_module_resolves_from_the_remaps_own_directory() {
    // rustc ground truth (verified with a real rustc build): `#[path = "moved/thing.rs"] pub mod
    // net;` makes `moved/thing.rs` mod-rs-like, so its own plain `pub mod inner;` resolves to
    // `moved/inner.rs`, NOT `net/inner.rs` (a name-derived location that has nothing to do with
    // where the file actually lives). descend()'s Branch redesign correctly threads `path_base`
    // for a FURTHER `#[path]` beneath a `#[path]`-loaded file, but a `child_dir` bug left the
    // CONVENTIONAL-child continuation still computed as the naive `<child_dir>/<seg>` regardless
    // of origin — silently resolving a plain child of a #[path]-remapped module at the wrong,
    // uncompiled location.
    let (metadata, _fixture) = fixture_metadata(
        "path-remap-plain-child",
        &[
            (
                "lib.rs",
                "pub mod infra;\n#[path = \"moved/thing.rs\"]\npub mod net;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("moved/thing.rs", "pub mod inner;\n"),
            (
                "moved/inner.rs",
                "pub fn get() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::net::inner")
        .must_not_expose("crate::infra")
        .because("net::inner must not expose infra even though net is #[path]-remapped");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "moved/inner.rs (the real, rustc-compiled file) must be read and react: {violations:?}"
    );
}

#[test]
fn cfg_mixed_plain_and_path_remapped_file_form_siblings_are_both_governed() {
    // rustc ground truth (verified with a real rustc build under either single-feature config):
    // `#[cfg(feature = "a")] pub mod platform;` (plain, backed by platform.rs) paired with
    // `#[cfg(feature = "b")] #[path = "win_platform.rs"] pub mod platform;` (remapped) is the
    // standard per-platform shim between two NON-inline variants — valid, common Rust, and once
    // #[path]-following exists the two variants need not name the same file at all. descend()'s
    // file-form search used to `break` at the first non-inline match regardless of source order,
    // silently dropping whichever variant did not win the race. Matching
    // `resolve_child_modules`'s own crate-wide policy (which never breaks after one match),
    // EVERY non-inline declaration for a segment now produces its own branch.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-plain-and-remapped-file-form",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform;\n\
                 #[cfg(feature = \"b\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("platform.rs", "pub fn open() -> u8 { 0 }\n"),
            (
                "win_platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in either the plain or #[path]-remapped arm");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the #[path]-remapped sibling's exposure must react even though a plain sibling was \
         declared first in source order: {violations:?}"
    );
}

#[test]
fn a_cfg_mixed_single_module_violation_names_the_offending_sibling_not_the_first_branch() {
    // Round-5 finding: resolve_module_root unions every surviving branch's ITEMS (fixed above —
    // the violation still fires) but used to always report `branches[0]`'s FILE regardless of
    // which branch actually produced the finding. Here the plain, clean `platform;` arm is
    // declared FIRST (branches[0]) and the offending #[path]-remapped `win_platform.rs` arm is
    // declared second — before the fix, `.file` named platform.rs, which contains no reference to
    // `crate::infra` at all. Every single-module finding now pairs with the real file its own
    // item's branch was resolved from, so `.file` must name win_platform.rs, where the offending
    // seam is actually written.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-file-names-offending-branch",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform;\n\
                 #[cfg(feature = \"b\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("platform.rs", "pub fn open() -> u8 { 0 }\n"),
            (
                "win_platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("the reported file must name the sibling that actually exposes infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "{violations:?}");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a semantic exposure violation carries its source file");
    assert!(
        file.ends_with("win_platform.rs"),
        "expected the offending sibling win_platform.rs, got {file} — a clean file must never \
         be reported as the source of a real violation: {violations:?}"
    );
}

#[test]
fn a_cfg_split_module_does_not_let_one_arms_use_alias_shadow_the_others() {
    // Round-6 finding: module_findings called collect_uses ONCE over the flattened union of every
    // #[cfg] branch's items, so two mutually-exclusive branches each declaring `use <different
    // path> as Handle;` collided in one shared use-map -- the branch unioned LAST silently
    // overwrote the earlier branch's mapping, misresolving the FIRST branch's own bare `Handle`
    // reference through the SECOND branch's `use` and hiding a real forbidden-exposure finding.
    // Verified against real rustc: both platform.rs and win_platform.rs compile cleanly under
    // their own respective feature. A control fixture with the identical platform.rs but no cfg
    // split correctly reports 1 violation, confirming this is a cfg-split-specific regression,
    // not a general resolution gap.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-split-use-alias-collision",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod other;\n\
                 #[cfg(feature = \"u\")] pub mod platform;\n\
                 #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("other.rs", "pub struct Widget;\n"),
            (
                "platform.rs",
                "use crate::infra::Db as Handle;\npub fn leak() -> Handle { unimplemented!() }\n",
            ),
            (
                "win_platform.rs",
                "use crate::other::Widget as Handle;\npub fn leak2() -> Handle { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("the unix arm's own Handle alias must resolve to infra, not the windows arm's");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the unix arm's leak() -> Handle genuinely exposes crate::infra::Db and must react: {violations:?}"
    );
}

#[test]
fn a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_reexport() {
    // Round-7 finding: module_findings still computed child_mods/externs_type/externs_reexport/
    // renames_bare ONCE over the flattened union of every #[cfg] branch's items -- the identical
    // conflation round 6 fixed for the use-map, left unfixed here. The "u" branch (platform.rs)
    // declares a LOCAL `mod net { .. }`; the mutually-exclusive "w" branch (win_platform.rs) has
    // no local `mod net` at all and its own `pub use net::Something;` genuinely names the real
    // extern crate `net` -- verified against real rustc/cargo (win_platform.rs alone, with the
    // `net` dependency declared, compiles cleanly). Before the fix, the "u" branch's local `mod
    // net` silently suppressed the "w" branch's own genuine extern re-export, since child_mods
    // (computed over the union) always contained "net".
    let out = findings_with_deps(
        "cfg-sibling-childmod-shadow",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"u\")] pub mod platform;\n\
                 #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            (
                "platform.rs",
                "pub mod net { pub struct Something; }\npub fn open() -> u8 { 0 }\n",
            ),
            ("win_platform.rs", "pub use net::Something;\n"),
        ],
        "crate::platform",
        &["net::Something"],
        &["net"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["net::Something exposed by pub use crate::platform::Something"],
        "the w branch's own genuine extern re-export must react, regardless of the u branch's \
         own local mod net: {out:?}"
    );
}

#[test]
fn a_cfg_split_module_with_two_inline_siblings_does_not_let_one_arms_use_alias_shadow_the_others() {
    // Round-8 finding: `descend()` used to MERGE every same-named inline `#[cfg]` occurrence into
    // one shared `Branch` before this whole per-file fix (round 6) even had a chance to run, so
    // the round-6/7 "per-file" use-map/shadow-set grouping was structurally a no-op for two INLINE
    // siblings — they always shared one `Branch`, one merged items list, one merged use-map,
    // regardless of which file's identity that fix grouped by. `descend()` now gives each inline
    // occurrence its OWN branch (mirroring the file-form loop), but two inline siblings still
    // share the identical ENCLOSING file (lib.rs here) — so `resolve_module_items_with_files`
    // pairs each item with a BRANCH INDEX, not just a file, and `module_findings` groups by that
    // index. This is the identical `a_cfg_split_module_does_not_let_one_arms_use_alias_shadow_the_others`
    // scenario (round 6), but with BOTH arms declared INLINE in the SAME file rather than as two
    // separate file-form siblings — exercising the file-keyed grouping's own blind spot.
    let out = findings(
        "cfg-split-inline-inline-use-alias-collision",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod other;\n\
             #[cfg(feature = \"u\")] pub mod platform {\n\
             use crate::infra::Db as Handle;\n\
             pub fn leak() -> Handle { unimplemented!() }\n}\n\
             #[cfg(feature = \"w\")] pub mod platform {\n\
             use crate::other::Widget as Handle;\n\
             pub fn leak2() -> Handle { unimplemented!() }\n}\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("other.rs", "pub struct Widget;\n"),
        ],
        "crate::platform",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Db exposed by fn crate::platform::leak"],
        "the u arm's own Handle alias must resolve to infra, not the w arm's, even though both \
         arms are inline and share lib.rs: {out:?}"
    );
}

#[test]
fn a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_extern_reexport()
 {
    // Round-8 finding, the childmod/extern-reexport analogue of the test above (round 7's own
    // file-form version is `a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_reexport`).
    // The "u" arm declares a LOCAL `mod net { .. }` inline; the mutually-exclusive "w" arm — also
    // inline, sharing the identical lib.rs — has no local `mod net` at all, so its own `pub use
    // net::Something;` genuinely names the real extern crate `net`. Grouping by file alone would
    // let the "u" arm's local `mod net` suppress the "w" arm's genuine extern re-export merely
    // because both share one file; grouping by branch index keeps them apart.
    let out = findings_with_deps(
        "cfg-split-inline-inline-childmod-shadow",
        &[(
            "lib.rs",
            "#[cfg(feature = \"u\")] pub mod platform {\n\
             pub mod net { pub struct Something; }\n\
             pub fn open() -> u8 { 0 }\n}\n\
             #[cfg(feature = \"w\")] pub mod platform {\n\
             pub use net::Something;\n}\n",
        )],
        "crate::platform",
        &["net::Something"],
        &["net"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["net::Something exposed by pub use crate::platform::Something"],
        "the w arm's own genuine extern re-export must react, regardless of the u arm's own \
         local mod net, even though both arms are inline and share lib.rs: {out:?}"
    );
}

#[test]
fn async_subtree_observes_both_arms_of_a_two_inline_sibling_cfg_split_anchor() {
    // Round-8 finding (b): when the async-exposure subtree boundary is anchored DIRECTLY at a
    // module reached through two mutually-exclusive INLINE `#[cfg]` siblings sharing one file,
    // `walk_subtree_modules` must observe EACH arm's own async fn — never merging the two arms'
    // items into one shared list (which happened to still union both fns correctly under the old
    // pre-round-8 `descend()`, since shape-only observation over a union list drops nothing) nor
    // dropping either arm now that `descend()` gives each its own `Branch` and its own
    // `collect_subtree` call (two entries sharing one file, each with only its own arm's items).
    let files = &[(
        "lib.rs",
        "#[cfg(feature = \"u\")] pub mod platform { pub async fn unix_seam() {} }\n\
         #[cfg(feature = \"w\")] pub mod platform { pub async fn win_seam() {} }\n",
    )];
    let mut labels =
        async_subtree_labels("inline-inline-cfg-split-anchor", files, "crate::platform");
    labels.sort();
    assert_eq!(
        labels,
        [
            "async fn crate::platform::unix_seam()",
            "async fn crate::platform::win_seam()",
        ],
        "both inline cfg arms' own async fns must be observed, even though they share lib.rs: {labels:?}"
    );
}

#[test]
fn a_visibility_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "vis",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "pub struct Leaked;\n"),
        ],
    );
    let boundary = VisibilityBoundary::in_crate("x")
        .module("crate::internal")
        .must_not_declare_pub()
        .because("internal exposes no pub");
    let mut violations = Vec::new();
    check_visibility_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "a pub item in internal violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("internal.rs"), "got {file}");
}

#[test]
fn a_trait_impl_locality_violation_carries_its_impl_site_file() {
    let (metadata, _fixture) = fixture_metadata(
        "locality",
        &[
            ("lib.rs", "pub mod plugins;\npub trait Command {}\n"),
            (
                "plugins.rs",
                "pub struct P;\nimpl crate::Command for P {}\n",
            ),
        ],
    );
    let boundary = TraitImplBoundary::in_crate("x")
        .trait_("crate::Command")
        .only_implemented_in("crate::allowed")
        .because("Command impls live in crate::allowed");
    let mut violations = Vec::new();
    check_trait_impl_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "the misplaced impl violates");
    let file = violations[0].file.as_deref().expect("the impl site's file");
    assert!(file.ends_with("plugins.rs"), "got {file}");
    // `file` is metadata, not identity.
    assert_eq!(
        violations[0].id(),
        violations[0].clone().with_file(None).id()
    );
}

#[test]
fn a_trait_impl_in_a_nested_module_resolves_to_mod_rs() {
    let (metadata, _fixture) = fixture_metadata(
        "locality-nested",
        &[
            ("lib.rs", "pub mod plugins;\npub trait Command {}\n"),
            (
                "plugins/mod.rs",
                "pub struct P;\nimpl crate::Command for P {}\n",
            ),
        ],
    );
    let boundary = TraitImplBoundary::in_crate("x")
        .trait_("crate::Command")
        .only_implemented_in("crate::allowed")
        .because("Command impls live in crate::allowed");
    let mut violations = Vec::new();
    check_trait_impl_boundary(&metadata, &boundary, &mut violations).unwrap();
    let file = violations[0].file.as_deref().expect("the impl site's file");
    assert!(file.ends_with("plugins/mod.rs"), "got {file}");
}

#[test]
fn forbidden_marker_impl_and_derive_each_name_their_own_module_file() {
    // A forbidden `impl` sits in internal.rs; a forbidden `#[derive]` sits on a type in
    // models.rs. Each finding must name its OWN module's file — the derive names the
    // defining type's file (models.rs), never the impl site's (internal.rs).
    let (metadata, _fixture) = fixture_metadata(
        "marker",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod models;\npub trait Secret {}\n",
            ),
            (
                "internal.rs",
                "pub struct Bar;\nimpl crate::Secret for Bar {}\n",
            ),
            ("models.rs", "#[derive(Secret)]\npub struct Foo;\n"),
        ],
    );
    let boundary = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate") // subtree = whole crate, so both Foo and Bar are under it
        .must_not_acquire("crate::Secret")
        .because("nothing may acquire Secret");
    let mut violations = Vec::new();
    check_forbidden_marker_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 2, "one impl finding + one derive finding");
    let impl_v = violations
        .iter()
        .find(|v| v.finding.starts_with("impl "))
        .expect("an impl finding");
    let derive_v = violations
        .iter()
        .find(|v| v.finding.starts_with("derive "))
        .expect("a derive finding");
    assert!(
        impl_v.file.as_deref().unwrap().ends_with("internal.rs"),
        "impl file: {:?}",
        impl_v.file
    );
    assert!(
        derive_v.file.as_deref().unwrap().ends_with("models.rs"),
        "derive file is the defining type's module, not an impl site: {:?}",
        derive_v.file
    );
}

#[test]
fn a_dyn_trait_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "dyn",
        &[
            ("lib.rs", "pub mod api;\npub trait Port {}\n"),
            (
                "api.rs",
                "pub fn f() -> Box<dyn crate::Port> { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_dyn()
        .because("the api seam is statically dispatched");
    let mut violations = Vec::new();
    check_dyn_trait_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the exposed dyn violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
fn an_impl_trait_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "impltrait",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "pub fn f() -> impl Iterator<Item = u8> { std::iter::empty() }\n",
            ),
        ],
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_impl_trait()
        .because("the api seam returns no existential");
    let mut violations = Vec::new();
    check_impl_trait_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the returned impl Trait violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
fn an_async_exposure_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "async",
        &[
            ("lib.rs", "pub mod api;\n"),
            ("api.rs", "pub async fn f() {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_async_fn()
        .because("the api seam exposes no async fn");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the async fn violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
fn a_facade_chain_reexport_reports_the_governed_module_file_not_the_facades() {
    // The exposing seam (`pub use crate::facade::Db;`) is in domain.rs; the type is defined in
    // infra.rs and hopped through facade.rs. The reported file is the seam's module
    // (domain.rs) — the actionable one — never the type's or the intermediate facade's file.
    let (metadata, _fixture) = fixture_metadata(
        "facade",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod facade;\npub mod domain;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("facade.rs", "pub use crate::infra::Db;\n"),
            ("domain.rs", "pub use crate::facade::Db;\n"),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("domain must not re-export infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "the facade-chain re-export violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(
        file.ends_with("domain.rs"),
        "the seam is in domain.rs, not infra.rs/facade.rs: got {file}"
    );
}

#[test]
fn path_remapped_module_resolves_to_its_target_not_the_conventional_orphan() {
    // `crate::domain` is `#[path = "weird.rs"]`, so it resolves to weird.rs — the file rustc
    // compiles — and NEVER to the same-named conventional orphan `domain.rs` (which rustc does not
    // compile). The FP-guard intent survives the switch from skip to follow: the target, not the
    // orphan.
    let file = resolve_file(
        "path-remap",
        &[
            (
                "lib.rs",
                "#[path = \"weird.rs\"]\npub mod domain;\npub mod normal;\n",
            ),
            ("weird.rs", "pub struct Real;\n"),
            ("domain.rs", "pub struct Orphan;\n"),
            ("normal.rs", "pub struct Normal;\n"),
        ],
        "crate::domain",
    )
    .expect("an unconditional #[path] module now resolves to its target");
    let file = file.display().to_string();
    assert!(
        file.ends_with("weird.rs"),
        "the resolver follows #[path] to weird.rs, never the conventional orphan domain.rs: {file}"
    );
}

#[test]
fn path_nested_in_an_inline_block_resolves_from_the_accumulated_dir_targeted() {
    // The targeted resolver's twin of the whole-crate walk fix. rustc ground truth (rustc 1.96.0):
    // `pub mod inline { #[path="other.rs"] pub mod inner; }` at the crate root resolves
    // crate::inline::inner to src/inline/other.rs. The earlier `descend` used current_file.parent()
    // (= src/) as the #[path] base, which drops the accumulated inline component — it would resolve
    // to the src/other.rs decoy (governing a file rustc never compiles = FP, and missing the real
    // src/inline/other.rs = FN). Pins the accumulated path_base.
    let file = resolve_file(
        "path-inline-targeted",
        &[
            (
                "lib.rs",
                "pub mod inline { #[path = \"other.rs\"] pub mod inner; }\n",
            ),
            ("inline/other.rs", "pub struct Real;\n"),
            ("other.rs", "pub struct Decoy;\n"),
        ],
        "crate::inline::inner",
    )
    .expect("a #[path] nested in an inline block resolves to its accumulated target");
    let file = file.display().to_string();
    assert!(
        file.replace('\\', "/").ends_with("inline/other.rs"),
        "the resolver accumulates the inline name: src/inline/other.rs, not the src/other.rs decoy: \
         {file}"
    );
}

#[test]
fn path_remapped_semantic_module_is_governed_at_its_target_not_the_orphan() {
    // `crate::domain` is `#[path = "weird.rs"]`; the boundary is now evaluated against weird.rs
    // (the compiled file), whose `real() -> crate::infra::Db` violates `must_not_expose`. The
    // same-named conventional orphan `domain.rs` — which rustc does not compile — is never
    // governed, so its `orphan()` exposure is neither the source of a violation nor masks the real
    // one. Previously this was a constitution error (the module skipped) — a false negative.
    let (metadata, _fixture) = fixture_metadata(
        "semantic-path-remap",
        &[
            (
                "lib.rs",
                "#[path = \"weird.rs\"]\npub mod domain;\npub mod infra;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "weird.rs",
                "pub fn real() -> crate::infra::Db { unimplemented!() }\n",
            ),
            (
                "domain.rs",
                "pub fn orphan() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SemanticBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("an unconditional #[path] module is governed at its target file");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations)
        .expect("the #[path] target resolves and is governed");
    let file = violations
        .first()
        .and_then(|v| v.file.as_deref())
        .map(str::to_string);

    assert_eq!(violations.len(), 1, "weird.rs's exposure of infra reacts");
    let file = file.expect("a governed-module file");
    assert!(
        file.ends_with("weird.rs"),
        "the reaction is in the #[path] target weird.rs, never the conventional orphan domain.rs: {file}"
    );
}

// --- resolver-rustc-fidelity: name-resolution divergences closed ----------

#[test]
fn fn1_bare_local_alias_shadowing_a_dependency_resolves_and_reacts() {
    // rustc: a local `type serde = …` shadows the extern prelude, so `X` is `crate::infra::Db`.
    // The alias-collection ladder must resolve the bare local alias BEFORE the extern oracle
    // (matching the query ladder), in either source order, so the chain closes to the target.
    for domain in [
        "type serde = crate::infra::Db;\ntype X = serde;\npub fn f() -> X { unimplemented!() }\n",
        "type X = serde;\ntype serde = crate::infra::Db;\npub fn f() -> X { unimplemented!() }\n",
    ] {
        let out = findings_with_deps(
            "fn1-alias-shadow",
            &[
                ("lib.rs", "pub mod infra;\npub mod domain;\n"),
                ("infra.rs", "pub struct Db;\n"),
                ("domain.rs", domain),
            ],
            "crate::domain",
            &["crate::infra"],
            &["serde"],
        )
        .unwrap();
        assert_eq!(
            out,
            ["crate::infra::Db exposed by fn crate::domain::f"],
            "source order: {domain}"
        );
    }
}

#[test]
fn fn2_leading_colon_is_an_unambiguous_extern_through_a_local_shadow() {
    // rustc: `::serde::Value` is the extern crate regardless of a local `mod serde`.
    let out = findings_with_deps(
        "fn2-leading-colon-mod",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod serde { pub struct Value; }\npub fn f() -> ::serde::Value { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(out, ["serde::Value exposed by fn crate::domain::f"]);
}

#[test]
fn fn2_leading_colon_bypasses_the_use_map_no_misattribution() {
    // `use crate::vendor::serde;` maps `serde`, but `::serde` bypasses the use-map: it reacts
    // as the extern `serde`, and NOT as `crate::vendor` (the false positive is gone).
    let files = &[
        ("lib.rs", "pub mod domain;\n"),
        (
            "domain.rs",
            "use crate::vendor::serde;\npub fn f() -> ::serde::Value { unimplemented!() }\n",
        ),
    ];
    let reacts = findings_with_deps(
        "fn2-usemap-extern",
        files,
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(reacts, ["serde::Value exposed by fn crate::domain::f"]);
    let no_fp = findings_with_deps(
        "fn2-usemap-nofp",
        files,
        "crate::domain",
        &["crate::vendor"],
        &["serde"],
    )
    .unwrap();
    assert!(
        no_fp.is_empty(),
        "leading-:: must not be misattributed to crate::vendor: {no_fp:?}"
    );
}

#[test]
fn fn2_leading_colon_alias_target_records_the_extern() {
    // The collection site honours leading-:: too: `type X = ::serde::Value;` records the extern
    // even under a local `mod serde`, so exposing `X` reacts.
    let out = findings_with_deps(
        "fn2-leading-colon-alias",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod serde { pub struct Value; }\ntype X = ::serde::Value;\npub fn f() -> X { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(out, ["serde::Value exposed by fn crate::domain::f"]);
}

#[test]
fn fp1_local_type_named_like_a_dependency_is_not_a_false_positive() {
    // rustc: a local `struct serde` shadows the dep in the type namespace, so `-> serde` is the
    // struct — the extern oracle must not fire. (A genuine extern exposure without the shadow, in
    // a separate module, still reacts — the regression half.)
    let clean = findings_with_deps(
        "fp1-local-struct",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct serde;\npub fn f() -> serde { serde }\n",
            ),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert!(
        clean.is_empty(),
        "a local `struct serde` shadows the dep; got {clean:?}"
    );
    let reacts = findings_with_deps(
        "fp1-real-extern",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use serde::Value;\npub fn g() -> Value { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(reacts, ["serde::Value exposed by fn crate::domain::g"]);
}

#[test]
fn fn4_enum_variant_fields_get_per_member_seams() {
    // Two forbidden fields of one variant stay distinct findings (per-member seam), so baselining
    // one never masks the other — the injectivity struct fields already had.
    let out = findings_with_deps(
        "fn4-variant-seam",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub enum E { V(crate::infra::Pool, crate::infra::Pool) }\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
        &[],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::Pool exposed by variant crate::domain::E::V::0",
            "crate::infra::Pool exposed by variant crate::domain::E::V::1",
        ]
    );
}

#[test]
fn fn2_leading_colon_through_a_crate_root_rename_reacts() {
    // Regression guard (apply-stage review): a leading-`::` path whose head is a crate-root
    // `extern crate … as` rename must still resolve through the rename — the base version reacted
    // to `::wc::spi::Foo`, and FN2's short-circuit must not drop it. Both the exposure position
    // and the alias-target collection site.
    let via_return = findings_with_deps(
        "fn2-leadingcolon-rename-return",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub fn make() -> ::wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        via_return,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
    let via_alias = findings_with_deps(
        "fn2-leadingcolon-rename-alias",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "type X = ::wc::spi::Foo;\npub fn make() -> X { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        via_alias,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

// --- operand-extern-oracle: inline extern trait operands react ------------

#[test]
fn dyn_operand_inline_sysroot_trait_reacts() {
    // The FN: an inline fully-qualified sysroot trait operand (no `use`) now resolves through the
    // extern oracle and reacts, exactly as the use-aliased spelling already did.
    let inline = dyn_operand_findings(
        "op-inline-std",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub fn f() -> Box<dyn std::error::Error> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["std::error::Error"],
        &[],
    )
    .unwrap();
    assert_eq!(inline, ["dyn std::error::Error exposed by fn crate::m::f"]);
    // The use-aliased spelling still reacts (parity, not regressed).
    let aliased = dyn_operand_findings(
        "op-aliased-std",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "use std::error::Error;\npub fn f() -> Box<dyn Error> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["std::error::Error"],
        &[],
    )
    .unwrap();
    assert_eq!(aliased, ["dyn Error exposed by fn crate::m::f"]);
    // An unlisted operand still passes.
    let unlisted = dyn_operand_findings(
        "op-unlisted-std",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub fn f() -> Box<dyn std::error::Error> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["crate::ports::Port"],
        &[],
    )
    .unwrap();
    assert!(
        unlisted.is_empty(),
        "unlisted operand must pass: {unlisted:?}"
    );
}

#[test]
fn dyn_operand_inline_dependency_and_crate_root_rename_react() {
    // An inline fully-qualified dependency trait operand reacts (extern oracle over declared deps).
    let inline_dep = dyn_operand_findings(
        "op-inline-dep",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub fn f() -> Box<dyn dep::spi::Port> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["dep::spi::Port"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(inline_dep, ["dyn dep::spi::Port exposed by fn crate::m::f"]);
    // A crate-root `extern crate dep as d;` rename head resolves to the real crate.
    let renamed = dyn_operand_findings(
        "op-rename-dep",
        &[
            ("lib.rs", "extern crate dep as d;\npub mod m;\n"),
            ("m.rs", "pub fn f() -> Box<dyn d::spi::Port> { todo!() }\n"),
        ],
        "crate::m",
        &["dep::spi::Port"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(renamed, ["dyn d::spi::Port exposed by fn crate::m::f"]);
}

#[test]
fn dyn_operand_crate_relative_extern_rename_reacts() {
    // The crate-relative spelling `crate::d::T` of a crate-root `extern crate dep as d;`
    // rename is rewritten (apply_crate_root_rename) exactly as the exposure resolver does, so it
    // reacts alike the bare `d::T` head — the specs' "same resolver ladder … with a crate-root
    // rename applied". Before, the operand resolver skipped this rewrite and this leak was silent.
    let out = dyn_operand_findings(
        "op-crate-rel-rename",
        &[
            ("lib.rs", "extern crate dep as d;\npub mod m;\n"),
            (
                "m.rs",
                "pub fn f() -> Box<dyn crate::d::Port> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["dep::Port"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dyn crate::d::Port exposed by fn crate::m::f"]);
}

#[test]
fn dyn_operand_child_shadowed_rename_head_does_not_react() {
    // The governed module declares its own child `mod d`, which shadows the crate-root
    // `extern crate dep as d;` alias within it (rustc resolves bare `d::Port` to the local module,
    // not the dep). The operand resolver's bare-head rewrite uses the child-shadowed rename map
    // (renames_bare), so it no longer rewrites `d` to `dep` and does not react. Before, it used the
    // full rename map and over-reacted on the local trait.
    let out = dyn_operand_findings(
        "op-child-shadow-rename",
        &[
            ("lib.rs", "extern crate dep as d;\npub mod m;\n"),
            (
                "m.rs",
                "pub mod d { pub trait Port {} }\npub fn f() -> Box<dyn d::Port> { todo!() }\n",
            ),
        ],
        "crate::m",
        &["dep::Port"],
        &["dep"],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "a child-shadowed bare rename head must not react: {out:?}"
    );
}

#[test]
fn impl_trait_operand_crate_relative_extern_rename_reacts() {
    // The crate-root-rename fix lives in the shared `resolve_principal`, so the impl-trait operand
    // path gets it too: `impl crate::d::Port` under `extern crate dep as d;` reacts alike the bare
    // head, closing the same FN on the existential-exposure rule.
    let out = impl_trait_operand_findings(
        "op-impl-crate-rel-rename",
        &[
            ("lib.rs", "extern crate dep as d;\npub mod m;\n"),
            ("m.rs", "pub fn f() -> impl crate::d::Port { todo!() }\n"),
        ],
        "crate::m",
        &["dep::Port"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["impl crate::d::Port exposed by fn crate::m::f"]);
}

#[test]
fn dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound() {
    // A bare single-segment principal that is neither in scope nor a declared/sysroot crate stays
    // dropped (the stated resolver bound) — the oracle does not over-reach (crate != trait anyway).
    let out = dyn_operand_findings(
        "op-unresolvable-bare",
        &[
            ("lib.rs", "pub mod m;\n"),
            ("m.rs", "pub fn f() -> Box<dyn Frobnicate> { todo!() }\n"),
        ],
        "crate::m",
        &["Frobnicate"],
        &[],
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "unresolvable bare principal must stay a bound: {out:?}"
    );
}

#[test]
fn impl_trait_operand_inline_sysroot_trait_reacts() {
    // Symmetric with dyn: a returned inline fully-qualified sysroot trait operand reacts.
    let inline = impl_trait_operand_findings(
        "iop-inline-std",
        &[
            ("lib.rs", "pub mod m;\n"),
            ("m.rs", "pub fn f() -> impl std::error::Error { todo!() }\n"),
        ],
        "crate::m",
        &["std::error::Error"],
        &[],
    )
    .unwrap();
    assert_eq!(inline, ["impl std::error::Error exposed by fn crate::m::f"]);
    // Unlisted still passes.
    let unlisted = impl_trait_operand_findings(
        "iop-unlisted-std",
        &[
            ("lib.rs", "pub mod m;\n"),
            ("m.rs", "pub fn f() -> impl std::error::Error { todo!() }\n"),
        ],
        "crate::m",
        &["crate::ports::Port"],
        &[],
    )
    .unwrap();
    assert!(
        unlisted.is_empty(),
        "unlisted impl-trait operand must pass: {unlisted:?}"
    );
}

// --- re-export head shadowed by a same-named child module (FP closure) -----

#[test]
fn reexport_head_shadowed_by_a_child_module_does_not_react() {
    // `pub use dep::spi::Foo;` in a module that also declares a child `mod dep`
    // resolves (per rustc) to the local module, not the dependency, so it must NOT react under a
    // boundary forbidding the dependency. The child `mod dep` is subtracted from the re-export set.
    let out = findings_with_deps(
        "reexport-child-shadow",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod dep { pub mod spi { pub struct Foo; } }\npub use dep::spi::Foo;\n",
            ),
        ],
        "crate::domain",
        &["dep::spi"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(
        out,
        Vec::<String>::new(),
        "the child-module shadow closes the FP: {out:?}"
    );
}

#[test]
fn reexport_head_with_crate_root_module_in_a_child_still_reacts() {
    // No FN: a crate-root `mod dep` does NOT shadow a bare `pub use dep::Foo;` in a CHILD module
    // (there `dep` reaches only the extern prelude). The child declares no `mod dep`, so `dep`
    // stays in its re-export extern set and the re-export still reacts.
    let out = findings_with_deps(
        "reexport-crateroot-mod",
        &[
            (
                "lib.rs",
                "pub mod dep { pub struct Foo; }\npub mod domain;\n",
            ),
            ("domain.rs", "pub use dep::Foo;\n"),
        ],
        "crate::domain",
        &["dep"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dep::Foo exposed by pub use crate::domain::Foo"]);
}

#[test]
fn reexport_head_is_not_suppressed_by_a_same_named_local_struct() {
    // Discriminating guard: only child MODULES are subtracted, not the full type namespace. A local
    // `struct dep;` (not a module) must NOT suppress the re-export — it still resolves to the
    // dependency. (If this ever reused `local_type_namespace_names`, the struct would wrongly
    // suppress it and this would return empty — a false negative.)
    let out = findings_with_deps(
        "reexport-struct-not-module",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub struct dep;\npub use dep::spi::Foo;\n"),
        ],
        "crate::domain",
        &["dep::spi"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dep::spi::Foo exposed by pub use crate::domain::Foo"]);
}

#[test]
fn reexport_leading_colon_reacts_despite_a_child_module_shadow() {
    // Escape hatch: `pub use ::dep::spi::Foo;` bypasses the shadow (leading-`::` uses the raw
    // extern set) and reacts even with a same-module child `mod dep`.
    let out = findings_with_deps(
        "reexport-leading-colon",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub mod dep { pub mod spi { pub struct Foo; } }\npub use ::dep::spi::Foo;\n",
            ),
        ],
        "crate::domain",
        &["dep::spi"],
        &["dep"],
    )
    .unwrap();
    assert_eq!(out, ["dep::spi::Foo exposed by pub use crate::domain::Foo"]);
}

// --- crate-root extern rename: crate-relative FN + submodule-shadow FP ------

#[test]
fn crate_relative_spelling_of_a_crate_root_rename_reacts() {
    // `crate::wc::spi::Foo` (the crate-relative spelling of a crate-root
    // `extern crate worklane_core as wc;`) is rewritten to the real crate and reacts.
    let out = findings_with_deps(
        "crate-alias-crate-relative",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub fn make() -> crate::wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn crate_relative_rename_behind_a_type_alias_and_reexport_reacts() {
    // The crate-relative rewrite is applied AFTER the alias/re-export closure, so `crate::wc::…`
    // reached through a `type` alias or a `pub use` target reacts too (not only when written
    // directly in a signature).
    let out = findings_with_deps(
        "crate-alias-through-alias",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "type H = crate::wc::spi::Foo;\npub fn make() -> H { unimplemented!() }\npub use crate::wc::spi::Bar;\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        [
            "worklane_core::spi::Bar exposed by pub use crate::domain::Bar",
            "worklane_core::spi::Foo exposed by fn crate::domain::make",
        ]
    );
}

#[test]
fn bare_rename_head_shadowed_by_a_submodule_child_mod_does_not_react() {
    // The governed submodule declares its own child `mod wc`, which rustc lets shadow the
    // crate-root extern alias, so bare `wc::spi::Foo` is the local module — not the dependency.
    let out = findings_with_deps(
        "crate-alias-submodule-shadow",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub mod wc { pub mod spi { pub struct Foo; } }\npub fn make() -> wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        Vec::<String>::new(),
        "the child mod wc shadow closes the FP: {out:?}"
    );
}

#[test]
fn bare_rename_head_with_no_local_shadow_still_reacts() {
    // No FN: with no local `mod wc`, the crate-wide bare rewrite is preserved and reacts.
    let out = findings_with_deps(
        "crate-alias-no-shadow",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub fn make() -> wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["worklane_core::spi::Foo exposed by fn crate::domain::make"]
    );
}

#[test]
fn a_deeper_crate_relative_alias_segment_is_not_rewritten() {
    // Guard: only the segment immediately after `crate` is the crate-root rename alias. A deeper
    // `crate::m::wc::…` is a local submodule item and must NOT be rewritten to the dependency.
    let out = findings_with_deps(
        "crate-alias-deeper-segment",
        &[
            (
                "lib.rs",
                "extern crate worklane_core as wc;\npub mod domain;\n",
            ),
            (
                "domain.rs",
                "pub mod m { pub mod wc { pub mod spi { pub struct Foo; } } }\npub fn make() -> crate::m::wc::spi::Foo { unimplemented!() }\n",
            ),
        ],
        "crate::domain",
        &["worklane_core::spi"],
        &["worklane_core"],
    )
    .unwrap();
    assert_eq!(
        out,
        Vec::<String>::new(),
        "a deeper crate::m::wc is local, not the rename: {out:?}"
    );
}

// --- forbidden-marker: re-export / alias / rename canonicalization (0.1.6 polish) ----------
// This battery pins the self-type canonicalization (folded into `resolve_self_type`) and the
// use-map leaf resolution against re-drift: a self-type written through a `pub use` facade or a
// `type` alias lands on its definition, a locally renamed trait/derive reacts by its true leaf,
// and the foreign/alias-to-foreign cases stay clean (no false positive).

#[test]
fn impl_self_type_spelled_through_a_reexport_reacts() {
    // `crate::wire` re-exports `crate::domain::Order`; a hand impl written against the RE-EXPORT
    // spelling still acquires the marker on the real def (coherence sees through the facade).
    let out = marker_findings(
        "mark-reexport-selftype",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "pub use crate::domain::Order;\nimpl serde::Serialize for crate::wire::Order {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::wire::Order in crate::wire"]
    );
}

#[test]
fn impl_self_type_use_imported_from_a_reexport_reacts() {
    // The impl lives in a third module and `use`s the re-exported spelling — the common form.
    let out = marker_findings(
        "mark-reexport-use-selftype",
        &[
            (
                "lib.rs",
                "pub mod domain;\npub mod wire;\npub mod client;\n",
            ),
            ("domain.rs", "pub struct Order;\n"),
            ("wire.rs", "pub use crate::domain::Order;\n"),
            (
                "client.rs",
                "use crate::wire::Order;\nimpl serde::Serialize for Order {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::wire::Order in crate::client"]
    );
}

#[test]
fn impl_self_type_through_an_alias_to_a_local_struct_still_reacts() {
    // Regression guard for the map change: the self-type resolver must keep catching a `type` alias
    // to a bare local struct (`type Bar = Real`) — the `CurrentModule`-landing map the exposure
    // (`Ignore`-built) alias map deliberately does not carry.
    let out = marker_findings(
        "mark-alias-local-struct",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Real;\ntype Bar = Real;\nimpl serde::Serialize for Bar {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::domain::Bar in crate::domain"]
    );
}

#[test]
fn impl_self_type_interleaved_alias_then_reexport_reacts() {
    // `type Alias = crate::wire::Reexp` (an alias hop) where `crate::wire` re-exports the real def
    // (a re-export hop): the interleaved fixpoint follows both to the definition.
    let out = marker_findings(
        "mark-alias-then-reexport",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\npub mod mid;\n"),
            ("domain.rs", "pub struct Order;\n"),
            ("wire.rs", "pub use crate::domain::Order as Reexp;\n"),
            (
                "mid.rs",
                "type Alias = crate::wire::Reexp;\nimpl serde::Serialize for Alias {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["impl serde::Serialize for crate::mid::Alias in crate::mid"]
    );
}

#[test]
fn impl_self_type_alias_to_a_foreign_type_stays_clean() {
    // An alias to a foreign/prelude type lands off the governed subtree — no false positive.
    let out = marker_findings(
        "mark-alias-foreign",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "pub struct Real;\ntype Baz = String;\nimpl serde::Serialize for Baz {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}

#[test]
fn impl_of_a_locally_renamed_trait_reacts_by_true_leaf() {
    // `use serde::Serialize as Ser; impl Ser for …` — leaf-matching now resolves the written trait
    // through the site's `use`-map, so the rename no longer evades the boundary.
    let out = marker_findings(
        "mark-rename-impl",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "use serde::Serialize as Ser;\nimpl Ser for crate::domain::Order {}\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, ["impl Ser for crate::domain::Order in crate::wire"]);
}

#[test]
fn derive_of_a_locally_renamed_macro_reacts_by_true_leaf() {
    // `use serde::Serialize as Ser; #[derive(Ser)]` — the derive form resolves through the defining
    // module's `use`-map too, symmetric with the impl form.
    let out = marker_findings(
        "mark-rename-derive",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "use serde::Serialize as Ser;\n#[derive(Ser)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, ["derive Ser on crate::domain::Order"]);
}

#[test]
fn derive_renamed_to_a_nonforbidden_local_trait_stays_clean() {
    // The dual: `use crate::other::Bar as Serialize; #[derive(Serialize)]` resolves to the local
    // `Bar` (leaf `Bar`), not serde — the leaf-collision false positive is closed by resolution.
    let out = marker_findings(
        "mark-rename-collision",
        &[
            ("lib.rs", "pub mod domain;\npub mod other;\n"),
            ("other.rs", "pub struct Bar;\n"),
            (
                "domain.rs",
                "use crate::other::Bar as Serialize;\n#[derive(Serialize)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(out, Vec::<String>::new());
}
