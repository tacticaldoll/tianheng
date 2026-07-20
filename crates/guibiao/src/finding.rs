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
