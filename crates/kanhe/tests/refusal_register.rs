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
use shengmo::workspace::MARKER;
use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;
use std::path::{Path, PathBuf};
use std::process::Command;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{
    Block, Expr, ExprCall, ExprField, ExprLit, ExprMethodCall, ExprPath, ItemUse, Lit, Stmt,
    UseTree,
};

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

    // **The shapes a hand-written scanner desynchronises on.** A quote inside a char literal is not an open
    // quote; the first form read it as one and ran to the next `"` anywhere after it, so a construction
    // between two of them vanished and a literal after one was exposed. Both directions are held, because a
    // scanner that is wrong desynchronises in both.
    assert_eq!(
        counted("fn a() { if c == '\"' {} }\nfn g() { violation(x); }\nfn b() { if c == '\"' {} }"),
        1,
        "a construction between two quote char literals is not swallowed"
    );
    assert_eq!(
        counted("fn a() { if c == '\"' {} }\nfn c() { let t = \"violation\"; }"),
        0,
        "a literal after a quote char literal is still a literal"
    );
    assert_eq!(
        counted("fn f() { let c = '\\''; violation(x); }"),
        1,
        "an escaped quote char literal closes, and what follows is code"
    );
    assert_eq!(
        counted("fn f<'a>(x: &'a str) { violation(x); }"),
        1,
        "a lifetime is not a char literal and opens nothing"
    );
    assert_eq!(
        counted("fn f() { g(br#\"violation\"#); g(b\"cannot_judge\"); }"),
        0,
        "a byte string is a literal, raw or not — the `b` does not stop the arm from seeing it"
    );

    // **Which region a token sits in, and what the token is, are two questions.** Every shape this direction
    // declares up to here asks only the first. A name is a construction when it is *called*, and `(` is a boundary character
    // like `|`, `.` and `,` — so a closure parameter, a binding and a field access all read as one.
    assert_eq!(
        counted("fn f() { xs.iter().any(|violation| violation.kind) }"),
        0,
        "a closure parameter and a field access are not constructions"
    );
    assert_eq!(
        counted("fn f() { let violation = x; }"),
        0,
        "a `let` binding introduces the name and constructs nothing"
    );
    // **What this reader cannot tell apart, stated rather than asserted.** A bare reference to a name is a
    // constructor taken by name — which the corpus declares as a construction — or a use of a local that
    // shares it, and nothing in the text says which. So `g(violation)` after `let violation = x;` counts,
    // and that is a limit of a text reader rather than a defect in this one.
    assert_eq!(
        counted("fn f() { let violation = x; g(violation); }"),
        1,
        "a bare reference is read as a constructor taken by name, whichever it is"
    );
    assert_eq!(
        counted("fn f() { let build = violation; build(x) }"),
        1,
        "a constructor taken by name is a construction, which is why a following `(` is not the test"
    );
    // **The boundary of the exclusion, which the shapes chosen from the defect did not reach.** A closure's
    // closing pipe looks exactly like its opening one to a character test, so a construction called in the
    // body was excluded with the parameter that introduces the name.
    assert_eq!(
        counted("fn f() { xs.iter().map(|e| violation(e)) }"),
        1,
        "a construction inside a closure body is kept — the pipe before it closes the parameter list"
    );
    assert_eq!(
        counted("fn f() { g(|| cannot_judge(x)) }"),
        1,
        "a zero-argument closure introduces no name, so what follows its pipes is a construction"
    );
    assert_eq!(
        counted("fn f() { xs.map(|violation| g(violation)) }"),
        1,
        "the parameter is excluded and the reference in the body is not, on one line"
    );
}

/// The reader swallows no declaration from any file it actually reads.
///
/// **A direction over synthetic fragments cannot reach a property of the real corpus.** Eleven shapes were
/// declared and every one is a well-formed Rust fragment; none is a comment carrying an unbalanced quote,
/// which is what four files of this repository hold — `region.rs`'s comments carry an odd number of double
/// quotes, and under a reader that stripped literals before comments a lone `"` in a comment opened a string
/// that ran into real code, swallowing three `pub fn` declarations. Both figures this register produces
/// stayed correct through that, by which tokens the swallowed spans happened to contain.
///
/// So the corpus is the subject: a declaration is code, `code_only` removes no code, and any file where the
/// count drops names a span the reader lost. It is `pub fn` rather than every token because a declaration
/// cannot appear inside a literal or a comment by accident — the assertion is about the reader, not about
/// how this repository writes prose.
#[test]
fn the_reader_swallows_no_declaration_from_the_corpus_it_reads() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut lost = Vec::new();
    let mut examined = 0;
    for dir in ["crates/kanhe/src", "crates/kanhe/tests"] {
        for path in tracked(&root, dir) {
            let text = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));
            // **At the start of its line, which is where a declaration is and an embedded fragment is
            // not.** These files put Rust in string literals on purpose — a projection template, a fixture,
            // this file's own `counted("fn f() { … }")` — and every one of those sits behind a call, never
            // at column zero of its own line. Comparing the same line index in both texts is what tells a
            // declaration the reader lost from a fragment it correctly emptied.
            let declares = |line: &str| {
                let t = line.trim_start();
                t.starts_with("fn ")
                    || t.starts_with("pub fn ")
                    || t.starts_with("pub(") && t.contains(" fn ")
            };
            let code = code_only(&text);
            let after: Vec<&str> = code.lines().collect();
            for (index, line) in text.lines().enumerate() {
                if !declares(line) {
                    continue;
                }
                examined += 1;
                match after.get(index) {
                    Some(kept) if declares(kept) => {}
                    _ => lost.push(format!(
                        "  {}:{}: {}",
                        path.display(),
                        index + 1,
                        line.trim()
                    )),
                }
            }
        }
    }
    assert!(
        examined > 0,
        "no declaration entered the corpus, so this direction would report clean over nothing"
    );
    assert!(
        lost.is_empty(),
        "the reader swallowed code from files it reads, so every figure it produces holds only by what the \
         swallowed spans happened to contain:\n{}",
        lost.join("\n")
    );
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

// =============================================================================================
// The reader: a real Rust parser (`syn`) rather than a character-by-character scanner.
//
// **Why a real parser closes this bug class rather than adding another shape to a scanner.** A
// hand-rolled predecessor of everything below needed a dedicated arm for every new shape it was
// found wrong on — a byte prefix before a char literal, a raw string's hash count, an escaped
// newline inside a string, a wrapped `use` list. Each fix was correct and each was local — none of
// them made the *next* shape less likely, because that reader was never parsing Rust; it was
// pattern-matching against Rust well enough for the corpus measured so far. `syn::parse_str` and
// `syn::Block::parse_within` parse the actual grammar, so a raw string, a byte char literal, or a
// closure whose parameter list spans two lines are read correctly by construction, not by an arm
// added for that specific shape after it was found wrong. See
// `a_naive_scanner_without_the_byte_prefix_fix_regresses_where_the_syn_reader_does_not` further
// down for "seen to fail" evidence of exactly that class, held against a reproduction of the
// bug this reader was written to make structurally impossible rather than merely patched.
// =============================================================================================

/// `text` parsed as a sequence of statements, however it is shaped.
///
/// A complete, compiling source file is always a valid [`syn::File`] — every real corpus file
/// this register reads is one, since it compiles. A **fixture** written to exercise one shape
/// in isolation often is not: `let x = a_violation(1);` alone is not a file (there is no item
/// there for a file to hold), and mixing a `use` with bare statements is not a sequence of
/// items either. Both are exactly what [`Block::parse_within`] parses: the grammar for the
/// inside of a block, where an item, a `let`, and a bare expression all stand as one [`Stmt`]
/// — which is also true of a real file's top-level items, since an item is a valid statement
/// inside a block since Rust 2018. Trying the file grammar first and falling back to the block
/// grammar reads every shape this register or its fixtures hand it, including a leading inner
/// attribute (`#![...]`), which only the file grammar accepts.
fn parse_fragment(text: &str) -> Vec<Stmt> {
    if let Ok(file) = syn::parse_str::<syn::File>(text) {
        return file.items.into_iter().map(Stmt::Item).collect();
    }
    syn::parse::Parser::parse_str(Block::parse_within, text).unwrap_or_else(|err| {
        panic!(
            "this text is not Rust this reader can parse — neither a complete file nor a \
             sequence of statements: {err}\n---\n{text}"
        )
    })
}

/// The four names this register reads, and nothing else — an identifier spelled any other way
/// is prose or an unrelated symbol, not a refusal construction.
const NAMES: [&str; 4] = [
    "violation_at",
    "cannot_judge_at",
    "violation",
    "cannot_judge",
];

fn last_segment_name(path: &syn::Path) -> Option<&'static str> {
    let last = path.segments.last()?.ident.to_string();
    NAMES.iter().copied().find(|name| *name == last)
}

fn as_str_lit(expr: &Expr) -> Option<String> {
    match expr {
        Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) => Some(s.value()),
        _ => None,
    }
}

/// One occurrence of a registered name, classified by where it sits rather than by a
/// character beside it.
///
/// **A position, not a heuristic.** The hand-rolled reader answered *is this a binder* by
/// counting `|` characters before the name on its line, and *is this a projection* by asking
/// whether a `.` immediately follows — both are approximations of a question the grammar
/// already answers exactly. A name that is a [`syn::Pat`] (a closure parameter, a `let`
/// binding, a function argument, a match arm) is never visited as an [`Expr::Path`] at all in
/// this walk, so it is excluded by construction rather than by counting pipes; a name that is
/// the base of a field access or a method receiver is excluded the same way, by checking the
/// one syntactic parent that matters instead of the one character that usually correlates
/// with it.
///
/// **What this still cannot tell, on purpose.** A bare [`ExprPath`] referring to `violation`
/// might be the constructor taken by value, or a local that happens to share its spelling —
/// the grammar alone does not say which, and this reader does not attempt to resolve it. Both
/// read as one occurrence, matching what this file's own tests already assert.
#[derive(Debug, Clone)]
struct Finding {
    name: &'static str,
    /// `Some(value)` when this occurrence is the callee of a call whose first argument is a
    /// plain or raw string literal — the shape a registered site takes, whichever quoting it
    /// was written with.
    call_first_arg_lit: Option<String>,
    line: usize,
}

struct NameFinder {
    findings: Vec<Finding>,
}

impl NameFinder {
    fn record(
        &mut self,
        name: &'static str,
        call_first_arg_lit: Option<String>,
        span: proc_macro2::Span,
    ) {
        self.findings.push(Finding {
            name,
            call_first_arg_lit,
            line: span.start().line,
        });
    }
}

impl<'ast> Visit<'ast> for NameFinder {
    fn visit_expr_call(&mut self, node: &'ast ExprCall) {
        if let Expr::Path(expr_path) = node.func.as_ref() {
            if let Some(name) = last_segment_name(&expr_path.path) {
                let lit = node.args.first().and_then(as_str_lit);
                self.record(name, lit, node.span());
                // The callee is handled; still walk the arguments for a nested construction,
                // e.g. `outer(violation(x))`.
                for arg in &node.args {
                    self.visit_expr(arg);
                }
                return;
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    fn visit_expr_field(&mut self, node: &'ast ExprField) {
        // `violation.kind` — a construction is called or referenced as a value, never used as
        // the receiver of a field it does not have. Suppressing the base here is what makes a
        // closure body doing `|violation| violation.kind` read as the two exclusions it
        // actually is: the parameter (never visited as a path at all) and this receiver.
        if let Expr::Path(p) = node.base.as_ref() {
            if last_segment_name(&p.path).is_some() {
                return;
            }
        }
        syn::visit::visit_expr_field(self, node);
    }

    fn visit_expr_method_call(&mut self, node: &'ast ExprMethodCall) {
        if let Expr::Path(p) = node.receiver.as_ref() {
            if last_segment_name(&p.path).is_some() {
                for arg in &node.args {
                    self.visit_expr(arg);
                }
                return;
            }
        }
        syn::visit::visit_expr_method_call(self, node);
    }

    fn visit_expr_path(&mut self, node: &'ast ExprPath) {
        if let Some(name) = last_segment_name(&node.path) {
            self.record(name, None, node.span());
            return;
        }
        syn::visit::visit_expr_path(self, node);
    }
}

/// Every occurrence of a registered name in `text`, by real syntax rather than by text.
///
/// A function's own name (`fn violation(...)`) is a [`syn::Signature`] field, never an
/// [`Expr::Path`]; a `use` import is a [`UseTree`], which holds no [`Expr`] at all. Neither is
/// visited by the walk above, so a definition and an import are excluded by the shape of the
/// grammar rather than by a rule this reader has to state and keep correct.
fn findings(text: &str) -> Vec<Finding> {
    let stmts = parse_fragment(text);
    let mut finder = NameFinder {
        findings: Vec::new(),
    };
    for stmt in &stmts {
        finder.visit_stmt(stmt);
    }
    finder.findings
}

/// A corpus with comments, string literal contents and imports already excluded from being
/// read as a construction — because none of them are [`Expr::Path`] nodes to begin with.
struct Countable(Vec<Finding>);

/// Occurrences of `text`'s registered names, over the corpus [`calls`] reads.
fn countable(text: &str) -> Countable {
    Countable(findings(text))
}

/// Occurrences of the constructor named `name` in `countable`'s corpus, however it is
/// reached — called, referenced bare, or taken by value and called through an alias.
fn calls(countable: &Countable, name: &str) -> usize {
    countable.0.iter().filter(|f| f.name == name).count()
}

/// How many registered constructions in `text` this reader could not parse to a site.
///
/// A call whose callee is `violation_at`/`cannot_judge_at` and whose first argument is a
/// string literal (raw or not) is a parsed site. Everything else this walk still counted as
/// that name — a bare reference taken by value, or a call whose first argument is not a
/// literal — is unparsed: seen, but not read as a site.
fn unparsed_constructions(text: &str) -> usize {
    findings(text)
        .into_iter()
        .filter(|f| f.name == "violation_at" || f.name == "cannot_judge_at")
        .filter(|f| f.call_first_arg_lit.is_none())
        .count()
}

fn use_tree_aliases_a_constructor(tree: &UseTree) -> bool {
    match tree {
        UseTree::Path(p) => use_tree_aliases_a_constructor(&p.tree),
        UseTree::Group(g) => g.items.iter().any(use_tree_aliases_a_constructor),
        UseTree::Rename(r) => {
            let original = r.ident.to_string();
            original.contains("violation") || original.contains("cannot_judge")
        }
        UseTree::Name(_) | UseTree::Glob(_) => false,
    }
}

struct AliasFinder(bool);

impl<'ast> Visit<'ast> for AliasFinder {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        if use_tree_aliases_a_constructor(&node.tree) {
            self.0 = true;
        }
        // No need to recurse further: a `use` item holds no nested `use` of its own.
    }
}

/// Whether `text` imports a refusal constructor under another name.
///
/// [`AliasFinder`] visits every [`ItemUse`] the walk reaches, at any depth — a module-level
/// import and one written inside a function body are the same node type to `syn`, so both are
/// found the same way, unlike a line-based reader that has to notice indentation meant
/// nothing.
fn aliases_a_constructor(text: &str) -> bool {
    let stmts = parse_fragment(text);
    let mut finder = AliasFinder(false);
    for stmt in &stmts {
        finder.visit_stmt(stmt);
    }
    finder.0
}

/// `text` with every comment and every string/char literal (delimiters and interior alike)
/// blanked, keeping the line count exactly as it was.
///
/// **The gaps between real tokens, not a scan for `//` and `"`.** `proc_macro2`'s tokenizer
/// already knows exactly where a raw string, a byte char literal, or a doc comment begins and
/// ends — comments are never emitted as tokens at all, and a literal is emitted as one token
/// spanning its whole quoted form however it is written. Copying an identifier's, a punctuation
/// mark's, or a group delimiter's byte range verbatim, and blanking every literal along with
/// everything between tokens (while keeping whitespace, so a `pub`/`fn` boundary a line-based
/// check depends on does not fuse into `pubfn`), can therefore never mistake a `//` inside a raw
/// string for a comment, or a quote inside a byte char literal for the start of a string — the
/// exact class of desynchronisation the character-by-character version above needed a dedicated
/// arm for, the first time each shape was found.
fn code_only(text: &str) -> String {
    // A `Literal`'s span is deliberately absent from `out` below: its whole quoted form —
    // delimiters and interior alike — is what this function's own doc comment calls blanked, so
    // a `pub fn` shaped line embedded in a string (this file's own fixtures write several) is
    // never read back as a declaration by `the_reader_swallows_no_declaration_from_the_corpus_it_reads`,
    // which depends on exactly that to tell a real declaration from one merely quoted.
    fn collect_spans(stream: proc_macro2::TokenStream, out: &mut Vec<Range<usize>>) {
        for tt in stream {
            match tt {
                proc_macro2::TokenTree::Group(g) => {
                    out.push(g.span_open().byte_range());
                    collect_spans(g.stream(), out);
                    out.push(g.span_close().byte_range());
                }
                proc_macro2::TokenTree::Ident(i) => out.push(i.span().byte_range()),
                proc_macro2::TokenTree::Punct(p) => out.push(p.span().byte_range()),
                proc_macro2::TokenTree::Literal(_) => {}
            }
        }
    }

    let Ok(tokens) = text.parse::<proc_macro2::TokenStream>() else {
        // Unlexable text is not a corpus this reader can hold an opinion about; an empty
        // reading is the honest one, and this repository's own corpus (which always compiles)
        // is what would notice this ever firing there.
        return String::new();
    };
    let mut spans = Vec::new();
    collect_spans(tokens, &mut spans);
    spans.sort_unstable_by_key(|range| range.start);

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0usize;
    for (byte, ch) in text.char_indices() {
        while cursor < spans.len() && byte >= spans[cursor].end {
            cursor += 1;
        }
        let in_code =
            cursor < spans.len() && byte >= spans[cursor].start && byte < spans[cursor].end;
        if in_code || ch.is_whitespace() {
            out.push(ch);
        }
    }
    out
}

fn expect_citations(text: &str) -> Vec<String> {
    struct ExpectFinder(Vec<String>);
    impl<'ast> Visit<'ast> for ExpectFinder {
        fn visit_expr_call(&mut self, node: &'ast ExprCall) {
            if let Expr::Path(expr_path) = node.func.as_ref() {
                let segments = &expr_path.path.segments;
                // **Path-qualified, never `.expect(...)`.** A method call is a distinct node
                // type in this grammar (`ExprMethodCall`), so `Result::expect` can never reach
                // this arm no matter how this walk recurses — the distinction the hand-rolled
                // reader drew by requiring the substring `::expect(` is drawn here by the
                // grammar itself.
                if segments.len() >= 2 && segments.last().is_some_and(|s| s.ident == "expect") {
                    if let Some(site) = node.args.first().and_then(as_str_lit) {
                        self.0.push(site);
                    }
                }
            }
            syn::visit::visit_expr_call(self, node);
        }
    }
    let stmts = parse_fragment(text);
    let mut finder = ExpectFinder(Vec::new());
    for stmt in &stmts {
        finder.visit_stmt(stmt);
    }
    finder.0
}

/// Read every registered refusal site under `root`, and how much of the repository still
/// constructs one without a site identity — the same contract this function has always had, now
/// answered by walking a real parse tree instead of scanning characters.
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
        if name.ends_with("src/refusal.rs") {
            continue;
        }
        assert_eq!(
            unparsed_constructions(&text),
            0,
            "{name} constructs a registered refusal in a shape this register cannot read"
        );
        assert!(
            !aliases_a_constructor(&text),
            "{name} imports a refusal constructor under another name"
        );
        for finding in findings(&text) {
            if finding.name != "violation_at" && finding.name != "cannot_judge_at" {
                continue;
            }
            let Some(site) = finding.call_first_arg_lit else {
                continue;
            };
            disagrees.insert(site.clone(), finding.name == "violation_at");
            registered
                .entry(site)
                .or_default()
                .push((name.clone(), finding.line));
        }
        let countable = countable(&text);
        let open = calls(&countable, "violation") + calls(&countable, "cannot_judge");
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
            for site in expect_citations(&text) {
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

/// A minimal reproduction of the bug the character-by-character reader this file used to hold
/// needed a dedicated arm to close — run here only as "seen to fail" evidence that the syn-based
/// [`code_only`] below was never vulnerable to it, not as a reader anything else calls.
///
/// **What this scanner gets wrong, and why.** Before that branch existed, a quote was read as
/// opening a char literal by looking at the character immediately before it: an identifier
/// character meant "this is the tail of a longer name", anything else meant "this opens a
/// literal". A `b` prefix is itself an identifier character, so `b'"'` was read as the identifier
/// `b` followed by a **new**, unrelated char literal `'"'` — one whose contents never close,
/// because the quote inside it is exactly the character this scanner is searching for to close it.
/// Every byte from there to the next real `'"'` later in the file disappears into that literal,
/// which in [`a_byte_char_literal_holding_a_quote.rs.txt`] takes the `violation(...)` call two
/// lines down with it.
fn naive_scan_without_the_byte_prefix_fix(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    let mut prev_ident = false;
    let ident = |c: char| c.is_alphanumeric() || c == '_';
    // Consume a literal opened by `open`, closed by the next unescaped occurrence of `close`,
    // preserving only newlines crossed along the way — the shape both the char arm and the string
    // arm shared before either knew about a byte prefix.
    let consume_literal = |chars: &mut std::str::Chars, out: &mut String, close: char| {
        while let Some(inner) = chars.next() {
            if inner == '\\' {
                chars.next();
                continue;
            }
            if inner == '\n' {
                out.push('\n');
            }
            if inner == close {
                break;
            }
        }
    };
    while let Some(c) = chars.next() {
        // **Both arms gated on the same, now-known-wrong, check.** A quote is read as opening a
        // literal only when the character before it is not an identifier character — which is
        // right for `'a'` and `"s"` and wrong for `b'"'`: the `'` right after `b` is declined (`b`
        // is an identifier character), so it is read as ordinary code, and the `"` right after
        // *that* `'` — preceded by a non-identifier character — is read as opening a **string**,
        // which then runs to the next unescaped `"` anywhere later in the file.
        if (c == '\'' || c == '"') && !prev_ident {
            let close = c;
            consume_literal(&mut chars, &mut out, close);
            prev_ident = false;
            continue;
        }
        out.push(c);
        prev_ident = ident(c);
    }
    out
}

/// The "seen to fail" evidence this migration itself demands: a construction a hand-rolled
/// character scanner swallows, read correctly by [`code_only`]/[`countable`]/[`calls`] — the real
/// reader this file ships — on the exact same input.
///
/// [`the_reader_swallows_no_declaration_from_the_corpus_it_reads`] already holds the shipped
/// reader to account over this repository's real corpus, so this is not a claim about *that*
/// reader; it is a claim about *why* a real parser is the fix rather than one more arm. The naive
/// scanner above reproduces the historical swallow by hand; nothing needs reproducing on the syn
/// side, because it never scanned characters to begin with.
#[test]
fn a_naive_scanner_without_the_byte_prefix_fix_regresses_where_the_syn_reader_does_not() {
    let Some(root) = workspace_root() else {
        return;
    };
    let path = root.join(
        "crates/kanhe/tests/fixtures/refusal_scan/a_byte_char_literal_holding_a_quote.rs.txt",
    );
    let text = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("cannot read {}: {err}", path.display()));

    let naive = naive_scan_without_the_byte_prefix_fix(&text);
    assert!(
        !naive.contains("violation(\""),
        "this demonstration expects the pre-fix scanner to have swallowed the construction into \
         the runaway char literal it opens on `b'\"'`; if the call still appears intact, this test \
         no longer reproduces the historical bug it exists to document:\n{naive}"
    );

    let count = {
        let c = countable(&text);
        calls(&c, "violation") + calls(&c, "cannot_judge")
    };
    assert_eq!(
        count, 1,
        "the syn reader must read the true construction regardless of how a hand-rolled scanner \
         run over the same bytes would have fared"
    );
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
         this document does not make. **No count of them is given**, and two different things stand in \
         the way. One is a floor: a name taken by reference reads the same whether it is the constructor or a \
         local that shares its spelling, and no reader of text can decide which. The other is a debt: whether \
         an occurrence is inside a closure's parameter list is a **position**, and this reader answers it by \
         counting the pipes standing before the name on its line — which is right for every shape the corpus \
         holds and is an approximation of the position rather than the position itself. The first cannot be \
         closed; the second can.\n\n\
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

/// The cases whose answer is not a count, named once each.
///
/// **Three lists had to agree and were written three times.** Each arm below spelled its own cases inline and
/// the completeness join at the end spelled all eight again — so a case could be answered by an arm and left
/// out of the join, or joined and answered by nothing, and neither would be reported. The join is derived
/// from these now, which is the same repair the counted table already had.
///
/// **`a_raw_literal_site` left this list when the reader stopped needing to.** A raw string
/// first argument used to be unparsed — not because the site cannot be known, but because the
/// hand-rolled reader's own quote-matching required a plain `"` immediately after the opening
/// parenthesis, and `r#"…"#` opens with `r#` first. `syn::Lit::Str` decodes a raw string exactly
/// like a plain one, with no extra code for it, so this case now belongs beside the other controls
/// this reader successfully parses. The two names remaining here are a different kind of limit —
/// the site is a local variable's *value*, not present in the text at all, which no reader of
/// syntax alone can recover.
const UNREADABLE_SITE_CASES: [&str; 2] = [
    "a_siteful_constructor_taken_by_name",
    "a_siteful_call_that_wraps",
];

/// The cases where a constructor arrives under another name, so the file cannot be counted at all.
const ALIASED_CASES: [&str; 2] = ["an_aliased_import", "a_wrapped_aliased_import"];

/// The cases that are an import and nothing else, in each spelling `use` takes.
const IMPORT_ONLY_CASES: [&str; 3] = [
    "a_wrapped_import_that_constructs_nothing",
    "a_public_import_that_constructs_nothing",
    "a_scoped_public_import_wrapped",
];

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
        ("a_byte_char_literal_holding_a_quote", 1),
        ("a_call_and_a_definition", 1),
        ("a_call_that_wraps", 1),
        ("a_comment", 0),
        ("a_constructor_taken_by_name", 1),
        ("a_longer_identifier", 0),
        ("a_raw_literal_site", 0),
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
        let got = {
            let countable = countable(&source);
            calls(&countable, "violation") + calls(&countable, "cannot_judge")
        };
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
    for case in UNREADABLE_SITE_CASES {
        assert_eq!(
            unparsed_constructions(&read(case)),
            1,
            "{case}: a registered construction this reader cannot parse went unreported"
        );
    }
    for case in std::iter::once("two_on_one_line")
        .chain(std::iter::once("a_raw_literal_site"))
        .chain(IMPORT_ONLY_CASES)
    {
        assert_eq!(
            unparsed_constructions(&read(case)),
            0,
            "{case}: the controls — a construction this reader parses (plain or raw), and an \
             import that constructs nothing — must not be reported as unread"
        );
    }

    for case in ALIASED_CASES {
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
            UNREADABLE_SITE_CASES
                .into_iter()
                .chain(ALIASED_CASES)
                .chain(IMPORT_ONLY_CASES)
                .map(str::to_string),
        )
        .collect();
    assert_eq!(
        cases, named,
        "the corpus and the cases this direction runs disagree, so a case exists that nothing answers"
    );
}
