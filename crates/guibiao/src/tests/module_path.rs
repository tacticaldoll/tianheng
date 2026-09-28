use super::helpers::*;
use crate::errors::{
    non_canonical_module_path_error, unknown_allowed_module_error, unknown_forbidden_module_error,
};
use crate::module_scan::canonical_module_spelling;
// --- module paths: one canonical spelling, naming a module that exists -----------------------

/// Every spelling a module path is judged on, with the canonical form it is accepted as or the
/// suggestion its refusal must carry. The table is the specification of the spelling rule, and each
/// role test below reads it rather than keeping a list of its own.
pub(super) const MODULE_PATH_SPELLINGS: &[(&str, Spelling)] = &[
    ("crate", Spelling::Canonical("crate")),
    ("crate::kernel", Spelling::Canonical("crate::kernel")),
    ("crate::r#kernel", Spelling::Canonical("crate::kernel")),
    (
        "crate::kernel::r#inner",
        Spelling::Canonical("crate::kernel::inner"),
    ),
    (
        "crate::kernel::",
        Spelling::Refused("write `crate::kernel`"),
    ),
    ("", Spelling::Refused("write `crate` for the crate root")),
    ("kernel", Spelling::Refused("write `crate::kernel`")),
    (
        "::crate::kernel",
        Spelling::Refused("write `crate::kernel`"),
    ),
    (
        "crate::::kernel",
        Spelling::Refused("write `crate::kernel`"),
    ),
    ("self::kernel", Spelling::Refused("starting `crate::`")),
    ("super::kernel", Spelling::Refused("starting `crate::`")),
    ("crate:: kernel", Spelling::Refused("write `crate::kernel`")),
    ("crate::kernel ", Spelling::Refused("write `crate::kernel`")),
    ("crate ::kernel", Spelling::Refused("write `crate::kernel`")),
    (
        "r#crate::kernel",
        Spelling::Refused("write `crate::kernel`"),
    ),
    ("crate::kernel::*", Spelling::Refused("starting `crate::`")),
];

#[derive(Clone, Copy, Debug)]
pub(super) enum Spelling {
    /// Accepted, and recorded as this form.
    Canonical(&'static str),
    /// Refused, with a message carrying this suggestion.
    Refused(&'static str),
}

#[test]
pub(super) fn a_module_path_is_accepted_only_in_its_canonical_spelling() {
    for (written, expected) in MODULE_PATH_SPELLINGS {
        let answer = canonical_module_spelling(written).map_err(|suggestion| {
            non_canonical_module_path_error(written, "x", suggestion.as_deref())
        });
        match expected {
            Spelling::Canonical(canonical) => {
                assert_eq!(answer.as_deref(), Ok(*canonical), "{written:?}")
            }
            Spelling::Refused(suggestion) => {
                let error = answer.expect_err(written);
                assert!(
                    error.contains(&format!("'{written}'")) && error.contains(suggestion),
                    "the refusal of {written:?} names it and suggests {suggestion:?}: {error}"
                );
            }
        }
    }
}

/// A segment is an identifier by the lexer's byte test: not starting with a digit, and behind `r#`
/// not one of the five names a raw identifier cannot spell. A non-ASCII segment passes this layer
/// whatever character it is, and is left to the existence check.
#[test]
pub(super) fn a_module_path_segment_is_an_identifier_by_the_lexers_byte_test() {
    for refused in [
        "crate::1kernel",
        "crate::r#",
        "crate::r#crate",
        "crate::r#self",
        "crate::r#super",
        "crate::r#Self",
        "crate::r#_",
        "crate::r#r#kernel",
        "crate::ker-nel",
        "crate::kernel\t",
    ] {
        assert!(canonical_module_spelling(refused).is_err(), "{refused:?}");
    }
    for accepted in [
        "crate::_",
        "crate::_kernel",
        "crate::kernel1",
        "crate::type",
        "crate::k\u{e9}rnel",
        "crate::\u{a0}kernel",
    ] {
        assert!(canonical_module_spelling(accepted).is_ok(), "{accepted:?}");
    }
}

/// A crate in which `crate::other` imports `crate::kernel` and `crate::kernel::inner`, each of which
/// imports `crate::other` and the external crate `extdep`, and calls `std::process::Command`.
fn module_path_fixture(label: &str) -> (TempWorkspace, Value) {
    let ws = TempWorkspace::new(label);
    ws.write("lib.rs", "pub mod kernel;\npub mod other;\n");
    ws.write(
        "kernel.rs",
        "pub mod inner;\npub struct K;\nuse crate::other::O;\nuse extdep::E;\n\
         pub fn f() { std::process::Command::new(\"x\"); }\n",
    );
    ws.write(
        "kernel/inner.rs",
        "pub struct I;\nuse crate::other::O;\nuse extdep::E;\n\
         pub fn h() { std::process::Command::new(\"z\"); }\n",
    );
    ws.write(
        "other.rs",
        "pub struct O;\nuse crate::kernel::K;\nuse crate::kernel::inner::I;\n\
         pub fn g() { std::process::Command::new(\"y\"); }\n",
    );
    let metadata = ws.metadata("x");
    (ws, metadata)
}

type Role = fn(&str) -> ModuleBoundary;

/// Each builder that takes a module path, with `path` in the module position.
fn module_path_roles() -> Vec<(&'static str, Role)> {
    vec![
        ("module", |p| {
            ModuleBoundary::in_crate("x")
                .module(p)
                .must_not_import("crate::other")
                .because("r")
        }),
        ("confine_external_crate's module", |p| {
            ModuleBoundary::in_crate("x")
                .module(p)
                .confine_external_crate("extdep")
                .because("r")
        }),
        ("must_not_call_inline's module", |p| {
            ModuleBoundary::in_crate("x")
                .module(p)
                .must_not_call_inline("std::process")
                .because("r")
        }),
        ("confine_inline_call's module", |p| {
            ModuleBoundary::in_crate("x")
                .module(p)
                .confine_inline_call("std::process::Command")
                .because("r")
        }),
        ("must_not_import", |p| {
            ModuleBoundary::in_crate("x")
                .module("crate::other")
                .must_not_import(p)
                .because("r")
        }),
        ("must_not_be_imported_by", |p| {
            ModuleBoundary::in_crate("x")
                .module("crate::other")
                .must_not_be_imported_by(p)
                .because("r")
        }),
        ("restrict_imports_to", |p| {
            ModuleBoundary::in_crate("x")
                .module("crate::other")
                .restrict_imports_to([p])
                .because("r")
        }),
        ("must_only_be_imported_by", |p| {
            ModuleBoundary::in_crate("x")
                .module("crate::other")
                .must_only_be_imported_by([p])
                .because("r")
        }),
        ("restrict_imports_to, second entry", |p| {
            ModuleBoundary::in_crate("x")
                .module("crate::other")
                .restrict_imports_to(["crate::kernel", p])
                .because("r")
        }),
    ]
}

/// Every refused spelling of the table is a constitution error in every role that takes a module
/// path, naming what was written and suggesting the canonical spelling — including a later entry of
/// an allowlist, behind one that is canonical. Each role and spelling is judged before the assertion,
/// so a failure lists all of them.
#[test]
pub(super) fn every_module_path_role_refuses_a_non_canonical_spelling() {
    let (_ws, metadata) = module_path_fixture("module-path-refused");
    let mut wrong = Vec::new();
    for (role, boundary) in module_path_roles() {
        for (written, expected) in MODULE_PATH_SPELLINGS {
            let Spelling::Refused(suggestion) = expected else {
                continue;
            };
            let mut violations = Vec::new();
            match check_module_boundary(&metadata, &boundary(written), &mut violations) {
                Ok(()) => wrong.push(format!(
                    "{role} {written:?}: judged, not refused ({} violations)",
                    violations.len()
                )),
                Err(error)
                    if !(error.contains(&format!("'{written}'")) && error.contains(suggestion)) =>
                {
                    wrong.push(format!(
                        "{role} {written:?}: refusal lacks the text or {suggestion:?}: {error}"
                    ))
                }
                Err(_) => {}
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A forbidden module written without its `crate::` root names the module the fixture really
/// imports, so judging it as written would report a clean crate over a forbidden edge. It is refused.
#[test]
pub(super) fn a_forbidden_module_written_without_its_root_is_refused_not_judged_clean() {
    let (_ws, metadata) = module_path_fixture("module-path-bare-forbidden");
    for boundary in [
        ModuleBoundary::in_crate("x")
            .module("crate::other")
            .must_not_import("kernel")
            .because("other must not reach the kernel"),
        ModuleBoundary::in_crate("x")
            .module("crate::other")
            .must_not_be_imported_by("kernel")
            .because("the kernel must not reach other"),
    ] {
        let mut violations = Vec::new();
        let result = check_module_boundary(&metadata, &boundary, &mut violations);
        assert_eq!(
            result,
            Err(non_canonical_module_path_error(
                "kernel",
                "x",
                Some("crate::kernel")
            )),
            "{:?}: {violations:?}",
            boundary.rule()
        );
    }
}

/// A canonical spelling naming no module is refused in every role: a governed module must be one the
/// crate has, a forbidden module that is absent can never be imported, and an allowlist entry that is
/// absent can never match.
#[test]
pub(super) fn every_module_path_role_refuses_a_module_that_does_not_exist() {
    let (_ws, metadata) = module_path_fixture("module-path-absent");
    let mut wrong = Vec::new();
    for (role, boundary) in module_path_roles() {
        let expected = match role {
            "must_not_import" | "must_not_be_imported_by" => {
                unknown_forbidden_module_error("crate::nope", "x", role)
            }
            "restrict_imports_to" | "restrict_imports_to, second entry" => {
                unknown_allowed_module_error("crate::nope", "x", "restrict_imports_to")
            }
            "must_only_be_imported_by" => unknown_allowed_module_error("crate::nope", "x", role),
            _ => unknown_module_error("crate::nope", "x"),
        };
        let mut violations = Vec::new();
        match check_module_boundary(&metadata, &boundary("crate::nope"), &mut violations) {
            Err(error) if error == expected => {}
            Err(error) => wrong.push(format!("{role}: refused as {error}")),
            Ok(()) => wrong.push(format!(
                "{role}: judged, not refused ({} violations)",
                violations.len()
            )),
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// An allowlist naming the module the governed one really imports or is imported by permits those
/// edges, so the boundary is clean. Kept for the contract rather than the change: an engine that checks
/// no spelling passes it too, and an existence check that refused a real module would not.
#[test]
pub(super) fn an_allowlist_naming_a_real_module_permits_its_edges() {
    let (_ws, metadata) = module_path_fixture("module-path-allowlist");
    for boundary in [
        ModuleBoundary::in_crate("x")
            .module("crate::other")
            .restrict_imports_to(["crate::kernel"])
            .because("r"),
        ModuleBoundary::in_crate("x")
            .module("crate::other")
            .must_only_be_imported_by(["crate::r#kernel"])
            .because("r"),
    ] {
        let mut violations = Vec::new();
        check_module_boundary(&metadata, &boundary, &mut violations).unwrap();
        assert!(
            violations.is_empty(),
            "{:?}: {violations:?}",
            boundary.rule()
        );
    }
}

/// A named module and its raw-identifier spelling are one module, so each role produces the same
/// violations with the same identities. Kept for the contract rather than the change: the raw fold
/// predates the spelling check, which must not undo it.
#[test]
pub(super) fn a_raw_identifier_module_path_is_the_same_identity_as_its_plain_spelling() {
    let (_ws, metadata) = module_path_fixture("module-path-raw");
    let mut wrong = Vec::new();
    for (role, boundary) in module_path_roles() {
        let ids = |written: &str| {
            let mut violations = Vec::new();
            check_module_boundary(&metadata, &boundary(written), &mut violations).unwrap();
            violations.iter().map(Violation::id).collect::<Vec<_>>()
        };
        let plain = ids("crate::kernel::inner");
        if plain != ids("crate::r#kernel::r#inner") {
            wrong.push(format!("{role}: identities differ"));
        }
        if role.starts_with("must_not") && plain.is_empty() {
            wrong.push(format!("{role}: the fixture does not react"));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A named module present in only one of a package's compilation units is a real module, and one
/// declared inline is declared: neither is refused as absent. The precision half of the existence
/// check: it bites an existence check read from one root's graph, not the absence of the check.
#[test]
pub(super) fn a_named_module_present_in_one_compilation_unit_or_inline_is_not_absent() {
    let ws = TempWorkspace::new("module-path-units");
    ws.write("lib.rs", "pub mod kernel;\npub mod shared;\n");
    ws.write("kernel.rs", "pub mod detail { pub struct D; }\n");
    ws.write("shared.rs", "use crate::kernel::detail::D;\n");
    ws.write("bin/tool.rs", "mod tooling;\nmod shared;\nfn main() {}\n");
    ws.write("bin/tooling.rs", "");
    ws.write("bin/shared.rs", "use crate::tooling;\n");
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "targets": [
                { "kind": ["lib"], "src_path": ws.src().join("lib.rs").to_string_lossy() },
                { "kind": ["bin"], "src_path": ws.src().join("bin/tool.rs").to_string_lossy() },
            ],
        }],
    });
    let bin_only = ModuleBoundary::in_crate("x")
        .module("crate::shared")
        .must_not_import("crate::tooling")
        .because("shared must not reach the tooling");
    let mut violations = Vec::new();
    check_module_boundary(&metadata, &bin_only, &mut violations)
        .expect("a module the binary declares is a real module");
    assert_eq!(
        violations.len(),
        1,
        "the binary's import of its own module reacts: {violations:?}"
    );

    let inline = ModuleBoundary::in_crate("x")
        .module("crate::shared")
        .must_not_import("crate::kernel::detail")
        .because("shared must not reach the kernel's detail");
    let mut violations = Vec::new();
    check_module_boundary(&metadata, &inline, &mut violations)
        .expect("an inline module is declared via `mod`");
    assert_eq!(violations.len(), 1, "{violations:?}");
}
