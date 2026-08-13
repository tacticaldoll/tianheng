//! External-view compile contract for the composed adopter surface.
//!
//! This integration test is a separate crate: every name below must therefore be reachable through
//! the same wildcard prelude an adopter uses. It deliberately names the whole promised surface,
//! including types that representative declarations do not otherwise need.

use std::path::Path;

use tianheng::prelude::*;

fn assert_public_type<T>() {}

/// A trait is not a type, so [`Observer`] cannot go through `assert_public_type` — that is
/// `E0782: expected a type, found a trait`. Naming it in a **bound** proves more than a type
/// assertion would: the trait is reachable through the prelude *and* the observer the prelude
/// re-exports beside it satisfies it, which is what a third party writing their own participant
/// needs to be true.
fn assert_public_observer<T: Observer>() {}

#[test]
fn wildcard_prelude_is_the_external_adopter_contract() {
    // Declaration and execution tier.
    assert_public_type::<Constitution>();
    assert_public_type::<CrateBoundary>();
    assert_public_type::<ModuleBoundary>();
    assert_public_type::<SignatureBoundary>();
    assert_public_type::<TraitImplBoundary>();
    assert_public_type::<VisibilityBoundary>();
    assert_public_type::<ForbiddenMarkerBoundary>();
    assert_public_type::<DynTraitBoundary>();
    assert_public_type::<ImplTraitBoundary>();
    assert_public_type::<AsyncExposureBoundary>();
    assert_public_type::<UnsafeBoundary>();
    assert_public_type::<RuntimeBoundary>();
    assert_public_type::<SansIoPure>();
    assert_public_type::<NoExistentialLeak>();
    assert_public_type::<GovernanceTest>();
    assert_public_type::<ScanDepth>();
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
    assert_public_type::<RuleKey>();
    assert_public_type::<StructuredFactIdentity>();
    assert_public_type::<Outcome>();
    // The evidence a clean verdict carries. Named here because an adopter that can read a violation's
    // structure but nothing from a clean run cannot tell an observed workspace from an unreached one.
    assert_public_type::<Subject>();
    assert_public_type::<Polarity>();
    assert_public_type::<Report>();
    assert_public_type::<Violation>();
    assert_public_type::<ViolationId>();

    // Observation-protocol tier. A third party's participant is written against exactly these: the
    // trait, the bound model it must return, and the run it joins. Each is promised by the prelude,
    // so each is named here in the form its kind admits — the observers as types, the trait as a
    // bound, and `Run` through the composition shape below.
    assert_public_type::<StaticObserver>();
    assert_public_type::<SemanticObserver>();
    assert_public_type::<RuntimeObserver>();
    assert_public_observer::<StaticObserver>();
    assert_public_observer::<SemanticObserver>();
    assert_public_observer::<RuntimeObserver>();
    assert_public_type::<Run>();
    assert_public_type::<BoundDecl>();
    assert_public_type::<BoundId>();
    assert_public_type::<Extent>();
    assert_public_type::<Reached>();
    assert_public_type::<Owner>();
    assert_public_type::<Defence>();
    assert_public_type::<Demonstrates>();
    assert_public_type::<FactGranularity>();

    let violation = Violation::new(
        BoundaryKind::Crate,
        ViolationId::new(
            "consumer-core",
            RuleKey::of("tianheng.rule/test/adopter", [] as [(&str, &str); 0]),
            StructuredFactIdentity::new(
                "tianheng.fact/test/adopter",
                "fact",
                [] as [(&str, &str); 0],
            )
            .unwrap(),
        ),
        "adopter rule",
        "adopter fact",
        "the consumer core stays governed".to_string(),
        Severity::Enforce,
    );
    assert_eq!(violation.target(), "consumer-core");

    let finding = Finding::new(
        "adopter fact",
        StructuredFactIdentity::of(
            "tianheng.fact/test/adopter",
            "fact",
            [] as [(&str, &str); 0],
        ),
    );
    assert_eq!(finding.fact(), finding.key());

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
        .depth(ScanDepth::Shallow)
        .including_submodules()
        .because("the domain depends inward only");
    assert_eq!(module_boundary.scan_depth(), ScanDepth::Subtree);
    let _: Boundary = module_boundary.clone().into();
    match module_boundary.rule() {
        ModuleRule::MustNotImport { module, .. } => assert_eq!(module, "crate::adapter"),
        _ => unreachable!(),
    }

    let signature_boundary = SignatureBoundary::in_crate("consumer-core")
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
    let existential_profile = NoExistentialLeak::in_crate("consumer-core")
        .module("crate::api")
        .because("the public API names concrete return types");

    let constitution = Constitution::new("consumer")
        .boundary(crate_boundary)
        .boundary(module_boundary)
        .signature_boundary(signature_boundary)
        .visibility_boundary(visibility_boundary)
        .runtime(runtime_boundary)
        .sans_io_pure(profile)
        .no_existential_leak(existential_profile);

    // Function items and closures are type-checked but never invoked: this contract proves the
    // public call shapes without parsing a CLI, scanning a workspace, or writing process output.
    let _run = run::<[&str; 0], &str>;
    let _static_check = |manifest: &Path| check(constitution.static_boundaries(), manifest);
    let _composed_check = |manifest: &Path| check_constitution(&constitution, manifest);
    let _signature_check = |manifest: &Path| {
        tianheng::check_semantic(&constitution.semantic_boundaries().signature, manifest)
    };

    // The composition shape, type-checked and never executed like the checks above: a run is opened
    // over a manifest, each dimension observer joins it, and one verdict comes out. This is the whole
    // entrypoint a participant outside this family composes into, so proving it reachable from the
    // wildcard prelude is the promise that matters most about `Run`.
    //
    // The clone per dimension is the documented shape, not an accident of this contract: each
    // observer's constructor takes its dimension's declarations by value while the composed
    // `Constitution` lends them, so an outside caller composing a run from one owns a copy. Writing it
    // here is what makes that cost visible from outside rather than only in the shell.
    let _composed_run = |manifest: &Path| {
        Run::over(manifest)
            .observe(StaticObserver::new(
                constitution.static_boundaries().clone(),
            ))
            .observe(SemanticObserver::new(
                constitution.semantic_boundaries().clone(),
            ))
            .observe(RuntimeObserver::new(
                constitution.runtime_boundaries().to_vec(),
            ))
            .verdict()
    };

    assert_eq!(BoundaryKind::Crate.as_str(), "crate");
    assert_eq!(Polarity::DenyBreach.as_str(), "deny_breach");
}
