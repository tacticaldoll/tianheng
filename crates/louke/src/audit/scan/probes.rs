use super::lexer::*;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

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
/// suppress the second's never-accepted violation. That is the injectivity class the 0.4.0 window closed at
/// five other identity sites, and the same lossy-`display` lesson `write_baseline_atomically` applied to
/// the temp path. An identity component must not lose information the observation had.
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
        // Keep the path as the literal wrote it — no relativization. See `labeled`.
        xingbiao::path_label(file)
    } else {
        labeled(file, anchor)
    };
    scan_source_with_markers(&source, &label, markers, probes);
    Ok(source)
}

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
            // The legacy directory corpus reaches a file by walking the tree, never through a
            // `#[path]` literal, so no file it finds is absolute-reached.
            scan_rust_file(&path, anchor, markers, probes, false)?;
        }
    }
    Ok(())
}

pub(crate) fn collect_reachable_probes(
    root: &Path,
    anchor: &Path,
    markers: &[&str],
    probes: &mut Vec<Probe>,
) -> Result<(), String> {
    let root_parent = root
        .parent()
        .ok_or_else(|| format!("source root has no parent: {}", root.display()))?;
    // The third element is "reached through an ABSOLUTE `#[path]` literal", which decides whether this
    // file's identity label may be relativized against the anchor. It is INHERITED by a file's
    // children: they resolve from the absolute-reached file's own directory, so the same
    // does-it-happen-to-lie-under-this-checkout's-anchor coincidence applies to them.
    let mut pending = vec![(root.to_path_buf(), root_parent.to_path_buf(), false)];
    // Uses canonicalized path visit tracking to prevent cycle loops on symlinks.
    let mut visited: HashSet<PathBuf> = HashSet::new();
    while let Some((file, child_base, absolute_reached)) = pending.pop() {
        if !xingbiao::try_visit(&mut visited, &file)? {
            continue;
        }
        let source = scan_rust_file(&file, anchor, markers, probes, absolute_reached)?;
        // rustc resolves a non-inline `#[path]` relative to the **containing file's own directory**,
        // which differs from `child_base` (the conventional-child base `<dir>/name/`) for a non-mod-rs
        // file. Pass the file's own directory so a relocated module resolves where rustc compiles it.
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
        // A file's OWN bases are not absolute-reached by themselves; whether the FILE was reached that
        // way is ORed in by `collect_reachable_probes` when it queues these children. This flag tracks
        // only a base made absolute *within* the file, by an inline `mod`'s absolute `#[path]`.
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
/// here); otherwise it is a real broken reference (exit 2). Pulled out of
/// [`collect_scope_modules`]'s `mod name;` arm.
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
        // A conventional child's own EDGE is never an absolute literal, so its provenance is entirely
        // inherited: from the enclosing file (ORed by the walk) and from the base it resolved against,
        // which is absolute when an enclosing inline `mod` carried an absolute `#[path]`.
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
pub(crate) fn inline_mod_bases(
    attrs: &ModPreambleAttrs,
    name: &str,
    child_base: &Path,
    file_dir: &Path,
) -> Vec<(PathBuf, bool)> {
    // The flag is "this base was reached through an ABSOLUTE `#[path]` literal", and it must ride with
    // the base rather than be recovered later. `Path::join` discards its receiver exactly when the
    // joinee is absolute, so an absolute literal makes the base itself absolute and the children
    // resolved from it inherit the same already-non-portable construct. The walk's file-chain
    // inheritance cannot supply this: the base is introduced WITHIN one file, so a conventionally
    // declared child of the body was queued with `false` and its label relativized whenever the target
    // happened to sit under this checkout's anchor — two identities for one committed literal.
    let mut inline_bases: Vec<(PathBuf, bool)> = Vec::new();
    match &attrs.path {
        Some(rel) => inline_bases.push((file_dir.join(rel), Path::new(rel).is_absolute())),
        None => {
            for rel in &attrs.cfg_attr_paths {
                let candidate = file_dir.join(rel);
                if candidate.is_dir() {
                    inline_bases.push((candidate, Path::new(rel).is_absolute()));
                }
            }
            let conventional = child_base.join(name);
            if inline_bases.is_empty() || conventional.is_dir() {
                // A conventional base derives from the enclosing base; whether THAT was absolute-reached
                // is the caller's `absolute_base`, ORed in at the recursion below.
                inline_bases.push((conventional, false));
            }
        }
    }
    inline_bases
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
            // The one transparent macro's arms hold real declarations: descend each as its own
            // scope instead of skipping the body. This must be a POSITIVE descent, not merely a
            // skipped skip — the walk's own catch-all `{` handling below treats any other brace
            // block as opaque, so a removed skip alone would still swallow the arm.
            let mut name_end = i;
            while name_end > 0 && bytes[name_end - 1].is_ascii_whitespace() {
                name_end -= 1;
            }
            if is_transparent_macro_name(bytes, name_end) {
                if let Some(body_end) = foreign_macro_body_end(bytes, i) {
                    for (arm_start, arm_end) in transparent_arm_ranges(bytes, i, body_end) {
                        // The ENCLOSING bases, unchanged: an arm is not a module and adds no
                        // directory component the way an inline `mod` does. Accumulating one here
                        // would resolve an arm-declared `mod net;` under a phantom directory and
                        // drop every probe beneath it — the coverage false negative this walk
                        // exists to prevent, one layer down. Everything found inside an arm is
                        // cfg-conditional: the predicate lives in the macro's `if #[cfg(..)]`
                        // header, not on the item.
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
                    // Descending an inline `mod x { … }`: x's children resolve from `inline_base` —
                    // `<child_base>/name`, or `<file_dir>/dir` for an inline `#[path = "dir"]` remap.
                    // rustc accumulates the inline-module name as a directory component, so this base
                    // governs BOTH x's conventional file-children AND any `#[path]` nested in x's body
                    // — i.e. `inline_base` becomes the body's `file_dir` too, NOT the enclosing
                    // `file_dir`. (Threading the enclosing `file_dir` here dropped the inline
                    // component and read a same-named orphan — a false negative.)
                    let inline_bases = inline_mod_bases(&attrs, name, child_base, file_dir);
                    for (inline_base, base_is_absolute) in &inline_bases {
                        collect_scope_modules(
                            bytes,
                            cursor + 1,
                            close.saturating_sub(1),
                            inline_base,
                            inline_base,
                            modules,
                            // Arm membership is NOT inherited into an inline `mod`'s body: a bare
                            // `#[cfg]` on an outer `mod` does not tolerate an absent file for an
                            // inner one either, in any of the three dimensions.
                            false,
                            // Absolute-reached IS inherited, and accumulates: an absolute `#[path]` on
                            // this inline `mod` makes its base absolute, and an enclosing one already
                            // having done so carries through to a conventional base derived from it.
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
            // Any other brace scope — a fn/const/static body, a bare block expression, a match
            // arm, and so on — is descended into, not skipped as opaque: Rust permits a `mod`
            // item statement inside any block scope, and the ONLY legal non-inline form there is
            // a `#[path = "…"] mod name;` (a bare `mod name;` with no `#[path]` has no established
            // file-path convention inside a block and does not compile) — but that legal form was
            // previously invisible, since every brace here was treated as one opaque unit and never
            // walked. `is_mod_keyword`'s own whole-word match means descending into an ordinary
            // struct-literal/match-arm/expression body costs nothing: no real `mod` token exists
            // there to misfire on. A `mod` found this way adds no directory component of its own
            // (unlike a NAMED inline `mod x { … }`), so the enclosing file's own bases are threaded
            // through unchanged; arm membership is inherited, since a block nested inside an arm is
            // exactly as cfg-conditional as the arm itself.
            let close = balanced_brace_end(bytes, i, end);
            collect_scope_modules(
                bytes,
                i + 1,
                close.saturating_sub(1),
                child_base,
                file_dir,
                modules,
                in_transparent_arm,
                // A block scope adds no directory component, so the enclosing base — and whether it was
                // reached absolutely — carries through unchanged.
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
    let file = match (flat.is_file(), nested.is_file()) {
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
    if !file.is_file() {
        return Ok(None);
    }
    let next_base = file.parent().unwrap_or(base).to_path_buf();
    Ok(Some((file, next_base, Path::new(rel).is_absolute())))
}
