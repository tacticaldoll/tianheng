//! Runtime-dimension facts and their stable shared-reaction projection.

use xuanji::{Finding, FindingKey};

pub(crate) enum RuntimeFact {
    RegisteredCrossing {
        origin: String,
        type_name: String,
    },
    UnregisteredCrossing {
        type_id: String,
    },
    // The probe-coverage facts are produced only by the CI-audit face (`mod audit`, gated behind the
    // non-default `audit` feature), so they exist only when that face does — otherwise they are dead
    // in the prod-light default build. Gated to match their sole constructor rather than silenced.
    #[cfg(feature = "audit")]
    DuplicateSeam {
        seam: String,
    },
    #[cfg(feature = "audit")]
    UnprobedSeam {
        seam: String,
    },
    #[cfg(feature = "audit")]
    UndeclaredProbe {
        seam: String,
    },
    #[cfg(feature = "audit")]
    UnauditableProbe {
        file: String,
    },
}

impl RuntimeFact {
    pub(crate) fn into_finding(self) -> Finding {
        match self {
            Self::RegisteredCrossing { origin, type_name } => Finding::new(
                format!("{origin} ({type_name})"),
                key(
                    "registered_crossing",
                    [
                        ("origin", origin.as_str()),
                        ("type_name", type_name.as_str()),
                    ],
                ),
            ),
            Self::UnregisteredCrossing { type_id } => Finding::new(
                format!("<unregistered origin> {type_id}"),
                key("unregistered_crossing", [("type_id", type_id.as_str())]),
            ),
            #[cfg(feature = "audit")]
            Self::DuplicateSeam { seam } => Finding::new(
                format!("seam '{seam}' is declared more than once"),
                key("duplicate_seam", [("seam", seam.as_str())]),
            ),
            #[cfg(feature = "audit")]
            Self::UnprobedSeam { seam } => Finding::new(
                format!("declared seam '{seam}' has no assert_boundary! probe"),
                key("unprobed_seam", [("seam", seam.as_str())]),
            ),
            #[cfg(feature = "audit")]
            Self::UndeclaredProbe { seam } => Finding::new(
                format!("probe references undeclared seam '{seam}'"),
                key("undeclared_probe", [("seam", seam.as_str())]),
            ),
            #[cfg(feature = "audit")]
            Self::UnauditableProbe { file } => Finding::new(
                format!(
                    "{file} has an assert_boundary! probe with a non-literal seam (const or \
                     expression), which the CI face cannot trace to a declared seam"
                ),
                key("unauditable_probe", [("file", file.as_str())]),
            ),
        }
    }
}

fn key<const N: usize>(code: &str, fields: [(&str, &str); N]) -> FindingKey {
    FindingKey::of("louke", code, fields)
}

// The only fact-distinctness test here exercises the audit-only probe-coverage facts, so it lives
// with them under the `audit` feature; CI runs the workspace tests with `--all-features`.
#[cfg(all(test, feature = "audit"))]
mod tests {
    use super::*;

    type KeyCase = (RuntimeFact, &'static str, Vec<(&'static str, &'static str)>);

    fn assert_runtime_fact_is_cataloged(fact: &RuntimeFact) {
        match fact {
            RuntimeFact::RegisteredCrossing {
                origin: _,
                type_name: _,
            }
            | RuntimeFact::UnregisteredCrossing { type_id: _ }
            | RuntimeFact::DuplicateSeam { seam: _ }
            | RuntimeFact::UnprobedSeam { seam: _ }
            | RuntimeFact::UndeclaredProbe { seam: _ }
            | RuntimeFact::UnauditableProbe { file: _ } => {}
        }
    }

    #[test]
    fn published_runtime_fact_identity_schema_is_exact_and_exhaustive() {
        let cases: Vec<KeyCase> = vec![
            (
                RuntimeFact::RegisteredCrossing {
                    origin: "app::adapter".to_string(),
                    type_name: "SqlAdapter".to_string(),
                },
                "registered_crossing",
                vec![("origin", "app::adapter"), ("type_name", "SqlAdapter")],
            ),
            (
                RuntimeFact::UnregisteredCrossing {
                    type_id: "TypeId(0x1234)".to_string(),
                },
                "unregistered_crossing",
                vec![("type_id", "TypeId(0x1234)")],
            ),
            (
                RuntimeFact::DuplicateSeam {
                    seam: "checkout".to_string(),
                },
                "duplicate_seam",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UnprobedSeam {
                    seam: "checkout".to_string(),
                },
                "unprobed_seam",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UndeclaredProbe {
                    seam: "checkout".to_string(),
                },
                "undeclared_probe",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UnauditableProbe {
                    file: "src/lib.rs".to_string(),
                },
                "unauditable_probe",
                vec![("file", "src/lib.rs")],
            ),
        ];

        for (fact, code, fields) in cases {
            assert_runtime_fact_is_cataloged(&fact);
            let finding = fact.into_finding();
            assert_eq!(finding.key().namespace(), "louke");
            assert_eq!(finding.key().code(), code);
            assert_eq!(finding.key().fields().collect::<Vec<_>>(), fields);
        }
    }

    #[test]
    fn runtime_fact_shape_and_values_stay_distinct() {
        let missing = RuntimeFact::UnprobedSeam {
            seam: "checkout".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        let undeclared = RuntimeFact::UndeclaredProbe {
            seam: "checkout".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        let other = RuntimeFact::UnprobedSeam {
            seam: "billing".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        assert_ne!(missing, undeclared);
        assert_ne!(missing, other);
    }
}
