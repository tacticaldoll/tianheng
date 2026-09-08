//! The release snapshot subject shared by the history, merge, and publish gates.
//!
//! A release version has one canonical rendering. The history reader also recognizes the retired rendering
//! so a release branch based on the existing spine can reach its first conventional snapshot; the gates in
//! front of new merges and publishes accept only the canonical form.

use crate::manifest::semver;

pub(crate) const CANONICAL_PREFIX: &str = "chore(release): ";
pub(crate) const LEGACY_PREFIX: &str = "release: ";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Form {
    Canonical,
    Legacy,
}

pub(crate) fn canonical(version: &str) -> String {
    format!("{CANONICAL_PREFIX}{version}")
}

pub(crate) fn parse(subject: &str) -> Option<(Form, &str)> {
    let (form, version) = subject
        .strip_prefix(CANONICAL_PREFIX)
        .map(|version| (Form::Canonical, version))
        .or_else(|| {
            subject
                .strip_prefix(LEGACY_PREFIX)
                .map(|version| (Form::Legacy, version))
        })?;
    semver(version).map(|_| (form, version))
}

pub(crate) fn canonical_version(subject: &str) -> Option<&str> {
    parse(subject).and_then(|(form, version)| (form == Form::Canonical).then_some(version))
}
