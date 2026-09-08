//! **Every** compiled root of a package is governed: a `main.rs` beside a `lib.rs`, a `src/bin/*.rs`, a
//! `[[bin]] path` inside the source directory, and one outside it.
//!
//! This file previously pinned the opposite — that only the first resolved root was governed — as a
//! stated bound, and said so in both directions so that "if this now reacts, the bound has been closed".
//! It did, in the same window: these tests started failing the moment the per-target corpus landed,
//! which is exactly the transition they were written to detect. They are inverted here rather than
//! deleted, because the direction they now assert is the one an adopter depends on and the one a future
//! regression would silently undo.
//!
//! Pinned at the **real** resolution: a real manifest, real `cargo metadata`, real
//! `xingbiao::crate_root_files`. Each root's violation must be reported with its own file, and — since
//! every root denotes the module path `crate` — with its own compilation-unit identity, so accepting one
//! in a baseline cannot suppress another.
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
        xingbiao::claim_scratch(&dir).expect("the fixture root is writable");
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
fn a_second_crate_root_beside_the_library_is_governed_too() {
    let probe = RootProbe::new(
        "libmain",
        "",
        &[
            ("src/lib.rs", OFFENDING),
            ("src/main.rs", &format!("fn main() {{}}\n{OFFENDING}")),
        ],
    );

    let files = reacting_files(&check(&clock_free("libmain"), probe.manifest()));
    for governed in ["src/lib.rs", "src/main.rs"] {
        assert!(
            files.iter().any(|f| f.ends_with(governed)),
            "{governed} is a compiled root of the package, so its violation must react: {files:?}"
        );
    }
}

#[test]
fn every_binary_target_root_is_governed_wherever_it_lives() {
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
    for governed in ["src/bin/conventional.rs", "src/custom_in_src.rs"] {
        assert!(
            files.iter().any(|f| f.ends_with(governed)),
            "{governed} is a compiled root, so its violation must react — a conventional `src/bin` \
             target and a custom `path` are treated identically: {files:?}"
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

/// A root Cargo reports **twice** yields one violation, not two.
///
/// **This test pins the contract, not a change.** It passed before `xingbiao::crate_root_files` was made
/// totally unique and passes after, and saying so is the point: the reason it passed is that both static
/// dimensions dedup violations by [`xuanji::ViolationId`] before reporting (`guibiao/src/lib.rs`,
/// `hunyi/src/driver.rs`), each for its own unrelated stated reason — two identical boundaries declared
/// on one constitution. That dedup is what kept a duplicated corpus from ever being visible, which is
/// exactly why the duplication survived unnoticed. Measured directly: with `dedup` in place
/// `crate_root_files` returned `[shared.rs, between.rs, shared.rs]` for the manifest below, and this
/// assertion still held.
///
/// It is kept because the property an adopter depends on is this one — a root Cargo names twice is one
/// architectural fact — and because it would now catch the composition failing from the other side, if
/// a consumer's identity dedup were ever removed or narrowed.
///
/// Asserted on the real [`xuanji::Violation::id`] rather than on the reported `file`, because `file`
/// is not identity: two genuinely distinct violations can share one file, so counting files would
/// answer a different question than the one this test asks.
#[test]
fn a_root_cargo_reports_twice_is_scanned_once() {
    let probe = RootProbe::new(
        "twicereported",
        // The target NAMES carry this test, not the declaration order: `cargo metadata` reports
        // targets sorted by name (measured — declaring `first`/`between`/`third` in that order
        // reports them as `between`, `first`, `third`). So the duplicate is separated only if the
        // name that sorts between the two `shared.rs` targets belongs to the OTHER file. `a`, `b`,
        // `c` gives `shared.rs`, `between.rs`, `shared.rs`; naming them `first`/`between`/`third`
        // instead sorts the two duplicates adjacent, where `dedup` does collapse them and this test
        // passes against the very defect it is written to catch.
        "[[bin]]\nname = \"a\"\npath = \"src/shared.rs\"\n\n\
         [[bin]]\nname = \"b\"\npath = \"src/between.rs\"\n\n\
         [[bin]]\nname = \"c\"\npath = \"src/shared.rs\"\n",
        &[
            ("src/shared.rs", &format!("fn main() {{}}\n{OFFENDING}")),
            ("src/between.rs", "fn main() {}\n"),
        ],
    );

    let outcome = check(&clock_free("twicereported"), probe.manifest());
    let Outcome::Violations(report) = &outcome else {
        panic!("expected Violations, got {outcome:?}");
    };

    let mut ids: Vec<_> = report.violations.iter().map(|v| v.id()).collect();
    let total = ids.len();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        total,
        "a root Cargo reports twice must be scanned once: {total} violation(s) carry only \
         {} distinct identities, so the duplicate would be indistinguishable in a baseline — \
         accepting it once accepts it always, and a second real occurrence could never be told \
         apart from the echo. Reported files: {:?}",
        ids.len(),
        reacting_files(&outcome)
    );
    assert_eq!(
        total,
        1,
        "the shared root holds exactly one forbidden call, so exactly one violation is expected: {:?}",
        reacting_files(&outcome)
    );
}

/// The one root shape that is **refused** rather than governed: a target whose source lies outside the
/// package's own directory.
///
/// A violation's identity is labeled by the compilation unit it came from, relative to the package
/// directory — so a root outside that directory has no checkout-independent label, and using the path as
/// given would make the identity depend on where the repository happens to be cloned. That is the defect
/// the label exists to prevent, so this is "cannot judge" (exit 2), the same ordering 漏刻 applies when it
/// refuses a relative or empty anchor.
///
/// Note how narrow this is: `tools/outside.rs` in the test above is outside `src/` and is governed
/// normally, because it is still inside the package. Only a root reached out of the package — a
/// `[[bin]] path = "../…"` — is refused.
#[test]
fn a_target_root_outside_the_package_directory_is_refused_not_labeled() {
    // The shared source lives beside the package, so the package's own directory does not contain it.
    let shared = std::env::temp_dir().join(format!(
        "guibiao-out-of-package-shared-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&shared);
    xingbiao::claim_scratch(&shared).expect("create shared dir");
    std::fs::write(
        shared.join("outside.rs"),
        format!("fn main() {{}}\n{OFFENDING}"),
    )
    .expect("write shared root");

    let probe = RootProbe::new(
        "outofpackage",
        &format!(
            "[[bin]]\nname = \"out\"\npath = {:?}\n",
            shared.join("outside.rs").display().to_string()
        ),
        &[("src/lib.rs", OFFENDING)],
    );

    match check(&clock_free("outofpackage"), probe.manifest()) {
        Outcome::ConstitutionError(message) => {
            assert!(
                message.contains("cannot be judged without a checkout-dependent identity"),
                "expected the out-of-package-root constitution error, got: {message}"
            );
        }
        other => panic!(
            "a target root outside the package directory must be refused, not labeled: {other:?}"
        ),
    }
    let _ = std::fs::remove_dir_all(&shared);
}
