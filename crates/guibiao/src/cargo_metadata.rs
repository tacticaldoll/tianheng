use super::*;
use serde_json::Value;

// The dimension-agnostic cargo-metadata reads (`cargo_metadata`, `find_package`, `crate_root_file`,
// and `member_src_dirs`, a pure derivation of the latter) live in 星表 (`xingbiao`), the shared
// substrate below the 三儀 — one reader, so the static and semantic dimensions cannot drift apart on
// how they read the workspace. 圭表 keeps only its own *observation semantics* below (dependency
// source/kind, workspace membership), which are not neutral infrastructure.
pub(crate) use xingbiao::{cargo_metadata, crate_root_file, find_package, member_src_dirs};

/// The names of the workspace's member crates. Because Modou runs
/// `cargo metadata --no-deps`, the `packages` array contains exactly the workspace
/// members (no transitive dependencies), so their names are the membership set used
/// by the workspace-scoped rule and by coverage. A `path` dependency that points
/// outside the workspace is therefore absent here, as intended.
pub(crate) fn workspace_member_names(metadata: &Value) -> Vec<String> {
    let mut names: Vec<String> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .filter_map(|package| package["name"].as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.dedup();
    names
}

/// Whether a `cargo metadata` dependency belongs to the selected table. `kind` is
/// null for normal deps, `"dev"` / `"build"` otherwise.
fn kind_matches(dependency: &Value, kind: DependencyKind) -> bool {
    // An unrecognized `kind` string (none exist today — cargo emits only null/dev/build)
    // matches no `DependencyKind`, so such a dependency is observed by no boundary. This
    // is deliberate and bounded: `DependencyKind` does not grow (see its model doc), so a
    // new cargo table is a conscious amendment, not a silent gap to defend here.
    matches!(
        (kind, dependency["kind"].as_str()),
        (DependencyKind::Normal, None)
            | (DependencyKind::Dev, Some("dev"))
            | (DependencyKind::Build, Some("build"))
    )
}

/// Names of the target's dependencies in the selected table that resolve to a registry
/// or git source. Path/internal dependencies, and dependencies in other tables, are
/// excluded.
///
/// Names are package names, not local renames (`foo = { package = "bar" }` is
/// reported as `bar`), and platform-specific (`[target.'cfg(…)'.dependencies]`) and
/// `optional` deps are included — a declared dependency is governed as declared
/// (PROJECT.md).
pub(crate) fn external_dependencies(package: &Value, kind: DependencyKind) -> Vec<String> {
    let mut found = Vec::new();
    if let Some(dependencies) = package["dependencies"].as_array() {
        for dependency in dependencies {
            if !kind_matches(dependency, kind) {
                continue;
            }
            // A path/internal dependency has a null `source`; any non-null source is
            // external. Match on presence, not on a fixed `registry+`/`git+` prefix
            // list, so a dependency from an alternative (e.g. `sparse+`) registry
            // cannot slip through unclassified and silently pass the boundary.
            let external = !dependency["source"].is_null();
            if external {
                // A dependency always carries a string `name` in cargo's metadata schema;
                // a present-but-non-string `name` (unexpected shape) is skipped rather
                // than failed. This relies on the schema guarantee — if it could be
                // violated, the loud path would be a scan error, not a silent skip.
                if let Some(name) = dependency["name"].as_str() {
                    found.push(name.to_string());
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// Names of the target's dependencies in the selected table, regardless of source —
/// internal workspace path dependencies included. Used by the forbid and restrict-to
/// rules, which (unlike the external rule) must see internal crate-to-crate
/// dependencies. Same conventions as [`external_dependencies`]: package names (not
/// local renames), and platform-specific / `optional` deps are included (PROJECT.md).
pub(crate) fn dependencies(package: &Value, kind: DependencyKind) -> Vec<String> {
    let mut found = Vec::new();
    if let Some(deps) = package["dependencies"].as_array() {
        for dependency in deps {
            if !kind_matches(dependency, kind) {
                continue;
            }
            if let Some(name) = dependency["name"].as_str() {
                found.push(name.to_string());
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

/// The **import identifiers** a crate's declared dependencies are written under in source: each
/// dependency's `rename` when present (a Cargo `pkg = { package = "…" }` / `dep = { package = "…" }`
/// rename), else its package `name`, normalized `-`→`_` to the Rust path spelling (`async-trait` →
/// `async_trait`). This is the vocabulary the strict-external inline confinement
/// (`ModuleRule::ConfineInlineSymbolPathExternal`) matches a fully-qualified path head against.
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
/// while the local-precedence ladder still keeps any genuinely-local item local. The only cost is a
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
                        .map(|name| name.replace('-', "_"))
                })
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names.dedup();
    names
}

/// Classify a dependency's **declared** source kind from its `cargo metadata`
/// (`--no-deps`) `source` field. Mirrors [`external_dependencies`]'s convention (a
/// null source is internal/path) one notch finer:
///
/// - **null** source → `Path` (a `path = "…"` / internal dependency)
/// - source beginning **`git+`** → `Git` (cargo spells a declared git source `git+<url>`)
/// - any other non-null source → `Registry` (the residual: `registry+`, `sparse+`, and
///   alternative registries, so a new registry scheme classifies correctly with no code
///   change — the same robustness `external_dependencies` relies on)
///
/// Only `Git` is matched by a positive prefix and only `Path` by null; `Registry` is the
/// residual. Verified against `cargo metadata --no-deps` on a probe manifest: a
/// `git = "…"` dependency reads `source = "git+…"` **even with a `version` key and even
/// when `optional = true`**, and a workspace-**inherited** git dependency
/// (`{ workspace = true }`) reads `git+…` too (cargo flattens the inherited source into
/// the member's manifest). A path dependency reads `source = null`. The read is hermetic
/// — a pure function of the manifests, no lockfile and no network.
fn classify_source(dependency: &Value) -> SourceKind {
    match dependency["source"].as_str() {
        None => SourceKind::Path,
        Some(source) if source.starts_with("git+") => SourceKind::Git,
        Some(_) => SourceKind::Registry,
    }
}

/// The **real package names** (not local renames) of the target's dependencies in the
/// selected table whose classified [`SourceKind`] is not in `allowed`. The observation
/// for [`Rule::RestrictDependencySourcesTo`]. Same conventions as [`dependencies`]:
/// walks every declared dependency of the kind (path/internal included, since `Path` is
/// a governed source), reports the package name (a renamed dep by its real name), and
/// includes `optional` deps — a declared source is governed as declared (PROJECT.md), and
/// an optional git dependency blocks publishing just as a required one does. An empty
/// `allowed` set flags every dependency of the kind.
pub(crate) fn dependencies_with_disallowed_source(
    package: &Value,
    kind: DependencyKind,
    allowed: &[SourceKind],
) -> Vec<String> {
    let mut found = Vec::new();
    if let Some(deps) = package["dependencies"].as_array() {
        for dependency in deps {
            if !kind_matches(dependency, kind) {
                continue;
            }
            if !allowed.contains(&classify_source(dependency)) {
                if let Some(name) = dependency["name"].as_str() {
                    found.push(name.to_string());
                }
            }
        }
    }
    found.sort();
    found.dedup();
    found
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classify_source_reads_the_declared_source_field() {
        // The three classifications, against the exact source strings `cargo metadata
        // --no-deps` emits (verified on a probe manifest).
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
        // An absent `source` key (Value::Null) classifies as Path, like a null one.
        assert_eq!(
            classify_source(&json!({ "name": "no_source_key" })),
            SourceKind::Path,
        );
    }
}
