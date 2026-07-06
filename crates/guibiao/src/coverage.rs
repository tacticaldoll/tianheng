use crate::{Boundary, Constitution};

/// Workspace coverage: how many workspace members exist and which are governed by no
/// boundary. A projection (an observation), not a reaction — it never changes the exit
/// code. Part of the projection surface (the 天衡 runner folds it into its report), not the
/// reaction/law API.
pub struct Coverage {
    /// Total number of workspace members.
    pub total: usize,
    /// Names of workspace members that are the target of no boundary, sorted.
    pub uncovered: Vec<String>,
}

/// The pure core of coverage: workspace `members` against the crates any boundary
/// targets. A crate counts as covered by a crate boundary on it or a module boundary
/// within it.
pub(crate) fn coverage_from(members: Vec<String>, constitution: &Constitution) -> Coverage {
    let mut targeted: Vec<&str> = Vec::new();
    for boundary in constitution.boundaries() {
        match boundary {
            Boundary::Crate(b) => targeted.push(b.target().package.as_str()),
            Boundary::Module(b) => targeted.push(b.crate_package.as_str()),
        }
    }
    let total = members.len();
    let uncovered = members
        .into_iter()
        .filter(|member| !targeted.contains(&member.as_str()))
        .collect();
    Coverage { total, uncovered }
}
