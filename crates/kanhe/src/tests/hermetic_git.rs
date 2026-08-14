use crate::hermetic_git::hermetic;
use std::process::Command;

/// The load-bearing half of [`hermetic`], as a case rather than as a sentence.
///
/// Every fixture in this crate assumes the global config file cannot reach it. If that stopped being true the
/// fixtures would silently build the judged machine's shape instead of the one they named, and every refusal
/// they claim to demonstrate would be demonstrating something else.
///
/// The control is the same command without the builder, so the assertion is a **difference** and not the
/// absence of a key that might never have been readable.
#[test]
fn the_global_config_file_cannot_reach_a_hermetic_command() {
    let home = std::env::temp_dir().join(format!("kanhe-hermetic-{}", std::process::id()));
    std::fs::create_dir_all(&home).expect("create the fixture home");
    std::fs::write(home.join(".gitconfig"), "[probe]\n\tkey = AMBIENT\n").expect("write");

    let read = |mut command: Command| {
        let out = command
            .args(["config", "--get", "probe.key"])
            .env("HOME", &home)
            .output()
            .expect("run git");
        String::from_utf8_lossy(&out.stdout).trim().to_string()
    };

    let ambient = read(Command::new("git"));
    let isolated = read(hermetic("git"));
    std::fs::remove_dir_all(&home).ok();

    assert_eq!(
        ambient, "AMBIENT",
        "the control did not read the fixture's global config, so this comparison would hold for the wrong \
         reason"
    );
    assert_eq!(
        isolated, "",
        "a hermetic command read the global config file; every fixture in this crate assumes it cannot"
    );
}
