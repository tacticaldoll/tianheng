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

/// The one setting that closes the ambient ignore channel, named once.
///
/// A constant rather than a literal at each site, and the reason is a direction rather than tidiness: a file
/// that *spells* this setting is read by `gate_exit_classes` as having closed the channel itself. The
/// direction that pins this builder's construction has to name the setting without claiming to neutralise
/// anything, and referring to it is how — which also gives the name one owner, as its own table's rule for a
/// declared set asks.
pub const EXCLUDES_SETTING: &str = "core.excludesFile";

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
/// | `GIT_CONFIG_COUNT` + `GIT_CONFIG_KEY_n` / `GIT_CONFIG_VALUE_n` | **yes** — by occupying index 0; this row read **no** until it was measured |
/// | `GIT_CONFIG_PARAMETERS` | **yes** — cleared, see `CONFIG_CHANNELS` below; a channel parallel to the count, which occupying index 0 does nothing to, and which git exports itself under any `git -c …` |
/// | `GIT_DIR` / `GIT_WORK_TREE` / `GIT_INDEX_FILE` | **yes** — cleared, see `REPOSITORY_SELECTORS` below; this row read **no** until a review asked why a stop nothing declared was policy |
/// | `GIT_AUTHOR_NAME` / `GIT_COMMITTER_NAME` and their emails | **no** — they override the fixture's own `.git/config` identity |
/// | `.git/info/exclude` | **no** — inside the repository, so no config setting reaches it |
///
/// **The ignore row is closed through the row above it, by taking that channel rather than by blocking it.**
/// Neutralising the config *files* does not neutralise `core.excludesFile`, because
/// `$XDG_CONFIG_HOME/git/ignore` is the default excludes path git uses when **no** config file names one — so
/// emptying the files leaves the default in force. The setting has to be *named*, and the only channel that
/// carries a setting without a config file is `GIT_CONFIG_COUNT`. **Occupying index `0` is what closes both
/// rows at once**: the setting reaches `git` from here, and the count this builder writes is what makes an
/// ambient key at any index unreachable. This paragraph said that channel *cannot be closed* until the row
/// above was measured; the channel is used, not open.
///
/// **Measured, on a fixture whose only exclusion came from an XDG ignore file:** `git add -A` with the three
/// file variables set and nothing else left the matching file *untracked* — a fixture silently built without
/// a file it named — and adds it once `core.excludesFile` is named. `git check-ignore` answers *ignored* and
/// stops. Both directions, and the first is why this moved into the builder rather than staying a flag each
/// judgement remembers: the reads were being fixed one at a time and every fixture construction was exposed.
///
/// **The `GIT_CONFIG_*` row is closed, and it read open until someone measured it.** The claim was that any
/// ambient key reaches `git`, `commit.gpgsign=true` included. It does not: [`Command::env`] overrides
/// `GIT_CONFIG_COUNT` to `1`, `git` then reads index `0` only, and this builder owns index `0` — so an
/// ambient key at any index is unreachable and an ambient key at index `0` is overwritten. Measured, with
/// `GIT_CONFIG_COUNT=2` and `GIT_CONFIG_KEY_1=user.name` in the environment: under this builder
/// `git config --get user.name` exits `1` with no output, and the same pair without it answers the ambient
/// value. A row saying **no** where the answer is **yes** is not a conservative error — it reads as governed
/// policy and would send the next fixture author to build isolation they already have.
///
/// **`GIT_DIR` and its siblings were the row this table did not have, and then the row it got wrong.** They
/// are not an ignore channel; they move which repository `git` acts on, so they reach past `current_dir(dir)`
/// entirely. The row stood at **no** on the ground that nothing in this tree sets them — zero occurrences,
/// repository-wide — which is a corpus that cannot decide it: the channel is *ambient*, so the variable
/// arrives from outside the tree the sweep read, and a review named the same defect class this crate spends
/// four rules closing. The stop was also undeclared — no `openspec/specs/*` scenario carried it, so it
/// appeared in neither observation register — and an undeclared stop is a defect rather than governed policy
/// by this repository's own reading rule. It is closed instead of declared, because closing it costs an
/// `env_remove` and refuses no caller.
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
        .env("GIT_CONFIG_KEY_0", EXCLUDES_SETTING)
        .env("GIT_CONFIG_VALUE_0", "/dev/null");
    for selector in REPOSITORY_SELECTORS {
        command.env_remove(selector);
    }
    for channel in CONFIG_CHANNELS {
        command.env_remove(channel);
    }
    command
}

/// The environment variables that inject **configuration** past the files this builder empties.
///
/// `GIT_CONFIG_COUNT` and its indexed pairs are closed by **occupation**: this builder writes the count and
/// index 0, so an ambient pair at any index is unreachable. `GIT_CONFIG_PARAMETERS` is a second, parallel
/// channel git parses independently of that count, so occupying index 0 does nothing to it.
///
/// **And git itself exports it**, which is what makes this ambient rather than hypothetical. Measured on git
/// 2.53.0: a `pre-commit` hook under `git -c probe.key=SET commit` sees
/// `GIT_CONFIG_PARAMETERS=['probe.key'='SET']`, and sees it unset without the `-c`. So a gate run from a
/// hook, an alias, or `bisect run` inherits whatever configuration that invocation set.
///
/// **The direction is under-refusal**, measured against this builder's full environment:
/// `'core.excludesFile=/tmp/x'` makes `config --get core.excludesFile` answer `/tmp/x`, and
/// `status --porcelain --untracked-files=all` stops reporting a file that path excludes. That read is the one
/// `publish-source-integrity#worktree-is-not-clean` rests on, in front of an upload that can be yanked and
/// never replaced.
///
/// **This list is not the defence; the case is.** Three rounds of widening this builder each added the
/// variable someone had just measured — the config files, then the repository selectors, then the object
/// pair — and a list grown that way is exactly as complete as the last person's memory.
/// `no_ambient_configuration_reaches_a_hermetic_command` asks the question the other way round: it runs a
/// child under an ambient environment and requires that the configuration git reports come from this
/// builder and nowhere else, so a channel nobody has named is found by the run rather than by the next
/// review.
/// `GIT_CONFIG` is here for a different reason and it is worth the sentence. It does **not** move a
/// judgement's reads — measured, `status --porcelain --untracked-files=all` reports an excluded file with and
/// without it, because that command does not consult it. What it moves is a **write**: under it,
/// `git config user.name t` lands in the file the variable names instead of the fixture's own `.git/config`,
/// so `fixture` and `build_fixture` would build a repository with no identity and the commit after them
/// fails. That is fail-loud, like the object-directory pair, and it is cleared for the reason the selector
/// row gives: an `env_remove` costs nothing and refuses no caller in this workspace.
const CONFIG_CHANNELS: [&str; 2] = ["GIT_CONFIG_PARAMETERS", "GIT_CONFIG"];

/// The environment variables that move **which repository** `git` answers about, cleared rather than set.
///
/// Named rather than linked from [`hermetic`]'s table above, for the reason `region.rs` already states for
/// its own private `Rule`: this constant is private and that item is public, so an intra-doc link resolves
/// only under `--document-private-items` and `-D rustdoc::private-intra-doc-links` refuses it. Measured by
/// CI rather than reasoned about — the first form of that row was a link and failed the doc job.
///
/// Cleared because there is no value that means *the one `current_dir` names* — git's own default is their
/// absence, so removing them restores discovery from the working directory, which is the property every caller
/// here already believes it has.
///
/// **The set is what measurement admits, not every `GIT_*` git defines.** Measured on this machine against two
/// repositories whose `HEAD` subjects and tags differ:
///
/// | variable | effect on a judgement's reads | in this set |
/// |---|---|---|
/// | `GIT_DIR` | `log -1 --format=%s` and `for-each-ref refs/tags` both answer the **other** repository | yes |
/// | `GIT_WORK_TREE` | `status --porcelain` reports the other tree's differences against this index | yes |
/// | `GIT_INDEX_FILE` | replaces the index `status` and `ls-files` compare against | yes |
/// | `GIT_NAMESPACE` | `for-each-ref refs/tags` still answered this repository's tags — **no effect** | no |
///
/// The `GIT_NAMESPACE` row is why this is a measured set rather than a swept prefix: a list built by reading
/// git's manual would carry it, and an entry that closes nothing reads as a defence that was never there —
/// which this workspace's own manifest already argues about inert `exclude` entries.
///
/// **The object-directory pair is measured too, and neither moves an answer.** They were first measured only
/// against `rev-parse HEAD`, which reads refs — so the read that matters, the tag body the signature check
/// reconstructs, was left unmeasured under a note saying it was filed. Nothing was filed, which left a private
/// doc comment as the only carrier of a stop; the row above records what that costs. Measured since, against
/// `for-each-ref --format=%(contents) refs/tags/<tag>` over two repositories whose tag bodies differ:
///
/// | variable | effect on the tag-object read | admitted |
/// |---|---|---|
/// | `GIT_OBJECT_DIRECTORY` | **replaces** the store, so this repository's own tag object goes missing — exit 128, `fatal: missing object … for refs/tags/…` | no: it refuses rather than answering wrongly |
/// | `GIT_ALTERNATE_OBJECT_DIRECTORIES` | **appends** a store, so the local object still answers — exit 0, this repository's own tag body | no: it moves nothing |
///
/// The reasons differ and both are stated, because *not admitted* covers a fail-closed variable and an inert
/// one alike while only the second is harmless in general. Reading a **different** body through either would
/// need the ref to resolve to an object id whose content differs, and the refs come from this repository
/// because `GIT_DIR` is cleared — so it needs a collision, not a variable.
const REPOSITORY_SELECTORS: [&str; 3] = ["GIT_DIR", "GIT_WORK_TREE", "GIT_INDEX_FILE"];

/// One read of `git` in `repo` through [`hermetic`], with the output/success/failure mapping every gate's
/// own `git()` wrapper otherwise has to restate — and the body [`run`] is this one plus a trim.
///
/// **The extraction history that used to open this doc is [`run`]'s**, and is now written there. That is
/// what an annexed doc looks like: a passage describing one item, attached to its neighbour, where both
/// read plausibly enough that nobody re-attributed it.
///
/// This accessor is the one **without** the trailing-whitespace trim, for a caller comparing **content**
/// rather than reading a value. Most callers read one line — a sha, a ref, a status — and the trim is what
/// makes those comparable. A caller comparing a committed file against a working one needs the bytes git
/// gave: trimming both sides makes a worktree edited only in its trailing whitespace read as unmodified,
/// which is a different tree reported as the same one.
pub fn run_exact(repo: &Path, flags: &[&str], args: &[&str]) -> Result<String, Failure> {
    let out = hermetic("git")
        .args(flags)
        .args(args)
        .current_dir(repo)
        .output()
        .map_err(|err| Failure::Spawn(format!("cannot run git {args:?}: {err}")))?;
    if out.status.success() {
        // **Refused, never mangled.** `from_utf8_lossy` replaces each undecodable byte with U+FFFD, and what
        // this runner mostly carries is **paths**: `ls-files -z` avoids git's own quoting and promises
        // nothing about encoding, so a tracked path that is not UTF-8 arrived here as a different path than
        // the one on disk, and every comparison downstream was made against that. `xingbiao::path_identity`
        // exists for the opposite property — two paths differing only in undecodable bytes keep two
        // identities — so a reader in this repository that silently collapses them contradicts the product's
        // own rule. A verdict is not owed on an input this reader cannot represent; saying so is.
        //
        // stderr below stays lossy, deliberately: it is a sentence for an operator, not a value anything
        // compares.
        String::from_utf8(out.stdout).map_err(|err| {
            Failure::Unreadable(format!(
                "git {args:?} answered bytes this reader cannot represent as text — {err}; a path that is \
                 not UTF-8 keeps its own identity, and reporting a replaced one would compare something the \
                 repository does not hold"
            ))
        })
    } else {
        Err(Failure::Exit {
            code: out.status.code(),
            stderr: String::from_utf8_lossy(&out.stderr).trim_end().to_string(),
        })
    }
}

/// [`run_exact`] with git's trailing whitespace trimmed off, which is what a caller reading a **value**
/// wants: a sha, a ref name, a status line.
///
/// It lived twice here too, byte-identical past the leading flags, in the same two files this module's own
/// doc comment already names for [`hermetic`]. `flags` are spliced in before `args` — `&[]` for
/// `release_coherence_gate`, `&["-c", "core.excludesFile=/dev/null"]` for `publish_source_gate`, which stated
/// per command what [`hermetic`] now states for every caller. The flag is kept there rather than dropped: it
/// is the narrower statement, it costs nothing, and the measurement that earned it is recorded beside it.
///
/// **And then it lived twice again, inside the module that exists to end exactly that.** This body was the
/// eight statements of [`run_exact`] with one `.map` inserted, and the copy had already drifted where
/// nothing was watching: the two `Failure::Unreadable` sentences differed, so one fact reached an operator
/// two ways depending on which accessor a caller reached for. The trim is the whole difference and is now
/// the whole body;
/// `both_accessors_report_an_undecodable_answer_in_the_same_words` holds that the two cannot part again.
pub fn run(repo: &Path, flags: &[&str], args: &[&str]) -> Result<String, Failure> {
    run_exact(repo, flags, args).map(|text| text.trim_end().to_string())
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
    /// git ran, succeeded, and answered bytes this reader cannot represent as text.
    ///
    /// Separate from [`Failure::Exit`] because git did not fail: the command answered, and the answer is one
    /// no `String` holds without changing it. Folding the two would report a working repository as a broken
    /// command, and folding this into success would compare a path against a replaced copy of itself.
    Unreadable(String),
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
            Failure::Spawn(why) | Failure::Unreadable(why) => write!(f, "{why}"),
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
