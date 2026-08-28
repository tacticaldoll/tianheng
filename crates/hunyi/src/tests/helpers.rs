use serde_json::Value;
pub(super) use std::path::{Path, PathBuf};

pub(super) use crate::containment::leaf_of;
pub(super) use crate::crate_scope::dependency_names;
pub(super) use crate::errors::{
    dual_backed_module_error, missing_module_file_error, unknown_module_error, unknown_trait_error,
};
pub(super) use crate::exposure::module_findings;
pub(super) use crate::finding::SemanticFact;
pub(super) use crate::module_resolve::resolve_module_file;

/// A unique, self-cleaning temp `src/` tree: write source files (and, where needed, a symlink),
/// then hand its root/src paths to a pure entrypoint under test — replaces the hand-rolled
/// `temp_dir().join(format!(...))` + manual `remove_dir_all` at both ends that this file's many
/// fixture-building helpers otherwise each repeat.
#[allow(dead_code)]
pub(super) struct TempSrcTree {
    pub(super) dir: PathBuf,
    pub(super) src: PathBuf,
}

impl TempSrcTree {
    pub(super) fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("hunyi-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        Self { dir, src }
    }

    /// Write a source file at `rel` (relative to `src/`), creating parent dirs as needed.
    /// Returns the file's absolute path.
    pub(super) fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent")).expect("mkdir");
        std::fs::write(&path, contents).expect("write source");
        path
    }

    /// Write every `(relative path, contents)` pair under `src/`.
    pub(super) fn write_all(&self, files: &[(&str, &str)]) {
        for (rel, contents) in files {
            self.write(rel, contents);
        }
    }

    pub(super) fn src(&self) -> &Path {
        &self.src
    }

    pub(super) fn root(&self) -> PathBuf {
        self.src.join("lib.rs")
    }

    #[cfg(unix)]
    pub(super) fn symlink(&self, target: impl AsRef<Path>, link_rel_to_src: &str) -> &Self {
        std::os::unix::fs::symlink(target, self.src.join(link_rel_to_src)).expect("create symlink");
        self
    }

    pub(super) fn metadata(&self) -> Value {
        serde_json::json!({
            "packages": [{
                "name": "x",
                "dependencies": [],
                "targets": [{ "kind": ["lib"], "src_path": self.root().to_string_lossy().into_owned() }],
            }],
        })
    }
}

impl Drop for TempSrcTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// Write `files` under a unique temp `src` tree and exercise the semantic evaluator without
/// spawning `cargo`. Capability-facing helpers below name the posture they need while this
/// function owns the shared module-resolution → exposure → use-resolution → match pipeline.
pub(super) fn semantic_findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    include_trait_impls: bool,
    deps: &[&str],
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(name);
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|s| s.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|s| s.to_string()).collect();
    let result = module_findings(
        tree.src(),
        &tree.root(),
        module,
        &forbidden,
        "x",
        include_trait_impls,
        &deps,
    );
    result.map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

pub(super) type ShapeModuleEvaluator =
    fn(&Path, &Path, &str, &str) -> Result<Vec<(SemanticFact, PathBuf)>, String>;
pub(super) type ShapeSubtreeEvaluator =
    fn(&Path, &Path, &str, &str) -> Result<Vec<(SemanticFact, String, PathBuf)>, String>;
pub(super) type OperandModuleEvaluator = fn(
    &Path,
    &Path,
    &str,
    &[String],
    &str,
    &[String],
) -> Result<Vec<(SemanticFact, PathBuf)>, String>;

/// Exercise one of hunyi's shape-only module observers and project its facts to the strings
/// capability tests assert. The observer remains capability-specific; only fixture plumbing and
/// the common reaction projection live here.
pub(super) fn shape_findings(
    family: &str,
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    evaluate: ShapeModuleEvaluator,
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("{family}-{name}"));
    tree.write_all(files);
    evaluate(tree.src(), &tree.root(), module, "x").map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

/// Exercise a shape observer across a governed subtree while retaining module attribution.
pub(super) fn subtree_findings(
    family: &str,
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    evaluate: ShapeSubtreeEvaluator,
) -> Result<Vec<(String, String)>, String> {
    let tree = TempSrcTree::new(&format!("{family}-sub-{name}"));
    tree.write_all(files);
    evaluate(tree.src(), &tree.root(), module, "x").map(|facts| {
        facts
            .into_iter()
            .map(|(fact, module, _file)| (fact.to_string(), module))
            .collect()
    })
}

/// Exercise an operand-scoped shape observer with the same canonical forbidden/dependency inputs
/// production receives, then project its facts to the reaction strings capability tests assert.
pub(super) fn operand_findings(
    family: &str,
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
    evaluate: OperandModuleEvaluator,
) -> Result<Vec<String>, String> {
    let tree = TempSrcTree::new(&format!("{family}op-{name}"));
    tree.write_all(files);
    let forbidden: Vec<String> = forbidden.iter().map(|value| value.to_string()).collect();
    let deps: Vec<String> = deps.iter().map(|value| value.to_string()).collect();
    evaluate(tree.src(), &tree.root(), module, &forbidden, "x", &deps).map(|facts| {
        facts
            .into_iter()
            .map(|(fact, _file)| fact.to_string())
            .collect()
    })
}

/// Return the default semantic findings for `module` against `forbidden`.
pub(super) fn findings(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    semantic_findings(name, files, module, forbidden, false, &[])
}

/// Like [`findings`] but with a declared **dependency-name set** (already `-`→`_`
/// normalized, as `dependency_names` produces), so an external-crate exposure resolves.
pub(super) fn findings_with_deps(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
    deps: &[&str],
) -> Result<Vec<String>, String> {
    semantic_findings(name, files, module, forbidden, false, deps)
}

/// Like [`findings`] but with the `semantic-trait-impl-exposure` opt-in enabled, so a trait
/// `impl` block's impl-site-authored positions are also observed.
pub(super) fn findings_including_trait_impls(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
    forbidden: &[&str],
) -> Result<Vec<String>, String> {
    semantic_findings(name, files, module, forbidden, true, &[])
}

/// Build fixtures under a temp `src` plus synthetic `cargo metadata --no-deps` for a single
/// crate `x` whose lib root is that `src/lib.rs`, so a private `check_*_boundary` can run
/// without spawning `cargo`. Returns `(metadata, tree)`; the tree's `Drop` removes the fixtures
/// once the caller drops it — hold it alive until after the check (the check reads the fixtures
/// from disk).
pub(super) fn fixture_metadata(name: &str, files: &[(&str, &str)]) -> (Value, TempSrcTree) {
    let tree = TempSrcTree::new(&format!("meta-{name}"));
    tree.write_all(files);
    (tree.metadata(), tree)
}
