//! Foreign items carrying a `safe` or `unsafe` qualifier inside an `unsafe extern` block, which
//! `syn` 2 leaves as `ForeignItem::Verbatim`, read through the one decoder both the visibility ceiling and
//! signature-coupling match on.

use super::super::*;
use super::helpers::TempSrcTree;
use crate::syn_util::{ForeignDecl, decode_foreign_item};

/// An edition-2024 manifest for crate `x`, the edition that requires the `unsafe extern` block the
/// qualifiers stand in. The qualifiers themselves are not edition-gated: measured under rustc 1.96.1
/// and 1.85.1, `unsafe extern "C" { pub safe fn h(); pub unsafe static S: u8; }` compiles with
/// `--edition` 2015, 2021 and 2024 alike, and the same items in a plain `extern` block are refused in
/// each (`items in \`extern\` blocks without an \`unsafe\` qualifier cannot have safety qualifiers`,
/// and in 2024 `extern blocks must be unsafe`).
fn manifest(tree: &TempSrcTree) -> std::path::PathBuf {
    let path = tree.dir.join("Cargo.toml");
    std::fs::write(
        &path,
        "[package]\nname = \"x\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .unwrap();
    path
}

/// A crate whose `crate::kernel` holds `kernel_rs`, beside a `crate::internal` declaring `Handle`.
fn kernel_tree(label: &str, kernel_rs: &str) -> TempSrcTree {
    let tree = TempSrcTree::new(label);
    tree.write("lib.rs", "pub mod internal;\npub mod kernel;\n");
    tree.write("internal.rs", "#[repr(C)]\npub struct Handle(pub i32);\n");
    tree.write("kernel.rs", kernel_rs);
    tree
}

/// Every qualified shape, plus `pub unsafe fn`, which `syn` 2 already reads as `ForeignItem::Fn`.
/// Measured under rustc 1.96.1 and 1.85.1, edition 2024, `--crate-type lib`: this block compiles.
const QUALIFIED: &str = "unsafe extern \"C\" {
    pub safe static SAFE_FOREIGN: i32;
    pub safe fn safe_fn();
    pub safe static SAFE_HANDLE: crate::internal::Handle;
    pub safe fn safe_takes(h: crate::internal::Handle);
    pub unsafe static U: i32;
    pub unsafe fn u_fn();
    pub unsafe static U_HANDLE: crate::internal::Handle;
}
";

const QUALIFIED_VISIBILITY_FINDINGS: [&str; 7] = [
    "pub fn safe_fn",
    "pub fn safe_takes",
    "pub fn u_fn",
    "pub static SAFE_FOREIGN",
    "pub static SAFE_HANDLE",
    "pub static U",
    "pub static U_HANDLE",
];

fn findings(outcome: &Outcome) -> Vec<String> {
    let Outcome::Violations(report) = outcome else {
        panic!("expected violations, got {outcome:?}")
    };
    let mut out: Vec<String> = report
        .violations
        .iter()
        .map(|v| v.finding.clone())
        .collect();
    out.sort();
    out
}

#[test]
pub(super) fn qualified_foreign_items_react_under_a_module_ceiling() {
    let tree = kernel_tree("foreign-qualified-module", QUALIFIED);
    let boundary = VisibilityBoundary::in_crate("x")
        .module("crate::kernel")
        .max_visibility(VisibilityCeiling::Module)
        .because("the kernel's foreign declarations stay private to it");
    let outcome = check_visibility(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    assert_eq!(findings(&outcome), QUALIFIED_VISIBILITY_FINDINGS);
}

#[test]
pub(super) fn qualified_foreign_items_react_under_must_not_declare_pub() {
    let tree = kernel_tree("foreign-qualified-no-pub", QUALIFIED);
    let boundary = VisibilityBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_declare_pub()
        .because("the kernel declares no public foreign item");
    let outcome = check_visibility(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    assert_eq!(findings(&outcome), QUALIFIED_VISIBILITY_FINDINGS);
}

#[test]
pub(super) fn qualified_foreign_items_expose_their_signature_types() {
    let tree = kernel_tree("foreign-qualified-exposure", QUALIFIED);
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_expose("crate::internal")
        .because("the kernel's API names no internal type");
    let outcome = check(&[boundary], &manifest(&tree));
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    assert_eq!(
        findings(&outcome),
        [
            "crate::internal::Handle exposed by fn crate::kernel::safe_takes",
            "crate::internal::Handle exposed by static crate::kernel::SAFE_HANDLE",
            "crate::internal::Handle exposed by static crate::kernel::U_HANDLE",
        ]
    );
}

/// Kept for the contract rather than the change: these were clean before the decoder too, because
/// every qualified item was dropped. They hold that decoding does not read a private or
/// `pub(crate)` qualified item as more visible than it is declared.
#[test]
pub(super) fn qualified_foreign_items_at_or_below_the_ceiling_are_clean() {
    let tree = TempSrcTree::new("foreign-qualified-clean");
    tree.write(
        "lib.rs",
        "pub mod internal;\npub mod private;\npub mod crate_visible;\n",
    );
    tree.write("internal.rs", "#[repr(C)]\npub struct Handle(pub i32);\n");
    tree.write(
        "private.rs",
        "unsafe extern \"C\" {\n    safe static PRIV: i32;\n    safe fn priv_fn();\n    safe static PRIV_HANDLE: crate::internal::Handle;\n}\n",
    );
    tree.write(
        "crate_visible.rs",
        "unsafe extern \"C\" {\n    pub(crate) safe fn crate_fn();\n}\n",
    );
    let manifest = manifest(&tree);
    let private = VisibilityBoundary::in_crate("x")
        .module("crate::private")
        .max_visibility(VisibilityCeiling::Module)
        .because("private stays private");
    let crate_visible = VisibilityBoundary::in_crate("x")
        .module("crate::crate_visible")
        .max_visibility(VisibilityCeiling::Crate)
        .because("crate_visible declares nothing public");
    let outcome = check_visibility(&[private, crate_visible], &manifest);
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
    let exposure = SignatureBoundary::in_crate("x")
        .module("crate::private")
        .must_not_expose("crate::internal")
        .because("private exposes nothing");
    let outcome = check(&[exposure], &manifest);
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

/// The qualifier enters neither the finding nor its identity: `pub safe static X` is the same
/// declaration as `pub static X`, so a baseline accepting one accepts the other.
#[test]
pub(super) fn a_qualified_foreign_item_has_the_identity_of_its_unqualified_form() {
    let qualified = kernel_tree(
        "foreign-identity-qualified",
        "unsafe extern \"C\" {\n    pub safe static X: crate::internal::Handle;\n}\n",
    );
    let plain = kernel_tree(
        "foreign-identity-plain",
        "unsafe extern \"C\" {\n    pub static X: crate::internal::Handle;\n}\n",
    );
    let visibility = || {
        [VisibilityBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_declare_pub()
            .because("no pub")]
    };
    let exposure = || {
        [SignatureBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_expose("crate::internal")
            .because("no internal type")]
    };
    let outcomes = [
        (
            check_visibility(&visibility(), &manifest(&qualified)),
            check_visibility(&visibility(), &manifest(&plain)),
        ),
        (
            check(&exposure(), &manifest(&qualified)),
            check(&exposure(), &manifest(&plain)),
        ),
    ];
    for (from_qualified, from_plain) in outcomes {
        let (Outcome::Violations(q), Outcome::Violations(p)) = (&from_qualified, &from_plain)
        else {
            panic!("both must react: {from_qualified:?} / {from_plain:?}")
        };
        assert_eq!(q.violations.len(), 1, "{q:?}");
        assert_eq!(p.violations.len(), 1, "{p:?}");
        assert_eq!(q.violations[0].finding, p.violations[0].finding);
        assert_eq!(q.violations[0].id(), p.violations[0].id());
    }
}

/// A foreign item that stays `Verbatim` with any qualifier removed is refused, never skipped.
/// Measured under rustc 1.96.1 and 1.85.1, edition 2024: a foreign `fn` with a body behind a
/// `#[cfg]` that is off compiles, because the item is removed before the body is rejected, and
/// `syn` 2.0.118 reads it as `ForeignItem::Verbatim`. A `type` with a definition behind one does too,
/// measured under the same two toolchains with `--edition 2021`, and both are refused once enabled.
///
/// The refusal names the module as well as the file, says what is unknown without claiming the
/// visibility was unread — it was read — and names the repair: rustc accepts the item only while cfg
/// removes it (delete it) or an attribute macro rewrites it (write the expanded declaration directly).
#[test]
pub(super) fn an_undecodable_foreign_item_is_a_constitution_error() {
    for (label, item, name) in [
        (
            "foreign-undecodable-fn",
            "pub fn with_body() {}",
            "with_body",
        ),
        (
            "foreign-undecodable-type",
            "pub type Defined = u8;",
            "Defined",
        ),
    ] {
        let tree = kernel_tree(
            label,
            &format!("unsafe extern \"C\" {{\n    #[cfg(any())]\n    {item}\n}}\n"),
        );
        let manifest = manifest(&tree);
        let visibility = VisibilityBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_declare_pub()
            .because("no pub");
        let exposure = SignatureBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_expose("crate::internal")
            .because("no internal type");
        for outcome in [
            check_visibility(&[visibility], &manifest),
            check(&[exposure], &manifest),
        ] {
            assert_eq!(outcome.exit_code(), 2, "{label}: {outcome:?}");
            let Outcome::ConstitutionError(message) = outcome else {
                panic!("{label}: expected a constitution error")
            };
            for expected in [
                "cannot judge a foreign item in module 'crate::kernel'",
                name,
                "kernel.rs",
                "cannot tell what it declares",
                "only while a `#[cfg]` removes it",
                "delete it",
                "attribute macro",
                "write the expanded declaration directly",
            ] {
                assert!(
                    message.contains(expected),
                    "{label}: `{expected}` in {message}"
                );
            }
            assert!(!message.contains("visibility"), "{label}: {message}");
        }
    }
}

fn verbatim(tokens: &str) -> syn::ForeignItem {
    syn::ForeignItem::Verbatim(tokens.parse().expect("tokens lex"))
}

#[test]
pub(super) fn the_decoder_reads_a_qualified_static_and_fn_as_their_unqualified_forms() {
    let ForeignDecl::Static { vis, ident, .. } =
        decode_foreign_item(&verbatim("pub unsafe static X: i32;"))
            .ok()
            .unwrap()
    else {
        panic!("a static")
    };
    assert!(matches!(vis, syn::Visibility::Public(_)));
    assert_eq!(ident, "X");

    let ForeignDecl::Fn { vis, sig } = decode_foreign_item(&verbatim(
        "#[link_name = \"y\"] pub(crate) safe fn f(a: u8) -> u8;",
    ))
    .ok()
    .unwrap() else {
        panic!("a fn")
    };
    assert!(matches!(vis, syn::Visibility::Restricted(_)));
    assert_eq!(sig.ident, "f");

    let ForeignDecl::Static { ident, .. } =
        decode_foreign_item(&verbatim("pub safe static safe: i32;"))
            .ok()
            .unwrap()
    else {
        panic!("a static")
    };
    assert_eq!(
        ident, "safe",
        "the qualifier is removed by position, not by word"
    );
}

#[test]
pub(super) fn the_decoder_refuses_a_verbatim_that_stays_verbatim() {
    for (tokens, seen) in [
        ("pub fn with_body() {}", "`pub fn with_body () { }`"),
        (
            "pub safe fn with_body() {}",
            "`pub safe fn with_body () { }`",
        ),
        (
            "pub safe unsafe static X: i32;",
            "`pub safe unsafe static X : i32 ;`",
        ),
        ("pub static X: i32 = 1;", "`pub static X : i32 = 1 ;`"),
        ("pub safe 1 2 3", "`pub safe 1 2 3`"),
    ] {
        let Err(undecodable) = decode_foreign_item(&verbatim(tokens)) else {
            panic!("`{tokens}` must be refused")
        };
        assert_eq!(undecodable.seen, seen, "{tokens}");
    }
}
