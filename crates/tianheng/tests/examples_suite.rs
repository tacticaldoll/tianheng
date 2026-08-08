//! Self-governance reaction: every example reacts as its documentation declares.
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

fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("examples").is_dir() {
        // Canonicalised, because this reaction COMPARES paths: `cargo metadata` prints a resolved manifest
        // path, and the manifest directory's grandparent is the same directory written differently. Measured — without it
        // every example read as unpatched.
        return Some(std::fs::canonicalize(&root).unwrap_or(root));
    }
    assert!(
        !marker_set,
        "examples/ expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — a governance \
         reaction that quietly does nothing in CI is the shape this family argues against"
    );
    None
}

fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
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
        for (label, head) in [
            ("fmt", vec!["fmt", "--all", "--check"]),
            ("clippy", vec!["clippy", "--all-targets"]),
            ("doc", vec!["doc", "--no-deps"]),
        ] {
            let tail: Vec<&str> = if label == "clippy" {
                vec!["--", "-D", "warnings"]
            } else {
                vec![]
            };
            let args = if label == "fmt" {
                argv(&head, &[], &tail)
            } else {
                argv(&head, &patch, &tail)
            };
            let (code, output) = cargo(&dir, &args);
            assert_eq!(
                code,
                Some(0),
                "{}: isolated {label} fails:\n{output}",
                example.name
            );
        }
    }
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-examples-suite-absent");
    let _ = std::fs::remove_dir_all(&absent);
    assert!(locate_layout(absent.clone(), false).is_none());
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}
