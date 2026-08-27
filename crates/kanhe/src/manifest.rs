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

use std::borrow::Cow;

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
    //
    // **A decoder now exists in this module, and a value is still refused rather than decoded -- for a
    // different reason than the one written here before.** That reason was *no decoder, and hand-rolling a
    // TOML grammar is the class `BACKLOG.md` carries*; [`decoded`] closed the first half in the same window,
    // so leaving the old sentence standing would have been a reason that had expired. The reason that holds:
    // a **key** decides which table or which key this is, so misreading one drops a whole table's contents
    // with nothing said, while a **value** is the thing being judged -- refusing it stops the judgement in
    // front of an operator and skips nothing. The refusal here is also measured to be unreached: no tracked
    // manifest carries a backslash in a quoted value. Decoding values would additionally mean finding the
    // closing quote past an escaped one, which is parsing the string rather than decoding a known body.
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
    // **One reader answers the whole question.** `is_table(line).then(…)` asked *is this a heading* twice —
    // once here and once inside `table_heading`, whose `None` arm then became a branch nothing reached. It
    // also put two referents on one name — one predicate meaning *is this a heading* and one method meaning
    // *is this heading that table* — standing a few lines apart at every call site.
    let tables = crate::sections::cut(source.toml().numbered_lines(), |line| {
        table_heading(line).map(|heading| heading.names("workspace.package"))
    });
    // **An undecodable heading is refused rather than skipped.** Its name matches nothing, so the table would
    // be passed over and the version would read `Absent` — the answer that says *nothing declared this* about
    // a manifest whose declaration is simply unreadable here.
    if let Some(line) = source
        .toml()
        .numbered_lines()
        .find_map(|(_, line)| table_heading(line).filter(|h| h.undecodable).map(|_| line))
    {
        return WorkspaceVersion::Unreadable(line.trim().to_string());
    }
    // `version` then `=`, so `version.workspace` and any other `version…` key is not this key.
    let values: Vec<Result<&str, String>> = tables
        .iter()
        .filter(|table| table.name)
        .flat_map(|table| table.body.iter())
        .filter_map(|(_, line)| match assigned(line, "version") {
            Assigned::Value(value) => Some(Ok(value)),
            Assigned::Other => None,
            // A spelling this reader does not decode is not an absent key, and saying so is the whole of
            // `WorkspaceVersion`'s third state. A dotted head naming `version` assigns a field of it —
            // `version.workspace = true` — which is not a version and is not an absent key either.
            Assigned::Field { .. } | Assigned::Unreadable => Some(Err(line.trim().to_string())),
        })
        .collect();
    let values: Vec<&str> = match values.into_iter().collect::<Result<Vec<&str>, String>>() {
        Ok(values) => values,
        Err(written) => return WorkspaceVersion::Unreadable(written),
    };
    // **Two keys refuse rather than the first one answering, which its two siblings already did.** This
    // returned on the first `version` it met. `publishable` states the reason in its own words — *cargo
    // refuses a manifest that declares one key twice, so a reader answering from the first of two would speak
    // for a file cargo will not read at all* — and `package_name` answers the same way. One of three reading
    // the same root manifest disagreed, and taking a value first is what made the count askable.
    if values.is_empty() {
        // **The table can be written as a value inside `[workspace]`, and that is not the same as absent.**
        // Measured under cargo 1.96.0, each resolves a member at `0.5.0`: `[workspace]` with
        // `package.version = "0.5.0"`, and with `package = { version = "0.5.0" }`. Composing a table out of a
        // dotted key path or an inline table is structure this reader does not build — it cuts headings — so
        // the shape is refused where it is met instead of being reported as a declaration nobody made. The
        // message an operator gets then names the line rather than saying *missing or malformed*.
        let workspace = crate::sections::cut(source.toml().numbered_lines(), |line| {
            table_heading(line).map(|heading| heading.names("workspace"))
        });
        //
        // **Only where that key could carry the version.** A review noted the over-refusal: `[workspace]` with
        // `package.authors = […]` and no version anywhere is a manifest whose workspace version genuinely IS
        // absent, and naming that line unreadable would answer the wrong fact. A dotted head is therefore
        // asked for its tail. The inline form is refused whatever it holds, because this reader does not parse
        // an inline table and so cannot tell `package = { version = "0.5.0" }` from
        // `package = { authors = […] }` — refusing there names the line it could not read, which is the fact.
        if let Some(line) = workspace
            .iter()
            .filter(|table| table.name)
            .flat_map(|table| table.body.iter())
            .find(|(_, line)| match assigned(line, "package") {
                Assigned::Field { tail, .. } => tail == "version",
                Assigned::Value(_) => true,
                Assigned::Other | Assigned::Unreadable => false,
            })
            .map(|(_, line)| line)
        {
            return WorkspaceVersion::Unreadable(line.trim().to_string());
        }
    }
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
    // An undecodable heading could be the package table, so the verdict is refused rather than reached by
    // passing over it — measured against cargo, `["\u0070ackage"]` is the package table to it.
    if let Some(line) = source
        .toml()
        .numbered_lines()
        .find_map(|(_, line)| table_heading(line).filter(|h| h.undecodable).map(|_| line))
    {
        return Publishable::Unreadable(line.trim().to_string());
    }
    let tables = crate::sections::cut(source.toml().numbered_lines(), |line| {
        table_heading(line).map(|heading| heading.names("package"))
    });
    for (_, line) in tables
        .iter()
        .filter(|table| table.name)
        .flat_map(|table| table.body.iter())
    {
        let trimmed = line.trim();
        // **Through the shared key reader, which exists in this file to answer exactly this.** This held its
        // own chain — `split_once('=')`, then `split_once('.')`, then `unquoted` — reaching the right answer
        // for `publish` while splitting on the first *raw* `=` and the first raw `.`, where the shared reader
        // cuts outside strings. A second implementation of one predicate, in the same module as the first, is
        // the shape this file spends its history closing; a review found it one round after the reader landed.
        //
        // What each answer means here has not changed. A key spelling this reader cannot decode **might** be
        // `publish` — measured against cargo, `"\u0070ublish" = false` reports `publish=[]`, so passing over
        // it answers *publishable* for a crate cargo refuses. And `publish.workspace = true` assigns a field
        // of the key rather than the key, deferring to the workspace manifest, which is not a verdict this
        // text carries.
        let verdict = match assigned(trimmed, "publish") {
            Assigned::Value(value) => classify(value.trim(), trimmed),
            Assigned::Field { .. } | Assigned::Unreadable => {
                Publishable::Unreadable(trimmed.to_string())
            }
            Assigned::Other => continue,
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

/// A TOML table heading: which table it names, and whether it opens an **array** of tables.
///
/// **One rule, one implementation, because it had five and only one of them was right.** `[ package ]`,
/// `["package"]` and `[package]` are the same table to cargo — measured, each reports `publish=[]` for a
/// `publish = false` beneath it — and a reader comparing raw text saw only the third. That equality was
/// measured once, repaired at one predicate, and left standing at every other cut — the `[workspace.package]`
/// one, `package_name`'s, `dependency_table`, and the lock reader's literal comparison. A review found them.
///
/// **The kind is carried because a flattened name collapsed `[[x]]` into `[x]`.** An array-of-tables heading
/// is a different shape, and the lock file's entries are exactly that — so the reader that cuts them asked for
/// the literal text rather than for this, which is how it stayed a fifth implementation. Both questions are
/// answered here now, and a caller says which it means.
///
/// **Escapes are decoded, because cargo decodes them and a reader that would not was blind to a whole
/// table.** A basic string may spell any part of a name in escapes, and each spelling below was put to
/// cargo rather than reasoned about: `["\u0064ependencies"]` and `[target.x86_64-unknown-linux-gnu."\u0064ependencies"]`
/// both have their `serde` read as a dependency, `["dep\u0065ndencies"]` too, so an escape is not a
/// prefix trick — it can sit anywhere in the name. A reader that answered *undecidable* for a backslash
/// left the pins in that table unread while the manifest beside it kept the aggregate guard satisfied.
/// Decoding is therefore what agrees with cargo, and it is also the smaller answer: the table is
/// classified, so nothing downstream needs a third state to carry.
///
/// **What stays undecodable is a manifest cargo will not read at all.** `["\q"]` is not a table this
/// reader refuses to name — `cargo metadata` rejects the file, naming the escapes it accepts (`b`, `e`,
/// `f`, `n`, `r`, `\\`, `"`, `x`, `u`, `U`). So [`TableHeading::undecodable`] marks a heading no build,
/// no packaging step and no publish could get past, and the readers whose answer turns on a table being
/// *absent* refuse on it rather than reporting nothing declared.
///
/// **A key's boundaries are the dots between keys, and a dot inside a key is not one of them.** TOML reads
/// `["workspace.package"]` as one key named `workspace.package` -- a top-level table -- and `[workspace.package]`
/// as the path `workspace` -> `package`. Measured, cargo agrees: a `version` under the first leaves the package
/// version untouched. So the heading is split at the dots **outside** quotes, each segment is unquoted and
/// decoded, and the segments are kept as segments. A caller asks with a dotted path and it is compared piece by
/// piece.
///
/// The reader before this one split on every dot, decoded, and joined the pieces back with dots -- and the
/// escaped spelling of the separator walked straight through that. `["workspace\u002Epackage"]` carries no
/// literal dot, so it survived the split as one piece, decoded to `workspace.package`, and the join could no
/// longer tell it from the path. Two reviews found it in the same round; cargo reads it as one key, and both
/// `workspace_version` and `dependency_table` answered as though it were the path. The `.` a caller means and
/// the `.` a key contains have to be different things in the representation, or a decoder puts them back
/// together.
///
/// **Declared residue, carried over from the predicate this replaced.** A line that is only an array
/// element -- `[1, 2]` alone, no trailing comma, inside a multi-line array -- reads as a heading named
/// `1, 2`. It names no table any caller asks for, so the effect is that it closes the open table early, and
/// a `[package]` scan holding such a line would drop the keys after it. The statement moved here with the
/// behaviour when its former home was deleted; nothing else in the tree said it.
pub fn table_heading(line: &str) -> Option<TableHeading> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('[')?.strip_suffix(']')?;
    let (array, inner) = match inner.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        Some(inner) => (true, inner),
        None => (false, inner),
    };
    let mut undecodable = false;
    let segments = dotted(inner)
        .into_iter()
        .map(|segment| match unquoted(segment.trim()) {
            Some(name) => name.into_owned(),
            // A segment carrying an escape cargo itself rejects leaves which table this is undecided.
            None => {
                undecodable = true;
                String::new()
            }
        })
        .collect();
    Some(TableHeading {
        array,
        undecodable,
        segments,
    })
}

/// `inner` cut at the dots that separate one key from the next -- the dots outside a quoted segment.
///
/// A quoted key may contain a dot, and that dot is content. The cut asks [`outside_strings`] which positions
/// are table syntax, so the two cannot be confused.
///
/// An unterminated quote yields the rest of the text as one segment. That segment does not *fail* to unquote
/// -- [`unquoted`] finds no closing delimiter to strip and hands back the text as it stands, quote included --
/// so what makes it name nothing is that no caller asks for a path with a quote in it. The distinction is
/// worth the sentence because the first version of it named a failure that does not happen; the case is
/// asserted in `a_quoted_key_carries_its_dots_and_an_array_is_not_a_table` rather than left as this claim.
/// Cargo does not parse such a heading either.
fn dotted(inner: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0;
    for at in outside_strings(inner) {
        if inner[at..].starts_with('.') {
            segments.push(&inner[start..at]);
            start = at + 1;
        }
    }
    segments.push(&inner[start..]);
    segments
}

/// The byte offset of every character of `text` that sits **outside** a TOML string.
///
/// **Two readers needed this fact and only one of them had it.** [`dotted`] tracked which quote it was inside,
/// so a dot in a quoted key is content rather than a separator. The assignment scanner in
/// `release_coherence_gate` asked something weaker -- that the byte before a key be a delimiter -- and a
/// quoted value supplies one: `{ path = "deps, workspace = true", version = "0.2.0" }` is a manifest cargo
/// reads at `0.2.0`, measured, and the scanner found an *offer* inside the path and reported a version it
/// could not read. Its own key boundary test is still needed and is still its own; what it lacked was the
/// lexical state, and there is now one walker for that rather than one reader guessing it.
///
/// A backslash inside a **basic** string escapes the character after it, so a `\"` does not close the
/// string; a literal string has no escapes, so a backslash there is content like any other byte. A delimiter
/// itself is not reported as outside -- nothing asks about the quotes, and a key never begins with one.
pub(crate) fn outside_strings(text: &str) -> Vec<usize> {
    let mut outside = Vec::new();
    let mut quote: Option<char> = None;
    let mut characters = text.char_indices();
    while let Some((at, character)) = characters.next() {
        match quote {
            None => match character {
                '"' | '\'' => quote = Some(character),
                _ => outside.push(at),
            },
            Some('"') if character == '\\' => {
                characters.next();
            }
            Some(open) if character == open => quote = None,
            Some(_) => {}
        }
    }
    outside
}

/// What a table-body line assigns to `key`.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Assigned<'a> {
    /// This line assigns `key`; the text after the `=`, uninterpreted.
    Value(&'a str),
    /// This line assigns a **field** of `key` through a dotted head: the segments after it, joined, and the
    /// text after the `=`.
    ///
    /// `version.workspace = true` is this, with `workspace` as the tail. It is not a value of `key`, and the
    /// readers that want the key's own value refuse on it — but the tail is what lets a reader asking about a
    /// *specific* field ask for it rather than compare spellings of the whole line.
    Field { tail: String, value: &'a str },
    /// This line assigns some other key, or is not an assignment.
    Other,
    /// This line assigns something this reader cannot attribute: a key spelling it does not decode.
    Unreadable,
}

/// The text `line` assigns to `key`, decided by decoding the key rather than by matching its raw text.
///
/// **The heading side of this module decoded and the key side did not, so spellings cargo accepts read
/// as *the key is absent*.** Measured under cargo 1.96.0: `[workspace.package]` with `"version" = "0.5.0"`
/// and with `'version' = "0.5.0"` each resolve a member at `0.5.0`, and `[package]` with `"name" = "m"` names
/// `m` — and each answered `Absent` here, the state both readers' docs reserve for a key that is not there.
/// The message an operator then read was *workspace version is missing or malformed*, about a manifest that
/// declares it plainly.
///
/// **What this owns, exactly.** Every reader asking *does this line assign the key I want* asks here:
/// [`workspace_version`], [`publishable`], and `release_coherence_gate::package_name`. The dependency reader
/// and the lock reader do **not**: they ask *which* key a line assigns, with the key unknown, which is a
/// different question. A first version of this sentence said *one reader owns the question for every table
/// body* — wider than the code, and found by a review that enumerated the walkers instead of grepping for the
/// shape the previous repair had replaced. `BACKLOG.md` carries the general form that would unify them, with
/// the trigger that would earn it.
///
/// A **dotted** head naming `key` — `version.workspace = true`, the spelling every member writes — assigns a
/// field of that key rather than the key, so it is [`Assigned::Field`] carrying the tail. A reader wanting the
/// key's own value refuses on it, because taking `true` as a version would not be visible; a reader asking
/// about that named field asks for the tail instead of comparing spellings of the whole line, which is what
/// the inherit recogniser in `release_coherence_gate` does. A dotted head naming anything else is `Other`,
/// because a member's `[package]` body is full of them and refusing on those would refuse every manifest.
pub(crate) fn assigned<'a>(line: &'a str, key: &str) -> Assigned<'a> {
    let line = line.trim();
    let Some(at) = outside_strings(line)
        .into_iter()
        .find(|at| line[*at..].starts_with('='))
    else {
        return Assigned::Other;
    };
    let (head, value) = line.split_at(at);
    let segments = dotted(head.trim());
    let Some(first) = segments.first().and_then(|first| unquoted(first.trim())) else {
        return Assigned::Unreadable;
    };
    if first != key {
        return Assigned::Other;
    }
    if segments.len() == 1 {
        return Assigned::Value(&value[1..]);
    }
    let tail: Vec<&str> = segments[1..].iter().map(|segment| segment.trim()).collect();
    Assigned::Field {
        tail: tail.join("."),
        value: &value[1..],
    }
}

/// What a TOML table heading names, and whether it opens an array of tables.
#[derive(Debug, PartialEq, Eq)]
pub struct TableHeading {
    /// `true` for `[[name]]`, which is a different shape from `[name]` and not the same table.
    pub array: bool,
    /// `true` when a segment carries an escape cargo itself rejects, so which table this heading names is
    /// undecided -- and, cargo having refused the file, a table in no manifest anything builds from.
    ///
    /// The undecodable segment is then empty, so [`Self::names`] matches nothing, which is silence -- and a
    /// caller whose answer turns on the table being absent has to refuse instead.
    pub undecodable: bool,
    /// The keys this heading names, in order, each unquoted and decoded.
    ///
    /// **Segments rather than one dotted string, because a dotted string cannot hold the difference.**
    /// `["workspace.package"]`, `["workspace\u002Epackage"]` and `["workspace"."package"]` join to the same
    /// text and are two different tables to cargo: the first two are one key carrying a dot, the third is the
    /// path. Kept apart here by how many segments there are, which is a fact the join destroyed.
    segments: Vec<String>,
}

impl TableHeading {
    /// Whether this heading names the ordinary table `name` — not an array of tables of the same name.
    ///
    /// **Named for what it answers.** A predicate asking *is this line a heading* once carried this method's
    /// former name, so one word meant two things standing a few lines apart at every call site — the shape
    /// this repository removes on sight. That predicate is gone: this reader answers both halves, and the
    /// direction that pinned the heading boundary now asks it rather than a second implementation.
    pub fn names(&self, path: &str) -> bool {
        !self.array && self.is(path)
    }

    /// Whether this heading names an array of tables called `path`.
    pub fn names_array(&self, path: &str) -> bool {
        self.array && self.is(path)
    }

    /// The keys this heading names, in order.
    pub(crate) fn segments(&self) -> &[String] {
        &self.segments
    }

    /// Whether the segments are exactly the keys `path` spells.
    ///
    /// `path` comes from this repository's own source and is always bare, so splitting it on every dot is
    /// the whole of its grammar -- where the *heading* needs the quote-aware cut, because it is the side that
    /// may carry a dot inside a key. An undecodable heading names nothing rather than matching a caller who
    /// asked about a shorter path.
    fn is(&self, path: &str) -> bool {
        !self.undecodable && self.segments.iter().map(String::as_str).eq(path.split('.'))
    }
}

/// A TOML key or header segment with its quotes removed and its escapes decoded, or `None` for an escape
/// cargo itself rejects.
///
/// `"publish" = false` and `'publish' = false` are the `publish` key to cargo -- measured, each reports
/// `publish=[]` -- and a reader comparing the raw text saw neither, which is the direction that answers
/// *publishable* for a crate that publishes nowhere.
///
/// **A basic string can spell its own name in escapes, and cargo decodes them, so this decodes them.**
/// Measured against cargo itself: a manifest writing `"\u0070ublish" = false` reports `publish=[]`, and one
/// whose package table is headed `["\u0070ackage"]` reports the package with `publish=[]` too. Stripping the
/// delimiters and stopping there left `\u0070ublish`, which matches nothing -- so the key went unread and the
/// crate answered *publishable* while cargo refuses to publish it.
///
/// An earlier repair reported that as undecodable rather than decoding it, which closed the false answer and
/// opened a false refusal: the escaped-quote cfg spelling `[target."cfg(feature = \"x\")".dependencies]` is
/// a manifest cargo reads -- measured, `serde` arrives with that target -- and any backslash anywhere in the
/// heading refused the whole document. Decoding answers both, and leaves nothing for a caller to propagate.
///
/// **A literal string carries no escapes, measured**: `'\u0070ublish'` reports `publish=None`, a different
/// key to cargo as well as to this reader, and `['other \table']` is a heading cargo reads without complaint.
///
/// A **value** carrying an escape is refused rather than decoded, and that asymmetry is deliberate: a key
/// decides *which table or which key this is*, so misreading one drops a whole table's contents silently,
/// while a value is the thing being judged and refusing it is the fail-closed answer with nothing skipped.
/// `release-coherence` pins that side in its own scenarios.
pub(crate) fn unquoted(text: &str) -> Option<Cow<'_, str>> {
    if let Some(inner) = text.strip_prefix('"').and_then(|r| r.strip_suffix('"')) {
        return decoded(inner).map(Cow::Owned);
    }
    if let Some(inner) = text.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')) {
        return Some(Cow::Borrowed(inner));
    }
    Some(Cow::Borrowed(text))
}

/// A basic string's body with its escapes resolved as cargo resolves them, or `None` for one it rejects.
///
/// The accepted set is cargo's own, read off its refusal of an unknown escape rather than off a TOML
/// revision: `b`, `e`, `f`, `n`, `r`, `\\`, `"`, `xHH`, `uHHHH`, `UHHHHHHHH`. `\e` and `\xHH` are not in
/// TOML 1.0, so choosing the specification over the tool would have refused both, which cargo compiles.
/// `\t` is decoded here too and is *not* in that message: asked of a heading rather than of a package name --
/// where a tab is refused for the name it makes, not for the escape -- cargo accepts it.
///
/// A scalar the digits do not spell is `None` here and rejected by cargo as well: `["\uD800"]` fails with
/// *invalid value, expected unicode hexadecimal value*, so the undecodable answer and the unbuildable
/// manifest are the same set rather than two sets that happen to overlap.
fn decoded(body: &str) -> Option<String> {
    let mut decoded = String::with_capacity(body.len());
    let mut rest = body.chars();
    while let Some(ch) = rest.next() {
        if ch != '\\' {
            decoded.push(ch);
            continue;
        }
        decoded.push(match rest.next()? {
            'b' => '\u{8}',
            'e' => '\u{1b}',
            'f' => '\u{c}',
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '"' => '"',
            '\\' => '\\',
            'x' => scalar(&mut rest, 2)?,
            'u' => scalar(&mut rest, 4)?,
            'U' => scalar(&mut rest, 8)?,
            // Every other escape is one cargo names in its refusal, so the manifest does not parse at all.
            _ => return None,
        });
    }
    Some(decoded)
}

/// The character `digits` hexadecimal digits spell, or `None` where they do not spell one.
///
/// A scalar outside Unicode -- `\uD800`, a lone surrogate -- is `None`; [`decoded`] records the measurement.
fn scalar(rest: &mut std::str::Chars<'_>, digits: usize) -> Option<char> {
    let mut value = 0u32;
    for _ in 0..digits {
        value = value * 16 + rest.next()?.to_digit(16)?;
    }
    char::from_u32(value)
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
