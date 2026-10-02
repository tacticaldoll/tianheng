use std::path::Path;

/// Whether a tracked path belongs to an active OpenSpec change directory.
pub(super) fn is_active_openspec_change_path(path: &str) -> bool {
    Path::new(path)
        .strip_prefix("openspec/changes")
        .is_ok_and(|remainder| remainder.components().next().is_some())
}
