use crate::bound_register_parse::{bare_references, marks_a_bound};

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

/// Every reference on a line is reported, not one of them.
#[test]
fn two_references_on_one_line_are_both_reported() {
    let found = bare_references(
        &capability_set(),
        "probe-capability/first-one and probe-capability/second-one\n",
    );
    assert_eq!(found.len(), 2, "{found:?}");
}
