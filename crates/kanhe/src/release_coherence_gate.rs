//! The release-coherence judgement, and one builder for the repository shapes it judges.
//!
//! Shared by the gate (`release_coherence.rs`, which runs it over this repository and over the fixtures of its
//! failure matrix) and by the pins citing this capability's declared bounds. Two constructions of "a
//! repository with a changelog and some machinery" is the twin-drift class this repository keeps closing.
//!
//! It separates a **violation** — the release surfaces disagree — from a **cannot-judge** — an input it could
//! not read. A shallow clone with no release spine, an absent manifest, a layout that moved: none of those say
//! the surfaces disagree, and reporting them as if they did tells a reader to go looking for a disagreement
//! that does not exist.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use crate::refusal::{Refusal, cannot_judge_at, violation_at};
use crate::region::Source;
use crate::sections::Section;

use crate::hermetic_git::fixture as run;
pub use crate::hermetic_git::hermetic;
use crate::manifest::{Quoted, WorkspaceVersion, quoted_value, semver, workspace_version};

fn git(repo: &Path, args: &[&str]) -> Result<String, crate::hermetic_git::Failure> {
    crate::hermetic_git::run(repo, &[], args)
}

fn read(repo: &Path, rel: &str) -> Result<String, Refusal> {
    std::fs::read_to_string(repo.join(rel)).map_err(|err| {
        cannot_judge_at(
            "release-coherence#changelog-or-manifest-unreadable",
            format!("could not read {rel}: {err}"),
        )
    })
}

/// Every value assigned to `key` inside a dependency's value text, recognised as a **table key** rather than
/// as a substring, in the order written.
///
/// **The candidates are a value first, so the caller answers *how many*.** `split("version").nth(1)` read the
/// first occurrence of the bare word on the whole line — the dependency's own name and its path included — so
/// `version-utils = { path = "crates/version-utils", version = "0.5.0" }` answered about the wrong span and
/// produced *has no version pin* in front of the release gate. That is the lossy-selection class
/// [`crate::selection`] exists for, in the file that predates it.
///
/// A key stands alone: what precedes it is a table delimiter or whitespace, and what follows is `=` after
/// optional space. Both halves are required — the first alone still admits `/version`, the second alone still
/// admits a key ending in `version`.
pub(crate) fn inline_assignments(value: &str, key: &str) -> Vec<Quoted> {
    assignments(value, key)
        .into_iter()
        .map(quoted_value)
        .collect()
}

/// The raw text assigned to `key` wherever it stands alone in `value`, in order.
///
/// [`inline_assignments`] is this with [`quoted_value`] over it. The split exists because one assignment this
/// reader needs is **not** a string: `workspace = true` carries a boolean, and reading it as a quoted value
/// answers *unreadable* for the one spelling that is correct. Two scanners for one grammar is the shape this
/// file exists to close, so the scan stayed here and only the interpretation moved out.
/// Whether an inline table carries a field whose key this reader cannot decode.
///
/// The cut and the decode are `manifest`'s; what is local is the question — a caller reading one named key
/// cannot see the fields it did not ask about, and one of those may be the reason the manifest does not parse.
fn undecodable_field(value: &str) -> bool {
    inline_fields(value).into_iter().any(|field| {
        match crate::manifest::assignment(field) {
            crate::manifest::Assignment::KeyUnreadable
            | crate::manifest::Assignment::FieldUnreadable { .. } => true,
            // Structure beneath a field this reader judges — `{ version = "1", version.extra = true }` — is
            // the same shape cargo refuses, in the inline spelling.
            crate::manifest::Assignment::Field { ref name, .. } => Field::of(name).is_some(),
            crate::manifest::Assignment::Key { .. } | crate::manifest::Assignment::None => false,
        }
    })
}

/// The fields an inline table assigns, cut at the commas outside strings.
///
/// **Two adjacent functions opened with these five lines byte-for-byte**, both answering *what are this
/// table's fields* — and a change to one (a nested table, a trailing comma, a value that is an array rather
/// than a table) would have left the other reading a different grammar. That is the two-implementations
/// shape this file has spent five review rounds closing, and it was reintroduced by the fix for a false
/// negative. A caller that is handed something other than an inline table gets the text back as one field,
/// which is what the bare-value spellings need.
fn inline_fields(value: &str) -> Vec<&str> {
    let inner = value.trim();
    let inner = inner
        .strip_prefix('{')
        .and_then(|inner| inner.strip_suffix('}'))
        .unwrap_or(inner);
    crate::manifest::split_outside(inner, ',')
}

fn assignments<'a>(value: &'a str, key: &str) -> Vec<&'a str> {
    // **The inner keys are cut and decoded, where this used to position a raw substring search.** It looked
    // for the key's letters with a delimiter in front and the position outside a string — which is not a
    // narrow match but an *exclusion*: a quoted inner key can never be found, because its letters sit inside
    // the quotes. Measured under cargo 1.96.0, `version = { "workspace" = true }` inherits, and this reader
    // could not see that key at all, so the member was refused for not inheriting. The fields are split at
    // the commas outside strings and each is asked of `manifest::assignment`, so an inner key is decoded
    // exactly as a table-body key is — one reader for both, which is what closed the same asymmetry twice
    // before.
    //
    // An array value carrying commas — `features = ["a", "b"]` — is cut across them, and each piece then
    // fails to be an assignment to `key`. That is why the split is safe without understanding arrays: a
    // fragment answers `None` or another key, never this one.
    inline_fields(value)
        .into_iter()
        .filter_map(|field| match crate::manifest::assigned(field, key) {
            crate::manifest::Assigned::Value(value) => Some(value),
            crate::manifest::Assigned::Field { .. }
            | crate::manifest::Assigned::Other
            | crate::manifest::Assigned::Unreadable => None,
        })
        .collect()
}

/// Whether this dependency takes its requirement from the workspace catalog.
///
/// **`workspace = true` is the only spelling cargo accepts, and it wins over a local `version`.** Measured:
/// `{ workspace = true, version = "2" }` beside a catalog offering `1.0` reports `^1.0` — the catalog answers,
/// not the local key — and `workspace = false` is refused outright with *`workspace` cannot be false*. So a
/// `workspace` assignment that is not `true` is in a manifest nothing builds, and the pin it carries is
/// reported unreadable rather than read past.
fn inheritance<'a>(offers: impl IntoIterator<Item = &'a str>) -> Inheritance {
    let offers: Vec<&str> = offers.into_iter().collect();
    // **One `workspace` key, whose value is `true`, and the cardinality is half of that.** `all` over the
    // values answered *inherits* for `{ workspace = true, workspace = true }` too -- duplicate keys, which
    // TOML itself rejects and cargo refuses to parse, read as one valid declaration. A review found it: the
    // predicate preserved the values and discarded how many there were, which is the same shape as a
    // `Several` state existing for `version` and `path` and not for this. Two of them is malformed rather
    // than emphatic, and malformed is not this reader's to choose from.
    match offers.as_slice() {
        [] => Inheritance::Declared,
        [offer] if offer_value(offer) == "true" => Inheritance::FromCatalog,
        _ => Inheritance::Unreadable,
    }
}

/// One assignment's value, ended where the value ends.
///
/// The scan hands back everything after the `=`, which inside an inline table runs on to the next field or to
/// the closing brace: `{ workspace = true }` yields ` true }`. A quoted value is delimited by its own quotes,
/// so [`quoted_value`] never needed this; a boolean is delimited by the table around it.
fn offer_value(text: &str) -> &str {
    text.split(['}', ','])
        .next()
        .expect("`str::split` yields at least one field")
        .trim()
}

/// One dependency's requirement: the catalog's where it takes the offer, and its own otherwise.
///
/// Every spelling of a dependency -- bare or inline, dotted, and a detailed table -- reaches this
/// with whatever `workspace` assignments they carry, so the offer cannot be recognised in one spelling and
/// missed in another. That divergence is what this file's history is made of.
fn requirement<'a>(
    offers: impl IntoIterator<Item = &'a str>,
    versions: Vec<Quoted>,
    written: &str,
) -> Declared {
    match inheritance(offers) {
        Inheritance::Declared => Declared::of(versions, written),
        Inheritance::FromCatalog => Declared::Inherited,
        Inheritance::Unreadable => Declared::Unreadable(written.trim().to_string()),
    }
}

/// Where a dependency's requirement comes from.
enum Inheritance {
    /// This dependency declares its own requirement.
    Declared,
    /// It takes the one the workspace catalog offers.
    FromCatalog,
    /// It carries a `workspace` value cargo does not accept.
    Unreadable,
}

/// Which dependency table a heading opens, if any.
///
/// **The reader used to look at no heading at all**, which cost it both directions at once. A
/// `[dependencies.alias]` table declares one dependency across its own lines, and none of those lines is a
/// `<family-crate> = …` entry, so the whole declaration — renamed or not — was invisible. And a `[features]`
/// key spelled after a family crate was read as a version requirement, because nothing said which tables hold
/// dependencies.
///
/// A context cargo writes in front of a dependency table is dropped before the heading is classified, so
/// `[target.<triple>.dependencies]`, its `.NAME` form, and `[target.'cfg(…)'.…]` are all read like any other.
/// A cfg expression carrying a **dot** is read too, and used not to be: the heading arrives as segments, so
/// the expression is one key whatever it contains and there is no dot for the context step to land inside.
/// That was the last of a wider declared bound this reader carried; [`dependency_table`] records how it was
/// retired.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Table {
    /// `[dependencies]` and its dev/build siblings: each line names one dependency.
    Entries,
    /// `[dependencies.NAME]`: the whole table is one dependency, named by its heading.
    One(String),
    /// Any other table. Not a source of dependencies, so nothing in it is read as one.
    Other,
}

/// The kinds of table whose entries are dependency declarations.
/// Which tables a caller means by *a dependency*, because this reader's consumers do not all mean the same
/// thing by it.
///
/// **`[workspace.dependencies]` is a catalog, not a dependency.** Measured: a package whose manifest carries
/// `[workspace.dependencies] xuanji = "0.5"` beside `[dependencies] serde_json = "1"` reports exactly one
/// dependency to `cargo metadata`, and it is not `xuanji`. The catalog is what *members* may inherit, and
/// inheriting is something a member does with `xuanji = { workspace = true }` -- not something the table does
/// on its own. One reader answered every consumer with one unqualified list, so an example manifest carrying
/// a catalog entry counted as an example requiring that crate: the per-example guard that exists to refuse an
/// example declaring **no** family dependency could be satisfied by a table cargo does not read as one.
///
/// The subject is a parameter rather than a field on the result, because a field is a thing a consumer may
/// forget to read -- the shape two reviews found in this same reader one round earlier, in an `escaped` flag
/// that only two of its consumers consulted.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Subject {
    /// What this package itself depends on: its own dependency tables and their target-specific variants.
    ///
    /// A catalog is excluded, because a table offering a version to members is not this package requiring it.
    Requires,
    /// What a workspace root offers its members to inherit: `[workspace.dependencies]` and nothing else.
    ///
    /// Two callers ask for this. The root's internal-pin check wants it *together with* `Requires`, because a
    /// path pin lives in whichever table its author reached for -- a fixture pinning `[dependencies.xuanji]`
    /// in a root manifest is what said so, and that caller asks for both rather than this being a third
    /// subject that silently means the union. And [`offered`] wants it alone, to resolve what a dependency
    /// taking `workspace = true` is actually held to.
    Offers,
}

/// Which field of a dependency record a key names.
///
/// **One classifier, because the set was written twice and used at one of three sites.** A `const` listed the
/// judged keys for the dotted-subfield guard while the record builder matched the same names again in a
/// `match`, so adding a field to one and not the other reopens the silent path — and the guard itself reached
/// only one of the three spellings a dependency can be written in. Naming the field once, and asking for it
/// wherever a key is met, is what makes those the same question.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Field {
    Package,
    Version,
    Path,
    Workspace,
}

impl Field {
    /// The field `name` names, or `None` where it names none this reader judges.
    fn of(name: &str) -> Option<Self> {
        match name {
            "package" => Some(Field::Package),
            "version" => Some(Field::Version),
            "path" => Some(Field::Path),
            "workspace" => Some(Field::Workspace),
            _ => None,
        }
    }
}

const DEPENDENCY_KINDS: [&str; 3] = ["dependencies", "dev-dependencies", "build-dependencies"];

/// Which dependency table `heading` opens, if any.
///
/// **The admitted forms are written out, because they are a small grammar and not a set of prefixes.** A
/// heading arrives as its keys, so which of them open a dependency table is a question about key sequences:
/// each kind alone or with a name after it, and each of those again behind a `target.<selector>` — every one
/// put to `cargo metadata`, which reads a dependency of the expected kind and target from each.
/// `[workspace.dependencies]`, alone or with a name, is admitted to one [`Subject`] only: it is a catalog,
/// and a catalog is no dependency of the package whose manifest carries it.
///
/// Two readers died on the way here. The context was first stepped past by `strip_prefix("target.")` and
/// `split_once('.')` over the joined name, which put the cut inside any cfg expression carrying a dot; the
/// repair for *that* was to notice a quote surviving the join and refuse, and the bound it left behind said a
/// pin under such a target went unobserved. Holding the heading as segments left no dot to land inside — the
/// selector is one key whatever it contains — and the bound was retired,
/// `a_pin_under_a_cfg_target_carrying_a_dot_is_read` being what retired it. What replaced the split was a
/// walk that dropped a leading `workspace` and then, independently, a leading `target`, and that composed two
/// contexts cargo never composes; the grammar below is what replaced *that*.
///
/// **`undecodable` is deliberately not consulted here, and that is a bounded residue rather than an
/// oversight.** Two reviews named it: the field is read by `manifest::workspace_version` and
/// `manifest::publishable`, whose answers turn on a table being *absent*, and not here -- so a heading this
/// reader cannot name classifies as `Table::Other` and its entries go unread, with an ordinary dependency
/// table beside it holding the aggregate guard above zero. What bounds it is what is left undecodable once
/// escapes are decoded: an escape cargo itself rejects, measured -- `["\q"]` and `["\uD800"]` both make
/// `cargo metadata` fail. A manifest carrying one reaches no build, no packaging step and no publish, and the
/// examples it would sit in are compiled by the *Examples dogfood* check in the same run.
fn dependency_table(heading: &str, subject: Subject) -> Table {
    // **Through the shared reader, because this compared raw text and the equality it needed was measured
    // elsewhere.** `[ dependencies ]` and `["dependencies"]` are the dependency table to cargo, and stripping
    // the brackets without trimming or unquoting left both as `Table::Other` — a whole dependency table
    // silently unclassified, which the aggregate guard downstream could not see.
    let Some(heading) = crate::manifest::table_heading(heading) else {
        return Table::Other;
    };
    // An array of tables is not a dependency table, whatever it is called.
    if heading.array {
        return Table::Other;
    }
    // **The context is a grammar, not a set of prefixes that may be stripped one after another.** Stripping
    // `workspace` and then `target` independently accepted `[workspace.dev-dependencies]` and
    // `[workspace.target.<triple>.dependencies]`, which cargo gives no dependency meaning at all: measured, a
    // member writing `serde = { workspace = true }` against either fails to load, because inheritance reads
    // `[workspace.dependencies]` and nothing else. Reading a pin out of one of those and refusing the release
    // over it is the false-refusal direction, and this reader carried it from before the segments were
    // segments -- the old prefix walk had the same shape. Each admitted form is now written out, and the
    // forms cargo does not admit fall to the last arm because nothing spells them.
    //
    // The forms cargo does not admit were measured too, as the shape they take rather than as an argument: a
    // member writing `serde = { workspace = true }` against `[workspace.dev-dependencies]` or
    // `[workspace.target.<triple>.dependencies]` fails to load, because inheritance reads
    // `[workspace.dependencies]` and nothing else.
    let keys: Vec<&str> = heading.segments().iter().map(String::as_str).collect();
    match (subject, keys.as_slice()) {
        (Subject::Requires, [kind]) if DEPENDENCY_KINDS.contains(kind) => Table::Entries,
        (Subject::Requires, [kind, named])
            if DEPENDENCY_KINDS.contains(kind) && !named.is_empty() =>
        {
            Table::One((*named).to_string())
        }
        // Only `dependencies` is inheritable; `[workspace.dev-dependencies]` is an unused key to cargo. And
        // only a caller asking what this manifest *pins* wants it: it is no dependency of the package it
        // sits in.
        (Subject::Offers, ["workspace", "dependencies"]) => Table::Entries,
        (Subject::Offers, ["workspace", "dependencies", named]) if !named.is_empty() => {
            Table::One((*named).to_string())
        }
        // `[target.<selector>.…]`, where the selector is one key -- a triple or a cfg expression, whatever it
        // contains, because a heading held as segments has no dot for this step to land inside.
        (Subject::Requires, ["target", _selector, kind]) if DEPENDENCY_KINDS.contains(kind) => {
            Table::Entries
        }
        (Subject::Requires, ["target", _selector, kind, named])
            if DEPENDENCY_KINDS.contains(kind) && !named.is_empty() =>
        {
            Table::One((*named).to_string())
        }
        _ => Table::Other,
    }
}

/// What a dependency declares as its version requirement, or why this reader could not tell.
///
/// **Every consumer answers every state, and the compiler is what asks.** Three call sites read a dependency's pin and each
/// decided the refusal class for itself: two matched exhaustively and the third collapsed to `_ => None`,
/// which reported an *absent* key as one this reader *could not read* — the very distinction its sibling had
/// just been repaired to make. A typed result makes the compiler ask each consumer when a state is added,
/// which is the shape [`PackageName`] and [`crate::manifest::WorkspaceVersion`] already carry in this family.
///
/// [`crate::selection::the_only`] is deliberately not used here, for the reason `manifest.rs` records for its
/// own reader: it reports none and several as one refusal, and here they are different facts — an absent pin
/// is the legal `{ path = "…" }` form, and two are a table this reader may not choose from.
#[derive(Debug, PartialEq, Eq)]
enum Declared {
    /// The value as written.
    Value(String),
    /// The key is absent. Legal for both of this reader's keys: a path-only dependency declares no
    /// `version`, and a registry dependency declares no `path`.
    Absent,
    /// A value this reader cannot read — one not in double quotes — quoted as written.
    Unreadable(String),
    /// More than one such key in one dependency. Malformed, and not this reader's to choose from.
    Several(usize),
    /// The requirement is the one the workspace catalog offers, taken with `workspace = true`.
    ///
    /// Only a *pin* is ever this: a `path` is not inheritable in the spelling this reader meets. Resolved
    /// against the catalog in the same manifest by [`offered`], because every example in this repository is
    /// its own workspace root -- the root manifest says so, and `exclude` keeps them out of this workspace.
    Inherited,
}

impl Declared {
    fn of(mut values: Vec<Quoted>, written: &str) -> Self {
        match values.len() {
            0 => Declared::Absent,
            1 => match values.pop() {
                Some(Quoted::Value(version)) => Declared::Value(version),
                _ => Declared::Unreadable(written.trim().to_string()),
            },
            several => Declared::Several(several),
        }
    }
}

/// Which crate a dependency names, or why this reader cannot say.
///
/// **There is no *absent* state, because absence is not unnameability here.** A dependency declaring no
/// `package` key names the crate by its own key — that is cargo's rule, not a gap — so absence resolves to a
/// name rather than to a missing one. What the rest of the enum carries is one distinct way of *failing* to
/// name it each: a value this reader cannot read, more than one such key, a key that is not bare, and a field
/// whose own key could not be decoded. Each has its own refusal site and its own sentence, because a reader
/// told the wrong cause looks for the wrong thing — which is what this type exists to prevent, and what it
/// did wrong twice while the last two of those were folded into the first.
///
/// It was a `String` with the empty string standing for both of those, in the same struct whose `pin` field
/// had just been given `Declared::{Absent, Unreadable, Several}` for exactly this distinction: one field was typed
/// and its sibling was left as a sentinel, so *several `package` keys* and *a `package` value this reader
/// cannot read* reached the operator as one sentence. The sentinel was not injective either — a literal
/// `package = ""` is a third fact that read as the same state.
#[derive(Debug, PartialEq, Eq)]
enum Package {
    /// The crate this dependency names: its `package` value, or its own key where it declares none.
    Named(String),
    /// A `package` value this reader cannot read — a value not in double quotes.
    Unreadable,
    /// More than one `package` key in one dependency. Malformed, and not this reader's to choose from.
    Several(usize),
    /// A **field** of this dependency has a key this reader cannot decode, so which crate it names is
    /// undecided — whatever its own `package` key or its own spelling says.
    ///
    /// **A separate state because the diagnostic is the point.** Reusing `Unreadable` made the gate say *a
    /// `package` value this check cannot read* about `alias = { version = "0.2", "\q" = "xuanji" }`, which
    /// declares no `package` key at all — sending an operator to look for a key that is not there. That is the
    /// misdirection this crate's own three-state readers exist to prevent, and every other pair of facts in
    /// this crate is typed apart rather than folded.
    FieldUnreadable,
    /// The dependency's own key is not a bare TOML key, so its spelling is not the package name — quoted as
    /// written.
    ///
    /// **The same false negative as a rename, through a second door.** Where a dependency declares no
    /// `package`, its key *is* the identity — and the key was taken as the raw text between the line's start
    /// and its `=`. TOML admits a quoted key, and cargo decodes it: measured, `"serde_json" = "1"` resolves
    /// to a dependency named `serde_json`. So `"xuanji" = "0.0.1"` is a real family requirement whose raw
    /// spelling matches no family member, and the entry was skipped by the same `continue` the sibling
    /// `Named` arm's own comment already describes for `alias = { package = "xuanji", … }`.
    ///
    /// Refused rather than decoded, and **the reason written here first has since expired**: it was *decoding
    /// is TOML string parsing, which `BACKLOG.md` files as its own entry*, and `manifest::decoded` landed in
    /// the same window for table headings. What holds instead is that this is not the same question.
    /// [`is_bare_key`] asks *is this one bare key*; `manifest::unquoted` asks *what does this key spell*, and
    /// substituting the second for the first would take `xuanji.version` -- a dotted key, two keys to TOML --
    /// as a package named `xuanji.version`, matching no family crate and skipped in silence. Composing them
    /// so that only a single quoted segment decodes is more code than this arm, for a refusal that is
    /// *visible*: a cannot-judge stops the gate in front of an operator, where the heading case that
    /// justified the decoder was a silent false negative. Measured before writing and still true: no tracked
    /// manifest carries a non-bare dependency key, so this refuses nothing the tree has.
    KeyUnreadable(String),
}

/// Whether `key` is a bare TOML key — the only spelling this reader takes as a package name.
///
/// TOML's bare keys are ASCII letters, digits, `_` and `-`. Anything else — a quoted key, a dotted key, a
/// key carrying whitespace — is a spelling whose decoded value is not its text, and this reader does not
/// decode. It is asked of the dependency's own key rather than of a value, which is why it is not
/// [`inline_assignments`]'s key recogniser: that one asks whether `version` *opens an assignment* inside an
/// inline table, a different question about a different key.
fn is_bare_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

impl Package {
    /// The identity `values` names for a dependency written under `key`.
    ///
    /// One function rather than the two arms that stood in `declared_dependencies` — the inline form and the
    /// detailed table each resolved this themselves, byte-identical, which is the shape that lets two
    /// readers of one rule disagree. That sharing is what gives the key rule below one home too.
    fn of(mut values: Vec<Quoted>, key: &str) -> Self {
        match values.len() {
            // No `package`, so the KEY is the identity — and only a bare key's text is its name.
            0 if is_bare_key(key) => Package::Named(key.to_string()),
            0 => Package::KeyUnreadable(key.to_string()),
            1 => match values.pop() {
                Some(Quoted::Value(name)) => Package::Named(name),
                _ => Package::Unreadable,
            },
            several => Package::Several(several),
        }
    }
}

/// One dependency a manifest declares: the key it is written under, the package it names, and its pin.
struct Dependency {
    key: String,
    package: Package,
    pin: Declared,
    path: Declared,
}

/// A detailed dependency table being read: one dependency spread over its own lines.
///
/// A named struct rather than a tuple because every field is a `Vec<Quoted>` or a `String`, and a reader
/// arriving at `pending.2` has to count to know which key it holds.
struct Detailed {
    key: String,
    packages: Vec<Quoted>,
    versions: Vec<Quoted>,
    paths: Vec<Quoted>,
    /// The raw text of every `workspace` assignment, which is a boolean rather than a string.
    offers: Vec<String>,
    /// A tail this reader could not decode was met, so which crate this names is undecided.
    field_unreadable: bool,
    written: String,
}

/// Every dependency `text` declares, in both forms cargo admits.
///
/// The inline form (`alias = { package = "xuanji", version = "0.5" }`, or a bare `xuanji = "0.5"`) and the
/// detailed table (`[dependencies.alias]` with its own `package` and `version` lines) are one grammar to a
/// reader that tracks the heading, and [`inline_assignments`] recognises a key the same way in both: at a
/// line's start, or after a table delimiter inside a value.
fn declared_dependencies(text: &str, subject: Subject) -> Vec<Dependency> {
    let mut found = Vec::new();
    // **A detailed table is one dependency spread over its own lines, and that is now what it is.** This
    // walked the manifest with a `Table` cursor and a `pending: Option<Detailed>` flushed at the next
    // heading — the `Option` existed for exactly one reason, that a `[dependencies.NAME]` table's fields
    // arrive across lines and the record could only be filed once the *next* heading proved the table over.
    // With each table carrying its own body, `Detailed` is a local built and filed inside one iteration, so
    // there is no half-built record to hold and no boundary to remember to flush at.
    let tables = crate::sections::cut(
        crate::region::Source::of(text).toml().numbered_lines(),
        // The heading reader answers *is this a heading* itself; asking first made its `None` arm dead.
        |line| crate::manifest::table_heading(line).map(|_| dependency_table(line.trim(), subject)),
    );
    for table in &tables {
        match &table.name {
            Table::Entries => {
                // **A dotted key is one dependency spread over its own lines, which is what the detailed
                // table already is.** `xuanji.path = "crates/xuanji"` with `xuanji.version = "0.5.0"` beneath
                // it is the form a maintainer reaches for — `version.workspace = true` is that spelling in
                // every member's `[package]` table — and reading the two lines as two dependencies is what let
                // a stale pin through: the `path` line carried a path and no version, the `version` line a
                // version and no path, and `require_internal_pins` selects on **path**, so neither was
                // internal to it. Measured before this repair: four correct inline siblings plus a stale
                // dotted pair answered `Ok(())`, where the same staleness written inline is a violation.
                //
                // **Grouped by head key, not repaired per line.** Filing each dotted line as its own record
                // was tried and refused itself: it reports `xuanji.path is pinned to crates/xuanji; expected
                // 0.5.0` — a false refusal of a manifest cargo reads correctly, with the path read as the
                // requirement. That is a defect in its own right, though the Core Contract's *one forbidden
                // bug* is the other direction — a real violation that silently passes — so the head key is
                // the record and the tail names the field.
                //
                // Only `path`, `version` and `package` are read from a tail. Every other dotted key —
                // `features`, `default-features`, `optional` — is ignored exactly as its inline counterpart
                // is, because nothing here judges them.
                let mut dotted: BTreeMap<String, Detailed> = BTreeMap::new();
                for (_, line) in &table.body {
                    let trimmed = line.trim();
                    let Some((key, rest)) = trimmed.split_once('=') else {
                        continue;
                    };
                    let key = key.trim();
                    let inline = rest.trim_start().starts_with('{');
                    if !inline {
                        // **Through the shared reader, because this split the key on the first raw dot.** A
                        // quoted tail then read with its quotes: measured under cargo 1.96.0,
                        // `xuanji."path" = "xuanji"` beside `xuanji.version = "0.5"` is a **path** dependency
                        // with requirement `^0.5`, and this answered *no path* — so `require_internal_pins`
                        // read it as external and skipped it, and a non-exact internal pin passed the release
                        // gate. That is a false negative in front of `cargo publish`, where a version is
                        // yankable and never replaceable, and it is the direction this repository orders above
                        // every other. A review found it in the third round of one asymmetry: the heading
                        // decoded, then the key, then the head — and the tail was still raw.
                        let assignment = crate::manifest::assignment(trimmed);
                        if let crate::manifest::Assignment::Field { name, tail, value } =
                            &assignment
                        {
                            let head = name.as_str();
                            let (tail, rest) = (tail.as_str(), *value);
                            // **The entry is created before the tail is judged, and that ordering is the
                            // point rather than an oversight.** A review read it as manufacturing a
                            // dependency the manifest does not declare, since `xuanji.features = [...]`
                            // with no other line reaches this and inserts a record carrying nothing. Moving
                            // the insert after the `match` was measured against the sibling spelling and is
                            // wrong: `xuanji = { features = [...] }` yields exactly the same record, because
                            // the inline reader takes its key before it reads any field. Skipping here would
                            // make one spelling of one manifest answer `1` and the other `0` — two readers
                            // of one fact reaching different verdicts, which is the class this whole file
                            // exists to close. What declares a dependency is the KEY; the fields say what
                            // kind it is, and this reader judges three of them. The equivalence is held by
                            // `an_unjudged_dotted_tail_declares_as_its_inline_spelling_does` rather than left to
                            // this comment, which is an integration direction because the difference is
                            // only visible through a consumer: with the insert deferred, an example
                            // requiring a family crate through such a tail reports `ok release coherence`
                            // where it must refuse.
                            let entry =
                                dotted.entry(head.to_string()).or_insert_with(|| Detailed {
                                    key: head.to_string(),
                                    packages: Vec::new(),
                                    versions: Vec::new(),
                                    paths: Vec::new(),
                                    offers: Vec::new(),
                                    field_unreadable: false,
                                    written: String::new(),
                                });
                            // A tail of one segment names the field; a longer one is **structure beneath**
                            // it, which cargo refuses — `xuanji.version.extra = true` fails with *cannot
                            // extend value of type string with a dotted key*, measured. The guard for that
                            // shape reached the detailed spelling only when it was first written.
                            let (head, deeper) = match tail.split_once('.') {
                                Some((head, _)) => (head, true),
                                None => (tail, false),
                            };
                            match (Field::of(head), deeper) {
                                (Some(_), true) => {
                                    entry.field_unreadable = true;
                                    entry.paths.push(Quoted::Unreadable);
                                    entry.versions.push(Quoted::Unreadable);
                                }
                                (Some(Field::Path), false) => entry.paths.push(quoted_value(rest)),
                                (Some(Field::Version), false) => {
                                    entry.versions.push(quoted_value(rest))
                                }
                                (Some(Field::Package), false) => {
                                    entry.packages.push(quoted_value(rest))
                                }
                                // `xuanji.workspace = true` is the dotted spelling of taking the offer.
                                (Some(Field::Workspace), false) => {
                                    entry.offers.push(rest.to_string())
                                }
                                (None, _) => continue,
                            }
                            entry.written.push_str(trimmed);
                            entry.written.push(' ');
                            continue;
                        }
                        if let crate::manifest::Assignment::FieldUnreadable { name } = &assignment {
                            // **The head decoded and a tail did not, so this is a field of a named
                            // dependency.** Filed under that name with the field state, where folding it into
                            // the key case reported *a dependency under the key `alias."\q" = "xuanji"`* —
                            // the whole line quoted as its own key, for a problem that is a field.
                            let entry = dotted.entry(name.clone()).or_insert_with(|| Detailed {
                                key: name.clone(),
                                packages: Vec::new(),
                                versions: Vec::new(),
                                paths: Vec::new(),
                                offers: Vec::new(),
                                field_unreadable: false,
                                written: String::new(),
                            });
                            entry.field_unreadable = true;
                            entry.paths.push(Quoted::Unreadable);
                            entry.versions.push(Quoted::Unreadable);
                            entry.written.push_str(trimmed);
                            entry.written.push(' ');
                            continue;
                        }
                        if matches!(assignment, crate::manifest::Assignment::KeyUnreadable) {
                            // A key or tail this reader cannot decode belongs to some dependency and names
                            // some field of it. Both are unknown, so the fields it could have carried are
                            // reported unreadable rather than left absent: a dependency whose path is
                            // *absent* is external and skipped, which is how the shape above reached a
                            // release, and *unreadable* stops in front of an operator instead.
                            let entry =
                                dotted
                                    .entry(trimmed.to_string())
                                    .or_insert_with(|| Detailed {
                                        key: trimmed.to_string(),
                                        packages: Vec::new(),
                                        versions: Vec::new(),
                                        paths: Vec::new(),
                                        offers: Vec::new(),
                                        field_unreadable: false,
                                        written: trimmed.to_string(),
                                    });
                            entry.paths.push(Quoted::Unreadable);
                            entry.versions.push(Quoted::Unreadable);
                            continue;
                        }
                    }
                    // **One undecodable field makes the whole entry unread, identity included.** The first
                    // repair reported the *version* and the *path* unreadable and left `package` to fall back
                    // to the key — so `alias = { version = "0.2", "\q" = "xuanji" }` was a dependency named
                    // `alias`, which the consumer skips as non-family **before** it reads the pin, and another
                    // valid family dependency satisfied the per-example counter: a clean release over a
                    // manifest `cargo metadata` refuses to parse. A review found it, and found that the
                    // direction written for the first repair used a family crate as the outer key, which is
                    // exactly the shape that masks this path.
                    //
                    // The state is computed once and consulted by all three views, rather than each scanning
                    // the table for itself.
                    let undecodable = inline && undecodable_field(rest);
                    let package = if undecodable {
                        Package::FieldUnreadable
                    } else {
                        Package::of(inline_assignments(rest, "package"), key)
                    };
                    // A bare `xuanji = "0.5"` carries its requirement as the value itself; an inline table
                    // carries it under a `version` key.
                    let versions = if inline {
                        inline_assignments(rest, "version")
                    } else {
                        vec![quoted_value(rest)]
                    };
                    // A bare `xuanji = "0.5"` declares no path at all; an inline table carries one under a
                    // `path` key, and a dotted key under its own `.path` line — handled above, where the head
                    // key holds the record.
                    let paths = if inline {
                        inline_assignments(rest, "path")
                    } else {
                        Vec::new()
                    };
                    // **An inner key this reader cannot decode is not an absent one.** `assignments` answers
                    // with the values it could attribute, so a `filter_map` over it erased the undecodable
                    // state `manifest::assignment` had already computed: measured,
                    // `serde = { version = "1.0", "\q" = true }` kept the readable version and dropped the
                    // rest, reporting a clean pin over a manifest `cargo metadata` refuses to parse. The
                    // examples check builds every example and would fail on such a file in the same run — but
                    // a compensating control in another gate is not this gate answering, and the Core
                    // Contract's one forbidden bug is a real violation that silently passes.
                    let (versions, paths) = if undecodable {
                        (vec![Quoted::Unreadable], vec![Quoted::Unreadable])
                    } else {
                        (versions, paths)
                    };
                    found.push(Dependency {
                        key: key.to_string(),
                        package,
                        pin: requirement(assignments(rest, "workspace"), versions, rest),
                        path: Declared::of(paths, rest),
                    });
                }
                for (_, detailed) in dotted {
                    found.push(Dependency {
                        package: if detailed.field_unreadable {
                            Package::FieldUnreadable
                        } else {
                            Package::of(detailed.packages, &detailed.key)
                        },
                        pin: requirement(
                            detailed.offers.iter().map(String::as_str),
                            detailed.versions,
                            &detailed.written,
                        ),
                        path: Declared::of(detailed.paths, &detailed.written),
                        key: detailed.key,
                    });
                }
            }
            Table::One(name) => {
                let mut detailed = Detailed {
                    key: name.clone(),
                    packages: Vec::new(),
                    versions: Vec::new(),
                    paths: Vec::new(),
                    offers: Vec::new(),
                    field_unreadable: false,
                    written: String::new(),
                };
                // **This body is read through the one reader too, and a field it cannot decode marks the
                // record.** It scanned the line once per watched key and kept whatever each scan attributed,
                // so a key it could not decode was filtered out four times over: `[dependencies.alias]`
                // carrying `package = "xuanji"`, `version = "0.5"` and `"\q" = true` produced a readable
                // identity and a readable pin, and a manifest `cargo metadata` refuses to parse reported a
                // clean release. That is the same false negative the inline and dotted spellings each had,
                // at the third producer of one record — a review found it after the other two were closed.
                for (_, line) in &table.body {
                    let trimmed = line.trim();
                    match crate::manifest::assignment(trimmed) {
                        crate::manifest::Assignment::Key { name, value } => {
                            match Field::of(&name) {
                                Some(Field::Package) => detailed.packages.push(quoted_value(value)),
                                Some(Field::Version) => detailed.versions.push(quoted_value(value)),
                                Some(Field::Path) => detailed.paths.push(quoted_value(value)),
                                // `workspace = true` is a boolean, kept raw for `inheritance`.
                                Some(Field::Workspace) => detailed.offers.push(value.to_string()),
                                None => {}
                            }
                        }
                        // **A dotted key whose head is one this reader judges is structure beneath a value,
                        // and cargo refuses it.** Measured: `[dependencies.alias]` carrying `version = "1.0"`
                        // and `version.extra = true` fails with *cannot extend value of type string with a
                        // dotted key*, and discarding it as unrelated kept the readable pin — the same
                        // false-clean class this branch was repaired for, one spelling in. A dotted head
                        // naming anything else stays another key's business.
                        crate::manifest::Assignment::Field { name, .. }
                            if Field::of(&name).is_some() =>
                        {
                            detailed.field_unreadable = true;
                        }
                        crate::manifest::Assignment::Field { .. }
                        | crate::manifest::Assignment::None => {}
                        crate::manifest::Assignment::KeyUnreadable
                        | crate::manifest::Assignment::FieldUnreadable { .. } => {
                            detailed.field_unreadable = true;
                        }
                    }
                    if !trimmed.is_empty() {
                        detailed.written.push_str(trimmed);
                        detailed.written.push(' ');
                    }
                }
                let unreadable = detailed.field_unreadable;
                found.push(Dependency {
                    package: if unreadable {
                        Package::FieldUnreadable
                    } else {
                        Package::of(detailed.packages, &detailed.key)
                    },
                    pin: if unreadable {
                        Declared::Unreadable(detailed.written.trim().to_string())
                    } else {
                        requirement(
                            detailed.offers.iter().map(String::as_str),
                            detailed.versions,
                            &detailed.written,
                        )
                    },
                    path: if unreadable {
                        Declared::Unreadable(detailed.written.trim().to_string())
                    } else {
                        Declared::of(detailed.paths, &detailed.written)
                    },
                    key: detailed.key,
                });
            }
            Table::Other => {}
        }
    }
    found
}

/// What the catalog in `text` offers for `wanted`, for a dependency that took the offer.
///
/// **The catalog is in the same manifest, because every example in this repository is its own workspace
/// root.** The root manifest's own comment says so and `exclude` enforces it, so a dependency spelling
/// `workspace = true` resolves against `[workspace.dependencies]` beside it. Measured: cargo resolves the
/// inline, dotted and detailed spellings of the offer to the catalog's requirement, and it resolves it even
/// when a local `version` sits in the same inline table -- so the catalog is *the* answer rather than one of
/// two. Cargo also refuses a manifest that inherits what its catalog does not declare, which is why
/// [`Offered::Missing`] is a refusal rather than a fallback.
fn offered(text: &str, wanted: &str) -> Offered {
    for Dependency {
        key,
        package,
        pin,
        path: _,
    } in declared_dependencies(text, Subject::Offers)
    {
        match package {
            Package::Named(named) if named == wanted => return Offered::Pin(pin),
            Package::Named(_) => {}
            // An entry whose identity cannot be read might be the one being inherited. *Might be* is not an
            // answer, and skipping it is how a stale pin would reach a release through the catalog.
            Package::Unreadable
            | Package::Several(_)
            | Package::KeyUnreadable(_)
            | Package::FieldUnreadable => {
                return Offered::Unresolvable(key);
            }
        }
    }
    Offered::Missing
}

/// What a catalog offers for one crate.
#[derive(Debug)]
enum Offered {
    /// The catalog declares it, with this requirement -- which may itself be absent, unreadable or several,
    /// and is then answered by the same arms a locally declared one is.
    Pin(Declared),
    /// No catalog entry names it. A manifest cargo refuses to parse.
    Missing,
    /// The catalog carries an entry whose identity this reader cannot resolve, quoted by its key.
    Unresolvable(String),
}

/// Whether `suffix` is an ISO date: three `-`-separated all-digit fields of widths 4, 2 and 2.
///
/// **Parsed, not counted.** The test this replaces asserted the heading was ten characters longer than its
/// own prefix and never read them, so `## [0.5.0] - notadate!!` satisfied *CHANGELOG carries dated release
/// notes*. A length test is a parse without its guarantee.
///
/// **And the day is answered against the calendar, by the reader that owns one.** Reading three all-digit
/// fields of the right widths admitted `2026-99-99` and `0000-00-00`; ranging them to the calendar's outer
/// bounds — a month `1..=12`, a day `1..=31` — still admitted `2026-02-31`, and that residue was recorded
/// here on the ground that a calendar was a dependency this crate's surface did not carry. It does:
/// `reading::date` reads `YYYY-MM-DD` through `days_in_month`, leap years included, and this now delegates to
/// it. Two date readers in one crate, the weaker one used where the stronger existed, is the shape this crate
/// converges rather than documents. What is left of the old residue — `2026-02-31` — is a date a human wrote wrong rather
/// than a shape that reads as one.
pub fn is_iso_date(suffix: &str) -> bool {
    crate::reading::date("changelog section date", suffix).is_ok()
}

/// What a member manifest says its package is called, or why this reader could not tell.
///
/// Typed apart rather than an `Option`, because every consumer here treated `None` as *not a package* and
/// skipped it — so a manifest this reader could not parse left its package's lock version unchecked and its
/// examples' pins unexamined, with the function still returning `Ok`. The template is
/// `capability_subjects::Declared`, applied to a second reader.
pub enum PackageName {
    /// The `[package]` table's `name`.
    Named(String),
    /// No `[package]` table, or no `name` key inside it.
    Absent,
    /// A `name` this reader cannot read: a value not in double quotes, or more than one key in `[package]`.
    Unreadable(String),
}

/// The `[package]` table's `name`, and only that table's.
///
/// **Scoped to the table, not to the first match in the file.** The previous read took the first line whose
/// trimmed start was `name` anywhere in the manifest, which is correct only while `[package]` precedes every
/// other name-bearing table — a premise TOML does not impose and nothing here stated.
/// `crates/tianheng/Cargo.toml` already carries three `name` keys (`[package]`, `[lib]`, `[[bin]]`), so the
/// multiplicity is present in this tree and the read was right by the order they happen to appear in and by
/// the three values happening to agree.
///
/// `the_only` is deliberately **not** used, though this is a class-A shape: it reports none and several as one
/// refusal, and here they are different facts — no `[package]` table means this is not a package manifest,
/// while two `name` keys in one means it is malformed. The consumer needs to tell them apart, so the
/// return carries the distinction instead of collapsing it.
pub fn package_name(manifest: &str) -> PackageName {
    // Executed manifest text. Raw lines were safe against a commented-out `name` only by accident — a
    // `#`-led line matched no key — and not safe at all against a comment on the **table
    // heading**: `[package] # the repository checks` fails `trimmed == "[package]"`, so the table never
    // opens, no `name` is found, and `require_example_pins` answers `cannot_judge` over a legal manifest.
    // Held by `a_package_heading_with_a_trailing_comment_still_opens_the_table`, run against raw lines.
    //
    // A first version of this comment claimed the benefit was at the `name` **value** —
    // `name = "kanhe" # …` supposedly reaching `quoted_value` as `Unreadable`. It never did:
    // `quoted_value` takes the text between the first pair of quotes and discards what follows. The claim
    // was refuted by a reviewer, and stating a benefit a reader could have checked against the function ten
    // lines up is the cheaper half of the discipline the previous commit wrote down.
    let source = crate::region::Source::of(manifest);
    // The cut owns the table boundary. `in_package` was a boolean walked by hand here, in
    // `require_lock_versions`, and once more as a `Table` cursor in `declared_dependencies` — three copies of
    // *a heading opens, the next heading closes*, which is the one thing all three shared. `is_table` says
    // which lines are headings and this predicate says which heading matters; `[package.metadata.docs.rs]` is
    // a different table and names no package.
    let tables = crate::sections::cut(source.toml().numbered_lines(), |line| {
        crate::manifest::table_heading(line).map(|heading| heading.names("package"))
    });
    let names: Vec<Result<&str, String>> = tables
        .iter()
        .filter(|table| table.name)
        .flat_map(|table| table.body.iter())
        // **Through the shared key reader, because this matched the key's raw text.** `[package]` with
        // `"name" = "kanhe"` is the package's name to cargo — measured — and answering `Absent` for it made
        // this say *declares no `[package]` name* about a manifest that declares one, and made the sibling
        // caller fall back to the directory, comparing a member under the wrong identity.
        .filter_map(|(_, line)| match crate::manifest::assigned(line, "name") {
            crate::manifest::Assigned::Value(value) => Some(Ok(value.trim())),
            crate::manifest::Assigned::Other => None,
            crate::manifest::Assigned::Field { .. } | crate::manifest::Assigned::Unreadable => {
                Some(Err(line.trim().to_string()))
            }
        })
        .collect();
    let names: Vec<&str> = match names.into_iter().collect::<Result<Vec<&str>, String>>() {
        Ok(names) => names,
        Err(written) => return PackageName::Unreadable(written),
    };
    match names.len() {
        0 => PackageName::Absent,
        1 => match quoted_value(names[0]) {
            Quoted::Value(name) => PackageName::Named(name),
            Quoted::Unreadable => PackageName::Unreadable(names[0].to_string()),
        },
        several => PackageName::Unreadable(format!("{several} `name` keys in `[package]`")),
    }
}

/// Which phase of the release ritual this repository is in.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Between releases: an adopter-facing `[Unreleased]` entry is required, and lockfile drift is
    /// tolerated as history.
    Development,
    /// The workspace version has moved forward for release preparation, so the dated section, the internal
    /// pins and every workspace entry in `Cargo.lock` must all name it.
    ReleaseReady,
    /// The `release: X.Y.Z` commit itself, held to the same alignment as `ReleaseReady`.
    Snapshot,
}

impl State {
    fn label(self) -> &'static str {
        match self {
            State::Development => "development",
            State::ReleaseReady => "release-ready",
            State::Snapshot => "snapshot",
        }
    }
}

const COMPARE: &str = "https://github.com/tacticaldoll/tianheng/compare";
const RELEASES: &str = "https://github.com/tacticaldoll/tianheng/releases/tag";

/// The release spine, and which phase of the ritual the workspace is in relative to it.
struct Spine {
    /// Which phase the workspace is in, relative to the latest release commit.
    state: State,
    /// The latest `release: X.Y.Z` subject's version.
    release_version: String,
    /// That commit's own date, `YYYY-MM-DD`, which the dated section is held against at the snapshot.
    release_date: String,
    /// The one before it, absent when the latest is the first release.
    previous_release: Option<String>,
}

/// Read the release spine out of the commit log and classify the workspace against it.
///
/// A malformed `release:` subject is a **violation** — the history disagrees with its own form — while an
/// absent spine is a **cannot-judge**, because a shallow clone cannot see one and that is not a disagreement.
fn release_spine(
    repo: &Path,
    version: &str,
    version_parts: (u64, u64, u64),
) -> Result<Spine, Refusal> {
    // `%ad` with `--date=short`, because the dated release section's value is held against the release
    // commit's own date and reading it here costs nothing — the log that answers "which commit" answers
    // "when" in the same line.
    let subjects =
        git(repo, &["log", "--date=short", "--format=%H%x09%ad%x09%s"]).map_err(|err| {
            cannot_judge_at(
                "release-coherence#release-history-unreadable",
                format!("could not read the release history: {err}"),
            )
        })?;
    let mut history: Vec<(String, String, String)> = Vec::new();
    // HEAD's own commit is the first line this log produced, so asking git for it again would be a second
    // read of something already in hand — and a refusal guarding that second read is a branch no input can
    // take. Taken here instead.
    let mut head: Option<String> = None;
    for line in subjects.lines() {
        let Some((commit, rest_of_line)) = line.split_once('\t') else {
            continue;
        };
        let Some((date, subject)) = rest_of_line.split_once('\t') else {
            continue;
        };
        if head.is_none() {
            head = Some(commit.to_string());
        }
        if let Some(rest) = subject.strip_prefix("release: ") {
            if semver(rest).is_none() {
                return Err(violation_at(
                    "release-coherence#release-history-version-malformed",
                    format!("malformed release history subject: {subject}"),
                ));
            }
            history.push((commit.to_string(), date.to_string(), rest.to_string()));
        } else if subject.starts_with("release:") {
            return Err(violation_at(
                "release-coherence#release-history-subject-malformed",
                format!("malformed release history subject: {subject}"),
            ));
        }
    }
    let Some((release_commit, release_date, release_version)) = history.first().cloned() else {
        return Err(cannot_judge_at(
            "release-coherence#release-history-shallow",
            "exact release history is unavailable; fetch full history containing release: X.Y.Z — a shallow \
             clone cannot see the release spine, which is not the same as surfaces that disagree",
        ));
    };
    let previous_release = history.get(1).map(|(_, _, v)| v.clone());
    // A release commit exists, so at least one line of the log parsed, so this is Some. Provable from the
    // loop above rather than assumed about git.
    let head =
        head.expect("the log line that produced a release commit also produced HEAD's own commit");

    let state = if head == release_commit {
        if version != release_version {
            return Err(violation_at(
                "release-coherence#release-snapshot-version-disagrees",
                format!(
                    "release snapshot subject is {release_version} but workspace version is {version}"
                ),
            ));
        }
        State::Snapshot
    } else {
        let released =
            semver(&release_version).expect("the history holds only well-formed versions");
        match version_parts.cmp(&released) {
            std::cmp::Ordering::Less => {
                return Err(violation_at(
                    "release-coherence#workspace-version-behind-latest-release",
                    format!(
                        "workspace version {version} is older than latest release {release_version}"
                    ),
                ));
            }
            std::cmp::Ordering::Equal => State::Development,
            std::cmp::Ordering::Greater => State::ReleaseReady,
        }
    };
    Ok(Spine {
        state,
        release_version,
        release_date,
        previous_release,
    })
}

/// Every version-bearing surface outside the changelog, and the member **names** the later phases read.
///
/// **The `and then` in that sentence is real and the obvious repair for it is not.** A review opened Gate 4
/// on it: this runs the per-member inherit loop and then calls the two pin checks, so its job needs two
/// clauses to state, and its name says *surfaces*. The suggested repair was to have the caller sequence the
/// three, since the `Vec<(String, String)>` return already exists for it — and that return is **not** the
/// manifests those checks consume. It is the `(path, name)` pairs `require_example_pins` produces, which
/// `require_changelog_state` and the lock reader read. The move was made and the failure matrix refused it:
/// `manifests` at the caller became the `(path, text)` list, and the lock check reported *Cargo.lock is
/// missing workspace package* with a whole manifest where a name belongs.
///
/// So the flow stays, and what is worth changing is the thing that made the move look safe: two lists of the
/// same type with different meanings in one function. `BACKLOG.md` carries that with its trigger, rather than
/// a rename of this function that would leave the swap compiling.
fn require_version_surfaces(
    repo: &Path,
    root_manifest: &str,
    version: &str,
) -> Result<Vec<(String, String)>, Refusal> {
    let manifests = workspace_manifests(repo)?;
    for (path, text) in &manifests {
        // Only the refusal message needs the name here — the inheritance read below works off the text
        // whichever state this is, so an unnameable package is reported by path rather than skipped. The
        // third consumer, and the only one for which that is the right answer.
        let name = match package_name(text) {
            PackageName::Named(name) => name,
            PackageName::Absent | PackageName::Unreadable(_) => path.clone(),
        };
        // This reader held its own `split('#')` — the last hand-rolled cut over TOML text outside `region`.
        // Measured, because an earlier wording called it "a fourth spelling of one language's rule": four
        // `split('#')`-shaped sites existed, but the other three read a Markdown heading, a shell command
        // and a URL fragment, so they are not this rule and never were.
        //
        // It was kept out of `region` while `toml()` cut at a token
        // start, because converting it then would have refused `version.workspace = true#c`, which is a
        // legal comment on a line that still inherits. `toml()` now tracks strings and cuts where TOML cuts,
        // so the exception has nothing left to protect and the hand-rolled rule is gone with it.
        //
        // Both directions run through `judge`: `an_inherit_line_with_a_glued_comment_still_inherits` and
        // `a_member_whose_only_inherit_line_is_commented_out_is_refused`.
        // **Through the shared key reader, because string equality recognised one spelling of four.** Measured
        // under cargo 1.96.0, each inherits `0.5.0`: `version.workspace = true`,
        // `version = { workspace = true }`, `"version".workspace = true` and `'version'.workspace = true`. The
        // whitespace-stripped equality took only the first, and the other three reached
        // `member-does-not-inherit-workspace-version` — a `violation_at`, exit 1, over a manifest cargo reads.
        // A false refusal is a defect and this is one; what the Core Contract names as *the one forbidden
        // bug* is the other direction — a real violation that silently passes. The window's own repair two
        // hundred lines up is what made the asymmetry worth naming: the key side
        // decodes now, so a recogniser comparing raw text is the odd one out.
        //
        // Two shapes inherit, and `manifest::assigned` tells them apart: a **dotted** head naming `version`
        // assigns a field of it, and this asks whether that field is `workspace`; or a `version` whose value
        // is an inline table carries the offer inside it. Both comparisons are of **decoded names** — the
        // tail's segments and the inline table's inner keys — which they were not when this comment was first
        // written, and a review refused four more spellings on that account. `manifest::assignment` is where
        // *decoded* became a property of the whole answer rather than of its first segment.
        let inherits = crate::region::Source::of(text.as_str())
            .toml()
            .lines()
            .any(|line| match crate::manifest::assigned(line, "version") {
                // `version.workspace = true`, in any spelling of the two keys.
                crate::manifest::Assigned::Field { tail, value } => {
                    tail == "workspace" && value.trim() == "true"
                }
                // `version = { workspace = true }`: the offer sits inside the value.
                crate::manifest::Assigned::Value(value) => assignments(value, "workspace")
                    .into_iter()
                    .any(|offer| offer_value(offer) == "true"),
                crate::manifest::Assigned::Other | crate::manifest::Assigned::Unreadable => false,
            });
        if !inherits {
            return Err(violation_at(
                "release-coherence#member-does-not-inherit-workspace-version",
                format!("workspace package {name} must inherit version.workspace = true"),
            ));
        }
    }
    require_internal_pins(root_manifest, version)?;
    require_example_pins(repo, &manifests, version)
}

/// The changelog surfaces whose required shape depends on which phase of the ritual this is.
fn require_changelog_state(
    repo: &Path,
    prose: crate::region::Prose<'_>,
    sections: &[Section],
    manifests: &[(String, String)],
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    // The cut answers this. It was a line count over the whole document, which could not tell a real
    // `## [Unreleased]` from one inside a fence — and this is the check the *rest* of this function's
    // reasoning rests on, since every arm below assumes exactly one such section exists.
    let unreleased_sections = sections
        .iter()
        .filter(|section| section.name == "## [Unreleased]")
        .count();
    if unreleased_sections != 1 {
        return Err(violation_at(
            "release-coherence#unreleased-section-not-exactly-one",
            "CHANGELOG must contain exactly one [Unreleased] section".to_string(),
        ));
    }
    let has_item = unreleased_has_item(sections);
    match spine.state {
        State::Development => {
            if !has_item {
                return Err(violation_at(
                    "release-coherence#unreleased-has-no-adopter-narrative",
                    "development requires adopter-facing release narrative under [Unreleased]"
                        .to_string(),
                ));
            }
            let link = format!("[Unreleased]: {COMPARE}/v{version}...HEAD");
            if !prose.lines().any(|line| line.trim_end() == link) {
                return Err(violation_at(
                    "release-coherence#unreleased-comparison-link-wrong",
                    format!(
                        "[Unreleased] comparison link must start at v{version} and end at HEAD"
                    ),
                ));
            }
        }
        State::ReleaseReady | State::Snapshot => {
            if has_item {
                return Err(violation_at(
                    "release-coherence#unreleased-not-empty-in-state",
                    format!(
                        "[Unreleased] must be empty in {} state",
                        spine.state.label()
                    ),
                ));
            }
            // Read off the section's own sentinel line rather than swept for across the document: the
            // derived name drops the ` - DATE` suffix, so this is the one question the name cannot answer and
            // `Section::line` exists for. A sweep also accepted a dated line belonging to no section at all.
            let prefix = format!("## [{version}] - ");
            // **Counted before it is taken, because the first of two answers is not an answer.** This
            // asked `.find()`, so a changelog carrying two `## [{version}]` sections answered from whichever
            // came first: a stale one dated years earlier ahead of the correct one reported *ok release
            // coherence*, and at the snapshot the same selection would compare the stale date against the
            // release commit and refuse naming the wrong line. The sibling check above counts `[Unreleased]`
            // sections and refuses any count but one, saying every arm below assumes exactly one exists —
            // the same assumption was made here and not checked. Four readers in this crate were each given
            // a *several* refusal when someone was in them; this one selects from a document and was never
            // asked.
            let dated: Vec<&str> = sections
                .iter()
                .filter(|section| section.name == format!("## [{version}]"))
                .filter_map(|section| section.line.trim_end().strip_prefix(&prefix))
                .filter(|rest| is_iso_date(rest))
                .collect();
            if dated.len() > 1 {
                return Err(cannot_judge_at(
                    "release-coherence#several-dated-release-sections",
                    format!(
                        "CHANGELOG carries {} dated sections for {version} ({}), so which one records the \
                     release is not this reader's to choose",
                        dated.len(),
                        dated.join(", ")
                    ),
                ));
            }
            let Some(dated) = dated.first().copied() else {
                return Err(violation_at(
                    "release-coherence#dated-release-notes-missing",
                    format!("CHANGELOG is missing dated release notes for {version}"),
                ));
            };
            // **Which date, not merely a date.** `is_iso_date` was hardened twice — parsed rather than
            // counted, then ranged rather than digit-tested — and each step asked a sharper question about
            // the SHAPE. The value was never asked, and the value is what a reader takes the release to have
            // happened on. Three releases got it right by someone remembering; the fourth was prepared with
            // a date four days behind the day it would be cut on, and nothing said so.
            //
            // Only at the snapshot, because that is the first moment the answer exists: before the
            // `release: X.Y.Z` commit there is no release commit to be dated against, and a date written
            // during preparation is an intent rather than a claim. Held here rather than by the wrapper,
            // since the wrapper stands in front of the publish and this is a property of the commit.
            if spine.state == State::Snapshot && dated != spine.release_date {
                return Err(violation_at(
                    "release-coherence#release-date-disagrees-with-its-commit",
                    format!(
                        "CHANGELOG dates {version} at {dated} and its `release: {version}` commit was made \
                         on {} — a reader takes the section's date for the day the release happened",
                        spine.release_date
                    ),
                ));
            }
            let from = if spine.state == State::ReleaseReady {
                Some(spine.release_version.clone())
            } else {
                spine.previous_release.clone()
            };
            let expected = match &from {
                Some(previous) => format!("[{version}]: {COMPARE}/v{previous}...v{version}"),
                None => format!("[{version}]: {RELEASES}/v{version}"),
            };
            if !prose.lines().any(|line| line.trim_end() == expected) {
                return Err(violation_at(
                    "release-coherence#release-comparison-link-wrong",
                    match &from {
                        Some(previous) => {
                            format!(
                                "CHANGELOG comparison link for {version} must start at v{previous}"
                            )
                        }
                        None => format!("first release CHANGELOG link must target v{version}"),
                    },
                ));
            }
            require_lock_versions(repo, manifests, version)?;
        }
    }
    Ok(())
}

/// Each release section's internal consistency: no repeated heading, and every `**BREAKING**` paired with a
/// `### Migration`.
///
/// The vacuity guard this walk once carried is UNREACHABLE and is gone. `## [Unreleased]` is itself a
/// `## [` section, and the exactly-one-`[Unreleased]` check in the caller already refuses a changelog with
/// none — more specifically, and as a violation rather than an undecidable. A guard whose input an earlier
/// check forecloses cannot fire, and keeping it would read as coverage. Found by trying to write its WHEN.
fn require_section_shape(sections: &[Section]) -> Result<(), Refusal> {
    let shape = section_shape(sections);
    let mut duplicates: Vec<String> = shape
        .headings
        .iter()
        .filter(|(_, count)| **count > 1)
        .map(|((section, heading), _)| format!("  {section} repeats `### {heading}`"))
        .collect();
    duplicates.sort();
    if !duplicates.is_empty() {
        return Err(violation_at(
            "release-coherence#changelog-section-repeats-a-heading",
            format!(
                "a CHANGELOG release section repeats a heading, so entries that belong together are split:\n{}",
                duplicates.join("\n")
            ),
        ));
    }
    let mut missing: Vec<&String> = shape
        .breaking
        .iter()
        .filter(|section| {
            !shape
                .headings
                .keys()
                .any(|(s, h)| *s == **section && h == "Migration")
        })
        .collect();
    missing.sort();
    if !missing.is_empty() {
        return Err(violation_at(
            "release-coherence#breaking-without-migration-section",
            format!(
                "a CHANGELOG section marks a change **BREAKING** and carries no `### Migration` section, so what \
             an adopter must do is scattered through the entries or absent:\n{}",
                missing
                    .iter()
                    .map(|s| format!("  {s}"))
                    .collect::<Vec<_>>()
                    .join("\n")
            ),
        ));
    }
    Ok(())
}

/// The adopter-facing narrative names none of this repository's own machinery.
fn require_adopter_narrative(
    repo: &Path,
    sections: &[Section],
    version: &str,
    spine: &Spine,
) -> Result<(), Refusal> {
    let leaked = adopter_cited_machinery(repo, sections, version, spine.state)?;
    if !leaked.is_empty() {
        return Err(violation_at(
            "release-coherence#adopter-entry-names-own-machinery",
            format!(
                "an adopter-facing CHANGELOG entry names this repository's own machinery, which ships in no \
             package and which an adopter can never run — move it under `### Self-governance`, or, where the \
             adopter-relevant fact is genuinely there, state the guarantee and drop the filename:\n{}",
                leaked.join("\n")
            ),
        ));
    }
    Ok(())
}

/// Judge a repository's release state, returning what to report or why it cannot be judged.
///
/// Read-only: it never bumps, commits, tags, or publishes.
pub fn judge(repo: &Path) -> Result<String, Refusal> {
    if !repo.join("Cargo.toml").is_file() {
        return Err(cannot_judge_at(
            "release-coherence#repository-root-has-no-manifest",
            format!("repository root {} has no Cargo.toml", repo.display()),
        ));
    }
    if !repo.join("CHANGELOG.md").is_file() {
        return Err(cannot_judge_at(
            "release-coherence#repository-root-has-no-changelog",
            format!("repository root {} has no CHANGELOG.md", repo.display()),
        ));
    }
    // The cause travels, for the reason its sibling in `publish_source_gate` records: a machine without git
    // was told the repository has no history.
    git(repo, &["rev-parse", "--is-inside-work-tree"]).map_err(|err| {
        cannot_judge_at("release-coherence#git-unrunnable", match err {
            crate::hermetic_git::Failure::Spawn(why) => format!(
                "git could not be run at all ({why}), so whether {} has a history was never asked",
                repo.display()
            ),
            crate::hermetic_git::Failure::Exit { stderr, .. } => format!(
                "repository root {} has no git history: {stderr}",
                repo.display()
            ),
        })
    })?;

    let root_manifest = read(repo, "Cargo.toml")?;
    // Each state is answered separately, and the middle one is why the reader does not collapse them. A value this
    // reader cannot read is not a key that is absent, and it is not a malformed version either: it is legal
    // TOML in a form this reader does not take, and telling an operator their version is *missing* sends them
    // to look for a key that is sitting in front of them.
    let version = match workspace_version(&root_manifest) {
        WorkspaceVersion::Declared(version) => version,
        WorkspaceVersion::Absent => {
            return Err(cannot_judge_at(
                "release-coherence#workspace-version-absent",
                crate::manifest::VERSION_ABSENT,
            ));
        }
        WorkspaceVersion::Unreadable(what) => {
            return Err(cannot_judge_at(
                "release-coherence#workspace-version-unreadable",
                crate::manifest::version_unreadable(
                    &what,
                    "whether every release surface names one version cannot be decided",
                ),
            ));
        }
    };
    let Some(version_parts) = semver(&version) else {
        return Err(cannot_judge_at(
            "release-coherence#workspace-version-malformed",
            crate::manifest::version_malformed(&version),
        ));
    };
    let changelog = read(repo, "CHANGELOG.md")?;
    // Cut **once**, and hand the value down. Four walks in this file each carried their own section cursor
    // over the same predicate; `sections::cut` owns the boundary question and `section_of` the naming one,
    // which is the split `section_of`'s own doc asks for. Over a `Prose` region, so a fenced `## [` heading
    // is not a section — the misread `region`'s header declares for the readers still below.
    let changelog_source = Source::of(changelog);
    let changelog_sections =
        crate::sections::cut(changelog_source.prose().numbered_lines(), section_of);

    // The phases, in the order a reader meets a refusal in. **The order is observable**: a repository with
    // two problems is refused for whichever phase reaches its own first, and the failure matrix asserts the
    // message. So these are a sequence rather than a set, and moving one moves what gets reported.
    let spine = release_spine(repo, &version, version_parts)?;
    let manifests = require_version_surfaces(repo, &root_manifest, &version)?;
    require_changelog_state(
        repo,
        changelog_source.prose(),
        &changelog_sections,
        &manifests,
        &version,
        &spine,
    )?;
    require_section_shape(&changelog_sections)?;
    require_adopter_narrative(repo, &changelog_sections, &version, &spine)?;

    Ok(format!(
        "ok release coherence ({}: {version})",
        spine.state.label()
    ))
}

/// The entries of a directory, with a failure to yield one **propagated** rather than dropped.
///
/// `filter_map(|e| e.ok())` silently shortens the enumeration, and the counters this judgement then reasons
/// from are satisfied by whatever did yield — so a run reports clean over the entry it never saw. One site
/// serves both enumerations, because two calls carrying one message would be shadowing.
fn entries_of(dir: &Path) -> Result<Vec<PathBuf>, Refusal> {
    let listing = std::fs::read_dir(dir).map_err(|err| {
        cannot_judge_at("release-coherence#directory-not-enumerable", format!(
            "found no enumerable directory at {}: {err} — the layout changed or is absent, so what it holds \
             cannot be judged",
            dir.display()
        ))
    })?;
    let mut paths = Vec::new();
    for entry in listing {
        let entry = entry.map_err(|err| {
            cannot_judge_at(
                "release-coherence#directory-entry-unreadable",
                format!(
                    "an entry of {} could not be read while enumerating it: {err}",
                    dir.display()
                ),
            )
        })?;
        paths.push(entry.path());
    }
    paths.sort();
    Ok(paths)
}

fn workspace_manifests(repo: &Path) -> Result<Vec<(String, String)>, Refusal> {
    let crates = repo.join("crates");
    let mut out = Vec::new();
    let dirs = entries_of(&crates)?;
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        if manifest.is_file() {
            let text = std::fs::read_to_string(&manifest).map_err(|err| {
                cannot_judge_at(
                    "release-coherence#crate-manifest-unreadable",
                    format!("could not read {manifest:?}: {err}"),
                )
            })?;
            out.push((
                manifest
                    .strip_prefix(repo)
                    .unwrap_or(&manifest)
                    .display()
                    .to_string(),
                text,
            ));
        }
    }
    if out.is_empty() {
        return Err(cannot_judge_at(
            "release-coherence#no-crate-manifests-found",
            "found no workspace crate manifests under crates/ — the crate layout changed or is absent",
        ));
    }
    Ok(out)
}

/// Every internal path dependency in the root manifest names the workspace version.
///
/// **One reader, and no loop beside it.** This held its own line-oriented scan while the sibling that judges
/// example pins was migrated to `declared_dependencies` in the same window, so the two disagreed
/// observably: the new reader knows the detailed table cargo writes and the old loop did not. Against
/// `[workspace.dependencies.xuanji]` with `path` and `version` on their own lines, the loop selected the
/// **path** line — it carries `path`, `"crates/` and `=` — split it at its `=`, and took `path` for the
/// dependency's name, while the `version` line carrying neither marker was never read. The result was
/// *internal dependency path has no version pin*: a false refusal in front of the release gate, over a
/// manifest cargo reads correctly. Which dependencies exist is one question, and it now has one answer.
///
/// The selection is the dependency's own `path` value rather than the shape of the line it sits on, which is
/// the same correction the sibling made when it stopped keying on the dependency's name.
pub(crate) fn require_internal_pins(root_manifest: &str, version: &str) -> Result<(), Refusal> {
    let mut pins = 0usize;
    for Dependency {
        key,
        package: _,
        pin,
        path,
    } in declared_dependencies(root_manifest, Subject::Requires)
        .into_iter()
        .chain(declared_dependencies(root_manifest, Subject::Offers))
    {
        // A dependency with no path is not an internal one; one whose path this reader cannot name might be,
        // and *might be* is not an answer. The old selection asked whether the **line** carried `path` and
        // `"crates/`, which is why it could not tell a path from the key of the dependency declaring it.
        //
        // **That premise now holds for a reason rather than by luck, and the reason is upstream.** It is true
        // only while every form cargo accepts reaches this loop as *one* dependency carrying its own path.
        // A dotted key did not: `xuanji.path` and `xuanji.version` arrived as two records, one with a path and
        // no version and one with a version and no path, so a stale pin was internal to neither and passed.
        // `declared_dependencies` groups a dotted key under its head now, so *no path* means the dependency
        // declares none rather than that the reader split it in half.
        //
        // **`Package` is deliberately not consulted here, and that asymmetry with `require_example_pins` is
        // earned.** An example depends by registry version and carries no path, so identity is its only
        // selector and all four of its arms are reachable. This selects on path, and identity never
        // participates: measured, a quoted key, a detailed table and a renamed dependency are each already
        // refused by the pin comparison whatever `Package::of` answered about them. Matching it here would add
        // three arms no input can reach — the dead-branch shape this file refuses one read earlier.
        let path = match path {
            Declared::Value(path) => path,
            // An inherited dependency declares no `path` of its own: whatever the catalog offers, this line
            // carries none, which is the same answer as the key being absent rather than a state of its own.
            Declared::Absent | Declared::Inherited => continue,
            Declared::Unreadable(written) => {
                return Err(cannot_judge_at(
                    "release-coherence#dependency-path-unreadable",
                    format!(
                        "dependency {key} declares a `path` this check cannot read ({written}), so whether it \
                     is an internal dependency cannot be decided"
                    ),
                ));
            }
            Declared::Several(several) => {
                return Err(cannot_judge_at(
                    "release-coherence#dependency-declares-several-paths",
                    format!(
                        "dependency {key} declares {several} `path` keys, so where it points is not this \
                     reader's to choose"
                    ),
                ));
            }
        };
        if !path.starts_with("crates/") {
            continue;
        }
        pins += 1;
        match pin {
            Declared::Value(pin) if pin == version => {}
            Declared::Value(pin) => {
                return Err(violation_at(
                    "release-coherence#internal-pin-disagrees",
                    format!("internal dependency {key} is pinned to {pin}; expected {version}"),
                ));
            }
            Declared::Absent => {
                return Err(violation_at(
                    "release-coherence#internal-pin-absent",
                    format!("internal dependency {key} has no version pin"),
                ));
            }
            // The root manifest **is** the workspace, so a dependency here taking `workspace = true` would be
            // inheriting from itself — measured, `cargo metadata` refuses a manifest whose catalog does not
            // declare what it inherits, and a catalog inheriting from itself declares nothing. Refused as
            // undecidable rather than guessed at, in the direction that stops in front of an operator.
            Declared::Inherited => {
                return Err(cannot_judge_at(
                    "release-coherence#internal-pin-inherited",
                    format!(
                        "internal dependency {key} takes its version from the workspace catalog, and this is \
                     the manifest that declares the catalog, so what holds it cannot be decided"
                    ),
                ));
            }
            Declared::Unreadable(written) => {
                return Err(cannot_judge_at(
                    "release-coherence#internal-pin-unreadable",
                    format!(
                        "internal dependency {key} declares a version this check cannot read ({written}), so \
                     whether it names the workspace version cannot be decided"
                    ),
                ));
            }
            Declared::Several(several) => {
                return Err(cannot_judge_at(
                    "release-coherence#internal-pin-several",
                    format!(
                        "internal dependency {key} declares {several} `version` keys, so which one it names is \
                     not this reader's to choose"
                    ),
                ));
            }
        }
    }
    // **Already per document, which is why this counter stays where it is.** A review read it as an aggregate
    // over every crate and asked for the treatment `require_example_pins` got — but that function walks a
    // directory of examples and this one reads the workspace ROOT manifest alone, so its loop is over one
    // document's `[workspace.dependencies]` entries. The granularity a partial read would need is already the
    // granularity it has: nothing else's success can keep this count non-zero.
    if pins == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-internal-path-dependency-found",
            "found no internal path dependency in Cargo.toml — the declaration form changed, so pin \
             coherence would be reported over nothing",
        ));
    }
    Ok(())
}

/// Returns the `(path, package)` each workspace manifest names, because resolving them is the first thing
/// this does and the lock reader needed exactly that list. It re-read the manifests instead, which gave it
/// two refusals for a name this reader had already refused on — branches nothing could reach.
pub(crate) fn require_example_pins(
    repo: &Path,
    manifests: &[(String, String)],
    version: &str,
) -> Result<Vec<(String, String)>, Refusal> {
    // A manifest whose package this reader cannot name is not a crate the examples may quietly skip: it would
    // drop out of `family`, and every example pinning it would then pass the `!family.iter().any(…)` filter
    // below without being examined. The two vacuity guards in this function are aggregate, so every crate but
    // one parsing keeps them silent while that one goes unchecked — which is the partial case a vacuity guard
    // is exactly unable to see.
    let mut members: Vec<(String, String)> = Vec::new();
    for (path, text) in manifests {
        match package_name(text) {
            PackageName::Named(name) => members.push((path.clone(), name)),
            PackageName::Absent => {
                return Err(cannot_judge_at(
                    "release-coherence#crate-package-name-absent",
                    format!(
                        "{path} declares no `[package]` name, so whether an example pins it cannot be decided"
                    ),
                ));
            }
            PackageName::Unreadable(what) => {
                return Err(cannot_judge_at(
                    "release-coherence#crate-package-name-unreadable",
                    format!(
                        "{path} declares a `[package]` name this check cannot read ({what}), so whether an \
                     example pins it cannot be decided"
                    ),
                ));
            }
        }
    }
    let family: Vec<String> = members.iter().map(|(_, name)| name.clone()).collect();
    let minor = version
        .rsplit_once('.')
        .map(|(head, _)| head)
        .unwrap_or(version);
    let mut example_manifests = 0usize;

    let dirs = entries_of(&repo.join("examples"))?;
    for dir in dirs {
        let manifest = dir.join("Cargo.toml");
        // Absent is not unreadable. Skipping both alike let the remaining readable examples satisfy the
        // counters below, so the judgement reported clean over the very manifest it could not read.
        if !manifest.is_file() {
            continue;
        }
        let text = std::fs::read_to_string(&manifest).map_err(|err| {
            cannot_judge_at(
                "release-coherence#example-manifest-unreadable",
                format!(
                    "could not read the example manifest {}: {err}",
                    manifest.display()
                ),
            )
        })?;
        example_manifests += 1;
        // **Counted per example, because the aggregate could not see a partial read.** This counter used to
        // live outside the loop, so seven examples parsing kept it non-zero while an eighth went unexamined
        // — the partial case this function's own header names, and the half that let a renamed and then a
        // quoted family key each reach a release as clean. One example is one subject: whatever the reader
        // failed to see there is invisible to every other example's success.
        let mut requirements_here = 0usize;
        let name = dir
            .file_name()
            .expect("a `read_dir` entry always has a file name")
            .to_string_lossy()
            .into_owned();
        // Executed text, for the reason `require_internal_pins` records: a commented-out family pin
        // would otherwise be read as a declared one.
        for Dependency {
            key,
            package,
            pin,
            path: _,
        } in declared_dependencies(&text, Subject::Requires)
        {
            // **Which crate a dependency names is its `package` field where it has one, and its key only
            // otherwise.** Keying on the name alone was a false negative of the class the Core Contract
            // forbids: cargo renames with `alias = { package = "xuanji", version = "stale" }`, `alias` is in
            // no family, and the entry was skipped entirely — while the aggregate `requirements` counter
            // stayed non-zero on the strength of the other examples. The sibling `require_internal_pins`
            // never had this hole because it keys on the PATH, which a rename cannot move; examples depend
            // by registry version and have no path, so the identity has to be read.
            let package = match package {
                Package::Named(package) => package,
                Package::Unreadable => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-package-value-unreadable",
                        format!(
                            "example {name} declares `{key}` with a `package` value this check cannot read, so \
                         which crate it requires cannot be decided"
                        ),
                    ));
                }
                Package::FieldUnreadable => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-dependency-field-unreadable",
                        format!(
                            "example {name} declares `{key}` with a field whose key this check cannot decode, \
                         so what it requires cannot be decided"
                        ),
                    ));
                }
                Package::Several(several) => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-declares-several-packages",
                        format!(
                            "example {name} declares {several} `package` keys for `{key}`, so which crate it \
                         requires is not this reader's to choose"
                        ),
                    ));
                }
                // **Refused here rather than filtered below, because below is where the false negative was.**
                // A key this reader cannot decode names some crate, and which one is exactly what it cannot
                // say — so it can neither be matched against the family nor passed over. Passing over is what
                // it did: the raw spelling matched no member and `continue` dropped the entry, while the
                // aggregate counter stayed non-zero on the strength of the other examples.
                Package::KeyUnreadable(written) => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-dependency-key-unreadable",
                        format!(
                            "example {name} declares a dependency under the key {written}, which is not a bare \
                         TOML key — cargo decodes such a key and this check does not, so whether it requires a \
                         family crate cannot be decided. Write it bare, or give it an explicit `package = \
                         \"…\"`"
                        ),
                    ));
                }
            };
            if !family.contains(&package) {
                continue;
            }
            // The entry is already known to name a family crate, so every way of failing to read its pin is
            // answered on its own terms. Collapsing them was the defect: an ABSENT `version` — legal, since
            // a path-only dependency declares none — was reported as one this reader could not read.
            // **The offer is resolved before the arms below, so every way of failing to read a pin keeps one
            // home.** A dependency taking `workspace = true` declares no `version` of its own, and the reader
            // filed that as `Absent` -- the state meaning *nothing holds this to a version* -- so an example
            // whose pin is held exactly was refused for having none. Cargo holds it to the catalog's
            // requirement, measured; the catalog is read here and its pin is judged as if written inline.
            let pin = match pin {
                Declared::Inherited => match offered(&text, &package) {
                    Offered::Pin(offered) => offered,
                    Offered::Missing => {
                        return Err(cannot_judge_at(
                            "release-coherence#example-inherits-what-no-catalog-offers",
                            format!(
                                "example {name} requires {package} from the workspace catalog, and no \
                             `[workspace.dependencies]` entry beside it names that crate, so what holds it \
                             cannot be decided"
                            ),
                        ));
                    }
                    Offered::Unresolvable(entry) => {
                        return Err(cannot_judge_at(
                            "release-coherence#example-catalog-entry-unresolvable",
                            format!(
                                "example {name} requires {package} from the workspace catalog, whose entry \
                             {entry} names a crate this check cannot resolve, so what holds it cannot be \
                             decided"
                            ),
                        ));
                    }
                },
                declared => declared,
            };
            let pin = match pin {
                Declared::Value(pin) => pin,
                Declared::Absent => {
                    return Err(violation_at(
                        "release-coherence#example-pin-absent",
                        format!(
                            "example {name} requires {package} with no version, so nothing holds it to the \
                         workspace version {version}"
                        ),
                    ));
                }
                Declared::Unreadable(written) => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-pin-unreadable",
                        format!(
                            "example {name} requires {package} with a version this check cannot read \
                         ({written}), so whether it satisfies the workspace version cannot be decided"
                        ),
                    ));
                }
                Declared::Several(several) => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-declares-several-pins",
                        format!(
                            "example {name} declares {several} `version` keys for {package}, so which one it \
                         requires is not this reader's to choose"
                        ),
                    ));
                }
                // Reached when the catalog entry resolved above **itself** takes the offer: a catalog
                // inheriting from itself, which cargo refuses to parse. Named rather than followed, because
                // following it is a loop with no end that a manifest could not have built anyway.
                Declared::Inherited => {
                    return Err(cannot_judge_at(
                        "release-coherence#example-catalog-entry-inherits",
                        format!(
                            "example {name} requires {package} from the workspace catalog, whose own entry \
                         takes its version from the catalog, so what holds it cannot be decided"
                        ),
                    ));
                }
            };
            requirements_here += 1;
            if pin != minor && pin != version {
                // The package, and the key where they differ: a renamed dependency reported by its key alone
                // sends a reader looking for a crate the manifest does not name.
                let named = if package == key {
                    package.clone()
                } else {
                    format!("{package} (as `{key}`)")
                };
                return Err(violation_at(
                    "release-coherence#example-pin-disagrees",
                    // **What was measured, not what a reader might infer.** The rule is string equality
                    // against the two spellings the release surfaces are held to; it does not evaluate a
                    // semver requirement. So `= "^0.5"`, which `0.5.0` genuinely satisfies, is refused —
                    // correctly by the rule and falsely by a sentence saying it is not satisfied, which
                    // sends a maintainer to check semver instead of changing the spelling.
                    format!(
                        "example {name} requires {named} = \"{pin}\"; this check admits only \"{version}\" \
                     or \"{minor}\", the two spellings the release surfaces are held to"
                    ),
                ));
            }
        }
        if requirements_here == 0 {
            return Err(cannot_judge_at(
                "release-coherence#example-requires-no-family-crate",
                format!(
                    "example {name} declares no family dependency requirement this check could read, so its \
                 pins would be reported over nothing. Either it requires no family crate — which is not an \
                 example of this family — or it declares one in a form this reader did not see"
                ),
            ));
        }
    }
    if example_manifests == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-example-manifests-found",
            "found no example manifests under examples/ — the layout changed or is absent",
        ));
    }
    // **The aggregate guard is gone rather than kept beside this one, because no input can reach it.** With
    // every example refusing on its own zero, a run that gets past the loop has `example_manifests` examples
    // each contributing at least one requirement; a run with none is the guard above. Keeping it would be the
    // dead branch this file already refuses one read earlier — *a branch no input can take, which is dead code
    // rather than a guard*. Its WHEN moved rather than vanished: the fixture that reached it, one example
    // requiring no family crate, now reaches the per-example refusal, and the direction that pinned it is
    // rewritten onto the new site rather than deleted.
    Ok(members)
}

/// **Names resolved once, by the reader that already had to resolve them.** This re-read every manifest's
/// `[package]` name and carried its own refusals for a name that is absent or unreadable — branches no input
/// could reach, because `manifests` exists only if the example-pin reader resolved every one of those names
/// first and refused otherwise. Two readers asking one question of one input is the shape this file has
/// spent the window removing; the dead branches were what it looked like from inside.
fn require_lock_versions(
    repo: &Path,
    members: &[(String, String)],
    version: &str,
) -> Result<(), Refusal> {
    let lock = read(repo, "Cargo.lock")?;
    // **Every entry under a name, and whether each carries a `source`.** A single-valued map keyed on the
    // name kept the first entry and dropped the rest, which is only right while no name appears twice — and
    // two entries under one name is ordinary in a lock file, either as two versions of one crate or as a
    // workspace member sharing a name with something from a registry. Nothing here stated that premise, and
    // `source` is what tells the two apart: a workspace member has none, everything fetched has one.
    let mut entries: BTreeMap<String, Vec<(String, bool)>> = BTreeMap::new();
    // **A block's fields are the block's by construction, where they were the block's by a boundary rule.**
    // This walked the lock with `name`, `version_of` and `sourced` as function-level state and a `close`
    // closure called on *every* table header — because `[[patch.unused]]`, written whenever a `[patch]`
    // section exists, carries its own `name`, `version` and `source`, and read as ordinary content it
    // overwrote the block above. That rule was correct and it was a *rule*: drop the call on the foreign
    // header and the fields bleed again.
    //
    // The cut gives each `[[package]]` its own body, so the three values are per-block locals and a foreign
    // table's keys are not reachable from here at all. Filing still happens after the body rather than when
    // the version is read — `source` is written after `version` in cargo's own output, and filing early would
    // record every entry as source-less — but *after the body* is where the block ends now, rather than where
    // the next header happens to be.
    //
    // The header stays an exact `[[package]]` rather than sharing the manifest readers' tolerance for
    // `[ package ]`: cargo generates this file and writes one spelling, so admitting others here would be a
    // tolerance no measurement asked for.
    //
    // **Every entry under a name, and whether each carries a `source`.** A single-valued map keyed on the
    // name kept the first entry and dropped the rest, which is only right while no name appears twice — and
    // two entries under one name is ordinary in a lock, either as two versions of one crate or as a workspace
    // member sharing a name with something from a registry. `source` tells the two apart: a workspace member
    // has none, everything fetched has one.
    let blocks = crate::sections::cut(
        crate::region::Source::of(lock.as_str())
            .toml()
            .numbered_lines(),
        // Through the shared reader, which is what makes the array-of-tables shape a question rather than a
        // literal: this was the fifth place deciding which table a heading names, and the only one whose
        // subject was an array.
        |line| crate::manifest::table_heading(line).map(|heading| heading.names_array("package")),
    );
    for block in blocks.iter().filter(|block| block.name) {
        let mut name = String::new();
        let mut version_of: Option<String> = None;
        let mut sourced = false;
        for (_, line) in &block.body {
            let trimmed = line.trim();
            // **The key is identified exactly, and `=` is decided once.** Each arm used to ask
            // `starts_with(..) && contains('=')` and then split again with an `unwrap_or_default()` the
            // `contains` had already made unreachable — two decisions about the same character and a default
            // nothing could reach. A prefix is also not a key: `versionx = 1` would have entered the version
            // arm, and cargo treats a key it does not know as unused.
            // **Through the one reader, which is what removes the ordering premise below.** This split the
            // line and picked out the keys it cared about, accumulating as it went — the shape the three
            // dependency producers were converted away from, and the last of its kind in this file. It is
            // what let `"version" if !name.is_empty()` be written: an unstated requirement that `name`
            // appear *before* `version` inside a `[[package]]` block. Measured, a block writing them in the
            // other order dropped the version and reached *Cargo.lock is missing workspace package xuanji*,
            // exit 1, about a lock recording it two lines apart. Cargo writes `name` first, so no lock it
            // writes fires this — but nothing said so, and an undeclared stop is a defect rather than
            // governed policy.
            //
            // A key carrying an escape cargo itself rejects is in a file cargo could not have written, and a
            // lock file is written by cargo alone — so that arm is the shape's, not an instance's.
            let (key, value) = match crate::manifest::assignment(trimmed) {
                crate::manifest::Assignment::Key { name, value } => (name, value),
                crate::manifest::Assignment::Field { .. }
                | crate::manifest::Assignment::KeyUnreadable
                | crate::manifest::Assignment::FieldUnreadable { .. }
                | crate::manifest::Assignment::None => continue,
            };
            match key.as_str() {
                "source" => sourced = true,
                "name" => {
                    // An unreadable name defaulted to the empty string, which the `!name.is_empty()` guard
                    // below then read as *no package here* — so that entry's version never entered the map
                    // and the workspace lookup reported it missing, or found a stale one under the previous
                    // name.
                    match quoted_value(value) {
                        Quoted::Value(value) => name = value,
                        Quoted::Unreadable => {
                            return Err(cannot_judge_at(
                                "release-coherence#lock-package-name-unreadable",
                                format!(
                                    "Cargo.lock carries a package name this check cannot read ({}), so the \
                                     versions it records cannot be compared",
                                    trimmed
                                ),
                            ));
                        }
                    }
                }
                // **Unguarded, and the block-level test below decides whether anything is recorded.** The
                // guard here required a name already read, which is the ordering premise. A top-level
                // `version = 4` sits outside every `[[package]]` block and is dropped by the block filter,
                // not by this arm — measured against a lock carrying that line.
                "version" => match quoted_value(value) {
                    Quoted::Value(value) => {
                        version_of = Some(value);
                    }
                    Quoted::Unreadable => {
                        return Err(cannot_judge_at(
                            "release-coherence#lock-version-unreadable",
                            format!(
                                "Cargo.lock records a version for {name} that this check cannot read ({}), \
                                 so whether it matches the workspace cannot be decided",
                                trimmed
                            ),
                        ));
                    }
                },
                _ => {}
            }
        }
        if !name.is_empty() {
            if let Some(found) = version_of {
                entries.entry(name).or_default().push((found, sourced));
            }
        }
    }

    for (_path, package) in members {
        let package = package.clone();
        // A workspace member is the entry with no `source`. Selecting by name alone would compare against a
        // registry entry that merely shares the name, which reads as a version disagreement that is not one.
        let mut local = entries
            .get(&package)
            .into_iter()
            .flatten()
            .filter(|(_, sourced)| !sourced)
            .map(|(found, _)| found);
        let first = local.next();
        let extra = local.count();
        if extra > 0 {
            return Err(cannot_judge_at(
                "release-coherence#lock-several-sourceless-entries",
                format!(
                    "Cargo.lock carries {} entries for {package} with no source, so which one is the workspace \
                 member is not decided",
                    extra + 1
                ),
            ));
        }
        match first {
            None => {
                return Err(violation_at(
                    "release-coherence#lock-missing-workspace-package",
                    format!("Cargo.lock is missing workspace package {package}"),
                ));
            }
            Some(found) if found != version => {
                return Err(violation_at(
                    "release-coherence#lock-package-version-disagrees",
                    format!("Cargo.lock package {package} is {found}; expected {version}"),
                ));
            }
            Some(_) => {}
        }
    }
    Ok(())
}

/// Whether the `[Unreleased]` section carries an adopter-facing item.
///
/// **The boundary question is the cut's now.** This ran an `inside: bool` that opened on the sentinel and
/// closed on the next `## [` — the same shape `wrapper_parser::parser_arms` was repaired for, correct here
/// only because Markdown headings do not nest. `section_of`'s doc left it alone deliberately, since folding a
/// boundary into a naming function makes one function answer two; `sections::cut` answers the boundary half
/// instead, so nothing here decides where a section ends.
///
/// `any` over the matching sections rather than the first of them: the caller refuses a changelog with more
/// than one before reaching here, so the two agree — and reading *the first* would be a choice this function
/// has no reason to make.
fn unreleased_has_item(sections: &[Section]) -> bool {
    sections
        .iter()
        .filter(|section| section.name == "## [Unreleased]")
        .any(|section| {
            section.body.iter().any(|(_, line)| {
                let trimmed = line.trim_start();
                trimmed.starts_with("- ") || trimmed.starts_with("* ")
            })
        })
}

struct Shape {
    headings: BTreeMap<(String, String), usize>,
    breaking: BTreeSet<String>,
}

/// The release section a `## [` heading names, with any ` - DATE` suffix dropped.
///
/// **One derivation.** It was written twice, byte-identical, in `section_shape` and
/// `adopter_cited_machinery` — the shape this file's
/// own header says it exists to close, in the file that says it. A third walk decides section boundaries by
/// a different rule again and is left alone deliberately: `unreleased_has_item` asks *where does
/// `[Unreleased]` end*, which is a boundary question, not a naming one, and folding it in would make one
/// function answer two.
fn section_of(line: &str) -> Option<String> {
    line.starts_with("## [").then(|| {
        line.split_once(" - ")
            .map_or(line, |(section, _)| section)
            .trim_end()
            .to_string()
    })
}

/// The document's grammar — which headings each release section carries, and which sections mark a break.
///
/// It once also collected the section names themselves. Nothing read them: `judge` consumes the headings and
/// the breaking set, so the collection was computed and discarded. `dead_code` cannot see that — `insert` counts
/// as a use of the field — which is why a `-D warnings` workspace passed over it.
///
/// The line between this and an entry's *content* is where the decidable stops: whether an entry is accurate,
/// whether "no adopter action" is true, whether a named symbol exists are judgements over prose, and the
/// detector they would need is the one this repository measured three times and rejected.
fn section_shape(sections: &[Section]) -> Shape {
    let mut shape = Shape {
        headings: BTreeMap::new(),
        breaking: BTreeSet::new(),
    };
    // The section heading itself is not in `body`, so the arms below cannot see it — which the cursor form
    // had to arrange with a `continue` that stood on its own and could be deleted without anything noticing.
    for section in sections {
        for (_, line) in &section.body {
            if let Some(heading) = line.strip_prefix("### ") {
                *shape
                    .headings
                    .entry((section.name.clone(), heading.trim_end().to_string()))
                    .or_default() += 1;
            }
            if line.contains("**BREAKING**") {
                shape.breaking.insert(section.name.clone());
            }
        }
    }
    shape
}

/// Every word that names this repository's own machinery: a tracked path under any package the workspace
/// does not publish, or under `scripts/`, plus the ancestor directories that enumeration derives.
///
/// **The corpus is produced from the manifests, not from a location.** It was `git ls-files scripts/` — which
/// was right when the machinery *was* fourteen shell gates, and stopped being right in the window that
/// deleted them and moved the machinery into `crates/kanhe/**` and `crates/shengmo/**`. `scripts/` now names
/// two wrappers, so the check whose violation message reads *machinery, which ships in no package and which
/// an adopter can never run* had been left pointing at the old address, and the property it holds — an
/// adopter-facing entry naming something an adopter cannot run — went unobserved for everything that moved.
/// `publish = false` is the same criterion the message states, read from the build rather than from a path.
///
/// **A basename enters only when it is unique across the whole tree.** Measured when this widened: the
/// machinery was 78 tracked paths against 182 published ones, and five basenames appeared on both sides —
/// `Cargo.toml`, `README.md`, `bounds.rs`, `lib.rs`, `mod.rs`. Admitting those would refuse an adopter-facing
/// entry for naming a published crate's own source, which is the opposite of this check's purpose. A full
/// path is unambiguous and always enters; a basename is a convenience that has to earn its place, and the
/// same rule governs the ancestor directories the enumeration derives — `crates/` leads to both sides.
fn machinery_names(repo: &Path) -> Result<BTreeSet<String>, Refusal> {
    let metadata = cargo_metadata(repo)?;
    // **The prefix comes from cargo, not from the caller's path.** `manifest_path` is canonical, while the
    // live call site passes `PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")`, which renders with its
    // `..` components intact — so stripping `repo.display()` failed for **all eight** members, machinery
    // collapsed to the two `scripts/` files, `published` stayed empty, and two `continue`s made it silent.
    // `workspace_root` is cargo's own answer for the tree it just described, so the two strings cannot
    // disagree about spelling.
    let Some(root) = metadata["workspace_root"].as_str() else {
        return Err(cannot_judge_at(
            "release-coherence#metadata-has-no-workspace-root",
            "cargo metadata reported no workspace_root, so no member directory can be resolved",
        ));
    };
    let prefix = format!("{root}/");
    let mut machinery: Vec<String> = Vec::new();
    let mut published: BTreeSet<String> = BTreeSet::new();
    let mut enumerated = 0usize;
    for package in metadata["packages"].as_array().into_iter().flatten() {
        // **The directory comes from the manifest, not from the package name.** Deriving it as
        // `crates/<name>/` was the residual location assumption inside a repair whose own thesis was
        // *produced from the manifests, not from a location*: a member whose directory differs from its
        // package name contributes to neither set, so it is machinery nothing refuses (silent), or published
        // source whose basenames then enter the machinery set and refuse honest adopter prose.
        // `cargo metadata` answers this exactly — `manifest_path` is the member's own `Cargo.toml`.
        let Some(manifest) = package["manifest_path"].as_str() else {
            return Err(cannot_judge_at(
                "release-coherence#metadata-package-has-no-manifest-path",
                "a package in cargo metadata carries no manifest_path, so its directory cannot be resolved",
            ));
        };
        let Some(directory) = manifest
            .strip_prefix(&prefix)
            .and_then(|rest| rest.strip_suffix("Cargo.toml"))
        else {
            // `--no-deps` lists workspace members only, so every manifest sits under the root cargo reported
            // alongside them. One that does not is this gate's two sources describing different trees, which
            // is a fact to report rather than a member to skip — skipping is what kept the collapse silent.
            return Err(cannot_judge_at(
                "release-coherence#member-manifest-outside-workspace-root",
                format!(
                    "member manifest {manifest} is not under the workspace root {root} cargo reported for it"
                ),
            ));
        };
        let unpublished = package["publish"].as_array().is_some_and(|r| r.is_empty());
        // **`-z`, because git quotes a path it cannot write plainly.** `core.quotePath` defaults on and
        // `hermetic()` neutralises the config that could turn it off, so a tracked path carrying non-ASCII
        // bytes enters this set in its ESCAPED spelling and its real name is absent — after which
        // `adopter_cited_machinery` cannot recognise a record citing that file, a false negative in the
        // release gate. Latent today (no tracked path needs quoting) and the sibling capability already
        // raises the class to a SHALL, which is why it is closed rather than declared.
        let listing = git(repo, &["ls-files", "-z", directory]).map_err(|err| {
            cannot_judge_at(
                "release-coherence#directory-listing-unreadable",
                format!("could not enumerate {directory}: {err}"),
            )
        })?;
        for path in listing.split('\0').filter(|l| !l.is_empty()) {
            enumerated += 1;
            if unpublished {
                machinery.push(path.to_string());
            } else {
                published.insert(
                    path.rsplit_once('/')
                        .map_or(path, |(_, base)| base)
                        .to_string(),
                );
                let mut dir = path.to_string();
                while let Some(cut) = dir.rfind('/') {
                    dir.truncate(cut + 1);
                    published.insert(dir.clone());
                    dir.truncate(cut);
                }
            }
        }
    }
    // Members resolved and enumerated nothing means the directories were resolved against a root this
    // repository's git does not share — the same collapse by another route, and `scripts/` alone would still
    // look like an answer.
    if enumerated == 0 {
        return Err(cannot_judge_at(
            "release-coherence#no-tracked-file-for-any-member",
            format!(
                "no tracked file was found for any of the {} workspace members under {root}, so the machinery set \
             would be `scripts/` alone and this check would pass over its own subject",
                metadata["packages"].as_array().map_or(0, Vec::len)
            ),
        ));
    }
    let scripts = git(repo, &["ls-files", "-z", "scripts/"]).map_err(|err| {
        cannot_judge_at(
            "release-coherence#scripts-not-enumerable",
            format!("could not enumerate scripts/: {err}"),
        )
    })?;
    machinery.extend(
        scripts
            .split('\0')
            .filter(|l| !l.is_empty())
            .map(str::to_string),
    );

    let mut names: BTreeSet<String> = BTreeSet::new();
    for path in &machinery {
        names.insert(path.clone());
        let base = path
            .rsplit_once('/')
            .map_or(path.as_str(), |(_, base)| base);
        // Unique across the tree, or it names a published crate's file as well and would refuse an
        // entry that is about the product rather than about the machinery.
        if !published.contains(base) {
            names.insert(base.to_string());
        }
        // The same rule as the basename, for the same reason and found the same way: widening the corpus
        // made `crates/` a machinery name, because it is an ancestor of `crates/kanhe/`. It is equally an
        // ancestor of every published crate, and the live CHANGELOG says `crates/` in adopter-facing prose —
        // so the first run of this widening refused the repository's own changelog. An ancestor enters only
        // where it leads to machinery alone.
        let mut dir = path.clone();
        while let Some(cut) = dir.rfind('/') {
            dir.truncate(cut + 1);
            if !published.contains(&dir) {
                names.insert(dir.clone());
            }
            dir.truncate(cut);
        }
    }
    Ok(names)
}

/// The workspace as cargo reports it, `--no-deps`.
///
/// `pub` so a direction outside this module can hold a text reader against cargo's own answer, which is what
/// `repository-checks`'s *one fact about a manifest has one reader* asks for: two deliberate readers of one
/// fact need a reaction between them, or the second encodes a belief about the first.
pub fn cargo_metadata(repo: &Path) -> Result<serde_json::Value, Refusal> {
    let out = std::process::Command::new(env!("CARGO"))
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(repo)
        .output()
        .map_err(|err| {
            cannot_judge_at(
                "release-coherence#cargo-metadata-unrunnable",
                format!("could not run cargo metadata: {err}"),
            )
        })?;
    if !out.status.success() {
        return Err(cannot_judge_at(
            "release-coherence#cargo-metadata-failed",
            format!(
                "cargo metadata failed for {}: {}",
                repo.display(),
                String::from_utf8_lossy(&out.stderr).trim()
            ),
        ));
    }
    serde_json::from_slice(&out.stdout).map_err(|err| {
        cannot_judge_at(
            "release-coherence#cargo-metadata-not-json",
            format!("cargo metadata is not JSON: {err}"),
        )
    })
}

/// Every adopter-facing `[Unreleased]` entry naming this repository's own machinery.
///
/// A name is a **word** — a maximal run of path characters, required to equal a tracked path, a tracked
/// basename, or an ancestor directory derived from the enumeration. That is exact matching of a lexical token,
/// not substring matching: the run is delimited by the first character a path cannot hold. An earlier rule
/// compared whole backticked spans and three shapes this document already uses passed clean — a span carrying
/// a command, a padded double-backtick span, and an inline span wrapped across a source line.
///
/// Adopter-facing is the **complement** of `### Self-governance`, so a heading nobody anticipated reacts
/// rather than being exempt by default.
///
/// **A dated section is record only once it is a record.** The exemption's reason is that rewriting a dated
/// section to satisfy a rule written afterwards would falsify it — and that reason does not reach the section
/// this release is *about*. Release preparation dates it and then keeps writing into it: measured on this
/// repository, `chore(release): prepare 0.5.0` dated `## [0.5.0]` and hundreds of lines were added to it
/// afterwards across later commits, none of them examined, because the reader looked only at
/// `## [Unreleased]` — which release-ready state requires to be **empty**. So during preparation the check
/// had no subject at all.
///
/// The state decides it, not a version comparison. In [`State::ReleaseReady`] and [`State::Snapshot`] the
/// section dated for the workspace version is still being written, so it is adopter-facing. In
/// [`State::Development`] the workspace version *equals* the latest released one, so the section carrying it
/// is genuinely record and stays exempt — a rule phrased as *versions strictly below the workspace version
/// stay exempt* would refuse it, which is the reading this comment exists to keep anyone from adopting.
fn adopter_cited_machinery(
    repo: &Path,
    sections: &[Section],
    version: &str,
    state: State,
) -> Result<Vec<String>, Refusal> {
    // One enumeration. A second copy lived here for one commit, built for a census that was dropped, and
    // two constructions of one set is the drift this file's own doc-comment says it exists to prevent.
    let names = machinery_names(repo)?;

    let mut found: BTreeSet<String> = BTreeSet::new();
    // `heading` resets at each section by construction now. The cursor form cleared it by hand beside the
    // section assignment, which is one statement holding a structural fact — delete it and headings leak
    // across a section boundary with nothing to say so.
    for section in sections {
        let mut heading = String::new();
        for (_, line) in &section.body {
            if let Some(next) = line.strip_prefix("### ") {
                heading = next.trim_end().to_string();
            }
            let being_written = matches!(state, State::ReleaseReady | State::Snapshot)
                && section.name == format!("## [{version}]");
            if (section.name != "## [Unreleased]" && !being_written) || heading == "Self-governance"
            {
                continue;
            }
            for run in line
                .split(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '/' | '-')))
            {
                let token = run.strip_prefix("./").unwrap_or(run).trim_end_matches('.');
                if token.is_empty() {
                    continue;
                }
                if names.contains(token) {
                    found.insert(format!(
                        "  {} under `### {}` names {token}",
                        section.name,
                        if heading.is_empty() {
                            "(no heading)"
                        } else {
                            &heading
                        }
                    ));
                }
            }
        }
    }
    Ok(found.into_iter().collect())
}

// --- the fixture ------------------------------------------------------------------------------------------

/// A repository in the shape this judgement reads, built hermetically.
///
/// A fixture that inherits the judged machine cannot demonstrate a refusal, because the shape it builds is not
/// the shape it named — measured on the sibling publish gate, where ambient signing configuration turned an
/// intentionally unsigned tag into a signed one.
pub struct Fixture {
    /// The fixture repository's working tree.
    pub repo: PathBuf,
}

fn write(path: PathBuf, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("the fixture directory is writable");
    }
    std::fs::write(path, body).expect("the fixture file is writable");
}

/// Write a workspace manifest, its members and a matching `Cargo.lock`, all naming one version.
pub fn workspace_files(repo: &Path, version: &str) {
    write(
        repo.join("Cargo.toml"),
        &format!(
            "[workspace]\nmembers = [\"crates/xuanji\", \"crates/tianheng\", \"crates/renamed-dir\"]\n\n\
             [workspace.package]\nversion = \"{version}\"\n\n\
             [workspace.dependencies]\nxuanji = {{ path = \"crates/xuanji\", version = \"{version}\" }}\n"
        ),
    );
    // `xuanji` publishes and `tianheng` does not, so a fixture exercises both sides of the criterion the
    // machinery corpus reads from the manifests. Each member carries a `src/lib.rs`, because a workspace
    // cargo cannot load is one this gate cannot enumerate — the fixture is a real workspace or it is not
    // evidence about one.
    for (package, publishes) in [("xuanji", true), ("tianheng", false)] {
        let publish = if publishes { "" } else { "publish = false\n" };
        write(
            repo.join(format!("crates/{package}/Cargo.toml")),
            &format!(
                "[package]\nname = \"{package}\"\nversion.workspace = true\nedition = \"2024\"\n{publish}"
            ),
        );
        write(repo.join(format!("crates/{package}/src/lib.rs")), "");
    }
    // **A member whose directory is not its package name.** Without it, the fixture's two sides agree by
    // construction — every member sits at `crates/<name>/` — so a corpus that derived the directory from the
    // package name would pass every row here while being wrong about any workspace that does not. It is
    // unpublished, so its files must reach the machinery set: if the derivation regresses, this member
    // contributes nothing and a changelog naming its gate reports clean.
    write(
        repo.join("crates/renamed-dir/Cargo.toml"),
        "[package]\nname = \"machinery-under-another-name\"\nversion.workspace = true\n\
         edition = \"2024\"\npublish = false\n",
    );
    write(repo.join("crates/renamed-dir/src/lib.rs"), "");
    write(
        repo.join("crates/renamed-dir/tests/renamed_gate.rs"),
        "#[test]\nfn t() {}\n",
    );
    let minor = version.rsplit_once('.').map(|(h, _)| h).unwrap_or(version);
    // The example package the fixture carries, named through a binding like the members above rather than
    // as one path literal: a literal here reads as a reference into *this* repository, which the reference
    // gate then reports as stale — the path belongs to the fixture, not to the tree being judged.
    let example = "adopter";
    write(
        repo.join(format!("examples/{example}/Cargo.toml")),
        &format!(
            "[package]\nname = \"{example}\"\nversion = \"0.0.0\"\nedition = \"2024\"\n\n\
             [dependencies]\nxuanji = \"{minor}\"\n"
        ),
    );
    write(
        repo.join("Cargo.lock"),
        &format!(
            "version = 4\n\n[[package]]\nname = \"tianheng\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"xuanji\"\nversion = \"{version}\"\n\n\
             [[package]]\nname = \"machinery-under-another-name\"\nversion = \"{version}\"\n"
        ),
    );
}

/// Write a changelog in the development shape: an `[Unreleased]` section, carrying an adopter-facing item
/// only when asked, so its absence can be refused.
pub fn development_changelog(repo: &Path, version: &str, with_item: bool) {
    let item = if with_item {
        "- An adopter-facing change.\n\n"
    } else {
        ""
    };
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n{item}[Unreleased]: {COMPARE}/v{version}...HEAD\n"
        ),
    );
}

/// Write a changelog in the release shape: a dated section for `version`, with the link block naming
/// `previous`.
pub fn release_changelog(repo: &Path, version: &str, previous: &str) {
    // The same day the fixture's commits carry, from the one owner — this section and those commits are the
    // two halves `release-coherence` compares.
    let day = crate::hermetic_git::FIXTURE_DAY;
    write(
        repo.join("CHANGELOG.md"),
        &format!(
            "# Changelog\n\n## [Unreleased]\n\n## [{version}] - {day}\n\n- Release notes.\n\n\
             [Unreleased]: {COMPARE}/v{version}...HEAD\n[{version}]: {COMPARE}/v{previous}...v{version}\n"
        ),
    );
}

/// A repository released at `version` over a `0.1.0` predecessor. Prints its path.
pub fn build_fixture(root: &Path, name: &str, version: &str) -> Fixture {
    let repo = root.join(name);
    std::fs::create_dir_all(&repo).expect("the fixture root is writable");
    run(&repo, "git", &["init", "-q", "-b", "main"]);
    run(
        &repo,
        "git",
        &["config", "user.name", "Release Coherence Test"],
    );
    run(
        &repo,
        "git",
        &["config", "user.email", "release-coherence@example.invalid"],
    );
    run(&repo, "git", &["config", "commit.gpgsign", "false"]);

    workspace_files(&repo, "0.1.0");
    release_changelog(&repo, "0.1.0", "0.0.0");
    commit(&repo, "release: 0.1.0");

    workspace_files(&repo, version);
    release_changelog(&repo, version, "0.1.0");
    commit(&repo, &format!("release: {version}"));

    Fixture { repo }
}

pub use crate::hermetic_git::commit;
