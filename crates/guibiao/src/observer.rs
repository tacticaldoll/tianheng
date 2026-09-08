//! 圭表 as an observation participant.

use std::path::Path;

use xuanji::{BoundDecl, Observer, Outcome};

use crate::{Constitution, check, observation_bounds};

/// The static dimension as an [`Observer`].
///
/// Holds the static boundaries it was built from, so the protocol's `observe` takes only a manifest path.
#[derive(Debug, Clone)]
pub struct StaticObserver {
    constitution: Constitution,
}

impl StaticObserver {
    /// Observe with the given static boundaries.
    pub fn new(constitution: Constitution) -> Self {
        Self { constitution }
    }
}

impl Observer for StaticObserver {
    /// Delegates to [`check`], the outcome-only face — **not** to `check_and_cover`, whose coverage advisory the
    /// protocol cannot carry and whose second call would read `cargo metadata` twice.
    fn observe(&self, manifest_path: &Path) -> Outcome {
        check(&self.constitution, manifest_path)
    }

    /// Delegation, never a second list: a divergent copy is what the bijection exists to refuse.
    fn bounds(&self) -> Vec<BoundDecl> {
        observation_bounds()
    }
}
