//! Repository check: Definition of Done coherence between AGENTS.md and .github/workflows/ci.yml.
//!
//! Asserts that every command listed in AGENTS.md's Definition of Done block appears
//! in .github/workflows/ci.yml so local pre-flight gates remain a strict subset of CI.
//!
//! **Deliberately narrower than its deleted shell predecessor, the DoD-coherence gate script.** That
//! script additionally required three named "focused example matrix" scripts and a positive driver
//! script to appear as one contiguous, ordered sequence in both documents, and required the driver's own
//! source to never name a matrix script directly — guarding against the matrices and the driver silently
//! reordering or nesting relative to each other. Investigated rather than ported: those separate scripts
//! no longer exist. The shell-to-Rust migration consolidated them into one Rust test,
//! `crates/shengmo/tests/examples_suite.rs`, which owns its own example table and ordering internally
//! (checked by the compiler, not by grepping source text for basenames) and is named on a single DoD/CI
//! line this file's membership check already covers. There is no longer a sequence of separate commands
//! to order, and no separate driver script that could recurse into a matrix script, so a check for either
//! would have nothing left to react to.

mod support;

use std::collections::BTreeMap;
use std::path::PathBuf;

use shengmo::workspace::MARKER;
use support::{shell, workflow};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("AGENTS.md").is_file() && root.join(".github/workflows/ci.yml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

fn dod_commands(agents: &str) -> Vec<String> {
    let mut in_dod = false;
    let mut in_code_block = false;
    let mut commands = Vec::new();

    for line in agents.lines() {
        if line.trim() == "## Definition of Done" {
            in_dod = true;
            continue;
        }
        if in_dod && line.trim() == "```bash" {
            in_code_block = true;
            continue;
        }
        if in_code_block {
            if line.trim() == "```" {
                break;
            }
            let command = line
                .split_once('#')
                .map_or(line, |(command, _)| command)
                .trim();
            if !command.is_empty() {
                commands.push(command.to_string());
            }
        }
    }

    assert!(
        !commands.is_empty(),
        "No commands found in AGENTS.md Definition of Done code block"
    );
    commands
}

/// Every command CI's steps spell, as argv under the environment the workflow declares.
///
/// **Read from the workflow's structure, scoped as GitHub scopes it.** Each step's `run:` script is read in the
/// environment the workflow declares for that step — the workflow's, its job's and its own — so `cargo "+$MSRV"`
/// is `cargo +1.85` in the job that declares `MSRV: "1.85"`, and is no witness in a job that does not. What a
/// variable holds at run time, after an earlier step writes `$GITHUB_ENV` or an action exports one, is not read;
/// `BACKLOG.md` carries when that has to be looked at again. A script is a witness only when
/// every line of it is a simple command: one line the tokenizer declines makes the whole script contribute
/// nothing, since such a line can turn the lines after it into data. That can only report a Definition of
/// Done command missing, the direction that fails.
///
/// **An action is a command too, where it runs one.** `EmbarkStudios/cargo-deny-action` runs `cargo deny
/// <command> <command-arguments>` from its `with` inputs, and a step with no `command` input contributes
/// nothing rather than borrowing one from another mapping. A step setting `arguments` or `manifest-path`
/// contributes nothing either: those are passed before the command, so it runs against another manifest or
/// with other flags than the Definition of Done line — including where the value written is the action's own
/// default, which reports the line missing, the direction that fails.
fn ci_commands(ci: &str) -> Vec<Vec<String>> {
    let workflow = workflow::parse(ci).unwrap_or_else(|why| {
        panic!("the workflow cannot be read, so which commands CI runs is not known: {why}")
    });
    let mut commands = Vec::new();
    for job in &workflow.jobs {
        for step in &job.steps {
            if let Some(run) = &step.run {
                let env = workflow.env_for(job, step);
                // A declined script is no witness — deliberately, not a lost read: see the doc above.
                if let Some(read) = shell::script_commands(&run.value, &env) {
                    commands.extend(read);
                }
            }
            let runs_cargo_deny = step
                .uses
                .as_ref()
                .is_some_and(|uses| uses.value.starts_with("EmbarkStudios/cargo-deny-action@"));
            let input = |key: &str| step.with.iter().find(|input| input.key == key);
            // A step that sets what the action passes before the command — which manifest, which flags —
            // runs a command the Definition of Done does not spell, so it is no witness at all.
            let moves_the_prefix = input("arguments").is_some() || input("manifest-path").is_some();
            if runs_cargo_deny && !moves_the_prefix {
                if let Some(command) = input("command") {
                    let mut words = vec!["cargo".to_string(), "deny".to_string()];
                    words.extend(command.value.split_whitespace().map(str::to_string));
                    if let Some(arguments) = input("command-arguments") {
                        words.extend(arguments.value.split_whitespace().map(str::to_string));
                    }
                    commands.push(words);
                }
            }
        }
    }
    commands
}

/// The Definition of Done commands CI does not run, compared by argv.
///
/// A Definition of Done line is read by the same tokenizer with nothing in its environment, and one it cannot
/// read is refused rather than compared: a declaration this check cannot turn into words is not one it can
/// hold CI to.
fn missing_from_ci(agents: &str, ci: &str) -> Vec<String> {
    let ci = ci_commands(ci);
    dod_commands(agents)
        .into_iter()
        .filter(|command| {
            let words = shell::words(command, &BTreeMap::new()).unwrap_or_else(|| {
                panic!(
                    "the Definition of Done line `{command}` is not a command this check can read as words, \
                     so whether CI runs it cannot be decided"
                )
            });
            !ci.contains(&words)
        })
        .collect()
}

#[test]
fn local_dod_commands_exist_in_ci() {
    let Some(root) = workspace_root() else {
        return;
    };

    let agents_path = root.join("AGENTS.md");
    let ci_path = root.join(".github/workflows/ci.yml");

    let agents_content = std::fs::read_to_string(&agents_path).expect("read AGENTS.md");
    let ci_content = std::fs::read_to_string(&ci_path).expect("read ci.yml");

    let missing = missing_from_ci(&agents_content, &ci_content);

    assert!(
        missing.is_empty(),
        "Local Definition of Done contains commands missing from CI workflow:\n{}",
        missing.join("\n")
    );
}

/// A Definition of Done block that parses to zero commands must refuse rather than report clean.
///
/// The direction this gate's own zero-commands guard defends: a heading or fence-marker change that this
/// parser no longer recognizes would silently produce an empty command list, and `missing_from_ci` over an
/// empty list is vacuously satisfied — "every command CI runs" holding over no commands at all. Only a
/// comment line inside the fence (which `dod_commands` strips to nothing), never an absent block or a
/// present-but-empty one, exercises the same collapse a shape change would cause.
#[test]
fn a_dod_block_with_no_commands_is_refused_not_reported_clean() {
    let agents = "## Definition of Done\n\n```bash\n# only a comment, no command\n```\n";
    let refused = std::panic::catch_unwind(|| dod_commands(agents));
    assert!(
        refused.is_err(),
        "a Definition of Done block parsing to zero commands must panic, not silently pass \
         `missing_from_ci` over an empty corpus"
    );
}

#[test]
fn a_missing_supply_chain_action_leaves_cargo_deny_missing() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    let missing = missing_from_ci(agents, "jobs:\n  build:\n    steps: []\n");
    assert_eq!(missing, ["cargo deny check"]);
}

#[test]
fn cargo_deny_action_contributes_its_effective_command() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    let ci = "jobs:\n  supply-chain:\n    steps:\n      - uses: actions/checkout@v5\n      - name: policy\n        uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          command: check\n";
    assert!(missing_from_ci(agents, ci).is_empty());
}

#[test]
fn a_wrong_cargo_deny_action_command_does_not_satisfy_check() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    let ci = "jobs:\n  supply-chain:\n    steps:\n      - uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          command: advisories\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo deny check"]);
}

/// The action's other inputs are words of the same command, and one that narrows it is read.
///
/// Its `action.yml`, at the commit `ci.yml` pins, passes `--manifest-path <manifest-path> <arguments> <command>
/// <command-arguments>` to `cargo deny` — read from
/// `https://raw.githubusercontent.com/EmbarkStudios/cargo-deny-action/<pinned sha>/action.yml`, whose `args:`
/// list is that order — so `command: check` beside `command-arguments: advisories` runs the
/// advisories check alone. A reader of `command` alone recorded `cargo deny check` for it.
#[test]
fn an_action_input_narrowing_the_command_does_not_satisfy_check() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    for narrowing in [
        "command-arguments: advisories",
        "arguments: --exclude kanhe",
        "manifest-path: crates/xuanji/Cargo.toml",
    ] {
        let ci = format!(
            "jobs:\n  supply-chain:\n    steps:\n      - uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          command: check\n          {narrowing}\n"
        );
        assert_eq!(
            missing_from_ci(agents, &ci),
            ["cargo deny check"],
            "`{narrowing}` changes what the action runs, so it is no witness for `cargo deny check`"
        );
    }
}

#[test]
fn an_absent_action_command_does_not_borrow_a_command_from_another_mapping() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    let ci = "jobs:\n  supply-chain:\n    steps:\n      - uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          log-level: warn\n        env:\n          command: check\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo deny check"]);
}

/// A step reading a job-level pin is the Definition of Done's literal line with the value in it.
#[test]
fn a_job_env_pin_expands_into_the_run_lines_that_read_it() {
    let agents = format!(
        "## Definition of Done\n\n```bash\n{MARKER}=1 cargo +1.85 test --workspace --all-features\n```\n"
    );
    let ci = format!(
        "jobs:\n  msrv:\n    env:\n      MSRV: \"1.85\"\n    steps:\n      - run: {MARKER}=1 cargo \"+$MSRV\" test --workspace --all-features\n"
    );
    assert!(missing_from_ci(&agents, &ci).is_empty());
}

#[test]
fn a_job_env_pin_at_another_value_does_not_satisfy_the_literal() {
    let agents = format!(
        "## Definition of Done\n\n```bash\n{MARKER}=1 cargo +1.85 test --workspace --all-features\n```\n"
    );
    let ci = format!(
        "jobs:\n  msrv:\n    env:\n      MSRV: \"1.86\"\n    steps:\n      - run: {MARKER}=1 cargo \"+$MSRV\" test --workspace --all-features\n"
    );
    assert_eq!(
        missing_from_ci(&agents, &ci),
        [format!(
            "{MARKER}=1 cargo +1.85 test --workspace --all-features"
        )]
    );
}

/// A pin in one job is not in force in another: the line reading it there is no witness.
#[test]
fn a_pin_in_another_job_does_not_satisfy_the_literal() {
    let agents = format!(
        "## Definition of Done\n\n```bash\n{MARKER}=1 cargo +1.85 test --workspace --all-features\n```\n"
    );
    let ci = format!(
        "jobs:\n  pins:\n    env:\n      MSRV: \"1.85\"\n    steps:\n      - run: echo pinned\n  runs:\n    steps:\n      - run: {MARKER}=1 cargo \"+$MSRV\" test --workspace --all-features\n"
    );
    assert_eq!(
        missing_from_ci(&agents, &ci),
        [format!(
            "{MARKER}=1 cargo +1.85 test --workspace --all-features"
        )]
    );
}

/// One env name in two jobs is two scopes, and the join reads each where it is in force.
#[test]
fn one_env_name_in_two_jobs_is_read_in_each() {
    let agents = "## Definition of Done\n\n```bash\ncargo build --color always\n```\n";
    let ci = "jobs:\n  a:\n    env:\n      COLOR: always\n    steps:\n      - run: cargo build --color \"$COLOR\"\n  b:\n    env:\n      COLOR: never\n    steps:\n      - run: cargo build --color \"$COLOR\"\n";
    assert!(missing_from_ci(agents, ci).is_empty());
}

/// A line whose words are decided when it runs is no witness, however much of it matches.
///
/// Held for the contract rather than the change: a text join also reports this command missing, since the two
/// lines differ as text. What it pins is that the argv reader declines the line instead of guessing its words.
#[test]
fn a_line_decided_at_run_time_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo test --workspace\n```\n";
    let ci = "jobs:\n  j:\n    steps:\n      - run: cargo test --workspace $(extra_flags)\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo test --workspace"]);
}

/// A backslash continuation is one command, as the shell reads it.
#[test]
fn a_continued_line_is_one_command() {
    let agents = "## Definition of Done\n\n```bash\ncargo test --workspace --all-features\n```\n";
    let ci = "jobs:\n  j:\n    steps:\n      - run: |\n          cargo test \\\n            --workspace --all-features\n";
    assert!(missing_from_ci(agents, ci).is_empty());
}

/// A here-document's body is data handed to a command, not a command.
#[test]
fn a_here_document_body_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo test --workspace\n```\n";
    let ci = "jobs:\n  j:\n    steps:\n      - run: |\n          cat <<'EOF'\n          cargo test --workspace\n          EOF\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo test --workspace"]);
}

/// A string spanning lines is one word of one command, however its lines read on their own.
#[test]
fn a_line_inside_a_multi_line_string_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo test --workspace\n```\n";
    let ci = "jobs:\n  j:\n    steps:\n      - run: |\n          echo \"about to run\n          cargo test --workspace\n          \"\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo test --workspace"]);
}

/// A command after a line that ends the script is never reached, however readable both lines are.
#[test]
fn a_command_after_the_script_ends_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo test --workspace\n```\n";
    let ci = "jobs:\n  j:\n    steps:\n      - run: |\n          exit 0\n          cargo test --workspace\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo test --workspace"]);
}

/// A standalone assignment changes what the lines after it expand to, so the declared value is not the one used.
#[test]
fn a_script_reassigning_a_variable_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo build --color always\n```\n";
    let ci = "jobs:\n  j:\n    env:\n      COLOR: always\n    steps:\n      - run: |\n          COLOR=never\n          cargo build --color \"$COLOR\"\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo build --color always"]);
}

/// An append is an assignment too: `V+=v` changes what a later `"$V"` expands to.
#[test]
fn a_script_appending_to_a_variable_is_no_witness() {
    let agents = "## Definition of Done\n\n```bash\ncargo build --features a\n```\n";
    let ci = "jobs:\n  j:\n    env:\n      V: a\n    steps:\n      - run: |\n          V+=,b\n          cargo build --features \"$V\"\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo build --features a"]);
}
