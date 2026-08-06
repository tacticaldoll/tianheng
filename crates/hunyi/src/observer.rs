//! 渾儀 as an observation participant.

use std::path::Path;

use xuanji::{BoundDecl, Observer, Outcome};

use crate::{SemanticBoundaries, check_all, observation_bounds};

/// The semantic dimension as an [`Observer`].
#[derive(Debug, Clone)]
pub struct SemanticObserver {
    boundaries: SemanticBoundaries,
}

impl SemanticObserver {
    /// Observe with the given semantic boundary bundle.
    pub fn new(boundaries: SemanticBoundaries) -> Self {
        Self { boundaries }
    }
}

impl Observer for SemanticObserver {
    /// Reads its own workspace metadata, as this dimension already does — 三儀 ⊥ 三儀: no shared scanner, so
    /// nothing is threaded in from a sibling.
    fn observe(&self, manifest_path: &Path) -> Outcome {
        check_all(&self.boundaries, manifest_path)
    }

    fn bounds(&self) -> Vec<BoundDecl> {
        observation_bounds()
    }
}
