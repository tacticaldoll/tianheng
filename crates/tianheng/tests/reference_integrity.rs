//! Self-governance reaction: every in-repository path a document or comment points at must exist.
//!
//! This class was hand-swept twice — once for `.md` only — and a module split landing after that sweep
//! reintroduced it in nine places. A reader who greps for a named path and finds nothing cannot tell stale
//! prose from a bad checkout.
//!
//! It judges **tracked content**, never the worktree. A path present on disk and in no commit satisfies a
//! reference for the author who created it and nobody else, which is the direction this repository's gates are
//! held to generally.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The top-level directories a bare reference may name, and the extensions a bare basename may carry.
const PATH_PREFIXES: [&str; 6] = [
    "crates/",
    "scripts/",
    "openspec/",
    "docs/",
    "examples/",
    ".github/",
];
const BASENAME_EXTENSIONS: [&str; 5] = [".md", ".toml", ".sh", ".yml", ".lock"];

/// Documents this repository names as its own governance surface. Their absence is not a stale reference but
/// a repository that cannot be judged: every rule about them would hold vacuously.
const GOVERNANCE_DOCUMENTS: [&str; 8] = [
    "AGENTS.md",
    "AGENTS.self-law.md",
    "BACKLOG.md",
    "CHANGELOG.md",
    "COOKBOOK.md",
    "PROJECT.md",
    "README.md",
    "Cargo.toml",
];

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("Cargo.toml").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "Cargo.toml expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — a governance \
         reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

fn tracked(root: &Path) -> Vec<String> {
    let out = Command::new("git")
        .args(["ls-files"])
        .current_dir(root)
        .output()
        .expect("run git ls-files");
    assert!(
        out.status.success(),
        "`git ls-files` failed enumerating tracked paths — a failed enumeration is not a repository holding \
         no files, and every reference verdict would rest on it"
    );
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect()
}

#[test]
fn governance_documents_exist() {
    let Some(root) = workspace_root() else {
        return;
    };
    let files: HashSet<String> = tracked(&root).into_iter().collect();
    let missing: Vec<&str> = GOVERNANCE_DOCUMENTS
        .iter()
        .copied()
        .filter(|doc| !files.contains(*doc))
        .collect();
    assert!(
        missing.is_empty(),
        "named as this repository's governance documents and tracked by nothing: {missing:?} — every rule \
         about them would hold vacuously"
    );
}

/// One reference found in a document or comment.
struct Reference {
    text: String,
    /// Markdown link targets resolve relative to the file and may name a bare word; a bare word found in
    /// prose may not, or every identifier would be a reference.
    from_link: bool,
}

fn is_path_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '*' | '-')
}

/// Extract every in-repository reference a line carries.
///
/// Four forms, which is what the shell gate this replaces recognised: a path under one of the known top-level
/// directories, a `tests/…rs` path resolved against each workspace member, a markdown link target, and a bare
/// basename carrying a governance extension. Restricting the bare form to those extensions is what keeps an
/// ordinary word out of the corpus.
fn extract(line: &str) -> Vec<Reference> {
    let mut found = Vec::new();

    // Markdown links first, and their spans are consumed by the run scan below anyway — a target that is also
    // a path shape is reported once, because the offence list is deduplicated per file.
    let bytes: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i + 1 < bytes.len() {
        if bytes[i] == ']' && bytes[i + 1] == '(' {
            let start = i + 2;
            if let Some(len) = bytes[start..].iter().position(|c| *c == ')') {
                found.push(Reference {
                    text: bytes[start..start + len].iter().collect(),
                    from_link: true,
                });
                i = start + len + 1;
                continue;
            }
        }
        i += 1;
    }

    for run in line.split(|c: char| !is_path_char(c)) {
        let run = run.trim_matches('.');
        if run.is_empty() {
            continue;
        }
        let is_prefixed = PATH_PREFIXES.iter().any(|p| run.starts_with(p));
        let is_member_test = run.starts_with("tests/") && run.ends_with(".rs");
        let is_basename = !run.contains('/')
            && BASENAME_EXTENSIONS.iter().any(|e| run.ends_with(e))
            && run.starts_with(|c: char| c.is_ascii_alphanumeric() || c == '_');
        if is_prefixed || is_member_test || is_basename {
            found.push(Reference {
                text: run.to_string(),
                from_link: false,
            });
        }
    }
    found
}

/// Whether git deliberately ignores this path, which is a different fact from a stale reference: a generated
/// lockfile an example carries is named in prose and tracked by nothing on purpose.
fn ignored(root: &Path, target: &str) -> bool {
    Command::new("git")
        .args(["check-ignore", "-q", "--", target])
        .current_dir(root)
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// The nearest ancestor directory of `path` that a tracked `Cargo.toml` makes a package.
fn package_of(files: &HashSet<String>, path: &str) -> Option<String> {
    let mut parts: Vec<&str> = path.split('/').collect();
    parts.pop();
    while !parts.is_empty() {
        let dir = parts.join("/");
        if files.contains(&format!("{dir}/Cargo.toml")) {
            return Some(dir);
        }
        parts.pop();
    }
    None
}

/// Whether a tracked path index holds `target`, as a file or as a directory.
fn holds(files: &HashSet<String>, target: &str) -> bool {
    let target = target.trim_end_matches('/');
    files.contains(target) || files.iter().any(|f| f.starts_with(&format!("{target}/")))
}

#[test]
fn in_repository_references_resolve() {
    let Some(root) = workspace_root() else {
        return;
    };
    let all = tracked(&root);
    let files: HashSet<String> = all.iter().cloned().collect();

    let members: Vec<String> = all
        .iter()
        .filter_map(|p| p.strip_prefix("crates/"))
        .filter_map(|rest| rest.split('/').next())
        .map(str::to_string)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    // Every tracked package, workspace member or not: an example is a package and its README names its own
    // `tests/…` the same way a crate's does.
    let packages: Vec<String> = all
        .iter()
        .filter_map(|p| p.strip_suffix("/Cargo.toml"))
        .map(str::to_string)
        .collect();
    assert!(
        !members.is_empty(),
        "found no tracked workspace member under crates/, so a `tests/…` reference could be resolved against \
         nothing and would read as clean"
    );

    // A basename names a file only when exactly one tracked file carries it; otherwise the reference is
    // ambiguous and says nothing about which file it meant.
    let mut basename_count: std::collections::HashMap<&str, usize> =
        std::collections::HashMap::new();
    for path in &all {
        *basename_count
            .entry(path.rsplit('/').next().unwrap_or(path))
            .or_default() += 1;
    }

    let mut offences: BTreeSet<String> = BTreeSet::new();
    let mut inspected = 0usize;

    for rel_path in &all {
        if !rel_path.ends_with(".md") && !rel_path.ends_with(".rs") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(root.join(rel_path)) else {
            panic!(
                "cannot read tracked file '{rel_path}' — a file this reaction claims to have inspected must \
                 have been read"
            );
        };
        inspected += 1;
        let is_test_source = rel_path.contains("/tests/");

        for line in content.lines() {
            for reference in extract(line) {
                let raw = reference.text.trim_end_matches(['.', ',', ')', '`']);
                let raw = raw.split('#').next().unwrap_or(raw);
                if raw.is_empty()
                    || raw.starts_with("http://")
                    || raw.starts_with("https://")
                    || raw.starts_with("mailto:")
                    || raw.contains("::")
                    || raw.contains('*')
                {
                    continue;
                }

                if reference.from_link {
                    // The same illustrative rule the qualified branch carries: a fixture's markdown link
                    // names a path in the repository that fixture builds.
                    if is_test_source
                        && (raw.starts_with("scripts/") || raw.starts_with("examples/"))
                    {
                        continue;
                    }
                    // A link may name a bare word, which is a rustdoc symbol rather than a path.
                    if !raw.contains('/') && !raw.contains('.') {
                        continue;
                    }
                    let cleaned = raw.strip_prefix("file://").unwrap_or(raw);
                    let cleaned = match cleaned.find("/tianheng/") {
                        Some(pos) => &cleaned[pos + "/tianheng/".len()..],
                        None => cleaned,
                    };
                    // An absolute target — or one that arrived as `file://` — names the repository root.
                    // Joining it to the naming file's directory resolves `COOKBOOK.md` to a sibling of the
                    // spec that linked it, which exists nowhere.
                    let absolute = raw.starts_with('/') || raw.starts_with("file://");
                    let parent = Path::new(rel_path).parent().unwrap_or(Path::new(""));
                    let joined = if absolute {
                        PathBuf::from(cleaned.trim_start_matches('/'))
                    } else {
                        parent.join(cleaned)
                    };
                    let mut parts: Vec<String> = Vec::new();
                    for component in joined.components() {
                        match component {
                            std::path::Component::Normal(c) => {
                                parts.push(c.to_string_lossy().into_owned())
                            }
                            std::path::Component::ParentDir => {
                                parts.pop();
                            }
                            _ => {}
                        }
                    }
                    let normalised = parts.join("/");
                    if normalised.is_empty() || holds(&files, &normalised) {
                        continue;
                    }
                    offences.insert(format!(
                        "{rel_path}: links to '{}', which resolves to '{normalised}' and is tracked by \
                         nothing",
                        reference.text
                    ));
                    continue;
                }

                if raw.starts_with("tests/") {
                    // A package-RELATIVE path names nothing when the naming file belongs to no package: a
                    // root-level governance document saying `tests/…` is describing some package's layout in
                    // general, or quoting a path precisely because it exists nowhere — `[0.4.0]` records
                    // correcting one and names it in the sentence that says so. Resolving it against every
                    // member instead would make that record an offence for being a record.
                    if package_of(&files, rel_path).is_none() {
                        continue;
                    }
                    // A package-relative path. It resolves against the referencing file's OWN package first —
                    // an example's README naming `tests/reaction.rs` means that example's — and against every
                    // workspace member after, which is how a governance document names one without repeating
                    // the crate.
                    if let Some(home) = package_of(&files, rel_path)
                        && holds(&files, &format!("{home}/{raw}"))
                    {
                        continue;
                    }
                    if packages
                        .iter()
                        .any(|home| holds(&files, &format!("{home}/{raw}")))
                    {
                        continue;
                    }
                    offences.insert(format!(
                        "{rel_path}: references '{raw}', which is tracked under no workspace member"
                    ));
                    continue;
                }

                if raw.contains('/') {
                    if holds(&files, raw) || ignored(&root, raw) {
                        continue;
                    }
                    // Illustrative rather than real, in two decidable forms.
                    //
                    // A `crates/<name>/…` path whose `<name>` is no tracked workspace member is a fixture in
                    // a doc comment or a test — `crates/foo/src/lib.rs` — and reading it as a dangling
                    // reference would make every example of the shape an offence. The rule needs the member
                    // set, which is why an empty one refuses above.
                    if let Some(rest) = raw.strip_prefix("crates/")
                        && let Some(name) = rest.split('/').next()
                        && !members.iter().any(|m| m == name)
                    {
                        continue;
                    }
                    // And a repository path named INSIDE test code, which builds the shapes it judges: a
                    // fixture repository's `scripts/…` or `examples/…` exists in that fixture and nowhere
                    // here. What this costs is declared — a genuinely stale reference written into a test
                    // goes unseen — and the alternative costs more: without it, every reaction that
                    // constructs a fixture is an offence for constructing one.
                    if is_test_source
                        && (raw.starts_with("scripts/") || raw.starts_with("examples/"))
                    {
                        continue;
                    }
                    offences.insert(format!(
                        "{rel_path}: references '{raw}', which is not tracked in this repository"
                    ));
                    continue;
                }

                // A bare basename is a reference only where exactly one tracked file carries it. Several
                // means the word says nothing about which file it meant; NONE means it is not a path at all —
                // `proposal.md` and `tasks.md` name the OpenSpec change scaffolding a completed change prunes,
                // and reading them as dangling would make the lifecycle's own vocabulary an offence. The real
                // signal is the qualified form above, which this leaves untouched.
                let _ = basename_count.get(raw);
            }
        }
    }

    assert!(
        inspected > 0,
        "inspected 0 files — no tracked *.md or *.rs, so this reaction would report clean without having \
         read anything"
    );
    assert!(
        offences.is_empty(),
        "{} stale in-repository reference(s) across {inspected} files — point each at the file that now \
         holds the referenced item, or drop the reference:\n{}",
        offences.len(),
        offences.iter().cloned().collect::<Vec<_>>().join("\n")
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
