use super::super::*;
use super::helpers::*;
use super::trait_impl::locality_findings;
// --- visibility boundary -------------------------------------------------

pub(super) fn vis_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<Vec<String>, String> {
    // The Crate ceiling (rank 2) — the `must_not_declare_pub` case existing tests assert.
    vis_findings_at(name, files, module, VisibilityCeiling::Crate.rank())
}

pub(super) fn vis_findings_at(
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
pub(super) fn visibility_rank_is_false_negative_safe_for_every_form() {
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
pub(super) fn super_ceiling_reacts_on_pub_and_pub_crate_only() {
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
pub(super) fn module_ceiling_reacts_on_pub_super_but_not_private() {
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
pub(super) fn pub_in_crate_path_is_clean_under_crate_ceiling() {
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
pub(super) fn pub_in_super_super_reacts_under_super_ceiling() {
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
pub(super) fn max_visibility_and_the_sugar_carry_the_ceiling() {
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
pub(super) fn ceiling_rule_strings_are_distinct_across_the_semantic_family() {
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
pub(super) fn pub_items_react_and_non_pub_items_are_clean() {
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
pub(super) fn a_pub_use_and_glob_react() {
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
pub(super) fn a_pub_submodule_reacts() {
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
pub(super) fn a_bare_pub_item_in_a_non_pub_module_still_reacts() {
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
pub(super) fn a_pub_extern_crate_and_pub_trait_alias_react() {
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
pub(super) fn a_leading_colon_pub_use_is_rendered_and_distinct() {
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
pub(super) fn a_macro_export_macro_is_out_of_scope() {
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
pub(super) fn a_macro_invocation_pub_item_is_a_documented_bound() {
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
pub(super) fn a_cfg_gated_pub_item_is_observed_as_written() {
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
pub(super) fn an_unknown_visibility_module_is_a_constitution_error() {
    let err = vis_findings(
        "vis-unknown",
        &[("lib.rs", "pub mod internal;\n"), ("internal.rs", "\n")],
        "crate::ghost",
    )
    .unwrap_err();
    assert_eq!(err, unknown_module_error("crate::ghost", "x"));
}

#[test]
pub(super) fn an_inline_visibility_module_is_scanned() {
    let out = vis_findings(
        "vis-inline",
        &[("lib.rs", "pub mod internal { pub fn a() {} fn b() {} }\n")],
        "crate::internal",
    )
    .unwrap();
    assert_eq!(out, ["pub fn a"]);
}

#[test]
pub(super) fn the_visibility_builder_carries_severity() {
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
pub(super) fn a_generic_self_type_is_rendered_distinctly() {
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
pub(super) fn distinct_trait_instantiations_for_one_self_type_stay_distinct_findings() {
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
pub(super) fn array_length_differing_trait_instantiations_stay_distinct() {
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
pub(super) fn complex_length_arrays_of_different_element_types_stay_distinct() {
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

// --- visibility boundary: extern-block foreign items ---------------------

/// The motivating false negative, with more than one `pub` foreign item in the SAME block so the
/// per-source-item result must carry all of them, not just one — `item_observation` returns a
/// `Vec` (widened from the prior `Option`) precisely because an `extern` block is one `syn::Item`
/// holding an arbitrary number of independently-visible foreign items. Also covers `pub type`
/// (an extern type declaration), which — unlike `pub fn`/`pub static` — carries no exposable
/// signature and so was out of `semantic-signature-coupling`'s own extern-block fix, but a bare
/// `pub type` IS a bare-pub declaration this capability must react to. Three non-pub foreign
/// items sit in the same block as a same-block control: they must not react.
#[test]
pub(super) fn a_pub_fn_pub_static_and_pub_type_inside_an_extern_block_all_react() {
    let out = vis_findings(
        "extern-block-multi",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            (
                "ffi.rs",
                "unsafe extern \"C\" {\n    pub fn open(h: *mut u8) -> u8;\n    pub static K: u8;\n    pub type Opaque;\n    fn hidden() -> u8;\n    static S: u8;\n    type T;\n}\n",
            ),
        ],
        "crate::ffi",
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub fn open", "pub static K", "pub type Opaque"],
        "every pub foreign item in the block reacts, every non-pub one stays clean: {out:?}"
    );
}

/// The identical shape in the plain edition-2021 `extern "C" { … }` form (no `unsafe` prefix) —
/// `syn::Item::ForeignMod` parses both forms identically, so there is no edition-specific gap.
#[test]
pub(super) fn a_pub_fn_and_pub_static_inside_a_plain_extern_block_react() {
    let out = vis_findings(
        "extern-block-plain",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            (
                "ffi.rs",
                "extern \"C\" {\n    pub fn plain(h: *mut u8) -> u8;\n    pub static K2: u8;\n}\n",
            ),
        ],
        "crate::ffi",
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub fn plain", "pub static K2"],
        "the plain 2021-edition extern block form reacts identically: {out:?}"
    );
}

/// A block whose foreign items are ALL non-`pub` must stay clean — the default (inherited)
/// visibility inside an `extern` block is private to the enclosing module, exactly like an
/// ordinary item, not implicitly public because it names an FFI declaration.
#[test]
pub(super) fn an_extern_block_with_no_pub_foreign_item_is_clean() {
    let out = vis_findings(
        "extern-block-none-pub",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            (
                "ffi.rs",
                "unsafe extern \"C\" {\n    fn hidden() -> u8;\n    static S: u8;\n    type T;\n}\n",
            ),
        ],
        "crate::ffi",
    )
    .unwrap();
    assert!(
        out.is_empty(),
        "no bare-pub foreign item in the block: {out:?}"
    );
}

/// A restricted-visibility foreign item (`pub(crate)`) is ranked exactly like an ordinary item's
/// own restricted visibility — the ceiling comparison downstream of `item_observation_parts`
/// applies uniformly regardless of item source, so a Super ceiling reacts on it exactly as
/// `super_ceiling_reacts_on_pub_and_pub_crate_only` already pins for ordinary items.
#[test]
pub(super) fn a_restricted_visibility_foreign_item_ranks_like_an_ordinary_one() {
    let out = vis_findings_at(
        "extern-block-restricted",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            (
                "ffi.rs",
                "unsafe extern \"C\" {\n    pub(crate) fn helper() -> u8;\n    pub(super) fn narrower() -> u8;\n}\n",
            ),
        ],
        "crate::ffi",
        VisibilityCeiling::Super.rank(),
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub(crate) fn helper"],
        "Super ceiling reacts on pub(crate), not pub(super), inside an extern block too: {out:?}"
    );
}

/// A `pub(in narrow-path)` item over-reacts under a tighter ceiling — the stated over-reaction bound.
///
/// Kept for the CONTRACT rather than for a change: `semantic-visibility-boundary` declares this bound and
/// nothing pinned it. The nearest existing test uses a `Super` ceiling, a different cell, so citing it would
/// have been reasoning dressed as evidence. Measured before the assertion was written: under a `Module`
/// ceiling the conservative `Crate` rank of `pub(in crate::a)` reacts even where the item is effectively
/// private, which is the over-reaction the bound states rather than a silent pass.
#[test]
pub(super) fn a_pub_in_narrow_path_over_reacts_under_a_module_ceiling() {
    let out = vis_findings_at(
        "pub-in-module-ceiling",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "pub(in crate::a) fn helper() {}\n"),
        ],
        "crate::a",
        0,
    )
    .unwrap();
    assert_eq!(
        out,
        ["pub(in crate::a) fn helper"],
        "the conservative rank must react rather than pass silently: {out:?}"
    );
}
