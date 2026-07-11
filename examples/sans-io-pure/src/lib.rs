//! A small kernel governed by the 天衡 shell's **`sans_io_pure`** profile — the two source-
//! observable axes of a sans-I/O core folded into one declaration: it reads no ambient clock
//! (圭表), and its public API is synchronous **throughout its subtree** (渾儀).
//!
//! `crate::kernel` deliberately breaks both axes:
//! - it calls `std::time::SystemTime::now()` inline — an ambient clock read (the 圭表 half); and
//! - its **submodule** `crate::kernel::inner` exposes a `pub async fn` — which only the subtree-
//!   scoped async half catches (a seam-only guard would miss a submodule). This is what makes
//!   `sans_io_pure`'s `including_submodules` opt-in load-bearing.
//!
//! `tests/reaction.rs` asserts both axes react; `bin/check` folds them into one exit code.
pub mod governance;
pub mod kernel;
