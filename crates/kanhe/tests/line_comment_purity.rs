//! Repository check: no published crate's `src/*.rs` carries an inner (`//`) comment.
//!
//! The rule is about **attention**, not aesthetics: a code diff that carries prose forces every
//! reader — human or agent — to run NLP judgment over it, and this repository has measured three
//! times that such judgment is undecidable by instrument. Moving the prose out of the diff (into
//! the DSL's `because`, into test names, into specs) is the only convergent repair.
//!
//! **The corpus is the published crates' `src/*.rs` directly, not their `src/tests/`** — test
//! fixtures carry `"// clean\n"` as string data, and excluding them is the boundary between
//! "this file's code carries no prose" and "this file's test data happens to contain `//`".
//! Doc comments (`///`, `//!`) are out of scope: they are the item's contract, not prose
//! about the implementation.

use std::path::{Path, PathBuf};

use kanhe::comment_scan;
use kanhe::refusal::{Refusal, violation};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every workspace member cargo will publish, derived rather than listed.
fn published_crates(root: &Path) -> Vec<String> {
    let mut published = Vec::new();
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/ enumerates") {
        let dir = entry.expect("a crates/ entry").path();
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).expect("a member manifest reads");
        match kanhe::manifest::publishable(&text) {
            kanhe::manifest::Publishable::Yes => published.push(
                dir.file_name()
                    .expect("a crate directory has a name")
                    .to_string_lossy()
                    .into_owned(),
            ),
            kanhe::manifest::Publishable::No => {}
            unreadable @ kanhe::manifest::Publishable::Unreadable(_) => panic!(
                "CannotJudge: {}: whether this crate publishes cannot be decided from its manifest \
                 ({unreadable:?}), so the corpus this sweep reads would be a guess",
                manifest.display()
            ),
        }
    }
    published.sort();
    assert!(
        published.len() > 1,
        "{} publishable crates were derived from crates/*/Cargo.toml — this family has several, and a sweep \
         over one is not the subject this check claims",
        published.len()
    );
    published
}

/// Every tracked `.rs` file under the published crates' `src`, excluding `src/tests/`,
/// as `(path, text)`.
fn published_sources(root: &Path) -> Vec<(String, String)> {
    let published = published_crates(root);
    let dirs: Vec<String> = published
        .iter()
        .map(|krate| format!("crates/{krate}/src"))
        .collect();
    let pathspec: Vec<&str> = dirs.iter().map(String::as_str).collect();
    let listing = kanhe::hermetic_git::tracked_paths(root, &pathspec).unwrap_or_else(|failure| {
        panic!("CannotJudge: `git ls-files` over the published crates' sources: {failure:?}")
    });
    let files: Vec<(String, String)> = listing
        .iter()
        .map(String::as_str)
        .filter(|path| path.ends_with(".rs"))
        .filter(|path| !path.contains("/tests/") && !path.ends_with("/tests.rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
                panic!("CannotJudge: cannot read the tracked file {path} ({err})")
            });
            (path.to_string(), text)
        })
        .collect();
    // Vacuity guard: every published crate must contribute at least one file.
    for krate in &published {
        let prefix = format!("crates/{krate}/src/");
        assert!(
            files.iter().any(|(path, _)| path.starts_with(&prefix)),
            "`git ls-files` enumerated no source under {prefix} — this sweep would report clean over a crate \
             it never opened. The corpus is every published crate's `src` (excluding `src/tests/` and \
             `src/tests.rs`), and one of them is missing"
        );
    }
    files
}

/// The crates the reaction currently holds to zero inner comments.
///
/// **Incremental rollout, not a baseline.** The published set carries a standing population of `//`
/// comments that a single sweep cannot remove without a false-red start. Each crate enters this list
/// only after its own `//` population has been moved out — falsifiers into test names, provenance
/// dropped, product contracts into specs — and the gate below sweeps only the listed crates'
/// sources. A crate not yet listed is not exempt: it is next.
///
/// **The list is hand-written, and each entry is a measurement made before the write.** `xuanji`,
/// `xingbiao`, `louke`, `tianheng`, `guibiao`, and `hunyi` are here because their `//` count outside `tests.rs` was measured at zero
/// before the entry landed; every later entry earns its place the same way — the crate's `//`
/// population moved first, then its name was written here. The per-crate guard in the gate below
/// refuses any listed name that contributes no file, so a typo or an unpublished crate cannot enter
/// silently.
fn enforced_crates() -> Vec<&'static str> {
    vec![
        "guibiao", "hunyi", "louke", "tianheng", "xingbiao", "xuanji",
    ]
}

/// The sources the enforced crates contribute to the sweep.
fn enforced_sources<'a>(
    all_sources: &'a [(String, String)],
    enforced: &[&str],
) -> Vec<&'a (String, String)> {
    all_sources
        .iter()
        .filter(|(path, _)| {
            enforced
                .iter()
                .any(|krate| path.starts_with(&format!("crates/{krate}/src/")))
        })
        .collect()
}

/// Every offence in `sources`: one per inner comment, naming the path, the line, and the text.
fn offences_of(sources: &[&(String, String)]) -> Vec<Refusal> {
    let mut offences = Vec::new();
    for (path, text) in sources {
        for comment in comment_scan::line_comments(text) {
            offences.push(violation(format!(
                "{path}:{}: {}",
                comment.line,
                comment.text.trim()
            )));
        }
    }
    offences
}

/// Every enforced name that contributed no source — a typo, or an unpublished crate, which can
/// never appear in the sweep at all. Returned rather than asserted so a direction can hold the
/// guard against a name it knows is absent; asserted inline, a broken entry would report clean
/// forever with no direction ever seeing the failure.
fn missing_enforced_crates<'a>(
    enforced: &[&'a str],
    sources: &[&(String, String)],
) -> Vec<&'a str> {
    enforced
        .iter()
        .filter(|krate| {
            let prefix = format!("crates/{krate}/src/");
            !sources.iter().any(|(path, _)| path.starts_with(&prefix))
        })
        .copied()
        .collect()
}

/// No published crate in the enforced set carries an inner comment in its `src/*.rs`.
#[test]
fn no_published_source_carries_an_inner_comment() {
    let Some(root) = workspace_root() else {
        return;
    };
    let enforced = enforced_crates();
    let all_sources = published_sources(&root);
    let sources = enforced_sources(&all_sources, &enforced);

    assert!(
        !sources.is_empty(),
        "no enforced source was inspected, so this check would report clean over nothing — the \
         vacuity direction"
    );

    // Per-crate vacuity: the corpus narrower than the claim, at one crate's granularity rather
    // than the whole sweep's.
    let missing = missing_enforced_crates(&enforced, &sources);
    assert!(
        missing.is_empty(),
        "the enforced list names {} crate(s) no source entered the sweep for ({missing:?}) — a typo \
         or an unpublished crate, and either way the sweep would report clean over a crate it never \
         opened",
        missing.len()
    );

    let offences = offences_of(&sources);
    assert!(
        offences.is_empty(),
        "{} published source file(s) inspected; an inner comment is prose in a code diff, and \
         prose in a code diff forces NLP judgment on every reader:\n{}",
        sources.len(),
        offences
            .iter()
            .map(|refusal| format!("  {:?}: {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The gate's own judgement, driven red: a corpus carrying one inner comment produces a
/// violation naming the path, the line, and the text. Without this direction the gate's
/// `violation(…)` site is asserted only by the clean tree — a guard never seen to fail.
#[test]
fn a_corpus_with_an_inner_comment_is_refused() {
    let sources = [
        (
            "crates/example/src/lib.rs".to_string(),
            "fn clean() {}\n".to_string(),
        ),
        (
            "crates/example/src/dirty.rs".to_string(),
            "fn head() {}\n// prose in the diff\n".to_string(),
        ),
    ];
    let refs: Vec<&(String, String)> = sources.iter().collect();
    let offences = offences_of(&refs);
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0]
            .message
            .contains("crates/example/src/dirty.rs:2: // prose in the diff"),
        "the refusal names the path, the line, and the text: {}",
        offences[0].message
    );
}

/// The per-crate guard, driven red: an enforced name no source backs is surfaced rather
/// than silently ignored.
#[test]
fn an_enforced_crate_with_no_source_is_surfaced() {
    let sources = [(
        "crates/xuanji/src/lib.rs".to_string(),
        "fn f() {}\n".to_string(),
    )];
    let refs: Vec<&(String, String)> = sources.iter().collect();
    let missing = missing_enforced_crates(&["xuanji", "xuanji-typoed"], &refs);
    assert_eq!(missing, ["xuanji-typoed"]);
}

/// The reader separates a comment from a string, a doc comment, and a block comment.
#[test]
fn the_reader_separates_a_comment_from_a_string() {
    let cases: Vec<(&str, usize)> = [
        ("// real\n", 1),
        ("/// doc\n", 0),
        ("//! inner doc\n", 0),
        ("//// plain — a fourth slash is not a doc comment\n", 1),
        ("///// also plain\n", 1),
        (r#"let s = "// not";"#, 0),
        (r##"let s = r#"// not"#;"##, 0),
        ("/* // not */\n", 0),
        ("'x' // real\n", 1),
        ("&'a str // real\n", 1),
        ("let x = 1; // real\n", 1),
    ]
    .into_iter()
    .collect();

    for (source, expected) in cases {
        let found = comment_scan::line_comments(source);
        assert_eq!(
            found.len(),
            expected,
            "source: {source:?} — expected {expected} comment(s), found {}",
            found.len()
        );
    }
}

/// A comment's reported line is the line it is on, after multi-line constructs.
#[test]
fn the_reader_counts_lines_through_skipped_constructs() {
    let cases: Vec<(&str, usize)> = [
        ("/* one\ntwo */\n// real\n", 3),
        ("let s = r#\"\n\n\"#;\n// real\n", 4),
        ("let s = \"one\\\ntwo\";\n// real\n", 3),
        ("let s = \"one\ntwo\"; // real\n", 2),
    ]
    .into_iter()
    .collect();

    for (source, expected_line) in cases {
        let found = comment_scan::line_comments(source);
        assert_eq!(found.len(), 1, "source: {source:?}");
        assert_eq!(
            found[0].line, expected_line,
            "source: {source:?} — the comment is on line {expected_line}, reported {}",
            found[0].line
        );
    }
}

/// Every tracked `.rs` file under `crates/` (including tests and unpublished crates),
/// as `(path, text)`.
fn all_tracked_sources(root: &Path) -> Vec<(String, String)> {
    let listing = kanhe::hermetic_git::tracked_paths(root, &["crates"])
        .unwrap_or_else(|failure| panic!("CannotJudge: `git ls-files` over `crates`: {failure:?}"));
    listing
        .iter()
        .map(String::as_str)
        .filter(|path| path.ends_with(".rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
                panic!("CannotJudge: cannot read the tracked file {path} ({err})")
            });
            (path.to_string(), text)
        })
        .collect()
}

/// Every comment reported over the real tree carries text that appears on the line it is
/// reported at — the consistency property a `{path}:{line}` diagnostic lives or dies by.
///
/// The oracle is the source itself: if the line number is wrong, the reported text is not
/// on that line. This is what would have caught the newline-loss defect the first version of
/// the scanner carried, measured at half the tree's comments before the skip helpers learned
/// to count the newlines they cross.
#[test]
fn every_reported_line_holds_its_own_text() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut checked = 0usize;
    for (path, text) in all_tracked_sources(&root) {
        let lines: Vec<&str> = text.lines().collect();
        for comment in comment_scan::line_comments(&text) {
            checked += 1;
            let reported = lines.get(comment.line - 1).copied().unwrap_or("");
            let line = comment.line;
            let comment_text = comment.text.trim();
            let reported_text = reported.trim();
            assert!(
                reported.contains(comment.text.trim_end()),
                "{path}:{line}: the scanner reports `{comment_text}` here, but that line reads \
                 `{reported_text}` — the line number is wrong",
            );
        }
    }
    assert!(
        checked > 0,
        "no comment was checked, so this consistency property would hold over nothing"
    );
}
