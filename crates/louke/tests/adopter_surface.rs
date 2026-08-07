use std::path::Path;

use louke::{
    BoundDecl, BoundId, Defence, Demonstrates, Extent, FactGranularity, Observer, Outcome, Owner,
    Reached, RuleKey, StructuredFactIdentity, Violation, ViolationId,
};

struct ExternalObserver;

impl Observer for ExternalObserver {
    fn observe(&self, _manifest_path: &Path) -> Outcome {
        Outcome::Clean
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
        Outcome::Clean
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
