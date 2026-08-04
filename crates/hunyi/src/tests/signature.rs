use super::super::*;
use super::helpers::*;
// --- extern-path exposure (the external-crate name set) -------------------

#[test]
pub(super) fn hyphenated_dependency_name_is_normalized() {
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
pub(super) fn duplicate_semantic_violations_collapse_keeping_the_more_severe() {
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
        .into_finding("app", "src/lib.rs");
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
pub(super) fn leaf_of_strips_a_raw_identifier() {
    // Declared marker leaf compares raw-canonicalized, symmetric with the observed `path_leaf`.
    assert_eq!(leaf_of("crate::a::r#Trait"), "Trait");
    assert_eq!(leaf_of("Plain"), "Plain");
}

#[test]
pub(super) fn bare_extern_reexport_reacts() {
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
pub(super) fn sysroot_reexport_reacts_without_a_declared_dependency() {
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
pub(super) fn hyphenated_dependency_reexport_reacts_under_the_underscore_spelling() {
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
pub(super) fn aliased_extern_reexport_is_keyed_by_its_alias() {
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
pub(super) fn grouped_extern_reexport_reacts_per_leaf() {
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
pub(super) fn single_segment_crate_root_reexport_reacts() {
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
pub(super) fn subtree_extern_reexport_reacts_despite_a_crate_root_module_of_the_same_name() {
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
pub(super) fn signature_child_module_shadowing_a_dependency_is_no_false_positive() {
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
pub(super) fn inline_extern_field_type_reacts() {
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
pub(super) fn inline_extern_signature_return_reacts() {
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
pub(super) fn signature_child_module_path_is_no_false_positive() {
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
pub(super) fn facade_chain_of_inline_reexports_to_an_extern_type_reacts() {
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
pub(super) fn facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound() {
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
pub(super) fn facade_reaching_a_child_shadowed_extern_head_does_not_react() {
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
pub(super) fn facade_reaching_a_child_shadowed_rename_alias_head_does_not_react() {
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
pub(super) fn leading_colon_facade_hop_reacts_through_the_closure_despite_a_child_module() {
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
pub(super) fn crate_root_mod_does_not_suppress_a_child_facade_reexport_through_the_closure() {
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
pub(super) fn private_alias_in_a_public_seam_reacts() {
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
pub(super) fn a_generic_param_shadowing_a_same_module_alias_is_not_a_finding() {
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
pub(super) fn a_def_site_generic_param_shadowing_a_use_alias_is_not_a_finding() {
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
pub(super) fn an_assoc_type_projection_off_a_shadowing_param_is_not_a_finding() {
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
pub(super) fn cross_module_alias_reached_via_use_reacts() {
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
pub(super) fn alias_through_a_reexport_chain_reacts() {
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
pub(super) fn a_type_reached_through_a_reexported_module_facade_reacts() {
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
pub(super) fn a_reexport_whose_key_prefixes_its_value_does_not_diverge() {
    // Termination guaranteed: a reexport map entry whose alias key is a strict
    // `::`-prefix of its own value — the shape a same-name nested re-export (`pub use self::x::x;`)
    // yields — made the longest-prefix rewrite re-fire on its own monotonically-growing output; the
    // exact-repeat `seen` guard never fires on a never-repeating sequence, so the tool hung / OOMed.
    // The hop cap now bounds the fixpoint regardless of map contents (this exercises the cap
    // directly, bypassing the build-time guard).
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};
    let mut map = ReexportMap::new();
    map.insert("crate::a".to_string(), vec!["crate::a::b".to_string()]);
    // Before the fix this never returned; the assertion is simply that it TERMINATES.
    let out = expand_canonical_paths("crate::a::foo", &AliasMap::new(), &map);
    assert!(
        !out.is_empty(),
        "canonicalization must terminate on a key⊂value reexport entry: {out:?}"
    );
}

#[test]
pub(super) fn resolve_self_type_does_not_diverge_on_a_reexport_whose_key_prefixes_its_value() {
    // The sibling of the reexports test above, at `resolve_self_type`'s own resolver: before it
    // was routed through the shared, hop-capped `expand_canonical_paths`, its hand-rolled outer
    // loop re-ran an already-capped inner reexport-only fixpoint every iteration, so a key⊂value
    // reexport entry made the outer `landing` grow by a bounded amount each iteration, never
    // exactly repeating — the outer exact-repeat `seen` guard alone could not catch that. The
    // assertion is simply that this terminates.
    use crate::containment::resolve_self_type;
    use crate::resolve::{AliasMap, ReexportMap, UseMap};
    use std::collections::HashSet;

    let self_ty: syn::Type = syn::parse_str("Foo").unwrap();
    let uses = UseMap::new();
    let aliases = AliasMap::new();
    let mut reexports = ReexportMap::new();
    reexports.insert(
        "crate::a::Foo".to_string(),
        vec!["crate::a::Foo::b".to_string()],
    );
    let landing = resolve_self_type(
        &self_ty,
        &uses,
        "crate::a",
        &aliases,
        &reexports,
        &HashSet::new(),
    );
    assert!(
        !landing.is_empty(),
        "canonicalization must terminate on a key⊂value reexport entry: {landing:?}"
    );
}

#[test]
pub(super) fn diamond_alias_graph_expansion_terminates_and_memoizes_intermediate_nodes() {
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};

    let mut aliases: AliasMap = std::collections::HashMap::new();
    let reexports = ReexportMap::new();

    // Multi-tier diamond graph:
    // A -> (B, C)
    // B -> D
    // C -> D
    // D -> Secret
    aliases.insert(
        "crate::A".to_string(),
        vec!["crate::B".to_string(), "crate::C".to_string()],
    );
    aliases.insert("crate::B".to_string(), vec!["crate::D".to_string()]);
    aliases.insert("crate::C".to_string(), vec!["crate::D".to_string()]);
    aliases.insert("crate::D".to_string(), vec!["crate::Secret".to_string()]);

    let res = expand_canonical_paths("crate::A", &aliases, &reexports);
    assert_eq!(res, vec!["crate::Secret".to_string()]);
}

#[test]
pub(super) fn cycle_branch_with_terminal_sibling_preserves_sibling() {
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};

    let mut aliases: AliasMap = std::collections::HashMap::new();
    let reexports = ReexportMap::new();

    // A -> [B, Secret]
    // B -> A
    aliases.insert(
        "crate::A".to_string(),
        vec!["crate::B".to_string(), "crate::Secret".to_string()],
    );
    aliases.insert("crate::B".to_string(), vec!["crate::A".to_string()]);

    let res = expand_canonical_paths("crate::A", &aliases, &reexports);
    assert!(
        res.contains(&"crate::Secret".to_string()),
        "sibling Secret target must be preserved even if child branch B cycles back to A: got {res:?}"
    );
}

#[test]
pub(super) fn deep_chain_expansion_reaches_terminal_without_truncation() {
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};

    // Build a 99-hop linear chain: N0 -> N1 -> … -> N99 (no alias on N99, so it is the fixpoint).
    // The iterative expansion must traverse the full chain without truncation or stack overflow.
    let mut aliases: AliasMap = std::collections::HashMap::new();
    let reexports = ReexportMap::new();
    for i in 0..99usize {
        aliases.insert(format!("crate::N{i}"), vec![format!("crate::N{}", i + 1)]);
    }

    let res = expand_canonical_paths("crate::N0", &aliases, &reexports);
    assert_eq!(
        res,
        vec!["crate::N99".to_string()],
        "full 99-hop chain must resolve to the terminal node N99, got {res:?}"
    );
}

#[test]
pub(super) fn self_growing_reexport_prefix_loop_terminates_without_hanging() {
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};

    let aliases: AliasMap = std::collections::HashMap::new();
    let mut reexports = ReexportMap::new();

    // A self-similar re-export entry `crate::a -> crate::a::b` creates a self-growing path chain:
    // crate::a::foo -> crate::a::b::foo -> crate::a::b::b::foo -> ...
    reexports.insert("crate::a".to_string(), vec!["crate::a::b".to_string()]);

    let res = expand_canonical_paths("crate::a::foo", &aliases, &reexports);
    assert!(
        !res.is_empty(),
        "expansion must terminate without hanging on self-growing prefix loops: got {res:?}"
    );
}

#[test]
pub(super) fn a_self_similar_reexport_is_dropped_and_the_real_type_still_reacts() {
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
pub(super) fn alias_of_an_alias_reacts() {
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
pub(super) fn alias_to_an_extern_path_reacts() {
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
pub(super) fn public_same_module_alias_still_reacts() {
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
pub(super) fn complex_target_alias_is_a_stated_bound() {
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
pub(super) fn generic_alias_is_a_stated_bound() {
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
pub(super) fn a_local_module_shadows_a_dependency_in_an_alias_target() {
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
pub(super) fn alias_to_a_nonforbidden_path_is_clean() {
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
pub(super) fn alias_hidden_and_direct_exposures_share_the_canonical_type() {
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
pub(super) fn a_single_segment_alias_named_like_a_dependency_resolves_to_the_local_alias() {
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
pub(super) fn alias_target_reached_via_use_reacts() {
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
pub(super) fn alias_in_a_trait_impl_position_reacts_under_the_opt_in() {
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
pub(super) fn extern_glob_forbidden_root_reacts() {
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
pub(super) fn extern_glob_nonforbidden_root_is_a_stated_bound() {
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
pub(super) fn foreign_prelude_rename_is_a_stated_bound() {
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
pub(super) fn source_level_crate_root_extern_crate_rename_reacts() {
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
pub(super) fn source_level_extern_crate_rename_in_a_type_position_reacts() {
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
pub(super) fn private_use_of_a_crate_root_extern_rename_reacts() {
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
pub(super) fn private_use_of_a_child_shadowed_rename_alias_does_not_react() {
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
pub(super) fn module_scoped_extern_crate_rename_is_a_stated_bound() {
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
pub(super) fn extern_crate_rename_to_a_nonforbidden_crate_is_clean() {
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
pub(super) fn pub_extern_crate_reacts_as_an_exposure() {
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
pub(super) fn pub_extern_crate_rename_names_the_real_crate() {
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
pub(super) fn private_extern_crate_is_not_an_exposure() {
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
pub(super) fn pub_extern_crate_outside_the_forbidden_set_is_clean() {
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
pub(super) fn a_bare_std_prelude_alias_target_is_not_mis_recorded() {
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
pub(super) fn a_bare_alias_to_a_complex_local_alias_stays_bounded() {
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
pub(super) fn bare_alias_of_an_alias_reacts() {
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
pub(super) fn bare_alias_of_an_alias_reacts_in_reverse_source_order() {
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
pub(super) fn alias_target_through_a_crate_root_extern_rename_reacts() {
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
pub(super) fn alias_target_through_extern_rename_reacts_when_alias_precedes_extern_crate() {
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
pub(super) fn renamed_head_is_not_suppressed_by_a_same_named_child_module_shadow() {
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
pub(super) fn facade_reexport_through_an_extern_rename_reacts() {
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
pub(super) fn a_bare_alias_to_a_nonforbidden_local_type_is_clean() {
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
pub(super) fn trait_impl_exposure_reacts_at_the_trait_arg_position() {
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
pub(super) fn trait_impl_exposure_reacts_at_the_self_position_bare() {
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
pub(super) fn trait_impl_exposure_reacts_at_the_self_position_nested() {
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
pub(super) fn trait_impl_exposure_reacts_at_the_assoc_position() {
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
pub(super) fn trait_impl_exposure_reacts_at_the_where_position() {
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
pub(super) fn trait_impl_exposure_reacts_at_an_associated_const_type() {
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
pub(super) fn trait_impl_exposure_reacts_at_a_where_clause_bounded_type() {
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
pub(super) fn trait_impl_exposure_reacts_at_a_const_generic_param_type() {
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
pub(super) fn trait_impl_exposure_unrenderable_where_bound_fails_loud_without_positional_identity()
{
    // Round-2 adversarial-review finding, reproduced and fixed: an unrenderable where-clause
    // bounded type (a complex const-generic argument the ordinary renderer cannot stringify) must
    // not fall back to the bare literal `_` — two SUCH bounds in one impl block would then share
    // that key, and their facts (identical kind, subject, and seam) would collapse to one,
    // silently losing the second bound's violation. `Arr<{ N + 1 }>` and `Arr<{ N + 2 }>` are
    // structurally distinct types that both fail to render the same way; both bounds independently
    // require `AsRef<crate::infra::Secret>`, so before the fix this collapsed to ONE finding
    // regardless of which or how many such bounds were present (verified: single-bound and
    // two-bound fixtures produced the byte-identical fact string). The fix must fail loud instead.
    let error = findings_including_trait_impls(
        "ti-where-unrenderable",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub struct Thing;\npub struct Arr<const N: usize>;\npub const N: usize = 1;\nimpl crate::Port for Thing where Arr<{ N + 1 }>: AsRef<crate::infra::Secret>, Arr<{ N + 2 }>: AsRef<crate::infra::Secret> {}\n",
            ),
        ],
        "crate::m",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(error.contains("stable structural label"), "{error}");
    // The sentinel that trips the gate is internal — never itself published as identity.
    assert!(!error.contains("_#"), "{error}");
}

#[test]
pub(super) fn trait_impl_exposure_unrenderable_where_bound_fails_loud_even_alone() {
    // The single-bound counterpart of the test above: even ONE unrenderable where-clause bound
    // (no sibling bound to collide with) must fail loud rather than silently publish the bare `_`
    // key — the fail-loud requirement does not depend on a second bound being present.
    let error = findings_including_trait_impls(
        "ti-where-unrenderable-solo",
        &[
            ("lib.rs", "pub mod m;\n"),
            (
                "m.rs",
                "pub struct Thing;\npub struct Arr<const N: usize>;\npub const N: usize = 1;\nimpl crate::Port for Thing where Arr<{ N + 1 }>: AsRef<crate::infra::Secret> {}\n",
            ),
        ],
        "crate::m",
        &["crate::infra"],
    )
    .unwrap_err();
    assert!(error.contains("stable structural label"), "{error}");
    assert!(!error.contains("_#"), "{error}");
}

#[test]
pub(super) fn trait_impl_exposure_where_bound_sentinels_never_share_a_bound_ordinal() {
    // White-box counterpart proving genuine collision-freedom, not merely that the shared
    // `reject_positional_identity` gate trips (the black-box tests above cannot distinguish a
    // truly per-bound sentinel from a reused bare-ordinal one, since either trips the SAME gate
    // with the SAME message). Calls `collect_trait_impl_exposures` directly — before the gate
    // ever runs — and asserts the two unrenderable bounds' `where`-position keys differ, each
    // carrying its own `bound_ordinal` composed with the shared item `ordinal`.
    use crate::collect::collect_trait_impl_exposures;
    use crate::finding::{PublicSeam, TraitImplPosition};
    use crate::resolve::UseMap;

    let item: syn::Item = syn::parse_str(
        "impl crate::Port for Thing where Arr<{ N + 1 }>: AsRef<crate::infra::Secret>, Arr<{ N + 2 }>: AsRef<crate::infra::Secret> {}",
    )
    .unwrap();
    let uses = UseMap::new();
    let mut out = Vec::new();
    collect_trait_impl_exposures(&item, "crate::m", &uses, 7, &mut out);

    let where_keys: std::collections::BTreeSet<&str> = out
        .iter()
        .filter_map(|exposure| match &exposure.seam {
            PublicSeam::TraitImpl {
                position: TraitImplPosition::Where(key),
                ..
            } => Some(key.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(
        where_keys.len(),
        2,
        "the two bounds must each key their exposures under their OWN distinct sentinel: {where_keys:?}"
    );
    for key in &where_keys {
        assert!(
            key.contains("_#"),
            "an unrenderable bound's key must carry the internal positional sentinel: {key}"
        );
    }
}

#[test]
pub(super) fn trait_impl_exposure_reacts_at_a_refined_rpitit_return() {
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
pub(super) fn a_trait_impl_generic_param_shadowing_an_alias_is_not_exposed() {
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
pub(super) fn trait_impl_method_parameter_is_not_observed_but_the_return_is() {
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
pub(super) fn implementing_a_forbidden_trait_is_a_non_goal() {
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
pub(super) fn a_bare_boundary_ignores_trait_impls() {
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
pub(super) fn two_where_bounds_exposing_the_same_type_stay_distinct() {
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
pub(super) fn two_positions_exposing_the_same_type_stay_distinct() {
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
pub(super) fn a_reexported_type_in_a_trait_impl_position_resolves_and_reacts() {
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
pub(super) fn a_bare_name_in_a_trait_impl_position_is_not_a_false_positive() {
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
pub(super) fn reexport_of_a_forbidden_type_reacts_by_default() {
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
pub(super) fn aliased_reexport_is_keyed_by_the_alias() {
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
pub(super) fn two_aliases_of_the_same_type_stay_distinct_findings() {
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
pub(super) fn grouped_reexport_reacts_per_leaf() {
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
pub(super) fn reexport_through_a_facade_chain_reacts() {
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
pub(super) fn reexport_through_a_self_group_facade_chain_reacts() {
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
pub(super) fn reexport_through_a_renamed_self_facade_chain_reacts_cleanly() {
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
pub(super) fn named_whole_module_reexport_reacts() {
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
pub(super) fn self_group_module_reexport_reacts_keyed_by_module_name() {
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
pub(super) fn reexport_with_raw_identifier_segment_reacts() {
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
pub(super) fn renamed_self_module_reexport_reacts_with_correct_type() {
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
pub(super) fn glob_reexport_with_forbidden_root_reacts() {
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
pub(super) fn glob_reexport_with_root_deeper_than_forbidden_prefix_reacts() {
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
pub(super) fn sibling_root_glob_does_not_react() {
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
pub(super) fn ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react() {
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
pub(super) fn restricted_and_private_and_underscore_reexports_do_not_react() {
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
pub(super) fn forbidden_type_in_a_public_return_is_a_finding() {
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
pub(super) fn a_type_used_only_internally_is_not_a_finding() {
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
pub(super) fn forbidden_type_in_a_public_field_is_a_finding() {
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
pub(super) fn a_private_field_does_not_expose() {
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
pub(super) fn inherent_impl_public_method_exposes() {
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
pub(super) fn trait_impl_is_out_of_scope() {
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
pub(super) fn a_renamed_import_resolves_and_reacts() {
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
pub(super) fn a_use_imported_type_resolves_via_its_head() {
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
pub(super) fn a_glob_import_is_a_documented_bound() {
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
pub(super) fn a_forbidden_trait_in_a_generic_bound_is_a_finding() {
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
pub(super) fn a_module_prefix_matches_beneath_but_not_a_sibling() {
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
pub(super) fn a_nested_generic_argument_is_observed() {
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
pub(super) fn an_unknown_module_is_a_constitution_error() {
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
pub(super) fn a_mod_rs_backed_module_resolves() {
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
pub(super) fn an_inline_module_resolves() {
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
pub(super) fn a_forbidden_type_via_a_pub_use_facade_resolves_and_reacts() {
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
pub(super) fn a_forbidden_type_via_a_super_relative_use_resolves_and_reacts() {
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
