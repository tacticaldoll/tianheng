use crate::manifest::{
    Publishable, WorkspaceVersion, is_semver, publishable, semver, workspace_version,
};

/// The two gates asked the same question about a version and answered differently.
///
/// `publish_source_gate::is_semver` was a digit check and `release_coherence_gate::semver` a parse, so a
/// component too large for `u64` was a version to one and not to the other — in front of `cargo publish`.
/// This is the boundary where they parted, held so the resolution cannot quietly reopen.
#[test]
fn a_component_too_large_to_order_is_not_a_version() {
    assert!(
        semver("1.0.99999999999999999999").is_none(),
        "a component that overflows `u64` cannot be ordered, so it is not a version this family reads"
    );
    assert_eq!(
        is_semver("1.0.99999999999999999999"),
        semver("1.0.99999999999999999999").is_some(),
        "the yes/no question and the parse must answer together — they are one implementation now, and \
         they were two that disagreed at exactly this input"
    );
    assert!(semver("1.0.0").is_some(), "an ordinary version still reads");
    assert!(is_semver("0.5.0"), "and so does this repository's own");
}

/// A `[package]` root reads as no workspace version, rather than as that package's.
///
/// The publish gate accepted a `[package]` table where its sibling did not. The fallback was unreachable —
/// this repository's root and both gates' fixtures declare `[workspace.package]` — so it was dropped rather
/// than carried forward as an untested branch settling a disagreement no input could produce. A root with
/// no workspace table is not the shape either gate judges, and both callers read `Absent` as a cannot-judge.
#[test]
fn a_single_crate_root_declares_no_workspace_version() {
    let single = "[package]\nname = \"solo\"\nversion = \"9.9.9\"\n";
    assert_eq!(
        workspace_version(single),
        WorkspaceVersion::Absent,
        "a `[package]` version is not the workspace's, and reading it as one is what the two gates \
         disagreed about"
    );
    let workspace = "[workspace]\nmembers = []\n\n[workspace.package]\nversion = \"0.5.0\"\n";
    assert_eq!(
        workspace_version(workspace),
        WorkspaceVersion::Declared("0.5.0".to_string())
    );
}

/// The scan is scoped to the table, so a later table's `version` is not the workspace's.
#[test]
fn a_version_under_another_table_is_not_the_workspace_version() {
    let manifest = "[workspace.package]\nversion = \"0.5.0\"\n\n[package]\nversion = \"9.9.9\"\n";
    assert_eq!(
        workspace_version(manifest),
        WorkspaceVersion::Declared("0.5.0".to_string())
    );
}

/// The three states, each given the input that produces it and no other.
///
/// A comment on the table heading and a comment after the value are the two shapes the raw-line reader got
/// wrong, in opposite directions: the first reported the version **absent**, the second reported the comment
/// as part of the **value**. Both are legal TOML. The third row is the state the `Option` could not hold at
/// all — a value this reader does not take, which is not a key that is missing.
#[test]
fn a_comment_never_becomes_the_version_and_an_unreadable_value_is_not_an_absent_one() {
    assert_eq!(
        workspace_version("[workspace.package] # the inherited version\nversion = \"0.5.0\"\n"),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "a comment on the heading closed the table before it opened"
    );
    assert_eq!(
        workspace_version("[workspace.package]\nversion = \"0.5.0\"  # bumped\n"),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "a trailing comment was carried into the value"
    );
    // **This row asserted the opposite until a real parser replaced the hand-rolled one.** A single-quoted
    // string is legal TOML that cargo accepts, and the reader not taking it was a limitation declared in the
    // release-coherence spec. The parser takes it, so the limitation is gone rather than documented.
    assert_eq!(
        workspace_version("[workspace.package]\nversion = '0.5.0'\n"),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "a single-quoted value is legal TOML and cargo accepts it"
    );
    assert_eq!(
        workspace_version("[workspace.package]\n# version = \"9.9.9\"\n"),
        WorkspaceVersion::Absent,
        "a commented-out key declares nothing"
    );
}

/// Every `publish` shape cargo honours, and the one this reader cannot decide.
///
/// **Measured against cargo 1.96.0, not assumed.** `cargo publish --dry-run` refuses `publish = false` and
/// `publish = []` identically and both report `[]` from `cargo metadata`; a non-empty list publishes to a
/// named registry; `publish.workspace = true` is honoured and reports whatever the workspace declared.
/// Those measurements are what this direction's table encodes — before this, three readers in this repository
/// looked for the word `false` and called the empty array published.
#[test]
fn every_publish_shape_cargo_honours_is_read_as_cargo_reads_it() {
    let package = |body: &str| format!("[package]\nname = \"m\"\nversion = \"0.1.0\"\n{body}\n");

    assert_eq!(
        publishable(&package("")),
        Publishable::Yes,
        "no key publishes"
    );
    assert_eq!(
        publishable(&package("publish = true")),
        Publishable::Yes,
        "an explicit true publishes"
    );
    assert_eq!(
        publishable(&package("publish = false")),
        Publishable::No,
        "false does not"
    );
    assert_eq!(
        publishable(&package("publish = []")),
        Publishable::No,
        "the empty registry list is what cargo reports for false, and it refuses the same way"
    );
    assert_eq!(
        publishable(&package(r#"publish = ["crates-io"]"#)),
        Publishable::Yes,
        "a named registry is a crate that publishes"
    );

    // **The same two arrays, spelled with whitespace.** A literal `"[]"` arm answered *publishable* for
    // `[ ]` — one space, legal TOML, and refused by `cargo publish` exactly as `[]` is. Measured on cargo
    // 1.96.0: `cargo metadata` reports `[]` for it and the dry run errors. The verdict follows the contents.
    assert_eq!(
        publishable(&package("publish = [ ]")),
        Publishable::No,
        "an empty array is empty however it is spaced, and cargo refuses it"
    );
    assert_eq!(
        publishable(&package("publish = [   ]")),
        Publishable::No,
        "and however much it is spaced"
    );

    // **The key decoded, which the hand-rolled reader answered `Unreadable` for.** Measured against cargo,
    // `"\u0070ublish" = false` reports `publish=[]` — the crate does not publish. The old answer was the safe
    // one rather than the right one, and a reader that cannot decode a key cannot tell *might be publish*
    // from *is publish*. Kept as a row because a negative run over the migration that made it readable
    // otherwise breaks nothing: the improvement would be revertible in silence.
    assert_eq!(
        publishable(&package("\"\\u0070ublish\" = false")),
        Publishable::No,
        "cargo decodes this key to `publish`, measured, so the crate does not publish"
    );
    assert_eq!(
        publishable(&package("publish.workspace = true")),
        Publishable::Unreadable("workspace = true".to_string()),
        "deferring to the workspace manifest is not a verdict this text carries"
    );
    assert_eq!(
        publishable(&package(r#"publish = [ "crates-io" ]"#)),
        Publishable::Yes,
        "a spaced non-empty array still names a registry"
    );

    // A bracket opened and not closed on this line needs the rest of the table, which this reader does not
    // take — so it refuses rather than reading an unterminated array as empty.
    assert!(
        matches!(
            publishable(&package("publish = [")),
            Publishable::Unreadable(_)
        ),
        "an unterminated array is not an empty one"
    );

    // The shape whose text cannot answer: cargo honours it, and deciding it needs the workspace manifest.
    assert!(
        matches!(
            publishable(&package("publish.workspace = true")),
            Publishable::Unreadable(_)
        ),
        "a workspace inheritance is not a verdict this text carries"
    );
    assert!(
        matches!(
            publishable(&package("publish = { workspace = true }")),
            Publishable::Unreadable(_)
        ),
        "nor is its inline-table spelling"
    );

    // `[workspace.package]`'s default is not a member's answer, and a commented-out key is not a key.
    assert_eq!(
        publishable("[workspace.package]\npublish = false\n"),
        Publishable::Yes,
        "a workspace default read as a member's verdict would report it for every member"
    );
    assert_eq!(
        publishable(&package("# publish = false")),
        Publishable::Yes,
        "executed text only — a commented-out key is not a declared one"
    );

    // **A key that merely begins with those seven letters is not that key.** `strip_prefix("publish")` sent
    // every such key down the value path, so an unrelated one standing before the real field refused the
    // whole member — while cargo treats a key it does not know as unused and carries on. Measured on cargo
    // 1.96.0: both of these report `publish=[]`. `publish-lockfile` is not hypothetical; cargo itself once
    // accepted it.
    assert_eq!(
        publishable(&package("publishx = true\npublish = false")),
        Publishable::No,
        "an unrelated key sharing the prefix is skipped, and the real field still answers"
    );
    assert_eq!(
        publishable(&package("publish-lockfile = true\npublish = false")),
        Publishable::No,
        "including a key cargo itself once had"
    );
    assert_eq!(
        publishable(&package("publishx = true")),
        Publishable::Yes,
        "and on its own it leaves the key absent, which is cargo's publishable default"
    );

    // **The quoted spellings of the key, which the raw comparison saw as no key at all.** This is the
    // direction that answers *publishable* for a crate cargo refuses — measured on cargo 1.96.0, both
    // report `publish=[]`.
    assert_eq!(
        publishable(&package(r#""publish" = false"#)),
        Publishable::No,
        "a basic-quoted key is the key"
    );
    assert_eq!(
        publishable(&package("'publish' = false")),
        Publishable::No,
        "and a literal-quoted one"
    );

    // The same direction one level out: a header spelling the reader skipped left the table unread, and an
    // unread `[package]` answers *publishable*. Measured on cargo 1.96.0: each reports `publish=[]`.
    assert_eq!(
        publishable("[ package ]\nname = \"m\"\npublish = false\n"),
        Publishable::No,
        "a spaced header opens the same table to cargo"
    );
    assert_eq!(
        publishable("[\"package\"]\nname = \"m\"\npublish = false\n"),
        Publishable::No,
        "and a quoted one"
    );
    assert_eq!(
        publishable(&(package("publish = false") + "[package.metadata]\npublish = true\n")),
        Publishable::No,
        "while `[package.metadata]` is a different table and its keys are not the package's"
    );

    // Cargo refuses a manifest declaring one key twice, so a reader answering from the first of two would
    // speak for a file cargo will not read at all.
    assert!(
        matches!(
            publishable(&package("publish = false\npublish = true")),
            Publishable::Unreadable(_)
        ),
        "two `publish` keys is not a verdict — cargo refuses the manifest"
    );

    // A trailing comment is not part of the value: `region::Source::toml` ends the line at the `#`, outside
    // strings. Measured on cargo 1.96.0: `publish=[]`.
    assert_eq!(
        publishable(&package("publish = false # why")),
        Publishable::No,
        "a trailing comment is not part of the value"
    );

    // The tree's own members, publishable and not, so the rows above are not the only subject this reader
    // is held over.
    assert_eq!(
        publishable("[package]\nname = \"kanhe\"\npublish = false\n"),
        Publishable::No
    );
}

/// Two `version` keys in `[workspace.package]` refuse, rather than the first one answering.
///
/// **One of three readers over the same root manifest disagreed with the other two.** `publishable` states
/// the reason in its own words — *cargo refuses a manifest that declares one key twice, so a reader answering
/// from the first of two would speak for a file cargo will not read at all* — and `package_name` answers the
/// same way. `workspace_version` returned on the first `version` it met, so it spoke for a manifest cargo
/// will not read.
///
/// Taking the values as a value first is what made the count askable at all: the shape that returned early
/// could not have counted them, which is `crate::selection`'s whole subject one module over.
///
/// Given the legal case beside it, because a reader that refuses everything also passes the duplicate case.
#[test]
fn two_workspace_version_keys_refuse_rather_than_the_first_answering() {
    let doubled = "[workspace.package]\nversion = \"0.5.0\"\nversion = \"0.6.0\"\n";
    match workspace_version(doubled) {
        WorkspaceVersion::Unreadable(what) => assert!(
            what.contains("version") && what.contains("duplicate"),
            "the refusal names the key and says it is duplicated: {what}"
        ),
        other => panic!(
            "two `version` keys must refuse rather than answer from the first; got {other:?}"
        ),
    }

    assert_eq!(
        workspace_version("[workspace.package]\nversion = \"0.5.0\"\n"),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "and one key still answers, so the refusal above is about the count rather than about reading at all"
    );

    // A `version` under another table is not this key, so two tables each carrying one are not two keys.
    assert_eq!(
        workspace_version(
            "[package]\nversion = \"9.9.9\"\n[workspace.package]\nversion = \"0.5.0\"\n"
        ),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "the cut keeps each table's keys to itself; counting across tables would refuse a legal manifest"
    );
}

/// A key cargo accepts is read, however the table carrying it is composed.
///
/// **Every row was put to `cargo metadata` first, through a member inheriting `version.workspace = true`.**
/// Each of `"version" = "0.5.0"`, `'version' = "0.5.0"` and `["workspace".package]` resolves the member at
/// `0.5.0`; so do `[workspace]` with `package.version = "0.5.0"` and with `package = { version = "0.5.0" }`.
///
/// The heading side of this module decoded and the key side matched raw text, so the first two answered
/// `Absent` — the state this type reserves for a key that is not there — and both git-reading gates then said
/// *workspace version is missing or malformed* about a manifest that declares it plainly. A review found it in
/// the last pass before the cut.
///
/// The parent-composed spellings are read the same way, because a parser builds the table they compose and so
/// does cargo. Refusing them was a false refusal over legal Cargo syntax: *this reader does not build that
/// structure* was a fact about the hand-rolled reader rather than about the manifest, and it did not survive
/// the reader. `Unreadable` keeps its subject — a `version` declared twice, which is a count this reader can
/// see and cargo refuses whole — and `Absent` keeps a `package` table that carries no version at all.
#[test]
fn a_key_spelling_cargo_accepts_is_read_however_its_table_is_composed() {
    for (label, manifest) in [
        (
            "quoted key",
            "[workspace.package]\n\"version\" = \"0.5.0\"\n",
        ),
        (
            "literal key",
            "[workspace.package]\n'version' = \"0.5.0\"\n",
        ),
        (
            "quoted heading segment",
            "[\"workspace\".package]\nversion = \"0.5.0\"\n",
        ),
        (
            "spaces around the key",
            "[workspace.package]\n  version   = \"0.5.0\"\n",
        ),
    ] {
        assert_eq!(
            workspace_version(manifest),
            WorkspaceVersion::Declared("0.5.0".to_string()),
            "{label}: cargo resolves a member at 0.5.0 through this spelling"
        );
    }

    for (label, manifest) in [
        (
            "dotted key in the parent table",
            "[workspace]\npackage.version = \"0.5.0\"\n",
        ),
        (
            "inline table in the parent",
            "[workspace]\npackage = { version = \"0.5.0\" }\n",
        ),
        // The tail decoded: this row answered `Absent` while the tail was compared raw, so the gates said
        // *missing or malformed* about a manifest cargo resolves at `0.5.0`.
        (
            "quoted tail in the parent",
            "[workspace]\npackage.\"version\" = \"0.5.0\"\n",
        ),
    ] {
        // **Measured against cargo, all three resolve at the declared version**, so refusing them was a
        // false refusal over legal TOML rather than a bound worth declaring. A real parser builds the table
        // these spellings compose, which is what cargo does.
        assert_eq!(
            workspace_version(manifest),
            WorkspaceVersion::Declared("0.5.0".to_string()),
            "{label}: cargo resolves this at 0.5.0, measured"
        );
    }

    // A `package`-headed key in `[workspace]` that cannot carry the version leaves the version absent, which
    // is the fact. A review found this over-refusal in the first version of the block above.
    assert_eq!(
        workspace_version("[workspace]\npackage.authors = [\"a\"]\n"),
        WorkspaceVersion::Absent,
        "a workspace declaring authors and no version anywhere declares no version, and says so"
    );

    // A dotted head naming this key assigns a field of it, not it. Every member writes this line.
    assert!(
        matches!(
            workspace_version("[workspace.package]\nversion.workspace = true\n"),
            WorkspaceVersion::Unreadable(_)
        ),
        "`version.workspace = true` is not a version, and it is not an absent key either"
    );
    // And a dotted head naming something else is another key's business, not a refusal.
    assert_eq!(
        workspace_version("[workspace.package]\nedition.workspace = true\nversion = \"0.5.0\"\n"),
        WorkspaceVersion::Declared("0.5.0".to_string()),
        "a member's `[package]` body is full of dotted keys; refusing on those would refuse every manifest"
    );
}

/// A name spelled in escapes is decoded, exactly as cargo decodes it.
///
/// **Every row below was put to `cargo metadata` first.** `"\u0070ublish" = false` reports `publish=[]`, and a
/// package table headed `["\u0070ackage"]` reports the package with `publish=[]` too, so cargo decodes escapes
/// in keys and in table names alike. Stripping the delimiters and stopping there left `\u0070ublish`, which
/// matched nothing: the key went unread and the crate answered *publishable* while cargo refuses to publish it.
///
/// **The first repair reported undecidability instead of decoding, and that traded a false answer for a false
/// refusal.** Any backslash anywhere in a heading refused the whole document -- including
/// `[target."cfg(feature = \"x\")".dependencies]`, which is a manifest cargo reads (measured: `serde` arrives
/// with that target), and `['other\table']`, a literal heading cargo reads without complaint. Decoding answers
/// both directions and leaves no third state for its consumers to carry.
///
/// A **literal** string decodes nothing, measured on both sides: `'\u0070ublish'` reports `publish=None` and
/// `['\u0070ackage']` is a different table to cargo, which reported the package beside it instead.
///
/// What is left undecodable is a file cargo will not read: `["\q"]` makes `cargo metadata` fail, naming the
/// escapes it accepts, and `["\uD800"]` fails as *invalid value, expected unicode hexadecimal value*. So the
/// refusals below stand for manifests nothing builds from, both of them measured rather than assumed.
#[test]
fn a_name_spelled_in_escapes_is_decoded_as_cargo_decodes_it() {
    // Escaped in the key, escaped in the heading, and escaped mid-name: each is `publish = false` to cargo.
    for manifest in [
        "[package]\nname = \"x\"\n\"\\u0070ublish\" = false\n",
        "[\"\\u0070ackage\"]\nname = \"x\"\npublish = false\n",
        "[\"pack\\u0061ge\"]\nname = \"x\"\npublish = false\n",
        "[\"\\x70ackage\"]\nname = \"x\"\npublish = false\n",
        "[\"\\U00000070ackage\"]\nname = \"x\"\npublish = false\n",
    ] {
        assert_eq!(
            publishable(manifest),
            Publishable::No,
            "cargo decodes this name, so this reads it: {manifest:?}"
        );
    }

    // The false refusal the first repair opened. Both are manifests cargo reads.
    assert_eq!(
        publishable("['other\\table']\nvalue = 1\n[package]\nname = \"x\"\npublish = false\n"),
        Publishable::No,
        "a literal heading carrying a backslash carries no escape, and refusing it refuses a legal manifest"
    );
    assert_eq!(
        publishable(
            "[package]\nname = \"x\"\npublish = false\n\n[target.\"cfg(feature = \\\"x\\\")\".dependencies]\nserde = \"1\"\n"
        ),
        Publishable::No,
        "an escaped-quote cfg target is the ordinary spelling of one, and it decides nothing about `publish`"
    );

    // A literal string is a different name to cargo, so the escape stays part of the name here too.
    assert_eq!(
        publishable("[package]\nname = \"x\"\n'\\u0070ublish' = false\n"),
        Publishable::Yes,
        "a literal key spells a different name, which cargo also reads as a different key"
    );
    assert_eq!(
        publishable("['\\u0070ackage']\npublish = false\n\n[package]\nname = \"x\"\n"),
        Publishable::Yes,
        "a literal heading is a different table, and cargo reported the package beside it"
    );

    // A dotted path whose segment is escaped folds to the path; one key carrying a dot stays one key.
    assert_eq!(
        workspace_version("[\"\\u0077orkspace\".package]\nversion = \"1\"\n"),
        WorkspaceVersion::Declared("1".to_string()),
        "each segment's quotes close, so the segment decodes and the path is the path"
    );
    assert_eq!(
        workspace_version("[\"\\u0077orkspace.package\"]\nversion = \"1\"\n"),
        WorkspaceVersion::Absent,
        "one key carrying a dot is not the dotted path, to cargo or here"
    );
    assert_eq!(
        workspace_version("[\"workspace\\u002Epackage\"]\nversion = \"1\"\n"),
        WorkspaceVersion::Absent,
        "the separator spelled as an escape is content, not a separator: cargo left the version alone"
    );

    // An escape cargo itself rejects: the file does not parse for cargo, and the verdict is refused here.
    for manifest in [
        "[\"\\q\"]\nname = \"x\"\npublish = false\n",
        "[\"\\uD800\"]\nname = \"x\"\npublish = false\n",
    ] {
        assert!(
            matches!(publishable(manifest), Publishable::Unreadable(_)),
            "an escape cargo rejects leaves which table this is undecided: {manifest:?}"
        );
    }
}
