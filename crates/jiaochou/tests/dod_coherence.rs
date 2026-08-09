//! Self-governance reaction: Definition of Done coherence between AGENTS.md and .github/workflows/ci.yml.
//!
//! Asserts that every command listed in AGENTS.md's Definition of Done block appears
//! in .github/workflows/ci.yml so local pre-flight gates remain a strict subset of CI.

use std::path::PathBuf;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("AGENTS.md").is_file() && root.join(".github/workflows/ci.yml").is_file(),
        shengmo::workspace::marker_set(),
    )
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

    // Extract Definition of Done fenced block
    let mut in_dod = false;
    let mut in_code_block = false;
    let mut dod_lines = Vec::new();

    for line in agents_content.lines() {
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
            dod_lines.push(line);
        }
    }

    assert!(
        !dod_lines.is_empty(),
        "No commands found in AGENTS.md Definition of Done code block"
    );

    let ci_normalized: Vec<String> = ci_content
        .lines()
        .map(|l| {
            let s = l.trim();
            let s = s.strip_prefix("- ").unwrap_or(s);
            let s = s.strip_prefix("run: ").unwrap_or(s);
            s.trim().to_string()
        })
        .collect();

    let mut missing = Vec::new();

    for raw_line in dod_lines {
        let code_part = raw_line.split('#').next().unwrap_or("").trim();
        if code_part.is_empty() || code_part == "cargo deny check" {
            continue;
        }

        let is_present = ci_normalized.iter().any(|ci_line| ci_line == code_part);
        if !is_present {
            missing.push(code_part.to_string());
        }
    }

    assert!(
        missing.is_empty(),
        "Local Definition of Done contains commands missing from CI workflow:\n{}",
        missing.join("\n")
    );
}
