//! Repository check: a token with a constant owner is not spelled again as a literal inside its reach.
//!
//! Two constants exist because one token had to have one owner — `kanhe::region::DO_NOT_EDIT`, the marker a
//! generated document declares itself with, and `shengmo::workspace::MARKER`, the variable saying a run must
//! find a repository. Both were then written out again as literals in every module that could reach them,
//! which is the shape `verdict_channel` closed between a shell script and Rust and which had stayed open
//! between Rust and Rust.
//!
//! **The corpus is what can reach the constant, and nothing wider.** `MARKER` is spelled out in `tianheng`,
//! `louke` and `xuanji` — published crates that cannot depend on `shengmo`, because `shengmo` depends on
//! `tianheng` and the edge would close a cycle — and `DO_NOT_EDIT` in `shengmo`'s own law projection header,
//! which cannot reach `kanhe`. Those are facts about the dependency graph rather than sites anyone declined
//! to fix, so they sit outside this check's subject rather than inside it as exceptions. No count is given:
//! nothing enumerates that set, so a figure here would be a census with no producer.

use std::path::{Path, PathBuf};

use kanhe::region::{DO_NOT_EDIT, Source};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/kanhe/src/region.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every tracked `.rs` file under `dirs`, as `(path, text)`.
fn tracked(root: &Path, dirs: &[&str]) -> Vec<(String, String)> {
    let listing = std::process::Command::new("git")
        .args(["ls-files"])
        .args(dirs)
        .current_dir(root)
        .output()
        .expect("git must be runnable to enumerate the corpus a constant reaches");
    assert!(
        listing.status.success(),
        "`git ls-files` did not enumerate {dirs:?}: {}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let files: Vec<(String, String)> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter(|path| path.ends_with(".rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path))
                .unwrap_or_else(|err| panic!("cannot read the tracked file {path} ({err})"));
            (path.to_string(), text)
        })
        .collect();
    // The enumeration is an input like any other: a corpus that collapsed to nothing satisfies "no file
    // spells it" exactly as a clean one does.
    assert!(
        files.len() > 5,
        "read {} tracked Rust file(s) under {dirs:?}, which is not a corpus this property can be about",
        files.len()
    );
    files
}

/// A token with a constant owner is spelled once inside the reach of that constant.
///
/// **Executed text, not the whole file.** A doc comment naming the marker in prose is a reader being told
/// what the constant is, not a second owner of it — and this check's own module header names both tokens.
/// `repository-checks` requires a check deciding a property over executed text to take its corpus from
/// `kanhe::region`, and this is the direction that holds it rather than the requirement being satisfied by
/// the import.
#[test]
fn a_token_with_a_constant_owner_has_no_second_spelling_in_reach() {
    let Some(root) = workspace_root() else {
        return;
    };
    for (constant, value, owner, dirs) in [
        (
            "kanhe::region::DO_NOT_EDIT",
            DO_NOT_EDIT,
            "crates/kanhe/src/region.rs",
            &["crates/kanhe"][..],
        ),
        (
            "shengmo::workspace::MARKER",
            shengmo::workspace::MARKER,
            "crates/shengmo/src/workspace.rs",
            &["crates/kanhe", "crates/shengmo"][..],
        ),
    ] {
        let mut offences = Vec::new();
        for (path, text) in tracked(&root, dirs) {
            if path == owner {
                continue;
            }
            for (number, line) in Source::of(text).rust().numbered_lines() {
                if line.contains(value) {
                    offences.push(format!("  {path}:{number}"));
                }
            }
        }
        assert!(
            offences.is_empty(),
            "these sites spell `{value}` as a literal while `{constant}` owns it, so the two can disagree \
             about one token and a change to the constant would leave them behind:\n{}",
            offences.join("\n")
        );
    }
}
