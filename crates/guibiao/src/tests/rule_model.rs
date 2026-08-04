use super::helpers::*;
// White-box unit tests for the crate-private machinery — the baseline, the JSON
// and text projections, and the source scanner. Black-box behavior (running
// `check` against fixture workspaces) lives in this crate's `tests/` directory.

#[test]
pub(super) fn every_static_rule_has_an_exact_semantic_key() {
    let crate_rules = vec![
        (
            Rule::DenyExternalDependencies {
                allowed: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/deny-external-dependencies",
            vec![("allowed", "[\"serde\"]")],
        ),
        (
            Rule::ForbidDependencyOn {
                crates: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/forbid-dependency-on",
            vec![("crates", "[\"serde\"]")],
        ),
        (
            Rule::RestrictDependenciesTo {
                allowed: vec!["serde".to_string()],
            },
            "tianheng.rule/guibiao/restrict-dependencies-to",
            vec![("allowed", "[\"serde\"]")],
        ),
        (
            Rule::RestrictWorkspaceDependenciesTo {
                allowed: vec!["domain".to_string()],
            },
            "tianheng.rule/guibiao/restrict-workspace-dependencies-to",
            vec![("allowed", "[\"domain\"]")],
        ),
        (
            Rule::RestrictDependencySourcesTo {
                allowed: vec![SourceKind::Registry],
            },
            "tianheng.rule/guibiao/restrict-dependency-sources-to",
            vec![("allowed", "[\"registry\"]")],
        ),
        (
            Rule::RestrictFeaturesOf {
                crate_: "serde".to_string(),
                allowed: vec!["derive".to_string()],
            },
            "tianheng.rule/guibiao/restrict-features-of",
            vec![("allowed", "[\"derive\"]"), ("crate", "serde")],
        ),
        (
            Rule::ForbidFeaturesOf {
                crate_: "serde".to_string(),
                forbidden: vec!["unstable".to_string()],
            },
            "tianheng.rule/guibiao/forbid-features-of",
            vec![("crate", "serde"), ("forbidden", "[\"unstable\"]")],
        ),
    ];
    for (rule, expected, fields) in crate_rules {
        assert_eq!(rule.key().rule_type(), expected);
        assert_eq!(rule.key().fields().collect::<Vec<_>>(), fields);
    }

    let module_rules = vec![
        (
            ModuleRule::MustNotImport {
                module: "crate::adapter".to_string(),
            },
            "tianheng.rule/guibiao/must-not-import",
            vec![("module", "crate::adapter")],
        ),
        (
            ModuleRule::RestrictImportsTo {
                allowed: vec!["crate::types".to_string()],
            },
            "tianheng.rule/guibiao/restrict-imports-to",
            vec![("allowed", "[\"crate::types\"]")],
        ),
        (
            ModuleRule::MustNotBeImportedBy {
                importer: "crate::http".to_string(),
            },
            "tianheng.rule/guibiao/must-not-be-imported-by",
            vec![("importer", "crate::http")],
        ),
        (
            ModuleRule::MustOnlyBeImportedBy {
                allowed: vec!["crate::facade".to_string()],
            },
            "tianheng.rule/guibiao/must-only-be-imported-by",
            vec![("allowed", "[\"crate::facade\"]")],
        ),
        (
            ModuleRule::ConfineExternalCrate {
                crate_name: "libc".to_string(),
            },
            "tianheng.rule/guibiao/confine-external-crate",
            vec![("crate", "libc")],
        ),
        (
            ModuleRule::ConfineInlineSymbolPath {
                prefix: "std::time".to_string(),
                ending_with: Some(vec!["now".to_string()]),
                strict: false,
                strict_external: false,
            },
            "tianheng.rule/guibiao/confine-inline-symbol-path",
            vec![
                ("ending_with", "[\"now\"]"),
                ("prefix", "std::time"),
                ("strict", "false"),
            ],
        ),
    ];
    for (rule, expected, fields) in module_rules {
        assert_eq!(rule.key().rule_type(), expected);
        assert_eq!(rule.key().fields().collect::<Vec<_>>(), fields);
    }
}

#[test]
pub(super) fn rule_set_order_is_canonical_and_presentation_is_not_identity() {
    let left = Rule::ForbidDependencyOn {
        crates: vec!["serde".to_string(), "tokio".to_string()],
    };
    let right = Rule::ForbidDependencyOn {
        crates: vec!["tokio".to_string(), "serde".to_string()],
    };
    assert_eq!(left.key(), right.key());
    let changed_law = Rule::ForbidDependencyOn {
        crates: vec!["serde".to_string(), "tracing".to_string()],
    };
    assert_ne!(left.key(), changed_law.key());

    let default = ModuleRule::ConfineInlineSymbolPath {
        prefix: "std::time".to_string(),
        ending_with: Some(vec!["now".to_string()]),
        strict: false,
        strict_external: false,
    };
    let strict_external = ModuleRule::ConfineInlineSymbolPath {
        prefix: "std::time".to_string(),
        ending_with: Some(vec!["now".to_string()]),
        strict: false,
        strict_external: true,
    };
    assert_ne!(default.text(), strict_external.text());
    assert_eq!(default.key(), strict_external.key());

    let raw = ModuleRule::MustNotImport {
        module: "crate::r#type".to_string(),
    };
    let plain = ModuleRule::MustNotImport {
        module: "crate::type".to_string(),
    };
    assert_eq!(raw.key(), plain.key());
}

#[test]
pub(super) fn dependency_fact_identity_survives_reorder_and_unrelated_insertion() {
    fn identities(package: &serde_json::Value) -> Vec<StructuredFactIdentity> {
        Rule::RestrictDependencySourcesTo {
            allowed: vec![SourceKind::Registry],
        }
        .facts(package, &[], DependencyKind::Normal)
        .into_iter()
        .map(|fact| fact.into_finding().key().clone())
        .collect()
    }

    let before = serde_json::json!({
        "dependencies": [
            { "name": "blocked", "source": "git+https://example.invalid/blocked", "kind": null }
        ]
    });
    let after = serde_json::json!({
        "dependencies": [
            { "name": "allowed", "source": "registry+https://example.invalid/index", "kind": null },
            { "name": "blocked", "source": "git+https://example.invalid/blocked", "kind": null }
        ]
    });
    assert_eq!(identities(&before), identities(&after));

    let distinct_sources = serde_json::json!({
        "dependencies": [
            { "name": "same", "source": null, "kind": null },
            { "name": "same", "source": "git+https://example.invalid/same", "kind": null }
        ]
    });
    let facts = Rule::RestrictDependencySourcesTo { allowed: vec![] }
        .facts(&distinct_sources, &[], DependencyKind::Normal)
        .into_iter()
        .map(|fact| fact.into_finding().key().clone())
        .collect::<Vec<_>>();
    assert_eq!(facts.len(), 2);
    assert_ne!(facts[0], facts[1]);
}
