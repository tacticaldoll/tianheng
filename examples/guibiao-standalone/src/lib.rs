//! A tiny hexagonal app, governed by 圭表 (guibiao) **alone** — the syn-free static import
//! linter. The rule: `domain` stays pure and must not import `infra`. This example
//! deliberately breaks that rule so the reaction is visible; `tests/reaction.rs` asserts it,
//! and the `demo` binary renders it.
//!
//! The whole dependency footprint is `guibiao = "0.2"` — no `syn` — which *is* the product
//! pitch: adopt the static dimension on its own, light.
pub mod domain;
pub mod governance;
pub mod infra;
