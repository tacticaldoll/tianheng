//! Reachability graph traversal and physical-source resolution.

use super::super::token_tree::{Edition, TokenTree};
use super::declarations::{block_path_modules, declared_modules_in};
use super::paths::module_path_of;
use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};

/// A physical file or inline body whose top-level declarations feed the graph walk.
///
/// The bases are deliberately distinct: `path_base` resolves attributes written in this
/// source, while `child_base` resolves its conventional file-form children. Ancestors belong
/// to this exact source path and must never be merged across cfg-blind sibling sources.
#[derive(Clone)]
enum ScanSource {
    File {
        file: PathBuf,
        path_base: PathBuf,
        child_base: PathBuf,
        ancestors: HashSet<PathBuf>,
    },
    /// An inline module's body, by the token indices of its `{` and `}` in the file's tree.
    Body {
        file: PathBuf,
        start: usize,
        end: usize,
        path_base: PathBuf,
        child_base: PathBuf,
        ancestors: HashSet<PathBuf>,
    },
}

struct LoadedSource {
    file: PathBuf,
    /// The token range whose top-level declarations this source holds: the whole file, or an inline body between
    /// its braces. `None` is the whole file, whose length is known only once it is lexed.
    range: Option<Range<usize>>,
    path_base: PathBuf,
    child_base: PathBuf,
    ancestors: HashSet<PathBuf>,
}

impl ScanSource {
    fn load(&self) -> LoadedSource {
        let (file, range, path_base, child_base, ancestors) = match self {
            Self::File {
                file,
                path_base,
                child_base,
                ancestors,
            } => (file, None, path_base, child_base, ancestors),
            Self::Body {
                file,
                start,
                end,
                path_base,
                child_base,
                ancestors,
            } => (
                file,
                Some(*start + 1..*end),
                path_base,
                child_base,
                ancestors,
            ),
        };
        LoadedSource {
            file: file.clone(),
            range,
            path_base: path_base.clone(),
            child_base: child_base.clone(),
            ancestors: ancestors.clone(),
        }
    }
}

struct InlineBody {
    file: PathBuf,
    start: usize,
    end: usize,
    base: PathBuf,
    /// A **direct** `#[path]`'s base: it replaces the conventional one outright.
    relocated_base: Option<PathBuf>,
    /// Every `cfg_attr(…, path = …)` base this declaration may compile, beside a direct one as without it: rustc takes
    /// the first path attribute written, which a `cfg_attr` written before a direct one is where its predicate holds,
    /// and one written after it never is, so the attribute reader keeps only the first kind. Candidates, not the
    /// base — see [`register_inline_sources`].
    candidate_bases: Vec<PathBuf>,
    ancestors: HashSet<PathBuf>,
}

struct PlainSource {
    base: PathBuf,
    ancestors: HashSet<PathBuf>,
    is_cfg_conditional: bool,
}

struct DirectPathSource {
    relative: PathBuf,
    base: PathBuf,
    ancestors: HashSet<PathBuf>,
    is_cfg_conditional: bool,
}

struct ConditionalPathSource {
    relative: PathBuf,
    base: PathBuf,
    ancestors: HashSet<PathBuf>,
}

/// What one child's declarations across the scanned sources say about where its source is.
///
/// `bodies` is non-empty exactly when the child is declared inline somewhere, since every inline declaration
/// carries a body. An inline-only child's suggested extraction path is the first declaring source's base, in
/// the order the sources were scanned — an example of a path rustc resolves, which the refusal words as one.
#[derive(Default)]
struct ChildSources {
    seen_plain_file: bool,
    bodies: Vec<InlineBody>,
    plain: Vec<PlainSource>,
    direct: Vec<DirectPathSource>,
    conditional: Vec<ConditionalPathSource>,
}

/// Collect child module declarations across the given scan sources.
///
/// A direct `#[path]` relocates the base, and every `cfg_attr(…, path = …)` target written before it, or with no direct
/// one at all, is a candidate base unioned in [`register_inline_sources`]: rustc compiles the first path attribute
/// written, and whether a `cfg_attr` before the direct one applies is a predicate this scanner does not evaluate. A
/// path written after the first direct one is never compiled, and the attribute reader drops it. For non-inline
/// declarations, candidates that physically exist prove a configuration compiles via that remap, granting tolerance to
/// absence of the conventional file. Unreadable targets return an error immediately via [`xingbiao::is_regular_file`].
///
/// A file of the crate is read and lexed once for the whole walk, into `texts` and `trees`: a file holding many inline
/// modules is the source of each of them, and each reads its own range of the one token tree. A file the crate's file
/// list does not hold is read and lexed where it is met.
fn collect_children<'t>(
    scan_sources: &[ScanSource],
    edition: Edition,
    texts: &'t HashMap<&PathBuf, OnceCell<String>>,
    trees: &mut HashMap<PathBuf, TokenTree<'t>>,
) -> Result<BTreeMap<String, ChildSources>, String> {
    let mut children: BTreeMap<String, ChildSources> = Default::default();
    for source in scan_sources {
        let loaded = source.load();
        let read = || {
            std::fs::read_to_string(&loaded.file).map_err(|err| {
                format!("cannot read source file '{}': {err}", loaded.file.display())
            })
        };
        let (unlisted, unlisted_tree);
        let tree = match texts.get(&loaded.file) {
            Some(slot) => {
                if !trees.contains_key(&loaded.file) {
                    let text = match slot.get() {
                        Some(text) => text,
                        None => {
                            let text = read()?;
                            slot.get_or_init(|| text)
                        }
                    };
                    trees.insert(loaded.file.clone(), TokenTree::lex(text, edition));
                }
                &trees[&loaded.file]
            }
            None => {
                unlisted = read()?;
                unlisted_tree = TokenTree::lex(&unlisted, edition);
                &unlisted_tree
            }
        };
        let range = loaded.range.clone().unwrap_or(0..tree.len());
        for declared in declared_modules_in(tree, range.clone())
            .into_iter()
            .chain(block_path_modules(tree, range))
        {
            let child_sources = children.entry(declared.name.clone()).or_default();
            if let Some((start, end)) = declared.body {
                let relocated_base = declared
                    .direct_path
                    .clone()
                    .flatten()
                    .map(|rel| loaded.path_base.join(rel));
                let candidate_bases: Vec<PathBuf> = declared
                    .conditional_paths
                    .iter()
                    .map(|rel| loaded.path_base.join(rel))
                    .collect();
                child_sources.bodies.push(InlineBody {
                    file: loaded.file.clone(),
                    start,
                    end,
                    base: loaded.child_base.clone(),
                    relocated_base,
                    candidate_bases,
                    ancestors: loaded.ancestors.clone(),
                });
                continue;
            }
            let mut resolved_conditional = Vec::new();
            for rel in &declared.conditional_paths {
                let candidate_target = loaded.path_base.join(rel);
                if xingbiao::is_regular_file(&candidate_target)? {
                    resolved_conditional.push(ConditionalPathSource {
                        relative: PathBuf::from(rel),
                        base: loaded.path_base.clone(),
                        ancestors: loaded.ancestors.clone(),
                    });
                }
            }
            let has_backing_conditional_target = !resolved_conditional.is_empty();
            child_sources.conditional.extend(resolved_conditional);

            if let Some(written) = &declared.direct_path {
                if let Some(rel) = written {
                    child_sources.direct.push(DirectPathSource {
                        relative: PathBuf::from(rel),
                        base: loaded.path_base.clone(),
                        ancestors: loaded.ancestors.clone(),
                        is_cfg_conditional: declared.is_cfg_conditional
                            || (declared.direct_path_is_conditional
                                && has_backing_conditional_target),
                    });
                }
            } else if declared.declared_in_block {
                if !has_backing_conditional_target {
                    return Err(format!(
                        "module `{}` a block declares names no file that exists by any of its `cfg_attr` path \
                         attributes, and a block's file-form `mod` has no conventional file, so no configuration \
                         builds it",
                        declared.name
                    ));
                }
            } else {
                child_sources.seen_plain_file = true;
                child_sources.plain.push(PlainSource {
                    base: loaded.child_base.clone(),
                    ancestors: loaded.ancestors.clone(),
                    is_cfg_conditional: declared.is_cfg_conditional
                        || has_backing_conditional_target,
                });
            }
        }
    }
    Ok(children)
}

#[derive(Default)]
struct GraphSources {
    by_module: BTreeMap<String, Vec<ScanSource>>,
    remapped: Vec<(PathBuf, String)>,
    remap_shadowed: BTreeSet<String>,
}

/// Register an inline `mod name { … }` body as a scan source, once per base its file-form children
/// may resolve from.
///
/// A **direct** `#[path]` relocates that base, and each `cfg_attr`-wrapped one written before it is a **candidate**
/// read beside it. With no direct one, a `cfg_attr`-wrapped path names a base per platform predicate, so every target
/// is a candidate, unioned with the conventional directory: the scanner does not evaluate `cfg` and cannot know which
/// arm a build compiles, so preferring one would silently drop the children beneath the other (the false negative the
/// core contract forbids). A candidate is descended only when it **exists as a directory** — recursing into an absent
/// one would spuriously fail loud on the body's other, unrelated nested items solely because one platform's directory
/// is missing, even when another candidate already backs them. When no candidate exists at all, the conventional base
/// is descended anyway, so a nested reference genuinely broken on every platform still fails loud exactly as it did
/// before this tolerance existed.
///
/// 漏刻 states a rule for the same shape and implements it independently (三儀 ⊥ 三儀: the same rule, not the
/// same function). The two are not yet one answer: 漏刻 still takes the last of several direct `#[path]`
/// attributes, where rustc and this walk take the first, which `BACKLOG.md` tracks with the inline-base
/// question under the 渾儀 and 漏刻 WATCH entry.
fn register_inline_sources(
    child: &str,
    child_path: &str,
    bodies: Vec<InlineBody>,
    graph: &mut GraphSources,
) -> Result<(), String> {
    let sources = graph.by_module.entry(child_path.to_string()).or_default();
    for body in bodies {
        let conventional = body.base.join(child);
        let bases: Vec<PathBuf> = match &body.relocated_base {
            Some(base) => {
                let mut bases = vec![base.clone()];
                for candidate in &body.candidate_bases {
                    if xingbiao::is_directory(candidate)? && !bases.contains(candidate) {
                        bases.push(candidate.clone());
                    }
                }
                bases
            }
            None if body.candidate_bases.is_empty() => vec![conventional],
            None => {
                let mut present: Vec<PathBuf> = Vec::new();
                for base in body
                    .candidate_bases
                    .iter()
                    .cloned()
                    .chain(std::iter::once(conventional.clone()))
                {
                    if xingbiao::is_directory(&base)? {
                        present.push(base);
                    }
                }
                present.sort();
                present.dedup();
                if present.is_empty() {
                    vec![conventional]
                } else {
                    present
                }
            }
        };
        for base in bases {
            sources.push(ScanSource::Body {
                file: body.file.clone(),
                start: body.start,
                end: body.end,
                path_base: base.clone(),
                child_base: base,
                ancestors: body.ancestors.clone(),
            });
        }
    }
    Ok(())
}

/// Resolve plain file sources for `child` (`child.rs` or `child/mod.rs`).
///
/// Refuses ambiguity if both exist. If neither exists and the declaration is cfg-conditional, it is
/// tolerated; otherwise an error is returned. An unreadable candidate fails immediately via
/// [`xingbiao::is_regular_file`].
///
/// A source's `path_base` is the directory of the path it is opened by, as rustc resolves a `#[path]` in it, not the
/// directory of a file a symlink there names — measured against rustc 1.96.0, `src/a.rs -> ../elsewhere/a.rs` holding
/// `#[path = "x.rs"]` reads `src/x.rs`. The canonical path is kept for the cycle check alone.
fn resolve_plain_sources(
    child: &str,
    child_path: &str,
    plain: Vec<PlainSource>,
    src_dir: &Path,
    files_literal: &HashSet<&PathBuf>,
    root_relative: Option<&Path>,
    graph: &mut GraphSources,
) -> Result<bool, String> {
    let mut already_sourced = HashSet::new();
    let mut any_structural_match = false;
    for plain_source in plain {
        let PlainSource {
            base,
            ancestors: source_ancestors,
            is_cfg_conditional,
        } = plain_source;
        let flat = base.join(format!("{child}.rs"));
        let nested = base.join(child).join("mod.rs");
        let flat_present = xingbiao::is_regular_file(&flat)?;
        let nested_present = xingbiao::is_regular_file(&nested)?;
        if flat_present && nested_present {
            return Err(format!(
                "module '{child_path}' resolves to both '{}' and '{}' — a plain \
                 `mod {child}` must be backed by exactly one file",
                flat.display(),
                nested.display()
            ));
        }
        if !flat_present && !nested_present {
            if is_cfg_conditional {
                continue;
            }
            return Err(format!(
                "module '{child_path}' is declared (`mod {child};`) but its source \
                 file could not be located (expected '{}' or '{}')",
                flat.display(),
                nested.display()
            ));
        }
        for (candidate, present) in [(flat, flat_present), (nested, nested_present)] {
            if !present {
                continue;
            }
            let canon = xingbiao::canonicalize_or_fail(&candidate)?;
            if !already_sourced.insert(canon.clone()) {
                continue;
            }
            if source_ancestors.contains(&canon) {
                return Err(format!(
                    "module '{child_path}' resolves to '{}', which cycles back to an already-open source file",
                    candidate.display()
                ));
            }
            let structurally_matches = files_literal.contains(&candidate)
                && candidate
                    .strip_prefix(src_dir)
                    .ok()
                    .is_some_and(|relative| module_path_of(relative, root_relative) == child_path);
            if structurally_matches {
                any_structural_match = true;
            } else {
                graph
                    .remapped
                    .push((candidate.clone(), child_path.to_string()));
            }
            let own_dir = candidate
                .parent()
                .map(Path::to_path_buf)
                .unwrap_or_else(|| base.clone());
            let new_child_base = base.join(child);
            let mut ancestors = source_ancestors.clone();
            ancestors.insert(canon);
            graph
                .by_module
                .entry(child_path.to_string())
                .or_default()
                .push(ScanSource::File {
                    file: candidate,
                    path_base: own_dir,
                    child_base: new_child_base,
                    ancestors,
                });
        }
    }
    let plain_file_resolved = !already_sourced.is_empty();
    if plain_file_resolved && !any_structural_match {
        graph.remap_shadowed.insert(child_path.to_string());
    }
    Ok(plain_file_resolved)
}

enum RemapKind {
    Direct,
    Conditional,
}

/// The remapped file's `path_base` is the directory of the path it is opened by, not of a file a symlink there names,
/// as [`resolve_plain_sources`] states.
fn register_remapped_source(
    child_path: &str,
    target: PathBuf,
    base: PathBuf,
    target_ancestors: HashSet<PathBuf>,
    kind: RemapKind,
    graph: &mut GraphSources,
) -> Result<(), String> {
    let canon = xingbiao::canonicalize_or_fail(&target)?;
    if target_ancestors.contains(&canon) {
        let attribute = match kind {
            RemapKind::Direct => "#[path]",
            RemapKind::Conditional => "#[cfg_attr(..., path = ...)]",
        };
        return Err(format!(
            "module '{child_path}' is remapped by {attribute} to '{}', which cycles back to an already-open source file",
            target.display()
        ));
    }
    graph
        .remapped
        .push((target.clone(), child_path.to_string()));
    let own_dir = target.parent().map(Path::to_path_buf).unwrap_or(base);
    let mut ancestors = target_ancestors;
    ancestors.insert(canon);
    graph
        .by_module
        .entry(child_path.to_string())
        .or_default()
        .push(ScanSource::File {
            file: target,
            path_base: own_dir.clone(),
            child_base: own_dir,
            ancestors,
        });
    Ok(())
}

fn resolve_direct_paths(
    child_path: &str,
    seen_plain_file: bool,
    direct: Vec<DirectPathSource>,
    graph: &mut GraphSources,
) -> Result<(), String> {
    if direct.is_empty() {
        return Ok(());
    }
    if !seen_plain_file {
        graph.remap_shadowed.insert(child_path.to_string());
    }
    for direct_source in direct {
        let DirectPathSource {
            relative,
            base,
            ancestors: target_ancestors,
            is_cfg_conditional,
        } = direct_source;
        let target = base.join(&relative);
        if !xingbiao::is_regular_file(&target)? {
            if is_cfg_conditional {
                continue;
            }
            return Err(format!(
                "module '{child_path}' is remapped by #[path = \"{}\"] to a file that does not exist: '{}'",
                relative.display(),
                target.display()
            ));
        }
        register_remapped_source(
            child_path,
            target,
            base,
            target_ancestors,
            RemapKind::Direct,
            graph,
        )?;
    }
    Ok(())
}

fn resolve_conditional_paths(
    child_path: &str,
    seen_plain_file: bool,
    conditional: Vec<ConditionalPathSource>,
    graph: &mut GraphSources,
) -> Result<(), String> {
    if conditional.is_empty() {
        return Ok(());
    }
    if !seen_plain_file {
        graph.remap_shadowed.insert(child_path.to_string());
    }
    for conditional_source in conditional {
        let ConditionalPathSource {
            relative,
            base,
            ancestors: target_ancestors,
        } = conditional_source;
        let target = base.join(&relative);
        if !xingbiao::is_regular_file(&target)? {
            continue;
        }
        register_remapped_source(
            child_path,
            target,
            base,
            target_ancestors,
            RemapKind::Conditional,
            graph,
        )?;
    }
    Ok(())
}

/// Index `files` by their path-derived module path — used ONLY to discover the crate root's own
/// file(s) below (`by_module.get("crate")`), the one place a module has no declaring source of
/// its own to probe a directory from. Every OTHER module's plain children are resolved by a live
/// per-source directory probe (`resolve_plain_sources`), not this index: a structural,
/// module-path-keyed lookup cannot tell which of a module's several sources (e.g.
/// mutually-exclusive `#[cfg]` arms) actually declared a given child, and — since a file can
/// physically coincide with a module's naive structural path even when that module was reached
/// through an unrelated `#[path]` remap — it can also phantom-match a stray, uncompiled file.
fn index_files_by_module<'a>(
    files: &'a [PathBuf],
    src_dir: &Path,
    root_relative: Option<&Path>,
) -> std::collections::BTreeMap<String, Vec<&'a PathBuf>> {
    let mut by_module: std::collections::BTreeMap<String, Vec<&PathBuf>> = Default::default();
    for file in files {
        if let Ok(relative) = file.strip_prefix(src_dir) {
            by_module
                .entry(module_path_of(relative, root_relative))
                .or_default()
                .push(file);
        }
    }
    by_module
}

/// The crate root's own initial scan sources, from its indexed file(s) — the one module with no
/// declaring source of its own to probe a directory from (every other module's file discovery
/// goes through a live per-source directory probe instead; see [`index_files_by_module`]'s doc).
fn root_scan_sources(root_files: &[&PathBuf], src_dir: &Path) -> Result<Vec<ScanSource>, String> {
    let mut root_ancestors = HashSet::new();
    for f in root_files {
        root_ancestors.insert(xingbiao::canonicalize_or_fail(f)?);
    }
    Ok(root_files
        .iter()
        .map(|f| ScanSource::File {
            file: (*f).clone(),
            path_base: src_dir.to_path_buf(),
            child_base: src_dir.to_path_buf(),
            ancestors: root_ancestors.clone(),
        })
        .collect())
}

/// Resolves the set of module paths reachable from the crate root via `mod` declarations.
/// Returns `(reachable, inline_only, remapped, remap_shadowed)`.
/// Unreachable orphan files are excluded; unreadable reachable files return a scan error.
///
/// File lookup is indexed by literal path to check walk presence without symlink canonicalization
/// aliasing. Every declared source for a child is additive and cfg-blind, carrying its own
/// source-local ancestor set to prevent false cycle reports across mutually-exclusive cfg arms.
/// An inline body's own declarations are re-scanned even if sibling plain-file or remapped sources
/// exist. `inline_only` excludes stray same-named conventional files only when no plain file
/// actually resolved.
#[allow(clippy::type_complexity)]
pub(crate) fn reachable_modules(
    src_dir: &Path,
    files: &[PathBuf],
    root_relative: Option<&Path>,
    edition: Edition,
) -> Result<
    (
        std::collections::BTreeSet<String>,
        std::collections::BTreeMap<String, PathBuf>,
        Vec<(PathBuf, String)>,
        std::collections::BTreeSet<String>,
    ),
    String,
> {
    let by_module = index_files_by_module(files, src_dir, root_relative);
    let files_literal: HashSet<&PathBuf> = files.iter().collect();

    let mut reachable = std::collections::BTreeSet::new();
    let mut inline_only = std::collections::BTreeMap::new();
    let mut graph = GraphSources::default();
    reachable.insert("crate".to_string());
    if let Some(root_files) = by_module.get("crate") {
        graph
            .by_module
            .insert("crate".to_string(), root_scan_sources(root_files, src_dir)?);
    }
    let texts: HashMap<&PathBuf, OnceCell<String>> =
        files.iter().map(|file| (file, OnceCell::new())).collect();
    let mut trees = HashMap::new();
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        let Some(scan_sources) = graph.by_module.get(&module).cloned() else {
            continue;
        };
        let children = collect_children(&scan_sources, edition, &texts, &mut trees)?;
        for (child, child_sources) in children {
            let ChildSources {
                seen_plain_file,
                bodies,
                plain,
                direct,
                conditional,
            } = child_sources;
            let child_path = format!("{module}::{child}");
            let inline_extraction_base = bodies.first().map(|b| b.base.clone());
            if !bodies.is_empty() {
                register_inline_sources(&child, &child_path, bodies, &mut graph)?;
            }
            let plain_file_resolved = if seen_plain_file {
                resolve_plain_sources(
                    &child,
                    &child_path,
                    plain,
                    src_dir,
                    &files_literal,
                    root_relative,
                    &mut graph,
                )?
            } else {
                false
            };
            if let Some(base) = inline_extraction_base.filter(|_| !plain_file_resolved) {
                inline_only.insert(child_path.clone(), base.join(format!("{child}.rs")));
            }
            resolve_direct_paths(&child_path, seen_plain_file, direct, &mut graph)?;
            resolve_conditional_paths(&child_path, seen_plain_file, conditional, &mut graph)?;
            if reachable.insert(child_path.clone()) {
                queue.push(child_path);
            }
        }
    }
    Ok((reachable, inline_only, graph.remapped, graph.remap_shadowed))
}
