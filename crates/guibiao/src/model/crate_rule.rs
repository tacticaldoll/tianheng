use super::constitution::*;
use crate::cargo_metadata::*;
use serde_json::Value;
use xuanji::{Polarity, RuleKey, Severity};

/// A boundary attached to one crate target, with a human-readable reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateBoundary {
    pub(crate) target: CrateTarget,
    pub(crate) rule: Rule,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
    pub(crate) kind: DependencyKind,
    pub(crate) anchor: Option<String>,
}

impl CrateBoundary {
    /// Begin a crate boundary for the crate named `package`.
    pub fn crate_(package: &str) -> CrateBoundaryBuilder {
        CrateBoundaryBuilder {
            target: CrateTarget {
                package: package.to_string(),
            },
        }
    }

    /// The crate this boundary governs.
    pub fn target(&self) -> &CrateTarget {
        &self.target
    }

    /// The rule the boundary enforces.
    pub fn rule(&self) -> &Rule {
        &self.rule
    }

    /// The human-readable reason recorded with the boundary (the repair hint).
    pub fn reason(&self) -> &str {
        &self.reason
    }

    /// The boundary's severity (`enforce` or `warn`).
    pub fn severity(&self) -> Severity {
        self.severity
    }

    /// The dependency table this boundary observes (`Normal` by default).
    pub fn dependency_kind(&self) -> DependencyKind {
        self.kind
    }

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional; a boundary with
    /// none projects and reacts exactly as before. Chained after [`because`](CrateBoundaryDraft::because).
    pub fn with_anchor(mut self, anchor: &str) -> Self {
        self.anchor = Some(anchor.to_string());
        self
    }

    /// The durable governance anchor recorded with the boundary, if any.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }
}

/// A crate identified by its package name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateTarget {
    /// The crate's package name, as it appears in `cargo metadata`.
    pub package: String,
}

impl CrateTarget {
    /// Create a new crate target for the given package name.
    pub fn new(package: impl Into<String>) -> Self {
        Self {
            package: package.into(),
        }
    }

    /// Access the crate's package name as a string slice.
    pub fn as_str(&self) -> &str {
        &self.package
    }
}

impl AsRef<str> for CrateTarget {
    fn as_ref(&self) -> &str {
        &self.package
    }
}

impl std::fmt::Display for CrateTarget {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.package)
    }
}

/// What a crate boundary forbids. Each variant is a reaction with an observation
/// source in `cargo metadata`; no variant is named for a reaction that does not
/// exist.
///
/// Rules are constructed through [`CrateBoundary::crate_`], not variant struct expressions. A
/// consumer inspecting a rule can match known fields forward-compatibly:
///
/// ```
/// use guibiao::{CrateBoundary, Rule};
///
/// let boundary = CrateBoundary::crate_("core")
///     .forbid_dependency_on(["serde"])
///     .because("core owns no serialization vocabulary");
/// match boundary.rule() {
///     Rule::ForbidDependencyOn { crates, .. } => assert_eq!(crates, &["serde"]),
///     _ => unreachable!(),
/// }
/// ```
///
/// ```compile_fail
/// use guibiao::Rule;
///
/// let _ = Rule::ForbidDependencyOn { crates: vec!["serde".to_string()] };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Rule {
    /// Deny external (registry/git) dependencies, except any named in `allowed`.
    #[non_exhaustive]
    DenyExternalDependencies {
        /// External crate names permitted despite the deny rule.
        allowed: Vec<String>,
    },
    /// Forbid a normal dependency on any of these crates (external or internal).
    #[non_exhaustive]
    ForbidDependencyOn {
        /// The forbidden crate names.
        crates: Vec<String>,
    },
    /// Restrict normal dependencies to a closed allowlist: any normal dependency
    /// (external or internal) whose name is not in `allowed` is a violation. An
    /// empty allowlist forbids every normal dependency.
    #[non_exhaustive]
    RestrictDependenciesTo {
        /// The closed allowlist of permitted dependency names.
        allowed: Vec<String>,
    },
    /// Restrict the target's dependencies on *other workspace members* to a closed
    /// allowlist: any normal dependency on a workspace member not in `allowed` is a
    /// violation, while external dependencies are ignored. Workspace membership is
    /// observed from `cargo metadata`, so a newly added member is governed by default.
    /// An empty allowlist forbids every workspace dependency.
    #[non_exhaustive]
    RestrictWorkspaceDependenciesTo {
        /// The closed allowlist of permitted workspace-member names.
        allowed: Vec<String>,
    },
    /// Restrict the **declared source kinds** of the target's dependencies to a closed
    /// allowlist: any dependency whose classified [`SourceKind`] (from its `cargo
    /// metadata` declared `source`) is not in `allowed` is a violation. The source-kind
    /// counterpart of [`RestrictDependenciesTo`](Rule::RestrictDependenciesTo) (which
    /// governs dependency *names*). An empty allowlist forbids every dependency by
    /// source. Governs the *declared* source, not the resolved one — a `[patch]`/
    /// `replace-with` redirect is not observed (the resolved layer is cargo-deny's
    /// `[sources]` lane, not a Tianheng capability).
    #[non_exhaustive]
    RestrictDependencySourcesTo {
        /// The closed allowlist of permitted declared source kinds.
        allowed: Vec<SourceKind>,
    },
    /// Restrict the **declared features** the target requests on a named dependency
    /// `crate_` to a closed allowlist: any feature in the target's declared set for
    /// `crate_` (its authored `features = [...]`, ∪ the `default` pseudo-feature when
    /// default features are left on) whose name is not in `allowed` is a violation. The
    /// feature-granularity counterpart of
    /// [`RestrictDependenciesTo`](Rule::RestrictDependenciesTo) (which governs dependency
    /// *names*). An empty allowlist forbids the target from declaring **any** feature of
    /// `crate_`, `default` included (i.e. requires `default-features = false` and no
    /// explicit features). Governs the *declared* request, not the resolved/unified
    /// feature set — a feature that `crate_`'s own `[features]` graph or a sibling crate's
    /// unification enables transitively is not chased (declared-not-resolved).
    #[non_exhaustive]
    RestrictFeaturesOf {
        /// The dependency whose declared features are governed (matched by package name).
        crate_: String,
        /// The closed allowlist of permitted feature names (`default` is the pseudo-feature
        /// for default features).
        allowed: Vec<String>,
    },
    /// Forbid the target from declaring specific named features of a dependency `crate_`:
    /// any feature in the target's declared set for `crate_` matching a `forbidden` name is
    /// a violation; a forbidden feature the target does not declare is not. The
    /// feature-granularity counterpart of
    /// [`ForbidDependencyOn`](Rule::ForbidDependencyOn). Forbidding the `default`
    /// pseudo-feature is the way to require `default-features = false`. An empty forbidden
    /// set is a no-op that always reports clean (symmetric with forbidding a crate the
    /// target does not depend on). Governs the *declared* request, not the resolved/unified
    /// feature set (transitive enables are not chased).
    #[non_exhaustive]
    ForbidFeaturesOf {
        /// The dependency whose declared features are governed (matched by package name).
        crate_: String,
        /// The forbidden feature names (`default` is the pseudo-feature for default features).
        forbidden: Vec<String>,
    },
}

impl Rule {
    /// Stable semantic identity for this declared crate rule.
    pub fn key(&self) -> RuleKey {
        match self {
            Rule::DenyExternalDependencies { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/deny-external-dependencies",
                [("allowed", canonical_set(allowed))],
            ),
            Rule::ForbidDependencyOn { crates } => RuleKey::of(
                "tianheng.rule/guibiao/forbid-dependency-on",
                [("crates", canonical_set(crates))],
            ),
            Rule::RestrictDependenciesTo { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-dependencies-to",
                [("allowed", canonical_set(allowed))],
            ),
            Rule::RestrictWorkspaceDependenciesTo { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-workspace-dependencies-to",
                [("allowed", canonical_set(allowed))],
            ),
            Rule::RestrictDependencySourcesTo { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-dependency-sources-to",
                [(
                    "allowed",
                    canonical_set(allowed.iter().map(SourceKind::label)),
                )],
            ),
            Rule::RestrictFeaturesOf { crate_, allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-features-of",
                [
                    ("allowed", canonical_set(allowed)),
                    ("crate", crate_.clone()),
                ],
            ),
            Rule::ForbidFeaturesOf { crate_, forbidden } => RuleKey::of(
                "tianheng.rule/guibiao/forbid-features-of",
                [
                    ("crate", crate_.clone()),
                    ("forbidden", canonical_set(forbidden)),
                ],
            ),
        }
    }

    /// Each crate rule is the single source of truth for its own behavior: its
    /// label, text and JSON projection, and which declared dependencies it flags
    /// (including its observation source). Every method is one exhaustive match, so
    /// adding a variant is a compile error until it is handled everywhere
    /// (see PROJECT.md). The label feeds human violation/projection text; [`Rule::key`]
    /// separately carries semantic identity so wording remains free to evolve.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Rule::DenyExternalDependencies { .. } => "deny external dependencies",
            Rule::ForbidDependencyOn { .. } => "forbid dependency on",
            Rule::RestrictDependenciesTo { .. } => "restrict dependencies to",
            Rule::RestrictWorkspaceDependenciesTo { .. } => "restrict workspace dependencies to",
            Rule::RestrictDependencySourcesTo { .. } => "restrict dependency sources to",
            Rule::RestrictFeaturesOf { .. } => "restrict features of",
            Rule::ForbidFeaturesOf { .. } => "forbid features of",
        }
    }

    /// The repair-direction [`Polarity`] of a violation of this rule. `ForbidDependencyOn` names
    /// specific forbidden crates (repair: remove) → `DenyBreach`; the rest permit a set and react
    /// to a member outside it (repair: remove or declare) → `AllowlistGap`. `DenyExternalDependencies`
    /// is `AllowlistGap` **by repair direction, not name**: its `allow_external` exceptions are an
    /// in-boundary declaration path, so a new external dep is either removed or excepted.
    pub(crate) fn polarity(&self) -> Polarity {
        match self {
            Rule::ForbidDependencyOn { .. } | Rule::ForbidFeaturesOf { .. } => Polarity::DenyBreach,
            Rule::DenyExternalDependencies { .. }
            | Rule::RestrictDependenciesTo { .. }
            | Rule::RestrictWorkspaceDependenciesTo { .. }
            | Rule::RestrictDependencySourcesTo { .. }
            | Rule::RestrictFeaturesOf { .. } => Polarity::AllowlistGap,
        }
    }

    /// The human-readable rule text with its parameters, for the text projection.
    pub(crate) fn text(&self) -> String {
        match self {
            Rule::DenyExternalDependencies { allowed } if allowed.is_empty() => {
                "deny external dependencies".to_string()
            }
            Rule::DenyExternalDependencies { allowed } => {
                format!("deny external dependencies (allow: {})", allowed.join(", "))
            }
            Rule::ForbidDependencyOn { crates } => {
                format!("forbid dependency on: {}", crates.join(", "))
            }
            Rule::RestrictDependenciesTo { allowed } if allowed.is_empty() => {
                "restrict dependencies to nothing".to_string()
            }
            Rule::RestrictDependenciesTo { allowed } => {
                format!("restrict dependencies to: {}", allowed.join(", "))
            }
            Rule::RestrictWorkspaceDependenciesTo { allowed } if allowed.is_empty() => {
                "forbid all workspace dependencies".to_string()
            }
            Rule::RestrictWorkspaceDependenciesTo { allowed } => {
                format!("restrict workspace dependencies to: {}", allowed.join(", "))
            }
            Rule::RestrictDependencySourcesTo { allowed } if allowed.is_empty() => {
                "forbid all dependencies (by source)".to_string()
            }
            Rule::RestrictDependencySourcesTo { allowed } => {
                format!(
                    "restrict dependency sources to: {}",
                    allowed
                        .iter()
                        .map(SourceKind::label)
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Rule::RestrictFeaturesOf { crate_, allowed } if allowed.is_empty() => {
                format!("restrict features of {crate_} to nothing")
            }
            Rule::RestrictFeaturesOf { crate_, allowed } => {
                format!("restrict features of {crate_} to: {}", allowed.join(", "))
            }
            Rule::ForbidFeaturesOf { crate_, forbidden } if forbidden.is_empty() => {
                format!("forbid no features of {crate_}")
            }
            Rule::ForbidFeaturesOf { crate_, forbidden } => {
                format!("forbid features of {crate_}: {}", forbidden.join(", "))
            }
        }
    }

    /// The JSON parameter fields for the projection. Deny-external's `allowed` is an
    /// optional exception list (emitted only when non-empty); restrict-to's `only` is
    /// the intrinsic closed set (always emitted, as `[]` when empty); forbid lists
    /// its `crates`; the workspace-scoped restrict-to uses `only_workspace`, distinct
    /// from `only` so the projection says which dependency surface it governs.
    pub(crate) fn json_params(&self) -> Vec<(&'static str, Value)> {
        match self {
            Rule::DenyExternalDependencies { allowed } if allowed.is_empty() => Vec::new(),
            Rule::DenyExternalDependencies { allowed } => {
                vec![("allowed", serde_json::json!(allowed))]
            }
            Rule::ForbidDependencyOn { crates } => vec![("crates", serde_json::json!(crates))],
            Rule::RestrictDependenciesTo { allowed } => vec![("only", serde_json::json!(allowed))],
            Rule::RestrictWorkspaceDependenciesTo { allowed } => {
                vec![("only_workspace", serde_json::json!(allowed))]
            }
            Rule::RestrictDependencySourcesTo { allowed } => {
                let sources: Vec<&str> = allowed.iter().map(SourceKind::label).collect();
                vec![("allowed_sources", serde_json::json!(sources))]
            }
            // `crate` names the governed dependency; `only_features` is the intrinsic closed
            // set (always emitted, as `[]` when empty), matching the restrict-to vocabulary.
            Rule::RestrictFeaturesOf { crate_, allowed } => vec![
                ("crate", serde_json::json!(crate_)),
                ("only_features", serde_json::json!(allowed)),
            ],
            // `forbidden_features` lists the denied names, distinct from restrict's
            // `only_features` so the projection says which polarity governs the feature set.
            Rule::ForbidFeaturesOf { crate_, forbidden } => vec![
                ("crate", serde_json::json!(crate_)),
                ("forbidden_features", serde_json::json!(forbidden)),
            ],
        }
    }

    /// The target's declared dependencies that violate this rule. Each rule owns both
    /// its observation source (external-only / all normal / workspace-only) and its
    /// filter. `workspace_members` is all workspace member names, observed from
    /// `cargo metadata`; only the workspace-scoped rule consults it — and excludes the
    /// TARGET's own name from that set (see the workspace-scoped arm below): Cargo genuinely
    /// permits a crate declaring itself as a `[dev-dependencies]` path dependency on itself
    /// (a common doctest/dogfooding pattern, `main = { path = "." }`), which `cargo metadata
    /// --no-deps` emits verbatim — a real edge, not a parse artifact. A self-dependency is
    /// never an inter-crate layering violation (there is no OTHER crate to leak across a
    /// boundary to), so it must never be governed as one (calling this case "harmless" while
    /// `workspace_members` still included the target's own name
    /// unfiltered, which is what actually made it flag).
    #[cfg(test)]
    pub(crate) fn findings(
        &self,
        package: &Value,
        workspace_members: &[String],
        kind: DependencyKind,
    ) -> Vec<String> {
        self.facts(package, workspace_members, kind)
            .into_iter()
            .map(|fact| fact.into_finding().text().to_string())
            .collect()
    }

    pub(crate) fn facts(
        &self,
        package: &Value,
        workspace_members: &[String],
        kind: DependencyKind,
    ) -> Vec<crate::finding::CrateFact> {
        let dependencies: Vec<String> = match self {
            Rule::DenyExternalDependencies { allowed } => external_dependencies(package, kind)
                .into_iter()
                .filter(|dependency| !allowed.contains(dependency))
                .collect(),
            Rule::ForbidDependencyOn { crates } => dependencies(package, kind)
                .into_iter()
                .filter(|dependency| crates.contains(dependency))
                .collect(),
            Rule::RestrictDependenciesTo { allowed } => dependencies(package, kind)
                .into_iter()
                .filter(|dependency| !allowed.contains(dependency))
                .collect(),
            // A dependency on the TARGET'S OWN name is never a cross-crate layering violation —
            // Cargo allows (and dogfooding/doctest patterns genuinely use) a crate listing
            // itself as a dev-dependency path on itself. `dependencies()` itself now excludes
            // this self-referential edge (see `cargo_metadata.rs::is_self_dependency`), a
            // round-12 fix that closed the identical gap for every OTHER rule reading the same
            // observation too — round 11's own fix filtered it only HERE, leaving every sibling
            // rule (`ForbidDependencyOn`, `RestrictDependenciesTo`, `RestrictDependencySourcesTo`)
            // still vulnerable; see `PROJECT.md`'s Decisions.
            Rule::RestrictWorkspaceDependenciesTo { allowed } => dependencies(package, kind)
                .into_iter()
                .filter(|dependency| {
                    workspace_members.contains(dependency) && !allowed.contains(dependency)
                })
                .collect(),
            Rule::RestrictDependencySourcesTo { allowed } => {
                return dependencies_with_disallowed_source(package, kind, allowed)
                    .into_iter()
                    .map(|(dependency, source)| {
                        crate::finding::CrateFact::source(dependency, source, kind)
                    })
                    .collect();
            }
            // Feature-granularity rules observe the target's DECLARED feature request on
            // `crate_` (declared-not-resolved; see `declared_features`) and qualify each
            // offending feature `f` as `crate_/f`. A feature name on a dependency edge is a
            // plain name (Cargo forbids `dep:`/`pkg/feat` there), so `crate_/f` is unambiguous.
            Rule::RestrictFeaturesOf { crate_, allowed } => {
                // Allowlist: a declared feature outside `allowed` violates. Empty allowlist ⇒
                // every declared feature (including `default`) violates.
                return declared_features(package, crate_, kind)
                    .into_iter()
                    .filter(|feature| !allowed.contains(feature))
                    .map(|feature| {
                        crate::finding::CrateFact::feature(crate_.clone(), feature, kind)
                    })
                    .collect();
            }
            Rule::ForbidFeaturesOf { crate_, forbidden } => {
                // Denylist: a declared feature matching a forbidden name violates. Empty
                // forbidden set ⇒ no findings (natural from the filter), a vacuous no-op.
                return declared_features(package, crate_, kind)
                    .into_iter()
                    .filter(|feature| forbidden.contains(feature))
                    .map(|feature| {
                        crate::finding::CrateFact::feature(crate_.clone(), feature, kind)
                    })
                    .collect();
            }
        };
        dependencies
            .into_iter()
            .map(|dependency| crate::finding::CrateFact::dependency(dependency, kind))
            .collect()
    }
}

/// Fluent builder: `CrateBoundary::crate_("x").deny_external_dependencies().because("…")`
/// or `CrateBoundary::crate_("x").forbid_dependency_on(["y"]).because("…")`.
pub struct CrateBoundaryBuilder {
    target: CrateTarget,
}

impl CrateBoundaryBuilder {
    /// Deny external dependencies. Chain [`DenyExternalDraft::allow_external`] to
    /// name exceptions, and [`DenyExternalDraft::warn`] to make it advisory, before
    /// [`DenyExternalDraft::because`].
    pub fn deny_external_dependencies(self) -> DenyExternalDraft {
        DenyExternalDraft {
            target: self.target,
            allowed: Vec::new(),
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Forbid a normal dependency on any of `crates`, whether it resolves to an
    /// external source or to an internal workspace path (crate-to-crate layering).
    pub fn forbid_dependency_on<I, S>(self, crates: I) -> CrateBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::ForbidDependencyOn {
                crates: crates.into_iter().map(Into::into).collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Restrict this crate's normal dependencies to a closed allowlist: any normal
    /// dependency (external or internal) not named in `allowed` is a violation. An
    /// empty allowlist forbids every normal dependency.
    pub fn restrict_dependencies_to<I, S>(self, allowed: I) -> CrateBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::RestrictDependenciesTo {
                allowed: allowed.into_iter().map(Into::into).collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Restrict this crate's dependencies on *other workspace members* to a closed
    /// allowlist: any normal dependency on a workspace member not named in `allowed`
    /// is a violation; external dependencies are ignored. Workspace members are
    /// derived from `cargo metadata`, so a newly added member is governed by default.
    /// Unlike [`restrict_dependencies_to`](Self::restrict_dependencies_to), which
    /// governs *all* normal dependencies (external included), this governs only the
    /// workspace surface.
    pub fn restrict_workspace_dependencies_to<I, S>(self, allowed: I) -> CrateBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::RestrictWorkspaceDependenciesTo {
                allowed: allowed.into_iter().map(Into::into).collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Forbid this crate from depending on *any* other workspace member — the
    /// empty-allowlist shorthand for
    /// [`restrict_workspace_dependencies_to`](Self::restrict_workspace_dependencies_to).
    pub fn forbid_all_workspace_dependencies(self) -> CrateBoundaryDraft {
        self.restrict_workspace_dependencies_to(Vec::<String>::new())
    }

    /// Restrict the **declared source kinds** of this crate's dependencies to a closed
    /// allowlist: any dependency whose classified [`SourceKind`] is not in `allowed` is
    /// a violation (a publishable infra crate declares `[Registry, Path]` to forbid a
    /// `git` source; a workspace tool may declare the opposite). An empty allowlist
    /// forbids every dependency by source. Chain [`warn`](CrateBoundaryDraft::warn),
    /// [`dependency_kind`](CrateBoundaryDraft::dependency_kind), and
    /// [`because`](CrateBoundaryDraft::because) as with the other crate rules.
    ///
    /// Two stated bounds (deliberate, not silent):
    /// - It governs the **declared** source, not the *resolved* one. A registry
    ///   dependency redirected to git/path by `[patch]` or `[source] replace-with`
    ///   reads as `Registry` (no violation) — correct for manifest hygiene, since
    ///   `[patch]` is workspace-local and never blocks `cargo publish`. Observing the
    ///   resolved source is cargo-deny's `[sources]` lane, not a Tianheng capability.
    /// - It is source-kind **hygiene**, not a `cargo publish` oracle. A
    ///   `{ git = "…", version = "…" }` (or `{ path = "…", version = "…" }`) dependency
    ///   declares a non-registry source and is flagged even though it would publish
    ///   successfully; the rule does not parse the `version` key.
    pub fn restrict_dependency_sources_to<I>(self, allowed: I) -> CrateBoundaryDraft
    where
        I: IntoIterator<Item = SourceKind>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::RestrictDependencySourcesTo {
                allowed: allowed.into_iter().collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Restrict the **declared features** this crate requests on dependency `crate_` to a
    /// closed allowlist: any feature in the target's declared set for `crate_` (its authored
    /// `features = [...]`, ∪ the `default` pseudo-feature when default features are left on)
    /// not named in `allowed` is a violation. An empty allowlist forbids declaring **any**
    /// feature of `crate_`, `default` included (i.e. requires `default-features = false`).
    /// The feature-granularity mirror of
    /// [`restrict_dependencies_to`](Self::restrict_dependencies_to). `crate_` is matched by
    /// package name, not a local `rename`/alias. Chain [`warn`](CrateBoundaryDraft::warn),
    /// [`dependency_kind`](CrateBoundaryDraft::dependency_kind), and
    /// [`because`](CrateBoundaryDraft::because) as with the other crate rules.
    pub fn restrict_features_of<C, I, S>(self, crate_: C, allowed: I) -> CrateBoundaryDraft
    where
        C: Into<String>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::RestrictFeaturesOf {
                crate_: crate_.into(),
                allowed: allowed.into_iter().map(Into::into).collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Forbid this crate from declaring specific named `forbidden` features of dependency
    /// `crate_`: any feature in the target's declared set for `crate_` matching a forbidden
    /// name is a violation; a forbidden feature the target does not declare is not. Forbidding
    /// the `default` pseudo-feature requires `default-features = false`. An empty forbidden
    /// set is a no-op that always reports clean. The feature-granularity mirror of
    /// [`forbid_dependency_on`](Self::forbid_dependency_on). `crate_` is matched by package
    /// name, not a local `rename`/alias. Chain [`warn`](CrateBoundaryDraft::warn),
    /// [`dependency_kind`](CrateBoundaryDraft::dependency_kind), and
    /// [`because`](CrateBoundaryDraft::because) as with the other crate rules.
    pub fn forbid_features_of<C, I, S>(self, crate_: C, forbidden: I) -> CrateBoundaryDraft
    where
        C: Into<String>,
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        CrateBoundaryDraft {
            target: self.target,
            rule: Rule::ForbidFeaturesOf {
                crate_: crate_.into(),
                forbidden: forbidden.into_iter().map(Into::into).collect(),
            },
            severity: Severity::Enforce,
            kind: DependencyKind::Normal,
        }
    }

    /// Forbid this crate from declaring the single `feature` of dependency `crate_` — the
    /// singular convenience for [`forbid_features_of`](Self::forbid_features_of). Forbidding
    /// `"default"` requires `default-features = false`.
    pub fn forbid_feature<C, S>(self, crate_: C, feature: S) -> CrateBoundaryDraft
    where
        C: Into<String>,
        S: Into<String>,
    {
        self.forbid_features_of(crate_, [feature])
    }
}

/// A deny-external boundary awaiting an optional allowlist, severity, and reason.
pub struct DenyExternalDraft {
    target: CrateTarget,
    allowed: Vec<String>,
    severity: Severity,
    kind: DependencyKind,
}

impl DenyExternalDraft {
    /// Allow these external dependencies as named exceptions to the deny rule.
    pub fn allow_external<I, S>(mut self, crates: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.allowed.extend(crates.into_iter().map(Into::into));
        self
    }

    /// Make this boundary advisory: its violations are reported but do not fail CI.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Observe a different dependency table (`Dev` or `Build`); the default is `Normal`.
    pub fn dependency_kind(mut self, kind: DependencyKind) -> Self {
        self.kind = kind;
        self
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> CrateBoundary {
        CrateBoundary {
            target: self.target,
            rule: Rule::DenyExternalDependencies {
                allowed: self.allowed,
            },
            reason: reason.to_string(),
            severity: self.severity,
            kind: self.kind,
            anchor: None,
        }
    }
}

/// A crate boundary awaiting its severity and reason.
pub struct CrateBoundaryDraft {
    target: CrateTarget,
    rule: Rule,
    severity: Severity,
    kind: DependencyKind,
}

impl CrateBoundaryDraft {
    /// Make this boundary advisory: its violations are reported but do not fail CI.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Observe a different dependency table (`Dev` or `Build`); the default is `Normal`.
    pub fn dependency_kind(mut self, kind: DependencyKind) -> Self {
        self.kind = kind;
        self
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> CrateBoundary {
        CrateBoundary {
            target: self.target,
            rule: self.rule,
            reason: reason.to_string(),
            severity: self.severity,
            kind: self.kind,
            anchor: None,
        }
    }
}
