//! White-box unit tests for the crate-private machinery — the baseline, the JSON
//! and text projections, and the source scanner. Black-box behavior (running
//! `check` against fixture workspaces) lives in `tests/dogfood.rs`.
use std::path::{Path, PathBuf};

use super::*;

fn test_id(target: &str, rule: &str, finding: &str) -> ViolationId {
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
    ViolationId::new(target, rule, finding)
}

fn one_enforce_violation() -> Report {
    Report::new(vec![Violation::new(
        BoundaryKind::Crate,
        test_id("core", "deny external dependencies", "serde"),
        "core must stay dependency-light".to_string(),
        Severity::Enforce,
    )])
}

#[test]
fn every_static_rule_has_an_exact_semantic_key() {
    let crate_rules = vec![
        (
            Rule::DenyExternalDependencies {
                allowed: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/deny-external-dependencies",
            vec![("allowed", "[\"serde\"]")],
        ),
        (
            Rule::ForbidDependencyOn {
                crates: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/forbid-dependency-on",
            vec![("crates", "[\"serde\"]")],
        ),
        (
            Rule::RestrictDependenciesTo {
                allowed: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/restrict-dependencies-to",
            vec![("allowed", "[\"serde\"]")],
        ),
        (
            Rule::RestrictWorkspaceDependenciesTo {
                allowed: vec!["domain".to_string()],
            },
            "tianheng.rule/guibiao/restrict-workspace-dependencies-to",
            vec![("allowed", "[\"domain\"]")],
        ),
        (
            Rule::RestrictDependencySourcesTo {
                allowed: vec![SourceKind::Registry],
            },
            "tianheng.rule/guibiao/restrict-dependency-sources-to",
            vec![("allowed", "[\"registry\"]")],
        ),
        (
            Rule::RestrictFeaturesOf {
                crate_: "serde".to_string(),
                allowed: vec!["derive".to_string()],
            },
            "tianheng.rule/guibiao/restrict-features-of",
            vec![("allowed", "[\"derive\"]"), ("crate", "serde")],
        ),
        (
            Rule::ForbidFeaturesOf {
                crate_: "serde".to_string(),
                forbidden: vec!["unstable".to_string()],
            },
            "tianheng.rule/guibiao/forbid-features-of",
            vec![("crate", "serde"), ("forbidden", "[\"unstable\"]")],
        ),
    ];
    for (rule, expected, fields) in crate_rules {
        assert_eq!(rule.key().rule_type(), expected);
        assert_eq!(rule.key().fields().collect::<Vec<_>>(), fields);
    }

    let module_rules = vec![
        (
            ModuleRule::MustNotImport {
                module: "crate::adapter".to_string(),
            },
            "tianheng.rule/guibiao/must-not-import",
            vec![("module", "crate::adapter")],
        ),
        (
            ModuleRule::RestrictImportsTo {
                allowed: vec!["crate::types".to_string()],
            },
            "tianheng.rule/guibiao/restrict-imports-to",
            vec![("allowed", "[\"crate::types\"]")],
        ),
        (
            ModuleRule::MustNotBeImportedBy {
                importer: "crate::http".to_string(),
            },
            "tianheng.rule/guibiao/must-not-be-imported-by",
            vec![("importer", "crate::http")],
        ),
        (
            ModuleRule::MustOnlyBeImportedBy {
                allowed: vec!["crate::facade".to_string()],
            },
            "tianheng.rule/guibiao/must-only-be-imported-by",
            vec![("allowed", "[\"crate::facade\"]")],
        ),
        (
            ModuleRule::ConfineExternalCrate {
                crate_name: "libc".to_string(),
            },
            "tianheng.rule/guibiao/confine-external-crate",
            vec![("crate", "libc")],
        ),
        (
            ModuleRule::ConfineInlineSymbolPath {
                prefix: "std::time".to_string(),
                ending_with: Some(vec!["now".to_string()]),
                strict: false,
                strict_external: false,
            },
            "tianheng.rule/guibiao/confine-inline-symbol-path",
            vec![
                ("ending_with", "[\"now\"]"),
                ("prefix", "std::time"),
                ("strict", "false"),
            ],
        ),
    ];
    for (rule, expected, fields) in module_rules {
        assert_eq!(rule.key().rule_type(), expected);
        assert_eq!(rule.key().fields().collect::<Vec<_>>(), fields);
    }
}

#[test]
fn rule_set_order_is_canonical_and_presentation_is_not_identity() {
    let left = Rule::ForbidDependencyOn {
        crates: vec!["serde".to_string(), "tokio".to_string()],
    };
    let right = Rule::ForbidDependencyOn {
        crates: vec!["tokio".to_string(), "serde".to_string()],
    };
    assert_eq!(left.key(), right.key());
    let changed_law = Rule::ForbidDependencyOn {
        crates: vec!["serde".to_string(), "tracing".to_string()],
    };
    assert_ne!(left.key(), changed_law.key());

    let default = ModuleRule::ConfineInlineSymbolPath {
        prefix: "std::time".to_string(),
        ending_with: Some(vec!["now".to_string()]),
        strict: false,
        strict_external: false,
    };
    let strict_external = ModuleRule::ConfineInlineSymbolPath {
        prefix: "std::time".to_string(),
        ending_with: Some(vec!["now".to_string()]),
        strict: false,
        strict_external: true,
    };
    assert_ne!(default.text(), strict_external.text());
    assert_eq!(default.key(), strict_external.key());

    let raw = ModuleRule::MustNotImport {
        module: "crate::r#type".to_string(),
    };
    let plain = ModuleRule::MustNotImport {
        module: "crate::type".to_string(),
    };
    assert_eq!(raw.key(), plain.key());
}

#[test]
fn dependency_fact_identity_survives_reorder_and_unrelated_insertion() {
    fn identities(package: &serde_json::Value) -> Vec<FindingKey> {
        Rule::RestrictDependencySourcesTo {
            allowed: vec![SourceKind::Registry],
        }
        .facts(package, &[], DependencyKind::Normal)
        .into_iter()
        .map(|fact| fact.into_finding().key().clone())
        .collect()
    }

    let before = serde_json::json!({
        "dependencies": [
            { "name": "blocked", "source": "git+https://example.invalid/blocked", "kind": null }
        ]
    });
    let after = serde_json::json!({
        "dependencies": [
            { "name": "allowed", "source": "registry+https://example.invalid/index", "kind": null },
            { "name": "blocked", "source": "git+https://example.invalid/blocked", "kind": null }
        ]
    });
    assert_eq!(identities(&before), identities(&after));

    let distinct_sources = serde_json::json!({
        "dependencies": [
            { "name": "same", "source": null, "kind": null },
            { "name": "same", "source": "git+https://example.invalid/same", "kind": null }
        ]
    });
    let facts = Rule::RestrictDependencySourcesTo { allowed: vec![] }
        .facts(&distinct_sources, &[], DependencyKind::Normal)
        .into_iter()
        .map(|fact| fact.into_finding().key().clone())
        .collect::<Vec<_>>();
    assert_eq!(facts.len(), 2);
    assert_ne!(facts[0], facts[1]);
}

/// A unique, self-cleaning temp Cargo-package-shaped `src/` tree: write source files, add
/// symlinks, then build the `cargo metadata`-shaped JSON `check_module_boundary` reads — replaces
/// the hand-rolled `temp_dir().join(format!(...))` + manual `remove_dir_all` at both ends that
/// `run_module_check` and the symlink regression tests below otherwise each repeat.
struct TempWorkspace {
    dir: PathBuf,
    src: PathBuf,
}

impl TempWorkspace {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("guibiao-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        Self { dir, src }
    }

    /// Write a source file at `rel` (relative to `src/`), creating parent dirs as needed.
    /// Returns the file's absolute path.
    fn write(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.src.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent"))
            .expect("create src dirs");
        std::fs::write(&path, contents).expect("write source file");
        path
    }

    /// Write a file at a workspace-root-relative path (e.g. outside `src/`, for a symlink
    /// target), creating parent dirs as needed. Returns the file's absolute path.
    fn write_at(&self, rel: &str, contents: &str) -> PathBuf {
        let path = self.dir.join(rel);
        std::fs::create_dir_all(path.parent().expect("file has a parent"))
            .expect("create parent dirs");
        std::fs::write(&path, contents).expect("write file");
        path
    }

    /// Symlink `target` (an absolute path, or a raw relative string resolved from the link's own
    /// directory — passed through verbatim either way) at `link_rel` (relative to `src/`).
    #[cfg(unix)]
    fn symlink(&self, target: impl AsRef<Path>, link_rel: &str) -> &Self {
        std::os::unix::fs::symlink(target, self.src.join(link_rel)).expect("create symlink");
        self
    }

    fn src(&self) -> &Path {
        &self.src
    }

    fn dir(&self) -> &Path {
        &self.dir
    }

    /// The `cargo metadata`-shaped JSON `check_module_boundary` reads, for package `name` with no
    /// declared dependencies.
    fn metadata(&self, name: &str) -> serde_json::Value {
        self.metadata_with_deps(name, &[])
    }

    /// Like [`Self::metadata`], but with declared dependencies — `(name, rename)`, `rename` is the
    /// Cargo `pkg = { package = "…" }` alias when `Some` (the `-`→`_` fold is applied by the
    /// reader under test, so pass names verbatim).
    fn metadata_with_deps(&self, name: &str, deps: &[(&str, Option<&str>)]) -> serde_json::Value {
        let manifest = self.dir.join("Cargo.toml");
        let dependencies: Vec<serde_json::Value> = deps
            .iter()
            .map(|(dep_name, rename)| match rename {
                Some(rename) => serde_json::json!({ "name": dep_name, "rename": rename }),
                None => serde_json::json!({ "name": dep_name }),
            })
            .collect();
        serde_json::json!({
            "packages": [{
                "name": name,
                "manifest_path": manifest.to_string_lossy().into_owned(),
                "dependencies": dependencies,
            }]
        })
    }
}

impl Drop for TempWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(unix)]
#[test]
fn rust_files_does_not_recurse_into_a_symlinked_directory_cycle() {
    // A directory symlink pointing back at an ancestor would make a symlink-following walk recurse
    // forever (stack overflow). `rust_files` decides recursion from `file_type()` (no symlink
    // follow), so the cycle is not entered and the real source file is still found.
    let ws = TempWorkspace::new("symlink-cycle");
    ws.write("lib.rs", "// root\n");
    ws.symlink(ws.src(), "loop");
    let files = crate::module_scan::rust_files(ws.src());
    let files = files.expect("rust_files must not error on a symlink cycle");
    assert_eq!(
        files.len(),
        1,
        "only the real source file; the symlinked directory must not be followed: {files:?}"
    );
}

#[cfg(unix)]
#[test]
fn rust_files_governs_a_symlinked_source_file() {
    // A symlink whose target is a real `.rs` file is compiled by rustc (once `mod`-declared) and
    // must be governed — it must NOT be dropped as a non-`is_file()` symlink, which would silently
    // miss its imports. Only the directory branch is symlink-blind (for cycle safety).
    let ws = TempWorkspace::new("symlink-file");
    ws.write("lib.rs", "// root\n");
    let shared = ws.write_at("external/shared.rs", "// shared\n");
    ws.symlink(&shared, "shared.rs");
    let files = crate::module_scan::rust_files(ws.src());
    let files = files.expect("rust_files must not error");
    assert_eq!(
        files.len(),
        2,
        "both the real file and the symlinked-in source file are governed: {files:?}"
    );
}

#[cfg(unix)]
#[test]
fn a_plain_child_reached_only_through_a_symlinked_directory_is_governed() {
    // `rust_files` deliberately never recurses into a symlinked directory (its own cycle guard,
    // see the sibling tests above), so a file that lives only behind one is absent from the
    // structural file list `governed_files` scans from. But `reachable_modules`'s live probe for
    // a plain child resolves the candidate path via `is_file`/`canonicalize`, which DO follow a
    // symlinked directory component — so before this fix, such a child was marked reachable and
    // even read/descended into, yet was absent from every `governed_files` output (neither the
    // structural iterator, which never walked it, nor `remapped`, since its naive path
    // structurally agreed with its own module path). Verified against a real `cargo check`: this
    // exact layout compiles `real_target/child.rs` as `crate::mymod::child` through the symlink.
    let ws = TempWorkspace::new("symlinked-child");
    ws.write(
        "lib.rs",
        "pub mod mymod;\npub mod secret { pub struct Thing; }\n",
    );
    ws.write("mymod.rs", "pub mod child;\n");
    let target = ws.write_at("real_target/child.rs", "use crate::secret::Thing;\n");
    ws.symlink(target.parent().expect("target has a parent"), "mymod");

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::mymod")
        .must_not_import("crate::secret")
        .because("a symlinked-directory child must still be governed, not silently invisible");
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    result.expect("a symlinked-directory-reached child is a valid, governable target");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::mymod");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a module-import violation carries its source file");
    assert!(
        file.ends_with("child.rs"),
        "the violation must name the real file reached through the symlinked directory: {file}"
    );
}

#[cfg(unix)]
#[test]
fn a_symlinked_module_aliasing_an_unrelated_walked_file_is_still_governed_under_its_own_path() {
    // Round-6 regression in the fix above: `files_canon` compared CANONICAL (symlink-resolved)
    // identity, so a module reached through a symlinked directory that happens to alias the same
    // physical file as some OTHER, unrelated, genuinely-walked module was wrongly treated as
    // "already found by the structural iterator" merely because canon(candidate) == canon(some
    // files entry) — even though the candidate itself was never walked. Here `mod real;` is
    // backed directly by `src/real/mod.rs` (normally walked), and a SEPARATE `mod kernel;` is
    // backed by `src/kernel/mod.rs`, where `src/kernel` is a symlink to `src/real` (never walked,
    // since rust_files skips symlinked directories) — so canon(src/kernel/mod.rs) ==
    // canon(src/real/mod.rs) even though they are two distinct, separately-declared modules.
    // Verified against a real `cargo check`: both crate::real and crate::kernel compile as
    // distinct modules, each observing `use crate::secret::Thing;`. Comparing LITERAL (not
    // canonical) path identity closes this: two on-disk paths are never literally equal merely
    // because they resolve to the same target.
    let ws = TempWorkspace::new("symlink-alias");
    ws.write(
        "lib.rs",
        "pub mod real;\npub mod kernel;\npub mod secret { pub struct Thing; }\n",
    );
    ws.write("real/mod.rs", "use crate::secret::Thing;\n");
    // A raw relative symlink target ("real", not an absolute path) — verbatim, matching the
    // original regression's exact on-disk shape.
    ws.symlink("real", "kernel");

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_import("crate::secret")
        .because(
            "a symlink-aliased module must be governed under its own path, not silently dropped",
        );
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    result.expect(
        "crate::kernel is a valid, governable target even though it aliases crate::real's file",
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::kernel");
}

#[test]
fn expand_use_tree_does_not_overflow_on_pathological_nesting() {
    // A pathologically brace-nested `use` must not overflow the stack. The depth cap bounds the
    // recursion and returns (truncated past the cap — the safe direction for adversarial input no
    // real, rustfmt-clean source reaches). The point of the test is that it terminates.
    let deep = format!("use {}b{};", "a::{".repeat(20_000), "}".repeat(20_000));
    let out = crate::module_scan::imported_module_paths(&deep, "crate::m", &[]);
    assert!(
        out.is_empty(),
        "past the depth cap the sub-tree is not expanded: {out:?}"
    );
}

#[test]
fn a_raw_identifier_use_does_not_swallow_the_following_real_use() {
    use crate::module_scan::imported_module_paths;
    // `r#use` is a valid raw identifier (a field here). `#` is not an identifier byte, so a naive
    // "the byte before `use` is not an ident byte" test would treat the `use` inside `r#use` as the
    // keyword, scan to the next `;`, and swallow the following real `use` — a false negative that
    // silently disables the import boundary. The observed imports must be identical to the same
    // source without the raw-identifier field, and must include the real import.
    let with_raw = "struct Config { r#use: bool }\nuse crate::secret::Thing;\n";
    let without = "struct Config { flag: bool }\nuse crate::secret::Thing;\n";
    let observed = imported_module_paths(with_raw, "crate::m", &[]);
    assert_eq!(
        observed,
        imported_module_paths(without, "crate::m", &[]),
        "an r#use field must not change which imports are observed"
    );
    assert!(
        !observed.is_empty(),
        "the real `use crate::secret::Thing` after an r#use field must still be observed: {observed:?}"
    );
}

#[test]
fn a_use_in_a_whitespace_spaced_macro_body_is_not_a_real_import() {
    use crate::module_scan::imported_module_paths;
    // Rust allows whitespace between a macro path and its `!` (`cfg_if ! { … }`). The body is a
    // macro-generated context, out of scope per the module-boundary spec, so its `use` must be
    // stripped — not observed as a real import (a false positive).
    let spaced = "cfg_if ! { use crate::secret::X; }\n";
    assert!(
        imported_module_paths(spaced, "crate::m", &[]).is_empty(),
        "a use inside a whitespace-spaced macro invocation body is macro-generated, not a real import"
    );
    // But a unary `!` on a real block after a keyword (`return !{ … }`) is NOT a macro — its block
    // is real code, and a `use` inside it must still be observed (guarding against a false negative
    // if the whitespace tolerance forgot the keyword check).
    let keyword_not = "pub fn f() -> bool { return !{ use crate::real::Y; true }; }\n";
    assert!(
        !imported_module_paths(keyword_not, "crate::m", &[]).is_empty(),
        "a use in a real block after `return !` must still be observed — a keyword is not a macro name"
    );
}

#[test]
fn a_submodule_file_named_lib_rs_is_governed_at_its_own_path() {
    // `lib.rs`/`main.rs` are segment-less only at the crate root. A declared submodule file
    // `foo/lib.rs` is `crate::foo::lib` (matching rustc and 渾儀's descent), so a boundary on it
    // resolves and reacts instead of raising a false inline-module exit-2 or scanning the wrong
    // module.
    let (result, violations) = run_module_check(
        "submodule-named-lib",
        &[
            ("lib.rs", "pub mod foo;\npub mod sink;\n"),
            ("foo.rs", "pub mod lib;\n"),
            ("foo/lib.rs", "use crate::sink;\n"),
            ("sink.rs", "// target\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::foo::lib")
            .must_not_import("crate::sink")
            .because("foo::lib must not touch sink"),
    );
    assert!(
        result.is_ok(),
        "the submodule file must resolve, not raise a false exit-2: {result:?}"
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::foo::lib");
    assert_eq!(violations[0].rule, "module must not import");
    assert_eq!(violations[0].finding, "crate::sink");
    let id = violations[0].id();
    assert_eq!(id.target, "crate::foo::lib");
    let rule = id
        .rule_key()
        .expect("a production rule has semantic identity");
    assert_eq!(rule.rule_type(), "tianheng.rule/guibiao/must-not-import");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![("module", "crate::sink")]
    );
    let key = id
        .finding_key()
        .expect("a production violation has structured identity");
    assert_eq!(key.fact_type(), "tianheng.fact/guibiao/imported-path");
    assert_eq!(key.shape(), "module-path");
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![("path", "crate::sink")]
    );
}

/// An unreadable governed source file must surface as a scan error (exit 2),
/// not a silent skip that could hide a real module-boundary violation. Unix
/// only (permission-based) and self-calibrating: it skips under a privileged
/// user (e.g. root in CI), where mode 0 is still readable, rather than
/// false-passing.
#[cfg(unix)]
#[test]
fn unreadable_governed_file_is_a_scan_error() {
    use std::os::unix::fs::PermissionsExt;

    let ws = TempWorkspace::new("unreadable");
    let file = ws.write("lib.rs", "use crate::forbidden::Thing;\n");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o000))
        .expect("drop read permission");

    // Self-calibrating root guard: if mode 0 is still readable, permissions do
    // not bite here, so the premise cannot hold — skip rather than false-pass.
    if std::fs::read_to_string(&file).is_ok() {
        return;
    }

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate")
        .must_not_import("crate::forbidden")
        .because("the test module must not import the forbidden module");

    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    let _ = std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o644));

    assert!(
        result.is_err(),
        "an unreadable governed file must be a scan error, not a silent skip"
    );
}

/// An unreadable governed *directory* must surface as a scan error (exit 2), the
/// same "cannot judge, not nothing to judge" rule as an unreadable file: a skipped
/// subtree could hide a real module-boundary violation. Unix only and
/// self-calibrating (skips under a privileged user where mode 0 is still readable).
#[cfg(unix)]
#[test]
fn unreadable_governed_directory_is_a_scan_error() {
    use std::os::unix::fs::PermissionsExt;

    let ws = TempWorkspace::new("unreadable-dir");
    ws.write("lib.rs", "// nothing\n");
    ws.write("sub/inner.rs", "use crate::forbidden::Thing;\n");
    let sub = ws.src().join("sub");
    std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o000))
        .expect("drop dir read/exec permission");

    // Self-calibrating root guard: if the directory is still traversable, the
    // premise cannot hold — skip rather than false-pass.
    if std::fs::read_dir(&sub).is_ok() {
        let _ = std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755));
        return;
    }

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate")
        .must_not_import("crate::forbidden")
        .because("the test module must not import the forbidden module");

    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    let _ = std::fs::set_permissions(&sub, std::fs::Permissions::from_mode(0o755));

    assert!(
        result.is_err(),
        "an unreadable governed directory must be a scan error, not a silent skip"
    );
}

/// A module whose name is a raw identifier (`mod r#type;`, file `type.rs`) must be
/// governable and its forbidden imports observed — exercising the canonicalization
/// in `check_module_boundary` end to end. The boundary is declared with the *plain*
/// form (`crate::type`) and still matches the raw-identifier source.
#[test]
fn a_raw_identifier_module_is_governed_and_its_import_observed() {
    let ws = TempWorkspace::new("rawid");
    ws.write("lib.rs", "pub mod r#type;\n");
    ws.write("type.rs", "use crate::r#mod::Thing;\n");

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::type")
        .must_not_import("crate::mod")
        .because("a raw-identifier module must be governable");

    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    assert!(
        result.is_ok(),
        "a raw-identifier module must be found, not an unknown-module error: {result:?}"
    );
    assert_eq!(
        violations.len(),
        1,
        "the forbidden import from inside the raw-identifier module must be observed: {violations:?}"
    );
    assert_eq!(violations[0].target, "crate::type");
    assert_eq!(violations[0].finding, "crate::mod::Thing");
}

#[test]
fn module_boundary_uses_the_package_target_src_path() {
    let ws = TempWorkspace::new("custom-lib-path");
    let root = ws.write_at("lib.rs", "pub mod kernel;\n");
    ws.write_at("kernel.rs", "use crate::io::Sink;\n");

    let manifest = ws.dir().join("Cargo.toml");
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "manifest_path": manifest.to_string_lossy().into_owned(),
            "dependencies": [],
            "targets": [{
                "kind": ["lib"],
                "src_path": root.to_string_lossy().into_owned()
            }]
        }]
    });
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_import("crate::io")
        .because("module boundaries must scan the compiled source root");

    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);

    assert!(
        result.is_ok(),
        "a custom [lib] path must not be misresolved to manifest_dir/src: {result:?}"
    );
    assert_eq!(
        violations.len(),
        1,
        "the forbidden import under the custom source root must be observed"
    );
    assert_eq!(violations[0].finding, "crate::io::Sink");
}

#[test]
fn path_remapped_module_is_followed_not_governed_via_a_conventional_orphan() {
    // rustc ground truth: `#[path = "weird.rs"] pub mod kernel;` compiles `weird.rs` as
    // `crate::kernel` (verified with a real `cargo build`), never the same-named conventional
    // orphan `kernel.rs`. The boundary must react on the REAL target's import, naming it as the
    // offending file, and must never react on the orphan's (different) import — a same-named
    // orphan is not compiled, so its content must never surface as this module's finding.
    let (result, violations) = run_module_check(
        "path-remap-boundary",
        &[
            ("lib.rs", "#[path = \"weird.rs\"]\npub mod kernel;\n"),
            ("weird.rs", "use crate::projection::Thing;\n"),
            ("kernel.rs", "use crate::projection::Wrong;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::projection")
            .because("closing the #[path]-following divergence from 渾儀/漏刻"),
    );

    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::kernel");
    assert_eq!(
        violations[0].finding, "crate::projection::Thing",
        "the real target's import is observed, never the orphan's: {violations:?}"
    );
    let file = violations[0]
        .file
        .as_deref()
        .expect("a module-import violation carries its source file");
    assert!(
        file.ends_with("weird.rs"),
        "the violation names the real #[path] target, not the conventional orphan: {file}"
    );
}

/// An unconditional `#[path = "…"]` preceding an INLINE module header is not a no-op: it
/// relocates the base directory the inline body's OWN file-form children resolve from, exactly
/// like a file-form `#[path]`. Verified against a real `cargo check`: `#[path = "thread_files"]
/// pub mod thread { pub mod local_data; }` compiles `thread_files/local_data.rs` as
/// `crate::thread::local_data`, with no `src/thread/` directory at all — the naive
/// (non-relocated) location `thread/local_data.rs` does not even exist. Before this fix the
/// scanner treated the preceding `#[path]` as a pure no-op and always looked in the naive
/// location, silently finding nothing and leaving the real file's imports unobserved.
#[test]
fn an_unconditional_path_on_an_inline_module_relocates_its_own_file_form_children() {
    let (result, violations) = run_module_check(
        "inline-path-relocate",
        &[
            (
                "lib.rs",
                "#[path = \"thread_files\"]\npub mod thread {\n    pub mod local_data;\n}\n\
                     pub mod secret { pub struct Thing; }\n",
            ),
            ("thread_files/local_data.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::thread")
            .must_not_import("crate::secret")
            .because("an inline module's #[path] must relocate its own children, not no-op"),
    );
    result.expect("the relocated child must be a valid, governable target");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::thread");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a module-import violation carries its source file");
    assert!(
        file.ends_with("thread_files/local_data.rs")
            || file.ends_with("thread_files\\local_data.rs"),
        "the violation must name the #[path]-relocated file, not a naive thread/local_data.rs: {file}"
    );
}

/// An inline `mod kernel { … }` is reachable but owns no source file, so it cannot
/// be a governed target (targets are file-based). The reaction must fail loud (exit 2)
/// with a *self-describing* error that names the inline cause — not the misleading
/// "not found among the reachable modules", which would suggest a typo. A genuinely
/// unknown module still gets the "not found" message.
#[test]
fn an_inline_module_target_is_a_self_describing_constitution_error() {
    let ws = TempWorkspace::new("inline");
    ws.write(
        "lib.rs",
        "pub mod kernel { use crate::projection::Thing; }\npub mod projection { pub struct Thing; }\n",
    );

    let metadata = ws.metadata("app");

    let inline = ModuleBoundary::in_crate("app")
        .module("crate::kernel")
        .must_not_import("crate::projection")
        .because("the kernel must not import a projection");
    let mut violations = Vec::new();
    let inline_err = check_module_boundary(&metadata, &inline, &mut violations)
        .expect_err("an inline target must be a constitution error");
    // Assert against the single-source constructor, not a brittle substring: the
    // inline target reports the inline cause, never the unknown-module message.
    assert_eq!(
        inline_err,
        inline_module_target_error("crate::kernel", "app", "kernel")
    );
    assert_ne!(inline_err, unknown_module_error("crate::kernel", "app"));

    // A genuinely unknown module path still gets the unknown-module message.
    let typo = ModuleBoundary::in_crate("app")
        .module("crate::ghost")
        .must_not_import("crate::projection")
        .because("typo");
    let typo_err = check_module_boundary(&metadata, &typo, &mut violations)
        .expect_err("an unknown module is a constitution error");
    assert_eq!(typo_err, unknown_module_error("crate::ghost", "app"));
}

/// The inline-target constitution error must hold **even when a same-named conventional orphan
/// file** sits beside the inline body. Rust compiles the inline body and never the orphan, so
/// governing the orphan (and silently missing the inline body's imports) is a false negative —
/// the one forbidden bug, and the inline twin of the `#[path]` orphan-shadow hazard. The orphan
/// must not make the inline target look file-backed.
#[test]
fn an_inline_target_with_a_same_named_orphan_file_is_still_a_constitution_error() {
    let (result, _) = run_module_check(
        "inline-orphan",
        &[
            ("lib.rs", "pub mod kernel { use crate::secret::Thing; }\n"),
            // Orphan: Rust never compiles this as `crate::kernel` (the inline body is it).
            ("kernel.rs", "// clean — no forbidden import\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("the kernel must not import a secret"),
    );
    let err = result.expect_err(
        "an inline target must stay the inline constitution error even with a same-named \
             orphan file — governing the orphan and missing the inline body is the forbidden \
             false negative, never a silent pass",
    );
    assert_eq!(
        err,
        inline_module_target_error("crate::kernel", "x", "kernel")
    );
}

/// An orphan beside an inline module contributes **no phantom child module**: the orphan is
/// not compiled, so its own `mod` declarations name no reachable module. Governing such a
/// phantom child is a not-found constitution error, never a silent pass over the orphan's file.
#[test]
fn an_orphan_beside_an_inline_module_contributes_no_phantom_child() {
    let (result, _) = run_module_check(
        "inline-phantom",
        &[
            ("lib.rs", "pub mod kernel { }\n"),
            ("kernel.rs", "pub mod deep;\n"), // orphan's declaration — phantom
            ("kernel/deep.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel::deep")
            .must_not_import("crate::secret")
            .because("deep must not import a secret"),
    );
    let err = result.expect_err("a phantom child of an orphan is not a reachable module");
    assert_eq!(err, unknown_module_error("crate::kernel::deep", "x"));
}

/// Only inline-occupied files are excluded: a genuinely file-backed module (`mod real;` +
/// `real.rs`) is still governed, its imports observed — proving the exclusion is not
/// over-broad.
#[test]
fn a_file_backed_module_is_still_governed() {
    let (result, violations) = run_module_check(
        "file-backed",
        &[
            ("lib.rs", "pub mod real;\n"),
            ("real.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::real")
            .must_not_import("crate::secret")
            .because("real must not import a secret"),
    );
    result.expect("a file-backed module is a valid, governable target");
    assert!(
        !violations.is_empty(),
        "the file-backed module's forbidden import must still be observed"
    );
}

/// A path declared **both** file-form (`mod kernel;`) and inline (`mod kernel { … }`) — which in
/// valid source arises only under mutually-exclusive `#[cfg]` — is NOT inline-only, so its
/// conventional file stays governed. This pins that the inline-only exclusion leaves the
/// existing cfg-blind lexical bound exactly as it was (never turning it into an inline error).
#[test]
fn a_cfg_dual_declared_module_keeps_governing_its_conventional_file() {
    let (result, violations) = run_module_check(
        "cfg-dual",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"k\")]\npub mod kernel;\n\
                     #[cfg(not(feature = \"k\"))]\npub mod kernel { }\n",
            ),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("kernel must not import a secret"),
    );
    result.expect("a cfg-dual-declared module keeps its conventional file as a valid target");
    assert!(
        !violations.is_empty(),
        "the conventional file must still be observed — the cfg-blind bound is unchanged"
    );
}

/// Stated bound (not a fix): a package that builds a lib AND a bin observes its whole `src/`
/// under one conventional-path tree, so both roots resolve to `crate` and there are no
/// per-target module graphs. A submodule declared inline in one root and file-backed in the
/// other governs the file-backed one; the inline body's imports are NOT observed. Closing it
/// needs per-target graphs (distinguishing the lib crate's `crate::shared` from the bin's) —
/// beyond the conventional-path scanner. Recorded here and in `module-boundary`, never a silent
/// claim of cleanliness.
#[test]
fn a_cross_root_same_named_submodule_is_a_documented_bound() {
    let (result, violations) = run_module_check(
        "cross-root-submodule",
        &[
            ("lib.rs", "pub mod shared { use crate::forbidden::X; }\n"),
            ("main.rs", "pub mod shared;\nfn main() {}\n"),
            ("shared.rs", "// clean — the bin root's shared module\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::shared")
            .must_not_import("crate::forbidden")
            .because("shared must not import forbidden"),
    );
    result.expect("a file-backed shared (via the bin root) is a valid target");
    assert!(
        violations.is_empty(),
        "documented lib+bin bound: the lib root's inline `mod shared` body is not observed \
             (shared.rs is governed instead) — recorded, not silently claimed clean: {violations:?}"
    );
}

/// A plain `mod child;` backed by BOTH `child.rs` and `child/mod.rs` at once is a genuine rustc
/// compile error (E0761) — closes a pre-existing debt: both forms were previously silently
/// accepted as separate sources (dual-governed), the mirror image of the missing-file gap.
/// Mirrors 漏刻's own `resolve_external_module`'s identical hard error (see
/// `dual_backed_module_conformance.rs` for the cross-dimension agreement pin).
#[test]
fn a_dual_backed_module_is_a_scan_error_not_silently_accepted() {
    let (result, _violations) = run_module_check(
        "dual-backed",
        &[
            ("lib.rs", "pub mod child;\n"),
            ("child.rs", "// flat form\n"),
            ("child/mod.rs", "// nested form\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("child must not import forbidden"),
    );
    let err = result.expect_err(
        "both conventional forms present is a genuine ambiguity, never a silent accept",
    );
    assert!(
        err.contains("resolves to both") && err.contains("child.rs") && err.contains("mod.rs"),
        "the error must name the ambiguity and both real files: {err}"
    );
}

/// A plain `mod child;` with NEITHER `child.rs` NOR `child/mod.rs` present, and no `#[cfg]`
/// anywhere on the declaration, is a genuine rustc compile error — closes the longstanding
/// "missing plain mod file is a silent gap" debt (BACKLOG: "圭表 gaining `#[cfg]` awareness for an
/// unrelated reason... closes this for free"). Previously `child` silently vanished from
/// `reachable` with no error, an undetected coverage gap; now matches 渾儀's own hard error for
/// the identical shape.
#[test]
fn an_unconditional_missing_plain_module_file_is_a_scan_error_not_a_silent_gap() {
    let (result, _violations) = run_module_check(
        "missing-plain-unconditional",
        &[("lib.rs", "pub mod child;\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("child must not import forbidden"),
    );
    let err = result.expect_err(
        "an unconditional plain mod with no backing file must fail loud, never silently vanish",
    );
    assert!(
        err.contains("crate::child") && err.contains("could not be located"),
        "the error must name the module and the missing-file cause: {err}"
    );
}

/// A BARE `#[cfg(...)]`-gated plain `mod child;` with no backing file is tolerated BY THE
/// SCANNER — an unrelated sibling boundary still resolves cleanly rather than the whole scan
/// erroring merely because one cfg-gated module has no file on this build/feature set (matching
/// 渾儀's `has_cfg_attr` tolerance and 漏刻's own `a_cfg_gated_module_with_no_file_is_skipped_not_errored`).
#[test]
fn a_cfg_gated_missing_plain_module_file_does_not_fail_an_unrelated_boundary() {
    let (result, violations) = run_module_check(
        "missing-plain-cfg-gated",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"absent\")]\npub mod child;\npub mod present;\n",
            ),
            ("present.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::present")
            .must_not_import("crate::forbidden")
            .because("present must not import forbidden"),
    );
    result.expect("a #[cfg]-gated missing sibling must not fail an unrelated boundary");
    assert_eq!(
        violations.len(),
        1,
        "the unrelated boundary must still observe its own real violation: {violations:?}"
    );
}

/// A boundary anchored DIRECTLY at a module whose sole declaration was `#[cfg]`-tolerated away
/// (no surviving file) is "cannot judge," not a vacuous clean pass — matching 渾儀's own `descend`
/// precedent for the identical shape (its empty-branches case also falls to
/// `unknown_module_error`, never silently reporting zero violations for something never checked).
#[test]
fn a_boundary_anchored_directly_at_a_cfg_gated_missing_module_is_unknown_not_clean() {
    let (result, _violations) = run_module_check(
        "missing-plain-cfg-gated-anchor",
        &[("lib.rs", "#[cfg(feature = \"absent\")]\npub mod child;\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("child must not import forbidden"),
    );
    let err = result.expect_err(
        "anchoring directly at a module absent on this build must fail loud, never vacuously pass",
    );
    assert_eq!(err, unknown_module_error("crate::child", "x"));
}

/// A mutually-exclusive `#[cfg]` shim pairing an inline arm with a plain-file arm whose file is
/// tolerated-away-missing must still report the SELF-DESCRIBING `inline_module_target_error`
/// ("declared inline... move it into its own file"), not the generic `unknown_module_error`
/// ("check the path", which wrongly implies a typo). Found on this session's own round-2
/// adversarial review: the bare-`#[cfg]` tolerance above made it newly possible for a plain
/// declaration to be *declared* yet resolve to nothing, and `inline_only`'s gating on mere
/// declaration presence (rather than actual resolution) then wrongly excluded this module from
/// `inline_only`, misreporting which error applies.
#[test]
fn an_inline_arm_paired_with_a_tolerated_away_plain_arm_still_reports_the_inline_error() {
    let (result, _violations) = run_module_check(
        "inline-plus-tolerated-plain",
        &[(
            "lib.rs",
            "#[cfg(unix)]\npub mod engine { pub struct A; }\n\
             #[cfg(windows)]\npub mod engine;\n",
        )],
        ModuleBoundary::in_crate("x")
            .module("crate::engine")
            .must_not_import("crate::forbidden")
            .because("engine must not import forbidden"),
    );
    let err = result.expect_err(
        "an inline arm alongside a tolerated-away plain arm is still an inline target, not unknown",
    );
    assert_eq!(
        err,
        inline_module_target_error("crate::engine", "x", "engine")
    );
}

/// A BARE `#[cfg(pred)]` co-occurring with an unconditional `#[path = "…"]` on the same item
/// removes the whole item, `#[path]` included, when `pred` is false — a standard per-platform
/// shim (`#[cfg(windows)] #[path = "windows_impl.rs"] mod imp;`) that must not hard-error an
/// unrelated boundary merely because this platform's target file was never written. Verified
/// against a real `rustc` build: this compiles cleanly with the target entirely absent.
#[test]
fn a_cfg_gated_unconditional_path_target_does_not_fail_an_unrelated_boundary_when_missing() {
    let (result, violations) = run_module_check(
        "cfg-gated-path-target-missing",
        &[
            (
                "lib.rs",
                "#[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod imp;\npub mod present;\n",
            ),
            ("present.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::present")
            .must_not_import("crate::forbidden")
            .because("present must not import forbidden"),
    );
    result.expect("a #[cfg]-gated #[path] target with no file must not fail an unrelated boundary");
    assert_eq!(
        violations.len(),
        1,
        "the unrelated boundary must still observe its own real violation: {violations:?}"
    );
}

/// The bare-`#[cfg]` tolerance for a missing unconditional `#[path]` target must not depend on
/// attribute order: `#[path]` written BEFORE `#[cfg]` (the reverse of the sibling test above)
/// must be tolerated identically — mirroring the existing
/// `an_unconditional_path_attr_wins_regardless_of_cfg_attr_order` guarantee for the `#[path]`
/// detector itself.
#[test]
fn a_cfg_gated_unconditional_path_target_is_tolerated_regardless_of_attribute_order() {
    let (result, violations) = run_module_check(
        "cfg-gated-path-target-order",
        &[
            (
                "lib.rs",
                "#[path = \"windows_impl.rs\"]\n#[cfg(windows)]\npub mod imp;\npub mod present;\n",
            ),
            ("present.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::present")
            .must_not_import("crate::forbidden")
            .because("present must not import forbidden"),
    );
    result.expect("attribute order must not affect the bare-#[cfg] tolerance");
    assert_eq!(
        violations.len(),
        1,
        "the unrelated boundary must still observe its own real violation: {violations:?}"
    );
}

/// A `#[cfg_attr(cond, path = …)]` IS recognized as a (conditional) remap, the same
/// stated `#[path]` bound as the separate `#[cfg(cond)] #[path = …]` spelling. Not recognizing it
/// would govern the conventionally-named file — a cfg-blind mishandling that is a
/// false POSITIVE when the cfg path is active (rustc compiles the remap target, not the
/// conventional file) and, when no conventional file exists, a silent false NEGATIVE (the real
/// remapped source never scanned). The remapped module is out of scope: a boundary on it
/// fails loud (exit 2, "cannot judge") rather than guessing a file, and the conventional file is
/// not silently governed as the wrong module.
#[test]
fn a_cfg_attr_wrapped_path_is_recognized_as_a_remap() {
    let (result, _violations) = run_module_check(
        "cfg-attr-path",
        &[
            (
                "lib.rs",
                "#[cfg_attr(unix, path = \"weird.rs\")]\npub mod foo;\n",
            ),
            ("foo.rs", "use crate::forbidden::Y;\n"),
            ("weird.rs", "// the cfg(unix) remap target, clean\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::foo")
            .must_not_import("crate::forbidden")
            .because("foo must not import forbidden"),
    );
    // crate::foo is a remapped module, out of scope — so a boundary anchored on it is a constitution
    // error (exit 2), matching a direct `#[path]` remap. The conventional foo.rs is NOT governed as
    // crate::foo (no false positive on the active-cfg configuration, no silent guess).
    assert!(
        result.is_err(),
        "a cfg_attr-remapped module is out of scope; a boundary on it must fail loud, not guess \
         the conventional file"
    );
}

/// Run a module boundary against a synthetic one-package workspace whose `src`
/// holds `files` (each `(relative path, contents)`), under a unique temp dir keyed
/// by `name`. Returns the check result and the collected violations.
fn run_module_check(
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

/// Like [`run_module_check`], but with declared dependencies in the synthesized `cargo metadata`
/// package — needed by the strict-external inline confinement, whose head ladder matches a bare
/// head against declared dependency import identifiers. Each dep is `(name, rename)`: `rename` is
/// the Cargo `pkg = { package = "…" }` alias when `Some` (the `-`→`_` fold is applied by the
/// reader under test, so pass names verbatim).
fn run_module_check_with_deps(
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

fn restrict_kernel_to_types(governed: &str, allowed: &[&str]) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module(governed)
        .restrict_imports_to(allowed.to_vec())
        .because("the kernel may import only the allowed modules")
}

#[test]
fn restrict_imports_to_flags_an_import_outside_the_allowlist() {
    let (result, violations) = run_module_check(
        "restrict-outside",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::io::Sink;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::kernel");
    assert_eq!(violations[0].finding, "crate::io::Sink");
}

#[test]
fn a_module_violation_carries_its_offending_file() {
    // The offending import sits in kernel.rs; the violation names that source file so an
    // agent knows where to repair — a faithful byproduct of the scan, not a new observation.
    let (result, violations) = run_module_check(
        "module-file",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::io::Sink;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a module-import violation carries its source file");
    assert!(
        file.ends_with("kernel.rs"),
        "file names the offending source: {file}"
    );
}

#[test]
fn a_module_backed_by_two_files_yields_one_violation_with_a_file() {
    // `crate` is backed by both lib.rs and main.rs (a lib+bin package); the same forbidden
    // import in each must still collapse to exactly one violation (the file is attached
    // after collapsing by identity, never a de-dup key), and that one carries a file.
    let (result, violations) = run_module_check(
        "module-two-files",
        &[
            ("lib.rs", "use crate::forbidden::Thing;\n"),
            ("main.rs", "use crate::forbidden::Thing;\nfn main() {}\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("crate must not import forbidden"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "two files of one module collapse to one violation: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
    assert!(
        violations[0].file.is_some(),
        "the surviving violation carries a representative file"
    );
}

#[test]
fn restrict_imports_to_is_clean_within_the_allowlist() {
    let (result, violations) = run_module_check(
        "restrict-within",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::types::Id;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn restrict_imports_to_allows_the_governed_modules_own_subtree() {
    // The exact module (`crate::kernel`), a descendant, and a `self::` import all
    // resolve within the governed subtree and are not outward edges — so none need
    // to be listed in the allowlist.
    let (result, violations) = run_module_check(
        "restrict-ownsubtree",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            (
                "kernel.rs",
                "use crate::kernel;\nuse crate::kernel::detail::Thing;\nuse self::other::Thing2;\n",
            ),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(violations.is_empty(), "own-subtree imports: {violations:?}");
}

#[test]
fn restrict_imports_to_with_an_empty_allowlist_forbids_outward_imports() {
    let (result, violations) = run_module_check(
        "restrict-empty",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::types::Id;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &[]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::types::Id");
}

#[test]
fn restrict_imports_to_does_not_treat_a_prefix_colliding_sibling_as_allowed() {
    // The `::`-delimited containment must not let `crate::types_extra` ride in on the
    // `crate::types` allowlist entry — the headline regression guard.
    let (result, violations) = run_module_check(
        "restrict-sibling",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            (
                "kernel.rs",
                "use crate::types::Id;\nuse crate::types_extra::Y;\n",
            ),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "only the sibling violates: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::types_extra::Y");
}

#[test]
fn restrict_imports_to_never_flags_an_external_import() {
    let (result, violations) = run_module_check(
        "restrict-external",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use serde::Deserialize;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &[]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "externals are out of scope: {violations:?}"
    );
}

#[test]
fn restrict_imports_to_governs_a_super_reaching_outward_import() {
    let (result, violations) = run_module_check(
        "restrict-super",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use super::other::Thing;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding, "crate::other::Thing",
        "super:: resolves to an absolute outward path that is governed"
    );
}

#[test]
fn restrict_imports_to_canonicalizes_a_raw_identifier_allowlist_entry() {
    let (result, violations) = run_module_check(
        "restrict-rawid",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::r#type::Thing;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::r#type"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the raw-id entry canonicalizes to match the import: {violations:?}"
    );
}

#[test]
fn restrict_imports_to_on_the_crate_root_is_a_constitution_error() {
    // The crate root has no outward internal edge, so the rule could never react —
    // fail loud (exit 2), never silently pass.
    let (result, _violations) = run_module_check(
        "restrict-crate",
        &[("lib.rs", "use crate::anything::X;\n")],
        restrict_kernel_to_types("crate", &["crate::types"]),
    );
    let err = result.expect_err("governing `crate` must be a constitution error");
    assert_eq!(err, restrict_imports_to_on_crate_error("x"));
}

#[test]
fn restrict_imports_to_honors_warn_severity_and_its_distinct_label() {
    let (result, violations) = run_module_check(
        "restrict-warn",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::io::Sink;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .restrict_imports_to(["crate::types"])
            .warn()
            .because("the kernel should import only types"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
    // A distinct rule family and semantic key prevent baseline identity collision.
    assert_eq!(violations[0].rule, "restrict imports to");
}

fn protect_internal_from(importer: &str) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::internal")
        .must_not_be_imported_by(importer)
        .because("internal is private to its layer")
}

#[test]
fn must_not_be_imported_by_flags_the_forbidden_importer_only() {
    let (result, violations) = run_module_check(
        "inbound-basic",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\npub mod api;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::Secret;\n"),
            ("api.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    // Only crate::http is beneath the forbidden importer; crate::api imports internal
    // too but is outside crate::http, so it is clean.
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::internal");
    assert_eq!(violations[0].finding, "crate::http");
    assert_eq!(violations[0].rule, "module must not be imported by");
}

#[test]
fn must_not_be_imported_by_flags_an_inline_module_importer() {
    // `crate::http` is an INLINE module in lib.rs, not a file. Its `use crate::internal`
    // is attributed to the inline importer `crate::http`, not the file's module `crate`, so the
    // forbidden inbound edge reacts. File-granular attribution would test `crate` against
    // the forbidden importer, pre-filter the file out, and silently miss the edge.
    let (result, violations) = run_module_check(
        "inbound-inline-importer",
        &[
            (
                "lib.rs",
                "pub mod internal;\nmod http { use crate::internal::Secret; }\n",
            ),
            ("internal.rs", "// protected\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::internal");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_applies_beneath_the_importer() {
    let (result, violations) = run_module_check(
        "inbound-beneath-importer",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "pub mod v1;\n"),
            ("http/v1.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding, "crate::http::v1",
        "the importer beneath crate::http is named"
    );
}

#[test]
fn must_not_be_imported_by_applies_beneath_the_protected_module() {
    let (result, violations) = run_module_check(
        "inbound-beneath-protected",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::deep::Thing;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "an import beneath the protected module violates: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_ignores_prefix_colliding_siblings_on_both_sides() {
    let (result, violations) = run_module_check(
        "inbound-collision",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod http;\npub mod httpx;\n",
            ),
            ("internal.rs", "// protected\n"),
            // forbidden importer is crate::http; crate::http imports a sibling of the
            // protected module (internal_util), which is clean.
            ("http.rs", "use crate::internal_util::X;\n"),
            // crate::httpx is a sibling of the forbidden importer; importing internal
            // is clean.
            ("httpx.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "::-delimited containment must not match siblings on either side: {violations:?}"
    );
}

#[test]
fn must_not_be_imported_by_does_not_flag_the_protected_modules_own_subtree() {
    let (result, violations) = run_module_check(
        "inbound-own-subtree",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "pub mod b;\n"),
            // crate::a::b is the protected module; it imports its own subtree and sits
            // beneath the forbidden importer crate::a — but a module importing itself
            // is not an inbound edge, so it is clean.
            ("a/b.rs", "use crate::a::b::detail::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::a::b")
            .must_not_be_imported_by("crate::a")
            .because("a::b is private"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the protected module's own subtree is not an importer: {violations:?}"
    );
}

#[test]
fn must_not_be_imported_by_ignores_external_imports() {
    let (result, violations) = run_module_check(
        "inbound-external",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use serde::Deserialize;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "externals are out of scope: {violations:?}"
    );
}

#[test]
fn must_not_be_imported_by_crate_forbids_every_outside_importer() {
    let (result, violations) = run_module_check(
        "inbound-x-crate",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate"),
    );
    assert!(result.is_ok(), "{result:?}");
    // Forbidding importer `crate` means nobody outside internal's own subtree may
    // import it; crate::http violates, internal's own files stay clean.
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inbound-m-crate",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_be_imported_by("crate::http")
            .because("the crate root cannot be protected this way"),
    );
    let err = result.expect_err("protecting `crate` must be a constitution error");
    assert_eq!(err, must_not_be_imported_by_on_crate_error("x"));
}

#[test]
fn must_not_be_imported_by_dedups_multiple_imports_from_one_importer() {
    let (result, violations) = run_module_check(
        "inbound-dedup",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            (
                "http.rs",
                "use crate::internal::A;\nuse crate::internal::B;\n",
            ),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one offending importer module yields one violation: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_honors_warn_severity() {
    let (result, violations) = run_module_check(
        "inbound-warn",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::internal::Secret;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_not_be_imported_by("crate::http")
            .warn()
            .because("internal should be private"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
}

#[test]
fn must_not_be_imported_by_projects_its_importer() {
    let constitution = Constitution::new("p").boundary(
        ModuleBoundary::in_crate("app")
            .module("crate::internal")
            .must_not_be_imported_by("crate::http")
            .because("internal is private to its layer"),
    );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("must not be imported by crate::http"),
        "{text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "module must not be imported by"
    );
    assert_eq!(doc["boundaries"][0]["target"], "crate::internal");
    // The declared forbidden importer projects as `importer`; no `forbidden`/`only`.
    assert_eq!(doc["boundaries"][0]["importer"], "crate::http");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    assert!(doc["boundaries"][0]["only"].is_null());
}

#[test]
fn must_not_be_imported_by_unknown_protected_module_is_a_constitution_error() {
    // The protected-module validation must fire for the inbound rule too: an unknown
    // `m` is exit 2 before any scan, never a silent clean.
    let (result, _violations) = run_module_check(
        "inbound-unknown-m",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::nope")
            .must_not_be_imported_by("crate::http")
            .because("typo target"),
    );
    let err = result.expect_err("an unknown protected module is a constitution error");
    assert_eq!(err, unknown_module_error("crate::nope", "x"));
}

#[test]
fn must_not_be_imported_by_inline_protected_module_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inbound-inline-m",
        &[
            (
                "lib.rs",
                "pub mod kernel { pub struct K; }\npub mod http;\n",
            ),
            ("http.rs", "// nothing\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_be_imported_by("crate::http")
            .because("inline target"),
    );
    let err = result.expect_err("an inline protected module is a constitution error");
    assert_eq!(
        err,
        inline_module_target_error("crate::kernel", "x", "kernel")
    );
}

#[test]
fn must_not_be_imported_by_matches_a_raw_identifier_importer() {
    // The forbidden importer is declared with a raw identifier; the importing file's
    // module canonicalizes to the same path, so the violation still fires (guards the
    // canonicalization lockstep against a false negative).
    let (result, violations) = run_module_check(
        "inbound-rawid-importer",
        &[
            ("lib.rs", "pub mod internal;\npub mod r#async;\n"),
            ("internal.rs", "// protected\n"),
            ("async.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::r#async"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::async");
}

#[test]
fn must_not_be_imported_by_protects_a_raw_identifier_module() {
    let (result, violations) = run_module_check(
        "inbound-rawid-protected",
        &[
            ("lib.rs", "pub mod r#type;\npub mod http;\n"),
            ("type.rs", "// protected\n"),
            ("http.rs", "use crate::r#type::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::r#type")
            .must_not_be_imported_by("crate::http")
            .because("type is private"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::type");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_flags_a_mod_rs_backed_importer() {
    let (result, violations) = run_module_check(
        "inbound-modrs",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http/mod.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
fn must_not_be_imported_by_orders_multiple_offenders_deterministically() {
    let (result, violations) = run_module_check(
        "inbound-multi",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod zeta;\npub mod alpha;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("zeta.rs", "use crate::internal::Secret;\n"),
            ("alpha.rs", "use crate::internal::Secret;\n"),
        ],
        protect_internal_from("crate"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(
        findings,
        ["crate::alpha", "crate::zeta"],
        "multiple offenders are sorted deterministically"
    );
}

#[test]
fn must_not_be_imported_by_dedups_an_importer_backed_by_lib_and_main() {
    // A lib+bin package has both `lib.rs` and `main.rs` at module `crate`. With
    // `must_not_be_imported_by("crate")`, both root files importing the protected
    // module would push `crate` twice — the spec's dedup must collapse it to one.
    let (result, violations) = run_module_check(
        "inbound-lib-and-main",
        &[
            (
                "lib.rs",
                "pub mod internal;\nuse crate::internal::Secret;\n",
            ),
            ("main.rs", "use crate::internal::Secret;\n"),
            ("internal.rs", "// protected\n"),
        ],
        protect_internal_from("crate"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one offending importer module, even when backed by two root files: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate");
    // The collapsed inbound violation still carries a representative file (the inbound path
    // also collects (key, file) before de-duplication), so the two-file case is locked on
    // the inbound rule, not only the outbound one.
    assert!(
        violations[0].file.is_some(),
        "the surviving inbound violation carries a representative file"
    );
}

#[test]
fn must_not_import_dedups_a_finding_across_subtree_files() {
    // crate::kernel spans kernel.rs + kernel/sub.rs; both import the forbidden module.
    // The same finding must be reported once, not once per file.
    let (result, violations) = run_module_check(
        "dedup-mni-subtree",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "pub mod sub;\nuse crate::forbidden::X;\n"),
            ("kernel/sub.rs", "use crate::forbidden::X;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::forbidden")
            .because("kernel must not import forbidden"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one violation per distinct finding: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::forbidden::X");
}

#[test]
fn restrict_imports_to_dedups_a_finding_across_subtree_files() {
    let (result, violations) = run_module_check(
        "dedup-rit-subtree",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "pub mod sub;\nuse crate::io::Sink;\n"),
            ("kernel/sub.rs", "use crate::io::Sink;\n"),
        ],
        restrict_kernel_to_types("crate::kernel", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one violation per distinct finding: {violations:?}"
    );
    assert_eq!(violations[0].finding, "crate::io::Sink");
}

#[test]
fn outbound_dedup_collapses_identical_findings_but_keeps_distinct_ones() {
    // Two subtree files: one imports X, the other imports X (duplicate) and Y.
    // Result must be {X, Y} — the identical finding collapsed, the distinct one kept.
    let (result, violations) = run_module_check(
        "dedup-distinct",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "pub mod sub;\nuse crate::forbidden::X;\n"),
            (
                "kernel/sub.rs",
                "use crate::forbidden::X;\nuse crate::forbidden::Y;\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::forbidden")
            .because("kernel must not import forbidden"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(
        findings,
        ["crate::forbidden::X", "crate::forbidden::Y"],
        "{violations:?}"
    );
    // And no two violations share an identity (target, rule, finding).
    let mut ids: Vec<_> = violations
        .iter()
        .map(|v| (&v.target, &v.rule, &v.finding))
        .collect();
    let before = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), before, "no duplicate violation identities");
}

#[test]
fn restrict_imports_to_does_not_flag_an_over_popped_super() {
    // `crate::a` over-pops with `super::super`; the path names no internal module, so
    // it must not be observed — and must not be mistaken for an outward edge that the
    // allowlist would flag (the regression this guards).
    let (result, violations) = run_module_check(
        "restrict-super-overflow",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "use super::super::other::X;\n"),
        ],
        restrict_kernel_to_types("crate::a", &["crate::types"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an over-popped super is not an outward edge: {violations:?}"
    );
}

#[test]
fn baseline_round_trips_through_json() {
    let report = one_enforce_violation();
    let json = Baseline::of(&report).to_json();
    let parsed = Baseline::from_json(&json).expect("a written baseline parses");
    assert!(
        parsed.contains(&report.violations[0]),
        "round-trip must preserve the violation identity"
    );
}

#[test]
fn from_json_rejects_malformed_and_unknown_version() {
    assert!(Baseline::from_json("not json").is_err());
    assert!(Baseline::from_json(r#"{"version":3,"violations":[]}"#).is_err());
    assert!(
        Baseline::from_json(r#"{"violations":[]}"#).is_err(),
        "a missing version must be an error, not a silent empty baseline"
    );
}

#[test]
fn a_baselined_enforce_violation_does_not_fail() {
    let mut report = one_enforce_violation();
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    assert!(report.violations[0].baselined);
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        0,
        "a fully baselined run must not fail"
    );
}

#[test]
fn a_new_enforce_violation_fails_against_a_baseline() {
    let baseline = Baseline::from_json(
            r#"{"version":1,"violations":[{"target":"core","rule":"deny external dependencies","finding":"other"}]}"#,
        )
        .unwrap();
    let mut report = one_enforce_violation();
    apply_baseline(&mut report, &baseline);
    assert!(
        !report.violations[0].baselined,
        "serde is not in the baseline"
    );
    assert_eq!(Outcome::Violations(report).exit_code(), 1);
}

#[test]
fn stale_finds_entries_with_no_current_match() {
    let report = one_enforce_violation();
    let baseline = Baseline::from_json(
            r#"{"version":1,"violations":[{"target":"core","rule":"deny external dependencies","finding":"gone"}]}"#,
        )
        .unwrap();
    let stale = baseline.stale(&report);
    assert_eq!(stale.len(), 1);
    assert_eq!(stale[0].finding, "gone");
}

#[test]
fn report_json_projects_a_violation_with_its_kind() {
    let json = report_json(&Outcome::Violations(one_enforce_violation()), &[], None);
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(doc["outcome"], "violations");
    assert_eq!(doc["exit_code"], 1);
    let violation = &doc["violations"][0];
    assert_eq!(violation["kind"], "crate");
    assert_eq!(violation["finding"], "serde");
    assert_eq!(
        violation["finding_key"]["namespace"],
        "tianheng.fact/guibiao/dependency"
    );
    assert_eq!(violation["finding_key"]["code"], "dependency-edge");
    assert_eq!(violation["finding_key"]["fields"]["package"], "serde");
    assert_eq!(violation["severity"], "enforce");
    assert_eq!(violation["baselined"], false);
    // `reason` is the repair hint; there is no separate field.
    assert!(violation["reason"].as_str().is_some_and(|r| !r.is_empty()));
    assert!(doc.get("repair_hint").is_none());
}

#[test]
fn report_json_renders_clean_and_constitution_error() {
    let clean: serde_json::Value =
        serde_json::from_str(&report_json(&Outcome::Clean, &[], None)).unwrap();
    assert_eq!(clean["outcome"], "clean");
    assert_eq!(clean["exit_code"], 0);
    assert_eq!(clean["violations"].as_array().unwrap().len(), 0);
    assert!(clean.get("coverage").is_none(), "no coverage when None");

    let error: serde_json::Value = serde_json::from_str(&report_json(
        &Outcome::ConstitutionError("boom".into()),
        &[],
        None,
    ))
    .unwrap();
    assert_eq!(error["outcome"], "constitution_error");
    assert_eq!(error["exit_code"], 2);
    assert_eq!(error["error"], "boom");
}

#[test]
fn report_json_reflects_baseline_and_stale_in_gate() {
    let mut report = one_enforce_violation();
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    // A baseline entry that no current violation matches is stale.
    let stale = vec![test_id("core", "deny external dependencies", "gone")];
    let doc: serde_json::Value =
        serde_json::from_str(&report_json(&Outcome::Violations(report), &stale, None)).unwrap();
    assert_eq!(doc["exit_code"], 0, "a fully baselined run does not fail");
    assert_eq!(doc["violations"][0]["baselined"], true);
    assert_eq!(doc["stale_baseline"][0]["finding"], "gone");
    assert!(doc["stale_baseline"][0]["finding_key"].is_object());

    let legacy = Baseline::from_json(
        r#"{"version":1,"violations":[{
            "target":"core","rule":"deny external dependencies","finding":"legacy-gone"
        }]}"#,
    )
    .unwrap();
    let legacy_stale: Vec<ViolationId> = legacy
        .stale(&Report::empty())
        .into_iter()
        .cloned()
        .collect();
    let legacy_doc: serde_json::Value =
        serde_json::from_str(&report_json(&Outcome::Clean, &legacy_stale, None)).unwrap();
    assert_eq!(legacy_doc["stale_baseline"][0]["finding"], "legacy-gone");
    assert_eq!(legacy_doc["stale_baseline"][0]["finding_key"], Value::Null);
}

#[test]
fn report_json_includes_coverage_when_present() {
    let coverage = Coverage {
        total: 3,
        uncovered: vec!["memory".to_string()],
    };
    let doc: serde_json::Value =
        serde_json::from_str(&report_json(&Outcome::Clean, &[], Some(&coverage))).unwrap();
    assert_eq!(doc["coverage"]["workspace_crates"], 3);
    assert_eq!(doc["coverage"]["uncovered"][0], "memory");
}

#[test]
fn external_classification_treats_any_non_null_source_as_external() {
    // A path/internal dep has a null `source`; registry, git, and alternative
    // (sparse) registry deps all have a non-null source and must be classified
    // external. The sparse case is the regression guard: a fixed `registry+`/
    // `git+` prefix list would silently pass an alternative `sparse+` registry.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "internal", "source": null, "kind": null },
            {
                "name": "crates_io",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "git_dep", "source": "git+https://example.com/x", "kind": null },
            { "name": "alt_sparse", "source": "sparse+https://my.registry/index/", "kind": null },
            {
                "name": "a_dev",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": "dev"
            },
        ]
    });
    assert_eq!(
        external_dependencies(&package, DependencyKind::Normal),
        vec![
            "alt_sparse".to_string(),
            "crates_io".to_string(),
            "git_dep".to_string(),
        ],
        "every non-null-source normal dep is external (incl. a sparse alt \
             registry); the null-source internal dep and the dev dep are excluded",
    );
}

#[test]
fn a_crate_violation_reports_no_file() {
    // A crate-dependency violation is an edge in the dependency graph (a manifest
    // relation), not a source line, so its `file` is a faithful `None`.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "core",
            "dependencies": [
                {
                    "name": "serde",
                    "source": "registry+https://github.com/rust-lang/crates.io-index",
                    "kind": null
                }
            ],
        }]
    });
    let boundary = CrateBoundary::crate_("core")
        .deny_external_dependencies()
        .because("core stays dependency-light");
    let mut violations = Vec::new();
    let result = check_crate_boundary(&metadata, &[], &boundary, &mut violations);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].kind, BoundaryKind::Crate);
    assert_eq!(violations[0].finding, "serde");
    assert!(
        violations[0].file.is_none(),
        "a crate-dependency violation has no single source file"
    );
}

#[test]
fn dedup_keeps_the_more_severe_of_duplicate_violations() {
    // The same crate rule declared once `warn` and once `enforce` on one crate — a plausible
    // mid-promotion state — flags the same dependency twice with equal `(target, rule, finding)`
    // identity but different severity. Deduping must keep the ENFORCE reaction: keeping the
    // first-seen `warn` would collapse to an advisory and drop exit-1 to exit-0 (a false
    // negative). Verified in both declaration orders (the fix is order-independent).
    let metadata = serde_json::json!({
        "packages": [{
            "name": "core",
            "dependencies": [
                { "name": "serde", "source": "registry+x", "kind": null }
            ],
        }]
    });
    let warn = || {
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .warn()
            .because("observing before enforcing")
    };
    let enforce = || {
        CrateBoundary::crate_("core")
            .deny_external_dependencies()
            .because("core stays dependency-light")
    };
    for (first, second) in [(warn(), enforce()), (enforce(), warn())] {
        let constitution = Constitution::new("mid-promotion")
            .boundary(first)
            .boundary(second);
        let outcome = evaluate(&constitution, &metadata);
        let Outcome::Violations(report) = &outcome else {
            panic!("expected violations, got {outcome:?}");
        };
        assert_eq!(
            report.violations.len(),
            1,
            "duplicates collapse to one: {report:?}"
        );
        assert_eq!(
            report.violations[0].severity,
            Severity::Enforce,
            "the more severe reaction is kept"
        );
        assert_eq!(
            outcome.exit_code(),
            1,
            "an enforce violation fails the reaction"
        );
    }
}

#[test]
fn dependency_kind_selects_which_table_is_observed() {
    // `serde` is a normal dep; `proptest` is a dev-dep; `cc` is a build-dep.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "serde", "source": "registry+x", "kind": null },
            { "name": "proptest", "source": "registry+x", "kind": "dev" },
            { "name": "cc", "source": "registry+x", "kind": "build" },
        ]
    });
    let deny = Rule::DenyExternalDependencies { allowed: vec![] };
    // Default (normal) sees only serde; dev sees only proptest; build only cc. The dev/build
    // findings carry a kind suffix so the same dep name in two tables stays a distinct finding.
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Normal),
        vec!["serde".to_string()]
    );
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Dev),
        vec!["proptest (dev)".to_string()]
    );
    assert_eq!(
        deny.findings(&package, &[], DependencyKind::Build),
        vec!["cc (build)".to_string()]
    );
}

#[test]
fn the_same_dep_in_two_tables_yields_distinct_findings() {
    // The one forbidden bug for the dependency family: `serde` from a git source in BOTH the
    // normal and the dev table, governed by same-rule boundaries differing only by kind, must
    // not collapse to one `(target, rule, finding)` — else baselining the normal violation
    // masks a new dev one. The kind suffix keeps them distinct.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "serde", "source": "git+https://x", "kind": null },
            { "name": "serde", "source": "git+https://x", "kind": "dev" },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    let normal = rule.findings(&package, &[], DependencyKind::Normal);
    let dev = rule.findings(&package, &[], DependencyKind::Dev);
    assert_eq!(normal, vec!["serde".to_string()]);
    assert_eq!(dev, vec!["serde (dev)".to_string()]);
    assert_ne!(normal, dev, "same dep in two tables must not collide");
}

#[test]
fn workspace_member_names_are_the_no_deps_packages() {
    // With `--no-deps`, `packages` is exactly the workspace members.
    let metadata = serde_json::json!({
        "packages": [ { "name": "core" }, { "name": "adapters" } ]
    });
    assert_eq!(
        workspace_member_names(&metadata),
        vec!["adapters".to_string(), "core".to_string()],
    );
}

#[test]
fn workspace_rule_flags_only_unlisted_workspace_members() {
    // Deps: two workspace members (core, adapters), one external (serde), and one
    // path dependency that is NOT a workspace member (outside).
    let package = serde_json::json!({
        "dependencies": [
            { "name": "core", "source": null, "kind": null },
            { "name": "adapters", "source": null, "kind": null },
            {
                "name": "serde",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "outside", "source": null, "kind": null },
        ]
    });
    let workspace = vec!["core".to_string(), "adapters".to_string()];

    // Restrict to [core]: adapters is an unlisted workspace member → flagged;
    // serde (external) and outside (path, non-member) are ignored.
    let restrict = Rule::RestrictWorkspaceDependenciesTo {
        allowed: vec!["core".to_string()],
    };
    assert_eq!(
        restrict.findings(&package, &workspace, DependencyKind::Normal),
        vec!["adapters".to_string()],
    );

    // Empty allowlist forbids every workspace member, still ignoring external and
    // the non-member path dependency.
    let forbid_all = Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] };
    assert_eq!(
        forbid_all.findings(&package, &workspace, DependencyKind::Normal),
        vec!["adapters".to_string(), "core".to_string()],
    );
}

#[test]
fn workspace_rule_never_flags_a_crates_own_self_referential_dev_dependency() {
    // Round-11 finding: Cargo genuinely permits (and real projects use, e.g. a doctest/
    // dogfooding pattern) a crate declaring itself as a `[dev-dependencies]` path dependency
    // on itself (`main = { path = "." }`), and `cargo metadata --no-deps` emits this edge
    // verbatim (verified against real cargo). `workspace_member_names` trivially includes the
    // crate's own name, and `dependencies()` matches by bare package name with no
    // self-exclusion — so before this fix, a `forbid_all_workspace_dependencies` /
    // `restrict_workspace_dependencies_to` boundary declared on that crate flagged its own
    // legitimate self-dev-dependency as an "unlisted workspace dependency", even though a
    // self-dependency can never be an inter-crate layering violation (there is no OTHER crate
    // to leak across a boundary to).
    let package = serde_json::json!({
        "name": "main",
        "dependencies": [
            { "name": "main", "source": null, "kind": "dev" },
        ]
    });
    let workspace = vec!["main".to_string()];
    let forbid_all = Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] };
    assert_eq!(
        forbid_all.findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "a crate's own self-referential dev-dependency must never be flagged as an unlisted \
         workspace dependency"
    );
}

#[test]
fn no_dependency_rule_ever_flags_a_crates_own_self_referential_dependency() {
    // Round-12 finding: round 11 excluded a crate's own self-referential dependency (Cargo's
    // legal `[dev-dependencies] main = { path = "." }` doctest/dogfooding pattern) ONLY inside
    // Rule::RestrictWorkspaceDependenciesTo's own arm — leaving the IDENTICAL false positive live
    // in every sibling rule reading the same `dependencies()` / `dependencies_with_disallowed_source()`
    // observation (ForbidDependencyOn, RestrictDependenciesTo, RestrictDependencySourcesTo, and
    // the feature-granularity rules when the boundary happens to name the target's own crate).
    // A self-dependency is never a CROSS-crate concern any of these rules exist to govern, so the
    // exclusion is now at the shared observation source (`cargo_metadata.rs::is_self_dependency`),
    // closing every rule at once rather than one at a time.
    let package = serde_json::json!({
        "name": "main",
        "dependencies": [
            { "name": "main", "source": null, "kind": "dev", "features": ["x"] },
        ]
    });
    let workspace = vec!["main".to_string()];

    assert_eq!(
        Rule::ForbidDependencyOn {
            crates: vec!["main".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "ForbidDependencyOn must not flag the crate's own self-dependency"
    );
    assert_eq!(
        Rule::RestrictDependenciesTo {
            allowed: vec!["serde".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictDependenciesTo must not flag the crate's own self-dependency"
    );
    assert_eq!(
        Rule::RestrictDependencySourcesTo {
            allowed: vec![SourceKind::Registry]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictDependencySourcesTo must not flag the crate's own self-dependency's Path source"
    );
    assert_eq!(
        Rule::RestrictFeaturesOf {
            crate_: "main".to_string(),
            allowed: vec![]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "RestrictFeaturesOf must not observe the crate's own self-dependency's declared features"
    );
    assert_eq!(
        Rule::ForbidFeaturesOf {
            crate_: "main".to_string(),
            forbidden: vec!["x".to_string()]
        }
        .findings(&package, &workspace, DependencyKind::Dev),
        Vec::<String>::new(),
        "ForbidFeaturesOf must not observe the crate's own self-dependency's declared features"
    );
}

#[test]
fn coverage_counts_a_module_only_covered_crate_as_covered() {
    let members = vec!["app".to_string(), "core".to_string(), "memory".to_string()];
    let constitution = Constitution::new("c")
        .boundary(
            CrateBoundary::crate_("core")
                .forbid_all_workspace_dependencies()
                .because("core is independent"),
        )
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::kernel")
                .must_not_import("crate::projection")
                .because("layering"),
        );
    let coverage = coverage_from(members, &constitution);
    assert_eq!(coverage.total, 3);
    // `app` is covered by the module boundary, `core` by the crate boundary;
    // only `memory` has no boundary at all.
    assert_eq!(coverage.uncovered, vec!["memory".to_string()]);
}

fn mixed_constitution() -> Constitution {
    Constitution::new("my-project")
        .boundary(
            CrateBoundary::crate_("my-core")
                .deny_external_dependencies()
                .allow_external(["serde"])
                .because("my-core must stay dependency-light"),
        )
        .boundary(
            CrateBoundary::crate_("my-core")
                .forbid_dependency_on(["my-adapters"])
                .because("the core must not depend on adapters"),
        )
        .boundary(
            ModuleBoundary::in_crate("my-app")
                .module("crate::domain")
                .must_not_import("crate::http")
                .warn()
                .because("the domain must not import the HTTP layer"),
        )
}

#[test]
fn constitution_text_projects_every_boundary_with_its_parameters() {
    let text = constitution_text(&mixed_constitution());
    assert!(
        text.contains("Constitution: my-project  (3 boundaries)"),
        "{text}"
    );
    assert!(text.contains("crate my-core"), "{text}");
    assert!(
        text.contains("deny external dependencies (allow: serde)"),
        "{text}"
    );
    assert!(text.contains("forbid dependency on: my-adapters"), "{text}");
    assert!(text.contains("module crate::domain in my-app"), "{text}");
    assert!(text.contains("must not import crate::http"), "{text}");
    // Severity and reason both surface.
    assert!(
        text.contains("[warn]") && text.contains("[enforce]"),
        "{text}"
    );
    assert!(
        text.contains("the domain must not import the HTTP layer"),
        "{text}"
    );
}

#[test]
fn constitution_json_projects_boundaries_with_kinds_and_parameters() {
    let json = constitution_json(&mixed_constitution());
    let doc: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");
    assert_eq!(doc["constitution"], "my-project");
    let boundaries = doc["boundaries"].as_array().expect("array");
    assert_eq!(boundaries.len(), 3);

    // Crate boundary with an allowlist.
    assert_eq!(boundaries[0]["kind"], "crate");
    assert_eq!(boundaries[0]["target"], "my-core");
    assert_eq!(boundaries[0]["rule"], "deny external dependencies");
    assert_eq!(boundaries[0]["severity"], "enforce");
    assert_eq!(boundaries[0]["allowed"][0], "serde");

    // Forbid-dependency-on carries its crate list.
    assert_eq!(boundaries[1]["rule"], "forbid dependency on");
    assert_eq!(boundaries[1]["crates"][0], "my-adapters");

    // Module boundary: target is the module path (report convention), plus crate
    // and forbidden import.
    assert_eq!(boundaries[2]["kind"], "module");
    assert_eq!(boundaries[2]["target"], "crate::domain");
    assert_eq!(boundaries[2]["crate"], "my-app");
    assert_eq!(boundaries[2]["forbidden"], "crate::http");
    assert_eq!(boundaries[2]["severity"], "warn");
}

#[test]
fn an_empty_constitution_projects_cleanly() {
    let constitution = Constitution::new("fresh");
    let text = constitution_text(&constitution);
    assert!(
        text.contains("Constitution: fresh  (0 boundaries)"),
        "{text}"
    );
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"].as_array().unwrap().len(), 0);
}

#[test]
fn restrict_to_projects_its_allowlist() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .restrict_dependencies_to(["serde", "types"])
                .because("a may depend on only serde and types"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .restrict_dependencies_to::<[&str; 0], &str>([])
                .because("b must depend on nothing"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict dependencies to: serde, types"),
        "{text}"
    );
    assert!(text.contains("restrict dependencies to nothing"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict dependencies to");
    // A distinct key (`only`, not deny-external's `allowed`) for the closed set.
    assert_eq!(doc["boundaries"][0]["only"][0], "serde");
    assert!(doc["boundaries"][0]["allowed"].is_null());
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(doc["boundaries"][1]["only"].as_array().unwrap().len(), 0);
}

// A synthesized `cargo metadata --no-deps` package mirroring the source-kind probe:
// a registry dep, a path dep, a plain git dep, an optional git dep, a renamed git dep
// (real name `serde`, local alias `mydep`), a `{ git, version }` dep, an inherited
// workspace git dep (cargo flattens the source into the member as `git+…`), and a git
// dev-dependency. Every `source` string is exactly what cargo emits (verified).
fn source_package() -> Value {
    serde_json::json!({
        "dependencies": [
            {
                "name": "crates_io",
                "source": "registry+https://github.com/rust-lang/crates.io-index",
                "kind": null
            },
            { "name": "localdep", "source": null, "kind": null },
            { "name": "gitdep", "source": "git+https://example.invalid/a.git", "kind": null },
            {
                "name": "optgit",
                "source": "git+https://example.invalid/b.git",
                "kind": null,
                "optional": true
            },
            {
                "name": "serde",
                "rename": "mydep",
                "source": "git+https://example.invalid/c.git",
                "kind": null
            },
            { "name": "gitver", "source": "git+https://example.invalid/d.git", "kind": null },
            {
                "name": "inherited",
                "source": "git+https://example.invalid/e.git",
                "kind": null
            },
            { "name": "devgit", "source": "git+https://example.invalid/f.git", "kind": "dev" },
        ]
    })
}

#[test]
fn source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist() {
    let package = source_package();
    // Permit [Registry, Path]: every git-sourced normal dep is flagged — the plain
    // git dep, the OPTIONAL git dep (declared regardless of feature state), the
    // `{ git, version }` dep (a stated hygiene bound — it would publish, yet its
    // declared source is git), and the INHERITED workspace git dep (cargo flattens
    // the git source into the member). A RENAMED git dep is reported by its REAL
    // package name `serde`, not its alias `mydep`. The registry and path deps pass;
    // the git DEV-dep is not in the Normal-scoped surface.
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry, SourceKind::Path],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec![
            "gitdep".to_string(),
            "gitver".to_string(),
            "inherited".to_string(),
            "optgit".to_string(),
            "serde".to_string(),
        ],
        "every declared git source is flagged (optional/version/inherited included), \
             by real package name, while registry+path pass and the dev git dep is unscoped",
    );
}

#[test]
fn source_rule_registry_only_flags_a_path_dependency() {
    // Permit only [Registry]: the path dep is now flagged too (alongside every git
    // dep), documenting that Path is a governed source, not a silent exemption.
    let package = source_package();
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    let findings = rule.findings(&package, &[], DependencyKind::Normal);
    assert!(findings.contains(&"localdep".to_string()), "{findings:?}");
    assert!(!findings.contains(&"crates_io".to_string()), "{findings:?}");
}

#[test]
fn source_rule_is_clean_when_every_governed_source_is_allowed() {
    // A package whose only normal deps are registry + path, under [Registry, Path].
    let package = serde_json::json!({
        "dependencies": [
            { "name": "crates_io", "source": "registry+https://x", "kind": null },
            { "name": "localdep", "source": null, "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry, SourceKind::Path],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "all-registry-or-path is clean under a [Registry, Path] allowlist",
    );
}

#[test]
fn source_rule_does_not_observe_a_patch_redirect_declared_as_registry() {
    // The declared-vs-resolved bound: a registry dep that `[patch]` would redirect to
    // git still declares `source = registry+…` in `--no-deps` metadata, so it
    // classifies Registry and does NOT violate a [Registry] allowlist. Observing the
    // resolved git source is cargo-deny's `[sources]` lane, not a Tianheng capability.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "patched", "source": "registry+https://x", "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "the declared layer does not observe [patch]; correct — [patch] never blocks publish",
    );
}

#[test]
fn source_rule_scopes_to_the_dependency_kind() {
    // Only the git dev-dep exists; a Normal-scoped boundary does not observe it, a
    // Dev-scoped one does.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "devgit", "source": "git+https://x", "kind": "dev" },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo {
        allowed: vec![SourceKind::Registry],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "a dev git dep is outside a Normal-scoped surface",
    );
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Dev),
        vec!["devgit (dev)".to_string()],
        "a Dev-scoped boundary governs the dev table",
    );
}

#[test]
fn source_rule_empty_allowlist_forbids_every_dependency_by_source() {
    let package = serde_json::json!({
        "dependencies": [
            { "name": "crates_io", "source": "registry+https://x", "kind": null },
            { "name": "localdep", "source": null, "kind": null },
            { "name": "gitdep", "source": "git+https://x", "kind": null },
        ]
    });
    let rule = Rule::RestrictDependencySourcesTo { allowed: vec![] };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec![
            "crates_io".to_string(),
            "gitdep".to_string(),
            "localdep".to_string(),
        ],
        "an empty source allowlist forbids every dependency regardless of source",
    );
}

#[test]
fn source_boundary_absent_target_is_a_constitution_error() {
    // Parity with the other crate rules: a boundary on a crate not in the workspace is
    // a constitution error (→ exit 2), never a silent pass.
    let metadata = serde_json::json!({ "packages": [{ "name": "present" }] });
    let boundary = CrateBoundary::crate_("absent")
        .restrict_dependency_sources_to([SourceKind::Registry])
        .because("absent must publish to crates.io");
    let mut violations = Vec::new();
    let result = check_crate_boundary(&metadata, &[], &boundary, &mut violations);
    assert!(
        result.is_err(),
        "an absent target crate must be a constitution error, not exit 0/1",
    );
}

#[test]
fn source_boundary_carries_its_severity_and_gates_against_the_baseline() {
    // A source violation folds into the shared report identity (target, rule,
    // finding) and honors severity + baseline exactly as the sibling rules do.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "infra",
            "dependencies": [
                { "name": "gitdep", "source": "git+https://x", "kind": null },
            ],
        }]
    });
    // Warn severity: the violation is recorded but must not fail the reaction.
    let warn = CrateBoundary::crate_("infra")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .warn()
        .because("infra should publish; a git source is advisory here");
    let mut violations = Vec::new();
    check_crate_boundary(&metadata, &[], &warn, &mut violations).unwrap();
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].severity, Severity::Warn);
    assert_eq!(violations[0].rule, "restrict dependency sources to");
    assert_eq!(violations[0].finding, "gitdep");
    let id = violations[0].id();
    assert_eq!(id.target, "infra");
    assert_eq!(
        id.rule_key().expect("production rule identity").rule_type(),
        "tianheng.rule/guibiao/restrict-dependency-sources-to"
    );
    let fact = id
        .finding_key()
        .expect("a production fact has semantic identity");
    assert_eq!(fact.fact_type(), "tianheng.fact/guibiao/dependency-source");
    assert_eq!(fact.shape(), "declared-source");
    assert_eq!(
        fact.fields().collect::<Vec<_>>(),
        vec![("kind", "normal"), ("package", "gitdep"), ("source", "git")]
    );
    assert!(
        violations[0].file.is_none(),
        "a source violation is a manifest relation, not a source line",
    );

    // Enforce + baseline parity: the same violation, once baselined, does not fail.
    let enforce = CrateBoundary::crate_("infra")
        .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
        .because("infra must publish to crates.io, so no git source");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &enforce, &mut v).unwrap();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&report);
    apply_baseline(&mut report, &baseline);
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        0,
        "a fully baselined source violation does not fail the reaction",
    );
}

#[test]
fn source_boundary_projects_its_allowed_sources() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("infra")
                .restrict_dependency_sources_to([SourceKind::Registry, SourceKind::Path])
                .because("infra must publish to crates.io, so its manifest declares no git"),
        )
        .boundary(
            CrateBoundary::crate_("locked")
                .restrict_dependency_sources_to([])
                .because("locked must declare no dependencies at all"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict dependency sources to: registry, path"),
        "{text}"
    );
    assert!(
        text.contains("forbid all dependencies (by source)"),
        "{text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "restrict dependency sources to"
    );
    assert_eq!(doc["boundaries"][0]["allowed_sources"][0], "registry");
    assert_eq!(doc["boundaries"][0]["allowed_sources"][1], "path");
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(
        doc["boundaries"][1]["allowed_sources"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

// ---- Feature-granularity crate rules (declared-not-resolved) --------------------------

// A `cargo metadata --no-deps` package declaring the dependency `C` under a variety of
// edges. Field names are exactly what cargo emits on a dependency edge: `features` (the
// authored list) and `uses_default_features` (the default toggle; absent ⇒ default-on).
fn feature_package() -> Value {
    serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "source": "registry+https://x",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": true
            },
        ]
    })
}

#[test]
fn declared_feature_is_observed() {
    // WHEN the target declares C = { features = ["extra"] } → the declared set contains `extra`.
    let package = feature_package();
    assert!(
        declared_features(&package, "C", DependencyKind::Normal).contains(&"extra".to_string()),
        "an authored feature is observed",
    );
}

#[test]
fn default_features_are_the_default_pseudo_feature() {
    // WHEN C is declared without `default-features = false` (uses_default_features true, or
    // the field absent) → the declared set contains the `default` pseudo-feature.
    let with_flag = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": true },
        ]
    });
    let absent_flag = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null },
        ]
    });
    assert!(
        declared_features(&with_flag, "C", DependencyKind::Normal).contains(&"default".to_string()),
        "uses_default_features=true ⇒ default is requested",
    );
    assert!(
        declared_features(&absent_flag, "C", DependencyKind::Normal)
            .contains(&"default".to_string()),
        "an absent uses_default_features field ⇒ default-on ⇒ default is requested",
    );
}

#[test]
fn disabling_default_features_drops_the_default_pseudo_feature() {
    // WHEN C = { default-features = false, features = ["extra"] } → declared set is
    // { extra }, without `default`.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "C", DependencyKind::Normal),
        vec!["extra".to_string()],
        "default-features = false drops the default pseudo-feature",
    );
}

#[test]
fn transitive_enables_are_not_chased() {
    // A MEANINGFUL test of the rejected "expand through C's `[features]` graph" layer: the
    // metadata carries C's expansion evidence (a `resolve.nodes` entry for C where `full`
    // has resolved into `unstable`), yet the target declares only `full`. A `forbid C/unstable`
    // rule must stay clean and a `forbid C/full` rule must fire — proving the rule reads the
    // target's AUTHORED request, never C's expanded/resolved graph. If `findings` were changed
    // to fold in the transitive `unstable` (via C's graph or the resolve node), the first
    // assertion below would flip to a `C/unstable` finding and fail.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "target",
            "dependencies": [
                {
                    "name": "C",
                    "source": "registry+https://x",
                    "kind": null,
                    "features": ["full"],
                    "uses_default_features": false
                },
            ],
        }],
        // The unified resolved graph: C's `full` has pulled in `unstable`. This is exactly the
        // contamination the declared-not-resolved model rejects; it must not reach the finding.
        "resolve": {
            "nodes": [
                { "id": "C 1.0.0", "features": ["full", "unstable"] },
            ]
        }
    });

    let forbid_unstable = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("C's unstable face is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &forbid_unstable, &mut v).unwrap();
    assert!(
        v.is_empty(),
        "the transitively-enabled `unstable` (present in resolve.nodes) is not chased: {v:?}",
    );

    // The authored `full` IS observed and flagged, so the metadata is genuinely exercised
    // (the clean result above is not merely an unread fixture).
    let forbid_full = CrateBoundary::crate_("target")
        .forbid_feature("C", "full")
        .because("even the authored `full` is governed");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &forbid_full, &mut v).unwrap();
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].finding, "C/full");
}

#[test]
fn a_sibling_crates_enable_is_not_attributed_to_the_target() {
    // A MEANINGFUL test of the rejected `resolve.nodes` unification layer: the metadata has a
    // SIBLING workspace member enabling C's `unstable` AND a `resolve.nodes` entry for C whose
    // unified set contains `unstable`. The TARGET declares C with default-features = false and
    // no features. A `forbid C/unstable` rule on the target must stay clean — the declared set
    // is computed from the target's own edge, not the sibling's enable nor the unified node. If
    // `findings` read `resolve.nodes[C].features` (or scanned other packages' edges), this would
    // report `C/unstable` and fail.
    let metadata = serde_json::json!({
        "packages": [
            {
                "name": "target",
                "dependencies": [
                    {
                        "name": "C",
                        "source": "registry+https://x",
                        "kind": null,
                        "uses_default_features": false
                    },
                ],
            },
            {
                "name": "sibling",
                "dependencies": [
                    {
                        "name": "C",
                        "source": "registry+https://x",
                        "kind": null,
                        "features": ["unstable"],
                        "uses_default_features": false
                    },
                ],
            },
        ],
        // Feature unification folds the sibling's `unstable` into the one shared C node.
        "resolve": {
            "nodes": [
                { "id": "C 1.0.0", "features": ["unstable"] },
            ]
        }
    });

    let boundary = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("target must not author C's unstable");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert!(
        v.is_empty(),
        "a sibling's enable (and the unified resolve node) is not attributed to the target: {v:?}",
    );

    // Sanity that the harness IS reactive: the SIBLING, which authored `unstable`, is flagged
    // by the same rule — so the target's clean result reflects its own edge, not a dead fixture.
    let sibling_boundary = CrateBoundary::crate_("sibling")
        .forbid_feature("C", "unstable")
        .because("the sibling did author it");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &sibling_boundary, &mut v).unwrap();
    assert_eq!(v.len(), 1, "the sibling authored C/unstable: {v:?}");
    assert_eq!(v[0].finding, "C/unstable");
}

#[test]
fn declared_set_unions_across_multiple_edges() {
    // WHEN C is declared in [dependencies] with features=["a"] (defaults on) AND under
    // [target.'cfg(windows)'.dependencies] with default-features=false, features=["unstable"]
    // — both Normal-kind edges — the declared set is the union { a, default, unstable }.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["a"],
                "uses_default_features": true
            },
            {
                "name": "C",
                "kind": null,
                "target": "cfg(windows)",
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "C", DependencyKind::Normal),
        vec![
            "a".to_string(),
            "default".to_string(),
            "unstable".to_string()
        ],
        "the set unions across every matching edge; default from the plain edge",
    );
    // A rule forbidding `unstable` therefore emits a violation.
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
fn a_dependency_is_matched_by_package_name_not_local_alias() {
    // WHEN the target declares myc = { package = "real-c", features = ["unstable"] } — cargo
    // reports name="real-c", rename="myc" — a rule against the package name `real-c` sees
    // `unstable`, while one against the local alias `myc` matches nothing.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "real-c",
                "rename": "myc",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "real-c", DependencyKind::Normal),
        vec!["unstable".to_string()],
        "matched by resolved package name",
    );
    assert!(
        declared_features(&package, "myc", DependencyKind::Normal).is_empty(),
        "the local alias matches nothing",
    );
}

#[test]
fn restrict_features_flags_a_feature_outside_the_allowlist() {
    // WHEN the target declares C's `unstable` and the boundary restricts C to ["stable"]
    // → a violation for C/unstable.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec!["stable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
fn restrict_features_is_clean_when_every_declared_feature_is_allowed() {
    // WHEN the only declared feature is `stable`, with default-features = false, under a
    // restrict-to ["stable"] → clean.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["stable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec!["stable".to_string()],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an in-allowlist feature (with defaults off) is clean",
    );
}

#[test]
fn empty_allowlist_forbids_declaring_any_feature_including_default() {
    // WHEN the target leaves defaults on (or declares any feature) and the boundary restricts
    // C's features to [] → every declared feature, `default` included, is a violation.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": true
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec![],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/default".to_string(), "C/extra".to_string()],
        "an empty allowlist forbids default and every explicit feature",
    );
}

#[test]
fn restrict_features_on_a_crate_not_depended_on_is_clean() {
    // WHEN the boundary restricts the features of a crate the target does not depend on in
    // the selected table → clean (there is no declared set to constrain; never exit 2 here).
    let package = serde_json::json!({
        "dependencies": [
            { "name": "other", "kind": null, "uses_default_features": true },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec![],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "a feature rule on an undepended crate is clean",
    );
}

#[test]
fn forbid_feature_flags_a_declared_forbidden_feature() {
    // WHEN the target declares C's `unstable` and the boundary forbids C's `unstable`
    // → a violation for C/unstable.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
fn forbidding_default_requires_default_features_off() {
    // WHEN the boundary forbids C's `default` and the target declares C WITHOUT
    // default-features = false → a violation for C/default.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": true },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["default".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/default".to_string()],
        "forbidding `default` ≡ requiring default-features = false",
    );
    // With default-features = false, the same rule is clean.
    let off = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": false },
        ]
    });
    assert!(
        rule.findings(&off, &[], DependencyKind::Normal).is_empty(),
        "with defaults off, forbidding `default` is satisfied",
    );
}

#[test]
fn a_forbidden_feature_the_target_does_not_declare_is_clean() {
    // WHEN the boundary forbids C's `unstable`, and the target declares only C's `full`
    // (which would transitively enable `unstable`, not chased) → clean.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["full"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an undeclared forbidden feature is clean; transitive enables are not chased",
    );
}

#[test]
fn empty_forbidden_set_is_a_no_op() {
    // WHEN a boundary forbids an empty set of features of C → clean regardless of what the
    // target declares.
    let package = feature_package();
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec![],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an empty forbidden set is a vacuous no-op",
    );
}

#[test]
fn feature_finding_carries_the_dependency_kind_suffix() {
    // A dev-table feature request carries the ` (dev)` suffix (the shared kind-qualify logic),
    // so C/unstable in [dependencies] and in [dev-dependencies] stay distinct findings.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": "dev",
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Dev),
        vec!["C/unstable (dev)".to_string()],
        "a dev-kind feature finding is `C/f (dev)`",
    );
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "the Normal surface does not observe a dev-table edge",
    );
}

#[test]
fn two_forbidden_features_of_the_same_crate_stay_distinct() {
    // The one forbidden bug for the feature family: C/unstable and C/nightly, both forbidden,
    // must not collapse — baselining C/unstable must not mask C/nightly.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable", "nightly"],
                "uses_default_features": false
            },
        ]
    });
    let metadata = serde_json::json!({ "packages": [{
        "name": "target",
        "dependencies": package["dependencies"].clone(),
    }] });
    let boundary = CrateBoundary::crate_("target")
        .forbid_features_of("C", ["unstable", "nightly"])
        .because("no unstable/nightly features of C");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert_eq!(
        v.iter().map(|x| x.finding.as_str()).collect::<Vec<_>>(),
        vec!["C/nightly", "C/unstable"],
        "both features are distinct findings under the one rule",
    );

    // Baseline records ONLY C/unstable (a prior accepted state). Re-applying it must mark
    // C/unstable baselined while leaving C/nightly live — the finding, not the (target, rule)
    // pair, is the baseline key, so one feature's acceptance never masks another's.
    let accepted = v
        .iter()
        .find(|violation| violation.finding == "C/unstable")
        .expect("C/unstable present")
        .clone();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&Report::new(vec![accepted]));
    apply_baseline(&mut report, &baseline);

    let unstable = report
        .violations
        .iter()
        .find(|x| x.finding == "C/unstable")
        .expect("C/unstable present");
    let nightly = report
        .violations
        .iter()
        .find(|x| x.finding == "C/nightly")
        .expect("C/nightly present");
    assert!(unstable.baselined, "the baselined C/unstable is marked");
    assert!(
        !nightly.baselined,
        "C/nightly is NOT masked by the C/unstable baseline entry",
    );
    // The baseline entry for C/unstable is matched (not stale); C/nightly is not in it.
    assert!(
        baseline.stale(&report).is_empty(),
        "the C/unstable baseline entry matches a current violation, so it is not stale",
    );
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "the still-live C/nightly enforce violation fails the reaction",
    );
}

#[test]
fn the_two_feature_rule_families_keep_identity_injective() {
    // A restrict and a forbid rule that both flag C/unstable on the same target stay distinct
    // triples, because their `rule` labels differ.
    assert_ne!(
        Rule::RestrictFeaturesOf {
            crate_: "C".to_string(),
            allowed: vec![],
        }
        .label(),
        Rule::ForbidFeaturesOf {
            crate_: "C".to_string(),
            forbidden: vec![],
        }
        .label(),
        "the two feature-rule labels must differ",
    );
    // And distinct from every crate rule label.
    let feature_labels = ["restrict features of", "forbid features of"];
    for other in [
        Rule::DenyExternalDependencies { allowed: vec![] }.label(),
        Rule::ForbidDependencyOn { crates: vec![] }.label(),
        Rule::RestrictDependenciesTo { allowed: vec![] }.label(),
        Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] }.label(),
        Rule::RestrictDependencySourcesTo { allowed: vec![] }.label(),
    ] {
        assert!(
            !feature_labels.contains(&other),
            "feature labels must be distinct from crate-rule label {other}",
        );
    }

    // Concrete non-masking: a restrict rule (empty allowlist) and a forbid rule both flag the
    // SAME `C/unstable` on the SAME target. Because their `rule` labels differ, they are two
    // distinct `(target, rule, finding)` triples: baselining the restrict violation must NOT
    // mask the forbid violation.
    let metadata = serde_json::json!({ "packages": [{
        "name": "target",
        "dependencies": [
            {
                "name": "C",
                "source": "registry+https://x",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ],
    }] });
    let restrict = CrateBoundary::crate_("target")
        .restrict_features_of("C", Vec::<String>::new())
        .because("C's feature surface is closed");
    let forbid = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("C's unstable is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &restrict, &mut v).unwrap();
    check_crate_boundary(&metadata, &[], &forbid, &mut v).unwrap();
    // Same target + same finding, two different rule labels ⇒ two distinct violations.
    assert_eq!(v.len(), 2, "{v:?}");
    assert!(v.iter().all(|x| x.finding == "C/unstable"));
    let mut rules: Vec<&str> = v.iter().map(|x| x.rule.as_str()).collect();
    rules.sort_unstable();
    assert_eq!(rules, vec!["forbid features of", "restrict features of"]);

    // Baseline ONLY the restrict-rule C/unstable; the forbid-rule C/unstable must stay live.
    let accepted = v
        .iter()
        .find(|violation| violation.rule == "restrict features of")
        .expect("restrict violation present")
        .clone();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&Report::new(vec![accepted]));
    apply_baseline(&mut report, &baseline);
    let restrict_v = report
        .violations
        .iter()
        .find(|x| x.rule == "restrict features of")
        .unwrap();
    let forbid_v = report
        .violations
        .iter()
        .find(|x| x.rule == "forbid features of")
        .unwrap();
    assert!(restrict_v.baselined, "the restrict C/unstable is baselined");
    assert!(
        !forbid_v.baselined,
        "the forbid C/unstable is a distinct triple, not masked by the restrict baseline",
    );
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "the still-live forbid C/unstable fails the reaction",
    );
}

#[test]
fn feature_report_names_target_rule_finding_and_reason() {
    // WHEN a feature rule on C is violated by the declared `unstable` → the report names the
    // target, the feature rule, the finding C/unstable, and the reason; the reaction fails.
    let metadata = serde_json::json!({ "packages": [{
        "name": "app",
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ],
    }] });
    let boundary = CrateBoundary::crate_("app")
        .forbid_feature("C", "unstable")
        .because("C's unstable API face is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].target, "app");
    assert_eq!(v[0].rule, "forbid features of");
    assert_eq!(v[0].finding, "C/unstable");
    assert_eq!(v[0].reason, "C's unstable API face is off-limits");
    assert!(
        v[0].file.is_none(),
        "a feature violation is a manifest relation, not a source line",
    );
    assert_eq!(
        Outcome::Violations(Report::new(v)).exit_code(),
        1,
        "an enforce feature violation fails the reaction",
    );
}

#[test]
fn a_feature_boundary_on_an_absent_target_is_a_constitution_error() {
    // Parity with the other crate rules: a feature boundary on a crate not in the workspace
    // is a constitution error (→ exit 2), never a silent pass. (`C` is never resolved, but the
    // TARGET must be a workspace member.)
    let metadata = serde_json::json!({ "packages": [{ "name": "present" }] });
    let boundary = CrateBoundary::crate_("absent")
        .restrict_features_of("C", ["stable"])
        .because("absent may use only C's stable face");
    let mut v = Vec::new();
    assert!(
        check_crate_boundary(&metadata, &[], &boundary, &mut v).is_err(),
        "an absent target crate must be a constitution error",
    );
}

#[test]
fn feature_boundary_defaults_to_enforce_normal_kind() {
    // The builders finish through Enforce severity / Normal kind, like restrict_dependencies_to.
    let restrict = CrateBoundary::crate_("app")
        .restrict_features_of("C", ["stable"])
        .because("r");
    assert_eq!(restrict.severity(), Severity::Enforce);
    assert_eq!(restrict.dependency_kind(), DependencyKind::Normal);
    let forbid = CrateBoundary::crate_("app")
        .forbid_feature("C", "default")
        .because("f");
    assert_eq!(forbid.severity(), Severity::Enforce);
    assert_eq!(forbid.dependency_kind(), DependencyKind::Normal);
    // forbid_feature is the singular convenience over forbid_features_of.
    assert_eq!(
        forbid.rule(),
        &Rule::ForbidFeaturesOf {
            crate_: "C".to_string(),
            forbidden: vec!["default".to_string()],
        },
    );
}

#[test]
fn feature_rules_project_to_text_and_json() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("app")
                .restrict_features_of("tokio", ["rt", "macros"])
                .because("app may use only tokio's rt and macros"),
        )
        .boundary(
            CrateBoundary::crate_("lib")
                .restrict_features_of::<&str, [&str; 0], &str>("tokio", [])
                .because("lib must declare no tokio feature at all"),
        )
        .boundary(
            CrateBoundary::crate_("worker")
                .forbid_features_of("tokio", ["unstable"])
                .dependency_kind(DependencyKind::Build)
                .because("no unstable tokio in the build script"),
        )
        .boundary(
            CrateBoundary::crate_("edge")
                .forbid_features_of::<&str, [&str; 0], &str>("serde", [])
                .because("vacuous"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict features of tokio to: rt, macros"),
        "{text}"
    );
    assert!(
        text.contains("restrict features of tokio to nothing"),
        "{text}"
    );
    assert!(
        text.contains("forbid features of tokio: unstable (build dependencies)"),
        "{text}"
    );
    assert!(text.contains("forbid no features of serde"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict features of");
    assert_eq!(doc["boundaries"][0]["crate"], "tokio");
    assert_eq!(doc["boundaries"][0]["only_features"][0], "rt");
    assert_eq!(doc["boundaries"][0]["only_features"][1], "macros");
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(
        doc["boundaries"][1]["only_features"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(doc["boundaries"][2]["rule"], "forbid features of");
    assert_eq!(doc["boundaries"][2]["crate"], "tokio");
    assert_eq!(doc["boundaries"][2]["forbidden_features"][0], "unstable");
    // Non-Normal kind is disclosed in the projection.
    assert_eq!(doc["boundaries"][2]["dependency_kind"], "build");
    // Distinct keys per polarity: restrict uses only_features, forbid uses forbidden_features.
    assert!(doc["boundaries"][0]["forbidden_features"].is_null());
    assert!(doc["boundaries"][2]["only_features"].is_null());
}

#[test]
fn module_restrict_imports_to_projects_its_allowlist() {
    let constitution = Constitution::new("p")
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::kernel")
                .restrict_imports_to(["crate::types"])
                .because("the kernel may import only types"),
        )
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::leaf")
                .restrict_imports_to::<[&str; 0], &str>([])
                .because("the leaf may import only its own subtree"),
        );

    let text = constitution_text(&constitution);
    assert!(text.contains("restrict imports to: crate::types"), "{text}");
    assert!(text.contains("restrict imports to nothing"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict imports to");
    assert_eq!(doc["boundaries"][0]["kind"], "module");
    assert_eq!(doc["boundaries"][0]["target"], "crate::kernel");
    // The closed set uses `only` (the crate-level vocabulary), never `forbidden`.
    assert_eq!(doc["boundaries"][0]["only"][0], "crate::types");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(doc["boundaries"][1]["only"].as_array().unwrap().len(), 0);
}

#[test]
fn an_unreadable_utf8_governed_source_file_is_a_scan_error() {
    // Deterministic on every platform and euid (unlike the permission-based tests,
    // which skip under a privileged user): invalid UTF-8 makes `read_to_string` fail,
    // so a reachable governed-crate source file that cannot be read is a scan error
    // (exit 2), never a silent skip — the core-contract "cannot judge" rule.
    let ws = TempWorkspace::new("utf8");
    ws.write("lib.rs", "pub mod kernel;\n");
    // 0xFF / 0xFE are not valid UTF-8; read_to_string returns Err on all platforms.
    std::fs::write(ws.src().join("kernel.rs"), [0xFF, 0xFE, 0x00, 0x80]).expect("write kernel.rs");

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_import("crate::forbidden")
        .because("kernel must not import forbidden");
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);
    assert!(
        result.is_err(),
        "an unreadable (invalid-UTF-8) governed source file must be a scan error"
    );
}

#[test]
fn dependency_kind_appears_in_the_projection() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .deny_external_dependencies()
                .dependency_kind(DependencyKind::Dev)
                .because("a's dev dependencies stay light"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .deny_external_dependencies()
                .because("b stays light"),
        );
    let text = constitution_text(&constitution);
    assert!(text.contains("(dev dependencies)"), "{text}");
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["dependency_kind"], "dev");
    // A Normal-kind boundary omits the field entirely (the common projection).
    assert!(doc["boundaries"][1]["dependency_kind"].is_null(), "{doc}");
}

#[test]
fn restrict_workspace_dependencies_to_projects_only_workspace() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .restrict_workspace_dependencies_to(["b"])
                .because("a may depend on only workspace member b"),
        )
        .boundary(
            CrateBoundary::crate_("c")
                .forbid_all_workspace_dependencies()
                .because("c must not depend on any workspace member"),
        );
    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict workspace dependencies to: b"),
        "{text}"
    );
    assert!(text.contains("forbid all workspace dependencies"), "{text}");
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "restrict workspace dependencies to"
    );
    // The distinct key (`only_workspace`, not `only`) says which dependency surface
    // is governed — the self-describing distinction with no coverage before now.
    assert_eq!(doc["boundaries"][0]["only_workspace"][0], "b");
    assert!(doc["boundaries"][0]["only"].is_null());
    // The empty allowlist (forbid-all) still emits `only_workspace: []`.
    assert_eq!(
        doc["boundaries"][1]["only_workspace"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

/// A boundary's `.with_anchor(...)` rides through the reaction onto the produced violation and
/// into its JSON — the durable governance pointer reaches the CI-consumable surface.
#[test]
fn an_anchored_boundary_stamps_its_violations_with_the_anchor() {
    let (result, violations) = run_module_check(
        "anchored",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("the kernel must not import a secret")
            .with_anchor("ADR-014"),
    );
    result.expect("a valid module boundary");
    assert!(!violations.is_empty(), "the forbidden import must react");
    assert_eq!(violations[0].anchor.as_deref(), Some("ADR-014"));
    assert_eq!(violations[0].to_json()["anchor"], "ADR-014");
}

/// The mirror: a boundary that declares no anchor produces `None` on its violations — the anchor
/// is opt-in and never fabricated.
#[test]
fn an_anchorless_boundary_leaves_its_violations_unanchored() {
    let (result, violations) = run_module_check(
        "unanchored",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("the kernel must not import a secret"),
    );
    result.expect("a valid module boundary");
    assert!(!violations.is_empty());
    assert_eq!(violations[0].anchor, None);
}

/// The reaction stamps a module violation with the rule's repair-direction polarity: a deny rule
/// (`must_not_import`) → `DenyBreach`, an allowlist rule (`restrict_imports_to`) → `AllowlistGap`.
#[test]
fn a_module_violation_carries_its_rule_repair_polarity() {
    let (_r, deny) = run_module_check(
        "polarity-deny",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("deny"),
    );
    assert_eq!(deny[0].polarity, Some(Polarity::DenyBreach));

    let (_r, allow) = run_module_check(
        "polarity-allow",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::infra::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .restrict_imports_to(["crate::types"])
            .because("allow"),
    );
    assert_eq!(allow[0].polarity, Some(Polarity::AllowlistGap));
}

/// The rule → polarity classification, incl. the deliberate call that `deny_external_dependencies`
/// is `AllowlistGap` (its `allow_external` is an in-boundary declaration path — by repair direction,
/// not rule name), while `forbid_dependency_on` names a specific forbidden crate → `DenyBreach`.
#[test]
fn crate_rule_polarity_classifies_by_repair_direction() {
    let deny_external = CrateBoundary::crate_("x")
        .deny_external_dependencies()
        .because("r");
    assert_eq!(deny_external.rule().polarity(), Polarity::AllowlistGap);

    let forbid = CrateBoundary::crate_("x")
        .forbid_dependency_on(["openssl"])
        .because("r");
    assert_eq!(forbid.rule().polarity(), Polarity::DenyBreach);

    let restrict = CrateBoundary::crate_("x")
        .restrict_dependencies_to(["serde"])
        .because("r");
    assert_eq!(restrict.rule().polarity(), Polarity::AllowlistGap);
}

/// The constitution projection surfaces a boundary's anchor **only when set**, so an anchor-less
/// boundary keeps byte-identical JSON (the Some-only discipline that protects the self-law
/// projection's staleness byte-check).
#[test]
fn constitution_json_emits_anchor_only_when_set() {
    let constitution = Constitution::new("anchors")
        .boundary(
            CrateBoundary::crate_("a")
                .forbid_all_workspace_dependencies()
                .because("anchored")
                .with_anchor("ADR-014"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .forbid_all_workspace_dependencies()
                .because("unanchored"),
        );
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["anchor"], "ADR-014");
    assert!(
        doc["boundaries"][1].get("anchor").is_none(),
        "an anchor-less boundary must emit no `anchor` key"
    );
}

// --- must_only_be_imported_by (the inbound closed allowlist) ----------------

fn only_importers(allowed: &[&str]) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::internal")
        .must_only_be_imported_by(allowed.iter().copied())
        .because("internal is imported only through its facade")
}

#[test]
fn must_only_be_imported_by_flags_an_importer_outside_the_allowlist() {
    let (result, violations) = run_module_check(
        "only-basic",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\npub mod consumer;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "use crate::internal::Secret;\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    // facade is allowlisted (clean); consumer is not (violates).
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "crate::internal");
    assert_eq!(violations[0].finding, "crate::consumer");
    assert_eq!(violations[0].rule, "module may only be imported by");
}

#[test]
fn must_only_be_imported_by_authorizes_an_allowed_inline_importer() {
    // `crate::facade` is an INLINE module and IS allow-listed. Its import is attributed
    // to `crate::facade` (not the file's `crate`), so it is correctly authorized. Testing the file
    // module `crate` against the allowlist would wrongly flag the allowed inline importer.
    let (result, violations) = run_module_check(
        "only-inline-allowed",
        &[
            (
                "lib.rs",
                "pub mod internal;\nmod facade { use crate::internal::Secret; }\n",
            ),
            ("internal.rs", "// protected\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an allow-listed inline importer must not be flagged: {violations:?}"
    );
}

#[test]
fn must_only_be_imported_by_flags_a_disallowed_inline_importer_by_its_true_identity() {
    // A disallowed INLINE importer is flagged with its true identity `crate::rogue` (not the file's
    // `crate`), so the structured fact — and thus `(target, rule key, fact)` identity — is
    // correct rather than shifted onto the file module.
    let (result, violations) = run_module_check(
        "only-inline-disallowed",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\nmod rogue { use crate::internal::Secret; }\n",
            ),
            ("internal.rs", "// protected\n"),
            (
                "facade.rs",
                "// the allow-listed importer declares no import here\n",
            ),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::rogue");
}

#[test]
fn must_only_be_imported_by_admits_the_allowlisted_importer_subtree() {
    let (result, violations) = run_module_check(
        "only-subtree",
        &[
            ("lib.rs", "pub mod internal;\npub mod facade;\n"),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "pub mod v1;\n"),
            ("facade/v1.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "crate::facade::v1 is beneath the allowlisted importer: {violations:?}"
    );
}

#[test]
fn must_only_be_imported_by_does_not_admit_a_prefix_colliding_sibling() {
    let (result, violations) = run_module_check(
        "only-prefix",
        &[
            ("lib.rs", "pub mod internal;\npub mod facadex;\n"),
            ("internal.rs", "// protected\n"),
            ("facadex.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding, "crate::facadex",
        "a sibling of the allowlisted importer is not admitted"
    );
}

#[test]
fn must_only_be_imported_by_never_flags_the_protected_subtree() {
    let (result, violations) = run_module_check(
        "only-own-subtree",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "pub mod deep;\n"),
            ("internal/deep.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a module within the protected subtree is not an inbound importer: {violations:?}"
    );
}

#[test]
fn must_only_be_imported_by_empty_allowlist_forbids_every_outside_importer() {
    let (result, violations) = run_module_check(
        "only-empty",
        &[
            ("lib.rs", "pub mod internal;\npub mod consumer;\n"),
            ("internal.rs", "// protected\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&[]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::consumer");
}

#[test]
fn must_only_be_imported_by_admits_multiple_allowlisted_importers() {
    let (result, violations) = run_module_check(
        "only-multiple",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod facade;\npub mod api;\npub mod consumer;\n",
            ),
            ("internal.rs", "// protected\n"),
            ("facade.rs", "use crate::internal::Secret;\n"),
            ("api.rs", "use crate::internal::Secret;\n"),
            ("consumer.rs", "use crate::internal::Secret;\n"),
        ],
        only_importers(&["crate::facade", "crate::api"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::consumer");
}

#[test]
fn must_only_be_imported_by_ignores_external_imports() {
    let (result, violations) = run_module_check(
        "only-external",
        &[
            ("lib.rs", "pub mod internal;\npub mod consumer;\n"),
            ("internal.rs", "// protected\n"),
            ("consumer.rs", "use serde::Deserialize;\n"),
        ],
        only_importers(&["crate::facade"]),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an external import is out of scope: {violations:?}"
    );
}

#[test]
fn must_only_be_imported_by_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "only-m-crate",
        &[("lib.rs", "pub mod http;\n"), ("http.rs", "// nothing\n")],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_only_be_imported_by(["crate::facade"])
            .because("the crate root cannot be protected this way"),
    );
    let err = result.expect_err("protecting `crate` must be a constitution error");
    assert_eq!(err, must_only_be_imported_by_on_crate_error("x"));
}

#[test]
fn must_only_be_imported_by_rule_text_and_json_params() {
    // Projection surface: distinct label/text and the surface-qualified `only_importers` key.
    let rule = ModuleRule::MustOnlyBeImportedBy {
        allowed: vec!["crate::facade".to_string()],
    };
    assert_eq!(rule.label(), "module may only be imported by");
    assert_eq!(rule.polarity(), Polarity::AllowlistGap);
    assert_eq!(rule.text(), "may only be imported by: crate::facade");
    assert_eq!(
        rule.json_params(),
        vec![("only_importers", serde_json::json!(["crate::facade"]))]
    );
    let empty = ModuleRule::MustOnlyBeImportedBy { allowed: vec![] };
    assert_eq!(empty.text(), "may only be imported by nothing");
}

// --- external-crate confinement (`confine_external_crate`) ------------------------------

fn confine(governed: &str, crate_name: &str) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module(governed)
        .confine_external_crate(crate_name)
        .because("the platform vocabulary stays behind the ffi module")
}

#[test]
fn confine_flags_an_external_import_outside_the_subtree() {
    // The confined crate is the target; the offending importer module is the finding.
    let (result, violations) = run_module_check(
        "confine-outside",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "libc");
    assert_eq!(violations[0].finding, "crate::service");
    let file = violations[0].file.as_deref().expect("carries its file");
    assert!(file.ends_with("service.rs"), "names the offender: {file}");
}

#[test]
fn confine_is_clean_within_the_subtree() {
    let (result, violations) = run_module_check(
        "confine-within",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "within the subtree is clean: {violations:?}"
    );
}

#[test]
fn confine_allows_beneath_the_subtree() {
    let (result, violations) = run_module_check(
        "confine-beneath",
        &[
            ("lib.rs", "pub mod ffi;\n"),
            ("ffi.rs", "pub mod raw;\n"),
            ("ffi/raw.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a module beneath the permitted subtree is clean: {violations:?}"
    );
}

#[test]
fn confine_flags_a_prefix_colliding_sibling_of_the_subtree() {
    // `crate::ffi_utils` is neither `crate::ffi` nor beneath `crate::ffi::` (`::`-delimited).
    let (result, violations) = run_module_check(
        "confine-sibling",
        &[
            ("lib.rs", "pub mod ffi;\npub mod ffi_utils;\n"),
            ("ffi.rs", "\n"),
            ("ffi_utils.rs", "use libc::c_int;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::ffi_utils");
}

#[test]
fn confine_observes_a_glob_and_a_bare_import() {
    let (result, violations) = run_module_check(
        "confine-glob",
        &[
            ("lib.rs", "pub mod ffi;\npub mod a;\npub mod b;\n"),
            ("ffi.rs", "\n"),
            ("a.rs", "use libc::*;\n"),
            ("b.rs", "use libc;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(
        findings.contains(&"crate::a") && findings.contains(&"crate::b"),
        "{findings:?}"
    );
}

#[test]
fn confine_ignores_a_different_external_crate() {
    let (result, violations) = run_module_check(
        "confine-other-crate",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use serde::Deserialize;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the rule observes only the confined crate: {violations:?}"
    );
}

#[test]
fn confine_of_an_unimported_crate_is_clean_with_no_error() {
    // No cargo-metadata cross-check: confining a crate the target never imports is clean,
    // exactly as forbidding a crate you do not depend on is clean — never a constitution error.
    let (result, violations) = run_module_check(
        "confine-unimported",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use serde::Deserialize;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(
        result.is_ok(),
        "no error for an unimported confined crate: {result:?}"
    );
    assert!(violations.is_empty(), "{violations:?}");
}

#[test]
fn confine_resolves_a_shadowing_root_module_as_internal_no_false_positive() {
    // A crate-root `mod libc;` shadows the extern prelude, so a root-file bare `use libc::…`
    // is the INTERNAL `crate::libc`, not the external crate — the external scan must not
    // observe it, or the rule would false-positive against the crate's own module.
    let (result, violations) = run_module_check(
        "confine-shadow",
        &[
            ("lib.rs", "pub mod libc;\npub mod ffi;\nuse libc::helper;\n"),
            ("libc.rs", "pub fn helper() {}\n"),
            ("ffi.rs", "\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a shadowing root module resolves internal, not as the confined external crate: {violations:?}"
    );
}

#[test]
fn confine_observes_submodule_bare_and_leading_colon_forms() {
    let (result, violations) = run_module_check(
        "confine-forms",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\npub mod other;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
            ("other.rs", "use ::libc::c_void;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    let findings: Vec<&str> = violations.iter().map(|v| v.finding.as_str()).collect();
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(
        findings.contains(&"crate::service") && findings.contains(&"crate::other"),
        "both a submodule-bare and a leading-`::` external import are observed: {findings:?}"
    );
}

#[test]
fn confine_ignores_a_use_inside_a_string_literal() {
    let (result, violations) = run_module_check(
        "confine-string",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            (
                "service.rs",
                "pub fn f() { let _s = \"use libc::c_int;\"; }\n",
            ),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a use inside a string literal is stripped before scanning: {violations:?}"
    );
}

#[test]
fn confine_dedups_multiple_imports_from_one_importer() {
    let (result, violations) = run_module_check(
        "confine-dedup",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\nuse libc::c_void;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "one importer importing the confined crate twice yields one violation: {violations:?}"
    );
}

#[test]
fn confine_identity_is_injective_across_confined_crates() {
    // The highest-risk invariant: two confinements of different crates on the same subtree,
    // breached by the same importer, must NOT collapse — baselining one must not mask the
    // other. Injectivity comes from the target being the confined crate.
    let files = [
        ("lib.rs", "pub mod ffi;\npub mod service;\n"),
        ("ffi.rs", "\n"),
        ("service.rs", "use libc::c_int;\nuse winapi::HANDLE;\n"),
    ];
    let (r1, libc_v) = run_module_check("confine-inj-libc", &files, confine("crate::ffi", "libc"));
    let (r2, winapi_v) = run_module_check(
        "confine-inj-winapi",
        &files,
        confine("crate::ffi", "winapi"),
    );
    assert!(r1.is_ok() && r2.is_ok(), "{r1:?} {r2:?}");
    assert_eq!(libc_v.len(), 1, "{libc_v:?}");
    assert_eq!(winapi_v.len(), 1, "{winapi_v:?}");
    assert_eq!(libc_v[0].target, "libc");
    assert_eq!(winapi_v[0].target, "winapi");
    assert_eq!(
        libc_v[0].finding, winapi_v[0].finding,
        "same offending importer — only the target (confined crate) differs"
    );
    // Baseline only the libc violation; the winapi violation must still react.
    let baseline = Baseline::of(&Report::new(libc_v.clone()));
    let mut both = Report::new(vec![libc_v[0].clone(), winapi_v[0].clone()]);
    apply_baseline(&mut both, &baseline);
    assert!(both.violations[0].baselined, "libc is baselined");
    assert!(
        !both.violations[1].baselined,
        "winapi is NOT masked by libc's baseline — distinct target keeps identity injective"
    );
    assert_eq!(
        Outcome::Violations(both).exit_code(),
        1,
        "the new winapi violation still fails"
    );
}

#[test]
fn confine_canonicalizes_a_raw_identifier_crate_name() {
    // The confined crate name and the observed head canonicalize (`r#name` -> `name`), so a
    // boundary written with either form matches an import written with either form.
    let (result, violations) = run_module_check(
        "confine-raw-id",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use r#async::spawn;\n"),
        ],
        confine("crate::ffi", "async"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "raw and plain crate-name forms are one identity: {violations:?}"
    );
    assert_eq!(violations[0].target, "async");
}

#[test]
fn confine_on_the_crate_root_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "confine-on-crate",
        &[("lib.rs", "use libc::c_int;\n")],
        confine("crate", "libc"),
    );
    let err = result.expect_err("confining to the crate root is a constitution error");
    assert_eq!(err, confine_external_crate_on_crate_error("x"));
}

#[test]
fn confine_unknown_subtree_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "confine-unknown",
        &[("lib.rs", "pub mod ffi;\n"), ("ffi.rs", "\n")],
        confine("crate::nope", "libc"),
    );
    let err = result.expect_err("an unreachable permitted subtree is a constitution error");
    assert_eq!(err, unknown_module_error("crate::nope", "x"));
}

#[test]
fn confine_honors_warn_severity() {
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::ffi")
        .confine_external_crate("libc")
        .warn()
        .because("advisory during adoption");
    let (result, violations) = run_module_check(
        "confine-warn",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc::c_int;\n"),
        ],
        boundary,
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
    assert_eq!(
        Outcome::Violations(Report::new(violations)).exit_code(),
        0,
        "a warn-only confinement does not fail CI"
    );
}

#[test]
fn confine_external_crate_rule_text_and_json_params() {
    let rule = ModuleRule::ConfineExternalCrate {
        crate_name: "libc".to_string(),
    };
    assert_eq!(rule.label(), "external crate confined to module");
    assert_eq!(rule.polarity(), Polarity::AllowlistGap);
    assert_eq!(
        rule.text(),
        "confines external crate libc to this module's subtree"
    );
    assert_eq!(
        rule.json_params(),
        vec![("external_crate", serde_json::json!("libc"))]
    );
}

#[test]
fn confine_external_crate_projects_its_crate_and_subtree() {
    // The full constitution projection (not just the rule params): the declared module
    // subtree and the confined crate must both be legible without reading the rule label.
    let constitution = Constitution::new("p").boundary(
        ModuleBoundary::in_crate("app")
            .module("crate::ffi")
            .confine_external_crate("libc")
            .because("the raw libc surface stays behind the ffi module"),
    );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("crate::ffi"),
        "the declared subtree is named: {text}"
    );
    assert!(
        text.contains("confines external crate libc"),
        "the confined crate is named: {text}"
    );

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "external crate confined to module"
    );
    // The declared module subtree is the boundary target (declaration view); the confined
    // crate projects under the self-describing `external_crate` key.
    assert_eq!(doc["boundaries"][0]["target"], "crate::ffi");
    assert_eq!(doc["boundaries"][0]["external_crate"], "libc");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    assert!(doc["boundaries"][0]["only"].is_null());
}

#[test]
fn confine_matches_a_hyphenated_package_name_against_its_underscore_identifier() {
    // A package with a `-` (e.g. `windows-sys`) is imported through its `_` identifier
    // (`use windows_sys::…`) — a `use` path cannot contain `-`. The confined name is written in
    // package form yet must match the identifier the scanner observes, or the rule would silently
    // never react for exactly the hyphenated FFI/platform crates it targets (a false negative).
    let (result, violations) = run_module_check(
        "confine-hyphen",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            (
                "service.rs",
                "use windows_sys::Win32::Foundation::HANDLE;\n",
            ),
        ],
        confine("crate::ffi", "windows-sys"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a hyphenated package name matches its underscore import identifier: {violations:?}"
    );
    assert_eq!(violations[0].target, "windows_sys");
    assert_eq!(violations[0].finding, "crate::service");
}

#[test]
fn confine_observes_an_aliased_import_of_the_confined_crate() {
    // `use libc as c;` still imports libc; the alias is dropped at expansion, so the confined
    // crate's head is observed regardless of a local rename.
    let (result, violations) = run_module_check(
        "confine-alias",
        &[
            ("lib.rs", "pub mod ffi;\npub mod service;\n"),
            ("ffi.rs", "\n"),
            ("service.rs", "use libc as c;\n"),
        ],
        confine("crate::ffi", "libc"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "an aliased import is still observed: {violations:?}"
    );
    assert_eq!(violations[0].target, "libc");
    assert_eq!(violations[0].finding, "crate::service");
}

// --- inline-symbol-path confinement (`must_not_call_inline`) ----------------------------

fn confine_core_clock() -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_call_inline("std::time")
        .because("core reads no wall clock — time is injected, not read")
}

#[test]
fn inline_default_reacts_on_an_associated_fn_call() {
    let (result, violations) = run_module_check(
        "inline-call",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn stamp() { let _ = std::time::SystemTime::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "std::time");
    assert!(
        violations[0].finding.contains("std::time::SystemTime::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_default_passes_a_type_annotation() {
    let (result, violations) = run_module_check(
        "inline-annot",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn handle(now: std::time::Instant) { let _ = now; }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a type annotation is not a call: {violations:?}"
    );
}

#[test]
fn inline_default_passes_a_constant() {
    let (result, violations) = run_module_check(
        "inline-const",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = std::time::SystemTime::UNIX_EPOCH; }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a constant read is not a call: {violations:?}"
    );
}

#[test]
fn inline_resolves_a_rename() {
    let (result, violations) = run_module_check(
        "inline-rename",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "use std::time::SystemTime as SysT;\nfn f() { let _ = SysT::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a renamed alias resolves: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("std::time::SystemTime::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_resolves_a_self_prefixed_group_alias() {
    // A use-group member whose name merely *starts with* the substring "self" (`self_utc`) is a
    // legal import, not the `self` leaf. An over-broad `starts_with("self")` dropped it, so the
    // alias was unresolved and a confined inline call through it silently passed — a false negative.
    let (result, violations) = run_module_check(
        "inline-self-prefixed-group",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "use std::time::{self_utc as clk, Duration};\nfn f() { let _ = clk::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a self-prefixed group alias resolves and reacts: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("std::time::self_utc::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_resolves_a_bare_path() {
    let (result, violations) = run_module_check(
        "inline-bare",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "use std::time;\nfn f() { let _ = time::Instant::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "a bare path resolves: {violations:?}");
    assert!(
        violations[0].finding.contains("std::time::Instant::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_resolves_a_type_alias() {
    let (result, violations) = run_module_check(
        "inline-typealias",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "type Clock = std::time::SystemTime;\nfn f() { let _ = Clock::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "a type alias resolves: {violations:?}");
}

#[test]
fn inline_resolves_a_multi_hop_type_alias() {
    let (result, violations) = run_module_check(
        "inline-multihop",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "type A = std::time::SystemTime;\ntype B = A;\nfn f() { let _ = B::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a multi-hop type alias chases to a fixpoint: {violations:?}"
    );
}

#[test]
fn inline_resolves_a_type_alias_past_a_defaulted_generic_param() {
    // The generic parameter list carries its own `=` (`Tz = LocalTz`); it must not be mistaken for
    // the alias `=`, or the alias resolves to the default (`LocalTz`) instead of its real target
    // (`std::time::SystemTime`) — a silent miss of the confined clock (a false negative).
    let (result, violations) = run_module_check(
        "inline-defaulted-generic-alias",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "type Clock<Tz = LocalTz> = std::time::SystemTime;\nfn f() { let _ = Clock::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "the alias resolves past the defaulted generic param to its real target: {violations:?}"
    );
}

#[test]
fn inline_resolves_a_cross_module_local_reexport() {
    let (result, violations) = run_module_check(
        "inline-reexport",
        &[
            ("lib.rs", "pub mod core;\npub mod support;\n"),
            ("support.rs", "pub use std::time::SystemTime;\n"),
            (
                "core.rs",
                "use crate::support::SystemTime;\nfn f() { let _ = SystemTime::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a cross-module local re-export resolves: {violations:?}"
    );
}

#[test]
fn inline_does_not_match_an_unresolved_same_named_local() {
    let (result, violations) = run_module_check(
        "inline-local",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "struct Instant;\nimpl Instant { fn now() {} }\nfn f() { Instant::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a same-named local is not matched by leaf: {violations:?}"
    );
}

#[test]
fn inline_glob_of_the_prefix_reacts() {
    let (result, violations) = run_module_check(
        "inline-glob",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "use std::time::*;\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a glob of the prefix reacts fail-closed: {violations:?}"
    );
    assert!(violations[0].finding.contains("glob"), "{violations:?}");
}

#[test]
fn inline_glob_above_the_prefix_reacts() {
    let (result, violations) = run_module_check(
        "inline-glob-above",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "use std::*;\nfn f() { let _ = time::Instant::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.iter().any(|v| v.finding.contains("glob")),
        "an ancestor glob reacts: {violations:?}"
    );
}

#[test]
fn inline_glob_of_a_local_reexporting_module_reacts() {
    let (result, violations) = run_module_check(
        "inline-glob-local",
        &[
            ("lib.rs", "pub mod core;\npub mod support;\n"),
            ("support.rs", "pub use std::time::SystemTime;\n"),
            ("core.rs", "use crate::support::*;\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a glob of a local re-exporting module reacts: {violations:?}"
    );
}

#[test]
fn inline_glob_of_a_module_that_itself_globs_the_prefix_reacts() {
    let (result, violations) = run_module_check(
        "inline-glob-recursive",
        &[
            ("lib.rs", "pub mod core;\npub mod support;\n"),
            ("support.rs", "pub use std::time::*;\n"),
            ("core.rs", "use crate::support::*;\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "recursive glob hazard reacts: {violations:?}"
    );
}

#[test]
fn inline_narrowing_drops_a_benign_constructor_and_keeps_the_read() {
    let (result, violations) = run_module_check(
        "inline-narrow",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = std::time::Instant::now(); let _ = std::time::Duration::from_secs(5); }\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(["now"])
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "only the now-read reacts under narrowing: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("Instant::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_narrowing_does_not_suppress_a_glob() {
    let (result, violations) = run_module_check(
        "inline-narrow-glob",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "use std::time::*;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(["now"])
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a glob still reacts under narrowing: {violations:?}"
    );
}

#[test]
fn inline_strict_flags_a_type_annotation() {
    let (result, violations) = run_module_check(
        "inline-strict",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn handle(now: std::time::Instant) { let _ = now; }\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .strict_prefix_only()
            .because("core may not name std::time at all"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "strict flags a mention: {violations:?}"
    );
}

#[test]
fn inline_value_capture_is_a_bound_under_the_default() {
    let (result, violations) = run_module_check(
        "inline-valuecap",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let g = std::time::SystemTime::now; let _ = g; }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "value-position capture is a stated bound under the default: {violations:?}"
    );
}

#[test]
fn inline_scans_a_macro_body() {
    let (result, violations) = run_module_check(
        "inline-macro",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { some_macro! { let _ = std::time::Instant::now(); } }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a read inside a macro body is scanned, not skipped: {violations:?}"
    );
}

#[test]
fn inline_empty_prefix_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inline-empty",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", "// clean\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("")
            .because("bad"),
    );
    assert_eq!(result.unwrap_err(), inline_empty_prefix_error("x"));
}

#[test]
fn inline_narrow_and_strict_is_a_constitution_error() {
    let (result, _violations) = run_module_check(
        "inline-combo",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", "// clean\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(["now"])
            .strict_prefix_only()
            .because("contradiction"),
    );
    assert_eq!(result.unwrap_err(), inline_narrow_and_strict_error("x"));
}

#[test]
fn inline_valid_zero_match_is_clean() {
    let (result, violations) = run_module_check(
        "inline-clean",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "fn f() { let _ = std::cmp::max(1, 2); }\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a subtree with no std::time call is clean: {violations:?}"
    );
}

// --- inline-symbol-path: strict-external opt-in (`.strict_external()`) -------------------

/// A strict-external confinement on `chrono::Utc`, with `chrono` declared as a dependency. `name`
/// keys a per-test temp dir (must be unique, since tests run in parallel).
fn confine_chrono_strict(
    name: &str,
    files: &[(&str, &str)],
) -> (Result<(), String>, Vec<Violation>) {
    run_module_check_with_deps(
        name,
        files,
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .strict_external()
            .because("core reads no wall clock — time is injected"),
    )
}

#[test]
fn inline_strict_external_reacts_on_a_fully_qualified_external_call() {
    // 4.1 Guard (FN close): a fully-qualified, un-`use`d `chrono::Utc::now()` REACTS under the flag.
    let (result, violations) = confine_chrono_strict(
        "inline-strict-ext-fq",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "fn stamp() { let _ = chrono::Utc::now(); }\n"),
        ],
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target, "chrono::Utc");
    assert!(
        violations[0].finding.contains("chrono::Utc::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_absent_fully_qualified_call_is_a_bound() {
    // 4.2 Default unchanged: the SAME call without the flag does NOT react (stated bound).
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-default",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "fn stamp() { let _ = chrono::Utc::now(); }\n"),
        ],
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a fully-qualified external call is a stated bound under the default: {violations:?}"
    );
}

#[test]
fn inline_strict_external_deep_local_module_stays_clean() {
    // 4.3 FP safety — a DEEP local module (non-crate-root) named like the dependency wins by local
    // precedence (rung iii at depth), NOT the crate-root shadow.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-deepmod",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "pub mod time;\nfn f() { let _ = time::format(); }\n",
            ),
            ("core/time.rs", "pub fn format() {}\n"),
        ],
        &[("time", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("time")
            .strict_external()
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a deep local module named like a dep stays local: {violations:?}"
    );
}

#[test]
fn inline_strict_external_local_fn_definition_stays_clean() {
    // 4.4 FP safety — a local `fn` named like the dependency wins by local precedence (rung iv).
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-localfn",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn rand() -> u32 { 4 }\nfn f() { let _ = rand(); }\n",
            ),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a local fn shadowing a dep name stays local: {violations:?}"
    );
}

#[test]
fn inline_strict_external_local_alias_stays_clean() {
    // 4.5 FP safety — a local `use crate::clock as time;` alias resolves through the use-map
    // (rung i, which precedes the dependency match) and stays clean.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-alias",
        &[
            ("lib.rs", "pub mod core;\npub mod clock;\n"),
            ("clock.rs", "pub fn read() {}\n"),
            (
                "core.rs",
                "use crate::clock as time;\nfn f() { let _ = time::read(); }\n",
            ),
        ],
        &[("time", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("time")
            .strict_external()
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a local alias shadowing a dep name resolves local: {violations:?}"
    );
}

#[test]
fn inline_strict_external_glob_reacts_and_default_glob_does_not() {
    // 4.6 An external-crate glob `use chrono::*;` reacts under the flag (an external glob is an
    // ancestor of the confined prefix); the same glob under the default does NOT react.
    let files = &[
        ("lib.rs", "pub mod core;\n"),
        ("core.rs", "use chrono::*;\n"),
    ];
    let (result, violations) = confine_chrono_strict("inline-strict-ext-glob", files);
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "external glob reacts: {violations:?}");
    assert!(violations[0].finding.contains("glob"), "{violations:?}");

    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-glob-default",
        files,
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "the same glob under the default resolves local and does not react: {violations:?}"
    );
}

#[test]
fn inline_strict_external_composes_with_narrowing() {
    // 4.7 `.strict_external().ending_with(["now"])` reacts on `now()` and not on `today()`.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-narrow",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = chrono::Utc::now(); let _ = chrono::Utc::today(); }\n",
            ),
        ],
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .strict_external()
            .ending_with(["now"])
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "only the now-read reacts under narrowing: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("chrono::Utc::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_extern_crate_rename_is_a_stated_bound() {
    // 4.8 `extern crate chrono as chr; chr::Utc::now()` does NOT react (stated bound — the use-map
    // reads `use` only), while the bare `chrono::Utc::now()` in the same subtree does.
    let (result, violations) = confine_chrono_strict(
        "inline-strict-ext-extern",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "extern crate chrono as chr;\nfn a() { let _ = chr::Utc::now(); }\nfn b() { let _ = chrono::Utc::now(); }\n",
            ),
        ],
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "only the real-name call reacts; the extern-crate-as rename is a bound: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("chrono::Utc::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_adds_nothing_to_paths_that_already_react() {
    // 4.9 A `use chrono::Utc; Utc::now()` reacts WITHOUT the flag; and a cross-module
    // `pub use chrono::Utc;` chased to `Utc::now()` reacts WITHOUT the flag — the flag adds nothing.
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod core;\npub mod support;\n"),
        ("support.rs", "pub use chrono::Utc;\n"),
        (
            "core.rs",
            "use chrono::Utc;\nuse crate::support::Utc as SupUtc;\nfn f() { let _ = Utc::now(); let _ = SupUtc::now(); }\n",
        ),
    ];
    // Without the flag.
    let (result, plain) = run_module_check_with_deps(
        "inline-strict-ext-already-default",
        files,
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        plain.len(),
        1,
        "the used import + chased re-export already react under the default: {plain:?}"
    );
    // With the flag — same finding count (adds nothing, no over-reach, no double count).
    let (result, flagged) = run_module_check_with_deps(
        "inline-strict-ext-already-flag",
        files,
        &[("chrono", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .strict_external()
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        flagged.len(),
        plain.len(),
        "the flag adds nothing to paths that already react: {flagged:?}"
    );
}

#[test]
fn inline_strict_external_preserves_identity_no_baseline_churn() {
    // 4.10 Baseline-churn guard: a sysroot `std::time::…::now()` finding must have byte-identical
    // (target, rule, finding) whether or not `.strict_external()` is added — so a baselined finding
    // survives the flag (identity parity, task 1.3). Locks target/rule-key/fact.
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod core;\n"),
        (
            "core.rs",
            "fn f() { let _ = std::time::SystemTime::now(); }\n",
        ),
    ];
    let (r1, plain) = run_module_check(
        "inline-identity-default",
        files,
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .because("core reads no wall clock"),
    );
    let (r2, flagged) = run_module_check_with_deps(
        "inline-identity-flag",
        files,
        &[],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .strict_external()
            .because("core reads no wall clock"),
    );
    assert!(r1.is_ok() && r2.is_ok(), "{r1:?} {r2:?}");
    assert_eq!(plain.len(), 1, "{plain:?}");
    assert_eq!(flagged.len(), 1, "{flagged:?}");
    assert_eq!(plain[0].target, flagged[0].target, "target parity");
    assert_eq!(plain[0].rule, flagged[0].rule, "rule (label) parity");
    assert_eq!(plain[0].finding, flagged[0].finding, "finding parity");
    assert_eq!(
        plain[0].rule, "inline symbol path confined to module",
        "the presentation label is unchanged by the flag"
    );
}

#[test]
fn inline_strict_external_runs_the_exit_2_checks() {
    // 4.11 The new variant must still run the exit-2 constitution checks, never silently skip them.
    // Contradictory triple → narrow-and-strict error.
    let (contradiction, _) = run_module_check_with_deps(
        "inline-strict-ext-contradiction",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", "// clean\n")],
        &[("std", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(["now"])
            .strict_prefix_only()
            .strict_external()
            .because("contradiction"),
    );
    assert_eq!(
        contradiction.unwrap_err(),
        inline_narrow_and_strict_error("x")
    );
    // Empty prefix → empty-prefix error.
    let (empty, _) = run_module_check_with_deps(
        "inline-strict-ext-empty",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", "// clean\n")],
        &[],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("")
            .strict_external()
            .because("bad"),
    );
    assert_eq!(empty.unwrap_err(), inline_empty_prefix_error("x"));
}

#[test]
fn inline_strict_external_cross_module_local_item_does_not_mask() {
    // Apply-review finding 1 (cardinal false negative): the item-definition set MUST be
    // module-qualified. A `fn rand` in `crate::helpers` must NOT suppress a real external
    // `rand::random()` call in the governed `crate::core` (a different module). Pre-fix, the set was
    // crate-flat and this call was silently passed (FN); this guard reacts.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-xmod",
        &[
            ("lib.rs", "pub mod core;\npub mod helpers;\n"),
            ("helpers.rs", "pub fn rand() -> u32 { 4 }\n"),
            ("core.rs", "fn f() { let _ = rand::random(); }\n"),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a same-named item of ANOTHER module must not mask a real external call: {violations:?}"
    );
    assert_eq!(violations[0].target, "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_block_local_item_does_not_mask() {
    // Apply-review finding 1 residual: only MODULE-TOP-LEVEL items shadow a bare head. A block-local
    // `const log` (brace depth ≥ 1) is NOT reachable as a bare head, so it must NOT suppress a real
    // external `log::logger()` call in the same module. Pre-fix (capture-all depth), the nested name
    // was captured and silently masked the call (a false negative); this guard reacts.
    // (A colliding *method*/nested `fn log` is instead a stated over-reaction bound — its definition
    // site `log(` reads as a call under a single-segment prefix — so this uses a non-call-shaped
    // `const` to isolate the depth-exclusion behaviour.)
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-blocklocal",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() {\n    const log: u32 = 3;\n    let _ = log::logger();\n}\n",
            ),
        ],
        &[("log", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("log")
            .strict_external()
            .because("core does not log"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a block-local item (depth ≥ 1) must not mask a same-module external call: {violations:?}"
    );
    assert_eq!(violations[0].target, "log");
    assert!(
        violations[0].finding.contains("log::logger"),
        "{violations:?}"
    );
}

// --- inline-symbol-path: adversarial-review regression + coverage ------------------------

#[test]
fn inline_reacts_on_a_leading_colon_path() {
    // A leading `::std::time::…::now()` must be extracted (its head sits after `::`).
    let (result, violations) = run_module_check(
        "inline-leading-colon",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = ::std::time::SystemTime::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a leading-:: call reacts: {violations:?}"
    );
    assert!(
        violations[0].finding.contains("std::time::SystemTime::now"),
        "{violations:?}"
    );
}

#[test]
fn inline_reacts_on_a_nested_grouped_glob() {
    // `use std::{time::*, cmp::max}` — the nested glob member `time::*` reaches under
    // the prefix and must react fail-closed, though it is not a top-level `*`.
    let (result, violations) = run_module_check(
        "inline-nested-glob",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "use std::{cmp::max, time::*};\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.iter().any(|v| v.finding.contains("glob")),
        "a nested grouped glob of the prefix reacts: {violations:?}"
    );
}

#[test]
fn inline_reacts_on_a_two_hop_use_realias() {
    // `use std::time::SystemTime; use self::SystemTime as Clock;` — the second use-hop
    // must chase through the file's own use-map, not only the crate-wide def closure.
    let (result, violations) = run_module_check(
        "inline-two-hop-use",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "use std::time::SystemTime;\nuse self::SystemTime as Clock;\nfn f() { let _ = Clock::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a two-hop use re-alias resolves to a fixpoint: {violations:?}"
    );
}

#[test]
fn inline_reacts_through_a_mid_path_turbofish() {
    // `Clock::<Utc>::now()` — the mid-path turbofish must not break the path, and the
    // terminal `now` call must still react (via the resolved `std::time::SystemTime::now`).
    let (result, violations) = run_module_check(
        "inline-turbofish",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "type Clock = std::time::SystemTime;\nfn f() { let _ = Clock::<u8>::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a mid-path turbofish call reacts: {violations:?}"
    );
}

#[test]
fn inline_reacts_through_interior_whitespace_and_field_colon() {
    // Interior whitespace in the path, and a no-space struct-field `:` before a path.
    let (result, violations) = run_module_check(
        "inline-ws",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = std :: time :: Instant :: now(); }\nstruct E { at: std::time::SystemTime }\nfn g() -> E { E { at:std::time::SystemTime::now() } }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    // Two distinct now-reads (Instant::now, SystemTime::now); the `at: SystemTime` annotation is a
    // non-call mention and does not react.
    assert_eq!(
        violations.len(),
        2,
        "interior-whitespace and field-colon calls both react: {violations:?}"
    );
}

#[test]
fn inline_ufcs_is_a_documented_bound_under_the_default() {
    // Stated bound: a UFCS-qualified call `<Type as Trait>::now()` puts the type inside `<…>`, not
    // a plain path — like a receiver-method read, out of scope under the default (strict catches
    // the mention). Asserted non-reaction so the bound is a declared non-observation, not silent.
    let (result, violations) = run_module_check(
        "inline-ufcs",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "trait Now { fn now(); }\nfn f() { <std::time::SystemTime as Now>::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "UFCS is a stated bound under the default (type in <…>): {violations:?}"
    );
}

#[test]
fn inline_receiver_method_read_is_a_bound() {
    // Stated bound: `inst.elapsed()` — the receiver's type is not in the written path (no type
    // inference), so it is out of scope. Asserted non-reaction (declared, not silent).
    let (result, violations) = run_module_check(
        "inline-receiver",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f(inst: std::time::Instant) { let _ = inst.elapsed(); }\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(["now", "elapsed"])
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a receiver-method read is a stated bound (type not in path): {violations:?}"
    );
}

#[test]
fn inline_grouped_self_glob_reacts() {
    let (result, violations) = run_module_check(
        "inline-selfglob",
        &[
            ("lib.rs", "pub mod core;\n"),
            ("core.rs", "use std::time::{self, Duration, *};\n"),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.iter().any(|v| v.finding.contains("glob")),
        "a grouped `{{self, *}}` glob reacts: {violations:?}"
    );
}

#[test]
fn inline_two_distinct_calls_stay_distinct() {
    // Identity: two distinct canonical calls in one module are two findings (no dedup masking).
    let (result, violations) = run_module_check(
        "inline-distinct",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = std::time::Instant::now(); let _ = std::time::SystemTime::now(); }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        2,
        "two distinct canonical calls stay distinct findings: {violations:?}"
    );
}

#[test]
fn inline_prefix_is_carried_in_the_violation_target() {
    // Identity: the confined prefix is the violation target, so nested-prefix confinements (`std`
    // vs `std::time`) breached by the same call never share an identity (no baseline masking).
    let files: &[(&str, &str)] = &[
        ("lib.rs", "pub mod core;\n"),
        ("core.rs", "fn f() { let _ = std::time::Instant::now(); }\n"),
    ];
    let (r1, v1) = run_module_check(
        "inline-target-time",
        files,
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .because("no clock"),
    );
    let (r2, v2) = run_module_check(
        "inline-target-std",
        files,
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std")
            .because("no std calls"),
    );
    assert!(r1.is_ok() && r2.is_ok(), "{r1:?} {r2:?}");
    assert_eq!(v1[0].target, "std::time");
    assert_eq!(v2[0].target, "std");
    assert_ne!(
        v1[0].target, v2[0].target,
        "distinct prefixes → distinct identity"
    );
}

#[test]
fn inline_warn_severity_is_advisory() {
    let (result, violations) = run_module_check(
        "inline-warn",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { let _ = std::time::SystemTime::now(); }\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .warn()
            .because("advisory"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].severity, Severity::Warn);
}

#[test]
fn inline_empty_verbs_is_a_constitution_error() {
    let (result, _v) = run_module_check(
        "inline-emptyverbs",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", "// clean\n")],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .ending_with(Vec::<String>::new())
            .because("bad"),
    );
    assert_eq!(result.unwrap_err(), inline_empty_verbs_error("x"));
}

#[test]
fn inline_scanner_does_not_panic_or_hang_on_odd_input() {
    // Robustness: malformed `use`/brace/self-referential-alias input must never panic or hang.
    for body in [
        "use } {;\n",
        "use std::{time::*;\n",
        "type A = A::B;\nfn f() { let _ = A::now(); }\n",
        "use ::;\nfn f() { ::::(); }\n",
        "fn f() { <>::(); std :: :: now (); }\n",
    ] {
        let (result, _v) = run_module_check(
            "inline-odd",
            &[("lib.rs", "pub mod core;\n"), ("core.rs", body)],
            confine_core_clock(),
        );
        // Either clean or a violation, but it must complete (no panic / no hang) and not error out.
        assert!(
            result.is_ok(),
            "odd input must not error: {body:?} -> {result:?}"
        );
    }
}

#[test]
fn inline_in_macro_body_alias_is_a_bound() {
    // Stated bound: an alias DEFINED INSIDE a macro body is not in the enclosing use-map, so a
    // call through it inside the same macro body does not resolve — a declared non-observation
    // (the macro body IS scanned for direct paths, but a body-local alias is out of scope).
    let (result, violations) = run_module_check(
        "inline-macro-alias",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn f() { some_macro! { use std::time::SystemTime as X; let _ = X::now(); } }\n",
            ),
        ],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "an alias defined inside a macro body is a stated bound: {violations:?}"
    );
}

#[test]
fn inline_strict_external_inline_submodule_call_is_not_masked() {
    // Cardinal false-negative guard (apply-review finding 1): a file-top `fn rand` must NOT mask a real external
    // `rand::random()` call inside an inline `mod tests { … }`. The call's TRUE module is
    // `crate::core::tests`, so the file-top item `crate::core::rand` cannot claim its head — the
    // external match fires. Pre-fix the call scan tracked no inline-`mod` nesting, so the file-top
    // item silently shadowed the submodule call (the bug the review caught); this guard reacts.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-submod-call",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn rand() -> u32 { 4 }\nmod tests { fn t() { let _ = rand::random(); } }\n",
            ),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a file-top item must not mask an external call inside an inline submodule: {violations:?}"
    );
    assert_eq!(violations[0].target, "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_submodule_local_item_stays_clean() {
    // Bonus FP guard: a submodule-local `fn rand` IS now captured under its true module
    // (`crate::core::tests::rand`), so a bare `rand()` call in that same submodule resolves to the
    // local item and stays clean. (Pre-fix the item was at brace depth ≥ 1 and never captured, so
    // this could false-positive; the inline-aware keying closes it.)
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-submod-local",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "mod tests { fn rand() -> u32 { 4 } fn t() { let _ = rand(); } }\n",
            ),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.is_empty(),
        "a submodule-local item named like a dep must claim its own submodule's call: {violations:?}"
    );
}

#[test]
fn inline_strict_external_deeply_nested_submodule_reacts() {
    // The inline-`mod` stack composes to any depth: a file-top `fn rand` cannot mask a
    // `rand::random()` call two submodules deep (`crate::core::a::b`), so the external match fires.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-nested",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn rand() -> u32 { 4 }\nmod a { mod b { fn t() { let _ = rand::random(); } } }\n",
            ),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a file-top item must not mask a call in a deeply nested submodule: {violations:?}"
    );
    assert_eq!(violations[0].target, "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_cfg_gated_submodule_reacts() {
    // A `#[cfg(test)]` attribute on the inline `mod` carries only `(…)`/`[…]` — no `{`/`}` — so it
    // does not perturb the brace-depth tracking: the `mod tests { … }` body is still entered and the
    // `rand::random()` call inside it is attributed to `crate::core::tests`, unmasked by the
    // file-top `fn rand`.
    let (result, violations) = run_module_check_with_deps(
        "inline-strict-ext-cfg-gated",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "fn rand() -> u32 { 4 }\n#[cfg(test)]\nmod tests { fn t() { let _ = rand::random(); } }\n",
            ),
        ],
        &[("rand", None)],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("rand")
            .strict_external()
            .because("core is deterministic"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(
        violations.len(),
        1,
        "a cfg-gated inline submodule must not perturb brace tracking: {violations:?}"
    );
    assert_eq!(violations[0].target, "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
fn inline_strict_external_default_path_module_attribution_unshifted() {
    // Default-path byte-identity: a NON-strict inline confinement whose call sits inside an inline
    // `mod tests { … }` must still attribute the finding to the FILE module (`crate::core`), NOT the
    // submodule — proving the new per-occurrence inline module is computed only under the flag and
    // never leaks into default attribution.
    let (result, violations) = run_module_check(
        "inline-default-attr",
        &[
            ("lib.rs", "pub mod core;\n"),
            (
                "core.rs",
                "mod tests { fn t() { let _ = std::time::SystemTime::now(); } }\n",
            ),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::time")
            .because("core reads no wall clock"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert!(
        violations[0].finding.ends_with("in crate::core"),
        "default attribution must stay on the FILE module, not the inline submodule: {violations:?}"
    );
}
