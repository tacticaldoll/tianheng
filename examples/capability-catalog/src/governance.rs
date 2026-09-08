//! The catalog Constitution. Its breadth is for contract coverage; focused examples remain the
//! imitable onboarding surface.

use tianheng::prelude::*;

/// Build the deliberately violated catalog law.
pub fn constitution() -> Constitution {
    Constitution::new("capability_catalog")
        .boundary(
            CrateBoundary::crate_("capability_catalog")
                .restrict_dependency_sources_to([SourceKind::Path])
                .because("catalog source metadata must produce its declared source reaction"),
        )
        .boundary(
            ModuleBoundary::in_crate("capability_catalog")
                .module("crate::governance")
                .confine_external_crate("tianheng")
                .because("the external shell dependency stays behind the governance module"),
        )
        .trait_impl_boundary(
            TraitImplBoundary::in_crate("capability_catalog")
                .trait_("crate::Command")
                .only_implemented_in("crate::allowed")
                .because("Command implementations live only under the allowed subtree"),
        )
        .forbidden_marker_boundary(
            ForbiddenMarkerBoundary::in_crate("capability_catalog")
                .module("crate::marked")
                .must_not_acquire("crate::Marker")
                .because("marked-domain types remain free of the catalog marker"),
        )
        .dyn_trait_boundary(
            DynTraitBoundary::in_crate("capability_catalog")
                .module("crate::shapes")
                .must_not_expose_dyn()
                .because("the catalog dyn family must produce its structured reaction"),
        )
        .impl_trait_boundary(
            ImplTraitBoundary::in_crate("capability_catalog")
                .module("crate::shapes")
                .must_not_expose_impl_trait()
                .because("the catalog impl-trait family must produce its structured reaction"),
        )
        .no_existential_leak(
            NoExistentialLeak::in_crate("capability_catalog")
                .module("crate::shapes")
                .because(
                    "the catalog's composed no-existential-leak profile must produce its \
                     structured reaction for both the written and the implicit existential signal",
                ),
        )
}
