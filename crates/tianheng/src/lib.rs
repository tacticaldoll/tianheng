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
//! Tianheng enforces on itself (`tests/self_governance.rs`).
//!
//! **One declared source, three projections.** An adopter writes one [`Constitution`] carrying
//! every dimension's boundaries; the static and semantic dimensions project as a CI exit code,
//! and 漏刻 projects both as a CI exit code (its probe-coverage audit, composed here) and, in
//! the adopter's binary, as a runtime event (the prod face, consumed directly via [`louke`]).
//! The runtime boundaries declared here are the same objects the adopter hands to
//! [`louke::install`] at startup — the single source of truth, two faces.

#![deny(missing_docs)]

mod runner;

pub use guibiao::{
    Boundary, BoundaryKind, CrateBoundary, CrateBoundaryBuilder, CrateBoundaryDraft, CrateTarget,
    DenyExternalDraft, DependencyKind, ModuleBoundary, ModuleBoundaryBuilder, ModuleBoundaryDraft,
    ModuleRule, ModuleTargetDraft, Outcome, Report, Rule, Severity, Violation, ViolationId, check,
    workspace_member_src_dirs,
};
// The static 圭表 (gnomon) constitution — the static dimension's own declaration, reached under
// its instrument name so the bare `Constitution` can be the unified shell-level type below. The
// pure static core (`guibiao::check`) takes this type; the self-governance gate uses it directly.
pub use guibiao::Constitution as GnomonConstitution;
// 渾儀 (semantic) dimension: the boundary DSL, re-exported so an adopter declares semantic
// boundaries the same way as static ones, then folds them into the unified [`Constitution`].
// `SemanticBoundaries` stays public (the runner reads it) but is off the prelude declaration path.
pub use hunyi::{
    DynTraitBoundary, DynTraitBoundaryDraft, DynTraitCrateDraft, DynTraitModuleDraft,
    ForbiddenMarkerBoundary, ForbiddenMarkerBoundaryDraft, ForbiddenMarkerCrateDraft,
    ForbiddenMarkerModuleDraft, SemanticBoundaries, SemanticBoundary, SemanticBoundaryDraft,
    SemanticCrateDraft, SemanticModuleDraft, TraitImplBoundary, TraitImplBoundaryDraft,
    TraitImplCrateDraft, TraitImplTraitDraft, VisibilityBoundary, VisibilityBoundaryDraft,
    VisibilityCrateDraft, VisibilityModuleDraft, check as check_semantic, check_all,
    check_dyn_trait, check_forbidden_marker, check_trait_impl_locality, check_visibility,
};
// 漏刻 (runtime) dimension DSL: declared here, then projected two ways — the CI probe-coverage
// audit (composed by [`run`]) and the prod face (the adopter calls [`louke::install`] /
// `assert_boundary!` directly; the `#[macro_export]` macros live at the `louke` root).
pub use louke::{
    OriginEntry, Posture, RuntimeBoundary, RuntimeBoundaryDraft, RuntimeSeamDraft, Tracked,
    audit_probe_coverage,
};

pub use runner::{constitution_markdown, run};

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
    pub fn signature_boundary(mut self, boundary: SemanticBoundary) -> Self {
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
pub mod prelude {
    pub use super::{
        Boundary, BoundaryKind, Constitution, CrateBoundary, DependencyKind, DynTraitBoundary,
        ForbiddenMarkerBoundary, ModuleBoundary, Outcome, Report, Rule, RuntimeBoundary,
        SemanticBoundary, Severity, TraitImplBoundary, Violation, ViolationId, VisibilityBoundary,
        check, run,
    };
}
