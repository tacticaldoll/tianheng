//! Controlled execution of the sanctioned publish workflow's argument handling.
//!
//! The source verdict remains in `publish_source.rs`; what this file holds is what may reach `cargo publish`.
//! It replaces `cargo` on `PATH`, so no upload and no build can occur.
//!
//! **The refusal had no direction at all until this file.** `scripts/publish.sh` refused `--manifest-path`, both
//! spellings, and argued in its own comment that a guard catching one would be a guard catching neither — the
//! same sentence its sibling `scripts/merge-pr.sh` carried while three spellings walked past it. Nothing ran the
//! script to find out.

use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus};
use std::sync::atomic::{AtomicUsize, Ordering};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("scripts/publish.sh").is_file(),
        shengmo::workspace::marker_set(),
    )
}

struct Run {
    /// Anything the wrapper left in the isolated `TMPDIR` it was given.
    leftover: Vec<String>,
    status: ExitStatus,
    stderr: String,
    cargo_log: String,
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

/// Run the wrapper with `extra` appended, against a `cargo` that logs and never uploads.
fn run_wrapper(root: &Path, extra: &[&str]) -> Run {
    run_wrapper_with_env(root, extra, &[])
}

/// The same, with `env` set in the wrapper's own environment.
///
/// Separate because what the wrapper reads from its ENVIRONMENT is a different surface from what it reads from
/// its arguments, and this repository declares a bound about exactly that difference.
fn run_wrapper_with_env(root: &Path, extra: &[&str], env: &[(&str, &str)]) -> Run {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    let scratch = loop {
        let candidate = std::env::temp_dir().join(format!(
            "tianheng-publish-workflow-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match std::fs::create_dir(&candidate) {
            Ok(()) => break candidate,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => panic!(
                "cannot acquire controlled publish-workflow root {}: {err}",
                candidate.display()
            ),
        }
    };
    let bin = scratch.join("bin");
    std::fs::create_dir(&bin).expect("create controlled PATH");
    // The wrapper's own `TMPDIR`, so what it leaves behind is observable and lands in the fixture rather than in
    // the developer's `/tmp`.
    let tmp = scratch.join("tmp");
    std::fs::create_dir(&tmp).expect("create the wrapper's temporary directory");
    let cargo_log = scratch.join("cargo.log");

    // The gate's own invocation must appear to pass, so that a refusal reaching this far is the argument's and
    // not the gate's. The wrapper's `require_one_pass` reads the `test result: ok. 1 passed` line.
    write_executable(
        &bin.join("cargo"),
        r##"#!/usr/bin/env bash
set -eu
# ONE line per invocation, carrying the arguments AND the environment. Two lines let a direction match an
# environment value against the wrong invocation: the gate's own `cargo test` runs first and inherits the same
# environment, so a bare `contains` for the value passed while the publish had been scrubbed — measured, the
# negative run for the environment bound did not fire until this became one line.
printf '%s |env CARGO_BUILD_TARGET=[%s]\n' "$*" "${CARGO_BUILD_TARGET-}" >> "$FAKE_CARGO_LOG"
printf '%s\n' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out'
"##,
    );

    let old_path = std::env::var_os("PATH").unwrap_or_default();
    let path = format!("{}:{}", bin.display(), old_path.to_string_lossy());
    let mut command = Command::new("bash");
    command
        .arg(root.join("scripts/publish.sh"))
        .args(extra)
        .env("PATH", path)
        .env("FAKE_CARGO_LOG", &cargo_log)
        .env("TMPDIR", &tmp);
    for (name, value) in env {
        command.env(name, value);
    }
    let output = command.output().expect("run controlled publish workflow");

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
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        cargo_log: read_if_present(&cargo_log).expect("read controlled cargo log"),
    };
    let _ = std::fs::remove_dir_all(&scratch);
    run
}

/// The line the wrapper's final `exec` produces, if it got there.
fn publish_invocation(cargo_log: &str) -> Option<&str> {
    cargo_log
        .lines()
        .find(|line| line.starts_with("publish "))
        .and_then(|line| line.split_once(" |env").map(|(arguments, _)| arguments))
}

/// Only an allowlisted argument reaches `cargo publish`; everything else is refused before the gate runs.
///
/// **The property is the default, not the list below.** The refused set is a sample of a rule — what makes the
/// rule hold for an argument nobody classified is the catch-all, which is why the sample includes a flag no
/// cargo has. Each entry names a distinct reason the wrapper's own comment gives: the source tree, the set of
/// crates, what cargo verifies, what gets packaged, and `--config`, which can reach every other one.
///
/// Refused **before the gate**, so the sample also asserts the order: the cargo log must be empty, and the gate
/// is a `cargo test` invocation that would appear in it. A refusal printed after the gate had run would still
/// exit 2 while having spent the gate's verdict on an invocation it was going to reject.
#[test]
fn only_an_allowlisted_argument_reaches_the_publish() {
    let Some(root) = workspace_root() else {
        return;
    };

    for argument in [
        // The source tree the gate judged, both spellings.
        "--manifest-path",
        "--manifest-path=/elsewhere",
        // The set of crates, under the `--workspace` the script writes itself.
        "--exclude",
        "--exclude=xuanji",
        "--workspace",
        // What cargo verifies before uploading, and what gets packaged.
        "--no-verify",
        "--allow-dirty",
        "--all-features",
        "--no-default-features",
        "--features",
        "-F",
        "--target",
        // The one that can become any of the above, including by naming a whole configuration file.
        "--config",
        "--config=/elsewhere/config.toml",
        "-Zunstable-options",
        // Deprecated by cargo itself, so the refusal points where cargo does.
        "--token",
        // Short and glued spellings of arguments the script does forward in their long form.
        "-p",
        "-pxuanji",
        "-n",
        "-j2",
        "-v",
        // And an argument nobody classified: refused, not passed on.
        "--some-flag-a-future-cargo-adds",
    ] {
        let run = run_wrapper(&root, &[argument]);
        assert_eq!(
            run.status.code(),
            Some(2),
            "`{argument}` must be refused as a usage error; got {:?} with stderr {:?}",
            run.status.code(),
            run.stderr
        );
        assert!(
            run.stderr.contains("publish source: refusing"),
            "`{argument}` must be refused in this script's own diagnostic form, got {:?}",
            run.stderr
        );
        assert!(
            run.cargo_log.is_empty(),
            "`{argument}` must be refused before the gate runs, but cargo was invoked:\n{}",
            run.cargo_log
        );
    }
}

/// An admitted argument reaches the publish as the SELECTION cargo would honour, not as the string the wrapper
/// typed.
///
/// Without this direction the one above is satisfied by a wrapper that refuses everything, and the release flow
/// genuinely needs some of these — `--dry-run` is the rehearsal the discipline requires before any real upload,
/// and `--package` is how a partly completed publish resumes when crates.io has already accepted some of the six.
///
/// **This direction asserted the typed string, and pinned a flag cargo discards.** It expected `publish
/// --workspace --package xuanji` against a controlled `cargo` that only logs its arguments, so it could not see
/// that cargo maps that combination to *all packages*: measured on cargo 1.96.0 with the identical selection
/// flags, `--workspace --package xuanji` selects 8 and `--package xuanji` selects 1, with no warning. The
/// expectation was the defect, and its name — *the workspace is always named* — stated the reason the flag was
/// inert.
///
/// A controlled executable cannot answer what the real tool does; that measurement belongs beside the
/// classification, in the script's own comment, against a named version. What this direction can hold is that
/// the wrapper composes a selection cargo would honour, so the two cases below that carry `--package` assert the
/// absence of `--workspace` as much as the presence of the selector.
#[test]
fn an_admitted_argument_reaches_the_publish_as_cargo_would_honour_it() {
    let Some(root) = workspace_root() else {
        return;
    };

    for (extra, expected) in [
        (vec![], "publish --workspace"),
        (vec!["--dry-run"], "publish --workspace --dry-run"),
        // `--package` REPLACES the default selection. With `--workspace` beside it, cargo publishes everything.
        (vec!["--package", "xuanji"], "publish --package xuanji"),
        (
            vec!["--package", "xuanji", "--package", "xingbiao"],
            "publish --package xuanji --package xingbiao",
        ),
        (
            vec!["--package", "xuanji", "--dry-run"],
            "publish --package xuanji --dry-run",
        ),
        (
            vec!["--locked", "--offline"],
            "publish --workspace --locked --offline",
        ),
        (
            vec!["--registry", "crates-io"],
            "publish --workspace --registry crates-io",
        ),
    ] {
        let run = run_wrapper(&root, &extra);
        assert!(
            run.status.success(),
            "{extra:?} must not be refused; got {:?} with stderr {:?}",
            run.status.code(),
            run.stderr
        );
        assert_eq!(
            publish_invocation(&run.cargo_log),
            Some(expected),
            "{extra:?} must reach the publish unchanged; cargo log was:\n{}",
            run.cargo_log
        );
    }
}

/// A value-taking argument given no value is named and refused, rather than failing on the arithmetic.
///
/// `shift 2` with one argument left returns non-zero, and under `set -e` that becomes the exit — silently, with
/// no diagnostic, while every other refusal here prints one. Its sibling wrapper failed exactly that way before
/// it was measured, which is why the check is here before the shift rather than after it.
#[test]
fn a_value_taking_argument_with_no_value_is_named_and_refused() {
    let Some(root) = workspace_root() else {
        return;
    };
    for flag in [
        "--package",
        "--jobs",
        "--color",
        "--target-dir",
        "--registry",
        "--index",
    ] {
        let run = run_wrapper(&root, &[flag]);
        assert_eq!(
            run.status.code(),
            Some(2),
            "`{flag}` with no value must exit 2; got {:?} with stderr {:?}",
            run.status.code(),
            run.stderr
        );
        assert!(
            run.stderr.contains(&format!("refusing `{flag}`"))
                && run.stderr.contains("the argument after its flag"),
            "the refusal must name `{flag}` and say where its value goes, got {:?}",
            run.stderr
        );
        assert!(
            run.cargo_log.is_empty(),
            "`{flag}` must be refused before the gate runs:\n{}",
            run.cargo_log
        );
    }
}

/// `repository-checks`'s bound *a tool configuration set in the environment is not observed*, demonstrated.
///
/// `UnderReacts`, owned by the engine. The allowlist classifies ARGUMENTS, and cargo takes the same configuration
/// from the environment: measured on cargo 1.96.0, `--target not-a-real-triple` and
/// `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical `failed to run \`rustc\` to learn about
/// target-specific information`. So a value this wrapper refuses as an argument reaches cargo unexamined when it
/// is exported instead.
///
/// **Both directions on one configuration key**, because a bound whose silence is not contrasted with a reaction
/// is indistinguishable from a wrapper that refuses nothing. The argument is refused; the environment passes and
/// arrives.
///
/// Closing it is ordinary work here rather than another layer's — the wrapper could scrub the environment before
/// invoking cargo. It is not closed because doing so needs an allowlist over the environment, and legitimate
/// setups export `CARGO_HOME`, `CARGO_TARGET_DIR` and more; choosing that set is a decision this bound records
/// instead of guessing.
#[test]
fn a_tool_configuration_set_in_the_environment_is_a_stated_bound() {
    let Some(root) = workspace_root() else {
        return;
    };

    // The reaction: as an argument, refused before the gate.
    let refused = run_wrapper(&root, &["--target", "not-a-real-triple"]);
    assert_eq!(
        refused.status.code(),
        Some(2),
        "`--target` must be refused as an argument, or this bound has nothing to contrast with; stderr {:?}",
        refused.stderr
    );

    // The silence: the same configuration, exported, is neither seen nor refused — and it arrives.
    let passed = run_wrapper_with_env(&root, &[], &[("CARGO_BUILD_TARGET", "not-a-real-triple")]);
    assert!(
        passed.status.success(),
        "the bound says the environment is not observed; a refusal here would mean it is closed and this \
         declaration is stale: {:?}",
        passed.stderr
    );
    // Matched on the PUBLISH's own line, not anywhere in the log. The gate's `cargo test` runs first and
    // inherits the same environment, so a bare search passed even with the publish scrubbed — the negative run
    // for this bound did not fire until the harness recorded arguments and environment together.
    let publish = passed
        .cargo_log
        .lines()
        .find(|line| line.starts_with("publish "))
        .unwrap_or_else(|| panic!("the publish must be reached:\n{}", passed.cargo_log));
    assert!(
        publish.contains("|env CARGO_BUILD_TARGET=[not-a-real-triple]"),
        "the exported configuration must be shown to reach THE PUBLISH, not merely to go unrefused: {publish}"
    );
}

/// The wrapper leaves no temporary file behind, on the path that completes the act as well as on the paths that
/// do not.
///
/// The same defect as its sibling's, from the same line: cleanup left to an EXIT trap that `exec` never reaches,
/// so the publishing path was the one path not cleaned. Asserted over the whole of an isolated `TMPDIR` rather
/// than over one known name, so a temporary file added later is covered for free.
#[test]
fn no_temporary_file_survives_the_wrapper() {
    let Some(root) = workspace_root() else {
        return;
    };
    let completed = run_wrapper(&root, &[]);
    assert!(
        completed.status.success(),
        "the controlled workflow must complete for this to be about the successful path:\n{}",
        completed.stderr
    );
    assert!(
        completed.leftover.is_empty(),
        "the path that completes the publish left {:?} behind — an `exec` never reaches an EXIT trap",
        completed.leftover
    );

    let refused = run_wrapper(&root, &["--no-verify"]);
    assert_eq!(
        refused.status.code(),
        Some(2),
        "the refusing run must refuse for this half to be about a failure path"
    );
    assert!(
        refused.leftover.is_empty(),
        "a path that stops before the publish left {:?} behind",
        refused.leftover
    );
}
