//! Reachability graph traversal and physical-source resolution.

use super::super::item_head::{BlockModule, block_modules};
use super::super::source_texts::SourceTexts;
use super::super::token_tree::{Edition, TokenTree};
use super::declarations::{DeclaredModule, block_path_modules, declared_modules_in};
use super::paths::module_path_of;
use std::cell::OnceCell;
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::rc::Rc;

/// A physical file or inline body whose top-level declarations feed the graph walk.
///
/// The bases are deliberately distinct: `path_base` resolves attributes written in this
/// source, while `child_base` resolves its conventional file-form children. Ancestors belong
/// to this exact source path and must never be merged across cfg-blind sibling sources.
#[derive(Clone)]
enum ScanSource {
    /// A whole file, read from its first token to its last.
    File {
        /// The path the file is opened by, not its canonical form.
        file: PathBuf,
        /// The directory a path attribute written in the file resolves against: that of the path it is opened by, and
        /// the source directory for the crate root.
        path_base: PathBuf,
        /// The directory the file's conventional children are probed under: its module's directory, or `path_base`
        /// for a file a path attribute opened or for the crate root.
        child_base: PathBuf,
        /// What the file inherits from the declarations that opened it.
        lineage: Lineage,
    },
    /// An inline module's body, by the token indices of its `{` and `}` in the file's tree.
    Body {
        /// The file whose token tree holds the body.
        file: PathBuf,
        /// The token index of the body's `{`; the source starts just past it.
        start: usize,
        /// The token index of the body's `}`, which ends the source.
        end: usize,
        /// One base the body's file-form children may resolve from, the same directory as `child_base`.
        path_base: PathBuf,
        /// The same directory as `path_base`: a body registered under several bases is one source per base.
        child_base: PathBuf,
        /// What the body inherits from the declarations that opened it.
        lineage: Lineage,
    },
}

/// What a source inherits from the declarations that opened it: the files already open along them, which a declaration
/// cycling back to one of them is refused for, and whether one of them may be compiled out — rustc loads nothing
/// beneath a module a `cfg` removes, so a `mod` of that source with no file is as tolerable as one carrying the `cfg`
/// itself. Measured against rustc 1.96.0, edition 2021: `#[cfg(any())] mod o;` over an `o.rs` holding `mod i;` builds
/// with no `i.rs`. Per source, since a module's several cfg-blind sources are opened by different declarations.
#[derive(Clone, Default)]
struct Lineage {
    /// The canonical path of every file opened along the declarations leading here, this source's own included.
    files: HashSet<PathBuf>,
    /// Whether some declaration along them may be compiled out, which tolerates a missing file beneath it.
    compiled_out: bool,
    /// Whether the source is a module a block declares, or one inside it that no path attribute gave a directory of its
    /// own, where rustc refuses a file-form `mod` with no path attribute: `fn f() { mod k { mod m; } }` is refused with "cannot declare a file module inside a block
    /// unless it has a path attribute", measured against rustc 1.96.0, edition 2021. A file such a module's `#[path]`
    /// names is a module file of its own, so the walk into it starts outside every block.
    in_block: bool,
}

impl Lineage {
    /// This lineage continued through a declaration that is itself compiled out where `compiled_out` holds.
    fn under(&self, compiled_out: bool) -> Lineage {
        Lineage {
            files: self.files.clone(),
            compiled_out: self.compiled_out || compiled_out,
            in_block: false,
        }
    }

    /// This lineage continued into an inline body, inside a block where `in_block` holds: one a block declares, or
    /// one inside it, unless the body's module carries a path attribute, which gives it a directory of its own —
    /// `fn f() { #[path = "d"] mod k { pub mod m; } }` reads `d/m.rs`, and so does the same with
    /// `#[cfg_attr(all(), path = "d")]`, measured against rustc 1.96.0, edition 2021.
    fn with_in_block(&self, in_block: bool) -> Lineage {
        Lineage {
            in_block,
            ..self.clone()
        }
    }
}

/// A [`ScanSource`] with its file and inline-body forms read as one shape.
struct LoadedSource {
    /// The file whose token tree holds this source.
    file: PathBuf,
    /// The token range whose top-level declarations this source holds: the whole file, or an inline body between
    /// its braces. `None` is the whole file, whose length is known only once it is lexed.
    range: Option<Range<usize>>,
    /// The directory a path attribute written in this source resolves against.
    path_base: PathBuf,
    /// The directory this source's conventional file-form children are probed under.
    child_base: PathBuf,
    /// What this source inherits from the declarations that opened it.
    lineage: Lineage,
}

impl ScanSource {
    /// This source in the one shape both forms share, an inline body's range lying strictly between its braces.
    fn load(&self) -> LoadedSource {
        let (file, range, path_base, child_base, lineage) = match self {
            Self::File {
                file,
                path_base,
                child_base,
                lineage,
            } => (file, None, path_base, child_base, lineage),
            Self::Body {
                file,
                start,
                end,
                path_base,
                child_base,
                lineage,
            } => (file, Some(*start + 1..*end), path_base, child_base, lineage),
        };
        LoadedSource {
            file: file.clone(),
            range,
            path_base: path_base.clone(),
            child_base: child_base.clone(),
            lineage: lineage.clone(),
        }
    }
}

/// An inline `mod name { … }` body, with what [`register_inline_sources`] decides the bases of its file-form children
/// from.
struct InlineBody {
    /// The file whose token tree holds the body.
    file: PathBuf,
    /// The token index of the body's `{`.
    start: usize,
    /// The token index of the body's `}`.
    end: usize,
    /// The directory the conventional base is taken under: the declaring source's `child_base`, or its `path_base`
    /// for a module a block declares.
    base: PathBuf,
    /// The directory the conventional base takes under `base`: the module's own name, which a block's `{block}::`
    /// naming is no part of.
    directory: String,
    /// A **direct** `#[path]`'s base: it replaces the conventional one outright.
    relocated_base: Option<PathBuf>,
    /// Every `cfg_attr(…, path = …)` base this declaration may compile, beside a direct one as without it: rustc takes
    /// the first path attribute written, which a `cfg_attr` written before a direct one is where its predicate holds,
    /// and one written after it never is, so the attribute reader keeps only the first kind. Candidates, not the
    /// base — see [`register_inline_sources`].
    candidate_bases: Vec<PathBuf>,
    /// Whether the conventional base is one rustc may compile beside the candidates: not inside a block, where an
    /// inline module compiled without its path attribute holds no file-form `mod` rustc accepts, so only the paths
    /// its `cfg_attr`s name are bases.
    conventional_compiles: bool,
    /// The lineage every source registered for this body carries, its `in_block` already decided for the body.
    lineage: Lineage,
}

/// A file-form declaration's source, by the conventional directory it resolves from. `declared_in` is the file whose
/// `mod` declares it, which each refusal of the source names, so a module several sources declare says which line to
/// change.
struct PlainSource {
    /// The directory `child.rs` and `child/mod.rs` are probed in: the declaring source's `child_base`.
    base: PathBuf,
    /// The file holding the `mod` declaration.
    declared_in: PathBuf,
    /// The declaring source's lineage, continued through this declaration.
    lineage: Lineage,
    /// Whether finding neither conventional file is tolerated: the lineage may be compiled out, or one of the
    /// declaration's `cfg_attr` path targets exists.
    is_cfg_conditional: bool,
}

/// A direct `#[path]` declaration's source; `declared_in` as [`PlainSource`] has it.
struct DirectPathSource {
    /// The `#[path]` value as written, resolved against `base`.
    relative: PathBuf,
    /// The declaring source's `path_base`.
    base: PathBuf,
    /// The file holding the `mod` declaration.
    declared_in: PathBuf,
    /// The declaring source's lineage, continued through this declaration.
    lineage: Lineage,
    /// Whether a missing target is tolerated: the lineage may be compiled out, or a `cfg_attr` path written before
    /// the direct one names a file that exists.
    is_cfg_conditional: bool,
}

/// A `cfg_attr(…, path = …)` declaration's source whose target exists; `declared_in` as [`PlainSource`] has it.
struct ConditionalPathSource {
    /// The `cfg_attr` path value as written, resolved against `base`.
    relative: PathBuf,
    /// The declaring source's `path_base`.
    base: PathBuf,
    /// The file holding the `mod` declaration.
    declared_in: PathBuf,
    /// The declaring source's lineage, continued through this declaration.
    lineage: Lineage,
}

/// What one child's declarations across the scanned sources say about where its source is.
///
/// `bodies` is non-empty exactly when the child is declared inline somewhere, since every inline declaration
/// carries a body. An inline-only child's suggested extraction path is the first declaring source's base, in
/// the order the sources were scanned — an example of a path rustc resolves, which the refusal words as one.
#[derive(Default)]
struct ChildSources {
    /// Whether some source declares the child as a file-form `mod` with no path attribute, the condition under which
    /// `plain` is pushed to; a remap of a child without one shadows its structurally located file.
    seen_plain_file: bool,
    /// One per inline declaration of the child.
    bodies: Vec<InlineBody>,
    /// One per file-form declaration with no direct `#[path]`, to be probed for in the conventional directory.
    plain: Vec<PlainSource>,
    /// One per file-form declaration whose direct `#[path]` value is readable.
    direct: Vec<DirectPathSource>,
    /// One per `cfg_attr` path target of a file-form declaration that exists as a regular file.
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
/// Every text comes from `sources`, the evaluation's one reading of each path. A file of the crate is lexed once for
/// the whole walk, its text held in its slot of `texts` and its tree in `trees`: a file holding many inline modules is
/// the source of each of them, and each reads its own range of the one token tree. A file the crate's file list does
/// not hold is lexed where it is met, from the same reading.
fn collect_children<'t>(
    module: &str,
    scan_sources: &[ScanSource],
    edition: Edition,
    sources: &SourceTexts,
    texts: &'t HashMap<&PathBuf, OnceCell<Rc<str>>>,
    trees: &mut HashMap<PathBuf, (TokenTree<'t>, Vec<BlockModule>)>,
) -> Result<BTreeMap<String, ChildSources>, String> {
    let mut children: BTreeMap<String, ChildSources> = Default::default();
    for source in scan_sources {
        let loaded = source.load();
        let read = || {
            sources.text(&loaded.file).map_err(|err| {
                format!("cannot read source file '{}': {err}", loaded.file.display())
            })
        };
        match texts.get(&loaded.file) {
            Some(slot) => {
                if !trees.contains_key(&loaded.file) {
                    let text = match slot.get() {
                        Some(text) => text,
                        None => {
                            let text = read()?;
                            slot.get_or_init(|| text)
                        }
                    };
                    let tree = TokenTree::lex(text, edition);
                    let blocks = block_modules(&tree);
                    trees.insert(loaded.file.clone(), (tree, blocks));
                }
                let (tree, blocks) = &trees[&loaded.file];
                record_declarations(module, &loaded, tree, blocks, &mut children)?;
            }
            None => {
                let text = read()?;
                let tree = TokenTree::lex(&text, edition);
                let blocks = block_modules(&tree);
                record_declarations(module, &loaded, &tree, &blocks, &mut children)?;
            }
        }
    }
    Ok(children)
}

/// Record the child modules one loaded source declares, each beside what the other sources say about it.
fn record_declarations(
    module: &str,
    loaded: &LoadedSource,
    tree: &TokenTree<'_>,
    blocks: &[BlockModule],
    children: &mut BTreeMap<String, ChildSources>,
) -> Result<(), String> {
    let range = loaded.range.clone().unwrap_or(0..tree.len());
    for declared in declared_modules_in(tree, range.clone())
        .into_iter()
        .chain(block_path_modules(tree, blocks, range))
    {
        let child_sources = children.entry(declared.name.clone()).or_default();
        match declared.body {
            Some((start, end)) => child_sources
                .bodies
                .push(inline_body(loaded, &declared, start, end)),
            None => record_file_form(module, loaded, &declared, child_sources)?,
        }
    }
    Ok(())
}

/// The scan source an inline `mod name { … }` body opens, with the bases its file-form children resolve from.
fn inline_body(
    loaded: &LoadedSource,
    declared: &DeclaredModule,
    start: usize,
    end: usize,
) -> InlineBody {
    let in_block = loaded.lineage.in_block || declared.declared_in_block;
    let (base, directory) = if declared.declared_in_block {
        let own = declared
            .name
            .rsplit_once("::")
            .map_or(&*declared.name, |(_, own)| own);
        (loaded.path_base.clone(), own.to_string())
    } else {
        (loaded.child_base.clone(), declared.name.clone())
    };
    InlineBody {
        file: loaded.file.clone(),
        start,
        end,
        base,
        directory,
        relocated_base: declared
            .direct_path
            .clone()
            .flatten()
            .map(|rel| loaded.path_base.join(rel)),
        candidate_bases: declared
            .conditional_paths
            .iter()
            .map(|rel| loaded.path_base.join(rel))
            .collect(),
        conventional_compiles: !in_block,
        lineage: loaded.lineage.with_in_block(
            in_block && declared.direct_path.is_none() && declared.conditional_paths.is_empty(),
        ),
    }
}

/// Record a file-form `mod name;` by each source it may resolve from: the `cfg_attr` path targets that exist, a direct
/// `#[path]`, or the conventional directory — refusing one inside a block that no configuration builds.
fn record_file_form(
    module: &str,
    loaded: &LoadedSource,
    declared: &DeclaredModule,
    child_sources: &mut ChildSources,
) -> Result<(), String> {
    let child_lineage = loaded.lineage.under(declared.is_cfg_conditional);
    let compiled_out = child_lineage.compiled_out;
    let mut has_backing_conditional_target = false;
    for rel in &declared.conditional_paths {
        if xingbiao::is_regular_file(&loaded.path_base.join(rel))? {
            has_backing_conditional_target = true;
            child_sources.conditional.push(ConditionalPathSource {
                relative: PathBuf::from(rel),
                base: loaded.path_base.clone(),
                declared_in: loaded.file.clone(),
                lineage: child_lineage.clone(),
            });
        }
    }
    match &declared.direct_path {
        Some(Some(rel)) => child_sources.direct.push(DirectPathSource {
            relative: PathBuf::from(rel),
            base: loaded.path_base.clone(),
            declared_in: loaded.file.clone(),
            lineage: child_lineage,
            is_cfg_conditional: compiled_out
                || (declared.direct_path_is_conditional && has_backing_conditional_target),
        }),
        Some(None) => {}
        None if declared.declared_in_block || loaded.lineage.in_block => {
            if !has_backing_conditional_target && !compiled_out {
                return Err(unbuildable_block_module(module, loaded, declared));
            }
        }
        None => {
            child_sources.seen_plain_file = true;
            child_sources.plain.push(PlainSource {
                base: loaded.child_base.clone(),
                declared_in: loaded.file.clone(),
                lineage: child_lineage,
                is_cfg_conditional: compiled_out || has_backing_conditional_target,
            });
        }
    }
    Ok(())
}

/// The refusal of a file-form `mod` inside a block that no configuration builds: with no path attribute rustc refuses
/// it, and with only `cfg_attr` paths none of which exists, no build has its file.
fn unbuildable_block_module(
    module: &str,
    loaded: &LoadedSource,
    declared: &DeclaredModule,
) -> String {
    let child = format!("{module}::{}", declared.name);
    if declared.conditional_paths.is_empty() {
        format!(
            "module `{child}`, declared in '{}', is a file-form `mod` inside a block with no path \
             attribute, which rustc refuses: a module a block holds has no conventional file",
            loaded.file.display()
        )
    } else {
        format!(
            "module `{child}`, declared in '{}', names no file that exists by any of its \
             `cfg_attr` path attributes, and a file-form `mod` inside a block has no conventional \
             file, so no configuration builds it",
            loaded.file.display()
        )
    }
}

/// The graph the walk accumulates: every module's scan sources, and the remap facts [`reachable_modules`] returns.
#[derive(Default)]
struct GraphSources {
    /// Each module path's scan sources, which its children are read from; a module reached with none has no children
    /// read.
    by_module: BTreeMap<String, Vec<ScanSource>>,
    /// Each file opened somewhere other than its module's structural path — a path attribute's target, or a plain file
    /// off that path — with the module path it is governed as.
    remapped: Vec<(PathBuf, String)>,
    /// Module paths whose structurally located file, where one exists, is not governed as that module: a path
    /// attribute remaps the module and no declaration of it is plain, or every plain file that resolved for it lies off
    /// its structural path.
    remap_shadowed: BTreeSet<String>,
}

/// Register an inline `mod name { … }` body as a scan source, once per base its file-form children
/// may resolve from.
///
/// A **direct** `#[path]` relocates that base, and each `cfg_attr`-wrapped one written before it is a **candidate**
/// read beside it; with a candidate before it, the direct one applies only where the candidate's predicate is false,
/// so it is descended as a candidate is — measured against rustc 1.96.0, edition 2021, on unix:
/// `#[cfg_attr(unix, path = "b")] #[path = "a"] mod m { mod c; }` builds with only `b/c.rs` on disk. With no direct one, a `cfg_attr`-wrapped path names a base per platform predicate, so every target
/// is a candidate, unioned with the conventional directory where that compiles — not inside a block, where a module
/// compiled without its path attribute holds no file-form `mod` rustc accepts: the scanner does not evaluate `cfg` and
/// cannot know which arm a build compiles, so preferring one would silently drop the children beneath the other (the
/// false negative the core contract forbids). A candidate is descended only when it **exists as a directory** — recursing into an absent
/// one would spuriously fail loud on the body's other, unrelated nested items solely because one platform's directory
/// is missing, even when another candidate already backs them. When no candidate exists at all, the conventional base
/// is descended anyway, or inside a block every candidate is, so a nested reference genuinely broken on every platform
/// still fails loud, naming each candidate it found no file under.
///
/// 漏刻 states a rule for the same shape and implements it independently (三儀 ⊥ 三儀: the same rule, not the
/// same function). The two are not yet one answer: 漏刻 still takes the last of several direct `#[path]`
/// attributes, where rustc and this walk take the first, which `BACKLOG.md` tracks with the inline-base
/// question under the 渾儀 and 漏刻 WATCH entry.
fn register_inline_sources(
    child_path: &str,
    bodies: Vec<InlineBody>,
    graph: &mut GraphSources,
) -> Result<(), String> {
    let sources = graph.by_module.entry(child_path.to_string()).or_default();
    for body in bodies {
        let conventional = body.base.join(&body.directory);
        let bases: Vec<PathBuf> = match &body.relocated_base {
            Some(base) if body.candidate_bases.is_empty() => vec![base.clone()],
            Some(base) => {
                let mut present: Vec<PathBuf> = Vec::new();
                for candidate in std::iter::once(base).chain(&body.candidate_bases) {
                    if xingbiao::is_directory(candidate)? && !present.contains(candidate) {
                        present.push(candidate.clone());
                    }
                }
                if present.is_empty() {
                    vec![base.clone()]
                } else {
                    present
                }
            }
            None if body.candidate_bases.is_empty() => vec![conventional],
            None => {
                let mut present: Vec<PathBuf> = Vec::new();
                for base in body
                    .candidate_bases
                    .iter()
                    .cloned()
                    .chain(body.conventional_compiles.then(|| conventional.clone()))
                {
                    if xingbiao::is_directory(&base)? {
                        present.push(base);
                    }
                }
                present.sort();
                present.dedup();
                match (present.is_empty(), body.conventional_compiles) {
                    (false, _) => present,
                    (true, true) => vec![conventional],
                    (true, false) => body.candidate_bases.clone(),
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
                lineage: body.lineage.clone(),
            });
        }
    }
    Ok(())
}

/// Resolve plain file sources for `child` (`child.rs` or `child/mod.rs`).
///
/// Refuses ambiguity if both exist. If neither exists and the declaration is cfg-conditional, it is
/// tolerated; otherwise an error is returned, naming every declaring source that found neither file, so a module
/// several sources declare lists each place to repair. An unreadable candidate fails immediately via
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
    let mut unlocated = Vec::new();
    for plain_source in plain {
        let PlainSource {
            base,
            declared_in,
            lineage: source_lineage,
            is_cfg_conditional,
        } = plain_source;
        let flat = base.join(format!("{child}.rs"));
        let nested = base.join(child).join("mod.rs");
        let flat_present = xingbiao::is_regular_file(&flat)?;
        let nested_present = xingbiao::is_regular_file(&nested)?;
        if flat_present && nested_present {
            return Err(format!(
                "module '{child_path}', declared in '{}', resolves to both '{}' and '{}' — a plain \
                 `mod {child}` must be backed by exactly one file",
                declared_in.display(),
                flat.display(),
                nested.display()
            ));
        }
        if !flat_present && !nested_present {
            if !is_cfg_conditional {
                unlocated.push(format!(
                    "declared in '{}', expected '{}' or '{}'",
                    declared_in.display(),
                    flat.display(),
                    nested.display()
                ));
            }
            continue;
        }
        for (candidate, present) in [(flat, flat_present), (nested, nested_present)] {
            if !present {
                continue;
            }
            let canon = xingbiao::canonicalize_or_fail(&candidate)?;
            if !already_sourced.insert(canon.clone()) {
                continue;
            }
            if source_lineage.files.contains(&canon) {
                return Err(format!(
                    "module '{child_path}', declared in '{}', resolves to '{}', which cycles back to an \
                     already-open source file",
                    declared_in.display(),
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
            let mut lineage = source_lineage.clone();
            lineage.files.insert(canon);
            graph
                .by_module
                .entry(child_path.to_string())
                .or_default()
                .push(ScanSource::File {
                    file: candidate,
                    path_base: own_dir,
                    child_base: new_child_base,
                    lineage,
                });
        }
    }
    if !unlocated.is_empty() {
        return Err(format!(
            "module '{child_path}' is declared (`mod {child};`) but its source file could not be located ({})",
            unlocated.join("; ")
        ));
    }
    let plain_file_resolved = !already_sourced.is_empty();
    if plain_file_resolved && !any_structural_match {
        graph.remap_shadowed.insert(child_path.to_string());
    }
    Ok(plain_file_resolved)
}

/// Which path attribute remapped a source, so a cycle refusal names the attribute to change.
enum RemapKind {
    /// An unconditional `#[path = …]`.
    Direct,
    /// A `cfg_attr(…, path = …)`.
    Conditional,
}

/// The remapped file's `path_base` is the directory of the path it is opened by, not of a file a symlink there names,
/// as [`resolve_plain_sources`] states.
fn register_remapped_source(
    child_path: &str,
    target: PathBuf,
    base: PathBuf,
    declared_in: &Path,
    target_lineage: Lineage,
    kind: RemapKind,
    graph: &mut GraphSources,
) -> Result<(), String> {
    let canon = xingbiao::canonicalize_or_fail(&target)?;
    if target_lineage.files.contains(&canon) {
        let attribute = match kind {
            RemapKind::Direct => "#[path]",
            RemapKind::Conditional => "#[cfg_attr(..., path = ...)]",
        };
        return Err(format!(
            "module '{child_path}', declared in '{}', is remapped by {attribute} to '{}', which cycles back to an \
             already-open source file",
            declared_in.display(),
            target.display()
        ));
    }
    graph
        .remapped
        .push((target.clone(), child_path.to_string()));
    let own_dir = target.parent().map(Path::to_path_buf).unwrap_or(base);
    let mut lineage = target_lineage;
    lineage.files.insert(canon);
    graph
        .by_module
        .entry(child_path.to_string())
        .or_default()
        .push(ScanSource::File {
            file: target,
            path_base: own_dir.clone(),
            child_base: own_dir,
            lineage,
        });
    Ok(())
}

/// Register each direct `#[path]` target of `child_path` as a source, refusing a missing target unless its
/// declaration is cfg-conditional. With no plain declaration of the child beside them, its structurally located file
/// is marked shadowed.
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
            declared_in,
            lineage: target_lineage,
            is_cfg_conditional,
        } = direct_source;
        let target = base.join(&relative);
        if !xingbiao::is_regular_file(&target)? {
            if is_cfg_conditional {
                continue;
            }
            return Err(format!(
                "module '{child_path}', declared in '{}', is remapped by #[path = \"{}\"] to a file that does not \
                 exist: '{}'",
                declared_in.display(),
                relative.display(),
                target.display()
            ));
        }
        register_remapped_source(
            child_path,
            target,
            base,
            &declared_in,
            target_lineage,
            RemapKind::Direct,
            graph,
        )?;
    }
    Ok(())
}

/// Register each `cfg_attr` path target of `child_path` that exists as a source, passing over one that does not.
/// With no plain declaration of the child beside them, its structurally located file is marked shadowed.
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
            declared_in,
            lineage: target_lineage,
        } = conditional_source;
        let target = base.join(&relative);
        if !xingbiao::is_regular_file(&target)? {
            continue;
        }
        register_remapped_source(
            child_path,
            target,
            base,
            &declared_in,
            target_lineage,
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
    let mut root_lineage = Lineage::default();
    for f in root_files {
        root_lineage
            .files
            .insert(xingbiao::canonicalize_or_fail(f)?);
    }
    Ok(root_files
        .iter()
        .map(|f| ScanSource::File {
            file: (*f).clone(),
            path_base: src_dir.to_path_buf(),
            child_base: src_dir.to_path_buf(),
            lineage: root_lineage.clone(),
        })
        .collect())
}

/// Resolves the set of module paths reachable from the crate root via `mod` declarations.
/// Returns `(reachable, inline_only, remapped, remap_shadowed)`.
/// Unreachable orphan files are excluded; unreadable reachable files return a scan error.
/// Every source is read through `sources`, so a path the evaluation has already read is not read again.
///
/// File lookup is indexed by literal path to check walk presence without symlink canonicalization
/// aliasing. Every declared source for a child is additive and cfg-blind, carrying its own
/// source-local ancestor set to prevent false cycle reports across mutually-exclusive cfg arms.
/// An inline body's own declarations are re-scanned even if sibling plain-file or remapped sources
/// exist. `inline_only` excludes stray same-named conventional files only when no plain file
/// actually resolved.
#[allow(clippy::type_complexity)]
pub(crate) fn reachable_modules(
    sources: &SourceTexts,
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
    let texts: HashMap<&PathBuf, OnceCell<Rc<str>>> =
        files.iter().map(|file| (file, OnceCell::new())).collect();
    let mut trees = HashMap::new();
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        let Some(scan_sources) = graph.by_module.get(&module).cloned() else {
            continue;
        };
        let children =
            collect_children(&module, &scan_sources, edition, sources, &texts, &mut trees)?;
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
                register_inline_sources(&child_path, bodies, &mut graph)?;
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
