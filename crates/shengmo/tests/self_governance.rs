//! Tianheng's self-governance dogfood gate invokes its delivered reaction against this workspace — the
//! strongest robustness statement a governance tool can make. The law it reacts against is [`shengmo::law::constitution`], declared in this
//! crate's `src/` because a declaration is code; what lives here is only what *runs* it.
//!
//! This is the crate-level upgrade of modou's module-level self-law: where modou could only
//! enforce `engine ⊥ runner` *within* one crate, 天衡 enforces the functional-core ⊥
//! imperative-shell split across *crate* boundaries.

use std::collections::BTreeSet;

use shengmo::law::{PREAMBLE, constitution, shell_dependency_boundary};
use shengmo::workspace::{manifest as workspace_manifest, root as workspace_root};
use tianheng::prelude::*;

#[test]
fn tianheng_governs_itself() {
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — self-governance runs in-repo only
    };
    GovernanceTest::for_constitution(constitution())
        .with_manifest_dir(manifest.parent().unwrap())
        .assert_clean();
}

#[test]
fn self_law_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — same repo-only discipline as the governance gate
    };
    GovernanceTest::for_constitution(constitution())
        .with_manifest_dir(&root)
        .assert_projection_fresh_with_preamble(
            root.join("AGENTS.self-law.md"),
            &format!("{PREAMBLE}\n"),
        );
}

/// Contract C — the **declaration-integrity** check (the 潛移/校讎-adjacent shape: its
/// observation source is the *declaration itself*, not governed code). A structural property of
/// `constitution()` is asserted, so a hand-written pointer to that property can be
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
/// check **fire** — a false refusal of a reason that genuinely states the law. A `because` that
/// carries the literal clause while *negating* it passes, and the projection then teaches the negation — the
/// false negative. A draft of this change declared the first as a false NEGATIVE, which one run of its own WHEN
/// falsified: a bound's extent is read off that run, never off the argument for it. Widening
/// `guibiao`'s allowlist to name `hunyi` left every test binary in this workspace green, with
/// `AGENTS.self-law.md` printing the sibling directly beneath the reason that forbids it. Freshness pinned the
/// projection against the declaration; nothing pinned the declaration against its own law.
/// The dimension crates, enumerated from the workspace rather than listed.
///
/// A dimension is a **published** crate that depends directly on 璇璣: the reaction model every dimension sits
/// above, which `PROJECT.md` states as the architecture and which a new dimension cannot avoid — a crate that
/// expressed findings in some other vocabulary would not be one. 璇璣 itself depends on no workspace member,
/// 星表 is the substrate beneath the dimensions and reaches the model through none, the shell composes them
/// and deliberately holds no direct edge to the model, and the two unpublished crates are governance rather
/// than product.
///
/// Read from tracked manifests, so an untracked scratch crate is neither a dimension nor a failure.
fn dimension_crates() -> BTreeSet<&'static str> {
    let root = workspace_root().expect("the workspace root this gate already located");
    let listing = std::process::Command::new("git")
        // `:(glob)` so `*` stops at the separator. git's default pathspec is fnmatch **without**
        // `FNM_PATHNAME`, so a bare `crates/*/Cargo.toml` crosses `/` and matches every manifest anywhere
        // beneath `crates/` — measured, 14 paths where 8 are crate manifests, the other six being test
        // fixtures. It returned the right answer only because no fixture happened to name 璇璣, and one
        // already names 星表 deliberately: `shell_metadata_edge` carries a workspace-member dependency
        // written so the fixture violates the edge under test. The next fixture that needs 璇璣 for the same
        // reason would have turned this gate red naming a fixture path.
        .args(["ls-files", ":(glob)crates/*/Cargo.toml"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files over the crate manifests");
    assert!(
        listing.status.success(),
        "`git ls-files` failed enumerating crate manifests, and a failed enumeration is not a workspace with \
         no crates"
    );
    let mut found = BTreeSet::new();
    for manifest in String::from_utf8_lossy(&listing.stdout).lines() {
        let text = std::fs::read_to_string(root.join(manifest))
            .unwrap_or_else(|error| panic!("cannot read tracked manifest {manifest}: {error}"));
        if text.contains("publish = false") {
            continue;
        }
        let deps = match text.split_once("\n[dependencies]\n") {
            Some((_, rest)) => rest.split("\n[").next().unwrap_or(rest),
            None => continue,
        };
        if !deps.lines().any(|line| line.starts_with("xuanji")) {
            continue;
        }
        let name = manifest
            .strip_prefix("crates/")
            .and_then(|rest| rest.strip_suffix("/Cargo.toml"))
            .expect("a crate manifest path names its crate");
        // A crate name carries no separator. The glob above already guarantees it; this refuses loudly if the
        // pathspec is ever loosened again, rather than letting a fixture path enter the set as a "dimension".
        assert!(
            !name.contains('/'),
            "the manifest enumeration reached {manifest}, which is not a crate manifest — the pathspec has \
             widened past the crate directories and a fixture would enter this comparison as a dimension"
        );
        found.insert(Box::leak(name.to_string().into_boxed_str()) as &'static str);
    }
    assert!(
        !found.is_empty(),
        "no dimension crate was enumerated, so this comparison would hold over nothing"
    );
    found
}

#[test]
fn dimension_boundaries_declare_the_mutual_independence_law() {
    const CLAUSE: &str = "三儀 ⊥ 三儀";
    // Held against an enumerator rather than kept by hand. A dimension born and not added here would have its
    // allowlist unchecked while this gate stayed green, and the set-coverage assertion below cannot notice
    // because `found` is produced by filtering on `expected` — measured, removing `guibiao` from the literal
    // left a `guibiao` allowlist naming `hunyi` green. The comparison below closes that.
    //
    // What still is not reached: `restrict_workspace_dependencies_to`, the more natural rule for this law.
    // `BACKLOG.md` carries that half.
    const DIMENSIONS: [&str; 3] = ["guibiao", "hunyi", "louke"];
    assert_eq!(
        DIMENSIONS.iter().copied().collect::<BTreeSet<&str>>(),
        dimension_crates(),
        "the dimensions this test judges are not the dimensions this workspace has — a 三儀 crate born and \
         not named here has its allowlist unchecked while the gate stays green"
    );

    let constitution = constitution();
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
             own `because` clause says so while the allowlist beside it allows it"
        );
    }
}

/// Contract D — the **declaration-integrity coverage** check (again the 潛移/校讎-adjacent
/// shape: its observation source is the *declaration and the workspace metadata*, not governed
/// code). Every workspace member must be the target of at least one boundary in
/// `constitution()`.
///
/// Without this, a crate added to the family with no self-governance boundary escapes the
/// dogfood gate **silently**: [`tianheng_governs_itself`] only reacts to crates a boundary
/// *names*, so an ungoverned member triggers no dependency reaction and could take any
/// dependency — heavy, cross-dimension, or the shell — undetected. That is a false negative of
/// the self-law itself (the one forbidden bug), and it is exactly the "all N crates are
/// governed" coverage claim that today is hand-restated across the docs
/// (`PROJECT.md`, `README.md`, `AGENTS.md`) rather than observed. Here the property is asserted
/// on the live `Constitution` + `cargo metadata`, so that claim need not be hand-counted — the
/// same move as Contract C (a prose index → a check), applied to coverage.
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
    GovernanceTest::for_constitution(constitution())
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
    let fixture = root.join("crates/shengmo/tests/fixtures/shell_metadata_edge/Cargo.toml");
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
    let fixture = root.join("crates/shengmo/tests/fixtures/cfg_if_violation/Cargo.toml");
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
    let fixture = root.join("crates/shengmo/tests/fixtures/glob_hazard_violation/Cargo.toml");
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
