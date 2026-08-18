//! The tracking reader's failure matrix: which git exits are an answer, and which are a refusal to answer.
//!
//! The question `ls-files --error-unmatch` is asked is *is this path tracked*, and git answers it with an
//! exit status. One status is the answer; the rest are git declining to read the repository at all. These
//! run real git against real directories because the distinction being tested **is** git's exit contract —
//! a fake would assert the reader against this file's belief about git rather than against git.

use crate::publish_source_gate::{Tracked, tracks};

/// A scratch directory of this process's own, removed and recreated so a previous run cannot answer for this
/// one.
fn scratch(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("kanhe-tracks-{}-{name}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    xingbiao::claim_scratch(&dir).expect("create the fixture directory");
    dir
}

fn git(dir: &std::path::Path, args: &[&str]) {
    let out = crate::hermetic_git::hermetic("git")
        .args(args)
        .current_dir(dir)
        .output()
        .expect("run git");
    assert!(out.status.success(), "git {args:?} failed in the fixture");
}

/// The control: in a repository git can read, an absent path is untracked and a present one is tracked.
///
/// Without it the refusal direction below would hold for a reader that answered `Unreadable` to everything.
#[test]
fn a_repository_git_can_read_answers_both_ways() {
    let dir = scratch("answers");
    git(&dir, &["init", "-q", "."]);
    std::fs::write(dir.join("tracked.txt"), "hi").expect("write");
    git(&dir, &["add", "tracked.txt"]);

    assert!(
        matches!(tracks(&dir, "tracked.txt"), Tracked::Yes),
        "a file added to the index was not read as tracked"
    );
    assert!(
        matches!(tracks(&dir, "absent.txt"), Tracked::No),
        "a path the index does not carry was not read as untracked — this is the exit status the question \
         expects, and folding it in with the rest would make every answer unreadable"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A directory that is not a repository: git exits **128**, which is not the answer to the question.
///
/// Measured, on this machine's git: `1` for a path the index does not carry, `128` for a directory that is
/// not a repository and for an index that cannot be parsed. Reading every non-zero exit as *not tracked*
/// reported a fact it never read — and reported it as a **violation**, in front of `cargo publish`, where
/// this repository's contract reserves that class for a source that disagrees.
///
/// **Negative run:** against the reader that matched `Failure::Exit(_)` as a whole, this failed with
/// `Tracked::No`; the control above passed unchanged in the same run.
#[test]
fn a_directory_git_will_not_read_is_not_a_directory_that_tracks_nothing() {
    let dir = scratch("norepo");
    std::fs::write(dir.join("present.txt"), "hi").expect("write");

    let read = tracks(&dir, "present.txt");
    assert!(
        matches!(read, Tracked::Unreadable(_)),
        "git declined to read a directory that is no repository, and the reader turned that into an answer \
         about the path: {read:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
