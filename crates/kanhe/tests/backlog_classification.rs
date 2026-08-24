//! Repository check: a live backlog entry sits under the class it declares.
//!
//! `BACKLOG.md` classifies work twice — once by the `### ` heading an entry sits under, and once by the
//! `*Class:*` line inside it. Two places holding one fact, and they disagreed: **seven of twelve live
//! entries under `### READY-PATCH` declared `*Class:* WATCH`**, and one declared no class at all.
//!
//! That is not a formatting complaint. The classification exists so a reader can ask *what is ready to work
//! on* and get an answer, and the heading is what they read — measured, the answer it gave was twelve when
//! the truth was four. A heading that over-reports promotable work is worse than no heading, because it is
//! consulted and believed.
//!
//! Both directions, because each catches what the other cannot:
//!
//! - an entry declaring a class the heading above it does not name — the seven;
//! - an entry declaring no class at all — the one, which the first direction cannot see, since a missing
//!   line disagrees with nothing.
//!
//! **Why this one is a reaction when the session that found it declined to build several others.** The
//! sibling residuals — whether a requirement outlived its mechanism, whether a figure reads as live or as a
//! record — need a judgement over what a sentence means, which this repository has designed, measured and
//! rejected three times. This one does not: a heading is a literal, a `*Class:*` line is a literal, and
//! whether they match is decidable. Where a class is decidable it gets a reaction.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use kanhe::refusal::{Refusal, cannot_judge_at};
use kanhe::selection::the_only;

/// The classes `BACKLOG.md`'s own governance section defines.
///
/// Named rather than derived from the headings, because a heading is exactly what this check does not
/// trust: deriving the legal set from the headings would let a typo'd heading define its own class and
/// admit every entry under it.
const CLASSES: [&str; 6] = [
    "READY-PATCH",
    "DESIGN-BREAKING",
    "WATCH",
    "ACCEPTED DEBT",
    "DECLINED",
    "BUILT / HISTORY",
];

/// The classes `AGENTS.md`'s classification sentence names, or `None` if that sentence cannot be read.
///
/// **`None` rather than an empty list, for the reason `merge_message_gate::admitted_types` gives**: a
/// contract that could not be parsed is a different fact from one that admits nothing, and returning empty
/// would make the comparison below hold vacuously — agreeing with whatever [`CLASSES`] already says while
/// reporting that it had checked.
///
/// Anchored on the sentence rather than on the individual words, and it ends at that sentence's period so
/// the backticked `BACKLOG.md` in the next one stays outside.
fn classified_classes(agents: &str) -> Result<Vec<String>, Refusal> {
    // **The anchor is asked how many times it occurs, not for its first occurrence.** `AGENTS.md` is
    // hand-edited prose that can hold the sentence twice, and this function derives the very class list the
    // whole check compares `BACKLOG.md` against — `nth(1)` would drop a second candidate clause without a
    // word, which is the habit two live defects in this repository already came from.
    let clause = the_only(
        "backlog classification clause in AGENTS.md",
        agents.split("Classify live work by its").skip(1),
    )?;
    let run = clause.split_once(". ").map_or(clause, |(run, _)| run);
    let classes: Vec<String> = run
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_string)
        .collect();
    if classes.is_empty() {
        Err(cannot_judge_at(
            "repository-checks#backlog-classification-clause-names-no-class",
            "the classification clause in AGENTS.md names no backticked class, and a contract this reader \
             cannot parse is not a contract admitting nothing"
                .to_string(),
        ))
    } else {
        Ok(classes)
    }
}

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("BACKLOG.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// The classes a `### ` heading names, or `None` if it names none.
///
/// A combined heading is legal and in use — `### WATCH / ACCEPTED / DECLINED / BUILT` holds four — so an
/// entry under one satisfies this check by declaring any of them.
///
/// A heading names a class when a `/`-separated part of it **opens** that class's name, because the headings
/// abbreviate: that same live heading writes `ACCEPTED` for `ACCEPTED DEBT` and `BUILT` for
/// `BUILT / HISTORY`. Guarded on a word boundary, so a heading cannot claim a class by sharing a prefix
/// with it.
fn heading_classes(heading: &str) -> Option<Vec<&'static str>> {
    let named: Vec<&'static str> = CLASSES
        .iter()
        .copied()
        .filter(|class| {
            heading.split('/').map(str::trim).any(|part| {
                !part.is_empty()
                    && class
                        .strip_prefix(part)
                        .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '/']))
            })
        })
        .collect();
    (!named.is_empty()).then_some(named)
}

/// The classes, longest first, so `ACCEPTED DEBT` is never read as `ACCEPTED` — a class this file does not
/// define, reported as if the entry had named something legal.
fn by_length() -> Vec<&'static str> {
    let mut classes: Vec<&'static str> = CLASSES.to_vec();
    classes.sort_by_key(|c| std::cmp::Reverse(c.len()));
    classes
}

/// The class an entry declares, in **either** form this file uses.
///
/// Two forms, both deliberate and both read here rather than one being made to conform to the other. The
/// classified sections carry a `*Class:*` line inside the entry; the combined section states the class as a
/// **title prefix** — `- **WATCH: …`, `- **ACCEPTED DEBT:**` — which is what a reader of that section scans.
/// Requiring the line as well would put the fact in two places inside the entry, which is the shape this
/// check exists to remove one level up.
///
/// The prefix is read first: where an entry carries both, the title is what the reader sees.
fn declared_class(entry: &str) -> Option<&'static str> {
    let title = entry.trim_start().strip_prefix("- ")?.trim_start();
    let title = title.strip_prefix("**").unwrap_or(title);
    // The same abbreviation the headings use — `ACCEPTED:` opens `ACCEPTED DEBT` — recognised by the same
    // rule, so the two readings of one convention cannot part company. Word-boundary guarded, so a title
    // cannot claim a class by sharing a prefix with it.
    if let Some(class) = by_length().into_iter().find(|class| {
        title.split_once(':').is_some_and(|(head, _)| {
            class
                .strip_prefix(head.trim())
                .is_some_and(|rest| rest.is_empty() || rest.starts_with([' ', '/']))
        })
    }) {
        return Some(class);
    }
    let marker = entry.find("*Class:*")?;
    let rest = entry[marker + "*Class:*".len()..].trim_start();
    by_length()
        .into_iter()
        .find(|class| rest.starts_with(class))
}

/// A closed entry does not stay under the live class it was filed under.
///
/// `BACKLOG.md`'s own governance says so, with its reason: *A **closed** item leaves the live class it was
/// filed under; it does not stay there struck through … because a class heading is read as a queue and an
/// entry that carries a question and its answer at once is a reader trap.* The closed-records section
/// repeats it from the other side — *They live here rather than under their own class heading because an
/// index that carries a question and its answer at once is a reader trap*. Two statements of one rule, and
/// nothing held either.
///
/// **This is the direction [`live_entries`] cannot supply, and the distinction is worth stating.** That
/// function skips a struck-through bullet deliberately: a closed entry keeps the `*Class:*` line it had when
/// it was written, and holding a record to today's headings is the falsification this repository refuses
/// generally. Skipping the **comparison** is right; it does not follow that the entry belongs where it sits.
/// This asks a different question — not *does its class match the heading* but *is it under a class heading
/// at all* — which needs none of the record's own text and so falsifies nothing.
///
/// Decidable, which is why it is a reaction: a struck-through bullet is a literal, and whether the heading
/// above it names a class is the same question [`heading_classes`] already answers for the check below. The
/// closed-records heading names none, so entries there are outside this by construction rather than by an
/// exception anyone maintains.
#[test]
fn a_closed_entry_does_not_stay_under_a_live_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    let text = std::fs::read_to_string(root.join("BACKLOG.md"))
        .expect("read BACKLOG.md, whose filing this check is about");
    let mut heading = String::new();
    let mut misfiled: Vec<String> = Vec::new();
    let mut struck = 0usize;
    for (index, line) in text.lines().enumerate() {
        if line.starts_with("### ") || line.starts_with("## ") {
            heading = line.trim_start_matches('#').trim().to_string();
            continue;
        }
        if !line.starts_with("- ~~") {
            continue;
        }
        struck += 1;
        if heading_classes(&heading).is_some() {
            let title: String = line.chars().skip(4).take(88).collect();
            misfiled.push(format!(
                "  BACKLOG.md:{}: under `{heading}` — {title}",
                index + 1
            ));
        }
    }
    // The corpus is the struck-through bullets, and it is scanned rather than literal — a change to how a
    // closed entry is marked would empty it, and an empty scan reports clean over nothing.
    assert!(
        struck > 0,
        "no entry in BACKLOG.md is struck through, so this direction decided nothing — the mark it reads by \
         has moved, not the property"
    );
    assert!(
        misfiled.is_empty(),
        "{} closed entr(y/ies) sit under a live class heading, which reads as a queue holding work that is \
         already done. Move each to `### Closed — reproduction records`, where the governance section and \
         that section's own preamble both say it belongs:\n{}",
        misfiled.len(),
        misfiled.join("\n")
    );
}

/// Every live entry, as `(heading, first line number, whole text)`.
///
/// A **live** entry is one whose bullet does not open with `~~`: a closed item is struck through and keeps
/// its `*Class:*` line as a record of what it was when written, which the closed-records section says in so
/// many words. Holding a record to today's headings is the falsification this repository refuses generally.
fn live_entries(text: &str) -> Vec<(String, usize, String)> {
    let mut out = Vec::new();
    let mut heading = String::new();
    let mut current: Option<(usize, String)> = None;
    for (index, line) in text.lines().enumerate() {
        let is_heading = line.starts_with("### ") || line.starts_with("## ");
        if is_heading || line.starts_with("- ") {
            if let Some((start, body)) = current.take() {
                out.push((heading.clone(), start, body));
            }
        }
        if is_heading {
            heading = line.trim_start_matches('#').trim().to_string();
            continue;
        }
        if let Some(bullet) = line.strip_prefix("- ") {
            if !bullet.starts_with("~~") {
                current = Some((index + 1, line.to_string()));
            }
            continue;
        }
        if let Some((_, body)) = current.as_mut() {
            body.push(' ');
            body.push_str(line.trim());
        }
    }
    if let Some((start, body)) = current {
        out.push((heading, start, body));
    }
    out
}

#[test]
fn every_live_entry_sits_under_the_class_it_declares() {
    let Some(root) = workspace_root() else {
        return;
    };
    let path: &Path = &root.join("BACKLOG.md");
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|err| panic!("cannot read BACKLOG.md, so nothing was classified: {err}"));

    let entries = live_entries(&text);
    let under_a_class: Vec<&(String, usize, String)> = entries
        .iter()
        .filter(|(heading, _, _)| heading_classes(heading).is_some())
        .collect();
    assert!(
        !under_a_class.is_empty(),
        "no live entry sits under a class heading, so this check would report clean over nothing — the \
         headings moved, and the reader this protects reads them"
    );

    let mut offences: Vec<String> = Vec::new();
    for (heading, line, body) in under_a_class {
        let allowed = heading_classes(heading).expect("filtered above");
        match declared_class(body) {
            None => offences.push(format!(
                "BACKLOG.md:{line} sits under `{heading}` and declares no `*Class:*`, so its class is \
                 whatever a reader infers from the heading — which is the second place this check exists to \
                 stop being authoritative"
            )),
            Some(declared) if !allowed.contains(&declared) => offences.push(format!(
                "BACKLOG.md:{line} declares `*Class:* {declared}` under the heading `{heading}`. The heading \
                 is what a reader consults to find promotable work; an entry filed against its own class \
                 makes that answer wrong"
            )),
            Some(_) => {}
        }
    }
    assert!(
        offences.is_empty(),
        "{} live backlog entr(ies) disagree with the heading they sit under:\n{}",
        offences.len(),
        offences.join("\n")
    );
}

/// [`CLASSES`] is the set `AGENTS.md` names, in both directions.
///
/// **The array was a second copy of the contract with nothing joining the two.** Its own doc explained why
/// the legal set is not derived from the *headings* — a typo'd heading would define its own class — and
/// said nothing about `AGENTS.md`, where the vocabulary is actually stated. A class dropped from the
/// contract would keep being admitted here, silently.
///
/// The shape is `merge_message_gate::admitted_types` / `gate_types`, which this crate already uses to hold
/// its Conventional Commit types against the same document. One sibling did it right; this is that.
#[test]
fn the_classes_are_the_ones_agents_md_names() {
    let Some(root) = workspace_root() else {
        return;
    };
    let agents = std::fs::read_to_string(root.join("AGENTS.md"))
        .expect("AGENTS.md states the classification this file judges by and must be readable");
    let contract: BTreeSet<String> = classified_classes(&agents)
        .unwrap_or_else(|refusal| {
            panic!(
                "cannot read the classification from AGENTS.md ({:?}): {}",
                refusal.kind, refusal.message
            )
        })
        .into_iter()
        .collect();
    let declared: BTreeSet<String> = CLASSES.iter().map(|c| (*c).to_string()).collect();

    let stated_but_unadmitted: Vec<&String> = contract.difference(&declared).collect();
    assert!(
        stated_but_unadmitted.is_empty(),
        "AGENTS.md classifies work under these and this check does not admit them, so an entry declaring \
         one reads as declaring nothing: {stated_but_unadmitted:?}"
    );
    let admitted_but_unstated: Vec<&String> = declared.difference(&contract).collect();
    assert!(
        admitted_but_unstated.is_empty(),
        "this check admits these classes and AGENTS.md names none of them, so a class could be dropped \
         from the contract and go on being accepted here: {admitted_but_unstated:?}"
    );
}
