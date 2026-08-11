//! The channel a gate reports its refusal class on, separate from everything else it prints.
//!
//! A wrapper standing in front of an irreversible act must tell a **disagreement** from an input it
//! **could not read**, because the two demand different operator actions. The judgement already types that in
//! [`crate::refusal::Kind`]; what was missing was a way to carry it across a process boundary.
//!
//! **The first attempt read it out of the gate's stdout, and that was the wrong channel twice over.** The gate
//! panicked as `merge message (Violation): …` and the wrapper grepped for `(Violation)`, which put the
//! parentheses in the shell and the variant name in Rust — two owners for one token, the shape this repository
//! spent a window replacing. Measured: changing the gate's format string to `merge message: {:?} — {}` left every
//! direction green while the wrapper's pattern matched nothing, so every violation would have reported as
//! unjudged. And the stream it searched carries arbitrary tooling output besides — a compile error, a harness
//! message — so a class could in principle be read from text no judgement wrote. That second one was **latent**,
//! not live: inducing a compile error on the rendering line produced no match, which is recorded because it is
//! the difference between a defect and a shape.
//!
//! So the class travels on its own channel. The gate writes it to a path the wrapper names, and the wrapper reads
//! that file rather than searching prose. Nothing is spelled twice: [`ENV`] is the variable name both sides use,
//! and the class is [`crate::refusal::Kind`]'s own rendering. A gate that did not reach a verdict writes nothing,
//! so *absent* means unjudged by construction rather than by a default the wrapper has to remember.

use std::path::Path;

use crate::refusal::Kind;

/// The environment variable naming the file a gate writes its refusal class to.
///
/// One constant, read by the gates and asserted against the wrappers, so the name cannot drift.
pub const ENV: &str = "TIANHENG_GATE_VERDICT";

/// The class as it travels: [`Kind`]'s own rendering, so there is no second spelling to keep in step.
pub fn rendered(kind: Kind) -> String {
    format!("{kind:?}")
}

/// Report `kind` on the channel, if the caller opened one.
///
/// Called by a gate at the moment it has a verdict and before it fails, so the file exists exactly when a
/// judgement was reached. A gate run outside a wrapper sets no variable and writes nothing.
///
/// A write that fails is **not** reported as a verdict: the wrapper reads an absent or unreadable file as
/// unjudged, which is the direction that errs toward telling an operator to look at the output rather than at
/// their message. Returning the outcome lets a direction assert that, rather than inferring it from a file that
/// is missing for either reason.
pub fn report(kind: Kind) -> bool {
    let Some(path) = std::env::var_os(ENV) else {
        return false;
    };
    std::fs::write(Path::new(&path), rendered(kind)).is_ok()
}
