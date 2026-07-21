//! Module resolution — descend a `crate::a::b` path from the crate root to the items it owns
//! **and** the source file they live in, in one traversal so the two views cannot drift (a
//! `mod`-resolution divergence is the false-negative class the project forbids). Handles inline
//! `mod x { … }` and file `mod x;` (`<name>.rs` / `<name>/mod.rs`); an **unconditional**
//! `#[path = "…"]` file module is followed to its author-chosen file (matching `walk_module`'s
//! crate-wide policy) — resolved from `path_base`, the containing file's own directory with each
//! enclosing inline-`mod` name accumulated onto it (rustc's rule), so a `#[path]` relocated inside
//! an inline block reads the file rustc compiles, not a same-named orphan. An inline or
//! `cfg_attr`-wrapped `#[path]` is not followed here — a narrow **fail-loud** bound
//! (`unknown_module_error`, exit 2), never a silent pass and never governing a stale conventional
//! file.

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

/// Resolve a module path to the items it owns, descending `mod` declarations from the crate
/// root (inline `mod x { … }` and file-based `mod x;` both). An unknown segment is a
/// constitution error; a declared-but-fileless module is a scan error — never a silent pass.
pub(crate) fn resolve_module_items(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<Vec<syn::Item>, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(items, _file)| items)
}

/// Resolve a module path to the **source file** its items live in — the crate root for `crate`
/// or an inline module, or the located `<name>.rs` / `<name>/mod.rs` for a file module. This is
/// the file a single-module semantic violation reports (`Violation::with_file`): the file the
/// reaction already descends to in order to observe the module's items, where the offending
/// seam is written (the finding names the canonicalized forbidden type, which may be *defined*
/// elsewhere). It shares [`resolve_module`]'s one traversal with [`resolve_module_items`], so
/// the reported file can never disagree with the items reacted on.
pub(crate) fn resolve_module_file(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<PathBuf, String> {
    resolve_module(src_dir, root_file, module, crate_package).map(|(_items, file)| file)
}

/// The shared module resolution: the items a module owns **and** the file they live in, from
/// one descent. [`resolve_module_items`] and [`resolve_module_file`] each keep one half, so the
/// two views come from the same traversal and never drift (a `mod`-resolution divergence is the
/// false-negative class the project forbids).
fn resolve_module(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf), String> {
    resolve_module_root(src_dir, root_file, module, crate_package)
        .map(|(items, file, _child_dir)| (items, file))
}

/// Like [`resolve_module`] but also returns the module's **child directory** — where its
/// file-based `mod x;` children live (`src/` for `crate`, `src/foo/` for `crate::foo`). Needed by
/// a subtree walk that must continue descending below the anchored module; the single descent
/// already computes it, so returning it cannot drift from the resolved items/file.
pub(crate) fn resolve_module_root(
    src_dir: &Path,
    root_file: &Path,
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf, PathBuf), String> {
    let root = read_parse(root_file)?;
    let segments = module_segments(module);
    descend(
        root.items,
        src_dir.to_path_buf(),
        root_file.to_path_buf(),
        // The crate root is mod-rs-like: its own directory (`src_dir`) is the `#[path]` base too.
        src_dir.to_path_buf(),
        &segments,
        module,
        crate_package,
    )
}

// `path_base` is the directory a non-inline `#[path]` at the current position resolves from: the
// containing file's own directory at file scope, but with each enclosing inline `mod` name
// accumulated onto it (rustc adds the inline-module chain as directory components). It equals
// `current_file`'s parent at file scope and diverges from it only after descending an inline block —
// which is exactly the case `current_file.parent()` alone got wrong (a false negative when a
// `#[path]` relocated inside an inline block was resolved from the enclosing file's dir).
fn descend(
    items: Vec<syn::Item>,
    child_dir: PathBuf,
    current_file: PathBuf,
    path_base: PathBuf,
    segments: &[String],
    module: &str,
    crate_package: &str,
) -> Result<(Vec<syn::Item>, PathBuf, PathBuf), String> {
    let Some(seg) = segments.first() else {
        return Ok((items, current_file, child_dir));
    };
    // Union every same-named **inline** `mod x { … }` for this segment before descending: a
    // `#[cfg(..)] mod x {..}` / `#[cfg(..)] mod x {..}` pair parses as two separate inline items
    // (syn does not evaluate `cfg`), so resolving only the source-first variant would let a
    // forbidden item in another variant pass unobserved — a `mod`-resolution divergence, the
    // false-negative class this resolver exists to prevent. This matches the crate-wide scan's
    // observe-all, cfg-blind policy (`scan::resolve_child_modules`). An unconditional `#[path]` file
    // module is followed below; an inline `#[path]` variant is not merged into this union (a narrow
    // fail-loud bound). Inline items live in the enclosing file, so `current_file` is unchanged;
    // file-children live under `<child_dir>/x/`.
    let mut inline: Vec<syn::Item> = Vec::new();
    for item in &items {
        if let syn::Item::Mod(module_item) = item {
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            if let Some((_, inner)) = &module_item.content {
                inline.extend(inner.iter().cloned());
            }
        }
    }
    // Resolve a file-form `mod seg;` too — ALWAYS attempted now, not only when no inline variant
    // was found above: a mutually-exclusive `#[cfg]` per-platform shim can legitimately pair an
    // inline variant on one platform with a file-form variant on another (the same additive,
    // cfg-blind union the whole-crate walk and 圭表 already apply to their own same-named
    // children), and the scanner does not evaluate `#[cfg]`, so observing only whichever variant
    // happened to carry an inline body was a real false negative: a forbidden item declared only
    // in the file-form sibling passed unobserved. (A cfg-duplicated file-form `mod seg;` pair
    // names one file, so the first match still suffices — unioning *multiple* file-form targets
    // is the whole-crate walk's job, not this single-path resolver's.)
    let mut file_form: Option<(Vec<syn::Item>, PathBuf, PathBuf)> = None;
    'find_file_form: for item in &items {
        if let syn::Item::Mod(module_item) = item {
            if module_item.content.is_some() {
                continue; // an inline body for this name is already collected above
            }
            if strip_raw(&module_item.ident.to_string()) != *seg {
                continue;
            }
            // Follow an **unconditional** `#[path = "…"]` file module. rustc resolves a non-inline
            // `#[path]` relative to `path_base` — the containing file's own directory, with each
            // enclosing inline-`mod` name accumulated onto it — NOT `child_dir` (the conventional-child
            // base `<dir>/seg/` for a non-mod-rs file), the false-negative the whole-crate walk shares.
            // Load `<path_base>/<rel>`, and since a `#[path]`-loaded file is mod-rs-like, descend with
            // its own directory as the base (both `child_dir` and `path_base`) for the next segment's
            // children. An inline `#[path]` (has a body) or a `cfg_attr`-wrapped `#[path]` is not
            // followed by this targeted resolver — a narrow **fail-loud** bound (exit 2 "cannot
            // judge"), never a silent pass; the whole-crate walks follow the unconditional form.
            if let Some(rel) = direct_path_value(&module_item.attrs) {
                let file = path_base.join(&rel);
                if !file.is_file() {
                    return Err(missing_module_file_error(module, crate_package));
                }
                let parsed = read_parse(&file)?;
                let next_dir = file
                    .parent()
                    .map(Path::to_path_buf)
                    .unwrap_or_else(|| child_dir.clone());
                file_form = Some((parsed.items, file, next_dir));
                break 'find_file_form;
            }
            if has_path_attr(&module_item.attrs) {
                continue;
            }
            let file = locate_module_file(&child_dir, seg)
                .ok_or_else(|| missing_module_file_error(module, crate_package))?;
            let parsed = read_parse(&file)?;
            // The loaded file's own directory is the base for a `#[path]` written at its top level
            // (`<dir>` for `seg.rs`, `<dir>/seg` for `seg/mod.rs`); its conventional children live
            // under `<child_dir>/seg`.
            let own_dir = file
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| child_dir.join(seg));
            file_form = Some((parsed.items, file, own_dir));
            break 'find_file_form;
        }
    }
    match (inline.is_empty(), file_form) {
        (false, Some((file_items, _file, _own_dir))) => {
            // Both an inline body and a file-form sibling declare `seg` under mutually-exclusive
            // `#[cfg]` arms: union their items, matching rustc (exactly one is ever compiled, and
            // the scanner does not evaluate which). Descent continues from the inline body's own
            // accumulated directory — for an ordinary (non-`#[path]`) file-form sibling this
            // coincides with its own directory too (both are `<child_dir>/seg`), so a *further*
            // segment resolves identically either way. A **stated, narrower bound**: if the
            // file-form sibling ALSO carries its own `#[path]` (relocating it elsewhere), a
            // segment nested beneath `seg` is resolved only from the inline accumulation, not
            // from that relocated directory — an amendment, not silently claimed complete, since
            // fixing it requires this resolver to carry more than one candidate directory forward.
            let mut merged = inline;
            merged.extend(file_items);
            let inline_dir = child_dir.join(seg);
            descend(
                merged,
                inline_dir.clone(),
                current_file,
                inline_dir,
                &segments[1..],
                module,
                crate_package,
            )
        }
        (false, None) => {
            // Descending an inline `mod seg { … }`: the body stays in `current_file`, but its own
            // children — conventional AND any nested `#[path]` — resolve from `<child_dir>/seg`
            // (rustc accumulates the inline-module name as a directory component), so that becomes
            // the new `path_base` as well as the new `child_dir`.
            let inline_dir = child_dir.join(seg);
            descend(
                inline,
                inline_dir.clone(),
                current_file,
                inline_dir,
                &segments[1..],
                module,
                crate_package,
            )
        }
        (true, Some((file_items, file, own_dir))) => descend(
            file_items,
            child_dir.join(seg),
            file,
            own_dir,
            &segments[1..],
            module,
            crate_package,
        ),
        (true, None) => Err(unknown_module_error(module, crate_package)),
    }
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
