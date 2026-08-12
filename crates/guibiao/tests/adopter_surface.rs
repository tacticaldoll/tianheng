use std::path::Path;

use guibiao::{
    BoundDecl, BoundId, Constitution, Defence, Demonstrates, Extent, FactGranularity,
    ModuleBoundary, Observer, Outcome, Owner, Reached, RuleKey, ScanDepth, StaticObserver,
    StructuredFactIdentity, Violation, ViolationId, check,
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
    let _ = std::mem::size_of::<(
        BoundDecl,
        BoundId,
        Defence,
        Demonstrates,
        Extent,
        FactGranularity,
        Owner,
        Reached,
    )>();
    fn accepts_observer<T: Observer>() {}
    accepts_observer::<StaticObserver>();
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

/// `observation_bounds()` is a plain library item an adopter can call without composing a run.
///
/// Named here because this file is the contract that enumerates the promise, and this member reached the
/// public surface in the same window that closed exactly this gap for the protocol's other thirteen — the
/// enumeration grew and the file naming it stood still. Reading a dimension's declared bounds without
/// implementing `Observer` is what `observation-bound-model` obliges, and until this nothing outside the
/// crate compiled against it.
#[test]
fn declared_bounds_are_readable_without_composing_a_run() {
    let declared: Vec<BoundDecl> = guibiao::observation_bounds();
    for bound in &declared {
        let _: &BoundId = bound.id();
        let _: &Extent = bound.extent();
    }
}
