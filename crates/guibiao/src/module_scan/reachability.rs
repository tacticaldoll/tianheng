//! The module-graph walk: resolves reachable compiled modules from the crate root and
//! selects governed source files, excluding undeclared orphans, inline shadows, and
//! remap-shadowed paths. Depends on [`super::lexer`] and [`super::path_vocab`].

use super::lexer::{balanced_group_end, clean_with_positions, is_ident_byte, read_path_string};
use super::path_vocab::{canonical_segment, is_mod_declaration_keyword, path_within};
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use xuanji::ScanDepth;

/// Selects file paths belonging to the governed module that are reachable in the module graph.
/// Excludes undeclared orphan files, inline-only shadows, and remap-shadowed paths.
#[allow(clippy::too_many_arguments)]
pub(crate) fn governed_files(
    src_dir: &Path,
    files: &[PathBuf],
    module: &str,
    reachable: &std::collections::BTreeSet<String>,
    inline_only: &std::collections::BTreeSet<String>,
    remapped: &[(PathBuf, String)],
    remap_shadowed: &std::collections::BTreeSet<String>,
    root_relative: Option<&Path>,
    depth: ScanDepth,
) -> Vec<(PathBuf, String)> {
    let matches_depth = |mod_path: &str| -> bool {
        match depth {
            ScanDepth::Shallow => mod_path == module,
            _ => path_within(mod_path, module),
        }
    };
    let structural = files.iter().filter_map(|file| {
        let relative = file.strip_prefix(src_dir).ok()?;
        let module_path = module_path_of(relative, root_relative);
        if inline_only.contains(&module_path) || remap_shadowed.contains(&module_path) {
            return None;
        }
        if !reachable.contains(&module_path) {
            return None;
        }
        if matches_depth(&module_path) {
            Some((file.clone(), module_path))
        } else {
            None
        }
    });
    let remap_entries = remapped.iter().filter_map(|(file, module_path)| {
        if matches_depth(module_path) {
            Some((file.clone(), module_path.clone()))
        } else {
            None
        }
    });
    let mut seen = std::collections::BTreeSet::new();
    structural
        .chain(remap_entries)
        .filter(|entry| seen.insert(entry.clone()))
        .collect()
}

/// Resolves the set of module paths reachable from the crate root via `mod` declarations.
/// Returns `(reachable, inline_only, remapped, remap_shadowed)`.
/// Unreachable orphan files are excluded; unreadable reachable files return a scan error.
#[allow(clippy::type_complexity)]
pub(crate) fn reachable_modules(
    src_dir: &Path,
    files: &[PathBuf],
    root_relative: Option<&Path>,
) -> Result<
    (
        std::collections::BTreeSet<String>,
        std::collections::BTreeSet<String>,
        Vec<(PathBuf, String)>,
        std::collections::BTreeSet<String>,
    ),
    String,
> {
    // Index files by their path-derived module path — used ONLY to discover the crate root's own
    // file(s) below (`by_module.get("crate")`), the one place a module has no declaring source of
    // its own to probe a directory from. Every OTHER module's plain children are resolved by a
    // live per-source directory probe (see `child_plain_bases` below), not this index: a
    // structural, module-path-keyed lookup cannot tell which of a module's several sources (e.g.
    // mutually-exclusive `#[cfg]` arms) actually declared a given child, and — since a file can
    // physically coincide with a module's naive structural path even when that module was reached
    // through an unrelated `#[path]` remap — it can also phantom-match a stray, uncompiled file.
    let mut by_module: std::collections::BTreeMap<String, Vec<&PathBuf>> = Default::default();
    for file in files {
        if let Ok(relative) = file.strip_prefix(src_dir) {
            by_module
                .entry(module_path_of(relative, root_relative))
                .or_default()
                .push(file);
        }
    }
    // Indexed by literal path to check walk presence without symlink canonicalization aliasing.
    let files_literal: HashSet<&PathBuf> = files.iter().collect();

    // Where the walk finds a module's own `mod` declarations: either its file(s) (scanned at
    // top level) or, for an inline-only module, the byte span of its declaring `mod name { … }`
    // body within its declaring file's cleaned text (scanned at that span's own top level).
    // `path_base` is where a `#[path]` found WITHIN this source resolves its relative value from
    // (the file's own containing directory — rustc's rule for a `#[path]` written at file scope).
    // `child_base` is where THIS source's own PLAIN (`#[path]`-free) children live — a
    // *different* directory whenever this source is an ordinary flat `name.rs` file (its
    // conventional children nest under `<path_base>/<name>/`, not beside it), coinciding with
    // `path_base` for every mod-rs-like source (the crate root, an inline body, and a
    // `#[path]`-loaded file, which rustc treats as mod-rs-like regardless of its own filename).
    // The trailing `HashSet<PathBuf>` is this SPECIFIC source's own ancestor set — every file
    // already open on the exact descent path that reached THIS source (see the cycle-guard note
    // below) — never merged with a sibling source's.
    #[derive(Clone)]
    enum ScanSource {
        File(PathBuf, PathBuf, PathBuf, HashSet<PathBuf>),
        Body(PathBuf, usize, usize, PathBuf, PathBuf, HashSet<PathBuf>),
    }

    let mut reachable = std::collections::BTreeSet::new();
    let mut inline_only = std::collections::BTreeSet::new();
    let mut remapped: Vec<(PathBuf, String)> = Vec::new();
    // A module path whose ONLY file-form source is an unconditional `#[path]` remap (no plain
    // sibling declaration under any `#[cfg]` arm) — the case where a same-named conventional file
    // really is the orphan-shadow hazard `governed_files` must exclude. When a plain-file sibling
    // ALSO exists (the per-platform shim pattern), that file is real and must NOT be excluded, so
    // this is tracked separately from mere membership in `remapped`.
    let mut remap_shadowed = std::collections::BTreeSet::new();
    reachable.insert("crate".to_string());
    let mut sources: std::collections::BTreeMap<String, Vec<ScanSource>> = Default::default();
    if let Some(root_files) = by_module.get("crate") {
        let mut root_ancestors = HashSet::new();
        for f in root_files {
            root_ancestors.insert(xingbiao::canonicalize_or_fail(f)?);
        }
        sources.insert(
            "crate".to_string(),
            root_files
                .iter()
                .map(|f| {
                    ScanSource::File(
                        (*f).clone(),
                        src_dir.to_path_buf(),
                        src_dir.to_path_buf(),
                        root_ancestors.clone(),
                    )
                })
                .collect(),
        );
    }
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        let Some(scan_sources) = sources.get(&module).cloned() else {
            continue; // no file backs this module and it declared no inline body; nothing to read
        };
        // Classify each child across this module's source(s) before descending: a child seen with
        // an inline body but never a file declaration is inline-only. (A path seen both ways arises
        // only under mutually-exclusive `#[cfg]`; it is not inline-only — the cfg-blind bound.)
        let mut child_kinds: std::collections::BTreeMap<String, (bool, bool)> = Default::default();
        // An inline body's own accumulated directory is the DECLARING source's `child_base`
        // (never its `path_base`) joined with the inline name — so a plain `mod x { … }` declared
        // inside an ordinary flat `bar.rs` still accumulates as `<bar's own child directory>/x`,
        // not `<bar.rs's containing dir>/x` (verified against a real rustc build: `bar.rs`
        // containing `mod x { mod y; }` compiles `y` at `bar/x/y.rs`, not `x/y.rs`) — UNLESS an
        // unconditional `#[path = "…"]` precedes the inline header, in which case that value
        // (resolved from the declaring source's own `path_base`, the fifth tuple element here)
        // relocates the base its own file-form children resolve from instead (rustc's rule;
        // verified against a real build).
        let mut child_bodies: std::collections::BTreeMap<
            String,
            Vec<(
                PathBuf,
                usize,
                usize,
                PathBuf,
                Option<PathBuf>,
                HashSet<PathBuf>,
            )>,
        > = Default::default();
        // Every direct `#[path]` target seen for a name, across this module's source(s), paired
        // with the DECLARING SOURCE's own ancestor set (critically: per-source, not merged across
        // this module's other source(s) — see the cycle-guard note below). A mutually-exclusive
        // `#[cfg]` gating two whole declarations of the same name with DIFFERENT unconditional
        // targets — the standard per-platform shim pattern (`#[cfg(unix)] #[path="unix.rs"] mod
        // imp;` / `#[cfg(windows)] #[path="windows.rs"] mod imp;`) — is valid, common Rust; the
        // scanner does not evaluate `#[cfg]`, so it follows ALL of them (cfg-blind union),
        // matching 渾儀's own cfg-blind observe-all policy for a same-named file-form child.
        // Picking only one (the prior single-target design) would silently drop the inactive
        // variant's imports — a false negative this design avoids.
        let mut child_direct_paths: std::collections::BTreeMap<
            String,
            Vec<(PathBuf, PathBuf, HashSet<PathBuf>, bool)>,
        > = Default::default();
        let mut child_conditional_paths: std::collections::BTreeMap<
            String,
            Vec<(PathBuf, PathBuf, HashSet<PathBuf>, bool)>,
        > = Default::default();
        // Every `child_base` (NOT `path_base` — see the `ScanSource` doc above) a PLAIN
        // (`#[path]`-free) declaration for a name was seen under, each paired with the DECLARING
        // SOURCE's own ancestor set — critically per-source, exactly like `child_direct_paths`
        // above, and for the same reason: a mutually-exclusive `#[cfg]` arm's own ancestors must
        // never leak into a sibling arm's plain child.
        let mut child_plain_bases: std::collections::BTreeMap<
            String,
            Vec<(PathBuf, HashSet<PathBuf>, bool)>,
        > = Default::default();
        for source in &scan_sources {
            let (file, text, cleaned, positions, range, path_base, child_base, source_ancestors) =
                match source {
                    ScanSource::File(file, path_base, child_base, ancestors) => {
                        let text = std::fs::read_to_string(file).map_err(|err| {
                            format!("cannot read source file '{}': {err}", file.display())
                        })?;
                        let (cleaned, positions) = clean_with_positions(&text);
                        let len = cleaned.len();
                        (
                            file.clone(),
                            text,
                            cleaned,
                            positions,
                            0..len,
                            path_base.clone(),
                            child_base.clone(),
                            ancestors.clone(),
                        )
                    }
                    ScanSource::Body(file, start, end, path_base, child_base, ancestors) => {
                        let text = std::fs::read_to_string(file).map_err(|err| {
                            format!("cannot read source file '{}': {err}", file.display())
                        })?;
                        let (cleaned, positions) = clean_with_positions(&text);
                        (
                            file.clone(),
                            text,
                            cleaned,
                            positions,
                            *start..*end,
                            path_base.clone(),
                            child_base.clone(),
                            ancestors.clone(),
                        )
                    }
                };
            for declared in declared_modules_in(&cleaned, range) {
                let seen = child_kinds.entry(declared.name.clone()).or_default();
                if declared.is_inline {
                    seen.0 = true;
                    // An unconditional `#[path = "…"]` preceding this inline header relocates the
                    // base its own file-form children resolve from — resolved from THIS source's
                    // own `path_base` (where a `#[path]` found within it resolves from), exactly
                    // like the file-form direct-path handling below. A value this reader cannot
                    // decode falls back to the default accumulated directory, same fail-safe as
                    // the file-form case.
                    let relocated_base = declared.direct_path_eq.and_then(|eq_cleaned| {
                        let &orig_eq = positions.get(eq_cleaned)?;
                        let rel = read_path_string(text.as_bytes(), orig_eq + 1, text.len())?;
                        Some(path_base.join(rel))
                    });
                    if let Some((start, end)) = declared.body {
                        child_bodies.entry(declared.name).or_default().push((
                            file.clone(),
                            start,
                            end,
                            child_base.clone(),
                            relocated_base,
                            source_ancestors.clone(),
                        ));
                    }
                    continue;
                }
                if let Some(eq_cleaned) = declared.direct_path_eq {
                    if let Some(&orig_eq) = positions.get(eq_cleaned) {
                        if let Some(rel) =
                            read_path_string(text.as_bytes(), orig_eq + 1, text.len())
                        {
                            child_direct_paths.entry(declared.name).or_default().push((
                                PathBuf::from(rel),
                                path_base.clone(),
                                source_ancestors.clone(),
                                declared.has_bare_cfg,
                            ));
                        }
                    }
                } else {
                    seen.1 = true;
                    child_plain_bases
                        .entry(declared.name.clone())
                        .or_default()
                        .push((
                            child_base.clone(),
                            source_ancestors.clone(),
                            declared.has_bare_cfg,
                        ));
                    for &eq_cleaned in &declared.conditional_path_eqs {
                        if let Some(&orig_eq) = positions.get(eq_cleaned) {
                            if let Some(rel) =
                                read_path_string(text.as_bytes(), orig_eq + 1, text.len())
                            {
                                let candidate_target = path_base.join(&rel);
                                if candidate_target.is_file() {
                                    child_conditional_paths
                                        .entry(declared.name.clone())
                                        .or_default()
                                        .push((
                                            PathBuf::from(rel),
                                            path_base.clone(),
                                            source_ancestors.clone(),
                                            declared.has_bare_cfg,
                                        ));
                                }
                            }
                        }
                    }
                }
            }
        }
        for (child, (seen_inline, seen_plain_file)) in child_kinds {
            let child_path = format!("{module}::{child}");
            // Every declared source for a name is additive, cfg-blind, never mutually exclusive —
            // a mutually-exclusive `#[cfg]` per-platform shim can legitimately pair ANY two (or
            // three) of a plain conventional file, an inline body, and a `#[path]` remap under the
            // same name, and the scanner does not evaluate `#[cfg]`, so it must observe every
            // variant's own real content (never picking one and silently dropping the others'
            // children). The inline body's OWN declarations are therefore re-scanned whenever it
            // is declared at all, regardless of a plain-file or `#[path]` sibling — dropping them
            // whenever any sibling existed was a real false negative (a per-platform shim pairing
            // an inline body with a sibling silently lost the inline body's own children).
            //
            // Critically, each new source below carries ITS OWN ancestor set — the descent path
            // that reached exactly that file — rather than a set merged across this child's other
            // sources. Two mutually-exclusive `#[cfg]` arms of the SAME name are never
            // simultaneously open in any real build, so treating one arm's target as an "ancestor"
            // while scanning the OTHER arm's target would misreport a real, cross-arm `#[path]`
            // reference as a cycle (see the lesson recorded in `PROJECT.md`'s Decisions).
            if seen_inline {
                if let Some(bodies) = child_bodies.remove(&child) {
                    // rustc accumulates the inline-module name as a directory component: a
                    // `#[path]` (or further nested inline `mod`) inside THIS body — or a further
                    // plain child of it — resolves from `<parent's child_base>/<child>`, not the
                    // parent's own `path_base` (which, for an ordinary flat file, is a DIFFERENT,
                    // shallower directory — see the `ScanSource` doc above) — UNLESS an
                    // unconditional `#[path]` preceded this inline header, in which case
                    // `relocated_base` (resolved above) is authoritative instead. An inline body
                    // opens no new file and is itself mod-rs-like either way, so `path_base` and
                    // `child_base` coincide for it; it simply carries forward whichever source
                    // declared it — its own ancestor set is already correct as-is.
                    sources
                        .entry(child_path.clone())
                        .or_default()
                        .extend(bodies.into_iter().map(
                            |(file, start, end, base, relocated_base, source_ancestors)| {
                                let inline_dir =
                                    relocated_base.unwrap_or_else(|| base.join(&child));
                                ScanSource::Body(
                                    file,
                                    start,
                                    end,
                                    inline_dir.clone(),
                                    inline_dir,
                                    source_ancestors,
                                )
                            },
                        ));
                }
            }
            // Whether at least one plain declaration for this child actually resolved to a real
            // file — NOT merely whether one was declared: a bare-`#[cfg]`-tolerated declaration
            // (see below) can now be declared yet resolve to nothing, and that must not count as
            // "a real plain file exists" for the `inline_only` decision just below (a false
            // negative found on this session's own round-2 adversarial review — an inline arm
            // paired with an entirely-tolerated-away plain arm was wrongly excluded from
            // `inline_only`, misreporting the self-describing `inline_module_target_error` as a
            // generic `unknown_module_error`). Defaults to `false` (matching the original
            // `!seen_plain_file` semantics) when no plain declaration exists at all.
            let mut plain_file_resolved = false;
            if seen_plain_file {
                // Resolved by a live probe from EACH declaring source's own directory —
                // uniformly for every plain child, whether its module sits at its own
                // structurally-derived location or was itself reached through a `#[path]` remap
                // (or nests inside one). A structural, module-path-keyed index (the prior
                // `by_module`-based design) cannot make this distinction: it does not know which
                // specific source declared the child, so it either merges an unrelated sibling
                // arm's ancestors into this child's own (a false-positive cycle one hop later) or
                // phantom-matches a stray, uncompiled file that coincidentally sits at the naive
                // structural location of a remapped module's child (a false positive of a
                // different shape) — both real bugs found and fixed here. Probing per source
                // instead means a probed child's OWN further children are resolved by this exact
                // same mechanism when their turn comes, with no special-casing needed.
                let mut already_sourced: HashSet<PathBuf> = HashSet::new();
                // Whether ANY resolved candidate for this child sits at its own genuinely correct
                // structural location. If none do — every resolution was reached only through a
                // divergent (remap-derived) source — a coincidental file at the naive structural
                // location is a true orphan (see `remap_shadowed` below), exactly like the
                // existing #[path]-target exclusion, generalized to a probed plain child.
                let mut any_structural_match = false;
                for (base, source_ancestors, has_bare_cfg) in
                    child_plain_bases.remove(&child).into_iter().flatten()
                {
                    let flat = base.join(format!("{child}.rs"));
                    let nested = base.join(&child).join("mod.rs");
                    // A plain `mod {child};` backed by BOTH conventional file forms at once is a
                    // genuine rustc compile error (E0761, "found at both … and …") independent of
                    // any `#[cfg]` — unlike a missing file, this ambiguity needs no `#[cfg]`
                    // awareness to detect: two real files at one source's own base is never
                    // legitimate, cfg-gated or not. 漏刻's own `resolve_external_module` already
                    // hard-errors on this exact shape; mirrored here rather than silently
                    // accepting both as separate sources.
                    if flat.is_file() && nested.is_file() {
                        return Err(format!(
                            "module '{child_path}' resolves to both '{}' and '{}' — a plain \
                             `mod {child}` must be backed by exactly one file",
                            flat.display(),
                            nested.display()
                        ));
                    }
                    if !flat.is_file() && !nested.is_file() {
                        // A BARE `#[cfg(pred)]` on this declaration means the whole item is
                        // removed when `pred` is false — a missing file is then expected, not
                        // broken (matching 渾儀's `has_cfg_attr` tolerance for the identical
                        // shape). An unconditional plain `mod {child};` with no backing file is a
                        // real, unrecoverable compile error: fail loud rather than silently drop
                        // the module from `reachable` (the false negative this closes — BACKLOG's
                        // longstanding "圭表 gaining #[cfg] awareness... closes this for free").
                        if has_bare_cfg {
                            continue;
                        }
                        return Err(format!(
                            "module '{child_path}' is declared (`mod {child};`) but its source \
                             file could not be located (expected '{}' or '{}')",
                            flat.display(),
                            nested.display()
                        ));
                    }
                    for candidate in [flat, nested] {
                        if !candidate.is_file() {
                            continue;
                        }
                        let canon = xingbiao::canonicalize_or_fail(&candidate)?;
                        if !already_sourced.insert(canon.clone()) {
                            continue;
                        }
                        // A directory symlink cycle could otherwise let this live probe re-open
                        // an already-open source file forever, growing `child_path` without
                        // bound. Checked against `source_ancestors` — the specific declaring
                        // source's own ancestor set — never a set merged across a
                        // mutually-exclusive `#[cfg]` sibling's own source.
                        if source_ancestors.contains(&canon) {
                            return Err(format!(
                                "module '{child_path}' resolves to '{}', which cycles back to an already-open source file",
                                candidate.display()
                            ));
                        }
                        // `governed_files`' structural iterator keys every file by ITS OWN
                        // on-disk path (`module_path_of`) — which agrees with `child_path` for an
                        // ordinary, non-remapped module (that iterator already finds and governs
                        // it on its own) but diverges once any ancestor was reached through a
                        // `#[path]` remap. Recorded in `remapped` only in that divergent case —
                        // exactly like a direct `#[path]` target — so a plain child is never
                        // double-registered under its own already-correct structural identity.
                        // Agreeing on the PATH alone is not enough: `candidate` (this exact,
                        // literal path) must ALSO be a literal member of `files` — built from
                        // `rust_files`, which never recurses into a symlinked directory — or the
                        // structural iterator this branch defers to will never actually find it.
                        // Compared by LITERAL identity, not canonical: a symlinked-directory
                        // candidate must never be treated as "found" merely because its resolved
                        // target happens to alias some other, genuinely-walked file.
                        let structurally_matches = files_literal.contains(&candidate)
                            && candidate
                                .strip_prefix(src_dir)
                                .ok()
                                .is_some_and(|relative| {
                                    module_path_of(relative, root_relative) == child_path
                                });
                        if structurally_matches {
                            any_structural_match = true;
                        } else {
                            remapped.push((candidate.clone(), child_path.clone()));
                        }
                        let own_dir = canon
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| base.clone());
                        let new_child_base = base.join(&child);
                        let mut anc = source_ancestors.clone();
                        anc.insert(canon);
                        sources
                            .entry(child_path.clone())
                            .or_default()
                            .push(ScanSource::File(candidate, own_dir, new_child_base, anc));
                    }
                }
                plain_file_resolved = !already_sourced.is_empty();
                if !already_sourced.is_empty() && !any_structural_match {
                    // Every real source for this child was reached only through a divergent
                    // (remap-derived) path, so any OTHER file that merely happens to sit at the
                    // naive structural location `governed_files`' own scan would otherwise find is
                    // an uncompiled orphan, never a legitimate sibling — the same hazard
                    // `remap_shadowed` already excludes for a direct `#[path]` target.
                    remap_shadowed.insert(child_path.clone());
                }
            }
            // `inline_only` is narrower than "inline was declared": it drives ONLY the
            // orphan-shadow exclusion for a STRAY same-named conventional file that no
            // declaration brings into scope. That question is live only when no plain file
            // ACTUALLY RESOLVED (a merely-declared-but-tolerated-away plain arm is not real) —
            // independent of whether a `#[path]` sibling also exists, since a `#[path]` target
            // relocates to an entirely different file and never competes with `x`'s own
            // conventional path.
            if seen_inline && !plain_file_resolved {
                inline_only.insert(child_path.clone());
            }
            if let Some(targets) = child_direct_paths.remove(&child) {
                // Every unconditional `#[path]` target is followed cfg-blind (see the
                // `child_direct_paths` doc above) and unioned alongside any plain-file sibling
                // registered just above: each target is resolved independently.
                if !seen_plain_file {
                    remap_shadowed.insert(child_path.clone());
                }
                for (rel, base, target_ancestors, has_bare_cfg) in targets {
                    let target = base.join(&rel);
                    if !target.is_file() {
                        // A BARE `#[cfg(pred)]` co-occurring with this unconditional `#[path]`
                        // (e.g. `#[cfg(windows)] #[path = "windows_impl.rs"] mod imp;`) removes
                        // the whole item, `#[path]` included, when `pred` is false — rustc never
                        // attempts to resolve the target on such a build (verified against a real
                        // build: this compiles cleanly with the target entirely absent). Tolerate
                        // exactly like the plain-missing-file case below; an unconditional item
                        // with no accompanying `#[cfg]` still fails loud.
                        if has_bare_cfg {
                            continue;
                        }
                        return Err(format!(
                            "module '{child_path}' is remapped by #[path = \"{}\"] to a file that does not exist: '{}'",
                            rel.display(),
                            target.display()
                        ));
                    }
                    let canon = xingbiao::canonicalize_or_fail(&target)?;
                    // Checked against THIS target's own declaring source's ancestor set
                    // (`target_ancestors`), never a set merged across a mutually-exclusive
                    // `#[cfg]` sibling's own target — two such targets are never simultaneously
                    // open in any real build, so one's target must never gate the other's.
                    if target_ancestors.contains(&canon) {
                        return Err(format!(
                            "module '{child_path}' is remapped by #[path] to '{}', which cycles back to an already-open source file",
                            target.display()
                        ));
                    }
                    remapped.push((target.clone(), child_path.clone()));
                    // The base for anything `#[path]`-resolved further inside `target` (or a
                    // conventional child of it) is `canon`'s own parent, not `target`'s: `target`
                    // may still carry an unresolved `..` (e.g. `base.join("../lib.rs")`), and
                    // joining onto that lexically — rather than the OS-resolved directory — would
                    // accumulate an ever-longer path on each further hop instead of a bounded
                    // one, masking a real `#[path]` cycle behind an unrelated "path too long"
                    // error before the ancestor check above ever gets a chance to fire.
                    let own_dir = canon
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| base.clone());
                    let mut anc = target_ancestors.clone();
                    anc.insert(canon);
                    // A `#[path]`-loaded file is mod-rs-like regardless of its own filename, so
                    // `path_base` and `child_base` coincide (both `own_dir`) — its own `#[path]`
                    // siblings and its own plain children resolve from the same directory.
                    sources
                        .entry(child_path.clone())
                        .or_default()
                        .push(ScanSource::File(target, own_dir.clone(), own_dir, anc));
                }
            }
            if let Some(targets) = child_conditional_paths.remove(&child) {
                if !seen_plain_file && !child_direct_paths.contains_key(&child) {
                    remap_shadowed.insert(child_path.clone());
                }
                for (rel, base, target_ancestors, _has_bare_cfg) in targets {
                    let target = base.join(&rel);
                    if !target.is_file() {
                        continue;
                    }
                    let Ok(canon) = xingbiao::canonicalize_or_fail(&target) else {
                        continue;
                    };
                    if target_ancestors.contains(&canon) {
                        return Err(format!(
                            "module '{child_path}' is remapped by #[cfg_attr(..., path = ...)] to '{}', which cycles back to an already-open source file",
                            target.display()
                        ));
                    }
                    remapped.push((target.clone(), child_path.clone()));
                    let own_dir = canon
                        .parent()
                        .map(Path::to_path_buf)
                        .unwrap_or_else(|| base.clone());
                    let mut anc = target_ancestors.clone();
                    anc.insert(canon);
                    sources
                        .entry(child_path.clone())
                        .or_default()
                        .push(ScanSource::File(target, own_dir.clone(), own_dir, anc));
                }
            }
            if reachable.insert(child_path.clone()) {
                queue.push(child_path);
            }
        }
    }
    Ok((reachable, inline_only, remapped, remap_shadowed))
}

/// One `mod` declared at the top level of a byte range within already-cleaned (comment/string/
/// macro-body-stripped) text: its canonical name, whether it is inline (`{ … }`, `true`) or file
/// (`;`, `false`), and — for an inline declaration — the byte range of its body's *content*
/// (excluding the enclosing braces), so a caller can re-scan just that span to find further
/// declarations nested inside it. `direct_path_eq` is the cleaned-text position of the `=` in an
/// **unconditional** `#[path = "…"]` preceding a FILE declaration — cleaning has already dropped
/// the quoted value itself, so a caller resolves it by mapping this position back to the
/// original source (see [`super::lexer::clean_with_positions`]) and reading from there.
struct DeclaredModule {
    name: String,
    is_inline: bool,
    body: Option<(usize, usize)>,
    direct_path_eq: Option<usize>,
    conditional_path_eqs: Vec<usize>,
    /// Whether a BARE `#[cfg(...)]` (never `cfg_attr`) precedes this declaration — see
    /// [`has_bare_cfg_attr_before_item`]. Only meaningful for a non-inline (file-form) declaration
    /// with no resolvable file: it is the "might legitimately be absent on this build" signal, the
    /// same one hunyi's `has_cfg_attr` checks.
    has_bare_cfg: bool,
}

/// The test-only `declared_modules_with_kind` generalized to scan `cleaned[range]` instead of a
/// whole file, so it can be re-applied to an inline module's own body — the byte span between its
/// braces — to find the `mod` declarations nested inside it. `path_attr_before_item` scans backward
/// from a candidate unbounded by `range.start`, which stays correct here: the nearest preceding
/// `;`/`{`/`}` it finds is either an earlier sibling's terminator within the range or the range's
/// own enclosing `{`, never a byte outside the declaration it is checking.
fn declared_modules_in(cleaned: &str, range: std::ops::Range<usize>) -> Vec<DeclaredModule> {
    let bytes = cleaned.as_bytes();
    let end = range.end.min(bytes.len());
    let mut declared = Vec::new();
    let mut depth: i32 = 0;
    let mut i = range.start.min(end);
    while i < end {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'm' if depth == 0 && is_mod_declaration_keyword(bytes, i) => {
                let mut j = i + 3;
                while j < end && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < end
                    && !bytes[j].is_ascii_whitespace()
                    && bytes[j] != b';'
                    && bytes[j] != b'{'
                {
                    j += 1;
                }
                let ident = cleaned[start..j].trim();
                let mut k = j;
                while k < end && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if !ident.is_empty() {
                    match bytes.get(k) {
                        Some(b'{') => {
                            // Skip the whole body in one jump — its content is re-scanned only if
                            // this module turns out to be inline-only, from `body` below. The
                            // module itself is always declared regardless of a preceding
                            // `#[path]` (rustc's `path` attribute never relocates an inline
                            // body's OWN content — the body already IS the module). It is NOT a
                            // no-op, though: it relocates the base directory THIS body's own
                            // file-form children resolve from (verified against a real rustc
                            // build: `#[path = "d"] mod x { mod y; }` compiles `y` at `d/y.rs`,
                            // never `<parent's child_base>/x/y.rs`) — an unconditional direct
                            // value is captured here (`direct_path_eq`) for exactly that reason;
                            // a `cfg_attr`-wrapped one stays the same stated, cfg-conditional skip
                            // bound as the file-form case (never followed cfg-blind).
                            let (direct_path_eq, conditional_path_eqs) =
                                match path_attr_before_item(bytes, i) {
                                    PathAttrKind::Direct(eq) => (Some(eq), Vec::new()),
                                    PathAttrKind::ConditionalRemaps(eqs) => (None, eqs),
                                    PathAttrKind::None | PathAttrKind::Excluded => {
                                        (None, Vec::new())
                                    }
                                };
                            let close = balanced_group_end(bytes, k).unwrap_or(bytes.len());
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: true,
                                body: Some((k + 1, close.saturating_sub(1))),
                                direct_path_eq,
                                conditional_path_eqs,
                                has_bare_cfg: false,
                            });
                            i = close;
                            continue;
                        }
                        Some(b';') => {
                            let has_bare_cfg = has_bare_cfg_attr_before_item(bytes, i);
                            let (direct_path_eq, conditional_path_eqs) =
                                match path_attr_before_item(bytes, i) {
                                    PathAttrKind::Direct(eq) => (Some(eq), Vec::new()),
                                    PathAttrKind::ConditionalRemaps(eqs) => (None, eqs),
                                    PathAttrKind::None | PathAttrKind::Excluded => {
                                        (None, Vec::new())
                                    }
                                };
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: false,
                                body: None,
                                direct_path_eq,
                                conditional_path_eqs,
                                has_bare_cfg,
                            });
                        }
                        _ => {}
                    }
                }
                i += 3;
            }
            _ => i += 1,
        }
    }
    declared
}

/// Names of modules declared at the top level (brace depth 0) of `source`, each paired with
/// whether it is an **inline** declaration (`mod name { … }`, `true`) or a **file** declaration
/// (`mod name;`, `false`) — the distinction [`reachable_modules`] needs to tell a real
/// file-backed module from an inline body whose same-named conventional file is an orphan.
/// Declared at any visibility (`pub mod`, `pub(crate) mod`, …). Comments, string/char literals,
/// and macro bodies are stripped first, so a commented-out, quoted, or macro-generated `mod` is
/// not counted; a `mod` nested inside another item (depth > 0) declares a child module, not a
/// crate-root one, and is skipped. Names are canonicalized (`r#name` -> `name`). Robust over
/// malformed input: it never panics (the same tolerance as the `use` scanner). Test-only: the
/// reachability walk itself calls [`declared_modules_in`] directly (over both whole files and
/// inline body spans), so production code no longer goes through this whole-file convenience.
#[cfg(test)]
fn declared_modules_with_kind(source: &str) -> Vec<(String, bool)> {
    // Strip macro bodies as well as comments/strings, the same hygiene the `use`
    // scanner applies: a `mod` written inside a macro body is macro-generated and out
    // of scope, so it must not be observed as a real declaration. (A `macro_rules!`
    // body is already excluded by brace depth; this also closes the `()`/`[]`-delimited
    // invocation gap, where `mod` would otherwise sit at brace depth 0.)
    let (cleaned, _positions) = clean_with_positions(source);
    let len = cleaned.len();
    declared_modules_in(&cleaned, 0..len)
        .into_iter()
        .map(|declared| (declared.name, declared.is_inline))
        .collect()
}

/// The declared module names only, discarding the inline/file kind — a test-only convenience
/// wrapping [`declared_modules_with_kind`] (itself test-only; see its doc).
#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declared_modules_with_kind(source)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// What the top-level item prefix before a `mod` keyword says about a `#[path]` remap. The
/// static scanner intentionally does not read attributes in general, but `path` is a stated
/// coverage concern either way: an unconditional, direct `#[path = "…"]` is now *followed*
/// (`Direct`, carrying the cleaned-text position of its `=`, so the real value can be read from
/// the untouched original source); a `cfg_attr`-wrapped one stays cfg-conditional and excluded,
/// same as a bare `path`-named attribute with no followable value — both `Excluded`, matching the
/// stated bound: a path-remapped module is not conventionally file-backed, so treating the `mod`
/// token as an ordinary file declaration would govern the wrong file (or a same-named orphan).
enum PathAttrKind {
    None,
    Direct(usize),
    ConditionalRemaps(Vec<usize>),
    Excluded,
}

fn path_attr_before_item(bytes: &[u8], mod_index: usize) -> PathAttrKind {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    match attr_prefix_path_kind(&bytes[start..mod_index]) {
        PathAttrKind::Direct(relative) => PathAttrKind::Direct(start + relative),
        PathAttrKind::ConditionalRemaps(relatives) => {
            PathAttrKind::ConditionalRemaps(relatives.into_iter().map(|rel| start + rel).collect())
        }
        other => other,
    }
}

fn attr_prefix_path_kind(bytes: &[u8]) -> PathAttrKind {
    let mut i = 0;
    let mut excluded = false;
    let mut conditional_eqs = Vec::new();
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'[') {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes[i..].starts_with(b"path")
            && bytes.get(i + 4).is_none_or(|byte| !is_ident_byte(*byte))
        {
            // An unconditional direct `path = "…"` wins over a cfg-conditional remap seen
            // elsewhere on the same item, regardless of which is scanned first: it is what
            // rustc compiles whenever a sibling `cfg_attr(pred, path = "…")`'s predicate is
            // false, so the scan must not stop at whichever `#[path]`-ish attribute comes first
            // textually — an early return here made the result attribute-order-dependent, a real
            // false negative when the cfg_attr happened to precede the direct attribute.
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes.get(j) == Some(&b'=') {
                return PathAttrKind::Direct(j);
            }
            // A bare `#[path]`/`#[path(...)]` (not valid remap syntax) excludes on its own, but
            // a later unconditional `#[path = "…"]` on the same item still wins — keep scanning
            // rather than returning.
            excluded = true;
            continue;
        }
        // The combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
        // `#[cfg(<pred>)] #[path = "…"]`) is a conditional remap. Collect candidate path = "..."
        // positions across all cfg_attr occurrences. An unconditional `#[path = "…"]` elsewhere
        // on the same item still wins (above), so this keeps scanning instead of returning immediately.
        if bytes[i..].starts_with(b"cfg_attr")
            && bytes.get(i + 8).is_none_or(|byte| !is_ident_byte(*byte))
        {
            cfg_attr_prefix_collect_path_eqs(&bytes[i + 8..], i + 8, &mut conditional_eqs);
            if conditional_eqs.is_empty() && cfg_attr_prefix_has_path(&bytes[i + 8..]) {
                excluded = true;
            }
            continue;
        }
    }
    if !conditional_eqs.is_empty() {
        PathAttrKind::ConditionalRemaps(conditional_eqs)
    } else if excluded {
        PathAttrKind::Excluded
    } else {
        PathAttrKind::None
    }
}

fn cfg_attr_prefix_collect_path_eqs(bytes: &[u8], base_offset: usize, eqs: &mut Vec<usize>) {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return;
    }
    i += 1;
    let mut depth = 1usize;
    let mut past_predicate = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                if ident == b"path" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        eqs.push(base_offset + j);
                    }
                } else if ident == b"cfg_attr" {
                    cfg_attr_prefix_collect_path_eqs(&bytes[i..], base_offset + i, eqs);
                }
            }
            _ => i += 1,
        }
    }
}

/// Whether a BARE `#[cfg(...)]` attribute (never `cfg_attr`) is among the attribute prefix
/// immediately preceding an item — the same "might legitimately be absent on this build" signal
/// hunyi's `has_cfg_attr` checks via `syn` (`crate::syn_util::has_cfg_attr`), hand-rolled here for
/// this crate's syn-free scanner. Deliberately narrow: this detects mere PRESENCE of the `cfg`
/// identifier, never evaluates a predicate — the same syntactic-identifier-only shape already used
/// above to detect `path`/`cfg_attr`, not a new capability tier or a step toward general attribute
/// evaluation. `cfg_attr` is deliberately excluded (verified against a real `rustc` build): unlike
/// a bare `#[cfg(pred)]`, which removes the whole item when `pred` is false, `#[cfg_attr(pred, …)]`
/// never removes the item — it only conditionally applies its wrapped attribute — so it must never
/// grant this tolerance (`#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a
/// genuine compile error, E0583, on every platform).
fn has_bare_cfg_attr_before_item(bytes: &[u8], mod_index: usize) -> bool {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    attr_prefix_has_bare_cfg(&bytes[start..mod_index])
}

fn attr_prefix_has_bare_cfg(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'[') {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        // The byte immediately after `cfg` must not continue the identifier (excludes `cfg_attr`,
        // whose next byte is `_`).
        if bytes[i..].starts_with(b"cfg")
            && bytes.get(i + 3).is_none_or(|byte| !is_ident_byte(*byte))
        {
            return true;
        }
        i += 1;
    }
    false
}

/// Whether a `cfg_attr(…)` attribute — `bytes` positioned just after the `cfg_attr` identifier —
/// carries a `path` meta among its **applied attributes**. `cfg_attr(<predicate>, <attr>, …)`: the
/// first meta is the cfg predicate (a condition, not an applied attribute), so it is **skipped**
/// before matching — mirroring hunyi's `is_path_remap` (`metas.iter().skip(1)`), so the two
/// dimensions agree. Scans the balanced parenthesis group and matches a depth-1 `path` identifier,
/// past the predicate, immediately followed by `=` (the `path = "…"` name-value form); it also
/// **recurses** into a nested applied `cfg_attr(…)`, so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
/// detected too. Conservative — a same-suffixed identifier (`target_path`), a `path` nested inside a
/// predicate group (`all(…)`), or a `path` in the predicate position is **not** matched — so a
/// non-remapping `cfg_attr` is never mistaken for a remap (which would drop a governed module — the
/// inverse false negative).
///
/// Input note: this runs on comment/string-stripped bytes (`declared_modules_with_kind` applies
/// `strip_comments_and_strings` first), so a `path` inside a string literal cannot reach here; the
/// `b'"'` arm below is defense-in-depth for that upstream invariant, not a live path.
fn cfg_attr_prefix_has_path(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return false;
    }
    i += 1;
    let mut depth = 1usize;
    // The first depth-1 meta is the cfg predicate, not an applied attribute; only match a `path`
    // meta AFTER the first depth-1 comma, so `#[cfg_attr(path = "…", …)]` (a `path` cfg key) is not
    // mistaken for a remap.
    let mut past_predicate = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'"' => {
                // Strings are stripped upstream (see doc); defense-in-depth for the invariant.
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                if ident == b"path" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        return true;
                    }
                } else if ident == b"cfg_attr" && cfg_attr_prefix_has_path(&bytes[i..]) {
                    // A nested `cfg_attr(<pred>, …)` applied meta: recurse into ITS group (which
                    // skips its own predicate), so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
                    // detected too — matching hunyi's recursive `is_path_remap`.
                    return true;
                }
            }
            _ => i += 1,
        }
    }
    false
}

/// The module path of a source file, mapping the crate ROOT file to `crate` regardless of its
/// filename. Cargo permits a custom target root (`[lib] path = "src/core.rs"`,
/// `[[bin]] path = "src/app.rs"`), which [`file_module_path`] would otherwise map to
/// `crate::core` / `crate::app` — leaving `crate` empty so no submodule is ever reached (a false
/// negative / spurious exit-2). `root_relative` is that root file's path relative to `src_dir`
/// when known; for the conventional `lib.rs`/`main.rs` it coincides with what `file_module_path`
/// already returns, so passing `None` is safe for the common case.
fn module_path_of(relative: &Path, root_relative: Option<&Path>) -> String {
    if root_relative == Some(relative) {
        return "crate".to_string();
    }
    // A custom crate root (`[lib] path = "src/core.rs"`) is in effect when `root_relative` is known
    // and is NOT the conventional top-level `lib.rs`/`main.rs`. In that case a STRAY top-level
    // `lib.rs`/`main.rs` is not the crate root — the explicit `path` disables cargo's lib/bin
    // autodetection, so rustc never compiles it — and must not also claim the segment-less `crate`
    // module (which would union its declared modules into the real root and make them
    // phantom-reachable). It maps to `crate::lib` / `crate::main` like any other file and, being
    // undeclared from the true root, stays unreached — matching the compiler.
    let custom_root = root_relative.is_some_and(|r| !is_conventional_root(r));
    file_module_path(relative, custom_root)
}

/// Whether `relative` is a conventional top-level cargo target root — `lib.rs` or `main.rs`
/// directly under `src/` (no parent segment). These are the roots [`file_module_path`] already maps
/// to the segment-less `crate`; any other root file is a *custom* root set via an explicit
/// `[lib]`/`[[bin]]` `path`.
fn is_conventional_root(relative: &Path) -> bool {
    relative
        .file_name()
        .is_some_and(|n| matches!(n.to_string_lossy().as_ref(), "lib.rs" | "main.rs"))
        && relative.components().count() == 1
}

/// The module path of a source file from its path relative to `src/`:
/// `lib.rs`/`main.rs`/`mod.rs` contribute no segment; `kernel/foo.rs` ->
/// `crate::kernel::foo`.
fn file_module_path(relative: &Path, custom_root: bool) -> String {
    let components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let mut segments = vec![String::from("crate")];
    let last = components.len().saturating_sub(1);
    for (index, component) in components.iter().enumerate() {
        if index == last {
            let stem = component.strip_suffix(".rs").unwrap_or(component);
            // `mod.rs` names its directory at any depth. `lib.rs`/`main.rs` are segment-less ONLY at
            // the crate root of a conventional layout — they are the cargo *target* roots there, not
            // module names. When a CUSTOM root is in effect (`custom_root`), a top-level `lib.rs`/
            // `main.rs` is NOT the target root (cargo autodetection is off) and must keep its stem so
            // it does not masquerade as the segment-less `crate` alongside the true root. A declared
            // submodule file literally named `lib.rs`/`main.rs` (`mod lib;` inside a subdir →
            // `foo/lib.rs` = `crate::foo::lib`) contributes its stem like any other file; stripping
            // it at depth would mis-map it to its parent and drift from 渾儀's declaration-driven
            // descent (which resolves it correctly).
            let segmentless = stem == "mod"
                || (!custom_root && components.len() == 1 && matches!(stem, "lib" | "main"));
            if !segmentless {
                segments.push(canonical_segment(stem).to_string());
            }
        } else {
            segments.push(canonical_segment(component).to_string());
        }
    }
    segments.join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_scan::rust_files;

    /// A unique, self-cleaning source tree for module-reachability fixtures.
    ///
    /// Reachability tests differ in the module graph they write, but not in the filesystem
    /// plumbing needed to host it. Keep that plumbing here so every case cleans up on panic and a
    /// new case only describes the source shape it is exercising.
    struct TempSrcTree {
        dir: PathBuf,
        src: PathBuf,
    }

    impl TempSrcTree {
        fn new(label: &str) -> Self {
            let dir = std::env::temp_dir().join(format!(
                "guibiao-reachability-{label}-{}",
                std::process::id()
            ));
            let _ = std::fs::remove_dir_all(&dir);
            let src = dir.join("src");
            std::fs::create_dir_all(&src).expect("create temp src");
            Self { dir, src }
        }

        fn src(&self) -> &Path {
            &self.src
        }
    }

    impl Drop for TempSrcTree {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.dir);
        }
    }

    #[test]
    fn declared_modules_finds_only_top_level_declarations() {
        let source = r#"
            pub mod kernel;
            mod projection;
            pub(crate) mod runner;
            mod inline { mod nested_child; }   // nested_child is depth 1, not a root module
            // mod commented_out;
            fn f() { let _ = "mod string_literal;"; }
        "#;
        let mut mods = declared_modules(source);
        mods.sort();
        assert_eq!(
            mods,
            vec![
                "inline".to_string(),
                "kernel".to_string(),
                "projection".to_string(),
                "runner".to_string(),
            ],
            "only top-level mod declarations count; nested, commented, and quoted are excluded"
        );
    }

    #[test]
    fn reachable_modules_follows_mod_declarations_not_filenames() {
        // The crate root declares `mod kernel;`, but two orphan files exist that no `mod`
        // brings into scope: a root orphan (`serde.rs`) and a subtree orphan
        // (`kernel/orphan.rs`, which `kernel.rs` never declares). Only `crate` and
        // `crate::kernel` are reachable; the orphans are not — at the root OR in a subtree.
        let tree = TempSrcTree::new("declared-not-filenames");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("kernel")).expect("create temp src/kernel");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod kernel;\nuse serde::Deserialize;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("kernel.rs"), "// kernel declares no submodule\n")
            .expect("write kernel.rs");
        std::fs::write(src.join("serde.rs"), "// root orphan, undeclared\n")
            .expect("write serde.rs");
        std::fs::write(
            src.join("kernel/orphan.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write kernel/orphan.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        assert!(reachable.contains("crate"), "{reachable:?}");
        assert!(
            reachable.contains("crate::kernel"),
            "a declared `mod kernel;` is reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::serde"),
            "an undeclared root orphan is not reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::kernel::orphan"),
            "an undeclared subtree orphan is not reachable: {reachable:?}"
        );
    }

    #[test]
    fn a_stray_lib_beside_a_custom_root_is_not_a_second_crate_root() {
        // With a custom target root (`[lib] path = "src/core.rs"`), a
        // leftover top-level `lib.rs` is NOT the crate root — cargo never compiles it — so it must
        // not also claim the segment-less `crate` module. If both `core.rs` and `lib.rs` mapped to
        // `crate`, the stray file's `mod ghost;` would union into the real root and make
        // `crate::ghost` phantom-reachable (a spurious module-boundary violation on an uncompiled file).
        let tree = TempSrcTree::new("custom-root");
        let src = tree.src().to_path_buf();
        std::fs::write(src.join("core.rs"), "pub mod real;\n").expect("write core.rs");
        std::fs::write(
            src.join("real.rs"),
            "// real, declared from the true root\n",
        )
        .expect("write real.rs");
        std::fs::write(src.join("lib.rs"), "pub mod ghost;\n").expect("write stray lib.rs");
        std::fs::write(
            src.join("ghost.rs"),
            "// declared only by the uncompiled lib.rs\n",
        )
        .expect("write ghost.rs");

        let files = rust_files(&src).expect("list files");
        let root_relative = std::path::PathBuf::from("core.rs");
        let (reachable, _inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, Some(&root_relative)).expect("walk modules");
        assert!(
            reachable.contains("crate"),
            "the custom root seeds crate: {reachable:?}"
        );
        assert!(
            reachable.contains("crate::real"),
            "a module declared from the true root is reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::ghost"),
            "a module declared only by the stray, uncompiled lib.rs is NOT reachable: {reachable:?}"
        );
    }

    #[test]
    fn path_remapped_modules_are_followed_to_their_target() {
        // rustc ground truth: `#[path = "weird.rs"] pub mod kernel;` compiles `weird.rs` as
        // `crate::kernel` — verified with a real `cargo build`. The conventional orphan
        // `kernel.rs` (which the remap's presence puts out of scope, module-source hardening
        // v0.1.4) must stay excluded even though `crate::kernel` is now reachable.
        let tree = TempSrcTree::new("path-remap");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"weird.rs\"]\npub mod kernel;\npub mod normal;\n",
        )
        .expect("write lib.rs");
        let target = src.join("weird.rs");
        std::fs::write(&target, "use crate::projection::Thing;\n").expect("write remapped file");
        let orphan = src.join("kernel.rs");
        std::fs::write(&orphan, "use crate::wrong_file_if_observed::Thing;\n")
            .expect("write conventional orphan");
        std::fs::write(src.join("normal.rs"), "// normal module\n").expect("write normal.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );

        assert!(reachable.contains("crate::normal"), "{reachable:?}");
        assert!(
            reachable.contains("crate::kernel"),
            "a #[path]-remapped module is now followed to its target: {reachable:?}"
        );
        assert_eq!(
            remapped,
            vec![(target.clone(), "crate::kernel".to_string())],
            "the remap is recorded under its logical path: {remapped:?}"
        );
        assert!(
            governed
                .iter()
                .any(|(file, module)| file == &target && module == "crate::kernel"),
            "the real remapped target is governed under crate::kernel: {governed:?}"
        );
        assert!(
            !governed.iter().any(|(file, _)| file == &orphan),
            "the conventional orphan must not be governed in the remap's place: {governed:?}"
        );
    }

    #[test]
    fn a_plain_child_of_a_path_remapped_module_is_governed_from_the_remaps_own_directory() {
        // rustc ground truth (verified with a real `rustc` build): a `#[path]`-loaded file is
        // itself mod-rs-like, so a plain `mod child;` written inside it compiles relative to the
        // REMAP TARGET's own directory, not to `by_module`'s structural index (which is keyed by
        // each file's own on-disk path and has no entry under the logical `crate::kernel::child`
        // when the backing file physically lives at `other/child.rs`). Before this fix, the child
        // was reachable (inserted unconditionally) but never a member of `sources`, so it was
        // never scanned and never governed — a real `use` passed every boundary unobserved.
        let tree = TempSrcTree::new("remap-plain-child");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("other")).expect("create temp src/other");
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write remap target");
        let child_file = src.join("other/child.rs");
        std::fs::write(&child_file, "use crate::projection::Thing;\n").expect("write child.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );

        assert!(
            reachable.contains("crate::kernel::child"),
            "the remap target's own plain child is reachable: {reachable:?}"
        );
        assert!(
            governed
                .iter()
                .any(|(file, module)| file == &child_file && module == "crate::kernel::child"),
            "the remap target's own plain child is governed under its logical path, so its real \
             `use` is observed: {governed:?}"
        );
    }

    #[test]
    fn path_attribute_detection_is_specific() {
        assert_eq!(
            declared_modules("#[pathology]\npub mod kernel;\n"),
            vec!["kernel".to_string()],
            "only the real `path` attribute is a remap marker"
        );
        // Rust permits whitespace in an outer attribute head; the direct remap is still
        // recognized (and, in `reachable_modules`, followed) — it is not dropped from
        // `declared_modules` (unlike a cfg_attr-wrapped one, still tested as empty below).
        assert_eq!(
            declared_modules("# [ path = \"weird.rs\" ]\npub mod kernel;\n"),
            vec!["kernel".to_string()],
        );
    }

    #[test]
    fn a_cfg_attr_nested_path_collects_conditional_remaps() {
        // `#[cfg_attr(<pred>, path = "…")]` (== `#[cfg(<pred>)] #[path = "…"]`) is a
        // conditional remap — under union-scan semantics, the declared module is captured with its
        // candidate conditional targets so all physically existing files undergo governance.
        let mods =
            declared_modules("#[cfg_attr(windows, path = \"os/windows.rs\")]\npub mod os;\n");
        assert_eq!(mods, vec!["os".to_string()]);
        // The remap may sit after the predicate among several applied attrs, and whitespace varies.
        let mods_a =
            declared_modules("#[cfg_attr(all(unix), deprecated, path = \"p.rs\")]\nmod a;\n");
        assert_eq!(mods_a, vec!["a".to_string()]);
        // A NESTED `cfg_attr` remap (== `#[cfg(all(a,b))] #[path]`) is detected too.
        let mods_m =
            declared_modules("#[cfg_attr(a, cfg_attr(b, path = \"secret.rs\"))]\npub mod m;\n");
        assert_eq!(mods_m, vec!["m".to_string()]);
    }

    #[test]
    fn an_unconditional_path_attr_wins_regardless_of_cfg_attr_order() {
        // A `cfg_attr(pred, path = "…")` and an unconditional `#[path = "…"]` can legitimately
        // sit on the same item (the cfg_attr's target is used only when `pred` holds; the direct
        // one is what rustc compiles whenever it does not — verified against real rustc, which
        // accepts this and resolves to the direct target when `pred` is false). scanning used to
        // return on the FIRST recognized `#[path]`-ish attribute, so whichever was written first
        // "won" — a cfg_attr(path) textually BEFORE the direct #[path] made the whole declaration
        // `Excluded`, dropping a module rustc genuinely compiles on every build where `pred` is
        // false. Order must not matter: the unconditional attribute always wins.
        assert_eq!(
            declared_modules(
                "#[cfg_attr(some_platform, path = \"b.rs\")]\n#[path = \"a.rs\"]\npub mod x;\n"
            ),
            vec!["x".to_string()],
            "cfg_attr before the direct #[path] must not drop the module",
        );
        assert_eq!(
            declared_modules(
                "#[path = \"a.rs\"]\n#[cfg_attr(some_platform, path = \"b.rs\")]\npub mod x;\n"
            ),
            vec!["x".to_string()],
            "the direct #[path] first must keep working as before",
        );
    }

    #[test]
    fn a_cfg_attr_without_a_path_meta_is_not_a_remap() {
        // The inverse false negative: a `cfg_attr` that carries NO `path` meta must not be mistaken
        // for a remap, or a normal file module would be dropped from scope and never governed.
        assert_eq!(
            declared_modules("#[cfg_attr(test, derive(Debug))]\npub mod real;\n"),
            vec!["real".to_string()],
            "a cfg_attr without a path meta is not a remap",
        );
        // A `path` substring inside a predicate's STRING value is not a `path` meta.
        assert_eq!(
            declared_modules("#[cfg_attr(feature = \"path\", deprecated)]\npub mod real;\n"),
            vec!["real".to_string()],
            "a `path` inside a predicate string is not a path meta",
        );
        // A same-suffixed identifier (`target_path`) is not the `path` meta.
        assert_eq!(
            declared_modules("#[cfg_attr(unix, target_path = \"x\")]\npub mod real;\n"),
            vec!["real".to_string()],
        );
        // A NESTED cfg_attr that carries no `path` meta must not be mistaken for a remap either.
        assert_eq!(
            declared_modules("#[cfg_attr(a, cfg_attr(b, deprecated))]\npub mod real;\n"),
            vec!["real".to_string()],
            "a nested cfg_attr without a path meta is not a remap",
        );
        // `path` in the PREDICATE position (first meta) is a cfg key, not an applied `path` attr —
        // must not be mistaken for a remap (would drop a normal module = inverse false negative).
        // Mirrors hunyi's `skip(1)`, keeping the two dimensions in agreement.
        assert_eq!(
            declared_modules("#[cfg_attr(path = \"x\", deprecated)]\npub mod real;\n"),
            vec!["real".to_string()],
            "a `path` cfg predicate key is not an applied path remap",
        );
    }

    #[test]
    fn a_cfg_attr_nested_path_on_an_inline_module_does_not_drop_it() {
        // As with a direct #[path], a cfg_attr(path) on an INLINE module is a rustc no-op, so the
        // module stays declared.
        assert_eq!(
            declared_modules(
                "#[cfg_attr(windows, path = \"x.rs\")]\npub mod a { pub mod inner; }\n"
            ),
            vec!["a".to_string()],
        );
    }

    #[test]
    fn a_path_attr_on_an_inline_module_does_not_drop_it() {
        // `#[path]` remaps only a FILE `mod name;`; on an INLINE `mod name { … }` it is a no-op
        // for rustc (the body IS the module), so the module must stay declared — dropping it
        // would leave a compiled module unobserved.
        assert_eq!(
            declared_modules("#[path = \"x.rs\"]\npub mod a { pub mod inner; }\n"),
            vec!["a".to_string()],
            "an inline module with a (no-op) #[path] is still declared",
        );
        // Control: on a FILE mod, #[path] is now a followed remap (0.2.2) — still declared (by
        // `declared_modules`, which does not distinguish a remap from an ordinary declaration),
        // unlike the cfg_attr-wrapped case, which stays excluded (tested elsewhere).
        assert_eq!(
            declared_modules("#[path = \"x.rs\"]\npub mod a;\n"),
            vec!["a".to_string()],
            "a #[path]-remapped FILE module is declared, to be followed to its target",
        );
    }

    #[test]
    fn a_block_comment_before_a_mod_name_does_not_fuse_it() {
        // `mod/*c*/foo;` must not strip to `modfoo;` (which drops the
        // declaration); a block comment leaves a separator.
        assert_eq!(
            declared_modules("mod/*c*/foo;"),
            vec!["foo".to_string()],
            "a block comment after `mod` must not swallow the declaration",
        );
    }

    #[test]
    fn a_custom_crate_root_filename_maps_to_crate() {
        // A crate whose target root is a custom filename
        // (`[lib] path = "src/core.rs"`) must still have its submodules reachable. The root file's
        // relative path is passed as root_relative so it maps to `crate` (not `crate::core`).
        let tree = TempSrcTree::new("custom-root-maps-to-crate");
        let src = tree.src().to_path_buf();
        std::fs::write(src.join("core.rs"), "pub mod sub;\n").expect("write core.rs");
        std::fs::write(src.join("sub.rs"), "// sub\n").expect("write sub.rs");
        let files = rust_files(&src).expect("list files");
        let (with_root, _, _, _) =
            reachable_modules(&src, &files, Some(std::path::Path::new("core.rs"))).expect("walk");
        let (without_root, _, _, _) = reachable_modules(&src, &files, None).expect("walk");
        assert!(
            with_root.contains("crate::sub"),
            "with the custom root mapped to crate, its submodule is reachable: {with_root:?}"
        );
        assert!(
            !without_root.contains("crate::sub"),
            "without the root override, core.rs maps to crate::core and sub is unreachable: {without_root:?}"
        );
    }

    #[test]
    fn declared_modules_ignores_a_mod_inside_a_macro_invocation() {
        // A `mod` written inside a macro body is macro-generated and out of scope — the
        // same rule the `use` scanner already applies. `()`/`[]`-delimited invocations
        // were the gap (a `macro_rules!` body is already excluded by brace depth).
        assert!(declared_modules("some_macro!( mod ghost; );").is_empty());
        assert!(declared_modules("some_macro![ mod ghost; ];").is_empty());
        assert!(declared_modules("macro_rules! m { () => { mod ghost; }; }").is_empty());
        // A real top-level declaration is still found.
        assert_eq!(declared_modules("mod real;"), vec!["real".to_string()]);
    }

    #[test]
    fn an_inline_modules_file_backed_child_is_reachable() {
        // rustc ground truth (rustc 1.96.0): `pub mod parent { pub mod child; }` in lib.rs
        // compiles `src/parent/child.rs` as `crate::parent::child` — verified with a real
        // `cargo build`. `parent` owns no file of its own (inline-only), so before this fix the
        // walk stopped at `crate::parent` without ever discovering `child`: the forbidden false
        // negative this test pins (an import in the real compiled file going unobserved).
        let tree = TempSrcTree::new("inline-file-child-reachable");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("parent")).expect("create temp src/parent");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod parent {\n    pub mod child;\n}\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            src.join("parent/child.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write parent/child.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");

        assert!(
            inline_only.contains("crate::parent"),
            "parent has no file of its own: {inline_only:?}"
        );
        assert!(
            reachable.contains("crate::parent::child"),
            "the real compiled file-backed child of an inline module must be reachable: {reachable:?}"
        );
        assert!(
            !inline_only.contains("crate::parent::child"),
            "the child is file-backed, not inline-only: {inline_only:?}"
        );
    }

    #[test]
    fn an_inline_modules_file_backed_child_is_governed() {
        // The end-to-end shape of the false negative: `governed_files` must actually select the
        // real compiled file for scanning, not just mark its module path reachable.
        let tree = TempSrcTree::new("inline-file-child-governed");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("parent")).expect("create temp src/parent");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod parent {\n    pub mod child;\n}\n",
        )
        .expect("write lib.rs");
        let child_file = src.join("parent/child.rs");
        std::fs::write(&child_file, "use crate::projection::Thing;\n").expect("write child.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(
            governed
                .iter()
                .any(|(file, module)| file == &child_file && module == "crate::parent::child"),
            "the real compiled child file must be governed: {governed:?}"
        );
    }

    #[test]
    fn a_chain_of_inline_modules_reaches_its_file_backed_leaf() {
        // rustc ground truth (rustc 1.96.0): from a FILE-backed module (`kernel.rs`), three more
        // levels of INLINE nesting (`parent`, `a`, `b`) still resolve a file-backed leaf `c` at
        // `src/kernel/parent/a/b/c.rs` — verified with a real `cargo build`. Each inline level's
        // own body must be re-scanned in turn, not just the first one.
        let tree = TempSrcTree::new("inline-chain");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("kernel/parent/a/b")).expect("mkdirs");
        std::fs::write(src.join("lib.rs"), "pub mod kernel;\n").expect("write lib.rs");
        std::fs::write(
            src.join("kernel.rs"),
            "pub mod parent {\n    pub mod a {\n        pub mod b {\n            pub mod c;\n        }\n    }\n}\n",
        )
        .expect("write kernel.rs");
        std::fs::write(
            src.join("kernel/parent/a/b/c.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write the deep leaf file");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        assert!(
            reachable.contains("crate::kernel::parent::a::b::c"),
            "a file-backed leaf beneath a chain of inline modules must be reachable: {reachable:?}"
        );
    }

    #[test]
    fn an_inline_modules_mod_rs_style_child_is_reachable() {
        // rustc ground truth: `mod name;` beneath an inline parent may also resolve via the
        // `<name>/mod.rs` directory form, not just `<name>.rs` — the same two conventional forms
        // available to any file module, verified here under an inline ancestor.
        let tree = TempSrcTree::new("inline-mod-rs-child");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("parent/child")).expect("mkdirs");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod parent {\n    pub mod child;\n}\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            src.join("parent/child/mod.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write parent/child/mod.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        assert!(
            reachable.contains("crate::parent::child"),
            "a mod.rs-style child beneath an inline parent must be reachable: {reachable:?}"
        );
    }

    #[test]
    fn an_inline_only_grandparents_conventional_orphan_stays_excluded() {
        // The existing inline-only orphan-shadow bound (BUILT v0.1.4) must still hold for an
        // inline module discovered through this fix's new path: a stray conventional file
        // matching the INLINE parent's own name (not the file-backed child) is still an orphan
        // Rust never compiles, so it must stay unreachable and ungoverned.
        let tree = TempSrcTree::new("inline-orphan");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("parent")).expect("mkdirs");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod parent {\n    pub mod child;\n}\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            src.join("parent/child.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write the real compiled child");
        std::fs::write(
            src.join("parent.rs"),
            "use crate::wrong_file_if_observed::Thing;\n",
        )
        .expect("write the conventional orphan Rust never compiles");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        assert!(
            inline_only.contains("crate::parent"),
            "parent is declared inline-only: {inline_only:?}"
        );
        assert!(
            reachable.contains("crate::parent::child"),
            "the real compiled child stays reachable: {reachable:?}"
        );
    }

    #[test]
    fn a_path_remapped_child_nested_in_an_inline_parent_is_followed() {
        // rustc ground truth (rustc 1.96.0): `mod parent { #[path = "weird.rs"] mod child; }` at
        // the crate root resolves `weird.rs` relative to `parent`'s own accumulated directory
        // (`src/parent/weird.rs`), never `src/weird.rs` — the same base-directory rule 渾儀/漏刻
        // already follow for an inline-nested `#[path]`. The conventional orphan
        // `parent/child.rs` must stay excluded from governance even though `crate::parent::child`
        // is now reachable (through `weird.rs`).
        let tree = TempSrcTree::new("inline-path-remap");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("parent")).expect("mkdirs");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod parent {\n    #[path = \"weird.rs\"]\n    pub mod child;\n}\n",
        )
        .expect("write lib.rs");
        let target = src.join("parent/weird.rs");
        std::fs::write(&target, "use crate::projection::Thing;\n")
            .expect("write the real #[path] target");
        let orphan = src.join("parent/child.rs");
        std::fs::write(&orphan, "use crate::wrong_file_if_observed::Thing;\n")
            .expect("write the conventional orphan the remap must not fall back to");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(
            inline_only.contains("crate::parent"),
            "parent is declared inline-only: {inline_only:?}"
        );
        assert!(
            reachable.contains("crate::parent::child"),
            "a #[path]-remapped child nested in an inline parent is followed to its target, \
             resolved relative to parent's own accumulated directory: {reachable:?}"
        );
        assert_eq!(
            remapped,
            vec![(target.clone(), "crate::parent::child".to_string())],
            "resolved from src/parent/, not src/: {remapped:?}"
        );
        assert!(
            !governed.iter().any(|(file, _)| file == &orphan),
            "the conventional orphan must not be governed in the remap's place: {governed:?}"
        );
    }

    #[test]
    fn a_path_remap_value_with_a_backslash_newline_continuation_is_followed() {
        // A backslash immediately followed by a newline is a valid Rust string-literal line
        // continuation: it and the following line's leading whitespace are stripped, joining the
        // two fragments — verified against a real `rustc` build (`"moved\` + newline + indentation
        // + `b.rs"` decodes to `"movedb.rs"`, and rustc follows it). `decode_str_escapes` must
        // decode this the same way `syn` (used by 渾儀) does, or this crate silently drops the
        // remapped module from `reachable` instead of following it — a coverage gap found on a
        // v0.2.0..v0.2.1 cross-dimension sweep.
        let tree = TempSrcTree::new("path-remap-line-continuation");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"moved\\\n    b.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("movedb.rs"), "// the continuation-named target\n")
            .expect("write movedb.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate::kernel",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        // A weak `reachable.contains(..)` check alone would pass even if decoding silently
        // failed: `child_kinds`/`reachable` gain an entry for a declared name regardless of
        // whether its `#[path]` value ever decodes, so the real proof is that the DECODED
        // TARGET FILE is what actually governs `crate::kernel` — never a same-named orphan, and
        // never simply absent from `governed`.
        assert_eq!(
            governed.len(),
            1,
            "crate::kernel must be governed by exactly the continuation-decoded target: {governed:?}"
        );
        assert!(
            governed[0].0.ends_with("movedb.rs"),
            "the governing file must be the continuation-decoded target, not a stale orphan: {governed:?}"
        );
    }

    #[test]
    fn a_path_remap_to_a_missing_target_is_a_scan_error() {
        // An unconditional `#[path]` target is a rustc compile error when absent — a genuine
        // broken reference, never a silent skip (the same "cannot judge, not nothing to judge"
        // discipline as an unreadable governed file).
        let tree = TempSrcTree::new("path-remap-missing");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"absent.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");

        let files = rust_files(&src).expect("list files");
        let result = reachable_modules(&src, &files, None);
        assert!(
            result.is_err(),
            "a #[path] target that does not exist is a scan error, not a silent skip: {result:?}"
        );
    }

    #[test]
    fn a_path_remap_cycle_is_a_scan_error_not_a_hang() {
        // A `#[path]` may point `..` back to an already-open source file, creating a genuine
        // graph cycle rustc itself rejects (a recursion-limit error) rather than compiling —
        // the scanner must fail loud (exit 2) instead of looping/overflowing the stack. Ordinary
        // conventional/inline nesting cannot cycle (bounded by the finite file list), so this
        // guard is exercised only through a `#[path]` chain, mirroring 渾儀's ancestor-path guard.
        let tree = TempSrcTree::new("path-remap-cycle");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("a")).expect("mkdirs");
        // lib.rs declares `mod a { #[path = "../lib.rs"] mod b; }` — `b`'s target resolves from
        // `a`'s own accumulated directory (`src/a/`), so `../lib.rs` is `src/lib.rs` itself: the
        // crate root re-declares `mod a { ... }`, looping crate::a::b::a::b::… forever.
        std::fs::write(
            src.join("lib.rs"),
            "pub mod a {\n    #[path = \"../lib.rs\"]\n    pub mod b;\n}\n",
        )
        .expect("write lib.rs");

        let files = rust_files(&src).expect("list files");
        let result = reachable_modules(&src, &files, None);
        // Asserting on the specific message (not just `is_err()`) pins that this is genuinely the
        // ancestor-cycle guard firing, not an unrelated error (e.g. an OS path-length limit from
        // an unnormalized `..` accumulating across repeated hops) that would happen to also return
        // `Err` while leaving the actual guard unexercised.
        let err = result.expect_err(
            "a #[path] chain cycling back to an already-open file is a scan error, not a hang",
        );
        assert!(
            err.contains("cycles back"),
            "expected the ancestor-cycle guard's own message, got: {err}"
        );
    }

    #[test]
    fn two_declarations_sharing_one_path_remap_target_is_not_a_cycle() {
        // rustc ground truth (rustc 1.96.0): `#[path="s.rs"] mod a; #[path="s.rs"] mod b;`
        // compiles — the SAME file twice, as two distinct modules — matching 渾儀's own
        // "two modules sharing one #[path] target is not a cycle" precedent. An ancestor-path (not
        // monotonic whole-tree) guard is required or this legitimate, compilable input would be
        // misreported as a cycle (a false positive).
        let tree = TempSrcTree::new("path-remap-shared");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"s.rs\"]\npub mod a;\n#[path = \"s.rs\"]\npub mod b;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("s.rs"), "// shared target\n").expect("write s.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, remapped, _remap_shadowed) = reachable_modules(
            &src, &files, None,
        )
        .expect("two modules sharing one #[path] target is not a cycle (rustc compiles it)");
        assert!(reachable.contains("crate::a"), "{reachable:?}");
        assert!(reachable.contains("crate::b"), "{reachable:?}");
        assert_eq!(remapped.len(), 2, "{remapped:?}");
    }

    #[test]
    fn cfg_gated_sibling_path_declarations_are_followed_cfg_blind_both() {
        // rustc ground truth (verified with a real `cargo build` on a unix host): mutually
        // exclusive `#[cfg(unix)]` / `#[cfg(windows)]` gating two whole `mod imp;` declarations of
        // the SAME name, each with a DIFFERENT unconditional `#[path]` target, is the standard
        // per-platform shim pattern — valid, common Rust, not a name collision. The scanner does
        // not evaluate `#[cfg]`, so it must follow BOTH targets (cfg-blind union, matching 渾儀's
        // own same-named-file-form-child policy), not pick one arbitrarily: a single-target
        // design would silently drop the inactive platform's imports depending on scan/file order.
        let tree = TempSrcTree::new("cfg-dual-path");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(unix)]\n#[path = \"unix_impl.rs\"]\npub mod imp;\n#[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod imp;\n",
        )
        .expect("write lib.rs");
        let unix_target = src.join("unix_impl.rs");
        std::fs::write(&unix_target, "use crate::projection::Unix;\n").expect("write unix_impl.rs");
        let windows_target = src.join("windows_impl.rs");
        std::fs::write(&windows_target, "use crate::projection::Windows;\n")
            .expect("write windows_impl.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("both cfg-gated targets are followed");
        assert!(reachable.contains("crate::imp"), "{reachable:?}");
        let mut targets: Vec<&PathBuf> = remapped
            .iter()
            .filter(|(_, module)| module == "crate::imp")
            .map(|(file, _)| file)
            .collect();
        targets.sort();
        let mut expected = vec![&unix_target, &windows_target];
        expected.sort();
        assert_eq!(
            targets, expected,
            "both platform targets are followed under crate::imp, cfg-blind: {remapped:?}"
        );
    }

    #[test]
    fn a_nested_path_crossing_into_a_cfg_siblings_own_target_is_not_a_cycle() {
        // rustc ground truth (verified with a real rustc build under EITHER single-feature
        // config): mutually-exclusive `#[cfg(feature = "a")]` / `#[cfg(feature = "b")]` gate two
        // `mod imp;` declarations with DIFFERENT unconditional `#[path]` targets (the standard
        // per-platform shim, already followed cfg-blind above) — variant_a.rs's OWN nested
        // `#[path]` legitimately points at variant_b.rs, the OTHER arm's target. The two targets
        // are never simultaneously open in any real single build, so this must compile (and be
        // observed) cleanly under either feature, never misreported as a cycle. Before the fix,
        // both targets' canons were unioned into ONE shared ancestor set for `crate::imp`, so
        // scanning variant_a.rs's own nested `#[path]` against that merged set wrongly matched
        // variant_b.rs's canon and returned a scan error for valid, compilable input.
        let tree = TempSrcTree::new("cfg-cross-arm-nested");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(feature = \"a\")]\n#[path = \"variant_a.rs\"]\npub mod imp;\n#[cfg(feature = \"b\")]\n#[path = \"variant_b.rs\"]\npub mod imp;\n",
        )
        .expect("write lib.rs");
        let variant_a = src.join("variant_a.rs");
        std::fs::write(
            &variant_a,
            "#[path = \"variant_b.rs\"]\nmod also_b;\nuse crate::projection::A;\n",
        )
        .expect("write variant_a.rs");
        let variant_b = src.join("variant_b.rs");
        std::fs::write(&variant_b, "use crate::projection::B;\n").expect("write variant_b.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect(
                "a nested #[path] crossing into a mutually-exclusive cfg sibling's own target must \
             not be misreported as a cycle",
            );
        assert!(reachable.contains("crate::imp"), "{reachable:?}");
        assert!(
            reachable.contains("crate::imp::also_b"),
            "the nested #[path] inside variant_a.rs is followed and governed: {reachable:?}"
        );
        let also_b_targets: Vec<&PathBuf> = remapped
            .iter()
            .filter(|(_, module)| module == "crate::imp::also_b")
            .map(|(file, _)| file)
            .collect();
        assert_eq!(
            also_b_targets,
            vec![&variant_b],
            "crate::imp::also_b resolves to variant_b.rs: {remapped:?}"
        );
    }

    #[test]
    fn a_nested_path_inside_an_inline_cfg_siblings_plain_child_is_not_a_cycle() {
        // rustc ground truth (verified with a real rustc build under the "u" feature): mutually
        // exclusive `#[cfg(feature = "u")] pub mod x { pub mod y; }` (inline) and
        // `#[cfg(feature = "w")] #[path = "windows_x.rs"] pub mod x;` (file-form, the standard
        // per-platform shim). `x`'s two cfg-sibling sources are an inline Body (ancestors =
        // {lib.rs}) and a #[path] File (ancestors = {lib.rs, windows_x.rs}) — but only the inline
        // source declares the plain child `y`. Before the fix, the plain-child branch unioned
        // ALL of `x`'s sources' ancestors regardless of which one actually declared `y`, so `y`'s
        // own ancestor set wrongly included `windows_x.rs`'s canon — and when `y.rs` legitimately
        // `#[path]`-references `windows_x.rs` (the OTHER, never-simultaneously-open cfg arm's own
        // target), the cycle guard misfired on valid, compilable input.
        let tree = TempSrcTree::new("cfg-inline-plain-child-cross-arm");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("x")).expect("create temp src/x");
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(feature = \"u\")]\npub mod x {\n    pub mod y;\n}\n#[cfg(feature = \"w\")]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
        )
        .expect("write lib.rs");
        std::fs::write(
            src.join("x/y.rs"),
            "#[path = \"../windows_x.rs\"]\nmod cross;\n",
        )
        .expect("write x/y.rs");
        std::fs::write(src.join("windows_x.rs"), "// the other cfg arm's target\n")
            .expect("write windows_x.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only, remapped, _remap_shadowed) = reachable_modules(
            &src, &files, None,
        )
        .expect(
            "a plain child's own nested #[path] crossing into a cfg sibling's target must not be a cycle",
        );
        assert!(reachable.contains("crate::x::y"), "{reachable:?}");
        assert!(
            reachable.contains("crate::x::y::cross"),
            "the nested #[path] inside y.rs is followed: {reachable:?}"
        );
        assert!(
            remapped
                .iter()
                .any(|(_, module)| module == "crate::x::y::cross"),
            "{remapped:?}"
        );
    }

    #[test]
    fn a_grandchild_of_a_probed_plain_child_is_governed() {
        // rustc ground truth (verified with a real rustc build): `#[path = "other/weird.rs"] pub
        // mod kernel;` where `other/weird.rs` declares `pub mod child;` (resolved to
        // `other/child.rs` via the live probe, fix 2) and `other/child.rs` itself declares a
        // further plain `pub mod grandchild;`. rustc compiles the grandchild at
        // `other/child/grandchild.rs` — the ordinary stem-subdirectory convention relative to
        // child.rs's own location, since child.rs (an ordinary flat file reached this way) is NOT
        // itself mod-rs-like. Before the fix, nothing resolved this: the probed child's own
        // `child_base` was never computed/carried forward, so its own plain children were
        // reachable (inserted unconditionally) but never governed — a real false negative.
        let tree = TempSrcTree::new("probed-child-grandchild");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("other/child")).expect("create temp dirs");
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write weird.rs");
        std::fs::write(src.join("other/child.rs"), "pub mod grandchild;\n")
            .expect("write other/child.rs");
        let grandchild_file = src.join("other/child/grandchild.rs");
        std::fs::write(&grandchild_file, "use crate::projection::Thing;\n")
            .expect("write grandchild.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(
            reachable.contains("crate::kernel::child::grandchild"),
            "{reachable:?}"
        );
        assert!(
            governed
                .iter()
                .any(|(file, module)| file == &grandchild_file
                    && module == "crate::kernel::child::grandchild"),
            "the probed child's own grandchild is governed under its logical path: {governed:?}"
        );
    }

    #[test]
    fn a_stray_file_at_a_remapped_modules_naive_structural_path_is_not_phantom_governed() {
        // rustc ground truth (verified with a real rustc build, including deliberately invalid
        // syntax in the stray file to confirm rustc never reads it): `#[path = "other/weird.rs"]
        // pub mod kernel;` means rustc NEVER looks at `kernel.rs` or `kernel/` at all — `kernel`
        // is wholly remapped. A leftover, wholly undeclared file that happens to physically sit
        // at the naive structural location a plain `mod child;` inside `kernel` would occupy if
        // `kernel` were NOT remapped (`src/kernel/child.rs`) is a true orphan. Before the fix, a
        // structural `by_module` lookup for the probed child's logical path did not know its
        // parent was remapped, so it phantom-matched this stray file alongside the real,
        // probe-resolved one — a false positive (an uncompiled file wrongly governed).
        let tree = TempSrcTree::new("remap-stray-structural-sibling");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("other")).expect("create temp src/other");
        std::fs::create_dir_all(src.join("kernel")).expect("create temp src/kernel");
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write weird.rs");
        let real_child = src.join("other/child.rs");
        std::fs::write(&real_child, "// the real, rustc-compiled child\n")
            .expect("write real child");
        // A stray file that coincidentally sits where a plain `mod child;` inside a
        // NON-remapped `kernel` would have looked — rustc never compiles this, since `kernel` is
        // wholly remapped to `other/weird.rs` and no `kernel.rs`/`kernel/mod.rs` exists.
        std::fs::write(
            src.join("kernel/child.rs"),
            "this is not even valid rust syntax {{{",
        )
        .expect("write stray file");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(
            governed
                .iter()
                .any(|(file, module)| file == &real_child && module == "crate::kernel::child"),
            "the real probed child is governed: {governed:?}"
        );
        assert_eq!(
            governed
                .iter()
                .filter(|(_, module)| module == "crate::kernel::child")
                .count(),
            1,
            "the stray file at the naive structural location must NOT be phantom-governed alongside the real one: {governed:?}"
        );
    }

    #[test]
    fn a_plain_file_sibling_of_a_path_remap_is_still_governed() {
        // rustc ground truth (verified with a real `cargo build`): `#[cfg(unix)] pub mod x;` +
        // `#[cfg(windows)] #[path = "windows_x.rs"] pub mod x;` compiles `x.rs` on unix — the
        // standard per-platform shim pairing a PLAIN file on one platform with a `#[path]`-
        // relocated one on another. A `#[path]` sibling must never suppress a same-named plain
        // file's own registration (the false negative this test pins): both are cfg-blind and
        // additive, never mutually exclusive, matching how multiple `#[path]` targets are already
        // unioned above.
        let tree = TempSrcTree::new("cfg-plain-path-sibling");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(unix)]\npub mod x;\n#[cfg(windows)]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
        )
        .expect("write lib.rs");
        let plain = src.join("x.rs");
        std::fs::write(&plain, "use crate::projection::Unix;\n").expect("write x.rs");
        let remapped_target = src.join("windows_x.rs");
        std::fs::write(&remapped_target, "use crate::projection::Windows;\n")
            .expect("write windows_x.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(reachable.contains("crate::x"), "{reachable:?}");
        assert!(
            governed.iter().any(|(f, _)| f == &plain),
            "the plain-file sibling must still be governed, not suppressed by the #[path] \
             sibling: {governed:?}"
        );
        assert!(
            governed.iter().any(|(f, _)| f == &remapped_target),
            "the #[path] sibling's real target must also be governed: {governed:?}"
        );
        assert!(
            !remap_shadowed.contains("crate::x"),
            "a plain-file sibling means x.rs is real, not an orphan-shadow: {remap_shadowed:?}"
        );
    }

    #[test]
    fn an_inline_sibling_of_a_path_remap_is_still_governed() {
        // rustc ground truth (verified with a real `cargo build`): `#[cfg(unix)] pub mod x {
        // pub mod y; }` + `#[cfg(windows)] #[path = "windows_x.rs"] pub mod x;` compiles the
        // inline body (and its own file-backed child `y`) on unix. An inline sibling is not the
        // plain-file-vs-inline cfg-blind bound (that bound is specifically about a same-named
        // CONVENTIONAL file, which a `#[path]` remap is not) — it must be observed alongside the
        // `#[path]` target, additively, the same as the plain-file case above.
        let tree = TempSrcTree::new("cfg-inline-path-sibling");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("x")).expect("mkdir x");
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(unix)]\npub mod x {\n    pub mod y;\n}\n#[cfg(windows)]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("x/y.rs"), "use crate::projection::Unix;\n").expect("write x/y.rs");
        let remapped_target = src.join("windows_x.rs");
        std::fs::write(&remapped_target, "use crate::projection::Windows;\n")
            .expect("write windows_x.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        assert!(
            reachable.contains("crate::x::y"),
            "the inline sibling's own file-backed child must still be reachable: {reachable:?}"
        );
        assert!(
            remapped
                .iter()
                .any(|(f, m)| f == &remapped_target && m == "crate::x"),
            "the #[path] sibling's real target must also be followed: {remapped:?}"
        );
        // `crate::x` is directly targetable despite carrying no file of its own besides the
        // remap target: `inline_only` marking it (there is no plain conventional file, so the
        // bound applies) does not suppress the remap's own governance — the remap is
        // unconditional in `governed_files`, never gated on `inline_only`.
        assert!(
            governed
                .iter()
                .any(|(f, m)| f == &remapped_target && m == "crate::x"),
            "crate::x is governed via its #[path] target regardless of the inline sibling: {governed:?}"
        );
    }

    #[test]
    fn an_inline_sibling_of_a_plain_file_is_still_governed() {
        // rustc ground truth (verified with a real `cargo build`, both feature configurations):
        // `#[cfg(not(feature = "b"))] pub mod x;` + `#[cfg(feature = "b")] pub mod x { pub mod y;
        // }` compiles the PLAIN `x.rs` by default and the INLINE body (with its own file-backed
        // child `x/y.rs` as `crate::x::y`) under feature `b`. The pre-existing v0.1.4 bound
        // ("a path declared both inline and file-form is observed through its conventional file")
        // is about which file backs `crate::x` itself for orphan-shadow purposes — it must not
        // also mean the inline body's OWN declarations go unscanned: `crate::x::y` is real,
        // compiled source under its own `#[cfg]` arm, and dropping it was a genuine false
        // negative (the scanner does not evaluate `#[cfg]`, so it must observe every variant).
        let tree = TempSrcTree::new("cfg-plain-inline-sibling");
        let src = tree.src().to_path_buf();
        std::fs::create_dir_all(src.join("x")).expect("mkdirs");
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(not(feature = \"b\"))]\npub mod x;\n#[cfg(feature = \"b\")]\npub mod x {\n    pub mod y;\n}\n",
        )
        .expect("write lib.rs");
        let plain = src.join("x.rs");
        std::fs::write(&plain, "use crate::projection::Plain;\n").expect("write x.rs");
        let inline_child = src.join("x/y.rs");
        std::fs::write(&inline_child, "use crate::projection::InlineChild;\n")
            .expect("write x/y.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, _remapped, _remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        assert!(
            !inline_only.contains("crate::x"),
            "a plain file is declared, so crate::x is not inline-only: {inline_only:?}"
        );
        assert!(
            reachable.contains("crate::x::y"),
            "the inline sibling's own file-backed child must still be reachable even though a \
             plain-file sibling of crate::x also exists: {reachable:?}"
        );
    }

    #[test]
    fn governed_files_does_not_duplicate_a_plain_files_own_path_remap_target() {
        // rustc ground truth (verified with a real `cargo build`, both feature configurations):
        // `#[cfg(not(feature = "b"))] pub mod a;` + `#[cfg(feature = "b")] #[path = "a.rs"] pub
        // mod a;` compiles the SAME `a.rs` under either arm — an unrelated `#[cfg]` arm's
        // `#[path]` can legitimately target the literal same file a plain-file sibling already
        // names. `governed_files`'s structural iterator (a real plain-file sibling, not shadowed)
        // and its `remap_entries` iterator (unconditional) then both name `(a.rs, crate::a)` —
        // pinning that the combined result carries it once, not twice.
        let tree = TempSrcTree::new("duplicate-remap-target");
        let src = tree.src().to_path_buf();
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg(not(feature = \"b\"))]\npub mod a;\n#[cfg(feature = \"b\")]\n#[path = \"a.rs\"]\npub mod a;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("a.rs"), "use crate::projection::Thing;\n").expect("write a.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, inline_only, remapped, remap_shadowed) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let governed = governed_files(
            &src,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            None,
            ScanDepth::Subtree,
        );
        let a_entries: Vec<_> = governed
            .iter()
            .filter(|(_, module)| module == "crate::a")
            .collect();
        assert_eq!(
            a_entries.len(),
            1,
            "the plain sibling and its own #[path] target are the same file — governed once, \
             not twice: {governed:?}"
        );
    }
}
