//! **Every** compiled root of a package is governed: a `main.rs` beside a `lib.rs`, a `src/bin/*.rs`, a
//! `[[bin]] path` inside the source directory, and one outside it.
//!
//! The directions here assert the governed side of each root on purpose: it is the one an adopter depends
//! on, and the one a regression would silently undo — a root dropped from the corpus reports nothing, which
//! reads exactly as a clean root.
//!
//! Pinned at the **real** resolution: a real manifest, real `cargo metadata`, real
//! `xingbiao::crate_roots`. Each root's violation must be reported with its own file, and — since
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
        Self::with_edition(name, "2021", manifest_extra, files)
    }

    /// Every file a probe writes, a path dependency's included, lands beneath its own `dir`, so the one
    /// `remove_dir_all` in `Drop` removes all of it.
    fn with_edition(
        name: &str,
        edition: &str,
        manifest_extra: &str,
        files: &[(&str, &str)],
    ) -> Self {
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
                "[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"{edition}\"\n\
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

    fn dir(&self) -> &Path {
        &self.dir
    }
}

impl Drop for RootProbe {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

/// The same forbidden construct in every root, so which roots react is the only variable.
const OFFENDING: &str = "pub fn touch() { let _ = std::fs::canonicalize(\".\"); }\n";

fn root_scope_fs_law(package: &str) -> Constitution {
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

    let files = reacting_files(&check(&root_scope_fs_law("libmain"), probe.manifest()));
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

    let files = reacting_files(&check(&root_scope_fs_law("binroots"), probe.manifest()));
    assert!(
        files.iter().any(|f| f.ends_with("src/lib.rs")),
        "the resolved library root must react: {files:?}"
    );
    for governed in [
        "src/bin/conventional.rs",
        "src/custom_in_src.rs",
        "tools/outside.rs",
    ] {
        assert!(
            files.iter().any(|f| f.ends_with(governed)),
            "{governed} is a compiled root, so its violation must react — a conventional `src/bin` \
             target and a custom `path` are treated identically: {files:?}"
        );
    }
}

#[test]
fn a_package_with_no_library_governs_its_binary_root() {
    // With no library target the binary root is still a root, so it is governed like any other.
    let probe = RootProbe::new(
        "binonly",
        "",
        &[("src/main.rs", &format!("fn main() {{}}\n{OFFENDING}"))],
    );

    let files = reacting_files(&check(&root_scope_fs_law("binonly"), probe.manifest()));
    assert!(
        files.iter().any(|f| f.ends_with("src/main.rs")),
        "with no library target, the binary root is still a compiled root and is governed: {files:?}"
    );
}

/// A root Cargo reports **twice** yields one violation, not two.
///
/// **This test pins the contract, not a change.** It passed before `xingbiao::crate_roots` was made
/// totally unique and passes after, and saying so is the point: the reason it passed is that both static
/// dimensions dedup violations by [`xuanji::ViolationId`] before reporting (`guibiao/src/lib.rs`,
/// `hunyi/src/driver.rs`), each for its own unrelated stated reason — two identical boundaries declared
/// on one constitution. That dedup is what kept a duplicated corpus from ever being visible, which is
/// exactly why the duplication survived unnoticed. Measured directly: with `dedup` in place
/// the root reader returned `[shared.rs, between.rs, shared.rs]` for the manifest below, and this
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

    let outcome = check(&root_scope_fs_law("twicereported"), probe.manifest());
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

    match check(&root_scope_fs_law("outofpackage"), probe.manifest()) {
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

/// A constitution over one named package.
type Law = fn(&str) -> Constitution;

/// `brick` is permitted only inside `crate::seam`, which the library root declares.
fn confined_to_seam(package: &str) -> Constitution {
    Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate::seam")
            .confine_external_crate("brick")
            .because("brick enters the package only through the seam"),
    )
}

const INLINE_IN_THE_BINARY: &[(&str, &str)] = &[
    ("src/lib.rs", "pub mod seam;\npub mod forbidden;\n"),
    ("src/seam.rs", "\n"),
    ("src/forbidden.rs", "\n"),
    (
        "src/main.rs",
        "mod seam {\n    use brick::B;\n}\nfn main() {}\n",
    ),
];

const INLINE_IN_THE_LIBRARY: &[(&str, &str)] = &[
    (
        "src/lib.rs",
        "pub mod forbidden;\npub mod seam {\n    use crate::forbidden::X;\n}\n",
    ),
    ("src/forbidden.rs", "pub struct X;\n"),
    ("src/main.rs", "mod seam;\nfn main() {}\n"),
    ("src/seam.rs", "\n"),
];

/// A confinement's perimeter is the whole package; the permitted module is only the region inside it
/// where the import is allowed. A root whose graph has no such module therefore has an **empty**
/// permitted region, so a direct import of the confined crate there is a violation, not a skip.
#[test]
fn a_confined_import_in_a_root_without_the_permitted_module_reacts() {
    let probe = RootProbe::new(
        "confinebin",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "pub use brick::B;\n"),
            (
                "src/main.rs",
                "use brick::B;\nfn main() {\n    let _ = B;\n}\n",
            ),
        ],
    );

    let outcome = check(&confined_to_seam("confinebin"), probe.manifest());
    let Outcome::Violations(report) = &outcome else {
        panic!(
            "the binary root imports the confined crate outside the seam, so it must react: {outcome:?}"
        );
    };
    assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
    let violation = &report.violations[0];
    assert_eq!(violation.target(), "brick");
    assert_eq!(violation.finding, "crate");
    assert!(
        violation
            .file
            .as_deref()
            .is_some_and(|f| f.ends_with("src/main.rs")),
        "the offending file is the binary root: {violation:?}"
    );
    assert!(
        format!("{:?}", violation.fact()).contains("src/main.rs"),
        "the identity carries the binary root's compilation unit, so a baseline of a library \
         finding cannot suppress it: {:?}",
        violation.fact()
    );
    assert_eq!(violation.polarity, Some(xuanji::Polarity::AllowlistGap));
}

/// A module that one root declares **inline** is present in that root, not absent from it, so it is
/// not deferred to a sibling root that backs it with a file. An inline module cannot be a governed
/// target, and that refusal holds whichever other roots exist — for every rule family, because the
/// deferral being refused is the shared one.
#[test]
fn an_inline_target_in_one_root_is_refused_even_when_another_root_backs_it_with_a_file() {
    let restrict = |package: &str| {
        Constitution::new("root-scope").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::seam")
                .must_not_import("crate::forbidden")
                .because("the seam does not reach the forbidden module"),
        )
    };
    let cases: [(&str, Law); 2] = [
        ("inlineconfine", confined_to_seam),
        ("inlinerestrict", restrict),
    ];
    for (package, law) in cases {
        for (side, files) in [
            ("binary", INLINE_IN_THE_BINARY),
            ("library", INLINE_IN_THE_LIBRARY),
        ] {
            let package = format!("{package}{side}");
            let probe = RootProbe::new(&package, "", files);
            match check(&law(&package), probe.manifest()) {
                Outcome::ConstitutionError(message) => {
                    assert!(
                        message.contains("inline"),
                        "{package}: expected the inline-target refusal, got: {message}"
                    );
                    assert!(
                        message.contains("compilation unit"),
                        "{package}: expected the inline-target refusal to name the compilation unit, got: {message}"
                    );
                }
                other => panic!(
                    "{package}: the {side} root declares the target inline, which is not a \
                     governable target and must not be deferred to a root backing it with a file: \
                     {other:?}"
                ),
            }
        }
    }
}

/// The inline-target refusal names the responsible compilation unit and suggests an
/// extraction path within that root's layout that rustc actually resolves
/// ({root directory}/{leaf}.rs), covering a bin main.rs, a library lib.rs, a binary in
/// src/bin/*.rs, and a module nested in the library.
#[test]
fn an_inline_target_refusal_names_its_root_and_a_path_rustc_resolves() {
    let inline_in_the_tool_binary: &[(&str, &str)] = &[
        ("src/lib.rs", "pub mod seam;\npub mod forbidden;\n"),
        ("src/seam.rs", "\n"),
        ("src/forbidden.rs", "\n"),
        (
            "src/bin/tool.rs",
            "mod seam {\n    use brick::B;\n}\nfn main() {}\n",
        ),
    ];
    let inline_nested_in_the_library: &[(&str, &str)] = &[
        ("src/lib.rs", "pub mod outer;\n"),
        ("src/outer.rs", "pub mod inner {\n    use brick::B;\n}\n"),
    ];

    let confined_to = |package: &str, module: &str| {
        Constitution::new("root-scope").boundary(
            ModuleBoundary::in_crate(package)
                .module(module)
                .confine_external_crate("brick")
                .because("brick enters the package only through the permitted module"),
        )
    };

    for (package_suffix, manifest_extra, files, target_module, expected_unit, expected_path) in [
        (
            "binmain",
            "",
            INLINE_IN_THE_BINARY,
            "crate::seam",
            "src/main.rs",
            "src/seam.rs",
        ),
        (
            "lib",
            "",
            INLINE_IN_THE_LIBRARY,
            "crate::seam",
            "src/lib.rs",
            "src/seam.rs",
        ),
        (
            "toolbin",
            "",
            inline_in_the_tool_binary,
            "crate::seam",
            "src/bin/tool.rs",
            "src/bin/seam.rs",
        ),
        (
            "nestedlib",
            "",
            inline_nested_in_the_library,
            "crate::outer::inner",
            "src/lib.rs",
            "src/outer/inner.rs",
        ),
    ] {
        let package = format!("inlinerefusal{package_suffix}");
        let probe = RootProbe::new(&package, manifest_extra, files);
        match check(&confined_to(&package, target_module), probe.manifest()) {
            Outcome::ConstitutionError(message) => {
                assert!(
                    message.contains("inline"),
                    "{package}: expected the inline-target refusal, got: {message}"
                );
                assert!(
                    message.contains(&format!("`{expected_path}`")),
                    "{package}: expected suggested path `{expected_path}`, got: {message}"
                );
                assert!(
                    message.contains(&format!("in compilation unit '{expected_unit}'")),
                    "{package}: expected refusal to name compilation unit '{expected_unit}', \
                     got: {message}"
                );
            }
            other => panic!("{package}: expected the inline-target refusal: {other:?}"),
        }
    }
}

/// A package whose every target is an example compiles no root, so a boundary over it is refused rather than
/// judged over a `src/` nothing builds.
///
/// Cargo reports the example as the package's one target; the library source beside it is compiled by no target,
/// so an import there is in no root's corpus and a violation read from it would be a finding about nothing built.
#[test]
fn a_package_whose_targets_compile_no_root_is_refused() {
    let probe = RootProbe::new(
        "noncompiled",
        concat!(
            "autolib = false\n",
            "autobins = false\n",
            "[[example]]\n",
            "name = \"dummy\"\n",
            "path = \"examples/dummy.rs\"\n",
        ),
        &[
            ("examples/dummy.rs", "fn main() {}\n"),
            ("src/lib.rs", "pub mod seam;\nuse brick::B;\n"),
            ("src/seam.rs", "\n"),
        ],
    );
    match check(&confined_to_seam("noncompiled"), probe.manifest()) {
        Outcome::ConstitutionError(message) => assert!(
            message.contains("has none: no target Cargo reports for it is a library or a binary"),
            "expected the no-compiled-root refusal, got: {message}"
        ),
        other => panic!("expected the no-compiled-root refusal, got {other:?}"),
    }
}

/// An example root importing the confined crate is not governed — a stated bound, shown rather than described.
///
/// `module-boundary/an-example-test-bench-or-build-script-root-is-not-governed-a-stated-bound`: the library root
/// is judged and clean, and the example beside it compiles its own `crate` that no boundary reads.
#[test]
fn an_example_root_is_not_governed() {
    let outcome = confinement_report(
        "exampleroot",
        "[[example]]\nname = \"e\"\npath = \"examples/e.rs\"\n",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("examples/e.rs", "use brick::B;\nfn main() {}\n"),
        ],
    );
    assert!(
        matches!(outcome, Outcome::Clean(_)),
        "an example root is outside the governed corpus, got {outcome:?}"
    );
}

fn confinement_report(package: &str, manifest_extra: &str, files: &[(&str, &str)]) -> Outcome {
    let probe = RootProbe::new(package, manifest_extra, files);
    check(&confined_to_seam(package), probe.manifest())
}

/// The finding is the importing module, so a confined import reached from the binary root's own
/// submodule names that submodule rather than the root.
#[test]
fn a_confined_import_in_a_binary_roots_submodule_names_that_submodule() {
    let outcome = confinement_report(
        "confinebinsub",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("src/main.rs", "mod cli;\nfn main() {}\n"),
            ("src/cli.rs", "use brick::B;\n"),
        ],
    );
    let Outcome::Violations(report) = &outcome else {
        panic!("expected Violations, got {outcome:?}");
    };
    let findings: Vec<_> = report
        .violations
        .iter()
        .map(|v| v.finding.as_str())
        .collect();
    assert_eq!(findings, ["crate::cli"], "{:?}", report.violations);
}

/// Every binary root without the permitted module is its own empty region, and each reacts under its
/// own compilation unit — so accepting one in a baseline cannot accept another.
#[test]
fn every_binary_root_without_the_permitted_module_reacts_under_its_own_unit() {
    let outcome = confinement_report(
        "confinebins",
        "[[bin]]\nname = \"custom\"\npath = \"src/custom.rs\"\n",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("src/bin/tool.rs", "use brick::B;\nfn main() {}\n"),
            ("src/custom.rs", "use brick::B;\nfn main() {}\n"),
        ],
    );
    let files = reacting_files(&outcome);
    for root in ["src/bin/tool.rs", "src/custom.rs"] {
        assert_eq!(
            files.iter().filter(|f| f.ends_with(root)).count(),
            1,
            "{root} is a root without the permitted module, so its import reacts once: {files:?}"
        );
    }
    let Outcome::Violations(report) = &outcome else {
        unreachable!("reacting_files accepted it")
    };
    let mut ids: Vec<_> = report.violations.iter().map(|v| v.id()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(
        ids.len(),
        2,
        "two roots, two identities: {:?}",
        report.violations
    );
}

/// Each spelling the scanner reads as an import of the confined crate reacts in a root without the
/// permitted module, not only the plain one.
#[test]
fn every_import_spelling_of_the_confined_crate_reacts_in_a_root_without_the_permitted_module() {
    for (package, main) in [
        ("spelllead", "use ::brick::B;\nfn main() {}\n"),
        ("spellglob", "use brick::*;\nfn main() {}\n"),
        ("spellbare", "use brick;\nfn main() {}\n"),
    ] {
        let outcome = confinement_report(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod seam;\n"),
                ("src/seam.rs", "\n"),
                ("src/main.rs", main),
            ],
        );
        assert_eq!(
            outcome.exit_code(),
            1,
            "{package}: {main:?} must react: {outcome:?}"
        );
    }
}

/// A root without the permitted module is judged at the boundary's own severity, like any other.
#[test]
fn a_warn_confinement_reports_a_binary_root_import_without_failing() {
    let probe = RootProbe::new(
        "confinewarn",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("src/main.rs", "use brick::B;\nfn main() {}\n"),
        ],
    );
    let law = Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate("confinewarn")
            .module("crate::seam")
            .confine_external_crate("brick")
            .warn()
            .because("brick enters the package only through the seam"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
    assert_eq!(
        reacting_files(&outcome).len(),
        1,
        "the advisory is still reported"
    );
}

/// A baseline accepting one binary root's finding does not accept a second root's.
#[test]
fn a_baselined_binary_root_finding_does_not_mask_a_new_root() {
    let probe = RootProbe::new(
        "confinebase",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("src/main.rs", "use brick::B;\nfn main() {}\n"),
        ],
    );
    let law = confined_to_seam("confinebase");
    let Outcome::Violations(accepted) = check(&law, probe.manifest()) else {
        panic!("the binary root's import must react before it can be baselined");
    };
    let baseline = xuanji::Baseline::of(&accepted);

    std::fs::create_dir_all(probe.dir.join("src/bin")).expect("create src/bin");
    std::fs::write(
        probe.dir.join("src/bin/tool.rs"),
        "use brick::B;\nfn main() {}\n",
    )
    .expect("write the new root");
    let Outcome::Violations(mut report) = check(&law, probe.manifest()) else {
        panic!("both roots import the confined crate");
    };
    xuanji::apply_baseline(&mut report, &baseline);
    let active: Vec<_> = report
        .violations
        .iter()
        .filter(|v| !v.baselined)
        .filter_map(|v| v.file.clone())
        .collect();
    assert_eq!(active.len(), 1, "{:?}", report.violations);
    assert!(active[0].ends_with("src/bin/tool.rs"), "{active:?}");
}

/// What a root without the permitted module may import without reacting: another crate, the package's
/// own library, a local module that shadows the confined crate's name, and a mention inside a string.
///
/// Kept for the contract rather than the change: these were clean before roots without the permitted
/// module were judged, because they were not judged at all. They pin the precision of the judgement
/// that now runs there.
#[test]
fn a_root_without_the_permitted_module_stays_clean_without_a_confined_import() {
    for (package, main, extra) in [
        ("cleanother", "use other::X;\nfn main() {}\n", None),
        (
            "cleanownlib",
            "use cleanownlib::seam::B;\nfn main() {}\n",
            None,
        ),
        (
            "cleanshadow",
            "mod brick;\nuse brick::helper;\nfn main() {}\n",
            Some(("src/brick.rs", "pub fn helper() {}\n")),
        ),
        (
            "cleanstring",
            "fn main() {\n    let _s = \"use brick::B;\";\n}\n",
            None,
        ),
    ] {
        let mut files = vec![
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "pub use brick::B;\n"),
            ("src/main.rs", main),
        ];
        files.extend(extra);
        let outcome = confinement_report(package, "", &files);
        assert_eq!(outcome.exit_code(), 0, "{package}: {outcome:?}");
    }
}

/// Where every root declares the permitted module and imports the confined crate only there, nothing
/// changes: no root is without the module, so no empty region exists. Kept for the contract: it passes
/// with or without the empty-region judgement, and pins that the judgement adds nothing here.
#[test]
fn roots_that_each_declare_the_permitted_module_are_clean() {
    let outcome = confinement_report(
        "confineboth",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "pub use brick::B;\n"),
            ("src/main.rs", "mod seam;\nfn main() {}\n"),
        ],
    );
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

/// The permitted region follows the root's declared module path even when `#[path]` puts that module in a
/// different file. This ordinary scenario has no mutation record: its negative proof is the single-engine
/// mutation that ignores the plain `#[path]` attribute, after which neither root has a conventional `seam.rs` fallback and the
/// permitted module is absent rather than silently clean.
#[test]
fn a_root_declaring_a_remapped_permitted_module_is_clean_by_module_path() {
    let probe = RootProbe::new(
        "confineremapped",
        "",
        &[
            ("src/lib.rs", "#[path = \"lib_seam.rs\"]\npub mod seam;\n"),
            ("src/lib_seam.rs", "\n"),
            (
                "src/main.rs",
                "#[path = \"bin_seam.rs\"]\nmod seam;\nmod cli;\nfn main() {}\n",
            ),
            ("src/bin_seam.rs", "use brick::B;\n"),
            ("src/cli.rs", "use brick::B;\n"),
        ],
    );
    let outcome = check(&confined_to_seam("confineremapped"), probe.manifest());
    let Outcome::Violations(report) = outcome else {
        panic!("the remapped seam is permitted but cli must react: {outcome:?}");
    };
    assert_eq!(report.violations.len(), 1, "{report:?}");
    let violation = &report.violations[0];
    assert_eq!(violation.target(), "brick");
    assert_eq!(violation.finding, "crate::cli");
    assert!(
        violation
            .file
            .as_deref()
            .is_some_and(|file| file.ends_with("src/cli.rs")),
        "the forbidden module is the cli source: {violation:?}"
    );
}

/// No root declaring the permitted module is still a constitution error, and the imports an absent root
/// was scanned for do not leak out beside it. Kept for the contract: it held before the empty-region
/// judgement existed, and pins that the findings it collects are dropped when no root is governed.
#[test]
fn no_root_declaring_the_permitted_module_is_still_a_constitution_error() {
    let outcome = confinement_report(
        "confinenone",
        "",
        &[
            ("src/lib.rs", "\n"),
            ("src/main.rs", "use brick::B;\nfn main() {}\n"),
        ],
    );
    let Outcome::ConstitutionError(message) = &outcome else {
        panic!("a permitted module no root declares is a typo, not an empty region: {outcome:?}");
    };
    assert!(
        message.contains("'crate::seam' is not found among the reachable modules"),
        "the refusal names the missing permitted module rather than some other failure: {message}"
    );
}

/// A file that a root without the permitted module reaches, and that cannot be read, is a refusal to
/// judge rather than a pass. Kept for the contract: resolving the root's module graph already reads it,
/// so this refused before that root's imports were judged too; it pins that the judgement cannot turn
/// the refusal into a clean report.
#[cfg(unix)]
#[test]
fn an_unreadable_file_in_a_root_without_the_permitted_module_is_refused() {
    let probe = RootProbe::new(
        "confineunread",
        "",
        &[
            ("src/lib.rs", "pub mod seam;\n"),
            ("src/seam.rs", "\n"),
            ("src/main.rs", "mod cli;\nfn main() {}\n"),
            ("src/cli.rs", "use brick::B;\n"),
        ],
    );
    let cli = probe.dir.join("src/cli.rs");
    let Some(_unreadable) = xingbiao::Unreadable::try_new(&cli) else {
        return;
    };
    let outcome = check(&confined_to_seam("confineunread"), probe.manifest());
    let Outcome::ConstitutionError(message) = &outcome else {
        panic!("an unreadable file in a root being judged is a refusal, not a pass: {outcome:?}");
    };
    assert!(
        message.contains("cli.rs"),
        "the refusal names the file it could not read rather than some other failure: {message}"
    );
}

/// A library finding's identity is unchanged by the binary root being judged, so no baseline recorded
/// before goes stale. Kept for the contract: without the empty-region judgement the leaking binary is not
/// read and the comparison holds trivially; with it, this is what says the library finding did not move.
#[test]
fn a_library_finding_keeps_its_identity_beside_a_judged_binary_root() {
    let library = [
        ("src/lib.rs", "pub mod seam;\npub mod leak;\n"),
        ("src/seam.rs", "\n"),
        ("src/leak.rs", "use brick::B as _L;\n"),
    ];
    let ids = |package: &str, main: &str| {
        let mut files = library.to_vec();
        files.push(("src/main.rs", main));
        let Outcome::Violations(report) = confinement_report(package, "", &files) else {
            panic!("the library leak reacts");
        };
        report
            .violations
            .iter()
            .filter(|v| {
                v.file
                    .as_deref()
                    .is_some_and(|f| f.ends_with("src/leak.rs"))
            })
            .map(|v| v.id().to_json().to_string().replace(package, "PKG"))
            .collect::<Vec<_>>()
    };
    let beside_a_clean_bin = ids("idclean", "fn main() {}\n");
    let beside_a_leaking_bin = ids("idleak", "use brick::B;\nfn main() {}\n");
    assert_eq!(beside_a_clean_bin.len(), 1, "{beside_a_clean_bin:?}");
    assert_eq!(beside_a_clean_bin, beside_a_leaking_bin);
}

/// A source file no target compiles belongs to no root's corpus. With `autobins = false`, Cargo reports
/// only the library, so `src/main.rs` is not compiled — yet it sits at a conventional root path, and read
/// as part of the library root its `mod shared;` would make `shared.rs` the governed target in place of
/// the library's inline `shared`, the one rustc compiles.
#[test]
fn an_uncompiled_conventional_root_file_does_not_stand_in_for_a_compiled_inline_module() {
    let probe = RootProbe::new(
        "strayinline",
        "autobins = false\n",
        &[
            (
                "src/lib.rs",
                "pub mod forbidden;\npub mod shared {\n    use crate::forbidden::X;\n}\n",
            ),
            ("src/forbidden.rs", "pub struct X;\n"),
            ("src/main.rs", "pub mod shared;\nfn main() {}\n"),
            ("src/shared.rs", "\n"),
        ],
    );
    let law = Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate("strayinline")
            .module("crate::shared")
            .must_not_import("crate::forbidden")
            .because("shared does not reach the forbidden module"),
    );
    match check(&law, probe.manifest()) {
        Outcome::ConstitutionError(message) => assert!(
            message.contains("inline"),
            "expected the inline-target refusal, got: {message}"
        ),
        other => panic!(
            "the compiled `shared` is inline, so it is refused; an uncompiled main.rs must not make \
             shared.rs its backing file: {other:?}"
        ),
    }
}

/// The other side of the same corpus: what an uncompiled file imports is not source the package
/// compiles, so it does not react. Every file whose path alone denotes `crate` is covered — an uncompiled
/// `main.rs`, and a top-level `mod.rs`, which no target compiles unless something declares it.
#[test]
fn a_top_level_file_no_target_compiles_is_not_judged() {
    for (package, stray) in [("straymain", "src/main.rs"), ("straymod", "src/mod.rs")] {
        let probe = RootProbe::new(
            package,
            "autobins = false\n",
            &[
                ("src/lib.rs", "pub mod forbidden;\n"),
                ("src/forbidden.rs", "pub struct X;\n"),
                (stray, "use crate::forbidden::X;\n"),
            ],
        );
        let outcome = check(
            &root_scope_must_not_import_forbidden(package),
            probe.manifest(),
        );
        assert_eq!(outcome.exit_code(), 0, "{stray}: {outcome:?}");
    }
}

fn root_scope_must_not_import_forbidden(package: &str) -> Constitution {
    Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_import("crate::forbidden")
            .because("the crate root does not reach the forbidden module"),
    )
}

/// A conventional root filename is excluded from another root only as a second source of `crate`. Reached
/// through a declaration, it is the declared module's source like any other file: a library declaring
/// `pub mod main;` governs `src/main.rs` as `crate::main`, when no target compiles that file on its own.
#[test]
fn a_conventional_root_filename_reached_through_a_declaration_is_that_modules_source() {
    let probe = RootProbe::new(
        "declaredmain",
        "autobins = false\n",
        &[
            ("src/lib.rs", "pub mod main;\npub mod forbidden;\n"),
            ("src/forbidden.rs", "pub struct X;\n"),
            ("src/main.rs", "use crate::forbidden::X;\n"),
        ],
    );
    let law = Constitution::new("root-scope").boundary(
        ModuleBoundary::in_crate("declaredmain")
            .module("crate::main")
            .must_not_import("crate::forbidden")
            .because("the main module does not reach the forbidden module"),
    );
    let Outcome::Violations(report) = check(&law, probe.manifest()) else {
        panic!("the declared `crate::main` imports the forbidden module, so it must react");
    };
    assert_eq!(report.violations.len(), 1, "{:?}", report.violations);
    let violation = &report.violations[0];
    assert!(
        violation
            .file
            .as_deref()
            .is_some_and(|f| f.ends_with("src/main.rs")),
        "{violation:?}"
    );
    let importer: Vec<_> = violation
        .fact()
        .fields()
        .filter(|(role, _)| *role == "importer")
        .map(|(_, value)| value)
        .collect();
    assert_eq!(
        importer,
        ["crate::main"],
        "the file is the declared module's source, so the import is attributed to that module: {:?}",
        violation.fact()
    );
}

/// Where an inline call to the confined prefix may appear: the permitted module's subtree, and nowhere else in any
/// compiled root. The perimeter is the whole package, so a sibling module and a root that declares no permitted
/// module are both outside the permitted region.
fn exec_confines_command(package: &str) -> Constitution {
    Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate::exec")
            .confine_inline_call("std::process::Command")
            .because("only exec spawns processes"),
    )
}

fn exec_confines_command_strict(package: &str) -> Constitution {
    Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate::exec")
            .confine_inline_call("std::process::Command")
            .strict_external()
            .because("only exec spawns processes"),
    )
}

/// The confined constructor call, assembled from two literals: this target spawns no process, and a census that
/// reads test sources for a spawning constructor must not read one out of fixture text.
macro_rules! spawn_call {
    () => {
        concat!("std::process::Command", "::new(\"true\")")
    };
}

const EXEC_SPAWNS: &str = concat!("pub fn run() { let _ = ", spawn_call!(), "; }\n");

/// A bare `Command` constructor call, assembled from two literals for the reason [`spawn_call`] gives.
macro_rules! command_new {
    () => {
        concat!("Command", "::new(\"x\")")
    };
}

fn fact_field<'a>(violation: &'a xuanji::Violation, key: &str) -> Option<&'a str> {
    violation
        .fact()
        .fields()
        .find_map(|(name, value)| (name == key).then_some(value))
}

fn confined_violations(outcome: &Outcome) -> &[xuanji::Violation] {
    match outcome {
        Outcome::Violations(report) => &report.violations,
        other => panic!("expected Violations, got {other:?}"),
    }
}

/// V1: a sibling of the permitted module calls the confined prefix inline, and the call is outside the permitted
/// region.
#[test]
fn an_inline_call_in_a_sibling_of_the_permitted_module_reacts() {
    let probe = RootProbe::new(
        "inlinesibling",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                concat!("pub fn leak() { let _ = ", spawn_call!(), "; }\n"),
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlinesibling"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    let violation = &violations[0];
    assert_eq!(
        violation.finding,
        "std::process::Command::new in crate::other"
    );
    assert_eq!(violation.target(), "std::process::Command");
    assert_eq!(violation.polarity, Some(xuanji::Polarity::AllowlistGap));
    assert_eq!(fact_field(violation, "unit"), Some("src/lib.rs"));
    assert!(
        violation
            .file
            .as_deref()
            .is_some_and(|f| f.ends_with("src/other.rs")),
        "{violation:?}"
    );
}

/// V2: a binary root whose graph has no permitted module calls the confined prefix inline. Its permitted region is
/// empty, so the call reacts under the binary root's own compilation unit.
#[test]
fn an_inline_call_in_a_root_without_the_permitted_module_reacts() {
    let probe = RootProbe::new(
        "inlinebin",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/main.rs",
                concat!("fn main() { let _ = ", spawn_call!(), "; }\n"),
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlinebin"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    let violation = &violations[0];
    assert_eq!(violation.finding, "std::process::Command::new in crate");
    assert_eq!(fact_field(violation, "unit"), Some("src/main.rs"));
    assert!(
        violation
            .file
            .as_deref()
            .is_some_and(|f| f.ends_with("src/main.rs")),
        "{violation:?}"
    );
}

/// V3: an aliased import outside the permitted module does not hide the call; the use-map resolves the alias.
#[test]
fn an_aliased_inline_call_outside_the_permitted_module_reacts() {
    let probe = RootProbe::new(
        "inlinealias",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                "use std::process::Command as Spawn;\npub fn leak() { let _ = Spawn::new(\"true\"); }\n",
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlinealias"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding,
        "std::process::Command::new in crate::other"
    );
}

/// V4: a glob import that can bring a prefix-resolving name into scope outside the permitted module reacts
/// fail-closed, as it does for `must_not_call_inline`.
#[test]
fn a_glob_bringing_the_confined_prefix_outside_the_permitted_module_reacts() {
    let probe = RootProbe::new(
        "inlineglob",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                "use std::process::*;\npub fn nothing() {}\n",
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlineglob"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "glob std::process in crate::other");
}

/// V5: at `warn()` the finding is reported as an advisory and the check still exits clean.
#[test]
fn a_warn_inline_call_confinement_reports_without_failing() {
    let probe = RootProbe::new(
        "inlinewarn",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                concat!("pub fn leak() { let _ = ", spawn_call!(), "; }\n"),
            ),
        ],
    );
    let law = Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate("inlinewarn")
            .module("crate::exec")
            .confine_inline_call("std::process::Command")
            .warn()
            .because("only exec spawns processes"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
    assert_eq!(
        confined_violations(&outcome).len(),
        1,
        "the advisory is still reported"
    );
}

/// V6: a baseline accepting one finding does not accept a second one added later in another module.
#[test]
fn a_baselined_inline_call_does_not_mask_a_new_one() {
    let probe = RootProbe::new(
        "inlinebase",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod exec;\npub mod other;\npub mod later;\n",
            ),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                concat!("pub fn leak() { let _ = ", spawn_call!(), "; }\n"),
            ),
            ("src/later.rs", "\n"),
        ],
    );
    let law = exec_confines_command("inlinebase");
    let Outcome::Violations(accepted) = check(&law, probe.manifest()) else {
        panic!("the sibling's call must react before it can be baselined");
    };
    let baseline = xuanji::Baseline::of(&accepted);
    std::fs::write(
        probe.dir.join("src/later.rs"),
        concat!("pub fn again() { let _ = ", spawn_call!(), "; }\n"),
    )
    .expect("write the new call");
    let Outcome::Violations(mut report) = check(&law, probe.manifest()) else {
        panic!("both siblings call the confined prefix");
    };
    xuanji::apply_baseline(&mut report, &baseline);
    let active: Vec<&str> = report
        .violations
        .iter()
        .filter(|v| !v.baselined)
        .map(|v| v.finding.as_str())
        .collect();
    assert_eq!(
        active,
        ["std::process::Command::new in crate::later"],
        "{:?}",
        report.violations
    );
}

fn assert_clean(outcome: &Outcome) {
    assert!(matches!(outcome, Outcome::Clean(_)), "{outcome:?}");
}

/// C1: the permitted module and its inline test module call the confined prefix, which is where it is permitted.
#[test]
fn inline_calls_within_the_permitted_module_and_its_inline_tests_are_clean() {
    let probe = RootProbe::new(
        "inlinepermitted",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\n"),
            (
                "src/exec.rs",
                concat!(
                    "pub fn run() { let _ = ",
                    spawn_call!(),
                    "; }\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn spawns() { let _ = ",
                    spawn_call!(),
                    "; }\n}\n"
                ),
            ),
        ],
    );
    assert_clean(&check(
        &exec_confines_command("inlinepermitted"),
        probe.manifest(),
    ));
}

/// C2: under the call-versus-mention default, naming the type outside the permitted module is not a call.
#[test]
fn a_type_only_mention_outside_the_permitted_module_is_clean() {
    let probe = RootProbe::new(
        "inlinemention",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                "pub fn take(_command: &std::process::Command) {}\n",
            ),
        ],
    );
    assert_clean(&check(
        &exec_confines_command("inlinemention"),
        probe.manifest(),
    ));
}

/// C3: narrowed to `new`, a call under the prefix with another terminal segment outside the permitted module is
/// clean.
#[test]
fn a_narrowed_inline_call_confinement_ignores_other_verbs() {
    let probe = RootProbe::new(
        "inlinenarrow",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod other;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
            (
                "src/other.rs",
                "pub fn finish(command: &mut std::process::Command) {\n    let _ = std::process::Command::output(command);\n}\n",
            ),
        ],
    );
    let law = Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate("inlinenarrow")
            .module("crate::exec")
            .confine_inline_call("std::process::Command")
            .ending_with(["new"])
            .because("only exec constructs a process"),
    );
    assert_clean(&check(&law, probe.manifest()));
}

/// C4: the permitted module's own private `use`, and `use super::*` inside a sibling's test module with no alias
/// of the prefix anywhere in the crate, are clean — the shape of an adopter confining process spawning to one
/// module.
#[test]
fn a_private_use_in_the_permitted_module_and_sibling_test_globs_are_clean() {
    let probe = RootProbe::new(
        "inlineshape",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            (
                "src/exec.rs",
                concat!(
                    "use std::process::Command;\npub fn run() { let _ = Command",
                    "::new(\"true\"); }\n"
                ),
            ),
            (
                "src/agent.rs",
                "pub fn act() {}\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn acts() { act(); }\n}\n",
            ),
        ],
    );
    assert_clean(&check(
        &exec_confines_command("inlineshape"),
        probe.manifest(),
    ));
}

fn constitution_error(outcome: &Outcome) -> &str {
    match outcome {
        Outcome::ConstitutionError(message) => message,
        other => panic!("expected a constitution error, got {other:?}"),
    }
}

/// E1: permitting the prefix within `crate` permits it everywhere, so the rule could never react.
#[test]
fn an_inline_call_confinement_to_the_crate_root_is_refused() {
    let probe = RootProbe::new(
        "inlineroot",
        "",
        &[(
            "src/lib.rs",
            concat!("pub fn run() { let _ = ", spawn_call!(), "; }\n"),
        )],
    );
    let law = Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate("inlineroot")
            .module("crate")
            .confine_inline_call("std::process::Command")
            .because("only the root spawns processes"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(
        constitution_error(&outcome).contains("confine_inline_call"),
        "{outcome:?}"
    );
}

/// E2: a permitted module no compiled root declares is a constitution error, not an empty region everywhere.
#[test]
fn an_inline_call_confinement_to_a_module_no_root_declares_is_refused() {
    let probe = RootProbe::new(
        "inlineabsent",
        "",
        &[
            ("src/lib.rs", "pub mod other;\n"),
            (
                "src/other.rs",
                concat!("pub fn leak() { let _ = ", spawn_call!(), "; }\n"),
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlineabsent"), probe.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
}

/// E3: an empty confined prefix is a misdeclaration, as it is for `must_not_call_inline`.
#[test]
fn an_inline_call_confinement_with_an_empty_prefix_is_refused() {
    let probe = RootProbe::new(
        "inlineempty",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\n"),
            ("src/exec.rs", EXEC_SPAWNS),
        ],
    );
    let law = Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate("inlineempty")
            .module("crate::exec")
            .confine_inline_call(" ")
            .because("a prefix is required"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(
        constitution_error(&outcome).contains("`confine_inline_call`"),
        "the refusal names the rule that was declared: {outcome:?}"
    );
}

/// The glob over-reaction, shown rather than described: a private alias beneath the glob's resolved module is
/// treated as a name the glob could bring into scope even when it is not actually imported.
#[test]
fn a_sibling_test_glob_reacts_to_an_alias_in_its_resolved_module() {
    let probe = RootProbe::new(
        "inlinealiasglob",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/agent.rs",
                "mod hidden {\n    pub type Spawner = std::process::Command;\n}\npub fn act() {}\nmod tests {\n    use super::*;\n    fn acts() { act(); }\n}\n",
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlinealiasglob"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "glob super in crate::agent");
}

/// Precision guard: a private alias directly in the glob's parent is in `super::*`'s scope and must react;
/// this is not the over-reaction bound's pin.
#[test]
fn a_sibling_test_glob_reacts_to_an_alias_in_its_parent_as_a_precision_guard() {
    let probe = RootProbe::new(
        "inlinealiasprecision",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/agent.rs",
                "type Spawner = std::process::Command;\npub fn act() {}\nmod tests {\n    use super::*;\n}\n",
            ),
        ],
    );
    let outcome = check(
        &exec_confines_command("inlinealiasprecision"),
        probe.manifest(),
    );
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "glob super in crate::agent");
}

fn inline_super_alias_probe(name: &str) -> RootProbe {
    RootProbe::new(
        name,
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/agent.rs",
                "type Cmd = std::process::Command;\nmod inner {\n    fn f() { let _ = super::Cmd::new(\"true\"); }\n}\n",
            ),
        ],
    )
}

#[test]
fn an_inline_super_path_resolves_from_its_true_module_in_default_mode() {
    let probe = inline_super_alias_probe("inlinealiaspathdefault");
    let outcome = check(
        &exec_confines_command("inlinealiaspathdefault"),
        probe.manifest(),
    );
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    assert_eq!(confined_violations(&outcome).len(), 1, "{outcome:?}");
}

#[test]
fn an_inline_super_path_resolves_from_its_true_module_in_strict_external_mode() {
    let probe = inline_super_alias_probe("inlinealiaspathstrict");
    let outcome = check(
        &exec_confines_command_strict("inlinealiaspathstrict"),
        probe.manifest(),
    );
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    assert_eq!(confined_violations(&outcome).len(), 1, "{outcome:?}");
}

/// The inline test module's `super::*` is resolved from its true inline module, so a private alias
/// in a sibling file cannot make this glob reach the confined prefix.
#[test]
fn a_sibling_test_glob_ignores_an_alias_in_another_file() {
    let probe = RootProbe::new(
        "inlinealiasother",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod exec;\npub mod other;\npub mod agent;\n",
            ),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/other.rs",
                "type Spawner = std::process::Command;\npub fn unrelated() {}\n",
            ),
            (
                "src/agent.rs",
                "pub fn act() {}\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn acts() { act(); }\n}\n",
            ),
        ],
    );
    assert_clean(&check(
        &exec_confines_command("inlinealiasother"),
        probe.manifest(),
    ));
}

/// Nested inline modules use the corresponding ancestor for `super::super::*`, not the file module.
#[test]
fn a_nested_inline_glob_resolves_to_its_true_ancestor() {
    let probe = RootProbe::new(
        "inlinealiasnested",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/agent.rs",
                "type Spawner = std::process::Command;\npub fn act() {}\nmod outer {\n    mod tests {\n        use super::super::*;\n        fn acts() { act(); }\n    }\n}\n",
            ),
        ],
    );
    let outcome = check(
        &exec_confines_command("inlinealiasnested"),
        probe.manifest(),
    );
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "glob super::super in crate::agent");
}

/// File-module `self`/`super` globs retain their existing file-module resolution contract.
#[test]
fn file_module_self_and_super_globs_keep_their_resolution() {
    let probe = RootProbe::new(
        "inlinealiasfile",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\npub mod agent;\n"),
            ("src/exec.rs", "pub fn run() {}\n"),
            (
                "src/agent.rs",
                "type Spawner = std::process::Command;\nuse self::*;\nuse super::*;\npub fn act() {}\n",
            ),
        ],
    );
    let outcome = check(&exec_confines_command("inlinealiasfile"), probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 2, "{violations:?}");
    assert!(
        violations
            .iter()
            .any(|v| v.finding == "glob self in crate::agent")
    );
    assert!(
        violations
            .iter()
            .any(|v| v.finding == "glob super in crate::agent")
    );
}

/// E4: at `ScanDepth::Shallow` the permitted region is the anchored module alone, so the permitted file's inline
/// child is outside it; the region is compared at file-module grain, which cannot tell the two apart, so the
/// declaration is refused rather than read as permitting the child's call.
#[test]
fn an_inline_call_confinement_at_shallow_depth_is_refused() {
    let probe = RootProbe::new(
        "inlineshallow",
        "",
        &[
            ("src/lib.rs", "pub mod exec;\n"),
            (
                "src/exec.rs",
                concat!(
                    "pub fn run() {}\nmod tests {\n    fn spawns() { let _ = ",
                    spawn_call!(),
                    "; }\n}\n"
                ),
            ),
        ],
    );
    let law = Constitution::new("inline-call-confinement").boundary(
        ModuleBoundary::in_crate("inlineshallow")
            .module("crate::exec")
            .confine_inline_call("std::process::Command")
            .depth(xuanji::ScanDepth::Shallow)
            .because("only exec itself spawns processes"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(
        constitution_error(&outcome).contains("Shallow"),
        "{outcome:?}"
    );
}

#[test]
fn an_inline_module_use_resolves_from_its_enclosing_module() {
    let probe = RootProbe::new(
        "inlineusemod",
        "",
        &[
            ("src/lib.rs", "pub mod clock;\npub mod core;\n"),
            ("src/clock.rs", "pub fn now() -> u64 { 0 }\n"),
            (
                "src/core.rs",
                concat!(
                    "mod inner {\n",
                    "    use super::super::clock::now;\n",
                    "    pub fn f() -> u64 { now() }\n",
                    "}\n",
                ),
            ),
        ],
    );
    let law = Constitution::new("inline-use-mod").boundary(
        ModuleBoundary::in_crate("inlineusemod")
            .module("crate::core")
            .must_not_call_inline("crate::clock")
            .because("clock is forbidden"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].finding, "crate::clock::now in crate::core");
}

#[test]
fn an_inline_module_use_does_not_leak_to_sibling_modules() {
    let probe = RootProbe::new(
        "inlineuseleak",
        "",
        &[
            ("src/lib.rs", "pub mod clock;\npub mod core;\n"),
            ("src/clock.rs", "pub fn now() -> u64 { 0 }\n"),
            (
                "src/core.rs",
                concat!(
                    "mod inner {\n",
                    "    use super::super::clock::now;\n",
                    "    pub fn f() -> u64 { 0 }\n",
                    "}\n",
                    "mod sibling {\n",
                    "    fn now() -> u64 { 1 }\n",
                    "    pub fn h() -> u64 { now() }\n",
                    "}\n",
                ),
            ),
        ],
    );
    let law = Constitution::new("inline-use-leak").boundary(
        ModuleBoundary::in_crate("inlineuseleak")
            .module("crate::core")
            .must_not_call_inline("crate::clock")
            .because("clock is forbidden"),
    );
    assert_clean(&check(&law, probe.manifest()));
}

#[test]
fn an_inline_module_type_alias_resolves_under_its_inline_path() {
    let probe = RootProbe::new(
        "inlinealiasmod",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                concat!(
                    "mod hidden {\n",
                    "    pub type Spawner = std::process::Command;\n",
                    "}\n",
                    "pub fn g() { let _ = hidden::Spawner::new(\"true\"); }\n",
                ),
            ),
        ],
    );
    let law = Constitution::new("inline-alias-mod").boundary(
        ModuleBoundary::in_crate("inlinealiasmod")
            .module("crate::core")
            .must_not_call_inline("std::process::Command")
            .because("spawning is forbidden"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
    let violations = confined_violations(&outcome);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(
        violations[0].finding,
        "std::process::Command::new in crate::core"
    );
}

/// A leading `::` names the dependency, never the same-named crate-root module: `::md5x::compute()` beside a local
/// `mod md5x` is outside the default, as every un-`use`d dependency call is, and under strict-external it is
/// reported as the dependency's — only that mode can show which of the two the `::` names.
#[test]
fn a_leading_colon_names_the_dependency_not_the_same_named_module() {
    let probe = RootProbe::new(
        "inlinedisambiguate",
        "[dependencies]\nmd5x_pkg = { path = \"md5x_dep\", package = \"md5x_pkg\" }\n",
        &[
            ("src/lib.rs", "pub mod md5x;\npub mod core;\n"),
            ("src/md5x.rs", "pub fn compute() -> u32 { 1 }\n"),
            (
                "src/core.rs",
                concat!(
                    "pub fn run() {\n",
                    "    let _ = ::md5x::compute();\n",
                    "}\n",
                ),
            ),
            (
                "md5x_dep/Cargo.toml",
                "[package]\nname = \"md5x_pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\nname = \"md5x\"\n",
            ),
            ("md5x_dep/src/lib.rs", "pub fn compute() -> u32 { 0 }\n"),
        ],
    );
    assert_inline_answers(
        &probe,
        "inlinedisambiguate",
        "crate::core",
        "::md5x::compute",
        &[],
        &["md5x::compute in crate::core"],
    );
    let root = probe.dir().to_path_buf();
    drop(probe);
    assert!(
        !root.exists(),
        "the probe removes its root, dependency included: {}",
        root.display()
    );
}

/// N: a prefix head that can never name a crate or module — `_`, a raw `crate`/`self`/`super`/`Self`/`_`, a bare
/// `self`/`super`/`Self`, or `crate` after a leading `::` — is refused by either builder, quoting the written
/// prefix, and suggesting the unraw spelling only where that is a valid prefix.
#[test]
fn a_prefix_head_naming_no_crate_or_module_is_refused() {
    let probe = RootProbe::new(
        "inlinenohead",
        "",
        &[
            ("src/lib.rs", "pub mod clock;\npub mod core;\n"),
            ("src/clock.rs", "pub fn now() -> u64 { 0 }\n"),
            ("src/core.rs", "pub fn g() -> u64 { crate::clock::now() }\n"),
        ],
    );
    let refused: [(&str, Option<&str>); 13] = [
        ("_", None),
        ("_::clock", None),
        ("r#_::clock", None),
        ("Self::now", None),
        ("Self::clock", None),
        ("r#Self::clock", None),
        ("self::clock", None),
        ("r#self::clock", None),
        ("super::clock", None),
        ("r#super::clock", None),
        ("::crate::clock", None),
        ("r#crate::clock", Some("crate::clock")),
        ("r#crate", Some("crate")),
    ];
    for (prefix, suggestion) in refused {
        for (module, draft) in [
            (
                "must_not_call_inline",
                ModuleBoundary::in_crate("inlinenohead")
                    .module("crate::core")
                    .must_not_call_inline(prefix),
            ),
            (
                "confine_inline_call",
                ModuleBoundary::in_crate("inlinenohead")
                    .module("crate::clock")
                    .confine_inline_call(prefix),
            ),
        ] {
            let law =
                Constitution::new("inline-no-head").boundary(draft.because("a head names nothing"));
            let outcome = check(&law, probe.manifest());
            assert_eq!(outcome.exit_code(), 2, "{module}({prefix}): {outcome:?}");
            let message = constitution_error(&outcome);
            assert!(
                message.contains(&format!("names '{prefix}'")),
                "{module}({prefix}) quotes the written prefix: {message}"
            );
            let repair = match suggestion {
                Some(spelling) => format!("write `{spelling}`"),
                None => "write the path from `crate`".to_string(),
            };
            assert!(
                message.contains(&repair),
                "{module}({prefix}) repairs with {repair:?}: {message}"
            );
        }
    }
    let found = ["crate::clock::now in crate::core"];
    assert_inline_answers(
        &probe,
        "inlinenohead",
        "crate::core",
        "crate::clock",
        &found,
        &found,
    );
}

/// One inline confinement's findings over a probe, sorted: empty for a clean run, and anything but a clean run or
/// violations fails the test.
fn inline_findings(
    probe: &RootProbe,
    package: &str,
    module: &str,
    prefix: &str,
    strict_external: bool,
) -> Vec<String> {
    inline_findings_and_ids(
        probe,
        package,
        module,
        prefix,
        strict_external,
        &mut Vec::new(),
    )
}

/// [`inline_findings`], also collecting each violation's baseline identity into `ids`.
fn inline_findings_and_ids(
    probe: &RootProbe,
    package: &str,
    module: &str,
    prefix: &str,
    strict_external: bool,
    ids: &mut Vec<xuanji::ViolationId>,
) -> Vec<String> {
    let draft = ModuleBoundary::in_crate(package)
        .module(module)
        .must_not_call_inline(prefix);
    let draft = if strict_external {
        draft.strict_external()
    } else {
        draft
    };
    let law = Constitution::new("inline-answers").boundary(draft.because("inline path resolution"));
    let outcome = check(&law, probe.manifest());
    if let Outcome::Violations(report) = &outcome {
        ids.extend(report.violations.iter().map(xuanji::Violation::id));
    }
    match &outcome {
        Outcome::Clean(_) => Vec::new(),
        Outcome::Violations(report) => {
            let mut found: Vec<String> = report
                .violations
                .iter()
                .map(|v| v.finding.clone())
                .collect();
            found.sort();
            found
        }
        other => {
            panic!("{prefix} over {module}: expected a clean run or violations, got {other:?}")
        }
    }
}

/// The default and strict-external answers of one prefix over a probe, each as its sorted findings.
fn assert_inline_answers(
    probe: &RootProbe,
    package: &str,
    module: &str,
    prefix: &str,
    default: &[&str],
    strict: &[&str],
) {
    for (strict_external, expected) in [(false, default), (true, strict)] {
        assert_eq!(
            inline_findings(probe, package, module, prefix, strict_external),
            expected,
            "{prefix} over {module} in {package}, strict_external = {strict_external}"
        );
    }
}

/// A path dependency written beneath the probe, keyed `key` in the manifest and packaged as `dep_<key>`, whose
/// library defines `f` and `compute`.
fn renamed_dependency(key: &str) -> (String, Vec<(String, String)>) {
    let package = format!("dep_{key}");
    let manifest =
        format!("[dependencies]\n{key} = {{ package = \"{package}\", path = \"{package}\" }}\n");
    let files = vec![
        (
            format!("{package}/Cargo.toml"),
            format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2015\"\n"),
        ),
        (
            format!("{package}/src/lib.rs"),
            "pub fn f() {}\npub fn compute() -> u32 { 0 }\n".to_string(),
        ),
    ];
    (manifest, files)
}

fn borrowed(files: &[(String, String)]) -> Vec<(&str, &str)> {
    files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect()
}

/// R2: a sysroot call written in full reacts under the relative prefix and under the `::`-rooted one.
#[test]
fn inline_relative_and_root_sysroot_prefixes_react_on_a_qualified_call() {
    let probe = RootProbe::new(
        "frozenr2",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "pub fn g() -> std::time::SystemTime { std::time::SystemTime::now() }\n",
            ),
        ],
    );
    let found = ["std::time::SystemTime::now in crate::core"];
    for prefix in ["std::time", "::std::time"] {
        assert_inline_answers(&probe, "frozenr2", "crate::core", prefix, &found, &found);
    }
}

/// R3: an un-`use`d dependency call is outside the default and inside strict-external, whichever root spelling
/// writes it and whichever spelling the prefix takes — `md5x` and `::md5x` name one crate.
#[test]
fn inline_external_dependency_root_spelling_is_mode_invariant() {
    for (package, call) in [
        ("rootbare", "md5x::compute()"),
        ("rootcolons", "::md5x::compute()"),
    ] {
        let (manifest, mut files) = renamed_dependency("md5x");
        files.push(("src/lib.rs".to_string(), "pub mod core;\n".to_string()));
        files.push((
            "src/core.rs".to_string(),
            format!("pub fn g() -> u32 {{ {call} }}\n"),
        ));
        let probe = RootProbe::new(package, &manifest, &borrowed(&files));
        for prefix in ["md5x", "::md5x"] {
            assert_inline_answers(
                &probe,
                package,
                "crate::core",
                prefix,
                &[],
                &["md5x::compute in crate::core"],
            );
        }
    }
}

/// R4: a crate-root `mod md5x` shadows the same-named dependency for a bare head, so an external-only prefix
/// matches nothing in either mode.
#[test]
fn inline_bare_local_module_shadows_same_named_dependency() {
    let (manifest, mut files) = renamed_dependency("md5x");
    files.push((
        "src/lib.rs".to_string(),
        "pub mod md5x {\n    pub struct Local;\n    pub fn compute() -> Local { Local }\n}\npub fn g() -> md5x::Local { md5x::compute() }\n"
            .to_string(),
    ));
    let probe = RootProbe::new("frozenr4", &manifest, &borrowed(&files));
    assert_inline_answers(&probe, "frozenr4", "crate", "::md5x", &[], &[]);
}

/// K: a boundary prefix is a name, not source, so a head that is a keyword in some edition is accepted bare or
/// raw. A dependency renamed to the head, imported with the spelling each edition requires, reacts under both
/// prefixes in every edition, and the two spellings carry one baseline identity.
#[test]
fn a_keyword_prefix_head_is_accepted_bare_or_raw() {
    for edition in ["2018", "2021", "2024"] {
        for head in ["gen", "async", "dyn", "try", "union"] {
            let raw_in_source = match head {
                "async" | "dyn" | "try" => true,
                "gen" => edition == "2024",
                _ => false,
            };
            let written = if raw_in_source {
                format!("r#{head}")
            } else {
                head.to_string()
            };
            let package = format!("frozenk{edition}{head}");
            let (manifest, mut files) = renamed_dependency(head);
            files.push(("src/lib.rs".to_string(), "pub mod core;\n".to_string()));
            files.push((
                "src/core.rs".to_string(),
                format!("use {written}::f;\npub fn g() {{ f(); }}\n"),
            ));
            let probe = RootProbe::with_edition(&package, edition, &manifest, &borrowed(&files));
            let found = format!("{head}::f in crate::core");
            for strict_external in [false, true] {
                let mut identities = Vec::new();
                for prefix in [head.to_string(), format!("r#{head}")] {
                    let mut ids = Vec::new();
                    assert_eq!(
                        inline_findings_and_ids(
                            &probe,
                            &package,
                            "crate::core",
                            &prefix,
                            strict_external,
                            &mut ids,
                        ),
                        [found.as_str()],
                        "{prefix} in {edition}, strict_external = {strict_external}"
                    );
                    identities.push(ids);
                }
                assert_eq!(
                    identities[0], identities[1],
                    "`{head}` and `r#{head}` are one identity in {edition}"
                );
            }
        }
    }
}

/// A, A2: a `use` inside a function body, or inside a block nested in one, resolves the calls in that block.
#[test]
fn a_block_local_use_resolves_inside_its_block() {
    for (package, core) in [
        (
            "frozena",
            concat!(
                "pub fn g() { use std::process::Command; let _ = ",
                command_new!(),
                "; }\n"
            ),
        ),
        (
            "frozena2",
            concat!(
                "pub fn g() { { use std::process::Command; let _ = ",
                command_new!(),
                "; } }\n"
            ),
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[("src/lib.rs", "pub mod core;\n"), ("src/core.rs", core)],
        );
        let found = ["std::process::Command::new in crate::core"];
        assert_inline_answers(
            &probe,
            package,
            "crate::core",
            "std::process",
            &found,
            &found,
        );
    }
}

/// Two modules each defining `X`, with an associated `f` returning a different type in each, as the B and E rows
/// use them.
const SAME_NAMED_X: [(&str, &str); 2] = [
    (
        "src/a.rs",
        "pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }\n",
    ),
    (
        "src/b.rs",
        "pub struct X;\nimpl X { pub fn f() -> u16 { 0 } }\n",
    ),
];

fn with_same_named_x(core: &str) -> Vec<(&str, &str)> {
    let mut files = vec![
        ("src/lib.rs", "pub mod a;\npub mod b;\npub mod core;\n"),
        ("src/core.rs", core),
    ];
    files.extend(SAME_NAMED_X);
    files
}

/// B: a function-local `use crate::b::X` shadows the module's `use crate::a::X` inside that function.
#[test]
fn a_fn_local_use_shadows_a_module_use_of_the_same_name() {
    let probe = RootProbe::new(
        "frozenb",
        "",
        &with_same_named_x(
            "#[allow(unused_imports)]\nuse crate::a::X;\npub fn g() -> u16 { use crate::b::X; X::f() }\n",
        ),
    );
    assert_inline_answers(&probe, "frozenb", "crate::core", "crate::a", &[], &[]);
    let found = ["crate::b::X::f in crate::core"];
    assert_inline_answers(&probe, "frozenb", "crate::core", "crate::b", &found, &found);
}

/// G: a block's `use` covers the whole block, including a call written before it.
#[test]
fn a_block_local_use_covers_text_before_it() {
    let probe = RootProbe::new(
        "frozeng",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                concat!(
                    "pub fn g() { let _ = ",
                    command_new!(),
                    "; use std::process::Command; }\n"
                ),
            ),
        ],
    );
    let found = ["std::process::Command::new in crate::core"];
    assert_inline_answers(
        &probe,
        "frozeng",
        "crate::core",
        "std::process",
        &found,
        &found,
    );
}

/// C1–C4: a path beginning with `<` names its associated item through a type, which needs inference the scanner
/// does not perform, so none of these calls is observed — the receiver-method bound's qualified form.
#[test]
fn inline_qualified_path_is_the_type_directed_bound() {
    const CLOCK: &str = "pub trait Clock { fn now() -> u64; }\npub struct S;\nimpl Clock for S { fn now() -> u64 { 0 } }\n";
    let rows: [(&str, &str, &[&str]); 4] = [
        (
            "frozenc1",
            "pub fn g() -> std::time::SystemTime { <std::time::SystemTime>::now() }\n",
            &["std::time"],
        ),
        (
            "frozenc2",
            "pub fn g() -> u64 { <crate::clock::S as crate::clock::Clock>::now() }\n",
            &["crate::clock::Clock", "crate::clock"],
        ),
        (
            "frozenc3",
            "use crate::clock::{Clock, S};\npub fn g() -> u64 { <S as Clock>::now() }\n",
            &["crate::clock::Clock", "crate::clock"],
        ),
        (
            "frozenc4",
            "use std::time::SystemTime;\npub fn g(t: &SystemTime) -> SystemTime { <SystemTime as Clone>::clone(t) }\n",
            &["std::time"],
        ),
    ];
    for (package, core, prefixes) in rows {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod clock;\npub mod core;\n"),
                ("src/clock.rs", CLOCK),
                ("src/core.rs", core),
            ],
        );
        for prefix in prefixes {
            assert_inline_answers(&probe, package, "crate::core", prefix, &[], &[]);
        }
    }
}

/// E1, E2, E4, E5: two globs that can each bring `X` into scope — used or not, cfg-exclusive, or one item reached
/// twice — react through the glob hazard, naming each glob that reaches the prefix. Where `X::f()` is called, the
/// globs are also followed to the modules they name, so the call reports under each candidate the prefix reaches.
#[test]
fn a_glob_that_can_bring_the_prefix_reacts_however_the_name_is_used() {
    let two_globs = "use crate::a::*;\nuse crate::b::*;\npub fn g() { let _ = X::f(); }\n";
    let unused = "#[allow(unused_imports)]\nuse crate::a::*;\n#[allow(unused_imports)]\nuse crate::b::*;\npub fn g() {}\n";
    let cfg_globs = "#[cfg(unix)]\nuse crate::a::*;\n#[cfg(not(unix))]\nuse crate::b::*;\npub fn g() { let _ = X::f(); }\n";
    let rows: [(&str, &str, &str, &[&str]); 4] = [
        (
            "frozene1",
            two_globs,
            "crate::a",
            &[
                "crate::a::X::f in crate::core",
                "glob crate::a in crate::core",
            ],
        ),
        (
            "frozene2",
            unused,
            "crate::a",
            &["glob crate::a in crate::core"],
        ),
        (
            "frozene4",
            cfg_globs,
            "crate::a",
            &[
                "crate::a::X::f in crate::core",
                "glob crate::a in crate::core",
            ],
        ),
        (
            "frozene4",
            cfg_globs,
            "crate::b",
            &[
                "crate::b::X::f in crate::core",
                "glob crate::b in crate::core",
            ],
        ),
    ];
    for (package, core, prefix, found) in rows {
        let probe = RootProbe::new(package, "", &with_same_named_x(core));
        assert_inline_answers(&probe, package, "crate::core", prefix, found, found);
    }
    let probe = RootProbe::new(
        "frozene5",
        "",
        &[
            ("src/lib.rs", "pub mod a;\npub mod b;\npub mod core;\n"),
            (
                "src/a.rs",
                "pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }\n",
            ),
            ("src/b.rs", "pub use crate::a::X;\n"),
            (
                "src/core.rs",
                "use crate::a::*;\nuse crate::b::*;\npub fn g() -> u8 { X::f() }\n",
            ),
        ],
    );
    let found = [
        "crate::a::X::f in crate::core",
        "glob crate::a in crate::core",
        "glob crate::b in crate::core",
    ];
    assert_inline_answers(
        &probe,
        "frozene5",
        "crate::core",
        "crate::a",
        &found,
        &found,
    );
}

/// E3: a named import beside a glob wins the name, and the glob still reacts as a hazard under the prefix it
/// reaches — the stated over-reaction.
#[test]
fn an_explicit_import_beside_a_glob_reacts_through_each() {
    let probe = RootProbe::new(
        "frozene3",
        "",
        &with_same_named_x(
            "#[allow(unused_imports)]\nuse crate::a::*;\nuse crate::b::X;\npub fn g() -> u16 { X::f() }\n",
        ),
    );
    let glob = ["glob crate::a in crate::core"];
    assert_inline_answers(&probe, "frozene3", "crate::core", "crate::a", &glob, &glob);
    let call = ["crate::b::X::f in crate::core"];
    assert_inline_answers(&probe, "frozene3", "crate::core", "crate::b", &call, &call);
}

/// H1, H3, H4: an associated `const`, an enum variant and an associated type named like an import are reached
/// only through a path, so none shadows the import. H4 is the type-namespace member, the one a lookup keyed by
/// namespace alone would not hold.
#[test]
fn inline_associated_item_or_variant_does_not_shadow_an_import() {
    for (package, core) in [
        (
            "frozenh1",
            concat!(
                "use std::process::Command; pub struct S; impl S { const Command: u8 = 0; pub fn f() { let _ = ",
                command_new!(),
                "; } }\n"
            ),
        ),
        (
            "frozenh3",
            concat!(
                "use std::process::Command; pub enum E { Command } pub fn f() { let _ = ",
                command_new!(),
                "; }\n"
            ),
        ),
        (
            "frozenh4",
            concat!(
                "use std::process::Command; pub struct S; pub trait T { type Command; fn f(); } impl T for S { type Command = u8; fn f() { let _ = ",
                command_new!(),
                "; } }\n"
            ),
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[("src/lib.rs", "pub mod core;\n"), ("src/core.rs", core)],
        );
        let found = ["std::process::Command::new in crate::core"];
        assert_inline_answers(
            &probe,
            package,
            "crate::core",
            "std::process",
            &found,
            &found,
        );
    }
}

/// H2: Rust resolves `Command` to the generic parameter; the scanner does not read parameter lists and reads the
/// module's import — the declared over-reaction.
#[test]
fn inline_generic_parameter_named_like_an_import_is_read_as_the_import() {
    let probe = RootProbe::new(
        "frozenh2",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "#[allow(unused_imports)] use std::process::Command; pub fn f<Command: Default>() -> Command { Command::default() }\n",
            ),
        ],
    );
    let found = ["std::process::Command::default in crate::core"];
    assert_inline_answers(
        &probe,
        "frozenh2",
        "crate::core",
        "std::process",
        &found,
        &found,
    );
}

/// B3, B4: `crate::core` imports `crate::a::X` at module level, `h` calls `X::fa()`, and `g` declares
/// `use crate::b::X;` and calls `X::fb()`. In any textual order the block's `use` binds only inside `g`, so each
/// prefix reports its own call and nothing else. The third order puts `h` after `g`, the one call a block scope
/// that closed late would read through `g`'s binding.
#[test]
fn inline_block_local_use_binds_only_inside_its_block() {
    for (package, core) in [
        (
            "blockuseafter",
            "use crate::a::X;\npub fn h() -> u8 { X::fa() }\npub fn g() -> u16 { use crate::b::X; X::fb() }\n",
        ),
        (
            "blockusebefore",
            "pub fn g() -> u16 { use crate::b::X; X::fb() }\npub fn h() -> u8 { X::fa() }\nuse crate::a::X;\n",
        ),
        (
            "blockusebetween",
            "use crate::a::X;\npub fn g() -> u16 { use crate::b::X; X::fb() }\npub fn h() -> u8 { X::fa() }\n",
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod a;\npub mod b;\npub mod core;\n"),
                (
                    "src/a.rs",
                    "pub struct X;\nimpl X { pub fn fa() -> u8 { 0 } }\n",
                ),
                (
                    "src/b.rs",
                    "pub struct X;\nimpl X { pub fn fb() -> u16 { 0 } }\n",
                ),
                ("src/core.rs", core),
            ],
        );
        let a = ["crate::a::X::fa in crate::core"];
        assert_inline_answers(&probe, package, "crate::core", "crate::a", &a, &a);
        let b = ["crate::b::X::fb in crate::core"];
        assert_inline_answers(&probe, package, "crate::core", "crate::b", &b, &b);
    }
}

/// F: a function body's own `struct Command` shadows the module's `use std::process::Command` inside that body.
#[test]
fn inline_block_local_item_shadows_a_module_import() {
    let probe = RootProbe::new(
        "blockitemshadow",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                concat!(
                    "#[allow(unused_imports)]\nuse std::process::Command;\npub fn g() { struct Command; impl Command { fn new(_: &str) -> Self { Command } } let _ = ",
                    command_new!(),
                    "; }\n"
                ),
            ),
        ],
    );
    assert_inline_answers(
        &probe,
        "blockitemshadow",
        "crate::core",
        "std::process",
        &[],
        &[],
    );
}

/// R1: `crate::clock` privately imports `std::time::SystemTime`, and its inline test module, through `use super::*`
/// — directly or two levels down through `use super::super::*` — calls `SystemTime::now()`. A glob of an ancestor
/// carries that ancestor's private imports, so the call resolves to `std::time`.
#[test]
fn inline_private_use_inherited_through_super_glob_resolves() {
    for (package, clock) in [
        (
            "superglob",
            "use std::time::SystemTime;\npub fn epoch() -> SystemTime { SystemTime::UNIX_EPOCH }\n#[cfg(test)]\nmod tests {\n    use super::*;\n    #[test]\n    fn stamps() { let _ = SystemTime::now(); }\n}\n",
        ),
        (
            "supersuperglob",
            "use std::time::SystemTime;\npub fn epoch() -> SystemTime { SystemTime::UNIX_EPOCH }\n#[cfg(test)]\nmod outer {\n    mod tests {\n        use super::super::*;\n        #[test]\n        fn stamps() { let _ = SystemTime::now(); }\n    }\n}\n",
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[("src/lib.rs", "pub mod clock;\n"), ("src/clock.rs", clock)],
        );
        let found = ["std::time::SystemTime::now in crate::clock"];
        assert_inline_answers(&probe, package, "crate::clock", "std::time", &found, &found);
    }
}

/// E6: `#[cfg(unix)] use crate::a::X;` beside `#[cfg(not(unix))] use crate::b::X;` — the scanner reads source
/// cfg-blind, so both bindings are candidates and the call reports under either prefix.
#[test]
fn inline_cfg_alternative_uses_are_both_observed() {
    let probe = RootProbe::new(
        "cfgnameduses",
        "",
        &with_same_named_x(
            "#[cfg(unix)]\nuse crate::a::X;\n#[cfg(not(unix))]\nuse crate::b::X;\npub fn g() { let _ = X::f(); }\n",
        ),
    );
    let a = ["crate::a::X::f in crate::core"];
    assert_inline_answers(&probe, "cfgnameduses", "crate::core", "crate::a", &a, &a);
    let b = ["crate::b::X::f in crate::core"];
    assert_inline_answers(&probe, "cfgnameduses", "crate::core", "crate::b", &b, &b);
}

/// D: in an edition-2015 package, a path beginning with `::` and a `use` path both start at the crate root, so
/// `::clock::now()` in `crate::core` and `use clock::now; now()` in `crate::use2015` both call `crate::clock::now`.
#[test]
fn inline_edition_2015_root_paths_resolve_from_the_crate_root() {
    let (manifest, mut files) = renamed_dependency("md5x");
    for (path, contents) in [
        (
            "src/lib.rs",
            "extern crate md5x;\npub mod clock;\npub mod core;\npub mod use2015;\n",
        ),
        ("src/clock.rs", "pub fn now() -> u64 { 0 }\n"),
        (
            "src/core.rs",
            "pub fn g() -> u64 { ::clock::now() }\npub fn h() -> u32 { ::md5x::compute() }\npub fn i() -> u32 { md5x::compute() }\n",
        ),
        (
            "src/use2015.rs",
            "use clock::now;\npub fn j() -> u64 { now() }\n",
        ),
    ] {
        files.push((path.to_string(), contents.to_string()));
    }
    let probe = RootProbe::with_edition("edition2015", "2015", &manifest, &borrowed(&files));
    for module in ["crate::core", "crate::use2015"] {
        let found = [format!("crate::clock::now in {module}")];
        let found = [found[0].as_str()];
        assert_inline_answers(
            &probe,
            "edition2015",
            module,
            "crate::clock",
            &found,
            &found,
        );
    }
}

/// A `type` alias written in a function body binds only inside it: the module's own `Clock` outside that body is
/// not read through the alias.
#[test]
fn a_block_local_type_alias_does_not_bind_outside_its_block() {
    let probe = RootProbe::new(
        "blockalias",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "pub struct Clock;\nimpl Clock { pub fn tick() -> u8 { 0 } }\npub fn a() -> std::time::SystemTime { type Clock = std::time::SystemTime; Clock::now() }\npub fn b() -> u8 { Clock::tick() }\n",
            ),
        ],
    );
    let found = ["std::time::SystemTime::now in crate::core"];
    assert_inline_answers(
        &probe,
        "blockalias",
        "crate::core",
        "std::time",
        &found,
        &found,
    );
}

/// An associated type is named through its type or `Self::`, never bare: the module's own `Item` is not read
/// through an `impl`'s `type Item`.
#[test]
fn an_associated_type_does_not_bind_a_bare_head() {
    let probe = RootProbe::new(
        "associatedtype",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "pub struct Item;\nimpl Item { pub fn tick() -> u8 { 0 } }\npub struct S;\nimpl Iterator for S { type Item = std::time::SystemTime; fn next(&mut self) -> Option<Self::Item> { None } }\npub fn b() -> u8 { Item::tick() }\n",
            ),
        ],
    );
    assert_inline_answers(
        &probe,
        "associatedtype",
        "crate::core",
        "std::time",
        &[],
        &[],
    );
}

/// P1–P4: a brace inside a char or byte literal or a raw string, or beside a lifetime, opens and closes no scope.
/// Each fixture puts the module-level `use crate::a::X` call after a function rebinding `X` to `crate::b::X`, so a
/// scope left open by a literal brace would read that call through the wrong binding; P3's block tuple struct
/// shadows an imported function in the value namespace.
#[test]
fn a_brace_in_a_literal_or_beside_a_lifetime_opens_no_scope() {
    const A: &str = "pub struct X;\nimpl X { pub fn fa() -> u8 { 0 } }\npub fn mk(_: u8) {}\n";
    const B: &str = "pub struct X;\nimpl X { pub fn fb() -> u16 { 0 } }\n";
    const H: &str = "pub fn h() -> u8 { X::fa() }\n";
    let rows: [(&str, String, &[&str], &[&str]); 4] = [
        (
            "literalp1",
            format!("use crate::a::X;\npub fn g() -> u16 {{ use crate::b::X; let _ = '{{'; X::fb() }}\n{H}"),
            &["crate::a::X::fa in crate::core"],
            &["crate::b::X::fb in crate::core"],
        ),
        (
            "literalp2",
            format!("use crate::a::X;\npub fn g() -> u16 {{ let _ = r#\"}}\"#; use crate::b::X; X::fb() }}\n{H}"),
            &["crate::a::X::fa in crate::core"],
            &["crate::b::X::fb in crate::core"],
        ),
        (
            "literalp3",
            "#[allow(unused_imports)]\nuse crate::a::mk;\n#[allow(non_camel_case_types)]\npub fn g() { struct mk(u8); let _ = mk(0); }\n".to_string(),
            &[],
            &[],
        ),
        (
            "literalp4",
            format!("use crate::a::X;\npub fn g<'q>(s: &'q str) -> u16 {{ use crate::b::X; let _ = b'}}'; let _ = s; X::fb() }}\n{H}"),
            &["crate::a::X::fa in crate::core"],
            &["crate::b::X::fb in crate::core"],
        ),
    ];
    let mut mismatches = Vec::new();
    for (package, core, under_a, under_b) in rows {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod a;\npub mod b;\npub mod core;\n"),
                ("src/a.rs", A),
                ("src/b.rs", B),
                ("src/core.rs", &core),
            ],
        );
        for (prefix, expected) in [("crate::a", under_a), ("crate::b", under_b)] {
            for strict_external in [false, true] {
                let found =
                    inline_findings(&probe, package, "crate::core", prefix, strict_external);
                if found != expected {
                    mismatches.push(format!(
                        "{package}: {prefix}, strict_external = {strict_external}: {found:?}, expected {expected:?}"
                    ));
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A block holding `#[cfg(unix)] type X = L;` for a block-local `L` beside `#[cfg(not(unix))] use crate::a::X;`
/// has two candidates for `X`. The alias names only a local item, which no prefix reaches, and that is no reason to
/// drop the import: the call reports under `crate::a` in either order, as the same pair does at module level.
#[test]
fn a_block_candidate_naming_a_local_item_keeps_the_other_candidates() {
    const L: &str = "    #[cfg(unix)] struct L;\n    #[cfg(unix)] impl L { fn f() -> u8 { 1 } }\n    #[cfg(unix)] type X = L;\n";
    const USE: &str = "    #[cfg(not(unix))] use crate::a::X;\n";
    let mut mismatches = Vec::new();
    for (package, body) in [
        ("blockcandaliasfirst", format!("{L}{USE}")),
        ("blockcandusefirst", format!("{USE}{L}")),
    ] {
        let core = format!("pub fn g() -> u8 {{\n{body}    X::f()\n}}\n");
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod a;\npub mod core;\n"),
                (
                    "src/a.rs",
                    "pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }\n",
                ),
                ("src/core.rs", &core),
            ],
        );
        for strict_external in [false, true] {
            let found =
                inline_findings(&probe, package, "crate::core", "crate::a", strict_external);
            if found != ["crate::a::X::f in crate::core"] {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {found:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// R2: `std::time` and `::std::time` name one crate, so a baseline recorded under either spelling suppresses the
/// same finding declared under the other.
#[test]
fn a_root_qualified_prefix_shares_its_baseline_identity() {
    let probe = RootProbe::new(
        "rootidentity",
        "",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "pub fn g() -> std::time::SystemTime { std::time::SystemTime::now() }\n",
            ),
        ],
    );
    let law = |prefix: &str| {
        Constitution::new("root-identity").boundary(
            ModuleBoundary::in_crate("rootidentity")
                .module("crate::core")
                .must_not_call_inline(prefix)
                .because("time is injected"),
        )
    };
    for (recorded, declared) in [("::std::time", "std::time"), ("std::time", "::std::time")] {
        let Outcome::Violations(accepted) = check(&law(recorded), probe.manifest()) else {
            panic!("{recorded}: the call must react before it can be baselined");
        };
        let baseline = xuanji::Baseline::of(&accepted);
        let Outcome::Violations(mut report) = check(&law(declared), probe.manifest()) else {
            panic!("{declared}: the call must react");
        };
        xuanji::apply_baseline(&mut report, &baseline);
        assert!(
            report.violations.iter().all(|v| v.baselined),
            "a baseline recorded under {recorded} suppresses {declared}: {:?}",
            report.violations
        );
    }
}

/// R6: `<W>::md5x()` calls `W`'s associated function; the `::` after the angle close continues a qualified path
/// and roots nothing, so an external `md5x` prefix is not reached in either mode.
#[test]
fn inline_associated_path_after_angle_close_is_not_global_root() {
    let (manifest, mut files) = renamed_dependency("md5x");
    files.push(("src/lib.rs".to_string(), "pub mod core;\n".to_string()));
    files.push((
        "src/core.rs".to_string(),
        "pub struct W;\nimpl W { pub fn md5x() {} }\npub fn g() { <W>::md5x(); }\n".to_string(),
    ));
    let probe = RootProbe::new("anglecloseroot", &manifest, &borrowed(&files));
    assert_inline_answers(&probe, "anglecloseroot", "crate::core", "md5x", &[], &[]);
}
