use super::helpers::*;
// ---- Feature-granularity crate rules (declared-not-resolved) --------------------------

// A `cargo metadata --no-deps` package declaring the dependency `C` under a variety of
// edges. Field names are exactly what cargo emits on a dependency edge: `features` (the
// authored list) and `uses_default_features` (the default toggle; absent ⇒ default-on).
pub(super) fn feature_package() -> Value {
    serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "source": "registry+https://x",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": true
            },
        ]
    })
}

#[test]
pub(super) fn declared_feature_is_observed() {
    // WHEN the target declares C = { features = ["extra"] } → the declared set contains `extra`.
    let package = feature_package();
    assert!(
        declared_features(&package, "C", DependencyKind::Normal).contains(&"extra".to_string()),
        "an authored feature is observed",
    );
}

#[test]
pub(super) fn default_features_are_the_default_pseudo_feature() {
    // WHEN C is declared without `default-features = false` (uses_default_features true, or
    // the field absent) → the declared set contains the `default` pseudo-feature.
    let with_flag = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": true },
        ]
    });
    let absent_flag = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null },
        ]
    });
    assert!(
        declared_features(&with_flag, "C", DependencyKind::Normal).contains(&"default".to_string()),
        "uses_default_features=true ⇒ default is requested",
    );
    assert!(
        declared_features(&absent_flag, "C", DependencyKind::Normal)
            .contains(&"default".to_string()),
        "an absent uses_default_features field ⇒ default-on ⇒ default is requested",
    );
}

#[test]
pub(super) fn disabling_default_features_drops_the_default_pseudo_feature() {
    // WHEN C = { default-features = false, features = ["extra"] } → declared set is
    // { extra }, without `default`.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "C", DependencyKind::Normal),
        vec!["extra".to_string()],
        "default-features = false drops the default pseudo-feature",
    );
}

#[test]
pub(super) fn transitive_enables_are_not_chased() {
    // A MEANINGFUL test of the rejected "expand through C's `[features]` graph" layer: the
    // metadata carries C's expansion evidence (a `resolve.nodes` entry for C where `full`
    // has resolved into `unstable`), yet the target declares only `full`. A `forbid C/unstable`
    // rule must stay clean and a `forbid C/full` rule must fire — proving the rule reads the
    // target's AUTHORED request, never C's expanded/resolved graph. If `findings` were changed
    // to fold in the transitive `unstable` (via C's graph or the resolve node), the first
    // assertion below would flip to a `C/unstable` finding and fail.
    let metadata = serde_json::json!({
        "packages": [{
            "name": "target",
            "dependencies": [
                {
                    "name": "C",
                    "source": "registry+https://x",
                    "kind": null,
                    "features": ["full"],
                    "uses_default_features": false
                },
            ],
        }],
        // The unified resolved graph: C's `full` has pulled in `unstable`. This is exactly the
        // contamination the declared-not-resolved model rejects; it must not reach the finding.
        "resolve": {
            "nodes": [
                { "id": "C 1.0.0", "features": ["full", "unstable"] },
            ]
        }
    });

    let forbid_unstable = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("C's unstable face is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &forbid_unstable, &mut v).unwrap();
    assert!(
        v.is_empty(),
        "the transitively-enabled `unstable` (present in resolve.nodes) is not chased: {v:?}",
    );

    // The authored `full` IS observed and flagged, so the metadata is genuinely exercised
    // (the clean result above is not merely an unread fixture).
    let forbid_full = CrateBoundary::crate_("target")
        .forbid_feature("C", "full")
        .because("even the authored `full` is governed");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &forbid_full, &mut v).unwrap();
    assert_eq!(v.len(), 1, "{v:?}");
    assert_eq!(v[0].finding, "C/full");
}

#[test]
pub(super) fn a_sibling_crates_enable_is_not_attributed_to_the_target() {
    // A MEANINGFUL test of the rejected `resolve.nodes` unification layer: the metadata has a
    // SIBLING workspace member enabling C's `unstable` AND a `resolve.nodes` entry for C whose
    // unified set contains `unstable`. The TARGET declares C with default-features = false and
    // no features. A `forbid C/unstable` rule on the target must stay clean — the declared set
    // is computed from the target's own edge, not the sibling's enable nor the unified node. If
    // `findings` read `resolve.nodes[C].features` (or scanned other packages' edges), this would
    // report `C/unstable` and fail.
    let metadata = serde_json::json!({
        "packages": [
            {
                "name": "target",
                "dependencies": [
                    {
                        "name": "C",
                        "source": "registry+https://x",
                        "kind": null,
                        "uses_default_features": false
                    },
                ],
            },
            {
                "name": "sibling",
                "dependencies": [
                    {
                        "name": "C",
                        "source": "registry+https://x",
                        "kind": null,
                        "features": ["unstable"],
                        "uses_default_features": false
                    },
                ],
            },
        ],
        // Feature unification folds the sibling's `unstable` into the one shared C node.
        "resolve": {
            "nodes": [
                { "id": "C 1.0.0", "features": ["unstable"] },
            ]
        }
    });

    let boundary = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("target must not author C's unstable");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert!(
        v.is_empty(),
        "a sibling's enable (and the unified resolve node) is not attributed to the target: {v:?}",
    );

    // Sanity that the harness IS reactive: the SIBLING, which authored `unstable`, is flagged
    // by the same rule — so the target's clean result reflects its own edge, not a dead fixture.
    let sibling_boundary = CrateBoundary::crate_("sibling")
        .forbid_feature("C", "unstable")
        .because("the sibling did author it");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &sibling_boundary, &mut v).unwrap();
    assert_eq!(v.len(), 1, "the sibling authored C/unstable: {v:?}");
    assert_eq!(v[0].finding, "C/unstable");
}

#[test]
pub(super) fn declared_set_unions_across_multiple_edges() {
    // WHEN C is declared in [dependencies] with features=["a"] (defaults on) AND under
    // [target.'cfg(windows)'.dependencies] with default-features=false, features=["unstable"]
    // — both Normal-kind edges — the declared set is the union { a, default, unstable }.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["a"],
                "uses_default_features": true
            },
            {
                "name": "C",
                "kind": null,
                "target": "cfg(windows)",
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "C", DependencyKind::Normal),
        vec![
            "a".to_string(),
            "default".to_string(),
            "unstable".to_string()
        ],
        "the set unions across every matching edge; default from the plain edge",
    );
    // A rule forbidding `unstable` therefore emits a violation.
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
pub(super) fn a_dependency_is_matched_by_package_name_not_local_alias() {
    // WHEN the target declares myc = { package = "real-c", features = ["unstable"] } — cargo
    // reports name="real-c", rename="myc" — a rule against the package name `real-c` sees
    // `unstable`, while one against the local alias `myc` matches nothing.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "real-c",
                "rename": "myc",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    assert_eq!(
        declared_features(&package, "real-c", DependencyKind::Normal),
        vec!["unstable".to_string()],
        "matched by resolved package name",
    );
    assert!(
        declared_features(&package, "myc", DependencyKind::Normal).is_empty(),
        "the local alias matches nothing",
    );
}

#[test]
pub(super) fn restrict_features_flags_a_feature_outside_the_allowlist() {
    // WHEN the target declares C's `unstable` and the boundary restricts C to ["stable"]
    // → a violation for C/unstable.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec!["stable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
pub(super) fn restrict_features_is_clean_when_every_declared_feature_is_allowed() {
    // WHEN the only declared feature is `stable`, with default-features = false, under a
    // restrict-to ["stable"] → clean.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["stable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec!["stable".to_string()],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an in-allowlist feature (with defaults off) is clean",
    );
}

#[test]
pub(super) fn empty_allowlist_forbids_declaring_any_feature_including_default() {
    // WHEN the target leaves defaults on (or declares any feature) and the boundary restricts
    // C's features to [] → every declared feature, `default` included, is a violation.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["extra"],
                "uses_default_features": true
            },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec![],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/default".to_string(), "C/extra".to_string()],
        "an empty allowlist forbids default and every explicit feature",
    );
}

#[test]
pub(super) fn restrict_features_on_a_crate_not_depended_on_is_clean() {
    // WHEN the boundary restricts the features of a crate the target does not depend on in
    // the selected table → clean (there is no declared set to constrain; never exit 2 here).
    let package = serde_json::json!({
        "dependencies": [
            { "name": "other", "kind": null, "uses_default_features": true },
        ]
    });
    let rule = Rule::RestrictFeaturesOf {
        crate_: "C".to_string(),
        allowed: vec![],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "a feature rule on an undepended crate is clean",
    );
}

#[test]
pub(super) fn forbid_feature_flags_a_declared_forbidden_feature() {
    // WHEN the target declares C's `unstable` and the boundary forbids C's `unstable`
    // → a violation for C/unstable.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/unstable".to_string()],
    );
}

#[test]
pub(super) fn forbidding_default_requires_default_features_off() {
    // WHEN the boundary forbids C's `default` and the target declares C WITHOUT
    // default-features = false → a violation for C/default.
    let package = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": true },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["default".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Normal),
        vec!["C/default".to_string()],
        "forbidding `default` ≡ requiring default-features = false",
    );
    // With default-features = false, the same rule is clean.
    let off = serde_json::json!({
        "dependencies": [
            { "name": "C", "kind": null, "uses_default_features": false },
        ]
    });
    assert!(
        rule.findings(&off, &[], DependencyKind::Normal).is_empty(),
        "with defaults off, forbidding `default` is satisfied",
    );
}

#[test]
pub(super) fn a_forbidden_feature_the_target_does_not_declare_is_clean() {
    // WHEN the boundary forbids C's `unstable`, and the target declares only C's `full`
    // (which would transitively enable `unstable`, not chased) → clean.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["full"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an undeclared forbidden feature is clean; transitive enables are not chased",
    );
}

#[test]
pub(super) fn empty_forbidden_set_is_a_no_op() {
    // WHEN a boundary forbids an empty set of features of C → clean regardless of what the
    // target declares.
    let package = feature_package();
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec![],
    };
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "an empty forbidden set is a vacuous no-op",
    );
}

#[test]
pub(super) fn feature_finding_carries_the_dependency_kind_suffix() {
    // A dev-table feature request carries the ` (dev)` suffix (the shared kind-qualify logic),
    // so C/unstable in [dependencies] and in [dev-dependencies] stay distinct findings.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": "dev",
                "features": ["unstable"],
                "uses_default_features": false
            },
        ]
    });
    let rule = Rule::ForbidFeaturesOf {
        crate_: "C".to_string(),
        forbidden: vec!["unstable".to_string()],
    };
    assert_eq!(
        rule.findings(&package, &[], DependencyKind::Dev),
        vec!["C/unstable (dev)".to_string()],
        "a dev-kind feature finding is `C/f (dev)`",
    );
    assert!(
        rule.findings(&package, &[], DependencyKind::Normal)
            .is_empty(),
        "the Normal surface does not observe a dev-table edge",
    );
}

#[test]
pub(super) fn two_forbidden_features_of_the_same_crate_stay_distinct() {
    // The one forbidden bug for the feature family: C/unstable and C/nightly, both forbidden,
    // must not collapse — baselining C/unstable must not mask C/nightly.
    let package = serde_json::json!({
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable", "nightly"],
                "uses_default_features": false
            },
        ]
    });
    let metadata = serde_json::json!({ "packages": [{
        "name": "target",
        "dependencies": package["dependencies"].clone(),
    }] });
    let boundary = CrateBoundary::crate_("target")
        .forbid_features_of("C", ["unstable", "nightly"])
        .because("no unstable/nightly features of C");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert_eq!(
        v.iter().map(|x| x.finding.as_str()).collect::<Vec<_>>(),
        vec!["C/nightly", "C/unstable"],
        "both features are distinct findings under the one rule",
    );

    // Baseline records ONLY C/unstable (a prior accepted state). Re-applying it must mark
    // C/unstable baselined while leaving C/nightly live — the finding, not the (target, rule)
    // pair, is the baseline key, so one feature's acceptance never masks another's.
    let accepted = v
        .iter()
        .find(|violation| violation.finding == "C/unstable")
        .expect("C/unstable present")
        .clone();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&Report::new(vec![accepted]));
    apply_baseline(&mut report, &baseline);

    let unstable = report
        .violations
        .iter()
        .find(|x| x.finding == "C/unstable")
        .expect("C/unstable present");
    let nightly = report
        .violations
        .iter()
        .find(|x| x.finding == "C/nightly")
        .expect("C/nightly present");
    assert!(unstable.baselined, "the baselined C/unstable is marked");
    assert!(
        !nightly.baselined,
        "C/nightly is NOT masked by the C/unstable baseline entry",
    );
    // The baseline entry for C/unstable is matched (not stale); C/nightly is not in it.
    assert!(
        baseline.stale(&report).is_empty(),
        "the C/unstable baseline entry matches a current violation, so it is not stale",
    );
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "the still-live C/nightly enforce violation fails the reaction",
    );
}

#[test]
pub(super) fn the_two_feature_rule_families_keep_identity_injective() {
    // A restrict and a forbid rule that both flag C/unstable on the same target stay distinct
    // triples, because their `rule` labels differ.
    assert_ne!(
        Rule::RestrictFeaturesOf {
            crate_: "C".to_string(),
            allowed: vec![],
        }
        .label(),
        Rule::ForbidFeaturesOf {
            crate_: "C".to_string(),
            forbidden: vec![],
        }
        .label(),
        "the two feature-rule labels must differ",
    );
    // And distinct from every crate rule label.
    let feature_labels = ["restrict features of", "forbid features of"];
    for other in [
        Rule::DenyExternalDependencies { allowed: vec![] }.label(),
        Rule::ForbidDependencyOn { crates: vec![] }.label(),
        Rule::RestrictDependenciesTo { allowed: vec![] }.label(),
        Rule::RestrictWorkspaceDependenciesTo { allowed: vec![] }.label(),
        Rule::RestrictDependencySourcesTo { allowed: vec![] }.label(),
    ] {
        assert!(
            !feature_labels.contains(&other),
            "feature labels must be distinct from crate-rule label {other}",
        );
    }

    // Concrete non-masking: a restrict rule (empty allowlist) and a forbid rule both flag the
    // SAME `C/unstable` on the SAME target. Because their `rule` labels differ, they are two
    // distinct `(target, rule, finding)` triples: baselining the restrict violation must NOT
    // mask the forbid violation.
    let metadata = serde_json::json!({ "packages": [{
        "name": "target",
        "dependencies": [
            {
                "name": "C",
                "source": "registry+https://x",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ],
    }] });
    let restrict = CrateBoundary::crate_("target")
        .restrict_features_of("C", Vec::<String>::new())
        .because("C's feature surface is closed");
    let forbid = CrateBoundary::crate_("target")
        .forbid_feature("C", "unstable")
        .because("C's unstable is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &restrict, &mut v).unwrap();
    check_crate_boundary(&metadata, &[], &forbid, &mut v).unwrap();
    // Same target + same finding, two different rule labels ⇒ two distinct violations.
    assert_eq!(v.len(), 2, "{v:?}");
    assert!(v.iter().all(|x| x.finding == "C/unstable"));
    let mut rules: Vec<&str> = v.iter().map(|x| x.rule.as_str()).collect();
    rules.sort_unstable();
    assert_eq!(rules, vec!["forbid features of", "restrict features of"]);

    // Baseline ONLY the restrict-rule C/unstable; the forbid-rule C/unstable must stay live.
    let accepted = v
        .iter()
        .find(|violation| violation.rule == "restrict features of")
        .expect("restrict violation present")
        .clone();
    let mut report = Report::new(v);
    let baseline = Baseline::of(&Report::new(vec![accepted]));
    apply_baseline(&mut report, &baseline);
    let restrict_v = report
        .violations
        .iter()
        .find(|x| x.rule == "restrict features of")
        .unwrap();
    let forbid_v = report
        .violations
        .iter()
        .find(|x| x.rule == "forbid features of")
        .unwrap();
    assert!(restrict_v.baselined, "the restrict C/unstable is baselined");
    assert!(
        !forbid_v.baselined,
        "the forbid C/unstable is a distinct triple, not masked by the restrict baseline",
    );
    assert_eq!(
        Outcome::Violations(report).exit_code(),
        1,
        "the still-live forbid C/unstable fails the reaction",
    );
}

#[test]
pub(super) fn feature_report_names_target_rule_finding_and_reason() {
    // WHEN a feature rule on C is violated by the declared `unstable` → the report names the
    // target, the feature rule, the finding C/unstable, and the reason; the reaction fails.
    let metadata = serde_json::json!({ "packages": [{
        "name": "app",
        "dependencies": [
            {
                "name": "C",
                "kind": null,
                "features": ["unstable"],
                "uses_default_features": false
            },
        ],
    }] });
    let boundary = CrateBoundary::crate_("app")
        .forbid_feature("C", "unstable")
        .because("C's unstable API face is off-limits");
    let mut v = Vec::new();
    check_crate_boundary(&metadata, &[], &boundary, &mut v).unwrap();
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].target(), "app");
    assert_eq!(v[0].rule, "forbid features of");
    assert_eq!(v[0].finding, "C/unstable");
    assert_eq!(v[0].reason, "C's unstable API face is off-limits");
    assert!(
        v[0].file.is_none(),
        "a feature violation is a manifest relation, not a source line",
    );
    assert_eq!(
        Outcome::Violations(Report::new(v)).exit_code(),
        1,
        "an enforce feature violation fails the reaction",
    );
}

#[test]
pub(super) fn a_feature_boundary_on_an_absent_target_is_a_constitution_error() {
    // Parity with the other crate rules: a feature boundary on a crate not in the workspace
    // is a constitution error (→ exit 2), never a silent pass. (`C` is never resolved, but the
    // TARGET must be a workspace member.)
    let metadata = serde_json::json!({ "packages": [{ "name": "present" }] });
    let boundary = CrateBoundary::crate_("absent")
        .restrict_features_of("C", ["stable"])
        .because("absent may use only C's stable face");
    let mut v = Vec::new();
    assert!(
        check_crate_boundary(&metadata, &[], &boundary, &mut v).is_err(),
        "an absent target crate must be a constitution error",
    );
}

#[test]
pub(super) fn feature_boundary_defaults_to_enforce_normal_kind() {
    // The builders finish through Enforce severity / Normal kind, like restrict_dependencies_to.
    let restrict = CrateBoundary::crate_("app")
        .restrict_features_of("C", ["stable"])
        .because("r");
    assert_eq!(restrict.severity(), Severity::Enforce);
    assert_eq!(restrict.dependency_kind(), DependencyKind::Normal);
    let forbid = CrateBoundary::crate_("app")
        .forbid_feature("C", "default")
        .because("f");
    assert_eq!(forbid.severity(), Severity::Enforce);
    assert_eq!(forbid.dependency_kind(), DependencyKind::Normal);
    // forbid_feature is the singular convenience over forbid_features_of.
    assert_eq!(
        forbid.rule(),
        &Rule::ForbidFeaturesOf {
            crate_: "C".to_string(),
            forbidden: vec!["default".to_string()],
        },
    );
}

#[test]
pub(super) fn feature_rules_project_to_text_and_json() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("app")
                .restrict_features_of("tokio", ["rt", "macros"])
                .because("app may use only tokio's rt and macros"),
        )
        .boundary(
            CrateBoundary::crate_("lib")
                .restrict_features_of::<&str, [&str; 0], &str>("tokio", [])
                .because("lib must declare no tokio feature at all"),
        )
        .boundary(
            CrateBoundary::crate_("worker")
                .forbid_features_of("tokio", ["unstable"])
                .dependency_kind(DependencyKind::Build)
                .because("no unstable tokio in the build script"),
        )
        .boundary(
            CrateBoundary::crate_("edge")
                .forbid_features_of::<&str, [&str; 0], &str>("serde", [])
                .because("vacuous"),
        );

    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict features of tokio to: rt, macros"),
        "{text}"
    );
    assert!(
        text.contains("restrict features of tokio to nothing"),
        "{text}"
    );
    assert!(
        text.contains("forbid features of tokio: unstable (build dependencies)"),
        "{text}"
    );
    assert!(text.contains("forbid no features of serde"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict features of");
    assert_eq!(doc["boundaries"][0]["crate"], "tokio");
    assert_eq!(doc["boundaries"][0]["only_features"][0], "rt");
    assert_eq!(doc["boundaries"][0]["only_features"][1], "macros");
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(
        doc["boundaries"][1]["only_features"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
    assert_eq!(doc["boundaries"][2]["rule"], "forbid features of");
    assert_eq!(doc["boundaries"][2]["crate"], "tokio");
    assert_eq!(doc["boundaries"][2]["forbidden_features"][0], "unstable");
    // Non-Normal kind is disclosed in the projection.
    assert_eq!(doc["boundaries"][2]["dependency_kind"], "build");
    // Distinct keys per polarity: restrict uses only_features, forbid uses forbidden_features.
    assert!(doc["boundaries"][0]["forbidden_features"].is_null());
    assert!(doc["boundaries"][2]["only_features"].is_null());
}

#[test]
pub(super) fn module_restrict_imports_to_projects_its_allowlist() {
    let constitution = Constitution::new("p")
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::kernel")
                .restrict_imports_to(["crate::types"])
                .because("the kernel may import only types"),
        )
        .boundary(
            ModuleBoundary::in_crate("app")
                .module("crate::leaf")
                .restrict_imports_to::<[&str; 0], &str>([])
                .because("the leaf may import only its own subtree"),
        );

    let text = constitution_text(&constitution);
    assert!(text.contains("restrict imports to: crate::types"), "{text}");
    assert!(text.contains("restrict imports to nothing"), "{text}");

    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["rule"], "restrict imports to");
    assert_eq!(doc["boundaries"][0]["kind"], "module");
    assert_eq!(doc["boundaries"][0]["target"], "crate::kernel");
    // The closed set uses `only` (the crate-level vocabulary), never `forbidden`.
    assert_eq!(doc["boundaries"][0]["only"][0], "crate::types");
    assert!(doc["boundaries"][0]["forbidden"].is_null());
    // The empty allowlist is still emitted, as `[]`.
    assert_eq!(doc["boundaries"][1]["only"].as_array().unwrap().len(), 0);
}

#[test]
pub(super) fn an_unreadable_utf8_governed_source_file_is_a_scan_error() {
    // Deterministic on every platform and euid (unlike the permission-based tests,
    // which skip under a privileged user): invalid UTF-8 makes `read_to_string` fail,
    // so a reachable governed-crate source file that cannot be read is a scan error
    // (exit 2), never a silent skip — the core-contract "cannot judge" rule.
    let ws = TempWorkspace::new("utf8");
    ws.write("lib.rs", "pub mod kernel;\n");
    // 0xFF / 0xFE are not valid UTF-8; read_to_string returns Err on all platforms.
    std::fs::write(ws.src().join("kernel.rs"), [0xFF, 0xFE, 0x00, 0x80]).expect("write kernel.rs");

    let metadata = ws.metadata("x");
    let boundary = ModuleBoundary::in_crate("x")
        .module("crate::kernel")
        .must_not_import("crate::forbidden")
        .because("kernel must not import forbidden");
    let mut violations = Vec::new();
    let result = check_module_boundary(&metadata, &boundary, &mut violations);
    assert!(
        result.is_err(),
        "an unreadable (invalid-UTF-8) governed source file must be a scan error"
    );
}

#[test]
pub(super) fn dependency_kind_appears_in_the_projection() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .deny_external_dependencies()
                .dependency_kind(DependencyKind::Dev)
                .because("a's dev dependencies stay light"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .deny_external_dependencies()
                .because("b stays light"),
        );
    let text = constitution_text(&constitution);
    assert!(text.contains("(dev dependencies)"), "{text}");
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["dependency_kind"], "dev");
    // A Normal-kind boundary omits the field entirely (the common projection).
    assert!(doc["boundaries"][1]["dependency_kind"].is_null(), "{doc}");
}

#[test]
pub(super) fn restrict_workspace_dependencies_to_projects_only_workspace() {
    let constitution = Constitution::new("p")
        .boundary(
            CrateBoundary::crate_("a")
                .restrict_workspace_dependencies_to(["b"])
                .because("a may depend on only workspace member b"),
        )
        .boundary(
            CrateBoundary::crate_("c")
                .forbid_all_workspace_dependencies()
                .because("c must not depend on any workspace member"),
        );
    let text = constitution_text(&constitution);
    assert!(
        text.contains("restrict workspace dependencies to: b"),
        "{text}"
    );
    assert!(text.contains("forbid all workspace dependencies"), "{text}");
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(
        doc["boundaries"][0]["rule"],
        "restrict workspace dependencies to"
    );
    // The distinct key (`only_workspace`, not `only`) says which dependency surface
    // is governed — the self-describing distinction with no coverage before now.
    assert_eq!(doc["boundaries"][0]["only_workspace"][0], "b");
    assert!(doc["boundaries"][0]["only"].is_null());
    // The empty allowlist (forbid-all) still emits `only_workspace: []`.
    assert_eq!(
        doc["boundaries"][1]["only_workspace"]
            .as_array()
            .unwrap()
            .len(),
        0
    );
}

/// A boundary's `.with_anchor(...)` rides through the reaction onto the produced violation and
/// into its JSON — the durable governance pointer reaches the CI-consumable surface.
#[test]
pub(super) fn an_anchored_boundary_stamps_its_violations_with_the_anchor() {
    let (result, violations) = run_module_check(
        "anchored",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("the kernel must not import a secret")
            .with_anchor("ADR-014"),
    );
    result.expect("a valid module boundary");
    assert!(!violations.is_empty(), "the forbidden import must react");
    assert_eq!(violations[0].anchor.as_deref(), Some("ADR-014"));
    assert_eq!(violations[0].to_json()["anchor"], "ADR-014");
}

/// The mirror: a boundary that declares no anchor produces `None` on its violations — the anchor
/// is opt-in and never fabricated.
#[test]
pub(super) fn an_anchorless_boundary_leaves_its_violations_unanchored() {
    let (result, violations) = run_module_check(
        "unanchored",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("the kernel must not import a secret"),
    );
    result.expect("a valid module boundary");
    assert!(!violations.is_empty());
    assert_eq!(violations[0].anchor, None);
}

/// The reaction stamps a module violation with the rule's repair-direction polarity: a deny rule
/// (`must_not_import`) → `DenyBreach`, an allowlist rule (`restrict_imports_to`) → `AllowlistGap`.
#[test]
pub(super) fn a_module_violation_carries_its_rule_repair_polarity() {
    let (_r, deny) = run_module_check(
        "polarity-deny",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::secret::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .must_not_import("crate::secret")
            .because("deny"),
    );
    assert_eq!(deny[0].polarity, Some(Polarity::DenyBreach));

    let (_r, allow) = run_module_check(
        "polarity-allow",
        &[
            ("lib.rs", "pub mod kernel;\n"),
            ("kernel.rs", "use crate::infra::Thing;\n"),
        ],
        ModuleBoundary::in_crate("x")
            .module("crate::kernel")
            .restrict_imports_to(["crate::types"])
            .because("allow"),
    );
    assert_eq!(allow[0].polarity, Some(Polarity::AllowlistGap));
}

/// The rule → polarity classification, incl. the deliberate call that `deny_external_dependencies`
/// is `AllowlistGap` (its `allow_external` is an in-boundary declaration path — by repair direction,
/// not rule name), while `forbid_dependency_on` names a specific forbidden crate → `DenyBreach`.
#[test]
pub(super) fn crate_rule_polarity_classifies_by_repair_direction() {
    let deny_external = CrateBoundary::crate_("x")
        .deny_external_dependencies()
        .because("r");
    assert_eq!(deny_external.rule().polarity(), Polarity::AllowlistGap);

    let forbid = CrateBoundary::crate_("x")
        .forbid_dependency_on(["openssl"])
        .because("r");
    assert_eq!(forbid.rule().polarity(), Polarity::DenyBreach);

    let restrict = CrateBoundary::crate_("x")
        .restrict_dependencies_to(["serde"])
        .because("r");
    assert_eq!(restrict.rule().polarity(), Polarity::AllowlistGap);
}

/// The constitution projection surfaces a boundary's anchor **only when set**, so an anchor-less
/// boundary keeps byte-identical JSON (the Some-only discipline that protects the self-law
/// projection's staleness byte-check).
#[test]
pub(super) fn constitution_json_emits_anchor_only_when_set() {
    let constitution = Constitution::new("anchors")
        .boundary(
            CrateBoundary::crate_("a")
                .forbid_all_workspace_dependencies()
                .because("anchored")
                .with_anchor("ADR-014"),
        )
        .boundary(
            CrateBoundary::crate_("b")
                .forbid_all_workspace_dependencies()
                .because("unanchored"),
        );
    let doc: serde_json::Value = serde_json::from_str(&constitution_json(&constitution)).unwrap();
    assert_eq!(doc["boundaries"][0]["anchor"], "ADR-014");
    assert!(
        doc["boundaries"][1].get("anchor").is_none(),
        "an anchor-less boundary must emit no `anchor` key"
    );
}
