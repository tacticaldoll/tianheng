//! Module resolution: descends a module path from crate root to its items and source files.
//! Handles inline `mod x { ... }`, file `mod x;`, and `#[path]` remaps.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::errors::{
    dual_backed_module_error, missing_module_file_error, unknown_module_error,
    unparseable_source_error, unreadable_source_error,
};
use crate::resolve::strip_raw;
#[cfg(test)]
use crate::syn_util::flatten_transparent_macro_items;
use crate::syn_util::{
    FlatItem, cfg_attr_path_values, direct_path_value, flatten_transparent_macros,
    flatten_with_body_nested_impls, has_cfg_attr,
};

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
///
/// Flattens transparent macro arms and body-nested `impl`s into the item list.
pub(crate) fn resolve_module_items_with_files(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(syn::Item, PathBuf, usize)>, String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut items = Vec::new();
    for (branch_index, (branch_items, file, ..)) in branches.iter().enumerate() {
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
///
/// Items are returned unflattened so callers that read transparent-macro arm membership
/// can preserve absence tolerance for fileless arm declarations.
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
        path_base: src_dir.to_path_buf(),
    };
    let branches = descend(vec![initial], &segments, module, crate_package)?;
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

/// Every same-named **inline** `mod x { … }` for `seg` in `branch` produces its OWN branch, not
/// merged into a shared one: a `#[cfg(..)] mod x {..}` / `#[cfg(..)] mod x {..}` pair parses as
/// two separate inline items (syn does not evaluate `cfg`), and while both are OBSERVED (matching
/// the crate-wide scan's observe-all, cfg-blind policy — `scan::resolve_child_modules`), merging
/// their items into one shared items list also merges everything a downstream caller derives from
/// those items — a `use`-map, a child-module-name shadow set — even though the two arms are never
/// simultaneously open in any real build. That conflation is the identical false-negative class
/// this whole resolver exists to prevent, just one hop past item observation itself: merging
/// genuinely produces every
/// item, but a caller resolving one arm's own bare reference through the OTHER arm's
/// `use`/child-module declaration silently misresolves it. Keeping every inline occurrence as its
/// own independent branch — exactly like [`push_file_form_branches`] already does — means
/// `resolve_module_items_with_files`' per-branch pairing keeps each arm's items (and, once the
/// caller groups by branch rather than file, each arm's resolution context) distinct even though
/// both arms share the identical enclosing `current_file`. Inline items live in the enclosing
/// file, so `current_file` is unchanged; file-children live under `<child_dir>/x/` by default —
/// UNLESS an unconditional `#[path = "…"]` precedes this inline header, which relocates that base
/// (rustc's rule for an inline module too; it is NOT a no-op merely because the header has a body
/// — verified against a real build), resolved per-occurrence so two inline arms can each carry
/// their own relocation (or lack thereof) without one overwriting the other. A `cfg_attr`-wrapped
/// `path` names a CANDIDATE base per platform predicate and every existing one is descended,
/// unioned with the conventional directory (cfg-blind, matching this crate's own file-form
/// resolution and 圭表's/漏刻's rule for the identical shape), so
/// it does not relocate.
///
/// `path_base` is the directory a non-inline `#[path]` at the current position resolves from: the
/// containing file's own directory at file scope, with each enclosing inline `mod` name accumulated.
/// An unconditional `#[path]` relocates the base outright; conditional `cfg_attr` bases are checked
/// for directory existence and unioned with conventional bases.
fn push_inline_mod_branches(
    branch: &Branch,
    flat_items: &[FlatItem],
    seg: &str,
    next_branches: &mut Vec<Branch>,
) -> Result<(), String> {
    for flat in flat_items {
        if let syn::Item::Mod(module_item) = &flat.item {
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            let Some((_, inner)) = &module_item.content else {
                continue;
            };
            let conventional = branch.child_dir.join(seg);
            let bases: Vec<std::path::PathBuf> = match direct_path_value(&module_item.attrs)
                .map(|rel| branch.path_base.join(rel))
            {
                Some(relocated) => vec![relocated],
                None => {
                    let present: Vec<std::path::PathBuf> = cfg_attr_path_values(&module_item.attrs)
                        .into_iter()
                        .map(|rel| branch.path_base.join(rel))
                        .chain(std::iter::once(conventional.clone()))
                        .collect();
                    let mut present: Vec<PathBuf> = {
                        let mut kept = Vec::new();
                        for base in present {
                            if xingbiao::is_directory(&base)? {
                                kept.push(base);
                            }
                        }
                        kept
                    };
                    present.sort();
                    present.dedup();
                    if present.is_empty() {
                        vec![conventional]
                    } else {
                        present
                    }
                }
            };
            for inline_dir in bases {
                next_branches.push(Branch {
                    items: inner.clone(),
                    current_file: branch.current_file.clone(),
                    child_dir: inline_dir.clone(),
                    path_base: inline_dir,
                });
            }
        }
    }
    Ok(())
}

/// Resolve EVERY file-form `mod seg;` declaration for `branch` — ALWAYS attempted, not only when
/// no inline variant was found by [`push_inline_mod_branches`], and never stopping at the first
/// match: a mutually-exclusive `#[cfg]` per-platform shim can legitimately pair an inline variant
/// with a file-form variant, or pair a PLAIN `mod seg;` with an unconditional `#[path]`-remapped
/// `mod seg;` of the same name — two declarations that, once `#[path]` is followed, need not name
/// the same file at all. Matching `resolve_child_modules`'s own crate-wide policy (which never
/// breaks after one match either), every non-inline declaration for this segment produces its own
/// branch; picking only the first was a real false negative (a forbidden item declared only in
/// the sibling that lost the race passed unobserved, nondeterministically depending on source
/// order). Deduped by the resolved file's CANONICAL path: two mutually-exclusive `#[cfg]` arms
/// that both plainly declare `mod seg;` (no `#[path]`, so both are found via the identical
/// `locate_module_file` lookup) are the same real file compiled twice by neither build — pushing a
/// branch per occurrence would duplicate that file's items in the merged result, inflating one
/// real violation into two apparently-distinct findings with no way for exact-string finding dedup
/// to collapse them back (their internal unsupported-syntax sentinels can differ before the public
/// observation path rejects them).
///
/// Follows unconditional `#[path]` targets relative to `path_base`. Conditional `cfg_attr`
/// targets are unioned with conventional files. Missing source files are tolerated when gated
/// by `#[cfg]` or transparent macro arms. Ambiguous modules with both flat and nested files
/// fail loud.
fn push_file_form_branches(
    branch: &Branch,
    flat_items: &[FlatItem],
    seg: &str,
    module: &str,
    crate_package: &str,
    next_branches: &mut Vec<Branch>,
) -> Result<(), String> {
    let mut file_forms: Vec<(Vec<syn::Item>, PathBuf, PathBuf, PathBuf)> = Vec::new();
    let mut seen_files: HashSet<PathBuf> = HashSet::new();
    for flat in flat_items {
        if let syn::Item::Mod(module_item) = &flat.item {
            if module_item.content.is_some() {
                continue;
            }
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            let cfg_conditional = flat.in_transparent_arm || has_cfg_attr(&module_item.attrs);
            if let Some(rel) = direct_path_value(&module_item.attrs) {
                let file = branch.path_base.join(&rel);
                if !xingbiao::is_regular_file(&file)? {
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
            let cfg_attr_targets = cfg_attr_path_values(&module_item.attrs);
            let mut has_backing_source = false;
            for rel in &cfg_attr_targets {
                let file = branch.path_base.join(rel);
                if xingbiao::is_regular_file(&file)? {
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
            let file = match locate_module_file(&branch.child_dir, seg)? {
                ModuleFile::One(file) => file,
                ModuleFile::Ambiguous { flat, nested } => {
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
    Ok(())
}

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
        let flat_items = flatten_transparent_macros(&branch.items);
        push_inline_mod_branches(branch, &flat_items, seg, &mut next_branches)?;
        push_file_form_branches(
            branch,
            &flat_items,
            seg,
            module,
            crate_package,
            &mut next_branches,
        )?;
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

pub(crate) fn locate_module_file(child_dir: &Path, seg: &str) -> Result<ModuleFile, String> {
    let flat = child_dir.join(format!("{seg}.rs"));
    let nested = child_dir.join(seg).join("mod.rs");
    match (
        xingbiao::is_regular_file(&flat)?,
        xingbiao::is_regular_file(&nested)?,
    ) {
        (true, true) => Ok(ModuleFile::Ambiguous { flat, nested }),
        (true, false) => Ok(ModuleFile::One(flat)),
        (false, true) => Ok(ModuleFile::One(nested)),
        (false, false) => Ok(ModuleFile::Absent),
    }
}

pub(crate) fn read_parse(file: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(file)
        .map_err(|err| unreadable_source_error(file, &err.to_string()))?;
    syn::parse_file(&text).map_err(|err| unparseable_source_error(file, &err.to_string()))
}
