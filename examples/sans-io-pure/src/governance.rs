//! The constitution the adopter writes — the imitable surface (潛移). One `sans_io_pure` profile
//! folds both axes of a sans-I/O kernel into a single declaration.
use tianheng::prelude::*;

/// The declared law: `crate::kernel` is sans-I/O throughout — no ambient clock, no `async` on its
/// public API anywhere under it.
pub fn constitution() -> Constitution {
    Constitution::new("sans_io_kernel").sans_io_pure(
        SansIoPure::in_crate("sans_io_kernel")
            .module("crate::kernel")
            .reading_clock_via("std::time", ["now"])
            .because("the kernel stays sans-I/O: time is injected, and async lives at the edges"),
    )
}
