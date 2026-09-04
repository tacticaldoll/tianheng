//! The release-coherence readers' failure matrix, for shapes a real manifest cannot legally carry.
//!
//! **Why these are unit tests and the rest of the matrix is not.** The end-to-end matrix builds a fixture
//! repository and runs `judge`, which puts every case through `cargo metadata` — so a case whose whole point
//! is a manifest shape cargo *rejects* cannot be written there. The first draft of the two-halves case tried
//! exactly that: `version.workspace = true, version = "0.2.0"` is a duplicate key, and the direction failed
//! on cargo's parse error rather than on what it meant to observe. A predicate over text is testable as a
//! predicate over text.

use crate::refusal::Kind;
use crate::release_coherence_gate::{is_iso_date, require_internal_pins};

/// Each half of the key recogniser, shown by the shape only that half rejects.
///
/// **The sentence this replaces was refuted by the code it described.** It claimed the delimiter half alone
/// still admits `/version` and the `=`-follows half alone still admits a key ending in `version` — but the
/// delimiter half rejects both of those, so the pair of examples established nothing about either half. One
/// case each, and a run rather than a description.
/// The workspace members these fixtures are written against, in the shape the caller resolves.
///
/// `require_internal_pins` selects its subject by **identity** — the crate a dependency names — so a unit
/// fixture has to say which crates the family holds, exactly as the end-to-end caller reads them from the
/// member manifests.
fn family() -> Vec<crate::release_coherence_gate::Member> {
    [
        "xuanji", "xingbiao", "guibiao", "hunyi", "louke", "tianheng",
    ]
    .into_iter()
    .map(|name| crate::release_coherence_gate::Member {
        name: name.to_string(),
        directory: std::path::PathBuf::from(format!("crates/{name}")),
    })
    .collect()
}

#[test]
fn a_value_is_not_a_key_however_it_reads() {
    // **The property the hand-rolled key recogniser needed two halves for.** One half rejected `version`
    // glued to the character before it — `/` in a path, `-` in a longer key — and the other rejected a
    // `version` preceded by a delimiter but not followed by `=`, which is what an occurrence inside a string
    // value looks like. A parser asks neither question: a value is a value. The property is kept and the
    // halves are gone with the recogniser.
    for manifest in [
        "[dependencies]\nxuanji = { path = \"crates/version-utils\", version = \"0.2.0\" }\n",
        "[dependencies]\nxuanji = { rust-version = \"1.85\", version = \"0.2.0\" }\n",
        "[dependencies]\nxuanji = { path = \"crates/xuanji\", version = \"0.2.0\", note = \"a version of record\" }\n",
    ] {
        let read = crate::release_coherence_gate::declared_dependencies(
            manifest,
            crate::release_coherence_gate::Subject::Requires,
        )
        .expect("these manifests parse");
        assert_eq!(read.len(), 1, "one dependency: {manifest}");
        assert_eq!(
            read[0].pin,
            crate::release_coherence_gate::Declared::Value("0.2.0".to_string()),
            "the requirement is the `version` key's value and nothing that merely reads like one: {manifest}"
        );
    }
}

/// A dated heading's fields are ranged, not merely three digit runs.
///
/// The repair's own standard — *a length test is a parse without its guarantee* — applied one level in: a
/// digit-field test is a parse without a date's guarantee.
#[test]
fn a_dated_suffix_is_a_date_and_not_only_three_digit_runs() {
    for real in ["2026-07-20", "1999-01-01", "2026-12-31"] {
        assert!(is_iso_date(real), "{real} is a date");
    }
    // The last two are the residue the field-range version admitted and this one does not: a day the
    // calendar does not have, and the leap rule in the direction that catches a century.
    for shaped in [
        "2026-99-99",
        "2026-00-10",
        "0000-00-00",
        "2026-13-01",
        "2026-02-31",
        "2026-04-31",
        "1900-02-29",
    ] {
        assert!(
            !is_iso_date(shaped),
            "{shaped} has a date's shape and names none"
        );
    }
    // And the day the leap rule does admit, so the refusals above are the calendar rather than a narrower
    // month.
    assert!(is_iso_date("2024-02-29"), "2024-02-29 is a date");
    for wrong_shape in ["notadate!!", "2026-7-20", "20260720", "2026-07-20-01"] {
        assert!(
            !is_iso_date(wrong_shape),
            "{wrong_shape} is not even the shape"
        );
    }
}

/// Two `path` or two `version` keys in one dependency are a manifest cargo refuses.
///
/// **This direction named two sites of its own until a real parser replaced the hand-rolled reader**, which
/// counted the values it had collected and reported *declares 2 `path` keys*. A duplicate key is not a
/// choice a reader has to decline — it is a document **cargo itself will not load**, so the honest answer is
/// the parse error, which names the key and says *duplicate key* at the position it met it.
///
/// Here rather than end-to-end for the reason this module's header gives: a duplicate key is a shape cargo
/// rejects outright, so a fixture carrying one fails on the parse rather than on what it means to observe.
#[test]
fn a_duplicate_key_in_one_dependency_is_a_manifest_cargo_refuses() {
    let several_paths = "[workspace.dependencies]\n\
                         xuanji = { path = \"crates/xuanji\", path = \"crates/other\", version = \"0.2.0\" }\n";
    let refusal = require_internal_pins(several_paths, "0.2.0", &family()).expect_err(
        "a duplicate key is a manifest cargo refuses, so nothing here is chosen between",
    );
    crate::refusal::expect("release-coherence#manifest-unparseable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("duplicate key") && refusal.message.contains("path"),
        "the refusal names the key and says it is duplicated, got: {}",
        refusal.message
    );

    let several_versions = "[workspace.dependencies]\n\
                            xuanji = { path = \"crates/xuanji\", version = \"0.2.0\", version = \"0.1.0\" }\n";
    let refusal = require_internal_pins(several_versions, "0.2.0", &family()).expect_err(
        "a duplicate key is a manifest cargo refuses, so nothing here is chosen between",
    );
    crate::refusal::expect("release-coherence#manifest-unparseable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    assert!(
        refusal.message.contains("duplicate key") && refusal.message.contains("version"),
        "the refusal names the key and says it is duplicated, got: {}",
        refusal.message
    );
}

/// A single-quoted `path` or `version` is read, and a value that is no string is still refused.
///
/// **This direction asserted the opposite until a real parser replaced the hand-rolled reader.** A
/// single-quoted TOML string is legal and cargo takes it; declining it was a limit of the reader written up
/// as though it were a fact about the manifest. Both halves now read, and the pin is judged against the
/// workspace version like any other.
///
/// The refusals themselves are kept and their WHEN rerun: what a parser will not take as a string is a value
/// that is not one. The unreadable **path** is still the half that matters most — it cannot be answered by
/// skipping the entry, because whether members inherit this workspace's crate is the thing that could not be
/// read.
#[test]
fn a_single_quoted_path_or_version_is_read_and_a_non_string_is_not() {
    let read = "[workspace.dependencies]\n\
                xuanji = { path = 'crates/xuanji', version = '0.2.0' }\n";
    require_internal_pins(read, "0.2.0", &family())
        .expect("cargo takes single-quoted strings, so this pin is judged and it agrees");

    let path = "[workspace.dependencies]\n\
                xuanji = { path = 5, version = \"0.2.0\" }\n";
    let refusal = require_internal_pins(path, "0.2.0", &family())
        .expect_err("a path this reader cannot take is not a path it may ignore");
    crate::refusal::expect("release-coherence#dependency-path-unreadable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);

    let version = "[workspace.dependencies]\n\
                   xuanji = { path = \"crates/xuanji\", version = 5 }\n";
    let refusal = require_internal_pins(version, "0.2.0", &family())
        .expect_err("a version this reader cannot take is not one that satisfies");
    crate::refusal::expect("release-coherence#internal-pin-unreadable", &refusal);
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
}

/// A quoted dependency key names its crate, and the stale pin behind it is judged.
///
/// **This direction asserted a refusal until a real parser replaced the hand-rolled reader.** TOML admits a
/// quoted key and cargo decodes it — measured, `"serde_json" = "1"` resolves to a dependency named
/// `serde_json` — so `"xuanji" = "0.0.1"` is a real family requirement. The old reader could not decode the
/// key, and refusing was the safe answer available to it; the parser decodes it, so the stale `0.0.1` is
/// **judged** rather than stopped in front of.
///
/// The second example is load-bearing rather than decoration: with only the quoted one present the per-example
/// counter would reach zero and the vacuity guard would refuse for its own reason, which is not this one.
#[test]
fn a_quoted_dependency_key_names_its_crate_and_its_pin_is_judged() {
    let root = std::env::temp_dir().join(format!("kanhe-quoted-key-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("the scratch root is writable");

    let write = |dir: &str, body: &str| {
        let at = root.join("examples").join(dir);
        std::fs::create_dir_all(&at).expect("the example directory is writable");
        std::fs::write(at.join("Cargo.toml"), body).expect("the example manifest is writable");
    };
    // The quoted key carries a stale pin; the bare one is correct and keeps the counter non-zero.
    write(
        "quoted",
        "[package]\nname = \"ex-quoted\"\n\n[dependencies]\n\"xuanji\" = \"0.0.1\"\n",
    );
    write(
        "bare",
        "[package]\nname = \"ex-bare\"\n\n[dependencies]\nxuanji = \"0.5.0\"\n",
    );

    let manifests = [(
        "crates/xuanji/Cargo.toml".to_string(),
        "[package]\nname = \"xuanji\"\n".to_string(),
    )];

    let members = super::super::release_coherence_gate::family_members(&manifests)
        .expect("these manifests each name their package");
    let refusal =
        super::super::release_coherence_gate::require_example_pins(&root, &members, "0.5.0")
            .expect_err("the quoted key names xuanji, so its stale pin is judged");
    crate::refusal::expect("release-coherence#example-pin-disagrees", &refusal);
    assert_eq!(refusal.kind, Kind::Violation);
    assert!(
        refusal.message.contains("0.0.1"),
        "the stale requirement is what the refusal names, got: {}",
        refusal.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// The TOML escape for `x`, built rather than typed.
///
/// **Typed into a literal it silently becomes the decoded value**, and that is measured rather than feared:
/// writing these two directions produced a plain `crates/xuanji` four times in a row, each time reading as
/// though it carried the escape. Naming the sequence puts the substitution where a reader sees it.
const ESCAPED_X: &str = "\\u0078";

/// The TOML escape for `j`, for the reason [`ESCAPED_X`] records.
const ESCAPED_J: &str = "\\u006A";

/// An escaped path is decoded and compared, where it used to be refused.
///
/// **This direction asserted a refusal until a real parser replaced the hand-rolled reader.** Cargo decodes a
/// TOML escape in a value — measured — so `cr` + [`ESCAPED_X`] + `tes/xuanji` is the directory
/// `crxtes/xuanji`, and the old reader could only say it did not know. The parser decodes it, so the path is
/// compared against the member's own directory like any other: it names somewhere else, which is a violation
/// that says where the member actually is.
///
/// The ordinary sibling stays: without it the vacuity floor would refuse for its own reason, which is not
/// this one.
#[test]
fn an_escaped_path_is_decoded_and_compared_and_an_ordinary_sibling_does_not_cover_for_it() {
    // **The escape decodes, so where it sits decides which check answers.** Before the parser both rows were
    // one refusal — *this reader cannot decode the path* — and the difference between them was invisible.
    for (path, site) in [
        (
            format!("cr{ESCAPED_X}tes/xuanji"),
            "release-coherence#internal-path-names-another-directory",
        ),
        // Decoded, this **is** the member's own directory, so the path is right and the stale pin is what is
        // left to refuse. The doc comment above always said this position was compared; now it is.
        (
            format!("crates/{ESCAPED_X}uanji"),
            "release-coherence#internal-pin-disagrees",
        ),
    ] {
        let manifest = format!(
            "[workspace.dependencies]\n\
             xingbiao = {{ path = \"crates/xingbiao\", version = \"0.5.0\" }}\n\
             xuanji = {{ path = \"{path}\", version = \"0.0.1\" }}\n"
        );
        let refusal = require_internal_pins(&manifest, "0.5.0", &family())
            .expect_err("cargo resolves this path and so does the parser, so it is compared");
        crate::refusal::expect(site, &refusal);
        assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    }
}

/// An escaped renamed package names its crate, and the stale pin behind it is judged.
///
/// **This direction asserted a refusal until a real parser replaced the hand-rolled reader.**
/// `release-coherence` requires a renamed dependency to be resolved by its `package` identity, and measured on
/// cargo 1.96.0 a `package` of `xuan` + [`ESCAPED_J`] + `i` reads as `xuanji`. The old reader compared the
/// undecoded source, found no family crate, and — once its value reader refused a backslash —
/// stopped in front of it. The parser decodes it, so the entry is matched against the family and its stale
/// `0.0.1` is judged.
///
/// The ordinary sibling in the same example is load-bearing: `requirements_here` is counted per example, so
/// without it the vacuity guard would refuse for its own reason.
#[test]
fn an_escaped_renamed_package_names_its_crate_and_its_pin_is_judged() {
    let root = std::env::temp_dir().join(format!("kanhe-escaped-rename-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    xingbiao::claim_scratch(&root).expect("the scratch root is writable");

    let at = root.join("examples").join("escaped");
    std::fs::create_dir_all(&at).expect("the example directory is writable");
    // **Both entries in ONE example, which is what makes the guard blind.** `requirements_here` is counted
    // per example, so an escaped entry alone in its own example leaves that counter at zero and the vacuity
    // guard catches it — measured: with the two split across two examples this direction passed under the
    // perturbation and was a restatement rather than a guard. Beside an ordinary family dependency in the
    // same manifest the counter is non-zero and the escaped entry's silence is invisible.
    std::fs::write(
        at.join("Cargo.toml"),
        format!(
            "[package]\nname = \"ex-escaped\"\n\n[dependencies]\n\
             xingbiao = \"0.5.0\"\n\
             alias = {{ package = \"xuan{ESCAPED_J}i\", version = \"0.0.1\" }}\n"
        ),
    )
    .expect("the example manifest is writable");

    let manifests = [
        (
            "crates/xuanji/Cargo.toml".to_string(),
            "[package]\nname = \"xuanji\"\n".to_string(),
        ),
        (
            "crates/xingbiao/Cargo.toml".to_string(),
            "[package]\nname = \"xingbiao\"\n".to_string(),
        ),
    ];

    let members = super::super::release_coherence_gate::family_members(&manifests)
        .expect("these manifests each name their package");
    let refusal = super::super::release_coherence_gate::require_example_pins(
        &root, &members, "0.5.0",
    )
    .expect_err(
        "cargo reads this package as xuanji and so does the parser, so its stale pin is judged",
    );
    assert_eq!(refusal.kind, Kind::Violation, "{}", refusal.message);
    // The SITE, not only the kind. Without it, a refusal the vacuity guard produced reads as this
    // direction's evidence — which is exactly how its first version passed under the perturbation.
    crate::refusal::expect("release-coherence#example-pin-disagrees", &refusal);
    assert!(
        refusal.message.contains("0.0.1"),
        "the stale requirement is what the refusal names, got: {}",
        refusal.message
    );

    let _ = std::fs::remove_dir_all(&root);
}

/// An internal pin taking the workspace offer is refused, because this manifest **is** the workspace.
///
/// A dependency in the root's own catalog spelling `workspace = true` would inherit from the catalog it sits
/// in. Cargo refuses a manifest that inherits what its catalog does not declare, and a catalog inheriting
/// from itself declares nothing — so this is undecidable rather than absent, and it stops in front of an
/// operator instead of being reported as a missing pin.
#[test]
fn an_internal_pin_taking_the_workspace_offer_is_refused() {
    let manifest = "\
[workspace.dependencies]
xuanji = { path = \"crates/xuanji\", workspace = true }
";
    let refusal = require_internal_pins(manifest, "0.5.0", &family()).expect_err(
        "a pin inheriting from the catalog it sits in is not a pin this reader can read",
    );
    assert_eq!(refusal.kind, Kind::CannotJudge, "{}", refusal.message);
    crate::refusal::expect("release-coherence#internal-pin-inherited", &refusal);
}

/// A detailed dependency table is filed from its own body, not from wherever the next heading falls.
///
/// **The `Option` that held a half-built record is what this closes.** `declared_dependencies` carried a
/// `pending: Option<Detailed>` flushed at the next heading, because a `[dependencies.NAME]` table's `package`,
/// `version` and `path` arrive across separate lines and the record could only be filed once a following
/// heading proved the table over. Every table now carries its own body, so `Detailed` is built and filed
/// inside one iteration — no half-built record to hold, and no boundary to remember to flush at.
///
/// Read through `require_internal_pins` rather than through the reader itself, so nothing's visibility is
/// widened for a test: the pin verdict is what a wrong boundary changes, and it is already reachable.
///
/// **A foreign table sits between two detailed ones, which is the arrangement that separates the two
/// implementations.** A record surviving a heading folds `[package.metadata.docs.rs]`'s `version` into the
/// table above it, and the pin then disagrees with the workspace version. Two detailed tables in a row would
/// pass either way, since the second heading flushes the first correctly.
#[test]
fn a_detailed_table_is_filed_from_its_own_body() {
    let manifest = "\
[dependencies.xuanji]
path = \"crates/xuanji\"
version = \"0.5.0\"

[package.metadata.docs.rs]
version = \"9.9.9\"

[dependencies.xingbiao]
path = \"crates/xingbiao\"
version = \"0.5.0\"
";
    require_internal_pins(manifest, "0.5.0", &family()).expect(
        "both internal dependencies pin the workspace version; the `9.9.9` between them is a docs.rs key and \
         belongs to neither table",
    );
}

/// A `path` value is read through its components, and the three refusals are told apart.
///
/// **Here as well as end to end, because the end-to-end rows cannot reach one of the arms.**
/// `Component::Prefix` is produced only on Windows, and this repository's CI is Ubuntu; the unit rows say
/// which classification each spelling earns on the host that runs them, so a change to the classifier is
/// caught without a repository, a fixture or a `judge`.
///
/// **The drive-prefix arm is compiled and unexercised here — a gap in coverage, not a bound.** It reacts, and
/// correctly, so the reaction declines to observe nothing and the bound register has nothing to hold; what is
/// missing is a run on a host this repository does not build for. It shares its answer with `RootDir`, which
/// the absolute row does exercise. What a Unix host does with `C:/x` is read it as an ordinary relative
/// directory named `C:` — cargo on that host does the same, so the two agree there too.
///
/// Negative runs. Removing the `CurDir` arm does not compile at all — the match over `Component` is
/// exhaustive, so an arm cannot be dropped in silence, which is a stronger guarantee than a direction can
/// give. Answering `CurDir` with `NamesNoDirectory` instead: `./crates/tianheng` classifies as
/// `NamesNoDirectory` where the row requires `crates/tianheng`. Folding `ParentDir` into `Absolute`: the
/// traversal rows come back `Absolute`, which is the wrong cause named — the defect this direction exists
/// for, one arm along.
#[test]
fn a_path_value_is_read_through_its_components() {
    use crate::release_coherence_gate::{Unresolvable, normalized_directory};
    use std::path::PathBuf;

    for spelling in [
        "crates/tianheng",
        "./crates/tianheng",
        "crates//tianheng",
        "crates/tianheng/",
        "crates/./tianheng",
    ] {
        assert_eq!(
            normalized_directory(spelling),
            Ok(PathBuf::from("crates/tianheng")),
            "cargo resolves {spelling} to that directory, measured"
        );
    }
    assert_eq!(
        normalized_directory("vendor/tianheng"),
        Ok(PathBuf::from("vendor/tianheng")),
        "a directory this reader can name is named, whether or not it is the member's"
    );
    for spelling in ["/opt/crates/tianheng", "/"] {
        assert_eq!(
            normalized_directory(spelling),
            Err(Unresolvable::Absolute),
            "{spelling} is rooted, and this reader is handed no repository to make it relative against"
        );
    }
    for spelling in ["crates/../vendor/tianheng", "../crates/tianheng", ".."] {
        assert_eq!(
            normalized_directory(spelling),
            Err(Unresolvable::Traversal),
            "{spelling} carries a `..`, applied after symlink resolution"
        );
    }
    for spelling in [".", "./", "", "./."] {
        assert_eq!(
            normalized_directory(spelling),
            Err(Unresolvable::NamesNoDirectory),
            "{spelling} names the manifest's own directory, which cargo refuses as a dependency"
        );
    }
}

/// A dotted internal pin is read, and a stale one in that form is refused.
///
/// **The gate passed a stale pin written this way.** `require_internal_pins` selected on **path** then, and
/// the per-line reading gave the `.path` line a path with no version and the `.version` line a version with
/// no path — so neither was internal to it. Measured before the repair: four correct inline siblings plus a stale
/// dotted pair answered `Ok(())`, where the same staleness written inline is a violation. Not a shape nobody
/// writes: `version.workspace = true` is that spelling in every member's `[package]` table.
///
/// **Each case carries an ordinary inline sibling, so the vacuity counter cannot cover for it.**
/// `require_internal_pins` returns early when it finds no internal pins at all, and a fixture holding only the
/// dotted pair would take that arm — passing for having read nothing rather than for reading correctly.
///
/// **Both directions, because the repair has two ways to be wrong.** Reading nothing leaves the miss; reading
/// per line produces a *false refusal* — measured, the naive form reports `xuanji.path is pinned to
/// crates/xuanji; expected 0.5.0`, the path read as the requirement, over a manifest cargo accepts. That is a
/// defect in its own right — though the Core Contract's *one forbidden bug* is the opposite direction, a real
/// violation that silently passes — so the correct case is asserted first and the stale one after it.
#[test]
fn a_dotted_internal_pin_is_read_and_a_stale_one_refused() {
    let sibling = "xingbiao = { path = \"crates/xingbiao\", version = \"0.5.0\" }\n";

    let correct = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.5.0\"\n"
    );
    require_internal_pins(&correct, "0.5.0", &family()).expect(
        "a dotted internal pin at the workspace version is read and passes; refusing it would be a false \
         refusal over a manifest cargo accepts",
    );

    let stale = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.4.0\"\n"
    );
    let refusal = require_internal_pins(&stale, "0.5.0", &family())
        .expect_err("a stale dotted pin is the defect this direction exists for");
    crate::refusal::expect("release-coherence#internal-pin-disagrees", &refusal);
    assert!(
        refusal.message.contains("xuanji") && refusal.message.contains("0.4.0"),
        "the refusal names the dependency by its head key and the pin it found, not the dotted line: {}",
        refusal.message
    );

    // A tail this reader does not judge is ignored exactly as its inline counterpart is, rather than
    // becoming a record of its own.
    let with_features = format!(
        "[workspace.dependencies]\n{sibling}xuanji.path = \"crates/xuanji\"\nxuanji.version = \"0.5.0\"\n\
         xuanji.features = [\"serde\"]\n"
    );
    require_internal_pins(&with_features, "0.5.0", &family())
        .expect("`features` is not a field this reader judges, dotted or inline");
}
