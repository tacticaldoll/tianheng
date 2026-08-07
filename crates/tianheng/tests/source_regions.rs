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

/// Markdown has two fence characters, and prose is what neither of them holds.
///
/// The requirement this serves — a generated document's path must appear where a reader is *sent*, and a path
/// only inside a fence is not that — is written about fenced code, not about one spelling of a fence. Reading
/// only the backtick form makes a `~~~` block count as prose, so a path nothing points at in prose would
/// satisfy the reachability rule. Latent rather than live: no tracked Markdown uses `~~~` today, which is
/// exactly the state in which a hole is cheapest to close and least likely to be noticed.
#[test]
fn prose_excludes_a_tilde_fenced_block() {
    let fenced = Source::of(format!("visible\n~~~bash\n{UNSEEN}\n~~~\n"));
    assert!(fenced.prose().contains("visible"));
    assert!(!fenced.prose().contains(UNSEEN));
}

/// A fence closes on its own character, so one form shown inside the other stays fenced.
///
/// The obvious repair — toggle on either marker — reopens the hole from the other side: a `~~~` line displayed
/// inside a backtick block would close it, and the rest of that block would become prose. Measured against the
/// naive form before this landed. CommonMark closes a fence only with a run of the character that opened it,
/// at least as long, which is why the state carries the character rather than a boolean.
///
/// This matters here specifically: `AGENTS.md` is documentation about documentation, so a fence displayed
/// inside a fence is ordinary content rather than a contrivance.
#[test]
fn a_fence_of_the_other_form_does_not_close_the_open_one() {
    let nested = Source::of(format!(
        "visible\n```markdown\nan example of the other fence:\n~~~\n{UNSEEN}\n~~~\n```\ntail\n"
    ));
    assert!(nested.prose().contains("visible"));
    assert!(nested.prose().contains("tail"), "and the block does close");
    assert!(
        !nested.prose().contains(UNSEEN),
        "the inner `~~~` is content of the backtick block, not a closing delimiter"
    );
}

/// A longer run closes a shorter opener; a shorter run inside a longer fence does not.
#[test]
fn a_fence_closes_only_on_a_run_at_least_as_long() {
    let long_opener = Source::of(format!("visible\n````\n```\n{UNSEEN}\n````\ntail\n"));
    assert!(
        !long_opener.prose().contains(UNSEEN),
        "a three-backtick line does not close a four-backtick fence"
    );
    assert!(long_opener.prose().contains("tail"), "the longer run does");
}
