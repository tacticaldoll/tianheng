use super::crate_rule::CrateBoundary;
use super::module_rule::ModuleBoundary;

pub(crate) fn canonical_set<I, S>(values: I) -> String
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

pub(crate) fn canonical_module_set<I, S>(values: I) -> String
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    canonical_set(values)
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
