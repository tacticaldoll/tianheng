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

fn run_wrapper(root: &Path, mode: &str) -> Run {
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
if [[ $1 == pr && $2 == view && $* == *"--json title"* ]]; then
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
    let run = run_wrapper(&root, "subjects");
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
    let run = run_wrapper(&root, "api-failure");
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
    let run = run_wrapper(&root, "empty");
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
    let run = run_wrapper(&root, "invalid-number");
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
