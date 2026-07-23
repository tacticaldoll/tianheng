//! 漏刻 (Lòukè) — the **runtime** observation dimension of Tianheng.
//!
//! Where 圭表 reads imports and 渾儀 reads the AST — both at CI time — 漏刻 reacts at
//! **runtime, in your binary**, against **live objects**: it sees the concrete type behind
//! a `dyn Trait` crossing an architectural seam, which static and semantic analysis structurally
//! cannot.
//!
//! Two faces:
//! - **Prod face.** Declare `RuntimeBoundary::at("seam").only_origins([…])` and [`install`]
//!   it once at startup. Probes read live type origins and react fail-closed.
//! - **CI face** (the non-default `audit` feature). Audits workspace source for `assert_boundary!`
//!   probes to guarantee every declared seam has a probe and every probe a declared seam.
//!
//! Govern by reaction, not instruction.

#![forbid(unsafe_code)]
#![deny(missing_docs)]

pub use xuanji::{
    BoundaryKind, Finding, Outcome, Polarity, Report, RuleKey, ScanDepth, Severity,
    StructuredFactIdentity, Violation, ViolationId,
};

mod dsl;
mod finding;
mod registry;
mod tracked;

// CI face (the non-default `audit` feature): the probe-coverage audit + source scanner, in its
// own module so a prod dependency on louke compiles none of it. The prod face (the declaration
// DSL, the write-once registry, and the fail-closed probe reaction) stays in this root module.
#[cfg(feature = "audit")]
mod audit;
#[cfg(feature = "audit")]
pub use audit::audit_probe_coverage;

// Public re-exports — all previously-public items, same paths as before.
pub use dsl::{OriginEntry, Posture, RuntimeBoundary, RuntimeBoundaryDraft, RuntimeSeamDraft};
pub use registry::{__react, install, set_sink};
pub use tracked::Tracked;

/// The canonical runtime seam-origin rule label — written **once** here and referenced by
/// both the prod reaction (the crate's internal `check_crossing`) and the 天衡 shell's `list`
/// projection (`tianheng` depends on `louke`, so importing this is the allowed direction). Editing it
/// in one place updates every projection. The specific allowed-origin set is a per-boundary
/// detail layered on at each site, not part of this rule-family label.
pub const RUNTIME_SEAM_RULE: &str = "only declared origins may cross the seam";

/// The full runtime seam **rule line** — the canonical [`RUNTIME_SEAM_RULE`] label with the
/// per-boundary allowed-origin set folded in (`… (only origins: A, B)`). Written **once** here and
/// shared by the prod reaction (`check_crossing`'s violation rule) and the 天衡 shell's
/// `list --format text` projection, so the human-readable line the two render never drifts. The JSON
/// projection deliberately keeps the label bare and carries the origins as a separate field, so it
/// does not use this.
pub fn runtime_seam_rule_line(allowed_origins: &[&str]) -> String {
    format!(
        "{RUNTIME_SEAM_RULE} (only origins: {})",
        allowed_origins.join(", ")
    )
}

/// Register a type's **observed** origin: `register_origin!(PostgresRepo)` captures
/// `module_path!()` at the call site (so the origin is *where the type is registered*, not a
/// self-asserted label) and yields an [`OriginEntry`] to pass to [`install`]. Declarative —
/// no proc-macro, no `syn`.
#[macro_export]
macro_rules! register_origin {
    ($ty:ty) => {
        $crate::OriginEntry::new(
            ::std::any::TypeId::of::<$ty>(),
            ::std::module_path!(),
            ::std::any::type_name::<$ty>(),
        )
    };
}

/// Probe a runtime seam: `assert_boundary!("domain-entry", obj)` reads `obj`'s concrete
/// origin (via the [`Tracked`] supertrait on its trait) and reacts fail-closed against the
/// seam's allowlist. `obj` must be a reference to a `dyn Trait` whose trait carries
/// `: louke::Tracked`.
#[macro_export]
macro_rules! assert_boundary {
    ($seam:expr, $obj:expr) => {
        $crate::__react($seam, $crate::Tracked::as_any($obj).type_id())
    };
}

#[cfg(test)]
mod tests;
