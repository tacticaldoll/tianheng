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

/// A component that is not UTF-8 is refused, never spelled with a replacement.
///
/// **Measured rather than reasoned about**, and the same measurement `hermetic_git`'s own decode records:
/// a path carrying bytes no `String` holds is legal on Unix, and it is the repository's own name for that
/// file. `to_string_lossy` substitutes U+FFFD per undecodable byte, so the answer names nothing git holds
/// — and two distinct names collapse onto one spelling, which is the collision a walk must not make.
///
/// Negative run: with the decode restored to `to_string_lossy`, this returns
/// `Below("crates/\u{fffd}kanhe")` — a verdict reached over a directory that is not the one on disk.
#[test]
#[cfg(unix)]
fn a_component_that_is_not_utf8_is_refused_rather_than_replaced() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let name = OsStr::from_bytes(b"\xffkanhe");
    let path = Path::new("/r").join("crates").join(name);

    assert!(
        matches!(
            repository_path(Path::new("/r"), &path),
            RepositoryPath::NotUtf8(_)
        ),
        "a component the repository holds as bytes is refused, not replaced; answered {:?}",
        repository_path(Path::new("/r"), &path)
    );
}

/// The refusal names the component it could not read, not merely that one existed.
///
/// A message saying only *some component is not UTF-8* leaves an operator with a workspace to search. The
/// carried spelling is lossy on purpose and only here: it is the message, never the identity.
#[test]
#[cfg(unix)]
fn the_refusal_names_the_component_it_could_not_read() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let path = Path::new("/r")
        .join("crates")
        .join(OsStr::from_bytes(b"\xffkanhe"));

    assert_eq!(
        repository_path(Path::new("/r"), &path),
        RepositoryPath::NotUtf8("\u{fffd}kanhe".to_string())
    );
}
