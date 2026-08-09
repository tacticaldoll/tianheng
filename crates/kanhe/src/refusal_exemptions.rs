//! Which refusal sites are declared out of reach, and the bound each is covered by.
//!
//! This table is **never the authority for anything**. Both of its columns are joined, in both directions,
//! against sets that are produced — the site enumeration and the live `observation_bounds()` — so it cannot
//! drift silently: a slug no site carries fails, a site whose slug is not here fails, a bound named here that
//! the live set does not hold fails, and a site declared here that a run is observed to reach fails as a
//! stale exemption. It is a join table, not a declaration.
//!
//! A site reaches this table only after both cheaper answers have been tried and refused. **Construct** it —
//! fourteen sites left this list that way, through a corrupt index, an everything-ignored worktree, a missing
//! object, a directory where a file was expected, and a file whose bytes are not UTF-8. **Delete** it — two
//! more were second reads of something the judgement already held, which is dead code rather than a guard.
//! Only what survives both is declared, because declaring first produces exemptions for code that should not
//! exist, and the exempt set then reads as a limit of the reaction rather than as a property of the world.

#![allow(dead_code)]

/// The bound covering every entry below.
pub const OUT_OF_REACH_BOUND: &str = "rust-repository-reactions/whether-a-declared-out-of-reach-refusal-is-genuinely-unconstructible-is-not-observed-a-stated-bound";

/// One site declared out of reach, joined to the bound that covers it.
pub struct Exemption {
    /// Carried at the site as a string literal, so it moves with the site and the compiler checks it exists.
    pub slug: &'static str,
    /// The `BoundId` covering this exemption, held against the live declaration set.
    pub bound: &'static str,
    /// Why no environment the suite runs in can produce this site's precondition.
    pub because: &'static str,
}

/// Every declared exemption. Two classes, and neither is about this repository's contents.
pub fn exemptions() -> Vec<Exemption> {
    vec![
        // --- the machine, not the repository ---
        Exemption {
            slug: "ssh-keygen-absent",
            bound: OUT_OF_REACH_BOUND,
            because: "the tool is resolved through the judging process's own PATH, which every direction \
                      running beside this one shares; a fixture cannot remove it for one call",
        },
        Exemption {
            slug: "signature-scratch-not-creatable",
            bound: OUT_OF_REACH_BOUND,
            because: "the scratch lives under the process's temp directory, so making it uncreatable is a \
                      change to the machine rather than to a repository under test",
        },
        Exemption {
            slug: "signature-mechanism-round-trip",
            bound: OUT_OF_REACH_BOUND,
            because: "it needs an ssh-keygen that is present and broken, which is a property of the machine \
                      the gate runs on and the reason this probe exists at all",
        },
        Exemption {
            slug: "signature-file-not-writable",
            bound: OUT_OF_REACH_BOUND,
            because: "the write goes into a directory this same call has just created under the process's \
                      temp directory",
        },
        // --- an I/O failure a fixture cannot schedule ---
        Exemption {
            slug: "directory-entry-unyieldable",
            bound: OUT_OF_REACH_BOUND,
            because: "a `read_dir` iterator yields an error only when the filesystem fails part-way through \
                      an enumeration, which a fixture cannot schedule; propagating it is still right, because \
                      dropping the entry lets the remainder satisfy the counters the judgement reasons from",
        },
        // --- git's two extractions disagreeing with each other ---
        Exemption {
            slug: "signature-block-unreadable",
            bound: OUT_OF_REACH_BOUND,
            because: "it is a third read of the tag object whose content was just read; removing it instead \
                      would mean extracting the signature ourselves, replacing something git does with \
                      something we reason about, in the path that decides whether a release tag is signed",
        },
        Exemption {
            slug: "signature-block-not-a-suffix",
            bound: OUT_OF_REACH_BOUND,
            because: "git extracted the block from the object it printed, so it is a suffix by construction; \
                      the guard exists in case git's two extractions ever disagree, which no fixture can \
                      arrange",
        },
    ]
}
