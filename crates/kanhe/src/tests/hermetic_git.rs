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

/// The probe half of [`no_ambient_configuration_reaches_a_hermetic_command`], reached as a child process.
///
/// It exists because the variables under test have to arrive by **inheritance**. Setting one on the builder's
/// own [`Command`] would overwrite the removal and test the case's last statement; mutating this process's
/// environment is unsafe in this edition and racy against a parallel run. A child inherits, so the parent
/// sets the ambient environment and this runs `hermetic` inside it.
///
/// Returns without doing anything unless the parent asked, so the ordinary suite pays nothing for it.
#[test]
fn hermetic_configuration_probe() {
    let Some(repo) = std::env::var_os("KANHE_HERMETIC_PROBE_REPO") else {
        return;
    };
    let out = hermetic("git")
        .args(["config", "--list", "--show-origin"])
        .current_dir(std::path::Path::new(&repo))
        .output()
        .expect("run git config under the builder");
    println!(
        "PROBE_BEGIN\n{}PROBE_END",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// No configuration reaches a hermetic command but this builder's own — asked of the run, not of a list.
///
/// **Three rounds widened this builder by name, and a name list is as complete as the last person's memory.**
/// The config files went first, then the repository selectors, then the object-directory pair — each added
/// after someone measured it. `GIT_CONFIG_PARAMETERS` was in none of them: it is a channel parallel to
/// `GIT_CONFIG_COUNT`, so occupying index 0 does nothing to it, and git **exports it itself** under any
/// `git -c …`, which reaches a gate run from a hook, an alias or `bisect run`. Measured, it moved
/// `status --porcelain --untracked-files=all` into hiding an excluded file — the read
/// `publish-source-integrity#worktree-is-not-clean` rests on.
///
/// So this asks the question the other way round. It sets an ambient environment, runs the builder in a child
/// that inherits it, and **classifies every line git reports** against the two origins this builder admits:
/// what it wrote, and the fixture's own `.git/config`, which it empties the global and system files around
/// and does not claim to govern. A channel nobody has named carries a setting from neither, so it is named by
/// what arrived rather than by anyone having thought of it.
///
/// Classifying the whole listing rather than the command-line entries is what lets a **replacing** channel be
/// seen at all: `GIT_CONFIG` names a file and git then lists that file *alone*, so a reader filtering to
/// command-line entries meets an empty set and reports the absence — naming neither the setting nor where to
/// close it. Both channels are delivered here, and either one left uncleared shows up in what is classified.
#[test]
fn no_ambient_configuration_reaches_a_hermetic_command() {
    let root = std::env::temp_dir().join(format!("kanhe-hermetic-config-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create the fixture root");
    for args in [
        vec!["init", "-q", "."],
        vec!["config", "user.email", "t@example.invalid"],
        vec!["config", "user.name", "t"],
    ] {
        assert!(
            hermetic("git")
                .args(&args)
                .current_dir(&root)
                .output()
                .expect("run git")
                .status
                .success()
        );
    }

    // Two settings on the ambient channel, and **neither is the ignore setting**, deliberately. The sibling
    // sweep `no_judgement_reads_an_ambient_ignore_file` decides whether a file closed that channel by
    // whether the file *names* the setting — a criterion its own requirement states — so writing the name
    // here as an attack string would read as this file having neutralised it, and would take the
    // channel-control exception with it. Measured: it does, and the sweep says so.
    //
    // Nothing is lost. The builder's defence against the indexed channel is occupying the count, and this
    // channel bypasses the count whatever key it carries; the assertion below requires every reported
    // `command line:` entry to be the builder's, so any setting at all is enough to fail it.
    let parameters = "'user.name=ambient-probe' 'core.fileMode=false'";

    // The second member of `CONFIG_CHANNELS`, delivered rather than only named. The constant listed
    // `GIT_CONFIG` and nothing constructed it: deleting it from there left the whole suite green, which is
    // the shape this file's own header argues against — a list grown by memory, one entry longer.
    //
    // The file is placed at a path ending `.git/config`, which is this variable's most natural value and was
    // the exact input that walked through the first classification: a substring test for the fixture's own
    // config admitted every line of it, `core.excludesFile` included — the one setting this whole surface
    // exists to own. The path stays as it is so that predicate cannot be loosened back without failing here.
    let foreign = root.join("foreign.git");
    std::fs::create_dir_all(&foreign).expect("create the foreign repository directory");
    let foreign = foreign.join("config");
    std::fs::write(
        &foreign,
        "[user]\n\tname = foreign-file-probe\n[core]\n\tfileMode = false\n",
    )
    .expect("write the foreign configuration file");

    // A control per channel, because they do not compose: measured, `GIT_CONFIG` **replaces** the listing and
    // suppresses `GIT_CONFIG_PARAMETERS` entirely, so one environment carrying both demonstrates only the
    // second. Without these the assertions below could hold because a variable was never readable here.
    let read_with = |key: &str, value: &std::ffi::OsStr| {
        let out = Command::new("git")
            .args(["config", "--list", "--show-origin"])
            .env(key, value)
            .current_dir(&root)
            .output()
            .expect("run the control");
        String::from_utf8_lossy(&out.stdout).into_owned()
    };
    let by_parameters = read_with("GIT_CONFIG_PARAMETERS", std::ffi::OsStr::new(parameters));
    let by_file = read_with("GIT_CONFIG", foreign.as_os_str());

    // The subject: a child of this test binary, inheriting the ambient environment, running the builder.
    let probe = Command::new(std::env::current_exe().expect("this test binary"))
        .args([
            "--exact",
            "tests::hermetic_git::hermetic_configuration_probe",
            "--nocapture",
            "--test-threads=1",
        ])
        .env("GIT_CONFIG_PARAMETERS", parameters)
        .env("GIT_CONFIG", &foreign)
        .env("KANHE_HERMETIC_PROBE_REPO", &root)
        .output()
        .expect("run the probe child");
    let probe = String::from_utf8_lossy(&probe.stdout).into_owned();
    let reported = probe
        .split_once("PROBE_BEGIN\n")
        .and_then(|(_, rest)| rest.split_once("PROBE_END"))
        .map(|(body, _)| body.to_string())
        .unwrap_or_else(|| panic!("the probe child produced no reading:\n{probe}"));

    std::fs::remove_dir_all(&root).ok();

    for (channel, control, arrival) in [
        ("GIT_CONFIG_PARAMETERS", &by_parameters, "ambient-probe"),
        ("GIT_CONFIG", &by_file, "foreign-file-probe"),
    ] {
        assert!(
            control.contains(arrival),
            "the control did not read `{channel}`, so the assertions below would hold for the wrong \
             reason:\n{control}"
        );
    }
    // **The observation port is the whole listing, not one origin class.** The first form filtered to
    // `command line:` entries and asserted each was the builder's — which reads a channel that *adds* to the
    // listing and cannot see one that *replaces* it. Measured with `GIT_CONFIG` naming a file: git lists that
    // file alone, `command line:` has zero entries, and the case failed on emptiness saying *this read is not
    // about the builder* — neither naming the setting that arrived nor where to close it, which is the half
    // of its own scenario that matters.
    //
    // So every line is classified, and the two admissible origins are named: what this builder wrote, and the
    // fixture's own repository config, which the builder empties the global and system files around and does
    // not claim to govern.
    // **Exact and derived, for the reason the repository-origin predicate below gives.** Asking whether the
    // line *contained* `core.excludesfile=/dev/null` admitted an ambient `core.excludesfile=/dev/null-evil`
    // arriving on the same channel as this builder's own — measured. The value the builder writes is a
    // prefix of every value that looks like it, and an unnamed channel is free to choose one. The builder
    // writes exactly one setting, taken from the constant beside it, so the whole entry is derivable and the
    // comparison is equality. `git config --list` lower-cases the section and key it echoes.
    let builders_entry = format!(
        "command line:\t{}=/dev/null",
        crate::hermetic_git::EXCLUDES_SETTING.to_lowercase()
    );
    let builders = |line: &str| line == builders_entry;
    // **Exact, because the ambient side controls the path.** A substring test for `.git/config` admitted any
    // file whose path happens to contain it, and `GIT_CONFIG` pointing at another repository's config is that
    // variable's ordinary use: measured, every line of the foreign file passed as *the fixture's own*, so the
    // classification fell through to the absence assertion below and reported what was missing instead of
    // what arrived. The case already holds `root` and git renders the repository's own origin relative to the
    // directory it runs in — measured, `file:.git/config` — so exactness costs nothing here.
    let the_repositorys = |line: &str| line.starts_with("file:.git/config\t");
    let unexpected: Vec<&str> = reported
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter(|line| !builders(line) && !the_repositorys(line))
        .collect();
    assert!(
        unexpected.is_empty(),
        "configuration reached a hermetic command that this builder did not write: {unexpected:?}. Whatever \
         channel carried it is open for every caller — close it in `hermetic` beside `CONFIG_CHANNELS`, and \
         keep this case as the thing that found it:\n{reported}"
    );
    assert!(
        reported.lines().any(builders),
        "the builder's own setting is absent from what git reported, so a channel did not add to this \
         listing but REPLACED it — the repair is the same and the channel is whatever produced what is left. \
         Asserted after the classification above so a replacing channel is named by its content first:\n\
         {reported}"
    );
}

/// The repository-selector row of [`hermetic`]'s table, in the two halves this one can be built in.
///
/// **The channel is real, and that half is a behaviour case.** `GIT_DIR` reaches past `current_dir` entirely:
/// a judgement that reads `HEAD`'s subject, the worktree's cleanliness and the release tag gets all three
/// from whatever repository the variable names, while `cargo publish` packages the directory on disk. The
/// gate and the act would then be about two different trees, in front of an upload that can be yanked and
/// never replaced.
///
/// **The other half is a construction case, and the reason is this file's own.** Making the variable arrive
/// the way it really would means mutating this process's environment — `set_var`, unsafe in this edition and
/// racy against a parallel run, exactly as [`hermetic`]'s header already records for the `GIT_CONFIG_*` row.
/// Setting it on the builder's own [`Command`] instead proves nothing: a later `env` overrides the
/// `env_remove`, so the case would be testing its own last statement. So the removal is read off the builder,
/// which is the strongest form available without the mutation — and it is a **difference** against a bare
/// `Command`, so it cannot hold because the key was never there.
///
/// What this pair does not establish is the composition: that a variable inherited from a real environment is
/// absent in the child. Stated rather than implied, and it is the same residue the sibling rows carry.
#[test]
fn a_repository_selector_cannot_reach_a_hermetic_command() {
    let root = std::env::temp_dir().join(format!("kanhe-hermetic-selector-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("create the fixture root");

    // Two repositories whose HEAD subjects differ, so a redirected read is legible as the wrong subject
    // rather than as an error.
    let build = |name: &str| {
        let dir = root.join(name);
        std::fs::create_dir(&dir).expect("create the fixture repository");
        for args in [
            vec!["init", "-q", "."],
            vec!["config", "user.email", "t@example.invalid"],
            vec!["config", "user.name", "t"],
        ] {
            assert!(
                hermetic("git")
                    .args(&args)
                    .current_dir(&dir)
                    .output()
                    .expect("run git")
                    .status
                    .success()
            );
        }
        std::fs::write(dir.join("f"), name).expect("write");
        for args in [vec!["add", "f"], vec!["commit", "-qm", name]] {
            assert!(
                hermetic("git")
                    .args(&args)
                    .current_dir(&dir)
                    .output()
                    .expect("run git")
                    .status
                    .success()
            );
        }
        dir
    };
    let judged = build("judged");
    let elsewhere = build("elsewhere");

    // The channel, demonstrated on a command that does NOT close it: the subject read in `judged` is
    // `elsewhere`'s.
    let redirected = Command::new("git")
        .args(["log", "-1", "--format=%s"])
        .env("GIT_DIR", elsewhere.join(".git"))
        .current_dir(&judged)
        .output()
        .expect("run git");
    let redirected = String::from_utf8_lossy(&redirected.stdout)
        .trim()
        .to_string();

    // The removal, read off the builder rather than off a run — see this case's header for why.
    let removed: Vec<String> = hermetic("git")
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(key, _)| key.to_string_lossy().into_owned())
        .collect();
    let bare: Vec<String> = Command::new("git")
        .get_envs()
        .filter(|(_, value)| value.is_none())
        .map(|(key, _)| key.to_string_lossy().into_owned())
        .collect();

    std::fs::remove_dir_all(&root).ok();

    assert_eq!(
        redirected, "elsewhere",
        "the control did not follow GIT_DIR, so the channel this case is about was not demonstrated and the \
         assertion below would hold for the wrong reason"
    );
    assert!(
        bare.is_empty(),
        "a bare `Command` already clears something, so the comparison below is not a difference: {bare:?}"
    );
    for selector in ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"] {
        assert!(
            removed.iter().any(|key| key == selector),
            "`hermetic` does not clear {selector}, so a judgement's reads follow it past `current_dir` to \
             whatever repository it names — while the act the gate stands in front of uses the directory on \
             disk. Cleared: {removed:?}"
        );
    }
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

/// The count this builder writes is what closes the `GIT_CONFIG_*` channel, asserted on the construction.
///
/// **This turns a corrected row into a check, and it checks the construction rather than simulating the
/// ambient environment.** The table recorded that channel as **open** — *any key reaches `git`* — and that
/// was wrong in the direction that costs: a row saying a channel is open reads as governed policy and would
/// send the next fixture author to build isolation they already have.
///
/// Why the construction and not a run: `Command::env` *overrides* the inherited environment for a key, so a
/// direction that sets `GIT_CONFIG_COUNT=2` on the command is overriding this builder rather than standing in
/// for an ambient value. The first draft of this did exactly that and failed, which is the trap rather than a
/// finding. Constructing the real ambient case needs the test process's own environment mutated — `set_var`,
/// unsafe in this edition and racy against a parallel run — or a child process to carry it.
///
/// So the property is split the way this repository's own law asks: the **construction** is asserted here —
/// the count is written, and index `0` is taken — and the *consequence* is measured once and recorded in
/// [`hermetic`]'s own table, where `git config --get user.name` under this builder exits `1` while the same
/// ambient pair without it answers the ambient value. `git` reading only indices below the count is git's
/// documented contract, not this file's guess.
#[test]
fn the_builder_writes_the_config_count_and_takes_index_zero() {
    let command = hermetic("git");
    let envs: Vec<(String, Option<String>)> = command
        .get_envs()
        .map(|(k, v)| {
            (
                k.to_string_lossy().into_owned(),
                v.map(|v| v.to_string_lossy().into_owned()),
            )
        })
        .collect();
    let value = |name: &str| {
        envs.iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.clone())
            .unwrap_or_else(|| panic!("`{name}` is not set on the command: {envs:?}"))
    };

    assert_eq!(
        value("GIT_CONFIG_COUNT").as_deref(),
        Some("1"),
        "the count is what makes an ambient key at any higher index unreachable; without it written here the \
         channel is as open as the table used to claim"
    );
    assert_eq!(
        value("GIT_CONFIG_KEY_0").as_deref(),
        Some(crate::hermetic_git::EXCLUDES_SETTING),
        "index 0 is the one index git will read under that count, so this builder has to own it — and what \
         it names is the setting that closes the ignore row"
    );
    assert_eq!(
        value("GIT_CONFIG_VALUE_0").as_deref(),
        Some("/dev/null"),
        "naming the setting without neutralising it would leave the XDG default in force"
    );
}
