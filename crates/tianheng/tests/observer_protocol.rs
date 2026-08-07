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

// Everything reaches this test through the shell; a direct edge to 璇璣 would breach the shell's
// self-governed dependency direction.
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
    // Compared by VALUE, not by rendered `Debug` text. `Outcome` derives `PartialEq`, so the comparison the
    // requirement asks for is available directly; going through `format!` asked a reader to trust that two
    // distinct outcomes cannot render alike, and gave a failure one long line instead of two structures.
    assert_eq!(
        built_in, folded,
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
                "{} yields no `fn bounds` body in {}: {} — the protocol's obligation is about that method, so \
                 neither its absence nor an ambiguous anchor is a pass",
                dimension.label,
                dimension.observer_source,
                decline_reason(&source, "fn bounds(")
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

/// A source with no `fn bounds` is a **refusal to judge**, not a pass.
///
/// The reaction panics naming the dimension when the method is absent from the file its array entry points at.
/// That path is unreachable for a conforming `Observer` — the trait requires the method, so a rename fails to
/// compile — and reachable the moment an impl moves to a file the array does not name. Asserted here because the
/// scenario states it: a reaction that finds nothing to read has not observed that the obligation holds, and the
/// distinction between "no body" and "an empty body" is what decides whether it refuses or reports.
#[test]
fn a_source_with_no_bounds_method_yields_no_body_to_judge() {
    assert!(
        bounds_body(&Source::of("fn other() -> u8 { 0 }\n")).is_none(),
        "no `fn bounds` means nothing to judge, which the reaction turns into a refusal rather than a pass"
    );
    // The discriminator: a body that exists and is EMPTY is `Some(vec![])`, which the reaction reports as an
    // offence. Without this, the assertion above would also hold for a recognizer that never finds anything.
    assert_eq!(
        bounds_body(&Source::of("fn bounds(&self) -> Vec<BoundDecl> {\n}\n")),
        Some(Vec::new()),
        "an empty body is found and judged, so absence and emptiness are distinguished"
    );
}

/// A `}` written in a comment TAIL does not close the body, so a second list behind one is still read.
///
/// The truncation this refuses was silent in the one direction that matters: `observation_bounds(); // }`
/// closed the body at the comment, `bounds_body`'s own `//`-tail stripping turned the remainder into exactly
/// the delegation, and a `Vec::new()` beneath it — a second list — was never presented to the assertion. The
/// repair is ordering: the tail is stripped *before* the braces are counted, not after.
///
/// The control is the second case. Without it a masker that blanked every brace everywhere would satisfy the
/// first assertion and look like a fix, while making every body unclosable.
#[test]
fn a_brace_in_a_comment_tail_no_longer_closes_the_body() {
    let hidden_second_list = Source::of(
        "fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds(); // }\n    Vec::new()\n}\n",
    );
    assert_eq!(
        bounds_body(&hidden_second_list).as_deref(),
        Some(["observation_bounds()".to_string(), "Vec::new()".to_string()].as_slice()),
        "the body runs to its real closing brace, so the second list is what the reaction judges"
    );

    let delegation_with_a_comment =
        Source::of("fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds() // why\n}\n");
    assert_eq!(
        bounds_body(&delegation_with_a_comment).as_deref(),
        Some(["observation_bounds()".to_string()].as_slice()),
        "and a conforming body carrying an ordinary comment still resolves — the mask blanks braces inside a \
         tail, never the tail's own line"
    );
}

/// A brace inside a block comment or a string literal still moves the body extent — a declared bound.
///
/// Recognizing it would need the string-literal lexing this file deliberately does not carry, and which
/// `check_bound_register.sh` measured and rejected for the same reason: this tree's own lexer suites put
/// comment delimiters inside string literals, several of them nested, so a delimiter-counting stripper opens a
/// phantom comment at the first of them.
///
/// It is declared rather than closed because **for this comparison** the error direction is the safe one, which
/// this pin is what shows: a moved extent makes a **conforming** body read as non-conforming, because no
/// brace-carrying construct survives the exact one-statement comparison. An author meets a refusal to argue
/// with, never a silent pass. The control is the same body with the comment removed, so the refusal is the
/// brace's doing and not the recognizer refusing everything.
///
/// The direction belongs to the comparison and not to the extent, and reading it as a property of the extent is
/// how the same moved extent went four windows accepting a divergent body elsewhere: the shell-delegation
/// reaction holds its body to an allowlist whose members a truncated remainder can still satisfy in full, so
/// what falls past the cut is simply not there to refuse. That reader refuses rather than inheriting this
/// bound — see
/// [`an_ambiguous_delegation_extent_is_refused_rather_than_judged`].
#[test]
fn a_brace_in_a_block_comment_moves_the_body_extent() {
    let braced_block_comment = Source::of(
        "fn bounds(&self) -> Vec<BoundDecl> {\n    /* } */\n    observation_bounds()\n}\n",
    );
    assert_ne!(
        bounds_body(&braced_block_comment).as_deref(),
        Some(["observation_bounds()".to_string()].as_slice()),
        "the extent stops at the commented brace, so this body — which delegates exactly — is refused; that \
         over-reaction is the declared bound"
    );

    let same_body_uncommented =
        Source::of("fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n");
    assert_eq!(
        bounds_body(&same_body_uncommented).as_deref(),
        Some(["observation_bounds()".to_string()].as_slice()),
        "the identical body without the comment resolves, so the bound is about the brace and not about the \
         recognizer refusing whatever it is given"
    );
}

/// What the reaction concluded about a composition body's semantic delegation.
///
/// `CannotJudge` is a distinct answer rather than a flavour of failure, because the two say opposite things to
/// whoever reads the reaction: one is a claim about the shell's code, the other a claim about this recognizer's
/// reach.
#[derive(Debug, PartialEq, Eq)]
enum Delegation {
    /// One semantic-boundary access, and it is the direct `hunyi::check_all` argument.
    Delegates,
    /// The body was read and does not delegate as the requirement demands.
    Diverges(String),
    /// The extent read may not be the function's body, so no verdict is given.
    CannotJudge(String),
}

/// Delimiters that can hide a brace from the extent count, so their presence makes the extent unsafe to judge.
///
/// A `"` or a `'` opens a literal whose braces are text; `/*` opens a comment whose braces are text too. Each
/// of the three is exercised by a route in
/// [`an_ambiguous_delegation_extent_is_refused_rather_than_judged`], and each is load-bearing: removing any one
/// re-opens its route, measured by removing it.
///
/// `*/` was listed here and is **not**, because no shape reaches it. A block comment opened above the signature
/// closes *after* the brace it hides, so the extent is cut before the `*/` and carries no delimiter at all —
/// see [`a_truncation_carrying_no_delimiter_still_fails_loudly`] for what that leaves. A member no route
/// exercises is a declared set drifting from its enumerator: it can be deleted with every document still
/// agreeing and the suite still green, which is the one property a declared set must not have.
const EXTENT_AMBIGUITY: [&str; 3] = ["\"", "'", "/*"];

/// One executed line with any line-comment tail removed.
///
/// The brace count already treats text after `//` as prose — [`mask_line_comment_braces`] blanks braces there —
/// so the ambiguity scan must agree, or the two disagree about what the body even is. It did disagree:
/// [`Executed`] filters only lines whose *trimmed start* is `//`, so a tail comment's line survives whole, and
/// an apostrophe in ordinary English prose (`// the shell's one access`) refused a body whose code carries no
/// literal at all. That is over-refusal on text that cannot move a brace, one `rustfmt` reflow away from the
/// tracked file.
fn without_comment_tail(line: &str) -> &str {
    match line.find("//") {
        Some(index) => &line[..index],
        None => line,
    }
}

/// Whether the shell's composition body delegates semantic emptiness, or `None` if no line anchors the
/// function or more than one does.
///
/// Split out of the reaction so it can be driven by a fixture. Reading only the tracked `runner.rs` left the
/// reaction with no negative run available: every claim about what it refuses was a claim about text that does
/// not exist, which is how it went four release windows accepting a body it was written to refuse.
///
/// **The ambiguity check runs before the comparison, not after.** A truncated extent can satisfy the allowlist
/// in full — every permitted owner present exactly once — while the use that breaks the requirement sits past
/// the cut, absent from what is checked, so the one shape this reaction refuses reads as the delegation it
/// demands. Checking afterwards would be checking a verdict already formed on the wrong text, and the
/// difference is observable rather than stylistic: a body both truncated *and* divergent within the remainder
/// answers `Diverges` under an escalate-a-pass ordering and `CannotJudge` under this one. That is the shape
/// [`an_ambiguous_extent_is_refused_even_when_the_remainder_looks_divergent`] holds, because a bolded claim
/// with no reaction is the drift this file exists to react to.
fn judge_delegation(source: &Source) -> Option<Delegation> {
    let body = function_body(source, "fn evaluate_constitution(")?;
    if let Some(delimiter) = EXTENT_AMBIGUITY.iter().find(|delimiter| {
        body.rust()
            .lines()
            .any(|line| without_comment_tail(line).contains(*delimiter))
    }) {
        return Some(Delegation::CannotJudge((*delimiter).to_string()));
    }
    // The allowlist walk reads the executed text with its **whitespace intact**, and the delegation
    // containment below reads the compacted form. That split is not tidiness: compaction removes the token
    // boundaries this walk depends on, gluing `if constitution` into `ifconstitution` so the occurrence reads
    // as the tail of a longer identifier and disappears — measured, it hid a direct read of the private
    // `semantic` field. Compaction exists only so `rustfmt` may break the delegation call, which is a
    // different question asked of a different string.
    let executed: Vec<String> = body
        .rust()
        .lines()
        .map(|line| without_comment_tail(line).to_string())
        .collect();
    let executed = fold_broken_method_chains(&executed.join("\n"));
    for use_of_parameter in whole_identifier_occurrences(&executed, COMPOSITION_PARAMETER) {
        if !PERMITTED_USES
            .iter()
            .any(|permitted| executed[use_of_parameter..].starts_with(permitted))
        {
            let offending: String = executed[use_of_parameter..]
                .chars()
                .take_while(|c| *c != '\n')
                .take(56)
                .collect();
            return Some(Delegation::Diverges(format!(
                "`{COMPOSITION_PARAMETER}` is used outside the permitted owners, at `{offending}`"
            )));
        }
    }
    for permitted in PERMITTED_USES {
        let uses = whole_identifier_occurrences(&executed, COMPOSITION_PARAMETER)
            .filter(|at| executed[*at..].starts_with(permitted))
            .count();
        if uses != 1 {
            return Some(Delegation::Diverges(format!(
                "`{permitted}` appears {uses} time(s), and the composition admits exactly one owner per \
                 dimension"
            )));
        }
    }
    let compact = compact_executed_rust(&body);
    if !contains_token_bounded(&compact, SEMANTIC_DELEGATION) {
        return Some(Delegation::Diverges(
            "the access is not the direct `hunyi::check_all` argument".to_string(),
        ));
    }
    Some(Delegation::Delegates)
}

/// The composition function's parameter, whose every use the allowlist below governs.
const COMPOSITION_PARAMETER: &str = "constitution";

/// The complete set of things the composition body may do with its constitution — one owner per dimension.
///
/// **An allowlist, and that is the whole point.** Counting reaches for the semantic boundaries was a denylist
/// over *spellings*, and spellings are an open set: a rebinding, an associated-function call, a reborrow, a
/// wrapper function, a trait method with another name, a macro, and — needing no accessor at all — a direct
/// read of the private `semantic` field, which a descendant module of the crate root may simply do. Each was
/// measured accepted; each needed its own patch; the next one was always one review away.
///
/// Permitting three uses and refusing everything else closes all of them at once, because the escape now
/// requires the body not to mention `constitution` — and then it has no constitution to decide with. Implicit
/// deny is what makes the enumeration complete: the forbidden set is not listed, it is the complement.
///
/// The cost is deliberate. A fourth dimension, or any other legitimate reach for the constitution, fails here
/// until it is admitted — which is the amendment discipline this repository wants for exactly that change,
/// not an accident of the recognizer.
const PERMITTED_USES: [&str; 3] = [
    "constitution.static_boundaries()",
    "constitution.semantic_boundaries()",
    "constitution.runtime_boundaries()",
];

/// The same text with a method chain that `rustfmt` broke across lines joined back onto one.
///
/// Only a newline whose next non-space character is `.` is closed up, so whitespace **between identifiers** is
/// untouched and the token boundaries the allowlist walk depends on survive. That is the whole difference from
/// [`compact_executed_rust`], which removes every space and thereby glues a keyword to the parameter.
///
/// Without this, `rustfmt` alone turns a conforming body into a reported violation: growing the runtime chain
/// past the line width makes it emit `constitution\n    .runtime_boundaries()`, which matches no permitted owner
/// and is reported as an independent shell decision. A gate that fires because the formatter this repository
/// also gates on reformatted the file is a false accusation, and the diagnostic names the wrong fault.
fn fold_broken_method_chains(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for (index, line) in text.lines().enumerate() {
        if line.trim_start().starts_with('.') {
            out.push_str(line.trim_start());
            continue;
        }
        if index > 0 {
            out.push('\n');
        }
        out.push_str(line);
    }
    out
}

/// Byte offsets where `identifier` occurs in `text` as a whole identifier rather than inside a longer one.
///
/// `evaluate_constitution` contains `constitution`, and a recursive call would otherwise read as a use of the
/// parameter.
fn whole_identifier_occurrences<'a>(
    text: &'a str,
    identifier: &'a str,
) -> impl Iterator<Item = usize> + 'a {
    text.match_indices(identifier)
        .filter(|(at, _)| {
            text[..*at]
                .chars()
                .next_back()
                .is_none_or(|before| !before.is_alphanumeric() && before != '_')
        })
        .map(|(at, _)| at)
}

/// Whether `needle` occurs in `text` with nothing pathlike attached to its left.
///
/// `contains` alone accepted `crate::shim::hunyi::check_all(…)`: any path *ending* in the required call
/// satisfied a bare substring test, so a wrapper module could stand between the shell and 渾儀 while the
/// reaction reported a direct delegation — which is what the required call exists to rule out.
fn contains_token_bounded(text: &str, needle: &str) -> bool {
    text.match_indices(needle).any(|(at, _)| {
        text[..at]
            .chars()
            .next_back()
            .is_none_or(|before| !before.is_alphanumeric() && before != '_' && before != ':')
    })
}

/// The shell reads semantic boundaries only to pass them directly into 渾儀's public composed entry point.
///
/// A behavioral comparison cannot observe a local empty-bundle guard here: the static dimension has already
/// read the same manifest, and `hunyi::check_all` would return the same `Clean`. The source shape is therefore
/// the observation level that moved. Exactly one accessor occurrence rules out a second shell-local decision,
/// while the required direct call rules out hiding that decision behind an alias or wrapper.
///
/// This is also the spec's *tracked composition body is still judged* scenario: it asserts `Delegates`, which a
/// refusal is not, so a refusal broad enough to swallow every input fails here rather than passing as the
/// closure it was added for.
#[test]
fn the_shell_delegates_semantic_emptiness_to_the_public_entry_point() {
    let Some(root) = workspace_root() else {
        return;
    };
    let relative = "crates/tianheng/src/runner.rs";
    let path = root.join(relative);
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {path:?}: {error}"));
    let source = Source::of(text);
    match judge_delegation(&source) {
        // Absent and ambiguous both decline, and the message says which: they are different repairs, and a
        // reader told only "cannot judge" would go looking for a function that is in fact sitting right there
        // beside a copy of itself.
        None => panic!(
            "the shell delegation reaction cannot judge {relative}: {}",
            decline_reason(&source, "fn evaluate_constitution(")
        ),
        Some(Delegation::Delegates) => {}
        Some(Delegation::Diverges(why)) => panic!(
            "the shell must access semantic boundaries exactly once, as the direct `hunyi::check_all` \
             argument; another access is an independent shell decision — {why}: {relative}"
        ),
        Some(Delegation::CannotJudge(delimiter)) => panic!(
            "the extent read for `evaluate_constitution` carries `{delimiter}` on an executed line, so it may \
             not be the function's body and no verdict is given. Separating a brace in code from one inside a \
             literal or block comment needs the lexing this repository measured and rejected; move the \
             construct out of the composition function, or widen the reaction deliberately: {relative}"
        ),
    }
}

/// A composition body in the shape the reaction reads, with `interposed` between the delegation and a second
/// semantic-boundary access.
///
/// The second access is the shape the requirement refuses, so every fixture built here *should* be reported as
/// divergent. What each interposed construct changes is whether the reaction can see it at all.
fn composition_body(interposed: &[&str]) -> Source {
    let mut lines = vec![
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        "    manifest_path: &Path,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let mut outcome = check_and_cover(constitution.static_boundaries(), manifest_path);",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "    );",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        RuntimeObserver::new(constitution.runtime_boundaries().to_vec()).observe(manifest_path),",
        "    );",
    ];
    lines.extend_from_slice(interposed);
    lines.extend_from_slice(&[
        "    if constitution.semantic_boundaries().is_empty() {",
        "        outcome = early();",
        "    }",
        "    (outcome, None)",
        "}",
        "",
    ]);
    Source::of(lines.join("\n"))
}

/// The control: with nothing hiding the extent, the second access is seen and reported.
///
/// Without it, a reaction that refused or failed on everything would satisfy
/// [`an_ambiguous_delegation_extent_is_refused_rather_than_judged`] while observing nothing at all.
#[test]
fn a_divergent_composition_body_is_reported_when_nothing_moves_its_extent() {
    assert!(
        matches!(
            judge_delegation(&composition_body(&[])),
            Some(Delegation::Diverges(_))
        ),
        "a second semantic-boundary access with no interposed construct is plainly visible, so the reaction \
         must report it"
    );
}

/// An extent that may not be the body earns a refusal, not a verdict — a declared bound.
///
/// Each construct below moves the extent so that the second semantic-boundary access beneath it falls outside
/// what is compared. Measured before the refusal existed: all four read as a conforming delegation, because a
/// count of one and a containment are both satisfied by the truncated remainder. That is the false negative the
/// Core Contract forbids, reached by four spellings of one mechanism.
///
/// The control is [`a_divergent_composition_body_is_reported_when_nothing_moves_its_extent`], which shares the
/// fixture and differs only by the interposed construct.
#[test]
fn an_ambiguous_delegation_extent_is_refused_rather_than_judged() {
    let routes: [(&str, Vec<&str>); 4] = [
        (
            "a `//` inside a string literal, blanking a real opening brace",
            vec![
                "    if \"https://host\".is_empty() {",
                "        log();",
                "    }",
            ],
        ),
        (
            "a closing brace inside a string literal",
            vec!["    let fence = \"}\";"],
        ),
        (
            "a closing brace inside a block comment",
            vec!["    /* } */"],
        ),
        (
            "a closing brace inside a character literal",
            vec!["    let closer = '}';"],
        ),
    ];
    // Every route is judged before anything is asserted, so a regression reports each spelling it re-opened
    // rather than only the first. The four are one mechanism, and seeing one name is not seeing the mechanism.
    // The verdict is carried into the message, not just the label: "not refused" spans both a loud `Diverges`
    // and the silent `Delegates` that is the actual defect, and a regression report that cannot tell them
    // apart cannot say whether the false negative came back or merely a worse diagnostic.
    let unrefused: Vec<String> = routes
        .iter()
        .filter_map(
            |(label, interposed)| match judge_delegation(&composition_body(interposed)) {
                Some(Delegation::CannotJudge(_)) => None,
                other => Some(format!("{label} -> {other:?}")),
            },
        )
        .collect();
    assert!(
        unrefused.is_empty(),
        "the extent is moved, so the reaction must refuse rather than read the truncated remainder as the \
         delegation it demands — not refused: {unrefused:#?}"
    );
}

/// A delimiter in prose is not a delimiter in code, and the scan agrees with the brace count about which is
/// which.
///
/// Both directions, because only the pair distinguishes "reads executed code" from "reads whatever it is
/// given": an apostrophe in a comment — tail or whole line — must not refuse, while the same character in code
/// must. The tail case is the one that bit: it refused a body whose code holds no literal, and the tracked
/// `runner.rs` escaped only because each of its apostrophes happens to sit on a full-line comment.
#[test]
fn a_delimiter_only_inside_a_comment_does_not_refuse_the_extent() {
    let tail = composition_body(&["    let n = count();  // the shell's one access"]);
    assert!(
        matches!(judge_delegation(&tail), Some(Delegation::Diverges(_))),
        "an apostrophe in a comment tail cannot move a brace — the brace count already ignores that text, so \
         the ambiguity scan must ignore it too rather than refuse a readable body"
    );

    let whole_line = composition_body(&["    // the shell's one access"]);
    assert!(
        matches!(judge_delegation(&whole_line), Some(Delegation::Diverges(_))),
        "and the same in a whole-line comment, which `Executed` already filters"
    );

    let in_code = composition_body(&["    let closer = '}';"]);
    assert!(
        matches!(judge_delegation(&in_code), Some(Delegation::CannotJudge(_))),
        "the control: the identical character in code still refuses, so the two cases above are the comment's \
         doing and not the scan having stopped looking"
    );
}

/// The refusal precedes the comparison, and this is the input where the order is observable.
///
/// The body is truncated *and* divergent within what survives, so an implementation that formed a verdict first
/// and only escalated a pass would answer `Diverges` — a verdict read off text it cannot vouch for. Two such
/// orderings were built and both satisfied every other test in this file, which is why this input exists.
#[test]
fn an_ambiguous_extent_is_refused_even_when_the_remainder_looks_divergent() {
    let body = composition_body(&[
        "    let echo = constitution.semantic_boundaries();",
        "    let fence = \"}\";",
    ]);
    assert!(
        matches!(judge_delegation(&body), Some(Delegation::CannotJudge(_))),
        "the remainder holds two accesses and would answer `Diverges` on its own, but the extent is not known \
         to be the body — a verdict formed on text the reaction cannot vouch for is the thing the ordering \
         exists to prevent, whichever way that verdict happens to fall"
    );
}

/// The residual this reaction does not close: a truncation that leaves no delimiter behind.
///
/// A block comment opened *above* the signature closes after the brace it hides, so the extent is cut before
/// the `*/` and carries none of [`EXTENT_AMBIGUITY`]. The scan cannot see it, and that is why `*/` is not in
/// the set — it would never fire here. What the shape is held to is the direction: the surviving text loses the
/// delegation along with everything else, so the count is wrong and the reaction fails loudly. It is recorded
/// as a test rather than as prose because "the residual is loud" is a claim that can stop being true.
#[test]
fn a_truncation_carrying_no_delimiter_still_fails_loudly() {
    let source = Source::of(
        [
            "/* note: the guard } was removed",
            "fn evaluate_constitution(",
            "    constitution: &Constitution,",
            ") -> (Outcome, Option<Coverage>) {",
            "    still commented } here */",
            "    let mut outcome = merge_outcomes(",
            "        outcome,",
            "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
            "    );",
            "    (outcome, None)",
            "}",
            "",
        ]
        .join("\n"),
    );
    assert!(
        matches!(judge_delegation(&source), Some(Delegation::Diverges(_))),
        "the extent stops inside the block comment, so no delimiter reaches the scan — the reaction must still \
         refuse to call this a delegation, which it does by losing the delegation from the extent as well"
    );
}

/// A whole composition source, for shapes the interposing builder cannot express.
fn composition_source(lines: &[&str]) -> Source {
    Source::of(lines.join("\n") + "\n")
}

/// The bounds-method reader declines an ambiguous anchor too, and its safe direction depends on that.
///
/// Both readers share [`function_body`], so both inherited the decoy hole — and for this one the consequence
/// was sharper: its bound records the moved extent as *over-reacting*, safe because an exact one-statement
/// equality refuses a conforming body. A decoy copy inverts that. The extent becomes the decoy's conforming
/// body while the real method holds a second, divergent list, and the equality then passes on text that is not
/// the method. Measured end-to-end on 渾儀's observer before the anchor was required to be unique.
///
/// Pinned here rather than left to the delegation reader's fixture, because the claim being defended is this
/// reader's own error direction.
#[test]
fn a_decoy_bounds_signature_refuses_rather_than_matching_the_conforming_copy() {
    let decoyed = Source::of(
        [
            "/*",
            "    fn bounds(&self) -> Vec<BoundDecl> {",
            "        observation_bounds()",
            "    }",
            "*/",
            "    fn bounds(&self) -> Vec<BoundDecl> {",
            "        let mut declared = observation_bounds();",
            "        declared.truncate(1);",
            "        declared",
            "    }",
            "",
        ]
        .join("\n"),
    );
    assert!(
        bounds_body(&decoyed).is_none(),
        "two lines could anchor the read, so the reader declines — matching the commented conforming copy \
         would let the divergent list beneath it satisfy the obligation"
    );

    let single = Source::of(
        [
            "    fn bounds(&self) -> Vec<BoundDecl> {",
            "        let mut declared = observation_bounds();",
            "        declared.truncate(1);",
            "        declared",
            "    }",
            "",
        ]
        .join("\n"),
    );
    assert_ne!(
        bounds_body(&single).as_deref(),
        Some([DELEGATION.to_string()].as_slice()),
        "the same divergent list with one anchor is read and refused, so the decline above is the decoy's \
         doing rather than the reader declining everything"
    );
}

/// A second line that could anchor the read makes the subject unknown, so the reaction declines.
///
/// Line position rules out a *mid-line* mention and nothing more. A whole-line copy of the signature — in a
/// block comment, a multi-line string, or a second module — anchors exactly as well as the real definition,
/// and the first in the file wins. A doc comment counts as an occurrence too — under an occurrence rule there
/// is no carve-out for which mentions could have anchored, which is the point: this file stated one and stated
/// it backwards. Every delimiter that made that extent wrong then sits
/// outside it, so the in-body scan is blind by construction: measured end-to-end, a decoy above the real
/// function let a body carrying an independent shell-local guard read as a conforming delegation.
///
/// The control is the identical divergent body with the decoy removed, so the refusal is the decoy's doing and
/// not the recognizer having stopped reading.
#[test]
fn a_second_line_that_could_anchor_the_read_refuses_the_subject() {
    let decoyed = composition_source(&[
        "/*",
        "Kept for reference while the composition seam settles:",
        "fn evaluate_constitution(",
        ") -> (Outcome, Option<Coverage>) {",
        "    hunyi::check_all(constitution.semantic_boundaries(), manifest_path)",
        "}",
        "*/",
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let mut outcome = merge_outcomes(",
        "        outcome,",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "    );",
        "    if !constitution.semantic_boundaries().is_empty() {",
        "        outcome = early();",
        "    }",
        "    (outcome, None)",
        "}",
    ]);
    assert!(
        judge_delegation(&decoyed).is_none(),
        "two lines could anchor the read, so the reaction cannot know which body is the subject and must \
         decline — picking the first reads a body that is not the function's and gives a verdict on it"
    );
    assert_eq!(
        anchor_count(&decoyed, "fn evaluate_constitution("),
        2,
        "and the decline is reported as ambiguous rather than absent: the two are different repairs, and a \
         reader sent looking for a missing function would not find the copy sitting beside it"
    );

    // The other two shapes the requirement enumerates. Listed and unmeasured is how an enumeration comes to
    // include a member nothing reaches — which the delimiter set had already done once in this change.
    for (label, decoy) in [
        (
            "a copy inside a multi-line string",
            vec![
                "const EXAMPLE: &str = \"",
                "fn evaluate_constitution(",
                ") -> (Outcome, Option<Coverage>) {",
                "}",
                "\";",
            ],
        ),
        (
            "a second module's definition",
            vec![
                "mod legacy {",
                "    fn evaluate_constitution(",
                "    ) -> (Outcome, Option<Coverage>) {",
                "        unimplemented!()",
                "    }",
                "}",
            ],
        ),
    ] {
        let mut lines = decoy.clone();
        lines.extend_from_slice(&[
            "fn evaluate_constitution(",
            "    constitution: &Constitution,",
            ") -> (Outcome, Option<Coverage>) {",
            "    let mut outcome = merge_outcomes(",
            "        outcome,",
            "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
            "    );",
            "    if !constitution.semantic_boundaries().is_empty() {",
            "        outcome = early();",
            "    }",
            "    (outcome, None)",
            "}",
        ]);
        assert!(
            judge_delegation(&composition_source(&lines)).is_none(),
            "{label} anchors as well as the definition, so the read is ambiguous and must decline"
        );
    }

    // A doc comment counts too. Under an occurrence rule there is no carve-out to get right: whether a
    // second mention could have been brace-matched from stops mattering, because the read declines either
    // way. The earlier trimmed-start rule needed that distinction and this file got it backwards once.
    let doc_comment = composition_source(&[
        "/// fn evaluate_constitution(",
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    hunyi::check_all(constitution.semantic_boundaries(), manifest_path)",
        "}",
    ]);
    assert!(
        judge_delegation(&doc_comment).is_none(),
        "a doc comment naming the signature is a second occurrence, so the read declines — the rule is \
         where the name appears, not whether that place could have anchored"
    );

    // The decoy that motivated the change of rule: the real definition carries a visibility modifier, so it
    // no longer begins its line, and a trimmed-start rule would have left the commented copy as the single
    // candidate — unique, and not the subject.
    let visibility_modified = composition_source(&[
        "/*",
        "fn evaluate_constitution(",
        ") -> (Outcome, Option<Coverage>) {",
        "    hunyi::check_all(constitution.semantic_boundaries(), manifest_path)",
        "}",
        "*/",
        "pub(crate) fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    if constitution.semantic_boundaries().is_empty() {",
        "        return (Outcome::Clean, None);",
        "    }",
        "    hunyi::check_all(constitution.semantic_boundaries(), manifest_path)",
        "}",
    ]);
    assert!(
        judge_delegation(&visibility_modified).is_none(),
        "two occurrences, so the read declines — a trimmed-start rule saw only the commented one and \
         judged it"
    );

    let single = composition_source(&[
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let mut outcome = merge_outcomes(",
        "        outcome,",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "    );",
        "    if !constitution.semantic_boundaries().is_empty() {",
        "        outcome = early();",
        "    }",
        "    (outcome, None)",
        "}",
    ]);
    assert!(
        matches!(judge_delegation(&single), Some(Delegation::Diverges(_))),
        "the same divergent body with one anchor is read and reported, so the refusal above is the decoy and \
         not a recognizer that declines everything"
    );
}

/// Every way of reaching the constitution that is not one of the three owners is refused, by complement.
///
/// These are not new spellings patched one at a time — they are the spellings that defeated the counting
/// rule, kept as the record of why counting was abandoned. Under an allowlist none of them needed its own
/// case: each is simply a use of `constitution` that is not on the list. The last is the one that ends the
/// argument for counting accessor names at all — it names no accessor, reading the private `semantic` field
/// directly, which a descendant module of the crate root may do.
#[test]
fn every_reach_for_the_constitution_outside_the_permitted_owners_is_refused() {
    let routes: [(&str, &[&str]); 7] = [
        (
            "a rebinding",
            &[
                "    let shell = constitution;",
                "    if shell.semantic_boundaries().is_empty() {",
                "        outcome = early();",
                "    }",
            ],
        ),
        (
            "an associated-function call",
            &["    let _ = Constitution::semantic_boundaries(constitution);"],
        ),
        (
            "a reborrow",
            &["    let _ = (&*constitution).semantic_boundaries();"],
        ),
        (
            "a wrapper function",
            &["    if semantic_view(constitution).is_empty() {", "    }"],
        ),
        (
            "a macro",
            &["    if semantic!(constitution).is_empty() {", "    }"],
        ),
        (
            "a trait method under another name",
            &["    if constitution.sem_view().is_empty() {", "    }"],
        ),
        (
            "a direct read of the private field, naming no accessor at all",
            &[
                "    if constitution.semantic.signature.len() > 7 {",
                "    }",
            ],
        ),
    ];
    let unreported: Vec<String> = routes
        .iter()
        .filter_map(|(label, extra)| {
            let mut lines = vec![
                "fn evaluate_constitution(",
                "    constitution: &Constitution,",
                ") -> (Outcome, Option<Coverage>) {",
                "    let mut outcome = check_and_cover(constitution.static_boundaries(), manifest_path);",
                "    outcome = merge_outcomes(",
                "        outcome,",
                "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
                "    );",
                "    outcome = merge_outcomes(",
                "        outcome,",
                "        RuntimeObserver::new(constitution.runtime_boundaries().to_vec()).observe(manifest_path),",
                "    );",
            ];
            lines.extend_from_slice(extra);
            lines.extend_from_slice(&["    (outcome, None)", "}"]);
            match judge_delegation(&composition_source(&lines)) {
                Some(Delegation::Diverges(_)) => None,
                other => Some(format!("{label} -> {other:?}")),
            }
        })
        .collect();
    assert!(
        unreported.is_empty(),
        "each of these reaches the constitution outside the three permitted owners and must be reported — \
         not reported: {unreported:#?}"
    );
}

/// The delegation must be 渾儀's own entry point, not any path that happens to end in it.
#[test]
fn a_callee_path_ending_in_the_entry_point_is_not_the_entry_point() {
    for shim in ["crate::shim::hunyi::check_all", "not_hunyi::check_all"] {
        let body = composition_source(&[
            "fn evaluate_constitution(",
            "    constitution: &Constitution,",
            ") -> (Outcome, Option<Coverage>) {",
            "    let mut outcome = check_and_cover(constitution.static_boundaries(), manifest_path);",
            "    outcome = merge_outcomes(",
            "        outcome,",
            &format!("        {shim}(constitution.semantic_boundaries(), manifest_path),"),
            "    );",
            "    outcome = merge_outcomes(",
            "        outcome,",
            "        RuntimeObserver::new(constitution.runtime_boundaries().to_vec()).observe(manifest_path),",
            "    );",
            "    (outcome, None)",
            "}",
        ]);
        assert!(
            matches!(judge_delegation(&body), Some(Delegation::Diverges(_))),
            "`{shim}` is a wrapper standing between the shell and 渾儀, which is exactly what requiring the \
             direct call rules out — a bare substring test accepted it"
        );
    }
}

/// A body `rustfmt` reflowed is the same body, and must not be reported as a violation.
///
/// The allowlist matches an owner as written, so a chain the formatter breaks across lines stops matching and
/// reads as an independent shell decision. That is a false accusation produced by the formatter this repository
/// *also* gates on — `cargo fmt --check` would demand the very reflow that turns the gate red, with a
/// diagnostic naming a fault nobody committed.
///
/// The control is the same shape with an unpermitted use interposed, so the fold is shown to close up a chain
/// without closing up the gap between two identifiers — which is what would hide a real reach.
#[test]
fn a_body_rustfmt_reflowed_is_still_the_same_body() {
    let reflowed = composition_source(&[
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let mut outcome = check_and_cover(constitution.static_boundaries(), manifest_path);",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "    );",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        RuntimeObserver::new(",
        "            constitution",
        "                .runtime_boundaries()",
        "                .to_vec(),",
        "        )",
        "        .observe(manifest_path),",
        "    );",
        "    (outcome, None)",
        "}",
    ]);
    assert_eq!(
        judge_delegation(&reflowed),
        Some(Delegation::Delegates),
        "`rustfmt` broke the runtime chain across lines and changed nothing about what the body does; the \
         reaction must read it as the same delegation"
    );

    let reflowed_but_divergent = composition_source(&[
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let mut outcome = check_and_cover(constitution.static_boundaries(), manifest_path);",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "    );",
        "    outcome = merge_outcomes(",
        "        outcome,",
        "        RuntimeObserver::new(",
        "            constitution",
        "                .runtime_boundaries()",
        "                .to_vec(),",
        "        )",
        "        .observe(manifest_path),",
        "    );",
        "    if constitution.semantic.signature.is_empty() {",
        "        outcome = early();",
        "    }",
        "    (outcome, None)",
        "}",
    ]);
    assert!(
        matches!(
            judge_delegation(&reflowed_but_divergent),
            Some(Delegation::Diverges(_))
        ),
        "folding a broken chain must not fold the space between two identifiers: the private-field reach is \
         still reported in a reflowed body"
    );
}

/// What the reaction reads is characters in one function body — a declared bound, and the reason it is one.
///
/// The allowlist closed every *spelling* of a reach for the constitution. It cannot close the family beneath
/// that, because those escapes are not spellings inside the body at all: **name resolution** (a `shim::hunyi`
/// shadowing the entry point, with the body byte-identical), the **binding site** (the parameter renamed, or a
/// second parameter carrying the boundaries — the parameter list sits outside the extent), **which definition
/// is the subject** (a raw identifier, so the only occurrence of the signature is a decoy), and the **caller
/// frame** (the guard moved up into `check_constitution`). Each was measured end-to-end against the tracked
/// composition function, with the full suite green and the formatter and linter silent.
///
/// This pin exhibits the binding-site member, which is the one a fixture can carry whole: the parameter is
/// renamed, so no occurrence of `constitution` in executed code belongs to it, while a `stringify!` bait —
/// text the reaction cannot tell from code — supplies the three owners and the literal delegation.
///
/// The control is the same body without the bait, which is reported, so the acceptance is the bait's doing and
/// not the reaction having stopped reading.
#[test]
fn a_reach_that_is_not_a_spelling_in_the_body_is_not_observed() {
    let baited = composition_source(&[
        "fn evaluate_constitution(",
        "    the_constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    const SHAPE: &str = stringify!(",
        "        check_and_cover(constitution.static_boundaries(), manifest_path),",
        "        hunyi::check_all(constitution.semantic_boundaries(), manifest_path),",
        "        RuntimeObserver::new(constitution.runtime_boundaries().to_vec()),",
        "    );",
        "    let _ = SHAPE;",
        "    if the_constitution.semantic_boundaries().is_empty() {",
        "        return (Outcome::Clean, None);",
        "    }",
        "    (outcome, None)",
        "}",
    ]);
    assert_eq!(
        judge_delegation(&baited),
        Some(Delegation::Delegates),
        "the body decides semantic emptiness for itself and the reaction reports a conforming delegation — a \
         stated bound. Closing it needs name resolution and the binding site, neither of which is a character \
         in this extent"
    );

    let unbaited = composition_source(&[
        "fn evaluate_constitution(",
        "    the_constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    if the_constitution.semantic_boundaries().is_empty() {",
        "        return (Outcome::Clean, None);",
        "    }",
        "    (outcome, None)",
        "}",
    ]);
    assert!(
        matches!(judge_delegation(&unbaited), Some(Delegation::Diverges(_))),
        "without the bait the three owners are absent and the body is reported, so the bound above is the \
         bait's doing rather than the reaction accepting anything"
    );
}

/// A delegation written in a comment is prose, and prose does not satisfy the requirement.
///
/// The comparison read comment tails as code, so the required call could be supplied entirely by a comment
/// while the real body delegated through a rebinding — measured accepted. Comment tails are now removed before
/// the comparison, which is what the brace count already did.
#[test]
fn a_delegation_written_only_in_a_comment_tail_does_not_satisfy_the_requirement() {
    let prose_delegation = composition_source(&[
        "fn evaluate_constitution(",
        "    constitution: &Constitution,",
        ") -> (Outcome, Option<Coverage>) {",
        "    let shell = constitution; // hunyi::check_all(constitution.semantic_boundaries(), manifest_path)",
        "    if shell.semantic_boundaries().is_empty() {",
        "        return (Outcome::Clean, None);",
        "    }",
        "    let outcome = merge_outcomes(outcome, hunyi::check_all(shell.semantic_boundaries(), manifest_path));",
        "    (outcome, None)",
        "}",
    ]);
    assert!(
        !matches!(
            judge_delegation(&prose_delegation),
            Some(Delegation::Delegates)
        ),
        "the only occurrence of the required call is inside a comment, so the body does not delegate — a \
         requirement satisfied by prose is satisfied in appearance and failed in substance"
    );
}

/// The delegation this reaction requires, in compacted form.
const SEMANTIC_DELEGATION: &str =
    "hunyi::check_all(constitution.semantic_boundaries(),manifest_path)";

/// A body's executed Rust with whitespace removed and a trailing comma before a closing parenthesis dropped.
///
/// Both normalizations answer to the same thing: `rustfmt` decides where a call breaks, and a broken argument
/// list gains a trailing comma. Joining the text without dropping that comma would make this reaction fail the
/// day the call grows past the line width — a false positive produced by `cargo fmt`, in a repository that gates
/// `cargo fmt --check`, over a delegation that did not change. `,)` cannot occur in compacted Rust for any other
/// reason, and nested occurrences collapse too, since the replacement scans the whole string.
///
/// Named and taking text so its limit is demonstrated by giving it both spellings — see
/// [`the_delegation_recognizer_reads_both_spellings_rustfmt_produces`] — rather than by widening this crate's
/// lines until `rustfmt` breaks them.
fn compact_executed_rust(source: &Source) -> String {
    let joined: String = source
        .rust()
        .lines()
        .map(without_comment_tail)
        .flat_map(str::chars)
        .filter(|character| !character.is_whitespace())
        .collect();
    joined.replace(",)", ")")
}

/// The recognizer reads the one-line and the wrapped spelling alike, and still refuses an alias.
///
/// The control is the third case: without it, a normalizer that had become permissive enough to accept anything
/// would satisfy the first two and look like a fix.
#[test]
fn the_delegation_recognizer_reads_both_spellings_rustfmt_produces() {
    let one_line =
        Source::of("hunyi::check_all(constitution.semantic_boundaries(), manifest_path),\n");
    let wrapped = Source::of(
        "hunyi::check_all(\n    constitution.semantic_boundaries(),\n    manifest_path,\n),\n",
    );
    let aliased = Source::of(
        "let boundaries = constitution.semantic_boundaries();\nhunyi::check_all(boundaries, manifest_path)\n",
    );

    assert!(
        compact_executed_rust(&one_line).contains(SEMANTIC_DELEGATION),
        "the spelling in the tree today"
    );
    assert!(
        compact_executed_rust(&wrapped).contains(SEMANTIC_DELEGATION),
        "and the spelling `rustfmt` produces once the call no longer fits — the same delegation, so the same \
         verdict"
    );
    assert!(
        !compact_executed_rust(&aliased).contains(SEMANTIC_DELEGATION),
        "an alias must still be refused: it is how a second behavior owner comes back"
    );
}

/// The one statement a conforming `bounds()` body holds.
const DELEGATION: &str = "observation_bounds()";

/// The executed statements inside `fn bounds`'s body, or `None` if no line anchors the method or more than
/// one does.
///
/// Brace-counted from the signature's opening brace, so a nested block inside the body would be included rather
/// than truncating at the first `}` — the body is required to be one statement, but a *wrong* body must be
/// reported whole rather than mis-parsed into looking right.
fn bounds_body(source: &Source) -> Option<Vec<String>> {
    let body = function_body(source, "fn bounds(")?;
    Some(
        body.rust()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| {
                // A trailing comment is PROSE, not a second list. `Executed` filters comment lines and not
                // comment tails, so without this `observation_bounds() // why` compares unequal and the reaction
                // reports an offence — measured. Both whole-line recognizers in `gate_shape_contract.rs` already
                // strip one; this is the same rule, not a new allowance.
                let code = match line.find("//") {
                    Some(index) => &line[..index],
                    None => line,
                };
                // Written as a tail expression today; a `return …;` says the same thing and must read the same.
                code.trim()
                    .trim_start_matches("return ")
                    .trim_end_matches(';')
                    .trim()
                    .to_string()
            })
            .collect(),
    )
}

/// The same text with `{` and `}` inside a line-comment tail replaced by a space.
///
/// Byte offsets are preserved exactly — only a one-byte ASCII brace is ever swapped for a one-byte ASCII space
/// — so the mask can be brace-matched while the ORIGINAL text is sliced with the offsets that produces. That
/// matters concretely: this tree's comments carry 漢字, and a mask that re-encoded anything would shift every
/// offset after the first multi-byte character.
///
/// Why it exists: [`function_body`] counted braces through comments, so `observation_bounds(); // }` closed the
/// body at the comment and everything after it — a second list — was never read. `bounds_body`'s own `//`-tail
/// stripping then made the truncated remainder look like the exact delegation, and the reaction passed. The
/// stripping had to move *before* the brace count, not after it.
///
/// [`Executed`] cannot do this job: it filters lines whose trimmed start is `//`, so a comment TAIL — which is
/// the shape above — survives it whole, brace and all.
///
/// What it does **not** do is understand literals: a `//` inside a string blanks a real opening brace whose
/// match is on a later line, and a brace inside a string, a character literal, or a block comment is counted as
/// code. The extent then moves, and what that costs depends entirely on the comparison reading it — refusal for
/// an exact equality, a silent pass for a count. Consumers that cannot detect their own truncation check
/// [`EXTENT_AMBIGUITY`] before comparing; this function does not do it for them, because the safe answer is not
/// the same for every reader.
fn mask_line_comment_braces(text: &str) -> String {
    let mut bytes = text.as_bytes().to_vec();
    let mut line_start = 0usize;
    while line_start <= bytes.len() {
        let line_end = bytes[line_start..]
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(bytes.len(), |at| line_start + at);
        if let Some(at) = bytes[line_start..line_end]
            .windows(2)
            .position(|pair| pair == b"//")
        {
            for byte in &mut bytes[line_start + at..line_end] {
                if *byte == b'{' || *byte == b'}' {
                    *byte = b' ';
                }
            }
        }
        if line_end >= bytes.len() {
            break;
        }
        line_start = line_end + 1;
    }
    String::from_utf8(bytes).expect("only ASCII braces were replaced, each by one ASCII space")
}

/// How many times `signature` occurs, so a decline can say whether it found none or too many.
///
/// The same rule [`function_body`] anchors by, deliberately: the two disagreed for one round — this counted
/// trimmed-start lines while the reader counted occurrences — so a declined read reported "1", sending a reader
/// to look for a second definition its own count denied.
fn anchor_count(source: &Source, signature: &str) -> usize {
    source.whole().matches(signature).count()
}

/// Why a read declined, in the reader's own words.
///
/// Three inputs decline and they call for different repairs: the signature is absent, it occurs more than once,
/// or it occurs exactly once and no balanced body follows it. Reporting all three as an anchor count sent a
/// reader hunting for a second definition that the same message had just counted as absent.
fn decline_reason(source: &Source, signature: &str) -> String {
    match anchor_count(source, signature) {
        0 => format!("`{signature}` does not occur, so there is no body to read"),
        1 => format!(
            "`{signature}` occurs once but no balanced brace-delimited body follows it, so the extent could \
             not be taken"
        ),
        many => format!(
            "`{signature}` occurs {many} times, so the subject is ambiguous and the reader judges only when it \
             occurs exactly once"
        ),
    }
}

fn function_body(source: &Source, signature: &str) -> Option<Source> {
    let text = source.whole();
    // Braces are counted over the MASK and the body is sliced out of the original, which the mask's
    // offset-for-offset construction makes the same positions. See [`mask_line_comment_braces`].
    let masked = mask_line_comment_braces(text);
    // The anchor is an OCCURRENCE of the signature, and there must be exactly one in the whole source.
    //
    // An earlier rule required the signature to begin a trimmed line. That only ruled out a mid-line mention;
    // it never made the definition findable, and a decoy exploits the gap from either side — a whole-line copy
    // inside a block comment anchors just as well as the definition, and writing the definition
    // `pub(crate) fn …` stops it anchoring while the commented copy still does, leaving one candidate that is
    // not the subject. Both measured end-to-end, on both readers of this recognizer.
    //
    // Counting occurrences catches both directions and subsumes the mid-line mention, so no carve-out for
    // which mentions could have anchored is needed — an earlier attempt stated one and stated it wrongly.
    // Every delimiter that made a wrong extent wrong sits OUTSIDE that extent, where no in-body scan reaches,
    // which is why this half cannot be recovered by checking the body once read.
    if text.matches(signature).count() != 1 {
        return None;
    }
    let signature = text.find(signature)?;
    let open = signature + masked[signature..].find('{')?;
    let mut depth = 0usize;
    let mut close = None;
    for (offset, character) in masked[open..].char_indices() {
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
    Some(Source::of(&text[open + 1..close?]))
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
/// It reads every Rust source recursively. Public re-exports can make an item in a private nested module
/// reachable, so module visibility is not a sound premise for excluding that file from the corpus.
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
    let (files, offenders) = trait_object_offenders(&src);
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

fn trait_object_offenders(root: &Path) -> (usize, Vec<String>) {
    let mut pending = vec![root.to_path_buf()];
    let mut files = 0usize;
    let mut offenders = Vec::new();
    while let Some(directory) = pending.pop() {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot read source directory {directory:?}: {error}"))
            .map(|entry| entry.expect("a readable source directory entry").path())
            .collect();
        entries.sort();
        for path in entries {
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files += 1;
                let text = std::fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("cannot read Rust source {path:?}: {error}"));
                for (number, line) in Source::of(text).rust().numbered_lines() {
                    if exposes_a_trait_object(line) {
                        offenders.push(format!(
                            "{}:{}: {}",
                            path.display(),
                            number,
                            line.trim_start()
                        ));
                    }
                }
            }
        }
    }
    offenders.sort();
    (files, offenders)
}

#[test]
fn a_trait_object_in_a_nested_source_file_is_observed() {
    let root = std::env::temp_dir().join(format!(
        "tianheng-observer-protocol-nested-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let nested = root.join("runner");
    std::fs::create_dir_all(&nested).expect("create nested source fixture");
    std::fs::write(
        root.join("lib.rs"),
        "mod runner;\npub use runner::leaked;\n",
    )
    .expect("write fixture root");
    std::fs::write(
        nested.join("mod.rs"),
        "pub fn leaked() -> Box<dyn std::fmt::Debug> { todo!() }\n",
    )
    .expect("write nested fixture");

    let (_, offenders) = trait_object_offenders(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        offenders.len(),
        1,
        "a private module can re-export a nested public item, so nesting must not remove it from the corpus"
    );
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
