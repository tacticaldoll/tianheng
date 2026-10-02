use super::*;
use serde_json::Value;

use crate::module_scan::{Edition, package_name_to_import_ident};

pub(crate) use xingbiao::{
    cargo_metadata, compilation_unit_label, crate_roots, find_package, member_src_dirs,
};

/// What the targets rooted at one file ask of reading it: the edition they are read in, and whether one is a
/// proc-macro crate, whose extern prelude holds `proc_macro` without an `extern crate` — measured against rustc 1.96.0,
/// edition 2021, a `[lib] proc-macro = true` crate calls `proc_macro::TokenStream::new()` with none.
pub(crate) struct RootReading {
    /// The one edition the root is read in; a mix of 2018 and later reads as 2018, and a mix with 2015 is refused.
    pub(crate) edition: Edition,
    /// Whether any target rooted at the file is a proc-macro crate, so `proc_macro` resolves with no `extern crate`.
    pub(crate) proc_macro: bool,
}

/// The [`RootReading`] of the targets rooted at `root_file`, or of every target of the package where the roots are not
/// reported. rustc compiles each target in its own edition, and `cargo metadata` reports a target's `[lib]` or
/// `[[bin]]` `edition` on the target and the package's own beside it — measured under cargo 1.96.0, a 2024 package with
/// `[lib] edition = "2015"` reports `2015` on its library target and `2024` on the package, and builds a 2015 crate. A
/// target reporting no edition reads as the package's. Targets sharing the root in editions the scanner reads apart are
/// compiled twice, once in each, so one reading cannot judge both and the root is refused; 2018 beside 2021, whose
/// paths it reads alike, are one reading, in 2018: the two differ only in whether a `c` before a string opens a C
/// string, and the 2018 reading lexes as code what the 2021 one could read as a string's contents, so it misses
/// nothing the other reads.
pub(crate) fn root_reading(
    package: &Value,
    root_file: Option<&Path>,
    crate_package: &str,
) -> Result<RootReading, String> {
    let package_edition = package["edition"].as_str();
    let targets: Vec<&Value> = package["targets"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(|target| {
            root_file.is_none_or(|root| target["src_path"].as_str().map(Path::new) == Some(root))
        })
        .collect();
    let proc_macro = targets.iter().any(|target| {
        target["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("proc-macro")))
    });
    let edition = match root_file {
        None => Edition::of(package_edition),
        Some(root) => {
            let mut written: Vec<Option<&str>> = targets
                .iter()
                .map(|target| target["edition"].as_str().or(package_edition))
                .collect();
            written.sort();
            written.dedup();
            let readings: Vec<Edition> = written
                .iter()
                .map(|edition| Edition::of(*edition))
                .collect();
            let paths_alike = |reading: &Edition| reading != &Edition::Rust2015;
            match readings.first() {
                None => Edition::of(package_edition),
                Some(first) if readings.iter().all(|reading| reading == first) => *first,
                Some(_) if readings.iter().all(paths_alike) => Edition::Rust2018,
                Some(_) => {
                    return Err(crate::errors::root_in_several_editions_error(
                        crate_package,
                        root,
                        &written.iter().map(|e| e.unwrap_or("?")).collect::<Vec<_>>(),
                    ));
                }
            }
        }
    };
    Ok(RootReading {
        edition,
        proc_macro,
    })
}

/// The membership set, or why it could not be read.
///
/// **Typed apart, because an empty answer had several causes and reported one.** A `packages` array
/// that is absent or is not an array, a package whose `name` this reader cannot take, and a workspace
/// that genuinely has no member all produced the same empty `Vec` — and both consumers read empty as
/// *nothing to govern*. Coverage rendered `total = 0, uncovered = []` as complete coverage over a
/// membership it never read.
///
/// The sibling rule is stated in this crate already, on
/// [`crate::workspace_member_src_dirs`]: *an unreadable workspace is a constitution error, never a
/// silent empty set*. This reader is the same question about the same metadata and did not follow it.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Members {
    /// The workspace's member names, sorted and deduplicated. Empty means the workspace declares none.
    Read(Vec<String>),
    /// The membership could not be read, and saying so is not the same fact as reading none.
    Unreadable(String),
}

/// The names of the workspace's member crates. Because Modou runs
/// `cargo metadata --no-deps`, the `packages` array contains exactly the workspace
/// members (no transitive dependencies), so their names are the membership set used
/// by the workspace-scoped rule and by coverage. A `path` dependency that points
/// outside the workspace is therefore absent here, as intended.
///
/// A package whose `name` is absent or is not a string **refuses** rather than being dropped: a
/// member this reader could not name is one it did not compare, which is not a member that is absent.
pub(crate) fn workspace_member_names(metadata: &Value) -> Members {
    let Some(packages) = metadata["packages"].as_array() else {
        return Members::Unreadable(
            "cargo metadata carries no `packages` array, so which crates are workspace members \
             cannot be decided"
                .to_string(),
        );
    };
    let mut names: Vec<String> = Vec::with_capacity(packages.len());
    for package in packages {
        match package["name"].as_str() {
            Some(name) => names.push(name.to_string()),
            None => {
                return Members::Unreadable(
                    "cargo metadata carries a package whose `name` is absent or is not a string, \
                     so the membership set this reader would compare against is incomplete"
                        .to_string(),
                );
            }
        }
    }
    names.sort();
    names.dedup();
    Members::Read(names)
}

/// Whether a `cargo metadata` dependency belongs to the selected table. `kind` is
/// null for normal deps, `"dev"` / `"build"` otherwise.
fn kind_matches(dependency: &Value, kind: DependencyKind) -> bool {
    matches!(
        (kind, dependency["kind"].as_str()),
        (DependencyKind::Normal, None)
            | (DependencyKind::Dev, Some("dev"))
            | (DependencyKind::Build, Some("build"))
    )
}

/// Every dependency-table edge of the target's declared `kind`, optionally excluding the
/// target's own self-referential edge (see [`is_self_dependency`]) — the shared filter every
/// consuming rule ([`external_dependencies`] / [`dependencies`] /
/// [`dependencies_with_disallowed_source`]) needs, so kind-matching and self-dependency
/// exclusion cannot silently diverge across them — the shared-filter reason
/// [`is_self_dependency`]'s own doc states. [`external_dependencies`] passes
/// `exclude_self: false`: its own `!source.is_null()` filter already excludes a self-dependency
/// (always a null-source `path` edge), so an extra explicit exclusion there would be redundant,
/// not a divergence.
fn governed_dependencies(
    package: &Value,
    kind: DependencyKind,
    exclude_self: bool,
) -> impl Iterator<Item = &Value> {
    package["dependencies"]
        .as_array()
        .into_iter()
        .flatten()
        .filter(move |dependency| {
            kind_matches(dependency, kind)
                && !(exclude_self && is_self_dependency(package, dependency))
        })
}

/// Names of the target's dependencies in the selected table that resolve to a registry
/// or git source (any non-null source). Path/internal dependencies, and dependencies in other tables, are
/// excluded.
///
/// Names are package names, not local renames (`foo = { package = "bar" }` is
/// reported as `bar`), and platform-specific (`[target.'cfg(…)'.dependencies]`) and
/// `optional` deps are included — a declared dependency is governed as declared
/// (PROJECT.md).
pub(crate) fn external_dependencies(package: &Value, kind: DependencyKind) -> Vec<String> {
    let mut found: Vec<String> = governed_dependencies(package, kind, false)
        .filter(|dependency| !dependency["source"].is_null())
        .filter_map(|dependency| dependency["name"].as_str().map(str::to_string))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// Whether `dependency` is the package's OWN self-referential edge — a null-`source` **path**
/// dependency on itself (e.g. a doctest/dogfooding `[dev-dependencies]` entry like
/// `main = { path = "." }`). See `crate-dependency-boundary`'s "A crate's own self-referential
/// PATH dependency is never a violation under any crate rule" requirement for why this is never a
/// cross-crate concern, and its "same-named but externally-sourced dependency is NOT exempted"
/// scenario for why `source.is_null()` must gate the name match. Filtering here, at the shared
/// observation source, is what every consuming rule ([`dependencies`] /
/// [`dependencies_with_disallowed_source`]) relies on — a per-rule copy left the identical false
/// positive live in every sibling rule.
fn is_self_dependency(package: &Value, dependency: &Value) -> bool {
    let own_name = package["name"].as_str();
    own_name.is_some() && dependency["name"].as_str() == own_name && dependency["source"].is_null()
}

/// Names of the target's dependencies in the selected table, regardless of source —
/// internal workspace path dependencies included. Used by the forbid and restrict-to
/// rules, which (unlike the external rule) must see internal crate-to-crate
/// dependencies. Same conventions as [`external_dependencies`]: package names (not
/// local renames), and platform-specific / `optional` deps are included (PROJECT.md).
/// Never includes the target's own self-referential edge (see [`is_self_dependency`]).
pub(crate) fn dependencies(package: &Value, kind: DependencyKind) -> Vec<String> {
    let mut found: Vec<String> = governed_dependencies(package, kind, true)
        .filter_map(|dependency| dependency["name"].as_str().map(str::to_string))
        .collect();
    found.sort();
    found.dedup();
    found
}

/// The **import identifiers** a crate's declared dependencies are written under in source: each
/// dependency's `rename` when present (a Cargo `pkg = { package = "…" }` / `dep = { package = "…" }`
/// rename), else its package `name`, normalized `-`→`_` to the Rust path spelling (`async-trait` →
/// `async_trait`). This is the vocabulary the inline confinement's `strict_external` modifier
/// matches a fully-qualified path head against.
///
/// 圭表-own (三儀 ⊥ 三儀 — see the module preamble): a small parallel of
/// `hunyi::crate_scope::dependency_names`, **not** a dependency on 渾儀, reading only the
/// `package["dependencies"]` value 圭表 already obtains via 星表 (so no new crate dependency). Unlike
/// [`dependencies`]/[`external_dependencies`] (which read `name` only), it is rename-aware and
/// `-`→`_`-folded, matching the source spelling.
///
/// **Deliberately unfiltered by kind or source** (unlike [`dependencies`]/[`external_dependencies`]):
/// dev-, build-, and path dependencies are all included. A broader name set makes MORE heads resolve
/// as external, never fewer — the fail-safe direction for the one forbidden bug (a false negative) —
/// while the scope lookup still keeps any genuinely-local item local. The only cost is a
/// possible reaction on a dev/build-dep name used inside scanned test code.
pub(crate) fn dependency_import_names(package: &Value) -> Vec<String> {
    let mut names: Vec<String> = package["dependencies"]
        .as_array()
        .map(|deps| {
            deps.iter()
                .filter_map(|dep| {
                    dep["rename"]
                        .as_str()
                        .or_else(|| dep["name"].as_str())
                        .map(package_name_to_import_ident)
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.dedup();
    names
}

/// The import identifiers of the package's own library targets — what a binary root of the same package
/// names that library by. A target's `name` is its crate name, `[lib] name` when one is declared, so it is
/// read rather than derived from the package name.
pub(crate) fn library_import_names(package: &Value) -> Vec<String> {
    const LIBRARY_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];
    package["targets"]
        .as_array()
        .map(|targets| {
            targets
                .iter()
                .filter(|target| {
                    target["kind"].as_array().is_some_and(|kinds| {
                        kinds
                            .iter()
                            .any(|kind| kind.as_str().is_some_and(|k| LIBRARY_KINDS.contains(&k)))
                    })
                })
                .filter_map(|target| target["name"].as_str().map(package_name_to_import_ident))
                .collect()
        })
        .unwrap_or_default()
}

/// Classify a dependency's **declared** source kind from its `cargo metadata` (`--no-deps`)
/// `source` field: null → `Path`, `git+`-prefixed → `Git`, any other non-null → `Registry` (the
/// residual, covering `registry+`/`sparse+`/alternative registries — see `crate-source-boundary`'s
/// "Declared source kind classified from cargo metadata" requirement for the full rule). Verified
/// against a probe manifest: a `git = "…"` dependency reads `git+…` even with a `version` key,
/// `optional = true`, or a workspace-inherited (`{ workspace = true }`) source.
fn classify_source(dependency: &Value) -> SourceKind {
    match dependency["source"].as_str() {
        None => SourceKind::Path,
        Some(source) if source.starts_with("git+") => SourceKind::Git,
        Some(_) => SourceKind::Registry,
    }
}

/// The **real package names** (not local renames) and declared source kinds of the target's
/// dependencies in the selected table whose classified [`SourceKind`] is not in `allowed` — the
/// observation for [`Rule::RestrictDependencySourcesTo`]. See `crate-source-boundary`'s "A
/// dependency outside the allowed source set is a violation" requirement for the governed-surface
/// and optional-dependency rationale. Never includes the target's own self-referential edge (see
/// [`is_self_dependency`]) — its declared source (always `Path`, a null `source`) is otherwise
/// indistinguishable from a genuine internal dependency.
pub(crate) fn dependencies_with_disallowed_source(
    package: &Value,
    kind: DependencyKind,
    allowed: &[SourceKind],
) -> Vec<(String, SourceKind)> {
    let mut found: Vec<(String, SourceKind)> = governed_dependencies(package, kind, true)
        .filter_map(|dependency| {
            let source = classify_source(dependency);
            if allowed.contains(&source) {
                return None;
            }
            dependency["name"]
                .as_str()
                .map(|name| (name.to_string(), source))
        })
        .collect();
    found.sort_by(|(left_name, left_source), (right_name, right_source)| {
        left_name
            .cmp(right_name)
            .then_with(|| left_source.label().cmp(right_source.label()))
    });
    found.dedup();
    found
}

/// The **declared feature request** the target authors on a dependency `crate_name` in the
/// selected table — the union across every matching edge of its `features = [...]` list plus the
/// pseudo-feature `default` when any such edge leaves default features enabled. See
/// `crate-dependency-boundary`'s "Declared feature-request observation model" requirement for the
/// full union/pseudo-feature/declared-not-resolved rationale. Matches `crate_name` by package
/// name, not a local rename, like [`dependencies`]/[`external_dependencies`]. The target's own
/// self-referential edge (see [`is_self_dependency`]) is never matched either, if `crate_name`
/// happens to name the target's own package — a self-dependency's feature request is not a
/// cross-crate concern this rule governs.
pub(crate) fn declared_features(
    package: &Value,
    crate_name: &str,
    kind: DependencyKind,
) -> Vec<String> {
    let mut found = Vec::new();
    for dependency in matching_dependency_edges(package, crate_name, kind) {
        found.extend(dependency_feature_request(dependency));
    }
    found.sort();
    found.dedup();
    found
}

/// Every dependency-table edge on `crate_name` of the requested `kind` — matched by resolved
/// package name, never the local `rename`/alias — excluding the target's own self-dependency edge
/// (see [`is_self_dependency`]).
fn matching_dependency_edges<'a>(
    package: &'a Value,
    crate_name: &str,
    kind: DependencyKind,
) -> impl Iterator<Item = &'a Value> {
    governed_dependencies(package, kind, true)
        .filter(move |dependency| dependency["name"].as_str() == Some(crate_name))
}

/// The declared feature request for one dependency edge: its explicit `features = [...]` list
/// plus the pseudo-feature `default` when the edge leaves default features enabled. Cargo's edge
/// carries `uses_default_features`; an absent field means defaults are on. Representing "the
/// target requests this dependency's default set" as the pseudo-feature `default` lets one rule
/// shape govern both explicit features and the default toggle (`forbid default` ≡ "require
/// default-features = false").
fn dependency_feature_request(dependency: &Value) -> Vec<String> {
    let mut requested: Vec<String> = dependency["features"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_string)
        .collect();
    if dependency["uses_default_features"].as_bool() != Some(false) {
        requested.push("default".to_string());
    }
    requested
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Classifies declared source field values against standard cargo metadata shapes,
    /// treating null or absent source as Path.
    #[test]
    fn classify_source_reads_the_declared_source_field() {
        assert_eq!(
            classify_source(&json!({ "name": "localdep", "source": null })),
            SourceKind::Path,
            "a null source is a path/internal dependency",
        );
        assert_eq!(
            classify_source(&json!({ "name": "gitdep", "source": "git+https://example.com/x" })),
            SourceKind::Git,
            "a git+ source is git",
        );
        assert_eq!(
            classify_source(&json!({
                "name": "crates_io",
                "source": "registry+https://github.com/rust-lang/crates.io-index"
            })),
            SourceKind::Registry,
            "a registry+ source is registry",
        );
        assert_eq!(
            classify_source(
                &json!({ "name": "alt", "source": "sparse+https://my.registry/index/" })
            ),
            SourceKind::Registry,
            "a sparse+ alternative registry is the residual Registry, not misread as git/path",
        );
        assert_eq!(
            classify_source(&json!({ "name": "no_source_key" })),
            SourceKind::Path,
        );
    }
}
