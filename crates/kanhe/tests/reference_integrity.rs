//! Repository check: every in-repository path named by tracked prose must exist.
//!
//! This class was hand-swept twice — once for `.md` only — and a module split landing after that sweep
//! reintroduced it in nine places. A reader who greps for a named path and finds nothing cannot tell stale
//! prose from a bad checkout.
//!
//! **Which formats carry prose is declared once, and every tracked format must be classified.** Before that it
//! was two lists: an extension filter deciding what to open, and a marker rule deciding which of its lines to
//! read. A format could sit in one and not the other, which is how shell — the two sanctioned wrappers, whose
//! comments cite the Rust gates they sequence *by path* — went unread while the marker rule had known `#` all
//! along. Closing that by adding one extension would have been the third turn of the same handle: the same
//! window replaced two argument denylists with allowlists for exactly this reason, and an extension list beside
//! a marker rule is the shape where a format is admitted with no marker or given a marker and never opened.
//!
//! So [`FORMATS`] is the single declaration, and [`every_tracked_format_is_classified`] fails on a format the
//! repository holds and it does not name. A new file type arrives as one row, or as a failure — not as a
//! silence. Measured when this landed: shell and YAML were the formats it had been reading nothing from, and
//! neither named an absent path, so both were silences rather than backlogs.
//!
//! It judges **tracked content**, never the worktree. A path present on disk and in no commit satisfies a
//! reference for the author who created it and nobody else, which is the direction this repository's gates are
//! held to generally.

use std::collections::{BTreeSet, HashSet};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

/// The top-level directories a bare reference may name, and the extensions a bare basename may carry.
const PATH_PREFIXES: [&str; 6] = [
    "crates/",
    "scripts/",
    "openspec/",
    "docs/",
    "examples/",
    ".github/",
];
const BASENAME_EXTENSIONS: [&str; 6] = [".md", ".toml", ".sh", ".yml", ".lock", ".rs"];

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

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

fn scratch(label: &str) -> PathBuf {
    static NEXT: AtomicUsize = AtomicUsize::new(0);
    loop {
        let candidate = std::env::temp_dir().join(format!(
            "tianheng-reference-integrity-{label}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        match xingbiao::claim_scratch(&candidate) {
            Ok(()) => return candidate,
            Err(err) if err.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(err) => panic!("cannot acquire reference-integrity fixture root: {err}"),
        }
    }
}

/// How a tracked format carries prose.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Prose {
    /// Every line is prose — a Markdown document.
    Whole,
    /// Prose lives on the lines whose first non-whitespace token is this marker.
    LineComment(&'static str),
    /// The format carries no prose a reader would follow a path from: a licence text, a data table, a
    /// placeholder. Classified rather than omitted, so *unclassified* stays a failure.
    None,
}

/// Every format this repository tracks, and how it carries prose. **The one declaration.**
///
/// Keyed by extension, or by whole file name where the format has none. `Cargo.lock` and `CODEOWNERS` carry `#`
/// comments and are read for the same reason every other comment is: a path named there rots the same way.
///
/// A format the repository holds and this array does not name is a failure, not a default — see
/// [`every_tracked_format_is_classified`]. Defaulting either way is the trap: `None` would read a new format's
/// prose as absent, and a marker would guess one it may not have.
/// How a key is matched against a file name.
///
/// Written per entry rather than inferred, because the rule it replaces — `name == key || name.ends_with(key)`
/// — conflated a whole name with an extension and could express no third shape at all. That is what pushed the
/// licence family out of this array and into an early return in [`prose_of`], leaving a declaration that calls
/// itself the one declaration while a classification lived somewhere else, and its entry here unreachable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Match {
    /// The whole file name, for a format that has no extension.
    Name,
    /// The extension, dot included.
    Extension,
    /// A prefix, for a family whose members carry a variant *suffix* instead of an extension.
    Prefix,
}

const FORMATS: [(&str, Match, Prose); 15] = [
    (".md", Match::Extension, Prose::Whole),
    // JSON admits no comment syntax at all, so there is no prose here to carry a reference — `Prose::None`
    // is the honest classification rather than a narrowing. The tracked members are the pinned validator's
    // `package.json` and the `package-lock.json` reproducing its tree.
    (".json", Match::Extension, Prose::None),
    (".rs", Match::Extension, Prose::LineComment("//")),
    (".toml", Match::Extension, Prose::LineComment("#")),
    (".sh", Match::Extension, Prose::LineComment("#")),
    (".yml", Match::Extension, Prose::LineComment("#")),
    (".yaml", Match::Extension, Prose::LineComment("#")),
    (".gitignore", Match::Name, Prose::LineComment("#")),
    (".npmrc", Match::Name, Prose::LineComment("#")),
    (".lock", Match::Extension, Prose::LineComment("#")),
    ("CODEOWNERS", Match::Name, Prose::LineComment("#")),
    (".txt", Match::Extension, Prose::None),
    (".tsv", Match::Extension, Prose::None),
    (".gitkeep", Match::Name, Prose::None),
    ("LICENSE", Match::Prefix, Prose::None),
];

/// How `path`'s format carries prose, or `None` if this repository has never classified it.
///
/// Matched on the whole file name first, then on the extension, so `CODEOWNERS` and `.gitignore` resolve without
/// an extension and `LICENSE-MIT` resolves by prefix — the licence files carry a variant suffix rather than an
/// extension.
fn prose_of(path: &str) -> Option<Prose> {
    let name = Path::new(path).file_name()?.to_str()?;
    FORMATS
        .iter()
        .find(|(key, how, _)| matches(name, key, *how))
        .map(|(_, _, prose)| *prose)
}

/// Whether one key claims this file name, under the shape that key declares.
fn matches(name: &str, key: &str, how: Match) -> bool {
    match how {
        Match::Name => name == key,
        Match::Extension => name.ends_with(key),
        Match::Prefix => name.starts_with(key),
    }
}

fn is_inspected_source(path: &str) -> bool {
    !matches!(prose_of(path), None | Some(Prose::None))
}

/// A line worth reading for the paths it names, decided by the same declaration that decided the file.
///
/// A shell shebang reaches this and names `/usr/bin/env`, an absolute path outside every prefix this check
/// recognizes, so it is not a reference and not a false positive.
fn is_inspected_line(path: &str, line: &str) -> bool {
    match prose_of(path) {
        Some(Prose::Whole) => true,
        Some(Prose::LineComment(marker)) => line.trim_start().starts_with(marker),
        None | Some(Prose::None) => false,
    }
}

/// Every format this repository tracks is named by [`FORMATS`].
///
/// The direction that makes the declaration single. Without it, a new file type is read by nothing and the whole
/// sweep still reports clean — which is exactly what happened to shell and to YAML, each for a different window.
#[test]
fn every_tracked_format_is_classified() {
    let Some(root) = workspace_root() else {
        return;
    };
    let all = tracked(&root);
    assert!(
        !all.is_empty(),
        "no tracked path was enumerated, so this direction would hold over nothing"
    );
    // Keyed by FORMAT, not by file, so one unclassified type reads as one entry rather than as every file
    // carrying it — the diagnostic says `format(s)` and must show formats.
    let unclassified: BTreeSet<String> = all
        .iter()
        .filter(|path| prose_of(path).is_none())
        .filter_map(|path| {
            let name = Path::new(path).file_name()?.to_str()?;
            Some(match name.rsplit_once('.') {
                Some((_, extension)) => format!(".{extension}"),
                None => name.to_string(),
            })
        })
        .collect();
    assert!(
        unclassified.is_empty(),
        "this repository tracks {} format(s) `FORMATS` does not classify: {}\nAdd each with the marker its \
         comments use, or `Prose::None` if it carries no prose — an unclassified format is read by nothing \
         while every sweep here still reports clean",
        unclassified.len(),
        unclassified.into_iter().collect::<Vec<_>>().join(", ")
    );
}

/// The direction the classification lacked: every declared entry is **exercised** by some tracked file.
///
/// `every_tracked_format_is_classified` runs one way — tracked file to entry — and a one-way check cannot see
/// a member nothing reaches. Measured before this direction existed: `("LICENSE", …)` was unreachable, because
/// an early return in `prose_of` classified every `LICENSE`-prefixed name before the array was consulted and
/// no tracked name ends with `LICENSE` without starting with it. The array called itself the one declaration
/// while one classification lived in two places, and the entry that proved it stood without anything
/// noticing.
///
/// A dead entry is not cosmetic here: it reads as coverage of a format, so the next reader adding that format
/// finds it already declared and moves on.
#[test]
fn every_declared_format_is_exercised_by_a_tracked_file() {
    let Some(root) = workspace_root() else {
        return;
    };
    let all = tracked(&root);
    assert!(
        !all.is_empty(),
        "no tracked path was enumerated, so this direction would hold over nothing"
    );
    let names: Vec<&str> = all
        .iter()
        .filter_map(|path| Path::new(path).file_name()?.to_str())
        .collect();
    let unexercised: Vec<&str> = FORMATS
        .iter()
        .filter(|(key, how, _)| !names.iter().any(|name| matches(name, key, *how)))
        .map(|(key, _, _)| *key)
        .collect();
    assert!(
        unexercised.is_empty(),
        "`FORMATS` declares {} entr(y/ies) no tracked file reaches: {}\nDrop it, or move the classification \
         that shadows it back into the array — a declaration nothing exercises reads as coverage while giving \
         none",
        unexercised.len(),
        unexercised.join(", ")
    );
}

/// **Exactly one** entry claims each tracked file, so `find`'s answer does not depend on the array's order.
///
/// The order-dependence was unstated and held only by no key being a suffix of another. Asserted over the real
/// corpus rather than as a rule about keys, because the keys now carry three different match shapes and
/// "no key is a suffix of another" does not describe a collision between a `Name` and a `Prefix`.
#[test]
fn no_tracked_file_is_claimed_by_two_declared_formats() {
    let Some(root) = workspace_root() else {
        return;
    };
    let all = tracked(&root);
    assert!(
        !all.is_empty(),
        "no tracked path was enumerated, so this direction would hold over nothing"
    );
    let mut contested = Vec::new();
    for path in &all {
        let Some(name) = Path::new(path).file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let claimants: Vec<&str> = FORMATS
            .iter()
            .filter(|(key, how, _)| matches(name, key, *how))
            .map(|(key, _, _)| *key)
            .collect();
        if claimants.len() > 1 {
            contested.push(format!(
                "  {path} is claimed by {}",
                claimants.join(" and ")
            ));
        }
    }
    assert!(
        contested.is_empty(),
        "a file matches more than one `FORMATS` entry, so which prose rule applies is decided by the array's \
         order:\n{}",
        contested.join("\n")
    );
}

fn tracked(root: &Path) -> Vec<String> {
    // Through the builder for the reason the whole capability's Purpose states: every read behind a verdict
    // here is isolated from config outside the repository being judged, not only the one an ignore file can
    // flip.
    let out = kanhe::hermetic_git::hermetic("git")
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
///
/// A directory-only `.gitignore` pattern (one ending in `/`, e.g. `/target/`) only matches a candidate git can
/// see is a directory. When the candidate exists on disk, `git check-ignore` `lstat`s it and knows; when it does
/// not — the ordinary case for a generated path a fresh checkout has not built yet — a bare query with no
/// trailing slash reads as "not a directory" and a directory-only pattern silently fails to match, so whether
/// this check fires depended on whichever example directories happened to be built on the machine running it.
/// Querying the target once more with a trailing slash forces the directory reading regardless of what is on
/// disk, so a real directory-only match no longer depends on incidental local build state.
///
/// **This widens what counts as ignored only when `target` is not already a real, non-directory file.** A
/// trailing slash asks git "if this were a directory, would a directory-only pattern ignore it" — sound when
/// nothing is there yet to say otherwise, but wrong once `target` already exists on disk as something that is
/// not a directory: forcing the directory reading there can match a directory-only pattern a real file must
/// never match (`git check-ignore` itself agrees the bare, no-slash query is correct in that case). The retry
/// is skipped whenever the candidate is on disk and not a directory, so a real file sharing a name with a
/// directory-only pattern is never misclassified as ignored.
fn ignored(root: &Path, target: &str) -> bool {
    let query = |candidate: &str| {
        // **This is the one read here whose answer an ambient file changes**, and it ran through a bare
        // `Command::new("git")`. Measured: with a global `core.excludesFile` naming a path,
        // `check-ignore` answers *ignored* for it; with the setting named it does not.
        //
        // `hermetic` closes it now — it names `core.excludesFile` through `GIT_CONFIG_COUNT`, which the
        // config-file variables alone did not do, because `$XDG_CONFIG_HOME/git/ignore` is the default
        // excludes path git uses when no config file names one. That row moved into the builder once this
        // finding showed it was not confined to one read. The flag stays as the narrower statement at the
        // call site whose verdict it decides; it is no longer the only thing standing here.
        //
        // The direction is what makes it consequential: `true` here means the offence is **not** reported,
        // so an entry in whoever's personal ignore file quietly excuses a stale reference — an
        // under-refusal whose verdict depends on who runs the gate, in the capability whose Purpose is
        // that a checkout's verdict does not depend on ambient process state.
        //
        // `.git/info/exclude` is inside the repository, so no config setting reaches it; that row of
        // `hermetic`'s table stays open here as it does for the publish gate.
        //
        // A spawn or fatal failure reads as *not ignored*, which reports the offence. That is the safe
        // direction for this predicate: the alternative excuses a reference over a read that never happened.
        kanhe::hermetic_git::hermetic("git")
            .args(["-c", "core.excludesFile=/dev/null"])
            .args(["check-ignore", "-q", "--", candidate])
            .current_dir(root)
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
    };
    if query(target) {
        return true;
    }
    let trimmed = target.trim_end_matches('/');
    if root.join(trimmed).is_file() {
        return false;
    }
    query(&format!("{trimmed}/"))
}

/// A directory-only ignore pattern must be recognised whether or not the directory happens to exist yet.
///
/// Built against a throwaway git repository rather than this workspace, so the result cannot depend on
/// whichever example or crate has already been built here. `/build/` is a directory-only pattern; the probe
/// never creates a `build` directory, matching the ordinary case of a generated path a fresh checkout has not
/// produced yet.
#[test]
fn a_directory_only_ignore_pattern_reacts_whether_or_not_the_directory_exists() {
    let repo = scratch("directory-ignore");
    // Through the shared fixture builder, so `init.templateDir` in a global config cannot seed this
    // repository from the machine being judged — which is verbatim what that builder exists to prevent, and
    // is the same channel the ignore query above closes.
    kanhe::hermetic_git::fixture(&repo, "git", &["init", "-q"]);
    std::fs::write(repo.join(".gitignore"), "/build/\n").expect("write the fixture .gitignore");

    let seen = ignored(&repo, "build");
    let _ = std::fs::remove_dir_all(&repo);
    assert!(
        seen,
        "a directory-only ignore pattern must match its candidate even before the directory is ever \
         created — otherwise this check's verdict on a generated path depends on which examples happen to \
         be built on the machine running it"
    );
}

/// A real file sharing a name with a directory-only ignore pattern is not ignored.
///
/// The trailing-slash retry above exists to catch a directory-only pattern before the directory is ever
/// created; forcing that same directory reading onto a candidate that already exists on disk as a real,
/// ordinary file would instead misreport a genuinely stale or tracked-elsewhere reference as "deliberately
/// ignored" — the false positive the previous doc comment's "can only widen, never narrow" claim did not
/// account for.
#[test]
fn a_real_file_sharing_a_directory_only_pattern_s_name_is_not_ignored() {
    let repo = scratch("directory-ignore-real-file");
    // Through the shared fixture builder, so `init.templateDir` in a global config cannot seed this
    // repository from the machine being judged — which is verbatim what that builder exists to prevent, and
    // is the same channel the ignore query above closes.
    kanhe::hermetic_git::fixture(&repo, "git", &["init", "-q"]);
    std::fs::write(repo.join(".gitignore"), "/build/\n").expect("write the fixture .gitignore");
    std::fs::write(repo.join("build"), "not a directory").expect("write the fixture file");

    let seen = ignored(&repo, "build");
    let _ = std::fs::remove_dir_all(&repo);
    assert!(
        !seen,
        "`build` exists on disk as an ordinary file, not a directory, so a directory-only `/build/` \
         pattern must not be read as ignoring it"
    );
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
    let offences = offences_in(&root, &root, &all, &all);
    assert!(
        offences.is_empty(),
        "{} stale in-repository reference(s) — point each at the file that now holds the referenced item, \
         or drop the reference:\n{}",
        offences.len(),
        offences.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Every stale reference `corpus` carries, judged against the paths `tracked` names.
///
/// Split from the check so a NEGATIVE fixture can call it. Asserting `offences.is_empty()` over a clean
/// tree is a verdict that deleting a detector can only make emptier: measured, all four extraction forms were
/// disabled one at a time and the check stayed green every time. A check whose only assertion is that
/// it found nothing cannot be shown to find anything.
/// A markdown link target, resolved against the linking file's own directory.
///
/// Returns the offence, or `None` when the target resolves or is not a path at all.
fn link_offence(
    files: &HashSet<String>,
    rel_path: &str,
    reference: &Reference,
    raw: &str,
    is_test_source: bool,
) -> Option<String> {
    // The same illustrative rule the qualified branch carries: a fixture's markdown link names a path in the
    // repository that fixture builds.
    if is_test_source && (raw.starts_with("scripts/") || raw.starts_with("examples/")) {
        return None;
    }
    // A link may name a bare word, which is a rustdoc symbol rather than a path.
    if !raw.contains('/') && !raw.contains('.') {
        return None;
    }
    let cleaned = raw.strip_prefix("file://").unwrap_or(raw);
    let cleaned = match cleaned.find("/tianheng/") {
        Some(pos) => &cleaned[pos + "/tianheng/".len()..],
        None => cleaned,
    };
    // An absolute target — or one that arrived as `file://` — names the repository root. Joining it to the
    // naming file's directory resolves `COOKBOOK.md` to a sibling of the spec that linked it, which exists
    // nowhere.
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
            std::path::Component::Normal(c) => parts.push(c.to_string_lossy().into_owned()),
            std::path::Component::ParentDir => {
                parts.pop();
            }
            _ => {}
        }
    }
    let normalised = parts.join("/");
    if normalised.is_empty() || holds(files, &normalised) {
        return None;
    }
    Some(format!(
        "{rel_path}: links to '{}', which resolves to '{normalised}' and is tracked by nothing",
        reference.text
    ))
}

/// A package-relative `tests/…` path, resolved against the naming file's own package and then every package.
fn package_relative_offence(
    files: &HashSet<String>,
    packages: &[String],
    rel_path: &str,
    raw: &str,
) -> Option<String> {
    // A package-RELATIVE path names nothing when the naming file belongs to no package: a root-level
    // governance document saying `tests/…` is describing some package's layout in general, or quoting a path
    // precisely because it exists nowhere — `[0.4.0]` records correcting one and names it in the sentence
    // that says so. Resolving it against every member instead would make that record an offence for being a
    // record.
    let home = package_of(files, rel_path)?;
    // It resolves against the referencing file's OWN package first — an example's README naming
    // `tests/reaction.rs` means that example's — and against every workspace member after, which is how a
    // governance document names one without repeating the crate.
    if holds(files, &format!("{home}/{raw}")) {
        return None;
    }
    if packages
        .iter()
        .any(|home| holds(files, &format!("{home}/{raw}")))
    {
        return None;
    }
    Some(format!(
        "{rel_path}: references '{raw}', which is tracked under no workspace member"
    ))
}

/// A path carrying a `/`, held against the tracked set and against git's own ignore rules.
fn qualified_offence(
    files: &HashSet<String>,
    members: &[String],
    root: &Path,
    rel_path: &str,
    raw: &str,
) -> Option<String> {
    if holds(files, raw) || ignored(root, raw) {
        return None;
    }
    // Illustrative rather than real, in two decidable forms.
    //
    // A `crates/<name>/…` path whose `<name>` is no tracked workspace member is a fixture in a doc comment
    // or a test — `crates/foo/src/lib.rs` — and reading it as a dangling reference would make every example
    // of the shape an offence. The rule needs the member set, which is why an empty one refuses in the
    // caller.
    if let Some(rest) = raw.strip_prefix("crates/") {
        let name = rest.split_once('/').map_or(rest, |(name, _)| name);
        if !members.iter().any(|m| m == name) {
            return None;
        }
    }
    Some(format!(
        "{rel_path}: references '{raw}', which is not tracked in this repository"
    ))
}

/// A bare basename, decided by whether this repository ever tracked it outside a change directory.
///
/// Several tracked files carrying it says nothing about which was meant, and one means the reference
/// resolves. NONE is the case the spec left open, and discarding it made this whole form inert — extracted
/// and thrown away, which is why twenty-odd references to files the 0.5.0 window deleted survived the sweep.
///
/// The decidable split: a name that WAS tracked, somewhere other than a change directory, is a stale
/// reference to something deleted. A name tracked only under `openspec/changes/` is the lifecycle's own
/// vocabulary — `proposal.md`, `tasks.md`, `design.md` are pruned at every sync by design — and a name never
/// tracked at all is not a path.
fn bare_basename_offence(
    basename_count: &std::collections::HashMap<&str, usize>,
    deleted_outside_changes: &BTreeSet<String>,
    rel_path: &str,
    raw: &str,
) -> Option<String> {
    if basename_count.contains_key(raw) || !deleted_outside_changes.contains(raw) {
        return None;
    }
    Some(format!(
        "{rel_path}: references '{raw}', which this repository deleted — point it at what replaced it, or \
         drop the reference"
    ))
}

fn offences_in(
    root: &Path,
    corpus_root: &Path,
    tracked_paths: &[String],
    corpus: &[String],
) -> BTreeSet<String> {
    let all = tracked_paths;
    let files: HashSet<String> = all.iter().cloned().collect();

    let members: Vec<String> = all
        .iter()
        .filter_map(|p| p.strip_prefix("crates/"))
        .map(|rest| rest.split_once('/').map_or(rest, |(name, _)| name))
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
    for path in all {
        *basename_count
            .entry(
                path.rsplit_once('/')
                    .map_or(path.as_str(), |(_, base)| base),
            )
            .or_default() += 1;
    }

    // Every basename this repository once tracked outside a change directory. A change directory's
    // scaffolding is pruned at every sync, so its names are the lifecycle's vocabulary rather than paths.
    let deleted_outside_changes: BTreeSet<String> = {
        let out = kanhe::hermetic_git::hermetic("git")
            .args(["log", "--diff-filter=D", "--name-only", "--format="])
            .current_dir(root)
            .output()
            .expect("run git log");
        // The sentence has one owner: `kanhe::hermetic_git::failed`. It stood verbatim at four sites in three
        // files, already diverged in what each printed beside it.
        assert!(
            out.status.success(),
            "{}",
            kanhe::hermetic_git::failed(
                "reading the deletion history",
                &out.status.to_string(),
                &String::from_utf8_lossy(&out.stderr)
            )
        );
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|p| !p.is_empty() && !p.starts_with("openspec/changes/"))
            .map(|p| p.rsplit_once('/').map_or(p, |(_, base)| base))
            .filter(|base| {
                !all.iter()
                    .any(|f| f.rsplit_once('/').map_or(f.as_str(), |(_, b)| b) == *base)
            })
            .map(str::to_string)
            .collect()
    };

    let mut offences: BTreeSet<String> = BTreeSet::new();
    let mut inspected = 0usize;

    for rel_path in corpus {
        if !is_inspected_source(rel_path) {
            continue;
        }
        // Counted before the exclusion below, because the guard downstream asks whether the **enumeration**
        // produced anything — a file deliberately left unjudged is still evidence that it did, while zero
        // files of either extension means the corpus never arrived.
        inspected += 1;
        // An active plan names what it intends to create, so judging it for existence refuses a proposal for
        // describing its own deliverable. The requirement states this exclusion and carries a scenario for it;
        // nothing held either until a plan first named a path that did not exist yet, and the check then
        // reported five offences against the change proposing them. Filtered here rather than at the caller,
        // so the fixture below exercises the same judgement the check runs.
        if rel_path.starts_with("openspec/changes/") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(corpus_root.join(rel_path)) else {
            panic!(
                "cannot read tracked file '{rel_path}' — a file this check claims to have inspected must \
                 have been read"
            );
        };
        let is_test_source = rel_path.contains("/tests/");
        let mut in_dated_section = false;

        for line in content.lines() {
            if rel_path == "CHANGELOG.md" && line.starts_with("## [") {
                in_dated_section = line.contains("] - ");
            }
            if !is_inspected_line(rel_path, line) {
                continue;
            }
            // **A dated section names what was true then**, and holding it to today's paths is the
            // falsification `release-coherence` refuses. Measured the hard way: a sweep that did not know
            // this rewrote eight hunks inside the released `[0.4.0]`, leaving it saying a Rust test
            // "normalizes a link target with portable shell". Measured again when this exemption was
            // narrowed: the dated sections carry eight unresolved paths and every one is a shell gate that
            // genuinely existed at `0.4.0` and was deleted when it migrated to Rust.
            //
            // `docs/history/` used to be exempt the same way, as a whole directory. It is not any more. The
            // facts a record must keep are shas, dates, versions and counts — **not paths** — and measured,
            // the exemption hid exactly one reference: a present-tense pointer at a gate that had moved
            // crates inside the 0.5.0 window, in the document the CHANGELOG advertises to adopters as the
            // provenance authority. Fourteen of the directory's fifteen path references already resolved,
            // so the blindness protected nothing and cost the one thing it was covering.
            //
            // **The exemption keys on *dated*, and dating is the freeze act — that is deliberate, and it is
            // the half a reader is most likely to challenge.** The version currently being prepared already
            // carries its date: `chore(release): prepare X.Y.Z` cuts `[Unreleased]` into
            // `## [X.Y.Z] - DATE`, and `release-coherence` then *requires* `[Unreleased]` to be empty in the
            // release-ready state — so from that commit until the release, every new entry is written into a
            // dated section this scan skips. The exemption is therefore widest exactly while the section is
            // still being edited.
            //
            // Kept as written, because the alternative reads worse: making the current version an exception
            // means this scan asking `release-coherence` which version is unreleased, so a reference verdict
            // would start depending on the release spine and a shallow checkout would move it. Measured on
            // the `0.5.0` section at the moment this was written — 2,572 lines, 41 distinct references
            // (22 prefixed paths, 19 bare basenames), **all resolving** — so the exposure is real and the
            // cost of it is not. Filed as WATCH in `BACKLOG.md` with the trigger that would change the answer:
            // a stale reference found inside the section of a version not yet released.
            if in_dated_section {
                continue;
            }
            for reference in extract(line) {
                let raw = reference.text.trim_end_matches(['.', ',', ')', '`']);
                let raw = raw.split_once('#').map_or(raw, |(path, _)| path);
                if raw.is_empty()
                    || raw.starts_with("http://")
                    || raw.starts_with("https://")
                    || raw.starts_with("mailto:")
                    || raw.contains("::")
                    || raw.contains('*')
                {
                    continue;
                }

                // The four forms, in the order they were always tried: a markdown link, a package-relative
                // `tests/…` path, a qualified path, and a bare basename. Each answers `Some(offence)` or
                // `None` for *resolves*, and they stay mutually exclusive because the dispatch is a chain —
                // the `continue` after each branch used to be what made that true, and an `else if` says it
                // rather than depending on every branch remembering to end.
                let offence = if reference.from_link {
                    link_offence(&files, rel_path, &reference, raw, is_test_source)
                } else if raw.starts_with("tests/") {
                    package_relative_offence(&files, &packages, rel_path, raw)
                } else if raw.contains('/') {
                    qualified_offence(&files, &members, root, rel_path, raw)
                } else {
                    bare_basename_offence(&basename_count, &deleted_outside_changes, rel_path, raw)
                };
                offences.extend(offence);
            }
        }
    }

    assert!(
        inspected > 0,
        "inspected 0 files — no tracked Markdown, Rust, TOML, or .gitignore source, so this check would \
         report clean without having read anything"
    );
    offences
}

/// Each extraction form, planted and required to be seen.
///
/// This is the direction the check had none of. Its verdict is that it found nothing, and a verdict of
/// that shape survives every detector being deleted — measured on all four forms, each disabled in turn,
/// green every time. The claim that they had been "disabled in turn" was therefore unfalsifiable, and one of
/// the four was in fact inert: the bare-basename branch extracted its references and discarded them, which is
/// why twenty-odd references to files the 0.5.0 window deleted survived the sweep.
#[test]
fn every_extraction_form_is_seen_when_it_names_something_absent() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked_paths = tracked(&root);

    let scratch = std::env::temp_dir().join(format!(
        "tianheng-reference-integrity-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("scratch is writable");
    std::fs::create_dir_all(scratch.join("crates/tianheng")).expect("scratch is writable");

    // The corpus is judged against THIS repository's tracked paths, so "absent" means absent here.
    let planted = [
        (
            "a qualified path",
            "probe-qualified.md",
            "See `crates/tianheng/src/zzz_absent.rs` for details.\n",
        ),
        (
            "a markdown link",
            "probe-link.md",
            // A ROOT-level target, which only the link form can see: the prefixed form needs a known
            // top-level directory and the bare form fires only on names this repository once tracked. A
            // probe both forms can see proves neither.
            "See [the doc](zzz-absent-root.md).\n",
        ),
        (
            "a package-relative test path",
            "crates/tianheng/probe-member.md",
            "See `tests/zzz_absent_probe.rs`.\n",
        ),
        // **The basename below must be one this repository once tracked and deleted, and that is the
        // extraction form's definition rather than an oversight.** `bare_basename_offence` reports a bare
        // run only when `deleted_outside_changes` holds it — otherwise a bare word is a word, not a
        // reference. So the `zzz`-sentinel form its neighbours use would leave this row inert: measured, a
        // rename to a never-tracked sentinel left it seen by nothing and this direction said so.
        //
        // Re-adding that script therefore does not silently repoint the probe — it fails this assertion,
        // naming the form. The premise is owned by a deletion, and this direction is what holds it; a review
        // reading the neighbours' sentinels as the rule proposed the rename, which is why the reason is
        // written here rather than left to the next one.
        //
        // The name is NOT repeated in this comment, deliberately: `in_repository_references_resolve` reads
        // executed Rust text, and a comment naming a deleted path is a stale reference to it. The row below
        // survives that check only because a backtick inside a string literal is written `\``, which the
        // reference form does not match — so an explanation of the row cannot be spelled the way the row is.
        (
            "a bare basename",
            "probe-basename.md",
            "The `check_dod_coherence.sh` gate says so.\n",
        ),
        (
            // A bare RUST basename, which the form excluded until the 0.5.0 window: the extension list admitted
            // the governance extensions only, so the branch below was inert for every Rust file this
            // repository has ever deleted. Its own row rather than trust in the `.sh` one, because what was
            // missing was an extension rather than a branch, and a row per branch cannot see that.
            "a bare Rust basename",
            "probe-rust-basename.md",
            "The shadowing lives in `collect.rs`.\n",
        ),
    ];
    for (_, path, body) in &planted {
        let full = scratch.join(path);
        std::fs::create_dir_all(full.parent().expect("a parent")).expect("scratch subdir");
        std::fs::write(full, body).expect("write the probe");
    }

    let mut unseen = Vec::new();
    for (form, path, _) in &planted {
        let offences = offences_in(&root, &scratch, &tracked_paths, &[path.to_string()]);
        if offences.is_empty() {
            unseen.push(format!("  {form} — planted in {path} and seen by nothing"));
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
    assert!(
        unseen.is_empty(),
        "an extraction form names something absent and the check says nothing:\n{}",
        unseen.join("\n")
    );
}

/// A **dated** CHANGELOG section keeps the paths it named then; everything else is held to today's tree.
///
/// The one place this check is deliberately blind, and until now it was blind by comment rather than by
/// anything that looks. Both directions on one body, differing only in whether the section heading carries a
/// date — without the control, the silence is satisfiable by a check that reads no CHANGELOG at all.
///
/// Why the blindness is right where it is: measured, this repository's dated sections carry eight unresolved
/// paths and every one is a shell gate that genuinely existed at `0.4.0` and was deleted when it migrated to
/// Rust. A sweep that did not know this rewrote eight hunks inside the released `[0.4.0]`, leaving it saying a
/// Rust test "normalizes a link target with portable shell" — a human falsifying a record to satisfy a check.
///
/// Why it stops there. `docs/history/` was exempt the same way, as a whole directory, and measured, that hid
/// exactly one reference: a present-tense pointer at a gate that had moved crates inside the 0.5.0 window, in the
/// document the CHANGELOG advertises to adopters as the provenance authority. Fourteen of the directory's
/// fifteen path references already resolved. The facts a record must keep are shas, dates, versions and
/// counts, and none of those is a path.
#[test]
fn a_dated_changelog_section_keeps_its_paths_and_an_undated_one_does_not() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked_paths = tracked(&root);
    let scratch = scratch("dated-section");
    let stale = "scripts/zzz_absent_reference_probe.sh";

    for (name, heading) in [
        ("dated", "## [0.4.0] - 2026-08-04"),
        ("undated", "## [Unreleased]"),
    ] {
        std::fs::write(
            scratch.join("CHANGELOG.md"),
            format!("# Changelog\n\n{heading}\n\n- it names `{stale}`.\n"),
        )
        .expect("write the probe changelog");
        let offences = offences_in(
            &root,
            &scratch,
            &tracked_paths,
            &["CHANGELOG.md".to_string()],
        );
        let named = offences.iter().any(|o| o.contains(stale));
        match name {
            "dated" => assert!(
                !named,
                "a dated section names what was true then, and holding it to today's tree is the \
                 falsification `release-coherence` refuses: {offences:?}"
            ),
            _ => assert!(
                named,
                "an undated section is not a record, so a stale path in it must react — without this the \
                 exemption above is satisfiable by a check that reads no CHANGELOG at all: {offences:?}"
            ),
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The bare Rust form reacts for a name this repository DELETED, and stays silent for one it never tracked.
///
/// Both directions on one body shape, differing only in the name, because the whole safety of admitting `.rs`
/// rests on that discriminator: this repository's prose is full of illustrative Rust filenames describing a
/// shape rather than naming a file — `weird.rs`, `never.rs`, `child.rs`, `foo.rs` — and every one of them
/// would be an offence if the form judged existence instead. Measured before admitting the extension: of the
/// bare Rust basenames this repository's documents name, the ones it once tracked and no longer does were the
/// only ones the form reports. The positive direction is the control; without it the negative is satisfiable
/// by a form that recognizes nothing.
#[test]
fn a_bare_rust_basename_reacts_only_for_a_name_this_repository_deleted() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked_paths = tracked(&root);
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-reference-integrity-rust-basename-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("scratch is writable");

    let deleted = "probe-deleted.md";
    let never = "probe-never-tracked.md";
    std::fs::write(
        scratch.join(deleted),
        "The shadowing lived in `collect.rs`.\n",
    )
    .expect("write the deleted-name probe");
    std::fs::write(
        scratch.join(never),
        "The shadowing lived in `zzz_never_tracked_probe.rs`.\n",
    )
    .expect("write the never-tracked probe");

    let seen_deleted = offences_in(&root, &scratch, &tracked_paths, &[deleted.to_string()]);
    let seen_never = offences_in(&root, &scratch, &tracked_paths, &[never.to_string()]);
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(
        !seen_deleted.is_empty(),
        "a bare Rust basename this repository deleted must be refused, or the silence below proves nothing"
    );
    assert!(
        seen_never.is_empty(),
        "a bare Rust basename this repository never tracked is an illustrative name, not a path, but got:\n{}",
        seen_never.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

#[test]
fn comment_bearing_sources_and_live_test_claims_are_inspected() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked_paths = tracked(&root);
    let fixture = scratch("comment-corpus");
    std::fs::write(fixture.join("anchor.md"), "No repository reference here.\n")
        .expect("write inspected anchor");
    let probes = [
        (
            "TOML comment",
            "probe.toml",
            "# See `scripts/zzz_absent_reference_probe.sh`.\n",
        ),
        (
            ".gitignore comment",
            ".gitignore",
            "# See `scripts/zzz_absent_reference_probe.sh`.\n",
        ),
        (
            "Rust test comment",
            "crates/kanhe/tests/probe.rs",
            "// `scripts/zzz_absent_integrity_probe.sh` holds this.\n",
        ),
        // The wrappers are shell, and they cite the Rust gates they sequence by path. A shebang above the
        // comment, because that is the shape every tracked script actually has and the line must not be read
        // as a reference to `/usr/bin/env`.
        (
            "shell comment",
            "scripts/probe.sh",
            "#!/usr/bin/env bash\n# The gate is `crates/kanhe/tests/zzz_absent_gate_probe.rs`.\n",
        ),
        // CI is where this repository's own gate list is duplicated, so its comments cite gates by path too.
        (
            "YAML comment",
            ".github/workflows/probe.yml",
            "jobs:\n  # Runs `crates/kanhe/tests/zzz_absent_gate_probe.rs`.\n  probe:\n",
        ),
    ];
    for (_, path, body) in probes {
        let full = fixture.join(path);
        std::fs::create_dir_all(full.parent().expect("a fixture parent"))
            .expect("create fixture parent");
        std::fs::write(full, body).expect("write fixture source");
    }

    let unseen: Vec<&str> = probes
        .iter()
        .filter_map(|(direction, path, _)| {
            offences_in(
                &root,
                &fixture,
                &tracked_paths,
                &["anchor.md".to_string(), path.to_string()],
            )
            .is_empty()
            .then_some(*direction)
        })
        .collect();
    let _ = std::fs::remove_dir_all(&fixture);

    assert!(
        unseen.is_empty(),
        "recognized stale references were inspected by nothing: {}",
        unseen.join(", ")
    );
}

/// `reference-integrity`'s scenario *An active OpenSpec plan names future paths*, held.
///
/// One body, two locations. Outside a plan it must be refused; inside one it must not — so the exclusion
/// cannot pass by the reference being unrecognizable, which is what asserting only the second half would
/// allow.
#[test]
fn an_active_plan_may_name_a_path_it_intends_to_create() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked_paths = tracked(&root);
    let scratch = std::env::temp_dir().join(format!(
        "tianheng-reference-integrity-plan-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&scratch);
    xingbiao::claim_scratch(&scratch).expect("scratch is writable");

    // Under a **tracked** member: an untracked crate directory is unenforceable by design, so a probe there
    // would be unrefused for a reason that has nothing to do with the exclusion being tested.
    let body = "The member will hold `crates/tianheng/src/zzz_absent_planned_dir/`.\n";
    let outside = "probe-outside-a-plan.md";
    let inside = "openspec/changes/zzz-probe-plan/proposal.md";
    for path in [outside, inside] {
        let full = scratch.join(path);
        std::fs::create_dir_all(full.parent().expect("a parent")).expect("scratch subdir");
        std::fs::write(full, body).expect("write the probe");
    }

    let seen_outside = offences_in(&root, &scratch, &tracked_paths, &[outside.to_string()]);
    let seen_inside = offences_in(&root, &scratch, &tracked_paths, &[inside.to_string()]);
    let _ = std::fs::remove_dir_all(&scratch);

    assert!(
        !seen_outside.is_empty(),
        "the same reference outside a plan must be refused, or the exclusion below proves nothing"
    );
    assert!(
        seen_inside.is_empty(),
        "an active plan naming a path it intends to create must not be a stale reference, but got:\n{}",
        seen_inside.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

// --- a reference nothing can check ------------------------------------------------------------------------
//
// The rest of this file asks whether a named path exists. This asks whether a reference was written in a form
// anything could ask that about.
//
// There is a ladder, and this repository has been down it. An intra-doc link is checked by the compiler; a
// path is checked by the sweep above; a path with a line number is checked by nothing; and a reference naming
// only a position is not even a name. Measured here: one such reference was off by 86 lines and another by 98,
// and the second was written by someone who had just been corrected about the first — which is the criterion
// `scripts/publish.sh` states for itself, that a rule stated and then missed needs a check rather than another
// sentence.
//
// **Every shape this refuses, and every reading it must leave alone, is a string in
// `every_positional_shape_reacts_and_a_named_thing_does_not`.** Named there and not described here, because
// this comment is inside the corpus: prose about the rule, written in the shapes the rule refuses, is the
// self-reading trap this repository has met before. The specimens live where they can be executed.
//
// **Comment lines only, through the same `is_inspected_line` rule as the sibling sweep.** That is a position,
// not a marker: a specimen written as a string literal is on an executed line and cannot be mistaken for a
// reference, and nothing can hide a comment inside one. It also settles the corpus question the same way for
// both properties in this file.
//
// **Every line-comment format, derived from [`FORMATS`] rather than listed again.** This filtered `.rs` and
// `.sh` by extension — a second list beside the declaration, which is the defect the declaration was introduced
// one change earlier to end. It left `.toml`, `.yml`, `Cargo.lock`, `CODEOWNERS` and `.gitignore` unswept, each
// of them source where a positional phrase rots exactly as it does in Rust, and nothing about the Markdown
// reasoning below reached any of them. Deriving the scope means a format admitted to the corpus is swept for
// both properties or for neither.
//
// Markdown stays out, and now by CONSTRUCTION rather than by omission: it is the one format classified as
// `Prose::Whole`, so it cannot be a line-comment format. In a record — a `CHANGELOG.md` entry, a `BACKLOG.md`
// history — a positional phrase legitimately narrates a past state, and telling that from a live reference is a
// judgement over prose, the instrument this repository designed, measured three times and rejected. In source
// there is no such reading: a comment describes the file it is in, so the reference is either live and rotting
// or narration that could have named the construct instead. Rephrasing costs a word.

/// The number words a positional reference is actually written with. A digit run counts too.
///
/// One array rather than a rule about spelling, because the alternative is a matcher that reads a count of
/// thirty-one and not one of thirty. This is the vocabulary prose uses; a count large enough to fall outside it
/// is a reference no reader was going to follow.
const POSITIONAL_COUNTS: [&str; 12] = [
    "one", "two", "three", "four", "five", "six", "seven", "eight", "nine", "ten", "eleven",
    "twelve",
];

/// The words that are **pure position**, carrying no thing of their own. Used by the article branch only.
///
/// **The counted branch used this list too, and that was the defect.** A count applied to any noun is an
/// offset — the requirement forbids *a counted offset* and names no vocabulary — so gating it on four words
/// made a second list beside the rule, joined to nothing and necessarily narrower than it.
///
/// **It stays for the article branch**, because the requirement's wording there turns on the noun: *a
/// definite article naming no thing*. A line is pure position and names nothing; a construct named and
/// located is a reference to a thing, which the rule permits and this check's quiet half asserts. Removing
/// the list from both branches was tried and measured, and took the tree an order of magnitude further —
/// almost entirely onto the `the <construct>` phrases the requirement allows. One list, one branch.
///
/// No figure is written here. The refusal prints its own count and its own list, and a number in prose
/// beside a reaction that produces one is the second owner this repository removes on sight.
/// **The plurals were half-present.** `line` carried `lines` and the other two carried nothing, so a phrase
/// counting paragraphs or sentences fell outside a list whose own doc says the article case takes the plural
/// too. Found by writing the reaction row for a widened direction and watching it not fire — the same
/// half-widened shape this constant's own history is about, one noun over.
const POSITIONAL_UNITS: [&str; 6] = [
    "lines",
    "line",
    "paragraphs",
    "paragraph",
    "sentences",
    "sentence",
];

/// The adverbs that stand in for the thing a reference should have named, each with the directions it may
/// stand before.
///
/// **One of them cannot stand before every direction, and the difference is a word rather than a branch.**
/// Before a direction carrying no sense but position it locates; before a relation-capable one the same word
/// is an intensifier, naming no position at all. Measured: this repository writes that intensifier in its own
/// tracked Markdown, out of corpus only because whole-document prose is not a line-comment format. The other
/// adverbs carry no second sense.
///
/// **The specimens are not written here**, for the reason [`POSITIONAL_UNIT_DIRECTIONS`] already records —
/// and drafting this paragraph produced one: a locating phrase quoted in a comment lands in the corpus and
/// this direction reported it. Both readings live on executed lines in
/// `every_positional_shape_reacts_and_a_named_thing_does_not`.
///
/// The pairing travels in the same array as the adverb, so a new adverb answers the question instead of
/// inheriting whichever answer the branch happened to apply — the shape `merge_message_gate`'s attribution
/// marks already use, for the same reason.
const POSITIONAL_ADVERBS: [(&str, Pairs); 4] = [
    ("just", Pairs::EveryDirection),
    ("immediately", Pairs::EveryDirection),
    ("directly", Pairs::EveryDirection),
    ("right", Pairs::PositionOnlyDirections),
];

/// Which directions an adverb stands in for a position before.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Pairs {
    /// Positional before every direction word.
    EveryDirection,
    /// Positional only before a direction carrying no sense but position, because this adverb has a second
    /// sense that a relation-capable direction completes.
    PositionOnlyDirections,
}

/// The direction words that locate a thing and mean nothing else.
///
/// **The direction was two words written inline, and that was the same defect [`POSITIONAL_UNITS`] records
/// fixing one dimension over.** That doc says a list gating the counted branch made *a second list beside the
/// rule, joined to nothing and necessarily narrower than it* — and the direction stayed `["above", "below"]`,
/// which is that shape exactly. The requirement forbids *a counted offset* and names no direction vocabulary,
/// so a phrase counting a unit was invisible for writing `down` instead of `below`.
///
/// They admit **any noun**, because they carry no sense but position.
const POSITIONAL_DIRECTIONS: [&str; 2] = ["above", "below"];

/// The direction words that locate a thing **or** name a relation, admitted only over a [`POSITIONAL_UNITS`]
/// noun.
///
/// **Split from [`POSITIONAL_DIRECTIONS`] by measurement, not by taste.** Widening the one list to include
/// these and keeping *any noun* was written first and run — measured at `afe51fd`, with these added to the
/// then-inline direction list and no unit restriction, by
/// `--exact no_tracked_source_names_a_position_instead_of_a_thing`: 20 offences on a tree that was green.
/// Both halves are needed to re-run it, and the commit that recorded the measurement also repaired comments
/// the sweep reads, so what the anchor names is a pair rather than a number.
///
/// **That most of them were not references is a judgement, and it is the half this split rests on.** Naming
/// the commit binds the count and nothing else. What is inspectable is the relation shapes kept as quiet rows
/// in `every_positional_shape_reacts_and_a_named_thing_does_not` — `one level up` and `a layer down` name a
/// relation between two rules rather than a place to look, and `three levels up` counts directories; the rest
/// of that majority is an adjective. Reporting a relation is the false refusal
/// `repository-checks` already forbids this family — refuse a shape for what it is, not for what it resembles.
///
/// **The restriction takes two forms, because the readings take a noun differently.** The counted and article
/// readings require a [`POSITIONAL_UNITS`] noun; the adverb reading requires no noun at all, so it instead
/// drops the adverbs carrying a second sense a relation-capable direction completes. Written as one condition
/// on the counted branch it reached neither of the others, and an intensifier before a relation-capable
/// direction was read as a position — latent, because no tracked comment carried the shape.
///
/// **The specimens are not written here**, for the reason the counted branch already records: quoting one
/// lands it in the corpus, where it refuses itself. A first draft of this paragraph quoted three and this
/// direction reported two of them. They live on executed lines in
/// `every_positional_shape_reacts_and_a_named_thing_does_not`.
const POSITIONAL_UNIT_DIRECTIONS: [&str; 4] = ["up", "down", "higher", "lower"];

/// Whether the direction found at `index` is a whole word rather than the tail of one.
///
/// **Measured, on the first run of the widened list** — the run [`POSITIONAL_UNIT_DIRECTIONS`] anchors. `up`
/// matched inside `group` and `d up` inside a word break, so `gro up` and `d up` were reported as positional
/// references. `above` and `below` never showed it because neither is a common substring, which is why they
/// survived without this guard. The counted branch already applies the same test to its count; the direction
/// had no equivalent.
fn is_whole_word(lower: &str, index: usize, direction: &str) -> bool {
    let before_ok = lower[..index]
        .chars()
        .next_back()
        .is_none_or(|c| !c.is_ascii_alphanumeric());
    let after_ok = lower[index + direction.len()..]
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphanumeric());
    before_ok && after_ok
}

/// The positional reference `line` carries, if it carries one.
///
/// Three shapes, because prose writes it three ways: a counted unit, a definite article naming no thing, and an
/// adverb standing in for one. A bare direction word is none of them — a construct plus a direction to find it
/// is a reference to a thing, and the quiet half of this rule's direction asserts that.
///
/// The article case takes the plural too. The hand sweep that preceded this check wrote its pattern with the
/// singular only, and this check's first tree-wide run found the instance it had missed.
/// The byte index just past the last non-alphanumeric character, or `0` if there is none.
///
/// **`at + 1` is wrong here and panicked on this repository's own text.** `rfind` yields the byte index a
/// character *starts* at, so adding one lands inside any character wider than a byte — measured, a comment
/// containing a CJK name aborted the run with *start byte index 31 is not a char boundary*. The `rsplit`
/// this replaced was boundary-safe for free; hand-rolled offsets are not, and the width has to be asked for.
fn after_last_break(text: &str) -> usize {
    text.char_indices()
        .rev()
        .find(|(_, character)| !character.is_ascii_alphanumeric())
        .map_or(0, |(at, character)| at + character.len_utf8())
}

fn positional_reference(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    for (direction, relation_capable) in POSITIONAL_DIRECTIONS
        .into_iter()
        .map(|d| (d, false))
        .chain(POSITIONAL_UNIT_DIRECTIONS.into_iter().map(|d| (d, true)))
    {
        for (index, _) in lower.match_indices(direction) {
            if !is_whole_word(&lower, index, direction) {
                continue;
            }
            let before = lower[..index].trim_end();
            // The noun the direction applies to, and what sits before it — with the gap between them kept,
            // because two of the conditions in `counted` turn on that gap rather than on the words.
            let noun_start = after_last_break(before);
            let noun = &before[noun_start..];
            let head = before[..noun_start].trim_end();
            let count_start = after_last_break(head);
            let count = &head[count_start..];
            let count_prefix = head[..count_start].chars().next_back();

            // **Counted: any noun, and the count must be a count.** A count applied to anything is an
            // offset, so the noun is read rather than matched against a list. Two conditions keep that from
            // becoming its opposite error, each measured against a live phrase it was refusing:
            //
            //   the count is a whole word    a count welded into a name is not counting the noun;
            //                                 `round-9 finding above` names a finding called round-9
            //   the count is adjacent         a count separated from the noun by punctuation is in another
            //                                 clause, and `after_last_break` yields it empty
            //
            // A `gap == " "` condition was written here for the second and **removed after measuring it**:
            // the emptiness check already excluded every live instance, so the condition was inert — and it
            // would have introduced a false negative of its own, refusing to see a phrase written with two
            // spaces. A discriminator that changes no verdict is a claim, not a guard.
            //
            // Neither adds a vocabulary. A third candidate did — refusing a copula as the noun — and was
            // dropped: a list of verbs is the shape just removed from this branch, and the phrase it would
            // have spared reads better named anyway.
            //
            // **The specimens are not written here.** Three times during this repair a positional phrase
            // quoted as an example landed in the corpus and refused itself. They live on executed lines in
            // `every_positional_shape_reacts_and_a_named_thing_does_not`, which is where this file already
            // kept them and where the discipline says they belong.
            // A direction that also names a relation is admitted only over a unit — see
            // [`POSITIONAL_UNIT_DIRECTIONS`] for the run that decided it, and for what that run does and
            // does not establish. This is one of the two forms that restriction takes; the adverb reading
            // takes the other, because it requires no noun for a list to admit.
            let noun_admitted = !relation_capable || POSITIONAL_UNITS.contains(&noun);
            let counted = noun_admitted
                && !noun.is_empty()
                && !count.is_empty()
                && !count_prefix.is_some_and(|c| c.is_ascii_alphanumeric() || c == '-')
                && (count.chars().all(|c| c.is_ascii_digit())
                    || POSITIONAL_COUNTS.contains(&count));
            // **Article: only a unit.** The requirement's article clause turns on the noun — pure position
            // names nothing, while a construct named and located is a reference to a thing the rule permits.
            let article = head.ends_with("the") && POSITIONAL_UNITS.contains(&noun);
            if counted || article {
                return Some(format!("{noun} {direction}"));
            }
            // **The restriction's other form.** An adverb stands in for the thing, so this reading requires
            // no noun — and the unit test above therefore cannot reach it. Written as a conjunct of `counted`
            // it governed that branch alone, and an intensifier before a relation-capable direction was read
            // as a position: the restriction reached the readings that happened to have a noun. Which adverbs a
            // relation-capable direction admits is read from [`POSITIONAL_ADVERBS`], where the pairing sits
            // beside the word.
            //
            // Whole-word on the adverb as well as on the direction: `outright above` ends with `right`.
            for (adverb, pairs) in POSITIONAL_ADVERBS {
                if relation_capable && pairs == Pairs::PositionOnlyDirections {
                    continue;
                }
                if before.ends_with(adverb)
                    && is_whole_word(before, before.len() - adverb.len(), adverb)
                {
                    return Some(format!("{adverb} {direction}"));
                }
            }
        }
    }
    None
}

/// The phrases that name a **moving** reference, so the passage is stale the moment what they point at moves.
///
/// `AGENTS.md`'s table of what earns a place in a doc comment gives this row its verdict — *neither* an
/// observation source nor provenance, because it names a moving reference — and says of the same table that
/// this is the one row a sweep can enumerate. The others need the criterion applied per site, which is prose
/// judgement this repository has designed, measured three times and rejected.
///
/// **Admitted by instance or by the rule's own text, and by nothing else.** Most were live in tracked line
/// comments when the sweep was written; the remainder is admitted because `AGENTS.md`'s row spells it out.
/// Nothing is added on the strength of sounding similar: an entry that closes nothing reads as a defence that
/// was never there, which is what `AMBIENT_IGNORE_READS` records about admitting a spelling with no call site.
///
/// **The members are not restated in this passage, and that is the check's own lesson applied to itself.**
/// Naming one here put a live instance in the corpus — the sweep reported its own documentation, which is
/// verbatim what `projection-register` records about a check whose subject is text. The list below is the one
/// owner; a reader wanting the members reads them.
///
/// **`the same window` is deliberately absent.** It can be anchored by what precedes it — *the same window as
/// `64ed18c`* names a moment — so deciding it means reading what the sentence points back to, and that is the
/// prose instrument this repository refuses. Seven passages carry it; each is a reviewer's call.
const RELATIVE_ANCHORS: [&str; 4] = [
    "this window",
    "for a window",
    "one commit ago",
    "the previous round",
];

/// Every relative anchor the comment lines of `corpus` carry, in `corpus_root`.
///
/// **Runs of comment lines are joined before matching, which is the whole difficulty.** A wrapped comment
/// splits a phrase across lines — `scripts/publish.sh` carries `for a` at one line's end and `window` at the
/// next line's start — so a per-line sweep sees six of the seven live instances and reports the seventh
/// clean. The marker is stripped **before** the join for the same reason: joining raw lines leaves `for a #
/// window`, which matches nothing either.
///
/// Split from the check so a negative fixture can call it, exactly as the sibling positional sweep is.
fn relative_anchor_offences_in(corpus_root: &Path, corpus: &[String]) -> BTreeSet<String> {
    let mut offences = BTreeSet::new();
    let mut read = 0usize;
    for path in corpus.iter() {
        let Some(Prose::LineComment(marker)) = prose_of(path) else {
            continue;
        };
        let text = std::fs::read_to_string(corpus_root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        read += 1;
        // One run of consecutive comment lines is one passage. A blank comment line joins as a space and
        // separates nothing, which is right: a phrase does not cross a paragraph, and treating the blank as
        // a break would re-open the wrap the join exists to close.
        //
        // Each line's offset into the joined text is kept so a hit reports the line the phrase **ends** on
        // rather than the line the passage began on. Without it a wrapped file header reports line 1 — and a
        // shell script's `#!` opens the run, so every offence in one would have named the shebang.
        //
        // Whitespace is normalised per line before the join rather than over the result, so the offsets stay
        // valid: a phrase wrapped across two lines is matched only if the join is single-spaced, and
        // normalising afterwards would move every offset it had just been compared against.
        let mut passage = String::new();
        let mut origins: Vec<(usize, usize)> = Vec::new();
        let flush = |passage: &mut String,
                     origins: &mut Vec<(usize, usize)>,
                     offences: &mut BTreeSet<String>| {
            for anchor in RELATIVE_ANCHORS {
                let mut from = 0usize;
                while let Some(at) = passage[from..].find(anchor) {
                    let end = from + at + anchor.len();
                    let line = origins
                        .iter()
                        .take_while(|(offset, _)| *offset < end)
                        .last()
                        .map_or(0, |(_, line)| *line);
                    offences.insert(format!(
                        "  {path}:{line} writes `{anchor}`, which names a moving reference — it is stale the \
                         moment that reference moves, and nothing can check it. Anchor it to the moment (a \
                         version, a date, a commit) or drop it: the sentence almost always means the same \
                         without it"
                    ));
                    from = end;
                }
            }
            passage.clear();
            origins.clear();
        };
        for (index, line) in text.lines().enumerate() {
            match line.trim_start().strip_prefix(marker) {
                Some(rest) => {
                    // **A doc comment extends the marker, and the extension has to go too.** `FORMATS`
                    // declares what a line comment *opens* with, which is right for it: `///` and `//!` both
                    // open with `//`. Stripping only that leaves `/` or `!` at the front of the line's
                    // contribution, and it lands inside the joined phrase — verbatim the failure this
                    // function's own comment describes for an unstripped `#`. Measured: with only the
                    // declared marker stripped, `publish_source.rs`'s wrapped `one commit / ago` joined as
                    // `one commit / ago` and was reported clean, in a tracked file inside the corpus.
                    //
                    // Trimming rather than a longest-first table, because the table would be a second
                    // declaration of what `FORMATS` already owns. Only leading glyphs are trimmed, so a
                    // comment whose text begins with a path — `// /etc/hosts` — keeps it: the space after
                    // the marker stops the trim.
                    let rest = rest.trim_start_matches(['/', '!']);
                    let normalised = rest.split_whitespace().collect::<Vec<_>>().join(" ");
                    passage.push(' ');
                    origins.push((passage.len(), index + 1));
                    passage.push_str(&normalised);
                }
                None => flush(&mut passage, &mut origins, &mut offences),
            }
        }
        flush(&mut passage, &mut origins, &mut offences);
    }
    assert!(
        read > 0,
        "no line-comment source was read, so this sweep would report clean over a corpus it never opened"
    );
    offences
}

/// The wrap, in both marker shapes, and the readings that must NOT react.
///
/// **The sweep claimed a negative fixture and had none**, which is how it shipped seeing only one of the two
/// wrap shapes it is for. Its own doc said it was split from the check *so a negative fixture can call it*;
/// nothing called it, so the only thing holding the wrap was the tree happening to contain a shell instance —
/// and the tree's majority shape, a Rust doc comment, went unread. A residue a repair states is the next
/// direction's specification, and this is that direction.
///
/// Four rows, and the quiet ones carry the weight. A sweep that stripped no marker would pass the first row
/// and fail the rest; one that stripped only the declared marker would pass the first two and fail the doc
/// forms — which is exactly what shipped. The executed row is what stops a matcher from reading the
/// specimens on this page as the corpus.
#[test]
fn a_wrapped_anchor_reacts_in_every_marker_shape() {
    let fixture = std::env::temp_dir().join(format!("kanhe-anchor-wrap-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&fixture);
    xingbiao::claim_scratch(&fixture).expect("create the fixture root");

    let shell = "wrapped.sh";
    let doc = "wrapped_doc.rs";
    let inner = "wrapped_inner.rs";
    let executed = "executed.rs";
    for (path, body) in [
        (
            shell,
            "# the second question was missing for a
# window, and the sibling paid for it
",
        ),
        (
            doc,
            "/// the twin was converged one commit
/// ago and this one was not
",
        ),
        (
            inner,
            "//! a per-line predicate extracted in this
//! window falsifies that reading
",
        ),
        (
            executed,
            "const SPECIMEN: &str = \"extracted in this window\";\n",
        ),
    ] {
        std::fs::write(fixture.join(path), body).expect("write fixture source");
    }
    let offences = relative_anchor_offences_in(
        &fixture,
        &[
            shell.to_string(),
            doc.to_string(),
            inner.to_string(),
            executed.to_string(),
        ],
    );
    let _ = std::fs::remove_dir_all(&fixture);

    let listed = offences.iter().cloned().collect::<Vec<_>>().join("\n");
    assert_eq!(
        offences.len(),
        3,
        "every wrapped marker shape must react and the executed line must not:\n{listed}"
    );
    for reacting in [shell, doc, inner] {
        assert!(
            offences.iter().any(|o| o.contains(reacting)),
            "`{reacting}` wraps an anchor across two comment lines and must react:\n{listed}"
        );
    }
    assert!(
        !offences.iter().any(|o| o.contains(executed)),
        "an anchor written as a string literal sits on an executed line and is not a comment:\n{listed}"
    );
}

/// `AGENTS.md`'s relative-anchor row, held over every tracked line comment.
#[test]
fn no_tracked_source_names_a_relative_anchor() {
    let Some(root) = workspace_root() else {
        return;
    };
    let offences = relative_anchor_offences_in(&root, &tracked(&root));
    assert!(
        offences.is_empty(),
        "{} relative anchor(s) in tracked source:\n{}",
        offences.len(),
        offences.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Every positional reference the comment lines of `corpus` carry, in `corpus_root`.
///
/// Split from the check so a negative fixture can call it, for the reason the sibling sweep states: a check
/// whose only assertion is that it found nothing cannot be shown to find anything.
///
/// The corpus is every format [`FORMATS`] classifies as carrying line comments — not a second extension list.
/// `Prose::Whole` is excluded by not being `LineComment`, which is how Markdown stays out by construction.
fn positional_offences_in(corpus_root: &Path, corpus: &[String]) -> BTreeSet<String> {
    let mut offences = BTreeSet::new();
    let mut read = 0usize;
    for path in corpus
        .iter()
        .filter(|p| matches!(prose_of(p), Some(Prose::LineComment(_))))
    {
        // A file this direction claims to have inspected must have been read. `read > 0` below is a
        // **vacuity** guard and not a completeness one: it catches every file being unreadable and says
        // nothing about one of them being, which would leave this sweep reporting clean over a corpus it
        // never opened in full. Two sibling directions in this file already refuse exactly this.
        let text = std::fs::read_to_string(corpus_root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        read += 1;
        for (index, line) in text.lines().enumerate() {
            if !is_inspected_line(path, line) {
                continue;
            }
            if let Some(shape) = positional_reference(line) {
                offences.insert(format!(
                    "  {path}:{} writes `{shape}`, which names a position rather than a thing — nothing can \
                     check it and it rots on the next edit. Name the item instead: an intra-doc link if the \
                     docs can reach it, otherwise the identifier",
                    index + 1
                ));
            }
        }
    }
    assert!(
        read > 0,
        "no Rust or shell source was read, so this sweep would report clean over a corpus it never opened"
    );
    offences
}

/// `reference-integrity`'s scenario *A reference names a position rather than a thing*, held tree-wide.
#[test]
fn no_tracked_source_names_a_position_instead_of_a_thing() {
    let Some(root) = workspace_root() else {
        return;
    };
    let all = tracked(&root);
    let offences = positional_offences_in(&root, &all);
    assert!(
        offences.is_empty(),
        "{} positional reference(s) in tracked source:\n{}",
        offences.len(),
        offences.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}

/// Each shape is seen, and the readings that must NOT react are not.
///
/// The quiet half is the load-bearing one. A matcher keyed on the direction word alone would refuse a construct
/// followed by a direction to find it — which is a reference to a thing — and asserting only that the shapes
/// react would be satisfied by a matcher that refuses every occurrence of the word.
///
/// These strings are the specimens the section comment declines to write, and they are here so they sit on
/// executed lines rather than in the corpus.
#[test]
fn every_positional_shape_reacts_and_a_named_thing_does_not() {
    for reacting in [
        "// The signing probe seven lines below checks its own write.",
        "# `--workspace` written three lines below, so the invocation reads as the whole",
        "/// stood over a state the line above had made unreachable.",
        "// the `inline_only` decision just below (a false negative)",
        "# the gate immediately above this one",
        "/// says so two lines above the allowlist",
        "// listed in `ALL` on the lines below it",
        "// the 12 lines below",
        "/// the paragraph above states it",
        // The counted branch reads any noun, not a unit list. These are shapes it missed while it did,
        // every one taken from a live comment this repository was carrying.
        "/// enumerated once for the two directions below.",
        "// one direction below carries the same shape",
        "// what the two statements above already guarantee",
        "// the two cases below that carry `--package`",
        // The directions that also name a relation, over a unit. Every one is a live comment this
        // repository was carrying under a green reaction, because the direction list was `above`/`below`
        // and these say the same thing in the words prose actually reaches for.
        "// the window five lines down did not",
        "/// reads as `NotFound` one line up (so this path runs)",
        "// the half that told them apart sat three lines lower",
        "// stated two paragraphs higher",
        // The ADVERB reading over a relation-capable direction — the cell the unit restriction cannot reach,
        // because this reading supplies the noun's place itself. Both rows were absent while the restriction
        // was a conjunct of the counted branch, which is why nothing held its claim to drop every relation.
        "// the value is set directly up from here",
        // The control for the `right` pairing: before a direction carrying no second sense, the same word
        // locates and must still react. Without this row, dropping `right` everywhere would pass.
        "// the guard right above this call",
    ] {
        assert!(
            positional_reference(reacting).is_some(),
            "this shape must be seen: {reacting}"
        );
    }
    for quiet in [
        "// a bare-`#[cfg]`-tolerated declaration (tolerated below) can resolve to nothing",
        "// The classification is written above the loop it governs.",
        "/// See [`sign_probe`], which checks its own write.",
        "// above",
        "// nine of them",
        // The two discriminators that keep the widened counted branch from becoming its opposite error.
        // Each is a live phrase the branch refused before them, and each fails a different condition.
        "// The `cfg_if!` form of the round-9 finding above",
        "/// it is one component and stays one (asserted above, where it does run)",
        // A direction that is the tail of a word is not a direction. Measured: the first widened run
        // reported three of these, `up` inside `group` among them.
        "// the members of one group are enumerated once",
        "// a second round wound back to the same shape",
        // A relation is not an offset. These are the phrases the unit restriction exists to spare, and
        // reporting them would be the false refusal this family forbids — refuse a shape for what it is,
        // not for what it resembles. Every one is live in this repository.
        "// the same narrowing one level up from it",
        "// the ownership is inherited from a layer down",
        "// resolved three levels up from the manifest",
        // `right` before a relation-capable direction is the intensifier English writes, not a position —
        // this repository writes it twice in its own Markdown, out of corpus only by format.
        "// this bubbles right up to the caller",
        // An adverb that is the tail of a word is not an adverb, the same test the direction already gets.
        "// an outright above-the-line claim names no position",
    ] {
        assert!(
            positional_reference(quiet).is_none(),
            "this must not react: {quiet}"
        );
    }
}

/// A positional reference in a fixture source is found, and one in a fixture's executed line is not.
///
/// The tree-wide direction asserts a clean tree, which deleting the matcher can only make cleaner. This shows
/// it finding something — and shows the position rule doing the work, since the two fixture files carry the
/// same phrase in a comment and in a string literal.
#[test]
fn a_positional_reference_reacts_only_from_a_comment() {
    let fixture = scratch("positional-corpus");
    let commented = "crates/probe/src/lib.rs";
    let executed = "crates/probe/src/other.rs";
    // A non-Rust line-comment format, so the corpus is shown to be derived rather than the two extensions this
    // filtered before. `.toml` was outside the sweep while being exactly the source the reasoning covers.
    let manifest = "crates/probe/Cargo.toml";
    for (path, body) in [
        (commented, "// The guard four lines above holds it.\n"),
        (
            executed,
            "const SPECIMEN: &str = \"the guard four lines above holds it\";\n",
        ),
        (manifest, "# The dependency two lines below needs it.\n"),
    ] {
        let full = fixture.join(path);
        std::fs::create_dir_all(full.parent().expect("a fixture parent"))
            .expect("create fixture parent");
        std::fs::write(full, body).expect("write fixture source");
    }
    let offences = positional_offences_in(
        &fixture,
        &[
            commented.to_string(),
            executed.to_string(),
            manifest.to_string(),
        ],
    );
    let _ = std::fs::remove_dir_all(&fixture);

    let listed = offences.iter().cloned().collect::<Vec<_>>().join("\n");
    assert_eq!(
        offences.len(),
        2,
        "both comments must react and the executed line must not:\n{listed}"
    );
    for reacting in [commented, manifest] {
        assert!(
            offences.iter().any(|o| o.contains(reacting)),
            "`{reacting}` must react:\n{listed}"
        );
    }
}

/// A backticked coordinate — `` `path:NNN` `` — is refused wherever it is written.
///
/// The sibling sweep above refuses a positional reference in every **line-comment** format, and Markdown sits
/// outside it by construction: a positional *phrase* in a record narrates a past state, and separating that from
/// a live reference is a judgement over prose this repository has designed, measured and declined. That reasoning
/// covers phrases and stops there.
///
/// A structured coordinate is not a phrase. It is decidable by **shape**, exactly as a bound id is, so refusing
/// it reads nothing around it and reopens no declined judgement. And the ladder's own argument reaches it without
/// help: a position is not a name, and it is not one in any tense — a record citing a coordinate serves its reader
/// no better than a live clause does, because neither can be checked and both rot on any edit above them.
///
/// **Refused, not resolved.** The other reference kinds resolve to an identity and fail when it is absent. A
/// coordinate cannot: a changelog path with a line number is *valid* while naming nothing anyone
/// meant, because the file does have such a line. Validity is the trap, so refusal is the only answer
/// that bites.
///
/// **The left side must be a tracked path**, produced by the enumeration this file already uses. That is what
/// keeps `1:1`, `note:5` and a clock time out of the corpus, and it is the same construction that made bare
/// bound ids precise: require the left side to name something the repository enumerates.
#[test]
fn no_reference_names_a_line_number() {
    let Some(root) = workspace_root() else {
        return;
    };
    let paths = tracked(&root);
    assert!(
        !paths.is_empty(),
        "no tracked path was enumerated, so this direction would report clean over nothing"
    );
    let known: std::collections::BTreeSet<&str> = paths.iter().map(String::as_str).collect();

    let mut coordinates = Vec::new();
    let mut read = 0usize;
    for path in &paths {
        // A file this direction claims to have inspected must have been read. Skipping an unreadable one is
        // the vacuity this file already refuses elsewhere for exactly the same reason: with every tracked file
        // unreadable the verdict would be clean over nothing examined, and clean-over-nothing is
        // indistinguishable from clean.
        let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        read += 1;
        // **The paragraph is the unit that pairs, and a line is not.** A Markdown code span wraps a line
        // freely, so a per-line reader joins one span's closer to the next line's opener and judges the
        // prose between them. A span cannot contain a blank line, so the paragraph is where they close, and
        // the paragraphs left odd are fenced blocks and doubled markers.
        //
        // The previous repair scanned an undecidable *line* entire and called that an over-reaction in the
        // safe direction. It was inert: the span then contains a backtick, so its left half is neither a
        // tracked path nor empty and the coordinate shape can never match. The hole was declared closed and
        // was not.
        //
        // **Whether a line pairs is the reader's answer, asked once, and which answer this check wants is
        // in the name it calls.** A `% 2 == 0` guard stood in front of `backticked` and decided the same
        // predicate the reader decides, so the `Err` arm behind it was a branch nothing could take and the
        // vector it filled was asserted empty by a guard that could never fire. Then the two arms were
        // written here, which kept this check's *scan the whole line* reading apart from its three siblings'
        // *refuse* reading by maintenance rather than by construction.
        let spans = kanhe::reading::backticked_by_paragraph(&text);
        for (index, span) in &spans {
            // Split on the FIRST colon, and require everything after it to be digits, optionally
            // separated by further colons. `path:N`, `path:N:M` and the elided `:N` are then one shape.
            //
            // Taking the LAST colon let a `path:line:column` escape: the left side became `path:line`,
            // which is neither empty nor a tracked path, so nothing matched — and that is the spelling
            // every rustc and clippy diagnostic prints, which makes it the form most likely to be pasted
            // into a document.
            let Some((left, right)) = span.split_once(':') else {
                continue;
            };
            let positional = !right.is_empty()
                && right
                    .split(':')
                    .all(|part| !part.is_empty() && part.chars().all(|c| c.is_ascii_digit()));
            if !positional {
                continue;
            }
            // The left side must be a tracked path, OR empty — the elided form, which cites the file
            // named just before it. Requiring a path missed exactly that shape: of the two live
            // coordinates, the second wrote only the colon and the number and escaped the first draft
            // of this direction. An elided reference is not a weaker coordinate, it is a coordinate
            // whose reader has to carry the file in their head as well as the position.
            if left.is_empty() || known.contains(left) {
                coordinates.push(format!("{path}:{index}: `{span}`"));
            }
        }
    }
    assert!(
        read > 0,
        "no tracked file was read, so this direction would report clean having examined none"
    );
    assert!(
        coordinates.is_empty(),
        "a reference names a position rather than a thing:\n{}\nA line number is valid while naming nothing \
         anyone meant, and it rots on any edit above it. Name the item — the entry, the requirement, the \
         function — which costs a clause and cannot go stale.",
        coordinates.join("\n")
    );
}

/// The documents that are **records** rather than live text, by path.
///
/// `AGENTS.md` names them: a commit message, a dated changelog section, `docs/history/`. A record is a
/// measurement of its moment, so a citation inside one is provenance of a decision that was made and stays
/// readable as what it was. Live text is read later against a tree it must be able to address, and that is
/// the difference this list draws.
///
/// A commit message is not a tracked file and needs no row. The two that are files are here, and nothing
/// else: adding a document to escape a refusal is the move this list exists to make visible.
const RECORD_DOCUMENTS: [&str; 2] = ["CHANGELOG.md", "docs/history/"];

/// Every citation the live prose of `corpus` carries that names a moment no reader can reach, in
/// `corpus_root`.
///
/// **What makes this decidable where the sibling prose sweeps were not.** A relative anchor needs a reader to
/// tell a pointer from a span, which is a judgement about the sentence. This one asks nothing about meaning:
/// a token is shaped like an abbreviated commit object or it is not, and a `#` is followed by a digit or it
/// is not. That is why it is a reaction where the neighbouring rule stays a rule.
///
/// **Both a letter and a digit are required, and the residue is declared rather than closed.** A run of hex
/// characters alone over-reaches in both directions this tree actually holds: `repository-checks` writes a
/// nineteen-digit run as the figure a fabricating reader produced, and English carries all-hex words at this
/// length. Requiring one of each admits neither. It also misses an abbreviation that happens to be all
/// digits or all letters — measured at 3.8% of uniformly random seven-character abbreviations — and that
/// miss is taken deliberately, because this repository's Core Contract forbids a false refusal more strictly
/// than it forbids a miss, and the bound is declared where a reader meets it.
///
/// Fenced blocks and HTML comment spans are excluded by reading through [`kanhe::region`]'s prose reader
/// rather than raw lines: a fence is where a command lives, and a command may legitimately carry a hash.
fn unanchored_citation_offences_in(corpus_root: &Path, corpus: &[String]) -> BTreeSet<String> {
    let mut offences = BTreeSet::new();
    for path in corpus.iter() {
        if !matches!(prose_of(path), Some(Prose::Whole)) {
            continue;
        }
        if RECORD_DOCUMENTS
            .iter()
            .any(|record| path == record || path.starts_with(record))
        {
            continue;
        }
        let text = std::fs::read_to_string(corpus_root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        // **Two sanctioned readers, and neither is re-implemented here.** `region`'s prose reader decides
        // what is prose — fenced blocks and HTML comment spans are not — and `reading`'s pairing reader
        // decides where a code span opens and closes. Pairing backticks here would be the shape
        // `no_source_outside_the_shared_reader_pairs_backticks_by_hand` refuses, and it refused this file
        // when the first draft did exactly that.
        //
        // A dropped line is rebuilt as an empty one rather than removed, so line numbers stay the
        // document's own and a fence becomes a paragraph break — which it already is, since a code span
        // cannot cross a blank line.
        let source = kanhe::region::Source::of(text.as_str());
        let visible: std::collections::BTreeMap<usize, String> =
            source.prose().numbered_lines().collect();
        let prose: String = (1..=text.lines().count())
            .map(|line| visible.get(&line).map(String::as_str).unwrap_or(""))
            .collect::<Vec<_>>()
            .join("\n");

        for (line, span) in kanhe::reading::backticked_by_paragraph(&prose) {
            if !is_abbreviated_object(&span) {
                continue;
            }
            offences.insert(format!(
                "  {path}:{line} cites the commit object `{span}`, and live text anchors to a release. \
                 `main` carries one commit per release, so a development commit is unreachable from a fresh \
                 clone by construction; a release commit is reachable and is still named better by its \
                 version. Name the release window, or move the citation into a record"
            ));
        }
        for (line, text) in &visible {
            for serial in hosting_serials(text) {
                offences.insert(format!(
                    "  {path}:{line} cites `{serial}`, a serial belonging to the hosting platform rather \
                     than to this repository. Name what the change was, or the release window it landed in"
                ));
            }
        }
    }
    offences
}

/// Whether a code span's content is an abbreviated commit object: 7 to 40 lowercase hex characters
/// carrying **both** a letter and a digit.
fn is_abbreviated_object(span: &str) -> bool {
    span.len() >= 7
        && span.len() <= 40
        && span
            .chars()
            .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        && span.chars().any(|c| c.is_ascii_digit())
        && span.chars().any(|c| c.is_ascii_alphabetic())
}

/// Every `#` immediately followed by a digit, which is how this hosting platform spells a serial.
fn hosting_serials(line: &str) -> Vec<String> {
    let bytes: Vec<char> = line.chars().collect();
    let mut found = Vec::new();
    for (at, ch) in bytes.iter().enumerate() {
        if *ch != '#' {
            continue;
        }
        // A bound identifier spells `capability#slug`, so a `#` glued to a word is not this shape; only a
        // digit immediately after it is.
        let before_is_word = at
            .checked_sub(1)
            .and_then(|prev| bytes.get(prev))
            .is_some_and(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_' || *c == '/');
        if before_is_word {
            continue;
        }
        let digits: String = bytes[at + 1..]
            .iter()
            .take_while(|c| c.is_ascii_digit())
            .collect();
        if !digits.is_empty() {
            found.push(format!("#{digits}"));
        }
    }
    found
}

/// No live governance document cites a moment a reader of a fresh clone cannot reach.
///
/// **The class five review rounds swept past.** Those rounds looked for typed counts, live line counts,
/// relative anchors and word-form figures — every drift class except the one where the *citation itself* is
/// the moving part. A commit hash looks like the most precise anchor available, which is exactly why nobody
/// examined it. Measured when this landed: of the commit objects cited by live governance text, all but one
/// were already unreachable from `origin/main`, because `main` carries one commit per release and a whole
/// development window squashes into it.
///
/// So this is not a repair waiting on the next squash; the citations were dead when they were written.
#[test]
fn no_live_document_cites_a_moment_a_fresh_clone_cannot_reach() {
    let Some(root) = workspace_root() else {
        return;
    };
    let offences = unanchored_citation_offences_in(&root, &tracked(&root));
    assert!(
        offences.is_empty(),
        "{} unreachable citation(s) in live governance text:\n{}",
        offences.len(),
        offences.iter().cloned().collect::<Vec<_>>().join("\n")
    );
}
