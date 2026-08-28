//! Collation: authored text against the law it describes.
//!
//! The membership of a declared allowlist belongs to `shengmo::law` and its projection. A second copy in a
//! comment or a document is a source of truth that goes stale with no check to say so, and the repair is
//! always the same — point at `AGENTS.self-law.md` instead of restating what it renders.
//!
//! This check read **one** crate's line comments against **one** dimension's allowlist until this change.
//! Measured, `PROJECT.md` named `serde_json`, `xuanji` and `xingbiao` — every member of 圭表's live
//! allowlist — in a file class nothing scanned. A rule enforced at one site and not its neighbour is a rule
//! about the site.
//!
//! Generated documents are excluded, and not as a convenience: a projection names every member of every
//! allowlist because rendering them is its job.

use std::path::PathBuf;

use kanhe::region::{Source, declares_itself_generated};
use kanhe::restatement::{
    assert_comment_block_does_not_copy_allowlist, comment_restates_the_declaration,
};
use shengmo::law::{constitution, shell_dependency_allowlist, shell_dependency_boundary};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("PROJECT.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Authored shell comments may explain the dependency boundary, but the live declaration and its
/// generated projection own the membership. The declaration token and a full member census are distinct
/// copied forms; both are refused without forbidding product code from legitimately calling the public DSL.
#[test]
fn shell_comments_do_not_restate_the_dependency_allowlist() {
    let Some(root) = workspace_root() else {
        return; // outside a checkout — the authored repository source is not present
    };
    let shell_boundary = shell_dependency_boundary();
    let allowlist = shell_dependency_allowlist(&shell_boundary);
    let mut pending = vec![root.join("crates/tianheng/src")];
    let mut rust_sources = Vec::new();

    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
        {
            let entry = entry.unwrap_or_else(|error| {
                panic!("cannot enumerate {}: {error}", directory.display())
            });
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                rust_sources.push(path);
            }
        }
    }

    rust_sources.sort();
    for source in rust_sources {
        let text = std::fs::read_to_string(&source)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", source.display()));
        let mut comment_block = String::new();
        let mut block_start = 0usize;
        for (index, line) in text.lines().enumerate() {
            if line.trim_start().starts_with("//") {
                if comment_block.is_empty() {
                    block_start = index + 1;
                }
                assert!(
                    !comment_restates_the_declaration(line),
                    "{}:{} names the shell dependency declaration in a comment; refer to \
                     AGENTS.self-law.md instead. This reads a comment's text and not its purpose, so it \
                     also refuses a doc example of the DSL — a declared false refusal of \
                     `self-law-projection`, not a case to work around",
                    source.display(),
                    index + 1
                );
                comment_block.push_str(line);
                comment_block.push('\n');
            } else if !comment_block.is_empty() {
                assert_comment_block_does_not_copy_allowlist(
                    &source,
                    block_start,
                    &comment_block,
                    allowlist,
                );
                comment_block.clear();
            }
        }
        if !comment_block.is_empty() {
            assert_comment_block_does_not_copy_allowlist(
                &source,
                block_start,
                &comment_block,
                allowlist,
            );
        }
    }

    let style_source = root.join("crates/tianheng/src/runner/term_color.rs");
    let style_text = std::fs::read_to_string(&style_source)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", style_source.display()));
    assert!(
        style_text.lines().any(|line| {
            line.trim_start().starts_with("//") && line.contains("AGENTS.self-law.md")
        }),
        "{} must direct its dependency rationale to the generated self-law projection",
        style_source.display()
    );
}

/// Every tracked governance document, against every declared allowlist.
///
/// Records are excluded because a record is not a claim about current law: `docs/history/` and
/// `CHANGELOG.md` describe what happened, and an active `openspec/changes/` plan describes what is being
/// proposed. Generated documents are excluded because rendering the membership is their whole job.
#[test]
fn no_governance_document_restates_a_declared_allowlist() {
    let Some(root) = workspace_root() else {
        return;
    };
    let out = std::process::Command::new("git")
        .args(["ls-files", "*.md"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    assert!(
        out.status.success(),
        "could not enumerate tracked Markdown; a failed enumeration is not an empty corpus"
    );
    let listing = String::from_utf8_lossy(&out.stdout).to_string();
    let paths: Vec<&str> = listing.lines().filter(|p| !p.is_empty()).collect();
    assert!(
        !paths.is_empty(),
        "no tracked Markdown was enumerated, so this check would report clean over nothing"
    );

    let allowlists =
        kanhe::restatement::allowlists(constitution().static_boundaries().boundaries());
    assert!(
        !allowlists.is_empty(),
        "the live constitution declares no dependency allowlist, so this check compares against nothing"
    );

    let mut offences = Vec::new();
    let mut read = 0usize;
    for path in paths {
        if path.starts_with("docs/history/")
            || path.starts_with("openspec/changes/")
            || path == "CHANGELOG.md"
        {
            continue;
        }
        let text = std::fs::read_to_string(root.join(path))
            .unwrap_or_else(|err| panic!("cannot read tracked {path}: {err}"));
        let source = Source::of(text);
        if declares_itself_generated(&source.header()) {
            continue;
        }
        read += 1;
        offences.extend(kanhe::restatement::document_offences(
            path,
            source.prose(),
            &allowlists,
        ));
    }
    assert!(read > 0, "no governance document was read");
    assert!(
        offences.is_empty(),
        "a governance document restates a declaration the law owns:\n{}",
        offences.join("\n")
    );
}
