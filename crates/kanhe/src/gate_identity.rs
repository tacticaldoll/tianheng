//! Whether the test a wrapper asks for by name is a test that exists.
//!
//! A gate reached through `cargo test … -- --exact <ident>` is asked for by a string, and `libtest` exits `0`
//! when that string selects nothing. The wrapper's own assertion covers the moment; this covers the interval,
//! by holding the identifier to the target it is cited against — a test identifier is a reference into this
//! repository exactly as a path is, and the reference gate matches paths only.

use std::path::Path;

use crate::refusal::{Refusal, cannot_judge_at, violation_at};

/// The directory every tracked file of which is a wrapper or the shared library — a category closed by
/// location, which is why nothing filters it by extension.
pub const SCRIPTS_DIRECTORY: &str = "scripts/";

/// Every tracked file under [`SCRIPTS_DIRECTORY`], with its text: what a direction that must see every script
/// reads, and what a list naming scripts is held against.
///
/// **No extension filter.** The citation check and the wrapper inventory each filtered the listing to `.sh`,
/// so an extensionless script was invisible to both at once while the requirement says what `git ls-files
/// scripts/` names. And they enumerated separately, so the inventory could find wrappers by one rule — a
/// sourcing line — while the citation check found scripts by another, and a script citing a gate without
/// loading the library was a member of the second set and not the first.
///
/// # Errors
///
/// A listing git did not answer, an empty one, or a tracked file that cannot be read — each described,
/// because every one of them returns what a repository holding no scripts would, and reporting that as clean
/// is the vacuity direction.
pub fn tracked_scripts(repo: &Path) -> Result<Vec<(String, String)>, String> {
    let listing = tracked_script_paths(repo)?;
    if listing.is_empty() {
        return Err(format!(
            "no tracked file under {SCRIPTS_DIRECTORY}, so a direction over the scripts would hold over nothing"
        ));
    }
    listing
        .into_iter()
        .map(|path| {
            let text = std::fs::read_to_string(repo.join(&path))
                .map_err(|err| format!("cannot read tracked {path}: {err}"))?;
            Ok((path, text))
        })
        .collect()
}

/// Every tracked path under `scripts/`: the one enumeration of that set, which [`tracked_scripts`] reads and the
/// release-coherence gate's machinery set is drawn from.
///
/// Empty is an answer here: a tree may hold no scripts, and whether that is a vacuity is the caller's to say.
///
/// # Errors
///
/// A listing git did not answer.
pub fn tracked_script_paths(repo: &Path) -> Result<Vec<String>, String> {
    crate::hermetic_git::tracked_paths(repo, &[SCRIPTS_DIRECTORY]).map_err(|failure| {
        format!("`git ls-files {SCRIPTS_DIRECTORY}` did not answer: {failure:?}")
    })
}

/// The one tracked script under `scripts/` that is not a wrapper: the shared library the wrappers source.
///
/// Declared once and named wherever a direction over the scripts would otherwise read it as a third wrapper —
/// a citation it cannot carry, an exit-class site it must not be counted against. One name, not a pattern:
/// a second library is a change to the requirement, not a row in a growing list.
pub const WRAPPERS_SHARED_LIBRARY: &str = "scripts/wrapper.sh";

/// One `--exact` citation found in a script: the identifier, and the invocation it belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Citation {
    /// The script the citation was read from, so a refusal can name the file to fix.
    pub script: String,
    /// The identifier given to `--exact`, exactly as written.
    pub identifier: String,
    /// The `--test <target>` of the same invocation, absent where the line names none.
    pub target: Option<String>,
    /// The `-p <package>` of the same invocation, absent where the line names none.
    pub package: Option<String>,
}

/// The value following the word `flag` in one statement's words, quote removal applied — what the shell
/// passes as the flag's argument.
fn value_after(words: &[crate::shell::Word], flag: &str) -> Option<String> {
    words
        .iter()
        .position(|word| !word.operator && word.value == flag)
        .and_then(|at| words.get(at + 1))
        .filter(|value| !value.operator)
        .map(|value| value.value.clone())
}

/// Every `--exact <ident>` a script cites, with the invocation each belongs to.
///
/// Read off the script's own statements — [`crate::shell::statements`] — so the invocation a flag belongs to
/// is the statement bash puts it in. A backslash-newline joins two lines into one invocation and nothing
/// else does: a line ending in an escaped backslash, or in a backslash inside single quotes, ends its
/// command at the newline, and a `--test` or `--exact` written on the next line is another statement's word,
/// never this citation's. A reader that joins those lines anyway binds an identifier to a target written in
/// a command bash never runs — the shape the fixture `a_citation_is_not_bound_across_a_line_bash_does_not_join`
/// carries. Comments need no region pass here: the lexer opens one where bash does, at an unquoted `#`
/// beginning a word.
///
/// # Errors
///
/// The line and name of the first construct the lexer cannot place. A script that cannot be read word by
/// word is refused rather than read past: a citation reported from past a misread word would be one this
/// reader never judged.
pub fn citations(script_path: &str, script: &str) -> Result<Vec<Citation>, (usize, &'static str)> {
    let mut found = Vec::new();
    for statement in crate::shell::statements(script)? {
        let words = &statement.words;
        for (at, word) in words.iter().enumerate() {
            if word.operator || word.value != "--exact" {
                continue;
            }
            let Some(identifier) = words.get(at + 1).filter(|word| !word.operator) else {
                continue;
            };
            found.push(Citation {
                script: script_path.to_string(),
                identifier: identifier.value.clone(),
                target: value_after(words, "--test"),
                package: value_after(words, "-p"),
            });
        }
    }
    Ok(found)
}

/// Each name a target's harness registers, **exactly as `--list` prints it**.
///
/// The full path, module qualification and all, because that is the string `--exact` compares against. Taking
/// the last `::` segment was the first shape and it is inexact in **both** directions: a test moved into a
/// module lists as `inner::the_gate`, truncates to `the_gate`, matches a citation of `the_gate` and reports the
/// gate registered — while `--exact the_gate` selects nothing, which is the condition this whole check exists
/// to catch. And a leaf shared by two modules truncates to one name twice, so a citation that `--exact` resolves
/// to exactly one test is refused for naming a set.
///
/// It read clean because both live citations sit at file scope, where the truncation is the identity function —
/// the comparison was `f() == f()`.
pub fn registered_names(listing: &str) -> Vec<String> {
    listing
        .lines()
        .filter_map(|line| line.split_once(": "))
        .filter(|(_, kind)| kind.trim() == "test")
        .map(|(name, _)| name.to_string())
        .collect()
}

/// Whether each citation names a test its target registers exactly once.
///
/// The identifier is resolved through the **harness** — `cargo test -p <pkg> --test <target> -- --list` — not
/// by mapping the target to a source path. That mapping would reimplement cargo's target resolution in string
/// form, and this repository has already shipped a false negative from mimicking a compiler's resolution by
/// reasoning instead of measuring. `--list` *is* the set `--exact` filters against, so the join is exact only
/// while both sides carry the **same** name — which is why the listed name is compared whole rather than by its
/// last segment, and why a name registered twice is settled for free.
pub fn offences(
    citations: &[Citation],
    list: impl Fn(&str, &str) -> Result<String, String>,
) -> Vec<Refusal> {
    let mut offences = Vec::new();
    for citation in citations {
        let (Some(package), Some(target)) = (&citation.package, &citation.target) else {
            offences.push(cannot_judge_at(
                "repository-checks#citation-names-no-test-target",
                format!(
                "{}: `--exact {}` names no `--test <target>` in its invocation, so the identifier cannot be \
                 bound to the harness that would register it — an identifier this check cannot resolve is \
                 not one it resolved as fine",
                citation.script, citation.identifier
            )));
            continue;
        };
        let listing = match list(package, target) {
            Ok(listing) => listing,
            Err(err) => {
                offences.push(cannot_judge_at(
                    "repository-checks#citation-target-listing-unreadable",
                    format!(
                    "{}: could not list the tests `{target}` registers in `{package}`, so whether it carries \
                     `{}` is unread rather than answered: {err}",
                    citation.script, citation.identifier
                )));
                continue;
            }
        };
        let matches = registered_names(&listing)
            .into_iter()
            .filter(|name| *name == citation.identifier)
            .count();
        match matches {
            1 => {}
            0 => offences.push(violation_at(
                "repository-checks#citation-names-an-unregistered-gate",
                format!(
                "{}: `--exact {}` names a test `{target}` does not register, so the gate this wrapper asks \
                 for selects nothing — and `libtest` exits 0 for a filter that matches nothing",
                citation.script, citation.identifier
            ))),
            many => offences.push(violation_at(
                "repository-checks#citation-names-a-gate-registered-several-times",
                format!(
                "{}: `--exact {}` names a test `{target}` registers {many} times, so the wrapper's citation \
                 names a set rather than the one gate it stands in front of",
                citation.script, citation.identifier
            ))),
        }
    }
    offences
}

/// Which of the enumerated scripts defer their verdict to no gate at all.
///
/// [`offences`] asks whether each citation resolves; this asks whether a script made one. The two are the same
/// question at different granularity, and only the second sees a script that cites **nothing** — which is a
/// script rendering its own verdict, the shape this repository deleted 1562 lines of and once had a whole
/// capability describing.
///
/// **Per script, never by counting.** Asserting that the citation total reaches the script count passes for two
/// scripts where one cites twice and the other not at all. Counting over the aggregate is exactly what the
/// direction above this did before, and it is what let the gap stand: every citation went into one list and the
/// list was asserted non-empty, so any single citing sibling covered for all the rest.
///
/// **A violation, not a cannot-judge.** The script was read and carries none; that is a source disagreeing with
/// the requirement rather than one this check could not read. An empty corpus is the different fact, and it is
/// the caller's to refuse — a set that never arrived is not a set in which every member cites a gate.
///
/// **One named script is not a wrapper and carries no citation: the shared library.** `scripts/wrapper.sh`
/// holds the lifecycle both wrappers are built on — the class helper, the ERR trap, the verdict channel, the
/// two guards over the gate's run — and renders no verdict of its own, so the citation stays with the
/// wrappers that source it. The exception is held **both ways**, because a one-way skip is how a named
/// exemption silently widens: a citation appearing inside the library is refused rather than skipped, and a
/// second library is a change to the requirement rather than a row in a growing exclusion list — the shape
/// `repository-checks` names when it says a refusal an operator cannot act on is one they work around.
///
/// **A script the lexer cannot place is a cannot-judge, the third state beside *cites* and *cites not*.**
/// [`citations`] refuses such a script rather than reading past the word it could not place, and this reader
/// reports that rather than either answer — an unreadable script is not one read as citing nothing.
///
/// What this buys is the **shape**: a script deferring to nothing cannot exist. It is not a proof that a script
/// which does defer does nothing else afterwards, and it does not try to be — deciding that from source text is
/// the judgement over prose this repository has designed, measured three times and rejected.
pub fn uncited_scripts<'a>(scripts: impl IntoIterator<Item = (&'a str, &'a str)>) -> Vec<Refusal> {
    scripts
        .into_iter()
        .filter_map(|(path, text)| {
            let found = match citations(path, text) {
                Ok(found) => found,
                Err((line, what)) => {
                    return Some(cannot_judge_at(
                        "repository-checks#a-script-the-shell-reader-cannot-place",
                        format!(
                            "{path}:{line} holds {what}, so which gates this script cites cannot be read — \
                             a script that cannot be read word by word is not one read as citing nothing"
                        ),
                    ));
                }
            };
            let cites = !found.is_empty();
            if path == WRAPPERS_SHARED_LIBRARY {
                return cites.then(|| {
                    violation_at(
                        "repository-checks#the-shared-library-names-a-gate",
                        format!(
                            "{path}: carries a gate citation, and it is the shared library the wrappers \
                             source — a citation here belongs to a wrapper, and adding one is a change to \
                             the requirement, not content the exemption covers"
                        ),
                    )
                });
            }
            (!cites).then(|| {
                violation_at(
                    "repository-checks#wrapper-cites-no-gate",
                    format!(
                        "{path}: names no gate by `--exact`, so it renders its own verdict rather than \
                         deferring to a Rust check. Every tracked script here is a wrapper: it gathers \
                         evidence and orders the act, and the judgement lives in `crates/kanhe`. A script \
                         that is not a wrapper belongs outside `scripts/`, or this requirement is amended \
                         deliberately"
                    ),
                )
            })
        })
        .collect()
}
