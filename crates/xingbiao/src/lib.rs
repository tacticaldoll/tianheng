//! 星表 (xīngbiǎo) — the shared declared-workspace-data substrate.
//!
//! Reads `cargo metadata --no-deps` and looks up packages and their crate-root source files:
//! the tabulated catalog every observation dimension references before it observes. Spawns
//! `cargo` and parses its JSON (`serde_json` + std only, no `syn`). Also carries the shared
//! path-identity primitives ([`canonicalize_or_fail`], [`try_visit`]) a module-graph cycle/dedup
//! guard needs — the same "single reader of truth" role, one file-identity notch finer than
//! which file is a crate root.
//!
//! Sits beneath static (圭表) and semantic (渾儀) dimensions as a single reader of truth,
//! preventing twin-drift in target resolution across observation dimensions.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

mod path_identity;

#[cfg(test)]
mod tests;

pub use path_identity::{canonicalize_or_fail, try_visit};

/// Target `kind` strings that denote a library crate root (library types + `proc-macro`).
const LIBRARY_KINDS: [&str; 6] = ["lib", "rlib", "dylib", "cdylib", "staticlib", "proc-macro"];

/// Run `cargo metadata --no-deps --format-version 1` for the workspace at `manifest_path`.
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
        let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
        return Err(if stderr.is_empty() {
            format!("cargo metadata failed: {}", output.status)
        } else {
            stderr
        });
    }
    serde_json::from_slice(&output.stdout).map_err(|err| err.to_string())
}

/// Find a workspace member package by name in parsed metadata.
pub fn find_package<'a>(metadata: &'a Value, package: &str) -> Option<&'a Value> {
    metadata["packages"]
        .as_array()?
        .iter()
        .find(|candidate| candidate["name"].as_str() == Some(package))
}

/// Whether a `cargo metadata` target's `kind` array contains `wanted` — the one-target shape
/// check shared by [`crate_root_file`] (picking one library/bin target) and
/// [`member_root_files`] (filtering every library/bin target across the workspace).
fn target_has_kind(target: &Value, wanted: &str) -> bool {
    target["kind"]
        .as_array()
        .is_some_and(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
}

/// Resolve a crate's root source file from `cargo metadata` (library target else `bin` target).
pub fn crate_root_file(package: &Value) -> Option<PathBuf> {
    let targets = package["targets"].as_array()?;
    let pick = targets
        .iter()
        .find(|t| LIBRARY_KINDS.iter().any(|k| target_has_kind(t, k)))
        .or_else(|| targets.iter().find(|t| target_has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}

/// The workspace root directory Cargo resolved for this metadata read — the directory holding the
/// workspace manifest, whichever member manifest `--manifest-path` happened to name (Cargo resolves
/// upward to the same root either way).
///
/// Read for its **stability**, not for locating anything: it is the one directory that does not move
/// when a workspace gains, loses, or relocates a member, which is what makes it the right thing to
/// label an observed file *relative to* when that label is baseline identity (see 漏刻's
/// `audit_probe_coverage` anchor). A path derived from the observed member set instead — their
/// longest common prefix, say — is checkout-independent yet shifts the moment the set does, silently
/// restating every recorded label.
///
/// `None` when the field is absent, which real `cargo metadata` output always carries; a caller
/// holding synthetic metadata is expected to supply its own anchor rather than receive a guess.
pub fn workspace_root(metadata: &Value) -> Option<PathBuf> {
    metadata["workspace_root"].as_str().map(PathBuf::from)
}

/// Workspace member source-root directories (deduplicated and sorted).
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

/// Every workspace member library, proc-macro, and binary crate-root source file reported by Cargo.
pub fn member_root_files(metadata: &Value) -> Vec<PathBuf> {
    let mut roots: Vec<PathBuf> = metadata["packages"]
        .as_array()
        .map(|packages| {
            packages
                .iter()
                .flat_map(|package| {
                    package["targets"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|target| {
                            LIBRARY_KINDS.iter().any(|k| target_has_kind(target, k))
                                || target_has_kind(target, "bin")
                        })
                        .filter_map(|target| target["src_path"].as_str().map(PathBuf::from))
                })
                .collect()
        })
        .unwrap_or_default();
    roots.sort();
    roots.dedup();
    roots
}
