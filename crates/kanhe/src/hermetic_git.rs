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

/// The instant every fixture commit is made at, so a fixture's dates are the fixture's rather than the
/// clock's.
///
/// UTC midnight, and that is the load-bearing half: `--date=short` renders in the commit's own timezone, so
/// this reads as [`FIXTURE_DAY`] on every machine — where a local-midnight stamp would read as the day
/// before anywhere west of UTC.
pub const FIXTURE_DATE: &str = concat!("2026-07-20", "T00:00:00+00:00");

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
/// | `GIT_CONFIG_COUNT` + `GIT_CONFIG_KEY_n` / `GIT_CONFIG_VALUE_n` | **no** — any key reaches `git`, `commit.gpgsign=true` included |
/// | `GIT_AUTHOR_NAME` / `GIT_COMMITTER_NAME` and their emails | **no** — they override the fixture's own `.git/config` identity |
/// | `$XDG_CONFIG_HOME/git/ignore` | **no** — see [`crate::publish_source_gate`]'s table |
/// | `.git/info/exclude` | **no** — inside the repository, so no config setting reaches it |
///
/// The last two rows are why the publish gate passes `-c core.excludesFile=/dev/null` on top of this rather
/// than routing through the builder and stopping there.
pub fn hermetic(program: &str) -> Command {
    let mut command = Command::new(program);
    command
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_SYSTEM", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1");
    command
}

/// One read of `git` in `repo` through [`hermetic`], with the output/success/failure mapping every gate's
/// own `git()` wrapper otherwise has to restate.
///
/// It lived twice here too, byte-identical past the leading flags, in the same two files this module's own
/// doc comment already names for [`hermetic`]. `flags` are spliced in before `args` — `&[]` for
/// `release_coherence_gate`, `&["-c", "core.excludesFile=/dev/null"]` for `publish_source_gate`, which must
/// also close the last ambient row this module's doc table leaves open.
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
/// The explicit signature is the one kept. `args[0]` makes the program a value the caller has to get right
/// inside a list, and there is no shape of that list a type refuses.
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
    let out = hermetic(program)
        .env("GIT_AUTHOR_DATE", FIXTURE_DATE)
        .env("GIT_COMMITTER_DATE", FIXTURE_DATE)
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
