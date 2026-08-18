//! The shared typed refusal returned by repository judgements.
//!
//! A disagreement and an input that cannot be judged demand different operator actions, so the distinction is
//! carried in one Rust type even though both ultimately fail a `cargo test` gate. Focused failure matrices own
//! the observable behavior; constructing this value has no process-global side effect.

/// Which of the two facts a refusal carries, since they demand different operator actions.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    /// The source disagrees with what it is judged against.
    Violation,
    /// The source could not be read, which is not the same fact.
    CannotJudge,
}

/// Which branch produced a refusal, or that it came from outside the register's corpus.
///
/// **Two states rather than a sentinel, because a sentinel let a false sentence be written.** This was a
/// `&'static str` with the empty string standing for *outside the corpus*, and the doc on it then claimed
/// that nothing could construct a refusal unable to say which branch produced it — which `violation("…")`
/// does, seventeen times, while [`violation`]'s own doc correctly calls that pair a deliberate corpus
/// boundary. One file, two mutually exclusive statements, and the false one on the field a reader meets
/// first. The type refuses to hold that sentence now, which is the same repair a sibling struct's `package`
/// field was given one cycle earlier.
///
/// What is actually true is narrower and is held by a run rather than by this comment: **no construction
/// under `crates/kanhe/src` lacks an identity**, which `no_refusal_site_is_untriaged` asserts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    /// The registered identity of the branch: `<capability>#<slug>`.
    Registered(&'static str),
    /// Constructed outside the register's corpus — a gate under `crates/kanhe/tests`, where the judgement
    /// and the directions over it share a file. A declared bound, with its own tracker.
    OutsideRegister,
}

/// One repository judgement's refusal: which branch produced it, what kind of fact it is, and what to tell
/// the operator.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// The branch this came from, where the register's corpus reaches it.
    pub site: Site,
    /// Whether the source disagreed, or could not be judged at all.
    pub kind: Kind,
    /// What the operator is told, which is the whole of what a refusal delivers.
    pub message: String,
}

/// A violation from a site the refusal register does not cover.
///
/// **Not the migration's leftover — a corpus boundary.** The register reads `crates/kanhe/src`, and several
/// gates are implemented in `crates/kanhe/tests` where the judgement and its directions share a file. Those
/// constructions carry no identity because nothing yet asks them to, and the register says so rather than
/// counting a corpus it does not read. Every site under `src` carries one, which is what
/// `no_refusal_site_is_untriaged` holds.
pub fn violation(message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::OutsideRegister,
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A cannot-judge from a site the refusal register does not cover — see [`violation`].
pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::OutsideRegister,
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}

/// A violation from a registered site.
pub fn violation_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Registered(site),
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A cannot-judge from a registered site.
pub fn cannot_judge_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Registered(site),
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}

/// Assert that `refusal` came from `site`, which is how a direction cites the branch it observed.
///
/// The citation is executable rather than textual: a direction naming a site it did not reach fails here,
/// and the register separately refuses a registered site no direction names. Reading the message for a
/// distinctive phrase is what this replaces — it could not tell a branch that was never exercised from one
/// whose wording had moved.
///
/// # Panics
///
/// When the refusal came from a different site, or from one not yet registered.
pub fn expect(site: &'static str, refusal: &Refusal) {
    assert_eq!(
        refusal.site,
        Site::Registered(site),
        "this direction cites a site the refusal it observed did not come from: {}",
        refusal.message
    );
}
