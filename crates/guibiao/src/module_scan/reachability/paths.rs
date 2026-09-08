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
pub(super) fn module_path_of(relative: &Path, root_relative: Option<&Path>) -> String {
    if root_relative == Some(relative) {
        return "crate".to_string();
    }
    // A custom crate root (`[lib] path = "src/core.rs"`) is in effect when `root_relative` is known
    // and is NOT the conventional top-level `lib.rs`/`main.rs`. In that case a STRAY top-level
    // `lib.rs`/`main.rs` is not the crate root — the explicit `path` disables cargo's lib/bin
    // autodetection, so rustc never compiles it — and must not also claim the segment-less `crate`
    // module (which would union its declared modules into the real root and make them
    // phantom-reachable). It maps to `crate::lib` / `crate::main` like any other file and, being
    // undeclared from the true root, stays unreached — matching the compiler.
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
            // `mod.rs` names its directory at any depth. `lib.rs`/`main.rs` are segment-less ONLY at
            // the crate root of a conventional layout — they are the cargo *target* roots there, not
            // module names. When a CUSTOM root is in effect (`custom_root`), a top-level `lib.rs`/
            // `main.rs` is NOT the target root (cargo autodetection is off) and must keep its stem so
            // it does not masquerade as the segment-less `crate` alongside the true root. A declared
            // submodule file literally named `lib.rs`/`main.rs` (`mod lib;` inside a subdir →
            // `foo/lib.rs` = `crate::foo::lib`) contributes its stem like any other file; stripping
            // it at depth would mis-map it to its parent and drift from 渾儀's declaration-driven
            // descent (which resolves it correctly).
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
