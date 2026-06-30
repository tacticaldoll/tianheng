//! Tianheng governs itself with its own reaction — the strongest robustness statement
//! a governance tool can make. Its architectural invariants are not prose in PROJECT.md
//! alone; they are declared here as a real constitution ([`tianheng_constitution`]) and
//! run as a `cargo test` gate, so CI fails the moment the law drifts.
//!
//! This is the crate-level upgrade of modou's module-level self-law: where modou could
//! only enforce `engine ⊥ runner` *within* one crate, Tianheng enforces the
//! functional-core ⊥ imperative-shell split across *crate* boundaries.

use std::path::PathBuf;

use tianheng::prelude::*;
use tianheng::{GnomonConstitution, constitution_markdown};

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
/// a hope. Seven boundaries, plus one cross-cutting law they jointly realize:
///
/// 1. **璇璣 is the bedrock** — the shared reaction model (`xuanji`) depends on
///    `serde_json` only and on no workspace member, so the whole family sits *above* it
///    and nothing leaks *into* the dimension-agnostic model (no engine, no shell).
/// 2. **Dependency-light core** — the 圭表 core (`guibiao`) takes `serde_json` plus the
///    internal `xuanji` only. Heavy AST/runtime dependencies are quarantined to their own
///    crates, never the core — which is why the static source scanner is hand-rolled,
///    not `syn`.
/// 3. **syn is quarantined to 渾儀** — the semantic dimension (`hunyi`) is the only
///    crate that may depend on `syn`; it takes `xuanji` + `serde_json` + `syn` and never
///    the 圭表 engine or the 天衡 shell (functional dimension ⊥ imperative shell).
/// 4. **Functional core ⊥ imperative shell**, at crate granularity — `guibiao` must
///    not depend on the 天衡 shell (`tianheng`). The core never reaches for the shell.
/// 5. **Bounded shell** — the 天衡 shell depends only on the dimensions it composes
///    (圭表 `guibiao` + 渾儀 `hunyi` + 漏刻 `louke`, whose CI probe-coverage face it folds into
///    `check`) and `serde_json`.
/// 6. **漏刻 ships light** — the 漏刻 runtime dimension (`louke`) depends on 璇璣 (`xuanji`)
///    only: no `syn`, no static engine, no sibling dimension, because it ships into the user's
///    production binary (`serde_json` reaches it only transitively via 璇璣, cold-path only).
///
/// **Cross-cutting — 三儀 ⊥ 三儀 (the dimensions are mutually independent).** The
/// observation dimensions — 圭表 (static), 渾儀 (semantic), and 漏刻 (runtime) — never depend
/// on one another; each sits on 璇璣 and is composed into one reaction *only* by the 天衡 shell
/// (for the CI dimensions) or reacts independently in prod (漏刻), never via a sibling. This
/// law is **named here and in each dimension's `because`** (boundaries 2, 4, and 6), so a
/// constitution reader and the 垂象 report both see the intent. It adds **no separate
/// boundary on purpose: a dimension's `restrict_dependencies_to` allowlist names no sibling
/// dimension, so a cross-dimension dependency already reacts. A `forbid_dependency_on`
/// between dimensions would be a second reaction for a drift the allowlist already catches
/// — and an allowlist is always stricter than a denylist, so it would add zero protection.
/// Minimalism forbids the redundant reaction; the law is made *visible*, not re-enforced.
///
/// A wrong boundary here is fixed by a human-reviewed amendment, never by quietly
/// weakening this function to make CI pass.
fn tianheng_constitution() -> GnomonConstitution {
    GnomonConstitution::new("tianheng")
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
            CrateBoundary::crate_("guibiao")
                .restrict_dependencies_to(["serde_json", "xuanji"])
                .because(
                    "the 圭表 core stays dependency-light: serde_json is the only external \
                     dependency (no syn / proc-macro, no heavy graph or runtime crates); the \
                     internal dependency on 璇璣 (the shared reaction model) is the price of \
                     the family split — the model carries no engine. 三儀 ⊥ 三儀: this allowlist \
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
                .restrict_dependencies_to(["xuanji", "serde_json", "syn"])
                .because(
                    "渾儀 is the semantic dimension and the sole holder of the heavy syn AST \
                     dependency — quarantined here, never the core or the model; it depends on \
                     璇璣 (the reaction model), serde_json, and syn only. 三儀 ⊥ 三儀: it never \
                     depends on the sibling 圭表 dimension (nor, when born, 漏刻), and never on \
                     the 天衡 shell — the dimensions are composed only by the shell, never by \
                     each other (functional dimension ⊥ imperative shell)",
                ),
        )
        .boundary(
            CrateBoundary::crate_("louke")
                .restrict_dependencies_to(["xuanji"])
                .because(
                    "漏刻 is the runtime dimension and ships into the user's production binary, \
                     so it stays production-light: it depends on 璇璣 (the reaction model) only \
                     — no syn, no static engine, no sibling dimension. 三儀 ⊥ 三儀: naming no \
                     sibling, it cannot depend on the 圭表/渾儀 dimensions, and it reacts in prod \
                     independently of the 天衡 shell (serde_json reaches it only transitively via \
                     璇璣, cold-path only)",
                ),
        )
        .boundary(
            CrateBoundary::crate_("tianheng")
                .restrict_dependencies_to(["guibiao", "hunyi", "louke", "serde_json"])
                .because(
                    "the 天衡 shell composes the 三儀 into one reaction, so it depends on the 圭表 \
                     static core, the 渾儀 semantic dimension, and the 漏刻 runtime dimension (whose \
                     CI probe-coverage face it composes into `check`), plus serde_json; the gate \
                     stands on every dimension it gates",
                ),
        )
}

#[test]
fn tianheng_governs_itself() {
    // The whole self-constitution reacts against the workspace. Any drift — a new
    // external dependency, or the core depending on the shell — surfaces here as a
    // `cargo test` failure, with the offending boundary's reason as the repair hint.
    let Some(manifest) = workspace_manifest() else {
        return; // no workspace root (e.g. a packaged crate) — self-governance runs in-repo only
    };
    let outcome = check(&tianheng_constitution(), &manifest);
    assert!(
        matches!(outcome, Outcome::Clean),
        "Tianheng's self-constitution drifted: {outcome:?}"
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
    let projection = constitution_markdown(&Constitution::from(tianheng_constitution()));
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

    if std::env::var_os("BLESS").is_some() {
        std::fs::write(&path, &live).expect("write AGENTS.self-law.md");
        return;
    }

    let checked_in = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        panic!(
            "read {}: {e} — generate it with `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`",
            path.display()
        )
    });
    assert_eq!(
        checked_in, live,
        "AGENTS.self-law.md is stale; regenerate it with \
         `BLESS=1 cargo test -p tianheng self_law_projection_is_fresh`"
    );
}
