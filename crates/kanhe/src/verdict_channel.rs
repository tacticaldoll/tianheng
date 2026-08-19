//! The channel a gate reports **whether it reached a verdict, and which**, separate from everything else it prints.
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

use crate::refusal::{Kind, Refusal};

/// The environment variable naming the file a gate writes its refusal class to.
///
/// One constant, read by the gates and asserted against the wrappers, so the name cannot drift.
pub const ENV: &str = "TIANHENG_GATE_VERDICT";

/// The class as it travels: [`Kind`]'s own rendering, so there is no second spelling to keep in step.
pub fn rendered(kind: Kind) -> String {
    format!("{kind:?}")
}

/// The rendering a **clean** verdict travels as.
///
/// A gate that judged and agreed writes this. Nothing wrote anything before, so a wrapper's success path had
/// no positive evidence a verdict had been reached at all — `require_one_pass` answered *a test passed*,
/// which is a different question and is satisfied by a harness that returned without judging.
pub const CLEAN: &str = "Clean";

/// What a gate harness reached, and **the only way one exits**.
///
/// Three separate exits is what this replaces, each of which had to remember to report its class before
/// failing. Remembering is what a construction removes: every arm of this enum is a value, so a harness
/// cannot leave without producing one, and [`deliver`] is the single place that decides what each means.
///
/// The direction that guarded the old shape could only reach one exit — it located the `Err(refusal)` arm by
/// substring and asserted the report preceded the panic *within it* — so any other exit owed nothing. That is
/// how a subject supplied as unreadable bytes left through a clean return.
#[derive(Debug)]
pub enum Verdict {
    /// The act this gate stands in front of is not being made, so there is nothing to judge. Carries what to
    /// tell a reader who ran the suite rather than the wrapper.
    NotAsked(String),
    /// A verdict was reached and the subject holds.
    Clean(String),
    /// A verdict was reached and the subject disagrees, or could not be read.
    Refused(Refusal),
}

/// What the channel must carry for `verdict`, and `None` where **no verdict was reached**.
///
/// Pure, and separated from [`deliver`] for the reason every split in this crate is: the channel is a file a
/// wrapper names through the process environment, which a parallel test run shares, so a direction asserting
/// what gets written could not otherwise construct the cases.
pub fn reached(verdict: &Verdict) -> Option<String> {
    match verdict {
        Verdict::NotAsked(_) => None,
        Verdict::Clean(_) => Some(CLEAN.to_string()),
        Verdict::Refused(refusal) => Some(rendered(refusal.kind)),
    }
}

/// Whether `verdict` must fail the run.
pub fn refuses(verdict: &Verdict) -> bool {
    matches!(verdict, Verdict::Refused(_))
}

/// Report `verdict` on the channel and fail where it refuses — the single exit of a gate harness.
///
/// `gate` names the judgement in the diagnostic, which is the one thing the two harnesses do not share.
///
/// # Panics
///
/// Where `verdict` refuses, **after** the channel has been written — and where the channel was opened and
/// could not be written, because that is a verdict this run reached and lost. [`reached`] and [`refuses`] are
/// what a direction holds the first pairing with, rather than reading this body.
pub fn deliver(gate: &str, verdict: Verdict) {
    if let Some(class) = reached(&verdict) {
        // **A write that fails is not an absence.** The predecessor of this function returned the write's
        // outcome and said why: *"Returning the outcome lets a direction assert that, rather than inferring
        // it from a file that is missing for either reason."* Routing every gate through one exit dropped
        // that return, and absence acquired a second cause — a verdict reached and lost. `absent means
        // unjudged` is what this module and `repository-checks` both claim **by construction**, and it was
        // true again only while nothing could fail silently between the two.
        //
        // The cost was a lost distinction rather than an unsafe pass: a `Refused` whose write failed reached
        // the wrapper as exit 2 instead of exit 1 — the class collapse this module exists to end, through a
        // different door.
        if let Delivery::Failed(why) = report_reached(&class) {
            panic!(
                "{gate}: reached a verdict and could not write it to the channel ({why}), so the class this \
                 run found cannot travel. That is not the same fact as a run that judged nothing"
            );
        }
    }
    match verdict {
        Verdict::NotAsked(why) => eprintln!("{gate}: not judged — {why}"),
        Verdict::Clean(report) => eprintln!("{gate}: {report}"),
        Verdict::Refused(refusal) => {
            panic!("{gate} ({:?}): {}", refusal.kind, refusal.message)
        }
    }
}

/// What became of a write to the channel.
///
/// Three states rather than a `bool`, for the reason every other split in this crate has: *no channel was
/// opened* and *a channel was opened and could not be written* are different facts, and a boolean makes the
/// second unobservable. A gate run in the ordinary suite opens none, which is not a failure; a gate run by a
/// wrapper opens one, and a failure there is a verdict lost.
enum Delivery {
    /// No caller opened a channel — a gate run outside a wrapper, which is the ordinary case.
    NoChannel,
    /// The class is on the channel.
    OnChannel,
    /// A channel was opened and the class could not be put on it.
    Failed(String),
}

/// Write `class` to the channel, if the caller opened one.
fn report_reached(class: &str) -> Delivery {
    let Some(path) = std::env::var_os(ENV) else {
        return Delivery::NoChannel;
    };
    match std::fs::write(Path::new(&path), class) {
        Ok(()) => Delivery::OnChannel,
        Err(err) => Delivery::Failed(format!("{}: {err}", Path::new(&path).display())),
    }
}
