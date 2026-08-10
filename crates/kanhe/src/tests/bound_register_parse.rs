use crate::bound_register_parse::marks_a_bound;

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
    ];

    for (heading, expected) in cases {
        assert_eq!(
            marks_a_bound(heading),
            expected,
            "unexpected marker decision for {heading:?}"
        );
    }
}
