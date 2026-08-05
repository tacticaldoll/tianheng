use super::helpers::*;
// --- external-crate confinement (`confine_external_crate`) ------------------------------

pub(super) fn confine(governed: &str, crate_name: &str) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module(governed)
        .confine_external_crate(crate_name)
        .because("the platform vocabulary stays behind the ffi module")
}

/// An import of the confined crate inside a `#[path]`-remapped module is observed.
///
/// Kept for the CONTRACT rather than for a change: the behaviour is already correct, and this pins it
/// so the spec's resolution sentence cannot drift away from it again. That sentence listed
/// `#[path]`-remapped modules among the scanner's out-of-scope bounds long after the scanner began
/// following such a remap to its target, which would have let a reader dismiss a real escape here as
/// governed policy.
#[test]
pub(super) fn confine_observes_an_import_inside_a_path_remapped_module() {
    let (result, violations) = run_module_check(
        "confine-remapped",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "#[path = \"far.rs\"]\npub mod inner;\n"),
            ("far.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "libc");
    assert!(
        violations[0].finding.contains("crate::service::inner"),
        "the finding must name the remapped module: {:?}",
        violations[0].finding
    );
}

#[test]
pub(super) fn confine_flags_an_external_import_outside_the_subtree() {
    // The confined crate is the target; the offending importer module is the finding.
    let (result, violations) = run_module_check(
        "confine-outside",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "libc");
    assert_eq!(violations[0].finding, "crate::service");
    let file = violations[0].file.as_deref().expect("carries its file");
    assert!(file.ends_with("service.rs"), "names the offender: {file}");
}

#[test]
pub(super) fn confine_is_clean_within_the_subtree() {
    let (result, violations) = run_module_check(
        "confine-within",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "within the subtree is clean: {violations:?}"
    );
}

#[test]
pub(super) fn confine_allows_beneath_the_subtree() {
    let (result, violations) = run_module_check(
        "confine-beneath",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "pub mod raw;\n"),
            ("ffi/raw.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a module beneath the permitted subtree is clean: {violations:?}"
    );
}

#[test]
pub(super) fn confine_flags_a_prefix_colliding_sibling_of_the_subtree() {
    // `crate::ffi_utils` is neither `crate::ffi` nor beneath `crate::ffi::` (`::`-delimited).
    let (result, violations) = run_module_check(
        "confine-sibling",
        &[
            ("lib.rs", "pub mod ffi;\npub mod ffi_utils;\n"),
            ("ffi.rs", "\n"),
            ("ffi_utils.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::ffi_utils");
}

#[test]
pub(super) fn confine_observes_a_glob_and_a_bare_import() {
    let (result, violations) = run_module_check(
        "confine-glob",
        &[
            ("lib.rs", "pub mod ffi;\npub mod a;\npub mod b;\n"),
            ("ffi.rs", "\n"),
            ("a.rs", "use libc::*;\n"),
            ("b.rs", "use libc;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(
        findings.contains(&"crate::a") && findings.contains(&"crate::b"),
        "{findings:?}"
    );
}

#[test]
pub(super) fn confine_ignores_a_different_external_crate() {
    let (result, violations) = run_module_check(
        "confine-other-crate",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use serde::Deserialize;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the rule observes only the confined crate: {violations:?}"
    );
}

#[test]
pub(super) fn confine_of_an_unimported_crate_is_clean_with_no_error() {
    // No cargo-metadata cross-check: confining a crate the target never imports is clean,
    // exactly as forbidding a crate you do not depend on is clean — never a constitution error.
    let (result, violations) = run_module_check(
        "confine-unimported",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use serde::Deserialize;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(
        result.is_ok(),
        "no error for an unimported confined crate: {result:?}"
    );
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
pub(super) fn confine_resolves_a_shadowing_root_module_as_internal_no_false_positive() {
    // A crate-root `mod libc;` shadows the extern prelude, so a root-file bare `use libc::…`
    // is the INTERNAL `crate::libc`, not the external crate — the external scan must not
    // observe it, or the rule would false-positive against the crate's own module.
    let (result, violations) = run_module_check(
        "confine-shadow",
        &[
            ("lib.rs", "pub mod libc;\npub mod ffi;\nuse libc::helper;\n"),
            ("libc.rs", "pub fn helper() {}\n"),
            ("ffi.rs", "\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a shadowing root module resolves internal, not as the confined external crate: {violations:?}"
    );
}

#[test]
pub(super) fn confine_observes_submodule_bare_and_leading_colon_forms() {
    let (result, violations) = run_module_check(
        "confine-forms",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\npub mod other;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
            ("other.rs", "use ::libc::c_void;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(
        findings.contains(&"crate::service") && findings.contains(&"crate::other"),
        "both a submodule-bare and a leading-`::` external import are observed: {findings:?}"
    );
}

#[test]
pub(super) fn confine_ignores_a_use_inside_a_string_literal() {
    let (result, violations) = run_module_check(
        "confine-string",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            (
                "service.rs",
                "pub fn f() { let _s = \"use libc::c_int;\"; }\n",
            ),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a use inside a string literal is stripped before scanning: {violations:?}"
    );
}

#[test]
pub(super) fn confine_dedups_multiple_imports_from_one_importer() {
    let (result, violations) = run_module_check(
        "confine-dedup",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\nuse libc::c_void;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one importer importing the confined crate twice yields one violation: {violations:?}"
    );
}

#[test]
pub(super) fn confine_identity_is_injective_across_confined_crates() {
    // The highest-risk invariant: two confinements of different crates on the same subtree,
    // breached by the same importer, must NOT collapse — baselining one must not mask the
    // other. Injectivity comes from the target being the confined crate.
    let files = [
        ("lib.rs", "pub mod ffi;\npub mod service;\n"),
        ("ffi.rs", "\n"),
        ("service.rs", "use libc::c_int;\nuse winapi::HANDLE;\n"),
    ];
    let (r1, libc_v) = run_module_check("confine-inj-libc", &files, confine("crate::ffi", "libc"));
    let (r2, winapi_v) = run_module_check(
        "confine-inj-winapi",
        &files,
        confine("crate::ffi", "winapi"),
    );
    assert!(r1.is_ok() && r2.is_ok(), "{r1:?} {r2:?}");
    assert_eq!(libc_v.len(), 1, "{libc_v:?}");
    assert_eq!(winapi_v.len(), 1, "{winapi_v:?}");
    assert_eq!(libc_v[0].target(), "libc");
    assert_eq!(winapi_v[0].target(), "winapi");
    assert_eq!(
        libc_v[0].finding, winapi_v[0].finding,
        "same offending importer — only the target (confined crate) differs"
    );
    // Baseline only the libc violation; the winapi violation must still react.
    let baseline = Baseline::of(&Report::new(libc_v.clone()));
    let mut both = Report::new(vec![libc_v[0].clone(), winapi_v[0].clone()]);
    apply_baseline(&mut both, &baseline);
    assert!(both.violations[0].baselined, "libc is baselined");
    assert!(
        !both.violations[1].baselined,
        "winapi is NOT masked by libc's baseline — distinct target keeps identity injective"
    );
    assert_eq!(
        Outcome::Violations(both).exit_code(),
        1,
        "the new winapi violation still fails"
    );
}

#[test]
pub(super) fn confine_canonicalizes_a_raw_identifier_crate_name() {
    // The confined crate name and the observed head canonicalize (`r#name` -> `name`), so a
    // boundary written with either form matches an import written with either form.
    let (result, violations) = run_module_check(
        "confine-raw-id",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use r#async::spawn;\n"),
        ],
        confine("crate::ffi", "async"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "raw and plain crate-name forms are one identity: {violations:?}"
    );
    assert_eq!(violations[0].target(), "async");
}

#[test]
pub(super) fn confine_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "confine-on-crate",
        &[("lib.rs", "use libc::c_int;\n")],
        confine("crate", "libc"),
    );
    let err = result.expect_err("confining to the crate root is a constitution error");
    assert_eq!(err, confine_external_crate_on_crate_error("x"));
}

#[test]
pub(super) fn confine_unknown_subtree_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "confine-unknown",
        &[("lib.rs", "pub mod ffi;\n"), ("ffi.rs", "\n")],
        confine("crate::nope", "libc"),
    );
    let err = result.expect_err("an unreachable permitted subtree is a constitution error");
    assert_eq!(err, unknown_module_error("crate::nope", "x"));
}

#[test]
pub(super) fn confine_honors_warn_severity() {
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::ffi")
        .confine_external_crate("libc")
        .warn()
        .because("advisory during adoption");
    let (result, violations) = run_module_check(
        "confine-warn",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
        ],
        boundary,
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
    assert_eq!(
        Outcome::Violations(Report::new(violations)).exit_code(),
        0,
        "a warn-only confinement does not fail CI"
    );
}

#[test]
pub(super) fn confine_external_crate_rule_text_and_json_params() {
    let rule = ModuleRule::ConfineExternalCrate {
        crate_name: "libc".to_string(),
    };
    assert_eq!(rule.label(), "external crate confined to module");
    assert_eq!(rule.polarity(), Polarity::AllowlistGap);
    assert_eq!(
        rule.text(),
        "confines external crate libc to this module's subtree"
    );
    assert_eq!(
        rule.json_params(),
        vec![("external_crate", serde_json::json!("libc"))]
    );
}

#[test]
pub(super) fn confine_external_crate_projects_its_crate_and_subtree() {
    // The full constitution projection (not just the rule params): the declared module
    // subtree and the confined crate must both be legible without reading the rule label.
    let constitution = Constitution::new("p").boundary(
        ModuleBoundary::in_crate("app")
            .module("crate::ffi")
            .confine_external_crate("libc")
            .because("the raw libc surface stays behind the ffi module"),
    );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("crate::ffi"),
        "the declared subtree is named: {text}"
    );
    assert!(
        text.contains("confines external crate libc"),
        "the confined crate is named: {text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "external crate confined to module"
    );
    // The declared module subtree is the boundary target (declaration view); the confined
    // crate projects under the self-describing `external_crate` key.
    assert_eq!(doc["boundaries"][0]["target"], "crate::ffi");
    assert_eq!(doc["boundaries"][0]["external_crate"], "libc");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    assert!(doc["boundaries"][0]["only"].is_null());
}

#[test]
pub(super) fn confine_matches_a_hyphenated_package_name_against_its_underscore_identifier() {
    // A package with a `-` (e.g. `windows-sys`) is imported through its `_` identifier
    // (`use windows_sys::…`) — a `use` path cannot contain `-`. The confined name is written in
    // package form yet must match the identifier the scanner observes, or the rule would silently
    // never react for exactly the hyphenated FFI/platform crates it targets (a false negative).
    let (result, violations) = run_module_check(
        "confine-hyphen",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            (
                "service.rs",
                "use windows_sys::Win32::Foundation::HANDLE;\n",
            ),
        ],
        confine("crate::ffi", "windows-sys"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a hyphenated package name matches its underscore import identifier: {violations:?}"
    );
    assert_eq!(violations[0].target(), "windows_sys");
    assert_eq!(violations[0].finding, "crate::service");
}

#[test]
pub(super) fn confine_observes_an_aliased_import_of_the_confined_crate() {
    // `use libc as c;` still imports libc; the alias is dropped at expansion, so the confined
    // crate's head is observed regardless of a local rename.
    let (result, violations) = run_module_check(
        "confine-alias",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc as c;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "an aliased import is still observed: {violations:?}"
    );
    assert_eq!(violations[0].target(), "libc");
    assert_eq!(violations[0].finding, "crate::service");
}

// The fourth inherited bound this capability's overview names, declared and pinned rather than left in
// prose. It was stated twice in the spec and invisible to the register both times — once on a line with no
// trigger words, once as "a stated, inherited bound" whose comma breaks the adjacency the scan needs — so
// nothing defended it while the overview read as permission.
//
// Probed before it was declared: the assertion here is that `extern crate libc;` outside the permitted
// subtree yields no violation, which is the scanner's use-only behaviour rather than a reasoned claim.
#[test]
pub(super) fn confine_ignores_an_extern_crate_declaration() {
    let (result, violations) = run_module_check(
        "confine-extern-crate",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "extern crate libc;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the rule is use-only: an `extern crate` declaration is not observed: {violations:?}"
    );
}

#[test]
pub(super) fn confine_external_crate_is_cfg_blind_to_unenabled_cfg_arms() {
    let (result, violations) = run_module_check(
        "confine-cfg-blind",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "#[cfg(impossible_predicate)]\nuse libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "libc");
    assert_eq!(violations[0].finding, "crate::service");
}

#[test]
pub(super) fn confine_external_crate_conflates_coincident_lib_and_bin_conventional_paths() {
    let (result, violations) = run_module_check(
        "confine-lib-bin-conflation",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "libc");
}
