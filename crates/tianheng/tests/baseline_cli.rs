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
    r#"{"version":2,"violations":[{
        "target":"example-core","rule":"deny external dependencies","finding":"serde",
        "finding_key":{"namespace":"crate","code":"dependency","fields":{"package":"serde"}},
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
fn baseline_rewrite_warns_before_replacing_wrong_typed_metadata() {
    let Some(manifest) = fixture_manifest("clean") else {
        return;
    };
    let path = temp_baseline("invalid-rewrite");
    std::fs::write(&path, wrong_typed_baseline()).expect("write malformed baseline");

    let output = run_with(&manifest, "--write-baseline", &path);
    assert_eq!(output.status.code(), Some(0));
    let stderr = String::from_utf8(output.stderr).expect("UTF-8 stderr");
    let warning = stderr
        .find("could not be parsed")
        .expect("warning names parse failure");
    let loss = stderr
        .find("owner/tracker metadata is not carried forward")
        .expect("warning names metadata loss");
    let written = stderr
        .find("wrote 0 violation(s)")
        .expect("write follows warning");
    assert!(warning < loss && loss < written, "{stderr}");

    let rewritten = std::fs::read_to_string(&path).expect("fresh baseline written");
    assert!(rewritten.contains("\"version\": 2"), "{rewritten}");
    assert!(!rewritten.contains("owner"), "{rewritten}");

    let _ = std::fs::remove_file(path);
}
