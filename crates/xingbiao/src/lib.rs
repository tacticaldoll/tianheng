//! 星表 (xīngbiǎo) — the shared declared-workspace-data substrate.
//!
//! Reads `cargo metadata --no-deps` and looks up packages and their crate-root source files:
//! the tabulated catalog every observation dimension references before it observes. Spawns
//! `cargo` and parses its JSON; **`serde_json` + std only, no `syn`**.
//!
//! It sits **below the 三儀**, like 璇璣 (the reaction model): a dimension depends on it
//! one-way (downward), so the static (圭表) and semantic (渾儀) dimensions read the workspace
//! through **one** reader instead of two hand-copied twins that drift apart (the twin-drift
//! bug class — e.g. one copy learning to resolve a `proc-macro` crate's root while its sibling
//! did not). It is **not 璇璣**: 璇璣 is the measure-only reaction model that renders no verdict,
//! whereas 星表 does IO (spawns `cargo`) and *observes*. Sharing it does not couple the
//! dimensions to each other — they compose only through the 天衡 shell; 星表 is a substrate
//! beneath them, named in each dependent's dependency allowlist.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// Run `cargo metadata --no-deps --format-version 1` for the workspace at `manifest_path`
/// and parse its JSON. `--no-deps` restricts `packages` to the workspace members (no
/// transitive dependencies). A non-zero `cargo` exit is returned as its trimmed stderr.
pub fn cargo_metadata(manifest_path: &Path) -> Result<Value, String> {
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
        // Prefer cargo's own stderr, but fall back to the exit status when it is empty — a signal
        // kill (OOM) or a silent non-zero exit would otherwise yield an unactionable empty error.
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("cargo metadata failed: {}", output.status)
        } else {
            stderr
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

/// Find a workspace member by package name in the parsed `metadata`.
pub fn find_package<'a>(metadata: &'a Value, package: &str) -> Option<&'a Value> {
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|candidate| candidate["name"].as_str() == Some(package))
}

/// The crate's root source file, observed from `cargo metadata`: the **library** target's
/// `src_path` (any of Cargo's library crate types), else a `bin` target's.
///
/// This is the **single** resolution both the static and semantic dimensions use — the whole
/// reason 星表 exists. The library crate types are `lib`, `rlib`, `dylib`, `cdylib`, `staticlib`,
/// and `proc-macro`, and `cargo metadata` reports a target's `kind` as its crate type: a
/// `crate-type = ["cdylib"]` (or `["staticlib"]` / `["rlib"]`) library reports `kind` as exactly
/// `["cdylib"]` — with **no** `"lib"` element — and a proc-macro crate as `["proc-macro"]`. Matching
/// only `"lib"` would drop such a crate to no root file even though it has a conventional
/// `src/lib.rs`, and a dimension reading through it would either raise a false `missing_src` (the
/// semantic exit-2) or silently drop the crate from its corpus (a runtime-audit false negative).
/// Both faces read the *same* function, so they cannot disagree on which crates are judgeable — the
/// divergence that used to live in two hand-copied bodies is structurally gone.
///
/// Bound (stated, not silent): a member with **only** a non-library, non-`bin` target — a `test`,
/// `example`, `bench`, or `custom-build` — resolves to `None` and is skipped; it carries no
/// crate-root source to govern.
pub fn crate_root_file(package: &Value) -> Option<PathBuf> {
    // Cargo's library crate types; a `[lib]` target reports its `kind` as these (never a plain
    // `"lib"` when a non-default `crate-type` is declared), so a library crate is any target whose
    // kind is one of them — preferred over a `bin`, which is the crate root only for a bin-only
    // member.
    const LIB_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];
    let targets = package["targets"].as_array()?;
    let has_kind = |target: &Value, wanted: &str| {
        target["kind"]
            .as_array()
            .map(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
            .unwrap_or(false)
    };
    let pick = targets
        .iter()
        .find(|t| LIB_KINDS.iter().any(|k| has_kind(t, k)))
        .or_else(|| targets.iter().find(|t| has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}

/// Each workspace member's source-root directory — the parent of its [`crate_root_file`]. A
/// member whose root cannot be resolved is skipped (it carries no `lib`/`proc-macro`/`bin` source).
/// Deduped and sorted for a deterministic corpus. This is a pure derivation of [`crate_root_file`],
/// so it lives beside it: a dimension (or the shell composing the runtime-audit corpus) that needs
/// every member's src root gets one resolution, not a re-derived twin.
pub fn member_src_dirs(metadata: &Value) -> Vec<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn find_package_selects_by_name() {
        let metadata = json!({ "packages": [
            { "name": "a", "targets": [] },
            { "name": "b", "targets": [] },
        ]});
        assert_eq!(find_package(&metadata, "b").unwrap()["name"], json!("b"));
        assert!(find_package(&metadata, "missing").is_none());
    }

    #[test]
    fn crate_root_file_prefers_lib_then_proc_macro_then_bin() {
        let lib_and_bin = json!({ "targets": [
            { "kind": ["bin"], "src_path": "/w/src/main.rs" },
            { "kind": ["lib"], "src_path": "/w/src/lib.rs" },
        ]});
        assert_eq!(
            crate_root_file(&lib_and_bin),
            Some(PathBuf::from("/w/src/lib.rs")),
            "the lib target wins over bin"
        );

        let bin_only = json!({ "targets": [{ "kind": ["bin"], "src_path": "/w/src/main.rs" }] });
        assert_eq!(
            crate_root_file(&bin_only),
            Some(PathBuf::from("/w/src/main.rs"))
        );
    }

    #[test]
    fn crate_root_file_resolves_a_proc_macro_target() {
        // A proc-macro crate's target kind is `["proc-macro"]` (never lib/bin); it must still
        // resolve its root file so the dimensions govern it, not skip it or raise a false
        // missing-src. This is the twin-drift the SSOT closes: one dimension used to resolve it
        // while its sibling did not.
        let package = json!({ "targets": [
            { "kind": ["proc-macro"], "src_path": "/w/src/lib.rs" }
        ]});
        assert_eq!(
            crate_root_file(&package),
            Some(PathBuf::from("/w/src/lib.rs"))
        );
    }

    #[test]
    fn crate_root_file_skips_a_member_with_no_lib_proc_macro_or_bin() {
        let bench_only =
            json!({ "targets": [{ "kind": ["bench"], "src_path": "/w/benches/b.rs" }] });
        assert_eq!(crate_root_file(&bench_only), None);
        let rootless = json!({ "targets": [] });
        assert_eq!(crate_root_file(&rootless), None);
    }

    #[test]
    fn crate_root_file_resolves_a_cdylib_staticlib_or_rlib_library() {
        // A `crate-type = ["cdylib"]` / `["staticlib"]` / `["rlib"]` library reports its target
        // `kind` as exactly that crate type — with no "lib" element (verified against real
        // `cargo metadata`). It is a real library crate with a governable src/lib.rs, so it must
        // resolve, not drop to None (which caused a false semantic missing-src and a runtime-audit
        // corpus drop).
        for kind in [["cdylib"], ["staticlib"], ["rlib"], ["dylib"]] {
            let package = json!({ "targets": [{ "kind": kind, "src_path": "/w/src/lib.rs" }] });
            assert_eq!(
                crate_root_file(&package),
                Some(PathBuf::from("/w/src/lib.rs")),
                "a {kind:?} library must resolve its crate root"
            );
        }
        // A multi-crate-type lib reports one target with all its kinds; still one lib root.
        let multi =
            json!({ "targets": [{ "kind": ["cdylib", "rlib"], "src_path": "/w/src/lib.rs" }] });
        assert_eq!(
            crate_root_file(&multi),
            Some(PathBuf::from("/w/src/lib.rs"))
        );
        // A library kind is preferred over a bin (the crate root is the lib).
        let lib_and_bin = json!({ "targets": [
            { "kind": ["bin"], "src_path": "/w/src/main.rs" },
            { "kind": ["cdylib"], "src_path": "/w/src/lib.rs" },
        ]});
        assert_eq!(
            crate_root_file(&lib_and_bin),
            Some(PathBuf::from("/w/src/lib.rs"))
        );
    }

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
