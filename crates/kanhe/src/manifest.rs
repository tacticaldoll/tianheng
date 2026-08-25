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
    // **A multiline basic string opens with three quotes, and this reader reads one.** TOML admits
    // `"""…"""` wherever it admits `"…"`, and cargo reads it — measured on cargo 1.96.0,
    // `path = """crates/xuanji"""` resolves the member and `name = """xuanji"""` reads as `xuanji`. Stripping
    // one quote and taking the next left `body` EMPTY, so this answered `Value("")`: an empty path, an empty
    // identity, an empty version, each of which its consumer compares and passes over. That is the same
    // silence the backslash branch below closes, reached without a backslash — so that branch cannot see this
    // shape and a check of its own is what closes it.
    //
    // **Its position is not the property, and an earlier version of this comment said it was.** It reads
    // `rest`, not `body`, so it answers the same before or after the split. Measured by moving it past the
    // body read: the direction over it stayed green. What matters is that it exists, not where it sits.
    //
    // Two quotes rather than three is what is tested, because `rest` has already lost the opening one. An
    // ordinary empty string is `""` — one quote here — and stays a value it can read.
    if rest.starts_with("\"\"") {
        return Quoted::Unreadable;
    }
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
    // **One predicate where there were two, which is what the bug above was made of.** This walked a heading
    // twice — an equality that opened the table, then a `starts_with('[')` that closed it — so a heading
    // failing the first matched the second and closed a table that had never opened. Reading a `toml()` region
    // stopped the comment from causing that; taking the cut removes the second test entirely, so the shape
    // cannot recur for a spelling nobody thought of.
    let tables = crate::sections::cut(source.toml().numbered_lines(), |line| {
        is_table(line).then(|| line.trim() == "[workspace.package]")
    });
    // `version` then `=`, so `version.workspace` and any other `version…` key is not this key.
    let values: Vec<&str> = tables
        .iter()
        .filter(|table| table.name)
        .flat_map(|table| table.body.iter())
        .filter_map(|(_, line)| {
            line.trim()
                .strip_prefix("version")
                .and_then(|rest| rest.trim_start().strip_prefix('='))
        })
        .collect();
    // **Two keys refuse rather than the first one answering, which its two siblings already did.** This
    // returned on the first `version` it met. `publishable` states the reason in its own words — *cargo
    // refuses a manifest that declares one key twice, so a reader answering from the first of two would speak
    // for a file cargo will not read at all* — and `package_name` answers the same way. One of three reading
    // the same root manifest disagreed, and taking a value first is what made the count askable.
    match values.len() {
        0 => WorkspaceVersion::Absent,
        1 => match quoted_value(values[0]) {
            Quoted::Value(version) => WorkspaceVersion::Declared(version),
            Quoted::Unreadable => WorkspaceVersion::Unreadable(values[0].trim().to_string()),
        },
        several => WorkspaceVersion::Unreadable(format!(
            "{several} `version` keys in `[workspace.package]`"
        )),
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
/// cannot say, so this refuses rather than guessing, exactly as `Quoted::Unreadable` does for a value one
/// field over. Written as prose rather than a link because this type is public and that one is
/// `pub(crate)`: rustdoc refuses the link form under `-D warnings`, which is `reference-integrity`'s
/// declared bound about prose-form references met from the other side — the link form has a reaction, so
/// the shape it refuses is the one left.
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
    let mut declared: Option<(&str, Publishable)> = None;
    let source = crate::region::Source::of(text);
    // The cut owns the table boundary; `names_the_package_table` says which heading is this reader's, which
    // is the only part that was ever specific to it.
    let tables = crate::sections::cut(source.toml().numbered_lines(), |line| {
        is_table(line).then(|| names_the_package_table(line.trim()))
    });
    for (_, line) in tables
        .iter()
        .filter(|table| table.name)
        .flat_map(|table| table.body.iter())
    {
        let trimmed = line.trim();
        // **The key is identified exactly, and it used to be identified by its prefix.** A
        // `strip_prefix("publish")` sent every `[package]` key beginning with those seven letters down this
        // path: `publish-lockfile = true`, which cargo itself once accepted, read as *unreadable manifest*
        // and refused the whole member — and cargo treats a key it does not know as unused and carries on.
        // A prefix is not a key.
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        // `publish.workspace = true` and `publish = { workspace = true }` both defer to the workspace
        // manifest, so neither is a verdict this text carries; the dotted spelling is recognised here and
        // the inline table falls to the value arms below.
        let (head, dotted) = match key.trim().split_once('.') {
            Some((head, _)) => (head, true),
            None => (key.trim(), false),
        };
        if unquoted(head.trim()) != "publish" {
            continue;
        }
        let verdict = if dotted {
            Publishable::Unreadable(trimmed.to_string())
        } else {
            classify(value.trim(), trimmed)
        };
        // Cargo refuses a manifest that declares one key twice, so a reader answering from the first of two
        // would speak for a file cargo will not read at all.
        if let Some((first, _)) = declared {
            return Publishable::Unreadable(format!(
                "two `publish` keys in one `[package]` table ({first}, {trimmed})"
            ));
        }
        declared = Some((trimmed, verdict));
    }
    // An absent key is cargo's default, which is publishable.
    declared.map_or(Publishable::Yes, |(_, verdict)| verdict)
}

/// Whether a table header opens `[package]`, in every spelling cargo honours.
///
/// **Measured on cargo 1.96.0, because the equality this replaced was one spelling of three.** `[ package ]`
/// and `["package"]` are the same table to cargo — each reports `publish=[]` for a `publish = false` beneath
/// it — while `trimmed == "[package]"` skipped both, and a reader that skips the table answers *publishable*
/// for a crate cargo refuses to publish. `[package.metadata]` is a different table and stays one.
fn names_the_package_table(header: &str) -> bool {
    let inner = header.trim_start_matches('[').trim_end_matches(']').trim();
    unquoted(inner) == "package"
}

/// A TOML key or header segment with its quotes removed, in both quoted forms cargo accepts.
///
/// `"publish" = false` and `'publish' = false` are the `publish` key to cargo — measured, each reports
/// `publish=[]` — and a reader comparing the raw text saw neither, which is the direction that answers
/// *publishable* for a crate that publishes nowhere.
pub(crate) fn unquoted(text: &str) -> &str {
    for quote in ['"', '\''] {
        if let Some(inner) = text.strip_prefix(quote).and_then(|r| r.strip_suffix(quote)) {
            return inner;
        }
    }
    text
}

/// What a `publish` value says, once the key is known to be exactly `publish`.
fn classify(value: &str, line: &str) -> Publishable {
    match value {
        "false" => Publishable::No,
        "true" => Publishable::Yes,
        // **The array is decided by its contents, not by matching one spelling of it.** A literal `"[]"` arm
        // stood here and every other bracketed value went to `Yes`, so `publish = [ ]` — one space, legal
        // TOML, and refused by `cargo publish` exactly as `[]` is (measured on cargo 1.96.0: `cargo
        // metadata` reports `[]` and the dry run errors) — was answered *publishable*, in the function
        // written because text readers called the empty array published.
        other if other.starts_with('[') && other.ends_with(']') => {
            if other[1..other.len() - 1].trim().is_empty() {
                Publishable::No
            } else {
                Publishable::Yes
            }
        }
        _ => Publishable::Unreadable(line.to_string()),
    }
}

/// Whether a line of executed TOML opens a table.
///
/// **Both ends, not just the opening bracket.** A bare `starts_with('[')` — which each reader this replaces
/// wrote for itself — also matches a multi-line array's continuation, since `  [1, 2],` trims to something
/// starting with `[`. Requiring the closing `]` refuses that and costs a real heading nothing: the comment a
/// heading may carry (`[package] # …`) is already gone, because this reads a
/// [`toml`](crate::region::Source::toml) region rather than raw text.
///
/// `[table]` and `[[array-of-tables]]` are both headings and both answer `true`; **which** heading matters is
/// the caller's predicate, not this function's.
///
/// **Residue, declared:** a nested array whose last element sits alone on a line — `  [1, 2]` with no
/// trailing comma — still answers `true`. Neither `Cargo.toml` nor `Cargo.lock` writes that shape: cargo
/// generates the lock, and the manifests here carry flat arrays only. Closing it needs value-level structure
/// this reader does not have and no caller has asked for.
pub fn is_table(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with('[') && trimmed.ends_with(']')
}
