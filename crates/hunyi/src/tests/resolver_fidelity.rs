use super::super::*;
use super::dyn_trait::dyn_operand_findings;
use super::helpers::*;
use super::impl_trait::impl_trait_operand_findings;
// --- resolver-rustc-fidelity: name-resolution divergences closed ----------

#[test]
pub(super) fn fn1_bare_local_alias_shadowing_a_dependency_resolves_and_reacts() {
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
pub(super) fn fn2_leading_colon_is_an_unambiguous_extern_through_a_local_shadow() {
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

/// A forbidden operand shaped with an empty `::`-segment (leading, trailing, or doubled `::`)
/// must be a constitution error — never a silent, permanent non-reaction. `extern_verbatim_renamed`
/// never produces a leading-`::` canonical path (it iterates `syn::Path` segments and never
/// consults `leading_colon`), so an operand spelled `"::serde"` could never equal or
/// prefix-contain the resolved `"serde::Value"` — the exact silent-pass class the adversarial
/// sweep's finding described, reproduced here directly against `must_not_expose`'s pure heart.
#[test]
pub(super) fn must_not_expose_rejects_a_malformed_colon_operand() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod api;\n"),
        (
            "api.rs",
            "pub fn ext() -> ::serde::Value { unimplemented!() }\n",
        ),
    ];
    for bad in ["::serde", "serde::", "::serde::"] {
        let err = findings_with_deps("fn2-malformed", files, "crate::api", &[bad], &["serde"])
            .unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed operand {bad:?}: {err}"
        );
    }
    // The empty string is also a malformed operand (`has_empty_path_segment` treats
    // `"".split("::")` as one empty segment) — the shared validator's own doc names this
    // case, but no call site had ever exercised the literal empty string until now.
    let empty_err = findings_with_deps(
        "fn2-malformed-empty",
        files,
        "crate::api",
        &[""],
        &["serde"],
    )
    .unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty operand: {empty_err}"
    );
    // Control: the bare spelling this operand should have been written as still reacts, so the
    // rejection above is a spelling gate, never a general serde-detection regression.
    let clean = findings_with_deps(
        "fn2-malformed-control",
        files,
        "crate::api",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(clean, ["serde::Value exposed by fn crate::api::ext"]);
}

#[test]
pub(super) fn fn2_leading_colon_bypasses_the_use_map_no_misattribution() {
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
pub(super) fn fn2_leading_colon_alias_target_records_the_extern() {
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
pub(super) fn fp1_local_type_named_like_a_dependency_is_not_a_false_positive() {
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
pub(super) fn fn4_enum_variant_fields_get_per_member_seams() {
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
pub(super) fn fn2_leading_colon_through_a_crate_root_rename_reacts() {
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
pub(super) fn dyn_operand_inline_sysroot_trait_reacts() {
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

/// `dyn_operand_module_findings` shares `exposure::module_findings`'s resolver
/// (`resolve_principal` → `extern_verbatim_renamed`), so it has the identical malformed-operand
/// silent-pass gap: a forbidden operand with an empty `::`-segment must be a constitution error.
#[test]
pub(super) fn must_not_expose_dyn_of_rejects_a_malformed_colon_operand() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod m;\n"),
        (
            "m.rs",
            "pub fn f() -> Box<dyn std::error::Error> { todo!() }\n",
        ),
    ];
    for bad in [
        "::std::error::Error",
        "std::error::Error::",
        "::std::error::Error::",
    ] {
        let err =
            dyn_operand_findings("dyn-malformed", files, "crate::m", &[bad], &[]).unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed operand {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed operand — see must_not_expose's identical note.
    let empty_err =
        dyn_operand_findings("dyn-malformed-empty", files, "crate::m", &[""], &[]).unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty operand: {empty_err}"
    );
}

#[test]
pub(super) fn dyn_operand_inline_dependency_and_crate_root_rename_react() {
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
pub(super) fn dyn_operand_crate_relative_extern_rename_reacts() {
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
pub(super) fn dyn_operand_child_shadowed_rename_head_does_not_react() {
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
pub(super) fn impl_trait_operand_crate_relative_extern_rename_reacts() {
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
pub(super) fn dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound() {
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
pub(super) fn impl_trait_operand_inline_sysroot_trait_reacts() {
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

/// `impl_trait_operand_module_findings` shares the identical resolver as `dyn_operand_...` and
/// `exposure::module_findings` (`resolve_principal` → `extern_verbatim_renamed`), so it has the
/// same malformed-operand silent-pass gap for its module-scoped path.
#[test]
pub(super) fn must_not_expose_impl_trait_of_rejects_a_malformed_colon_operand() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod m;\n"),
        ("m.rs", "pub fn f() -> impl std::error::Error { todo!() }\n"),
    ];
    for bad in [
        "::std::error::Error",
        "std::error::Error::",
        "::std::error::Error::",
    ] {
        let err = impl_trait_operand_findings("iop-malformed", files, "crate::m", &[bad], &[])
            .unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed operand {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed operand — see must_not_expose's identical note.
    let empty_err =
        impl_trait_operand_findings("iop-malformed-empty", files, "crate::m", &[""], &[])
            .unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty operand: {empty_err}"
    );
}

/// The subtree-scoped operand path (`including_submodules()`) canonicalizes its own copy of the
/// forbidden set independently of the module-scoped path above, so it needs its own regression
/// coverage rather than relying on the module-scoped test to stand in for it.
#[test]
pub(super) fn must_not_expose_impl_trait_of_subtree_rejects_a_malformed_colon_operand() {
    let tree = TempSrcTree::new("iop-subtree-malformed");
    tree.write_all(&[
        ("lib.rs", "pub mod m;\n"),
        ("m.rs", "pub fn f() -> impl std::error::Error { todo!() }\n"),
    ]);
    for bad in [
        "::std::error::Error",
        "std::error::Error::",
        "::std::error::Error::",
    ] {
        let forbidden = vec![bad.to_string()];
        let err = impl_trait_operand_subtree_findings(
            tree.src(),
            &tree.root(),
            "crate",
            &forbidden,
            "x",
            &[],
        )
        .unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed operand {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed operand — see must_not_expose's identical note.
    let empty_err = impl_trait_operand_subtree_findings(
        tree.src(),
        &tree.root(),
        "crate",
        &[String::new()],
        "x",
        &[],
    )
    .unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty operand: {empty_err}"
    );
}

// --- re-export head shadowed by a same-named child module (FP closure) -----

#[test]
pub(super) fn reexport_head_shadowed_by_a_child_module_does_not_react() {
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
pub(super) fn reexport_head_with_crate_root_module_in_a_child_still_reacts() {
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
pub(super) fn reexport_head_is_not_suppressed_by_a_same_named_local_struct() {
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
pub(super) fn reexport_leading_colon_reacts_despite_a_child_module_shadow() {
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
pub(super) fn crate_relative_spelling_of_a_crate_root_rename_reacts() {
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
pub(super) fn crate_relative_rename_behind_a_type_alias_and_reexport_reacts() {
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
pub(super) fn bare_rename_head_shadowed_by_a_submodule_child_mod_does_not_react() {
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
pub(super) fn bare_rename_head_with_no_local_shadow_still_reacts() {
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
pub(super) fn a_deeper_crate_relative_alias_segment_is_not_rewritten() {
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
