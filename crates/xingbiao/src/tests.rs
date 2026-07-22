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
