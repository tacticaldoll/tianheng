//! Physical source paths mapped to canonical logical module identities.

use super::super::path_vocab::canonical_segment;
use std::path::Path;

/// The module path of a source file, mapping the crate ROOT file to `crate` regardless of its
/// filename. Cargo permits a custom target root (`[lib] path = "src/core.rs"`,
/// `[[bin]] path = "src/app.rs"`), which [`file_module_path`] would otherwise map to
/// `crate::core` / `crate::app` — leaving `crate` empty so no submodule is ever reached (a false
/// negative / spurious exit-2). `root_relative` is that root file's path relative to `src_dir`
/// when known; for the conventional `lib.rs`/`main.rs` it coincides with what `file_module_path`
/// already returns, so passing `None` is safe for the common case.
///
/// When a custom root is in effect, any stray top-level `lib.rs`/`main.rs` maps to
/// `crate::lib` / `crate::main` rather than claiming the segment-less `crate`.
pub(super) fn module_path_of(relative: &Path, root_relative: Option<&Path>) -> String {
    if root_relative == Some(relative) {
        return "crate".to_string();
    }
    let custom_root = root_relative.is_some_and(|r| !is_conventional_root(r));
    file_module_path(relative, custom_root)
}

/// Whether `relative` is a conventional top-level cargo target root — `lib.rs` or `main.rs`
/// directly under `src/` (no parent segment). These are the roots [`file_module_path`] already maps
/// to the segment-less `crate`; any other root file is a *custom* root set via an explicit
/// `[lib]`/`[[bin]]` `path`.
fn is_conventional_root(relative: &Path) -> bool {
    relative
        .file_name()
        .is_some_and(|n| matches!(n.to_string_lossy().as_ref(), "lib.rs" | "main.rs"))
        && relative.components().count() == 1
}

/// The module path of a source file from its path relative to `src/`:
/// `lib.rs`/`main.rs`/`mod.rs` contribute no segment; `kernel/foo.rs` ->
/// `crate::kernel::foo`.
///
/// `mod.rs` names its directory at any depth. In conventional layout, top-level `lib.rs`/`main.rs`
/// contribute no segment. When `custom_root` is true or at depth, they contribute their stem.
fn file_module_path(relative: &Path, custom_root: bool) -> String {
    let components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let mut segments = vec![String::from("crate")];
    let last = components.len().saturating_sub(1);
    for (index, component) in components.iter().enumerate() {
        if index == last {
            let stem = component.strip_suffix(".rs").unwrap_or(component);
            let segmentless = stem == "mod"
                || (!custom_root && components.len() == 1 && matches!(stem, "lib" | "main"));
            if !segmentless {
                segments.push(canonical_segment(stem).to_string());
            }
        } else {
            segments.push(canonical_segment(component).to_string());
        }
    }
    segments.join("::")
}
