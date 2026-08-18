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

/// Whether a refusal names the site that produced it.
///
/// **Identity lived only in the message, which is why nothing could measure this.** `AGENTS.md` states that
/// a guard is not a guard until it has been seen to fail, and `repository-checks` requires every refusal to
/// have been run against a tree carrying the shape it refuses. Nothing held the second half: a refusal
/// message is a *template* and a direction asserts a *rendering* of it, and no textual predicate bridges
/// those — five were written and measured, and each was wrong in a different direction, over the same
/// corpus. Whether a branch was observed is a question about running a program, exactly as `pin_bites` says
/// of whether a test bites.
///
/// So the site travels in the value. A direction that observed this refusal names the same site, and the
/// two are compared by running rather than by reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    /// The registered identity of the branch that produced this refusal: `<capability>#<slug>`.
    ///
    /// `#` rather than `/`, which the bound register already resolves as a reference to a declared
    /// observation bound — the opposite fact, about what is *not* observed.
    ///
    /// Registering a site is a commitment that a direction observes it — the register refuses a registered
    /// site no direction names — so the migration cannot outrun the coverage it exists to measure.
    Named(&'static str),
    /// A site not yet migrated. The register counts these and requires the count to fall, never rise.
    Unregistered,
}

/// One repository judgement's refusal: which branch produced it, what kind of fact it is, and what to tell
/// the operator.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// The branch this came from, where that branch has been registered.
    pub site: Site,
    /// Whether the source disagreed, or could not be judged at all.
    pub kind: Kind,
    /// What the operator is told, which is the whole of what a refusal delivers.
    pub message: String,
}

/// A refusal recording that the source disagrees with what it is judged against.
///
/// The unregistered form. It exists for the length of the migration and no longer: when the register's
/// count of unregistered sites reaches zero this and its sibling are deleted, and the two constructors
/// become one again. Two constructors for one rule is the shape this repository closes, and it is carried
/// here deliberately and visibly rather than by rewriting every site in one unreadable change.
pub fn violation(message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Unregistered,
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A refusal recording that the source could not be read, which is not the same fact as disagreement.
///
/// The unregistered form — see [`violation`].
pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Unregistered,
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}

/// A violation from a registered site.
pub fn violation_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Named(site),
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A cannot-judge from a registered site.
pub fn cannot_judge_at(site: &'static str, message: impl Into<String>) -> Refusal {
    Refusal {
        site: Site::Named(site),
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
    let met = match refusal.site {
        Site::Named(met) if met == site => return,
        Site::Named(met) => format!("came from {met:?}"),
        Site::Unregistered => "came from a site that carries no identity yet".to_string(),
    };
    panic!(
        "this direction cites {site:?}, and the refusal it observed {met}: {}",
        refusal.message
    );
}
