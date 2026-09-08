//! How this repository's checks spell a path below a root — one rule, one implementation.
//!
//! Every gate here that compares its own paths against git's, or against another reader's, has to turn an
//! absolute path cargo reported into the repository-relative identity git and this repository's prose both
//! use. `release_coherence_gate::machinery_names`, `release_coherence_gate::workspace_manifests` and
//! `member_enumeration`'s comparison all ask it, and they ask it here.
//!
//! **Two of them are the two sides of one comparison**, which is what makes a shared spelling load-bearing
//! rather than tidy: `member_enumeration` holds cargo's member set against the gate's walk as **strings**, so
//! a difference in how either renders a separator is a difference in every member. Written out per site, the
//! two disagreed — one joined components with `/` and the other used `Path::display`, the host's own
//! separator — and on a host where that separator is not `/` the two sets share no member at all. That state
//! is closed; the property it falsifies is why the derivation has one owner.
//!
//! **The separator is not a character here.** Cargo reports native paths, so stripping a `"{root}/"` string
//! rather than a prefix of components leaves every member outside the prefix wherever that separator is not
//! `/` — the defect `release_coherence_gate::machinery_names` records having already met. It is private,
//! so this is the prose form rather than the link form: a reference rustdoc cannot resolve is what
//! `reference-integrity` declares a bound for, and a link to a private item is one. The
//! result is joined back with `/` deliberately: it is compared against git's paths and cited in this
//! repository's own prose, both of which spell a separator that way whatever the host does.

use std::path::Path;

/// Where a path sits relative to a root, or why this reader cannot say.
///
/// Typed apart rather than an `Option`, so a consumer answers each fact rather than choosing a reading for
/// a missing value, and so *not under the root* stays apart from *not spellable at all* — two facts an
/// operator repairs in opposite directions.
///
/// The readings were live: one consumer refused a `None` and another took it as licence to carry the
/// absolute path forward, into a set of repository-relative paths.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryPath {
    /// The path below the root, spelled with `/` whatever separator the host uses.
    Below(String),
    /// The path does not sit under the root it was resolved against.
    Outside,
    /// A component below the root is not UTF-8, so this reader cannot spell the identity the repository
    /// holds. Carries the component as this reader sees it, for the message only.
    NotUtf8(String),
}

/// What the *directory of* a path is, relative to a root — [`RepositoryPath`] plus the one answer only this
/// question has.
///
/// **It wraps rather than repeats, and the compiler is why.** Carrying the extra answer as a fourth
/// `RepositoryPath` variant made the plain path question match a state it can never produce, and the only
/// arms available there are a fold or an `unreachable!` — one is the defect being repaired and the other is
/// what `unreachable_branch` refuses. Restating the three variants in a second enum would be two lists that
/// must agree. Composing costs one `match` at the one site that asks this question, and nothing anywhere
/// else.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectoryOf {
    /// The directory, answered by exactly the rule any other path is answered by.
    Directory(RepositoryPath),
    /// The path has no parent directory, so there is no directory to spell.
    ///
    /// **It exists because the caller asking this question folded it.** `Path::parent` is an `Option`, and
    /// the one site reading it took `None` through an `else` arm whose refusal says the manifest is *not
    /// under the workspace root* — a claim that is false for a path with no parent, and exactly the reading
    /// [`RepositoryPath`]'s variants were typed apart to stop, arriving at the `Path::parent` call rather
    /// than at the strip that typing reached.
    HasNoDirectory,
}

/// `path` spelled relative to `root`, the way git spells one.
///
/// The comparison is component-wise, so a prefix matches on its own boundaries rather than on a separator
/// byte, and the answer is rebuilt with `/` so it can be handed to `git` and compared against what git
/// answers.
///
/// **A component that is not UTF-8 is refused, never replaced** — the policy `hermetic_git` states for the
/// other end of the same comparison: *a path that is not UTF-8 keeps its own identity, and reporting a
/// replaced one would compare something the repository does not hold*. A lossy decode substitutes U+FFFD
/// per undecodable byte, so the answer names nothing git holds, and two distinct names collapse onto one
/// spelling — the collision `xingbiao::path_identity` exists to keep out of a walk.
///
/// Reaching it needs a `path` whose components come from the filesystem: a caller handing over a `&str` it
/// read out of cargo's JSON has already been given UTF-8 by the parser, and no arm of this can fire for it.
pub fn repository_path(root: &Path, path: &Path) -> RepositoryPath {
    let Ok(below) = path.strip_prefix(root) else {
        return RepositoryPath::Outside;
    };
    let mut spelled = Vec::new();
    for part in below.components() {
        let Some(text) = part.as_os_str().to_str() else {
            return RepositoryPath::NotUtf8(part.as_os_str().to_string_lossy().into_owned());
        };
        spelled.push(text.to_string());
    }
    RepositoryPath::Below(spelled.join("/"))
}

/// The directory `path` sits in, spelled relative to `root` — the *directory of* question, asked here.
///
/// **The `Option` lives here rather than in front of a caller.** `Path::parent` answers `None` for a path
/// with no parent, and a caller reaching for it has to decide what that means while it is holding a
/// different question; the one site that did read it through an `else` arm whose refusal named *not under
/// the workspace root*, which is false for such a path. That is the same fold this module's enum was typed
/// apart to stop, arriving at the `Path::parent` call, so the question moves to the owner
/// and the answer only this question has is [`DirectoryOf::HasNoDirectory`].
///
/// Every other answer is [`repository_path`]'s, over the directory rather than over `path` itself, so the
/// separator rule and the refusal of a non-UTF-8 component are stated once and hold for both questions.
pub fn repository_directory(root: &Path, path: &Path) -> DirectoryOf {
    match path.parent() {
        Some(directory) => DirectoryOf::Directory(repository_path(root, directory)),
        None => DirectoryOf::HasNoDirectory,
    }
}

/// `path` spelled as a Git **pathspec** that means exactly that path.
///
/// **`--` separates revisions from paths and nothing else.** Git parses pathspec *magic* after the
/// separator too, so a path this repository computed and handed over was read as an instruction wherever it
/// happened to begin with one. Measured on this machine's git, with a directory literally named
/// `:(exclude)odd` tracked in a fixture: `ls-files -- ':(exclude)odd'` answers the **whole** repository,
/// because the pathspec stopped restricting and became an exclusion of something else. A caller asking
/// *what does this repository track under this path* got an answer about a different question.
///
/// **On the owner of the spelling, and the two wider fixes were measured and rejected.**
/// `GIT_LITERAL_PATHSPECS=1` on the shared builder makes `check-ignore` fail outright — `fatal: pathspec
/// magic not supported by this command: 'literal'`, exit 128 — so the exclusion classifier cannot take it.
/// `--literal-pathspecs` on `hermetic_git::tracked_records` breaks the callers whose pathspec **is** a
/// pattern: `capability_subjects` asks for `openspec/specs/*/spec.md` and `law_restatement` for `*.md`, and
/// the flag disables glob magic with the rest. So the literalization belongs where the knowledge is — at
/// the point that produced a path rather than a pattern — which is here.
///
/// **What this owns and what it does not.** The spelling is held by a direction with its own control. The
/// *choice* to use it at each call site is not: measured, dropping the call in `machinery_names` compiles
/// and no direction goes red, because by then the directory is a `String` and the type that knew it was a
/// path is gone. Holding that would need a reader over code deciding which `&str` came from a path, which
/// is the judgement-over-source this repository has measured and declined. One owner is what is available;
/// the residual is that a future call site can hand git a bare derived path, and it is stated here rather
/// than left for the next reader to find.
pub fn pathspec(path: &str) -> String {
    format!(":(literal){path}")
}
