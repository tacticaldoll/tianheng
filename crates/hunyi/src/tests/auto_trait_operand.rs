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
    let builder = match kind {
        "dyn" => "must_not_expose_dyn_bounded_by",
        _ => "must_not_expose_impl_trait_bounded_by",
    };
    assert!(message.contains(builder), "{message}");
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

// --- B1 Auto-Trait Bound Governance Tests ---

#[test]
fn impl_auto_bound_trait_method_violation() {
    let tree = TempSrcTree::new("impl-trait-method-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub trait AsyncRegistry {\n    fn settle(&self) -> impl std::future::Future<Output = ()> + Send;\n}\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no returned Send impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let xuanji::Outcome::Violations(report) = outcome else {
        panic!("{outcome:?}")
    };
    let finding = &report.violations[0].finding;
    assert!(finding.contains("Future<Output = ()> + Send"));
    assert!(finding.contains("AsyncRegistry::settle"));
}

#[test]
fn impl_auto_bound_free_fn_violation() {
    let tree = TempSrcTree::new("impl-free-fn-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn items() -> impl Iterator<Item = u8> + Sync { todo!() }\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Sync"])
        .because("no returned Sync impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let xuanji::Outcome::Violations(report) = outcome else {
        panic!("{outcome:?}")
    };
    let finding = &report.violations[0].finding;
    assert!(finding.contains("impl Iterator<Item = u8> + Sync"));
}

#[test]
fn impl_auto_bound_inherent_method_violation() {
    let tree = TempSrcTree::new("impl-inherent-method-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub struct Service;\nimpl Service {\n    pub fn run(&self) -> impl std::future::Future<Output = ()> + Send + Sync { todo!() }\n}\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no returned Send impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let xuanji::Outcome::Violations(report) = outcome else {
        panic!("{outcome:?}")
    };
    let finding = &report.violations[0].finding;
    assert!(finding.contains("Future<Output = ()> + Send + Sync"));
}

#[test]
fn impl_auto_bound_qualified_path_violation() {
    let tree = TempSrcTree::new("impl-qualified-path-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn run() -> impl std::future::Future<Output = ()> + std::marker::Send { todo!() }\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no returned Send impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
}

#[test]
fn impl_auto_bound_subtree_submodule_violation() {
    let tree = TempSrcTree::new("impl-subtree-submodule-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub mod sub;\n");
    tree.write(
        "m/sub.rs",
        "pub fn sub_fn() -> impl std::future::Future<Output = ()> + Send { todo!() }\n",
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .including_submodules()
        .because("no returned Send impl trait in subtree");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
}

#[test]
fn impl_auto_bound_clean_cases() {
    let tree = TempSrcTree::new("impl-clean-cases");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
// 1. impl Future without Send
pub fn fut() -> impl std::future::Future<Output = ()> { todo!() }

// 2. associated type with Send bound, but return type without Send
pub trait WithAssoc {
    type Error: std::error::Error + Send;
    fn process(&self) -> impl std::future::Future<Output = ()>;
}

// 3. APIT (argument position) carrying Send
pub fn consume(x: impl std::fmt::Display + Send) { let _ = x; }

// 4. (nested_dyn moved to dedicated test)

// 5. async fn (implicit compiler existential, not written RPIT)
pub async fn async_worker() {}
"#,
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no returned Send impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

#[test]
fn impl_auto_bound_errors() {
    let tree = TempSrcTree::new("impl-errors");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> impl std::future::Future + Send { todo!() }\n",
    );
    let m = manifest(&tree);

    // Empty set -> 2, points to must_not_expose_impl_trait
    let empty_b = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by([] as [&str; 0])
        .because("empty");
    let outcome = check_impl_trait(&[empty_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("must_not_expose_impl_trait()"));

    // Non-auto trait Clone -> 2, points to must_not_expose_impl_trait_of
    let clone_b = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Clone"])
        .because("clone");
    let outcome = check_impl_trait(&[clone_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("must_not_expose_impl_trait_of"));

    // Malformed path ::Send -> 2
    let malformed_b = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["::Send"])
        .because("malformed");
    let outcome = check_impl_trait(&[malformed_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("no empty segment"));
}

#[test]
fn dyn_auto_bound_trait_method_violation() {
    let tree = TempSrcTree::new("dyn-trait-method-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "use std::pin::Pin;\npub trait AsyncRegistry {\n    fn settle(&self) -> Pin<Box<dyn std::future::Future<Output = ()> + Send>>;\n}\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("no returned Send dyn");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let xuanji::Outcome::Violations(report) = outcome else {
        panic!("{outcome:?}")
    };
    let finding = &report.violations[0].finding;
    assert!(finding.contains("Future<Output = ()> + Send"));
}

#[test]
fn dyn_auto_bound_free_fn_violation() {
    let tree = TempSrcTree::new("dyn-free-fn-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn handler() -> Box<dyn Fn() + Send + Sync> { todo!() }\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("no returned Send dyn");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let xuanji::Outcome::Violations(report) = outcome else {
        panic!("{outcome:?}")
    };
    let finding = &report.violations[0].finding;
    assert!(finding.contains("dyn Fn() + Send + Sync"));
}

#[test]
fn dyn_auto_bound_qualified_path_violation() {
    let tree = TempSrcTree::new("dyn-qualified-path-violation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn handler() -> Box<dyn std::fmt::Display + std::marker::Send> { todo!() }\n",
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("no returned Send dyn");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
}

#[test]
fn dyn_auto_bound_clean_cases() {
    let tree = TempSrcTree::new("dyn-clean-cases");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
// 1. Box<dyn Fn()> without Send
pub fn f1() -> Box<dyn Fn()> { todo!() }

// 2. Returns Box<dyn Fn() -> Box<dyn Iterator<Item = u8> + Send>>, but boundary forbids ["Sync"]
pub fn f2() -> Box<dyn Fn() -> Box<dyn Iterator<Item = u8> + Send>> { todo!() }
"#,
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Sync"])
        .because("no Sync dyn");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

#[test]
fn dyn_auto_bound_errors() {
    let tree = TempSrcTree::new("dyn-errors");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn std::fmt::Display + Send> { todo!() }\n",
    );
    let m = manifest(&tree);

    // Empty set -> 2, points to must_not_expose_dyn
    let empty_b = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by([] as [&str; 0])
        .because("empty");
    let outcome = check_dyn_trait(&[empty_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("must_not_expose_dyn()"));

    // Non-auto trait Clone -> 2, points to must_not_expose_dyn_of
    let clone_b = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Clone"])
        .because("clone");
    let outcome = check_dyn_trait(&[clone_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("must_not_expose_dyn_of"));

    // Malformed path ::Send -> 2
    let malformed_b = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["::Send"])
        .because("malformed");
    let outcome = check_dyn_trait(&[malformed_b], &m);
    assert_eq!(outcome.exit_code(), 2);
    let xuanji::Outcome::ConstitutionError(msg) = outcome else {
        panic!()
    };
    assert!(msg.contains("no empty segment"));
}

#[test]
fn shape_and_auto_bound_violation_ids_do_not_collide() {
    let tree = TempSrcTree::new("identity-no-collision");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> impl std::future::Future<Output = ()> + Send { todo!() }\n",
    );
    let m = manifest(&tree);
    let shape_boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait()
        .because("shape only");
    let auto_boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("auto bound only");

    let outcome_shape = check_impl_trait(&[shape_boundary], &m);
    let outcome_auto = check_impl_trait(&[auto_boundary], &m);

    let xuanji::Outcome::Violations(rep_shape) = outcome_shape else {
        panic!()
    };
    let xuanji::Outcome::Violations(rep_auto) = outcome_auto else {
        panic!()
    };
    let v_shape = &rep_shape.violations[0];
    let v_auto = &rep_auto.violations[0];

    assert_eq!(v_shape.target(), v_auto.target());
    assert_ne!(v_shape.id(), v_auto.id(), "violation IDs must not collide");
    assert_ne!(v_shape.rule_key(), v_auto.rule_key());
}

#[test]
fn impl_trait_local_auto_trait_leaf_over_reacts_is_a_bound() {
    let tree = TempSrcTree::new("impl-local-auto-leaf-bound");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
pub trait Send {}
pub fn f() -> impl Send { todo!() }
"#,
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("impl-local-auto-leaf-bound");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(
        outcome.exit_code(),
        1,
        "expected over-reaction on local Send trait"
    );
}

#[test]
fn impl_trait_macro_generated_auto_bound_is_a_bound() {
    let tree = TempSrcTree::new("impl-macro-auto-bound");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
macro_rules! emit_fut_macro_token {
    () => {
        pub fn f() -> impl std::future::Future + Send { todo!() }
    };
}
emit_fut_macro_token!();
"#,
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no Send");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(
        outcome.exit_code(),
        0,
        "macro-generated impl trait is out of reach"
    );
}

#[test]
fn dyn_trait_local_auto_trait_leaf_over_reacts_is_a_bound() {
    let tree = TempSrcTree::new("dyn-local-auto-leaf-bound");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
pub trait Send {}
pub fn f() -> Box<dyn Send> { todo!() }
"#,
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("dyn-local-auto-leaf-bound");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(
        outcome.exit_code(),
        1,
        "expected over-reaction on local Send trait"
    );
}

#[test]
fn dyn_macro_generated_auto_bound_is_a_bound() {
    let tree = TempSrcTree::new("dyn-macro-auto-bound");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
macro_rules! emit_dyn_macro_token {
    () => {
        pub fn f() -> Box<dyn std::any::Any + Send> { todo!() }
    };
}
emit_dyn_macro_token!();
"#,
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("no Send");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(
        outcome.exit_code(),
        0,
        "macro-generated dyn is out of reach"
    );
}

#[test]
fn dyn_private_alias_hiding_auto_bound_is_a_bound() {
    let tree = TempSrcTree::new("dyn-alias-auto-bound");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        r#"
type HiddenAlias = Box<dyn std::any::Any + Send>;
pub fn f() -> HiddenAlias { todo!() }
"#,
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("no Send");
    let outcome = check_dyn_trait(&[boundary], &manifest(&tree));
    assert_eq!(
        outcome.exit_code(),
        0,
        "alias-hidden dyn is out of reach from signature"
    );
}

#[test]
fn impl_trait_auto_bound_nested_dyn_does_not_react() {
    let tree = TempSrcTree::new("impl-nested-dyn");
    tree.write(
        "lib.rs",
        "pub mod m;
",
    );
    tree.write(
        "m.rs",
        r#"
// Send is on inner dyn, not on the outer impl Trait — in impl rule this must not react.
pub fn nested_dyn() -> impl std::future::Future<Output = Box<dyn std::error::Error + Send>> { todo!() }
"#,
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("no returned Send impl trait");
    let outcome = check_impl_trait(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

#[test]
fn impl_auto_bound_rule_key_normalized_identity() {
    let b1 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send"])
        .because("reason");
    let b2 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["std::marker::Send"])
        .because("reason");
    let b3 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["core::marker::Send"])
        .because("reason");
    let b4 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["r#Send"])
        .because("reason");
    let b5 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["Send", "std::marker::Send"])
        .because("reason");

    assert_eq!(b1.rule_key(), b2.rule_key());
    assert_eq!(b1.rule_key(), b3.rule_key());
    assert_eq!(b1.rule_key(), b4.rule_key());
    assert_eq!(b1.rule_key(), b5.rule_key());
    assert_eq!(
        b1.rule_key()
            .fields()
            .find(|(k, _)| *k == "forbidden_auto_bounds")
            .map(|(_, v)| v),
        Some(r#"["Send"]"#)
    );
}

#[test]
fn dyn_auto_bound_rule_key_normalized_identity() {
    let b1 = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send"])
        .because("reason");
    let b2 = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["std::marker::Send"])
        .because("reason");
    let b3 = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["core::marker::Send"])
        .because("reason");
    let b4 = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["r#Send"])
        .because("reason");
    let b5 = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["Send", "std::marker::Send"])
        .because("reason");

    assert_eq!(b1.rule_key(), b2.rule_key());
    assert_eq!(b1.rule_key(), b3.rule_key());
    assert_eq!(b1.rule_key(), b4.rule_key());
    assert_eq!(b1.rule_key(), b5.rule_key());
    assert_eq!(
        b1.rule_key()
            .fields()
            .find(|(k, _)| *k == "forbidden_auto_bounds")
            .map(|(_, v)| v),
        Some(r#"["Send"]"#)
    );
}

#[test]
fn impl_auto_bound_invalid_qualifier_exits_2_shallow_and_subtree() {
    let tree = TempSrcTree::new("impl-invalid-qual");
    tree.write(
        "lib.rs",
        "pub mod m;
",
    );
    tree.write(
        "m.rs",
        "pub fn f() -> impl std::future::Future + Send { todo!() }
",
    );

    // Shallow
    let b1 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["foo::Send"])
        .because("reason");
    let o1 = check_impl_trait(&[b1], &manifest(&tree));
    assert_eq!(o1.exit_code(), 2, "{o1:?}");
    let err1 = match o1 {
        crate::Outcome::ConstitutionError(e) => e,
        other => panic!("expected constitution error, got {other:?}"),
    };
    assert!(err1.contains("foo::Send"), "{err1}");

    // Subtree
    let b2 = ImplTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_impl_trait_bounded_by(["foo::Send"])
        .including_submodules()
        .because("reason");
    let o2 = check_impl_trait(&[b2], &manifest(&tree));
    assert_eq!(o2.exit_code(), 2, "{o2:?}");
    let err2 = match o2 {
        crate::Outcome::ConstitutionError(e) => e,
        other => panic!("expected constitution error, got {other:?}"),
    };
    assert!(err2.contains("foo::Send"), "{err2}");
}

#[test]
fn dyn_auto_bound_invalid_qualifier_exits_2() {
    let tree = TempSrcTree::new("dyn-invalid-qual");
    tree.write(
        "lib.rs",
        "pub mod m;
",
    );
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn std::any::Any + Send> { todo!() }
",
    );

    let b = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["foo::Send"])
        .because("reason");
    let o = check_dyn_trait(&[b], &manifest(&tree));
    assert_eq!(o.exit_code(), 2, "{o:?}");
    let err = match o {
        crate::Outcome::ConstitutionError(e) => e,
        other => panic!("expected constitution error, got {other:?}"),
    };
    assert!(err.contains("foo::Send"), "{err}");
}

#[test]
fn impl_auto_bound_three_segment_invalid_qualifier_exits_2_shallow_and_subtree() {
    let tree = TempSrcTree::new("impl-three-segment-invalid-qual");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub fn f() -> impl Send { todo!() }\n");
    let m = manifest(&tree);

    for boundary in [
        ImplTraitBoundary::in_crate("x")
            .module("crate::m")
            .must_not_expose_impl_trait_bounded_by(["crate::marker::Send"])
            .because("reason"),
        ImplTraitBoundary::in_crate("x")
            .module("crate::m")
            .must_not_expose_impl_trait_bounded_by(["crate::marker::Send"])
            .including_submodules()
            .because("reason"),
    ] {
        assert_eq!(check_impl_trait(&[boundary], &m).exit_code(), 2);
    }
}

#[test]
fn dyn_auto_bound_three_segment_invalid_qualifier_exits_2() {
    let tree = TempSrcTree::new("dyn-three-segment-invalid-qual");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write("m.rs", "pub fn f() -> Box<dyn Send> { todo!() }\n");
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["crate::marker::Send"])
        .because("reason");
    assert_eq!(
        check_dyn_trait(&[boundary], &manifest(&tree)).exit_code(),
        2
    );
}

#[test]
fn auto_bound_qualified_path_matrix() {
    let cases = [
        ("std::panic::UnwindSafe", true),
        ("core::panic::RefUnwindSafe", true),
        ("std::marker::Send", true),
        ("std::marker::UnwindSafe", false),
        ("std::panic::Send", false),
    ];
    for (path, accepted) in cases {
        for kind in ["impl", "dyn"] {
            let tree = TempSrcTree::new(&format!("auto-bound-path-{kind}-{path}"));
            tree.write("lib.rs", "pub mod m;\n");
            let source = if kind == "impl" {
                format!(
                    "pub fn f() -> impl std::future::Future<Output = ()> + {path} {{ todo!() }}\n"
                )
            } else {
                format!(
                    "pub fn f() -> Box<dyn std::future::Future<Output = ()> + {path}> {{ todo!() }}\n"
                )
            };
            tree.write("m.rs", &source);
            let exit = if kind == "impl" {
                let boundary = ImplTraitBoundary::in_crate("x")
                    .module("crate::m")
                    .must_not_expose_impl_trait_bounded_by([path])
                    .because("auto-bound path guard");
                check_impl_trait(&[boundary], &manifest(&tree)).exit_code()
            } else {
                let boundary = DynTraitBoundary::in_crate("x")
                    .module("crate::m")
                    .must_not_expose_dyn_bounded_by([path])
                    .because("auto-bound path guard");
                check_dyn_trait(&[boundary], &manifest(&tree)).exit_code()
            };
            assert_eq!(exit, if accepted { 1 } else { 2 }, "{kind} {path}");
        }
    }
}
#[test]
fn auto_trait_operand_error_recommendation_is_executable() {
    let tree = TempSrcTree::new("auto-operand-recommendation");
    tree.write("lib.rs", "pub mod m;\n");
    tree.write(
        "m.rs",
        "pub fn f() -> Box<dyn crate::ports::Port + std::panic::UnwindSafe> { todo!() }\n",
    );
    let bad = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["std::marker::UnwindSafe"])
        .because("auto-bound recommendation");
    let outcome = check_dyn_trait(&[bad], &manifest(&tree));
    let message = match outcome {
        crate::Outcome::ConstitutionError(message) => message,
        other => panic!("expected constitution error, got {other:?}"),
    };
    assert!(
        message.contains("UnwindSafe is defined in std::panic"),
        "{message}"
    );
    assert!(message.contains("std::panic::UnwindSafe"), "{message}");
    assert!(message.contains("bare UnwindSafe"), "{message}");
    let good = DynTraitBoundary::in_crate("x")
        .module("crate::m")
        .must_not_expose_dyn_bounded_by(["UnwindSafe"])
        .because("auto-bound recommendation");
    assert_eq!(check_dyn_trait(&[good], &manifest(&tree)).exit_code(), 1);
}
