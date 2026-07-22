//! Module resolution — descend a `crate::a::b` path from the crate root to the items it owns
//! **and** the source file they live in, in one traversal so the two views cannot drift (a
//! `mod`-resolution divergence is the false-negative class the project forbids). Handles inline
//! `mod x { … }` and file `mod x;` (`<name>.rs` / `<name>/mod.rs`); an **unconditional**
//! `#[path = "…"]` file module is followed to its author-chosen file (matching `walk_module`'s
//! crate-wide policy) — resolved from `path_base`, the containing file's own directory with each
//! enclosing inline-`mod` name accumulated onto it (rustc's rule), so a `#[path]` relocated inside
//! an inline block reads the file rustc compiles, not a same-named orphan. An unconditional
//! `#[path]` preceding an INLINE module header is followed too — not for the header's own content
//! (already inline, unaffected), but to relocate the base its own file-form children resolve
//! from, matching `walk_module`'s crate-wide policy for the identical shape. A `cfg_attr`-wrapped
//! `#[path]` on a FILE module remains a narrow **fail-loud** bound (`unknown_module_error`, exit
//! 2) when it is the only declaration for a segment, never a silent pass and never governing a
//! stale conventional file.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::errors::{
    missing_module_file_error, unknown_module_error, unparseable_source_error,
    unreadable_source_error,
};
use crate::resolve::strip_raw;
use crate::syn_util::{direct_path_value, has_path_attr};

/// The path segments of a module relative to the crate root: `crate::domain::sub` →
/// `["domain", "sub"]`; `crate` → `[]`. A leading `crate` is stripped; canonicalized so a
/// raw-identifier segment (`r#type`) compares as its plain form.
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

/// Resolve a module path to its items, each paired with the **file its own branch was resolved
/// from** and a **branch index** — needed by a caller that must attribute EACH finding to the real
/// file that produced it, rather than a single, first-branch file for the whole module, AND must
/// derive any per-branch resolution context (a `use`-map, a child-module shadow set) from exactly
/// one branch's own items, never a union. When `module` is ordinary (reached through exactly one
/// branch), every item pairs with that one file and index `0`, same as the test-only
/// `resolve_module_file` would report. When `module` was reached through a mutually-exclusive
/// `#[cfg]` split, an item pairs with the file its OWN branch actually lives in and that branch's
/// own index — so a finding produced from a non-first branch's item is never misattributed to a
/// different branch's file (the false-attribution class the test-only `resolve_module_root`'s
/// single-file shape cannot avoid, found on a round-5 adversarial review — see `PROJECT.md`'s
/// Decisions). The index is REQUIRED, not merely the file, because two mutually-exclusive **inline**
/// `#[cfg]` siblings share one identical enclosing file — grouping a per-branch resolution context
/// by file alone re-merges them, reproducing the identical cross-branch conflation one hop past
/// item observation itself (found on a round-8 adversarial review — see `PROJECT.md`'s Decisions).
/// An unknown segment is a constitution error; a declared-but-fileless module is a scan error —
/// never a silent pass.
pub(crate) fn resolve_module_items_with_files(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<(syn::Item, PathBuf, usize)>, String> {
    let branches = resolve_module_branches(src_dir, root_file, module, crate_package)?;
    let mut items = Vec::new();
    for (branch_index, (branch_items, file, ..)) in branches.iter().enumerate() {
        items.extend(
            branch_items
                .iter()
                .cloned()
                .map(|item| (item, file.clone(), branch_index)),
        );
    }
    Ok(items)
}

/// Resolve a module path to the **source file** its items live in — the crate root for `crate`
/// or an inline module, or the located `<name>.rs` / `<name>/mod.rs` for a file module. Test-only
/// (production callers all need per-finding attribution and use
/// [`resolve_module_items_with_files`] instead — this single-file shape is exactly the false-
/// attribution hazard its own doc warns about for a `#[cfg]`-split module, see `PROJECT.md`'s
/// Decisions): a convenience for a resolver-level test that only cares which file one particular
/// module path resolves to.
#[cfg(test)]
pub(crate) fn resolve_module_file(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<PathBuf, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(_items, file)| file)
}

/// The shared module resolution backing [`resolve_module_file`] (test-only): the items a module
/// owns **and** the file they live in, from one descent, so the two views come from the same
/// traversal and never drift (a `mod`-resolution divergence is the false-negative class the
/// project forbids).
#[cfg(test)]
fn resolve_module(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf), String> {
    resolve_module_root(src_dir, root_file, module, crate_package)
        .map(|(items, file, _child_dir, _path_base)| (items, file))
}

/// Like [`resolve_module`] but also returns the module's **child directory** (where its
/// file-based `mod x;` children live) and its `#[path]`-resolution **base** (where a `#[path]`
/// written directly in it resolves from — the same two concepts [`Branch`] tracks throughout the
/// descent). Needed by a subtree walk that must continue descending below the anchored module;
/// the single descent already computes both, so returning them cannot drift from the resolved
/// items/file. Callers MUST use the returned `path_base`, not re-derive it as `file.parent()`: an
/// inline-module anchor's `path_base` is its accumulated directory, which differs from the
/// *enclosing* file's own directory (the inline body stays in the parent's file) — re-deriving it
/// from `file` alone silently substitutes the wrong base.
///
/// When `module` was reached through a mutually-exclusive `#[cfg]` split (an inline variant
/// paired with a file-form sibling, or several same-named non-inline siblings, the standard
/// per-platform shim), the returned **items** are the union of every surviving branch (see
/// [`descend`]) — but the returned file/child-dir/path-base are the **first** branch's own, a
/// stated, deterministic choice: a single-module violation carries one `file` field, not one per
/// branch, so there is no way to report "the file" precisely when more than one legitimately
/// backs the module. A caller that must keep descending *beneath* the anchor (a subtree walk)
/// MUST NOT pair these unioned items with this single directory pair — a non-first branch's own
/// child would then resolve against the wrong directory (a real false negative, found on
/// adversarial review); such a caller should use [`resolve_module_branches`] instead, which keeps
/// every branch's own items and directories together. Test-only in production terms: every real
/// caller needing a module's file now goes through [`resolve_module_items_with_files`] instead,
/// for exactly the false-attribution reason above, generalized to any finding, not just a subtree
/// walk's further descent.
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
        items.extend(branch_items.iter().cloned());
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
        for item in &branch.items {
            if let syn::Item::Mod(module_item) = item {
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
        // exact-string finding dedup to collapse them back (their owner labels can differ when an
        // impl-Trait self type falls back to a positional ordinal marker).
        let mut seen_files: HashSet<PathBuf> = HashSet::new();
        for item in &branch.items {
            if let syn::Item::Mod(module_item) = item {
                if module_item.content.is_some() {
                    continue; // an inline body for this name is already collected above
                }
                if strip_raw(&module_item.ident.to_string()) != *seg {
                    continue;
                }
                // Follow an **unconditional** `#[path = "…"]` file module. rustc resolves a
                // non-inline `#[path]` relative to `path_base` — the containing file's own
                // directory, with each enclosing inline-`mod` name accumulated onto it — NOT
                // `child_dir` (the conventional-child base `<dir>/seg/` for a non-mod-rs file),
                // the false-negative the whole-crate walk shares. Load `<path_base>/<rel>`, and
                // since a `#[path]`-loaded file is mod-rs-like, its own children (both
                // conventional and any further `#[path]`) resolve from ITS OWN directory too — so
                // `path_base` and the child-continuation directory are the SAME value here (unlike
                // the plain, non-`#[path]` case below, where they differ for a flat `seg.rs`). An
                // inline `#[path]` (has a body) or a `cfg_attr`-wrapped `#[path]` is not followed
                // by this targeted resolver — a narrow **fail-loud** bound (exit 2 "cannot
                // judge"), never a silent pass; the whole-crate walks follow the unconditional
                // form.
                if let Some(rel) = direct_path_value(&module_item.attrs) {
                    let file = branch.path_base.join(&rel);
                    if !file.is_file() {
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
                if has_path_attr(&module_item.attrs) {
                    continue;
                }
                let file = locate_module_file(&branch.child_dir, seg)
                    .ok_or_else(|| missing_module_file_error(module, crate_package))?;
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

pub(crate) fn locate_module_file(child_dir: &Path, seg: &str) -> Option<PathBuf> {
    let flat = child_dir.join(format!("{seg}.rs"));
    if flat.is_file() {
        return Some(flat);
    }
    let nested = child_dir.join(seg).join("mod.rs");
    if nested.is_file() {
        return Some(nested);
    }
    None
}

pub(crate) fn read_parse(file: &Path) -> Result<syn::File, String> {
    let text = std::fs::read_to_string(file)
        .map_err(|err| unreadable_source_error(file, &err.to_string()))?;
    syn::parse_file(&text).map_err(|err| unparseable_source_error(file, &err.to_string()))
}
