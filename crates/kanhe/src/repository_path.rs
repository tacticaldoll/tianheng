//! How this repository's checks spell a path below a root — one rule, one implementation.
//!
//! **Three readers asked one question and answered it three ways.** Every gate here that compares its own
//! paths against git's, or against another reader's, has to turn an absolute path cargo reported into the
//! repository-relative identity git and this repository's prose both use. That derivation was written out
//! at each site:
//!
//! | site | how it stripped the root | how it spelled the result |
//! |---|---|---|
//! | `release_coherence_gate::machinery_names` | `Path::strip_prefix`, component-wise | components joined with `/` |
//! | `release_coherence_gate::workspace_manifests` | `Path::strip_prefix`, falling back to the absolute path | `Path::display`, the host's own separator |
//! | `member_enumeration`'s comparison | `Path::strip_prefix`, component-wise | components joined with `/` |
//!
//! The first and third agree; the second does not, and the second is the one the third **compares
//! against**. On a host whose separator is not `/` the two sides of that comparison share no member at all.
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
/// Typed apart rather than an `Option`, because the consumers of a missing value read it differently: one
/// refuses, one used it as licence to carry the absolute path forward. A variant each leaves no reading to
/// choose, and the third keeps *not under the root* apart from *not spellable at all* — two facts an
/// operator repairs in opposite directions.
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
