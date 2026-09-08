//! The constitution the adopter writes — the imitable surface (潛移). Copy this shape into
//! your own project and change the module paths and the reason.
use guibiao::{Constitution, CrateBoundary, ModuleBoundary, ScanDepth};

/// The declared law: two boundaries, one per granularity 圭表 observes.
///
/// The **module** boundary is the one that reacts here — `crate::domain` imports
/// `crate::infra` on purpose, so this example exits 1 and shows what drift looks like.
///
/// The **crate** boundary is the one that holds, and it is here because a passing
/// boundary is the other half of the teaching: this crate's manifest declares exactly
/// one dependency, and that light footprint is 圭表's own pitch. A pitch stated in a
/// manifest comment is prose; declared as an allowlist it reacts the moment a second
/// dependency arrives. An allowlist is also strictly stronger than naming what to
/// forbid — a dependency nobody thought to forbid is already refused.
pub fn constitution() -> Constitution {
    Constitution::new("hexagonal_demo")
        .boundary(
            ModuleBoundary::in_crate("hexagonal_demo")
                .module("crate::domain")
                .must_not_import("crate::infra")
                .depth(ScanDepth::Subtree)
                .because("the domain stays pure — it never depends on infrastructure"),
        )
        .boundary(
            CrateBoundary::crate_("hexagonal_demo")
                .restrict_dependencies_to(["guibiao"])
                .because(
                    "the demo depends on the instrument that governs it and nothing else — the \
                     light footprint is what 圭表 is for",
                ),
        )
}
