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
    dir: xingbiao::ScratchRoot,
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
        let dir = xingbiao::scratch_root(&format!("guibiao-single-root-{name}"));
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

    /// Declare a path dependency keyed `key`, written beneath the probe with `lib_rs` as its library, so a
    /// fixture importing a crate names one that exists.
    fn with_path_dependency(self, key: &str, lib_rs: &str) -> Self {
        let crate_dir = self.dir.join(key);
        std::fs::create_dir_all(crate_dir.join("src")).expect("create dependency dir");
        std::fs::write(
            crate_dir.join("Cargo.toml"),
            format!("[package]\nname = \"{key}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )
        .expect("write dependency manifest");
        std::fs::write(crate_dir.join("src/lib.rs"), lib_rs).expect("write dependency source");
        let manifest = std::fs::read_to_string(&self.manifest).expect("read Cargo.toml");
        std::fs::write(
            &self.manifest,
            format!("{manifest}\n[dependencies.{key}]\npath = \"{key}\"\n"),
        )
        .expect("write Cargo.toml");
        self
    }
}

/// The crate the external-confinement fixtures confine: a unit struct `B` and a function `helper`.
const BRICK: &str = "pub struct B;\npub fn helper() {}\n";

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
    let shared = xingbiao::scratch_root("guibiao-out-of-package-shared");
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
    )
    .with_path_dependency("brick", BRICK);

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
            let probe = RootProbe::new(&package, "", files).with_path_dependency("brick", BRICK);
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
        let probe =
            RootProbe::new(&package, manifest_extra, files).with_path_dependency("brick", BRICK);
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
    )
    .with_path_dependency("brick", BRICK);
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
    let probe = RootProbe::new(package, manifest_extra, files).with_path_dependency("brick", BRICK);
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
    )
    .with_path_dependency("brick", BRICK);
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
    )
    .with_path_dependency("brick", BRICK);
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
        ("cleanother", "use std::fmt::Write;\nfn main() {}\n", None),
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
    )
    .with_path_dependency("brick", BRICK);
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
    )
    .with_path_dependency("brick", BRICK);
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

/// A bare `Command` constructor call, assembled from two literals for the same reason as [`spawn_call!`].
macro_rules! command_new_call {
    () => {
        concat!("Command", "::new(\"x\")")
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

/// A sibling of the permitted module calls the confined prefix inline, and the call is outside the permitted
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

/// A binary root whose graph has no permitted module calls the confined prefix inline. Its permitted region is
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

/// An aliased import outside the permitted module does not hide the call; its `use` binding resolves the alias.
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

/// A glob import that can bring a prefix-resolving name into scope outside the permitted module reacts
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

/// At `warn()` the finding is reported as an advisory and the check still exits clean.
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

/// A baseline accepting one finding does not accept a second one added later in another module.
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

/// The permitted module and its inline test module call the confined prefix, which is where it is permitted.
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

/// Under the call-versus-mention default, naming the type outside the permitted module is not a call.
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

/// Narrowed to `new`, a call under the prefix with another terminal segment outside the permitted module is
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

/// The permitted module's own private `use`, and `use super::*` inside a sibling's test module with no alias
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

/// Permitting the prefix within `crate` permits it everywhere, so the rule could never react.
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

/// A permitted module no compiled root declares is a constitution error, not an empty region everywhere.
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

/// An empty confined prefix is a misdeclaration, as it is for `must_not_call_inline`.
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
    assert_eq!(violations[0].finding, "glob crate::agent in crate::agent");
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
    assert_eq!(violations[0].finding, "glob crate::agent in crate::agent");
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

/// Nested inline modules use the corresponding ancestor for `super::super::*`, not the file module, and the glob's
/// finding names that ancestor.
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
    assert_eq!(violations[0].finding, "glob crate::agent in crate::agent");
}

/// File-module `self`/`super` globs resolve from the file module, and each finding names the module its glob
/// resolved to. The `self` glob names a child, since `use self::*;` cannot glob-import a module into itself (E0432).
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
                "type Spawner = std::process::Command;\nmod tools {\n    pub type Spawner = std::process::Command;\n}\nuse self::tools::*;\nuse super::*;\npub fn act() {}\n",
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
            .any(|v| v.finding == "glob crate::agent::tools in crate::agent")
    );
    assert!(
        violations
            .iter()
            .any(|v| v.finding == "glob crate in crate::agent")
    );
}

/// At `ScanDepth::Shallow` the permitted region is the anchored module alone, so the permitted file's inline
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
        "[dependencies]\nmd5x = { path = \"md5x_dep\", package = \"md5x_pkg\" }\n",
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
                "[package]\nname = \"md5x_pkg\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
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

/// A prefix head that can never name a crate or module — `_`, a raw `crate`/`self`/`super`/`Self`/`_`, a bare
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
    let mut wrong = Vec::new();
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
            let repair = match suggestion {
                Some(spelling) => format!("write `{spelling}`"),
                None => "write the path from `crate`".to_string(),
            };
            match &outcome {
                Outcome::ConstitutionError(message)
                    if message.contains(&format!("names '{prefix}'")) && message.contains(&repair) => {}
                other => wrong.push(format!(
                    "{module}({prefix}): expected exit 2 quoting it and repairing with {repair:?}, got {other:?}"
                )),
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
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

/// A sysroot call written in full reacts under the relative prefix and under the `::`-rooted one.
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

/// An un-`use`d dependency call is outside the default and inside strict-external, whichever root spelling
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

/// A crate-root `mod md5x` shadows the same-named dependency for a bare head, so an external-only prefix
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

/// A boundary prefix is a name, not source, so a head that is a keyword in some edition is accepted bare or
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

/// A `use` inside a function body, or inside a block nested in one, resolves the calls in that block.
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

/// A function-local `use crate::b::X` shadows the module's `use crate::a::X` inside that function.
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

/// A block's `use` covers the whole block, including a call written before it.
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

/// A path beginning with `<` names its associated item through a type, which needs inference the scanner
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

/// Two globs that can each bring `X` into scope — used or not, cfg-exclusive, or one item reached
/// twice — react through the glob hazard, naming each glob that reaches the prefix. Where `X::f()` is called, the
/// globs are also followed to the modules they name, so the call reports under each candidate the prefix reaches.
///
/// The two-glob call is deliberately source rustc rejects — `X::f()` through two globs of `X` is `error[E0659]: `X` is ambiguous`
/// — kept because the scanner reads the glob hazard and both candidates before any compiler would stop it.
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

/// A named import beside a glob wins the name, and the glob still reacts as a hazard under the prefix it
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

/// An associated `const`, an enum variant and an associated type named like an import are reached
/// only through a path, so none shadows the import. The associated type is the type-namespace member, the one a lookup keyed by
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

/// Rust resolves `Command` to the generic parameter; the scanner does not read parameter lists and reads the
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

/// `crate::core` imports `crate::a::X` at module level, `h` calls `X::fa()`, and `g` declares
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

/// A function body's own `struct Command` shadows the module's `use std::process::Command` inside that body.
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

/// `crate::clock` privately imports `std::time::SystemTime`, and its inline test module, through `use super::*`
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

/// A glob carries an ancestor's private glob, so `crate::agent`'s `use super::*` beneath a crate root holding a
/// private `use std::process::*` reacts: nothing enumerates what the external glob brings, and the `id()` it makes
/// callable resolves to nothing. A sibling's private glob is not visible from `crate::agent`, so a glob of
/// that sibling does not react. A private concrete import is enumerated instead: a descendant's `use super::*`
/// holding no call reports nothing, and a call through it reports as a call, as a private `extern crate` does.
#[test]
fn a_glob_carries_an_ancestors_private_glob_and_enumerates_its_private_imports() {
    for (package, lib, agent, found) in [
        (
            "privglobancestor",
            "use std::process::*;\npub mod agent;\n",
            "use super::*;\npub fn f() -> u32 { id() }\n",
            &["glob crate in crate::agent"][..],
        ),
        (
            "privglobsibling",
            "pub mod other;\npub mod agent;\n",
            "use crate::other::*;\npub fn f() { g(); }\n",
            &[][..],
        ),
        (
            "privuseglobonly",
            "use std::process::id;\npub mod agent;\npub fn pid() -> u32 { id() }\n",
            "use super::*;\npub fn f(_: fn() -> u32) {}\n",
            &[][..],
        ),
        (
            "privuseglobcall",
            "use std::process::id;\npub mod agent;\n",
            "use super::*;\npub fn f() -> u32 { id() }\n",
            &["std::process::id in crate::agent"][..],
        ),
        (
            "privexternglobcall",
            "extern crate std as sp;\npub mod agent;\n",
            "use super::*;\npub fn f() -> u32 { sp::process::id() }\n",
            &["std::process::id in crate::agent"][..],
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", lib),
                ("src/other.rs", "use std::process::*;\npub fn g() {}\n"),
                ("src/agent.rs", agent),
            ],
        );
        assert_inline_answers(
            &probe,
            package,
            "crate::agent",
            "std::process",
            found,
            found,
        );
    }
}

/// A glob carries the private globs of the module it names alone: `crate::a::b` privately writes
/// `use std::process::*;`, and `crate::a::b::c`, which can see it, writes `use crate::a::*;`, which brings `crate::a`'s
/// names — `b` among them — and none of `b`'s. So the glob does not react under `std::process`, though the walk
/// reads every module beneath `crate::a`. Compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_glob_carries_no_private_glob_of_a_module_beneath_its_target() {
    let probe = RootProbe::new(
        "privglobbeneath",
        "",
        &[
            ("src/lib.rs", "pub mod a;\n"),
            ("src/a.rs", "pub mod b;\n"),
            (
                "src/a/b.rs",
                "#[allow(unused_imports)]\nuse std::process::*;\npub mod c;\n",
            ),
            (
                "src/a/b/c.rs",
                "#[allow(unused_imports)]\nuse crate::a::*;\npub fn f() {}\n",
            ),
        ],
    );
    assert_inline_answers(
        &probe,
        "privglobbeneath",
        "crate::a::b::c",
        "std::process",
        &[],
        &[],
    );
}

/// `#[cfg(unix)] use crate::a::X;` beside `#[cfg(not(unix))] use crate::b::X;` — the scanner reads source
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

/// In an edition-2015 package, a path beginning with `::` and a `use` path both start at the crate root, so
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

/// A brace inside a char or byte literal or a raw string, or beside a lifetime, opens and closes no scope.
/// Each fixture puts the module-level `use crate::a::X` call after a function rebinding `X` to `crate::b::X`, so a
/// scope left open by a literal brace would read that call through the wrong binding; the `literalp3` row's block
/// tuple struct shadows an imported function in the value namespace.
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

/// `std::time` and `::std::time` name one crate, so a baseline recorded under either spelling suppresses the
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

/// `<W>::md5x()` calls `W`'s associated function; the `::` after the angle close continues a qualified path
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

/// A `fn` item's own name is its definition, never a call: under a single-segment `md5x` prefix with strict
/// external on, a nested `fn md5x()`, an associated `fn md5x()` and a trait's `fn md5x();` react to nothing. The
/// nested one is also a block-local item; the associated and trait ones sit in bodies that record no member, so
/// only the definition reading keeps their `md5x(` from resolving to the dependency.
#[test]
fn a_fn_name_is_its_definition_not_a_call() {
    let mut mismatches = Vec::new();
    for (package, core) in [
        ("fndefnested", "pub fn g() { fn md5x() {} }\n"),
        (
            "fndefassociated",
            "pub struct W;\nimpl W { pub fn md5x() -> u8 { 0 } }\n",
        ),
        ("fndeftrait", "pub trait T { fn md5x(); }\n"),
    ] {
        let (manifest, mut files) = renamed_dependency("md5x");
        files.push(("src/lib.rs".to_string(), "pub mod core;\n".to_string()));
        files.push(("src/core.rs".to_string(), core.to_string()));
        let probe = RootProbe::new(package, &manifest, &borrowed(&files));
        for strict_external in [false, true] {
            let found = inline_findings(&probe, package, "crate::core", "md5x", strict_external);
            if !found.is_empty() {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {found:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A glob brings only the names visible where it is written: `crate::a::support` holds a private `Hidden`, a
/// `pub(super)` `Near` and a `pub(in crate::a)` `Far`, which `crate::core`'s glob of it cannot name, so their `now()`
/// calls there are `crate::clocks`' items and none reports under `crate::a::support`; its `pub`, `pub(in crate)` and
/// `pub(crate)` functions are named, and report beside the glob hazard, which reacts to the glob naming the prefix
/// itself.
#[test]
fn a_glob_brings_only_the_names_visible_where_it_is_written() {
    let probe = RootProbe::new(
        "globvisibility",
        "",
        &[
            ("src/lib.rs", "pub mod a;\npub mod clocks;\npub mod core;\n"),
            ("src/a.rs", "pub mod support;\n"),
            (
                "src/a/support.rs",
                "#[allow(dead_code)]\nstruct Hidden;\n#[allow(dead_code)]\npub(super) struct Near;\npub fn helper() {}\n\
                 #[allow(dead_code)]\npub(in crate::a) struct Far;\npub(in crate) fn wide() {}\npub(crate) fn crate_wide() {}\n",
            ),
            (
                "src/clocks.rs",
                "pub struct Hidden;\nimpl Hidden { pub fn now() -> u8 { 1 } }\npub struct Near;\nimpl Near { pub fn now() -> u8 { 1 } }\n\
                 pub struct Far;\nimpl Far { pub fn now() -> u8 { 1 } }\n",
            ),
            (
                "src/core.rs",
                "use crate::a::support::*;\nuse crate::clocks::*;\n\
                 pub fn g() -> u8 { helper(); wide(); crate_wide(); Hidden::now() + Near::now() + Far::now() }\n",
            ),
        ],
    );
    let found = [
        "crate::a::support::crate_wide in crate::core",
        "crate::a::support::helper in crate::core",
        "crate::a::support::wide in crate::core",
        "glob crate::a::support in crate::core",
    ];
    assert_inline_answers(
        &probe,
        "globvisibility",
        "crate::core",
        "crate::a::support",
        &found,
        &found,
    );
    let clocks = [
        "crate::clocks::Far::now in crate::core",
        "crate::clocks::Hidden::now in crate::core",
        "crate::clocks::Near::now in crate::core",
        "glob crate::clocks in crate::core",
    ];
    assert_inline_answers(
        &probe,
        "globvisibility",
        "crate::core",
        "crate::clocks",
        &clocks,
        &clocks,
    );
}

/// In an edition-2015 package a `use` path and a `::`-rooted path start at the crate root, so they name a
/// crate-root item as well as a crate-root module: `use now;` and `::now()` both call `crate::now`.
#[test]
fn inline_edition_2015_root_paths_reach_a_crate_root_item() {
    let probe = RootProbe::with_edition(
        "rootitem2015",
        "2015",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod usepath;\npub mod rooted;\npub fn now() -> u64 { 0 }\n",
            ),
            ("src/usepath.rs", "use now;\npub fn g() -> u64 { now() }\n"),
            ("src/rooted.rs", "pub fn h() -> u64 { ::now() }\n"),
        ],
    );
    let mut mismatches = Vec::new();
    for module in ["crate::usepath", "crate::rooted"] {
        let expected = [format!("crate::now in {module}")];
        for strict_external in [false, true] {
            let found = inline_findings(
                &probe,
                "rootitem2015",
                module,
                "crate::now",
                strict_external,
            );
            if found != expected {
                mismatches.push(format!(
                    "{module}, strict_external = {strict_external}: {found:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `{self}` leaf binds the group's prefix module under its last segment, or under its alias: `use std::io::{self,
/// Write};` binds `io`, `use crate::a::{self};` binds `a`, and `use std::time::{self as t};` binds `t`, so a call
/// through each reports under the module's prefix.
#[test]
fn a_self_leaf_binds_its_module_under_its_last_segment_or_alias() {
    let mut mismatches = Vec::new();
    for (package, core, prefix, found) in [
        (
            "selfleafio",
            "#[allow(unused_imports)]\nuse std::io::{self, Write};\npub fn g() { let _ = io::stdout(); }\n",
            "std::io",
            "std::io::stdout in crate::core",
        ),
        (
            "selfleafcrate",
            "use crate::a::{self};\npub fn g() { a::X::f(); }\n",
            "crate::a",
            "crate::a::X::f in crate::core",
        ),
        (
            "selfleafalias",
            "use std::time::{self as t};\npub fn g() { let _ = t::Instant::now(); }\n",
            "std::time",
            "std::time::Instant::now in crate::core",
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod a;\npub mod core;\n"),
                ("src/a.rs", "pub struct X;\nimpl X { pub fn f() {} }\n"),
                ("src/core.rs", core),
            ],
        );
        for strict_external in [false, true] {
            let got = inline_findings(&probe, package, "crate::core", prefix, strict_external);
            if got != [found] {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// One inline confinement's whole outcome over a probe, for the directions whose answer is not a finding list.
fn inline_outcome(
    probe: &RootProbe,
    package: &str,
    module: &str,
    prefix: &str,
    strict_external: bool,
) -> Outcome {
    let draft = ModuleBoundary::in_crate(package)
        .module(module)
        .must_not_call_inline(prefix);
    let draft = if strict_external {
        draft.strict_external()
    } else {
        draft
    };
    check(
        &Constitution::new("inline-outcome").boundary(draft.because("inline path resolution")),
        probe.manifest(),
    )
}

/// Two modules each defining `X` with an associated `f`, inline at the crate root.
const TWO_XS: &str = "pub mod a { pub struct X; impl X { pub fn f() {} } }\npub mod b { pub struct X; impl X { pub fn f() {} } }\n";

/// A support module re-exports `X` from `crate::a` under one cfg and from `crate::b` under the other, and the crate
/// root imports it from there. The scanner reads source cfg-blind, so each re-export is a candidate for what
/// `crate::support::X` names, in whichever order the two are written, and the call reports under each prefix.
#[test]
fn a_cfg_exclusive_reexport_reports_under_each_candidate() {
    let mut mismatches = Vec::new();
    for (package, reexports) in [
        (
            "cfgreexportab",
            "#[cfg(unix)] pub use crate::a::X; #[cfg(not(unix))] pub use crate::b::X;",
        ),
        (
            "cfgreexportba",
            "#[cfg(unix)] pub use crate::b::X; #[cfg(not(unix))] pub use crate::a::X;",
        ),
    ] {
        let lib = format!(
            "{TWO_XS}mod support {{ {reexports} }}\nuse crate::support::X;\npub fn g() {{ X::f() }}\n"
        );
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib)]);
        for (prefix, found) in [
            ("crate::a", "crate::a::X::f in crate"),
            ("crate::b", "crate::b::X::f in crate"),
        ] {
            for strict_external in [false, true] {
                let got = inline_findings(&probe, package, "crate", prefix, strict_external);
                if got != [found] {
                    mismatches.push(format!(
                        "{package} under {prefix}, strict_external = {strict_external}: {got:?}"
                    ));
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A support module imports `X` from `crate::a` under one cfg and from `crate::b` under the other, and publishes
/// `pub type Y = X;`. The alias's target is read in the support module's own scope, where both imports bind `X`, so
/// a call through `crate::support::Y` reports under each prefix in either order.
#[test]
fn a_module_alias_of_cfg_exclusive_imports_reports_under_each_candidate() {
    let mut mismatches = Vec::new();
    for (package, imports) in [
        (
            "cfgaliasab",
            "#[cfg(unix)] use crate::a::X; #[cfg(not(unix))] use crate::b::X;",
        ),
        (
            "cfgaliasba",
            "#[cfg(unix)] use crate::b::X; #[cfg(not(unix))] use crate::a::X;",
        ),
    ] {
        let lib = format!(
            "{TWO_XS}mod support {{ {imports} pub type Y = X; }}\nuse crate::support::Y;\npub fn g() {{ Y::f() }}\n"
        );
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib)]);
        for (prefix, found) in [
            ("crate::a", "crate::a::X::f in crate"),
            ("crate::b", "crate::b::X::f in crate"),
        ] {
            for strict_external in [false, true] {
                let got = inline_findings(&probe, package, "crate", prefix, strict_external);
                if got != [found] {
                    mismatches.push(format!(
                        "{package} under {prefix}, strict_external = {strict_external}: {got:?}"
                    ));
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A block-local alias whose target begins with `::` names a sysroot crate, as its unrooted spelling does:
/// `type Clock = ::std::time::SystemTime;` inside a function body, then `Clock::now()`.
#[test]
fn a_root_qualified_block_alias_resolves_to_std() {
    let probe = RootProbe::new(
        "rootedblockstd",
        "",
        &[(
            "src/lib.rs",
            "pub fn g() { type Clock = ::std::time::SystemTime; let _ = Clock::now(); }\n",
        )],
    );
    let found = ["std::time::SystemTime::now in crate"];
    assert_inline_answers(
        &probe,
        "rootedblockstd",
        "crate",
        "std::time",
        &found,
        &found,
    );
}

/// The same for `core`: `type D = ::core::time::Duration;` inside a function body, then `D::from_secs(1)`.
#[test]
fn a_root_qualified_block_alias_resolves_to_core() {
    let probe = RootProbe::new(
        "rootedblockcore",
        "",
        &[(
            "src/lib.rs",
            "pub fn g() { type D = ::core::time::Duration; let _ = D::from_secs(1); }\n",
        )],
    );
    let found = ["core::time::Duration::from_secs in crate"];
    assert_inline_answers(
        &probe,
        "rootedblockcore",
        "crate",
        "core::time",
        &found,
        &found,
    );
}

/// In an edition-2015 package a path beginning with `::` starts at the crate root, in a block-local alias as
/// anywhere else: `type K = ::clock::C;` inside a function body names the crate-root module's `C`.
#[test]
fn an_edition_2015_root_qualified_block_alias_resolves_from_the_crate_root() {
    let probe = RootProbe::with_edition(
        "rootedblock2015",
        "2015",
        "",
        &[(
            "src/lib.rs",
            "pub mod clock { pub struct C; impl C { pub fn now() {} } }\npub fn g() { type K = ::clock::C; K::now(); }\n",
        )],
    );
    let found = ["crate::clock::C::now in crate"];
    assert_inline_answers(
        &probe,
        "rootedblock2015",
        "crate",
        "crate::clock",
        &found,
        &found,
    );
}

/// A `<` that compares opens no angle group: in `0 < n && n > ::std::process::id()` the `>` is a comparison too, so
/// the `::` after it begins a path rooted at `std` and the call reports.
#[test]
fn a_comparison_before_a_root_qualified_call_opens_no_angle_group() {
    let probe = RootProbe::new(
        "comparisonrooted",
        "",
        &[(
            "src/lib.rs",
            "pub fn g(n: u32) -> bool { 0 < n && n > ::std::process::id() }\n",
        )],
    );
    let found = ["std::process::id in crate"];
    assert_inline_answers(
        &probe,
        "comparisonrooted",
        "crate",
        "std::process",
        &found,
        &found,
    );
}

/// Controls beside the rooted-path directions above, each an answer the scanner already gives: a module-level alias
/// whose target begins with `::`, a direct `::std::…` call, and a turbofish compared with a rooted call — whose `<`
/// closes its own group before the comparison's `>`.
#[test]
fn a_root_qualified_path_outside_a_block_alias_reacts() {
    let mut mismatches = Vec::new();
    for (package, lib, prefix, found) in [
        (
            "rootedmodulealias",
            "type Clock = ::std::time::SystemTime;\npub fn g() { let _ = Clock::now(); }\n",
            "std::time",
            "std::time::SystemTime::now in crate",
        ),
        (
            "rooteddirect",
            "pub fn g() { let _ = ::std::time::SystemTime::now(); }\n",
            "std::time",
            "std::time::SystemTime::now in crate",
        ),
        (
            "rootedturbofish",
            "fn f<T>() -> u32 { 0 }\npub fn g() -> bool { f::<u32>() > ::std::process::id() }\n",
            "std::process",
            "std::process::id in crate",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", lib)]);
        for strict_external in [false, true] {
            let got = inline_findings(&probe, package, "crate", prefix, strict_external);
            if got != [found] {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `<` after a keyword opens a qualified path: `return <W>::md5x();` calls `W`'s associated function, so the
/// `::` after the angle close roots nothing and an external `md5x` prefix is not reached in either mode.
#[test]
fn a_qualified_path_after_a_keyword_is_not_a_global_root() {
    let (manifest, mut files) = renamed_dependency("md5x");
    files.push((
        "src/lib.rs".to_string(),
        "pub struct W;\nimpl W { pub fn md5x() {} }\npub fn g() { return <W>::md5x(); }\n"
            .to_string(),
    ));
    let probe = RootProbe::new("keywordqualified", &manifest, &borrowed(&files));
    assert_inline_answers(&probe, "keywordqualified", "crate", "md5x", &[], &[]);
}

/// `type A0 = std::time::SystemTime;` followed by `n` aliases each naming the one before, in a function body or at
/// module level, and a call through the last.
fn alias_chain(links: usize, in_block: bool) -> String {
    let mut aliases = String::from("type A0 = std::time::SystemTime;\n");
    for i in 1..=links {
        aliases.push_str(&format!("type A{i} = A{};\n", i - 1));
    }
    if in_block {
        format!("pub fn g() {{\n{aliases}let _ = A{links}::now();\n}}\n")
    } else {
        format!("{aliases}pub fn g() {{ let _ = A{links}::now(); }}\n")
    }
}

/// `crate::m0` defines `X`, each `crate::m{i}` re-exports `crate::m{i-1}::X`, and the crate root calls through the
/// last.
fn reexport_chain(links: usize) -> String {
    let mut lib = String::from("pub mod m0 { pub struct X; impl X { pub fn f() {} } }\n");
    for i in 1..=links {
        lib.push_str(&format!(
            "pub mod m{i} {{ pub use crate::m{}::X; }}\n",
            i - 1
        ));
    }
    lib.push_str(&format!("pub fn g() {{ m{links}::X::f() }}\n"));
    lib
}

/// A chain of `type` aliases or re-exports longer than the scanner's measured nesting cap is refused as a scan
/// error that quotes where the chain was read, never resolved partway and dropped.
#[test]
fn an_alias_or_reexport_chain_past_the_depth_cap_is_a_scan_error() {
    let mut mismatches = Vec::new();
    for (package, lib, prefix, quoted) in [
        (
            "blockchain100",
            alias_chain(100, true),
            "std::time",
            "type A",
        ),
        (
            "modulechain100",
            alias_chain(100, false),
            "std::time",
            "type A",
        ),
        (
            "reexportchain100",
            reexport_chain(100),
            "crate::m0",
            "crate::m",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib)]);
        for strict_external in [false, true] {
            match inline_outcome(&probe, package, "crate", prefix, strict_external) {
                Outcome::ConstitutionError(message)
                    if message.contains("64") && message.contains(quoted) => {}
                other => mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {other:?}"
                )),
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The control for the cap: the same chains well under it resolve to the item they name.
#[test]
fn an_alias_or_reexport_chain_under_the_depth_cap_resolves() {
    let mut mismatches = Vec::new();
    for (package, lib, prefix, found) in [
        (
            "blockchain32",
            alias_chain(32, true),
            "std::time",
            "std::time::SystemTime::now in crate",
        ),
        (
            "modulechain32",
            alias_chain(32, false),
            "std::time",
            "std::time::SystemTime::now in crate",
        ),
        (
            "reexportchain32",
            reexport_chain(32),
            "crate::m0",
            "crate::m0::X::f in crate",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib)]);
        for strict_external in [false, true] {
            let got = inline_findings(&probe, package, "crate", prefix, strict_external);
            if got != [found] {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Deliberately source rustc rejects — `error[E0391]: cycle detected when expanding type alias` for the two alias
/// cycles, `error[E0432]: unresolved import` for the re-export cycle — since what is under test is that a cycle
/// ends resolution with a verdict rather than a panic or an unbounded walk. Which verdict is not asserted.
#[test]
fn a_cyclic_alias_or_reexport_ends_resolution() {
    for (package, lib, prefix) in [
        (
            "cycleblock",
            "pub fn g() { type A = B; type B = A; let _ = A::now(); }\n",
            "std::time",
        ),
        (
            "cyclemodule",
            "type A = B;\ntype B = A;\npub fn g() { let _ = A::now(); }\n",
            "std::time",
        ),
        (
            "cyclereexport",
            "pub mod a { pub use crate::b::X; }\npub mod b { pub use crate::a::X; }\npub fn g() { a::X::f() }\n",
            "crate::a",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", lib)]);
        for strict_external in [false, true] {
            let outcome = inline_outcome(&probe, package, "crate", prefix, strict_external);
            assert!(
                matches!(outcome, Outcome::Clean(_) | Outcome::Violations(_)),
                "{package}, strict_external = {strict_external}: {outcome:?}"
            );
        }
    }
}

/// The `>` of `->` inside a turbofish closes no group: `std::mem::size_of::<fn() -> u8>()` is one path applied as
/// a call, so it reports.
#[test]
fn a_turbofish_holding_a_fn_arrow_is_one_group() {
    let probe = RootProbe::new(
        "turbofisharrow",
        "",
        &[(
            "src/lib.rs",
            "pub fn g() -> usize { std::mem::size_of::<fn() -> u8>() }\n",
        )],
    );
    let found = ["std::mem::size_of in crate"];
    assert_inline_answers(
        &probe,
        "turbofisharrow",
        "crate",
        "std::mem",
        &found,
        &found,
    );
}

/// A single-file package whose `src/lib.rs` is `lib`.
fn lib_probe(package: &str, lib: &str) -> RootProbe {
    RootProbe::new(package, "", &[("src/lib.rs", lib)])
}

/// A `type` alias's target is read past a reference or pointer written before it, a lifetime and `mut` included:
/// rustc requires a lifetime on a reference in an alias, so `&'static` and `&'static mut` are the forms one
/// is written in, and a raw pointer is `*const` or `*mut`. Through a trait implemented for that type, `A::f()` is
/// a call that reports under the path the alias names. Each row is compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_type_alias_is_read_past_a_reference_or_pointer() {
    let items = "pub mod x { pub struct Y; }\npub trait T { fn f() -> u8; }\n";
    for (package, alias) in [
        ("aliasref", "&'static crate::x::Y"),
        ("aliasrefmut", "&'static mut crate::x::Y"),
        ("aliasconstptr", "*const crate::x::Y"),
        ("aliasmutptr", "*mut crate::x::Y"),
    ] {
        let lib = format!(
            "{items}impl T for {alias} {{ fn f() -> u8 {{ 0 }} }}\ntype A = {alias};\npub fn g() -> u8 {{ A::f() }}\n"
        );
        assert_crate_answers(package, &lib, "crate::x", &["crate::x::Y::f in crate"]);
    }
}

/// A generic `type` alias's parameter list closes where rustc splits its last token: at the `>` a `>=` or `>>=` holds,
/// the `=` it also holds being the alias's own. So `type A<T>= crate::x::Y<T>;` and `type A<T: Into<u8>>=
/// crate::x::Y<T>;`, written with no space before the `=`, bind `A` as the spaced forms do, and `A::<u8>::f()` reports
/// under `crate::x`. Each row is compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_generic_type_alias_closing_at_a_compound_token_binds_its_target() {
    for (package, alias) in [
        ("aliasge", "type A<T>= crate::x::Y<T>;"),
        ("aliasshreq", "type A<T: Into<u8>>= crate::x::Y<T>;"),
        ("aliasspaced", "type A<T: Into<u8>> = crate::x::Y<T>;"),
    ] {
        let lib = format!(
            "pub mod x {{ pub struct Y<T>(pub T); impl<T> Y<T> {{ pub fn f() -> u8 {{ 0 }} }} }}\n\
             #[allow(type_alias_bounds)]\n{alias}\npub fn g() -> u8 {{ A::<u8>::f() }}\n"
        );
        assert_crate_answers(package, &lib, "crate::x", &["crate::x::Y::f in crate"]);
    }
}

/// Every answer of one prefix over `crate`, as the same sorted findings in both modes.
fn assert_crate_answers(package: &str, lib: &str, prefix: &str, found: &[&str]) {
    let probe = lib_probe(package, lib);
    assert_inline_answers(&probe, package, "crate", prefix, found, found);
}

/// The answers of one prefix over `module` of `probe` that differ from `found`, in either mode, collected into
/// `mismatches`, so a table of fixtures names every row that fails rather than the first.
fn answer_mismatches(
    probe: &RootProbe,
    package: &str,
    module: &str,
    prefix: &str,
    found: &[&str],
    mismatches: &mut Vec<String>,
) {
    for strict_external in [false, true] {
        let got = inline_findings(probe, package, module, prefix, strict_external);
        if got != found {
            mismatches.push(format!(
                "{package} {module} under {prefix}, strict_external = {strict_external}: {got:?}"
            ));
        }
    }
}

/// A `use` path's first segment is looked up in the scope the `use` is written in before it names a crate: in
/// `src/core2/mod.rs`, `mod exec; use exec::Command;` names the child module `exec`, written in its own file, whose
/// `pub use std::process::Command` the call then reaches.
#[test]
fn a_uniform_use_path_through_a_child_module_in_another_file_resolves() {
    let probe = RootProbe::new(
        "uniformfile",
        "",
        &[
            ("src/lib.rs", "pub mod core2;\n"),
            (
                "src/core2/mod.rs",
                concat!(
                    "mod exec;\nuse exec::Command;\npub fn g() { let _ = Command",
                    "::new(\"true\"); }\n"
                ),
            ),
            ("src/core2/exec.rs", "pub use std::process::Command;\n"),
        ],
    );
    let found = ["std::process::Command::new in crate::core2"];
    assert_inline_answers(
        &probe,
        "uniformfile",
        "crate::core2",
        "std::process",
        &found,
        &found,
    );
}

/// The same with the child module inline: `use exec::Command;` beside `pub mod exec { pub use std::process::Command; }`.
#[test]
fn a_uniform_use_path_through_a_child_module_resolves() {
    assert_crate_answers(
        "uniforminline",
        concat!(
            "pub mod core2 { pub mod exec { pub use std::process::Command; } use exec::Command; pub fn g() { let _ = Command",
            "::new(\"true\"); } }\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// `use inner::X;` beside `pub mod inner` names the child module's own item.
#[test]
fn a_uniform_use_path_names_a_child_modules_item() {
    assert_crate_answers(
        "uniformitem",
        "pub mod m { pub mod inner { pub struct X; impl X { pub fn f() {} } } use inner::X; pub fn g() { X::f(); } }\n",
        "crate::m::inner",
        &["crate::m::inner::X::f in crate"],
    );
}

/// The same `use inner::X;` written in a function body: a block's `use` path starts from the block's scope, which sees
/// its module's items.
#[test]
fn a_block_uniform_use_path_names_a_child_modules_item() {
    assert_crate_answers(
        "uniformblockitem",
        "pub mod m { pub mod inner { pub struct X; impl X { pub fn f() {} } } pub fn g() { use inner::X; X::f(); } }\n",
        "crate::m::inner",
        &["crate::m::inner::X::f in crate"],
    );
}

/// `use m::inner; use inner::X;` at the crate root: the second path's head is the first `use`'s binding.
#[test]
fn a_uniform_use_path_through_an_imported_module_resolves() {
    assert_crate_answers(
        "uniformimported",
        "pub mod m { pub mod inner { pub struct X; impl X { pub fn f() {} } } }\nuse m::inner;\nuse inner::X;\npub fn g() { X::f(); }\n",
        "crate::m::inner",
        &["crate::m::inner::X::f in crate"],
    );
}

/// A `pub use` re-export is a `use` path too: `pub use exec::Command;` in `facade` names its private child `exec`,
/// and the crate root's `use facade::Command;` reaches `std::process::Command` through both.
#[test]
fn a_uniform_pub_use_reexport_resolves_through_a_private_child() {
    assert_crate_answers(
        "uniformfacade",
        concat!(
            "pub mod facade { mod exec { pub use std::process::Command; } pub use exec::Command; }\nuse facade::Command;\npub fn g() { let _ = Command",
            "::new(\"true\"); }\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// `use std::process; use process::Command;`: the second path's head is the first `use`'s binding of a sysroot module.
#[test]
fn a_uniform_use_path_through_an_imported_std_module_resolves() {
    assert_crate_answers(
        "uniformstd",
        concat!(
            "pub mod core2 { use std::process; use process::Command; pub fn g() { let _ = Command",
            "::new(\"true\"); } }\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// Each leaf of `use exec::{Command, Stdio};` starts from the same scope, so both calls report.
#[test]
fn a_uniform_use_group_resolves_each_leaf() {
    assert_crate_answers(
        "uniformgroup",
        concat!(
            "pub mod core2 { pub mod exec { pub use std::process::{Command, Stdio}; } use exec::{Command, Stdio}; pub fn g() { let _ = Stdio::null(); let _ = Command",
            "::new(\"x\"); } }\n"
        ),
        "std::process",
        &[
            "std::process::Command::new in crate",
            "std::process::Stdio::null in crate",
        ],
    );
}

/// `use exec::Command;` in a function body names its module's child `exec`.
#[test]
fn a_block_uniform_use_path_through_a_child_module_resolves() {
    assert_crate_answers(
        "uniformblock",
        concat!(
            "pub mod core2 { pub mod exec { pub use std::process::Command; } pub fn g() { use exec::Command; let _ = Command",
            "::new(\"x\"); } }\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// A child module named `std2` is a local name like any other: `use std2::Command;` names it, not a crate.
#[test]
fn a_uniform_use_path_head_named_like_a_crate_resolves_locally() {
    assert_crate_answers(
        "uniformstd2",
        concat!(
            "pub mod core2 { pub mod std2 { pub use std::process::Command; } use std2::Command; pub fn g() { let _ = Command",
            "::new(\"x\"); } }\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// A glob's path starts from its scope as a `use` path does: `use inner::*;` beside `pub mod inner` names
/// `crate::m::inner`, which is the glob's identity, and the call it brings `X` for reports beside it.
#[test]
fn a_uniform_glob_path_resolves_and_its_call_reports() {
    assert_crate_answers(
        "uniformglob",
        "pub mod m { pub mod inner { pub struct X; impl X { pub fn f() {} } } use inner::*; pub fn g() { X::f(); } }\n",
        "crate::m::inner",
        &[
            "crate::m::inner::X::f in crate",
            "glob crate::m::inner in crate",
        ],
    );
}

/// `use exec::*;` names `crate::m::exec`, whose `pub use crate::a::X` the glob hazard reaches, and the call through the
/// name the glob brings reports under `crate::a`.
#[test]
fn a_uniform_glob_of_a_reexporting_child_reports_the_call() {
    assert_crate_answers(
        "uniformglobreexport",
        "pub mod a { pub struct X; impl X { pub fn f() {} } }\npub mod m { pub mod exec { pub use crate::a::X; } use exec::*; pub fn g() { X::f(); } }\n",
        "crate::a",
        &["crate::a::X::f in crate", "glob crate::m::exec in crate"],
    );
}

/// `pub type A0 = u8;` followed by seventy aliases each naming the one before.
fn unused_alias_chain() -> String {
    let mut lib = String::from("pub type A0 = u8;\n");
    for i in 1..=70 {
        lib.push_str(&format!("pub type A{i} = A{};\n", i - 1));
    }
    lib
}

/// A chain longer than the cap that no call or glob resolves through is not read, so it refuses nothing.
#[test]
fn an_unused_alias_chain_past_the_cap_is_not_judged() {
    let lib = format!("{}pub fn g() {{}}\n", unused_alias_chain());
    assert_crate_answers("unusedchain", &lib, "std::time", &[]);
}

/// Nor does it keep a call that resolves elsewhere from reporting.
#[test]
fn a_call_beside_an_unused_alias_chain_past_the_cap_reports() {
    let lib = format!(
        "{}pub fn g() {{ let _ = std::time::SystemTime::now(); }}\n",
        unused_alias_chain()
    );
    assert_crate_answers(
        "unusedchaincall",
        &lib,
        "std::time",
        &["std::time::SystemTime::now in crate"],
    );
}

/// Controls beside the uniform-path directions, each an answer the scanner already gives: a `type` alias through a
/// child module, a `self::` path, a crate-root `use inner::X`, a direct `exec::Command::new`, a block glob of
/// `crate::a` with the call it brings, and a cfg-exclusive re-export in another file under each prefix.
#[test]
fn uniform_path_controls_keep_their_answers() {
    const X: &str = "pub struct X; impl X { pub fn f() {} }";
    for (package, lib, prefix, found) in [
        (
            "controlalias",
            "pub mod core2 { pub mod exec { pub use std::process::Command; } type C = exec::Command; pub fn g() { let _ = C::new(\"x\"); } }\n".to_string(),
            "std::process",
            vec!["std::process::Command::new in crate"],
        ),
        (
            "controlself",
            format!("pub mod m {{ pub mod inner {{ {X} }} use self::inner::X; pub fn g() {{ X::f(); }} }}\n"),
            "crate::m::inner",
            vec!["crate::m::inner::X::f in crate"],
        ),
        (
            "controlroot",
            format!("pub mod inner {{ {X} }}\nuse inner::X;\npub fn g() {{ X::f(); }}\n"),
            "crate::inner",
            vec!["crate::inner::X::f in crate"],
        ),
        (
            "controldirect",
            concat!("pub mod core2 { pub mod exec { pub use std::process::Command; } pub fn g() { let _ = exec::Command", "::new(\"x\"); } }\n").to_string(),
            "std::process",
            vec!["std::process::Command::new in crate"],
        ),
        (
            "controlblockglob",
            format!("pub mod a {{ {X} }}\npub fn g() {{ use crate::a::*; X::f(); }}\n"),
            "crate::a",
            vec!["crate::a::X::f in crate", "glob crate::a in crate"],
        ),
    ] {
        assert_crate_answers(package, &lib, prefix, &found);
    }
    let probe = RootProbe::new(
        "controlcfgfile",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod a; pub mod b; pub mod support; pub mod core2;\n",
            ),
            ("src/a.rs", X),
            ("src/b.rs", X),
            (
                "src/support.rs",
                "#[cfg(unix)] pub use crate::a::X;\n#[cfg(not(unix))] pub use crate::b::X;\n",
            ),
            (
                "src/core2.rs",
                "use crate::support::X;\npub fn g() { X::f(); }\n",
            ),
        ],
    );
    for prefix in ["crate::a", "crate::b"] {
        let found = [format!("{prefix}::X::f in crate::core2")];
        let found = [found[0].as_str()];
        assert_inline_answers(
            &probe,
            "controlcfgfile",
            "crate::core2",
            prefix,
            &found,
            &found,
        );
    }
}

/// Under `.strict_prefix_only()` a `use` leaf is judged as the `use` path it is, whatever tree it is written in: a
/// grouped `use crate::{clock::now};` holds no path `clock::now`, and in edition 2015 `use clock::now;` in a submodule
/// starts at the crate root. An empty group, `use crate::clock::{};`, imports nothing and names the path before it,
/// which rustc resolves. Nor is a leaf read as an expression path from its module: `use crate::other::{clock::now};`
/// in a module declaring its own `clock` mentions `crate::other::clock::now` alone. Each row is compiled by rustc
/// 1.96.0, in edition 2021 but for the 2015 row.
#[test]
fn a_use_leaf_is_judged_as_a_use_path_under_strict_prefix_only() {
    for (package, edition, use_line, expected) in [
        (
            "strictuseflat",
            "2021",
            "use crate::clock::now;",
            "crate::clock::now in crate",
        ),
        (
            "strictusegroup",
            "2021",
            "use crate::{clock::now};",
            "crate::clock::now in crate",
        ),
        (
            "strictusenested",
            "2021",
            "use crate::{clock::{now}};",
            "crate::clock::now in crate",
        ),
        (
            "strictuseouter",
            "2021",
            "use {crate::clock::now};",
            "crate::clock::now in crate",
        ),
        (
            "strictuseself",
            "2021",
            "use crate::clock::{self};",
            "crate::clock in crate",
        ),
        (
            "strictuse2015",
            "2015",
            "use clock::now;",
            "crate::clock::now in crate",
        ),
        (
            "strictuseempty",
            "2021",
            "use crate::clock::{};",
            "crate::clock in crate",
        ),
        (
            "strictusenestedempty",
            "2021",
            "use crate::{clock::{}};",
            "crate::clock in crate",
        ),
    ] {
        let probe = RootProbe::with_edition(
            package,
            edition,
            "",
            &[(
                "src/lib.rs",
                &format!(
                    "pub mod clock {{ pub fn now() {{}} }}\npub mod sub {{ #[allow(unused_imports)] {use_line} }}\n"
                ),
            )],
        );
        let law = Constitution::new("strict-use").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate")
                .must_not_call_inline("crate::clock")
                .strict_prefix_only()
                .depth(xuanji::ScanDepth::Subtree)
                .because("no mention of the clock"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert_eq!(
                report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect::<Vec<_>>(),
                [expected],
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected the use leaf to react, got {other:?}"),
        }
    }
    let package = "strictusenoexpression";
    let probe = lib_probe(
        package,
        "pub mod other { pub mod clock { pub fn now() {} } }\npub mod sub { pub mod clock { pub fn now() {} } \
         #[allow(unused_imports)] use crate::other::{clock::now}; }\n",
    );
    let law = Constitution::new("strict-use").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_call_inline("crate::sub::clock")
            .strict_prefix_only()
            .depth(xuanji::ScanDepth::Subtree)
            .because("no mention of sub's clock"),
    );
    let outcome = check(&law, probe.manifest());
    assert_eq!(
        outcome.exit_code(),
        0,
        "a grouped leaf is no expression path read from its module: {outcome:?}"
    );
}

/// A parenthesized bound of the `Fn` family is written as a call is — `F: std::ops::Fn(u8) -> u8`,
/// `impl std::ops::FnOnce()` — so it is read as one: the declared over-reaction. rustc 1.96.0, edition 2021, compiles
/// both, and neither calls anything.
#[test]
fn a_parenthesized_fn_bound_is_read_as_a_call() {
    for (package, item, expected) in [
        (
            "fnboundsugar",
            "pub fn f<F: std::ops::Fn(u8) -> u8>(g: F) -> u8 { g(1) }",
            "std::ops::Fn in crate",
        ),
        (
            "fnoncesugar",
            "pub fn h() -> impl std::ops::FnOnce() { || () }",
            "std::ops::FnOnce in crate",
        ),
        (
            "fnmutdynsugar",
            "pub fn k(g: &dyn std::ops::FnMut(u8)) -> usize { std::mem::size_of_val(g) }",
            "std::ops::FnMut in crate",
        ),
    ] {
        let probe = lib_probe(package, &format!("{item}\n"));
        assert_inline_answers(
            &probe,
            package,
            "crate",
            "std::ops",
            &[expected],
            &[expected],
        );
    }
}

/// A `type` alias a block declares of something no path names — a tuple, an array — is a block-local item, so a call
/// through it names nothing the module imports: rustc 1.96.0, edition 2021, calls the alias's `default` in
/// `fn g() { type Command = (u8, u8); let _ = Command::default(); }` beside `use std::process::Command;`. A block's alias
/// of a path is read through its target as ever, parenthesized or not.
#[test]
fn a_block_alias_of_no_path_is_a_block_local_item() {
    for (package, alias, call, expected) in [
        (
            "blockaliastuple",
            "type Command = (u8, u8);",
            "Command::default()",
            &[][..],
        ),
        (
            "blockaliasarray",
            "type Command = [u8; 2];",
            "Command::default()",
            &[][..],
        ),
        (
            "blockaliaspath",
            "type Command = std::process::Command;",
            "Command::new(\"true\")",
            &["std::process::Command::new in crate"][..],
        ),
        (
            "blockaliasparenthesized",
            "type Command = (std::process::Command);",
            "Command::new(\"true\")",
            &["std::process::Command::new in crate"][..],
        ),
    ] {
        let probe = lib_probe(
            package,
            &format!(
                "#[allow(unused_imports)]\nuse std::process::Command;\npub fn g() {{ {alias} let _ = {call}; }}\n"
            ),
        );
        assert_inline_answers(&probe, package, "crate", "std::process", expected, expected);
    }
}

/// A parenthesized alias target is read with the generic arguments a type path takes, so
/// `type C = (crate::secret::G<u8>);` in a block names `crate::secret::G`, and a call through it reports as it does
/// without the parentheses. rustc 1.96.0, edition 2021, calls `crate::secret::G::make`.
#[test]
fn a_parenthesized_alias_of_a_generic_path_is_read_through_it() {
    let package = "parengeneric";
    let probe = lib_probe(
        package,
        "pub mod secret { pub struct G<T>(pub T); impl<T> G<T> { pub fn make() {} } }\n\
         pub fn g() { type C = (crate::secret::G<u8>); C::make(); }\n",
    );
    let expected = ["crate::secret::G::make in crate"];
    assert_inline_answers(
        &probe,
        package,
        "crate",
        "crate::secret",
        &expected,
        &expected,
    );
}

/// A comma inside an enum discriminant's turbofish separates no variants, so the path after it is read as a path:
/// `enum E { A = f::<u8, std::process::Command>() }` mentions `std::process::Command` as the same call in a `const`
/// does, and reports under `.strict_prefix_only()`. rustc 1.96.0, edition 2021, compiles both with a generic
/// `const fn f`; so does a qualified path after an operator, `A = 1 + <u8 as Tr<u8, std::process::Command>>::X`,
/// whose `<` opens a path as it does in any expression.
#[test]
fn a_path_in_a_discriminants_turbofish_is_read() {
    for (package, item) in [
        (
            "discriminantturbofish",
            "pub enum E { A = f::<u8, std::process::Command>() }",
        ),
        (
            "constturbofish",
            "pub const K: isize = f::<u8, std::process::Command>();",
        ),
        (
            "discriminantqualifiedsum",
            "pub enum E { A = 1 + <u8 as Tr<u8, std::process::Command>>::X, B }",
        ),
        (
            "discriminantqualifiednegation",
            "pub enum E { A = -<u8 as Tr<u8, std::process::Command>>::X, B }",
        ),
    ] {
        let probe = lib_probe(
            package,
            &format!(
                "pub const fn f<A, B>() -> isize {{ 0 }}\npub trait Tr<A, B> {{ const X: isize; }}\n\
                 impl<A, B> Tr<A, B> for u8 {{ const X: isize = 5; }}\n{item}\n"
            ),
        );
        let law = Constitution::new("discriminant").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate")
                .must_not_call_inline("std::process")
                .strict_prefix_only()
                .depth(xuanji::ScanDepth::Subtree)
                .because("no mention of processes"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert_eq!(
                report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect::<Vec<_>>(),
                ["std::process::Command in crate"],
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected the mention to react, got {other:?}"),
        }
    }
}

/// Under `.strict_prefix_only()` a single identifier read as a value is a path mentioned, so `let g: fn() = now;` in
/// `crate::clock` reports `crate::clock::now`, where it went unreported because only a call, a rooted path or a path
/// of several segments was an occurrence. An item's name, a field and a parameter being declared are not mentions:
/// `pub fn now() {}`, a field `now: u8` and a parameter `now: u8` report nothing, nor do bindings `later` and `k`
/// nothing in scope bears. A binding's name is otherwise a mention left to the resolver, so a constant named in a
/// pattern, `if let DENIED = x`, reports, and a `let now` beside the module's `fn now` reports too — the declared
/// over-reaction `a-local-binding-named-like-an-import-is-read-as-the-import`. A block's `struct now
/// {}` holds the name as a type alone, so the value `now` is still the module's function, while a block's `fn now()
/// {}` shadows it; and `&mut Clock` mentions the unit struct `Clock`, its `mut` a reference's and not a binding's.
/// rustc 1.96.0, edition 2021, builds every row.
#[test]
fn a_single_identifier_read_as_a_value_is_mentioned_under_strict_prefix_only() {
    for (package, clock, prefix, found) in [
        (
            "singlevaluemention",
            "pub fn now() {}\npub fn run() {\n    let g: fn() = now;\n    g();\n}\n",
            "crate::clock::now",
            &["crate::clock::now in crate::clock"][..],
        ),
        (
            "singlenameintroduced",
            "pub fn now() {}\npub struct S { pub now: u8 }\npub fn f(now: u8) {}\npub fn h() { let mut later = 2u8; later += 1; for k in 0..later { let _ = k; } }\n",
            "crate::clock::now",
            &[][..],
        ),
        (
            "bindingnamedlikeitem",
            "pub fn now() {}\npub fn h() { let now = 1u8; let _ = now; }\n",
            "crate::clock::now",
            &["crate::clock::now in crate::clock"][..],
        ),
        (
            "constantpattern",
            "pub const DENIED: u8 = 0;\npub fn run(x: u8) {\n    if let DENIED = x {}\n}\n",
            "crate::clock::DENIED",
            &["crate::clock::DENIED in crate::clock"][..],
        ),
        (
            "typeonlyshadow",
            "pub fn now() {}\n#[allow(non_camel_case_types)]\npub fn run() {\n    struct now {}\n    let g: fn() = now;\n    g();\n}\n",
            "crate::clock::now",
            &["crate::clock::now in crate::clock"][..],
        ),
        (
            "valueshadow",
            "pub fn now() {}\npub fn run() {\n    fn now() {}\n    let g: fn() = now;\n    g();\n}\n",
            "crate::clock::now",
            &[][..],
        ),
        (
            "mutreference",
            "pub struct Clock;\npub fn run() {\n    let _ = &mut Clock;\n}\n",
            "crate::clock::Clock",
            &["crate::clock::Clock in crate::clock"][..],
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[("src/lib.rs", "pub mod clock;\n"), ("src/clock.rs", clock)],
        );
        let law = Constitution::new("single-mention").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::clock")
                .must_not_call_inline(prefix)
                .strict_prefix_only()
                .because("the prefix is not mentioned"),
        );
        let got: Vec<String> = match check(&law, probe.manifest()) {
            Outcome::Violations(report) => report
                .violations
                .iter()
                .map(|v| v.finding.clone())
                .collect(),
            Outcome::Clean(_) => Vec::new(),
            other => panic!("{package}: {other:?}"),
        };
        assert_eq!(got, found, "{package}");
    }
}

/// Whitespace is Unicode's `Pattern_White_Space`, as the Reference states: a vertical tab, which
/// `u8::is_ascii_whitespace` leaves out, and five characters past ASCII separate tokens as a space does, so
/// `use crate::forbidden::{\u{b}Thing\u{b}};` imports `crate::forbidden::Thing` and the name is read without either.
/// rustc 1.96.0, edition 2021, compiles each row.
#[test]
fn every_pattern_white_space_character_separates_tokens() {
    for (i, space) in [
        '\t', '\n', '\u{b}', '\u{c}', '\r', ' ', '\u{85}', '\u{200e}', '\u{200f}', '\u{2028}',
        '\u{2029}',
    ]
    .into_iter()
    .enumerate()
    {
        let package = format!("whitespace{i}");
        let probe = RootProbe::new(
            &package,
            "",
            &[
                (
                    "src/lib.rs",
                    "pub mod forbidden { pub struct Thing; }\npub mod m;\n",
                ),
                (
                    "src/m.rs",
                    &format!(
                        "#[allow(unused_imports)]\nuse crate::forbidden::{{{space}Thing{space}}};\n"
                    ),
                ),
            ],
        );
        let law = Constitution::new("whitespace").boundary(
            ModuleBoundary::in_crate(&package)
                .module("crate::m")
                .must_not_import("crate::forbidden")
                .because("m does not reach forbidden"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert_eq!(
                report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect::<Vec<_>>(),
                ["crate::forbidden::Thing"],
                "{space:?}: {report:?}"
            ),
            other => panic!("{space:?}: expected the import to react, got {other:?}"),
        }
    }
}

/// A use tree holding a token no path segment is, or a path that ends in `::`, is refused rather than read with the
/// leaf dropped: rustc refuses both, and a dropped leaf is an import no rule sees.
#[test]
fn a_use_tree_holding_what_no_path_is_is_refused() {
    for (package, use_line, refusal) in [
        (
            "usetreetoken",
            "use crate::forbidden::{1};",
            "holding `1` where a path segment stands",
        ),
        (
            "usetreeendscolons",
            "use crate::forbidden::;",
            "whose path ends in `::`",
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                (
                    "src/lib.rs",
                    "pub mod forbidden { pub struct Thing; }\npub mod m;\n",
                ),
                ("src/m.rs", &format!("{use_line}\n")),
            ],
        );
        let law = Constitution::new("use-tree").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::m")
                .must_not_import("crate::forbidden")
                .because("m does not reach forbidden"),
        );
        match check(&law, probe.manifest()) {
            Outcome::ConstitutionError(message) => {
                assert!(message.contains(refusal), "{package}: {message}")
            }
            other => panic!("{package}: expected a refusal, got {other:?}"),
        }
    }
}

/// A macro's group is read conservatively under `.strict_prefix_only()`, and a `use` statement it holds is judged as
/// the `use` path it is: `id! { use crate::{clock::now}; }` in `crate::core` mentions `crate::clock::now`, which its
/// tokens read as an expression would not. A `$crate` head in a `macro_rules!` body reads as `crate`, grouped or not:
/// `use $crate::{clock::now};` mentions `crate::clock::now` as `use $crate::clock::now;` does. rustc 1.96.0, edition
/// 2021, builds each.
#[test]
fn a_use_in_a_macros_group_is_judged_as_a_use_path_under_strict_prefix_only() {
    for (package, files, expected) in [
        (
            "strictmacrouse",
            &[
                (
                    "src/lib.rs",
                    "pub mod clock { pub fn now() {} }\npub mod core;\n",
                ),
                (
                    "src/core.rs",
                    "macro_rules! id { ($($t:tt)*) => { $($t)* }; }\nid! { #[allow(unused_imports)] use crate::{clock::now}; }\n",
                ),
            ][..],
            "crate::clock::now in crate::core",
        ),
        (
            "strictmacrodollarcrate",
            &[(
                "src/lib.rs",
                "pub mod clock { pub fn now() {} }\n#[macro_export]\nmacro_rules! m { () => { #[allow(unused_imports)] use $crate::clock::now; }; }\npub mod core { crate::m!(); }\n",
            )][..],
            "crate::clock::now in crate",
        ),
        (
            "strictmacrodollargroup",
            &[
                (
                    "src/lib.rs",
                    "pub mod clock { pub fn now() {} }\npub mod core;\n",
                ),
                (
                    "src/core.rs",
                    "macro_rules! m { () => { #[allow(unused_imports)] use $crate::{clock::now}; }; }\npub fn f() { m!(); }\n",
                ),
            ][..],
            "crate::clock::now in crate::core",
        ),
    ] {
        let probe = RootProbe::new(package, "", files);
        let law = Constitution::new("strict-macro-use").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate")
                .must_not_call_inline("crate::clock")
                .strict_prefix_only()
                .depth(xuanji::ScanDepth::Subtree)
                .because("no mention of the clock"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert!(
                report.violations.iter().any(|v| v.finding == expected),
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected {expected:?}, got {other:?}"),
        }
    }
}

/// A `use` a block inside a macro's group holds binds the paths beside it, as it binds them in the expansion: under
/// the default confinement `macro_rules! m { () => { use crate::clock::{self}; clock::now(); }; }` invoked in
/// `crate::core` names `crate::clock::now`, and so does a glob there; a `use` a module body inside a macro's group
/// holds binds that module's paths, so `id! { mod m { use crate::clock::{self}; pub fn f() { clock::now(); } } }` names
/// it too; and under `.strict_prefix_only()` a glob a macro's group holds outside any block is judged as the glob it
/// is. rustc 1.96.0, edition 2021, builds each row.
#[test]
fn a_use_in_a_macros_group_binds_and_globs_as_written() {
    let lib = "pub mod clock { pub fn now() {} }\npub mod core;\n";
    for (package, core, strict, expected) in [
        (
            "macroblockself",
            "macro_rules! m { () => { use crate::clock::{self}; clock::now(); }; }\npub fn f() { m!(); }\n",
            false,
            "crate::clock::now in crate::core",
        ),
        (
            "macroblockglob",
            "macro_rules! m { () => { use crate::clock::*; now(); }; }\npub fn f() { m!(); }\n",
            false,
            "glob crate::clock in crate::core",
        ),
        (
            "macromoduleself",
            "macro_rules! id { ($($t:tt)*) => { $($t)* }; }\nid! { mod m { use crate::clock::{self}; pub fn f() { clock::now(); } } }\n",
            false,
            "crate::clock::now in crate::core",
        ),
        (
            "macrogroupglob",
            "macro_rules! id { ($($t:tt)*) => { $($t)* }; }\nid! { #[allow(unused_imports)] use crate::clock::*; }\n",
            true,
            "glob crate::clock in crate::core",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", lib), ("src/core.rs", core)]);
        let boundary = ModuleBoundary::in_crate(package)
            .module("crate::core")
            .must_not_call_inline("crate::clock");
        let boundary = if strict {
            boundary.strict_prefix_only()
        } else {
            boundary
        };
        let law = Constitution::new("macro-use").boundary(boundary.because("no clock in the core"));
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert!(
                report.violations.iter().any(|v| v.finding == expected),
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected {expected:?}, got {other:?}"),
        }
    }
}

/// A `use` written directly in a macro's group, outside any block it holds, binds nothing: where the group expands
/// it is not read, so `id! { use crate::clock::{self}; pub fn f() { clock::now(); } }` is not claimed observed under
/// the default confinement — a stated bound — while the same `use` judged as a path under `.strict_prefix_only()`
/// still reacts. rustc 1.96.0, edition 2021, builds the row.
#[test]
fn a_use_written_in_a_macro_group_outside_any_block_binds_nothing() {
    let probe = RootProbe::new(
        "macrogroupuse",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod clock { pub fn now() {} }\npub mod core;\n",
            ),
            (
                "src/core.rs",
                "macro_rules! id { ($($t:tt)*) => { $($t)* }; }\nid! { use crate::clock::{self}; pub fn f() { clock::now(); } }\n",
            ),
        ],
    );
    let law = Constitution::new("macro-group-use").boundary(
        ModuleBoundary::in_crate("macrogroupuse")
            .module("crate::core")
            .must_not_call_inline("crate::clock")
            .because("no clock in the core"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Clean(_) => {}
        other => panic!("a stated bound: the call is not claimed observed, got {other:?}"),
    }
}

/// rustc compares identifiers in NFC, so a module declared `s` + U+00E9 + `cret` and named `se` + U+0301 + `cret` are
/// one module: a call or an import written in one composition is judged under a prefix or a module path written in
/// the other, and reported in NFC. rustc 1.96.0, edition 2021, builds each row; a file-form `mod` with a name past
/// ASCII it refuses (E0754), so the module is inline.
#[test]
fn an_identifier_is_one_name_in_either_composition() {
    let precomposed = "s\u{e9}cret";
    let decomposed = "se\u{301}cret";
    for (package, declared, written, prefix) in [
        ("nfccall", precomposed, decomposed, precomposed),
        ("nfcprefix", precomposed, precomposed, decomposed),
        ("nfcdecl", decomposed, precomposed, precomposed),
    ] {
        let lib = format!("pub mod {declared} {{ pub fn go() {{}} }}\npub mod core;\n");
        let core = format!("pub fn f() {{ crate::{written}::go(); }}\n");
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib), ("src/core.rs", &core)]);
        let law = Constitution::new("nfc").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::core")
                .must_not_call_inline(&format!("crate::{prefix}"))
                .because("no secret in the core"),
        );
        let expected = format!("crate::{precomposed}::go in crate::core");
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert!(
                report.violations.iter().any(|v| v.finding == expected),
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected {expected:?}, got {other:?}"),
        }
    }
    let probe = RootProbe::new(
        "nfcimport",
        "",
        &[
            (
                "src/lib.rs",
                &format!("pub mod {precomposed} {{ pub fn go() {{}} }}\npub mod core;\n"),
            ),
            (
                "src/core.rs",
                &format!("#[allow(unused_imports)]\nuse crate::{decomposed}::go;\n"),
            ),
        ],
    );
    let law = Constitution::new("nfc-import").boundary(
        ModuleBoundary::in_crate("nfcimport")
            .module("crate::core")
            .must_not_import(&format!("crate::{precomposed}"))
            .because("no secret in the core"),
    );
    assert!(
        matches!(check(&law, probe.manifest()), Outcome::Violations(_)),
        "an import written decomposed is judged under the module declared precomposed"
    );
}

/// An attribute's contents are its macro's input, so a `use` written inside one is no import: rustc 1.96.0, edition
/// 2021, compiles `#[cfg_attr(any(), my_attr(use crate::forbidden::Thing;))] pub fn f() {}` importing nothing. The
/// `use` an attribute is written on is read as ever.
#[test]
fn a_use_inside_an_attribute_is_no_import() {
    let law = |package: &str| {
        Constitution::new("attribute-use").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::m")
                .must_not_import("crate::forbidden")
                .because("m does not reach forbidden"),
        )
    };
    let inside = RootProbe::new(
        "attributeuse",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod forbidden { pub struct Thing; }\npub mod m;\n",
            ),
            (
                "src/m.rs",
                "#[cfg_attr(any(), my_attr(use crate::forbidden::Thing;))]\npub fn f() {}\n",
            ),
        ],
    );
    let outcome = check(&law("attributeuse"), inside.manifest());
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
    let on = RootProbe::new(
        "attributeonuse",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod forbidden { pub struct Thing; }\npub mod m;\n",
            ),
            (
                "src/m.rs",
                "#[allow(unused_imports)]\nuse crate::forbidden::Thing;\n",
            ),
        ],
    );
    let outcome = check(&law("attributeonuse"), on.manifest());
    assert_eq!(outcome.exit_code(), 1, "{outcome:?}");
}

/// `crate::m0` defines `X`, each of a hundred `crate::m{i}` re-exports the one before, and nothing calls
/// through the chain: its `pub use` paths are mentions, which only `.strict_prefix_only()` judges. So the chain is
/// read, and refused past the cap, only there.
#[test]
fn an_unused_reexport_chain_past_the_cap_is_judged_only_where_a_mention_is() {
    let mut lib = String::from("pub mod m0 { pub struct X; impl X { pub fn f() {} } }\n");
    for i in 1..=100 {
        lib.push_str(&format!(
            "pub mod m{i} {{ pub use crate::m{}::X; }}\n",
            i - 1
        ));
    }
    lib.push_str("pub fn g() {}\n");
    let probe = lib_probe("unusedreexportchain", &lib);
    assert_inline_answers(
        &probe,
        "unusedreexportchain",
        "crate",
        "crate::m0",
        &[],
        &[],
    );
    let law = Constitution::new("mentions").boundary(
        ModuleBoundary::in_crate("unusedreexportchain")
            .module("crate")
            .must_not_call_inline("crate::m0")
            .strict_prefix_only()
            .because("inline path resolution"),
    );
    match check(&law, probe.manifest()) {
        Outcome::ConstitutionError(message) => assert!(
            message.contains("64") && message.contains("use crate::m"),
            "{message}"
        ),
        other => panic!("a mention walking the chain past the cap is refused: {other:?}"),
    }
}

/// A `type` alias's target is resolved as a `use` path is: `type T = md5x::W;` with no local `md5x` names the
/// dependency, as `use md5x::W as T;` does, so a call through `T` reports under a `md5x` prefix in either mode.
#[test]
fn a_type_alias_to_an_unused_dependency_names_the_dependency() {
    let probe = RootProbe::new(
        "aliasdependency",
        "[dependencies]\nmd5x = { package = \"dep_md5x\", path = \"dep_md5x\" }\n",
        &[
            (
                "dep_md5x/Cargo.toml",
                "[package]\nname = \"dep_md5x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
            ),
            (
                "dep_md5x/src/lib.rs",
                "pub struct W;\nimpl W { pub fn f() {} }\n",
            ),
            ("src/lib.rs", "type T = md5x::W;\npub fn g() { T::f(); }\n"),
        ],
    );
    let found = ["md5x::W::f in crate"];
    assert_inline_answers(&probe, "aliasdependency", "crate", "md5x", &found, &found);
}

/// A package whose `src/lib.rs` and files are `files`, in `edition`, with a path dependency for each of `deps`.
fn files_probe(package: &str, edition: &str, files: &[(&str, &str)], deps: &[&str]) -> RootProbe {
    let mut probe = RootProbe::with_edition(package, edition, "", files);
    for dep in deps {
        probe = probe.with_path_dependency(
            dep,
            "pub struct B;\npub fn helper() {}\npub fn compute() -> u32 { 0 }\n",
        );
    }
    probe
}

/// A package whose crate root declares the file module `m`, holding `m`, beside `pub fn g() {}`.
fn file_module_m_probe(package: &str, m: &str) -> RootProbe {
    files_probe(
        package,
        "2021",
        &[
            ("src/lib.rs", "pub mod m;\npub fn g() {}\n"),
            ("src/m.rs", m),
        ],
        &[],
    )
}

/// The findings under `prefix` over [`file_module_m_probe`]'s package, governed from `crate`, in both modes.
fn file_module_answers(package: &str, m: &str, prefix: &str, found: &[&str]) {
    let probe = file_module_m_probe(package, m);
    assert_inline_answers(&probe, package, "crate", prefix, found, found);
}

/// A head no scope binds names no item of the module it stands in: the scope table records every item a module
/// declares, so what it does not find there is an attribute's name, a prelude name, a parameter or a keyword —
/// never `crate::m::…`. Attributes, a derive, a `Fn` bound, a parameter called, a `fn` pointer type, and tuple
/// declarations of an enum variant and a struct.
#[test]
fn an_unbound_bare_head_names_no_item_of_its_module() {
    file_module_answers(
        "unboundattrs",
        "#![allow(dead_code)]\n#[cfg(any(unix, not(windows)))]\npub fn h() {}\n#[derive(Clone, Debug)]\npub struct S;\npub enum E { A(u8) }\npub struct P(pub u8);\npub fn k<F: Fn(u8) -> u8>(f: F) -> u8 { f(1) }\npub type Cb = fn(u8) -> u8;\n",
        "crate::m",
        &[],
    );
}

/// Prelude names called bare — `Ok`, `Some`, `drop` — name nothing in the module that calls them.
#[test]
fn a_prelude_name_called_bare_names_no_item_of_its_module() {
    file_module_answers(
        "unboundprelude",
        "pub fn h(x: u8) -> Result<Option<u8>, ()> { let v = vec![x]; drop(v); Ok(Some(x)) }\n",
        "crate::m",
        &[],
    );
}

/// The control: the same source in another module, `crate::api`, governed under a prefix naming that module, reports
/// nothing either — so a misread naming an item of the module a bare head stands in, `crate::api::drop`, would fall
/// under the prefix and report.
#[test]
fn prelude_names_outside_the_prefixs_module_report_nothing() {
    let probe = files_probe(
        "unboundpreludeapi",
        "2021",
        &[
            ("src/lib.rs", "pub mod m;\npub mod api;\npub fn g() {}\n"),
            ("src/m.rs", "pub fn k() {}\n"),
            (
                "src/api.rs",
                "pub fn h(x: u8) -> Result<Option<u8>, ()> { let v = vec![x]; drop(v); Ok(Some(x)) }\n",
            ),
        ],
        &[],
    );
    assert_inline_answers(
        &probe,
        "unboundpreludeapi",
        "crate::api",
        "crate::api",
        &[],
        &[],
    );
}

/// An attribute's name and a `cfg` predicate are not code: `#[cfg(unix)]` beside `pub fn cfg() {}` calls nothing,
/// although its head names that function, and the attribute `cfg_attr(any(), serde(default))` applies names no
/// `serde` call beside a `serde` dependency under strict-external.
#[test]
fn an_attributes_name_and_predicate_hold_no_call() {
    file_module_answers(
        "attributecfg",
        "pub fn cfg() {}\n#[cfg(unix)]\npub fn h() {}\n",
        "crate::m::cfg",
        &[],
    );
    let probe = files_probe(
        "attributeserde",
        "2021",
        &[(
            "src/lib.rs",
            "#[allow(unknown_lints)]\n#[cfg_attr(any(), serde(default))]\npub struct S;\n",
        )],
        &["serde"],
    );
    assert_inline_answers(&probe, "attributeserde", "crate", "serde", &[], &[]);
}

/// A tuple struct's or tuple variant's declaration defines its constructor rather than calling it: `pub struct
/// P(pub u8);` and `pub enum E { A(u8) }` beside `pub use self::E::A;`, which binds the variant's name in the module.
#[test]
fn a_tuple_struct_or_variant_declaration_is_not_a_call() {
    const DECL: &str = "pub struct P(pub u8);\npub enum E { A(u8) }\npub use self::E::A;\n";
    file_module_answers("tupledeclp", DECL, "crate::m::P", &[]);
    file_module_answers("tupledecle", DECL, "crate::m::E", &[]);
}

/// The control: constructing them, `P(1)` and `E::A(1)`, is a call.
#[test]
fn a_tuple_struct_or_variant_construction_is_a_call() {
    const NEW: &str = "pub struct P(pub u8);\npub enum E { A(u8) }\npub use self::E::A;\npub fn mk() -> P { P(1) }\npub fn mk2() -> E { E::A(1) }\n";
    file_module_answers(
        "tuplenewp",
        NEW,
        "crate::m::P",
        &["crate::m::P in crate::m"],
    );
    file_module_answers(
        "tuplenewe",
        NEW,
        "crate::m::E",
        &["crate::m::E::A in crate::m"],
    );
}

/// A keyword is never a path head, even beside an item whose raw name spells it: `match (x)` beside `pub fn
/// r#match()` calls nothing. In edition 2015 `try` is an identifier, so `try()` there calls the local function.
#[test]
fn a_keyword_is_never_a_path_head() {
    file_module_answers(
        "keywordhead",
        "pub fn r#match() {}\npub fn h(x: u8) -> u8 { match (x) { _ => 0 } }\n",
        "crate::m",
        &[],
    );
    let probe = files_probe(
        "keywordhead2015",
        "2015",
        &[
            ("src/lib.rs", "pub mod m;\n"),
            (
                "src/m.rs",
                "pub fn try() -> u8 { 0 }\npub fn h() -> u8 { try() }\n",
            ),
        ],
        &[],
    );
    let found = ["crate::m::try in crate::m"];
    assert_inline_answers(
        &probe,
        "keywordhead2015",
        "crate",
        "crate::m",
        &found,
        &found,
    );
}

/// An item a macro generates is not in the scope table, so a bare call of it in its own module names nothing — a
/// declared bound.
#[test]
fn a_macro_generated_item_called_bare_in_its_module_is_a_bound() {
    file_module_answers(
        "macrogenbare",
        "macro_rules! make { () => { pub fn gen() -> u8 { 0 } }; }\nmake!();\npub fn h() -> u8 { gen() }\n",
        "crate::m",
        &[],
    );
}

/// A path naming the macro-generated item from another module is crate-rooted and still reports.
#[test]
fn a_crate_rooted_call_of_a_macro_generated_item_reports() {
    let probe = files_probe(
        "macrogenrooted",
        "2021",
        &[
            (
                "src/lib.rs",
                "pub mod m;\npub fn g() -> u8 { crate::m::gen() }\n",
            ),
            (
                "src/m.rs",
                "macro_rules! make { () => { pub fn gen() -> u8 { 0 } }; }\nmake!();\n",
            ),
        ],
        &[],
    );
    let found = ["crate::m::gen in crate"];
    assert_inline_answers(
        &probe,
        "macrogenrooted",
        "crate",
        "crate::m",
        &found,
        &found,
    );
}

/// A prelude name called bare is not read as its standard-library path: `drop(x)` is not observed under
/// `std::mem` — a declared bound.
#[test]
fn a_prelude_name_called_bare_is_not_read_as_its_std_path() {
    file_module_answers(
        "preludedrop",
        "pub fn h(x: Vec<u8>) { drop(x); }\n",
        "std::mem",
        &[],
    );
}

/// A scope's candidates for a name are all its bindings and the item it declares: with `pub use crate::a::X` under
/// one cfg and `pub struct X` under the other, `m::X::f()` reports under `crate::m::X` and under `crate::a`, in
/// either order.
#[test]
fn a_declared_item_and_a_binding_of_one_name_are_both_candidates() {
    const A: &str = "pub mod a { pub struct X; impl X { pub fn f() {} } }\n";
    for (package, m) in [
        (
            "itembinding",
            "pub mod m { #[cfg(unix)] pub use crate::a::X; #[cfg(not(unix))] pub struct X; #[cfg(not(unix))] impl X { pub fn f() {} } }\n",
        ),
        (
            "itembindingrev",
            "pub mod m { #[cfg(not(unix))] pub use crate::a::X; #[cfg(unix)] pub struct X; #[cfg(unix)] impl X { pub fn f() {} } }\n",
        ),
    ] {
        let lib = format!("{A}{m}pub fn g() {{ m::X::f(); }}\n");
        assert_crate_answers(package, &lib, "crate::m::X", &["crate::m::X::f in crate"]);
        assert_crate_answers(package, &lib, "crate::a", &["crate::a::X::f in crate"]);
    }
}

/// A glob brings each binding only where that binding's visibility reaches: `use s::*;` at the crate root sees
/// `s`'s `pub use crate::a::X`, not its private `use crate::b::X`.
#[test]
fn a_glob_brings_each_binding_only_where_it_is_visible() {
    let lib = "pub mod a { pub struct X; impl X { pub fn f() {} } }\npub mod b { pub struct X; impl X { pub fn f() {} } }\npub mod s { #[cfg(unix)] pub use crate::a::X; #[cfg(not(unix))] #[allow(unused_imports)] use crate::b::X; }\nuse s::*;\npub fn g() { X::f(); }\n";
    assert_crate_answers("globvisiblebinding", lib, "crate::b", &[]);
    assert_crate_answers(
        "globvisiblebinding",
        lib,
        "crate::a",
        &["crate::a::X::f in crate", "glob crate::s in crate"],
    );
}

/// A local `mod std` is a local name like any other, so `std::process::id()` beside it calls the local function.
#[test]
fn a_local_std_module_shadows_the_sysroot() {
    assert_crate_answers(
        "localstd",
        "pub mod std { pub mod process { pub fn id() -> u32 { 0 } } }\npub fn g() { let _ = std::process::id(); }\n",
        "std::process",
        &[],
    );
}

/// A name a glob brings shadows a dependency of that name, so under strict-external `md5x::compute()` beside
/// `use local::*;` bringing a local `md5x` calls the local module.
#[test]
fn a_glob_brought_name_shadows_a_same_named_dependency() {
    let probe = files_probe(
        "globshadowsdep",
        "2021",
        &[(
            "src/lib.rs",
            "pub mod local { pub mod md5x { pub fn compute() -> u32 { 1 } } }\nuse local::*;\npub fn g() -> u32 { md5x::compute() }\n",
        )],
        &["md5x"],
    );
    assert_inline_answers(&probe, "globshadowsdep", "crate", "md5x", &[], &[]);
}

/// Every kind of literal is an operand, so a `<` after it compares: each row is a literal and the source that puts
/// it before `< y && n > ::std::process::id()`, whose call reports under `std::process` in either mode.
#[test]
fn a_literal_before_a_comparison_is_an_operand() {
    let rows: [(&str, &str); 10] = [
        (
            "\"x\"",
            "pub fn g(y: &str, n: u32) -> bool { \"x\" < y && n > ::std::process::id() }\n",
        ),
        (
            "r\"x\"",
            "pub fn g(y: &str, n: u32) -> bool { r\"x\" < y && n > ::std::process::id() }\n",
        ),
        (
            "r#\"x\"#",
            "pub fn g(y: &str, n: u32) -> bool { r#\"x\"# < y && n > ::std::process::id() }\n",
        ),
        (
            "r##\"x\"##",
            "pub fn g(y: &str, n: u32) -> bool { r##\"x\"## < y && n > ::std::process::id() }\n",
        ),
        (
            "b\"x\"",
            "pub fn g(y: &[u8; 1], n: u32) -> bool { b\"x\" < y && n > ::std::process::id() }\n",
        ),
        (
            "br#\"x\"#",
            "pub fn g(y: &[u8; 1], n: u32) -> bool { br#\"x\"# < y && n > ::std::process::id() }\n",
        ),
        (
            "c\"x\"",
            "pub fn g(y: &core::ffi::CStr, n: u32) -> bool { c\"x\" < y && n > ::std::process::id() }\n",
        ),
        (
            "'a'",
            "pub fn g(y: char, n: u32) -> bool { 'a' < y && n > ::std::process::id() }\n",
        ),
        (
            "'\\''",
            "pub fn g(y: char, n: u32) -> bool { '\\'' < y && n > ::std::process::id() }\n",
        ),
        (
            "b'a'",
            "pub fn g(y: u8, n: u32) -> bool { b'a' < y && n > ::std::process::id() }\n",
        ),
    ];
    let mut mismatches = Vec::new();
    for (i, (literal, lib)) in rows.iter().enumerate() {
        let package = format!("literaloperand{i}");
        let probe = lib_probe(&package, lib);
        for strict_external in [false, true] {
            let got = inline_findings(&probe, &package, "crate", "std::process", strict_external);
            if got != ["std::process::id in crate"] {
                mismatches.push(format!(
                    "{literal}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Controls beside the literal table, each an answer the scanner already gives: a suffixed number, a method call on a
/// byte string, a `?`, and a lifetime before a byte literal all end an operand, so the rooted call reports; and a
/// `<W>` after a block or after a statement ending in a literal still opens a qualified path.
#[test]
fn operands_and_qualified_paths_beside_literals_keep_their_answers() {
    for (package, lib) in [
        (
            "y1cnum",
            "pub fn g(n: u32) -> bool { 0u32 < n && n > ::std::process::id() }\n",
        ),
        (
            "y1cbslice",
            "pub fn g(y: &[u8], n: u32) -> bool { b\"x\".as_slice() < y && n > ::std::process::id() }\n",
        ),
        (
            "y1ctry",
            "pub fn g(r: Result<u32, ()>, n: u32) -> Result<bool, ()> { Ok(r? < 5 && n > ::std::process::id()) }\n",
        ),
        (
            "y1clife",
            "pub struct L<'a>(pub &'a u8);\nimpl<'a> L<'a> { pub fn f(&self, n: u32) -> bool { *self.0 < b'c' && n > ::std::process::id() } }\n",
        ),
    ] {
        assert_crate_answers(package, lib, "std::process", &["std::process::id in crate"]);
    }
    let probe = files_probe(
        "y1cqual",
        "2021",
        &[(
            "src/lib.rs",
            "pub struct W;\nimpl W { pub fn md5x() {} }\npub fn h() { let _s = \"a\"; <W>::md5x(); }\n",
        )],
        &["md5x"],
    );
    assert_inline_answers(&probe, "y1cqual", "crate", "md5x", &[], &[]);
}

/// The head every pattern-position fixture's `src/m.rs` starts with.
const PATTERN_TYPES: &str =
    "pub struct P(pub u8);\npub enum E { A(u8), B }\npub struct S { pub a: P }\n";

/// A file module `m` holding [`PATTERN_TYPES`] and `body`, governed from `crate`: the findings under `prefix`, in both
/// modes, collected per fixture into `mismatches`.
fn pattern_answers(
    package: &str,
    body: &str,
    prefix: &str,
    found: &[&str],
    mismatches: &mut Vec<String>,
) {
    let probe = file_module_m_probe(package, &format!("{PATTERN_TYPES}{body}"));
    for strict_external in [false, true] {
        let got = inline_findings(&probe, package, "crate", prefix, strict_external);
        if got != found {
            mismatches.push(format!(
                "{package}, strict_external = {strict_external}: {got:?}"
            ));
        }
    }
}

/// Controls beside the pattern positions: a construction in a block arm's body, in a closure's body, and after a
/// bitwise `|` — whose `|` opens no closure — is a call, and reports once.
#[test]
fn a_construction_beside_a_pattern_position_is_a_call() {
    let mut mismatches = Vec::new();
    pattern_answers(
        "ptblockarm",
        "pub fn f(e: E) -> u8 { match e { E::A(x) => { x } E::B => { P(1).0 } } }\n",
        "crate::m::P",
        &["crate::m::P in crate::m"],
        &mut mismatches,
    );
    pattern_answers(
        "pcclosurebody",
        "pub fn f() -> P { let c = |x: u8| P(x); c(1) }\n",
        "crate::m::P",
        &["crate::m::P in crate::m"],
        &mut mismatches,
    );
    pattern_answers(
        "pcbitwise2",
        "pub fn f(a: u8) -> u8 { a | P(1).0 }\n",
        "crate::m::P",
        &["crate::m::P in crate::m"],
        &mut mismatches,
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A path's role is read from the tokens beside it, and a tuple-struct or tuple-variant pattern is written as a call is
/// written, so every pattern position — a `let`, a let-else, `if let`, `while let`, a `for` loop, a match arm, a struct
/// pattern's field, a `fn`, closure or `move` closure parameter, a macro's arguments and a destructuring assignment's
/// left side — is read as a call, in either mode: a declared over-reaction.
#[test]
fn a_path_in_a_pattern_position_is_read_as_a_call() {
    let mut mismatches = Vec::new();
    for (package, body, prefix, found) in [
        (
            "ptlet",
            "pub fn f(p: P) -> u8 { let P(x) = p; x }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptletelse",
            "pub fn f(e: E) -> u8 { let E::A(x) = e else { return 0 }; x }\n",
            "crate::m::E",
            "crate::m::E::A in crate::m",
        ),
        (
            "ptiflet",
            "pub fn f(e: E) -> u8 { if let E::A(x) = e { x } else { 0 } }\n",
            "crate::m::E",
            "crate::m::E::A in crate::m",
        ),
        (
            "ptwhilelet",
            "pub fn f(mut v: Vec<E>) -> u8 { while let Some(E::A(x)) = v.pop() { return x; } 0 }\n",
            "crate::m::E",
            "crate::m::E::A in crate::m",
        ),
        (
            "ptfor",
            "pub fn f(v: Vec<P>) -> u8 { let mut s = 0; for P(x) in v { s += x; } s }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptmatcharm",
            "pub fn f(e: E) -> u8 { match e { E::A(x) if x > 0 => x, _ => 0 } }\n",
            "crate::m::E",
            "crate::m::E::A in crate::m",
        ),
        (
            "ptstructfield",
            "pub fn f(s: S) -> u8 { match s { S { a: P(x) } => x } }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptfnparam",
            "pub fn f(P(x): P) -> u8 { x }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptclosureparam",
            "pub fn f(p: P) -> u8 { let c = |P(x): P| x; c(p) }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptmoveclosure",
            "pub fn f(p: P) -> u8 { let c = move |P(x): P| x; c(p) }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
        (
            "ptmacroarg",
            "pub fn f(e: &E) -> bool { matches!(e, E::A(_)) }\n",
            "crate::m::E",
            "crate::m::E::A in crate::m",
        ),
        (
            "ptassign",
            "pub fn f(p: P) -> u8 { let x; P(x) = p; x }\n",
            "crate::m::P",
            "crate::m::P in crate::m",
        ),
    ] {
        pattern_answers(package, body, prefix, &[found], &mut mismatches);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Every multi-byte operator is one token: each row puts an operator of the Reference's punctuation table between two
/// operands — before a rooted call, as a compound assignment of one, or between a `<` and a `>` it must not pair
/// — and the call reports under `std::process` in either mode.
#[test]
fn a_multi_byte_operator_is_one_token() {
    let rows: [(&str, &str); 30] = [
        (
            "&&",
            "pub fn g(a: bool) -> bool { a && ::std::process::id() > 0 }\n",
        ),
        (
            "||",
            "pub fn g(a: bool) -> bool { a || ::std::process::id() > 0 }\n",
        ),
        (
            "<<",
            "pub fn g(a: u32) -> u32 { a << ::std::process::id() }\n",
        ),
        (
            ">>",
            "pub fn g(a: u32) -> u32 { a >> ::std::process::id() }\n",
        ),
        (
            "==",
            "pub fn g(a: u32) -> bool { a == ::std::process::id() }\n",
        ),
        (
            "!=",
            "pub fn g(a: u32) -> bool { a != ::std::process::id() }\n",
        ),
        (
            ">=",
            "pub fn g(a: u32) -> bool { a >= ::std::process::id() }\n",
        ),
        (
            "<=",
            "pub fn g(a: u32) -> bool { a <= ::std::process::id() }\n",
        ),
        (
            "..",
            "pub fn g(a: u32) -> core::ops::Range<u32> { a .. ::std::process::id() }\n",
        ),
        (
            "..=",
            "pub fn g(a: u32) -> core::ops::RangeInclusive<u32> { a ..= ::std::process::id() }\n",
        ),
        (
            "+=",
            "pub fn g(mut a: u32) -> u32 { a += ::std::process::id(); a }\n",
        ),
        (
            "-=",
            "pub fn g(mut a: u32) -> u32 { a -= ::std::process::id(); a }\n",
        ),
        (
            "*=",
            "pub fn g(mut a: u32) -> u32 { a *= ::std::process::id(); a }\n",
        ),
        (
            "/=",
            "pub fn g(mut a: u32) -> u32 { a /= ::std::process::id(); a }\n",
        ),
        (
            "%=",
            "pub fn g(mut a: u32) -> u32 { a %= ::std::process::id(); a }\n",
        ),
        (
            "^=",
            "pub fn g(mut a: u32) -> u32 { a ^= ::std::process::id(); a }\n",
        ),
        (
            "&=",
            "pub fn g(mut a: u32) -> u32 { a &= ::std::process::id(); a }\n",
        ),
        (
            "|=",
            "pub fn g(mut a: u32) -> u32 { a |= ::std::process::id(); a }\n",
        ),
        (
            "<<=",
            "pub fn g(mut a: u32) -> u32 { a <<= ::std::process::id(); a }\n",
        ),
        (
            ">>=",
            "pub fn g(mut a: u32) -> u32 { a >>= ::std::process::id(); a }\n",
        ),
        (
            "&&",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && d > ::std::process::id() }\n",
        ),
        (
            "||",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b || d > ::std::process::id() }\n",
        ),
        (
            "<<",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < (b << c) && d > ::std::process::id() }\n",
        ),
        (
            ">>",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < (b >> c) && d > ::std::process::id() }\n",
        ),
        (
            "==",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && b == c && d > ::std::process::id() }\n",
        ),
        (
            "!=",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && b != c && d > ::std::process::id() }\n",
        ),
        (
            ">=",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && b >= c && d > ::std::process::id() }\n",
        ),
        (
            "<=",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && b <= c && d > ::std::process::id() }\n",
        ),
        (
            "..",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && (b..c).contains(&d) && d > ::std::process::id() }\n",
        ),
        (
            "..=",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> bool { a < b && (b..=c).contains(&d) && d > ::std::process::id() }\n",
        ),
    ];
    let mut mismatches = Vec::new();
    for (i, (operator, lib)) in rows.iter().enumerate() {
        let package = format!("operatortoken{i}");
        let probe = lib_probe(&package, lib);
        for strict_external in [false, true] {
            let got = inline_findings(&probe, &package, "crate", "std::process", strict_external);
            if got != ["std::process::id in crate"] {
                mismatches.push(format!(
                    "`{operator}` in {lib:?}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn a_logical_or_before_a_call_reports() {
    assert_crate_answers(
        "orcall",
        "pub fn g(a: bool) -> bool { a || std::time::SystemTime::now() > std::time::UNIX_EPOCH }\n",
        "std::time",
        &["std::time::SystemTime::now in crate"],
    );
}

#[test]
fn a_logical_or_before_a_rooted_call_reports() {
    assert_crate_answers(
        "orrooted",
        "pub fn g(a: u64, b: u64) -> bool { a == b || ::std::time::SystemTime::now() > ::std::time::UNIX_EPOCH }\n",
        "std::time",
        &["std::time::SystemTime::now in crate"],
    );
}

#[test]
fn a_comparison_then_a_logical_or_before_a_rooted_call_reports() {
    assert_crate_answers(
        "comparethenorrooted",
        "pub fn g(a: u64, b: u64) -> bool { a < b || ::std::time::SystemTime::now() > ::std::time::UNIX_EPOCH }\n",
        "std::time",
        &["std::time::SystemTime::now in crate"],
    );
}

#[test]
fn a_logical_or_then_a_comparison_pair_reports() {
    assert_crate_answers(
        "orthencomparepair",
        "pub fn g(n: u32, b: bool) -> bool { b || n < 3 && n > ::std::process::id() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

/// A shift before a comparison with a rooted call reports the call, whether or not a later `>` at the same level
/// would close a `<<`-opened generic list: `&& n > 0`, a tuple's second element and an `as` cast each put one there.
#[test]
fn a_shift_before_a_comparison_with_a_rooted_call_reports() {
    for (package, body) in [
        (
            "shiftthencompare",
            "pub fn g(n: u32) -> bool { 1 << n > ::std::process::id() }\n",
        ),
        (
            "shiftcomparethen",
            "pub fn g(n: u32) -> bool { 1 << n > ::std::process::id() && n > 0 }\n",
        ),
        (
            "shiftcomparetuple",
            "pub fn g(a: u32, b: u32, c: u32, d: u32) -> (bool, bool) { (a << b > ::std::process::id(), c > d) }\n",
        ),
        (
            "shiftcomparecast",
            "pub fn g(a: u32, b: u8, c: u32, d: u32) -> bool { a << b as u32 > ::std::process::id() && c > d }\n",
        ),
    ] {
        assert_crate_answers(
            package,
            body,
            "std::process",
            &["std::process::id in crate"],
        );
    }
}

#[test]
fn a_bitwise_or_before_a_rooted_call_reports() {
    assert_crate_answers(
        "bitorrooted",
        "pub fn g(a: u32) -> u32 { a | ::std::process::id() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

#[test]
fn a_logical_and_before_a_rooted_call_reports() {
    assert_crate_answers(
        "androoted",
        "pub fn g(a: bool) -> bool { a && ::std::process::id() > 0 }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

#[test]
fn a_call_in_a_parameterless_closure_body_reports() {
    assert_crate_answers(
        "parameterlessclosure",
        "pub fn g() -> u32 { let c = || ::std::process::id(); c() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

#[test]
fn a_shift_right_before_a_rooted_call_reports() {
    assert_crate_answers(
        "shiftrightrooted",
        "pub fn g(n: u32) -> u32 { n >> ::std::process::id() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

#[test]
fn a_less_or_equal_then_a_comparison_pair_reports() {
    assert_crate_answers(
        "lessequalpair",
        "pub fn g(n: u32) -> bool { 0 <= n && n > ::std::process::id() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

#[test]
fn a_logical_or_in_a_let_initializer_reports() {
    file_module_answers(
        "orinlet",
        "pub fn check(_: u8) -> bool { true }\npub fn f(a: bool) -> bool { let b = a || check(1); b }\n",
        "crate::m::check",
        &["crate::m::check in crate::m"],
    );
}

#[test]
fn a_logical_or_in_a_closure_body_reports() {
    file_module_answers(
        "orinclosure",
        "pub fn check(_: u8) -> bool { true }\npub fn f() -> bool { let c = |a: bool| a || check(1); c(false) }\n",
        "crate::m::check",
        &["crate::m::check in crate::m"],
    );
}

#[test]
fn a_compound_bitwise_or_assignment_of_a_call_reports() {
    file_module_answers(
        "orassigncall",
        "pub fn check(_: u8) -> bool { true }\npub fn f(mut a: bool) -> bool { a |= check(1); a }\n",
        "crate::m::check",
        &["crate::m::check in crate::m"],
    );
}

/// One binding shape: a package holding [`TWO_XS`], `lib` beside it, and two file modules — `lexical`, which reaches
/// `X` through a name the shape binds, and `rooted`, which writes a path through the shape — each expected to report
/// the call under every prefix of `prefixes`.
struct BindingShape {
    package: &'static str,
    lib: &'static str,
    lexical: &'static str,
    rooted: &'static str,
    prefixes: &'static [&'static str],
    /// What the call names after the prefix: `X::f` for a call of the struct's associated `f`, `X` for a call of the
    /// function `crate::v::X`.
    tail: &'static str,
}

/// Every binding shape a head or a path segment can be named through reports its call whether the name is reached
/// lexically (`use …::X; X::f()`) or as a segment of a path (`crate::…::X::f()`), because both readings go through one
/// module-level lookup. A block-local module has no crate-rooted path, so its second cell is the path through it from
/// its own block. Each cell is its own file module, so a cell one reading misses is named by the table. The last two
/// rows give one name a type and a value from different sources — a binding in one namespace, a glob in the other —
/// and the call reaches the one its namespace names.
#[test]
fn every_binding_shape_resolves_a_lexical_head_and_a_path_segment_alike() {
    let shapes = [
        BindingShape {
            package: "shapenameduse",
            lib: "",
            lexical: "use crate::a::X;\npub fn g() { X::f(); }\n",
            rooted: "#[allow(unused_imports)]\nuse crate::a::X;\npub fn g() { crate::rooted::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapepubuse",
            lib: "pub mod support { pub use crate::a::X; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapeglob",
            lib: "pub mod support { pub use crate::a::*; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapealias",
            lib: "pub mod support { pub type X = crate::a::X; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapecfg",
            lib: "pub mod support { #[cfg(unix)] pub use crate::a::X; #[cfg(not(unix))] pub use crate::b::X; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a", "crate::b"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapeselfleaf",
            lib: "pub mod support { pub use crate::a::{self}; }\n",
            lexical: "use crate::support::a;\npub fn g() { a::X::f(); }\n",
            rooted: "pub fn g() { crate::support::a::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapeblockmod",
            lib: "",
            lexical: "pub fn g() { mod m { pub use crate::a::X; } use m::X; X::f(); }\n",
            rooted: "pub fn g() { mod m { pub use crate::a::X; } m::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapeexterncrate",
            lib: "extern crate self as me;\n",
            lexical: "use me::a::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::me::a::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapevalueandtype",
            lib: "pub mod support { #[allow(non_snake_case)] pub fn X() {} pub use crate::a::*; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
        BindingShape {
            package: "shapetypebindingvalueglob",
            lib: "pub mod v { #[allow(non_snake_case)] pub fn X() {} }\n\
                  pub mod support { pub type X = crate::a::X; pub use crate::v::*; }\n",
            lexical: "use crate::support::X;\npub fn g() { X(); }\n",
            rooted: "pub fn g() { crate::support::X(); }\n",
            prefixes: &["crate::v"],
            tail: "X",
        },
        BindingShape {
            package: "shapevaluebindingtypeglob",
            lib: "pub mod v { #[allow(non_snake_case)] pub fn X() {} }\n\
                  pub mod support { pub use crate::v::X; pub use crate::a::*; }\n",
            lexical: "use crate::support::X;\npub fn g() { X::f(); }\n",
            rooted: "pub fn g() { crate::support::X::f(); }\n",
            prefixes: &["crate::a"],
            tail: "X::f",
        },
    ];
    let mut mismatches = Vec::new();
    for shape in shapes {
        let lib = format!("{TWO_XS}{}pub mod lexical;\npub mod rooted;\n", shape.lib);
        let probe = files_probe(
            shape.package,
            "2021",
            &[
                ("src/lib.rs", &lib),
                ("src/lexical.rs", shape.lexical),
                ("src/rooted.rs", shape.rooted),
            ],
            &[],
        );
        for cell in ["crate::lexical", "crate::rooted"] {
            for prefix in shape.prefixes {
                let expected = [format!("{prefix}::{} in {cell}", shape.tail)];
                for strict_external in [false, true] {
                    let found =
                        inline_findings(&probe, shape.package, cell, prefix, strict_external);
                    if found != expected {
                        mismatches.push(format!(
                            "{} {cell} under {prefix}, strict_external = {strict_external}: {found:?}",
                            shape.package
                        ));
                    }
                }
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A path through a module reads what that module's glob brings, as a lexical head does: `use crate::support::X;
/// X::f()` in another file, the same `use` in the crate root, and `crate::support::X::f()` written out all call
/// `crate::a::X::f` when `support` is `pub use crate::a::*;` — beside the glob's own finding where the glob is governed.
#[test]
fn a_path_through_a_glob_reexport_names_the_item_the_glob_brings() {
    const SUPPORT: &str = "pub mod a { pub struct X; impl X { pub fn f() {} } }\n\
                           pub mod support { pub use crate::a::*; }\n";
    let mut mismatches = Vec::new();
    let probe = files_probe(
        "globpathfile",
        "2021",
        &[
            ("src/lib.rs", &format!("{SUPPORT}pub mod core2;\n")),
            (
                "src/core2.rs",
                "use crate::support::X;\npub fn g() { X::f(); }\n",
            ),
        ],
        &[],
    );
    answer_mismatches(
        &probe,
        "globpathfile",
        "crate::core2",
        "crate::a",
        &["crate::a::X::f in crate::core2"],
        &mut mismatches,
    );
    for (package, call) in [
        (
            "globpathuse",
            "use crate::support::X;\npub fn g() { X::f(); }\n",
        ),
        ("globpathrooted", "pub fn g() { crate::support::X::f(); }\n"),
    ] {
        answer_mismatches(
            &lib_probe(package, &format!("{SUPPORT}{call}")),
            package,
            "crate",
            "crate::a",
            &["crate::a::X::f in crate", "glob crate::a in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A path segment a module binds only through a glob of a crate whose contents are not read names that crate's path:
/// `crate::support::Command` with `support` being `pub use std::process::*;` names `std::process::Command`, beside the
/// glob's own finding. The constructor call is assembled from literals for the reason [`spawn_call`] gives.
#[test]
fn a_path_through_a_foreign_glob_names_the_foreign_path() {
    assert_crate_answers(
        "foreignglobpath",
        concat!(
            "pub mod support { pub use std::process::*; }\nuse crate::support::Command;\n",
            "pub fn g() { let _ = Command",
            "::new(\"x\"); }\n",
        ),
        "std::process",
        &[
            "glob std::process in crate",
            "std::process::Command::new in crate",
        ],
    );
}

/// A declaration binds its name in its own namespace only: `support`'s `fn X` is a value, so `X::f()` through
/// `use crate::support::X` looks `X` up as a type and reaches the struct `support`'s glob brings.
#[test]
fn a_value_item_does_not_hide_a_type_a_glob_brings() {
    let probe = files_probe(
        "valuehidesglob",
        "2021",
        &[
            (
                "src/lib.rs",
                "pub mod a { pub struct X; impl X { pub fn f() {} } }\n\
                 pub mod support { #[allow(non_snake_case)] pub fn X() {} pub use crate::a::*; }\n\
                 pub mod core2;\n",
            ),
            (
                "src/core2.rs",
                "use crate::support::X;\npub fn g() { X::f(); }\n",
            ),
        ],
        &[],
    );
    let found = ["crate::a::X::f in crate::core2"];
    assert_inline_answers(
        &probe,
        "valuehidesglob",
        "crate::core2",
        "crate::a",
        &found,
        &found,
    );
}

/// Every declaration of a name is recorded with its own visibility: with a `pub struct X` under one cfg and a private
/// one under the other, a glob in another module brings the public one whichever is written last.
#[test]
fn cfg_exclusive_declarations_keep_each_visibility() {
    let mut mismatches = Vec::new();
    for (package, a) in [
        (
            "cfgvispubfirst",
            "pub mod a { #[cfg(unix)] pub struct X; #[cfg(not(unix))] struct X; impl X { pub fn f() {} } }\n",
        ),
        (
            "cfgvisprivfirst",
            "pub mod a { #[cfg(not(unix))] struct X; #[cfg(unix)] pub struct X; impl X { pub fn f() {} } }\n",
        ),
    ] {
        let probe = files_probe(
            package,
            "2021",
            &[
                (
                    "src/lib.rs",
                    &format!("{a}pub mod r {{ pub use crate::a::*; }}\npub mod user;\n"),
                ),
                ("src/user.rs", "use crate::r::X;\npub fn g() { X::f(); }\n"),
            ],
            &[],
        );
        answer_mismatches(
            &probe,
            package,
            "crate::user",
            "crate::a",
            &["crate::a::X::f in crate::user"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// An `extern crate` is an item binding its name — or its `as` alias — to the crate it names, and one at the crate
/// root is in scope in every module: `extern crate self as me;` makes `me::clock::now()` call `crate::clock::now`.
#[test]
fn an_extern_crate_self_alias_names_the_crate_root() {
    assert_crate_answers(
        "externcrateself",
        "extern crate self as me;\npub mod clock { pub fn now() {} }\npub fn g() { me::clock::now(); }\n",
        "crate::clock",
        &["crate::clock::now in crate"],
    );
}

/// An `extern crate dep;` binds `dep`, and `extern crate dep as alias;` binds `alias`, to the dependency as a `use` does,
/// so a call through the name reports in either mode — written in the calling module, or at the crate root, which puts
/// the name in every module's scope. In edition 2015 a `::`-rooted path reads the crate root's scope, where the
/// `extern crate` is an item.
#[test]
fn an_extern_crate_names_its_crate_under_its_name_or_alias() {
    let mut mismatches = Vec::new();
    for (package, edition, lib, core) in [
        (
            "externrenamecore",
            "2021",
            "pub mod core;\n",
            "extern crate md5x as chr;\npub fn a() -> u32 { chr::compute() }\n",
        ),
        (
            "externrenameroot",
            "2021",
            "extern crate md5x as chr;\npub mod core;\n",
            "pub fn a() -> u32 { chr::compute() }\n",
        ),
        (
            "externplainroot",
            "2021",
            "extern crate md5x;\npub mod core;\n",
            "pub fn a() -> u32 { md5x::compute() }\n",
        ),
        (
            "externrootedroot2015",
            "2015",
            "extern crate md5x;\npub mod core;\n",
            "pub fn h() -> u32 { ::md5x::compute() }\n",
        ),
        (
            "externplainroot2015",
            "2015",
            "extern crate md5x;\npub mod core;\n",
            "pub fn i() -> u32 { md5x::compute() }\n",
        ),
    ] {
        let probe = files_probe(
            package,
            edition,
            &[("src/lib.rs", lib), ("src/core.rs", core)],
            &["md5x"],
        );
        answer_mismatches(
            &probe,
            package,
            "crate::core",
            "md5x",
            &["md5x::compute in crate::core"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A module declared in a block is named through, not read as a block-local item: `m::read("x")` and `use m::*;
/// read("x")` both call `std::fs::read`. The glob of the block-local module names no path a finding could carry, so
/// it reacts through the calls it brings rather than as a hazard of its own. A block's own `struct L` stays
/// block-local, so `L::f()` names nothing a prefix reaches.
#[test]
fn a_block_local_module_is_resolved_through() {
    let mut mismatches = Vec::new();
    for (package, lib) in [
        (
            "blockmodpath",
            "pub fn g() { mod m { pub use std::fs::read; } let _ = m::read(\"x\"); }\n",
        ),
        (
            "blockmodglob",
            "pub fn g() { mod m { pub use std::fs::read; } use m::*; let _ = read(\"x\"); }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, lib),
            package,
            "crate",
            "std::fs",
            &["std::fs::read in crate"],
            &mut mismatches,
        );
    }
    let local = files_probe(
        "blockstructlocal",
        "2021",
        &[
            ("src/lib.rs", "pub mod m;\npub fn g() {}\n"),
            (
                "src/m.rs",
                "pub fn h() { struct L; impl L { fn f() {} } L::f(); }\n",
            ),
        ],
        &[],
    );
    answer_mismatches(
        &local,
        "blockstructlocal",
        "crate",
        "crate::m",
        &[],
        &mut mismatches,
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// `..` is one token, so the path after it is a head rather than a method call's receiver: `0..std::process::id()`
/// reports, as `0..=std::process::id()` and `0..::std::process::id()` do.
#[test]
fn a_range_before_a_path_leaves_the_path_its_head() {
    let mut mismatches = Vec::new();
    for (package, lib) in [
        (
            "rangehead",
            "pub fn g() { for _ in 0..std::process::id() {} }\n",
        ),
        (
            "rangeinclusivehead",
            "pub fn g() -> bool { (0..=std::process::id()).is_empty() }\n",
        ),
        (
            "rangerootedhead",
            "pub fn g() { for _ in 0..::std::process::id() {} }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, lib),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A call reports wherever an expression stands, since no reader decides where an expression ends: inside a
/// block-like scrutinee, `unsafe { … }`, `{ … }`, `loop`, `if … else`, `async { … }` or a `match` nested sixty-six
/// deep; in a match arm's body after a turbofish holding a comma, a `collect::<HashMap<_, _>>()`, a bare, boxed or
/// called closure; and after a `|` that follows `.await`.
#[test]
fn a_call_reports_wherever_an_expression_stands() {
    let mut mismatches = Vec::new();
    let nested = nested_scrutinee(66);
    for (package, lib) in [
        (
            "whereunsafe",
            "pub fn g() { match unsafe { std::process::id() } { _ => () } }\n",
        ),
        (
            "whereblock",
            "pub fn g() { match { std::process::id() } { _ => () } }\n",
        ),
        (
            "whereloop",
            "pub fn g() -> u32 { match loop { break std::process::id() } { x => x } }\n",
        ),
        (
            "whereifelse",
            "pub fn g(c: bool) -> u32 { match if c { std::process::id() } else { 0 } { x => x } }\n",
        ),
        (
            "whereasync",
            "pub fn g() -> u32 { match async { std::process::id() } { _ => 0 } }\n",
        ),
        ("wherenested", nested.as_str()),
        (
            "whereturbofish",
            "pub enum K { A, B }\nfn convert<T, U: From<T>>(t: T) -> U { U::from(t) }\n\
             pub fn g(k: K) -> u64 { match k { K::A => convert::<u32, u64>(std::process::id()), K::B => 0 } }\n",
        ),
        (
            "wherecollect",
            "use std::collections::HashMap;\npub enum K { A, B }\n\
             pub fn g(k: K) -> usize { match k { K::A => vec![(1u8, 2u8)].into_iter().collect::<HashMap<_, _>>().len() \
             + std::process::id() as usize, K::B => 0 } }\n",
        ),
        (
            "wherebareclosure",
            "pub enum K { A, B }\n\
             pub fn g(k: K) -> fn(u32, u32) -> u32 { match k { K::A => |a, b| a + b, K::B => |_, _| std::process::id() } }\n",
        ),
        (
            "whereboxedclosure",
            "pub enum K { A, B }\npub fn g(k: K) -> Box<dyn Fn(u32, u32) -> u32> { match k { \
             K::A => Box::new(|a, b| a + b + std::process::id()), K::B => Box::new(|_, _| 0) } }\n",
        ),
        (
            "wherecalledclosure",
            "pub enum K { A, B }\n\
             pub fn g(k: K) -> u32 { match k { K::A => (|a: u32, b: u32| a + b)(1, std::process::id()), K::B => 0 } }\n",
        ),
        (
            "whereawaitbitor",
            "async fn a() -> u32 { 1 }\npub async fn g() -> u32 { a().await | std::process::id() }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, lib),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `match` whose scrutinee is a `match`, `levels` deep, the innermost scrutinee calling `std::process::id()`.
fn nested_scrutinee(levels: usize) -> String {
    let mut expression = String::from("std::process::id()");
    for _ in 0..levels {
        expression = format!("match {expression} {{ _ => 0u32 }}");
    }
    format!("pub fn g() -> u32 {{ {expression} }}\n")
}

/// `crate::b` importing `crate::a::X` through `use crate::{…{a::X}…};`, the group nested `levels` braces deep.
fn nested_use_import(levels: usize) -> RootProbe {
    let tree = format!("{}a::X{}", "{".repeat(levels), "}".repeat(levels));
    RootProbe::new(
        &format!("nesteduse{levels}"),
        "",
        &[
            ("src/lib.rs", "pub mod a { pub struct X; }\npub mod b;\n"),
            (
                "src/b.rs",
                &format!("#[allow(unused_imports)]\nuse crate::{tree};\n"),
            ),
        ],
    )
}

/// An import rule reads a `use` tree through the one use-tree parser under its own cap: nested 128 braces deep the
/// import is observed, and 129 deep it is refused as a scan error naming the cap, so a refusal saying a tree is
/// nested past 128 levels is true of the tree it refuses.
#[test]
fn an_import_rule_reads_a_use_tree_128_braces_deep_and_refuses_129() {
    let law = |levels: usize| {
        Constitution::new("nested-use").boundary(
            ModuleBoundary::in_crate(&format!("nesteduse{levels}"))
                .module("crate::b")
                .must_not_import("crate::a")
                .because("b does not reach a"),
        )
    };
    let observed = nested_use_import(128);
    match check(&law(128), observed.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert_eq!(found, ["crate::a::X"], "{report:?}");
        }
        other => panic!("128 braces: {other:?}"),
    }
    let refused = nested_use_import(129);
    match check(&law(129), refused.manifest()) {
        Outcome::ConstitutionError(message)
            if message.contains("nested past 128 brace levels") && message.contains("src/b.rs") => {
        }
        other => panic!("129 braces: {other:?}"),
    }
}

/// An import rule refuses an import whose head it reads through another file's unreadable `use` tree, rather than
/// reading the head without the bindings that tree makes: in an edition-2015 package `crate::a` writes
/// `use hub::X;`, and the crate root binds `hub` only in a `use` tree nested 130 braces deep. Read without that
/// binding, the head named nothing and the import of `crate::forbidden::X` went unreported; so it is a scan error
/// (exit 2) naming the module whose file holds the tree. Written with the binding at its own level, `pub use forbidden
/// as hub;`, the import reports.
#[test]
fn an_import_read_through_another_files_unreadable_use_tree_is_refused() {
    let law = |package: &str| {
        Constitution::new("refused-root").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::a")
                .must_not_import("crate::forbidden")
                .because("a does not reach forbidden"),
        )
    };
    let nested = format!("{}forbidden as hub{}", "{".repeat(130), "}".repeat(130));
    let refused = files_probe(
        "refusedroot",
        "2015",
        &[
            (
                "src/lib.rs",
                &format!(
                    "pub mod a;\npub mod forbidden {{ pub struct X; }}\npub use crate::{nested};\n"
                ),
            ),
            ("src/a.rs", "use hub::X;\npub fn h(_x: X) {}\n"),
        ],
        &[],
    );
    match check(&law("refusedroot"), refused.manifest()) {
        Outcome::ConstitutionError(message)
            if message.contains("src/a.rs")
                && message.contains("it reads names through `crate`")
                && message.contains("nested past 128 brace levels") => {}
        other => panic!(
            "expected the import read through the unreadable tree to be refused, got {other:?}"
        ),
    }
    let read = files_probe(
        "refusedrootcontrol",
        "2015",
        &[
            (
                "src/lib.rs",
                "pub mod a;\npub mod forbidden { pub struct X; }\npub use forbidden as hub;\n",
            ),
            ("src/a.rs", "use hub::X;\npub fn h(_x: X) {}\n"),
        ],
        &[],
    );
    match check(&law("refusedrootcontrol"), read.manifest()) {
        Outcome::Violations(report) => assert_eq!(
            report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect::<Vec<_>>(),
            ["crate::forbidden::X"]
        ),
        other => panic!("expected the control's import to react, got {other:?}"),
    }
}

/// The refusal of a file holding an unreadable `use` tree is met through a glob as it is met directly: `crate::client`
/// writes `use crate::bad::*;` and `use hub::X;`, and `bad` binds `hub` only in a `use` tree nested 129 braces deep.
/// Read through the glob without that tree's bindings, `hub` named nothing and the import went unreported as an
/// external crate's; it is a scan error (exit 2) naming the module read through. With `pub use crate::forbidden as
/// hub;` in `bad` the import names `crate::bad::hub::X` — the module the glob brings `hub` from, the re-export not
/// followed — and reports, beside the glob's own `crate::bad`, under a rule forbidding `crate::bad`. rustc 1.96.0, edition 2021, builds both.
#[test]
fn an_import_read_through_a_glob_into_an_unreadable_file_is_refused() {
    let law = |package: &str| {
        Constitution::new("refused-glob").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::client")
                .must_not_import("crate::bad")
                .because("the client does not reach bad"),
        )
    };
    let lib = "pub mod forbidden { pub struct X; }\npub mod bad;\npub mod client;\n";
    let client =
        "#[allow(unused_imports)]\nuse crate::bad::*;\n#[allow(unused_imports)]\nuse hub::X;\n";
    let nested = format!("{}forbidden as hub{}", "{".repeat(129), "}".repeat(129));
    let refused = RootProbe::new(
        "refusedglob",
        "",
        &[
            ("src/lib.rs", lib),
            ("src/bad.rs", &format!("pub use crate::{nested};\n")),
            ("src/client.rs", client),
        ],
    );
    match check(&law("refusedglob"), refused.manifest()) {
        Outcome::ConstitutionError(message)
            if message.contains("src/client.rs")
                && message.contains("it reads names through `crate::bad`") => {}
        other => panic!("expected the import read through the glob to be refused, got {other:?}"),
    }
    let read = RootProbe::new(
        "refusedglobcontrol",
        "",
        &[
            ("src/lib.rs", lib),
            ("src/bad.rs", "pub use crate::forbidden as hub;\n"),
            ("src/client.rs", client),
        ],
    );
    match check(&law("refusedglobcontrol"), read.manifest()) {
        Outcome::Violations(report) => assert_eq!(
            report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect::<Vec<_>>(),
            ["crate::bad", "crate::bad::hub::X"]
        ),
        other => panic!("expected the control's import to react, got {other:?}"),
    }
}

/// A binding holds a name only in the namespaces its target provides: a `type` alias names a type, and an import names
/// what its target names in each namespace. So `support`'s alias `X` of a struct leaves the value `X` to its glob of
/// `crate::v`, whose `fn X` a call `X()` reaches; and `support`'s import of `crate::v::X`, a function, leaves the type
/// `X` to its glob of `crate::a`, whose struct `X::f()` reaches.
#[test]
fn a_binding_holds_only_the_namespaces_its_target_provides() {
    const ITEMS: &str = "pub mod a { pub struct X; impl X { pub fn f() {} } }\n\
                         pub mod v { #[allow(non_snake_case)] pub fn X() {} }\n";
    let mut mismatches = Vec::new();
    for (package, support, call, prefix, found) in [
        (
            "bindingtypeonly",
            "pub type X = crate::a::X; pub use crate::v::*;",
            "use crate::support::X;\npub fn g() { X(); }\n",
            "crate::v",
            "crate::v::X in crate::core2",
        ),
        (
            "bindingvalueonly",
            "pub use crate::v::X; pub use crate::a::*;",
            "use crate::support::X;\npub fn g() { X::f(); }\n",
            "crate::a",
            "crate::a::X::f in crate::core2",
        ),
    ] {
        let lib = format!("{ITEMS}pub mod support {{ {support} }}\npub mod core2;\n");
        let probe = files_probe(
            package,
            "2021",
            &[("src/lib.rs", &lib), ("src/core2.rs", call)],
            &[],
        );
        answer_mismatches(
            &probe,
            package,
            "crate::core2",
            prefix,
            &[found],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A call reports under every path its resolution passes through: the path as written and each path a rewrite of it
/// names. Through `support`'s named `pub use`, glob or `type` alias of `crate::a::X`, reached through a `use` or
/// written crate-rooted, `X::f()` reports `crate::support::X::f` under `crate::support` and `crate::a::X::f` under
/// `crate::a`, beside the glob's own finding where the glob reaches that prefix.
#[test]
fn a_call_reports_under_every_path_its_resolution_passes() {
    const A: &str = "pub mod a { pub struct X; impl X { pub fn f() {} } }\n";
    const USE: &str = "use crate::support::X;\npub fn g() { X::f(); }\n";
    const PATH: &str = "pub fn g() { crate::support::X::f(); }\n";
    let mut mismatches = Vec::new();
    for (package, support, call, glob) in [
        ("everypathnameduse", "pub use crate::a::X;", USE, false),
        ("everypathnamedpath", "pub use crate::a::X;", PATH, false),
        ("everypathglobuse", "pub use crate::a::*;", USE, true),
        ("everypathglobpath", "pub use crate::a::*;", PATH, true),
        ("everypathaliasuse", "pub type X = crate::a::X;", USE, false),
        (
            "everypathaliaspath",
            "pub type X = crate::a::X;",
            PATH,
            false,
        ),
    ] {
        let probe = lib_probe(
            package,
            &format!("{A}pub mod support {{ #[allow(unused_imports)] {support} }}\n{call}"),
        );
        answer_mismatches(
            &probe,
            package,
            "crate",
            "crate::support",
            &["crate::support::X::f in crate"],
            &mut mismatches,
        );
        let under_a: &[&str] = if glob {
            &["crate::a::X::f in crate", "glob crate::a in crate"]
        } else {
            &["crate::a::X::f in crate"]
        };
        answer_mismatches(
            &probe,
            package,
            "crate",
            "crate::a",
            under_a,
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A finding's identity is the path it names, so one call whose resolution passes two paths under one prefix is two
/// findings: `crate::p::support::X::f()`, with `support` re-exporting its sibling `a`'s `X`, reports both
/// `crate::p::a::X::f` and `crate::p::support::X::f` under `crate::p`.
#[test]
fn one_call_through_two_paths_under_one_prefix_is_two_findings() {
    let probe = lib_probe(
        "twopathsoneprefix",
        "pub mod p {\n    pub mod a { pub struct X; impl X { pub fn f() {} } }\n    \
         pub mod support { pub use super::a::X; }\n}\npub fn g() { crate::p::support::X::f(); }\n",
    );
    let found = [
        "crate::p::a::X::f in crate",
        "crate::p::support::X::f in crate",
    ];
    assert_inline_answers(
        &probe,
        "twopathsoneprefix",
        "crate",
        "crate::p",
        &found,
        &found,
    );
}

/// A qualified path's `<…>` is read as one group by the one angle reading: `<HashMap<u8, u8> as Default>::…`,
/// `<HashMap<String, u32>>::…` and `<Result<u8, u8> as Clone>::…` each close at their own `>`, so the tail is left to
/// the receiver-method bound and the rooted call after it, in the same match arm, reports.
#[test]
fn a_qualified_paths_type_is_read_as_one_group() {
    const K: &str = "pub enum K { A, B }\n";
    let mut mismatches = Vec::new();
    for (package, lib) in [
        (
            "typequalifiedtrait",
            "use std::collections::HashMap;\npub fn g(k: K) -> usize { match k { K::A => <HashMap<u8, u8> as \
             Default>::default().len() + std::process::id() as usize, K::B => 0 } }\n",
        ),
        (
            "typequalifiedbare",
            "use std::collections::HashMap;\npub fn g(k: K) -> usize { match k { K::A => <HashMap<String, u32>>::\
             with_capacity(std::process::id() as usize).len(), K::B => 0 } }\n",
        ),
        (
            "typequalifiedresult",
            "pub fn g(k: K) -> u32 { match k { K::A => <Result<u8, u8> as Clone>::clone(&Ok(1)).map_or(0, \
             u32::from) + std::process::id(), K::B => 0 } }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, &format!("{K}{lib}")),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A cast's type ends where rustc ends it, so a comparison beside a cast opens no generic list: in
/// `(n as u32) < m && m > ::std::process::id()` and `x as u32 > ::std::process::id()` the `::` roots the call. Measured
/// with rustc: a cast type ends at `+`, `-`, `*`, `&`, `&&`, `==`, `>`, `,`, `)` and `as`, while a `<` or `<<` after it
/// is read as the type's generic arguments, so `x as u32 < m` does not compile. After a block-like expression's `}` a
/// `<` compares too: `match x { _ => 1 } < n && n > ::std::process::id()` roots the call, and a scrutinee
/// `{ 1 } < 2` ends at the body's `{`.
#[test]
fn a_comparison_beside_a_cast_or_after_a_block_opens_no_type_position() {
    let mut mismatches = Vec::new();
    for (package, lib) in [
        (
            "castparencompare",
            "pub fn g(n: u8, m: u32) -> bool { (n as u32) < m && m > ::std::process::id() }\n",
        ),
        (
            "castgreater",
            "pub fn g(x: u8) -> bool { x as u32 > ::std::process::id() }\n",
        ),
        (
            "castliteralcompare",
            "pub fn g(n: u32) -> bool { 0 < n && n > ::std::process::id() }\n",
        ),
        (
            "bracecomparethenrooted",
            "pub fn g(x: u8, n: u32) -> bool { let b = match x { _ => 1 } < n && n > ::std::process::id(); b }\n",
        ),
        (
            "bracecomparescrutinee",
            "pub fn g() -> u32 { match { 1 } < 2 { true => std::process::id(), false => 0 } }\n",
        ),
        (
            "caststringcompare",
            "pub fn g(y: &str) -> bool { \"x\" < y && 1 > ::std::process::id() }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, lib),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// An operand keyword ends an operand, so a `|`, `||` or `<` after it is an operator and the call after that reports:
/// `true`, `false`, `self` and `Self` as values, and a postfix `.await`. `Self` names no `bool`, so it has no `||`
/// row. The controls are a field and a method call on `self`, and a comparison pair after `true &&`.
#[test]
fn an_operand_keyword_ends_an_operand() {
    const U32_TRAIT: &str = "pub trait T { fn m(self) -> u32; }\n";
    const BOOL_TRAIT: &str = "pub trait T { fn m(self) -> bool; }\n";
    const S_BITOR: &str = "pub struct S;\nimpl std::ops::BitOr<u32> for S { type Output = u32; fn bitor(self, r: u32) \
                           -> u32 { r } }\n";
    const S_ORD: &str = "pub struct S;\nimpl PartialEq<u32> for S { fn eq(&self, _: &u32) -> bool { false } }\n\
                         impl PartialOrd<u32> for S { fn partial_cmp(&self, _: &u32) -> Option<std::cmp::Ordering> \
                         { None } }\n";
    let rows = [
        (
            "keywordtruebitor",
            String::from("pub fn g() -> bool { true | (std::process::id() > 0) }\n"),
        ),
        (
            "keywordtrueor",
            String::from("pub fn g() -> bool { true || std::process::id() > 0 }\n"),
        ),
        (
            "keywordtruelt",
            String::from("pub fn g() -> bool { true < (std::process::id() > 0) }\n"),
        ),
        (
            "keywordfalsebitor",
            String::from("pub fn g() -> bool { false | (std::process::id() > 0) }\n"),
        ),
        (
            "keywordfalseor",
            String::from("pub fn g() -> bool { false || std::process::id() > 0 }\n"),
        ),
        (
            "keywordfalselt",
            String::from("pub fn g() -> bool { false < (std::process::id() > 0) }\n"),
        ),
        (
            "keywordselfbitor",
            format!(
                "{U32_TRAIT}impl T for u32 {{ fn m(self) -> u32 {{ self | std::process::id() }} }}\n"
            ),
        ),
        (
            "keywordselfor",
            format!(
                "{BOOL_TRAIT}impl T for bool {{ fn m(self) -> bool {{ self || std::process::id() > 0 }} }}\n"
            ),
        ),
        (
            "keywordselflt",
            format!(
                "{BOOL_TRAIT}impl T for u32 {{ fn m(self) -> bool {{ self < std::process::id() }} }}\n"
            ),
        ),
        (
            "keywordselftypebitor",
            format!("{S_BITOR}impl S {{ pub fn m() -> u32 {{ Self | std::process::id() }} }}\n"),
        ),
        (
            "keywordselftypelt",
            format!("{S_ORD}impl S {{ pub fn m() -> bool {{ Self < std::process::id() }} }}\n"),
        ),
        (
            "keywordawaitbitor",
            String::from(
                "async fn a() -> u32 { 1 }\npub async fn g() -> u32 { a().await | std::process::id() }\n",
            ),
        ),
        (
            "keywordawaitor",
            String::from(
                "async fn b() -> bool { true }\npub async fn g() -> bool { b().await || std::process::id() > 0 }\n",
            ),
        ),
        (
            "keywordawaitlt",
            String::from(
                "async fn a() -> u32 { 1 }\npub async fn g() -> bool { a().await < std::process::id() }\n",
            ),
        ),
        (
            "keywordctlselffield",
            String::from(
                "pub struct S(pub u32);\nimpl S { pub fn m(&self) -> u32 { self.0 | std::process::id() } }\n",
            ),
        ),
        (
            "keywordctlselfmethod",
            String::from(
                "pub struct S;\nimpl S { fn t(&self) -> bool { true } pub fn m(&self) -> bool { self.t() || \
                 std::process::id() > 0 } }\n",
            ),
        ),
        (
            "keywordctltrueand",
            String::from(
                "pub fn g(n: u32) -> bool { true && n < 3 && n > ::std::process::id() }\n",
            ),
        ),
    ];
    let mut mismatches = Vec::new();
    for (package, lib) in rows {
        answer_mismatches(
            &lib_probe(package, &lib),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A refusal names the file it was met in: a crate of three files whose `src/deep.rs` nests a `use` tree past the
/// use-tree parser's cap is refused with a message naming `src/deep.rs`, and not the others. The nested groups name
/// a real item, so rustc builds the fixture.
#[test]
fn a_refusal_names_the_file_it_was_met_in() {
    let deep = format!(
        "#[allow(unused_imports)]\nuse std::{}time::Instant{};\n",
        "{".repeat(130),
        "}".repeat(130)
    );
    let probe = RootProbe::new(
        "refusalnamesfile",
        "",
        &[
            (
                "src/lib.rs",
                "pub mod deep;\npub mod other;\npub fn root() {}\n",
            ),
            ("src/deep.rs", &deep),
            ("src/other.rs", "pub fn h() -> u32 { std::process::id() }\n"),
        ],
    );
    for strict_external in [false, true] {
        match inline_outcome(
            &probe,
            "refusalnamesfile",
            "crate",
            "std::process",
            strict_external,
        ) {
            Outcome::ConstitutionError(message)
                if message.contains("src/deep.rs")
                    && !message.contains("src/other.rs")
                    && !message.contains("src/lib.rs")
                    && message.contains("nested past 128 brace levels") => {}
            other => panic!("strict_external = {strict_external}: {other:?}"),
        }
    }
}

/// A name a glob brings to a bare head passes through the glob's module, so the call reports under that path too:
/// `use crate::support::*; X::f()` with `support` re-exporting `crate::a::X` reports `crate::support::X::f` under
/// `crate::support` and `crate::a::X::f` under `crate::a`, beside the glob's own finding. The path through the glob's
/// module is not read again, so a binding the glob cannot see adds nothing to it.
#[test]
fn a_name_a_glob_brings_reports_under_the_globs_module_too() {
    let probe = lib_probe(
        "globthroughpath",
        "pub mod a { pub struct X; impl X { pub fn f() {} } }\npub mod support { pub use crate::a::X; }\n\
         use crate::support::*;\npub fn g() { X::f(); }\n",
    );
    let mut mismatches = Vec::new();
    for (prefix, found) in [
        (
            "crate::support",
            [
                "crate::support::X::f in crate",
                "glob crate::support in crate",
            ],
        ),
        (
            "crate::a",
            ["crate::a::X::f in crate", "glob crate::support in crate"],
        ),
    ] {
        answer_mismatches(
            &probe,
            "globthroughpath",
            "crate",
            prefix,
            &found,
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `}` ends a block-like operand as well as a statement, and is read as an operand's end, so a comparison after a
/// block — in a `let` initializer, a call's argument — opens no qualified path and the rooted call after it reports.
#[test]
fn a_comparison_after_a_block_opens_no_qualified_path() {
    let mut mismatches = Vec::new();
    for (package, body) in [
        (
            "exprifelse",
            "pub fn g(c: bool, n: u32) -> bool { let b = if c { 1 } else { 2 } < n && n > ::std::process::id(); b }\n",
        ),
        (
            "exprloop",
            "pub fn g(n: u32) -> bool { let b = loop { break 1 } < n && n > ::std::process::id(); b }\n",
        ),
        (
            "exprcallarg",
            "pub fn h(_: bool) {}\npub fn g(x: u8, n: u32) { h(match x { _ => 1 } < n && n > ::std::process::id()) }\n",
        ),
        (
            "exprblock",
            "pub fn g(n: u32) -> bool { let b = { 1 } < n && n > ::std::process::id(); b }\n",
        ),
        (
            "exprunsafe",
            "pub fn g(n: u32) -> bool { let b = unsafe { 1 } < n && n > ::std::process::id(); b }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, body),
            package,
            "crate",
            "std::process",
            &["std::process::id in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The price of reading a `}` as an operand's end: a qualified path opening a statement right after a block is read
/// as a comparison, so its tail `::md5x` is read as a rooted path, which names the dependency `md5x` under
/// `.strict_external()` — a declared over-reaction. After a `;` or a `{` the qualified path's tail is left to the
/// receiver-method bound, as the controls show.
#[test]
fn a_qualified_path_after_a_closing_brace_is_read_as_a_rooted_path() {
    const W: &str = "pub struct W;\nimpl W { pub fn md5x() {} }\n";
    let mut mismatches = Vec::new();
    for (package, body, strict) in [
        (
            "stmtifelse",
            "pub fn g(c: bool) { if c {} else {} <W>::md5x(); }\n",
            &["md5x in crate"][..],
        ),
        (
            "stmtloop",
            "pub fn g() { loop { break } <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtmatch",
            "pub fn g(c: bool) { match c { _ => {} } <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtblock",
            "pub fn g() { { } <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtlabel",
            "pub fn g() { 'a: loop { break 'a } <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtunsafe",
            "pub fn g() { unsafe {} <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtfnitem",
            "pub fn g() { fn h() {} <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtwhile",
            "pub fn g(c: bool) { while c {} <W>::md5x(); }\n",
            &["md5x in crate"],
        ),
        (
            "stmtaftersemi",
            "pub fn g() { let _s = \"a\"; <W>::md5x(); }\n",
            &[],
        ),
        ("stmtafteropen", "pub fn g() { <W>::md5x(); }\n", &[]),
    ] {
        let probe = files_probe(
            package,
            "2021",
            &[("src/lib.rs", &format!("#![allow(unused)]\n{W}{body}"))],
            &["md5x"],
        );
        for (strict_external, expected) in [(false, &[][..]), (true, strict)] {
            let got = inline_findings(&probe, package, "crate", "md5x", strict_external);
            if got != expected {
                mismatches.push(format!(
                    "{package}, strict_external = {strict_external}: {got:?}"
                ));
            }
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// Each import of a name is read for its own target: with `support` importing `X` from `crate::v`, a function, under
/// one cfg and from `crate::w`, a struct, under the other, `X::f()` through `support` reaches the struct `crate::w::X`,
/// whatever the import written first holds.
#[test]
fn each_import_of_a_name_holds_the_namespaces_of_its_own_target() {
    let probe = files_probe(
        "importsownnamespaces",
        "2021",
        &[
            (
                "src/lib.rs",
                "pub mod a { pub struct X; impl X { pub fn f() {} } }\n\
                 pub mod v { #[allow(non_snake_case)] pub fn X() {} }\n\
                 pub mod w { pub struct X; impl X { pub fn f() {} } }\n\
                 pub mod support {\n    #[cfg(unix)] pub use crate::v::X;\n    #[cfg(not(unix))] pub use crate::w::X;\n    \
                 pub use crate::a::*;\n}\npub mod core2;\n",
            ),
            (
                "src/core2.rs",
                "use crate::support::X;\npub fn g() { X::f(); }\n",
            ),
        ],
        &[],
    );
    let found = ["crate::w::X::f in crate::core2"];
    assert_inline_answers(
        &probe,
        "importsownnamespaces",
        "crate::core2",
        "crate::w",
        &found,
        &found,
    );
}

/// Whether a `}` ends a statement is read back one construct at a time, never through the statements before it, so a
/// body of twenty thousand consecutive `for` loops is read to its end, and the call after them reports. The qualified
/// path between them is a construct the read passes over; under `std::process` this test judges nothing of it.
#[test]
fn a_long_run_of_statements_is_read_without_recursing_through_it() {
    let body = "    for _ in 0..1 {}\n".repeat(20_000);
    let lib = format!(
        "pub struct W;\nimpl W {{ pub fn md5x() {{}} }}\npub fn g() {{\n{body}    <W>::md5x();\n    let _ = std::process::id();\n}}\n"
    );
    let probe = lib_probe("longstatementrun", &lib);
    let found = ["std::process::id in crate"];
    assert_inline_answers(
        &probe,
        "longstatementrun",
        "crate",
        "std::process",
        &found,
        &found,
    );
}

/// The two file modules every fixture below governs from `crate::core`: `crate::a`, holding `X` with its associated
/// `f`, and `crate::core`, holding `core`.
fn core_and_a(package: &str, lib_extra: &str, core: &str) -> RootProbe {
    files_probe(
        package,
        "2021",
        &[
            (
                "src/lib.rs",
                &format!(
                    "#![allow(non_snake_case, dead_code, unused_imports)]\npub mod a;\npub mod core;\n{lib_extra}"
                ),
            ),
            (
                "src/a.rs",
                "pub struct X;\nimpl X { pub fn f() -> u8 { 0 } }\n",
            ),
            ("src/core.rs", core),
        ],
        &[],
    )
}

/// `super` in a module declared in a function body names the module the function stands in, as rustc resolves it —
/// measured on rustc 1.96.0, edition 2021: `super::Y` naming a `struct Y` of the same block is refused with
/// `E0433`, so the block is not what `super` names — and the call through the enclosing module's import reports.
#[test]
fn a_super_path_in_a_module_declared_in_a_block_names_the_enclosing_module() {
    let probe = core_and_a(
        "superpastblock",
        "",
        "use crate::a::X;\npub fn g() -> u8 { mod m { pub fn h() -> u8 { super::X::f() } } m::h() }\n",
    );
    let found = ["crate::a::X::f in crate::core"];
    assert_inline_answers(
        &probe,
        "superpastblock",
        "crate::core",
        "crate::a",
        &found,
        &found,
    );
}

/// An item declared right after a macro invocation is an item: `thread_local! { … }` is one node, so the `mod` after
/// it starts an item and the call through the module's re-export reports. The control writes a `const` item in the
/// macro's place.
#[test]
fn an_item_after_a_macro_invocation_is_an_item() {
    let mut mismatches = Vec::new();
    for (package, before) in [
        ("itemaftermacro", "thread_local! { static T: u8 = 0; }"),
        ("itemafterconst", "const T: u8 = 0;"),
    ] {
        let probe = core_and_a(
            package,
            "",
            &format!(
                "{before}\nmod inner {{ pub use crate::a::X; }}\npub fn g() -> u8 {{ inner::X::f() }}\n"
            ),
        );
        answer_mismatches(
            &probe,
            package,
            "crate::core",
            "crate::a",
            &["crate::a::X::f in crate::core"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A braced macro called through a path headed by `crate`, `self` or `super` stands as an item, so the `mod q;` after
/// it is declared and `q.rs` is governed — with or without an attribute before the call.
#[test]
fn a_mod_after_a_braced_macro_called_through_a_keyword_headed_path_is_declared() {
    const MACRO: &str = "#[macro_export]\nmacro_rules! m { () => {} }\npub mod a;\n";
    const IMPORT: &str = "#[allow(unused_imports)]\nuse crate::a::X;\n";
    let mut mismatches = Vec::new();
    for (package, lib, q_file, q_module) in [
        (
            "cratepathmacro",
            "crate::m!{}\nmod q;\n",
            "src/q.rs",
            "crate::q",
        ),
        (
            "selfpathmacro",
            "self::m!{}\nmod q;\n",
            "src/q.rs",
            "crate::q",
        ),
        (
            "attrpathmacro",
            "#[allow(unused)] crate::m!{}\nmod q;\n",
            "src/q.rs",
            "crate::q",
        ),
        (
            "superpathmacro",
            "mod z { super::m!{} mod q; }\n",
            "src/z/q.rs",
            "crate::z::q",
        ),
    ] {
        let lib = format!("{MACRO}{lib}");
        let probe = files_probe(
            package,
            "2021",
            &[
                ("src/lib.rs", &lib),
                ("src/a.rs", "pub struct X;\n"),
                (q_file, IMPORT),
            ],
            &[],
        );
        let law = Constitution::new("path-macro").boundary(
            ModuleBoundary::in_crate(package)
                .module(q_module)
                .must_not_import("crate::a")
                .because("q does not reach a"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report)
                if report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .eq(["crate::a::X"]) => {}
            other => mismatches.push(format!("{package}: {other:?}")),
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// An import holds a namespace its target's module may generate: `crate::c` declares a `fn X` and a macro invoked
/// there — bare, or through a path headed by `self` or `crate` — generates a braced `struct X`, which the scanner
/// does not read, so `use crate::c::X; X::f()` still names `crate::c::X::f` — the target is unknown in the type
/// namespace, not absent.
#[test]
fn an_import_holds_a_namespace_its_targets_macro_may_generate() {
    for (package, call) in [
        ("macronamespace", "gen!();"),
        ("macronamespaceself", "self::gen!();"),
        ("macronamespacecrate", "crate::c::gen! {}"),
    ] {
        let lib = format!(
            "#![allow(non_snake_case)]\npub mod c {{\n    macro_rules! gen {{ () => {{ pub struct X {{}} impl X {{ pub fn f() {{}} }} }}; }}\n    \
             pub(crate) use gen;\n    {call}\n    pub fn X() {{}}\n}}\npub mod core2;\n"
        );
        let probe = files_probe(
            package,
            "2021",
            &[
                ("src/lib.rs", &lib),
                ("src/core2.rs", "use crate::c::X;\npub fn g() { X::f(); }\n"),
            ],
            &[],
        );
        let found = ["crate::c::X::f in crate::core2"];
        assert_inline_answers(&probe, package, "crate::core2", "crate::c", &found, &found);
    }
}

/// A `<` after `.await` compares, since `.await` ends an operand: in `a().await < b && c > ::std::process::id()` the
/// `>` closes no qualified path and the rooted call reports.
#[test]
fn a_less_than_after_await_opens_no_qualified_path() {
    assert_crate_answers(
        "awaitless",
        "async fn a() -> u32 { 1 }\npub async fn g(b: u32, c: u32) -> bool { a().await < b && c > ::std::process::id() }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

/// A `<` after `impl`, or after a `for` with a lifetime following it, opens a generic parameter list, never a
/// qualified path, so a rooted path right after its `>` is read rather than taken for a qualified path's tail:
/// `impl<T> ::std::marker::Unpin for W<T>` mentions `std::marker::Unpin`, and `F: for<'a> ::std::ops::Fn(&'a u8)` and
/// `&dyn for<'a> ::std::ops::Fn(&'a u8)` each read `std::ops::Fn` as the call a parenthesized bound is read as.
/// rustc 1.96.0, edition 2021, builds all three.
#[test]
fn a_less_than_after_impl_or_a_binders_for_opens_no_qualified_path() {
    let probe = lib_probe(
        "implgenerics",
        "pub struct W<T>(T);\nimpl<T> ::std::marker::Unpin for W<T> {}\n",
    );
    let law = Constitution::new("implgenerics").boundary(
        ModuleBoundary::in_crate("implgenerics")
            .module("crate")
            .must_not_call_inline("std::marker")
            .strict_prefix_only()
            .depth(xuanji::ScanDepth::Subtree)
            .because("no mention of markers"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => assert_eq!(
            report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect::<Vec<_>>(),
            ["std::marker::Unpin in crate"]
        ),
        other => panic!("expected the mention after `impl<T>` to react, got {other:?}"),
    }
    for (package, item) in [
        (
            "wherebinder",
            "pub fn h<F>(_f: F) where F: for<'a> ::std::ops::Fn(&'a u8) {}\n",
        ),
        (
            "dynbinder",
            "pub fn k(_f: &dyn for<'a> ::std::ops::Fn(&'a u8)) {}\n",
        ),
    ] {
        assert_crate_answers(package, item, "std::ops", &["std::ops::Fn in crate"]);
    }
}

/// The verdict does not depend on the order modules are met in: a glob of `crate::p` reaches `std::process` through
/// `crate::p::a`, and a re-export chain past the cap stands in `crate::p::b`, which sorts after it. Reading every
/// module before answering, the chain's refusal is the answer in either mode.
#[test]
fn the_verdict_does_not_depend_on_the_order_modules_are_read() {
    let mut chain = String::from("pub mod m0 { pub struct X; impl X { pub fn f() {} } }\n");
    for i in 1..=70 {
        chain.push_str(&format!(
            "pub mod m{i} {{ pub use super::m{}::X; }}\n",
            i - 1
        ));
    }
    let lib = format!(
        "#![allow(unused_imports)]\npub mod p {{\n    pub mod a {{ pub use std::process::Command; }}\n    pub mod b {{\n{chain}    }}\n}}\n\
         use p::*;\npub fn g() {{}}\n"
    );
    let probe = lib_probe("verdictorder", &lib);
    for strict_external in [false, true] {
        match inline_outcome(
            &probe,
            "verdictorder",
            "crate",
            "std::process",
            strict_external,
        ) {
            Outcome::ConstitutionError(message) if message.contains("cannot judge a chain") => {}
            other => panic!("strict_external = {strict_external}: {other:?}"),
        }
    }
}

/// A chain refused in a module declared in a block names that module as a reader can find it: the block is written
/// `{block}`, never by the table and scope numbers the resolver keys it on.
#[test]
fn a_chain_refused_in_a_block_declared_module_names_it_readably() {
    let aliases: String = (1..=70)
        .map(|i| format!("        pub type A{i} = A{};\n", i - 1))
        .collect();
    let lib = format!(
        "pub fn g() {{\n    mod inner {{\n        pub struct X;\n        impl X {{ pub fn f() {{}} }}\n        \
         pub type A0 = X;\n{aliases}    }}\n    inner::A70::f();\n}}\n"
    );
    let probe = lib_probe("blockchain", &lib);
    match inline_outcome(&probe, "blockchain", "crate", "std::process", false) {
        Outcome::ConstitutionError(message)
            if message.contains("cannot judge a chain")
                && message.contains("in crate::{block}::inner") => {}
        other => panic!("{other:?}"),
    }
}

/// A call inside a turbofish's const argument reports, whether the turbofish follows a method or a path: the `<…>`
/// after `::` is counted to find where the path ends, and its contents are read as any other tokens are.
#[test]
fn a_call_inside_a_turbofish_reports() {
    const ITEMS: &str = "pub mod k { pub const fn n() -> usize { 1 } }\npub struct V;\n\
                         impl V { pub fn m<const N: usize>(&self) -> usize { N } }\n\
                         pub fn f<const N: usize>() -> usize { N }\n";
    let mut mismatches = Vec::new();
    for (package, call) in [
        (
            "turbofishmethod",
            "pub fn g(v: &V) -> usize { v.m::<{ crate::k::n() }>() }\n",
        ),
        (
            "turbofishpath",
            "pub fn g() -> usize { f::<{ crate::k::n() }>() }\n",
        ),
    ] {
        answer_mismatches(
            &lib_probe(package, &format!("{ITEMS}{call}")),
            package,
            "crate",
            "crate::k",
            &["crate::k::n in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A float literal written with a trailing `.` is one token, as rustc reads it, so the `<` after `1.` compares and
/// the rooted call after the `>` reports.
#[test]
fn a_float_literal_ending_in_a_dot_is_one_token() {
    assert_crate_answers(
        "floatdot",
        "pub fn h(x: f64) -> bool { 1. < x && x > ::std::process::id() as f64 }\n",
        "std::process",
        &["std::process::id in crate"],
    );
}

/// An `impl` whose self type is a macro invocation still has a member body: `impl Tr for t!() { … }` binds none of
/// its members to a bare head, so beside a member `fn exit`, `exit(3)` in its method is the imported
/// `std::process::exit`, as rustc resolves it.
#[test]
fn an_impl_for_a_macro_type_binds_none_of_its_members() {
    assert_crate_answers(
        "implformacro",
        "macro_rules! t { () => { S } }\npub struct S;\npub trait Tr { fn exit(); fn g(); }\nuse std::process::exit;\n\
         impl Tr for t!() { fn exit() {} fn g() { exit(3) } }\n",
        "std::process",
        &["std::process::exit in crate"],
    );
}

/// A `const` named like a contextual qualifier is a `const` item: `pub const safe: u8` and `pub const default: u8`
/// are declared, so a prefix naming either is accepted and, with no call under it, reports nothing.
#[test]
fn a_const_named_like_a_qualifier_is_declared() {
    let probe = lib_probe(
        "constqualifiername",
        "#![allow(non_upper_case_globals)]\npub const safe: u8 = 1;\npub const default: u8 = 2;\n",
    );
    for prefix in ["crate::safe", "crate::default"] {
        match inline_outcome(&probe, "constqualifiername", "crate", prefix, false) {
            Outcome::Clean(_) => {}
            other => panic!("{prefix}: {other:?}"),
        }
    }
}

/// A macro invocation inside an item — a `const`'s initializer calling `concat!` — generates no item, so an import of
/// a `fn X` from that module holds the value namespace alone and the struct `X` a glob brings answers `X::f()`.
#[test]
fn a_macro_inside_an_item_generates_no_namespace() {
    let probe = files_probe(
        "macroininitializer",
        "2021",
        &[
            (
                "src/lib.rs",
                "#![allow(non_snake_case)]\npub mod c {\n    pub fn X() {}\n    pub const S: usize = concat!(\"ab\").len();\n}\n\
                 pub mod p { pub struct X; impl X { pub fn f() {} } }\n\
                 pub mod support { pub use crate::c::X; pub use crate::p::*; }\npub mod core2;\n",
            ),
            (
                "src/core2.rs",
                "use crate::support::X;\npub fn g() { X::f(); }\n",
            ),
        ],
        &[],
    );
    let found = ["crate::p::X::f in crate::core2"];
    assert_inline_answers(
        &probe,
        "macroininitializer",
        "crate::core2",
        "crate::p",
        &found,
        &found,
    );
}

/// Every reader of a file reads it in its target's edition, here the package's: in edition 2015 `dyn` is an identifier, so
/// `mod dyn { use crate::a::X; }` is a module named `dyn`, and an inbound rule forbidding `crate::dyn` from importing
/// `crate::a` reports its import.
#[test]
fn an_import_scan_reads_the_targets_edition() {
    let probe = files_probe(
        "editionimport",
        "2015",
        &[
            (
                "src/lib.rs",
                "#![allow(dead_code, unused_imports)]\npub mod a;\nmod dyn { use crate::a::X; }\n",
            ),
            ("src/a.rs", "pub struct X;\n"),
        ],
        &[],
    );
    let law = Constitution::new("edition-import").boundary(
        ModuleBoundary::in_crate("editionimport")
            .module("crate::a")
            .must_not_be_imported_by("crate::dyn")
            .because("dyn does not reach a"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert!(found.iter().any(|f| f.contains("crate::dyn")), "{found:?}");
        }
        other => panic!("{other:?}"),
    }
}

/// A module declared in a function body is not the same-named module its file declares at module level: rustc names
/// no path to it. So an import of `crate::x` written in `fn f() { mod m { … } }` inside `crate::a` is not
/// `crate::a::m`'s, and a rule forbidding `crate::a::m` to import `crate::x` reports nothing for it, while one
/// forbidding `crate::a`'s subtree reports it, importer `crate::a::{block}::m`. A `super` there names `crate::a`, as
/// rustc resolves it, so `mod n { use super::super::x::Y; }` in another body imports `crate::x`. A second body's
/// `mod m` is a second importer, `crate::a::{block 2}::m`: the importer is part of a finding's identity, so two
/// block modules read alike would be one finding, and a baseline accepting it would hide the other.
#[test]
fn an_import_in_a_block_module_is_not_its_same_named_file_modules() {
    let probe = files_probe(
        "blockmodimport",
        "2021",
        &[
            ("src/lib.rs", "pub mod a;\npub mod x;\n"),
            ("src/x.rs", "pub struct Y;\n"),
            (
                "src/a.rs",
                "pub mod m;\n#[allow(unused_imports, dead_code)]\nfn f() { mod m { use crate::x::Y; } }\n\
                 #[allow(unused_imports, dead_code)]\nfn g() { mod n { use super::super::x::Y; } }\n\
                 #[allow(unused_imports, dead_code)]\nfn h() { mod m { use crate::x::Y; } }\n",
            ),
            ("src/a/m.rs", "\n"),
        ],
        &[],
    );
    let findings = |module: &str| {
        let law = Constitution::new("block-mod-import").boundary(
            ModuleBoundary::in_crate("blockmodimport")
                .module("crate::x")
                .must_not_be_imported_by(module)
                .because("x stays out of a"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => report
                .violations
                .iter()
                .map(|v| v.finding.clone())
                .collect::<Vec<_>>(),
            Outcome::Clean(_) => Vec::new(),
            other => panic!("{module}: {other:?}"),
        }
    };
    assert_eq!(findings("crate::a::m"), Vec::<String>::new());
    assert_eq!(
        findings("crate::a"),
        [
            "crate::a::{block 2}::m",
            "crate::a::{block}::m",
            "crate::a::{block}::n"
        ]
    );
}

/// A target is read in its own edition, not its package's: a 2024 package whose `[lib]` declares `edition = "2015"`
/// compiles its library as 2015, where `use clock::now;` in a submodule starts at the crate root, so a rule
/// forbidding `crate::sub` from importing `crate::clock` reports it. Targets sharing one root in two editions the
/// scanner reads apart compile it twice, which one reading cannot judge, so the check refuses; in 2018 and 2021, which it
/// reads alike, the root is judged — cargo 1.96.0 builds both targets of such a manifest.
#[test]
fn a_target_is_read_in_its_own_edition() {
    let files = [
        ("src/lib.rs", "pub mod clock;\npub mod sub;\n"),
        ("src/clock.rs", "pub fn now() {}\n"),
        ("src/sub.rs", "#[allow(unused_imports)]\nuse clock::now;\n"),
    ];
    let law = |package: &str| {
        Constitution::new("target-edition").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::sub")
                .must_not_import("crate::clock")
                .because("sub does not reach the clock"),
        )
    };
    let own = RootProbe::with_edition(
        "libedition",
        "2024",
        "[lib]\nedition = \"2015\"\npath = \"src/lib.rs\"\n",
        &files,
    );
    match check(&law("libedition"), own.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert_eq!(found, ["crate::clock::now"], "{report:?}");
        }
        other => panic!("a 2015 library target is read in 2015: {other:?}"),
    }
    let shared = RootProbe::with_edition(
        "sharededition",
        "2024",
        "[lib]\nedition = \"2015\"\npath = \"src/lib.rs\"\n[[bin]]\nname = \"sharededition-bin\"\npath = \"src/lib.rs\"\n",
        &files,
    );
    let outcome = check(&law("sharededition"), shared.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(
        format!("{outcome:?}").contains("roots targets in editions 2015, 2024"),
        "{outcome:?}"
    );
    let alike = RootProbe::with_edition(
        "alikeedition",
        "2021",
        "[lib]\nedition = \"2018\"\npath = \"src/lib.rs\"\n[[bin]]\nname = \"alikeedition-bin\"\npath = \"src/lib.rs\"\n",
        &[
            (
                "src/lib.rs",
                "pub mod clock;\npub mod sub;\n#[allow(dead_code)]\nfn main() {}\n",
            ),
            ("src/clock.rs", "pub fn now() {}\n"),
            (
                "src/sub.rs",
                "#[allow(unused_imports)]\nuse crate::clock::now;\n",
            ),
        ],
    );
    match check(&law("alikeedition"), alike.manifest()) {
        Outcome::Violations(report) => assert_eq!(report.violations.len(), 1, "{report:?}"),
        other => panic!("a root 2018 and 2021 targets share is judged: {other:?}"),
    }
}

/// A proc-macro crate's extern prelude holds `proc_macro`, so `proc_macro::TokenStream::new()` written with no
/// `extern crate proc_macro;` names the crate, as it does with one: rustc 1.96.0, edition 2021, compiles both, and a
/// call outside the permitted module reports in each.
#[test]
fn a_proc_macro_crate_names_proc_macro_without_an_extern_crate() {
    for (package, extern_crate) in [
        ("procmacrobare", ""),
        ("procmacroextern", "extern crate proc_macro;\n"),
    ] {
        let probe = RootProbe::new(
            package,
            "[lib]\nproc-macro = true\n",
            &[
                (
                    "src/lib.rs",
                    &format!(
                        "{extern_crate}mod allowed;\nmod inner {{ pub fn bad() -> proc_macro::TokenStream {{ \
                         proc_macro::TokenStream::new() }} }}\n#[proc_macro]\npub fn m(_i: proc_macro::TokenStream) \
                         -> proc_macro::TokenStream {{ let _ = allowed::ok(); inner::bad() }}\n"
                    ),
                ),
                (
                    "src/allowed.rs",
                    "pub fn ok() -> proc_macro::TokenStream { proc_macro::TokenStream::new() }\n",
                ),
            ],
        );
        let law = Constitution::new("proc-macro-prelude").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::allowed")
                .confine_inline_call("proc_macro::TokenStream")
                .because("only allowed builds token streams"),
        );
        let outcome = check(&law, probe.manifest());
        match &outcome {
            Outcome::Violations(report) => assert_eq!(
                report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect::<Vec<_>>(),
                ["proc_macro::TokenStream::new in crate"],
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected the call outside allowed to react, got {other:?}"),
        }
    }
}

/// Two cfg-exclusive `#[path]` files of one module each number their blocks from their own start, so a block is
/// named by its file as well as its number: a path through the module `a_unix.rs` declares in a function body
/// reaches that body, whatever the other file holds.
#[test]
fn blocks_of_two_files_of_one_module_are_two_blocks() {
    let probe = RootProbe::new(
        "blocksoftwofiles",
        "",
        &[
            (
                "src/lib.rs",
                "#[cfg_attr(unix, path = \"a_unix.rs\")]\n#[cfg_attr(not(unix), path = \"a_other.rs\")]\npub mod a;\n",
            ),
            (
                "src/a_unix.rs",
                "pub fn g() { mod m { pub use std::fs::read; } let _ = m::read(\"x\"); }\n",
            ),
            ("src/a_other.rs", "pub fn g() { let _ = 1; }\n"),
        ],
    );
    let found = ["std::fs::read in crate::a"];
    assert_inline_answers(
        &probe,
        "blocksoftwofiles",
        "crate",
        "std::fs",
        &found,
        &found,
    );
}

/// An attribute's arguments can hold a call: an attribute macro reads them as its own input, and
/// `#[attr::instrument(fields(t = std::process::id()))]` passes the call to one. So an attribute's contents are
/// read as tokens are, past the attribute's own path and a `cfg` or `cfg_attr` predicate, which name
/// configuration rather than code; `#[cfg_attr(all(), derive(Debug))]` beside it reports nothing.
#[test]
fn a_call_in_an_attributes_arguments_reports() {
    let probe = RootProbe::new(
        "attrcall",
        "[dependencies]\nattr = { path = \"attr\" }\n",
        &[
            (
                "src/lib.rs",
                "#[attr::instrument(fields(t = std::process::id()))]\npub fn g() {}\n\
                 #[cfg_attr(all(), derive(Debug))]\npub struct S;\n",
            ),
            (
                "attr/Cargo.toml",
                "[package]\nname = \"attr\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\nproc-macro = true\n",
            ),
            (
                "attr/src/lib.rs",
                "use proc_macro::TokenStream;\n#[proc_macro_attribute]\n\
                 pub fn instrument(_args: TokenStream, item: TokenStream) -> TokenStream { item }\n",
            ),
        ],
    );
    let found = ["std::process::id in crate"];
    assert_inline_answers(&probe, "attrcall", "crate", "std::process", &found, &found);
}

/// The module graph is walked in its target's edition, here the package's: in edition 2015 `mod dyn;` declares the file module
/// `crate::dyn`, so a rule forbidding it to import `crate::a` governs `src/dyn.rs` and reports its import.
#[test]
fn the_module_graph_is_walked_in_the_targets_edition() {
    let probe = files_probe(
        "editionwalk",
        "2015",
        &[
            ("src/lib.rs", "pub mod a;\nmod dyn;\n"),
            ("src/a.rs", "pub struct X;\n"),
            ("src/dyn.rs", "#[allow(unused_imports)]\nuse crate::a::X;\n"),
        ],
        &[],
    );
    let law = Constitution::new("edition-walk").boundary(
        ModuleBoundary::in_crate("editionwalk")
            .module("crate::dyn")
            .must_not_import("crate::a")
            .because("dyn does not reach a"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert_eq!(found, ["crate::a::X"], "{report:?}");
        }
        other => panic!("{other:?}"),
    }
}

/// A turbofish nested in a turbofish is a type, not a call: `size_of::<crate::k::T::<u8>>()` closes both groups at
/// one `>>`, and the inner run ends inside the outer group, so the `(` after that group applies to `size_of` alone.
#[test]
fn a_turbofish_nested_in_a_turbofish_is_read_as_a_type() {
    assert_crate_answers(
        "nestedturbofish",
        "pub mod k { pub struct T<U>(pub U); }\npub fn g() -> usize { std::mem::size_of::<crate::k::T::<u8>>() }\n",
        "crate::k",
        &[],
    );
}

/// The items the shift-token rows share, compiled by rustc 1.96.0 with each row.
const SHIFT_ITEMS: &str = "#![allow(non_camel_case_types, dead_code)]\npub trait Tr { type md5x; }\n\
                           impl Tr for u8 { type md5x = u8; }\npub trait Tr2 {}\n\
                           pub fn g<T: Default>() -> T { T::default() }\n";

/// What a confinement on the dependency `md5x` reports over one shift-token row, under `.strict_external()` and
/// under `.strict_prefix_only().strict_external()`.
fn shift_row_answers(package: &str, row: &str) -> [Vec<String>; 2] {
    let lib = format!("{SHIFT_ITEMS}{row}");
    let probe = files_probe(package, "2021", &[("src/lib.rs", &lib)], &["md5x"]);
    [false, true].map(|prefix_only| {
        let draft = ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_call_inline("md5x")
            .strict_external();
        let draft = if prefix_only {
            draft.strict_prefix_only()
        } else {
            draft
        };
        match check(
            &Constitution::new("shift-qualified").boundary(draft.because("inline path resolution")),
            probe.manifest(),
        ) {
            Outcome::Clean(_) => Vec::new(),
            Outcome::Violations(report) => report
                .violations
                .iter()
                .map(|v| v.finding.clone())
                .collect(),
            other => panic!("{package}: {other:?}"),
        }
    })
}

/// A turbofish opened by `<<` holds a qualified path: in `a::<<u8 as Tr>::md5x>()` the inner `<` opens
/// `<u8 as Tr>`, so `::md5x` is its tail, left to the receiver-method bound — not a rooted mention naming the
/// dependency `md5x`, which a confinement judging every mention and every external root would report.
#[test]
fn a_qualified_path_opened_by_a_shift_token_keeps_its_tail() {
    let answers = shift_row_answers(
        "shiftturbofish",
        "pub fn a<T>() {}\npub fn h() { a::<<u8 as Tr>::md5x>(); }\n",
    );
    assert_eq!(answers, [Vec::<String>::new(), Vec::new()]);
}

/// A `<<` opening a generic list, `Vec<<u8 as Tr>::md5x>`, is read as a shift: the token before it ends an operand,
/// and `1 << n > ::std::process::id() && n > 0` is written the same way, so the `::md5x` after the inner `>` is read
/// as a rooted path. It is a mention, so it reports only under `.strict_prefix_only()`, and names the dependency
/// `md5x` only under `.strict_external()` — a declared over-reaction, in each position rustc compiles one.
#[test]
fn a_qualified_path_a_shift_opens_in_a_generic_list_is_read_as_a_rooted_path() {
    let mut mismatches = Vec::new();
    for (package, row) in [
        (
            "shiftlet",
            "pub fn a() { let _v: Vec<<u8 as Tr>::md5x> = g(); }\n",
        ),
        ("shiftparam", "pub fn f(_x: Vec<<u8 as Tr>::md5x>) {}\n"),
        ("shiftalias", "pub type A = Vec<<u8 as Tr>::md5x>;\n"),
        ("shiftimpl", "impl Tr2 for Vec<<u8 as Tr>::md5x> {}\n"),
    ] {
        let answers = shift_row_answers(package, row);
        if answers != [Vec::new(), vec!["md5x in crate".to_string()]] {
            mismatches.push(format!("{package}: {answers:?}"));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A literal takes a `.` only straight after its digits, as rustc reads it: after a suffix, `1u8. max(2)` is a method
/// call on `1u8`, and in `t.0. clone()` the `.` after the tuple index is a method call on the field. Neither `max` nor
/// `clone` is a bare call of the module's own function of that name.
#[test]
fn a_dot_after_a_suffix_or_a_tuple_index_calls_a_method() {
    file_module_answers(
        "literaldotmethod",
        "pub fn max(_: u8) -> u8 { 0 }\npub fn clone(_: u8) -> u8 { 0 }\n\
         pub fn lit(t: (u8, u8)) -> u8 { let _ = t.0. clone(); 1u8. max(2) }\n",
        "crate::m",
        &[],
    );
}

/// An `impl` whose self type is a braced macro invocation still has a member body: in
/// `impl crate::Tr2 for t!{} { fn g() {} fn h() { g() } }`, `g()` is the imported `crate::k::g`.
#[test]
fn an_impl_for_a_braced_macro_type_binds_none_of_its_members() {
    assert_crate_answers(
        "implforbracedmacro",
        "pub mod k { pub fn g() {} }\nmacro_rules! t { () => { crate::S } }\npub struct S;\n\
         pub trait Tr2 { fn g(); fn h(); }\nmod inner {\n    use crate::k::g;\n    \
         impl crate::Tr2 for t!{} { fn g() {} fn h() { g() } }\n}\n",
        "crate::k",
        &["crate::k::g in crate"],
    );
}

/// A macro invocation inside an item — in an array's elements, an array type's length, a call's argument —
/// generates no item, so none makes an import of a `fn X` from its module hold the type namespace.
#[test]
fn a_macro_inside_any_item_generates_no_namespace() {
    let mut mismatches = Vec::new();
    for (package, item) in [
        ("macroinarray", "pub const C: [usize; 2] = [m!(); 2];"),
        ("macroinlength", "pub static A: [u8; m!()] = [0];"),
        (
            "macroinargument",
            "pub const D: usize = f(concat!(\"a\"));\n    pub const fn f(_: &str) -> usize { 0 }",
        ),
    ] {
        let lib = format!(
            "#![allow(non_snake_case)]\npub mod c {{\n    macro_rules! m {{ () => {{ 1 }} }}\n    pub fn X() {{}}\n    {item}\n}}\n\
             pub mod p {{ pub struct X; impl X {{ pub fn f() {{}} }} }}\n\
             pub mod support {{ pub use crate::c::X; pub use crate::p::*; }}\npub mod core2;\n"
        );
        let probe = files_probe(
            package,
            "2021",
            &[
                ("src/lib.rs", &lib),
                (
                    "src/core2.rs",
                    "use crate::support::X;\npub fn g() { X::f(); }\n",
                ),
            ],
            &[],
        );
        answer_mismatches(
            &probe,
            package,
            "crate::core2",
            "crate::p",
            &["crate::p::X::f in crate::core2"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// In edition 2015 a `use` path is read from the crate root, written bare or after `::`, so `use kernel::Thing;` and
/// `use ::kernel::Thing;` in `crate::core` each import `crate::kernel::Thing`, and a rule forbidding `crate::core` to
/// import `crate::kernel` reports it.
#[test]
fn an_edition_2015_use_path_is_read_from_the_crate_root() {
    for (package, written) in [
        ("edition2015use", "kernel::Thing"),
        ("edition2015globaluse", "::kernel::Thing"),
    ] {
        let core = format!("#[allow(unused_imports)]\nuse {written};\n");
        let probe = files_probe(
            package,
            "2015",
            &[
                ("src/lib.rs", "pub mod kernel;\npub mod core;\n"),
                ("src/kernel.rs", "pub struct Thing;\n"),
                ("src/core.rs", &core),
            ],
            &[],
        );
        let law = Constitution::new("edition-2015-use").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::core")
                .must_not_import("crate::kernel")
                .because("core does not reach kernel"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => {
                let found: Vec<&str> = report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect();
                assert_eq!(found, ["crate::kernel::Thing"], "{written}: {report:?}");
            }
            other => panic!("{written}: {other:?}"),
        }
    }
}

/// A derive's path in an attribute is read as any other tokens are: `#[derive(serx::Ser)]` mentions the dependency
/// `serx`, which only a confinement judging every mention and every external root reports. The `clippy` rows are a
/// control, not a pin: `#[allow(clippy::all)]` names no crate the package depends on, so they hold whatever the
/// attribute reader does.
#[test]
fn a_derive_path_in_an_attribute_is_a_mention() {
    let probe = RootProbe::new(
        "derivepath",
        "[dependencies]\nserx = { path = \"serx\" }\n",
        &[
            (
                "src/lib.rs",
                "#[derive(serx::Ser)]\n#[allow(clippy::all)]\npub struct S;\n",
            ),
            (
                "serx/Cargo.toml",
                "[package]\nname = \"serx\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[lib]\nproc-macro = true\n",
            ),
            (
                "serx/src/lib.rs",
                "use proc_macro::TokenStream;\n#[proc_macro_derive(Ser)]\n\
                 pub fn ser(_item: TokenStream) -> TokenStream { TokenStream::new() }\n",
            ),
        ],
    );
    let mut mismatches = Vec::new();
    for (prefix, prefix_only, strict_external, expected) in [
        ("serx", false, false, &[][..]),
        ("serx", false, true, &[][..]),
        ("serx", true, false, &[][..]),
        ("serx", true, true, &["serx::Ser in crate"][..]),
        ("clippy", false, true, &[][..]),
        ("clippy", true, true, &[][..]),
    ] {
        let mut draft = ModuleBoundary::in_crate("derivepath")
            .module("crate")
            .must_not_call_inline(prefix);
        if prefix_only {
            draft = draft.strict_prefix_only();
        }
        if strict_external {
            draft = draft.strict_external();
        }
        let found: Vec<String> = match check(
            &Constitution::new("derive-path").boundary(draft.because("inline path resolution")),
            probe.manifest(),
        ) {
            Outcome::Clean(_) => Vec::new(),
            Outcome::Violations(report) => report
                .violations
                .iter()
                .map(|v| v.finding.clone())
                .collect(),
            other => panic!("{prefix}: {other:?}"),
        };
        if found != expected {
            mismatches.push(format!(
                "{prefix}, prefix_only = {prefix_only}, strict_external = {strict_external}: {found:?}, expected {expected:?}"
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// One file a `#[path]` names twice is compiled twice, as two modules, and each is governed as its own: the scan
/// reads the file once per module, and a finding in `crate::b` is reported under `crate::b`, not under the first
/// module the file was read as. Compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_file_two_modules_share_is_judged_as_each() {
    let probe = RootProbe::new(
        "onefiletwomodules",
        "",
        &[
            (
                "src/lib.rs",
                "#[path = \"s.rs\"]\npub mod a;\n#[path = \"s.rs\"]\npub mod b;\n",
            ),
            ("src/s.rs", "pub fn f() -> u32 { std::process::id() }\n"),
        ],
    );
    for module in ["crate::a", "crate::b"] {
        let found = [format!("std::process::id in {module}")];
        let found: Vec<&str> = found.iter().map(String::as_str).collect();
        assert_inline_answers(
            &probe,
            "onefiletwomodules",
            module,
            "std::process",
            &found,
            &found,
        );
    }
}

/// A `use` whose alias is `_` imports its path and binds no name: `use crate::forbidden as _;` is an import of
/// `crate::forbidden` for the import rules, as `as f` is. The use-tree parser reads tokens, so `_` is the alias
/// rather than text fused onto `as`. Compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_use_aliased_to_underscore_is_an_import() {
    for alias in ["_", "f"] {
        let package = if alias == "_" {
            "useasunderscore"
        } else {
            "useasname"
        };
        let probe = files_probe(
            package,
            "2021",
            &[
                (
                    "src/lib.rs",
                    "pub mod forbidden { pub struct T; }\npub mod core;\n",
                ),
                (
                    "src/core.rs",
                    &format!("#[allow(unused_imports)]\nuse crate::forbidden as {alias};\n"),
                ),
            ],
            &[],
        );
        let law = Constitution::new("use-as-underscore").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::core")
                .must_not_import("crate::forbidden")
                .because("core does not reach forbidden"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => {
                let found: Vec<&str> = report
                    .violations
                    .iter()
                    .map(|v| v.finding.as_str())
                    .collect();
                assert_eq!(found, ["crate::forbidden"], "as {alias}: {report:?}");
            }
            other => panic!("as {alias}: {other:?}"),
        }
    }
}

/// In edition 2015 a `use` path starts at the crate root, so `use ::*;`, `use *;` and `use {*};` in `mod m` each glob
/// the crate root's names, and `time::Instant::now()` there reaches the root's `pub use std::time;`, reporting under `std::time`
/// beside the glob's own finding. The use-tree parser reads a glob with no path before it — `::*`, `*` or `{*}` — as a
/// glob of `::`, which the edition-2015 root dispatch names the crate root. Compiled by rustc 1.96.0, edition 2015.
#[test]
fn an_edition_2015_root_glob_brings_the_crate_roots_names() {
    let mut mismatches = Vec::new();
    for (package, glob) in [
        ("rootglob2015", "::*"),
        ("rootglobbare2015", "*"),
        ("rootglobgroup2015", "{*}"),
    ] {
        let probe = RootProbe::with_edition(
            package,
            "2015",
            "",
            &[(
                "src/lib.rs",
                &format!(
                    "pub use std::time;\nmod m {{ use {glob}; pub fn f() {{ let _ = time::Instant::now(); }} }}\n"
                ),
            )],
        );
        answer_mismatches(
            &probe,
            package,
            "crate",
            "std::time",
            &["glob crate in crate", "std::time::Instant::now in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `use` path is a uniform path from edition 2018 on: its first segment names what the scope it stands in binds or
/// declares before it names a crate, so in `crate::a` a `use inner::X;` beside `pub mod inner` imports
/// `crate::a::inner::X`, and a rule forbidding `crate::a` to import `crate::a::inner` reports it. The import scan
/// reads that head through the resolver the inline scan reads, rather than a crate-root-only rule of its own.
/// Compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_use_heads_its_path_from_the_scope_it_stands_in() {
    let probe = files_probe(
        "useuniformpath",
        "2021",
        &[
            ("src/lib.rs", "pub mod a;\n"),
            (
                "src/a.rs",
                "pub mod inner { pub struct X; }\n#[allow(unused_imports)]\nuse inner::X;\n",
            ),
        ],
        &[],
    );
    let law = Constitution::new("use-uniform-path").boundary(
        ModuleBoundary::in_crate("useuniformpath")
            .module("crate::a")
            .must_not_import("crate::a::inner")
            .because("a does not reach its inner module"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert_eq!(found, ["crate::a::inner::X"], "{report:?}");
        }
        other => panic!("{other:?}"),
    }
}

/// A brace group inside a `<…>` is a const argument or default, not the end of an item's header: `impl Tr for
/// S<{ 1 }> { … }` is an `impl` body, whose `fn exit` binds no bare head, so `exit(0)` there calls the module's
/// imported `std::process::exit` as `impl Tr for S<1>` does, and `enum E<const N: usize = { 1 }> { exit(i32) }` is an
/// `enum` body, whose `exit(i32)` declares a variant rather than calling the import. A comparison's `>` pairs with no
/// `<`, so `if a > b { use std::process::exit; exit(0) }` after a `use …;` is the `if`'s block, whose call reports. A
/// turbofish opened by `<-`, `f::<-1>()`, holds `-1`, so
/// the call after it applies to `f`. Each row is compiled by rustc 1.96.0, edition 2021.
#[test]
fn an_angle_group_holding_a_brace_or_a_negative_is_one_group() {
    let mut mismatches = Vec::new();
    for (package, lib, prefix, found) in [
        (
            "anglebraceimpl",
            "use std::process::exit;\npub struct S<const N: usize>;\npub trait Tr { fn exit(); fn run(); }\nimpl Tr for S<{ 1 }> { fn exit() {} fn run() { exit(0) } }\n",
            "std::process",
            &["std::process::exit in crate"][..],
        ),
        (
            "anglebraceenum",
            "#![allow(unused_imports, non_camel_case_types)]\nuse std::process::exit;\npub enum E<const N: usize = { 1 }> { exit(i32) }\n",
            "std::process",
            &[][..],
        ),
        (
            "anglecomparisonbody",
            "pub fn run(a: u8, b: u8) { use std::fmt::Debug; if a > b { use std::process::exit; exit(0) } }\n",
            "std::process",
            &["std::process::exit in crate"][..],
        ),
        (
            "anglenegative",
            "pub mod a { pub fn f<const N: i32>() {} }\npub fn g() { a::f::<-1>() }\n",
            "crate::a",
            &["crate::a::f in crate"][..],
        ),
    ] {
        let probe = lib_probe(package, lib);
        answer_mismatches(&probe, package, "crate", prefix, found, &mut mismatches);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A glob of an enum brings its variants, and a crate-rooted path reads on through what a module's globs bring, so
/// with `pub mod s { pub use crate::a::E::*; }` the call `crate::s::V(1)` is the variant `crate::a::E::V` and reports
/// under `crate::a` beside the glob's own finding. An enum has no scope the resolver reads, so its glob is read as a
/// crate's whose contents are not: a bare `V(1)` beside `use crate::a::E::*;` may name a prelude item instead, and
/// the glob's finding is what reacts for it. Compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_path_through_an_enum_glob_names_the_variant() {
    let mut mismatches = Vec::new();
    for (package, lib, found) in [
        (
            "enumglobpath",
            "pub mod a { pub enum E { V(u8) } }\npub mod s { pub use crate::a::E::*; }\npub fn g() { let _ = crate::s::V(1); }\n",
            &["crate::a::E::V in crate", "glob crate::a::E in crate"][..],
        ),
        (
            "enumglobbare",
            "pub mod a { pub enum E { V(u8) } }\nuse crate::a::E::*;\npub fn g() -> Option<u8> { let _ = V(1); Some(1) }\n",
            &["glob crate::a::E in crate"][..],
        ),
    ] {
        let probe = lib_probe(package, lib);
        answer_mismatches(&probe, package, "crate", "crate::a", found, &mut mismatches);
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A file opening with a shebang line reads its items after it: rustc strips `#!/usr/bin/env run`, so `pub mod core;`
/// below it declares `crate::core`, and a rule over `crate::core` judges its import. Compiled by rustc 1.96.0.
#[test]
fn a_crate_root_after_a_shebang_line_declares_its_modules() {
    let probe = files_probe(
        "shebangroot",
        "2021",
        &[
            (
                "src/lib.rs",
                "#!/usr/bin/env run\npub mod core;\npub mod x;\n",
            ),
            ("src/x.rs", "pub struct Y;\n"),
            (
                "src/core.rs",
                "#[allow(unused_imports)]\nuse crate::x::Y;\n",
            ),
        ],
        &[],
    );
    let law = Constitution::new("shebang").boundary(
        ModuleBoundary::in_crate("shebangroot")
            .module("crate::core")
            .must_not_import("crate::x")
            .because("core does not reach x"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => {
            let found: Vec<&str> = report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect();
            assert_eq!(found, ["crate::x::Y"], "{report:?}");
        }
        other => panic!("{other:?}"),
    }
}

/// A file-form `mod` a block declares with a `#[path]` is compiled, so its file is governed: `fn f() { #[path = "x.rs"]
/// mod m; … }` reads `x.rs` as `crate::{block}::m`, a second body's `#[path = "y.rs"] mod m;` as `crate::{block 2}::m`,
/// and a call in `x.rs` reports under `crate::{block}::m`. A path through `m` from the body that declares it reads that
/// file's scope, so `m::h()` through `x.rs`'s `pub use std::process::id as h;` reports under `std::process`. Compiled
/// by rustc 1.96.0, edition 2021.
#[test]
fn a_path_module_a_block_declares_is_governed() {
    let probe = RootProbe::new(
        "blockpathmod",
        "",
        &[
            (
                "src/lib.rs",
                "pub fn f() -> u32 { #[path = \"x.rs\"] mod m; m::h() }\npub fn g() -> u32 { #[path = \"y.rs\"] mod m; m::h() }\n",
            ),
            ("src/x.rs", "pub fn h() -> u32 { std::process::id() }\n"),
            ("src/y.rs", "pub fn h() -> u32 { 0 }\n"),
        ],
    );
    let found = ["std::process::id in crate::{block}::m"];
    assert_inline_answers(
        &probe,
        "blockpathmod",
        "crate",
        "std::process",
        &found,
        &found,
    );
    let through = RootProbe::new(
        "blockpathmodthrough",
        "",
        &[
            (
                "src/lib.rs",
                "pub fn f() -> u32 { #[path = \"x.rs\"] mod m; m::h() }\n",
            ),
            ("src/x.rs", "pub use std::process::id as h;\n"),
        ],
    );
    let found = ["std::process::id in crate"];
    assert_inline_answers(
        &through,
        "blockpathmodthrough",
        "crate",
        "std::process",
        &found,
        &found,
    );
}

/// A lattice of glob diamonds `layers` deep: each layer's two modules glob both of the next layer's, so a name the
/// crate root's glob of the first layer brings is reached along `2^layers` paths, and the last layer's modules each
/// define `leaf`.
fn glob_lattice(layers: usize) -> String {
    let mut lib = String::new();
    for i in 0..layers {
        for side in ["x", "y"] {
            if i + 1 < layers {
                lib.push_str(&format!(
                    "pub mod l{i}{side} {{ pub use crate::l{n}x::*; pub use crate::l{n}y::*; }}\n",
                    n = i + 1
                ));
            } else {
                lib.push_str(&format!("pub mod l{i}{side} {{ pub fn leaf() {{}} }}\n"));
            }
        }
    }
    lib.push_str("use l0x::*;\npub fn g() { leaf(); }\n");
    lib
}

/// What `read` answers, run on its own thread and awaited for ten seconds, a bound no reading the direction defends
/// against meets: a reading past it fails the direction naming `what`, rather than holding the suite until it
/// finishes.
fn answered_within<T: Send + 'static>(what: &str, read: impl FnOnce() -> T + Send + 'static) -> T {
    const BOUND: std::time::Duration = std::time::Duration::from_secs(10);
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = sender.send(read());
    });
    match receiver.recv_timeout(BOUND) {
        Ok(found) => found,
        Err(error) => panic!("{what} returned nothing within {BOUND:?}: {error}"),
    }
}

/// An import is resolved without itself, as rustc resolves it, so `pub use md5x::md5x;` names the dependency, and a chain
/// of forty modules each re-exporting the one before is read in time its length bounds: reading the import without
/// itself is the scope's answer for that lookup, kept like any other, rather than a cycle the walk cut, which would
/// leave every lookup along the chain unkept and each read once per namespace, twice per link. The call through the
/// last module still reports. rustc 1.96.0, edition 2021, builds the crate.
#[test]
fn a_chain_through_a_self_named_re_export_is_read_once_per_link() {
    let mut lib = String::from("pub mod m0 { pub use md5x::md5x; }\n");
    for i in 1..=40 {
        lib.push_str(&format!(
            "pub mod m{i} {{ pub use crate::m{}::md5x; }}\n",
            i - 1
        ));
    }
    lib.push_str("pub fn g() { crate::m40::md5x::f(); }\n");
    let found = answered_within("a forty-link self-named re-export chain", move || {
        let probe = RootProbe::new(
            "selfnamedchain",
            "[dependencies]\nmd5x = { path = \"md5x\" }\n",
            &[
                ("src/lib.rs", &lib),
                (
                    "md5x/Cargo.toml",
                    "[package]\nname = \"md5x\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
                ),
                ("md5x/src/lib.rs", "pub mod md5x { pub fn f() {} }\n"),
            ],
        );
        inline_findings(&probe, "selfnamedchain", "crate", "md5x", true)
    });
    assert_eq!(found, ["md5x::md5x::f in crate"]);
}

/// Two modules each globbing the other, neither binding `drop`, end the lookup of `drop` where it comes back to the
/// scope it went out from, rather than walking the pair without end; the call `b` makes beside them still reports,
/// under the file module both are written in.
/// rustc 1.96.0 builds the crate, edition 2021.
#[test]
fn a_lookup_through_two_modules_globbing_each_other_ends() {
    let found = answered_within("two modules globbing each other", || {
        let probe = lib_probe(
            "mutualglobs",
            "mod a { pub use crate::b::*; pub fn f() { drop(1u8); } }\n\
             mod b { pub use crate::a::*; pub fn g() -> u32 { std::process::id() } }\n\
             pub fn h() { a::f(); let _ = b::g(); }\n",
        );
        inline_findings(&probe, "mutualglobs", "crate", "std::process", true)
    });
    assert_eq!(found, ["std::process::id in crate"]);
}

/// Sixteen globs of standard modules at one crate root, each glob's own path headed by `std`, are read once each
/// rather than each through every subset of the others: a glob's path is read with every glob of the unit it passes
/// answered from one reading of them all, so what a head those globs could bring names is read in time the globs'
/// count bounds. A path through a module a glob brings still names what that module declares. rustc 1.96.0 builds
/// both crates, edition 2021.
#[test]
fn a_crate_roots_globs_read_through_each_other_are_read_once_each() {
    let modules = [
        "io",
        "fmt",
        "collections",
        "sync",
        "thread",
        "time",
        "env",
        "fs",
        "mem",
        "ptr",
        "cell",
        "rc",
        "iter",
        "ops",
        "cmp",
        "convert",
    ];
    let lib: String = modules
        .iter()
        .map(|m| format!("#[allow(unused_imports)] use std::{m}::*;\n"))
        .chain(["pub fn f() { drop(1u8); let _ = std::process::id(); }\n".to_string()])
        .collect();
    let found = answered_within("sixteen std globs", move || {
        let probe = lib_probe("stdglobs", &lib);
        inline_findings(&probe, "stdglobs", "crate", "std::process", true)
    });
    assert_eq!(found, ["std::process::id in crate"]);
    let probe = lib_probe(
        "globthroughglob",
        "pub mod m { pub mod x { pub fn f() {} } }\nuse m::*;\nuse x::*;\npub fn g() { f(); }\n",
    );
    assert_eq!(
        inline_findings(&probe, "globthroughglob", "crate", "crate::m::x", false),
        [
            "crate::m::x::f in crate",
            "glob crate::m in crate",
            "glob crate::m::x in crate"
        ]
    );
}

/// Each scope's answer for a name is read once per walk depth rather than once per path to it, so a lattice of glob
/// diamonds thirty layers deep — `2^30` paths to the last layer — resolves `leaf()` to the last layer's definition,
/// and the crate root's own glob reacts, within a bound no per-path reading meets:
/// measured in a debug build, forty layers took 0.26 s with the memo, and twenty took 23 s without it, doubling per
/// layer. The scan runs on its own thread and the bound is a receive timeout, so a per-path reading fails the
/// direction at the bound rather than holding the suite until it finishes.
#[test]
fn a_lattice_of_globs_is_read_once_per_scope() {
    const LAYERS: usize = 30;
    let found = answered_within(&format!("{LAYERS} layers"), move || {
        let probe = lib_probe("globlattice", &glob_lattice(LAYERS));
        inline_findings(
            &probe,
            "globlattice",
            "crate",
            &format!("crate::l{}x", LAYERS - 1),
            false,
        )
    });
    for expected in [
        format!("crate::l{}x::leaf in crate", LAYERS - 1),
        "glob crate::l0x in crate".to_string(),
    ] {
        assert!(found.contains(&expected), "{expected} among {found:?}");
    }
}

/// Rust resolves a head naming a `fn` parameter, a `let` binding or a closure parameter to that binding; the scanner
/// records none of them, so `now()` there is read through the module's `use crate::clock::now;`, or through the item
/// `now` the module declares — the declared over-reaction; under `.strict_prefix_only()` so is the name a `let`
/// introduces, `let now = 1u8;` beside `fn now`. Each row is compiled by rustc 1.96.0, edition 2021.
#[test]
fn a_local_binding_named_like_an_import_is_read_as_the_import() {
    let mut mismatches = Vec::new();
    for (package, body) in [
        ("localparam", "pub fn f(now: fn()) { now(); }"),
        ("locallet", "pub fn f() { let now = || (); now(); }"),
        (
            "localclosureparam",
            "pub fn f() { let g = |now: fn()| now(); g(|| ()); }",
        ),
    ] {
        let lib = format!(
            "pub mod clock {{ pub fn now() {{}} }}\n#[allow(unused_imports)] use crate::clock::now;\n{body}\n"
        );
        let probe = lib_probe(package, &lib);
        answer_mismatches(
            &probe,
            package,
            "crate",
            "crate::clock",
            &["crate::clock::now in crate"],
            &mut mismatches,
        );
        let package = format!("{package}item");
        let lib = format!("pub mod clock {{\n    pub fn now() {{}}\n    {body}\n}}\n");
        let probe = lib_probe(&package, &lib);
        let package = package.as_str();
        answer_mismatches(
            &probe,
            package,
            "crate",
            "crate::clock",
            &["crate::clock::now in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    let probe = lib_probe(
        "localletintroduced",
        "pub mod clock {\n    pub fn now() {}\n    pub fn h() {\n        let now = 1u8;\n    }\n}\n",
    );
    let law = Constitution::new("local-binding").boundary(
        ModuleBoundary::in_crate("localletintroduced")
            .module("crate")
            .must_not_call_inline("crate::clock")
            .strict_prefix_only()
            .depth(xuanji::ScanDepth::Subtree)
            .because("no clock in the crate"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => assert_eq!(
            report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect::<Vec<_>>(),
            ["crate::clock::now in crate"],
            "under strict the name a `let` introduces is read as the item it shares its name with"
        ),
        other => panic!("expected the introduced name to be read as the item, got {other:?}"),
    }
}

/// A block's import of what is not read may hold its name in the other namespace alone, so the name is also read
/// from the scope around the block: `use std::fmt;` names a module and no value, and rustc resolves `fmt()` in
/// `fn g() { use std::fmt; fmt(); }` to the module's `use crate::forbidden::fmt;`. Each row is compiled by rustc 1.96.0,
/// edition 2021.
#[test]
fn a_block_import_of_what_is_not_read_leaves_the_name_to_the_scope_around_it() {
    let mut mismatches = Vec::new();
    for (package, block) in [
        ("blockforeignuse", "use std::fmt;"),
        ("blockforeignself", "use std::fmt::{self};"),
    ] {
        let lib = format!(
            "pub mod forbidden {{ pub fn fmt() {{}} }}\nuse crate::forbidden::fmt;\npub fn g() {{ {block} fmt(); }}\n"
        );
        let probe = lib_probe(package, &lib);
        answer_mismatches(
            &probe,
            package,
            "crate",
            "crate::forbidden",
            &["crate::forbidden::fmt in crate"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A `{self}` leaf imports the module its group names and nothing else of that name, so a value of that name is read
/// from the scope around it: `use crate::local::both::{self};` in a block, with `local` declaring both `mod both` and
/// `fn both`, leaves `both()` to the module's `use crate::secret::both;`. rustc 1.96.0, edition 2021, calls
/// `crate::secret::both`.
#[test]
fn a_self_leaf_imports_its_module_and_no_value_of_that_name() {
    let package = "selfleafvalue";
    let probe = lib_probe(
        package,
        "pub mod secret { pub fn both() {} }\npub mod local { pub mod both {} pub fn both() {} }\n\
         #[allow(unused_imports)]\nuse crate::secret::both;\npub fn g() { use crate::local::both::{self}; both(); }\n",
    );
    let mut mismatches = Vec::new();
    answer_mismatches(
        &probe,
        package,
        "crate",
        "crate::secret",
        &["crate::secret::both in crate"],
        &mut mismatches,
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// The same reading where the block's import does hold the name: `std::process::id` is a function, and rustc calls it
/// in `fn g() -> u32 { use std::process::id; id() }`, while the scanner, which does not read `std`, also reads `id`
/// from the module's `use crate::forbidden::id;` — the declared over-reaction. rustc 1.96.0 compiles it, edition 2021.
#[test]
fn a_block_import_of_what_is_not_read_is_read_with_the_scope_around_it() {
    let package = "blockforeignholds";
    let probe = lib_probe(
        package,
        "pub mod forbidden { pub fn id() -> u32 { 0 } }\n#[allow(unused_imports)]\nuse crate::forbidden::id;\n\
         pub fn g() -> u32 { use std::process::id; id() }\n",
    );
    let mut mismatches = Vec::new();
    answer_mismatches(
        &probe,
        package,
        "crate",
        "crate::forbidden",
        &["crate::forbidden::id in crate"],
        &mut mismatches,
    );
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A scope that binds a name only through an import of what is not read may not hold it in the namespace a head is
/// read in, so the scope's globs are read too: in `crate::core`, `use crate::forbidden::*; use std::fmt;` and then
/// `fmt()` calls the glob's `crate::forbidden::fmt`, since `std::fmt` names a module and no value. Where the import
/// does hold the name — `use std::mem::swap;` beside the glob, with `swap` a function in both — rustc calls
/// `std::mem::swap`, and the scanner, which does not read `std`, reads the glob's `swap` too: the declared
/// over-reaction. rustc 1.96.0, edition 2021, builds each.
#[test]
fn an_import_of_what_is_not_read_beside_a_glob_is_read_with_the_glob() {
    for (package, forbidden, import, call, found) in [
        (
            "globbesidesysrootmodule",
            "pub fn fmt() {}",
            "std::fmt",
            "fmt();",
            "crate::forbidden::fmt in crate::core",
        ),
        (
            "globbesidesysrootvalue",
            "pub fn swap(_: &mut u8, _: &mut u8) {}",
            "std::mem::swap",
            "let (mut a, mut b) = (1u8, 2u8); swap(&mut a, &mut b);",
            "crate::forbidden::swap in crate::core",
        ),
    ] {
        let lib = format!("pub mod forbidden {{ {forbidden} }}\npub mod core;\n");
        let core = format!(
            "#[allow(unused_imports)]\nuse crate::forbidden::*;\n#[allow(unused_imports)]\nuse {import};\n\
             pub fn g() {{ {call} }}\n"
        );
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib), ("src/core.rs", &core)]);
        let mut mismatches = Vec::new();
        answer_mismatches(
            &probe,
            package,
            "crate::core",
            "crate::forbidden",
            &[found, "glob crate::forbidden in crate::core"],
            &mut mismatches,
        );
        assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    }
}

/// An import is an import of the path its head names, and a re-export the rest of the path runs through is not
/// followed: `use crate::support::x;` imports `crate::support` whatever `x` re-exports, so under
/// `restrict_imports_to(["crate::support"])` it is permitted though `support` holds `pub use crate::forbidden::x;`. A
/// head a glob brings is bound as the glob's module followed by the head: after `use crate::support::*;`, `use
/// fmod::x;` imports `crate::support::fmod::x` though `support` holds `pub use crate::forbidden as fmod;`. Each crate
/// is compiled by rustc 1.96.0, edition 2021.
#[test]
fn an_import_through_a_re_export_imports_the_module_it_names() {
    for (package, support, core) in [
        (
            "importthroughreexport",
            "pub use crate::forbidden::x;\n",
            "#[allow(unused_imports)]\nuse crate::support::x;\n",
        ),
        (
            "importthroughglob",
            "pub use crate::forbidden as fmod;\n",
            "#[allow(unused_imports)]\nuse crate::support::*;\n#[allow(unused_imports)]\nuse fmod::x;\n",
        ),
    ] {
        let probe = files_probe(
            package,
            "2021",
            &[
                (
                    "src/lib.rs",
                    "pub mod forbidden { pub fn x() {} }\npub mod support;\npub mod core;\n",
                ),
                ("src/support.rs", support),
                ("src/core.rs", core),
            ],
            &[],
        );
        let law = Constitution::new("import-through-reexport").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate::core")
                .restrict_imports_to(["crate::support"])
                .because("core reaches support alone"),
        );
        let outcome = check(&law, probe.manifest());
        assert!(
            matches!(outcome, Outcome::Clean(_)),
            "{package}: {outcome:?}"
        );
    }
}

/// A crate root's items after a byte-order mark, or after an inner attribute whose `#!` a nested block comment
/// separates from its `[`, are declared: rustc strips the mark, and reads `#! /* /* n */ */ [allow(dead_code)]` as
/// that inner attribute rather than a shebang line, so the `pub mod m;` beside it declares `crate::m`. Each root is
/// built by rustc 1.96.0, edition 2021.
#[test]
fn a_crate_root_after_a_byte_order_mark_or_a_commented_inner_attribute_declares_its_modules() {
    let mut mismatches = Vec::new();
    for (package, lib) in [
        ("rootbom", "\u{feff}pub mod m;\n"),
        (
            "rootnestedcomment",
            "#! /* /* n */ */ [allow(dead_code)] pub mod m;\n",
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", lib),
                ("src/m.rs", "pub fn g() -> u32 { std::process::id() }\n"),
            ],
        );
        answer_mismatches(
            &probe,
            package,
            "crate::m",
            "std::process",
            &["std::process::id in crate::m"],
            &mut mismatches,
        );
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

/// A file-form `mod` a block declares with only a `cfg_attr` path is compiled where the predicate holds, so its
/// file is governed: `fn f() { #[cfg_attr(unix, path = "x.rs")] mod m; m::h() }` builds on unix with rustc 1.96.0,
/// edition 2021, and a call in `x.rs` reports under `crate::{block}::m`. With no `x.rs`, no configuration builds it —
/// a block's file-form `mod` names no conventional file — and it is a scan error.
#[test]
fn a_cfg_attr_path_module_a_block_declares_is_governed() {
    let lib = "pub fn f() -> u32 { #[cfg_attr(unix, path = \"x.rs\")] mod m; m::h() }\n";
    let probe = RootProbe::new(
        "blockcfgattrmod",
        "",
        &[
            ("src/lib.rs", lib),
            ("src/x.rs", "pub fn h() -> u32 { std::process::id() }\n"),
        ],
    );
    let found = ["std::process::id in crate::{block}::m"];
    assert_inline_answers(
        &probe,
        "blockcfgattrmod",
        "crate",
        "std::process",
        &found,
        &found,
    );
    let absent = RootProbe::new("blockcfgattrmodabsent", "", &[("src/lib.rs", lib)]);
    let law = Constitution::new("block-cfg-attr-mod").boundary(
        ModuleBoundary::in_crate("blockcfgattrmodabsent")
            .module("crate")
            .must_not_call_inline("std::process")
            .because("no process calls"),
    );
    let outcome = check(&law, absent.manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
}

/// A module a block declares carries one path, read alike by the walk that reads its file and by the scope a path
/// through it names: `k::m::s()` in `fn g() { mod k { #[path = "y.rs"] pub mod m; } }` reaches the `s` that `y.rs`
/// re-exports from `crate::secret`, and `super::sx::go()` written in `y.rs` reaches `k`'s own `sx`. rustc 1.96.0, edition
/// 2021, builds both. An inline module a macro's group holds has no such path and is read without a panic.
#[test]
fn a_block_modules_file_and_a_path_through_it_carry_one_path() {
    for (package, lib, file, found) in [
        (
            "blockpathforward",
            "pub mod secret { pub fn go() {} }\npub fn g() { mod k { #[path = \"y.rs\"] pub mod m; } k::m::s(); }\n",
            "pub use crate::secret::go as s;\n",
            "crate::secret::go in crate",
        ),
        (
            "blockpathbackward",
            "pub mod secret { pub fn go() {} }\npub fn g() { mod k { #[path = \"y.rs\"] pub mod m; pub mod sx { pub use crate::secret::go; } } k::m::s(); }\n",
            "pub fn s() { super::sx::go(); }\n",
            "crate::secret::go in crate::{block}::k::m",
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", lib), ("src/k/y.rs", file)]);
        let law = Constitution::new("block-path").boundary(
            ModuleBoundary::in_crate(package)
                .module("crate")
                .must_not_call_inline("crate::secret")
                .depth(xuanji::ScanDepth::Subtree)
                .because("no call of the secret"),
        );
        match check(&law, probe.manifest()) {
            Outcome::Violations(report) => assert!(
                report.violations.iter().any(|v| v.finding == found),
                "{package}: {report:?}"
            ),
            other => panic!("{package}: expected {found:?}, got {other:?}"),
        }
    }
    let probe = lib_probe(
        "blockmacromodule",
        "macro_rules! id { ($($t:tt)*) => { $($t)* }; }\npub fn a() { id! { mod k1 { pub fn f() {} } } }\n",
    );
    let outcome = check(
        &root_scope_process_law("blockmacromodule"),
        probe.manifest(),
    );
    assert_eq!(outcome.exit_code(), 0, "{outcome:?}");
}

/// A path attribute on an inline module a block declares gives it a directory of its own, so a file-form `mod` inside
/// it needs no path attribute and is read from there: `fn f() { #[path = "d"] mod k { pub mod m; } }` reads `d/m.rs`,
/// and so do the same with `#[cfg_attr(all(), path = "d")]`, a nested `pub mod j { pub mod m; }` reading `d/j/m.rs`,
/// and an inline `k` whose inner `#[path = "d"] pub mod j` reads `k/d/m.rs`. rustc 1.96.0, edition 2021, builds each.
#[test]
fn a_path_attribute_gives_a_block_inline_module_a_directory_of_its_own() {
    let call = "pub fn s() { let _ = std::process::id(); }\n";
    for (i, (lib, file)) in [
        (
            "pub fn f() { #[path = \"d\"] mod k { pub mod m; } }\n",
            "src/d/m.rs",
        ),
        (
            "pub fn f() { #[cfg_attr(all(), path = \"d\")] mod k { pub mod m; } }\n",
            "src/d/m.rs",
        ),
        (
            "pub fn f() { #[path = \"d\"] mod k { pub mod j { pub mod m; } } }\n",
            "src/d/j/m.rs",
        ),
        (
            "pub fn f() { mod k { #[path = \"d\"] pub mod j { pub mod m; } } }\n",
            "src/k/d/m.rs",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let package = format!("blockpathdir{i}");
        let probe = RootProbe::new(&package, "", &[("src/lib.rs", lib), (file, call)]);
        let outcome = check(&root_scope_process_law(&package), probe.manifest());
        assert_eq!(
            reacting_files(&outcome),
            [probe.dir().join(file).display().to_string()],
            "{lib}: {outcome:?}"
        );
    }
}

/// Inside a block, an inline module compiled without its path attribute holds no file-form `mod` rustc accepts, so a
/// `cfg_attr` path is the only base its children are read from: with a crate-root `mod k;` occupying `src/k/`,
/// `fn f() { #[cfg_attr(unix, path = "d")] mod k { pub mod m; } }` reads `src/d/m.rs` and neither refuses a missing
/// `src/k/m.rs` nor reads one that exists. rustc 1.96.0, edition 2021, builds each row.
#[test]
fn a_cfg_attr_path_is_a_block_inline_modules_only_base() {
    let lib = "pub mod k;\npub fn f() { #[cfg_attr(unix, path = \"d\")] mod k { pub mod m; } k::m::s(); }\n";
    let call = "pub fn s() { let _ = std::process::id(); }\n";
    let unread = "pub fn s() { std::process::abort(); }\n";
    for (package, files) in [
        (
            "blockcfgbase",
            &[
                ("src/lib.rs", lib),
                ("src/k/mod.rs", ""),
                ("src/d/m.rs", call),
            ][..],
        ),
        (
            "blockcfgbaseshadow",
            &[
                ("src/lib.rs", lib),
                ("src/k/mod.rs", ""),
                ("src/d/m.rs", call),
                ("src/k/m.rs", unread),
            ][..],
        ),
    ] {
        let probe = RootProbe::new(package, "", files);
        let outcome = check(&root_scope_process_law(package), probe.manifest());
        assert_eq!(
            reacting_files(&outcome),
            [probe.dir().join("src/d/m.rs").display().to_string()],
            "{package}: {outcome:?}"
        );
    }
}

/// The `cfg` of the construct owning the block that holds the `mod` is read back to the previous `;`, `,`, brace group
/// or attribute, so an item, statement, parameter or field whose own tokens hold one of those before that block has
/// its `cfg` left unread, and the missing file is refused: the declared bound. rustc 1.96.0, edition 2021, builds each
/// row with no `x.rs`.
#[test]
fn a_cfg_before_a_separator_its_construct_holds_is_not_read() {
    for (i, body) in [
        "pub fn f() { #[cfg(any())] let _v: std::collections::HashMap<u8, u8> = { #[path = \"x.rs\"] mod m; std::collections::HashMap::new() }; }",
        "pub fn f() { #[cfg(any())] let _c = |_a: u8, _b: u8| { #[path = \"x.rs\"] mod m; }; }",
        "pub fn f(a: bool) { #[cfg(any())] if a {} else { #[path = \"x.rs\"] mod m; } }",
        "pub struct Foo { pub a: u8 }\npub fn f(v: Foo) { match v { #[cfg(any())] Foo { a } => { let _ = a; #[path = \"x.rs\"] mod m; } _ => {} } }",
        "pub struct S { pub a: u8 }\npub fn f() { #[cfg(any())] let _s = S { a: 1 }.a + { #[path = \"x.rs\"] mod m; 1 }; }",
        "#[cfg(any())] pub fn h<A, B>(_: [u8; { #[path = \"x.rs\"] mod m; 1 }]) {}",
        "#[cfg(any())] pub fn h<T>() where T: Copy, [u8; { #[path = \"x.rs\"] mod m; 1 }]: Sized {}",
        "pub struct S { #[cfg(any())] pub a: std::collections::HashMap<u8, [u8; { #[path = \"x.rs\"] mod m; 1 }]> }",
        "pub fn h(_a: u8, #[cfg(any())] _b: std::collections::HashMap<u8, [u8; { #[path = \"x.rs\"] mod m; 1 }]>) {}",
        "#[cfg(any())] pub struct T<A, B>(A, B, [u8; { #[path = \"x.rs\"] mod m; 1 }]);",
    ]
    .into_iter()
    .enumerate()
    {
        let package = format!("separatedcfg{i}");
        let probe = lib_probe(&package, &format!("{body}\n"));
        let outcome = check(&root_scope_process_law(&package), probe.manifest());
        assert_eq!(outcome.exit_code(), 2, "{body}: {outcome:?}");
    }
}

/// An inline `mod` a block declares is read, and its file-form children are followed from the declaring file's
/// directory: `fn f() { mod k { #[path = "y.rs"] pub mod m; } }` compiles `src/k/y.rs` from `lib.rs`, and the same
/// with a nested `j` in `a.rs` compiles `src/k/j/y.rs`, not `src/a/k/j/y.rs`, measured against rustc 1.96.0, edition
/// 2021. A file-form `mod` with no path attribute inside it is refused, as rustc refuses it ("cannot declare a file
/// module inside a block unless it has a path attribute"), and the refusal names the module by its path, the file
/// declaring it, and that it has no path attribute.
#[test]
fn an_inline_module_a_block_declares_is_read() {
    let call = "pub fn s() { let _ = std::process::id(); }\n";
    let root = RootProbe::new(
        "blockinline",
        "",
        &[
            (
                "src/lib.rs",
                "pub fn f() { mod k { #[path = \"y.rs\"] pub mod m; } k::m::s(); }\n",
            ),
            ("src/k/y.rs", call),
        ],
    );
    let outcome = check(&root_scope_process_law("blockinline"), root.manifest());
    assert_eq!(
        reacting_files(&outcome),
        [root.dir().join("src/k/y.rs").display().to_string()],
        "{outcome:?}"
    );
    let nested = RootProbe::new(
        "blockinlinenested",
        "",
        &[
            ("src/lib.rs", "pub mod a;\n"),
            (
                "src/a.rs",
                "pub fn f() { mod k { pub mod j { #[path = \"y.rs\"] pub mod m; } } k::j::m::s(); }\n",
            ),
            ("src/k/j/y.rs", call),
        ],
    );
    let outcome = check(
        &root_scope_process_law("blockinlinenested"),
        nested.manifest(),
    );
    assert_eq!(
        reacting_files(&outcome),
        [nested.dir().join("src/k/j/y.rs").display().to_string()],
        "{outcome:?}"
    );
    let plain = RootProbe::new(
        "blockinlineplain",
        "",
        &[
            (
                "src/lib.rs",
                "pub fn f() { mod k { pub mod m; } k::m::s(); }\n",
            ),
            ("src/k/m.rs", "pub fn s() {}\n"),
        ],
    );
    match check(
        &root_scope_process_law("blockinlineplain"),
        plain.manifest(),
    ) {
        Outcome::ConstitutionError(message) => assert!(
            message.contains(&plain.dir().join("src/lib.rs").display().to_string())
                && message.contains("`crate::{block}::k::m`")
                && message.contains("with no path attribute")
                && !message.contains("cfg_attr"),
            "{message}"
        ),
        other => panic!("expected a scan error, got {other:?}"),
    }
}

/// A `mod` whose file is absent is tolerated wherever something enclosing it may be compiled out, since rustc loads
/// nothing beneath what a `cfg` removes: a `fn`, a block statement, a match arm, a field, an inline module or a
/// `cfg_if!` arm carrying one,
/// bare or applied through `cfg_attr`, and a module a `cfg` removes whose file declares the `mod`. Each row builds with
/// no file backing the inner `mod`, and each control is refused by rustc for that file — among them a `cfg` on the
/// match arm or field before the one enclosing the `mod`, which removes that one alone, measured against rustc 1.96.0,
/// edition 2021, with `cfg-if` 1.0 for the arm rows. A file that does exist beneath a compiled-out item is still
/// read, so the call in it reports.
#[test]
fn an_absent_module_file_beneath_what_a_cfg_removes_is_tolerated() {
    let absent: &[(&str, &[(&str, &str)])] = &[
        (
            "a cfg on the enclosing fn",
            &[(
                "src/lib.rs",
                "#[cfg(any())]\nfn f() { #[path = \"x.rs\"] mod m; }\npub fn t() {}\n",
            )],
        ),
        (
            "a cfg on a block statement",
            &[(
                "src/lib.rs",
                "pub fn f() { #[cfg(any())] { #[path = \"x.rs\"] mod m; } }\n",
            )],
        ),
        (
            "a cfg applied through cfg_attr on a block statement",
            &[(
                "src/lib.rs",
                "pub fn f() { #[cfg_attr(all(), cfg(any()))] { #[path = \"x.rs\"] mod m; } }\n",
            )],
        ),
        (
            "a cfg_if arm inside a fn",
            &[(
                "src/lib.rs",
                "pub fn f() { cfg_if::cfg_if! { if #[cfg(any())] { #[path = \"x.rs\"] mod m; } } }\n",
            )],
        ),
        (
            "a fn inside a cfg_if arm",
            &[(
                "src/lib.rs",
                "cfg_if::cfg_if! { if #[cfg(any())] { fn f() { #[path = \"x.rs\"] mod m; } } }\npub fn t() {}\n",
            )],
        ),
        (
            "a cfg on the enclosing generic fn",
            &[(
                "src/lib.rs",
                "#[cfg(any())]\nfn f<A, B>() { #[path = \"x.rs\"] mod m; }\npub fn t() {}\n",
            )],
        ),
        (
            "a cfg on the enclosing match arm",
            &[(
                "src/lib.rs",
                "pub fn f(v: u8) { match v { #[cfg(any())] 0 => { #[path = \"x.rs\"] mod m; } _ => () } }\n",
            )],
        ),
        (
            "a cfg on the enclosing field",
            &[(
                "src/lib.rs",
                "pub struct S { pub a: u8 }\npub fn f() -> S { S { #[cfg(all())] a: { 1 }, #[cfg(any())] a: { #[path = \"x.rs\"] mod m; 2 } } }\n",
            )],
        ),
        (
            "a cfg on the enclosing inline module",
            &[(
                "src/lib.rs",
                "#[cfg(any())]\nmod o { mod i; }\npub fn t() {}\n",
            )],
        ),
        (
            "a cfg on the module whose file declares the mod",
            &[
                ("src/lib.rs", "#[cfg(any())]\nmod o;\npub fn t() {}\n"),
                ("src/o.rs", "fn f() { #[path = \"x.rs\"] mod m; }\nmod i;\n"),
            ],
        ),
    ];
    for (i, (shape, files)) in absent.iter().enumerate() {
        let package = format!("compiledout{i}");
        let probe = RootProbe::new(&package, "", files);
        let outcome = check(&root_scope_process_law(&package), probe.manifest());
        assert_eq!(outcome.exit_code(), 0, "{shape}: {outcome:?}");
    }
    let refused: &[(&str, &[(&str, &str)])] = &[
        (
            "no cfg anywhere",
            &[("src/lib.rs", "pub fn f() { #[path = \"x.rs\"] mod m; }\n")],
        ),
        (
            "a cfg_attr applying no cfg on the enclosing fn",
            &[(
                "src/lib.rs",
                "#[cfg_attr(unix, allow(dead_code))]\nfn f() { #[path = \"x.rs\"] mod m; }\npub fn t() {}\n",
            )],
        ),
        (
            "a cfg on the match arm before the enclosing one",
            &[(
                "src/lib.rs",
                "pub fn f(v: u8) { match v { #[cfg(any())] 1 => (), _ => { #[path = \"x.rs\"] mod m; } } }\n",
            )],
        ),
        (
            "a cfg on the field before the enclosing one",
            &[(
                "src/lib.rs",
                "pub struct S { pub a: u8, pub b: u8 }\npub fn f() -> S { S { #[cfg(all())] a: 1, b: { #[path = \"x.rs\"] mod m; 2 } } }\n",
            )],
        ),
        (
            "an unconditional module whose file declares the mod",
            &[
                ("src/lib.rs", "mod o;\npub fn t() {}\n"),
                ("src/o.rs", "mod i;\n"),
            ],
        ),
    ];
    for (i, (shape, files)) in refused.iter().enumerate() {
        let package = format!("notcompiledout{i}");
        let probe = RootProbe::new(&package, "", files);
        let outcome = check(&root_scope_process_law(&package), probe.manifest());
        assert_eq!(outcome.exit_code(), 2, "{shape}: {outcome:?}");
    }
    let present = RootProbe::new(
        "compiledoutpresent",
        "",
        &[
            (
                "src/lib.rs",
                "#[cfg(any())]\nfn f() { #[path = \"x.rs\"] mod m; }\npub fn t() {}\n",
            ),
            ("src/x.rs", "pub fn h() -> u32 { std::process::id() }\n"),
        ],
    );
    let outcome = check(
        &root_scope_process_law("compiledoutpresent"),
        present.manifest(),
    );
    assert_eq!(
        reacting_files(&outcome),
        [present.dir().join("src/x.rs").display().to_string()],
        "{outcome:?}"
    );
}

/// Every call under `std::process` in the whole of `package` reacts.
fn root_scope_process_law(package: &str) -> Constitution {
    Constitution::new("compiled-out").boundary(
        ModuleBoundary::in_crate(package)
            .module("crate")
            .must_not_call_inline("std::process")
            .depth(xuanji::ScanDepth::Subtree)
            .because("no process calls"),
    )
}

/// A list of forty thousand brace groups after a comparison, `[a < b, {0} > 0, {1} > 0, …]`, is read in time its length
/// bounds: whether a group is a const argument is read from one pairing of the tree's `<…>` groups rather than by
/// reading back from each group to the previous `;`, and a comparison's `<` pairs with nothing. The call after the
/// list still reports. rustc 1.96.0 builds the crate, edition 2021.
#[test]
fn a_long_list_of_brace_groups_is_read_once() {
    let items: Vec<String> = (0..40_000).map(|i| format!("{{{i}}}")).collect();
    let lib = format!(
        "pub fn f(a: u8, b: u8) -> u32 {{ let _x = [a < b, {}]; std::process::id() }}\n",
        items
            .iter()
            .map(|i| format!("{i} > 0"))
            .collect::<Vec<_>>()
            .join(", ")
    );
    let found = answered_within("forty thousand brace groups", move || {
        let probe = lib_probe("bracelist", &lib);
        inline_findings(&probe, "bracelist", "crate", "std::process", false)
    });
    assert_eq!(found, ["std::process::id in crate"]);
}

/// Six hundred inline modules each nested in the last, and two thousand side by side, are read in time the source's
/// size bounds: a file is lexed once however many inline modules it holds, and a `mod` beneath an inline module is
/// passed over at the first module body its walk out meets. The call in the innermost module, and in the last of the
/// side-by-side ones, still reports. rustc 1.96.0 builds both crates, edition 2021.
#[test]
fn many_inline_modules_are_read_once_each() {
    let nested = format!(
        "{}pub fn f() -> u32 {{ std::process::id() }}{}\n",
        "pub mod m { ".repeat(600),
        " }".repeat(600)
    );
    let found = answered_within("six hundred nested modules", move || {
        let probe = lib_probe("nestedmods", &nested);
        inline_findings(&probe, "nestedmods", "crate", "std::process", false)
    });
    assert_eq!(found, ["std::process::id in crate"]);
    let side_by_side: String = (0..2000)
        .map(|i| format!("pub mod m{i} {{ pub fn f() {{}} }}\n"))
        .chain(["pub mod last { pub fn f() -> u32 { std::process::id() } }\n".to_string()])
        .collect();
    let found = answered_within("two thousand modules side by side", move || {
        let probe = lib_probe("sidebysidemods", &side_by_side);
        inline_findings(&probe, "sidebysidemods", "crate", "std::process", false)
    });
    assert_eq!(found, ["std::process::id in crate"]);
}

/// A lookup through a chain of three hundred and one globs, the crate root globbing `m0`, each module globbing the
/// next and the last defining `leaf`, resolves `leaf()` to it: the scopes a lookup through globs reaches are read
/// into one graph without recursing through them, so no chain of globs meets the chain cap or the stack. rustc
/// 1.96.0 builds the crate, edition 2021.
#[test]
fn a_long_chain_of_globs_resolves() {
    const LINKS: usize = 300;
    let mut lib = String::from("#[allow(unused_imports)] use m0::*;\npub fn g() { leaf(); }\n");
    for i in 0..LINKS {
        lib.push_str(&format!(
            "pub mod m{i} {{ pub use crate::m{}::*; }}\n",
            i + 1
        ));
    }
    lib.push_str(&format!("pub mod m{LINKS} {{ pub fn leaf() {{}} }}\n"));
    let found = answered_within("a chain of three hundred and one globs", move || {
        let probe = lib_probe("globchain", &lib);
        inline_findings(
            &probe,
            "globchain",
            "crate",
            &format!("crate::m{LINKS}"),
            false,
        )
    });
    assert!(
        found.contains(&format!("crate::m{LINKS}::leaf in crate")),
        "{found:?}"
    );
}

/// Twelve thousand modules, each globbed at the crate root, are read in time the unit's size bounds: the glob hazard
/// reads the modules beneath a glob as the one run of the ordered module map the glob's path starts, rather than
/// testing every module of the unit against each glob. The call beside them still reports. rustc 1.96.0 builds the
/// crate, edition 2021.
#[test]
fn a_glob_hazard_reads_only_the_modules_beneath_its_glob() {
    let lib: String = (0..12_000)
        .map(|i| {
            format!("pub mod m{i} {{ pub fn f() {{}} }}\n#[allow(unused_imports)] use m{i}::*;\n")
        })
        .chain(["pub fn g() -> u32 { std::process::id() }\n".to_string()])
        .collect();
    let found = answered_within("twelve thousand globbed modules", move || {
        let probe = lib_probe("manyglobbedmodules", &lib);
        inline_findings(&probe, "manyglobbedmodules", "crate", "std::process", false)
    });
    assert_eq!(found, ["std::process::id in crate"]);
}

/// Modules globbing one another in a cycle are read once each for a name none of them binds: twelve child modules
/// each `pub(crate) use super::*;` beneath a crate root that `pub use`s every child's glob, and eight modules each
/// globbing all seven others, each calling `drop(1u8)` beside a call that reports. The scopes a lookup through globs
/// reaches are read into one graph and settled to a fixed point, rather than walked once per path through the cycle.
/// rustc 1.96.0 builds both crates, edition 2021.
#[test]
fn modules_globbing_one_another_are_read_once_each() {
    let children: String = (0..12)
        .map(|i| {
            format!(
                "#[allow(unused_imports)] pub use c{i}::*;\npub mod c{i} {{ #[allow(unused_imports)] pub(crate) use super::*; pub fn f{i}() {{ drop(1u8); }} }}\n"
            )
        })
        .chain(["pub fn g() -> u32 { drop(1u8); std::process::id() }\n".to_string()])
        .collect();
    let found = answered_within("twelve children globbing their parent", move || {
        let probe = lib_probe("childrenglobparent", &children);
        inline_findings(&probe, "childrenglobparent", "crate", "std::process", true)
    });
    assert_eq!(found, ["std::process::id in crate"]);
    let complete: String = (0..8)
        .map(|i| {
            let globs: String = (0..8)
                .filter(|&j| j != i)
                .map(|j| format!("#[allow(unused_imports)] pub use crate::k{j}::*; "))
                .collect();
            format!("pub mod k{i} {{ {globs}pub fn f{i}() {{ drop(1u8); }} }}\n")
        })
        .chain(["pub fn g() -> u32 { std::process::id() }\n".to_string()])
        .collect();
    let found = answered_within("eight modules each globbing the others", move || {
        let probe = lib_probe("completeglobs", &complete);
        inline_findings(&probe, "completeglobs", "crate", "std::process", true)
    });
    assert_eq!(found, ["std::process::id in crate"]);
}

/// A name bound in one namespace and brought by a glob in the other is read as both where an import names it: in
/// `crate::sub`, `x` is a `type` alias of `crate::m::T` and a function `crate::g::*` brings, so `pub use x as y;`
/// imports both, and under `restrict_imports_to(["crate::g"])` the alias's edge to `crate::m` reacts. Each namespace's
/// answer carries what the name alone names there, and the join of the two keeps both. rustc 1.96.0 builds the
/// crate, edition 2021.
#[test]
fn a_name_bound_in_one_namespace_and_brought_in_the_other_imports_both() {
    let probe = files_probe(
        "bothnamespaces",
        "2021",
        &[
            ("src/lib.rs", "pub mod m;\npub mod g;\npub mod sub;\n"),
            ("src/m.rs", "pub struct T;\n"),
            ("src/g.rs", "pub fn x() {}\n"),
            (
                "src/sub.rs",
                "#[allow(non_camel_case_types)]\npub type x = crate::m::T;\n#[allow(unused_imports)]\npub use crate::g::*;\n#[allow(unused_imports)]\npub use x as y;\n",
            ),
        ],
        &[],
    );
    let law = Constitution::new("both-namespaces").boundary(
        ModuleBoundary::in_crate("bothnamespaces")
            .module("crate::sub")
            .restrict_imports_to(["crate::g"])
            .because("sub reaches g alone"),
    );
    match check(&law, probe.manifest()) {
        Outcome::Violations(report) => assert!(
            report.violations.iter().any(|v| v.finding == "crate::m::T"),
            "{report:?}"
        ),
        other => panic!("expected the alias's edge to react, got {other:?}"),
    }
}

/// Globs each read through the one before them settle past the chain cap: nested modules `a0::a1::…`, the innermost
/// defining `leaf`, and a crate root globbing every one of them by its bare name, so each glob's head is a module the
/// glob before it brings. A hundred and fifty of them settle within a few passes whether written in order or in
/// reverse, since passes read the globs forwards and backwards in turn. Either way `leaf()` resolves. rustc 1.96.0 builds
/// both crates, edition 2021.
#[test]
fn globs_read_through_one_another_settle_however_long_their_chain() {
    for (package, depth, reversed) in [("globsinorder", 150, false), ("globsreversed", 150, true)] {
        let mut globs: Vec<String> = (0..depth)
            .map(|k| format!("#[allow(unused_imports)] use a{k}::*;\n"))
            .collect();
        if reversed {
            globs.reverse();
        }
        let modules = (0..depth).fold(String::from("pub fn leaf() {}"), |inner, k| {
            format!("pub mod a{} {{ {inner} }}", depth - 1 - k)
        });
        let lib = format!("{}pub fn g() {{ leaf(); }}\n{modules}\n", globs.concat());
        let prefix = format!(
            "crate::{}",
            (0..depth)
                .map(|k| format!("a{k}"))
                .collect::<Vec<_>>()
                .join("::")
        );
        let found = answered_within(package, move || {
            let probe = lib_probe(package, &lib);
            inline_findings(&probe, package, "crate", &prefix, false)
        });
        assert!(
            found.iter().any(|f| f.ends_with("::leaf in crate")),
            "{package}: {found:?}"
        );
    }
}

/// A glob's own path is never read through the glob itself: in `d`, `use self::hash_map::*;` names the
/// `hash_map` that `use std::collections::*;` brings, and reading it through what it brings in turn would name
/// `…::hash_map::hash_map` and grow every pass without settling. So the unit is judged and the call in `d`
/// reports. rustc 1.96.0 builds the crate, edition 2021.
#[test]
fn a_glob_is_never_read_through_itself() {
    let probe = lib_probe(
        "globthroughitself",
        "pub mod d {\n    use std::collections::*;\n    use self::hash_map::*;\n    pub fn f() -> Option<Entry<'static, u8, u8>> {\n        let _ = std::process::id();\n        None\n    }\n}\n",
    );
    assert_eq!(
        inline_findings(&probe, "globthroughitself", "crate", "std::process", true),
        ["std::process::id in crate"]
    );
}

/// Before edition 2021 a `c` before a string literal is an identifier, not a C string's prefix, so `cr#"x"` is `cr`, `#`
/// and the string `"x"`, and the code after it is read: in an edition-2018 package `m!(cr#"x");` before a call of
/// `std::fs::canonicalize` and `m!("#");` after it, the call reports. Read as a raw C string it ran to the `"#` and took
/// the call with it. rustc 1.96.0 builds the crate in edition 2018 and refuses it in 2021.
#[test]
fn a_c_before_a_string_is_an_identifier_before_edition_2021() {
    let probe = files_probe(
        "cstring2018",
        "2018",
        &[(
            "src/lib.rs",
            "macro_rules! m { ($($t:tt)*) => {}; }\npub fn g() { m!(cr#\"x\"); let _ = std::fs::canonicalize(\".\"); m!(\"#\"); }\n",
        )],
        &[],
    );
    let found = ["std::fs::canonicalize in crate"];
    assert_inline_answers(&probe, "cstring2018", "crate", "std::fs", &found, &found);
}

/// A crate-rooted path keeps what a glob of a crate whose contents are not read brings beside what the unit's own
/// modules bind, since under exclusive cfgs either can be the live one: `crate::m::Command::new` reports under
/// `std::process` where `m` globs `crate::a::*` under one `cfg` and `std::process::*` under its negation, where `m` is
/// two files under exclusive `cfg`s, one declaring `Command` and one globbing `std::process`, and — read as a path
/// mentioned, in both namespaces — where `m` declares a `struct exit` beside its glob of `std::process`. rustc 1.96.0,
/// edition 2021, builds each.
#[test]
fn a_crate_rooted_path_keeps_a_foreign_globs_candidate_beside_a_local_one() {
    let user = concat!(
        "pub fn f() { let _ = crate::m::",
        command_new_call!(),
        "; }\n"
    );
    let found = ["std::process::Command::new in crate::user"];
    let globs = RootProbe::new(
        "foreignbesidelocalglob",
        "",
        &[
            (
                "src/lib.rs",
                "#![allow(unused)]\npub mod a { pub struct Command; impl Command { pub fn new(_: &str) -> Self { Command } } }\n\
                 pub mod m {\n    #[cfg(any())]\n    pub use crate::a::*;\n    #[cfg(not(any()))]\n    pub use std::process::*;\n}\npub mod user;\n",
            ),
            ("src/user.rs", user),
        ],
    );
    assert_inline_answers(
        &globs,
        "foreignbesidelocalglob",
        "crate::user",
        "std::process",
        &found,
        &found,
    );
    let files = RootProbe::new(
        "foreignbesidelocalfile",
        "",
        &[
            (
                "src/lib.rs",
                "#![allow(unused)]\n#[cfg(any())]\n#[path = \"m_a.rs\"]\npub mod m;\n#[cfg(not(any()))]\n#[path = \"m_b.rs\"]\npub mod m;\npub mod user;\n",
            ),
            (
                "src/m_a.rs",
                "pub struct Command;\nimpl Command { pub fn new(_: &str) -> Self { Command } }\n",
            ),
            ("src/m_b.rs", "pub use std::process::*;\n"),
            ("src/user.rs", user),
        ],
    );
    assert_inline_answers(
        &files,
        "foreignbesidelocalfile",
        "crate::user",
        "std::process",
        &found,
        &found,
    );
    let namespaces = RootProbe::new(
        "foreignbesidelocaltype",
        "",
        &[
            (
                "src/lib.rs",
                "#![allow(unused, non_camel_case_types)]\npub mod m { pub struct exit {} pub use std::process::*; }\npub mod user;\n",
            ),
            (
                "src/user.rs",
                "pub fn f() { let g: fn(i32) -> ! = crate::m::exit; }\n",
            ),
        ],
    );
    let law = Constitution::new("foreignbesidelocaltype").boundary(
        ModuleBoundary::in_crate("foreignbesidelocaltype")
            .module("crate::user")
            .must_not_call_inline("std::process")
            .strict_prefix_only()
            .because("no mention of processes"),
    );
    match check(&law, namespaces.manifest()) {
        Outcome::Violations(report) => assert_eq!(
            report
                .violations
                .iter()
                .map(|v| v.finding.as_str())
                .collect::<Vec<_>>(),
            ["std::process::exit in crate::user"]
        ),
        other => panic!("expected the mention through `crate::m::exit` to react, got {other:?}"),
    }
}

/// A name a scope binds or declares only by what a `cfg` gates is read through the scope's globs too, since a build
/// that compiles the gated item out reads the name from the glob: beside `use super::*;`, over a crate root's private
/// `use std::process::Command;`, a `Command::new` call reports under `std::process` where `Command` is also imported
/// under `#[cfg(test)]`, imported in a `cfg_if!` arm, or declared under `#[cfg(any())]`. Imported with no `cfg`, the
/// import shadows the glob on every build and nothing reports. rustc 1.96.0, edition 2021, builds each, the `cfg_if!`
/// one with the `cfg-if` crate.
#[test]
fn a_name_bound_only_where_a_cfg_gates_it_is_read_through_the_scopes_globs() {
    let call = concat!("    pub fn f() { let _ = ", command_new_call!(), "; }\n");
    for (package, shadow, found) in [
        (
            "cfggatedimport",
            "    #[cfg(test)]\n    use crate::mock::Command;\n",
            &["std::process::Command::new in crate"][..],
        ),
        (
            "cfggatedarm",
            "    cfg_if::cfg_if! {\n        if #[cfg(test)] {\n            use crate::mock::Command;\n        }\n    }\n",
            &["std::process::Command::new in crate"][..],
        ),
        (
            "cfggateditem",
            "    #[cfg(any())]\n    struct Command;\n",
            &["std::process::Command::new in crate"][..],
        ),
        ("ungatedimport", "    use crate::mock::Command;\n", &[][..]),
    ] {
        assert_crate_answers(
            package,
            &format!(
                "#![allow(unused)]\nuse std::process::Command;\npub mod mock {{ pub struct Command; impl Command {{ pub fn new(_: &str) -> Self {{ Command }} }} }}\npub mod agent {{\n    use super::*;\n{shadow}{call}}}\n"
            ),
            "std::process",
            found,
        );
    }
}

/// Whether a scope's answer ends a lookup is one judgement wherever the lookup meets the scope: a name a scope holds
/// only by a `cfg`-gated item does not end it, so a block holding `#[cfg(any())] use crate::mock::now;`, or a gated
/// `fn now() {}` of its own, leaves `now()` to the module's `use crate::clock::now;` — or to a `use super::*;` beside
/// that gated item in the same block, which brings the parent's private import — and a module `relay` holding a gated `pub use crate::mock::now;` beside
/// `pub use crate::clock::*;` brings `clock`'s `now` through `bridge`'s glob of it to `use crate::bridge::now;`. Each
/// went unreported, the block case read where the lookup walks out of a block and the relay case where a lookup
/// through globs reaches a scope. With the gated item written ungated, it shadows the outer name and nothing reports.
/// rustc 1.96.0, edition 2021, builds each.
#[test]
fn a_scope_holding_a_name_only_where_a_cfg_gates_it_ends_no_lookup() {
    let items = "#![allow(unused)]\npub mod clock { pub fn now() {} }\npub mod mock { pub fn now() {} }\npub mod core;\n";
    let relay = |gate: &str| {
        format!(
            "pub mod relay {{\n    {gate}pub use crate::mock::now;\n    pub use crate::clock::*;\n}}\n\
             pub mod bridge {{\n    pub use crate::relay::*;\n}}\n"
        )
    };
    for (package, lib, core, found) in [
        (
            "gatedblockshadow",
            items.to_string(),
            "use crate::clock::now;\npub fn run() {\n    #[cfg(any())]\n    use crate::mock::now;\n    now();\n}\n",
            &["crate::clock::now in crate::core"][..],
        ),
        (
            "ungatedblockshadow",
            items.to_string(),
            "use crate::clock::now;\npub fn run() {\n    use crate::mock::now;\n    now();\n}\n",
            &[][..],
        ),
        (
            "gatedblockitem",
            items.to_string(),
            "use crate::clock::now;\npub fn run() {\n    #[cfg(any())]\n    fn now() {}\n    now();\n}\n",
            &["crate::clock::now in crate::core"][..],
        ),
        (
            "ungatedblockitem",
            items.to_string(),
            "use crate::clock::now;\npub fn run() {\n    fn now() {}\n    now();\n}\n",
            &[][..],
        ),
        (
            "gatedblockitemglob",
            items.to_string(),
            "use crate::clock::now;\npub mod inner {\n    pub fn run() {\n        use super::*;\n        #[cfg(any())]\n        fn now() {}\n        now();\n    }\n}\n",
            &["crate::clock::now in crate::core"][..],
        ),
        (
            "ungatedblockitemglob",
            items.to_string(),
            "use crate::clock::now;\npub mod inner {\n    pub fn run() {\n        use super::*;\n        fn now() {}\n        now();\n    }\n}\n",
            &[][..],
        ),
        (
            "gatedrelay",
            format!("{items}{}", relay("#[cfg(any())]\n    ")),
            "use crate::bridge::now;\npub fn run() { now(); }\n",
            &["crate::clock::now in crate::core"][..],
        ),
        (
            "ungatedrelay",
            format!("{items}{}", relay("")),
            "use crate::bridge::now;\npub fn run() { now(); }\n",
            &[][..],
        ),
    ] {
        let probe = RootProbe::new(package, "", &[("src/lib.rs", &lib), ("src/core.rs", core)]);
        assert_inline_answers(&probe, package, "crate::core", "crate::clock", found, found);
    }
}

/// A head a scope holds only by a gated item may also name what no scope binds: on a build that compiles the item out
/// it names a sysroot crate or a dependency. So `#[cfg(any())] mod std {}` leaves `std::process::id()` to `std`, which
/// reports under `std::process`; and `#[cfg(any())] mod md5x {}` leaves `md5x::compute()` to the dependency, which
/// reports under `.strict_external()`, where an un-`use`d dependency call is observed. A module written ungated
/// shadows the crate and nothing reports, and so does a crate root's `extern crate core as std;`, which answers `std`
/// for certain — the call is `core::mem::drop` — unless that alias is gated too, even beside a root `mod std` no `cfg`
/// gates, which no other module sees. rustc 1.96.0, edition 2021, builds each.
#[test]
fn a_head_held_only_by_a_gated_item_may_name_a_crate() {
    let core = |module: &str| {
        format!(
            "#[allow(unused)]\n{module}\npub fn run() {{\n    let _ = std::process::id();\n}}\n"
        )
    };
    for (package, module, found) in [
        (
            "gatedstd",
            "#[cfg(any())]\nmod std {}",
            &["std::process::id in crate::core"][..],
        ),
        (
            "ungatedstd",
            "mod std { pub mod process { pub fn id() -> u32 { 0 } } }",
            &[][..],
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", "pub mod core;\n"),
                ("src/core.rs", &core(module)),
            ],
        );
        assert_inline_answers(&probe, package, "crate::core", "std::process", found, found);
    }
    for (package, alias, found) in [
        ("certainalias", "extern crate core as std;", &[][..]),
        (
            "gatedalias",
            "#[cfg(any())]\nextern crate core as std;",
            &["std::mem::drop in crate::core"][..],
        ),
        (
            "gatedaliasbesidemod",
            "#[cfg(any())]\nextern crate core as std;\npub mod std {}",
            &["std::mem::drop in crate::core"][..],
        ),
    ] {
        let probe = RootProbe::new(
            package,
            "",
            &[
                ("src/lib.rs", &format!("{alias}\npub mod core;\n")),
                ("src/core.rs", "pub fn run() {\n    std::mem::drop(1);\n}\n"),
            ],
        );
        assert_inline_answers(&probe, package, "crate::core", "std::mem", found, found);
    }
    for (package, module, strict) in [
        (
            "gateddependency",
            "#[cfg(any())]\nmod md5x {}",
            &["md5x::compute in crate::core"][..],
        ),
        (
            "ungateddependency",
            "mod md5x { pub fn compute() -> u32 { 0 } }",
            &[][..],
        ),
    ] {
        let probe = files_probe(
            package,
            "2021",
            &[
                ("src/lib.rs", "pub mod core;\n"),
                (
                    "src/core.rs",
                    &format!(
                        "#[allow(unused)]\n{module}\npub fn run() {{\n    let _ = md5x::compute();\n}}\n"
                    ),
                ),
            ],
            &["md5x"],
        );
        assert_inline_answers(&probe, package, "crate::core", "md5x", &[], strict);
    }
}

/// The over-reaction the cfg-gated reading declares: where the gated import is the one compiled — `#[cfg(not(any()))]`
/// holds on every build — the glob's `std::process::Command` is read beside it, and its `Command::new` reports,
/// though rustc resolves `Command` to `crate::mock::Command` there. rustc 1.96.0, edition 2021, builds it with the call
/// annotated as `crate::mock::Command`.
#[test]
fn a_cfg_gated_name_beside_a_glob_is_read_with_the_glob() {
    assert_crate_answers(
        "cfggatedlive",
        concat!(
            "#![allow(unused)]\nuse std::process::Command;\npub mod mock { pub struct Command; impl Command { pub fn new(_: &str) -> Self { Command } } }\n\
             pub mod live {\n    use super::*;\n    #[cfg(not(any()))]\n    use crate::mock::Command;\n    pub fn f() { let _: crate::mock::Command = ",
            command_new_call!(),
            "; }\n}\n"
        ),
        "std::process",
        &["std::process::Command::new in crate"],
    );
}

/// A glob read later in a pass is not read through itself by an answer remembered earlier in that pass: `s` globs
/// `crate::lib_a::*`, which brings a module `x` holding a cfg-closed `x`, and `self::x::*`, while `p` globs
/// `crate::s::x::*`. Reading `p` first remembers `s`'s `x` as the glob `self::x::*` last answered, and reading that
/// glob from the remembered answer named its own reading and grew every pass, so the unit was refused as one whose
/// globs do not settle. Each glob is read with no answer remembered, so the unit is judged and the call reports.
/// rustc 1.96.0 builds the crate, edition 2021.
#[test]
fn a_glob_read_later_in_a_pass_is_not_read_through_itself() {
    let probe = lib_probe(
        "globselfmemo",
        "#![allow(unused)]\npub mod s {\n    pub use crate::lib_a::*;\n    pub use self::x::*;\n}\npub mod lib_a { pub mod x { #[cfg(any())] pub mod x { pub fn leaf2() {} } pub fn leaf() {} } }\npub mod p { pub use crate::s::x::*; }\npub fn g() -> u32 { std::process::id() }\n",
    );
    assert_eq!(
        inline_findings(&probe, "globselfmemo", "crate", "std::process", true),
        ["std::process::id in crate"]
    );
}

/// Globs whose readings grow more than once along a chain still settle: read cfg-blind, `x` names both `crate::p`
/// and `late::q`, so `use x::*;` grows once through each, and the root's `use c1::*;`, `use c2::*;` and `use c3::*;`,
/// written against that order, each grow again after it. `leaf()` resolves to `crate::p::c1::c2::c3::leaf`. rustc
/// 1.96.0 builds the crate under the host's cfg, edition 2021.
#[test]
fn globs_whose_readings_grow_more_than_once_settle() {
    let probe = lib_probe(
        "globsgrowtwice",
        "#![allow(unused_imports)]\nuse c3::*;\nuse c2::*;\nuse c1::*;\nuse x::*;\n#[cfg(unix)]\nuse crate::p as x;\n#[cfg(not(unix))]\nuse late::q as x;\npub mod p { pub mod c1 { pub mod c2 { pub mod c3 { pub mod late { pub mod q { pub mod c1 { pub mod c2 { pub mod c3 { pub fn leaf2() {} } } } } } pub fn leaf() {} } } } }\npub fn g() { leaf(); }\n",
    );
    assert!(
        inline_findings(&probe, "globsgrowtwice", "crate", "crate::p", false)
            .contains(&"crate::p::c1::c2::c3::leaf in crate".to_string())
    );
}

/// Gated child-module heads do not invent extern crates while their sibling globs are resolved. The call in a
/// child remains observed for two, three and four globs.
#[test]
fn gated_sibling_globs_settle_and_observe_calls() {
    for count in [2, 3, 4] {
        let mut module = String::new();
        let mut files = vec![("src/lib.rs".to_string(), "pub mod fs;\n".to_string())];
        for i in 0..count {
            module.push_str(&format!("#[cfg(unix)] mod m{i};\npub use m{i}::*;\n"));
            files.push((
                format!("src/fs/m{i}.rs"),
                if i == 0 {
                    "pub fn f() { let _ = std::process::id(); }\n".to_string()
                } else {
                    String::new()
                },
            ));
        }
        files.push(("src/fs/mod.rs".to_string(), module));
        let probe = RootProbe::new("gatedsiblings", "", &borrowed(&files));
        let found = ["std::process::id in crate::fs::m0"];
        assert_inline_answers(
            &probe,
            "gatedsiblings",
            "crate",
            "std::process",
            &found,
            &found,
        );
    }
}

/// Compiling out a module leaves its name to a real dependency, whose glob still reacts under strict external
/// confinement. The path through the gated module remains a candidate in either mode.
#[test]
fn a_gated_module_glob_keeps_its_dependency_candidate() {
    let probe = files_probe(
        "gateddepglob",
        "2021",
        &[
            ("src/lib.rs", "pub mod core;\n"),
            (
                "src/core.rs",
                "#[cfg(any())] mod md5x {}\nuse md5x::*;\npub fn f() { let _ = compute(); }\n",
            ),
        ],
        &["md5x"],
    );
    for strict in [false, true] {
        let found = inline_findings(&probe, "gateddepglob", "crate", "md5x", strict);
        assert!(
            found
                .iter()
                .any(|p| p == "glob crate::core::md5x in crate::core"),
            "{found:?}"
        );
        if strict {
            assert!(
                found.iter().any(|p| p == "glob md5x in crate::core"),
                "{found:?}"
            );
        }
    }
}

/// A missing crate-internal glob target can supply candidate names but no contents to read repeatedly. An enum
/// glob beside it continues to bring its variant, and a direct forbidden call remains observed.
#[test]
fn an_internal_non_module_glob_target_keeps_enum_variants_and_calls() {
    let probe = lib_probe(
        "internalglobtarget",
        "#[cfg(any())] pub mod backend {}\n\
        pub enum E { V(u8) }\npub mod constants {\n\
        #[cfg(any())] pub use crate::backend::fs::types::*;\n\
        pub use crate::E::*;\n}\npub fn f() { let _ = crate::constants::V(1); let _ = std::process::id(); }\n",
    );
    let found = ["std::process::id in crate"];
    assert_inline_answers(
        &probe,
        "internalglobtarget",
        "crate",
        "std::process",
        &found,
        &found,
    );
    let found = inline_findings(&probe, "internalglobtarget", "crate", "crate::E", false);
    assert!(
        found.iter().any(|p| p == "crate::E::V in crate"),
        "{found:?}"
    );
}
