//! Shared path-identity primitives for a cycle/dedup guard: every observation dimension that
//! walks a module graph or a `#[path]` remap needs to know "is this file the same file I have
//! already opened", and the answer must be the resolved (symlink-following) identity, never the
//! literal path string — two literal paths can name one real file. Centralized here so 圭表 and
//! 渾儀's independently hand-rolled cycle guards cannot drift to different canonicalize-failure
//! policies the way they once did (the 0.2.2 lesson: the same guard, reimplemented per call site,
//! settled on three disagreeing policies in one file alone).

use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Canonicalize `path` (resolving symlinks) for a cycle/dedup identity. Fails loud — never a
/// silent skip — so an unresolvable path surfaces as a scan error instead of quietly admitting a
/// weakened guard (the single decision this crate makes on a dimension's behalf; a dimension that
/// wants a different failure posture composes its own handling around this, but must not
/// reimplement the canonicalize call itself).
pub fn canonicalize_or_fail(path: &Path) -> Result<PathBuf, String> {
    std::fs::canonicalize(path).map_err(|err| format!("cannot resolve '{}': {err}", path.display()))
}

/// Canonicalize `path` and record it in `visited`, reporting whether this is its first visit
/// (`Ok(true)`) or a repeat (`Ok(false)`) — the shape behind every plain "have I already walked
/// this file" dedup guard. `Err` only when the path itself cannot be resolved (see
/// [`canonicalize_or_fail`]); a cycle-specific reaction (as opposed to a plain dedup skip) is the
/// caller's own decision once it sees `Ok(false)`.
pub fn try_visit(visited: &mut HashSet<PathBuf>, path: &Path) -> Result<bool, String> {
    Ok(visited.insert(canonicalize_or_fail(path)?))
}
