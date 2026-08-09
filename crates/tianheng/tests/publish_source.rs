//! Self-governance reaction: the source a `cargo publish` runs from.
//!
//! It stands before an **irreversible act**. `cargo publish` records the commit it ran on in every tarball's
//! `.cargo_vcs_info.json`, and a version can never be re-uploaded — the `0.4.0` family recorded a release
//! branch's tip rather than the commit its tag names, permanently, which is why this exists.
//!
//! **The gate itself does not run in development.** No development checkout is a release snapshot, so a
//! pre-flight run could only ever refuse; `scripts/publish.sh` sets `TIANHENG_PUBLISH_SOURCE=1` immediately
//! before publishing and this judges the repository then. What runs in the ordinary suite is the failure
//! matrix below, which builds repositories in known-wrong shapes and holds the judgement to refusing each —
//! and to refusing them as a **violation** rather than as a cannot-judge, because an operator standing before
//! an irreversible act must be able to tell "the source disagrees" from "the source could not be read".

#[path = "support/refusal.rs"]
mod refusal;

#[path = "support/publish_source_gate.rs"]
mod gate;

use gate::{build_fixture, hermetic, judge};
use refusal::Kind;
use std::path::{Path, PathBuf};

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("Cargo.toml").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "Cargo.toml expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — a governance \
         reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

fn scratch(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "tianheng-publish-source-{name}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).expect("the fixture root is writable");
    root
}

fn git(repo: &Path, args: &[&str]) {
    let out = hermetic("git")
        .args(args)
        .current_dir(repo)
        .output()
        .unwrap_or_else(|err| panic!("cannot run git {args:?}: {err}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

/// The gate, over this repository, at publish time only.
#[test]
fn the_publish_source_is_the_signed_release_snapshot() {
    let Some(root) = workspace_root() else {
        return;
    };
    if std::env::var_os("TIANHENG_PUBLISH_SOURCE").is_none() {
        eprintln!(
            "publish source: not judged — no development checkout is a release snapshot, so a pre-flight run \
             could only ever refuse. `scripts/publish.sh` sets TIANHENG_PUBLISH_SOURCE=1 before publishing."
        );
        return;
    }
    match judge(&root, "origin") {
        Ok(report) => eprintln!("{report}"),
        Err(refusal) => panic!("publish source ({:?}): {}", refusal.kind, refusal.message),
    }
}

// --- the failure matrix ------------------------------------------------------------------------------------

/// The whole shape, accepted — so every refusal below is about the thing it names and not about the fixture.
#[test]
fn a_signed_tagged_snapshot_at_the_tip_of_main_is_accepted() {
    let root = scratch("accepted");
    let fixture = build_fixture(&root, "ok", "9.9.9");
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    assert!(verdict.is_ok(), "{:?}", verdict.err());
}

#[test]
fn a_dirty_worktree_is_a_violation() {
    let root = scratch("dirty");
    let fixture = build_fixture(&root, "dirty", "9.9.9");
    std::fs::write(fixture.repo.join("stray.txt"), "untracked").expect("write a stray file");
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a dirty worktree must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("worktree is not clean"),
        "{}",
        refusal.message
    );
}

#[test]
fn a_head_that_is_not_the_release_snapshot_is_a_violation() {
    let root = scratch("subject");
    let fixture = build_fixture(&root, "subject", "9.9.9");
    std::fs::write(fixture.repo.join("note.md"), "later work").expect("write");
    git(&fixture.repo, &["add", "."]);
    git(&fixture.repo, &["commit", "-qm", "docs: later work"]);
    git(&fixture.repo, &["push", "-q", "origin", "main"]);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a non-release HEAD must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("is not this version's release snapshot"),
        "{}",
        refusal.message
    );
}

#[test]
fn an_untagged_snapshot_is_a_violation() {
    let root = scratch("untagged");
    let fixture = build_fixture(&root, "untagged", "9.9.9");
    git(&fixture.repo, &["tag", "-d", "v9.9.9"]);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("an untagged snapshot must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("there is no tag"),
        "{}",
        refusal.message
    );
}

#[test]
fn a_lightweight_tag_is_a_violation() {
    let root = scratch("lightweight");
    let fixture = build_fixture(&root, "lightweight", "9.9.9");
    git(&fixture.repo, &["tag", "-d", "v9.9.9"]);
    git(&fixture.repo, &["tag", "v9.9.9"]);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a lightweight tag must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("lightweight tag"),
        "{}",
        refusal.message
    );
}

/// An annotated tag whose *message* quotes a signature block, carrying no signature of its own.
///
/// This is the direction the whole signature check exists for: a block quoted in a message is text.
#[test]
fn an_unsigned_annotated_tag_is_a_violation() {
    let root = scratch("unsigned");
    let fixture = build_fixture(&root, "unsigned", "9.9.9");
    git(&fixture.repo, &["tag", "-d", "v9.9.9"]);
    git(
        &fixture.repo,
        &[
            "-c",
            "commit.gpgsign=false",
            "tag",
            "-a",
            "v9.9.9",
            "-m",
            "v9.9.9\n-----BEGIN SSH SIGNATURE-----\nnot a signature\n-----END SSH SIGNATURE-----",
        ],
    );
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("an unsigned annotated tag must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("carries no signature")
            || refusal.message.contains("does not verify"),
        "{}",
        refusal.message
    );
}

#[test]
fn a_tag_pointing_elsewhere_than_head_is_a_violation() {
    let root = scratch("elsewhere");
    let fixture = build_fixture(&root, "elsewhere", "9.9.9");
    // Move HEAD forward, then move it back to a *different* commit with the same subject, so the tag no
    // longer names HEAD while every other property still holds.
    std::fs::write(fixture.repo.join("extra.md"), "x").expect("write");
    git(&fixture.repo, &["add", "."]);
    git(
        &fixture.repo,
        &["commit", "-q", "--amend", "-m", "release: 9.9.9"],
    );
    git(&fixture.repo, &["push", "-qf", "origin", "main"]);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a tag that does not name HEAD must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("publish the commit the tag names"),
        "{}",
        refusal.message
    );
}

/// The `0.4.0` shape, exactly: a correct snapshot that is not the tip of `main`.
///
/// Built by moving the remote's `main` past the snapshot from within the fixture rather than through a second
/// clone — the shape under test is "the remote moved on", and a clone adds a checkout whose default branch is
/// one more thing to get right.
#[test]
fn a_snapshot_that_is_not_the_tip_of_main_is_a_violation() {
    let root = scratch("not-tip");
    let fixture = build_fixture(&root, "not-tip", "9.9.9");
    git(&fixture.repo, &["checkout", "-q", "-b", "later"]);
    std::fs::write(fixture.repo.join("after.md"), "after").expect("write");
    git(&fixture.repo, &["add", "."]);
    git(&fixture.repo, &["commit", "-qm", "docs: after the release"]);
    git(&fixture.repo, &["push", "-q", "origin", "later:main"]);
    git(&fixture.repo, &["checkout", "-q", "main"]);
    git(&fixture.repo, &["branch", "-qD", "later"]);

    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a snapshot behind main must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("is not the tip of"),
        "{}",
        refusal.message
    );
}

/// The other half of the contract: an input it cannot read is **not** an incoherence.
#[test]
fn an_unreadable_source_cannot_be_judged_rather_than_refused() {
    let root = scratch("unreadable");
    let bare = root.join("no-manifest");
    std::fs::create_dir_all(&bare).expect("create");
    let no_manifest =
        judge(&bare, "origin").expect_err("a directory with no manifest is unjudgeable");
    assert_eq!(
        no_manifest.kind,
        Kind::CannotJudge,
        "{}",
        no_manifest.message
    );

    let fixture = build_fixture(&root, "malformed", "9.9.9");
    std::fs::write(
        fixture.repo.join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"not-a-version\"\n",
    )
    .expect("write a malformed manifest");
    let malformed = judge(&fixture.repo, &fixture.remote.display().to_string())
        .expect_err("a malformed version is unjudgeable");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(malformed.kind, Kind::CannotJudge, "{}", malformed.message);
    assert!(
        malformed.message.contains("malformed"),
        "{}",
        malformed.message
    );
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-publish-source-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}

// --- the cannot-judge directions, and the two the matrix was shadowing -------------------------------------

/// The refusal KIND was defended; the refusal's SUBJECT was not.
///
/// `an_unreadable_source_cannot_be_judged_rather_than_refused` asserted only `kind == CannotJudge` for a
/// directory with no manifest, so either of two branches alone satisfied it — a shadowing pair review found by
/// deleting each and watching nothing fail. Each direction below names the message it expects.
#[test]
fn each_unreadable_input_says_which_one_it_could_not_read() {
    let root = scratch("unreadable-each");

    let bare = root.join("no-manifest");
    std::fs::create_dir_all(&bare).expect("create");
    let refusal = judge(&bare, "origin").expect_err("a directory with no manifest is unjudgeable");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("has no Cargo.toml"),
        "{}",
        refusal.message
    );

    // A manifest but no repository: the next branch, which the one above was standing in for.
    let not_a_repo = root.join("not-a-repo");
    std::fs::create_dir_all(&not_a_repo).expect("create");
    std::fs::write(
        not_a_repo.join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"9.9.9\"\n",
    )
    .expect("write");
    let refusal =
        judge(&not_a_repo, "origin").expect_err("a directory that is no worktree is unjudgeable");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("is not a git worktree"),
        "{}",
        refusal.message
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// An annotated tag carrying no signature at all, distinguished from one whose signature does not verify.
///
/// `an_unsigned_annotated_tag_is_a_violation` asserted `contains("carries no signature") || contains("does
/// not verify")` — a disjunction that cannot say which branch fired, and which always fired on the second.
#[test]
fn a_tag_with_no_signature_block_is_named_as_such() {
    let root = scratch("no-signature");
    let fixture = build_fixture(&root, "no-signature", "9.9.9");
    git(&fixture.repo, &["tag", "-d", "v9.9.9"]);
    git(
        &fixture.repo,
        &[
            "tag",
            "-a",
            "v9.9.9",
            "-m",
            "v9.9.9 with no signature at all",
        ],
    );
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("an annotated tag carrying no signature must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("carries no signature"),
        "the refusal must name the ABSENT signature rather than a failed verification: {}",
        refusal.message
    );
}

/// A signature block this gate cannot read is undecidable, not a violation.
#[test]
fn a_signature_this_gate_cannot_read_cannot_be_judged() {
    let root = scratch("foreign-signature");
    let fixture = build_fixture(&root, "foreign-signature", "9.9.9");
    git(&fixture.repo, &["tag", "-d", "v9.9.9"]);
    git(
        &fixture.repo,
        &[
            "tag",
            "-a",
            "v9.9.9",
            "-m",
            "v9.9.9\n-----BEGIN PGP SIGNATURE-----\nnot ssh\n-----END PGP SIGNATURE-----",
        ],
    );
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a signature this gate cannot read must be refused");
    assert_eq!(
        refusal.kind,
        Kind::CannotJudge,
        "a block this gate cannot verify is undecidable, not a disagreement: {}",
        refusal.message
    );
    assert!(
        refusal
            .message
            .contains("carries a signature this gate cannot verify"),
        "the refusal must name THIS undecidable — a kind-only assertion in a judgement with fifteen \
         cannot-judge sites says nothing about which one fired, and repairing that in one test three tests \
         earlier did not stop it being written again here: {}",
        refusal.message
    );
}

/// A remote whose `main` cannot be read is undecidable — the direction that keeps a network failure from
/// reading as "the snapshot is behind".
#[test]
fn a_remote_that_cannot_be_read_cannot_be_judged() {
    let root = scratch("no-remote");
    let fixture = build_fixture(&root, "no-remote", "9.9.9");
    let absent = root.join("there-is-no-remote-here.git");
    let verdict = judge(&fixture.repo, &absent.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("an unreadable remote must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read refs/heads/main"),
        "{}",
        refusal.message
    );
}

// --- the inputs this judgement cannot read ---------------------------------------------------------------
//
// Found by running the refusal-site sweep rather than by reading: a `cannot_judge` no direction constructs is
// a refusal whose kind and message can both change with the suite green — and this gate stands in front of an
// irreversible act, where the kind is what an operator acts on.

/// An index git cannot parse: the worktree state cannot be read, which is not a dirty worktree.
///
/// Measured against real git before being relied on: a corrupt index fails `status` and `ls-files` while
/// `rev-parse --is-inside-work-tree` and `log` still answer, so the judgement reaches this point rather than
/// refusing earlier for a different reason.
#[test]
fn a_worktree_state_that_cannot_be_read_cannot_be_judged() {
    let root = scratch("unreadable-index");
    let fixture = build_fixture(&root, "unreadable-index", "9.9.9");
    std::fs::write(fixture.repo.join(".git/index"), b"not an index").expect("corrupt the index");
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("an unreadable worktree state must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("could not read the worktree state"),
        "{}",
        refusal.message
    );
}

/// A repository with no commit, whose worktree still reads clean because everything in it is ignored.
///
/// The clean worktree is what makes this reachable at all: without it the judgement refuses one step earlier,
/// for being dirty rather than for having no HEAD.
#[test]
fn a_repository_with_no_commit_cannot_have_its_head_read() {
    let root = scratch("no-commit");
    let repo = root.join("repo");
    std::fs::create_dir_all(&repo).expect("create");
    git(&repo, &["init", "-q", "-b", "main"]);
    std::fs::write(repo.join(".gitignore"), "*\n").expect("write");
    std::fs::write(
        repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"9.9.9\"\n",
    )
    .expect("write");
    let verdict = judge(&repo, "origin");
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a repository with no commit must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read HEAD"),
        "{}",
        refusal.message
    );
}

/// The object file a loose git object lives in, asserted to exist before it is removed.
///
/// A fixture that packed its objects would leave the file absent and the corruption unapplied — a perturbation
/// that never happened reads as a judgement that did.
fn drop_object(repo: &Path, sha: &str) {
    let (dir, rest) = sha.split_at(2);
    let object = repo.join(".git/objects").join(dir).join(rest);
    assert!(
        object.is_file(),
        "{object:?} is not a loose object, so removing it would not corrupt anything and the direction below \
         would be about an intact repository"
    );
    std::fs::remove_file(&object).expect("remove the object");
}

fn rev(repo: &Path, what: &str) -> String {
    let out = hermetic("git")
        .args(["rev-parse", what])
        .current_dir(repo)
        .output()
        .expect("run git rev-parse");
    assert!(out.status.success(), "git rev-parse {what} failed");
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// The tag ref resolves and the tag object is gone: whether it is annotated cannot be decided.
#[test]
fn a_tag_whose_object_is_missing_cannot_have_its_kind_read() {
    let root = scratch("missing-tag");
    let fixture = build_fixture(&root, "missing-tag", "9.9.9");
    drop_object(&fixture.repo, &rev(&fixture.repo, "refs/tags/v9.9.9"));
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a missing tag object must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("object type"),
        "{}",
        refusal.message
    );
}

/// A commit HEAD depends on is missing: its subject cannot be read, which is not a wrong subject.
///
/// Reading `HEAD` itself answers and `git status` answers, because both need only HEAD's own object. Reading
/// its **subject** traverses parents — measured, not assumed: removing HEAD's own object made `status` refuse
/// first, and removing an ancestor's is what leaves this the first thing that cannot be read.
#[test]
fn a_head_whose_ancestor_is_missing_cannot_have_its_subject_read() {
    let root = scratch("missing-ancestor");
    let fixture = build_fixture(&root, "missing-ancestor", "9.9.9");
    let tagged = rev(&fixture.repo, "refs/tags/v9.9.9^{commit}");
    std::fs::write(fixture.repo.join("later.txt"), "later").expect("write");
    git(&fixture.repo, &["add", "."]);
    git(&fixture.repo, &["commit", "-qm", "release: 9.9.9"]);
    drop_object(&fixture.repo, &tagged);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a missing ancestor object must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read HEAD's subject"),
        "{}",
        refusal.message
    );
}
