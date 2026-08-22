//! Controlled execution of the sanctioned squash-merge workflow.
//!
//! The wrapper gathers evidence and orders external commands; the message verdict remains in
//! `merge_message.rs`. These directions replace `gh` and `cargo`, so no network call or merge can occur.

use std::collections::BTreeSet;
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

/// The body written to the fixture file, and therefore the body the gate judges.
const JUDGED_BODY: &str = "Why this change exists and what contract it preserves.\n\
                           \n\
                           A second paragraph, so the value carries a newline of its own.\n";

/// What the controlled `cargo` writes over the body file while standing where the gate runs.
const REWRITTEN_BODY: &str = "A body nobody judged, written while the gate was running.\n";

/// The body a merge should record: the judged file's content as the wrapper's `$(cat …)` yields it.
///
/// Command substitution strips trailing newlines, and the controlled `gh` normalises the `--body-file` path
/// the same way, so the two arms differ only in **which content** they carry — never in how it was trimmed.
fn judged_value() -> &'static str {
    JUDGED_BODY.trim_end_matches('\n')
}

struct Run {
    /// Anything the wrapper left in the isolated `TMPDIR` it was given.
    leftover: Vec<String>,
    status: ExitStatus,
    stdout: String,
    stderr: String,
    gh_log: String,
    /// The body the merge would **record**, resolved by the controlled `gh` the way the real tool resolves it.
    ///
    /// Separate from [`Run::gh_log`] because the question it answers is different: the log says which
    /// arguments were spelled, this says what the act would write. A direction asserting the flag name would
    /// pass for a wrapper spelling `--body "$(cat "$body_file")"` at merge time, which re-reads and is the
    /// defect.
    gh_body: String,
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

/// The wrapper run from the workspace it lives in, which is how every direction but one exercises it.
fn run_wrapper(root: &Path, mode: &str, extra: &[&str]) -> Run {
    run_wrapper_in(root, mode, extra, None)
}

/// The wrapper run from `cwd`, or from this process's own directory when it is `None`.
///
/// Split because the wrapper reads its gate from its own tree and its evidence from the working directory,
/// and a harness that never varies the second cannot construct the case where they differ.
fn run_wrapper_in(root: &Path, mode: &str, extra: &[&str], cwd: Option<&Path>) -> Run {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = loop {
        let candidate = std::env::temp_dir().join(format!(
            "tianheng-merge-workflow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match xingbiao::claim_scratch(&candidate) {
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
    // The wrapper's own `TMPDIR`, so what it leaves behind is observable and lands in the fixture rather than in
    // the developer's `/tmp`.
    let tmp = scratch.join("tmp");
    std::fs::create_dir(&tmp).expect("create the wrapper's temporary directory");

    let gh_log = scratch.join("gh.log");
    let gh_body = scratch.join("gh.body");
    let cargo_log = scratch.join("cargo.log");
    let commits = scratch.join("commits");
    let body = scratch.join("body.md");
    // Deliberately **multi-line**, which is what a curated squash body actually is. A single-line fixture
    // would let the value travel through `argv` without ever exercising the newline the controlled `gh` has
    // to log safely, so every direction here would pass while the shape they all read stayed fragile.
    std::fs::write(&body, JUDGED_BODY).expect("write merge body");
    if mode == "unreadable-body" {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = std::fs::metadata(&body)
            .expect("read merge body metadata")
            .permissions();
        permissions.set_mode(0o000);
        std::fs::set_permissions(&body, permissions).expect("make the merge body unreadable");
    }

    write_executable(
        &bin.join("gh"),
        r##"#!/usr/bin/env bash
set -eu
# Logged with newlines escaped, so ONE invocation stays ONE log line even when an argument carries them — a
# curated body travelling in `argv` does. Directions locate the merge with `starts_with("pr merge")` and then
# read later arguments off that same line; a split invocation would leave those reading a line that no longer
# holds what they assert, silently and for a reason unrelated to what they test.
argv=$*
printf '%s\n' "${argv//$'\n'/\\n}" >> "$FAKE_GH_LOG"
if [[ $1 == repo && $2 == view ]]; then
    printf '%s\n' 'tacticaldoll/tianheng'
elif [[ $1 == pr && $2 == view && $* == *"--json title"* ]]; then
    # The wrapper reads the title TWICE: once as evidence for the gate, once after it, so the subject it is
    # about to record is still the title. Answering the same string both times is what left the second read
    # unexercised — the guard was shipped without a direction that could tell it apart from its absence.
    # A counter on disk is what makes "the second call" mean second across two separate processes.
    calls=$(cat "$FAKE_TITLE_CALLS" 2>/dev/null || printf '0')
    calls=$((calls + 1))
    printf '%s' "$calls" > "$FAKE_TITLE_CALLS"
    if [[ $FAKE_GH_MODE == title-moved ]] && ((calls >= 2)); then
        printf '%s\n' 'fix(kanhe): a title edited while the gate ran'
    else
        printf '%s\n' 'fix(kanhe): harden workflow evidence'
    fi
elif [[ $1 == pr && $2 == view && $* == *"--json number"* ]]; then
    if [[ $FAKE_GH_MODE == invalid-number ]]; then
        printf '%s\n' 'not-a-number'
    else
        printf '%s\n' '42'
    fi
elif [[ $1 == pr && $2 == view && $* == *"--json changedFiles"* ]]; then
    # How many files the pull request changes. `empty-diff` is its own mode because an empty diff and a
    # clean one are the same to every other guard here — which is how a squash carrying no content was
    # merged with its message intact.
    case $FAKE_GH_MODE in
    empty-diff) printf '%s\n' '0' ;;
    unreadable-count) printf '%s\n' 'not-a-number' ;;
    *) printf '%s\n' '5' ;;
    esac
elif [[ $1 == pr && $2 == view && $* == *"--json statusCheckRollup"* ]]; then
    # **Raw JSON, and the wrapper's OWN `-q` filter applied to it.** This arm used to print the
    # already-transformed `<conclusion>\t<name>` lines, which meant the filter in `scripts/merge-pr.sh` was
    # executed by no direction at all: the stub stood exactly where it ran. A filter reading one of the
    # rollup's two node shapes was therefore invisible, and so would its repair have been.
    #
    # The filter is read out of this call's own argv rather than copied here, so it keeps one owner. Copying
    # it would be the two-places-that-must-agree shape a stub is especially bad at holding, since nothing
    # compares them.
    #
    # `StatusContext` is the second shape — an external commit status, carrying `.state`/`.context` and
    # neither `.conclusion` nor `.name`. `ci-unclaimed` answers an empty array, which is distinct from the
    # clean mode: a pull request with no checks and one whose checks all passed were byte-identical under the
    # printed form, which is how the third state came to be unreachable.
    filter=""
    for ((i = 1; i <= $#; i++)); do
        if [[ ${!i} == -q ]]; then
            j=$((i + 1))
            filter=${!j}
        fi
    done
    case $FAKE_GH_MODE in
    ci-red) body='{"statusCheckRollup":[{"conclusion":"FAILURE","name":"MSRV (rust-version)"},{"conclusion":"SUCCESS","name":"Definition of Done"}]}' ;;
    ci-pending) body='{"statusCheckRollup":[{"conclusion":null,"name":"MSRV (rust-version)"},{"conclusion":"SUCCESS","name":"Definition of Done"}]}' ;;
    ci-unclaimed) body='{"statusCheckRollup":[]}' ;;
    # A FAILED external commit status: the second node shape, with none of a CheckRun's fields.
    ci-red-status) body='{"statusCheckRollup":[{"state":"FAILURE","context":"continuous-integration/legacy"},{"conclusion":"SUCCESS","name":"Definition of Done"}]}' ;;
    # And one still expected: required, never posted. Agreement would merge past it.
    ci-expected-status) body='{"statusCheckRollup":[{"state":"EXPECTED","context":"continuous-integration/legacy"},{"conclusion":"SUCCESS","name":"Definition of Done"}]}' ;;
    # Two checks that finished and said nothing. Both conclusions sat beside `SUCCESS` with no measurement.
    ci-no-evidence) body='{"statusCheckRollup":[{"conclusion":"SKIPPED","name":"Examples dogfood"},{"conclusion":"NEUTRAL","name":"Supply chain (cargo-deny)"},{"conclusion":"SUCCESS","name":"Definition of Done"}]}' ;;
    # **The default is what a green suite looks like, so every conclusion in it must be one.** It carried a
    # `SKIPPED` beside a `SUCCESS`, which made every success-path direction in this file assert — silently,
    # as a premise nobody had written down — that a skipped check is agreement. Measured when that premise
    # was withdrawn: four directions failed at once, none of them about CI, each having reached its own
    # subject only because this fixture agreed on the way past.
    *) body='{"statusCheckRollup":[{"conclusion":"SUCCESS","name":"Definition of Done"},{"conclusion":"SUCCESS","name":"Examples dogfood"}]}' ;;
    esac
    printf '%s' "$body" | jq -r "$filter"
elif [[ $1 == pr && $2 == view && $* == *"--json headRefOid"* ]]; then
    if [[ $FAKE_GH_MODE == unreadable-head ]]; then
        printf '%s\n' ''
    else
        printf '%s\n' '81c9ef062fafee9fafe2ebfaacf68288f5554747'
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
    subjects | invalid-number | unreadable-head | unreadable-body | body-moved | title-moved | clean | no-verdict | ci-red | ci-red-status | ci-expected-status | ci-no-evidence | ci-pending | ci-unclaimed | empty-diff | unreadable-count)
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
    # Resolve the body the way the real tool does — `--body` by value, `--body-file` by reading the path —
    # and record what would be written. The question a direction must be able to ask is what the merge
    # RECORDS, not which flag was spelled: asserting the flag name would pass for a wrapper spelling
    # `--body "$(cat "$body_file")"` at merge time, which re-reads the file and is the defect itself.
    merge_body=""
    while (($#)); do
        case $1 in
        --body)
            merge_body=${2-}
            shift $(($# >= 2 ? 2 : 1))
            ;;
        --body-file)
            merge_body=$(cat -- "${2-}")
            shift $(($# >= 2 ? 2 : 1))
            ;;
        *)
            shift
            ;;
        esac
    done
    printf '%s' "$merge_body" > "$FAKE_GH_BODY"
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
# The gate's own interval, made reproducible. This executable stands exactly where the judgement runs, so a
# rewrite here lands between the read and the merge — which is when an editor autosave or a "quick typo fix"
# started after the wrapper was launched actually arrives.
if [[ -n ${FAKE_BODY_REWRITE-} ]]; then
    printf '%s' "$FAKE_BODY_REWRITE_TEXT" > "$FAKE_BODY_REWRITE"
fi
# The gate reports on the channel whether it agrees or refuses, so a controlled gate that only prints
# `1 passed` is a gate that ran and judged nothing — which is what the wrapper's success path now refuses.
# `no-verdict` is the mode that keeps that state constructible.
#
# Only where the channel was opened: this executable also stands in for the tool the wrapper `exec`s, and the
# wrapper hands the channel to the gate alone — so an unguarded write would both fail under `set -u` on that
# second invocation and recreate the file the wrapper removed one statement earlier.
if [[ ${FAKE_GATE_VERDICT-} != none && -n ${TIANHENG_GATE_VERDICT-} ]]; then
    printf '%s' 'Clean' > "$TIANHENG_GATE_VERDICT"
fi
printf '%s\n' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
"##,
    );

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin.display(), old_path.to_string_lossy());
    let mut command = Command::new("bash");
    command
        .arg(root.join("scripts/merge-pr.sh"))
        .args(["42", "--body-file"])
        .arg(&body)
        .args(extra)
        .env("PATH", path)
        .env("FAKE_GH_MODE", mode)
        .env("FAKE_GH_LOG", &gh_log)
        .env("FAKE_GH_BODY", &gh_body)
        .env("FAKE_CARGO_LOG", &cargo_log)
        .env("FAKE_COMMITS", &commits)
        .env("FAKE_TITLE_CALLS", scratch.join("title-calls"))
        .env("TMPDIR", &tmp);
    if let Some(cwd) = cwd {
        command.current_dir(cwd);
    }
    if mode == "no-verdict" {
        command.env("FAKE_GATE_VERDICT", "none");
    }
    if mode == "body-moved" {
        command
            .env("FAKE_BODY_REWRITE", &body)
            .env("FAKE_BODY_REWRITE_TEXT", REWRITTEN_BODY);
    }
    let output = command.output().expect("run controlled merge workflow");

    let run = Run {
        leftover: std::fs::read_dir(&tmp)
            .expect("read the wrapper's temporary directory")
            .map(|entry| {
                entry
                    .expect("a temporary directory entry")
                    .file_name()
                    .to_string_lossy()
                    .into_owned()
            })
            .collect(),
        status: output.status,
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        gh_log: read_if_present(&gh_log).expect("read controlled gh log"),
        gh_body: read_if_present(&gh_body)
            .expect("read the body the controlled merge would record"),
        cargo_log: read_if_present(&cargo_log).expect("read controlled cargo log"),
        commits: read_if_present(&commits).expect("read commits received by controlled gate"),
    };
    let _ = std::fs::remove_dir_all(&scratch);
    run
}

/// A gate from one repository never judges another repository's pull request.
///
/// The wrapper loads its gate from its own tree and resolves every input from the working directory. Run the
/// way its own refusals say to run it those are one tree, and its `--repo` refusal enumerated them as one set
/// — while nothing held them together. Invoked by absolute path from another checkout they come apart in
/// silence, and the wrapper would apply this repository's law to a stranger's pull request and then merge it.
///
/// Asserted on the two logs as much as on the exit class: the refusal must land before any evidence is read
/// and before the gate runs, because a wrapper that discovers this after `gh pr view` has already asked the
/// wrong repository a question.
#[test]
fn a_pull_request_from_another_worktree_is_refused_before_any_evidence_is_read() {
    let Some(root) = workspace_root() else {
        return;
    };
    let elsewhere = std::env::temp_dir().join(format!(
        "tianheng-merge-workflow-elsewhere-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&elsewhere);
    xingbiao::claim_scratch(&elsewhere).expect("create an unrelated worktree");
    let init = Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(&elsewhere)
        .output()
        .expect("run git init");
    assert!(
        init.status.success(),
        "the unrelated worktree is a git repository"
    );

    let run = run_wrapper_in(&root, "subjects", &[], Some(&elsewhere));
    let _ = std::fs::remove_dir_all(&elsewhere);

    assert_eq!(
        run.status.code(),
        Some(2),
        "a wrapper that cannot say whose law applies has not judged anything, so it owes the cannot-judge \
         class rather than the one that means a gate ran and refused: {}",
        run.stderr
    );
    assert!(
        run.stderr.contains("is one repository's law"),
        "the refusal must say which two trees it found, so an operator can see the mismatch rather than \
         guess at it: {}",
        run.stderr
    );
    assert!(
        run.gh_log.trim().is_empty(),
        "the wrapper asked the wrong repository a question before refusing: {}",
        run.gh_log
    );
    assert!(
        run.cargo_log.trim().is_empty(),
        "the gate ran before the wrapper knew whose pull request it was judging: {}",
        run.cargo_log
    );
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

/// An input this wrapper could not read stopped it, in the class that says so.
///
/// **This asserted only `!success()` and was blind to the class.** Five could-not-read conditions were split
/// across both exit classes with no rule, and two of them are facts `merge_message_gate::judge` types as
/// cannot-judge — so the wrapper reported a disagreement its own gate calls unjudgeable, and every direction
/// covering those sites passed. A helper that cannot see `1` from `2` is why the split survived being written
/// five times.
fn assert_stopped_before_gate_and_merge(run: &Run) {
    assert_eq!(
        run.status.code(),
        Some(2),
        "an input this wrapper could not read is the unjudged class, not a gate's disagreement; stderr was {:?}",
        run.stderr
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

/// Every **usage** refusal carries this wrapper's own diagnostic form, and none is a bare `usage; exit 2`.
///
/// **Two of them were exactly that**, printing none of the `merge message:` prefix every other refusal in the
/// file carries — so an operator, or a log filter keyed on that prefix, lost the two refusals that fire on the
/// commonest misinvocations: no pull request at all, and no body file. Neither had a direction of any kind,
/// which is why the flag-value refusal above was converged and these two were not: a defect nothing observes
/// is repaired only when someone happens to read the file.
///
/// **A table rather than three cases, so the description IS the run.** A stop added to the wrapper later lands
/// here as a row instead of as a finding in the next review — which is what `AGENTS.md` asks for in place of a
/// comment enumerating what a check decides.
///
/// The URL selector is deliberately **not** a row: it has its own direction below, which asserts the same
/// pair over a claim this table does not make. A second assertion of one fact is the shape this repository
/// removes rather than the coverage it reads as.
///
/// No controlled `PATH`: every row refuses before the wrapper reaches `gh` or its gate. It is **not** true
/// that they refuse before any process runs — the body-file row is decided after the worktree comparison, so
/// it spawns `git` and requires this process's directory to sit inside the repository, which
/// `workspace_root` above has already established. Said exactly, because a claim one step wider is what the
/// wrapper under test is being repaired for.
#[test]
fn every_usage_refusal_carries_the_wrappers_own_diagnostic() {
    let Some(root) = workspace_root() else {
        return;
    };
    // The arguments, and a phrase the refusal must name so a row cannot pass on the prefix alone.
    let shapes: [(&[&str], &str); 3] = [
        (&[], "first positional argument"),
        (&["--subject"], "not a flag"),
        (&["42"], "--body-file"),
    ];
    // **No vacuity guard, and the difference from its neighbours is the reason.** The sweeps in this crate
    // guard one because their corpus is *scanned* — a rename takes the last subject out of reach and the
    // direction reports clean over nothing. This table is a literal whose length is in its own type, so an
    // empty one is something a person wrote rather than a side effect of touching something else, and
    // `clippy::const_is_empty` refuses the assertion as a constant expression besides.
    for (args, phrase) in shapes {
        let output = Command::new("bash")
            .arg(root.join("scripts/merge-pr.sh"))
            .args(args)
            .output()
            .expect("run the wrapper on a misconfigured invocation");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(2),
            "{args:?} is a misconfigured invocation, which is the unjudged class rather than a gate's \
             disagreement; got {:?} with stderr {stderr:?}",
            output.status.code()
        );
        assert!(
            stderr.contains("merge message:"),
            "{args:?} must refuse in this wrapper's own diagnostic form rather than a bare `usage; exit 2`, \
             got {stderr:?}"
        );
        assert!(
            stderr.contains("usage:"),
            "{args:?} must show the usage the operator needs, got {stderr:?}"
        );
        assert!(
            stderr.contains(phrase),
            "{args:?} must say which input is missing — expected the refusal to name {phrase:?}, got \
             {stderr:?}"
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
        // Flags gh accepts here and does NOT honour as a merge of the judged evidence: one defers the merge
        // past the commit set the gate read, the other is not a merge at all. Both pass gh's own argument
        // validation — measured against a pull-request number that does not exist, so nothing could merge.
        "--auto",
        "--disable-auto",
        // Supplied by the wrapper itself now, so a caller's would replace the head the gate read from.
        "--match-head-commit",
        "--match-head-commit=abc123",
        // A **post-merge** act, and the one admitted argument that had an irreversible side effect. It shared
        // the `--admin` arm with no sentence of its own while the criterion beside it admits only flags that
        // change whether the merge proceeds.
        "--delete-branch",
        "-d",
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

    // A refusal an operator cannot act on is one they work around, and this is the refusal most likely to be
    // met by someone who just wanted their branch tidied. It must name the consequence rather than the rule —
    // the catch-all would already produce exit 2, so without this the arm carries no more than deleting it
    // would.
    let deletion = run_wrapper(&root, "subjects", &["--delete-branch"]);
    assert!(
        deletion.stderr.contains("auto-closes"),
        "the refusal must name what deleting a branch does to a pull request stacked on it, so an operator \
         can tell when it is safe to do by hand: {:?}",
        deletion.stderr
    );

    // `gh`'s own short spelling for the same flag must carry the same consequence, not just the same exit
    // code: every other admitted-consequence flag family catches its short form in the same arm (`-t*` with
    // `--subject`, `-F*`/`-b*` with `--body-file`/`--body`, and so on), and `-d` had been the one exception,
    // falling through to the generic catch-all instead.
    let short_deletion = run_wrapper(&root, "subjects", &["-d"]);
    assert!(
        short_deletion.stderr.contains("auto-closes"),
        "`-d` is gh's own short spelling of `--delete-branch` and must name the same consequence, not just \
         be refused generically: {:?}",
        short_deletion.stderr
    );

    // The other half: EVERY flag that changes whether the merge may proceed — never what it records, and never
    // when it happens relative to the evidence — still arrives. Without this the assertions above are satisfied
    // by a wrapper that refuses its own arguments.
    for admitted in [vec!["--admin"]] {
        let forwarded = run_wrapper(&root, "subjects", &admitted);
        assert!(
            forwarded.status.success(),
            "{admitted:?} must not be refused, got {:?} with stderr {:?}",
            forwarded.status.code(),
            forwarded.stderr
        );
        let merge = forwarded
            .gh_log
            .lines()
            .find(|line| line.starts_with("pr merge"))
            .unwrap_or_else(|| panic!("the merge must be reached:\n{}", forwarded.gh_log));
        for token in &admitted {
            assert!(
                merge.contains(token),
                "the admitted flag `{token}` must reach the merge, got {merge}"
            );
        }
    }
}

/// The merge is pinned to the head the gate read its evidence from.
///
/// The race this closes: the gate judges the body against the pull request's commit subjects as they are while it
/// runs, and the merge happens afterwards. A commit pushed in between changes the set the body must equal, and
/// before this nothing noticed — `gh pr merge` would record the approved body over a commit set that had moved.
///
/// **The head is read BEFORE the commit set, and the order is the guard.** Captured first, a push in between
/// leaves the commit set ahead of the pinned head and gh refuses: fails closed. Captured after, the pinned head
/// would include the new commit while the gate judged the older set, so the merge would proceed and record a body
/// missing it: fails open. This direction asserts the order in the log, not only the presence of the flag.
#[test]
fn the_merge_is_pinned_to_the_head_the_gate_read() {
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
    let merge = run
        .gh_log
        .lines()
        .find(|line| line.starts_with("pr merge"))
        .unwrap_or_else(|| panic!("the merge must be reached:\n{}", run.gh_log));
    assert!(
        merge.contains("--match-head-commit 81c9ef062fafee9fafe2ebfaacf68288f5554747"),
        "the merge must pin the head the evidence came from, got {merge}"
    );

    let head_at = run
        .gh_log
        .lines()
        .position(|line| line.contains("--json headRefOid"))
        .expect("the head must be read");
    let commits_at = run
        .gh_log
        .lines()
        .position(|line| line.starts_with("api "))
        .expect("the commit set must be read");
    assert!(
        head_at < commits_at,
        "the head must be read BEFORE the commit set, or the pin fails open; gh log was:\n{}",
        run.gh_log
    );
}

/// A head that cannot be read stops before the gate and the merge.
///
/// An unreadable head is not a head that has not moved. Merging unpinned because the pin could not be built is
/// the vacuity direction every check here owes a refusal to — and this one stands in front of a record that
/// cannot be amended.
#[test]
fn an_unreadable_head_stops_before_the_gate_and_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "unreadable-head", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "an unreadable head must stop the wrapper; got {:?} with stderr {:?}",
        run.status.code(),
        run.stderr
    );
    assert!(
        run.stderr.contains("could not be pinned"),
        "the refusal must say the merge could not be pinned, got {:?}",
        run.stderr
    );
    assert!(
        run.cargo_log.is_empty() && !run.gh_log.lines().any(|line| line.starts_with("pr merge")),
        "neither the gate nor the merge may be reached:\ngh:\n{}\ncargo:\n{}",
        run.gh_log,
        run.cargo_log
    );
}

/// A body file that cannot be READ is unjudgeable, not a body that disagrees.
///
/// `-f` says a regular file is there; it does not say this process may read it. The read used to sit inside the
/// gate's own invocation as `TIANHENG_MERGE_BODY=$(cat …)`, unguarded, so an unreadable file made the variable
/// empty — and the gate refuses an empty body as a violation, *the squash body is empty*. The operator was told
/// they had written the record wrongly about a file nobody could open.
#[test]
fn an_unreadable_body_file_is_unjudgeable_rather_than_an_empty_body() {
    let Some(root) = workspace_root() else {
        return;
    };
    // Whether an unreadable file can be produced at all is asked of a probe of this direction's own, NEVER of
    // the wrapper's behaviour. Skipping on `run.status.success()` was the first draft and it swallowed the
    // defect exactly: a wrapper that merged with an empty body succeeds, so the escape hatch read the failure
    // as "this user ignores the mode" and the direction passed. Measured — reverting the guard produced no
    // failure at all.
    if !mode_is_enforced() {
        return;
    }
    let run = run_wrapper(&root, "unreadable-body", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a body file this wrapper could not read is the unjudged class, not a gate's disagreement; stderr was \
         {:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("cannot read the body file")
            && run
                .stderr
                .contains("not the same fact as a body that disagrees"),
        "the refusal must name the read it could not make, got {:?}",
        run.stderr
    );
    assert!(
        run.cargo_log.is_empty(),
        "the gate must not be asked to judge a body that was never read:\n{}",
        run.cargo_log
    );
}

/// Whether a `0o000` file is unreadable to this process — root, and some filesystems, ignore the mode.
///
/// Asked of a file this function makes and removes, so the answer cannot come from the subject under test.
fn mode_is_enforced() -> bool {
    use std::os::unix::fs::PermissionsExt;

    let probe = std::env::temp_dir().join(format!(
        "tianheng-mode-probe-{}-{}",
        std::process::id(),
        MODE_PROBE.fetch_add(1, Ordering::Relaxed)
    ));
    if std::fs::write(&probe, b"probe").is_err() {
        return false;
    }
    let mut permissions = match std::fs::metadata(&probe) {
        Ok(metadata) => metadata.permissions(),
        Err(_) => return false,
    };
    permissions.set_mode(0o000);
    if std::fs::set_permissions(&probe, permissions).is_err() {
        let _ = std::fs::remove_file(&probe);
        return false;
    }
    let enforced = std::fs::read_to_string(&probe).is_err();
    let _ = std::fs::remove_file(&probe);
    enforced
}

static MODE_PROBE: AtomicUsize = AtomicUsize::new(0);

/// The wrapper leaves no temporary file behind, on the path that completes the act as well as on the paths that
/// do not.
///
/// **The successful path was the one not cleaned.** Cleanup was left to `trap 'rm -f …' EXIT`, and an EXIT trap
/// does not run when `exec` replaces the shell image — measured, `bash -c 'trap "echo T" EXIT; exec true'` prints
/// nothing while the same script without `exec` prints `T`. So the trap fired on every path where nothing
/// happened and was skipped on the one path that merges: three successful runs left three empty files.
///
/// Asserted over the whole of an isolated `TMPDIR` rather than over one known name, so a temporary file added
/// later is covered without this direction being touched. Both halves, because removing the trap would satisfy
/// the successful path while reopening every failing one.
#[test]
fn no_temporary_file_survives_the_wrapper() {
    let Some(root) = workspace_root() else {
        return;
    };
    let completed = run_wrapper(&root, "subjects", &[]);
    assert!(
        completed.status.success(),
        "the controlled workflow must complete for this to be about the successful path:\n{}",
        completed.stderr
    );
    assert!(
        completed.leftover.is_empty(),
        "the path that completes the merge left {:?} behind — an `exec` never reaches an EXIT trap",
        completed.leftover
    );

    let refused = run_wrapper(&root, "empty", &[]);
    assert!(
        !refused.status.success(),
        "the refusing run must refuse for this half to be about a failure path"
    );
    assert!(
        refused.leftover.is_empty(),
        "a path that stops before the merge left {:?} behind",
        refused.leftover
    );
}

/// What the gate judged is what the merge records, even when the file it was read from moves in between.
///
/// The interval is real rather than theoretical: between the read and the merge sits a whole `cargo test` run,
/// minutes of it on a cold target directory. Anything that rewrites the file in that window — an editor
/// autosave, a "quick typo fix" started after the wrapper was launched — was judged by nothing and is recorded
/// permanently, because a squash commit's hash is cited by the pull request's merge record and amending it
/// decouples the two.
///
/// **The assertion is on the recorded body, never on which flag was spelled.** A direction checking for
/// `--body` would pass for a wrapper spelling `--body "$(cat "$body_file")"` at merge time, which re-reads the
/// file and is the defect wearing the fix's clothes. So the controlled `gh` resolves its body the way the real
/// tool does and records what would be written, and this reads that.
///
/// Three of the four judged inputs already held this property — the subject travels as a value, the repository
/// is resolved once, the head is pinned with `--match-head-commit` and the commit set through it — and nothing
/// said they were one set, which is how the fourth sat here through the rounds that built this wrapper.
#[test]
fn the_merge_records_the_body_the_gate_judged_not_the_file_it_came_from() {
    let Some(root) = workspace_root() else {
        return;
    };

    let run = run_wrapper(&root, "body-moved", &[]);
    assert!(
        run.status.success(),
        "the controlled workflow must reach the merge, got {:?} with stderr {:?}",
        run.status.code(),
        run.stderr
    );

    // Not vacuous: the rewrite has to have actually happened, or this direction would hold over a file nobody
    // touched and pass for either wrapper.
    let after = std::fs::read_to_string(root.join("scripts/merge-pr.sh"));
    assert!(
        after.is_ok(),
        "the wrapper must still be readable after the run"
    );
    assert_ne!(
        judged_value(),
        REWRITTEN_BODY.trim_end_matches('\n'),
        "the judged and rewritten bodies must differ, or this direction compares a value with itself"
    );

    assert_eq!(
        run.gh_body,
        judged_value(),
        "the merge recorded a body the gate never judged.\n  recorded: {:?}\n  judged:   {:?}\nThe wrapper \
         must hand the tool the value it gave the gate, never the path it read that value from — the file can \
         move while the gate runs, and the record cannot be amended afterwards",
        run.gh_body,
        judged_value()
    );
}

/// A title edited while the gate ran stops the wrapper, as a cannot-judge.
///
/// **The guard this exercises was shipped without one.** The wrapper judges three inputs and pins two of them
/// by construction — the body travels as the value the gate judged, the commit set through
/// `--match-head-commit` — and the third was captured once. Re-reading it closed that, and the controlled
/// `gh` answered the same string on every call, so no direction could tell the guard from its absence.
///
/// The class is a **cannot-judge**, not a disagreement: the gate did not find the subject wrong, it found it
/// right against a title that no longer exists, so what the wrapper holds is a verdict about a vanished
/// input. Asserting the code and not merely the failure is what separates the two.
///
/// Negative run: with the post-gate re-read removed, the wrapper reaches `pr merge` and exits 0.
#[test]
fn a_title_edited_while_the_gate_ran_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "title-moved", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a moved title is a cannot-judge, not a disagreement; got {:?} with stderr {:?}",
        run.status.code(),
        run.stderr
    );
    assert!(
        run.stderr.contains("harden workflow evidence")
            && run.stderr.contains("a title edited while the gate ran"),
        "the refusal must name both titles so an operator can see what moved, got {:?}",
        run.stderr
    );
    assert!(
        !run.gh_log.lines().any(|line| line.starts_with("pr merge")),
        "the merge must not be reached, got {:?}",
        run.gh_log
    );
}

/// The control: an unchanged title still reaches the merge.
///
/// Without it the direction above is satisfied by a wrapper that refuses every run, and the re-read would be
/// indistinguishable from a stop-everything guard.
#[test]
fn an_unchanged_title_still_reaches_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "clean", &[]);
    assert_eq!(
        run.status.code(),
        Some(0),
        "an unchanged title must not be refused; stderr {:?}",
        run.stderr
    );
    assert!(
        run.gh_log.lines().any(|line| line.starts_with("pr merge")),
        "the merge must still be reached, got {:?}",
        run.gh_log
    );
}

/// A gate that ran, passed, and judged nothing does not reach the merge.
///
/// **`require_one_pass` cannot see this state, and that is the point.** It asks *did the selected test pass*
/// — which a harness returning without a verdict satisfies, and one did: a subject supplied as bytes the gate
/// could not read printed "not judged" and returned, so `1 passed` was true and nothing had been judged. The
/// A pull request whose checks disagree stops before the merge.
///
/// **This wrapper merged nineteen consecutive red runs.** Every local gate reported green each time, because
/// the Definition of Done is the LOCAL pre-flight list and CI runs a superset of it — one job in that
/// superset, the MSRV build, is not in the local list because it installs a toolchain and rebuilds the
/// workspace. A single let-chain the default toolchain accepts and 1.85 refuses was red there and green here,
/// for nineteen merges, until the job was run by hand. So the wrapper reads CI's answer the way it already
/// reads its own gate's: as a verdict rather than an inference.
#[test]
fn a_pull_request_whose_checks_disagree_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-red", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a suite this wrapper could not get agreement from is a cannot-judge, not a gate that refused: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge, which is the act that cannot be repaired: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("MSRV (rust-version)"),
        "the operator is told WHICH check disagreed, not merely that one did: {}",
        run.stderr
    );
}

/// A FAILED external commit status disagrees; it is not an unfinished check.
///
/// **The rollup is a union and the filter read one half of it.** `StatusCheckRollupContext` is
/// `CheckRun | StatusContext`, and a `StatusContext` carries `.state`/`.context` with neither `.conclusion`
/// nor `.name` — so `(.conclusion // "")` answered `""` for every commit status and a FAILED one was
/// classified as *unfinished*, reported as `these checks have not finished: ?`. A refusal naming a check that
/// does not exist is verbatim the defect the filter's own paragraph records fixing once, returned through the
/// node shape it never read.
///
/// Fail-closed either way, which is why it was latent: both classes refuse. What moved is what the operator
/// is told, and `?` sends them looking for nothing.
///
/// This direction is only possible because the stub now applies the wrapper's own `-q` filter to raw JSON
/// instead of printing what the filter would have produced. Under the printed form the filter was executed
/// nowhere and this shape was unreachable.
#[test]
fn a_failed_commit_status_disagrees_rather_than_reading_as_unfinished() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-red-status", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a suite this wrapper could not get agreement from is a cannot-judge: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("continuous-integration/legacy"),
        "the operator is told which context disagreed, by the name the status actually carries: {}",
        run.stderr
    );
    assert!(
        !run.stderr.contains("have not finished"),
        "a status that FAILED is a disagreement, not a check still running — and `?` for its name is what \
         sends an operator looking for a check that does not exist: {}",
        run.stderr
    );
}

/// A required commit status that was never posted is unfinished, not agreement.
///
/// `EXPECTED` is GitHub's *a status is expected*: required, and not yet reported. Reading it as agreement
/// would merge past a required status that never arrived, which is the false-negative direction this guard
/// exists to close — so it is classified with `PENDING`, whose operator action is identical. Recorded because
/// the review that found the union proposed the opposite mapping.
#[test]
fn a_commit_status_still_expected_is_unfinished_rather_than_agreement() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-expected-status", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a required status that never arrived is not agreement: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("have not finished"),
        "an expected-but-unposted status is reported as unfinished, so the operator waits rather than \
         hunting a disagreement: {}",
        run.stderr
    );
}

/// `--admin` does not carry a red suite past this wrapper.
///
/// **The admitted flag and the CI guard had never been observed together.** `--admin` was reasoned in as
/// consistent with *whether CI is green stays a human's call*, and `require_ci_green` landed 204 commits
/// later and refuses unconditionally — so the arm's premise was false and nothing said so, because the one
/// direction covering `--admin` proves it reaches the merge against a GREEN stub. What `--admin` still does
/// is bypass required REVIEWS, which a single-steward repository genuinely needs; what it does not do is
/// reach `gh` with a red rollup behind it.
///
/// **This pins the contract, not a change, and says so rather than claiming a negative run.** The behaviour
/// it asserts held before this change too — `require_ci_green` already refused unconditionally. What was
/// missing was any direction that looked, so the arm's prose could go on describing a premise its own
/// repository had falsified. Closing an observation gap and moving behaviour are different acts; only the
/// second has a run that fails beforehand, and reporting the first as the second is how a restatement gets
/// mistaken for a guard.
#[test]
fn the_admin_flag_does_not_carry_a_red_suite_past_this_wrapper() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-red", &["--admin"]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "an admitted flag does not change what the CI guard found: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "`--admin` bypasses required reviews on GitHub's side; it must not carry a red suite past a guard \
         that runs before `gh` is reached: {}",
        run.gh_log
    );
}

/// A pull request that changes no file stops before the merge.
///
/// **This wrapper merged one.** The content was committed onto the release branch itself while the branch
/// the pull request named still pointed at an already-merged commit, so its diff was empty and every guard
/// was satisfied: the live commit set was non-empty, CI was green because nothing had changed, and the
/// squash recorded a message asserting seven repairs across five files while carrying none of them.
#[test]
fn a_pull_request_that_changes_no_file_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "empty-diff", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a record about work that is not in the pull request is a cannot-judge: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge, which is the act that cannot be repaired: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("changes no file"),
        "the operator is told what is missing and where to look: {}",
        run.stderr
    );
}

/// A changed-file count this wrapper cannot read is not a count of zero, and not a count of some either.
#[test]
fn an_unreadable_changed_file_count_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "unreadable-count", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "an unreadable count is a cannot-judge: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge: {}",
        run.gh_log
    );
    // **The message is asserted because the two refusals collapse without it.** `(( changed == 0 ))` is
    // *true* for `not-a-number` — bash arithmetic resolves an unset identifier to zero — so deleting the
    // shape guard leaves this direction green while the wrapper tells the operator the pull request changes
    // no file, which is the other refusal and a different fact.
    assert!(
        run.stderr.contains("not a number"),
        "a count this wrapper cannot read is reported as unreadable, not as a count of zero: {}",
        run.stderr
    );
}

/// A pull request no workflow has claimed stops before the merge.
///
/// **The third state, which the first form could not reach.** It asked two independent filters about the
/// rollup, and a pull request with no checks at all is a value neither can produce — the disagreement filter
/// answers the empty string and the unfinished filter answers zero, so nothing refused and the merge ran.
/// The fake answers nothing here rather than reusing `clean`, because clean and no-checks were byte-identical
/// to that form and no direction could have told them apart.
#[test]
fn a_pull_request_no_workflow_has_claimed_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-unclaimed", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a pull request nothing has checked is not one that checked out: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("no workflow has claimed"),
        "the operator is told this head was never checked, not that a check disagreed: {}",
        run.stderr
    );
}

/// Every shape the reader has to decide, and the ones it must leave alone.
///
/// **The direction reads one real file, so nothing exercised the shapes that file does not have.** Its
/// documentation listed three ways a job could be lost and the repair closed two; the third — a job key at a
/// depth the reader did not expect — stayed open because it loses a **key** rather than a **name**, so the set
/// equality holds, nothing is carried, and the direction passes. A fixture is what asks the question the real
/// file cannot.
///
/// The rows that must NOT react carry as much as the rows that must: a `steps:` entry may legitimately carry
/// `if:` without the job's own conclusion moving, and a reader that refused it would refuse correct code —
/// which is the narrowing the direction argues for and would silently lose if the depth rule were widened
/// carelessly.
#[test]
fn the_workflow_reader_decides_every_shape_of_the_block() {
    let base = |keys: &str| {
        format!(
            "name: ci\n\non:\n  push:\n    branches: [main]\n\njobs:\n  alpha:\n{keys}\n  beta:\n    name: B\n    runs-on: x\n"
        )
    };

    // (label, document, jobs it must find, keys it must carry)
    let rows: [(&str, String, usize, usize); 10] = [
        ("clean", base("    name: A\n    runs-on: x\n"), 2, 0),
        (
            "if: at the file's own key depth",
            base("    name: A\n    if: x\n    runs-on: x\n"),
            2,
            1,
        ),
        // The shape that shipped unread. Legal YAML — indentation only has to be consistent within one
        // mapping — and the old reader found the job, held the equality, and examined no key.
        (
            "if: at a deeper key depth the whole job uses",
            base("      name: A\n      if: x\n      runs-on: x\n"),
            2,
            1,
        ),
        // A column-0 comment inside the block: the round-3 defect, kept as a row so it cannot come back.
        (
            "a column-0 comment does not end the block",
            "name: ci\n\njobs:\n  alpha:\n    name: A\n# --- divider ---\n  beta:\n    name: B\n    if: x\n"
                .to_string(),
            2,
            1,
        ),
        // Must NOT react: a step's own `if:` leaves the job's conclusion alone.
        (
            "if: on a step is not the job's",
            base("    name: A\n    steps:\n      - uses: x\n        if: y\n"),
            2,
            0,
        ),
        // A sequence item written at the key's own depth is still a sequence item.
        (
            "a sequence item at key depth is not a key",
            base("    name: A\n    steps:\n    - uses: x\n"),
            2,
            0,
        ),
        // The same defect on the block's other axis, and the row this fixture lacked when it was written:
        // the job **names** sit deeper than two. A reader assuming that width finds no job at all, and the
        // set equality then reports every job missing rather than the key it never examined — loud, but for
        // the wrong reason. Derived, it simply reads.
        // A path filter is a trigger condition and belongs under `on:`, where it does move whether a job
        // runs at all.
        (
            "paths: under on: reacts",
            "name: ci\n\non:\n  push:\n    paths:\n      - src/**\n\njobs:\n  alpha:\n    name: A\n"
                .to_string(),
            1,
            1,
        ),
        // Must NOT react: `paths` is an ordinary input name for several published actions, and a step input
        // moves no job's conclusion. Reading the key at any depth refused this.
        (
            "paths: as a step input does not react",
            base("    name: A\n    steps:\n      - uses: dorny/paths-filter@v3\n        with:\n          paths: src/**\n"),
            2,
            0,
        ),
        // The quoted spelling names the same block: YAML 1.1 reads a bare `on` as a boolean.
        (
            "paths: under a quoted on: still reacts",
            "name: ci\n\n\"on\":\n  push:\n    paths:\n      - src/**\n\njobs:\n  alpha:\n    name: A\n"
                .to_string(),
            1,
            1,
        ),
        (
            "the whole job block is indented deeper",
            "name: ci\n\njobs:\n    alpha:\n      name: A\n      if: x\n    beta:\n      name: B\n"
                .to_string(),
            2,
            1,
        ),
    ];

    for (label, document, jobs, keys) in rows {
        let shape = workflow_shape(&document);
        assert_eq!(
            shape.jobs.len(),
            jobs,
            "{label}: read {:?}, expected {jobs} job(s) — a reader that loses a job holds its set equality \
             over whatever it reached",
            shape.jobs
        );
        assert_eq!(
            shape.carried.len(),
            keys,
            "{label}: carried {:?}, expected {keys}",
            shape.carried
        );
    }
}

/// What a workflow's job block declares: the job names, and any key that lets a job skip.
///
/// Split from the direction so a fixture can hand it shapes the real file does not currently have — which is
/// the half the previous form lacked, and the reason it shipped blind to one of the three losses its own
/// documentation listed.
///
/// **Depths are read out of the file, not assumed.** The first form matched a job name at two spaces and a
/// job key at four. YAML fixes neither: indentation only has to be consistent within a mapping, so a job
/// whose keys sit at six is the same document. Measured — `pyyaml` parses it, and with `if:` among those
/// six-space keys the old reader found the job, held the set equality, examined no key, and passed. Binding
/// the width to a declared literal would have made that fail loudly, which is better than passing; deriving
/// it makes the question not arise, and it removes a literal rather than adding one.
///
/// So: the job-name depth is whatever the first structural line under `jobs:` sits at, and each job's key
/// depth is whatever its own first deeper non-sequence line sits at. A `-` opens a sequence item, so a
/// `steps:` entry written at the key's own depth is not read as a key — which is what keeps the step-level
/// narrowing the direction argues for.
struct WorkflowShape {
    jobs: BTreeSet<String>,
    carried: Vec<String>,
}

fn workflow_shape(text: &str) -> WorkflowShape {
    // Two key classes, each read at the position it can occupy. A path filter is a **trigger** condition
    // and lives under `on:`; the other three sit on a job. Reading the pair at any depth instead was
    // justified as *those two keys have no other meaning anywhere in it* — a claim about this file's current
    // content rather than about the keys, and the same kind of assumption this reader removed for both
    // indentation widths. Measured: a step input named `paths` — the shape `dorny/paths-filter` and
    // `tj-actions/changed-files` take — made the direction refuse, telling a maintainer that a job can now
    // legitimately skip about an input that moves no job's conclusion.
    const ON_THE_JOB: [&str; 3] = ["if:", "needs:", "continue-on-error:"];
    const ON_THE_WORKFLOW: [&str; 2] = ["paths:", "paths-ignore:"];

    let mut jobs = BTreeSet::new();
    let mut carried = Vec::new();
    let mut in_jobs = false;
    let mut in_on = false;
    let mut job_name_depth: Option<usize> = None;
    let mut key_depth: Option<usize> = None;
    let mut in_job = false;

    for (index, line) in text.lines().enumerate() {
        let trimmed = line.trim_start();
        // A blank line and a comment end nothing — only a real top-level key does.
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let depth = line.len() - trimmed.len();

        if depth == 0 {
            // YAML 1.1 reads a bare `on` as a boolean, so a workflow may quote the key; both spellings name
            // the same block. Taken off the key rather than matched as a prefix, so `once:` is not `on:`.
            let top = line
                .split_once(':')
                .map_or("", |(key, _)| key.trim().trim_matches(['"', '\'']));
            in_jobs = top == "jobs";
            in_on = top == "on";
            in_job = false;
            job_name_depth = None;
            continue;
        }
        if in_on {
            if let Some(key) = ON_THE_WORKFLOW
                .iter()
                .find(|key| trimmed.starts_with(**key))
            {
                carried.push(format!("  ci.yml:{}: {key}", index + 1));
            }
            continue;
        }
        if !in_jobs {
            continue;
        }

        let sequence = trimmed.starts_with('-');
        let names_depth = *job_name_depth.get_or_insert(depth);

        if depth == names_depth && !sequence && trimmed.ends_with(':') {
            jobs.insert(trimmed.trim_end_matches(':').to_string());
            key_depth = None;
            in_job = true;
            continue;
        }
        if !in_job || depth <= names_depth || sequence {
            continue;
        }
        let keys_depth = *key_depth.get_or_insert(depth);
        if depth != keys_depth {
            continue;
        }
        // Two statements rather than a `let` chain: chained `let` in an `if` condition is stable well past
        // this workspace's declared `rust-version`, and the local Definition of Done compiles on whatever
        // toolchain is installed. This is the shape `require_ci_green`'s own header records riding nineteen
        // merges green here and red in CI.
        if let Some(key) = ON_THE_JOB.iter().find(|key| trimmed.starts_with(**key)) {
            carried.push(format!("  ci.yml:{}: {key}", index + 1));
        }
    }

    WorkflowShape { jobs, carried }
}

/// The jobs `.github/workflows/ci.yml` declares, held against what the reader finds in both directions.
///
/// A literal beside an enumerator, which `AGENTS.md` admits exactly where something downstream filters on the
/// claim: *the literal is not a weakening here; it is what gives the enumerator something to disagree with*.
/// A new job lands here after someone has looked at whether it can skip — which is the question this whole
/// direction is about, asked at the one moment the answer is free.
const JOBS: [&str; 8] = [
    "dod",
    "examples",
    "license-files",
    "msrv",
    "packaged-selftest",
    "reaction",
    "release-coherence",
    "supply-chain",
];

/// The premise the no-evidence refusal rests on, held against the workflow that has to keep it true.
///
/// **The classification and the diagnostic both filter on this claim, and nothing held it.** The wrapper
/// tells an operator that a skip here *cannot* be legitimate — *no job in this repository's workflow carries
/// `if:`, `needs:`, `paths:` or `continue-on-error:`* — and refuses on that basis. Add one of those keys and
/// the sentence becomes false at the moment it is printed, while the refusal it justifies goes on happening:
/// a legitimate skip refused with a message asserting legitimate skips are impossible. `AGENTS.md` names the
/// rule this falls under — *something downstream filters on the claim → declare it, and hold it to the
/// producer both ways* — and the producer is a tracked file a sibling direction already reads.
///
/// **Job level, not the whole file, and the difference is not tidiness.** A `steps:` entry may carry `if:` or
/// `continue-on-error:` without the job's own conclusion moving: the step is skipped and the job still reports
/// success, so the rollup this wrapper reads is unaffected and refusing it would refuse correct code. What
/// moves a job to `SKIPPED` is a key on the job itself, or a workflow-level path filter — so those are what
/// this reads.
///
/// **The names are held against [`JOBS`] both ways**, which is the form `AGENTS.md` prescribes for a claim
/// something downstream filters on — *the literal is not a weakening here; it is what gives the enumerator
/// something to disagree with*. The first form asserted only that the reader had found *some* job, which
/// catches a read that found nothing and cannot catch one that found **fewer**.
///
/// **Three ways this reader could lose a job were named, two were closed, and the third took another round.**
/// The latch (a column-0 comment ending the block) and the equality landed together; the two indentation
/// assumptions did not, and one of them — a job key at a depth other than the assumed four — loses a **key**
/// rather than a **name**, so the equality holds, nothing is carried, and the direction passes. Measured, on
/// a document `pyyaml` accepts: with `if:` among a job's six-space keys the reader found the job, held the
/// equality, examined no key, and reported the premise intact. Both assumptions are now derived from the
/// document instead — see [`workflow_shape`] — which removes a literal rather than adding one, and
/// [`the_workflow_reader_decides_every_shape_of_the_block`] holds each shape including the two that must not
/// react.
///
/// What the equality is for is what remains after that: a loss nobody has thought of yet. It names which jobs
/// went missing, so a reader meets the shape rather than the absence.
#[test]
fn no_workflow_job_can_legitimately_skip() {
    let Some(root) = workspace_root() else {
        return;
    };
    let workflow = root.join(".github/workflows/ci.yml");
    let text = std::fs::read_to_string(&workflow)
        .expect("read .github/workflows/ci.yml, whose shape the no-evidence refusal asserts");

    let shape = workflow_shape(&text);
    let (found, carried) = (shape.jobs, shape.carried);

    let declared: BTreeSet<String> = JOBS.iter().map(|job| (*job).to_string()).collect();
    assert_eq!(
        found,
        declared,
        "the jobs read out of {} are not the jobs this direction is declared over, so it judged a corpus \
         other than the one it names — missing {:?}, unexpected {:?}. A read that loses jobs satisfies \
         `none of them carries a forbidden key` over whatever it happened to reach: add a new job to `JOBS` \
         after checking whether it can skip, or find what stopped the reader",
        workflow.display(),
        declared.difference(&found).collect::<Vec<_>>(),
        found.difference(&declared).collect::<Vec<_>>()
    );
    assert!(
        carried.is_empty(),
        "a job can now legitimately skip, and `require_ci_green` says otherwise to an operator's face: its \
         refusal reads \"no job in this repository's workflow carries `if:`, `needs:`, `paths:` or \
         `continue-on-error:`\", which is now false. Either move that conclusion back beside `SUCCESS` with \
         the measurement that earns it — which job, and why its skip is evidence — or drop the key:\n{}",
        carried.join("\n")
    );
}

/// A check that finished and produced no evidence is not a check that agreed.
///
/// **`NEUTRAL` and `SKIPPED` sat beside `SUCCESS` with no measurement**, while the `EXPECTED` classification
/// beside them was reasoned about at length: it is unfinished because *reading it as agreement would merge past a
/// required status that never arrived*. The identical argument covers a check that did not run — it produced
/// no evidence, so agreement merges past whatever it would have said — and nothing had applied it.
///
/// Measured on this repository's own workflow rather than argued from GitHub's vocabulary: no job in
/// `.github/workflows/ci.yml` carries `if:`, `needs:`, `paths:`, `paths-ignore:` or `continue-on-error:`, so
/// a skip here cannot mean *legitimately not applicable*. It can only mean the workflow changed or the run
/// was interfered with.
///
/// **The default fixture carried one**, which is how the premise stayed invisible: every success-path
/// direction in this file was asserting, without saying so, that a skipped check is agreement. Withdrawing it
/// failed four directions at once, none of them about CI.
///
/// Its own refusal rather than the unfinished one, because the operator action differs: an unfinished check
/// is waited for and a skipped one is investigated. The assertion reads the sentence, not just the class,
/// since both are cannot-judge and an exit code cannot tell them apart.
#[test]
fn a_check_that_produced_no_evidence_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-no-evidence", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a check that produced no evidence is the unjudged class, not a gate's disagreement; stderr was {:?}",
        run.stderr
    );
    assert!(
        run.stderr.contains("produced no evidence"),
        "the refusal must say the checks produced no evidence rather than that they disagreed or have not \
         finished, got {:?}",
        run.stderr
    );
    for named in ["Examples dogfood", "Supply chain (cargo-deny)"] {
        assert!(
            run.stderr.contains(named),
            "the refusal must name {named:?} so an operator can look at why it did not run, got {:?}",
            run.stderr
        );
    }
    assert!(
        !run.stderr.contains("have not finished"),
        "a skipped check is not an unfinished one — waiting is the wrong action and that sentence asks for \
         it, got {:?}",
        run.stderr
    );
    // The log carries every `gh` call, and this wrapper legitimately makes several before reading the
    // rollup — so the question is whether the **merge** was reached, not whether the tool was.
    assert!(
        !run.gh_log.contains("pr merge"),
        "`gh pr merge` must never be reached when a check produced no evidence, but it ran: {:?}",
        run.gh_log
    );
}

/// A pull request whose checks have not finished stops too, and for a different reason.
///
/// **Three states, and the middle one is why a boolean would not do.** A run still in flight is not a run
/// that failed; merging on *not success* would refuse a pull request nobody has answered yet, and merging on
/// *not failure* would merge one nobody has answered yet. The wrapper says which of the two it met.
#[test]
fn a_pull_request_whose_checks_have_not_finished_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "ci-pending", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "an unfinished suite is a cannot-judge: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("have not finished"),
        "an unfinished run is reported as unfinished rather than as a disagreement: {}",
        run.stderr
    );
}

/// gate now reports on the channel from its clean arm too, so *absent on success* means unjudged by
/// construction, and this reads it rather than inferring it from an exit status.
///
/// The two guards catch different states and both stay: a renamed test means nothing ran, this means
/// something ran and reached no verdict.
#[test]
fn a_gate_that_passes_without_judging_stops_before_the_merge() {
    let Some(root) = workspace_root() else {
        return;
    };
    let run = run_wrapper(&root, "no-verdict", &[]);
    assert_eq!(
        run.status.code(),
        Some(2),
        "a run that judged nothing belongs to the class this wrapper could not judge, never to the one \
         reserved for a gate that ran and refused: {}{}",
        run.stdout,
        run.stderr
    );
    assert!(
        !run.gh_log.contains("pr merge"),
        "and it must stop before the merge, which is the act that cannot be repaired: {}",
        run.gh_log
    );
    assert!(
        run.stderr.contains("without reaching a verdict"),
        "the operator is told which of the two it met, got: {}",
        run.stderr
    );
}
