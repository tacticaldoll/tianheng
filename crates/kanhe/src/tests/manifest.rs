use crate::manifest::{is_semver, semver, workspace_version};

/// The two gates asked the same question about a version and answered differently.
///
/// `publish_source_gate::is_semver` was a digit check and `release_coherence_gate::semver` a parse, so a
/// component too large for `u64` was a version to one and not to the other — in front of `cargo publish`.
/// This is the boundary where they parted, held so the resolution cannot quietly reopen.
#[test]
fn a_component_too_large_to_order_is_not_a_version() {
    assert!(
        semver("1.0.99999999999999999999").is_none(),
        "a component that overflows `u64` cannot be ordered, so it is not a version this family reads"
    );
    assert_eq!(
        is_semver("1.0.99999999999999999999"),
        semver("1.0.99999999999999999999").is_some(),
        "the yes/no question and the parse must answer together — they are one implementation now, and \
         they were two that disagreed at exactly this input"
    );
    assert!(semver("1.0.0").is_some(), "an ordinary version still reads");
    assert!(is_semver("0.5.0"), "and so does this repository's own");
}

/// A `[package]` root reads as no workspace version, rather than as that package's.
///
/// The publish gate accepted a `[package]` table where its sibling did not. The fallback was unreachable —
/// this repository's root and both gates' fixtures declare `[workspace.package]` — so it was dropped rather
/// than carried forward as an untested branch settling a disagreement no input could produce. A root with
/// no workspace table is not the shape either gate judges, and both callers read `None` as a cannot-judge.
#[test]
fn a_single_crate_root_declares_no_workspace_version() {
    let single = "[package]\nname = \"solo\"\nversion = \"9.9.9\"\n";
    assert_eq!(
        workspace_version(single),
        None,
        "a `[package]` version is not the workspace's, and reading it as one is what the two gates \
         disagreed about"
    );
    let workspace = "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"0.5.0\"\n";
    assert_eq!(workspace_version(workspace), Some("0.5.0".to_string()));
}

/// The scan is scoped to the table, so a later table's `version` is not the workspace's.
#[test]
fn a_version_under_another_table_is_not_the_workspace_version() {
    let manifest = "[workspace.package]\nversion = \"0.5.0\"\n\n[package]\nversion = \"9.9.9\"\n";
    assert_eq!(workspace_version(manifest), Some("0.5.0".to_string()));
}
