//! Source-file resolution shared by the capability reactions: the target-crate preamble
//! (`resolve_crate_units`) every `check_*_boundary` opens with. Each finding's own `file`
//! metadata is collected directly at the site that produced it (an item's own resolved branch for
//! a single-module capability, or `ImplSite`/`TypeDef`/`UnsafeSite`/the subtree walker's own
//! per-branch file for a whole-crate-scan one) — never re-resolved afterward from a module string,
//! which misattributes a finding whenever two `#[cfg]`-split branches share one module path.

use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::errors::{
    crate_not_found_error, missing_src_error, no_compiled_root_error, out_of_package_root_error,
};
use xingbiao::find_package;

/// One compilation unit: its root file, that root's own source directory, and the unit's identity label.
pub(crate) type CompilationUnit = (PathBuf, PathBuf, String);

/// Every compilation unit of a package: `(root file, its source directory, the unit's identity role)`.
///
/// The shared preamble every `check_*_boundary` opens with, and one home for the constitution errors
/// resolution can raise — crate-not-found, no-compiled-root (every reported target an example, a test,
/// a bench or a build script), missing-src (metadata reporting no target, or a root file with no
/// parent dir), and a root outside the package's own directory — so no capability can
/// drift from another on any of them. Each `src_dir` is owned (it would otherwise borrow
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
    let roots = match xingbiao::crate_roots(package) {
        xingbiao::CrateRoots::Compiled(roots) => roots,
        xingbiao::CrateRoots::NoneCompiled => {
            return Err(no_compiled_root_error(crate_package));
        }
        xingbiao::CrateRoots::Unreported => {
            return Err(missing_src_error(crate_package));
        }
    };
    for root_file in roots.as_slice().iter().cloned() {
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

/// The anchor a boundary evaluates against a compilation unit.
///
/// A boundary is anchored either at a module (for module-level or subtree boundaries)
/// or at a trait (for trait-implementation locality). Each kind carries its canonical path
/// and package name so an absence error can be checked for exact identity against this anchor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UnitAnchor<'a> {
    Module {
        module: &'a str,
        crate_package: &'a str,
    },
    Trait {
        trait_path: &'a str,
        crate_package: &'a str,
    },
}

/// Whether a per-unit failure is the one kind that legitimately varies BETWEEN units: the boundary's
/// **anchor** is absent from this root's graph — matching the exact anchor, not any wildcard.
///
/// A package's roots are separate compilation units, so a library's internals are not the binary's — a
/// boundary anchored at `crate::api` is real for the library root and meaningless for a `src/bin/*.rs`
/// root beside it. Erroring per root would refuse to judge source that compiles; the caller therefore
/// defers this one failure and reports it only if NO unit hosts the anchor.
///
/// A module anchor's absence is decided before the checker runs, by [`over_each_unit`]'s pre-check, so a
/// unit that lacks the module never reaches this predicate. A trait anchor's absence is known only to
/// the checker's own resolution, so for a trait this predicate is where [`over_each_unit`] matches the
/// error the closure returned.
///
/// The deferral check matches the exact anchor rather than using wildcards: an error naming a
/// different module or trait is an unexpected resolution failure that must propagate immediately.
pub(crate) fn is_anchor_absent_from_unit(
    err: &crate::errors::ResolveError,
    anchor: UnitAnchor<'_>,
) -> bool {
    match (err, anchor) {
        (
            crate::errors::ResolveError::UnresolvableModule(m, c),
            UnitAnchor::Module {
                module,
                crate_package,
            },
        ) => m == module && c == crate_package,
        (
            crate::errors::ResolveError::UnknownTrait(t, c),
            UnitAnchor::Trait {
                trait_path,
                crate_package,
            },
        ) => t == trait_path && c == crate_package,
        _ => false,
    }
}

/// Evaluate `per_unit` over every compilation unit of a package, deferring an anchor that is absent from
/// one unit but present in another.
///
/// For a module anchor ([`UnitAnchor::Module`]), existence is pre-checked directly against each unit.
/// If absent from a unit, the failure is deferred without running the checker closure. If present,
/// the closure is invoked. For a trait anchor ([`UnitAnchor::Trait`]), the closure executes and any
/// absence error matching the anchor is deferred.
///
/// A package's crate roots are separate compilation units — same `crate` module path, separate module
/// graph — so each is evaluated on its own and the unit is carried into each finding's identity. An
/// anchor absent from one unit is not absent from the boundary: it is deferred, and refused only if no
/// unit governed it. The FIRST such reason is the one kept, so the refusal names a unit rather than the
/// last one tried.
pub(crate) fn over_each_unit<F>(
    units: &[CompilationUnit],
    anchor: UnitAnchor<'_>,
    mut per_unit: F,
) -> Result<(), String>
where
    F: FnMut(&Path, &Path, &str) -> Result<(), crate::errors::ResolveError>,
{
    let mut governed_somewhere = false;
    let mut deferred: Option<crate::errors::ResolveError> = None;
    for (root_file, src_dir, unit) in units {
        if let UnitAnchor::Module {
            module,
            crate_package,
        } = anchor
        {
            match crate::anchor::module_exists_in_unit(src_dir, root_file, module, crate_package) {
                Ok(true) => {}
                Ok(false) => {
                    if deferred.is_none() {
                        deferred = Some(crate::errors::ResolveError::UnresolvableModule(
                            module.to_string(),
                            crate_package.to_string(),
                        ));
                    }
                    continue;
                }
                Err(reason) => return Err(reason.to_string()),
            }
        }

        match per_unit(root_file, src_dir.as_path(), unit.as_str()) {
            Ok(()) => governed_somewhere = true,
            Err(reason) if is_anchor_absent_from_unit(&reason, anchor) => {
                if deferred.is_none() {
                    deferred = Some(reason);
                }
            }
            Err(reason) => return Err(reason.to_string()),
        }
    }
    match deferred {
        Some(reason) if !governed_somewhere => Err(reason.to_string()),
        _ => Ok(()),
    }
}
