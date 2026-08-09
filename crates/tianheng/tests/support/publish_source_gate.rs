//! The publish-source judgement, and one builder for the repository shape it judges.
//!
//! Shared by the gate (`publish_source.rs`, which runs it over this repository at publish time and over
//! fixtures in its failure matrix) and by the pin that cites this capability's declared bound
//! (`publish_source_integrity.rs`). Two constructions of "a signed release repository" is the twin-drift
//! class this repository keeps closing, so there is one, here, and both callers use it.
//!
//! It stands before an **irreversible act**: `cargo publish` records the commit it ran on in every tarball and
//! a version can never be re-uploaded. So the separation the shell gate held is kept in the type rather than
//! in an exit code — a [`Refusal`] is either a violation (the source disagrees) or a cannot-judge (the source
//! could not be read), and collapsing the two would tell an operator to go looking for a disagreement that
//! does not exist.

#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::refusal::{Refusal, cannot_judge, cannot_judge_out_of_reach, violation};

/// The judgement's own git, isolated from everything outside the repository it judges.
///
/// This file already had [`hermetic`] and used it for its **fixtures**; the judgement ran through a bare
/// `Command::new("git")`. The fixtures were isolated and the verdict was not, which is the wrong way round —
/// a `core.excludesFile` outside the repository made the cleanliness read return empty for an untracked file.
///
/// Neutralising `core.excludesFile` **explicitly** is the load-bearing half, measured rather than assumed:
///
/// | ambient source | hermetic alone | `-c core.excludesFile=/dev/null` |
/// |---|---|---|
/// | global / system `core.excludesFile` | closed | closed |
/// | `$XDG_CONFIG_HOME/git/ignore`, the default no config names | **survives** | closed |
/// | `.git/info/exclude` | **survives** | **survives** |
///
/// Routing through the builder and stopping there would have read as a repair while the XDG default still hid
/// files. What the third row costs is handled by [`hidden_by_the_checkout`], which classifies rather than
/// refuses.
fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = hermetic("git")
        .args(["-c", "core.excludesFile=/dev/null"])
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|err| format!("cannot run git {args:?}: {err}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim_end().to_string())
    }
}

/// One read of the worktree, and **one** refusal site serving every caller.
///
/// Two `cannot_judge` calls carrying one message would be shadowing: a direction asserting the message could
/// not say which read failed, and replacing either site's message would leave it passing. One site, and the
/// caller says what it was reading.
fn read_worktree(repo: &Path, args: &[&str], what: &str) -> Result<String, Refusal> {
    git(repo, args)
        .map_err(|err| cannot_judge(format!("could not read the worktree {what}: {err}")))
}

/// Untracked files hidden by this **checkout** rather than by the repository.
///
/// `clean` is defined by the repository: a file ignored by tracked repository content is clean, because
/// `cargo publish` applies the same exclusion and would not package it either. A file hidden by this clone's
/// `.git/info/exclude`, or by a `core.excludesFile` on this machine, is not — the same commit would otherwise
/// get different verdicts in different places.
///
/// `ls-files --others` applies no exclusion and `status` applies all of them, so the difference is the
/// excluded set; `check-ignore -v` names the source file for each. A source counts as repository content only
/// if it is **tracked** — measured, not read: an *untracked* `.gitignore` reports a repository-looking source
/// while being no more part of the repository than the clone's own exclude file.
///
/// An exclusion whose source cannot be shown to be tracked is treated as the checkout's. That is the
/// conservative direction and it is deliberate: the alternative is granting an exemption on the strength of
/// not having read one.
pub fn hidden_by_the_checkout(repo: &Path) -> Result<Vec<String>, Refusal> {
    let unexcluded = read_worktree(repo, &["ls-files", "--others"], "untracked files")?;
    let visible = read_worktree(
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
        "state",
    )?;
    let shown: Vec<&str> = visible
        .lines()
        .filter_map(|line| line.strip_prefix("?? "))
        .collect();
    let excluded: Vec<&str> = unexcluded
        .lines()
        .filter(|path| !path.is_empty() && !shown.contains(path))
        .collect();
    if excluded.is_empty() {
        return Ok(Vec::new());
    }

    let mut args = vec!["check-ignore", "-v", "--no-index", "--"];
    args.extend(excluded.iter().copied());
    // `check-ignore` exits 1 when it matched nothing, which for a path this function computed as excluded is
    // a disagreement between two listings rather than an answer. Either way the source is unshown, and an
    // unshown source is the checkout's.
    let classified = git(repo, &args).unwrap_or_default();

    let mut sources: std::collections::BTreeMap<&str, &str> = std::collections::BTreeMap::new();
    for line in classified.lines() {
        let Some((left, path)) = line.split_once('\t') else {
            continue;
        };
        // `<source>:<line>:<pattern>` — split from the right, since a source path may hold a colon.
        let mut fields = left.rsplitn(3, ':');
        let (_pattern, _line, source) = (fields.next(), fields.next(), fields.next());
        if let Some(source) = source {
            sources.insert(path, source);
        }
    }

    let mut hidden = Vec::new();
    for path in excluded {
        let source = sources.get(path).copied().unwrap_or("<unshown>");
        let tracked = source != "<unshown>"
            && git(repo, &["ls-files", "--error-unmatch", "--", source]).is_ok();
        if !tracked {
            hidden.push(format!(
                "  {path} — hidden by {source}, which this repository does not track"
            ));
        }
    }
    Ok(hidden)
}

/// The `[workspace.package]` version, or the `[package]` version where there is no workspace table.
fn workspace_version(repo: &Path) -> Option<String> {
    let text = std::fs::read_to_string(repo.join("Cargo.toml")).ok()?;
    let mut in_package = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            in_package = trimmed == "[workspace.package]" || trimmed == "[package]";
            continue;
        }
        if in_package {
            if let Some(rest) = trimmed.strip_prefix("version") {
                let rest = rest.trim_start();
                if let Some(rest) = rest.strip_prefix('=') {
                    return Some(rest.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    None
}

fn is_semver(version: &str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();
    parts.len() == 3
        && parts.iter().all(|p| {
            !p.is_empty()
                && p.chars().all(|c| c.is_ascii_digit())
                && (p.len() == 1 || !p.starts_with('0'))
        })
}

/// Judge whether `repo` is the source a release publishes from.
///
/// `remote` names the remote whose `main` the snapshot must be the live tip of.
pub fn judge(repo: &Path, remote: &str) -> Result<String, Refusal> {
    if !repo.join("Cargo.toml").is_file() {
        return Err(cannot_judge(format!(
            "repository root {} has no Cargo.toml",
            repo.display()
        )));
    }
    git(repo, &["rev-parse", "--is-inside-work-tree"]).map_err(|_| {
        cannot_judge(format!(
            "repository root {} is not a git worktree",
            repo.display()
        ))
    })?;

    let version = workspace_version(repo).unwrap_or_default();
    if !is_semver(&version) {
        return Err(cannot_judge(format!(
            "workspace version is missing or malformed: {}",
            if version.is_empty() {
                "<missing>"
            } else {
                &version
            }
        )));
    }
    let tag = format!("v{version}");

    // HEAD describes what would be packaged only if nothing is uncommitted or untracked.
    let dirty = read_worktree(
        repo,
        &["status", "--porcelain=v1", "--untracked-files=all"],
        "state",
    )?;
    if !dirty.is_empty() {
        return Err(violation(format!(
            "worktree is not clean, so HEAD does not describe what would be packaged:\n{dirty}"
        )));
    }
    let hidden = hidden_by_the_checkout(repo)?;
    if !hidden.is_empty() {
        return Err(violation(format!(
            "worktree carries untracked files that only this checkout hides, so the same commit would be \
             judged differently elsewhere:\n{}",
            hidden.join("\n")
        )));
    }

    // Shares the read site above. Once `clean` is defined by the repository, a clean worktree with an
    // unresolvable HEAD cannot be constructed — measured: every route leaves either an untracked file the
    // checkout hides or a staged one `status` reports — so a refusal of its own would be a branch no input
    // can take.
    let head_commit = read_worktree(repo, &["rev-parse", "HEAD"], "HEAD")?;
    let head_subject = git(repo, &["log", "-1", "--format=%s", "HEAD"])
        .map_err(|err| cannot_judge(format!("could not read HEAD's subject: {err}")))?;
    if head_subject != format!("release: {version}") {
        return Err(violation(format!(
            "HEAD is not this version's release snapshot: its subject is \"{head_subject}\", expected \
             \"release: {version}\""
        )));
    }

    if git(
        repo,
        &["rev-parse", "--verify", &format!("refs/tags/{tag}")],
    )
    .is_err()
    {
        return Err(violation(format!(
            "there is no tag {tag}; the release snapshot is tagged before it is published"
        )));
    }
    // The tag object, read **once**. Asking git for its type and then for its content is two reads of one
    // object, and the second cannot fail once the first has answered — a branch no input can take, which is
    // dead code rather than a guard. So the content is read first, and the type is asked for only to say what
    // that failure *means*: a lightweight tag is a violation, an unreadable object is not.
    let tag_object = match git(repo, &["cat-file", "tag", &format!("refs/tags/{tag}")]) {
        Ok(object) => object,
        Err(err) => {
            let kind = git(repo, &["cat-file", "-t", &format!("refs/tags/{tag}")]);
            return Err(match kind.as_deref() {
                Ok("tag") | Err(_) => {
                    cannot_judge(format!("could not read the tag object for {tag}: {err}"))
                }
                Ok(_) => violation(format!(
                    "{tag} is a lightweight tag; the release tags are annotated (`git tag -s`)"
                )),
            });
        }
    };

    verify_tag_signature(repo, &tag, &tag_object)?;

    let tag_commit = git(repo, &["rev-list", "-n", "1", &tag])
        .map_err(|err| cannot_judge(format!("could not resolve {tag} to a commit: {err}")))?;
    if tag_commit != head_commit {
        return Err(violation(format!(
            "{tag} points at {tag_commit} but HEAD is {head_commit}; publish the commit the tag names"
        )));
    }

    let listing = git(repo, &["ls-remote", remote, "refs/heads/main"]).unwrap_or_default();
    let remote_main = listing
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().next())
        .unwrap_or_default()
        .to_string();
    if remote_main.is_empty() {
        return Err(cannot_judge(format!(
            "could not read refs/heads/main from remote \"{remote}\", so whether HEAD is the released \
             snapshot cannot be decided"
        )));
    }
    if remote_main != head_commit {
        return Err(violation(format!(
            "HEAD {head_commit} is not the tip of {remote}/main ({remote_main}); `main` is the release-only \
             branch a publish runs from"
        )));
    }

    Ok(format!(
        "ok publish source ({remote}/main at {head_commit}, tagged {tag})"
    ))
}

/// The tag must carry an SSH signature that verifies **over the tag object**.
///
/// A signature block quoted in a tag *message* is text; only the payload the object actually signs decides.
fn verify_tag_signature(repo: &Path, tag: &str, tag_object: &str) -> Result<(), Refusal> {
    if Command::new("ssh-keygen").arg("-h").output().is_err() {
        return Err(cannot_judge_out_of_reach(
            "ssh-keygen-absent",
            format!("ssh-keygen is unavailable, so {tag}'s signature cannot be verified"),
        ));
    }
    // Unique per CALL, not per (process, tag). Every fixture in the failure matrix tags `v9.9.9`, and the
    // matrix runs in parallel, so a key built from the tag had each test's `Drop` deleting another's scratch
    // mid-verification — a test that passed alone and failed beside its siblings.
    static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-publish-source-sig-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).map_err(|err| {
        cannot_judge_out_of_reach(
            "signature-scratch-not-creatable",
            format!("could not create a signature scratch dir: {err}"),
        )
    })?;
    let guard = Scratch(scratch.clone());

    // The mechanism proves itself before it is trusted to judge: a round trip over a throwaway key. Without
    // it, a broken `ssh-keygen -Y` would refuse every signature and read as a violation.
    let probe = scratch.join("probe");
    let round_trip = Command::new("ssh-keygen")
        .args(["-q", "-t", "ed25519", "-N", "", "-f"])
        .arg(&probe)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
        && sign_probe(&probe, &scratch);
    if !round_trip {
        return Err(cannot_judge_out_of_reach(
            "signature-mechanism-round-trip",
            format!(
                "the signature mechanism failed its own round-trip, so no verdict on {tag}'s signature \
                 would be about {tag}"
            ),
        ));
    }

    let signature = git(
        repo,
        &[
            "for-each-ref",
            "--format=%(contents:signature)",
            &format!("refs/tags/{tag}"),
        ],
    )
    .map_err(|err| {
        cannot_judge_out_of_reach(
            "signature-block-unreadable",
            format!("could not read {tag}'s signature block: {err}"),
        )
    })?;

    if signature.trim().is_empty() {
        return Err(violation(format!(
            "{tag} carries no signature; the release tags are signed (`git tag -s`)"
        )));
    }
    if !signature.starts_with("-----BEGIN SSH SIGNATURE-----") {
        return Err(cannot_judge(format!(
            "{tag} carries a signature this gate cannot verify — it reads SSH signatures, and this block is \
             something else"
        )));
    }
    let Some(payload) = tag_object.strip_suffix(signature.trim_end()) else {
        return Err(cannot_judge_out_of_reach(
            "signature-block-not-a-suffix",
            format!(
                "{tag}'s extracted signature is not the tag object's suffix, so the signed payload cannot \
                 be reconstructed"
            ),
        ));
    };

    let sig_path = scratch.join("tag.sig");
    std::fs::write(&sig_path, format!("{}\n", signature.trim_end())).map_err(|err| {
        cannot_judge_out_of_reach(
            "signature-file-not-writable",
            format!("could not write the signature for checking: {err}"),
        )
    })?;
    let verified = check_novalidate(payload, &sig_path);
    drop(guard);
    if !verified {
        return Err(violation(format!(
            "{tag}'s signature does not verify over the tag object; a signature block quoted in a tag \
             message is text, not a signature"
        )));
    }
    Ok(())
}

struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn pipe_into(mut child: std::process::Child, payload: &str) -> bool {
    use std::io::Write;
    if let Some(stdin) = child.stdin.as_mut() {
        let _ = stdin.write_all(payload.as_bytes());
    }
    drop(child.stdin.take());
    child.wait().map(|s| s.success()).unwrap_or(false)
}

fn sign_probe(key: &Path, scratch: &Path) -> bool {
    use std::io::Write;
    let sig = scratch.join("probe.sig");
    let Ok(mut child) = Command::new("ssh-keygen")
        .args(["-Y", "sign", "-n", "git", "-f"])
        .arg(key)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        return false;
    };
    // The payload must reach stdin before the child is waited on. Spawning and waiting immediately closes
    // stdin at once, so `ssh-keygen` signs the EMPTY payload and the round trip then fails against "probe" —
    // reporting the mechanism broken when only the harness was.
    if let Some(stdin) = child.stdin.as_mut() {
        if stdin.write_all(b"probe").is_err() {
            return false;
        }
    }
    drop(child.stdin.take());
    let Ok(out) = child.wait_with_output() else {
        return false;
    };
    if !out.status.success() || std::fs::write(&sig, &out.stdout).is_err() {
        return false;
    }
    check_novalidate("probe", &sig)
}

fn check_novalidate(payload: &str, signature: &Path) -> bool {
    let Ok(child) = Command::new("ssh-keygen")
        .args(["-Y", "check-novalidate", "-n", "git", "-s"])
        .arg(signature)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    else {
        return false;
    };
    pipe_into(child, payload)
}

// --- the fixture ------------------------------------------------------------------------------------------

/// A repository in the exact shape a publish runs from: `main` pushed to a bare remote, its tip a
/// `release: <version>` snapshot, tagged with a signed annotated tag, worktree clean.
///
/// The caller owns the root and its cleanup, because a builder that also decided lifetime would make a
/// caller's single guard two.
pub struct Fixture {
    pub repo: PathBuf,
    pub remote: PathBuf,
    pub key: PathBuf,
}

/// Every fixture command runs **hermetically**.
///
/// Measured rather than assumed: without this the fixture inherited this repository's own signing
/// configuration, so `git tag -a` produced a genuinely signed tag where the fixture wanted an unsigned one,
/// and a bare `git tag` demanded a message. A fixture that inherits the judged machine cannot demonstrate a
/// refusal, because the shape it builds is not the shape it named.
pub fn hermetic(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command
}

fn run(dir: &Path, program: &str, args: &[&str]) {
    let out = hermetic(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {program} {args:?}: {err}"));
    assert!(
        out.status.success(),
        "{program} {args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

pub fn build_fixture(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    let remote = root.join(format!("{name}-origin.git"));
    let key = root.join(format!("{name}-key"));
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");

    run(
        root,
        "ssh-keygen",
        &[
            "-q",
            "-t",
            "ed25519",
            "-N",
            "",
            "-C",
            "fixture",
            "-f",
            &key.display().to_string(),
        ],
    );
    run(
        root,
        "git",
        &["init", "-q", "--bare", &remote.display().to_string()],
    );
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    for (k, v) in [
        ("user.name", "Publish Source Test"),
        ("user.email", "publish-source@example.invalid"),
        ("gpg.format", "ssh"),
        ("commit.gpgsign", "false"),
        ("tag.gpgsign", "false"),
        ("tag.forceSignAnnotated", "false"),
    ] {
        run(&repo, "git", &["config", k, v]);
    }
    run(
        &repo,
        "git",
        &["config", "user.signingkey", &key.display().to_string()],
    );

    std::fs::write(
        repo.join("Cargo.toml"),
        format!("[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"{version}\"\n"),
    )
    .expect("write the fixture manifest");
    run(&repo, "git", &["add", "."]);
    run(
        &repo,
        "git",
        &["commit", "-qm", &format!("release: {version}")],
    );
    run(
        &repo,
        "git",
        &[
            "tag",
            "-s",
            &format!("v{version}"),
            "-m",
            &format!("v{version}"),
        ],
    );
    run(
        &repo,
        "git",
        &["remote", "add", "origin", &remote.display().to_string()],
    );
    run(&repo, "git", &["push", "-q", "origin", "main"]);

    Fixture { repo, remote, key }
}
