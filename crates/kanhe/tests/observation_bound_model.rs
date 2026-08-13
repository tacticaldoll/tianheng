//! `observation-bound-model`'s check: the specs' declared bounds and the code's typed declarations are one
//! set, and the classification is projected where a reader can count it.
//!
//! Why this lives in Kanhe rather than in a dimension: this unpublished repository-governance crate sees
//! 圭表, 渾儀 and 漏刻 through Tianheng, and the bijection is meaningless from inside one of them. Why it is a Rust check rather
//! than a seventh shell gate: a `PINNED-BY` citation resolves only to a harness-registered Rust function, so a
//! shell-defended capability could not pin the bounds this one declares — they would land `UNPINNED` and
//! increase the register projection's audit backlog.
//!
//! What it does **not** take over: `crates/kanhe/tests/bound_register.rs` still owns the citation, tracker, prose and
//! projection directions. This check owns one obligation — that every declared bound is classified, and every
//! classification names a declared bound.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::Command;

use tianheng::prelude::*;
use tianheng::testing::assert_projection_matches;
use tianheng::{BoundDecl, Defence, Extent, Owner, Reached};

use kanhe::bound_register_parse::marks_a_bound;

/// The projection this check holds fresh.
const EXTENT_PROJECTION: &str = "docs/observation-bound-extents.md";

/// The workspace root, or `None` outside a checkout.
///
/// Same discipline six crates already follow: absent layout is a skip outside a checkout and a LOUD failure
/// when `TIANHENG_WORKSPACE_TESTS` is set. A governance check that quietly does nothing in CI is the shape
/// this whole capability argues against.
fn workspace_root() -> Option<PathBuf> {
    shengmo::workspace::locate(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."),
        |root| root.join("openspec/specs").is_dir(),
        shengmo::workspace::marker_set(),
    )
}

/// The slug rule the register derives an id with: lowercased, each run of non-alphanumerics collapsed to one
/// hyphen, ends trimmed.
///
/// A second implementation of one rule is the divergence `crates/kanhe/tests/bound_register.rs` has already paid
/// for — its own comment records a review round lost to two matchers whose character classes differed. So this
/// one is not trusted on its own: [`derived_ids_agree_with_the_register_projection`] asserts the set it produces
/// equals the set the register wrote, which catches a drifted rule and a stale projection in the same assertion.
fn slug_of(heading: &str) -> String {
    let mut out = String::with_capacity(heading.len());
    let mut pending_hyphen = false;
    for ch in heading.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }
    out
}

/// One bound as the specs declare it: where it sits, and the test it cites.
struct SpecBound {
    file: String,
    defence: Option<SpecDefence>,
}

#[derive(Debug, PartialEq, Eq)]
enum SpecDefence {
    PinnedBy(Vec<String>),
    Unpinned { tracker: String },
}

fn declared_defence(decl: &BoundDecl) -> SpecDefence {
    match decl.defence() {
        Defence::PinnedBy { .. } => SpecDefence::PinnedBy(
            decl.defence()
                .pinning_tests()
                .expect("the matched pinned defence carries tests")
                .map(str::to_string)
                .collect(),
        ),
        Defence::Unpinned { tracker } => SpecDefence::Unpinned {
            tracker: tracker.to_string(),
        },
        _ => panic!(
            "{}: the bound model does not know how to compare this defence variant",
            decl.id().as_str()
        ),
    }
}

fn spec_defence(line: &str) -> Option<SpecDefence> {
    let line = line.trim();
    if let Some(rest) = line.strip_prefix("- **PINNED-BY** ") {
        return Some(SpecDefence::PinnedBy(vec![
            rest.trim().trim_matches('`').to_string(),
        ]));
    }
    line.strip_prefix("- **UNPINNED** ")
        .map(|tracker| SpecDefence::Unpinned {
            tracker: tracker.trim().to_string(),
        })
}

fn unpinned_fixture() -> BoundDecl {
    BoundDecl::unpinned(
        tianheng::BoundId::new("probe-capability/an-unpinned-fixture-a-stated-bound"),
        "a synthetic bound kept outside the live declaration set",
        Extent::OutOfReach {
            because: "the fixture exists only to exercise a supported defence state".into(),
        },
        "BACKLOG.md READY-PATCH fixture-defence",
    )
}

/// Every declared bound the specs state, keyed by derived id.
///
/// Fails loudly on an empty enumeration in either direction. A check reporting a perfect bijection between
/// two empty sets is the vacuity this repository has re-opened six times in one window.
fn spec_bounds(root: &Path) -> BTreeMap<String, SpecBound> {
    let specs_dir = root.join("openspec/specs");
    let mut capabilities: Vec<PathBuf> = std::fs::read_dir(&specs_dir)
        .unwrap_or_else(|err| panic!("cannot read {specs_dir:?}: {err}"))
        .map(|entry| entry.expect("a readable directory entry").path())
        .filter(|path| path.join("spec.md").is_file())
        .collect();
    capabilities.sort();
    assert!(
        !capabilities.is_empty(),
        "no capability spec found under {specs_dir:?}; a bijection over an empty set holds while proving nothing"
    );

    let mut bounds = BTreeMap::new();
    for capability_dir in capabilities {
        let capability = capability_dir
            .file_name()
            .expect("a capability directory has a name")
            .to_string_lossy()
            .into_owned();
        let spec = capability_dir.join("spec.md");
        let text = std::fs::read_to_string(&spec)
            .unwrap_or_else(|err| panic!("cannot read {spec:?}: {err}"));

        let mut open: Option<String> = None;
        for line in text.lines() {
            if let Some(heading) = line.strip_prefix("#### Scenario: ") {
                open = marks_a_bound(heading).then(|| format!("{capability}/{}", slug_of(heading)));
                if let Some(id) = &open {
                    let previous = bounds.insert(
                        id.clone(),
                        SpecBound {
                            file: format!("openspec/specs/{capability}/spec.md"),
                            defence: None,
                        },
                    );
                    assert!(
                        previous.is_none(),
                        "two declared bounds derive the id `{id}`; set equality would hold while one of them \
                         went unclassified, so duplicates are refused before the sets are compared"
                    );
                }
                continue;
            }
            if line.starts_with("###") || line.starts_with("## ") {
                open = None;
                continue;
            }
            if let Some(id) = &open {
                if let Some(defence) = spec_defence(line) {
                    let bound = bounds
                        .get_mut(id)
                        .expect("the open bound was just inserted");
                    match (&mut bound.defence, defence) {
                        (None, defence) => bound.defence = Some(defence),
                        (Some(SpecDefence::PinnedBy(tests)), SpecDefence::PinnedBy(mut more)) => {
                            tests.append(&mut more);
                        }
                        _ => panic!("{id} mixes pinned and unpinned defence states"),
                    }
                }
            }
        }
    }

    assert!(
        !bounds.is_empty(),
        "no declared bound found under {specs_dir:?}; the heading marker may have changed, and a bijection \
         over an empty set holds while proving nothing"
    );
    bounds
}

#[test]
fn an_unpinned_spec_defence_keeps_its_tracker_and_no_test() {
    assert_eq!(
        spec_defence("- **UNPINNED** BACKLOG.md READY-PATCH missing-defence"),
        Some(SpecDefence::Unpinned {
            tracker: "BACKLOG.md READY-PATCH missing-defence".to_string(),
        })
    );
}

#[test]
fn an_unpinned_typed_defence_compares_with_its_tracker() {
    assert_eq!(
        declared_defence(&unpinned_fixture()),
        SpecDefence::Unpinned {
            tracker: "BACKLOG.md READY-PATCH fixture-defence".to_string(),
        }
    );
}

#[test]
fn an_unpinned_typed_defence_projects_its_tracker() {
    let fixture = unpinned_fixture();
    let mut declarations = BTreeMap::new();
    declarations.insert(fixture.id().as_str().to_string(), fixture);

    let rendered = render_extents(&declarations);
    assert!(
        rendered.lines().any(
            |line| line == "- **unpinned**, tracked by: BACKLOG.md READY-PATCH fixture-defence"
        ),
        "the unpinned projection must preserve the tracker in the register vocabulary:\n{rendered}"
    );
}

/// Every typed declaration the dimensions export.
///
/// Duplicate ids are refused here for the same reason as on the spec side.
fn declared_bounds() -> BTreeMap<String, BoundDecl> {
    let mut all = BTreeMap::new();
    // Each dimension is asked **through `Observer::bounds`**, not through its free function. That method is the
    // protocol's whole justification — a participant must declare what it does not observe — and until this it
    // had no consumer at all: nothing in the tree read it, so a dimension could have answered anything. Now the
    // bijection's verdict depends on the answer.
    //
    // Repository-governance declarations are chained from the unpublished crate whose check they qualify:
    // Kanhe for record/coherence checks and Shengmo for self-law dogfood. The published shell is **not** an
    // observer and has no catalog of its own; it composes product dimensions rather than restating them.
    for decl in StaticObserver::new(Constitution::new("bounds").static_boundaries().clone())
        .bounds()
        .into_iter()
        .chain(
            SemanticObserver::new(Constitution::new("bounds").semantic_boundaries().clone())
                .bounds(),
        )
        .chain(RuntimeObserver::new(Vec::new()).bounds())
        .chain(kanhe::bounds::observation_bounds())
        .chain(shengmo::bounds::observation_bounds())
    {
        let id = decl.id().as_str().to_string();
        assert!(
            all.insert(id.clone(), decl).is_none(),
            "two typed declarations carry the id `{id}`; one of them classifies nothing"
        );
    }
    assert!(
        !all.is_empty(),
        "no dimension exported a typed declaration; a bijection over an empty set holds while proving nothing"
    );
    all
}

#[test]
fn the_published_shell_defines_no_repository_bound_catalog() {
    let Some(root) = workspace_root() else {
        return;
    };
    let listed = Command::new("git")
        .args([
            "-C",
            root.to_str().expect("UTF-8 root"),
            "ls-files",
            "-z",
            "--",
            "crates/tianheng/src",
        ])
        .output()
        .expect("git must enumerate Tianheng's tracked source");
    assert!(
        listed.status.success(),
        "git could not enumerate Tianheng's tracked source: {}",
        String::from_utf8_lossy(&listed.stderr)
    );

    let offenders: Vec<String> = listed
        .stdout
        .split(|byte| *byte == 0)
        .filter(|raw| !raw.is_empty())
        .map(|raw| String::from_utf8(raw.to_vec()).expect("tracked source path must be UTF-8"))
        .filter(|file| file.ends_with(".rs"))
        .filter(|file| {
            std::fs::read_to_string(root.join(file))
                .unwrap_or_else(|err| panic!("cannot read {file}: {err}"))
                .contains("observation_bounds")
        })
        .collect();

    assert!(
        offenders.is_empty(),
        "the published Tianheng source defines repository bound-catalog vocabulary in {offenders:?}"
    );
}

/// Every one of the family's own declarations borrows every string it carries.
///
/// The specification says the family's declarations stay literals and that the owned-or-borrowed form exists for
/// implementors whose reactions do not know their limits when they are written. Nothing measured it: the
/// constructors accept anything convertible, so a declaration rewritten as `format!(…)` compiles, allocates on
/// every pass of this check and of the projection below, and would be named by nothing.
///
/// The counter-example is deliberate and lives outside this workspace — `examples/observer-participant`'s
/// declarations are computed on purpose, which is what the form is for. This check is about *these*
/// declarations, so it reads what `declared_bounds` composes and nothing else.
#[test]
fn every_declaration_of_this_family_borrows_every_string_it_carries() {
    let declarations = declared_bounds();
    // Not vacuous: an empty set would satisfy the filter below. `declared_bounds()` already refuses to be empty,
    // and this states the dependency where a reader of this assertion is standing.
    assert!(
        !declarations.is_empty(),
        "no declaration was read, so this check would hold over nothing"
    );
    let allocating: Vec<&str> = declarations
        .values()
        .filter(|decl| !decl.borrows_every_string())
        .map(|decl| decl.id().as_str())
        .collect();
    assert!(
        allocating.is_empty(),
        "these declarations carry a computed string: {allocating:?} — the family's own bounds are literals \
         because a bound is a property of a reaction that knows its limits when it is written, and the owned \
         form exists for implementors whose reactions do not"
    );
}

#[test]
fn every_declared_bound_is_classified_and_every_classification_names_one() {
    let Some(root) = workspace_root() else {
        return;
    };
    let specs = spec_bounds(&root);
    let code = declared_bounds();

    let spec_ids: BTreeSet<&str> = specs.keys().map(String::as_str).collect();
    let code_ids: BTreeSet<&str> = code.keys().map(String::as_str).collect();

    // Both directions, for the reason the register requires both of its own: a spec bound with no declaration
    // is an unclassified claim, and a declaration with no spec bound is a classification no reader can find.
    let unclassified: Vec<&&str> = spec_ids.difference(&code_ids).collect();
    assert!(
        unclassified.is_empty(),
        "declared in a spec and classified nowhere: {unclassified:?} — the qualifier slot that used to carry \
         a classification is gone, so an unclassified bound would otherwise pass silently"
    );
    let orphaned: Vec<&&str> = code_ids.difference(&spec_ids).collect();
    assert!(
        orphaned.is_empty(),
        "classified in code and declared in no spec: {orphaned:?} — a classification a spec reader cannot \
         find is a fact recorded where nobody looks"
    );
}

#[test]
fn every_classification_cites_the_test_its_spec_cites() {
    let Some(root) = workspace_root() else {
        return;
    };
    let specs = spec_bounds(&root);
    let code = declared_bounds();

    // Ids alone would let a declaration name a different defence than its spec does, and the extent predicts
    // what that defence must demonstrate — so a mis-transcribed pin makes the prediction land on the wrong test.
    let mut disagreements = Vec::new();
    for (id, decl) in &code {
        let Some(spec) = specs.get(id) else {
            continue; // the bijection test above owns that direction
        };
        let declared = declared_defence(decl);
        if spec.defence.as_ref() != Some(&declared) {
            disagreements.push(format!(
                "{id}: {} declares {:?}, the typed declaration carries {:?}",
                spec.file, spec.defence, declared
            ));
        }
    }
    assert!(
        disagreements.is_empty(),
        "a classification names a different defence than its spec does:\n{}",
        disagreements.join("\n")
    );
}

#[test]
fn derived_ids_agree_with_the_register_projection() {
    let Some(root) = workspace_root() else {
        return;
    };
    // The shell gate derives these ids too, and this is the only guard against the two rules drifting. It
    // catches a stale projection in the same assertion, which is why reading the projection INSTEAD of deriving
    // was rejected: `cargo test` runs before that gate in the Definition of Done, so a stale projection would
    // let the bijection pass while the specs and the code disagreed.
    let projection = root.join("docs/observation-bounds.md");
    let text = std::fs::read_to_string(&projection)
        .unwrap_or_else(|err| panic!("cannot read {projection:?}: {err}"));
    let projected: BTreeSet<String> = text
        .lines()
        .filter_map(|line| line.strip_prefix("### `"))
        .filter_map(|rest| rest.strip_suffix('`'))
        .map(str::to_string)
        .collect();
    assert!(
        !projected.is_empty(),
        "no bound id parsed from {projection:?}; its shape may have changed, and an empty set agrees with \
         anything"
    );

    let derived: BTreeSet<String> = spec_bounds(&root).into_keys().collect();
    assert_eq!(
        derived, projected,
        "the ids this check derives differ from the ids `crates/kanhe/tests/bound_register.rs` wrote into \
         {projection:?} — either the slug rule has drifted between the two implementations, or the projection \
         is stale. Regenerate with `BLESS=1 bash crates/kanhe/tests/bound_register.rs` and, if the difference \
         survives, the two derivations disagree."
    );
}

#[test]
fn the_extent_projection_is_fresh() {
    let Some(root) = workspace_root() else {
        return;
    };
    let code = declared_bounds();
    let rendered = render_extents(&code);
    assert_projection_matches(&root, EXTENT_PROJECTION, &rendered);
}

/// The projection: every declared bound grouped by where its measure stops.
///
/// It leads with the count of declared false negatives and their owners, for the same reason the register's
/// projection leads with its unpinned count — a number in a footnote is not read, and this one is the family's
/// own audit backlog.
fn render_extents(code: &BTreeMap<String, BoundDecl>) -> String {
    let mut out = String::new();
    out.push_str("# Observation bound extents\n\n");
    out.push_str(
        "Where each declared **observation bound** stops the measure — not how far a scan walks (that is\n\
         `ScanDepth`, an adopter's knob), but where this family's own reaction deliberately stops.\n\n",
    );

    let false_negatives: Vec<(&String, &BoundDecl)> = code
        .iter()
        .filter(|(_, decl)| decl.extent().is_declared_false_negative())
        .collect();
    out.push_str(&format!(
        "**{} of {} declared bounds are declared false negatives** — the reaction fires less than the truth, \
         which is the one direction this family treats as a defect. That figure leads this document because a \
         number in a footnote is not read, and each such bound names who must act:\n\n",
        false_negatives.len(),
        code.len()
    ));
    for (id, decl) in &false_negatives {
        if let Extent::Reached(Reached::UnderReacts { owner, .. }) = decl.extent() {
            let owner_text = match owner {
                Owner::Inherited { from } => format!("inherited from {from}"),
                other => other.as_str().to_string(),
            };
            out.push_str(&format!("- `{id}` — owner: {owner_text}\n"));
        }
    }

    out.push_str(
        "\nGenerated from each dimension's `observation_bounds()` by \
         `crates/kanhe/tests/observation_bound_model.rs`. **Do not edit by hand** — regenerate with\n\
         `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test observation_bound_model`.\n\n",
    );
    out.push_str(
        "**What this document does not claim.** The classification is *authored*: the type refuses a \
         contradiction and derives what each bound's defence must demonstrate, but nothing verifies that a \
         bound recorded as over-reacting really over-reacts rather than under-reacting. This capability declares \
         further limits as bounds of its own — among them, an answer that depends on which corpus entry point \
         observed it has no extent of its own and is recorded as under-reacting with the entry point as its \
         owner, and a bound both out of reach and granularity-limited cannot be expressed at all. The \
         sections below list every one of them; this paragraph deliberately does not, because a list typed \
         here is a literal in a template and the freshness check compares that text with itself. This \
         disclosure is authored rather than derived from the specification and held both ways; the backlog \
         carries that.\n\n",
    );
    // The membership claim this paragraph used to make — that **refuses to judge** carried no bound — was a
    // literal in this template, which is the one place a freshness check cannot catch a falsehood: the
    // comparison is the generator's own text against itself, so it stayed "true" while the sections below
    // rendered a bound under that very heading. The lesson it carries does not need the count, so the count is
    // gone rather than re-typed; which values carry bounds is what the sections are.
    out.push_str(
        "**refuses to judge** and *out of reach* are kept distinct deliberately. The misclassification this \
         model exists to prevent was exactly a confusion between them — a prediction of a silent false \
         negative where the real behaviour was a fail-loud refusal — and a direction that cannot be named \
         cannot be predicted with.\n",
    );

    let mut by_extent: BTreeMap<&str, Vec<(&String, &BoundDecl)>> = BTreeMap::new();
    for (id, decl) in code {
        by_extent
            .entry(decl.extent().as_str())
            .or_default()
            .push((id, decl));
    }
    for (extent, mut bounds) in by_extent {
        bounds.sort_by_key(|(id, _)| id.as_str());
        out.push_str(&format!("\n## {extent} ({})\n", bounds.len()));
        for (id, decl) in bounds {
            out.push_str(&format!("\n### `{id}`\n\n"));
            out.push_str(&format!("> {}\n\n", decl.shape()));
            out.push_str(&format!("- **because**: {}\n", because_of(decl.extent())));
            out.push_str(&format!(
                "- **its defence must show**: {}\n",
                decl.extent().demonstrates().as_str()
            ));
            match decl.defence() {
                Defence::PinnedBy { .. } => {
                    for test in decl
                        .defence()
                        .pinning_tests()
                        .expect("the matched pinned defence carries tests")
                    {
                        out.push_str(&format!("- **pinned by**: `{test}`\n"));
                    }
                }
                Defence::Unpinned { tracker } => {
                    out.push_str(&format!("- **unpinned**, tracked by: {tracker}\n"));
                }
                // Refuses rather than rendering a placeholder, for the reason [`because_of`] states once for
                // every unrendered-variant arm in this file.
                _ => panic!(
                    "the extent projection does not know this defence variant; extend this arm rather than \
                     blessing a document that says it cannot render its own subject"
                ),
            }
        }
    }
    out
}

/// The rationale an extent carries, for the projection. Prose the model does not read — see this capability's
/// own declared bound on exactly that.
///
/// # The one policy this file holds for an unrendered variant, stated once
///
/// Every wildcard arm here is unreachable in this tree: it exists only because the extent and defence enums are
/// `#[non_exhaustive]` in another crate, and no value they carry goes unrendered. What the arm decides is which
/// message the author meets on the day a variant is added — and it decides *only* that, because a new variant
/// **used by a declaration** makes this projection stale whatever the arm does, so
/// [`assert_projection_matches`] fails either way.
///
/// This arm used to return a placeholder, argued for on the ground that adding a new *answer* to an existing
/// question must not force re-examination the way adding a new *question* does. That argument does not reach
/// this file: re-examination arrives regardless, as a stale projection. What the placeholder changes is only
/// what a `BLESS=1` then writes into the tracked document — the string "a value this projection does not yet
/// render", a generated document admitting it cannot render its own subject. A test naming the file to extend
/// is the better first message, so all three arms refuse alike.
fn because_of(extent: &Extent) -> &str {
    match extent {
        Extent::OutOfReach { because }
        | Extent::Reached(
            Reached::RefusesToJudge { because }
            | Reached::DeclinesToRefuse { because }
            | Reached::OverReacts { because }
            | Reached::UnderReacts { because, .. }
            | Reached::NotAViolation { because }
            | Reached::AsIntended { because, .. },
        ) => because,
        _ => panic!(
            "the extent projection does not know this extent variant; extend `because_of` rather than \
             blessing a document that says it cannot render its own subject"
        ),
    }
}

// --- this capability's own declared bounds, demonstrated ---
//
// A capability whose subject is honesty about what is not observed cannot be implicit about its own limits, so
// each of the three is a bound-marked scenario in `observation-bound-model`'s spec and each is demonstrated
// here. Every one shows the direction its extent predicts: the model does not react.

/// `observation-bound-model/whether-a-declaration-s-stated-cause-is-the-real-cause-is-not-observed-a-stated-bound`
///
/// The extent is typed and checkable; the rationale is prose. Requiring the prose to agree with the extent would
/// trade a fact for a heuristic, so a declaration whose rationale says the opposite of its extent is accepted.
#[test]
fn a_rationale_that_contradicts_its_extent_is_a_stated_bound() {
    let contradictory = xuanji_bound_decl_with_a_lying_rationale();
    // The model reports nothing about it: the extent still decides what the defence must demonstrate, and the
    // sentence disagreeing with it changes neither.
    assert_eq!(
        contradictory.extent().demonstrates(),
        tianheng::Demonstrates::DoesNotReact,
        "the extent decides the predicted evidence, never the rationale beside it"
    );
    assert!(
        because_of(contradictory.extent()).contains("over-reacts"),
        "the fixture's rationale must actually contradict its extent, or this bound is demonstrated by nothing"
    );
}

/// A declaration whose rationale claims the opposite of the extent it carries.
fn xuanji_bound_decl_with_a_lying_rationale() -> BoundDecl {
    BoundDecl::pinned(
        tianheng::BoundId::new("probe-capability/a-fixture-bound"),
        "a shape used only to demonstrate that the rationale is not read",
        Extent::OutOfReach {
            because: "this sentence claims the reaction over-reacts, which its extent denies"
                .into(),
        },
        "a_rationale_that_contradicts_its_extent_is_a_stated_bound",
    )
}

/// `observation-bound-model/an-answer-that-depends-on-the-corpus-entry-point-has-no-extent-of-its-own-a-stated-bound`
///
/// One declared bound's outcome differs by which entry point observed it. It is expressed through an existing
/// value rather than earning one: a single instance does not justify a value every other member has several of,
/// and the direction that matters — a seam reported covered when it is not — is recorded either way.
#[test]
fn an_entry_dependent_bound_is_declared_as_under_reacting() {
    let code = declared_bounds();
    let symlink = code
        .get(
            "runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound",
        )
        .expect("the entry-dependent bound is declared");
    match symlink.extent() {
        Extent::Reached(Reached::UnderReacts { owner, .. }) => assert!(
            matches!(owner, Owner::Inherited { from } if from.contains("entry point")),
            "the entry point is the layer that decides the answer, so the ownership is inherited from it"
        ),
        other => panic!(
            "an entry-dependent answer is recorded as an under-reaction, not as {}",
            other.as_str()
        ),
    }
}

/// `observation-bound-model/a-bound-both-out-of-reach-and-granularity-limited-cannot-be-expressed-a-stated-bound`
///
/// Granularity is carried only by the as-intended extent, so the pair cannot be written. What this test can
/// show is that the pair has no instance to express: every granularity-bounded declaration is one whose
/// reaction is exactly right.
#[test]
fn granularity_is_carried_only_by_the_as_intended_extent() {
    let code = declared_bounds();
    let granularity_bounded: Vec<&str> = code
        .iter()
        .filter(|(_, decl)| matches!(decl.extent(), Extent::Reached(Reached::AsIntended { .. })))
        .map(|(id, _)| id.as_str())
        .collect();
    assert!(
        !granularity_bounded.is_empty(),
        "no granularity-bounded declaration exists, so this bound would be demonstrated by an empty set"
    );
    // And no out-of-reach declaration carries one, which is what the type makes unwritable rather than what
    // this assertion discovers — stated here so a reader meets the claim beside its evidence.
    for (id, decl) in &code {
        if matches!(decl.extent(), Extent::OutOfReach { .. }) {
            assert_eq!(
                decl.extent().demonstrates(),
                tianheng::Demonstrates::DoesNotReact,
                "{id}: an out-of-reach bound has no granularity and no owner to carry"
            );
        }
    }
}
