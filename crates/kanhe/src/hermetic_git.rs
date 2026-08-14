//! The one command builder the git-reading gates and their fixtures run through.
//!
//! It lived twice, byte-identical, in `publish_source_gate` and `release_coherence_gate`, with a doc on only
//! one of them. The undocumented copy was then given a doc written without reading the other, and that doc
//! overclaimed — which is what two implementations of one thing cost even when the code cannot drift.

use std::process::Command;

/// A command that reads neither the **global** nor the **system** git config file.
///
/// Measured rather than assumed: without this the fixture inherited this repository's own signing
/// configuration, so `git tag -a` produced a genuinely signed tag where the fixture wanted an unsigned one,
/// and a bare `git tag` demanded a message. A fixture that inherits the judged machine cannot demonstrate a
/// refusal, because the shape it builds is not the shape it named.
///
/// **It does not make a command read no ambient configuration**, and a new fixture deciding how much
/// isolation it needs should read this list rather than the sentence above it. Measured with exactly the
/// environment this function sets:
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
