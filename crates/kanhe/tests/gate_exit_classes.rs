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

use std::collections::BTreeSet;
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

/// Every test target that spawns a process itself, and what each spawns it for.
///
/// **One helper lived twice and the convergence took one copy.** `release_coherence.rs` and
/// `publish_source.rs` each held the same `hermetic("git")`-plus-assert runner — byte-identical past a doc
/// comment — and when the fixture dates were extracted, only the file the work was already in was
/// converged. Every fixture commit in the other kept taking its dates from the clock. Reading a file finds
/// the copy being edited; reading the pair finds the copy that is not.
///
/// **The detector enumerated spellings, and was one short three rounds running.** `hermetic(`, then
/// `Command::new("git")`, then `Command::new(args[0])` — the program-as-value form, which
/// `kanhe::hermetic_git`'s own header had already recorded as one of the two variants it converged, before
/// this guard was written. A detector keyed on *how* something is written will keep being one form short of
/// a requirement about *what is done*; each round the requirement was right and the reach was not.
///
/// So the question is the one with a single syntactic form and no knowledge of the program: **does this
/// target spawn a process itself**. It no longer has to know how `git` is spelled, or that it is `git` —
/// and the earlier form's own argument, that an allowlist is stricter than a denylist, now applies to the
/// detector as well as to the set it feeds. The cost is three more members out of eighteen files.
///
/// The purpose beside each path is prose with no producer: a reader's aid for whoever adds the next one,
/// not a fact this direction holds. What it holds is membership.
const TARGETS_SPAWNING_A_PROCESS: [(&str, &str); 22] = [
    (
        "crates/kanhe/tests/bound_register.rs",
        "git: enumerates, and builds a scratch repository's tree",
    ),
    (
        "crates/kanhe/tests/capability_subjects.rs",
        "git: enumerates and reads a scratch repository's state; its one commit goes through the builder",
    ),
    ("crates/kanhe/tests/census.rs", "git: enumerates"),
    (
        "crates/kanhe/tests/gate_exit_classes.rs",
        "git: two enumerations — the test targets this direction reads, and the tracked scripts the \
         wrapper direction beside it reads",
    ),
    (
        "crates/kanhe/tests/gate_identity.rs",
        "git, through a program-as-value runner: enumerates the scripts and the tracked Markdown",
    ),
    ("crates/kanhe/tests/law_restatement.rs", "git: enumerates"),
    (
        "crates/kanhe/tests/merge_message.rs",
        "this test binary itself, to re-run one direction in a child process",
    ),
    (
        "crates/kanhe/tests/merge_workflow.rs",
        "bash, to run the wrapper; git, to initialise a fixture",
    ),
    (
        "crates/kanhe/tests/observation_bound_model.rs",
        "git: enumerates",
    ),
    ("crates/kanhe/tests/one_spelling.rs", "git: enumerates"),
    (
        "crates/kanhe/tests/pin_bites.rs",
        "git: enumerates and reads a blob back through a program-as-value runner, and removes the worktree \
         it added",
    ),
    (
        "crates/kanhe/tests/projection_register.rs",
        "git: enumerates, and builds a scratch repository's tree",
    ),
    (
        "crates/kanhe/tests/publish_source.rs",
        "git: reads `rev-parse` and a tracked-path probe; its commits and tags go through the builder",
    ),
    (
        "crates/kanhe/tests/publish_workflow.rs",
        "bash, to run the publish wrapper",
    ),
    (
        "crates/kanhe/tests/reference_integrity.rs",
        "git: enumerates, initialises fixtures, reads the log, and asks about exclusion",
    ),
    ("crates/kanhe/tests/refusal_register.rs", "git: enumerates"),
    (
        "crates/kanhe/tests/whitespace_hygiene.rs",
        "git: enumerates",
    ),
    (
        "crates/kanhe/tests/workspace_isolation.rs",
        "git: enumerates, and builds a scratch repository's tree",
    ),
    (
        "crates/shengmo/tests/examples_suite.rs",
        "cargo, to build and run each example; git, to enumerate the examples directory",
    ),
    (
        "crates/shengmo/tests/family_coverage.rs",
        "git: enumerates the published family's sources",
    ),
    (
        "crates/shengmo/tests/self_governance.rs",
        "cargo, as a program-as-value, to read this workspace's metadata",
    ),
    (
        "crates/tianheng/tests/baseline_cli.rs",
        "the shell's own binary, as a program-as-value, to run the delivered CLI against a fixture",
    ),
];

/// The declared set of targets spawning a process equals the set the tree carries.
#[test]
fn no_test_target_spawns_a_process_unnamed() {
    let Some(root) = workspace_root() else {
        return;
    };
    let declared: BTreeSet<String> = TARGETS_SPAWNING_A_PROCESS
        .iter()
        .map(|(path, _)| (*path).to_string())
        .collect();
    assert_eq!(
        declared.len(),
        TARGETS_SPAWNING_A_PROCESS.len(),
        "a path is declared twice, so the comparison below is over fewer targets than the list holds"
    );
    let listing = std::process::Command::new("git")
        // **Every test target in the workspace, which is the noun the requirement uses.** The corpus was
        // `crates/kanhe/tests` from the first form, when the finding was two files in that directory, and
        // every widening since asked *what to look for* rather than *where to look* — so the spelling axis
        // and the verb axis were each closed while the set equality went on passing over a corpus the
        // requirement does not describe. Four targets outside that directory spawn a process and two run
        // `git` directly, in the crates whose own gates this guard protects.
        .args(["ls-files", "-z", "crates"])
        .current_dir(&root)
        .output()
        .expect("git ls-files is runnable");
    assert!(
        listing.status.success(),
        "could not enumerate the test targets, so this direction would report clean over nothing"
    );
    let paths: Vec<String> = String::from_utf8_lossy(&listing.stdout)
        .split('\0')
        // An integration test target is `crates/<member>/tests/<name>.rs` — the shape cargo compiles as its
        // own binary. Matched by shape rather than by a git pathspec glob, whose `*` crosses `/` and would
        // also take a nested fixture.
        .filter(|path| {
            let parts: Vec<&str> = path.split('/').collect();
            parts.len() == 4 && parts[0] == "crates" && parts[2] == "tests" && path.ends_with(".rs")
        })
        .map(str::to_string)
        .collect();
    assert!(
        !paths.is_empty(),
        "no test target entered the corpus, so this direction would report clean over nothing"
    );

    let mut reaching: BTreeSet<String> = BTreeSet::new();
    for path in &paths {
        let text = std::fs::read_to_string(root.join(path))
            .unwrap_or_else(|err| panic!("cannot read {path}: {err}"));
        // Executed text, so a doc comment naming a call is not read as one — and by position rather than by
        // the bare marker, because this direction's own source is in the corpus it reads and holds both
        // markers as literals. A call has a boundary before it where the literal has a quote, which is the
        // argument `refusal_register` makes for `::expect(` against its own panic messages.
        let source = Source::of(&text);
        let executed = source.rust();
        // Not preceded by a quote, so this file does not match its own marker literals — and not preceded
        // by an identifier character either, so `PhantomCommand::new(` is a different type's constructor
        // rather than a spawn. A path qualifier ends in `:` and a bare call in whitespace, so both real
        // spellings survive the boundary. Found by a perturbation that renamed the type and did not move
        // the verdict, which is a probe that was measuring nothing.
        let opens = |line: &str, marker: &str| {
            line.match_indices(marker).any(|(at, _)| {
                at == 0 || {
                    let before = line.as_bytes()[at - 1];
                    before != b'"' && !before.is_ascii_alphanumeric() && before != b'_'
                }
            })
        };
        if executed
            .lines()
            .any(|line| opens(line, "Command::new(") || opens(line, "hermetic("))
        {
            reaching.insert(path.clone());
        }
    }
    assert_eq!(
        declared, reaching,
        "the test targets spawning a process differ from the set named here. A target that gains one must \
         be named with what it spawns, and a name that outlives its reason must go — the helper this guards \
         lived twice and the convergence took one copy"
    );
}

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
        // **Only the scalar a wrapper actually READS is declared.** `GATE_VERDICT_ENV` was declared beside
        // this one and never read: the invocation writes the variable name literally, because a shell cannot
        // expand one into an environment-assignment prefix. So the declaration was a second spelling of a
        // token the assertion below already pins against `verdict_channel::ENV` — dead in the shell, and
        // held alive here by an assertion demanding it exist. Both are gone; the pin that does the work
        // stays.
        // **Both classes, because both decide something.** The violation class decides which exit a failing
        // gate produces; the clean class decides whether a *passing* run judged anything at all. The second
        // was missing while the gate wrote nothing on its clean arm, and a run that returned without judging
        // was indistinguishable from one that agreed.
        for (name, expected) in [
            (
                "GATE_VIOLATION_CLASS",
                verdict_channel::rendered(Kind::Violation),
            ),
            ("GATE_CLEAN_CLASS", verdict_channel::CLEAN.to_string()),
        ] {
            let declared = text
                .lines()
                .find_map(|line| line.trim().strip_prefix(&format!("{name}=")))
                .unwrap_or_else(|| {
                    panic!(
                        "{wrapper} declares no `{name}`, so the class it reads off the channel rests on \
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

/// Each gate leaves through the one exit that reports.
///
/// **This replaced a scan for an arm, and the replacement is why.** It located `Err(refusal) => {` by
/// substring and asserted the report preceded the panic *within that arm* — so every other exit of the
/// harness owed nothing, and a subject supplied as bytes the gate could not read left through a clean
/// `return`, writing no class, exiting `0`, and reaching `gh pr merge`.
///
/// The pairing of *reached the channel* with *fails the run* is now a property of
/// `kanhe::verdict_channel::Verdict`, held over the whole enum by `every_refusing_verdict_reaches_the
/// channel`. What is left for this direction is the half a type cannot carry: that each gate actually
/// delegates to it, rather than deciding for itself.
#[test]
fn each_gate_leaves_through_the_verdict_channel() {
    let Some(root) = workspace_root() else {
        return;
    };
    for gate in [
        "crates/kanhe/tests/merge_message.rs",
        "crates/kanhe/tests/publish_source.rs",
    ] {
        let text = read(&root, gate);
        let executed = Source::of(text.clone());
        // The gate's own `#[test]` body, which must be one delegation and nothing else. Read from the
        // executed region, so a commented-out call cannot satisfy it.
        assert!(
            executed.rust().contains("kanhe::verdict_channel::deliver("),
            "{gate} does not deliver its verdict through the channel, so what it reports on failure and \
             what it reports on success are its own decisions rather than one"
        );
        // A harness writing the channel itself is now unconstructible rather than forbidden: the only
        // writer is private to `verdict_channel`, so this asked a question the module boundary answers.
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
