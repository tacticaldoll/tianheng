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

/// One repository judgement's refusal: which branch produced it, what kind of fact it is, and what to tell
/// the operator.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// The branch this came from, as its registered identity: `<capability>#<slug>`.
    ///
    /// **Not optional, because the compiler is what holds this.** For the length of the migration a second
    /// pair of constructors took no site and a projection counted what had not moved; the count reached zero
    /// and they are gone. Two constructors for one rule was the cost, carried visibly and paid off, and
    /// nothing can now construct a refusal that cannot say which branch produced it.
    pub site: &'static str,
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
        site: "",
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A cannot-judge from a site the refusal register does not cover — see [`violation`].
pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        site: "",
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}

/// A violation from a registered site.
pub fn violation_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site,
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A cannot-judge from a registered site.
pub fn cannot_judge_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site,
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
        refusal.site, site,
        "this direction cites a site the refusal it observed did not come from: {}",
        refusal.message
    );
}
