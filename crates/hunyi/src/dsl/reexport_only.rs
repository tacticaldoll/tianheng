//! Re-export-only module boundary declaration.

use xuanji::{RuleKey, ScanDepth, Severity};

/// A module whose direct items are `use` declarations. At subtree depth, child modules are
/// containers and each child is governed by the same rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReexportOnlyBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
    pub(crate) depth: ScanDepth,
}

impl ReexportOnlyBoundary {
    /// Stable rule identity, distinct from visibility ceilings.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of::<_, &str, &str>("tianheng.rule/hunyi/reexport-only-module", [])
    }

    /// Begin a re-export-only boundary in the named package.
    pub fn in_crate(package: &str) -> ReexportOnlyCrateDraft {
        ReexportOnlyCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The governed module path.
    pub fn module(&self) -> &str {
        &self.module
    }
    /// The reason supplied with this boundary.
    pub fn reason(&self) -> &str {
        &self.reason
    }
    /// The declared observation depth.
    pub fn scan_depth(&self) -> ScanDepth {
        self.depth
    }
}

crate::dsl::boundary_common!(ReexportOnlyBoundary, ReexportOnlyBoundaryDraft);

/// A package boundary awaiting its module.
#[doc(hidden)]
pub struct ReexportOnlyCrateDraft {
    crate_package: String,
}
impl ReexportOnlyCrateDraft {
    /// Name the module to govern.
    pub fn module(self, module: &str) -> ReexportOnlyModuleDraft {
        ReexportOnlyModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module boundary awaiting its rule.
#[doc(hidden)]
pub struct ReexportOnlyModuleDraft {
    crate_package: String,
    module: String,
}
impl ReexportOnlyModuleDraft {
    /// Require every direct item to be a `use` declaration.
    pub fn must_declare_only_reexports(self) -> ReexportOnlyBoundaryDraft {
        ReexportOnlyBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            severity: Severity::Enforce,
            depth: ScanDepth::Shallow,
        }
    }
}

/// A rule awaiting its reason, with optional depth and severity.
#[doc(hidden)]
pub struct ReexportOnlyBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
    depth: ScanDepth,
}
impl ReexportOnlyBoundaryDraft {
    /// Choose shallow or whole-subtree observation.
    pub fn depth(mut self, depth: ScanDepth) -> Self {
        self.depth = depth;
        self
    }
    /// Finish the boundary with its repair direction.
    pub fn because(self, reason: &str) -> ReexportOnlyBoundary {
        ReexportOnlyBoundary {
            crate_package: self.crate_package,
            module: self.module,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
            depth: self.depth,
        }
    }
}
