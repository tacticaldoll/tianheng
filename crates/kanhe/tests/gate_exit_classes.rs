//! Repository check: a wrapper's exit class agrees with the gate it fronts.
//!
//! Two facts live in two languages here. `refusal::Kind` types the distinction a gate draws — a source that
//! **disagrees** against one that **could not be read** — and a shell wrapper turns a gate's failure into a
//! process exit class. The wrappers read the class out of the gate's own output, which means a Rust enum's
//! rendering and a `grep` pattern in a script must agree. Two places that must agree is the shape this
//! repository has spent a window replacing, so it is checked rather than commented.
//!
//! **What went wrong without it.** Five could-not-read conditions in `scripts/merge-pr.sh` were split across
//! both exit classes with no stated rule, and two of the facts on the `1` side are ones
//! `merge_message_gate::judge` types as cannot-judge — so the wrapper reported as a disagreement what its own
//! gate calls unjudgeable. No direction could have caught it: the ones covering those sites asserted only that
//! the wrapper failed, which cannot see `1` from `2`.

use std::path::{Path, PathBuf};

use kanhe::refusal::Kind;

fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("scripts/merge-pr.sh").is_file(),
        shengmo::workspace::marker_set(),
    )
}

/// Every wrapper that fronts a gate, and the gate identifier it asks for.
///
/// Both scripts are named rather than globbed: a script this array forgets is a wrapper whose exit classes
/// nothing compares, and the sibling direction below holds the array against what the tree actually carries.
const WRAPPERS: [&str; 2] = ["scripts/merge-pr.sh", "scripts/publish.sh"];

fn read(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
        panic!("cannot read {path}, so its exit classes were never compared: {err}")
    })
}

/// The token each wrapper matches on is the one `Kind` actually renders for a disagreement.
///
/// The wrappers grep the gate's own output for the class it printed. `merge_message.rs` and
/// `publish_source.rs` both render it as `{:?}` on `Kind`, so renaming the variant would silently turn every
/// violation into the unjudged class — the wrapper would still exit non-zero, still print the gate's output, and
/// still be wrong about which fact it found. Nothing else would notice.
#[test]
fn each_wrapper_matches_the_token_the_gate_renders() {
    let Some(root) = workspace_root() else {
        return;
    };
    let rendered = format!("{:?}", Kind::Violation);
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        let declared = text
            .lines()
            .find_map(|line| line.trim().strip_prefix("GATE_VIOLATION_TOKEN="))
            .unwrap_or_else(|| {
                panic!(
                    "{wrapper} declares no `GATE_VIOLATION_TOKEN`, so the class it reports for a failing gate \
                     rests on nothing this check can compare"
                )
            });
        assert_eq!(
            declared, rendered,
            "{wrapper} matches on `{declared}` while `refusal::Kind` renders a disagreement as `{rendered}` — \
             every violation would be reported as an input the gate could not judge"
        );
    }
}

/// Both gates render the class in the form the wrappers read.
///
/// The other half of the same agreement. The token could match `Kind`'s rendering exactly while no gate ever
/// printed it, which would make every failing gate read as unjudged — and the direction above would still pass.
#[test]
fn each_gate_renders_its_class_where_the_wrapper_reads_it() {
    let Some(root) = workspace_root() else {
        return;
    };
    for gate in [
        "crates/kanhe/tests/merge_message.rs",
        "crates/kanhe/tests/publish_source.rs",
    ] {
        let text = read(&root, gate);
        assert!(
            text.contains("refusal.kind, refusal.message"),
            "{gate} does not render its refusal's kind beside its message, so a wrapper reading the class out \
             of this gate's output has nothing to read"
        );
    }
}

/// Only the gate's own verdict may exit the violation class.
///
/// The rule this file exists to hold, read off the scripts: `1` is reachable exactly where a gate ran and
/// reported a disagreement. Every other stop — a misconfigured invocation, an input that could not be read, a
/// gate that did not run — is the unjudged class. Counted rather than described, because the split that
/// prompted this was five sites spelled out one at a time.
#[test]
fn a_wrapper_exits_the_violation_class_only_for_a_gates_own_verdict() {
    let Some(root) = workspace_root() else {
        return;
    };
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        let sites: Vec<(usize, &str)> = text
            .lines()
            .enumerate()
            .filter(|(_, line)| line.trim() == "exit 1")
            .map(|(index, line)| (index + 1, line))
            .collect();
        assert_eq!(
            sites.len(),
            1,
            "{wrapper} exits the violation class at {} site(s); only a gate's own verdict may, and every \
             could-not-read stop belongs to the unjudged class: {:?}",
            sites.len(),
            sites.iter().map(|(line, _)| line).collect::<Vec<_>>()
        );
        // And that one site must be inside the branch that read the gate's class, not merely somewhere after it.
        let line = sites[0].0;
        let window: String = text
            .lines()
            .skip(line.saturating_sub(4))
            .take(4)
            .collect::<Vec<_>>()
            .join("\n");
        assert!(
            window.contains("GATE_VIOLATION_TOKEN"),
            "{wrapper}'s only `exit 1` is not guarded by the token the gate renders, so it reports a \
             disagreement without having read one:\n{window}"
        );
    }
}

/// Every acquisition a wrapper makes is guarded, so a failing tool cannot choose the exit class.
///
/// `var=$(tool …)` under `set -e` exits with the TOOL's status and only the tool's stderr. Measured, a failing
/// commits read left the merge wrapper exiting **91** with nothing of its own said — a class that is neither of
/// the two it defines, carrying the tool's words for a fact about the wrapper. Four acquisitions were unguarded,
/// and the direction covering one of them passed because it asserted only that the wrapper failed.
#[test]
fn every_acquisition_is_guarded_so_the_tool_cannot_choose_the_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        let mut unguarded = Vec::new();
        let lines: Vec<&str> = text.lines().collect();
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim_start();
            if trimmed.starts_with('#') {
                continue;
            }
            // An assignment whose value is a command substitution of an external tool.
            let Some((_, rest)) = trimmed.split_once("=$(") else {
                continue;
            };
            if !rest.starts_with("gh ") && !rest.starts_with("cargo ") {
                continue;
            }
            // The guard may sit on this line or on any continuation of it.
            let mut guarded = false;
            let mut cursor = index;
            loop {
                let current = lines[cursor];
                if current.contains("cannot_judge") || current.contains("|| {") {
                    guarded = true;
                    break;
                }
                if !current.trim_end().ends_with('\\') || cursor + 1 >= lines.len() {
                    break;
                }
                cursor += 1;
            }
            if !guarded {
                unguarded.push(format!("{wrapper}:{}", index + 1));
            }
        }
        assert!(
            unguarded.is_empty(),
            "these acquisitions are unguarded, so a failing tool exits with its own status and its own stderr \
             instead of one of this wrapper's two classes: {unguarded:?}"
        );
    }
}

/// Every tracked wrapper that runs a gate is named by [`WRAPPERS`].
///
/// Without this the array is a second list beside the tree: a new wrapper would front a gate with its exit
/// classes compared to nothing, while every direction above kept passing over the two it does name.
#[test]
fn every_gate_running_wrapper_is_named() {
    let Some(root) = workspace_root() else {
        return;
    };
    let out = std::process::Command::new("git")
        .args(["ls-files", "scripts"])
        .current_dir(&root)
        .output()
        .expect("run git ls-files");
    assert!(
        out.status.success(),
        "`git ls-files scripts` failed, and a failed enumeration is not a repository with no wrappers"
    );
    let tracked: Vec<String> = String::from_utf8_lossy(&out.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert!(
        !tracked.is_empty(),
        "no tracked script was enumerated, so this direction would hold over nothing"
    );
    let fronting: Vec<&String> = tracked
        .iter()
        .filter(|path| path.ends_with(".sh"))
        .filter(|path| read(&root, path).contains("require_one_pass"))
        .collect();
    let unnamed: Vec<&&String> = fronting
        .iter()
        .filter(|path| !WRAPPERS.contains(&path.as_str()))
        .collect();
    assert!(
        unnamed.is_empty(),
        "these wrappers sequence a gate and are not named by `WRAPPERS`, so their exit classes are compared to \
         nothing: {unnamed:?}"
    );
    assert_eq!(
        fronting.len(),
        WRAPPERS.len(),
        "`WRAPPERS` names {} script(s) while {} tracked script(s) sequence a gate",
        WRAPPERS.len(),
        fronting.len()
    );
}
