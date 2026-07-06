use super::*;

use std::path::PathBuf;

use crate::errors::unknown_module_error;
use crate::module_resolve::resolve_module_file;

/// Write `files` (each `(relative path, contents)`) under a unique temp `src` dir, then
/// return the findings for `module` against `forbidden`. Exercises the whole evaluator
/// (module resolution → exposure → use-resolution → match) without spawning `cargo`.
fn findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let dir = std::env::temp_dir().join(format!("hunyi-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let root = src.join("lib.rs");
    let result = module_findings(&src, &root, module, &forbidden, "x", false, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    result
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
    let dir = std::env::temp_dir().join(format!("hunyi-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|s| s.to_string()).collect();
    let root = src.join("lib.rs");
    let result = module_findings(&src, &root, module, &forbidden, "x", false, &deps);
    let _ = std::fs::remove_dir_all(&dir);
    result
}

/// Like [`findings`] but with the `semantic-trait-impl-exposure` opt-in enabled, so a trait
/// `impl` block's impl-site-authored positions are also observed.
fn findings_including_trait_impls(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let dir = std::env::temp_dir().join(format!("hunyi-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let root = src.join("lib.rs");
    let result = module_findings(&src, &root, module, &forbidden, "x", true, &[]);
    let _ = std::fs::remove_dir_all(&dir);
    result
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
        Violation::new(
            BoundaryKind::Semantic,
            "crate::m".to_string(),
            SIGNATURE_RULE.to_string(),
            "crate::infra::Db exposed by fn crate::m::f".to_string(),
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
fn crate_root_file_resolves_a_proc_macro_target() {
    // A proc-macro crate's target kind is `["proc-macro"]` (never lib/bin); it must still resolve
    // its root file so the semantic dimension governs it, not raise a false missing-src exit-2.
    let package = serde_json::json!({
        "targets": [{ "kind": ["proc-macro"], "src_path": "/w/src/lib.rs" }]
    });
    assert_eq!(
        crate::metadata::crate_root_file(&package),
        Some(std::path::PathBuf::from("/w/src/lib.rs"))
    );
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
    // FP closed (extern-set variant): `crate::a` re-exports `dep::spi::Foo` but declares a child
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
    // FP closed (rename-alias variant): a crate-root `extern crate worklane_core as wc;`, but
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
    // FN closed: a forbidden type imported by a PRIVATE `use wc::spi::Foo;` (wc = a crate-root
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
    let dir = std::env::temp_dir().join(format!("hunyi-loc-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let allowed: Vec<String> = allowed.iter().map(|s| s.to_string()).collect();
    let root = src.join("lib.rs");
    let result = trait_impl_findings(&src, &root, trait_path, &allowed, "x");
    let _ = std::fs::remove_dir_all(&dir);
    // The pure-heart tests assert on findings only; drop the per-finding module here.
    result.map(|v| v.into_iter().map(|(finding, _module)| finding).collect())
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
        ["crate::commandeer (impl for crate::commandeer::Foo)"],
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
    assert_eq!(out, ["crate::command (impl for crate::command::Foo)"]);
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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
fn a_path_remapped_module_is_a_documented_bound() {
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
    assert!(
        out.is_empty(),
        "a #[path]-remapped module is out of scope, not silently matched: {out:?}"
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
            "crate::domain (impl for crate::domain::A)",
            "crate::domain (impl for crate::domain::B)"
        ]
    );
}

#[test]
fn const_generic_expr_self_types_stay_distinct_owners() {
    // Two inherent impls whose self types differ ONLY in a complex const-generic
    // *expression* argument (`Arr<{ 1 + 1 }>` vs `Arr<{ 2 + 2 }>`). The expression is
    // unrenderable, so the owner falls back to `{base}<_#{ordinal}>` keyed on the impl
    // block's position among the module's items — keeping the two blocks INJECTIVE.
    // Previously both collapsed to `fn <_>::a`, masking one leak behind the other.
    // Items in `domain`: 0 = `struct Arr`, 1 = first impl, 2 = second impl.
    let out = findings(
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
    .unwrap();
    assert_eq!(
        out,
        [
            "crate::infra::T exposed by fn <crate::domain::Arr<_#1>>::a",
            "crate::infra::T exposed by fn <crate::domain::Arr<_#2>>::a",
        ],
        "two const-generic-expr self types yield two distinct positional owners, not one",
    );
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
    assert_eq!(out, ["crate::domain (impl for crate::domain::Foo)"]);
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

// --- visibility boundary -------------------------------------------------

fn vis_findings(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<String>, String> {
    let dir = std::env::temp_dir().join(format!("hunyi-vis-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let result = visibility_findings(&src, &root, module, "x");
    let _ = std::fs::remove_dir_all(&dir);
    result
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
            "crate::domain (impl for crate::domain::W<u16>)",
            "crate::domain (impl for crate::domain::W<u8>)"
        ]
    );
}

// --- forbidden-marker ----------------------------------------------------

fn marker_findings(
    name: &str,
    files: &[(&str, &str)],
    subtree: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    let dir = std::env::temp_dir().join(format!("hunyi-mark-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let root = src.join("lib.rs");
    let result = forbidden_marker_findings(&src, &root, subtree, &forbidden, "x");
    let _ = std::fs::remove_dir_all(&dir);
    // The pure-heart tests assert on findings only; drop the per-finding module here.
    result.map(|v| v.into_iter().map(|(finding, _module)| finding).collect())
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
            "derive serde::Serialize on crate::domain::A",
            "derive serde::Serialize on crate::domain::B"
        ],
        "serde_derive path (leaf) and cfg_attr-wrapped derive both react: {out:?}"
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
        ["impl serde::Serialize for crate::domain::Order"],
        "a hand impl written outside the subtree, for a subtree type, reacts: {out:?}"
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
        ["derive Serialize on crate::domain::Order"],
        "leaf-match reacts (accepted false positive; path-qualify to document intent)"
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
    let dir = std::env::temp_dir().join(format!("hunyi-dyn-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let result = dyn_module_findings(&src, &root, module, "x");
    let _ = std::fs::remove_dir_all(&dir);
    result
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
    let dir = std::env::temp_dir().join(format!("hunyi-dynop-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let forbidden: Vec<String> = forbidden.iter().map(|f| f.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|d| d.to_string()).collect();
    let result = dyn_operand_module_findings(&src, &root, module, &forbidden, "x", &deps);
    let _ = std::fs::remove_dir_all(&dir);
    result
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
fn dyn_operand_ignores_auto_trait_markers() {
    // `dyn Port + Send`: principal is Port (first bound). Forbidding Port flags it; forbidding
    // only the Send marker flags nothing (Send is not the principal, and a bare Send does not
    // resolve).
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
    let dir = std::env::temp_dir().join(format!("hunyi-impl-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let result = impl_trait_module_findings(&src, &root, module, "x");
    let _ = std::fs::remove_dir_all(&dir);
    result
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
    let dir = std::env::temp_dir().join(format!("hunyi-implop-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let forbidden: Vec<String> = forbidden.iter().map(|f| f.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|d| d.to_string()).collect();
    let result = impl_trait_operand_module_findings(&src, &root, module, &forbidden, "x", &deps);
    let _ = std::fs::remove_dir_all(&dir);
    result
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

// --- async-exposure -------------------------------------------------------

fn async_findings(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<String>, String> {
    let dir = std::env::temp_dir().join(format!("hunyi-async-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let result = async_exposure_module_findings(&src, &root, module, "x");
    let _ = std::fs::remove_dir_all(&dir);
    result
}

fn async_mod(name: &str, body: &str) -> Result<Vec<String>, String> {
    async_findings(
        name,
        &[("lib.rs", "pub mod m;\n"), ("m.rs", body)],
        "crate::m",
    )
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
    // FN closed: the dyn collector's inherent-impl arm now observes public associated `const`/`type`
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
    let dir = std::env::temp_dir().join(format!("hunyi-file-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root = src.join("lib.rs");
    let result = resolve_module_file(&src, &root, module, "x");
    let _ = std::fs::remove_dir_all(&dir);
    result
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

/// Build fixtures under a temp `src` plus synthetic `cargo metadata --no-deps` for a single
/// crate `x` whose lib root is that `src/lib.rs`, so a private `check_*_boundary` can run
/// without spawning `cargo`. Returns `(metadata, tempdir)`; the caller removes `tempdir`
/// **after** the check (the check reads the fixtures from disk).
fn fixture_metadata(name: &str, files: &[(&str, &str)]) -> (Value, PathBuf) {
    let dir = std::env::temp_dir().join(format!("hunyi-meta-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let src = dir.join("src");
    for (rel, contents) in files {
        let path = src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
    }
    let root_str = src.join("lib.rs").to_string_lossy().into_owned();
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "dependencies": [],
            "targets": [{ "kind": ["lib"], "src_path": root_str }],
        }],
    });
    (metadata, dir)
}

#[test]
fn semantic_violation_carries_the_governed_module_file_not_the_types_file() {
    // The forbidden type `crate::infra::Db` is *defined* in infra.rs; the exposing seam is in
    // domain.rs. The reported `file` is the seam's module (domain.rs), the actionable one.
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    assert_eq!(violations.len(), 1, "one exposure violation");
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
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    let v = &violations[0];
    assert!(v.file.is_some(), "the violation now carries a file");
    // `file` is metadata, not identity: a violation baselined while `file` was null still
    // matches once populated, so populating it never re-baselines or changes the count.
    assert_eq!(v.id(), v.clone().with_file(None).id());
}

#[test]
fn a_visibility_violation_carries_its_module_file() {
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!violations.is_empty(), "a pub item in internal violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("internal.rs"), "got {file}");
}

#[test]
fn a_trait_impl_locality_violation_carries_its_impl_site_file() {
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    let file = violations[0].file.as_deref().expect("the impl site's file");
    assert!(file.ends_with("plugins/mod.rs"), "got {file}");
}

#[test]
fn forbidden_marker_impl_and_derive_each_name_their_own_module_file() {
    // A forbidden `impl` sits in internal.rs; a forbidden `#[derive]` sits on a type in
    // models.rs. Each finding must name its OWN module's file — the derive names the
    // defining type's file (models.rs), never the impl site's (internal.rs).
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!violations.is_empty(), "the exposed dyn violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
fn an_impl_trait_violation_carries_its_module_file() {
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
    assert!(!violations.is_empty(), "the returned impl Trait violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
fn an_async_exposure_violation_carries_its_module_file() {
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
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
    let (metadata, dir) = fixture_metadata(
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
    let _ = std::fs::remove_dir_all(&dir);
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
fn path_remapped_module_file_is_not_resolved_via_a_conventional_orphan() {
    let err = resolve_file(
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
    .expect_err("a #[path]-remapped module is outside single-module resolution");
    assert_eq!(
        err,
        unknown_module_error("crate::domain", "x"),
        "the resolver must not govern the same-named conventional orphan"
    );
}

#[test]
fn path_remapped_semantic_module_is_not_governed_via_a_conventional_orphan() {
    let (metadata, dir) = fixture_metadata(
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
        .because("path-remapped modules are outside single-module semantic resolution");
    let mut violations = Vec::new();
    let err = check_boundary(&metadata, &boundary, &mut violations)
        .expect_err("a #[path]-remapped governed module is a constitution error");
    let _ = std::fs::remove_dir_all(&dir);

    assert_eq!(err, unknown_module_error("crate::domain", "x"));
    assert!(
        violations.is_empty(),
        "the same-named conventional orphan is not compiled and must not produce a violation"
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
    // FN closed: the crate-relative spelling `crate::d::T` of a crate-root `extern crate dep as d;`
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
    // FP closed: the governed module declares its own child `mod d`, which shadows the crate-root
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
    // FP closed: `pub use dep::spi::Foo;` in a module that also declares a child `mod dep`
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
    // FN closed: `crate::wc::spi::Foo` (the crate-relative spelling of a crate-root
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
    // FP closed: the governed submodule declares its own child `mod wc`, which rustc lets shadow the
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
