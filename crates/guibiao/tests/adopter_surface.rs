use std::path::Path;

use guibiao::{
    Constitution, Outcome, RuleKey, StructuredFactIdentity, Violation, ViolationId, check,
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
