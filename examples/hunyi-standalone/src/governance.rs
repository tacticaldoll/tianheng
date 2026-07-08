//! The constitution the adopter writes — the imitable surface (潛移). One semantic boundary:
//! the public API must not expose the internal pool type.
use hunyi::SemanticBoundary;

/// The declared law: `crate::api`'s public surface must not expose `crate::infra::DbPool`.
pub fn constitution() -> Vec<SemanticBoundary> {
    vec![SemanticBoundary::in_crate("api_hygiene")
        .module("crate::api")
        .must_not_expose("crate::infra::DbPool")
        .because("the public API must not leak the internal database pool")]
}
