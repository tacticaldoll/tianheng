//! The **sans-I/O purity** composed profile — a shell-level convenience that folds two existing
//! reactions into one declaration. It adds no new observation and no new requirement: it composes
//! the 圭表 `must_not_call_inline` and 渾儀 `must_not_expose_async_fn` boundaries an adopter could
//! write by hand. Because it spans two dimensions it lives in the shell (三儀 ⊥ 三儀: a dimension
//! never imports its sibling; only 天衡 composes them).

use crate::{AsyncExposureBoundary, Constitution, ModuleBoundary, Severity};

/// A **sans-I/O purity** profile: the two source-observable axes of a pure kernel composed into one
/// declaration — the core reads no ambient clock, and its public API is synchronous. Passed to
/// [`Constitution::sans_io_pure`], it expands into a 圭表 `must_not_call_inline` boundary and a 渾儀
/// `must_not_expose_async_fn` boundary on the module. Both reach the module's **subtree** (the clock
/// rule is inherently subtree-wide over the filesystem; the async rule opts in via
/// `including_submodules`, descending the declared mod-tree), so a pure kernel is sans-I/O
/// *throughout*, not only at its own seam — though a `#[path]`-remapped module is the semantic
/// dimension's stated bound (the async half does not descend one).
///
/// **Honestly scoped — this is not the whole of sans-I/O purity.** It governs the *clock* and
/// *async* axes only; a core that must also avoid ambient `fs` / `net` / `env` adds those explicitly
/// (e.g. `must_not_call_inline("std::fs")`, `confine_external_crate(...)`). Nothing is baked in — you
/// supply the time prefix and read verbs via [`reading_clock_via`](SansIoPureModuleDraft::reading_clock_via) —
/// so a second consumer's clock-marker needs cannot silently diverge from a frozen default set.
///
/// The builder is fluent so each argument is anchored by its own method: a flat positional `&str`
/// list would let a caller silently swap the confinement prefix for the reason, producing a
/// boundary that resolves nothing and never reacts — the one forbidden false negative the whole DSL
/// exists to prevent.
///
/// ```no_run
/// use tianheng::prelude::*;
///
/// let _ = Constitution::new("pacta").sans_io_pure(
///     SansIoPure::in_crate("pacta-contract")
///         .module("crate::kernel")
///         .reading_clock_via("std::time", ["now"])
///         .because("the kernel stays sans-I/O: no ambient clock, synchronous API"),
/// );
/// ```
#[derive(Debug, Clone)]
pub struct SansIoPure {
    crate_package: String,
    module: String,
    time_prefix: String,
    read_verbs: Vec<String>,
    severity: Severity,
    reason: String,
}

impl SansIoPure {
    /// Begin a sans-I/O purity profile in the crate named `package`.
    pub fn in_crate(package: &str) -> SansIoPureCrateDraft {
        SansIoPureCrateDraft {
            crate_package: package.to_string(),
        }
    }
}

/// A profile awaiting its module anchor.
pub struct SansIoPureCrateDraft {
    crate_package: String,
}

impl SansIoPureCrateDraft {
    /// Anchor the profile to a module path within the crate (e.g. `crate::kernel`). Both composed
    /// boundaries govern this one module.
    pub fn module(self, module: &str) -> SansIoPureModuleDraft {
        SansIoPureModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored profile awaiting its clock-read declaration.
pub struct SansIoPureModuleDraft {
    crate_package: String,
    module: String,
}

impl SansIoPureModuleDraft {
    /// Declare the clock-read surface to forbid: an inline call resolving under `time_prefix` whose
    /// terminal segment is one of `read_verbs` (the canonical set is `"std::time"` + `["now"]`,
    /// covering `SystemTime::now` / `Instant::now`). You own any false negative from omitting a verb;
    /// an empty `read_verbs` is a constitution error (exit 2), inherited from the underlying
    /// `must_not_call_inline` rule — never a silent no-op.
    pub fn reading_clock_via<I, S>(self, time_prefix: &str, read_verbs: I) -> SansIoPureDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        SansIoPureDraft {
            crate_package: self.crate_package,
            module: self.module,
            time_prefix: time_prefix.to_string(),
            read_verbs: read_verbs.into_iter().map(Into::into).collect(),
            severity: Severity::Enforce,
        }
    }
}

/// A profile awaiting its (optional) severity and its reason.
pub struct SansIoPureDraft {
    crate_package: String,
    module: String,
    time_prefix: String,
    read_verbs: Vec<String>,
    severity: Severity,
}

impl SansIoPureDraft {
    /// Make both composed boundaries advisory (`warn`): violations are reported but do not fail the
    /// reaction — the first rung of adoption. Applies to the clock and async boundaries alike.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the profile with its human-readable reason (the repair hint), carried by both composed
    /// boundaries — the composite is one intent; each boundary's own rule text distinguishes what it
    /// observes.
    pub fn because(self, reason: &str) -> SansIoPure {
        SansIoPure {
            crate_package: self.crate_package,
            module: self.module,
            time_prefix: self.time_prefix,
            read_verbs: self.read_verbs,
            severity: self.severity,
            reason: reason.to_string(),
        }
    }
}

impl Constitution {
    /// Declare a [`SansIoPure`] profile: compose the two source-observable axes of a pure kernel —
    /// the core reads no ambient clock (圭表 `must_not_call_inline`) and its public API is
    /// synchronous (渾儀 `must_not_expose_async_fn`) — onto one module. Shell-composed (三儀 ⊥ 三儀):
    /// the two dimensions never see each other; the shell folds them.
    ///
    /// A convenience over declaring the two boundaries by hand; it adds no new reaction. See
    /// [`SansIoPure`] for the honest scope (clock + async only) and the misorder-proof fluent shape.
    pub fn sans_io_pure(self, profile: SansIoPure) -> Self {
        let SansIoPure {
            crate_package,
            module,
            time_prefix,
            read_verbs,
            severity,
            reason,
        } = profile;

        // 圭表 — the core reads no ambient clock; time is injected, not read.
        let clock = ModuleBoundary::in_crate(&crate_package)
            .module(&module)
            .must_not_call_inline(&time_prefix)
            .ending_with(read_verbs.iter().cloned());
        let clock = if severity == Severity::Warn {
            clock.warn()
        } else {
            clock
        };

        // 渾儀 — the public seam is synchronous; async lives at the edges. Subtree-scoped: a public
        // `async fn` anywhere under the module's public mod-tree reacts, so a pure kernel is sans-I/O
        // *throughout*, not only at its own seam (a seam-only async guard would silently miss a
        // submodule's async fn). Both halves
        // reach the subtree, but by different observation — the clock half over the filesystem, the
        // async half over the declared mod-tree — so a `#[path]`-remapped module is the semantic
        // dimension's stated bound here (the async half does not descend one), not full parity.
        let sync_api = AsyncExposureBoundary::in_crate(&crate_package)
            .module(&module)
            .must_not_expose_async_fn()
            .including_submodules();
        let sync_api = if severity == Severity::Warn {
            sync_api.warn()
        } else {
            sync_api
        };

        self.boundary(clock.because(&reason))
            .async_exposure_boundary(sync_api.because(&reason))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::constitution_markdown;

    // The two boundaries `sans_io_pure` must expand into, written by hand — the faithful-composition
    // reference. `sans_io_pure(...)` must be byte-identical to this (proven via the projection).
    fn hand_composed(prefix: &str, verbs: &[&str], warn: bool) -> Constitution {
        let clock = ModuleBoundary::in_crate("pacta-contract")
            .module("crate::kernel")
            .must_not_call_inline(prefix)
            .ending_with(verbs.iter().copied());
        let clock = if warn { clock.warn() } else { clock };
        let sync_api = AsyncExposureBoundary::in_crate("pacta-contract")
            .module("crate::kernel")
            .must_not_expose_async_fn()
            .including_submodules();
        let sync_api = if warn { sync_api.warn() } else { sync_api };
        Constitution::new("pacta")
            .boundary(clock.because("the kernel stays sans-I/O"))
            .async_exposure_boundary(sync_api.because("the kernel stays sans-I/O"))
    }

    fn via_profile(prefix: &str, verbs: &[&str], warn: bool) -> Constitution {
        let profile = SansIoPure::in_crate("pacta-contract")
            .module("crate::kernel")
            .reading_clock_via(prefix, verbs.iter().copied());
        let profile = if warn { profile.warn() } else { profile };
        Constitution::new("pacta").sans_io_pure(profile.because("the kernel stays sans-I/O"))
    }

    #[test]
    fn sans_io_pure_composes_faithfully() {
        // The profile expands to exactly the hand-composed clock + async pair (every
        // reaction-affecting field is in the projection, so projection-equality ⇒ object-equality).
        assert_eq!(
            constitution_markdown(&via_profile("std::time", &["now"], false)),
            constitution_markdown(&hand_composed("std::time", &["now"], false)),
        );
    }

    #[test]
    fn sans_io_pure_threads_severity_to_both() {
        // `warn` threads to BOTH composed boundaries (matches the hand-composed both-warn pair)…
        assert_eq!(
            constitution_markdown(&via_profile("std::time", &["now"], true)),
            constitution_markdown(&hand_composed("std::time", &["now"], true)),
        );
        // …and severity is really carried, not a no-op: the warn projection differs from enforce.
        assert_ne!(
            constitution_markdown(&via_profile("std::time", &["now"], true)),
            constitution_markdown(&via_profile("std::time", &["now"], false)),
        );
    }

    #[test]
    fn sans_io_pure_bakes_no_defaults() {
        // Non-canonical prefix/verbs must thread through unchanged — guarding the design decision
        // that nothing is baked in. An implementation hardcoding `std::time`/`["now"]` (the canonical
        // set the other tests use) would pass those yet fail here.
        assert_eq!(
            constitution_markdown(&via_profile("quanta::clock", &["fetch", "raw"], false)),
            constitution_markdown(&hand_composed("quanta::clock", &["fetch", "raw"], false)),
        );
    }
}
