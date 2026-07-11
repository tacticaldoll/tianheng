//! Constitution- and scan-error message builders shared across 渾儀's capabilities — the
//! single home for the exit-2 "cannot judge" wordings (an unresolvable crate/module/trait
//! anchor, an unreadable workspace, an unreadable/unparseable source file), so no capability
//! or sibling module drifts a copy.

use std::path::Path;

// Deliberate **verbatim** twin of guibiao's `unreadable_workspace_error` (the price of the
// dimension split; a shared module would need a forbidden guibiao↔hunyi edge). MUST stay
// byte-identical — an unreadable workspace reads the same in either dimension.
pub(crate) fn unreadable_workspace_error(manifest_path: &Path, err: &str) -> String {
    format!(
        "a boundary is observed against a real workspace, so an unreadable one cannot be judged \
         and its verdict would be a false pass: cannot read target workspace at {} ({err}); check \
         the manifest path and that `cargo metadata` succeeds",
        manifest_path.display()
    )
}

// Deliberate **verbatim** twin of guibiao's `crate_not_found_error` (dimension split; a shared
// module would need a forbidden guibiao↔hunyi edge). MUST stay byte-identical.
pub(crate) fn crate_not_found_error(crate_package: &str) -> String {
    format!(
        "a boundary must govern a real crate or it silently never reacts: target crate \
         '{crate_package}' is not a member of the target workspace — check the name or --manifest-path"
    )
}

// Deliberate **parallel** twin of guibiao's `missing_src_error`: same intent and structure,
// differing only in the dimension noun ("semantic" here in 渾儀, "module" in 圭表).
pub(crate) fn missing_src_error(crate_package: &str) -> String {
    format!(
        "a semantic boundary is observed from source, so with no src it could never react: cannot \
         locate the crate root source for '{crate_package}'"
    )
}

// Deliberate **parallel** twin of guibiao's `unknown_module_error`: both carry the same principle
// preamble and `— check the path` tail, differing only in the dimension-accurate detail (渾儀
// descends declared `mod`s incl. inline; 圭表's graph is file-based reachability).
pub(crate) fn unknown_module_error(module: &str, crate_package: &str) -> String {
    format!(
        "a boundary must anchor to a real module or it silently never reacts: module '{module}' is \
         not found among the modules of crate '{crate_package}' (declared via `mod`) — check the path"
    )
}

pub(crate) fn unknown_trait_error(trait_path: &str, crate_package: &str) -> String {
    format!(
        "a trait-impl-locality boundary must anchor to a real local trait or it silently never \
         reacts: trait '{trait_path}' is not found as a `trait` item (directly or via a local \
         `pub use`) in crate '{crate_package}' — check the path"
    )
}

/// An unsafe-confinement boundary with an empty allowed set — "no `unsafe` anywhere" is
/// `#![forbid(unsafe_code)]`'s stronger, compile-time job, not this confinement rule's.
pub(crate) fn unsafe_empty_allowed_error(crate_package: &str) -> String {
    format!(
        "an unsafe-confinement boundary on crate '{crate_package}' declares an empty `only_under([])`: \
         this rule confines `unsafe` to a subtree, it does not ban it crate-wide — for that use \
         `#![forbid(unsafe_code)]` (compile-time, unbypassable); name at least one allowed subtree"
    )
}

/// An unsafe-confinement boundary whose allowed set names the crate root — `unsafe` would be
/// permitted everywhere, so the rule could never react.
pub(crate) fn unsafe_crate_root_allowed_error(crate_package: &str) -> String {
    format!(
        "an unsafe-confinement boundary on crate '{crate_package}' allows `unsafe` under `crate` \
         (the crate root): the whole crate would be permitted, so the rule could never react — \
         confine it to a submodule (e.g. `crate::ffi`) instead"
    )
}

pub(crate) fn missing_module_file_error(module: &str, crate_package: &str) -> String {
    format!(
        "module '{module}' of crate '{crate_package}' is declared (`mod …;`) but its source file \
         could not be located (expected `<name>.rs` or `<name>/mod.rs`)"
    )
}

pub(crate) fn unreadable_source_error(file: &Path, err: &str) -> String {
    format!("cannot read source file '{}': {err}", file.display())
}

pub(crate) fn unparseable_source_error(file: &Path, err: &str) -> String {
    // A file we cannot parse is "cannot judge", not "nothing to judge": skipping it could
    // hide a real exposure. Fail loud as a scan error (exit 2), never a silent pass.
    format!("cannot parse source file '{}': {err}", file.display())
}
