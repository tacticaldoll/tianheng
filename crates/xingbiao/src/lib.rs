//! 星表 (xīngbiǎo) — the shared declared-workspace-data substrate.
//!
//! Reads `cargo metadata --no-deps` and looks up packages and their crate-root source files:
//! the tabulated catalog every observation dimension references before it observes. Spawns
//! `cargo` and parses its JSON (`serde_json` + std only, no `syn`).
//!
//! Sits beneath static (圭表) and semantic (渾儀) dimensions as a single reader of truth,
//! preventing twin-drift in target resolution across observation dimensions.
#![forbid(unsafe_code)]
#![deny(missing_docs)]

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

#[cfg(test)]
mod tests;

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

/// Resolve a crate's root source file from `cargo metadata` (library target else `bin` target).
pub fn crate_root_file(package: &Value) -> Option<PathBuf> {
    let targets = package["targets"].as_array()?;
    let has_kind = |target: &Value, wanted: &str| {
        target["kind"]
            .as_array()
            .is_some_and(|kinds| kinds.iter().any(|k| k.as_str() == Some(wanted)))
    };
    let pick = targets
        .iter()
        .find(|t| LIBRARY_KINDS.iter().any(|k| has_kind(t, k)))
        .or_else(|| targets.iter().find(|t| has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
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
                            target["kind"].as_array().is_some_and(|kinds| {
                                kinds.iter().any(|kind| {
                                    kind.as_str()
                                        .is_some_and(|k| LIBRARY_KINDS.contains(&k) || k == "bin")
                                })
                            })
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
