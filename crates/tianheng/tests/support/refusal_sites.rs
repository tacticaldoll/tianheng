//! Where every refusal is constructed, taken from **what the compiler read** rather than from a second
//! implementation of its module resolution.
//!
//! The first design resolved `#[path]` attributes by text. That is a reimplementation of rustc's resolution,
//! and it is wrong in four directions at once: it misses a conventional `mod foo;`, an `include!`, and a
//! `#[cfg_attr(…, path = …)]`, and it *admits* files `cfg` excluded from the build actually being run. This
//! repository has already shipped a false negative from mimicking rustc's resolution by reasoning instead of
//! measuring against a real build.
//!
//! So the corpus is rustc's own answer. `cargo test --no-run --message-format=json` names every test target
//! and its executable, and the dep-info beside each executable lists precisely the sources rustc read. The
//! target list comes from the same place, so a newly added gate target is inside the corpus without anyone
//! remembering a list.
//!
//! **Why plain text scanning is sound here.** Recognising a construction inside a comment or a string literal
//! would be a false *positive* — a site nothing can reach, which this reaction reports loudly as unreachable.
//! The dangerous direction is the other one, and it cannot happen: a definition or a call written inside a
//! comment or a string is not a definition or a call. A fourth hand-rolled lexer would add a failure mode
//! without closing one. Measured before relying on it: the workspace today carries **no** occurrence of any of
//! these tokens outside the three files that own them.
//!
//! The reaction cross-checks this enumeration against a produced set anyway — every site a run *records*
//! having constructed must appear here, or the scan missed something.

#![allow(dead_code)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The one module that may define a refusal vocabulary.
pub const SHARED: &str = "crates/tianheng/tests/support/refusal.rs";

/// The target implementing this reaction. Its own sources are exempt from the vocabulary scan.
///
/// A reaction over text contains the text it recognises: the needles below are literals in this file, and a
/// scan that read them would refuse its own scanner. The exemption is **derived** — the compiler reports what
/// this target compiled — rather than a list of paths anyone maintains, and it is as narrow as the reaction
/// itself. What it costs is stated as a declared bound: a second vocabulary declared inside this reaction's
/// own sources is not observed.
pub const REACTION_TARGET: &str = "refusal_bites";

/// The constructors a site is built with.
pub const CONSTRUCTORS: [&str; 2] = ["violation", "cannot_judge"];

/// One test binary, and the sources the compiler reported reading for it.
#[derive(Debug, Clone)]
pub struct Target {
    pub name: String,
    pub executable: PathBuf,
    pub sources: BTreeSet<String>,
}

impl Target {
    /// Whether this target compiles the shared vocabulary, and so can construct a refusal at all.
    pub fn is_judged(&self) -> bool {
        self.sources.contains(SHARED)
    }
}

/// One construction of a refusal.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Site {
    pub file: String,
    pub line: u32,
    pub constructor: String,
}

impl Site {
    pub fn key(&self) -> String {
        format!("{}:{}", self.file, self.line)
    }
}

/// The corpus, its sites, and anything about the enumeration itself that fails.
#[derive(Debug)]
pub struct Corpus {
    pub targets: Vec<Target>,
    pub files: BTreeSet<String>,
    pub sites: Vec<Site>,
    pub offences: Vec<String>,
}

impl Corpus {
    /// The targets that can observe a site: those whose reported sources contain the site's file.
    ///
    /// Not "the targets that include the shared module" — that would run publish targets for release sites.
    /// Harmless, since a target that does not compile a site cannot be affected by poisoning it, but it would
    /// triple the sweep and stop being a statement about who can observe what.
    pub fn observers(&self, file: &str) -> Vec<&Target> {
        self.targets
            .iter()
            .filter(|target| target.is_judged() && target.sources.contains(file))
            .collect()
    }

    pub fn judged(&self) -> Vec<&Target> {
        self.targets.iter().filter(|t| t.is_judged()).collect()
    }
}

fn run(root: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(args[0])
        .args(&args[1..])
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .unwrap_or_else(|err| panic!("cannot run {args:?}: {err}"));
    (
        out.status.code(),
        String::from_utf8_lossy(&out.stdout).to_string(),
        String::from_utf8_lossy(&out.stderr).to_string(),
    )
}

/// Every test executable of the workspace, with the sources the compiler reported for each.
///
/// The whole workspace rather than one package: a refusal vocabulary defined in another crate's tests would
/// otherwise be outside, and the check that no second vocabulary exists is the one that has to see everywhere.
pub fn targets(root: &Path) -> Vec<Target> {
    let (code, stdout, stderr) = run(
        root,
        &[
            "cargo",
            "test",
            "--workspace",
            "--all-features",
            "--no-run",
            "--message-format=json",
        ],
    );
    assert_eq!(
        code,
        Some(0),
        "the corpus could not be built, and a failed build is not an empty corpus:\n{stderr}"
    );

    let mut targets = Vec::new();
    for line in stdout.lines() {
        let Ok(message) = serde_json::from_str::<serde_json::Value>(line) else {
            continue;
        };
        if message["reason"] != "compiler-artifact" || message["profile"]["test"] != true {
            continue;
        }
        let Some(executable) = message["executable"].as_str() else {
            continue;
        };
        targets.push(Target {
            name: message["target"]["name"]
                .as_str()
                .unwrap_or_default()
                .to_string(),
            executable: PathBuf::from(executable),
            sources: dep_info(root, Path::new(executable)),
        });
    }
    assert!(
        !targets.is_empty(),
        "the build reported no test executable; every property of an empty corpus holds, and reporting that \
         as a clean run is the vacuity direction"
    );
    targets
}

/// The workspace-relative Rust sources rustc reported reading for one executable.
fn dep_info(root: &Path, executable: &Path) -> BTreeSet<String> {
    let path = PathBuf::from(format!("{}.d", executable.display()));
    let text = std::fs::read_to_string(&path).unwrap_or_else(|err| {
        panic!(
            "the compiler reported no source list at {path:?}, so what {} compiled cannot be read — a \
             missing dep-info is not an empty corpus: {err}",
            executable.display()
        )
    });
    let mut sources = BTreeSet::new();
    for line in text.lines() {
        // `<target>: <dep> <dep> …`. Lines with no dependency list (`crates/a.rs:`) and the trailing
        // `# env-dep:` prose carry nothing.
        let Some((_, deps)) = line.split_once(": ") else {
            continue;
        };
        if line.starts_with('#') {
            continue;
        }
        for dep in deps.split_whitespace() {
            // Relative and inside the workspace: an absolute path is a dependency's own source, and
            // `target/` holds generated files no review sees as source.
            if dep.ends_with(".rs")
                && !dep.starts_with('/')
                && !dep.starts_with("target/")
                && root.join(dep).is_file()
            {
                sources.insert(dep.to_string());
            }
        }
    }
    sources
}

/// Everything tracked, as a set, for deciding whether a compiled file is reviewable.
pub fn tracked(root: &Path) -> BTreeSet<String> {
    let (code, stdout, stderr) = run(root, &["git", "ls-files"]);
    assert_eq!(
        code,
        Some(0),
        "`git ls-files` failed; a failed enumeration is not a repository tracking nothing:\n{stderr}"
    );
    stdout.lines().map(str::to_string).collect()
}

/// Read the corpus, its sites, and every offence against the enumeration's own totality.
pub fn build(root: &Path) -> Corpus {
    let targets = targets(root);
    let tracked = tracked(root);
    let mut files = BTreeSet::new();
    for target in &targets {
        files.extend(target.sources.iter().cloned());
    }

    // Derived, not listed: whatever the compiler says this reaction compiled.
    let machinery: BTreeSet<String> = targets
        .iter()
        .find(|target| target.name == REACTION_TARGET)
        .map(|target| target.sources.clone())
        .unwrap_or_default();

    // A refusal site can only exist where the shared vocabulary is compiled. Scanning every corpus file for
    // sites read 25 calls to an unrelated `fn violation(…) -> Violation` in the shell's own unit tests as
    // refusal sites — each of them a phantom the sweep could never perturb. The vocabulary scan still covers
    // every file, because a *second* vocabulary is exactly what would be declared somewhere the first is not.
    let mut can_construct: BTreeSet<String> = BTreeSet::new();
    for target in targets.iter().filter(|t| t.is_judged()) {
        can_construct.extend(target.sources.iter().cloned());
    }

    let mut sites = Vec::new();
    let mut offences = Vec::new();
    if machinery.is_empty() {
        offences.push(format!(
            "  no target named `{REACTION_TARGET}` was built, so the sources exempt from the vocabulary scan \
             cannot be derived; exempting nothing would refuse this reaction's own scanner, and exempting a \
             typed list is the drift this derivation exists to avoid"
        ));
    }
    for file in &files {
        if !tracked.contains(file) {
            offences.push(format!(
                "  {file} is compiled into a judged target but is not tracked; a reaction correlating source \
                 with a run judges what ran, then requires that it be reviewable"
            ));
        }
        let Ok(text) = std::fs::read_to_string(root.join(file)) else {
            offences.push(format!(
                "  {file} was reported as a source but cannot be read; a file that cannot be read is not a \
                 file holding no site"
            ));
            continue;
        };
        scan(
            file,
            &text,
            machinery.contains(file),
            can_construct.contains(file),
            &mut sites,
            &mut offences,
        );
    }

    sites.sort();
    Corpus {
        targets,
        files,
        sites,
        offences,
    }
}

/// A function signature starting at `line`, joined until its opening brace.
///
/// A signature may wrap, and reading only the line the `fn` sits on would miss `-> Refusal` on the next one —
/// a false negative in the direction that matters. Bounded, because an unterminated signature is a broken
/// file rather than a very long one.
fn signature_from(lines: &[&str], index: usize) -> String {
    let mut joined = String::new();
    for line in lines.iter().skip(index).take(8) {
        joined.push_str(line);
        joined.push(' ');
        if line.contains('{') {
            break;
        }
    }
    joined
}

/// One file's sites, and its offences against the vocabulary being singular.
///
/// `owns_the_scan` exempts only the **vocabulary** offences, never the site enumeration: the reaction's own
/// sources hold the needles as literals, but they hold no refusal construction. `can_construct` says whether
/// the shared vocabulary is compiled into this file's targets at all — where it is not, a call spelled
/// `violation(` is a different function with the same ordinary name.
fn scan(
    file: &str,
    text: &str,
    owns_the_scan: bool,
    can_construct: bool,
    sites: &mut Vec<Site>,
    offences: &mut Vec<String>,
) {
    let lines: Vec<&str> = text.lines().collect();
    let judged_for_vocabulary = file != SHARED && !owns_the_scan;
    for (index, line) in lines.iter().enumerate() {
        let number = index as u32 + 1;
        let trimmed = line.trim_start();
        if trimmed.starts_with("//") {
            continue;
        }

        if judged_for_vocabulary {
            for definition in ["struct Refusal", "enum Refusal"] {
                if line.contains(definition) {
                    offences.push(format!(
                        "  {file}:{number} defines `{definition}` outside {SHARED}; two refusal types can \
                         disagree about what each kind means while both read as one contract"
                    ));
                }
            }
            // A cannot-judge variant being *declared*. `Kind::CannotJudge` names the shared one and is how a
            // direction asserts a kind, which is the behaviour this reaction exists to require.
            if let Some(at) = line.find("CannotJudge") {
                if !line[..at].trim_end().ends_with("::") {
                    offences.push(format!(
                        "  {file}:{number} declares a second cannot-judge variant outside {SHARED}"
                    ));
                }
            }
        }

        let mut on_this_line = 0;
        for constructor in CONSTRUCTORS {
            let needle = format!("{constructor}(");
            let mut from = 0;
            while let Some(at) = line[from..].find(&needle) {
                let at = from + at;
                from = at + needle.len();
                let preceded_by_identifier = line[..at]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric() || c == '_');
                if preceded_by_identifier {
                    continue;
                }
                if line[..at].trim_end().ends_with("fn") {
                    // A definition, not a site — and a second vocabulary only if it actually returns the
                    // shared refusal. `violation` is an ordinary word: this workspace already has an
                    // unrelated `fn violation(…) -> Violation` building a structured finding, and refusing it
                    // would be a reaction crying about a name rather than about a contract.
                    if judged_for_vocabulary && signature_from(&lines, index).contains("-> Refusal")
                    {
                        offences.push(format!(
                            "  {file}:{number} defines `fn {constructor}` returning a Refusal outside {SHARED}"
                        ));
                    }
                    continue;
                }
                if !can_construct {
                    continue;
                }
                on_this_line += 1;
                sites.push(Site {
                    file: file.to_string(),
                    line: number,
                    constructor: constructor.to_string(),
                });
            }
        }
        if on_this_line > 1 {
            offences.push(format!(
                "  {file}:{number} constructs {on_this_line} refusals on one line; a site is selected by its \
                 line, so two on one line cannot be perturbed apart"
            ));
        }
    }
}

/// Group sites by file, for reporting and for deriving who observes what.
pub fn by_file(sites: &[Site]) -> BTreeMap<&str, Vec<&Site>> {
    let mut grouped: BTreeMap<&str, Vec<&Site>> = BTreeMap::new();
    for site in sites {
        grouped.entry(site.file.as_str()).or_default().push(site);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The shapes this scan is shown, kept as **data** rather than as string literals here.
    ///
    /// A fixture written inline would sit inside the corpus this scan reads, and a probe carrying
    /// `struct Refusal` would be found in the file that looks for it. Measured, not foreseen: the first run
    /// of this reaction reported five offences against its own test module. The `.rs.txt` suffix keeps them
    /// out of what the compiler reports as source, which is what the corpus is built from.
    fn fixture(name: &str) -> String {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/refusal_scan")
            .join(format!("{name}.rs.txt"));
        std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("the probe {path:?} is unreadable: {err}"))
    }

    fn scanned(name: &str) -> (Vec<Site>, Vec<String>) {
        let mut sites = Vec::new();
        let mut offences = Vec::new();
        scan(
            "some/gate.rs",
            &fixture(name),
            false,
            true,
            &mut sites,
            &mut offences,
        );
        (sites, offences)
    }

    #[test]
    fn a_call_is_a_site_and_a_definition_is_not() {
        let (sites, offences) = scanned("a_call_and_a_definition");
        assert_eq!(sites.len(), 1, "{sites:?}");
        assert_eq!(sites[0].line, 2);
        assert_eq!(sites[0].constructor, "violation");
        assert!(offences.is_empty(), "{offences:?}");
    }

    #[test]
    fn a_longer_identifier_ending_in_a_constructor_is_not_a_site() {
        let (sites, _) = scanned("a_longer_identifier");
        assert!(
            sites.is_empty(),
            "`a_violation(` was read as a site, so the scan names sites the sweep cannot perturb: {sites:?}"
        );
    }

    #[test]
    fn a_comment_carries_no_site() {
        let (sites, offences) = scanned("a_comment");
        assert!(sites.is_empty(), "{sites:?}");
        assert!(offences.is_empty(), "{offences:?}");
    }

    #[test]
    fn two_constructions_on_one_line_are_refused() {
        let (_, offences) = scanned("two_on_one_line");
        assert!(
            offences.iter().any(|o| o.contains("on one line")),
            "two sites on one line must be refused, since a site is selected by its line: {offences:?}"
        );
    }

    #[test]
    fn a_second_vocabulary_is_refused_wherever_it_is_declared() {
        for (probe, expected) in [
            ("a_second_refusal_type", "defines `struct Refusal`"),
            ("a_second_cannot_judge_variant", "cannot-judge variant"),
            ("a_second_constructor", "defines `fn cannot_judge`"),
            ("a_second_constructor_wrapped", "defines `fn violation`"),
        ] {
            let (_, offences) = scanned(probe);
            assert!(
                offences.iter().any(|o| o.contains(expected)),
                "{probe} declared a refusal vocabulary outside the shared module and was not refused: \
                 {offences:?}"
            );
        }
    }

    /// A signature that wraps still has to be read, or a second vocabulary hides behind a line break.
    #[test]
    fn a_wrapped_signature_is_still_read() {
        let (_, offences) = scanned("a_second_constructor_wrapped");
        assert!(
            !offences.is_empty(),
            "a constructor whose `-> Refusal` sits on a later line was not seen, which is a false negative \
             behind a line break"
        );
    }

    /// `violation` is an ordinary word, and this workspace already builds an unrelated one.
    ///
    /// Found by running this reaction rather than by reading: `crates/tianheng/src/runner/tests.rs` defines
    /// `fn violation(…) -> Violation`, a structured-finding builder with nothing to do with a refusal. A scan
    /// keyed on the name alone refuses a contract that does not exist.
    #[test]
    fn a_constructor_returning_something_else_is_not_a_second_vocabulary() {
        let (_, offences) = scanned("an_unrelated_violation_builder");
        assert!(
            offences.is_empty(),
            "an unrelated `fn violation(…) -> Violation` was refused as a second refusal vocabulary: \
             {offences:?}"
        );
    }

    /// Asserting a kind is the behaviour this reaction requires; it must not read as a second vocabulary.
    #[test]
    fn naming_the_shared_kind_is_not_declaring_one() {
        let (_, offences) = scanned("naming_the_shared_kind");
        assert!(
            offences.is_empty(),
            "asserting `Kind::CannotJudge` was read as declaring a variant, which would refuse the very \
             directions this reaction exists to require: {offences:?}"
        );
    }

    /// The scan's own sources are exempt from the vocabulary offences, and from nothing else.
    #[test]
    fn the_scan_does_not_refuse_its_own_needles() {
        let mut sites = Vec::new();
        let mut offences = Vec::new();
        scan(
            "crates/tianheng/tests/support/refusal_sites.rs",
            &fixture("a_second_refusal_type"),
            true,
            true,
            &mut sites,
            &mut offences,
        );
        assert!(
            offences.is_empty(),
            "the file implementing this scan holds the needles as literals; refusing them would make the \
             reaction refuse its own scanner: {offences:?}"
        );
    }
}
