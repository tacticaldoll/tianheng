//! Tianheng governs itself with its own reaction — the strongest robustness statement
//! a governance tool can make. Its architectural invariants are not prose in PROJECT.md
//! alone; they are declared here as a real constitution ([`tianheng_constitution`]) and
//! run as a `cargo test` gate, so CI fails the moment the law drifts.
//!
//! This is the crate-level upgrade of modou's module-level self-law: where modou could
//! only enforce `engine ⊥ runner` *within* one crate, Tianheng enforces the
//! functional-core ⊥ imperative-shell split across *crate* boundaries.

use std::path::PathBuf;

use guibiao::check_and_cover;
use tianheng::prelude::*;
use tianheng::{Boundary, Rule, constitution_markdown};

/// The Tianheng workspace manifest. `None` when it is absent — e.g. inside a published
/// `.crate` tarball, which has no workspace root — so the self-governance gate SKIPS rather
/// than fails when the crate is tested standalone. In the repo the path exists, so the gate
/// runs for real.
fn workspace_manifest() -> Option<PathBuf> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    if path.exists() {
        return Some(path);
    }
    // Absent. CI sets TIANHENG_WORKSPACE_TESTS=1 so a missing manifest (a checkout/layout
    // regression) fails LOUD rather than silently skipping the dogfood gate; without the env
    // (e.g. a packaged .crate tested standalone) the absence is legitimate, so skip.
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "workspace manifest expected but absent while TIANHENG_WORKSPACE_TESTS is set — \
         the self-governance gate must not silently skip in CI"
    );
    None
}

/// **Tianheng's self-constitution — the law it enforces on itself.**
///
/// Declared in the same Rust DSL adopters use. [`tianheng_governs_itself`] runs it as a
/// real reaction against the workspace, so the dogfooding is a non-bypassable gate, not
/// a hope. Each boundary below carries its own `because` — its target, allowlist, and reason
/// — and those project (target · rule · reason) into the generated, byte-checked
/// `AGENTS.self-law.md` (gated by [`self_law_projection_is_fresh`]). This doc comment therefore
/// does **not** re-list the boundaries by hand: a per-boundary index restated in prose is the
/// drift surface the declaration-integrity pattern retires — the same class as the removed
/// `(boundaries 2, 3, 6)` pointer that once drifted off-by-one. It records only the cross-cutting
/// law the boundaries jointly realize, which no single `because` owns:
///
/// **Cross-cutting — 三儀 ⊥ 三儀 (the dimensions are mutually independent).** The
/// observation dimensions — 圭表 (static), 渾儀 (semantic), and 漏刻 (runtime) — never depend
/// on one another; each sits on the shared bases below them (璇璣 the reaction model, and — for
/// the dimensions that read the workspace — 星表 the metadata substrate) and is composed into one
/// reaction *only* by the 天衡 shell (for the CI dimensions) or reacts independently in prod
/// (漏刻), never via a sibling. Depending on a shared base beneath the dimensions is not a
/// cross-dimension edge; 三儀 ⊥ 三儀 forbids only dimension-to-dimension dependence. This
/// law is **named here and in each dimension's `because`**, and a reaction —
/// [`dimension_boundaries_declare_the_mutual_independence_law`] — asserts that every dimension
/// boundary carries the clause, so the claim is *self-observed*, not a hand-maintained pointer
/// that could drift (the declaration-integrity pattern: replace a prose index with a reaction).
/// A constitution reader and the 垂象 report both see the intent. It adds **no separate
/// boundary on purpose: a dimension's `restrict_dependencies_to` allowlist names no sibling
/// dimension, so a cross-dimension dependency already reacts. A `forbid_dependency_on`
/// between dimensions would be a second reaction for a drift the allowlist already catches
/// — and an allowlist is always stricter than a denylist, so it would add zero protection.
/// Minimalism forbids the redundant reaction; the law is made *visible*, not re-enforced.
///
/// A wrong boundary here is fixed by a human-reviewed amendment, never by quietly
/// weakening this function to make CI pass.
fn tianheng_constitution() -> Constitution {
    Constitution::new("tianheng")
        .boundary(
            CrateBoundary::crate_("xuanji")
                .restrict_dependencies_to(["serde_json"])
                .because(
                    "璇璣 is the dimension-agnostic reaction model: serde_json only, and \
                     below every dimension — it must not depend on any workspace member \
                     (no engine, no shell), so nothing in the family sits beneath it",
                ),
        )
        .boundary(
            CrateBoundary::crate_("xingbiao")
                .restrict_dependencies_to(["serde_json"])
                .because(
                    "星表 is the shared declared-workspace-data substrate: serde_json only, and \
                     below every dimension like 璇璣 — it reads `cargo metadata` and must not \
                     depend on any workspace member, so the static and semantic dimensions read \
                     the workspace through one source of truth, not two hand-copied twins that \
                     drift apart. Not 璇璣: it does IO (spawns cargo) and observes, so it is not \
                     the measure-only reaction model — a substrate beneath the dimensions, not \
                     the measure they react in",
                ),
        )
        .boundary(
            CrateBoundary::crate_("guibiao")
                .restrict_dependencies_to(["serde_json", "xuanji", "xingbiao"])
                .because(
                    "the 圭表 core stays dependency-light: serde_json is the only external \
                     dependency (no syn / proc-macro, no heavy graph or runtime crates); the \
                     internal dependencies on 璇璣 (the shared reaction model) and 星表 (the \
                     shared metadata substrate) are the price of the family split — both are \
                     serde_json-only bases below the dimensions: the model renders no verdict \
                     and the substrate only reads the workspace, neither drags in an engine. \
                     三儀 ⊥ 三儀: this allowlist \
                     names no sibling dimension, so 圭表 cannot depend on 渾儀 (nor, when born, \
                     漏刻) — the dimensions are composed only by the 天衡 shell, never by each \
                     other",
                ),
        )
        .boundary(
            CrateBoundary::crate_("guibiao")
                .forbid_dependency_on(["tianheng"])
                .because(
                    "functional core ⊥ imperative shell: the 圭表 core crate must not \
                     depend on the 天衡 gate/shell",
                ),
        )
        .boundary(
            CrateBoundary::crate_("hunyi")
                .restrict_dependencies_to(["xuanji", "xingbiao", "serde_json", "syn"])
                .because(
                    "渾儀 is the semantic dimension and the sole holder of the heavy syn AST \
                     dependency — quarantined here, never the core or the model; it depends on \
                     璇璣 (the reaction model), 星表 (the shared metadata substrate), serde_json, \
                     and syn only. 三儀 ⊥ 三儀: it never \
                     depends on the sibling 圭表 dimension (nor, when born, 漏刻), and never on \
                     the 天衡 shell — the dimensions are composed only by the shell, never by \
                     each other (functional dimension ⊥ imperative shell)",
                ),
        )
        .boundary(
            CrateBoundary::crate_("louke")
                .restrict_dependencies_to(["xuanji", "xingbiao"])
                .because(
                    "漏刻 is the runtime dimension and ships into the user's production binary, \
                     so its hot path stays production-light: it depends on 璇璣 (the reaction \
                     model) only — no syn, no static engine, no sibling dimension. 星表 is an \
                     additive, `audit`-feature-gated exception (never reaches the production hot \
                     path): the CI-only probe scanner's own cycle guard routes through 星表's \
                     shared canonicalize/cycle-guard primitives, the same ones 圭表/渾儀 already \
                     use, rather than carrying a third independently hand-rolled copy. 三儀 ⊥ 三儀: \
                     naming no sibling, it cannot depend on the 圭表/渾儀 dimensions, and it reacts \
                     in prod independently of the 天衡 shell (serde_json reaches it only \
                     transitively via 璇璣, cold-path only)",
                ),
        )
        .boundary(
            CrateBoundary::crate_("tianheng")
                .restrict_dependencies_to([
                    "guibiao",
                    "hunyi",
                    "louke",
                    "serde_json",
                    "xingbiao",
                ])
                .because(
                    "the 天衡 shell composes the 三儀 into one reaction, so it depends on the 圭表 \
                     static core, the 渾儀 semantic dimension, and the 漏刻 runtime dimension (whose \
                     CI probe-coverage face it composes into `check`), reads exact Cargo target \
                     roots through the shared 星表 substrate, and projects with serde_json; all \
                     edges point to dimensions or shared bases beneath the shell",
                ),
        )
        // The first *semantic* self-boundary: the family dogfoods its own `sans_io_pure` profile on
        // 璇璣, the crate that most owes the sans-I/O property. It spans two dimensions (圭表
        // must-not-call-inline for the clock, 渾儀 must-not-expose-async for the API), so it is the
        // shell's to compose — exactly the 三儀 ⊥ 三儀 shape stated above, now exercised on self.
        .sans_io_pure(
            SansIoPure::in_crate("xuanji")
                .module("crate")
                .reading_clock_via("std::time", ["now"])
                .because(
                    "璇璣 is the measure-only reaction model: it reads no ambient clock inline and \
                     exposes no async surface — time and effects enter only through the dimensions \
                     above it, never the model itself. The clock axis reacts via 圭表 \
                     (must-not-call-inline `std::time::…::now`), the async axis via 渾儀 \
                     (must-not-expose an async public fn)",
                ),
        )
        // 圭表's own inline-symbol-path confinement, reused against 渾儀 and 圭表's own module-graph
        // walkers (not just 璇璣): the 0.2.2 lesson found the same canonicalize-before-cycle-guard
        // step hand-rolled at multiple call sites with disagreeing failure policies (three in one
        // `reachability.rs` file alone). Both walkers now route through the shared,
        // fail-loud `xingbiao::canonicalize_or_fail`/`try_visit`; these boundaries confine the raw
        // call so a future reintroduced inline `std::fs::…::canonicalize` fails CI instead of
        // waiting for the next adversarial round to notice.
        .boundary(
            ModuleBoundary::in_crate("hunyi")
                .module("crate::module_resolve")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization for this resolver's own cycle/dedup guard must go \
                     through the shared, fail-loud `xingbiao::try_visit`, never be re-hand-rolled \
                     inline here — the 0.2.2 lesson (a canonicalize-failure policy hand-rolled per \
                     call site drifted to disagreeing behavior across this crate)",
                ),
        )
        .boundary(
            ModuleBoundary::in_crate("guibiao")
                .module("crate::module_scan::reachability")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization for this walker's own cycle/dedup guard must go \
                     through the shared, fail-loud `xingbiao::canonicalize_or_fail`/`try_visit`, \
                     never be re-hand-rolled inline here — the 0.2.2 lesson (this exact file once \
                     carried three disagreeing canonicalize-failure policies at once)",
                ),
        )
        .boundary(
            ModuleBoundary::in_crate("hunyi")
                .module("crate::scan")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization for this crate-wide walker's own cycle/dedup guard \
                     must go through the shared, fail-loud `xingbiao::canonicalize_or_fail`, \
                     never be re-hand-rolled inline here — a sibling instance of the 0.2.2 lesson \
                     found in this same crate's `module_resolve` (a second, independently \
                     hand-rolled wrapper here once carried its own disagreeing error-message \
                     policy)",
                ),
        )
        .boundary(
            ModuleBoundary::in_crate("louke")
                .module("crate::audit::scan")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "this CI-only probe scanner's module-cycle guard must go through the shared, \
                     fail-loud `xingbiao::try_visit`, never be re-hand-rolled inline here — closes \
                     the same class of drift 圭表/渾儀's own guards were confined against, now that \
                     漏刻's self-law permits the additive, `audit`-feature-gated `xingbiao` \
                     dependency this routes through",
                ),
        )
}

#[test]
fn tianheng_governs_itself() {
    // The whole self-constitution reacts through the same composed evaluator an adopter calls.
    // Static → semantic → runtime-audit ordering and constitution-error precedence therefore
    // dogfood the public shell heart, including the always-run runtime audit when this law declares
    // no runtime boundaries. Any drift surfaces with the producing boundary's reason.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — self-governance runs in-repo only
    };
    let constitution = tianheng_constitution();
    let outcome = check_constitution(&constitution, &manifest);
    assert!(
        matches!(outcome, Outcome::Clean),
        "Tianheng's composed self-law drifted: {outcome:?}"
    );
    assert_eq!(outcome.exit_code(), 0);
}

/// The fixed preamble of the agent-loaded self-law projection (`AGENTS.self-law.md`). It is a
/// generated constant — never hand-edited prose — so the whole artifact is byte-checked and
/// cannot drift. It describes only **how to read the projection** and the reaction loop it
/// serves; it makes **no crate-specific architectural claim** (every such claim comes only from
/// the generated projection below it, where it traces to a boundary that actually reacts —
/// otherwise it would be the open-loop prose prescription PROJECT.md's 潛移 section forbids).
const SELF_LAW_PREAMBLE: &str = "\
# Tianheng Self-Law Projection

Generated from `tianheng_constitution()` in `crates/tianheng/tests/self_governance.rs`.
**Do not edit by hand.** If this file is stale, regenerate it:
`BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`.
If the law itself is wrong, amend `self_governance.rs` through review — never edit this projection.

Read the projection below as the imitable shape of Tianheng itself, and work *with* the reaction:

- Declare intent in Rust; the source is the single source of truth.
- Observe only what has a real observation source; name nothing that does not react.
- React with the outcomes: `0` clean, `1` violation, `2` constitution/usage error.
- On a violation, repair toward the boundary's declared reason — never weaken the law to pass.
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 三司 (垂象 · 實錄 · 校讎) administer.
";

/// The repository root — the parent of the workspace manifest. Reuses [`workspace_manifest`]'s
/// repo-only discipline verbatim: `None` (skip) outside a checkout, fail-loud under
/// `TIANHENG_WORKSPACE_TESTS`.
fn workspace_root() -> Option<PathBuf> {
    workspace_manifest().map(|m| {
        m.parent()
            .expect("the workspace manifest has a parent directory")
            .to_path_buf()
    })
}

/// The whole self-law artifact: the fixed preamble, then the live projection of the *same*
/// self-constitution the dogfood gate reacts against, rendered by the *same* renderer as
/// `list --format markdown`. The seam newline between the two is owned here (the doc-builder),
/// never smuggled into [`constitution_markdown`], which adds nothing of its own; the trailing
/// newline makes the file end conventionally.
fn render_self_law_doc() -> String {
    let projection = constitution_markdown(&tianheng_constitution());
    format!("{SELF_LAW_PREAMBLE}\n{projection}\n")
}

/// Contract A — the agent-loaded `AGENTS.self-law.md` must byte-match the live projection of
/// `tianheng_constitution()`. Stale → fail (with the regenerate command); `BLESS=1` → rewrite
/// the file instead of asserting (so the artifact changes by regeneration, never by hand).
#[test]
fn self_law_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — same repo-only discipline as the governance gate
    };
    let path = root.join("AGENTS.self-law.md");
    let live = render_self_law_doc();

    // Delegate the read/bless/compare to the reusable `projection_gate` helper — the same gate
    // adopters call for their own projection. The test owns the workspace-only early return above
    // and reads its own `BLESS` (the helper reads no environment); the helper's `Err` names the
    // artifact path, preserving the "names the artifact" staleness contract.
    let bless = std::env::var_os("BLESS").is_some();
    tianheng::projection_gate(
        &live,
        &path,
        "BLESS=1 cargo test -p tianheng self_law_projection_is_fresh",
        bless,
    )
    .unwrap_or_else(|e| panic!("{e}"));
}

/// Contract C — the **declaration-integrity** reaction (the 潛移/校讎-adjacent shape: its
/// observation source is the *declaration itself*, not governed code). A structural property of
/// `tianheng_constitution()` is asserted, so a hand-written pointer to that property can be
/// *deleted* rather than kept correct by hand. Here: the cross-cutting 三儀 ⊥ 三儀 law is carried
/// in every dimension boundary's `because`. If a dimension's reason drops the clause — or a
/// dimension boundary is removed, renamed, or duplicated (the selected targets are compared as a
/// set, so "duplicate one, drop another" cannot pass on count alone) — this fails; the
/// `(boundaries 2, 3, 6)` prose index it replaces would instead have silently rotted (the exact
/// class of the off-by-one it retires).
///
/// Stated bound: the predicate observes the `because` **text** (a `contains` check), weaker than a
/// structural fact — a reworded clause would slip it. It still reacts to the real drift (a
/// dimension boundary losing the law), which a hand-maintained pointer could not.
#[test]
fn dimension_boundaries_declare_the_mutual_independence_law() {
    const CLAUSE: &str = "三儀 ⊥ 三儀";
    const DIMENSIONS: [&str; 3] = ["guibiao", "hunyi", "louke"];

    let constitution = tianheng_constitution();
    let dimension_allowlists: Vec<_> = constitution
        .static_boundaries()
        .boundaries()
        .iter()
        .filter_map(|boundary| match boundary {
            Boundary::Crate(cb)
                if DIMENSIONS.contains(&cb.target().package.as_str())
                    && matches!(cb.rule(), Rule::RestrictDependenciesTo { .. }) =>
            {
                Some(cb)
            }
            _ => None,
        })
        .collect();

    // Each dimension must appear **exactly once** — assert set coverage, not a bare count. A
    // bare `len == 3` would pass a copy-paste drift that duplicates one dimension and drops
    // another (two `hunyi` allowlists, no `louke`): the count still reads 3 and every selected
    // reason still carries the clause, yet `louke`'s allowlist has silently vanished — and
    // `tianheng_governs_itself` cannot backstop it (a dropped `louke` boundary triggers no
    // dependency reaction, since `louke` really does depend only on `xuanji`). So this test is
    // the sole guard, and it must compare the selected targets, sorted, to the dimensions.
    let mut found: Vec<&str> = dimension_allowlists
        .iter()
        .map(|cb| cb.target().package.as_str())
        .collect();
    found.sort_unstable();
    let mut expected: Vec<&str> = DIMENSIONS.to_vec();
    expected.sort_unstable();
    assert_eq!(
        found, expected,
        "each dimension needs exactly one restrict-dependencies allowlist ({DIMENSIONS:?}); \
         a dimension boundary was renamed, removed, or duplicated"
    );
    for cb in dimension_allowlists {
        assert!(
            cb.reason().contains(CLAUSE),
            "dimension boundary for `{}` dropped the `{CLAUSE}` clause from its because — \
             the cross-cutting law is no longer self-declared at that dimension",
            cb.target().package
        );
    }
}

/// Contract D — the **declaration-integrity coverage** reaction (again the 潛移/校讎-adjacent
/// shape: its observation source is the *declaration and the workspace metadata*, not governed
/// code). Every workspace member must be the target of at least one boundary in
/// `tianheng_constitution()`.
///
/// Without this, a crate added to the family with no self-governance boundary escapes the
/// dogfood gate **silently**: [`tianheng_governs_itself`] only reacts to crates a boundary
/// *names*, so an ungoverned member triggers no dependency reaction and could take any
/// dependency — heavy, cross-dimension, or the shell — undetected. That is a false negative of
/// the self-law itself (the one forbidden bug), and it is exactly the "all N crates are
/// governed" coverage claim that today is hand-restated across the docs
/// (`PROJECT.md`, `README.md`, `AGENTS.md`) rather than observed. Here the property is asserted
/// on the live `Constitution` + `cargo metadata`, so that claim need not be hand-counted — the
/// same move as Contract C (a prose index → a reaction), applied to coverage.
///
/// The `total > 0` guard forecloses a **vacuous** pass: if the metadata read ever returned zero
/// members, `uncovered` would be empty and the assertion would hold for the wrong reason. A
/// count floor is deliberately *not* hardcoded (it would be the very hand-maintained index this
/// pattern retires) — growth must not require editing this test.
#[test]
fn every_workspace_member_is_self_governed() {
    let Some(manifest) = workspace_manifest() else {
        return; // outside a checkout — same repo-only discipline as the governance gate
    };
    let constitution = tianheng_constitution();
    let (_, coverage) = check_and_cover(constitution.static_boundaries(), &manifest);
    let coverage = coverage.expect("workspace metadata is readable in-repo");
    assert!(
        coverage.total > 0,
        "coverage observed zero workspace members — the metadata read is degenerate, so an \
         empty `uncovered` would pass this gate vacuously"
    );
    assert!(
        coverage.uncovered.is_empty(),
        "workspace members escape self-governance (no boundary in `tianheng_constitution()` \
         targets them): {:?} — add a boundary for each, or the dogfood gate silently skips \
         them (a false negative of the self-law)",
        coverage.uncovered
    );
}
