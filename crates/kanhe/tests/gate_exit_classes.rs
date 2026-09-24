//! Repository check: a wrapper's exit class agrees with the gate it fronts.
//!
//! Two facts live in two languages here. `refusal::Kind` types the distinction a gate draws — a source that
//! **disagrees** against one that **could not be read** — and a shell wrapper turns a gate's failure into a
//! process exit class. The gate writes its class to a verdict file the wrapper names, the shared library reads
//! it back, and the wrapper exits with a code `kanhe::verdict_channel` owns — so a Rust enum's rendering, a
//! Rust code table and the library's declarations must agree. Two places that must agree is the shape this
//! repository has spent a window replacing, so it is checked rather than commented.
//!
//! **What went wrong without it.** Five could-not-read conditions in `scripts/merge-pr.sh` were split across
//! both exit classes with no stated rule, and two of the facts on the `1` side are ones
//! `merge_message_gate::judge` types as cannot-judge — so the wrapper reported as a disagreement what its own
//! gate calls unjudgeable. No direction could have caught it: the ones covering those sites asserted only that
//! the wrapper failed, which cannot see `1` from `2`.

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kanhe::refusal::Kind;
use kanhe::region::Source;
use kanhe::verdict_channel;
use kanhe::wrapper_parser;

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
const TARGETS_SPAWNING_A_PROCESS: [(&str, &str); 30] = [
    (
        "crates/kanhe/tests/workflow_model.rs",
        "bash: `compgen`, to hold the declared shell-own words against the shell that owns them",
    ),
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
        "crates/kanhe/tests/doc_provenance.rs",
        "git: enumerates the published crates' sources, through the hermetic builder; cargo, to read this \
         workspace's metadata for the both-ways check against the text reader",
    ),
    (
        "crates/kanhe/tests/gate_exit_classes.rs",
        "git: two enumerations — the test targets this direction reads, and the tracked scripts the \
         wrapper direction beside it reads",
    ),
    (
        "crates/kanhe/tests/hermetic_invocations.rs",
        "git: enumerates the tracked Rust it reads; cargo, to run each declared site's proving direction",
    ),
    (
        "crates/kanhe/tests/gate_identity.rs",
        "git, to enumerate the scripts and the tracked Markdown; cargo, to list a target's tests and to \
         read this workspace's metadata — both through one program-as-value runner",
    ),
    ("crates/kanhe/tests/law_restatement.rs", "git: enumerates"),
    (
        "crates/kanhe/tests/line_comment_purity.rs",
        "git: enumerates the published crates' sources and initialises a fixture repository, both \
         through the hermetic builder",
    ),
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
        "git, to enumerate and to read a record blob back; cargo, to build each mutated checkout — both \
         through one program-as-value runner — and git again to remove the worktree it added",
    ),
    (
        "crates/kanhe/tests/projection_register.rs",
        "git: enumerates, and builds a scratch repository's tree",
    ),
    (
        "crates/kanhe/tests/release_coherence.rs",
        "git: initialises each fixture repository and writes its commits — every one through the shared \
         fixture builder, which is why no spelling this array's detector knew reached it",
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
        "crates/kanhe/tests/repeated_paragraph.rs",
        "git: enumerates the tracked Rust its sweep reads",
    ),
    (
        "crates/kanhe/tests/unreachable_branch.rs",
        "git: enumerates the tracked Rust sources its two sweeps read",
    ),
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
        "crates/tianheng/tests/attribute_spelling_differential.rs",
        "rustc, as the third party a differential over three readers cannot be: it decides whether a          generated spelling is legal Rust at all, and which file a live predicate makes the build contain",
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
    let paths = test_targets(&root);

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
        // **The whole module, not each spawning function it exports** — the fourth round of the defect the
        // doc above predicts. `hermetic_git::fixture` is itself a call site's spelling, and a
        // target whose only spawn were that would have gone undetected while `Command::new(` and
        // `hermetic(` both passed over it. Naming the module closes every entry point it has and every one
        // it gains. Two of its items — `failed` and `program_and_args` — spawn nothing, so a target reaching
        // only those would be over-declared; over-declaring is the safe direction here, and no target does
        // (measured: every file reaching this module also runs something through it).
        //
        // `hermetic(` stays beside it because an imported `hermetic` is spelled bare, with no module
        // qualifier to match.
        if executed.lines().any(|line| {
            opens(line, "Command::new(")
                || opens(line, "hermetic_git::")
                || opens(line, "hermetic(")
        }) {
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

/// Every tracked Rust file under `crates`, enumerated **once** for the two directions that read it.
///
/// One enumeration because two would be two corpora that must agree, and a file the second forgot would be
/// judged by one direction and not the other — the granularity defect this file's own directions exist to
/// close, reintroduced one level up.
/// Whether `line` opens a call to `marker`, rather than merely containing its text.
///
/// Not preceded by a quote, so a direction using this does not match its own marker literals — and not
/// preceded by an identifier character either, so `PhantomCommand::new(` is a different type's constructor
/// rather than a spawn. A path qualifier ends in `:` and a bare call in whitespace, so both real spellings
/// survive the boundary. Found by a perturbation that renamed the type and did not move the verdict, which is
/// a probe that was measuring nothing.
///
/// One owner because two directions ask it now. It was a closure inside the first, which is where a second
/// caller copies from.
fn opens(line: &str, marker: &str) -> bool {
    line.match_indices(marker).any(|(at, _)| {
        at == 0 || {
            let before = line.as_bytes()[at - 1];
            before != b'"' && !before.is_ascii_alphanumeric() && before != b'_'
        }
    })
}

fn tracked_rust(root: &Path) -> Vec<String> {
    let listing = kanhe::hermetic_git::tracked_paths(root, &["crates"]).unwrap_or_else(|failure| {
        panic!(
            "could not enumerate the tracked Rust ({failure:?}), so the directions over it would report \
             clean over nothing"
        )
    });
    let paths: Vec<String> = listing
        .into_iter()
        .filter(|path| path.ends_with(".rs"))
        .collect();
    assert!(
        !paths.is_empty(),
        "no tracked Rust entered the corpus, so the directions over it would report clean over nothing"
    );
    paths
}

/// The test targets among [`tracked_rust`].
///
/// **Every test target in the workspace, which is the noun the requirement uses.** The corpus was
/// `crates/kanhe/tests` from the first form, when the finding was two files in that directory, and every
/// widening since asked *what to look for* rather than *where to look* — so the spelling axis and the verb
/// axis were each closed while the set equality went on passing over a corpus the requirement does not
/// describe. Four targets outside that directory spawn a process and two run `git` directly, in the crates
/// whose own gates this guard protects.
fn test_targets(root: &Path) -> Vec<String> {
    // An integration test target is `crates/<member>/tests/<name>.rs` — the shape cargo compiles as its own
    // binary. Matched by shape rather than by a git pathspec glob, whose `*` crosses `/` and would also take
    // a nested fixture.
    let targets: Vec<String> = tracked_rust(root)
        .into_iter()
        .filter(|path| {
            let parts: Vec<&str> = path.split('/').collect();
            parts.len() == 4 && parts[0] == "crates" && parts[2] == "tests" && path.ends_with(".rs")
        })
        .collect();
    assert!(
        !targets.is_empty(),
        "no test target entered the corpus, so the direction over it would report clean over nothing"
    );
    targets
}

/// Every git subcommand whose answer an ignore file outside the repository changes, spelled as the whole
/// argument a caller passes.
///
/// **The whole literal, because a fragment matched prose and a diagnostic.** `publish_source.rs` builds the
/// string `"check-ignore exploded"` for a stub's failure and names the subcommand in two doc comments; none
/// of the three runs anything. A real caller passes the subcommand as one argument, so the closing quote is
/// what separates the two — and writing the quotes escaped here is also why this file does not match its own
/// array: the text on disk carries a backslash where a real call site carries the quote.
///
/// `status` earns its place through `--untracked-files`: it reports an untracked file only if nothing outside
/// the repository excludes it, which is the same channel `check-ignore` answers on directly.
const AMBIENT_IGNORE_READS: [&str; 5] = [
    "\"check-ignore\"",
    "\"--others\"",
    "\"--untracked-files\"",
    "\"--untracked-files=all\"",
    // **Staging, not only querying — and the source of truth named this one FIRST.** `hermetic_git`'s
    // ambient table records the measurement in both directions, and its leading sentence is `git add -A`
    // with the three file variables set leaving the matching file *untracked* — a fixture silently built
    // without a file it named. Every other entry here asks git what it would ignore; this one is git ACTING
    // on the answer, which is the half that corrupts a fixture rather than merely misreporting one.
    "\"add\"",
];

/// The setting whose absence leaves the channel open, however it comes to be named.
const NEUTRALISER: &str = "core.excludesFile";

/// The one file that must leave the channel open, and why.
///
/// It pins the channel's existence by *difference*: one command with the setting closed, one without, and the
/// assertion is that the two answers differ. The control cannot name the setting — a control that closed the
/// channel would be comparing a value against itself, which is the inert-probe shape this repository refuses
/// wherever it finds one. So this file legitimately runs the read bare, and the direction below says so by
/// name rather than by a pattern that would also excuse a judgement.
///
/// Held live below rather than trusted: an exception whose instance has gone is an exception that silently
/// widens what passes.
const CHANNEL_CONTROL: &str = "crates/kanhe/src/tests/hermetic_git.rs";

/// The direction whose existence earns [`CHANNEL_CONTROL`] its exception.
///
/// **Named, because holding the file was satisfied for the wrong reason.** The first guard asked only whether
/// that file still ran an ignore-sensitive read through a `Command` of its own — and a *different* test in
/// the same file spawns bare for a different property, so the guard went on passing with the ignore control
/// converted away. A protection can outlive its instance while looking green; what the exception is *for* is
/// this one direction, so this one direction is what is held.
const CHANNEL_CONTROL_PINS: &str =
    "fn an_ignore_file_outside_the_repository_cannot_reach_a_hermetic_command(";

/// No judgement runs a subcommand an ambient ignore file answers differently with that channel left open.
///
/// **The measurement, on this machine's git, both directions.** With `$XDG_CONFIG_HOME/git/ignore` naming a
/// path, `git check-ignore -q -- <that path>` exits `0` — *ignored* — and `git add -A` leaves the file
/// **untracked**, so a fixture is built without a file it named. With `core.excludesFile` named as
/// `/dev/null`, the query exits `1` and the file is added. Neutralising the config *files* alone does not do
/// it, because that path is the default git uses when no config file names one.
///
/// **What it cost.** `reference_integrity::ignored` asked `check-ignore` through a bare `Command::new("git")`,
/// on the real repository, on the verdict path — and *ignored* there means the offence is **not** reported. So
/// an entry in whoever's personal ignore file excused a stale path reference: an under-refusal whose verdict
/// depended on who ran the gate, in the capability whose stated Purpose is that it does not. `hermetic` was
/// reachable throughout and nothing required it.
///
/// **Two ways to close it, and this accepts either**, because after the repair above the builder closes it
/// for every caller and requiring the flag as well would refuse correct code and call the redundancy a fix:
///
/// - the file names the setting itself, which the two judgements whose verdict turns on the answer do; or
/// - the file starts no process itself, so every command it runs is the builder's and carries the setting.
///
/// A file that does neither is running an ignore-sensitive read through a `Command` it built itself, with the
/// channel open — which is the defect, exactly.
///
/// **This does not reach**, each limit measured rather than supposed:
///
/// - **File granularity.** A file that names the setting once and spawns a bare `Command` for something else
///   passes. Per-call would refuse `publish_source_gate`, where one wrapper closes the channel for every
///   judgement in the file, so the tighter rule would be wrong on the site that was already right.
/// - **A literal argument.** A subcommand composed at run time — `format!`, a variable, a `const` — is not
///   seen. Every call site in this workspace spells it as a literal (measured).
/// - **`.git/info/exclude`.** Inside the repository, so no config setting reaches it.
///   `publish_source_gate::hidden_by_the_checkout` classifies rather than refuses because of it.
/// - **One marker's reach, individually.** The vacuity guard holds that *some* marker still matches, not that
///   each does — measured by removing one and watching this stay green. Holding each would be the stronger
///   shape and is not adoptable while the array admits `"--untracked-files"` as its own argument, a spelling
///   git accepts with no call site here: the repair would be to stop admitting it.
#[test]
fn no_judgement_reads_an_ambient_ignore_file() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut reading = 0usize;
    let mut control_seen = false;
    let mut open = Vec::new();
    for path in tracked_rust(&root) {
        let text = read(&root, &path);
        // Executed text, so the two doc comments naming the subcommand are not read as calls.
        let source = Source::of(&text);
        let executed = source.rust();
        let lines: Vec<&str> = executed.lines().collect();
        if !AMBIENT_IGNORE_READS
            .iter()
            .any(|marker| lines.iter().any(|line| line.contains(marker)))
        {
            continue;
        }
        reading += 1;
        if lines.iter().any(|line| line.contains(NEUTRALISER)) {
            continue;
        }
        if !lines.iter().any(|line| opens(line, "Command::new(")) {
            continue;
        }
        if path == CHANNEL_CONTROL {
            control_seen = true;
            continue;
        }
        open.push(path);
    }
    // Without this the direction reports clean the moment a rename or a rewrite takes the last call site out
    // of its reach — which is the vacuity every enumeration in this file guards against.
    assert!(
        reading > 0,
        "no file in the corpus runs a subcommand an ambient ignore file answers, so this direction would \
         report clean over nothing — the reach was lost, not the risk"
    );
    // The exception is held against its own instance, both halves: that it is still being used, and that the
    // direction it exists for is still there. Either alone passes for the wrong reason.
    assert!(
        control_seen,
        "`{CHANNEL_CONTROL}` is named as the one file that must leave this channel open, and it no longer \
         runs an ignore-sensitive read through a `Command` of its own — so the exception excuses nothing and \
         should say so by being removed"
    );
    assert!(
        opens(&read(&root, CHANNEL_CONTROL), CHANNEL_CONTROL_PINS),
        "`{CHANNEL_CONTROL}` is excused because `{CHANNEL_CONTROL_PINS}` pins the channel by difference, and \
         that direction is no longer there under that name. Either it moved, in which case name where, or \
         the exception is now excusing a file with nothing to pin"
    );
    assert!(
        open.is_empty(),
        "a judgement runs a subcommand an ambient ignore file answers differently through a `Command` it \
         built itself, without naming `{NEUTRALISER}` — so its verdict depends on who runs it, and for an \
         ignore query the ambient answer is the one that excuses an offence:\n{}",
        open.join("\n")
    );
}

fn read(root: &Path, path: &str) -> String {
    std::fs::read_to_string(root.join(path)).unwrap_or_else(|err| {
        panic!("cannot read {path}, so its exit classes were never compared: {err}")
    })
}

/// Every `exit` in the wrappers and their library names a code `kanhe::verdict_channel` owns, and each code is
/// chosen at **one** site.
///
/// An `exit` statement IS the choice of class, so reading every one decides the property. Every exit is one of
/// two shapes: `exit "$WRAPPER_EXIT_<NAME>"` for a declared code, or the literal `2` inside a wrapper's bootstrap
/// guard — the `if ! source … fi` block, the one stop that runs before the library that declares the codes is
/// loaded. Anything else — a numeral elsewhere, a bare `exit`, another variable — is refused, which is what a
/// count of the two literals could not do: `exit "$ANY"` and `exit 3` were invisible to it.
///
/// **Found as a word, wherever it stands.** Every word whose value is `exit` — what quote removal leaves of it,
/// under any quoting the shell removes — is held to those two shapes, in a command's position or not. Deciding
/// *where a command begins* needs the shell's grammar: separators, reserved words, precommand words, case
/// patterns. A reader modelling that answered wrongly one production at a time, and a missing production is an
/// exit that passes; a word needs only quote removal, which is finite and done. So `printf eval exit 3` is
/// refused as well — `exit` as another command's argument is held to the same form, and the repair is to quote
/// it into a longer word, as a refusal message already is. `EXIT_SHAPES` is the whole of it, run.
///
/// **What this does not reach**: a class chosen by an unguarded command's own status, which the ERR trap and
/// `every_acquisition_is_guarded_so_the_tool_cannot_choose_the_class` close; a refusal spelled `return`
/// inside a function whose caller then exits, which has no instance and would need block structure to see;
/// and a command name the text does not spell — a variable, or a string another command runs — which is the
/// stated bound [`a_command_name_computed_when_the_line_runs_is_not_read`] pins.
#[test]
fn each_wrapper_chooses_its_exit_class_in_one_place() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY;
    let mut files: Vec<&str> = WRAPPERS.to_vec();
    files.push(library);
    for script in files {
        let text = read(&root, script);
        let source = Source::of(text);
        let lines: Vec<(usize, String)> = source
            .shell()
            .numbered_lines()
            .map(|(number, line)| (number, line.to_string()))
            .collect();
        let bootstrap = bootstrap_region(&lines);
        let mut chosen: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut refused = Vec::new();
        for (number, argument) in exit_sites(&executed_text(&lines)) {
            let named = argument
                .strip_prefix("\"$WRAPPER_EXIT_")
                .and_then(|rest| rest.strip_suffix('"'))
                .filter(|name| DECLARED_EXITS.iter().any(|(declared, _)| declared == name));
            match named {
                Some(name) => chosen.entry(name.to_string()).or_default().push(number),
                None if argument == "2" && bootstrap.contains(&number) => {
                    chosen
                        .entry("bootstrap".to_string())
                        .or_default()
                        .push(number);
                }
                None => refused.push(format!("  {script}:{number}: exit {argument}")),
            }
        }
        assert!(
            refused.is_empty(),
            "{script} exits through a code no declaration owns — every exit names `$WRAPPER_EXIT_<NAME>`, \
             and the literal `2` belongs only to a wrapper's bootstrap guard:\n{}",
            refused.join("\n")
        );
        let expected: Vec<&str> = if script == library {
            // Each code once: the misuse guard, `cannot_judge`, and `exit_for_the_gates_refusal`.
            DECLARED_EXITS.iter().map(|(name, _)| *name).collect()
        } else {
            // A wrapper chooses nothing after its `source`: every stop delegates to the library.
            vec!["bootstrap"]
        };
        let mut found: Vec<&str> = chosen.keys().map(String::as_str).collect();
        found.sort_unstable();
        let mut wanted = expected.clone();
        wanted.sort_unstable();
        assert_eq!(
            found, wanted,
            "{script} chooses these codes {chosen:?}; it should choose exactly {expected:?}"
        );
        for (name, sites) in &chosen {
            assert_eq!(
                sites.len(),
                1,
                "{script} chooses `{name}` at {} sites {sites:?}; one site is the classification chosen once, \
                 and a second is it chosen twice",
                sites.len()
            );
        }
    }
}

/// The exit codes the library declares, each with the value `kanhe::verdict_channel` owns for it.
const DECLARED_EXITS: [(&str, u8); 3] = [
    (
        "VIOLATION",
        kanhe::verdict_channel::wrapper_exit(Kind::Violation),
    ),
    (
        "UNJUDGED",
        kanhe::verdict_channel::wrapper_exit(Kind::CannotJudge),
    ),
    ("MISUSE", kanhe::verdict_channel::LIBRARY_MISUSE),
];

/// What [`exit_arguments`] reads from each shape a wrapper line can take, the shapes it declines included.
///
/// The table is the description of the reader: a row cannot drift from the code, because it runs. No row asks
/// where a command begins, because the reader does not: an `exit` word in a case arm, a condition, a pipeline
/// or another command's arguments reads the same way.
const EXIT_SHAPES: &[(&str, &[&str])] = &[
    ("exit 2", &["2"]),
    (
        "    exit \"$WRAPPER_EXIT_UNJUDGED\"",
        &["\"$WRAPPER_EXIT_UNJUDGED\""],
    ),
    ("command || exit 1", &["1"]),
    ("then exit 1; fi", &["1"]),
    ("exit 1;", &["1"]),
    // A bare exit, its terminator on the word or standing after it, reads with no argument.
    ("{ exit; }", &[""]),
    ("exit ;;", &[""]),
    ("(exit 3)", &["3"]),
    ("( exit 3 )", &["3"]),
    ("*) exit 3 ;;", &["3"]),
    ("\"(odd)\") exit 3 ;;", &["3"]),
    ("if exit 3; then :; fi", &["3"]),
    ("true | exit 3", &["3"]),
    ("time -p exit 3", &["3"]),
    ("printf eval exit 3", &["3"]),
    ("for exit in a; do :; done", &["in"]),
    // Every quoting the shell removes spells `exit`, ANSI-C quoting and each of its escapes included.
    ("\"exit\" 3", &["3"]),
    ("e\\xit 3", &["3"]),
    ("x) $'exit' 3 ;;", &["3"]),
    ("$'\\x65xit' 3", &["3"]),
    ("$'\\145xit' 3", &["3"]),
    ("$'\\u0065xit' 3", &["3"]),
    ("$'e\\x78it' 3", &["3"]),
    // A word whose value is longer than `exit` is not one: a message's prose, one quoted word.
    ("tell \"reach gh, and exit 0 having merged nothing\"", &[]),
    ("judged+=$'\\n\\n'$body", &[]),
    ("printf '%s' $(date +%s) exit_code", &[]),
    // Quotes and continuations span lines, so the text is read whole: a message's later line is still inside
    // its quote, a backslash-newline joins `exit` to its argument, and a newline ends a bare `exit`.
    (
        "refuse \"$1\" \"it would\nreach gh, and exit 0 having merged nothing\"",
        &[],
    ),
    ("exit \\\n    3", &["3"]),
    ("exit\ncargo test", &[""]),
    // A command substitution's contents are text the shell runs, wherever the substitution stands.
    ("printf '%s' \"$(exit 3)\"", &["3"]),
    ("status=$(exit 3)", &["3"]),
    ("echo `exit 3`", &["3"]),
    ("printf '%s' \"`exit 3`\"", &["3"]),
    ("printf '%s' \"$(exit)\"", &[""]),
    // A substitution is part of the word it stands in, so text beside it joins that word.
    ("printf '%s' $(printf x)exit 3", &[]),
    ("printf '%s' exit$(printf x) 3", &[]),
    ("printf '%s' `printf x`exit 3", &[]),
    // A parenthesis inside quotes is the word's text, not the shell's operator.
    ("printf \"(exit\" 3", &[]),
    ("exit \"$(printf 3)\"", &["\"$(printf 3)\""]),
    // A command name computed when the line runs is not the line's to read: the declared bound below.
    ("$stop 3", &[]),
];

#[test]
fn the_exit_reader_decides_every_shape_a_wrapper_line_takes() {
    for (line, expected) in EXIT_SHAPES {
        assert_eq!(
            exit_arguments(line),
            expected
                .iter()
                .map(|argument| argument.to_string())
                .collect::<Vec<_>>(),
            "the exit reader misreads `{line}`"
        );
    }
}

/// A command name the shell computes when the line runs is not read — a stated bound, shown rather than described.
///
/// `$stop 3` runs `exit 3` where `stop=exit`, and `eval "exit 3"`, `trap 'exit 3' EXIT` and `bash -c 'exit 3'` run it
/// from a string another command parses again: each is decided by a
/// value the line does not spell, so a reader of words has nothing to read them from. Every word the line does
/// spell as `exit` is read, [`EXIT_SHAPES`] says so, and this is what is left.
#[test]
fn a_command_name_computed_when_the_line_runs_is_not_read() {
    for line in [
        "$stop 3",
        "\"$stop\" 3",
        "eval \"exit 3\"",
        "eval \"$stop 3\"",
        "trap 'exit 3' EXIT",
        "bash -c 'exit 3'",
    ] {
        assert_eq!(
            exit_arguments(line),
            Vec::<String>::new(),
            "`{line}` names its command at run time, and the declared bound says it is not read"
        );
    }
}

/// The argument of every `exit` word in `text`, as written — empty for a bare `exit`.
fn exit_arguments(text: &str) -> Vec<String> {
    exit_sites(text)
        .into_iter()
        .map(|(_, argument)| argument)
        .collect()
}

/// Every `exit` word in `text`, with the line it stands on and its argument as written.
///
/// **Over the whole text, not a line at a time**, because a quote the shell opens on one line closes on a later
/// one: `merge-pr.sh` writes refusal messages across lines, and a line reader saw a message's *and exit 0* on
/// a continuation line as standing outside every quote. The argument is the next word, unless an operator — a
/// separator, a redirection, a newline — ends the command first.
fn exit_sites(text: &str) -> Vec<(usize, String)> {
    let words = shell_words(text);
    let mut sites = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if word.operator || word.value != "exit" {
            continue;
        }
        let argument = words
            .get(index + 1)
            .filter(|next| !next.operator)
            .map_or("", |next| next.written.as_str());
        sites.push((word.line, argument.to_string()));
    }
    sites
}

/// The executed text of a script, each executed line at its own line number and every other line empty, so a
/// word's line is the one a reader of the file finds it on.
fn executed_text(lines: &[(usize, String)]) -> String {
    let last = lines.last().map_or(0, |(number, _)| *number);
    let mut placed = vec![""; last];
    for (number, line) in lines {
        placed[number - 1] = line;
    }
    placed.join("\n")
}

/// One word of shell text, or one operator, where bash's own grammar puts the boundary.
///
/// **The boundaries are bash's definition, not a list grown to fit.** `bash(1)`, *DEFINITIONS*: a metacharacter
/// is space, tab, newline, `|`, `&`, `;`, `(`, `)`, `<` or `>`, and outside quotes each one ends a word; every one
/// but a blank also stands as an operator. So `(exit 3)`, `exit;` and `*) exit 3 ;;` put `exit` in a word of its
/// own, while `"(exit"` quotes its parenthesis into the word's value. A command substitution's contents are shell
/// text the shell runs, so they are read as words too, in order, wherever the substitution stands — unquoted,
/// inside double quotes, or between backquotes — while the substitution itself stays part of the word it stands
/// in, so `$(printf x)exit` is one word. A backslash before a newline joins the two lines.
struct ShellWord {
    /// Where the word begins in the text, which orders words read out of a substitution after the one holding it.
    start: usize,
    /// The line the word begins on, counted from one.
    line: usize,
    /// The word as written, quotes included — what an `exit` argument is compared as.
    written: String,
    /// What quote removal leaves of it — what the shell runs as a command name.
    value: String,
    /// Whether it is a metacharacter standing as an operator rather than a word.
    operator: bool,
}

/// bash's metacharacters, from `bash(1)`, *DEFINITIONS*.
const METACHARACTERS: [char; 10] = [' ', '\t', '\n', '|', '&', ';', '(', ')', '<', '>'];

fn shell_words(text: &str) -> Vec<ShellWord> {
    let mut tokens = Tokens {
        chars: text.chars().collect(),
        at: 0,
        line: 1,
        out: Vec::new(),
    };
    tokens.run(None);
    let mut words = tokens.out;
    words.sort_by_key(|word| word.start);
    words
}

struct Tokens {
    chars: Vec<char>,
    at: usize,
    line: usize,
    out: Vec<ShellWord>,
}

impl Tokens {
    fn take(&mut self) -> Option<char> {
        let ch = *self.chars.get(self.at)?;
        self.at += 1;
        if ch == '\n' {
            self.line += 1;
        }
        Some(ch)
    }

    fn push(&mut self, word: Option<ShellWord>) {
        // A word that is only a backslash-newline is a continuation standing between words, not a word.
        if let Some(word) = word.filter(|word| word.written != "\\\n") {
            self.out.push(word);
        }
    }

    /// Reads words and operators until `close` stands unquoted — the end of the substitution this run reads —
    /// or the text ends. A `)` closes only at the depth it opened, so a subshell inside a substitution is its own.
    fn run(&mut self, close: Option<char>) {
        let mut word: Option<ShellWord> = None;
        let mut depth = 0usize;
        loop {
            let (start, line) = (self.at, self.line);
            let Some(ch) = self.take() else { break };
            let closes = Some(ch) == close && (ch == '`' || depth == 0);
            if closes || METACHARACTERS.contains(&ch) {
                self.push(word.take());
                match ch {
                    '(' => depth += 1,
                    ')' => depth = depth.saturating_sub(1),
                    _ => {}
                }
                if ch != ' ' && ch != '\t' {
                    self.out.push(ShellWord {
                        start,
                        line,
                        written: ch.to_string(),
                        value: ch.to_string(),
                        operator: true,
                    });
                }
                if closes {
                    return;
                }
                continue;
            }
            let mut current = word.take().unwrap_or(ShellWord {
                start,
                line,
                written: String::new(),
                value: String::new(),
                operator: false,
            });
            current.written.push(ch);
            match ch {
                '\\' => {
                    if let Some(escaped) = self.take() {
                        current.written.push(escaped);
                        if escaped != '\n' {
                            current.value.push(escaped);
                        }
                    }
                }
                '\'' => {
                    while let Some(inner) = self.take() {
                        current.written.push(inner);
                        if inner == '\'' {
                            break;
                        }
                        current.value.push(inner);
                    }
                }
                // ANSI-C quoting, `$'…'`: its content is the value, each escape decoded as bash decodes it.
                '$' if self.chars.get(self.at) == Some(&'\'') => {
                    self.take();
                    current.written.push('\'');
                    while let Some(inner) = self.take() {
                        current.written.push(inner);
                        match inner {
                            '\'' => break,
                            '\\' => {
                                let from = self.at;
                                let decoded = ansi_c_escape(&self.chars, &mut self.at);
                                current.written.extend(&self.chars[from..self.at]);
                                current.value.extend(decoded);
                            }
                            _ => current.value.push(inner),
                        }
                    }
                }
                '$' if self.chars.get(self.at) == Some(&'(') => {
                    self.substitution(&mut current, ')')
                }
                '`' => self.substitution(&mut current, '`'),
                '"' => self.double_quoted(&mut current),
                _ => current.value.push(ch),
            }
            word = Some(current);
        }
        self.push(word);
    }

    /// A command substitution, its `$` or opening backquote already read: **one** reading, quoted or not.
    ///
    /// Its contents are shell text the shell runs, read as words of their own. The substitution itself is part of
    /// the word it stands in, held there as a placeholder, so text beside it joins that word — `$(printf x)exit` is
    /// one word, as it is to the shell — and the word's value is never `exit`.
    fn substitution(&mut self, word: &mut ShellWord, close: char) {
        let from = self.at;
        if close == ')' {
            self.take();
            word.value.push_str("$(…)");
        } else {
            word.value.push_str("`…`");
        }
        self.run(Some(close));
        word.written.extend(&self.chars[from..self.at]);
    }

    /// The rest of a double-quoted span, its opening quote already read. A substitution inside it is shell text,
    /// read as words of its own; the quoted word holds it as a placeholder, so its value is never `exit`.
    fn double_quoted(&mut self, word: &mut ShellWord) {
        while let Some(ch) = self.take() {
            word.written.push(ch);
            match ch {
                '"' => return,
                '\\' => {
                    if let Some(escaped) = self.take() {
                        word.written.push(escaped);
                        // Inside double quotes a backslash escapes only these; before anything else it stays.
                        if !matches!(escaped, '$' | '`' | '"' | '\\' | '\n') {
                            word.value.push('\\');
                        }
                        if escaped != '\n' {
                            word.value.push(escaped);
                        }
                    }
                }
                '$' if self.chars.get(self.at) == Some(&'(') => self.substitution(word, ')'),
                '`' => self.substitution(word, '`'),
                _ => word.value.push(ch),
            }
        }
    }
}

/// The character one ANSI-C escape names, read from just after its backslash; `at` is left past the escape.
///
/// **The whole of bash's list, because it is finite.** `bash(1)`, *QUOTING*, names every form: the single
/// letters, `\\`, `\'`, `\"`, `\?`, octal `\nnn`, hex `\xHH`, `\uHHHH`, `\UHHHHHHHH` and control `\cx`. A
/// reader decoding some of them is one spelling short of `exit` for as long as the rest exist, so none is left
/// out. Measured: `$'\x65xit'`, `$'\145xit'`, `$'\u0065xit'` and `$'e\x78it'` each run `exit` under bash 5.
/// An escape bash leaves as written — a backslash before any other character — is both characters.
fn ansi_c_escape(chars: &[char], at: &mut usize) -> Vec<char> {
    let Some(&first) = chars.get(*at) else {
        return vec!['\\'];
    };
    *at += 1;
    let digits = |at: &mut usize, radix: u32, most: usize| {
        let start = *at;
        while *at < chars.len() && *at - start < most && chars[*at].is_digit(radix) {
            *at += 1;
        }
        let text: String = chars[start..*at].iter().collect();
        u32::from_str_radix(&text, radix)
            .ok()
            .and_then(char::from_u32)
    };
    let named = match first {
        'a' => Some('\u{7}'),
        'b' => Some('\u{8}'),
        'e' | 'E' => Some('\u{1b}'),
        'f' => Some('\u{c}'),
        'n' => Some('\n'),
        'r' => Some('\r'),
        't' => Some('\t'),
        'v' => Some('\u{b}'),
        '\\' | '\'' | '"' | '?' => Some(first),
        _ => None,
    };
    if let Some(named) = named {
        return vec![named];
    }
    let decoded = match first {
        '0'..='7' => {
            *at -= 1;
            digits(at, 8, 3)
        }
        'x' => digits(at, 16, 2),
        'u' => digits(at, 16, 4),
        'U' => digits(at, 16, 8),
        'c' => chars.get(*at).map(|&control| {
            *at += 1;
            char::from(control as u8 & 0x1f)
        }),
        _ => None,
    };
    decoded.map_or_else(|| vec!['\\', first], |decoded| vec![decoded])
}

/// A closed or broken stream moves no class the library chooses, nor any a wrapper chooses before its trap.
///
/// Measured on bash 5.2 before every write went through `tell` and `say`: with standard error closed,
/// `cannot_judge` exited `1` — its `printf` failed, the ERR trap re-entered it, and the second failure left
/// through errexit with printf's own status. So every unjudged stop of both wrappers read, to a caller whose
/// stderr was gone, as a gate that ran and refused. And with SIGPIPE ignored only by the trap installer, a usage
/// error into a broken stderr exited `141`, which is neither class.
///
/// Two halves, each over both streams' failures. The library's three class-choosing paths, sourced as a wrapper
/// sources it: the violation, the unjudged stop, and the ERR trap. And the stops that run before any trap is
/// installed: a wrapper refusing its arguments, a wrapper whose library is absent, and the library run as a
/// command — each the first thing its script writes.
#[test]
fn no_closed_stream_moves_the_library_s_classes() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = root.join(kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    let scratch =
        std::env::temp_dir().join(format!("tianheng-closed-stream-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).expect("create the closed-stream scratch directory");
    let verdict = scratch.join("verdict");
    std::fs::write(&verdict, verdict_channel::rendered(Kind::Violation))
        .expect("write the gate's verdict for the violation path");
    // A tree holding the wrappers and not their library: the bootstrap guard's one reachable state.
    let bare = scratch.join("bare");
    std::fs::create_dir_all(bare.join("scripts")).expect("create the library-less tree");
    for wrapper in WRAPPERS {
        std::fs::copy(root.join(wrapper), bare.join(wrapper))
            .expect("copy a wrapper into the library-less tree");
    }

    let unjudged = i32::from(verdict_channel::wrapper_exit(Kind::CannotJudge));
    let probe = |statement: &str| -> Vec<String> {
        vec![
            "-c".to_string(),
            format!(
                r#"set -Eeuo pipefail; trap '' PIPE; WRAPPER_SUBJECT=probe; source "$1"; install_exit_class_trap; verdict_file=$2; {statement}"#
            ),
            // `$0` is the probe's own name: were it the library's path, the library's misuse guard would read the
            // probe as the library run as a command.
            "probe".to_string(),
            library.display().to_string(),
            verdict.display().to_string(),
        ]
    };
    let mut cases: Vec<(String, Vec<String>, i32)> = vec![
        (
            "exit_for_the_gates_refusal".to_string(),
            probe("exit_for_the_gates_refusal out"),
            i32::from(verdict_channel::wrapper_exit(Kind::Violation)),
        ),
        (
            "cannot_judge".to_string(),
            probe("cannot_judge out"),
            unjudged,
        ),
        ("the ERR trap".to_string(), probe("false"), unjudged),
        (
            "the library run as a command".to_string(),
            vec![library.display().to_string()],
            i32::from(verdict_channel::LIBRARY_MISUSE),
        ),
    ];
    for wrapper in WRAPPERS {
        cases.push((
            format!("{wrapper} without its library"),
            vec![bare.join(wrapper).display().to_string()],
            unjudged,
        ));
    }
    cases.push((
        "scripts/merge-pr.sh with no arguments".to_string(),
        vec![root.join("scripts/merge-pr.sh").display().to_string()],
        unjudged,
    ));
    cases.push((
        "scripts/publish.sh refusing an argument".to_string(),
        vec![
            root.join("scripts/publish.sh").display().to_string(),
            "--allow-dirty".to_string(),
        ],
        unjudged,
    ));

    for (name, arguments, expected) in &cases {
        let open = std::process::Command::new("bash")
            .args(arguments)
            .output()
            .expect("run the stop with its streams open");
        assert_eq!(
            open.status.code(),
            Some(*expected),
            "{name} with its streams open is the baseline, and it does not exit its class: {}",
            String::from_utf8_lossy(&open.stderr)
        );
        for (stream, launcher) in [
            (
                "closed",
                vec!["-c", r#"exec 2>&-; exec bash "$@""#, "launcher"],
            ),
            (
                "broken",
                vec!["-c", support::streams::BROKEN, "launcher", "2"],
            ),
        ] {
            let output = std::process::Command::new("bash")
                .args(&launcher)
                .args(arguments)
                .output()
                .expect("run the stop with its stderr taken away");
            assert_eq!(
                output.status.code(),
                Some(*expected),
                "{name} with stderr {stream} exits {:?}, where it exits {expected} with the stream open",
                output.status
            );
        }
    }
    let _ = std::fs::remove_dir_all(&scratch);
}

/// The lines of a wrapper's bootstrap guard: from `if ! source` to the `fi` that closes it.
fn bootstrap_region(lines: &[(usize, String)]) -> BTreeSet<usize> {
    let mut region = BTreeSet::new();
    let mut inside = false;
    for (number, line) in lines {
        let trimmed = line.trim();
        if trimmed.starts_with("if ! source ") {
            inside = true;
        }
        if inside {
            region.insert(*number);
            if trimmed == "fi" {
                inside = false;
            }
        }
    }
    region
}

/// A wrapper whose library cannot be read is the **unjudged** class, in its own voice.
///
/// The one acquisition the shared machinery cannot guard is the one that loads it, and a `source` failing
/// under `set -e` exits with `source`'s own status — `1`, the class reserved for a gate that ran and
/// refused. Measured before the guard existed: both wrappers exited 1. And measured against bash itself
/// while repairing it: the ERR trap does not fire for a failed `source`, which is why the guard speaks
/// rather than trapping. Held by **running** each wrapper from a fixture tree that has no library — the
/// class and the message are both asserted, because an exit-2 silence is bash's line alone, and that is a
/// diagnosis of the wrong thing in this wrapper's own contract.
#[test]
fn a_wrapper_without_its_library_is_the_unjudged_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    for (wrapper, invocation) in [
        ("scripts/merge-pr.sh", vec!["42", "--body-file", "body.md"]),
        ("scripts/publish.sh", vec!["--dry-run"]),
    ] {
        let scratch = std::env::temp_dir().join(format!(
            "tianheng-missing-library-{}-{}",
            std::process::id(),
            wrapper.replace('/', "-")
        ));
        let _ = std::fs::remove_dir_all(&scratch);
        xingbiao::claim_scratch(&scratch).expect("the scratch root is writable");
        std::fs::create_dir_all(scratch.join("scripts"))
            .expect("create the fixture's scripts directory");
        std::fs::copy(root.join(wrapper), scratch.join(wrapper)).expect("copy the wrapper");
        // The library is the thing this direction removes — nothing else is planted, so the refusal is
        // measured against the wrapper's own text rather than a rewrite of it.
        let output = std::process::Command::new("bash")
            .arg(scratch.join(wrapper))
            .args(&invocation)
            .current_dir(&scratch)
            .output()
            .expect("run the wrapper without its library");
        let _ = std::fs::remove_dir_all(&scratch);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(
            output.status.code(),
            Some(i32::from(verdict_channel::wrapper_exit(Kind::CannotJudge))),
            "{wrapper} without its library is an input it could not read, not a gate that ran and refused: \
             {stderr}"
        );
        assert!(
            stderr.contains("cannot read the shared wrapper library"),
            "{wrapper} must say which of its own two facts failed — the library is missing — rather than \
             leaving bash's line alone to name it: {stderr}"
        );
    }
}

/// The library run as a command stops without reaching either class a wrapper reserves.
///
/// The guard exists so nobody executes the library by mistake; what makes it a guard rather than a sentence
/// is that the file is **run** and its outcome asserted — a count of its `exit` text beside it passes while
/// the guard's condition is false and the file runs to its definitions. Asserted on both halves of *stops
/// without reaching a wrapper's classes*: the exit code, and the message that says what was run instead.
#[test]
fn a_library_run_as_a_command_stops_without_reaching_a_wrapper_s_classes() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY;
    let output = std::process::Command::new("bash")
        .arg(root.join(library))
        .output()
        .expect("run the shared library directly");
    assert!(
        !output.status.success(),
        "{library} run as a command succeeded, so its execution guard is not stopping it"
    );
    // The code `kanhe` owns for this answer, which `the_library_misuse_code_is_outside_every_wrapper_class`
    // holds apart from both wrapper classes — so asserting equality here is what keeps the guard outside them,
    // where asserting only *not 1 and not 2* let the declared code move unobserved.
    assert_eq!(
        output.status.code(),
        Some(i32::from(verdict_channel::LIBRARY_MISUSE)),
        "{library} run as a command must answer the misuse code `kanhe::verdict_channel` owns"
    );
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("not a wrapper itself"),
        "{library} run as a command must say what to run instead, got: {stderr}"
    );
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
///
/// **The exit codes are held the same way.** The library declares one `WRAPPER_EXIT_<NAME>` per code and each
/// is compared with the value `kanhe::verdict_channel` owns for it, so a number is spelled in Rust and read in
/// the shell rather than typed in both.
#[test]
fn each_wrapper_uses_the_channel_the_gates_report_on() {
    let Some(root) = workspace_root() else {
        return;
    };
    // The scalars are declared once, in the library the wrappers source — the extraction this file records
    // — so a wrapper *carrying* one would be the definition site doubled. The comparison below therefore
    // reads the library for the declaration and each wrapper for the one half that cannot move: the channel
    // must still be opened for the gate from the wrapper's own invocation, because a shell cannot inherit a
    // name into an environment-assignment prefix.
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
        assert!(
            !text.contains("WRAPPER_EXIT_VIOLATION=")
                && !text.contains("WRAPPER_EXIT_UNJUDGED=")
                && !text.contains("WRAPPER_EXIT_MISUSE="),
            "{wrapper} declares an exit code itself, and the library owns all three — a second declaration \
             is a number typed in two places"
        );
        assert!(
            !text.contains("GATE_VIOLATION_CLASS=") && !text.contains("GATE_CLEAN_CLASS="),
            "{wrapper} declares a channel scalar itself, and the extraction put both in the shared library \
             — a second declaration is the two-places-that-must-agree shape this file exists to remove"
        );
        // The variable must actually be handed to the gate, not merely declared. Declared and unused would make
        // the file absent for every run, so every violation would report as unjudged.
        assert!(
            text.contains(&format!("{}=$verdict_file", verdict_channel::ENV)),
            "{wrapper} declares the channel and never opens it for the gate, so no verdict could ever arrive"
        );
    }
    let text = read(&root, kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
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
                    "{} declares no `{name}`, so the class the wrappers read off the channel rests on \
                     nothing this check can compare",
                    kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY
                )
            });
        assert_eq!(
            declared,
            expected,
            "{} uses `{declared}` for {name} while `kanhe::verdict_channel` defines `{expected}`",
            kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY
        );
    }
    for (name, expected) in DECLARED_EXITS {
        let variable = format!("WRAPPER_EXIT_{name}");
        let declared = text
            .lines()
            .find_map(|line| line.trim().strip_prefix(&format!("{variable}=")))
            .unwrap_or_else(|| {
                panic!(
                    "{} declares no `{variable}`, so the code its exits name rests on nothing this check can \
                     compare",
                    kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY
                )
            });
        assert_eq!(
            declared,
            expected.to_string(),
            "{} declares `{variable}={declared}` while `kanhe::verdict_channel` owns `{expected}` for it",
            kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY
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
/// `1` is reachable exactly where a gate ran and reported a disagreement. The violation exit is written once,
/// in the library's `exit_for_the_gates_refusal`, directly behind a comparison with the class the gate wrote on
/// its channel — so a wrapper cannot reach it on a fact the gate did not report. What is left for each wrapper
/// is to route its gate's failure there: exactly one call, in the statement that runs the gate by `--exact`.
/// A call anywhere else would read a channel no gate has written, and leave as the unjudged class.
#[test]
fn a_wrapper_exits_the_violation_class_only_for_a_gates_own_verdict() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY;
    let lines: Vec<(usize, String)> = Source::of(read(&root, library))
        .shell()
        .numbered_lines()
        .map(|(number, line)| (number, line.to_string()))
        .collect();
    let sites: Vec<usize> = exit_sites(&executed_text(&lines))
        .into_iter()
        .filter(|(_, argument)| argument == "\"$WRAPPER_EXIT_VIOLATION\"")
        .map(|(number, _)| number)
        .collect();
    assert_eq!(
        sites.len(),
        1,
        "{library} exits the violation class at {sites:?}; exactly one site may, behind the gate's own verdict"
    );
    let site = sites[0];
    // The enclosing function: the nearest definition above the site.
    let enclosing = lines
        .iter()
        .rev()
        .find(|(number, line)| *number < site && line.trim_end().ends_with("() {"))
        .map(|(_, line)| line.trim().to_string());
    assert_eq!(
        enclosing.as_deref(),
        Some("exit_for_the_gates_refusal() {"),
        "{library}:{site} exits the violation class outside the one function that reads the gate's verdict"
    );
    let window: String = lines
        .iter()
        .filter(|(number, _)| *number < site && *number + 6 > site)
        .map(|(_, text)| text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        window.contains("GATE_VIOLATION_CLASS"),
        "{library}:{site} exits the violation class without having read the gate's verdict:\n{window}"
    );
    for wrapper in WRAPPERS {
        let source = Source::of(read(&root, wrapper));
        let positioned = source.shell().positioned_lines();
        let statements = kanhe::gate_identity::logical_lines(&positioned.join("\n"));
        let calls: Vec<usize> = statements
            .iter()
            .enumerate()
            .filter(|(_, (_, statement))| statement.contains("exit_for_the_gates_refusal"))
            .map(|(index, _)| index)
            .collect();
        assert_eq!(
            calls.len(),
            1,
            "{wrapper} routes a failing gate to `exit_for_the_gates_refusal` at {} statements; exactly one \
             may, the one that runs its gate: {:?}",
            calls.len(),
            calls
                .iter()
                .map(|index| statements[*index].0)
                .collect::<Vec<_>>()
        );
        // The call is the body of the `|| {` that closes the gate's own invocation.
        let call = calls[0];
        let opener = call
            .checked_sub(1)
            .map(|index| statements[index].1.trim_end());
        assert!(
            opener.is_some_and(|opener| opener.ends_with("|| {") && opener.contains("-- --exact ")),
            "{wrapper}:{} routes to the gate's refusal from outside the `|| {{` of the statement that runs its \
             gate, so it reads a channel nothing has written",
            statements[call].0
        );
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
    // The library too: its functions run inside each wrapper, so an unguarded acquisition there chooses the
    // class exactly as one written in a wrapper would.
    let mut corpus: Vec<&str> = WRAPPERS.to_vec();
    corpus.push(kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    for wrapper in corpus {
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
            let trimmed = left.trim();
            let variable = trimmed
                .rsplit_once(char::is_whitespace)
                .map_or(trimmed, |(_, variable)| variable);
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

/// Every tracked script is named by [`WRAPPERS`] or is the shared library, and every wrapper loads it.
///
/// Without this the array is a second list beside the tree: a new wrapper would front a gate with its exit
/// classes compared to nothing, while every direction above kept passing over the two it does name.
///
/// **The members come from the enumeration the citation check reads**, every tracked file under `scripts/`,
/// rather than from a rule of this direction's own. It used to find wrappers by their sourcing line, which
/// cannot find the wrapper that matters most — one that never loaded the library — so a script citing and
/// running a gate without it was in the citation check's set and absent from this one. Loading the library is
/// asserted of each member now rather than used to find them, and the array is held to the set both ways.
#[test]
fn every_gate_running_wrapper_is_named() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY;
    let scripts =
        kanhe::gate_identity::tracked_scripts(&root).unwrap_or_else(|why| panic!("{why}"));
    assert!(
        scripts.iter().any(|(path, _)| path == library),
        "{library} is not tracked, so every wrapper sourcing it loads nothing"
    );
    let members: BTreeSet<&str> = scripts
        .iter()
        .map(|(path, _)| path.as_str())
        .filter(|path| *path != library)
        .collect();
    let named: BTreeSet<&str> = WRAPPERS.iter().copied().collect();
    assert_eq!(
        members,
        named,
        "`WRAPPERS` and the tracked scripts disagree — tracked and unnamed, so their exit classes are compared \
         to nothing: {:?}; named and untracked: {:?}",
        members.difference(&named).collect::<Vec<_>>(),
        named.difference(&members).collect::<Vec<_>>()
    );
    let sourcing = format!("/{library}");
    for (path, text) in scripts.iter().filter(|(path, _)| path != library) {
        let loads = Source::of(text.clone()).shell().lines().any(|line| {
            let line = line.trim_start();
            (line.starts_with("source ") || line.starts_with("if ! source "))
                && line.contains(&sourcing)
        });
        assert!(
            loads,
            "{path} is a tracked script and does not load {library}, so the exit codes, the trap and the \
             verdict read every direction here holds it to are not the ones it runs under"
        );
    }
}

/// Both wrappers refuse a **flag-shaped value**, and every arm that takes one is held to it.
///
/// **This is the second instance of one class, so it is held rather than repaired twice.**
/// `scripts/publish.sh` grew the shape check with a paragraph arguing for it, and `scripts/merge-pr.sh` kept
/// a guard that checked only that *something* followed. The consequences differ — cargo does not consume a
/// flag-shaped value, `gh`'s caller here does, so one leaked a refused flag past the wrapper and the other
/// swallowed an admitted one as text — and both are the wrong diagnosis at the moment before an irreversible
/// act. Two implementations of one rule agree by maintenance; a direction over both agrees by running.
///
/// **The guard's NAME is derived from how it is called, not written here.** The two wrappers spell it
/// differently (`require_value`, `require_a_value`), and a literal pair would be a third thing to keep in
/// step — the shape this file exists to remove.
///
/// **Three properties, because two could not see the arity.** A first cut asserted only that *some* guard
/// call carried a value and that the guard judged the shape. Measured: with `--subject`'s call shortened to
/// two arguments, that passed — the other arm still carried a value, so the guard was still found and still
/// checked. The arm-level reading closes it: an arm that consumes the following argument must hand its guard
/// that argument, which `kanhe::wrapper_parser::Arm` separates from merely naming the guard.
#[test]
fn each_wrapper_refuses_a_flag_shaped_value_in_every_value_position() {
    let Some(root) = workspace_root() else {
        return;
    };
    for wrapper in WRAPPERS {
        let text = read(&root, wrapper);
        let guard = wrapper_parser::value_guard(&text, wrapper).unwrap_or_else(|refusal| {
            panic!(
                "{wrapper}: {} — no arm hands a value to a guard as `<name>{}`, or several differently-named \
                 guards do, so no arm's value can have its shape judged",
                refusal.message,
                wrapper_parser::VALUE_GUARD_CALL
            )
        });

        // Every arm that takes a value must hand it over, and every arm handed one must take it. Both
        // directions, because one alone is satisfied by two sets agreeing on the wrong answer.
        let arms = wrapper_parser::parser_arms(&text, &guard);
        let takes: BTreeSet<&str> = arms
            .iter()
            .filter(|(_, arm)| arm.consumes)
            .map(|(flag, _)| flag.as_str())
            .collect();
        let judged: BTreeSet<&str> = arms
            .iter()
            .filter(|(_, arm)| arm.guards_with_value)
            .map(|(flag, _)| flag.as_str())
            .collect();
        assert_eq!(
            takes, judged,
            "{wrapper}: an arm reads the following argument without handing it to `{guard}` to judge, or is \
             handed one it never reads — takes {takes:?}, judged {judged:?}. The first is a refused flag \
             reaching the tool as a value, or an admitted one swallowed as text; the second is a refusal \
             that never runs"
        );
        assert!(
            !takes.is_empty(),
            "{wrapper}: no arm takes a value, so this direction compared two empty sets and reported \
             agreement over nothing"
        );

        // The guard's own body: from its definition to the closing brace at column zero.
        let source = Source::of(text.as_str());
        let executed: Vec<String> = source
            .shell()
            .lines()
            .map(|line| line.trim().to_string())
            .collect();
        let opens = format!("{guard}() {{");
        let start = executed
            .iter()
            .position(|line| *line == opens)
            .unwrap_or_else(|| {
                panic!("{wrapper} calls `{guard}` and this direction found no `{opens}` to read")
            });
        let body: Vec<&String> = executed[start + 1..]
            .iter()
            .take_while(|line| *line != "}")
            .collect();
        assert!(
            body.iter().any(|line| line.contains("== -*")),
            "{wrapper}'s `{guard}` is handed a value and never judges its shape: a refused argument does \
             not become admitted by sitting in a value position, and an admitted one does not reach the \
             tool by being read as text. Its sibling wrapper refuses `-`-leading values; this one must too"
        );
    }
}
