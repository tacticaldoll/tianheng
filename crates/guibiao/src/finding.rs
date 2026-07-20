//! Dimension-owned observed facts and their shared reaction projection.

use xuanji::{Finding, FindingKey};

use crate::DependencyKind;

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

    pub(crate) fn into_finding(self) -> Finding {
        match self {
            CrateFact::Dependency { package, kind } => Finding::new(
                format!("{package}{}", kind.finding_suffix()),
                key(
                    "dependency",
                    [("kind", kind.key_label()), ("package", package.as_str())],
                ),
            ),
            CrateFact::Feature {
                package,
                feature,
                kind,
            } => Finding::new(
                format!("{package}/{feature}{}", kind.finding_suffix()),
                key(
                    "dependency_feature",
                    [
                        ("feature", feature.as_str()),
                        ("kind", kind.key_label()),
                        ("package", package.as_str()),
                    ],
                ),
            ),
        }
    }
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum ModuleFact {
    ImportedPath(String),
    ImporterModule(String),
    ExternalImporter(String),
    InlinePath { path: String, module: String },
    InlineGlob { path: String, module: String },
}

impl ModuleFact {
    pub(crate) fn into_finding(self) -> Finding {
        match self {
            ModuleFact::ImportedPath(path) => {
                let key = key("imported_path", [("path", path.as_str())]);
                Finding::new(path, key)
            }
            ModuleFact::ImporterModule(module) => {
                let key = key("importer_module", [("module", module.as_str())]);
                Finding::new(module, key)
            }
            ModuleFact::ExternalImporter(module) => {
                let key = key("external_importer", [("module", module.as_str())]);
                Finding::new(module, key)
            }
            ModuleFact::InlinePath { path, module } => Finding::new(
                format!("{path} in {module}"),
                key(
                    "inline_path",
                    [("module", module.as_str()), ("path", path.as_str())],
                ),
            ),
            ModuleFact::InlineGlob { path, module } => Finding::new(
                format!("glob {path} in {module}"),
                key(
                    "inline_glob",
                    [("module", module.as_str()), ("path", path.as_str())],
                ),
            ),
        }
    }
}

fn key<const N: usize>(code: &str, fields: [(&str, &str); N]) -> FindingKey {
    FindingKey::of("guibiao", code, fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    type KeyCase<F> = (F, &'static str, Vec<(&'static str, &'static str)>);

    fn assert_dependency_kind_is_cataloged(kind: DependencyKind) {
        match kind {
            DependencyKind::Normal | DependencyKind::Dev | DependencyKind::Build => {}
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
        }
    }

    fn assert_module_fact_is_cataloged(fact: &ModuleFact) {
        match fact {
            ModuleFact::ImportedPath(_)
            | ModuleFact::ImporterModule(_)
            | ModuleFact::ExternalImporter(_)
            | ModuleFact::InlinePath { path: _, module: _ }
            | ModuleFact::InlineGlob { path: _, module: _ } => {}
        }
    }

    fn assert_key(fact: impl IntoFinding, code: &str, fields: &[(&str, &str)]) {
        let finding = fact.into_finding();
        assert_eq!(finding.key().namespace(), "guibiao");
        assert_eq!(finding.key().code(), code);
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
            ModuleFact::into_finding(self)
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
            assert_key(fact, "dependency", &[("kind", label), ("package", "serde")]);

            let fact = CrateFact::feature("serde".to_string(), "derive".to_string(), kind);
            assert_crate_fact_is_cataloged(&fact);
            assert_key(
                fact,
                "dependency_feature",
                &[("feature", "derive"), ("kind", label), ("package", "serde")],
            );
        }
    }

    #[test]
    fn published_module_fact_identity_schema_is_exact_and_exhaustive() {
        let cases: Vec<KeyCase<ModuleFact>> = vec![
            (
                ModuleFact::ImportedPath("crate::ports".to_string()),
                "imported_path",
                vec![("path", "crate::ports")],
            ),
            (
                ModuleFact::ImporterModule("crate::api".to_string()),
                "importer_module",
                vec![("module", "crate::api")],
            ),
            (
                ModuleFact::ExternalImporter("crate::ffi".to_string()),
                "external_importer",
                vec![("module", "crate::ffi")],
            ),
            (
                ModuleFact::InlinePath {
                    path: "std::time::SystemTime::now".to_string(),
                    module: "crate::kernel".to_string(),
                },
                "inline_path",
                vec![
                    ("module", "crate::kernel"),
                    ("path", "std::time::SystemTime::now"),
                ],
            ),
            (
                ModuleFact::InlineGlob {
                    path: "std::time::*".to_string(),
                    module: "crate::kernel".to_string(),
                },
                "inline_glob",
                vec![("module", "crate::kernel"), ("path", "std::time::*")],
            ),
        ];
        for (fact, code, fields) in cases {
            assert_module_fact_is_cataloged(&fact);
            assert_key(fact, code, &fields);
        }
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

        let import = ModuleFact::ImportedPath("crate::ports".to_string())
            .into_finding()
            .key()
            .clone();
        let importer = ModuleFact::ImporterModule("crate::ports".to_string())
            .into_finding()
            .key()
            .clone();
        assert_ne!(import, importer);
    }
}
