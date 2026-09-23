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

    let files = reacting_files(&check(&root_scope_fs_law("binonly"), probe.manifest()));
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
/// src/bin/*.rs, and omitting the compilation unit qualifier under the no-target fallback.
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
    let inline_with_no_targets: &[(&str, &str)] = &[
        ("examples/dummy.rs", "fn main() {}\n"),
        (
            "src/lib.rs",
            "pub mod forbidden;\npub mod seam {\n    use crate::forbidden::X;\n}\n",
        ),
        ("src/forbidden.rs", "pub struct X;\n"),
    ];
    let inline_nested_in_the_library: &[(&str, &str)] = &[
        ("src/lib.rs", "pub mod outer;\n"),
        ("src/outer.rs", "pub mod inner {\n    use brick::B;\n}\n"),
    ];
    let no_target_manifest = concat!(
        "autolib = false\n",
        "autobins = false\n",
        "[[example]]\n",
        "name = \"dummy\"\n",
        "path = \"examples/dummy.rs\"\n",
    );

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
            Some("src/main.rs"),
            "src/seam.rs",
        ),
        (
            "lib",
            "",
            INLINE_IN_THE_LIBRARY,
            "crate::seam",
            Some("src/lib.rs"),
            "src/seam.rs",
        ),
        (
            "toolbin",
            "",
            inline_in_the_tool_binary,
            "crate::seam",
            Some("src/bin/tool.rs"),
            "src/bin/seam.rs",
        ),
        (
            "notarget",
            no_target_manifest,
            inline_with_no_targets,
            "crate::seam",
            None,
            "src/seam.rs",
        ),
        (
            "nestedlib",
            "",
            inline_nested_in_the_library,
            "crate::outer::inner",
            Some("src/lib.rs"),
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
                if let Some(unit) = expected_unit {
                    assert!(
                        message.contains(&format!("in compilation unit '{unit}'")),
                        "{package}: expected refusal to name compilation unit '{unit}', \
                         got: {message}"
                    );
                } else {
                    assert!(
                        !message.contains("compilation unit"),
                        "{package}: expected no compilation unit qualifier for no-target \
                         fallback, got: {message}"
                    );
                }
            }
            other => panic!("{package}: expected the inline-target refusal: {other:?}"),
        }
    }
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
