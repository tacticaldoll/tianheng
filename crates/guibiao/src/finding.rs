//! Dimension-owned observed facts and their shared reaction projection.

use xuanji::{Finding, StructuredFactIdentity};

use crate::{DependencyKind, SourceKind};

pub(crate) enum CrateFact {
    Dependency {
        package: String,
        kind: DependencyKind,
    },
    Feature {
        package: String,
        feature: String,
        kind: DependencyKind,
    },
    Source {
        package: String,
        source: SourceKind,
        kind: DependencyKind,
    },
}

impl CrateFact {
    pub(crate) fn dependency(package: String, kind: DependencyKind) -> Self {
        Self::Dependency { package, kind }
    }

    pub(crate) fn feature(package: String, feature: String, kind: DependencyKind) -> Self {
        Self::Feature {
            package,
            feature,
            kind,
        }
    }

    pub(crate) fn source(package: String, source: SourceKind, kind: DependencyKind) -> Self {
        Self::Source {
            package,
            source,
            kind,
        }
    }

    pub(crate) fn into_finding(self) -> Finding {
        match self {
            CrateFact::Dependency { package, kind } => Finding::new(
                format!("{package}{}", kind.finding_suffix()),
                fact(
                    "dependency",
                    "dependency-edge",
                    [("kind", kind.key_label()), ("package", package.as_str())],
                ),
            ),
            CrateFact::Feature {
                package,
                feature,
                kind,
            } => Finding::new(
                format!("{package}/{feature}{}", kind.finding_suffix()),
                fact(
                    "dependency-feature",
                    "declared-feature",
                    [
                        ("feature", feature.as_str()),
                        ("kind", kind.key_label()),
                        ("package", package.as_str()),
                    ],
                ),
            ),
            CrateFact::Source {
                package,
                source,
                kind,
            } => Finding::new(
                format!("{package}{}", kind.finding_suffix()),
                fact(
                    "dependency-source",
                    "declared-source",
                    [
                        ("kind", kind.key_label()),
                        ("package", package.as_str()),
                        ("source", source.label()),
                    ],
                ),
            ),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ModuleFact {
    ImportedPath { path: String, importer: String },
    ImporterModule(String),
    ExternalImporter(String),
    InlinePath { path: String, module: String },
    InlineGlob { path: String, module: String },
}

impl ModuleFact {
    /// `governing_package` is the crate the violated boundary was declared against
    /// (`boundary.crate_package`) — an identity-bearing field distinct from `CrateFact`'s
    /// `"package"` above, which names the *observed dependency*, not the declaring crate. Without
    /// it, two crates declaring the identical boundary against the identical module path produce
    /// identical identities and silently collapse (see `structured-violation-identity` spec).
    /// `unit` is the **compilation unit** the observation came from: the root's source path relative to
    /// the package's manifest directory (`src/lib.rs`, `src/main.rs`, `tools/x.rs`). A package builds more
    /// than one root — a library beside a `bin` — and every root denotes the module path `crate` and
    /// shares the package name, so without this role the same violation in two roots carries ONE identity
    /// and a baseline accepting it in one silently masks it appearing in the other.
    ///
    /// It is not the target's NAME: a package may build a library and a `bin` of the same name (this
    /// repository does), so a name is not unique within a package. It is not an index or metadata order
    /// either — `semantic-signature-coupling`'s prohibition on positional identity applies here too. The
    /// root path is declaration-derived, unique per unit, and moves with neither the checkout nor the
    /// member set. A root **outside** the manifest directory has no checkout-independent label at all
    /// and is a constitution error rather than a fallback — see [`xingbiao::compilation_unit_label`],
    /// whose `None` has exactly that one cause.
    ///
    /// This is deliberately NOT the rule 漏刻 applies to a file reached through an absolute `#[path]`
    /// literal, and the difference is the whole reason: that literal is **committed text**, identical in
    /// every checkout, so keeping it verbatim is what makes it stable. A root path outside the manifest
    /// directory is the clone's own location, so keeping it verbatim is what makes it unstable. Same
    /// shape, opposite consequence.
    pub(crate) fn into_finding(self, governing_package: &str, unit: &str) -> Finding {
        match self {
            ModuleFact::ImportedPath { path, importer } => {
                let key = fact(
                    "imported-path",
                    "module-path",
                    [
                        ("governing_package", governing_package),
                        ("unit", unit),
                        ("importer", importer.as_str()),
                        ("path", path.as_str()),
                    ],
                );
                Finding::new(path, key)
            }
            ModuleFact::ImporterModule(module) => {
                let key = fact(
                    "importer-module",
                    "module-path",
                    [
                        ("governing_package", governing_package),
                        ("unit", unit),
                        ("module", module.as_str()),
                    ],
                );
                Finding::new(module, key)
            }
            ModuleFact::ExternalImporter(module) => {
                let key = fact(
                    "external-importer",
                    "module-path",
                    [
                        ("governing_package", governing_package),
                        ("unit", unit),
                        ("module", module.as_str()),
                    ],
                );
                Finding::new(module, key)
            }
            ModuleFact::InlinePath { path, module } => Finding::new(
                format!("{path} in {module}"),
                fact(
                    "inline-path",
                    "path-in-module",
                    [
                        ("governing_package", governing_package),
                        ("unit", unit),
                        ("module", module.as_str()),
                        ("path", path.as_str()),
                    ],
                ),
            ),
            ModuleFact::InlineGlob { path, module } => Finding::new(
                format!("glob {path} in {module}"),
                fact(
                    "inline-glob",
                    "path-in-module",
                    [
                        ("governing_package", governing_package),
                        ("unit", unit),
                        ("module", module.as_str()),
                        ("path", path.as_str()),
                    ],
                ),
            ),
        }
    }
}

fn fact<const N: usize>(
    family: &str,
    shape: &str,
    fields: [(&str, &str); N],
) -> StructuredFactIdentity {
    StructuredFactIdentity::of(format!("tianheng.fact/guibiao/{family}"), shape, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_dependency_kind_is_cataloged(kind: DependencyKind) {
        match kind {
            DependencyKind::Normal | DependencyKind::Dev | DependencyKind::Build => {}
        }
    }

    fn assert_source_kind_is_cataloged(source: SourceKind) {
        match source {
            SourceKind::Registry | SourceKind::Git | SourceKind::Path => {}
        }
    }

    fn assert_crate_fact_is_cataloged(fact: &CrateFact) {
        match fact {
            CrateFact::Dependency { package: _, kind }
            | CrateFact::Feature {
                package: _,
                feature: _,
                kind,
            } => assert_dependency_kind_is_cataloged(*kind),
            CrateFact::Source {
                package: _,
                source,
                kind,
            } => {
                assert_dependency_kind_is_cataloged(*kind);
                assert_source_kind_is_cataloged(*source);
            }
        }
    }

    fn assert_module_fact_is_cataloged(fact: &ModuleFact) {
        match fact {
            ModuleFact::ImportedPath { .. }
            | ModuleFact::ImporterModule(_)
            | ModuleFact::ExternalImporter(_)
            | ModuleFact::InlinePath { path: _, module: _ }
            | ModuleFact::InlineGlob { path: _, module: _ } => {}
        }
    }

    fn assert_key(fact: impl IntoFinding, family: &str, shape: &str, fields: &[(&str, &str)]) {
        let finding = fact.into_finding();
        assert_eq!(
            finding.key().fact_type(),
            format!("tianheng.fact/guibiao/{family}")
        );
        assert_eq!(finding.key().shape(), shape);
        assert_eq!(finding.key().fields().collect::<Vec<_>>(), fields);
    }

    trait IntoFinding {
        fn into_finding(self) -> Finding;
    }

    impl IntoFinding for CrateFact {
        fn into_finding(self) -> Finding {
            CrateFact::into_finding(self)
        }
    }

    impl IntoFinding for ModuleFact {
        fn into_finding(self) -> Finding {
            ModuleFact::into_finding(self, "app", "src/lib.rs")
        }
    }

    #[test]
    fn published_crate_fact_identity_schema_is_exact_and_exhaustive() {
        let dependency_cases = [
            (DependencyKind::Normal, "normal"),
            (DependencyKind::Dev, "dev"),
            (DependencyKind::Build, "build"),
        ];
        for (kind, label) in dependency_cases {
            let fact = CrateFact::dependency("serde".to_string(), kind);
            assert_crate_fact_is_cataloged(&fact);
            assert_key(
                fact,
                "dependency",
                "dependency-edge",
                &[("kind", label), ("package", "serde")],
            );

            let fact = CrateFact::feature("serde".to_string(), "derive".to_string(), kind);
            assert_crate_fact_is_cataloged(&fact);
            assert_key(
                fact,
                "dependency-feature",
                "declared-feature",
                &[("feature", "derive"), ("kind", label), ("package", "serde")],
            );
        }

        let source_cases = [
            (SourceKind::Registry, "registry"),
            (SourceKind::Git, "git"),
            (SourceKind::Path, "path"),
        ];
        for (source, label) in source_cases {
            let fact = CrateFact::source("serde".to_string(), source, DependencyKind::Normal);
            assert_crate_fact_is_cataloged(&fact);
            assert_key(
                fact,
                "dependency-source",
                "declared-source",
                &[("kind", "normal"), ("package", "serde"), ("source", label)],
            );
        }
    }

    #[test]
    fn published_module_fact_identity_schema_is_exact_and_exhaustive() {
        struct ModuleKeyCase {
            fact: ModuleFact,
            fields: Vec<(&'static str, &'static str)>,
            family: &'static str,
            shape: &'static str,
        }

        let cases = vec![
            ModuleKeyCase {
                fact: ModuleFact::ImportedPath {
                    path: "crate::ports".to_string(),
                    importer: "crate::core".to_string(),
                },
                fields: vec![
                    ("governing_package", "app"),
                    ("importer", "crate::core"),
                    ("path", "crate::ports"),
                    ("unit", "src/lib.rs"),
                ],
                family: "imported-path",
                shape: "module-path",
            },
            ModuleKeyCase {
                fact: ModuleFact::ImporterModule("crate::api".to_string()),
                fields: vec![
                    ("governing_package", "app"),
                    ("module", "crate::api"),
                    ("unit", "src/lib.rs"),
                ],
                family: "importer-module",
                shape: "module-path",
            },
            ModuleKeyCase {
                fact: ModuleFact::ExternalImporter("crate::ffi".to_string()),
                fields: vec![
                    ("governing_package", "app"),
                    ("module", "crate::ffi"),
                    ("unit", "src/lib.rs"),
                ],
                family: "external-importer",
                shape: "module-path",
            },
            ModuleKeyCase {
                fact: ModuleFact::InlinePath {
                    path: "std::time::SystemTime::now".to_string(),
                    module: "crate::kernel".to_string(),
                },
                fields: vec![
                    ("governing_package", "app"),
                    ("module", "crate::kernel"),
                    ("path", "std::time::SystemTime::now"),
                    ("unit", "src/lib.rs"),
                ],
                family: "inline-path",
                shape: "path-in-module",
            },
            ModuleKeyCase {
                fact: ModuleFact::InlineGlob {
                    path: "std::time::*".to_string(),
                    module: "crate::kernel".to_string(),
                },
                fields: vec![
                    ("governing_package", "app"),
                    ("module", "crate::kernel"),
                    ("path", "std::time::*"),
                    ("unit", "src/lib.rs"),
                ],
                family: "inline-glob",
                shape: "path-in-module",
            },
        ];
        for case in cases {
            assert_module_fact_is_cataloged(&case.fact);
            // Every module fact is observed from SOURCE, so two coordinates of the observation's
            // location can always vary for it and must always be present: which declaration governs it
            // (a second crate can declare the identical boundary against the identical module path) and
            // which compilation unit it came from (a package builds more than one crate root, and every
            // root denotes the module path `crate`). This is the enforcement point
            // `structured-violation-identity` names: a family added later that omits either fails here,
            // rather than surviving until two observations are found to collide.
            //
            // The remaining coordinates are per-family and are asserted by the exact field lists above:
            // the module for the inline forms, the importing module for an outbound finding, and the
            // observed path or module as the thing itself. A crate fact carries neither of these two,
            // and that omission is recorded rather than silent: its target IS the package, and it
            // observes the manifest rather than any compilation unit.
            for required in ["governing_package", "unit"] {
                assert!(
                    case.fields.iter().any(|(name, _)| *name == required),
                    "{}: a source-observed fact must carry the '{required}' coordinate — see \
                     `structured-violation-identity`'s coordinate derivation",
                    case.family
                );
            }
            assert_key(case.fact, case.family, case.shape, &case.fields);
        }
    }

    #[test]
    fn distinct_governing_packages_produce_distinct_module_fact_identity() {
        let alpha = ModuleFact::ImporterModule("crate::app".to_string())
            .into_finding("alpha", "src/lib.rs")
            .key()
            .clone();
        let beta = ModuleFact::ImporterModule("crate::app".to_string())
            .into_finding("beta", "src/lib.rs")
            .key()
            .clone();
        assert_ne!(
            alpha, beta,
            "two crates declaring the identical module path must not share one fact identity"
        );
    }

    #[test]
    fn identity_bearing_values_and_fact_shapes_stay_distinct() {
        let normal = CrateFact::dependency("serde".to_string(), DependencyKind::Normal)
            .into_finding()
            .key()
            .clone();
        let dev = CrateFact::dependency("serde".to_string(), DependencyKind::Dev)
            .into_finding()
            .key()
            .clone();
        let feature = CrateFact::feature(
            "serde".to_string(),
            "derive".to_string(),
            DependencyKind::Normal,
        )
        .into_finding()
        .key()
        .clone();
        assert_ne!(normal, dev);
        assert_ne!(normal, feature);

        let import = ModuleFact::ImportedPath {
            path: "crate::ports".to_string(),
            importer: "crate::core".to_string(),
        }
        .into_finding("app", "src/lib.rs")
        .key()
        .clone();
        let importer = ModuleFact::ImporterModule("crate::ports".to_string())
            .into_finding("app", "src/lib.rs")
            .key()
            .clone();
        assert_ne!(import, importer);
    }

    #[test]
    fn unrelated_construction_order_does_not_change_fact_identity() {
        let before = ModuleFact::ImportedPath {
            path: "crate::ports".to_string(),
            importer: "crate::core".to_string(),
        }
        .into_finding("app", "src/lib.rs")
        .key()
        .clone();
        let _unrelated = ModuleFact::InlinePath {
            path: "std::time::SystemTime::now".to_string(),
            module: "crate::adapter".to_string(),
        }
        .into_finding("app", "src/lib.rs");
        let after = ModuleFact::ImportedPath {
            path: "crate::ports".to_string(),
            importer: "crate::core".to_string(),
        }
        .into_finding("app", "src/lib.rs")
        .key()
        .clone();
        assert_eq!(before, after);
    }
}
