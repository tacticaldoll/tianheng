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
    /// Delegates to the dimension's composed entry point, which reads its own workspace metadata
    /// when boundaries require observation — 三儀 ⊥ 三儀: no scanner is threaded in from a sibling.
    fn observe(&self, manifest_path: &Path) -> Outcome {
        check_all(&self.boundaries, manifest_path)
    }

    fn bounds(&self) -> Vec<BoundDecl> {
        observation_bounds()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_boundaries_are_clean_without_reading_a_manifest() {
        let observer = SemanticObserver::new(SemanticBoundaries::default());
        let absent = std::env::temp_dir().join(format!(
            "tianheng-empty-semantic-observer-{}-does-not-exist/Cargo.toml",
            std::process::id()
        ));

        assert!(matches!(observer.observe(&absent), Outcome::Clean));
    }
}
