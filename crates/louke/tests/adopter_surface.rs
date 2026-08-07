#[cfg(feature = "audit")]
use louke::Observer;
use louke::{
    BoundDecl, BoundId, Defence, Demonstrates, Extent, FactGranularity, Owner, Reached, RuleKey,
    StructuredFactIdentity, Violation, ViolationId,
};

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
