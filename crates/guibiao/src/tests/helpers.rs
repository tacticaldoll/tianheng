pub(super) use crate::module_check::check_module_boundary;
pub(super) use crate::*;
pub(super) use serde_json::Value;
use std::path::{Path, PathBuf};

#[allow(dead_code)]
pub(super) struct TempWorkspace {
    pub(super) dir: PathBuf,
    pub(super) src: PathBuf,
}

impl TempWorkspace {
    pub(super) fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("guibiao-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        Self { dir, src }
    }

    pub(super) fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent"))
            .expect("create src dirs");
        std::fs::write(&path, contents).expect("write source file");
        path
    }

    pub(super) fn write_at(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent"))
            .expect("create parent dirs");
        std::fs::write(&path, contents).expect("write file");
        path
    }

    #[cfg(unix)]
    #[allow(dead_code)]
    pub(super) fn symlink(&self, target: impl AsRef<Path>, link_rel: &str) -> &Self {
        std::os::unix::fs::symlink(target, self.src.join(link_rel)).expect("create symlink");
        self
    }

    pub(super) fn src(&self) -> &Path {
        &self.src
    }

    pub(super) fn dir(&self) -> &Path {
        &self.dir
    }

    pub(super) fn metadata(&self, pkg_name: &str) -> Value {
        serde_json::json!({
            "packages": [{
                "name": pkg_name,
                "targets": [{
                    "kind": ["lib"],
                    "src_path": self.src.join("lib.rs").to_string_lossy(),
                }],
            }],
        })
    }

    pub(super) fn metadata_with_deps(
        &self,
        pkg_name: &str,
        deps: &[(&str, Option<&str>)],
    ) -> Value {
        let deps_json: Vec<Value> = deps
            .iter()
            .map(|(name, rename)| {
                let mut dep = serde_json::json!({ "name": name });
                if let Some(rename) = rename {
                    dep["rename"] = serde_json::json!(rename);
                }
                dep
            })
            .collect();
        serde_json::json!({
            "packages": [{
                "name": pkg_name,
                "dependencies": deps_json,
                "targets": [{
                    "kind": ["lib"],
                    "src_path": self.src.join("lib.rs").to_string_lossy(),
                }],
            }],
        })
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

pub(super) fn run_module_check(
    name: &str,
    files: &[(&str, &str)],
    boundary: ModuleBoundary,
) -> (Result<(), String>, Vec<Violation>) {
    let ws = TempWorkspace::new(name);
    for (rel, contents) in files {
        ws.write(rel, contents);
    }
    let metadata = ws.metadata("x");
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);
    (result, violations)
}

pub(super) fn run_module_check_with_deps(
    name: &str,
    files: &[(&str, &str)],
    deps: &[(&str, Option<&str>)],
    boundary: ModuleBoundary,
) -> (Result<(), String>, Vec<Violation>) {
    let ws = TempWorkspace::new(name);
    for (rel, contents) in files {
        ws.write(rel, contents);
    }
    let metadata = ws.metadata_with_deps("x", deps);
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);
    (result, violations)
}

pub(super) fn test_id(target: &str, rule: &str, finding: &str) -> ViolationId {
    let finding = match finding.split_once('/') {
        Some((package, feature)) => crate::finding::CrateFact::feature(
            package.to_string(),
            feature.to_string(),
            DependencyKind::Normal,
        )
        .into_finding(),
        None => crate::finding::CrateFact::dependency(finding.to_string(), DependencyKind::Normal)
            .into_finding(),
    };
    ViolationId::new(
        target,
        RuleKey::of("tianheng.rule/test/policy", [("policy", rule)]),
        finding.key().clone(),
    )
}

pub(super) fn one_enforce_violation() -> Report {
    Report::new(vec![Violation::new(
        BoundaryKind::Crate,
        test_id("core", "deny external dependencies", "serde"),
        "deny external dependencies",
        "serde",
        "core must stay dependency-light".to_string(),
        Severity::Enforce,
    )])
}

pub(super) fn protect_internal_from(forbidden_importer: &str) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::internal")
        .must_not_be_imported_by(forbidden_importer)
        .because("internal is private to the crate")
}

pub(super) fn restrict_kernel_to_types(governed: &str, allowed: &[&str]) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module(governed)
        .restrict_imports_to(allowed.to_vec())
        .because("the kernel may import only the allowed modules")
}
