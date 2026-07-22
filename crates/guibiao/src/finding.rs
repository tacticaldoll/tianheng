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
                let key = fact("imported-path", "module-path", [("path", path.as_str())]);
                Finding::new(path, key)
            }
            ModuleFact::ImporterModule(module) => {
                let key = fact(
                    "importer-module",
                    "module-path",
                    [("module", module.as_str())],
                );
                Finding::new(module, key)
            }
            ModuleFact::ExternalImporter(module) => {
                let key = fact(
                    "external-importer",
                    "module-path",
                    [("module", module.as_str())],
                );
                Finding::new(module, key)
            }
            ModuleFact::InlinePath { path, module } => Finding::new(
                format!("{path} in {module}"),
                fact(
                    "inline-path",
                    "path-in-module",
                    [("module", module.as_str()), ("path", path.as_str())],
                ),
            ),
            ModuleFact::InlineGlob { path, module } => Finding::new(
                format!("glob {path} in {module}"),
                fact(
                    "inline-glob",
                    "path-in-module",
                    [("module", module.as_str()), ("path", path.as_str())],
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

    type KeyCase<F> = (F, Vec<(&'static str, &'static str)>);

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
            ModuleFact::ImportedPath(_)
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
        let cases: Vec<KeyCase<ModuleFact>> = vec![
            (
                ModuleFact::ImportedPath("crate::ports".to_string()),
                vec![("path", "crate::ports")],
            ),
            (
                ModuleFact::ImporterModule("crate::api".to_string()),
                vec![("module", "crate::api")],
            ),
            (
                ModuleFact::ExternalImporter("crate::ffi".to_string()),
                vec![("module", "crate::ffi")],
            ),
            (
                ModuleFact::InlinePath {
                    path: "std::time::SystemTime::now".to_string(),
                    module: "crate::kernel".to_string(),
                },
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
                vec![("module", "crate::kernel"), ("path", "std::time::*")],
            ),
        ];
        let shapes = [
            ("imported-path", "module-path"),
            ("importer-module", "module-path"),
            ("external-importer", "module-path"),
            ("inline-path", "path-in-module"),
            ("inline-glob", "path-in-module"),
        ];
        for ((fact, fields), (family, shape)) in cases.into_iter().zip(shapes) {
            assert_module_fact_is_cataloged(&fact);
            assert_key(fact, family, shape, &fields);
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

    #[test]
    fn unrelated_construction_order_does_not_change_fact_identity() {
        let before = ModuleFact::ImportedPath("crate::ports".to_string())
            .into_finding()
            .key()
            .clone();
        let _unrelated = ModuleFact::InlinePath {
            path: "std::time::SystemTime::now".to_string(),
            module: "crate::adapter".to_string(),
        }
        .into_finding();
        let after = ModuleFact::ImportedPath("crate::ports".to_string())
            .into_finding()
            .key()
            .clone();
        assert_eq!(before, after);
    }
}
