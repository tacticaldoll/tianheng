//! Self-governance reaction: whitespace hygiene across every tracked text file.
//!
//! Asserts that tracked text files carry no trailing whitespace on any line,
//! no blank line at end of file, and end with a single newline character.

use std::path::PathBuf;
use std::process::Command;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

#[test]
fn whitespace_hygiene_across_tracked_text_files() {
    let Some(root) = workspace_root() else {
        return;
    };

    let output = Command::new("git")
        .args(["ls-files", "--eol"])
        .current_dir(&root)
        .output()
        .expect("git ls-files --eol should succeed");

    assert!(
        output.status.success(),
        "git ls-files --eol failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut offenses = Vec::new();

    for line in stdout.lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() < 2 {
            continue;
        }
        let eol_info = parts[0];
        let path_str = parts[1];

        // Skip binary files as classified by git
        if eol_info.starts_with("i/-text") {
            continue;
        }

        let file_path = root.join(path_str);
        if !file_path.is_file() {
            continue;
        }

        let content = match std::fs::read(&file_path) {
            Ok(bytes) => bytes,
            Err(_) => continue,
        };

        if content.is_empty() {
            continue;
        }

        // Check final newline
        if content.last() != Some(&b'\n') {
            offenses.push(format!("{path_str}: missing final newline"));
        }

        let text = String::from_utf8_lossy(&content);
        let lines: Vec<&str> = text.lines().collect();

        // Check trailing whitespace on lines
        for (idx, line_str) in lines.iter().enumerate() {
            let normalized = line_str.strip_suffix('\r').unwrap_or(line_str);
            if normalized.ends_with(' ') || normalized.ends_with('\t') {
                offenses.push(format!("{path_str}:{}: trailing whitespace", idx + 1));
            }
        }

        // Check blank line at end of file
        if text.ends_with("\n\n") || text.ends_with("\r\n\r\n") {
            offenses.push(format!("{path_str}: blank line at end of file"));
        }
    }

    assert!(
        offenses.is_empty(),
        "Whitespace hygiene offenses found:\n{}",
        offenses.join("\n")
    );
}
