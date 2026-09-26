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
         wrapper direction beside it reads; bash, to ask which function holds the violation exit",
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
        "bash, to run the publish wrapper; this test binary itself, to re-run the signal direction under a \
         parent that ignores a signal",
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
        // qualifier to match. `bash::` is the same case as `hermetic_git::`: the one `bash` the checks run is
        // built in `support::bash`, and a target reaches it as `support::bash::…` or, having imported the
        // module, as `bash::…` — both carry the module's name before `::`, which is what is matched. A rename
        // of the module on import, `use support::bash as sh`, is not read here: no target writes one, and the
        // builder is what constructs a `bash`, held by `hermetic_invocations`' reader, which binds renames.
        if executed.lines().any(|line| {
            opens(line, "Command::new(")
                || opens(line, "hermetic_git::")
                || opens(line, "hermetic(")
                || opens(line, "bash::")
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
        let source = Source::of(text.clone());
        let lines: Vec<(usize, String)> = source
            .shell()
            .numbered_lines()
            .map(|(number, line)| (number, line.to_string()))
            .collect();
        let bootstrap = bootstrap_region(&lines);
        let mut chosen: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut refused = Vec::new();
        let sites = exit_sites(&text).unwrap_or_else(|why| panic!("{script}: {why}"));
        for (number, argument) in sites {
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
    ("exit 2 extra", &["2 extra"]),
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
    ("for exit in a; do :; done", &["in a"]),
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
    // A here-string is a word, read like any other; a `${…}` ends where bash ends it, past a quoted brace.
    ("grep -q x <<< \"$out\"; exit 3", &["3"]),
    ("printf '%s' \"${x:-'}'}\"; exit 3", &["3"]),
    ("printf '%s' ${x:-'}'}; exit 3", &["3"]),
    ("printf '%s' ${x:-$(exit 4)}", &["4"]),
    // A backslash-newline is removed before a word begins, so a `#` after one opens a comment, as bash reads it.
    ("true \\\n#exit 3", &[]),
    // Arithmetic runs no command: its `<<` is a shift, and an `exit` after it is read.
    ("x=$((1 << 2)); exit 3", &["3"]),
    ("(( x <<= 1 )); exit 3", &["3"]),
    // Arithmetic nested in arithmetic is arithmetic, and `case` as an argument is a word.
    ("x=$(( $((1)) + 1 )); exit 3", &["3"]),
    ("x=$(printf '%s' case); exit 3", &["3"]),
    // A command name computed when the line runs is not the line's to read: the declared bound below.
    ("$stop 3", &[]),
];

#[test]
fn the_exit_reader_decides_every_shape_a_wrapper_line_takes() {
    for (line, expected) in EXIT_SHAPES {
        assert_eq!(
            exit_arguments(line),
            Ok(expected
                .iter()
                .map(|argument| argument.to_string())
                .collect::<Vec<_>>()),
            "the exit reader misreads `{line}`"
        );
    }
    // What the lexer cannot place is refused rather than read past: the words after it stand where it did not
    // decide, so an `exit` among them would pass.
    for line in [
        "printf '%s' 'open\nexit 3",
        "cat <<EOF\nexit 3\nEOF",
        "cat <<-EOF\n\texit 3\nEOF",
        "$\"exit\" 7",
        // Measured on bash 5.3: `v="$(case a in a) exit 4;; esac)"` runs the exit, and a `case` pattern's `)`
        // stands where the substitution's close would.
        "v=\"$(case a in a) exit 4;; esac)\"",
        "x=${y:-$\"exit\"}",
        "x=$(( $(exit 3) ))",
        // Measured on bash 5.3: `((exit 6) ) # ))` exits 6 — a `((` whose first close is single is nested
        // subshells — and `"$\<newline>(exit 5)"` runs the substitution the continuation joins.
        "((exit 6) ) # ))",
        "(( a[\"))\"] = 1 )); exit 3 # \"",
        "v=\"$\\\n(exit 5)\"",
    ] {
        assert!(
            exit_arguments(line).is_err(),
            "`{line}` holds what the lexer cannot place, and must be refused rather than read: {:?}",
            exit_arguments(line)
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
            Ok(Vec::<String>::new()),
            "`{line}` names its command at run time, and the declared bound says it is not read"
        );
    }
}

/// The argument of every `exit` word in `text`, as written — empty for a bare `exit` — or why it was not read.
fn exit_arguments(text: &str) -> Result<Vec<String>, String> {
    Ok(exit_sites(text)?
        .into_iter()
        .map(|(_, argument)| argument)
        .collect())
}

/// Every `exit` word in `text`, with the line it stands on and its argument as written.
///
/// **Over the whole text, not a line at a time**, because a quote the shell opens on one line closes on a later
/// one: `merge-pr.sh` writes refusal messages across lines, and a line reader saw a message's *and exit 0* on
/// a continuation line as standing outside every quote. The argument is the next word, unless an operator — a
/// separator, a redirection, a newline — ends the command first.
///
/// **The file as written, lexed once.** The lexer reads a comment where bash opens one, so no line-wise cut runs
/// first to disagree with its quoting. And **what it cannot place is a refusal, not a skip**: a quote the text
/// ends inside, a here-document, a locale-translated string each leave the words after them in positions the
/// lexer did not decide, so an `exit` there would pass unread.
fn exit_sites(text: &str) -> Result<Vec<(usize, String)>, String> {
    let words = kanhe::shell::lex_placed(text).map_err(|(line, what)| {
        format!(
            "line {line} holds {what}, so where the words after it stand is not known and an `exit` among them \
             would pass unread"
        )
    })?;
    let mut sites = Vec::new();
    for (index, word) in words.iter().enumerate() {
        if word.operator || word.value != "exit" {
            continue;
        }
        // Every argument up to the operator that ends the command, so `exit 2 extra` — which bash answers with
        // *too many arguments* and status `1` — is not read as the declared shape it begins with.
        let argument: Vec<&str> = words[index + 1..]
            .iter()
            .take_while(|next| !next.operator)
            .filter(|next| next.depth == word.depth)
            .map(|next| next.written.as_str())
            .collect();
        sites.push((word.line, argument.join(" ")));
    }
    Ok(sites)
}

/// Whether any of `words` spells `name` other than as the parameter `$NAME` — anywhere in its text, quoted or not,
/// glued to a flag or inside arithmetic, as a whole identifier.
///
/// **The question is where the name is written, not which form writes it.** An assignment word, `read -r "X"`,
/// `printf -vX`, `unset 'X'`, `(( X = 3 ))` and `declare -n r=X` each spell the name in text, while reading it is
/// the one place it is a parameter. So every occurrence outside a parameter is refused, and a wrapper has no form
/// of writing a library scalar left to find. The name is matched up to its end and not from its start, since
/// `printf -vNAME` glues it to a flag; a longer name ending in it is refused too, which renaming answers.
fn names_a_scalar(words: &[kanhe::shell::Word], name: &str) -> bool {
    let spells = |text: &str| {
        text.match_indices(name).any(|(at, _)| {
            let identifier = |c: char| c.is_ascii_alphanumeric() || c == '_';
            !text[at + name.len()..]
                .chars()
                .next()
                .is_some_and(identifier)
        })
    };
    words.iter().any(|word| {
        !word.operator
            && word.parts.iter().any(|part| match part {
                kanhe::shell::Part::Literal { text, .. }
                | kanhe::shell::Part::Unquoted(text)
                | kanhe::shell::Part::Arithmetic(text)
                | kanhe::shell::Part::AnsiC(text)
                | kanhe::shell::Part::Compound(text) => spells(text),
                _ => false,
            })
    })
}

/// A wrapper names a library scalar in any form that writes it, and reading one is not naming it.
#[test]
fn a_wrapper_naming_a_library_scalar_is_refused() {
    for (text, named) in [
        ("X=1", true),
        ("X+=0", true),
        ("read -r X <<< 1", true),
        ("printf -v X 1", true),
        ("readonly X", true),
        ("unset X", true),
        ("(( X = 3 ))", true),
        ("printf -vX 3", true),
        ("read -r \"X\"", true),
        ("unset 'X'", true),
        ("declare -n r=X", true),
        ("exit \"$X\"", false),
        ("printf '%s' \"${X}\"", false),
        ("XY=1", false),
        ("AX=1", true),
        ("tell \"the XY class\"", false),
    ] {
        assert_eq!(
            names_a_scalar(&kanhe::shell::lex(text), "X"),
            named,
            "whether `{text}` names X"
        );
    }
}

/// The value of every assignment word among `words` that assigns `name` — `NAME=value` or `NAME+=value`, by the
/// one reading of an assignment word the workflow's command reader uses too.
///
/// Where a word stands is not asked, so an argument written `NAME=value` is counted as well. Counting one too
/// many only refuses; where such an argument would stand in for the declaration it replaced, [`declared_value`]
/// asks bash for the value, and bash holds none.
fn plain_assignments(words: &[kanhe::shell::Word], name: &str) -> Vec<String> {
    words
        .iter()
        .filter(|word| !word.operator && word.assigns() == Some(name))
        .filter_map(|word| {
            word.value
                .split_once('=')
                .map(|(_, value)| value.to_string())
        })
        .collect()
}

/// The one value `library` declares for `name`: exactly one assignment word for it in the file, and the same
/// value in bash once the library is sourced. The file is lexed as written, so its comments are the lexer's to
/// drop, and a construct the lexer cannot place is a refusal rather than a skip.
///
/// **The two halves answer different questions, and each covers the other's gap.** The words say how many
/// places assign the name, which bash cannot say after the fact; bash says whether the word it read is an
/// assignment where it stands, which the words decide only by modelling where a command begins.
fn declared_value(root: &Path, library: &str, name: &str) -> Result<String, String> {
    let words = kanhe::shell::lex_placed(&read(root, library)).map_err(|(line, what)| {
        format!("{library}:{line} holds {what}, so its assignments cannot all be read")
    })?;
    let declared = kanhe::selection::the_only(
        &format!("assignment of `{name}` in {library}"),
        plain_assignments(&words, name),
    )
    .map_err(|refusal| refusal.message)?;
    match value_as_bash_holds_it(root, library, name) {
        Some(held) if held == declared => Ok(declared),
        Some(held) => Err(format!(
            "{library} spells `{name}={declared}`, and bash holds `{held}` for it once the library is sourced"
        )),
        None => Err(format!(
            "{library} spells `{name}={declared}`, and bash holds no value for it once the library is sourced — \
             the word is not an assignment where it stands"
        )),
    }
}

/// An assignment that is not an assignment word is not read — a stated bound, shown rather than described.
///
/// An assignment word, `NAME=value` or `NAME+=value`, is read wherever it stands, `local`, `declare`, `export` and
/// `readonly` ones included. bash also assigns through builtins and expansions — `read`, `printf -v`, `let`,
/// `(( ))`, `${NAME:=…}` and more — and a reader listing those is the enumeration `AGENTS.md`'s *A repair loop is
/// a diagnosis* says to stop. What holds a changed value is bash: the library declares each name `readonly`, so an
/// assignment to it in any form ends the wrapper, and the library's EXIT trap reports that as the unjudged class —
/// `a_declared_name_assigned_again_is_the_unjudged_class` runs each form.
#[test]
fn an_assignment_that_is_not_an_assignment_word_is_not_read() {
    let rows: [(&str, &[&str]); 8] = [
        ("X=1", &["1"]),
        ("local X=1", &["1"]),
        ("declare -r X=1", &["1"]),
        ("X+=0", &["0"]),
        // The bound: each assigns X, and none is an assignment word.
        ("read X <<< 3", &[]),
        ("printf -v X 3", &[]),
        ("(( X = 3 ))", &[]),
        (": \"${X:=3}\"", &[]),
    ];
    for (text, expected) in rows {
        assert_eq!(
            plain_assignments(&kanhe::shell::lex(text), "X"),
            expected
                .iter()
                .map(|value| value.to_string())
                .collect::<Vec<_>>(),
            "the plain assignments of X in `{text}`"
        );
    }
}

/// bash run on `library` sourced and then `probe`, with the library as `$1` and `name` as `$2` — as positional
/// parameters, never spliced into the `-c` text, so a path is one word whatever it holds.
///
/// The library is handed over joined to `root`, so it always carries a slash: `source` searches `PATH` for a
/// name without one before the working directory, and would read a host file of that name in the fixture's
/// place.
///
/// **The probe's standard output is the only one read.** What the library prints while it is sourced goes to
/// `/dev/null`, so a library printing text shaped like the probe's answer cannot stand in for it.
///
/// Run through [`support::bash::bash`], so no ambient startup file runs before the library does. Sourcing runs
/// the library's statements outside every function, so a run that did not complete means one of them ended it.
fn after_sourcing(root: &Path, library: &str, probe: &str, name: &str) -> String {
    let output = support::bash::bash()
        .current_dir(root)
        .args([
            "-c",
            &format!(r#"source "$1" >/dev/null && {probe}"#),
            "probe",
        ])
        .arg(root.join(library))
        .arg(name)
        .output()
        .unwrap_or_else(|error| {
            panic!("bash could not be run to read {library}'s `{name}`: {error}")
        });
    assert!(
        output.status.success(),
        "sourcing {library} did not complete ({}), so what it holds for `{name}` is not known: {}. Where a \
         statement outside every function ended it, an `exit` there stands outside every function by construction",
        output.status,
        String::from_utf8_lossy(&output.stderr).trim()
    );
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// The body bash holds for function `name` once `library` is sourced, as `declare -f` prints it.
fn body_as_bash_holds_it(root: &Path, library: &str, name: &str) -> String {
    let body = after_sourcing(root, library, r#"declare -f "$2""#, name);
    assert!(
        !body.trim().is_empty(),
        "bash holds no function `{name}` after sourcing {library}, so where its exits stand is not known"
    );
    body
}

/// The value bash holds for variable `name` once `library` is sourced, or `None` where it holds none.
fn value_as_bash_holds_it(root: &Path, library: &str, name: &str) -> Option<String> {
    after_sourcing(
        root,
        library,
        r#"if [[ -v $2 ]]; then printf 'set:%s' "${!2}"; fi"#,
        name,
    )
    .strip_prefix("set:")
    .map(str::to_string)
}

/// A word spelled as a declaration is one only where bash reads it as one.
///
/// The words alone count an argument written `NAME=value` as an assignment, so with the declaration removed and
/// `printf '%s\n' NAME=1` in its place there is still exactly one. bash, sourcing the library, holds no value for
/// the name, and that is what refuses it — including where the library itself prints what the probe would.
#[test]
fn a_word_spelled_as_a_declaration_is_one_only_where_bash_reads_it() {
    let scratch = support::fixture::Scratch::claim("tianheng-declared-value");
    for (text, expected) in [
        ("X=1\n", Ok("1".to_string())),
        (
            "printf '%s\\n' X=1\n",
            Err("lib.sh spells `X=1`, and bash holds no value for it once the library is sourced — the word is not \
                 an assignment where it stands"
                .to_string()),
        ),
        (
            "printf 'set:1'\n: X=1\n",
            Err("lib.sh spells `X=1`, and bash holds no value for it once the library is sourced — the word is not \
                 an assignment where it stands"
                .to_string()),
        ),
        (
            "X=1\nX=2\n",
            Err("expected exactly one assignment of `X` in lib.sh".to_string()),
        ),
    ] {
        std::fs::write(scratch.path().join("lib.sh"), text).expect("write the library");
        let found = declared_value(scratch.path(), "lib.sh", "X");
        match (&found, &expected) {
            (Ok(value), Ok(wanted)) => assert_eq!(value, wanted, "`{text}`"),
            (Err(why), Err(wanted)) => assert!(why.starts_with(wanted.as_str()), "`{text}`: {why}"),
            _ => panic!("`{text}`: {found:?}, expected {expected:?}"),
        }
    }
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
    let scratch_root = support::fixture::Scratch::claim("tianheng-closed-stream");
    let scratch = scratch_root.path();
    // Every stop here comes before the act, so `gh` and `cargo` are never this run's to reach. Each is a stub that
    // leaves a mark, found first on `PATH`: a stop that did reach one runs no host tool, and says so below.
    let bin = scratch.join("bin");
    std::fs::create_dir(&bin).expect("create the closed-stream stub PATH");
    let reached = scratch.join("reached");
    // The mark's path reaches the stub through its environment, as the wrapper harnesses hand theirs, so a path
    // holding a quote cannot make the stub a syntax error that exits without leaving one.
    for tool in ["gh", "cargo"] {
        support::fixture::write_executable(
            &bin.join(tool),
            &format!("#!/usr/bin/env bash\nprintf '%s\\n' {tool} >> \"$FAKE_REACHED\"\nexit 99\n"),
        );
    }
    let path = support::fixture::path_with(&bin);
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
        // An expansion error ends bash without the ERR trap, with status 1; the EXIT trap holds it.
        (
            "an expansion error".to_string(),
            probe(r#"printf '%s' "${wrapper_typo}""#),
            unjudged,
        ),
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
        let open = support::bash::bash()
            .args(arguments)
            .env("PATH", &path)
            .env("FAKE_REACHED", &reached)
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
                vec!["-c", support::streams::CLOSED, "launcher", "2"],
            ),
            (
                "broken",
                vec!["-c", support::streams::BROKEN, "launcher", "2"],
            ),
        ] {
            let output = support::bash::bash()
                .args(&launcher)
                .args(arguments)
                .env("PATH", &path)
                .env("FAKE_REACHED", &reached)
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
    let reached_tools = support::fixture::read_if_present(&reached).expect("read the stub marks");
    assert!(
        reached_tools.is_empty(),
        "a stop meant to come before the act reached a tool it runs:\n{reached_tools}"
    );
}

/// How a wrapper's bootstrap guard opens: the one `source` of the library it guards.
const BOOTSTRAP_SOURCE: &str = "if ! source ";

/// The lines of a wrapper's bootstrap guard: from `if ! source` to the `fi` that closes it.
fn bootstrap_region(lines: &[(usize, String)]) -> BTreeSet<usize> {
    let mut region = BTreeSet::new();
    let mut inside = false;
    for (number, line) in lines {
        let trimmed = line.trim();
        if trimmed.starts_with(BOOTSTRAP_SOURCE) {
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
        let claimed = support::fixture::Scratch::claim("tianheng-missing-library");
        let scratch = claimed.path();
        std::fs::create_dir_all(scratch.join("scripts"))
            .expect("create the fixture's scripts directory");
        std::fs::copy(root.join(wrapper), scratch.join(wrapper)).expect("copy the wrapper");
        // The library is the thing this direction removes — nothing else is planted, so the refusal is
        // measured against the wrapper's own text rather than a rewrite of it.
        let output = support::bash::bash()
            .arg(scratch.join(wrapper))
            .args(&invocation)
            .current_dir(scratch)
            .output()
            .expect("run the wrapper without its library");
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
    let output = support::bash::bash()
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
        // A wrapper reads the library's scalars and never names them: every form that assigns, declares or
        // unsets a name spells it as a word of its own, while reading it is `$NAME`, a parameter. So the question
        // is whether a wrapper writes the name at all rather than which of bash's assignment forms it used.
        let words = kanhe::shell::lex_placed(&text).unwrap_or_else(|(line, what)| {
            panic!("{wrapper}:{line} holds {what}, so its words cannot all be read")
        });
        let named_here: Vec<String> = DECLARED_EXITS
            .iter()
            .map(|(name, _)| format!("WRAPPER_EXIT_{name}"))
            .chain(["GATE_VIOLATION_CLASS", "GATE_CLEAN_CLASS"].map(String::from))
            .filter(|name| names_a_scalar(&words, name))
            .collect();
        assert!(
            named_here.is_empty(),
            "{wrapper} names {named_here:?} as a word of its own, and the library owns every exit code and \
             channel class — a wrapper assigning, declaring or unsetting one moves a value typed in one place"
        );
        // The variable must actually be handed to the gate, not merely declared. Declared and unused would make
        // the file absent for every run, so every violation would report as unjudged.
        assert!(
            text.contains(&format!("{}=$verdict_file", verdict_channel::ENV)),
            "{wrapper} declares the channel and never opens it for the gate, so no verdict could ever arrive"
        );
    }
    // Every assignment word the library's executed text spells for `name` — `NAME+=value` and `local`, `declare`,
    // `export` and `readonly` ones included, since each is a word of its own — exactly one of them, and the value
    // bash holds once the library is sourced. An assignment that is not an assignment word is the stated bound
    // `plain_assignments` shows.
    let declaration = |name: &str| {
        declared_value(&root, kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY, name)
            .unwrap_or_else(|why| panic!("{why}"))
    };
    for (name, expected) in [
        (
            "GATE_VIOLATION_CLASS",
            verdict_channel::rendered(Kind::Violation),
        ),
        ("GATE_CLEAN_CLASS", verdict_channel::CLEAN.to_string()),
    ] {
        let declared = declaration(name);
        assert_eq!(
            declared,
            expected,
            "{} uses `{declared}` for {name} while `kanhe::verdict_channel` defines `{expected}`",
            kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY
        );
    }
    for (name, expected) in DECLARED_EXITS {
        let variable = format!("WRAPPER_EXIT_{name}");
        let declared = declaration(&variable);
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
    let text = read(&root, library);
    let source = Source::of(text.clone());
    let lines: Vec<(usize, String)> = source
        .shell()
        .numbered_lines()
        .map(|(number, line)| (number, line.to_string()))
        .collect();
    const VIOLATION: &str = "\"$WRAPPER_EXIT_VIOLATION\"";
    let sites: Vec<usize> = exit_sites(&text)
        .unwrap_or_else(|why| panic!("{library}: {why}"))
        .into_iter()
        .filter(|(_, argument)| argument == VIOLATION)
        .map(|(number, _)| number)
        .collect();
    assert_eq!(
        sites.len(),
        1,
        "{library} exits the violation class at {sites:?}; exactly one site may, behind the gate's own verdict"
    );
    let site = sites[0];
    // **Which function holds it is bash's answer, not a reading of braces.** A brace is syntax only where the
    // shell parses it as one — unquoted, in a command's position — which is the grammar this file stopped
    // modelling for `exit`. So the body is asked of bash: the library is sourced, which runs only its
    // declarations and a guard that does not fire when sourced, and `declare -f` prints the function as bash
    // parsed it. The one site in the file being the one site in that body is the site being inside it.
    let body = body_as_bash_holds_it(&root, library, "exit_for_the_gates_refusal");
    let inside = exit_sites(&body)
        .unwrap_or_else(|why| panic!("the body bash holds for `exit_for_the_gates_refusal`: {why}"))
        .into_iter()
        .filter(|(_, argument)| argument == VIOLATION)
        .count();
    assert_eq!(
        inside, 1,
        "{library}:{site} exits the violation class outside the one function that reads the gate's verdict — \
         the body bash holds for `exit_for_the_gates_refusal` carries {inside} such exits:\n{body}"
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
        let text = read(&root, wrapper);
        let statements = kanhe::shell::statements(&text).unwrap_or_else(|(line, what)| {
            panic!("{wrapper}:{line} holds {what}, so its statements cannot all be read")
        });
        gate_refusal_routing(&statements).unwrap_or_else(|why| panic!("{wrapper}: {why}"));
    }
}

/// The one call routing a wrapper's failing gate to the library's refusal helper: a command at a statement's
/// head, exactly once, the statement before it being the gate's invocation closed by `|| {`.
///
/// Read off the lexer's statements, so a statement ends where bash ends the command. A line ending in an
/// escaped backslash ends there, and a join past it would fabricate the very `|| {` this check exists to
/// require — the call then reads a channel the statement above it never guarded.
fn gate_refusal_routing(statements: &[kanhe::shell::Statement]) -> Result<(), String> {
    const CALL: &str = "exit_for_the_gates_refusal";
    let calls: Vec<usize> = statements
        .iter()
        .enumerate()
        .filter(|(_, statement)| {
            let commands: BTreeSet<usize> = kanhe::shell::command_positions(&statement.words)
                .into_iter()
                .collect();
            statement
                .words
                .iter()
                .enumerate()
                .any(|(at, word)| commands.contains(&at) && word.value == CALL)
        })
        .map(|(index, _)| index)
        .collect();
    let &[call] = calls.as_slice() else {
        return Err(format!(
            "routes a failing gate to `{CALL}` at {} statements; exactly one may, the one that runs its \
             gate: {:?}",
            calls.len(),
            calls
                .iter()
                .map(|index| statements[*index].line)
                .collect::<Vec<_>>()
        ));
    };
    let Some(opener) = call.checked_sub(1).map(|index| &statements[index]) else {
        return Err(format!(
            "calls `{CALL}` at line {} with no statement above it, so no gate's `|| {{` can hold the call",
            statements[call].line
        ));
    };
    // The call is the body of the `|| {` that closes the gate's own invocation: the statement above ends in
    // the separator and the brace, and carries the `-- --exact` the gate is asked for by. The separator is
    // two adjacent `|` words, as `is_or_separator` states.
    let mut tail = opener.words.iter().rev();
    let closes_the_gate = match (tail.next(), tail.next(), tail.next()) {
        (Some(brace), Some(second), Some(first)) => {
            !brace.operator
                && brace.value == "{"
                && brace.written == "{"
                && second.operator
                && second.value == "|"
                && first.operator
                && first.value == "|"
                && first.start + 1 == second.start
        }
        _ => false,
    };
    let asks_for_the_gate = opener.words.windows(2).any(|pair| {
        !pair[0].operator
            && pair[0].value == "--"
            && !pair[1].operator
            && pair[1].value == "--exact"
    });
    if !(closes_the_gate && asks_for_the_gate) {
        return Err(format!(
            "line {} routes to the gate's refusal from outside the `|| {{` of the statement that runs its \
             gate, so it reads a channel nothing has written",
            statements[call].line
        ));
    }
    Ok(())
}

/// The join this reader no longer does. The gate's line ends in an escaped backslash, which bash reads as a
/// literal backslash ending the command — measured, the `||` opening the next line is a syntax error there —
/// so the block below guards nothing, and the routing must be refused rather than read as the gate's own.
/// Under the retired join the two lines were one statement ending in `|| {`, and this passed.
#[test]
fn a_gate_statement_ended_by_an_escaped_backslash_routes_no_refusal_call() {
    let fixture = concat!(
        "gate_output=$(cargo test -q -p kanhe --test merge_message -- --exact the_gate 2>&1) \\\\\n",
        "|| {\n",
        "    exit_for_the_gates_refusal \"$gate_output\"\n",
        "}\n",
    );
    let statements = kanhe::shell::statements(fixture).expect("the fixture is placed");
    let why = gate_refusal_routing(&statements).expect_err(
        "bash ends the gate's statement at the escaped backslash, so the `|| {` below it opens \
                     another command — the call is nobody's guard",
    );
    assert!(
        why.contains("outside the `|| {"),
        "the refusal names the misrouting, not just that one was found: {why}"
    );
}

/// The functions that stop a wrapper: `cannot_judge`, and every function whose body calls one, read from the
/// scripts rather than listed, so a wrapper's own refusal helper is a stop by what it does.
///
/// Each script is lexed whole, once, and only a word standing where a command's name does —
/// [`kanhe::shell::command_positions`] — is a call: a `cannot_judge` printed as an argument, quoted or not, calls
/// nothing, and one inside a command substitution ends that subshell rather than the wrapper. A body is the words
/// between a `NAME ( ) {` and the `}` that closes it at depth, a brace counted only where bash reads it as the
/// reserved word it is, in command position. A script the lexer cannot place is refused rather than read past.
fn stops_of(scripts: &[&str]) -> BTreeSet<String> {
    let mut bodies: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for script in scripts {
        let words = kanhe::shell::lex_placed(script).unwrap_or_else(|(line, what)| {
            panic!("a wrapper script holds {what} at line {line}, so which functions stop cannot be read")
        });
        let commands: BTreeSet<usize> = kanhe::shell::command_positions(&words)
            .into_iter()
            .collect();
        let brace = |at: usize, text: &str| commands.contains(&at) && words[at].value == text;
        let mut index = 0;
        while index < words.len() {
            let opens = commands.contains(&index)
                && words
                    .get(index + 1)
                    .is_some_and(|w| w.operator && w.value == "(")
                && words
                    .get(index + 2)
                    .is_some_and(|w| w.operator && w.value == ")");
            let mut at = index + 3;
            while opens && words.get(at).is_some_and(|w| w.operator && w.value == "\n") {
                at += 1;
            }
            if !opens || at >= words.len() || !brace(at, "{") {
                index += 1;
                continue;
            }
            let body = bodies.entry(words[index].value.clone()).or_default();
            let mut depth = 1usize;
            at += 1;
            while at < words.len() {
                if brace(at, "{") {
                    depth += 1;
                } else if brace(at, "}") {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                } else if commands.contains(&at) && words[at].depth == words[index].depth {
                    // A call inside a command substitution runs in a subshell, and its `exit` ends only that.
                    body.push(words[at].value.clone());
                }
                at += 1;
            }
            index = at + 1;
        }
    }
    let mut stops = BTreeSet::from(["cannot_judge".to_string()]);
    loop {
        let before = stops.len();
        for (name, words) in &bodies {
            if words.iter().any(|word| stops.contains(word)) {
                stops.insert(name.clone());
            }
        }
        if stops.len() == before {
            return stops;
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
/// **The corpus is every assignment from a command substitution, `NAME=$(…)`, and names no tool.** What a
/// substitution invokes is no part of the property: `repo=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)`
/// invokes no tool, and measured, a failed `cd` there under `set -e` exits **1** — the class that means a gate
/// ran and refused, reported by a wrapper whose gate had not been found. A substitution standing elsewhere — an
/// argument, a condition — is not in this corpus; its failure is the ERR trap's.
///
/// A failed acquisition is **refused** or **given a value**, never ignored: `|| verdict=""` supplies a
/// fallback and is handled exactly as `|| cannot_judge` is, and a `|| {` block guards only when its first
/// command stops or supplies one — `|| { true; }` swallows the failure and is not admitted, exactly as
/// `|| true` is not. Which functions stop is read from the scripts by [`stops_of`] rather than listed here,
/// and the command an acquisition stands in is read from the lexer's words, so it ends where bash ends it.
#[test]
fn every_acquisition_is_guarded_so_the_tool_cannot_choose_the_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    // The library too: its functions run inside each wrapper, so an unguarded acquisition there chooses the
    // class exactly as one written in a wrapper would.
    let mut corpus: Vec<&str> = WRAPPERS.to_vec();
    corpus.push(kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    let library = read(&root, kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    for wrapper in corpus {
        let text = read(&root, wrapper);
        let stops = stops_of(&[library.as_str(), text.as_str()]);
        let words = kanhe::shell::lex_placed(&text).unwrap_or_else(|(line, what)| {
            panic!("{wrapper}:{line} holds {what}, so its acquisitions cannot all be read")
        });
        let found = unguarded_acquisitions(&words, &stops);
        // **Per wrapper, before the verdict.** `unguarded.is_empty()` is satisfied by a corpus that collapsed
        // to nothing exactly as it is by one that is clean, and the two are opposite facts. Every sibling
        // direction here already guards its own corpus this way; this one asserted only the finding.
        assert!(
            found.examined > 0,
            "{wrapper}: no acquisition entered the corpus, so this direction would report clean over nothing \
             — a wrapper standing in front of an irreversible act must not be judged by an empty reading"
        );
        assert!(
            found.unguarded.is_empty(),
            "{wrapper}: these acquisitions are unguarded, so a failing tool exits with its own status and its \
             own stderr instead of one of this wrapper's two classes: {:?}",
            found.unguarded
        );
    }
}

/// What [`unguarded_acquisitions`] found in one script: how many acquisitions were examined, and the
/// one-based lines of those no guard reaches.
struct Acquisitions {
    examined: usize,
    unguarded: Vec<usize>,
}

/// Every acquisition among `words` — an assignment word carrying a command substitution — checked for its
/// guard.
///
/// Read off the lexer's words rather than off joined text, so a command ends where bash ends it: a
/// backslash-newline joins two lines and nothing else does — not an escaped backslash ending a line, not a
/// backslash inside single quotes. A join over text pulled a guard token up from a line bash keeps apart,
/// and reported the acquisition guarded on the strength of another command's words. The corpus is the
/// assignment word itself, so `x="$(tool)"` counts exactly as `x=$(tool)` does, while a substitution
/// standing in an argument is not an acquisition — its failure is the ERR trap's.
fn unguarded_acquisitions(words: &[kanhe::shell::Word], stops: &BTreeSet<String>) -> Acquisitions {
    let mut found = Acquisitions {
        examined: 0,
        unguarded: Vec::new(),
    };
    for (at, word) in words.iter().enumerate() {
        let Some(variable) = word.assigns() else {
            continue;
        };
        if !word
            .parts
            .iter()
            .any(|part| matches!(part, kanhe::shell::Part::Substitution))
        {
            continue;
        }
        found.examined += 1;
        // The first operator at the acquisition's own depth ends the command carrying it: the
        // substitution's interior words sit deeper, and the command's own arguments are not operators.
        let mut end = at + 1;
        while let Some(next) = words.get(end) {
            if next.depth == word.depth && next.operator {
                break;
            }
            end += 1;
        }
        if !guard_after(words, end, word.depth, variable, stops) {
            found.unguarded.push(word.line);
        }
    }
    found
}

/// Whether `words[at]` opens an `||`: the lexer emits one word per metacharacter, so the list separator
/// arrives as two adjacent `|` words — the same adjacency [`kanhe::shell::command_positions`] uses to read
/// `>&` as one redirection.
fn is_or_separator(words: &[kanhe::shell::Word], at: usize, depth: usize) -> bool {
    let half = |index: usize| {
        words
            .get(index)
            .is_some_and(|word| word.operator && word.depth == depth && word.value == "|")
    };
    half(at) && half(at + 1) && words[at].start + 1 == words[at + 1].start
}

/// Whether the operator ending an acquisition's command guards it: `||` followed by a stop as [`stops_of`]
/// reads them, by a fallback assignment to the name being acquired, or by a block whose first command is one
/// of those two. Newlines after the `||` or the brace continue the list, as bash continues it, and are
/// stepped over.
fn guard_after(
    words: &[kanhe::shell::Word],
    at: usize,
    depth: usize,
    variable: &str,
    stops: &BTreeSet<String>,
) -> bool {
    if !is_or_separator(words, at, depth) {
        return false;
    }
    // The next word at the list's own depth, over the newlines bash continues a `||` list or a brace group
    // across.
    let next_at = |mut at: usize| -> Option<usize> {
        loop {
            at += 1;
            match words.get(at) {
                Some(word) if word.operator && word.depth == depth && word.value == "\n" => {}
                Some(_) => return Some(at),
                None => return None,
            }
        }
    };
    let Some(first) = next_at(at + 1) else {
        return false;
    };
    let word = &words[first];
    if word.operator || word.depth != depth {
        return false;
    }
    // `|| stop`, or `|| NAME=` supplying the acquired name's fallback.
    if stops.contains(&word.value) || word.assigns() == Some(variable) {
        return true;
    }
    // `|| { …`: the block's first command decides.
    if word.value == "{" && word.written == "{" {
        let Some(command) = next_at(first) else {
            return false;
        };
        let first_command = &words[command];
        return !first_command.operator
            && first_command.depth == depth
            && (first_command.assigns().is_some() || stops.contains(&first_command.value));
    }
    false
}

/// A line bash does not join guards nothing: the escaped backslash ends the acquisition's command, and the
/// `|| cannot_judge` on the next line is another command — measured, a syntax error there. The retired join
/// read the two as one statement and reported the acquisition guarded.
#[test]
fn an_acquisition_guarded_only_across_a_line_bash_does_not_join_is_unguarded() {
    let stops = BTreeSet::from(["cannot_judge".to_string()]);
    let words = kanhe::shell::lex_placed("title=$(gh pr view x) \\\\\n|| cannot_judge \"no\"\n")
        .expect("the fixture is placed");
    let found = unguarded_acquisitions(&words, &stops);
    assert_eq!(found.examined, 1);
    assert_eq!(
        found.unguarded,
        vec![1],
        "bash ends the acquisition's command at the escaped backslash, so the guard on the next line guards \
         nothing"
    );
    // The control: a real continuation keeps the guard in the acquisition's own command.
    let words = kanhe::shell::lex_placed("title=$(gh pr view x) \\\n|| cannot_judge \"no\"\n")
        .expect("the fixture is placed");
    let found = unguarded_acquisitions(&words, &stops);
    assert_eq!(found.examined, 1);
    assert!(
        found.unguarded.is_empty(),
        "a backslash-newline joins, as bash joins, and the guard holds"
    );
}

/// A block guard whose first command does not stop swallows the failure: `|| { true; }` runs `true` and the
/// acquisition's failure ends nothing. Reading the opener alone admitted it.
#[test]
fn a_block_guard_whose_first_command_is_not_a_stop_is_unguarded() {
    let stops = BTreeSet::from([
        "cannot_judge".to_string(),
        "exit_for_the_gates_refusal".to_string(),
    ]);
    let words =
        kanhe::shell::lex_placed("x=$(tool) || { true; }\n").expect("the fixture is placed");
    let found = unguarded_acquisitions(&words, &stops);
    assert_eq!(found.examined, 1);
    assert_eq!(
        found.unguarded,
        vec![1],
        "the block opens and its first command returns, so the failure is ignored rather than refused"
    );
    // The controls: a block opening with a stop, and one opening with a fallback assignment, both guard.
    for text in [
        "x=$(tool) || { cannot_judge \"no\"; }\n",
        "x=$(tool) || {\n    x=\"\"\n}\n",
    ] {
        let words = kanhe::shell::lex_placed(text).expect("the fixture is placed");
        let found = unguarded_acquisitions(&words, &stops);
        assert_eq!(found.examined, 1);
        assert!(
            found.unguarded.is_empty(),
            "{text:?} guards the acquisition — its block's first command stops or supplies the value"
        );
    }
}

/// Every tracked script is named by [`WRAPPERS`] or is the shared library, and every wrapper loads it.
///
/// Without this the array is a second list beside the tree: a new wrapper would front a gate with its exit
/// classes compared to nothing, while every direction above kept passing over the two it does name.
///
/// **The members come from the enumeration the citation check reads**, every tracked file under `scripts/`,
/// rather than from a rule of this direction's own. Finding wrappers by their sourcing line cannot find the
/// wrapper that matters most — one that never loaded the library — so loading the library is asserted of each
/// member rather than used to find them, and the array is held to the set both ways.
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
            (line.starts_with("source ") || line.starts_with(BOOTSTRAP_SOURCE))
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
            body.iter()
                .any(|line| line.contains(r#"$(value_refusal "$@")"#)),
            "{wrapper}'s `{guard}` is handed a value and does not hand it to the library's `value_refusal`, \
             the one rule both wrappers judge a value by: a refused argument does not become admitted by \
             sitting in a value position, and an admitted one does not reach the tool by being read as text"
        );
    }
    // The rule itself, read once from the library both guards delegate to.
    let library = Source::of(read(&root, kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY));
    let executed: Vec<String> = library
        .shell()
        .lines()
        .map(|line| line.trim().to_string())
        .collect();
    let start = executed
        .iter()
        .position(|line| line == "value_refusal() {")
        .expect("the library defines `value_refusal`");
    assert!(
        executed[start + 1..]
            .iter()
            .take_while(|line| *line != "}")
            .any(|line| line.contains("== -*")),
        "the library's `value_refusal` never judges a value's shape, so neither wrapper does"
    );
}

/// A site's line is the one a reader of the file finds it on, after an ANSI-C escape that consumes a newline.
///
/// `$'a\` followed by a newline continues the quote on the next line, so the `exit` below stands on line 3; a
/// reader that advanced past the escape without counting it reported line 2.
#[test]
fn a_site_after_an_ansi_c_escaped_newline_reports_its_own_line() {
    assert_eq!(
        exit_sites("x=$'a\\\nb'\nexit 3\n"),
        Ok(vec![(3, "3".to_string())]),
        "the exit after an ANSI-C escaped newline stands on line 3"
    );
}

/// The library and the function reach bash as arguments, never as shell text.
///
/// A path holding a space, spliced into the `-c` string, splits into two words and names a file that does not
/// exist; passed as a positional parameter it is one word, whatever it holds.
#[test]
fn a_library_path_holding_a_space_is_one_argument() {
    let scratch = support::fixture::Scratch::claim("tianheng-bash-body");
    let dir = scratch.path().join("a b");
    std::fs::create_dir(&dir).expect("create a directory whose name holds a space");
    std::fs::write(dir.join("lib.sh"), "stop() {\n    exit 7\n}\n").expect("write the library");
    let body = body_as_bash_holds_it(scratch.path(), "a b/lib.sh", "stop");
    assert!(
        body.contains("exit 7"),
        "the body bash holds for `stop`, read through a path with a space: {body}"
    );
}

/// A scratch root that cannot be removed fails the run that dropped it, rather than being left behind unsaid.
#[test]
fn a_scratch_root_that_cannot_be_removed_is_a_failure() {
    let scratch = support::fixture::Scratch::claim("tianheng-unremovable");
    let held = scratch.path().join("held");
    std::fs::create_dir(&held).expect("create a directory inside the scratch root");
    std::fs::write(held.join("file"), "x").expect("write a file the removal must reach");
    let Some(guard) = xingbiao::Unreadable::try_new(&held) else {
        return;
    };
    let root = scratch.path().to_path_buf();
    let dropped = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(scratch)));
    drop(guard);
    std::fs::remove_dir_all(&root).expect("remove the scratch root once it is readable again");
    let message = dropped
        .expect_err("dropping a scratch root that cannot be removed passed in silence")
        .downcast::<String>()
        .map(|message| *message)
        .unwrap_or_default();
    assert!(
        message.starts_with("Scratch: removing"),
        "the failure names what it could not remove: {message}"
    );
}

/// A library named without a slash is the fixture's file, never one `source` finds on `PATH`.
///
/// The fixture is named `bash`, which is on `PATH` wherever this runs, so a name handed to `source` as written
/// reads the host's executable instead.
#[test]
fn a_library_named_without_a_slash_is_the_fixture_s_file() {
    let scratch = support::fixture::Scratch::claim("tianheng-slashless-library");
    std::fs::write(scratch.path().join("bash"), "X=1\n").expect("write the library");
    assert_eq!(
        value_as_bash_holds_it(scratch.path(), "bash", "X"),
        Some("1".to_string()),
        "the value bash holds after sourcing the fixture named `bash`"
    );
}

/// A declared name assigned again when a wrapper runs ends it in the unjudged class, in every form bash assigns by.
///
/// The static declaration check reads assignment words only; this is what holds the rest. The library declares
/// each name `readonly` and bash refuses the assignment: most forms end the shell with status `1` outside the ERR
/// trap, which the library's EXIT trap holds to the unjudged class, and `declare` and `local` fail as builtins
/// through the ERR trap. The class is the unjudged one either way.
#[test]
fn a_declared_name_assigned_again_is_the_unjudged_class() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = root.join(kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    let unjudged = i32::from(verdict_channel::wrapper_exit(Kind::CannotJudge));
    // The gate's channel carries a violation, so the refusal after the assignment exits by the declared values:
    // a moved one is seen in the status rather than hidden behind a channel that carries nothing.
    let scratch = support::fixture::Scratch::claim("tianheng-reassigned-name");
    let verdict = scratch.path().join("verdict");
    std::fs::write(&verdict, verdict_channel::rendered(Kind::Violation))
        .expect("write the gate's verdict");
    for form in [
        "WRAPPER_EXIT_VIOLATION=3",
        "WRAPPER_EXIT_UNJUDGED+=0",
        "read GATE_CLEAN_CLASS <<< Violation",
        "printf -v WRAPPER_EXIT_UNJUDGED 1",
        "(( WRAPPER_EXIT_VIOLATION = 2 ))",
        "declare GATE_VIOLATION_CLASS=Clean",
        "f() { local WRAPPER_EXIT_UNJUDGED=1; }; f",
    ] {
        let output = support::bash::bash()
            .args([
                "-c",
                &format!(
                    r#"set -Eeuo pipefail; trap '' PIPE; WRAPPER_SUBJECT=probe; source "$1"; install_exit_class_trap; verdict_file=$2; {form}; exit_for_the_gates_refusal out"#
                ),
                "probe",
            ])
            .arg(&library)
            .arg(&verdict)
            .output()
            .expect("run the library with a declared name assigned again");
        assert_eq!(
            output.status.code(),
            Some(unjudged),
            "`{form}` after the library is loaded: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

/// Which functions stop is read from the lexed words: a one-line function is a function, and a quoted brace in a
/// message closes nothing.
#[test]
fn the_stops_are_the_functions_reaching_cannot_judge() {
    // `fake` prints the name as an argument, quoted and not, and `brace` prints a lone `{` and `}` as arguments.
    let script = "f() {\n    tell \"}\"\n    cannot_judge x\n}\ng() { f; }\nh() { :; }\n\
                  fake() { printf '%s' 'cannot_judge'; printf '%s' cannot_judge; }\n\
                  brace() { printf '%s\\n' { \"$1\"; }\nlater() { cannot_judge y; }\n\
                  after_sub() { printf '%s' \"$(date)\" cannot_judge; }\n\
                  after_tick() { echo `date` cannot_judge; }\n\
                  redirect() { printf x >&2 cannot_judge; }\n\
                  test_expr() { [[ a && cannot_judge ]]; }\n\
                  inner() { x=$(cannot_judge z); }\n";
    assert_eq!(
        stops_of(&[script]),
        BTreeSet::from(["cannot_judge", "f", "g", "later"].map(String::from)),
        "the stops of {script:?}"
    );
}

/// The pass line a controlled gate prints is one the wrappers' `require_one_pass` accepts, asked of the
/// library's own function rather than of a copy of its pattern.
#[test]
fn the_controlled_gate_s_pass_line_is_the_one_the_wrappers_require() {
    let Some(root) = workspace_root() else {
        return;
    };
    let library = root.join(kanhe::gate_identity::WRAPPERS_SHARED_LIBRARY);
    let output = support::bash::bash()
        .args([
            "-c",
            r#"set -Eeuo pipefail; WRAPPER_SUBJECT=probe; source "$1"; require_one_pass "$2""#,
            "probe",
        ])
        .arg(&library)
        .arg(support::fixture::GATE_PASS_LINE)
        .output()
        .expect("run the library's require_one_pass");
    assert!(
        output.status.success(),
        "`require_one_pass` refuses the line the controlled gates print, so every wrapper direction would stop \
         at the gate for a reason the fixture made: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}
