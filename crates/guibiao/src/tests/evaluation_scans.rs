use super::helpers::*;
use crate::module_scan::EvaluationScans;

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

/// `module-boundary` scenario "Shared and independent scans yield one outcome": every constitution
/// of the corpus — outbound, inbound with the value-namespace form, external confinement, the
/// inline family with strict and strict-external, a two-root package, and two refused
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
