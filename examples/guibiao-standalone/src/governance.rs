//! The constitution the adopter writes — the imitable surface (潛移). Copy this shape into
//! your own project and change the module paths and the reason.
use guibiao::{Constitution, ModuleBoundary, ScanDepth};

/// The declared law: in this crate, `crate::domain` must not import `crate::infra`.
pub fn constitution() -> Constitution {
    Constitution::new("hexagonal_demo").boundary(
        ModuleBoundary::in_crate("hexagonal_demo")
            .module("crate::domain")
            .must_not_import("crate::infra")
            .depth(ScanDepth::Subtree)
            .because("the domain stays pure — it never depends on infrastructure"),
    )
}
