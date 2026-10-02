use std::path::Path;

/// Whether a tracked path lies under `openspec/changes/`, `openspec/changes/archive/` included.
///
/// The extent is the directory, not the word *active*: an archive entry is under it too, and the one
/// tracked file the archive holds is excluded by the same test. The tracked paths of a change directory
/// are a plan's working files, so a direction that reads them as live text judges a proposal for naming
/// its own deliverable.
pub(super) fn is_openspec_change_path(path: &str) -> bool {
    Path::new(path)
        .strip_prefix("openspec/changes")
        .is_ok_and(|remainder| remainder.components().next().is_some())
}

/// The tracked paths a direction reads as **source**, with every path under `openspec/changes/` already
/// excluded.
///
/// The only constructor is [`SourceCorpus::of`], and it applies [`is_openspec_change_path`], so a direction
/// that takes this type cannot read a change directory and cannot forget to exclude it. A direction that
/// resolves a reference, or asks what the repository once tracked, is asking about **evidence** and takes
/// the plain tracked list instead: a plan's files still exist there, and a reference to one still resolves.
///
/// Every count a direction takes over its corpus, a vacuity guard included, is taken over the **excluded**
/// corpus. A corpus holding only change-directory paths is therefore empty, and the direction's guard
/// refuses it as it refuses any corpus that held nothing to read; a deliberately excluded file no longer
/// counts as evidence that the enumeration produced something.
pub(super) struct SourceCorpus(Vec<String>);

impl SourceCorpus {
    pub(super) fn of<S: AsRef<str>>(tracked: &[S]) -> Self {
        Self(
            tracked
                .iter()
                .map(AsRef::as_ref)
                .filter(|path| !is_openspec_change_path(path))
                .map(str::to_string)
                .collect(),
        )
    }

    pub(super) fn paths(&self) -> &[String] {
        &self.0
    }
}
