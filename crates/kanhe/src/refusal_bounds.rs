//! The refusal sites this repository declares **unheld**: reached by no direction, and deliberately so.
//!
//! A refusal is normally held by a direction that observes it, and the refusal register refuses a registered
//! site nothing names. This is the third state, and it exists because forcing the second would be worse
//! than the gap it closes.
//!
//! **The distinction is not difficulty, it is what the fixture would test.** A refusal about the judged
//! *subject* — a manifest cargo accepts, a tag that exists, a changelog someone wrote — has a fixture that
//! *is* its specification, and neutralising the branch lets a real defect reach a release. A refusal about
//! the *reading* failing — a tool that will not run, a directory that will not enumerate, output that is not
//! JSON — can only be reached by breaking the machine, so its fixture must simulate the broken tool. A
//! fixture that simulates a tool tests the simulation: it can pass while the branch it names is wrong, which
//! is a false green, and a false green is worse than a declared gap.
//!
//! The split is not a judgement made here. Every site in this table is a **cannot-judge**, and the compiler
//! said so before anyone claimed it: with the last subject-class refusal registered, `violation` became an
//! unused import in the release gate.
//!
//! **The escape hatch is not free.** A declaration is typed, counted, projected, and carries an owner and a
//! tracker; the register holds this table and the sites in a bijection, refuses a declared site that a
//! direction *does* observe — that one is held, and should say so — and requires the count of untriaged
//! sites to be **zero**. What this table cannot do is decide its own membership, which stays a reviewer's
//! obligation, stated here rather than implied by a list that looks complete.

use tianheng::Owner;

/// One refusal site held by no direction, and why.
pub struct Unheld {
    /// The site's registered identity, exactly as the refusal carries it.
    pub site: &'static str,
    /// Why a direction over it would test something other than this branch.
    pub because: &'static str,
    /// Who would close it.
    pub owner: Owner,
    /// Where closing it is tracked.
    pub tracker: &'static str,
}

/// Every refusal site this repository declares unheld.
pub fn unheld() -> Vec<Unheld> {
    const TRACKER: &str =
        "`BACKLOG.md` — *a refusal reachable only by a broken tool is not observed*";
    let tool = |site: &'static str, because: &'static str| Unheld {
        site,
        because,
        owner: Owner::Engine,
        tracker: TRACKER,
    };
    vec![
        tool(
            "release-coherence#directory-entry-unreadable",
            "a directory entry that errors while the directory itself enumerates is produced by the \
             filesystem between two syscalls, and a fixture would have to hold that window open",
        ),
        tool(
            "release-coherence#metadata-has-no-workspace-root",
            "cargo emits `workspace_root` for every workspace it can load, so reaching this means replacing \
             cargo with something that answers differently — the direction would then observe the \
             replacement",
        ),
        tool(
            "release-coherence#metadata-package-has-no-manifest-path",
            "same corpus as its sibling above: a package cargo reports without a manifest path is not a \
             shape cargo produces, so the fixture is a fake cargo",
        ),
        tool(
            "release-coherence#member-manifest-outside-workspace-root",
            "cargo resolves member paths against the root it reports, so a member outside it is a \
             disagreement inside cargo rather than a shape a manifest can carry",
        ),
        tool(
            "release-coherence#no-tracked-file-for-any-member",
            "every member having no tracked file requires a workspace cargo can load from paths git does \
             not track, which is a repository shape rather than a release surface — the guard stays because \
             the enumeration it protects is the one that decides the machinery corpus",
        ),
        tool(
            "release-coherence#scripts-not-enumerable",
            "`git ls-files` failing while the same process already read the repository is a git failure \
             mid-run, and simulating it means putting a fake git on the path",
        ),
        tool(
            "release-coherence#cargo-metadata-unrunnable",
            "cargo absent from the path of a process cargo is running",
        ),
        tool(
            "release-coherence#cargo-metadata-not-json",
            "cargo emitting something that is not JSON, which is a fake cargo by construction",
        ),
        tool(
            "publish-source-integrity#repository-root-is-not-a-worktree",
            "git failing to *start*, as against git running and refusing — the refusing half is observed. \
             Reaching this means a machine without git, and a fixture that removes git from the path tests \
             the path manipulation",
        ),
        tool(
            "publish-source-integrity#ssh-keygen-unavailable",
            "the same shape one tool over, and with the same fixture: `ssh-keygen` removed from the path",
        ),
        tool(
            "publish-source-integrity#signature-mechanism-round-trip-failed",
            "the gate's own probe signs and verifies a payload before trusting its verdict; reaching this \
             means an `ssh-keygen` that signs and then fails to verify its own signature",
        ),
        tool(
            "publish-source-integrity#signature-block-unreadable",
            "git failing to read a tag object it has already resolved, mid-run",
        ),
        tool(
            "publish-source-integrity#signature-is-not-the-tag-object-suffix",
            "a tag object whose signature block is not its own suffix is one git does not write, so the \
             fixture would be a hand-assembled object testing this reader against that assembly",
        ),
        tool(
            "publish-source-integrity#signature-unwritable",
            "a scratch directory this process created and cannot write to. Running as root defeats the \
             fixture outright, which makes the direction's own result depend on who runs it",
        ),
    ]
}
