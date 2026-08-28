//! Dogfood gate: every boundary family the composed shell publishes has an adopter-shaped owner.
//!
//! **Both sides are derived, and that is the whole design.** The requirement this holds once asked for "an
//! executable, reviewable inventory" mapping a published family set to its owners, and named the set in
//! prose — thirteen families, anchored to the `0.2.x` surface. A hand-kept list beside an enumerator is the
//! shape this repository keeps removing: a family added to the code keeps its old answer in the list, and
//! nobody re-examines it. So neither side is written down here.
//!
//! - The **families** are the boundary types `crates/tianheng/src/lib.rs` re-exports. That is one uniform
//!   source and it is the right one: the composed shell's public surface is exactly what an adopter can
//!   declare, and a standalone adopter reaches the same types through the dimension crates.
//! - The **owners** are the tracked files under `examples/` and this crate that name such a type — an
//!   isolated example workspace, or the repository's own self-law.
//!
//! # What is deliberately not a family
//!
//! `sans_io_pure` and `no_existential_leak` were listed beside the families and are **profiles**, not
//! families: `Constitution::sans_io_pure`'s own documentation says it is "a convenience over declaring the
//! two boundaries by hand; it adds no new reaction". A profile is a bundle, so covering the boundaries it
//! bundles covers it by construction, and counting it as a thirteenth family would have this gate assert
//! coverage of a reaction that does not exist.
//!
//! # What this does not claim
//!
//! That an owner exercises its family *well*. It asserts that a family is reachable from something an
//! adopter can read and run, which is what makes a family lose its owner visible. Whether the owner's
//! assertion is a good one is the owner's own test's job.
//!
//! # The residual: ownership is credited by type NAME, so a profile is invisible
//!
//! A profile constructs its boundaries internally — `Constitution::sans_io_pure` builds a
//! `ModuleBoundary` and an `AsyncExposureBoundary` from one `SansIoPure` — and a file declaring the family
//! that way never spells the type. This reader would not credit it.
//!
//! Live instance, and the reason this is stated rather than reasoned about: `examples/sans-io-pure` declares
//! async exposure in its `src/governance.rs` **through the profile**, and is credited here only because its
//! `tests/reaction.rs` happens to name `AsyncExposureBoundary` outright. Delete that one test line and the
//! family reads as unowned while the example still teaches it — a false refusal, in the direction this
//! reaction exists to make loud.
//!
//! Not closed by expanding the reader, because the honest closure is not a bigger name list: it is asking
//! what a profile *expands to*, which means evaluating constructor bodies rather than reading declarations.
//! Filed in `BACKLOG.md` as WATCH with the trigger that would change the answer.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/tianheng/src/lib.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every tracked path, so the corpora below come from the repository rather than from a walk of the worktree.
fn tracked(root: &Path) -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("run git ls-files");
    assert!(
        out.status.success(),
        "`git ls-files` failed, and a failed enumeration is not a repository with no files"
    );
    let files: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert!(
        !files.is_empty(),
        "no tracked file was enumerated, so both sides of this comparison would be empty and agree"
    );
    files
}

/// The executed part of a Rust line: everything before a `//` that begins a token.
///
/// **The rule is `kanhe::region`'s `cut_tail_comment`, replicated rather than called.** `kanhe` depends on
/// this crate, so reaching the other way is a cycle; lifting the rule into a crate both can see would put
/// text-region machinery on a published surface, which is the mistake `xingbiao::claim_scratch` was just
/// corrected for. Two implementations, one owner for the rule — and the owner is worth citing rather than
/// re-deriving, because the narrow condition is measured: cutting at the *first* `//` corrupts 26 lines in
/// this repository, including `"https://…"` constants and a string carrying `"/// …"`.
fn executed(line: &str) -> &str {
    let mut from = 0;
    while let Some(offset) = line[from..].find("//") {
        let at = from + offset;
        let begins_a_token = at == 0
            || line[..at]
                .chars()
                .next_back()
                .is_some_and(char::is_whitespace);
        if begins_a_token {
            return &line[..at];
        }
        from = at + "//".len();
    }
    line
}

/// Every `…Boundary` identifier in `text`'s **executed** lines, as whole words.
///
/// One recognizer for both sides, so the families and the owners cannot disagree about what a boundary type
/// is named. A second copy of this rule is exactly the drift the derivation exists to avoid.
///
/// **Comments are cut, which is what closes the class rather than one instance of it.** This read used to
/// take the raw blob, so any corpus file naming a family in a doc comment credited it — the gate's own file
/// did, and was excluded by a one-path denylist. A denylist is a second list that has to stay right; the
/// comment cut needs no list, and the exclusion is gone with it.
///
/// **Residue, stated rather than closed:** a Rust `/* … */` span is executed text to this reader, exactly as
/// it is to `kanhe::region` and for the same reason — the cut is the line-comment marker and nothing else.
/// A family named only inside a block comment in an owner file would still be credited. It errs toward
/// **over**-crediting, which is the direction that reports a family owned rather than refusing one that is.
fn boundary_types(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for token in text
        .lines()
        .flat_map(|line| executed(line).split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')))
    {
        if token.len() > "Boundary".len()
            && token.ends_with("Boundary")
            && token.starts_with(|c: char| c.is_ascii_uppercase())
        {
            found.insert(token.to_string());
        }
    }
    found
}

/// The families: the boundary types the composed shell re-exports.
fn families(root: &Path) -> BTreeSet<String> {
    let shell = root.join("crates/tianheng/src/lib.rs");
    let text = std::fs::read_to_string(&shell).unwrap_or_else(|err| {
        panic!("cannot read the composed shell's surface at {shell:?}: {err}")
    });
    let found = boundary_types(&text);
    assert!(
        !found.is_empty(),
        "the composed shell re-exports no boundary type, so this gate would hold over an empty family set — \
         the surface moved, and the derivation has to move with it rather than reporting clean"
    );
    found
}

/// Whether a tracked path is somewhere an adopter-shaped reaction can live.
///
/// An isolated example workspace, or this repository's own self-law. **Not** the dimension crates' internal
/// tests: the requirement is about a reaction an adopter could read and run, and `hunyi`'s unit tests are
/// neither an example nor self-governance.
///
/// **This file is in the corpus and that is now safe.** It sits under `crates/shengmo/`, and it names
/// boundary types throughout its own documentation — so while [`boundary_types`] read the raw blob, the gate
/// credited families to itself for *talking about them*. That was patched by excluding this one path, which
/// is a second list that has to stay right: any other corpus file could do the same. Cutting comments closes
/// the class, so the exclusion is gone rather than kept alongside.
fn is_owner_path(path: &str) -> bool {
    (path.starts_with("examples/") || path.starts_with("crates/shengmo/")) && path.ends_with(".rs")
}

/// Each family, and the owners naming it.
fn owners(root: &Path, files: &[String]) -> BTreeMap<String, BTreeSet<String>> {
    let mut owned: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut read = 0usize;
    for path in files.iter().filter(|p| is_owner_path(p)) {
        let Ok(text) = std::fs::read_to_string(root.join(path)) else {
            panic!(
                "cannot read tracked file '{path}' — a file this gate counts as an owner corpus must have \
                 been read, and skipping it would drop whatever family it owns"
            );
        };
        read += 1;
        // The owner is the example workspace or the self-law, not the file: two files of one example are one
        // owner, and naming the directory is what a refusal can be acted on.
        let owner = path
            .strip_prefix("examples/")
            .and_then(|rest| rest.split('/').next())
            .unwrap_or("self-law")
            .to_string();
        for family in boundary_types(&text) {
            owned.entry(family).or_default().insert(owner.clone());
        }
    }
    assert!(
        read > 0,
        "no owner file entered the corpus, so every family would report unowned and the refusal would be \
         about this gate rather than about the tree"
    );
    owned
}

/// Every published family is owned, and every owned family is published.
///
/// Both directions, because each catches what the other cannot. Without the first, a family added to the
/// shell reaches adopters with nothing demonstrating it. Without the second, an example can exercise a type
/// the shell does not publish and be counted as coverage of a surface no adopter has.
#[test]
fn every_published_family_has_an_adopter_shaped_owner() {
    let Some(root) = workspace_root() else {
        return;
    };
    let files = tracked(&root);
    let families = families(&root);
    let owned = owners(&root, &files);

    let unowned: Vec<&String> = families
        .iter()
        .filter(|f| !owned.contains_key(*f))
        .collect();
    assert!(
        unowned.is_empty(),
        "the composed shell publishes {} boundary famil(ies) that no example workspace and no self-law file \
         declares, so an adopter has nothing to read for them and nothing would notice if the reaction went: \
         {unowned:?}",
        unowned.len()
    );

    let unpublished: Vec<&String> = owned.keys().filter(|f| !families.contains(*f)).collect();
    assert!(
        unpublished.is_empty(),
        "these boundary types are exercised by an example or by the self-law and are not re-exported by the \
         composed shell, so the coverage they demonstrate is of a surface no adopter reaches: {unpublished:?}"
    );
}
