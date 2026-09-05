//! Dogfood gate: every example's product reaction behaves as its documentation declares.
//!
//! An example is a claim about what an adopter gets, and the claim is the **exit code**: a demo that reacts
//! exits 1, a run-mode that only reports events exits 0. Checking that an example merely builds says nothing
//! about either — the reaction it demonstrates could be gone entirely.
//!
//! Every example is built against **local source**, and that the patch took effect is asserted rather than
//! assumed: a `patch.crates-io` that silently fails to apply leaves the example exercising the *published*
//! crates, so the suite would pass while demonstrating nothing about this working tree. That direction is the
//! whole reason the patch is checked instead of trusted.
//!
//! It is gated behind `TIANHENG_EXAMPLES` and named on its own line in the Definition of Done and in CI,
//! because it builds seven separate crate graphs.

use std::path::{Path, PathBuf};
use std::process::Command;

/// What each example declares: the family crates it patches, the binary it runs, and the code that run owes.
struct Example {
    name: &'static str,
    family: &'static [&'static str],
    binary: &'static str,
    args: &'static [&'static str],
    /// `1` where the example exists to react; `0` where its point is that it does not.
    expected: i32,
}

const EXAMPLES: [Example; 7] = [
    Example {
        name: "guibiao-standalone",
        family: &["guibiao", "xuanji", "xingbiao"],
        binary: "demo",
        args: &[],
        expected: 1,
    },
    Example {
        name: "hunyi-standalone",
        family: &["hunyi", "xuanji", "xingbiao"],
        binary: "demo",
        args: &[],
        expected: 1,
    },
    Example {
        name: "unsafe-confinement",
        family: &["hunyi", "xuanji", "xingbiao"],
        binary: "demo",
        args: &[],
        expected: 1,
    },
    Example {
        name: "capability-catalog",
        family: &[
            "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
        ],
        binary: "check",
        args: &["check", "--manifest-path", "Cargo.toml", "--format", "json"],
        expected: 1,
    },
    Example {
        name: "composed",
        family: &[
            "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
        ],
        binary: "runtime_demo",
        args: &[],
        expected: 0,
    },
    Example {
        name: "sans-io-pure",
        family: &[
            "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
        ],
        binary: "check",
        args: &["check", "--manifest-path", "Cargo.toml"],
        expected: 1,
    },
    Example {
        name: "observer-participant",
        family: &[
            "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
        ],
        binary: "demo",
        args: &[],
        expected: 1,
    },
];

/// One isolated quality gate an example must pass.
///
/// **The two properties beside `head` were decided by comparing `label` against a literal**, twelve and
/// sixteen lines from where the label was written, in one expression. So `label` was at once the sentence
/// an operator reads and the dispatch key for whether warnings fail the build — and renaming it in the
/// table, which reads as a wording change, silently dropped `-D warnings`: clippy would still run, still
/// exit `0`, and the gate would go green having stopped reacting. That is the shape the suite's own module
/// doc names, one level up: *checking that an example merely builds says nothing about either — the
/// reaction it demonstrates could be gone entirely.*
///
/// Declared, the label decides nothing, so a rename is a rename.
///
/// Negative run, against the tuple form with `"clippy"` renamed to `"lint"` in the table and the dispatch
/// left comparing against `"clippy"` — which is what that edit does:
///
/// ```text
/// test every_example_passes_its_isolated_quality_gates ... ok
/// test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 2 filtered out
/// ```
///
/// Green, with `-D warnings` gone from every example's clippy run. Nothing else in the tree references
/// these labels, so nothing would have said so.
struct Gate {
    /// Names the gate in the assertion message, and nothing else.
    label: &'static str,
    head: &'static [&'static str],
    /// Whether warnings must fail the build. `-D warnings` is the whole point of the clippy gate.
    denies_warnings: bool,
    /// Whether the example's family patch arguments apply. `fmt` reads the source and resolves nothing.
    takes_the_family_patch: bool,
}

const GATES: [Gate; 3] = [
    Gate {
        label: "fmt",
        head: &["fmt", "--all", "--check"],
        denies_warnings: false,
        takes_the_family_patch: false,
    },
    Gate {
        label: "clippy",
        head: &["clippy", "--all-targets"],
        denies_warnings: true,
        takes_the_family_patch: true,
    },
    Gate {
        label: "doc",
        head: &["doc", "--no-deps"],
        denies_warnings: false,
        takes_the_family_patch: true,
    },
];

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("examples").is_dir(),
        shengmo::workspace::marker_set(),
    )
    // Canonicalised, because this gate COMPARES paths: `cargo metadata` prints a resolved manifest
    // path, and the manifest directory's grandparent is the same directory written differently. Measured —
    // without it every example read as unpatched. The one place a caller's answer differs from the shared
    // locator's, so it stays here rather than becoming an option nobody else would pass.
    .map(|root| std::fs::canonicalize(&root).unwrap_or(root))
}

/// `--config` arguments patching every family crate this example names to local source.
fn patch_args(root: &Path, family: &[&str]) -> Vec<String> {
    family
        .iter()
        .map(|crate_name| {
            format!(
                "--config=patch.crates-io.{crate_name}.path=\"{}\"",
                root.join("crates").join(crate_name).display()
            )
        })
        .collect()
}

fn cargo(dir: &Path, args: &[String]) -> (Option<i32>, String) {
    let out = Command::new("cargo")
        .args(args)
        .current_dir(dir)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .unwrap_or_else(|err| panic!("cannot run cargo {args:?}: {err}"));
    (
        out.status.code(),
        format!(
            "{}{}",
            String::from_utf8_lossy(&out.stdout),
            String::from_utf8_lossy(&out.stderr)
        ),
    )
}

fn argv(head: &[&str], patch: &[String], tail: &[&str]) -> Vec<String> {
    let mut args: Vec<String> = head.iter().map(|s| s.to_string()).collect();
    args.extend(patch.iter().cloned());
    args.extend(tail.iter().map(|s| s.to_string()));
    args
}

/// Every family crate this example names must resolve to **local source**, not to a registry version.
///
/// A `patch.crates-io` that fails to apply is silent: cargo warns and resolves the published crate, and the
/// example then demonstrates a release rather than this tree. `cargo metadata` is asked which it got.
fn assert_patched(root: &Path, dir: &Path, name: &str, family: &[&str], patch: &[String]) {
    let (code, output) = cargo(
        dir,
        &argv(&["metadata", "--format-version", "1"], patch, &[]),
    );
    assert_eq!(code, Some(0), "{name}: cargo metadata failed:\n{output}");
    for crate_name in family {
        // The decisive evidence, not a heuristic over field order: the resolved package's manifest is THIS
        // tree's. A window scan around the name field reads whichever fields cargo happened to emit next.
        let local = root
            .join("crates")
            .join(crate_name)
            .join("Cargo.toml")
            .display()
            .to_string();
        assert!(
            output.contains(&format!("\"manifest_path\":\"{local}\"")),
            "{name}: {crate_name} did not resolve to local source — patch.crates-io was silently unused, so \
             this example would demonstrate the published crate rather than this working tree"
        );
    }
}

#[test]
fn every_example_reacts_as_declared() {
    let Some(root) = workspace_root() else {
        return;
    };
    if std::env::var_os("TIANHENG_EXAMPLES").is_none() {
        eprintln!(
            "examples: skipped — set TIANHENG_EXAMPLES=1 to run it. It is named on its own line in the \
             Definition of Done and in CI, so skipping here is a cost decision rather than a hole."
        );
        return;
    }

    for example in &EXAMPLES {
        let dir = root.join("examples").join(example.name);
        assert!(
            dir.join("Cargo.toml").is_file(),
            "examples/{} carries no manifest, so the declaration above names something absent",
            example.name
        );
        let patch = patch_args(&root, example.family);
        assert_patched(&root, &dir, example.name, example.family, &patch);

        let (code, output) = cargo(&dir, &argv(&["test"], &patch, &[]));
        assert_eq!(
            code,
            Some(0),
            "{}: its own tests fail:\n{output}",
            example.name
        );

        let (code, output) = cargo(
            &dir,
            &argv(&["run", "--quiet", "--bin", example.binary], &patch, &{
                let mut tail = vec!["--"];
                tail.extend(example.args.iter().copied());
                tail
            }),
        );
        assert_eq!(
            code,
            Some(example.expected),
            "{}: `{}` exited {code:?} where its documentation declares {} — an example that stops reacting \
             is a claim about what an adopter gets that has quietly stopped being true:\n{output}",
            example.name,
            example.binary,
            example.expected
        );
    }
    eprintln!("examples ok ({} reacted as declared)", EXAMPLES.len());
}

/// Each example passes fmt, Clippy and rustdoc **in isolation**, which the workspace passes cannot see:
/// `examples/` is excluded from the workspace precisely so an adopter's build is what is tested.
#[test]
fn every_example_passes_its_isolated_quality_gates() {
    let Some(root) = workspace_root() else {
        return;
    };
    if std::env::var_os("TIANHENG_EXAMPLES").is_none() {
        return;
    }
    for example in &EXAMPLES {
        let dir = root.join("examples").join(example.name);
        let patch = patch_args(&root, example.family);
        for gate in &GATES {
            let tail: &[&str] = if gate.denies_warnings {
                &["--", "-D", "warnings"]
            } else {
                &[]
            };
            let none: [String; 0] = [];
            let args = if gate.takes_the_family_patch {
                argv(gate.head, &patch, tail)
            } else {
                argv(gate.head, &none, tail)
            };
            let (code, output) = cargo(&dir, &args);
            assert_eq!(
                code,
                Some(0),
                "{}: isolated {} fails:\n{output}",
                example.name,
                gate.label
            );
        }
    }
}

/// The declared example set equals the tracked example directories, in both directions.
///
/// A directory present under `examples/` and absent from [`EXAMPLES`] is exercised by **neither** of this
/// suite's directions nor by the workflow job that runs them. That is a false negative in the gate that runs
/// the product against itself — the one gate whose silence is least likely to be questioned, because a green
/// dogfood reads as the strongest evidence there is.
///
/// The reverse matters too and is not symmetry for its own sake: an entry naming a directory the tree no longer
/// carries reads as coverage while defending nothing, which is the shape this repository's register refuses for
/// citations.
///
/// Enumerated from **tracked** content, so an untracked scratch directory is neither a failure nor an example
/// — the rule every sibling direction here follows.
///
/// This is the same guard `every_gate_running_wrapper_is_named` already applies to the wrapper constant. That
/// one direction existing while three sibling constants had none is what made this a class rather than an
/// oversight.
#[test]
fn every_tracked_example_is_declared_and_every_declaration_exists() {
    let Some(root) = workspace_root() else {
        return;
    };
    let out = Command::new("git")
        .args(["ls-files", "examples"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files examples");
    assert!(
        out.status.success(),
        "`git ls-files examples` failed, and a failed enumeration is not a repository with no examples"
    );
    let tracked: std::collections::BTreeSet<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter(|path| path.ends_with("/Cargo.toml"))
        .filter_map(|path| path.strip_prefix("examples/"))
        .filter_map(|rest| rest.split_once('/'))
        .map(|(directory, _)| directory.to_string())
        .collect();
    assert!(
        !tracked.is_empty(),
        "no tracked example was enumerated, so this direction would hold over nothing"
    );

    let declared: std::collections::BTreeSet<String> =
        EXAMPLES.iter().map(|e| e.name.to_string()).collect();

    let undeclared: Vec<&String> = tracked.difference(&declared).collect();
    assert!(
        undeclared.is_empty(),
        "these tracked examples are declared by nothing, so neither direction of this suite nor the workflow \
         job that runs it exercises them: {undeclared:?}"
    );
    let absent: Vec<&String> = declared.difference(&tracked).collect();
    assert!(
        absent.is_empty(),
        "these declarations name no tracked example directory, so they read as coverage while defending \
         nothing: {absent:?}"
    );
}
