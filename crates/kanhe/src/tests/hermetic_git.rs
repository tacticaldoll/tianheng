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
    let _ = std::fs::remove_dir_all(&home);
    xingbiao::claim_scratch(&home).expect("create the fixture home");
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

/// The ignore row of [`hermetic`]'s table, as a case rather than as a sentence.
///
/// **This row read *not closed* for two windows and the sentence was correct.** Emptying the config *files*
/// leaves `$XDG_CONFIG_HOME/git/ignore` in force, because that path is the default git uses when no config
/// file names one — so the fixtures were isolated from configuration and not from exclusion. What it cost is
/// the direction worth pinning: a fixture's `git add -A` left a matching file **untracked**, silently building
/// a repository without a file the fixture named, and an ignore query on the real workspace answered *ignored*
/// where that answer excuses an offence.
///
/// The control is the same command with those variables cleared rather than a bare `Command`, so the
/// assertion is a **difference** on the one variable set under test and cannot hold because the probe was
/// never readable on this machine.
#[test]
fn an_ignore_file_outside_the_repository_cannot_reach_a_hermetic_command() {
    let home = std::env::temp_dir().join(format!("kanhe-hermetic-ignore-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&home);
    xingbiao::claim_scratch(&home).expect("create the fixture home");
    let xdg = home.join("xdg");
    std::fs::create_dir_all(xdg.join("git")).expect("create the fixture XDG tree");
    std::fs::write(xdg.join("git").join("ignore"), "probe-excluded\n").expect("write");
    let repo = home.join("repo");
    std::fs::create_dir_all(&repo).expect("create the fixture repository");
    std::fs::write(repo.join("probe-excluded"), "content").expect("write");

    let ignored = |mut command: Command| {
        command
            .args(["check-ignore", "-q", "--", "probe-excluded"])
            .env("HOME", &home)
            .env("XDG_CONFIG_HOME", &xdg)
            .current_dir(&repo)
            .status()
            .expect("run git")
            .success()
    };
    let bare = || {
        let mut command = Command::new("git");
        command
            .env_remove("GIT_CONFIG_COUNT")
            .env_remove("GIT_CONFIG_KEY_0")
            .env_remove("GIT_CONFIG_VALUE_0")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_CONFIG_SYSTEM", "/dev/null")
            .env("GIT_CONFIG_NOSYSTEM", "1");
        command
    };

    let init = hermetic("git")
        .args(["init", "-q"])
        .current_dir(&repo)
        .status()
        .expect("run git init");
    assert!(init.success(), "could not init the fixture repository");

    let ambient = ignored(bare());
    let isolated = ignored(hermetic("git"));
    std::fs::remove_dir_all(&home).ok();

    assert!(
        ambient,
        "the control did not read the fixture's XDG ignore file, so this comparison would hold for the wrong \
         reason — emptying the config files is what the control already does"
    );
    assert!(
        !isolated,
        "a hermetic command read an ignore file outside the repository. For an ignore query that answer \
         excuses an offence, and for a fixture's `add` it silently omits a file the fixture named"
    );
}
