//! Building the repositories this crate's gates are judged against — one owner, apart from the judgements.
//!
//! **A gate and the fixture that demonstrates it are two jobs.** `release_coherence_gate` and
//! `publish_source_gate` each held both, each behind a banner comment marking the seam, and
//! `hermetic_git` held a third piece of the same job beside its invocation forms. A banner is a
//! reader's note about a boundary; a module is the boundary. What the split buys is a place where the
//! shared primitives can live without either gate owning them — the state before it was that a helper
//! written for one fixture was reached for by the other only after someone noticed.
//!
//! **The direction runs one way and is meant to.** A fixture writes a shape its gate declares, so
//! `release_coherence` reads that gate's compare-link constant; no gate reads anything here. The
//! judgement modules are the authority on the shapes, and these builders are consumers of them.
//!
//! **What stayed behind, and why it is not an exception.** `hermetic_git::fixture` asserts success
//! because its caller is building rather than judging — but that is a distinction among *invocation
//! forms*, which is what that module is: `run` trims for a caller reading a value, `run_exact` keeps
//! the bytes for one comparing content, `run_with_stdin` delivers records, and `fixture` panics. Moving
//! one member out would put two modules in charge of how a `git` is invoked here, which is the twin
//! drift this crate spends itself closing. `FIXTURE_DAY` stayed with it for the reason its own doc
//! gives: the owner of a stamp is the thing that stamps it, and `fixture` is what writes the date onto
//! a commit.

use std::path::{Path, PathBuf};

use crate::hermetic_git::fixture as run;

pub mod publish_source;
pub mod release_coherence;

/// Write a fixture file, creating the directories above it.
///
/// **One owner, because the two builders wrote a file two ways.** This was private to the
/// release-coherence builder while its sibling spelled `std::fs::write(...).expect(...)` inline — the
/// shape where one copy gains a repair and the other keeps the defect, which is what the parent
/// directory creation here already is: the inline spelling had none, so a fixture naming a nested path
/// depended on some earlier call having made the directory.
///
/// # Panics
///
/// As [`crate::hermetic_git::fixture`] does: this builds a fixture rather than judging one.
pub fn write(path: PathBuf, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the fixture directory is writable");
    }
    std::fs::write(path, body).expect("the fixture file is writable");
}

/// Stage everything in a fixture and commit it under one subject.
///
/// **Both builders spell one act, so one of them owns it.** Each was left writing `git add .` and then a
/// commit while one of the two had already extracted exactly this helper for itself — an extraction held
/// by one module and not its sibling, which is the shape where a repair reaches one copy. It was invisible
/// until the layer above it, the shared command builder, converged.
///
/// It sits here rather than beside that builder because the distinction it carries is about a fixture's
/// content — a subject, and staging everything under it — and not about how a `git` is invoked.
///
/// # Panics
///
/// As [`crate::hermetic_git::fixture`] does: this builds a subject rather than judging one.
pub fn commit(repo: &Path, subject: &str) {
    run(repo, "git", &["add", "."]);
    run(repo, "git", &["commit", "-qm", subject]);
}
