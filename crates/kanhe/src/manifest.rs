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

/// A double-quoted value this reader found, or a statement that it could not read one.
///
/// **Not an `Option`, because every consumer of the one this replaces read `None` as *the key is absent* and
/// skipped the line.** Single-quoted and literal TOML strings are valid and are not read here; that is a
/// limit of this reader, not a fact about the manifest, and conflating the two let a readable-to-cargo
/// manifest go unchecked while the surrounding function still returned `Ok`. Measured on this repository with
/// one crate's name single-quoted, the release gate reported a clean release.
///
/// A type that cannot be defaulted, so the compiler asks each site which of the two it meant.
///
/// It lives here rather than beside one of its readers because **both** manifest facts this module owns need
/// it, and a value-reading rule with two owners is the twin this module's header exists to close.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Quoted {
    /// The double-quoted string the value **is**.
    Value(String),
    /// The value is not a double-quoted string: a single-quoted or literal value, a number or boolean, or a
    /// shape this reader has never met.
    Unreadable,
}

/// The double-quoted string `value` is, or a statement that it is not one.
///
/// `value` is the text **after** the `=`, and the quote has to open it. Taking the first pair of quotes
/// anywhere in the text instead let an unquoted value borrow the next key's: `package = xuanji, version =
/// "0.2.0"` read its package as `0.2.0`, so an identity this reader cannot read was reported as one it
/// could, and the entry then failed to match any family crate and was skipped in silence. `Unreadable` is
/// the whole point of this type and it was reachable only when nothing else on the line was quoted.
///
/// Unifying the contract is what makes strictness possible: two `Cargo.lock` readers passed a whole line
/// where every other caller passed the value, so a rule about where the quote sits had no single subject to
/// be about. They split on the `=` now, like everyone else.
pub(crate) fn quoted_value(value: &str) -> Quoted {
    let Some(rest) = value.trim_start().strip_prefix('"') else {
        return Quoted::Unreadable;
    };
    let Some((body, _)) = rest.split_once('"') else {
        return Quoted::Unreadable;
    };
    // **A backslash opens an escape, and this reader decodes none — so it refuses instead of returning the
    // source as though it were the value.** In a TOML *basic* string a `\\` is never literal; cargo decodes
    // the escape and this reader would hand its consumers the raw sequence. Measured on cargo 1.96.0 against
    // a scratch workspace: `path = "crates/\\u0078uanji"` resolves the member at `crates/xuanji`,
    // `name = "xuan\\u006Ai"` reads as `xuanji`, and `version = "0.\\u0035.0"` reads as `0.5.0`. Every
    // consumer here then compares the *undecoded* text — against a family crate list, against a
    // `crates/` prefix, against a version — and a comparison that fails takes a `continue`, so an internal
    // dependency or a renamed family crate stops being checked with nothing saying so. The per-manifest
    // vacuity guards cannot see it either: one escaped entry beside one ordinary one leaves their counters
    // non-zero.
    //
    // `Unreadable` is what this type exists for, and each consumer already answers it by refusing to judge.
    // Decoding the escapes here is deliberately NOT done: that is a TOML grammar, and writing a second
    // hand-rolled one is the defect class `BACKLOG.md` already carries for these readers.
    //
    // It also closes the narrower shape the same check missed: `"a\\"b"` split at the ESCAPED quote and
    // answered `a\\`, an identity no manifest declares.
    //
    // No tracked manifest carries a backslash in a quoted value — measured over `git ls-files '*.toml'` —
    // so this refuses nothing this repository writes today.
    if body.contains('\\') {
        return Quoted::Unreadable;
    }
    Quoted::Value(body.to_string())
}

/// What `[workspace.package]` declares its version to be, or why this reader could not tell.
///
/// Three states rather than an `Option`, because both consumers read `None` as *the key is absent* and said
/// so to an operator — over a manifest whose value is legal to cargo and merely not in a form this reader
/// takes. The template is the sibling `PackageName` in `release_coherence_gate`, applied to the one manifest
/// fact both git-reading gates ask for.
#[derive(Debug, PartialEq, Eq)]
pub enum WorkspaceVersion {
    /// The `[workspace.package]` table's `version`.
    Declared(String),
    /// No `[workspace.package]` table, or no `version` key inside it.
    Absent,
    /// A `version` this reader cannot read — a value not in double quotes — quoted as written.
    Unreadable(String),
}

/// The version `[workspace.package]` declares.
///
/// Scoped to that table: the first `version` key inside it, and no other table's. A `[package]` table, or
/// any other, closes the scan rather than contributing — see this module's header for why the publish
/// gate's former fallback is gone.
///
/// **Read from the shared region, not from raw lines.** `repository-checks` requires a check deciding a
/// property over executed text to take its corpus from [`crate::region`], and this reader did not. Both
/// directions were live and both were false refusals over legal TOML: `[workspace.package] # …` failed the
/// heading equality, then matched `starts_with('[')` and *closed* the table before it opened, so the version
/// read as absent; and `version = "0.5.0" # bumped` carried its comment into the value, which then parsed as
/// no semantic version. The second is the release-prep spelling, so the reader failed at the one moment
/// someone is most likely to annotate that line — in front of `cargo publish` and the release gate.
///
/// The sibling `package_name` had already been repaired this way, which left one root `Cargo.toml` scanned
/// under two different region decisions inside a single judgement.
pub fn workspace_version(text: &str) -> WorkspaceVersion {
    let source = crate::region::Source::of(text);
    let mut inside = false;
    for line in source.toml().lines() {
        let trimmed = line.trim();
        if trimmed == "[workspace.package]" {
            inside = true;
            continue;
        }
        if trimmed.starts_with('[') {
            inside = false;
            continue;
        }
        if !inside {
            continue;
        }
        // `version` then `=`, so `version.workspace` and any other `version…` key is not this key.
        let Some(rest) = trimmed
            .strip_prefix("version")
            .and_then(|rest| rest.trim_start().strip_prefix('='))
        else {
            continue;
        };
        return match quoted_value(rest) {
            Quoted::Value(version) => WorkspaceVersion::Declared(version),
            Quoted::Unreadable => WorkspaceVersion::Unreadable(rest.trim().to_string()),
        };
    }
    WorkspaceVersion::Absent
}

/// What both git-reading gates tell an operator when `[workspace.package]` names no version.
///
/// **The text is shared; the site is not, and cannot be.** This sentence and the two arms around it were
/// written out in both gates. Converging the whole arm is the obvious repair and the refusal register
/// forbids it: a site is registered by the string literal that **opens** the constructor's argument list, so
/// a site id arriving as a variable is a construction the register cannot parse — and it holds that count at
/// zero. Collapsing the arms instead would fold each gate's own identity into a shared one, which is the thing
/// the register exists to prevent. So the arms stay twinned by that constraint, and only what was genuinely
/// duplicable moved.
///
/// So what moves is what was actually duplicated. Each gate keeps its own constructor call with its own
/// literal identity, and the sentence those calls carry has one owner.
pub const VERSION_ABSENT: &str = "workspace version is missing or malformed: <missing>";

/// What both gates say for a version this reader cannot read, with each gate's own consequence appended.
///
/// `tail` is the half that genuinely differs — what the caller could not decide as a result — and it is a
/// parameter rather than a second copy of the sentence.
pub fn version_unreadable(what: &str, tail: &str) -> String {
    format!("Cargo.toml declares a workspace version this check cannot read ({what}), so {tail}")
}

/// What both gates say for a version that is present, readable, and not a semantic version.
pub fn version_malformed(version: &str) -> String {
    format!("workspace version is missing or malformed: {version}")
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
