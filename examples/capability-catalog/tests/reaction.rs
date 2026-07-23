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
                violation.fact().fact_type(),
                violation.fact().shape(),
                violation.reason.as_str(),
            )
        })
        .collect();

    for expected in [
        (
            "crate",
            "restrict dependency sources to",
            "tianheng.fact/guibiao/dependency-source",
            "declared-source",
            "catalog source metadata must produce its declared source reaction",
        ),
        (
            "module",
            "external crate confined to module",
            "tianheng.fact/guibiao/external-importer",
            "module-path",
            "the external shell dependency stays behind the governance module",
        ),
        (
            "semantic",
            "must only be implemented in the declared location(s)",
            "tianheng.fact/hunyi/trait-impl-site",
            "misplaced-implementation",
            "Command implementations live only under the allowed subtree",
        ),
        (
            "semantic",
            "must not acquire trait",
            "tianheng.fact/hunyi/forbidden-marker-acquisition",
            "impl",
            "marked-domain types remain free of the catalog marker",
        ),
        (
            "semantic",
            "must not expose dyn",
            "tianheng.fact/hunyi/dyn-trait-exposure",
            "public-seam",
            "the catalog dyn family must produce its structured reaction",
        ),
        (
            "semantic",
            "must not expose impl trait",
            "tianheng.fact/hunyi/impl-trait-exposure",
            "public-seam",
            "the catalog impl-trait family must produce its structured reaction",
        ),
        (
            "semantic",
            "must not expose impl trait",
            "tianheng.fact/hunyi/impl-trait-exposure",
            "public-seam",
            "the catalog's composed no-existential-leak profile must produce its structured \
             reaction for both the written and the implicit existential signal",
        ),
        (
            "semantic",
            "must not expose async fn",
            "tianheng.fact/hunyi/async-exposure",
            "async-free-function",
            "the catalog's composed no-existential-leak profile must produce its structured \
             reaction for both the written and the implicit existential signal",
        ),
    ] {
        assert!(
            observed.contains(&expected),
            "missing structured reaction owner for `{expected:?}`: {observed:#?}"
        );
    }
}
