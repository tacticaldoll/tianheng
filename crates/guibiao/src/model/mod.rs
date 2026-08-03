//! Structural model and rules for guibiao.

/// Constitution model.
pub mod constitution;
/// Crate boundary rules.
pub mod crate_rule;
/// Module boundary rules.
pub mod module_rule;

pub use constitution::*;
pub use crate_rule::*;
pub use module_rule::*;
