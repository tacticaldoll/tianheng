//! Source-file resolution shared by the capability reactions: the target-crate preamble
//! (`resolve_crate_units`) every `check_*_boundary` opens with. Each finding's own `file`
//! metadata is collected directly at the site that produced it (an item's own resolved branch for
//! a single-module capability, or `ImplSite`/`TypeDef`/`UnsafeSite`/the subtree walker's own
//! per-branch file for a whole-crate-scan one) — never re-resolved afterward from a module string,
//! which misattributes a finding whenever two `#[cfg]`-split branches share one module path (see
//! `PROJECT.md`'s Decisions, the round-5 addendum).

use std::path::PathBuf;

use serde_json::Value;

use crate::errors::{crate_not_found_error, missing_src_error, out_of_package_root_error};
use xingbiao::{crate_root_file, find_package};

/// One compilation unit: its root file, that root's own source directory, and the unit's identity label.
pub(crate) type CompilationUnit = (PathBuf, PathBuf, String);

/// Every compilation unit of a package: `(root file, its source directory, the unit's identity role)`.
///
/// The shared preamble every `check_*_boundary` opens with, and one home for the constitution errors
/// resolution can raise — crate-not-found, missing-src (a target with no crate-root file, or a root
/// file with no parent dir), and a root outside the package's own directory — so the eight
/// capabilities cannot drift apart on any of them. Each `src_dir` is owned (it would otherwise borrow
/// its root file), so callers hold both.
///
/// A package builds more than one crate root — a library beside a `bin` — and each is its own module
/// graph. 渾儀 resolves a boundary against each, so a violation written in any of them reacts; governing
/// only the first left the others unobserved. The unit role is the root's path relative to the package's
/// manifest directory, the same value 圭表 uses, so one adopter reads one vocabulary across both static
/// dimensions.
///
/// Unlike 圭表's directory-globbing corpus, this walk descends `mod` declarations from each root, so a
/// sibling root is reached only if a root declares it as a module — no sibling-root exclusion is needed.
pub(crate) fn resolve_crate_units<'m>(
    metadata: &'m Value,
    crate_package: &str,
) -> Result<(&'m Value, Vec<CompilationUnit>), String> {
    let package = find_package(metadata, crate_package)
        .ok_or_else(|| crate_not_found_error(crate_package))?;
    let mut units = Vec::new();
    for root_file in xingbiao::crate_root_files(package) {
        let src_dir = root_file
            .parent()
            .ok_or_else(|| missing_src_error(crate_package))?
            .to_path_buf();
        let unit = xingbiao::compilation_unit_label(package, &root_file)
            .ok_or_else(|| out_of_package_root_error(crate_package, &root_file))?;
        units.push((root_file, src_dir, unit));
    }
    if units.is_empty() {
        // Metadata reporting no target is the shape synthetic metadata in a caller's own tests carries;
        // the single-root resolution below is the fallback, and its unit is the conventional root.
        let root_file = crate_root_file(package).ok_or_else(|| missing_src_error(crate_package))?;
        let src_dir = root_file
            .parent()
            .ok_or_else(|| missing_src_error(crate_package))?
            .to_path_buf();
        let unit = xingbiao::compilation_unit_label(package, &root_file)
            .ok_or_else(|| out_of_package_root_error(crate_package, &root_file))?;
        units.push((root_file, src_dir, unit));
    }
    Ok((package, units))
}

/// Whether a per-unit failure is the one kind that legitimately varies BETWEEN units: the boundary's
/// **anchor** is absent from this root's graph — whichever kind of anchor it is, a governed module or a
/// governed trait.
///
/// A package's roots are separate compilation units, so a library's internals are not the binary's — a
/// boundary anchored at `crate::api` is real for the library root and meaningless for a `src/bin/*.rs`
/// root beside it. Erroring per root would refuse to judge source that compiles; the caller therefore
/// defers this one failure and reports it only if NO unit hosts the module.
///
/// The caller passes its own canonical absence error rather than a substring, so a change to the message
/// moves both sides together. Every OTHER failure — an unreadable source, a resolution ambiguity, a root
/// outside the package directory — propagates immediately: deferring it until a sibling unit happened to
/// be governable would silently pass over source the system could not read.
pub(crate) fn is_anchor_absent_from_unit(err: &str, canonical_absence: &str) -> bool {
    err == canonical_absence
}
