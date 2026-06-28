use super::*;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;

pub(crate) fn cargo_metadata(manifest_path: &Path) -> Result<Value, String> {
    let output = Command::new("cargo")
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--manifest-path",
        ])
        .arg(manifest_path)
        .output()
        .map_err(|err| err.to_string())?;

    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_string());
    }

    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

pub(crate) fn find_package<'a>(metadata: &'a Value, package: &str) -> Option<&'a Value> {
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|candidate| candidate["name"].as_str() == Some(package))
}

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

/// A crate's root source file: the `lib` target's `src_path`, else a `bin` target's,
/// observed from `cargo metadata`. This is the same resolution the 渾儀 (semantic)
/// dimension uses — the dimensions cannot share code (三儀 ⊥ 三儀 forbids a cross-dimension
/// dependency), so they agree by using the same algorithm, not the same function. It is NOT
/// the `manifest_dir/src` shortcut, which is wrong for a custom `[lib] path` or a bin-only
/// crate — a member whose real source root is missed would let a probe there escape the
/// runtime CI audit (a false negative).
///
/// Bound (stated, not silent): a member with **only** a `proc-macro`, `test`, `example`, or
/// `bench` target — no `lib`/`bin` — resolves to `None` and is skipped. Such a member's source
/// is out of the runtime-audit corpus, the same lib/bin-subtree bound the semantic dimension
/// has; declaring a runtime seam probed only from there is unsupported by design.
fn crate_root_file(package: &Value) -> Option<PathBuf> {
    let targets = package["targets"].as_array()?;
    let has_kind = |target: &Value, wanted: &str| {
        target["kind"]
            .as_array()
            .map(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
            .unwrap_or(false)
    };
    let pick = targets
        .iter()
        .find(|t| has_kind(t, "lib"))
        .or_else(|| targets.iter().find(|t| has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}

/// Each workspace member's source-root directory (the parent of its [`crate_root_file`]).
/// Members whose root cannot be resolved are skipped (they carry no lib/bin source to scan).
/// Deduped and sorted for a deterministic corpus.
pub(crate) fn member_src_dirs(metadata: &Value) -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .filter_map(crate_root_file)
                .filter_map(|root| root.parent().map(Path::to_path_buf))
                .collect()
        })
        .unwrap_or_default();
    dirs.sort();
    dirs.dedup();
    dirs
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn member_src_dirs_resolves_from_src_path_including_a_custom_layout() {
        // crate_a: conventional src/lib.rs → src dir is .../crate_a/src
        // crate_b: a custom [lib] path = "lib.rs" (root at the manifest dir, NOT under src/)
        //          → resolving via manifest_dir/src would WRONGLY miss it; src_path is right.
        // crate_c: bin-only with src/main.rs.
        let metadata = json!({
            "packages": [
                { "name": "crate_a", "targets": [
                    { "kind": ["lib"], "src_path": "/ws/crate_a/src/lib.rs" }
                ]},
                { "name": "crate_b", "targets": [
                    { "kind": ["lib"], "src_path": "/ws/crate_b/lib.rs" }
                ]},
                { "name": "crate_c", "targets": [
                    { "kind": ["bin"], "src_path": "/ws/crate_c/src/main.rs" }
                ]},
            ]
        });
        let dirs = member_src_dirs(&metadata);
        assert!(dirs.contains(&PathBuf::from("/ws/crate_a/src")), "{dirs:?}");
        assert!(
            dirs.contains(&PathBuf::from("/ws/crate_b")),
            "a custom [lib] path must resolve to its real root, not manifest_dir/src: {dirs:?}"
        );
        assert!(dirs.contains(&PathBuf::from("/ws/crate_c/src")), "{dirs:?}");
    }

    #[test]
    fn member_src_dirs_prefers_lib_over_bin_and_skips_rootless_members() {
        let metadata = json!({
            "packages": [
                { "name": "both", "targets": [
                    { "kind": ["bin"], "src_path": "/ws/both/src/main.rs" },
                    { "kind": ["lib"], "src_path": "/ws/both/src/lib.rs" }
                ]},
                { "name": "rootless", "targets": [] },
            ]
        });
        let dirs = member_src_dirs(&metadata);
        // The lib target wins; both targets share the same src dir here, so one entry.
        assert_eq!(dirs, vec![PathBuf::from("/ws/both/src")], "{dirs:?}");
    }
}
