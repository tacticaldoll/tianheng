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
            "release-coherence#changelog-in-head-unreadable",
            "presence is asked by `ls-tree HEAD -- CHANGELOG.md`, which exits non-zero only where git \
             cannot read HEAD's **tree** — and the judgement resolves HEAD and the release commit before \
             it, through reads that fail first on any object store broken enough to reach this one. \
             Measured: a fixture whose `.git/index` is corrupt leaves tree reads working, which is why the \
             content read moved off `status` in the first place, and a fixture whose objects are gone \
             fails at `rev-parse`. The blob half of the same question **is** observed, by \
             `a_changelog_git_cannot_read_at_head_is_not_a_modified_worktree`, because a tree can name a \
             blob git will not hand over",
        ),
        tool(
            "publish-source-integrity#release-tag-unreadable",
            "measured, not assumed: every ref-store perturbation a fixture can build answers `1`, which is this read's ANSWER — an unreadable `refs/tags` and a `refs/tags` replaced by a file both exit `1`, and in that state `rev-parse HEAD` fails first at `128`, so the judgement never reaches this arm. The classifier itself IS observed, against a directory that is no repository; what no fixture can build is a repository whose earlier reads succeed and whose tag read declines",
        ),
        tool(
            "publish-source-integrity#remote-tag-unreadable",
            "the sibling read of the same remote runs first: `ls-remote refs/heads/main` and \
             `ls-remote refs/tags/<tag>` reach one transport by one command, so a remote a fixture can make \
             unreadable answers the first read that way and returns before this one. Reaching this needs the \
             remote to fail between two invocations, which a fixture would have to hold open — measured \
             against the unreachable-remote direction, which lands on `remote-main-unreadable`",
        ),
        tool(
            "publish-source-integrity#tag-object-unresolvable",
            "`rev-parse refs/tags/<tag>` runs after the tag's presence, its object and its signature have \
             all been read from the same store, so a ref-store state that answers those and not this is one \
             no fixture can build — the sibling `release-tag-unreadable` records the same measurement for \
             the reads above it",
        ),
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
            "release-coherence#member-directory-not-utf8",
            "the manifest this reader spells is a `&str` cargo's JSON handed over, so the parser made its \
             components UTF-8 before this gate saw them and no path built from one can carry a byte the \
             decode refuses — the same arm reached from a filesystem walk IS observed, by \
             `a_crate_directory_that_is_not_utf8_is_refused_by_the_walk`, which is where the bytes are the \
             operating system's rather than a parser's",
        ),
        tool(
            "release-coherence#crate-manifest-outside-repository",
            "the manifest is joined onto `repo` in the same loop that strips it, so a walk output sitting \
             outside `repo` is this reader disagreeing with itself rather than a tree a fixture can build \
             — the sibling `member-manifest-outside-workspace-root` above records the same shape for the \
             answer cargo gives, where the join is cargo's rather than this walk's",
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
