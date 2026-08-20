//! Reading the observation-bound register out of the tracked specs.
//!
//! Shared by the register check and by the census sweep, because a census is produced by the check that
//! enumerates the set — a second parse would let the two disagree, which is the drift the census rule exists
//! to end.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

/// This workspace's root, or `None` when the checks are running somewhere that is not it.
///
/// Every gate reading tracked files returns early on `None` rather than judging a tree it cannot find.
pub fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("openspec/specs").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

/// Run a search whose *ordinary* no-match answer is a non-zero status, and return the matching lines.
///
/// `grep` exits 1 on a clean miss. Treating that as a failure was found the hard way in this repository's
/// shell era and is recorded in the library that replaced it: a producer's contract has to be named per call
/// site rather than inferred, because the alternative — treating every non-zero as empty — turns a failed
/// read into a clean verdict, which is the one direction the Core Contract forbids.
///
/// **`program` is its own parameter, because no caller here ever chose one at run time.** This pair took the
/// program inside `args`, which [`crate::hermetic_git::fixture`]'s doc calls the shape to reach for only
/// where the program *is* chosen at run time — and every one of this pair's call sites named a literal,
/// `"git"` or `"cargo"`. A rule with an exception nothing needed is a rule the next caller reads as
/// permission.
pub fn search(root: &Path, what: &str, program: &str, args: &[&str]) -> Vec<String> {
    crate::hermetic_git::search(root, what, program, args)
}

/// Run a command in `root`, requiring success, and return its stdout.
///
/// A failed read is not an empty result: reporting one as the other would report a verdict over content that
/// was never read, which is the vacuity direction the Core Contract forbids.
///
/// `program` is its own parameter for the reason [`search`] states.
pub fn must(root: &Path, what: &str, program: &str, args: &[&str]) -> String {
    crate::hermetic_git::read(root, what, program, args)
}

/// The slug rule, applied to a scenario heading to derive a bound's id.
///
/// One implementation, shared by the register and by the model gate. It used to be duplicated, argued for as
/// the guard against the two rules drifting — but the two were byte-identical, so they could only catch drift
/// between themselves, a risk that existed solely because there were two. What the model gate actually
/// compares is a derived set against a **tracked file**, not against a second computation, and that property
/// is held by a case rather than by this comment: see [`projection_offences`].
pub fn slug_of(heading: &str) -> String {
    let mut out = String::with_capacity(heading.len());
    let mut pending_hyphen = false;
    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }
    out
}

/// Whether a scenario heading marks itself a bound, in the one form the register admits: the marker word
/// adjacent to `bound`, with no interposed qualifier. Measured: admitting one interposed word let the phrase
/// "stated and not yet declared as bounds" read as a declaration.
///
/// Case-folded before matching, for the same reason [`states_a_bound_in_prose`] and
/// [`negates_bound_in_prose`] are: a heading spelled `"... - A Stated Bound"` (Title Case, the shape a
/// heading-writing convention could plausibly reach for) marks a bound exactly as `"... - a stated bound"`
/// does, and an exact-case match would silently miss it — opening no bound for it in [`bounds_in`], and then
/// having the now-case-folded prose scan correctly (and confusingly) report the surrounding text as an
/// undeclared bound the author believed they *had* declared.
pub fn marks_a_bound(heading: &str) -> bool {
    let lower = heading.to_ascii_lowercase();
    ["a stated bound", "a documented bound"]
        .into_iter()
        .any(|marker| contains_words(&lower, marker))
}

/// Whether `trimmed` ends whichever scenario was open.
///
/// **One predicate, because three readers of this grammar had three copies of it and one of them differed.**
/// [`bounds_in`] and [`citations_in`] stopped at `## `/`### `/`#### `; [`undeclared_prose_offences`] stopped
/// at *any* line whose trimmed form starts with `#`. A `##### ` sub-heading fell between them — the first two
/// kept the bound scenario open while the third had left it, so prose below it was reported as an undeclared
/// bound that [`bounds_in`] had in fact registered. Latent, since no tracked spec carries a five-hash heading,
/// and the same class [`citations_in`]'s own comment records closing one heading-depth up.
///
/// A `#` line this does not name — `##### `, `# `, or a `#tag` — is ordinary content, which is what
/// [`bounds_in`] has always treated it as.
fn ends_scenario(trimmed: &str) -> bool {
    trimmed.starts_with("#### ") || trimmed.starts_with("### ") || trimmed.starts_with("## ")
}

fn contains_words(text: &str, words: &str) -> bool {
    text.match_indices(words).any(|(start, matched)| {
        let before = text[..start].chars().next_back();
        let after = text[start + matched.len()..].chars().next();
        before.is_none_or(|ch| !ch.is_alphanumeric())
            && after.is_none_or(|ch| !ch.is_alphanumeric())
    })
}

/// A token stripped of everything but its alphanumeric/hyphen core, so `"bound,"`, `"(a"`, and `"a"` compare
/// equal to their bare forms without a second tokenization pass.
fn bare_word(token: &str) -> &str {
    token.trim_matches(|c: char| !c.is_alphanumeric() && c != '-')
}

/// Whether `word` is a single word of letters/hyphens only — the shell era's `[A-Za-z-]+` class, for the one
/// interposed word both [`states_a_bound_in_prose`] and [`negates_bound_in_prose`] tolerate.
fn is_plain_word(word: &str) -> bool {
    !word.is_empty() && word.chars().all(|c| c.is_ascii_alphabetic() || c == '-')
}

/// Whether `line` states a bound in prose: `stated`/`documented`, optionally one interposed word of
/// letters/hyphens, then `bound`/`bounds`.
///
/// Ported from the shell era's own `BOUND_PROSE` scan (`(stated|documented)( [A-Za-z-]+)? bounds?`,
/// the shell era's bound-register gate, deleted by the migration to Rust and never reimplemented), as a
/// whitespace-tokenized walk rather than a regex — `kanhe`'s dependency law admits no regex crate. This is
/// **not** [`marks_a_bound`]: that recognizer derives a bound's *identity* from a scenario heading and was
/// deliberately narrowed to admit no interposed word, because a qualifier there doubled as an unclosed
/// classification feeding the derived id. This function's match feeds no id — it only decides whether a
/// line is a candidate for the declaration/exemption/reference check around it — so that reason for
/// tightening does not carry over, and the shell era's own looser tolerance is the right one to keep.
///
/// Word-boundary aware on every token (tighter than the shell's raw substring match, which had no boundary
/// on `stated`/`documented` itself — `"understated bounds"` would have matched it).
///
/// **Case-folded before tokenizing.** A sentence-initial `"Stated renderer-granularity bounds MAY..."` is
/// exactly the shape this function exists to catch, and an exact-case comparison against lowercase literals
/// never matches ordinary sentence capitalization — measured directly against a real tracked spec
/// (`semantic-dyn-trait-boundary/spec.md`'s "Stated renderer-granularity bounds MAY coalesce..."), which this
/// reader read past silently before this fix.
pub fn states_a_bound_in_prose(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let words: Vec<&str> = lower.split_whitespace().map(bare_word).collect();
    for (i, word) in words.iter().enumerate() {
        if *word != "stated" && *word != "documented" {
            continue;
        }
        match words.get(i + 1) {
            Some(&"bound") | Some(&"bounds") => return true,
            Some(interposed) if is_plain_word(interposed) => {
                if matches!(words.get(i + 2), Some(&"bound") | Some(&"bounds")) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// Whether `line` negates the bound noun directly: `rather than`/`not`/`never` immediately before
/// `a`/`an`, optionally one interposed word, then `bound`/`bounds`.
///
/// Ported from the shell era's own measured lesson (the deleted bound-register gate's `negated()`): a
/// wider "negation anywhere nearby" rule was tried first and hid three real declarations in this
/// repository's own specs while catching none of the intended cases, because each of those three has a
/// negation somewhere in the sentence that applies to a different verb, not to the bound noun. Only a
/// negation word sitting immediately against `a`/`an` denies the bound itself.
///
/// **Case-folded before tokenizing**, for the same reason [`states_a_bound_in_prose`] is: a sentence-initial
/// `"Not a stated bound"` denies the bound exactly as `"not a stated bound"` does, and an exact-case
/// comparison would silently fail to recognize the denial — the opposite-direction failure from
/// `states_a_bound_in_prose`'s, since a missed negation here reports a genuinely negated sentence as a
/// declaration instead of exempting it.
pub fn negates_bound_in_prose(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    let words: Vec<&str> = lower.split_whitespace().map(bare_word).collect();
    for (i, word) in words.iter().enumerate() {
        if *word != "a" && *word != "an" {
            continue;
        }
        let negator_immediately_before = (i >= 1 && matches!(words[i - 1], "not" | "never"))
            || (i >= 2 && words[i - 2] == "rather" && words[i - 1] == "than");
        if !negator_immediately_before {
            continue;
        }
        match words.get(i + 1) {
            Some(&"bound") | Some(&"bounds") => return true,
            Some(interposed) if is_plain_word(interposed) => {
                if matches!(words.get(i + 2), Some(&"bound") | Some(&"bounds")) {
                    return true;
                }
            }
            _ => {}
        }
    }
    false
}

/// How a declared bound answers the question every bound must answer: what defends it.
///
/// The two legal forms are exclusive — a test pins it, or a tracker owns the gap. The remaining variants are
/// the malformed answers, kept as values rather than as parse failures so the register can name which one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Citation {
    /// Every test cited. Several are legal: `observation-bound-model` declares that one bound may be
    /// defended by more than one test, and `BoundDecl::pinned_by_many` is its typed counterpart. The
    /// register's "exactly one citation" is about the two exclusive FORMS — pinned or tracked — not about
    /// how many tests a defence names. A first draft of this check read it as a bullet count, refused
    /// the one live instance, and split the scenario in two; the model's own bijection caught it.
    PinnedBy(Vec<String>),
    /// The whole remainder of the `UNPINNED` line, which names the tracker and says what it owns.
    Unpinned(String),
    /// `UNPINNED` with nothing after it — the gap is admitted and no one owns it.
    UnpinnedWithoutTracker,
    /// More than one `UNPINNED`. Several trackers are several **owners of one gap**, which is two answers to
    /// the question a citation exists to answer — unlike several `PINNED-BY`, which are several defences of
    /// one bound. The declaration holds one tracker, so keeping one of them silently records a bound whose
    /// owner is whichever line happened to be last.
    RepeatedUnpinned,
    /// Both forms at once. They are exclusive answers, so carrying both says the bound is defended and
    /// admittedly not defended.
    Both,
    /// Neither form. The scenario declares itself a bound and then says nothing about what holds it.
    Neither,
}

/// One observation bound as the tracked specs declare it.
#[derive(Debug, Clone)]
pub struct Bound {
    /// The `<capability>/<scenario-slug>` id, derived by [`slug_of`] rather than written down.
    pub id: String,
    /// The capability whose spec declares it.
    pub capability: String,
    /// The spec file it was read from, for citing the declaration back to a reader.
    pub spec: String,
    /// One-based line of the scenario heading, so a refusal can point at it.
    pub line: usize,
    /// The `THEN` bullet, continuation lines joined — what the projection quotes.
    pub body: String,
    /// What the scenario says defends it, in whichever form it took — including the malformed ones.
    pub citation: Citation,
}

/// Every tracked capability spec, as `(capability, repo-relative path)`.
pub fn tracked_specs(root: &Path) -> Vec<(String, String)> {
    let listing = must(
        root,
        "`git ls-files openspec/specs`",
        "git",
        &["ls-files", "-z", "openspec/specs"],
    );
    // **`-z`, because the capability name is derived from the path.** A quoted path fails `strip_prefix`,
    // so the spec is filtered out and that whole capability's declared bounds are never registered — the
    // same false-negative direction the release gate's listings carried, one layer worse because a dropped
    // path takes a set of bounds with it. Latent: no tracked path needs quoting today.
    let specs: Vec<(String, String)> = listing
        .split('\0')
        .filter(|p| p.ends_with("/spec.md"))
        .filter_map(|p| {
            p.strip_prefix("openspec/specs/")
                .and_then(|rest| rest.strip_suffix("/spec.md"))
                .map(|cap| (cap.to_string(), p.to_string()))
        })
        .collect();
    assert!(
        !specs.is_empty(),
        "`git ls-files` matched no openspec/specs/*/spec.md — this check would report clean without \
         reading anything, which is the vacuity direction"
    );
    specs
}

/// Read every declared bound out of the tracked specs.
/// Every declared bound in **one** spec's text.
///
/// Split out so a direction can be shown a shape rather than only this repository's own specs. A second
/// implementation of the parse would be the twin-drift class this repository keeps closing, and a control
/// pinned against a copy of the parse would say nothing about the parse.
pub fn bounds_in(capability: &str, spec: &str, text: &str) -> Vec<Bound> {
    let mut bounds = Vec::new();
    let lines: Vec<&str> = text.lines().collect();

    for (index, raw) in lines.iter().enumerate() {
        let Some(heading) = raw.strip_prefix("#### Scenario:") else {
            continue;
        };
        let heading = heading.trim();
        if !marks_a_bound(heading) {
            continue;
        }

        let mut body = String::new();
        let mut pinned: Vec<String> = Vec::new();
        let mut unpinned: Vec<String> = Vec::new();
        // A **count**, not a flag. The classification below asks whether more than one `UNPINNED`
        // appears, and a flag cannot answer that: two bare ones collapse to `true` exactly as one
        // does. That is the same narrowing as counting only the tracker-bearing lines, one variable
        // along.
        let mut unpinned_bare = 0usize;
        let mut in_then = false;

        for line in lines.iter().skip(index + 1) {
            let trimmed = line.trim();
            if ends_scenario(trimmed) {
                break;
            }
            if let Some(rest) = trimmed.strip_prefix("- **THEN** ") {
                body.push_str(rest);
                in_then = true;
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("- **PINNED-BY** ") {
                pinned.push(rest.trim().trim_matches('`').to_string());
                in_then = false;
                continue;
            }
            if trimmed == "- **UNPINNED**" {
                unpinned_bare += 1;
                in_then = false;
                continue;
            }
            if let Some(rest) = trimmed.strip_prefix("- **UNPINNED** ") {
                let rest = rest.trim();
                if rest.is_empty() {
                    unpinned_bare += 1;
                } else {
                    unpinned.push(rest.to_string());
                }
                in_then = false;
                continue;
            }
            if trimmed.starts_with("- ") {
                in_then = false;
                continue;
            }
            if in_then && !trimmed.is_empty() {
                body.push(' ');
                body.push_str(trimmed);
            }
        }

        // A scenario carrying two citations declares two bounds behind one heading, so the register
        // holds one of them and the other is defended by a test nothing points at. The old shell gate
        // projected the FIRST and moved on; rebuilding this check surfaced the one live instance.
        let tracked = !unpinned.is_empty() || unpinned_bare > 0;
        let citation = if !pinned.is_empty() && tracked {
            Citation::Both
        } else if !pinned.is_empty() {
            Citation::PinnedBy(pinned)
        // **Every `UNPINNED` line counts, whichever form it takes.** Counting only the
        // tracker-bearing ones let a scenario carrying one bare `UNPINNED` and one with a tracker
        // fall through to the single-tracker arm and read as a well-formed citation, silently
        // dropping the bare one — and counting the bare ones with a flag let two of them do the
        // same. The variant's own doc says *more than one `UNPINNED`*, which is this sum.
        } else if unpinned.len() + unpinned_bare > 1 {
            Citation::RepeatedUnpinned
        } else if let Some(tracker) = unpinned.pop() {
            Citation::Unpinned(tracker)
        } else if unpinned_bare > 0 {
            Citation::UnpinnedWithoutTracker
        } else {
            Citation::Neither
        };

        bounds.push(Bound {
            id: format!("{capability}/{}", slug_of(heading)),
            capability: capability.to_string(),
            spec: spec.to_string(),
            line: index + 1,
            body,
            citation,
        });
    }
    bounds
}

/// Whether a `### Requirement:` heading's own wording names bounds — the exemption
/// `observation-bound-register/spec.md` declares ("Prose under a requirement whose heading names bounds is
/// not reported"). Ported from the shell era's `tolower(req) ~ /bounds?([^a-z]|$)/`: `bound`/`bounds` as a
/// substring whose following character, if any, is not a letter — so `boundary` is excluded (`ary` follows)
/// but `bounds:`, `bound.`, or a heading ending in `bound` are not.
fn requirement_heading_is_bounds_named(heading: &str) -> bool {
    let lower = heading.to_ascii_lowercase();
    lower.match_indices("bound").any(|(start, _)| {
        let after_bound = &lower[start + "bound".len()..];
        let after = after_bound.strip_prefix('s').unwrap_or(after_bound);
        after
            .chars()
            .next()
            .is_none_or(|ch| !ch.is_ascii_alphabetic())
    })
}

/// The first `limit` **characters** of `text` (not bytes), so a truncation cannot split a multi-byte
/// character and panic.
fn truncate_chars(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}

/// Every offence this capability's spec commits against "A bound stated in prose but not declared as a
/// scenario SHALL fail": a bound-declaring prose line outside any declared bound scenario with no
/// resolvable reference, or a bounds-named requirement that states one and declares no bound scenario of
/// its own.
///
/// Ported from the shell era's own single-pass `awk` state machine (the deleted bound-register gate,
/// deleted by the migration to Rust and never reimplemented — the requirement's own text has described this
/// reaction the whole time). One pass over `text`'s lines tracks two nested states exactly as that script
/// did: the enclosing `### Requirement:` heading (and whether its own wording names bounds), and the
/// enclosing `####` block (and whether [`marks_a_bound`] accepts it as a declared bound scenario). A
/// triggering line ([`states_a_bound_in_prose`], minus [`negates_bound_in_prose`]) is cleared by sitting
/// inside a declared scenario, by carrying a resolvable reference ([`bare_references`]), or — inside a
/// bounds-named requirement — is exempted there in favor of charging that requirement for declaring at
/// least one bound scenario of its own.
///
/// `capabilities` is the same caller-enumerated set [`bare_references`] takes, for the same reason: a
/// capability added later must be recognized without this function being touched.
pub fn undeclared_prose_offences(
    spec: &str,
    text: &str,
    capabilities: &BTreeSet<String>,
) -> Vec<String> {
    let mut offences = Vec::new();

    let mut in_bound_scenario = false;
    let mut req_heading = String::new();
    let mut req_line = 0usize;
    let mut req_is_bounds = false;
    let mut req_declared_bound = false;
    let mut req_stated_undeclared = false;

    macro_rules! flush_requirement {
        () => {
            if req_is_bounds && req_stated_undeclared && !req_declared_bound {
                offences.push(format!(
                    "{spec}:{req_line} — the requirement \"{req_heading}\" names bounds, so its prose may \
                     state them, but it declares no bound scenario; a prose list with no reaction anywhere \
                     is the state this register opposes"
                ));
            }
        };
    }

    for (index, raw) in text.lines().enumerate() {
        let line_no = index + 1;
        let trimmed = raw.trim();

        if trimmed.starts_with("#### ") {
            // Any #### heading, however indented, closes whichever bound scenario was open —
            // `bounds_in`'s own body scan stops the same way, tolerating indentation once it is already
            // inside a scenario. Only a heading spelled "Scenario:" **at column zero** and accepted by
            // `marks_a_bound` reopens one: `bounds_in`'s own *opening* check is `raw.strip_prefix("####
            // Scenario:")`, untrimmed, so an indented "#### Scenario: ..." line never opens a bound there
            // either — checking `raw` rather than `trimmed` here keeps that exact, so an indented heading
            // cannot be treated as a declared bound scenario by this reader while going unregistered by
            // `bounds_in`, silently dropped from both.
            in_bound_scenario = false;
            if let Some(heading) = raw.strip_prefix("#### Scenario:") {
                if marks_a_bound(heading.trim()) {
                    in_bound_scenario = true;
                    req_declared_bound = true;
                }
            }
            continue;
        }

        if ends_scenario(trimmed) {
            // A `## `/`### ` heading (`#### ` was handled above and already `continue`d): closes whatever
            // requirement section was open, and opens a new one when it is itself a Requirement.
            //
            // **Through `ends_scenario`, not `starts_with('#')`.** The wider form ended a scenario here that
            // `bounds_in` and `citations_in` kept open — a `##### ` sub-heading, a `# ` title, a `#tag` —
            // and the prose below it was then read outside the very scenario that declares it. Those lines
            // are ordinary content to the other two readers, so they are ordinary content here.
            flush_requirement!();
            in_bound_scenario = false;
            req_heading.clear();
            req_line = 0;
            req_is_bounds = false;
            req_declared_bound = false;
            req_stated_undeclared = false;
            if let Some(name) = trimmed.strip_prefix("### Requirement:") {
                let name = name.trim();
                req_heading = name.to_string();
                req_line = line_no;
                req_is_bounds = requirement_heading_is_bounds_named(name);
            }
            continue;
        }

        if in_bound_scenario {
            continue;
        }
        if !states_a_bound_in_prose(trimmed) || negates_bound_in_prose(trimmed) {
            continue;
        }
        if req_is_bounds {
            req_stated_undeclared = true;
            continue;
        }
        if !bare_references(capabilities, trimmed).is_empty() {
            continue;
        }
        offences.push(format!(
            "{spec}:{line_no} states a bound outside any declared bound scenario, so it is absent from the \
             register: {}",
            truncate_chars(trimmed, 108)
        ));
    }
    flush_requirement!();
    offences
}

/// One `PINNED-BY` citation, wherever in the tracked specs it was written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinningCitation {
    /// The spec file it was read from.
    pub spec: String,
    /// One-based line of the citation itself, so a refusal points at the line to fix.
    pub line: usize,
    /// The cited test name, backticks stripped, exactly as written.
    pub name: String,
    /// The bound this citation defends, where its scenario declares one. `None` for a citation under an
    /// ordinary scenario, which cites evidence without declaring a bound.
    pub bound: Option<String>,
}

/// Every declaration-form `PINNED-BY` citation in the tracked specs, **wherever it appears**.
///
/// The register reads citations only under a heading [`marks_a_bound`] accepts. That is the right corpus for
/// the register — a citation under an ordinary scenario declares no bound, and admitting it would invent one —
/// and the wrong corpus for **resolution**. The marker means one thing in both places: *this test is the
/// evidence*. A renamed or deleted test leaves an ordinary scenario citing nothing exactly as silently as it
/// would a bound, and the reader has no way to tell which sense a given line was written in.
///
/// Measured: 70 of the 75 citations in the tracked specs sat under bound headings and were resolved, while 5
/// under ordinary scenario headings were parsed by nothing — renaming one of the five left the entire gate
/// suite green with the spec citing a function that no longer existed.
///
/// `observation-bound-register` already states this rule for the sibling marker: a reference is resolved
/// wherever it appears, independent of whether its line also states a bound. This is that rule, for this one.
pub fn pinning_citations(root: &Path) -> Vec<PinningCitation> {
    let mut found = Vec::new();
    for (capability, spec) in tracked_specs(root) {
        let text = std::fs::read_to_string(root.join(&spec)).unwrap_or_else(|err| {
            panic!(
                "could not read the citations from {spec}: {err} — a spec this check cannot parse leaves \
                 them unresolved rather than resolved"
            )
        });
        found.extend(citations_in(&capability, &spec, &text));
    }
    assert!(
        !found.is_empty(),
        "parsed 0 pinning citations across the tracked specs — the declaration form changed, so this check \
         cannot judge rather than reporting every citation resolved"
    );
    found
}

/// Every `PINNED-BY` citation in one spec's already-read `text`, wherever it appears, with the bound it
/// defends where its enclosing scenario declares one.
///
/// Extracted so a caller holding `text` from somewhere other than the worktree — `HEAD` via `git show`, for
/// instance — gets the identical recognition [`pinning_citations`] uses, rather than a second hand-written
/// scanner over the same `#### Scenario:`/`- **PINNED-BY**` grammar that could drift from it.
pub fn citations_in(capability: &str, spec: &str, text: &str) -> Vec<PinningCitation> {
    let mut found = Vec::new();
    let mut bound: Option<String> = None;
    for (index, raw) in text.lines().enumerate() {
        let trimmed = raw.trim();
        if let Some(heading) = raw.strip_prefix("#### Scenario:") {
            let heading = heading.trim();
            bound = marks_a_bound(heading).then(|| format!("{capability}/{}", slug_of(heading)));
            continue;
        }
        // Any other heading ends the scenario, so a citation below it belongs to no scenario at all —
        // including a bare `#### ` heading not spelled `Scenario:`, matching `bounds_in`'s own body-scan
        // stopping rule (which also treats any `#### `/`### `/`## ` as ending the scenario body) exactly.
        // Checking only `### `/`## ` here once left a citation after such a heading still attributed to
        // whichever bound scenario opened above it, disagreeing with `bounds_in` about which bound (if
        // any) that citation defends. No tracked spec currently has a `#### ` heading spelled any other
        // way, so this was latent rather than observed.
        if ends_scenario(trimmed) {
            bound = None;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("- **PINNED-BY** ") {
            found.push(PinningCitation {
                spec: spec.to_string(),
                line: index + 1,
                name: rest.trim().trim_matches('`').to_string(),
                bound: bound.clone(),
            });
        }
    }
    found
}

/// Every bound declared across the tracked specs, in one enumeration.
///
/// One parse, shared by the register check and the census, because a census is produced by the check that
/// enumerates the set — see this module's own doc for why a second parse is not an option.
pub fn parse_bounds(root: &Path) -> Vec<Bound> {
    let mut bounds = Vec::new();

    for (capability, spec) in tracked_specs(root) {
        let text = std::fs::read_to_string(root.join(&spec)).unwrap_or_else(|err| {
            panic!(
                "could not read the declared bounds from {spec}: {err} — a spec this check cannot parse \
                 leaves the register undecided rather than clean"
            )
        });
        bounds.extend(bounds_in(&capability, &spec, &text));
    }

    assert!(
        !bounds.is_empty(),
        "parsed 0 declared bounds across the tracked specs — the heading form changed, so this check \
         cannot judge rather than reporting a register of nothing as clean"
    );
    bounds
}

/// Whether a character can appear inside a path-like word.
///
/// The run is what makes a path safe from being mistaken for a reference: reading maximal runs, the token in
/// `openspec/specs/repository-checks/spec.md` is the whole path and carries three slashes, so it is not a
/// `<capability>/<slug>` pair and is excluded by construction rather than by an exception list. A substring
/// search would find `repository-checks/spec.md` inside it and refuse a path for resembling a reference — the
/// false refusal this repository forbids its gates.
fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | '-')
}

/// Whether `slug` is the kebab-case form a derived bound id carries.
fn is_kebab(slug: &str) -> bool {
    !slug.is_empty()
        && slug.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}

/// Every **bare** `<capability>/<slug>` this text carries, as `(line number, id)`.
///
/// The `(bound: …)` form clears prose; this resolves the **id**, which is no less a reference for being
/// written without the wrapper. Both defects that motivated it sat in a doc comment above the very test
/// defending the bound, where the bijection cannot look — it compares the two declaration sides and a doc
/// comment is neither.
///
/// `capabilities` is **enumerated by the caller** from the tracked specs, never listed here: a capability
/// added later must be recognized without this function being touched, which is the register's own
/// prohibition against a hand-kept membership beside its enumerator.
///
/// This resolves nothing by itself — it reports what looks like a reference, and the caller holds those
/// against the produced id set. Recognition and resolution are kept apart so the bare form cannot grow a
/// second opinion about which ids exist.
pub fn bare_references(capabilities: &BTreeSet<String>, text: &str) -> Vec<(usize, String)> {
    let mut found = Vec::new();
    for (index, line) in text.lines().enumerate() {
        let mut rest = line;
        while let Some(start) = rest.find(is_word_char) {
            let tail = &rest[start..];
            let end = tail.find(|c| !is_word_char(c)).unwrap_or(tail.len());
            let (word, remainder) = tail.split_at(end);
            rest = remainder;
            let Some((capability, slug)) = word.split_once('/') else {
                continue;
            };
            if slug.contains('/') || !capabilities.contains(capability) || !is_kebab(slug) {
                continue;
            }
            found.push((index + 1, word.to_string()));
        }
    }
    found
}

/// Whether a generated projection carries exactly the ids a derivation produced, naming every id only one
/// side has.
///
/// The projection arrives as **text** so a stale one can be constructed. That is the point: what this
/// comparison catches is held by a case rather than by an argument, and the property survives however many
/// implementations the slug rule has, because the comparison is a derived set against a tracked file and not
/// against a second computation.
///
/// Reading the projection *instead* of deriving stays rejected: `cargo test` runs before the register gate in
/// the Definition of Done, so a stale projection would let the bijection pass while the specs and the code
/// disagreed.
pub fn projection_offences(derived: &BTreeSet<String>, projection: &str) -> Vec<String> {
    let projected: BTreeSet<&str> = projection
        .lines()
        .filter_map(|line| line.strip_prefix("### `"))
        .filter_map(|rest| rest.strip_suffix('`'))
        .collect();
    let mut offences = Vec::new();
    for id in derived {
        if !projected.contains(id.as_str()) {
            offences.push(format!(
                "`{id}` is derived from the specs and is absent from the projection"
            ));
        }
    }
    for id in &projected {
        if !derived.contains(*id) {
            offences.push(format!(
                "`{id}` is in the projection and is derived from no spec"
            ));
        }
    }
    offences
}
