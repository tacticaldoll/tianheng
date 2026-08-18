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
    NoClassification, Tracked, build_fixture, hermetic, hidden_by_the_checkout,
    hidden_by_the_checkout_with, judge,
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
    xingbiao::claim_scratch(&root).expect("the fixture root is writable");
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
        Err(refusal) => {
            // The class travels on its own channel, written before this fails, so a wrapper reads a verdict
            // rather than searching this message for one. See `kanhe::verdict_channel`.
            kanhe::verdict_channel::report(refusal.kind);
            panic!("publish source ({:?}): {}", refusal.kind, refusal.message)
        }
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
    refusal::expect("publish-source-integrity#worktree-is-not-clean", &refusal);
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
    refusal::expect(
        "publish-source-integrity#head-is-not-the-release-snapshot",
        &refusal,
    );
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
    refusal::expect("publish-source-integrity#release-tag-absent", &refusal);
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
    refusal::expect(
        "publish-source-integrity#release-tag-is-lightweight",
        &refusal,
    );
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
    refusal::expect(
        "publish-source-integrity#signature-does-not-verify",
        &refusal,
    );
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
    refusal::expect(
        "publish-source-integrity#release-tag-does-not-name-head",
        &refusal,
    );
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
    refusal::expect(
        "publish-source-integrity#head-is-not-the-tip-of-main",
        &refusal,
    );
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
    refusal::expect(
        "publish-source-integrity#workspace-version-malformed",
        &malformed,
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(malformed.kind, Kind::CannotJudge, "{}", malformed.message);
    assert!(
        malformed.message.contains("malformed"),
        "{}",
        malformed.message
    );
}

/// A manifest that cannot be READ (not merely absent or malformed once read) must not be judged as
/// though its version were missing — a real `io::Error` and a genuinely absent version are different
/// facts right before an irreversible `cargo publish`, and folding one into the other hides which one
/// happened.
#[test]
fn a_manifest_this_gate_cannot_read_is_not_judged_as_though_its_version_is_missing() {
    let root = scratch("unreadable-manifest");
    let fixture = build_fixture(&root, "unreadable-manifest", "9.9.9");
    std::fs::write(fixture.repo.join("Cargo.toml"), [0xff, 0xfe, 0xfd])
        .expect("write invalid utf-8 in place of the manifest");
    let refusal = judge(&fixture.repo, &fixture.remote.display().to_string())
        .expect_err("a manifest this gate cannot read must not be judged as though absent");
    refusal::expect(
        "publish-source-integrity#workspace-manifest-unreadable",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("could not read") && refusal.message.contains("Cargo.toml"),
        "the refusal must name the read failure rather than fold it into the generic missing message: {}",
        refusal.message
    );
    assert!(
        !refusal.message.contains("<missing>"),
        "a real read failure must not be reported identically to a genuinely absent version: {}",
        refusal.message
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
    refusal::expect(
        "publish-source-integrity#repository-root-has-no-manifest",
        &refusal,
    );
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

/// A version value this reader cannot read stops the publish, and says so in its own words.
///
/// Legal TOML in a form this reader does not take — a single-quoted literal. Before the reader answered in
/// three states it reported this as *missing or malformed*, which in front of `cargo publish` sends an
/// operator to look for a version key that is sitting in the manifest, correctly spelled for cargo.
///
/// The message is this gate's own rather than the release gate's. Both read the same manifest fact and each
/// cannot decide a different thing about it, so a shared sentence would tell an operator which fact was
/// unreadable and not which judgement it blocked.
#[test]
fn a_version_this_reader_cannot_read_stops_the_publish_as_a_cannot_judge() {
    let root = scratch("unreadable-version");
    let fixture = build_fixture(&root, "unreadable", "9.9.9");
    // A real repository, so the worktree branch above this one is already satisfied and the refusal is about
    // the version. Rewriting the manifest also leaves the tree dirty, which is refused *after* the version is
    // read — so the order of `judge`'s phases is what keeps this direction about the thing it names.
    std::fs::write(
        fixture.repo.join("Cargo.toml"),
        "[workspace]\nmembers = []\n\n[workspace.package]\nversion = '9.9.9'\n",
    )
    .expect("write");
    let refusal = judge(&fixture.repo, &fixture.remote.display().to_string())
        .expect_err("a version this reader cannot read must stop the publish");
    refusal::expect(
        "publish-source-integrity#workspace-version-unreadable",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal
            .message
            .contains("declares a workspace version this check cannot read"),
        "{}",
        refusal.message
    );
    assert!(
        refusal
            .message
            .contains("which tag this tree would have to be the release snapshot of"),
        "the refusal must name what THIS gate could not decide: {}",
        refusal.message
    );
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
    refusal::expect(
        "publish-source-integrity#release-tag-carries-no-signature",
        &refusal,
    );
    refusal::expect(
        "publish-source-integrity#release-tag-carries-no-signature",
        &refusal,
    );
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
    refusal::expect(
        "publish-source-integrity#signature-armour-unverifiable",
        &refusal,
    );
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
    refusal::expect("publish-source-integrity#remote-main-unreadable", &refusal);
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
    refusal::expect("publish-source-integrity#remote-has-no-main", &refusal);
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
    refusal::expect(
        "publish-source-integrity#worktree-state-unreadable",
        &refusal,
    );
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
    refusal::expect("publish-source-integrity#tag-object-unreadable", &refusal);
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
    refusal::expect("publish-source-integrity#head-subject-unreadable", &refusal);
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
    refusal::expect("publish-source-integrity#tag-commit-unresolvable", &refusal);
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
// than about the whole judgement; a sibling direction carries the same shape through `judge` to show the
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
    let refusal = hidden_by_the_checkout_with(
        &repo,
        |_, _| {
            Err(NoClassification::Failed(
                "check-ignore exploded".to_string(),
            ))
        },
        |_, _| Tracked::Yes,
    )
    .expect_err("a classifier that could not run must refuse rather than answer");
    refusal::expect(
        "publish-source-integrity#exclusion-classifier-cannot-run",
        &refusal,
    );
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
    let hidden = hidden_by_the_checkout_with(
        &repo,
        |_, _| Err(NoClassification::MatchedNothing),
        |_, _| Tracked::Yes,
    )
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

// --- the size of the conversation, which every fixture above leaves unexercised -----------------------------
//
// Each fixture above hides a handful of files, so "the excluded set is small" held in all of them and was
// never a claim anything made. Measured on this repository when these were written, the set was 73,670 paths
// — `/target/` alone — and the two directions naming that pattern are the only place that number's consequences are
// constructed.
//
// **Every figure either sizes a fixture or is read back from one.** The first draft typed the fixture size
// into the assertion message beside the literal that built it, twice, which is the two-lists shape this
// repository keeps closing: change the fixture and the message reports a count nothing had.

/// Enough ignored files that the classification cannot fit in the kernel's pipe buffers.
///
/// At ~190 bytes of name each that is roughly 380 KB in and 400 KB out against 64 KB each way. The old shape
/// blocks after about the first 117 KB — one pipe's worth buffered plus one pipe's worth consumed — so this
/// clears the threshold by more than three times rather than sitting near it.
const OUTGROWS_A_PIPE: usize = 2_000;

/// Enough excluded paths that asking per path rather than per source is visible in a count.
///
/// Smaller than [`OUTGROWS_A_PIPE`] because this one spawns a process per ask in the shape it refuses, and
/// the property is a ratio rather than a volume.
const MANY_PATHS_ONE_SOURCE: usize = 400;

/// A repository whose `.gitignore` hides `count` files with long names.
///
/// Long deliberately: the deadlock needs the conversation to exceed the kernel's pipe buffers in **bytes**,
/// so bytes-per-file is the cheap axis and files-created is the expensive one.
fn crowded(name: &str, count: usize) -> (PathBuf, PathBuf) {
    let (root, repo) = hiding(name, &[(".gitignore", "/ignored/\n")], "kept.txt");
    let ignored = repo.join("ignored");
    std::fs::create_dir_all(&ignored).expect("create the ignored directory");
    for index in 0..count {
        let stem = "a".repeat(180);
        std::fs::write(ignored.join(format!("{stem}-{index:06}")), "x").expect("write");
    }
    (root, repo)
}

/// The gate reaches a verdict on a repository whose ignored set does not fit in a pipe.
///
/// The negative run: against the write-everything-then-read shape this replaced, this direction does not
/// fail — it *hangs*, `git check-ignore` blocked in `pipe_wait` on a full 64 KB stdout while the judgement
/// blocks on a full 64 KB stdin. So the bound is enforced here rather than left to the harness: a test that
/// never returns reports nothing, and reporting nothing is exactly how this reached a release branch.
#[test]
fn a_repository_whose_ignored_set_outgrows_a_pipe_is_still_answered() {
    let (root, repo) = crowded("crowded-pipe", OUTGROWS_A_PIPE);
    let (tx, rx) = std::sync::mpsc::channel();
    let judging = repo.clone();
    std::thread::spawn(move || {
        let _ = tx.send(hidden_by_the_checkout(&judging).map_err(|refusal| refusal.message));
    });
    let answered = rx.recv_timeout(std::time::Duration::from_secs(60));
    let _ = std::fs::remove_dir_all(&root);
    let hidden = match answered {
        Ok(hidden) => hidden.expect("the classifier reads this repository"),
        Err(_) => panic!(
            "the publish gate reached no verdict in 60s on a repository with {OUTGROWS_A_PIPE} ignored \
             files. It does not refuse and it does not accept — it never returns, so `scripts/publish.sh` \
             hangs at the one moment nothing can be undone"
        ),
    };
    assert!(
        hidden.is_empty(),
        "every one of these files is hidden by a tracked `.gitignore`, so none is the checkout's: {:?}",
        &hidden[..hidden.len().min(3)]
    );
}

/// The tracked-source question is asked once per **source**, not once per path.
///
/// Counted rather than timed. A wall-clock assertion over process spawns is a flake on a loaded machine and
/// says nothing about the shape; the count says the shape directly. Measured on this repository before the
/// repair: 73,670 paths, 73,670 spawns, **one** distinct source — 147 seconds spent asking one question.
#[test]
fn the_tracked_question_is_asked_once_per_source_not_once_per_path() {
    let (root, repo) = crowded("crowded-sources", MANY_PATHS_ONE_SOURCE);
    let asked = std::sync::atomic::AtomicUsize::new(0);
    let hidden = hidden_by_the_checkout_with(&repo, gate::classify, |repo, source| {
        asked.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        match gate::hermetic("git")
            .args(["ls-files", "--error-unmatch", "-z", "--", source])
            .current_dir(repo)
            .output()
        {
            Ok(out) if out.status.success() => Tracked::Yes,
            Ok(_) => Tracked::No,
            Err(err) => Tracked::Unreadable(err.to_string()),
        }
    })
    .expect("the classifier reads this repository");
    let _ = std::fs::remove_dir_all(&root);
    assert!(hidden.is_empty(), "{hidden:?}");
    let asked = asked.load(std::sync::atomic::Ordering::Relaxed);
    assert_eq!(
        asked, 1,
        "{MANY_PATHS_ONE_SOURCE} excluded paths named one source, `.gitignore`, and the tracked question \
         was asked {asked} times. Asked per path it is one process spawn each, which on the 73,670 measured \
         on this repository was 147 seconds of process creation to answer a single question"
    );
}

// --- the scratch directory the signature verdict is reached in ---------------------------------------------

/// A scratch path someone else already owns is refused, not written through.
///
/// The window this models: the gate removes the path and then creates it, and `remove_dir_all` **does**
/// remove a symlink rather than follow it — measured — so an attacker cannot leave one lying around. They can
/// re-create one in the gap, which is why the claim itself has to refuse rather than the removal.
///
/// What it buys, measured on this machine: `create_dir_all` on a symlink-to-directory returns `Ok(())` and
/// the writes land in the link's target. The scratch holds `tag.sig`, which `check_novalidate` reads back, and
/// `ssh-keygen -Y check-novalidate` asks *is this signature valid over this payload* without asking whose key
/// made it. So whoever owns that directory owns both ends of the write-then-read and can substitute a
/// signature over the same payload made with their own key — and a release tag whose signature does not
/// verify over the tag object would verify.
#[test]
fn a_scratch_path_another_user_could_own_is_refused_rather_than_written_through() {
    let root = scratch("claimed-scratch");
    let elsewhere = root.join("someone-elses-directory");
    std::fs::create_dir_all(&elsewhere).expect("create the directory the link would redirect to");
    let claimed = root.join("scratch");
    std::os::unix::fs::symlink(&elsewhere, &claimed).expect("plant the redirect");

    let refusal = gate::claim_scratch(&claimed)
        .expect_err("a path this process did not create must not be adopted as its scratch");
    refusal::expect(
        "publish-source-integrity#signature-scratch-unclaimable",
        &refusal,
    );
    assert_eq!(
        refusal.kind,
        Kind::CannotJudge,
        "a scratch it could not claim is a verdict not reached, not a source that disagrees: {}",
        refusal.message
    );
    assert!(refusal.message.contains("scratch"), "{}", refusal.message);
    assert!(
        std::fs::read_dir(&elsewhere)
            .expect("the redirect target is readable")
            .next()
            .is_none(),
        "the refusal must happen before anything is written, or the signature material has already been \
         handed to whoever owns that directory"
    );

    // The control: the same call on a path nobody holds succeeds, so the refusal above is about the
    // pre-existing path rather than about a claim that never works.
    let fresh = root.join("unclaimed");
    gate::claim_scratch(&fresh).expect("a path nobody holds is claimable");
    assert!(fresh.is_dir());
    let _ = std::fs::remove_dir_all(&root);
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
    refusal::expect(
        "publish-source-integrity#worktree-hides-untracked-files",
        &refusal,
    );
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    assert!(
        refusal.message.contains("only this checkout hides"),
        "{}",
        refusal.message
    );
}

/// A source whose tracked-ness could not be read refuses, rather than counting as untracked.
///
/// `ls-files --error-unmatch` exits non-zero for *this path is untracked* — the question — and also when git
/// cannot be run at all. Folded into a boolean by `.is_ok()`, the second answered the first: a machine
/// without git reported every exclusion source as untracked and the gate refused with *hidden by X, which
/// this repository does not track*. That is an **exit 1**, a disagreement, for a fact never read, in front of
/// the one act that cannot be undone — and this repository reserves `1` for a source that disagrees.
///
/// Negative run: with `tracks` answering a bool, this returns the hidden-file **violation** rather than a
/// cannot-judge, naming a repository fact it never established.
#[test]
fn a_source_whose_tracking_cannot_be_read_is_not_untracked() {
    let (root, repo) = crowded("unreadable-tracking", 3);
    let refusal = hidden_by_the_checkout_with(&repo, gate::classify, |_, source| {
        Tracked::Unreadable(format!("git is not on this machine (asked about {source})"))
    })
    .expect_err("a tracked question that could not be asked must refuse");
    refusal::expect(
        "publish-source-integrity#tracking-question-unaskable",
        &refusal,
    );
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        refusal.kind,
        Kind::CannotJudge,
        "an unread fact is not a disagreement: {}",
        refusal.message
    );
    assert!(
        refusal
            .message
            .contains("could not decide whether this repository tracks"),
        "the refusal must say the question went unasked, got: {}",
        refusal.message
    );
}
