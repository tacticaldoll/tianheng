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
