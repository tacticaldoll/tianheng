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
use std::process::Command;
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
        match std::fs::create_dir(&candidate) {
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
const FORMATS: [(&str, Prose); 13] = [
    (".md", Prose::Whole),
    (".rs", Prose::LineComment("//")),
    (".toml", Prose::LineComment("#")),
    (".sh", Prose::LineComment("#")),
    (".yml", Prose::LineComment("#")),
    (".yaml", Prose::LineComment("#")),
    (".gitignore", Prose::LineComment("#")),
    (".lock", Prose::LineComment("#")),
    ("CODEOWNERS", Prose::LineComment("#")),
    (".txt", Prose::None),
    (".tsv", Prose::None),
    (".gitkeep", Prose::None),
    ("LICENSE", Prose::None),
];

/// How `path`'s format carries prose, or `None` if this repository has never classified it.
///
/// Matched on the whole file name first, then on the extension, so `CODEOWNERS` and `.gitignore` resolve without
/// an extension and `LICENSE-MIT` resolves by prefix — the licence files carry a variant suffix rather than an
/// extension.
fn prose_of(path: &str) -> Option<Prose> {
    let name = Path::new(path).file_name()?.to_str()?;
    if name.starts_with("LICENSE") {
        return Some(Prose::None);
    }
    FORMATS
        .iter()
        .find(|(key, _)| name == *key || name.ends_with(*key))
        .map(|(_, prose)| *prose)
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
    for path in all {
        *basename_count
            .entry(path.rsplit('/').next().unwrap_or(path))
            .or_default() += 1;
    }

    // Every basename this repository once tracked outside a change directory. A change directory's
    // scaffolding is pruned at every sync, so its names are the lifecycle's vocabulary rather than paths.
    let deleted_outside_changes: BTreeSet<String> = {
        let out = Command::new("git")
            .args(["log", "--diff-filter=D", "--name-only", "--format="])
            .current_dir(root)
            .output()
            .expect("run git log");
        assert!(
            out.status.success(),
            "could not read the deletion history; a failed read is not an empty result"
        );
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .filter(|p| !p.is_empty() && !p.starts_with("openspec/changes/"))
            .filter_map(|p| p.rsplit('/').next())
            .filter(|base| !all.iter().any(|f| f.rsplit('/').next() == Some(*base)))
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
            // crates inside this window, in the document the CHANGELOG advertises to adopters as the
            // provenance authority. Fourteen of the directory's fifteen path references already resolved,
            // so the blindness protected nothing and cost the one thing it was covering.
            if in_dated_section {
                continue;
            }
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
                    if let Some(home) = package_of(&files, rel_path) {
                        if holds(&files, &format!("{home}/{raw}")) {
                            continue;
                        }
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
                    if holds(&files, raw) || ignored(root, raw) {
                        continue;
                    }
                    // Illustrative rather than real, in two decidable forms.
                    //
                    // A `crates/<name>/…` path whose `<name>` is no tracked workspace member is a fixture in
                    // a doc comment or a test — `crates/foo/src/lib.rs` — and reading it as a dangling
                    // reference would make every example of the shape an offence. The rule needs the member
                    // set, which is why an empty one refuses above.
                    if let Some(rest) = raw.strip_prefix("crates/") {
                        if let Some(name) = rest.split('/').next() {
                            if !members.iter().any(|m| m == name) {
                                continue;
                            }
                        }
                    }
                    offences.insert(format!(
                        "{rel_path}: references '{raw}', which is not tracked in this repository"
                    ));
                    continue;
                }

                // A bare basename. Several tracked files carrying it says nothing about which was meant, and
                // one means the reference resolves. NONE is the case the spec left open, and discarding it
                // made this whole form inert — extracted and thrown away, which is why twenty-odd references
                // to files this window deleted survived the sweep.
                //
                // The decidable split: a name that WAS tracked, somewhere other than a change directory, is a
                // stale reference to something deleted. A name tracked only under `openspec/changes/` is the
                // lifecycle's own vocabulary — `proposal.md`, `tasks.md`, `design.md` are pruned at every
                // sync by design — and a name never tracked at all is not a path.
                match basename_count.get(raw) {
                    Some(_) => {}
                    None => {
                        if deleted_outside_changes.contains(raw) {
                            offences.insert(format!(
                                "{rel_path}: references '{raw}', which this repository deleted — point it at \
                                 what replaced it, or drop the reference"
                            ));
                        }
                    }
                }
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
/// why twenty-odd references to files this window deleted survived the sweep.
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
        (
            "a bare basename",
            "probe-basename.md",
            "The `check_dod_coherence.sh` gate says so.\n",
        ),
        (
            // A bare RUST basename, which the form excluded until this window: the extension list admitted
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
/// exactly one reference: a present-tense pointer at a gate that had moved crates inside this window, in the
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
    std::fs::create_dir_all(&scratch).expect("scratch is writable");

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
            "// `scripts/check_reference_integrity.sh` holds this.\n",
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

/// The unit words that carry a position instead of a name.
const POSITIONAL_UNITS: [&str; 4] = ["lines", "line", "paragraph", "sentence"];

/// The adverbs that stand in for the thing a reference should have named.
const POSITIONAL_ADVERBS: [&str; 4] = ["just", "immediately", "directly", "right"];

/// The positional reference `line` carries, if it carries one.
///
/// Three shapes, because prose writes it three ways: a counted unit, a definite article naming no thing, and an
/// adverb standing in for one. A bare direction word is none of them — a construct plus a direction to find it
/// is a reference to a thing, and the quiet half of this rule's direction asserts that.
///
/// The article case takes the plural too. The hand sweep that preceded this check wrote its pattern with the
/// singular only, and this check's first tree-wide run found the instance it had missed.
fn positional_reference(line: &str) -> Option<String> {
    let lower = line.to_ascii_lowercase();
    for direction in ["above", "below"] {
        for (index, _) in lower.match_indices(direction) {
            let before = lower[..index].trim_end();
            for unit in POSITIONAL_UNITS {
                if let Some(head) = before.strip_suffix(unit) {
                    let head = head.trim_end();
                    let last = head.rsplit(|c: char| !c.is_ascii_alphanumeric()).next();
                    let counted = last.is_some_and(|token| {
                        !token.is_empty()
                            && (token.chars().all(|c| c.is_ascii_digit())
                                || POSITIONAL_COUNTS.contains(&token))
                    });
                    if counted || head.ends_with("the") {
                        return Some(format!("{unit} {direction}"));
                    }
                }
            }
            for adverb in POSITIONAL_ADVERBS {
                if before.ends_with(adverb) {
                    return Some(format!("{adverb} {direction}"));
                }
            }
        }
    }
    None
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
        for (index, line) in text.lines().enumerate() {
            for span in line.split('`').skip(1).step_by(2) {
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
                    coordinates.push(format!("{path}:{}: `{span}`", index + 1));
                }
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
