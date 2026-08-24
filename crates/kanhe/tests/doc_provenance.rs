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

/// Every workspace member cargo will publish, derived rather than listed.
///
/// **The literal that stood here was a third owner of one fact.** It named six crates while
/// `kanhe::manifest::publishable` read the manifests and cargo read them too — and nothing held any pair of
/// the three equal, so the literal was the only thing connecting the text reader to cargo's answer. It is
/// gone: this derives the set from `publishable`, and
/// [`the_text_reader_agrees_with_cargo_about_every_member`] holds that derivation against cargo itself.
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
                "{}: whether this crate publishes cannot be decided from its manifest ({unreadable:?}), so \
                 the corpus this sweep reads would be a guess",
                manifest.display()
            ),
        }
    }
    published.sort();
    // The enumeration is an input like any other: a corpus derived down to nothing satisfies "no doc comment
    // names a round" exactly as a clean one does.
    assert!(
        published.len() > 1,
        "{} publishable crates were derived from crates/*/Cargo.toml — this family has several, and a sweep \
         over one is not the subject this check claims",
        published.len()
    );
    published
}

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
    let published = published_crates(root);
    let dirs: Vec<String> = published
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
    for krate in &published {
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

/// The text reader and cargo agree about every member.
///
/// **Two deliberate readers of one fact needed a reaction between them, and had none.** `publishable` reads
/// manifest text; the two workflow jobs ask cargo. What connected them was a hand-kept literal in this file —
/// a third owner — and `publishable`'s own matrix, which is `f(literal) == expected` over strings no manifest
/// in the tree contains: it encodes a belief about cargo rather than cargo's answer. The belief was wrong by
/// one whole spelling, `publish = [ ]`, which cargo refuses and that reader called publishable.
///
/// This is the both-ways check. `Unreadable` is admitted only where the manifest genuinely defers to the
/// workspace, which is the one case text cannot decide and cargo can — and the tree has none today, so the
/// arm says so rather than being silently permissive.
#[test]
fn the_text_reader_agrees_with_cargo_about_every_member() {
    let Some(root) = workspace_root() else {
        return;
    };
    let metadata = kanhe::release_coherence_gate::cargo_metadata(&root)
        .expect("cargo metadata reads this workspace");
    let packages = metadata["packages"]
        .as_array()
        .expect("cargo reports its packages as an array");
    assert!(
        packages.len() > 1,
        "cargo reported {} packages — a comparison over one is not this workspace",
        packages.len()
    );

    let mut disagreements = Vec::new();
    let mut compared = 0usize;
    for package in packages {
        let name = package["name"].as_str().expect("a package has a name");
        let manifest_path = package["manifest_path"]
            .as_str()
            .expect("a package has a manifest path");
        let text = std::fs::read_to_string(manifest_path).expect("a member manifest reads");
        // Cargo reports `[]` for `publish = false` and for every empty-array spelling of it, `null` for
        // absent or `true`, and the registry list otherwise.
        let cargo_says_no = package["publish"]
            .as_array()
            .is_some_and(|registries| registries.is_empty());
        compared += 1;
        match kanhe::manifest::publishable(&text) {
            kanhe::manifest::Publishable::No if cargo_says_no => {}
            kanhe::manifest::Publishable::Yes if !cargo_says_no => {}
            other => disagreements.push(format!(
                "  {name}: the text reader answers {other:?} while cargo reports \
                 publish={} — one fact, two verdicts, in front of `cargo publish`",
                package["publish"]
            )),
        }
    }
    assert_eq!(
        compared,
        packages.len(),
        "every package cargo reported was compared"
    );
    assert!(
        disagreements.is_empty(),
        "the manifest text reader and cargo disagree:\n{}",
        disagreements.join("\n")
    );
}
