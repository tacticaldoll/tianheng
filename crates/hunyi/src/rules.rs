//! Canonical rule labels — the single source per semantic rule.
//!
//! Each rule's label is written **once**, here, and referenced by both the `check`-side
//! `Violation::new(...)` (the reaction, via each capability module) and the `list` projections in
//! the 天衡 shell (`tianheng` depends on `hunyi`, so importing these is the allowed direction).
//! Editing a label in one place updates every projection — the `list`/`check` and text/JSON drift
//! this closes. These are the rule *family* strings; a per-boundary operand detail (e.g. the
//! dyn/impl-trait operand set) stays a parameter layered on at projection.

/// Signature-coupling: a module's public API must not expose a forbidden type.
pub const SIGNATURE_RULE: &str = "must not expose";
/// Dyn-trait: a module's public API must not expose trait-object (`dyn`) syntax.
pub const DYN_TRAIT_RULE: &str = "must not expose dyn";
/// Impl-trait: a module's public API must not return a written `impl Trait` (RPIT).
pub const IMPL_TRAIT_RULE: &str = "must not expose impl trait";
/// Async-exposure: a module's public API must not declare an `async fn`.
pub const ASYNC_EXPOSURE_RULE: &str = "must not expose async fn";
/// Trait-impl-locality: a trait may be implemented only in its declared location(s).
pub const TRAIT_IMPL_RULE: &str = "must only be implemented in the declared location(s)";
/// Visibility, `Crate` ceiling (the `must_not_declare_pub` sugar): no bare-`pub` items. Kept
/// verbatim so the sugar's findings and baselines never churn.
pub const VISIBILITY_RULE: &str = "must not declare pub items";
/// Visibility, `Super` ceiling: nothing more visible than `pub(super)`.
pub const VISIBILITY_SUPER_RULE: &str = "must not declare items more visible than pub(super)";
/// Visibility, `Module` ceiling: nothing more visible than module-private.
pub const VISIBILITY_MODULE_RULE: &str = "must not declare items more visible than module-private";
/// Forbidden-marker: a subtree's types must not acquire a forbidden trait.
pub const FORBIDDEN_MARKER_RULE: &str = "must not acquire trait";
/// Unsafe-confinement: `unsafe` is confined to the declared subtree(s).
pub const UNSAFE_CONFINEMENT_RULE: &str = "unsafe is confined to the declared subtree(s)";
