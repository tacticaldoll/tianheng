//! The adapters that cross the port seam. `blessed` is the declared-allowed origin; `rogue` is
//! not in the allowlist — at runtime, crossing the seam with a `rogue` object reacts fail-closed.
//!
//! Each adapter registers its origin **inside its own module**, because `register_origin!`
//! captures the registration site's `module_path!()` — the origin is *where the type is
//! registered*, not a free label. `louke::Tracked` is implemented automatically for any
//! `'static` type (a blanket impl), so an adapter earns it for free.
use louke::{register_origin, OriginEntry};

use crate::port::Adapter;

/// The blessed adapter — the one declared origin allowed to cross the seam.
pub mod blessed {
    use super::*;

    /// An adapter authored in the blessed module.
    pub struct BlessedAdapter;

    impl Adapter for BlessedAdapter {
        fn handle(&self) {}
    }

    /// Register this adapter's origin — captured as `composed_app::adapters::blessed`.
    pub fn origin() -> OriginEntry {
        register_origin!(BlessedAdapter)
    }
}

/// The rogue adapter — a real impl of the port, but its origin is not on the allowlist.
pub mod rogue {
    use super::*;

    /// An adapter authored in the rogue module (not a blessed origin).
    pub struct RogueAdapter;

    impl Adapter for RogueAdapter {
        fn handle(&self) {}
    }

    /// Register this adapter's origin — captured as `composed_app::adapters::rogue`.
    pub fn origin() -> OriginEntry {
        register_origin!(RogueAdapter)
    }
}
