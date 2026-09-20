//! Filesystem traversal for the source scanner: list every `.rs` file under a crate's
//! `src/`, recursively. Symlink-cycle-safe and error-loud — a subtree it cannot read is a
//! scan error, never a silent skip. Pure `std::fs`: no path-vocabulary or parse logic.

use std::path::{Path, PathBuf};

/// All `.rs` files under `dir`, recursively. A directory that cannot be read (or an
/// entry that cannot be resolved) is a scan error, never a silent skip: a skipped
/// subtree could hide a real module-boundary violation — "cannot judge", not "nothing
/// to judge", the same rule as an unreadable governed file.
///
/// Recurses only into real directories via `entry.file_type().is_dir()` (avoiding symlink
/// recursion cycles), but collects symlinked `.rs` source files. Results are sorted for determinism.
pub(crate) fn rust_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|err| {
        format!(
            "cannot read governed source directory '{}': {err}",
            dir.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "cannot read an entry in governed source directory '{}': {err}",
                dir.display()
            )
        })?;
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "cannot stat an entry in governed source directory '{}': {err}",
                dir.display()
            )
        })?;
        let path = entry.path();
        if file_type.is_dir() {
            found.extend(rust_files(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            found.push(path);
        }
    }
    found.sort();
    Ok(found)
}
