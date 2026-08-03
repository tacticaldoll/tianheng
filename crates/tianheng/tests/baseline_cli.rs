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

    let real_baseline = std::fs::canonicalize(&baseline).expect("canonicalize baseline");
    let child = command_for(&manifest)
        .args([
            "--write-baseline",
            baseline.to_str().expect("UTF-8 baseline path"),
        ])
        .spawn()
        .expect("spawn tianheng CLI");
    let predicted_tmp = format!("{}.tmp-{}", real_baseline.display(), child.id());
    let _ = std::os::unix::fs::symlink(&victim, &predicted_tmp);
    let output = child.wait_with_output().expect("wait for tianheng CLI");
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
