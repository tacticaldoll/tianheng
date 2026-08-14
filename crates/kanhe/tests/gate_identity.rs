//! Collation: the test identifier a wrapper cites, against the tests its target registers.
//!
//! `libtest` exits `0` when `--exact` selects no test, so a rename disarms a wrapper silently and the script
//! proceeds to the act it stands in front of. The wrapper's own `1 passed` assertion covers that moment; this
//! covers the interval before it — a rename lands in a pull request long before anyone runs a wrapper.

use std::path::{Path, PathBuf};
use std::process::Command;

use kanhe::gate_identity::{citations, offences, uncited_scripts};

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("scripts").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

fn run(root: &Path, args: &[&str]) -> Result<String, String> {
    let out = Command::new(args[0])
        .args(&args[1..])
        .current_dir(root)
        .env("CARGO_TERM_COLOR", "never")
        .output()
        .map_err(|err| format!("cannot run {args:?}: {err}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Every tracked shell script and its text, enumerated once for the two directions below.
///
/// One implementation because two would be two enumerations that must agree, and a script the second forgot
/// would be judged by one direction and not the other — which is the granularity defect this file's newer
/// direction exists to close, reintroduced one level up.
fn tracked_scripts(root: &Path) -> Vec<(String, String)> {
    let listing = run(root, &["git", "ls-files", "scripts/"]).expect(
        "the tracked scripts are enumerable; a failed enumeration returns exactly what a repository holding \
         no scripts returns, and reporting that as clean is the vacuity direction",
    );
    let scripts: Vec<String> = listing
        .lines()
        .filter(|path| path.ends_with(".sh"))
        .map(str::to_string)
        .collect();
    assert!(
        !scripts.is_empty(),
        "no tracked shell script was enumerated, so this check would report clean over nothing"
    );
    scripts
        .into_iter()
        .map(|script| {
            let text = std::fs::read_to_string(root.join(&script))
                .unwrap_or_else(|err| panic!("cannot read tracked {script}: {err}"));
            (script, text)
        })
        .collect()
}

/// Every tracked script defers its verdict to a gate it names.
///
/// The sibling below asks whether each citation resolves. This asks whether a script made one — and only this
/// one sees a script that cites **nothing**, which is a script rendering its own verdict rather than gathering
/// evidence for a Rust check. That shape is what this repository deleted 1562 lines of, and until this the way
/// back was open: every citation went into one list and the list was asserted non-empty, so a single citing
/// sibling covered for all the rest.
///
/// Holding this closes `scripts/` as a category. A tracked script that is not a wrapper cannot be added while
/// it stands, which is what `repository-checks` already claims when it says `git ls-files scripts/` names only
/// wrappers — the claim is now held rather than written.
#[test]
fn every_tracked_script_defers_its_verdict_to_a_gate() {
    let Some(root) = workspace_root() else {
        return;
    };
    let sources = tracked_scripts(&root);
    let refusals = uncited_scripts(
        sources
            .iter()
            .map(|(script, text)| (script.as_str(), text.as_str())),
    );
    assert!(
        refusals.is_empty(),
        "a tracked script defers its verdict to nothing:\n{}",
        refusals
            .iter()
            .map(|refusal| format!("  ({:?}) {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Every identifier a tracked script asks a gate for names a test that target registers exactly once.
#[test]
fn every_gate_a_wrapper_cites_is_a_test_that_exists() {
    let Some(root) = workspace_root() else {
        return;
    };
    let sources = tracked_scripts(&root);

    let mut cited = Vec::new();
    for (script, text) in &sources {
        cited.extend(citations(script, text));
    }
    assert!(
        !cited.is_empty(),
        "no tracked script cites a gate by `--exact`, so this check holds nothing — if the wrappers stopped \
         asking for their gates that way, this check should be retired rather than left asserting an empty \
         set"
    );

    let refusals = offences(&cited, |package, target| {
        run(
            &root,
            &[
                "cargo", "test", "-q", "-p", package, "--test", target, "--", "--list",
            ],
        )
    });
    assert!(
        refusals.is_empty(),
        "a wrapper asks for a gate by a name its target does not carry:\n{}",
        refusals
            .iter()
            .map(|refusal| format!("  ({:?}) {}", refusal.kind, refusal.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// Every `cargo test -p <package> --test <target>` a tracked **document** hands a reader names a real target.
///
/// The sibling above holds the commands a *wrapper* runs. This holds the ones a *document* tells a human to
/// run, which is the same claim reaching a different audience — and the audience that cannot debug it. The
/// instance: `COOKBOOK.md` told an adopter `cargo test -p tianheng --test examples_suite` for a target living
/// in `shengmo`, so cargo answered `no test target named 'examples_suite' in 'tianheng' package`. It arrived
/// in the `0.5.0` window, when the shell suite migrated, while `AGENTS.md` and `BACKLOG.md` both carried the
/// correct package — the repository disagreeing with itself in the one place an adopter reads.
///
/// **The target set is produced, never modelled.** `cargo metadata` publishes each package's test targets by
/// name, so this is set membership against cargo's own answer. Mapping a package and target to
/// `crates/<pkg>/tests/<target>.rs` would reimplement cargo's target resolution in string form, which this
/// repository has already shipped a false negative from doing — and the sibling above says so, which is why
/// it resolves through `--list` rather than through a path.
///
/// Markdown only, deliberately. A Rust source carries these pairs as **fixture input** — the directions in
/// `reference_integrity.rs` plant `-p k --test t` as text for a parser to chew on — and admitting them would
/// report a test asserting its own parser as a broken command. Measured: 35 occurrences across the tree, of
/// which those fixtures are two and the `COOKBOOK.md` line was the one real defect.
#[test]
fn every_command_a_document_hands_a_reader_names_a_target_that_exists() {
    let Some(root) = workspace_root() else {
        return;
    };
    let metadata = run(
        &root,
        &["cargo", "metadata", "--no-deps", "--format-version", "1"],
    )
    .expect("cargo metadata answers for this workspace");
    let metadata: serde_json::Value =
        serde_json::from_str(&metadata).expect("cargo metadata is JSON");
    let mut targets: std::collections::BTreeSet<(String, String)> = Default::default();
    for package in metadata["packages"].as_array().into_iter().flatten() {
        let Some(name) = package["name"].as_str() else {
            continue;
        };
        for target in package["targets"].as_array().into_iter().flatten() {
            let is_test = target["kind"]
                .as_array()
                .is_some_and(|k| k.iter().any(|k| k.as_str() == Some("test")));
            if let (true, Some(target)) = (is_test, target["name"].as_str()) {
                targets.insert((name.to_string(), target.to_string()));
            }
        }
    }
    assert!(
        !targets.is_empty(),
        "cargo named no test target, so this direction would hold over nothing"
    );

    let listing =
        run(&root, &["git", "ls-files", "*.md"]).expect("the tracked Markdown is listable");
    let mut examined = 0usize;
    let mut broken = Vec::new();
    for path in listing.lines().filter(|l| !l.is_empty()) {
        // `examined` counts what was opened, which is a vacuity guard and not a completeness one: one
        // unreadable document leaves every command it hands a reader unchecked while the count still says
        // this direction ran.
        let text = std::fs::read_to_string(root.join(path)).unwrap_or_else(|error| {
            panic!(
                "cannot read tracked file '{path}' — a file this check claims to have inspected must have \
                 been read: {error}"
            )
        });
        for line in text.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            for window in words.windows(4) {
                let ["-p", package, "--test", target] = window else {
                    continue;
                };
                // A prose line ends the command with whatever punctuation the sentence needs — a backtick,
                // then a period, a comma, a colon, a closing paren. Trimming only the backtick left every
                // correct command in the tree reported as broken, which is a detector that cannot tell its
                // own noise from a finding.
                let target = target.trim_end_matches(|c: char| !c.is_alphanumeric() && c != '_');
                examined += 1;
                if !targets.contains(&((*package).to_string(), target.to_string())) {
                    broken.push(format!(
                        "  {path}: `cargo test -p {package} --test {target}` — cargo names no such test \
                         target in that package"
                    ));
                }
            }
        }
    }
    assert!(
        examined > 0,
        "no document names a package and a test target, so this direction would report clean over nothing"
    );
    assert!(
        broken.is_empty(),
        "a tracked document hands a reader a command cargo rejects:\n{}",
        broken.join("\n")
    );
}
