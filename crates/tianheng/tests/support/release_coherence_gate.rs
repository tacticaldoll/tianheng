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

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Kind {
    Violation,
    CannotJudge,
}

#[derive(Debug, Clone)]
pub struct Refusal {
    pub kind: Kind,
    pub message: String,
}

fn violation(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::Violation,
        message: message.into(),
    }
}

fn cannot_judge(message: impl Into<String>) -> Refusal {
    Refusal {
        kind: Kind::CannotJudge,
        message: message.into(),
    }
}

pub fn hermetic(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command
}

fn git(repo: &Path, args: &[&str]) -> Result<String, String> {
    let out = hermetic("git")
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|err| format!("cannot run git {args:?}: {err}"))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim_end().to_string())
    }
}

fn read(repo: &Path, rel: &str) -> Result<String, Refusal> {
    std::fs::read_to_string(repo.join(rel))
        .map_err(|err| cannot_judge(format!("could not read {rel}: {err}")))
}

/// The first `version = "…"` under `[workspace.package]`.
fn workspace_version(text: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if inside {
            if let Some(rest) = trimmed
                .strip_prefix("version")
                .and_then(|rest| rest.trim_start().strip_prefix('='))
            {
                return Some(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

fn semver(version: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.chars().all(|c| c.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return None;
        }
        out[index] = part.parse().ok()?;
    }
    Some((out[0], out[1], out[2]))
}

fn first_string_value(line: &str) -> Option<String> {
    let (_, rest) = line.split_once('"')?;
    let (value, _) = rest.split_once('"')?;
    Some(value.to_string())
}

fn package_name(manifest: &str) -> Option<String> {
    manifest
        .lines()
        .find(|line| line.trim_start().starts_with("name") && line.contains('='))
        .and_then(first_string_value)
}

/// Which phase of the release ritual this repository is in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    Development,
    ReleaseReady,
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
    git(repo, &["rev-parse", "--is-inside-work-tree"]).map_err(|_| {
        cannot_judge(format!(
            "repository root {} has no git history",
            repo.display()
        ))
    })?;

    let root_manifest = read(repo, "Cargo.toml")?;
    let version = workspace_version(&root_manifest).unwrap_or_default();
    let Some(version_parts) = semver(&version) else {
        return Err(cannot_judge(format!(
            "workspace version is missing or malformed: {}",
            if version.is_empty() {
                "<missing>"
            } else {
                &version
            }
        )));
    };
    let changelog = read(repo, "CHANGELOG.md")?;

    // --- the release spine ---
    let subjects = git(repo, &["log", "--format=%H%x09%s"])
        .map_err(|err| cannot_judge(format!("could not read the release history: {err}")))?;
    let mut history: Vec<(String, String)> = Vec::new();
    for line in subjects.lines() {
        let Some((commit, subject)) = line.split_once('\t') else {
            continue;
        };
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
    let head = git(repo, &["rev-parse", "HEAD"])
        .map_err(|err| cannot_judge(format!("could not read HEAD: {err}")))?;

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

    // --- the version-bearing surfaces ---
    let manifests = workspace_manifests(repo)?;
    for (path, text) in &manifests {
        let name = package_name(text).unwrap_or_else(|| path.clone());
        let inherits = text.lines().any(|line| {
            let line = line.trim();
            let line = line.split('#').next().unwrap_or(line).trim_end();
            line.replace(' ', "") == "version.workspace=true"
        });
        if !inherits {
            return Err(violation(format!(
                "workspace package {name} must inherit version.workspace = true"
            )));
        }
    }
    require_internal_pins(&root_manifest, &version)?;
    require_example_pins(repo, &manifests, &version)?;

    // --- state-dependent changelog surfaces ---
    let unreleased_sections = changelog
        .lines()
        .filter(|line| line.trim_end() == "## [Unreleased]")
        .count();
    if unreleased_sections != 1 {
        return Err(violation(
            "CHANGELOG must contain exactly one [Unreleased] section".to_string(),
        ));
    }
    let has_item = unreleased_has_item(&changelog);
    match state {
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
                    state.label()
                )));
            }
            let dated = changelog.lines().any(|line| {
                let line = line.trim_end();
                line.starts_with(&format!("## [{version}] - "))
                    && line.len() == format!("## [{version}] - ").len() + 10
            });
            if !dated {
                return Err(violation(format!(
                    "CHANGELOG is missing dated release notes for {version}"
                )));
            }
            let from = if state == State::ReleaseReady {
                Some(release_version.clone())
            } else {
                previous_release.clone()
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
            require_lock_versions(repo, &manifests, &version)?;
        }
    }

    // --- each release section's internal consistency ---
    // The vacuity guard this walk once carried is UNREACHABLE and is gone. `## [Unreleased]` is itself a
    // `## [` section, and the exactly-one-`[Unreleased]` check above already refuses a changelog with none —
    // more specifically, and as a violation rather than an undecidable. A guard whose input an earlier check
    // forecloses cannot fire, and keeping it would read as coverage. Found by trying to write its WHEN.
    let shape = section_shape(&changelog);
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

    // --- adopter narrative names no self-governance machinery ---
    let leaked = adopter_cited_machinery(repo, &changelog)?;
    if !leaked.is_empty() {
        return Err(violation(format!(
            "an adopter-facing CHANGELOG entry names this repository's own machinery, which ships in no \
             package and which an adopter can never run — move it under `### Self-governance`, or, where the \
             adopter-relevant fact is genuinely there, state the guarantee and drop the filename:\n{}",
            leaked.join("\n")
        )));
    }

    Ok(format!(
        "ok release coherence ({}: {version})",
        state.label()
    ))
}

fn workspace_manifests(repo: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let crates = repo.join("crates");
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&crates).map_err(|err| {
        cannot_judge(format!(
            "found no workspace crate manifests under {}: {err} — the crate layout changed or is absent, so \
             version inheritance cannot be judged",
            crates.display()
        ))
    })?;
    let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    dirs.sort();
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
    for line in root_manifest.lines() {
        let trimmed = line.trim();
        if !trimmed.contains("path") || !trimmed.contains("\"crates/") || !trimmed.contains('=') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        pins += 1;
        let name = name.trim();
        let pin = trimmed
            .split("version")
            .nth(1)
            .and_then(|rest| rest.split_once('"'))
            .and_then(|(_, rest)| rest.split_once('"'))
            .map(|(value, _)| value.to_string());
        match pin {
            None => {
                return Err(violation(format!(
                    "internal dependency {name} has no version pin"
                )));
            }
            Some(pin) if pin != version => {
                return Err(violation(format!(
                    "internal dependency {name} is pinned to {pin}; expected {version}"
                )));
            }
            Some(_) => {}
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
    let family: Vec<String> = manifests
        .iter()
        .filter_map(|(_, text)| package_name(text))
        .collect();
    let minor = version
        .rsplit_once('.')
        .map(|(head, _)| head)
        .unwrap_or(version);
    let mut example_manifests = 0usize;
    let mut requirements = 0usize;

    let Ok(entries) = std::fs::read_dir(repo.join("examples")) else {
        return Err(cannot_judge(
            "found no example manifests under examples/ — the layout changed or is absent, so example pins \
             would be judged over nothing",
        ));
    };
    let mut dirs: Vec<PathBuf> = entries.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    dirs.sort();
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        let Ok(text) = std::fs::read_to_string(&manifest) else {
            continue;
        };
        example_manifests += 1;
        let name = dir
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();
        for line in text.lines() {
            let trimmed = line.trim();
            let Some((key, rest)) = trimmed.split_once('=') else {
                continue;
            };
            let key = key.trim();
            if !family.iter().any(|f| f == key) {
                continue;
            }
            let pin = if rest.trim_start().starts_with('{') {
                rest.split("version")
                    .nth(1)
                    .and_then(|r| r.split_once('"'))
                    .and_then(|(_, r)| r.split_once('"'))
                    .map(|(v, _)| v.to_string())
            } else {
                first_string_value(rest)
            };
            let Some(pin) = pin else { continue };
            requirements += 1;
            if pin != minor && pin != version {
                return Err(violation(format!(
                    "example {name} requires {key} = \"{pin}\", which the workspace version {version} does \
                     not satisfy"
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
    let mut versions: BTreeMap<String, String> = BTreeMap::new();
    let mut name = String::new();
    for line in lock.lines() {
        let trimmed = line.trim();
        if trimmed == "[[package]]" {
            name.clear();
        } else if trimmed.starts_with("name") && trimmed.contains('=') {
            name = first_string_value(trimmed).unwrap_or_default();
        } else if trimmed.starts_with("version") && trimmed.contains('=') && !name.is_empty() {
            if let Some(value) = first_string_value(trimmed) {
                versions.entry(name.clone()).or_insert(value);
            }
        }
    }
    for (_, text) in manifests {
        let Some(package) = package_name(text) else {
            continue;
        };
        match versions.get(&package) {
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
    sections: BTreeSet<String>,
    headings: BTreeMap<(String, String), usize>,
    breaking: BTreeSet<String>,
}

/// The document's grammar — which sections exist, which headings each carries, which mark a break.
///
/// The line between this and an entry's *content* is where the decidable stops: whether an entry is accurate,
/// whether "no adopter action" is true, whether a named symbol exists are judgements over prose, and the
/// detector they would need is the one this repository measured three times and rejected.
fn section_shape(changelog: &str) -> Shape {
    let mut shape = Shape {
        sections: BTreeSet::new(),
        headings: BTreeMap::new(),
        breaking: BTreeSet::new(),
    };
    let mut section = String::new();
    for line in changelog.lines() {
        if line.starts_with("## [") {
            section = line
                .split(" - ")
                .next()
                .unwrap_or(line)
                .trim_end()
                .to_string();
            shape.sections.insert(section.clone());
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

/// Every adopter-facing `[Unreleased]` entry naming this repository's own machinery.
///
/// A name is a **word** — a maximal run of path characters, required to equal a tracked path, a tracked
/// basename, or an ancestor directory derived from the enumeration. That is exact matching of a lexical token,
/// not substring matching: the run is delimited by the first character a path cannot hold. An earlier rule
/// compared whole backticked spans and three shapes this document already uses passed clean — a span carrying
/// a command, a padded double-backtick span, and an inline span wrapped across a source line.
///
/// Adopter-facing is the **complement** of `### Self-governance`, so a heading nobody anticipated reacts
/// rather than being exempt by default. Dated sections are record: rewriting one to satisfy a rule written
/// afterwards would falsify it.
/// Every word that names this repository's own machinery: a tracked path under `scripts/`, its basename, or an
/// ancestor directory derived from that enumeration.
fn machinery_names(repo: &Path) -> Result<BTreeSet<String>, Refusal> {
    let listing = git(repo, &["ls-files", "scripts/"])
        .map_err(|err| cannot_judge(format!("could not enumerate scripts/: {err}")))?;
    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in listing.lines().filter(|l| !l.is_empty()) {
        names.insert(path.to_string());
        if let Some(base) = path.rsplit('/').next() {
            names.insert(base.to_string());
        }
        let mut dir = path.to_string();
        while let Some(cut) = dir.rfind('/') {
            dir.truncate(cut + 1);
            names.insert(dir.clone());
            dir.truncate(cut);
        }
    }
    Ok(names)
}

fn names_machinery(line: &str, names: &BTreeSet<String>) -> bool {
    line.split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')))
        .any(|run| {
            let token = run.strip_prefix("./").unwrap_or(run).trim_end_matches('.');
            !token.is_empty() && names.contains(token)
        })
}

fn adopter_cited_machinery(repo: &Path, changelog: &str) -> Result<Vec<String>, Refusal> {
    // One enumeration. A second copy lived here for one commit, built for a census that was dropped, and
    // two constructions of one set is the drift this file's own doc-comment says it exists to prevent.
    let names = machinery_names(repo)?;

    let mut found: BTreeSet<String> = BTreeSet::new();
    let mut section = String::new();
    let mut heading = String::new();
    for line in changelog.lines() {
        if line.starts_with("## [") {
            section = line
                .split(" - ")
                .next()
                .unwrap_or(line)
                .trim_end()
                .to_string();
            heading.clear();
            continue;
        }
        if section.is_empty() {
            continue;
        }
        if let Some(next) = line.strip_prefix("### ") {
            heading = next.trim_end().to_string();
        }
        if section != "## [Unreleased]" || heading == "Self-governance" {
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

pub fn workspace_files(repo: &Path, version: &str) {
    write(
        repo.join("Cargo.toml"),
        &format!(
            "[workspace]\nmembers = [\"crates/xuanji\", \"crates/tianheng\"]\n\n\
             [workspace.package]\nversion = \"{version}\"\n\n\
             [workspace.dependencies]\nxuanji = {{ path = \"crates/xuanji\", version = \"{version}\" }}\n"
        ),
    );
    for package in ["xuanji", "tianheng"] {
        write(
            repo.join(format!("crates/{package}/Cargo.toml")),
            &format!(
                "[package]\nname = \"{package}\"\nversion.workspace = true\nedition = \"2024\"\n"
            ),
        );
    }
    let minor = version.rsplit_once('.').map(|(h, _)| h).unwrap_or(version);
    write(
        repo.join("examples/adopter/Cargo.toml"),
        &format!(
            "[package]\nname = \"adopter\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nxuanji = \"{minor}\"\n"
        ),
    );
    write(
        repo.join("Cargo.lock"),
        &format!(
            "version = 4\n\n[[package]]\nname = \"tianheng\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"xuanji\"\nversion = \"{version}\"\n"
        ),
    );
}

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

pub fn commit(repo: &Path, subject: &str) {
    run(repo, &["git", "add", "."]);
    run(repo, &["git", "commit", "-qm", subject]);
}
