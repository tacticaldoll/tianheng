//! Repository check: the source a `cargo publish` runs from.
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

use kanhe::refusal;

use kanhe::publish_source_gate as gate;

use gate::{
    NoClassification, build_fixture, hermetic, hidden_by_the_checkout, hidden_by_the_checkout_with,
    judge,
};
use refusal::Kind;
use std::path::{Path, PathBuf};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
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
        refusal
            .message
            .contains("does not appear to be a git repository"),
        "the refusal must preserve the failed live read's cause: {}",
        refusal.message
    );
    assert!(
        refusal.message.contains(&absent.display().to_string()),
        "the refusal must name the remote whose read failed: {}",
        refusal.message
    );
}

/// A successful live read with no `main` is distinct from a live read that could not run.
#[test]
fn a_remote_without_main_is_named_as_missing_the_ref() {
    let root = scratch("remote-without-main");
    let fixture = build_fixture(&root, "remote-without-main", "9.9.9");
    let remote_without_main = root.join("remote-without-main.git");
    git(
        &root,
        &[
            "init",
            "--bare",
            "-q",
            remote_without_main.to_str().expect("fixture path is UTF-8"),
        ],
    );

    let verdict = judge(
        &fixture.repo,
        remote_without_main.to_str().expect("fixture path is UTF-8"),
    );
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a remote without main must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("has no refs/heads/main"),
        "a successful empty read must name the absent ref, not a command failure: {}",
        refusal.message
    );
}

// --- the inputs this judgement cannot read ---------------------------------------------------------------

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

/// The tag ref resolves and the tag object is gone: it cannot be read, which is not "it is lightweight".
#[test]
fn a_tag_whose_object_is_missing_cannot_be_read() {
    let root = scratch("missing-tag");
    let fixture = build_fixture(&root, "missing-tag", "9.9.9");
    drop_object(&fixture.repo, &rev(&fixture.repo, "refs/tags/v9.9.9"));
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a missing tag object must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read the tag object"),
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

/// The tag object reads and the commit it names does not: the tag cannot be resolved to a commit.
///
/// HEAD is an **orphan** commit carrying the same release subject, so reading its subject traverses no
/// parents — measured, not assumed: a HEAD with the missing commit as an ancestor refuses one step earlier,
/// which is the direction above. Everything before this point reads objects that are still there; only
/// peeling the tag needs the one that is gone.
#[test]
fn a_tag_whose_commit_is_missing_cannot_be_resolved() {
    let root = scratch("missing-tag-commit");
    let fixture = build_fixture(&root, "missing-tag-commit", "9.9.9");
    let tagged = rev(&fixture.repo, "refs/tags/v9.9.9^{commit}");
    git(&fixture.repo, &["checkout", "-q", "--orphan", "detached"]);
    git(&fixture.repo, &["rm", "-rq", "--cached", "."]);
    std::fs::write(fixture.repo.join("only.txt"), "orphan").expect("write");
    for stray in ["Cargo.toml", "CHANGELOG.md"] {
        let _ = std::fs::remove_file(fixture.repo.join(stray));
    }
    std::fs::write(
        fixture.repo.join("Cargo.toml"),
        "[workspace.package]\nversion = \"9.9.9\"\n",
    )
    .expect("write");
    git(&fixture.repo, &["add", "-A"]);
    git(&fixture.repo, &["commit", "-qm", "release: 9.9.9"]);
    drop_object(&fixture.repo, &tagged);
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a tag naming a missing commit must be refused");
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not resolve"),
        "{}",
        refusal.message
    );
}

// --- what `clean` means ------------------------------------------------------------------------------------
//
// A file ignored by **tracked** repository content is clean: `cargo publish` applies the same exclusion and
// would not package it either. A file hidden by this clone or this machine is not — the same commit would
// otherwise be judged differently in different places, which is the one thing a governance gate must not do.
//
// The classifier is exercised directly, because these three cases are about which source hid a path rather
// than about the whole judgement; one direction below carries the same shape through `judge` to show the
// wiring.

/// A repository whose only commit tracks whatever `tracked` names, with `stray` present and untracked.
fn hiding(name: &str, tracked: &[(&str, &str)], stray: &str) -> (PathBuf, PathBuf) {
    let root = scratch(name);
    let repo = root.join("repo");
    std::fs::create_dir_all(&repo).expect("create");
    git(&repo, &["init", "-q", "-b", "main"]);
    // The fixture's git is hermetic, so it inherits no identity — which is the point of hermetic fixtures and
    // the reason this is set here rather than assumed from the machine.
    git(&repo, &["config", "user.name", "T"]);
    git(&repo, &["config", "user.email", "t@example.invalid"]);
    git(&repo, &["config", "commit.gpgsign", "false"]);
    for (path, contents) in tracked {
        std::fs::write(repo.join(path), contents).expect("write");
        git(&repo, &["add", "-f", "--", path]);
    }
    if !tracked.is_empty() {
        git(&repo, &["commit", "-qm", "release: 9.9.9"]);
    }
    std::fs::write(repo.join(stray), "stray").expect("write");
    (root, repo)
}

#[test]
fn a_file_ignored_by_tracked_repository_content_is_clean() {
    let (root, repo) = hiding(
        "ignored-tracked",
        &[(".gitignore", "stray.txt\n")],
        "stray.txt",
    );
    let hidden = hidden_by_the_checkout(&repo).expect("the classifier reads this repository");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        hidden.is_empty(),
        "a file ignored by a tracked `.gitignore` was reported as hidden by the checkout, which would block a \
         release the repository itself excludes: {hidden:?}"
    );
}

/// A classifier that could not run is not one that found nothing.
///
/// Constructed rather than declared: the repository must still answer `ls-files` and `status` for the
/// judgement to reach the classification at all, so the failure is supplied rather than arranged.
#[test]
fn an_exclusion_classifier_that_cannot_run_cannot_be_judged() {
    let (root, repo) = hiding(
        "classifier-failed",
        &[(".gitignore", "stray.txt\n")],
        "stray.txt",
    );
    let refusal = hidden_by_the_checkout_with(&repo, |_, _| {
        Err(NoClassification::Failed(
            "check-ignore exploded".to_string(),
        ))
    })
    .expect_err("a classifier that could not run must refuse rather than answer");
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge);
    assert!(
        refusal.message.contains("could not classify"),
        "{}",
        refusal.message
    );
}

/// The same judgement with a classifier that ran and matched nothing: the source is unshown, which is the
/// checkout's, and that is an answer rather than a refusal.
#[test]
fn an_exclusion_classifier_that_matched_nothing_still_answers() {
    let (root, repo) = hiding(
        "classifier-empty",
        &[(".gitignore", "stray.txt\n")],
        "stray.txt",
    );
    let hidden = hidden_by_the_checkout_with(&repo, |_, _| Err(NoClassification::MatchedNothing))
        .expect("matching nothing is an answer, not a failure to read");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        hidden.iter().any(|line| line.contains("<unshown>")),
        "an unshown source is the checkout's: {hidden:?}"
    );
}

/// The same rule, for a name git prints **quoted**.
///
/// `ls-files --others` prints a path with non-ASCII bytes as `"ignored-\346\231\256\351\200\232"`, and
/// asking `check-ignore` about that literal asks about a file that does not exist. Measured before the
/// repair: the source went unshown and the gate refused a file the repository itself ignores.
#[test]
fn a_file_with_quoted_bytes_ignored_by_tracked_content_is_clean() {
    let (root, repo) = hiding(
        "ignored-quoted",
        &[(".gitignore", "ignored-*\n")],
        "ignored-\u{666e}\u{901a}",
    );
    let hidden = hidden_by_the_checkout(&repo).expect("the classifier reads this repository");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        hidden.is_empty(),
        "a file whose name git prints quoted, ignored by a tracked `.gitignore`, was reported as hidden by \
         the checkout — the classifier was asked about the quoted spelling, which names no file: {hidden:?}"
    );
}

#[test]
fn a_file_hidden_by_an_untracked_gitignore_is_not_clean() {
    let (root, repo) = hiding("ignored-untracked", &[("kept.txt", "k\n")], "stray.txt");
    std::fs::write(repo.join(".gitignore"), "stray.txt\n.gitignore\n").expect("write");
    let hidden = hidden_by_the_checkout(&repo).expect("the classifier reads this repository");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        hidden.iter().any(|line| line.contains("stray.txt")),
        "a file hidden by an *untracked* `.gitignore` was accepted; the source is named like repository \
         content and is no more part of it than the clone's own exclude file: {hidden:?}"
    );
}

#[test]
fn a_file_hidden_by_this_clones_exclude_file_is_not_clean() {
    let (root, repo) = hiding("ignored-clone", &[("kept.txt", "k\n")], "stray.txt");
    std::fs::write(repo.join(".git/info/exclude"), "stray.txt\n").expect("write");
    let hidden = hidden_by_the_checkout(&repo).expect("the classifier reads this repository");
    let _ = std::fs::remove_dir_all(&root);
    assert!(
        hidden.iter().any(|line| line.contains("info/exclude")),
        "a file hidden by this clone's exclude file was accepted, so the same commit would be judged \
         differently elsewhere: {hidden:?}"
    );
}

/// The same shape through the whole judgement, so the classifier is known to be wired to a verdict.
#[test]
fn a_worktree_the_checkout_hides_is_a_violation() {
    let root = scratch("checkout-hidden");
    let fixture = build_fixture(&root, "checkout-hidden", "9.9.9");
    std::fs::write(fixture.repo.join(".git/info/exclude"), "stray.txt\n").expect("write");
    std::fs::write(fixture.repo.join("stray.txt"), "stray").expect("write");
    let verdict = judge(&fixture.repo, &fixture.remote.display().to_string());
    let _ = std::fs::remove_dir_all(&root);
    let refusal = verdict.expect_err("a file only this checkout hides must be refused");
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("only this checkout hides"),
        "{}",
        refusal.message
    );
}
