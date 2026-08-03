use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn fixture_manifest(name: &str) -> Option<PathBuf> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("tests/fixtures/{name}/Cargo.toml"));
    if path.exists() {
        return Some(path);
    }
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "{name} fixture expected but absent while TIANHENG_WORKSPACE_TESTS is set"
    );
    None
}

fn temp_baseline(test: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "tianheng-{test}-{}-baseline.json",
        std::process::id()
    ))
}

fn command_for(manifest: &Path) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tianheng"));
    command.args([
        "check",
        "--manifest-path",
        manifest.to_str().expect("UTF-8 fixture path"),
    ]);
    command
}

fn run_with(manifest: &Path, flag: &str, baseline: &Path) -> Output {
    command_for(manifest)
        .args([flag, baseline.to_str().expect("UTF-8 baseline path")])
        .output()
        .expect("run tianheng CLI")
}

/// How many times [`write_baseline_colliding_with`] re-races before giving up. The parent normally
/// wins on the first attempt — the child has a whole `cargo metadata` read and scan to perform
/// before it opens its temp file — so this is headroom for a loaded runner, not an expected cost.
const TEMP_PLANT_ATTEMPTS: usize = 20;

/// Overwrite `baseline` through the CLI while `plant` puts something at the temp path that run will
/// predict, returning the output of the run whose temp-file open **actually collided** with it,
/// along with that temp path.
///
/// The plant unavoidably races the child: the temp name embeds the writing process's pid, so the
/// parent can only compute `<target>.tmp-<pid>` after `spawn`, and a loaded runner can let the child
/// open its own temp file first. A run where the plant lands too late does not exercise the
/// collision at all — and neither guard below can tell the difference from its own assertions. The
/// stale-temp guard fails spuriously (it demands exit 2 and sees a clean exit 0 — observed once in
/// CI, which is what prompted this helper), while the symlink guard passes VACUOUSLY: an untouched
/// victim and a non-symlinked baseline are exactly what an unexercised run also leaves behind, so it
/// would report a verdict it never earned. That is the failure mode `AGENTS.md` names — a guard is
/// not a guard until it has been seen to fail, and a guard that cannot tell whether it ran is worse
/// than none.
///
/// So the race is retried until the child reports the collision at the planted path, and if it never
/// does, this fails loud saying so. Either way no assertion below rests on a coin flip.
fn write_baseline_colliding_with(
    manifest: &Path,
    baseline: &Path,
    mut plant: impl FnMut(&str),
) -> (Output, String) {
    for attempt in 1..=TEMP_PLANT_ATTEMPTS {
        // The overwrite path needs an existing baseline. A previous attempt removed whatever it left
        // behind (see the reset below), so this reseeds a known-good one each time rather than
        // inheriting a document some earlier attempt may have corrupted.
        if !baseline.exists() {
            let seed = run_with(manifest, "--write-baseline", baseline);
            assert_eq!(seed.status.code(), Some(0), "{seed:?}");
        }
        let real_baseline = std::fs::canonicalize(baseline).expect("canonicalize baseline");
        let child = command_for(manifest)
            .args([
                "--write-baseline",
                baseline.to_str().expect("UTF-8 baseline path"),
            ])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("spawn tianheng CLI");
        let predicted_tmp = format!("{}.tmp-{}", real_baseline.display(), child.id());
        plant(&predicted_tmp);
        let output = child.wait_with_output().expect("wait for tianheng CLI");
        let stderr = String::from_utf8_lossy(&output.stderr);
        // The one signal that the planted path was really the path the write tried to open: the
        // refusal names it. `create_new`'s `O_EXCL` fails on any existing entry, a symlink included,
        // so this holds for every obstruction a caller plants.
        if output.status.code() == Some(2) && stderr.contains(&predicted_tmp) {
            return (output, predicted_tmp);
        }
        // The plant landed too late (or not at all). Reset BOTH the plant and the target before
        // retrying, because a plant that lands mid-window corrupts the baseline: the write stages its
        // document at the temp path and only then renames it into place, so a `fs::write` arriving
        // between those two steps replaces the staged bytes and the run renames the plant's own
        // content onto the target. The next attempt then meets an unparseable baseline and is refused
        // as unsupported — exit 2, but with a message that names the baseline rather than the temp
        // path, so the collision check below never matches and no later attempt can recover. Observed
        // exactly that way in CI, where this loop burned all its attempts against a poisoned target.
        // `remove_file` unlinks a symlink itself, never its target.
        let _ = std::fs::remove_file(&predicted_tmp);
        let _ = std::fs::remove_file(baseline);
        assert!(
            attempt < TEMP_PLANT_ATTEMPTS,
            "the temp-path plant never landed before the child opened its own temp file in \
             {TEMP_PLANT_ATTEMPTS} attempts, so the collision path was never exercised — this \
             guard must not report a verdict it did not earn (last run: {output:?})"
        );
    }
    unreachable!("the loop either returns an exercised run or fails loud on its last attempt")
}

fn wrong_typed_baseline() -> &'static str {
    r#"{"format":"tianheng.baseline/structured-facts","violations":[{
        "target":"example-core","rule":"deny external dependencies","finding":"serde",
        "rule_key":{"type":"tianheng.rule/guibiao/deny-external-dependencies","fields":{"allowed":"[]","dependency_kind":"normal"}},
        "fact":{"type":"tianheng.fact/guibiao/dependency","shape":"dependency-edge","fields":{"kind":"normal","package":"serde"}},
        "owner":["team-core"]
    }]}"#
}

#[test]
fn baseline_gate_rejects_wrong_typed_metadata_through_the_cli() {
    let Some(manifest) = fixture_manifest("violating") else {
        return;
    };
    let path = temp_baseline("invalid-gate");
    std::fs::write(&path, wrong_typed_baseline()).expect("write malformed baseline");

    let control = command_for(&manifest)
        .output()
        .expect("run unbaselined control");
    assert_eq!(
        control.status.code(),
        Some(1),
        "fixture must really violate"
    );

    let output = run_with(&manifest, "--baseline", &path);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(stderr.contains("invalid baseline"), "{stderr}");
    assert!(stderr.contains("owner"), "{stderr}");

    let _ = std::fs::remove_file(path);
}

#[test]
fn baseline_rewrite_refuses_wrong_typed_metadata_and_preserves_the_file() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("invalid-rewrite");
    std::fs::write(&path, wrong_typed_baseline()).expect("write malformed baseline");

    let output = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    for guidance in [
        "refusing to overwrite unsupported baseline",
        "Preserve any desired owner/tracker annotations",
        "move or delete the unsupported file",
        "--write-baseline",
    ] {
        assert!(stderr.contains(guidance), "missing `{guidance}`: {stderr}");
    }
    assert_eq!(
        std::fs::read_to_string(&path).unwrap(),
        wrong_typed_baseline(),
        "unsupported input must remain byte-for-byte unchanged"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn the_equals_form_still_carries_a_flag_shaped_value() {
    // `cli-check-runner` requires the `--flag=<value>` form to stay accepted for a value that
    // legitimately begins with `--`: it carries its value in the same token, so it can consume no
    // following flag, and rejecting a flag-shaped value in the space form must not also reject it.
    //
    // Both invocations exit 2, so the exit code cannot tell them apart — which is why this is an
    // end-to-end test rather than a `dispatch` unit test. stderr is what distinguishes them: the
    // space form never reaches the baseline reader and names the flag it found, while the equals
    // form reaches it with `--weird` as the path and reports it unreadable.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };

    let space_form = command_for(&manifest)
        .args(["--baseline", "--weird"])
        .output()
        .expect("run tianheng CLI");
    assert_eq!(space_form.status.code(), Some(2), "{space_form:?}");
    let space_stderr = String::from_utf8(space_form.stderr).expect("UTF-8 stderr");
    for guidance in [
        "--baseline requires a value",
        "'--weird'",
        "--baseline=<value>",
    ] {
        assert!(
            space_stderr.contains(guidance),
            "the space form must fail as a usage error naming the flag it found, missing \
             `{guidance}`: {space_stderr}"
        );
    }
    assert!(
        !space_stderr.contains("cannot read baseline"),
        "the space form must be rejected during parsing, before any baseline is read: \
         {space_stderr}"
    );

    let equals_form = command_for(&manifest)
        .arg("--baseline=--weird")
        .output()
        .expect("run tianheng CLI");
    assert_eq!(equals_form.status.code(), Some(2), "{equals_form:?}");
    let equals_stderr = String::from_utf8(equals_form.stderr).expect("UTF-8 stderr");
    assert!(
        equals_stderr.contains("cannot read baseline --weird"),
        "the equals form must deliver `--weird` to the baseline reader as a path, not be rejected \
         as a usage error: {equals_stderr}"
    );
    assert!(
        !equals_stderr.contains("requires a value"),
        "the equals form must not be treated as a missing value: {equals_stderr}"
    );
}

#[test]
fn a_repeated_flag_names_the_repeat_not_a_downstream_failure() {
    // `--baseline a --baseline b` exited 2 before the once-only rule and exits 2 after it, so no
    // exit code distinguishes them and this is the guard that actually reacts to the change: what
    // moved is which mistake the diagnostic names. Before, the second value silently won and the run
    // reported the FIRST file as if the invocation had never named it — "cannot read baseline
    // second", against a path the adopter did type but not the one they typed first, with no word
    // that two were given. Now the parse refuses before any baseline is read.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };

    let output = command_for(&manifest)
        .args(["--baseline", "first", "--baseline", "second"])
        .output()
        .expect("run tianheng CLI");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    for guidance in ["--baseline was given more than once", "single value"] {
        assert!(
            stderr.contains(guidance),
            "the repeat must be named as the usage error it is, missing `{guidance}`: {stderr}"
        );
    }
    assert!(
        !stderr.contains("cannot read baseline"),
        "a repeated flag must be rejected during parsing, before either value reaches the \
         baseline reader: {stderr}"
    );
}

#[test]
fn write_baseline_names_the_flag_that_cannot_apply_and_records_nothing() {
    // The end-to-end half of the inapplicable-flag rule: the exit code moves 0 -> 2 (asserted as a
    // unit test too), but only stderr shows the diagnostic names the flag rather than failing for
    // some unrelated reason, and only the filesystem shows the recording did not happen anyway.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let baseline = temp_baseline("inapplicable-flag");

    let output = command_for(&manifest)
        .args([
            "--write-baseline",
            &baseline.to_string_lossy(),
            "--warn-uncovered",
        ])
        .output()
        .expect("run tianheng CLI");
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    for guidance in [
        "--warn-uncovered cannot apply to --write-baseline",
        "usage:",
    ] {
        assert!(
            stderr.contains(guidance),
            "the diagnostic must name the inapplicable flag, missing `{guidance}`: {stderr}"
        );
    }
    assert!(
        !baseline.exists(),
        "a rejected invocation must record no baseline: {}",
        baseline.display()
    );
    let _ = std::fs::remove_file(&baseline);
}

#[test]
fn a_zero_length_baseline_is_recorded_afresh_but_partial_content_is_still_refused() {
    // A crash mid-create leaves exactly a zero-length file: `create_baseline_file` publishes its
    // directory entry before its first byte. Refusing to overwrite it protected nothing — zero bytes
    // hold no owner/tracker annotations — while telling the adopter to "preserve any desired
    // annotations" that are not there, and requiring a manual file move to recover. So it is
    // recorded afresh, and said so.
    //
    // The second half is what keeps that exception honest: whitespace and truncated JSON might have
    // held annotations before they were damaged, so they must still be refused and left untouched.
    // Without this, "empty" could quietly grow to mean "looks empty enough".
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };

    let empty = temp_baseline("zero-length-recorded");
    std::fs::write(&empty, "").expect("create a zero-length baseline");
    let recorded = run_with(&manifest, "--write-baseline", &empty);
    assert_eq!(
        recorded.status.code(),
        Some(0),
        "a zero-length baseline must be recorded afresh, not refused: {recorded:?}"
    );
    let recorded_stderr = String::from_utf8(recorded.stderr).expect("UTF-8 stderr");
    for guidance in ["was empty", "recording a fresh snapshot"] {
        assert!(
            recorded_stderr.contains(guidance),
            "the recovery must not be silent, missing `{guidance}`: {recorded_stderr}"
        );
    }
    let document: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&empty).expect("read back the baseline"))
            .expect("the recorded baseline must be valid JSON");
    assert_eq!(
        document["format"], "tianheng.baseline/structured-facts",
        "the fresh snapshot must be a whole semantic baseline: {document:?}"
    );
    let _ = std::fs::remove_file(&empty);

    for partial in ["   \n", "{\"format\":"] {
        let path = temp_baseline("partial-still-refused");
        std::fs::write(&path, partial).expect("create a partially-written baseline");
        let refused = run_with(&manifest, "--write-baseline", &path);
        assert_eq!(
            refused.status.code(),
            Some(2),
            "partial content ({partial:?}) must still be refused: {refused:?}"
        );
        let refused_stderr = String::from_utf8(refused.stderr).expect("UTF-8 stderr");
        assert!(
            refused_stderr.contains("refusing to overwrite unsupported baseline"),
            "partial content ({partial:?}) must keep the preserve-and-move guidance: \
             {refused_stderr}"
        );
        assert_eq!(
            std::fs::read_to_string(&path).expect("read back the refused file"),
            partial,
            "refused content must remain byte-for-byte unchanged"
        );
        let _ = std::fs::remove_file(&path);
    }
}

#[test]
fn the_gate_does_not_tolerate_a_zero_length_baseline() {
    // The asymmetry is deliberate and worth pinning, because it reads as an inconsistency without
    // its reason: the write action may regenerate a snapshot it owns, but the gate consumes a
    // declaration the adopter wrote, so a baseline it cannot parse must be reported rather than read
    // as "nothing is accepted" — which would silently discard their accepted-violation record.
    let Some(manifest) = fixture_manifest("violating") else {
        return;
    };
    let path = temp_baseline("zero-length-gate");
    std::fs::write(&path, "").expect("create a zero-length baseline");

    let output = run_with(&manifest, "--baseline", &path);
    assert_eq!(
        output.status.code(),
        Some(2),
        "the gate must reject a zero-length baseline: {output:?}"
    );
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.contains("invalid baseline"),
        "the gate must name it an invalid baseline: {stderr}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
fn an_empty_flag_value_names_the_flag_not_a_missing_file() {
    // An empty value is a flag given no value, and must be reported as that. Both forms exit 2
    // either way, so the exit code cannot guard this — stderr is the whole signal, which is why the
    // guard lives here rather than beside the parse tests. Before the rule, `--baseline=` carried
    // `""` onward and answered `cannot read baseline ` — a malformed invocation misreported as a
    // missing file, against a path nobody typed, with a dangling space where the path would be.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };

    for arg in ["--baseline=", "--write-baseline="] {
        let output = command_for(&manifest)
            .arg(arg)
            .output()
            .expect("run tianheng CLI");
        assert_eq!(output.status.code(), Some(2), "{arg}: {output:?}");
        let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
        let flag = arg.trim_end_matches('=');
        assert!(
            stderr.contains(&format!(
                "{flag} requires a value, but was given an empty one"
            )),
            "{arg} must be reported as a flag given no value: {stderr}"
        );
        assert!(
            !stderr.contains("cannot read baseline")
                && !stderr.contains("cannot write baseline")
                && !stderr.contains("No such file"),
            "{arg} must be rejected during parsing, never reported as a missing file: {stderr}"
        );
    }
}

#[test]
fn rewriting_an_existing_baseline_leaves_no_stray_temp_file() {
    // The overwrite path (an already-existing, supported baseline) writes durably: the merged
    // document lands at a sibling temp path first, then an atomic rename swaps it into place.
    // Exercise that path twice — the first `--write-baseline` creates the file, the second
    // overwrites the now-existing, supported baseline — and confirm the temp sibling never
    // lingers once the command exits, whichever branch (create or overwrite) actually ran.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("overwrite-no-stray-temp");
    let _ = std::fs::remove_file(&path);

    let first = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    assert!(path.exists(), "the first write must create the baseline");

    let second = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(
        second.status.code(),
        Some(0),
        "rewriting an already-valid baseline must still succeed: {second:?}"
    );

    let rewritten = std::fs::read_to_string(&path).expect("read back the rewritten baseline");
    let rewritten_doc: serde_json::Value =
        serde_json::from_str(&rewritten).expect("the rewritten baseline must be valid JSON");
    assert_eq!(
        rewritten_doc["format"], "tianheng.baseline/structured-facts",
        "the atomic-rename write must land the document whole, not truncated: {rewritten_doc:?}"
    );

    let sibling_temp_files: Vec<_> = std::fs::read_dir(path.parent().unwrap())
        .expect("read baseline's parent dir")
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.file_name())
        .filter(|name| {
            name.to_string_lossy()
                .starts_with(path.file_name().unwrap().to_string_lossy().as_ref())
                && name.to_string_lossy().contains(".tmp-")
        })
        .collect();
    assert!(
        sibling_temp_files.is_empty(),
        "no temp sibling should remain after a durable baseline write: {sibling_temp_files:?}"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
#[cfg(unix)]
fn a_directory_that_cannot_be_flushed_does_not_fail_a_landed_write() {
    // Both write paths flush the containing directory so a reported success is not undone by a
    // later crash. That flush needs a readable directory handle, which a directory can legally
    // refuse: mode 0300 is writable and traversable but not readable, so `File::open` on it answers
    // EACCES — and some FUSE and network mounts answer EINVAL/ENOSYS to the fsync itself. The flush
    // only strengthens a write that has already landed, so none of those may turn it into a
    // failure: reporting "cannot write baseline" for a baseline sitting correctly on disk would be
    // the worse outcome, and would regress adopters for whom this path worked before the flush
    // existed. Both branches are covered — create (no file yet) and overwrite (file present).
    use std::os::unix::fs::PermissionsExt;

    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let dir = std::env::temp_dir().join(format!("tianheng-unflushable-dir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).expect("create the test directory");

    // Restore a readable mode before any assertion can unwind, so a failure cannot leave an
    // unreadable directory behind in the temp dir.
    struct Restore(PathBuf);
    impl Drop for Restore {
        fn drop(&mut self) {
            let _ = std::fs::set_permissions(&self.0, std::fs::Permissions::from_mode(0o700));
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    let _restore = Restore(dir.clone());

    let created = dir.join("created.json");
    let overwritten = dir.join("overwritten.json");
    let first = run_with(&manifest, "--write-baseline", &overwritten);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o300))
        .expect("make the directory writable but not readable");

    // Prove the precondition before trusting the result. Root bypasses the read bit, so under it the
    // directory just made unreadable can still be opened and flushed and this test would pass
    // without exercising the tolerance path at all — a vacuous gate. Mirror `fixture_manifest`
    // above: skip when the precondition cannot hold, but never skip silently in CI, whose runner is
    // an ordinary user for whom it always does.
    if std::fs::File::open(&dir).is_ok() {
        assert!(
            std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
            "a mode-0300 directory is still readable by this process (running as root?), so the \
             unflushable-directory path is not exercised — this must not be skipped in CI"
        );
        return;
    }

    let create_in_unreadable = run_with(&manifest, "--write-baseline", &created);
    assert_eq!(
        create_in_unreadable.status.code(),
        Some(0),
        "creating a baseline must succeed even where the directory cannot be flushed: \
         {create_in_unreadable:?}"
    );
    let overwrite_in_unreadable = run_with(&manifest, "--write-baseline", &overwritten);
    assert_eq!(
        overwrite_in_unreadable.status.code(),
        Some(0),
        "overwriting a baseline must succeed even where the directory cannot be flushed: \
         {overwrite_in_unreadable:?}"
    );

    std::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700))
        .expect("restore a readable mode to read the results back");
    for path in [&created, &overwritten] {
        let document: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(path).expect("read the baseline"))
                .expect("the baseline must be valid JSON");
        assert_eq!(
            document["format"], "tianheng.baseline/structured-facts",
            "the write must land the document whole: {document:?}"
        );
    }
}

#[test]
#[cfg(unix)]
fn rewriting_an_existing_baseline_preserves_its_permissions() {
    // rename replaces whatever sits at its destination unconditionally, so a naive temp-then-
    // rename write would silently reset the baseline's mode to the temp file's process-umask
    // default — quietly widening permissions an adopter deliberately narrowed. The overwrite path
    // must read the existing mode and carry it over to the replacement.
    use std::os::unix::fs::PermissionsExt;

    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("overwrite-preserves-mode");
    let _ = std::fs::remove_file(&path);

    let first = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(first.status.code(), Some(0), "{first:?}");

    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600))
        .expect("narrow the baseline's permissions");

    let second = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(second.status.code(), Some(0), "{second:?}");

    let mode = std::fs::metadata(&path)
        .expect("read back the rewritten baseline's metadata")
        .permissions()
        .mode()
        & 0o777;
    assert_eq!(
        mode, 0o600,
        "rewriting an existing baseline must preserve its permissions, not reset them to the \
         process umask"
    );

    let _ = std::fs::remove_file(path);
}

#[test]
#[cfg(unix)]
fn rewriting_a_symlinked_baseline_preserves_the_symlink() {
    // rename replaces whatever sits at its destination unconditionally, so a naive temp-then-
    // rename write targeting the symlink path directly would replace the symlink itself with a
    // plain file, orphaning whatever it pointed at. The overwrite path must resolve the symlink
    // and swap the real target instead.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let real_target = temp_baseline("symlink-preserved-real-target");
    let link = temp_baseline("symlink-preserved-link");
    let _ = std::fs::remove_file(&real_target);
    let _ = std::fs::remove_file(&link);

    let first = run_with(&manifest, "--write-baseline", &real_target);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    std::os::unix::fs::symlink(&real_target, &link).expect("create the symlinked baseline");

    let second = run_with(&manifest, "--write-baseline", &link);
    assert_eq!(second.status.code(), Some(0), "{second:?}");

    assert!(
        link.symlink_metadata()
            .expect("read the link's own metadata")
            .file_type()
            .is_symlink(),
        "rewriting through a symlinked baseline path must not replace the symlink with a plain file"
    );
    assert_eq!(
        std::fs::read_link(&link).expect("read the symlink target"),
        real_target,
        "the symlink must still point at its original target"
    );

    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_file(&real_target);
}

#[test]
#[cfg(unix)]
fn a_symlink_planted_at_the_predicted_temp_path_is_refused_not_followed() {
    // The temp file's name is predictable (`<target>.tmp-<pid>`). A plain create-or-truncate write
    // would follow whatever already sat at that path — a symlink included — so an attacker who can
    // create files in the baseline's directory and watch for the process could plant a symlink to
    // an arbitrary victim file right after the process starts, redirecting the write (and the
    // permission change that follows it) onto it. `create_new` (O_EXCL) must refuse outright instead.
    use std::os::unix::fs::PermissionsExt;

    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let baseline = temp_baseline("symlink-race-baseline");
    let victim = temp_baseline("symlink-race-victim");
    let _ = std::fs::remove_file(&baseline);
    let _ = std::fs::remove_file(&victim);

    let first = run_with(&manifest, "--write-baseline", &baseline);
    assert_eq!(first.status.code(), Some(0), "{first:?}");
    std::fs::write(&victim, "TOP SECRET, MUST NOT BE OVERWRITTEN").expect("write victim file");
    std::fs::set_permissions(&victim, std::fs::Permissions::from_mode(0o640))
        .expect("narrow the victim's permissions");
    let victim_before = std::fs::read_to_string(&victim).expect("read victim before");

    // The plant races the child, and an unexercised run leaves an untouched victim too — which is
    // exactly what this test asserts, so it would pass vacuously. `write_baseline_colliding_with`
    // returns only a run that really tried to open the planted path, and fails loud otherwise.
    let (output, predicted_tmp) =
        write_baseline_colliding_with(&manifest, &baseline, |predicted| {
            let _ = std::os::unix::fs::symlink(&victim, predicted);
        });
    let _ = std::fs::remove_file(&predicted_tmp);

    let victim_after = std::fs::read_to_string(&victim).expect("read victim after");
    assert_eq!(
        victim_after, victim_before,
        "a symlink planted at the predicted temp path must never redirect the write onto it: \
         {output:?}"
    );
    assert_eq!(
        std::fs::metadata(&victim)
            .expect("read victim metadata")
            .permissions()
            .mode()
            & 0o777,
        0o640,
        "the victim's permissions must not be touched by the redirected write"
    );
    assert!(
        !baseline
            .symlink_metadata()
            .expect("read baseline's own metadata")
            .file_type()
            .is_symlink(),
        "the real baseline path must not become a dangling symlink to the victim"
    );

    let _ = std::fs::remove_file(&baseline);
    let _ = std::fs::remove_file(&victim);
}

#[test]
#[cfg(unix)]
fn a_stale_leftover_temp_file_is_reported_by_its_own_name_not_the_baseline_path() {
    // A stale `<target>.tmp-<pid>` left behind by an interrupted prior run (a killed process, or a
    // pid reused across a fresh container) makes create_new's open fail with AlreadyExists. That
    // must not surface as a bare "cannot write baseline <path>: File exists" — <path> already
    // existing is the whole point of an overwrite, so that message names nothing the adopter can
    // act on. It must name the actual colliding temp file and say why it's there.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let baseline = temp_baseline("stale-temp-collision-baseline");
    let _ = std::fs::remove_file(&baseline);

    // The plant races the child; a run where it lands late writes the baseline cleanly and exits 0,
    // which used to fail this test spuriously (observed in CI) rather than telling anyone the
    // collision was never reached. `write_baseline_colliding_with` re-races until it is.
    let (output, predicted_tmp) =
        write_baseline_colliding_with(&manifest, &baseline, |predicted| {
            let _ = std::fs::write(predicted, "leftover from an interrupted run");
        });

    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.contains(&predicted_tmp),
        "the message must name the actual colliding temp file, not just the baseline path: {stderr}"
    );
    assert!(
        stderr.contains("interrupted run"),
        "the message must explain why a leftover temp file is a plausible, non-alarming cause: \
         {stderr}"
    );
    assert!(
        !stderr.contains("File exists"),
        "the raw io::Error text must not leak through as the whole explanation: {stderr}"
    );

    let _ = std::fs::remove_file(&predicted_tmp);
    let _ = std::fs::remove_file(&baseline);
}

#[test]
#[cfg(unix)]
fn a_dangling_symlink_baseline_path_is_reported_by_its_own_cause_not_a_race() {
    // A baseline path that is a symlink to a deleted target reads as NotFound (the create-new
    // path runs), then create_new's O_EXCL fails with AlreadyExists — indistinguishable from a
    // genuine concurrent creation without checking symlink_metadata explicitly. Unlike a real
    // race, this is a permanent state: "rerun the command" (the concurrent-creation arm's own
    // remedy) can never succeed here, so it must not be reported as if something appeared
    // concurrently.
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let target = temp_baseline("dangling-symlink-target");
    let link = temp_baseline("dangling-symlink-link");
    let _ = std::fs::remove_file(&target);
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&target, &link).expect("create a dangling symlink");

    let output = run_with(&manifest, "--write-baseline", &link);
    assert_eq!(output.status.code(), Some(2), "{output:?}");
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.contains(target.to_str().unwrap()),
        "the message must name what the dangling symlink (no longer) points at: {stderr}"
    );
    assert!(
        !stderr.contains("appeared while the new snapshot was being prepared"),
        "a dangling symlink is a permanent state, not a concurrent-creation race, and must not be \
         reported as one — \"rerun the command\" would fail identically forever: {stderr}"
    );
    assert!(
        link.symlink_metadata()
            .expect("read the link's own metadata")
            .file_type()
            .is_symlink(),
        "the dangling link itself must be left untouched"
    );

    let _ = std::fs::remove_file(&link);
}

#[test]
#[cfg(unix)]
fn rewriting_through_a_symlink_into_a_non_utf8_named_directory_still_succeeds() {
    // The temp path is built by appending to the resolved target's raw OsString, never through
    // `Path::display()` (which lossily replaces non-UTF-8 bytes for human-readable formatting) — a
    // resolved path is not guaranteed valid UTF-8, and a lossy round-trip through a new string can
    // point at a directory that does not exist, failing an otherwise-valid overwrite outright.
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };

    // A normal write first, purely to obtain valid baseline content without needing a non-UTF-8
    // CLI argument (which std::env::args() would reject before this code ever runs).
    let seed = temp_baseline("nonutf8-dir-seed");
    let _ = std::fs::remove_file(&seed);
    let seed_write = run_with(&manifest, "--write-baseline", &seed);
    assert_eq!(seed_write.status.code(), Some(0), "{seed_write:?}");
    let valid_baseline_content = std::fs::read_to_string(&seed).expect("read seed baseline");
    let _ = std::fs::remove_file(&seed);

    let mut dir_name = std::env::temp_dir().into_os_string().into_vec();
    dir_name.extend_from_slice(format!("/tianheng-nonutf8-{}-", std::process::id()).as_bytes());
    dir_name.push(0xFF);
    let weird_dir = PathBuf::from(OsString::from_vec(dir_name));
    let _ = std::fs::remove_dir_all(&weird_dir);
    std::fs::create_dir(&weird_dir).expect("create the non-UTF-8-named directory");

    let real_target = weird_dir.join("baseline.json");
    std::fs::write(&real_target, &valid_baseline_content).expect("seed the real target");
    let link = temp_baseline("nonutf8-dir-link");
    let _ = std::fs::remove_file(&link);
    std::os::unix::fs::symlink(&real_target, &link).expect("create the symlinked baseline");

    let second = run_with(&manifest, "--write-baseline", &link);
    assert_eq!(
        second.status.code(),
        Some(0),
        "rewriting through a symlink into a non-UTF-8-named directory must still succeed: {second:?}"
    );

    let _ = std::fs::remove_file(&link);
    let _ = std::fs::remove_dir_all(&weird_dir);
}

#[test]
fn disallow_stale_without_baseline_is_a_usage_error() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let output = command_for(&manifest)
        .arg("--disallow-stale")
        .output()
        .expect("run CLI");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.contains("--disallow-stale requires --baseline"),
        "{stderr}"
    );
    assert!(
        stderr.contains("[--disallow-stale]"),
        "usage synopsis must advertise the supported flag: {stderr}"
    );
}

#[test]
fn disallow_stale_fails_gate_when_stale_entry_is_present() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("stale-gate");
    let stale_baseline_text = r#"{"format":"tianheng.baseline/structured-facts","violations":[{
        "target":"clean","rule":"test-rule","finding":"test-finding",
        "rule_key":{"type":"tianheng.rule/test","fields":{}},
        "fact":{"type":"tianheng.fact/test","shape":"test","fields":{}}
    }]}"#;
    std::fs::write(&path, stale_baseline_text).expect("write baseline with stale entry");

    // Normal gate without --disallow-stale exits 0 (stale entries are advisory)
    let normal_output = run_with(&manifest, "--baseline", &path);
    assert_eq!(normal_output.status.code(), Some(0));

    // Gate with --disallow-stale fails and exits 1
    let stale_output = command_for(&manifest)
        .args(["--baseline", path.to_str().unwrap(), "--disallow-stale"])
        .output()
        .expect("run CLI");
    assert_eq!(stale_output.status.code(), Some(1));
    let stderr = String::from_utf8(stale_output.stderr).expect("UTF-8 stderr");
    assert!(stderr.contains("stale baseline entry"), "{stderr}");
    assert!(stderr.contains("--disallow-stale failed"), "{stderr}");

    let _ = std::fs::remove_file(path);
}

#[test]
fn disallow_stale_json_and_sarif_projections_are_consistent_with_exit_code() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("stale-projections");
    let stale_baseline_text = r#"{"format":"tianheng.baseline/structured-facts","violations":[{
        "target":"clean","rule":"test-rule","finding":"test-finding",
        "rule_key":{"type":"tianheng.rule/test","fields":{}},
        "fact":{"type":"tianheng.fact/test","shape":"test","fields":{}}
    }]}"#;
    std::fs::write(&path, stale_baseline_text).expect("write baseline with stale entry");

    // JSON format under --disallow-stale
    let json_output = command_for(&manifest)
        .args([
            "--baseline",
            path.to_str().unwrap(),
            "--disallow-stale",
            "--format",
            "json",
        ])
        .output()
        .expect("run CLI");
    assert_eq!(json_output.status.code(), Some(1));
    let json_doc: serde_json::Value =
        serde_json::from_slice(&json_output.stdout).expect("valid JSON");
    assert_eq!(json_doc["exit_code"], 1);
    assert_eq!(json_doc["outcome"], "violations");
    assert_eq!(json_doc["stale_disallowed"], true);
    assert_eq!(json_doc["stale_baseline"].as_array().unwrap().len(), 1);

    // SARIF format under --disallow-stale
    let sarif_output = command_for(&manifest)
        .args([
            "--baseline",
            path.to_str().unwrap(),
            "--disallow-stale",
            "--format",
            "sarif",
        ])
        .output()
        .expect("run CLI");
    assert_eq!(sarif_output.status.code(), Some(1));
    let sarif_doc: serde_json::Value =
        serde_json::from_slice(&sarif_output.stdout).expect("valid SARIF JSON");
    let run = &sarif_doc["runs"][0];
    assert_eq!(run["results"].as_array().unwrap().len(), 1);
    assert_eq!(run["results"][0]["level"], "error");
    assert_eq!(run["invocations"][0]["executionSuccessful"], false);

    let _ = std::fs::remove_file(path);
}

#[test]
fn disallow_stale_equals_form_is_unrecognized_argument_usage_error() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("stale-equals");
    std::fs::write(&path, wrong_typed_baseline()).expect("write baseline");

    let output = command_for(&manifest)
        .args([
            "--baseline",
            path.to_str().unwrap(),
            "--disallow-stale=false",
        ])
        .output()
        .expect("run CLI");
    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    assert!(
        stderr.contains("unrecognized argument '--disallow-stale=false'"),
        "{stderr}"
    );

    let _ = std::fs::remove_file(path);
}
