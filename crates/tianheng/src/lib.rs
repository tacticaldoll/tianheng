//! 天衡 (Tiānhéng) — reactive architectural governance.
//!
//! **Govern by reaction, not instruction.** The *balance* (天衡, 玉衡): this crate is the
//! imperative shell + facade. It composes the 三儀 (圭表 static, 渾儀 semantic, 漏刻 runtime CI
//! face) into one reaction, and exposes a single declared [`Constitution`] plus [`run`] — the
//! CLI reaction that turns it into a process exit code (`0` clean / `1` enforced violation /
//! `2` constitution-or-scan error).
//!
//! The functional cores ([`guibiao`], [`hunyi`], [`louke`]) do the observation and
//! comparison; this crate owns the side effects (argument parsing, filesystem, stdout/stderr)
//! and the composition. The cores must not depend on this shell — a crate-level invariant
//! Tianheng enforces on itself (`crates/shengmo/src/law.rs`).
//!
//! **One declared source, three projections.** An adopter writes one [`Constitution`] carrying
//! every dimension's boundaries; the static and semantic dimensions project as a CI exit code,
//! and 漏刻 projects both as a CI exit code (its probe-coverage audit, composed here) and, in
//! the adopter's binary, as a runtime event (the prod face, consumed directly via [`louke`]).
//! The runtime boundaries declared here are the same objects the adopter hands to
//! [`louke::install`] at startup — the single source of truth, two faces.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

/// Compile the crate README's Rust examples as doctests so the adopter-facing
/// snippets cannot rot (a wrong signature or removed export fails `cargo test`).
/// Gated on `cfg(doctest)` so it neither enters the public API nor the docs.rs
/// page — only rustdoc's doctest pass sees it.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
pub struct ReadmeDoctests;

mod existential;
mod runner;
mod sans_io;
pub mod testing;

pub use testing::GovernanceTest;

pub use guibiao::{
    Baseline, BaselineEntry, Boundary, BoundaryKind, CrateBoundary, CrateTarget, DependencyKind,
    Finding, ModuleBoundary, ModuleRule, Outcome, Polarity, Report, Rule, RuleKey, ScanDepth,
    Severity, SourceKind, StructuredFactIdentity, Subject, Violation, ViolationId, check,
    workspace_member_src_dirs,
};
/// The static 圭表 (gnomon) constitution — the static dimension's own declaration, reached under
/// its instrument name so the bare `Constitution` can be the unified shell-level type.
pub use guibiao::Constitution as GnomonConstitution;

pub use hunyi::{
    BoundDecl, BoundId, Defence, Demonstrates, Extent, FactGranularity, Observer, Owner, Reached,
};

pub use guibiao::StaticObserver;
pub use hunyi::SemanticObserver;
pub use louke::RuntimeObserver;

pub use hunyi::{
    AsyncExposureBoundary, DynTraitBoundary, ForbiddenMarkerBoundary, ImplTraitBoundary,
    SemanticBoundaries, SignatureBoundary, TraitImplBoundary, UnsafeBoundary, VisibilityBoundary,
    VisibilityCeiling, check as check_semantic,
};
#[doc(hidden)]
pub use hunyi::{
    check_all, check_async_exposure, check_dyn_trait, check_forbidden_marker, check_impl_trait,
    check_trait_impl_locality, check_unsafe_confinement, check_visibility,
};
pub use louke::{OriginEntry, Posture, RuntimeBoundary, Tracked, audit_probe_coverage};

#[doc(hidden)]
pub use guibiao::{
    CrateBoundaryBuilder, CrateBoundaryDraft, DenyExternalDraft, ModuleBoundaryBuilder,
    ModuleBoundaryDraft, ModuleTargetDraft,
};
#[doc(hidden)]
pub use hunyi::{
    AsyncExposureBoundaryDraft, AsyncExposureCrateDraft, AsyncExposureModuleDraft,
    DynTraitBoundaryDraft, DynTraitCrateDraft, DynTraitModuleDraft, ForbiddenMarkerBoundaryDraft,
    ForbiddenMarkerCrateDraft, ForbiddenMarkerModuleDraft, ImplTraitBoundaryDraft,
    ImplTraitCrateDraft, ImplTraitModuleDraft, SignatureBoundaryDraft, SignatureCrateDraft,
    SignatureModuleDraft, TraitImplBoundaryDraft, TraitImplCrateDraft, TraitImplTraitDraft,
    UnsafeBoundaryDraft, UnsafeCrateDraft, VisibilityBoundaryDraft, VisibilityCrateDraft,
    VisibilityModuleDraft,
};
#[doc(hidden)]
pub use louke::{RuntimeBoundaryDraft, RuntimeSeamDraft};

pub use existential::NoExistentialLeak;
#[doc(hidden)]
pub use existential::{NoExistentialLeakCrateDraft, NoExistentialLeakModuleDraft};
pub use sans_io::SansIoPure;
#[doc(hidden)]
pub use sans_io::{SansIoPureCrateDraft, SansIoPureDraft, SansIoPureModuleDraft};

pub use runner::{Run, check_constitution, constitution_markdown, projection_gate, run};

/// A declared constitution composing every observation dimension's boundaries — the single
/// source of truth, in Rust. The static (圭表) boundaries, the semantic (渾儀) bundle, and the
/// runtime (漏刻) boundaries hang off one builder, so adding a dimension is a field, never a new
/// `run` argument.
///
/// A static-only adopter writes `Constitution::new(name).boundary(...)` exactly as before —
/// `.boundary` delegates to the inner static constitution. Semantic and runtime boundaries are
/// folded in via the typed adders. For the *pure static core* (`guibiao::check`), use
/// [`GnomonConstitution`] directly; this unified type is the shell's composition surface.
#[derive(Debug, Clone)]
pub struct Constitution {
    static_: GnomonConstitution,
    semantic: SemanticBoundaries,
    runtime: Vec<RuntimeBoundary>,
}

impl Constitution {
    /// Begin a constitution with the given project name.
    pub fn new(name: &str) -> Self {
        Constitution {
            static_: GnomonConstitution::new(name),
            semantic: SemanticBoundaries::default(),
            runtime: Vec::new(),
        }
    }

    /// Add a static (圭表) boundary — a [`CrateBoundary`] or [`ModuleBoundary`]. Delegates to the
    /// inner static constitution, so the ergonomics match the static-only path exactly.
    pub fn boundary(mut self, boundary: impl Into<Boundary>) -> Self {
        self.static_ = self.static_.boundary(boundary);
        self
    }

    /// Add a 渾儀 signature-coupling boundary (a module's API must not expose a forbidden type).
    pub fn signature_boundary(mut self, boundary: SignatureBoundary) -> Self {
        self.semantic.signature.push(boundary);
        self
    }

    /// Add a 渾儀 trait-impl-locality boundary (a trait may only be implemented in declared places).
    pub fn trait_impl_boundary(mut self, boundary: TraitImplBoundary) -> Self {
        self.semantic.trait_impl.push(boundary);
        self
    }

    /// Add a 渾儀 visibility boundary (a module must not declare `pub` items).
    pub fn visibility_boundary(mut self, boundary: VisibilityBoundary) -> Self {
        self.semantic.visibility.push(boundary);
        self
    }

    /// Add a 渾儀 forbidden-marker boundary (a subtree must not acquire a forbidden trait).
    pub fn forbidden_marker_boundary(mut self, boundary: ForbiddenMarkerBoundary) -> Self {
        self.semantic.forbidden_marker.push(boundary);
        self
    }

    /// Add a 渾儀 dyn-trait boundary (a module's API must not expose `dyn` trait-object syntax).
    pub fn dyn_trait_boundary(mut self, boundary: DynTraitBoundary) -> Self {
        self.semantic.dyn_trait.push(boundary);
        self
    }

    /// Add a 渾儀 impl-trait boundary (a module's API must not return a written `impl Trait`).
    pub fn impl_trait_boundary(mut self, boundary: ImplTraitBoundary) -> Self {
        self.semantic.impl_trait.push(boundary);
        self
    }

    /// Add a 渾儀 async-exposure boundary (a module's API must not declare an `async fn`).
    pub fn async_exposure_boundary(mut self, boundary: AsyncExposureBoundary) -> Self {
        self.semantic.async_exposure.push(boundary);
        self
    }

    /// Add a 渾儀 unsafe-confinement boundary (`unsafe` may appear only under the declared subtree).
    pub fn unsafe_boundary(mut self, boundary: UnsafeBoundary) -> Self {
        self.semantic.unsafe_confinement.push(boundary);
        self
    }

    /// Add a 漏刻 runtime boundary. The CI face audits its probe coverage (via [`run`]); the same
    /// object is what the adopter hands to [`louke::install`] for the prod face.
    pub fn runtime(mut self, boundary: RuntimeBoundary) -> Self {
        self.runtime.push(boundary);
        self
    }

    /// The static (圭表) constitution, for the pure static core and projection.
    pub fn static_boundaries(&self) -> &GnomonConstitution {
        &self.static_
    }

    /// The semantic (渾儀) bundle, for the semantic dimension and projection.
    pub fn semantic_boundaries(&self) -> &SemanticBoundaries {
        &self.semantic
    }

    /// The runtime (漏刻) boundaries — the single source both the CI audit and [`louke::install`]
    /// project from.
    pub fn runtime_boundaries(&self) -> &[RuntimeBoundary] {
        &self.runtime
    }
}

/// Lift a pure static ([`GnomonConstitution`]) constitution into the unified [`Constitution`],
/// with no semantic or runtime boundaries. This is the projection-side bridge: a static-only
/// law (e.g. a self-governance constitution) can be rendered through the shell's
/// [`constitution_markdown`] without being re-declared, while the
/// static `check` path keeps consuming the `GnomonConstitution` directly.
impl From<GnomonConstitution> for Constitution {
    fn from(static_: GnomonConstitution) -> Self {
        Constitution {
            static_,
            semantic: SemanticBoundaries::default(),
            runtime: Vec::new(),
        }
    }
}

/// The public facade for declaring a constitution and running the reaction. The projection,
/// baseline, and scanner internals stay in the dimension crates; consumers go through
/// `Constitution` / `run` (and `check` for the pure static core).
///
/// # Compatibility contract
///
/// The wildcard prelude has two usage tiers:
///
/// - **Declaration and execution:** `Constitution`, the terminal boundary and composed-profile
///   types, their selector enums, [`Severity`], and [`run`]. This is the normal adopter path.
/// - **Reaction inspection:** [`Outcome`], reports, violations, stable finding/violation identity,
///   baselines, boundary/rule model types, the pure static [`check`], and the unified
///   [`check_constitution`]. These let a caller inspect the reaction without constructing rules
///   outside the boundary DSL or driving CLI presentation.
///
/// The tiers explain purpose, not stability. Specialized semantic checks do not expand this
/// wildcard menu: the signature-coupling check is the explicit [`check_semantic`] root import, and
/// composed governance continues through [`Constitution`] plus [`run`].
pub mod prelude {
    pub use super::{
        AsyncExposureBoundary, Baseline, BaselineEntry, BoundDecl, BoundId, Boundary, BoundaryKind,
        Constitution, CrateBoundary, Defence, Demonstrates, DependencyKind, DynTraitBoundary,
        Extent, FactGranularity, Finding, ForbiddenMarkerBoundary, GovernanceTest,
        ImplTraitBoundary, ModuleBoundary, ModuleRule, NoExistentialLeak, Observer, Outcome, Owner,
        Polarity, Reached, Report, Rule, RuleKey, Run, RuntimeBoundary, RuntimeObserver,
        SansIoPure, ScanDepth, SemanticObserver, Severity, SignatureBoundary, SourceKind,
        StaticObserver, StructuredFactIdentity, Subject, TraitImplBoundary, UnsafeBoundary,
        Violation, ViolationId, VisibilityBoundary, VisibilityCeiling, check, check_constitution,
        run,
    };
}
