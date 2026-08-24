//! Repository check: a doc comment in a published crate does not index its own provenance by review round.
//!
//! `AGENTS.md`'s *What earns a place in a doc comment* table settles the disposition: **a review round
//! number, a pull request number → provenance**, while *a past defect described with the invariant it
//! violated* keeps the invariant and drops the debrief. Twenty-eight doc lines across five published crates
//! carried the round number anyway, eleven of them as the index to `see PROJECT.md's Decisions` — and
//! `PROJECT.md` holds **no** entry organised by round, so those eleven pointed at a structure that does not
//! exist. Measured at `11674bc`, before the cleanup this check now holds.
//!
//! **Not an adopter-facing defect, and saying so is the honest scope.** None of the twenty-eight attached to
//! a `pub` item — ten private, eight `pub(crate)`, one private-module `//!` — so `cargo doc --no-deps`
//! generates none of them and docs.rs shows none. The reader is whoever opens the source, which includes an
//! agent with it in context, and that is reason enough: the round number names *when*, not *what*, and
//! nothing downstream reads it.
//!
//! **The corpus stops at doc comments, deliberately, and the stop is observed rather than claimed.** A `//`
//! inner comment is a note to whoever edits the line, not part of any item's contract, and twenty-seven of
//! them carry a round number today. `BACKLOG.md` carries that residue. The precision direction below gives
//! this reader both forms and requires it to separate them, so the boundary is a property of a run.

use std::path::{Path, PathBuf};

/// The crates whose `src` this reads: every workspace member that publishes.
///
/// A literal beside no enumerator would be the shape this repository refuses, so it is held against
/// `publish = false` in the manifests by [`the_published_set_is_the_one_the_manifests_declare`].
const PUBLISHED: [&str; 6] = [
    "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
];

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/hunyi/src/exposure.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Whether a line names a review round — `round 9`, `round-10`, `round-11→round-12`.
///
/// Hand-written rather than a pattern crate: `kanhe` takes `serde_json` for cargo's message stream and
/// nothing else, and this is one shape. `rounds to 3 decimals` does not match — the token is `round`
/// followed immediately by a hyphen or a space and then a digit.
fn names_a_round(text: &str) -> bool {
    let mut at = 0;
    while let Some(found) = text[at..].find("round") {
        let after = at + found + "round".len();
        let rest = &text[after..];
        let mut chars = rest.chars();
        if let Some(separator) = chars.next() {
            if (separator == '-' || separator == ' ')
                && chars.next().is_some_and(|c| c.is_ascii_digit())
            {
                return true;
            }
        }
        at = after;
    }
    false
}

/// Whether a line is a doc comment rather than an inner one.
fn is_doc_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with("///") || trimmed.starts_with("//!")
}

/// Every tracked `.rs` file under the published crates' `src`, as `(path, text)`.
///
/// Read through `kanhe::hermetic_git`, so no ambient git configuration decides which repository answers.
fn published_sources(root: &Path) -> Vec<(String, String)> {
    let dirs: Vec<String> = PUBLISHED
        .iter()
        .map(|krate| format!("crates/{krate}/src"))
        .collect();
    let mut args = vec!["ls-files"];
    args.extend(dirs.iter().map(String::as_str));
    let listing = kanhe::hermetic_git::read(
        root,
        "`git ls-files` over the published crates' sources",
        "git",
        &args,
    );
    let files: Vec<(String, String)> = listing
        .lines()
        .filter(|path| path.ends_with(".rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path))
                .unwrap_or_else(|err| panic!("cannot read the tracked file {path} ({err})"));
            (path.to_string(), text)
        })
        .collect();
    // The enumeration is an input like any other: a corpus that collapsed to nothing satisfies "no doc
    // comment names a round" exactly as a clean one does.
    //
    // **The property, not a threshold.** A `> 50` floor stood here against an actual 131 — a number answering
    // a question nobody asked, and one that says nothing about a single crate dropping out. What the floor was
    // standing in for is that **every** published crate contributed, which is what a lost `ls-files` argument
    // or a renamed directory would break, and which no count can see.
    for krate in PUBLISHED {
        let prefix = format!("crates/{krate}/src/");
        assert!(
            files.iter().any(|(path, _)| path.starts_with(&prefix)),
            "`git ls-files` enumerated no source under {prefix} — this sweep would report clean over a crate \
             it never opened. The corpus is every published crate's `src`, and one of them is missing"
        );
    }
    files
}

/// No doc comment under a published crate's `src` names a review round.
#[test]
fn no_doc_comment_in_a_published_crate_indexes_its_provenance_by_round() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut offences = Vec::new();
    for (path, text) in published_sources(&root) {
        for (number, line) in text.lines().enumerate() {
            if is_doc_line(line) && names_a_round(line) {
                offences.push(format!("  {path}:{}: {}", number + 1, line.trim()));
            }
        }
    }
    assert!(
        offences.is_empty(),
        "a doc comment indexes its own provenance by review round — the number names when, not what, and \
         nothing downstream reads it. Keep the invariant the passage carries and drop the round:\n{}",
        offences.join("\n")
    );
}

/// The reader separates a doc comment from an inner one, and a round from a rounding.
///
/// **Both directions, because either alone reads as the other's defect.** A reader that took every `//`
/// would report twenty-seven sites this check declares out of its corpus; one that took no `///` would
/// report none of the twenty-eight it exists for.
#[test]
fn the_reader_separates_a_doc_comment_from_an_inner_one() {
    for (line, doc, round) in [
        ("/// found on a round-9 adversarial review", true, true),
        (
            "//! see PROJECT.md's Decisions, the round-5 addendum",
            true,
            true,
        ),
        ("    /// the round 6 fix for the use-map", true, true),
        ("// found on a round-9 adversarial review", false, true),
        ("    // round-11→round-12", false, true),
        (
            "/// the shared filter every consuming rule needs",
            true,
            false,
        ),
        ("/// rounds to 3 decimal places", true, false),
        ("/// a round table of consumers", true, false),
        ("let x = 1; // round 9", false, true),
    ] {
        assert_eq!(is_doc_line(line), doc, "doc-ness of {line:?}");
        assert_eq!(names_a_round(line), round, "round-ness of {line:?}");
    }
}

/// The published set this check reads is the set the manifests declare.
///
/// A literal beside no enumerator agrees with nothing. Held **both ways**, because one direction catches an
/// omission and misses a member that has outlived its subject.
#[test]
fn the_published_set_is_the_one_the_manifests_declare() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut declared: Vec<String> = Vec::new();
    for entry in std::fs::read_dir(root.join("crates")).expect("crates/ enumerates") {
        let dir = entry.expect("a crates/ entry").path();
        let manifest = dir.join("Cargo.toml");
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).expect("a member manifest reads");
        let unpublished = kanhe::region::Source::of(text.as_str())
            .toml()
            .lines()
            .any(|line| {
                let trimmed = line.trim();
                trimmed.starts_with("publish") && trimmed.contains("false")
            });
        if !unpublished {
            declared.push(
                dir.file_name()
                    .expect("a crate directory has a name")
                    .to_string_lossy()
                    .into_owned(),
            );
        }
    }
    declared.sort();
    let mut read: Vec<String> = PUBLISHED.iter().map(|s| (*s).to_string()).collect();
    read.sort();
    assert_eq!(
        read, declared,
        "this check's published-crate literal and the set the manifests declare disagree — a crate added or \
         retired on one side only"
    );
}
