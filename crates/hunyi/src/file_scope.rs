//! Source-file resolution shared by the capability reactions: the target-crate preamble
//! (`resolve_crate`) every single-crate `check_*_boundary` opens with. Each finding's own `file`
//! metadata is collected directly at the site that produced it (an item's own resolved branch for
//! a single-module capability, or `ImplSite`/`TypeDef`/`UnsafeSite`/the subtree walker's own
//! per-branch file for a whole-crate-scan one) — never re-resolved afterward from a module string,
//! which misattributes a finding whenever two `#[cfg]`-split branches share one module path (see
//! `PROJECT.md`'s Decisions, the round-5 addendum).

use std::path::PathBuf;

use serde_json::Value;

use crate::errors::{crate_not_found_error, missing_src_error};
use xingbiao::{crate_root_file, find_package};

/// Resolve a semantic boundary's target crate to `(package, crate-root file, source dir)` — the
/// shared preamble every single-crate `check_*_boundary` opens with. One home for the three
/// constitution errors (crate-not-found, and missing-src for a target with no crate-root file or a
/// root file with no parent dir) so the eight capabilities cannot drift apart on resolution. The
/// `src_dir` is returned owned (it would otherwise borrow the root file), so callers hold both.
pub(crate) fn resolve_crate<'m>(
    metadata: &'m Value,
    crate_package: &str,
) -> Result<(&'m Value, PathBuf, PathBuf), String> {
    let package = find_package(metadata, crate_package)
        .ok_or_else(|| crate_not_found_error(crate_package))?;
    let root_file = crate_root_file(package).ok_or_else(|| missing_src_error(crate_package))?;
    let src_dir = root_file
        .parent()
        .ok_or_else(|| missing_src_error(crate_package))?
        .to_path_buf();
    Ok((package, root_file, src_dir))
}
