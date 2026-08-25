//! What each capability governs, and whether a change named the capabilities it touched.
//!
//! A capability that does not say what it governs makes every filing decision about it unfalsifiable, and the
//! decision is made once — in a proposal — and checked by nothing. Both halves are judged here: the
//! declaration, and the join between a change's **produced** diff and the capabilities its proposal names.

use std::collections::{BTreeMap, BTreeSet};

use crate::refusal::{Refusal, cannot_judge_at, violation_at};
use crate::region::Prose;
use crate::sections::Section;

/// Any `## ` heading, named by the whole line.
///
/// **Any, not the one each caller wants** — because terminating a section is what a sentinel does, and the
/// section this module reads ends at whichever `## ` comes next. Matching only `## Subject` would make every
/// other heading part of the subject's body, which is the `split_once("\n## ")` this replaces, one level up.
/// The exact heading is the caller's filter, exactly as `release_coherence_gate` filters `## [Unreleased]` out
/// of every `## [` section.
///
/// `### ` does not match: the prefix carries its space, so a deeper heading is body rather than a boundary.
fn heading_section(line: &str) -> Option<String> {
    line.starts_with("## ").then(|| line.trim_end().to_string())
}

/// The sections one document declares under a named `## ` heading, in document order.
///
/// Each reader cuts its **own** document — a spec and a proposal are two files — which is why the cut happens
/// here rather than being handed in as `release_coherence_gate` hands it down. Four readers there share one
/// document and must share one cut; these two share a predicate and nothing else.
fn named_sections<'a>(document: &'a [Section], heading: &str) -> Vec<&'a Section> {
    document
        .iter()
        .filter(|section| section.name == heading)
        .collect()
}

/// The globs one capability declares, in the order it declares them.
pub type Subjects = BTreeMap<String, Vec<String>>;

/// What a spec's `## Subject` section declares.
///
/// A bullet this reader cannot parse is not a section listing fewer globs, and several sections are not the
/// first of them — see [`subject_globs`]. Four outcomes are what keeps those four answers apart; collapsing
/// any two of them is how one becomes the other's report.
#[derive(Debug, PartialEq, Eq)]
pub enum Declared {
    /// The spec carries no `## Subject` section.
    Absent,
    /// The globs the section lists, in the order it lists them.
    Globs(Vec<String>),
    /// A bullet this reader cannot understand, quoted as written.
    Unreadable(String),
    /// Several `## Subject` sections, so which one declares the subject is not this reader's to pick.
    SeveralSections(usize),
}

/// What a proposal's `## Capabilities` section names.
///
/// Sibling of [`Declared`], and the same four answers minus the one this section cannot give: a proposal with
/// no such section names nothing, which is [`Named::Names`] over an empty set rather than an absence. The
/// error channel this replaced was a bare `usize`, which had room for *how many sections* and none for
/// *unreadable* — so the reader that could not express the state skipped it, in the module whose sibling
/// reader refuses it.
#[derive(Debug, PartialEq, Eq)]
pub enum Named {
    /// The capability names the section mentions.
    Names(BTreeSet<String>),
    /// A section this reader cannot pair up, quoted as the refusal put it.
    Unreadable(String),
    /// Several `## Capabilities` sections, so which one names the capabilities is not this reader's to pick.
    SeveralSections(usize),
}

/// What a spec's `## Subject` section declares, refusing a bullet it cannot read.
///
/// **A bullet this reader cannot understand is refused, never dropped.** The form it reads is one backticked
/// glob and nothing else. A `- ` bullet with prose after the closing backtick, or with no backticks at all,
/// used to fall out of a `filter_map` — so the capability's declared subject silently shrank by exactly the
/// bullets the reader failed to parse, and [`join_offences`] then missed every file those globs claimed. That
/// is a capability quietly governing less than it says, which is the condition this whole module exists to
/// make falsifiable, performed by the module.
///
/// No spec's subject bullet was unparseable when this was written, so the silent narrowing was latent and
/// running the check could not have found it.
///
/// **And the same rule reaches the section, which `.nth(1)` did not.** Taking the text after the *first*
/// marker made no choice about how many there are: a spec carrying two `## Subject` sections had the second
/// one's globs dropped, so the capability governed less than it says while reading as a complete
/// declaration — the identical narrowing the bullet loop below refuses, one level up from it, and correct
/// only while a second section happened not to exist. The candidates are a value first now, and *how many*
/// is answered explicitly.
///
/// [`crate::selection::the_only`] is deliberately not used, for the reason `package_name` records: it
/// reports none and several as one refusal, and here they are different facts — no section means the
/// capability declared nothing, several means it declared twice and this reader may not pick.
pub fn subject_globs(spec: Prose<'_>) -> Declared {
    let document = crate::sections::cut(spec.numbered_lines(), heading_section);
    let sections = named_sections(&document, "## Subject");
    let block = match sections.len() {
        0 => return Declared::Absent,
        1 => sections[0],
        several => return Declared::SeveralSections(several),
    };
    let mut globs = Vec::new();
    for (_, line) in &block.body {
        let Some(rest) = line.trim().strip_prefix("- ") else {
            continue;
        };
        // One reader for a backticked run, shared with [`proposal_capabilities`] and
        // `merge_message_gate::admitted_types`. This site was the one of the three that refused correctly,
        // and it refused by a shape check rather than by counting the markers — so it could not have told
        // the two siblings anything. The form is still *one backticked glob and nothing else*: exactly one
        // run, and the bullet is that run with its markers.
        match crate::reading::backticked("subject bullet", rest) {
            Ok(runs) if runs.len() == 1 && rest == format!("`{}`", runs[0]) => {
                globs.push(runs[0].clone());
            }
            _ => return Declared::Unreadable(line.trim().to_string()),
        }
    }
    Declared::Globs(globs)
}

/// The capability names a proposal's `## Capabilities` section mentions, or how many such sections there are
/// where this reader may not pick one.
///
/// Read from backticked names, because that is how the template writes them and because a bare word in the
/// surrounding prose is not a claim about where a requirement belongs.
///
/// **`Err(count)` rather than the first section's names**, for the reason [`subject_globs`] states one
/// document over: reading past a second section drops exactly the capabilities it names, and [`join_offences`]
/// then reports a change as having accounted for a capability it never listed. An empty set is the honest
/// answer for a proposal carrying **no** such section — it names nothing — and that stays `Ok`.
pub fn proposal_capabilities(proposal: Prose<'_>) -> Named {
    let document = crate::sections::cut(proposal.numbered_lines(), heading_section);
    let sections = named_sections(&document, "## Capabilities");
    let block = match sections.len() {
        0 => return Named::Names(BTreeSet::new()),
        1 => sections[0],
        several => return Named::SeveralSections(several),
    };
    // Rejoined for the one reader that takes the section whole. The blank lines survive as empty strings, so
    // the paragraph structure `reading::backticked` pairs within is the document's own.
    let block: String = block
        .body
        .iter()
        .map(|(_, line)| line.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    let block = block.as_str();
    // **The error channel used to be a bare `usize`, so there was nowhere to put *unreadable*.** The loop
    // this replaced paired markers as it found them and `break`d on an opener with no closer, which drops
    // everything after it and shifts every pair before that: measured on a section listing `` `alpha` ``, a
    // stray marker, then `` `beta` ``, it answered `{" here\n- ", "alpha"}` — prose admitted as a capability
    // name and `beta` gone. Its sibling [`subject_globs`] returned a four-state enum and refused; this one
    // could not express the state, which is how the same rule got two answers in one module.
    match crate::reading::backticked("proposal's `## Capabilities` section", block) {
        Ok(runs) => Named::Names(runs.into_iter().collect()),
        Err(refusal) => Named::Unreadable(refusal.message),
    }
}

/// Every capability that claims `path`.
pub fn claimants(path: &str, claimed: &BTreeMap<String, BTreeSet<String>>) -> Vec<String> {
    claimed
        .iter()
        .filter(|(_, paths)| paths.contains(path))
        .map(|(capability, _)| capability.clone())
        .collect()
}

/// Whether every capability declares a subject, and whether every glob it declares resolves.
///
/// A glob matching nothing is a claim about nothing, and it reads as coverage while providing none.
pub fn declaration_offences(
    specs: &BTreeMap<String, String>,
    resolve: impl Fn(&str) -> Result<Vec<String>, String>,
) -> Vec<Refusal> {
    let mut offences = Vec::new();
    for (capability, spec) in specs {
        let source = crate::region::Source::of(spec.as_str());
        let globs = match subject_globs(source.prose()) {
            Declared::Absent => {
                offences.push(violation_at(
                    "repository-checks#capability-declares-no-subject",
                    format!(
                    "`{capability}` declares no `## Subject`, so which files it governs is unfalsifiable and \
                     every requirement filed under it is filed by a name read loosely"
                )));
                continue;
            }
            // A cannot-judge, not a violation: the section may well claim exactly the right files, and this
            // reader cannot say. Reporting it as a shorter list would be the silent narrowing itself.
            Declared::Unreadable(bullet) => {
                offences.push(cannot_judge_at(
                    "repository-checks#capability-subject-bullet-unreadable",
                    format!(
                    "`{capability}` lists the subject bullet `{bullet}`, which this reader does not \
                     understand — the form it reads is one backticked glob and nothing else. Until it parses, \
                     what this capability governs cannot be decided, and reading past it would shrink the \
                     claim by exactly the bullet that could not be read"
                )));
                continue;
            }
            // Its own message, because the count is what an author acts on and the bullet wording above
            // would send them looking for a bullet that parses fine.
            Declared::SeveralSections(count) => {
                offences.push(cannot_judge_at(
                    "repository-checks#capability-declares-several-subjects",
                    format!(
                    "`{capability}` carries {count} `## Subject` sections, so which one declares what it \
                     governs is decided by whichever comes first in the file. Reading the first would drop \
                     every glob the others claim — the silent narrowing this requirement exists to make \
                     falsifiable — so it is reported rather than resolved"
                )));
                continue;
            }
            Declared::Globs(globs) => globs,
        };
        if globs.is_empty() {
            offences.push(violation_at(
                "repository-checks#capability-subject-lists-no-glob",
                format!(
                "`{capability}` carries a `## Subject` section listing no glob, which claims nothing while \
                 reading as a declaration"
            )));
            continue;
        }
        for glob in globs {
            match resolve(&glob) {
                Err(err) => offences.push(cannot_judge_at(
                    "repository-checks#capability-subject-glob-unresolvable",
                    format!(
                    "could not resolve `{capability}`'s subject glob `{glob}`: {err}"
                ))),
                Ok(paths) if paths.is_empty() => offences.push(violation_at(
                    "repository-checks#capability-subject-glob-matches-nothing",
                    format!(
                    "`{capability}` declares the subject glob `{glob}`, which matches no tracked path — a \
                     glob matching nothing is a claim about nothing"
                ))),
                Ok(_) => {}
            }
        }
    }
    offences
}

/// Whether a change accounted for every capability claiming a file it touches.
///
/// **Every** claimant, not one of them. Naming one was the first rule and it was measured unable to catch the
/// defect it was written from: `scripts/publish.sh` is claimed both by the capability governing what must be
/// true before a publish and by the capability governing this repository's checks, so a change naming only
/// the second passed while filing a wrapper's requirement under a repository-check subject.
///
/// Requiring all of them does not refuse an honest proposal, because *accounting for* a capability is not
/// *listing it as modified*: a Capabilities section that names it while saying why its requirements do not
/// change satisfies this, and writing that sentence is the discipline the join exists to make routine.
///
/// A file no capability claims is not judged — subjects are declared where a capability has something to say,
/// and that blindness is a declared bound rather than an omission.
pub fn join_offences(
    change: &str,
    touched: &[String],
    listed: &BTreeSet<String>,
    claimed: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<Refusal> {
    let mut offences = Vec::new();
    for path in touched {
        let claimants = claimants(path, claimed);
        let unaccounted: Vec<String> = claimants
            .into_iter()
            .filter(|c| !listed.contains(c))
            .collect();
        if unaccounted.is_empty() {
            continue;
        }
        offences.push(violation_at(
            "repository-checks#change-touches-a-governed-path-unaccounted",
            format!(
            "`{change}` touches `{path}`, which `{}` governs without being accounted for, and its proposal \
             names {}. Name each in the Capabilities section — as modified, or with the reason its \
             requirements do not change",
            unaccounted.join("`, `"),
            if listed.is_empty() {
                "no capability".to_string()
            } else {
                format!(
                    "`{}`",
                    listed.iter().cloned().collect::<Vec<_>>().join("`, `")
                )
            }
        )));
    }
    offences
}
