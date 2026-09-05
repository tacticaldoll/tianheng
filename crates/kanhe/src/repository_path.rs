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

/// Where a path sits relative to a root, or that it does not sit under one.
///
/// Typed apart rather than an `Option`, because the two consumers of the answer read a missing value
/// differently: one refuses, one used it as licence to carry the absolute path forward. A variant each
/// leaves no reading to choose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryPath {
    /// The path below the root, spelled with `/` whatever separator the host uses.
    Below(String),
    /// The path does not sit under the root it was resolved against.
    Outside,
}

/// `path` spelled relative to `root`, the way git spells one.
///
/// The comparison is component-wise, so a prefix matches on its own boundaries rather than on a separator
/// byte, and the answer is rebuilt with `/` so it can be handed to `git` and compared against what git
/// answers.
pub fn repository_path(root: &Path, path: &Path) -> RepositoryPath {
    let Ok(below) = path.strip_prefix(root) else {
        return RepositoryPath::Outside;
    };
    RepositoryPath::Below(
        below
            .components()
            .map(|part| part.as_os_str().to_string_lossy().into_owned())
            .collect::<Vec<_>>()
            .join("/"),
    )
}
