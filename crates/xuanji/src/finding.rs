//! Dimension-agnostic structured identity and human finding text.

use crate::StructuredFactIdentity;

/// Pair human-readable finding text with stable structured identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub(crate) text: String,
    pub(crate) fact: StructuredFactIdentity,
}

impl Finding {
    /// Pair human finding text with a dimension-owned stable observed-fact identity.
    pub fn new(text: impl Into<String>, fact: StructuredFactIdentity) -> Self {
        Self {
            text: text.into(),
            fact,
        }
    }

    /// Human-readable finding text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Structured observed-fact identity.
    pub fn fact(&self) -> &StructuredFactIdentity {
        &self.fact
    }

    /// Compatibility alias for [`Finding::fact`].
    ///
    /// New code should use `fact` so the same concept has the same name on [`Finding`] and
    /// [`crate::Violation`]. This alias remains source-compatible for existing consumers.
    pub fn key(&self) -> &StructuredFactIdentity {
        self.fact()
    }
}
