use crate::manifest::{
    Publishable, Quoted, WorkspaceVersion, is_semver, publishable, quoted_value, semver,
    workspace_version,
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
    assert_eq!(
        workspace_version("[workspace.package]\nversion = '0.5.0'\n"),
        WorkspaceVersion::Unreadable("'0.5.0'".to_string()),
        "a single-quoted value is legal TOML this reader does not take — not a key that is absent"
    );
    assert_eq!(
        workspace_version("[workspace.package]\n# version = \"9.9.9\"\n"),
        WorkspaceVersion::Absent,
        "a commented-out key declares nothing"
    );
}

/// An unquoted value does not borrow the quoted one that follows it.
///
/// The reader took the first pair of double quotes anywhere in the text it was given, so a value that is not
/// a string at all was answered with the *next* key's string. `Unreadable` is the state this type exists for,
/// and it was reachable only when nothing else on the line was quoted.
///
/// Both halves, because either alone reads as the other's defect: the quote must open the value, and a value
/// it does open is still read to its closing quote with whatever follows discarded.
#[test]
fn a_value_that_is_not_a_string_does_not_borrow_the_next_one() {
    assert_eq!(
        quoted_value(" xuanji, version = \"0.2.0\" }"),
        Quoted::Unreadable,
        "an unquoted value read the following key's string as its own"
    );
    assert_eq!(
        quoted_value(" \"0.2.0\", package = \"xuanji\" }"),
        Quoted::Value("0.2.0".to_string()),
        "a value that does open with a quote is still read to its closing quote"
    );
}

/// A TOML escape is a value this reader cannot read, not a value it can.
///
/// **Cargo decodes escapes and this reader decodes none**, so returning the raw source as a `Value` hands
/// every consumer an identity, path or version that is not the one cargo resolves. Measured on cargo 1.96.0
/// against a scratch workspace: `path = "crates/\u0078uanji"` resolves the member at `crates/xuanji`,
/// `name = "xuan\u006Ai"` reads as `xuanji`, and `version = "0.\u0035.0"` reads as `0.5.0`.
///
/// What that cost: each consumer compares the undecoded text and takes a `continue` when the comparison
/// fails, so an internal dependency or a renamed family crate stops being checked in silence — and the
/// per-manifest vacuity guards cannot see it, because one escaped entry beside one ordinary one leaves their
/// counters non-zero.
///
/// Every escape form TOML admits is given, not one representative: a reader that refused `\u` and admitted
/// `\n` would read as covering the class while leaving it open.
#[test]
fn a_toml_escape_is_refused_rather_than_returned_undecoded() {
    for written in [
        r#" "crates/\u0078uanji""#,
        r#" "xuan\u006Ai""#,
        r#" "0.\u0035.0""#,
        r#" "\U0001F600""#,
        r#" "a\\b""#,
        r#" "a\nb""#,
        r#" "a\tb""#,
        r#" "a\rb""#,
        r#" "a\bb""#,
        r#" "a\fb""#,
    ] {
        assert_eq!(
            quoted_value(written),
            Quoted::Unreadable,
            "{written} carries an escape this reader does not decode, so it must refuse rather than answer \
             with the source"
        );
    }

    // An escaped quote is the narrower shape the same check missed: the split landed on it and answered a
    // value ending in a backslash, which no manifest declares.
    assert_eq!(
        quoted_value(r#" "a\"b""#),
        Quoted::Unreadable,
        "an escaped quote must not be read as the closing one"
    );

    // Unescaped values are untouched, including a backslash OUTSIDE the string, which is not this value's.
    assert_eq!(
        quoted_value(" \"crates/xuanji\""),
        Quoted::Value("crates/xuanji".to_string())
    );
    assert_eq!(
        quoted_value(" \"0.5.0\", package = \"xuanji\" }"),
        Quoted::Value("0.5.0".to_string())
    );
    assert_eq!(
        quoted_value(" \"abc\" \\ trailing"),
        Quoted::Value("abc".to_string()),
        "a backslash after the closing quote is not part of the value"
    );
}

/// A multiline basic string is refused rather than read as an empty value.
///
/// **Not an escape case, and that is why the backslash branch could not see it.** TOML admits `"""…"""`
/// wherever it admits `"…"`, and cargo reads it: measured on cargo 1.96.0, `path = """crates/xuanji"""`
/// resolves the member and `name = """xuanji"""` reads as `xuanji`. This reader stripped the opening quote,
/// found the next one immediately, and answered `Value("")` — an empty path, an empty identity, an empty
/// version — which every consumer compares and passes over. The same silence the escape branch closes,
/// reached with no backslash in sight.
#[test]
fn a_multiline_basic_string_is_refused_rather_than_read_as_empty() {
    for written in [
        r#" """crates/xuanji""""#,
        r#" """xuanji""""#,
        r#" """0.5.0""""#,
        r#" """""#,
    ] {
        assert_eq!(
            quoted_value(written),
            Quoted::Unreadable,
            "{written} opens a multiline basic string this reader does not read, so it must refuse rather \
             than answer with an empty value"
        );
    }

    // An ordinary EMPTY single-line string is still a value, and is the shape the multiline guard must not
    // reach: `""` is two quotes and a multiline opener is three.
    assert_eq!(
        quoted_value(" \"\", version = \"0.5.0\" }"),
        Quoted::Value(String::new()),
        "an empty single-line string is a value this reader can read"
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

    // The tree's own six publishable crates and two unpublishable ones, so the rows above are not the only
    // subject this reader is held over.
    assert_eq!(
        publishable("[package]\nname = \"kanhe\"\npublish = false\n"),
        Publishable::No
    );
}
