//! Repository check: a fixture root is the helper's, never built from the system temporary directory.
//!
//! `xingbiao::scratch_root` and `xingbiao::scratch_base` place every fixture under the build directory, where
//! the user owns the directory and `cargo clean` reaches it. A tracked Rust file that asks the system for its
//! temporary directory builds a root outside that, and a root there is claimable by anyone who can write the
//! directory and fails with it when the directory stops being writable.
//!
//! **This file answers one question — which tracked files construct a root from the system — and never
//! whether a run is isolated.** `hermetic_invocations` records why deciding isolation by reading source is the
//! wrong instrument; the same loop reads the files (`support::tracked_rust`), the same three-state answer says
//! when a file could not be decided, and a file that cannot be parsed is refused rather than reported clean.
//!
//! **Why 圭表 does not hold this.** It scans compilation units reached from library and binary roots, and most
//! fixture roots are written in `tests/*.rs`, which it never reads.
//!
//! **What the reader names:** a path whose last segment is `temp_dir`, however it is qualified, imported or
//! renamed on import, and a read of the `TMPDIR` variable through `var`, `var_os`, `env!` or `option_env!`.
//! What it cannot see is declared as a bound in `openspec/specs/repository-checks`.

mod support;

use std::collections::BTreeSet;
use support::tracked_rust::{Reading, files_where, string_of, workspace_root};

/// Every file that may ask the system for a temporary directory, and why.
///
/// Empty: the helper reads the running executable's path and asks the system for nothing. A site that gains
/// one must be named with why; the comparison is two-directional, so a name that outlives its site must go.
const NAMES_A_SYSTEM_TEMP_DIR: [(&str, &str); 0] = [];

/// What this reader found in one file.
#[derive(Default)]
struct Roots {
    found: bool,
    /// A macro body neither grammar parsed, naming `temp_dir` or the `TMPDIR` variable.
    undecided: bool,
}

fn names_the_system_temp(tokens: proc_macro2::TokenStream) -> bool {
    let mut pending = vec![tokens];
    while let Some(stream) = pending.pop() {
        for tree in stream {
            match tree {
                proc_macro2::TokenTree::Group(group) => pending.push(group.stream()),
                proc_macro2::TokenTree::Ident(word) if word == "temp_dir" => return true,
                proc_macro2::TokenTree::Literal(literal) => {
                    if syn::parse_str::<syn::LitStr>(&literal.to_string())
                        .is_ok_and(|text| text.value() == "TMPDIR")
                    {
                        return true;
                    }
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

impl<'ast> syn::visit::Visit<'ast> for Roots {
    /// A path ending in `temp_dir`, called or passed as a value: `temp_dir()`, `env::temp_dir()`,
    /// `std::env::temp_dir()`, `.map(std::env::temp_dir)`.
    fn visit_expr_path(&mut self, node: &'ast syn::ExprPath) {
        if last_segment(&node.path).as_deref() == Some("temp_dir") {
            self.found = true;
        }
        syn::visit::visit_expr_path(self, node);
    }

    /// An import of `temp_dir`, which a later bare or renamed call is not required to spell the path of.
    fn visit_use_name(&mut self, node: &'ast syn::UseName) {
        if node.ident == "temp_dir" {
            self.found = true;
        }
    }

    fn visit_use_rename(&mut self, node: &'ast syn::UseRename) {
        if node.ident == "temp_dir" {
            self.found = true;
        }
    }

    /// A read of `TMPDIR`: `var("TMPDIR")` or `var_os("TMPDIR")` under any qualification. A call that *sets* a
    /// child's `TMPDIR` is a method call and not a path call, so it is not this.
    fn visit_expr_call(&mut self, node: &'ast syn::ExprCall) {
        if let syn::Expr::Path(callee) = &*node.func
            && matches!(
                last_segment(&callee.path).as_deref(),
                Some("var" | "var_os")
            )
            && node
                .args
                .iter()
                .any(|arg| string_of(arg).as_deref() == Some("TMPDIR"))
        {
            self.found = true;
        }
        syn::visit::visit_expr_call(self, node);
    }

    /// `env!("TMPDIR")` and `option_env!("TMPDIR")` are read here; a macro body that is neither an expression
    /// list nor statements and names `temp_dir` or the variable is undecidable rather than clean.
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        use syn::punctuated::Punctuated;
        let reads_env = matches!(
            last_segment(&node.path).as_deref(),
            Some("env" | "option_env")
        );
        let as_expressions =
            node.parse_body_with(Punctuated::<syn::Expr, syn::Token![,]>::parse_terminated);
        if let Ok(arguments) = as_expressions {
            if reads_env
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
        } else if names_the_system_temp(node.tokens.clone()) {
            self.undecided = true;
        }
        syn::visit::visit_macro(self, node);
    }
}

/// Whether `text` builds a root from the system, or says it could not decide.
fn names_a_system_root(text: &str) -> Reading {
    let Ok(parsed) = syn::parse_file(text) else {
        return Reading::Undecidable;
    };
    let mut roots = Roots::default();
    syn::visit::Visit::visit_file(&mut roots, &parsed);
    if roots.found {
        Reading::Constructs
    } else if roots.undecided {
        Reading::Undecidable
    } else {
        Reading::DoesNot
    }
}

fn reads(text: &str) -> bool {
    matches!(names_a_system_root(text), Reading::Constructs)
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
