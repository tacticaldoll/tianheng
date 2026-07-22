//! Dimension-agnostic structured identity and human finding text.

use crate::StructuredFactIdentity;

/// Pair human-readable finding text with stable structured identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub(crate) text: String,
    pub(crate) key: StructuredFactIdentity,
}

impl Finding {
    /// Pair human finding text with dimension-owned stable key.
    pub fn new(text: impl Into<String>, key: StructuredFactIdentity) -> Self {
        Self {
            text: text.into(),
            key,
        }
    }

    /// Human-readable finding text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Stable structured identity key.
    pub fn key(&self) -> &StructuredFactIdentity {
        &self.key
    }
}
