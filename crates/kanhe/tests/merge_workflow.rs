//! Controlled execution of the sanctioned squash-merge workflow.
//!
//! The wrapper gathers evidence and orders external commands; the message verdict remains in
//! `merge_message.rs`. These directions replace `gh` and `cargo`, so no network call or merge can occur.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicUsize, Ordering};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("scripts/merge-pr.sh").is_file(),
        shengmo::workspace::marker_set(),
    )
}

struct Run {
    status: ExitStatus,
    stdout: String,
    stderr: String,
    gh_log: String,
    cargo_log: String,
    commits: String,
}

fn write_executable(path: &Path, text: &str) {
    use std::os::unix::fs::PermissionsExt;

    std::fs::write(path, text).expect("write controlled executable");
    let mut permissions = std::fs::metadata(path)
        .expect("read controlled executable metadata")
        .permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(path, permissions).expect("make controlled executable runnable");
}

fn read_if_present(path: &Path) -> std::io::Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(err) => Err(err),
    }
}

fn run_wrapper(root: &Path, mode: &str, extra: &[&str]) -> Run {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = loop {
        let candidate = std::env::temp_dir().join(format!(
            "tianheng-merge-workflow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => {
                assert_eq!(
                    err.kind(),
                    std::io::ErrorKind::AlreadyExists,
                    "cannot acquire controlled merge-workflow root {}: {err}",
                    candidate.display()
                );
            }
        }
    };
    let bin = scratch.join("bin");
    std::fs::create_dir(&bin).expect("create controlled PATH");

    let gh_log = scratch.join("gh.log");
    let cargo_log = scratch.join("cargo.log");
    let commits = scratch.join("commits");
    let body = scratch.join("body.md");
    std::fs::write(
        &body,
        "Why this change exists and what contract it preserves.\n",
    )
    .expect("write merge body");

    write_executable(
        &bin.join("gh"),
        r##"#!/usr/bin/env bash
set -eu
printf '%s\n' "$*" >> "$FAKE_GH_LOG"
if [[ $1 == repo && $2 == view ]]; then
    printf '%s\n' 'tacticaldoll/tianheng'
elif [[ $1 == pr && $2 == view && $* == *"--json title"* ]]; then
    printf '%s\n' 'fix(kanhe): harden workflow evidence'
elif [[ $1 == pr && $2 == view && $* == *"--json number"* ]]; then
    if [[ $FAKE_GH_MODE == invalid-number ]]; then
        printf '%s\n' 'not-a-number'
    else
        printf '%s\n' '42'
    fi
elif [[ $1 == api ]]; then
    case $FAKE_GH_MODE in
    api-failure)
        printf '%s\n' 'controlled API failure' >&2
        exit 91
        ;;
    empty)
        :
        ;;
    subjects | invalid-number)
        if [[ $* != *"--paginate"* ]]; then
            printf '%s\n' 'feat(x): live first subject'
        else
            printf '%s\n' \
                'feat(x): live first subject' \
                'fix(y): live subject absent from local refs remains complete beyond headline truncation' \
                'docs(z): subject from the next page'
        fi
        ;;
    *)
        printf 'unexpected fake mode: %s\n' "$FAKE_GH_MODE" >&2
        exit 92
        ;;
    esac
elif [[ $1 == pr && $2 == merge ]]; then
    :
else
    printf 'unexpected gh invocation: %s\n' "$*" >&2
    exit 97
fi
"##,
    );
    write_executable(
        &bin.join("cargo"),
        r##"#!/usr/bin/env bash
set -eu
printf '%s\n' "$*" >> "$FAKE_CARGO_LOG"
printf '%s' "${TIANHENG_MERGE_COMMITS-}" > "$FAKE_COMMITS"
printf '%s\n' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
"##,
    );

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin.display(), old_path.to_string_lossy());
    let output = Command::new("bash")
        .arg(root.join("scripts/merge-pr.sh"))
        .args(["42", "--body-file"])
        .arg(&body)
        .args(extra)
        .env("PATH", path)
        .env("FAKE_GH_MODE", mode)
        .env("FAKE_GH_LOG", &gh_log)
        .env("FAKE_CARGO_LOG", &cargo_log)
        .env("FAKE_COMMITS", &commits)
        .output()
        .expect("run controlled merge workflow");

    let run = Run {
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        gh_log: read_if_present(&gh_log).expect("read controlled gh log"),
        cargo_log: read_if_present(&cargo_log).expect("read controlled cargo log"),
        commits: read_if_present(&commits).expect("read commits received by controlled gate"),
    };
    let _ = std::fs::remove_dir_all(&scratch);
    run
}

#[test]
fn live_pull_request_commits_reach_the_gate_without_local_refs() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "subjects", &[]);
    assert!(
        run.status.success(),
        "controlled workflow failed:\nstdout:\n{}\nstderr:\n{}",
        run.stdout,
        run.stderr
    );
    assert!(
        run.gh_log.contains("api --paginate"),
        "the live pull-request commits API was not asked for:\n{}",
        run.gh_log
    );
    assert!(
        run.gh_log.contains("pulls/42/commits") && run.gh_log.contains("commit.message"),
        "the API call must use the canonical number and full-message projection:\n{}",
        run.gh_log
    );
    assert_eq!(
        run.commits,
        "feat(x): live first subject\nfix(y): live subject absent from local refs remains complete beyond headline truncation\ndocs(z): subject from the next page",
        "the Rust gate must receive the complete live commit set; cargo invocation: {}",
        run.cargo_log
    );
}

fn assert_stopped_before_gate_and_merge(run: &Run) {
    assert!(
        !run.status.success(),
        "incomplete live pull-request evidence must fail"
    );
    assert!(
        run.cargo_log.is_empty(),
        "the Rust gate ran without complete live commit evidence: {}",
        run.cargo_log
    );
    assert!(
        !run.gh_log.lines().any(|line| line.starts_with("pr merge ")),
        "the irreversible merge command was reached:\n{}",
        run.gh_log
    );
}

#[test]
fn a_failed_live_commit_read_stops_before_the_gate_and_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "api-failure", &[]);
    assert_stopped_before_gate_and_merge(&run);
    assert!(
        run.stderr.contains("controlled API failure"),
        "the failed read's cause must remain visible: {}",
        run.stderr
    );
}

#[test]
fn an_empty_live_commit_set_stops_before_the_gate_and_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "empty", &[]);
    assert_stopped_before_gate_and_merge(&run);
    assert!(
        run.stderr.contains("cannot read any commit subjects"),
        "the empty-set refusal must name the missing evidence: {}",
        run.stderr
    );
}

#[test]
fn an_unresolved_canonical_pull_request_number_stops_before_live_acquisition() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "invalid-number", &[]);
    assert_stopped_before_gate_and_merge(&run);
    assert!(
        run.stderr
            .contains("cannot resolve 42 to one pull request number"),
        "the identity refusal must name the unresolved selector: {}",
        run.stderr
    );
    assert!(
        !run.gh_log.lines().any(|line| line.starts_with("api ")),
        "the commits endpoint was built from an unvalidated selector:\n{}",
        run.gh_log
    );
}

/// A value-taking flag given no value is named, not silently arithmetic.
///
/// The shape this closes: `shift 2` with one argument left returns non-zero, `set -e` takes that as the exit,
/// and the wrapper stops with **no output at all** — in a script where every other refusal prints
/// `merge message: …`. A missing flag value is an observable misconfiguration, which is precisely what the
/// minimalism bound says to fail loud on, and the operator meets it at the moment before a record lands and
/// stops being repairable.
///
/// No controlled `PATH` is needed: this refusal happens before the wrapper reaches `gh` or the gate, which is
/// itself part of the claim — the assertions below require that nothing was invoked.
#[test]
fn a_value_taking_flag_with_no_value_is_named_and_refused() {
    let Some(root) = workspace_root() else {
        return;
    };
    for flag in ["--subject", "--body-file"] {
        let output = Command::new("bash")
            .arg(root.join("scripts/merge-pr.sh"))
            .args(["42", flag])
            .output()
            .expect("run the wrapper with a flag given no value");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(2),
            "a missing flag value is a usage refusal, not a silent stop; got {:?} with stderr {stderr:?}",
            output.status.code()
        );
        assert!(
            stderr.contains("merge message:") && stderr.contains(flag),
            "the refusal must name the flag in this script's own diagnostic form, got {stderr:?}"
        );
        assert!(
            stderr.contains("usage:"),
            "the refusal must show the usage the operator needs, got {stderr:?}"
        );
    }
}

/// A repository selector is refused **before** any evidence is read.
///
/// The hole it closes: the title, the canonical pull-request number, the live commit subjects and the gate are
/// all read from the ambient repository, while a repository selector reaches only the final `gh pr merge`. One
/// argument would therefore have this wrapper judge pull request N here and merge pull request N somewhere
/// else — the gate's whole claim undone by an argument, which is what `scripts/publish.sh` already refuses
/// `--manifest-path` for.
///
/// **The exit code is the weaker half of this assertion.** What matters is the ORDER, so the controlled `gh`
/// logs every invocation and the log must be empty: a refusal printed after the title had already been fetched
/// would still exit 2 while having read the wrong repository's evidence.
///
/// **What this direction holds is the reason, not the spelling class.** It once claimed to cover every
/// spelling and named the three its arms had been written for, which left `gh`'s glued and equals forms of the
/// same flag open — a claim about a class, made by enumerating a sample of it. The class is
/// [`only_an_allowlisted_flag_reaches_the_merge`]'s to hold, by refusing anything unlisted rather than by
/// listing; what this one adds is that the refusal an operator reads names the repository problem.
#[test]
fn a_repository_selector_is_refused_before_any_evidence_is_read() {
    let Some(root) = workspace_root() else {
        return;
    };
    for selector in ["--repo", "--repo=other/thing", "-R"] {
        let run = run_wrapper(&root, "subjects", &[selector, "other/thing"]);
        assert_eq!(
            run.status.code(),
            Some(2),
            "`{selector}` must be refused as a usage error; got {:?} with stderr {:?}",
            run.status.code(),
            run.stderr
        );
        assert!(
            run.stderr.contains("merge message:") && run.stderr.contains("judge one pull request"),
            "the refusal must say why in this script's own diagnostic form, got {:?}",
            run.stderr
        );
        assert!(
            run.gh_log.is_empty(),
            "`{selector}` must be refused before any evidence is read, but gh was invoked:\n{}",
            run.gh_log
        );
        assert!(
            run.cargo_log.is_empty(),
            "`{selector}` must be refused before the gate runs, but cargo was invoked:\n{}",
            run.cargo_log
        );
    }
}

/// Every `gh` call names the **same** repository, and a selector naming another one is refused first.
///
/// The hole: `gh pr view` and `gh pr merge` follow a pull-request URL to its own repository, while the
/// live-commits endpoint was built from a placeholder `gh` expands from the working directory. A
/// cross-repository URL therefore had the gate judge one pull request and the merge record another — the same
/// hole a `--repo` flag opened, reopened through the positional selector.
///
/// Two assertions, because they answer different questions. The refusal is what closes it and is decidable
/// offline: a URL names a repository and this wrapper reads its evidence from the one it runs in. The identity
/// pin is what keeps the calls from diverging **again**: four references defaulting to the same place is
/// agreement by circumstance, and a fifth call added later would inherit the circumstance rather than the rule.
#[test]
fn every_call_names_one_repository_and_another_one_is_refused() {
    let Some(root) = workspace_root() else {
        return;
    };

    // Refused before anything is read: a URL names a repository this wrapper does not read its evidence from.
    let url = run_wrapper(&root, "subjects", &[]);
    let refused = std::process::Command::new("bash")
        .arg(root.join("scripts/merge-pr.sh"))
        .args(["https://github.com/other/thing/pull/42", "--body-file"])
        .arg(root.join("README.md"))
        .output()
        .expect("run the wrapper with a cross-repository URL");
    assert_eq!(
        refused.status.code(),
        Some(2),
        "a pull-request URL must be refused as a usage error"
    );
    let stderr = String::from_utf8_lossy(&refused.stderr);
    assert!(
        stderr.contains("merge message:") && stderr.contains("names its own repository"),
        "the refusal must say why, got {stderr:?}"
    );

    // And on the accepted path, every invocation carries the one identity this checkout resolved.
    let invocations: Vec<&str> = url
        .gh_log
        .lines()
        .filter(|line| !line.starts_with("repo view"))
        .collect();
    assert!(
        !invocations.is_empty(),
        "the accepted path must reach gh, or the assertion below holds vacuously:\n{}",
        url.gh_log
    );
    for invocation in invocations {
        assert!(
            invocation.contains("tacticaldoll/tianheng"),
            "every gh call must name the resolved repository, but this one does not: {invocation}"
        );
    }
}

/// Only an allowlisted flag reaches the merge; every other spelling is refused before anything runs.
///
/// **The property is not "these six are refused" — it is that an unlisted flag is refused by default.** This
/// guard was a denylist and leaked three times: a `--repo` flag, a positional URL, and every short spelling of
/// the flags its long-form arms named. `gh` accepts `-t` for `--subject` and `-F` for `--body-file`, the
/// passthrough is spliced after this wrapper's own flags, and `gh` reads the **last** occurrence of a repeated
/// flag — so one unlisted spelling replaced the message the gate had just approved.
///
/// The refused set below is therefore a sample of a rule, not the rule. What makes it hold for a spelling
/// nobody thought of is the catch-all, and the second half of this direction is what shows the allowlist did
/// not simply refuse everything.
#[test]
fn only_an_allowlisted_flag_reaches_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };

    // Every spelling `gh` accepts for a flag that moves what the gate judged — long, short, glued, equals.
    for spelling in [
        "-t",
        "-t=x",
        "-tx",
        "-F",
        "-Fx",
        "--subject=x",
        "--body-file=x",
        "--body",
        "-b",
        "-R",
        "-Rowner/repo",
        "-R=x",
        "--repo=x",
        "-m",
        "-r",
        "-s",
        "-A",
        "--author-email",
        // And one nobody classified: an argument the wrapper does not know is refused, not passed on.
        "--some-flag-a-future-gh-adds",
    ] {
        let run = run_wrapper(&root, "subjects", &[spelling]);
        assert_eq!(
            run.status.code(),
            Some(2),
            "`{spelling}` must be refused; got {:?} with stderr {:?}",
            run.status.code(),
            run.stderr
        );
        assert!(
            run.stderr.contains("merge message:"),
            "`{spelling}` must be refused in this script's own diagnostic form, got {:?}",
            run.stderr
        );
        assert!(
            run.gh_log.is_empty() && run.cargo_log.is_empty(),
            "`{spelling}` must be refused before anything runs, but gh log was {:?} and cargo log {:?}",
            run.gh_log,
            run.cargo_log
        );
    }

    // The other half: a flag that changes whether the merge may proceed, never what it records, still arrives.
    // Without this the assertions above are satisfied by a wrapper that refuses its own arguments.
    let forwarded = run_wrapper(&root, "subjects", &["--delete-branch"]);
    assert!(
        forwarded.status.success(),
        "an allowlisted flag must not be refused, got {:?} with stderr {:?}",
        forwarded.status.code(),
        forwarded.stderr
    );
    let merge = forwarded
        .gh_log
        .lines()
        .find(|line| line.starts_with("pr merge"))
        .unwrap_or_else(|| panic!("the merge must be reached:\n{}", forwarded.gh_log));
    assert!(
        merge.contains("--delete-branch"),
        "the allowlisted flag must reach the merge, got {merge}"
    );
}
