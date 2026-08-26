//! Repository check: a branch a guard has already made unreachable is not written.
//!
//! `AGENTS.md` states the bound — *fail loud only on observable misconfiguration, no defensive
//! over-foolproofing of impossible states* — and until now nothing observed it. `str::split` and
//! `str::rsplit` always yield at least one item, so a `.next()` on either is always `Some`, and every
//! consumer that treats it as fallible is a branch nothing can reach: the fallback is dead, and reading it
//! tells a later reader that the empty case happens.
//!
//! **This was measured once and not swept.** `merge_message_gate` records the measurement in its own words —
//! *`"".split(". ").next()` is `Some("")`, measured — so the branch saying the clause has no sentence after
//! its anchor was unreachable* — and replaced its own site. Measured when this reader was written: twenty-four
//! sibling sites across three crates kept the shape, two of them written *after* that paragraph existed. A
//! measurement that repairs one site and leaves its class open is the shape this repository removes on sight;
//! a reaction is what closes it.
//!
//! **What it closes is the DECIDABLE part of the class, and saying otherwise was an over-claim.** The
//! consumer side here is general — chosen by what each does to the `Option` rather than by the spellings one
//! round met — but the producer side is two, and deliberately: `str::split` and `str::rsplit` always yield an
//! item *unconditionally*, which is what a reader over text can know. `xs.max()`, `slice.chars().next_back()`
//! and `entry.file_name()` are always-`Some` only on a producer that is non-empty, and whether it is takes
//! the surrounding code rather than the line — measured, the tree carries such calls where a lexical reader
//! cannot tell which. Widening the list would refuse them, and a false refusal is the direction the Core
//! Contract forbids more strictly than a miss.
//!
//! So the wider class — *any* branch a guard has already made unreachable — is a row in `AGENTS.md`'s
//! disposition table, applied by a reviewer, beside the other rows no reaction can reach. A review found four
//! live sites of it in one pass, which is the row working.
//!
//! `splitn` and `rsplitn` are outside the corpus: `"".splitn(0, ' ').next()` is `None`, so their fallbacks are
//! reachable. `.expect(…)` is admitted — it documents the impossibility instead of branching on it — and so
//! is `.filter(…)`, which can genuinely produce `None`.
//!
//! The corpus is executed Rust through [`kanhe::region::Source::rust`], so this file's own prose may write
//! the shape it forbids: a comment is not code, and a reader matching the bare text would refuse its own
//! reason.

use std::path::PathBuf;

use kanhe::refusal::{Kind, Refusal, cannot_judge};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("Cargo.toml").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// The receiver methods whose `.next()` is always `Some`.
///
/// Both are infallible for the reason `str::split`'s contract gives: the iterator yields the text before the
/// first separator, and a subject with no separator is one item, not none. `splitn` is absent deliberately —
/// its zero-limit form yields nothing.
const INFALLIBLE: [&str; 2] = [".split(", ".rsplit("];

/// Consumers that read an always-`Some` value as if it could be absent.
///
/// **Chosen by what each does to the `Option`, not by the spellings one round happened to meet.** The first
/// list held four, so the same dead default written `.map_or(d, f)` was invisible and two live sites — one in
/// a published crate — used `.is_some_and(` and `== Some(`. Every entry here is total over `Option` and a
/// no-op on an always-`Some` one: it either carries a value nothing reaches, or answers a question whose
/// answer is fixed. `.map(`, `.and_then(`, `.filter(` and `.expect(` are absent deliberately — the first two
/// keep the `Option`, `.filter` can genuinely produce `None`, and `.expect` documents the impossibility
/// instead of branching on it.
const AS_IF_FALLIBLE: [&str; 13] = [
    "?",
    ".unwrap_or(",
    ".unwrap_or_default(",
    ".unwrap_or_else(",
    ".map_or(",
    ".map_or_else(",
    ".ok_or(",
    ".ok_or_else(",
    ".is_some(",
    ".is_none(",
    ".is_some_and(",
    "== Some(",
    "!= Some(",
];

/// Constructs that consume the `Option` itself, so the offence is the enclosing form rather than a suffix.
///
/// `let Some(` covers both `if let Some(… ) = …` and the `let … else` binding; `match` covers the arm pair
/// whose `None` half nothing reaches.
const CONSUMES_THE_OPTION: [&str; 3] = ["let Some(", "match ", "filter_map(|"];

/// Where a call's argument list ends, counting nested parentheses and skipping quoted text.
///
/// A separator is often a character literal, and `split(')')` would otherwise close the list one character
/// into it — the argument is not text to this reader unless its quotes are honoured.
fn argument_list_ends(text: &str, open: usize) -> Option<usize> {
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut at = open;
    while at < bytes.len() {
        match bytes[at] {
            b'(' => depth += 1,
            b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some(at);
                }
            }
            b'\'' | b'"' => {
                let quote = bytes[at];
                at += 1;
                while at < bytes.len() && bytes[at] != quote {
                    at += if bytes[at] == b'\\' { 2 } else { 1 };
                }
            }
            _ => {}
        }
        at += 1;
    }
    None
}

/// One file's executed Rust as logical lines, each continued method chain joined to the line that starts it.
///
/// **The positions come from the region reader, not from re-counting.** `numbered_lines` *drops* a whole-line
/// comment rather than yielding it blank, so enumerating what `lines` returns numbers the remainder, and every
/// offence is then reported at a line holding something else. Four were, on the first run of this reader.
///
/// `rustfmt` breaks a long chain so that `.next()` and its consumer land on their own lines, and a
/// line-at-a-time reader sees neither beside the call they belong to — which is how a `grep` over this
/// repository found twenty sites of this shape when this was written and this reader found twenty-four.
fn logical_lines(text: &str) -> Vec<(usize, String)> {
    let source = kanhe::region::Source::of(text);
    let mut joined: Vec<(usize, String)> = Vec::new();
    for (number, line) in source.rust().numbered_lines() {
        let trimmed = line.trim_start();
        match joined.last_mut() {
            Some((_, previous)) if trimmed.starts_with('.') || trimmed.starts_with('?') => {
                previous.push_str(trimmed);
            }
            _ => joined.push((number, trimmed.to_string())),
        }
    }
    joined
}

/// Every unreachable branch in one file's executed Rust.
fn offences(path: &str, text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (number, line) in logical_lines(text) {
        for receiver in INFALLIBLE {
            let mut from = 0usize;
            while let Some(at) = line[from..].find(receiver) {
                let call = from + at;
                from = call + receiver.len();
                let Some(close) = argument_list_ends(&line, call + receiver.len() - 1) else {
                    continue;
                };
                let after = &line[close + 1..];
                let Some(tail) = after.strip_prefix(".next()") else {
                    continue;
                };
                let after = tail.trim_start();
                let suffix = AS_IF_FALLIBLE.iter().find(|c| after.starts_with(**c));
                // **The construct must consume *this* expression, not merely stand earlier on the line.**
                // Asking whether the prefix *contains* the marker reported
                // `if let Some(x) = lookup() { let first = value.split('/').next(); }` — where the `if let`
                // judges `lookup()`. Between a construct that consumes an expression and that expression
                // there is no statement boundary, so a `;`, `{` or `}` in between says it consumes
                // something else.
                let enclosing = CONSUMES_THE_OPTION
                    .iter()
                    .filter(|marker| {
                        line[..call].rfind(**marker).is_some_and(|at| {
                            !line[at + marker.len()..call].contains([';', '{', '}'])
                        })
                    })
                    .find(|_| {
                        after.is_empty()
                            || after.starts_with(')')
                            || after.starts_with(',')
                            || after.starts_with(';')
                            || after.starts_with('{')
                            || after.starts_with("else")
                    });
                let (consumer, how) = match (suffix, enclosing) {
                    (Some(consumer), _) => (*consumer, "the fallback is dead"),
                    (None, Some(construct)) => (*construct, "the condition cannot fail"),
                    (None, None) => continue,
                };
                found.push(format!(
                    "  {path}:{number}: `{}` always yields at least one item, so `{consumer}` here is a \
                     branch nothing reaches — {how}",
                    receiver.trim_start_matches('.').trim_end_matches('(')
                ));
            }
        }
    }
    found
}

/// Every tracked Rust file under `crates/`, read through the hermetic builder.
fn tracked_rust(root: &std::path::Path) -> Result<Vec<(String, String)>, Refusal> {
    let listing = kanhe::hermetic_git::run(root, &[], &["ls-files", "-z", "crates"]).map_err(
        |failure| {
            cannot_judge(format!(
                "CannotJudge: could not enumerate the tracked Rust sources ({failure:?}), so no file was \
                 inspected"
            ))
        },
    )?;
    let mut read = Vec::new();
    for path in listing.split('\0').filter(|p| p.ends_with(".rs")) {
        let text = std::fs::read_to_string(root.join(path)).map_err(|err| {
            cannot_judge(format!(
                "CannotJudge: {path} is tracked and could not be read ({err}) — an unread file is not a file \
                 without an offence"
            ))
        })?;
        read.push((path.to_string(), text));
    }
    Ok(read)
}

/// No tracked Rust source branches on a value a guard has already decided.
#[test]
fn no_branch_reads_an_always_some_value_as_if_it_could_be_absent() {
    let Some(root) = workspace_root() else {
        return;
    };
    let sources = tracked_rust(&root).expect("enumerate and read the tracked Rust sources");
    // A corpus that collapsed to nothing satisfies *no offence* exactly as a clean one does, and those are
    // opposite facts.
    assert!(
        sources.len() > 50,
        "only {} tracked Rust sources were read, which is not this workspace — the sweep would report clean \
         over almost none of it",
        sources.len()
    );
    let offences: Vec<String> = sources
        .iter()
        .flat_map(|(path, text)| offences(path, text))
        .collect();
    assert!(
        offences.is_empty(),
        "these branches cannot be reached, so their fallbacks tell a later reader that a case happens which \
         cannot:\n{}",
        offences.join("\n")
    );
}

/// The reader finds the shape however `rustfmt` broke it, and leaves the reachable forms alone.
///
/// Constructed subjects rather than the tree, for the reason the sweep above cannot serve: the tree is clean,
/// so it reports green whether this reader works or not. Each row states which side of the boundary it is on.
#[test]
fn the_reader_separates_a_dead_fallback_from_a_reachable_one() {
    // Built from pieces so the offending shape never appears whole in one literal — the sweep above reads
    // this file too, and a fixture written plainly would be an offence in the corpus it is testing.
    let split = ".split('/')";
    let rsplit = ".rsplit(\"::\")";
    let offending = [
        format!("let a = b{split}.next().unwrap_or(b);"),
        format!("let a = b{rsplit}.next().unwrap_or_default();"),
        format!("let a = b{split}.next()?;"),
        // The chain as `rustfmt` leaves it when it is too long for one line.
        format!("let a = b\n    {split}\n    .next()\n    .unwrap_or(b);"),
        format!("if let Some(a) = b{split}.next() {{}}"),
        format!("c.filter_map(|b| b{split}.next()).collect()"),
        // The spellings the first vocabulary missed, two of them taken from live sites — one in a published
        // crate. Each is total over `Option` and a no-op on an always-`Some` one.
        format!("let a = b{rsplit}.next().is_some_and(|leaf| leaf == c);"),
        format!("let a = b{rsplit}.next() == Some(c);"),
        format!("let a = b{split}.next().map_or(d, f);"),
        format!("let a = b{split}.next().ok_or(e)?;"),
        format!("let a = b{split}.next().is_none();"),
        format!("let Some(a) = b{split}.next() else {{ return }};"),
        format!("match b{split}.next() {{ Some(a) => a, None => d }}"),
    ];
    for subject in &offending {
        assert_eq!(
            offences("fixture.rs", subject).len(),
            1,
            "this is a dead branch and the reader reported none: {subject:?}"
        );
    }

    let reachable = [
        // `expect` documents the impossibility instead of branching on it.
        format!("let a = b{split}.next().expect(\"split yields one item\");"),
        // `filter` can genuinely produce `None`.
        format!("let a = b{split}.next().filter(|s| !s.is_empty());"),
        // `splitn`'s zero-limit form yields nothing, so its fallback is reachable.
        "let a = b.splitn(0, '/').next().unwrap_or(b);".to_string(),
        // `split_once` returns `None` when the separator is absent — the shape every repair moved to.
        "let a = b.split_once('/').map_or(b, |(head, _)| head);".to_string(),
        // A separator that is itself a parenthesis must not close the argument list early.
        "let a = b.split(')').next().expect(\"one item\");".to_string(),
        // Comments are not code: this file's own prose writes the shape it forbids.
        format!("// let a = b{split}.next().unwrap_or(b);"),
        // **The construct must consume this expression, not merely stand earlier on the line.** Both of
        // these are legal code the first reader called a dead branch, because it asked whether the text
        // before the call *contained* the marker: the `if let` judges `lookup()`, and the `filter_map`
        // closure returns a `String`.
        format!("if let Some(x) = lookup() {{ let f = v{split}.next(); g(x, f); }}"),
        format!(
            "c.filter_map(|r| r.strip_prefix(p)).map(|r| {{ let h = r{split}.next(); h.unwrap_or(r).to_string() }})"
        ),
        // `.map(` and `.and_then(` keep the `Option`, so neither reads it as absent.
        format!("let a = b{split}.next().map(str::trim);"),
        format!("let a = b{split}.next().and_then(|s| s.parse().ok());"),
    ];
    for subject in &reachable {
        assert_eq!(
            offences("fixture.rs", subject),
            Vec::<String>::new(),
            "this branch is reachable, or is not code, and the reader called it an offence: {subject:?}"
        );
    }
}

/// No source outside `reading` pairs backtick markers by hand.
///
/// **The extraction closed three sites and the class stayed open at three more.** `reading::backticked` was
/// written because pairing markers as they arrive lets one unpaired marker shift every pair after it; the
/// round that wrote it converted the three sites a review had named and claimed the rule. Sweeping this
/// session's own output found `split('`').skip(1).step_by(2)` still standing in three test targets — the
/// exact shape, in the window whose subject was that shape. A reaction is what makes the seventh impossible.
///
/// The corpus is executed Rust, so the paragraphs recording this — including this one — write the shape they
/// forbid and are not read.
#[test]
fn no_source_outside_the_shared_reader_pairs_backticks_by_hand() {
    let Some(root) = workspace_root() else {
        return;
    };
    let sources = tracked_rust(&root).expect("enumerate and read the tracked Rust sources");
    let mut standing = Vec::new();
    for (path, text) in &sources {
        // `reading` is where the pairing lives, so it is the one file that may write it.
        if path.ends_with("src/reading.rs") {
            continue;
        }
        for (number, line) in logical_lines(text) {
            // Assembled from pieces, so the needle never appears whole in this file: the sweep reads this
            // source too, and a plainly written detector is the offence it detects.
            // Assembled from pieces, so no needle appears whole in this file: the sweep reads this source
            // too, and a plainly written detector is the offence it detects. Both primitives, because
            // `reading`'s own doc names two shapes — a `find` twice in a loop, and a `split` with
            // `step_by(2)` — and the reader that closed the second called the first clean. Measured: no
            // site outside `reading` uses either today, so this closes the door rather than a set of them.
            for needle in [concat!(".split(", "'`')"), concat!(".find(", "'`')")] {
                if line.contains(needle) {
                    standing.push(format!(
                        "  {path}:{number}: searches for a backtick marker itself — pairing them as they \
                         arrive lets one unpaired marker shift every pair after it, which reads as prose \
                         named and a name dropped rather than as an error. Call \
                         `kanhe::reading::backticked`"
                    ));
                }
            }
        }
    }
    assert!(
        standing.is_empty(),
        "these sites pair backticks themselves, so the reader that decides the count first is not the only \
         one:\n{}",
        standing.join("\n")
    );
}

/// An unread file is not a file without an offence.
#[test]
fn a_source_that_cannot_be_read_refuses_rather_than_being_skipped() {
    let Some(root) = workspace_root() else {
        return;
    };
    let refusal = tracked_rust(std::path::Path::new(&root.join("no-such-directory")));
    match refusal {
        Err(refusal) => assert_eq!(
            refusal.kind,
            Kind::CannotJudge,
            "a corpus that could not be enumerated is not a clean one"
        ),
        Ok(read) => panic!(
            "enumerating a directory that is not a repository read {} files rather than refusing",
            read.len()
        ),
    }
}
