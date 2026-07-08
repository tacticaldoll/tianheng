//! The composed example: a small hexagonal app governed by **all 三儀** through the 天衡
//! (tianheng) shell — the funnel target. It carries one fault per instrument:
//!
//! - **圭表 (static)** — `domain` imports `infra` (an inward-only breach).
//! - **渾儀 (semantic)** — `api` exposes `infra::DbPool` on its public surface (a leak).
//! - **漏刻 (runtime)** — a `rogue` adapter with an un-blessed origin crosses the port seam.
//!
//! The static and semantic faults react at CI time (`bin/check`, `tests/funnel.rs`); the runtime
//! fault reacts at runtime, in a running binary, against a live object (`bin/runtime_demo`,
//! `tests/runtime.rs`) — which is why the runtime dimension lives here, in the composed example,
//! never as a standalone on-ramp.
pub mod adapters;
pub mod api;
pub mod domain;
pub mod governance;
pub mod infra;
pub mod port;
