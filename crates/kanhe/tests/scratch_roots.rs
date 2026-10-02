//! Repository check: a fixture root is the helper's — never the system temporary directory's, and never a name a
//! caller composed.
//!
//! `xingbiao::scratch_root` places every fixture under the build directory, where the user owns the directory
//! and `cargo clean` reaches it, and names it `<label>-<pid>-<counter>`, so two roots are distinct by
//! construction and not by the labels their callers happened to choose. A tracked Rust file that asks the
//! system for its temporary directory builds a root outside that, and a root there is claimable by anyone who
//! can write the directory and fails with it when the directory stops being writable. A tracked Rust file that
//! composes a name under `xingbiao::scratch_base` builds a root the counter does not name, whose name is unique
//! only while no two callers choose one label.
//!
//! **This file answers two questions — which tracked files construct a root from the system, and which name
//! the helper's base directory instead of taking a root from it — and never whether a run is isolated, nor
//! whether a name is unique.** Uniqueness is the helper's, by its counter; what is asked of a caller is a
//! single identifier, which a parse tree answers. `hermetic_invocations` records why deciding isolation by
//! reading source is the wrong instrument; the same loop reads the files (`support::tracked_rust`), the same
//! three-state answer says when a file could not be decided, and a file that cannot be parsed is refused rather
//! than reported clean.
//!
//! **Why 圭表 does not hold this.** It scans compilation units reached from library and binary roots, and most
//! fixture roots are written in `tests/*.rs`, which it never reads.
//!
//! **What the reader names:** a path whose last segment is `temp_dir`, `scratch_base` or `scratch_ceiling`,
//! however it is qualified, imported or renamed on import, and — for the system temporary directory — a read of
//! the `TMPDIR` variable through `var`, `var_os`, `env!` or `option_env!`. What it cannot see is declared as a
//! bound in `openspec/specs/repository-checks`.

mod support;

use std::collections::BTreeSet;
use support::tracked_rust::{Reading, files_where, string_of, workspace_root};

/// Every file that may ask the system for a temporary directory, and why.
///
/// Empty: the helper reads the running executable's path and asks the system for nothing. A site that gains
/// one must be named with why; the comparison is two-directional, so a name that outlives its site must go.
const NAMES_A_SYSTEM_TEMP_DIR: [(&str, &str); 0] = [];

/// The crate that owns the helper. Its files are where `scratch_root` takes the base from, so they are not held
/// to the question of who names it.
const THE_HELPERS_OWNER: &str = "crates/xingbiao/";

/// Every file outside [`THE_HELPERS_OWNER`] that names the helper's base directory or its restatement, and why.
///
/// `kanhe`'s normal edges may not reach `xingbiao` (the `kanhe` crate boundary in `crates/shengmo/src/law.rs`
/// restricts them), so the library cannot take a root from the helper and restates the layout rule instead. The
/// comparison is two-directional, so a name that outlives its site must go, and a site that gains one must be
/// named with why.
const NAMES_THE_HELPERS_BASE: [(&str, &str); 3] = [
    (
        "crates/kanhe/src/hermetic_git.rs",
        "defines `scratch_ceiling`, the helper's layout rule restated, and hands it to git as the ceiling",
    ),
    (
        "crates/kanhe/src/publish_source_gate.rs",
        "builds the signature scratch under the ceiling and answers a `Refusal` where the helper would panic",
    ),
    (
        "crates/kanhe/src/tests/scratch_ceiling.rs",
        "holds the restated rule equal to the helper's, the one direction that must name both",
    ),
];

/// What a question asks of a file: the final path segments that answer it, and whether a read of `TMPDIR` does.
#[derive(Clone, Copy)]
struct Question {
    names: &'static [&'static str],
    reads_tmpdir: bool,
}

/// Which files ask the system for a temporary directory.
const THE_SYSTEM_TEMP: Question = Question {
    names: &["temp_dir"],
    reads_tmpdir: true,
};

/// Which files name the helper's base directory, or the restatement of it, rather than take a root from the helper.
const THE_HELPERS_BASE: Question = Question {
    names: &["scratch_base", "scratch_ceiling"],
    reads_tmpdir: false,
};

/// What this reader found in one file.
struct Roots {
    question: Question,
    found: bool,
    /// A macro body neither grammar parsed, naming a word the question asks for.
    undecided: bool,
}

fn names_what_is_asked(tokens: proc_macro2::TokenStream, question: Question) -> bool {
    let mut pending = vec![tokens];
    while let Some(stream) = pending.pop() {
        for tree in stream {
            match tree {
                proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                proc_macro2::TokenTree::Ident(word)
                    if question.names.iter().any(|name| word == name) =>
                {
                    return true;
                }
                proc_macro2::TokenTree::Literal(literal)
                    if question.reads_tmpdir
                        && syn::parse_str::<syn::LitStr>(&literal.to_string())
                            .is_ok_and(|text| text.value() == "TMPDIR") =>
                {
                    return true;
                }
                _ => {}
            }
        }
    }
    false
}

fn last_segment(path: &syn::Path) -> Option<String> {
    path.segments
        .last()
        .map(|segment| segment.ident.to_string())
}

impl Roots {
    fn asks_for(&self, word: &syn::Ident) -> bool {
        self.question.names.iter().any(|name| word == name)
    }
}

impl<'ast> syn::visit::Visit<'ast> for Roots {
    /// A path ending in a word the question asks for, called or passed as a value: `temp_dir()`,
    /// `env::temp_dir()`, `std::env::temp_dir()`, `.map(std::env::temp_dir)`, `xingbiao::scratch_base()`.
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if node
            .path
            .segments
            .last()
            .is_some_and(|segment| self.asks_for(&segment.ident))
        {
            self.found = true;
        }
        syn::visit::visit_expr_path(self, node);
    }

    /// An import of such a word, which a later bare or renamed call is not required to spell the path of.
    fn visit_use_name(&mut self, node: &'ast syn::UseName) {
        if self.asks_for(&node.ident) {
            self.found = true;
        }
    }

    fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
        if self.asks_for(&node.ident) {
            self.found = true;
        }
    }

    /// A read of `TMPDIR`: `var("TMPDIR")` or `var_os("TMPDIR")` under any qualification. A call that *sets* a
    /// child's `TMPDIR` is a method call and not a path call, so it is not this.
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if self.question.reads_tmpdir {
            if let syn::Expr::Path(callee) = &*node.func {
                if matches!(
                    last_segment(&callee.path).as_deref(),
                    Some("var" | "var_os")
                ) && node
                    .args
                    .iter()
                    .any(|arg| string_of(arg).as_deref() == Some("TMPDIR"))
                {
                    self.found = true;
                }
            }
        }
        syn::visit::visit_expr_call(self, node);
    }

    /// `env!("TMPDIR")` and `option_env!("TMPDIR")` are read here; a macro body that is neither an expression
    /// list nor statements and names a word the question asks for is undecidable rather than clean.
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        use syn::punctuated::Punctuated;
        let reads_env = matches!(
            last_segment(&node.path).as_deref(),
            Some("env" | "option_env")
        );
        let as_expressions =
            node.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
        if let Ok(arguments) = as_expressions {
            if self.question.reads_tmpdir
                && reads_env
                && arguments
                    .iter()
                    .any(|arg| string_of(arg).as_deref() == Some("TMPDIR"))
            {
                self.found = true;
            }
            for argument in &arguments {
                syn::visit::Visit::visit_expr(self, argument);
            }
        } else if let Ok(statements) = node.parse_body_with(syn::Block::parse_within) {
            for statement in &statements {
                syn::visit::Visit::visit_stmt(self, statement);
            }
        } else if names_what_is_asked(node.tokens.clone(), self.question) {
            self.undecided = true;
        }
        syn::visit::visit_macro(self, node);
    }
}

/// Whether `text` answers `question`, or says it could not decide.
fn reading(text: &str, question: Question) -> Reading {
    let Ok(parsed) = syn::parse_file(text) else {
        return Reading::Undecidable;
    };
    let mut roots = Roots {
        question,
        found: false,
        undecided: false,
    };
    syn::visit::Visit::visit_file(&mut roots, &parsed);
    if roots.found {
        Reading::Constructs
    } else if roots.undecided {
        Reading::Undecidable
    } else {
        Reading::DoesNot
    }
}

/// Whether `text` builds a root from the system, or says it could not decide.
fn names_a_system_root(text: &str) -> Reading {
    reading(text, THE_SYSTEM_TEMP)
}

/// Whether `text` names the helper's base directory, or says it could not decide.
fn names_the_helpers_base(text: &str) -> Reading {
    reading(text, THE_HELPERS_BASE)
}

fn reads(text: &str) -> bool {
    matches!(names_a_system_root(text), Reading::Constructs)
}

fn reads_the_base(text: &str) -> bool {
    matches!(names_the_helpers_base(text), Reading::Constructs)
}

#[test]
fn every_fixture_root_this_repository_builds_is_the_helpers() {
    let Some(root) = workspace_root() else {
        return;
    };
    let declared: BTreeSet<String> = NAMES_A_SYSTEM_TEMP_DIR
        .iter()
        .map(|(path, _)| (*path).to_string())
        .collect();
    assert_eq!(
        declared,
        files_where(
            &root,
            "asks the system for a temporary directory",
            names_a_system_root
        ),
        "the files asking the system for a temporary directory differ from the set named here. A fixture \
         root comes from `xingbiao::scratch_root`; a site that gains a system root must be named with why, \
         and a name that outlives its site must go"
    );
}

#[test]
fn every_fixture_root_is_taken_from_the_helper_and_never_named_by_a_caller() {
    let Some(root) = workspace_root() else {
        return;
    };
    let declared: BTreeSet<String> = NAMES_THE_HELPERS_BASE
        .iter()
        .map(|(path, _)| (*path).to_string())
        .collect();
    let naming = files_where(
        &root,
        "names the helper's base directory",
        names_the_helpers_base,
    );
    let (owned, outside): (BTreeSet<String>, BTreeSet<String>) = naming
        .into_iter()
        .partition(|path| path.starts_with(THE_HELPERS_OWNER));
    assert!(
        !owned.is_empty(),
        "no file under {THE_HELPERS_OWNER} names the helper's base directory, so the exclusion of that crate \
         names a place where the helper is not, and every other file would be judged against a question \
         nothing answers"
    );
    assert_eq!(
        declared, outside,
        "the files naming the helper's base directory differ from the set named here. A fixture root is a \
         `xingbiao::scratch_root`, whose name is unique per call; a caller that composes a name under \
         `scratch_base` reaches for a name the counter does not make. A site that gains one must be named with \
         why, and a name that outlives its site must go"
    );
}

#[test]
fn a_bare_temp_dir_call_is_read() {
    assert!(reads("fn f() { let _ = temp_dir(); }"));
    assert!(reads(
        "use std::env::*;\nfn f() { let _ = temp_dir().join(\"x\"); }"
    ));
}

#[test]
fn a_qualified_temp_dir_call_is_read() {
    assert!(reads("fn f() { let _ = std::env::temp_dir(); }"));
    assert!(reads("fn f() { let _ = env::temp_dir(); }"));
    assert!(reads("fn f() { let _ = ::std::env::temp_dir(); }"));
    assert!(reads(
        "fn f() { let _ = Some(1).map(|_| std::env::temp_dir()); }"
    ));
    assert!(reads("fn f() { let _ = (|| 1, std::env::temp_dir); }"));
}

#[test]
fn an_imported_or_renamed_temp_dir_is_read() {
    assert!(reads("use std::env::temp_dir;\nfn f() {}"));
    assert!(reads(
        "use std::env::temp_dir as scratch;\nfn f() { scratch(); }"
    ));
    assert!(reads("use std::env::{args, temp_dir};\nfn f() {}"));
    assert!(reads(
        "fn f() { use std::env::temp_dir; let _ = temp_dir(); }"
    ));
}

#[test]
fn a_read_of_tmpdir_is_read() {
    assert!(reads("fn f() { let _ = std::env::var(\"TMPDIR\"); }"));
    assert!(reads("fn f() { let _ = std::env::var_os(\"TMPDIR\"); }"));
    assert!(reads(
        "use std::env::var_os;\nfn f() { let _ = var_os(r\"TMPDIR\"); }"
    ));
    assert!(reads("const T: &str = env!(\"TMPDIR\");"));
    assert!(reads("const T: Option<&str> = option_env!(\"TMPDIR\");"));
}

#[test]
fn naming_temp_dir_in_a_comment_a_string_or_a_child_environment_is_not_read() {
    assert!(!reads("// std::env::temp_dir()\nfn f() {}"));
    assert!(!reads("/// builds `temp_dir()` roots\nfn f() {}"));
    assert!(!reads("fn f() { let _ = \"std::env::temp_dir()\"; }"));
    assert!(!reads(
        "fn f(c: &mut std::process::Command) { c.env(\"TMPDIR\", \"/x\"); }"
    ));
    assert!(!reads("const INHERITED: [&str; 1] = [\"TMPDIR\"];"));
}

#[test]
fn a_root_reached_through_a_value_is_not_read() {
    assert!(!reads(
        "fn f(base: std::path::PathBuf) { let _ = base.join(\"x\"); }"
    ));
    assert!(!reads(
        "fn f() { let key = \"TMPDIR\"; let _ = std::env::var(key); }"
    ));
}

#[test]
fn a_file_the_root_reader_cannot_parse_is_undecidable() {
    assert_eq!(names_a_system_root("fn ("), Reading::Undecidable);
}

#[test]
fn an_unclassifiable_macro_body_naming_the_system_temp_is_undecidable() {
    assert_eq!(
        names_a_system_root("fn f() { passthrough!(=> temp_dir => ); }"),
        Reading::Undecidable
    );
    assert_eq!(
        names_a_system_root("fn f() { passthrough!(=> \"TMPDIR\" => ); }"),
        Reading::Undecidable
    );
    assert_eq!(
        names_a_system_root("fn f() { passthrough!(=> other => ); }"),
        Reading::DoesNot
    );
}

#[test]
fn a_call_to_the_helpers_base_is_read() {
    assert!(reads_the_base(
        "fn f() { let _ = xingbiao::scratch_base(); }"
    ));
    assert!(reads_the_base(
        "use xingbiao::*;\nfn f() { let _ = scratch_base().join(\"x\"); }"
    ));
    assert!(reads_the_base(
        "fn f() { let _ = ::xingbiao::scratch_base(); }"
    ));
    assert!(reads_the_base(
        "fn f() { let _ = crate::ceiling::scratch_ceiling(); }"
    ));
    assert!(reads_the_base(
        "fn f() { let _ = Some(1).map(|_| xingbiao::scratch_base()); }"
    ));
    assert!(reads_the_base(
        "fn f() { let _ = (|| 1, xingbiao::scratch_base); }"
    ));
}

#[test]
fn an_imported_or_renamed_base_is_read() {
    assert!(reads_the_base("use xingbiao::scratch_base;\nfn f() {}"));
    assert!(reads_the_base(
        "use xingbiao::scratch_base as b;\nfn f() { b(); }"
    ));
    assert!(reads_the_base(
        "use xingbiao::{claim_scratch, scratch_base};\nfn f() {}"
    ));
    assert!(reads_the_base(
        "use crate::ceiling::scratch_ceiling as ceiling;\nfn f() {}"
    ));
    assert!(reads_the_base(
        "fn f() { use xingbiao::scratch_base; let _ = scratch_base(); }"
    ));
}

#[test]
fn taking_a_root_from_the_helper_is_not_read_as_naming_the_base() {
    assert!(!reads_the_base(
        "fn f() { let _ = xingbiao::scratch_root(\"x\"); }"
    ));
    assert!(!reads_the_base(
        "use xingbiao::scratch_root;\nfn f() { let _ = scratch_root(\"x\"); }"
    ));
    assert!(!reads_the_base(
        "fn f(root: xingbiao::ScratchRoot) { let _ = root.path().join(\"x\"); }"
    ));
}

#[test]
fn each_question_reads_only_its_own_words() {
    assert!(!reads("fn f() { let _ = xingbiao::scratch_base(); }"));
    assert!(!reads_the_base("fn f() { let _ = std::env::temp_dir(); }"));
    assert!(!reads_the_base(
        "fn f() { let _ = std::env::var(\"TMPDIR\"); }"
    ));
}

#[test]
fn the_base_named_in_a_comment_a_string_or_a_method_is_not_read() {
    assert!(!reads_the_base("// xingbiao::scratch_base()\nfn f() {}"));
    assert!(!reads_the_base(
        "/// builds `scratch_base()` roots\nfn f() {}"
    ));
    assert!(!reads_the_base(
        "fn f() { let _ = \"xingbiao::scratch_base()\"; }"
    ));
    assert!(!reads_the_base(
        "fn f(helper: Helper) { let _ = helper.scratch_base(); }"
    ));
}

#[test]
fn a_root_composed_without_naming_the_helper_is_not_read() {
    assert!(!reads_the_base(
        "fn f() -> std::path::PathBuf { std::env::current_exe().unwrap().join(\"tmp\") }"
    ));
    assert!(!reads_the_base(
        "fn f() -> std::path::PathBuf { std::path::PathBuf::from(env!(\"CARGO_TARGET_TMPDIR\")).join(\"x\") }"
    ));
}

#[test]
fn a_second_naming_in_a_declared_file_is_not_separated_from_the_first() {
    let once = "fn f() { let _ = xingbiao::scratch_base(); }";
    let twice = "fn f() { let _ = xingbiao::scratch_base(); let _ = xingbiao::scratch_base(); }";
    assert_eq!(names_the_helpers_base(once), names_the_helpers_base(twice));
    assert_eq!(names_the_helpers_base(twice), Reading::Constructs);
}

#[test]
fn a_file_the_base_reader_cannot_parse_is_undecidable() {
    assert_eq!(names_the_helpers_base("fn ("), Reading::Undecidable);
}

#[test]
fn an_unclassifiable_macro_body_naming_the_base_is_undecidable() {
    assert_eq!(
        names_the_helpers_base("fn f() { passthrough!(=> scratch_base => ); }"),
        Reading::Undecidable
    );
    assert_eq!(
        names_the_helpers_base("fn f() { passthrough!(=> scratch_ceiling => ); }"),
        Reading::Undecidable
    );
    assert_eq!(
        names_the_helpers_base("fn f() { passthrough!(=> temp_dir => ); }"),
        Reading::DoesNot
    );
}
