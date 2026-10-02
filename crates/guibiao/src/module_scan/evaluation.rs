//! One evaluation's scans of the compilation units its module boundaries judge: each root's scan
//! is built once, on demand, and shared by every boundary judged over that root.
//!
//! A [`RootScan`] is what one compiled root's sources say — its source directory, its file list,
//! its reachability walk, and its unit scan — and every fact it holds is decided by the package
//! and the root alone. A boundary's own conditions (its governed set, its prefix and verbs, its
//! strict and external modifiers) are applied when that boundary is judged, against this reading,
//! and are never kept as a fact of the root — so two boundaries judging one root differently still
//! read it alike. [`EvaluationScans`] keeps an evaluation's scans keyed by the package's name and
//! the root's path exactly as the metadata reports it: the path is used as written, never
//! canonicalized, which is the identity the walk itself reads by. It also keeps the evaluation's
//! one [`SourceTexts`], shared by every root of every package, so a source two roots or two module
//! positions reach is read once.

use std::cell::{OnceCell, RefCell};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use serde_json::Value;
use xuanji::ScanDepth;

use crate::cargo_metadata::root_reading;
use crate::errors::{missing_src_error, out_of_package_root_error, walk_refusal_in_unit};

use super::source_texts::SourceTexts;
use super::{UnitScan, governed_files, names_crate_by_path_alone, reachable_modules, rust_files};

/// The conventional source directory, `manifest_dir/src`, for metadata reporting no target — the one case a root
/// is judged without a root file, since every package Cargo reports carries its targets.
fn package_src_dir(package: &Value) -> Option<PathBuf> {
    package["manifest_path"]
        .as_str()
        .and_then(|manifest| Path::new(manifest).parent())
        .map(|crate_dir| crate_dir.join("src"))
}

/// What one compiled root of a package says, read once. Every field is decided by the package and
/// the root alone; nothing a single boundary declares is recorded here.
pub(crate) struct RootScan {
    /// The directory the root's module tree is walked from: the root file's parent, or the
    /// conventional `src/` where the metadata reports no target.
    pub(crate) src_dir: PathBuf,
    /// The root file relative to `src_dir`, where the metadata named one.
    pub(crate) root_relative: Option<PathBuf>,
    /// The root's compilation unit label, absent where the metadata reports no target and the
    /// conventional source directory stands in.
    pub(crate) unit: Option<String>,
    /// Every `.rs` file under `src_dir`, minus the package's sibling roots (each is its own
    /// compilation unit) and the files that name `crate` by their path alone.
    pub(crate) files: Vec<PathBuf>,
    /// The module paths reachable from the root through `mod` declarations.
    pub(crate) reachable: BTreeSet<String>,
    /// The modules an inline declaration owns with no backing file, to the candidate file a
    /// suggestion names.
    pub(crate) inline_only: BTreeMap<String, PathBuf>,
    /// The `#[path]` remaps: a file and the module it is compiled as.
    pub(crate) remapped: Vec<(PathBuf, String)>,
    /// The conventional files a remap shadows.
    pub(crate) remap_shadowed: BTreeSet<String>,
    /// Every file of the unit read as its module — the crate-wide selection every rule family
    /// that scans the whole root shares, computed once at the read rather than once per family.
    all_files: Vec<(PathBuf, String)>,
    /// The unit's scan: every file read once into its scope table and occurrences, with the one
    /// resolver over all of them. Its memos key on a written path, a table, a scope, a site and a
    /// namespace — never on a boundary's conditions — so every boundary judged over this root
    /// resolves through the same answers.
    unit_scan: UnitScan,
    /// The unit's item definitions, collected the first time a boundary asks for them: an
    /// evaluation whose boundaries declare no inline confinement never pays for the collection.
    item_definitions: OnceCell<BTreeSet<String>>,
}

impl RootScan {
    /// Read the root's scan. The refusal order is the one a judgement over the scan must keep:
    /// the source walk's refusal first, then a file the unit scan cannot read, each ahead of
    /// anything a boundary decides from what the walk found. Every source is read through `sources`.
    fn read(
        sources: Rc<SourceTexts>,
        package: &Value,
        crate_package: &str,
        root_file: Option<&Path>,
        sibling_roots: Option<&[PathBuf]>,
    ) -> Result<Self, String> {
        let src_dir = match root_file.and_then(Path::parent) {
            Some(dir) => dir.to_path_buf(),
            None => package_src_dir(package).ok_or_else(|| missing_src_error(crate_package))?,
        };
        let root_relative = root_file
            .and_then(|root| root.strip_prefix(&src_dir).ok())
            .map(Path::to_path_buf);
        let unit = match root_file {
            Some(root) => Some(
                crate::cargo_metadata::compilation_unit_label(package, root)
                    .ok_or_else(|| out_of_package_root_error(crate_package, root))?,
            ),
            None => None,
        };
        let reading = root_reading(package, root_file, crate_package)?;
        let mut files = rust_files(&src_dir)?;
        if let Some(siblings) = sibling_roots {
            files.retain(|file| {
                root_file.is_some_and(|root| root == file.as_path()) || !siblings.contains(file)
            });
        }
        files.retain(|file| !names_crate_by_path_alone(file, &src_dir, root_relative.as_deref()));
        let (reachable, inline_only, remapped, remap_shadowed) = reachable_modules(
            &sources,
            &src_dir,
            &files,
            root_relative.as_deref(),
            reading.edition,
        )
        .map_err(|refusal| walk_refusal_in_unit(crate_package, unit.as_deref(), &refusal))?;
        let all_files = governed_files(
            &src_dir,
            &files,
            "crate",
            &reachable,
            &inline_only,
            &remapped,
            &remap_shadowed,
            root_relative.as_deref(),
            ScanDepth::Subtree,
        );
        let dependencies = crate::cargo_metadata::dependency_import_names(package)
            .into_iter()
            .chain(crate::cargo_metadata::library_import_names(package))
            .collect();
        let unit_scan = UnitScan::read(
            &sources,
            &all_files,
            reading.edition,
            reading.proc_macro,
            dependencies,
        )?;
        Ok(RootScan {
            src_dir,
            root_relative,
            unit,
            files,
            reachable,
            inline_only,
            remapped,
            remap_shadowed,
            all_files,
            unit_scan,
            item_definitions: OnceCell::new(),
        })
    }

    /// The unit's label for a finding: the compilation unit label, or `src` where the metadata
    /// reported no target and the conventional source directory was read.
    pub(crate) fn unit_label(&self) -> &str {
        self.unit.as_deref().unwrap_or("src")
    }

    /// Every file of the unit read as its module, selected once at the read.
    pub(crate) fn all_files(&self) -> &[(PathBuf, String)] {
        &self.all_files
    }

    /// The unit's scan, shared by every rule family of every boundary judged over this root.
    pub(crate) fn unit_scan(&self) -> &UnitScan {
        &self.unit_scan
    }

    /// Every item the unit's modules declare at their own level, keyed `{module}::{name}`,
    /// collected the first time a boundary asks.
    pub(crate) fn item_definitions(&self) -> &BTreeSet<String> {
        self.item_definitions
            .get_or_init(|| self.unit_scan.item_definitions())
    }

    /// The value-namespace item names `governed_module` declares at its own top level, read from the
    /// files that back that module alone (`Shallow`), not the whole crate: the question is only ever
    /// about the governed module itself. A module can be backed by more than one reachable file (a
    /// `#[path]` remap beside a conventional file, a `cfg_attr` union), so every backing file
    /// contributes, and inline descendants are excluded by the collector's own true-module keying.
    /// Each file's names are read from the table the unit scan built for that file and module, so the
    /// inventory reads no source and builds no table of its own.
    pub(crate) fn governed_module_value_items(
        &self,
        governed_module: &str,
    ) -> Result<HashSet<String>, String> {
        let mut items = HashSet::new();
        for (file, module) in governed_files(
            &self.src_dir,
            &self.files,
            governed_module,
            &self.reachable,
            &self.inline_only,
            &self.remapped,
            &self.remap_shadowed,
            self.root_relative.as_deref(),
            ScanDepth::Shallow,
        ) {
            items.extend(self.unit_scan.value_items(&file, &module)?);
        }
        Ok(items)
    }
}

/// One root of one package, as the boundaries judging it name it: the package's name and the
/// root's path exactly as the metadata reports it.
type RootKey = (String, Option<PathBuf>);

/// The root scans of one evaluation of a constitution: each is built the first time a boundary
/// asks for its root and shared by every boundary that asks again, so a root's sources are read
/// once per evaluation however many boundaries are judged over them.
pub(crate) struct EvaluationScans {
    scans: RefCell<HashMap<RootKey, Rc<RootScan>>>,
    /// The evaluation's one reading of each source path, shared by every root scan it builds.
    sources: Rc<SourceTexts>,
    /// Whether a lookup shares the scan it finds or builds again. Always [`Sharing::Shared`]
    /// outside tests; the independent form exists so a direction can hold the shared form to
    /// yielding one outcome with a scan built per boundary.
    #[cfg(test)]
    sharing: Sharing,
    /// How many root scans this evaluation has built, incremented where a build happens and never
    /// where a lookup finds one already built — the work the one-scan-per-root requirement names,
    /// observable by a direction as `resolve`'s `scope_reads` observes a resolution's work.
    #[cfg(test)]
    builds: std::cell::Cell<usize>,
}

/// Whether an [`EvaluationScans`] lookup reuses the scan it built for a root.
#[cfg(test)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum Sharing {
    /// The first lookup builds; every later lookup of the same root shares that scan.
    Shared,
    /// Every lookup builds again and nothing is reused — neither a root's scan nor a source's
    /// text: a scan per boundary, as an evaluation without sharing reads.
    Independent,
}

impl EvaluationScans {
    /// The shared scan set of one evaluation — the only form an evaluation builds.
    pub(crate) fn shared() -> Self {
        EvaluationScans {
            scans: RefCell::new(HashMap::new()),
            sources: Rc::default(),
            #[cfg(test)]
            sharing: Sharing::Shared,
            #[cfg(test)]
            builds: std::cell::Cell::new(0),
        }
    }

    /// A scan set that builds a root's scan again at every lookup and never reuses one: the
    /// scan-per-boundary reading a direction compares the shared form against, never a form an
    /// evaluation runs.
    #[cfg(test)]
    pub(crate) fn independent() -> Self {
        EvaluationScans {
            scans: RefCell::new(HashMap::new()),
            sources: Rc::default(),
            sharing: Sharing::Independent,
            builds: std::cell::Cell::new(0),
        }
    }

    /// The scan of the package's root at `root_file`, building it the first time any boundary of
    /// this evaluation asks. A build failure is returned and never kept: the evaluation returns at
    /// the first error, so nothing asks for the root again.
    pub(crate) fn root_scan(
        &self,
        package: &Value,
        crate_package: &str,
        root_file: Option<&Path>,
        sibling_roots: Option<&[PathBuf]>,
    ) -> Result<Rc<RootScan>, String> {
        #[cfg(test)]
        if self.sharing == Sharing::Independent {
            return Ok(Rc::new(self.build(
                package,
                crate_package,
                root_file,
                sibling_roots,
            )?));
        }
        let key: RootKey = (crate_package.to_string(), root_file.map(Path::to_path_buf));
        if let Some(scan) = self.scans.borrow().get(&key) {
            return Ok(Rc::clone(scan));
        }
        let scan = Rc::new(self.build(package, crate_package, root_file, sibling_roots)?);
        self.scans.borrow_mut().insert(key, Rc::clone(&scan));
        Ok(scan)
    }

    /// Build the root's scan — the one place an evaluation reads a root's sources, so it is where
    /// the build count is kept; a lookup answered by a scan already built does none of this work.
    fn build(
        &self,
        package: &Value,
        crate_package: &str,
        root_file: Option<&Path>,
        sibling_roots: Option<&[PathBuf]>,
    ) -> Result<RootScan, String> {
        #[cfg(test)]
        self.builds.set(self.builds.get() + 1);
        #[cfg(test)]
        if self.sharing == Sharing::Independent {
            return RootScan::read(
                Rc::default(),
                package,
                crate_package,
                root_file,
                sibling_roots,
            );
        }
        RootScan::read(
            Rc::clone(&self.sources),
            package,
            crate_package,
            root_file,
            sibling_roots,
        )
    }

    /// How many root scans this evaluation has built: one per root judged when every lookup
    /// shares, one per lookup when nothing is reused.
    #[cfg(test)]
    pub(crate) fn roots_built(&self) -> usize {
        self.builds.get()
    }

    /// How many times this evaluation has read each source path from the file system, through the
    /// text set every root scan it shares reads from. A scan built when nothing is reused reads
    /// through a text set of its own, which this does not count.
    #[cfg(test)]
    pub(crate) fn source_reads(&self) -> HashMap<PathBuf, usize> {
        self.sources.reads()
    }

    /// Classification work by file and module across the root scans this evaluation shares.
    #[cfg(test)]
    pub(crate) fn classifications(&self) -> HashMap<(PathBuf, String), usize> {
        let mut counts = HashMap::new();
        for root in self.scans.borrow().values() {
            for (pair, count) in root.unit_scan().classifications() {
                *counts.entry(pair).or_default() += count;
            }
        }
        counts
    }
}
