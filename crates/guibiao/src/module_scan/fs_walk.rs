//! Filesystem traversal for the source scanner: list every `.rs` file under a crate's
//! `src/`, recursively. Symlink-cycle-safe and error-loud — a subtree it cannot read is a
//! scan error, never a silent skip. Pure `std::fs`: no path-vocabulary or parse logic.

use std::path::{Path, PathBuf};

/// All `.rs` files under `dir`, recursively. A directory that cannot be read (or an
/// entry that cannot be resolved) is a scan error, never a silent skip: a skipped
/// subtree could hide a real module-boundary violation — "cannot judge", not "nothing
/// to judge", the same rule as an unreadable governed file.
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
        // Recurse only into a real directory: `file_type()` does NOT follow symlinks (unlike
        // `path.is_dir()`, which stats the target), so a symlinked directory (a cyclic
        // `src/loop -> .`) is not entered — avoiding an unbounded recursion → stack overflow.
        // Matches louke's probe scanner, which guards the same hazard the same way.
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
            // A `.rs` file — including a symlink whose target is a real file: rustc compiles such a
            // `mod`-declared source (and `read_to_string` follows the symlink to it), so it is
            // governed. Not gated on `file_type.is_file()`, which would drop a symlinked source and
            // silently miss its imports; a symlinked *directory* is already excluded above.
            found.push(path);
        }
    }
    // Sort so the governed-file order — and hence module-violation order in the report —
    // is deterministic, independent of the filesystem's `read_dir` order.
    found.sort();
    Ok(found)
}
