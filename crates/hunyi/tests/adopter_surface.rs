use std::path::Path;

use hunyi::{
    BoundDecl, BoundId, Defence, Demonstrates, Extent, FactGranularity, Observer, Outcome, Owner,
    Reached, RuleKey, SemanticBoundaries, SemanticObserver, StructuredFactIdentity, Violation,
    ViolationId, check_all,
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
    accepts_observer::<SemanticObserver>();
}
