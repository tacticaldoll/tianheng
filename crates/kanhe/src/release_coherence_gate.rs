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

use crate::refusal::{Refusal, cannot_judge_at, violation_at};
use crate::region::Source;
use crate::sections::Section;

use crate::hermetic_git::fixture as run;
pub use crate::hermetic_git::hermetic;
use crate::manifest::{WorkspaceVersion, semver, workspace_version};

fn git(repo: &Path, args: &[&str]) -> Result<String, crate::hermetic_git::Failure> {
    crate::hermetic_git::run(repo, &[], args)
}

fn read(repo: &Path, rel: &str) -> Result<String, Refusal> {
    std::fs::read_to_string(repo.join(rel)).map_err(|err| {
        cannot_judge_at(
            "release-coherence#changelog-or-manifest-unreadable",
            format!("could not read {rel}: {err}"),
        )
    })
}

/// Which tables a caller means by *a dependency*, because this reader's consumers do not all mean the same
/// thing by it.
///
/// **`[workspace.dependencies]` is a catalog, not a dependency.** Measured: a package whose manifest carries
/// `[workspace.dependencies] xuanji = "0.5"` beside `[dependencies] serde_json = "1"` reports exactly one
/// dependency to `cargo metadata`, and it is not `xuanji`. The catalog is what *members* may inherit, and
/// inheriting is something a member does with `xuanji = { workspace = true }` -- not something the table does
/// on its own. One reader answered every consumer with one unqualified list, so an example manifest carrying
/// a catalog entry counted as an example requiring that crate: the per-example guard that exists to refuse an
/// example declaring **no** family dependency could be satisfied by a table cargo does not read as one.
///
/// The subject is a parameter rather than a field on the result, because a field is a thing a consumer may
/// forget to read -- the shape two reviews found in this same reader one round earlier, in an `escaped` flag
/// that only two of its consumers consulted.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Subject {
    /// What this package itself depends on: its own dependency tables and their target-specific variants.
    ///
    /// A catalog is excluded, because a table offering a version to members is not this package requiring it.
    Requires,
    /// What a workspace root offers its members to inherit: `[workspace.dependencies]` and nothing else.
    ///
    /// Two callers ask for this. The root's internal-pin check wants it *together with* `Requires`, because a
    /// path pin lives in whichever table its author reached for -- a fixture pinning `[dependencies.xuanji]`
    /// in a root manifest is what said so, and that caller asks for both rather than this being a third
    /// subject that silently means the union. And [`offered`] wants it alone, to resolve what a dependency
    /// taking `workspace = true` is actually held to.
    Offers,
}

/// The kinds of table whose entries are dependency declarations.
const DEPENDENCY_KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// What a dependency declares as its version requirement, or why this reader could not tell.
///
/// **Every consumer answers every state, and the compiler is what asks.** Three call sites read a dependency's pin and each
/// decided the refusal class for itself: two matched exhaustively and the third collapsed to `_ => None`,
/// which reported an *absent* key as one this reader *could not read* — the very distinction its sibling had
/// just been repaired to make. A typed result makes the compiler ask each consumer when a state is added,
/// which is the shape [`PackageName`] and [`crate::manifest::WorkspaceVersion`] already carry in this family.
///
/// [`crate::selection::the_only`] is deliberately not used here, for the reason `manifest.rs` records for its
/// own reader: it reports none and several as one refusal, and here they are different facts — an absent pin
/// is the legal `{ path = "…" }` form, and two are a table this reader may not choose from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Declared {
    /// The value as written.
    Value(String),
    /// The key is absent. Legal for both of this reader's keys: a path-only dependency declares no
    /// `version`, and a registry dependency declares no `path`.
    Absent,
    /// A value this reader will not take as a string — quoted as written.
    ///
    /// **Not *not in double quotes*, which is what this said while the reader was hand-rolled.** The parser
    /// reads a literal-quoted string as readily as a basic one, so what reaches here is a value that is no
    /// string at all — an integer, an array, a table — which is the condition
    /// `a_single_quoted_path_or_version_is_read_and_a_non_string_is_not` observes from both sides.
    Unreadable(String),
    /// The requirement is the one the workspace catalog offers, taken with `workspace = true`.
    ///
    /// Only a *pin* is ever this: a `path` is not inheritable in the spelling this reader meets. Resolved
    /// against the catalog in the same manifest by [`offered`], because every example in this repository is
    /// its own workspace root -- the root manifest says so, and `exclude` keeps them out of this workspace.
    Inherited,
}

/// Which crate a dependency names, or why this reader cannot say.
///
/// **There is no *absent* state, because absence is not unnameability here.** A dependency declaring no
/// `package` key names the crate by its own key — that is cargo's rule, not a gap — so absence resolves to a
/// name rather than to a missing one. What the rest of the enum carries is one distinct way of *failing* to
/// name it each: a value this reader cannot read, more than one such key, a key that is not bare, and a field
/// whose own key could not be decoded. Each has its own refusal site and its own sentence, because a reader
/// told the wrong cause looks for the wrong thing — which is what this type exists to prevent, and what it
/// did wrong twice while the last two of those were folded into the first.
///
/// It was a `String` with the empty string standing for both of those, in the same struct whose `pin` field
/// had just been given `Declared::{Absent, Unreadable, Several}` for exactly this distinction: one field was typed
/// and its sibling was left as a sentinel, so *several `package` keys* and *a `package` value this reader
/// cannot read* reached the operator as one sentence. The sentinel was not injective either — a literal
/// `package = ""` is a third fact that read as the same state.
#[derive(Debug, PartialEq, Eq)]
enum Package {
    /// The crate this dependency names: its `package` value, or its own key where it declares none.
    Named(String),
    /// A `package` value this reader cannot read — a value that is **not a string at all**.
    ///
    /// Not *not in double quotes*, and not *declared twice*: a literal string is read, and a key declared
    /// twice is a document the parser refuses whole, which never reaches this reader.
    Unreadable,
}

/// One dependency a manifest declares: the key it is written under, the package it names, and its pin.
pub(crate) struct Dependency {
    key: String,
    package: Package,
    pub(crate) pin: Declared,
    path: Declared,
}

/// One field of a dependency's table, as a [`Declared`].
///
/// Absent, a string, or a value this reader will not take as one. Two states a hand-rolled reader also
/// carried are gone: a key it could not decode, because the parser decodes every spelling cargo decodes; and
/// the same key twice, because that is a document cargo itself refuses, so it never reaches here.
fn declared_field(table: &dyn toml_edit::TableLike, name: &str) -> Declared {
    match table.get(name) {
        None => Declared::Absent,
        Some(value) => match value.as_str() {
            Some(text) => Declared::Value(text.to_string()),
            None => Declared::Unreadable(value.to_string().trim().to_string()),
        },
    }
}

/// One dependency, however its author spelled it.
///
/// **The spellings are one thing to a parser, and telling them apart by hand is what this reader kept being
/// repaired for.** `xuanji = "0.5"`, `xuanji = { version = "0.5" }`, `xuanji.version = "0.5"` and
/// `[dependencies.xuanji]` with `version` on its own line are the same entry to cargo. The hand-rolled reader
/// filed the dotted form as two dependencies, read a quoted tail as no path, and could not tell a key named
/// `version.extra` from structure beneath `version` — each a false negative in front of `cargo publish`, and
/// each answered here by asking the document instead of the line.
fn dependency_of(key: &str, item: &toml_edit::Item) -> Dependency {
    // `xuanji = "0.5"`: the whole entry is the requirement, and the key is the crate.
    if let Some(version) = item.as_str() {
        return Dependency {
            key: key.to_string(),
            package: Package::Named(key.to_string()),
            pin: Declared::Value(version.to_string()),
            path: Declared::Absent,
        };
    }
    let Some(table) = item.as_table_like() else {
        // Neither a string nor a table — `xuanji = 5`. What it requires and where it points are both
        // undecided, and answering that for each keeps a caller asking either question from reading past it.
        let written = item.to_string().trim().to_string();
        return Dependency {
            key: key.to_string(),
            package: Package::Named(key.to_string()),
            pin: Declared::Unreadable(written.clone()),
            path: Declared::Unreadable(written),
        };
    };
    let package = match table.get("package") {
        None => Package::Named(key.to_string()),
        Some(renamed) => match renamed.as_str() {
            Some(name) => Package::Named(name.to_string()),
            None => Package::Unreadable,
        },
    };
    // `workspace = true`, in every spelling of the two keys: the catalog holds this one, so it declares no
    // requirement of its own. Anything else under that key is not the offer being taken.
    let inherits = table
        .get("workspace")
        .and_then(toml_edit::Item::as_bool)
        .unwrap_or(false);
    Dependency {
        key: key.to_string(),
        package,
        pin: if inherits {
            Declared::Inherited
        } else {
            declared_field(table, "version")
        },
        path: declared_field(table, "path"),
    }
}

/// Every dependency `subject` admits, read from the parsed document.
///
/// **Which tables, asked of the tree rather than of a heading's text.** A heading was matched segment by
/// segment against every admitted form — `[dependencies]`, `[dependencies.NAME]`, `[target.<sel>.<kind>]`
/// and their `[workspace.dependencies]` counterparts — and a detailed table's fields were then collected
/// across lines and filed when the *next* heading proved the table over. Walking the document removes both:
/// `[dependencies.xuanji]` and `xuanji = { … }` are one entry in one table, and there is no boundary to find.
///
/// **A manifest the parser refuses is refused here, not reported as declaring nothing.** Returning empty was
/// the first shape and the corpus refused it: a duplicate key inside one dependency reached a caller's
/// vacuity floor, which then said *found no dependency on a family crate* — a sentence about the declaration
/// form over a file cargo will not read at all. That is the misdirection this crate's typed readers exist to
/// prevent, so the refusal carries the parse error instead.
pub(crate) fn declared_dependencies(
    text: &str,
    subject: Subject,
) -> Result<Vec<Dependency>, Refusal> {
    let doc = text.parse::<toml_edit::DocumentMut>().map_err(|err| {
        cannot_judge_at(
            "release-coherence#manifest-unparseable",
            crate::manifest::manifest_unreadable(&err),
        )
    })?;
    let mut found = Vec::new();
    let mut take = |item: Option<&toml_edit::Item>| {
        if let Some(table) = item.and_then(toml_edit::Item::as_table_like) {
            for (key, entry) in table.iter() {
                found.push(dependency_of(key, entry));
            }
        }
    };
    match subject {
        Subject::Requires => {
            for kind in DEPENDENCY_KINDS {
                take(doc.get(kind));
            }
            // `[target.<selector>.<kind>]`, where the selector is one key — a triple or a cfg expression,
            // whatever it contains, because a parsed table has no dot for this step to land inside.
            if let Some(targets) = doc.get("target").and_then(toml_edit::Item::as_table_like) {
                for (_selector, target) in targets.iter() {
                    if let Some(target) = target.as_table_like() {
                        for kind in DEPENDENCY_KINDS {
                            take(target.get(kind));
                        }
                    }
                }
            }
        }
        // Only `dependencies` is inheritable; `[workspace.dev-dependencies]` is an unused key to cargo, and
        // only a caller asking what this manifest pins wants it at all.
        Subject::Offers => {
            take(
                doc.get("workspace")
                    .and_then(toml_edit::Item::as_table_like)
                    .and_then(|workspace| workspace.get("dependencies")),
            );
        }
    }
    Ok(found)
}

/// What the catalog in this manifest offers under `key`, for a dependency that took the offer.
///
/// **The lookup is the dependency's key against a catalog key, and the crate comes from the entry.**
/// Measured under cargo 1.96.0: a catalog offering `alias = { package = "realdep", version = "0.0.1" }`
/// beside a dependency spelling `alias = { workspace = true }` resolves to `realdep` at `^0.0.1` under the
/// rename `alias`. Neither shape that would make the dependency's own key an identity survives the same
/// measurement: a `package` written beside `workspace = true` is **accepted and ignored** -- cargo warns
/// `unused manifest key: dependencies.alias.package`, resolves `realdep` from the catalog anyway, and builds
/// -- and inheritance spelled under the crate's name rather than the catalog's key is refused outright,
/// `dependency.realdep was not found in workspace.dependencies`. So there is one lookup, it is by key, and
/// the crate is the catalog entry's even where the dependency names another. Searching by resolved identity
/// asked a question cargo never asks, and matched no entry at all for every crate the catalog renames.
///
/// **The catalog is in the same manifest, because every example in this repository is its own workspace
/// root.** The root manifest's own comment says so and `exclude` enforces it, so a dependency spelling
/// `workspace = true` resolves against `[workspace.dependencies]` beside it. Measured: cargo resolves the
/// inline, dotted and detailed spellings of the offer to the catalog's requirement, and it resolves it even
/// when a local `version` sits in the same inline table -- so the catalog is *the* answer rather than one of
/// two. Cargo refuses a manifest that inherits what its catalog does not declare, so [`Offered::Missing`]
/// describes a manifest nothing builds -- but what to do about it depends on whether the local key names a
/// family crate, so it is answered by the **caller** rather than here. This sentence said `Missing` *is a
/// refusal rather than a fallback*, which held while this search was reached only for a crate already known
/// to be in the family, and stopped holding when the search moved in front of that question.
fn offered(catalog: &[Dependency], key: &str) -> Offered {
    for entry in catalog {
        if entry.key != key {
            continue;
        }
        return match &entry.package {
            Package::Named(named) => Offered::Entry {
                package: named.clone(),
                pin: entry.pin.clone(),
            },
            // The entry being taken names a crate this reader cannot read. *Might be a family crate* is not
            // an answer, and passing it over is how a stale pin would reach a release through the catalog.
            Package::Unreadable => Offered::Unresolvable(entry.key.clone()),
        };
    }
    Offered::Missing
}

/// What a catalog offers under one key.
#[derive(Debug)]
enum Offered {
    /// The catalog declares an entry there: the crate it names, and the requirement it carries -- which may
    /// itself be absent, unreadable or take an offer of its own, and is then answered by the same arms a
    /// locally declared one is.
    Entry { package: String, pin: Declared },
    /// No catalog entry is written under that key.
    ///
    /// One fact, since this reader is handed a catalog that is already parsed: a manifest the parser refuses
    /// never reaches here, because the caller met that refusal before it had a catalog to search. The state
    /// carried both for as long as the search did its own parsing, and *nothing is written there* and *the
    /// document is not a manifest* are different things to tell an operator.
    Missing,
    /// The entry written under that key names a crate this reader cannot resolve, quoted by its key.
    Unresolvable(String),
}

/// Whether `suffix` is an ISO date: three `-`-separated all-digit fields of widths 4, 2 and 2.
///
/// **Parsed, not counted.** The test this replaces asserted the heading was ten characters longer than its
/// own prefix and never read them, so `## [0.5.0] - notadate!!` satisfied *CHANGELOG carries dated release
/// notes*. A length test is a parse without its guarantee.
///
/// **And the day is answered against the calendar, by the reader that owns one.** Reading three all-digit
/// fields of the right widths admitted `2026-99-99` and `0000-00-00`; ranging them to the calendar's outer
/// bounds — a month `1..=12`, a day `1..=31` — still admitted `2026-02-31`, and that residue was recorded
/// here on the ground that a calendar was a dependency this crate's surface did not carry. It does:
/// `reading::date` reads `YYYY-MM-DD` through `days_in_month`, leap years included, and this now delegates to
/// it. Two date readers in one crate, the weaker one used where the stronger existed, is the shape this crate
/// converges rather than documents. What is left of the old residue — `2026-02-31` — is a date a human wrote wrong rather
/// than a shape that reads as one.
pub fn is_iso_date(suffix: &str) -> bool {
    crate::reading::date("changelog section date", suffix).is_ok()
}

/// What a member manifest says its package is called, or why this reader could not tell.
///
/// Typed apart rather than an `Option`, because every consumer here treated `None` as *not a package* and
/// skipped it — so a manifest this reader could not parse left its package's lock version unchecked and its
/// examples' pins unexamined, with the function still returning `Ok`. The template is
/// `capability_subjects::Declared`, applied to a second reader.
pub enum PackageName {
    /// The `[package]` table's `name`.
    Named(String),
    /// No `[package]` table, or no `name` key inside it.
    Absent,
    /// A `name` this reader cannot read: a value that is **not a string at all**.
    ///
    /// Neither of the two shapes this once also carried reaches it. A literal string is read, and a `name`
    /// declared twice is a document the parser refuses whole rather than a key with two answers.
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
/// return carries the distinction instead of collapsing it.
pub fn package_name(manifest: &str) -> PackageName {
    let doc = match manifest.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        // A manifest cargo cannot parse declares no name to be reported absent, and answering `Absent` would
        // send an operator to add a key that may already be there. The whole error, collapsed: a duplicate
        // key reports its position on the first line and names the key on later ones.
        Err(err) => {
            return PackageName::Unreadable(crate::manifest::manifest_unreadable(&err));
        }
    };
    let Some(name) = doc
        .get("package")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("name"))
    else {
        return PackageName::Absent;
    };
    match name.as_str() {
        Some(named) => PackageName::Named(named.to_string()),
        // What remains unreadable is a `name` that is not a string: an inheritance spelling, an inline table,
        // an array. The old reader also answered this for a single-quoted string, which cargo accepts.
        None => PackageName::Unreadable(name.to_string().trim().to_string()),
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
    /// That commit's own date, `YYYY-MM-DD`, which the dated section is held against at the snapshot.
    release_date: String,
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
    changelog: &str,
) -> Result<Spine, Refusal> {
    // `%ad` with `--date=short`, because the dated release section's value is held against the release
    // commit's own date and reading it here costs nothing — the log that answers "which commit" answers
    // "when" in the same line.
    let subjects =
        git(repo, &["log", "--date=short", "--format=%H%x09%ad%x09%s"]).map_err(|err| {
            cannot_judge_at(
                "release-coherence#release-history-unreadable",
                format!("could not read the release history: {err}"),
            )
        })?;
    let mut history: Vec<(String, String, String)> = Vec::new();
    // HEAD's own commit is the first line this log produced, so asking git for it again would be a second
    // read of something already in hand — and a refusal guarding that second read is a branch no input can
    // take. Taken here instead.
    let mut head: Option<String> = None;
    for line in subjects.lines() {
        let Some((commit, rest_of_line)) = line.split_once('\t') else {
            continue;
        };
        let Some((date, subject)) = rest_of_line.split_once('\t') else {
            continue;
        };
        if head.is_none() {
            head = Some(commit.to_string());
        }
        if let Some(rest) = subject.strip_prefix("release: ") {
            if semver(rest).is_none() {
                return Err(violation_at(
                    "release-coherence#release-history-version-malformed",
                    format!("malformed release history subject: {subject}"),
                ));
            }
            history.push((commit.to_string(), date.to_string(), rest.to_string()));
        } else if subject.starts_with("release:") {
            return Err(violation_at(
                "release-coherence#release-history-subject-malformed",
                format!("malformed release history subject: {subject}"),
            ));
        }
    }
    let Some((release_commit, release_date, release_version)) = history.first().cloned() else {
        return Err(cannot_judge_at(
            "release-coherence#release-history-shallow",
            "exact release history is unavailable; fetch full history containing release: X.Y.Z — a shallow \
             clone cannot see the release spine, which is not the same as surfaces that disagree",
        ));
    };
    let previous_release = history.get(1).map(|(_, _, v)| v.clone());
    // A release commit exists, so at least one line of the log parsed, so this is Some. Provable from the
    // loop above rather than assumed about git.
    let head =
        head.expect("the log line that produced a release commit also produced HEAD's own commit");

    // **A snapshot is a checkout, not a commit.** This asked `head == release_commit` alone — a fact about
    // the COMMIT — while every other reader in this gate judges the worktree, which `read` takes with
    // `std::fs::read_to_string`. Two sources, one answer, and the first edit of the next cycle falls between
    // them: at the release commit, writing the `[Unreleased]` entry that `Development` **requires** is judged
    // in `Snapshot`, where `[Unreleased]` must be **empty**. Measured on `release/0.6.0`'s first change — the
    // tree could not be made to pass until it was committed, because committing is what moved `head`.
    //
    // **The CHANGELOG is what carries the cycle, so it is what decides.** A first attempt asked whether
    // *anything* tracked was modified, and two existing directions refuted it: a fixture whose `Cargo.lock`
    // has been replaced by a directory is a **broken release checkout**, not the next cycle beginning, and
    // classifying it as `Development` made it refuse for a missing `[Unreleased]` entry before it could
    // report the lockfile it cannot read. The directions were right and the wider rule was wrong.
    //
    // What distinguishes the two states is exactly what this repository's own requirement names: *active
    // development SHALL retain the current released version and at least one changelog list item under
    // `[Unreleased]`*. Writing that item is how a cycle begins, and it is a change to this file. Untracked
    // files are excluded — this gate reads named tracked paths, so a file it never opens decides nothing.
    // **Read from the object database, not the index.** `git status` was the first spelling and it took a
    // WHEN that belonged to another guard: an existing direction corrupts `.git/index` to make the machinery
    // enumeration fail, and a `status` here intercepted it — measured, that direction stopped reaching its
    // own site. `git show HEAD:…` reads the tree, so the same corrupt index leaves it working, also measured.
    //
    // `hermetic_git::run` trims trailing whitespace from git's output, so the committed text arrives without
    // its final newline while `read` keeps one. Measured: every snapshot direction in the corpus failed on
    // that difference alone before the two sides were compared on the same footing, which is why the exact
    // read is used here.
    //
    // **Presence is asked first, by a command whose exit status answers it.** `git show HEAD:…` exits `128`
    // for a path that is not in HEAD *and* for a tree it cannot read, so a single `Err` arm had to choose one
    // meaning for both — and choosing *not a snapshot* classified a broken object store as the next cycle.
    // Measured on this machine's git: `ls-tree HEAD -- <path>` exits `0` with an empty listing when the path
    // is absent, `0` with a line when it is there, and `128` only when the tree cannot be read. The question
    // decides the command, which is what the sibling tag-presence reader already does for the same shape.
    // **Presence is asked first, by a command whose exit status answers it.** `git show HEAD:…` exits `128`
    // for a path that is not in HEAD *and* for a tree it cannot read, so a single `Err` arm had to choose one
    // meaning for both — and choosing *not a snapshot* classified a broken object store as the next cycle.
    // Measured on this machine's git: `ls-tree HEAD -- <path>` exits `0` with an empty listing when the path
    // is absent, `0` with a line when it is there, and `128` only when the tree cannot be read. The question
    // decides the command, which is what the sibling tag-presence reader already does for the same shape.
    let listed = crate::hermetic_git::run(repo, &[], &["ls-tree", "HEAD", "--", "CHANGELOG.md"])
        .map_err(|err| {
            cannot_judge_at(
                "release-coherence#changelog-in-head-unreadable",
                format!(
                    "git could not answer what HEAD's tree holds for `CHANGELOG.md` ({err}), so whether the \
                     worktree still matches the release commit was never read"
                ),
            )
        })?;
    // **A release commit that carries no changelog is its own fact, not a modified checkout.** Absence at
    // any other commit is unremarkable — a tree from before the file existed. At the exact `release: X.Y.Z`
    // commit it means the release shipped without the document it is narrated in, and reading that as *the
    // next cycle has begun* let it pass on the worktree's copy alone.
    let unmodified = if listed.trim().is_empty() {
        if head == release_commit {
            return Err(violation_at(
                "release-coherence#release-commit-carries-no-changelog",
                format!(
                    "the release commit for {release_version} carries no `CHANGELOG.md` in its own tree, so \
                     the release it names is narrated nowhere a reader of that commit can reach"
                ),
            ));
        }
        false
    } else {
        match crate::hermetic_git::run_exact(repo, &[], &["show", "HEAD:CHANGELOG.md"]) {
            Ok(committed) => committed == changelog,
            // The path is listed in HEAD's tree, so a failure here is git declining to read a blob it just
            // named — a fact about the object store, never about the worktree.
            Err(other) => {
                return Err(cannot_judge_at(
                    "release-coherence#changelog-blob-unreadable",
                    format!(
                        "HEAD's tree names `CHANGELOG.md` and git could not read it ({other}), so whether \
                         the worktree still matches the release commit was never read"
                    ),
                ));
            }
        }
    };
    let state = if head == release_commit && unmodified {
        if version != release_version {
            return Err(violation_at(
                "release-coherence#release-snapshot-version-disagrees",
                format!(
                    "release snapshot subject is {release_version} but workspace version is {version}"
                ),
            ));
        }
        State::Snapshot
    } else {
        let released =
            semver(&release_version).expect("the history holds only well-formed versions");
        match version_parts.cmp(&released) {
            std::cmp::Ordering::Less => {
                return Err(violation_at(
                    "release-coherence#workspace-version-behind-latest-release",
                    format!(
                        "workspace version {version} is older than latest release {release_version}"
                    ),
                ));
            }
            std::cmp::Ordering::Equal => State::Development,
            std::cmp::Ordering::Greater => State::ReleaseReady,
        }
    };
    Ok(Spine {
        state,
        release_version,
        release_date,
        previous_release,
    })
}

/// Every version-bearing surface outside the changelog, and the member **names** the later phases read.
///
/// **The `and then` in that sentence is real and the obvious repair for it is not.** A review opened Gate 4
/// on it: this runs the per-member inherit loop and then calls the two pin checks, so its job needs two
/// clauses to state, and its name says *surfaces*. The suggested repair was to have the caller sequence the
/// three, since the `Vec<(String, String)>` return already exists for it — and that return is **not** the
/// manifests those checks consume. It is the `(path, name)` pairs `require_example_pins` produces, which
/// `require_changelog_state` and the lock reader read. The move was made and the failure matrix refused it:
/// `manifests` at the caller became the `(path, text)` list, and the lock check reported *Cargo.lock is
/// missing workspace package* with a whole manifest where a name belongs.
///
/// So the flow stays, and what is worth changing is the thing that made the move look safe: two lists of the
/// same type with different meanings in one function. `BACKLOG.md` carries that with its trigger, rather than
/// a rename of this function that would leave the swap compiling.
fn require_version_surfaces(
    repo: &Path,
    root_manifest: &str,
    version: &str,
) -> Result<Vec<Member>, Refusal> {
    let manifests = workspace_manifests(repo)?;
    for (path, text) in &manifests {
        // Only the refusal message needs the name here — the inheritance read below works off the text
        // whichever state this is, so an unnameable package is reported by path rather than skipped. The
        // third consumer, and the only one for which that is the right answer.
        let name = match package_name(text) {
            PackageName::Named(name) => name,
            PackageName::Absent | PackageName::Unreadable(_) => path.clone(),
        };
        // **One expression over a parsed document, where a line-oriented reader spent four rounds.** Each
        // round moved the boundary of *decoded* one segment right and each closed real false refusals:
        // measured under cargo 1.96.0, all of `version.workspace = true`, `version = { workspace = true }`,
        // `"version".workspace = true`, `'version'.workspace = true`, the quoted and escaped spellings of
        // the tail, and the quoted inner key inherit — and a raw-text recogniser took one of them.
        //
        // The fourth round is the one that ended the approach rather than extending it: a member may inherit
        // through a **sub-table heading**, `[package.version]` with `workspace = true`, which cargo resolves
        // — measured in a scratch workspace — and which a reader asking each line *does this assign
        // `version`* cannot represent at all, because a heading assigns nothing. There was no segment left to
        // move.
        //
        // The parser also makes the read `[package]`-scoped, which the line walk was not: it took an
        // assignment in any table. Cargo honours `version.workspace` under `[package]` and nowhere else, so
        // narrowing is the answer agreeing with cargo, not a tightening.
        //
        // Directions: `every_inherit_spelling_cargo_honours_is_read_as_inheriting`,
        // `a_member_inheriting_through_a_sub_table_heading_is_read_as_inheriting`,
        // `an_inherit_line_with_a_glued_comment_still_inherits` — `true#c` is legal TOML and the parser
        // takes it — and `a_member_whose_only_inherit_line_is_commented_out_is_refused`.
        let doc = text.parse::<toml_edit::DocumentMut>().map_err(|err| {
            cannot_judge_at(
                "release-coherence#member-manifest-unparseable",
                format!(
                    "{name}: a member manifest this parser cannot read — {}",
                    crate::manifest::parse_error_on_one_line(&err)
                ),
            )
        })?;
        let inherits = doc
            .get("package")
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|package| package.get("version"))
            .and_then(toml_edit::Item::as_table_like)
            .and_then(|version| version.get("workspace"))
            .and_then(toml_edit::Item::as_bool)
            .unwrap_or(false);
        if !inherits {
            return Err(violation_at(
                "release-coherence#member-does-not-inherit-workspace-version",
                format!("workspace package {name} must inherit version.workspace = true"),
            ));
        }
    }
    let members = family_members(&manifests)?;
    require_internal_pins(root_manifest, version, &members)?;
    require_example_pins(repo, &members, version)?;
    Ok(members)
}

/// The changelog surfaces whose required shape depends on which phase of the ritual this is.
fn require_changelog_state(
    repo: &Path,
    prose: crate::region::Prose<'_>,
    sections: &[Section],
    members: &[Member],
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    // The cut answers this. It was a line count over the whole document, which could not tell a real
    // `## [Unreleased]` from one inside a fence — and this is the check the *rest* of this function's
    // reasoning rests on, since every arm below assumes exactly one such section exists.
    let unreleased_sections = sections
        .iter()
        .filter(|section| section.name == "## [Unreleased]")
        .count();
    if unreleased_sections != 1 {
        return Err(violation_at(
            "release-coherence#unreleased-section-not-exactly-one",
            "CHANGELOG must contain exactly one [Unreleased] section".to_string(),
        ));
    }
    let has_item = unreleased_has_item(sections);
    match spine.state {
        State::Development => {
            if !has_item {
                return Err(violation_at(
                    "release-coherence#unreleased-has-no-adopter-narrative",
                    "development requires adopter-facing release narrative under [Unreleased]"
                        .to_string(),
                ));
            }
            let link = format!("[Unreleased]: {COMPARE}/v{version}...HEAD");
            if !prose.lines().any(|line| line.trim_end() == link) {
                return Err(violation_at(
                    "release-coherence#unreleased-comparison-link-wrong",
                    format!(
                        "[Unreleased] comparison link must start at v{version} and end at HEAD"
                    ),
                ));
            }
        }
        State::ReleaseReady | State::Snapshot => {
            if has_item {
                return Err(violation_at(
                    "release-coherence#unreleased-not-empty-in-state",
                    format!(
                        "[Unreleased] must be empty in {} state",
                        spine.state.label()
                    ),
                ));
            }
            // Read off the section's own sentinel line rather than swept for across the document: the
            // derived name drops the ` - DATE` suffix, so this is the one question the name cannot answer and
            // `Section::line` exists for. A sweep also accepted a dated line belonging to no section at all.
            let prefix = format!("## [{version}] - ");
            // **Counted before it is taken, because the first of two answers is not an answer.** This
            // asked `.find()`, so a changelog carrying two `## [{version}]` sections answered from whichever
            // came first: a stale one dated years earlier ahead of the correct one reported *ok release
            // coherence*, and at the snapshot the same selection would compare the stale date against the
            // release commit and refuse naming the wrong line. The sibling check above counts `[Unreleased]`
            // sections and refuses any count but one, saying every arm below assumes exactly one exists —
            // the same assumption was made here and not checked. Four readers in this crate were each given
            // a *several* refusal when someone was in them; this one selects from a document and was never
            // asked.
            // **Counted over the sections, then the survivor is read — not counted over what already
            // parsed.** The first repair placed the count *after* the date filter, so a malformed sibling was
            // invisible to it: `## [{version}] - notadate` above a correct section left one candidate and
            // reported clean, and so did a bare `## [{version}]`. Two headings for one version is the defect
            // whether or not the second parses, and the malformed one — a heading left behind, a typo'd date —
            // is the likelier mistake. The spec's own scenario for a bad suffix fires only when there is no
            // well-formed sibling to hide behind, so nothing observed it.
            let claiming: Vec<&str> = sections
                .iter()
                .filter(|section| section.name == format!("## [{version}]"))
                .map(|section| section.line.trim_end())
                .collect();
            if claiming.len() > 1 {
                return Err(cannot_judge_at(
                    "release-coherence#several-release-sections",
                    format!(
                        "CHANGELOG carries {} sections for {version} ({}), so which one records the release \
                     is not this reader's to choose",
                        claiming.len(),
                        claiming.join(", ")
                    ),
                ));
            }
            let dated: Vec<&str> = claiming
                .iter()
                .filter_map(|line| line.strip_prefix(&prefix))
                .filter(|rest| is_iso_date(rest))
                .collect();
            let Some(dated) = dated.first().copied() else {
                return Err(violation_at(
                    "release-coherence#dated-release-notes-missing",
                    format!("CHANGELOG is missing dated release notes for {version}"),
                ));
            };
            // **Which date, not merely a date.** `is_iso_date` was hardened twice — parsed rather than
            // counted, then ranged rather than digit-tested — and each step asked a sharper question about
            // the SHAPE. The value was never asked, and the value is what a reader takes the release to have
            // happened on. Three releases got it right by someone remembering; the fourth was prepared with
            // a date four days behind the day it would be cut on, and nothing said so.
            //
            // Only at the snapshot, because that is the first moment the answer exists: before the
            // `release: X.Y.Z` commit there is no release commit to be dated against, and a date written
            // during preparation is an intent rather than a claim. Held here rather than by the wrapper,
            // since the wrapper stands in front of the publish and this is a property of the commit.
            if spine.state == State::Snapshot && dated != spine.release_date {
                return Err(violation_at(
                    "release-coherence#release-date-disagrees-with-its-commit",
                    format!(
                        "CHANGELOG dates {version} at {dated} and its `release: {version}` commit was made \
                         on {} — a reader takes the section's date for the day the release happened",
                        spine.release_date
                    ),
                ));
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
            if !prose.lines().any(|line| line.trim_end() == expected) {
                return Err(violation_at(
                    "release-coherence#release-comparison-link-wrong",
                    match &from {
                        Some(previous) => {
                            format!(
                                "CHANGELOG comparison link for {version} must start at v{previous}"
                            )
                        }
                        None => format!("first release CHANGELOG link must target v{version}"),
                    },
                ));
            }
            require_lock_versions(repo, members, version)?;
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
fn require_section_shape(sections: &[Section]) -> Result<(), Refusal> {
    let shape = section_shape(sections);
    let mut duplicates: Vec<String> = shape
        .headings
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|((section, heading), _)| format!("  {section} repeats `### {heading}`"))
        .collect();
    duplicates.sort();
    if !duplicates.is_empty() {
        return Err(violation_at(
            "release-coherence#changelog-section-repeats-a-heading",
            format!(
                "a CHANGELOG release section repeats a heading, so entries that belong together are split:\n{}",
                duplicates.join("\n")
            ),
        ));
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
        return Err(violation_at(
            "release-coherence#breaking-without-migration-section",
            format!(
                "a CHANGELOG section marks a change **BREAKING** and carries no `### Migration` section, so what \
             an adopter must do is scattered through the entries or absent:\n{}",
                missing
                    .iter()
                    .map(|s| format!("  {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ));
    }
    Ok(())
}

/// The adopter-facing narrative names none of this repository's own machinery.
fn require_adopter_narrative(
    repo: &Path,
    sections: &[Section],
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    let leaked = adopter_cited_machinery(repo, sections, version, spine.state)?;
    if !leaked.is_empty() {
        return Err(violation_at(
            "release-coherence#adopter-entry-names-own-machinery",
            format!(
                "an adopter-facing CHANGELOG entry names this repository's own machinery, which ships in no \
             package and which an adopter can never run — move it under `### Self-governance`, or, where the \
             adopter-relevant fact is genuinely there, state the guarantee and drop the filename:\n{}",
                leaked.join("\n")
            ),
        ));
    }
    Ok(())
}

/// Judge a repository's release state, returning what to report or why it cannot be judged.
///
/// Read-only: it never bumps, commits, tags, or publishes.
pub fn judge(repo: &Path) -> Result<String, Refusal> {
    if !repo.join("Cargo.toml").is_file() {
        return Err(cannot_judge_at(
            "release-coherence#repository-root-has-no-manifest",
            format!("repository root {} has no Cargo.toml", repo.display()),
        ));
    }
    if !repo.join("CHANGELOG.md").is_file() {
        return Err(cannot_judge_at(
            "release-coherence#repository-root-has-no-changelog",
            format!("repository root {} has no CHANGELOG.md", repo.display()),
        ));
    }
    // The cause travels, for the reason its sibling in `publish_source_gate` records: a machine without git
    // was told the repository has no history.
    git(repo, &["rev-parse", "--is-inside-work-tree"]).map_err(|err| {
        cannot_judge_at("release-coherence#git-unrunnable", match err {
            crate::hermetic_git::Failure::Spawn(why) => format!(
                "git could not be run at all ({why}), so whether {} has a history was never asked",
                repo.display()
            ),
            crate::hermetic_git::Failure::Exit { stderr, .. } => format!(
                "repository root {} has no git history: {stderr}",
                repo.display()
            ),
            crate::hermetic_git::Failure::Unreadable(why) => format!(
                "git answered about {} in bytes this reader cannot represent ({why}), so whether it has a \
                 history was answered and not read",
                repo.display()
            ),
        })
    })?;

    let root_manifest = read(repo, "Cargo.toml")?;
    // Each state is answered separately, and the middle one is why the reader does not collapse them. A value this
    // reader cannot read is not a key that is absent, and it is not a malformed version either: it is legal
    // TOML in a form this reader does not take, and telling an operator their version is *missing* sends them
    // to look for a key that is sitting in front of them.
    let version = match workspace_version(&root_manifest) {
        WorkspaceVersion::Declared(version) => version,
        WorkspaceVersion::Absent => {
            return Err(cannot_judge_at(
                "release-coherence#workspace-version-absent",
                crate::manifest::VERSION_ABSENT,
            ));
        }
        WorkspaceVersion::Unreadable(what) => {
            return Err(cannot_judge_at(
                "release-coherence#workspace-version-unreadable",
                crate::manifest::version_unreadable(
                    &what,
                    "whether every release surface names one version cannot be decided",
                ),
            ));
        }
    };
    let Some(version_parts) = semver(&version) else {
        return Err(cannot_judge_at(
            "release-coherence#workspace-version-malformed",
            crate::manifest::version_malformed(&version),
        ));
    };
    let changelog = read(repo, "CHANGELOG.md")?;
    let changelog_text = changelog.clone();
    // Cut **once**, and hand the value down. Four walks in this file each carried their own section cursor
    // over the same predicate; `sections::cut` owns the boundary question and `section_of` the naming one,
    // which is the split `section_of`'s own doc asks for. Over a `Prose` region, so a fenced `## [` heading
    // is not a section — the misread `region`'s header declares for the readers still below.
    let changelog_source = Source::of(changelog);
    let changelog_sections =
        crate::sections::cut(changelog_source.prose().numbered_lines(), section_of);

    // The phases, in the order a reader meets a refusal in. **The order is observable**: a repository with
    // two problems is refused for whichever phase reaches its own first, and the failure matrix asserts the
    // message. So these are a sequence rather than a set, and moving one moves what gets reported.
    let spine = release_spine(repo, &version, version_parts, &changelog_text)?;
    let members = require_version_surfaces(repo, &root_manifest, &version)?;
    require_changelog_state(
        repo,
        changelog_source.prose(),
        &changelog_sections,
        &members,
        &version,
        &spine,
    )?;
    require_section_shape(&changelog_sections)?;
    require_adopter_narrative(repo, &changelog_sections, &version, &spine)?;

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
        cannot_judge_at("release-coherence#directory-not-enumerable", format!(
            "found no enumerable directory at {}: {err} — the layout changed or is absent, so what it holds \
             cannot be judged",
            dir.display()
        ))
    })?;
    let mut paths = Vec::new();
    for entry in listing {
        let entry = entry.map_err(|err| {
            cannot_judge_at(
                "release-coherence#directory-entry-unreadable",
                format!(
                    "an entry of {} could not be read while enumerating it: {err}",
                    dir.display()
                ),
            )
        })?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

/// Every member manifest this gate reaches, as `(repository-relative path, text)`.
///
/// The set is the directories under `crates/` that carry a `Cargo.toml`, which is a **layout** premise:
/// cargo's own answer is `[workspace] members`. The two agree in this repository today, and
/// `crates/kanhe/tests/member_enumeration.rs` is what asks them rather than assuming it — public for that
/// reader, so the comparison uses this walk rather than restating it and becoming a third enumerator.
pub fn workspace_manifests(repo: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let crates = repo.join("crates");
    let mut out = Vec::new();
    let dirs = entries_of(&crates)?;
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = std::fs::read_to_string(&manifest).map_err(|err| {
                cannot_judge_at(
                    "release-coherence#crate-manifest-unreadable",
                    format!("could not read {manifest:?}: {err}"),
                )
            })?;
            // **Spelled by the one owner, because this is the side `member_enumeration` compares
            // against.** This read `strip_prefix(repo).unwrap_or(&manifest).display()`, which is the
            // host's own separator and, on a failed strip, the absolute path carried forward as if it
            // were relative. The comparison's other side joins components with `/`, so the two sets
            // shared no member wherever that separator is not `/` — and the reader that stands beside
            // this one records the identical defect at its own site.
            match crate::repository_path::repository_path(repo, &manifest) {
                crate::repository_path::RepositoryPath::Below(path) => out.push((path, text)),
                // The manifest is built from `repo` a few lines above, so the strip cannot fail for the
                // input this walk produces. It is answered rather than assumed away because the answer
                // is what the type asks for, and a reader that cannot say where a file sits must not
                // report it as sitting anywhere.
                crate::repository_path::RepositoryPath::Outside => {
                    return Err(cannot_judge_at(
                        "release-coherence#crate-manifest-outside-repository",
                        format!(
                            "crate manifest {} is not under the repository root {} it was walked from",
                            manifest.display(),
                            repo.display()
                        ),
                    ));
                }
                // **This is the arm a filesystem walk can actually reach.** The path comes from
                // `read_dir`, so its components are the bytes the operating system holds rather than a
                // string a parser already validated: a crate directory whose name is not UTF-8 is legal
                // on Unix, and spelling it lossily would hand every reader downstream a name that
                // resolves to nothing — and collapse two distinct names onto one spelling.
                crate::repository_path::RepositoryPath::NotUtf8(component) => {
                    return Err(cannot_judge_at(
                        "release-coherence#crate-directory-not-utf8",
                        format!(
                            "the crate directory holding {} carries a component this reader cannot \
                             represent as text — {component}; a path that is not UTF-8 keeps its own \
                             identity, and reporting a replaced one would name a file this repository \
                             does not hold",
                            manifest.display()
                        ),
                    ));
                }
            }
        }
    }
    if out.is_empty() {
        return Err(cannot_judge_at(
            "release-coherence#no-crate-manifests-found",
            "found no workspace crate manifests under crates/ — the crate layout changed or is absent",
        ));
    }
    Ok(out)
}

/// Which crate a dependency names, or a refusal saying why that cannot be decided.
///
/// **The identity of a dependency, asked once for both pin readers.** `whose` names the manifest for the
/// message — *example adopter*, *the workspace catalog* — and the sites are named for the question rather
/// than for the caller that first asked it, because the same answers decide membership on both sides.
///
/// Which crate a dependency names is its `package` field where it has one, and its key only otherwise. Keying
/// on the name alone was a false negative of the class the Core Contract forbids: cargo renames with
/// `alias = { package = "xuanji", version = "stale" }`, `alias` is in no family, and the entry was skipped
/// entirely — while the aggregate counter stayed non-zero on the strength of the other declarations.
///
/// A key this reader cannot decode names some crate, and which one is exactly what it cannot say — so it can
/// neither be matched against the family nor passed over. Passing over is what it did.
fn dependency_identity(package: Package, key: &str, whose: &str) -> Result<String, Refusal> {
    match package {
        Package::Named(package) => Ok(package),
        Package::Unreadable => Err(cannot_judge_at(
            "release-coherence#dependency-package-value-unreadable",
            format!(
                "{whose} declares `{key}` with a `package` value this check cannot read, so which crate it \
                 names cannot be decided"
            ),
        )),
    }
}

/// Every internal path dependency in the root manifest names the workspace version.
///
/// **One reader, and no loop beside it.** This held its own line-oriented scan while the sibling that judges
/// example pins was migrated to `declared_dependencies` in the same window, so the two disagreed
/// observably: the new reader knows the detailed table cargo writes and the old loop did not. Against
/// `[workspace.dependencies.xuanji]` with `path` and `version` on their own lines, the loop selected the
/// **path** line — it carries `path`, `"crates/` and `=` — split it at its `=`, and took `path` for the
/// dependency's name, while the `version` line carrying neither marker was never read. The result was
/// *internal dependency path has no version pin*: a false refusal in front of the release gate, over a
/// manifest cargo reads correctly. Which dependencies exist is one question, and it now has one answer.
///
/// The selection is the dependency's own `path` value rather than the shape of the line it sits on, which is
/// the same correction the sibling made when it stopped keying on the dependency's name.
pub(crate) fn require_internal_pins(
    root_manifest: &str,
    version: &str,
    members: &[Member],
) -> Result<(), Refusal> {
    let mut pins = 0usize;
    for Dependency {
        key,
        package,
        pin,
        path,
    } in declared_dependencies(root_manifest, Subject::Requires)?
        .into_iter()
        .chain(declared_dependencies(root_manifest, Subject::Offers)?)
    {
        // **Which crate this names decides membership; where it points is then a requirement.** The selection
        // was the dependency's `path`, and an earlier note here called the asymmetry with
        // `require_example_pins` earned — identity for one reader, path for the other. It was not. A family
        // crate offered with no `path` at all resolves from the **registry**: measured under cargo 1.96.0, a
        // catalog entry `xuanji = "0.4.0"` beside a local member `xuanji 0.9.0` gives the inheriting member
        // `registry+…#xuanji@0.4.0`, and the local member sits unused. Every such requirement is published
        // verbatim — `cargo package` on a `git` dependency carrying a `version` drops the source and records
        // the version alone, measured the same way. So a stale family requirement reached `cargo publish`
        // through a line the subject never contained, and dropping one `path = …` is the whole of the edit
        // that gets there. Two readers of one question is what made it invisible; there is one now.
        let package = dependency_identity(package, &key, "the workspace catalog")?;
        let Some(member) = members.iter().find(|member| member.name == package) else {
            continue;
        };
        pins += 1;
        // A family crate is a member of this workspace, so the catalog holds it by a path to **that member's
        // own directory**. Anything else — a registry requirement, a `git` source, a path to somewhere else,
        // a path to a different member — is another crate that happens to share the name, and members
        // inheriting it build against that one.
        //
        // **Compared against the member, not against a prefix.** `starts_with("crates/")` was a spelling
        // test standing in for a location, and two reviews falsified it in opposite directions in one round:
        // `./crates/xuanji` names the member and was refused at exit 1, `crates/../vendor/xuanji` resolves
        // to `vendor/xuanji` and passed. Both measured under cargo 1.96.0 through `cargo metadata`. The
        // question was never *does this text begin with `crates/`* but *is this the directory this member
        // lives in*, and the member is what answers it.
        match path {
            Declared::Value(path) => match normalized_directory(&path) {
                Ok(directory) if directory == member.directory => {}
                Ok(directory) => {
                    return Err(violation_at(
                        "release-coherence#internal-path-names-another-directory",
                        format!(
                            "internal dependency {key} is offered from {path}, which names {}, and this \
                     workspace's {package} is at {}",
                            directory.display(),
                            member.directory.display()
                        ),
                    ));
                }
                // **Each reason says which one it is.** One message enumerating the others sends an operator
                // to look for a `..` that is not there.
                Err(reason) => {
                    return Err(cannot_judge_at(
                        "release-coherence#internal-path-unresolvable",
                        format!(
                            "internal dependency {key} is offered from {path}, which {}, so whether members \
                     inherit this workspace's {package} cannot be decided",
                            match reason {
                                Unresolvable::Absolute =>
                                    "is absolute — this reader compares repository-relative directories and \
                             is handed no repository to make one relative against",
                                Unresolvable::Traversal =>
                                    "carries a `..` segment — applied after symlink resolution, which this \
                             reader touches no filesystem to follow",
                                Unresolvable::NamesNoDirectory =>
                                    "names no directory beneath the manifest's own — cargo refuses such a \
                             dependency outright, measured",
                            }
                        ),
                    ));
                }
            },
            Declared::Absent => {
                return Err(violation_at(
                    "release-coherence#internal-path-absent",
                    format!(
                        "internal dependency {key} names the family crate {package} with no `path`, so \
                     members inherit the registry crate rather than this workspace's"
                    ),
                ));
            }
            // An inherited dependency declares no `path` of its own — and this is the manifest that declares
            // the catalog, so the pin arm below refuses it as undecidable rather than reading it here.
            Declared::Inherited => {}
            Declared::Unreadable(written) => {
                return Err(cannot_judge_at(
                    "release-coherence#dependency-path-unreadable",
                    format!(
                        "dependency {key} declares a `path` this check cannot read ({written}), so whether \
                     members inherit this workspace's {package} cannot be decided"
                    ),
                ));
            }
        }
        match pin {
            Declared::Value(pin) if pin == version => {}
            Declared::Value(pin) => {
                return Err(violation_at(
                    "release-coherence#internal-pin-disagrees",
                    format!("internal dependency {key} is pinned to {pin}; expected {version}"),
                ));
            }
            Declared::Absent => {
                return Err(violation_at(
                    "release-coherence#internal-pin-absent",
                    format!("internal dependency {key} has no version pin"),
                ));
            }
            // The root manifest **is** the workspace, so a dependency here taking `workspace = true` would be
            // inheriting from itself — measured, `cargo metadata` refuses a manifest whose catalog does not
            // declare what it inherits, and a catalog inheriting from itself declares nothing. Refused as
            // undecidable rather than guessed at, in the direction that stops in front of an operator.
            Declared::Inherited => {
                return Err(cannot_judge_at(
                    "release-coherence#internal-pin-inherited",
                    format!(
                        "internal dependency {key} takes its version from the workspace catalog, and this is \
                     the manifest that declares the catalog, so what holds it cannot be decided"
                    ),
                ));
            }
            Declared::Unreadable(written) => {
                return Err(cannot_judge_at(
                    "release-coherence#internal-pin-unreadable",
                    format!(
                        "internal dependency {key} declares a version this check cannot read ({written}), so \
                     whether it names the workspace version cannot be decided"
                    ),
                ));
            }
        }
    }
    // **Already per document, which is why this counter stays where it is.** A review read it as an aggregate
    // over every crate and asked for the treatment `require_example_pins` got — but that function walks a
    // directory of examples and this one reads the workspace ROOT manifest alone, so its loop is over one
    // document's `[workspace.dependencies]` entries. The granularity a partial read would need is already the
    // granularity it has: nothing else's success can keep this count non-zero.
    if pins == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-internal-family-dependency-found",
            "found no dependency on a family crate in Cargo.toml — the declaration form changed, so pin \
             coherence would be reported over nothing",
        ));
    }
    Ok(())
}

/// A workspace member, by the name its manifest declares.
///
/// **A distinct type because the swap was reachable and then happened.** This list and the `(path, text)`
/// manifests were both `Vec<(String, String)>`, so passing one where the other belonged compiled: measured
/// once when a proposed refactor was reverted after the lock reader reported *Cargo.lock is missing workspace
/// package* with a whole manifest where a name belongs, and again when a unit direction handed the manifests
/// to a reader expecting members and got a vacuity refusal instead of the one it observes. `BACKLOG.md` filed
/// the shape with the trigger *the next edit to this sequence, or a third consumer of either list*; the
/// identity selection below is the third consumer.
pub(crate) struct Member {
    /// The `[package]` name the manifest declares.
    pub(crate) name: String,
    /// The member's directory, repository-relative, as its manifest path names it.
    ///
    /// **Carried because the catalog's `path` has to be compared against something.** It was dropped one
    /// change earlier as state nothing read — true then, and the reason it stopped being true is that the
    /// pin reader was still answering *is this path into this workspace* with `starts_with("crates/")`. Two
    /// reviews falsified that in opposite directions: `./crates/xuanji` names the member and was refused,
    /// and `crates/../vendor/xuanji` resolves outside and passed. Both measured under cargo 1.96.0. A
    /// prefix cannot decide either, and `crates/<name>` cannot be derived — this repository's own fixture
    /// holds `machinery-under-another-name` under `crates/renamed-dir`, and the machinery reader records
    /// deriving a directory from a package name as a defect it already fixed.
    pub(crate) directory: PathBuf,
}

/// Why this reader will not name the directory a `path` value points at.
///
/// **Each reason typed apart, not one `None`.** The first version answered every one of them with `None` and the caller's
/// message enumerated two of them, so `path = "."` was told it was *absolute or carries a `..` segment* —
/// neither, and exactly the misdirection this crate's typed readers exist to prevent. Every other pair of
/// facts in this file is typed apart for the same reason.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Unresolvable {
    /// A root or a drive prefix. Repository-relative is the only thing this reader compares, and it is handed
    /// no repository to make one relative against.
    Absolute,
    /// A `..` segment. The kernel applies it **after** symlink resolution, so `crates/../vendor` is `vendor`
    /// only while `crates` is not a link, and this reader touches no filesystem to find out.
    Traversal,
    /// Every component was `.` or a separator, so the value names the directory the manifest is already in
    /// rather than one beneath it. Measured under cargo 1.96.0: `path = "."` fails resolution with *failed to
    /// get `xuanji` as a dependency* — a manifest nothing builds, which this reader stops in front of rather
    /// than reading a pin past.
    NamesNoDirectory,
}

/// The repository-relative directory a `path` value names, or why this reader will not name it.
///
/// **Read through [`std::path::Component`], which is also how the member side is built.** A `.` component,
/// a repeated separator and a trailing separator are spelling: cargo resolves `crates/xuanji`,
/// `./crates/xuanji`, `crates//xuanji` and `crates/xuanji/` to one directory, measured, and `Components`
/// drops exactly those. Hand-splitting on `/` did the same thing on this repository's CI and only there — the
/// member side comes from a `Path`, which renders `crates{sep}xuanji` with the **platform's** separator,
/// while a manifest always spells `/`. On Windows the two operands then disagreed about every ordinary path,
/// and a drive-qualified path read as an ordinary relative one. Comparing `PathBuf`s built from components
/// gives both sides one representation on every platform, because `Path` equality is component-wise and
/// Windows accepts both separators.
///
/// **A coverage limitation, not a bound: `Component::Prefix` is compiled and unexercised here.** It is
/// produced only on Windows, and this repository's CI is Ubuntu. The arm *reacts*, and correctly — a drive
/// prefix is rooted — so there is nothing the reaction declines to observe and nothing for the bound register
/// to hold; what is missing is a run, not a verdict. It shares its answer with `RootDir`, which the absolute
/// row does exercise. An earlier wording here called it a *declared bound*, which is this repository's term
/// of art for a shape the register holds, and the register held no such entry.
pub(crate) fn normalized_directory(path: &str) -> Result<PathBuf, Unresolvable> {
    let mut directory = PathBuf::new();
    let mut named = false;
    for component in Path::new(path).components() {
        match component {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                return Err(Unresolvable::Absolute);
            }
            std::path::Component::ParentDir => return Err(Unresolvable::Traversal),
            std::path::Component::CurDir => {}
            std::path::Component::Normal(segment) => {
                directory.push(segment);
                named = true;
            }
        }
    }
    named
        .then_some(directory)
        .ok_or(Unresolvable::NamesNoDirectory)
}

/// Each workspace manifest's member — the family, resolved once for everything that asks which crates it
/// holds.
///
/// A manifest whose package cannot be named is not a crate a reader may quietly skip: it would drop out of
/// the family, and every declaration requiring it would then pass the membership filter without being
/// examined. The vacuity guards downstream are aggregate, so every crate but one parsing keeps them silent
/// while that one goes unchecked — which is the partial case a vacuity guard is exactly unable to see.
///
/// **Resolved at the caller rather than inside one consumer.** It was the first thing `require_example_pins`
/// did, and it returned the list for the lock reader; the pin reader for the workspace catalog needs the same
/// list and ran *before* it, which is how that reader came to select its subject by `path` instead. One
/// resolution, three consumers, and no reader owning a list on another's behalf.
pub(crate) fn family_members(manifests: &[(String, String)]) -> Result<Vec<Member>, Refusal> {
    let mut members: Vec<Member> = Vec::new();
    for (path, text) in manifests {
        match package_name(text) {
            PackageName::Named(name) => members.push(Member {
                name,
                // **The member's own directory, as a `Path`.** Built by `Path::parent` rather than by
                // trimming text, so it is component-wise comparable with the value read from a manifest on
                // every platform — the two operands were a `Path`-rendered string and a `/`-split string,
                // which agree only where the separator does.
                directory: Path::new(path)
                    .parent()
                    .unwrap_or(Path::new(""))
                    .to_path_buf(),
            }),
            PackageName::Absent => {
                return Err(cannot_judge_at(
                    "release-coherence#crate-package-name-absent",
                    format!(
                        "{path} declares no `[package]` name, so which crates the family holds cannot be \
                     decided"
                    ),
                ));
            }
            PackageName::Unreadable(what) => {
                return Err(cannot_judge_at(
                    "release-coherence#crate-package-name-unreadable",
                    format!(
                        "{path} declares a `[package]` name this check cannot read ({what}), so which crates \
                     the family holds cannot be decided"
                    ),
                ));
            }
        }
    }
    Ok(members)
}

pub(crate) fn require_example_pins(
    repo: &Path,
    members: &[Member],
    version: &str,
) -> Result<(), Refusal> {
    let family: Vec<String> = members.iter().map(|member| member.name.clone()).collect();
    let minor = version
        .rsplit_once('.')
        .map(|(head, _)| head)
        .unwrap_or(version);
    let mut example_manifests = 0usize;

    let dirs = entries_of(&repo.join("examples"))?;
    for dir in dirs {
        // An example is a **directory**, and `examples/` holds files of its own — a README among them. The
        // entry that is not a directory holds no example, which is a different fact from a directory whose
        // manifest cannot be read, and it is the one this loop may pass over.
        if !dir.is_dir() {
            continue;
        }
        let manifest = dir.join("Cargo.toml");
        // Absent is not unreadable. Skipping both alike let the remaining readable examples satisfy the
        // counters below, so the judgement reported clean over the very manifest it could not read.
        //
        // `is_file()` answered both with one `false`: a directory named `Cargo.toml`, or a path that exists
        // and is not a regular file, read as *no example here*. Asking for the metadata separates them —
        // `NotFound` is the absence this loop may skip, and anything else is a fact to report.
        //
        // One construction, and the message carries which of the two it met: the register holds a site
        // identity to exactly one branch, so two arms reaching two calls would be one identity vouching for a
        // branch no direction reached.
        let present = match std::fs::metadata(&manifest) {
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => continue,
            Err(err) => Err(err.to_string()),
            Ok(found) if found.is_file() => Ok(()),
            Ok(_) => Err("it is there and is not a regular file".to_string()),
        };
        if let Err(why) = present {
            return Err(cannot_judge_at(
                "release-coherence#example-manifest-not-a-readable-file",
                format!(
                    "the example manifest {} is not one this check can read — {why}, which is not the same \
                     fact as an example that declares none",
                    manifest.display()
                ),
            ));
        }
        let text = std::fs::read_to_string(&manifest).map_err(|err| {
            cannot_judge_at(
                "release-coherence#example-manifest-unreadable",
                format!(
                    "could not read the example manifest {}: {err}",
                    manifest.display()
                ),
            )
        })?;
        example_manifests += 1;
        // **Counted per example, because the aggregate could not see a partial read.** This counter used to
        // live outside the loop, so seven examples parsing kept it non-zero while an eighth went unexamined
        // — the partial case this function's own header names, and the half that let a renamed and then a
        // quoted family key each reach a release as clean. One example is one subject: whatever the reader
        // failed to see there is invisible to every other example's success.
        let mut requirements_here = 0usize;
        let name = dir
            .file_name()
            .expect("a `read_dir` entry always has a file name")
            .to_string_lossy()
            .into_owned();
        // The catalog this example's inherited pins resolve against, read **once** for the whole manifest.
        // Resolving inside the loop parsed the same document per inherited dependency, and gave the search a
        // parse failure to answer — a fact its `Missing` state then carried alongside *no entry names it*.
        // Parsed here, the refusal belongs to the caller that met it and the search answers one question.
        let catalog = declared_dependencies(&text, Subject::Offers)?;
        // Executed text, for the reason `require_internal_pins` records: a commented-out family pin
        // would otherwise be read as a declared one.
        for Dependency {
            key,
            package,
            pin,
            path: _,
        } in declared_dependencies(&text, Subject::Requires)?
        {
            // **A dependency that takes the offer is resolved before it is identified, because its key is a
            // lookup key rather than a name.** The identity rule above holds for a dependency that declares
            // itself; cargo applies neither half of it to one spelling `workspace = true`. Measured under
            // cargo 1.96.0, a `package` beside `workspace = true` is accepted and ignored with an
            // `unused manifest key` warning, and inheritance spelled under the crate's name rather than the
            // catalog's key is refused outright -- so the only lookup is the
            // dependency's key against a catalog key, and the crate is whatever that entry names. Deciding
            // membership on the local key first passed over every dependency the catalog renames, and a
            // stale pin behind one reached a release as clean: the example was then reported as declaring no
            // family requirement, which is a different fact about a different manifest.
            //
            // The offer is resolved before the pin arms below, so every way of failing to read a pin keeps
            // one home. A dependency taking `workspace = true` declares no `version` of its own, and the
            // reader filed that as `Absent` -- the state meaning *nothing holds this to a version* -- so an
            // example whose pin is held exactly was refused for having none. Cargo holds it to the catalog's
            // requirement, measured; the catalog is read here and its pin is judged as if written inline.
            let (package, pin) = match pin {
                Declared::Inherited => match offered(&catalog, &key) {
                    Offered::Entry { package, pin } => (package, pin),
                    // Answered before membership, not after it: the entry taken names a crate this reader
                    // cannot read, so whether it is a family crate is exactly what cannot be decided.
                    Offered::Unresolvable(entry) => {
                        return Err(cannot_judge_at(
                            "release-coherence#example-catalog-entry-unresolvable",
                            format!(
                                "example {name} takes the workspace catalog's offer under `{key}`, whose \
                             entry {entry} names a crate this check cannot resolve, so what holds it cannot \
                             be decided"
                            ),
                        ));
                    }
                    // Nothing is written under that key, so the catalog renames nothing here and the local
                    // key is the only identity there is. Where it names no family crate this dependency is
                    // not this check's subject -- the same answer one declaring itself would get.
                    Offered::Missing => {
                        let package =
                            dependency_identity(package, &key, &format!("example {name}"))?;
                        if !family.contains(&package) {
                            continue;
                        }
                        return Err(cannot_judge_at(
                            "release-coherence#example-inherits-what-no-catalog-offers",
                            format!(
                                "example {name} requires {package} from the workspace catalog, and no \
                             `[workspace.dependencies]` entry beside it is written under `{key}`, so what \
                             holds it cannot be decided"
                            ),
                        ));
                    }
                },
                // **Which crate a dependency names is its `package` field where it has one, and its key only
                // otherwise** — asked of [`dependency_identity`], which the root's pin reader asks too. This
                // reader resolved identity because an example carries no path, and the sibling selected on
                // path and called the asymmetry earned; a family crate the catalog offers *without* a path
                // is what that cost. One question, one reader.
                declared => (
                    dependency_identity(package, &key, &format!("example {name}"))?,
                    declared,
                ),
            };
            if !family.contains(&package) {
                continue;
            }
            let pin = match pin {
                Declared::Value(pin) => pin,
                Declared::Absent => {
                    return Err(violation_at(
                        "release-coherence#example-pin-absent",
                        format!(
                            "example {name} requires {package} with no version, so nothing holds it to the \
                         workspace version {version}"
                        ),
                    ));
                }
                Declared::Unreadable(written) => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-pin-unreadable",
                        format!(
                            "example {name} requires {package} with a version this check cannot read \
                         ({written}), so whether it satisfies the workspace version cannot be decided"
                        ),
                    ));
                }
                // Reached when the catalog entry resolved above **itself** takes the offer: a catalog
                // inheriting from itself, which cargo refuses to parse. Named rather than followed, because
                // following it is a loop with no end that a manifest could not have built anyway.
                Declared::Inherited => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-catalog-entry-inherits",
                        format!(
                            "example {name} requires {package} from the workspace catalog, whose own entry \
                         takes its version from the catalog, so what holds it cannot be decided"
                        ),
                    ));
                }
            };
            requirements_here += 1;
            if pin != minor && pin != version {
                // The package, and the key where they differ: a renamed dependency reported by its key alone
                // sends a reader looking for a crate the manifest does not name.
                let named = if package == key {
                    package.clone()
                } else {
                    format!("{package} (as `{key}`)")
                };
                return Err(violation_at(
                    "release-coherence#example-pin-disagrees",
                    // **What was measured, not what a reader might infer.** The rule is string equality
                    // against the two spellings the release surfaces are held to; it does not evaluate a
                    // semver requirement. So `= "^0.5"`, which `0.5.0` genuinely satisfies, is refused —
                    // correctly by the rule and falsely by a sentence saying it is not satisfied, which
                    // sends a maintainer to check semver instead of changing the spelling.
                    format!(
                        "example {name} requires {named} = \"{pin}\"; this check admits only \"{version}\" \
                     or \"{minor}\", the two spellings the release surfaces are held to"
                    ),
                ));
            }
        }
        if requirements_here == 0 {
            return Err(cannot_judge_at(
                "release-coherence#example-requires-no-family-crate",
                format!(
                    "example {name} declares no family dependency requirement this check could read, so its \
                 pins would be reported over nothing. Either it requires no family crate — which is not an \
                 example of this family — or it declares one in a form this reader did not see"
                ),
            ));
        }
    }
    if example_manifests == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-example-manifests-found",
            "found no example manifests under examples/ — the layout changed or is absent",
        ));
    }
    // **The aggregate guard is gone rather than kept beside this one, because no input can reach it.** With
    // every example refusing on its own zero, a run that gets past the loop has `example_manifests` examples
    // each contributing at least one requirement; a run with none is the guard above. Keeping it would be the
    // dead branch this file already refuses one read earlier — *a branch no input can take, which is dead code
    // rather than a guard*. Its WHEN moved rather than vanished: the fixture that reached it, one example
    // requiring no family crate, now reaches the per-example refusal, and the direction that pinned it is
    // rewritten onto the new site rather than deleted.
    Ok(())
}

/// **Names resolved once, by the reader that already had to resolve them.** This re-read every manifest's
/// `[package]` name and carried its own refusals for a name that is absent or unreadable — branches no input
/// could reach, because `manifests` exists only if the example-pin reader resolved every one of those names
/// first and refused otherwise. Two readers asking one question of one input is the shape this file has
/// spent the window removing; the dead branches were what it looked like from inside.
fn require_lock_versions(repo: &Path, members: &[Member], version: &str) -> Result<(), Refusal> {
    let lock = read(repo, "Cargo.lock")?;
    let doc = lock.parse::<toml_edit::DocumentMut>().map_err(|err| {
        cannot_judge_at(
            "release-coherence#lock-unreadable",
            format!(
                "Cargo.lock is not a lock file this parser can read — {}",
                crate::manifest::parse_error_on_one_line(&err)
            ),
        )
    })?;

    // **Every entry under a name, and whether each carries a `source`.** Two entries under one name is
    // ordinary in a lock, either as two versions of one crate or as a workspace member sharing a name with
    // something from a registry, so a single-valued map keyed on the name would keep whichever came first.
    // `source` is what tells them apart: a workspace member has none, everything fetched has one.
    //
    // **The block boundary comes from the parser now.** It was the literal string `[[package]]` and an
    // ordering premise beneath it — `source` is written after `version` in cargo's own output, so filing an
    // entry early recorded every one as source-less. An array of tables has neither question: each element
    // *is* one entry, and the order its keys were written in is not a fact this reader has to know.
    let mut entries: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    for entry in doc
        .get("package")
        .and_then(toml_edit::Item::as_array_of_tables)
        .into_iter()
        .flatten()
    {
        let Some(name) = entry.get("name") else {
            continue;
        };
        let Some(name) = name.as_str() else {
            return Err(cannot_judge_at(
                "release-coherence#lock-package-name-unreadable",
                format!(
                    "Cargo.lock carries a package name this check cannot read ({}), so the versions it \
                     records cannot be compared",
                    name.to_string().trim()
                ),
            ));
        };
        let Some(found) = entry.get("version") else {
            continue;
        };
        let Some(found) = found.as_str() else {
            return Err(cannot_judge_at(
                "release-coherence#lock-version-unreadable",
                format!(
                    "Cargo.lock records a version for {name} that this check cannot read ({}), so whether \
                     it matches the workspace cannot be decided",
                    found.to_string().trim()
                ),
            ));
        };
        entries
            .entry(name.to_string())
            .or_default()
            .push((found.to_string(), entry.get("source").is_some()));
    }

    for Member { name: package, .. } in members {
        let package = package.clone();
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
            return Err(cannot_judge_at(
                "release-coherence#lock-several-sourceless-entries",
                format!(
                    "Cargo.lock carries {} entries for {package} with no source, so which one is the workspace \
                 member is not decided",
                    extra + 1
                ),
            ));
        }
        match first {
            None => {
                return Err(violation_at(
                    "release-coherence#lock-missing-workspace-package",
                    format!("Cargo.lock is missing workspace package {package}"),
                ));
            }
            Some(found) if found != version => {
                return Err(violation_at(
                    "release-coherence#lock-package-version-disagrees",
                    format!("Cargo.lock package {package} is {found}; expected {version}"),
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Whether the `[Unreleased]` section carries an adopter-facing item.
///
/// **The boundary question is the cut's now.** This ran an `inside: bool` that opened on the sentinel and
/// closed on the next `## [` — the same shape `wrapper_parser::parser_arms` was repaired for, correct here
/// only because Markdown headings do not nest. `section_of`'s doc left it alone deliberately, since folding a
/// boundary into a naming function makes one function answer two; `sections::cut` answers the boundary half
/// instead, so nothing here decides where a section ends.
///
/// `any` over the matching sections rather than the first of them: the caller refuses a changelog with more
/// than one before reaching here, so the two agree — and reading *the first* would be a choice this function
/// has no reason to make.
fn unreleased_has_item(sections: &[Section]) -> bool {
    sections
        .iter()
        .filter(|section| section.name == "## [Unreleased]")
        .any(|section| {
            section.body.iter().any(|(_, line)| {
                let trimmed = line.trim_start();
                trimmed.starts_with("- ") || trimmed.starts_with("* ")
            })
        })
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
        line.split_once(" - ")
            .map_or(line, |(section, _)| section)
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
fn section_shape(sections: &[Section]) -> Shape {
    let mut shape = Shape {
        headings: BTreeMap::new(),
        breaking: BTreeSet::new(),
    };
    // The section heading itself is not in `body`, so the arms below cannot see it — which the cursor form
    // had to arrange with a `continue` that stood on its own and could be deleted without anything noticing.
    for section in sections {
        for (_, line) in &section.body {
            if let Some(heading) = line.strip_prefix("### ") {
                *shape
                    .headings
                    .entry((section.name.clone(), heading.trim_end().to_string()))
                    .or_default() += 1;
            }
            if line.contains("**BREAKING**") {
                shape.breaking.insert(section.name.clone());
            }
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
    // `..` components intact — so stripping `repo.display()` failed for **every** member, machinery
    // collapsed to the two `scripts/` files, `published` stayed empty, and two `continue`s made it silent.
    // `workspace_root` is cargo's own answer for the tree it just described, so the two strings cannot
    // disagree about spelling.
    let Some(root) = metadata["workspace_root"].as_str() else {
        return Err(cannot_judge_at(
            "release-coherence#metadata-has-no-workspace-root",
            "cargo metadata reported no workspace_root, so no member directory can be resolved",
        ));
    };
    let root = Path::new(root);
    let mut machinery: Vec<String> = Vec::new();
    let mut published: BTreeSet<String> = BTreeSet::new();
    let mut enumerated = 0usize;
    // **The members the machinery set is drawn from, kept as a set rather than counted.** The floor below
    // guarded this subject with `enumerated`, which counts every member's tracked paths — published ones
    // included — while `machinery` is filled from the unpublished branch alone. One tracked file under any
    // published crate therefore kept the counter non-zero while the machinery set was `scripts/` alone,
    // which is the state that floor's own message describes. Third time this file has written a guard over
    // a wider set than the one it protects, so what is counted is now what is guarded, and the two facts
    // are typed apart below.
    let mut unpublished_members: Vec<String> = Vec::new();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        // **The directory comes from the manifest, not from the package name.** Deriving it as
        // `crates/<name>/` was the residual location assumption inside a repair whose own thesis was
        // *produced from the manifests, not from a location*: a member whose directory differs from its
        // package name contributes to neither set, so it is machinery nothing refuses (silent), or published
        // source whose basenames then enter the machinery set and refuse honest adopter prose.
        // `cargo metadata` answers this exactly — `manifest_path` is the member's own `Cargo.toml`.
        let Some(manifest) = package["manifest_path"].as_str() else {
            return Err(cannot_judge_at(
                "release-coherence#metadata-package-has-no-manifest-path",
                "a package in cargo metadata carries no manifest_path, so its directory cannot be resolved",
            ));
        };
        // **Spelled by the one owner**, which is where the component-wise strip and the `/` join now live
        // for every reader that asks this question — see [`crate::repository_path`] for what the three
        // hand-written spellings cost. What is this site's own is the *directory*: the member's own
        // `Cargo.toml` is what cargo reports, and the tracked files under it are what this gate enumerates.
        // **Matched rather than pattern-bound**, because a `let … else` reads every answer that is not the
        // one it wants as the one fact its `else` names. When the owner gained a third state this site
        // absorbed it into *not under the workspace root*, which is a different repair in a different place
        // — the shape the owner's own type exists to stop.
        let manifest_path = Path::new(manifest);
        let outside = || {
            // `--no-deps` lists workspace members only, so every manifest sits under the root cargo reported
            // alongside them. One that does not is this gate's two sources describing different trees, which
            // is a fact to report rather than a member to skip — skipping is what kept the collapse silent.
            cannot_judge_at(
                "release-coherence#member-manifest-outside-workspace-root",
                format!(
                    "member manifest {manifest} is not under the workspace root {} cargo reported for it",
                    root.display()
                ),
            )
        };
        let Some(spelled) = manifest_path
            .parent()
            .map(|dir| crate::repository_path::repository_path(root, dir))
        else {
            return Err(outside());
        };
        let directory = match spelled {
            crate::repository_path::RepositoryPath::Below(directory) => directory,
            crate::repository_path::RepositoryPath::Outside => return Err(outside()),
            // `manifest` is a `&str` the JSON parser handed over, so its components are UTF-8 before this
            // reader sees them and no path built from it can reach here. It is answered rather than folded
            // into the arm above because the two are repaired in opposite directions, and because the fold
            // is what this site had.
            crate::repository_path::RepositoryPath::NotUtf8(component) => {
                return Err(cannot_judge_at(
                    "release-coherence#member-directory-not-utf8",
                    format!(
                        "member manifest {manifest} sits under a component this reader cannot represent as \
                         text — {component}; a path that is not UTF-8 keeps its own identity, and judging a \
                         replaced one would compare something the repository does not hold"
                    ),
                ));
            }
        };
        let directory = if directory.is_empty() {
            String::new()
        } else {
            format!("{directory}/")
        };
        let unpublished = package["publish"].as_array().is_some_and(|r| r.is_empty());
        if unpublished {
            unpublished_members.push(directory.trim_end_matches('/').to_string());
        }
        // **`-z`, because git quotes a path it cannot write plainly.** `core.quotePath` defaults on and
        // `hermetic()` neutralises the config that could turn it off, so a tracked path carrying non-ASCII
        // bytes enters this set in its ESCAPED spelling and its real name is absent — after which
        // `adopter_cited_machinery` cannot recognise a record citing that file, a false negative in the
        // release gate. Latent today (no tracked path needs quoting) and the sibling capability already
        // raises the class to a SHALL, which is why it is closed rather than declared.
        let listing = git(repo, &["ls-files", "-z", &directory]).map_err(|err| {
            cannot_judge_at(
                "release-coherence#directory-listing-unreadable",
                format!("could not enumerate {directory}: {err}"),
            )
        })?;
        for path in listing.split('\0').filter(|l| !l.is_empty()) {
            enumerated += 1;
            if unpublished {
                machinery.push(path.to_string());
            } else {
                published.insert(
                    path.rsplit_once('/')
                        .map_or(path, |(_, base)| base)
                        .to_string(),
                );
                let mut dir = path.to_string();
                while let Some(cut) = dir.rfind('/') {
                    dir.truncate(cut + 1);
                    published.insert(dir.clone());
                    dir.truncate(cut);
                }
            }
        }
    }
    // **Two facts, two refusals.** Members resolved and *nothing at all* enumerated means the directories were
    // resolved against a root this repository's git does not share — cargo and git describing different trees.
    if enumerated == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-tracked-file-for-any-member",
            format!(
                "no tracked file was found for any of the {} workspace members under {}, so cargo and git \
             are describing different trees",
                metadata["packages"].as_array().map_or(0, Vec::len),
                root.display()
            ),
        ));
    }
    // The subject's own floor. A workspace declaring unpublished members none of which contributed a tracked
    // file leaves the machinery set as `scripts/` alone, and this check would then pass over its own subject.
    // A workspace with **no** unpublished members legitimately has `scripts/` alone, which is why the
    // condition is *declared and contributed nothing* rather than *empty*.
    if !unpublished_members.is_empty() && machinery.is_empty() {
        return Err(cannot_judge_at(
            "release-coherence#no-machinery-from-unpublished-members",
            format!(
                "the unpublished members ({}) contributed no tracked file, so the machinery set would be \
             `scripts/` alone and this check would pass over its own subject",
                unpublished_members.join(", ")
            ),
        ));
    }
    let scripts = git(repo, &["ls-files", "-z", "scripts/"]).map_err(|err| {
        cannot_judge_at(
            "release-coherence#scripts-not-enumerable",
            format!("could not enumerate scripts/: {err}"),
        )
    })?;
    machinery.extend(
        scripts
            .split('\0')
            .filter(|l| !l.is_empty())
            .map(str::to_string),
    );

    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in &machinery {
        names.insert(path.clone());
        let base = path
            .rsplit_once('/')
            .map_or(path.as_str(), |(_, base)| base);
        // Unique across the tree, or it names a published crate's file as well and would refuse an
        // entry that is about the product rather than about the machinery.
        if !published.contains(base) {
            names.insert(base.to_string());
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

/// The workspace as cargo reports it, `--no-deps`.
///
/// `pub` so a direction outside this module can hold a text reader against cargo's own answer, which is what
/// `repository-checks`'s *one fact about a manifest has one reader* asks for: two deliberate readers of one
/// fact need a reaction between them, or the second encodes a belief about the first.
pub fn cargo_metadata(repo: &Path) -> Result<serde_json::Value, Refusal> {
    let out = std::process::Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo)
        .output()
        .map_err(|err| {
            cannot_judge_at(
                "release-coherence#cargo-metadata-unrunnable",
                format!("could not run cargo metadata: {err}"),
            )
        })?;
    if !out.status.success() {
        return Err(cannot_judge_at(
            "release-coherence#cargo-metadata-failed",
            format!(
                "cargo metadata failed for {}: {}",
                repo.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|err| {
        cannot_judge_at(
            "release-coherence#cargo-metadata-not-json",
            format!("cargo metadata is not JSON: {err}"),
        )
    })
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
    sections: &[Section],
    version: &str,
    state: State,
) -> Result<Vec<String>, Refusal> {
    // One enumeration. A second copy lived here for one commit, built for a census that was dropped, and
    // two constructions of one set is the drift this file's own doc-comment says it exists to prevent.
    let names = machinery_names(repo)?;

    let mut found: BTreeSet<String> = BTreeSet::new();
    // `heading` resets at each section by construction now. The cursor form cleared it by hand beside the
    // section assignment, which is one statement holding a structural fact — delete it and headings leak
    // across a section boundary with nothing to say so.
    for section in sections {
        let mut heading = String::new();
        for (_, line) in &section.body {
            if let Some(next) = line.strip_prefix("### ") {
                heading = next.trim_end().to_string();
            }
            let being_written = matches!(state, State::ReleaseReady | State::Snapshot)
                && section.name == format!("## [{version}]");
            if (section.name != "## [Unreleased]" && !being_written) || heading == "Self-governance"
            {
                continue;
            }
            for run in line
                .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')))
            {
                // The token is compared as written before any punctuation is taken off it. `trim_end_matches`
                // stripped **every** trailing dot, so a path legitimately ending in one was rewritten before
                // it could match — identity normalised to suit a sentence. A Markdown sentence ends in one
                // period, so one is what comes off, and only where the name as written matches nothing.
                let written = run.strip_prefix("./").unwrap_or(run);
                let token = if names.contains(written) {
                    written
                } else {
                    written.strip_suffix('.').unwrap_or(written)
                };
                if token.is_empty() {
                    continue;
                }
                if names.contains(token) {
                    found.insert(format!(
                        "  {} under `### {}` names {token}",
                        section.name,
                        if heading.is_empty() {
                            "(no heading)"
                        } else {
                            &heading
                        }
                    ));
                }
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
    // The same day the fixture's commits carry, from the one owner — this section and those commits are the
    // two halves `release-coherence` compares.
    let day = crate::hermetic_git::FIXTURE_DAY;
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n## [{version}] - {day}\n\n- Release notes.\n\n\
             [Unreleased]: {COMPARE}/v{version}...HEAD\n[{version}]: {COMPARE}/v{previous}...v{version}\n"
        ),
    );
}

/// A repository released at `version` over a `0.1.0` predecessor. Prints its path.
pub fn build_fixture(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    run(
        &repo,
        "git",
        &["config", "user.name", "Release Coherence Test"],
    );
    run(
        &repo,
        "git",
        &["config", "user.email", "release-coherence@example.invalid"],
    );
    run(&repo, "git", &["config", "commit.gpgsign", "false"]);

    workspace_files(&repo, "0.1.0");
    release_changelog(&repo, "0.1.0", "0.0.0");
    commit(&repo, "release: 0.1.0");

    workspace_files(&repo, version);
    release_changelog(&repo, version, "0.1.0");
    commit(&repo, &format!("release: {version}"));

    Fixture { repo }
}

pub use crate::hermetic_git::commit;
