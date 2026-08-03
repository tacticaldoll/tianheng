//! Structural model and rules for guibiao.

macro_rules! boundary_common {
    ($boundary:ty) => {
        impl $boundary {
            /// The crate this boundary governs.
            pub fn crate_package(&self) -> &str {
                &self.crate_package
            }

            /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
            /// project's governance, distinct from the free-text `reason`. Optional; a boundary
            /// with none projects and reacts exactly as before.
            pub fn with_anchor(mut self, anchor: &str) -> Self {
                self.anchor = Some(anchor.to_string());
                self
            }

            /// The durable governance anchor recorded with the boundary, if any.
            pub fn anchor(&self) -> Option<&str> {
                self.anchor.as_deref()
            }

            /// The boundary's severity (`enforce` or `warn`).
            pub fn severity(&self) -> xuanji::Severity {
                self.severity
            }
        }
    };
}

macro_rules! draft_common {
    ($draft:ty) => {
        impl $draft {
            /// Make this an advisory (`warn`) boundary: violations are reported but do not fail
            /// the reaction — the first rung of adoption.
            pub fn warn(mut self) -> Self {
                self.severity = xuanji::Severity::Warn;
                self
            }
        }
    };
}

/// Constitution model.
pub mod constitution;
/// Crate boundary rules.
pub mod crate_rule;
/// Module boundary rules.
pub mod module_rule;

pub use constitution::*;
pub use crate_rule::*;
pub use module_rule::*;
