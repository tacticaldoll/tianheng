//! `gate-shape-contract`'s reaction: the repository's own gate surface, enumerated from tracked content and
//! held to the shape every gate in it already has.
//!
//! Six structural classes recurred across that surface in one window, every one repaired a site at a time and
//! twice leaving a sibling behind. The Definition of Done binds the gate *list* to CI; nothing bound a gate's
//! *shape* to anything, so the seventh gate inherited the shape only if its author read six others first.
//!
//! Why a Rust reaction rather than a seventh shell gate: a `PINNED-BY` citation resolves only to a
//! harness-registered Rust function, so a shell-defended capability could not pin the bounds this one declares
//! — they would land `UNPINNED` and move the register projection's leading figure off zero. It also rides the
//! existing `cargo test` line, where a shell gate would have added a Definition of Done entry and a CI step.
//!
//! What it does **not** claim: three of the six classes are semantic, and each is a declared bound with a
//! pinning test below rather than an approximation. Form conformance is not substance — see the projection's
//! own header, which says so where a reader meets the table.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

use tianheng::testing::assert_projection_matches;

mod support;
use support::region::Source;

/// The projection this reaction holds fresh.
const PROJECTION: &str = "docs/gate-shape-contract.md";

/// The gate whose Definition-of-Done membership is excused by name, because it runs at publish time: no
/// development checkout is a release snapshot, so a pre-flight run of it could only ever refuse.
///
/// A policy exemption rather than an observation bound — a bound says a reaction stops at a shape, this says
/// one named instance is excused from a requirement — so it is not in the register, and it is checked live
/// below rather than merely honoured.
const PUBLISH_TIME_GATE: &str = "scripts/check_publish_source.sh";

/// The one unit outside the pairing that may carry the backstop's name, because it **defines** it.
const BACKSTOP_LIBRARY: &str = "scripts/lib/exit_contract.sh";

/// The shared backstop's name, as a unit installing it writes it.
const BACKSTOP: &str = "exit_contract_backstop";

/// The fixture gate, twin and directory, as basenames joined where they are needed.
///
/// Written in pieces deliberately: `scripts/check_reference_integrity.sh` reads a repository-shaped path in
/// any tracked `.md` or `.rs` as a claim that the file exists, and these exist only inside a temporary
/// directory this test builds and removes. A reader who greps for one and finds nothing cannot tell
/// illustrative prose from a bad checkout, which is the class that gate is for.
const SCRIPTS: &str = "scripts";
const FIXTURE_GATE: &str = "check_probe.sh";
const FIXTURE_TWIN: &str = "test_probe.sh";

/// Which file a failure of a property names.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject {
    Gate,
    Twin,
    /// The one property over both files at once: Definition-of-Done membership.
    BothFiles,
}

/// Whether one file holds one property.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Holds {
    Yes,
    No,
    /// Held by a declared policy exemption rather than by the file — today, only the publish-time gate's
    /// absence from the Definition of Done.
    ByExemption,
}

impl Holds {
    fn cell(self) -> &'static str {
        match self {
            Holds::Yes => "yes",
            Holds::No => "**NO**",
            Holds::ByExemption => "twin only (exempt)",
        }
    }
}

/// One property: the column it prints as, the file a failure names, the remedy that failure states, and how it
/// is measured.
struct Property {
    label: &'static str,
    subject: Subject,
    remedy: &'static str,
    holds: fn(&Unit) -> Holds,
    /// What this offence can say about *this* gate that a static remedy cannot — the label it wrote against the
    /// label its name asks for. Most properties have nothing to add, because the absence *is* the whole fact.
    detail: fn(&Unit) -> Option<String>,
}

/// Every property, in the order the projection prints them and the order a failure names them.
///
/// Some are properties of the gate, some of its twin, and one of `AGENTS.md`'s Definition of Done — each entry
/// says which. Each is a class this repository observed rather than a checklist item assembled for symmetry. How
/// many there are is printed by the projection and stated in no prose: this array is the only place that knows.
///
/// **One array, not an enum beside a list of its variants.** Written the second way first — a `Property` enum
/// with an `ALL` constant — and it carried a silent false negative of exactly the kind this capability exists
/// to refuse: a tenth variant compiles once it has a label and a remedy, and is then never measured, because
/// nothing forces it into `ALL`. Every test here iterates that list, so the new property would go unchecked
/// while the reaction reported the surface conformant. Here there is no second list to forget.
const PROPERTIES: [Property; 11] = [
    Property {
        label: "backstop",
        subject: Subject::Gate,
        remedy: "source scripts/lib/exit_contract.sh and invoke exit_contract_backstop, so an unhandled \
                 command's status cannot escape as a foreign exit code",
        holds: |unit| holds(installs_the_backstop(&unit.gate_text)),
        detail: |_| None,
    },
    Property {
        label: "backstop label",
        subject: Subject::Gate,
        remedy: "pass the backstop this gate's own name, written as a literal: its basename with `check_` and \
                 `.sh` removed and underscores read as spaces",
        holds: |unit| match backstop_label(&unit.gate_text) {
            BackstopLabel::Literal(label) => holds(label == name_from_basename(&unit.gate)),
            BackstopLabel::Computed | BackstopLabel::Absent => Holds::No,
        },
        detail: |unit| {
            Some(match backstop_label(&unit.gate_text) {
                BackstopLabel::Literal(label) => format!(
                    "it passes `{label}` where its basename asks for `{}`",
                    name_from_basename(&unit.gate)
                ),
                // Said as what it is. Reporting a computed label as a *mismatch* would compare against a label
                // the gate never wrote — and the shape most likely to appear here, deriving the name from `$0`,
                // is a better implementation than any literal rather than a naming error.
                BackstopLabel::Computed => {
                    "its label is built by expansion, and a reaction that reads text \
                     cannot confirm one"
                        .to_string()
                }
                BackstopLabel::Absent => {
                    "it invokes no backstop, so it passes no label".to_string()
                }
            })
        },
    },
    Property {
        label: "one checked capture",
        subject: Subject::Gate,
        remedy: "materialize the observation source, check the producer's status in the parent shell, then read \
                 it — `scripts/lib/capture.sh` holds the rule; a process substitution's status never reaches the \
                 parent, so a producer that emits some rows and then fails leaves a partial read judged as a \
                 whole one",
        holds: |unit| holds(reads_through_one_checked_capture(&unit.gate_text)),
        detail: |_| None,
    },
    Property {
        label: "contract header",
        subject: Subject::Gate,
        remedy: "state the three-way contract in the header — `Exit 0 <clean>, 1 <violation>, 2 cannot judge` \
                 — with the verdict words for 0 and 1 chosen for this gate's subject",
        holds: |unit| holds(declares_the_three_way_contract(&unit.gate_text)),
        detail: |_| None,
    },
    Property {
        label: "target directory",
        subject: Subject::Gate,
        remedy: "take the repository to judge as `${1:-<this checkout>}`, so a fixture can be pointed at it; a \
                 gate that cannot be pointed at a fixture cannot be observed refusing",
        holds: |unit| holds(accepts_a_target_directory(&unit.gate_text)),
        detail: |_| None,
    },
    Property {
        label: "twin exists",
        subject: Subject::Twin,
        remedy: "add the companion failure matrix beside it, named by substituting `test_` for `check_`; a gate \
                 nobody has watched refuse is protection claimed rather than observed",
        holds: |unit| holds(unit.twin_text.is_some()),
        detail: |_| None,
    },
    Property {
        label: "exit codes",
        subject: Subject::Twin,
        remedy: "assert the expected exit CODE in the matrix, not merely non-zero: a 1 collapsing into a 2 \
                 rode green through CI exactly once this way",
        holds: |unit| unit.twin_holds(asserts_exit_codes),
        detail: |_| None,
    },
    Property {
        label: "both directions",
        subject: Subject::Twin,
        remedy: "hold both an `expect_pass` and an `expect_fail` direction; a gate that refuses everything \
                 satisfies a refusal-only matrix completely",
        holds: |unit| unit.twin_holds(holds_both_directions),
        detail: |_| None,
    },
    Property {
        label: "read-only",
        subject: Subject::Twin,
        remedy: "assert the judged repository is unchanged after the gate runs, on a fixture the gate has not \
                 already judged, and say `mutated` when it is not",
        holds: |unit| unit.twin_holds(asserts_read_only),
        detail: |_| None,
    },
    Property {
        label: "silent clean run",
        subject: Subject::Twin,
        remedy: "capture a clean run's stderr alone (`2>&1 >/dev/null`) and assert the variable it assigned is \
                 empty; nothing about the exit code can see a gate printing cannot-judge on every clean input",
        holds: |unit| unit.twin_holds(asserts_a_silent_clean_run),
        detail: |_| None,
    },
    Property {
        label: "definition of done",
        subject: Subject::BothFiles,
        remedy: "add the file to AGENTS.md's Definition of Done block, the single source for the local \
                 pre-flight list; a gate nothing invokes is a comment",
        holds: |unit| {
            // The publish-time gate's absence is excused; its twin's membership is not. When that gate is
            // PRESENT the cell reads as ordinary membership, and the staleness of the exemption is refused by
            // `every_gate_and_twin_is_reachable_from_the_definition_of_done` rather than hidden in a cell.
            if unit.gate == PUBLISH_TIME_GATE && !unit.gate_in_dod {
                if unit.twin_in_dod {
                    Holds::ByExemption
                } else {
                    Holds::No
                }
            } else {
                holds(unit.gate_in_dod && unit.twin_in_dod)
            }
        },
        detail: |_| None,
    },
];

fn holds(condition: bool) -> Holds {
    if condition { Holds::Yes } else { Holds::No }
}

/// One gate, its twin, and everything the properties are measured from.
struct Unit {
    gate: String,
    twin: String,
    gate_text: Source,
    twin_text: Option<Source>,
    gate_in_dod: bool,
    twin_in_dod: bool,
}

impl Unit {
    /// A twin property: `No` when there is no twin, which is honest rather than noisy — a file that does not
    /// exist holds none of them, and each absence names a real one.
    fn twin_holds(&self, held: fn(&Source) -> bool) -> Holds {
        match &self.twin_text {
            Some(text) => holds(held(text)),
            None => Holds::No,
        }
    }
}

/// The repository layout, or `None` outside a checkout.
///
/// Split from [`workspace_root`] so the marker discipline itself can be observed failing without a test
/// mutating the process environment — `set_var` is unsafe and would race every other test in this binary.
fn locate_layout(root: PathBuf, marker_set: bool) -> Option<PathBuf> {
    if root.join("scripts").is_dir() && root.join("AGENTS.md").is_file() {
        return Some(root);
    }
    assert!(
        !marker_set,
        "scripts/ and AGENTS.md expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — \
         a governance reaction that quietly does nothing in CI is the shape this capability argues against"
    );
    None
}

/// The workspace root, or `None` outside a checkout.
fn workspace_root() -> Option<PathBuf> {
    locate_layout(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_some(),
    )
}

/// Tracked paths under `pathspec`, read with `-z`.
///
/// `git ls-files` quotes a non-ASCII path by default, so a quoted path would name no file on disk — the trap
/// `scripts/check_bound_register.sh` documents at its own single read site. The exit status is checked here,
/// in the caller, which is the direction this repository has re-opened most often.
fn tracked(root: &Path, pathspec: &str) -> Vec<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["ls-files", "-z", pathspec])
        .output()
        .unwrap_or_else(|err| panic!("cannot run `git ls-files` in {root:?}: {err}"));
    assert!(
        output.status.success(),
        "`git ls-files` failed in {root:?}: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
        .stdout
        .split(|byte| *byte == 0)
        .filter(|entry| !entry.is_empty())
        .map(|entry| {
            String::from_utf8(entry.to_vec())
                .unwrap_or_else(|err| panic!("a tracked path under {root:?} is not UTF-8: {err}"))
        })
        .collect()
}

/// Every tracked shell unit under `scripts/`.
///
/// Filtered in Rust rather than by pathspec glob: git matches pathspec wildcards without `FNM_PATHNAME`, so
/// `scripts/*.sh` already reaches into subdirectories and the glob would be describing something other than
/// what it appears to say.
fn shell_units(root: &Path) -> Vec<String> {
    let mut units: Vec<String> = tracked(root, "scripts")
        .into_iter()
        .filter(|path| path.ends_with(".sh"))
        .collect();
    units.sort();
    units
}

/// The twin a gate is paired with, by substituting `check_` for `test_` in its basename.
fn twin_of(gate: &str) -> String {
    let (dir, base) = gate.rsplit_once('/').unwrap_or(("", gate));
    let twin_base = base.replacen("check_", "test_", 1);
    if dir.is_empty() {
        twin_base
    } else {
        format!("{dir}/{twin_base}")
    }
}

/// Read a tracked file, naming it if it cannot be read.
/// A tracked text, as a [`Source`] rather than a `String`: the region a property is about is then decided in the
/// type, and a recognizer that wants executed text cannot be handed the whole file.
fn read(root: &Path, relative: &str) -> Source {
    let path = root.join(relative);
    Source::of(
        std::fs::read_to_string(&path).unwrap_or_else(|err| panic!("cannot read {path:?}: {err}")),
    )
}

/// A shell gate's header: everything before the first `set -` line.
///
/// Not [`Source::header`], which cuts at the first `##` heading — that is a Markdown document's shape. A gate's
/// self-description ends where its execution begins, and the region types carry the document rule because that is
/// the one every registered generated document shares.
fn gate_header(gate: &Source) -> &str {
    let text = gate.whole();
    match text.find("\nset -") {
        Some(index) => &text[..index],
        None => text,
    }
}

/// Property 1 — the shared backstop is sourced **and invoked**.
///
/// Sourcing alone installs nothing, and the difference is not hypothetical: the trap is what turns an
/// unhandled command's own status into the family's cannot-judge, and a gate that sourced without invoking
/// would exit 7 or 131 with no output.
fn installs_the_backstop(gate: &Source) -> bool {
    let sourced = gate.shell().lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with("source ") || trimmed.starts_with(". "))
            && trimmed.contains("lib/exit_contract.sh")
    });
    let invoked = gate.shell().starts_a_line_with(&format!("{BACKSTOP} ")[..]);
    sourced && invoked
}

/// What a gate wrote as its backstop label.
enum BackstopLabel {
    Literal(String),
    /// Built by expansion. Not resolved: this reaction reads a gate's text and does not evaluate it, so it
    /// cannot confirm such a label — and must not report an unconfirmed one as correct.
    Computed,
    /// No invocation at all, so there is no label. The backstop property reports that absence too, and both
    /// offences stand: each names something real, which is the precedent an absent twin already set.
    Absent,
}

/// The label a gate passes to the shared backstop.
///
/// A trailing comment is cut before the argument is read, for the reason the Definition-of-Done parse cuts one:
/// a comment is not part of what runs, and reading it as part of the label would report a mismatch against text
/// the shell never sees.
fn backstop_label(gate: &Source) -> BackstopLabel {
    let invocation = gate
        .shell()
        .lines()
        .find(|line| line.trim_start().starts_with(&format!("{BACKSTOP} ")[..]));
    let Some(line) = invocation else {
        return BackstopLabel::Absent;
    };
    let argument = line.trim_start()[BACKSTOP.len()..].trim();
    let argument = match argument.find(" #") {
        Some(index) => argument[..index].trim(),
        None => argument,
    };
    let literal = match (argument.starts_with('\''), argument.starts_with('"')) {
        (true, _) if argument.len() >= 2 && argument.ends_with('\'') => {
            &argument[1..argument.len() - 1]
        }
        (_, true) if argument.len() >= 2 && argument.ends_with('"') => {
            &argument[1..argument.len() - 1]
        }
        // Unquoted is accepted: the shell accepts it for a single-word label, and refusing it would be a rule
        // about quoting rather than about naming.
        _ => argument,
    };
    if literal.is_empty() || literal.contains('$') || literal.contains('`') {
        return BackstopLabel::Computed;
    }
    BackstopLabel::Literal(literal.to_string())
}

/// The name a gate's own basename asks it to answer to: `check_` and `.sh` removed, underscores read as spaces.
///
/// Derived rather than looked up. A table from gate to label would be a second declaration of the gate's name,
/// and would rot exactly as the thing it checks.
fn name_from_basename(gate: &str) -> String {
    let base = gate.rsplit_once('/').map(|(_, base)| base).unwrap_or(gate);
    let stem = base.strip_prefix("check_").unwrap_or(base);
    let stem = stem.strip_suffix(".sh").unwrap_or(stem);
    stem.replace('_', " ")
}

/// Property — no observation source is consumed through a process substitution whose producer can fail.
///
/// `BACKLOG.md` recorded a swallowed subshell status as this window's most recurring class — nine mentions, every
/// recurrence repaired one site at a time — and the review found eight more. Both directions are measured: a
/// truncated `git ls-files --eol` made a gate report `ok (1 tracked text files)` at exit 0, and a truncated
/// `git log` made another invent `[Unreleased] must be empty` at exit 1.
///
/// A **builtin over data already in memory** is permitted: `printf` re-splitting a held variable has no I/O to
/// fail at, so demanding a temporary file for it would make the gate longer without making it safer. Permitted by
/// naming the builtin rather than by listing the call sites, because a list of sites rots on the next edit and
/// would make the property about where code is rather than what it does.
///
/// Every occurrence on a line is checked, not the first: a line whose first producer is a builtin and whose second
/// is `git` would otherwise pass.
fn reads_through_one_checked_capture(gate: &Source) -> bool {
    gate.shell().lines().all(|line| {
        line.split("< <(").skip(1).all(|producer| {
            matches!(
                producer.split_whitespace().next().unwrap_or(""),
                "printf" | "echo"
            )
        })
    })
}

/// The declared bound: a permitted builtin **piped onward** is still permitted.
///
/// The permission is granted on a stated reason — a builtin over data already in memory has no I/O to fail at —
/// and the recognizer applies it by reading the producer's first word. `printf … | sort` therefore passes while
/// `sort` is an external process whose failure the parent never sees.
///
/// Three legs, because the first alone would prove nothing:
///
///   1. the under-reaction is **accepted**, which is the bound;
///   2. an external producer is still **refused**, so the pin cannot hold for a recognizer that never fires;
///   3. the obvious repair — also refuse a producer containing `|` — **false-positives on the live shape**, which
///      is the reason this is declared rather than closed. Asserting it here keeps that reason executable: if a
///      future parameter expansion no longer carries a pipe, this leg fails and the bound is worth re-examining.
#[test]
fn a_builtin_piped_into_an_external_command_is_a_stated_bound() {
    // As it appears in the tree, twice: a `|` inside a parameter expansion, over data already in memory.
    let live = r#"    while IFS= read -r one; do :; done < <(printf '%s\n' "${b//|/$'\n'}")"#;
    let piped_onward = r#"    done < <(printf '%s\n' "$rows" | sort)"#;
    let external = r#"    done < <(git ls-files -z)"#;

    assert!(
        reads_through_one_checked_capture(&Source::of(live)),
        "the live shape is permitted, as the requirement's reason intends"
    );
    assert!(
        reads_through_one_checked_capture(&Source::of(piped_onward)),
        "and so is a builtin piped into an external command — the stated bound"
    );
    assert!(
        !reads_through_one_checked_capture(&Source::of(external)),
        "while an external producer is still refused, so this bound is not a recognizer that never fires"
    );

    // Leg three: the repair that would close it, measured against the tree rather than assumed.
    let naive_refuses = |line: &str| {
        line.split("< <(")
            .skip(1)
            .any(|producer| producer.contains('|'))
    };
    assert!(
        naive_refuses(piped_onward),
        "refusing a producer that contains a pipe would catch the bound"
    );
    assert!(
        naive_refuses(live),
        "and would also refuse the live shape — a pipe inside `${{…}}` is not a pipe operator, and telling them \
         apart needs shell parsing rather than text, which is why this is declared instead of closed"
    );
}

/// What this projection says about each bound the capability declares, keyed by its declaring scenario's heading.
///
/// **A single array, held to the specification in both directions** by
/// [`the_projection_discloses_every_declared_bound`]. The figure in the projection is `len()`, never typed — the
/// paragraph it replaced typed both the figure and the membership ("Three of the six classes … a fourth bound is
/// about coverage") and a bound added in the same window went unlisted, invisible because a literal in a generated
/// document's template is compared with itself by the freshness check.
///
/// Keyed by the **exact heading**, not by a distinctive substring: recognizing a declaration by a marker it merely
/// contains is the trap this family has paid for four times.
const BOUND_NOTES: [(&str, &str); 5] = [
    (
        "Whether an enumeration carries a vacuity guard is not observed — a stated bound",
        "A gate that iterates nothing and reports clean holds every column below.",
    ),
    (
        "Whether a read's status is checked in the parent shell is not observed — a stated bound",
        "The backstop this table does check narrows the damage without detecting the shape.",
    ),
    (
        "Whether a gate's 1-versus-2 assignment is correct is not observed — a stated bound",
        "The twin is required to assert codes, never to assert the right ones. A `return`-instead-of-`exit` \
         inversion held every column while reporting every violation as cannot-judge.",
    ),
    (
        "A permitted builtin piped into an external command is still permitted — a stated bound",
        "The permission reads the producer's first word, so a builtin's exemption reaches a pipeline stage that \
         has I/O to fail at. Refusing a producer containing a pipe was measured and refuses this tree's own \
         legitimate sites too.",
    ),
    (
        "Shell units that are not a gate or its twin are outside the surface — a stated bound",
        "Coverage rather than form: such units are listed at the end, so their absence from the table is visible \
         rather than inferred.",
    ),
];

/// The bound headings this capability's specification declares.
fn declared_bound_headings(root: &Path) -> Vec<String> {
    let spec = root.join("openspec/specs/gate-shape-contract/spec.md");
    let text = std::fs::read_to_string(&spec)
        .unwrap_or_else(|error| panic!("cannot read {spec:?}: {error}"));
    Source::of(text)
        .whole()
        .lines()
        .filter_map(|line| line.strip_prefix("#### Scenario: "))
        .filter(|heading| heading.trim_end().ends_with("a stated bound"))
        .map(|heading| heading.trim_end().to_string())
        .collect()
}

/// The projection discloses **every** bound the specification declares, and none it does not.
///
/// Both directions, because either alone leaves the hole this reaction was built to close: a bound added to the
/// spec with no note here would go undisclosed, and a note whose bound was retired would disclose a bound that no
/// longer exists. Neither is catchable by the freshness check, which compares the generator's literals with
/// themselves.
#[test]
fn the_projection_discloses_every_declared_bound() {
    let Some(root) = workspace_root() else {
        return;
    };
    let mut declared = declared_bound_headings(&root);
    let mut disclosed: Vec<String> = BOUND_NOTES
        .iter()
        .map(|(heading, _)| (*heading).to_string())
        .collect();
    assert!(
        !declared.is_empty(),
        "no declared bound was read from the specification, so this reaction would hold vacuously"
    );
    declared.sort();
    disclosed.sort();
    assert_eq!(
        disclosed, declared,
        "the projection's bound disclosure and the specification must name the same bounds — a bound added \
         without a note here is a bound the generated document does not mention, and the freshness check cannot \
         see it because the template's own text is on both sides of that comparison"
    );
}

/// Property 2 — the header declares the three-way contract, recognized by **shape, not by wording**.
///
/// Each gate words the verdicts its own way — clean/violation, coherent/incoherent, publishable/wrong source
/// — and each names its own subject better than a shared sentence would. Measured when six gates existed: a
/// probe requiring one literal sentence read 3 of them and would have reported three gates violating a
/// requirement every one of them meets. So the words for 0 and 1 are free and only the third term is fixed,
/// because *cannot judge* is the term the family's contract is about.
fn declares_the_three_way_contract(gate: &Source) -> bool {
    gate_header(gate)
        .lines()
        .filter_map(|line| line.trim_start().strip_prefix('#'))
        .any(|comment| {
            let Some(rest) = comment.trim_start().strip_prefix("Exit 0 ") else {
                return false;
            };
            let Some((clean_verdict, rest)) = rest.split_once(", 1 ") else {
                return false;
            };
            let Some((violation_verdict, _)) = rest.split_once(", 2 cannot judge") else {
                return false;
            };
            !clean_verdict.trim().is_empty() && !violation_verdict.trim().is_empty()
        })
}

/// Property 3 — the gate takes the repository to judge as an argument.
fn accepts_a_target_directory(gate: &Source) -> bool {
    gate.shell().contains("${1:-")
}

/// Property 5 — the twin asserts an expected exit **code**.
fn asserts_exit_codes(twin: &Source) -> bool {
    twin.shell().contains("expected_status")
}

/// Property 6 — the twin holds both a passing and a refusing direction.
///
/// Checked through the twins' own helper names, which is legitimate here by ownership: these files are
/// authored in this repository for this purpose, the same line `observation-bound-register` draws when it
/// requires a scenario heading's form while declining to require a pinning test's name.
fn holds_both_directions(twin: &Source) -> bool {
    twin.shell().contains("expect_pass") && twin.shell().contains("expect_fail")
}

/// Property 7 — the twin asserts the judged repository is unchanged.
///
/// Recognized by the refusal's own word in executed text. The comparison itself cannot be recognized
/// mechanically — the twins compare a `git status` porcelain listing, a `HEAD`, a tag list and a `find`
/// walk in four combinations — so what is required is the authored diagnostic, on the same ownership
/// argument as property 6.
fn asserts_read_only(twin: &Source) -> bool {
    twin.shell().contains("mutated")
}

/// Property 8 — the twin asserts a clean run's stderr is empty.
///
/// By shape: a capture of stderr **alone** (`2>&1 >/dev/null`), and an emptiness test on the variable that
/// capture assigned. Four twins previously grepped that capture for the backstop's own diagnostic, which
/// catches the one line it names and reads every other line as clean — so the shape required here is
/// emptiness, which has no wording to keep in step.
fn asserts_a_silent_clean_run(twin: &Source) -> bool {
    let captures: Vec<&str> = twin
        .shell()
        .lines()
        .filter(|line| line.contains("2>&1 >/dev/null"))
        .filter_map(|line| line.trim_start().split_once("=$(").map(|(name, _)| name))
        .filter(|name| {
            !name.is_empty()
                && name
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        })
        .collect();
    captures.iter().any(|name| {
        twin.shell().lines().any(|line| {
            line.contains("-z")
                && (line.contains(&format!("${name}")) || line.contains(&format!("${{{name}}}")))
        })
    })
}

/// The commands in `AGENTS.md`'s Definition of Done block.
///
/// Located by heading and fence, and bounded by the next `## ` heading so a removed fence cannot be answered
/// by the next one further down the document. Every failure here is loud: a block whose shape changed, or one
/// that parses to zero commands, must refuse rather than report every property of nothing satisfied — the
/// flattering direction, and the one `scripts/check_dod_coherence.sh` already refuses for this same list.
fn definition_of_done(root: &Path) -> Vec<String> {
    let source = read(root, "AGENTS.md");
    let text = source.whole();
    let heading = "\n## Definition of Done\n";
    let start = text.find(heading).unwrap_or_else(|| {
        panic!(
            "AGENTS.md has no `## Definition of Done` heading — the block this reaction reads has moved or \
             been renamed, which must fail loudly rather than parse to nothing"
        )
    });
    let section = &text[start + heading.len()..];
    let section = match section.find("\n## ") {
        Some(index) => &section[..index],
        None => section,
    };
    let fence = "```bash\n";
    let body_start = section.find(fence).unwrap_or_else(|| {
        panic!(
            "AGENTS.md's Definition of Done section holds no ```bash fence — the block's shape has changed"
        )
    }) + fence.len();
    let body = &section[body_start..];
    let end = body
        .find("\n```")
        .unwrap_or_else(|| panic!("AGENTS.md's Definition of Done fence is never closed"));
    // Trailing comments are cut, not only whole-line ones. Every entry in this block carries a comment
    // explaining what its gate is for, and several of those name other paths — so membership read off the whole
    // line would answer yes for a file that is merely *discussed*. The direction that matters is the exemption:
    // the publish-time gate's comment is where a reader would most naturally mention the gate that runs at
    // publish time, and a false yes there reports its exemption stale.
    let commands: Vec<String> = body[..end]
        .lines()
        .map(|line| match line.find('#') {
            Some(index) => line[..index].trim(),
            None => line.trim(),
        })
        .filter(|command| !command.is_empty())
        .map(str::to_string)
        .collect();
    assert!(
        !commands.is_empty(),
        "AGENTS.md's Definition of Done block parsed to zero commands; every membership question would then \
         answer itself"
    );
    commands
}

/// Whether the Definition of Done **invokes** `path`, rather than merely naming it.
///
/// Measured defect: `commands.iter().any(|c| c.contains(path))` accepted
/// `test -f scripts/check_whitespace_hygiene.sh` in the block — a gate that executes nothing satisfying the
/// membership requirement, which is worse than its absence because the projection then reports it reachable.
///
/// An invocation is recognized by the path sitting in **command position**: the interpreter's argument
/// (`bash <path>`) or the command word itself (`<path>` or `./<path>`). Trailing arguments are allowed; a path
/// appearing anywhere later on the line is a mention, which is what `test -f` and `printf` produce.
fn definition_of_done_runs(commands: &[String], path: &str) -> bool {
    commands.iter().any(|command| {
        let mut words = command.split_whitespace();
        let Some(first) = words.next() else {
            return false;
        };
        let invoked = match first {
            "bash" | "sh" => words.next(),
            _ => Some(first),
        };
        invoked.is_some_and(|word| word == path || word.trim_start_matches("./") == path)
    })
}

/// Measure the whole surface. The enumeration, the pairing, and every property, in one pass.
///
/// Fails loudly on an empty enumeration rather than reporting every property of zero gates satisfied. Six
/// occurrences of that direction in one window is why it is a requirement of this capability and not a detail
/// of its implementation.
fn measure(root: &Path) -> Vec<Unit> {
    let tracked_units = shell_units(root);
    let commands = definition_of_done(root);

    let gates: Vec<String> = tracked_units
        .iter()
        .filter(|path| {
            path.rsplit_once('/')
                .map(|(_, base)| base.starts_with("check_"))
                .unwrap_or(false)
        })
        .cloned()
        .collect();

    let present: BTreeSet<&str> = tracked_units.iter().map(String::as_str).collect();

    gates
        .into_iter()
        .map(|gate| {
            let gate_text = read(root, &gate);
            let twin = twin_of(&gate);
            let twin_text = present.contains(twin.as_str()).then(|| read(root, &twin));
            Unit {
                gate_in_dod: definition_of_done_runs(&commands, &gate),
                twin_in_dod: definition_of_done_runs(&commands, &twin),
                gate,
                twin,
                gate_text,
                twin_text,
            }
        })
        .collect()
}

/// Collect one offence per file per property, so a failure names what to repair and where.
///
/// A reaction reporting "the gate surface is non-conformant" has moved the search cost onto the reader, which
/// is the cost this capability exists to remove. `subject` selects the group a test owns, and is the same field
/// that decides which of the two files an offence names.
fn offences(units: &[Unit], subject: Subject) -> Vec<String> {
    let mut found = Vec::new();
    for unit in units {
        for property in PROPERTIES.iter().filter(|p| p.subject == subject) {
            if (property.holds)(unit) != Holds::No {
                continue;
            }
            match subject {
                Subject::Gate => found.push(offence(&unit.gate, property, unit)),
                Subject::Twin => found.push(offence(&unit.twin, property, unit)),
                // Both files are required, so the offence names the one that is missing — naming the gate for
                // an absent twin would send a reader to the wrong file. The publish-time gate's absence is not
                // an offence, which is what its exemption means.
                Subject::BothFiles => {
                    if !unit.gate_in_dod && unit.gate != PUBLISH_TIME_GATE {
                        found.push(offence(&unit.gate, property, unit));
                    }
                    if !unit.twin_in_dod {
                        found.push(offence(&unit.twin, property, unit));
                    }
                }
            }
        }
    }
    found
}

fn offence(file: &str, property: &Property, unit: &Unit) -> String {
    match (property.detail)(unit) {
        Some(detail) => format!("{file}: {} — {detail}; {}", property.label, property.remedy),
        None => format!("{file}: {} — {}", property.label, property.remedy),
    }
}

/// Every offence over the whole surface, in the order the properties are declared.
fn all_offences(units: &[Unit]) -> Vec<String> {
    [Subject::Gate, Subject::Twin, Subject::BothFiles]
        .into_iter()
        .flat_map(|subject| offences(units, subject))
        .collect()
}

#[test]
fn every_gate_holds_the_exit_contract_in_a_checkable_form() {
    let Some(root) = workspace_root() else {
        return;
    };
    let found = offences(&measure(&root), Subject::Gate);
    assert!(
        found.is_empty(),
        "the gate surface does not hold the exit contract in a checkable form:\n{}",
        found.join("\n")
    );
}

#[test]
fn every_gate_has_a_twin_holding_the_five_matrix_properties() {
    let Some(root) = workspace_root() else {
        return;
    };
    let found = offences(&measure(&root), Subject::Twin);
    assert!(
        found.is_empty(),
        "a gate's failure matrix does not hold the shape it is for:\n{}",
        found.join("\n")
    );
}

#[test]
fn every_gate_and_twin_is_reachable_from_the_definition_of_done() {
    let Some(root) = workspace_root() else {
        return;
    };
    let units = measure(&root);
    let found = offences(&units, Subject::BothFiles);
    assert!(
        found.is_empty(),
        "a gate or its twin runs nowhere by default:\n{}",
        found.join("\n")
    );

    if units.is_empty() {
        return;
    }

    // The exemption excuses an ABSENCE, so a publish-time gate that has joined the block means the exemption is
    // stale and must be retired, not silently kept: an exception that only ever permits keeps permitting, and
    // the next reader inherits a licence with no live instance behind it.
    if let Some(publish) = units.iter().find(|unit| unit.gate == PUBLISH_TIME_GATE) {
        assert!(
            !publish.gate_in_dod,
            "{PUBLISH_TIME_GATE} now appears in AGENTS.md's Definition of Done, so its membership exemption is \
             stale: retire the exemption in `gate-shape-contract`'s spec and in this reaction rather than keeping \
             a licence nothing exercises"
        );
        let membership = PROPERTIES
            .iter()
            .find(|property| property.subject == Subject::BothFiles)
            .expect("the membership property is declared");
        let exempt: Vec<&str> = units
            .iter()
            .filter(|unit| (membership.holds)(unit) == Holds::ByExemption)
            .map(|unit| unit.gate.as_str())
            .collect();
        assert_eq!(
            exempt,
            [PUBLISH_TIME_GATE],
            "exactly one gate is excused from Definition-of-Done membership, and it is the publish-time one"
        );
    }
}

/// The tracked shell units under `scripts/` that are neither a gate nor a twin.
///
/// One definition, used by the projection, by the contract-carrying refusal and by the bound that declares the
/// exclusion — three readers of one rule rather than three copies of it.
fn outside_the_surface(root: &Path, units: &[Unit]) -> Vec<String> {
    let paired: BTreeSet<&str> = units
        .iter()
        .flat_map(|unit| [unit.gate.as_str(), unit.twin.as_str()])
        .collect();
    shell_units(root)
        .into_iter()
        .filter(|path| !paired.contains(path.as_str()))
        .collect()
}

#[test]
fn no_unit_outside_the_pairing_carries_the_gate_contract() {
    let Some(root) = workspace_root() else {
        return;
    };
    let units = measure(&root);

    // Detection, not a requirement on authored form: any mention in executed text, so a unit that reaches the
    // backstop by an unusual spelling is still seen. The exclusion from the surface is by *naming*, so this is
    // what stops it becoming a place a gate can hide — a `verify_*.sh` carrying the contract would otherwise
    // leave the surface by rename rather than by a spec change.
    let carries = |path: &str| read(&root, path).shell().contains(BACKSTOP);

    // The one exception is checked live, and BEFORE the loop it protects. Written after that loop first, and
    // the observation was that it never ran: pointing the exception at a unit carrying nothing made the real
    // library look like a gate in hiding, so the failure was real and the message was about the wrong thing.
    // An exception whose subject no longer matches it reads as licence, and must be retired rather than kept.
    assert!(
        carries(BACKSTOP_LIBRARY),
        "{BACKSTOP_LIBRARY} does not mention `{BACKSTOP}`, so the exception excusing it from the check below \
         describes nothing"
    );

    let hiding: Vec<String> = outside_the_surface(&root, &units)
        .into_iter()
        .filter(|path| path != BACKSTOP_LIBRARY && carries(path))
        .collect();
    assert!(
        hiding.is_empty(),
        "a unit outside the gate-and-twin pairing carries the gate contract, which is a gate wearing another \
         name — pair it with a twin and name it `check_*`, or argue the exclusion as a spec change: {hiding:?}"
    );
}

/// A throwaway repository holding one gate, its twin, and a Definition of Done that runs both — with exactly
/// one property withheld, named by its label.
///
/// The point of building it per property rather than once: a reaction that fails only in aggregate cannot be
/// trusted to have as many reasons as it has properties, and two of them were originally written against a shape that would have
/// made three real gates look non-conformant.
fn fixture_missing(withheld: Option<&str>) -> PathBuf {
    let missing = |label: &str| withheld == Some(label);
    let root = std::env::temp_dir().join(format!(
        "tianheng-gate-shape-{}-{}",
        withheld.unwrap_or("nothing").replace(' ', "-"),
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join(SCRIPTS)).expect("the fixture directory is writable");

    let contract = if missing("contract header") {
        "# Exit 0 clean, 1 violation.\n"
    } else {
        "# Exit 0 clean, 1 violation, 2 cannot judge — the family's own Core Contract.\n"
    };
    // Sourced and never invoked is the half that installs nothing, so that is the withholding.
    let source = "source \"$(dirname \"$0\")/lib/exit_contract.sh\"\n";
    let backstop = if missing("backstop") {
        source.to_string()
    } else if missing("backstop label") {
        // A sibling's name: the copy-paste shape the property exists for.
        format!("{source}exit_contract_backstop 'some other gate'\n")
    } else {
        format!("{source}exit_contract_backstop 'probe'\n")
    };
    // Withheld by consuming a fallible producer the way every measured recurrence did.
    let capture = if missing("one checked capture") {
        "while read -r line; do :; done < <(git ls-files)\n"
    } else {
        "capture=$(mktemp)\ngit ls-files >\"$capture\" || exit 2\nwhile read -r line; do :; done <\"$capture\"\n"
    };
    let target = if missing("target directory") {
        "repo=$(pwd)\n"
    } else {
        "repo=${1:-$(pwd)}\n"
    };
    std::fs::write(
        root.join(SCRIPTS).join(FIXTURE_GATE),
        format!(
            "#!/usr/bin/env bash\n#\n{contract}set -Eeuo pipefail\n{backstop}{target}{capture}printf 'probe ok\\n'\n"
        ),
    )
    .expect("the fixture gate is writable");

    if !missing("twin exists") {
        let mut twin = String::from("#!/usr/bin/env bash\nset -Eeuo pipefail\n");
        twin.push_str("expect_pass() { :; }\n");
        if !missing("both directions") {
            twin.push_str("expect_fail() { :; }\n");
        }
        if !missing("exit codes") {
            twin.push_str("assert_code() { local expected_status=$1; }\n");
        }
        if !missing("silent clean run") {
            twin.push_str("clean_stderr=$(\"$check\" \"$clean\" 2>&1 >/dev/null || true)\n");
            twin.push_str("[[ -z $clean_stderr ]] || exit 1\n");
        }
        if !missing("read-only") {
            twin.push_str(
                "[[ $before == \"$after\" ]] || { printf 'the gate mutated it\\n' >&2; exit 1; }\n",
            );
        }
        std::fs::write(root.join(SCRIPTS).join(FIXTURE_TWIN), twin)
            .expect("the fixture twin is writable");
    }

    let gate_line = if missing("definition of done") {
        String::new()
    } else {
        format!("bash {SCRIPTS}/{FIXTURE_GATE}\n")
    };
    std::fs::write(
        root.join("AGENTS.md"),
        format!(
            "# AGENTS\n\n## Definition of Done\n\n```bash\n{gate_line}bash {SCRIPTS}/{FIXTURE_TWIN}\n```\n"
        ),
    )
    .expect("the fixture AGENTS.md is writable");

    for arguments in [["init", "-q"], ["add", "-A"]] {
        let status = Command::new("git")
            .arg("-C")
            .arg(&root)
            .args(arguments)
            .status()
            .expect("git is available");
        assert!(status.success(), "the fixture repository is prepared");
    }
    root
}

#[test]
fn a_gate_missing_one_property_is_named_by_that_property() {
    // The conforming fixture first: a reaction that reported an offence here would make every case below agree
    // with it for the wrong reason.
    let clean = fixture_missing(None);
    let clean_offences = all_offences(&measure(&clean));
    let _ = std::fs::remove_dir_all(&clean);
    assert!(
        clean_offences.is_empty(),
        "the conforming fixture must hold every property, got: {clean_offences:?}"
    );

    for property in &PROPERTIES {
        let root = fixture_missing(Some(property.label));
        let found = all_offences(&measure(&root));
        let _ = std::fs::remove_dir_all(&root);

        // The fixture's withholding is expressed in whichever file the property is a property of, and the
        // membership one is withheld from the gate's side.
        let expected_file = match property.subject {
            Subject::Twin => format!("{SCRIPTS}/{FIXTURE_TWIN}"),
            Subject::Gate | Subject::BothFiles => format!("{SCRIPTS}/{FIXTURE_GATE}"),
        };
        // Matched through the separator, not on the prefix: `backstop` is a prefix of `backstop label`, and the
        // looser form counted the label's offence as the backstop's — found by this test failing rather than by
        // reading it.
        let named = found
            .iter()
            .filter(|offence| {
                offence.starts_with(&format!("{expected_file}: {} — ", property.label))
            })
            .count();
        assert_eq!(
            named, 1,
            "withholding `{}` must produce exactly that offence against {expected_file}, got: {found:?}",
            property.label
        );

        // Two properties carry dependents, and reporting them is honest rather than noisy — each names a real
        // absence. An absent twin cannot hold the four matrix properties; a gate that never invokes the backstop
        // passes no label. Suppressing either would need a third value of "held" meaning *not applicable*, which
        // is a claim about the file that neither yes nor no is making.
        let expected_total = match property.label {
            "twin exists" => 5,
            "backstop" => 2,
            _ => 1,
        };
        assert_eq!(
            found.len(),
            expected_total,
            "withholding `{}` must not disturb the other properties, got: {found:?}",
            property.label
        );
    }
}

#[test]
fn an_empty_surface_fails_rather_than_reporting_clean() {
    let Some(_root) = workspace_root() else {
        return;
    };
    // A repository with a `scripts/` directory, an `AGENTS.md` holding a Definition of Done, and no gate. Every
    // property of zero gates holds, so a reaction that did not refuse here would report the surface conformant.
    let fixture = std::env::temp_dir().join(format!(
        "tianheng-gate-shape-empty-surface-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&fixture);
    std::fs::create_dir_all(fixture.join("scripts")).expect("the fixture directory is writable");
    std::fs::write(
        fixture.join("AGENTS.md"),
        "# AGENTS\n\n## Definition of Done\n\n```bash\ncargo build --workspace\n```\n",
    )
    .expect("the fixture AGENTS.md is writable");
    std::fs::write(
        fixture.join(SCRIPTS).join("helper.sh"),
        "#!/usr/bin/env bash\n",
    )
    .expect("the fixture unit is writable");
    let init = Command::new("git")
        .arg("-C")
        .arg(&fixture)
        .args(["init", "-q"])
        .status()
        .expect("git is available");
    assert!(init.success(), "the fixture repository initialises");
    let add = Command::new("git")
        .arg("-C")
        .arg(&fixture)
        .args(["add", "-A"])
        .status()
        .expect("git is available");
    assert!(add.success(), "the fixture content is tracked");

    let units = measure(&fixture);
    let _ = std::fs::remove_dir_all(&fixture);
    assert!(
        units.is_empty(),
        "an empty surface yields zero measured bash units"
    );
}

#[test]
fn an_absent_layout_is_loud_when_the_workspace_marker_is_set() {
    let absent = std::env::temp_dir().join("tianheng-gate-shape-absent-layout");
    let _ = std::fs::remove_dir_all(&absent);
    // Outside a checkout with no marker: a skip, exactly as six crates already do.
    assert!(locate_layout(absent.clone(), false).is_none());
    // With the marker set, the same absence is a loud failure. Asserted through the pure function rather than
    // by setting the variable, because `set_var` is unsafe and would race every other test in this binary.
    assert!(
        std::panic::catch_unwind(|| locate_layout(absent, true)).is_err(),
        "an absent layout must fail loudly under TIANHENG_WORKSPACE_TESTS rather than skip"
    );
}

#[test]
fn the_gate_shape_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let units = measure(&root);
    let rendered = render(&units, &outside_the_surface(&root, &units));
    assert_projection_matches(&root, PROJECTION, &rendered);
}

/// The projection: the measured surface, its measured columns, and what conformance in it does not mean.
///
/// The columns are **printed**, never written into prose. A hand-maintained table of this shape is the drift
/// class this repository has closed twice, and the count in a sentence is the one a later reader trusts
/// without re-measuring.
fn render(units: &[Unit], excluded: &[String]) -> String {
    let mut out = String::new();
    out.push_str("# The gate shape contract\n\n");
    out.push_str(
        "Every `scripts/check_*.sh` gate in this repository and its companion failure matrix, with the\n\
         structural properties each holds. Enumerated from tracked content — a gate enters this table the\n\
         moment it is tracked, with no edit to any list.\n\n",
    );
    out.push_str(
        "Generated by `crates/tianheng/tests/gate_shape_contract.rs`. **Do not edit by hand** — regenerate\n\
         with `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test gate_shape_contract`.\n\n",
    );
    out.push_str("## What conformance in this table does not mean\n\n");
    // The membership and the figure are DERIVED; only the explanations are written. This paragraph used to type
    // both — "Three of the six classes … a fourth bound is about coverage" — and a bound added in the same window
    // went unlisted, because a literal in a generated document's template is compared with itself by the
    // freshness check. That is the one place a projection cannot self-correct.
    out.push_str(&format!(
        "Every column here is a property of a gate's **form**. This capability also declares bounds that are \
         **not** columns — most are semantic classes it was built for, one is about coverage — and it declares \
         {} of them. The list and its figure are read from `openspec/specs/gate-shape-contract/spec.md` rather \
         than restated, so neither can fall behind it:\n\n",
        BOUND_NOTES.len()
    ));
    for (heading, note) in BOUND_NOTES {
        out.push_str(&format!("- **{heading}**\n  {note}\n"));
    }
    out.push('\n');
    out.push_str(
        "And form conformance is not substance. `expect_pass` can sit in a comment; a target-directory\n\
         argument can be accepted and ignored. The reaction is aimed at an author who *forgets* the shape,\n\
         which is what every recurrence in the window that motivated it was.\n\n",
    );

    out.push_str("## The surface\n\n");
    out.push_str("| gate | twin |");
    for property in &PROPERTIES {
        out.push_str(&format!(" {} |", property.label));
    }
    out.push_str("\n| --- | --- |");
    for _ in &PROPERTIES {
        out.push_str(" --- |");
    }
    out.push('\n');
    for unit in units {
        let twin = match unit.twin_text {
            Some(_) => format!("`{}`", unit.twin),
            None => "**absent**".to_string(),
        };
        out.push_str(&format!("| `{}` | {twin} |", unit.gate));
        for property in &PROPERTIES {
            out.push_str(&format!(" {} |", (property.holds)(unit).cell()));
        }
        out.push('\n');
    }
    out.push_str(&format!(
        "\n{} gates, {} properties each.\n",
        units.len(),
        PROPERTIES.len()
    ));

    out.push_str("\n## Declared policy exemptions\n\n");
    out.push_str(
        "One, and it is checked live rather than honoured: `scripts/check_publish_source.sh` is excused from\n\
         Definition-of-Done membership because it runs from `scripts/publish.sh` at publish time — no\n\
         development checkout is a release snapshot, so a pre-flight run could only ever refuse. Its twin is\n\
         in the block. Were the gate ever added, the reaction fails and says the exemption is stale.\n\n",
    );
    out.push_str(
        "This is deliberately *not* an observation bound: a bound says a reaction stops at a shape, an\n\
         exemption says one named instance is excused from a requirement. Nothing enumerates exemptions today,\n\
         which is a recorded gap rather than a solved problem — the trigger for giving them their own register\n\
         is a second instance.\n\n",
    );

    out.push_str("## Outside the surface\n\n");
    out.push_str(
        "Shell units under `scripts/` that are neither a gate nor a twin. None may carry the shared exit\n\
         contract — a unit that does is a gate wearing another name, and the reaction refuses it:\n\n",
    );
    for unit in excluded {
        out.push_str(&format!("- `{unit}`\n"));
    }
    out.push_str(&format!("\n{} units.\n", excluded.len()));
    out
}

// --- this capability's own declared bounds, demonstrated ---
//
// Each is a bound-marked scenario in `gate-shape-contract`'s spec, carries a typed classification in
// `tianheng::observation_bounds()`, and is demonstrated here rather than asserted in prose. Every one shows
// the direction its extent predicts: the reaction does not react.

/// A gate holding all three gate properties whose loop has no guard against zero iterations, and a read whose
/// status the parent never inspects.
///
/// One fixture serves two bounds because one gate exhibits both shapes, which is how they reached this
/// repository — the same gate, in the same commit.
fn a_gate_with_both_semantic_defects() -> Source {
    Source::of(
        r#"#!/usr/bin/env bash
#
# A fixture gate.
#
# Exit 0 clean, 1 violation, 2 cannot judge — the family's own Core Contract.
set -Eeuo pipefail
source "$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/lib/exit_contract.sh"
exit_contract_backstop 'fixture'
repo=${1:-$(pwd)}
# An enumeration with no vacuity guard: zero matches reports clean having asserted nothing.
while read -r line; do
    printf 'considering %s\n' "$line"
done < <(grep -rn 'never matches anything' "$repo")
printf 'fixture ok\n'
"#,
    )
}

/// `gate-shape-contract/whether-an-enumeration-carries-a-vacuity-guard-is-not-observed-a-stated-bound`
///
/// `OutOfReach`: the reaction reads a gate's text for the three properties above and never models its control
/// flow, so a loop that iterates zero times and reports clean is a shape it does not look at.
#[test]
fn a_missing_vacuity_guard_is_a_stated_semantic_bound() {
    let gate = a_gate_with_both_semantic_defects();
    assert!(
        installs_the_backstop(&gate)
            && declares_the_three_way_contract(&gate)
            && accepts_a_target_directory(&gate),
        "the fixture must hold every property this reaction checks, or the bound is demonstrated by a gate \
         that fails for another reason"
    );
    // The defect is present and the reaction is silent about it. Demonstrated rather than stated: nothing in
    // the three properties above is a function of the guard.
    assert!(
        gate.whole().contains("no vacuity guard"),
        "the fixture must actually carry the defect this bound is about"
    );
}

/// `gate-shape-contract/whether-a-read-s-status-is-checked-in-the-parent-shell-is-not-observed-a-stated-bound`
///
/// `OutOfReach`, for the same reason and separately declared: the backstop the reaction *does* check narrows
/// the damage — an unhandled failure becomes cannot-judge instead of a foreign status — without detecting a
/// process substitution whose `grep` exit status the parent never reads.
#[test]
fn an_unchecked_read_status_is_a_stated_semantic_bound() {
    let gate = a_gate_with_both_semantic_defects();
    assert!(
        gate.whole().contains("done < <(grep"),
        "the fixture must read through a process substitution, or this bound is demonstrated by nothing"
    );
    assert!(
        installs_the_backstop(&gate),
        "the gate holds the property the reaction does check — which is the point: the backstop is present \
         and the unchecked status is still invisible"
    );
}

/// `gate-shape-contract/whether-a-gate-s-1-versus-2-assignment-is-correct-is-not-observed-a-stated-bound`
///
/// `UnderReacts`, owned by the engine — the one of the four that is a declared false negative. The reaction
/// *sees* the codes: it requires the twin to assert an expected status. It declines to judge whether the code
/// the gate assigned is the right one, which is exactly the judgment that let a `fail` returning instead of
/// exiting turn every violation into cannot-judge and ride green.
#[test]
fn a_wrong_one_versus_two_assignment_is_a_stated_semantic_bound() {
    let twin = Source::of(
        r#"#!/usr/bin/env bash
set -Eeuo pipefail
expect_pass() { :; }
expect_fail() {
    local repo=$1 expected_status=$2 output status=0
    output=$("$check" "$repo" 2>&1) || status=$?
    [[ $status -eq $expected_status ]] || exit 1
}
expect_pass "$clean"
# The wrong code, asserted as if it were right: the gate reports a genuine violation as cannot-judge.
expect_fail "$violating" 2 'a real violation'
clean_stderr=$("$check" "$clean" 2>&1 >/dev/null || true)
[[ -z $clean_stderr ]] || { printf 'noise\n' >&2; exit 1; }
[[ $before == "$after" ]] || { printf 'the gate mutated the repository\n' >&2; exit 1; }
"#,
    );
    assert!(
        asserts_exit_codes(&twin)
            && holds_both_directions(&twin)
            && asserts_read_only(&twin)
            && asserts_a_silent_clean_run(&twin),
        "the fixture twin must hold every matrix property, or the bound is demonstrated by a twin that fails \
         for another reason"
    );
    // Every column reads held while the asserted code is the wrong one. Nothing above is a function of which
    // code the gate should have chosen, and choosing it is the judgment this reaction declines.
    assert!(
        twin.whole().contains("expect_fail \"$violating\" 2"),
        "the fixture must assert the wrong code, or this bound is demonstrated by nothing"
    );
}

/// `gate-shape-contract/shell-units-that-are-not-a-gate-or-its-twin-are-outside-the-surface-a-stated-bound`
///
/// `OutOfReach`: the enumeration is `scripts/check_*.sh` and the twin its basename names. A sourced function
/// library, a matrix over one, the example runner and the publish tool are not judged — demonstrated on the
/// real tree, because a fixture could not show that live units sit outside it.
#[test]
fn units_outside_the_gate_pairing_are_outside_the_surface() {
    let Some(root) = workspace_root() else {
        return;
    };
    let units = measure(&root);
    let outside = outside_the_surface(&root, &units);
    assert!(
        !outside.is_empty(),
        "no unit sits outside the pairing, so this bound would be demonstrated by an empty set"
    );
    // Each is a real shell unit this reaction judges on none of its properties. The one thing asserted
    // about them is the contract-carrying refusal in `no_unit_outside_the_pairing_carries_the_gate_contract`,
    // which is what keeps the exclusion from being a hiding place rather than a claim of coverage.
    for path in &outside {
        assert!(
            !units
                .iter()
                .any(|unit| unit.gate == *path || unit.twin == *path),
            "{path} is both inside and outside the surface, which is not a state this reaction can report"
        );
    }
}
