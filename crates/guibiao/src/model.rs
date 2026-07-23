use super::*;
use crate::module_scan::{canonical_module_path, package_name_to_import_ident};
use serde_json::Value;
use xuanji::RuleKey;

fn canonical_set<I, S>(values: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut values: Vec<String> = values
        .into_iter()
        .map(|value| value.as_ref().to_string())
        .collect();
    values.sort_unstable();
    values.dedup();
    serde_json::to_string(&values).expect("a list of strings always serializes")
}

fn canonical_module_set<I, S>(values: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    canonical_set(
        values
            .into_iter()
            .map(|value| canonical_module_path(value.as_ref())),
    )
}

/// The governed shape, declared in Rust (the single source of truth).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Constitution {
    name: String,
    boundaries: Vec<Boundary>,
}

impl Constitution {
    /// Begin a constitution for a project (the name is a label, not a path).
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            boundaries: Vec::new(),
        }
    }

    /// Add one boundary — a [`CrateBoundary`] or a [`ModuleBoundary`].
    pub fn boundary(mut self, boundary: impl Into<Boundary>) -> Self {
        self.boundaries.push(boundary.into());
        self
    }

    /// The constitution's name (a label, not a path).
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The declared boundaries, in declaration order.
    pub fn boundaries(&self) -> &[Boundary] {
        &self.boundaries
    }
}

/// Which dependency table a crate rule observes. Defaults to `Normal`. Mirrors
/// cargo's fixed set (normal / dev / build), so it is intentionally not
/// `#[non_exhaustive]` — unlike [`Rule`], this enum will not grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DependencyKind {
    /// The normal `[dependencies]` table. The default.
    #[default]
    Normal,
    /// The `[dev-dependencies]` table.
    Dev,
    /// The `[build-dependencies]` table.
    Build,
}

impl DependencyKind {
    /// The finding suffix that keeps a dependency's identity distinct per table. `Normal` (the
    /// default, the overwhelming common case) stays bare — so existing baselines do not churn —
    /// while `Dev`/`Build` carry ` (dev)`/` (build)`. Without this, two boundaries governing the
    /// same crate under the same rule but different kinds (e.g. a `serde` git source in both
    /// `[dependencies]` and `[dev-dependencies]`) would emit the identical `(target, rule,
    /// finding)` and one baselined violation would mask the other (the one forbidden bug).
    pub(crate) fn finding_suffix(&self) -> &'static str {
        match self {
            DependencyKind::Normal => "",
            DependencyKind::Dev => " (dev)",
            DependencyKind::Build => " (build)",
        }
    }

    /// The published identity value for a dependency table. This is baseline wire, not a
    /// presentation label; changing a byte re-keys every matching 圭表 finding.
    pub(crate) fn key_label(&self) -> &'static str {
        match self {
            DependencyKind::Normal => "normal",
            DependencyKind::Dev => "dev",
            DependencyKind::Build => "build",
        }
    }
}

/// A dependency's **declared** source kind, classified from `cargo metadata`'s
/// `source` field. The vocabulary of the [`Rule::RestrictDependencySourcesTo`]
/// allowlist. Like [`DependencyKind`], it mirrors a fixed cargo distinction (a
/// declared source is a registry, a git, or a path), so it is intentionally not
/// `#[non_exhaustive]`: it will not grow.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// A registry source (`registry+…`, `sparse+…`, or an alternative registry) —
    /// the residual kind, matched by neither of the others.
    Registry,
    /// A git source (`git+…`).
    Git,
    /// A path/internal source (a null declared source).
    Path,
}

impl SourceKind {
    /// The stable string label, feeding the rule's text and JSON projection.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            SourceKind::Registry => "registry",
            SourceKind::Git => "git",
            SourceKind::Path => "path",
        }
    }
}

/// One boundary, of either kind. Named `Boundary` (umbrella) with the crate kind as
/// [`CrateBoundary`], since a module reaction is also a boundary (drift law D2).
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Boundary {
    /// A rule on a crate target, observed via `cargo metadata`.
    Crate(CrateBoundary),
    /// A rule on an intra-crate module, observed from source `use` declarations.
    Module(ModuleBoundary),
}

impl From<CrateBoundary> for Boundary {
    fn from(boundary: CrateBoundary) -> Self {
        Boundary::Crate(boundary)
    }
}

impl From<ModuleBoundary> for Boundary {
    fn from(boundary: ModuleBoundary) -> Self {
        Boundary::Module(boundary)
    }
}

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
    /// boundary to), so it must never be governed as one (found on a round-11 adversarial
    /// review — see `PROJECT.md`'s Decisions; a stale comment here previously claimed this
    /// case was "harmless" while `workspace_members` still included the target's own name
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

/// A boundary over the intra-crate module import graph — the layering Cargo cannot
/// see. Observed from the target crate's source `use` declarations (PROJECT.md).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleBoundary {
    pub(crate) crate_package: String,
    pub(crate) module: String,
    pub(crate) rule: ModuleRule,
    pub(crate) reason: String,
    pub(crate) severity: Severity,
    pub(crate) anchor: Option<String>,
}

impl ModuleBoundary {
    /// Begin a module boundary within the crate named `package`.
    pub fn in_crate(package: &str) -> ModuleBoundaryBuilder {
        ModuleBoundaryBuilder {
            crate_package: package.to_string(),
        }
    }

    /// Attach a durable governance anchor (e.g. `"ADR-014"`) — a stable pointer into the
    /// project's governance, distinct from the free-text `reason`. Optional; a boundary with
    /// none projects and reacts exactly as before. Chained after [`because`](ModuleBoundaryDraft::because).
    pub fn with_anchor(mut self, anchor: &str) -> Self {
        self.anchor = Some(anchor.to_string());
        self
    }

    /// The durable governance anchor recorded with the boundary, if any.
    pub fn anchor(&self) -> Option<&str> {
        self.anchor.as_deref()
    }

    /// The rule this boundary declares, exposed read-only for projection and model inspection.
    pub fn rule(&self) -> &ModuleRule {
        &self.rule
    }
}

/// What a module boundary forbids.
///
/// Rules are constructed through [`ModuleBoundary::in_crate`], not variant struct expressions. A
/// consumer can inspect a builder-produced rule without closing over its complete representation:
///
/// ```
/// use guibiao::{ModuleBoundary, ModuleRule};
///
/// let boundary = ModuleBoundary::in_crate("app")
///     .module("crate::core")
///     .must_not_import("crate::adapter")
///     .because("core depends inward only");
/// match boundary.rule() {
///     ModuleRule::MustNotImport { module, .. } => assert_eq!(module, "crate::adapter"),
///     _ => unreachable!(),
/// }
/// ```
///
/// ```compile_fail
/// use guibiao::ModuleRule;
///
/// let _ = ModuleRule::MustNotImport { module: "crate::adapter".to_string() };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ModuleRule {
    /// The governed module must not import this module (or anything beneath it).
    #[non_exhaustive]
    MustNotImport {
        /// The forbidden module path (e.g. `"crate::projection"`).
        module: String,
    },
    /// The governed module may import only these modules (each "or beneath"), plus its
    /// own subtree; any other internal import is a violation. An empty allowlist permits
    /// only the module's own subtree.
    #[non_exhaustive]
    RestrictImportsTo {
        /// The closed allowlist of importable module paths (e.g. `["crate::types"]`).
        allowed: Vec<String>,
    },
    /// The governed (protected) module must not be imported by this module (or anything
    /// beneath it) — an inbound encapsulation rule, the mirror of `MustNotImport`. A
    /// module within the protected module's own subtree is never an importer.
    #[non_exhaustive]
    MustNotBeImportedBy {
        /// The forbidden importer module path (e.g. `"crate::http"`).
        importer: String,
    },
    /// The governed (protected) module may be imported only by these importers (each "or
    /// beneath") or by its own subtree; any other module that imports it (or anything beneath
    /// it) is a violation — the inbound dual of `RestrictImportsTo`. An empty allowlist permits
    /// only the protected module's own subtree.
    #[non_exhaustive]
    MustOnlyBeImportedBy {
        /// The closed allowlist of importer module paths (e.g. `["crate::facade"]`).
        allowed: Vec<String>,
    },
    /// An **external** crate may be imported only within the governed module's own subtree
    /// (the permitted subtree, or beneath it); any `use <crate_name>::…` from a module
    /// outside that subtree is a violation. The first module rule that observes external
    /// imports — every other rule ignores them. The confined crate is the violation target,
    /// so identity stays injective across different confined crates on the same subtree.
    #[non_exhaustive]
    ConfineExternalCrate {
        /// The confined external crate name (e.g. `"libc"`).
        crate_name: String,
    },
    /// Within the governed module's subtree, forbid inline symbol-path **calls** resolving under
    /// a declared module-path prefix — the inline-symbol-path (layer b) sibling of
    /// [`ConfineExternalCrate`](ModuleRule::ConfineExternalCrate), observing *calls* rather than
    /// `use` imports. The "core reads no ambient clock; time is injected" pattern. The confined
    /// prefix is the violation target, so identity stays injective across nested prefixes on the
    /// same subtree.
    #[non_exhaustive]
    ConfineInlineSymbolPath {
        /// The confined module-path prefix (e.g. `"std::time"`).
        prefix: String,
        /// If `Some`, react only on calls whose terminal segment (leaf-exact) is one of these
        /// verbs (e.g. `["now"]`); `None` reacts on every call under the prefix. Adopter-owned:
        /// a read reachable only through an undeclared verb is a false negative the adopter
        /// accepts by narrowing.
        ending_with: Option<Vec<String>>,
        /// If `true`, react on **any** path under the prefix (mentions included — type
        /// annotations, constants, value captures), not only calls. Mutually exclusive with
        /// `ending_with` (both set is a constitution error).
        strict: bool,
        /// If `true`, resolve a bare path head matching a declared dependency as external after
        /// local precedence checks. Projection metadata and scan breadth only; never identity.
        strict_external: bool,
    },
}

/// The inline-confinement text projection. Neither it nor [`ModuleRule::label`] is identity;
/// [`ModuleRule::key`] carries the semantic rule identity.
fn inline_confinement_text(
    prefix: &str,
    ending_with: &Option<Vec<String>>,
    strict: bool,
) -> String {
    match (ending_with, strict) {
        (_, true) => format!("must not name inline under {prefix} (strict: mentions too)"),
        (Some(verbs), false) => format!(
            "must not call inline under {prefix} ending with: {}",
            verbs.join(", ")
        ),
        (None, false) => format!("must not call inline under {prefix}"),
    }
}

/// The inline-confinement JSON parameters. `strict_external` is emitted only when set, matching the emit-when-set
/// discipline of `ending_with`/`strict` — a strict boundary must not project byte-identically to a
/// default one. This is projection metadata only; it never leaks into [`ModuleRule::label`].
fn inline_confinement_json(
    prefix: &str,
    ending_with: &Option<Vec<String>>,
    strict: bool,
    external: bool,
) -> Vec<(&'static str, Value)> {
    let mut params = vec![("confined_prefix", serde_json::json!(prefix))];
    if let Some(verbs) = ending_with {
        params.push(("ending_with", serde_json::json!(verbs)));
    }
    if strict {
        params.push(("strict", serde_json::json!(true)));
    }
    if external {
        params.push(("strict_external", serde_json::json!(true)));
    }
    params
}

impl ModuleRule {
    /// Stable semantic identity for this declared module rule.
    pub fn key(&self) -> RuleKey {
        match self {
            ModuleRule::MustNotImport { module } => RuleKey::of(
                "tianheng.rule/guibiao/must-not-import",
                [("module", canonical_module_path(module))],
            ),
            ModuleRule::RestrictImportsTo { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/restrict-imports-to",
                [("allowed", canonical_module_set(allowed))],
            ),
            ModuleRule::MustNotBeImportedBy { importer } => RuleKey::of(
                "tianheng.rule/guibiao/must-not-be-imported-by",
                [("importer", canonical_module_path(importer))],
            ),
            ModuleRule::MustOnlyBeImportedBy { allowed } => RuleKey::of(
                "tianheng.rule/guibiao/must-only-be-imported-by",
                [("allowed", canonical_module_set(allowed))],
            ),
            ModuleRule::ConfineExternalCrate { crate_name } => RuleKey::of(
                "tianheng.rule/guibiao/confine-external-crate",
                [("crate", package_name_to_import_ident(crate_name))],
            ),
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external: _,
            } => RuleKey::of(
                "tianheng.rule/guibiao/confine-inline-symbol-path",
                [
                    (
                        "ending_with",
                        canonical_set(
                            ending_with
                                .iter()
                                .flat_map(|values| values.iter())
                                .map(|verb| canonical_module_path(verb)),
                        ),
                    ),
                    ("prefix", canonical_module_path(prefix)),
                    ("strict", strict.to_string()),
                ],
            ),
        }
    }

    /// The label feeding the violation `rule` string and the projection — one source.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            ModuleRule::MustNotImport { .. } => "module must not import",
            ModuleRule::RestrictImportsTo { .. } => "restrict imports to",
            ModuleRule::MustNotBeImportedBy { .. } => "module must not be imported by",
            ModuleRule::MustOnlyBeImportedBy { .. } => "module may only be imported by",
            ModuleRule::ConfineExternalCrate { .. } => "external crate confined to module",
            // Presentation parity: the modifier remains a projection detail of the same
            // inline-rule family; `key()` separately preserves the established no-rekey contract.
            ModuleRule::ConfineInlineSymbolPath { .. } => "inline symbol path confined to module",
        }
    }

    /// The inline-confinement payload — `(prefix, ending_with, strict, external)` — or `None` for a
    /// non-inline rule. Dispatch and the exit-2 constitution checks route through this accessor; the only
    /// `external`-conditional behavior lives in the scan (`inline_symbol_findings` / `resolve_head`).
    pub(crate) fn inline_payload(&self) -> Option<(&str, Option<&[String]>, bool, bool)> {
        match self {
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => Some((prefix, ending_with.as_deref(), *strict, *strict_external)),
            _ => None,
        }
    }

    /// The repair-direction [`Polarity`] of a violation of this rule. The two `MustNot*` rules
    /// forbid a specific module edge (repair: remove the import) → `DenyBreach`; `RestrictImportsTo`,
    /// `MustOnlyBeImportedBy`, and `ConfineExternalCrate` permit a region and react to an edge
    /// outside it (repair: move the import into the permitted subtree, or widen) → `AllowlistGap`.
    pub(crate) fn polarity(&self) -> Polarity {
        match self {
            ModuleRule::MustNotImport { .. } | ModuleRule::MustNotBeImportedBy { .. } => {
                Polarity::DenyBreach
            }
            ModuleRule::RestrictImportsTo { .. }
            | ModuleRule::MustOnlyBeImportedBy { .. }
            | ModuleRule::ConfineExternalCrate { .. } => Polarity::AllowlistGap,
            // A forbidden inline call under the prefix is a breach to remove (or replace with
            // injected time) — the same repair shape as `MustNotImport`, not an allowlist gap.
            // Identity parity: the strict-external modifier shares the polarity.
            ModuleRule::ConfineInlineSymbolPath { .. } => Polarity::DenyBreach,
        }
    }

    /// The human-readable rule text with its parameter, for the text projection.
    pub(crate) fn text(&self) -> String {
        match self {
            ModuleRule::MustNotImport { module } => format!("must not import {module}"),
            ModuleRule::RestrictImportsTo { allowed } if allowed.is_empty() => {
                "restrict imports to nothing".to_string()
            }
            ModuleRule::RestrictImportsTo { allowed } => {
                format!("restrict imports to: {}", allowed.join(", "))
            }
            ModuleRule::MustNotBeImportedBy { importer } => {
                format!("must not be imported by {importer}")
            }
            ModuleRule::MustOnlyBeImportedBy { allowed } if allowed.is_empty() => {
                "may only be imported by nothing".to_string()
            }
            ModuleRule::MustOnlyBeImportedBy { allowed } => {
                format!("may only be imported by: {}", allowed.join(", "))
            }
            ModuleRule::ConfineExternalCrate { crate_name } => {
                format!("confines external crate {crate_name} to this module's subtree")
            }
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => {
                let text = inline_confinement_text(prefix, ending_with, *strict);
                if *strict_external {
                    format!("{text} (strict-external)")
                } else {
                    text
                }
            }
        }
    }

    /// The JSON parameter fields for the projection. `must_not_import` names its single
    /// `forbidden` path; `restrict_imports_to` emits its closed set as `only` (always,
    /// as `[]` when empty), matching the crate-level restrict-to vocabulary;
    /// `must_not_be_imported_by` names its declared forbidden `importer`;
    /// `confine_external_crate` names the confined `external_crate`.
    pub(crate) fn json_params(&self) -> Vec<(&'static str, Value)> {
        match self {
            ModuleRule::MustNotImport { module } => {
                vec![("forbidden", serde_json::json!(module))]
            }
            ModuleRule::RestrictImportsTo { allowed } => {
                vec![("only", serde_json::json!(allowed))]
            }
            ModuleRule::MustNotBeImportedBy { importer } => {
                vec![("importer", serde_json::json!(importer))]
            }
            // `only_importers` (not bare `only`): this rule governs the inbound *importer*
            // surface, distinct from `restrict_imports_to`'s outbound `only` — the same
            // surface-qualified-key precedent `only_workspace` sets, so the projection is
            // self-describing without reading the `rule` label.
            ModuleRule::MustOnlyBeImportedBy { allowed } => {
                vec![("only_importers", serde_json::json!(allowed))]
            }
            // `external_crate` (self-describing): this rule confines a named external crate to
            // the governed module's subtree, a surface distinct from every internal-edge rule.
            ModuleRule::ConfineExternalCrate { crate_name } => {
                vec![("external_crate", serde_json::json!(crate_name))]
            }
            // `confined_prefix` (self-describing): the module-path prefix whose inline calls are
            // forbidden in the subtree. `ending_with` / `strict` are emitted only when set, so a
            // bare confinement keeps byte-identical JSON (the same discipline as the anchor).
            ModuleRule::ConfineInlineSymbolPath {
                prefix,
                ending_with,
                strict,
                strict_external,
            } => inline_confinement_json(prefix, ending_with, *strict, *strict_external),
        }
    }
}

/// Fluent builder for a [`ModuleBoundary`].
pub struct ModuleBoundaryBuilder {
    crate_package: String,
}

impl ModuleBoundaryBuilder {
    /// The module whose imports are governed (e.g. `"crate::kernel"`).
    pub fn module(self, module: &str) -> ModuleTargetDraft {
        ModuleTargetDraft {
            crate_package: self.crate_package,
            module: module.to_string(),
        }
    }
}

/// A module boundary awaiting its module rule.
pub struct ModuleTargetDraft {
    crate_package: String,
    module: String,
}

impl ModuleTargetDraft {
    /// Forbid the governed module from importing `module` (or anything beneath it).
    ///
    /// **Stated bound (glob of an ancestor):** a `use`-glob is observed at its base module only,
    /// so a glob of an *ancestor* of the forbidden module (`use crate::*;` while forbidding
    /// `crate::secret`) is recorded as the base (`crate`) — not as the forbidden descendant edge —
    /// and does not react, though it does bring the forbidden module into nameable scope. The
    /// narrow forms (`use crate::secret;`, `use crate::secret::*;`) are caught. This is a declared
    /// partial-coverage bound, not a silent gap: forbid or confine the *parent* to close it.
    pub fn must_not_import(self, module: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::MustNotImport {
            module: module.to_string(),
        })
    }

    /// Restrict the governed module's internal imports to a closed allowlist: any
    /// internal `use` reaching a module that is neither within the governed module's
    /// own subtree nor within an allowlist entry (each matched "or beneath") is a
    /// violation. An empty allowlist permits only the module's own subtree. Governs
    /// new internal modules by default, the module-level mirror of the crate-level
    /// [`restrict_dependencies_to`](CrateBoundaryBuilder::restrict_dependencies_to).
    pub fn restrict_imports_to<I, S>(self, allowed: I) -> ModuleBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_rule(ModuleRule::RestrictImportsTo {
            allowed: allowed.into_iter().map(Into::into).collect(),
        })
    }

    /// Forbid the governed (protected) module from being imported by `importer` (or
    /// anything beneath it) — an inbound encapsulation rule, the mirror of
    /// [`must_not_import`](Self::must_not_import). A module within the protected module's
    /// own subtree is never treated as an importer.
    pub fn must_not_be_imported_by(self, importer: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::MustNotBeImportedBy {
            importer: importer.to_string(),
        })
    }

    /// Restrict who may import the governed (protected) module to a closed allowlist: only a
    /// listed importer (each "or beneath") or the protected module's own subtree may import it;
    /// any other importer is a violation — the inbound dual of
    /// [`restrict_imports_to`](Self::restrict_imports_to). An empty allowlist permits only the
    /// module's own subtree.
    pub fn must_only_be_imported_by<I, S>(self, allowed: I) -> ModuleBoundaryDraft
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.with_rule(ModuleRule::MustOnlyBeImportedBy {
            allowed: allowed.into_iter().map(Into::into).collect(),
        })
    }

    /// Confine an **external** crate's imports to the governed module's own subtree: any
    /// `use <crate_name>::…` written from a module outside this module (or beneath it) is a
    /// violation. This is the first module rule that observes external-crate imports — every
    /// other rule ignores them. Only the named crate is observed (a `cargo metadata`
    /// cross-check is deliberately *not* performed: confining a crate the target never imports
    /// is simply clean). The confined crate is the violation target, so declaring two
    /// confinements of different crates on the same module stays injective. Confining on
    /// `crate` (the root) is a constitution error, since it would permit the crate everywhere.
    ///
    /// The crate name may be written in either **package** form (`"windows-sys"`) or **import
    /// identifier** form (`"windows_sys"`): the rule observes the source `use` identifier, in
    /// which Cargo maps a package's `-` to `_`, so the confined name is matched with that same
    /// fold. A raw identifier (`r#name`) is canonicalized as elsewhere.
    pub fn confine_external_crate(self, crate_name: &str) -> ModuleBoundaryDraft {
        self.with_rule(ModuleRule::ConfineExternalCrate {
            crate_name: crate_name.to_string(),
        })
    }

    /// Within the governed subtree, forbid inline symbol-path **calls** resolving under the
    /// module-path `prefix` (e.g. `"std::time"`) — the inline-symbol-path (layer b) sibling of
    /// [`confine_external_crate`](Self::confine_external_crate), for the "core reads no ambient
    /// clock; time is injected" pattern. By default only a **call** (`prefix::…::verb(...)`)
    /// reacts; a type annotation, a bare constant reference, and any non-call mention pass (so the
    /// core may *receive* injected time), keeping 圭表 free of a built-in read-verb heuristic. The
    /// returned [`InlineConfinementDraft`] is a dedicated draft — its `.ending_with` /
    /// `.strict_prefix_only` modifiers cannot be applied to the other module rules.
    ///
    /// Resolution follows the alias-carrying use-map, local `type` aliases, and the local
    /// `pub use` re-export closure to a fixpoint, and reacts fail-closed on a glob that can bring
    /// a prefix-resolving name into scope. The stated bounds (receiver-method reads, in-macro-body
    /// aliases, fragment/proc-macro construction, external-crate re-exports, value-position
    /// captures under the default, and the inherited file-scope scanner bounds) are declared
    /// non-observations, never silent passes.
    pub fn must_not_call_inline(self, prefix: &str) -> InlineConfinementDraft {
        InlineConfinementDraft {
            crate_package: self.crate_package,
            module: self.module,
            prefix: prefix.to_string(),
            ending_with: None,
            strict: false,
            external: false,
            severity: Severity::Enforce,
        }
    }

    fn with_rule(self, rule: ModuleRule) -> ModuleBoundaryDraft {
        ModuleBoundaryDraft {
            crate_package: self.crate_package,
            module: self.module,
            rule,
            severity: Severity::Enforce,
        }
    }
}

/// A module boundary awaiting its severity and reason.
pub struct ModuleBoundaryDraft {
    crate_package: String,
    module: String,
    rule: ModuleRule,
    severity: Severity,
}

impl ModuleBoundaryDraft {
    /// Make this boundary advisory: its violations are reported but do not fail CI.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> ModuleBoundary {
        ModuleBoundary {
            crate_package: self.crate_package,
            module: self.module,
            rule: self.rule,
            reason: reason.to_string(),
            severity: self.severity,
            anchor: None,
        }
    }
}

/// A dedicated draft for an inline-symbol-path confinement (from
/// [`must_not_call_inline`](ModuleTargetDraft::must_not_call_inline)). Distinct from
/// [`ModuleBoundaryDraft`] so its narrowing / escalation modifiers cannot be applied to the other
/// module rules (no modifier pollution). Chain [`ending_with`](Self::ending_with) **or**
/// [`strict_prefix_only`](Self::strict_prefix_only) (they are mutually exclusive), and
/// [`warn`](Self::warn), before [`because`](Self::because).
pub struct InlineConfinementDraft {
    crate_package: String,
    module: String,
    prefix: String,
    ending_with: Option<Vec<String>>,
    strict: bool,
    external: bool,
    severity: Severity,
}

impl InlineConfinementDraft {
    /// Narrow the confinement to react only on calls whose **terminal segment** (leaf-exact) is
    /// one of `verbs` (e.g. `["now"]`) — the adopter's declared read verbs. The adopter owns any
    /// false negative from omitting a verb (a future `::current()` passes); the engine bakes in no
    /// default verb set. Mutually exclusive with [`strict_prefix_only`](Self::strict_prefix_only):
    /// declaring both is a constitution error (exit 2).
    pub fn ending_with<I, S>(mut self, verbs: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.ending_with = Some(verbs.into_iter().map(Into::into).collect());
        self
    }

    /// Escalate the confinement to react on **any** path under the prefix — mentions included
    /// (type annotations, constants, value-position captures), not only calls. The whole-surface
    /// isolation posture for a subtree that may not even name the module. Mutually exclusive with
    /// [`ending_with`](Self::ending_with): declaring both is a constitution error (exit 2).
    pub fn strict_prefix_only(mut self) -> Self {
        self.strict = true;
        self
    }

    /// **Opt-in.** Resolve a written path's bare head that matches a **declared dependency name**
    /// (rename-aware, `-`→`_`-normalized to its import identifier) as that external crate — so a
    /// **fully-qualified, un-`use`d external call** (`chrono::Utc::now()` with no `use chrono`)
    /// resolving under the confined prefix reacts. This closes the asymmetry whereby a sysroot
    /// head (`std`/`core`/`alloc`) was caught while a fully-qualified external head resolved as a
    /// fake local path and was silently missed (a false negative).
    ///
    /// The flag has a second effect: the existing glob-hazard reaction **extends** to external-crate
    /// globs. A `use chrono::*;` under a `chrono::…` confinement now resolves its glob head as
    /// external `chrono` (an ancestor of the prefix) and reacts fail-closed; under the default it
    /// stays `{module}::chrono` and does not react.
    ///
    /// The reclassification honors **local precedence, first match wins**, checked against the
    /// call's TRUE inline module (`{module}::inner…`, following any `mod name { … }` around it): the
    /// enclosing module's `use`-map, a crate-root module shadow, any local module `{module}::head`,
    /// then any top-level item definition (mod/struct/enum/union/trait/type/fn/const/static) of that
    /// name **in the calling module** — only if none claim the head does the dependency match fire.
    /// A local item shadows a same-named external call only within its OWN module: a file-top
    /// `fn rand` does not mask a `rand::random()` call inside an inline `mod tests { … }`, and a
    /// submodule-local `fn rand` masks only calls in that submodule.
    ///
    /// It catches fully-qualified external calls **by the crate's real name**. It does NOT close:
    /// an `extern crate dep as alias;` rename (a call through the local `alias` head is a stated
    /// bound — the use-map observes `use` only), glob-brought names beyond the glob-hazard
    /// reaction, and macro-constructed names. Do not read this as "all external calls caught."
    ///
    /// One further stated bound, strict-external only: a `mod name {` token or unbalanced braces
    /// **inside a macro-invocation body** can perturb the call scan's inline-module tracking (the
    /// call scan keeps macro bodies — real reads hide there — while the item collector strips them),
    /// so a call's true module may be mis-attributed. Rare and declared, never a silent pass.
    ///
    /// One stated **over-**reaction bound, only under a **single-segment** bare crate prefix
    /// (`must_not_call_inline("rand")`) — a multi-segment prefix (`chrono::Utc`) is immune: 圭表's
    /// text scan cannot tell a local binding or a definition site from a call, so a local
    /// `let rand = …; rand()`, or the definition site of an associated / nested `fn rand(…)` (whose
    /// `rand(` reads as a call), may false-positive. Module-top-level definitions are exempt (they
    /// resolve to the local item). Declared, not silent.
    ///
    /// Orthogonal to [`ending_with`](Self::ending_with) / [`strict_prefix_only`](Self::strict_prefix_only):
    /// it changes head *resolution*, not call-vs-mention breadth, and composes with either — it is
    /// **not** itself part of their mutual exclusion (but the contradictory
    /// `.ending_with(…).strict_prefix_only()` pair is still a constitution error under the flag).
    /// When not set, the fully-qualified external call remains a stated non-observation and
    /// behavior is byte-identical to a confinement without the flag.
    pub fn strict_external(mut self) -> Self {
        self.external = true;
        self
    }

    /// Make this boundary advisory: its violations are reported but do not fail CI.
    pub fn warn(mut self) -> Self {
        self.severity = Severity::Warn;
        self
    }

    /// Finish the boundary, recording the human-readable `reason` (the repair hint).
    pub fn because(self, reason: &str) -> ModuleBoundary {
        let rule = ModuleRule::ConfineInlineSymbolPath {
            prefix: self.prefix,
            ending_with: self.ending_with,
            strict: self.strict,
            strict_external: self.external,
        };
        ModuleBoundary {
            crate_package: self.crate_package,
            module: self.module,
            rule,
            reason: reason.to_string(),
            severity: self.severity,
            anchor: None,
        }
    }
}
