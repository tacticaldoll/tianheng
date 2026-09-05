//! Repository check: the two enumerations of this workspace's members agree.
//!
//! `cargo` names its members in `[workspace] members`; this repository's release gate reaches them by
//! walking `crates/*/Cargo.toml`. Both answer *which crates are the family*, and nothing has ever asked
//! whether they answer the same. They do, and this direction is what says so: the agreement is a premise
//! holding by layout rather than by declaration, and layout is not a thing anyone declared. The figure is
//! the run's, not this sentence's.
//!
//! **The direction that matters is the false negative.** A member declared outside `crates/` is invisible to
//! the walk: it would never be held to `version.workspace = true`, never enter the family the catalog's pins
//! are judged against, and never be read in the lock. Its stale pin would reach `cargo publish` through a
//! subject that never contained it — the same shape as a family crate offered without a `path`, one layer up,
//! in the **enumerator** rather than in the selector.
//!
//! The other direction is a false refusal and is asserted too: a directory under `crates/` carrying a
//! manifest that `[workspace] members` does not name is judged as a member cargo does not build.
//!
//! **Here rather than inside `judge`.** The gate's phases are a sequence whose order is observable, and its
//! failure matrix asserts which refusal a repository meets first; adding a `cargo metadata` call to that
//! sequence would move that order for every repository it judges. This asks the question of **this**
//! repository, which is the only one whose layout the premise is about.

use std::collections::BTreeSet;
use std::path::PathBuf;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// `manifest` with `root` removed, spelled the way the gate this direction compares against spells it.
///
/// **Taken through the one owner rather than restated.** This file's first draft wrote the strip and the
/// `/` join out again, which made it a third spelling of the identity — and the side it compares against
/// was the one that disagreed. Two readers of one question is the shape this comparison exists to catch;
/// writing a third here would have been it again, one level up.
fn relative_to(manifest: &str, root: &std::path::Path) -> String {
    match kanhe::repository_path::repository_path(root, std::path::Path::new(manifest)) {
        kanhe::repository_path::RepositoryPath::Below(path) => path,
        kanhe::repository_path::RepositoryPath::Outside => panic!(
            "cargo reported member manifest {manifest} outside the workspace root {} it reported \
             alongside it, so the two enumerations describe different trees",
            root.display()
        ),
    }
}

#[test]
fn cargos_members_and_the_walked_directories_are_one_set() {
    // The marker is read by `workspace::locate` through `marker_set()`, which is the one place it is
    // spelled — a second spelling here is what `one_spelling` refuses, and it refused this file's first
    // draft for exactly that.
    let Some(root) = workspace_root() else {
        return;
    };

    // Cargo's own answer. `--no-deps` lists members only, and `manifest_path` is each member's own manifest,
    // which is what makes the two sides comparable without deriving a directory from a package name — a
    // derivation the machinery reader records having already got wrong.
    let metadata = kanhe::release_coherence_gate::cargo_metadata(&root)
        .expect("cargo describes the workspace this check runs in");
    // Both sides spell the identity through `kanhe::repository_path`, which is what makes them comparable
    // at all: they are compared as strings, so a difference in how each renders a separator is a difference
    // in every member. That was live — this side joined components with `/` while the gate's own walk used
    // `Path::display`, the host's separator — and it is why the spelling has an owner.
    let cargo_root = std::path::Path::new(
        metadata["workspace_root"]
            .as_str()
            .expect("cargo reports the root of the tree it just described"),
    );
    let declared: BTreeSet<String> = metadata["packages"]
        .as_array()
        .expect("cargo reports an array of packages")
        .iter()
        .map(|package| {
            relative_to(
                package["manifest_path"]
                    .as_str()
                    .expect("every member carries its own manifest path"),
                cargo_root,
            )
        })
        .collect();

    // The gate's answer, taken through the gate's own reader rather than restated here — two readers of one
    // question is the shape this comparison exists to catch, and writing a third would be it again.
    let walked: BTreeSet<String> = kanhe::release_coherence_gate::workspace_manifests(&root)
        .expect("the crate directories enumerate")
        .into_iter()
        .map(|(path, _)| path)
        .collect();

    let unwalked: Vec<&String> = declared.difference(&walked).collect();
    let undeclared: Vec<&String> = walked.difference(&declared).collect();

    assert!(
        unwalked.is_empty(),
        "cargo names {} member(s) the release gate's walk does not reach: {:?}. A member outside `crates/` \
         is held to no inherited version, enters no family the catalog's pins are judged against, and is \
         read in no lock — its stale pin would reach `cargo publish` through a subject that never contained \
         it. Either move it under `crates/`, or make the gate read `[workspace] members`",
        unwalked.len(),
        unwalked
    );
    assert!(
        undeclared.is_empty(),
        "the release gate's walk reaches {} manifest(s) `[workspace] members` does not name: {:?}. The gate \
         would hold a crate cargo does not build to this workspace's version",
        undeclared.len(),
        undeclared
    );
    assert!(
        !declared.is_empty(),
        "cargo named no members at all, so this comparison would report agreement over nothing"
    );
}
