//! 漏刻 as an observation participant.

use std::path::Path;

use xingbiao::audit_corpus_and_anchor;
use xuanji::{BoundDecl, Observer, Outcome};

use crate::{RuntimeBoundary, audit_probe_coverage, observation_bounds};

/// The runtime dimension as an [`Observer`].
#[derive(Debug, Clone)]
pub struct RuntimeObserver {
    declared: Vec<RuntimeBoundary>,
}

impl RuntimeObserver {
    /// Observe with the declared runtime seams — the authoritative set, never a source scan.
    pub fn new(declared: Vec<RuntimeBoundary>) -> Self {
        Self { declared }
    }
}

impl Observer for RuntimeObserver {
    /// Derives its own corpus and label anchor, as the composed shell does today.
    ///
    /// The anchor must be Cargo's resolved `workspace_root`, because the audit labels every observed file
    /// relative to it and that label is **baseline identity**: it is the one directory that moves neither with
    /// the checkout location nor with the workspace's member set. The fallback exists only for metadata carrying
    /// no such field.
    fn observe(&self, manifest_path: &Path) -> Outcome {
        match audit_corpus_and_anchor(manifest_path) {
            Ok((roots, anchor)) => audit_probe_coverage(&self.declared, &roots, &anchor),
            Err(message) => Outcome::ConstitutionError(format!(
                "cannot read workspace '{}': {message}",
                manifest_path.display()
            )),
        }
    }

    fn bounds(&self) -> Vec<BoundDecl> {
        observation_bounds()
    }
}
