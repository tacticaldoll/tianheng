//! The participant's reactions, asserted as runnable proof.
//!
//! Every assertion here is bound to the participant's **own** contribution, never to the exit code alone: the
//! module boundary reacts on its own, so `exit_code() == 1` would hold with the participant contributing
//! nothing at all. That is the shape of false negative this file exists to refuse.

use std::path::PathBuf;

use house_rules::governance::{constitution, manifest, participant, verdict};
use house_rules::observer::ModuleHeaderObserver;
use tianheng::prelude::*;

/// The house rule reacts, with the participant's own structured identity — not a family fact type.
#[test]
fn the_missing_module_header_reacts_with_the_participant_s_own_identity() {
    let Outcome::Violations(report) = participant().observe(&manifest()) else {
        panic!("the file with no `//!` header must produce a structured violation");
    };
    let house_rule: Vec<&Violation> = report
        .violations
        .iter()
        .filter(|violation| violation.fact().fact_type() == "house-rules.fact/module-header")
        .collect();
    assert_eq!(
        house_rule.len(),
        1,
        "exactly one file in `src/` lacks a header: {report:?}"
    );
    assert_eq!(house_rule[0].fact().shape(), "missing-header");
    assert!(
        house_rule[0].finding.ends_with("undocumented.rs"),
        "the finding names the offending file: {:?}",
        house_rule[0].finding
    );
}

/// **The composition, not the exit code.** Both contributions must be present in one verdict: the family
/// dimension's `module` violation and the participant's own. Asserting exit 1 alone would pass while either
/// one silently stopped contributing, since each reacts by itself.
#[test]
fn one_verdict_carries_both_the_dimension_s_finding_and_the_participant_s() {
    let outcome = verdict();
    assert_eq!(outcome.exit_code(), 1);
    let Outcome::Violations(report) = &outcome else {
        panic!("the composed run must report violations: {outcome:?}");
    };
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.fact().fact_type() == "tianheng.fact/guibiao/imported-path"),
        "圭表's contribution is missing, so the fold dropped the dimension: {report:?}"
    );
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.fact().fact_type() == "house-rules.fact/module-header"),
        "the participant's contribution is missing, so joining a run did nothing: {report:?}"
    );
}

/// Precision: the participant reacts to a *missing* header, not to every file. Every other file in `src/`
/// has one, so the count above being exactly one is the discriminator — and a subtree where every file is
/// documented is clean rather than merely quieter.
#[test]
fn a_subtree_whose_files_all_carry_headers_is_clean() {
    let bin = ModuleHeaderObserver::reading(["src/bin"]);
    assert_eq!(bin.observe(&manifest()).exit_code(), 0);
}

/// A subtree the participant was told to read and cannot is **exit 2**, never a quiet pass. An outsider
/// joining a run inherits the family's contract: the one forbidden bug is reporting clean because the look
/// failed.
#[test]
fn a_subtree_that_cannot_be_read_cannot_judge() {
    let absent = ModuleHeaderObserver::reading(["src/no-such-subtree"]);
    let outcome = absent.observe(&manifest());
    assert_eq!(outcome.exit_code(), 2, "{outcome:?}");
    assert!(matches!(outcome, Outcome::ConstitutionError(_)));
}

/// The bounds are **computed**, one per configured subtree, so the declaration set depends on configuration
/// rather than on a literal written in advance. This is what `BoundId`'s owned-or-borrowed form is for, and
/// no declaration inside the family exercises it — every family bound is a literal.
#[test]
fn the_declared_bounds_are_built_from_the_configuration() {
    let two = ModuleHeaderObserver::reading(["src", "src/bin"]);
    let ids: Vec<String> = two
        .bounds()
        .iter()
        .map(|bound| bound.id().as_str().to_string())
        .collect();
    // The whole declared SET, never a bare count: a bound added without a pin has to be visible here, and
    // `len() == 2` would have accepted any two.
    assert_eq!(
        ids,
        vec![
            "house-rules/a-file-nested-below-src-is-out-of-reach".to_string(),
            "house-rules/a-header-below-a-leading-comment-in-src-over-reacts".to_string(),
            "house-rules/a-file-nested-below-src/bin-is-out-of-reach".to_string(),
            "house-rules/a-header-below-a-leading-comment-in-src/bin-over-reacts".to_string(),
        ],
        "one bound per extent per governed subtree, each named after it"
    );
    // Two extents, not one: the example is about the bound model, so a participant declaring only shapes it never
    // reads would be teaching half of it.
    let extents: Vec<String> = participant()
        .bounds()
        .iter()
        .map(|bound| format!("{:?}", bound.extent()))
        .collect();
    assert_eq!(extents.len(), 2, "{extents:?}");
    assert!(
        extents.iter().any(|extent| extent.contains("OutOfReach"))
            && extents.iter().any(|extent| extent.contains("OverReacts")),
        "a shape never read, and a shape read and judged too harshly: {extents:?}"
    );
}

/// The declared over-reaction: a real module header below a leading comment is reported missing.
///
/// The **control** is what keeps this from holding for the wrong reason — the same content with its header on line
/// one does not react, so the fixture proves the *position* of the header decides it and not the file's presence.
///
/// The `_in_src` in the name is not decoration: the bound this defends is declared per governed subtree, so its
/// pin is computed as `a_header_below_a_leading_comment_in_{slug}_over_reacts`. Naming the test anything else
/// leaves the citation resolving to nothing — which it did, and which the dogfood gate now refuses.
#[test]
fn a_header_below_a_leading_comment_in_src_over_reacts() {
    let fixture = Fixture::new("over-reaction");
    let governed = fixture.root.join("src");
    let manifest = fixture.root.join("Cargo.toml");
    let header = "//! This file carries a module header.\n\npub fn probe() {}\n";

    std::fs::write(
        governed.join("licensed.rs"),
        format!("// SPDX-License-Identifier: MIT\n{header}"),
    )
    .expect("a writable fixture");
    let outcome = ModuleHeaderObserver::reading(["src"]).observe(&manifest);
    let Outcome::Violations(report) = &outcome else {
        panic!("the declared over-reaction must be reproducible, or the bound describes nothing: {outcome:?}");
    };
    assert!(
        report
            .violations
            .iter()
            .any(|violation| violation.finding.ends_with("licensed.rs")),
        "a header below a leading comment reads as absent — the declared over-reaction: {report:?}"
    );

    // The control: the same header, on line one.
    std::fs::write(governed.join("licensed.rs"), header).expect("a writable fixture");
    assert_eq!(
        ModuleHeaderObserver::reading(["src"])
            .observe(&manifest)
            .exit_code(),
        0,
        "the header's POSITION is what the over-reaction turns on, not the file's presence"
    );
}

/// **The public surface under test is enough**, checked rather than claimed.
///
/// The example's whole load-bearing result is that joining a run needed no addition to any crate's public API —
/// and the spec says that if an outside crate *cannot* do it with the public surface alone, that is the finding
/// rather than a reason to add whatever export the example wanted. An assertion in prose cannot hold that: a future
/// edit reaching into `guibiao` or `xuanji` directly would keep every other test green while quietly making the
/// example prove the opposite of what it exists to prove.
///
/// So the reach is measured from this crate's own sources: exactly one dependency, and no import of a family crate
/// other than the shell.
#[test]
fn the_participant_reaches_only_the_public_shell() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).expect("this crate's manifest");
    let dependencies: Vec<&str> = manifest
        .lines()
        .skip_while(|line| line.trim() != "[dependencies]")
        .skip(1)
        .take_while(|line| !line.trim_start().starts_with('['))
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .collect();
    assert_eq!(
        dependencies.len(),
        1,
        "one dependency, the shell: {dependencies:?}"
    );
    assert!(
        dependencies[0].starts_with("tianheng"),
        "and it is the shell: {dependencies:?}"
    );

    // The dimensions and 璇璣 are family crates an example could reach past the shell into. Naming them rather
    // than allow-listing the shell keeps the property about what is forbidden — a new family crate would have
    // to be added here deliberately, which is the visible edit.
    const PAST_THE_SHELL: [&str; 5] = ["xuanji", "xingbiao", "guibiao", "hunyi", "louke"];
    let mut reaching = Vec::new();
    for directory in ["src", "src/bin", "tests"] {
        let Ok(entries) = std::fs::read_dir(root.join(directory)) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("rs") {
                continue;
            }
            let text = std::fs::read_to_string(&path).expect("a readable source file");
            for line in text.lines() {
                let trimmed = line.trim_start();
                // Executed text only: this very file names all five crates in the constant above.
                if trimmed.starts_with("//") || !trimmed.starts_with("use ") {
                    continue;
                }
                for crate_ in PAST_THE_SHELL {
                    if trimmed.starts_with(&format!("use {crate_}::")) {
                        reaching.push(format!("{}: {trimmed}", path.display()));
                    }
                }
            }
        }
    }
    assert!(
        reaching.is_empty(),
        "a participant reaching past the shell is the example disproving its own point — if the public \
         surface is not enough, that is the finding rather than a reason to import around it: {reaching:?}"
    );
}

/// A temporary governed subtree, qualified by process id and removed on drop.
///
/// Both properties are the repository's own discipline rather than caution: a fixed path makes two concurrent
/// invocations share one root, and cleaning up at the end of the test body leaves the root behind whenever the
/// test fails — which is exactly when someone is running it. `Drop` runs on the panic path too.
struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new(name: &str) -> Self {
        let root = std::env::temp_dir().join(format!("house-rules-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("src")).expect("a writable temporary subtree");
        std::fs::write(
            root.join("Cargo.toml"),
            "# not parsed by this participant\n",
        )
        .expect("a writable manifest stand-in");
        Self { root }
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

/// The bound this participant declares is the truth about it: a file nested below the governed subtree is
/// never read. Declaring a limit that does not exist would be worse than declaring none.
#[test]
fn a_file_nested_below_src_is_out_of_reach() {
    let nested = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bin/demo.rs");
    assert!(nested.is_file(), "the fixture for this bound must exist");
    let Outcome::Violations(report) = participant().observe(&manifest()) else {
        panic!("the participant must react at all for this bound to mean anything");
    };
    assert!(
        !report
            .violations
            .iter()
            .any(|violation| violation.finding.contains("bin")),
        "nothing below `src/` may appear in the findings: {report:?}"
    );
}

/// The static half is a real boundary of its own, so the example is not teaching a participant beside an
/// inert declaration.
#[test]
fn the_declared_module_boundary_reacts_on_its_own() {
    assert_eq!(
        check_constitution(&constitution(), &manifest()).exit_code(),
        1
    );
}
