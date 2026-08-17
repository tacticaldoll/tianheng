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
use kanhe::region::Source;
use kanhe::verdict_channel;

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

/// Both scalars the wrappers use for the verdict channel are the ones `kanhe::verdict_channel` defines.
///
/// **The pin this replaces held the argument list and not the rendering.** The wrappers used to grep the gate's
/// output for `(Violation)`, with the parentheses in the shell and the variant name in Rust; this file asserted
/// the token equalled `Kind`'s rendering and that each gate contained the substring `refusal.kind,
/// refusal.message`. Neither mentioned the delimiter. Measured: changing a gate's format string to `merge
/// message: {:?} — {}` left all five directions green while `grep -q "(Violation)"` matched nothing, so every
/// violation would have reported as the unjudged class — verbatim the failure the replaced direction's own doc
/// comment said it existed to prevent.
///
/// A channel has no delimiter to forget. Two scalars travel: the variable name and the class spelling, and both
/// are compared here against the module the gates call.
#[test]
fn each_wrapper_uses_the_channel_the_gates_report_on() {
    let Some(root) = workspace_root() else {
        return;
    };
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        for (name, expected) in [
            ("GATE_VERDICT_ENV", verdict_channel::ENV.to_string()),
            (
                "GATE_VIOLATION_CLASS",
                verdict_channel::rendered(Kind::Violation),
            ),
        ] {
            let declared = text
                .lines()
                .find_map(|line| line.trim().strip_prefix(&format!("{name}=")))
                .unwrap_or_else(|| {
                    panic!(
                        "{wrapper} declares no `{name}`, so the class it reports for a failing gate rests on \
                         nothing this check can compare"
                    )
                });
            assert_eq!(
                declared, expected,
                "{wrapper} uses `{declared}` for {name} while `kanhe::verdict_channel` defines `{expected}`"
            );
        }
        // The variable must actually be handed to the gate, not merely declared. Declared and unused would make
        // the file absent for every run, so every violation would report as unjudged.
        assert!(
            text.contains(&format!("{}=$verdict_file", verdict_channel::ENV)),
            "{wrapper} declares the channel and never opens it for the gate, so no verdict could ever arrive"
        );
    }
}

/// Each gate reports on the channel before it fails.
///
/// The other half. The scalars could match perfectly while no gate ever wrote the file, which would make every
/// failing gate read as unjudged — and the direction above would still pass. Held by position, not by the bare
/// call: the report must sit in the arm that has a refusal, above the failure.
#[test]
fn each_gate_reports_its_class_before_it_fails() {
    let Some(root) = workspace_root() else {
        return;
    };
    for gate in [
        "crates/kanhe/tests/merge_message.rs",
        "crates/kanhe/tests/publish_source.rs",
    ] {
        let text = read(&root, gate);
        let arm = text.find("Err(refusal) => {").unwrap_or_else(|| {
            panic!("{gate} has no refusal arm for a wrapper to read a class from")
        });
        let rest = &text[arm..];
        let reported = rest
            .find("verdict_channel::report(refusal.kind)")
            .unwrap_or_else(|| panic!("{gate} does not report its refusal class on the channel"));
        let failed = rest
            .find("panic!")
            .unwrap_or_else(|| panic!("{gate}'s refusal arm does not fail"));
        assert!(
            reported < failed,
            "{gate} fails before reporting its class, so the wrapper would find no verdict"
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
        // Matched as a STATEMENT, not as a whole line. Requiring the trimmed line to equal `exit 1` missed
        // `… || exit 1`, `exit 1;` and `[[ … ]] && exit 1` — measured, adding `if [[ ! -f $x ]]; then exit 1; fi`
        // left the count at one and the new site escaped both this and the window check below. Tightening the
        // detector while the requirement says *any* violation-class exit is the false negative this file is for.
        // ONE region, read once and shared by the scan and the window below. Two scans re-deciding it is the
        // shape `kanhe::region` was written to end, and it had already cost this file a disagreement: the scan
        // excluded comments while the `positioned_lines` window did not.
        let source = Source::of(text.clone());
        let executed = source.shell();
        let sites: Vec<(usize, String)> = executed
            .numbered_lines()
            .filter(|(_, line)| {
                line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                    .zip(
                        line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                            .skip(1),
                    )
                    .any(|(a, b)| a == "exit" && b == "1")
            })
            .map(|(number, line)| (number, line.to_string()))
            .collect();
        assert_eq!(
            sites.len(),
            1,
            "{wrapper} exits the violation class at {} site(s); only a gate's own verdict may, and every \
             could-not-read stop belongs to the unjudged class: {:?}",
            sites.len(),
            sites.iter().map(|(line, _)| line).collect::<Vec<_>>()
        );
        // EVERY site must sit inside the branch that read the gate's verdict, not merely somewhere after it.
        // Checking `sites[0]` alone was a second way for a new site to escape: the count would have to fail
        // first, and it did not.
        for (line, _) in &sites {
            // From the SAME region as the scan above, so the two cannot disagree about what counts. A comment
            // naming the class within five lines of a misplaced exit used to satisfy this.
            let window: String = executed
                .numbered_lines()
                .filter(|(number, _)| *number < *line && *number + 6 > *line)
                .map(|(_, text)| text)
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                window.contains("GATE_VIOLATION_CLASS"),
                "{wrapper}:{line} exits the violation class without having read the gate's verdict, so it \
                 reports a disagreement no judgement formed:\n{window}"
            );
        }
    }
}

/// Every acquisition a wrapper makes is guarded, so a failing tool cannot choose the exit class.
///
/// `var=$(tool …)` under `set -e` exits with the TOOL's status and only the tool's stderr. Measured, a failing
/// commits read left the merge wrapper exiting **91** with nothing of its own said — a class that is neither of
/// the two it defines, carrying the tool's words for a fact about the wrapper. Four acquisitions were unguarded,
/// and the direction covering one of them passed because it asserted only that the wrapper failed.
///
/// **The corpus is every command substitution, and names no tool.** It used to be the acquisitions invoking
/// `gh` or `cargo` — a list of the tools someone had thought of, with a helper beside it for reading past an
/// environment prefix to find the tool's name. `repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)`, the
/// first statement of *both* wrappers and the one that locates the gate, invoked neither and so was never
/// examined. It was unguarded, and measured, a failed `cd` under `set -e` exits **1** — the class that means a
/// gate ran and refused, reported by a wrapper whose gate had not been found. A sweep that exists to stop a
/// tool choosing the class was letting one choose it, and the wrong class at that. A command substitution is
/// the shape that carries the defect; what it invokes is no part of the property, so the tool test and its
/// helper are gone rather than extended.
///
/// A failed acquisition is **refused** or **given a value**, never ignored: `|| verdict=""` supplies a
/// fallback and is handled exactly as `|| cannot_judge` is. `|| true` is neither, and is not admitted.
#[test]
fn every_acquisition_is_guarded_so_the_tool_cannot_choose_the_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        let mut unguarded = Vec::new();
        let mut examined = 0usize;
        let source = Source::of(text.clone());
        // **One region, laid back out at its own positions.** The corpus came from `shell()` while the
        // continuation walk read `text.lines()` — two scans of one file disagreeing about what counts as
        // executed. A tail comment mentioning `cannot_judge` on an acquisition line would have marked it
        // guarded, which is the region confusion `repository-checks` names a defect whether or not either
        // scan currently admits a wrong answer. A dropped comment line becomes `""`, which ends no
        // continuation, so the walk stops there and the acquisition reports unguarded — loud, and the safe
        // direction for a wrapper standing in front of an irreversible act.
        //
        // Through `positioned_lines` rather than built here, and joined by `gate_identity::logical_lines`
        // rather than by a second copy of the shell's continuation rule. Both halves were hand-rolled at the
        // two sites that need them and both pairs disagreed: the layout half was unified first, and this —
        // the join — kept a `trim_end().strip_suffix('\\')` that continues a line ending in
        // backslash-then-whitespace. Measured, bash does not: `echo A \\ ` then `echo B` runs **two**
        // commands. Over-joining here reports an unguarded acquisition as guarded, because the pulled-in text
        // can carry the very token the guard is recognised by.
        let lines = source.shell().positioned_lines();
        for (number, statement) in kanhe::gate_identity::logical_lines(&lines.join("\n")) {
            // An assignment whose value is a command substitution. Read on the whole statement, because the
            // guard is part of it: both wrappers spread the gate acquisition across seven lines with its
            // `|| {` on the last.
            let Some((left, _)) = statement.split_once("=$(") else {
                continue;
            };
            // The assigned name: the last whitespace-separated word before `=$(`, so `local x=$(…)` names
            // `x` rather than `local x`. `rsplit` always yields at least one piece — measured, `""` and `" "`
            // both give `Some("")` — so the fallback names no state any input can reach, and dressing it as
            // `left.trim()` claimed otherwise while evaluating the trim twice.
            let variable = left
                .trim()
                .rsplit(char::is_whitespace)
                .next()
                .unwrap_or_default();
            let guarded = statement.contains("cannot_judge")
                || statement.contains("|| {")
                || statement.contains(&format!("|| {variable}="));
            examined += 1;
            if !guarded {
                unguarded.push(format!("{wrapper}:{number}"));
            }
        }
        // **Per wrapper, before the verdict.** `unguarded.is_empty()` is satisfied by a corpus that collapsed
        // to nothing exactly as it is by one that is clean, and the two are opposite facts. Every sibling
        // direction here already guards its own corpus this way; this one asserted only the finding.
        assert!(
            examined > 0,
            "{wrapper}: no acquisition entered the corpus, so this direction would report clean over nothing \
             — a wrapper standing in front of an irreversible act must not be judged by an empty reading"
        );
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
