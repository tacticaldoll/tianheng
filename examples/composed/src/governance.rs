//! The constitution the adopter writes — and the funnel made literal. It grows by one
//! `.boundary()` / `.signature_boundary()` / `.runtime()` per instrument: adopt one 儀, then
//! add the next. `static_only` is the 圭表 view; `plus_semantic` adds 渾儀; `constitution` is
//! 天衡 all-open, including the 漏刻 runtime seam.
use tianheng::prelude::*;

/// Stage 1 — 圭表 alone: the domain must not import infra.
pub fn static_only() -> Constitution {
    Constitution::new("composed_app").boundary(
        ModuleBoundary::in_crate("composed_app")
            .module("crate::domain")
            .must_not_import("crate::infra")
            .because("the domain stays pure — it never depends on infrastructure"),
    )
}

/// Stage 2 — + 渾儀: the public API must not expose the internal pool type.
pub fn plus_semantic() -> Constitution {
    static_only().signature_boundary(
        SemanticBoundary::in_crate("composed_app")
            .module("crate::api")
            .must_not_expose("crate::infra::DbPool")
            .because("the public API must not leak the internal database pool"),
    )
}

/// Stage 3 — 天衡 all-open: + 漏刻's port seam. Only the blessed adapter's origin may cross.
pub fn constitution() -> Constitution {
    plus_semantic().runtime(
        RuntimeBoundary::at("adapter-seam")
            .only_origins(["composed_app::adapters::blessed"])
            .because("only the blessed adapter may cross the port seam"),
    )
}
