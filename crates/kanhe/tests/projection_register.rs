//! `projection-register`'s check: the inventory of this repository's generated documents.
//!
//! Four documents are blessed and diffed by something, and until this existed the list of them was prose in
//! `AGENTS.md` — written across two paragraphs at four different times, checked by nothing. A fifth check
//! could project a document, hold it fresh, be named nowhere, and no gate or test would find out. That is the
//! class the two changes before this one closed a level down, carried by the mechanism whose whole purpose is to
//! stop documents drifting.
//!
//! What it owns is one obligation the projections' own capabilities do not: that the **set** of such documents is
//! known, that each has a check behind its warning, and that a reader can find each one. Freshness stays with
//! each holder — a second implementation of that rule, inside the register built to end duplication, would refute
//! itself.

use shengmo::workspace::MARKER;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tianheng::testing::assert_projection_matches;

use kanhe::region::{DO_NOT_EDIT, Header, Prose, Source, declares_itself_generated};

/// This check's own projection, which is itself a member of the register.
const PROJECTION: &str = "docs/projection-register.md";

/// The file that **defines** the shared blessing rule is not a holder of anything.
///
/// Excluded by defining the rule rather than by name, so a move or a rename cannot turn the exclusion into a
/// silent gap — the shape a capability one change earlier needed for the library defining the exit contract.
const RULE_DEFINITION: &str = "pub fn assert_projection_matches";

/// Calling either is what makes a Rust unit a holder.
const RULE_CALLS: [&str; 2] = ["assert_projection_matches(", "assert_projection_fresh"];

/// The document a reader is told to open first, and therefore the one every projection must be named in.
const READERS_ENTRY_POINT: &str = "AGENTS.md";

/// The repository root, or `None` outside a checkout: the shared locator with this check\'s own
/// prerequisite.
fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(READERS_ENTRY_POINT).is_file() && root.join("docs").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

/// Tracked paths under `pathspec`, read with `-z` because `git ls-files` quotes a non-ASCII path by default.
fn tracked(root: &Path, pathspec: &str) -> Vec<String> {
    // **Not the shared reader, and the reason is a clause it does not carry.** `hermetic_git::read` converts
    // with `from_utf8_lossy`, which turns a path this enumeration cannot represent into replacement
    // characters and reports a name no file has. This one refuses instead, because the paths it returns are
    // compared against a register: a mangled name would read as a document that is not there. Converging it
    // was tried and reverted for exactly that clause.
    let output = kanhe::hermetic_git::hermetic("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", pathspec])
        .output()
        .unwrap_or_else(|err| panic!("cannot run `git ls-files` in {root:?}: {err}"));
    assert!(
        output.status.success(),
        "{}",
        kanhe::hermetic_git::failed(
            "`git ls-files`",
            &output.status.to_string(),
            &String::from_utf8_lossy(&output.stderr)
        )
    );
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            String::from_utf8(entry.to_vec())
                .unwrap_or_else(|err| panic!("a tracked path under {root:?} is not UTF-8: {err}"))
        })
        .collect()
}

/// A tracked text, as a [`Source`] rather than a `String`: the region a property is about is then decided in the
/// type, and a recognizer that wants executed text cannot be handed the whole file.
fn read(root: &Path, relative: &str) -> Source {
    let path = root.join(relative);
    Source::of(
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("cannot read {path:?}: {err}")),
    )
}

/// The tracked paths a document's header names.
///
/// Every candidate is checked against the tracked set rather than against a path-shaped pattern, so a glob
/// (`openspec/specs/*/spec.md`, which one header names as its *input*) names no file and drops out on its own.
fn paths_named_in_header(header: &Header<'_>, tracked_files: &BTreeSet<&str>) -> BTreeSet<String> {
    header
        .text()
        .split(|ch: char| ch.is_whitespace() || ch == '`' || ch == '(' || ch == ')')
        .map(|token| token.trim_matches(|ch: char| matches!(ch, '.' | ',' | ';' | ':' | '*' | '"')))
        .filter(|token| tracked_files.contains(*token))
        .map(str::to_string)
        .collect()
}

/// Whether a Rust unit holds a projection fresh — and not the unit that defines the rule.
///
/// The definition is recognized as a **line beginning** with the signature, not as a substring. The looser form
/// excluded *this file*, which names the signature in a constant and therefore contains it: a self-reference trap,
/// and the third in this capability after the specification quoting the marker it requires and the register having
/// to bless itself twice. The lesson each time is the same — when a check's subject is text, the check's own
/// text is part of the corpus, so recognize by position or shape rather than by the bare string.
fn defines_the_rule(source: &Source) -> bool {
    source.rust().starts_a_line_with(RULE_DEFINITION)
}

/// A holder calls the shared rule in **executed** text. A bare comment mentioning the call used to be enough:
/// measured, a `// … assert_projection_matches( …` line added to an unrelated file made it register as a holder.
/// How many projections a unit blesses: one per call to the shared rule, in executed text.
///
/// The count is the correspondence — see the caller for what counting does and does not reach.
fn blessing_call_sites(source: &Source) -> usize {
    if defines_the_rule(source) {
        return 0;
    }
    source
        .rust()
        .lines()
        .map(|line| {
            RULE_CALLS
                .iter()
                .map(|call| {
                    line.match_indices(call)
                        // A call site is not preceded by a quote. THIS file declares the call names as string
                        // data, so counting the bare string counted its own constant — the fourth self-reference
                        // trap in the 0.5.0 window, and the fourth time position rather than the string was the answer.
                        .filter(|(at, _)| *at == 0 || !line[..*at].ends_with('"'))
                        .count()
                })
                .sum::<usize>()
        })
        .sum()
}

fn holds_a_projection(source: &Source) -> bool {
    !defines_the_rule(source) && RULE_CALLS.iter().any(|call| source.rust().contains(call))
}

/// Whether `needle` appears in `text` outside every fenced block.
///
/// Prose is where a reader is sent; a fence is where a command lives. The first draft of this rule said "not in a
/// comment", carried over from the change before it without checking that it transfers — and it does not, because
/// Markdown has no `#` comment and cutting each line at its first `#` truncates every heading, including the
/// paragraph this check most depends on. The concern is live either way: one projection's path appears both in
/// prose and in a comment inside the Definition of Done fence.
fn mentions_in_prose(prose: &Prose<'_>, needle: &str) -> bool {
    prose.contains(needle)
}

/// One registered document: what it is, and the holder its header names.
struct Registered {
    document: String,
    generator: Option<String>,
    /// Every tracked path its header names, kept so an ambiguous header can be reported as ambiguous rather than
    /// resolved by picking one.
    named: BTreeSet<String>,
}

/// Every generated document, keyed by path.
///
/// Fails loudly on an empty enumeration: a correspondence between two empty sets holds while proving nothing, and
/// this is the direction this repository has re-opened most often.
fn registered(root: &Path) -> BTreeMap<String, Registered> {
    let all: Vec<String> = tracked(root, ".");
    let tracked_files: BTreeSet<&str> = all.iter().map(String::as_str).collect();

    let mut found = BTreeMap::new();
    for path in all.iter().filter(|path| path.ends_with(".md")) {
        let text = read(root, path);
        if !declares_itself_generated(&text.header()) {
            continue;
        }
        let named = paths_named_in_header(&text.header(), &tracked_files);
        // The document itself is often named in its own header's regeneration command; it is not its generator.
        let mut candidates: Vec<&String> = named.iter().filter(|name| *name != path).collect();
        let generator = match candidates.len() {
            1 => Some(candidates.remove(0).clone()),
            _ => None,
        };
        found.insert(
            path.clone(),
            Registered {
                document: path.clone(),
                generator,
                named,
            },
        );
    }
    assert!(
        !found.is_empty(),
        "no generated document found under {root:?}; the marker may have changed, and a correspondence over an \
         empty set holds while proving nothing"
    );
    found
}

/// Every unit that holds a projection fresh, from **both** mechanisms.
///
/// The shell holder is recognized as a `check_*` gate, never by mentioning `BLESS`: two scripts mention it and one
/// is the twin that *proves* the blessing behaves, writing no projection of its own. A twin is not a gate, so the
/// shape excludes it without an exclusion list.
fn holders(root: &Path) -> Vec<String> {
    let mut found: Vec<String> = tracked(root, ".")
        .into_iter()
        .filter(|path| {
            let is_rust = path.ends_with(".rs");
            let is_gate = path
                .rsplit_once('/')
                .is_some_and(|(dir, base)| dir == "scripts" && base.starts_with("check_"));
            // The shell arm has no possible instance: `git ls-files scripts/` names one unit and it is a
            // wrapper, not a `check_` gate. It is kept only to make the emptiness visible in the assertion
            // below rather than silently pruned — a recogniser arm that can never match is the vacuity this
            // register exists to report, and it is now reported.
            assert!(
                !is_gate,
                "a `scripts/check_*` unit exists again ({path}); this recogniser arm was retired as \
                 unreachable and its projection text says one mechanism is recognized"
            );
            if !is_rust {
                return false;
            }
            let text = read(root, path);
            holds_a_projection(&text)
        })
        .collect();
    found.sort();
    assert!(
        !found.is_empty(),
        "no unit holds a projection under {root:?}; the blessing rule may have moved, and a correspondence over \
         an empty set holds while proving nothing"
    );
    found
}

#[test]
fn every_generated_document_has_a_holder_and_every_holder_is_registered() {
    let Some(root) = workspace_root() else {
        return;
    };
    let documents = registered(&root);
    let holders = holders(&root);
    let holder_set: BTreeSet<&str> = holders.iter().map(String::as_str).collect();

    let mut offences = Vec::new();

    // Document → holder. A file saying "do not edit by hand" whose freshness nothing asserts is a
    // hand-maintained document wearing a generated one's warning: worse than plain prose, because a reader
    // trusts it more and no check defends it.
    for entry in documents.values() {
        match &entry.generator {
            None if entry.named.is_empty() => offences.push(format!(
                "{}: names no generator in its header, so its warning not to edit rests on nothing",
                entry.document
            )),
            None => offences.push(format!(
                "{}: its header names more than one tracked unit ({:?}), so which one holds it is ambiguous",
                entry.document, entry.named
            )),
            Some(generator) if !holder_set.contains(generator.as_str()) => offences.push(format!(
                "{}: names `{generator}`, which holds no projection — the document claims a freshness nothing \
                 asserts",
                entry.document
            )),
            Some(_) => {}
        }
    }

    // Holder → document, enumerated independently of what the documents claim. A document naming its generator is
    // a claim by the document; the call site is the fact.
    //
    // Counted per blessing CALL SITE, not per file. Measured defect: a second `assert_projection_matches` in an
    // existing holder, blessing a tracked document with no marker, was accepted in silence — the file was already
    // paired with its first document and nothing asked about the second.
    //
    // What the count does not reach, stated because it would otherwise read as a per-pair correspondence: which
    // call blesses which document is not resolved. The path is a constant in the source, and reading it would mean
    // evaluating Rust rather than reading it, so two holders that swapped which document they name would satisfy
    // this. A blessing nothing registers is caught; a permutation is not.
    for holder in &holders {
        let blessings = blessing_call_sites(&read(&root, holder));
        let registering: Vec<&str> = documents
            .values()
            .filter(|entry| entry.generator.as_deref() == Some(holder.as_str()))
            .map(|entry| entry.document.as_str())
            .collect();
        if registering.len() == 1 && blessings > 1 {
            offences.push(format!(
                "{holder}: blesses {blessings} projections and is registered by {} — a document it writes is \
                 unregistered, and the register cannot know which",
                registering.len()
            ));
            continue;
        }
        match registering.len() {
            1 => {
                // And the pair is tied from both sides: the holder must name the document it blesses, or the two
                // agree only by a claim made in one direction.
                let document = registering[0];
                let source = read(&root, holder);
                let names_document = if holder.ends_with(".rs") {
                    source.rust().contains(document)
                } else {
                    source.shell().contains(document)
                };
                if !names_document {
                    offences.push(format!(
                        "{holder}: registered as the generator of {document} and does not name it, so the pair \
                         rests on the document's word alone"
                    ));
                }
            }
            0 => offences.push(format!(
                "{holder}: holds a projection that no generated document registers — the document exists and the \
                 register does not know it"
            )),
            _ => offences.push(format!(
                "{holder}: registered by more than one document ({registering:?}), so at most one of them is right"
            )),
        }
    }

    assert!(
        offences.is_empty(),
        "the projection register and the checks holding those projections disagree:\n{}",
        offences.join("\n")
    );
}

#[test]
fn every_generated_document_is_reachable_from_where_a_reader_is_sent() {
    let Some(root) = workspace_root() else {
        return;
    };
    let entry_point = read(&root, READERS_ENTRY_POINT);
    let documents = registered(&root);
    let unreachable: Vec<&str> = documents
        .keys()
        .filter(|document| !mentions_in_prose(&entry_point.prose(), document))
        .map(String::as_str)
        .collect();
    assert!(
        unreachable.is_empty(),
        "a generated document is named nowhere a reader is sent: {unreachable:?} — {READERS_ENTRY_POINT} must \
         name each one in its prose, because the register knowing a document exists is not the same as a reader \
         being able to find it (a mention inside a fenced block or an HTML comment does not count — neither is prose a reader sees)"
    );
}

#[test]
fn the_register_includes_itself() {
    let Some(root) = workspace_root() else {
        return;
    };
    let documents = registered(&root);
    let own = documents.get(PROJECTION).unwrap_or_else(|| {
        panic!(
            "{PROJECTION} is not in its own register — a register whose figure counts documents cannot exempt \
             the document doing the counting"
        )
    });
    assert_eq!(
        own.generator.as_deref(),
        Some("crates/kanhe/tests/projection_register.rs"),
        "the register's own projection must name this check as its generator"
    );
}

#[test]
fn an_empty_surface_fails_rather_than_reporting_clean() {
    let Some(_root) = workspace_root() else {
        return;
    };
    // A repository with the layout and no generated document. Every property of zero documents holds, so a
    // check that did not refuse here would report the register complete.
    let fixture = std::env::temp_dir().join(format!(
        "tianheng-projection-register-empty-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&fixture);
    xingbiao::claim_scratch(&fixture).expect("the fixture directory is writable");
    std::fs::create_dir_all(fixture.join("docs")).expect("the fixture directory is writable");
    std::fs::write(fixture.join(READERS_ENTRY_POINT), "# AGENTS\n").expect("writable");
    // Joined from components, and the fixture document below is named without a directory, because
    // `crates/kanhe/tests/reference_integrity.rs` reads a repository-shaped path in a Rust line comment as a
    // claim that the file exists — and these exist only inside a temporary directory this test builds and
    // removes.
    std::fs::write(
        fixture.join("docs").join("notes.md"),
        "# Notes\n\nHand written.\n",
    )
    .expect("writable");
    // Through the shared fixture builder rather than a bare spawn: `init.templateDir` and a global
    // `core.excludesFile` both reach `init` and `add`, so a fixture built bare inherits the machine being
    // judged — and `add -A` is exactly the verb an ambient excludes file changes the result of.
    for arguments in [["init", "-q"], ["add", "-A"]] {
        kanhe::hermetic_git::fixture(&fixture, "git", &arguments);
    }

    let refused = std::panic::catch_unwind(|| registered(&fixture));
    let _ = std::fs::remove_dir_all(&fixture);
    let message = refused
        .err()
        .map(|payload| {
            payload
                .downcast_ref::<String>()
                .cloned()
                .or_else(|| payload.downcast_ref::<&str>().map(ToString::to_string))
                .unwrap_or_default()
        })
        .unwrap_or_else(|| {
            panic!("an enumeration yielding zero documents must refuse, not report clean")
        });
    assert!(
        message.contains("no generated document found"),
        "the refusal must name the emptiness rather than fail incidentally, got: {message}"
    );
}

#[test]
fn the_projection_register_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let documents = registered(&root);
    let holders = holders(&root);
    assert_projection_matches(&root, PROJECTION, &render(&documents, &holders));
}

/// The projection: every generated document, the check that holds it fresh, and the command its header names.
///
/// The command is **printed rather than checked** — see the declared bound below — and saying so beside it is the
/// point of printing it at all.
fn render(documents: &BTreeMap<String, Registered>, holders: &[String]) -> String {
    let mut out = String::new();
    out.push_str("# The projection register\n\n");
    out.push_str(
        "Every generated document in this repository, the check that holds it fresh, and the command its own\n\
         header names for regenerating it. Enumerated from tracked content by the marker each document carries, so\n\
         a document enters this table the moment it declares itself generated.\n\n",
    );
    out.push_str(&format!(
        "Generated by `crates/kanhe/tests/projection_register.rs`. **{DO_NOT_EDIT}** — regenerate with\n\
         `BLESS=1 {MARKER}=1 cargo test -p kanhe --test projection_register`.\n\n"
    ));
    out.push_str("## What registration in this table does not mean\n\n");
    out.push_str(
        "**Not freshness.** Each document's own holder asserts that its content matches what the code would\n\
         produce, and duplicating that here — inside the register built to end duplication — would refute itself.\n\
         This table says a document is known, has a check behind its warning, and can be found.\n\n",
    );
    out.push_str(
        "**Not that the command works.** The regeneration command in each header is registered and never run.\n\
         Verifying it would mean re-entering the `cargo test` harness already running, or letting this check\n\
         write into the tree it judges. A header naming a command that regenerates nothing is invisible here, and\n\
         that is a declared observation bound rather than an oversight.\n\n",
    );
    out.push_str(
        "**Not that the set is complete.** The recognized mechanism is a Rust call to the shared blessing\n\
         rule. A document generated some other way, whose author also omitted the marker, is absent from both\n\
         sides of the correspondence — a declared false negative owned by this engine, not a limit of what it\n\
         can read.\n\n\
         A second was recognized until the 0.5.0 window: a `check_*` gate writing its projection under `BLESS`. No\n\
         tracked unit is one, so that arm asserts its own emptiness rather than being pruned — if such a unit\n\
         exists again the check says so, which is the difference between retiring a recognizer and\n\
         forgetting it.\n\n",
    );

    out.push_str("## The register\n\n");
    out.push_str("| document | held fresh by | regenerate with |\n| --- | --- | --- |\n");
    for entry in documents.values() {
        let generator = match &entry.generator {
            Some(generator) => format!("`{generator}`"),
            None => "**unregistered**".to_string(),
        };
        out.push_str(&format!(
            "| `{}` | {generator} | {} |\n",
            entry.document,
            command_in(&entry.document)
        ));
    }
    out.push_str(&format!("\n{} documents.\n", documents.len()));

    out.push_str("\n## The checks holding them\n\n");
    out.push_str(
        "Enumerated independently of what the documents claim, because a document naming its generator is a claim\n\
         by the document and the call site is the fact:\n\n",
    );
    for holder in holders {
        out.push_str(&format!("- `{holder}`\n"));
    }
    out.push_str(&format!("\n{} checks.\n", holders.len()));
    out
}

/// The regeneration command a document's header names, rendered for the table.
///
/// Read out of the header as the first `BLESS`-prefixed span, and never executed. A document whose header names no
/// such command is reported as naming none rather than silently blank: the register's subject is what each header
/// claims, and a missing claim is itself the fact.
fn command_in(document: &str) -> String {
    let text = Source::of(
        std::fs::read_to_string(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(document),
        )
        .unwrap_or_default(),
    );
    for line in text.header().lines() {
        if let Some(start) = line.find("BLESS") {
            let rest = &line[start..];
            let command = rest.split('`').next().unwrap_or(rest);
            return format!("`{}`", command.trim().trim_end_matches(['.', ',']));
        }
    }
    "**names none**".to_string()
}

// --- this capability's own declared bounds, demonstrated ---

/// `projection-register/whether-a-stated-regeneration-command-regenerates-its-document-is-not-observed-a-stated-bound`
///
/// `OutOfReach`: the header is read and never evaluated. Running the command means re-entering the harness already
/// running, or — for the shell mechanism — writing the projection into the tree the check is judging, which is
/// the property this repository requires of every gate.
#[test]
fn a_regeneration_command_is_registered_and_never_run() {
    // A document whose header names a command that cannot regenerate anything. Every property this check
    // checks holds: it declares itself generated, and it names exactly one tracked generator.
    // The marker comes from the constant the recognizer reads, so this fixture is a member of the surface
    // by construction rather than by the two spellings happening to agree.
    let document = Source::of(format!(
        "# A fixture projection\n\n\
         Generated by `crates/kanhe/tests/projection_register.rs`. **{DO_NOT_EDIT}** — regenerate with\n\
         `BLESS=1 false`.\n\n## Body\n"
    ));
    assert!(
        declares_itself_generated(&document.header()),
        "the fixture must be a member of the surface, or this bound is demonstrated by a non-member"
    );
    let tracked_files: BTreeSet<&str> =
        BTreeSet::from(["crates/kanhe/tests/projection_register.rs"]);
    assert_eq!(
        paths_named_in_header(&document.header(), &tracked_files).len(),
        1,
        "the fixture must name exactly one generator, so nothing but the command is wrong with it"
    );
    // And the command is a command no run of it could satisfy. The check reports nothing about that: nothing
    // above is a function of it, which is what makes this a bound rather than a check.
    assert!(
        document.whole().contains("BLESS=1 false"),
        "the fixture must actually carry the defect this bound is about"
    );
}

/// `projection-register/a-document-generated-by-an-unrecognized-mechanism-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the **engine**. Not out of reach: the third mechanism's source sits in the tree this
/// check already reads. It is seen and not flagged, so the correspondence holds over a surface missing a
/// member — which is a false negative, and recording it as out-of-reach would be the misclassification
/// `observation-bound-model` exists to prevent.
#[test]
fn a_third_generation_mechanism_is_not_recognized() {
    // A unit that writes a document under a bless flag through neither recognized mechanism.
    let third_mechanism = Source::of(
        "fn main() {\n    \
         if std::env::var(\"BLESS\").is_ok() {\n        \
         std::fs::write(\"generated-elsewhere.md\", render()).unwrap();\n    }\n}\n",
    );
    assert!(
        !holds_a_projection(&third_mechanism),
        "the fixture must be invisible to the holder enumeration, or this bound is demonstrated by nothing"
    );
    // And its document, written without the marker, is invisible to the document enumeration too — so it is
    // absent from BOTH sides and the correspondence between them still holds.
    let its_document = Source::of("# Generated elsewhere\n\nWritten by a build step.\n\n## Body\n");
    assert!(
        !declares_itself_generated(&its_document.header()),
        "the fixture document must be outside the surface as well, which is what makes the pair invisible"
    );
}
