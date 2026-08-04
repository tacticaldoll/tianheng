//! The governed corpus is **one** crate root — the resolved one — and the modules reachable from it.
//! Every other compiled root of the same package is out of scope: a `main.rs` beside a `lib.rs`, a
//! `src/bin/*.rs`, a `[[bin]] path` inside the source directory, and one outside it.
//!
//! This is a **stated bound**, pinned rather than described, because the shape it affects is the most
//! ordinary one in Rust and because three surfaces used to claim otherwise:
//! `module-boundary` asserted that "both crate roots (`lib.rs` and `main.rs`) resolve to `crate`",
//! `module_check.rs` repeated it to justify a dedup step, and a unit test named for the claim could
//! not distinguish "two roots deduplicated" from "one root scanned" — its synthetic metadata declared
//! only one target, so its assertion held either way.
//!
//! Pinned at the **real** resolution: a real manifest, real `cargo metadata`, real
//! `xingbiao::crate_root_file` (which returns the first library-kind target, else the first `bin`).
//! Both directions are asserted — the governed root reacts, the others are silent — so if the bound is
//! ever closed this fails and the specification, the comment, and `BACKLOG.md` must move with it.
use std::path::{Path, PathBuf};

use guibiao::{Constitution, ModuleBoundary, Outcome, check};

/// A real single-package workspace with a real manifest, so the root resolution under test is the one
/// adopters get rather than a synthetic `targets` array.
struct RootProbe {
    dir: PathBuf,
    manifest: PathBuf,
}

impl RootProbe {
    fn new(name: &str, manifest_extra: &str, files: &[(&str, &str)]) -> Self {
        use std::sync::atomic::{AtomicU32, Ordering};
        static COUNTER: AtomicU32 = AtomicU32::new(0);
        let unique = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!(
            "guibiao-single-root-{name}-{}-{unique}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("src")).expect("create src dir");
        let manifest = dir.join("Cargo.toml");
        std::fs::write(
            &manifest,
            format!(
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n\
                 {manifest_extra}\n[workspace]\n"
            ),
        )
        .expect("write Cargo.toml");
        for (relative, contents) in files {
            let target = dir.join(relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent).expect("create parent dir");
            }
            std::fs::write(target, contents).expect("write source file");
        }
        Self { dir, manifest }
    }

    fn manifest(&self) -> &Path {
        &self.manifest
    }
}

impl Drop for RootProbe {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The same forbidden construct in every root, so which roots react is the only variable.
const OFFENDING: &str = "pub fn touch() { let _ = std::fs::canonicalize(\".\"); }\n";

fn clock_free(package: &str) -> Constitution {
    Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_call_inline("std::fs")
            .ending_with(["canonicalize"])
            .depth(xuanji::ScanDepth::Subtree)
            .because("the governed corpus is the resolved crate root and what it reaches"),
    )
}

fn reacting_files(outcome: &Outcome) -> Vec<String> {
    match outcome {
        Outcome::Violations(report) => report
            .violations
            .iter()
            .filter_map(|v| v.file.clone())
            .collect(),
        other => panic!("expected Violations, got {other:?}"),
    }
}

#[test]
fn a_second_crate_root_beside_the_library_is_not_observed() {
    let probe = RootProbe::new(
        "libmain",
        "",
        &[
            ("src/lib.rs", OFFENDING),
            ("src/main.rs", &format!("fn main() {{}}\n{OFFENDING}")),
        ],
    );

    let files = reacting_files(&check(&clock_free("libmain"), probe.manifest()));
    assert!(
        files.iter().any(|f| f.ends_with("src/lib.rs")),
        "the resolved library root must react: {files:?}"
    );
    assert!(
        !files.iter().any(|f| f.ends_with("src/main.rs")),
        "STATED BOUND: `main.rs` beside a `lib.rs` is not the resolved root and is not observed. If \
         this now reacts, the bound has been closed — update `module-boundary`'s \
         single-governed-root requirement, the dedup comment in `module_check.rs`, and `BACKLOG.md` \
         together with it: {files:?}"
    );
}

#[test]
fn no_binary_target_root_is_observed_wherever_it_lives() {
    let probe = RootProbe::new(
        "binroots",
        "[[bin]]\nname = \"custom_in_src\"\npath = \"src/custom_in_src.rs\"\n\n\
         [[bin]]\nname = \"custom_outside\"\npath = \"tools/outside.rs\"\n",
        &[
            ("src/lib.rs", OFFENDING),
            (
                "src/bin/conventional.rs",
                &format!("fn main() {{}}\n{OFFENDING}"),
            ),
            (
                "src/custom_in_src.rs",
                &format!("fn main() {{}}\n{OFFENDING}"),
            ),
            ("tools/outside.rs", &format!("fn main() {{}}\n{OFFENDING}")),
        ],
    );

    let files = reacting_files(&check(&clock_free("binroots"), probe.manifest()));
    assert!(
        files.iter().any(|f| f.ends_with("src/lib.rs")),
        "the resolved library root must react: {files:?}"
    );
    for unobserved in [
        "src/bin/conventional.rs",
        "src/custom_in_src.rs",
        "tools/outside.rs",
    ] {
        assert!(
            !files.iter().any(|f| f.ends_with(unobserved)),
            "STATED BOUND: {unobserved} is a binary target's own root, not the resolved one, so it \
             is not observed — identically for a conventional `src/bin` target and for a custom \
             `path`, inside the source directory or outside it: {files:?}"
        );
    }
}

#[test]
fn a_package_with_no_library_governs_its_first_binary_root() {
    // The other half of the resolution, so the bound above reads as scope rather than as "binaries
    // are never governed": with no library target, the first `bin` IS the resolved root.
    let probe = RootProbe::new(
        "binonly",
        "",
        &[("src/main.rs", &format!("fn main() {{}}\n{OFFENDING}"))],
    );

    let files = reacting_files(&check(&clock_free("binonly"), probe.manifest()));
    assert!(
        files.iter().any(|f| f.ends_with("src/main.rs")),
        "with no library target, the first binary root is the governed one: {files:?}"
    );
}
