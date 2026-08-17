use kanhe::region::Source;

#[test]
fn executed_regions_respect_the_source_language() {
    let rust = Source::of("#[cfg(test)]\n// hidden\nfn guarded() {}\n");
    assert!(rust.rust().contains("#[cfg(test)]"));
    assert!(!rust.rust().contains("hidden"));

    let shell = Source::of("# hidden\nprintf '%s\\n' '// data'\n");
    assert!(!shell.shell().contains("hidden"));
    assert!(shell.shell().contains("// data"));
}

/// The positioned region keeps every line's own place, and blanks the ones it dropped.
///
/// The property `lines()` cannot have and every position-sensitive caller needs. Both halves are asserted,
/// because either alone is satisfiable by a degenerate implementation — one that blanks everything, or one
/// that blanks nothing and compacts.
///
/// The rule this exists for is the shell's continuation rule, measured rather than reasoned about: given
/// `echo START \`, `# comment`, `--exact ghost`, bash prints `START` and then reports
/// `--exact: command not found`. Compacted, the backslash reaches across the comment and binds the third
/// line into the first command; positioned, the blank ends the continuation exactly where bash does.
#[test]
fn the_positioned_region_holds_each_line_at_its_own_index() {
    let source = Source::of("echo START \\\n# comment\n--exact ghost\necho tail # cut\n");
    let positioned = source.shell().positioned_lines();

    assert_eq!(
        positioned.len(),
        4,
        "every source line keeps a slot, or an index into this is an index into something else: {positioned:?}"
    );
    assert_eq!(positioned[0], "echo START \\");
    assert_eq!(
        positioned[1], "",
        "a whole-line comment is blanked in place, not removed — removing it makes its neighbours adjacent \
         and a trailing backslash then continues across the line bash uses to end the command"
    );
    assert_eq!(positioned[2], "--exact ghost");
    assert_eq!(
        positioned[3].trim_end(),
        "echo tail",
        "a tail comment is still cut; positioning changes where a line sits, not what counts as executed"
    );

    // The control: the compacted reading is genuinely shorter, so the assertion above is about a difference
    // that exists rather than about two spellings of one thing.
    assert_eq!(source.shell().lines().count(), 3);
}

/// A comment is not executed text **wherever it sits**, and a marker that begins no token is not a comment.
///
/// Both directions, because either alone is satisfiable by a degenerate reader — one that cuts everything, or
/// one that cuts nothing. The second direction is the load-bearing one: cutting at the first marker was measured
/// against this repository and corrupts constants holding a URL, a string carrying a doc marker, and the region
/// helper's own `comment` field.
#[test]
fn a_tail_comment_is_not_executed_text_and_a_glued_marker_is_not_a_comment() {
    let tail = Source::of("let n = 1; // hidden\nlet m = 2;\n");
    assert!(
        tail.rust().contains("let n = 1;"),
        "the executed head of the line survives"
    );
    assert!(
        !tail.rust().contains("hidden"),
        "a tail comment is a comment, so its text is not executed"
    );

    let glued = Source::of("let u = \"https://example.invalid/x\";\n");
    assert!(
        glued.rust().contains("https://example.invalid/x"),
        "a marker glued to what precedes it begins no token and is not a comment"
    );

    let shell_tail = Source::of("printf '%s' one # hidden\n");
    assert!(
        shell_tail.shell().contains("printf '%s' one"),
        "the same rule in the shell region"
    );
    assert!(
        !shell_tail.shell().contains("hidden"),
        "a shell tail comment is not executed text either"
    );
}

/// Prose excludes the comment SPAN, not the line carrying it.
///
/// `projection-register`'s requirement is that a path appearing **only** inside an HTML comment is not a
/// mention. A whole-line drop answers a different question: it also hides a path a reader plainly sees, so the
/// register refused a document that satisfies the rule. The two directions are asserted together because either
/// alone is satisfiable by a degenerate reader — one that hides everything, or one that hides nothing.
/// Real projection paths rather than invented ones: the consumer of this region searches for exactly these, and
/// an invented `docs/…` path is a stale in-repository reference that
/// `crates/kanhe/tests/reference_integrity.rs` refuses.
const SEEN: &str = "docs/projection-register.md";
const UNSEEN: &str = "the retired gate-shape projection";

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
/// satisfy the reachability rule. Latent rather than live: no tracked Markdown has a line that *opens* a
/// `~~~` fence, which is exactly the state in which a hole is cheapest to close and least likely to be noticed.
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

/// Two delimiters are not a fence, so a line opening with them is prose.
///
/// Three is CommonMark's minimum and also the pre-change threshold, so nothing observed it until now — a
/// two-character run opening a fence would silently hide the rest of a document from the reader it serves.
#[test]
fn a_run_shorter_than_three_does_not_open_a_fence() {
    let inline = Source::of(format!("``{SEEN}`` and ~~{UNSEEN}~~\n"));
    let prose = inline.prose();
    assert!(
        prose.contains(SEEN) && prose.contains(UNSEEN),
        "a two-character run is inline markup, not a fence, so the line stays prose"
    );
}

/// Whitespace after a closing run is still a closing run.
///
/// CommonMark allows spaces and tabs there, and losing that means a closer with a trailing space stops closing
/// and silently swallows the remainder of the document — the direction this whole reader exists to avoid. A CRLF
/// closer is *not* the reason: `str::lines` already drops the trailing `\r`, measured, and an earlier version of
/// this comment claimed otherwise.
#[test]
fn a_closing_run_may_be_followed_by_whitespace() {
    let padded = Source::of(format!("visible\n```\n{UNSEEN}\n```  \t\ntail\n"));
    assert!(!padded.prose().contains(UNSEEN));
    assert!(
        padded.prose().contains("tail"),
        "trailing whitespace does not stop a run from closing"
    );
}

/// A closing fence carries no info string, so a run followed by text is content.
///
/// This is the third leg of the same problem, and the only one that errs in **both** directions from one
/// construct: an inner ```` ```rust ```` closed the block, so its contents counted as prose, and the bare run
/// beneath it then re-opened a fence that never closed, so everything after was excluded forever. Found by
/// review of the two-character fix, which had closed the cross-character and run-length legs and left this one.
#[test]
fn a_run_followed_by_an_info_string_does_not_close_a_fence() {
    let inner_info_string = Source::of(format!(
        "visible\n```\nan example markdown file:\n```rust\n{UNSEEN}\n```\nafter\n"
    ));
    let prose = inner_info_string.prose();
    assert!(prose.contains("visible"));
    assert!(
        !prose.contains(UNSEEN),
        "`\u{60}\u{60}\u{60}rust` inside an open fence is content, not a closer, so what follows it is still fenced"
    );
    assert!(
        prose.contains("after"),
        "and the bare run does close, so the fence does not swallow the rest of the document"
    );
}

/// A longer run closes a shorter opener; a shorter run inside a longer fence does not.
///
/// Both halves, because the pre-existing case closed a four-backtick opener with a four-backtick run — equal,
/// not longer — so `==` satisfied it and `>=` was carried by nothing until the first case below was added.
#[test]
fn a_fence_closes_only_on_a_run_at_least_as_long() {
    let longer_closes_shorter = Source::of(format!("visible\n```\n{UNSEEN}\n````\ntail\n"));
    assert!(
        !longer_closes_shorter.prose().contains(UNSEEN),
        "the fenced content is still fenced"
    );
    assert!(
        longer_closes_shorter.prose().contains("tail"),
        "a four-backtick run closes a three-backtick fence, which `>=` allows and `==` would not"
    );

    let long_opener = Source::of(format!("visible\n````\n```\n{UNSEEN}\n````\ntail\n"));
    assert!(
        !long_opener.prose().contains(UNSEEN),
        "a three-backtick line does not close a four-backtick fence"
    );
    assert!(long_opener.prose().contains("tail"), "the longer run does");
}

/// TOML's comment rule is TOML's, not the shell's spelled the same way.
///
/// `toml()` and `shell()` share the marker `#` and were once one rule. They are not one rule: TOML admits
/// zero whitespace before `#`, the shell does not, and the difference is not cosmetic in either direction.
///
/// Both were live. `{ path = "crates/xuanji" }#, version = "0.2.0"` declares **no** version, and the release
/// gate read the commented one and certified the pin — a false pass in front of `cargo publish`. Meanwhile
/// `version.workspace = true#c` is a legal comment on a line that still inherits, so cutting nothing there
/// was a false refusal. One rule that is actually TOML's answers both; no adjustment of a borrowed rule does.
///
/// The negative run for each direction is a separate edit, which is why they are separate assertions with
/// separate reasons rather than one fixture.
#[test]
fn a_toml_comment_needs_no_whitespace_before_it_and_a_hash_in_a_string_is_not_one() {
    let glued = Source::of("xuanji = { path = \"crates/xuanji\" }#, version = \"0.2.0\"\n");
    assert!(
        glued.toml().contains("path = \"crates/xuanji\""),
        "the executed head survives the cut"
    );
    assert!(
        !glued.toml().contains("version"),
        "a glued `#` opens a TOML comment, so the version it carries was never declared — reading it is how \
         a manifest with no pin passed the gate that exists to check the pin"
    );

    // The value the token-start rule was reaching for, now held by knowing it is a string rather than by
    // hoping no space precedes the fragment. A space does precede it here, which the old rule cut.
    let fragment = Source::of("documentation = \"https://docs.rs/kanhe/ #anchor\"\n");
    assert!(
        fragment.toml().contains("#anchor\""),
        "a `#` inside a string is string content at any distance from the quote"
    );

    let escaped = Source::of("description = \"a quote \\\" then #not-a-comment\"\n");
    assert!(
        escaped.toml().contains("#not-a-comment"),
        "an escaped quote does not close the string, so what follows is still inside it"
    );

    let literal = Source::of("path = 'C:\\x #still-a-path'\n");
    assert!(
        literal.toml().contains("#still-a-path"),
        "a literal string takes no escapes and still holds its `#`"
    );

    // The shell keeps the token-start rule as its own approximation: `echo a#b` prints `a#b`.
    let shell = Source::of("printf '%s' a#b\n");
    assert!(
        shell.shell().contains("a#b"),
        "widening TOML's rule must not widen the shell's — they share a marker, not a decision"
    );
}

/// The shell region's whole decision table, so the description of the rule *is* the run.
///
/// **Written because three doc paragraphs asserted the rule instead of a case asserting it**, and two of the
/// three overclaimed — one calling the token-start rule *the shell's own*, one scoping the string-literal
/// residue to Rust alone. A claim about what this region decides now lands as a row here, and
/// `Rule::TokenStart` reads its table off these shapes rather than restating bash.
///
/// Every expectation below was measured with `bash -c` before it was written, not reasoned about.
#[test]
fn the_shell_region_decides_these_shapes_and_no_others() {
    for (source, kept, why) in [
        ("printf a#b\n", true, "glued to a word, bash prints it"),
        ("printf a #b\n", false, "after whitespace, bash comments it"),
        (
            "#!/usr/bin/env bash\n",
            false,
            "a whole-line comment is dropped",
        ),
        (
            "curl \"$url#frag\"\n",
            true,
            "glued inside a value, bash keeps the fragment",
        ),
    ] {
        assert_eq!(
            Source::of(source).shell().contains("#"),
            kept,
            "{why}: {source:?}"
        );
    }
}

/// A `#` opened by an unquoted metacharacter stays in the region — a declared over-inclusion.
///
/// Measured: `bash -c 'printf a;#b'` prints `a`, and `bash -c '(printf a)#b'` prints `a`, so bash opens a
/// comment at both. The token-start rule tests only for whitespace or line start, so both survive into the
/// executed region and a property over executed text can be satisfied by commentary. That is the first defect
/// this module's own header says it exists to end, reappearing in the one language whose rule the header
/// claimed to implement exactly.
///
/// Latent: no tracked script carries the shape on an executed line. Closing it needs word-splitting.
#[test]
fn a_shell_marker_after_a_metacharacter_stays_in_the_region() {
    for source in ["printf a;#b\n", "(printf a)#b\n", "printf a|#b\n"] {
        assert!(
            Source::of(source).shell().contains("#b"),
            "bash opens a comment here and this region does not: {source:?}"
        );
    }
}

/// A whitespace-preceded `#` inside quotes is cut — a declared under-inclusion, the forbidden direction.
///
/// Measured: `bash -c 'printf "a #b"'` prints `a #b`, so the marker is string content and not a comment. The
/// token-start rule cuts it, deleting executed text — which `cut_tail_comment`'s own doc names as the
/// direction the Core Contract forbids, and which a sentence in this module once scoped to `rust()` alone
/// while `shell()` ran the identical rule.
///
/// Latent: no tracked script carries the shape on an executed line. Closing it needs the quote tracking
/// `Rule::Toml` has, rewritten for the shell's own quoting.
#[test]
fn a_shell_marker_inside_quotes_is_cut_from_the_region() {
    for source in ["printf \"a #b\"\n", "printf 'a #b'\n"] {
        assert!(
            !Source::of(source).shell().contains("#b"),
            "bash keeps this as string content and this region cuts it: {source:?}"
        );
    }
}

/// A multi-line TOML string carries its `#` across the line boundary, whole-line ones included.
///
/// The reason the scan is not per-line. `"""` and `'''` span lines, so a `#`-led line inside one is string
/// content; dropping it as a whole-line comment deletes executed text, which is the direction the Core
/// Contract forbids — and the drop is the one branch a tail-cut alone would not have covered.
///
/// The single-line forms deliberately do **not** carry: TOML forbids a raw newline inside them, so an
/// unterminated `"` is malformed and its damage stays on its own line instead of swallowing the file.
#[test]
fn a_multi_line_toml_string_carries_across_lines_and_a_broken_one_does_not() {
    let spanning = Source::of(
        "description = \"\"\"\n# not a comment\nstill inside\n\"\"\"\nversion = \"1\"\n",
    );
    assert!(
        spanning.toml().contains("# not a comment"),
        "a `#`-led line inside `\"\"\"` is string content, not a whole-line comment"
    );
    assert!(
        spanning.toml().contains("still inside"),
        "and the string continues to its own delimiter"
    );
    assert!(
        spanning.toml().contains("version = \"1\""),
        "the delimiter closes it, so what follows is code again"
    );

    let unterminated = Source::of("name = \"broken\nversion = \"1\" # cut\n");
    assert!(
        !unterminated.toml().contains("cut"),
        "an unterminated single-line string ends at its line, so the next line is read as the code it is"
    );
}
