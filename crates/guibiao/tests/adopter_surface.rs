use std::path::Path;

use guibiao::{
    Constitution, ModuleBoundary, Outcome, RuleKey, ScanDepth, StructuredFactIdentity, Violation,
    ViolationId, check,
};

fn inspect(violation: &Violation) {
    let _: &RuleKey = violation.rule_key();
    let _: &StructuredFactIdentity = violation.fact();
    let id: ViolationId = violation.id();
    let _ = (id.target(), id.rule_key(), id.fact());
}

#[test]
fn standalone_static_surface_exposes_the_shared_reaction_model() {
    let _: fn(&Constitution, &Path) -> Outcome = check;
    let _: fn(&Violation) = inspect;
}

#[test]
fn legacy_module_builder_and_finding_identity_names_stay_source_compatible() {
    let boundary = ModuleBoundary::in_crate("consumer")
        .module("crate::core")
        .must_not_import("crate::adapter")
        .depth(ScanDepth::Shallow)
        .including_submodules()
        .because("the core depends inward");
    assert_eq!(boundary.scan_depth(), ScanDepth::Subtree);

    let finding = guibiao::Finding::new(
        "forbidden import",
        StructuredFactIdentity::of(
            "tianheng.fact/test/adopter",
            "module-import",
            [("module", "crate::adapter")],
        ),
    );
    assert_eq!(finding.fact(), finding.key());
}
