use super::super::*;
use super::helpers::*;
use super::impl_trait::impl_trait_subtree;
use super::unsafe_confinement::unsafe_labels;
// --- forbidden-marker ----------------------------------------------------

pub(super) fn marker_findings(
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
pub(super) fn a_forbidden_derive_on_a_subtree_type_reacts_and_a_clean_type_does_not() {
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

/// Leaf-identifier matching (`leaf_of`) is immune to a *leading* `::` (the leaf is still the
/// last segment) but not to a *trailing* one — `leaf_of("serde::")` computes an empty leaf, which
/// no real identifier can ever equal. Both spellings are rejected as a constitution error: the
/// trailing case because it could never match anything, and the leading case for consistency with
/// every other forbidden-operand-shaped DSL method in this family (none of which assigns the
/// leading-`::` spelling a meaning distinct from the bare form).
#[test]
pub(super) fn must_not_acquire_rejects_a_malformed_colon_operand() {
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod domain;\n"),
        (
            "domain.rs",
            "#[derive(serde::Serialize)]\npub struct Order;\n",
        ),
    ];
    for bad in [
        "::serde::Serialize",
        "serde::Serialize::",
        "::serde::Serialize::",
    ] {
        let err = marker_findings("marker-malformed", files, "crate::domain", &[bad]).unwrap_err();
        assert!(
            err.contains(bad),
            "constitution error must name the malformed operand {bad:?}: {err}"
        );
    }
    // The empty string itself is also a malformed operand — see must_not_expose's identical note.
    let empty_err =
        marker_findings("marker-malformed-empty", files, "crate::domain", &[""]).unwrap_err();
    assert!(
        empty_err.contains("is empty"),
        "constitution error must flag the empty operand: {empty_err}"
    );
    // Control: the bare spelling still reacts, so the rejection above is a spelling gate, not a
    // general leaf-matching regression.
    let clean = marker_findings(
        "marker-malformed-control",
        files,
        "crate::domain",
        &["serde::Serialize"],
    )
    .unwrap();
    assert_eq!(clean, ["derive serde::Serialize on crate::domain::Order"]);
}

#[test]
pub(super) fn a_serde_derive_path_and_cfg_attr_derive_react_by_leaf() {
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
pub(super) fn a_hand_impl_outside_the_subtree_reacts_via_the_self_type() {
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

/// Two mutually-exclusive `#[cfg]`-gated `use ... as Name;` aliases for a `#[derive(Name)]`'s own
/// name must both react (cfg-blind): before the fix, `resolve_path`'s single-candidate lookup took
/// only one `use`-map entry before leaf-matching, so whether the derive's TRUE leaf was seen
/// depended on which cfg branch's alias happened to be declared first (found on adversarial review
/// of `hunyi-cfg-branch-use-reexport-merging`).
#[test]
pub(super) fn forbidden_derive_leaf_reacts_when_the_forbidden_alias_is_declared_first() {
    let out = marker_findings(
        "derive-cfg-forbidden-first",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[cfg(unix)]\nuse bad::Marker as M;\n#[cfg(not(unix))]\nuse good::NotBad as M;\n#[derive(M)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["bad::Marker"],
    )
    .unwrap();
    assert_eq!(out, ["derive M on crate::domain::Order"]);
}

/// The identical shape with the forbidden alias declared SECOND. Before the fix this silently
/// passed (`Ok([])`): `resolve_path` took only the first `use`-map candidate, and here that
/// candidate was the non-forbidden one.
#[test]
pub(super) fn forbidden_derive_leaf_reacts_when_the_forbidden_alias_is_declared_second() {
    let out = marker_findings(
        "derive-cfg-forbidden-second",
        &[
            ("lib.rs", "pub mod domain;\n"),
            (
                "domain.rs",
                "#[cfg(not(unix))]\nuse good::NotBad as M;\n#[cfg(unix)]\nuse bad::Marker as M;\n#[derive(M)]\npub struct Order;\n",
            ),
        ],
        "crate::domain",
        &["bad::Marker"],
    )
    .unwrap();
    assert_eq!(out, ["derive M on crate::domain::Order"]);
}

/// The identical collision at the impl form's trait-leaf match: `impl M for X` where `M` is a
/// mutually-exclusive `#[cfg]`-gated `use` alias.
#[test]
pub(super) fn forbidden_impl_trait_leaf_reacts_when_the_forbidden_alias_is_declared_first() {
    let out = marker_findings(
        "impl-cfg-forbidden-first",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "#[cfg(unix)]\nuse bad::Marker as M;\n#[cfg(not(unix))]\nuse good::NotBad as M;\nimpl M for crate::domain::Order {}\n",
            ),
        ],
        "crate::domain",
        &["bad::Marker"],
    )
    .unwrap();
    assert_eq!(out, ["impl M for crate::domain::Order in crate::wire"]);
}

#[test]
pub(super) fn forbidden_impl_trait_leaf_reacts_when_the_forbidden_alias_is_declared_second() {
    let out = marker_findings(
        "impl-cfg-forbidden-second",
        &[
            ("lib.rs", "pub mod domain;\npub mod wire;\n"),
            ("domain.rs", "pub struct Order;\n"),
            (
                "wire.rs",
                "#[cfg(not(unix))]\nuse good::NotBad as M;\n#[cfg(unix)]\nuse bad::Marker as M;\nimpl M for crate::domain::Order {}\n",
            ),
        ],
        "crate::domain",
        &["bad::Marker"],
    )
    .unwrap();
    assert_eq!(out, ["impl M for crate::domain::Order in crate::wire"]);
}

#[test]
pub(super) fn a_foreign_or_prelude_self_type_is_not_a_governed_subtree_type() {
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
pub(super) fn distinct_generic_marker_instantiations_stay_distinct_findings() {
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
pub(super) fn unrenderable_generic_marker_instantiations_fail_loud_without_positional_identity() {
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
pub(super) fn a_forbidden_marker_on_a_local_type_alias_reacts() {
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
pub(super) fn a_blanket_impls_own_generic_param_is_not_resolved_through_a_same_named_alias() {
    // Round-9 finding: resolve_self_type (containment.rs) resolved a bare self type exactly like
    // any other path reference, with no awareness that the identifier might be the impl's OWN
    // declared generic type parameter rather than a nominal type. `impl<T> Marker for T {}` (a
    // blanket impl — T is a parameter use, not a type) in a module that also happens to declare an
    // unrelated `use ... as T` alias resolved the self type through that alias, fabricating a
    // marker-acquisition finding on the aliased type even though the source never writes `impl
    // Marker for` it at all. The sibling exposure collectors already shadow an impl's own generic
    // params for every OTHER position (collect::exposure::type_param_names); the marker gate's self-type
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
pub(super) fn a_blanket_impls_generic_param_is_shadowed_even_through_a_multi_segment_projection() {
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
pub(super) fn a_qualified_path_self_type_off_the_impls_own_generic_param_is_not_resolved_through_an_alias()
 {
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
pub(super) fn a_forbidden_marker_on_an_alias_to_a_foreign_type_is_clean() {
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
pub(super) fn two_same_leaf_derives_on_one_type_stay_distinct() {
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
pub(super) fn a_submodule_type_is_governed_and_a_sibling_is_not() {
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
pub(super) fn a_same_leaf_different_trait_is_a_documented_false_positive() {
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
pub(super) fn an_unresolvable_glob_self_type_is_a_documented_bound() {
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
pub(super) fn a_nested_cfg_attr_derive_reacts() {
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
pub(super) fn a_malformed_derive_is_a_scan_error_not_a_silent_pass() {
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
pub(super) fn a_symlinked_module_cycle_is_a_scan_error_not_a_stack_overflow() {
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

/// Unlike the symlinked-cycle case above, this tree is genuinely acyclic — pure inline `mod`
/// nesting, no repeated canonical file — so `ancestors` alone never catches it (an inline child
/// never opens a new file and so never grows `ancestors`). Only a native recursion-depth counter
/// bounds it. Past `MAX_MODULE_DEPTH` levels the walk must fail loud (a scan error naming the
/// depth bound), never silently recurse into a native stack overflow. 60 levels of nesting is
/// comfortably past `MAX_MODULE_DEPTH` (32, so the check fires with margin to spare) and
/// comfortably short of both `syn::parse_file`'s own debug-build recursion limit AND this
/// walker's own measured 2MB-test-thread crash line (~80-90 levels, unrelated to the fix — see
/// `MAX_MODULE_DEPTH`'s own doc) — this exercises the depth check itself, not a crash in either
/// direction. Driven through `forbidden_marker_findings` (`scan_crate` / `walk_module`), matching
/// the symlink test's own vehicle.
#[test]
pub(super) fn a_deeply_nested_acyclic_module_tree_is_a_scan_error_not_a_stack_overflow() {
    let depth = 60;
    let source = format!(
        "{}pub struct Leaf;{}\n",
        "pub mod a{".repeat(depth),
        "}".repeat(depth)
    );
    let tree = TempSrcTree::new("deep-acyclic-walk-module");
    tree.write("lib.rs", &source);
    let result = forbidden_marker_findings(tree.src(), &tree.root(), "crate", &[], "x");
    let err = result.expect_err(
        "a deeply nested but acyclic module tree must be a scan error, not a hang/overflow",
    );
    assert!(
        err.contains("depth bound"),
        "the error must name the depth bound it could not judge past: {err}"
    );
}

/// Same property as above, for `walk_subtree_modules` (`collect_subtree`) — the subtree-scoped
/// walker `impl_trait_subtree` drives.
#[test]
pub(super) fn a_deeply_nested_acyclic_subtree_walk_is_a_scan_error_not_a_stack_overflow() {
    let depth = 60;
    let source = format!(
        "{}pub struct Leaf;{}\n",
        "pub mod a{".repeat(depth),
        "}".repeat(depth)
    );
    let files = &[("lib.rs", source.as_str())];
    let err = impl_trait_subtree("deep-acyclic-subtree", files, "crate")
        .expect_err("a deeply nested but acyclic subtree walk must be a scan error, not a hang");
    assert!(
        err.contains("depth bound"),
        "the error must name the depth bound it could not judge past: {err}"
    );
}

/// Same property as above, for `scan_unsafe_sites` (`walk_unsafe`).
#[test]
pub(super) fn a_deeply_nested_acyclic_unsafe_walk_is_a_scan_error_not_a_stack_overflow() {
    let depth = 60;
    let source = format!(
        "{}pub struct Leaf;{}\n",
        "pub mod a{".repeat(depth),
        "}".repeat(depth)
    );
    let err = unsafe_labels(
        "deep-acyclic-unsafe",
        &[("lib.rs", &source)],
        &["crate::ffi"],
    )
    .expect_err("a deeply nested but acyclic unsafe walk must be a scan error, not a hang");
    assert!(
        err.contains("depth bound"),
        "the error must name the depth bound it could not judge past: {err}"
    );
}

/// Control: nesting well under the depth bound (32) is unaffected — the fix must not narrow
/// ordinary observation. A forbidden marker 20 levels deep still reacts.
#[test]
pub(super) fn a_moderately_nested_module_tree_still_observes_a_real_violation() {
    let depth = 20;
    let source = format!(
        "{}#[derive(serde::Serialize)]\npub struct Order;{}\n",
        "pub mod a{".repeat(depth),
        "}".repeat(depth)
    );
    let out = marker_findings(
        "moderate-depth",
        &[("lib.rs", &source)],
        "crate",
        &["serde::Serialize"],
    )
    .expect("nesting well under the depth bound must still be judged, not error");
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].contains("Order"), "{out:?}");
}

#[test]
pub(super) fn two_same_named_types_in_different_submodules_stay_distinct() {
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
pub(super) fn a_cfg_dual_declared_module_backed_by_one_file_does_not_duplicate_its_marker_finding()
{
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
pub(super) fn the_forbidden_marker_builder_carries_severity() {
    let b = ForbiddenMarkerBoundary::in_crate("app")
        .module("crate::domain")
        .must_not_acquire("serde::Serialize")
        .and_not_acquire("serde::Deserialize")
        .warn()
        .because("r");
    assert_eq!(b.forbidden(), &["serde::Serialize", "serde::Deserialize"]);
    assert_eq!(b.severity(), Severity::Warn);
}

// --- forbidden-marker: re-export / alias / rename canonicalization (0.1.6 polish) ----------
// This battery pins the self-type canonicalization (folded into `resolve_self_type`) and the
// use-map leaf resolution against re-drift: a self-type written through a `pub use` facade or a
// `type` alias lands on its definition, a locally renamed trait/derive reacts by its true leaf,
// and the foreign/alias-to-foreign cases stay clean (no false positive).

#[test]
pub(super) fn impl_self_type_spelled_through_a_reexport_reacts() {
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
pub(super) fn impl_self_type_use_imported_from_a_reexport_reacts() {
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
pub(super) fn impl_self_type_through_an_alias_to_a_local_struct_still_reacts() {
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
pub(super) fn impl_self_type_interleaved_alias_then_reexport_reacts() {
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
pub(super) fn impl_self_type_alias_to_a_foreign_type_stays_clean() {
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
pub(super) fn impl_of_a_locally_renamed_trait_reacts_by_true_leaf() {
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
pub(super) fn derive_of_a_locally_renamed_macro_reacts_by_true_leaf() {
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
pub(super) fn derive_renamed_to_a_nonforbidden_local_trait_stays_clean() {
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

#[test]
pub(super) fn hunyi_boundary_depth_getters_and_delegation_parity() {
    use xuanji::ScanDepth;

    // AsyncExposureBoundary
    let async_b = AsyncExposureBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_async_fn()
        .because("sync core");
    assert_eq!(async_b.scan_depth(), ScanDepth::Shallow);

    let async_subtree = AsyncExposureBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_async_fn()
        .depth(ScanDepth::Subtree)
        .because("sync core subtree");
    assert_eq!(async_subtree.scan_depth(), ScanDepth::Subtree);

    let async_submodules = AsyncExposureBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_async_fn()
        .including_submodules()
        .because("sync core submodules");
    assert_eq!(async_submodules.scan_depth(), ScanDepth::Subtree);

    // ImplTraitBoundary
    let impl_b = ImplTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .because("no RPIT leak");
    assert_eq!(impl_b.scan_depth(), ScanDepth::Shallow);

    let impl_subtree = ImplTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .depth(ScanDepth::Subtree)
        .because("no RPIT leak in subtree");
    assert_eq!(impl_subtree.scan_depth(), ScanDepth::Subtree);

    let impl_submodules = ImplTraitBoundary::in_crate("app")
        .module("crate::core")
        .must_not_expose_impl_trait()
        .including_submodules()
        .because("no RPIT leak in submodules");
    assert_eq!(impl_submodules.scan_depth(), ScanDepth::Subtree);
}

#[test]
pub(super) fn non_generic_compound_type_alias_target_walk_detects_nested_exposure() {
    let out = findings(
        "compound-alias-walk",
        &[
            ("lib.rs", "pub mod domain;\npub mod infra;\n"),
            (
                "domain.rs",
                "pub type TupleAlias = (crate::infra::DbPool, u32);\n\
                 pub type RefAlias = &'static crate::infra::DbPool;\n\
                 pub type SliceAlias = [crate::infra::DbPool];\n\
                 pub fn leak_tuple() -> TupleAlias { loop {} }\n\
                 pub fn leak_ref() -> RefAlias { loop {} }\n\
                 pub fn leak_slice() -> &'static SliceAlias { loop {} }\n",
            ),
            ("infra.rs", "pub struct DbPool;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        6,
        "6 exposures (3 type aliases + 3 functions using them) must be detected: {out:?}"
    );
}

#[test]
pub(super) fn tuple_alias_with_forbidden_type_in_first_position_and_private_helper_reacts() {
    let out = findings(
        "tuple-alias-first-pos",
        &[
            ("lib.rs", "pub mod domain;\npub mod infra;\npub mod api;\n"),
            (
                "domain.rs",
                "type PrivateHelper = (crate::infra::DbPool, crate::api::Public);\n\
                 pub fn leak_first_pos() -> PrivateHelper { loop {} }\n",
            ),
            ("infra.rs", "pub struct DbPool;\n"),
            ("api.rs", "pub struct Public;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        1,
        "private helper tuple alias with forbidden type in first position must react: {out:?}"
    );
    assert!(out[0].contains("crate::infra::DbPool exposed by fn crate::domain::leak_first_pos"));
}

#[test]
pub(super) fn raw_pointer_type_alias_target_walk_detects_nested_exposure() {
    let out = findings(
        "ptr-alias-walk",
        &[
            ("lib.rs", "pub mod domain;\npub mod infra;\n"),
            (
                "domain.rs",
                "pub type PtrAlias = *const crate::infra::DbPool;\n\
                 pub fn leak_ptr() -> PtrAlias { loop {} }\n",
            ),
            ("infra.rs", "pub struct DbPool;\n"),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out.len(),
        2,
        "raw pointer type alias declaration and function return type must react: {out:?}"
    );
}

#[test]
pub(super) fn wide_tuple_alias_with_forbidden_first_member_expands_without_truncation() {
    let out = findings(
        "wide-tuple-alias",
        &[
            ("lib.rs", "pub mod domain;\npub mod infra;\npub mod api;\n"),
            (
                "domain.rs",
                "type Inner = crate::infra::Secret;\n\
                 type Wide = (Inner, crate::api::A, crate::api::B, crate::api::C, crate::api::D, crate::api::E);\n\
                 pub fn leak_wide() -> Wide { loop {} }\n",
            ),
            ("infra.rs", "pub struct Secret;\n"),
            (
                "api.rs",
                "pub struct A;\npub struct B;\npub struct C;\npub struct D;\npub struct E;\n",
            ),
        ],
        "crate::domain",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        vec!["crate::infra::Secret exposed by fn crate::domain::leak_wide"],
        "wide tuple alias with 6 resolvable targets and forbidden target first must expand without truncation: {out:?}"
    );
}

#[test]
pub(super) fn diamond_alias_expansion_does_not_leak_intermediate_aliases() {
    use crate::resolve::{AliasMap, ReexportMap, expand_canonical_paths};
    let mut aliases = AliasMap::new();
    aliases.insert(
        "crate::domain::Mid".to_string(),
        vec!["crate::infra::Secret".to_string()],
    );
    aliases.insert(
        "crate::domain::Other".to_string(),
        vec!["crate::domain::Mid".to_string()],
    );
    aliases.insert(
        "crate::domain::Diamond".to_string(),
        vec![
            "crate::domain::Mid".to_string(),
            "crate::domain::Other".to_string(),
        ],
    );
    let reexports = ReexportMap::new();
    let expanded = expand_canonical_paths("crate::domain::Diamond", &aliases, &reexports);
    assert_eq!(
        expanded,
        vec!["crate::infra::Secret"],
        "diamond alias expansion must yield strictly terminal target without intermediate alias leakage: {expanded:?}"
    );
}
