//! Constitution- and scan-error message builders shared across 渾儀's capabilities — the
//! single home for the exit-2 "cannot judge" wordings (an unresolvable crate/module/trait
//! anchor, an unreadable workspace, an unreadable/unparseable source file), so no capability
//! or sibling module drifts a copy.

use std::path::Path;

pub(crate) fn unreadable_workspace_error(manifest_path: &Path, err: &str) -> String {
    format!(
        "a boundary is observed against a real workspace, so an unreadable one cannot be judged \
         and its verdict would be a false pass: cannot read target workspace at {} ({err}); check \
         the manifest path and that `cargo metadata` succeeds",
        manifest_path.display()
    )
}

pub(crate) fn crate_not_found_error(crate_package: &str) -> String {
    format!(
        "a boundary must govern a real crate or it silently never reacts: target crate \
         '{crate_package}' is not a member of the target workspace — check the name or --manifest-path"
    )
}

pub(crate) fn missing_src_error(crate_package: &str) -> String {
    format!(
        "a semantic boundary is observed from source, so with no src it could never react: cannot \
         locate the crate root source for '{crate_package}'"
    )
}

/// A package whose every target is an example, a test, a bench or a build script: no compiled root
/// reads its `src/`, so a semantic boundary there could never react.
///
/// Deliberate **parallel** twin of guibiao's `no_compiled_root_error`: same intent and structure,
/// differing only in the dimension noun ("semantic" here in 渾儀, "module" in 圭表) — not a
/// verbatim twin, because each dimension names its own boundary kind.
pub(crate) fn no_compiled_root_error(crate_package: &str) -> String {
    format!(
        "a semantic boundary is observed from a compiled crate root, and '{crate_package}' has none: no target \
         Cargo reports for it is a library or a binary, so nothing its src directory holds is compiled into a \
         root this boundary could govern"
    )
}

pub(crate) fn unknown_module_error(module: &str, crate_package: &str) -> String {
    format!(
        "a boundary must anchor to a real module or it silently never reacts: module '{module}' is \
         not found among the modules of crate '{crate_package}' (declared via `mod`) — check the path"
    )
}

/// A module anchor or allowed location written in a spelling other than the canonical one.
///
/// `suggestion` is the canonical spelling the written one most plausibly meant, when there is one.
pub(crate) fn non_canonical_module_anchor_error(
    written: &str,
    crate_package: &str,
    suggestion: Option<&str>,
) -> String {
    let repair = match suggestion {
        Some("crate") => "write `crate` for the crate root".to_string(),
        Some(spelling) => format!("write `{spelling}`"),
        None => "write the module's path from the crate root, starting `crate::`".to_string(),
    };
    format!(
        "a module is named by one spelling or it becomes two identities: '{written}' in crate \
         '{crate_package}' is not `crate` or `crate::` followed by `::`-separated identifiers — \
         {repair}"
    )
}

/// An allowed location naming a module that no compilation unit of the crate declares.
pub(crate) fn unknown_location_error(location: &str, crate_package: &str) -> String {
    format!(
        "an allowed location must name a real module or it can never match: location \
         '{location}' is not found among the modules of crate '{crate_package}' (declared via \
         `mod`) — check the path"
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

/// A dyn/impl-trait operand whose leaf is an auto trait can never match: those observers
/// remove auto-trait bounds before principal-trait resolution.
pub(crate) fn auto_trait_operand_error(operand: &str, boundary_kind: &str) -> String {
    let builder = match boundary_kind {
        "impl-trait" => "must_not_expose_impl_trait_bounded_by",
        "dyn" | "dyn-trait" => "must_not_expose_dyn_bounded_by",
        _ => "must_not_expose_impl_trait_bounded_by",
    };
    format!(
        "{boundary_kind} forbidden operand '{operand}' can never react: this operand set matches \
         principal traits, and auto-trait bounds are removed before resolution; to govern \
         auto-trait bounds, use {builder}(...) instead, or remove this entry"
    )
}

pub(crate) fn empty_auto_bound_error(boundary_kind: &str) -> String {
    let shape_builder = match boundary_kind {
        "impl-trait" => "must_not_expose_impl_trait",
        "dyn" | "dyn-trait" => "must_not_expose_dyn",
        _ => "must_not_expose_impl_trait",
    };
    format!(
        "{boundary_kind} forbidden auto-trait bound set cannot be empty: to forbid all {boundary_kind} \
         exposures, use {shape_builder}() instead"
    )
}

pub(crate) fn unrecognized_auto_trait_error(operand: &str, boundary_kind: &str) -> String {
    let of_builder = match boundary_kind {
        "impl-trait" => "must_not_expose_impl_trait_of",
        "dyn" | "dyn-trait" => "must_not_expose_dyn_of",
        _ => "must_not_expose_impl_trait_of",
    };
    format!(
        "{boundary_kind} forbidden auto-trait bound '{operand}' is not a recognized std auto trait: \
         auto-trait bounds accept only Send, Sync, Unpin, UnwindSafe, RefUnwindSafe (bare or qualified); \
         for principal trait operands, use {of_builder} instead"
    )
}

pub(crate) fn missing_module_file_error(module: &str, crate_package: &str) -> String {
    format!(
        "module '{module}' of crate '{crate_package}' is declared (`mod …;`) but its source file \
         could not be located (expected `<name>.rs` or `<name>/mod.rs`)"
    )
}

/// A plain `mod name;` backed by both flat (`<name>.rs`) and nested (`<name>/mod.rs`) forms at once.
///
/// `module` is the module being resolved (which may be deeper than the ambiguous declaration) and
/// `declaration` is the ambiguous `mod` name itself.
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

/// A file that cannot be parsed fails loud as a scan error (exit 2) rather than silently skipping it.
pub(crate) fn unparseable_source_error(file: &Path, err: &str) -> String {
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

/// A foreign item in `file` that `crate::syn_util::decode_foreign_item` cannot read as a `fn`,
/// `static`, `type` or macro, with any `safe` or `unsafe` qualifier removed.
///
/// Its visibility and signature are unread, so neither the visibility ceiling nor an exposure rule
/// can be judged against it, and passing it would be a silent pass over a declaration.
pub(crate) fn undecodable_foreign_item_error(file: &Path, seen: &str) -> String {
    format!(
        "cannot judge a foreign item in {}: {seen} inside an `extern` block does not parse as a \
         `fn`, `static`, `type` or macro invocation with any leading `safe` or `unsafe` \
         qualifier removed, so its visibility and signature cannot be read and a boundary over \
         this module would pass it unobserved",
        file.display()
    )
}

/// A crate governed by a static-item boundary renames `thread_local!` (`use std::thread_local as
/// tls;`), so an invocation under the new name escapes the name `thread_local!` is recognized by.
pub(crate) fn thread_local_rename_error(
    renamed_to: &str,
    module: &str,
    crate_package: &str,
) -> String {
    format!(
        "cannot judge static-item boundaries over crate '{crate_package}': module '{module}' renames \
         `thread_local` to `{renamed_to}`, and a `thread_local!` is recognized by its name, so a static \
         declared through `{renamed_to}!` would not be seen — write `thread_local!` (or \
         `std::thread_local!`) directly instead of renaming it"
    )
}

/// A `thread_local!` invocation whose body is not a sequence of `static` declarations.
pub(crate) fn thread_local_body_error(module: &str, file: &Path, why: &str) -> String {
    format!(
        "cannot judge a `thread_local!` in module '{module}' ({}): its body does not read as `static` \
         declarations ({why}), so the statics it declares cannot be named",
        file.display()
    )
}

/// A static whose enclosing owner cannot be named without inventing a positional label.
pub(crate) fn static_owner_unnameable_error(name: &str, module: &str, cause: &str) -> String {
    format!(
        "cannot identify static {name} in {module} — its enclosing {cause}; no positional fallback is \
         invented for it, because a label that names a traversal position is not an identity"
    )
}
