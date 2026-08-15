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

use std::path::PathBuf;

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
            let command = line.split('#').next().unwrap_or("").trim();
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

fn cargo_deny_action_commands(ci: &str) -> Vec<String> {
    let lines: Vec<&str> = ci.lines().collect();
    let mut commands = Vec::new();
    let mut start = 0;

    while start < lines.len() {
        let step_indent = lines[start].len() - lines[start].trim_start().len();
        if !lines[start].trim_start().starts_with("- ") {
            start += 1;
            continue;
        }

        let mut end = start + 1;
        while end < lines.len() {
            let line = lines[end];
            let indent = line.len() - line.trim_start().len();
            let trimmed = line.trim_start();
            if (!trimmed.is_empty() && indent < step_indent)
                || (indent == step_indent && trimmed.starts_with("- "))
            {
                break;
            }
            end += 1;
        }

        let step = &lines[start..end];
        let is_cargo_deny = step.iter().any(|line| {
            line.trim()
                .strip_prefix("- ")
                .unwrap_or(line.trim())
                .strip_prefix("uses: ")
                .is_some_and(|action| action.starts_with("EmbarkStudios/cargo-deny-action@"))
        });
        if is_cargo_deny {
            if let Some((with_index, with_indent)) =
                step.iter().enumerate().find_map(|(index, line)| {
                    let indent = line.len() - line.trim_start().len();
                    (line.trim() == "with:").then_some((index, indent))
                })
            {
                let with_body = &step[with_index + 1..];
                let with_end = with_body
                    .iter()
                    .position(|line| {
                        let indent = line.len() - line.trim_start().len();
                        !line.trim().is_empty() && indent <= with_indent
                    })
                    .unwrap_or(with_body.len());
                if let Some(command) = with_body[..with_end].iter().find_map(|line| {
                    line.trim()
                        .strip_prefix("command: ")
                        .map(|value| value.trim().trim_matches(['\'', '"']))
                }) {
                    if !command.is_empty() {
                        commands.push(format!("cargo deny {command}"));
                    }
                }
            }
        }
        start = end;
    }

    commands
}

fn missing_from_ci(agents: &str, ci: &str) -> Vec<String> {
    let mut ci_effective: Vec<String> = ci
        .lines()
        .map(|line| {
            let line = line.trim();
            let line = line.strip_prefix("- ").unwrap_or(line);
            let line = line.strip_prefix("run: ").unwrap_or(line);
            line.trim().to_string()
        })
        .collect();
    ci_effective.extend(cargo_deny_action_commands(ci));

    dod_commands(agents)
        .into_iter()
        .filter(|command| !ci_effective.iter().any(|ci_line| ci_line == command))
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

#[test]
fn an_absent_action_command_does_not_borrow_a_command_from_another_mapping() {
    let agents = "## Definition of Done\n\n```bash\ncargo deny check\n```\n";
    let ci = "jobs:\n  supply-chain:\n    steps:\n      - uses: EmbarkStudios/cargo-deny-action@v2\n        with:\n          log-level: warn\n        env:\n          command: check\n";
    assert_eq!(missing_from_ci(agents, ci), ["cargo deny check"]);
}
