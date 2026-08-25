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
        let name = package_name(&text)
            .unwrap_or_else(|| panic!("{member}/Cargo.toml declares no package name"));
        for dep in declared_dependencies(&text) {
            if dep != name
                && members
                    .iter()
                    .any(|m| m.rsplit_once('/').map_or(m.as_str(), |(_, name)| name) == dep)
            {
                edges.push((name.clone(), dep));
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

/// The `[package]` name a manifest declares, read from executed TOML.
fn package_name(text: &str) -> Option<String> {
    let source = Source::of(text);
    let mut inside = false;
    for line in source.toml().lines() {
        let trimmed = line.trim();
        if trimmed == "[package]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("name") {
            if let Some(value) = rest.trim_start().strip_prefix('=') {
                return Some(value.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Every crate a manifest declares a dependency on, across the table forms cargo admits.
///
/// **Two readers preceded this one and each was a guess at a layout.** The first recognised an edge by
/// `text.contains("path = \"../{dep}\"")`, so `path="../x"` without the spaces was invisible. The second
/// read `key = value` lines and looked for the word `package` anywhere in the value — which missed
/// `[dependencies.alias]`, where the heading names the dependency and no key does, and **deleted** a real
/// edge from `{ path = "…", features = ["package"] }`, because the word appears as a feature name and the
/// rename branch then found no `=` after it. An edge lost on both sides of a two-way comparison is a corpus
/// that shrinks while agreeing with itself.
///
/// So the table is read as a table. A heading ending in `dependencies` opens a table whose **keys** are
/// dependencies; a heading carrying `dependencies.` opens **one** dependency's own table, named by the
/// heading unless a `package` key inside renames it. A `package` is recognised as a key of the inline table,
/// never as a substring of its value.
///
/// The corpus is `crate::region`'s TOML region, so a commented-out dependency is not a dependency and a `#`
/// inside a string is not a comment — the same reader every other manifest question in this repository uses.
fn declared_dependencies(text: &str) -> Vec<String> {
    /// Which table the scan stands in.
    enum Table {
        /// Not a dependency table.
        Other,
        /// A dependency table: every key is a dependency.
        Keys,
        /// One dependency's own table, named by its heading until a `package` key renames it.
        One(String),
    }

    /// The value of a `package` **key**, whether the table is inline or its own.
    ///
    /// Split on `,` and compared as a key, so `features = ["package"]` is a feature named `package` rather
    /// than a rename — the shape that deleted an edge from the reader this replaces.
    fn renamed_to(value: &str) -> Option<String> {
        value
            .trim()
            .trim_start_matches('{')
            .trim_end_matches('}')
            .split(',')
            .filter_map(|field| field.split_once('='))
            .find(|(key, _)| key.trim() == "package")
            .map(|(_, name)| name.trim().trim_matches('"').to_string())
    }

    let source = Source::of(text);
    let mut found = Vec::new();
    let mut table = Table::Other;
    let mut named: Option<String> = None;
    let close = |table: &Table, named: &mut Option<String>, found: &mut Vec<String>| {
        if let Table::One(heading) = table {
            found.push(named.take().unwrap_or_else(|| heading.clone()));
        }
        *named = None;
    };
    for line in source.toml().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            close(&table, &mut named, &mut found);
            let heading = trimmed.trim_start_matches('[').trim_end_matches(']');
            table = if heading.ends_with("dependencies") {
                Table::Keys
            } else if let Some((_, one)) = heading.rsplit_once("dependencies.") {
                Table::One(one.trim_matches('"').to_string())
            } else {
                Table::Other
            };
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        match &table {
            Table::Other => {}
            Table::Keys => found.push(
                renamed_to(value).unwrap_or_else(|| key.trim().trim_matches('"').to_string()),
            ),
            Table::One(_) => {
                if key.trim() == "package" {
                    named = Some(value.trim().trim_matches('"').to_string());
                }
            }
        }
    }
    close(&table, &mut named, &mut found);
    found
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
            .map(|dir| {
                dir.rsplit_once('/')
                    .map_or(*dir, |(_, name)| name)
                    .to_string()
            })
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

/// A dependency edge is read as TOML, not as one string layout.
///
/// **The reader this replaces recognised `path = "../x"` and nothing else.** `path="../x"` declares the same
/// edge with no spaces; `alias = { package = "x", … }` is cargo's rename, which this repository's own release
/// gate already reads for exactly this reason. Both were invisible, and an edge missed on **both** sides of
/// `assert_eq!(declared, reaching)` is a corpus that shrinks while agreeing with itself.
///
/// **Asked of the reader rather than of the graph, and that is forced.** The two constants are owned by the
/// two crates at the top of this workspace's dependency DAG, so no member can be made to reach either without
/// closing a cycle — measured: adding `shengmo` to `xuanji` produces
/// `cyclic package dependency: package guibiao depends on itself`, and cargo refuses to build before any
/// direction runs. The declared side is perturbable and is held that way; the graph side is not, so the
/// reader is exercised over text it is given.
#[test]
fn every_spelling_of_a_dependency_edge_is_read() {
    for (why, manifest, expected) in [
        (
            "the spelling the reader this replaces recognised",
            "[dependencies]\nshengmo = { path = \"../shengmo\" }\n",
            "shengmo",
        ),
        (
            "the same edge with no spaces around the equals",
            "[dependencies]\nshengmo = { path=\"../shengmo\" }\n",
            "shengmo",
        ),
        (
            "cargo's rename, where the key is not the crate",
            "[dependencies]\nalias = { package = \"shengmo\", path = \"../shengmo\" }\n",
            "shengmo",
        ),
        (
            "a dev-dependency is an edge too",
            "[dev-dependencies]\nshengmo = { path = \"../shengmo\" }\n",
            "shengmo",
        ),
        (
            "a target-scoped table is still a dependency table",
            "[target.'cfg(unix)'.dependencies]\nshengmo = { path = \"../shengmo\" }\n",
            "shengmo",
        ),
        (
            "one dependency's own table, named by its heading",
            "[dependencies.shengmo]\npath = \"../shengmo\"\n",
            "shengmo",
        ),
        (
            "one dependency's own table, renamed by a `package` key inside it",
            "[dependencies.alias]\npackage = \"shengmo\"\npath = \"../shengmo\"\n",
            "shengmo",
        ),
        (
            "a feature literally named `package` is a feature, not a rename",
            "[dependencies]\nshengmo = { path = \"../shengmo\", features = [\"package\"] }\n",
            "shengmo",
        ),
        (
            "and it is a feature even in a list beside others",
            "[dependencies]\nshengmo = { path = \"../x\", features = [\"a\", \"package\"] }\n",
            "shengmo",
        ),
    ] {
        assert!(
            declared_dependencies(manifest).contains(&expected.to_string()),
            "{why}: {expected} is declared and this reader did not see it — {:?}",
            declared_dependencies(manifest)
        );
    }

    // And the corpus is the TOML region, so a commented-out dependency is not one.
    assert!(
        !declared_dependencies("[dependencies]\n# shengmo = { path = \"../shengmo\" }\n")
            .contains(&"shengmo".to_string()),
        "a commented-out dependency is not a declared edge"
    );
    // A table that is not a dependency table declares no edge.
    assert!(
        declared_dependencies("[package]\nshengmo = \"1\"\n").is_empty(),
        "only dependency tables declare edges"
    );
}
