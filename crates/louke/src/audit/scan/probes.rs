use super::lexer::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use xingbiao::{is_directory, is_regular_file};


/// What the source scan found for a probe occurrence (`assert_boundary!`).
#[derive(Debug)]
pub(crate) enum Probe {
    /// A probe whose seam is a string literal (auditable, plain or raw): the seam value.
    Literal(String),
    /// A probe whose seam argument is NOT a string literal (a const or expression): the CI
    /// face cannot trace it to a declared seam, so it reacts rather than skipping. Carries the
    /// matched marker, source file, an owner-qualified enclosing item (never a bare name — two
    /// owners may share a method name), and the offending expression's own trimmed source text,
    /// so distinct non-literal probes in one file are distinct findings (never an absolute byte
    /// offset; an anonymous lexical scope may carry a parent-local equal-header discriminator —
    /// see `fn_scopes`/`first_macro_arg_end`).
    Unauditable {
        marker: String,
        file: String,
        owner: String,
        expr: String,
    },
}

pub(crate) const DEFAULT_MARKERS: &[&str] = &["assert_boundary"];

/// One observed file's `file` identity label: its path relative to the caller-supplied `anchor`
/// (the checkout/workspace root — see `audit_probe_coverage_with_markers`, which owns the whole
/// rationale for the anchor being given rather than derived), falling back to the absolute form
/// when the file does not lie under it.
///
/// A raw `display()` form would vary with the checkout location and land directly in
/// `UnauditableProbe`'s identity (see `finding.rs`), so a baseline recorded in one clone would
/// match nothing in another. See `runtime-origin-assertion`'s "An un-auditable probe's identity
/// distinguishes distinct offending expressions" requirement for the checkout-relocation and
/// member-set scenarios, and for the stated residual gap an absolute `#[path]` literal keeps.
///
/// The relative path is rendered by [`xingbiao::path_label`], never through `Path::display()`, and that
/// one shared function is where the whole rule lives: `/` as the only component separator whatever the
/// observing platform uses, and every byte preserved. `display()` is **lossy** — it replaces each
/// undecodable byte with U+FFFD, so two source paths differing only in invalid-UTF-8 bytes would produce
/// one label, one `UnauditableProbe` identity, and a baseline accepting the first would silently
/// suppress the second's violation. An identity component must not lose information the observation had.
///
/// 漏刻 held that rule privately while 圭表 and 渾儀's compilation-unit label did not, which is how the two
/// came to disagree about the same input; it is shared rather than copied so they cannot drift again.
/// 漏刻 is also where the byte half is genuinely reachable — these paths come from filesystem walks, not
/// from Cargo's JSON.
pub(crate) fn labeled(path: &Path, anchor: &Path) -> String {
    xingbiao::path_label(path.strip_prefix(anchor).unwrap_or(path))
}

pub(crate) fn collect_probes_with_markers(
    input: &Path,
    anchor: &Path,
    markers: &[&str],
    probes: &mut Vec<Probe>,
) -> Result<(), String> {
    if input.is_file() {
        return collect_reachable_probes(input, anchor, markers, probes);
    }
    let mut visited = HashSet::new();
    collect_directory_probes(input, anchor, markers, probes, &mut visited)
}

/// Read `dir`'s entries — I/O only, no scan dispatch — as `(is_dir, path)` pairs sorted so the
/// downstream traversal order (and thus the violation order in the report) is deterministic
/// across runs (`read_dir` order is OS/filesystem-dependent and unsorted). `file_type()` does NOT
/// follow symlinks, so a symlinked directory is reported as a file here — avoiding an infinite
/// loop on a cyclic symlink (fail safe, not stack-overflow loud).
pub(crate) fn read_dir_entries_sorted(dir: &Path) -> Result<Vec<(bool, PathBuf)>, String> {
    let read = std::fs::read_dir(dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut paths = Vec::new();
    for entry in read {
        let entry =
            entry.map_err(|e| format!("cannot read a dir entry under {}: {e}", dir.display()))?;
        let file_type = entry
            .file_type()
            .map_err(|e| format!("cannot stat {}: {e}", entry.path().display()))?;
        paths.push((file_type.is_dir(), entry.path()));
    }
    paths.sort();
    Ok(paths)
}

/// Read `file`'s source and scan it for probes, returning the read source text — the one I/O
/// touch (a full read, unlike the directory-listing metadata [`read_dir_entries_sorted`] reads)
/// shared by [`collect_directory_probes`] and [`collect_reachable_probes`], each of which decides
/// whether to reach this leaf action from cheap metadata alone (an extension check, an `is_dir`
/// flag) before ever calling it. [`collect_reachable_probes`] also needs the source text
/// afterward (to walk this file's own further module references), so it is returned rather than
/// discarded.
///
/// When `absolute_reached` is true, the path label is preserved as written without relativization
/// against `anchor` (see [`labeled`]).
pub(crate) fn scan_rust_file(
    file: &Path,
    anchor: &Path,
    markers: &[&str],
    probes: &mut Vec<Probe>,
    absolute_reached: bool,
) -> Result<String, String> {
    let source = std::fs::read_to_string(file)
        .map_err(|e| format!("cannot read source {}: {e}", file.display()))?;
    let label = if absolute_reached {
        xingbiao::path_label(file)
    } else {
        labeled(file, anchor)
    };
    scan_source_with_markers(&source, &label, markers, probes);
    Ok(source)
}

/// Walks directory tree, passing `absolute_reached = false` since directory traversal does not
/// resolve `#[path]` literals.
pub(crate) fn collect_directory_probes(
    dir: &Path,
    anchor: &Path,
    markers: &[&str],
    probes: &mut Vec<Probe>,
    visited: &mut HashSet<PathBuf>,
) -> Result<(), String> {
    if !xingbiao::try_visit(visited, dir)? {
        return Ok(());
    }
    for (is_dir, path) in read_dir_entries_sorted(dir)? {
        if is_dir {
            collect_directory_probes(&path, anchor, markers, probes, visited)?;
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs")
            && xingbiao::try_visit(visited, &path)?
        {
            scan_rust_file(&path, anchor, markers, probes, false)?;
        }
    }
    Ok(())
}

/// Collects probes reachable from `root` by traversing module declarations.
///
/// Tracks canonicalized visits to prevent cyclic loops on symlinks. Pending items carry
/// `(file, child_base, absolute_reached)`: an absolute `#[path]` literal is inherited by
/// child modules because they resolve relative to the absolute-reached file's directory.
/// A non-inline `#[path]` resolves relative to the containing file's own directory rather
/// than `child_base`.
pub(crate) fn collect_reachable_probes(
    root: &Path,
    anchor: &Path,
    markers: &[&str],
    probes: &mut Vec<Probe>,
) -> Result<(), String> {
    let root_parent = root
        .parent()
        .ok_or_else(|| format!("source root has no parent: {}", root.display()))?;
    let mut pending = vec![(root.to_path_buf(), root_parent.to_path_buf(), false)];
    let mut visited: HashSet<PathBuf> = HashSet::new();
    while let Some((file, child_base, absolute_reached)) = pending.pop() {
        if !xingbiao::try_visit(&mut visited, &file)? {
            continue;
        }
        let source = scan_rust_file(&file, anchor, markers, probes, absolute_reached)?;
        let file_dir = file.parent().unwrap_or(child_base.as_path());
        let mut children = external_module_files(&source, &child_base, file_dir)?;
        children.sort();
        children.reverse();
        pending.extend(
            children
                .into_iter()
                .map(|(f, b, child_absolute)| (f, b, absolute_reached || child_absolute)),
        );
    }
    Ok(())
}

/// Finds external module declarations within `source`.
///
/// Returns tuples of `(target_path, next_child_base, absolute_reached)` where `absolute_reached`
/// records whether the target module path was formed from an absolute `#[path]` within this file.
pub(crate) fn external_module_files(
    source: &str,
    child_base: &Path,
    file_dir: &Path,
) -> Result<Vec<(PathBuf, PathBuf, bool)>, String> {
    let mut modules = Vec::new();
    collect_scope_modules(
        source.as_bytes(),
        0,
        source.len(),
        child_base,
        file_dir,
        &mut modules,
        false,
        false,
        0,
    )?;
    Ok(modules)
}

/// Resolve an external `mod name;` declaration (found at `mod_index` in `bytes`, within the scope
/// starting at `scope_start`) into `modules`, or fail loud when genuinely unresolvable on every
/// configuration. An unconditional `#[path]` is authoritative on every build — the sole source; a
/// non-inline `#[path]` resolves from the containing file's OWN directory (`file_dir`), not the
/// conventional-child base — rustc's mod-rs-blind rule. Absent that, every `cfg_attr`-wrapped
/// `#[path]` candidate that exists on disk (resolved the identical way) AND the conventional file
/// are unioned — cfg-blind observation cannot know which one a given build actually uses, so
/// neither is silently preferred over the other. No file at any candidate location is tolerated
/// when the declaration is `#[cfg]`-gated or arm-conditional (may legitimately compile no probes
/// here); otherwise it is a real broken reference (exit 2). Conventional child modules inherit
/// `absolute_base` from their enclosing scope because their declaration carries no `#[path]`
/// literal of its own. Pulled out of [`collect_scope_modules`]'s `mod name;` arm.
#[allow(clippy::too_many_arguments)]
pub(crate) fn resolve_external_mod_decl(
    bytes: &[u8],
    scope_start: usize,
    mod_index: usize,
    name: &str,
    child_base: &Path,
    file_dir: &Path,
    modules: &mut Vec<(PathBuf, PathBuf, bool)>,
    in_transparent_arm: bool,
    absolute_base: bool,
) -> Result<(), String> {
    let attrs = mod_preamble_attrs(bytes, scope_start, mod_index);
    if let Some(rel) = &attrs.path {
        match resolve_path_module(file_dir, rel)? {
            Some((file, base, reached)) => modules.push((file, base, reached || absolute_base)),
            None if absence_is_tolerated(&attrs, in_transparent_arm) => {}
            None => {
                return Err(format!(
                    "cannot resolve reachable module `{name}` under {}",
                    child_base.display()
                ));
            }
        }
        return Ok(());
    }
    let mut has_backing_source = false;
    for rel in &attrs.cfg_attr_paths {
        if let Some((file, base, reached)) = resolve_path_module(file_dir, rel)? {
            has_backing_source = true;
            modules.push((file, base, reached || absolute_base));
        }
    }
    if let Some((file, next_base)) = resolve_external_module(child_base, name)? {
        has_backing_source = true;
        modules.push((file, next_base, absolute_base));
    }
    if !(has_backing_source || absence_is_tolerated(&attrs, in_transparent_arm)) {
        return Err(format!(
            "cannot resolve reachable module `{name}` under {}",
            child_base.display()
        ));
    }
    Ok(())
}

/// Whether a module declaration may legitimately have no backing source file on this
/// configuration: its own bare `#[cfg]`, or membership in a transparent macro arm — the arm's
/// predicate lives in the macro's `if #[cfg(..)]` header rather than on the item, and every arm
/// is conditionally compiled by construction (the trailing `else` on its predicates' negation).
/// 圭表 settled this rule for the same shape and 渾儀 adopted it; a third hand-assembled
/// derivation here would be the silent-divergence class the cross-dimension ledger exists to
/// catch.
pub(crate) fn absence_is_tolerated(attrs: &ModPreambleAttrs, in_transparent_arm: bool) -> bool {
    attrs.cfg || in_transparent_arm
}

/// The candidate base directories an inline `mod name { … }`'s body should be descended from. An
/// unconditional `#[path]` is the sole authority, exactly as [`resolve_external_mod_decl`] applies
/// for an external `mod`. A `cfg_attr`-wrapped `#[path]` is cfg-conditional on which predicate a
/// given build selects — cfg-blind observation cannot know which, so the body is descended once
/// per candidate base that exists as a directory: every `cfg_attr` target that resolves, AND the
/// conventional base if IT resolves (the predicate could evaluate false on every one, in which
/// case rustc strips the attribute entirely and the plain, unremapped base applies). A candidate
/// base that isn't a real directory contributes nothing — recursing into it would spuriously
/// fail-loud on the module's own OTHER, unrelated nested items merely because this one platform's
/// directory happens not to exist, when another candidate already backs them. If NO candidate
/// resolves at all, fall back to the conventional base anyway (the pre-existing, un-remapped
/// behavior) so a nested reference that is genuinely broken on every platform still fails loud
/// rather than being silently dropped.
///
/// Returns tuples of `(base_dir, is_absolute_base)`. An absolute `#[path]` literal makes the base
/// itself absolute, tracked directly on the base so children resolved from it inherit the non-portable
/// identity. A conventional base carries `false` because whether it was reached absolutely is inherited
/// from the enclosing caller's `absolute_base`.
pub(crate) fn inline_mod_bases(
    attrs: &ModPreambleAttrs,
    name: &str,
    child_base: &Path,
    file_dir: &Path,
) -> Result<Vec<(PathBuf, bool)>, String> {
    let mut inline_bases: Vec<(PathBuf, bool)> = Vec::new();
    match &attrs.path {
        Some(rel) => inline_bases.push((file_dir.join(rel), Path::new(rel).is_absolute())),
        None => {
            for rel in &attrs.cfg_attr_paths {
                let candidate = file_dir.join(rel);
                if is_directory(&candidate)? {
                    inline_bases.push((candidate, Path::new(rel).is_absolute()));
                }
            }
            let conventional = child_base.join(name);
            if inline_bases.is_empty() || is_directory(&conventional)? {
                inline_bases.push((conventional, false));
            }
        }
    }
    Ok(inline_bases)
}

/// A recursion-depth cap for [`collect_scope_modules`]'s native-stack descent into nested
/// blocks, transparent-macro arms, and inline `mod` bodies — a DoS backstop set far below the
/// measured crash threshold (safe at depth 1100, a real SIGABRT stack overflow at depth 1105+
/// under a 2MB test-thread stack), so a pathologically nested source file fails loud (a scan
/// error) rather than crashing the process. Past the cap, this is a stated observation bound,
/// never a silent truncation — matching every other depth-bound walker in this workspace
/// (`hunyi::scan::MAX_MODULE_DEPTH`, `guibiao::use_scan::MAX_USE_NEST_DEPTH`,
/// `guibiao::symbol_scan::MAX_SYMBOL_NEST_DEPTH`).
const MAX_SCOPE_NEST_DEPTH: usize = 300;

/// Traverses module declarations in `bytes[start..end]`.
///
/// Descends transparent macro arms without accumulating a directory component, inline `mod`
/// bodies using their resolved `inline_base` as both `child_base` and `file_dir`, and arbitrary
/// brace blocks (which may contain `#[path = "..."] mod name;` items) keeping enclosing bases.
/// Propagates `absolute_base` across nested scopes. Arm membership is not inherited into inline
/// `mod` bodies.
#[allow(clippy::too_many_arguments)]
pub(crate) fn collect_scope_modules(
    bytes: &[u8],
    start: usize,
    end: usize,
    child_base: &Path,
    file_dir: &Path,
    modules: &mut Vec<(PathBuf, PathBuf, bool)>,
    in_transparent_arm: bool,
    absolute_base: bool,
    depth: usize,
) -> Result<(), String> {
    if depth >= MAX_SCOPE_NEST_DEPTH {
        return Err(format!(
            "cannot judge {}: scope nesting exceeds the depth bound ({MAX_SCOPE_NEST_DEPTH}) \
             this scanner supports without risking a native stack overflow",
            file_dir.display()
        ));
    }
    let mut i = start;
    while i < end {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(end);
            continue;
        }
        if bytes[i] == b'!' && preceding_token_is_ident(bytes, i) {
            let mut name_end = i;
            while name_end > 0 && bytes[name_end - 1].is_ascii_whitespace() {
                name_end -= 1;
            }
            if is_transparent_macro_name(bytes, name_end) {
                if let Some(body_end) = foreign_macro_body_end(bytes, i) {
                    for (arm_start, arm_end) in transparent_arm_ranges(bytes, i, body_end) {
                        collect_scope_modules(
                            bytes,
                            arm_start,
                            arm_end.min(end),
                            child_base,
                            file_dir,
                            modules,
                            true,
                            absolute_base,
                            depth + 1,
                        )?;
                    }
                    i = body_end.min(end);
                    continue;
                }
            }
            if let Some(next) = foreign_macro_body_end(bytes, i) {
                i = next.min(end);
                continue;
            }
        }
        if is_mod_keyword(bytes, i) {
            let mut cursor = skip_space_and_comments(bytes, i + 3);
            let name_start = cursor;
            if bytes.get(cursor..cursor + 2) == Some(b"r#") {
                cursor += 2;
            }
            while cursor < end && is_ident_byte(bytes[cursor]) {
                cursor += 1;
            }
            if cursor == name_start
                || (cursor == name_start + 2 && &bytes[name_start..cursor] == b"r#")
            {
                i += 3;
                continue;
            }
            let raw_name = &bytes[name_start..cursor];
            let name = if raw_name.starts_with(b"r#") {
                &raw_name[2..]
            } else {
                raw_name
            };
            let name = std::str::from_utf8(name).map_err(|e| e.to_string())?;
            cursor = skip_space_and_comments(bytes, cursor);
            match bytes.get(cursor) {
                Some(b';') => {
                    resolve_external_mod_decl(
                        bytes,
                        start,
                        i,
                        name,
                        child_base,
                        file_dir,
                        modules,
                        in_transparent_arm,
                        absolute_base,
                    )?;
                    i = cursor + 1;
                    continue;
                }
                Some(b'{') => {
                    let close = balanced_brace_end(bytes, cursor, end);
                    let attrs = mod_preamble_attrs(bytes, start, i);
                    let inline_bases = inline_mod_bases(&attrs, name, child_base, file_dir)?;
                    for (inline_base, base_is_absolute) in &inline_bases {
                        collect_scope_modules(
                            bytes,
                            cursor + 1,
                            close.saturating_sub(1),
                            inline_base,
                            inline_base,
                            modules,
                            false,
                            *base_is_absolute || absolute_base,
                            depth + 1,
                        )?;
                    }
                    i = close;
                    continue;
                }
                _ => {}
            }
        }
        if bytes[i] == b'{' {
            let close = balanced_brace_end(bytes, i, end);
            collect_scope_modules(
                bytes,
                i + 1,
                close.saturating_sub(1),
                child_base,
                file_dir,
                modules,
                in_transparent_arm,
                absolute_base,
                depth + 1,
            )?;
            i = close;
            continue;
        }
        i += 1;
    }
    Ok(())
}

/// Resolve a `mod name;` to its conventional file and the base directory for its own children:
/// `Ok(Some(..))` for `<base>/name.rs` or `<base>/name/mod.rs`, `Ok(None)` when neither exists (the
/// caller decides whether an absent file is a legitimate `#[cfg]`-gated skip or a hard error), and
/// `Err` only for a genuine ambiguity (both files present).
pub(crate) fn resolve_external_module(
    base: &Path,
    name: &str,
) -> Result<Option<(PathBuf, PathBuf)>, String> {
    let flat = base.join(format!("{name}.rs"));
    let nested = base.join(name).join("mod.rs");
    let file = match (is_regular_file(&flat)?, is_regular_file(&nested)?) {
        (true, false) => flat,
        (false, true) => nested,
        (true, true) => {
            return Err(format!(
                "module `{name}` resolves to both '{}' and '{}'",
                flat.display(),
                nested.display()
            ));
        }
        (false, false) => return Ok(None),
    };
    let next_base = if file.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
        file.parent().unwrap_or(base).to_path_buf()
    } else {
        file.parent().unwrap_or(base).join(name)
    };
    Ok(Some((file, next_base)))
}

/// Resolve an unconditional `#[path = "rel"] mod name;` to its author-chosen file and the base
/// directory for its own children. `rel` is relative to `base` — the containing file's own directory
/// (`file_dir`), with each enclosing inline-`mod` name already accumulated onto it by the caller;
/// for a non-mod-rs `name.rs` this differs from the conventional-child directory a plain `mod name;`
/// uses. A `#[path]`-loaded file is mod-rs-like, so its children resolve from the target file's
/// **own** directory. `Ok(None)` when the target is absent (the caller tolerates a cfg-conditional
/// absence and fails loud otherwise) — no ambiguity is possible (the path names one file), unlike the
/// conventional `name.rs` / `name/mod.rs` pair.
/// Resolve a `#[path = "…"]` target, reporting whether the literal that reached it was **absolute**.
///
/// That third value is load-bearing for identity, not a detail: `Path::join` discards its receiver
/// exactly when the joinee is absolute, so an absolute literal names the same file in every checkout
/// while a relative one names a checkout-dependent file that the anchor relativizes. Labeling the
/// absolute case relative to the anchor is therefore what made its identity checkout-DEPENDENT — the
/// label came out relative wherever the target happened to lie under that checkout's anchor and
/// absolute wherever it did not, for one identical committed literal. The flag lets the label keep the
/// path as written, which is the same in both.
pub(crate) fn resolve_path_module(
    base: &Path,
    rel: &str,
) -> Result<Option<(PathBuf, PathBuf, bool)>, String> {
    let file = base.join(rel);
    if !is_regular_file(&file)? {
        return Ok(None);
    }
    let next_base = file.parent().unwrap_or(base).to_path_buf();
    Ok(Some((file, next_base, Path::new(rel).is_absolute())))
}
