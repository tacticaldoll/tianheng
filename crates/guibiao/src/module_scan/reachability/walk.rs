//! Reachability graph traversal and physical-source resolution.

use super::super::lexer::{clean_with_positions, read_path_string};
use super::declarations::declared_modules_in;
use super::paths::module_path_of;
use std::collections::{BTreeMap, BTreeSet, HashSet};
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
    text: String,
    cleaned: String,
    positions: Vec<usize>,
    range: Range<usize>,
    path_base: PathBuf,
    child_base: PathBuf,
    ancestors: HashSet<PathBuf>,
}

impl ScanSource {
    fn load(&self) -> Result<LoadedSource, String> {
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
            } => (file, Some(*start..*end), path_base, child_base, ancestors),
        };
        let text = std::fs::read_to_string(file)
            .map_err(|err| format!("cannot read source file '{}': {err}", file.display()))?;
        let (cleaned, positions) = clean_with_positions(&text);
        let range = range.unwrap_or(0..cleaned.len());
        Ok(LoadedSource {
            file: file.clone(),
            text,
            cleaned,
            positions,
            range,
            path_base: path_base.clone(),
            child_base: child_base.clone(),
            ancestors: ancestors.clone(),
        })
    }
}

struct InlineBody {
    file: PathBuf,
    start: usize,
    end: usize,
    base: PathBuf,
    /// A **direct** `#[path]`'s base: it replaces the conventional one outright.
    relocated_base: Option<PathBuf>,
    /// Every `cfg_attr(…, path = …)` base this declaration carries, when no direct attribute
    /// overrides them. Candidates, not the base — see [`register_inline_sources`].
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

#[derive(Default)]
struct ChildSources {
    seen_inline: bool,
    seen_plain_file: bool,
    bodies: Vec<InlineBody>,
    plain: Vec<PlainSource>,
    direct: Vec<DirectPathSource>,
    conditional: Vec<ConditionalPathSource>,
}

fn collect_children(scan_sources: &[ScanSource]) -> Result<BTreeMap<String, ChildSources>, String> {
    let mut children: BTreeMap<String, ChildSources> = Default::default();
    for source in scan_sources {
        let loaded = source.load()?;
        for declared in declared_modules_in(&loaded.cleaned, loaded.range) {
            let child_sources = children.entry(declared.name.clone()).or_default();
            if declared.is_inline {
                child_sources.seen_inline = true;
                let base_at = |eq_cleaned: usize| -> Option<PathBuf> {
                    let &orig_eq = loaded.positions.get(eq_cleaned)?;
                    let rel =
                        read_path_string(loaded.text.as_bytes(), orig_eq + 1, loaded.text.len())?;
                    Some(loaded.path_base.join(rel))
                };
                // A **direct** `#[path]` takes precedence over any sibling `cfg_attr` paths and
                // relocates the base outright — one base, replacing the conventional one.
                let relocated_base = declared.direct_path_eq.and_then(base_at);
                // With no direct attribute, every `cfg_attr(…, path = …)` target is a **candidate**
                // base rather than the base: the scanner is cfg-blind and cannot know which arm a
                // given build compiles, so preferring one would silently drop the other's children.
                // Collected here and unioned in `register_inline_sources`, which owns the
                // existence rule.
                let candidate_bases: Vec<PathBuf> = match relocated_base {
                    Some(_) => Vec::new(),
                    None => declared
                        .conditional_path_eqs
                        .iter()
                        .copied()
                        .filter_map(base_at)
                        .collect(),
                };
                if let Some((start, end)) = declared.body {
                    child_sources.bodies.push(InlineBody {
                        file: loaded.file.clone(),
                        start,
                        end,
                        base: loaded.child_base.clone(),
                        relocated_base,
                        candidate_bases,
                        ancestors: loaded.ancestors.clone(),
                    });
                }
                continue;
            }
            // Resolve every `cfg_attr(path)` candidate THIS declaration carries before deciding
            // below whether its plain conventional file is required: `cfg_attr` never removes
            // the item, but a candidate that physically exists is proof SOME real configuration
            // compiles this declaration through that remap rather than the conventional file —
            // the same "might legitimately be absent on this build" signal a bare `#[cfg]` or a
            // `cfg_if!` arm already carries (`declared.is_cfg_conditional`), just discovered from
            // the filesystem instead of the source text. Neither candidate existing (every
            // `cfg_attr(path)` target absent, same as none declared at all) leaves the
            // conventional-file requirement exactly as strict as it already is — this only adds a
            // tolerance, never removes the existing one. 渾儀/漏刻 already apply the identical rule
            // to their own crate-wide walk (三儀 ⊥ 三儀: the same rule, not the same function).
            let mut resolved_conditional = Vec::new();
            for &eq_cleaned in &declared.conditional_path_eqs {
                if let Some(&orig_eq) = loaded.positions.get(eq_cleaned) {
                    if let Some(rel) =
                        read_path_string(loaded.text.as_bytes(), orig_eq + 1, loaded.text.len())
                    {
                        let candidate_target = loaded.path_base.join(&rel);
                        // An unreadable target is not an absent one; registering neither would drop
                        // this source and everything it reaches, in silence.
                        if xingbiao::is_regular_file(&candidate_target)? {
                            resolved_conditional.push(ConditionalPathSource {
                                relative: PathBuf::from(rel),
                                base: loaded.path_base.clone(),
                                ancestors: loaded.ancestors.clone(),
                            });
                        }
                    }
                }
            }
            let has_backing_conditional_target = !resolved_conditional.is_empty();
            child_sources.conditional.extend(resolved_conditional);

            if let Some(eq_cleaned) = declared.direct_path_eq {
                if let Some(&orig_eq) = loaded.positions.get(eq_cleaned) {
                    if let Some(rel) =
                        read_path_string(loaded.text.as_bytes(), orig_eq + 1, loaded.text.len())
                    {
                        child_sources.direct.push(DirectPathSource {
                            relative: PathBuf::from(rel),
                            base: loaded.path_base.clone(),
                            ancestors: loaded.ancestors.clone(),
                            is_cfg_conditional: declared.is_cfg_conditional,
                        });
                    }
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
/// A **direct** `#[path]` relocates that base outright — one source, as before. A `cfg_attr`-wrapped
/// one names a base per platform predicate, so every target is a **candidate**, unioned with the
/// conventional directory: the scanner does not evaluate `cfg` and cannot know which arm a build
/// compiles, so preferring one would silently drop the children beneath the other (the false negative
/// the core contract forbids). A candidate is descended only when it **exists as a directory** —
/// recursing into an absent one would spuriously fail loud on the body's other, unrelated nested
/// items solely because one platform's directory is missing, even when another candidate already
/// backs them. When no candidate exists at all, the conventional base is descended anyway, so a
/// nested reference genuinely broken on every platform still fails loud exactly as it did before this
/// tolerance existed.
///
/// This is 漏刻's own already-stated rule for the identical shape, implemented independently here
/// (三儀 ⊥ 三儀: the same rule, not the same function), so the two dimensions cannot disagree about
/// what rustc compiles.
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
            Some(base) => vec![base.clone()],
            None if body.candidate_bases.is_empty() => vec![conventional],
            None => {
                // Which candidate bases exist, refusing rather than dropping one this reader could
                // not stat: a base filtered out here takes its whole subtree with it.
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
        // An unreadable candidate is not an absent one: `is_file` answers `false` for both, and the
        // cfg tolerance below is what an absence is owed, so a target this reader could not stat was
        // swallowed with whatever its subtree holds. `xingbiao` owns the criterion for all three
        // dimensions.
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
            let own_dir = canon
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
    let own_dir = canon.parent().map(Path::to_path_buf).unwrap_or(base);
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
    let by_module = index_files_by_module(files, src_dir, root_relative);
    // Indexed by literal path to check walk presence without symlink canonicalization aliasing.
    let files_literal: HashSet<&PathBuf> = files.iter().collect();

    let mut reachable = std::collections::BTreeSet::new();
    let mut inline_only = std::collections::BTreeSet::new();
    let mut graph = GraphSources::default();
    // A module path whose ONLY file-form source is an unconditional `#[path]` remap (no plain
    // sibling declaration under any `#[cfg]` arm) — the case where a same-named conventional file
    // really is the orphan-shadow hazard `governed_files` must exclude. When a plain-file sibling
    // ALSO exists (the per-platform shim pattern), that file is real and must NOT be excluded, so
    // this is tracked separately from mere membership in `remapped`.
    reachable.insert("crate".to_string());
    if let Some(root_files) = by_module.get("crate") {
        graph
            .by_module
            .insert("crate".to_string(), root_scan_sources(root_files, src_dir)?);
    }
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        let Some(scan_sources) = graph.by_module.get(&module).cloned() else {
            continue; // no file backs this module and it declared no inline body; nothing to read
        };
        // Classify each child across this module's source(s) before descending. All declarations
        // remain additive and cfg-blind, while their ancestor sets stay source-local.
        let children = collect_children(&scan_sources)?;
        for (child, child_sources) in children {
            let ChildSources {
                seen_inline,
                seen_plain_file,
                bodies,
                plain,
                direct,
                conditional,
            } = child_sources;
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
                register_inline_sources(&child, &child_path, bodies, &mut graph)?;
            }
            // Whether at least one plain declaration for this child actually resolved to a real
            // file — NOT merely whether one was declared: a bare-`#[cfg]`-tolerated declaration
            // (tolerated below) can be declared yet resolve to nothing, and that must not count as
            // "a real plain file exists" for the `inline_only` decision. An inline arm paired with
            // an entirely-tolerated-away plain arm must still be recognized as `inline_only`
            // (reporting `inline_module_target_error` rather than `unknown_module_error`). Defaults
            // to `false` when no plain declaration exists at all.
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
            resolve_direct_paths(&child_path, seen_plain_file, direct, &mut graph)?;
            resolve_conditional_paths(&child_path, seen_plain_file, conditional, &mut graph)?;
            if reachable.insert(child_path.clone()) {
                queue.push(child_path);
            }
        }
    }
    Ok((reachable, inline_only, graph.remapped, graph.remap_shadowed))
}
