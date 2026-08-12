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
