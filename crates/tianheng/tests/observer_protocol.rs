//! `observer-protocol`'s reaction: the trait-driven fold and the built-in path are one verdict, each observer
//! declares exactly its dimension's bounds, and the fold's ordering directions hold.
//!
//! Two composition paths exist deliberately — the built-in one carries a coverage advisory the protocol cannot
//! and splitting its single `cargo metadata` read would double it — so the cost is paid here rather than
//! accepted: paths that could disagree silently are the drift a seam is supposed to end.

use std::path::{Path, PathBuf};

// Everything reaches this test through the shell, never through a direct edge to 璇璣: the shell's own
// dependency boundary allows guibiao, hunyi, louke, xingbiao and serde_json.
use tianheng::check_constitution;
use tianheng::prelude::*;

fn workspace_manifest() -> Option<PathBuf> {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../Cargo.toml");
    if manifest.is_file() {
        return Some(manifest);
    }
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "workspace manifest expected at {manifest:?} but absent while TIANHENG_WORKSPACE_TESTS is set — \
         the protocol's equality reaction must not silently skip in CI"
    );
    None
}

/// A constitution both paths can evaluate identically, in the one DSL shape the self-governance suite already
/// proves. Deliberately a crate boundary only: the comparison is about **composition**, not about which
/// boundaries were declared, and a richer fixture would risk failing for a reason that is not this seam's.
fn comparable_constitution() -> Constitution {
    Constitution::new("observer-protocol-equality").boundary(
        CrateBoundary::crate_("xuanji")
            .restrict_dependencies_to(["serde_json"])
            .because("璇璣 is the measure-only reaction model: serde_json only"),
    )
}

#[test]
fn the_trait_driven_fold_agrees_with_the_built_in_path() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let constitution = comparable_constitution();

    let built_in = check_constitution(&constitution, &manifest);
    let folded = Run::over(&manifest)
        .observe(StaticObserver::new(
            constitution.static_boundaries().clone(),
        ))
        .observe(SemanticObserver::new(
            constitution.semantic_boundaries().clone(),
        ))
        .observe(RuntimeObserver::new(
            constitution.runtime_boundaries().to_vec(),
        ))
        .verdict();

    assert_eq!(
        format!("{built_in:?}"),
        format!("{folded:?}"),
        "the two composition paths must produce one verdict; an additional entry that quietly judges \
         differently is worse than no entry at all"
    );
}

#[test]
fn every_observer_declares_exactly_its_dimension_s_bounds() {
    let ids = |bounds: Vec<BoundDecl>| -> Vec<String> {
        let mut ids: Vec<String> = bounds
            .into_iter()
            .map(|decl| decl.id().as_str().to_string())
            .collect();
        ids.sort();
        ids
    };
    // Delegation, never a second list: a divergent copy is exactly what the bijection installed by
    // `observation-bound-model` exists to refuse, and satisfying the protocol's obligation with one would be
    // declaring bounds nobody classified.
    assert_eq!(
        ids(StaticObserver::new(empty_constitution().static_boundaries().clone()).bounds()),
        ids(guibiao_bounds()),
        "圭表's observer declares its dimension's bounds"
    );
    assert_eq!(
        ids(SemanticObserver::new(empty_constitution().semantic_boundaries().clone()).bounds()),
        ids(hunyi_bounds()),
        "渾儀's observer declares its dimension's bounds"
    );
    assert_eq!(
        ids(RuntimeObserver::new(Vec::new()).bounds()),
        ids(louke_bounds()),
        "漏刻's observer declares its dimension's bounds"
    );
}

/// A constitution with no boundary: the bounds a dimension declares are a property of the dimension, not of
/// what an adopter happened to govern, so the fixture deliberately governs nothing.
fn empty_constitution() -> Constitution {
    Constitution::new("observer-protocol-bounds")
}

fn guibiao_bounds() -> Vec<BoundDecl> {
    guibiao::observation_bounds()
}
fn hunyi_bounds() -> Vec<BoundDecl> {
    hunyi::observation_bounds()
}
fn louke_bounds() -> Vec<BoundDecl> {
    louke::observation_bounds()
}

// --- the fold's ordering directions, on hand-written observers ---

struct Stub {
    outcome: Outcome,
    evaluated: std::cell::Cell<bool>,
}

impl Stub {
    fn new(outcome: Outcome) -> Self {
        Self {
            outcome,
            evaluated: std::cell::Cell::new(false),
        }
    }
}

impl Observer for &Stub {
    fn observe(&self, _manifest_path: &Path) -> Outcome {
        self.evaluated.set(true);
        self.outcome.clone()
    }

    fn bounds(&self) -> Vec<BoundDecl> {
        Vec::new()
    }
}

fn violating(rule: &str) -> Outcome {
    let fact = StructuredFactIdentity::new("probe", "fact", [("value", rule)])
        .expect("a well-formed fact identity");
    let id = ViolationId::new(
        "crate::probe",
        RuleKey::of("tianheng.rule/probe/policy", [("policy", rule)]),
        fact,
    );
    Outcome::Violations(Report::new(vec![Violation::new(
        BoundaryKind::Crate,
        id,
        rule,
        "crate::probe",
        "a stub observer's declared reason".to_string(),
        Severity::Enforce,
    )]))
}

#[test]
fn a_cannot_judge_stops_a_later_observer_being_evaluated() {
    let refuses = Stub::new(Outcome::ConstitutionError("first cannot judge".into()));
    let later = Stub::new(violating("must not import"));
    let verdict = Run::over(Path::new("Cargo.toml"))
        .observe(&refuses)
        .observe(&later)
        .verdict();

    assert!(
        matches!(verdict, Outcome::ConstitutionError(ref message) if message == "first cannot judge"),
        "a cannot-judge supersedes every violation: a verdict resting on a boundary that could not be \
         evaluated is not a verdict"
    );
    assert!(
        !later.evaluated.get(),
        "the later observer must not be evaluated at all — the short-circuit is a property of the fold, not \
         a filter on its result"
    );
}

#[test]
fn the_earlier_of_two_cannot_judges_wins_deterministically() {
    let first = Stub::new(Outcome::ConstitutionError("earlier".into()));
    let second = Stub::new(Outcome::ConstitutionError("later".into()));
    let verdict = Run::over(Path::new("Cargo.toml"))
        .observe(&first)
        .observe(&second)
        .verdict();
    assert!(
        matches!(verdict, Outcome::ConstitutionError(ref message) if message == "earlier"),
        "assembly order decides which cannot-judge is reported, and it is deterministic — that is why the \
         order is part of the contract rather than incidental"
    );
}

#[test]
fn violations_from_several_observers_merge_into_one_report() {
    let a = Stub::new(violating("must not import"));
    let b = Stub::new(violating("must not expose"));
    let verdict = Run::over(Path::new("Cargo.toml"))
        .observe(&a)
        .observe(&b)
        .verdict();
    match verdict {
        Outcome::Violations(report) => assert_eq!(
            report.violations.len(),
            2,
            "violations accumulate into one report, gated and baselined together"
        ),
        other => panic!("expected merged violations, got {other:?}"),
    }
}

#[test]
fn a_run_that_composed_no_observer_cannot_judge() {
    // Reporting clean here would be the vacuous pass this repository has re-opened most often: composing
    // nothing is a misconfiguration, not a clean workspace.
    let verdict = Run::over(Path::new("Cargo.toml")).verdict();
    assert!(
        matches!(verdict, Outcome::ConstitutionError(ref message) if message.contains("composed no observer")),
        "an empty run cannot judge, and says so"
    );
}

#[test]
fn every_clean_observer_folds_to_one_clean_outcome() {
    let a = Stub::new(Outcome::Clean);
    let b = Stub::new(Outcome::Clean);
    assert!(matches!(
        Run::over(Path::new("Cargo.toml"))
            .observe(&a)
            .observe(&b)
            .verdict(),
        Outcome::Clean
    ));
}

// --- this capability's own declared bounds, demonstrated ---

/// `observer-protocol/whether-an-observer-s-declared-bounds-are-complete-is-not-observed-a-stated-bound`
///
/// The trait compels a declaration, never a complete one. No reaction can enumerate the limits of a reaction it
/// did not write, so an observer declaring one of its two limits composes without complaint.
#[test]
fn an_observer_may_under_declare_its_bounds() {
    let under_declaring = Stub::new(Outcome::Clean);
    let verdict = Run::over(Path::new("Cargo.toml"))
        .observe(&under_declaring)
        .verdict();
    assert!(
        matches!(verdict, Outcome::Clean),
        "an observer declaring no bound at all still composes: the obligation is to answer the question, \
         which an empty answer does"
    );
    assert!(
        Observer::bounds(&&under_declaring).is_empty(),
        "the fixture must actually under-declare, or this bound is demonstrated by nothing"
    );
}

/// `observer-protocol/whether-an-observer-s-own-verdict-is-correct-is-not-observed-a-stated-bound`
///
/// The fold composes verdicts and does not adjudicate them; second-guessing each participant would need a second
/// implementation of every dimension.
#[test]
fn the_fold_does_not_adjudicate_a_participant_s_verdict() {
    // This observer reports a violation about a path that does not exist, against a manifest it never read.
    let inventing = Stub::new(violating("a rule about nothing"));
    let verdict = Run::over(Path::new("/nonexistent/Cargo.toml"))
        .observe(&inventing)
        .verdict();
    match verdict {
        Outcome::Violations(report) => assert_eq!(
            report.violations.len(),
            1,
            "the invented violation is merged as given — the fold trusts each participant's verdict"
        ),
        other => panic!("expected the verdict to be taken as given, got {other:?}"),
    }
}

/// The protocol introduces no trait object, asserted mechanically rather than trusted.
///
/// A collection-based entry taking `&[&dyn Observer]` was designed first and rejected on measurement: no module
/// of this crate is governed by a semantic boundary, and the `dyn`-trait DSL offers only forbid-all and
/// forbid-named-operands, so a declared exposure would have been a name with no reaction. The eager fold removes
/// the exposure instead of governing it — and this assertion is what keeps that true, since 渾儀 is not watching
/// this crate.
#[test]
fn composition_introduces_no_trait_object() {
    let src = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src");
    if !src.is_dir() {
        assert!(
            std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
            "{src:?} expected but absent while TIANHENG_WORKSPACE_TESTS is set"
        );
        return;
    }
    let mut offenders = Vec::new();
    let mut files = 0usize;
    for entry in std::fs::read_dir(&src).expect("the crate's source directory is readable") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        files += 1;
        let text = std::fs::read_to_string(&path).expect("a readable source file");
        for (number, line) in text.lines().enumerate() {
            let trimmed = line.trim_start();
            // Public signatures only. A `dyn` inside a private item is not an exposure, and a doc comment
            // mentioning one is prose.
            if trimmed.starts_with("pub ") && trimmed.contains(" dyn ") {
                offenders.push(format!("{}:{}: {trimmed}", path.display(), number + 1));
            }
        }
    }
    assert!(
        files > 0,
        "no source file was inspected, so this assertion would hold vacuously"
    );
    assert!(
        offenders.is_empty(),
        "the composed shell must expose no trait object; the protocol's own exposure was removed rather than \
         governed, because governing it was not available:\n{}",
        offenders.join("\n")
    );
}
