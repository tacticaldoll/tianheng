use std::path::Path;

/// The target workspace could not be read — a missing or malformed manifest, or a
/// `cargo metadata` failure. `err` is the underlying cause.
///
/// Deliberate **verbatim** twin of hunyi's `unreadable_workspace_error` (the price of the dimension
/// split; sharing a module would need a forbidden guibiao↔hunyi edge). The two copies MUST stay
/// byte-identical — an unreadable workspace reads the same in either dimension.
pub(crate) fn unreadable_workspace_error(manifest_path: &Path, err: &str) -> String {
    format!(
        "a boundary is observed against a real workspace, so an unreadable one cannot be judged \
         and its verdict would be a false pass: cannot read target workspace at {} ({err}); check \
         the manifest path and that `cargo metadata` succeeds",
        manifest_path.display()
    )
}

/// A boundary names a crate that is not a member of the target workspace.
///
/// Deliberate **verbatim** twin of hunyi's `crate_not_found_error` (dimension split; a shared module
/// would need a forbidden guibiao↔hunyi edge). The two copies MUST stay byte-identical.
pub(crate) fn crate_not_found_error(crate_package: &str) -> String {
    format!(
        "a boundary must govern a real crate or it silently never reacts: target crate \
         '{crate_package}' is not a member of the target workspace — check the name or --manifest-path"
    )
}

/// A workspace member's `src` directory could not be located from its manifest.
///
/// Deliberate **parallel** twin of hunyi's `missing_src_error`: same intent and structure, differing
/// only in the dimension noun ("module" here in 圭表, "semantic" in 渾儀) — not a verbatim twin,
/// because each dimension names its own boundary kind.
pub(crate) fn missing_src_error(crate_package: &str) -> String {
    format!(
        "a module boundary is observed from source, so with no src it could never react: cannot \
         locate the crate root source for '{crate_package}'"
    )
}

/// A module boundary targets an inline `mod name { … }`, which owns no source file
/// and so cannot be a governed target — distinct from an unknown-module typo.
pub(crate) fn inline_module_target_error(module: &str, crate_package: &str, leaf: &str) -> String {
    format!(
        "module '{module}' in crate '{crate_package}' is declared inline (`mod {leaf} {{ … }}`) and \
         owns no source file; module boundaries govern file-based modules — move it \
         into its own file (e.g. `src/{leaf}.rs`), or target an enclosing file-based \
         module"
    )
}

/// A module boundary targets a path that is not a reachable module of the crate
/// (e.g. a typo), distinct from an inline target.
///
/// Deliberate **parallel** twin of hunyi's `unknown_module_error`: both carry the same principle
/// preamble and `— check the path` tail, differing only in the dimension-accurate detail (圭表's
/// module graph is **file-based reachability**; 渾儀 descends declared `mod`s incl. inline).
pub(crate) fn unknown_module_error(module: &str, crate_package: &str) -> String {
    format!(
        "a boundary must anchor to a real module or it silently never reacts: module '{module}' is \
         not found among the reachable modules of crate '{crate_package}' (declared via `mod`, \
         file-based) — check the path"
    )
}

/// A `restrict_imports_to` boundary targets the crate root `crate`, which has no
/// outward internal edge.
pub(crate) fn restrict_imports_to_on_crate_error(crate_package: &str) -> String {
    format!(
        "the `restrict_imports_to` rule cannot govern `crate` (the crate root) of crate \
         '{crate_package}': the root has no outward internal edge — every internal import is \
         within its own subtree, so the rule could never react; declare it on a submodule \
         (e.g. `crate::kernel`) instead"
    )
}

/// A `must_not_be_imported_by` boundary protects the crate root `crate`, against which
/// every internal import (`crate::…`) is "the protected module or beneath".
pub(crate) fn must_not_be_imported_by_on_crate_error(crate_package: &str) -> String {
    format!(
        "the `must_not_be_imported_by` rule cannot protect `crate` (the crate root) of crate \
         '{crate_package}': every internal import is within the crate root, so the rule could \
         never react as an inbound rule; declare it on a submodule (e.g. `crate::internal`) \
         instead"
    )
}

/// A `must_only_be_imported_by` boundary protecting the crate root `crate` degenerates the same
/// way as `must_not_be_imported_by`.
pub(crate) fn must_only_be_imported_by_on_crate_error(crate_package: &str) -> String {
    format!(
        "the `must_only_be_imported_by` rule cannot protect `crate` (the crate root) of crate \
         '{crate_package}': every module is within the crate root, so none is an inbound importer \
         and the allowlist could never react; declare it on a submodule (e.g. `crate::internal`) \
         instead"
    )
}

/// A `confine_external_crate` boundary confines a crate to the crate root `crate`, whose
/// subtree is the whole crate — the confined crate would be permitted everywhere and the
/// rule could never react.
pub(crate) fn confine_external_crate_on_crate_error(crate_package: &str) -> String {
    format!(
        "the `confine_external_crate` rule cannot confine a crate to `crate` (the crate root) of \
         crate '{crate_package}': the root's subtree is the whole crate, so the confined crate \
         would be permitted everywhere and the rule could never react; confine it to a submodule \
         (e.g. `crate::ffi`) instead"
    )
}

/// A `must_not_call_inline` boundary declares an empty confined prefix, which would match
/// everything or nothing — a misdeclaration, never a silent no-op.
pub(crate) fn inline_empty_prefix_error(crate_package: &str) -> String {
    format!(
        "the `must_not_call_inline` rule needs a non-empty module-path prefix (e.g. `std::time`) \
         to confine, in crate '{crate_package}': an empty prefix cannot name a surface, so the \
         rule could never react meaningfully"
    )
}

/// A `must_not_call_inline` boundary declares `.ending_with([])` with an empty verb set, which
/// would narrow the reaction to nothing — a silent no-op, resolved loudly (exit 2).
pub(crate) fn inline_empty_verbs_error(crate_package: &str) -> String {
    format!(
        "a `must_not_call_inline` boundary in crate '{crate_package}' declares `.ending_with([])` \
         with an empty verb set, which would react on nothing (a silent no-op); pass at least one \
         read verb (e.g. `[\"now\"]`) or drop the narrowing"
    )
}

/// A `must_not_call_inline` boundary declares both `.ending_with(…)` (narrow to read verbs) and
/// `.strict_prefix_only()` (widen to all mentions) — a contradiction, resolved loudly (exit 2)
/// rather than by a silent precedence choice.
pub(crate) fn inline_narrow_and_strict_error(crate_package: &str) -> String {
    format!(
        "a `must_not_call_inline` boundary in crate '{crate_package}' declares both `.ending_with(…)` \
         and `.strict_prefix_only()`, which contradict (narrow to read verbs vs. widen to all \
         mentions); choose one"
    )
}

/// A governed source file could not be read. Failing loud rather than skipping it,
/// which could hide a real violation.
pub(crate) fn unreadable_governed_file_error(file: &Path, err: &str) -> String {
    format!(
        "a governed file that cannot be read is 'cannot judge', not 'nothing to judge' — skipping \
         it could hide a real violation: cannot read governed source file '{}' ({err})",
        file.display()
    )
}
