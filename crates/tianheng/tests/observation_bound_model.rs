//! `observation-bound-model`'s reaction: the specs' declared bounds and the code's typed declarations are one
//! set, and the classification is projected where a reader can count it.
//!
//! Why this lives in the composed shell rather than in a dimension: it is the only crate that sees 圭表, 渾儀
//! and 漏刻 at once, and the bijection is meaningless from inside one of them. Why it is a Rust reaction rather
//! than a seventh shell gate: a `PINNED-BY` citation resolves only to a harness-registered Rust function, so a
//! shell-defended capability could not pin the bounds this one declares — they would land `UNPINNED` and turn
//! the register projection's leading figure from zero into three.
//!
//! What it does **not** take over: `scripts/check_bound_register.sh` still owns the citation, tracker, prose and
//! projection directions. This reaction owns one obligation — that every declared bound is classified, and every
//! classification names a declared bound.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tianheng::testing::assert_projection_matches;
use tianheng::{BoundDecl, Extent, Owner, Reached};

/// The projection this reaction holds fresh.
const EXTENT_PROJECTION: &str = "docs/observation-bound-extents.md";

/// The workspace root, or `None` outside a checkout.
///
/// Same discipline six crates already follow: absent layout is a skip outside a checkout and a LOUD failure
/// when `TIANHENG_WORKSPACE_TESTS` is set. A governance reaction that quietly does nothing in CI is the shape
/// this whole capability argues against.
fn workspace_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    if root.join("openspec/specs").is_dir() {
        return Some(root);
    }
    assert!(
        std::env::var_os("TIANHENG_WORKSPACE_TESTS").is_none(),
        "openspec/specs expected under {root:?} but absent while TIANHENG_WORKSPACE_TESTS is set — \
         the bound-model reaction must not silently skip in CI"
    );
    None
}

/// The slug rule the register derives an id with: lowercased, each run of non-alphanumerics collapsed to one
/// hyphen, ends trimmed.
///
/// A second implementation of one rule is the divergence `check_bound_register.sh` has already paid for — its
/// own comment records a review round lost to two matchers whose character classes differed. So this one is not
/// trusted on its own: [`derived_ids_agree_with_the_register_projection`] asserts the set it produces equals the
/// set the shell wrote, which catches a drifted rule and a stale projection in the same assertion.
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

/// Whether a scenario heading marks itself a bound, in the one form the register now admits: the marker word
/// adjacent to "bound", no interposed qualifier.
fn marks_a_bound(heading: &str) -> bool {
    heading.contains("stated bound")
        || heading.contains("documented bound")
        || heading.contains("stated bounds")
        || heading.contains("documented bounds")
}

/// One bound as the specs declare it: where it sits, and the test it cites.
struct SpecBound {
    file: String,
    pinned_by: Option<String>,
}

/// Every declared bound the specs state, keyed by derived id.
///
/// Fails loudly on an empty enumeration in either direction. A reaction reporting a perfect bijection between
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
                            pinned_by: None,
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
                if let Some(rest) = line.trim().strip_prefix("- **PINNED-BY** ") {
                    let name = rest.trim().trim_matches('`').to_string();
                    bounds
                        .get_mut(id)
                        .expect("the open bound was just inserted")
                        .pinned_by = Some(name);
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

/// Every typed declaration the dimensions export.
///
/// Duplicate ids are refused here for the same reason as on the spec side.
fn declared_bounds() -> BTreeMap<String, BoundDecl> {
    let mut all = BTreeMap::new();
    for decl in guibiao::observation_bounds()
        .into_iter()
        .chain(hunyi::observation_bounds())
        .chain(louke::observation_bounds())
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
        let cited = spec.pinned_by.as_deref();
        // A spec citation may be crate-qualified (`hunyi::name`); the declaration transcribes it verbatim.
        if cited != Some(decl.pinned_by()) {
            disagreements.push(format!(
                "{id}: {} cites `{}`, the declaration cites `{}`",
                spec.file,
                cited.unwrap_or("<no PINNED-BY>"),
                decl.pinned_by()
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
        "the ids this reaction derives differ from the ids `scripts/check_bound_register.sh` wrote into \
         {projection:?} — either the slug rule has drifted between the two implementations, or the projection \
         is stale. Regenerate with `BLESS=1 bash scripts/check_bound_register.sh` and, if the difference \
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
         `crates/tianheng/tests/observation_bound_model.rs`. **Do not edit by hand** — regenerate with\n\
         `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --test observation_bound_model`.\n\n",
    );
    out.push_str(
        "**What this document does not claim.** The classification is *authored*: the type refuses a \
         contradiction and derives what each bound's defence must demonstrate, but nothing verifies that a \
         bound recorded as over-reacting really over-reacts rather than under-reacting. Two further limits are \
         declared as bounds of `observation-bound-model` itself: an answer that depends on which corpus entry \
         point observed it has no extent of its own and is recorded as under-reacting with the entry point as \
         its owner, and a bound both out of reach and granularity-limited cannot be expressed at all.\n\n",
    );
    out.push_str(
        "One value carries no bound today and is kept deliberately: **refuses to judge**. The \
         misclassification this model exists to prevent was exactly a confusion between that and *out of \
         reach* — a prediction of a silent false negative where the real behaviour was a fail-loud refusal — \
         and a direction that cannot be named cannot be predicted with.\n",
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
            out.push_str(&format!("- **pinned by**: `{}`\n", decl.pinned_by()));
        }
    }
    out
}

/// The rationale an extent carries, for the projection. Prose the model does not read — see this capability's
/// own declared bound on exactly that.
///
/// The wildcard arms are what `#[non_exhaustive]` buys and are not an oversight: adding a *new answer* to an
/// existing question must not break a reader, where adding a new *question* — a stage every declaration must
/// answer — is a trait method with no default and breaks every one of them. Only the second kind of addition
/// should force re-examination, and a projection that refused to render an unknown value would invert that.
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
        _ => "(a value this projection does not yet render)",
    }
}
