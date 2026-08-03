use super::super::*;
use super::async_exposure::async_subtree_labels;
use super::helpers::*;
use super::trait_impl::locality_findings;
use super::unsafe_confinement::unsafe_labels;
// --- semantic finding source file (the reaction-layer `file`) --------------

/// Write `files` under a unique temp `src`, resolve the governed `module`'s source file
/// (the file a single-module semantic violation reports), and return it. Cleans up; the
/// returned path is asserted by suffix, not existence.
pub(super) fn resolve_file(
    name: &str,
    files: &[(&str, &str)],
    module: &str,
) -> Result<PathBuf, String> {
    let tree = TempSrcTree::new(&format!("file-{name}"));
    tree.write_all(files);
    resolve_module_file(tree.src(), &tree.root(), module, "x")
}

#[test]
pub(super) fn module_file_is_the_crate_root_for_the_root_module() {
    let file = resolve_file("root", &[("lib.rs", "pub struct A;\n")], "crate").unwrap();
    assert!(file.ends_with("src/lib.rs"), "got {}", file.display());
}

#[test]
pub(super) fn module_file_is_the_file_module_source() {
    let file = resolve_file(
        "filemod",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain.rs", "pub struct A;\n"),
        ],
        "crate::domain",
    )
    .unwrap();
    assert!(file.ends_with("domain.rs"), "got {}", file.display());
}

/// A mutually-exclusive `#[cfg]` per-platform shim — an inline arm plus a file-form sibling
/// arm whose file is absent on this build — now resolves via the inline arm instead of hard
/// erroring on the sibling's missing file, aligning `descend` with `scan::resolve_child_modules`'s
/// identical `#[cfg]`-tolerance for a missing plain module file (previously the two walkers
/// silently disagreed on this exact shape — the 0.2.2 lesson).
#[test]
pub(super) fn descend_tolerates_a_cfg_gated_missing_sibling_when_an_inline_arm_resolves() {
    let file = resolve_file(
        "cfg-shim",
        &[(
            "lib.rs",
            "#[cfg(unix)]\npub mod shared { pub struct A; }\n\
             #[cfg(windows)]\npub mod shared;\n",
        )],
        "crate::shared",
    )
    .expect("the inline arm resolves even though the windows-only file-form sibling has no file");
    assert!(file.ends_with("lib.rs"), "got {}", file.display());
}

/// When EVERY declaration for the anchored module is `#[cfg]`-gated and none resolves (no inline
/// sibling to fall back on), resolution still fails loud — never a silent, vacuous "zero items"
/// pass. `descend`'s own `next_branches.is_empty()` guard (which already existed for the ordinary
/// "no branch survived this segment" case) catches this for free: cfg-tolerance only ever removes
/// candidates, so an entirely-eliminated segment reads the same as an always-had genuinely unknown
/// one.
#[test]
pub(super) fn descend_still_errors_when_every_candidate_for_a_module_is_cfg_gated_missing() {
    let err = resolve_file(
        "cfg-only-missing",
        &[("lib.rs", "#[cfg(feature = \"absent\")]\npub mod gated;\n")],
        "crate::gated",
    )
    .expect_err("a module with no surviving branch must be a scan error, never a vacuous pass");
    assert_eq!(err, unknown_module_error("crate::gated", "x"));
}

/// A BARE `#[cfg]`-gated missing file is tolerated (the sibling test above), but a
/// `#[cfg_attr(pred, …)]`-decorated one is NOT: unlike a bare `#[cfg]`, `cfg_attr` never removes
/// the `mod` item itself — it only conditionally applies its wrapped attribute — so the file must
/// always exist regardless of the predicate. Verified against a real `rustc` build: this exact
/// shape (`#[cfg_attr(unix, allow(dead_code))] mod gated;` with no `gated.rs`) is E0583 on every
/// platform. `has_cfg_attr` deliberately does not match `cfg_attr` for this reason.
#[test]
pub(super) fn descend_does_not_tolerate_a_cfg_attr_decorated_missing_file_only_bare_cfg() {
    let err = resolve_file(
        "cfg-attr-not-tolerated",
        &[(
            "lib.rs",
            "#[cfg_attr(unix, allow(dead_code))]\npub mod gated;\n",
        )],
        "crate::gated",
    )
    .expect_err("a cfg_attr-decorated (not cfg-gated) missing file must still be a scan error");
    assert_eq!(err, missing_module_file_error("crate::gated", "x"));
}

/// A plain `mod child;` backed by BOTH conventional forms at once (`child.rs` AND `child/mod.rs`) is
/// a genuine rustc compile error (E0761) for a live declaration. `locate_module_file` previously
/// returned the FIRST form it probed, so the anchor silently resolved to `child.rs` and the other
/// file was never read. 圭表 and 漏刻 each already hard-error on this exact shape; this brings the
/// semantic dimension into that agreement (pinned across all three in
/// `crates/tianheng/tests/dual_backed_module_conformance.rs`).
#[test]
pub(super) fn a_dual_backed_module_anchor_is_a_constitution_error() {
    let tree = TempSrcTree::new("dual-backed-anchor");
    tree.write_all(&[
        ("lib.rs", "pub mod child;\n"),
        ("child.rs", "// flat form\n"),
        ("child/mod.rs", "// nested form\n"),
    ]);
    let err = resolve_module_file(tree.src(), &tree.root(), "crate::child", "x")
        .expect_err("both conventional forms present must be a constitution error, not a pick");
    assert_eq!(
        err,
        dual_backed_module_error(
            "crate::child",
            "child",
            "x",
            &tree.src().join("child.rs"),
            &tree.src().join("child").join("mod.rs"),
        )
    );
}

/// When the ambiguous declaration is an ANCESTOR of the anchor, the error must name the anchor being
/// resolved AND the ambiguous `mod` declaration separately — the two resolved paths belong to the
/// ancestor, so attributing them to the deeper anchor would point a reader at the wrong module. The
/// `openspec` requirement claims this case explicitly, so it is pinned rather than assumed.
#[test]
pub(super) fn a_dual_backed_ancestor_reacts_when_the_anchor_is_a_deeper_segment() {
    let tree = TempSrcTree::new("dual-backed-ancestor");
    tree.write_all(&[
        ("lib.rs", "pub mod child;\n"),
        ("child.rs", "pub mod deep;\n"),
        ("child/mod.rs", "pub mod deep;\n"),
        ("child/deep.rs", "pub struct A;\n"),
    ]);
    let err = resolve_module_file(tree.src(), &tree.root(), "crate::child::deep", "x")
        .expect_err("a dual-backed ancestor must react before the deeper segment resolves");
    assert_eq!(
        err,
        dual_backed_module_error(
            "crate::child::deep",
            "child",
            "x",
            &tree.src().join("child.rs"),
            &tree.src().join("child").join("mod.rs"),
        ),
        "the anchor and the ambiguous declaration must be named separately"
    );
}

/// The motivating false negative, through the real capability rather than the resolver alone: with
/// `child.rs` clean and the forbidden exposure written only in `child/mod.rs`, the previous
/// first-form pick read the clean file and returned **zero findings** — a silent pass. Whether the
/// module was governed at all therefore depended on which of the two files its author wrote the item
/// in (the same boundary reacts when the exposure sits in `child.rs`, the sibling control below).
#[test]
pub(super) fn a_dual_backed_anchor_does_not_let_an_exposure_in_the_unselected_form_escape() {
    let err = findings(
        "dual-backed-escape",
        &[
            ("lib.rs", "pub mod child;\npub mod infra;\n"),
            ("infra.rs", "pub struct DbPool;\n"),
            ("child.rs", "// nothing exposed here\n"),
            (
                "child/mod.rs",
                "pub fn pools() -> crate::infra::DbPool { crate::infra::DbPool }\n",
            ),
        ],
        "crate::child",
        &["crate::infra"],
    )
    .expect_err("an exposure in the unselected conventional form must never be a silent pass");
    assert!(
        err.contains("resolves to both"),
        "the ambiguity must be named, not the exposure: {err}"
    );
}

/// The control for the test above: the identical boundary and forbidden type DO react when the
/// exposure sits in the single conventional form. Without this, the `expect_err` above could pass on
/// a misconfigured boundary that never reacts to anything.
#[test]
pub(super) fn a_single_form_anchor_still_reacts_on_the_same_exposure() {
    let out = findings(
        "single-form-control",
        &[
            ("lib.rs", "pub mod child;\npub mod infra;\n"),
            ("infra.rs", "pub struct DbPool;\n"),
            (
                "child.rs",
                "pub fn pools() -> crate::infra::DbPool { crate::infra::DbPool }\n",
            ),
        ],
        "crate::child",
        &["crate::infra"],
    )
    .expect("a single-form anchor resolves and the boundary reacts");
    assert_eq!(
        out,
        ["crate::infra::DbPool exposed by fn crate::child::pools"]
    );
}

/// A `mod.rs`-only module must still resolve — the ambiguity reaction must not have collapsed the
/// nested branch along with the ambiguous one.
#[test]
pub(super) fn a_nested_only_module_still_resolves_to_its_mod_rs() {
    let file = resolve_file(
        "nested-only",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain/mod.rs", "pub struct A;\n"),
        ],
        "crate::domain",
    )
    .expect("a `mod.rs`-only module resolves to that file");
    assert!(file.ends_with("domain/mod.rs"), "got {}", file.display());
}

/// Unlike an ABSENT conventional file, an ambiguity is not a legitimate configuration: no `#[cfg]`
/// predicate value makes two files compile as one module. So the ambiguity test runs ahead of the
/// bare-`#[cfg]` absence tolerance, exactly as 圭表's `resolve_plain_sources` and 漏刻's
/// `resolve_external_module` each independently order it.
///
/// Worth stating plainly: with the predicate off, rustc strips the declaration before module
/// resolution, so this crate COMPILES and raises no E0761 — the reaction here is deliberately not
/// confined to uncompilable source, because a cfg-blind scanner cannot know which arm is live.
#[test]
pub(super) fn a_cfg_gated_dual_backed_declaration_is_still_an_ambiguity() {
    let tree = TempSrcTree::new("dual-backed-cfg");
    tree.write_all(&[
        ("lib.rs", "#[cfg(feature = \"never\")]\npub mod child;\n"),
        ("child.rs", "// flat form\n"),
        ("child/mod.rs", "// nested form\n"),
    ]);
    let err = resolve_module_file(tree.src(), &tree.root(), "crate::child", "x")
        .expect_err("a cfg gate tolerates an absent file, never two present ones");
    assert_eq!(
        err,
        dual_backed_module_error(
            "crate::child",
            "child",
            "x",
            &tree.src().join("child.rs"),
            &tree.src().join("child").join("mod.rs"),
        )
    );
}

/// The `cfg_attr` counterpart of the test above. `has_cfg_attr` deliberately does not match
/// `cfg_attr` (it never removes the item), so a `cfg_attr`-decorated declaration is not even
/// eligible for the absence tolerance — but the requirement claims the ambiguity reacts "regardless
/// of any `#[cfg(...)]` or `#[cfg_attr(...)]` gate", so both gate spellings are pinned rather than
/// one being left to structural inference.
#[test]
pub(super) fn a_cfg_attr_decorated_dual_backed_declaration_is_still_an_ambiguity() {
    let tree = TempSrcTree::new("dual-backed-cfg-attr");
    tree.write_all(&[
        (
            "lib.rs",
            "#[cfg_attr(unix, allow(dead_code))]\npub mod child;\n",
        ),
        ("child.rs", "// flat form\n"),
        ("child/mod.rs", "// nested form\n"),
    ]);
    let err = resolve_module_file(tree.src(), &tree.root(), "crate::child", "x")
        .expect_err("a cfg_attr-decorated dual-backed declaration is still an ambiguity");
    assert_eq!(
        err,
        dual_backed_module_error(
            "crate::child",
            "child",
            "x",
            &tree.src().join("child.rs"),
            &tree.src().join("child").join("mod.rs"),
        )
    );
}

/// The crate-wide walk (`scan::resolve_child_modules`, behind trait-impl locality, forbidden marker,
/// unsafe confinement, and signature-coupling's own alias/extern scan) reacts on the same shape as
/// the anchored descent — the two walkers must not disagree on it, the 0.2.2 lesson that their
/// missing-file policies once silently had. The dual-backed module here is unrelated to the
/// boundary's own trait, so this also pins the wider blast radius: a crate whose module graph cannot
/// be resolved cannot be judged, rather than the one module being quietly excluded.
#[test]
pub(super) fn the_crate_wide_walk_reacts_on_a_dual_backed_module_elsewhere_in_the_crate() {
    let err = locality_findings(
        "dual-backed-crate-wide",
        &[
            ("lib.rs", "pub mod command;\npub mod child;\n"),
            ("command.rs", "pub trait Command {}\n"),
            ("child.rs", "// flat form\n"),
            ("child/mod.rs", "// nested form\n"),
        ],
        "crate::command::Command",
        &["crate::commands"],
    )
    .expect_err("the crate-wide walk must refuse to judge a crate with an unresolvable module");
    assert!(
        err.contains("resolves to both") && err.contains("crate::child"),
        "the crate-wide walk must name the ambiguous module: {err}"
    );
}

/// A BARE `#[cfg(pred)]` co-occurring with an unconditional `#[path = "…"]` on the SAME item
/// removes the whole item, `#[path]` included, when `pred` is false — a standard per-platform
/// shim (`#[cfg(windows)] #[path = "windows_impl.rs"] mod imp;`) that must not hard-error `descend`
/// merely because this platform's target file was never written. Verified against a real `rustc`
/// build: this compiles cleanly with the target entirely absent. The mutually-exclusive inline
/// sibling arm (always present, no file needed) still resolves.
#[test]
pub(super) fn descend_tolerates_a_cfg_gated_unconditional_path_target_when_missing() {
    let file = resolve_file(
        "cfg-path-shim",
        &[(
            "lib.rs",
            "#[cfg(unix)]\npub mod shared { pub struct A; }\n\
             #[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod shared;\n",
        )],
        "crate::shared",
    )
    .expect("the inline arm resolves even though the windows-only #[path] target has no file");
    assert!(file.ends_with("lib.rs"), "got {}", file.display());
}

/// The crate-wide walker (`scan::resolve_child_modules`, backing `semantic-unsafe-confinement`,
/// which has no single-module anchor mode) must tolerate the identical shape: a cfg-gated
/// unconditional `#[path]` target with no file must not fail the whole scan, so an unrelated
/// module's real `unsafe` site is still observed.
#[test]
pub(super) fn resolve_child_modules_tolerates_a_cfg_gated_unconditional_path_target_when_missing() {
    let out = unsafe_labels(
        "cfg-path-shim-crate-wide",
        &[
            (
                "lib.rs",
                "#[cfg(windows)]\n#[path = \"windows_impl.rs\"]\npub mod imp;\npub mod live;\n",
            ),
            ("live.rs", "unsafe fn f() {}\n"),
        ],
        &["crate::allowed_elsewhere"],
    )
    .expect("a cfg-gated #[path] target with no file must not fail the crate-wide scan");
    assert_eq!(out, ["unsafe fn f in crate::live"]);
}

#[test]
pub(super) fn module_file_is_mod_rs_for_a_nested_module() {
    let file = resolve_file(
        "nested",
        &[
            ("lib.rs", "pub mod domain;\n"),
            ("domain/mod.rs", "pub struct A;\n"),
        ],
        "crate::domain",
    )
    .unwrap();
    assert!(file.ends_with("domain/mod.rs"), "got {}", file.display());
}

#[test]
pub(super) fn module_file_is_the_enclosing_file_for_an_inline_submodule() {
    // `crate::inner` is inline in lib.rs, so its file is lib.rs — never a (non-existent)
    // inner.rs. This is the case the naive "module name → <name>.rs" guess gets wrong.
    let file = resolve_file(
        "inline",
        &[("lib.rs", "pub mod inner { pub struct A; }\n")],
        "crate::inner",
    )
    .unwrap();
    assert!(file.ends_with("src/lib.rs"), "got {}", file.display());
}

#[test]
pub(super) fn module_file_descends_a_deep_file_module() {
    let file = resolve_file(
        "deep",
        &[
            ("lib.rs", "pub mod a;\n"),
            ("a.rs", "pub mod b;\n"),
            ("a/b.rs", "pub struct A;\n"),
        ],
        "crate::a::b",
    )
    .unwrap();
    assert!(file.ends_with("a/b.rs"), "got {}", file.display());
}

#[test]
pub(super) fn module_file_follows_an_unconditional_path_on_an_inline_module_to_its_relocated_child()
{
    // rustc ground truth (verified with a real `cargo check`): `#[path = "thread_files"] pub mod
    // thread { pub mod local_data; }` compiles `thread_files/local_data.rs` as
    // `crate::thread::local_data`, with no `src/thread/` directory at all — the naive
    // (non-relocated) location does not even exist. Before the fix, `descend`'s inline-collection
    // loop skipped ANY `#[path]`-bearing mod (inline or not) before ever checking its content,
    // and the file-form loop then also skipped it (assuming it was "already collected above"),
    // so the item vanished from both loops — `crate::thread` itself failed with a spurious
    // "module not found" error, even though it demonstrably exists and compiles.
    let file = resolve_file(
        "inline-path-relocate",
        &[
            (
                "lib.rs",
                "#[path = \"thread_files\"]\npub mod thread {\n    pub mod local_data;\n}\n",
            ),
            ("thread_files/local_data.rs", "pub struct A;\n"),
        ],
        "crate::thread::local_data",
    )
    .unwrap();
    assert!(
        file.ends_with("thread_files/local_data.rs")
            || file.ends_with("thread_files\\local_data.rs"),
        "got {}",
        file.display()
    );
}

#[test]
pub(super) fn semantic_violation_carries_the_governed_module_file_not_the_types_file() {
    // The forbidden type `crate::infra::Db` is *defined* in infra.rs; the exposing seam is in
    // domain.rs. The reported `file` is the seam's module (domain.rs), the actionable one.
    let (metadata, _fixture) = fixture_metadata(
        "seam",
        &[
            ("lib.rs", "pub mod infra;\npub mod domain;\n"),
            ("infra.rs", "pub struct Db;\n"),
            (
                "domain.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("domain must not expose infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "one exposure violation");
    assert_eq!(violations[0].target(), "crate::domain");
    assert_eq!(violations[0].rule, SIGNATURE_RULE);
    let id = violations[0].id();
    let key = id.fact();
    let rule = id.rule_key();
    assert_eq!(rule.rule_type(), "tianheng.rule/hunyi/signature-exposure");
    assert_eq!(
        rule.fields().collect::<Vec<_>>(),
        vec![
            ("forbidden", "[\"crate::infra\"]"),
            ("including_trait_impls", "false"),
        ]
    );
    assert_eq!(key.fact_type(), "tianheng.fact/hunyi/signature-exposure");
    assert_eq!(key.shape(), "public-seam");
    assert_eq!(
        key.fields().collect::<Vec<_>>(),
        vec![
            ("governing_package", "x"),
            ("seam_kind", "free_fn"),
            ("seam_module", "crate::domain"),
            ("seam_name", "leak"),
            ("subject", "crate::infra::Db"),
        ]
    );
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(
        file.ends_with("domain.rs"),
        "the file is the seam's module (domain.rs), not the type's file (infra.rs): got {file}"
    );
}

#[test]
pub(super) fn the_semantic_file_is_not_part_of_the_baseline_identity() {
    let (metadata, _fixture) = fixture_metadata(
        "baseline",
        &[
            ("lib.rs", "pub mod infra;\npub mod domain;\n"),
            ("infra.rs", "pub struct Db;\n"),
            (
                "domain.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("r");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    let v = &violations[0];
    assert!(v.file.is_some(), "the violation now carries a file");
    // `file` is metadata, not identity: a violation baselined while `file` was null still
    // matches once populated, so populating it never re-baselines or changes the count.
    assert_eq!(v.id(), v.clone().with_file(None).id());
}

#[test]
pub(super) fn cfg_duplicated_inline_modules_are_all_governed() {
    // Two `#[cfg(..)] mod platform {..}` variants parse as separate inline modules (syn does not
    // evaluate cfg). A signature-coupling boundary anchored on `crate::platform` must observe BOTH:
    // resolving only the source-first variant let a forbidden exposure in the other pass unobserved
    // (exit 0) — a mod-resolution divergence, the forbidden false-negative class. Matches the
    // crate-wide scan's observe-all policy for same-named modules.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-dup-platform",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(unix)] pub mod platform { pub fn open() -> u8 { 0 } }\n\
                 #[cfg(windows)] pub mod platform { pub fn open() -> crate::infra::Db { unimplemented!() } }\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in any cfg variant");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the non-source-first cfg variant's exposure must react: {violations:?}"
    );
}

#[test]
pub(super) fn cfg_mixed_inline_and_file_form_siblings_are_both_governed() {
    // rustc ground truth (verified with a real rustc build under EITHER single-feature config):
    // `#[cfg(feature = "a")] pub mod platform { .. }` (inline) and `#[cfg(feature = "b")] pub mod
    // platform;` (file-form, backed by platform.rs) is the standard per-platform shim pairing an
    // inline variant with a file-form one — valid, common Rust, not a name collision. `descend`
    // used to return as soon as it found ANY inline variant, never reading the file-form sibling
    // at all: a boundary anchored on `crate::platform` observed only the inline arm's exposures,
    // silently missing the file-form arm's — a real false negative (the resolver never even
    // opened platform.rs). Both must react now, matching the crate-wide scan's own cfg-blind,
    // observe-all policy for same-named children.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-inline-file",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform { pub fn open() -> u8 { 0 } }\n\
                 #[cfg(feature = \"b\")] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in any cfg variant, inline or file-form");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the file-form sibling's exposure must react even though an inline variant exists: {violations:?}"
    );
}

#[test]
pub(super) fn a_semantic_boundary_anchored_at_an_inline_module_with_an_unconditional_path_reacts_instead_of_erroring()
 {
    // Before the fix, ANY single-module-anchored capability (signature-coupling-exposure,
    // dyn-trait-boundary, impl-trait-boundary, visibility-boundary, and async-exposure's
    // non-subtree seam) hard-failed with a spurious "module not found" (exit 2) when anchored at
    // an inline module carrying an unconditional `#[path]` — or any of its descendants — even
    // though hunyi's own crate-wide walker (`walk_subtree_modules`/`resolve_child_modules`)
    // resolved the identical layout without trouble. This asserts the single-module path now
    // agrees with the crate-wide one: the boundary must react on the real exposure, not error.
    let (metadata, _fixture) = fixture_metadata(
        "inline-path-boundary",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[path = \"thread_files\"]\npub mod thread {\n    pub mod local_data;\n}\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "thread_files/local_data.rs",
                "pub fn leak() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::thread::local_data")
        .must_not_expose("crate::infra")
        .because("an inline module's own #[path] must not make its children unresolvable");
    let mut violations = Vec::new();
    let result = check_boundary(&metadata, &boundary, &mut violations);
    result.expect("crate::thread::local_data must resolve, not hard-error as an unknown module");
    assert_eq!(
        violations.len(),
        1,
        "the relocated child's exposure must still be observed: {violations:?}"
    );
}

#[test]
pub(super) fn a_further_segment_beneath_a_flat_file_form_cfg_sibling_resolves_from_its_own_directory()
 {
    // rustc ground truth (verified with a real rustc build under the "b" feature): a flat
    // (non-`mod.rs`) file-form cfg sibling's OWN `#[path]` resolves relative to ITS OWN
    // containing directory, not `<child_dir>/<its own name>/` — the same rule an ordinary flat
    // file always follows, regardless of whether it also happens to pair with a mutually-
    // exclusive `#[cfg]` inline sibling. Before the fix, descend()'s merged-branch case
    // unconditionally continued a further segment from the INLINE sibling's accumulated
    // directory, which only coincides with a `mod.rs`-style file-form sibling's own directory —
    // silently misresolving (or hard-erroring on) the real target for a flat one instead.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-flat-further-segment",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod plat { pub struct Marker; }\n\
                 #[cfg(feature = \"b\")] #[path = \"moved/plat_moved.rs\"] pub mod plat;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "moved/plat_moved.rs",
                "#[path = \"elsewhere.rs\"]\npub mod target;\n",
            ),
            (
                "moved/elsewhere.rs",
                "pub fn get() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::plat::target")
        .must_not_expose("crate::infra")
        .because(
            "plat::target must not expose infra even through a flat cfg-sibling's own #[path]",
        );
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the flat file-form sibling's own #[path] target (moved/elsewhere.rs, a SIBLING of \
         plat_moved.rs, not a child of a plat/ subdirectory) must be read and react: {violations:?}"
    );
}

#[test]
pub(super) fn a_plain_child_of_a_path_remapped_module_resolves_from_the_remaps_own_directory() {
    // rustc ground truth (verified with a real rustc build): `#[path = "moved/thing.rs"] pub mod
    // net;` makes `moved/thing.rs` mod-rs-like, so its own plain `pub mod inner;` resolves to
    // `moved/inner.rs`, NOT `net/inner.rs` (a name-derived location that has nothing to do with
    // where the file actually lives). descend()'s Branch redesign correctly threads `path_base`
    // for a FURTHER `#[path]` beneath a `#[path]`-loaded file, but a `child_dir` bug left the
    // CONVENTIONAL-child continuation still computed as the naive `<child_dir>/<seg>` regardless
    // of origin — silently resolving a plain child of a #[path]-remapped module at the wrong,
    // uncompiled location.
    let (metadata, _fixture) = fixture_metadata(
        "path-remap-plain-child",
        &[
            (
                "lib.rs",
                "pub mod infra;\n#[path = \"moved/thing.rs\"]\npub mod net;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("moved/thing.rs", "pub mod inner;\n"),
            (
                "moved/inner.rs",
                "pub fn get() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::net::inner")
        .must_not_expose("crate::infra")
        .because("net::inner must not expose infra even though net is #[path]-remapped");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "moved/inner.rs (the real, rustc-compiled file) must be read and react: {violations:?}"
    );
}

#[test]
pub(super) fn cfg_mixed_plain_and_path_remapped_file_form_siblings_are_both_governed() {
    // rustc ground truth (verified with a real rustc build under either single-feature config):
    // `#[cfg(feature = "a")] pub mod platform;` (plain, backed by platform.rs) paired with
    // `#[cfg(feature = "b")] #[path = "win_platform.rs"] pub mod platform;` (remapped) is the
    // standard per-platform shim between two NON-inline variants — valid, common Rust, and once
    // #[path]-following exists the two variants need not name the same file at all. descend()'s
    // file-form search used to `break` at the first non-inline match regardless of source order,
    // silently dropping whichever variant did not win the race. Matching
    // `resolve_child_modules`'s own crate-wide policy (which never breaks after one match),
    // EVERY non-inline declaration for a segment now produces its own branch.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-plain-and-remapped-file-form",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform;\n\
                 #[cfg(feature = \"b\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("platform.rs", "pub fn open() -> u8 { 0 }\n"),
            (
                "win_platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("platform must not expose infra in either the plain or #[path]-remapped arm");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the #[path]-remapped sibling's exposure must react even though a plain sibling was \
         declared first in source order: {violations:?}"
    );
}

#[test]
pub(super) fn a_cfg_mixed_single_module_violation_names_the_offending_sibling_not_the_first_branch()
{
    // Round-5 finding: resolve_module_root unions every surviving branch's ITEMS (fixed above —
    // the violation still fires) but used to always report `branches[0]`'s FILE regardless of
    // which branch actually produced the finding. Here the plain, clean `platform;` arm is
    // declared FIRST (branches[0]) and the offending #[path]-remapped `win_platform.rs` arm is
    // declared second — before the fix, `.file` named platform.rs, which contains no reference to
    // `crate::infra` at all. Every single-module finding now pairs with the real file its own
    // item's branch was resolved from, so `.file` must name win_platform.rs, where the offending
    // seam is actually written.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-mixed-file-names-offending-branch",
        &[
            (
                "lib.rs",
                "pub mod infra;\n\
                 #[cfg(feature = \"a\")] pub mod platform;\n\
                 #[cfg(feature = \"b\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("platform.rs", "pub fn open() -> u8 { 0 }\n"),
            (
                "win_platform.rs",
                "pub fn open() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("the reported file must name the sibling that actually exposes infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "{violations:?}");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a semantic exposure violation carries its source file");
    assert!(
        file.ends_with("win_platform.rs"),
        "expected the offending sibling win_platform.rs, got {file} — a clean file must never \
         be reported as the source of a real violation: {violations:?}"
    );
}

#[test]
pub(super) fn a_cfg_split_module_does_not_let_one_arms_use_alias_shadow_the_others() {
    // Round-6 finding: module_findings called collect_uses ONCE over the flattened union of every
    // #[cfg] branch's items, so two mutually-exclusive branches each declaring `use <different
    // path> as Handle;` collided in one shared use-map -- the branch unioned LAST silently
    // overwrote the earlier branch's mapping, misresolving the FIRST branch's own bare `Handle`
    // reference through the SECOND branch's `use` and hiding a real forbidden-exposure finding.
    // Verified against real rustc: both platform.rs and win_platform.rs compile cleanly under
    // their own respective feature. A control fixture with the identical platform.rs but no cfg
    // split correctly reports 1 violation, confirming this is a cfg-split-specific regression,
    // not a general resolution gap.
    let (metadata, _fixture) = fixture_metadata(
        "cfg-split-use-alias-collision",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod other;\n\
                 #[cfg(feature = \"u\")] pub mod platform;\n\
                 #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("other.rs", "pub struct Widget;\n"),
            (
                "platform.rs",
                "use crate::infra::Db as Handle;\npub fn leak() -> Handle { unimplemented!() }\n",
            ),
            (
                "win_platform.rs",
                "use crate::other::Widget as Handle;\npub fn leak2() -> Handle { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::platform")
        .must_not_expose("crate::infra")
        .because("the unix arm's own Handle alias must resolve to infra, not the windows arm's");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(
        violations.len(),
        1,
        "the unix arm's leak() -> Handle genuinely exposes crate::infra::Db and must react: {violations:?}"
    );
}

#[test]
pub(super) fn a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_reexport() {
    // Round-7 finding: module_findings still computed child_mods/externs_type/externs_reexport/
    // renames_bare ONCE over the flattened union of every #[cfg] branch's items -- the identical
    // conflation round 6 fixed for the use-map, left unfixed here. The "u" branch (platform.rs)
    // declares a LOCAL `mod net { .. }`; the mutually-exclusive "w" branch (win_platform.rs) has
    // no local `mod net` at all and its own `pub use net::Something;` genuinely names the real
    // extern crate `net` -- verified against real rustc/cargo (win_platform.rs alone, with the
    // `net` dependency declared, compiles cleanly). Before the fix, the "u" branch's local `mod
    // net` silently suppressed the "w" branch's own genuine extern re-export, since child_mods
    // (computed over the union) always contained "net".
    let out = findings_with_deps(
        "cfg-sibling-childmod-shadow",
        &[
            (
                "lib.rs",
                "#[cfg(feature = \"u\")] pub mod platform;\n\
                 #[cfg(feature = \"w\")] #[path = \"win_platform.rs\"] pub mod platform;\n",
            ),
            (
                "platform.rs",
                "pub mod net { pub struct Something; }\npub fn open() -> u8 { 0 }\n",
            ),
            ("win_platform.rs", "pub use net::Something;\n"),
        ],
        "crate::platform",
        &["net::Something"],
        &["net"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["net::Something exposed by pub use crate::platform::Something"],
        "the w branch's own genuine extern re-export must react, regardless of the u branch's \
         own local mod net: {out:?}"
    );
}

#[test]
pub(super) fn a_cfg_split_module_with_two_inline_siblings_does_not_let_one_arms_use_alias_shadow_the_others()
 {
    // Round-8 finding: `descend()` used to MERGE every same-named inline `#[cfg]` occurrence into
    // one shared `Branch` before this whole per-file fix (round 6) even had a chance to run, so
    // the round-6/7 "per-file" use-map/shadow-set grouping was structurally a no-op for two INLINE
    // siblings — they always shared one `Branch`, one merged items list, one merged use-map,
    // regardless of which file's identity that fix grouped by. `descend()` now gives each inline
    // occurrence its OWN branch (mirroring the file-form loop), but two inline siblings still
    // share the identical ENCLOSING file (lib.rs here) — so `resolve_module_items_with_files`
    // pairs each item with a BRANCH INDEX, not just a file, and `module_findings` groups by that
    // index. This is the identical `a_cfg_split_module_does_not_let_one_arms_use_alias_shadow_the_others`
    // scenario (round 6), but with BOTH arms declared INLINE in the SAME file rather than as two
    // separate file-form siblings — exercising the file-keyed grouping's own blind spot.
    let out = findings(
        "cfg-split-inline-inline-use-alias-collision",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod other;\n\
             #[cfg(feature = \"u\")] pub mod platform {\n\
             use crate::infra::Db as Handle;\n\
             pub fn leak() -> Handle { unimplemented!() }\n}\n\
             #[cfg(feature = \"w\")] pub mod platform {\n\
             use crate::other::Widget as Handle;\n\
             pub fn leak2() -> Handle { unimplemented!() }\n}\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("other.rs", "pub struct Widget;\n"),
        ],
        "crate::platform",
        &["crate::infra"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["crate::infra::Db exposed by fn crate::platform::leak"],
        "the u arm's own Handle alias must resolve to infra, not the w arm's, even though both \
         arms are inline and share lib.rs: {out:?}"
    );
}

#[test]
pub(super) fn a_cfg_split_module_with_two_inline_siblings_child_module_does_not_shadow_the_others_extern_reexport()
 {
    // Round-8 finding, the childmod/extern-reexport analogue of the test above (round 7's own
    // file-form version is `a_cfg_sibling_child_module_does_not_shadow_a_different_branchs_own_extern_reexport`).
    // The "u" arm declares a LOCAL `mod net { .. }` inline; the mutually-exclusive "w" arm — also
    // inline, sharing the identical lib.rs — has no local `mod net` at all, so its own `pub use
    // net::Something;` genuinely names the real extern crate `net`. Grouping by file alone would
    // let the "u" arm's local `mod net` suppress the "w" arm's genuine extern re-export merely
    // because both share one file; grouping by branch index keeps them apart.
    let out = findings_with_deps(
        "cfg-split-inline-inline-childmod-shadow",
        &[(
            "lib.rs",
            "#[cfg(feature = \"u\")] pub mod platform {\n\
             pub mod net { pub struct Something; }\n\
             pub fn open() -> u8 { 0 }\n}\n\
             #[cfg(feature = \"w\")] pub mod platform {\n\
             pub use net::Something;\n}\n",
        )],
        "crate::platform",
        &["net::Something"],
        &["net"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["net::Something exposed by pub use crate::platform::Something"],
        "the w arm's own genuine extern re-export must react, regardless of the u arm's own \
         local mod net, even though both arms are inline and share lib.rs: {out:?}"
    );
}

#[test]
pub(super) fn a_bare_cfg_negated_sibling_child_module_does_not_shadow_the_others_extern_reexport() {
    // Round-9 finding (audit `crates/hunyi/src/exposure.rs:157`): rounds 6-8 fixed child_mods
    // being computed once over the UNION of every #[cfg]-*branch*'s items (a branch = a distinct
    // candidate resolution of the governed MODULE ITSELF, produced by `descend()`'s per-occurrence
    // splitting). This finding is one level finer: `#[cfg(unix)] mod serde;` and
    // `#[cfg(not(unix))] pub use serde::Value;` are two SIBLING ITEMS inside the SAME file/branch
    // (there is no module-path split here at all -- `api` resolves to exactly one branch), so the
    // existing per-branch grouping is a no-op and `child_module_names` still runs cfg-blind over
    // both items together. The "unix" mod and the "not(unix)" pub use are never compiled
    // together -- verified against real rustc, api.rs alone compiles cleanly on every platform --
    // so the mod must not shadow the use's own genuine extern re-export.
    let out = findings_with_deps(
        "cfg-negated-sibling-childmod-shadow",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "#[cfg(unix)]\nmod serde;\n#[cfg(not(unix))]\npub use serde::Value;\n",
            ),
            ("api/serde.rs", "pub struct Local;\n"),
        ],
        "crate::api",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["serde::Value exposed by pub use crate::api::Value"],
        "the not(unix) arm's own genuine extern re-export must react, regardless of the unix \
         arm's own local mod serde, since the two are never compiled together: {out:?}"
    );
}

#[test]
pub(super) fn a_cfg_if_sibling_child_module_does_not_shadow_the_other_arms_extern_reexport() {
    // The `cfg_if!` form of the round-9 finding above: `mod serde;` and `pub use serde::Value;`
    // are declared in two arms of the SAME invocation, flattened into one shared item list by
    // `flatten_transparent_macro_items` before `module_findings` ever sees them -- so, like the
    // bare-#[cfg] form, there is no branch split to lean on and `child_module_names` must instead
    // recognize the two arms as mutually exclusive on its own.
    let out = findings_with_deps(
        "cfg-if-sibling-childmod-shadow",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "cfg_if::cfg_if! {\n    if #[cfg(unix)] {\n        mod serde;\n    } else {\n        pub use serde::Value;\n    }\n}\n",
            ),
            ("api/serde.rs", "pub struct Local;\n"),
        ],
        "crate::api",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["serde::Value exposed by pub use crate::api::Value"],
        "the else arm's own genuine extern re-export must react, regardless of the if arm's own \
         local mod serde, since cfg_if arms are never compiled together: {out:?}"
    );
}

#[test]
pub(super) fn a_bare_cfg_negated_sibling_child_module_does_not_shadow_a_facades_extern_reexport() {
    // The crate-wide-closure sibling of the two round-9 findings above: `crate::a`'s two
    // mutually-exclusive sibling items are reached only THROUGH a local facade
    // (`crate::domain`'s `pub use crate::a::Value;`), not directly by the governed module itself
    // -- so this exercises `scan.rs`'s `collect_reexports`/`walk_module`, not `module_findings`'s
    // own direct-head resolution. `collect_reexports` computed its child-module shadow the
    // identical cfg-blind way `module_findings` used to, so `crate::a`'s own local `mod serde`
    // (cfg(unix)) must not suppress recording `crate::a::Value -> serde::Value` in the crate-wide
    // reexport closure just because a mutually-exclusive `cfg(not(unix))` sibling in the SAME
    // file happens to declare it.
    let out = findings_with_deps(
        "cfg-negated-sibling-childmod-shadow-facade",
        &[
            ("lib.rs", "pub mod a;\npub mod domain;\n"),
            (
                "a.rs",
                "#[cfg(unix)]\nmod serde;\n#[cfg(not(unix))]\npub use serde::Value;\n",
            ),
            ("a/serde.rs", "pub struct Local;\n"),
            ("domain.rs", "pub use crate::a::Value;\n"),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["serde::Value exposed by pub use crate::domain::Value"],
        "the facade must still canonicalize to the not(unix) arm's genuine extern re-export, \
         regardless of the unix arm's own local mod serde in the SAME defining module: {out:?}"
    );
}

#[test]
pub(super) fn a_mutually_exclusive_sibling_child_module_does_not_shadow_a_rename_aliased_reexport()
{
    // The crate-root-rename-alias sibling of the round-9 finding above (found by an independent
    // adversarial review of the fix, not the original audit): `renames_bare` -- the shadow applied
    // to a bare head that resolves through a crate-root `extern crate X as Y;` alias, per
    // `extern_verbatim_renamed`'s rename-map-before-externs-set precedence -- was left cfg-blind
    // even after `mod_decls`/`reexport_externs_for` made the plain extern-name shadow cfg-aware.
    // `#[cfg(unix)] mod wc;` beside `#[cfg(not(unix))] pub use wc::Value;`, with a crate-root
    // `extern crate serde as wc;` rename, is never compiled together (verified against real rustc:
    // api.rs alone compiles cleanly on every platform) -- so the unix arm's own local `mod wc` must
    // not shadow the not(unix) arm's own genuine `wc::Value` (== `serde::Value`) re-export.
    let out = findings_with_deps(
        "cfg-negated-sibling-childmod-shadow-rename-alias",
        &[
            ("lib.rs", "extern crate serde as wc;\npub mod api;\n"),
            (
                "api.rs",
                "#[cfg(unix)]\nmod wc;\n#[cfg(not(unix))]\npub use wc::Value;\n",
            ),
            ("api/wc.rs", "pub struct Local;\n"),
        ],
        "crate::api",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["serde::Value exposed by pub use crate::api::Value"],
        "the not(unix) arm's own genuine rename-aliased re-export must react, regardless of the \
         unix arm's own local mod wc, since the two are never compiled together: {out:?}"
    );
}

#[test]
pub(super) fn a_mutually_exclusive_sibling_child_module_does_not_shadow_a_rename_aliased_facade_reexport()
 {
    // The crate-wide-closure sibling of the rename-alias fix above: `crate::a`'s mutually-exclusive
    // `mod wc;` / `pub use wc::Value;` pair is reached only THROUGH a facade
    // (`crate::domain`'s `pub use crate::a::Value;`), exercising `collect_reexports`'s own
    // `renames_bare` computation rather than `module_findings`'s direct-head resolution.
    let out = findings_with_deps(
        "cfg-negated-sibling-childmod-shadow-rename-alias-facade",
        &[
            (
                "lib.rs",
                "extern crate serde as wc;\npub mod a;\npub mod domain;\n",
            ),
            (
                "a.rs",
                "#[cfg(unix)]\nmod wc;\n#[cfg(not(unix))]\npub use wc::Value;\n",
            ),
            ("a/wc.rs", "pub struct Local;\n"),
            ("domain.rs", "pub use crate::a::Value;\n"),
        ],
        "crate::domain",
        &["serde"],
        &["serde"],
    )
    .unwrap();
    assert_eq!(
        out,
        ["serde::Value exposed by pub use crate::domain::Value"],
        "the facade must still canonicalize to the not(unix) arm's genuine rename-aliased \
         re-export, regardless of the unix arm's own local mod wc in the SAME defining module: \
         {out:?}"
    );
}

#[test]
pub(super) fn async_subtree_observes_both_arms_of_a_two_inline_sibling_cfg_split_anchor() {
    // Round-8 finding (b): when the async-exposure subtree boundary is anchored DIRECTLY at a
    // module reached through two mutually-exclusive INLINE `#[cfg]` siblings sharing one file,
    // `walk_subtree_modules` must observe EACH arm's own async fn — never merging the two arms'
    // items into one shared list (which happened to still union both fns correctly under the old
    // pre-round-8 `descend()`, since shape-only observation over a union list drops nothing) nor
    // dropping either arm now that `descend()` gives each its own `Branch` and its own
    // `collect_subtree` call (two entries sharing one file, each with only its own arm's items).
    let files = &[(
        "lib.rs",
        "#[cfg(feature = \"u\")] pub mod platform { pub async fn unix_seam() {} }\n\
         #[cfg(feature = \"w\")] pub mod platform { pub async fn win_seam() {} }\n",
    )];
    let mut labels =
        async_subtree_labels("inline-inline-cfg-split-anchor", files, "crate::platform");
    labels.sort();
    assert_eq!(
        labels,
        [
            "async fn crate::platform::unix_seam()",
            "async fn crate::platform::win_seam()",
        ],
        "both inline cfg arms' own async fns must be observed, even though they share lib.rs: {labels:?}"
    );
}

#[test]
pub(super) fn a_visibility_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "vis",
        &[
            ("lib.rs", "pub mod internal;\n"),
            ("internal.rs", "pub struct Leaked;\n"),
        ],
    );
    let boundary = VisibilityBoundary::in_crate("x")
        .module("crate::internal")
        .must_not_declare_pub()
        .because("internal exposes no pub");
    let mut violations = Vec::new();
    check_visibility_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "a pub item in internal violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("internal.rs"), "got {file}");
}

#[test]
pub(super) fn a_trait_impl_locality_violation_carries_its_impl_site_file() {
    let (metadata, _fixture) = fixture_metadata(
        "locality",
        &[
            ("lib.rs", "pub mod plugins;\npub trait Command {}\n"),
            (
                "plugins.rs",
                "pub struct P;\nimpl crate::Command for P {}\n",
            ),
        ],
    );
    let boundary = TraitImplBoundary::in_crate("x")
        .trait_("crate::Command")
        .only_implemented_in("crate::allowed")
        .because("Command impls live in crate::allowed");
    let mut violations = Vec::new();
    check_trait_impl_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "the misplaced impl violates");
    let file = violations[0].file.as_deref().expect("the impl site's file");
    assert!(file.ends_with("plugins.rs"), "got {file}");
    // `file` is metadata, not identity.
    assert_eq!(
        violations[0].id(),
        violations[0].clone().with_file(None).id()
    );
}

#[test]
pub(super) fn a_trait_impl_in_a_nested_module_resolves_to_mod_rs() {
    let (metadata, _fixture) = fixture_metadata(
        "locality-nested",
        &[
            ("lib.rs", "pub mod plugins;\npub trait Command {}\n"),
            (
                "plugins/mod.rs",
                "pub struct P;\nimpl crate::Command for P {}\n",
            ),
        ],
    );
    let boundary = TraitImplBoundary::in_crate("x")
        .trait_("crate::Command")
        .only_implemented_in("crate::allowed")
        .because("Command impls live in crate::allowed");
    let mut violations = Vec::new();
    check_trait_impl_boundary(&metadata, &boundary, &mut violations).unwrap();
    let file = violations[0].file.as_deref().expect("the impl site's file");
    assert!(file.ends_with("plugins/mod.rs"), "got {file}");
}

#[test]
pub(super) fn forbidden_marker_impl_and_derive_each_name_their_own_module_file() {
    // A forbidden `impl` sits in internal.rs; a forbidden `#[derive]` sits on a type in
    // models.rs. Each finding must name its OWN module's file — the derive names the
    // defining type's file (models.rs), never the impl site's (internal.rs).
    let (metadata, _fixture) = fixture_metadata(
        "marker",
        &[
            (
                "lib.rs",
                "pub mod internal;\npub mod models;\npub trait Secret {}\n",
            ),
            (
                "internal.rs",
                "pub struct Bar;\nimpl crate::Secret for Bar {}\n",
            ),
            ("models.rs", "#[derive(Secret)]\npub struct Foo;\n"),
        ],
    );
    let boundary = ForbiddenMarkerBoundary::in_crate("x")
        .module("crate") // subtree = whole crate, so both Foo and Bar are under it
        .must_not_acquire("crate::Secret")
        .because("nothing may acquire Secret");
    let mut violations = Vec::new();
    check_forbidden_marker_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 2, "one impl finding + one derive finding");
    let impl_v = violations
        .iter()
        .find(|v| v.finding.starts_with("impl "))
        .expect("an impl finding");
    let derive_v = violations
        .iter()
        .find(|v| v.finding.starts_with("derive "))
        .expect("a derive finding");
    assert!(
        impl_v.file.as_deref().unwrap().ends_with("internal.rs"),
        "impl file: {:?}",
        impl_v.file
    );
    assert!(
        derive_v.file.as_deref().unwrap().ends_with("models.rs"),
        "derive file is the defining type's module, not an impl site: {:?}",
        derive_v.file
    );
}

#[test]
pub(super) fn a_dyn_trait_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "dyn",
        &[
            ("lib.rs", "pub mod api;\npub trait Port {}\n"),
            (
                "api.rs",
                "pub fn f() -> Box<dyn crate::Port> { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = DynTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_dyn()
        .because("the api seam is statically dispatched");
    let mut violations = Vec::new();
    check_dyn_trait_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the exposed dyn violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
pub(super) fn an_impl_trait_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "impltrait",
        &[
            ("lib.rs", "pub mod api;\n"),
            (
                "api.rs",
                "pub fn f() -> impl Iterator<Item = u8> { std::iter::empty() }\n",
            ),
        ],
    );
    let boundary = ImplTraitBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_impl_trait()
        .because("the api seam returns no existential");
    let mut violations = Vec::new();
    check_impl_trait_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the returned impl Trait violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
pub(super) fn an_async_exposure_violation_carries_its_module_file() {
    let (metadata, _fixture) = fixture_metadata(
        "async",
        &[
            ("lib.rs", "pub mod api;\n"),
            ("api.rs", "pub async fn f() {}\n"),
        ],
    );
    let boundary = AsyncExposureBoundary::in_crate("x")
        .module("crate::api")
        .must_not_expose_async_fn()
        .because("the api seam exposes no async fn");
    let mut violations = Vec::new();
    check_async_exposure_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert!(!violations.is_empty(), "the async fn violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(file.ends_with("api.rs"), "got {file}");
}

#[test]
pub(super) fn a_facade_chain_reexport_reports_the_governed_module_file_not_the_facades() {
    // The exposing seam (`pub use crate::facade::Db;`) is in domain.rs; the type is defined in
    // infra.rs and hopped through facade.rs. The reported file is the seam's module
    // (domain.rs) — the actionable one — never the type's or the intermediate facade's file.
    let (metadata, _fixture) = fixture_metadata(
        "facade",
        &[
            (
                "lib.rs",
                "pub mod infra;\npub mod facade;\npub mod domain;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            ("facade.rs", "pub use crate::infra::Db;\n"),
            ("domain.rs", "pub use crate::facade::Db;\n"),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("domain must not re-export infra");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations).unwrap();
    assert_eq!(violations.len(), 1, "the facade-chain re-export violates");
    let file = violations[0]
        .file
        .as_deref()
        .expect("a governed-module file");
    assert!(
        file.ends_with("domain.rs"),
        "the seam is in domain.rs, not infra.rs/facade.rs: got {file}"
    );
}

#[test]
pub(super) fn path_remapped_module_resolves_to_its_target_not_the_conventional_orphan() {
    // `crate::domain` is `#[path = "weird.rs"]`, so it resolves to weird.rs — the file rustc
    // compiles — and NEVER to the same-named conventional orphan `domain.rs` (which rustc does not
    // compile). The FP-guard intent survives the switch from skip to follow: the target, not the
    // orphan.
    let file = resolve_file(
        "path-remap",
        &[
            (
                "lib.rs",
                "#[path = \"weird.rs\"]\npub mod domain;\npub mod normal;\n",
            ),
            ("weird.rs", "pub struct Real;\n"),
            ("domain.rs", "pub struct Orphan;\n"),
            ("normal.rs", "pub struct Normal;\n"),
        ],
        "crate::domain",
    )
    .expect("an unconditional #[path] module now resolves to its target");
    let file = file.display().to_string();
    assert!(
        file.ends_with("weird.rs"),
        "the resolver follows #[path] to weird.rs, never the conventional orphan domain.rs: {file}"
    );
}

#[test]
pub(super) fn path_nested_in_an_inline_block_resolves_from_the_accumulated_dir_targeted() {
    // The targeted resolver's twin of the whole-crate walk fix. rustc ground truth (rustc 1.96.0):
    // `pub mod inline { #[path="other.rs"] pub mod inner; }` at the crate root resolves
    // crate::inline::inner to src/inline/other.rs. The earlier `descend` used current_file.parent()
    // (= src/) as the #[path] base, which drops the accumulated inline component — it would resolve
    // to the src/other.rs decoy (governing a file rustc never compiles = FP, and missing the real
    // src/inline/other.rs = FN). Pins the accumulated path_base.
    let file = resolve_file(
        "path-inline-targeted",
        &[
            (
                "lib.rs",
                "pub mod inline { #[path = \"other.rs\"] pub mod inner; }\n",
            ),
            ("inline/other.rs", "pub struct Real;\n"),
            ("other.rs", "pub struct Decoy;\n"),
        ],
        "crate::inline::inner",
    )
    .expect("a #[path] nested in an inline block resolves to its accumulated target");
    let file = file.display().to_string();
    assert!(
        file.replace('\\', "/").ends_with("inline/other.rs"),
        "the resolver accumulates the inline name: src/inline/other.rs, not the src/other.rs decoy: \
         {file}"
    );
}

#[test]
pub(super) fn path_remapped_semantic_module_is_governed_at_its_target_not_the_orphan() {
    // `crate::domain` is `#[path = "weird.rs"]`; the boundary is now evaluated against weird.rs
    // (the compiled file), whose `real() -> crate::infra::Db` violates `must_not_expose`. The
    // same-named conventional orphan `domain.rs` — which rustc does not compile — is never
    // governed, so its `orphan()` exposure is neither the source of a violation nor masks the real
    // one. Previously this was a constitution error (the module skipped) — a false negative.
    let (metadata, _fixture) = fixture_metadata(
        "semantic-path-remap",
        &[
            (
                "lib.rs",
                "#[path = \"weird.rs\"]\npub mod domain;\npub mod infra;\n",
            ),
            ("infra.rs", "pub struct Db;\n"),
            (
                "weird.rs",
                "pub fn real() -> crate::infra::Db { unimplemented!() }\n",
            ),
            (
                "domain.rs",
                "pub fn orphan() -> crate::infra::Db { unimplemented!() }\n",
            ),
        ],
    );
    let boundary = SignatureBoundary::in_crate("x")
        .module("crate::domain")
        .must_not_expose("crate::infra")
        .because("an unconditional #[path] module is governed at its target file");
    let mut violations = Vec::new();
    check_boundary(&metadata, &boundary, &mut violations)
        .expect("the #[path] target resolves and is governed");
    let file = violations
        .first()
        .and_then(|v| v.file.as_deref())
        .map(str::to_string);

    assert_eq!(violations.len(), 1, "weird.rs's exposure of infra reacts");
    let file = file.expect("a governed-module file");
    assert!(
        file.ends_with("weird.rs"),
        "the reaction is in the #[path] target weird.rs, never the conventional orphan domain.rs: {file}"
    );
}
