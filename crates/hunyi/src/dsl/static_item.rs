//! Static-item boundary declaration DSL.

use xuanji::{RuleKey, Severity};

/// A static-item boundary: a module, and every module beneath it, declares no `static` item and no
/// `thread_local!`. Declared in Rust (the single source of truth), composed at the gate.
///
/// It governs *declarations*: a `static` or `static mut` item wherever it is written in the subtree
/// — at module level, in a function body, closure or initializer — a foreign `static` in an
/// `extern` block, and each static a `thread_local!` invocation declares. What a call does to
/// process-global state (`std::env::set_var`, a registry a dependency keeps) is not a declaration
/// here; confining such calls is `must_not_call_inline`'s.
///
/// **Scope.** The whole subtree of the anchored module, always: a static declared anywhere beneath
/// the anchor is state the anchored layer holds, so there is no seam-only depth. Anchoring at
/// `crate` governs the whole crate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) reason: String,
    pub(crate) anchor: Option<String>,
    pub(crate) severity: Severity,
}

impl StaticBoundary {
    /// Stable semantic identity for this static-item rule.
    pub fn rule_key(&self) -> RuleKey {
        RuleKey::of::<_, &str, &str>("tianheng.rule/hunyi/static-item", [])
    }

    /// Begin a static-item boundary in the crate named `package`.
    pub fn in_crate(package: &str) -> StaticCrateDraft {
        StaticCrateDraft {
            crate_package: package.to_string(),
        }
    }

    /// The governed module path (e.g. `crate::core`).
    pub fn module(&self) -> &str {
        &self.module
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

crate::dsl::boundary_common!(StaticBoundary, StaticBoundaryDraft);

/// A static-item boundary awaiting its module anchor.
#[doc(hidden)]
pub struct StaticCrateDraft {
    crate_package: String,
}

impl StaticCrateDraft {
    /// Anchor the boundary to a module path within the crate (e.g. `crate::core`).
    /// Written from the crate root — `crate` or `crate::a::b`, a raw identifier read as its plain
    /// form; any other spelling, or a module the crate does not declare, is a constitution error (exit 2).
    pub fn module(self, module: &str) -> StaticModuleDraft {
        StaticModuleDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module-anchored static-item boundary awaiting the rule.
#[doc(hidden)]
pub struct StaticModuleDraft {
    crate_package: String,
    module: String,
}

impl StaticModuleDraft {
    /// Forbid the module and its whole subtree from declaring a `static` item or a `thread_local!`.
    ///
    /// A `thread_local!` is recognized by its name — `thread_local!`, `std::thread_local!`,
    /// `::std::thread_local!` and `r#thread_local!` alike — and a crate that renames it
    /// (`use std::thread_local as tls;`) cannot be judged, so the boundary refuses it (exit 2) and
    /// asks for the macro to be written by its name.
    pub fn must_not_declare_static(self) -> StaticBoundaryDraft {
        StaticBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            severity: Severity::Enforce,
        }
    }
}

/// A boundary awaiting severity (optional) and its reason.
#[doc(hidden)]
pub struct StaticBoundaryDraft {
    crate_package: String,
    module: String,
    severity: Severity,
}

impl StaticBoundaryDraft {
    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> StaticBoundary {
        StaticBoundary {
            crate_package: self.crate_package,
            module: self.module,
            reason: reason.to_string(),
            anchor: None,
            severity: self.severity,
        }
    }
}
