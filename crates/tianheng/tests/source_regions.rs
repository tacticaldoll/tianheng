mod support;

use support::region::Source;

#[test]
fn executed_regions_respect_the_source_language() {
    let rust = Source::of("#[cfg(test)]\n// hidden\nfn guarded() {}\n");
    assert!(rust.rust().contains("#[cfg(test)]"));
    assert!(!rust.rust().contains("hidden"));

    let shell = Source::of("# hidden\nprintf '%s\\n' '// data'\n");
    assert!(!shell.shell().contains("hidden"));
    assert!(shell.shell().contains("// data"));
}

/// Prose excludes the comment SPAN, not the line carrying it.
///
/// `projection-register`'s requirement is that a path appearing **only** inside an HTML comment is not a
/// mention. A whole-line drop answers a different question: it also hides a path a reader plainly sees, so the
/// register refused a document that satisfies the rule. The two directions are asserted together because either
/// alone is satisfiable by a degenerate reader — one that hides everything, or one that hides nothing.
/// Real projection paths rather than invented ones: the consumer of this region searches for exactly these, and
/// an invented `docs/…` path is a stale in-repository reference that `check_reference_integrity.sh` refuses.
const SEEN: &str = "docs/projection-register.md";
const UNSEEN: &str = "docs/gate-shape-contract.md";

#[test]
fn prose_keeps_visible_text_beside_an_html_comment() {
    let mixed = Source::of(format!("See {SEEN} <!-- {UNSEEN} -->\n"));
    assert!(
        mixed.prose().contains(SEEN),
        "a path outside the comment span is what the reader sees, so it is a mention"
    );
    assert!(
        !mixed.prose().contains(UNSEEN),
        "a path inside the span is invisible to that reader, so it is not — the exclusion this replaces"
    );
}

/// The span survives the line it opened on, and text after it closes is visible again.
#[test]
fn prose_excludes_a_multi_line_comment_span_and_resumes_after_it() {
    let middle = "docs/observation-bounds.md";
    let closing = "docs/observation-bound-extents.md";
    let spanning = Source::of(format!(
        "intro <!-- {UNSEEN}\n{middle}\n{closing} --> {SEEN}\nplain\n"
    ));
    let prose = spanning.prose();
    assert!(prose.contains("intro"), "text before the opener is visible");
    for hidden in [UNSEEN, middle, closing] {
        assert!(
            !prose.contains(hidden),
            "{hidden} sits inside the span, on the opening, middle, or closing line"
        );
    }
    assert!(
        prose.contains(SEEN),
        "the span closes mid-line, so what follows on that line is visible again"
    );
    assert!(prose.contains("plain"), "and the span does not leak onward");
}

/// A fence still hides what it holds, so the excision above did not widen prose.
#[test]
fn prose_still_excludes_a_fenced_block() {
    let fenced = Source::of(format!("visible\n```bash\n{UNSEEN}\n```\n"));
    assert!(fenced.prose().contains("visible"));
    assert!(!fenced.prose().contains(UNSEEN));
}
