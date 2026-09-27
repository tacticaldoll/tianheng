use super::super::*;
use super::helpers::TempSrcTree;

fn manifest(tree: &TempSrcTree) -> std::path::PathBuf {
    let path = tree.dir.join("Cargo.toml");
    std::fs::write(
        &path,
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    path
}

fn assert_auto_operand_error(outcome: xuanji::Outcome, operand: &str, kind: &str) {
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    let xuanji::Outcome::ConstitutionError(message) = outcome else {
        panic!("expected constitution error");
    };
    assert!(message.contains(operand), "{message}");
    assert!(message.contains(kind), "{message}");
    assert!(message.contains("remove"), "{message}");
}

#[test]
fn dyn_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("dyn-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_of(["Send"])
        .because("no dyn Port");
    assert_auto_operand_error(
        check_dyn_trait(&[boundary], &manifest(&tree)),
        "Send",
        "dyn",
    );
}

#[test]
fn dyn_qualified_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("dyn-qualified-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn crate::ports::Port + Sync> { todo!() }\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_of(["std::marker::Sync"])
        .because("no dyn Port");
    assert_auto_operand_error(
        check_dyn_trait(&[boundary], &manifest(&tree)),
        "std::marker::Sync",
        "dyn",
    );
}

#[test]
fn dyn_mixed_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("dyn-mixed-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn crate::ports::Port + Send> { todo!() }\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_of(["crate::ports::Port", "Send"])
        .because("no dyn Port");
    assert_auto_operand_error(
        check_dyn_trait(&[boundary], &manifest(&tree)),
        "Send",
        "dyn",
    );
}

#[test]
fn impl_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("impl-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> impl crate::ports::Port + Send { todo!() }\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_of(["Send"])
        .because("no impl Port");
    assert_auto_operand_error(
        check_impl_trait(&[boundary], &manifest(&tree)),
        "Send",
        "impl",
    );
}

#[test]
fn impl_subtree_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("impl-subtree-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub mod child;\n");
    tree.write("m/child.rs", "pub fn f() -> impl Send { todo!() }\n");
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_of(["Send"])
        .including_submodules()
        .because("no impl Send");
    assert_auto_operand_error(
        check_impl_trait(&[boundary], &manifest(&tree)),
        "Send",
        "impl",
    );
}

#[test]
fn mixed_auto_trait_operand_is_a_constitution_error() {
    let tree = TempSrcTree::new("mixed-auto-operand");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> impl crate::ports::Port + Send { todo!() }\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_of(["crate::ports::Port", "Send"])
        .because("no impl Port");
    assert_auto_operand_error(
        check_impl_trait(&[boundary], &manifest(&tree)),
        "Send",
        "impl",
    );
}

fn assert_qualified_auto_trait_operand_is_refused(operand: &str, label: &str) {
    let tree = TempSrcTree::new(label);
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub fn f() -> impl Send { todo!() }\n");
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_of([operand])
        .because("no impl Send");
    assert_auto_operand_error(
        check_impl_trait(&[boundary], &manifest(&tree)),
        operand,
        "impl",
    );
}

#[test]
fn qualified_std_auto_trait_operand_is_a_constitution_error() {
    assert_qualified_auto_trait_operand_is_refused(
        "std::marker::Sync",
        "qualified-std-auto-operand",
    );
}

#[test]
fn qualified_local_auto_trait_leaf_is_a_constitution_error() {
    assert_qualified_auto_trait_operand_is_refused(
        "crate::ports::Send",
        "qualified-local-auto-operand",
    );
}

#[test]
fn raw_auto_trait_leaf_is_a_constitution_error() {
    assert_qualified_auto_trait_operand_is_refused("r#Send", "raw-auto-operand");
}

#[test]
fn principal_collector_keeps_named_trait_and_discards_auto_marker() {
    use syn::visit::Visit;

    let node: syn::TypeTraitObject = syn::parse_str("dyn crate::ports::Port + Send").unwrap();
    let mut collector = crate::resolve::DynCollector::default();
    collector.visit_type_trait_object(&node);
    assert_eq!(collector.exposures.len(), 1);
    assert_eq!(collector.exposures[0].principals.len(), 1);
    assert_eq!(
        collector.exposures[0].principals[0]
            .segments
            .last()
            .unwrap()
            .ident,
        "Port"
    );
}

#[test]
fn named_principal_and_forbidden_marker_send_remain_observable() {
    let tree = TempSrcTree::new("auto-operand-controls");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub fn f() -> impl crate::ports::Port + Send { todo!() }\npub struct T;\nimpl Send for T {}\n");
    let manifest = manifest(&tree);
    let impl_boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_of(["crate::ports::Port"])
        .because("no impl Port");
    assert_eq!(check_impl_trait(&[impl_boundary], &manifest).exit_code(), 1);
    let marker = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate::m")
        .must_not_acquire("Send")
        .because("no Send acquisition");
    assert_eq!(check_forbidden_marker(&[marker], &manifest).exit_code(), 1);
}
