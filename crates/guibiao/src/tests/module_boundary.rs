use super::helpers::*;
/// An unreadable governed source file must surface as a scan error (exit 2),
/// not a silent skip that could hide a real module-boundary violation. Unix
/// only (permission-based) and self-calibrating: it skips under a privileged
/// user (e.g. root in CI), where mode 0 is still readable, rather than
/// false-passing.
#[cfg(unix)]
#[test]
pub(super) fn unreadable_governed_file_is_a_scan_error() {
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
pub(super) fn unreadable_governed_directory_is_a_scan_error() {
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
pub(super) fn a_raw_identifier_module_is_governed_and_its_import_observed() {
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
    assert_eq!(violations[0].target(), "crate::type");
    assert_eq!(violations[0].finding, "crate::mod::Thing");
}

#[test]
pub(super) fn module_boundary_uses_the_package_target_src_path() {
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
pub(super) fn path_remapped_module_is_followed_not_governed_via_a_conventional_orphan() {
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
    assert_eq!(violations[0].target(), "crate::kernel");
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
pub(super) fn an_unconditional_path_on_an_inline_module_relocates_its_own_file_form_children() {
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
    assert_eq!(violations[0].target(), "crate::thread");
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
pub(super) fn an_inline_module_target_is_a_self_describing_constitution_error() {
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
pub(super) fn an_inline_target_with_a_same_named_orphan_file_is_still_a_constitution_error() {
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
pub(super) fn an_orphan_beside_an_inline_module_contributes_no_phantom_child() {
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
pub(super) fn a_file_backed_module_is_still_governed() {
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
pub(super) fn a_cfg_dual_declared_module_keeps_governing_its_conventional_file() {
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
pub(super) fn a_cross_root_same_named_submodule_is_a_documented_bound() {
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
pub(super) fn a_dual_backed_module_is_a_scan_error_not_silently_accepted() {
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
pub(super) fn an_unconditional_missing_plain_module_file_is_a_scan_error_not_a_silent_gap() {
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
pub(super) fn a_cfg_gated_missing_plain_module_file_does_not_fail_an_unrelated_boundary() {
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

/// The same tolerance for the other spelling of one per-platform shim: a `mod` declared inside a
/// `cfg_if!` arm carries no `#[cfg]` attribute of its own — the predicate sits in the macro's
/// `if #[cfg(..)]` header — so before arm membership counted as cfg-conditional, this exact tree
/// exited 2 while the bare-attribute form above exited 0. rustc strips the non-selected arm, so the
/// source compiles: the scan was refusing to judge a working build, and refusing it for only one of
/// two equivalent forms.
#[test]
pub(super) fn a_missing_module_file_declared_inside_a_cfg_if_arm_is_tolerated() {
    let (result, violations) = run_module_check(
        "missing-plain-cfg-if-arm",
        &[
            (
                "lib.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub mod unix_impl;\n\
                 } else {\n\
                 pub mod windows_impl;\n\
                 }\n\
                 }\n\
                 pub mod present;\n",
            ),
            ("unix_impl.rs", "// clean\n"),
            ("present.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::present")
            .must_not_import("crate::forbidden")
            .because("present must not import forbidden"),
    );
    result.expect("an arm-declared module with no file must not fail an unrelated boundary");
    assert_eq!(
        violations.len(),
        1,
        "the unrelated boundary must still observe its own real violation: {violations:?}"
    );
}

/// The control for the test above: tolerating the fileless sibling arm must not stop the arm whose
/// file DOES exist from being reached and governed. Without this, the tolerance could pass by
/// dropping both arm modules from the graph.
#[test]
pub(super) fn an_arm_declared_module_whose_file_exists_is_still_governed() {
    let (result, violations) = run_module_check(
        "present-plain-cfg-if-arm",
        &[
            (
                "lib.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub mod unix_impl;\n\
                 } else {\n\
                 pub mod windows_impl;\n\
                 }\n\
                 }\n",
            ),
            ("unix_impl.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::unix_impl")
            .must_not_import("crate::forbidden")
            .because("the present arm's module is still observed"),
    );
    result.expect("the present arm's module must resolve");
    assert_eq!(
        violations.len(),
        1,
        "the arm module whose file exists must still be governed: {violations:?}"
    );
}

/// Arm membership makes an ABSENCE tolerable; it never makes two present files resolvable. The
/// ambiguity test runs ahead of the tolerance, so an arm-declared module backed by both conventional
/// forms is still a constitution error — the ordering this pins is the one a later "simplification"
/// would be most likely to collapse.
#[test]
pub(super) fn a_dual_backed_module_declared_inside_a_cfg_if_arm_is_still_a_scan_error() {
    let (result, _violations) = run_module_check(
        "dual-backed-cfg-if-arm",
        &[
            (
                "lib.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(unix)] {\n\
                 pub mod child;\n\
                 }\n\
                 }\n",
            ),
            ("child.rs", "// flat form\n"),
            ("child/mod.rs", "// nested form\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("an arm-declared dual-backed module is still unresolvable"),
    );
    let err = result.expect_err("both conventional forms present is never tolerated");
    assert!(
        err.contains("resolves to both"),
        "the ambiguity must be reported, not tolerated as a cfg-conditional absence: {err}"
    );
}

/// The `cfg_attr` half of the cfg-conditional rule, which nothing in 圭表 previously pinned even
/// though the requirement asserts it: `cfg_attr` never REMOVES the item, it only conditionally applies
/// its wrapped attribute, so a missing file beneath it is a genuine compile error (E0583) on every
/// configuration and must not be tolerated. Without this test, an `attr_prefix_has_bare_cfg` that
/// accidentally matched `cfg_attr` would turn a real build failure into a silent skip.
#[test]
pub(super) fn a_cfg_attr_decorated_missing_module_file_is_not_tolerated() {
    let (result, _violations) = run_module_check(
        "missing-plain-cfg-attr",
        &[(
            "lib.rs",
            "#[cfg_attr(unix, allow(dead_code))]\npub mod child;\n",
        )],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("a cfg_attr-decorated missing file is not cfg-conditional"),
    );
    let err = result.expect_err("cfg_attr must not grant the absent-file tolerance");
    assert!(
        err.contains("could not be located"),
        "the absence must be reported, not tolerated: {err}"
    );
}

/// A bare `#[cfg]` makes an ABSENCE tolerable, never two present files resolvable — the ambiguity test
/// runs first. The requirement asserts this for the attribute form as well as the arm form; only the
/// arm form was pinned.
#[test]
pub(super) fn a_cfg_gated_dual_backed_module_is_still_a_scan_error() {
    let (result, _violations) = run_module_check(
        "dual-backed-cfg-gated",
        &[
            ("lib.rs", "#[cfg(feature = \"never\")]\npub mod child;\n"),
            ("child.rs", "// flat form\n"),
            ("child/mod.rs", "// nested form\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("a cfg-gated dual-backed module is still unresolvable"),
    );
    let err = result.expect_err("both conventional forms present is never tolerated");
    assert!(
        err.contains("resolves to both"),
        "the ambiguity must be reported, not tolerated as a cfg-conditional absence: {err}"
    );
}

/// The second absence outcome the same flag governs: an unconditional `#[path]` whose target is
/// missing. Declared inside a `cfg_if!` arm, rustc strips the whole item — `#[path]` included — so
/// this is tolerated exactly as the bare-`#[cfg]`-plus-`#[path]` shim already is. One flag, so the
/// two outcomes cannot drift apart.
#[test]
pub(super) fn a_missing_path_remap_target_declared_inside_a_cfg_if_arm_is_tolerated() {
    let (result, violations) = run_module_check(
        "missing-path-cfg-if-arm",
        &[
            (
                "lib.rs",
                "cfg_if::cfg_if! {\n\
                 if #[cfg(windows)] {\n\
                 #[path = \"windows_impl.rs\"]\n\
                 pub mod imp;\n\
                 }\n\
                 }\n\
                 pub mod present;\n",
            ),
            ("present.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::present")
            .must_not_import("crate::forbidden")
            .because("present must not import forbidden"),
    );
    result.expect("an arm-declared #[path] with a missing target must not fail the scan");
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
pub(super) fn a_boundary_anchored_directly_at_a_cfg_gated_missing_module_is_unknown_not_clean() {
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
pub(super) fn an_inline_arm_paired_with_a_tolerated_away_plain_arm_still_reports_the_inline_error()
{
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
pub(super) fn a_cfg_gated_unconditional_path_target_does_not_fail_an_unrelated_boundary_when_missing()
 {
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
pub(super) fn a_cfg_gated_unconditional_path_target_is_tolerated_regardless_of_attribute_order() {
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
pub(super) fn a_cfg_attr_wrapped_path_is_recognized_as_a_remap() {
    let (result, violations) = run_module_check(
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
    // Under union-scan semantics, crate::foo performs a reachability walk across all candidate files
    // that physically exist on disk (both foo.rs and weird.rs), observing the forbidden import in foo.rs.
    assert!(
        result.is_ok(),
        "a cfg_attr-remapped module performs union-scan over physically existing target files"
    );
    assert_eq!(violations.len(), 1);
    assert_eq!(violations[0].finding, "crate::forbidden::Y");
}

pub(super) fn restrict_kernel_to_types(governed: &str, allowed: &[&str]) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module(governed)
        .restrict_imports_to(allowed.to_vec())
        .because("the kernel may import only the allowed modules")
}

#[test]
pub(super) fn restrict_imports_to_flags_an_import_outside_the_allowlist() {
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
    assert_eq!(violations[0].target(), "crate::kernel");
    assert_eq!(violations[0].finding, "crate::io::Sink");
}

#[test]
pub(super) fn a_module_violation_carries_its_offending_file() {
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
pub(super) fn a_module_backed_by_two_files_yields_one_violation_with_a_file() {
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
pub(super) fn restrict_imports_to_is_clean_within_the_allowlist() {
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
pub(super) fn restrict_imports_to_allows_the_governed_modules_own_subtree() {
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
pub(super) fn restrict_imports_to_with_an_empty_allowlist_forbids_outward_imports() {
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
pub(super) fn restrict_imports_to_does_not_treat_a_prefix_colliding_sibling_as_allowed() {
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
pub(super) fn restrict_imports_to_never_flags_an_external_import() {
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
pub(super) fn restrict_imports_to_governs_a_super_reaching_outward_import() {
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
pub(super) fn restrict_imports_to_canonicalizes_a_raw_identifier_allowlist_entry() {
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
pub(super) fn restrict_imports_to_on_the_crate_root_is_a_constitution_error() {
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
pub(super) fn restrict_imports_to_honors_warn_severity_and_its_distinct_label() {
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

pub(super) fn protect_internal_from(importer: &str) -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::internal")
        .must_not_be_imported_by(importer)
        .because("internal is private to its layer")
}

#[test]
pub(super) fn must_not_be_imported_by_flags_the_forbidden_importer_only() {
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
    assert_eq!(violations[0].target(), "crate::internal");
    assert_eq!(violations[0].finding, "crate::http");
    assert_eq!(violations[0].rule, "module must not be imported by");
}

#[test]
pub(super) fn must_not_be_imported_by_flags_ancestor_glob_import() {
    let (result, violations) = run_module_check(
        "inbound-ancestor-glob",
        &[
            ("lib.rs", "pub mod internal;\npub mod http;\n"),
            ("internal.rs", "// protected\n"),
            ("http.rs", "use crate::*;\n"),
        ],
        protect_internal_from("crate::http"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::internal");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_flags_an_inline_module_importer() {
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
    assert_eq!(violations[0].target(), "crate::internal");
    assert_eq!(violations[0].finding, "crate::http");
}

#[test]
pub(super) fn must_not_be_imported_by_applies_beneath_the_importer() {
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
pub(super) fn must_not_be_imported_by_applies_beneath_the_protected_module() {
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
pub(super) fn must_not_be_imported_by_ignores_prefix_colliding_siblings_on_both_sides() {
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
pub(super) fn must_not_be_imported_by_does_not_flag_the_protected_modules_own_subtree() {
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
