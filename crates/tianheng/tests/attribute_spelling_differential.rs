//! A **generated** differential over attribute spellings, with rustc as the third party.
//!
//! **Why this exists, and what it replaces.** Every defect this family has had in its attribute readers was
//! found by a person reading one more spelling, and each repair was correct and followed by another shape.
//! `BACKLOG.md`'s `path_meta_values` entry states the cost — nine wrong answers across two crates, every one
//! a clean verdict over source no build compiles or a violation against source the governed tree does not
//! have — and states what the null option cannot buy: *any statement about what remains*.
//!
//! **What the corpus is, stated as what it is.** It is the cross-product of the axes `corpus()` declares,
//! summed over attribute positions, plus the look-alikes declared beside them — and the count is printed
//! on every clean run rather than written here. It is **not** every lexical form Rust admits: no
//! cross-product can be, and this one said so while being false along three axes at once. Two of the
//! three are axes now; the third answers a different question and `BACKLOG.md` carries it.
//!
//! **Three parties, because two are not enough.** A differential over the three dimensions alone answers
//! *do they agree*, and agreement is not correctness: the raw bare-`cfg` spelling was missed by all three at
//! once, and a reader comparing only the three would have reported that as clean. So each shape carries a
//! **declared** answer, and rustc is asked whether that declaration is true:
//!
//! 1. **Is the spelling legal Rust?** Every shape is compiled with no cross-module reference. A failure
//!    here is the **generator's**: it claimed a shape Rust does not admit.
//! 2. **Does the remap apply?** The same source, plus a reference to an item defined only in the named
//!    target. For a governed shape under a live predicate this must resolve; for a look-alike its live
//!    probe must **not**, either by rustc refusing the attribute outright or by compiling without it.
//!
//! **The two questions were one arm, and the arm asserted the wrong fact.** A single compile carrying the
//! reference fails both when the spelling is illegal and when the spelling is fine and the *declared
//! answer* is wrong, and the message named only the first — measured, `#[cfg_attr(unix, allow(dead_code))]`
//! declared governed reported *rustc rejects the generated spelling … the corpus claims a shape Rust does
//! not admit*, over source rustc accepts. Two facts an author repairs in opposite places, so two questions.
//!
//! **What rustc cannot decide here, said rather than left to be assumed.** Under a false predicate no
//! configuration on this host compiles the target, so rustc can say only that the source is legal. The
//! declaration carries those, and the true-predicate rows are what hold the declaration to something. The
//! dimensions are cfg-blind by construction — they union every candidate a build *could* compile through —
//! so a false predicate is expected to be governed exactly as a true one is, and that property is asserted
//! rather than assumed.
//!
//! **Gated behind `TIANHENG_SPELLING_DIFFERENTIAL`**, because it compiles one crate per shape, and named in
//! the Definition of Done and in CI on its own line — the shape `pin_bites` already sets for a direction
//! that must not make the ordinary suite pay for it, and that must not be left to whoever remembers.

#[path = "support/mod.rs"]
mod support;
use support::{TempFixture, guibiao_exit, hunyi_exit, louke_exit};

const REASON: &str = "differential: one spelling, one answer, in every dimension";
const SEAM: &str = "conformance-seam";
const FORBIDDEN: &str = "pub mod forbidden { pub struct Thing; }\n";
/// A probe for the declared seam at top level, so 漏刻's declared-but-unprobed direction never fires as an
/// incidental confound: the only finding a fixture can produce is the one inside the file under test.
const PROBED: &str = "pub fn probed(o: u8) { assert_boundary!(\"conformance-seam\", o); }\n";
/// All three dimensions' violations in one file, plus the item rustc is asked to resolve.
const VIOLATIONS: &str = "use crate::forbidden::Thing;\n\
                          pub fn leak() -> crate::forbidden::Thing { crate::forbidden::Thing }\n\
                          pub fn typo(o: u8) { assert_boundary!(\"conformance-saem\", o); }\n\
                          pub fn only_in_target() -> u8 { 7 }\n";
const CLEAN: &str = "pub fn conventional() -> u8 { 0 }\n";

/// What the generator declares about a shape, and what rustc is asked to confirm.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Answer {
    /// `target.rs` is a remap candidate: every dimension governs it, so the violation inside reacts.
    Governed,
    /// The spelling looks like a remap and names nothing: no configuration compiles `target.rs`, so a
    /// dimension governing it is over-reading — a violation against source the tree does not have.
    Decoy,
}

struct Shape {
    label: String,
    attribute: String,
    answer: Answer,
    /// Which position this shape occupies, carried as the value rather than left in the label — the
    /// coverage floor below reads it, and a floor that matched on label text would be a reader over
    /// prose where an axis value was available.
    position: Position,
    /// How this shape spells its value literal, carried for the same reason `position` is: the floor reads
    /// the axis value, and an axis added beside a floored one is an axis nothing floors.
    value: Value,
    /// Whether the predicate holds on this host, and so whether rustc can be asked which file the build
    /// contains rather than only whether the source is legal.
    live_predicate: bool,
}

// --- the axes, and what each one owes ---------------------------------------------------------------------
//
// **Three instruments per axis, and each catches a shrink the other two cannot.** A `//` comment rather than
// an item doc, and above both definitions rather than on one: this rule governs the pair, and attached to
// either item it becomes the annexed doc this repository names elsewhere — a passage describing one thing,
// hanging off its neighbour, where both read plausibly enough that nobody re-attributes it. It displaced
// `Position`'s own summary line out of first position while `Value` carried no statement of the rule at all,
// which inverts the reason it was written down: a third axis modelled on `Value` — the per-variant shape,
// which is the one that generalises — would have been read beside the sibling that said nothing.
//
//   1. **Set agreement** — the direction declares the axis's membership and holds it against the enumerator
//      both ways, so a trimmed enumerator has something independent to disagree with. A floor iterating the
//      array the corpus was built from cannot see that array shrink.
//   2. **Per-variant coverage** — every declared variant produced at least one shape, read off a field on
//      `Shape` rather than off its label.
//   3. **Variant distinctness** — the variants actually produce different source. Coverage cannot see this:
//      a shape is labelled from the loop variable, not from what the axis *did*, so a variant collapsed onto
//      its sibling satisfies every coverage assertion while generating one form twice under two names.
//
// Both axes owe all three. Measured, twice, that the third is the one that gets missed: a `Value::Raw`
// spelling collapsed to the ordinary form, and a `Position::Direct` arm emitting a `cfg_attr`-wrapped
// attribute, each passed every assertion that existed when it was tried. `BACKLOG.md` carries the class as a
// `WATCH` whose trigger is a third axis.

/// Where the `#[path]` attribute sits — the axis every reader has an arm for and the corpus had none.
///
/// **Not orthogonal to the rest, which is why the corpus is a sum over positions rather than one product.**
/// A direct attribute is resolved unconditionally, so it carries no wrapper and no predicate; and it admits
/// no look-alike either — measured, `#[foo::path = "target.rs"] mod m;` is `error[E0433]: cannot find module
/// or crate 'foo'`, because there is no false predicate to keep the qualified path from being resolved. A
/// decoy has to compile while naming nothing, so decoys live only where a predicate carries them.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Position {
    /// `#[path = "…"]` written on the declaration itself.
    Direct,
    /// `#[cfg_attr(<pred>, path = "…")]`, where the attribute is applied conditionally.
    CfgAttrWrapped,
}

impl Position {
    const ALL: [Self; 2] = [Self::Direct, Self::CfgAttrWrapped];
}

/// How the value literal is spelled. Both decode to the same path, and a reader comparing renderings
/// rather than values answers differently for the two.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Value {
    Ordinary,
    Raw,
}

impl Value {
    const ALL: [Self; 2] = [Self::Ordinary, Self::Raw];

    fn label(self) -> &'static str {
        match self {
            Value::Ordinary => "ordinary value",
            Value::Raw => "raw value",
        }
    }

    fn spell(self, path: &str) -> String {
        match self {
            Value::Ordinary => format!("\"{path}\""),
            Value::Raw => format!("r#\"{path}\"#"),
        }
    }
}

/// How the `path` key and its `=` are written, holding the raw-identifier and trivia spellings.
const METAS: [(&str, &str); 4] = [
    ("path", "path = {value}"),
    ("raw path", "r#path = {value}"),
    ("comment before name", "/*c*/ path = {value}"),
    ("comment before eq", "path /*c*/ = {value}"),
];

/// The `cfg_attr` nestings a wrapped attribute can arrive through.
const WRAPPERS: [(&str, &str); 3] = [
    ("plain", "cfg_attr({pred}, {meta})"),
    ("nested", "cfg_attr({pred}, cfg_attr({pred}, {meta}))"),
    (
        "raw wrapper",
        "cfg_attr({pred}, r#cfg_attr({pred}, {meta}))",
    ),
];

/// Predicates, with whether each holds on this host — which decides whether rustc can be asked *which file
/// the build contains* rather than only whether the source is legal.
const PREDICATES: [(&str, &str, bool); 3] = [
    ("unix", "unix", cfg!(unix)),
    (
        "compound",
        "all(unix, not(target_os = \"none\"))",
        cfg!(unix),
    ),
    ("false", "any()", false),
];

/// The generated corpus: a **sum over positions**, each position's own product, plus the declared decoys.
///
/// **The count is `len()`, and the claim above it names these axes rather than the language.** This function
/// used to be a flat `wrapper × predicate × meta` product under a module doc reading *the corpus is every
/// lexical form Rust admits, which no inspection enumerates* — a claim no cross-product can make, and one
/// that was false along three axes at once: every shape was `cfg_attr`-headed, so the direct attribute
/// position had no row; every value was an ordinary literal; and the bare-`cfg` spelling the doc names as
/// the defect that motivated the whole file had nothing either.
///
/// **Two of those three are axes here and the third is not**, because it answers a different question. A
/// bare `#[cfg(pred)]` removes the whole item when `pred` is false, where `#[cfg_attr(pred, …)]` never
/// removes the item — so what a reader does with a bare `cfg` is *absence tolerance*, whether a missing
/// backing file is an error, and its probe is a file that does not exist rather than an item that resolves.
/// `Answer` has no value for it. It is a second subject, and `BACKLOG.md` carries it as one.
///
/// A decoy is paired with a **false** predicate deliberately. `foo::path` under a live predicate is a path
/// rustc resolves and rejects, so the pairing is what makes the shape legal source at all — and the
/// compile step below is what would catch the pairing being wrong.
fn corpus() -> Vec<Shape> {
    let mut out = Vec::new();
    for position in Position::ALL {
        for value in Value::ALL {
            for (ml, meta) in METAS {
                let meta = meta.replace("{value}", &value.spell("target.rs"));
                match position {
                    Position::Direct => out.push(Shape {
                        label: format!("direct · {ml} · {}", value.label()),
                        attribute: format!("#[{meta}]"),
                        answer: Answer::Governed,
                        position,
                        value,
                        // No predicate stands between a direct attribute and the build, so the remap always
                        // applies and rustc can always be asked which file the build contains.
                        live_predicate: true,
                    }),
                    Position::CfgAttrWrapped => {
                        for (wl, wrapper) in WRAPPERS {
                            for (pl, pred, live) in PREDICATES {
                                out.push(Shape {
                                    label: format!("{wl} · {pl} · {ml} · {}", value.label()),
                                    attribute: format!(
                                        "#[{}]",
                                        wrapper.replace("{pred}", pred).replace("{meta}", &meta)
                                    ),
                                    answer: Answer::Governed,
                                    position,
                                    value,
                                    live_predicate: live,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
    // The look-alikes. Each names `target.rs` in a position no build compiles it from, and each takes the
    // value axis too — a reader comparing renderings rather than values answers differently for the two.
    for value in Value::ALL {
        let spelled = value.spell("target.rs");
        for (label, meta) in [
            ("qualified path", format!("foo::path = {spelled}")),
            ("raw qualified path", format!("foo::r#path = {spelled}")),
            (
                "path in a non-cfg_attr group",
                format!("foo(path = {spelled})"),
            ),
        ] {
            out.push(Shape {
                label: format!("decoy · {label} · {}", value.label()),
                attribute: format!("#[cfg_attr(any(), {meta})]"),
                answer: Answer::Decoy,
                position: Position::CfgAttrWrapped,
                value,
                live_predicate: false,
            });
        }
    }
    for (label, attribute) in [
        (
            // Literal text rather than a path value, so the value axis does not reach it.
            "path inside a string literal",
            "#[cfg_attr(any(), doc = \"path = \\\"target.rs\\\"\")]",
        ),
        (
            "qualified wrapper",
            "#[cfg_attr(any(), foo::cfg_attr(unix, path = \"target.rs\"))]",
        ),
        (
            "path key in a compound predicate",
            "#[cfg_attr(all(unix, path = \"target.rs\"), allow(dead_code))]",
        ),
        (
            "path before the first comma",
            "#[cfg_attr(path = \"target.rs\", allow(dead_code))]",
        ),
    ] {
        out.push(Shape {
            label: format!("decoy · {label}"),
            attribute: attribute.to_string(),
            answer: Answer::Decoy,
            position: Position::CfgAttrWrapped,
            // These carry the value inside a literal attribute string rather than through the axis, so
            // they are the ordinary spelling by construction.
            value: Value::Ordinary,
            live_predicate: false,
        });
    }
    out
}

/// Compile `source` as a crate and hand back rustc's own first line on failure.
fn compiles(name: &str, source: &str) -> Result<(), String> {
    let dir = std::env::temp_dir().join(format!("tianheng-spelling-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    xingbiao::claim_scratch(&dir).expect("the rustc scratch root is writable");
    let out = dir.join("out");
    std::fs::create_dir_all(&out).expect("create out dir");
    std::fs::write(
        dir.join("target.rs"),
        "pub fn only_in_target() -> u8 { 7 }\n",
    )
    .expect("write");
    std::fs::write(dir.join("imp.rs"), CLEAN).expect("write");
    let lib = dir.join("lib.rs");
    std::fs::write(&lib, source).expect("write");

    let run = std::process::Command::new("rustc")
        .args(["--crate-type", "lib", "--edition", "2021"])
        .arg(&lib)
        .arg("--out-dir")
        .arg(&out)
        .output()
        .map_err(|err| format!("rustc could not be run: {err}"))?;
    let verdict = if run.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&run.stderr)
            .lines()
            .next()
            .unwrap_or("(no stderr)")
            .to_string())
    };
    let _ = std::fs::remove_dir_all(&dir);
    verdict
}

/// The shape with no cross-module reference: the source whose legality is the generator's claim.
fn plain(attribute: &str) -> String {
    format!("{attribute}\npub mod imp;\n")
}

/// The same source, resolving an item defined **only** in the named target — so success is rustc saying
/// the remap applied and the build contains that file.
fn referencing_the_target(attribute: &str) -> String {
    format!("{attribute}\npub mod imp;\npub fn use_it() -> u8 {{ imp::only_in_target() }}\n")
}

/// A look-alike's live probe: the same spelling with its false predicate made true, so *no remap applies*
/// is a fact rustc can be asked rather than one the generator asserts. A shape carrying no `any()` is
/// already live — its predicate is the look-alike — and is probed as written.
fn live_probe(attribute: &str) -> String {
    attribute.replace("any()", "unix")
}

/// Every generated spelling is legal Rust, and every dimension answers it the way the generator declares —
/// which is also the way the other two answer it.
#[test]
fn every_generated_spelling_is_answered_the_same_way_by_every_dimension() {
    if std::env::var_os("TIANHENG_SPELLING_DIFFERENTIAL").is_none() {
        println!(
            "spelling differential: skipped — set TIANHENG_SPELLING_DIFFERENTIAL=1 to run it. It compiles \
             one crate per shape, so it is named on its own line in the Definition of Done and in CI \
             rather than made part of the ordinary suite."
        );
        return;
    }

    let mut offences: Vec<String> = Vec::new();
    let corpus = corpus();
    for (i, shape) in corpus.iter().enumerate() {
        let name = format!("diff-{i:02}");

        // **Question one, and the generator is what it holds.** A shape rustc will not take is not a
        // spelling a maintainer could write, and reporting the dimensions' answers about it would attribute
        // the generator's defect to them. No cross-module reference here, so this failure can mean only
        // one thing.
        if let Err(stderr) = compiles(&format!("{name}-plain"), &plain(&shape.attribute)) {
            offences.push(format!(
                "{}: rustc will not take the generated spelling `{}`, so the corpus claims a shape Rust \
                 does not admit — {stderr}",
                shape.label, shape.attribute
            ));
            continue;
        }

        // **Question two: is the declared answer true of a real build?** The same source, resolving an item
        // defined only in the named target. A failure here is the *declaration*, not the spelling — the
        // two were one arm and the arm named only the first.
        match shape.answer {
            Answer::Governed if shape.live_predicate => {
                if let Err(stderr) = compiles(
                    &format!("{name}-ref"),
                    &referencing_the_target(&shape.attribute),
                ) {
                    offences.push(format!(
                        "{}: rustc takes `{}` but the build does not contain the named target, so the \
                         corpus declares a remap that does not apply — {stderr}",
                        shape.label, shape.attribute
                    ));
                    continue;
                }
            }
            Answer::Decoy => {
                // Measured rather than asserted: with the predicate made live, the look-alike must still
                // name no remap — either rustc refuses the attribute outright, or it compiles and the
                // target's item does not resolve. Both are *no remap applies*; a success would mean the
                // generator called a real remap a decoy.
                let probe = live_probe(&shape.attribute);
                if compiles(&format!("{name}-decoy"), &referencing_the_target(&probe)).is_ok() {
                    offences.push(format!(
                        "{}: with its predicate made live, `{probe}` DOES apply a remap, so the corpus \
                         calls a real module target a look-alike",
                        shape.label
                    ));
                    continue;
                }
            }
            // A governed shape under a dead predicate: rustc on this host compiles no configuration that
            // reaches the target, so question one is all it can answer and the declaration carries the
            // rest. The live rows above are what keep that declaration honest.
            Answer::Governed => {}
        }

        let lib = format!("{FORBIDDEN}{PROBED}{}\npub mod imp;\n", shape.attribute);
        let fixture = TempFixture::new(&name, &lib);
        let src = fixture.lib().parent().expect("lib.rs has a parent");
        std::fs::write(src.join("imp.rs"), CLEAN).expect("write the conventional file");
        std::fs::write(src.join("target.rs"), VIOLATIONS).expect("write the named target");

        let expected = match shape.answer {
            Answer::Governed => 1,
            Answer::Decoy => 0,
        };
        let answers = [
            (
                "圭表",
                guibiao_exit(&name, fixture.manifest(), "crate::imp", REASON),
            ),
            (
                "渾儀",
                hunyi_exit(&name, fixture.manifest(), "crate::imp", REASON),
            ),
            ("漏刻", louke_exit(fixture.lib(), SEAM, REASON)),
        ];
        for (dimension, got) in answers {
            if got != expected {
                offences.push(format!(
                    "{}: {dimension} answered {got} where the spelling `{}` declares {expected} ({})",
                    shape.label,
                    shape.attribute,
                    match shape.answer {
                        Answer::Governed =>
                            "the named target is a candidate every dimension governs, so the violation \
                             inside it reacts",
                        Answer::Decoy =>
                            "no configuration compiles the named file, so governing it reports a violation \
                             against source the tree does not have",
                    }
                ));
            }
        }
    }

    assert!(
        offences.is_empty(),
        "{} of {} generated spellings were answered wrongly:\n  {}",
        offences.len(),
        corpus.len(),
        offences.join("\n  ")
    );
    // **Every declared variant of every axis produced a shape**, and the axes are held against a set this
    // direction declares for itself.
    //
    // Two holes, and the second is why the declared arrays are here. A branch that stops emitting is a
    // corpus that shrank, and a shrinking corpus passes — measured by making the direct branch emit
    // nothing: *no generated shape occupies Direct*. But a floor iterating the same `ALL` the corpus
    // iterates cannot see that array **trimmed**: measured, `Value::ALL` cut to `[Ordinary]` drops 43 of
    // 90 shapes and every assertion here passes, because the floor asks about the set the corpus was built
    // from rather than about the set the corpus is supposed to cover. So the expectation is *declared*
    // and held to each enumerator both ways, which is what gives a trimmed enumerator something to
    // disagree with — the shape `AGENTS.md` records for a claim something downstream filters on.
    //
    // Negative runs, each against the assertion built for it:
    //   the value axis is declared here and enumerated by the corpus … neither may be trimmed alone
    //     left: "[Ordinary, Raw]"   right: "[Ordinary]"
    //   the two value spellings are the whole of this axis …
    //     left: "\"target.rs\""    right: "\"target.rs\""
    //   no generated shape occupies Direct, so this direction would report clean over a corpus that no
    //   longer covers every declared position
    const EXPECTED_POSITIONS: [Position; 2] = [Position::Direct, Position::CfgAttrWrapped];
    const EXPECTED_VALUES: [Value; 2] = [Value::Ordinary, Value::Raw];
    for (axis, declared, enumerated) in [
        (
            "position",
            format!("{EXPECTED_POSITIONS:?}"),
            format!("{:?}", Position::ALL),
        ),
        (
            "value",
            format!("{EXPECTED_VALUES:?}"),
            format!("{:?}", Value::ALL),
        ),
    ] {
        assert_eq!(
            declared, enumerated,
            "the {axis} axis is declared here and enumerated by the corpus, and these are two statements \
             of one membership, in one order, held in both directions: neither may be trimmed alone. A \
             reorder fails here too — the comparison is over the rendered sequences, because the two axes \
             are different types and one loop holds both"
        );
    }
    for position in EXPECTED_POSITIONS {
        assert!(
            corpus.iter().any(|shape| shape.position == position),
            "no generated shape occupies {position:?}, so this direction would report clean over a corpus \
             that no longer covers every declared position"
        );
    }
    for value in EXPECTED_VALUES {
        assert!(
            corpus.iter().any(|shape| shape.value == value),
            "no generated shape spells its value as {value:?}, so this direction would report clean over a \
             corpus that no longer covers every declared value spelling"
        );
    }
    // **An axis whose variants coincide is not an axis.** The floors above read the value each shape was
    // *labelled* with, so a `spell` that stopped distinguishing the two would satisfy every one of them
    // while generating one spelling twice. This is the assertion that says the axis does something, and it
    // is independent of both arrays above.
    assert_ne!(
        Value::Ordinary.spell("target.rs"),
        Value::Raw.spell("target.rs"),
        "the two value spellings are the whole of this axis, so a spelling that answers both the same way \
         leaves the corpus generating one form twice under two labels"
    );
    // The same obligation for the other axis, and it was owed from the moment `Position` existed. A shape's
    // `position` is set from the loop variable and its `attribute` from the match arm, so the two can part:
    // measured, a `Direct` arm emitting `#[cfg_attr(unix, {meta})]` passes the set agreement, both value
    // assertions and the per-position floor, reports the same 90 spellings, and leaves the direct attribute
    // position — the axis this corpus grew to cover — unexercised. Matched rather than iterated over an
    // array, so a new variant is a compile error here as it is in `corpus`.
    //
    // Negative run:
    //   direct · path · ordinary value is labelled a direct attribute and its source is `cfg_attr`-wrapped:
    //   #[cfg_attr(unix, path = "target.rs")]
    for shape in &corpus {
        let wrapped = shape.attribute.contains("cfg_attr");
        match shape.position {
            Position::Direct => assert!(
                !wrapped,
                "{} is labelled a direct attribute and its source is `cfg_attr`-wrapped: {}",
                shape.label, shape.attribute
            ),
            Position::CfgAttrWrapped => assert!(
                wrapped,
                "{} is labelled a wrapped attribute and its source carries no `cfg_attr`: {}",
                shape.label, shape.attribute
            ),
        }
    }
    let (governed, decoys) = corpus
        .iter()
        .fold((0, 0), |(g, d), shape| match shape.answer {
            Answer::Governed => (g + 1, d),
            Answer::Decoy => (g, d + 1),
        });
    println!(
        "spelling differential: {} spellings — {governed} governed and {decoys} look-alikes. Every one is \
         legal Rust; every look-alike was shown to apply no remap with its predicate made live; and every \
         dimension answered every one the way the corpus declares.",
        corpus.len()
    );
}
