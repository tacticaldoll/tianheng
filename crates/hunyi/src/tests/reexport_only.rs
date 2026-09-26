use super::super::*;
use super::helpers::TempSrcTree;

fn findings(name: &str, source: &str) -> Vec<String> {
    let tree = TempSrcTree::new(name);
    tree.write("lib.rs", source);
    reexport_only_findings(tree.src(), &tree.root(), "crate", "x", ScanDepth::Shallow)
        .unwrap()
        .into_iter()
        .map(|(fact, _module, _file)| fact.to_string())
        .collect()
}

#[test]
fn v1_helper_in_reexport_module_fails() {
    let tree = TempSrcTree::new("reexport-v1");
    tree.write("lib.rs", "pub use contract::*;\npub fn helper() {}\n");
    let boundary = ReexportOnlyBoundary::in_crate("x")
        .module("crate")
        .must_declare_only_reexports()
        .because("this module carries only re-exports");
    let mut violations = Vec::new();
    check_reexport_only_boundary(&tree.metadata(), &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].finding.as_str(), "fn helper");
    assert_eq!(crate::driver::outcome_from(violations, 1, 1).exit_code(), 1);
}

#[test]
fn v2_impl_is_rendered_as_a_finding() {
    assert_eq!(
        findings("reexport-v2", "impl Trait for Foo {}\n"),
        ["impl Trait for Foo"]
    );
}

#[test]
fn v3_item_macro_is_rendered_as_a_finding() {
    assert_eq!(
        findings("reexport-v3", "some_macro!{}\n"),
        ["macro some_macro!"]
    );
}

#[test]
fn kind_struct() {
    assert_eq!(findings("kind_struct", "struct S;"), ["struct S"]);
}
#[test]
fn kind_enum() {
    assert_eq!(findings("kind_enum", "enum E {}"), ["enum E"]);
}
#[test]
fn kind_union() {
    assert_eq!(findings("kind_union", "union U { x: u8 }"), ["union U"]);
}
#[test]
fn kind_type() {
    assert_eq!(findings("kind_type", "type T = u8;"), ["type T"]);
}
#[test]
fn kind_const() {
    assert_eq!(findings("kind_const", "const C: u8 = 0;"), ["const C"]);
}
#[test]
fn kind_static() {
    assert_eq!(findings("kind_static", "static K: u8 = 0;"), ["static K"]);
}
#[test]
fn kind_trait() {
    assert_eq!(findings("kind_trait", "trait T {}"), ["trait T"]);
}
#[test]
fn kind_trait_alias() {
    assert_eq!(
        findings("kind_trait_alias", "trait A = Send;"),
        ["trait alias A"]
    );
}
#[test]
fn kind_mod() {
    assert_eq!(findings("kind_mod", "mod child {}"), ["mod child"]);
}
#[test]
fn kind_extern_crate() {
    assert_eq!(
        findings("kind_extern_crate", "extern crate core;"),
        ["extern crate core"]
    );
}
#[test]
fn kind_extern_block() {
    assert_eq!(
        findings("kind_extern_block", "extern \"C\" { fn x(); }"),
        ["extern block"]
    );
}
#[test]
fn kind_macro_rules() {
    assert_eq!(
        findings("kind_macro_rules", "macro_rules! m { () => {} }"),
        ["macro_rules m"]
    );
}
#[test]
fn kind_inherent_impl() {
    assert_eq!(findings("kind_inherent_impl", "impl Foo {}"), ["impl Foo"]);
}
#[test]
fn kind_fn() {
    assert_eq!(findings("kind_fn", "fn helper() {}"), ["fn helper"]);
}
#[test]
fn clean_glob() {
    assert!(findings("clean_glob", "pub use c::*;").is_empty());
}
#[test]
fn clean_selective() {
    assert!(findings("clean_selective", "pub use c::{A, B};").is_empty());
}
#[test]
fn clean_private() {
    assert!(findings("clean_private", "use c::A;").is_empty());
}
#[test]
fn clean_crate_use() {
    assert!(findings("clean_crate_use", "pub(crate) use c::A;").is_empty());
}
#[test]
fn clean_real_facade() {
    assert!(
        findings(
            "clean_real_facade",
            "//! Facade documentation.\npub use contract::*;"
        )
        .is_empty()
    );
}

#[test]
fn subtree_permits_containers_and_governs_children() {
    let tree = TempSrcTree::new("reexport-subtree");
    tree.write(
        "lib.rs",
        "pub use contract::*;\npub mod prelude { pub use crate::contract::A; }\n",
    );
    let clean =
        reexport_only_findings(tree.src(), &tree.root(), "crate", "x", ScanDepth::Subtree).unwrap();
    assert!(clean.is_empty());
    tree.write(
        "lib.rs",
        "pub mod prelude { pub use crate::contract::A; fn hidden() {} }\n",
    );
    let findings =
        reexport_only_findings(tree.src(), &tree.root(), "crate", "x", ScanDepth::Subtree).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].0.to_string(), "fn hidden");
}

#[test]
fn cfg_if_arm_item_is_observed() {
    let source =
        "cfg_if::cfg_if! { if #[cfg(unix)] { fn unix_helper() {} } else { pub use c::*; } }";
    assert_eq!(findings("cfg-if", source), ["fn unix_helper"]);
}

#[test]
fn unresolved_anchor_is_constitution_error() {
    let tree = TempSrcTree::new("reexport-unknown-anchor");
    tree.write("lib.rs", "pub use contract::*;");
    let boundary = ReexportOnlyBoundary::in_crate("x")
        .module("crate::missing")
        .must_declare_only_reexports()
        .because("only re-exports");
    let mut violations = Vec::new();
    assert!(check_reexport_only_boundary(&tree.metadata(), &boundary, &mut violations).is_err());
    assert!(violations.is_empty());
    let manifest = tree.dir.join("Cargo.toml");
    std::fs::write(
        &manifest,
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    assert_eq!(check_reexport_only(&[boundary], &manifest).exit_code(), 2);
}

#[test]
fn repeated_macro_path_shares_one_identity() {
    let tree = TempSrcTree::new("reexport-repeat-macro");
    tree.write("lib.rs", "m!{}\nm!{}\n");
    let facts =
        reexport_only_findings(tree.src(), &tree.root(), "crate", "x", ScanDepth::Shallow).unwrap();
    assert_eq!(facts.len(), 1);
    assert_eq!(facts[0].0.to_string(), "macro m!");
}

#[test]
fn unrenderable_item_is_a_finding() {
    let (kind, name) =
        crate::reexport_only::describe_item(&syn::Item::Verbatim(Default::default()));
    assert_eq!(
        (kind.as_str(), name.as_str()),
        ("verbatim", "<unrenderable>")
    );
}

#[test]
fn function_body_impl_is_not_a_direct_item() {
    assert_eq!(
        findings("nested-impl", "fn outer() { impl Foo {} }"),
        ["fn outer"]
    );
}

#[test]
fn cfg_is_observed_as_written() {
    assert_eq!(
        findings("cfg-written", "#[cfg(windows)] fn gated() {}"),
        ["fn gated"]
    );
}

#[test]
fn rule_key_and_fact_are_distinct_from_visibility() {
    let tree = TempSrcTree::new("reexport-identity");
    tree.write("lib.rs", "pub fn helper() {}\n");
    let reexport = ReexportOnlyBoundary::in_crate("x")
        .module("crate")
        .must_declare_only_reexports()
        .because("only re-exports");
    let visibility = VisibilityBoundary::in_crate("x")
        .module("crate")
        .must_not_declare_pub()
        .because("no bare pub");
    let mut a = Vec::new();
    let mut b = Vec::new();
    check_reexport_only_boundary(&tree.metadata(), &reexport, &mut a).unwrap();
    check_visibility_boundary(&tree.metadata(), &visibility, &mut b).unwrap();
    assert_eq!(a.len(), 1);
    assert_eq!(b.len(), 1);
    assert_eq!(
        a[0].rule_key().rule_type(),
        "tianheng.rule/hunyi/reexport-only-module"
    );
    assert_eq!(
        b[0].rule_key().rule_type(),
        "tianheng.rule/hunyi/visibility-ceiling"
    );
    assert_eq!(a[0].fact().shape(), "declared-item-kind");
    assert_eq!(b[0].fact().shape(), "declared-item-visibility");
    assert_eq!(a[0].polarity, Some(xuanji::Polarity::DenyBreach));
}

#[test]
fn same_named_child_items_keep_distinct_identities() {
    let tree = TempSrcTree::new("reexport-child-identity");
    tree.write(
        "lib.rs",
        "mod a { fn helper() {} }\nmod b { fn helper() {} }\n",
    );
    let boundary = ReexportOnlyBoundary::in_crate("x")
        .module("crate")
        .must_declare_only_reexports()
        .depth(ScanDepth::Subtree)
        .because("only re-exports");
    let mut violations = Vec::new();
    check_reexport_only_boundary(&tree.metadata(), &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 2);
    let names: Vec<_> = violations
        .iter()
        .map(|v| {
            let fields: Vec<_> = v.fact().fields().collect();
            assert_eq!(fields.len(), 4);
            fields
                .into_iter()
                .find(|(key, _)| *key == "item_name")
                .unwrap()
                .1
                .to_string()
        })
        .collect();
    assert_eq!(names, ["crate::a::helper", "crate::b::helper"]);
}
