//! Module resolution — descend a `crate::a::b` path from the crate root to the items it owns
//! **and** the source file they live in, in one traversal so the two views cannot drift (a
//! `mod`-resolution divergence is the false-negative class the project forbids). Handles inline
//! `mod x { … }` and file `mod x;` (`<name>.rs` / `<name>/mod.rs`); a `#[path]`-remapped module
//! is skipped (matching `walk_module`'s crate-wide skip), falling through to a loud
//! `unknown_module_error` rather than governing a stale conventional file.

use std::path::{Path, PathBuf};

use crate::errors::{
    missing_module_file_error, unknown_module_error, unparseable_source_error,
    unreadable_source_error,
};
use crate::resolve::strip_raw;
use crate::syn_util::has_path_attr;

/// The path segments of a module relative to the crate root: `crate::domain::sub` →
/// `["domain", "sub"]`; `crate` → `[]`. A leading `crate` is stripped; canonicalized so a
/// raw-identifier segment (`r#type`) compares as its plain form.
fn module_segments(module: &str) -> Vec<String> {
    module
        .split("::")
        .map(strip_raw)
        .enumerate()
        .filter(|(i, seg)| !(*i == 0 && seg == "crate"))
        .map(|(_, seg)| seg)
        .filter(|seg| !seg.is_empty())
        .collect()
}

/// Resolve a module path to the items it owns, descending `mod` declarations from the crate
/// root (inline `mod x { … }` and file-based `mod x;` both). An unknown segment is a
/// constitution error; a declared-but-fileless module is a scan error — never a silent pass.
pub(crate) fn resolve_module_items(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<syn::Item>, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(items, _file)| items)
}

/// Resolve a module path to the **source file** its items live in — the crate root for `crate`
/// or an inline module, or the located `<name>.rs` / `<name>/mod.rs` for a file module. This is
/// the file a single-module semantic violation reports (`Violation::with_file`): the file the
/// reaction already descends to in order to observe the module's items, where the offending
/// seam is written (the finding names the canonicalized forbidden type, which may be *defined*
/// elsewhere). It shares [`resolve_module`]'s one traversal with [`resolve_module_items`], so
/// the reported file can never disagree with the items reacted on.
pub(crate) fn resolve_module_file(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<PathBuf, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(_items, file)| file)
}

/// The shared module resolution: the items a module owns **and** the file they live in, from
/// one descent. [`resolve_module_items`] and [`resolve_module_file`] each keep one half, so the
/// two views come from the same traversal and never drift (a `mod`-resolution divergence is the
/// false-negative class the project forbids).
fn resolve_module(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf), String> {
    resolve_module_root(src_dir, root_file, module, crate_package)
        .map(|(items, file, _child_dir)| (items, file))
}

/// Like [`resolve_module`] but also returns the module's **child directory** — where its
/// file-based `mod x;` children live (`src/` for `crate`, `src/foo/` for `crate::foo`). Needed by
/// a subtree walk that must continue descending below the anchored module; the single descent
/// already computes it, so returning it cannot drift from the resolved items/file.
pub(crate) fn resolve_module_root(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf, PathBuf), String> {
    let root = read_parse(root_file)?;
    let segments = module_segments(module);
    descend(
        root.items,
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        &segments,
        module,
        crate_package,
    )
}

fn descend(
    items: Vec<syn::Item>,
    child_dir: PathBuf,
    current_file: PathBuf,
    segments: &[String],
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf, PathBuf), String> {
    let Some(seg) = segments.first() else {
        return Ok((items, current_file, child_dir));
    };
    for item in &items {
        if let syn::Item::Mod(module_item) = item {
            // A `#[path]`-remapped module is located off the conventional path; the
            // single-module resolver does not observe it (matching `walk_module`'s
            // crate-wide skip), so it falls through to a loud `unknown_module_error`
            // (exit 2) rather than governing a same-named stale conventional file — never
            // a silent claim of cleanliness over a file rustc does not compile.
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            match &module_item.content {
                // Inline `mod x { … }`: descend into the lexical items; the current file is
                // unchanged (an inline module's items live in the enclosing file). Its
                // file-children (if any) live under `<child_dir>/x/`.
                Some((_, inner)) => {
                    return descend(
                        inner.clone(),
                        child_dir.join(seg),
                        current_file,
                        &segments[1..],
                        module,
                        crate_package,
                    );
                }
                // File `mod x;`: `<child_dir>/x.rs` or `<child_dir>/x/mod.rs` becomes the current
                // file; x's children live under `<child_dir>/x/`.
                None => {
                    let file = locate_module_file(&child_dir, seg)
                        .ok_or_else(|| missing_module_file_error(module, crate_package))?;
                    let parsed = read_parse(&file)?;
                    return descend(
                        parsed.items,
                        child_dir.join(seg),
                        file,
                        &segments[1..],
                        module,
                        crate_package,
                    );
                }
            }
        }
    }
    Err(unknown_module_error(module, crate_package))
}

pub(crate) fn locate_module_file(child_dir: &Path, seg: &str) -> Option<PathBuf> {
    let flat = child_dir.join(format!("{seg}.rs"));
    if flat.is_file() {
        return Some(flat);
    }
    let nested = child_dir.join(seg).join("mod.rs");
    if nested.is_file() {
        return Some(nested);
    }
    None
}

pub(crate) fn read_parse(file: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(file)
        .map_err(|err| unreadable_source_error(file, &err.to_string()))?;
    syn::parse_file(&text).map_err(|err| unparseable_source_error(file, &err.to_string()))
}
