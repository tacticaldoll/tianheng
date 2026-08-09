//! `observer-protocol`'s reaction: the trait-driven fold and the built-in path are one verdict, each observer
//! declares exactly its dimension's bounds, and the fold's ordering directions hold.
//!
//! Two composition paths exist deliberately for the **static** dimension alone — the built-in one calls
//! `check_and_cover`, whose coverage advisory the protocol cannot carry and whose second call would read
//! `cargo metadata` twice, while the observer calls `check`. Two implementations that could disagree silently
//! are the drift a seam is supposed to end, so the cost of comparing them is paid here.
//!
//! For the **semantic** and **runtime** dimensions there is no second path left to compare: the built-in one
//! invokes `SemanticObserver` and `RuntimeObserver`, so equality for those holds by construction. What this
//! file still observes for them is that the fixture's boundary in each reacts at all — an arm that went vacuous
//! would otherwise leave the whole comparison resting on one dimension.
//!
//! Construction-held **equality** is not a construction-held delegation, and the two are one word apart. That
//! the shell honours its obligation to leave semantic decisions to 渾儀 is still unobserved and still a
//! declared bound: a guard deciding emptiness above the invocation compiles and passes everything here.
//!
//! Some of the properties below hold **by construction**, and each says which reaction stands in for the
//! comparison that would be inert. That is deliberate, and the alternative was worse: an assertion that cannot
//! fail reads exactly like a guarantee.

use std::path::{Path, PathBuf};

// Everything reaches this test through the shell; a direct edge to 璇璣 would breach the shell's
// self-governed dependency direction.
use tianheng::check_constitution;
use tianheng::prelude::*;

use jiaochou::region::Source;

fn workspace_manifest() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
    .map(|root| root.join("Cargo.toml"))
}

/// The workspace root, or `None` outside a checkout — the same skip-here / loud-in-CI discipline as above.
fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates").is_dir(),
        shengmo::workspace::marker_set(),
    )
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
/// how the same moved extent went four windows accepting a divergent body elsewhere. That second reader compared
/// by count and containment, which a truncated remainder satisfies in full, so what fell past the cut was not
/// there to refuse; it was retired rather than narrowed again, and the distinction is kept here so this bound's
/// safety is not read as transferring to the next reader written over the same recognizer.
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

/// The reader's behaviour, run rather than described — every row of [`ANCHOR_CASES`].
///
/// Three properties are asserted over the table itself, because a table that had gone degenerate would satisfy
/// every row while proving nothing: some shape is read, some is declined, and at least one is read WRONGLY.
/// That last is what makes the declared bound visible here instead of only in prose.
#[test]
fn the_reader_decides_every_shape_as_the_table_says() {
    for case in ANCHOR_CASES {
        let source = Source::of(case.source);
        let read = function_body(&source, "fn bounds(").map(|body| body.whole().to_string());
        let expected = match case.verdict {
            Verdict::Reads(body) | Verdict::ReadsTheWrongBody(body) => Some(body.to_string()),
            Verdict::Declines => None,
        };
        assert_eq!(
            read, expected,
            "{}: the reader's decision and this table must be the same statement",
            case.shape
        );
    }

    let read = ANCHOR_CASES
        .iter()
        .filter(|case| matches!(case.verdict, Verdict::Reads(_)))
        .count();
    let declined = ANCHOR_CASES
        .iter()
        .filter(|case| case.verdict == Verdict::Declines)
        .count();
    let wrong = ANCHOR_CASES
        .iter()
        .filter(|case| matches!(case.verdict, Verdict::ReadsTheWrongBody(_)))
        .count();
    assert!(
        read > 0 && declined > 0,
        "a table with no reading row or no declining row is satisfied by a reader that does one thing"
    );
    assert!(
        wrong > 0,
        "the declared bound over this reader is shown by a row, so removing the last such row would leave the \
         bound asserted only in prose"
    );
}

/// A **mid-line** mention anchors nothing, even when it is the signature's only occurrence.
///
/// Occurrence counting alone admits this: one occurrence, in a comment, with the definition absent because the
/// impl moved to another file. The anchor lands in the prose, the next `{` belongs to whatever follows, and its
/// body is returned as this method's. The line-start rule the count replaced declined it — so the two rules
/// each admit what the other refuses, and [`function_body`] requires both.
///
/// It is **mid-line** and not *mention* that this closes, which an earlier version of this test claimed and got
/// wrong: a whole-line copy anchors, and that is a declared bound.
///
/// The control is an ordinary definition, which must still read — requiring both conditions must decline more,
/// not decline everything. It cannot be "the same mention beside a definition", because that is two occurrences
/// and the uniqueness rule refuses it.
#[test]
fn a_mid_line_mention_anchors_nothing() {
    let mention_only = Source::of(
        "// the array points here, but the impl for `fn bounds(` moved away\nfn other() -> u8 { 0 }\n",
    );
    assert_eq!(
        anchor(mention_only.whole(), "fn bounds("),
        Anchor::MentionOnly,
        "the fixture is the hard case precisely because the count alone admits it"
    );
    assert!(
        function_body(&mention_only, "fn bounds(").is_none(),
        "a lone mid-line mention with no definition must decline, not read the next function's body"
    );

    let ordinary_definition =
        Source::of("fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n");
    assert!(
        function_body(&ordinary_definition, "fn bounds(").is_some(),
        "and the ordinary definition still reads, so requiring both conditions declines more rather than \
         declining everything"
    );
    assert_eq!(
        decline_reason(&mention_only, "fn bounds("),
        "`fn bounds(` occurs once but not at the start of a line, so it is a mention rather than a definition \
         and anchors nothing",
        "and the decline names the condition that failed — reporting a missing brace here sent a repairer \
         looking for one that is present"
    );
}

/// The bounds-method reader declines an ambiguous anchor too, and its safe direction depends on that.
///
/// The reader that survives and the one this window retired both used [`function_body`], so both inherited
/// the decoy hole — and for this one the consequence
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
    let single_body = bounds_body(&single);
    assert!(
        single_body.is_some(),
        "the same divergent list with one anchor is READ — asserting only that it differs from the delegation \
         is satisfied by a reader that declines everything, which is what this control exists to rule out"
    );
    assert_ne!(
        single_body.as_deref(),
        Some([DELEGATION.to_string()].as_slice()),
        "and refused, so the decline above is the decoy's doing rather than the reader declining everything"
    );

    // The decline says WHICH condition it met. Absent and ambiguous call for different repairs, and the two
    // counters disagreed for one round — the reader counting occurrences while the reporter counted
    // trimmed-start lines — so a declined read reported "1" and sent a reader after a definition its own count
    // denied. The retirement deleted the only assertion over that, and this is its replacement.
    // A MID-LINE mention, which is what separates counting occurrences from counting lines that begin with the
    // signature. The decoy above begins its line, so both counters agree on it and it discriminates nothing;
    // this one is invisible to a trimmed-start count and is exactly the regression the two counters' one-round
    // disagreement produced.
    let mentioned = Source::of(
        [
            "// the obligation is about `fn bounds(` and nothing else",
            "    fn bounds(&self) -> Vec<BoundDecl> {",
            "        observation_bounds()",
            "    }",
            "",
        ]
        .join("\n"),
    );
    assert_eq!(
        anchor(mentioned.whole(), "fn bounds("),
        Anchor::Ambiguous(2),
        "a mention inside a comment is a second occurrence; counting only lines that BEGIN with the signature \
         sees one and judges a subject it cannot know"
    );
    assert!(
        bounds_body(&mentioned).is_none(),
        "so the read declines, rather than brace-matching from whichever of the two came first"
    );
    assert_eq!(
        anchor(decoyed.whole(), "fn bounds("),
        Anchor::Ambiguous(2),
        "the block-comment decoy is counted too, though it begins its line and either counter would see it"
    );
    assert!(
        decline_reason(&decoyed, "fn bounds(").contains("ambiguous"),
        "an ambiguous anchor is reported as ambiguous: {}",
        decline_reason(&decoyed, "fn bounds(")
    );
    assert!(
        decline_reason(&single, "fn missing_signature(").contains("does not occur"),
        "and an absent one as absent, which is the distinction the message exists to draw"
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
/// the exact one-statement equality below, a silent pass for a count-and-containment. A reader of that second
/// kind existed and was retired; this function does not guard against the move on a caller's behalf, because
/// the safe answer is not the same for every reader.
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

/// What this reader does with every shape it can meet — **including the shapes it gets wrong**.
///
/// The table is the description. A comment saying which shapes the anchor rule refuses drifted from the code
/// twice in one window, and each repair corrected the sentence review had named and then wrote the next one; a
/// row cannot drift, because it runs. `observer-protocol`'s declared bound over this reader is read off the
/// [`Verdict::ReadsTheWrongBody`] rows rather than typed beside them, and a reviewer's perturbation lands here
/// as a row instead of as a finding.
struct AnchorCase {
    /// The shape, in the words a spec scenario would use for it.
    shape: &'static str,
    source: &'static str,
    verdict: Verdict,
}

/// What the reader makes of a shape.
#[derive(Debug, PartialEq, Eq)]
enum Verdict {
    /// It reads the method's own body.
    Reads(&'static str),
    /// It declines, which is this reader's declared error direction.
    Declines,
    /// It reads a body that is **not** the method's — a declared false negative, shown rather than described.
    ReadsTheWrongBody(&'static str),
}

const ANCHOR_CASES: &[AnchorCase] = &[
    AnchorCase {
        shape: "an ordinary definition",
        source: "fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n",
        verdict: Verdict::Reads("\n    observation_bounds()\n"),
    },
    AnchorCase {
        shape: "a definition whose delegation carries a comment tail",
        source: "fn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds() // why\n}\n",
        verdict: Verdict::Reads("\n    observation_bounds() // why\n"),
    },
    AnchorCase {
        shape: "no occurrence at all",
        source: "fn other() -> u8 { 0 }\n",
        verdict: Verdict::Declines,
    },
    AnchorCase {
        shape: "a mid-line mention, the definition absent",
        source: "// the impl for `fn bounds(` moved away\nfn other() -> u8 { 0 }\n",
        verdict: Verdict::Declines,
    },
    AnchorCase {
        shape: "a whole-line copy beside the definition — the decoy",
        source: "/*\nfn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n*/\nfn bounds(&self) -> Vec<BoundDecl> {\n    divergent()\n}\n",
        verdict: Verdict::Declines,
    },
    AnchorCase {
        shape: "a whole-line copy in a block comment, the definition moved out of the file",
        source: "/*\nfn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n*/\nfn other() -> u8 { 0 }\n",
        verdict: Verdict::ReadsTheWrongBody("\n    observation_bounds()\n"),
    },
    AnchorCase {
        shape: "a whole-line copy in a string literal, the definition moved out of the file",
        source: "const MOVED: &str = \"\nfn bounds(&self) -> Vec<BoundDecl> {\n    observation_bounds()\n}\n\";\nfn other() -> u8 { 0 }\n",
        verdict: Verdict::ReadsTheWrongBody("\n    observation_bounds()\n"),
    },
];

/// What the anchor rule decides about `signature` in a source — **one rule, one return**.
///
/// The reader and its diagnostic both match on this. Two callers each re-deriving the rule agreed by
/// maintenance and drifted twice: once when a count of trimmed-start lines faced a count of occurrences, once
/// when a line-start condition was added to the reader and not to the reason. Matching on one function's
/// return, they agree by construction, and a fifth case forces every consumer to answer it or the build fails.
///
/// A doc comment enumerating these is a census of a set this type already holds, so there is none.
#[derive(Debug, PartialEq, Eq)]
enum Anchor {
    /// The signature does not occur, so there is no body to read.
    Absent,
    /// It occurs more than once: the subject is a set rather than a site.
    Ambiguous(usize),
    /// It occurs once, but mid-line — a mention rather than a definition.
    MentionOnly,
    /// It occurs once and begins a line: the offset a body is read from.
    At(usize),
}

/// Whether the byte offset `at` is preceded on its own line by nothing but whitespace.
fn begins_a_line(text: &str, at: usize) -> bool {
    text[..at]
        .rsplit_once('\n')
        .map_or(&text[..at], |(_, last)| last)
        .trim()
        .is_empty()
}

/// The anchor rule. What it does **not** decide is shown, not described, in [`ANCHOR_CASES`].
fn anchor(text: &str, signature: &str) -> Anchor {
    match text.matches(signature).count() {
        0 => Anchor::Absent,
        1 => {
            let at = text
                .find(signature)
                .expect("one occurrence was just counted, so it is findable");
            if begins_a_line(text, at) {
                Anchor::At(at)
            } else {
                Anchor::MentionOnly
            }
        }
        many => Anchor::Ambiguous(many),
    }
}

/// Why a read declined, in the reader's own words — each decline naming the condition that failed, because
/// reporting them all as an anchor count sent a reader hunting for a second definition the same message had
/// just counted as absent.
fn decline_reason(source: &Source, signature: &str) -> String {
    match anchor(source.whole(), signature) {
        Anchor::Absent => format!("`{signature}` does not occur, so there is no body to read"),
        Anchor::MentionOnly => format!(
            "`{signature}` occurs once but not at the start of a line, so it is a mention rather than a \
             definition and anchors nothing"
        ),
        Anchor::At(_) => format!(
            "`{signature}` occurs once, at a line start, but no balanced brace-delimited body follows it, so \
             the extent could not be taken"
        ),
        Anchor::Ambiguous(many) => format!(
            "`{signature}` occurs {many} times, so the subject is ambiguous and the reader judges only when it \
             occurs exactly once"
        ),
    }
}

/// The brace-delimited body the unique occurrence of `signature` anchors, or `None` if none anchors it or more
/// than one does.
///
/// Declining rather than taking the first is the point: an occurrence in a comment anchors exactly as well as a
/// definition, so uniqueness is what makes the subject knowable, and the anchor scan therefore reads the whole
/// source rather than [`Executed`].
fn function_body(source: &Source, signature: &str) -> Option<Source> {
    let text = source.whole();
    // Braces are counted over the MASK and the body is sliced out of the original, which the mask's
    // offset-for-offset construction makes the same positions. See [`mask_line_comment_braces`].
    let masked = mask_line_comment_braces(text);
    // What this rule decides, and what it decides WRONGLY, is shown in [`ANCHOR_CASES`] rather than described
    // here — a comment saying which shapes it refuses drifted from it twice in one window.
    let Anchor::At(signature) = anchor(text, signature) else {
        return None;
    };
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
