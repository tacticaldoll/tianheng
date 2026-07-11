//! Source-file resolution shared by the capability reactions: the target-crate preamble
//! (`resolve_crate`) every single-crate `check_*_boundary` opens with, and the two finding-file
//! renderers — [`seam_file`] for a single-module violation and [`per_finding_file`] for the
//! whole-crate scans whose findings each name their own module. One home so the capabilities
//! cannot drift apart on crate/module → file resolution.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::errors::{crate_not_found_error, missing_src_error};
use crate::module_resolve::resolve_module_file;
use xingbiao::{crate_root_file, find_package};

/// The governed module's source file rendered for a single-module semantic violation's `file`
/// (`display()`-rendered to match the static dimension). Resolved **only when there is
/// something to report**, so a clean module never pays the second traversal and no error path
/// opens on an empty result; `None` when there are no findings. Shared by the five
/// single-module semantic capabilities (exposure, dyn-trait, impl-trait, async-exposure,
/// visibility). The three whole-crate-scan capabilities (trait-impl-locality, forbidden-marker,
/// unsafe-confinement) do NOT use this — their violations sit at per-site files across the crate,
/// a stated `null` bound narrowed to them.
pub(crate) fn seam_file(
    findings: &[String],
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Option<String>, String> {
    if findings.is_empty() {
        return Ok(None);
    }
    Ok(Some(
        resolve_module_file(src_dir, root_file, module, crate_package)?
            .display()
            .to_string(),
    ))
}

/// The source file for a **whole-crate-scan** semantic violation (trait-impl-locality,
/// forbidden-marker), whose findings each name their own module — unlike [`seam_file`]'s single
/// per-boundary module. Memoized in `cache` so a boundary with many findings across few modules
/// parses each module path once. Degrades to `None` on a resolution failure (**`.ok()`, never
/// `?`**): the module comes from the whole-crate scan while the file comes from the single-path
/// resolver, so — though they agree for every module a finding can come from — a failure must
/// leave the violation firing with a `null` file, never turn it into an exit-2 error or drop it.
pub(crate) fn per_finding_file(
    module: &str,
    src_dir: &Path,
    root_file: &Path,
    crate_package: &str,
    cache: &mut HashMap<String, Option<String>>,
) -> Option<String> {
    if let Some(cached) = cache.get(module) {
        return cached.clone();
    }
    let file = resolve_module_file(src_dir, root_file, module, crate_package)
        .ok()
        .map(|path| path.display().to_string());
    cache.insert(module.to_string(), file.clone());
    file
}

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
