//! `observer-protocol`'s reaction: the trait-driven fold and the built-in path are one verdict, each observer
//! declares exactly its dimension's bounds, and the fold's ordering directions hold.
//!
//! Two composition paths exist deliberately for the static and semantic dimensions — the built-in one carries a
//! coverage advisory the protocol cannot, and splitting its single `cargo metadata` read would double it — so
//! the cost is paid here rather than accepted: paths that could disagree silently are the drift a seam is
//! supposed to end. For the **runtime** dimension there is no second path left to compare: the built-in one
//! delegates to `RuntimeObserver`, so equality there holds by construction, and what this file still observes
//! is that the fixture's runtime boundary reacts at all.
//!
//! Two of the properties below hold **by construction**, and each says which reaction stands in for the
//! comparison that would be inert. That is deliberate, and the alternative was worse: an assertion that cannot
//! fail reads exactly like a guarantee.

use std::path::{Path, PathBuf};

// Everything reaches this test through the shell, never through a direct edge to 璇璣: the shell's own
// dependency boundary allows guibiao, hunyi, louke, xingbiao and serde_json.
use tianheng::check_constitution;
use tianheng::prelude::*;

mod support;
use support::region::Source;

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

/// The workspace root, or `None` outside a checkout — the same skip-here / loud-in-CI discipline as above.
fn workspace_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    if root.join("crates").is_dir() {
        return Some(root);
    }
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "crates/ expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — the protocol's \
         delegation reaction must not silently skip in CI"
    );
    None
}

/// One dimension of 三儀, as both reactions below need it.
///
/// One array rather than three hand-written arms, because both reactions are three-way and a forgotten arm is
/// precisely an arm that silently proves nothing — which is the defect this shape exists to have closed. An
/// entry says everything either reaction needs about its dimension, so the fixture and the fold cannot come to
/// describe different dimension sets.
struct Dimension {
    label: &'static str,
    /// Declares a boundary of this dimension that this workspace **violates**. Measured, not reasoned; see
    /// `declare_*` below for what each one bites.
    declare: fn(Constitution) -> Constitution,
    /// Folds this dimension's observer into a run, reading its own boundaries back out of the constitution.
    fold: for<'a> fn(Run<'a>, &Constitution) -> Run<'a>,
    /// A violation of this kind proves this dimension's arm actually fired. A predicate rather than a
    /// `BoundaryKind`: 圭表 owns two kinds, and `BoundaryKind` is `#[non_exhaustive]` so a downstream crate
    /// cannot match it exhaustively anyway.
    reacted: fn(BoundaryKind) -> bool,
    /// Where this dimension's `Observer` impl is written, relative to the workspace root.
    ///
    /// A path rather than a `bounds()` call, because the obligation is about the *shape* of that method — see
    /// [`every_observer_declares_exactly_its_dimension_s_bounds`].
    observer_source: &'static str,
}

/// 三儀, in the order the built-in path assembles them.
///
/// **The order is part of the comparison, not cosmetic.** `Run::observe` folds eagerly and `merge_outcomes`
/// concatenates violations in fold order, so the two `Debug` renderings compared below only match while this
/// array is in `evaluate_constitution`'s order: 圭表, 渾儀, 漏刻. Sorting it would break the equality reaction
/// without any dimension having changed.
const DIMENSIONS: [Dimension; 3] = [
    Dimension {
        label: "圭表 (static)",
        declare: declare_violated_static,
        fold: |run, constitution| {
            run.observe(StaticObserver::new(
                constitution.static_boundaries().clone(),
            ))
        },
        reacted: |kind| matches!(kind, BoundaryKind::Crate | BoundaryKind::Module),
        observer_source: "crates/guibiao/src/observer.rs",
    },
    Dimension {
        label: "渾儀 (semantic)",
        declare: declare_violated_semantic,
        fold: |run, constitution| {
            run.observe(SemanticObserver::new(
                constitution.semantic_boundaries().clone(),
            ))
        },
        reacted: |kind| matches!(kind, BoundaryKind::Semantic),
        observer_source: "crates/hunyi/src/observer.rs",
    },
    Dimension {
        label: "漏刻 (runtime)",
        declare: declare_violated_runtime,
        fold: |run, constitution| {
            run.observe(RuntimeObserver::new(
                constitution.runtime_boundaries().to_vec(),
            ))
        },
        reacted: |kind| matches!(kind, BoundaryKind::Runtime),
        observer_source: "crates/louke/src/observer.rs",
    },
];

/// 璇璣's real `serde_json` edge falls outside an allowlist holding only `syn`, which it does not depend on.
///
/// An **empty** allowlist was tried first and reads as clean, which is why every dimension's reaction is
/// asserted below rather than assumed from the declaration looking violating.
fn declare_violated_static(constitution: Constitution) -> Constitution {
    constitution.boundary(
        CrateBoundary::crate_("xuanji")
            .restrict_dependencies_to(["syn"])
            .because(
                "a deliberately violated boundary, so the compared verdict is not trivially clean",
            ),
    )
}

/// 渾儀's own `SemanticBoundaries::crate_packages` returns `impl Iterator<Item = &str>`.
///
/// The narrowest reacting semantic declaration found by running candidates: exactly one violation, from one
/// public method. A visibility ceiling on 璇璣's root also reacts and produces eight, which makes a failure
/// message harder to read while proving nothing more.
fn declare_violated_semantic(constitution: Constitution) -> Constitution {
    constitution.impl_trait_boundary(
        ImplTraitBoundary::in_crate("hunyi")
            .module("crate")
            .must_not_expose_impl_trait()
            .because(
                "a deliberately violated boundary, so the semantic arm is not compared vacuously",
            ),
    )
}

/// A seam name no probe in this tree writes, so the audit reacts declared-but-unprobed.
///
/// Chosen because it cannot become accidentally satisfied: the only way to stop this reacting is to add a probe
/// for a seam invented for this fixture. An **empty** runtime declaration was measured first and is `Clean` on
/// this workspace — the very hole this array closes.
fn declare_violated_runtime(constitution: Constitution) -> Constitution {
    constitution.runtime(
        RuntimeBoundary::at("observer-protocol-equality-unprobed-seam")
            .only_origins(["tianheng"])
            .because(
                "a deliberately violated boundary, so the runtime arm is not compared vacuously",
            ),
    )
}

/// A constitution every dimension of 三儀 evaluates to a **violation of its own kind**.
///
/// Deliberately violating in *each* dimension, not just overall. A dimension whose declared set is empty
/// contributes nothing to either side of the comparison, so the two paths agree for it however wrongly one of
/// them behaves — measured: an empty constitution is `Clean` here, and with the previous static-only fixture,
/// replacing `SemanticObserver::observe`'s body with `Outcome::Clean` left this suite passing.
fn comparable_constitution() -> Constitution {
    let mut constitution = Constitution::new("observer-protocol-equality");
    for dimension in &DIMENSIONS {
        constitution = (dimension.declare)(constitution);
    }
    // The guard against an entry being deleted from `DIMENSIONS`: a deleted entry leaves its dimension's
    // accessor empty, and that dimension is then compared vacuously again. Checked against the constitution
    // rather than by asserting the array's length beside the array, which is the same hand-kept census.
    assert!(
        !constitution.static_boundaries().boundaries().is_empty(),
        "圭表 declares nothing — the static arm would be compared vacuously"
    );
    assert!(
        !constitution.semantic_boundaries().is_empty(),
        "渾儀 declares nothing — the semantic arm would be compared vacuously"
    );
    assert!(
        !constitution.runtime_boundaries().is_empty(),
        "漏刻 declares nothing — the runtime arm would be compared vacuously"
    );
    constitution
}

#[test]
fn the_trait_driven_fold_agrees_with_the_built_in_path() {
    let Some(manifest) = workspace_manifest() else {
        return;
    };
    let constitution = comparable_constitution();

    let built_in = check_constitution(&constitution, &manifest);
    let mut run = Run::over(&manifest);
    for dimension in &DIMENSIONS {
        run = (dimension.fold)(run, &constitution);
    }
    let folded = run.verdict();

    // The comparison must not be able to hold vacuously in ANY ONE dimension. The earlier form asserted only
    // that the whole verdict was a violation, which a single reacting dimension satisfies while the other two
    // compare `Clean` against `Clean`. Reaction is therefore checked per dimension, and a fixture that goes
    // vacuous because the workspace changed under it fails here naming the dimension to repair.
    let Outcome::Violations(report) = &built_in else {
        panic!("the fixture must react, or comparing the two paths proves nothing: {built_in:?}");
    };
    for dimension in &DIMENSIONS {
        assert!(
            report
                .violations
                .iter()
                .any(|violation| (dimension.reacted)(violation.kind)),
            "{} did not react, so the comparison proves nothing about it — repair the fixture's declaration \
             for this dimension, not either path: {report:?}",
            dimension.label
        );
    }
    assert_eq!(
        format!("{built_in:?}"),
        format!("{folded:?}"),
        "the two composition paths must produce one verdict; an additional entry that quietly judges \
         differently is worse than no entry at all"
    );
}

/// Each observer's `bounds()` is **exactly a delegation** to its dimension's exported declarations.
///
/// This replaces a comparison that could not fail. Every `bounds()` already *is* `observation_bounds()`, so
/// asserting `observer.bounds() == dimension::observation_bounds()` compared one function with itself —
/// measured: drifting a declaration's extent with its id untouched left this suite at 10 passed. Comparing
/// whole `BoundDecl`s instead of ids would have been a better comparison of two identical things, and still
/// inert.
///
/// What the requirement actually fears is a **second, divergent list** — and a second list is something written
/// in the body. So the property is the body's shape, which can fail: write a `vec![...]` there and this reaction
/// reports it. The declarations' *content* is held elsewhere and does not need re-asserting here: drifting an
/// extent fails `observation_bound_model`'s `the_extent_projection_is_fresh`, verified by the same perturbation.
///
/// Recognized by **position**, never by the bare call appearing somewhere in the file: the body between
/// `fn bounds`'s brace and its closing brace must hold one executed statement, and that statement must be the
/// call. A file that merely mentions `observation_bounds()` elsewhere — as every one of these does, in `use` —
/// satisfies nothing.
#[test]
fn every_observer_declares_exactly_its_dimension_s_bounds() {
    let Some(root) = workspace_root() else {
        return;
    };
    for dimension in &DIMENSIONS {
        let path = root.join(dimension.observer_source);
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("cannot read {path:?}: {error}"));
        let source = Source::of(text);
        let body = bounds_body(&source).unwrap_or_else(|| {
            panic!(
                "{} has no `fn bounds` body in {} — the protocol's obligation is about that method, so its \
                 absence is a cannot-judge, not a pass",
                dimension.label, dimension.observer_source
            )
        });
        assert_eq!(
            body.iter().map(String::as_str).collect::<Vec<_>>(),
            vec![DELEGATION],
            "{}'s `bounds()` must be exactly `{DELEGATION}` — the obligation is satisfied by delegating to the \
             dimension's exported declarations, and a body holding anything else is the second, divergent list \
             the bijection refuses ({})",
            dimension.label,
            dimension.observer_source
        );
    }
}

/// The one statement a conforming `bounds()` body holds.
const DELEGATION: &str = "observation_bounds()";

/// The executed statements inside `fn bounds`'s body, or `None` if the method is absent.
///
/// Brace-counted from the signature's opening brace, so a nested block inside the body would be included rather
/// than truncating at the first `}` — the body is required to be one statement, but a *wrong* body must be
/// reported whole rather than mis-parsed into looking right.
fn bounds_body(source: &Source) -> Option<Vec<String>> {
    let text = source.whole();
    let signature = text.find("fn bounds(")?;
    let open = signature + text[signature..].find('{')?;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, character) in text[open..].char_indices() {
        match character {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    close = Some(open + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let body = Source::of(&text[open + 1..close?]);
    Some(
        body.executed()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                // Written as a tail expression today; a `return …;` says the same thing and must read the same.
                line.trim_start_matches("return ")
                    .trim_end_matches(';')
                    .trim()
                    .to_string()
            })
            .collect(),
    )
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

/// Whether one line of source declares a publicly exposed trait object.
///
/// A named recognizer over **one line**, so its limit can be demonstrated by giving it text rather than by
/// rewriting this crate — see [`a_trait_object_on_a_continuation_line_is_not_recognized`], which pins the
/// declared bound this shape carries.
///
/// Two decisions inside it, both paid for:
///
///   * **`pub ` prefix.** A `dyn` inside a private item is not an exposure, and a doc comment mentioning one is
///     prose. It over-approximates in the safe direction: it cannot tell a `pub` item in a private module from a
///     reachable one and flags both, because a false positive here is a sentence to write while a false negative
///     is an exposure nobody governs.
///   * **`dyn ` anywhere on the line, never ` dyn `.** `Box<dyn T>` is the commonest exposure and reads `<dyn`,
///     which a space-prefixed matcher silently misses. Measured: an injected `pub fn … -> Vec<Box<dyn Observer>>`
///     passed the earlier pattern.
fn exposes_a_trait_object(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("pub ") && trimmed.contains("dyn ")
}

/// The protocol introduces no trait object, asserted mechanically rather than trusted.
///
/// A collection-based entry taking `&[&dyn Observer]` was designed first and rejected on measurement: no module
/// of this crate is governed by a semantic boundary, and the `dyn`-trait DSL offers only forbid-all and
/// forbid-named-operands, so a declared exposure would have been a name with no reaction. The eager fold removes
/// the exposure instead of governing it — and this assertion is what keeps that true, since 渾儀 is not watching
/// this crate.
///
/// It reads this crate's **top-level** source files only, and that is sound because of a premise this test now
/// checks rather than assumes: every subdirectory of `src/` is reached through a non-`pub` `mod`, so nothing
/// beneath one is reachable from outside the crate. Measured, the eight files under `src/runner/` are never
/// opened here and an injected `pub fn … -> Option<Box<dyn Debug>>` among them leaves this passing — harmless
/// while those modules are private, and invisible the moment one is not.
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
    let mut subdirectories = Vec::new();
    for entry in std::fs::read_dir(&src).expect("the crate's source directory is readable") {
        let path = entry.expect("a readable directory entry").path();
        if path.is_dir() {
            subdirectories.push(path);
            continue;
        }
        if path.extension().is_none_or(|ext| ext != "rs") {
            continue;
        }
        files += 1;
        let text = std::fs::read_to_string(&path).expect("a readable source file");
        for (number, line) in text.lines().enumerate() {
            if exposes_a_trait_object(line) {
                offenders.push(format!(
                    "{}:{}: {}",
                    path.display(),
                    number + 1,
                    line.trim_start()
                ));
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

    // The premise, checked. Reading only the top level is sound exactly while every subdirectory sits behind a
    // private `mod`; making one public would take its files out of this reaction's reach with nothing to say so,
    // which is the difference between a bounded reaction and one that quietly shrank.
    for directory in &subdirectories {
        let name = directory
            .file_name()
            .and_then(|name| name.to_str())
            .expect("a source subdirectory has a UTF-8 name");
        // A subdirectory `foo` is declared either in `src/foo.rs` or in `src/lib.rs`.
        let declaring = [src.join(format!("{name}.rs")), src.join("lib.rs")];
        let declaration = declaring
            .iter()
            .filter_map(|candidate| std::fs::read_to_string(candidate).ok())
            .find_map(|text| {
                text.lines()
                    .map(str::trim_start)
                    .find(|line| {
                        line.starts_with(&format!("mod {name};"))
                            || line.starts_with(&format!("pub mod {name};"))
                            || line.starts_with(&format!("pub(crate) mod {name};"))
                    })
                    .map(str::to_string)
            })
            .unwrap_or_else(|| {
                panic!(
                    "no `mod {name};` declaration found for the source subdirectory {directory:?} — this \
                     reaction reads only the top level, and it cannot judge that soundness without finding \
                     how the subdirectory is reached"
                )
            });
        assert!(
            !declaration.starts_with("pub mod "),
            "`{declaration}` makes {directory:?} publicly reachable, so this reaction's premise no longer \
             holds: it reads only this crate's top-level files, and the ones beneath that module would leave \
             its reach unnoticed. Recurse into subdirectories, or keep the module private."
        );
    }
}

/// The declared bound: the recognizer reads **one line**, so a wrapped signature's continuation is invisible.
///
/// Pinned by giving the recognizer text rather than by rewriting this crate — which is why
/// [`exposes_a_trait_object`] is a named function at all. The control matters as much as the bound: the same
/// exposure written on **one** line *is* recognized, so this test shows a limit of the line split rather than a
/// recognizer that never fires.
///
/// Closing it needs 渾儀 watching this crate, and that was measured to be unavailable: no module here carries a
/// semantic boundary, and the `dyn`-trait DSL offers only forbid-all and forbid-named-operands, so the
/// declaration would have been a name with no reaction. Hence a stated bound rather than a fix.
#[test]
fn a_trait_object_on_a_continuation_line_is_not_recognized() {
    assert!(
        exposes_a_trait_object("pub fn participants() -> Vec<Box<dyn Observer>> {"),
        "the control: on one line, this exposure is recognized"
    );
    // The same signature, wrapped. The marker is on the `pub fn` line and the exposure on the next, and the
    // recognizer sees neither line as an exposure.
    assert!(
        !exposes_a_trait_object("pub fn participants("),
        "the signature's first line names no trait object"
    );
    assert!(
        !exposes_a_trait_object(") -> Vec<Box<dyn Observer>> {"),
        "and its continuation carries the trait object without the `pub ` the recognizer needs — the stated bound"
    );
}
