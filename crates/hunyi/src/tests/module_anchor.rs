use super::super::*;
use super::helpers::*;
use crate::anchor::canonical_module_anchor;
use crate::errors::unknown_location_error;
// --- module anchors: one canonical spelling, naming a module that exists -------------------

/// Every spelling a module anchor is judged on, with the canonical form it is accepted as or the
/// suggestion its refusal must carry. The table is the specification of the spelling rule, and each
/// capability test below reads it rather than keeping a list of its own.
pub(super) const ANCHOR_SPELLINGS: &[(&str, Spelling)] = &[
    ("crate", Spelling::Canonical("crate")),
    ("crate::kernel", Spelling::Canonical("crate::kernel")),
    ("crate::r#kernel", Spelling::Canonical("crate::kernel")),
    ("crate::type", Spelling::Canonical("crate::type")),
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
pub(super) fn a_module_anchor_is_accepted_only_in_its_canonical_spelling() {
    for (written, expected) in ANCHOR_SPELLINGS {
        let answer = canonical_module_anchor(written, "x");
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

/// A fixture holding one finding for every module-anchored capability under `crate::kernel`, and one
/// misplaced `unsafe` site and trait impl under `crate::other` for the two location lists.
fn anchor_fixture(name: &str) -> (Value, TempSrcTree) {
    fixture_metadata(
        name,
        &[
            ("lib.rs", "pub mod kernel;\npub mod other;\nmod r#type;\n"),
            ("type.rs", ""),
            (
                "kernel.rs",
                "pub mod inner {}\n\
                 pub fn run() {}\n\
                 pub fn leak() -> crate::other::Secret { crate::other::Secret }\n\
                 pub fn d() -> Box<dyn crate::other::Tr> { Box::new(K) }\n\
                 pub fn i() -> impl crate::other::Tr { K }\n\
                 pub async fn a() {}\n\
                 pub struct K;\n\
                 impl crate::other::Tr for K {}\n\
                 impl crate::other::Marker for K {}\n\
                 pub fn u() { unsafe {} }\n\
                 static STATE: u8 = 0;\n",
            ),
            (
                "other.rs",
                "pub struct Secret;\npub trait Tr {}\npub trait Marker {}\npub struct O;\n\
                 impl Tr for O {}\npub fn u() { unsafe {} }\n",
            ),
        ],
    )
}

type Checker = fn(&Value, &str, &mut Vec<Violation>) -> Result<(), String>;

/// Each capability that takes a module anchor or an allowed-location list, driven through its real
/// `check_*_boundary` with `anchor` in the module position.
fn anchored_capabilities() -> Vec<(&'static str, Checker)> {
    vec![
        ("visibility", |m, a, v| {
            let b = VisibilityBoundary::in_crate("x")
                .module(a)
                .max_visibility(VisibilityCeiling::Module)
                .because("r");
            check_visibility_boundary(m, &b, v)
        }),
        ("reexport-only", |m, a, v| {
            let b = ReexportOnlyBoundary::in_crate("x")
                .module(a)
                .must_declare_only_reexports()
                .because("r");
            check_reexport_only_boundary(m, &b, v)
        }),
        ("signature", |m, a, v| {
            let b = SignatureBoundary::in_crate("x")
                .module(a)
                .must_not_expose("crate::other::Secret")
                .because("r");
            crate::exposure::check_boundary(m, &b, v)
        }),
        ("dyn-trait", |m, a, v| {
            let b = DynTraitBoundary::in_crate("x")
                .module(a)
                .must_not_expose_dyn()
                .because("r");
            check_dyn_trait_boundary(m, &b, v)
        }),
        ("impl-trait", |m, a, v| {
            let b = ImplTraitBoundary::in_crate("x")
                .module(a)
                .must_not_expose_impl_trait()
                .because("r");
            check_impl_trait_boundary(m, &b, v)
        }),
        ("async-exposure", |m, a, v| {
            let b = AsyncExposureBoundary::in_crate("x")
                .module(a)
                .must_not_expose_async_fn()
                .including_submodules()
                .because("r");
            check_async_exposure_boundary(m, &b, v)
        }),
        ("forbidden-marker", |m, a, v| {
            let b = ForbiddenMarkerBoundary::in_crate("x")
                .module(a)
                .must_not_acquire("crate::other::Marker")
                .because("r");
            check_forbidden_marker_boundary(m, &b, v)
        }),
        ("static-item", |m, a, v| {
            let b = StaticBoundary::in_crate("x")
                .module(a)
                .must_not_declare_static()
                .because("r");
            crate::static_item::check_static_boundary(m, &b, v)
        }),
        ("unsafe only_under", |m, a, v| {
            let b = UnsafeBoundary::in_crate("x").only_under([a]).because("r");
            check_unsafe_boundary(m, &b, v)
        }),
        ("trait-impl only_implemented_in", |m, a, v| {
            let b = TraitImplBoundary::in_crate("x")
                .trait_("crate::other::Tr")
                .only_implemented_in(a)
                .because("r");
            check_trait_impl_boundary(m, &b, v)
        }),
    ]
}

/// Every refused spelling of the table is a constitution error in every capability that takes a
/// module path, naming what was written and suggesting the canonical spelling. The crate root is
/// left out: `crate` keeps each capability's own answer, which for `only_under` is its own refusal.
/// Each capability and spelling is judged before the assertion, so a failure lists all of them.
#[test]
pub(super) fn every_anchored_capability_refuses_a_non_canonical_spelling() {
    let (metadata, _fixture) = anchor_fixture("anchor-refused");
    let mut wrong = Vec::new();
    for (capability, check) in anchored_capabilities() {
        for (written, expected) in ANCHOR_SPELLINGS {
            let Spelling::Refused(suggestion) = expected else {
                continue;
            };
            match check(&metadata, written, &mut Vec::new()) {
                Ok(()) => wrong.push(format!("{capability} {written:?}: judged, not refused")),
                Err(error)
                    if !(error.contains(&format!("'{written}'")) && error.contains(suggestion)) =>
                {
                    wrong.push(format!(
                        "{capability} {written:?}: refusal lacks the text or {suggestion:?}: {error}"
                    ))
                }
                Err(_) => {}
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A bare keyword segment reaches a module declared with its raw spelling.
#[test]
pub(super) fn a_bare_keyword_anchor_reaches_a_raw_declared_module() {
    let (metadata, _fixture) = anchor_fixture("anchor-keyword");
    for (capability, check) in anchored_capabilities() {
        check(&metadata, "crate::type", &mut Vec::new())
            .unwrap_or_else(|error| panic!("{capability}: {error}"));
    }
}

/// A canonical anchor and its raw-identifier spelling are one module, so each produces the same
/// violations with the same identities, and the target is recorded without the raw prefix.
#[test]
pub(super) fn a_raw_identifier_anchor_is_the_same_identity_as_its_plain_spelling() {
    let (metadata, _fixture) = anchor_fixture("anchor-raw");
    let mut wrong = Vec::new();
    for (capability, check) in anchored_capabilities() {
        let mut plain = Vec::new();
        check(&metadata, "crate::kernel", &mut plain).unwrap();
        let mut raw = Vec::new();
        check(&metadata, "crate::r#kernel", &mut raw).unwrap();
        assert!(!plain.is_empty(), "{capability}: the fixture reacts");
        let ids = |vs: &[Violation]| vs.iter().map(Violation::id).collect::<Vec<_>>();
        if ids(&plain) != ids(&raw) {
            let targets = |vs: &[Violation]| {
                let mut t: Vec<String> = vs.iter().map(|v| v.target().to_string()).collect();
                t.dedup();
                t
            };
            wrong.push(format!(
                "{capability}: identities differ, targets {:?} vs {:?}",
                targets(&plain),
                targets(&raw)
            ));
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A canonical spelling naming no module is refused in every capability: an anchor governs a real
/// module, and a location that names none can never match.
#[test]
pub(super) fn every_anchored_capability_refuses_a_module_that_does_not_exist() {
    let (metadata, _fixture) = anchor_fixture("anchor-absent");
    let mut wrong = Vec::new();
    for (capability, check) in anchored_capabilities() {
        let expected = if capability.contains(' ') {
            unknown_location_error("crate::nope", "x")
        } else {
            unknown_module_error("crate::nope", "x")
        };
        let mut violations = Vec::new();
        match check(&metadata, "crate::nope", &mut violations) {
            Err(error) if error == expected => {}
            Err(error) => wrong.push(format!("{capability}: refused as {error}")),
            Ok(()) => wrong.push(format!(
                "{capability}: judged, not refused ({} violations)",
                violations.len()
            )),
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// A module that exists in only one of a package's compilation units is a real module: forbidden-
/// marker's anchor and both location lists are refused only when no unit declares it.
#[test]
pub(super) fn a_module_present_in_one_compilation_unit_is_not_absent() {
    let tree = TempSrcTree::new("anchor-units");
    tree.write_all(&[
        ("lib.rs", "pub mod ffi;\npub trait Tr {}\n"),
        (
            "ffi.rs",
            "pub struct K;\nimpl crate::Tr for K {}\nimpl Clone for K { fn clone(&self) -> Self { K } }\n\
             pub fn u() { unsafe {} }\n",
        ),
        ("bin/tool.rs", "fn main() {}\n"),
    ]);
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "dependencies": [],
            "targets": [
                { "kind": ["lib"], "src_path": tree.root().to_string_lossy().into_owned() },
                { "kind": ["bin"], "src_path": tree.src().join("bin/tool.rs").to_string_lossy().into_owned() },
            ],
        }],
    });
    let marker = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate::ffi")
        .must_not_acquire("Clone")
        .because("r");
    let mut violations = Vec::new();
    check_forbidden_marker_boundary(&metadata, &marker, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "the library's marker impl reacts");

    let confined = UnsafeBoundary::in_crate("x")
        .only_under(["crate::ffi"])
        .because("r");
    let mut violations = Vec::new();
    check_unsafe_boundary(&metadata, &confined, &mut violations).unwrap();
    assert!(violations.is_empty(), "{violations:?}");

    let located = TraitImplBoundary::in_crate("x")
        .trait_("crate::Tr")
        .only_implemented_in("crate::ffi")
        .because("r");
    let mut violations = Vec::new();
    check_trait_impl_boundary(&metadata, &located, &mut violations).unwrap();
    assert!(violations.is_empty(), "{violations:?}");
}

/// Two compilation units that both declare `crate::target`, so a deferral is the only way either
/// row below could come back `Ok`.
fn two_units_declaring_target(tree: &TempSrcTree) -> Vec<crate::file_scope::CompilationUnit> {
    tree.write_all(&[
        ("lib.rs", "pub mod target;\n"),
        ("target.rs", "\n"),
        ("bin/tool.rs", "pub mod target;\nfn main() {}\n"),
    ]);
    vec![
        (tree.root(), tree.src().to_path_buf(), "lib".to_string()),
        (
            tree.src().join("bin/tool.rs"),
            tree.src().to_path_buf(),
            "bin".to_string(),
        ),
    ]
}

/// An unresolvable-module error pointing to a DIFFERENT module is an unexpected resolution failure,
/// not the absence of the boundary's module anchor, and must not be deferred.
#[test]
pub(super) fn unresolvable_error_pointing_to_different_module_anchor_is_not_deferred() {
    let tree = TempSrcTree::new("diff-module-anchor-multi-unit");
    let units = two_units_declaring_target(&tree);
    let anchor = crate::file_scope::UnitAnchor::Module {
        module: "crate::target",
        crate_package: "x",
    };
    let res = crate::file_scope::over_each_unit(&units, anchor, |_root_file, _src_dir, unit| {
        if unit == "lib" {
            Err(crate::errors::ResolveError::UnresolvableModule(
                "crate::different".to_string(),
                "x".to_string(),
            ))
        } else {
            Ok(())
        }
    });
    assert!(
        res.is_err(),
        "unresolvable error for a different module must not be deferred: {res:?}"
    );
}

/// An unknown-trait error pointing to a DIFFERENT trait is an unexpected resolution failure, not the
/// absence of the boundary's trait anchor, and must not be deferred.
#[test]
pub(super) fn unknown_trait_error_pointing_to_different_trait_anchor_is_not_deferred() {
    let tree = TempSrcTree::new("diff-trait-anchor-multi-unit");
    let units = two_units_declaring_target(&tree);
    let anchor = crate::file_scope::UnitAnchor::Trait {
        trait_path: "crate::TargetTr",
        crate_package: "x",
    };
    let res = crate::file_scope::over_each_unit(&units, anchor, |_root_file, _src_dir, unit| {
        if unit == "lib" {
            Err(crate::errors::ResolveError::UnknownTrait(
                "crate::DifferentTr".to_string(),
                "x".to_string(),
            ))
        } else {
            Ok(())
        }
    });
    assert!(
        res.is_err(),
        "unknown trait error for a different trait must not be deferred: {res:?}"
    );
}
