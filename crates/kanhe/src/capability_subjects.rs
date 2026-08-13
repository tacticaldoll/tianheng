//! What each capability governs, and whether a change named the capabilities it touched.
//!
//! A capability that does not say what it governs makes every filing decision about it unfalsifiable, and the
//! decision is made once — in a proposal — and checked by nothing. Both halves are judged here: the
//! declaration, and the join between a change's **produced** diff and the capabilities its proposal names.

use std::collections::{BTreeMap, BTreeSet};

use crate::refusal::{Refusal, cannot_judge, violation};

/// The globs one capability declares, in the order it declares them.
pub type Subjects = BTreeMap<String, Vec<String>>;

/// What a spec's `## Subject` section declares.
///
/// Three outcomes, because two of them used to be one. A bullet this reader cannot parse is not a section
/// listing fewer globs — see [`subject_globs`].
#[derive(Debug, PartialEq, Eq)]
pub enum Declared {
    /// The spec carries no `## Subject` section.
    Absent,
    /// The globs the section lists, in the order it lists them.
    Globs(Vec<String>),
    /// A bullet this reader cannot understand, quoted as written.
    Unreadable(String),
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
/// Measured when this was written: 87 subject bullets across the specs, none of them unparseable — so the
/// silent narrowing was latent, and running the check could not have found it.
pub fn subject_globs(spec: &str) -> Declared {
    let Some(block) = spec.split("\n## Subject\n").nth(1) else {
        return Declared::Absent;
    };
    let block = block.split("\n## ").next().unwrap_or(block);
    let mut globs = Vec::new();
    for line in block.lines() {
        let Some(rest) = line.trim().strip_prefix("- ") else {
            continue;
        };
        match rest
            .strip_prefix('`')
            .and_then(|rest| rest.strip_suffix('`'))
        {
            Some(glob) if !glob.contains('`') => globs.push(glob.to_string()),
            _ => return Declared::Unreadable(line.trim().to_string()),
        }
    }
    Declared::Globs(globs)
}

/// The capability names a proposal's `## Capabilities` section mentions.
///
/// Read from backticked names, because that is how the template writes them and because a bare word in the
/// surrounding prose is not a claim about where a requirement belongs.
pub fn proposal_capabilities(proposal: &str) -> BTreeSet<String> {
    let Some(block) = proposal.split("\n## Capabilities\n").nth(1) else {
        return BTreeSet::new();
    };
    let block = block.split("\n## ").next().unwrap_or(block);
    let mut named = BTreeSet::new();
    let mut rest = block;
    while let Some(open) = rest.find('`') {
        rest = &rest[open + 1..];
        let Some(close) = rest.find('`') else { break };
        named.insert(rest[..close].to_string());
        rest = &rest[close + 1..];
    }
    named
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
        let globs = match subject_globs(spec) {
            Declared::Absent => {
                offences.push(violation(format!(
                    "`{capability}` declares no `## Subject`, so which files it governs is unfalsifiable and \
                     every requirement filed under it is filed by a name read loosely"
                )));
                continue;
            }
            // A cannot-judge, not a violation: the section may well claim exactly the right files, and this
            // reader cannot say. Reporting it as a shorter list would be the silent narrowing itself.
            Declared::Unreadable(bullet) => {
                offences.push(cannot_judge(format!(
                    "`{capability}` lists the subject bullet `{bullet}`, which this reader does not \
                     understand — the form it reads is one backticked glob and nothing else. Until it parses, \
                     what this capability governs cannot be decided, and reading past it would shrink the \
                     claim by exactly the bullet that could not be read"
                )));
                continue;
            }
            Declared::Globs(globs) => globs,
        };
        if globs.is_empty() {
            offences.push(violation(format!(
                "`{capability}` carries a `## Subject` section listing no glob, which claims nothing while \
                 reading as a declaration"
            )));
            continue;
        }
        for glob in globs {
            match resolve(&glob) {
                Err(err) => offences.push(cannot_judge(format!(
                    "could not resolve `{capability}`'s subject glob `{glob}`: {err}"
                ))),
                Ok(paths) if paths.is_empty() => offences.push(violation(format!(
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
        offences.push(violation(format!(
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
