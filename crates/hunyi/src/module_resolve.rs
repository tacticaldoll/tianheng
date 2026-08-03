//! Module resolution: descends a module path from crate root to its items and source files.
//! Handles inline `mod x { ... }`, file `mod x;`, and `#[path]` remaps.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::errors::{
    dual_backed_module_error, missing_module_file_error, unknown_module_error,
    unparseable_source_error, unreadable_source_error,
};
use crate::resolve::strip_raw;
use crate::syn_util::{
    FlatItem, cfg_attr_path_values, direct_path_value, flatten_transparent_macros,
    flatten_with_body_nested_impls, has_cfg_attr,
};
// Test-only: `resolve_module_root` below is the sole (`#[cfg(test)]`) caller.
#[cfg(test)]
use crate::syn_util::flatten_transparent_macro_items;

/// The path segments of a module relative to the crate root.
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

/// Resolves a module path to its items, paired with the source file and branch index of origin.
pub(crate) fn resolve_module_items_with_files(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(syn::Item, PathBuf, usize)>, String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut items = Vec::new();
    for (branch_index, (branch_items, file, ..)) in branches.iter().enumerate() {
        // See `flatten_with_body_nested_impls`: transparent-macro (`cfg_if!`) arms flattened in,
        // plus every `impl` a `const`/`fn` among them directly bodies, attributed to this SAME
        // branch/file (extraction never crosses a file boundary). Tags discarded — nothing here
        // consults arm membership.
        let (flat, nested_impls) = flatten_with_body_nested_impls(branch_items);
        let plain: Vec<syn::Item> = flat.into_iter().map(|f| f.item).collect();
        items.extend(
            plain
                .into_iter()
                .chain(nested_impls)
                .map(|item| (item, file.clone(), branch_index)),
        );
    }
    Ok(items)
}

/// Like [`resolve_module_items_with_files`], but retains each item's [`FlatItem`] tag (its own
/// `cfg_if!` arm membership) instead of discarding it. A `#[cfg]`/`cfg_if!`-split at the MODULE
/// level already gets its own branch index above; this is for the finer split that stays WITHIN
/// one branch's own file — two mutually-exclusive sibling items (a `#[cfg(unix)] mod x;` beside a
/// `#[cfg(not(unix))] pub use x::Y;`, or the two arms of one `cfg_if!` invocation) that share the
/// identical branch index and file, but must not be treated as always coexisting when resolving
/// one against the other (see `exposure.rs`'s cfg-aware re-export child-module shadow).
pub(crate) fn resolve_module_items_with_cfg_tags(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(FlatItem, PathBuf, usize)>, String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut items = Vec::new();
    for (branch_index, (branch_items, file, ..)) in branches.iter().enumerate() {
        // See `flatten_with_body_nested_impls`; unlike `resolve_module_items_with_files`, `flat`'s
        // own arm-membership tags survive into `items` here, while the const/fn-body-nested impls
        // are wrapped as plain `FlatItem`s (no arm membership — nothing that consults it ever
        // matches an `Item::Impl`, so a synthetic tag would claim membership this walk never
        // observed).
        let (flat, nested_impls) = flatten_with_body_nested_impls(branch_items);
        let nested_impls = nested_impls.into_iter().map(FlatItem::plain);
        items.extend(
            flat.into_iter()
                .chain(nested_impls)
                .map(|flat| (flat, file.clone(), branch_index)),
        );
    }
    Ok(items)
}

/// Resolves a module path to its primary source file (test-only helper). Delegates to
/// [`resolve_module_root`] directly (one descent backs every view this module offers, so the
/// file and item views never drift — a `mod`-resolution divergence is the false-negative class
/// the project forbids).
#[cfg(test)]
pub(crate) fn resolve_module_file(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<PathBuf, String> {
    resolve_module_root(src_dir, root_file, module, crate_package).map(|(_items, file, _, _)| file)
}

/// Resolves a module path to its items, file, child directory, and path base (test-only helper).
#[cfg(test)]
pub(crate) fn resolve_module_root(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf, PathBuf, PathBuf), String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut items = Vec::new();
    for (branch_items, ..) in &branches {
        items.extend(flatten_transparent_macro_items(branch_items));
    }
    let (_, file, child_dir, path_base) = &branches[0];
    Ok((items, file.clone(), child_dir.clone(), path_base.clone()))
}

/// The full descent result: every surviving [`Branch`] on its own, each keeping its own items
/// paired with the directories they must be resolved against. A subtree walk that continues
/// descending below the anchor needs this — never the single, unioned-items/first-branch-only
/// shape the test-only `resolve_module_root` returns, which is correct only for a single-module
/// violation's "one file" report and actively wrong for further descent (a non-first branch's own
/// child would resolve against a directory pair that isn't its own).
#[allow(clippy::type_complexity)]
pub(crate) fn resolve_module_branches(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(Vec<syn::Item>, PathBuf, PathBuf, PathBuf)>, String> {
    let root = read_parse(root_file)?;
    let segments = module_segments(module);
    let initial = Branch {
        items: root.items,
        current_file: root_file.to_path_buf(),
        child_dir: src_dir.to_path_buf(),
        // The crate root is mod-rs-like: its own directory (`src_dir`) is the `#[path]` base too.
        path_base: src_dir.to_path_buf(),
    };
    let branches = descend(vec![initial], &segments, module, crate_package)?;
    // Items are returned UNFLATTENED, deliberately. Each caller flattens transparent-macro
    // (`cfg_if!`) arms for itself: the two that only observe items do it at once
    // ([`resolve_module_items_with_files`], `resolve_module_root`), while `scan::collect_subtree`
    // must keep the raw list a moment longer — its own child-module resolution reads arm
    // membership, which flattening erases. Flattening here would have silently cost that walk its
    // absence tolerance (a legitimately fileless arm-declared child reported as exit 2).
    Ok(branches
        .into_iter()
        .map(|b| (b.items, b.current_file, b.child_dir, b.path_base))
        .collect())
}

/// One candidate continuation of the descent: the items visible at this position, the file they
/// live in, and the two directories a further segment resolves from (`child_dir` for a
/// conventional file-form child, `path_base` for a `#[path]` written at this position — see the
/// module-level doc for why these can differ). Ordinarily there is exactly one branch; a
/// mutually-exclusive `#[cfg]` split (an inline variant paired with a file-form sibling) produces
/// two **independent** branches rather than merging their items into one, because each has its
/// own correct directories for anything nested *beneath* the split — merging into one shared pair
/// of directories silently mis-resolved a further segment whenever the file-form sibling's own
/// directories differed from the inline accumulation (the false negative this design fixes).
/// The test-only `resolve_module_root` merges every surviving branch's items back into one list
/// at the leaf; production callers use [`resolve_module_items_with_files`] instead, which keeps
/// each item paired with its own branch's file rather than collapsing to the first.
struct Branch {
    items: Vec<syn::Item>,
    current_file: PathBuf,
    child_dir: PathBuf,
    path_base: PathBuf,
}

// `path_base` is the directory a non-inline `#[path]` at the current position resolves from: the
// containing file's own directory at file scope, but with each enclosing inline `mod` name
// accumulated onto it (rustc adds the inline-module chain as directory components). It equals
// `current_file`'s parent at file scope and diverges from it only after descending an inline block —
// which is exactly the case `current_file.parent()` alone got wrong (a false negative when a
// `#[path]` relocated inside an inline block was resolved from the enclosing file's dir).
fn descend(
    branches: Vec<Branch>,
    segments: &[String],
    module: &str,
    crate_package: &str,
) -> Result<Vec<Branch>, String> {
    let Some(seg) = segments.first() else {
        return Ok(branches);
    };
    let mut next_branches = Vec::new();
    for branch in &branches {
        // Both passes below walk the branch's items with transparent-macro (`cfg_if!`) arms
        // flattened in, so a `mod` declared only inside an arm is descended like any other — 圭表
        // already observes such a declaration, and the two dimensions must not disagree on one
        // shape. Each flattened item carries whether it came from an arm, which the absence
        // tolerance below consults exactly like a bare `#[cfg]` on the declaration itself.
        let flat_items = flatten_transparent_macros(&branch.items);
        // Every same-named **inline** `mod x { … }` for this segment produces its OWN branch, not
        // merged into a shared one: a `#[cfg(..)] mod x {..}` / `#[cfg(..)] mod x {..}` pair parses
        // as two separate inline items (syn does not evaluate `cfg`), and while both are
        // OBSERVED (matching the crate-wide scan's observe-all, cfg-blind policy —
        // `scan::resolve_child_modules`), merging their items into one shared items list also
        // merges everything a downstream caller derives from those items — a `use`-map, a
        // child-module-name shadow set — even though the two arms are never simultaneously open in
        // any real build. That conflation is the identical false-negative class this whole
        // resolver exists to prevent, just one hop past item observation itself (found on a
        // round-8 adversarial review; see `PROJECT.md`'s Decisions): merging genuinely produces
        // every item, but a caller resolving one arm's own bare reference through the OTHER arm's
        // `use`/child-module declaration silently misresolves it. Keeping every inline occurrence
        // as its own independent branch — exactly like the file-form loop below already does —
        // means `resolve_module_items_with_files`' per-branch pairing keeps each arm's items
        // (and, once the caller groups by branch rather than file, each arm's resolution context)
        // distinct even though both arms share the identical enclosing `current_file`. Inline
        // items live in the enclosing file, so `current_file` is unchanged; file-children live
        // under `<child_dir>/x/` by default — UNLESS an unconditional `#[path = "…"]` precedes
        // this inline header, which relocates that base (rustc's rule for an inline module too;
        // it is NOT a no-op merely because the header has a body — verified against a real
        // build), resolved per-occurrence so two inline arms can each carry their own relocation
        // (or lack thereof) without one overwriting the other. A `cfg_attr`-wrapped `path` is not
        // followed (the same cfg-conditional bound as the file-form case below), so it does not
        // relocate.
        for flat in &flat_items {
            if let syn::Item::Mod(module_item) = &flat.item {
                if strip_raw(&module_item.ident.to_string()) != *seg {
                    continue;
                }
                let Some((_, inner)) = &module_item.content else {
                    continue; // a file-form declaration of this name; handled below
                };
                let relocated_base =
                    direct_path_value(&module_item.attrs).map(|rel| branch.path_base.join(rel));
                let inline_dir = relocated_base.unwrap_or_else(|| branch.child_dir.join(seg));
                next_branches.push(Branch {
                    items: inner.clone(),
                    current_file: branch.current_file.clone(),
                    child_dir: inline_dir.clone(),
                    path_base: inline_dir,
                });
            }
        }
        // Resolve EVERY file-form `mod seg;` too — ALWAYS attempted, not only when no inline
        // variant was found above, and never stopping at the first match: a mutually-exclusive
        // `#[cfg]` per-platform shim can legitimately pair an inline variant with a file-form
        // variant, or pair a PLAIN `mod seg;` with an unconditional `#[path]`-remapped `mod seg;`
        // of the same name — two declarations that, once `#[path]` is followed, need not name the
        // same file at all. Matching `resolve_child_modules`'s own crate-wide policy (which never
        // breaks after one match either), every non-inline declaration for this segment produces
        // its own branch; picking only the first was a real false negative (a forbidden item
        // declared only in the sibling that lost the race passed unobserved, nondeterministically
        // depending on source order).
        let mut file_forms: Vec<(Vec<syn::Item>, PathBuf, PathBuf, PathBuf)> = Vec::new();
        // Deduped by the resolved file's CANONICAL path: two mutually-exclusive `#[cfg]` arms
        // that both plainly declare `mod seg;` (no `#[path]`, so both are found via the identical
        // `locate_module_file` lookup) are the same real file compiled twice by neither build —
        // pushing a branch per occurrence would duplicate that file's items in the merged result,
        // inflating one real violation into two apparently-distinct findings with no way for
        // exact-string finding dedup to collapse them back (their internal unsupported-syntax
        // sentinels can differ before the public observation path rejects them).
        let mut seen_files: HashSet<PathBuf> = HashSet::new();
        for flat in &flat_items {
            if let syn::Item::Mod(module_item) = &flat.item {
                if module_item.content.is_some() {
                    continue; // an inline body for this name is already collected above
                }
                if strip_raw(&module_item.ident.to_string()) != *seg {
                    continue;
                }
                // Whether this declaration may legitimately have no source file on this build:
                // its own bare `#[cfg]`, or membership in a transparent macro arm (every
                // `cfg_if!` arm is gated by a predicate in the macro header, the trailing `else`
                // by the negation of the rest). Computed once so the two absence sites below
                // cannot drift apart on it — the divergence class this walker's shared policy
                // with `scan::resolve_child_modules` exists to prevent.
                let cfg_conditional = flat.in_transparent_arm || has_cfg_attr(&module_item.attrs);
                // Follow an **unconditional** `#[path = "…"]` file module. rustc resolves a
                // non-inline `#[path]` relative to `path_base` — the containing file's own
                // directory, with each enclosing inline-`mod` name accumulated onto it — NOT
                // `child_dir` (the conventional-child base `<dir>/seg/` for a non-mod-rs file),
                // the false-negative the whole-crate walk shares. Load `<path_base>/<rel>`, and
                // since a `#[path]`-loaded file is mod-rs-like, its own children (both
                // conventional and any further `#[path]`) resolve from ITS OWN directory too — so
                // `path_base` and the child-continuation directory are the SAME value here (unlike
                // the plain, non-`#[path]` case below, where they differ for a flat `seg.rs`).
                if let Some(rel) = direct_path_value(&module_item.attrs) {
                    let file = branch.path_base.join(&rel);
                    if !file.is_file() {
                        // A BARE `#[cfg(pred)]` co-occurring with this unconditional `#[path]`
                        // (e.g. `#[cfg(windows)] #[path = "windows_impl.rs"] mod imp;`) removes
                        // the whole item, `#[path]` included, when `pred` is false — rustc never
                        // attempts to resolve the target on such a build (verified against a real
                        // build: this compiles cleanly with the target entirely absent). Tolerate
                        // exactly like the plain-missing-file case below; an unconditional item
                        // with no accompanying `#[cfg]` still fails loud.
                        if cfg_conditional {
                            continue;
                        }
                        return Err(missing_module_file_error(module, crate_package));
                    }
                    if !xingbiao::try_visit(&mut seen_files, &file)? {
                        continue;
                    }
                    let parsed = read_parse(&file)?;
                    let next_dir = file
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| branch.child_dir.clone());
                    file_forms.push((parsed.items, file, next_dir.clone(), next_dir));
                    continue;
                }
                // A `cfg_attr`-wrapped `#[path]` target is unioned with the conventional file
                // below, not followed in its place: `cfg_attr` never removes the `mod` item, so
                // cfg-blind observation cannot know which one a given build actually compiles.
                // Mirrors `scan::resolve_child_modules`'s identical union (found on adversarial
                // review: this targeted resolver's own doc once claimed to fail loud on this shape,
                // but a mutually-exclusive sibling declaration for the same name silently absorbed
                // the branch count, so the cfg_attr target's own file was dropped with no error at
                // all whenever ANY sibling resolved — never truly fail-loud in that case).
                let cfg_attr_targets = cfg_attr_path_values(&module_item.attrs);
                let mut has_backing_source = false;
                for rel in &cfg_attr_targets {
                    let file = branch.path_base.join(rel);
                    if file.is_file() {
                        has_backing_source = true;
                        if xingbiao::try_visit(&mut seen_files, &file)? {
                            let parsed = read_parse(&file)?;
                            let next_dir = file
                                .parent()
                                .map(Path::to_path_buf)
                                .unwrap_or_else(|| branch.child_dir.clone());
                            file_forms.push((parsed.items, file, next_dir.clone(), next_dir));
                        }
                    }
                }
                // A `#[cfg]`-gated plain module may legitimately have no source file when the
                // predicate is off (a standard optional-feature pattern) — matching
                // `scan::resolve_child_modules`'s identical tolerance for the crate-wide walk, so
                // this single-module-anchored descent no longer disagrees with its own sibling
                // walker on the identical shape (the 0.2.2 lesson: the two walkers' missing-file
                // policies had silently drifted apart). An unconditional missing file stays a real
                // scan error (exit 2), UNLESS the `cfg_attr` target above already backs this
                // declaration on some other build. BOTH conventional forms present is checked
                // FIRST and is never tolerated: no predicate value makes two files compile as one
                // module, so unlike an absence it cannot be a legitimate configuration — the same
                // ordering 圭表 and 漏刻 each independently apply to this shape.
                let file = match locate_module_file(&branch.child_dir, seg) {
                    ModuleFile::One(file) => file,
                    ModuleFile::Ambiguous { flat, nested } => {
                        // `seg`, not `module`: the ambiguous declaration may be an ANCESTOR of the
                        // anchor being resolved, and the two paths below are that ancestor's.
                        return Err(dual_backed_module_error(
                            module,
                            seg,
                            crate_package,
                            &flat,
                            &nested,
                        ));
                    }
                    ModuleFile::Absent => {
                        if has_backing_source || cfg_conditional {
                            continue;
                        }
                        return Err(missing_module_file_error(module, crate_package));
                    }
                };
                if !xingbiao::try_visit(&mut seen_files, &file)? {
                    continue;
                }
                let parsed = read_parse(&file)?;
                // The loaded file's own directory is the base for a `#[path]` written at its top
                // level (`<dir>` for `seg.rs`, `<dir>/seg` for `seg/mod.rs`); its CONVENTIONAL
                // children (a further plain `mod y;`) always live under `<child_dir>/seg`
                // regardless — the two conventions only diverge for `#[path]`-resolution
                // purposes, never for where a plain child nests.
                let own_dir = file
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| branch.child_dir.join(seg));
                file_forms.push((parsed.items, file, own_dir, branch.child_dir.join(seg)));
            }
        }
        for (file_items, file, path_base, child_dir) in file_forms {
            next_branches.push(Branch {
                items: file_items,
                current_file: file,
                child_dir,
                path_base,
            });
        }
    }
    if next_branches.is_empty() {
        return Err(unknown_module_error(module, crate_package));
    }
    descend(next_branches, &segments[1..], module, crate_package)
}

/// The outcome of resolving a plain `mod name;` to its conventional source file.
///
/// The two conventional forms are mutually exclusive in source rustc accepts, so "both present" is
/// its own variant rather than collapsing into a first-form pick: an item written in the unselected
/// form would otherwise escape observation entirely, and whether the module is governed at all would
/// depend on which file its author happened to write it in (a false negative).
pub(crate) enum ModuleFile {
    /// Neither conventional form exists. The caller decides whether this is a legitimate
    /// `#[cfg]`-gated absence or an unconditional missing file (a hard error).
    Absent,
    /// Exactly one conventional form exists.
    One(PathBuf),
    /// Both `name.rs` and `name/mod.rs` exist — rustc E0761 for a live declaration. Unresolvable
    /// under every `#[cfg]` predicate value, so callers react ahead of any absence tolerance.
    Ambiguous { flat: PathBuf, nested: PathBuf },
}

pub(crate) fn locate_module_file(child_dir: &Path, seg: &str) -> ModuleFile {
    let flat = child_dir.join(format!("{seg}.rs"));
    let nested = child_dir.join(seg).join("mod.rs");
    match (flat.is_file(), nested.is_file()) {
        (true, true) => ModuleFile::Ambiguous { flat, nested },
        (true, false) => ModuleFile::One(flat),
        (false, true) => ModuleFile::One(nested),
        (false, false) => ModuleFile::Absent,
    }
}

pub(crate) fn read_parse(file: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(file)
        .map_err(|err| unreadable_source_error(file, &err.to_string()))?;
    syn::parse_file(&text).map_err(|err| unparseable_source_error(file, &err.to_string()))
}
