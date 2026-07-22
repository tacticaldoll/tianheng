use louke::{RuleKey, StructuredFactIdentity, Violation, ViolationId};

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
}

#[cfg(feature = "audit")]
#[test]
fn standalone_runtime_audit_exposes_a_pure_reaction() {
    let _ = louke::audit_probe_coverage;
}
