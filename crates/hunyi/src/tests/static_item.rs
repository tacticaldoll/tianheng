use super::super::*;
use super::helpers::*;
use crate::errors::unknown_module_error;
use crate::static_item::check_static_boundary;
// --- static-item boundary: no `static` or `thread_local!` under the anchored module -----------

fn boundary(module: &str) -> StaticBoundary {
    StaticBoundary::in_crate("x")
        .module(module)
        .must_not_declare_static()
        .because("the kernel declares no `static` item or `thread_local!`")
}

/// Every violation a boundary on `module` produces over a fixture, or its refusal.
fn statics(name: &str, files: &[(&str, &str)], module: &str) -> Result<Vec<Violation>, String> {
    let (metadata, _tree) = fixture_metadata(name, files);
    let mut violations = Vec::new();
    check_static_boundary(&metadata, &boundary(module), &mut violations)?;
    Ok(violations)
}

/// One `(kind, module, name, owner)` row per violation, read from its structured identity.
fn rows(violations: &[Violation]) -> Vec<(String, String, String, String)> {
    violations
        .iter()
        .map(|violation| {
            let fact = violation.fact();
            let field = |name: &str| {
                fact.fields()
                    .find(|(field, _)| *field == name)
                    .map(|(_, value)| value.to_string())
                    .unwrap_or_else(|| panic!("the static-item fact carries `{name}`"))
            };
            (
                fact.shape().to_string(),
                field("module"),
                field("name"),
                field("owner"),
            )
        })
        .collect()
}

fn row(kind: &str, module: &str, name: &str, owner: &str) -> (String, String, String, String) {
    (
        kind.to_string(),
        module.to_string(),
        name.to_string(),
        owner.to_string(),
    )
}

fn lib_and_kernel(kernel: &str) -> Vec<(&'static str, String)> {
    vec![
        ("lib.rs", "pub mod kernel;\npub mod other;\n".to_string()),
        ("kernel.rs", kernel.to_string()),
        ("other.rs", "pub static OUTSIDE: u8 = 0;\n".to_string()),
    ]
}

fn kernel_statics(name: &str, kernel: &str) -> Result<Vec<Violation>, String> {
    let files = lib_and_kernel(kernel);
    let files: Vec<(&str, &str)> = files.iter().map(|(p, c)| (*p, c.as_str())).collect();
    statics(name, &files, "crate::kernel")
}

#[test]
pub(super) fn a_module_level_static_and_static_mut_react_with_their_kind() {
    let violations = kernel_statics(
        "static-module-level",
        "pub static COUNTER: u8 = 0;\nstatic mut LEGACY: u8 = 0;\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "COUNTER", ""),
            row("static_mut", "crate::kernel", "LEGACY", ""),
        ]
    );
    let violation = &violations[0];
    assert_eq!(violation.target(), "crate::kernel");
    assert_eq!(violation.rule, "must not declare static items");
    assert_eq!(violation.polarity, Some(Polarity::DenyBreach));
    assert_eq!(
        violation.fact().fact_type(),
        "tianheng.fact/hunyi/static-item"
    );
    let mut fields: Vec<&str> = violation.fact().fields().map(|(name, _)| name).collect();
    fields.sort_unstable();
    assert_eq!(
        fields,
        ["governing_package", "module", "name", "owner", "unit"]
    );
}

/// A static in a body is owned by the chain of named value items around it, a closure belonging to
/// the fn that holds it and an initializer to the item it initializes.
#[test]
pub(super) fn a_body_static_is_owned_by_its_named_value_items() {
    let violations = kernel_statics(
        "static-owners",
        "pub struct Foo;\npub trait Tr { fn d() { static IN_DEFAULT: u8 = 0; } fn m(); }\n\
         pub fn f() { static IN_FN: u8 = 0; fn inner() { static IN_INNER: u8 = 0; } }\n\
         impl Foo { pub fn m() { static IN_METHOD: u8 = 0; } }\n\
         impl Tr for Foo { fn m() { static IN_IMPL: u8 = 0; } }\n\
         pub fn g() { let c = || { static IN_CLOSURE: u8 = 0; }; c(); }\n\
         pub const C: u8 = { static IN_CONST: u8 = 0; 1 };\n\
         pub static S: u8 = { static IN_STATIC: u8 = 0; 1 };\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    let mut expected = vec![
        row("static", "crate::kernel", "IN_DEFAULT", "Tr::d"),
        row("static", "crate::kernel", "IN_FN", "f"),
        row("static", "crate::kernel", "IN_INNER", "f::inner"),
        row("static", "crate::kernel", "IN_METHOD", "Foo::m"),
        row("static", "crate::kernel", "IN_IMPL", "<Tr for Foo>::m"),
        row("static", "crate::kernel", "IN_CLOSURE", "g"),
        row("static", "crate::kernel", "IN_CONST", "const C"),
        row("static", "crate::kernel", "IN_STATIC", "static S"),
        row("static", "crate::kernel", "S", ""),
    ];
    expected.sort();
    assert_eq!(observed, expected);
}

/// One `thread_local!` declaring two statics is two findings: the second is the one a reader
/// taking the first item of the body would drop.
#[test]
pub(super) fn every_static_a_thread_local_declares_reacts() {
    let violations = kernel_statics(
        "static-thread-local-two",
        "use std::cell::Cell;\nthread_local! {\n    static SCRATCH: Cell<u8> = const { Cell::new(0) };\n    \
         pub static DEPTH: Cell<u8> = Cell::new(0);\n}\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("thread_local", "crate::kernel", "DEPTH", ""),
            row("thread_local", "crate::kernel", "SCRATCH", ""),
        ]
    );
}

/// std's grammar makes the last declaration's `;` optional, and its own documentation writes
/// `thread_local!(static FOO: Cell<u32> = Cell::new(1));`. So a body whose last static carries no `;`
/// is observed, and so is every static before it in the same body.
#[test]
pub(super) fn a_thread_local_whose_last_static_has_no_semicolon_reacts() {
    let violations = kernel_statics(
        "static-thread-local-unterminated",
        "use std::cell::Cell;\nthread_local!(static FOO: Cell<u32> = Cell::new(1));\n\
         thread_local! {\n    static A: Cell<u8> = Cell::new(0);\n    \
         pub static B: Cell<u8> = const { Cell::new(0) }\n}\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("thread_local", "crate::kernel", "A", ""),
            row("thread_local", "crate::kernel", "B", ""),
            row("thread_local", "crate::kernel", "FOO", ""),
        ]
    );
}

#[test]
pub(super) fn a_statement_position_thread_local_is_owned_by_its_fn() {
    let violations = kernel_statics(
        "static-thread-local-stmt",
        "pub fn bump() { std::thread_local! { static LOCAL: u8 = 0; } }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("thread_local", "crate::kernel", "LOCAL", "bump")]
    );
}

/// `std::thread_local!`, `::std::thread_local!` and `r#thread_local!` are the bare macro by name, so
/// each declares the same identity the bare spelling does.
#[test]
pub(super) fn every_spelling_of_thread_local_shares_the_bare_identity() {
    let ids = |label: &str, invocation: &str| {
        kernel_statics(label, &format!("{invocation} {{ static T: u8 = 0; }}\n"))
            .unwrap()
            .iter()
            .map(Violation::id)
            .collect::<Vec<_>>()
    };
    let bare = ids("static-tl-bare", "thread_local!");
    assert_eq!(bare.len(), 1);
    for (label, invocation) in [
        ("static-tl-std", "std::thread_local!"),
        ("static-tl-global", "::std::thread_local!"),
        ("static-tl-raw", "r#thread_local!"),
    ] {
        assert_eq!(ids(label, invocation), bare, "{invocation}");
    }
}

/// A rename of `thread_local` anywhere in the crate — module level, a function body, or a re-export
/// another module globs in — is refused, naming the new name and asking for the macro by its name.
#[test]
pub(super) fn a_renamed_thread_local_refuses_to_judge() {
    for (label, files) in [
        (
            "static-rename-module",
            vec![
                ("lib.rs", "pub mod kernel;\n"),
                (
                    "kernel.rs",
                    "use std::thread_local as tls;\ntls! { static A: u8 = 0; }\n",
                ),
            ],
        ),
        (
            "static-rename-fn-body",
            vec![
                (
                    "lib.rs",
                    "pub mod kernel;\npub fn outside() { use std::thread_local as tls; }\n",
                ),
                ("kernel.rs", "pub fn f() {}\n"),
            ],
        ),
        (
            "static-rename-reexport",
            vec![
                (
                    "lib.rs",
                    "pub mod kernel;\npub mod q { pub(crate) use std::thread_local as tlq; }\n",
                ),
                (
                    "kernel.rs",
                    "use crate::q::*;\ntlq! { static B: u8 = 0; }\n",
                ),
            ],
        ),
    ] {
        let error = statics(label, &files, "crate::kernel").unwrap_err();
        assert!(
            error.contains("renames `thread_local`") && error.contains("write `thread_local!`"),
            "{label}: {error}"
        );
    }
}

/// A rename that brings no new name into scope is not one: `as thread_local` and `as _`.
#[test]
pub(super) fn a_rename_to_thread_local_or_underscore_is_not_refused() {
    let violations = kernel_statics(
        "static-rename-harmless",
        "use std::thread_local as thread_local;\nuse std::thread_local as _;\n\
         thread_local! { static A: u8 = 0; }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("thread_local", "crate::kernel", "A", "")]
    );
}

#[test]
pub(super) fn foreign_statics_react_with_their_mutability() {
    let violations = kernel_statics(
        "static-foreign",
        "extern \"C\" {\n    static A: i32;\n    static mut B: i32;\n}\n\
         unsafe extern \"C\" {\n    pub safe static C: i32;\n    pub unsafe static mut D: i32;\n    \
         static E: i32;\n    pub safe fn not_a_static();\n}\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("foreign_static", "crate::kernel", "A", ""),
            row("foreign_static", "crate::kernel", "C", ""),
            row("foreign_static", "crate::kernel", "E", ""),
            row("foreign_static_mut", "crate::kernel", "B", ""),
            row("foreign_static_mut", "crate::kernel", "D", ""),
        ]
    );
}

/// A static is attributed to the module that declares it — an inline child, a `#[path]` child, a
/// `cfg_if!` arm — exactly once, and one in a module outside the anchor does not react.
#[test]
pub(super) fn each_static_is_attributed_once_to_its_declaring_module() {
    let violations = statics(
        "static-attribution",
        &[
            ("lib.rs", "pub mod kernel;\npub mod other;\n"),
            (
                "kernel.rs",
                "pub mod inline { pub static IN_INLINE: u8 = 0; }\n\
                 #[path = \"remapped.rs\"]\npub mod remapped;\n\
                 pub mod child;\n\
                 cfg_if::cfg_if! { if #[cfg(unix)] { static IN_ARM: u8 = 0; } else { static IN_ARM: u8 = 1; } }\n",
            ),
            ("remapped.rs", "pub static IN_REMAPPED: u8 = 0;\n"),
            ("kernel/child.rs", "pub static IN_CHILD: u8 = 0;\n"),
            ("other.rs", "pub static OUTSIDE: u8 = 0;\n"),
        ],
        "crate::kernel",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "IN_ARM", ""),
            row("static", "crate::kernel::child", "IN_CHILD", ""),
            row("static", "crate::kernel::inline", "IN_INLINE", ""),
            row("static", "crate::kernel::remapped", "IN_REMAPPED", ""),
        ]
    );
    assert!(violations.iter().all(|v| v.target() == "crate::kernel"));
}

#[test]
pub(super) fn a_boundary_anchored_at_crate_governs_the_whole_crate() {
    let violations = kernel_statics("static-crate-root", "pub static IN_KERNEL: u8 = 0;\n");
    assert_eq!(violations.unwrap().len(), 1);
    let files = lib_and_kernel("pub static IN_KERNEL: u8 = 0;\n");
    let files: Vec<(&str, &str)> = files.iter().map(|(p, c)| (*p, c.as_str())).collect();
    let violations = statics("static-crate-root-anchor", &files, "crate").unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "IN_KERNEL", ""),
            row("static", "crate::other", "OUTSIDE", ""),
        ]
    );
}

#[test]
pub(super) fn a_warn_boundary_reports_at_warn_severity() {
    let (metadata, _tree) = fixture_metadata(
        "static-warn",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "static A: u8 = 0;\n"),
        ],
    );
    let warn = StaticBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_declare_static()
        .warn()
        .because("advisory");
    let mut violations = Vec::new();
    check_static_boundary(&metadata, &warn, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warn);
}

/// Nothing here declares a static: a `'static` lifetime, a `let` binding, a `const` item.
#[test]
pub(super) fn a_static_lifetime_a_let_and_a_const_are_clean() {
    let violations = kernel_statics(
        "static-clean",
        "pub fn name() -> &'static str { let local = \"k\"; local }\npub const LIMIT: u8 = 3;\n\
         pub struct Holder<T: 'static>(T);\n",
    )
    .unwrap();
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
pub(super) fn an_interior_mutable_const_is_not_a_static() {
    let violations = kernel_statics(
        "static-interior-const",
        "use std::cell::Cell;\npub const FRESH: Cell<u8> = Cell::new(0);\n",
    )
    .unwrap();
    assert!(violations.is_empty(), "{violations:?}");
}

/// Identity is structural, so a baseline recorded before a reorder still describes the tree after it.
#[test]
pub(super) fn reordering_statics_keeps_their_identities() {
    let ids = |label: &str, kernel: &str| {
        let mut ids: Vec<String> = kernel_statics(label, kernel)
            .unwrap()
            .iter()
            .map(|v| format!("{:?}", v.id()))
            .collect();
        ids.sort();
        ids
    };
    assert_eq!(
        ids(
            "static-order-a",
            "static A: u8 = 0;\nfn f() { static B: u8 = 0; }\n"
        ),
        ids(
            "static-order-b",
            "fn f() { static B: u8 = 0; }\nstatic A: u8 = 0;\n"
        ),
    );
}

#[test]
pub(super) fn a_macro_generated_static_is_a_documented_bound() {
    let violations = kernel_statics(
        "static-macro-generated",
        "macro_rules! make_static { ($n:ident) => { static $n: u8 = 0; }; }\nmake_static!(HIDDEN);\n\
         macro_rules! wrap { ($($t:tt)*) => { thread_local! { $($t)* } }; }\nwrap! { static WRAPPED: u8 = 0; }\n",
    )
    .unwrap();
    assert!(
        violations.is_empty(),
        "a static only a macro's expansion declares is out of reach: {violations:?}"
    );
}

#[test]
pub(super) fn a_local_thread_local_macro_over_reacts_is_a_bound() {
    let violations = kernel_statics(
        "static-local-thread-local",
        "macro_rules! thread_local { ($($t:tt)*) => {}; }\nthread_local! { static NOT_REAL: u32 = 0; }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("thread_local", "crate::kernel", "NOT_REAL", "")]
    );
}

#[test]
pub(super) fn static_cfg_is_observed_as_written() {
    let violations = kernel_statics(
        "static-cfg",
        "#[cfg(test)]\nstatic TEST_ONLY: u8 = 0;\n#[cfg(any())]\nstatic NEVER: u8 = 0;\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "NEVER", ""),
            row("static", "crate::kernel", "TEST_ONLY", ""),
        ]
    );
}

#[test]
pub(super) fn nested_block_statics_share_one_identity() {
    let violations = kernel_statics(
        "static-nested-blocks",
        "pub fn f() { { static M: u8 = 0; } { static M: u8 = 1; } }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("static", "crate::kernel", "M", "f")]
    );
}

#[test]
pub(super) fn anonymous_const_statics_share_one_identity() {
    let violations = kernel_statics(
        "static-anonymous-const",
        "const _: () = { static H: u8 = 0; };\nconst _: () = { static H: u8 = 1; };\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("static", "crate::kernel", "H", "const _")]
    );
}

#[test]
pub(super) fn closure_statics_share_one_identity() {
    let violations = kernel_statics(
        "static-closures",
        "pub fn f() { let a = || { static C: u8 = 0; }; let b = || { static C: u8 = 1; }; a(); b(); }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("static", "crate::kernel", "C", "f")]
    );
}

#[test]
pub(super) fn an_unparseable_thread_local_body_refuses_to_judge() {
    for (label, body) in [
        ("static-tl-unparseable", "not a static"),
        ("static-tl-not-static", "fn not_a_static() {}"),
    ] {
        let error = kernel_statics(label, &format!("thread_local! {{ {body} }}\n")).unwrap_err();
        assert!(
            error.contains("cannot judge a `thread_local!` in module 'crate::kernel'"),
            "{label}: {error}"
        );
    }
}

#[test]
pub(super) fn a_foreign_crate_rename_of_thread_local_is_a_documented_bound() {
    let violations = kernel_statics(
        "static-foreign-rename",
        "use dep::tls;\ntls! { static FROM_DEP: u8 = 0; }\n",
    )
    .unwrap();
    assert!(
        violations.is_empty(),
        "a rename another crate made is out of reach: {violations:?}"
    );
}

/// The anchor is read through the one module-anchor reader: a non-canonical spelling is refused
/// with the canonical one suggested, and an absent module is the shared unknown-module refusal.
#[test]
pub(super) fn the_anchor_is_held_to_the_module_anchor_rule() {
    let error = kernel_statics("static-anchor-spelling", "static A: u8 = 0;\n");
    assert!(error.is_ok());
    let files = lib_and_kernel("static A: u8 = 0;\n");
    let files: Vec<(&str, &str)> = files.iter().map(|(p, c)| (*p, c.as_str())).collect();
    let misspelled = statics("static-anchor-misspelled", &files, "kernel").unwrap_err();
    assert!(
        misspelled.contains("'kernel'") && misspelled.contains("write `crate::kernel`"),
        "{misspelled}"
    );
    let absent = statics("static-anchor-absent", &files, "crate::nope").unwrap_err();
    assert_eq!(absent, unknown_module_error("crate::nope", "x"));
}

#[test]
pub(super) fn an_unnameable_owner_refuses_to_judge() {
    let error = kernel_statics(
        "static-unnameable-owner",
        "pub struct Foo;\npub const N: usize = 1;\npub trait Tr<const M: usize> { fn m(); }\n\
         impl Tr<{ N + 1 }> for Foo { fn m() { static X: u8 = 0; } }\n",
    )
    .unwrap_err();
    assert!(
        error.contains("cannot identify static X in crate::kernel") && error.contains("impl trait"),
        "{error}"
    );
}

#[test]
pub(super) fn an_undecodable_foreign_item_refuses_to_judge() {
    let error = kernel_statics(
        "static-undecodable-foreign",
        "unsafe extern \"C\" {\n    #[cfg(any())]\n    pub fn with_body() {}\n}\n",
    )
    .unwrap_err();
    assert!(
        error.contains("cannot judge a foreign item in module 'crate::kernel'")
            && error.contains("with_body")
            && error.contains("delete it"),
        "{error}"
    );
}

/// A `mod` written in a function body is not a module of the crate's graph, so a static inside it
/// belongs to the module holding the function, owned by the function.
#[test]
pub(super) fn a_static_in_a_body_module_is_owned_by_its_fn() {
    let violations = kernel_statics(
        "static-body-mod",
        "pub fn f() { mod m { pub static X: u8 = 0; } }\n",
    )
    .unwrap();
    assert_eq!(
        rows(&violations),
        [row("static", "crate::kernel", "X", "f")]
    );
}

/// An associated const's initializer is owned as `const` of its qualified name, in an impl and in a
/// trait default alike.
#[test]
pub(super) fn a_static_in_an_associated_const_is_owned_by_it() {
    let violations = kernel_statics(
        "static-assoc-const",
        "pub struct Foo;\nimpl Foo { pub const NAME: u8 = { static IN_IMPL: u8 = 0; 1 }; }\n\
         pub trait Tr { const N: u8 = { static IN_TRAIT: u8 = 0; 1 }; }\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "IN_IMPL", "const Foo::NAME"),
            row("static", "crate::kernel", "IN_TRAIT", "const Tr::N"),
        ]
    );
}

/// A static nested in a `thread_local!` static's initializer is owned by that static.
#[test]
pub(super) fn a_static_in_a_thread_local_initializer_is_owned_by_it() {
    let violations = kernel_statics(
        "static-tl-initializer",
        "thread_local! { static T: u8 = { static INNER: u8 = 0; 1 }; }\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "INNER", "static T"),
            row("thread_local", "crate::kernel", "T", ""),
        ]
    );
}

/// A rename is looked for in every file of the governed crate, so a file outside the anchor that does
/// not parse leaves the rename undecided and the boundary is refused, never judged over the rest.
#[test]
pub(super) fn an_unparseable_file_outside_the_anchor_refuses_to_judge() {
    let error = statics(
        "static-unparseable-outside",
        &[
            ("lib.rs", "pub mod kernel;\npub mod other;\n"),
            ("kernel.rs", "pub static IN_KERNEL: u8 = 0;\n"),
            ("other.rs", "pub fn broken( {\n"),
        ],
        "crate::kernel",
    )
    .unwrap_err();
    assert!(
        error.contains("other.rs"),
        "the refusal names the unparseable file: {error}"
    );
}

#[test]
pub(super) fn semantic_error_in_ungoverned_module_does_not_fail_governed_static_scan() {
    let violations = statics(
        "static-semantic-err-outside",
        &[
            ("lib.rs", "pub mod kernel;\npub mod other;\n"),
            ("kernel.rs", "pub static IN_KERNEL: u8 = 0;\n"),
            ("other.rs", "thread_local! { not a static }\n"),
        ],
        "crate::kernel",
    )
    .unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].fact().shape(), "static");
    let name = violations[0]
        .fact()
        .fields()
        .find(|(k, _)| *k == "name")
        .map(|(_, v)| v)
        .unwrap();
    assert_eq!(name, "IN_KERNEL");
}

/// A declaration outside the anchor that cannot be named is ungoverned, but the module holding it is
/// still read for a rename: a rename written after it in the same module is refused, never dropped
/// with it.
#[test]
pub(super) fn a_rename_beside_an_unnameable_declaration_outside_the_anchor_is_still_refused() {
    let error = statics(
        "static-rename-beside-unnameable",
        &[
            ("lib.rs", "pub mod kernel;\npub mod other;"),
            ("kernel.rs", "thread_local! { static A: u8 = 0; }"),
            (
                "other.rs",
                "thread_local! { not a static }\nuse std::thread_local as tls;",
            ),
        ],
        "crate::kernel",
    )
    .unwrap_err();
    assert!(
        error.contains("tls")
            && error.contains("renames `thread_local`")
            && error.contains("write `thread_local!`"),
        "the rename is refused, not the ungoverned declaration beside it: {error}"
    );
}

#[test]
pub(super) fn static_nested_in_thread_local_type_position_reacts() {
    let violations = kernel_statics(
        "static-in-tl-type",
        "thread_local! { static FOO: [u8; { static NESTED: u8 = 42; 1 }] = [0]; }\n",
    )
    .unwrap();
    let mut observed = rows(&violations);
    observed.sort();
    assert_eq!(
        observed,
        [
            row("static", "crate::kernel", "NESTED", "static FOO"),
            row("thread_local", "crate::kernel", "FOO", ""),
        ]
    );
}

#[test]
pub(super) fn thread_local_missing_semicolon_between_statics_is_refused_exit_2() {
    let error = kernel_statics(
        "static-tl-missing-semi",
        "thread_local! { static A: u8 = 0 static B: u8 = 0; }\n",
    )
    .unwrap_err();
    assert!(
        error.contains("cannot judge a `thread_local!` in module 'crate::kernel'"),
        "error should refuse unparseable thread_local body: {error}"
    );
}
