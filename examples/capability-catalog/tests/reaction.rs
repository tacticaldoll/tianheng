//! Structured contract checks for the catalog's otherwise-unowned boundary families.

use std::path::PathBuf;

use capability_catalog::governance::constitution;
use tianheng::prelude::*;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

#[test]
fn uncovered_public_families_react_through_the_composed_evaluator() {
    let outcome = check_constitution(&constitution(), &manifest());
    assert_eq!(outcome.exit_code(), 1, "catalog must be deliberately red");
    let Outcome::Violations(report) = outcome else {
        panic!("catalog must return structured violations")
    };

    let observed: Vec<_> = report
        .violations
        .iter()
        .map(|violation| {
            (
                violation.kind.as_str(),
                violation.rule.as_str(),
                violation.finding_key().namespace(),
                violation.finding_key().code(),
                violation.reason.as_str(),
            )
        })
        .collect();

    for expected in [
        (
            "crate",
            "restrict dependency sources to",
            "guibiao",
            "dependency",
            "catalog source metadata must produce its declared source reaction",
        ),
        (
            "module",
            "external crate confined to module",
            "guibiao",
            "external_importer",
            "the external shell dependency stays behind the governance module",
        ),
        (
            "semantic",
            "must only be implemented in the declared location(s)",
            "hunyi",
            "trait_impl_site",
            "Command implementations live only under the allowed subtree",
        ),
        (
            "semantic",
            "must not acquire trait",
            "hunyi",
            "forbidden_marker_acquisition",
            "marked-domain types remain free of the catalog marker",
        ),
        (
            "semantic",
            "must not expose dyn",
            "hunyi",
            "dyn_trait_exposure",
            "the catalog dyn family must produce its structured reaction",
        ),
        (
            "semantic",
            "must not expose impl trait",
            "hunyi",
            "impl_trait_exposure",
            "the catalog impl-trait family must produce its structured reaction",
        ),
    ] {
        assert!(
            observed.contains(&expected),
            "missing structured reaction owner for `{expected:?}`: {observed:#?}"
        );
    }
}
