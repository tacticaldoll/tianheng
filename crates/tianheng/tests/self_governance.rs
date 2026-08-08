//! Tianheng governs itself with its own reaction — the strongest robustness statement
//! a governance tool can make. Its architectural invariants are not prose in PROJECT.md
//! alone; they are declared here as a real constitution ([`tianheng_constitution`]) and
//! run as a `cargo test` gate, so CI fails the moment the law drifts.
//!
//! This is the crate-level upgrade of modou's module-level self-law: where modou could
//! only enforce `engine ⊥ runner` *within* one crate, Tianheng enforces the
//! functional-core ⊥ imperative-shell split across *crate* boundaries.

use std::path::{Path, PathBuf};

use tianheng::prelude::*;
use tianheng::{Boundary, Rule};

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
fn tianheng_constitution() -> Constitution {
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

#[test]
fn tianheng_governs_itself() {
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — self-governance runs in-repo only
    };
    GovernanceTest::for_constitution(tianheng_constitution())
        .with_manifest_dir(manifest.parent().unwrap())
        .assert_clean();
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
- 三儀 (圭表 static · 渾儀 semantic · 漏刻 runtime) measure; 垂象 surfaces a reaction, 實錄 records one, 校讎 amends one.
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

/// The unique live dependency allowlist governing the shell.
fn shell_dependency_boundary() -> Boundary {
    let mut shell_boundaries: Vec<Boundary> = tianheng_constitution()
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

fn shell_dependency_allowlist(boundary: &Boundary) -> &[String] {
    match boundary {
        Boundary::Crate(crate_boundary) => match crate_boundary.rule() {
            Rule::RestrictDependenciesTo { allowed, .. } => allowed,
            _ => unreachable!("the shell boundary selector admits only a dependency allowlist"),
        },
        _ => unreachable!("the shell boundary selector admits only a crate boundary"),
    }
}

/// Whether one comment line restates the shell's dependency declaration.
///
/// Named and taking a line, so the shape it refuses — and the shape it over-refuses — can be shown by giving it
/// text rather than by editing the shell until it trips. Its over-reaction is a declared bound of
/// `self-law-projection`, pinned by [`a_doc_example_of_the_dependency_dsl_is_refused`].
fn comment_restates_the_declaration(line: &str) -> bool {
    line.contains("restrict_dependencies_to(")
}

/// Whether one contiguous comment block names every member of the live allowlist.
///
/// Its over-reaction is likewise declared, pinned by
/// [`a_comment_naming_every_member_for_another_reason_is_refused`]: the question it answers is whether the
/// members all appear, never why, so a block naming them for a different purpose reads the same as a copy.
fn comment_block_copies_allowlist(block: &str, allowlist: &[String]) -> bool {
    !allowlist.is_empty()
        && allowlist.iter().all(|member| {
            block
                .split(|character: char| {
                    !(character.is_ascii_alphanumeric() || character == '_' || character == '-')
                })
                .any(|token| token == member)
        })
}

fn assert_comment_block_does_not_copy_allowlist(
    source: &Path,
    block_start: usize,
    block: &str,
    allowlist: &[String],
) {
    assert!(
        !comment_block_copies_allowlist(block, allowlist),
        "{}:{} names every live shell dependency allowlist member ({}) inside one line-comment block; \
         refer to AGENTS.self-law.md instead. This asks whether the members all appear, never why, so it \
         also refuses a block naming them for another reason — a declared over-reaction of \
         `self-law-projection`, not a case to work around",
        source.display(),
        block_start,
        allowlist.join(", ")
    );
}

/// Authored shell comments may explain the dependency boundary, but the live declaration and its
/// generated projection own the membership. The declaration token and a full member census are distinct
/// copied forms; both are refused without forbidding product code from legitimately calling the public DSL.
#[test]
fn shell_comments_do_not_restate_the_dependency_allowlist() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — the authored repository source is not present
    };
    let shell_boundary = shell_dependency_boundary();
    let allowlist = shell_dependency_allowlist(&shell_boundary);
    let mut pending = vec![root.join("crates/tianheng/src")];
    let mut rust_sources = Vec::new();

    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        {
            let entry = entry.unwrap_or_else(|error| {
                panic!("cannot enumerate {}: {error}", directory.display())
            });
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                rust_sources.push(path);
            }
        }
    }

    rust_sources.sort();
    for source in rust_sources {
        let text = std::fs::read_to_string(&source)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()));
        let mut comment_block = String::new();
        let mut block_start = 0usize;
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                if comment_block.is_empty() {
                    block_start = index + 1;
                }
                assert!(
                    !comment_restates_the_declaration(line),
                    "{}:{} names the shell dependency declaration in a comment; refer to \
                     AGENTS.self-law.md instead. This reads a comment's text and not its purpose, so it \
                     also refuses a doc example of the DSL — a declared over-reaction of \
                     `self-law-projection`, not a case to work around",
                    source.display(),
                    index + 1
                );
                comment_block.push_str(line);
                comment_block.push('\n');
            } else if !comment_block.is_empty() {
                assert_comment_block_does_not_copy_allowlist(
                    &source,
                    block_start,
                    &comment_block,
                    allowlist,
                );
                comment_block.clear();
            }
        }
        if !comment_block.is_empty() {
            assert_comment_block_does_not_copy_allowlist(
                &source,
                block_start,
                &comment_block,
                allowlist,
            );
        }
    }

    let style_source = root.join("crates/tianheng/src/runner/term_color.rs");
    let style_text = std::fs::read_to_string(&style_source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", style_source.display()));
    assert!(
        style_text.lines().any(|line| {
            line.trim_start().starts_with("//") && line.contains("AGENTS.self-law.md")
        }),
        "{} must direct its dependency rationale to the generated self-law projection",
        style_source.display()
    );
}

/// `self-law-projection/a-doc-example-of-the-dependency-dsl-is-refused-a-stated-bound`
///
/// The recognizer reads a comment's **text**, never its purpose, so a rustdoc example teaching the re-exported
/// DSL is refused exactly as a restatement of the shell's own declaration would be. That is the safe direction —
/// a false positive is a sentence to rewrite, where the false negative would be a copied declaration nothing
/// governs — and the shell publishes this DSL, so the shape is live even with no instance in the tree today.
///
/// The control matters as much: a comment discussing the boundary without naming the call is accepted, so this
/// shows a limit of reading text rather than a recognizer that refuses every comment.
#[test]
fn a_doc_example_of_the_dependency_dsl_is_refused() {
    assert!(
        comment_restates_the_declaration(
            "/// CrateBoundary::crate_(\"x\").restrict_dependencies_to([\"y\"])"
        ),
        "a doc example of the DSL is refused, though it restates nothing about this shell — the declared \
         over-reaction"
    );
    assert!(
        comment_restates_the_declaration(
            "// the shell's own restrict_dependencies_to(guibiao, hunyi) list"
        ),
        "the control in the other direction: a real restatement is what the reaction is for"
    );
    assert!(
        !comment_restates_the_declaration(
            "// the shell's dependency allowlist, see AGENTS.self-law.md"
        ),
        "and a comment explaining the boundary without naming the call is accepted, so the refusals above \
         are a limit of reading text rather than a recognizer that refuses everything"
    );
}

/// `self-law-projection/a-comment-naming-every-member-for-another-reason-is-refused-a-stated-bound`
///
/// The block check asks whether every allowlist member appears, never why, so a block naming them for a
/// different purpose — a crate-level note on what the shell composes, say — reads the same as a copied census.
/// Kept over-reacting rather than taught to read intent: the alternative is a heuristic over prose, which this
/// repository has measured and rejected elsewhere.
#[test]
fn a_comment_naming_every_member_for_another_reason_is_refused() {
    let allowlist: Vec<String> = ["guibiao", "hunyi", "louke", "serde_json"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert!(
        comment_block_copies_allowlist(
            "//! The shell composes guibiao, hunyi and louke, and serializes its report with serde_json.\n",
            &allowlist
        ),
        "a block naming the members for another reason is refused — the declared over-reaction"
    );
    assert!(
        !comment_block_copies_allowlist(
            "//! The shell composes guibiao, hunyi and louke.\n",
            &allowlist
        ),
        "the control: naming some of them is accepted, so the refusal above is about the full set and not \
         about mentioning a crate at all"
    );
}

/// Contract A — the agent-loaded `AGENTS.self-law.md` must byte-match the live projection of
/// `tianheng_constitution()`. Stale → fail (with the regenerate command); `BLESS=1` → rewrite
/// the file instead of asserting (so the artifact changes by regeneration, never by hand).
#[test]
fn self_law_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — same repo-only discipline as the governance gate
    };
    GovernanceTest::for_constitution(tianheng_constitution())
        .with_manifest_dir(&root)
        .assert_projection_fresh_with_preamble(
            root.join("AGENTS.self-law.md"),
            &format!("{SELF_LAW_PREAMBLE}\n"),
        );
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
/// Two statements, and only one of them was here. The `because` **text** is observed by a `contains` check;
/// what was missing is the other statement — that the allowlist itself **obeys** the clause it quotes.
///
/// The text half has two limits, both measured by writing them into the tree rather than argued about, and
/// **neither is a declared bound**; `BACKLOG.md` carries them. Paraphrasing `guibiao`'s clause makes this
/// reaction **fire** — an over-reaction, a refusal of a reason that genuinely states the law. A `because` that
/// carries the literal clause while *negating* it passes, and the projection then teaches the negation — the
/// under-reaction. A draft of this change declared the first as a false NEGATIVE, which one run of its own WHEN
/// falsified: a bound's extent is read off that run, never off the argument for it. Widening
/// `guibiao`'s allowlist to name `hunyi` left every test binary in this workspace green, with
/// `AGENTS.self-law.md` printing the sibling directly beneath the reason that forbids it. Freshness pinned the
/// projection against the declaration; nothing pinned the declaration against its own law.
#[test]
fn dimension_boundaries_declare_the_mutual_independence_law() {
    const CLAUSE: &str = "三儀 ⊥ 三儀";
    // A hand-kept list beside an enumerable set: a dimension born and not added here has its allowlist
    // unchecked, and the set-coverage assertion below cannot notice, because `found` is produced by filtering
    // on `expected`. Measured — removing `guibiao` from this literal leaves a `guibiao` allowlist naming
    // `hunyi` green. Nor does the filter reach `restrict_workspace_dependencies_to`, which is the more natural
    // rule for this law. `BACKLOG.md` carries both; neither is a declared bound.
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

        // The clause in the `because` said the law; this asserts the ALLOWLIST obeys it, which is a
        // different statement and was the missing one. Reproduced before adding it: widening `guibiao`'s
        // allowlist to name `hunyi` left every one of this workspace's test binaries green, and
        // `AGENTS.self-law.md` regenerated to print `only: serde_json, xuanji, xingbiao, hunyi`
        // directly beneath the reason that says no sibling is named. Freshness pinned projection against
        // declaration; nothing pinned the declaration against the law it quotes.
        //
        // `tianheng_governs_itself` cannot backstop this either: a WIDENED allowlist permits more than
        // the tree does, so no dependency violation appears and the reaction stays clean.
        let target = cb.target().package.as_str();
        let Rule::RestrictDependenciesTo { allowed, .. } = cb.rule() else {
            unreachable!("the filter above selected only restrict-dependencies rules");
        };
        let siblings: Vec<&str> = allowed
            .iter()
            .map(String::as_str)
            .filter(|name| DIMENSIONS.contains(name) && *name != target)
            .collect();
        assert!(
            siblings.is_empty(),
            "`{target}`'s allowlist names sibling dimension(s) {siblings:?}, so the boundary permits \
             exactly what `{CLAUSE}` forbids — a dimension must never learn from a sibling, and its \
             own `because` says so two lines above the allowlist that now allows it"
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
    GovernanceTest::for_constitution(tianheng_constitution())
        .with_manifest_dir(manifest.parent().unwrap())
        .assert_all_workspace_members_covered();
}

#[test]
fn fixture_negative_testing_observes_violating_fixture() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let root = manifest.parent().unwrap();
    let fixture = root.join("crates/tianheng/tests/fixtures/violating/Cargo.toml");
    let fixture_constitution = Constitution::new("example").boundary(
        CrateBoundary::crate_("example-core")
            .deny_external_dependencies()
            .because("example-core is a domain-free core and must stay dependency-light"),
    );

    GovernanceTest::for_constitution(fixture_constitution)
        .with_manifest_dir(root)
        .test_fixture(fixture);
}

/// The real shell dependency boundary reacts to a direct edge into the lower metadata substrate.
///
/// The boundary is selected from [`tianheng_constitution`] instead of restating its allowlist here. Exactly one
/// match is required so a duplicate or renamed shell declaration cannot turn this into evidence about an
/// arbitrary boundary. The isolated fixture carries no other dependency, so its violation cannot be satisfied
/// by a different forbidden edge.
#[test]
fn fixture_negative_testing_observes_shell_metadata_edge() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let root = manifest.parent().unwrap();
    let fixture = root.join("crates/tianheng/tests/fixtures/shell_metadata_edge/Cargo.toml");
    let fixture_constitution =
        Constitution::new("shell-metadata-edge").boundary(shell_dependency_boundary());

    GovernanceTest::for_constitution(fixture_constitution)
        .with_manifest_dir(root)
        .test_fixture(fixture);
}

#[test]
fn fixture_negative_testing_observes_cfg_if_violation() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let root = manifest.parent().unwrap();
    let fixture = root.join("crates/tianheng/tests/fixtures/cfg_if_violation/Cargo.toml");
    let fixture_constitution = Constitution::new("example").boundary(
        ModuleBoundary::in_crate("example-core")
            .module("crate::kernel_mod")
            .must_not_import("crate::secret")
            .because("kernel_mod must not import secret even inside cfg_if!"),
    );

    GovernanceTest::for_constitution(fixture_constitution)
        .with_manifest_dir(root)
        .test_fixture(fixture);
}

#[test]
fn fixture_negative_testing_observes_glob_hazard_violation() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let root = manifest.parent().unwrap();
    let fixture = root.join("crates/tianheng/tests/fixtures/glob_hazard_violation/Cargo.toml");
    let fixture_constitution = Constitution::new("example").boundary(
        ModuleBoundary::in_crate("example-core")
            .module("crate::app")
            .must_not_import("crate::domain::secret")
            .because("app must not import domain::secret via ancestor glob"),
    );

    GovernanceTest::for_constitution(fixture_constitution)
        .with_manifest_dir(root)
        .test_fixture(fixture);
}
