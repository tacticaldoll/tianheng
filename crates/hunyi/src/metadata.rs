//! `cargo metadata` IO and package/crate-root lookup — the observation source the semantic
//! checks and module resolution read the workspace through. Spawns `cargo` and parses its JSON;
//! `serde_json` + std only, no `syn`.

use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

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

/// The crate's root source file (the `lib` target's `src_path`, else a `proc-macro` target's, else
/// a `bin` target's), observed from `cargo metadata`. A proc-macro crate's target kind is
/// `["proc-macro"]` (never `lib`/`bin`), so without it such a crate would resolve to no root file
/// and the semantic reaction would raise a false `missing_src` (exit 2) on a crate that plainly has
/// `src/lib.rs` — the 圭表 static dimension already governs it via its own `src` fallback, so this
/// keeps the two dimensions agreeing on which crates are judgeable.
pub(crate) fn crate_root_file(package: &Value) -> Option<PathBuf> {
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
        .or_else(|| targets.iter().find(|t| has_kind(t, "proc-macro")))
        .or_else(|| targets.iter().find(|t| has_kind(t, "bin")))?;
    pick["src_path"].as_str().map(PathBuf::from)
}
