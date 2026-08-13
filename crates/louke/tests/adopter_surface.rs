use std::path::Path;

use louke::{
    BoundDecl, BoundId, Defence, Demonstrates, Extent, FactGranularity, Observer, Outcome, Owner,
    Reached, RuleKey, StructuredFactIdentity, Subject, Violation, ViolationId,
};

struct ExternalObserver;

impl Observer for ExternalObserver {
    fn observe(&self, _manifest_path: &Path) -> Outcome {
        // A third party with nothing declared says so by name, from the public surface alone — no reach into
        // any dimension's internals, which is what this fixture exists to demonstrate.
        Outcome::Clean(Subject::nothing_declared())
    }

    fn bounds(&self) -> Vec<BoundDecl> {
        Vec::new()
    }
}

fn inspect(violation: &Violation) {
    let _: &RuleKey = violation.rule_key();
    let _: &StructuredFactIdentity = violation.fact();
    let id: ViolationId = violation.id();
    let _ = (id.target(), id.rule_key(), id.fact());
}

#[test]
fn standalone_runtime_surface_exposes_the_shared_reaction_model() {
    let _: fn(&Violation) = inspect;
    let _ = louke::set_sink::<fn(&Violation)>;
    assert!(matches!(
        ExternalObserver.observe(Path::new("Cargo.toml")),
        Outcome::Clean(_)
    ));
    assert!(ExternalObserver.bounds().is_empty());
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
}

#[cfg(feature = "audit")]
#[test]
fn standalone_runtime_audit_exposes_a_pure_reaction() {
    let _ = louke::audit_probe_coverage;
    fn accepts_observer<T: Observer>() {}
    accepts_observer::<louke::RuntimeObserver>();
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
    let declared: Vec<BoundDecl> = louke::observation_bounds();
    for bound in &declared {
        let _: &BoundId = bound.id();
        let _: &Extent = bound.extent();
    }
}
