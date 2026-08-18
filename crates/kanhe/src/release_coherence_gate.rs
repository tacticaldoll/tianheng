//! The release-coherence judgement, and one builder for the repository shapes it judges.
//!
//! Shared by the gate (`release_coherence.rs`, which runs it over this repository and over the fixtures of its
//! failure matrix) and by the pins citing this capability's declared bounds. Two constructions of "a
//! repository with a changelog and some machinery" is the twin-drift class this repository keeps closing.
//!
//! It separates a **violation** — the release surfaces disagree — from a **cannot-judge** — an input it could
//! not read. A shallow clone with no release spine, an absent manifest, a layout that moved: none of those say
//! the surfaces disagree, and reporting them as if they did tells a reader to go looking for a disagreement
//! that does not exist.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::refusal::{Refusal, cannot_judge, violation};

pub use crate::hermetic_git::hermetic;
use crate::manifest::{Quoted, WorkspaceVersion, quoted_value, semver, workspace_version};

fn git(repo: &Path, args: &[&str]) -> Result<String, crate::hermetic_git::Failure> {
    crate::hermetic_git::run(repo, &[], args)
}

fn read(repo: &Path, rel: &str) -> Result<String, Refusal> {
    std::fs::read_to_string(repo.join(rel))
        .map_err(|err| cannot_judge(format!("could not read {rel}: {err}")))
}

/// Every value assigned to `key` inside a dependency's value text, recognised as a **table key** rather than
/// as a substring, in the order written.
///
/// **The candidates are a value first, so the caller answers *how many*.** `split("version").nth(1)` read the
/// first occurrence of the bare word on the whole line — the dependency's own name and its path included — so
/// `version-utils = { path = "crates/version-utils", version = "0.5.0" }` answered about the wrong span and
/// produced *has no version pin* in front of the release gate. That is the lossy-selection class
/// [`crate::selection`] exists for, in the file that predates it.
///
/// A key stands alone: what precedes it is a table delimiter or whitespace, and what follows is `=` after
/// optional space. Both halves are required — the first alone still admits `/version`, the second alone still
/// admits a key ending in `version`.
pub(crate) fn inline_assignments(value: &str, key: &str) -> Vec<Quoted> {
    let mut found = Vec::new();
    let bytes = value.as_bytes();
    let mut at = 0;
    while let Some(offset) = value[at..].find(key) {
        let start = at + offset;
        let after = start + key.len();
        let opens = start == 0 || matches!(bytes[start - 1], b'{' | b',' | b' ' | b'\t');
        let rest = value[after..].trim_start();
        if opens {
            if let Some(assignment) = rest.strip_prefix('=') {
                found.push(quoted_value(assignment));
            }
        }
        at = after;
    }
    found
}

/// Which dependency table a heading opens, if any.
///
/// **The reader used to look at no heading at all**, which cost it both directions at once. A
/// `[dependencies.alias]` table declares one dependency across its own lines, and none of those lines is a
/// `<family-crate> = …` entry, so the whole declaration — renamed or not — was invisible. And a `[features]`
/// key spelled after a family crate was read as a version requirement, because nothing said which tables hold
/// dependencies.
///
/// `[target.'cfg(…)'.dependencies]` is **not** admitted, and that is a declared bound rather than an
/// oversight: its heading grammar carries a quoted cfg expression, which is where a line-oriented reader is
/// likeliest to be wrong again, and no example manifest has ever carried one.
enum Table {
    /// `[dependencies]` and its dev/build siblings: each line names one dependency.
    Entries,
    /// `[dependencies.NAME]`: the whole table is one dependency, named by its heading.
    One(String),
    /// Any other table. Not a source of dependencies, so nothing in it is read as one.
    Other,
}

fn dependency_table(heading: &str) -> Table {
    let Some(inner) = heading.strip_prefix('[').and_then(|h| h.strip_suffix(']')) else {
        return Table::Other;
    };
    for kind in ["dependencies", "dev-dependencies", "build-dependencies"] {
        if inner == kind {
            return Table::Entries;
        }
        if let Some(named) = inner.strip_prefix(kind).and_then(|r| r.strip_prefix('.')) {
            if !named.is_empty() {
                return Table::One(named.to_string());
            }
        }
    }
    Table::Other
}

/// What a dependency declares as its version requirement, or why this reader could not tell.
///
/// **Four states, and every consumer answers all four.** Three call sites read a dependency's pin and each
/// decided the refusal class for itself: two matched exhaustively and the third collapsed to `_ => None`,
/// which reported an *absent* key as one this reader *could not read* — the very distinction its sibling had
/// just been repaired to make. A typed result makes the compiler ask each consumer when a state is added,
/// which is the shape [`PackageName`] and [`crate::manifest::WorkspaceVersion`] already carry in this family.
///
/// [`crate::selection::the_only`] is deliberately not used here, for the reason `manifest.rs` records for its
/// own reader: it reports none and several as one refusal, and here they are different facts — an absent pin
/// is the legal `{ path = "…" }` form, and two are a table this reader may not choose from.
#[derive(Debug, PartialEq, Eq)]
enum Pin {
    /// The version requirement as written.
    Declared(String),
    /// No `version` key. Legal: a path-only or git-only dependency declares no version.
    Absent,
    /// A `version` this reader cannot read — a value not in double quotes — quoted as written.
    Unreadable(String),
    /// More than one `version` key in one dependency. Malformed, and not this reader's to choose from.
    Several(usize),
}

impl Pin {
    fn of(mut values: Vec<Quoted>, written: &str) -> Self {
        match values.len() {
            0 => Pin::Absent,
            1 => match values.pop() {
                Some(Quoted::Value(version)) => Pin::Declared(version),
                _ => Pin::Unreadable(written.trim().to_string()),
            },
            several => Pin::Several(several),
        }
    }
}

/// One dependency a manifest declares: the key it is written under, the package it names, and its pin.
struct Dependency {
    key: String,
    package: String,
    pin: Pin,
}

/// Every dependency `text` declares, in both forms cargo admits.
///
/// The inline form (`alias = { package = "xuanji", version = "0.5" }`, or a bare `xuanji = "0.5"`) and the
/// detailed table (`[dependencies.alias]` with its own `package` and `version` lines) are one grammar to a
/// reader that tracks the heading, and [`inline_assignments`] recognises a key the same way in both: at a
/// line's start, or after a table delimiter inside a value.
fn declared_dependencies(text: &str) -> Vec<Dependency> {
    let mut found = Vec::new();
    let mut table = Table::Other;
    // A detailed table is one dependency spread over its lines, so it is emitted at the boundary that closes
    // it — the next heading, or the end of the text.
    let mut pending: Option<(String, Vec<Quoted>, Vec<Quoted>, String)> = None;
    let flush = |pending: &mut Option<(String, Vec<Quoted>, Vec<Quoted>, String)>,
                 found: &mut Vec<Dependency>| {
        if let Some((key, packages, versions, written)) = pending.take() {
            let package = match packages.len() {
                1 => match &packages[0] {
                    Quoted::Value(name) => name.clone(),
                    Quoted::Unreadable => String::new(),
                },
                // No `package` key names the crate by its own heading; several is malformed and names no
                // one crate, so it enters as unnameable rather than as the heading's own name.
                0 => key.clone(),
                _ => String::new(),
            };
            found.push(Dependency {
                key,
                package,
                pin: Pin::of(versions, &written),
            });
        }
    };
    for line in crate::region::Source::of(text).toml().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            flush(&mut pending, &mut found);
            table = dependency_table(trimmed);
            if let Table::One(name) = &table {
                pending = Some((name.clone(), Vec::new(), Vec::new(), String::new()));
            }
            continue;
        }
        match &table {
            Table::Entries => {
                let Some((key, rest)) = trimmed.split_once('=') else {
                    continue;
                };
                let key = key.trim();
                let packages = inline_assignments(rest, "package");
                let package = match packages.len() {
                    1 => match &packages[0] {
                        Quoted::Value(name) => name.clone(),
                        Quoted::Unreadable => String::new(),
                    },
                    0 => key.to_string(),
                    _ => String::new(),
                };
                // A bare `xuanji = "0.5"` carries its requirement as the value itself; an inline table
                // carries it under a `version` key.
                let versions = if rest.trim_start().starts_with('{') {
                    inline_assignments(rest, "version")
                } else {
                    vec![quoted_value(rest)]
                };
                found.push(Dependency {
                    key: key.to_string(),
                    package,
                    pin: Pin::of(versions, rest),
                });
            }
            Table::One(_) => {
                if let Some((_, packages, versions, written)) = pending.as_mut() {
                    packages.extend(inline_assignments(trimmed, "package"));
                    versions.extend(inline_assignments(trimmed, "version"));
                    if !trimmed.is_empty() {
                        written.push_str(trimmed);
                        written.push(' ');
                    }
                }
            }
            Table::Other => {}
        }
    }
    flush(&mut pending, &mut found);
    found
}

/// Whether `suffix` is an ISO date: three `-`-separated all-digit fields of widths 4, 2 and 2.
///
/// **Parsed, not counted.** The test this replaces asserted the heading was ten characters longer than its
/// own prefix and never read them, so `## [0.5.0] - notadate!!` satisfied *CHANGELOG carries dated release
/// notes*. A length test is a parse without its guarantee.
///
/// **And the fields are ranged, because a digit test is that same shortfall one level in.** Reading only
/// three all-digit fields of the right widths admitted `2026-99-99` and `0000-00-00` — shapes that pass for
/// a date while naming none. The ranges are the calendar's outer bounds and no more: a month is `1..=12`, a
/// day `1..=31`. Whether that day exists in that month needs a calendar, which is a dependency this crate's
/// declared surface does not carry, and the residue — `2026-02-31` — is a date a human wrote wrong rather
/// than a shape that reads as one.
pub(crate) fn is_iso_date(suffix: &str) -> bool {
    let parts: Vec<&str> = suffix.split('-').collect();
    let [year, month, day] = parts.as_slice() else {
        return false;
    };
    let field = |part: &str, width: usize| {
        (part.len() == width && part.chars().all(|c| c.is_ascii_digit()))
            .then(|| part.parse::<u32>().ok())
            .flatten()
    };
    let (Some(_), Some(month), Some(day)) = (field(year, 4), field(month, 2), field(day, 2)) else {
        return false;
    };
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// A TOML line with its whitespace removed, for a predicate that compares a whole line against one spelling.
///
/// **TOML's `wschar` is `%x20` and `%x09`, and this removes exactly those.** The predicate it serves used to
/// be `line.trim()` then `replace(' ', "")`, and routing the reader through `region::toml()` dropped the
/// `trim()` — leaving a rule that removed spaces and not tabs. `\tversion.workspace = true` stopped matching
/// and its member was refused: a false refusal in front of the release gate over a legal manifest, of the
/// same class and direction as the defect that repair had just closed.
///
/// Restoring the `trim()` would have fixed the indent and left the tab *before a comment*, which the region
/// correctly leaves in the head. Asking the question the predicate means — this line with its whitespace
/// gone — cannot come apart that way again.
///
/// `split_whitespace()` would also do it and is not used: it removes every Unicode whitespace character,
/// which is wider than the grammar and would accept a line TOML rejects. Reading the language's own rule
/// instead of a wider borrowed one is the whole subject of the repair this regressed out of.
fn without_wschar(line: &str) -> String {
    line.chars().filter(|c| !matches!(c, ' ' | '\t')).collect()
}

/// What a member manifest says its package is called, or why this reader could not tell.
///
/// Three states rather than an `Option`, because every consumer here treated `None` as *not a package* and
/// skipped it — so a manifest this reader could not parse left its package's lock version unchecked and its
/// examples' pins unexamined, with the function still returning `Ok`. The template is
/// `capability_subjects::Declared`, applied to a second reader.
enum PackageName {
    /// The `[package]` table's `name`.
    Named(String),
    /// No `[package]` table, or no `name` key inside it.
    Absent,
    /// A `name` this reader cannot read: a value not in double quotes, or more than one key in `[package]`.
    Unreadable(String),
}

/// The `[package]` table's `name`, and only that table's.
///
/// **Scoped to the table, not to the first match in the file.** The previous read took the first line whose
/// trimmed start was `name` anywhere in the manifest, which is correct only while `[package]` precedes every
/// other name-bearing table — a premise TOML does not impose and nothing here stated.
/// `crates/tianheng/Cargo.toml` already carries three `name` keys (`[package]`, `[lib]`, `[[bin]]`), so the
/// multiplicity is present in this tree and the read was right by the order they happen to appear in and by
/// the three values happening to agree.
///
/// `the_only` is deliberately **not** used, though this is a class-A shape: it reports none and several as one
/// refusal, and here they are different facts — no `[package]` table means this is not a package manifest,
/// while two `name` keys in one means it is malformed. The consumer needs to tell them apart, so the
/// three-state return carries the distinction instead.
fn package_name(manifest: &str) -> PackageName {
    // Executed manifest text. Raw lines were safe against a commented-out `name` only by accident — a
    // `#`-led line fails `strip_prefix("name")` — and not safe at all against a comment on the **table
    // heading**: `[package] # the repository checks` fails `trimmed == "[package]"`, so the table never
    // opens, no `name` is found, and `require_example_pins` answers `cannot_judge` over a legal manifest.
    // Held by `a_package_heading_with_a_trailing_comment_still_opens_the_table`, run against raw lines.
    //
    // A first version of this comment claimed the benefit was at the `name` **value** —
    // `name = "kanhe" # …` supposedly reaching `quoted_value` as `Unreadable`. It never did:
    // `quoted_value` takes the text between the first pair of quotes and discards what follows. The claim
    // was refuted by a reviewer, and stating a benefit a reader could have checked against the function ten
    // lines up is the cheaper half of the discipline the previous commit wrote down.
    let source = crate::region::Source::of(manifest);
    let mut in_package = false;
    let mut names: Vec<&str> = Vec::new();
    for line in source.toml().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // `[package]` exactly. `[package.metadata.docs.rs]` is a different table and names no package.
            in_package = trimmed == "[package]";
            continue;
        }
        if !in_package {
            continue;
        }
        // `name` then `=`, so `name_of` and any other `name…` key is not this key.
        let Some(rest) = trimmed.strip_prefix("name") else {
            continue;
        };
        let Some(value) = rest.trim_start().strip_prefix('=') else {
            continue;
        };
        names.push(value.trim());
    }
    match names.len() {
        0 => PackageName::Absent,
        1 => match quoted_value(names[0]) {
            Quoted::Value(name) => PackageName::Named(name),
            Quoted::Unreadable => PackageName::Unreadable(names[0].to_string()),
        },
        several => PackageName::Unreadable(format!("{several} `name` keys in `[package]`")),
    }
}

/// Which phase of the release ritual this repository is in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Between releases: an adopter-facing `[Unreleased]` entry is required, and lockfile drift is
    /// tolerated as history.
    Development,
    /// The workspace version has moved forward for release preparation, so the dated section, the internal
    /// pins and every workspace entry in `Cargo.lock` must all name it.
    ReleaseReady,
    /// The `release: X.Y.Z` commit itself, held to the same alignment as `ReleaseReady`.
    Snapshot,
}

impl State {
    fn label(self) -> &'static str {
        match self {
            State::Development => "development",
            State::ReleaseReady => "release-ready",
            State::Snapshot => "snapshot",
        }
    }
}

const COMPARE: &str = "https://github.com/tacticaldoll/tianheng/compare";
const RELEASES: &str = "https://github.com/tacticaldoll/tianheng/releases/tag";

/// The release spine, and which phase of the ritual the workspace is in relative to it.
struct Spine {
    /// Which phase the workspace is in, relative to the latest release commit.
    state: State,
    /// The latest `release: X.Y.Z` subject's version.
    release_version: String,
    /// The one before it, absent when the latest is the first release.
    previous_release: Option<String>,
}

/// Read the release spine out of the commit log and classify the workspace against it.
///
/// A malformed `release:` subject is a **violation** — the history disagrees with its own form — while an
/// absent spine is a **cannot-judge**, because a shallow clone cannot see one and that is not a disagreement.
fn release_spine(
    repo: &Path,
    version: &str,
    version_parts: (u64, u64, u64),
) -> Result<Spine, Refusal> {
    let subjects = git(repo, &["log", "--format=%H%x09%s"])
        .map_err(|err| cannot_judge(format!("could not read the release history: {err}")))?;
    let mut history: Vec<(String, String)> = Vec::new();
    // HEAD's own commit is the first line this log produced, so asking git for it again would be a second
    // read of something already in hand — and a refusal guarding that second read is a branch no input can
    // take. Taken here instead.
    let mut head: Option<String> = None;
    for line in subjects.lines() {
        let Some((commit, subject)) = line.split_once('\t') else {
            continue;
        };
        if head.is_none() {
            head = Some(commit.to_string());
        }
        if let Some(rest) = subject.strip_prefix("release: ") {
            if semver(rest).is_none() {
                return Err(violation(format!(
                    "malformed release history subject: {subject}"
                )));
            }
            history.push((commit.to_string(), rest.to_string()));
        } else if subject.starts_with("release:") {
            return Err(violation(format!(
                "malformed release history subject: {subject}"
            )));
        }
    }
    let Some((release_commit, release_version)) = history.first().cloned() else {
        return Err(cannot_judge(
            "exact release history is unavailable; fetch full history containing release: X.Y.Z — a shallow \
             clone cannot see the release spine, which is not the same as surfaces that disagree",
        ));
    };
    let previous_release = history.get(1).map(|(_, v)| v.clone());
    // A release commit exists, so at least one line of the log parsed, so this is Some. Provable from the
    // loop above rather than assumed about git.
    let head =
        head.expect("the log line that produced a release commit also produced HEAD's own commit");

    let state = if head == release_commit {
        if version != release_version {
            return Err(violation(format!(
                "release snapshot subject is {release_version} but workspace version is {version}"
            )));
        }
        State::Snapshot
    } else {
        let released =
            semver(&release_version).expect("the history holds only well-formed versions");
        match version_parts.cmp(&released) {
            std::cmp::Ordering::Less => {
                return Err(violation(format!(
                    "workspace version {version} is older than latest release {release_version}"
                )));
            }
            std::cmp::Ordering::Equal => State::Development,
            std::cmp::Ordering::Greater => State::ReleaseReady,
        }
    };
    Ok(Spine {
        state,
        release_version,
        previous_release,
    })
}

/// Every version-bearing surface outside the changelog, and the member manifests the later phases read.
fn require_version_surfaces(
    repo: &Path,
    root_manifest: &str,
    version: &str,
) -> Result<Vec<(String, String)>, Refusal> {
    let manifests = workspace_manifests(repo)?;
    for (path, text) in &manifests {
        // Only the refusal message needs the name here — the inheritance read below works off the text
        // whichever state this is, so an unnameable package is reported by path rather than skipped. The
        // third consumer, and the only one for which that is the right answer.
        let name = match package_name(text) {
            PackageName::Named(name) => name,
            PackageName::Absent | PackageName::Unreadable(_) => path.clone(),
        };
        // This reader held its own `split('#')` — the last hand-rolled cut over TOML text outside `region`.
        // Measured, because an earlier wording called it "a fourth spelling of one language's rule": four
        // `split('#')`-shaped sites existed, but the other three read a Markdown heading, a shell command
        // and a URL fragment, so they are not this rule and never were.
        //
        // It was kept out of `region` while `toml()` cut at a token
        // start, because converting it then would have refused `version.workspace = true#c`, which is a
        // legal comment on a line that still inherits. `toml()` now tracks strings and cuts where TOML cuts,
        // so the exception has nothing left to protect and the hand-rolled rule is gone with it.
        //
        // Both directions run through `judge`: `an_inherit_line_with_a_glued_comment_still_inherits` and
        // `a_member_whose_only_inherit_line_is_commented_out_is_refused`.
        let inherits = crate::region::Source::of(text.as_str())
            .toml()
            .lines()
            .any(|line| without_wschar(line) == "version.workspace=true");
        if !inherits {
            return Err(violation(format!(
                "workspace package {name} must inherit version.workspace = true"
            )));
        }
    }
    require_internal_pins(root_manifest, version)?;
    require_example_pins(repo, &manifests, version)?;
    Ok(manifests)
}

/// The changelog surfaces whose required shape depends on which phase of the ritual this is.
fn require_changelog_state(
    repo: &Path,
    changelog: &str,
    manifests: &[(String, String)],
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    let unreleased_sections = changelog
        .lines()
        .filter(|line| line.trim_end() == "## [Unreleased]")
        .count();
    if unreleased_sections != 1 {
        return Err(violation(
            "CHANGELOG must contain exactly one [Unreleased] section".to_string(),
        ));
    }
    let has_item = unreleased_has_item(changelog);
    match spine.state {
        State::Development => {
            if !has_item {
                return Err(violation(
                    "development requires adopter-facing release narrative under [Unreleased]"
                        .to_string(),
                ));
            }
            let link = format!("[Unreleased]: {COMPARE}/v{version}...HEAD");
            if !changelog.lines().any(|line| line.trim_end() == link) {
                return Err(violation(format!(
                    "[Unreleased] comparison link must start at v{version} and end at HEAD"
                )));
            }
        }
        State::ReleaseReady | State::Snapshot => {
            if has_item {
                return Err(violation(format!(
                    "[Unreleased] must be empty in {} state",
                    spine.state.label()
                )));
            }
            let prefix = format!("## [{version}] - ");
            let dated = changelog.lines().any(|line| {
                line.trim_end()
                    .strip_prefix(&prefix)
                    .is_some_and(is_iso_date)
            });
            if !dated {
                return Err(violation(format!(
                    "CHANGELOG is missing dated release notes for {version}"
                )));
            }
            let from = if spine.state == State::ReleaseReady {
                Some(spine.release_version.clone())
            } else {
                spine.previous_release.clone()
            };
            let expected = match &from {
                Some(previous) => format!("[{version}]: {COMPARE}/v{previous}...v{version}"),
                None => format!("[{version}]: {RELEASES}/v{version}"),
            };
            if !changelog.lines().any(|line| line.trim_end() == expected) {
                return Err(violation(match &from {
                    Some(previous) => {
                        format!("CHANGELOG comparison link for {version} must start at v{previous}")
                    }
                    None => format!("first release CHANGELOG link must target v{version}"),
                }));
            }
            require_lock_versions(repo, manifests, version)?;
        }
    }
    Ok(())
}

/// Each release section's internal consistency: no repeated heading, and every `**BREAKING**` paired with a
/// `### Migration`.
///
/// The vacuity guard this walk once carried is UNREACHABLE and is gone. `## [Unreleased]` is itself a
/// `## [` section, and the exactly-one-`[Unreleased]` check in the caller already refuses a changelog with
/// none — more specifically, and as a violation rather than an undecidable. A guard whose input an earlier
/// check forecloses cannot fire, and keeping it would read as coverage. Found by trying to write its WHEN.
fn require_section_shape(changelog: &str) -> Result<(), Refusal> {
    let shape = section_shape(changelog);
    let mut duplicates: Vec<String> = shape
        .headings
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|((section, heading), _)| format!("  {section} repeats `### {heading}`"))
        .collect();
    duplicates.sort();
    if !duplicates.is_empty() {
        return Err(violation(format!(
            "a CHANGELOG release section repeats a heading, so entries that belong together are split:\n{}",
            duplicates.join("\n")
        )));
    }
    let mut missing: Vec<&String> = shape
        .breaking
        .iter()
        .filter(|section| {
            !shape
                .headings
                .keys()
                .any(|(s, h)| *s == **section && h == "Migration")
        })
        .collect();
    missing.sort();
    if !missing.is_empty() {
        return Err(violation(format!(
            "a CHANGELOG section marks a change **BREAKING** and carries no `### Migration` section, so what \
             an adopter must do is scattered through the entries or absent:\n{}",
            missing
                .iter()
                .map(|s| format!("  {s}"))
                .collect::<Vec<_>>()
                .join("\n")
        )));
    }
    Ok(())
}

/// The adopter-facing narrative names none of this repository's own machinery.
fn require_adopter_narrative(
    repo: &Path,
    changelog: &str,
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    let leaked = adopter_cited_machinery(repo, changelog, version, spine.state)?;
    if !leaked.is_empty() {
        return Err(violation(format!(
            "an adopter-facing CHANGELOG entry names this repository's own machinery, which ships in no \
             package and which an adopter can never run — move it under `### Self-governance`, or, where the \
             adopter-relevant fact is genuinely there, state the guarantee and drop the filename:\n{}",
            leaked.join("\n")
        )));
    }
    Ok(())
}

/// Judge a repository's release state, returning what to report or why it cannot be judged.
///
/// Read-only: it never bumps, commits, tags, or publishes.
pub fn judge(repo: &Path) -> Result<String, Refusal> {
    if !repo.join("Cargo.toml").is_file() {
        return Err(cannot_judge(format!(
            "repository root {} has no Cargo.toml",
            repo.display()
        )));
    }
    if !repo.join("CHANGELOG.md").is_file() {
        return Err(cannot_judge(format!(
            "repository root {} has no CHANGELOG.md",
            repo.display()
        )));
    }
    // The cause travels, for the reason its sibling in `publish_source_gate` records: a machine without git
    // was told the repository has no history.
    git(repo, &["rev-parse", "--is-inside-work-tree"]).map_err(|err| {
        cannot_judge(match err {
            crate::hermetic_git::Failure::Spawn(why) => format!(
                "git could not be run at all ({why}), so whether {} has a history was never asked",
                repo.display()
            ),
            crate::hermetic_git::Failure::Exit(stderr) => format!(
                "repository root {} has no git history: {stderr}",
                repo.display()
            ),
        })
    })?;

    let root_manifest = read(repo, "Cargo.toml")?;
    // The three states are answered separately, and the middle one is why the reader has three. A value this
    // reader cannot read is not a key that is absent, and it is not a malformed version either: it is legal
    // TOML in a form this reader does not take, and telling an operator their version is *missing* sends them
    // to look for a key that is sitting in front of them.
    let version = match workspace_version(&root_manifest) {
        WorkspaceVersion::Declared(version) => version,
        WorkspaceVersion::Absent => {
            return Err(cannot_judge(
                "workspace version is missing or malformed: <missing>",
            ));
        }
        WorkspaceVersion::Unreadable(what) => {
            return Err(cannot_judge(format!(
                "Cargo.toml declares a workspace version this check cannot read ({what}), so whether every \
                 release surface names one version cannot be decided"
            )));
        }
    };
    let Some(version_parts) = semver(&version) else {
        return Err(cannot_judge(format!(
            "workspace version is missing or malformed: {version}"
        )));
    };
    let changelog = read(repo, "CHANGELOG.md")?;

    // The phases, in the order a reader meets a refusal in. **The order is observable**: a repository with
    // two problems is refused for whichever phase reaches its own first, and the failure matrix asserts the
    // message. So these are a sequence rather than a set, and moving one moves what gets reported.
    let spine = release_spine(repo, &version, version_parts)?;
    let manifests = require_version_surfaces(repo, &root_manifest, &version)?;
    require_changelog_state(repo, &changelog, &manifests, &version, &spine)?;
    require_section_shape(&changelog)?;
    require_adopter_narrative(repo, &changelog, &version, &spine)?;

    Ok(format!(
        "ok release coherence ({}: {version})",
        spine.state.label()
    ))
}

/// The entries of a directory, with a failure to yield one **propagated** rather than dropped.
///
/// `filter_map(|e| e.ok())` silently shortens the enumeration, and the counters this judgement then reasons
/// from are satisfied by whatever did yield — so a run reports clean over the entry it never saw. One site
/// serves both enumerations, because two calls carrying one message would be shadowing.
fn entries_of(dir: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let listing = std::fs::read_dir(dir).map_err(|err| {
        cannot_judge(format!(
            "found no enumerable directory at {}: {err} — the layout changed or is absent, so what it holds \
             cannot be judged",
            dir.display()
        ))
    })?;
    let mut paths = Vec::new();
    for entry in listing {
        let entry = entry.map_err(|err| {
            cannot_judge(format!(
                "an entry of {} could not be read while enumerating it: {err}",
                dir.display()
            ))
        })?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn workspace_manifests(repo: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let crates = repo.join("crates");
    let mut out = Vec::new();
    let dirs = entries_of(&crates)?;
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = std::fs::read_to_string(&manifest)
                .map_err(|err| cannot_judge(format!("could not read {manifest:?}: {err}")))?;
            out.push((
                manifest
                    .strip_prefix(repo)
                    .unwrap_or(&manifest)
                    .display()
                    .to_string(),
                text,
            ));
        }
    }
    if out.is_empty() {
        return Err(cannot_judge(
            "found no workspace crate manifests under crates/ — the crate layout changed or is absent",
        ));
    }
    Ok(out)
}

fn require_internal_pins(root_manifest: &str, version: &str) -> Result<(), Refusal> {
    let mut pins = 0usize;
    // Executed manifest text, not raw lines. A commented-out internal dependency —
    // `# xuanji = { path = "crates/xuanji" }` — satisfies every predicate this filter applies, so it was counted as
    // a pin and then refused for having no version: a **false refusal** in front of the release gate, and
    // one that also inflated the vacuity guard with text declaring nothing. The sibling four hundred lines
    // up already stripped `#` by hand; this module now asks `region` instead, which is the module written
    // so that forgetting was not possible, and which this file imported nowhere.
    for line in crate::region::Source::of(root_manifest).toml().lines() {
        let trimmed = line.trim();
        if !trimmed.contains("path") || !trimmed.contains("\"crates/") || !trimmed.contains('=') {
            continue;
        }
        let Some((name, value)) = trimmed.split_once('=') else {
            continue;
        };
        pins += 1;
        let name = name.trim();
        // Read from the dependency's VALUE and by table key, never from the whole line by substring — see
        // `inline_assignments` for the false refusal that produced.
        let assignments = inline_assignments(value, "version");
        // **Absence is answered before the enumeration is consumed, not from a second one.** `the_only`
        // reports none and several as one refusal, and here they are different facts: none is the missing
        // pin this check exists to name, several is a line this reader may not choose from. Reading the
        // first from a re-run of `inline_assignments` inside the `Err` arm answered it correctly and built
        // the candidate list twice to ask two questions about it — the shape `crate::selection` exists to
        // discourage, in the repair that introduced the module's own call site.
        if assignments.is_empty() {
            return Err(violation(format!(
                "internal dependency {name} has no version pin"
            )));
        }
        let pin = match crate::selection::the_only("`version` key", assignments) {
            Ok(Quoted::Value(pin)) => pin,
            Ok(Quoted::Unreadable) => {
                return Err(cannot_judge(format!(
                    "internal dependency {name} declares a version this check cannot read, so whether it \
                     names the workspace version cannot be decided"
                )));
            }
            Err(refusal) => return Err(refusal),
        };
        if pin != version {
            return Err(violation(format!(
                "internal dependency {name} is pinned to {pin}; expected {version}"
            )));
        }
    }
    if pins == 0 {
        return Err(cannot_judge(
            "found no internal path dependency in Cargo.toml — the declaration form changed, so pin \
             coherence would be reported over nothing",
        ));
    }
    Ok(())
}

fn require_example_pins(
    repo: &Path,
    manifests: &[(String, String)],
    version: &str,
) -> Result<(), Refusal> {
    // A manifest whose package this reader cannot name is not a crate the examples may quietly skip: it would
    // drop out of `family`, and every example pinning it would then pass the `!family.iter().any(…)` filter
    // below without being examined. The two vacuity guards in this function are aggregate, so seven of eight
    // crates parsing keeps them silent while the eighth goes unchecked — which is the partial case a vacuity
    // guard is exactly unable to see.
    let mut family: Vec<String> = Vec::new();
    for (path, text) in manifests {
        match package_name(text) {
            PackageName::Named(name) => family.push(name),
            PackageName::Absent => {
                return Err(cannot_judge(format!(
                    "{path} declares no `[package]` name, so whether an example pins it cannot be decided"
                )));
            }
            PackageName::Unreadable(what) => {
                return Err(cannot_judge(format!(
                    "{path} declares a `[package]` name this check cannot read ({what}), so whether an \
                     example pins it cannot be decided"
                )));
            }
        }
    }
    let minor = version
        .rsplit_once('.')
        .map(|(head, _)| head)
        .unwrap_or(version);
    let mut example_manifests = 0usize;
    let mut requirements = 0usize;

    let dirs = entries_of(&repo.join("examples"))?;
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        // Absent is not unreadable. Skipping both alike let the remaining readable examples satisfy the
        // counters below, so the judgement reported clean over the very manifest it could not read.
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).map_err(|err| {
            cannot_judge(format!(
                "could not read the example manifest {}: {err}",
                manifest.display()
            ))
        })?;
        example_manifests += 1;
        let name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        // Executed text, for the reason `require_internal_pins` records: a commented-out family pin
        // would otherwise be read as a declared one.
        for Dependency { key, package, pin } in declared_dependencies(&text) {
            // **Which crate a dependency names is its `package` field where it has one, and its key only
            // otherwise.** Keying on the name alone was a false negative of the class the Core Contract
            // forbids: cargo renames with `alias = { package = "xuanji", version = "stale" }`, `alias` is in
            // no family, and the entry was skipped entirely — while the aggregate `requirements` counter
            // stayed non-zero on the strength of the other examples. The sibling `require_internal_pins`
            // never had this hole because it keys on the PATH, which a rename cannot move; examples depend
            // by registry version and have no path, so the identity has to be read.
            if package.is_empty() {
                return Err(cannot_judge(format!(
                    "example {name} declares `{key}` with a package identity this check cannot read, so \
                     which crate it requires cannot be decided"
                )));
            }
            if !family.contains(&package) {
                continue;
            }
            // The entry is already known to name a family crate, so every way of failing to read its pin is
            // answered on its own terms. Collapsing them was the defect: an ABSENT `version` — legal, since
            // a path-only dependency declares none — was reported as one this reader could not read.
            let pin = match pin {
                Pin::Declared(pin) => pin,
                Pin::Absent => {
                    return Err(violation(format!(
                        "example {name} requires {package} with no version, so nothing holds it to the \
                         workspace version {version}"
                    )));
                }
                Pin::Unreadable(written) => {
                    return Err(cannot_judge(format!(
                        "example {name} requires {package} with a version this check cannot read \
                         ({written}), so whether it satisfies the workspace version cannot be decided"
                    )));
                }
                Pin::Several(several) => {
                    return Err(cannot_judge(format!(
                        "example {name} declares {several} `version` keys for {package}, so which one it \
                         requires is not this reader's to choose"
                    )));
                }
            };
            requirements += 1;
            if pin != minor && pin != version {
                // The package, and the key where they differ: a renamed dependency reported by its key alone
                // sends a reader looking for a crate the manifest does not name.
                let named = if package == key {
                    package.clone()
                } else {
                    format!("{package} (as `{key}`)")
                };
                return Err(violation(format!(
                    "example {name} requires {named} = \"{pin}\", which the workspace version {version} \
                     does not satisfy"
                )));
            }
        }
    }
    if example_manifests == 0 {
        return Err(cannot_judge(
            "found no example manifests under examples/ — the layout changed or is absent",
        ));
    }
    if requirements == 0 {
        return Err(cannot_judge(format!(
            "read {example_manifests} example manifest(s) and found no family dependency requirement in any \
             of them — the declaration form changed, so example pins would be reported over nothing"
        )));
    }
    Ok(())
}

fn require_lock_versions(
    repo: &Path,
    manifests: &[(String, String)],
    version: &str,
) -> Result<(), Refusal> {
    let lock = read(repo, "Cargo.lock")?;
    // **Every entry under a name, and whether each carries a `source`.** A single-valued map keyed on the
    // name kept the first entry and dropped the rest, which is only right while no name appears twice — and
    // two entries under one name is ordinary in a lock file, either as two versions of one crate or as a
    // workspace member sharing a name with something from a registry. Nothing here stated that premise, and
    // `source` is what tells the two apart: a workspace member has none, everything fetched has one.
    let mut entries: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    let mut name = String::new();
    let mut version_of: Option<String> = None;
    let mut sourced = false;
    // A block ends at the next **table header** or at end of input, so the record is filed on the boundary
    // rather than when its version is read — `source` is written after `version` in cargo's own output, and
    // filing early would record every entry as source-less.
    //
    // **`[[package]]` is not the only table a lock carries, and the boundary is not the only thing that
    // depended on believing it was.** `[[patch.unused]]` — written whenever a `[patch]` section exists — has
    // its own `name`, `version` and `source`, and `[metadata]` closes an older lock. Read as ordinary content
    // they left the block above still open and overwrote its fields, so the last member's version was
    // replaced before it was ever filed and the workspace lookup reported that member missing from a lock
    // that records it. Every table therefore closes the record, and only `[[package]]` reopens one.
    let close = |name: &mut String,
                 version_of: &mut Option<String>,
                 sourced: &mut bool,
                 entries: &mut BTreeMap<String, Vec<(String, bool)>>| {
        if let (false, Some(found)) = (name.is_empty(), version_of.take()) {
            entries
                .entry(name.clone())
                .or_default()
                .push((found, *sourced));
        }
        name.clear();
        *sourced = false;
    };
    // Whether the lines being read belong to a `[[package]]` block. A foreign table's keys are not this
    // package's, and skipping them by name would be a list of the tables someone thought of.
    let mut in_package = false;
    // Executed text here too. A lock file is generated and rarely carries comments, but the reader is the
    // same shape as its two siblings and a corpus narrower than the claim is the defect all three shared.
    for line in crate::region::Source::of(lock.as_str()).toml().lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            close(&mut name, &mut version_of, &mut sourced, &mut entries);
            in_package = trimmed == "[[package]]";
        } else if !in_package {
            continue;
        } else if trimmed.starts_with("source") && trimmed.contains('=') {
            sourced = true;
        } else if trimmed.starts_with("name") && trimmed.contains('=') {
            // An unreadable name defaulted to the empty string, which the `!name.is_empty()` guard below
            // then read as *no package here* — so that entry's version never entered the map and the
            // workspace lookup reported it missing, or found a stale one under the previous name.
            match quoted_value(trimmed) {
                Quoted::Value(value) => name = value,
                Quoted::Unreadable => {
                    return Err(cannot_judge(format!(
                        "Cargo.lock carries a package name this check cannot read ({}), so the versions it \
                         records cannot be compared",
                        trimmed
                    )));
                }
            }
        } else if trimmed.starts_with("version") && trimmed.contains('=') && !name.is_empty() {
            match quoted_value(trimmed) {
                Quoted::Value(value) => {
                    version_of = Some(value);
                }
                Quoted::Unreadable => {
                    return Err(cannot_judge(format!(
                        "Cargo.lock records a version for {name} that this check cannot read ({}), so \
                         whether it matches the workspace cannot be decided",
                        trimmed
                    )));
                }
            }
        }
    }
    close(&mut name, &mut version_of, &mut sourced, &mut entries);

    for (path, text) in manifests {
        // Skipping an unnameable package left its `Cargo.lock` version unchecked while this function still
        // returned `Ok` — and nothing above proves every entry parses, only that the list is non-empty.
        let package = match package_name(text) {
            PackageName::Named(name) => name,
            PackageName::Absent => {
                return Err(cannot_judge(format!(
                    "{path} declares no `[package]` name, so its lock entry cannot be looked up"
                )));
            }
            PackageName::Unreadable(what) => {
                return Err(cannot_judge(format!(
                    "{path} declares a `[package]` name this check cannot read ({what}), so its lock entry \
                     cannot be looked up"
                )));
            }
        };
        // A workspace member is the entry with no `source`. Selecting by name alone would compare against a
        // registry entry that merely shares the name, which reads as a version disagreement that is not one.
        let mut local = entries
            .get(&package)
            .into_iter()
            .flatten()
            .filter(|(_, sourced)| !sourced)
            .map(|(found, _)| found);
        let first = local.next();
        let extra = local.count();
        if extra > 0 {
            return Err(cannot_judge(format!(
                "Cargo.lock carries {} entries for {package} with no source, so which one is the workspace \
                 member is not decided",
                extra + 1
            )));
        }
        match first {
            None => {
                return Err(violation(format!(
                    "Cargo.lock is missing workspace package {package}"
                )));
            }
            Some(found) if found != version => {
                return Err(violation(format!(
                    "Cargo.lock package {package} is {found}; expected {version}"
                )));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

fn unreleased_has_item(changelog: &str) -> bool {
    let mut inside = false;
    for line in changelog.lines() {
        if line.trim_end() == "## [Unreleased]" {
            inside = true;
            continue;
        }
        if inside && line.starts_with("## [") {
            return false;
        }
        if inside {
            let trimmed = line.trim_start();
            if trimmed.starts_with("- ") || trimmed.starts_with("* ") {
                return true;
            }
        }
    }
    false
}

struct Shape {
    headings: BTreeMap<(String, String), usize>,
    breaking: BTreeSet<String>,
}

/// The release section a `## [` heading names, with any ` - DATE` suffix dropped.
///
/// **One derivation.** It was written twice, byte-identical, in `section_shape` and
/// `adopter_cited_machinery` — the shape this file's
/// own header says it exists to close, in the file that says it. A third walk decides section boundaries by
/// a different rule again and is left alone deliberately: `unreleased_has_item` asks *where does
/// `[Unreleased]` end*, which is a boundary question, not a naming one, and folding it in would make one
/// function answer two.
fn section_of(line: &str) -> Option<String> {
    line.starts_with("## [").then(|| {
        line.split(" - ")
            .next()
            .unwrap_or(line)
            .trim_end()
            .to_string()
    })
}

/// The document's grammar — which headings each release section carries, and which sections mark a break.
///
/// It once also collected the section names themselves. Nothing read them: `judge` consumes the headings and
/// the breaking set, so the collection was computed and discarded. `dead_code` cannot see that — `insert` counts
/// as a use of the field — which is why a `-D warnings` workspace passed over it.
///
/// The line between this and an entry's *content* is where the decidable stops: whether an entry is accurate,
/// whether "no adopter action" is true, whether a named symbol exists are judgements over prose, and the
/// detector they would need is the one this repository measured three times and rejected.
fn section_shape(changelog: &str) -> Shape {
    let mut shape = Shape {
        headings: BTreeMap::new(),
        breaking: BTreeSet::new(),
    };
    let mut section = String::new();
    for line in changelog.lines() {
        if let Some(named) = section_of(line) {
            section = named;
            // The `continue` stands on its own: a section heading carries no `### …` and marks no break, so
            // the arms below must not see it.
            continue;
        }
        if section.is_empty() {
            continue;
        }
        if let Some(heading) = line.strip_prefix("### ") {
            *shape
                .headings
                .entry((section.clone(), heading.trim_end().to_string()))
                .or_default() += 1;
        }
        if line.contains("**BREAKING**") {
            shape.breaking.insert(section.clone());
        }
    }
    shape
}

/// Every word that names this repository's own machinery: a tracked path under any package the workspace
/// does not publish, or under `scripts/`, plus the ancestor directories that enumeration derives.
///
/// **The corpus is produced from the manifests, not from a location.** It was `git ls-files scripts/` — which
/// was right when the machinery *was* fourteen shell gates, and stopped being right in the window that
/// deleted them and moved the machinery into `crates/kanhe/**` and `crates/shengmo/**`. `scripts/` now names
/// two wrappers, so the check whose violation message reads *machinery, which ships in no package and which
/// an adopter can never run* had been left pointing at the old address, and the property it holds — an
/// adopter-facing entry naming something an adopter cannot run — went unobserved for everything that moved.
/// `publish = false` is the same criterion the message states, read from the build rather than from a path.
///
/// **A basename enters only when it is unique across the whole tree.** Measured when this widened: the
/// machinery was 78 tracked paths against 182 published ones, and five basenames appeared on both sides —
/// `Cargo.toml`, `README.md`, `bounds.rs`, `lib.rs`, `mod.rs`. Admitting those would refuse an adopter-facing
/// entry for naming a published crate's own source, which is the opposite of this check's purpose. A full
/// path is unambiguous and always enters; a basename is a convenience that has to earn its place, and the
/// same rule governs the ancestor directories the enumeration derives — `crates/` leads to both sides.
fn machinery_names(repo: &Path) -> Result<BTreeSet<String>, Refusal> {
    let metadata = cargo_metadata(repo)?;
    // **The prefix comes from cargo, not from the caller's path.** `manifest_path` is canonical, while the
    // live call site passes `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")`, which renders with its
    // `..` components intact — so stripping `repo.display()` failed for **all eight** members, machinery
    // collapsed to the two `scripts/` files, `published` stayed empty, and two `continue`s made it silent.
    // `workspace_root` is cargo's own answer for the tree it just described, so the two strings cannot
    // disagree about spelling.
    let Some(root) = metadata["workspace_root"].as_str() else {
        return Err(cannot_judge(
            "cargo metadata reported no workspace_root, so no member directory can be resolved",
        ));
    };
    let prefix = format!("{root}/");
    let mut machinery: Vec<String> = Vec::new();
    let mut published: BTreeSet<String> = BTreeSet::new();
    let mut enumerated = 0usize;
    for package in metadata["packages"].as_array().into_iter().flatten() {
        // **The directory comes from the manifest, not from the package name.** Deriving it as
        // `crates/<name>/` was the residual location assumption inside a repair whose own thesis was
        // *produced from the manifests, not from a location*: a member whose directory differs from its
        // package name contributes to neither set, so it is machinery nothing refuses (silent), or published
        // source whose basenames then enter the machinery set and refuse honest adopter prose.
        // `cargo metadata` answers this exactly — `manifest_path` is the member's own `Cargo.toml`.
        let Some(manifest) = package["manifest_path"].as_str() else {
            return Err(cannot_judge(
                "a package in cargo metadata carries no manifest_path, so its directory cannot be resolved",
            ));
        };
        let Some(directory) = manifest
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix("Cargo.toml"))
        else {
            // `--no-deps` lists workspace members only, so every manifest sits under the root cargo reported
            // alongside them. One that does not is this gate's two sources describing different trees, which
            // is a fact to report rather than a member to skip — skipping is what kept the collapse silent.
            return Err(cannot_judge(format!(
                "member manifest {manifest} is not under the workspace root {root} cargo reported for it"
            )));
        };
        let unpublished = package["publish"].as_array().is_some_and(|r| r.is_empty());
        let listing = git(repo, &["ls-files", directory])
            .map_err(|err| cannot_judge(format!("could not enumerate {directory}: {err}")))?;
        for path in listing.lines().filter(|l| !l.is_empty()) {
            enumerated += 1;
            if unpublished {
                machinery.push(path.to_string());
            } else {
                if let Some(base) = path.rsplit('/').next() {
                    published.insert(base.to_string());
                }
                let mut dir = path.to_string();
                while let Some(cut) = dir.rfind('/') {
                    dir.truncate(cut + 1);
                    published.insert(dir.clone());
                    dir.truncate(cut);
                }
            }
        }
    }
    // Members resolved and enumerated nothing means the directories were resolved against a root this
    // repository's git does not share — the same collapse by another route, and `scripts/` alone would still
    // look like an answer.
    if enumerated == 0 {
        return Err(cannot_judge(format!(
            "no tracked file was found for any of the {} workspace members under {root}, so the machinery set \
             would be `scripts/` alone and this check would pass over its own subject",
            metadata["packages"].as_array().map_or(0, Vec::len)
        )));
    }
    let scripts = git(repo, &["ls-files", "scripts/"])
        .map_err(|err| cannot_judge(format!("could not enumerate scripts/: {err}")))?;
    machinery.extend(
        scripts
            .lines()
            .filter(|l| !l.is_empty())
            .map(str::to_string),
    );

    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in &machinery {
        names.insert(path.clone());
        if let Some(base) = path.rsplit('/').next() {
            // Unique across the tree, or it names a published crate's file as well and would refuse an
            // entry that is about the product rather than about the machinery.
            if !published.contains(base) {
                names.insert(base.to_string());
            }
        }
        // The same rule as the basename, for the same reason and found the same way: widening the corpus
        // made `crates/` a machinery name, because it is an ancestor of `crates/kanhe/`. It is equally an
        // ancestor of every published crate, and the live CHANGELOG says `crates/` in adopter-facing prose —
        // so the first run of this widening refused the repository's own changelog. An ancestor enters only
        // where it leads to machinery alone.
        let mut dir = path.clone();
        while let Some(cut) = dir.rfind('/') {
            dir.truncate(cut + 1);
            if !published.contains(&dir) {
                names.insert(dir.clone());
            }
            dir.truncate(cut);
        }
    }
    Ok(names)
}

/// `cargo metadata` for the workspace at `repo`, so the corpus above comes from the build.
fn cargo_metadata(repo: &Path) -> Result<serde_json::Value, Refusal> {
    let out = std::process::Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo)
        .output()
        .map_err(|err| cannot_judge(format!("could not run cargo metadata: {err}")))?;
    if !out.status.success() {
        return Err(cannot_judge(format!(
            "cargo metadata failed for {}: {}",
            repo.display(),
            String::from_utf8_lossy(&out.stderr).trim()
        )));
    }
    serde_json::from_slice(&out.stdout)
        .map_err(|err| cannot_judge(format!("cargo metadata is not JSON: {err}")))
}

/// Every adopter-facing `[Unreleased]` entry naming this repository's own machinery.
///
/// A name is a **word** — a maximal run of path characters, required to equal a tracked path, a tracked
/// basename, or an ancestor directory derived from the enumeration. That is exact matching of a lexical token,
/// not substring matching: the run is delimited by the first character a path cannot hold. An earlier rule
/// compared whole backticked spans and three shapes this document already uses passed clean — a span carrying
/// a command, a padded double-backtick span, and an inline span wrapped across a source line.
///
/// Adopter-facing is the **complement** of `### Self-governance`, so a heading nobody anticipated reacts
/// rather than being exempt by default.
///
/// **A dated section is record only once it is a record.** The exemption's reason is that rewriting a dated
/// section to satisfy a rule written afterwards would falsify it — and that reason does not reach the section
/// this release is *about*. Release preparation dates it and then keeps writing into it: measured on this
/// repository, `chore(release): prepare 0.5.0` dated `## [0.5.0]` and hundreds of lines were added to it
/// afterwards across later commits, none of them examined, because the reader looked only at
/// `## [Unreleased]` — which release-ready state requires to be **empty**. So during preparation the check
/// had no subject at all.
///
/// The state decides it, not a version comparison. In [`State::ReleaseReady`] and [`State::Snapshot`] the
/// section dated for the workspace version is still being written, so it is adopter-facing. In
/// [`State::Development`] the workspace version *equals* the latest released one, so the section carrying it
/// is genuinely record and stays exempt — a rule phrased as *versions strictly below the workspace version
/// stay exempt* would refuse it, which is the reading this comment exists to keep anyone from adopting.
fn adopter_cited_machinery(
    repo: &Path,
    changelog: &str,
    version: &str,
    state: State,
) -> Result<Vec<String>, Refusal> {
    // One enumeration. A second copy lived here for one commit, built for a census that was dropped, and
    // two constructions of one set is the drift this file's own doc-comment says it exists to prevent.
    let names = machinery_names(repo)?;

    let mut found: BTreeSet<String> = BTreeSet::new();
    let mut section = String::new();
    let mut heading = String::new();
    for line in changelog.lines() {
        if let Some(named) = section_of(line) {
            section = named;
            heading.clear();
            continue;
        }
        if section.is_empty() {
            continue;
        }
        if let Some(next) = line.strip_prefix("### ") {
            heading = next.trim_end().to_string();
        }
        let being_written = matches!(state, State::ReleaseReady | State::Snapshot)
            && section == format!("## [{version}]");
        if (section != "## [Unreleased]" && !being_written) || heading == "Self-governance" {
            continue;
        }
        for run in
            line.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')))
        {
            let token = run.strip_prefix("./").unwrap_or(run).trim_end_matches('.');
            if token.is_empty() {
                continue;
            }
            if names.contains(token) {
                found.insert(format!(
                    "  {section} under `### {}` names {token}",
                    if heading.is_empty() {
                        "(no heading)"
                    } else {
                        &heading
                    }
                ));
            }
        }
    }
    Ok(found.into_iter().collect())
}

// --- the fixture ------------------------------------------------------------------------------------------

/// A repository in the shape this judgement reads, built hermetically.
///
/// A fixture that inherits the judged machine cannot demonstrate a refusal, because the shape it builds is not
/// the shape it named — measured on the sibling publish gate, where ambient signing configuration turned an
/// intentionally unsigned tag into a signed one.
pub struct Fixture {
    /// The fixture repository's working tree.
    pub repo: PathBuf,
}

fn run(dir: &Path, args: &[&str]) {
    let out = hermetic(args[0])
        .args(&args[1..])
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {args:?}: {err}"));
    assert!(
        out.status.success(),
        "{args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

fn write(path: PathBuf, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the fixture directory is writable");
    }
    std::fs::write(path, body).expect("the fixture file is writable");
}

/// Write a workspace manifest, its members and a matching `Cargo.lock`, all naming one version.
pub fn workspace_files(repo: &Path, version: &str) {
    write(
        repo.join("Cargo.toml"),
        &format!(
            "[workspace]\nmembers = [\"crates/xuanji\", \"crates/tianheng\", \"crates/renamed-dir\"]\n\n\
             [workspace.package]\nversion = \"{version}\"\n\n\
             [workspace.dependencies]\nxuanji = {{ path = \"crates/xuanji\", version = \"{version}\" }}\n"
        ),
    );
    // `xuanji` publishes and `tianheng` does not, so a fixture exercises both sides of the criterion the
    // machinery corpus reads from the manifests. Each member carries a `src/lib.rs`, because a workspace
    // cargo cannot load is one this gate cannot enumerate — the fixture is a real workspace or it is not
    // evidence about one.
    for (package, publishes) in [("xuanji", true), ("tianheng", false)] {
        let publish = if publishes { "" } else { "publish = false\n" };
        write(
            repo.join(format!("crates/{package}/Cargo.toml")),
            &format!(
                "[package]\nname = \"{package}\"\nversion.workspace = true\nedition = \"2024\"\n{publish}"
            ),
        );
        write(repo.join(format!("crates/{package}/src/lib.rs")), "");
    }
    // **A member whose directory is not its package name.** Without it, the fixture's two sides agree by
    // construction — every member sits at `crates/<name>/` — so a corpus that derived the directory from the
    // package name would pass every row here while being wrong about any workspace that does not. It is
    // unpublished, so its files must reach the machinery set: if the derivation regresses, this member
    // contributes nothing and a changelog naming its gate reports clean.
    write(
        repo.join("crates/renamed-dir/Cargo.toml"),
        "[package]\nname = \"machinery-under-another-name\"\nversion.workspace = true\n\
         edition = \"2024\"\npublish = false\n",
    );
    write(repo.join("crates/renamed-dir/src/lib.rs"), "");
    write(
        repo.join("crates/renamed-dir/tests/renamed_gate.rs"),
        "#[test]\nfn t() {}\n",
    );
    let minor = version.rsplit_once('.').map(|(h, _)| h).unwrap_or(version);
    // The example package the fixture carries, named through a binding like the members above rather than
    // as one path literal: a literal here reads as a reference into *this* repository, which the reference
    // gate then reports as stale — the path belongs to the fixture, not to the tree being judged.
    let example = "adopter";
    write(
        repo.join(format!("examples/{example}/Cargo.toml")),
        &format!(
            "[package]\nname = \"{example}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nxuanji = \"{minor}\"\n"
        ),
    );
    write(
        repo.join("Cargo.lock"),
        &format!(
            "version = 4\n\n[[package]]\nname = \"tianheng\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"xuanji\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"machinery-under-another-name\"\nversion = \"{version}\"\n"
        ),
    );
}

/// Write a changelog in the development shape: an `[Unreleased]` section, carrying an adopter-facing item
/// only when asked, so its absence can be refused.
pub fn development_changelog(repo: &Path, version: &str, with_item: bool) {
    let item = if with_item {
        "- An adopter-facing change.\n\n"
    } else {
        ""
    };
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n{item}[Unreleased]: {COMPARE}/v{version}...HEAD\n"
        ),
    );
}

/// Write a changelog in the release shape: a dated section for `version`, with the link block naming
/// `previous`.
pub fn release_changelog(repo: &Path, version: &str, previous: &str) {
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n## [{version}] - 2026-07-20\n\n- Release notes.\n\n\
             [Unreleased]: {COMPARE}/v{version}...HEAD\n[{version}]: {COMPARE}/v{previous}...v{version}\n"
        ),
    );
}

/// A repository released at `version` over a `0.1.0` predecessor. Prints its path.
pub fn build_fixture(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");
    run(&repo, &["git", "init", "-q", "-b", "main"]);
    run(
        &repo,
        &["git", "config", "user.name", "Release Coherence Test"],
    );
    run(
        &repo,
        &[
            "git",
            "config",
            "user.email",
            "release-coherence@example.invalid",
        ],
    );
    run(&repo, &["git", "config", "commit.gpgsign", "false"]);

    workspace_files(&repo, "0.1.0");
    release_changelog(&repo, "0.1.0", "0.0.0");
    run(&repo, &["git", "add", "."]);
    run(&repo, &["git", "commit", "-qm", "release: 0.1.0"]);

    workspace_files(&repo, version);
    release_changelog(&repo, version, "0.1.0");
    run(&repo, &["git", "add", "."]);
    run(
        &repo,
        &["git", "commit", "-qm", &format!("release: {version}")],
    );

    Fixture { repo }
}

/// Commit everything in the fixture under one subject, with hermetic identity and no signing.
pub fn commit(repo: &Path, subject: &str) {
    run(repo, &["git", "add", "."]);
    run(repo, &["git", "commit", "-qm", subject]);
}
