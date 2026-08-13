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

/// `observation_bounds()` is a plain library item an adopter can call without composing a run.
///
/// Named here because this file is the contract that enumerates the promise, and this member reached the
/// public surface in the same window that closed exactly this gap for the protocol's other prelude
/// members — the
/// enumeration grew and the file naming it stood still. Reading a dimension's declared bounds without
/// implementing `Observer` is what `observation-bound-model` obliges, and until this nothing outside the
/// crate compiled against it.
#[test]
fn declared_bounds_are_readable_without_composing_a_run() {
    let declared: Vec<BoundDecl> = hunyi::observation_bounds();
    for bound in &declared {
        let _: &BoundId = bound.id();
        let _: &Extent = bound.extent();
    }
}
