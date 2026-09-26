//! The gate-identity judgement's failure matrix.

use crate::gate_identity::{Citation, citations, offences, registered_names, uncited_scripts};
use crate::refusal::Kind;

/// A listing carrying every shape the join has to tell apart:
///
/// * `module::ident` — a test **inside a module**, which `--exact ident` does not select;
/// * `the_gate` and `other::the_gate` — one leaf under two paths, which `--exact the_gate` resolves to
///   exactly one;
/// * `twice::same` appearing twice — a genuine duplicate, the only shape that is really registered twice.
const LISTING: &str = "module::ident: test\nthe_gate: test\nother::the_gate: test\ntwice::same: test\ntwice::same: test\n";

fn lists(_pkg: &str, _target: &str) -> Result<String, String> {
    Ok("the_gate: test\nsomething_else: test\n".to_string())
}

fn invocation(identifier: &str) -> String {
    format!(
        "cargo test --manifest-path x -p kanhe --test merge_message \\\n    -- --exact {identifier}\n"
    )
}

/// The citations of a fixture script, which every fixture here keeps placeable by construction.
fn cited(script: &str) -> Vec<Citation> {
    citations("scripts/w.sh", script).unwrap_or_else(|(line, what)| {
        panic!(
            "the fixture holds {what} at line {line}, which is not a row this matrix means to carry"
        )
    })
}

#[test]
fn an_identifier_is_bound_to_the_target_of_its_own_invocation() {
    let found = cited(&invocation("the_gate"));
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].identifier, "the_gate");
    assert_eq!(found[0].target.as_deref(), Some("merge_message"));
    assert_eq!(found[0].package.as_deref(), Some("kanhe"));
}

#[test]
fn a_commented_invocation_cites_nothing() {
    assert!(cited("# cargo test -p k --test t -- --exact ghost\n").is_empty());
}

/// A line ending in an escaped backslash ends where bash ends the command, so a flag on the next line
/// belongs to a statement of its own — never to this citation's invocation.
///
/// The reading this row replaces joined the two lines and bound `--test target_a` to `the_gate`: an
/// invocation bash never runs, reported as the one it does. Here the identifier stands alone in its own
/// statement, so it carries no target — and `offences` refuses it as one it cannot bind.
#[test]
fn a_citation_is_not_bound_across_a_line_bash_does_not_join() {
    let found = cited("cargo test -p kanhe --test target_a \\\\\n-- --exact the_gate\n");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].identifier, "the_gate");
    assert_eq!(
        found[0].target.as_deref(),
        None,
        "the `--test` sits in the statement bash ended at the escaped backslash, not in this citation's"
    );
    let refusals = offences(&found, lists);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#citation-names-no-test-target",
        &refusals[0],
    );
}

/// Inside single quotes a backslash is literal text and joins nothing: the string runs across the newline,
/// exactly as bash passes it, and the identifier carries both.
#[test]
fn a_backslash_inside_single_quotes_is_part_of_the_word() {
    let found = cited("cargo test -p kanhe --test target_a -- --exact 'the_gate\\\ncontinued'\n");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].identifier, "the_gate\\\ncontinued");
    assert_eq!(found[0].target.as_deref(), Some("target_a"));
}

/// A script the lexer cannot place is a cannot-judge, never an empty reading: reporting it as citing
/// nothing would be the vacuity direction, and reading past the word would be a citation never judged.
#[test]
fn a_script_the_shell_reader_cannot_place_is_a_cannot_judge() {
    let refusals = uncited_scripts([("scripts/w.sh", "cat <<EOF\n--exact ghost\nEOF\n")]);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#a-script-the-shell-reader-cannot-place",
        &refusals[0],
    );
}

/// The listed name is carried whole, because that is the string `--exact` compares against.
#[test]
fn a_registered_name_is_the_whole_listed_path() {
    assert_eq!(
        registered_names(LISTING),
        vec![
            "module::ident",
            "the_gate",
            "other::the_gate",
            "twice::same",
            "twice::same"
        ]
    );
}

/// A citation naming a leaf whose test lives in a module is a **violation**.
///
/// The false negative the requirement exists to close: the target lists `module::ident`, so
/// `--exact ident` selects nothing and `libtest` exits 0 over it. Truncating the listed name to its last
/// segment made this read clean.
#[test]
fn a_citation_naming_a_leaf_inside_a_module_is_a_violation() {
    let refusals = offences(&cited(&invocation("ident")), |_, _| Ok(LISTING.to_string()));
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].kind, Kind::Violation);
    crate::refusal::expect(
        "repository-checks#citation-names-an-unregistered-gate",
        &refusals[0],
    );
    assert!(
        refusals[0].message.contains("does not register"),
        "{}",
        refusals[0].message
    );
}

/// One leaf under two module paths is **not** a citation naming a set.
///
/// `--exact the_gate` resolves to exactly one test — the one at file scope — so refusing this citation was a
/// false refusal invented by the truncation rather than a fact about the target.
#[test]
fn a_leaf_shared_by_two_module_paths_is_not_a_duplicate() {
    let refusals = offences(&cited(&invocation("the_gate")), |_, _| {
        Ok(LISTING.to_string())
    });
    assert!(
        refusals.is_empty(),
        "a citation `--exact` resolves to one test must not be refused, got {refusals:?}"
    );
}

#[test]
fn a_gate_the_target_does_not_register_is_a_violation() {
    let refusals = offences(&cited(&invocation("renamed_away")), lists);
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::Violation);
    crate::refusal::expect(
        "repository-checks#citation-names-an-unregistered-gate",
        &refusals[0],
    );
    assert!(refusals[0].message.contains("does not register"));
}

#[test]
fn a_gate_registered_twice_is_a_violation() {
    let refusals = offences(&cited(&invocation("twice::same")), |_, _| {
        Ok(LISTING.to_string())
    });
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::Violation);
    crate::refusal::expect(
        "repository-checks#citation-names-a-gate-registered-several-times",
        &refusals[0],
    );
    assert!(refusals[0].message.contains("registers 2 times"));
}

#[test]
fn an_identifier_with_no_target_cannot_be_judged() {
    let refusals = offences(&cited("cargo test -- --exact loose\n"), lists);
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#citation-names-no-test-target",
        &refusals[0],
    );
    assert!(refusals[0].message.contains("names no `--test <target>`"));
}

#[test]
fn a_listing_that_cannot_be_read_cannot_be_judged() {
    let refusals = offences(&cited(&invocation("the_gate")), |_, _| {
        Err("cargo exploded".to_string())
    });
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::CannotJudge);
    crate::refusal::expect(
        "repository-checks#citation-target-listing-unreadable",
        &refusals[0],
    );
    assert!(refusals[0].message.contains("could not list"));
}

#[test]
fn a_gate_registered_once_is_clean() {
    assert!(offences(&cited(&invocation("the_gate")), lists).is_empty());
}

/// A script that defers its verdict to a named gate is what a wrapper is.
#[test]
fn a_script_citing_a_gate_is_a_wrapper() {
    assert!(uncited_scripts([("scripts/merge-pr.sh", invocation("the_gate").as_str())]).is_empty());
}

/// The named library is exempt while it cites nothing, and **refused when it starts to** — the two halves
/// of the exemption, held together so the skip cannot silently widen into accepting a gate it defers.
#[test]
fn the_shared_library_is_exempt_only_while_it_defers_nothing() {
    let library = crate::gate_identity::WRAPPERS_SHARED_LIBRARY;
    // Citing nothing is the state the exemption is for.
    assert!(
        uncited_scripts([(library, "#!/usr/bin/env bash\nset -eu\n")]).is_empty(),
        "the library defers nothing, so nothing is owed"
    );
    // A citation appearing inside it is a change to the requirement, refused rather than skipped.
    let refusals = uncited_scripts([(library, invocation("the_gate").as_str())]);
    assert_eq!(refusals.len(), 1, "{refusals:?}");
    assert_eq!(refusals[0].kind, Kind::Violation);
    crate::refusal::expect(
        "repository-checks#the-shared-library-names-a-gate",
        &refusals[0],
    );
    assert!(refusals[0].message.contains(library));
}

/// A script citing nothing renders its own verdict, and is named.
#[test]
fn a_script_citing_no_gate_is_named() {
    let refusals = uncited_scripts([(
        "scripts/check_something.sh",
        "#!/usr/bin/env bash\nset -eu\nif grep -q bad tracked; then exit 1; fi\n",
    )]);
    assert_eq!(refusals.len(), 1);
    assert_eq!(refusals[0].kind, Kind::Violation);
    crate::refusal::expect("repository-checks#wrapper-cites-no-gate", &refusals[0]);
    assert!(refusals[0].message.contains("scripts/check_something.sh"));
    assert!(refusals[0].message.contains("`--exact`"));
}

/// **One script citing twice does not excuse a sibling citing none.**
///
/// This is the whole reason the question is asked per script. Asserting that the citation total reaches the
/// script count passes here — two citations, two scripts — while one of them defers to nothing, which is the
/// aggregate reading the direction above this replaced.
#[test]
fn a_sibling_citing_twice_does_not_cover_a_script_citing_none() {
    let citing_twice = format!("{}{}", invocation("first_gate"), invocation("second_gate"));
    let refusals = uncited_scripts([
        ("scripts/publish.sh", citing_twice.as_str()),
        ("scripts/lib/helper.sh", "#!/usr/bin/env bash\nset -eu\n"),
    ]);
    assert_eq!(refusals.len(), 1);
    crate::refusal::expect("repository-checks#wrapper-cites-no-gate", &refusals[0]);
    assert!(refusals[0].message.contains("scripts/lib/helper.sh"));
}

/// A citation that is commented out is not a citation, so the script carrying only one is named.
///
/// Composes with `a_commented_invocation_cites_nothing` rather than restating it: that row says the extractor
/// ignores a commented invocation, this says what follows for the script it sat in.
///
/// **Every line carries the marker**, which is what commenting out a block actually is. This fixture used to
/// prefix one `#` to a two-line invocation and call the whole thing commented — see
/// [`a_marker_on_the_first_line_does_not_comment_the_continuation`] for why that is not what bash does.
#[test]
fn a_script_whose_only_invocation_is_commented_out_is_named() {
    let commented: String = invocation("the_gate")
        .lines()
        .map(|line| format!("# {line}\n"))
        .collect();
    let refusals = uncited_scripts([("scripts/probe.sh", commented.as_str())]);
    assert_eq!(refusals.len(), 1);
    crate::refusal::expect("repository-checks#wrapper-cites-no-gate", &refusals[0]);
    assert!(refusals[0].message.contains("scripts/probe.sh"));
}

/// A continuation does not reach **across** a comment, because bash ends the command there.
///
/// Measured: for `echo START \` / `# comment` / `--exact ghost`, bash prints `START` and then reports
/// `--exact: command not found` — the backslash pulls the comment onto the line and `#` at a word boundary
/// ends the command, so the third line is its own command rather than part of the first. The reader lexes,
/// so the comment opens exactly where bash opens it and no region pass is needed before the split.
#[test]
fn a_continuation_does_not_reach_across_a_comment() {
    let script = "cargo test -p kanhe --test t \\\n# a comment ends the command here\n    -- --exact ghost\n";
    let found = cited(script);
    assert_eq!(
        found.len(),
        1,
        "`--exact ghost` is its own command line, so it is a citation: {found:?}"
    );
    assert_eq!(found[0].identifier, "ghost");
    assert_eq!(
        found[0].target, None,
        "the comment ended the `cargo test` command, so this citation names no `--test` — binding it to one \
         would report a target bash never gave it: {found:?}"
    );
    assert_eq!(found[0].package, None, "{found:?}");
}

/// A `#` on the first line of a continued invocation comments **that line**, not the continuation.
///
/// Measured, not reasoned about — `bash -c` on `# echo COMMENTED \` followed by `  echo THIS_RAN` prints
/// `THIS_RAN`. A comment runs to end of line and a backslash inside one continues nothing, so the
/// continuation executes and the citation on it is a citation. The reader sees this because the lexer opens
/// a comment where bash does — an unquoted `#` beginning a word — rather than by dropping comment-marked
/// lines before a join of its own, which would take the continuation down with the line.
#[test]
fn a_marker_on_the_first_line_does_not_comment_the_continuation() {
    let found = cited(&format!("# {}", invocation("ghost")));
    assert_eq!(
        found.len(),
        1,
        "the continuation of a commented line executes, so the citation on it is a citation: {found:?}"
    );
    assert_eq!(found[0].identifier, "ghost");
}

/// An empty corpus yields no refusal here, and refusing it is the caller's job.
///
/// Stated as a row rather than left implicit: a set that never arrived is not a set in which every member cites
/// a gate, and the repository direction keeps its own enumeration guard for exactly that.
#[test]
fn an_empty_corpus_is_not_this_judgement_s_refusal() {
    assert!(uncited_scripts([]).is_empty());
}
