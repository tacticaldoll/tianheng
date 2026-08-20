//! The one command builder the git-reading gates and their fixtures run through.
//!
//! It lived twice, byte-identical, in `publish_source_gate` and `release_coherence_gate`, with a doc on only
//! one of them. The undocumented copy was then given a doc written without reading the other, and that doc
//! overclaimed — which is what two implementations of one thing cost even when the code cannot drift.

use std::path::Path;
use std::process::Command;

/// The day every fixture's dates are on.
///
/// **One owner, because two things have to agree.** A fixture's commits carry this date and the changelog a
/// fixture writes dates its release section with it — and `release-coherence` now compares those two. The
/// first extraction took the constant from the half that needed it (the commit) and left the other half
/// (the section) a literal in the generator and in four directions, which is one fact with an enumerator
/// available and unused.
pub const FIXTURE_DAY: &str = "2026-07-20";

/// A command that reads neither the **global** nor the **system** git config file.
///
/// Measured rather than assumed: without this the fixture inherited this repository's own signing
/// configuration, so `git tag -a` produced a genuinely signed tag where the fixture wanted an unsigned one,
/// and a bare `git tag` demanded a message. A fixture that inherits the judged machine cannot demonstrate a
/// refusal, because the shape it builds is not the shape it named.
///
/// **It does not make a command read no ambient configuration.** A new fixture deciding how much isolation it
/// needs should go by this measurement, taken with exactly the environment this function sets:
///
/// | ambient source | closed here |
/// |---|---|
/// | global / system config file | yes |
/// | `$XDG_CONFIG_HOME/git/ignore` | yes — see below; this row read **no** until a gate was found relying on it |
/// | `GIT_CONFIG_COUNT` + `GIT_CONFIG_KEY_n` / `GIT_CONFIG_VALUE_n` | **no** — any key reaches `git`, `commit.gpgsign=true` included, and index `0` here is taken |
/// | `GIT_AUTHOR_NAME` / `GIT_COMMITTER_NAME` and their emails | **no** — they override the fixture's own `.git/config` identity |
/// | `.git/info/exclude` | **no** — inside the repository, so no config setting reaches it |
///
/// **The ignore row is closed through the row above it, which is the one that cannot be closed.** Neutralising
/// the config *files* does not neutralise `core.excludesFile`, because
/// `$XDG_CONFIG_HOME/git/ignore` is the default excludes path git uses when **no** config file names one — so
/// emptying the files leaves the default in force. The setting has to be *named*, and the only channel that
/// carries a setting without a config file is `GIT_CONFIG_COUNT`, the row this table records as open. What
/// makes it open to a caller is what makes it usable here.
///
/// **Measured, on a fixture whose only exclusion came from an XDG ignore file:** `git add -A` with the three
/// file variables set and nothing else left the matching file *untracked* — a fixture silently built without
/// a file it named — and adds it once `core.excludesFile` is named. `git check-ignore` answers *ignored* and
/// stops. Both directions, and the first is why this moved into the builder rather than staying a flag each
/// judgement remembers: the reads were being fixed one at a time and every fixture construction was exposed.
///
/// A caller needing its own key starts at `_1` and sets `GIT_CONFIG_COUNT` to `2`; overwriting the count
/// without carrying index `0` forward reopens the ignore row. No caller in this workspace sets these
/// variables — this function is the only writer of them (measured) — so the collision is stated rather than
/// guarded.
pub fn hermetic(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "core.excludesFile")
        .env("GIT_CONFIG_VALUE_0", "/dev/null");
    command
}

/// One read of `git` in `repo` through [`hermetic`], with the output/success/failure mapping every gate's
/// own `git()` wrapper otherwise has to restate.
///
/// It lived twice here too, byte-identical past the leading flags, in the same two files this module's own
/// doc comment already names for [`hermetic`]. `flags` are spliced in before `args` — `&[]` for
/// `release_coherence_gate`, `&["-c", "core.excludesFile=/dev/null"]` for `publish_source_gate`, which stated
/// per command what [`hermetic`] now states for every caller. The flag is kept there rather than dropped: it
/// is the narrower statement, it costs nothing, and the measurement that earned it is recorded beside it.
pub fn run(repo: &Path, flags: &[&str], args: &[&str]) -> Result<String, Failure> {
    let out = hermetic("git")
        .args(flags)
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|err| Failure::Spawn(format!("cannot run git {args:?}: {err}")))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
    } else {
        Err(Failure::Exit {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).trim_end().to_string(),
        })
    }
}

/// Run `program` in `dir` through [`hermetic`] and assert it succeeded — the fixture side of this module.
///
/// **It lived twice too, and the extraction that took [`hermetic`] and [`run`] walked past it.** The two
/// copies sat in the same pair of files this module's header already names, differing only in how the
/// program was passed: one took it as its own argument, the other read `args[0]` and sliced the rest. The
/// second spelling also panicked on an empty slice where the first could not, so the twin had begun to
/// diverge in the way `manifest`'s header describes for its own pair.
///
/// The explicit signature is the shape to reach for, and it is what this function takes. `args[0]` makes the
/// program a value the caller has to get right inside a list, and there is no shape of that list a type
/// refuses. It read *the one kept **here***, which was the correct narrowing while three other runners still
/// composed the program into a list; two of those have since been converged, so the qualifier understated it.
///
/// **It is not the only shape admitted, which this sentence used to claim.** Four runner bodies in this crate
/// composed the program into the list — `bound_register_parse::search`, `bound_register_parse::must`,
/// `gate_identity::run` and `pin_bites::run` — and **two** of them cannot do otherwise: `pin_bites` chooses
/// `cargo` for a mutation build and `git` for a record read through one runner, and `gate_identity` chooses
/// `git` to enumerate and `cargo` to list a target's tests. The other two never chose at run time — every one
/// of their call sites named a literal — so they were given this signature, and the list form now survives
/// only where the rule admits it, unpacked in exactly one place, [`program_and_args`], which also turns the
/// empty-slice panic this paragraph names into a stated one. Every fixture that knows its program still
/// passes it here.
///
/// **The enumeration rather than the counts, which were written from inside the repair.** This paragraph
/// first said *three* runners and *one* that cannot; both were measured over the set the author had just
/// edited rather than over the base commit, and both were wrong. Nothing reacts to a count in a Rust doc
/// comment — the shape [`crate::refusal::Site`] describes for its own drifted figure.
///
/// # Panics
///
/// When the process cannot be started, or exits non-zero. This builds a fixture rather than judging one, so
/// a failure here is the harness being unable to construct its own subject.
pub fn fixture(dir: &Path, program: &str, args: &[&str]) {
    // **A fixture's commits carry a fixed date**, so a direction can assert what a date is rather than only
    // what shape it has. `release_coherence` writes its dated release section as a literal and now holds it
    // against the `release: X.Y.Z` commit's own date; with the date taken from the clock those two agree
    // only until midnight, and the fixture would be asserting the machine rather than the subject.
    //
    // Both variables, because git takes the author date from one and the committer date from the other.
    // Nothing reads `%cd` today — the release spine reads `%ad` — so the committer date is set for symmetry
    // rather than for a consumer, and saying which it is keeps the next reader from looking for the one
    // that does not exist. It matters under `--amend`, which preserves the author date and rewrites the
    // committer date to the clock: a fixture routed around this builder would carry one of each.
    // UTC midnight, and that is the load-bearing half: `--date=short` renders in the commit's own timezone,
    // so this reads as [`FIXTURE_DAY`] on every machine — where a local-midnight stamp would read as the day
    // before anywhere west of UTC.
    //
    // Built here rather than kept as a second constant. A `const` would have to be a `&'static str`, so it
    // could only be spelled as a literal — `concat!` takes literals and not a constant's name, measured with
    // rustc — and a second literal under a `concat!` reads as a derivation from the first while being a
    // second place the day is written. `Command::env` takes anything that is `AsRef<OsStr>`, so the value
    // this needs is a `String` and the const was never required.
    let stamp = format!("{FIXTURE_DAY}T00:00:00+00:00");
    let out = hermetic(program)
        .env("GIT_AUTHOR_DATE", &stamp)
        .env("GIT_COMMITTER_DATE", &stamp)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {program} {args:?}: {err}"));
    assert!(
        out.status.success(),
        "{program} {args:?} failed: {}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

/// Stage everything in a fixture and commit it under one subject.
///
/// **The third instance of this module's own class, and converging [`fixture`] is what exposed it.** With the
/// command builder shared, both fixture builders were left spelling `git add .` and then a commit — while
/// `release_coherence_gate` had already written exactly this helper for itself and `publish_source_gate` had
/// not. One module holding the extraction and its sibling not is the same shape the two earlier extractions
/// left behind, one layer down, and it was invisible until the layer above it closed.
///
/// # Panics
///
/// As [`fixture`] does: this builds a subject rather than judging one.
pub fn commit(repo: &Path, subject: &str) {
    fixture(repo, "git", &["add", "."]);
    fixture(repo, "git", &["commit", "-qm", subject]);
}

/// Why a `git` read produced no output.
///
/// **Two facts, folded into one `Err(String)` until a review named the cost.** *git could not be run at all*
/// and *git ran and refused* read identically to a caller, and they are not the same fact for an operator:
/// the first means the tool is absent or the directory cannot be entered, the second means the repository
/// answered. With them folded, a machine without git reached `cargo publish`'s gate and was told
/// `repository root X is not a git worktree` — a sentence about the repository, for a fact about the machine.
///
/// [`Display`](std::fmt::Display) renders the cause, so a caller that only wants to say what went wrong is
/// unchanged by the split; a caller that wants to tell the two apart now can.
#[derive(Debug)]
pub enum Failure {
    /// The process could not be started: git is absent, or `repo` is not a directory this process can enter.
    Spawn(String),
    /// git ran and exited non-zero. Carries its status and its stderr.
    ///
    /// **The status, because non-zero is not one fact.** A git subcommand answers some questions *with* an
    /// exit status — `ls-files --error-unmatch` exits `1` for *this path is not tracked*, which is the
    /// answer — and reserves the rest for declining to read the repository at all. Measured on this
    /// machine's git: `1` for an absent path, `128` both for a directory that is no repository and for an
    /// index that cannot be parsed. With the status dropped, a caller could only ask *did git succeed*, and
    /// the publish gate answered *this repository does not track it* for a repository it had never read.
    ///
    /// `None` where a signal ended the process, which is no answer either.
    Exit {
        /// The exit status, where the process exited rather than being signalled.
        code: Option<i32>,
        /// What git wrote to stderr.
        stderr: String,
    },
}

impl std::fmt::Display for Failure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Failure::Spawn(why) => write!(f, "{why}"),
            Failure::Exit { stderr, .. } => write!(f, "{stderr}"),
        }
    }
}

/// What a caller is told when a read it required did not happen.
///
/// **One owner, because this sentence stood at four sites.** *A failed read is not an empty result* was
/// written verbatim in `bound_register_parse::{search, must}`, in `pin_bites`'s own `must`, and in
/// `reference_integrity` — four copies of the one rule the Core Contract turns on, in three files, already
/// diverged in what they printed beside it. The rule is that reporting a failed read as an empty one reports
/// a verdict over content that was never read, which is the vacuity direction the contract forbids; a
/// sentence that says so belongs where the readers that say it live.
pub fn failed(what: &str, status: &str, output: &str) -> String {
    format!("{what} failed ({status}); a failed read is not an empty result: {output}")
}

/// The program a caller composed into its argument list, split from the rest.
///
/// **The second admitted shape, stated rather than left to a reader to notice.** [`fixture`] keeps the
/// program as its own parameter and its doc says why: a list gives the caller a position to get wrong and no
/// type refuses the wrong one. That is the shape to reach for — and **two** callers genuinely cannot, because
/// each composes its program at run time: `pin_bites::run` builds `["cargo", "test", …]` for a mutation build
/// and `["git", "show", …]` for a record read through one runner, and `gate_identity::run` builds
/// `["git", "ls-files", …]` to enumerate and `["cargo", "test", …, "--list"]` to list a target's tests. Those
/// two are the whole of it: no other caller in this crate reaches this function. So both shapes are admitted,
/// this is where the list form is unpacked, and the empty case is a stated panic instead of the unstated
/// index it was.
pub fn program_and_args<'a>(what: &str, args: &'a [&'a str]) -> (&'a str, &'a [&'a str]) {
    let (program, rest) = args
        .split_first()
        .unwrap_or_else(|| panic!("{what}: an empty argument list names no program to run"));
    (program, rest)
}

/// One read of `program` in `dir` through [`hermetic`], requiring success, returning its stdout.
///
/// A failed read is not an empty result — see [`failed`] — so this asserts rather than returning a status a
/// caller might drop. Through [`hermetic`], which the copies this replaces were not: they read the global and
/// system git config every read *this module* owns closes off.
///
/// **Not *the rest of this crate's git*, which this sentence used to claim.** Bare `Command::new("git")`
/// survives across this crate's test targets, and `gate_exit_classes` enumerates every one of them. What is
/// held rather than asserted is narrower and is the half that can move a verdict: no judgement runs a
/// subcommand an ambient ignore file answers differently without neutralising it — see
/// `no_judgement_reads_an_ambient_ignore_file`.
pub fn read(dir: &Path, what: &str, program: &str, args: &[&str]) -> String {
    let out = hermetic(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {what}: {err}"));
    assert!(
        out.status.success(),
        "{}",
        failed(
            what,
            &out.status.to_string(),
            &format!(
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            )
        )
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

/// A search whose *ordinary* no-match answer is a non-zero status, returning the matching lines.
///
/// `grep` exits 1 on a clean miss. Treating that as a failure was found the hard way in this repository's
/// shell era: a producer's contract has to be named per call site rather than inferred, because the
/// alternative — reading every non-zero as empty — turns a failed read into a clean verdict, which is the
/// one direction the Core Contract forbids. So exit 1 is *no match* and anything else is a failure.
pub fn search(dir: &Path, what: &str, program: &str, args: &[&str]) -> Vec<String> {
    let out = hermetic(program)
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|err| panic!("cannot run {what}: {err}"));
    match out.status.code() {
        Some(0) => String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
        Some(1) => Vec::new(),
        other => panic!(
            "{}",
            failed(
                what,
                &format!("exit {other:?}"),
                &String::from_utf8_lossy(&out.stderr)
            )
        ),
    }
}
