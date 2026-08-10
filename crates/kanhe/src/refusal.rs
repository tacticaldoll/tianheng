//! The shared typed refusal returned by repository judgements.
//!
//! A disagreement and an input that cannot be judged demand different operator actions, so the distinction is
//! carried in one Rust type even though both ultimately fail a `cargo test` gate. Focused failure matrices own
//! the observable behavior; constructing this value has no process-global side effect.

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    /// The source disagrees with what it is judged against.
    Violation,
    /// The source could not be read, which is not the same fact.
    CannotJudge,
}

#[derive(Debug, Clone)]
pub struct Refusal {
    pub kind: Kind,
    pub message: String,
}

pub fn violation(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::Violation,
        message: message.into(),
    }
}

pub fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}
