//! The module-graph walk: resolves reachable compiled modules from the crate root and
//! selects governed source files, excluding undeclared orphans, inline shadows, and
//! remap-shadowed paths. Depends on the shared lexer and path vocabulary.

mod declarations;
mod paths;
mod walk;

use super::path_vocab::path_within;
use paths::module_path_of;
use std::path::{Path, PathBuf};
use xuanji::ScanDepth;

/// Selects file paths belonging to the governed module that are reachable in the module graph.
/// Excludes undeclared orphan files, inline-only shadows, and remap-shadowed paths.
#[allow(clippy::too_many_arguments)]
pub(crate) fn governed_files(
    src_dir: &Path,
    files: &[PathBuf],
    module: &str,
    reachable: &std::collections::BTreeSet<String>,
    inline_only: &std::collections::BTreeSet<String>,
    remapped: &[(PathBuf, String)],
    remap_shadowed: &std::collections::BTreeSet<String>,
    root_relative: Option<&Path>,
    depth: ScanDepth,
) -> Vec<(PathBuf, String)> {
    let matches_depth = |mod_path: &str| -> bool {
        match depth {
            ScanDepth::Shallow => mod_path == module,
            _ => path_within(mod_path, module),
        }
    };
    let structural = files.iter().filter_map(|file| {
        let relative = file.strip_prefix(src_dir).ok()?;
        let module_path = module_path_of(relative, root_relative);
        if inline_only.contains(&module_path) || remap_shadowed.contains(&module_path) {
            return None;
        }
        if !reachable.contains(&module_path) {
            return None;
        }
        if matches_depth(&module_path) {
            Some((file.clone(), module_path))
        } else {
            None
        }
    });
    let remap_entries = remapped.iter().filter_map(|(file, module_path)| {
        if matches_depth(module_path) {
            Some((file.clone(), module_path.clone()))
        } else {
            None
        }
    });
    let mut seen = std::collections::BTreeSet::new();
    structural
        .chain(remap_entries)
        .filter(|entry| seen.insert(entry.clone()))
        .collect()
}

pub(crate) use walk::reachable_modules;

/// Whether `file` is a second path to the module `crate` in the root at `root_relative`: a file whose
/// path-derived module is `crate` while not being that root. That set is a top-level `mod.rs` under any
/// root, and a top-level `lib.rs` or `main.rs` beside a conventional root (a custom root gives those their
/// stem). Each is either another compiled root — resolved on its own — or a file no target compiles, so its
/// path does not make it a source of this root's `crate`. A `mod` or `#[path]` declaration that reaches it still makes it that module's source: the
/// resolver finds declared children on disk, not in the filtered file set. Without a named root
/// (metadata reporting no target) nothing is excluded, because the conventional files are the root.
pub(crate) fn is_another_crate_root(
    file: &Path,
    src_dir: &Path,
    root_relative: Option<&Path>,
) -> bool {
    let Some(root) = root_relative else {
        return false;
    };
    file.strip_prefix(src_dir)
        .is_ok_and(|relative| relative != root && module_path_of(relative, Some(root)) == "crate")
}

#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declarations::declared_modules(source)
}

#[cfg(test)]
mod tests;
