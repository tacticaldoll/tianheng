use super::*;
use crate::module_scan::rust_files;
use crate::module_scan::source_texts::SourceTexts;

/// The walk of one fixture, as an evaluation of its own reads it: through a text set no other walk shares.
#[allow(clippy::type_complexity)]
fn reachable_modules(
    src_dir: &Path,
    files: &[PathBuf],
    root_relative: Option<&Path>,
    edition: crate::module_scan::Edition,
) -> Result<
    (
        std::collections::BTreeSet<String>,
        std::collections::BTreeMap<String, PathBuf>,
        Vec<(PathBuf, String)>,
        std::collections::BTreeSet<String>,
    ),
    String,
> {
    walk::reachable_modules(
        &SourceTexts::default(),
        src_dir,
        files,
        root_relative,
        edition,
    )
}

/// A unique, self-cleaning source tree for module-reachability fixtures.
///
/// Reachability tests differ in the module graph they write, but not in the filesystem
/// plumbing needed to host it. Keep that plumbing here so every case cleans up on panic and a
/// new case only describes the source shape it is exercising.
struct TempSrcTree {
    dir: PathBuf,
    src: PathBuf,
}

impl TempSrcTree {
    fn new(label: &str) -> Self {
        let dir = xingbiao::scratch_base().join(format!(
            "guibiao-reachability-{label}-{}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
        Self { dir, src }
    }

    fn src(&self) -> &Path {
        &self.src
    }
}

impl Drop for TempSrcTree {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[test]
fn declared_modules_finds_only_top_level_declarations() {
    let source = r#"
        pub mod kernel;
        mod projection;
        pub(crate) mod runner;
        mod inline { mod nested_child; }   // nested_child is depth 1, not a root module
        // mod commented_out;
        fn f() { let _ = "mod string_literal;"; }
    "#;
    let mut mods = declared_modules(source);
    mods.sort();
    assert_eq!(
        mods,
        vec![
            "inline".to_string(),
            "kernel".to_string(),
            "projection".to_string(),
            "runner".to_string(),
        ],
        "only top-level mod declarations count; nested, commented, and quoted are excluded"
    );
}

#[test]
fn reachable_modules_follows_mod_declarations_not_filenames() {
    // The crate root declares `mod kernel;`, but two orphan files exist that no `mod`
    // brings into scope: a root orphan (`serde.rs`) and a subtree orphan
    // (`kernel/orphan.rs`, which `kernel.rs` never declares). Only `crate` and
    // `crate::kernel` are reachable; the orphans are not — at the root OR in a subtree.
    let tree = TempSrcTree::new("declared-not-filenames");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("kernel")).expect("create temp src/kernel");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod kernel;\nuse serde::Deserialize;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("kernel.rs"), "// kernel declares no submodule\n")
        .expect("write kernel.rs");
    std::fs::write(src.join("serde.rs"), "// root orphan, undeclared\n").expect("write serde.rs");
    std::fs::write(
        src.join("kernel/orphan.rs"),
        "use crate::projection::Thing;\n",
    )
    .expect("write kernel/orphan.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    assert!(reachable.contains("crate"), "{reachable:?}");
    assert!(
        reachable.contains("crate::kernel"),
        "a declared `mod kernel;` is reachable: {reachable:?}"
    );
    assert!(
        !reachable.contains("crate::serde"),
        "an undeclared root orphan is not reachable: {reachable:?}"
    );
    assert!(
        !reachable.contains("crate::kernel::orphan"),
        "an undeclared subtree orphan is not reachable: {reachable:?}"
    );
}

#[test]
fn a_stray_lib_beside_a_custom_root_is_not_a_second_crate_root() {
    // With a custom target root (`[lib] path = "src/core.rs"`), a
    // leftover top-level `lib.rs` is NOT the crate root — cargo never compiles it — so it must
    // not also claim the segment-less `crate` module. If both `core.rs` and `lib.rs` mapped to
    // `crate`, the stray file's `mod ghost;` would union into the real root and make
    // `crate::ghost` phantom-reachable (a spurious module-boundary violation on an uncompiled file).
    let tree = TempSrcTree::new("custom-root");
    let src = tree.src().to_path_buf();
    std::fs::write(src.join("core.rs"), "pub mod real;\n").expect("write core.rs");
    std::fs::write(
        src.join("real.rs"),
        "// real, declared from the true root\n",
    )
    .expect("write real.rs");
    std::fs::write(src.join("lib.rs"), "pub mod ghost;\n").expect("write stray lib.rs");
    std::fs::write(
        src.join("ghost.rs"),
        "// declared only by the uncompiled lib.rs\n",
    )
    .expect("write ghost.rs");

    let files = rust_files(&src).expect("list files");
    let root_relative = std::path::PathBuf::from("core.rs");
    let (reachable, _inline_only, _remapped, _remap_shadowed) = reachable_modules(
        &src,
        &files,
        Some(&root_relative),
        crate::module_scan::Edition::Rust2018,
    )
    .expect("walk modules");
    assert!(
        reachable.contains("crate"),
        "the custom root seeds crate: {reachable:?}"
    );
    assert!(
        reachable.contains("crate::real"),
        "a module declared from the true root is reachable: {reachable:?}"
    );
    assert!(
        !reachable.contains("crate::ghost"),
        "a module declared only by the stray, uncompiled lib.rs is NOT reachable: {reachable:?}"
    );
}

#[test]
fn path_remapped_modules_are_followed_to_their_target() {
    // rustc ground truth: `#[path = "weird.rs"] pub mod kernel;` compiles `weird.rs` as
    // `crate::kernel` — verified with a real `cargo build`. The conventional orphan
    // `kernel.rs` (which the remap's presence puts out of scope, module-source hardening
    // v0.1.4) must stay excluded even though `crate::kernel` is now reachable.
    let tree = TempSrcTree::new("path-remap");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"weird.rs\"]\npub mod kernel;\npub mod normal;\n",
    )
    .expect("write lib.rs");
    let target = src.join("weird.rs");
    std::fs::write(&target, "use crate::projection::Thing;\n").expect("write remapped file");
    let orphan = src.join("kernel.rs");
    std::fs::write(&orphan, "use crate::wrong_file_if_observed::Thing;\n")
        .expect("write conventional orphan");
    std::fs::write(src.join("normal.rs"), "// normal module\n").expect("write normal.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );

    assert!(reachable.contains("crate::normal"), "{reachable:?}");
    assert!(
        reachable.contains("crate::kernel"),
        "a #[path]-remapped module is now followed to its target: {reachable:?}"
    );
    assert_eq!(
        remapped,
        vec![(target.clone(), "crate::kernel".to_string())],
        "the remap is recorded under its logical path: {remapped:?}"
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &target && module == "crate::kernel"),
        "the real remapped target is governed under crate::kernel: {governed:?}"
    );
    assert!(
        !governed.iter().any(|(file, _)| file == &orphan),
        "the conventional orphan must not be governed in the remap's place: {governed:?}"
    );
}

#[test]
fn a_plain_child_of_a_path_remapped_module_is_governed_from_the_remaps_own_directory() {
    // rustc ground truth (verified with a real `rustc` build): a `#[path]`-loaded file is
    // itself mod-rs-like, so a plain `mod child;` written inside it compiles relative to the
    // REMAP TARGET's own directory, not to `by_module`'s structural index (which is keyed by
    // each file's own on-disk path and has no entry under the logical `crate::kernel::child`
    // when the backing file physically lives at `other/child.rs`). Before this fix, the child
    // was reachable (inserted unconditionally) but never a member of `sources`, so it was
    // never scanned and never governed — a real `use` passed every boundary unobserved.
    let tree = TempSrcTree::new("remap-plain-child");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("other")).expect("create temp src/other");
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write remap target");
    let child_file = src.join("other/child.rs");
    std::fs::write(&child_file, "use crate::projection::Thing;\n").expect("write child.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );

    assert!(
        reachable.contains("crate::kernel::child"),
        "the remap target's own plain child is reachable: {reachable:?}"
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &child_file && module == "crate::kernel::child"),
        "the remap target's own plain child is governed under its logical path, so its real \
         `use` is observed: {governed:?}"
    );
}

#[test]
fn path_attribute_detection_is_specific() {
    assert_eq!(
        declared_modules("#[pathology]\npub mod kernel;\n"),
        vec!["kernel".to_string()],
        "only the real `path` attribute is a remap marker"
    );
    // Rust permits whitespace in an outer attribute head; the direct remap is still
    // recognized (and, in `reachable_modules`, followed) — it is not dropped from
    // `declared_modules` (unlike a cfg_attr-wrapped one, still tested as empty below).
    assert_eq!(
        declared_modules("# [ path = \"weird.rs\" ]\npub mod kernel;\n"),
        vec!["kernel".to_string()],
    );
}

#[test]
fn a_cfg_attr_nested_path_collects_conditional_remaps() {
    // `#[cfg_attr(<pred>, path = "…")]` (== `#[cfg(<pred>)] #[path = "…"]`) is a
    // conditional remap — under union-scan semantics, the declared module is captured with its
    // candidate conditional targets so all physically existing files undergo governance.
    let mods = declared_modules("#[cfg_attr(windows, path = \"os/windows.rs\")]\npub mod os;\n");
    assert_eq!(mods, vec!["os".to_string()]);
    // The remap may sit after the predicate among several applied attrs, and whitespace varies.
    let mods_a = declared_modules("#[cfg_attr(all(unix), deprecated, path = \"p.rs\")]\nmod a;\n");
    assert_eq!(mods_a, vec!["a".to_string()]);
    // A NESTED `cfg_attr` remap (== `#[cfg(all(a,b))] #[path]`) is detected too.
    let mods_m =
        declared_modules("#[cfg_attr(a, cfg_attr(b, path = \"secret.rs\"))]\npub mod m;\n");
    assert_eq!(mods_m, vec!["m".to_string()]);
}

#[test]
fn a_deeply_nested_cfg_attr_path_is_followed_without_native_recursion() {
    const DEPTH: usize = 512;
    let tree = TempSrcTree::new("deeply-nested-cfg-attr");
    let src = tree.src().to_path_buf();
    let mut attribute = String::from("#[cfg_attr");
    for _ in 0..DEPTH {
        attribute.push_str("(predicate, cfg_attr");
    }
    attribute.push_str("(predicate, path = \"target.rs\")");
    for _ in 0..=DEPTH {
        attribute.push(')');
    }
    attribute.push_str("]\npub mod target;\n");
    std::fs::write(src.join("lib.rs"), attribute).expect("write lib.rs");
    let target = src.join("target.rs");
    std::fs::write(&target, "use crate::projection::Thing;\n").expect("write target.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk deeply nested cfg_attr");

    assert!(reachable.contains("crate::target"), "{reachable:?}");
    assert!(
        remapped
            .iter()
            .any(|(file, module)| file == &target && module == "crate::target"),
        "{remapped:?}"
    );
}

#[test]
fn mixed_direct_and_conditional_path_attrs_keep_the_module_regardless_of_order() {
    assert_eq!(
        declared_modules(
            "#[cfg_attr(some_platform, path = \"b.rs\")]\n#[path = \"a.rs\"]\npub mod x;\n"
        ),
        vec!["x".to_string()],
        "cfg_attr before the direct #[path] must not drop the module",
    );
    assert_eq!(
        declared_modules(
            "#[path = \"a.rs\"]\n#[cfg_attr(some_platform, path = \"b.rs\")]\npub mod x;\n"
        ),
        vec!["x".to_string()],
        "the direct #[path] first must keep working as before",
    );
}

#[test]
fn stacked_cfg_attr_path_only_targets_are_governed_without_a_plain_file() {
    // The 0.4.0 audit trigger, reconstructed at the reachability-walk level: a single `pub mod
    // imp;` decorated with TWO STACKED `#[cfg_attr(.., path = ..)]` attributes, one per platform,
    // jointly exhaustive (`unix` / `not(unix)`) — every real rustc build compiles cleanly
    // through exactly one target, and neither a plain `imp.rs` nor `imp/mod.rs` exists (nor is
    // ever needed). Before this fix `resolve_plain_sources` required one of those conventional
    // files regardless of the resolved `cfg_attr(path)` candidates, hard-erroring on source that
    // compiles cleanly on every platform.
    let tree = TempSrcTree::new("stacked-cfg-attr-path-only");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg_attr(unix, path = \"unix_imp.rs\")]\n#[cfg_attr(not(unix), path = \"other_imp.rs\")]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    let unix_target = src.join("unix_imp.rs");
    let other_target = src.join("other_imp.rs");
    std::fs::write(&unix_target, "use crate::from_unix::Thing;\n").expect("write unix_imp.rs");
    std::fs::write(&other_target, "use crate::from_other::Thing;\n").expect("write other_imp.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("stacked cfg_attr(path)-only must not hard error");

    assert!(reachable.contains("crate::imp"), "{reachable:?}");
    assert!(
        remapped
            .iter()
            .any(|(file, module)| file == &unix_target && module == "crate::imp"),
        "{remapped:?}"
    );
    assert!(
        remapped
            .iter()
            .any(|(file, module)| file == &other_target && module == "crate::imp"),
        "{remapped:?}"
    );
}

#[test]
fn single_cfg_attr_path_only_target_is_governed_without_a_plain_file() {
    // Control: the fix is not specific to "stacked" — a SINGLE cfg_attr(path) target with no
    // plain fallback must be tolerated identically.
    let tree = TempSrcTree::new("single-cfg-attr-path-only");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg_attr(unix, path = \"unix_imp.rs\")]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    let target = src.join("unix_imp.rs");
    std::fs::write(&target, "use crate::from_unix::Thing;\n").expect("write unix_imp.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("single cfg_attr(path)-only must not hard error");

    assert!(reachable.contains("crate::imp"), "{reachable:?}");
    assert!(
        remapped
            .iter()
            .any(|(file, module)| file == &target && module == "crate::imp"),
        "{remapped:?}"
    );
}

#[test]
fn a_cfg_attr_path_target_absent_with_no_plain_file_is_still_a_scan_error() {
    // The fix must not widen tolerance beyond a RESOLVED candidate: when the cfg_attr(path)
    // target itself does not exist on disk, and no plain conventional file exists either, and
    // no bare `#[cfg]`/`cfg_if!` arm applies, every candidate is absent — the module is genuinely
    // unbacked on every configuration, matching hunyi's own `!has_backing_source &&
    // !cfg_conditional` boundary for the identical shape (crates/hunyi/src/scan/items.rs).
    let tree = TempSrcTree::new("cfg-attr-path-absent-no-plain");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg_attr(windows, path = \"windows_only.rs\")]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    // Deliberately do not create `windows_only.rs`, `imp.rs`, or `imp/mod.rs`.

    let files = rust_files(&src).expect("list files");
    let result = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
    let err = result.expect_err(
        "a cfg_attr(path) target absent with no plain conventional file is still a scan error",
    );
    assert!(
        err.contains("crate::imp") && err.contains("could not be located"),
        "expected the missing-plain-file constitution error, got: {err}"
    );
}

#[test]
fn both_conventional_forms_present_stays_an_ambiguity_alongside_a_resolved_cfg_attr_target() {
    // The fix must not widen tolerance to override the pre-existing, stricter ambiguity check:
    // both conventional forms present is unresolvable under every predicate value, so it stays a
    // constitution error even when a `cfg_attr(path)` candidate on the same declaration also
    // resolves to a real file.
    let tree = TempSrcTree::new("dual-form-with-cfg-attr");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg_attr(unix, path = \"unix_imp.rs\")]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("unix_imp.rs"), "use crate::from_unix::Thing;\n")
        .expect("write unix_imp.rs");
    std::fs::write(src.join("imp.rs"), "// conventional flat form\n").expect("write imp.rs");
    std::fs::create_dir_all(src.join("imp")).expect("mkdir imp");
    std::fs::write(src.join("imp/mod.rs"), "// conventional nested form\n")
        .expect("write imp/mod.rs");

    let files = rust_files(&src).expect("list files");
    let result = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
    let err = result.expect_err("both conventional forms present must still be an ambiguity error");
    assert!(
        err.contains("resolves to both"),
        "expected the dual-backed ambiguity constitution error, got: {err}"
    );
}

#[test]
fn a_cfg_attr_without_a_path_meta_is_not_a_remap() {
    // The inverse false negative: a `cfg_attr` that carries NO `path` meta must not be mistaken
    // for a remap, or a normal file module would be dropped from scope and never governed.
    assert_eq!(
        declared_modules("#[cfg_attr(test, derive(Debug))]\npub mod real;\n"),
        vec!["real".to_string()],
        "a cfg_attr without a path meta is not a remap",
    );
    // A `path` substring inside a predicate's STRING value is not a `path` meta.
    assert_eq!(
        declared_modules("#[cfg_attr(feature = \"path\", deprecated)]\npub mod real;\n"),
        vec!["real".to_string()],
        "a `path` inside a predicate string is not a path meta",
    );
    // A same-suffixed identifier (`target_path`) is not the `path` meta.
    assert_eq!(
        declared_modules("#[cfg_attr(unix, target_path = \"x\")]\npub mod real;\n"),
        vec!["real".to_string()],
    );
    // A NESTED cfg_attr that carries no `path` meta must not be mistaken for a remap either.
    assert_eq!(
        declared_modules("#[cfg_attr(a, cfg_attr(b, deprecated))]\npub mod real;\n"),
        vec!["real".to_string()],
        "a nested cfg_attr without a path meta is not a remap",
    );
    // `path` in the PREDICATE position (first meta) is a cfg key, not an applied `path` attr —
    // must not be mistaken for a remap (would drop a normal module = inverse false negative).
    // Mirrors hunyi's `skip(1)`, keeping the two dimensions in agreement.
    assert_eq!(
        declared_modules("#[cfg_attr(path = \"x\", deprecated)]\npub mod real;\n"),
        vec!["real".to_string()],
        "a `path` cfg predicate key is not an applied path remap",
    );
}

#[test]
fn a_cfg_attr_nested_path_on_an_inline_module_does_not_drop_it() {
    // As with a direct #[path], a cfg_attr(path) on an INLINE module is a rustc no-op, so the
    // module stays declared.
    assert_eq!(
        declared_modules("#[cfg_attr(windows, path = \"x.rs\")]\npub mod a { pub mod inner; }\n"),
        vec!["a".to_string()],
    );
}

#[test]
fn a_path_attr_on_an_inline_module_does_not_drop_it() {
    // `#[path]` remaps only a FILE `mod name;`; on an INLINE `mod name { … }` it is a no-op
    // for rustc (the body IS the module), so the module must stay declared — dropping it
    // would leave a compiled module unobserved.
    assert_eq!(
        declared_modules("#[path = \"x.rs\"]\npub mod a { pub mod inner; }\n"),
        vec!["a".to_string()],
        "an inline module with a (no-op) #[path] is still declared",
    );
    // Control: on a FILE mod, #[path] is now a followed remap (0.2.2) — still declared (by
    // `declared_modules`, which does not distinguish a remap from an ordinary declaration),
    // unlike the cfg_attr-wrapped case, which stays excluded (tested elsewhere).
    assert_eq!(
        declared_modules("#[path = \"x.rs\"]\npub mod a;\n"),
        vec!["a".to_string()],
        "a #[path]-remapped FILE module is declared, to be followed to its target",
    );
}

#[test]
fn a_block_comment_before_a_mod_name_does_not_fuse_it() {
    // `mod/*c*/foo;` declares `foo`: a block comment is no token, so `mod` and
    // `foo` stay two tokens.
    assert_eq!(
        declared_modules("mod/*c*/foo;"),
        vec!["foo".to_string()],
        "a block comment after `mod` must not swallow the declaration",
    );
}

#[test]
fn a_custom_crate_root_filename_maps_to_crate() {
    // A crate whose target root is a custom filename
    // (`[lib] path = "src/core.rs"`) must still have its submodules reachable. The root file's
    // relative path is passed as root_relative so it maps to `crate` (not `crate::core`).
    let tree = TempSrcTree::new("custom-root-maps-to-crate");
    let src = tree.src().to_path_buf();
    std::fs::write(src.join("core.rs"), "pub mod sub;\n").expect("write core.rs");
    std::fs::write(src.join("sub.rs"), "// sub\n").expect("write sub.rs");
    let files = rust_files(&src).expect("list files");
    let (with_root, _, _, _) = reachable_modules(
        &src,
        &files,
        Some(std::path::Path::new("core.rs")),
        crate::module_scan::Edition::Rust2018,
    )
    .expect("walk");
    let (without_root, _, _, _) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018).expect("walk");
    assert!(
        with_root.contains("crate::sub"),
        "with the custom root mapped to crate, its submodule is reachable: {with_root:?}"
    );
    assert!(
        !without_root.contains("crate::sub"),
        "without the root override, core.rs maps to crate::core and sub is unreachable: {without_root:?}"
    );
}

#[test]
fn declared_modules_ignores_a_mod_inside_a_macro_invocation() {
    // A `mod` written inside a macro body is macro-generated and out of scope — the
    // same rule the `use` scanner already applies. `()`/`[]`-delimited invocations
    // were the gap (a `macro_rules!` body is already excluded by brace depth).
    assert!(declared_modules("some_macro!( mod ghost; );").is_empty());
    assert!(declared_modules("some_macro![ mod ghost; ];").is_empty());
    assert!(declared_modules("macro_rules! m { () => { mod ghost; }; }").is_empty());
    // A real top-level declaration is still found.
    assert_eq!(declared_modules("mod real;"), vec!["real".to_string()]);
}

#[test]
fn declared_modules_observes_mod_inside_cfg_if_macro_body() {
    let src = r#"
cfg_if::cfg_if! {
if #[cfg(feature = "x")] {
    mod child;
    fn f() {
        mod local_inner {
            use crate::secret;
        }
    }
}
}
"#;
    assert_eq!(declared_modules(src), vec!["child".to_string()]);

    // Parenthesized macro delimiter form `cfg_if!(...)`
    let src_parens = r#"
cfg_if::cfg_if!(
if #[cfg(feature = "x")] {
    mod child_paren;
}
);
"#;
    assert_eq!(
        declared_modules(src_parens),
        vec!["child_paren".to_string()]
    );

    // Unary negation `! { ... }` is NOT a macro and MUST NOT treat its block as top-level module scope
    let src_unary = r#"
const FLAG: bool = ! {
mod local_child {}
false
};
"#;
    assert!(declared_modules(src_unary).is_empty());

    // `cfg_if!` invoked inside an item body (fn/const) MUST NOT promote its mod declarations to crate top-level
    let src_inside_fn = r#"
fn owner() {
cfg_if::cfg_if! {
    if #[cfg(feature = "x")] {
        mod item_local_child;
    }
}
}
"#;
    assert!(declared_modules(src_inside_fn).is_empty());
}

#[test]
fn declared_modules_ignores_mod_inside_const_or_static_block() {
    let src = r#"
const _: () = {
mod child {
    use crate::secret;
}
};
const X: () = if true {
()
} else {
mod child {
    use crate::secret;
}
};
static FOO: () = {
mod inner;
};
"#;
    assert!(declared_modules(src).is_empty());
}

#[test]
fn an_inline_modules_file_backed_child_is_reachable() {
    // rustc ground truth (rustc 1.96.0): `pub mod parent { pub mod child; }` in lib.rs
    // compiles `src/parent/child.rs` as `crate::parent::child` — verified with a real
    // `cargo build`. `parent` owns no file of its own (inline-only), so before this fix the
    // walk stopped at `crate::parent` without ever discovering `child`: the forbidden false
    // negative this test pins (an import in the real compiled file going unobserved).
    let tree = TempSrcTree::new("inline-file-child-reachable");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("parent")).expect("create temp src/parent");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod parent {\n    pub mod child;\n}\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        src.join("parent/child.rs"),
        "use crate::projection::Thing;\n",
    )
    .expect("write parent/child.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");

    assert!(
        inline_only.contains_key("crate::parent"),
        "parent has no file of its own: {inline_only:?}"
    );
    assert!(
        reachable.contains("crate::parent::child"),
        "the real compiled file-backed child of an inline module must be reachable: {reachable:?}"
    );
    assert!(
        !inline_only.contains_key("crate::parent::child"),
        "the child is file-backed, not inline-only: {inline_only:?}"
    );
}

#[test]
fn an_inline_modules_file_backed_child_is_governed() {
    // The end-to-end shape of the false negative: `governed_files` must actually select the
    // real compiled file for scanning, not just mark its module path reachable.
    let tree = TempSrcTree::new("inline-file-child-governed");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("parent")).expect("create temp src/parent");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod parent {\n    pub mod child;\n}\n",
    )
    .expect("write lib.rs");
    let child_file = src.join("parent/child.rs");
    std::fs::write(&child_file, "use crate::projection::Thing;\n").expect("write child.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &child_file && module == "crate::parent::child"),
        "the real compiled child file must be governed: {governed:?}"
    );
}

#[test]
fn a_chain_of_inline_modules_reaches_its_file_backed_leaf() {
    // rustc ground truth (rustc 1.96.0): from a FILE-backed module (`kernel.rs`), three more
    // levels of INLINE nesting (`parent`, `a`, `b`) still resolve a file-backed leaf `c` at
    // `src/kernel/parent/a/b/c.rs` — verified with a real `cargo build`. Each inline level's
    // own body must be re-scanned in turn, not just the first one.
    let tree = TempSrcTree::new("inline-chain");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("kernel/parent/a/b")).expect("mkdirs");
    std::fs::write(src.join("lib.rs"), "pub mod kernel;\n").expect("write lib.rs");
    std::fs::write(
        src.join("kernel.rs"),
        "pub mod parent {\n    pub mod a {\n        pub mod b {\n            pub mod c;\n        }\n    }\n}\n",
    )
    .expect("write kernel.rs");
    std::fs::write(
        src.join("kernel/parent/a/b/c.rs"),
        "use crate::projection::Thing;\n",
    )
    .expect("write the deep leaf file");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    assert!(
        reachable.contains("crate::kernel::parent::a::b::c"),
        "a file-backed leaf beneath a chain of inline modules must be reachable: {reachable:?}"
    );
}

#[test]
fn an_inline_modules_mod_rs_style_child_is_reachable() {
    // rustc ground truth: `mod name;` beneath an inline parent may also resolve via the
    // `<name>/mod.rs` directory form, not just `<name>.rs` — the same two conventional forms
    // available to any file module, verified here under an inline ancestor.
    let tree = TempSrcTree::new("inline-mod-rs-child");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("parent/child")).expect("mkdirs");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod parent {\n    pub mod child;\n}\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        src.join("parent/child/mod.rs"),
        "use crate::projection::Thing;\n",
    )
    .expect("write parent/child/mod.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    assert!(
        reachable.contains("crate::parent::child"),
        "a mod.rs-style child beneath an inline parent must be reachable: {reachable:?}"
    );
}

#[test]
fn an_inline_only_grandparents_conventional_orphan_stays_excluded() {
    // The existing inline-only orphan-shadow bound (BUILT v0.1.4) must still hold for an
    // inline module discovered through this fix's new path: a stray conventional file
    // matching the INLINE parent's own name (not the file-backed child) is still an orphan
    // Rust never compiles, so it must stay unreachable and ungoverned.
    let tree = TempSrcTree::new("inline-orphan");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("parent")).expect("mkdirs");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod parent {\n    pub mod child;\n}\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        src.join("parent/child.rs"),
        "use crate::projection::Thing;\n",
    )
    .expect("write the real compiled child");
    std::fs::write(
        src.join("parent.rs"),
        "use crate::wrong_file_if_observed::Thing;\n",
    )
    .expect("write the conventional orphan Rust never compiles");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    assert!(
        inline_only.contains_key("crate::parent"),
        "parent is declared inline-only: {inline_only:?}"
    );
    assert!(
        reachable.contains("crate::parent::child"),
        "the real compiled child stays reachable: {reachable:?}"
    );
}

#[test]
fn a_path_remapped_child_nested_in_an_inline_parent_is_followed() {
    // rustc ground truth (rustc 1.96.0): `mod parent { #[path = "weird.rs"] mod child; }` at
    // the crate root resolves `weird.rs` relative to `parent`'s own accumulated directory
    // (`src/parent/weird.rs`), never `src/weird.rs` — the same base-directory rule 渾儀/漏刻
    // already follow for an inline-nested `#[path]`. The conventional orphan
    // `parent/child.rs` must stay excluded from governance even though `crate::parent::child`
    // is now reachable (through `weird.rs`).
    let tree = TempSrcTree::new("inline-path-remap");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("parent")).expect("mkdirs");
    std::fs::write(
        src.join("lib.rs"),
        "pub mod parent {\n    #[path = \"weird.rs\"]\n    pub mod child;\n}\n",
    )
    .expect("write lib.rs");
    let target = src.join("parent/weird.rs");
    std::fs::write(&target, "use crate::projection::Thing;\n")
        .expect("write the real #[path] target");
    let orphan = src.join("parent/child.rs");
    std::fs::write(&orphan, "use crate::wrong_file_if_observed::Thing;\n")
        .expect("write the conventional orphan the remap must not fall back to");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        inline_only.contains_key("crate::parent"),
        "parent is declared inline-only: {inline_only:?}"
    );
    assert!(
        reachable.contains("crate::parent::child"),
        "a #[path]-remapped child nested in an inline parent is followed to its target, \
         resolved relative to parent's own accumulated directory: {reachable:?}"
    );
    assert_eq!(
        remapped,
        vec![(target.clone(), "crate::parent::child".to_string())],
        "resolved from src/parent/, not src/: {remapped:?}"
    );
    assert!(
        !governed.iter().any(|(file, _)| file == &orphan),
        "the conventional orphan must not be governed in the remap's place: {governed:?}"
    );
}

#[test]
fn a_path_remap_value_with_a_backslash_newline_continuation_is_followed() {
    // A backslash immediately followed by a newline is a valid Rust string-literal line
    // continuation: it and the following line's leading whitespace are stripped, joining the
    // two fragments — verified against a real `rustc` build (`"moved\` + newline + indentation
    // + `b.rs"` decodes to `"movedb.rs"`, and rustc follows it). `decode_str_escapes` must
    // decode this the same way `syn` (used by 渾儀) does, or this crate silently drops the
    // remapped module from `reachable` instead of following it — a coverage gap found on a
    // 0.2.1 window's cross-dimension sweep.
    let tree = TempSrcTree::new("path-remap-line-continuation");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"moved\\\n    b.rs\"]\npub mod kernel;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("movedb.rs"), "// the continuation-named target\n")
        .expect("write movedb.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate::kernel",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    // A weak `reachable.contains(..)` check alone would pass even if decoding silently
    // failed: the collected child state and `reachable` gain an entry for a declared name
    // regardless of
    // whether its `#[path]` value ever decodes, so the real proof is that the DECODED
    // TARGET FILE is what actually governs `crate::kernel` — never a same-named orphan, and
    // never simply absent from `governed`.
    assert_eq!(
        governed.len(),
        1,
        "crate::kernel must be governed by exactly the continuation-decoded target: {governed:?}"
    );
    assert!(
        governed[0].0.ends_with("movedb.rs"),
        "the governing file must be the continuation-decoded target, not a stale orphan: {governed:?}"
    );
}

#[test]
fn a_path_remap_to_a_missing_target_is_a_scan_error() {
    // An unconditional `#[path]` target is a rustc compile error when absent — a genuine
    // broken reference, never a silent skip (the same "cannot judge, not nothing to judge"
    // discipline as an unreadable governed file).
    let tree = TempSrcTree::new("path-remap-missing");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"absent.rs\"]\npub mod kernel;\n",
    )
    .expect("write lib.rs");

    let files = rust_files(&src).expect("list files");
    let result = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
    assert!(
        result.is_err(),
        "a #[path] target that does not exist is a scan error, not a silent skip: {result:?}"
    );
}

#[test]
fn a_path_remap_cycle_is_a_scan_error_not_a_hang() {
    // A `#[path]` may point `..` back to an already-open source file, creating a genuine
    // graph cycle rustc itself rejects (a recursion-limit error) rather than compiling —
    // the scanner must fail loud (exit 2) instead of looping/overflowing the stack. Ordinary
    // conventional/inline nesting cannot cycle (bounded by the finite file list), so this
    // guard is exercised only through a `#[path]` chain, mirroring 渾儀's ancestor-path guard.
    let tree = TempSrcTree::new("path-remap-cycle");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("a")).expect("mkdirs");
    // lib.rs declares `mod a { #[path = "../lib.rs"] mod b; }` — `b`'s target resolves from
    // `a`'s own accumulated directory (`src/a/`), so `../lib.rs` is `src/lib.rs` itself: the
    // crate root re-declares `mod a { ... }`, looping crate::a::b::a::b::… forever.
    std::fs::write(
        src.join("lib.rs"),
        "pub mod a {\n    #[path = \"../lib.rs\"]\n    pub mod b;\n}\n",
    )
    .expect("write lib.rs");

    let files = rust_files(&src).expect("list files");
    let result = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
    // Asserting on the specific message (not just `is_err()`) pins that this is genuinely the
    // ancestor-cycle guard firing, not an unrelated error (e.g. an OS path-length limit from
    // an unnormalized `..` accumulating across repeated hops) that would happen to also return
    // `Err` while leaving the actual guard unexercised.
    let err = result.expect_err(
        "a #[path] chain cycling back to an already-open file is a scan error, not a hang",
    );
    assert!(
        err.contains("cycles back"),
        "expected the ancestor-cycle guard's own message, got: {err}"
    );
}

#[test]
fn two_declarations_sharing_one_path_remap_target_is_not_a_cycle() {
    // rustc ground truth (rustc 1.96.0): `#[path="s.rs"] mod a; #[path="s.rs"] mod b;`
    // compiles — the SAME file twice, as two distinct modules — matching 渾儀's own
    // "two modules sharing one #[path] target is not a cycle" precedent. An ancestor-path (not
    // monotonic whole-tree) guard is required or this legitimate, compilable input would be
    // misreported as a cycle (a false positive).
    let tree = TempSrcTree::new("path-remap-shared");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"s.rs\"]\npub mod a;\n#[path = \"s.rs\"]\npub mod b;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("s.rs"), "// shared target\n").expect("write s.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("two modules sharing one #[path] target is not a cycle (rustc compiles it)");
    assert!(reachable.contains("crate::a"), "{reachable:?}");
    assert!(reachable.contains("crate::b"), "{reachable:?}");
    assert_eq!(remapped.len(), 2, "{remapped:?}");
}

#[test]
fn cfg_gated_sibling_path_declarations_are_followed_cfg_blind_both() {
    // rustc ground truth (verified with a real `cargo build` on a unix host): mutually
    // exclusive `#[cfg(unix)]` / `#[cfg(windows)]` gating two whole `mod imp;` declarations of
    // the SAME name, each with a DIFFERENT unconditional `#[path]` target, is the standard
    // per-platform shim pattern — valid, common Rust, not a name collision. The scanner does
    // not evaluate `#[cfg]`, so it must follow BOTH targets (cfg-blind union, matching 渾儀's
    // own same-named-file-form-child policy), not pick one arbitrarily: a single-target
    // design would silently drop the inactive platform's imports depending on scan/file order.
    let tree = TempSrcTree::new("cfg-dual-path");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(unix)]\n#[path = \"unix_impl.rs\"]\npub mod imp;\n#[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    let unix_target = src.join("unix_impl.rs");
    std::fs::write(&unix_target, "use crate::projection::Unix;\n").expect("write unix_impl.rs");
    let windows_target = src.join("windows_impl.rs");
    std::fs::write(&windows_target, "use crate::projection::Windows;\n")
        .expect("write windows_impl.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("both cfg-gated targets are followed");
    assert!(reachable.contains("crate::imp"), "{reachable:?}");
    let mut targets: Vec<&PathBuf> = remapped
        .iter()
        .filter(|(_, module)| module == "crate::imp")
        .map(|(file, _)| file)
        .collect();
    targets.sort();
    let mut expected = vec![&unix_target, &windows_target];
    expected.sort();
    assert_eq!(
        targets, expected,
        "both platform targets are followed under crate::imp, cfg-blind: {remapped:?}"
    );
}

#[test]
fn a_nested_path_crossing_into_a_cfg_siblings_own_target_is_not_a_cycle() {
    // rustc ground truth (verified with a real rustc build under EITHER single-feature
    // config): mutually-exclusive `#[cfg(feature = "a")]` / `#[cfg(feature = "b")]` gate two
    // `mod imp;` declarations with DIFFERENT unconditional `#[path]` targets (the standard
    // per-platform shim, already followed cfg-blind above) — variant_a.rs's OWN nested
    // `#[path]` legitimately points at variant_b.rs, the OTHER arm's target. The two targets
    // are never simultaneously open in any real single build, so this must compile (and be
    // observed) cleanly under either feature, never misreported as a cycle. Before the fix,
    // both targets' canons were unioned into ONE shared ancestor set for `crate::imp`, so
    // scanning variant_a.rs's own nested `#[path]` against that merged set wrongly matched
    // variant_b.rs's canon and returned a scan error for valid, compilable input.
    let tree = TempSrcTree::new("cfg-cross-arm-nested");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(feature = \"a\")]\n#[path = \"variant_a.rs\"]\npub mod imp;\n#[cfg(feature = \"b\")]\n#[path = \"variant_b.rs\"]\npub mod imp;\n",
    )
    .expect("write lib.rs");
    let variant_a = src.join("variant_a.rs");
    std::fs::write(
        &variant_a,
        "#[path = \"variant_b.rs\"]\nmod also_b;\nuse crate::projection::A;\n",
    )
    .expect("write variant_a.rs");
    let variant_b = src.join("variant_b.rs");
    std::fs::write(&variant_b, "use crate::projection::B;\n").expect("write variant_b.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018).expect(
            "a nested #[path] crossing into a mutually-exclusive cfg sibling's own target must \
         not be misreported as a cycle",
        );
    assert!(reachable.contains("crate::imp"), "{reachable:?}");
    assert!(
        reachable.contains("crate::imp::also_b"),
        "the nested #[path] inside variant_a.rs is followed and governed: {reachable:?}"
    );
    let also_b_targets: Vec<&PathBuf> = remapped
        .iter()
        .filter(|(_, module)| module == "crate::imp::also_b")
        .map(|(file, _)| file)
        .collect();
    assert_eq!(
        also_b_targets,
        vec![&variant_b],
        "crate::imp::also_b resolves to variant_b.rs: {remapped:?}"
    );
}

#[test]
fn a_nested_path_inside_an_inline_cfg_siblings_plain_child_is_not_a_cycle() {
    // rustc ground truth (verified with a real rustc build under the "u" feature): mutually
    // exclusive `#[cfg(feature = "u")] pub mod x { pub mod y; }` (inline) and
    // `#[cfg(feature = "w")] #[path = "windows_x.rs"] pub mod x;` (file-form, the standard
    // per-platform shim). `x`'s two cfg-sibling sources are an inline Body (ancestors =
    // {lib.rs}) and a #[path] File (ancestors = {lib.rs, windows_x.rs}) — but only the inline
    // source declares the plain child `y`. Before the fix, the plain-child branch unioned
    // ALL of `x`'s sources' ancestors regardless of which one actually declared `y`, so `y`'s
    // own ancestor set wrongly included `windows_x.rs`'s canon — and when `y.rs` legitimately
    // `#[path]`-references `windows_x.rs` (the OTHER, never-simultaneously-open cfg arm's own
    // target), the cycle guard misfired on valid, compilable input.
    let tree = TempSrcTree::new("cfg-inline-plain-child-cross-arm");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("x")).expect("create temp src/x");
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(feature = \"u\")]\npub mod x {\n    pub mod y;\n}\n#[cfg(feature = \"w\")]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
    )
    .expect("write lib.rs");
    std::fs::write(
        src.join("x/y.rs"),
        "#[path = \"../windows_x.rs\"]\nmod cross;\n",
    )
    .expect("write x/y.rs");
    std::fs::write(src.join("windows_x.rs"), "// the other cfg arm's target\n")
        .expect("write windows_x.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, _inline_only, remapped, _remap_shadowed) = reachable_modules(
        &src,
        &files,
        None,
        crate::module_scan::Edition::Rust2018,
    )
    .expect(
        "a plain child's own nested #[path] crossing into a cfg sibling's target must not be a cycle",
    );
    assert!(reachable.contains("crate::x::y"), "{reachable:?}");
    assert!(
        reachable.contains("crate::x::y::cross"),
        "the nested #[path] inside y.rs is followed: {reachable:?}"
    );
    assert!(
        remapped
            .iter()
            .any(|(_, module)| module == "crate::x::y::cross"),
        "{remapped:?}"
    );
}

#[test]
fn a_grandchild_of_a_probed_plain_child_is_governed() {
    // rustc ground truth (verified with a real rustc build): `#[path = "other/weird.rs"] pub
    // mod kernel;` where `other/weird.rs` declares `pub mod child;` (resolved to
    // `other/child.rs` via the live probe, fix 2) and `other/child.rs` itself declares a
    // further plain `pub mod grandchild;`. rustc compiles the grandchild at
    // `other/child/grandchild.rs` — the ordinary stem-subdirectory convention relative to
    // child.rs's own location, since child.rs (an ordinary flat file reached this way) is NOT
    // itself mod-rs-like. Before the fix, nothing resolved this: the probed child's own
    // `child_base` was never computed/carried forward, so its own plain children were
    // reachable (inserted unconditionally) but never governed — a real false negative.
    let tree = TempSrcTree::new("probed-child-grandchild");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("other/child")).expect("create temp dirs");
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write weird.rs");
    std::fs::write(src.join("other/child.rs"), "pub mod grandchild;\n")
        .expect("write other/child.rs");
    let grandchild_file = src.join("other/child/grandchild.rs");
    std::fs::write(&grandchild_file, "use crate::projection::Thing;\n")
        .expect("write grandchild.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        reachable.contains("crate::kernel::child::grandchild"),
        "{reachable:?}"
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &grandchild_file
                && module == "crate::kernel::child::grandchild"),
        "the probed child's own grandchild is governed under its logical path: {governed:?}"
    );
}

#[test]
fn a_stray_file_at_a_remapped_modules_naive_structural_path_is_not_phantom_governed() {
    // rustc ground truth (verified with a real rustc build, including deliberately invalid
    // syntax in the stray file to confirm rustc never reads it): `#[path = "other/weird.rs"]
    // pub mod kernel;` means rustc NEVER looks at `kernel.rs` or `kernel/` at all — `kernel`
    // is wholly remapped. A leftover, wholly undeclared file that happens to physically sit
    // at the naive structural location a plain `mod child;` inside `kernel` would occupy if
    // `kernel` were NOT remapped (`src/kernel/child.rs`) is a true orphan. Before the fix, a
    // structural `by_module` lookup for the probed child's logical path did not know its
    // parent was remapped, so it phantom-matched this stray file alongside the real,
    // probe-resolved one — a false positive (an uncompiled file wrongly governed).
    let tree = TempSrcTree::new("remap-stray-structural-sibling");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("other")).expect("create temp src/other");
    std::fs::create_dir_all(src.join("kernel")).expect("create temp src/kernel");
    std::fs::write(
        src.join("lib.rs"),
        "#[path = \"other/weird.rs\"]\npub mod kernel;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("other/weird.rs"), "pub mod child;\n").expect("write weird.rs");
    let real_child = src.join("other/child.rs");
    std::fs::write(&real_child, "// the real, rustc-compiled child\n").expect("write real child");
    // A stray file that coincidentally sits where a plain `mod child;` inside a
    // NON-remapped `kernel` would have looked — rustc never compiles this, since `kernel` is
    // wholly remapped to `other/weird.rs` and no `kernel.rs`/`kernel/mod.rs` exists.
    std::fs::write(
        src.join("kernel/child.rs"),
        "this is not even valid rust syntax {{{",
    )
    .expect("write stray file");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &real_child && module == "crate::kernel::child"),
        "the real probed child is governed: {governed:?}"
    );
    assert_eq!(
        governed
            .iter()
            .filter(|(_, module)| module == "crate::kernel::child")
            .count(),
        1,
        "the stray file at the naive structural location must NOT be phantom-governed alongside the real one: {governed:?}"
    );
}

#[test]
fn a_plain_file_sibling_of_a_path_remap_is_still_governed() {
    // rustc ground truth (verified with a real `cargo build`): `#[cfg(unix)] pub mod x;` +
    // `#[cfg(windows)] #[path = "windows_x.rs"] pub mod x;` compiles `x.rs` on unix — the
    // standard per-platform shim pairing a PLAIN file on one platform with a `#[path]`-
    // relocated one on another. A `#[path]` sibling must never suppress a same-named plain
    // file's own registration (the false negative this test pins): both are cfg-blind and
    // additive, never mutually exclusive, matching how multiple `#[path]` targets are already
    // unioned above.
    let tree = TempSrcTree::new("cfg-plain-path-sibling");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(unix)]\npub mod x;\n#[cfg(windows)]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
    )
    .expect("write lib.rs");
    let plain = src.join("x.rs");
    std::fs::write(&plain, "use crate::projection::Unix;\n").expect("write x.rs");
    let remapped_target = src.join("windows_x.rs");
    std::fs::write(&remapped_target, "use crate::projection::Windows;\n")
        .expect("write windows_x.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(reachable.contains("crate::x"), "{reachable:?}");
    assert!(
        governed.iter().any(|(f, _)| f == &plain),
        "the plain-file sibling must still be governed, not suppressed by the #[path] \
         sibling: {governed:?}"
    );
    assert!(
        governed.iter().any(|(f, _)| f == &remapped_target),
        "the #[path] sibling's real target must also be governed: {governed:?}"
    );
    assert!(
        !remap_shadowed.contains("crate::x"),
        "a plain-file sibling means x.rs is real, not an orphan-shadow: {remap_shadowed:?}"
    );
}

#[test]
fn an_inline_sibling_of_a_path_remap_is_still_governed() {
    // rustc ground truth (verified with a real `cargo build`): `#[cfg(unix)] pub mod x {
    // pub mod y; }` + `#[cfg(windows)] #[path = "windows_x.rs"] pub mod x;` compiles the
    // inline body (and its own file-backed child `y`) on unix. An inline sibling is not the
    // plain-file-vs-inline cfg-blind bound (that bound is specifically about a same-named
    // CONVENTIONAL file, which a `#[path]` remap is not) — it must be observed alongside the
    // `#[path]` target, additively, the same as the plain-file case above.
    let tree = TempSrcTree::new("cfg-inline-path-sibling");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("x")).expect("mkdir x");
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(unix)]\npub mod x {\n    pub mod y;\n}\n#[cfg(windows)]\n#[path = \"windows_x.rs\"]\npub mod x;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("x/y.rs"), "use crate::projection::Unix;\n").expect("write x/y.rs");
    let remapped_target = src.join("windows_x.rs");
    std::fs::write(&remapped_target, "use crate::projection::Windows;\n")
        .expect("write windows_x.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        reachable.contains("crate::x::y"),
        "the inline sibling's own file-backed child must still be reachable: {reachable:?}"
    );
    assert!(
        remapped
            .iter()
            .any(|(f, m)| f == &remapped_target && m == "crate::x"),
        "the #[path] sibling's real target must also be followed: {remapped:?}"
    );
    // `crate::x` is directly targetable despite carrying no file of its own besides the
    // remap target: `inline_only` marking it (there is no plain conventional file, so the
    // bound applies) does not suppress the remap's own governance — the remap is
    // unconditional in `governed_files`, never gated on `inline_only`.
    assert!(
        governed
            .iter()
            .any(|(f, m)| f == &remapped_target && m == "crate::x"),
        "crate::x is governed via its #[path] target regardless of the inline sibling: {governed:?}"
    );
}

#[test]
fn an_inline_sibling_of_a_plain_file_is_still_governed() {
    // rustc ground truth (verified with a real `cargo build`, both feature configurations):
    // `#[cfg(not(feature = "b"))] pub mod x;` + `#[cfg(feature = "b")] pub mod x { pub mod y;
    // }` compiles the PLAIN `x.rs` by default and the INLINE body (with its own file-backed
    // child `x/y.rs` as `crate::x::y`) under feature `b`. The pre-existing v0.1.4 bound
    // ("a path declared both inline and file-form is observed through its conventional file")
    // is about which file backs `crate::x` itself for orphan-shadow purposes — it must not
    // also mean the inline body's OWN declarations go unscanned: `crate::x::y` is real,
    // compiled source under its own `#[cfg]` arm, and dropping it was a genuine false
    // negative (the scanner does not evaluate `#[cfg]`, so it must observe every variant).
    let tree = TempSrcTree::new("cfg-plain-inline-sibling");
    let src = tree.src().to_path_buf();
    std::fs::create_dir_all(src.join("x")).expect("mkdirs");
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(not(feature = \"b\"))]\npub mod x;\n#[cfg(feature = \"b\")]\npub mod x {\n    pub mod y;\n}\n",
    )
    .expect("write lib.rs");
    let plain = src.join("x.rs");
    std::fs::write(&plain, "use crate::projection::Plain;\n").expect("write x.rs");
    let inline_child = src.join("x/y.rs");
    std::fs::write(&inline_child, "use crate::projection::InlineChild;\n").expect("write x/y.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, _remapped, _remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    assert!(
        !inline_only.contains_key("crate::x"),
        "a plain file is declared, so crate::x is not inline-only: {inline_only:?}"
    );
    assert!(
        reachable.contains("crate::x::y"),
        "the inline sibling's own file-backed child must still be reachable even though a \
         plain-file sibling of crate::x also exists: {reachable:?}"
    );
}

#[test]
fn governed_files_does_not_duplicate_a_plain_files_own_path_remap_target() {
    // rustc ground truth (verified with a real `cargo build`, both feature configurations):
    // `#[cfg(not(feature = "b"))] pub mod a;` + `#[cfg(feature = "b")] #[path = "a.rs"] pub
    // mod a;` compiles the SAME `a.rs` under either arm — an unrelated `#[cfg]` arm's
    // `#[path]` can legitimately target the literal same file a plain-file sibling already
    // names. `governed_files`'s structural iterator (a real plain-file sibling, not shadowed)
    // and its `remap_entries` iterator (unconditional) then both name `(a.rs, crate::a)` —
    // pinning that the combined result carries it once, not twice.
    let tree = TempSrcTree::new("duplicate-remap-target");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg(not(feature = \"b\"))]\npub mod a;\n#[cfg(feature = \"b\")]\n#[path = \"a.rs\"]\npub mod a;\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("a.rs"), "use crate::projection::Thing;\n").expect("write a.rs");

    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    let a_entries: Vec<_> = governed
        .iter()
        .filter(|(_, module)| module == "crate::a")
        .collect();
    assert_eq!(
        a_entries.len(),
        1,
        "the plain sibling and its own #[path] target are the same file — governed once, \
         not twice: {governed:?}"
    );
}

/// An inner attribute belongs to the item it is written in, not to the `mod` after it: `#![cfg(unix)] mod c;` with no
/// `c.rs` is `error[E0583]` whatever the platform, so the declaration is no more tolerated in its absence than a
/// plain `mod c;` is; `#![path = "x.rs"] mod c;` beside an existing `x.rs` does not remap `c` to it; and
/// `mod tests { #![cfg(test)] mod c; }` does not make `c` conditional. Each is a missing-file error naming `c`.
#[test]
fn an_inner_attribute_is_no_attribute_of_the_mod_after_it() {
    let mut mismatches = Vec::new();
    for (lib, module, expected) in [
        ("#![cfg(unix)]\nmod c;\n", "crate::c", "src/c.rs"),
        ("#![path = \"x.rs\"]\nmod c;\n", "crate::c", "src/c.rs"),
        (
            "mod tests { #![cfg(test)] mod c; }\n",
            "crate::tests::c",
            "src/tests/c.rs",
        ),
    ] {
        let tree = TempSrcTree::new("inner-attribute");
        let src = tree.src().to_path_buf();
        std::fs::write(src.join("lib.rs"), lib).expect("write lib.rs");
        std::fs::write(src.join("x.rs"), "\n").expect("write x.rs");
        let files = rust_files(&src).expect("list files");
        let result = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
        let err = result.err().unwrap_or_default();
        let located = format!(
            "module '{module}' is declared (`mod c;`) but its source file could not be located"
        );
        if !(err.contains(&located) && err.contains(expected)) {
            mismatches.push(format!("{lib:?}: {err}"));
        }
    }
    assert!(
        mismatches.is_empty(),
        "a missing file for a `mod` after an inner attribute is a scan error naming it:\n{}",
        mismatches.join("\n")
    );
}

/// Which path attributes of a `mod` are read is decided by their position, as rustc decides it: rustc compiles the
/// first path attribute written and reports every later one as unused. So the first direct `#[path]` is the remap,
/// a `cfg_attr` path written before it is a candidate — the first attribute wherever its predicate holds — and
/// every path written after it, direct or `cfg_attr`, is never compiled and is not read. Each row was compiled by
/// rustc 1.96.0, edition 2021, on unix: the direct-first rows build with the later file absent and warn that the
/// later attribute is unused, and a `cfg_attr`-first row builds its `cfg_attr` target. A candidate that exists as a
/// directory without the child an inline body declares is the missing-file error, since the configuration that
/// selects it does not build.
#[test]
fn a_path_attribute_is_read_by_its_position() {
    const DIRECT_THEN_DIRECT: &str = "#[path = \"a.rs\"]\n#[path = \"b.rs\"]";
    const DIRECT_THEN_CFG: &str = "#[path = \"a.rs\"]\n#[cfg_attr(unix, path = \"b.rs\")]";
    const CFG_THEN_DIRECT: &str = "#[cfg_attr(unix, path = \"b.rs\")]\n#[path = \"a.rs\"]";
    const INLINE_DIRECT_THEN_CFG: &str = "#[path = \"d\"]\n#[cfg_attr(unix, path = \"c\")]";
    const INLINE_DIRECT_THEN_DIRECT: &str = "#[path = \"d\"]\n#[path = \"c\"]";
    const INLINE_CFG_THEN_DIRECT: &str = "#[cfg_attr(unix, path = \"c\")]\n#[path = \"d\"]";
    // (row, attributes, inline, whether the later file exists, the files governed as the module, or the file a
    // missing-file error names)
    type Row<'a> = (&'a str, &'a str, bool, bool, Result<&'a [&'a str], &'a str>);
    let rows: [Row; 11] = [
        ("dd-both", DIRECT_THEN_DIRECT, false, true, Ok(&["a.rs"])),
        (
            "dd-first-only",
            DIRECT_THEN_DIRECT,
            false,
            false,
            Ok(&["a.rs"]),
        ),
        ("dc-both", DIRECT_THEN_CFG, false, true, Ok(&["a.rs"])),
        (
            "dc-first-only",
            DIRECT_THEN_CFG,
            false,
            false,
            Ok(&["a.rs"]),
        ),
        (
            "cd-both",
            CFG_THEN_DIRECT,
            false,
            true,
            Ok(&["a.rs", "b.rs"]),
        ),
        (
            "cd-direct-only",
            CFG_THEN_DIRECT,
            false,
            false,
            Ok(&["a.rs"]),
        ),
        (
            "inline-dc-both",
            INLINE_DIRECT_THEN_CFG,
            true,
            true,
            Ok(&["d/k.rs"]),
        ),
        (
            "inline-dc-first-only",
            INLINE_DIRECT_THEN_CFG,
            true,
            false,
            Ok(&["d/k.rs"]),
        ),
        (
            "inline-dd-first-only",
            INLINE_DIRECT_THEN_DIRECT,
            true,
            false,
            Ok(&["d/k.rs"]),
        ),
        (
            "inline-cd-both",
            INLINE_CFG_THEN_DIRECT,
            true,
            true,
            Ok(&["c/k.rs", "d/k.rs"]),
        ),
        (
            "inline-cd-direct-only",
            INLINE_CFG_THEN_DIRECT,
            true,
            false,
            Err("c/k.rs"),
        ),
    ];
    let mut mismatches = Vec::new();
    for (row, attrs, inline, later, expected) in rows {
        let tree = TempSrcTree::new(row);
        let src = tree.src().to_path_buf();
        let (module, first, second) = if inline {
            std::fs::create_dir_all(src.join("c")).expect("create c");
            std::fs::create_dir_all(src.join("d")).expect("create d");
            ("crate::m::k", "d/k.rs", "c/k.rs")
        } else {
            ("crate::m", "a.rs", "b.rs")
        };
        let body = if inline {
            "mod m { pub mod k; }"
        } else {
            "mod m;"
        };
        std::fs::write(src.join("lib.rs"), format!("{attrs}\n{body}\n")).expect("write lib.rs");
        std::fs::write(src.join(first), "\n").expect("write the first path's file");
        if later {
            std::fs::write(src.join(second), "\n").expect("write the later path's file");
        }
        let files = rust_files(&src).expect("list files");
        let walked = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018);
        let answer: Result<Vec<String>, String> =
            walked.map(|(reachable, inline_only, remapped, remap_shadowed)| {
                let mut governed: Vec<String> = governed_files(
                    &src,
                    &files,
                    "crate",
                    &reachable,
                    &inline_only,
                    &remapped,
                    &remap_shadowed,
                    None,
                    ScanDepth::Subtree,
                )
                .into_iter()
                .filter(|(_, m)| m == module)
                .map(|(file, _)| {
                    file.strip_prefix(&src)
                        .expect("a governed file lies under src")
                        .to_string_lossy()
                        .replace('\\', "/")
                })
                .collect();
                governed.sort();
                governed
            });
        let matches = match (&answer, expected) {
            (Ok(governed), Ok(files)) => governed == files,
            (Err(err), Err(file)) => err.contains(file) && err.contains("could not be located"),
            _ => false,
        };
        if !matches {
            mismatches.push(format!("{row}: {answer:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// An inline module's direct `#[path]` written after a `cfg_attr` path applies only where that predicate is false, so
/// its base is descended as a candidate's is: only where it exists as a directory. `#[cfg_attr(unix, path = "c")]
/// #[path = "d"] mod m { pub mod k; }` with only `c/k.rs` on disk governs `c/k.rs`, where the direct base `d/` was
/// descended and its absent `d/k.rs` refused; with neither directory present the direct base is still descended, so
/// a declaration no configuration backs is the missing-file error. rustc 1.96.0, edition 2021, on unix, builds the
/// first.
#[test]
fn an_inline_modules_direct_path_after_a_candidate_is_descended_as_a_candidate() {
    for (row, candidate_exists, expected) in [
        (
            "inline-cd-candidate-only",
            true,
            Ok(vec!["c/k.rs".to_string()]),
        ),
        ("inline-cd-neither", false, Err("d/k.rs")),
    ] {
        let tree = TempSrcTree::new(row);
        let src = tree.src().to_path_buf();
        if candidate_exists {
            std::fs::create_dir_all(src.join("c")).expect("create c");
            std::fs::write(src.join("c/k.rs"), "\n").expect("write c/k.rs");
        }
        std::fs::write(
            src.join("lib.rs"),
            "#[cfg_attr(unix, path = \"c\")]\n#[path = \"d\"]\nmod m { pub mod k; }\n",
        )
        .expect("write lib.rs");
        let files = rust_files(&src).expect("list files");
        let walked = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2021);
        let answer: Result<Vec<String>, String> =
            walked.map(|(reachable, inline_only, remapped, remap_shadowed)| {
                let mut governed: Vec<String> = governed_files(
                    &src,
                    &files,
                    "crate",
                    &reachable,
                    &inline_only,
                    &remapped,
                    &remap_shadowed,
                    None,
                    ScanDepth::Subtree,
                )
                .into_iter()
                .filter(|(_, m)| m == "crate::m::k")
                .map(|(file, _)| {
                    file.strip_prefix(&src)
                        .expect("a governed file lies under src")
                        .to_string_lossy()
                        .replace('\\', "/")
                })
                .collect();
                governed.sort();
                governed
            });
        match (&answer, expected) {
            (Ok(governed), Ok(files)) => assert_eq!(governed, &files, "{row}"),
            (Err(err), Err(file)) => assert!(
                err.contains(file) && err.contains("could not be located"),
                "{row}: {err}"
            ),
            _ => panic!("{row}: {answer:?}"),
        }
    }
}

/// Two shapes whose declared file may be absent on a build rustc accepts, each measured against rustc 1.96.0, edition
/// 2021, on unix. A direct `#[path]` written after a `cfg_attr` path is the first path attribute only where that
/// predicate is false, so `#[cfg_attr(unix, path = "b.rs")] #[path = "a.rs"] mod m;` builds with only `b.rs`, which is
/// governed, and the absent `a.rs` is no scan error. A `cfg_attr` that applies a `cfg` can remove the item, so
/// `#[cfg_attr(all(), cfg(any()))] mod m;` builds with no `m.rs`, and the declaration is conditional.
#[test]
fn a_declaration_a_predicate_may_remove_or_redirect_tolerates_its_absent_file() {
    let mut mismatches = Vec::new();
    for (name, lib, file, governed) in [
        (
            "direct-after-candidate",
            "#[cfg_attr(unix, path = \"b.rs\")]\n#[path = \"a.rs\"]\nmod m;\n",
            Some("b.rs"),
            Some("b.rs"),
        ),
        (
            "cfg-inside-cfg-attr",
            "#[cfg_attr(all(), cfg(any()))]\nmod m;\n",
            None,
            None,
        ),
    ] {
        let tree = TempSrcTree::new(name);
        let src = tree.src().to_path_buf();
        std::fs::write(src.join("lib.rs"), lib).expect("write lib.rs");
        if let Some(file) = file {
            std::fs::write(src.join(file), "\n").expect("write the candidate");
        }
        let files = rust_files(&src).expect("list files");
        match reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018) {
            Ok((reachable, inline_only, remapped, remap_shadowed)) => {
                let found: Vec<String> = governed_files(
                    &src,
                    &files,
                    "crate",
                    &reachable,
                    &inline_only,
                    &remapped,
                    &remap_shadowed,
                    None,
                    ScanDepth::Subtree,
                )
                .into_iter()
                .filter(|(_, module)| module == "crate::m")
                .map(|(file, _)| file.file_name().unwrap().to_string_lossy().into_owned())
                .collect();
                if found.first().map(String::as_str) != governed {
                    mismatches.push(format!("{name}: governed {found:?}"));
                }
            }
            Err(err) => mismatches.push(format!("{name}: {err}")),
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Two bodies each declaring `#[path] mod m;` govern two files as two modules, `crate::{block}::m` and
/// `crate::{block 2}::m`, numbered in source order among the modules named `m` the file's blocks declare.
#[test]
fn block_declared_path_modules_are_numbered_apart() {
    let tree = TempSrcTree::new("block-path-mods");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "pub fn f() { #[path = \"x.rs\"] mod m; }\npub fn g() { #[path = \"y.rs\"] mod m; }\n",
    )
    .expect("write lib.rs");
    std::fs::write(src.join("x.rs"), "\n").expect("write x.rs");
    std::fs::write(src.join("y.rs"), "\n").expect("write y.rs");
    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let mut governed: Vec<(String, String)> = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    )
    .into_iter()
    .map(|(file, module)| {
        (
            file.file_name().unwrap().to_string_lossy().into_owned(),
            module,
        )
    })
    .collect();
    governed.sort();
    assert_eq!(
        governed,
        [
            ("lib.rs".to_string(), "crate".to_string()),
            ("x.rs".to_string(), "crate::{block}::m".to_string()),
            ("y.rs".to_string(), "crate::{block 2}::m".to_string()),
        ]
    );
}

/// A `#[path]` in a file reached through a symlink resolves from the directory of the path the file is opened by, as
/// rustc resolves it: with `src/a.rs -> ../elsewhere/a.rs` holding `#[path = "x.rs"] pub mod k;`, rustc 1.96.0 reads
/// `src/x.rs`, and refuses the crate when `x.rs` lies beside the symlink's target instead. So `src/x.rs` is governed
/// as `crate::a::k`.
#[cfg(unix)]
#[test]
fn a_path_in_a_symlinked_file_resolves_from_the_symlinks_directory() {
    let tree = TempSrcTree::new("symlink-path-base");
    let src = tree.src().to_path_buf();
    let elsewhere = src.parent().expect("src has a parent").join("elsewhere");
    std::fs::create_dir_all(&elsewhere).expect("create elsewhere");
    std::fs::write(src.join("lib.rs"), "pub mod a;\n").expect("write lib.rs");
    std::fs::write(elsewhere.join("a.rs"), "#[path = \"x.rs\"]\npub mod k;\n").expect("write a.rs");
    std::os::unix::fs::symlink("../elsewhere/a.rs", src.join("a.rs")).expect("create the symlink");
    std::fs::write(src.join("x.rs"), "\n").expect("write x.rs");
    let files = rust_files(&src).expect("list files");
    let (reachable, inline_only, remapped, remap_shadowed) =
        reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
            .expect("walk modules");
    let governed = governed_files(
        &src,
        &files,
        "crate",
        &reachable,
        &inline_only,
        &remapped,
        &remap_shadowed,
        None,
        ScanDepth::Subtree,
    );
    assert!(
        governed
            .iter()
            .any(|(file, module)| file == &src.join("x.rs") && module == "crate::a::k"),
        "{governed:?}"
    );
}

/// A direct `#[path]` after a `cfg_attr` candidate is tolerated absent only where a candidate's file exists: with
/// neither `a.rs` nor `b.rs`, `#[cfg_attr(unix, path = "b.rs")] #[path = "a.rs"] mod m;` builds on no configuration —
/// rustc 1.96.0 refuses it on unix for the missing `b.rs` — and is the scan error naming `a.rs`.
#[test]
fn a_direct_path_after_a_candidate_no_file_backs_is_a_scan_error() {
    let tree = TempSrcTree::new("direct-after-absent-candidate");
    let src = tree.src().to_path_buf();
    std::fs::write(
        src.join("lib.rs"),
        "#[cfg_attr(unix, path = \"b.rs\")]\n#[path = \"a.rs\"]\nmod m;\n",
    )
    .expect("write lib.rs");
    let files = rust_files(&src).expect("list files");
    let err = reachable_modules(&src, &files, None, crate::module_scan::Edition::Rust2018)
        .err()
        .unwrap_or_default();
    assert!(
        err.contains("a.rs") && err.contains("does not exist"),
        "{err}"
    );
}
