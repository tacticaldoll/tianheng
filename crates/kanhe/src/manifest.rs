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

/// What `[workspace.package]` declares its version to be, or why this reader could not tell.
///
/// Typed apart rather than an `Option`, because both consumers read `None` as *the key is absent* and said
/// so to an operator — over a manifest whose value is legal to cargo and merely not in a form this reader
/// takes. The template is the sibling `PackageName` in `release_coherence_gate`, applied to the one manifest
/// fact both git-reading gates ask for.
#[derive(Debug, PartialEq, Eq)]
pub enum WorkspaceVersion {
    /// The `[workspace.package]` table's `version`.
    Declared(String),
    /// No `[workspace.package]` table, or no `version` key inside it.
    Absent,
    /// A `version` this reader cannot read — a value that is **not a string at all**, quoted as written.
    ///
    /// Not *not in double quotes*: a literal string is a string to the parser, as it is to cargo, so
    /// `version = '0.5.0'` is read. What reaches here is an integer, an array, a table.
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
    let doc = match text.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        // A root manifest cargo cannot parse has no version to report as absent. Answering `Absent` would
        // send an operator to add a key that may already be there, so the parse error is what it met.
        // The whole error, collapsed onto one line rather than its first line: measured, a duplicate
        // `version` reports the position on line one and names the key and *duplicate key* on later lines,
        // so truncating loses exactly the two facts an operator needs.
        Err(err) => {
            return WorkspaceVersion::Unreadable(format!(
                "a manifest this parser cannot read — {}",
                err.to_string()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    };
    let Some(version) = doc
        .get("workspace")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|workspace| workspace.get("package"))
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("version"))
    else {
        return WorkspaceVersion::Absent;
    };
    match version.as_str() {
        Some(declared) => WorkspaceVersion::Declared(declared.to_string()),
        // Still reachable, and now only for what it always meant: a `version` that is not a string at all —
        // `version = 5`, or the inheritance spelling in a table that declares the catalog.
        None => WorkspaceVersion::Unreadable(version.to_string().trim().to_string()),
    }
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
/// Delegates rather than re-deciding, so one question has one answer. Measured while the two were separate
/// implementations: they disagreed at the overflow boundary.
pub fn is_semver(version: &str) -> bool {
    semver(version).is_some()
}

/// Whether a member manifest's `[package]` permits publication, as far as its own text can say.
///
/// **One owner for a fact four readers had answered separately.** Two CI jobs grepped
/// `^\s*publish\s*=\s*false`, a repository check asked `starts_with("publish") && contains("false")`, and
/// `shengmo`'s self-governance read cargo's own report — and only the last one carried cargo's semantics.
/// `manifest`'s own header states the class: *two readers of one fact reaching different verdicts, in front
/// of `cargo publish`*.
///
/// **The shapes are measured against cargo 1.96.0 rather than assumed.** `cargo publish --dry-run` refuses
/// `publish = false` and refuses `publish = []` identically, and `cargo metadata` reports `[]` for both —
/// so a crate spelling its exclusion as the empty array is unpublishable, and every text reader looking for
/// the word `false` called it published. A non-empty array is a crate that publishes, to a named registry.
///
/// [`Publishable::Unreadable`] is not defensive: `publish.workspace = true` is legal and cargo honours it —
/// measured, a member inheriting `publish = false` from `[workspace.package]` reports `[]`. Its text alone
/// cannot say, so this refuses rather than guessing, exactly as [`WorkspaceVersion::Unreadable`] does one
/// field over. Both are public, so this is the **link** form and rustdoc resolves it under `-D warnings`:
/// a reference with a reaction, rather than the prose form `reference-integrity` declares a bound for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Publishable {
    /// No `publish` key, an explicit `true`, or a non-empty registry list.
    Yes,
    /// `publish = false`, or the empty registry list cargo treats identically.
    No,
    /// A value this reader cannot decide from the manifest alone, quoted as written — a workspace
    /// inheritance, an inline table, or a shape it does not know.
    Unreadable(String),
}

/// What a member manifest's own `[package]` says about publication.
///
/// Reads the `[package]` table only: a `publish` under `[workspace.package]` is the *default* a member may
/// inherit, not that member's answer, and treating the two alike would report the workspace's verdict for
/// every member.
///
/// Executed TOML text, so a commented-out `publish = false` is not read as a declared one — the reason
/// `require_internal_pins` records for the same corpus.
pub fn publishable(text: &str) -> Publishable {
    let doc = match text.parse::<toml_edit::DocumentMut>() {
        Ok(doc) => doc,
        // Cargo refuses a manifest it cannot parse — a key declared twice among them — so a reader answering
        // *publishable* from one would speak for a file cargo will not read at all.
        Err(err) => {
            return Publishable::Unreadable(format!(
                "a manifest this parser cannot read — {}",
                err.to_string()
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
        }
    };
    let Some(declared) = doc
        .get("package")
        .and_then(toml_edit::Item::as_table_like)
        .and_then(|package| package.get("publish"))
    else {
        // An absent key is cargo's default, which is publishable.
        return Publishable::Yes;
    };
    if let Some(flag) = declared.as_bool() {
        return if flag {
            Publishable::Yes
        } else {
            Publishable::No
        };
    }
    match declared.as_array() {
        // **Decided by contents, not by one spelling of the array.** `publish = [ ]` — one space, legal TOML
        // — is refused by `cargo publish` exactly as `[]` is, measured on cargo 1.96.0.
        Some(registries) if registries.is_empty() => Publishable::No,
        Some(_) => Publishable::Yes,
        // A `publish` that is neither a boolean nor an array: `publish.workspace = true` defers to the
        // workspace manifest, which is not a verdict this text carries, and any other shape is one this
        // reader has not met.
        None => Publishable::Unreadable(declared.to_string().trim().to_string()),
    }
}
