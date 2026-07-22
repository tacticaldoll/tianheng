use std::any::TypeId;

use xuanji::{RuleKey, Severity};

/// How a violated boundary reacts in production. `Event` (the default) emits a structured
/// `Violation`; `Panic` additionally aborts — opt-in only, never the default.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Posture {
    /// Emit a `Violation` event to the sink and continue. The default.
    Event,
    /// Emit the event, then panic — opt-in only (`enforce` severity).
    Panic,
}

impl Posture {
    /// A stable lower-case label for projection (`list`/`--format json`).
    pub fn as_str(&self) -> &'static str {
        match self {
            Posture::Event => "event",
            Posture::Panic => "panic",
        }
    }
}

/// A runtime boundary: only the declared **origins** may cross the named **seam**. Declared
/// in Rust (the single source of truth) and installed once at startup; a probe references the
/// seam by name, so the policy lives in this declaration, not at the call site.
#[derive(Debug, Clone)]
pub struct RuntimeBoundary {
    pub(crate) seam: &'static str,
    pub(crate) allowed: Vec<&'static str>,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
    pub(crate) posture: Posture,
    pub(crate) anchor: Option<String>,
}

impl RuntimeBoundary {
    /// Stable semantic identity for the runtime-seam allowlist rule.
    pub fn rule_key(&self) -> RuleKey {
        runtime_rule_key(&self.allowed)
    }

    /// Begin a boundary at the named runtime seam.
    pub fn at(seam: &'static str) -> RuntimeSeamDraft {
        RuntimeSeamDraft { seam }
    }

    /// The governed seam name.
    pub fn seam(&self) -> &str {
        self.seam
    }

    /// The origins allowed to cross the seam.
    pub fn allowed_origins(&self) -> &[&'static str] {
        &self.allowed
    }

    /// The human-readable reason (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional; a boundary with
    /// none projects and reacts exactly as before.
    pub fn with_anchor(mut self, anchor: &str) -> Self {
        self.anchor = Some(anchor.to_string());
        self
    }

    /// The durable governance anchor recorded with the boundary, if any.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// The declared severity. The CI face reacts to a declared-but-unprobed seam at this
    /// severity (a `warn` boundary yields an advisory, not a CI failure).
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The declared production reaction posture (`Event` default, `Panic` opt-in). Exposed so the
    /// `list` projection is faithful — a `panic_on_violation` boundary must not project identically
    /// to a default event-only one.
    pub fn posture(&self) -> Posture {
        self.posture
    }
}

pub(crate) fn runtime_rule_key(allowed: &[&str]) -> RuleKey {
    let mut allowed = allowed.to_vec();
    allowed.sort_unstable();
    allowed.dedup();
    RuleKey::new(
        "tianheng.rule/louke/runtime-seam",
        allowed
            .into_iter()
            .enumerate()
            .map(|(index, origin)| (format!("allowed_origin_{index}"), origin)),
    )
    .expect("canonical allowlist field names are non-empty and unique")
}

/// A boundary awaiting its allowed-origin set.
#[doc(hidden)]
pub struct RuntimeSeamDraft {
    pub(crate) seam: &'static str,
}

impl RuntimeSeamDraft {
    /// Allow only the given origins (origin labels — typically a `module_path!()` captured by
    /// [`crate::register_origin!`]) to cross this seam.
    pub fn only_origins<I>(self, origins: I) -> RuntimeBoundaryDraft
    where
        I: IntoIterator<Item = &'static str>,
    {
        RuntimeBoundaryDraft {
            seam: self.seam,
            allowed: origins.into_iter().collect(),
            severity: Severity::Enforce,
            posture: Posture::Event,
        }
    }
}

/// A boundary awaiting severity/posture (optional) and a reason.
#[doc(hidden)]
pub struct RuntimeBoundaryDraft {
    pub(crate) seam: &'static str,
    pub(crate) allowed: Vec<&'static str>,
    pub(crate) severity: Severity,
    pub(crate) posture: Posture,
}

impl RuntimeBoundaryDraft {
    /// Make this advisory (`warn`): violations are reported but never panic, regardless of
    /// posture — the first rung of adoption.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Opt into panicking on an `enforce`-severity violation (default is event-only).
    pub fn panic_on_violation(mut self) -> Self {
        self.posture = Posture::Panic;
        self
    }

    /// Finish the boundary with its human-readable reason (the repair hint).
    pub fn because(self, reason: &str) -> RuntimeBoundary {
        RuntimeBoundary {
            seam: self.seam,
            allowed: self.allowed,
            reason: reason.to_string(),
            severity: self.severity,
            posture: self.posture,
            anchor: None,
        }
    }
}

/// An origin registration produced by [`crate::register_origin!`] — a `TypeId`, the **observed**
/// origin (`module_path!()` at the registration site), and the type's name (for findings).
/// Pass these to [`crate::install`].
#[derive(Debug, Clone)]
pub struct OriginEntry {
    pub(crate) type_id: TypeId,
    pub(crate) origin: &'static str,
    pub(crate) type_name: &'static str,
}

impl OriginEntry {
    /// Construct an origin entry. Prefer [`crate::register_origin!`], which captures the call-site
    /// `module_path!()` so the origin is observed, not hand-asserted.
    pub fn new(type_id: TypeId, origin: &'static str, type_name: &'static str) -> Self {
        OriginEntry {
            type_id,
            origin,
            type_name,
        }
    }
}
