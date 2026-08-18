//! Repository check: no window of executed statements lives in two of this repository's own modules.
//!
//! The judgement, its calibration and its declared residue are `kanhe::twins`. This runs it over the real
//! corpus — the modules of the two crates that hold this repository's law and its record — because that is
//! the corpus the class was found in, twice, by review.

use std::path::{Path, PathBuf};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/kanhe/src/twins.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every tracked `.rs` module under the governed directories, as `(path, text)`.
///
/// **Tracked content, not the worktree.** A tree-wide check that walks the filesystem reads scratch files a
/// contributor has not added and misses nothing it should, which is the opposite of what `git ls-files`
/// gives it — and staging is what the Definition of Done already asks for.
///
/// Every failure to read is a refusal rather than a skip: a module this reader could not open is a module it
/// did not compare, and a corpus that quietly lost members answers "no window is shared" for the wrong reason.
fn corpus(root: &Path) -> Vec<(String, String)> {
    let listing = std::process::Command::new("git")
        .args(["ls-files", "crates/kanhe/src", "crates/shengmo/src"])
        .current_dir(root)
        .output()
        .expect("git must be runnable to enumerate the modules this check compares");
    assert!(
        listing.status.success(),
        "`git ls-files` did not enumerate the corpus: {}",
        String::from_utf8_lossy(&listing.stderr)
    );
    String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter(|path| path.ends_with(".rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
                panic!("cannot read the tracked module {path} ({err}); a module this check could not open \
                        is one it did not compare, which is not a module without a twin")
            });
            (path.to_string(), text)
        })
        .collect()
}

/// One implementation may not live in two modules.
#[test]
fn no_window_of_executed_statements_lives_in_two_modules() {
    let Some(root) = workspace_root() else {
        return;
    };
    let sources = corpus(&root);
    // The corpus size travels into the diagnostic, because `judge` reporting clean over a collapsed
    // enumeration is the failure this check would be least able to see from its own output.
    match kanhe::twins::judge(&sources) {
        Ok(report) => eprintln!("{report}"),
        Err(refusal) => panic!(
            "twins ({:?}) over {} tracked module(s): {}",
            refusal.kind,
            sources.len(),
            refusal.message
        ),
    }
}
