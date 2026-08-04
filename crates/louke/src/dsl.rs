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

/// An origin registration produced by [`crate::register_origin!`] — a type's identity, its
/// **observed** origin (the module the type is *defined* in), and its name (for findings). Every field
/// is **derived from the type**, so a registration cannot present an origin the type does not have.
/// Pass these to [`crate::install`].
#[derive(Debug, Clone)]
pub struct OriginEntry {
    pub(crate) type_id: TypeId,
    pub(crate) origin: &'static str,
    pub(crate) type_name: &'static str,
}

/// The module a type is defined in, taken from the type's own reported path — the origin. Not the
/// caller's to choose, which is the whole point: a type's path is a property of the type.
///
/// The generic argument list is cut **first**. A type's own arguments can contain path separators
/// (`Repo<std::string::String>`), so searching for the final `::` before removing them would land
/// inside the arguments and report a module the type has nothing to do with. The first `<` in the
/// rendering is necessarily the top-level one, since nesting can only begin after it opens.
///
/// A shape with no path at all — a primitive, a reference, a tuple — yields its own rendering
/// unchanged. That is a stated bound rather than an error: such an origin matches no allowlist entry,
/// so the crossing reacts fail-closed with the observed value named in the finding, which is the safe
/// direction and needs no separate gate.
pub(crate) fn defining_module(type_path: &'static str) -> &'static str {
    let head = match type_path.find('<') {
        Some(open) => &type_path[..open],
        None => type_path,
    };
    match head.rfind("::") {
        Some(cut) => &head[..cut],
        None => head,
    }
}

impl OriginEntry {
    /// **Not a supported constructor — [`crate::register_origin!`]'s expansion target.** Hidden from
    /// the documented surface and named so a hand-written call reads as what it is.
    ///
    /// It must stay `pub`: a `macro_rules!` expands at its *call site*, so everything the macro names
    /// has to be reachable from there — `pub(crate)` here would break every legitimate
    /// `register_origin!` in an adopter's crate, which is a real Rust rule rather than an oversight. A
    /// proc-macro would not change that: it is expanded into the caller's crate and resolved there
    /// too, so a private constructor fails with `E0603` at the adopter's own call (verified with a
    /// three-crate probe). A macro form has no privilege its caller lacks.
    ///
    /// That no longer matters, because being reachable is not the same as being *usable to lie*. This
    /// takes **no arguments**: the type identity, the origin, and the type name are all derived from
    /// `T`. A hand-written call is therefore possible and pointless — the only entry it can build for a
    /// type is the honest one, naming the module that type is defined in. Naming someone else's type
    /// produces that type's correct registration, which a second registration of the same type then
    /// rejects as a duplicate. An origin a type does not have is unrepresentable rather than detected.
    ///
    /// The derivation lives here, where `T` is still a type parameter, because it cannot live anywhere
    /// later: no reverse lookup from a `TypeId` back to a path exists, so by the time [`crate::install`]
    /// sees an entry the type is gone.
    #[doc(hidden)]
    pub fn __from_register_origin<T: 'static>() -> Self {
        let type_name = std::any::type_name::<T>();
        OriginEntry {
            type_id: TypeId::of::<T>(),
            origin: defining_module(type_name),
            type_name,
        }
    }
}
