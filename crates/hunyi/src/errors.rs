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

/// A trait-impl-locality boundary whose declared anchor reaches more than one distinct local trait
/// definition through the crate's own `pub use` closure — two mutually-exclusive `#[cfg]` branches
/// re-exporting different traits under one facade name.
///
/// The anchor becomes the violation's `target` and its rule key, so it must denote exactly one trait:
/// picking one of two would make identity arbitrary, and the declaration itself is what is ambiguous.
/// The adopter can say which they mean by naming the defining path instead of the facade.
pub(crate) fn ambiguous_trait_anchor_error(
    trait_path: &str,
    crate_package: &str,
    anchors: &[String],
) -> String {
    format!(
        "a trait-impl-locality boundary must anchor to exactly one trait, but '{trait_path}' in \
         crate '{crate_package}' reaches {} distinct trait definitions through this crate's own \
         re-exports ({}) — two mutually-exclusive `#[cfg]` branches re-export different traits \
         under that name, so the anchor cannot identify one. Declare the defining path instead of \
         the facade",
        anchors.len(),
        anchors.join(", ")
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

/// A forbidden/allowed operand's `::`-delimited spelling has an empty segment — a leading
/// `::`, a trailing `::`, a doubled `::`, or the empty string. No canonical path this crate
/// ever resolves carries one (`extern_verbatim_renamed` builds it purely from `syn::Path`
/// segments, never consulting `leading_colon`, and rustc's own grammar forbids one in real
/// source either), so an operand shaped this way could never equal or prefix-contain a real
/// resolved path — the identical shape of problem `unsafe_empty_allowed_error` and
/// `unsafe_crate_root_allowed_error` guard against for `unsafe`-confinement's own allowed set.
pub(crate) fn malformed_path_operand_error(operand: &str) -> String {
    format!(
        "a forbidden/allowed operand must be a `::`-delimited path with no empty segment: \
         '{operand}' has a leading, trailing, or doubled `::` (or is empty) — no resolved path \
         this system ever produces carries one, so the boundary could never react to it; write it \
         as a bare path instead (e.g. `serde`, not `::serde` or `serde::`)"
    )
}

pub(crate) fn missing_module_file_error(module: &str, crate_package: &str) -> String {
    format!(
        "module '{module}' of crate '{crate_package}' is declared (`mod …;`) but its source file \
         could not be located (expected `<name>.rs` or `<name>/mod.rs`)"
    )
}

// A plain `mod name;` backed by BOTH conventional forms at once. Deliberately NOT claimed as a twin
// of 圭表's or 漏刻's own message for this shape: those two already differ from each other (a
// quoted full module path vs a backticked bare name, and a trailing rule clause present in one and
// absent in the other), so a parity claim here would pick a side while sounding like agreement. The
// three dimensions agree on the *reaction* — exit 2, pinned by
// `crates/tianheng/tests/dual_backed_module_conformance.rs` — never on the text.
// `module` is the module being resolved (an anchor, which may be DEEPER than the ambiguous
// declaration) and `declaration` is the ambiguous `mod` name itself — the two differ whenever an
// ancestor is the dual-backed one, so naming only `module` would attribute the two paths below to a
// module they do not belong to.
pub(crate) fn dual_backed_module_error(
    module: &str,
    declaration: &str,
    crate_package: &str,
    flat: &Path,
    nested: &Path,
) -> String {
    format!(
        "cannot resolve module '{module}' of crate '{crate_package}': its `mod {declaration};` \
         declaration resolves to both '{}' and '{}' — a plain `mod` must be backed by exactly one \
         file, so which of the two governs cannot be judged",
        flat.display(),
        nested.display()
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

/// The semantic dimension's own copy of the static dimension's rule (三儀 ⊥ 三儀: the same rule, not the
/// same function): a crate root outside the package's own manifest directory has no
/// checkout-independent identity label, so it is "cannot judge" rather than a checkout-dependent one.
pub(crate) fn out_of_package_root_error(crate_package: &str, root: &std::path::Path) -> String {
    format!(
        "a violation's identity is labeled by the compilation unit it came from, relative to the \
         package's own directory, so a crate root outside that directory cannot be judged without a \
         checkout-dependent identity: crate '{crate_package}' declares a target rooted at '{}', which \
         is not under the package's manifest directory; move the target's source under the package \
         directory, or declare the boundary against the package that owns it",
        root.display()
    )
}
