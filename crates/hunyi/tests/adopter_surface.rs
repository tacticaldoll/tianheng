use std::path::Path;

use hunyi::{
    Outcome, RuleKey, SemanticBoundaries, StructuredFactIdentity, Violation, ViolationId, check_all,
};

fn inspect(violation: &Violation) {
    let _: &RuleKey = violation.rule_key();
    let _: &StructuredFactIdentity = violation.fact();
    let id: ViolationId = violation.id();
    let _ = (id.target(), id.rule_key(), id.fact());
}

#[test]
fn standalone_semantic_surface_exposes_the_shared_reaction_model() {
    let _: fn(&SemanticBoundaries, &Path) -> Outcome = check_all;
    let _: fn(&Violation) = inspect;
}
