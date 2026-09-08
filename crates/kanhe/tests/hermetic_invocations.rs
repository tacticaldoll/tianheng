//! Repository check: a `git` this repository constructs itself is the builder's, or it is declared — and a
//! declared site's isolation is proven by a run.
//!
//! `kanhe::hermetic_git::hermetic` decides what a `git` behind a verdict may inherit — the configuration
//! files, the `GIT_CONFIG_*` channels, and `GIT_DIR`/`GIT_WORK_TREE`/`GIT_INDEX_FILE`, which move **which
//! repository** the command acts on and so reach past `current_dir` entirely. A caller that constructs its
//! own `Command::new("git")` inherits all of it.
//!
//! **This file answers two questions, and neither is whether a command is isolated.** It answers which
//! tracked files construct their own `git`, held two-directionally against a declared set; and, for a site
//! that cannot reach the builder, that the site names a proving direction the **harness lists**. Isolation
//! itself is what that direction's run says.
//!
//! **The split is what ten rounds of findings bought.** This check once decided isolation by reading the
//! source that builds a command — which methods, with which literals, on which array — and every round a
//! review supplied a spelling that reading missed: a rename, a macro, a literal compared by its rendering,
//! a constant kept while its loop was deleted, a removal made on a decoy receiver. Each is a different way
//! to write the same program, and a run does not care how the program is written. The environment model is
//! deleted rather than extended.
//!
//! **What remains is stated as a stop where it is one.** `constructs_git` reads literal constructions: a
//! program passed as a value, a name bound outside the file, a construction inside a comment or a string
//! literal are each a declared bound with a pinning direction, carried in
//! `docs/observation-bounds.md` rather than in this header.

use std::collections::BTreeSet;
use std::path::PathBuf;

/// What a declared site does about the environment a bare `git` inherits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Isolation {
    /// The builder itself, or a direction whose subject **is** what an un-isolated `git` answers.
    Exempt,
    /// A site that cannot reach the builder and proves its isolation by the named child-process direction,
    /// which injects one channel at a time and reads what that channel moves.
    ProvenBy(&'static str),
}

/// Every file that constructs a `git` without the builder, why, and what it does about the environment.
///
/// A site that gains one must be named here. A name that outlives its site must go: the comparison below is
/// two-directional for the reason `repository-checks` states for every constant a check judges by — a
/// one-directional comparison catches an omission and misses an entry that has outlived its subject.
const CONSTRUCTS_GIT_ITSELF: [(&str, Isolation, &str); 3] = [
    (
        "crates/kanhe/src/tests/hermetic_git.rs",
        Isolation::Exempt,
        "the builder's own directions, whose subject is what an un-isolated `git` answers — an isolated \
         control could not measure the difference the builder makes",
    ),
    (
        "crates/shengmo/tests/family_coverage.rs",
        Isolation::ProvenBy("no_ambient_channel_moves_what_the_family_coverage_builder_reads"),
        "boundary-forced: `shengmo` cannot reach `kanhe`, since `kanhe` depends on `shengmo` and the edge \
         would close a cycle",
    ),
    (
        "crates/shengmo/tests/examples_suite.rs",
        Isolation::ProvenBy("no_ambient_channel_moves_what_the_examples_suite_builder_reads"),
        "boundary-forced, for the same reason as its sibling above",
    ),
];

/// What this reader could decide about a file.
///
/// **Three states, because a file it cannot parse is not a file that constructs nothing.** The tokeniser's
/// failure arm fell back to an exact substring, which answers `false` for a construction split across lines
/// — so an unparseable file carrying one was reported clean, silently, which is the one direction the Core
/// Contract forbids.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Reading {
    Constructs,
    DoesNot,
    Undecidable,
}

/// A string literal's **value**, not its rendering.
///
/// `Literal::to_string` gives back the source spelling, so `r"git"` and `"\x67it"` — both of which decode to
/// `git` — compared unequal to `"git"`, and a construction written either way was not read.
fn literal_value(literal: &syn::Lit) -> Option<String> {
    match literal {
        syn::Lit::Str(text) => Some(text.value()),
        _ => None,
    }
}

/// That expression, where it is a string literal.
fn string_of(expression: &syn::Expr) -> Option<String> {
    match expression {
        syn::Expr::Lit(literal) => literal_value(&literal.lit),
        _ => None,
    }
}

/// **Read with a parser, not by counting tokens around a name.**
///
/// Four rounds of findings in this file were one cause, and it survived a repair: reading *lines* gave way
/// to reading *tokens*, and the readers stayed positional — `trees[index - 3]` for the owner segment, a
/// literal compared by its rendering, a value at `inner.get(2)` or dropped. Each round closed the instance
/// the review brought. `refusal_register`'s own header records this repository already learning it once:
/// a reader that is *text over Rust* is not exhaustive over the language, and reading its own Rust with a
/// real parser is what closed that floor. `syn` is already a dev-dependency here for exactly that.
///
/// A call's callee, its argument count and each argument's decoded value are what a parse gives; none of
/// them is an offset from something else.
#[derive(Default)]
struct Constructions {
    found: bool,
    /// A macro body neither grammar parsed, naming something bound to `Command`.
    undecided: bool,
    /// Names this file binds to `Command` — `use std::process::Command as Cmd` makes `Cmd` one.
    aliases: BTreeSet<String>,
}

/// Every name a file binds to `std::process::Command`, from **any** scope.
///
/// Collected from the top-level items alone, a `use std::process::Command as Cmd;` inside a function or an
/// inline module bound nothing and `Cmd::new("git")` was read as somebody else's `new`. A visitor reaches
/// every `use` wherever it stands.
///
/// **The path is checked, not only the rename.** Binding on the segment `Command` alone would make
/// `use foo::Command as Cmd` a `std::process::Command`, which widens the reaction rather than the bound: the
/// declared stop is a name bound *elsewhere*, not a name bound here to something else.
#[derive(Default)]
struct Aliases {
    names: BTreeSet<String>,
}

impl<'ast> syn::visit::Visit<'ast> for Aliases {
    fn visit_item_use(&mut self, node: &'ast syn::ItemUse) {
        Constructions::walk_use(&node.tree, &[], &mut self.names);
        syn::visit::visit_item_use(self, node);
    }
}

impl Constructions {
    /// Whether any token at any depth names something bound to `Command`.
    ///
    /// **At any depth.** Testing the immediate tokens alone, a body nesting the construction inside a
    /// delimiter — `passthrough!([Command::new("git")] => ())` — named nothing this reader saw, so an
    /// unclassifiable grammar carrying a construction was reported as carrying none.
    fn names_a_command(tokens: proc_macro2::TokenStream, aliases: &BTreeSet<String>) -> bool {
        let mut pending = vec![tokens];
        while let Some(stream) = pending.pop() {
            for tree in stream {
                match tree {
                    proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                    proc_macro2::TokenTree::Ident(word) if aliases.contains(&word.to_string()) => {
                        return true;
                    }
                    _ => {}
                }
            }
        }
        false
    }

    /// Every name this file may spell `Command` as, collected before the calls are read.
    ///
    /// **A rename is decidable inside one file and not outside it.** `use std::process::Command as Cmd`
    /// makes `Cmd::new("git")` the same construction, and reading only the segment `Command` missed it. What
    /// a rename *elsewhere* binds is not written down anywhere a parse tree carries — the floor
    /// `repository-checks/a-construction-shape-the-register-s-reader-does-not-model-a-stated-bound` already
    /// names for this repository's other reader of its own Rust, reached here by the same road.
    fn bind_aliases(&mut self, file: &syn::File) {
        self.aliases.insert("Command".to_string());
        let mut binder = Aliases::default();
        syn::visit::Visit::visit_file(&mut binder, file);
        self.aliases.extend(binder.names);
    }

    /// `prefix` carries the segments already walked, so the whole path decides rather than one segment.
    ///
    /// **The path, not a `process` somewhere in it.** Binding on *a segment named `process` occurred* made
    /// `use foo::process::Command as Cmd` a `std::process::Command`, which widens the reaction rather than
    /// the bound: the declared stop is a name bound **elsewhere**, not a name bound here to something else.
    /// A leading `::` and a `self`/`crate` prefix are skipped, and what remains must be exactly
    /// `std::process`.
    fn walk_use(tree: &syn::UseTree, prefix: &[String], aliases: &mut BTreeSet<String>) {
        match tree {
            syn::UseTree::Path(path) => {
                let mut deeper = prefix.to_vec();
                deeper.push(path.ident.to_string());
                Self::walk_use(&path.tree, &deeper, aliases);
            }
            syn::UseTree::Group(group) => {
                for item in &group.items {
                    Self::walk_use(item, prefix, aliases);
                }
            }
            syn::UseTree::Rename(renamed)
                if renamed.ident == "Command" && Self::is_std_process(prefix) =>
            {
                aliases.insert(renamed.rename.to_string());
            }
            _ => {}
        }
    }

    /// Whether the walked prefix is the **external** `std::process`.
    ///
    /// **A `crate::std` is not the standard library, and stripping the root said it was.** `self::` and
    /// `crate::` are lexical roots carrying meaning: `crate::std::process::Command` names a module this
    /// repository could define, and normalising them away merged it with the external crate — a false
    /// positive in the direction that reports a construction where there is none. `::std::process` is the
    /// absolute spelling of the same external path and is admitted; anything rooted at `self` or `crate` is
    /// not.
    fn is_std_process(prefix: &[String]) -> bool {
        // A leading `::` is held on the item rather than in the tree, so `::std::process` and
        // `std::process` reach this reader identically — both external, both admitted. Only a lexical root
        // that names *this* crate has to be refused.
        if matches!(
            prefix.first().map(String::as_str),
            Some("self") | Some("crate") | Some("super")
        ) {
            return false;
        }
        prefix == ["std".to_string(), "process".to_string()]
    }
}

impl<'ast> syn::visit::Visit<'ast> for Constructions {
    /// A construction inside a macro is read, by visiting the body the macro was handed.
    ///
    /// `dbg!(Command::new("git"))` is a construction the default walk never reaches, because a macro's
    /// tokens are not expressions until something parses them. The body is parsed as an expression list
    /// where it is one; a body that is not — an `assert!` with a message, a `matches!` pattern — carries no
    /// call this reader would have read anyway, and the file is not made undecidable for it.
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        use syn::punctuated::Punctuated;
        let as_expressions =
            node.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
        if let Ok(arguments) = as_expressions {
            for argument in &arguments {
                syn::visit::Visit::visit_expr(self, argument);
            }
        } else if let Ok(statements) = node.parse_body_with(syn::Block::parse_within) {
            // A statement-oriented body — `passthrough!(let _ = Command::new("git");)` — is not an
            // expression list, and discarding the failure silently let it carry a construction past this
            // reader.
            for statement in &statements {
                syn::visit::Visit::visit_stmt(self, statement);
            }
        } else if Self::names_a_command(node.tokens.clone(), &self.aliases) {
            // Neither grammar, and the body names something this file binds to `Command`. Saying so is the
            // safe direction: the alternative is reporting a file clean over tokens nothing classified.
            self.undecided = true;
        }
        syn::visit::visit_macro(self, node);
    }

    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = &*node.func {
            let segments: Vec<String> = callee
                .path
                .segments
                .iter()
                .map(|segment| segment.ident.to_string())
                .collect();
            let names_command = segments.len() >= 2
                && self.aliases.contains(&segments[segments.len() - 2])
                && segments[segments.len() - 1] == "new";
            if names_command
                && node.args.len() == 1
                && string_of(&node.args[0]).as_deref() == Some("git")
            {
                self.found = true;
            }
        }
        syn::visit::visit_expr_call(self, node);
    }
}

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("crates/kanhe/src/hermetic_git.rs").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Whether `text` constructs a `git`, or says it could not decide.
fn constructs_git(text: &str) -> Reading {
    let Ok(parsed) = syn::parse_file(text) else {
        return Reading::Undecidable;
    };
    let mut constructions = Constructions::default();
    constructions.bind_aliases(&parsed);
    syn::visit::Visit::visit_file(&mut constructions, &parsed);
    if constructions.found {
        Reading::Constructs
    } else if constructions.undecided {
        Reading::Undecidable
    } else {
        Reading::DoesNot
    }
}

/// The reading a direction asserts, so a tri-state reader is not flattened at every call site.
fn reads(text: &str) -> bool {
    matches!(constructs_git(text), Reading::Constructs)
}

#[test]
fn every_git_this_repository_constructs_is_the_builders_or_is_declared() {
    let Some(root) = workspace_root() else {
        return;
    };

    let tracked = kanhe::hermetic_git::tracked_paths(&root, &["*.rs"]).expect(
        "the tracked Rust is enumerable; a failed enumeration is not a repository with no sources",
    );
    let mut constructing = BTreeSet::new();
    let mut undecidable = Vec::new();
    let mut examined = 0usize;
    for path in &tracked {
        let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {err}"
            )
        });
        examined += 1;
        match constructs_git(&text) {
            Reading::Constructs => {
                constructing.insert(path.clone());
            }
            Reading::DoesNot => {}
            Reading::Undecidable => undecidable.push(path.clone()),
        }
    }
    assert!(
        examined > 0,
        "no tracked Rust was inspected, so this check would report clean over nothing"
    );
    // A file this reader cannot parse is not a file that constructs nothing. Its predecessor fell back to a
    // substring, which answers `false` for a construction split across lines — clean, silently, over a file
    // it never read.
    assert!(
        undecidable.is_empty(),
        "tracked Rust this reader could not decide, so whether it constructs a `git` was never \
         answered:\n  {}",
        undecidable.join("\n  ")
    );

    let declared: BTreeSet<String> = CONSTRUCTS_GIT_ITSELF
        .iter()
        .map(|(path, _, _)| (*path).to_string())
        .collect();
    assert_eq!(
        declared, constructing,
        "the files constructing a `git` differ from the set named here. A site that gains one must be \
         named with why; a name that outlives its site must go. {examined} tracked Rust file(s) were read"
    );
}

/// Every declared site's proving direction is one that **runs, and passes**.
///
/// **Executability is an execution result, and three readers of syntax said otherwise.** The first compared
/// `sig.ident` alone, so an ordinary function of the same name satisfied a citation nothing runs; the second
/// required `#[test]` and refused `#[ignore]`, and `#[cfg(any())] #[test]` — or an `ignore` reached through
/// `cfg_attr` — satisfied it while never entering the harness; the third asked the harness to **list** the
/// direction, and a listing carries ignored tests an ordinary run does not execute, under whichever features
/// the listing was taken with rather than the ones CI uses.
///
/// So the direction is run. `--all-features`, because that is what the Definition of Done and CI run;
/// `--include-ignored`, so a proof left ignored is still executed rather than silently skipped; and the run
/// must report exactly one test passing, because a filter that matches nothing also exits zero.
#[test]
fn every_declared_site_names_a_direction_that_runs_and_passes() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut missing = Vec::new();
    let mut checked = 0usize;
    for (path, isolation, _) in CONSTRUCTS_GIT_ITSELF {
        let Isolation::ProvenBy(direction) = isolation else {
            continue;
        };
        checked += 1;
        let (package, target) = declared_target(path);
        let out = std::process::Command::new(env!("CARGO"))
            .args([
                "test",
                "-p",
                package,
                "--test",
                target,
                "--all-features",
                "--",
                "--exact",
                direction,
                "--include-ignored",
            ])
            .current_dir(&root)
            .env(shengmo::workspace::MARKER, "1")
            .output()
            .unwrap_or_else(|err| panic!("cannot run {package}/{target}::{direction}: {err}"));
        let stdout = String::from_utf8_lossy(&out.stdout).into_owned();
        if !out.status.success() {
            // Labelled, and separated: trimmed and concatenated, a non-empty stdout's last line ran into a
            // non-empty stderr's first and the reader could not tell which stream said what.
            missing.push(format!(
                "  {path}: `{direction}` did not pass\n    stdout:\n{}\n    stderr:\n{}",
                stdout.trim_end(),
                String::from_utf8_lossy(&out.stderr).trim_end()
            ));
            continue;
        }
        // **One summary, parsed — not a token found somewhere in the output.** A filter matching nothing
        // exits zero, so a count is what says the proof ran; but `contains("1 passed")` is satisfied by any
        // process output carrying those words, including a build line or another target's summary, and it
        // cannot tell one passing test from one passing test beside a failure.
        match summary(&stdout) {
            Ok(Summary {
                passed: 1,
                failed: 0,
            }) => {}
            Ok(Summary { passed, failed }) => missing.push(format!(
                "  {path}: running `{direction}` reported {passed} passed and {failed} failed, where the \
                 proof is exactly one test passing"
            )),
            Err(NoSummary::Missing) => missing.push(format!(
                "  {path}: running `{direction}` produced no test summary, so nothing says it ran"
            )),
            Err(NoSummary::Multiple(count)) => missing.push(format!(
                "  {path}: running `{direction}` produced {count} test summaries, so which one carried the \
                 proof is not decidable from here"
            )),
        }
    }
    assert!(
        checked > 0,
        "no declared site names a proving direction, so this would hold over nothing — the state it was \
         written for is a copy whose isolation nothing runs"
    );
    assert!(
        missing.is_empty(),
        "a site names a direction that proves its isolation and running it does not:\n{}",
        missing.join("\n")
    );
}

/// The counts one libtest summary reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Summary {
    passed: usize,
    failed: usize,
}

/// Why a run carried no single summary.
///
/// **Two refusals, not one message.** Held as `None` for both, the report said *produced no test summary*
/// where two runs had produced two — a diagnostic naming the wrong fact about the tree, which is the class
/// this repository refuses in its own words.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NoSummary {
    /// Nothing in the output opened a line with a libtest summary.
    Missing,
    /// More than one did: two summaries are two runs, and which carried the proof is not decidable here.
    Multiple(usize),
}

/// The single libtest summary in `stdout`, or which of the two ways there was not one.
///
/// **Anchored, and exactly one.** Held as `contains("1 passed")`, the evidence was a token that any output
/// could carry — a build line, another target's summary, or a run of one passing test beside a failing one.
/// A summary opens its line, carries both counts, and there must be one of it.
fn summary(stdout: &str) -> Result<Summary, NoSummary> {
    let mut found = Vec::new();
    for line in stdout.lines() {
        let Some(rest) = line.trim_start().strip_prefix("test result: ") else {
            continue;
        };
        let counts = |name: &str| -> Option<usize> {
            rest.split(';')
                .find_map(|field| field.trim().strip_suffix(&format!(" {name}")))
                .and_then(|count| count.rsplit(' ').next())
                .and_then(|count| count.parse().ok())
        };
        let (Some(passed), Some(failed)) = (counts("passed"), counts("failed")) else {
            continue;
        };
        found.push(Summary { passed, failed });
    }
    match found.as_slice() {
        [] => Err(NoSummary::Missing),
        [only] => Ok(*only),
        many => Err(NoSummary::Multiple(many.len())),
    }
}

/// The `(package, test target)` a declared path names — `crates/<package>/tests/<target>.rs`.
fn declared_target(path: &str) -> (&str, &str) {
    let mut parts = path.split('/');
    let (Some("crates"), Some(package), Some("tests"), Some(file), None) = (
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
        parts.next(),
    ) else {
        panic!("a site proven by a direction lives in a test target: {path}");
    };
    (
        package,
        file.strip_suffix(".rs")
            .unwrap_or_else(|| panic!("a test target is a `.rs` file: {path}")),
    )
}

/// A `git` mentioned in prose is not a `git` constructed.
///
/// One of the two stops [`constructs_git`] makes, each now pinned by a direction named for it. They were one
/// test, and the bound for the **program-value** stop cited this name — a pin that resolves and exercises a
/// different stop, which is the class repaired one round earlier for
/// `a-paragraph-repeated-out-of-line-is-not-read`, in the round that repaired it.
#[test]
fn a_construction_named_in_prose_is_not_read() {
    assert!(!reads(
        "/// `Command::new(\"git\")` is what this forbids\nfn f() {}"
    ));
    assert!(!reads(
        "fn f() {\n    // a bare Command::new(\"git\") inherits the environment\n}"
    ));
    // **Every comment form, which is what asking a lexer buys.** A trimmed-line reader saw `//` and not
    // `/* … */`, so a block comment carrying the spelling was read as a construction — the prose bound
    // saying the opposite two files away.
    assert!(!reads(
        "fn f() {\n    /* a bare Command::new(\"git\") inherits the environment */\n}"
    ));
    assert!(!reads(
        "fn f() {\n    let x = 1; /* Command::new(\"git\") */\n}"
    ));
    assert!(reads("fn f() {\n    let out = Command::new(\"git\");\n}"));
}

/// A construction rustfmt split across lines is still a construction.
///
/// The other half of what reading lines cost: a substring over one trimmed line saw neither the opening nor
/// the argument, so a file whose construction wrapped was **not** in the constructing set and the check's
/// own requirement — that every file constructing a `git` is declared — was wider than its reader.
#[test]
fn a_construction_split_across_lines_is_read() {
    assert!(reads(
        "fn f() {\n    let out = Command::new(\n        \"git\",\n    );\n}"
    ));
    // The control: the same wrapping around a different program is not read, so the assertion above is
    // about the argument rather than about a reader that reports every `new`.
    assert!(!reads(
        "fn f() {\n    let out = Command::new(\n        \"cargo\",\n    );\n}"
    ));
    assert!(!reads("fn f() {\n    let out = Other::new(\"git\");\n}"));
}

/// A `git` constructed through a program value is not read.
///
/// The stop the bound of that name declares, pinned by a direction that exercises **it**. Whether a value
/// names `git` is not decidable from the line that constructs it, and the builder itself is written that way
/// — `Command::new(program)` — which is why the stop exists rather than being closed.
#[test]
fn a_construction_through_a_program_value_is_not_read() {
    assert!(!reads(
        "fn f() {\n    let mut command = Command::new(program);\n}"
    ));
    assert!(!reads("fn f() {\n    let out = Command::new(&exe);\n}"));
    // The control: the literal form in the same file is read, so the assertions above are about the value
    // rather than about a reader that reports nothing.
    assert!(reads(
        "fn f() {\n    let mut command = Command::new(\"git\");\n}"
    ));
}

/// A construction inside an ordinary string literal is not read.
///
/// The third stop, and the one that was mis-stated twice in a row: first as *a construction inside a string
/// literal is not read either*, which is false for a raw string; then, correcting that, as **not a stop at
/// all** — which threw away the half that is one. Both halves are real and they point opposite ways. This
/// one is the under-reaction, so it is the one declared:
/// `repository-checks/a-git-constructed-inside-a-string-literal-is-not-read-a-stated-bound`.
///
/// A literal is one expression, so what it carries is that expression's value and not a call — and a file
/// that writes Rust and compiles it is where that matters. Reading lines, this split in two by escaping;
/// read as syntax, both literal forms answer the same way and the raw-string over-report is gone.
///
/// **Both fixtures are assembled, and the raw one has to be.** This file is inside the corpus the live sweep
/// reads, so a raw string carrying the spelling verbatim would make this file report itself — measured, it
/// did. The alternatives were to declare this file exempt, which would blind the check to a real
/// construction added here later, or to leave the fixture written out and lose that. Assembling is not the
/// workaround `repeated_paragraph` records regretting: there it hid a defect being fixed, and here the
/// over-report is behaviour this direction exists to state.
#[test]
fn a_construction_inside_an_ordinary_string_literal_is_not_read() {
    // Assembled, because this file is inside the corpus the live sweep reads.
    let quote = '"';
    let ordinary = format!(
        "fn f() {{\n    let fixture = {quote}let out = Command::new(\\{quote}git\\{quote}){quote};\n}}"
    );
    assert!(
        !reads(&ordinary),
        "an ordinary literal escapes the quotes, so it carries a different text and drops out on its own"
    );
    // The control: the same line without the literal around it is read, so the assertion above is about the
    // escaping rather than about a reader that reports nothing.
    assert!(reads(&format!(
        "fn f() {{\n    let out = Command::new({quote}git{quote});\n}}"
    )));
}

/// A construction inside a **raw** string is not read either, which reading tokens closed.
///
/// It was an over-report while this file read lines: a raw string carries the spelling verbatim, so a
/// fixture written that way was reported as constructing a `git` — measured, by this file reporting itself.
/// A literal is one token, so asking a lexer answers both literal forms the same way and the over-report is
/// gone rather than declared. What remains is the one stop, in both forms, and the bound says so:
/// `repository-checks/a-git-constructed-inside-a-string-literal-is-not-read-a-stated-bound`, which cites
/// this direction beside its sibling.
///
/// **The citation is why deleting this is not silent.** The bound's subject was *either form* and its
/// citation named only the ordinary one, so this direction could be removed with the register, both
/// projections and every gate unchanged — the bound would have kept claiming a stop it had half the evidence
/// for. Two directions defending one bound is what `pinned_by_many` is for; two bounds would have declared
/// two stops where a lexer makes one.
///
/// Measured by renaming this function, which is what a deletion looks like to the register:
///
/// ```text
/// pinning citations that do not resolve to one registered test:
///   repository-checks/a-git-constructed-inside-a-string-literal-is-not-read-a-stated-bound is PINNED-BY
///   `a_construction_inside_a_raw_string_is_not_read`, which the test harness does not register — a renamed
///   or deleted test leaves this citation naming nothing
/// ```
#[test]
fn a_construction_inside_a_raw_string_is_not_read() {
    // Assembled, because this file is inside the corpus the live sweep reads.
    let quote = '"';
    let raw = format!(
        "fn f() {{\n    let fixture = r#{quote}let out = Command::new({quote}git{quote}){quote}#;\n}}"
    );
    assert!(
        !reads(&raw),
        "a raw string is one literal token, so what it carries is that token's text and not a call"
    );
}

/// A construction spelled another way is the same construction.
///
/// `Literal::to_string` gives back the **source rendering**, so `r"git"` and `"\x67it"` — both of which
/// decode to `git` — compared unequal to `"git"` and neither was read. A parser decodes, so the reader
/// answers about the program named rather than about how it was typed.
#[test]
fn a_construction_spelled_another_way_is_still_read() {
    assert!(reads("fn f() {\n    let out = Command::new(r\"git\");\n}"));
    assert!(reads(
        "fn f() {\n    let out = Command::new(\"\\x67it\");\n}"
    ));
    // The control: another program spelled the same ways is not read.
    assert!(!reads(
        "fn f() {\n    let out = Command::new(r\"cargo\");\n}"
    ));
}

/// A file this reader cannot parse is undecidable, not a file that constructs nothing.
///
/// The tokeniser's failure arm fell back to an exact substring, which answers `false` for a construction
/// split across lines — so an unparseable file carrying one was reported clean, silently, which is the one
/// direction the Core Contract forbids. The corpus direction names such a path and fails.
#[test]
fn a_file_this_reader_cannot_parse_is_undecidable() {
    assert_eq!(constructs_git("fn f( {"), Reading::Undecidable);
    assert_eq!(
        constructs_git("fn f() {\n    let out = Command::new(\n        \"git\",\n    );\n}"),
        Reading::Constructs
    );
    assert_eq!(constructs_git("fn f() {}"), Reading::DoesNot);
}

/// A construction through a rename or inside a macro is the same construction.
///
/// Both were escapes a review supplied and both worked: reading only the segment `Command` missed
/// `use std::process::Command as Cmd; Cmd::new("git")`, and the default walk never reaches a macro's tokens,
/// so `dbg!(Command::new("git"))` was a construction in an undeclared file that this check reported as none.
#[test]
fn a_construction_through_a_rename_or_inside_a_macro_is_read() {
    assert!(reads(
        "use std::process::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    assert!(reads("fn f() {\n    dbg!(Command::new(\"git\"));\n}"));
    assert!(reads(
        "use std::process::Command as Cmd;\nfn f() {\n    dbg!(Cmd::new(\"git\"));\n}"
    ));
    assert!(reads(
        "fn f() {\n    let c = std::process::Command::new(\"git\");\n}"
    ));

    // A `use` inside a function or an inline module binds as surely as one at the top: collected from the
    // top-level items alone, both of these bound nothing.
    assert!(reads(
        "fn f() {\n    use std::process::Command as Cmd;\n    let c = Cmd::new(\"git\");\n}"
    ));
    assert!(reads(
        "mod m {\n    use std::process::Command as Cmd;\n    pub fn f() {\n        let c = Cmd::new(\"git\");\n    }\n}"
    ));

    // A statement-oriented body is not an expression list, and discarding that failure let it carry a
    // construction past this reader.
    assert!(reads(
        "fn f() {\n    passthrough!(let _ = Command::new(\"git\"););\n}"
    ));

    // The controls: a rename this file does not bind is not `Command`, a rename to something else is not
    // either, and a macro carrying another program is not read — so the assertions above are about the
    // binding and the body rather than about a reader that reports every `new`.
    assert!(!reads("fn f() {\n    let c = Cmd::new(\"git\");\n}"));
    assert!(!reads(
        "use foo::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    // **The path, not a `process` somewhere in it.** Binding on *a segment named `process` occurred* made
    // somebody else's `Command` a `std::process::Command`, which widens the reaction rather than the bound.
    assert!(!reads(
        "use foo::process::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    // A lexical root naming *this* crate is not the standard library: `crate::std::process::Command` is a
    // module this repository could define, and normalising the root away merged it with the external crate.
    assert!(!reads(
        "use crate::std::process::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    assert!(!reads(
        "use self::std::process::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));

    // And the canonical spellings past a leading `::` are bound.
    assert!(reads(
        "use ::std::process::Command as Cmd;\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    assert!(reads(
        "use std::process::{Command as Cmd, Stdio};\nfn f() {\n    let c = Cmd::new(\"git\");\n}"
    ));
    assert!(!reads("fn f() {\n    dbg!(Command::new(\"cargo\"));\n}"));
}

/// A macro body neither grammar parsed, naming a bound `Command`, is undecidable.
///
/// The safe direction where the alternative is reporting a file clean over tokens nothing classified. A body
/// that names nothing bound to `Command` is not made undecidable for being unparseable — most macro bodies
/// are neither grammar and carry no call this reader would have read.
#[test]
fn an_unclassifiable_macro_body_naming_a_command_is_undecidable() {
    assert_eq!(
        constructs_git("fn f() {\n    weird!(Command => \"git\");\n}"),
        Reading::Undecidable
    );
    assert_eq!(
        constructs_git("fn f() {\n    matches!(x, Some(_) if true);\n}"),
        Reading::DoesNot
    );
    // **At any depth.** Testing the immediate tokens alone, a body nesting the construction inside a
    // delimiter named nothing this reader saw.
    assert_eq!(
        constructs_git("fn f() {\n    weird!([Command::new(\"git\")] => ());\n}"),
        Reading::Undecidable
    );
}

/// The summary reader accepts one passing test and nothing that merely reads like one.
///
/// The evidence was `contains("1 passed")` over combined stdout and stderr — satisfied by any output
/// carrying those words, and unable to tell one passing test from one passing test beside a failure.
#[test]
fn a_summary_is_parsed_rather_than_matched() {
    let one = "running 1 test\ntest a ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 12 filtered out; finished in 0.01s\n";
    assert_eq!(
        summary(one),
        Ok(Summary {
            passed: 1,
            failed: 0
        })
    );

    // A filter that matched nothing exits zero and says so.
    let none = "running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 13 filtered out; finished in 0.00s\n";
    assert_eq!(
        summary(none),
        Ok(Summary {
            passed: 0,
            failed: 0
        })
    );

    // The token without a summary: what the old reading accepted.
    assert_eq!(
        summary("Compiling something that says 1 passed somewhere\n"),
        Err(NoSummary::Missing)
    );

    // Two summaries are two runs, and which carried the proof is not decidable from here — and **which of
    // the two refusals** it is, is what the report says. Both answered `None` before, so a run producing two
    // summaries was reported as producing none.
    assert_eq!(summary(&format!("{one}{one}")), Err(NoSummary::Multiple(2)));

    // One passing beside one failing is not one passing.
    let mixed = "test result: FAILED. 1 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s\n";
    assert_eq!(
        summary(mixed),
        Ok(Summary {
            passed: 1,
            failed: 1
        })
    );
}

/// The declared set names a path this repository tracks.
///
/// A declaration pointing at a file that no longer exists is an entry nobody can falsify: the membership
/// comparison above would report it as missing from the constructing set and read as a site that stopped
/// constructing, rather than as a name that outlived its subject.
#[test]
fn every_declared_site_is_a_tracked_path() {
    let Some(root) = workspace_root() else {
        return;
    };
    let tracked: BTreeSet<String> = kanhe::hermetic_git::tracked_paths(&root, &[])
        .expect("the tracked set is enumerable")
        .into_iter()
        .collect();
    for (path, _, _) in CONSTRUCTS_GIT_ITSELF {
        assert!(
            tracked.contains(path),
            "the declared site {path} is not tracked"
        );
    }
}

/// Every declared site carries a purpose, so the next reader is told why rather than left to infer it.
#[test]
fn every_declared_site_says_why() {
    for (path, _, why) in CONSTRUCTS_GIT_ITSELF {
        assert!(
            why.len() > 30,
            "the entry for {path} states no purpose a reader could act on"
        );
    }
}
