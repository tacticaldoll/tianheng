//! Where a reaction's measure deliberately stops — the typed model of a declared observation bound.
//!
//! A **declared observation bound** is a claim that an observation stops at a named shape, so that shape is
//! governed policy rather than a defect. It is the most consequential sentence this family writes, because it
//! reads as *permission*: it tells a future auditor that something which looks like an escape is intended.
//!
//! Not to be confused with [`ScanDepth`](crate::ScanDepth), which lives one module over on purpose. That is
//! how far a scan **walks** — a knob an adopter turns on a boundary. This is where the **measure stops**, and
//! it renders no verdict. Filing the two apart is how the confusion would have survived.
//!
//! # Why the types are nested
//!
//! [`Extent`] is `OutOfReach | Reached(..)` rather than a flat enum with a direction field. A shape the
//! observation source never saw has **nowhere** to record how the reaction treated it, so the contradiction
//! "never observed it, and it over-reacts" cannot be written. That contradiction is not hypothetical: a
//! backlog entry once predicted a false negative for a `#[cfg_attr(pred, path=…)]` remap where reproduction
//! found a fail-loud constitution error, and that entry's own recorded lesson is that the risk class is what
//! decides urgency.
//!
//! The value set was read out of the family's own declarations rather than designed. Two pairs a tidier model
//! would have merged are kept apart because their adopter consequences are opposite — see
//! [`Reached::RefusesToJudge`] against [`Reached::DeclinesToRefuse`], and [`Reached::UnderReacts`], which
//! alone carries an [`Owner`].

use std::borrow::Cow;

/// A declared bound's identity: `<capability>/<scenario-slug>`, derived from where the bound is declared.
///
/// Owned-or-borrowed: a literal id borrows and allocates nothing, which is what every one of this family's own
/// declarations is, while an implementor whose ids are discovered rather than written can still name them. It
/// said "never allocated" while the type was `&'static str`, and that mandated a declaration the caller had no
/// way to produce. The slug is the declaring scenario's heading, lowercased, with each run of non-alphanumerics
/// collapsed to one hyphen and the ends trimmed.
///
/// The form is **not** validated here. A malformed id simply matches no declared scenario, and the reaction
/// holding the two sets equal names it — one check rather than two that could disagree about what is well
/// formed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BoundId(Cow<'static, str>);

impl BoundId {
    /// The id as written in the declaring spec's derived form.
    ///
    /// Takes anything convertible, so a literal reads as it always did and a computed id — an observer over a
    /// discovered plugin set, or over roots it scanned — is expressible at all. Before this it was not: the type
    /// was `&'static str`, which mandated a declaration and admitted only one written by hand.
    pub fn new(id: impl Into<Cow<'static, str>>) -> Self {
        Self(id.into())
    }

    /// The underlying `<capability>/<scenario-slug>` text, borrowed from this id.
    ///
    /// `&str` rather than `&'static str`: an owned-or-borrowed value can only honestly lend for as long as it
    /// lives, and nothing holds one beyond its declaration.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for BoundId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// One declared observation bound: its identity, the shape it stops at, and where the measure stops.
///
/// The declaring spec scenario states the bound for a *reader*; this states what kind of stop it is for a
/// *reaction*. Neither alone is the declaration — a scenario with no declaration is an unclassified claim, and
/// a declaration with no scenario is a classification no spec reader can find.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoundDecl {
    id: BoundId,
    shape: Cow<'static, str>,
    extent: Extent,
    pinned_by: Cow<'static, str>,
}

impl BoundDecl {
    /// Declare a bound.
    ///
    /// `shape` names what the bound stops at, in the declaring scenario's terms. `pinned_by` is the test that
    /// defends it — what that test must *demonstrate* is not a parameter, because [`Extent::demonstrates`]
    /// already determines it and a second copy of one fact can disagree with the first.
    pub fn new(
        id: BoundId,
        shape: impl Into<Cow<'static, str>>,
        extent: Extent,
        pinned_by: impl Into<Cow<'static, str>>,
    ) -> Self {
        Self {
            id,
            shape: shape.into(),
            extent,
            pinned_by: pinned_by.into(),
        }
    }

    /// The declared id, matched against the declaring spec's derived id.
    pub fn id(&self) -> &BoundId {
        &self.id
    }

    /// What the bound stops at.
    pub fn shape(&self) -> &str {
        &self.shape
    }

    /// Where the measure stops.
    pub const fn extent(&self) -> &Extent {
        &self.extent
    }

    /// The name of the test that defends this bound.
    pub fn pinned_by(&self) -> &str {
        &self.pinned_by
    }
}

/// Where a reaction's measure stops for one declared shape.
///
/// Nested rather than flat so that a shape the observation source never reached has no place to carry a claim
/// about how the reaction treated it.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Extent {
    /// The observation source never sees the shape — text stripped before scanning, source outside the
    /// corpus, a body behind an unexpanded macro, a name the resolver cannot reach.
    ///
    /// This is the one extent that can hide a false negative *without* the reaction having had a choice, and
    /// it carries no owner for that reason: nothing is owed for a shape nothing observes by design.
    OutOfReach {
        /// Why the source does not reach it.
        because: Cow<'static, str>,
    },
    /// The observation source sees the shape. What the reaction then does is the nested question.
    Reached(Reached),
}

impl Extent {
    /// What this bound's pinning test must demonstrate.
    ///
    /// Derived rather than declared: an extent already determines the direction of its own evidence, and a
    /// field beside it would be a second copy of one fact that could contradict the first.
    pub const fn demonstrates(&self) -> Demonstrates {
        match self {
            Extent::OutOfReach { .. } => Demonstrates::DoesNotReact,
            Extent::Reached(reached) => reached.demonstrates(),
        }
    }

    /// Whether this bound is a declared false negative — a reaction that fires less than the truth.
    ///
    /// The one direction this family treats as a defect, so the projection leads with the count of these.
    pub const fn is_declared_false_negative(&self) -> bool {
        matches!(self, Extent::Reached(Reached::UnderReacts { .. }))
    }

    /// The projection label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Extent::OutOfReach { .. } => "out of reach",
            Extent::Reached(reached) => reached.as_str(),
        }
    }
}

/// What a reaction does with a shape it *did* see.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Reached {
    /// Refuses to give a verdict — exit 2 — rather than guess.
    ///
    /// Fail-loud, so it can never be a silent pass. Kept distinct from [`Reached::DeclinesToRefuse`]: one
    /// withholds a verdict, the other gives one while stepping over something, and an adopter meets them
    /// differently.
    ///
    /// **No declared bound uses this today**, and that is stated rather than taken as a reason to drop it. The
    /// misclassification this whole model exists to prevent was exactly a confusion between this and
    /// [`Extent::OutOfReach`]: a backlog entry predicted a silent false negative for a `#[cfg_attr]` path remap
    /// where the real behaviour was a fail-loud refusal, and it drove urgency the wrong way. A direction that
    /// cannot be *named* cannot be predicted with, so the value earns its place from the prediction side rather
    /// than from a current instance.
    RefusesToJudge {
        /// Why a verdict cannot be given here.
        because: Cow<'static, str>,
    },
    /// Deliberately does *not* refuse — continues past a shape that could have been a scan error.
    ///
    /// The mirror of [`Reached::RefusesToJudge`], and a real declaration rather than a symmetry: one declared
    /// bound records that a cfg-gated module with an absent file is skipped rather than failing the gate.
    DeclinesToRefuse {
        /// Why continuing is preferred to erroring.
        because: Cow<'static, str>,
    },
    /// Reacts more than the truth. The safe direction; the cost is a false positive an adopter must dismiss.
    OverReacts {
        /// Why the reaction is deliberately wider than the shape.
        because: Cow<'static, str>,
    },
    /// Reacts *less* than the truth — a declared false negative.
    ///
    /// The one direction this family treats as a defect, which is why it is the only extent that must name an
    /// [`Owner`]: a declared false negative with nobody responsible for closing it is how one outlives its
    /// reason.
    UnderReacts {
        /// Why the reaction stops short.
        because: Cow<'static, str>,
        /// Who must act if this is ever to close.
        owner: Owner,
    },
    /// Reacts correctly by not reacting: the shape is genuinely not a violation, and the bound exists only so a
    /// reader does not misread the silence as an escape.
    ///
    /// Distinct from [`Extent::OutOfReach`] in the direction that matters — the reaction *saw* the shape and was
    /// *right* — and distinct from [`Reached::AsIntended`] because nothing is bounded at all. Three declared
    /// bounds are exactly this: `pub use … as _` binds no nameable path a consumer can reach, and a `mod` or a
    /// plain item inside a function body is unreachable as `crate::…`.
    NotAViolation {
        /// Why the shape is genuinely not a violation.
        because: Cow<'static, str>,
    },
    /// Reacts exactly as intended. What is bounded is the *granularity of the fact*, not the reaction.
    ///
    /// One declared bound is precisely this: two trait objects differing only in an unrenderable sub-node
    /// share one identity, and each still reacts on first occurrence — only baseline-dedup granularity is
    /// bounded. An extent implying the reaction were limited would misreport a working one.
    AsIntended {
        /// Which part of the fact is bounded. Carried only here: no declared bound is both out of reach and
        /// granularity-limited, so offering this on every extent would invite a combination nothing exhibits.
        bounded: FactGranularity,
        /// Why the granularity stops where it does.
        because: Cow<'static, str>,
    },
}

impl Reached {
    /// What a bound with this extent must demonstrate. See [`Extent::demonstrates`].
    pub const fn demonstrates(&self) -> Demonstrates {
        match self {
            Reached::RefusesToJudge { .. } => Demonstrates::RefusesToJudge,
            Reached::DeclinesToRefuse { .. } => Demonstrates::DoesNotRefuse,
            Reached::OverReacts { .. } => Demonstrates::ReactsOnHarmlessShape,
            Reached::UnderReacts { .. } => Demonstrates::DoesNotReact,
            Reached::NotAViolation { .. } => Demonstrates::DoesNotReact,
            Reached::AsIntended { .. } => Demonstrates::CollapsesGranularity,
        }
    }

    /// The projection label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Reached::RefusesToJudge { .. } => "refuses to judge",
            Reached::DeclinesToRefuse { .. } => "declines to refuse",
            Reached::OverReacts { .. } => "over-reacts",
            Reached::UnderReacts { .. } => "under-reacts",
            Reached::NotAViolation { .. } => "not a violation",
            Reached::AsIntended { .. } => "as intended, granularity bounded",
        }
    }
}

/// Who must act if a declared false negative is ever to close.
///
/// Carried only by [`Reached::UnderReacts`]. Nothing is owed for a shape nothing observes by design, and an
/// owner field on every extent would be decorative wherever it is not load-bearing.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Owner {
    /// This dimension's own engine. Closing it is ordinary work here.
    Engine,
    /// A layer beneath this dimension, named. Closing it *in this dimension* would fork the layer rather than
    /// fix it, which is why the layer is named rather than implied.
    Inherited {
        /// The layer the bound is inherited from.
        from: Cow<'static, str>,
    },
    /// The adopter, by narrowing their own declaration. One bound says so outright: the engine declines to
    /// guess, and the narrowing is the adopter's to make.
    Adopter,
}

impl Owner {
    /// The projection label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Owner::Engine => "engine",
            Owner::Inherited { .. } => "inherited",
            Owner::Adopter => "adopter",
        }
    }
}

/// Which part of an observed fact a granularity bound limits.
///
/// The *kind* is shared across dimensions; which field it names is stated in the declaring dimension's own
/// spec sentence, because a fact's fields are dimension-owned (三儀 ⊥ 三儀: no shared referent).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum FactGranularity {
    /// Two distinct occurrences share one identity, so baseline granularity is bounded. Each still reacts.
    Identity,
    /// The rendered presentation is bounded; identity is not.
    Presentation,
}

impl FactGranularity {
    /// The projection label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            FactGranularity::Identity => "identity",
            FactGranularity::Presentation => "presentation",
        }
    }
}

/// What a bound's pinning test must demonstrate, derived from its [`Extent`].
///
/// This is what turns a classification into a claim about evidence: the extent does not merely name a kind, it
/// constrains what the defence has to look like.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum Demonstrates {
    /// The reaction does not fire on the shape.
    DoesNotReact,
    /// The reaction fires on a shape that is not really a violation.
    ReactsOnHarmlessShape,
    /// The reaction refuses to judge — exit 2.
    RefusesToJudge,
    /// The reaction continues rather than erroring on a shape that could have errored.
    DoesNotRefuse,
    /// Two distinct occurrences collapse to one finding.
    CollapsesGranularity,
}

impl Demonstrates {
    /// The projection label.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Demonstrates::DoesNotReact => "does not react",
            Demonstrates::ReactsOnHarmlessShape => "reacts on a harmless shape",
            Demonstrates::RefusesToJudge => "refuses to judge",
            Demonstrates::DoesNotRefuse => "does not refuse",
            Demonstrates::CollapsesGranularity => "collapses granularity",
        }
    }
}
