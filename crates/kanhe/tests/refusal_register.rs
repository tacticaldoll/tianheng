//! Repository check: every registered refusal site is observed by a direction, and the unregistered ones
//! are counted.
//!
//! `AGENTS.md` states that **a guard is not a guard until it has been seen to fail**, and
//! `repository-checks` requires every refusal a check holds to have been run against a tree carrying the
//! shape it refuses. That requirement's scenarios held only its other clause — where a check may live — so
//! the half about refusals was carried by reviewer attention, and attention failed three times in one
//! window: a title guard with no negative run, three example-pin branches, then four internal-pin branches
//! in the change immediately after the one whose own record names the class.
//!
//! **Why nothing held it before.** A refusal's identity lived only in its message, and a message is a
//! *template* while a direction asserts a *rendering* of it. Five textual predicates were written against
//! that gap and measured; each was wrong in a different direction, and no reading of text answers the
//! question — which is what `pin_bites` already says about whether a test bites. So the site travels in the
//! value, a direction names the site it observed, and the two are compared by running.
//!
//! **The migration is visible rather than instantaneous.** Rewriting every site at once would be one
//! unreadable change; `refusal::violation` and `refusal::cannot_judge` stay beside their `_at` siblings
//! while modules move across, and the projection below carries how many sites have not moved. Registering a
//! site is a commitment that a direction observes it — a registered site no direction names refuses here —
//! so coverage cannot lag behind the migration.

use kanhe::region::DO_NOT_EDIT;
use kanhe::region::Source;
use shengmo::workspace::MARKER;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

const PROJECTION: &str = "docs/refusal-register.md";

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join(PROJECTION).is_file() || root.join("AGENTS.md").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Tracked paths under `dir` ending in `.rs`, through git rather than a filesystem walk.
///
/// The corpus is what the repository tracks, for the reason every sibling here gives: an untracked scratch
/// copy of a gate file is not repository content and must not decide a verdict.
fn tracked(root: &Path, dir: &str) -> Vec<PathBuf> {
    let out = Command::new("git")
        .args(["ls-files", "-z", "--", dir])
        .current_dir(root)
        .output()
        .unwrap_or_else(|err| panic!("cannot enumerate {dir}: {err}"));
    assert!(
        out.status.success(),
        "git ls-files failed over {dir}, which is not the same fact as an empty directory: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout)
        .split('\0')
        .filter(|p| p.ends_with(".rs"))
        .map(|p| root.join(p))
        .collect()
}

/// The first string literal argument of each `call` in `text`, with the line it sits on.
///
/// Deliberately not a Rust parser: the argument is a literal by construction — the constructors take
/// `&'static str` — so the first quote after the opening parenthesis begins it.
fn first_literal_args(text: &str, call: &str) -> Vec<(String, usize)> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(offset) = text[at..].find(call) {
        let start = at + offset;
        at = start + call.len();
        // A call, not the tail of a longer identifier: `violation_at(` must not be read as `violation(`.
        // A `call` opening with `::` carries its own left boundary, so no check on the byte before it —
        // which is the last byte of the path, and always an identifier byte.
        if call.starts_with(|c: char| c.is_ascii_alphanumeric()) && start > 0 {
            let before = text.as_bytes()[start - 1];
            if before.is_ascii_alphanumeric() || before == b'_' {
                continue;
            }
        }
        let rest = &text[at..];
        let Some(open) = rest.find('"') else { continue };
        // Only a literal that opens the argument list, so a call whose first argument is an expression is
        // not read as though the next literal on the line were its site.
        if rest[..open].chars().any(|c| !c.is_whitespace()) {
            continue;
        }
        let Some(close) = rest[open + 1..].find('"') else {
            continue;
        };
        found.push((
            rest[open + 1..open + 1 + close].to_string(),
            text[..start].matches('\n').count() + 1,
        ));
    }
    found
}

/// `text` with its comments, string literals and imports removed, so neither a name in prose nor a name in
/// a `use` list is read as a construction.
///
/// The import line was the second way a count could be wrong: a module holding both forms names all four
/// constructors in one `use`, and the bare identifiers there were counted as two more unregistered sites —
/// a figure two above the truth in the module where the truth is what the migration is steering by.
fn executed_rust(text: &str) -> Executed {
    Executed(imports_and_rest(text).1)
}

/// [`Executed`] with the contents of every string literal removed as well — the corpus a **count** reads.
///
/// **A literal is noise to a count and identity to a parse, so they cannot share a corpus.** A registered
/// site *is* a string literal, which is what [`first_literal_args`] reads; a bare construction is an
/// identifier, and a `"violation"` written as a search term is not one. Stripping literals for both broke
/// the register's own raw-literal fixture, which is what says the two needs are different rather than one
/// reader used two ways.
fn countable(text: &str) -> Countable {
    // **Literals first, then comments — the other order desynchronises the scanner.** `Source::rust` cuts a
    // `//` wherever it sees one, and `region`'s own header records that this includes a `//` *inside* a
    // string literal. That truncation leaves an unmatched quote, after which a literal scanner reads code as
    // string and string as code for the rest of the file: measured, the register's own module reported
    // twelve constructions it does not make. Removing literals from the raw text first leaves the comment
    // rule nothing to truncate.
    Countable(imports_and_rest(&without_string_literals(text)).1)
}

/// A name inside a string literal is not a construction, and a name in code is.
///
/// **The direction the corpus move needed and did not have.** While this register read
/// `crates/kanhe/src` its executed count was zero, so a literal contributed nothing and the unimplemented
/// half of the cleaning never showed. Moving the corpus to the test targets made it load-bearing — the
/// figure came out well above the truth, and the largest contributor was this file, which passes the
/// constructor names to [`calls`] as search terms.
#[test]
fn a_name_written_as_a_literal_is_not_a_construction() {
    let counted = |text: &str| {
        let countable = countable(text);
        calls(&countable, "violation") + calls(&countable, "cannot_judge")
    };
    assert_eq!(counted("fn f() { violation(x); }"), 1, "a call is counted");
    assert_eq!(
        counted("fn f() { let s = \"violation\"; }"),
        0,
        "a literal is not"
    );
    assert_eq!(
        counted("fn f() { g(r#\"cannot_judge and violation\"#); }"),
        0,
        "a raw literal is not, whatever it holds"
    );
    assert_eq!(
        counted("fn f() { g([\"violation_at(\", \"cannot_judge_at(\"]); }"),
        0,
        "adjacent literals in one expression are each closed"
    );
    assert_eq!(
        counted("use kanhe::refusal::{cannot_judge, violation};\nfn f() { violation(x); }"),
        1,
        "an import names them and constructs nothing"
    );
    assert_eq!(
        counted("// violation\n/// cannot_judge\nfn f() { cannot_judge(x); }"),
        1,
        "a comment names them and constructs nothing"
    );
}

/// A corpus with comments, imports and string literals removed.
///
/// The second newtype, for the same reason as the first: three call sites cleaned their corpus and a fourth
/// did not, and a type is what made that impossible to forget. This one carries the *rule*, not only the
/// fact of having been called — the guarantee the first one could not give, since the defect it was added
/// for then moved one layer inside it.
struct Countable(String);

/// `text` with the contents of every string literal replaced by nothing.
///
/// **The half [`executed_rust`]'s doc claimed and its body did not do.** `Source::rust` cuts comments by a
/// token-start rule and `region`'s own header records that a string literal therefore survives it — a
/// residue it declares rather than closes. That was inert while this register read `crates/kanhe/src`,
/// where the executed count is zero; moving the corpus to the test targets made it load-bearing, and the
/// figure came out **40** against a true **18**. The largest single contributor was this file, which passes
/// `"violation"` and `"cannot_judge"` to [`calls`] as search terms and so counted its own arguments as
/// refusal constructions — the same self-reference it already guards against for `::expect(`.
///
/// A raw string is handled by its own arm: `r"…"`, `r#"…"#` and any hash count, since a `\` inside one
/// escapes nothing and a naive escape-aware scan would run past the closing quote.
fn without_string_literals(text: &str) -> String {
    let bytes: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        // A raw string opens with `r` then any number of `#` then `"`, and closes on `"` followed by the
        // same number of `#`.
        if c == 'r' && !(i > 0 && (bytes[i - 1].is_alphanumeric() || bytes[i - 1] == '_')) {
            let mut hashes = 0;
            let mut at = i + 1;
            while at < bytes.len() && bytes[at] == '#' {
                hashes += 1;
                at += 1;
            }
            if at < bytes.len() && bytes[at] == '"' {
                out.push_str("\"\"");
                at += 1;
                while at < bytes.len() {
                    if bytes[at] == '"'
                        && bytes[at + 1..]
                            .iter()
                            .take(hashes)
                            .filter(|h| **h == '#')
                            .count()
                            == hashes
                    {
                        at += 1 + hashes;
                        break;
                    }
                    at += 1;
                }
                i = at;
                continue;
            }
        }
        if c == '"' {
            out.push_str("\"\"");
            i += 1;
            while i < bytes.len() {
                if bytes[i] == '\\' {
                    i += 2;
                    continue;
                }
                if bytes[i] == '"' {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

/// A corpus with its comments and imports already removed.
///
/// **A newtype, because the one call site that forgot was the one that was wrong.** `calls` took a `&str`,
/// and three sites handed it `executed_rust(&text)` while a fourth handed it the file — which counted every
/// doc comment naming a constructor and rendered **18** into a projection whose header says every number in
/// it is produced. The true count for that corpus is zero. `region`'s own header makes this argument for
/// every other recognizer in this crate — *a corpus is never handed to a recognizer as `&str`* — and this
/// recognizer had not taken it.
struct Executed(String);

/// Whether `trimmed` opens a `use` item — including one carrying a visibility.
///
/// **The item, not one textual prefix.** `pub use` and `pub(crate) use` are imports and construct nothing,
/// and a reader matching `use ` alone left them in the executed text, counted the constructor names they
/// carry as calls, and refused a module over an import. That is an **over**-reaction, on the side of this
/// reader whose failures were supposed to be the loud harmless ones, and it is outside the bound declared
/// for the other side — which is about constructions the reader misses. A gate that refuses correct source
/// is a defect, not a limit.
///
/// The space after `use` is load-bearing: `impl Iterator<…> + use<'a>` is precise capturing rather than an
/// import, and this repository writes it.
fn opens_a_use(trimmed: &str) -> bool {
    let rest = match trimmed.strip_prefix("pub") {
        Some(after) => {
            let after = after.trim_start();
            match after.strip_prefix('(') {
                Some(scope) => match scope.find(')') {
                    Some(close) => scope[close + 1..].trim_start(),
                    None => return false,
                },
                None => after,
            }
        }
        None => trimmed,
    };
    rest.starts_with("use ")
}

/// A file's `use` **statements**, and everything else, split once.
///
/// **One implementation, because two readers ask this and one of them was wrong.** Where a `use` statement
/// ends is a fact about Rust, and it lived twice here: the alias detector accumulated to the `;` while its
/// neighbour, fifteen lines up and reading the same input, dropped the line that *opens* a statement and
/// kept every continuation. A wrapped import naming `cannot_judge_at` on a line of its own then counted as
/// a call with nothing to parse, and the register refused a module that constructs nothing — a shape
/// `cargo fmt` produces the moment an import list grows too wide, which would have put two gates in this
/// repository's Definition of Done in direct contradiction. Asked once now, by both.
fn imports_and_rest(text: &str) -> (Vec<String>, String) {
    let source = Source::of(text);
    let executed = source.rust();
    let mut imports = Vec::new();
    let mut rest = Vec::new();
    let mut open: Option<String> = None;
    for line in executed.lines() {
        let trimmed = line.trim_start();
        let statement = match open.as_mut() {
            Some(statement) => {
                statement.push(' ');
                statement.push_str(trimmed);
                statement
            }
            None if opens_a_use(trimmed) => {
                open = Some(trimmed.to_string());
                open.as_mut().expect("just assigned")
            }
            None => {
                rest.push(line);
                continue;
            }
        };
        if statement.contains(';') {
            imports.push(std::mem::take(statement));
            open = None;
        }
    }
    if let Some(unterminated) = open {
        imports.push(unterminated);
    }
    (imports, rest.join("\n"))
}

/// How many registered constructions in `text` this reader could not parse.
///
/// **Counted against the calls, because parsing alone cannot report what it did not see.** The parser reads
/// a direct call whose first argument is an ordinary quoted literal, and three shapes are not that: a
/// constructor taken by name and called through the binding, a wrapper whose site arrives as a parameter,
/// and a raw string literal. Each was invisible to *both* halves of this register — no parsed site, and not
/// counted as unregistered either, since the unregistered counter reads the site-less constructors — so a
/// real refusal site was neither held, declared, nor reported missing. Comparing the two readings is what
/// turns *did not see it* into *cannot answer for this module*.
fn unparsed_constructions(text: &str) -> usize {
    let executed = executed_rust(text);
    let called =
        calls(&countable(text), "violation_at") + calls(&countable(text), "cannot_judge_at");
    let parsed = first_literal_args(&executed.0, "violation_at(").len()
        + first_literal_args(&executed.0, "cannot_judge_at(").len();
    called.saturating_sub(parsed)
}

/// Whether `text` imports a refusal constructor under another name.
///
/// A reader that matches names cannot follow an alias: `use crate::refusal::cannot_judge as cj;` makes every
/// later `cj(…)` invisible, and invisible reads as *this module constructs no refusal*. The corpus written
/// for this reader names that case, and the answer is not a count — it is that this file cannot be counted,
/// which is the same distinction between *disagrees* and *could not be read* that every gate here draws.
fn aliases_a_constructor(text: &str) -> bool {
    imports_and_rest(text).0.iter().any(|statement| {
        statement.contains(" as ")
            && (statement.contains("violation") || statement.contains("cannot_judge"))
    })
}

/// Whether `text` has a site identity's shape: `<capability>#<slug>`, lowercase and hyphenated.
///
/// **`#` and not `/`, because `<capability>/<slug>` is already an identity here.** The bound register
/// resolves that spelling anywhere in tracked Rust or Markdown as a reference to a declared observation
/// bound, so the first draft of these identities was read as ten references to bounds that do not exist —
/// measured, against this repository, before the shape was settled. A refusal site and an observation bound
/// are opposite facts, one about what is observed and one about what is not, and giving the new one its own
/// separator leaves the older reader's floor exactly where it was.
fn is_a_site(text: &str) -> bool {
    let Some((capability, slug)) = text.split_once('#') else {
        return false;
    };
    let word = |part: &str| {
        !part.is_empty()
            && part
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    word(capability) && word(slug)
}

/// Occurrences of the constructor named `name` in `text`, however it is reached.
///
/// **The identifier, not `name(`.** Counting the call syntax missed a constructor used as a value:
/// `workspace_version(repo).map_err(cannot_judge)` has no opening parenthesis after the name, so a live
/// refusal site was invisible to the register built to count them — found by migrating the module the site
/// was in, and only because the compiler then objected to the import. A register whose corpus reader can be
/// stepped around by a point-free call is one that reports a smaller number than the truth, which is the
/// direction that matters.
///
/// Both boundaries, so `cannot_judge_at` is not counted as `cannot_judge`.
fn calls(countable: &Countable, name: &str) -> usize {
    let text = &countable.0;
    let boundary = |byte: u8| !(byte.is_ascii_alphanumeric() || byte == b'_');
    let mut count = 0;
    let mut at = 0;
    while let Some(offset) = text[at..].find(name) {
        let start = at + offset;
        at = start + name.len();
        let before = if start == 0 {
            b' '
        } else {
            text.as_bytes()[start - 1]
        };
        let after = text.as_bytes().get(at).copied().unwrap_or(b' ');
        // **A definition is not a construction.** `fn violation(…) -> Refusal` declares the constructor and
        // `fn violation(target, rule, …) -> Violation` merely shares its name; counting either as a use
        // reported a module that constructs nothing as constructing one. Both are named in the corpus
        // written for this reader, and both were read wrong until it was run.
        let defines = text[..start].trim_end().ends_with("fn");
        if boundary(before) && boundary(after) && !defines {
            count += 1;
        }
    }
    count
}

struct Register {
    /// Registered site to the module and line of each branch producing it.
    registered: BTreeMap<String, Vec<(String, usize)>>,
    /// Registered site to whether it is a violation, as against a cannot-judge.
    disagrees: BTreeMap<String, bool>,
    /// Sites still constructed through the unregistered form, by module.
    unregistered: BTreeMap<String, usize>,
    /// Sites named by a direction, to the test files naming them.
    cited: BTreeMap<String, BTreeSet<String>>,
}

fn read(root: &Path) -> Register {
    let mut registered: BTreeMap<String, Vec<(String, usize)>> = BTreeMap::new();
    let mut disagrees: BTreeMap<String, bool> = BTreeMap::new();
    let mut unregistered: BTreeMap<String, usize> = BTreeMap::new();
    for path in tracked(root, "crates/kanhe/src") {
        let text = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .into_owned();
        // The constructors themselves are declarations, not sites.
        if name.ends_with("src/refusal.rs") {
            continue;
        }
        // **An alias makes this reader unable to answer, which is not the same as answering zero.** Every
        // call through `use … as cj` is invisible to a reader that matches names, so a module that aliases a
        // constructor would be counted as constructing none. Refused rather than counted, in the class this
        // repository reserves for a source it could not read.
        assert_eq!(
            unparsed_constructions(&text),
            0,
            "{name} constructs a registered refusal in a shape this register cannot read — a site taken by \
             name, arriving as a parameter, or written as a raw literal is invisible to both halves of it"
        );
        assert!(
            !aliases_a_constructor(&text),
            "{name} imports a refusal constructor under another name, so every construction through that \
             alias is invisible to this register — which would read as a module that constructs none"
        );
        for call in ["violation_at(", "cannot_judge_at("] {
            for (site, line) in first_literal_args(&text, call) {
                disagrees.insert(site.clone(), call.starts_with("violation"));
                registered
                    .entry(site)
                    .or_default()
                    .push((name.clone(), line));
            }
        }
        // **Executed Rust, not the file.** Counting the bare identifier over the whole text counted every
        // doc comment naming a constructor — this repository's prose names them constantly, and the figure
        // jumped by four modules that construct no refusal at all. `region` is the module written so that
        // forgetting to ask was not possible, and the same reader the gates themselves use.
        let open = calls(&countable(&text), "violation") + calls(&countable(&text), "cannot_judge");
        if open > 0 {
            unregistered.insert(name, open);
        }
    }
    let mut cited: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for dir in ["crates/kanhe/tests", "crates/kanhe/src/tests"] {
        for path in tracked(root, dir) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
            let name = path
                .strip_prefix(root)
                .unwrap_or(&path)
                .to_string_lossy()
                .into_owned();
            // **By position and by shape, never by the bare name.** `Result::expect` is a method taking a
            // panic message, and this reader's own source is in the corpus it reads — so a citation is
            // recognised as a path-qualified call, `refusal::expect`, whose first argument has a site's
            // shape. Reading every `expect(` counted panic messages as citations, this file's own included.
            for (site, _) in first_literal_args(&text, "::expect(") {
                if is_a_site(&site) {
                    cited.entry(site).or_default().insert(name.clone());
                }
            }
        }
    }
    Register {
        registered,
        disagrees,
        unregistered,
        cited,
    }
}

/// A registered site names one branch, and no two branches share a name.
///
/// Identity that is not injective is the defect this repository has already recorded once, where a
/// per-item finding not qualified by its owner let a baseline mask a new violation. Here a shared slug
/// would let one direction's citation vouch for a branch it never reached.
#[test]
fn a_registered_site_names_exactly_one_branch() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let shared: Vec<String> = register
        .registered
        .iter()
        .filter(|(_, sites)| sites.len() > 1)
        .map(|(slug, sites)| {
            let at: Vec<String> = sites
                .iter()
                .map(|(module, line)| format!("{module}:{line}"))
                .collect();
            format!("  {slug} — {}", at.join(", "))
        })
        .collect();
    assert!(
        shared.is_empty(),
        "these site identities name more than one branch, so a direction citing one vouches for the \
         others:\n{}",
        shared.join("\n")
    );
}

/// A site's capability half names a capability this repository specifies.
#[test]
fn a_registered_site_is_owned_by_a_capability() {
    let Some(root) = workspace_root() else {
        return;
    };
    let capabilities: BTreeSet<String> = entries_of(&root.join("openspec/specs"))
        .into_iter()
        .filter(|entry| entry.path().join("spec.md").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    assert!(
        !capabilities.is_empty(),
        "found no capability with a spec.md, so this direction would hold over nothing"
    );
    let register = read(&root);
    let orphans: Vec<String> = register
        .registered
        .keys()
        .filter(|slug| {
            !slug.split_once('#').is_some_and(|(capability, rest)| {
                capabilities.contains(capability) && !rest.is_empty()
            })
        })
        .cloned()
        .collect();
    assert!(
        orphans.is_empty(),
        "these site identities name no specified capability, so nothing owns the refusal they \
         identify:\n  {}",
        orphans.join("\n  ")
    );
}

/// Every registered site is observed by a direction, and every citation names a registered site.
///
/// Both ways, because either alone is satisfiable by doing nothing: a register nobody cites passes the
/// first direction of a one-way check, and a citation of a site that no longer exists passes the other.
#[test]
fn a_registered_site_and_the_directions_that_observe_it_agree() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    assert!(
        !register.registered.is_empty(),
        "found no registered refusal site, so this comparison would hold over nothing"
    );
    let declared: BTreeSet<&str> = kanhe::refusal_bounds::unheld()
        .into_iter()
        .map(|entry| entry.site)
        .collect();
    let unobserved: Vec<&String> = register
        .registered
        .keys()
        .filter(|slug| !register.cited.contains_key(*slug))
        .filter(|slug| !declared.contains(slug.as_str()))
        .collect();
    assert!(
        unobserved.is_empty(),
        "these refusal sites are registered, no direction observes them, and nothing declares them \
         unheld — a site is one or the other:\n  {}",
        unobserved
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>()
            .join("\n  ")
    );
    let dangling: Vec<String> = register
        .cited
        .iter()
        .filter(|(slug, _)| !register.registered.contains_key(*slug))
        .map(|(slug, files)| {
            format!(
                "  {slug} — cited by {}",
                files.iter().cloned().collect::<Vec<_>>().join(", ")
            )
        })
        .collect();
    assert!(
        dangling.is_empty(),
        "these directions cite a site no refusal produces, so they assert nothing about this \
         repository:\n{}",
        dangling.join("\n")
    );
}

/// A site declared unheld exists, and no direction observes it.
///
/// Both ways. A declaration naming no site is prose about nothing — the drift this whole register was built
/// to end, one level up. And a declared site that a direction *does* observe is held: saying otherwise
/// understates the coverage, and the honest record is the one a reader can act on.
#[test]
fn a_site_declared_unheld_exists_and_is_not_observed() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let declared = kanhe::refusal_bounds::unheld();
    assert!(
        !declared.is_empty(),
        "nothing is declared unheld, so this comparison would hold over nothing"
    );
    let orphans: Vec<&str> = declared
        .iter()
        .map(|entry| entry.site)
        .filter(|site| !register.registered.contains_key(*site))
        .collect();
    assert!(
        orphans.is_empty(),
        "these declarations name a refusal site nothing produces:\n  {}",
        orphans.join("\n  ")
    );
    let observed: Vec<&str> = declared
        .iter()
        .map(|entry| entry.site)
        .filter(|site| register.cited.contains_key(*site))
        .collect();
    assert!(
        observed.is_empty(),
        "these sites are declared unheld and a direction observes them — they are held, and the \
         declaration understates what this repository has:\n  {}",
        observed.join("\n  ")
    );
    // **A violation may not be declared unheld.** The declaration exists because a refusal about the
    // *reading* failing can only be reached by breaking the machine, and its fixture would test that break.
    // A refusal about the **subject** has no such excuse: its fixture is the defect it names, and if the
    // shape cannot be built then the branch is not about the subject after all. Without this, *declare it*
    // is available to any branch whose fixture is merely inconvenient, which is the escape hatch this table
    // is otherwise unable to close — the split was measured before it was a rule, and it is a rule now.
    let disagreeing: Vec<&str> = declared
        .iter()
        .map(|entry| entry.site)
        .filter(|site| register.disagrees.get(*site).copied().unwrap_or(false))
        .collect();
    assert!(
        disagreeing.is_empty(),
        "these sites are declared unheld and refuse as a **violation** — a disagreement with the judged \
         subject, whose fixture is the defect it names:\n  {}",
        disagreeing.join("\n  ")
    );
    let twice: Vec<String> = {
        let mut seen = BTreeSet::new();
        declared
            .iter()
            .filter(|entry| !seen.insert(entry.site))
            .map(|entry| entry.site.to_string())
            .collect()
    };
    assert!(
        twice.is_empty(),
        "these sites are declared more than once, so which declaration owns them is not decided:\n  {}",
        twice.join("\n  ")
    );
}

/// Every entry of `dir`, or a loud failure naming what could not be read.
///
/// **An entry that fails after the directory itself opened is not an entry that is absent.** Both readers
/// here reached for `filter_map(|entry| entry.ok())`, which encodes a failure as a missing member — so an
/// unreadable capability directory would have reported an orphan site, and an unreadable fixture would have
/// reported a case nothing names. One rule, one implementation, and failure is a refusal rather than a
/// silence.
///
/// **It returns the entries, not their paths, and that is the second half of the same rule.** The first
/// draft returned `PathBuf`, which turns the infallible `DirEntry::file_name` into `Path::file_name`'s
/// `Option` — and the callers then defaulted it, encoding an absent name as an empty member. That is the
/// class this function exists to remove, reintroduced one call further along by the repair for it.
fn entries_of(dir: &Path) -> Vec<std::fs::DirEntry> {
    let listing = std::fs::read_dir(dir)
        .unwrap_or_else(|err| panic!("cannot enumerate {} ({err}), so this direction would hold over a corpus it could not read", dir.display()));
    listing
        .map(|entry| {
            entry.unwrap_or_else(|err| {
                panic!(
                    "an entry of {} could not be read ({err}); a member this reader could not open is \
                     not a member that is absent",
                    dir.display()
                )
            })
        })
        .collect()
}

/// No refusal site is untriaged.
///
/// **This is the teeth.** Every site is observed by a direction or declared unheld with an owner and a
/// tracker; *not yet looked at* is not a state this repository keeps. The declaration is the escape hatch
/// and it is deliberately expensive — typed, counted, projected, owned — but an escape hatch that nothing
/// forces you through is just the prose that drifted.
#[test]
fn no_refusal_site_is_untriaged() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let remaining: usize = register.unregistered.values().sum();
    assert_eq!(
        remaining,
        0,
        "these modules construct refusals that carry no identity, so nothing can say whether a direction \
         observes them:\n{}",
        register
            .unregistered
            .iter()
            .map(|(module, count)| format!("  {module} — {count}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The register, and how much of it has not moved yet, as a document rather than as a claim.
///
/// The count of unregistered sites is **produced**, which is the whole reason it can be trusted to fall: a
/// figure typed into prose is one nothing measures, and this repository has already spent a window
/// replacing those. A change in either direction has to be blessed, so a module migrating shows up here and
/// a new unregistered site cannot arrive quietly.
#[test]
fn the_register_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let register = read(&root);
    let remaining: usize = register.unregistered.values().sum();
    let declared = kanhe::refusal_bounds::unheld();
    // **The floor is computed, over the corpus where the constructions actually are.** The header used to
    // say every construction goes through the `_at` forms "since nothing else exists". `refusal::violation`
    // and `refusal::cannot_judge` do exist — they carry `Site::OutsideRegister` precisely because they take
    // no identity. The first repair of that sentence counted them over the RAW text of this register's own
    // corpus and rendered 18, every one of which was the English word in a doc comment; on executed Rust
    // that corpus holds none, which the count of unregistered sites beside it already said. The
    // constructions are in the test targets, which this register's corpus excludes.
    let mut out = format!(
        "# Refusal register\n\nEvery refusal site in this repository, and what holds it. A site is \
         registered by being constructed through `refusal::violation_at` or `refusal::cannot_judge_at`, and \
         **held** by a direction calling `refusal::expect` with the same identity, compared by running \
         rather than by reading a message.\n\n\
         **What this document does not claim.** `refusal::violation` and `refusal::cannot_judge` construct a \
         refusal carrying no site identity — `Site::OutsideRegister` — so this register does not see them. \
         Its corpus, `crates/kanhe/src`, holds **none** of them, which is the figure beside *carry no \
         identity at all* above. The test targets do hold them, and this corpus excludes those: none is \
         registered, held, or declared here, and whether any should have taken an identity is a judgement \
         this document does not make. **No count of them is given**, and that is a decision rather than an \
         omission — the figure was rendered three times from three readers and no two agreed, so it is a \
         census this register cannot produce and therefore does not claim.\n\n\
         A site that no direction holds is **declared unheld**, with why, an owner and a tracker, in the \
         table this register reads. There is no third state among *registered* sites: one is held or \
         declared, and the register refuses anything else.\n\nGenerated from `crates/kanhe/src/**.rs` by \
         `crates/kanhe/tests/refusal_register.rs`. **{DO_NOT_EDIT}** — regenerate with `BLESS=1 \
         {MARKER}=1 cargo test -p kanhe --test refusal_register`. A stale projection fails \
         that gate.\n\n"
    );
    out.push_str(&format!(
        "**{} of {} refusal sites are declared unheld.** {remaining} carry no identity at all, which is a \
         state this repository does not keep — the register refuses a non-zero figure here.\n\n",
        declared.len(),
        register.registered.len(),
    ));
    out.push_str("## Declared unheld\n\n");
    for entry in &declared {
        out.push_str(&format!(
            "### `{}`\n\n- because {}\n- owner: {:?}\n- tracked by {}\n\n",
            entry.site, entry.because, entry.owner, entry.tracker
        ));
    }
    out.push_str("## Held\n\n");
    for (slug, sites) in &register.registered {
        if declared.iter().any(|entry| entry.site == slug.as_str()) {
            continue;
        }
        let cited = register
            .cited
            .get(slug)
            .map(|files| files.iter().cloned().collect::<Vec<_>>().join(", "))
            .unwrap_or_default();
        // **The module, never the line.** A reference naming a position is refused in tracked content here,
        // and this document is tracked content: a line number is right at the moment it is written and wrong
        // after the next edit above it. The identity is the slug; the module says where to look.
        let modules: BTreeSet<&str> = sites.iter().map(|(module, _)| module.as_str()).collect();
        out.push_str(&format!(
            "### `{slug}`\n\n- produced in `{}`\n- observed by `{cited}`\n\n",
            modules.iter().copied().collect::<Vec<_>>().join("`, `")
        ));
    }
    // One trailing newline and no blank line before it, which is the whitespace this repository keeps — the
    // per-entry blocks are separated by one, and the last of them would otherwise leave two.
    while out.ends_with("\n\n") {
        out.pop();
    }
    tianheng::testing::assert_projection_matches(&root, PROJECTION, &out);
}

/// The reader, run over the corpus written for it.
///
/// **This corpus was tracked and unread.** `crates/kanhe/tests/fixtures/refusal_scan/` entered this
/// repository on 10 August and nothing referenced any of its fourteen cases. Three of them name holes this
/// reader hit the hard way and closed one at a time — a constructor taken by name, a longer identifier, a
/// comment — and running them is how the rest were found rather than rediscovered.
///
/// The cases are text rather than compiled units, because what is being tested is a reader over text and a
/// case that had to compile could not carry a second `Refusal` type or a half-written call.
#[test]
fn the_reader_answers_the_corpus_written_for_it() {
    let Some(root) = workspace_root() else {
        return;
    };
    let dir = root.join("crates/kanhe/tests/fixtures/refusal_scan");
    // Each case, and how many refusal constructions it holds. A **definition** of a constructor is not a
    // construction, and neither is a function that merely shares a name with one.
    let expected: &[(&str, usize)] = &[
        ("a_call_and_a_definition", 1),
        ("a_call_that_wraps", 1),
        ("a_comment", 0),
        ("a_constructor_taken_by_name", 1),
        ("a_longer_identifier", 0),
        ("an_unrelated_violation_builder", 0),
        ("a_second_cannot_judge_variant", 0),
        ("a_second_constructor", 0),
        ("a_second_constructor_wrapped", 0),
        ("a_second_refusal_type", 0),
        ("a_vocabulary_under_other_names", 0),
        ("naming_the_shared_kind", 0),
        ("two_on_one_line", 2),
    ];
    let read = |case: &str| {
        std::fs::read_to_string(dir.join(format!("{case}.rs.txt")))
            .unwrap_or_else(|err| panic!("cannot read the case {case}: {err}"))
    };
    let mut wrong = Vec::new();
    for (case, want) in expected {
        let source = read(case);
        let got =
            calls(&countable(&source), "violation") + calls(&countable(&source), "cannot_judge");
        if got != *want {
            wrong.push(format!("  {case}: expected {want}, read {got}"));
        }
    }
    assert!(
        wrong.is_empty(),
        "the reader disagrees with the corpus written for it:\n{}",
        wrong.join("\n")
    );

    // **The one case that is not a count.** An aliased import makes every later call invisible to a reader
    // that matches names, so the honest answer is that this file cannot be counted at all — which is a
    // different fact from counting zero, and the same distinction every gate in this repository draws.
    // The shapes a parser cannot read, answered as *cannot answer* rather than as zero.
    for case in [
        "a_siteful_constructor_taken_by_name",
        "a_siteful_call_that_wraps",
        "a_raw_literal_site",
    ] {
        assert_eq!(
            unparsed_constructions(&read(case)),
            1,
            "{case}: a registered construction this reader cannot parse went unreported"
        );
    }
    for case in [
        "two_on_one_line",
        "a_wrapped_import_that_constructs_nothing",
        "a_public_import_that_constructs_nothing",
        "a_scoped_public_import_wrapped",
    ] {
        assert_eq!(
            unparsed_constructions(&read(case)),
            0,
            "{case}: the controls — a construction this reader parses, and an import that constructs \
             nothing — must not be reported as unread"
        );
    }

    for case in ["an_aliased_import", "a_wrapped_aliased_import"] {
        assert!(
            aliases_a_constructor(&read(case)),
            "{case}: an aliased import went unnoticed, so every call through the alias would read as absent"
        );
    }
    assert!(
        !aliases_a_constructor(&read("a_call_and_a_definition")),
        "the control: a case with no alias must not be read as one"
    );

    // Every case is used, so a case added to the corpus and forgotten is reported rather than sitting unread
    // the way the whole corpus did.
    let cases: BTreeSet<String> = entries_of(&dir)
        .into_iter()
        .filter_map(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .strip_suffix(".rs.txt")
                .map(str::to_string)
        })
        .collect();
    let named: BTreeSet<String> = expected
        .iter()
        .map(|(case, _)| (*case).to_string())
        .chain(
            [
                "an_aliased_import",
                "a_wrapped_aliased_import",
                "a_siteful_constructor_taken_by_name",
                "a_siteful_call_that_wraps",
                "a_raw_literal_site",
                "a_wrapped_import_that_constructs_nothing",
                "a_public_import_that_constructs_nothing",
                "a_scoped_public_import_wrapped",
            ]
            .into_iter()
            .map(str::to_string),
        )
        .collect();
    assert_eq!(
        cases, named,
        "the corpus and the cases this direction runs disagree, so a case exists that nothing answers"
    );
}
