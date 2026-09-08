//! Repository check: no comment paragraph in this repository's Rust is written twice in a row.
//!
//! A paste that lands twice compiles, formats, lints and reads as deliberate. Nothing in the toolchain has an
//! opinion about it, and the second copy is invisible to the person who made it for the same reason it was
//! made: the eye that skips a paragraph it has already read is the eye that pasted it.
//!
//! It matters here more than the missing newline this crate's sibling refuses, because of what this
//! repository's prose is **for**. The rules are carried by weight rather than by enforcement — what sits in
//! an agent's context is what gets imitated — and a paragraph appearing twice is that weight doubled by
//! accident. The defect this was written for stood in `release_coherence_gate`, where the paragraph naming
//! why presence is asked by `ls-tree` rather than `show` was six lines long and stood twice, byte-identical.
//!
//! **What is compared is line content, not line bytes**, so a block terminated `\r\n` and a copy terminated
//! `\n` are one repetition here. A paste an editor re-terminated is still a paste, and requiring the
//! terminators to agree would let it through — a silent false negative, which is the one direction the Core
//! Contract forbids. The requirement says so in those words rather than saying *byte-identical*, which
//! claimed a precision this reader deliberately does not have.
//!
//! **A comment-shaped line inside a string literal is text, not a comment.** This repository holds Rust
//! fixtures as Rust strings, so the two cannot be told apart by reading a trimmed line — and the class is
//! not hypothetical: the first spelling of this check's own fixture was such a string, and the sweep
//! reported this file. Comments are what a lexer discards, so the classification is taken from
//! `proc_macro2`: a line strictly inside a literal's span is that literal's text. A doc comment reaches the
//! lexer as `#[doc = "…"]`, so it is told apart by the one thing that distinguishes it — a literal whose own
//! first line is comment-shaped is a doc comment — and shadowing those would have stopped this check reading
//! `///` at all, which is a false negative over most of this repository's prose.
//!
//! **The reach is adjacency, and the corpus is Rust.** Both are stops this reader makes deliberately, so
//! both are declared rather than left to be inferred: `docs/observation-bounds.md` carries them as
//! `repository-checks/a-paragraph-repeated-out-of-line-is-not-read-a-stated-bound` and
//! `repository-checks/a-paragraph-repeated-in-prose-is-not-read-a-stated-bound`.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kanhe::refusal::{Kind, Refusal, cannot_judge, violation};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// What a comment line says, or `None` where the line is not one.
///
/// `//`, `///` and `//!` are one class here: the question is whether a paragraph was written twice, and the
/// marker that introduces it decides who reads it, not whether it was pasted. A line whose content is empty
/// answers `Some("")` and is refused a place in a block below — see [`repetitions`].
fn comment_body(line: &str) -> Option<&str> {
    let trimmed = line.trim();
    let rest = trimmed.strip_prefix("//")?;
    Some(rest.trim_start_matches(['/', '!']).trim())
}

/// The lines of `text` that sit **inside** a literal, so a comment-shaped line among them is not a comment.
///
/// **A line beginning `//` is a comment, unless it is text.** `comment_body` reads the trimmed line, which
/// cannot tell a comment from a line of a multi-line string literal that happens to start with the same two
/// characters — and this repository is full of Rust fixtures that are Rust source held as strings. The class
/// is not hypothetical: the first spelling of this check's own six-line fixture *was* such a string, and the
/// live sweep reported this file at its own line 200. That was worked around by assembling the fixture at
/// run time, which left the defect in place and the workaround as a tax on whoever writes the next one.
///
/// **Tokens, because comments are what a lexer discards.** `proc_macro2` is already a dev-dependency here,
/// for the register that reads this repository's own Rust with a real parser instead of scanning it. A
/// literal's span covers the lines its text occupies, so a `//`-shaped line strictly inside one is shadowed
/// and every other stays a comment.
///
/// **Doc comments are literals too, and excluding them is the whole risk.** `/// text` reaches the lexer as
/// `#[doc = " text"]`, whose literal spans the doc comment's own line — so shadowing it would silently stop
/// this check from reading `///` paragraphs at all, which is most of this repository's prose and a false
/// negative rather than a tax. They are told apart by the only thing that distinguishes them: a literal
/// whose own first line is comment-shaped is a doc comment, because a real literal cannot begin on one.
///
/// A `text` the lexer refuses yields an empty set, which reports more rather than less; [`offences`] refuses
/// such a file separately, so the silence is not where the decision is made.
fn shadowed_by_a_literal(text: &str) -> BTreeSet<usize> {
    let mut shadowed = BTreeSet::new();
    let Ok(stream) = text.parse::<proc_macro2::TokenStream>() else {
        return shadowed;
    };
    let lines: Vec<&str> = text.lines().collect();
    let mut pending = vec![stream];
    while let Some(stream) = pending.pop() {
        for tree in stream {
            match tree {
                proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                proc_macro2::TokenTree::Literal(literal) => {
                    let span = literal.span();
                    let (first, last) = (span.start().line, span.end().line);
                    let opens_on_a_comment = lines
                        .get(first.saturating_sub(1))
                        .is_some_and(|line| line.trim_start().starts_with("//"));
                    if opens_on_a_comment {
                        continue;
                    }
                    shadowed.extend((first + 1)..=last);
                }
                _ => {}
            }
        }
    }
    shadowed
}

/// Every comment paragraph in `text` that is immediately followed by a copy of itself with identical line
/// content, as `(line of the second copy, how many lines it spans)`.
///
/// **Longest first, and non-overlapping.** A twelve-line paragraph written twice also contains a six-line
/// half written twice; reporting the longest run at a position and resuming past both copies names the
/// paste once, the way the author made it.
///
/// **Every line of the block must carry content.** Without that, two consecutive bare `//` lines — a
/// paragraph break spelled twice, which is a formatting matter and not a paste — are a one-line block
/// repeated, and this check would report a file whose text is exactly what its author wrote. That
/// requirement, not a minimum length, is what keeps formatting out: **a one-line paragraph is a paragraph**,
/// and the floor sat at two lines while the module above claimed every paragraph. Measured over the whole
/// tracked Rust corpus with the floor at one and at two: zero either way, so the wider reach costs no
/// report the author would argue with.
///
/// **Line content, not line bytes.** `str::lines` drops `\r\n` and `\n` alike, so a block terminated one way
/// and its copy terminated the other compare equal here and are reported. That is the reach this check
/// wants: a paste an editor re-terminated is still a paste, and comparing terminators would let it through —
/// a silent false negative, the one direction the Core Contract forbids.
fn repetitions(text: &str) -> Vec<(usize, usize)> {
    let lines: Vec<&str> = text.lines().collect();
    let shadowed = shadowed_by_a_literal(text);
    let total = lines.len();
    let mut found = Vec::new();
    let mut start = 0usize;

    while start < total {
        let longest = (total - start) / 2;
        let mut matched = None;
        for span in (1..=longest).rev() {
            let block = &lines[start..start + span];
            if block != &lines[start + span..start + 2 * span] {
                continue;
            }
            if block.iter().enumerate().any(|(offset, line)| {
                shadowed.contains(&(start + offset + 1))
                    || shadowed.contains(&(start + span + offset + 1))
                    || !matches!(comment_body(line), Some(body) if !body.is_empty())
            }) {
                continue;
            }
            matched = Some(span);
            break;
        }
        match matched {
            Some(span) => {
                found.push((start + span + 1, span));
                start += 2 * span;
            }
            None => start += 1,
        }
    }

    found
}

/// Every repeated paragraph under `root` among the Rust files `listing` names, and how many were read.
///
/// The count is returned for the reason its sibling in `whitespace_hygiene` returns one: an empty offence
/// list is what a clean corpus and a corpus that collapsed to nothing both produce, and those are opposite
/// facts. Here the corpus can collapse two ways — the enumeration answering nothing, and the `.rs` filter
/// matching nothing — and neither is visible in a verdict.
fn offences(root: &Path, listing: &[String]) -> (Vec<Refusal>, usize) {
    let mut offences = Vec::new();
    let mut inspected = 0usize;

    for path_str in listing {
        if !path_str.ends_with(".rs") {
            continue;
        }
        let content = match std::fs::read(root.join(path_str)) {
            Ok(content) => content,
            Err(err) => {
                offences.push(cannot_judge(format!(
                    "{path_str} is tracked Rust and could not be read ({err}), so whether it repeats a \
                     paragraph is unknown — an unread file is not a file that repeats nothing"
                )));
                continue;
            }
        };
        let Ok(text) = String::from_utf8(content) else {
            offences.push(cannot_judge(format!(
                "{path_str} is tracked Rust and is not UTF-8, so its comments were never read. A lossy \
                 decode would compare text this repository does not hold"
            )));
            continue;
        };
        // A file the lexer refuses is one whose comments this reader cannot separate from its strings. The
        // shadow map would be empty and the sweep would report more rather than less — loud, not silent —
        // but *more* here means reporting a file's own text back at its author, so the honest answer is that
        // the question was not decided.
        if text.parse::<proc_macro2::TokenStream>().is_err() {
            offences.push(cannot_judge(format!(
                "{path_str} is tracked Rust and does not lex, so a comment could not be told from a line of \
                 a string literal — an undecided file is not a file that repeats nothing"
            )));
            continue;
        }
        inspected += 1;

        for (line, span) in repetitions(&text) {
            offences.push(violation(format!(
                "{path_str}:{line}: a {span}-line comment paragraph is written twice in a row; the second \
                 copy begins here"
            )));
        }
    }

    (offences, inspected)
}

/// Every path `git` tracks, exactly as git spells it.
///
/// `-z` rather than a line-oriented read: git quotes a path carrying a special character, and a reader that
/// took the quoted spelling would open a file that is not there and report the tracked file it could not
/// read. The refusal would be honest and the corpus would still have lost the file.
///
/// **Through [`kanhe::hermetic_git::run_exact`], which refuses bytes it cannot represent.** Spelled here with its
/// own `Command`, this took git's stdout through `from_utf8_lossy` — and that runner's own header names
/// **this command** as the reason it does not: `ls-files -z` promises nothing about encoding, so a tracked
/// path that is not UTF-8 arrives as a different path than the one on disk, and every read downstream is
/// made against that. The offence would then name an identity the repository does not hold, which is the
/// property `xingbiao::path_identity` exists to keep and `repository_path` refuses in the same words. A
/// verdict is not owed on an input this reader cannot represent; saying so is.
///
/// `run_exact` rather than `run`, because that module assigns the two by what the caller does with the
/// answer: `run` trims for a caller reading a value, `run_exact` keeps the bytes for one comparing content,
/// and a NUL-separated path list is the second. The trim is harmless on this output — `\0` is not ASCII
/// whitespace, so nothing was being lost — but taking the accessor whose stated criterion fits is what keeps
/// that criterion true of its callers.
fn tracked(root: &Path) -> Vec<String> {
    match kanhe::hermetic_git::tracked_paths(root, &[]) {
        Ok(listing) => listing,
        Err(failure) => panic!(
            "CannotJudge: `git ls-files -z` did not answer a path list this reader can hold ({failure:?}), \
             so no file was inspected — an enumeration that could not be read is not an empty repository"
        ),
    }
}

#[test]
fn no_tracked_rust_file_writes_a_comment_paragraph_twice() {
    let Some(root) = workspace_root() else {
        return;
    };

    let listing = tracked(&root);
    let (offences, inspected) = offences(&root, &listing);

    assert!(
        inspected > 0,
        "no tracked Rust file was inspected, so this check would report clean over nothing — the vacuity \
         direction. {} tracked path(s) were enumerated",
        listing.len()
    );

    assert!(
        offences.is_empty(),
        "{inspected} tracked Rust file(s) inspected; a comment paragraph is written twice, or a tracked \
         file could not be read:\n{}",
        offences
            .iter()
            .map(|refusal| format!("  {:?}: {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The paragraph this check was written for is read, and the file around it is not.
///
/// The shape as it stood: a six-line comment paragraph, byte-identical, with ordinary source on both sides.
/// Supplied here rather than cited from history, because a direction resting on the live corpus stops
/// guarding the moment the corpus is repaired — the defect the citation reader's own scenario carried, and
/// the reason that one now supplies the spans it reads.
///
/// **Written out, deliberately, and that is a second direction.** The first spelling of this fixture was
/// exactly this text and the live sweep reported *this file* at its own line 200, because a comment-shaped
/// line was a comment wherever it stood. It was then assembled at run time to get out of the way — a
/// workaround that left the defect in place and made a tax of it. The classification now separates a comment
/// from a string, so the fixture is written out again: it is a duplicated comment paragraph inside a string
/// literal, sitting in a file the live sweep reads, which makes this file a live instance of the class
/// rather than a description of one. If the shadow ever stops working, the sweep says so here.
///
/// The block is a `mod` and not a `fn` because a sibling check states the convention: embedded Rust sits
/// behind a call, never opening a line of its own, so a line whose trimmed text starts with `fn ` and is
/// emptied by the refusal register's span reader is a declaration that reader lost. Written as a `fn`, this
/// fixture tripped it — one guard catching what another had just been taught to ignore.
#[test]
fn a_paragraph_pasted_twice_is_read_and_its_neighbours_are_not() {
    const OPENS: &str = "    // **Presence";
    let repeated = "\
mod judge {
    // **Presence is asked first, by a command whose exit status answers it.** `show HEAD:` exits `128`
    // for a path that is not in HEAD *and* for a tree it cannot read, so a single `Err` arm had to choose
    // one meaning for both — and choosing *not a snapshot* classified a broken object store as a cycle.
    // Measured: `ls-tree HEAD -- <path>` exits `0` with an empty listing when the path is absent, `0` with
    // a line when it is there, and `128` only when the tree cannot be read. The question decides the
    // command, which is what the sibling tag-presence reader already does for the same shape.
    // **Presence is asked first, by a command whose exit status answers it.** `show HEAD:` exits `128`
    // for a path that is not in HEAD *and* for a tree it cannot read, so a single `Err` arm had to choose
    // one meaning for both — and choosing *not a snapshot* classified a broken object store as a cycle.
    // Measured: `ls-tree HEAD -- <path>` exits `0` with an empty listing when the path is absent, `0` with
    // a line when it is there, and `128` only when the tree cannot be read. The question decides the
    // command, which is what the sibling tag-presence reader already does for the same shape.
    const LISTED: () = ();
}
";
    assert_eq!(
        repetitions(repeated),
        vec![(8, 6)],
        "the six-line paste must be read once, at the line its second copy begins"
    );

    // The control: with one copy, the same source is silent — so the reading above is about the repetition
    // rather than about a check that reports whatever it is shown. The halves are found by the second
    // occurrence rather than by halving the bytes, which the em dashes above would not survive.
    let opens = repeated.find(OPENS).expect("the paragraph opens the block");
    let closes = repeated
        .rfind("    const LISTED")
        .expect("and the block closes after it");
    let doubled = &repeated[opens..closes];
    let second = doubled[1..]
        .find(OPENS)
        .expect("the paragraph stands twice")
        + 1;
    let single = repeated.replacen(doubled, &doubled[..second], 1);
    assert_ne!(single, repeated, "the control must differ from the subject");
    assert_eq!(
        repetitions(&single),
        Vec::new(),
        "one copy of the paragraph is not a repetition"
    );
}

/// A paragraph break written twice is formatting, not a paste.
///
/// The narrowing that keeps this check from reporting text its author wrote: without the content
/// requirement, four bare `//` lines are a two-line block repeated.
#[test]
fn consecutive_empty_comment_lines_are_not_a_repetition() {
    assert_eq!(repetitions("//\n//\n//\n//\n"), Vec::new());
    assert_eq!(repetitions("/// a\n//\n//\n//\n//\n/// b\n"), Vec::new());
}

/// Repeated code is not this check's question.
///
/// The shape is live in this repository and it is deliberate: `runner/tests.rs` passes `--manifest-path`
/// twice to assert the duplicate flag exits `2`, and `hunyi`'s test helper spells the same parameter types
/// twice in one function type. A reader keyed on identical lines rather than on identical *comment* lines
/// reports them, which is the tax that would make this check something to work around.
#[test]
fn identical_code_lines_are_not_read() {
    assert_eq!(
        repetitions(
            "    \"--manifest-path\",\n    &fixture(\"clean\"),\n    \"--manifest-path\",\n    &fixture(\"clean\"),\n"
        ),
        Vec::new()
    );
}

/// A repetition the reader is not adjacent to is outside its declared reach.
///
/// Split from `identical_code_lines_are_not_read`, which held this fact under a name that states the other
/// one. The out-of-line bound cites a direction, and a reader following that citation has to land on a name
/// that says what the bound says — an identifier is a carrier of a claim, which is this repository's own
/// rule about names.
#[test]
fn a_repetition_split_by_code_is_not_read() {
    assert_eq!(
        repetitions(
            "    // a note\n    // and its second line\n    let x = 1;\n    // a note\n    // and its second line\n"
        ),
        Vec::new(),
        "a repetition that is not adjacent is outside this check's declared reach"
    );
}

/// A copy terminated differently from the block it copies is still a copy.
///
/// The reach the requirement now states in place of *byte-identical*. `str::lines` drops `\r\n` and `\n`
/// alike, so this comparison is over line content — deliberately, because a paste an editor re-terminated is
/// still a paste and requiring the terminators to agree would let it through. Unreachable in this tree
/// today: every tracked file is `i/lf` and there is no `.gitattributes`, so the direction supplies the shape
/// itself rather than resting on a corpus that would have to acquire it.
#[test]
fn a_copy_terminated_differently_is_still_read() {
    let crlf_then_lf = "// a note\r\n// and its second line\r\n// a note\n// and its second line\n";
    assert_eq!(
        repetitions(crlf_then_lf),
        vec![(3, 2)],
        "the terminators differ and the paragraph is the same, which is the paste this reads"
    );
}

/// A one-line comment paragraph written twice is a paragraph written twice.
///
/// The stop that was in the reader and in none of its declarations: the span floor sat at two lines while
/// the module doc claimed every paragraph. Lowering it to one reports zero across the whole tracked Rust
/// corpus, so the wider reach was bought at no cost — measured both ways before it moved.
#[test]
fn a_one_line_paragraph_written_twice_is_read() {
    assert_eq!(
        repetitions("fn f() {\n    // the same note\n    // the same note\n    let x = 1;\n}\n"),
        vec![(3, 1)],
        "a single comment line repeated is the shortest paste there is"
    );
}

/// A comment-shaped line inside a string literal is text, and a doc comment is still a comment.
///
/// The two halves of the same classification, in one direction because getting either wrong breaks the
/// other. Shadowing too little reports Rust fixtures held as strings — this check's own first fixture was
/// one. Shadowing too much would take `///` paragraphs with it, since a doc comment reaches the lexer as
/// `#[doc = "…"]` and its literal spans the comment's own line; that direction is a **false negative** over
/// most of this repository's prose, which is the one the Core Contract forbids.
#[test]
fn a_comment_shaped_line_inside_a_literal_is_not_a_comment() {
    let plain = "fn f() {\n    let fixture = \"\\\n    // a note\n    // a note\n    \";\n}\n";
    assert_eq!(
        repetitions(plain),
        Vec::new(),
        "a duplicated comment paragraph inside a string literal is that string's text"
    );

    let raw = "fn f() {\n    let fixture = r#\"\n// a note\n// a note\n\"#;\n}\n";
    assert_eq!(
        repetitions(raw),
        Vec::new(),
        "and the same inside a raw string, whose contents no escape can end early"
    );

    // The control on the other side: the same paragraph written as real comments is still read, and so is a
    // doc comment, whose literal the lexer synthesises over the comment's own line.
    assert_eq!(
        repetitions("fn f() {\n    // a note\n    // a note\n}\n"),
        vec![(3, 1)],
        "a real comment paragraph is unaffected by the shadow"
    );
    assert_eq!(
        repetitions(
            "/// a note\n/// and its second line\n/// a note\n/// and its second line\nfn f() {}\n"
        ),
        vec![(3, 2)],
        "a doc comment is a comment: shadowing its synthesised literal would stop this check reading `///`"
    );
}

/// A prose file is not this check's corpus, and the reason is not that prose repeats less.
///
/// Markdown repeats identical adjacent lines for its own reasons — a table's rule row, two list items that
/// happen to read the same, a fenced block quoted beside the text that explains it — and none of them is a
/// paste. Reading `.md` here would buy a class this repository has never met with a false positive it meets
/// routinely, which is the trade `PROJECT.md` refuses. The stop is declared rather than assumed, and this
/// direction is where it is pinned: the same duplicated paragraph that fires in a `.rs` file is silent in a
/// `.md` one, and the file is reported as uninspected rather than as clean.
#[test]
fn a_repeated_paragraph_in_a_prose_file_is_outside_the_corpus() {
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-repeated-paragraph-prose-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    let body = "// a paragraph worth pasting\n// and its second line\n// a paragraph worth pasting\n// and its second line\n";
    std::fs::write(scratch.join("probe.rs"), body).expect("write the Rust probe");
    std::fs::write(scratch.join("probe.md"), body).expect("write the prose probe");

    let (rust, rust_inspected) = offences(&scratch, &["probe.rs".to_string()]);
    let (prose, prose_inspected) = offences(&scratch, &["probe.md".to_string()]);
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(rust_inspected, 1);
    assert_eq!(
        rust.len(),
        1,
        "the same text must fire inside the corpus: {rust:?}"
    );
    assert_eq!(rust[0].kind, Kind::Violation);

    assert_eq!(
        prose_inspected, 0,
        "a prose file is outside the corpus, so it is not counted as inspected — a stop, not a clean read"
    );
    assert!(prose.is_empty(), "and it is not reported: {prose:?}");
}

/// A tracked path that is not UTF-8 is refused, never renamed.
///
/// The property this file's enumeration stands on — held over [`tracked`] itself, not over the runner it
/// calls. Written against `hermetic_git::run` the direction passed with the defect restored, because the
/// defect was `tracked` reaching past that runner; a direction that observes a dependency cannot see its
/// caller stop using it.
///
/// `ls-files -z` hands back git's raw bytes, so an undecodable one arrives intact. A lossy decode
/// substitutes U+FFFD per byte and answers a path the repository does not hold, which is then opened,
/// missed, and reported as *a tracked file that could not be read* — a refusal that is honest about the
/// wrong file. The control is that measurement: git's spelling and the lossy one differ, so the file
/// reported would not be the file tracked.
#[cfg(unix)]
#[test]
fn a_tracked_path_that_is_not_utf8_is_refused_rather_than_renamed() {
    use std::os::unix::ffi::OsStrExt;

    let scratch = std::env::temp_dir().join(format!(
        "tianheng-repeated-paragraph-not-utf8-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    std::fs::write(scratch.join("plain.rs"), "fn main() {}\n").expect("write the decodable probe");
    for args in [
        &["init", "-q", "."][..],
        &["config", "user.email", "probe@example.invalid"][..],
        &["config", "user.name", "probe"][..],
        &["add", "-A"][..],
        &["commit", "-qm", "probe"][..],
    ] {
        kanhe::hermetic_git::run(&scratch, &[], args).expect("the fixture repository is built");
    }

    // The control first: with every path decodable, this enumeration answers them.
    let listed = tracked(&scratch);
    assert_eq!(
        listed,
        vec!["plain.rs".to_string()],
        "the enumeration reads a repository whose paths it can represent"
    );

    let undecodable = std::ffi::OsStr::from_bytes(b"probe-\xff.rs");
    std::fs::write(scratch.join(undecodable), "// a note\n// a note\n")
        .expect("write the undecodable probe");
    for args in [&["add", "-A"][..], &["commit", "-qm", "undecodable"][..]] {
        kanhe::hermetic_git::run(&scratch, &[], args).expect("the undecodable path is tracked");
    }

    let refused = std::panic::catch_unwind(|| tracked(&scratch));
    let _ = std::fs::remove_dir_all(&scratch);

    let payload = refused.expect_err(
        "a path this reader cannot represent must stop the enumeration, not be decoded into another name",
    );
    let said = payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("");
    assert!(
        said.contains("CannotJudge") && said.contains("ls-files"),
        "the refusal names what could not be read: {said:?}"
    );

    // And the measurement the refusal exists for: decoded lossily, the path is a name nothing tracks.
    let lossy = String::from_utf8_lossy(b"probe-\xff.rs").into_owned();
    assert!(
        lossy.contains('\u{fffd}') && lossy != "probe-\u{ff}.rs",
        "a lossy decode substitutes U+FFFD, so the name reported would not be the name tracked: {lossy:?}"
    );
}

/// An unreadable tracked file is a cannot-judge, not a file that repeats nothing.
#[test]
fn an_unreadable_tracked_rust_file_is_refused_rather_than_skipped() {
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-repeated-paragraph-unreadable-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    let listing = vec!["zzz_absent_paragraph_probe.rs".to_string()];
    let (offences, inspected) = offences(&scratch, &listing);
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].kind,
        Kind::CannotJudge,
        "an unread file is not a file without offences, so it is not a violation"
    );
    assert!(
        offences[0]
            .message
            .contains("zzz_absent_paragraph_probe.rs"),
        "the refusal must name the file it could not read, got {:?}",
        offences[0].message
    );
    assert_eq!(inspected, 0);
}

/// A file the lexer refuses is undecided, not a file that repeats a paragraph.
///
/// **The two answers this separates are a refusal and an accusation.** Without the lex question,
/// `shadowed_by_a_literal` yields an empty shadow map for such a file — a lexer that could not run
/// shadows nothing — so every comment-shaped line inside its string literals is read as a comment and
/// a doubled one is reported as an offence. That is loud rather than silent, which is the safe
/// direction in general; here *more* means reporting a file's own literal text back at its author as a
/// repetition they did not write. The honest answer is that the question was not decided.
///
/// The fixture is an unterminated string literal wrapping a doubled comment-shaped paragraph, so it
/// reaches both halves at once: `proc_macro2` refuses it, and the paragraph inside it is exactly what
/// the empty shadow map would let through.
#[test]
fn a_tracked_rust_file_that_does_not_lex_is_undecided_rather_than_accused() {
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-repeated-paragraph-does-not-lex-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");

    // The control first: the identical paragraph with the literal TERMINATED is a genuine repetition
    // inside a string, which the shadow map keeps out — so the assertion below is about the lexer
    // having refused this file, not about the paragraph being unreachable either way.
    let terminated = "fn f() {\n    let s = \"\n    // a note\n    // a note\n    \";\n}\n";
    std::fs::write(scratch.join("terminated.rs"), terminated).expect("write the control");
    let (control, inspected) = offences(&scratch, &["terminated.rs".to_string()]);
    assert!(
        control.is_empty(),
        "a comment-shaped line inside a terminated literal is not a comment: {control:?}"
    );
    assert_eq!(inspected, 1, "the control file was read");

    let unterminated = "fn f() {\n    let s = \"\n    // a note\n    // a note\n}\n";
    std::fs::write(scratch.join("unterminated.rs"), unterminated).expect("write the probe");
    let (offences, inspected) = offences(&scratch, &["unterminated.rs".to_string()]);
    let _ = std::fs::remove_dir_all(&scratch);

    assert_eq!(offences.len(), 1, "{offences:?}");
    assert_eq!(
        offences[0].kind,
        Kind::CannotJudge,
        "a file whose comments could not be told from its literals is undecided, never an offence: {:?}",
        offences[0]
    );
    assert!(
        offences[0].message.contains("unterminated.rs")
            && offences[0].message.contains("does not lex"),
        "the refusal names the file and what it could not decide, got {:?}",
        offences[0].message
    );
    assert_eq!(
        inspected, 0,
        "an undecided file is not an inspected one, so it must not count toward the vacuity floor"
    );
}
