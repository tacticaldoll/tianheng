//! The module-graph walk: from a crate's precomputed file list, resolve which modules Rust
//! actually compiles — the `mod`-declared graph reachable from the crate root — and select the
//! source files that belong to a governed module. An undeclared orphan file, an inline-only
//! shadow, and a `#[path]`-remapped module are all excluded, matching the compiler. Depends
//! downward on [`super::lexer`] (hygiene / token boundaries) and [`super::path_vocab`] (segment
//! canonicalization, containment, the `mod`-keyword test); reads files via `std::fs`.

use super::lexer::{balanced_group_end, clean_with_positions, is_ident_byte, read_path_string};
use super::path_vocab::{canonical_segment, is_mod_declaration_keyword, path_within};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Every `(file, module path)` that belongs to the governed `module` (it *is* `module`
/// or sits beneath it) **and** is reachable in the crate's module graph (see
/// [`reachable_modules`]). An undeclared orphan file is not a governed source file even
/// when its path would map under the module, because Rust never compiles it — so its
/// imports must not be observed. Operates on a precomputed file list so the crate's
/// source is scanned once per boundary. `remapped` is [`reachable_modules`]'s third return
/// value — every `(target file, logical module path)` pair reached through an unconditional
/// `#[path]` — added here alongside the structurally-derived files, since a remapped file's own
/// on-disk path rarely coincides with its logical module path. `remap_shadowed` is its fourth:
/// the module paths where a same-named conventional file really is an orphan (the `#[path]` is
/// the ONLY file-form source for that name — no plain-file sibling under a different `#[cfg]`
/// arm), so a structural file matching one of THOSE paths is excluded; membership in `remapped`
/// alone is not enough, since a legitimate plain-file sibling can share a path with a remap.
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
) -> Vec<(PathBuf, String)> {
    let structural = files.iter().filter_map(|file| {
        let relative = file.strip_prefix(src_dir).ok()?;
        let module_path = module_path_of(relative, root_relative);
        // A conventional file whose path is claimed by an inline-only module is an orphan Rust
        // never compiles as that module (the inline body is the module), so it is not a
        // governed source file — the same "not compiled ⇒ not governed" rule as an undeclared
        // orphan, keyed on the inline shadow rather than mere unreachability.
        if inline_only.contains(&module_path) {
            return None;
        }
        // A conventional file whose path coincides with a module remapped ELSEWHERE by
        // `#[path]`, with no plain-file sibling of its own, is the same orphan hazard: rustc
        // compiles the remap's target, never this same-named file, so it must not be governed in
        // the remap's place even though the logical module path IS now reachable (through its
        // real target, below). A plain-file sibling under a different `#[cfg]` arm is real,
        // though — `remap_shadowed` (not mere membership in `remapped`) is the right test.
        if remap_shadowed.contains(&module_path) {
            return None;
        }
        if !reachable.contains(&module_path) {
            return None;
        }
        if path_within(&module_path, module) {
            Some((file.clone(), module_path))
        } else {
            None
        }
    });
    let remap_entries = remapped.iter().filter_map(|(file, module_path)| {
        if path_within(module_path, module) {
            Some((file.clone(), module_path.clone()))
        } else {
            None
        }
    });
    structural.chain(remap_entries).collect()
}

/// The set of module paths reachable from the crate root via `mod` declarations — the
/// crate's real module graph as Rust compiles it, observed from source. The walk seeds at
/// `crate` (the crate-root file(s) `lib.rs` / `main.rs`), reads each reachable module's
/// file(s) for their top-level `mod name;` / `mod name { … }` declarations, and adds the
/// child `crate::…::name`.
///
/// A file's existence alone does **not** make it a module: an undeclared orphan file —
/// at the crate root (`src/serde.rs` with no `mod serde;`) or in a subtree
/// (`src/kernel/orphan.rs` that `kernel` never declares) — is never reached, so it is not
/// governed and its imports are not observed, matching the compiler (which never compiles
/// it). This realizes the spec's "a sibling `mod` is in scope" rule for the whole module
/// tree, not just the crate root. A reachable file that cannot be read is a scan error
/// ("cannot judge"), never a silent skip.
///
/// An inline `mod name { … }` is walked too: its own `mod` declarations — file-backed or
/// further inline — sit at brace depth > 0 of the file that declares it, so they are found by
/// re-scanning that declaration's own body span (see [`declared_modules_in`]), not the file's
/// top level. A file-backed grandchild reached only through an inline parent
/// (`mod parent { mod child; }`, compiling `parent/child.rs`) is therefore reachable like any
/// other declared module; the file itself is already indexed by [`module_path_of`], which is
/// purely structural (derived from the file's own path on disk), so no directory-base
/// bookkeeping is needed beyond locating the inline body to re-scan.
///
/// An **unconditional** `#[path = "…"]` file declaration is now *followed* to its author-chosen
/// target — closing the divergence from 渾儀/漏刻, which already follow it — resolved from
/// `path_base`: the containing file's own directory, with each enclosing inline-`mod` name
/// accumulated onto it (rustc's rule; the crate root's own directory is `src_dir`). A
/// `#[path]`-loaded file is itself mod-rs-like, so its own children (and any `#[path]` written
/// inside it) resolve from ITS OWN directory. A `cfg_attr`-wrapped `path` remains a stated,
/// cfg-conditional skip bound (never followed cfg-blind); an absent unconditional target is a
/// genuine broken reference and a scan error, never a silent skip; and a `#[path]` chain that
/// cycles back to an already-open source file (only reachable through `#[path]`, since ordinary
/// conventional/inline nesting is bounded by the crate's finite file list) is a scan error, never
/// an unbounded walk — mirroring 渾儀's ancestor-path (not monotonic whole-tree) cycle guard, so
/// two sibling declarations legitimately sharing one `#[path]` target is never misreported.
///
/// Returns `(reachable, inline_only, remapped, remap_shadowed)`. `inline_only` names every path
/// declared **inline-only** — declared with an inline body and NOT ALSO declared plain file-form
/// (`mod name;`) in its parent — so a same-named conventional file (`name.rs` / `name/mod.rs`)
/// beside it is an orphan Rust never compiles as that module. The walk does not read such a file
/// in place of the inline body, and [`governed_files`] excludes it, so an inline target remains
/// the self-describing inline constitution error rather than silently governing the orphan. The
/// inline body's OWN declarations are re-scanned regardless of `inline_only` — a plain-file or
/// `#[path]` sibling of the same name (only possible under mutually-exclusive `#[cfg]`, the
/// per-platform shim pattern) is additive with the inline body, cfg-blind, never mutually
/// exclusive; `inline_only` governs only whether a *stray, undeclared* same-named conventional
/// file is an orphan, which is moot once a plain file is genuinely declared. `remapped` is every
/// `(target file, logical module path)` pair reached through an unconditional `#[path]`;
/// `remap_shadowed` names the paths where that `#[path]` is the ONLY file-form source (no plain
/// file also declared) — the genuine orphan-shadow hazard [`governed_files`] excludes, since a
/// plain-file sibling under a different `#[cfg]` arm is real and must not be excluded merely for
/// sharing a path with an unrelated remap.
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
    // Index files by their path-derived module path so a module's file(s) are found fast. This
    // mapping is purely structural (derived from each file's own path relative to `src_dir`), so
    // a file backing a deeply inline-nested module (`parent/child.rs` for
    // `mod parent { mod child; }`) is already keyed at its correct `crate::parent::child` path —
    // discovering that path only requires walking into `parent`'s inline body, not recomputing a
    // directory base.
    let mut by_module: std::collections::BTreeMap<String, Vec<&PathBuf>> = Default::default();
    for file in files {
        if let Ok(relative) = file.strip_prefix(src_dir) {
            by_module
                .entry(module_path_of(relative, root_relative))
                .or_default()
                .push(file);
        }
    }

    // Where the walk finds a module's own `mod` declarations: either its file(s) (scanned at
    // top level) or, for an inline-only module, the byte span of its declaring `mod name { … }`
    // body within its declaring file's cleaned text (scanned at that span's own top level).
    // `path_base` is carried alongside so a `#[path]` found within a source can be resolved.
    #[derive(Clone)]
    enum ScanSource {
        File(PathBuf, PathBuf),
        Body(PathBuf, usize, usize, PathBuf),
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
    // Every source file already opened on the path from the crate root to a given module — the
    // cycle guard for a `#[path]` follow (see the doc comment above). Ordinary conventional/inline
    // nesting cannot cycle (bounded by the crate's finite file list), so this only needs to be
    // consulted, and only needs to grow, around a `#[path]` follow; it is still propagated through
    // every branch so a *later* `#[path]` anywhere in the subtree sees the full open chain.
    let mut ancestors: std::collections::BTreeMap<String, HashSet<PathBuf>> = Default::default();
    if let Some(root_files) = by_module.get("crate") {
        sources.insert(
            "crate".to_string(),
            root_files
                .iter()
                .map(|f| ScanSource::File((*f).clone(), src_dir.to_path_buf()))
                .collect(),
        );
        let mut root_ancestors = HashSet::new();
        for f in root_files {
            if let Ok(canon) = std::fs::canonicalize(f) {
                root_ancestors.insert(canon);
            }
        }
        ancestors.insert("crate".to_string(), root_ancestors);
    }
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        let Some(scan_sources) = sources.get(&module).cloned() else {
            continue; // no file backs this module and it declared no inline body; nothing to read
        };
        let module_ancestors = ancestors.get(&module).cloned().unwrap_or_default();
        // Classify each child across this module's source(s) before descending: a child seen with
        // an inline body but never a file declaration is inline-only. (A path seen both ways arises
        // only under mutually-exclusive `#[cfg]`; it is not inline-only — the cfg-blind bound.)
        let mut child_kinds: std::collections::BTreeMap<String, (bool, bool)> = Default::default();
        let mut child_bodies: std::collections::BTreeMap<
            String,
            Vec<(PathBuf, usize, usize, PathBuf)>,
        > = Default::default();
        // Every direct `#[path]` target seen for a name, across this module's source(s). A
        // mutually-exclusive `#[cfg]` gating two whole declarations of the same name with
        // DIFFERENT unconditional targets — the standard per-platform shim pattern
        // (`#[cfg(unix)] #[path="unix.rs"] mod imp;` / `#[cfg(windows)] #[path="windows.rs"] mod
        // imp;`) — is valid, common Rust; the scanner does not evaluate `#[cfg]`, so it follows
        // ALL of them (cfg-blind union), matching 渾儀's own cfg-blind observe-all policy for a
        // same-named file-form child. Picking only one (the prior single-target design) would
        // silently drop the inactive variant's imports — a false negative this design avoids.
        let mut child_direct_paths: std::collections::BTreeMap<String, Vec<(PathBuf, PathBuf)>> =
            Default::default();
        for source in &scan_sources {
            let (file, text, cleaned, positions, range, path_base) = match source {
                ScanSource::File(file, path_base) => {
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
                    )
                }
                ScanSource::Body(file, start, end, path_base) => {
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
                    )
                }
            };
            for declared in declared_modules_in(&cleaned, range) {
                let seen = child_kinds.entry(declared.name.clone()).or_default();
                if declared.is_inline {
                    seen.0 = true;
                    if let Some((start, end)) = declared.body {
                        child_bodies.entry(declared.name).or_default().push((
                            file.clone(),
                            start,
                            end,
                            path_base.clone(),
                        ));
                    }
                    continue;
                }
                let Some(eq_cleaned) = declared.direct_path_eq else {
                    // A PLAIN file declaration (no `#[path]`) — resolved conventionally via
                    // `by_module`. Kept a separate flag from a direct `#[path]` sibling below: the
                    // two are additive (a `#[cfg]`-gated per-platform shim commonly pairs a plain
                    // file on one platform with a `#[path]`-relocated one on another), never
                    // mutually exclusive.
                    seen.1 = true;
                    continue;
                };
                let Some(&orig_eq) = positions.get(eq_cleaned) else {
                    continue;
                };
                // A value this reader cannot decode falls back to the conventional (excluded,
                // non-relocated) handling — fail-safe, never a mis-decoded path — matching 漏刻's
                // own `read_path_string` precedent for the identical scenario. On valid rustc input
                // this reader decodes every accepted escape, so the fallback is not expected to fire.
                if let Some(rel) = read_path_string(text.as_bytes(), orig_eq + 1, text.len()) {
                    child_direct_paths
                        .entry(declared.name)
                        .or_default()
                        .push((PathBuf::from(rel), path_base.clone()));
                }
            }
        }
        for (child, (seen_inline, seen_plain_file)) in child_kinds {
            let child_path = format!("{module}::{child}");
            let mut next_ancestors = module_ancestors.clone();
            // Every declared source for a name is additive, cfg-blind, never mutually exclusive —
            // a mutually-exclusive `#[cfg]` per-platform shim can legitimately pair ANY two (or
            // three) of a plain conventional file, an inline body, and a `#[path]` remap under the
            // same name, and the scanner does not evaluate `#[cfg]`, so it must observe every
            // variant's own real content (never picking one and silently dropping the others'
            // children). The inline body's OWN declarations are therefore re-scanned whenever it
            // is declared at all, regardless of a plain-file or `#[path]` sibling — dropping them
            // whenever any sibling existed was a real false negative (a per-platform shim pairing
            // an inline body with a sibling silently lost the inline body's own children).
            if seen_inline {
                if let Some(bodies) = child_bodies.remove(&child) {
                    // rustc accumulates the inline-module name as a directory component: a
                    // `#[path]` (or further nested inline `mod`) inside THIS body resolves from
                    // `<parent's path_base>/<child>`, not the parent's own path_base unchanged.
                    sources
                        .entry(child_path.clone())
                        .or_default()
                        .extend(bodies.into_iter().map(|(file, start, end, base)| {
                            ScanSource::Body(file, start, end, base.join(&child))
                        }));
                }
            }
            // `inline_only` is narrower than "inline was declared": it drives ONLY the
            // orphan-shadow exclusion for a STRAY same-named conventional file that no
            // declaration brings into scope. That question is live only when no plain file is
            // ALSO declared (a declared plain file is real, not stray) — independent of whether a
            // `#[path]` sibling also exists, since a `#[path]` target relocates to an entirely
            // different file and never competes with `x`'s own conventional path.
            if seen_inline && !seen_plain_file {
                inline_only.insert(child_path.clone());
            }
            if seen_plain_file {
                if let Some(files) = by_module.get(&child_path) {
                    for f in files {
                        let base = f
                            .parent()
                            .map(Path::to_path_buf)
                            .unwrap_or_else(|| src_dir.to_path_buf());
                        sources
                            .entry(child_path.clone())
                            .or_default()
                            .push(ScanSource::File((*f).clone(), base));
                        if let Ok(canon) = std::fs::canonicalize(f) {
                            next_ancestors.insert(canon);
                        }
                    }
                }
            }
            if let Some(targets) = child_direct_paths.remove(&child) {
                // Every unconditional `#[path]` target is followed cfg-blind (see the
                // `child_direct_paths` doc above) and unioned alongside any plain-file sibling
                // registered just above: each target is resolved independently.
                if !seen_plain_file {
                    remap_shadowed.insert(child_path.clone());
                }
                for (rel, base) in targets {
                    let target = base.join(&rel);
                    if !target.is_file() {
                        return Err(format!(
                            "module '{child_path}' is remapped by #[path = \"{}\"] to a file that does not exist: '{}'",
                            rel.display(),
                            target.display()
                        ));
                    }
                    let canon = std::fs::canonicalize(&target)
                        .map_err(|err| format!("cannot resolve '{}': {err}", target.display()))?;
                    if module_ancestors.contains(&canon) {
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
                    sources
                        .entry(child_path.clone())
                        .or_default()
                        .push(ScanSource::File(target, own_dir));
                    next_ancestors.insert(canon);
                }
            }
            ancestors.insert(child_path.clone(), next_ancestors);
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
}

/// [`declared_modules_with_kind`] generalized to scan `cleaned[range]` instead of a whole file,
/// so it can be re-applied to an inline module's own body — the byte span between its braces —
/// to find the `mod` declarations nested inside it. `path_attr_before_item` scans backward
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
                            // this module turns out to be inline-only, from `body` below. A
                            // `#[path]` on an inline module is a no-op for rustc (the body IS the
                            // module), so it is always declared regardless of `path_attr_before_item`.
                            let close = balanced_group_end(bytes, k).unwrap_or(bytes.len());
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: true,
                                body: Some((k + 1, close.saturating_sub(1))),
                                direct_path_eq: None,
                            });
                            i = close;
                            continue;
                        }
                        Some(b';') => match path_attr_before_item(bytes, i) {
                            PathAttrKind::Excluded => {}
                            PathAttrKind::None => {
                                declared.push(DeclaredModule {
                                    name: canonical_segment(ident).to_string(),
                                    is_inline: false,
                                    body: None,
                                    direct_path_eq: None,
                                });
                            }
                            PathAttrKind::Direct(eq) => {
                                declared.push(DeclaredModule {
                                    name: canonical_segment(ident).to_string(),
                                    is_inline: false,
                                    body: None,
                                    direct_path_eq: Some(eq),
                                });
                            }
                        },
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
        other => other,
    }
}

fn attr_prefix_path_kind(bytes: &[u8]) -> PathAttrKind {
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
        if bytes[i..].starts_with(b"path")
            && bytes.get(i + 4).is_none_or(|byte| !is_ident_byte(*byte))
        {
            // The direct name-value form (`path = "…"`) is followed; anything else spelled
            // `path` (a bare `#[path]`/`#[path(...)]` — not valid syntax for a real rustc remap,
            // but matched conservatively as before) stays excluded rather than followed.
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            return if bytes.get(j) == Some(&b'=') {
                PathAttrKind::Direct(j)
            } else {
                PathAttrKind::Excluded
            };
        }
        // The combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
        // `#[cfg(<pred>)] #[path = "…"]`) is a conditional remap too — recognized cfg-blindly, the
        // same stated `#[path]` bound: cfg-conditional, so never followed. Matched only on a
        // genuine nested `path` meta, so a `#[cfg_attr(<pred>, deprecated)]` on a normal file
        // module is not mistaken for a remap.
        if bytes[i..].starts_with(b"cfg_attr")
            && bytes.get(i + 8).is_none_or(|byte| !is_ident_byte(*byte))
            && cfg_attr_prefix_has_path(&bytes[i + 8..])
        {
            return PathAttrKind::Excluded;
        }
    }
    PathAttrKind::None
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
        let dir = std::env::temp_dir().join(format!("guibiao-reach-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!("guibiao-custom-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!("guibiao-path-remap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        );
        let _ = std::fs::remove_dir_all(&dir);

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
    fn a_cfg_attr_nested_path_is_a_remap() {
        // `#[cfg_attr(<pred>, path = "…")]` (== `#[cfg(<pred>)] #[path = "…"]`) is a
        // conditional remap — the FILE module is out of scope, the same stated `#[path]` bound. Not
        // recognizing it would scan the module against a wrong/absent conventional file (a silent
        // false negative in the static dimension).
        assert!(
            declared_modules("#[cfg_attr(windows, path = \"os/windows.rs\")]\npub mod os;\n")
                .is_empty(),
            "a cfg_attr-nested path remap puts the FILE module out of scope",
        );
        // The remap may sit after the predicate among several applied attrs, and whitespace varies.
        assert!(
            declared_modules("#[cfg_attr(all(unix), deprecated, path = \"p.rs\")]\nmod a;\n")
                .is_empty(),
        );
        // A NESTED `cfg_attr` remap (== `#[cfg(all(a,b))] #[path]`) is
        // detected too — the recursion must descend the applied `cfg_attr`, or guibiao would
        // silently govern nothing while hunyi failed loud (a cross-dimension divergence).
        assert!(
            declared_modules("#[cfg_attr(a, cfg_attr(b, path = \"secret.rs\"))]\npub mod m;\n")
                .is_empty(),
            "a nested cfg_attr path remap is detected",
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
        let dir = std::env::temp_dir().join(format!("guibiao-customroot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        std::fs::write(src.join("core.rs"), "pub mod sub;\n").expect("write core.rs");
        std::fs::write(src.join("sub.rs"), "// sub\n").expect("write sub.rs");
        let files = rust_files(&src).expect("list files");
        let (with_root, _, _, _) =
            reachable_modules(&src, &files, Some(std::path::Path::new("core.rs"))).expect("walk");
        let (without_root, _, _, _) = reachable_modules(&src, &files, None).expect("walk");
        let _ = std::fs::remove_dir_all(&dir);
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
        let dir = std::env::temp_dir().join(format!("guibiao-inline-child-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-inline-child-gov-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        );
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!("guibiao-inline-chain-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!("guibiao-inline-modrs-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-inline-orphan-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-inline-path-remap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        );
        let _ = std::fs::remove_dir_all(&dir);

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
    fn a_path_remap_to_a_missing_target_is_a_scan_error() {
        // An unconditional `#[path]` target is a rustc compile error when absent — a genuine
        // broken reference, never a silent skip (the same "cannot judge, not nothing to judge"
        // discipline as an unreadable governed file).
        let dir =
            std::env::temp_dir().join(format!("guibiao-path-remap-missing-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"absent.rs\"]\npub mod kernel;\n",
        )
        .expect("write lib.rs");

        let files = rust_files(&src).expect("list files");
        let result = reachable_modules(&src, &files, None);
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-path-remap-cycle-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-path-remap-shared-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir =
            std::env::temp_dir().join(format!("guibiao-cfg-dual-path-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
    fn a_plain_file_sibling_of_a_path_remap_is_still_governed() {
        // rustc ground truth (verified with a real `cargo build`): `#[cfg(unix)] pub mod x;` +
        // `#[cfg(windows)] #[path = "windows_x.rs"] pub mod x;` compiles `x.rs` on unix — the
        // standard per-platform shim pairing a PLAIN file on one platform with a `#[path]`-
        // relocated one on another. A `#[path]` sibling must never suppress a same-named plain
        // file's own registration (the false negative this test pins): both are cfg-blind and
        // additive, never mutually exclusive, matching how multiple `#[path]` targets are already
        // unioned above.
        let dir = std::env::temp_dir().join(format!(
            "guibiao-cfg-plain-path-sibling-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        );
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!(
            "guibiao-cfg-inline-path-sibling-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
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
        );
        let _ = std::fs::remove_dir_all(&dir);

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
        let dir = std::env::temp_dir().join(format!(
            "guibiao-cfg-plain-inline-sibling-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
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
        let _ = std::fs::remove_dir_all(&dir);

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
}
