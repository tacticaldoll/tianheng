use super::helpers::*;
// --- must_only_be_imported_by (the inbound closed allowlist) ----------------

pub(super) fn only_importers(allowed: &[&str]) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::internal")
        .must_only_be_imported_by(allowed.iter().copied())
        .because("internal is imported only through its facade")
}

#[test]
pub(super) fn must_only_be_imported_by_flags_an_importer_outside_the_allowlist() {
    let (result, violations) = run_module_check(
        "only-basic",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\npub mod consumer;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "use crate::internal::Secret;\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    // facade is allowlisted (clean); consumer is not (violates).
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::internal");
    assert_eq!(violations[0].finding, "crate::consumer");
    assert_eq!(violations[0].rule, "module may only be imported by");
}

#[test]
pub(super) fn must_only_be_imported_by_authorizes_an_allowed_inline_importer() {
    // `crate::facade` is an INLINE module and IS allow-listed. Its import is attributed
    // to `crate::facade` (not the file's `crate`), so it is correctly authorized. Testing the file
    // module `crate` against the allowlist would wrongly flag the allowed inline importer.
    let (result, violations) = run_module_check(
        "only-inline-allowed",
        &[
            (
                "lib.rs",
                "pub mod internal;\nmod facade { use crate::internal::Secret; }\n",
            ),
            ("internal.rs", "// protected\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an allow-listed inline importer must not be flagged: {violations:?}"
    );
}

#[test]
pub(super) fn must_only_be_imported_by_flags_a_disallowed_inline_importer_by_its_true_identity() {
    // A disallowed INLINE importer is flagged with its true identity `crate::rogue` (not the file's
    // `crate`), so the structured fact — and thus `(target, rule key, fact)` identity — is
    // correct rather than shifted onto the file module.
    let (result, violations) = run_module_check(
        "only-inline-disallowed",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\nmod rogue { use crate::internal::Secret; }\n",
            ),
            ("internal.rs", "// protected\n"),
            (
                "facade.rs",
                "// the allow-listed importer declares no import here\n",
            ),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::rogue");
}

#[test]
pub(super) fn must_only_be_imported_by_admits_the_allowlisted_importer_subtree() {
    let (result, violations) = run_module_check(
        "only-subtree",
        &[
            ("lib.rs", "pub mod internal;\npub mod facade;\n"),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "pub mod v1;\n"),
            ("facade/v1.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "crate::facade::v1 is beneath the allowlisted importer: {violations:?}"
    );
}

#[test]
pub(super) fn must_only_be_imported_by_does_not_admit_a_prefix_colliding_sibling() {
    let (result, violations) = run_module_check(
        "only-prefix",
        &[
            ("lib.rs", "pub mod internal;\npub mod facadex;\n"),
            ("internal.rs", "// protected\n"),
            ("facadex.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding, "crate::facadex",
        "a sibling of the allowlisted importer is not admitted"
    );
}

#[test]
pub(super) fn must_only_be_imported_by_never_flags_the_protected_subtree() {
    let (result, violations) = run_module_check(
        "only-own-subtree",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "pub mod deep;\n"),
            ("internal/deep.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a module within the protected subtree is not an inbound importer: {violations:?}"
    );
}

#[test]
pub(super) fn must_only_be_imported_by_empty_allowlist_forbids_every_outside_importer() {
    let (result, violations) = run_module_check(
        "only-empty",
        &[
            ("lib.rs", "pub mod internal;\npub mod consumer;\n"),
            ("internal.rs", "// protected\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&[]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::consumer");
}

#[test]
pub(super) fn must_only_be_imported_by_admits_multiple_allowlisted_importers() {
    let (result, violations) = run_module_check(
        "only-multiple",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\npub mod api;\npub mod consumer;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "use crate::internal::Secret;\n"),
            ("api.rs", "use crate::internal::Secret;\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade", "crate::api"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::consumer");
}

#[test]
pub(super) fn must_only_be_imported_by_ignores_external_imports() {
    let (result, violations) = run_module_check(
        "only-external",
        &[
            ("lib.rs", "pub mod internal;\npub mod consumer;\n"),
            ("internal.rs", "// protected\n"),
            ("consumer.rs", "use serde::Deserialize;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an external import is out of scope: {violations:?}"
    );
}

#[test]
pub(super) fn must_only_be_imported_by_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "only-m-crate",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_only_be_imported_by(["crate::facade"])
            .because("the crate root cannot be protected this way"),
    );
    let err = result.expect_err("protecting `crate` must be a constitution error");
    assert_eq!(err, must_only_be_imported_by_on_crate_error("x"));
}

#[test]
pub(super) fn must_only_be_imported_by_rule_text_and_json_params() {
    // Projection surface: distinct label/text and the surface-qualified `only_importers` key.
    let rule = ModuleRule::MustOnlyBeImportedBy {
        allowed: vec!["crate::facade".to_string()],
    };
    assert_eq!(rule.label(), "module may only be imported by");
    assert_eq!(rule.polarity(), Polarity::AllowlistGap);
    assert_eq!(rule.text(), "may only be imported by: crate::facade");
    assert_eq!(
        rule.json_params(),
        vec![("only_importers", serde_json::json!(["crate::facade"]))]
    );
    let empty = ModuleRule::MustOnlyBeImportedBy { allowed: vec![] };
    assert_eq!(empty.text(), "may only be imported by nothing");
}
