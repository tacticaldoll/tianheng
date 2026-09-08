//! The **no-existential-leak** composed profile — a shell-level convenience that folds two
//! existing reactions into one declaration. It adds no new reaction: it composes the 渾儀
//! `must_not_expose_impl_trait` and `must_not_expose_async_fn` boundaries an adopter could write by
//! hand. Both are 渾儀 capabilities (not sibling dimensions), but the composition still lives here
//! rather than in `hunyi`: `Constitution` — the only place multiple boundaries assemble — is a
//! `tianheng` type, and `hunyi`'s self-law forbids depending on `tianheng` at all.

use crate::{AsyncExposureBoundary, Constitution, ImplTraitBoundary, Severity};

/// A **no-existential-leak** profile: impl-trait's written `-> impl Trait` (RPIT) and
/// async-exposure's implicit, compiler-inserted `impl Future` are two distinct syntactic signals
/// for the same underlying leak — an unnameable existential type escaping a public seam. Passed to
/// [`Constitution::no_existential_leak`], it expands into an `ImplTraitBoundary`
/// `must_not_expose_impl_trait` and an `AsyncExposureBoundary` `must_not_expose_async_fn` on the
/// module, each keeping its own separate identity — two distinct causes remain two distinct,
/// separately-baseline-able findings. Both composed boundaries reach the module's **subtree**
/// unconditionally (not an adopter choice), so "no existential leak" holds throughout the anchored
/// module, not only at its own seam — mirroring [`Constitution::sans_io_pure`]'s own subtree
/// discipline.
///
/// **Honestly scoped — this is not every existential form.** It governs a *written* `impl Trait`
/// return and an `async fn`'s implicit `impl Future` only; argument-position `impl Trait` (APIT,
/// universal, not existential) and any operand-scoping (forbidding only a *named* trait's shape)
/// are out of scope — compose [`ImplTraitBoundary`]'s operand-scoped `must_not_expose_impl_trait_of`
/// by hand if you need that depth.
///
/// ```no_run
/// use tianheng::prelude::*;
///
/// let _ = Constitution::new("pacta").no_existential_leak(
///     NoExistentialLeak::in_crate("pacta-contract")
///         .module("crate::core")
///         .because("the core seam names concrete types; no unnameable existential leaks"),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct NoExistentialLeak {
    crate_package: String,
    module: String,
    severity: Severity,
    reason: String,
}

impl NoExistentialLeak {
    /// Begin a no-existential-leak profile in the crate named `package`.
    pub fn in_crate(package: &str) -> NoExistentialLeakCrateDraft {
        NoExistentialLeakCrateDraft {
            crate_package: package.to_string(),
        }
    }
}

/// A profile awaiting its module anchor.
pub struct NoExistentialLeakCrateDraft {
    crate_package: String,
}

impl NoExistentialLeakCrateDraft {
    /// Anchor the profile to a module path within the crate (e.g. `crate::core`). Both composed
    /// boundaries govern this module's whole subtree.
    pub fn module(self, module: &str) -> NoExistentialLeakModuleDraft {
        NoExistentialLeakModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
            severity: Severity::Enforce,
        }
    }
}

/// A module-anchored profile awaiting its (optional) severity and its reason.
pub struct NoExistentialLeakModuleDraft {
    crate_package: String,
    module: String,
    severity: Severity,
}

impl NoExistentialLeakModuleDraft {
    /// Make both composed boundaries advisory (`warn`): violations are reported but do not fail the
    /// reaction — the first rung of adoption. Applies to the impl-trait and async-exposure
    /// boundaries alike.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the profile with its human-readable reason (the repair hint), carried by both
    /// composed boundaries — the composite is one intent; each boundary's own rule text
    /// distinguishes what it observes.
    pub fn because(self, reason: &str) -> NoExistentialLeak {
        NoExistentialLeak {
            crate_package: self.crate_package,
            module: self.module,
            severity: self.severity,
            reason: reason.to_string(),
        }
    }
}

impl Constitution {
    /// Declare a [`NoExistentialLeak`] profile: compose impl-trait's written `-> impl Trait` (RPIT)
    /// and async-exposure's implicit `impl Future` — both existential-leak signals — onto one
    /// module's whole subtree. Shell-composed: the two boundaries never share identity; the shell
    /// only folds their declaration.
    ///
    /// A convenience over declaring the two boundaries by hand; it adds no new reaction. See
    /// [`NoExistentialLeak`] for the honest scope (no APIT, no operand-scoping) and the
    /// misorder-proof fluent shape.
    pub fn no_existential_leak(self, profile: NoExistentialLeak) -> Self {
        let NoExistentialLeak {
            crate_package,
            module,
            severity,
            reason,
        } = profile;

        // 渾儀 — a written `-> impl Trait` (RPIT) leaks an unnameable existential.
        let impl_trait = ImplTraitBoundary::in_crate(&crate_package)
            .module(&module)
            .must_not_expose_impl_trait()
            .including_submodules();
        let impl_trait = if severity == Severity::Warn {
            impl_trait.warn()
        } else {
            impl_trait
        };

        // 渾儀 — an `async fn`'s compiler-inserted `impl Future` is the same leak, implicit form.
        // Subtree-scoped unconditionally (not an adopter choice) — exactly like `sans_io_pure`
        // hardcodes its own async half — so there is only one place either half's subtree reach
        // could silently drift out of sync with the other.
        let async_exposure = AsyncExposureBoundary::in_crate(&crate_package)
            .module(&module)
            .must_not_expose_async_fn()
            .including_submodules();
        let async_exposure = if severity == Severity::Warn {
            async_exposure.warn()
        } else {
            async_exposure
        };

        self.impl_trait_boundary(impl_trait.because(&reason))
            .async_exposure_boundary(async_exposure.because(&reason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constitution_markdown;

    // The two boundaries `no_existential_leak` must expand into, written by hand — the faithful-
    // composition reference. `no_existential_leak(...)` must be byte-identical to this (proven via
    // the projection), mirroring `sans_io.rs`'s own `hand_composed`/`via_profile` test shape.
    fn hand_composed(warn: bool) -> Constitution {
        let impl_trait = ImplTraitBoundary::in_crate("pacta-contract")
            .module("crate::core")
            .must_not_expose_impl_trait()
            .including_submodules();
        let impl_trait = if warn { impl_trait.warn() } else { impl_trait };
        let async_exposure = AsyncExposureBoundary::in_crate("pacta-contract")
            .module("crate::core")
            .must_not_expose_async_fn()
            .including_submodules();
        let async_exposure = if warn {
            async_exposure.warn()
        } else {
            async_exposure
        };
        Constitution::new("pacta")
            .impl_trait_boundary(impl_trait.because("no existential leaks from the core seam"))
            .async_exposure_boundary(
                async_exposure.because("no existential leaks from the core seam"),
            )
    }

    fn via_profile(warn: bool) -> Constitution {
        let profile = NoExistentialLeak::in_crate("pacta-contract").module("crate::core");
        let profile = if warn { profile.warn() } else { profile };
        Constitution::new("pacta")
            .no_existential_leak(profile.because("no existential leaks from the core seam"))
    }

    #[test]
    fn no_existential_leak_composes_faithfully() {
        // The profile expands to exactly the hand-composed impl-trait + async-exposure pair (every
        // reaction-affecting field is in the projection, so projection-equality ⇒ object-equality).
        assert_eq!(
            constitution_markdown(&via_profile(false)),
            constitution_markdown(&hand_composed(false)),
        );
    }

    #[test]
    fn no_existential_leak_threads_severity_to_both() {
        // `warn` threads to BOTH composed boundaries (matches the hand-composed both-warn pair)…
        assert_eq!(
            constitution_markdown(&via_profile(true)),
            constitution_markdown(&hand_composed(true)),
        );
        // …and severity is really carried, not a no-op: the warn projection differs from enforce.
        assert_ne!(
            constitution_markdown(&via_profile(true)),
            constitution_markdown(&via_profile(false)),
        );
    }

    #[test]
    fn no_existential_leak_hardcodes_subtree_scope_on_both_halves() {
        // Neither half's subtree opt-in is adopter-visible on `NoExistentialLeak` itself; the
        // profile hardcodes it on both. If a future edit silently dropped `.including_submodules()`
        // from either half, the projection would stop matching a hand-composed reference that keeps
        // it on both — this test is that reaction.
        let markdown = constitution_markdown(&via_profile(false));
        assert!(
            markdown.contains("including submodules") || markdown.contains("including_submodules"),
            "{markdown}"
        );
        let without_impl_trait_subtree = constitution_markdown(
            &Constitution::new("pacta")
                .impl_trait_boundary(
                    ImplTraitBoundary::in_crate("pacta-contract")
                        .module("crate::core")
                        .must_not_expose_impl_trait()
                        .because("no existential leaks from the core seam"),
                )
                .async_exposure_boundary(
                    AsyncExposureBoundary::in_crate("pacta-contract")
                        .module("crate::core")
                        .must_not_expose_async_fn()
                        .including_submodules()
                        .because("no existential leaks from the core seam"),
                ),
        );
        assert_ne!(
            markdown, without_impl_trait_subtree,
            "the profile's impl-trait half must also carry subtree scope, not only its async half"
        );
    }
}
