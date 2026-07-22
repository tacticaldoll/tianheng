use std::path::PathBuf;

use serde_json::json;

use super::*;

#[test]
fn find_package_selects_by_name() {
    let metadata = json!({ "packages": [
        { "name": "a", "targets": [] },
        { "name": "b", "targets": [] },
    ]});
    assert_eq!(find_package(&metadata, "b").unwrap()["name"], json!("b"));
    assert!(find_package(&metadata, "missing").is_none());
}

#[test]
fn crate_root_file_prefers_lib_then_proc_macro_then_bin() {
    let lib_and_bin = json!({ "targets": [
        { "kind": ["bin"], "src_path": "/w/src/main.rs" },
        { "kind": ["lib"], "src_path": "/w/src/lib.rs" },
    ]});
    assert_eq!(
        crate_root_file(&lib_and_bin),
        Some(PathBuf::from("/w/src/lib.rs")),
        "the lib target wins over bin"
    );

    let bin_only = json!({ "targets": [{ "kind": ["bin"], "src_path": "/w/src/main.rs" }] });
    assert_eq!(
        crate_root_file(&bin_only),
        Some(PathBuf::from("/w/src/main.rs"))
    );
}

#[test]
fn crate_root_file_resolves_a_proc_macro_target() {
    let package = json!({ "targets": [
        { "kind": ["proc-macro"], "src_path": "/w/src/lib.rs" }
    ]});
    assert_eq!(
        crate_root_file(&package),
        Some(PathBuf::from("/w/src/lib.rs"))
    );
}

#[test]
fn crate_root_file_skips_a_member_with_no_lib_proc_macro_or_bin() {
    let bench_only = json!({ "targets": [{ "kind": ["bench"], "src_path": "/w/benches/b.rs" }] });
    assert_eq!(crate_root_file(&bench_only), None);
    let rootless = json!({ "targets": [] });
    assert_eq!(crate_root_file(&rootless), None);
}

#[test]
fn crate_root_file_resolves_a_cdylib_staticlib_or_rlib_library() {
    for kind in [["cdylib"], ["staticlib"], ["rlib"], ["dylib"]] {
        let package = json!({ "targets": [{ "kind": kind, "src_path": "/w/src/lib.rs" }] });
        assert_eq!(
            crate_root_file(&package),
            Some(PathBuf::from("/w/src/lib.rs")),
            "a {kind:?} library must resolve its crate root"
        );
    }
    let multi = json!({ "targets": [{ "kind": ["cdylib", "rlib"], "src_path": "/w/src/lib.rs" }] });
    assert_eq!(
        crate_root_file(&multi),
        Some(PathBuf::from("/w/src/lib.rs"))
    );
    let lib_and_bin = json!({ "targets": [
        { "kind": ["bin"], "src_path": "/w/src/main.rs" },
        { "kind": ["cdylib"], "src_path": "/w/src/lib.rs" },
    ]});
    assert_eq!(
        crate_root_file(&lib_and_bin),
        Some(PathBuf::from("/w/src/lib.rs"))
    );
}

#[test]
fn member_src_dirs_resolves_from_src_path_including_a_custom_layout() {
    let metadata = json!({
        "packages": [
            { "name": "crate_a", "targets": [
                { "kind": ["lib"], "src_path": "/ws/crate_a/src/lib.rs" }
            ]},
            { "name": "crate_b", "targets": [
                { "kind": ["lib"], "src_path": "/ws/crate_b/lib.rs" }
            ]},
            { "name": "crate_c", "targets": [
                { "kind": ["bin"], "src_path": "/ws/crate_c/src/main.rs" }
            ]},
        ]
    });
    let dirs = member_src_dirs(&metadata);
    assert!(dirs.contains(&PathBuf::from("/ws/crate_a/src")), "{dirs:?}");
    assert!(
        dirs.contains(&PathBuf::from("/ws/crate_b")),
        "a custom [lib] path must resolve to its real root, not manifest_dir/src: {dirs:?}"
    );
    assert!(dirs.contains(&PathBuf::from("/ws/crate_c/src")), "{dirs:?}");
}

#[test]
fn member_src_dirs_prefers_lib_over_bin_and_skips_rootless_members() {
    let metadata = json!({
        "packages": [
            { "name": "both", "targets": [
                { "kind": ["bin"], "src_path": "/ws/both/src/main.rs" },
                { "kind": ["lib"], "src_path": "/ws/both/src/lib.rs" }
            ]},
            { "name": "rootless", "targets": [] },
        ]
    });
    let dirs = member_src_dirs(&metadata);
    assert_eq!(dirs, vec![PathBuf::from("/ws/both/src")], "{dirs:?}");
}

#[test]
fn member_root_files_preserves_exact_custom_roots_and_is_deterministic() {
    let metadata = json!({ "packages": [
        { "targets": [
            { "kind": ["lib"], "src_path": "/ws/z/src/lib.rs" },
            { "kind": ["bin"], "src_path": "/ws/z/src/main.rs" }
        ] },
        { "targets": [{ "kind": ["lib"], "src_path": "/ws/a/custom_root.rs" }] },
        { "targets": [{ "kind": ["lib"], "src_path": "/ws/a/custom_root.rs" }] },
        { "targets": [{ "kind": ["test"], "src_path": "/ws/t/test.rs" }] }
    ]});
    assert_eq!(
        member_root_files(&metadata),
        [
            PathBuf::from("/ws/a/custom_root.rs"),
            PathBuf::from("/ws/z/src/lib.rs"),
            PathBuf::from("/ws/z/src/main.rs")
        ]
    );
}

/// A unique, self-cleaning temp directory for a path-identity fixture: replaces the hand-rolled
/// `temp_dir().join(format!(...))` + manual `remove_dir_all` at both ends the two tests below
/// otherwise each repeat.
struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let dir = std::env::temp_dir().join(format!("xingbiao-{label}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn write(&self, name: &str, contents: &str) -> PathBuf {
        let path = self.0.join(name);
        std::fs::write(&path, contents).unwrap();
        path
    }

    fn path(&self, name: &str) -> PathBuf {
        self.0.join(name)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn canonicalize_or_fail_resolves_a_real_file_and_errors_on_a_missing_one() {
    let dir = TempDir::new("canonicalize");
    let file = dir.write("real.rs", "");

    assert!(canonicalize_or_fail(&file).is_ok());

    let missing = dir.path("does_not_exist.rs");
    let err = canonicalize_or_fail(&missing).unwrap_err();
    assert!(
        err.contains("cannot resolve"),
        "a missing path must fail loud, not silently skip: {err}"
    );
}

#[test]
fn try_visit_reports_first_visit_then_repeat_and_fails_loud_on_an_unresolvable_path() {
    let dir = TempDir::new("try-visit");
    let file = dir.write("a.rs", "");

    let mut visited = std::collections::HashSet::new();
    assert_eq!(
        try_visit(&mut visited, &file),
        Ok(true),
        "the first visit to a real file is new"
    );
    assert_eq!(
        try_visit(&mut visited, &file),
        Ok(false),
        "a repeat visit to the same canonical file is not new"
    );
    assert!(try_visit(&mut visited, &dir.path("missing.rs")).is_err());
}
