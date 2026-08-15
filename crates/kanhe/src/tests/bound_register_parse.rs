use crate::bound_register_parse::{
    bare_references, marks_a_bound, negates_bound_in_prose, projection_offences,
    states_a_bound_in_prose, undeclared_prose_offences,
};
use std::collections::BTreeSet;

/// What the model gate's comparison catches, as a case rather than as a claim.
///
/// The spec used to require the slug rule to be implemented twice, reasoning that one shared rule would
/// collapse the comparison to `f() == f()`. That reasoning was measured and did not hold, because the
/// comparison is a derived set against a **tracked file**. This holds the property that argument was trying to
/// protect, and it holds it however many implementations the slug rule has — which is why it replaces the
/// requirement rather than restating it.
#[test]
fn a_projection_disagreeing_with_the_derived_set_names_the_id_on_either_side() {
    let derived: BTreeSet<String> = ["a/one", "b/two"].iter().map(|id| id.to_string()).collect();

    let complete = "# heading\n\n### `a/one`\n\ntext\n\n### `b/two`\n";
    assert!(
        projection_offences(&derived, complete).is_empty(),
        "a projection carrying exactly the derived ids must raise nothing"
    );

    let stale = "# heading\n\n### `a/one`\n";
    let missing = projection_offences(&derived, stale);
    assert_eq!(missing.len(), 1, "expected one offence, got {missing:?}");
    assert!(
        missing[0].contains("b/two") && missing[0].contains("absent from the projection"),
        "a stale projection must name the id it lacks, got {missing:?}"
    );

    let invented = "### `a/one`\n### `b/two`\n### `c/three`\n";
    let extra = projection_offences(&derived, invented);
    assert_eq!(extra.len(), 1, "expected one offence, got {extra:?}");
    assert!(
        extra[0].contains("c/three") && extra[0].contains("derived from no spec"),
        "an id no spec derives must be named, got {extra:?}"
    );
}

#[test]
fn bound_markers_are_bare_singular_word_sequences() {
    let cases = [
        ("Which member owns it — a stated bound", true),
        ("Which member owns it — a documented bound", true),
        ("Which member owns it — stated bound", false),
        ("Which member owns it — a stated bounds", false),
        ("Which member owns it — documented bounds", false),
        ("Which member owns it — a cfg-blind stated bound", false),
        ("Metadata stated boundary", false),
        ("Data stated bound", false),
        ("資料a stated bound", false),
    ];

    for (heading, expected) in cases {
        assert_eq!(
            marks_a_bound(heading),
            expected,
            "unexpected marker decision for {heading:?}"
        );
    }
}

/// The shell era's `BOUND_PROSE` trigger, ported: `stated`/`documented`, at most one interposed word, then
/// `bound`/`bounds` — deliberately looser than `marks_a_bound` (see `states_a_bound_in_prose`'s own doc for
/// why the two are not the same rule).
#[test]
fn prose_bound_trigger_admits_one_interposed_word() {
    let cases = [
        ("this is a stated bound", true),
        ("this is a documented bound", true),
        ("this is a stated coverage bound", true),
        ("this is a documented residual bounds", true),
        ("this is a stated two word bound", false),
        ("nothing here mentions a boundary at all", false),
        ("understated bounds should not match", false),
    ];
    for (line, expected) in cases {
        assert_eq!(
            states_a_bound_in_prose(line),
            expected,
            "unexpected prose-trigger decision for {line:?}"
        );
    }
}

/// The three sentences the shell era measured a *wider* negation rule to hide, each carrying a negation that
/// applies to a different verb than the bound noun — copied verbatim from
/// the deleted shell-era bound-register gate's own comment recording the measurement. All three must still trigger
/// as declarations: negation adjacency is the fix, not negation absence.
#[test]
fn negation_adjacent_to_the_noun_is_excluded_a_negation_elsewhere_in_the_sentence_is_not() {
    let real_declarations_despite_a_nearby_negation = [
        "type aliases are not expanded (a stated bound)",
        "the invocation is not transparent, so its body stays a stated coverage bound",
        "a production probe must not live behind a non-production cfg — a stated bound",
    ];
    for line in real_declarations_despite_a_nearby_negation {
        assert!(
            states_a_bound_in_prose(line),
            "must still trigger the prose scan: {line:?}"
        );
        assert!(
            !negates_bound_in_prose(line),
            "the negation here applies to a different verb, not the bound noun, so it must not be treated \
             as a denial: {line:?}"
        );
    }

    let genuine_denials = [
        "a cfg-blind union rather than a skip bound",
        "this is not a stated bound",
        "this is never a documented bound",
        // The interposed-word tolerance belongs to "a/an ... bound", not to "stated/documented ... bound" —
        // one word between "a" and "bound" is tolerated here, same as the base case above.
        "this is not a plain bound",
    ];
    for line in genuine_denials {
        assert!(
            negates_bound_in_prose(line),
            "the negation sits directly on the bound noun and must be recognized: {line:?}"
        );
    }
}

/// The capability set these rows are shown, including one this repository deliberately does **not** declare.
///
/// **`probe-capability` is load-bearing, not filler.** A row needing a synthetic *slug* must hang it off a
/// synthetic *capability*, because the repository direction resolves every bare id in tracked Rust — this file
/// included — against the real declarations. Hanging an invented slug off a real capability name made the
/// reaction report this file's own fixtures as dangling references, and then made it report *this paragraph*
/// when the paragraph named the offending token. That is the class this repository has hit before: a text
/// reaction reading its own text, which is why the rule is to recognize by position or shape and why this
/// explanation describes the shape instead of spelling it. Do not "correct" the fixture to a real capability
/// name, and do not quote one here.
fn capability_set() -> std::collections::BTreeSet<String> {
    [
        "repository-checks",
        "external-crate-confinement",
        "probe-capability",
    ]
    .into_iter()
    .map(str::to_string)
    .collect()
}

/// A bare id naming a capability this repository declares is a reference.
#[test]
fn a_bare_id_is_recognised_as_a_reference() {
    let found = bare_references(
        &capability_set(),
        "/// See repository-checks/files-no-capability-claims-a-stated-bound for why.\n",
    );
    assert_eq!(
        found,
        vec![(
            1,
            "repository-checks/files-no-capability-claims-a-stated-bound".to_string()
        )]
    );
}

/// **A path containing a capability name is not a reference.**
///
/// The whole reason recognition reads maximal runs: a substring search finds
/// `repository-checks/spec.md` inside this path and would refuse it for resembling a reference. The run
/// carries three slashes, so it is not a `<capability>/<slug>` pair at all.
#[test]
fn a_path_containing_a_capability_name_is_not_a_reference() {
    assert!(
        bare_references(
            &capability_set(),
            "// declared in openspec/specs/repository-checks/spec.md\n"
        )
        .is_empty()
    );
}

/// A slug that is not kebab-case is not a derived bound id.
#[test]
fn a_non_kebab_right_hand_side_is_not_a_reference() {
    for text in [
        "repository-checks/Not_Kebab\n",
        "repository-checks/trailing-\n",
        "repository-checks/UPPER\n",
    ] {
        assert!(
            bare_references(&capability_set(), text).is_empty(),
            "{text:?} must not read as a reference"
        );
    }
}

/// A name the enumerated capability set does not carry is not a reference.
///
/// The set is enumerated from the tracked specs by the caller, so a capability added later is recognized
/// without this recognizer being touched — and a word that merely looks like one is not invented into a
/// reference here.
#[test]
fn an_unknown_left_hand_side_is_not_a_reference() {
    assert!(bare_references(&capability_set(), "some-other-thing/a-slug-shaped-tail\n").is_empty());
}

// --- undeclared_prose_offences: the four scenarios `observation-bound-register/spec.md`'s "A bound stated
// in prose but not declared as a scenario SHALL fail" requirement names -------------------------------------

/// Scenario: Spec prose states a bound that no scenario declares.
#[test]
fn prose_stating_a_bound_with_no_scenario_and_no_reference_fails_naming_the_occurrence() {
    let text = "### Requirement: Some requirement\n\nSome prose states a stated bound here.\n";
    let offences = undeclared_prose_offences(
        "openspec/specs/some-capability/spec.md",
        text,
        &capability_set(),
    );
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert!(
        offences[0].contains("spec.md:3") && offences[0].contains("stated bound"),
        "must name the file and the occurrence: {offences:?}"
    );
}

/// Scenario: The same statement inside a declared bound scenario does not fail.
#[test]
fn the_same_statement_inside_a_declared_bound_scenario_does_not_fail() {
    let text = "### Requirement: Some requirement\n\n#### Scenario: Something is not observed — a stated bound\n\n- **WHEN** a shape appears\n- **THEN** a stated bound holds here, restated in the body\n";
    let offences = undeclared_prose_offences(
        "openspec/specs/some-capability/spec.md",
        text,
        &capability_set(),
    );
    assert!(
        offences.is_empty(),
        "prose inside a declared bound scenario must not fail: {offences:?}"
    );
}

/// Scenario: Prose under a bounds-named requirement is exempt, and the requirement pays for it.
#[test]
fn prose_under_a_bounds_named_requirement_is_exempt_but_the_requirement_then_owes_a_scenario() {
    let declares = "### Requirement: Observation bounds are stated, not silent\n\nItem 1 is a stated bound.\n\n#### Scenario: Item one is unreachable — a stated bound\n\n- **WHEN** x\n- **THEN** y\n";
    assert!(
        undeclared_prose_offences(
            "openspec/specs/some-capability/spec.md",
            declares,
            &capability_set()
        )
        .is_empty(),
        "the requirement declares a bound scenario, so its own prose list is exempt with nothing further owed"
    );

    let does_not_declare =
        "### Requirement: Observation bounds are stated, not silent\n\nItem 1 is a stated bound.\n";
    let offences = undeclared_prose_offences(
        "openspec/specs/some-capability/spec.md",
        does_not_declare,
        &capability_set(),
    );
    assert_eq!(offences.len(), 1, "{offences:?}");
    assert!(
        offences[0].contains("names bounds") && offences[0].contains("declares no bound scenario"),
        "a bounds-named requirement that states one in prose and declares no scenario must be charged for \
         it: {offences:?}"
    );
}

/// A resolvable bare reference clears bound-declaring prose even with no wrapping scenario — the companion
/// requirement ("Prose MAY reference a declared bound") this scan must not re-flag.
#[test]
fn a_resolvable_bare_reference_clears_bound_declaring_prose_with_no_scenario() {
    let text = "### Requirement: Some requirement\n\nSee probe-capability/some-slug, a stated bound already declared elsewhere.\n";
    let offences = undeclared_prose_offences(
        "openspec/specs/some-capability/spec.md",
        text,
        &capability_set(),
    );
    assert!(
        offences.is_empty(),
        "a line carrying a resolvable bare reference must be cleared: {offences:?}"
    );
}

/// A negation directly on the bound noun is not a declaration, matching the shell era's own measured
/// tolerance (see `negation_adjacent_to_the_noun_is_excluded_a_negation_elsewhere_in_the_sentence_is_not`).
#[test]
fn a_negated_bound_in_prose_is_not_an_offence() {
    let text = "### Requirement: Some requirement\n\nThis is a cfg-blind union rather than a skip bound.\n";
    let offences = undeclared_prose_offences(
        "openspec/specs/some-capability/spec.md",
        text,
        &capability_set(),
    );
    assert!(
        offences.is_empty(),
        "a negation directly on the bound noun denies the bound rather than declaring one: {offences:?}"
    );
}

/// Every reference on a line is reported, not one of them.
#[test]
fn two_references_on_one_line_are_both_reported() {
    let found = bare_references(
        &capability_set(),
        "probe-capability/first-one and probe-capability/second-one\n",
    );
    assert_eq!(found.len(), 2, "{found:?}");
}
