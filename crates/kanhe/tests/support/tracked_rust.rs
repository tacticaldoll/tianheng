//! The reading every repository check over tracked Rust shares: a tri-state answer for a file, the string
//! literal's value, the workspace locator, and the loop that reads **every** tracked `.rs` file and refuses to
//! report over one it could not decide.
//!
//! Extracted from `hermetic_invocations` so the check that holds who constructs a scratch root decides the
//! way the check that holds who constructs a `git` does, rather than writing the loop a second time.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// What this reader could decide about a file.
///
/// **Three states, because a file it cannot parse is not a file that constructs nothing.** The tokeniser's
/// failure arm fell back to an exact substring, which answers `false` for a construction split across lines
/// — so an unparseable file carrying one was reported clean, silently, which is the one direction the Core
/// Contract forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reading {
    Constructs,
    DoesNot,
    Undecidable,
}

/// A string literal's **value**, not its rendering.
///
/// `Literal::to_string` gives back the source spelling, so `r"git"` and `"\x67it"` — both of which decode to
/// `git` — compared unequal to `"git"`, and a construction written either way was not read.
pub fn literal_value(literal: &syn::Lit) -> Option<String> {
    match literal {
        syn::Lit::Str(text) => Some(text.value()),
        _ => None,
    }
}

/// That expression, where it is a string literal.
pub fn string_of(expression: &syn::Expr) -> Option<String> {
    match expression {
        syn::Expr::Lit(literal) => literal_value(&literal.lit),
        _ => None,
    }
}

pub fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/kanhe/src/hermetic_git.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// The tracked Rust files `read` answers `Constructs` for, every one read and decided; `what` names the
/// question in a refusal.
pub fn files_where(
    root: &std::path::Path,
    what: &str,
    read: impl Fn(&str) -> Reading,
) -> BTreeSet<String> {
    let tracked = kanhe::hermetic_git::tracked_paths(root, &["*.rs"]).expect(
        "the tracked Rust is enumerable; a failed enumeration is not a repository with no sources",
    );
    let mut constructing = BTreeSet::new();
    let mut undecidable = Vec::new();
    let mut examined = 0usize;
    for path in &tracked {
        let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {err}"
            )
        });
        examined += 1;
        match read(&text) {
            Reading::Constructs => {
                constructing.insert(path.clone());
            }
            Reading::DoesNot => {}
            Reading::Undecidable => undecidable.push(path.clone()),
        }
    }
    assert!(
        examined > 0,
        "no tracked Rust was inspected, so this check would report clean over nothing"
    );
    // A file this reader cannot parse is not a file that constructs nothing. Its predecessor fell back to a
    // substring, which answers `false` for a construction split across lines — clean, silently, over a file
    // it never read.
    assert!(
        undecidable.is_empty(),
        "tracked Rust this reader could not decide, so whether it {what} was never answered:\n  {}",
        undecidable.join("\n  ")
    );
    constructing
}
