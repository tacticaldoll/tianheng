//! The manifest facts both git-reading gates ask for, with one implementation each.
//!
//! **The same pair of files, and the same lesson, as [`crate::hermetic_git`].** That module's own doc says
//! the command builder "lived twice, byte-identical, in `publish_source_gate` and `release_coherence_gate`
//! … what two implementations of one thing cost". Two more twins were left behind in that extraction —
//! *which version the workspace declares*, and *is this a semantic version* — and unlike the pair that was
//! taken, **these two had diverged**:
//!
//! | fact | publish gate | coherence gate |
//! |---|---|---|
//! | workspace version | also accepted a `[package]` table | `[workspace.package]` only |
//! | semver | a digit check, so `1.0.99999999999999999999` passed | parsed to `u64`, so it did not |
//!
//! Two readers of one fact reaching different verdicts, in front of `cargo publish`.
//!
//! # The `[package]` fallback is not carried forward
//!
//! It was unreachable for every subject either gate has, measured rather than assumed: this repository's
//! root declares `[workspace.package]` and no `[package]`, and both gates' own fixtures write
//! `[workspace.package]` too. Keeping it would have preserved an untested branch to settle a disagreement
//! that no input could produce.
//!
//! A single-crate root now reads as *no workspace version*, which both callers already treat as a
//! cannot-judge. That is the right direction for a wrapper whose publish is `--workspace`: a root with no
//! workspace table is not the shape either gate was written to judge, and saying so beats guessing.

/// The version `[workspace.package]` declares, if it declares one.
///
/// Scoped to that table: the first `version` key inside it, and no other table's. A `[package]` table, or
/// any other, closes the scan rather than contributing — see this module's header for why the publish
/// gate's former fallback is gone.
pub fn workspace_version(text: &str) -> Option<String> {
    let mut inside = false;
    for line in text.lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if inside {
            if let Some(rest) = trimmed
                .strip_prefix("version")
                .and_then(|rest| rest.trim_start().strip_prefix('='))
            {
                return Some(rest.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// `major.minor.patch` as numbers, or `None` if `version` is not one.
///
/// **Parsed, not pattern-matched**, and that is the divergence this replaces. A digit check answers *does
/// this look like a version* and admits `1.0.99999999999999999999`; parsing answers *is this a version this
/// family can order*, and a component that overflows `u64` is not. The publish gate asked the first
/// question and the coherence gate the second, about the same string.
///
/// A leading zero is refused on a multi-digit component, so `01.0.0` is not a version.
pub fn semver(version: &str) -> Option<(u64, u64, u64)> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut out = [0u64; 3];
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty()
            || !part.chars().all(|c| c.is_ascii_digit())
            || (part.len() > 1 && part.starts_with('0'))
        {
            return None;
        }
        out[index] = part.parse().ok()?;
    }
    Some((out[0], out[1], out[2]))
}

/// Whether `version` is a semantic version — [`semver`]'s question, asked for a yes or no.
///
/// Delegates rather than re-deciding: the two used to be separate implementations and answered differently
/// at the overflow boundary.
pub fn is_semver(version: &str) -> bool {
    semver(version).is_some()
}
