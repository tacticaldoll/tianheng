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

use std::path::{Path, PathBuf};

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
