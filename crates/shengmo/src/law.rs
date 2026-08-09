//! The law 天衡 declares over its own repository.
//!
//! Written with the product's own declaration API: the capability applied to its author. This is
//! code, not a test — what runs it is a reaction, and what reads it is the projection.

use tianheng::prelude::*;
use tianheng::{Boundary, Rule};

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
/// [`dimension_boundaries_declare_the_mutual_independence_law`] — asserts both that every
/// dimension boundary carries the clause and that its allowlist **obeys** it, so the claim is
/// *self-observed* rather than a hand-maintained pointer that could drift (the
/// declaration-integrity pattern: replace a prose index with a reaction). The second half was
/// added after a widened allowlist naming a sibling was measured green everywhere.
/// A constitution reader and the 垂象 report both see the intent. It adds **no separate
/// boundary on purpose: a dimension's `restrict_dependencies_to` allowlist names no sibling
/// dimension, so a cross-dimension dependency already reacts. A `forbid_dependency_on`
/// between dimensions would be a second reaction for a drift the allowlist already catches
/// — and an allowlist is always stricter than a denylist, so it would add zero protection.
/// Minimalism forbids the redundant reaction; the law is made *visible*, not re-enforced.
///
/// A wrong boundary here is fixed by a human-reviewed amendment, never by quietly
/// weakening this function to make CI pass.
pub fn constitution() -> Constitution {
    Constitution::new("tianheng")
        .boundary(
            CrateBoundary::crate_("xuanji")
                .restrict_dependencies_to(["serde_json"])
                .because(
                    "璇璣 is the dimension-agnostic reaction model: serde_json only, below every \
                     dimension, and must not depend on any workspace member",
                ),
        )
        .boundary(
            CrateBoundary::crate_("xingbiao")
                .restrict_dependencies_to(["serde_json"])
                .because(
                    "星表 is the shared metadata substrate: serde_json only, reading cargo \
                     metadata beneath the dimensions without depending on workspace members",
                ),
        )
        .boundary(
            CrateBoundary::crate_("guibiao")
                .restrict_dependencies_to(["serde_json", "xuanji", "xingbiao"])
                .because(
                    "the 圭表 static core stays dependency-light: serde_json, xuanji (reaction \
                     model), and xingbiao (metadata substrate) only. functional core ⊥ imperative \
                     shell: 圭表 must not depend on the 天衡 shell. 三儀 ⊥ 三儀: naming no \
                     sibling dimension, the observation dimensions are composed only by the 天衡 \
                     shell, never by each other",
                ),
        )
        .boundary(
            CrateBoundary::crate_("hunyi")
                .restrict_dependencies_to(["xuanji", "xingbiao", "serde_json", "syn"])
                .because(
                    "渾儀 is the semantic AST dimension: quarantined syn dependency only. 三儀 ⊥ \
                     三儀: it depends on no sibling dimension and never on the 天衡 shell \
                     (functional dimension ⊥ imperative shell)",
                ),
        )
        .boundary(
            CrateBoundary::crate_("louke")
                .restrict_dependencies_to(["xuanji", "xingbiao"])
                .because(
                    "漏刻 is the runtime dimension: hot path depends on 璇璣 only, with xingbiao \
                     audit-gated for CI probe coverage. 三儀 ⊥ 三儀: naming no sibling dimension, \
                     it reacts in prod independently of the 天衡 shell",
                ),
        )
        .boundary(
            CrateBoundary::crate_("tianheng")
                .restrict_dependencies_to(["guibiao", "hunyi", "louke", "serde_json"])
                .because(
                    "the 天衡 shell remains the outward composition layer: direct normal edges \
                     end at observation dimensions and projection serialization, never at the \
                     lower reaction model or metadata substrate",
                ),
        )
        .boundary(
            CrateBoundary::crate_("shengmo")
                .restrict_dependencies_to(["tianheng", "serde_json"])
                .because(
                    "繩墨 is an adopter of 天衡, not a member of the family it governs: it \
                     declares this law through the shell's published surface and reaches no \
                     dimension directly, so the repository's own governance exercises exactly the \
                     surface an adopter has. serde_json is the one addition, for reading cargo's \
                     own message stream where a reaction's corpus must come from the build rather \
                     than from a list",
                ),
        )
        .boundary(
            CrateBoundary::crate_("jiaochou")
                .restrict_dependencies_to(["shengmo", "tianheng", "serde_json"])
                .because(
                    "校讎 collates this repository's record against itself and governs no product \
                     contract: it reaches the shell's published surface and the law's own locator, \
                     never a dimension. Keeping it distinct from 繩墨 is what stops a claim about \
                     the law being read as a claim about document hygiene",
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
                     above it, never the model itself",
                ),
        )
        // Path canonicalization and cycle/dedup guards in observation crates must resolve through
        // `xingbiao::canonicalize_or_fail` or `try_visit` for fail-loud failure handling across all
        // modules in the crate subtree.
        .boundary(
            ModuleBoundary::in_crate("guibiao")
                .module("crate")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization and cycle/dedup guards in guibiao must resolve \
                     through `xingbiao::canonicalize_or_fail` or `try_visit` for unified \
                     failure handling",
                ),
        )
        .boundary(
            ModuleBoundary::in_crate("hunyi")
                .module("crate")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization and cycle/dedup guards in hunyi must resolve \
                     through `xingbiao::canonicalize_or_fail` or `try_visit` for unified \
                     failure handling",
                ),
        )
        .boundary(
            ModuleBoundary::in_crate("louke")
                .module("crate")
                .must_not_call_inline("std::fs")
                .ending_with(["canonicalize"])
                .depth(ScanDepth::Subtree)
                .because(
                    "path canonicalization and cycle/dedup guards in louke must resolve \
                     through `xingbiao::try_visit` for unified failure handling",
                ),
        )
}

/// The fixed preamble of the agent-loaded self-law projection (`AGENTS.self-law.md`). It is a
/// generated constant — never hand-edited prose — so the whole artifact is byte-checked and
/// cannot drift. It describes only **how to read the projection** and the reaction loop it
/// serves; it makes **no crate-specific architectural claim** (every such claim comes only from
/// the generated projection below it, where it traces to a boundary that actually reacts —
/// otherwise it would be the open-loop prose prescription PROJECT.md's 潛移 section forbids).
pub const PREAMBLE: &str = "\
# Tianheng Self-Law Projection

Generated from `shengmo::law::constitution()` by `crates/shengmo/tests/self_governance.rs`.
**Do not edit by hand.** If this file is stale, regenerate it:
`BLESS=1 cargo test -p shengmo self_law_projection_is_fresh`.
If the law itself is wrong, amend `shengmo::law` through review — never edit this projection.
The law is named by module rather than by file here: this header registers the unit holding the
projection fresh, and a second tracked path in it would be an ambiguous claim about which one does.

Read the projection below as the imitable shape of Tianheng itself, and work *with* the reaction:

- Declare intent in Rust; the source is the single source of truth.
- Observe only what has a real observation source; name nothing that does not react.
- React with the outcomes: `0` clean, `1` violation, `2` constitution/usage error.
- On a violation, repair toward the boundary's declared reason — never weaken the law to pass.
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 垂象 surfaces a reaction, 實錄 records one, 校讎 amends one.
";

/// The unique live dependency allowlist governing the shell.
pub fn shell_dependency_boundary() -> Boundary {
    let mut shell_boundaries: Vec<Boundary> = constitution()
        .static_boundaries()
        .boundaries()
        .iter()
        .filter(|boundary| {
            matches!(
                boundary,
                Boundary::Crate(crate_boundary)
                    if crate_boundary.target().package == "tianheng"
                        && matches!(crate_boundary.rule(), Rule::RestrictDependenciesTo { .. })
            )
        })
        .cloned()
        .collect();
    assert_eq!(
        shell_boundaries.len(),
        1,
        "the self-constitution must declare exactly one tianheng dependency allowlist; a repository reaction \
         must not choose an arbitrary duplicate or silently stop observing a renamed boundary"
    );
    shell_boundaries.pop().expect("the unique shell boundary")
}

pub fn shell_dependency_allowlist(boundary: &Boundary) -> &[String] {
    match boundary {
        Boundary::Crate(crate_boundary) => match crate_boundary.rule() {
            Rule::RestrictDependenciesTo { allowed, .. } => allowed,
            _ => unreachable!("the shell boundary selector admits only a dependency allowlist"),
        },
        _ => unreachable!("the shell boundary selector admits only a crate boundary"),
    }
}
