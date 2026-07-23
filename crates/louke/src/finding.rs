//! Runtime-dimension facts and their stable shared-reaction projection.

use xuanji::{Finding, StructuredFactIdentity};

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
    // `owner` is the owner-qualified enclosing item (never a bare name — see `fn_scopes` in
    // `audit::scan`), and `expr` the offending expression's own trimmed source text; together
    // with `file` these are the identity discriminator, never a byte offset or occurrence count.
    #[cfg(feature = "audit")]
    UnauditableProbe {
        file: String,
        owner: String,
        expr: String,
    },
}

impl RuntimeFact {
    pub(crate) fn into_finding(self) -> Finding {
        match self {
            Self::RegisteredCrossing { origin, type_name } => Finding::new(
                format!("{origin} ({type_name})"),
                key(
                    "tianheng.fact/louke/runtime-crossing",
                    "registered-origin",
                    [
                        ("origin", origin.as_str()),
                        ("type_name", type_name.as_str()),
                    ],
                ),
            ),
            Self::UnregisteredCrossing { type_id } => Finding::new(
                format!("<unregistered origin> {type_id}"),
                key(
                    "tianheng.fact/louke/runtime-crossing",
                    "unregistered-origin",
                    [("type_id", type_id.as_str())],
                ),
            ),
            #[cfg(feature = "audit")]
            Self::DuplicateSeam { seam } => Finding::new(
                format!("seam '{seam}' is declared more than once"),
                key(
                    "tianheng.fact/louke/runtime-seam-audit",
                    "duplicate-declaration",
                    [("seam", seam.as_str())],
                ),
            ),
            #[cfg(feature = "audit")]
            Self::UnprobedSeam { seam } => Finding::new(
                format!("declared seam '{seam}' has no assert_boundary! probe"),
                key(
                    "tianheng.fact/louke/runtime-seam-audit",
                    "unprobed-declaration",
                    [("seam", seam.as_str())],
                ),
            ),
            #[cfg(feature = "audit")]
            Self::UndeclaredProbe { seam } => Finding::new(
                format!("probe references undeclared seam '{seam}'"),
                key(
                    "tianheng.fact/louke/runtime-seam-audit",
                    "undeclared-probe",
                    [("seam", seam.as_str())],
                ),
            ),
            #[cfg(feature = "audit")]
            Self::UnauditableProbe { file, owner, expr } => Finding::new(
                format!(
                    "{file}: {owner} has an assert_boundary! probe with a non-literal seam \
                     `{expr}` (const or expression), which the CI face cannot trace to a \
                     declared seam"
                ),
                key(
                    "tianheng.fact/louke/runtime-seam-audit",
                    "unauditable-probe",
                    [
                        ("file", file.as_str()),
                        ("owner", owner.as_str()),
                        ("expr", expr.as_str()),
                    ],
                ),
            ),
        }
    }
}

fn key<const N: usize>(
    fact_type: &str,
    shape: &str,
    fields: [(&str, &str); N],
) -> StructuredFactIdentity {
    StructuredFactIdentity::of(fact_type, shape, fields)
}

// The production catalog runs with audit both off and on; audit-only cases extend it when that
// face exists, so neither configuration can silently lose its own fact schemas.
#[cfg(test)]
mod tests {
    use super::*;

    type KeyCase = (
        RuntimeFact,
        &'static str,
        &'static str,
        Vec<(&'static str, &'static str)>,
    );

    fn assert_runtime_fact_is_cataloged(fact: &RuntimeFact) {
        match fact {
            RuntimeFact::RegisteredCrossing {
                origin: _,
                type_name: _,
            }
            | RuntimeFact::UnregisteredCrossing { type_id: _ } => {}
            #[cfg(feature = "audit")]
            RuntimeFact::DuplicateSeam { seam: _ }
            | RuntimeFact::UnprobedSeam { seam: _ }
            | RuntimeFact::UndeclaredProbe { seam: _ }
            | RuntimeFact::UnauditableProbe {
                file: _,
                owner: _,
                expr: _,
            } => {}
        }
    }

    fn assert_case((fact, fact_type, shape, fields): KeyCase) {
        assert_runtime_fact_is_cataloged(&fact);
        let finding = fact.into_finding();
        assert_eq!(finding.key().fact_type(), fact_type);
        assert_eq!(finding.key().shape(), shape);
        assert_eq!(finding.key().fields().collect::<Vec<_>>(), fields);
    }

    #[test]
    fn published_runtime_fact_identity_schema_is_exact_and_exhaustive() {
        let cases: Vec<KeyCase> = vec![
            (
                RuntimeFact::RegisteredCrossing {
                    origin: "app::adapter".to_string(),
                    type_name: "SqlAdapter".to_string(),
                },
                "tianheng.fact/louke/runtime-crossing",
                "registered-origin",
                vec![("origin", "app::adapter"), ("type_name", "SqlAdapter")],
            ),
            (
                RuntimeFact::UnregisteredCrossing {
                    type_id: "TypeId(0x1234)".to_string(),
                },
                "tianheng.fact/louke/runtime-crossing",
                "unregistered-origin",
                vec![("type_id", "TypeId(0x1234)")],
            ),
        ];
        for case in cases {
            assert_case(case);
        }

        #[cfg(feature = "audit")]
        for case in [
            (
                RuntimeFact::DuplicateSeam {
                    seam: "checkout".to_string(),
                },
                "tianheng.fact/louke/runtime-seam-audit",
                "duplicate-declaration",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UnprobedSeam {
                    seam: "checkout".to_string(),
                },
                "tianheng.fact/louke/runtime-seam-audit",
                "unprobed-declaration",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UndeclaredProbe {
                    seam: "checkout".to_string(),
                },
                "tianheng.fact/louke/runtime-seam-audit",
                "undeclared-probe",
                vec![("seam", "checkout")],
            ),
            (
                RuntimeFact::UnauditableProbe {
                    file: "src/lib.rs".to_string(),
                    owner: "fn install".to_string(),
                    expr: "SEAM_CONST".to_string(),
                },
                "tianheng.fact/louke/runtime-seam-audit",
                "unauditable-probe",
                vec![
                    ("expr", "SEAM_CONST"),
                    ("file", "src/lib.rs"),
                    ("owner", "fn install"),
                ],
            ),
        ] {
            assert_case(case);
        }
    }

    #[cfg(feature = "audit")]
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

    #[cfg(feature = "audit")]
    #[test]
    fn unauditable_probe_identity_distinguishes_owner_and_expression() {
        let base = RuntimeFact::UnauditableProbe {
            file: "src/lib.rs".to_string(),
            owner: "fn a".to_string(),
            expr: "SEAM_A".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        let different_owner = RuntimeFact::UnauditableProbe {
            file: "src/lib.rs".to_string(),
            owner: "fn b".to_string(),
            expr: "SEAM_A".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        let different_expr = RuntimeFact::UnauditableProbe {
            file: "src/lib.rs".to_string(),
            owner: "fn a".to_string(),
            expr: "compute_seam()".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        let different_file = RuntimeFact::UnauditableProbe {
            file: "src/other.rs".to_string(),
            owner: "fn a".to_string(),
            expr: "SEAM_A".to_string(),
        }
        .into_finding()
        .key()
        .clone();
        assert_ne!(base, different_owner);
        assert_ne!(base, different_expr);
        assert_ne!(base, different_file);
    }
}
