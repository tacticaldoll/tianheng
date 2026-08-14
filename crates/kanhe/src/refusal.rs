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

/// One repository judgement's refusal: what kind of fact it is, and what to tell the operator.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// Whether the source disagreed, or could not be judged at all.
    pub kind: Kind,
    /// What the operator is told, which is the whole of what a refusal delivers.
    pub message: String,
}

/// A refusal recording that the source disagrees with what it is judged against.
pub fn violation(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::Violation,
        message: message.into(),
    }
}

/// A refusal recording that the source could not be read, which is not the same fact as disagreement.
pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}
