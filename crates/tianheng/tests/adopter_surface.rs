//! External-view compile contract for the composed adopter surface.
//!
//! This integration test is a separate crate: every name below must therefore be reachable through
//! the same wildcard prelude an adopter uses. It deliberately names the whole promised surface,
//! including types that representative declarations do not otherwise need.

use std::path::Path;

use tianheng::prelude::*;

fn assert_public_type<T>() {}

#[test]
fn wildcard_prelude_is_the_external_adopter_contract() {
    // Declaration and execution tier.
    assert_public_type::<Constitution>();
    assert_public_type::<CrateBoundary>();
    assert_public_type::<ModuleBoundary>();
    assert_public_type::<SemanticBoundary>();
    assert_public_type::<TraitImplBoundary>();
    assert_public_type::<VisibilityBoundary>();
    assert_public_type::<ForbiddenMarkerBoundary>();
    assert_public_type::<DynTraitBoundary>();
    assert_public_type::<ImplTraitBoundary>();
    assert_public_type::<AsyncExposureBoundary>();
    assert_public_type::<UnsafeBoundary>();
    assert_public_type::<RuntimeBoundary>();
    assert_public_type::<SansIoPure>();
    assert_public_type::<DependencyKind>();
    assert_public_type::<SourceKind>();
    assert_public_type::<VisibilityCeiling>();
    assert_public_type::<Severity>();

    // Reaction-inspection tier.
    assert_public_type::<Boundary>();
    assert_public_type::<BoundaryKind>();
    assert_public_type::<Rule>();
    assert_public_type::<ModuleRule>();
    assert_public_type::<Baseline>();
    assert_public_type::<BaselineEntry>();
    assert_public_type::<Finding>();
    assert_public_type::<FindingKey>();
    assert_public_type::<Outcome>();
    assert_public_type::<Polarity>();
    assert_public_type::<Report>();
    assert_public_type::<Violation>();
    assert_public_type::<ViolationId>();

    let crate_boundary = CrateBoundary::crate_("consumer-core")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .dependency_kind(DependencyKind::Normal)
        .warn()
        .because("the core declares only reviewable dependency sources");
    assert_eq!(crate_boundary.severity(), Severity::Warn);
    match crate_boundary.rule() {
        Rule::RestrictDependencySourcesTo { allowed, .. } => {
            assert_eq!(allowed, &[SourceKind::Registry, SourceKind::Path]);
        }
        _ => unreachable!(),
    }

    let module_boundary = ModuleBoundary::in_crate("consumer-core")
        .module("crate::domain")
        .must_not_import("crate::adapter")
        .because("the domain depends inward only");
    let _: Boundary = module_boundary.clone().into();
    match module_boundary.rule() {
        ModuleRule::MustNotImport { module, .. } => assert_eq!(module, "crate::adapter"),
        _ => unreachable!(),
    }

    let signature_boundary = SemanticBoundary::in_crate("consumer-core")
        .module("crate::api")
        .must_not_expose("crate::adapter::Client")
        .because("the public API owns its vocabulary");
    let visibility_boundary = VisibilityBoundary::in_crate("consumer-core")
        .module("crate::internal")
        .max_visibility(VisibilityCeiling::Crate)
        .because("internal implementation stays crate-visible");
    let runtime_boundary = RuntimeBoundary::at("domain-entry")
        .only_origins(["consumer::adapter"])
        .because("only the declared adapter crosses the seam");
    let profile = SansIoPure::in_crate("consumer-core")
        .module("crate::domain")
        .reading_clock_via("std::time", ["now"])
        .because("the domain receives time through its seam");

    let constitution = Constitution::new("consumer")
        .boundary(crate_boundary)
        .boundary(module_boundary)
        .signature_boundary(signature_boundary)
        .visibility_boundary(visibility_boundary)
        .runtime(runtime_boundary)
        .sans_io_pure(profile);

    // Function items and closures are type-checked but never invoked: this contract proves the
    // public call shapes without parsing a CLI, scanning a workspace, or writing process output.
    let _run = run::<[&str; 0], &str>;
    let _static_check = |manifest: &Path| check(constitution.static_boundaries(), manifest);
    let _signature_check = |manifest: &Path| {
        tianheng::check_semantic(&constitution.semantic_boundaries().signature, manifest)
    };

    assert_eq!(BoundaryKind::Crate.as_str(), "crate");
    assert_eq!(Polarity::DenyBreach.as_str(), "deny_breach");
}
