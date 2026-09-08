//! The tracking reader's failure matrix: which git exits are an answer, and which are a refusal to answer.
//!
//! The question `ls-files --error-unmatch` is asked is *is this path tracked*, and git answers it with an
//! exit status. One status is the answer; the rest are git declining to read the repository at all. These
//! run real git against real directories because the distinction being tested **is** git's exit contract —
//! a fake would assert the reader against this file's belief about git rather than against git.

use crate::publish_source_gate::{TagPresence, Tracked, tag_presence, tracks};

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

/// The control for the tag read: in a repository git can read, a present tag and an absent one differ.
///
/// Without it the refusal direction below would hold for a reader answering `Unreadable` to everything.
#[test]
fn a_repository_git_can_read_answers_both_ways_about_a_tag() {
    let dir = scratch("tag-answers");
    git(&dir, &["init", "-q", "."]);
    // The ref points at a blob rather than a commit, because a commit needs an identity and `hermetic`
    // closes the config that would carry one. A tag ref may name any object, and what this reads is whether
    // the ref RESOLVES — so the object's kind is not the subject and a blob keeps the fixture to one step.
    std::fs::write(dir.join("o.txt"), "x").expect("write");
    let sha = crate::hermetic_git::read(
        &dir,
        "a blob to tag",
        "git",
        &["hash-object", "-w", "o.txt"],
    );
    git(&dir, &["update-ref", "refs/tags/v9.9.9", sha.trim()]);

    assert!(
        matches!(tag_presence(&dir, "v9.9.9"), TagPresence::Present),
        "a tag this fixture just created was not read as present"
    );
    assert!(
        matches!(tag_presence(&dir, "v0.0.0"), TagPresence::Absent),
        "a tag the repository does not carry was not read as absent — this is the exit status the question \
         expects, and folding it in with the rest would make every answer unreadable"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A directory that is not a repository: git exits **128**, which is not the answer to the question.
///
/// This site read `.is_err()` and reported every failure as *there is no tag*, as a **violation**, in front
/// of `cargo publish` — where this repository's contract reserves that class for a source that disagrees.
/// `publish-source-integrity` states the rule over the class of every status-answering git read, and records
/// that it was generalized because it arrived through a second door; this was the third.
///
/// **Negative run:** against the reader that took `.is_err()` as *absent*, this failed reporting the tag
/// missing, and the control above passed unchanged in the same run. `--quiet` is what makes the split exist
/// at all: without it an absent ref and an unreadable directory both exit `128`.
#[test]
fn a_directory_git_will_not_read_is_not_a_repository_with_no_tag() {
    let dir = scratch("tag-norepo");

    let read = tag_presence(&dir, "v9.9.9");
    assert!(
        matches!(read, TagPresence::Unreadable(_)),
        "git declined to read a directory that is no repository, and the reader turned that into an answer \
         about the tag: {read:?}"
    );
    std::fs::remove_dir_all(&dir).ok();
}

/// A verifier that could not run is a cannot-judge; one that ran and rejected is the violation.
///
/// **The two used to be one `bool`.** `check_novalidate` answered `false` for a failed spawn, a failed
/// write, a failed reap, *and* a verifier that ran and rejected the payload — and the caller turned every
/// `false` into `signature-does-not-verify`, a **violation**. `publish-source-integrity` states the rule
/// the other way in so many words: *a signature this gate cannot read SHALL be a cannot-judge, never a
/// violation*, and the three refusals guarding armour, suffix and writability already answer that way. So a machine out of processes
/// would have been told its release tag's signature was bad, one line before an irreversible upload, and
/// `scripts/publish.sh` would have exited `1` — *a gate ran and refused* — where the fact was `2`.
///
/// **Asserted as the class, not as "it refused".** A direction checking only that verification failed
/// passes under both implementations, which is how the collapse survived review: non-zero cannot see `1`
/// from `2`. This reads the refusal's `kind`, and the control below reads the opposite arm — a verifier
/// that *did* run over a signature it rejects, which must stay `Ok(false)` so the caller's violation is
/// still reachable.
#[test]
fn a_verifier_that_could_not_run_is_not_a_bad_signature() {
    let root = std::env::temp_dir().join(format!(
        "kanhe-verifier-class-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir(&root).expect("the fixture root is claimable");
    let sig = root.join("tag.sig");
    std::fs::write(
        &sig,
        "-----BEGIN SSH SIGNATURE-----\nnot a real signature\n",
    )
    .expect("write the fixture signature");

    let unrunnable = crate::publish_source_gate::verify_with(
        "kanhe-no-such-verifier-98f2abc",
        "a payload",
        &sig,
    )
    .expect_err("a verifier that cannot be started reached no verdict");
    crate::refusal::expect(
        "publish-source-integrity#signature-verifier-reached-no-verdict",
        &unrunnable,
    );
    assert_eq!(
        unrunnable.kind,
        crate::refusal::Kind::CannotJudge,
        "a verifier that could not run is unjudgeable, never a disagreement: {}",
        unrunnable.message
    );

    // The control, and the reason this direction is not satisfied by a function that refuses everything:
    // the real verifier runs here and rejects the fixture, which must stay `Ok(false)` — that is the arm
    // the caller turns into `signature-does-not-verify`, and closing the class must not close it too.
    let ran = crate::publish_source_gate::verify_with("ssh-keygen", "a payload", &sig);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        ran.as_ref().ok().copied(),
        Some(false),
        "a verifier that ran and rejected the payload is a completed verification: {ran:?}"
    );
}
