//! Self-governance reaction: reference integrity across tracked markdown and rust files.
//!
//! Asserts that every governance document exists and that markdown links and relative file
//! references in tracked `*.md` and `*.rs` files point to existing tracked content.

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("Cargo.toml").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "Cargo.toml expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

#[test]
fn governance_documents_exist() {
    let Some(root) = workspace_root() else {
        return;
    };

    let required = [
        "AGENTS.md",
        "AGENTS.self-law.md",
        "BACKLOG.md",
        "CHANGELOG.md",
        "COOKBOOK.md",
        "PROJECT.md",
        "README.md",
        "Cargo.toml",
        "deny.toml",
    ];

    for doc in required {
        let path = root.join(doc);
        assert!(
            path.is_file(),
            "Required governance document {doc} is missing in workspace root"
        );
    }
}

#[test]
fn in_repository_references_resolve() {
    let Some(root) = workspace_root() else {
        return;
    };

    let output = Command::new("git")
        .args(["ls-files"])
        .current_dir(&root)
        .output()
        .expect("git ls-files should succeed");

    assert!(output.status.success());
    let tracked_files: HashSet<String> = String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(|s| s.to_string())
        .collect();

    let mut offenses = Vec::new();

    for rel_path in &tracked_files {
        if !rel_path.ends_with(".md") && !rel_path.ends_with(".rs") {
            continue;
        }

        let full_path = root.join(rel_path);
        if !full_path.is_file() {
            continue;
        }

        let content = match std::fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        // Check markdown link targets: ](target)
        let mut idx = 0;
        while let Some(start) = content[idx..].find("](") {
            let target_start = idx + start + 2;
            if let Some(end) = content[target_start..].find(')') {
                let raw_target = &content[target_start..target_start + end];
                idx = target_start + end + 1;

                // Split off anchor (#...)
                let target = raw_target.split('#').next().unwrap_or(raw_target);
                if target.is_empty()
                    || target.starts_with("http://")
                    || target.starts_with("https://")
                    || target.starts_with("mailto:")
                {
                    continue;
                }

                // Ignore rustdoc symbol links (e.g. Self::foo, crate::Bar, Module::Item, bare_ident without extension or slash)
                if !target.contains('/') && !target.contains('.') {
                    continue;
                }
                if target.contains("::") {
                    continue;
                }

                // Clean file:// scheme if present
                let clean_target = if target.starts_with("file:///") {
                    // Extract path after file://
                    let p = target.strip_prefix("file://").unwrap_or(target);
                    // If it's an absolute path from another environment (/home/qaz/work/tianheng/...)
                    if let Some(pos) = p.find("/tianheng/") {
                        &p[pos + "/tianheng/".len()..]
                    } else if let Some(last_slash) = p.rfind('/') {
                        &p[last_slash + 1..]
                    } else {
                        p
                    }
                } else {
                    target.strip_prefix("file://").unwrap_or(target)
                };

                // Resolve relative path
                let parent = Path::new(rel_path)
                    .parent()
                    .unwrap_or_else(|| Path::new(""));
                let resolved = if clean_target.starts_with('/') {
                    PathBuf::from(clean_target.trim_start_matches('/'))
                } else {
                    parent.join(clean_target)
                };

                // Normalize path components (remove . and ..)
                let mut components = Vec::new();
                for comp in resolved.components() {
                    match comp {
                        std::path::Component::Normal(c) => {
                            components.push(c.to_string_lossy().to_string())
                        }
                        std::path::Component::ParentDir => {
                            components.pop();
                        }
                        _ => {}
                    }
                }
                let norm_str = components.join("/");

                if norm_str.is_empty() {
                    continue;
                }

                // Verify resolved path exists in tracked_files, root, or as a directory
                let exists_file = tracked_files.contains(&norm_str);
                let exists_dir = tracked_files
                    .iter()
                    .any(|f| f.starts_with(&format!("{norm_str}/")));
                let exists_disk = root.join(&norm_str).exists() || root.join(clean_target).exists();

                if !exists_file && !exists_dir && !exists_disk {
                    offenses.push(format!(
                        "{rel_path}: markdown link target '{raw_target}' -> '{norm_str}' not found"
                    ));
                }
            } else {
                break;
            }
        }
    }

    assert!(
        offenses.is_empty(),
        "Stale reference integrity offenses found:\n{}",
        offenses.join("\n")
    );
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-reference-integrity-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
