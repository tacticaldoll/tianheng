//! Repository check: every in-repo fixture and example manifest declares its own workspace root.
//!
//! **This holds a single line of defence, not a second one.** The root `Cargo.toml`'s `exclude` key is a real
//! second line for `examples/`, and cargo offers none at all for the fixtures: measured, a package nested
//! inside a workspace member's own directory cannot be excluded, whichever form the entry takes — exact path,
//! directory prefix, or glob. Entries naming `crates/*/tests/fixtures` were therefore inert while reading as
//! protection, and were dropped in favour of this check.
//!
//! What a missing `[workspace]` costs: the fixture is inferred into this workspace, so its deliberate faults —
//! the violations these fixtures exist to carry — become this workspace's own, and `cargo build --workspace`
//! starts compiling code written to be wrong. For a fixture under a member, nothing else notices.

use std::path::PathBuf;

use kanhe::refusal::{Kind, Refusal, cannot_judge, violation};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every tracked manifest that is a fixture's or an example's own, as `git` records them.
///
/// Tracked rather than walked, so a manifest present only in a working tree cannot make this pass, and one
/// deleted from the tree but still on disk cannot make it fail.
fn subject_manifests(root: &std::path::Path) -> Result<Vec<String>, Refusal> {
    // Through the shared builder like the fixture writes below. `ls-files` without `--others` consults no
    // ignore file, so this read is not in that channel — but one file spawning git two ways is the twin this
    // crate keeps converging, and the builder's failure type already separates *git could not run* from
    // *git ran and refused*, which the hand-rolled form folded into one sentence.
    let out = kanhe::hermetic_git::run(
        root,
        &[],
        &[
            "ls-files",
            "--",
            "crates/*/tests/fixtures/*/Cargo.toml",
            "examples/*/Cargo.toml",
        ],
    );
    let listing = match out {
        Ok(listing) => listing,
        Err(err) => {
            return Err(cannot_judge(format!(
                "cannot enumerate the tracked manifests: {err}"
            )));
        }
    };
    Ok(listing
        .lines()
        .filter(|line| !line.is_empty())
        .map(str::to_string)
        .collect())
}

/// Whether a manifest declares itself a workspace root.
///
/// A line of its own, because `[workspace]` inside a string or after a `#` is not a table. The tables cargo
/// admits here are the bare one and its sub-tables, so `[workspace.dependencies]` counts as declaring the
/// root too.
fn declares_a_workspace(manifest: &str) -> bool {
    manifest.lines().map(str::trim).any(|line| {
        line == "[workspace]" || (line.starts_with("[workspace.") && line.ends_with(']'))
    })
}

fn judge(root: &std::path::Path) -> Result<usize, Refusal> {
    let manifests = subject_manifests(root)?;
    if manifests.is_empty() {
        return Err(cannot_judge(
            "no tracked fixture or example manifest was found, so this check would hold over nothing; the \
             layout may have moved",
        ));
    }
    let mut offences = Vec::new();
    for relative in &manifests {
        let text = std::fs::read_to_string(root.join(relative)).map_err(|err| {
            // An unread manifest is not a manifest that declares a workspace.
            cannot_judge(format!("could not read {relative}: {err}"))
        })?;
        if !declares_a_workspace(&text) {
            offences.push(format!(
                "  {relative} declares no `[workspace]`, so it is inferred into this workspace"
            ));
        }
    }
    if offences.is_empty() {
        Ok(manifests.len())
    } else {
        Err(violation(format!(
            "a fixture or example manifest does not declare its own workspace root, and for one nested under \
             a member `exclude` cannot substitute — cargo does not exclude a package inside a workspace \
             member's directory:\n{}",
            offences.join("\n")
        )))
    }
}

#[test]
fn every_fixture_and_example_declares_its_own_workspace_root() {
    let Some(root) = workspace_root() else {
        return;
    };
    match judge(&root) {
        Ok(count) => {
            eprintln!("{count} fixture and example manifests declare their own workspace root")
        }
        Err(refusal) => panic!(
            "workspace isolation ({:?}): {}",
            refusal.kind, refusal.message
        ),
    }
}

#[test]
fn a_manifest_without_the_table_is_a_violation() {
    assert!(declares_a_workspace(
        "[package]\nname = \"x\"\n\n[workspace]\n"
    ));
    assert!(declares_a_workspace(
        "[package]\nname = \"x\"\n\n[workspace.dependencies]\nserde = \"1\"\n"
    ));
    assert!(
        !declares_a_workspace("[package]\nname = \"x\"\n"),
        "a manifest with no table must not read as declaring one"
    );
    assert!(
        !declares_a_workspace("[package]\nname = \"x\"\n# [workspace]\n"),
        "a commented-out table is not a table"
    );
    assert!(
        !declares_a_workspace("[package]\ndescription = \"see [workspace] in the root\"\n"),
        "the marker inside a string is not a table"
    );
}

#[test]
fn the_refusal_classes_are_distinct() {
    let root = std::env::temp_dir().join(format!("kanhe-isolation-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create");
    // Not a git repository, so the enumeration cannot answer — which is not the same fact as a manifest
    // that disagrees.
    let refusal = judge(&root).expect_err("an unenumerable tree cannot be judged");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
}

/// A repository this enumeration can read, carrying none of the manifests it judges.
///
/// The vacuity guard. Its sibling above covers an enumeration that could not be *made*; this covers one that
/// was made and returned nothing, which is a different fact and the one that would let this check pass over
/// a layout that moved. Both are cannot-judge, and neither is a manifest that disagrees.
///
/// Negative run: with the guard replaced by `Ok(0)`, this returned a clean count of zero — the check
/// reporting agreement over a set it never had.
#[test]
fn a_repository_carrying_none_of_the_judged_manifests_holds_over_nothing() {
    let root = std::env::temp_dir().join(format!("kanhe-isolation-empty-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create");
    // Through the shared builder, which closes the ambient ignore channel. A bare `Command` here left
    // `git add` reading whatever `core.excludesFile` this machine has, so a fixture could be built without
    // the file it names — and the file it names IS the subject.
    let git = |args: &[&str]| kanhe::hermetic_git::fixture(&root, "git", args);
    git(&["init", "-q", "."]);
    std::fs::write(
        root.join("README.md"),
        "a repository with no judged manifest\n",
    )
    .expect("write");
    git(&["add", "README.md"]);

    let refusal = judge(&root).expect_err("a repository carrying none of them holds over nothing");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("would hold over nothing"),
        "the refusal must name the emptiness rather than a manifest: {}",
        refusal.message
    );
}

/// A tracked manifest this check cannot read is not a manifest that declares no workspace.
///
/// Negative run: with the read's failure mapped to an empty string, the manifest declared no `[workspace]`
/// table and was reported as a **violation** — a file that could not be read, shown to an operator as one
/// they had written wrongly.
#[test]
fn a_tracked_manifest_that_cannot_be_read_is_not_one_that_disagrees() {
    let root = std::env::temp_dir().join(format!("kanhe-isolation-unread-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create");
    // Through the shared builder, which closes the ambient ignore channel. A bare `Command` here left
    // `git add` reading whatever `core.excludesFile` this machine has, so a fixture could be built without
    // the file it names — and the file it names IS the subject.
    let git = |args: &[&str]| kanhe::hermetic_git::fixture(&root, "git", args);
    git(&["init", "-q", "."]);
    std::fs::create_dir_all(root.join("examples/adopter")).expect("create");
    std::fs::write(root.join("examples/adopter/Cargo.toml"), [0xff, 0xfe, 0xfd]).expect("write");
    git(&["add", "examples/adopter/Cargo.toml"]);

    let refusal = judge(&root).expect_err("a manifest this check cannot read is not one it read");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read"),
        "the refusal must name the read failure: {}",
        refusal.message
    );
}
