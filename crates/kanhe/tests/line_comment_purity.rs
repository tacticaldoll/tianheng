//! Repository check: no published crate's `src/*.rs` carries an inner (`//`) comment.
//!
//! The rule is about **attention**, not aesthetics: a code diff that carries prose forces every
//! reader — human or agent — to run NLP judgment over it, and this repository has measured three
//! times that such judgment is undecidable by instrument. Moving the prose out of the diff (into
//! the DSL's `because`, into test names, into specs) is the only convergent repair.
//!
//! **The corpus is the published crates' `src` files, excluding `src/tests/` and
//! `src/tests.rs`.** Test files may carry explanatory prose. An inline `#[cfg(test)]`
//! module in another source file remains in the corpus because this boundary is by file
//! path. The scanner skips `//` inside strings; doc comments (`///`, `//!`) are out of
//! scope because they carry the item's contract.

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

/// Every offence in `sources`: one per inner comment, naming the path, the line, and the text.
fn offences_of(sources: &[(String, String)]) -> Vec<Refusal> {
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

/// Judge exactly the source set derived from publishable manifests.
fn published_source_offences(root: &Path) -> (usize, Vec<Refusal>) {
    let sources = published_sources(root);
    let offences = offences_of(&sources);
    (sources.len(), offences)
}

/// No published crate carries an inner comment in its governed `src` files.
#[test]
fn no_published_source_carries_an_inner_comment() {
    let Some(root) = workspace_root() else {
        return;
    };
    let (inspected, offences) = published_source_offences(&root);

    assert!(
        inspected > 0,
        "no published source was inspected, so this check would report clean over nothing — the \
         vacuity direction"
    );

    assert!(
        offences.is_empty(),
        "{} published source file(s) inspected; an inner comment is prose in a code diff, and \
         prose in a code diff forces NLP judgment on every reader:\n{}",
        inspected,
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
    let offences = offences_of(&sources);
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0]
            .message
            .contains("crates/example/src/dirty.rs:2: // prose in the diff"),
        "the refusal names the path, the line, and the text: {}",
        offences[0].message
    );
}

/// A newly publishable crate enters the same corpus the gate judges; an unpublished one does not.
#[test]
fn a_new_published_crate_with_an_inner_comment_is_refused() {
    let root = std::env::temp_dir().join(format!(
        "kanhe-line-comment-new-crate-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create fixture root");
    for (name, publish, source) in [
        ("existing", "", "pub fn clean() {}\n"),
        ("newly_published", "", "// newly published prose\n"),
        ("unpublished", "publish = false\n", "// outside corpus\n"),
    ] {
        let dir = root.join("crates").join(name);
        std::fs::create_dir_all(dir.join("src")).expect("create source directory");
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\n{publish}"),
        )
        .expect("write manifest");
        std::fs::write(dir.join("src/lib.rs"), source).expect("write source");
    }
    let git = |args: &[&str]| kanhe::hermetic_git::fixture(&root, "git", args);
    git(&["init", "-q", "."]);
    git(&["add", "crates"]);

    let (inspected, offences) = published_source_offences(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(inspected, 2);
    assert_eq!(offences.len(), 1);
    assert!(
        offences[0]
            .message
            .contains("crates/newly_published/src/lib.rs:1")
    );
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
