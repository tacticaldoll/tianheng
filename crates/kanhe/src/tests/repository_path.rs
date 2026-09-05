//! The failure matrix for [`crate::repository_path`]: what one spelling of a path below a root answers.

use std::path::Path;

use crate::repository_path::{RepositoryPath, repository_path};

/// A path under the root is spelled the way git spells one, whatever the host's separator is.
///
/// The join is `/` because the answer is handed to `git ls-files` and compared, as a **string**, against
/// what git and another reader answer. On a host whose separator is already `/` this direction cannot tell
/// a `/` join from `Path::display` — that difference is what the walk and its comparator disagreed about,
/// and it is unobservable here. What this pins is the shape both of them must produce.
#[test]
fn a_path_under_the_root_is_spelled_with_the_separator_git_uses() {
    assert_eq!(
        repository_path(Path::new("/r"), Path::new("/r/crates/kanhe/Cargo.toml")),
        RepositoryPath::Below("crates/kanhe/Cargo.toml".to_string())
    );
}

/// A path that is not under the root says so, rather than handing back the path it was given.
///
/// **This is the arm that was live.** `workspace_manifests` read
/// `strip_prefix(repo).unwrap_or(&manifest)`, so a failed strip carried the **absolute** path forward as
/// though it were repository-relative — into a set compared against repository-relative paths, where it
/// matches nothing and reports the member as both unwalked and undeclared.
///
/// Negative run: with the `else` arm replaced by the previous reader's fallback — returning
/// `RepositoryPath::Below(path.display().to_string())` — this direction fails with
/// `Below("/elsewhere/kanhe/Cargo.toml")`, an absolute path presented as one below the root.
#[test]
fn a_path_outside_the_root_is_not_reported_as_sitting_under_it() {
    assert_eq!(
        repository_path(Path::new("/r"), Path::new("/elsewhere/kanhe/Cargo.toml")),
        RepositoryPath::Outside
    );
}

/// A root that is a text prefix of a **sibling's** name does not strip it.
///
/// `/r/crates` is a character-for-character prefix of `/r/crates-extra`, and a reader comparing text would
/// report the sibling as sitting inside the root. `Path::strip_prefix` compares components, so the boundary
/// is where a path boundary is rather than where the bytes stop matching.
///
/// Negative run: with the strip written as `path.to_string_lossy().strip_prefix(&root.to_string_lossy())`,
/// this direction fails with `Below("-extra/kanhe/Cargo.toml")` — a spelling that names nothing.
#[test]
fn a_root_that_is_a_text_prefix_of_a_sibling_does_not_contain_it() {
    assert_eq!(
        repository_path(
            Path::new("/r/crates"),
            Path::new("/r/crates-extra/kanhe/Cargo.toml")
        ),
        RepositoryPath::Outside
    );
}

/// The root itself is below itself, and the answer is empty rather than absent.
///
/// `machinery_names` asks this of a member whose manifest sits at the root, and the empty string is what it
/// then has to say something about. Answering `Outside` here would make a member of the root unreachable
/// through a reader that can see it.
#[test]
fn the_root_itself_is_below_the_root_and_spells_as_nothing() {
    assert_eq!(
        repository_path(Path::new("/r"), Path::new("/r")),
        RepositoryPath::Below(String::new())
    );
}
