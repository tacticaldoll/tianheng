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

#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declarations::declared_modules(source)
}

#[cfg(test)]
mod tests;
