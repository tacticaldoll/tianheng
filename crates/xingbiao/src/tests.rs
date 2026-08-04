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

/// Every shape that actually occurs labels byte-identically to the rendering `path_label` replaced, so
/// no recorded baseline entry re-keys on unix.
///
/// This is the assertion that made the change safe to make at all: 漏刻's observed-file label and
/// 圭表/渾儀's compilation unit are both baseline identity, so a label that shifted for an ordinary path
/// would restate every entry an adopter has recorded. The previous rendering was
/// `encoded(path.as_os_str())` — the bytes escaped, separators untouched — and for a path with one
/// separator style and no `.`/`//` noise the two agree exactly.
#[test]
fn path_label_does_not_re_key_any_shape_that_occurs() {
    for path in [
        "src/lib.rs",
        "src/main.rs",
        "src/bin/conventional.rs",
        "tools/outside.rs",
        "single.rs",
        "/abs/src/lib.rs",
        "a/../b",
    ] {
        assert_eq!(
            path_label(Path::new(path)),
            path,
            "an ordinary path must label as itself, or every recorded baseline entry re-keys"
        );
    }
    assert_eq!(
        path_label(Path::new("with%pct/f.rs")),
        "with%25pct/f.rs",
        "a literal `%` escapes so no escaped label can be spelled by an unescaped one"
    );
    // The stated normalizations. Neither form is reachable from a `cargo metadata` `src_path` or a
    // walked path, both already canonical; asserted so the bound is pinned rather than implied.
    assert_eq!(path_label(Path::new("./a")), "a");
    assert_eq!(path_label(Path::new("a//b")), "a/b");
    assert_eq!(path_label(Path::new("/")), "/");
}

/// A byte that is not valid UTF-8 survives as an escape, so two paths differing only there keep two
/// labels — and a separator byte that is legal *inside* a name is not treated as a separator.
///
/// The second half is what proves the separator rule is delegated to `std::path` rather than
/// hardcoded: on unix `\` is an ordinary byte, so `a\b` is ONE component and must not label as `a/b`.
/// A `replace('\\', "/")` implementation would map two distinct paths onto one label — the exact
/// injectivity loss this labeling exists to prevent — and this is the assertion that would catch it.
#[cfg(unix)]
#[test]
fn path_label_preserves_bytes_and_does_not_invent_separators() {
    use std::os::unix::ffi::OsStrExt;

    let bad = std::ffi::OsStr::from_bytes(b"src/ba\xffd.rs");
    assert_eq!(
        path_label(Path::new(bad)),
        "src/ba%FFd.rs",
        "an undecodable byte escapes rather than collapsing to U+FFFD"
    );
    let other = std::ffi::OsStr::from_bytes(b"src/ba\xfed.rs");
    assert_ne!(
        path_label(Path::new(bad)),
        path_label(Path::new(other)),
        "two paths differing only in undecodable bytes must keep two identities"
    );

    let backslash_in_name = std::ffi::OsStr::from_bytes(b"a\\b");
    assert_eq!(
        path_label(Path::new(backslash_in_name)),
        "a\\b",
        "on unix `\\` is a byte within one component, not a separator: labeling it as `a/b` would \
         collide with the genuinely two-component path"
    );
    assert_ne!(
        path_label(Path::new(backslash_in_name)),
        path_label(Path::new("a/b")),
        "the single file `a\\b` and the file `b` inside directory `a` are distinct observations"
    );
}

/// The case this change exists for, asserted where it can actually run.
///
/// Not executed in this repository — there is no Windows runner and no wine — so it is here as the
/// statement of the closed defect rather than as evidence for it. `Components` splits on
/// `sys::path::is_sep_byte`, which `library/std/src/sys/path/windows.rs` defines as
/// `path_separator_bytes!(b'\\', b'/')` and `.../unix.rs` as `path_separator_bytes!(b'/')` — so on
/// Windows `src\lib.rs` is two components and joins back as `src/lib.rs`, matching what Linux CI
/// recorded, while on unix it is one component and stays one (asserted above, where it does run).
#[cfg(windows)]
#[test]
fn a_windows_separator_labels_as_the_canonical_one() {
    assert_eq!(path_label(Path::new("src\\lib.rs")), "src/lib.rs");
    assert_eq!(path_label(Path::new("src\\bin\\x.rs")), "src/bin/x.rs");
    assert_eq!(
        path_label(Path::new("src/lib.rs")),
        path_label(Path::new("src\\lib.rs")),
        "both separators are separators on Windows, so both must reach one label"
    );
}

/// No label carries the platform's own separator unless that separator is already `/`.
///
/// **This assertion is vacuous on unix and load-bearing on Windows**, and is written this way
/// deliberately rather than left to a platform CI runner that does not exist. On unix
/// `MAIN_SEPARATOR` IS `/`, so the check cannot fail here; on Windows it is `\`, and the defect this
/// change closes — `src\lib.rs` reaching a baseline as identity — is exactly what it catches. The
/// Windows behaviour rests on `std::path::Components`' documented separator parsing, not on a
/// measurement taken in this repository: there is no Windows runner and no wine here, and claiming
/// otherwise would be the kind of unearned confidence a green unix suite invites.
#[test]
fn a_label_never_carries_a_platform_separator() {
    for path in ["src/lib.rs", "a/b/c.rs", "/abs/x.rs"] {
        let label = path_label(Path::new(path));
        assert!(
            std::path::MAIN_SEPARATOR == '/' || !label.contains(std::path::MAIN_SEPARATOR),
            "a label must use `/` alone, so one commit yields one identity on every platform: {label:?}"
        );
    }
}

/// A root reported under two target names collapses to one, wherever the two reports sit.
///
/// `Vec::dedup` removes only CONSECUTIVE equal elements, which is total for
/// [`member_root_files`] because it sorts first — and was not total here, where Cargo's own order is
/// preserved on purpose. Two targets may legitimately name the same `path`; Cargo accepts it, builds
/// both, and reports targets sorted by NAME, so the duplicate reports are adjacent only when no third
/// target's name sorts between them. The `[x, y, x]` arrangement below is that case, and it is what
/// `dedup` alone left untouched — measured against a real three-`[[bin]]` manifest, where
/// `crate_root_files` returned `[shared.rs, between.rs, shared.rs]`.
///
/// Adjacency is asserted alongside it so the test states the whole rule rather than one arrangement of
/// it: `[x, x, y]` collapsed under `dedup` too, so a fixture using only that shape would pass against
/// the defect.
#[test]
fn crate_root_files_is_unique_by_root_not_by_adjacency() {
    let non_adjacent = json!({ "targets": [
        { "kind": ["bin"], "src_path": "/p/src/shared.rs" },
        { "kind": ["bin"], "src_path": "/p/src/between.rs" },
        { "kind": ["bin"], "src_path": "/p/src/shared.rs" }
    ]});
    assert_eq!(
        crate_root_files(&non_adjacent),
        [
            PathBuf::from("/p/src/shared.rs"),
            PathBuf::from("/p/src/between.rs")
        ],
        "a root reported twice is ONE compilation unit, and Cargo's reported order is preserved"
    );

    let adjacent = json!({ "targets": [
        { "kind": ["lib"], "src_path": "/p/src/shared.rs" },
        { "kind": ["bin"], "src_path": "/p/src/shared.rs" },
        { "kind": ["bin"], "src_path": "/p/src/other.rs" }
    ]});
    assert_eq!(
        crate_root_files(&adjacent),
        [
            PathBuf::from("/p/src/shared.rs"),
            PathBuf::from("/p/src/other.rs")
        ],
        "the adjacent arrangement collapses too — this half held before the fix, and is here so the \
         test cannot be mistaken for pinning only it"
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
