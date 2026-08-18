//! Repository check: a token with a constant owner is not spelled again as a literal inside its reach.
//!
//! Two constants exist because one token had to have one owner — `kanhe::region::DO_NOT_EDIT`, the marker a
//! generated document declares itself with, and `shengmo::workspace::MARKER`, the variable saying a run must
//! find a repository. Both were then written out again as literals in every module that could reach them,
//! which is the shape `verdict_channel` closed between a shell script and Rust and which had stayed open
//! between Rust and Rust.
//!
//! **The corpus is what can reach the constant, and nothing wider.** `MARKER` is spelled out in `tianheng`,
//! `louke` and `xuanji` — published crates that cannot depend on `shengmo`, because `shengmo` depends on
//! `tianheng` and the edge would close a cycle — and `DO_NOT_EDIT` in `shengmo`'s own law projection header,
//! which cannot reach `kanhe`. Those are facts about the dependency graph rather than sites anyone declined
//! to fix, so they sit outside this check's subject rather than inside it as exceptions. No count is given:
//! nothing enumerates that set, so a figure here would be a census with no producer.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kanhe::region::{DO_NOT_EDIT, Source};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/kanhe/src/region.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every workspace member that can reach `owner`'s crate — the crate itself and everything depending on it,
/// directly or through another member.
///
/// **The list this holds is not removed, it is given an adversary.** `repository-checks` requires a constant a
/// check judges by to be compared with whatever enumerates its set, **in both directions**, and its own text
/// records why: a one-directional comparison catches an omission and misses an entry that has outlived its
/// subject. Measured elsewhere in this repository — removing a member from a hand-kept dimension list left a
/// coverage assertion green because the assertion filtered on the literal.
///
/// Path dependencies are what the family declares between members, so the manifests are the graph.
fn members_reaching(root: &Path, crate_name: &str) -> BTreeSet<String> {
    let manifest =
        std::fs::read_to_string(root.join("Cargo.toml")).expect("the workspace manifest");
    let members: Vec<String> = manifest
        .lines()
        .find(|line| line.trim_start().starts_with("members = ["))
        .expect("the workspace manifest declares its members on one line")
        .split('"')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect();
    assert!(
        members.len() > 2,
        "read {} workspace member(s), which is not a graph this reader can be about",
        members.len()
    );
    let mut edges: Vec<(String, String)> = Vec::new();
    for member in &members {
        let text = std::fs::read_to_string(root.join(member).join("Cargo.toml"))
            .unwrap_or_else(|err| panic!("cannot read {member}/Cargo.toml ({err})"));
        let name = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("name = "))
            .map(|value| value.trim().trim_matches('"').to_string())
            .unwrap_or_else(|| panic!("{member}/Cargo.toml declares no package name"));
        for other in &members {
            let dep = other.rsplit('/').next().unwrap_or(other);
            if dep != name && text.contains(&format!("path = \"../{dep}\"")) {
                edges.push((name.clone(), dep.to_string()));
            }
        }
    }
    let mut reaching: BTreeSet<String> = BTreeSet::from([crate_name.to_string()]);
    loop {
        let grown: BTreeSet<String> = edges
            .iter()
            .filter(|(_, to)| reaching.contains(to))
            .map(|(from, _)| from.clone())
            .collect();
        let before = reaching.len();
        reaching.extend(grown);
        if reaching.len() == before {
            break;
        }
    }
    reaching
}

/// Every tracked `.rs` file under `dirs`, as `(path, text)`.
fn tracked(root: &Path, dirs: &[&str]) -> Vec<(String, String)> {
    let listing = std::process::Command::new("git")
        .args(["ls-files"])
        .args(dirs)
        .current_dir(root)
        .output()
        .expect("git must be runnable to enumerate the corpus a constant reaches");
    assert!(
        listing.status.success(),
        "`git ls-files` did not enumerate {dirs:?}: {}",
        String::from_utf8_lossy(&listing.stderr)
    );
    let files: Vec<(String, String)> = String::from_utf8_lossy(&listing.stdout)
        .lines()
        .filter(|path| path.ends_with(".rs"))
        .map(|path| {
            let text = std::fs::read_to_string(root.join(path))
                .unwrap_or_else(|err| panic!("cannot read the tracked file {path} ({err})"));
            (path.to_string(), text)
        })
        .collect();
    // The enumeration is an input like any other: a corpus that collapsed to nothing satisfies "no file
    // spells it" exactly as a clean one does.
    assert!(
        files.len() > 5,
        "read {} tracked Rust file(s) under {dirs:?}, which is not a corpus this property can be about",
        files.len()
    );
    files
}

/// A token with a constant owner is spelled once inside the reach of that constant.
///
/// **Executed text, not the whole file.** A doc comment naming the marker in prose is a reader being told
/// what the constant is, not a second owner of it — and this check's own module header names both tokens.
/// `repository-checks` requires a check deciding a property over executed text to take its corpus from
/// `kanhe::region`, and this is the direction that holds it rather than the requirement being satisfied by
/// the import.
#[test]
fn a_token_with_a_constant_owner_has_no_second_spelling_in_reach() {
    let Some(root) = workspace_root() else {
        return;
    };
    for (constant, value, owner, owning_crate, dirs) in [
        (
            "kanhe::region::DO_NOT_EDIT",
            DO_NOT_EDIT,
            "crates/kanhe/src/region.rs",
            "kanhe",
            &["crates/kanhe"][..],
        ),
        (
            "shengmo::workspace::MARKER",
            shengmo::workspace::MARKER,
            "crates/shengmo/src/workspace.rs",
            "shengmo",
            &["crates/kanhe", "crates/shengmo"][..],
        ),
    ] {
        // The declared corpus, against what the manifests produce — both directions.
        let declared: BTreeSet<String> = dirs
            .iter()
            .map(|dir| dir.rsplit('/').next().unwrap_or(dir).to_string())
            .collect();
        let reaching = members_reaching(&root, owning_crate);
        assert_eq!(
            declared, reaching,
            "`{constant}`'s declared corpus and the members the manifests say can reach `{owning_crate}` \
             disagree. A member added to the graph and not to the list goes unchecked; one left in the list \
             after it stopped depending has outlived its subject"
        );

        // **Every occurrence, the owner's included.** Skipping the owner FILE exempted more than the owner
        // DECLARATION: a second constant carrying the same value beside it read as clean, which is the
        // corpus-narrower-than-the-claim shape this check exists to refuse.
        let mut sites = Vec::new();
        for (path, text) in tracked(&root, dirs) {
            for (number, line) in Source::of(text.clone()).rust().numbered_lines() {
                if line.contains(value) {
                    sites.push(format!("{path}:{number}"));
                }
            }
        }
        assert_eq!(
            sites.len(),
            1,
            "`{value}` is spelled {} time(s) inside `{constant}`'s reach; exactly one may exist and it is the \
             declaration itself:\n  {}",
            sites.len(),
            sites.join("\n  ")
        );
        assert!(
            sites[0].starts_with(owner),
            "the one spelling of `{value}` is at {} rather than in its owner {owner}",
            sites[0]
        );
    }
}
