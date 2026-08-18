//! Repository check: every registered refusal site is observed by a direction, and the unregistered ones
//! are counted.
//!
//! `AGENTS.md` states that **a guard is not a guard until it has been seen to fail**, and
//! `repository-checks` requires every refusal a check holds to have been run against a tree carrying the
//! shape it refuses. That requirement's scenarios held only its other clause — where a check may live — so
//! the half about refusals was carried by reviewer attention, and attention failed three times in one
//! window: a title guard with no negative run, three example-pin branches, then four internal-pin branches
//! in the change immediately after the one whose own record names the class.
//!
//! **Why nothing held it before.** A refusal's identity lived only in its message, and a message is a
//! *template* while a direction asserts a *rendering* of it. Five textual predicates were written against
//! that gap and measured; each was wrong in a different direction, and no reading of text answers the
//! question — which is what `pin_bites` already says about whether a test bites. So the site travels in the
//! value, a direction names the site it observed, and the two are compared by running.
//!
//! **The migration is visible rather than instantaneous.** Rewriting every site at once would be one
//! unreadable change; `refusal::violation` and `refusal::cannot_judge` stay beside their `_at` siblings
//! while modules move across, and the projection below carries how many sites have not moved. Registering a
//! site is a commitment that a direction observes it — a registered site no direction names refuses here —
//! so coverage cannot lag behind the migration.

use kanhe::region::Source;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

const PROJECTION: &str = "docs/refusal-register.md";

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(PROJECTION).is_file() || root.join("AGENTS.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Tracked paths under `dir` ending in `.rs`, through git rather than a filesystem walk.
///
/// The corpus is what the repository tracks, for the reason every sibling here gives: an untracked scratch
/// copy of a gate file is not repository content and must not decide a verdict.
fn tracked(root: &Path, dir: &str) -> Vec<PathBuf> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", dir])
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("cannot enumerate {dir}: {err}"));
    assert!(
        out.status.success(),
        "git ls-files failed over {dir}, which is not the same fact as an empty directory: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| p.ends_with(".rs"))
        .map(|p| root.join(p))
        .collect()
}

/// The first string literal argument of each `call` in `text`, with the line it sits on.
///
/// Deliberately not a Rust parser: the argument is a literal by construction — the constructors take
/// `&'static str` — so the first quote after the opening parenthesis begins it.
fn first_literal_args(text: &str, call: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = text[at..].find(call) {
        let start = at + offset;
        at = start + call.len();
        // A call, not the tail of a longer identifier: `violation_at(` must not be read as `violation(`.
        // A `call` opening with `::` carries its own left boundary, so no check on the byte before it —
        // which is the last byte of the path, and always an identifier byte.
        if call.starts_with(|c: char| c.is_ascii_alphanumeric()) && start > 0 {
            let before = text.as_bytes()[start - 1];
            if before.is_ascii_alphanumeric() || before == b'_' {
                continue;
            }
        }
        let rest = &text[at..];
        let Some(open) = rest.find('"') else { continue };
        // Only a literal that opens the argument list, so a call whose first argument is an expression is
        // not read as though the next literal on the line were its site.
        if rest[..open].chars().any(|c| !c.is_whitespace()) {
            continue;
        }
        let Some(close) = rest[open + 1..].find('"') else {
            continue;
        };
        found.push((
            rest[open + 1..open + 1 + close].to_string(),
            text[..start].matches('\n').count() + 1,
        ));
    }
    found
}

/// `text` with its comments, string literals and imports removed, so neither a name in prose nor a name in
/// a `use` list is read as a construction.
///
/// The import line was the second way a count could be wrong: a module holding both forms names all four
/// constructors in one `use`, and the bare identifiers there were counted as two more unregistered sites —
/// a figure two above the truth in the module where the truth is what the migration is steering by.
fn executed_rust(text: &str) -> String {
    Source::of(text)
        .rust()
        .lines()
        .filter(|line| !line.trim_start().starts_with("use "))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Whether `text` has a site identity's shape: `<capability>#<slug>`, lowercase and hyphenated.
///
/// **`#` and not `/`, because `<capability>/<slug>` is already an identity here.** The bound register
/// resolves that spelling anywhere in tracked Rust or Markdown as a reference to a declared observation
/// bound, so the first draft of these identities was read as ten references to bounds that do not exist —
/// measured, against this repository, before the shape was settled. A refusal site and an observation bound
/// are opposite facts, one about what is observed and one about what is not, and giving the new one its own
/// separator leaves the older reader's floor exactly where it was.
fn is_a_site(text: &str) -> bool {
    let Some((capability, slug)) = text.split_once('#') else {
        return false;
    };
    let word = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    word(capability) && word(slug)
}

/// Occurrences of the constructor named `name` in `text`, however it is reached.
///
/// **The identifier, not `name(`.** Counting the call syntax missed a constructor used as a value:
/// `workspace_version(repo).map_err(cannot_judge)` has no opening parenthesis after the name, so a live
/// refusal site was invisible to the register built to count them — found by migrating the module the site
/// was in, and only because the compiler then objected to the import. A register whose corpus reader can be
/// stepped around by a point-free call is one that reports a smaller number than the truth, which is the
/// direction that matters.
///
/// Both boundaries, so `cannot_judge_at` is not counted as `cannot_judge`.
fn calls(text: &str, name: &str) -> usize {
    let boundary = |byte: u8| !(byte.is_ascii_alphanumeric() || byte == b'_');
    let mut count = 0;
    let mut at = 0;
    while let Some(offset) = text[at..].find(name) {
        let start = at + offset;
        at = start + name.len();
        let before = if start == 0 {
            b' '
        } else {
            text.as_bytes()[start - 1]
        };
        let after = text.as_bytes().get(at).copied().unwrap_or(b' ');
        if boundary(before) && boundary(after) {
            count += 1;
        }
    }
    count
}

struct Register {
    /// Registered site to the module and line of each branch producing it.
    registered: BTreeMap<String, Vec<(String, usize)>>,
    /// Sites still constructed through the unregistered form, by module.
    unregistered: BTreeMap<String, usize>,
    /// Sites named by a direction, to the test files naming them.
    cited: BTreeMap<String, BTreeSet<String>>,
}

fn read(root: &Path) -> Register {
    let mut registered: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    let mut unregistered: BTreeMap<String, usize> = BTreeMap::new();
    for path in tracked(root, "crates/kanhe/src") {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        // The constructors themselves are declarations, not sites.
        if name.ends_with("src/refusal.rs") {
            continue;
        }
        for call in ["violation_at(", "cannot_judge_at("] {
            for (site, line) in first_literal_args(&text, call) {
                registered
                    .entry(site)
                    .or_default()
                    .push((name.clone(), line));
            }
        }
        // **Executed Rust, not the file.** Counting the bare identifier over the whole text counted every
        // doc comment naming a constructor — this repository's prose names them constantly, and the figure
        // jumped by four modules that construct no refusal at all. `region` is the module written so that
        // forgetting to ask was not possible, and the same reader the gates themselves use.
        let executed = executed_rust(&text);
        let open = calls(&executed, "violation") + calls(&executed, "cannot_judge");
        if open > 0 {
            unregistered.insert(name, open);
        }
    }
    let mut cited: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for dir in ["crates/kanhe/tests", "crates/kanhe/src/tests"] {
        for path in tracked(root, dir) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
            let name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            // **By position and by shape, never by the bare name.** `Result::expect` is a method taking a
            // panic message, and this reader's own source is in the corpus it reads — so a citation is
            // recognised as a path-qualified call, `refusal::expect`, whose first argument has a site's
            // shape. Reading every `expect(` counted panic messages as citations, this file's own included.
            for (site, _) in first_literal_args(&text, "::expect(") {
                if is_a_site(&site) {
                    cited.entry(site).or_default().insert(name.clone());
                }
            }
        }
    }
    Register {
        registered,
        unregistered,
        cited,
    }
}

/// A registered site names one branch, and no two branches share a name.
///
/// Identity that is not injective is the defect this repository has already recorded once, where a
/// per-item finding not qualified by its owner let a baseline mask a new violation. Here a shared slug
/// would let one direction's citation vouch for a branch it never reached.
#[test]
fn a_registered_site_names_exactly_one_branch() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let shared: Vec<String> = register
        .registered
        .iter()
        .filter(|(_, sites)| sites.len() > 1)
        .map(|(slug, sites)| {
            let at: Vec<String> = sites
                .iter()
                .map(|(module, line)| format!("{module}:{line}"))
                .collect();
            format!("  {slug} — {}", at.join(", "))
        })
        .collect();
    assert!(
        shared.is_empty(),
        "these site identities name more than one branch, so a direction citing one vouches for the \
         others:\n{}",
        shared.join("\n")
    );
}

/// A site's capability half names a capability this repository specifies.
#[test]
fn a_registered_site_is_owned_by_a_capability() {
    let Some(root) = workspace_root() else {
        return;
    };
    let capabilities: BTreeSet<String> = std::fs::read_dir(root.join("openspec/specs"))
        .expect("read the spec directories")
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().join("spec.md").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !capabilities.is_empty(),
        "found no capability with a spec.md, so this direction would hold over nothing"
    );
    let register = read(&root);
    let orphans: Vec<String> = register
        .registered
        .keys()
        .filter(|slug| {
            !slug.split_once('#').is_some_and(|(capability, rest)| {
                capabilities.contains(capability) && !rest.is_empty()
            })
        })
        .cloned()
        .collect();
    assert!(
        orphans.is_empty(),
        "these site identities name no specified capability, so nothing owns the refusal they \
         identify:\n  {}",
        orphans.join("\n  ")
    );
}

/// Every registered site is observed by a direction, and every citation names a registered site.
///
/// Both ways, because either alone is satisfiable by doing nothing: a register nobody cites passes the
/// first direction of a one-way check, and a citation of a site that no longer exists passes the other.
#[test]
fn a_registered_site_and_the_directions_that_observe_it_agree() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    assert!(
        !register.registered.is_empty(),
        "found no registered refusal site, so this comparison would hold over nothing"
    );
    let unobserved: Vec<&String> = register
        .registered
        .keys()
        .filter(|slug| !register.cited.contains_key(*slug))
        .collect();
    assert!(
        unobserved.is_empty(),
        "these refusal sites are registered and no direction observes them — registering a site is the \
         commitment that one does:\n  {}",
        unobserved
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    let dangling: Vec<String> = register
        .cited
        .iter()
        .filter(|(slug, _)| !register.registered.contains_key(*slug))
        .map(|(slug, files)| {
            format!(
                "  {slug} — cited by {}",
                files.iter().cloned().collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    assert!(
        dangling.is_empty(),
        "these directions cite a site no refusal produces, so they assert nothing about this \
         repository:\n{}",
        dangling.join("\n")
    );
}

/// The register, and how much of it has not moved yet, as a document rather than as a claim.
///
/// The count of unregistered sites is **produced**, which is the whole reason it can be trusted to fall: a
/// figure typed into prose is one nothing measures, and this repository has already spent a window
/// replacing those. A change in either direction has to be blessed, so a module migrating shows up here and
/// a new unregistered site cannot arrive quietly.
#[test]
fn the_register_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let remaining: usize = register.unregistered.values().sum();
    let mut out = String::from(
        "# Refusal register\n\nEvery **registered** refusal site, and the direction that observes it. A \
         site is registered by being constructed through `refusal::violation_at` or \
         `refusal::cannot_judge_at`, and observed by a direction calling `refusal::expect` with the same \
         identity — compared by running, because a message is a template and a direction asserts a \
         rendering of it.\n\nGenerated from `crates/kanhe/src/**.rs` by \
         `crates/kanhe/tests/refusal_register.rs`. **Do not edit by hand** — regenerate with `BLESS=1 \
         TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test refusal_register`. A stale projection fails \
         that gate.\n\n",
    );
    out.push_str(&format!(
        "## Not registered yet\n\n**{remaining} refusal sites are not registered yet**, across {} \
         module(s). They are constructed through `refusal::violation` and `refusal::cannot_judge`, whose \
         only remaining purpose is to be deleted when this figure reaches zero. The module paths are below \
         this heading rather than above it, because the header is where the projection register reads which \
         single unit holds this document.\n\n",
        register.unregistered.len()
    ));
    for (module, count) in &register.unregistered {
        out.push_str(&format!("- `{module}` — {count}\n"));
    }
    out.push_str("\n## Registered\n\n");
    for (slug, sites) in &register.registered {
        let cited = register
            .cited
            .get(slug)
            .map(|files| files.iter().cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        // **The module, never the line.** A reference naming a position is refused in tracked content
        // here, and this document is tracked content: a line number is right at the moment it is written
        // and wrong after the next edit above it. The identity is the slug; the module says where to look.
        let modules: BTreeSet<&str> = sites.iter().map(|(module, _)| module.as_str()).collect();
        out.push_str(&format!(
            "### `{slug}`\n\n- produced in `{}`\n- observed by `{cited}`\n\n",
            modules.iter().copied().collect::<Vec<_>>().join("`, `")
        ));
    }
    // One trailing newline and no blank line before it, which is the whitespace this repository keeps —
    // the per-entry blocks are separated by one, and the last of them would otherwise leave two.
    while out.ends_with("\n\n") {
        out.pop();
    }
    tianheng::testing::assert_projection_matches(&root, PROJECTION, &out);
}
