use super::helpers::*;
// --- inline-symbol-path confinement (`must_not_call_inline`) ----------------------------

pub(super) fn confine_core_clock() -> ModuleBoundary {
    ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_call_inline("std::time")
        .because("core reads no wall clock — time is injected, not read")
}

#[test]
pub(super) fn inline_default_reacts_on_an_associated_fn_call() {
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
    assert_eq!(violations[0].target(), "std::time");
    assert!(
        violations[0].finding.contains("std::time::SystemTime::now"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_default_passes_a_type_annotation() {
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
pub(super) fn inline_default_passes_a_constant() {
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
pub(super) fn inline_resolves_a_rename() {
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
pub(super) fn inline_resolves_a_self_prefixed_group_alias() {
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
pub(super) fn inline_resolves_a_bare_path() {
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
pub(super) fn inline_resolves_a_type_alias() {
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
pub(super) fn inline_resolves_a_multi_hop_type_alias() {
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
pub(super) fn inline_resolves_a_type_alias_past_a_defaulted_generic_param() {
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
pub(super) fn inline_resolves_a_cross_module_local_reexport() {
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
pub(super) fn inline_does_not_match_an_unresolved_same_named_local() {
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
pub(super) fn inline_glob_of_the_prefix_reacts() {
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
pub(super) fn inline_glob_above_the_prefix_reacts() {
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
pub(super) fn inline_glob_of_a_local_reexporting_module_reacts() {
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
pub(super) fn inline_glob_of_a_module_that_itself_globs_the_prefix_reacts() {
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
pub(super) fn inline_narrowing_drops_a_benign_constructor_and_keeps_the_read() {
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
pub(super) fn inline_narrowing_does_not_suppress_a_glob() {
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
pub(super) fn inline_strict_flags_a_type_annotation() {
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
pub(super) fn inline_value_capture_is_a_bound_under_the_default() {
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
pub(super) fn inline_scans_a_macro_body() {
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
pub(super) fn inline_empty_prefix_is_a_constitution_error() {
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
pub(super) fn inline_narrow_and_strict_is_a_constitution_error() {
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
pub(super) fn inline_valid_zero_match_is_clean() {
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
pub(super) fn confine_chrono_strict(
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
pub(super) fn inline_strict_external_reacts_on_a_fully_qualified_external_call() {
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
    assert_eq!(violations[0].target(), "chrono::Utc");
    assert!(
        violations[0].finding.contains("chrono::Utc::now"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_strict_external_absent_fully_qualified_call_is_a_bound() {
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
pub(super) fn inline_strict_external_deep_local_module_stays_clean() {
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
pub(super) fn inline_strict_external_local_fn_definition_stays_clean() {
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
pub(super) fn inline_strict_external_local_alias_stays_clean() {
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
pub(super) fn inline_strict_external_glob_reacts_and_default_glob_does_not() {
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
pub(super) fn inline_strict_external_composes_with_narrowing() {
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
pub(super) fn inline_strict_external_extern_crate_rename_is_a_stated_bound() {
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
pub(super) fn inline_strict_external_adds_nothing_to_paths_that_already_react() {
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
pub(super) fn inline_strict_external_preserves_identity_no_baseline_churn() {
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
    assert_eq!(plain[0].target(), flagged[0].target(), "target parity");
    assert_eq!(plain[0].rule, flagged[0].rule, "rule (label) parity");
    assert_eq!(plain[0].finding, flagged[0].finding, "finding parity");
    assert_eq!(
        plain[0].rule, "inline symbol path confined to module",
        "the presentation label is unchanged by the flag"
    );
}

#[test]
pub(super) fn inline_strict_external_runs_the_exit_2_checks() {
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
pub(super) fn inline_strict_external_cross_module_local_item_does_not_mask() {
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
    assert_eq!(violations[0].target(), "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_strict_external_block_local_item_does_not_mask() {
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
    assert_eq!(violations[0].target(), "log");
    assert!(
        violations[0].finding.contains("log::logger"),
        "{violations:?}"
    );
}

// --- inline-symbol-path: adversarial-review regression + coverage ------------------------

#[test]
pub(super) fn inline_reacts_on_a_leading_colon_path() {
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
pub(super) fn inline_reacts_on_a_nested_grouped_glob() {
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
pub(super) fn inline_reacts_on_a_two_hop_use_realias() {
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
pub(super) fn inline_glob_nested_past_the_depth_cap_is_a_scan_error_not_a_silent_drop() {
    // A pathologically brace-nested grouped glob must not silently vanish from the glob-hazard
    // observation past `glob_bases`'s depth cap — a real, compilable glob nested that deep would
    // otherwise pass unobserved with no report, the false negative PROJECT.md's core contract
    // forbids. Past the cap, this must be a scan error, never a silent truncation (mirrors
    // `use_scan.rs`'s identical fix for the same shape of walker).
    let depth = 200;
    let source = format!(
        "use std::{{{}time::*{}}};",
        "a::{".repeat(depth),
        "}".repeat(depth)
    );
    let (result, _violations) = run_module_check(
        "inline-glob-depth-cap",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", &source)],
        confine_core_clock(),
    );
    let err = result.expect_err(
        "a grouped glob nested past the depth cap must be a scan error, not a silent drop",
    );
    assert!(
        err.contains("brace levels"),
        "the error must name the depth bound it could not judge past: {err}"
    );
}

#[test]
pub(super) fn inline_alias_chain_nested_past_the_depth_cap_is_a_scan_error_not_a_silent_drop() {
    // The identical false-negative shape as the glob test above, for `expand_use_leaves`'s inner
    // `go`: a pathologically nested grouped `use` introducing an alias must not silently drop the
    // alias from the use-map past the depth cap — an inline call through that alias would
    // otherwise pass unresolved (never even reaching the confinement check).
    let depth = 200;
    let source = format!(
        "use std::{{{}time::SystemTime as Clock{}}};\nfn f() {{ let _ = Clock::now(); }}\n",
        "a::{".repeat(depth),
        "}".repeat(depth)
    );
    let (result, _violations) = run_module_check(
        "inline-alias-depth-cap",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", &source)],
        confine_core_clock(),
    );
    let err = result
        .expect_err("an alias nested past the depth cap must be a scan error, not a silent drop");
    assert!(
        err.contains("brace levels"),
        "the error must name the depth bound it could not judge past: {err}"
    );
}

#[test]
pub(super) fn inline_grouped_glob_nested_moderately_is_still_observed() {
    // Control: nesting comfortably under the depth cap is unaffected — the fix must not narrow
    // ordinary observation of a real, deeply (but not pathologically) nested grouped glob.
    // Braces-only nesting (no extra path segment per layer), so the glob still resolves to
    // exactly `std::time::*` regardless of depth — the pathological tests above only need SOME
    // resolvable-or-not path, but this control must still name the real confined prefix.
    let depth = 40;
    let source = format!(
        "use std::{}time::*{};",
        "{".repeat(depth),
        "}".repeat(depth)
    );
    let (result, violations) = run_module_check(
        "inline-glob-under-cap",
        &[("lib.rs", "pub mod core;\n"), ("core.rs", &source)],
        confine_core_clock(),
    );
    assert!(result.is_ok(), "{result:?}");
    assert!(
        violations.iter().any(|v| v.finding.contains("glob")),
        "a moderately nested grouped glob of the prefix still reacts: {violations:?}"
    );
}

#[test]
pub(super) fn inline_reacts_through_a_mid_path_turbofish() {
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
pub(super) fn inline_reacts_through_interior_whitespace_and_field_colon() {
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
pub(super) fn inline_ufcs_is_a_documented_bound_under_the_default() {
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
pub(super) fn inline_receiver_method_read_is_a_bound() {
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
pub(super) fn inline_grouped_self_glob_reacts() {
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
pub(super) fn inline_two_distinct_calls_stay_distinct() {
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
pub(super) fn inline_prefix_is_carried_in_the_violation_target() {
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
    assert_eq!(v1[0].target(), "std::time");
    assert_eq!(v2[0].target(), "std");
    assert_ne!(
        v1[0].target(),
        v2[0].target(),
        "distinct prefixes → distinct identity"
    );
}

#[test]
pub(super) fn inline_warn_severity_is_advisory() {
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
pub(super) fn inline_empty_verbs_is_a_constitution_error() {
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
pub(super) fn inline_scanner_does_not_panic_or_hang_on_odd_input() {
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
pub(super) fn inline_in_macro_body_alias_is_a_bound() {
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
pub(super) fn inline_strict_external_inline_submodule_call_is_not_masked() {
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
    assert_eq!(violations[0].target(), "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_strict_external_submodule_local_item_stays_clean() {
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
pub(super) fn inline_strict_external_deeply_nested_submodule_reacts() {
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
    assert_eq!(violations[0].target(), "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_strict_external_cfg_gated_submodule_reacts() {
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
    assert_eq!(violations[0].target(), "rand");
    assert!(
        violations[0].finding.contains("rand::random"),
        "{violations:?}"
    );
}

#[test]
pub(super) fn inline_strict_external_default_path_module_attribution_unshifted() {
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

#[test]
pub(super) fn scan_depth_shallow_vs_subtree_evaluates_submodule_matching() {
    let files = &[
        ("lib.rs", "pub mod core;\n"),
        ("core.rs", "pub mod sub;\nuse crate::forbidden_on_core;\n"),
        ("core/sub.rs", "use crate::forbidden_on_sub;\n"),
    ];

    // 1. Shallow depth: boundary on crate::core only governs core.rs, so import in core/sub.rs does NOT react
    let shallow_boundary = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_import("crate::forbidden_on_sub")
        .depth(xuanji::ScanDepth::Shallow)
        .because("core seam does not import forbidden_on_sub");
    let (res1, shallow_violations) =
        run_module_check("scan-depth-shallow", files, shallow_boundary);
    assert!(res1.is_ok(), "{res1:?}");
    assert_eq!(
        shallow_violations.len(),
        0,
        "Shallow depth ignores submodule files"
    );

    // 2. Subtree depth: boundary on crate::core governs core.rs AND core/sub.rs, so import in core/sub.rs DOES react
    let subtree_boundary = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_import("crate::forbidden_on_sub")
        .depth(xuanji::ScanDepth::Subtree)
        .because("core subtree does not import forbidden_on_sub");
    let (res2, subtree_violations) =
        run_module_check("scan-depth-subtree", files, subtree_boundary);
    assert!(res2.is_ok(), "{res2:?}");
    assert_eq!(
        subtree_violations.len(),
        1,
        "Subtree depth evaluates submodule files"
    );
    assert_eq!(
        subtree_violations[0]
            .rule_key()
            .fields()
            .collect::<Vec<_>>(),
        vec![("module", "crate::forbidden_on_sub")]
    );
}

#[test]
pub(super) fn module_boundary_including_submodules_is_a_compatible_subtree_modifier() {
    let default = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_import("crate::adapter")
        .because("r");
    let explicit = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_import("crate::adapter")
        .depth(xuanji::ScanDepth::Subtree)
        .because("r");
    let ergonomic = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_import("crate::adapter")
        .depth(xuanji::ScanDepth::Shallow)
        .including_submodules()
        .because("r");

    assert_eq!(default.scan_depth(), xuanji::ScanDepth::Subtree);
    assert_eq!(explicit.scan_depth(), xuanji::ScanDepth::Subtree);
    assert_eq!(ergonomic.scan_depth(), xuanji::ScanDepth::Subtree);
    assert_eq!(default.rule_key(), ergonomic.rule_key());
}

#[test]
pub(super) fn shallow_restrict_imports_to_ignores_descendant_imports() {
    let files = &[
        ("lib.rs", "pub mod core;\n"),
        ("core.rs", "pub mod detail;\n"),
        ("core/detail.rs", "use crate::adapter;\n"),
    ];
    let shallow = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .restrict_imports_to(["crate::core"])
        .depth(xuanji::ScanDepth::Shallow)
        .because("the core seam depends inward");
    let (shallow_result, shallow_violations) =
        run_module_check("shallow-restrict-imports", files, shallow);
    assert!(shallow_result.is_ok(), "{shallow_result:?}");
    assert!(shallow_violations.is_empty(), "{shallow_violations:?}");

    let subtree = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .restrict_imports_to(["crate::core"])
        .because("the core subtree depends inward");
    let (subtree_result, subtree_violations) =
        run_module_check("subtree-restrict-imports", files, subtree);
    assert!(subtree_result.is_ok(), "{subtree_result:?}");
    assert_eq!(subtree_violations.len(), 1, "{subtree_violations:?}");
}

#[test]
pub(super) fn shallow_inbound_rules_protect_only_the_exact_module() {
    let files = &[
        ("lib.rs", "pub mod protected;\npub mod client;\n"),
        ("protected.rs", "pub mod detail;\n"),
        ("protected/detail.rs", "pub struct Item;\n"),
        ("client.rs", "use crate::protected::detail::Item;\n"),
    ];

    let shallow_forbid = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_not_be_imported_by("crate::client")
        .depth(xuanji::ScanDepth::Shallow)
        .because("only the protected seam rejects this importer");
    let (_, shallow_forbid_violations) =
        run_module_check("shallow-inbound-forbid", files, shallow_forbid);
    assert!(
        shallow_forbid_violations.is_empty(),
        "{shallow_forbid_violations:?}"
    );

    let subtree_forbid = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_not_be_imported_by("crate::client")
        .because("the protected subtree rejects this importer");
    let (_, subtree_forbid_violations) =
        run_module_check("subtree-inbound-forbid", files, subtree_forbid);
    assert_eq!(
        subtree_forbid_violations.len(),
        1,
        "{subtree_forbid_violations:?}"
    );

    let shallow_allow = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_only_be_imported_by(["crate::facade"])
        .depth(xuanji::ScanDepth::Shallow)
        .because("only the protected seam has a closed importer set");
    let (_, shallow_allow_violations) =
        run_module_check("shallow-inbound-allow", files, shallow_allow);
    assert!(
        shallow_allow_violations.is_empty(),
        "{shallow_allow_violations:?}"
    );

    let subtree_allow = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_only_be_imported_by(["crate::facade"])
        .because("the protected subtree has a closed importer set");
    let (_, subtree_allow_violations) =
        run_module_check("subtree-inbound-allow", files, subtree_allow);
    assert_eq!(
        subtree_allow_violations.len(),
        1,
        "{subtree_allow_violations:?}"
    );
}

#[test]
pub(super) fn shallow_inbound_rules_react_to_an_item_import_of_the_anchored_module() {
    // An item-form import (`use crate::protected::Secret;`, an item declared directly in the
    // anchored module) must still react under Shallow — unlike an import of a descendant
    // module's item (already covered by `shallow_inbound_rules_protect_only_the_exact_module`),
    // this import's target module IS the anchored module itself, so narrowing to Shallow must
    // not exempt it. A lexical string comparison of the full import path against the anchored
    // module conflates the two cases; this regression pins the anchored one distinctly.
    let files = &[
        ("lib.rs", "pub mod protected;\npub mod client;\n"),
        ("protected.rs", "pub struct Secret;\n"),
        ("client.rs", "use crate::protected::Secret;\n"),
    ];

    let shallow_forbid = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_not_be_imported_by("crate::client")
        .depth(xuanji::ScanDepth::Shallow)
        .because("an item import of the anchored module still reacts under Shallow");
    let (forbid_result, forbid_violations) =
        run_module_check("shallow-inbound-item-forbid", files, shallow_forbid);
    assert!(forbid_result.is_ok(), "{forbid_result:?}");
    assert_eq!(
        forbid_violations.len(),
        1,
        "an item-form import of the anchored module must react even under Shallow: {forbid_violations:?}"
    );

    let shallow_allow = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_only_be_imported_by(["crate::facade"])
        .depth(xuanji::ScanDepth::Shallow)
        .because("an item import of the anchored module still reacts under Shallow");
    let (allow_result, allow_violations) =
        run_module_check("shallow-inbound-item-allow", files, shallow_allow);
    assert!(allow_result.is_ok(), "{allow_result:?}");
    assert_eq!(
        allow_violations.len(),
        1,
        "an item-form import by an importer outside the allowlist must react even under Shallow: {allow_violations:?}"
    );
}

#[test]
pub(super) fn shallow_inbound_rules_never_flag_the_protected_modules_own_descendant_as_an_importer()
{
    // The self-import exemption ("a file within the protected module's own subtree is never an
    // inbound importer") is unconditional in the spec, not depth-gated — narrowing to Shallow
    // scopes what counts as *reaching* the protected module, never who counts as *inside* it.
    // A descendant submodule of the protected module importing an item declared directly in the
    // protected module itself must stay exempt even under Shallow, exactly as it already is under
    // Subtree — otherwise fixing the target-match precision (the item-import false negative)
    // would introduce a false positive here instead.
    let files = &[
        ("lib.rs", "pub mod protected;\n"),
        ("protected.rs", "pub mod detail;\npub struct Secret;\n"),
        ("protected/detail.rs", "use crate::protected::Secret;\n"),
    ];

    let shallow_allow = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_only_be_imported_by(["crate::facade"])
        .depth(xuanji::ScanDepth::Shallow)
        .because("the protected module's own descendant is never an inbound importer");
    let (allow_result, allow_violations) = run_module_check(
        "shallow-inbound-self-descendant-allow",
        files,
        shallow_allow,
    );
    assert!(allow_result.is_ok(), "{allow_result:?}");
    assert!(
        allow_violations.is_empty(),
        "a descendant of the protected module importing the protected module's own item must \
         never be flagged, even under Shallow: {allow_violations:?}"
    );

    let shallow_forbid = ModuleBoundary::in_crate("x")
        .module("crate::protected")
        .must_not_be_imported_by("crate::protected::detail")
        .depth(xuanji::ScanDepth::Shallow)
        .because("the protected module's own descendant is never an inbound importer");
    let (forbid_result, forbid_violations) = run_module_check(
        "shallow-inbound-self-descendant-forbid",
        files,
        shallow_forbid,
    );
    assert!(forbid_result.is_ok(), "{forbid_result:?}");
    assert!(
        forbid_violations.is_empty(),
        "a descendant of the protected module is never an inbound importer, even when it is also \
         (degenerately) named as the forbidden importer: {forbid_violations:?}"
    );
}

#[test]
pub(super) fn shallow_inbound_rules_do_not_read_a_file_the_self_import_exemption_excuses() {
    // The exemption above is observable in the violation output at either depth, because the
    // per-import check excuses these importers even when the file IS read. What was NOT the same at
    // both depths was whether the file gets read at all: a depth-gated fast path over a depth-free
    // exemption skipped the read only under `Subtree`, so at `Shallow` a file inside the protected
    // subtree was read and scanned — and any read that can fail then diverges. A `use` tree nested
    // past the scanner's brace-nesting cap is such a failure (fail-loud by design, exit 2), so it
    // pins the divergence: the identical protected-subtree file must be excused, not judged, at both
    // depths. Anything the exemption excuses must not be able to decide the exit code.
    let deep_use = format!(
        "use crate::protected::{}Secret{};\n",
        "{".repeat(200),
        "}".repeat(200)
    );
    let files = &[
        ("lib.rs", "pub mod protected;\n"),
        ("protected.rs", "pub mod detail;\npub struct Secret;\n"),
        ("protected/detail.rs", deep_use.as_str()),
    ];

    for depth in [xuanji::ScanDepth::Shallow, xuanji::ScanDepth::Subtree] {
        let boundary = ModuleBoundary::in_crate("x")
            .module("crate::protected")
            .must_only_be_imported_by(["crate::facade"])
            .depth(depth)
            .because("the protected module's own descendant is never an inbound importer");
        let (result, violations) =
            run_module_check("shallow-inbound-self-descendant-unread", files, boundary);
        assert!(
            result.is_ok(),
            "a file the self-import exemption excuses must never be read, so its content cannot \
             produce a scan error at {depth:?}: {result:?}"
        );
        assert!(violations.is_empty(), "{depth:?}: {violations:?}");
    }
}

#[test]
pub(super) fn shallow_external_confinement_permits_only_the_exact_module() {
    let files = &[
        ("lib.rs", "pub mod secret;\n"),
        ("secret.rs", "pub mod detail;\n"),
        ("secret/detail.rs", "use libc::c_int;\n"),
    ];
    let shallow = ModuleBoundary::in_crate("x")
        .module("crate::secret")
        .confine_external_crate("libc")
        .depth(xuanji::ScanDepth::Shallow)
        .because("only the secret seam may import libc");
    let (shallow_result, shallow_violations) =
        run_module_check("shallow-external-confinement", files, shallow);
    assert!(shallow_result.is_ok(), "{shallow_result:?}");
    assert_eq!(shallow_violations.len(), 1, "{shallow_violations:?}");

    let subtree = ModuleBoundary::in_crate("x")
        .module("crate::secret")
        .confine_external_crate("libc")
        .because("the secret subtree may import libc");
    let (subtree_result, subtree_violations) =
        run_module_check("subtree-external-confinement", files, subtree);
    assert!(subtree_result.is_ok(), "{subtree_result:?}");
    assert!(subtree_violations.is_empty(), "{subtree_violations:?}");
}

#[test]
pub(super) fn scan_depth_projection_omits_legacy_subtree_and_emits_shallow() {
    let legacy = Constitution::new("legacy").boundary(
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_import("crate::adapter")
            .because("core depends inward only"),
    );
    let legacy_json: serde_json::Value = serde_json::from_str(&constitution_json(&legacy)).unwrap();
    assert!(
        legacy_json["boundaries"][0].get("scan_depth").is_none(),
        "legacy subtree projection must stay byte-compatible: {legacy_json}"
    );

    let shallow = Constitution::new("shallow").boundary(
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_import("crate::adapter")
            .depth(xuanji::ScanDepth::Shallow)
            .because("only the core seam depends inward"),
    );
    let shallow_json: serde_json::Value =
        serde_json::from_str(&constitution_json(&shallow)).unwrap();
    assert_eq!(shallow_json["boundaries"][0]["scan_depth"], "shallow");
}

#[test]
pub(super) fn legacy_inline_confinement_defaults_to_subtree_and_preserves_identity() {
    let files = &[
        ("lib.rs", "pub mod core;\n"),
        ("core.rs", "pub mod sub;\n"),
        (
            "core/sub.rs",
            "fn t() { let _ = std::time::SystemTime::now(); }\n",
        ),
    ];

    // 1. Legacy inline boundary without explicit depth -> defaults to Subtree -> catches call in core/sub.rs
    let legacy_boundary = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_call_inline("std::time")
        .because("core reads no wall clock");
    let (res, violations) = run_module_check("legacy-inline-subtree", files, legacy_boundary);
    assert!(res.is_ok(), "{res:?}");
    assert_eq!(
        violations.len(),
        1,
        "Legacy inline boundary governs submodules"
    );
    // Verify legacy RuleKey identity has NO "scan_depth" field (zero baseline breakage)
    assert_eq!(
        violations[0].rule_key().fields().collect::<Vec<_>>(),
        vec![
            ("ending_with", "[]"),
            ("prefix", "std::time"),
            ("strict", "false")
        ]
    );

    // 2. Explicit Shallow depth -> restricts observation seam to core.rs -> ignores call in core/sub.rs
    let shallow_boundary = ModuleBoundary::in_crate("x")
        .module("crate::core")
        .must_not_call_inline("std::time")
        .depth(xuanji::ScanDepth::Shallow)
        .because("core seam reads no wall clock");
    let (res2, shallow_violations) =
        run_module_check("shallow-inline-seam", files, shallow_boundary);
    assert!(res2.is_ok(), "{res2:?}");
    assert_eq!(
        shallow_violations.len(),
        0,
        "Explicit Shallow depth ignores submodule calls"
    );
}

/// Two-crate reproduction of the audit-sweep finding: identical governed module path + rule
/// declared against two different workspace members must stay two distinct violations, never
/// dedup into one and never let one crate's baseline suppress the other's unaccepted violation.
/// Mirrors the exact shape `crates/tianheng/tests/self_governance.rs` declares on itself
/// (the identical rule on guibiao/hunyi/louke).
#[test]
pub(super) fn two_crates_with_the_identical_module_boundary_stay_distinct_violations() {
    let alpha = TempWorkspace::new("identity-scope-alpha");
    alpha.write("lib.rs", "pub mod app;\npub mod secret;\n");
    alpha.write("app.rs", "use crate::secret::S;\n");
    alpha.write("secret.rs", "pub struct S;\n");

    let beta = TempWorkspace::new("identity-scope-beta");
    beta.write("lib.rs", "pub mod app;\npub mod secret;\n");
    beta.write("app.rs", "use crate::secret::S;\n");
    beta.write("secret.rs", "pub struct S;\n");

    fn package_json(ws: &TempWorkspace, name: &str) -> serde_json::Value {
        let manifest = ws.dir().join("Cargo.toml");
        serde_json::json!({
            "name": name,
            "manifest_path": manifest.to_string_lossy().into_owned(),
            "dependencies": [],
        })
    }
    let metadata = serde_json::json!({
        "packages": [package_json(&alpha, "alpha"), package_json(&beta, "beta")]
    });

    fn boundary_for(package: &str) -> ModuleBoundary {
        ModuleBoundary::in_crate(package)
            .module("crate::app")
            .must_not_import("crate::secret")
            .because("app must not touch secret")
    }

    // Both crates violating, evaluated together: must yield two distinct violations, not one.
    let constitution = Constitution::new("probe")
        .boundary(boundary_for("alpha"))
        .boundary(boundary_for("beta"));
    let outcome = evaluate(&constitution, &metadata);
    let report = match outcome {
        Outcome::Violations(report) => report,
        other => panic!("expected two violations, got {other:?}"),
    };
    assert_eq!(
        report.violations.len(),
        2,
        "each crate's violation must survive dedup: {:?}",
        report.violations
    );
    let ids: std::collections::BTreeSet<_> = report.violations.iter().map(Violation::id).collect();
    assert_eq!(
        ids.len(),
        2,
        "identity must differ by crate, not collapse to one"
    );
    let files: std::collections::BTreeSet<_> = report
        .violations
        .iter()
        .map(|v| v.file.clone().expect("each violation carries a file"))
        .collect();
    assert_eq!(
        files.len(),
        2,
        "each violation must keep its own crate's file, not share one"
    );

    // Baseline written against alpha alone must not suppress beta's unaccepted violation.
    let alpha_only_constitution = Constitution::new("probe").boundary(boundary_for("alpha"));
    let alpha_only_report = match evaluate(&alpha_only_constitution, &metadata) {
        Outcome::Violations(report) => report,
        other => panic!("expected alpha's violation, got {other:?}"),
    };
    assert_eq!(alpha_only_report.violations.len(), 1);
    let baseline = Baseline::of(&alpha_only_report);

    let both_constitution = Constitution::new("probe")
        .boundary(boundary_for("alpha"))
        .boundary(boundary_for("beta"));
    let mut both_report = match evaluate(&both_constitution, &metadata) {
        Outcome::Violations(report) => report,
        other => panic!("expected two violations, got {other:?}"),
    };
    apply_baseline(&mut both_report, &baseline);
    let alpha_violation = both_report
        .violations
        .iter()
        .find(|v| v.target() == "crate::app" && baseline.contains(v))
        .expect("alpha's violation must match the baseline");
    assert!(alpha_violation.baselined, "alpha's violation was accepted");
    let unbaselined: Vec<_> = both_report
        .violations
        .iter()
        .filter(|v| !v.baselined)
        .collect();
    assert_eq!(
        unbaselined.len(),
        1,
        "beta's violation must react as new, not be suppressed by alpha's baseline: {:?}",
        both_report.violations
    );
}

/// A source file ending in an unterminated block comment (no closing `*/`, no trailing newline)
/// that swallows a multi-byte UTF-8 character must react 0/1/2 like any other source, never
/// panic. The trigger's exact shape matters: `strip_comments_and_strings_tracked`'s block-comment
/// loop stops peeking once fewer than two bytes remain, which — for an unterminated comment — can
/// leave exactly one trailing byte unconsumed. When that byte is the orphaned tail of a multi-byte
/// character whose lead byte(s) were already dropped inside the comment, the outer loop used to
/// re-scan it as ordinary code and push it into `out` alone, an invalid UTF-8 fragment that
/// `String::from_utf8_lossy` then *lengthened* (1 byte becomes the 3-byte U+FFFD replacement),
/// desynchronizing the position map from the string it indexes into and panicking the next
/// stage's `input_positions[i]` lookup.
#[test]
pub(super) fn an_unterminated_block_comment_swallowing_a_multibyte_char_does_not_panic() {
    let (result, violations) = run_module_check(
        "unterminated-block-comment-multibyte",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\npub mod child;\n/* \u{672a}\u{5b8c}",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
            ("child.rs", "use crate::forbidden::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(
        result.is_ok(),
        "an unterminated block comment must not abort the scan: {result:?}"
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::child");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// The identical defect, triggered on the crate root file directly (governed at `crate` rather
/// than a submodule) with only a single `pub mod` before the unterminated comment, so the
/// swallowed trailing byte lands at a different absolute offset — exercising the same code path
/// from a second, independently-chosen position rather than only the sibling test's exact shape.
#[test]
pub(super) fn an_unterminated_block_comment_at_end_of_file_with_no_trailing_newline_does_not_panic()
{
    let (result, violations) = run_module_check(
        "unterminated-block-comment-eof",
        &[
            ("lib.rs", "pub mod child;\n/*\u{7121}"),
            ("child.rs", "use crate::forbidden::Thing;\n"),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::child")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(
        result.is_ok(),
        "an unterminated block comment must not abort the scan: {result:?}"
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::child");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// A non-ASCII char literal immediately adjacent to a `'{'` literal (`['«','{']`, no space) must
/// not leak `{` as a spurious structural brace into the cleaned text — which used to drop every
/// later top-level `mod` from the reachable set, so a boundary anchored above the affected module
/// silently passed a real forbidden import (exit 0 Clean on source `rustc` compiles as-is).
#[test]
pub(super) fn a_non_ascii_char_literal_adjacent_to_a_brace_literal_does_not_leak_a_spurious_brace()
{
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-no-space",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\nconst Q: [char; 2] = ['\u{ab}','{'];\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// The identical shape, but the boundary is anchored directly at the module the leak used to drop
/// (`crate::hidden`) rather than above it. Before the fix this failed loud (exit 2, "module
/// 'crate::hidden' is not found among the reachable modules") instead of silently passing — after
/// the fix the module is genuinely reachable, so the boundary resolves and reacts normally.
#[test]
pub(super) fn a_boundary_anchored_directly_at_the_previously_dropped_module_resolves() {
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-anchored-at-dropped",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\nconst Q: [char; 2] = ['\u{ab}','{'];\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::hidden")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(
        result.is_ok(),
        "crate::hidden must be reachable, not a constitution error: {result:?}"
    );
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].target(), "crate::hidden");
}

/// The identical defect in its everyday form — a `match` arm pattern, not an array literal — one
/// hop from the audit's original trigger shape, exercising the same lexer branch from different
/// surrounding syntax. The pipe must sit with **no surrounding spaces** (`'é'|'{'`, matching the
/// audit's exact citation) — a spaced pipe (`'é' | '{'`) inserts extra separator bytes between the
/// misread closing quote and the next literal's opening quote, which happens not to collide with
/// this exact defect and would silently test nothing (confirmed while writing this test: the
/// spaced form passed even with the bug still present).
#[test]
pub(super) fn a_non_ascii_char_literal_adjacent_to_a_brace_literal_in_a_match_arm_does_not_leak() {
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-match-arm",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\npub fn is_special(c: char) -> bool { match c { '\u{e9}'|'{' => true, _ => false } }\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// Control: the identical array literal but with a space inserted (`['«', '{']`) already worked
/// correctly before this fix and must keep working — locks in that the fix does not regress the
/// already-passing spelling.
#[test]
pub(super) fn the_spaced_spelling_of_the_same_array_literal_already_reacts_and_keeps_reacting() {
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-with-space",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\nconst Q: [char; 2] = ['\u{ab}', '{'];\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// A 4-byte scalar (an emoji) adjacent to `'{'` — added after an independent apply-stage review
/// constructed this case and confirmed it fails without the fix, generalizing the defect beyond
/// the 2/3-byte scalars the other tests exercise.
#[test]
pub(super) fn a_four_byte_scalar_char_literal_adjacent_to_a_brace_literal_does_not_leak() {
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-four-byte-scalar",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\nconst Q: [char; 2] = ['\u{1f980}','{'];\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}

/// Three char literals in a row, `['«','{','{']` — a cascading version of the two-literal defect:
/// the misread first literal's closing quote can coincidentally swallow *each subsequent* literal's
/// opening quote in turn (closing-quote, comma, opening-quote matches the old one-byte assumption
/// every time), so a fix that only demonstrably closes the single-literal case could still leak a
/// *second*, unmatched brace one hop further. Deliberately two unmatched `{` rather than a `{`/`}`
/// pair: a leaked *matched* pair nets to zero depth change and was verified, while constructing
/// this test, to pass even with the bug present — it doesn't actually corrupt the reachability
/// walker's brace-depth tracking, so it would have been a vacuous regression test. Two unmatched
/// opens do shift depth permanently, which is what a naive one-hop fix could still miss.
#[test]
pub(super) fn two_unmatched_braces_cascading_from_chained_char_literals_do_not_leak() {
    let (result, violations) = run_module_check(
        "char-literal-brace-leak-two-unmatched-cascade",
        &[
            (
                "lib.rs",
                "pub mod forbidden;\nconst Q: [char; 3] = ['\u{ab}','{','{'];\npub mod hidden;\n",
            ),
            (
                "hidden.rs",
                "use crate::forbidden::Thing;\npub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n",
            ),
            ("forbidden.rs", "pub struct Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("probe"),
    );
    assert!(result.is_ok(), "{result:?}");
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::forbidden::Thing");
}
