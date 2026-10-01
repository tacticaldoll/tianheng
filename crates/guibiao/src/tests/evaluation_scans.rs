use super::helpers::*;
use crate::module_scan::EvaluationScans;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;

/// Judge one constitution through both scan forms: shared (each root's scan built once — the
/// production reading) and independent (a scan rebuilt at every lookup, none reused).
fn shared_and_independent(constitution: &Constitution, metadata: &Value) -> (Outcome, Outcome) {
    (
        crate::evaluate_with_scans(constitution, metadata, &EvaluationScans::shared()),
        crate::evaluate_with_scans(constitution, metadata, &EvaluationScans::independent()),
    )
}

/// Assemble `boundaries` into a constitution, in the order given.
fn constitution_of(boundaries: Vec<ModuleBoundary>) -> Constitution {
    let mut constitution = Constitution::new("corpus");
    for boundary in boundaries {
        constitution = constitution.boundary(boundary);
    }
    constitution
}

/// `module-boundary` scenario "Many boundaries over one root build its scan once": a constitution
/// of several module boundaries over one root builds the root's scan once, and a package of two
/// compiled roots builds each of theirs once — the work counted where a scan is built, never where
/// a lookup finds one already built.
#[test]
pub(super) fn many_boundaries_over_one_root_build_its_scan_once() {
    let ws = TempWorkspace::new("root-scan-once");
    ws.write(
        "lib.rs",
        "pub mod kernel;\npub mod projection;\npub mod types;\n",
    );
    ws.write("kernel.rs", "use crate::projection::P;\npub struct K;\n");
    ws.write("projection.rs", "pub struct P;\n");
    ws.write("types.rs", "pub struct T;\n");
    let metadata = ws.metadata("x");
    let constitution = Constitution::new("t")
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::kernel")
                .must_not_import("crate::projection")
                .because("the kernel does not reach the projection"),
        )
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::kernel")
                .restrict_imports_to(vec!["crate::types"])
                .because("the kernel imports only the types"),
        )
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::projection")
                .must_not_import("crate::kernel")
                .because("the projection does not reach back"),
        )
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::kernel")
                .must_not_call_inline("std::process")
                .because("the kernel spawns no process"),
        );
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the forbidden import reacts: {outcome:?}"
    );
    assert_eq!(
        scans.roots_built(),
        1,
        "four boundaries over one root build its scan once, not once per boundary"
    );

    let ws = TempWorkspace::new("root-scan-per-root");
    ws.write("lib.rs", "pub mod kernel;\npub mod types;\n");
    ws.write("kernel.rs", "pub struct K;\n");
    ws.write("types.rs", "pub struct T;\n");
    ws.write("bin/tool.rs", "fn main() {}\n");
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "targets": [
                { "kind": ["lib"], "src_path": ws.src().join("lib.rs").to_string_lossy() },
                { "kind": ["bin"], "src_path": ws.src().join("bin/tool.rs").to_string_lossy() },
            ],
        }],
    });
    let constitution = Constitution::new("t")
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::kernel")
                .must_not_import("crate::types")
                .because("the kernel does not reach the types"),
        )
        .boundary(
            ModuleBoundary::in_crate("x")
                .module("crate::types")
                .must_not_import("crate::kernel")
                .because("the types do not reach back"),
        );
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
    assert_eq!(
        scans.roots_built(),
        2,
        "a package of a lib and a bin builds each root once across both boundaries"
    );
}

/// One corpus entry's fixture and constitution parts, built fresh per run.
type CorpusEntry = (TempWorkspace, Value, Vec<ModuleBoundary>);

/// One corpus entry: its name and the builder of its fixture and boundaries.
struct Corpus {
    name: &'static str,
    build: fn() -> CorpusEntry,
}

/// The outbound family: a forbid and a restrict over one governed module, both breached.
fn outbound() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-outbound");
    ws.write(
        "lib.rs",
        "pub mod kernel;\npub mod projection;\npub mod types;\n",
    );
    ws.write("kernel.rs", "use crate::projection::P;\npub struct K;\n");
    ws.write("projection.rs", "pub struct P;\n");
    ws.write("types.rs", "pub struct T;\n");
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::projection")
            .because("the kernel does not reach the projection"),
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .restrict_imports_to(vec!["crate::types"])
            .because("the kernel imports only the types"),
    ];
    (ws, metadata, boundaries)
}

/// The inbound family: `crate::internal` binds `foo` in both namespaces (`mod foo` and `fn foo`),
/// so the shallow rule reacts through the value-namespace reading of `use crate::internal::foo;`.
fn inbound() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-inbound");
    ws.write(
        "lib.rs",
        "pub mod internal;\npub mod api;\npub mod kernel;\n",
    );
    ws.write(
        "internal.rs",
        "pub mod foo { pub const INSIDE: u8 = 0; }\npub fn foo() {}\n",
    );
    ws.write("api.rs", "use crate::internal::foo;\npub fn a() {}\n");
    ws.write(
        "kernel.rs",
        "use crate::internal::foo as f;\npub fn k() {}\n",
    );
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_not_be_imported_by("crate::api")
            .depth(ScanDepth::Shallow)
            .because("internal is private to the crate"),
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_only_be_imported_by(vec!["crate::kernel"])
            .because("only the kernel reaches internal"),
    ];
    (ws, metadata, boundaries)
}

/// External-crate confinement beside an outbound rule over the same root, one of them clean.
fn external_confinement() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-external");
    ws.write("lib.rs", "pub mod ffi;\npub mod service;\npub mod extra;\n");
    ws.write("ffi.rs", "\n");
    ws.write("service.rs", "use libc::c_int;\n");
    ws.write("extra.rs", "use serde::Deserialize;\n");
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::ffi")
            .confine_external_crate("libc")
            .because("the raw libc surface stays behind the ffi module"),
        ModuleBoundary::in_crate("x")
            .module("crate::service")
            .must_not_import("crate::ffi")
            .because("service does not reach the ffi module"),
    ];
    (ws, metadata, boundaries)
}

/// The inline family: a call confinement, its permitting dual, a strict confinement, and a
/// strict-external confinement against a declared dependency.
fn inline() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-inline");
    ws.write("lib.rs", "pub mod core;\npub mod exec;\n");
    ws.write(
        "core.rs",
        "pub fn stamp() { let _ = std::process::id(); }\npub fn tick() { let _ = chrono::Utc::now(); }\n",
    );
    ws.write("exec.rs", "pub fn run() { std::process::exit(0); }\n");
    let metadata = ws.metadata_with_deps("x", &[("chrono", None)]);
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::process")
            .because("core spawns no process"),
        ModuleBoundary::in_crate("x")
            .module("crate::exec")
            .confine_inline_call("std::process")
            .because("only exec spawns a process"),
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::process")
            .strict_prefix_only()
            .because("core does not even name a process"),
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("chrono::Utc")
            .strict_external()
            .because("core reads no wall clock — time is injected"),
    ];
    (ws, metadata, boundaries)
}

/// A package of two compiled roots: a rule breached only in the bin root, a rule whose governed
/// module the bin root lacks, and a whole-root rule judged in the bin root with an empty permitted
/// region.
fn several_roots() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-roots");
    ws.write(
        "lib.rs",
        "pub mod kernel;\npub mod shared;\npub mod internal;\n",
    );
    ws.write("kernel.rs", "pub struct K;\n");
    ws.write("shared.rs", "use crate::kernel::K;\n");
    ws.write("internal.rs", "\n");
    ws.write("bin/tool.rs", "mod tooling;\nmod shared;\nfn main() {}\n");
    ws.write("bin/tooling.rs", "use libc::c_int;\n");
    ws.write("bin/shared.rs", "use crate::tooling;\nuse libc::c_void;\n");
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "targets": [
                { "kind": ["lib"], "src_path": ws.src().join("lib.rs").to_string_lossy() },
                { "kind": ["bin"], "src_path": ws.src().join("bin/tool.rs").to_string_lossy() },
            ],
        }],
    });
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::shared")
            .must_not_import("crate::tooling")
            .because("shared must not reach the tooling"),
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::tooling")
            .because("the kernel must not reach the tooling"),
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .confine_external_crate("libc")
            .because("the raw libc surface stays behind internal"),
    ];
    (ws, metadata, boundaries)
}

/// A constitution refused two ways: a rule's named module no root declares, and a governed module
/// no root declares.
fn refused_unknown_module() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-refused-named");
    ws.write("lib.rs", "pub mod real;\n");
    ws.write("real.rs", "\n");
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::real")
            .must_not_import("crate::io")
            .because("real does not reach the io"),
        ModuleBoundary::in_crate("x")
            .module("crate::ghost")
            .must_not_import("crate::real")
            .because("a ghost does not reach the real"),
    ];
    (ws, metadata, boundaries)
}

/// A constitution refused by an inline module as a governed target.
fn refused_inline_target() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-refused-inline");
    ws.write("lib.rs", "pub mod wrapper;\npub mod real;\n");
    ws.write("wrapper.rs", "pub mod inline {}\n");
    ws.write("real.rs", "\n");
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::wrapper::inline")
            .must_not_import("crate::real")
            .because("an inline module cannot be governed"),
        ModuleBoundary::in_crate("x")
            .module("crate::real")
            .must_not_import("crate::wrapper")
            .because("real does not reach the wrapper"),
    ];
    (ws, metadata, boundaries)
}

/// A ring of imports inside one module, read from two of its bindings: `x` and `y` each name a terminal of their own
/// under `cfg(unix)` and each other otherwise, and `z` names `y`. A reading entering at `x` reads `y` one binding deep
/// and cuts the ring where `y` comes back to `x`, so what it finds for `y` there lacks `x`'s terminal; a reading
/// entering at `z` reads `y` at that same depth, from that same module, and its answer holds both terminals. Two inline
/// boundaries enter at the two bindings, from two child modules, under different prefixes and strictness, so a cut
/// answer the first kept for `y` would hide from the second the terminal its prefix names.
fn import_ring() -> CorpusEntry {
    let ws = TempWorkspace::new("eq-ring");
    ws.write("lib.rs", "pub mod t0;\npub mod t1;\npub mod r;\n");
    ws.write("t0.rs", "pub fn f() {}\n");
    ws.write("t1.rs", "pub fn f() {}\n");
    ws.write(
        "r.rs",
        "#[cfg(unix)]\npub use crate::t0 as x;\n#[cfg(not(unix))]\npub use y as x;\n\
         #[cfg(unix)]\npub use crate::t1 as y;\n#[cfg(not(unix))]\npub use x as y;\n\
         pub use y as z;\npub mod ra;\npub mod rb;\n",
    );
    ws.write("r/ra.rs", "pub fn run() { super::x::f(); }\n");
    ws.write("r/rb.rs", "pub fn run() { super::z::f(); }\n");
    let metadata = ws.metadata("x");
    let boundaries = vec![
        ModuleBoundary::in_crate("x")
            .module("crate::r::ra")
            .must_not_call_inline("crate::t1")
            .because("ra does not reach the second terminal"),
        ModuleBoundary::in_crate("x")
            .module("crate::r::rb")
            .must_not_call_inline("crate::t0")
            .strict_prefix_only()
            .because("rb does not even name the first terminal"),
    ];
    (ws, metadata, boundaries)
}

/// `module-boundary` scenario "Shared and independent scans yield one outcome": every constitution
/// of the corpus — outbound, inbound with the value-namespace form, external confinement, the
/// inline family with strict and strict-external, a ring of imports read from two of its bindings
/// under different prefixes and strictness, a two-root package, and two refused
/// constitutions — is judged through both scan forms, in each of two boundary orders, and the two
/// readings of one order yield the same outcome: the same violations with the same identities and
/// severities, or the same refusal. Nothing is claimed across orders: the first error an
/// evaluation returns may be a different boundary's.
#[test]
pub(super) fn shared_and_independent_scans_yield_one_outcome() {
    let corpus = [
        Corpus {
            name: "outbound",
            build: outbound,
        },
        Corpus {
            name: "inbound",
            build: inbound,
        },
        Corpus {
            name: "external-confinement",
            build: external_confinement,
        },
        Corpus {
            name: "inline",
            build: inline,
        },
        Corpus {
            name: "import-ring",
            build: import_ring,
        },
        Corpus {
            name: "several-roots",
            build: several_roots,
        },
        Corpus {
            name: "refused-unknown-module",
            build: refused_unknown_module,
        },
        Corpus {
            name: "refused-inline-target",
            build: refused_inline_target,
        },
    ];
    let mut saw_violations = false;
    let mut saw_refusal = false;
    for Corpus { name, build } in corpus {
        let (_ws, metadata, boundaries) = build();
        for reversed in [false, true] {
            let mut ordered = boundaries.clone();
            if reversed {
                ordered.reverse();
            }
            let constitution = constitution_of(ordered);
            let (shared, independent) = shared_and_independent(&constitution, &metadata);
            match &shared {
                Outcome::Violations(_) => saw_violations = true,
                Outcome::ConstitutionError(_) => saw_refusal = true,
                _ => {}
            }
            assert_eq!(
                shared, independent,
                "{name} (reversed: {reversed}): a shared scan per root and a scan per boundary \
                 yield one outcome"
            );
        }
    }
    assert!(
        saw_violations,
        "the corpus judges violations — without one, the equivalence shown is vacuous"
    );
    assert!(
        saw_refusal,
        "the corpus is refused — without one, the equivalence shown is vacuous"
    );
}

/// Each path in `paths`, read once: what an evaluation that reads every source it meets once, and
/// no other, has read.
fn read_once(paths: &[PathBuf]) -> HashMap<PathBuf, usize> {
    paths.iter().map(|path| (path.clone(), 1)).collect()
}

/// `module-boundary` scenario "Each source path is read once, on demand": several boundaries over
/// one root — import rules with an inbound rule's value-namespace reading of the governed module,
/// and the inline family with a strict confinement — read each source they reach once, among them a
/// source outside the root's file list that a `#[path]` reaches; the set read is exactly the set
/// reached, held both ways.
#[test]
pub(super) fn each_source_path_is_read_once_on_demand() {
    let ws = TempWorkspace::new("read-once");
    ws.write(
        "lib.rs",
        "pub mod internal;\npub mod api;\npub mod core;\n#[path = \"../outside.rs\"]\npub mod outside;\n",
    );
    ws.write(
        "internal.rs",
        "pub mod foo { pub const INSIDE: u8 = 0; }\npub fn foo() {}\n",
    );
    ws.write("api.rs", "use crate::internal::foo;\npub fn a() {}\n");
    ws.write(
        "core.rs",
        "use crate::internal::foo as f;\npub fn stamp() { let _ = std::process::id(); }\n",
    );
    ws.write_at("outside.rs", "pub fn o() { std::process::exit(0); }\n");
    let metadata = ws.metadata("x");
    let constitution = constitution_of(vec![
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_not_be_imported_by("crate::api")
            .depth(ScanDepth::Shallow)
            .because("internal is private to the crate"),
        ModuleBoundary::in_crate("x")
            .module("crate::internal")
            .must_only_be_imported_by(vec!["crate::core"])
            .because("only core reaches internal"),
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::process")
            .because("core spawns no process"),
        ModuleBoundary::in_crate("x")
            .module("crate::core")
            .must_not_call_inline("std::process")
            .strict_prefix_only()
            .because("core does not even name a process"),
        ModuleBoundary::in_crate("x")
            .module("crate::outside")
            .must_not_call_inline("std::process")
            .because("the outside module spawns no process"),
    ]);
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    assert_eq!(outcome.exit_code(), 1, "the boundaries react: {outcome:?}");
    let src = ws.src();
    assert_eq!(
        scans.source_reads(),
        read_once(&[
            src.join("lib.rs"),
            src.join("internal.rs"),
            src.join("api.rs"),
            src.join("core.rs"),
            src.join("../outside.rs"),
        ]),
        "every source the boundaries reach is read once, by the path it was opened at, and no other is read"
    );
}

/// `module-boundary` scenario "A source two roots or two modules reach is read once": a file two
/// roots of one package compile is read once across both, and a file two `#[path]` attributes load
/// as two modules is read once for both — while every module position it holds is judged as a scan
/// per boundary, reading the file afresh, judges it.
#[test]
pub(super) fn a_source_two_roots_or_two_modules_reach_is_read_once() {
    let ws = TempWorkspace::new("read-once-two-roots");
    ws.write("lib.rs", "pub mod kernel;\npub mod common;\n");
    ws.write("main.rs", "mod kernel;\nmod common;\nfn main() {}\n");
    ws.write("kernel.rs", "pub struct K;\n");
    ws.write("common.rs", "use crate::kernel::K;\n");
    let metadata = serde_json::json!({
        "packages": [{
            "name": "x",
            "targets": [
                { "kind": ["lib"], "src_path": ws.src().join("lib.rs").to_string_lossy() },
                { "kind": ["bin"], "src_path": ws.src().join("main.rs").to_string_lossy() },
            ],
        }],
    });
    let constitution = constitution_of(vec![
        ModuleBoundary::in_crate("x")
            .module("crate::common")
            .must_not_import("crate::kernel")
            .because("common does not reach the kernel"),
    ]);
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    let src = ws.src();
    assert_eq!(
        scans.source_reads(),
        read_once(&[
            src.join("lib.rs"),
            src.join("main.rs"),
            src.join("kernel.rs"),
            src.join("common.rs"),
        ]),
        "a file both roots compile is read once across them"
    );
    let units: BTreeSet<String> = match &outcome {
        Outcome::Violations(report) => report
            .violations
            .iter()
            .flat_map(|violation| violation.fact().fields())
            .filter(|(field, _)| *field == "unit")
            .map(|(_, unit)| unit.to_string())
            .collect(),
        other => panic!("each root's common module reacts: {other:?}"),
    };
    assert_eq!(
        units,
        BTreeSet::from(["lib.rs".to_string(), "main.rs".to_string()]),
        "each root judges its own position of the shared file: {outcome:?}"
    );
    assert_eq!(
        outcome,
        crate::evaluate_with_scans(&constitution, &metadata, &EvaluationScans::independent()),
        "the shared reading judges each root as a reading per boundary does"
    );

    let ws = TempWorkspace::new("read-once-two-modules");
    ws.write(
        "lib.rs",
        "#[path = \"twin.rs\"]\npub mod a;\n#[path = \"twin.rs\"]\npub mod b;\n",
    );
    ws.write("twin.rs", "pub fn t() { std::process::exit(0); }\n");
    let metadata = ws.metadata("x");
    let constitution = constitution_of(vec![
        ModuleBoundary::in_crate("x")
            .module("crate::a")
            .must_not_call_inline("std::process")
            .because("a spawns no process"),
        ModuleBoundary::in_crate("x")
            .module("crate::b")
            .must_not_call_inline("std::process")
            .strict_prefix_only()
            .because("b does not even name a process"),
    ]);
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    let src = ws.src();
    assert_eq!(
        scans.source_reads(),
        read_once(&[src.join("lib.rs"), src.join("twin.rs")]),
        "a file loaded as two modules is read once for both"
    );
    let findings: BTreeSet<String> = match &outcome {
        Outcome::Violations(report) => report
            .violations
            .iter()
            .map(|violation| violation.finding.clone())
            .collect(),
        other => panic!("each module the file is loaded as reacts: {other:?}"),
    };
    assert_eq!(
        findings,
        BTreeSet::from([
            "std::process::exit in crate::a".to_string(),
            "std::process::exit in crate::b".to_string(),
        ]),
        "each module position of the one file is judged: {outcome:?}"
    );
    assert_eq!(
        outcome,
        crate::evaluate_with_scans(&constitution, &metadata, &EvaluationScans::independent()),
        "the shared reading judges each module position as a reading per boundary does"
    );
}

/// `module-boundary` scenario "A source no root reaches is never read": a `.rs` file under the
/// source directory that no `mod` declaration reaches, and that cannot be read, is never asked for,
/// so the evaluation's exit code is the one its reachable sources decide. Unix only and
/// self-calibrating: it skips under a privileged user, where mode 0 is still readable.
#[cfg(unix)]
#[test]
pub(super) fn a_source_no_root_reaches_is_never_read() {
    let ws = TempWorkspace::new("read-never");
    ws.write("lib.rs", "pub mod kernel;\npub mod projection;\n");
    ws.write("kernel.rs", "use crate::projection::P;\npub struct K;\n");
    ws.write("projection.rs", "pub struct P;\n");
    let orphan = ws.write("orphan.rs", "use crate::kernel::K;\n");
    let Some(_unreadable) = xingbiao::Unreadable::try_new(&orphan) else {
        return;
    };
    let metadata = ws.metadata("x");
    let constitution = constitution_of(vec![
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::projection")
            .because("the kernel does not reach the projection"),
    ]);
    let scans = EvaluationScans::shared();
    let outcome = crate::evaluate_with_scans(&constitution, &metadata, &scans);
    assert_eq!(
        outcome.exit_code(),
        1,
        "the reachable sources decide the exit code: {outcome:?}"
    );
    let src = ws.src();
    assert_eq!(
        scans.source_reads(),
        read_once(&[
            src.join("lib.rs"),
            src.join("kernel.rs"),
            src.join("projection.rs"),
        ]),
        "the unreachable, unreadable file is never read"
    );
}
