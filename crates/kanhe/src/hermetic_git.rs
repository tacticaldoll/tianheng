//! The one command builder the git-reading gates and their fixtures run through.
//!
//! It lived twice, byte-identical, in `publish_source_gate` and `release_coherence_gate`, with a doc on only
//! one of them. The undocumented copy was then given a doc written without reading the other, and that doc
//! overclaimed — which is what two implementations of one thing cost even when the code cannot drift.

use std::path::Path;
use std::process::Command;

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
