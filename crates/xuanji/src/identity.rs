//! Vocabulary-neutral structured identity primitives.

use serde_json::Value;
use std::collections::BTreeMap;

fn canonical_fields<I, K, V>(kind: &str, fields: I) -> Result<BTreeMap<String, String>, String>
where
    I: IntoIterator<Item = (K, V)>,
    K: Into<String>,
    V: Into<String>,
{
    let mut canonical = BTreeMap::new();
    for (name, value) in fields {
        let name = name.into();
        if name.is_empty() {
            return Err(format!("{kind} field name must not be empty"));
        }
        if canonical.insert(name.clone(), value.into()).is_some() {
            return Err(format!("{kind} field `{name}` is duplicated"));
        }
    }
    Ok(canonical)
}

fn required_string<'a>(value: &'a Value, kind: &str, name: &str) -> Result<&'a str, String> {
    value[name]
        .as_str()
        .ok_or_else(|| format!("{kind} is missing string `{name}`"))
}

fn json_fields<'a>(value: &'a Value, kind: &str) -> Result<Vec<(&'a str, &'a str)>, String> {
    value["fields"]
        .as_object()
        .ok_or_else(|| format!("{kind} `fields` must be an object"))?
        .iter()
        .map(|(name, value)| {
            value
                .as_str()
                .map(|value| (name.as_str(), value))
                .ok_or_else(|| format!("{kind} field `{name}` must be a string"))
        })
        .collect()
}

/// A validated semantic key for one rule family and its identity-bearing parameters.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct RuleKey {
    rule_type: String,
    fields: BTreeMap<String, String>,
}

impl RuleKey {
    /// Build a semantic rule key.
    pub fn new<I, K, V>(rule_type: impl Into<String>, fields: I) -> Result<Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let rule_type = rule_type.into();
        if rule_type.is_empty() {
            return Err("rule key type must not be empty".to_string());
        }
        Ok(Self {
            rule_type,
            fields: canonical_fields("rule key", fields)?,
        })
    }

    /// Build a key from a statically known rule schema.
    pub fn of<I, K, V>(rule_type: impl Into<String>, fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::new(rule_type, fields)
            .expect("rule schemas use non-empty semantic types and unique static field names")
    }

    /// The semantic rule-family identifier.
    pub fn rule_type(&self) -> &str {
        &self.rule_type
    }

    /// Canonically name-ordered identity-bearing rule fields.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Project the rule key into canonical JSON.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "type": self.rule_type,
            "fields": self.fields,
        })
    }

    pub(crate) fn from_json(value: &Value) -> Result<Self, String> {
        Self::new(
            required_string(value, "rule key", "type")?,
            json_fields(value, "rule key")?,
        )
    }
}

/// A validated, dimension-agnostic identity for one observed fact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StructuredFactIdentity {
    fact_type: String,
    shape: String,
    fields: BTreeMap<String, String>,
}

impl StructuredFactIdentity {
    /// Build a structured observed-fact identity.
    pub fn new<I, K, V>(
        fact_type: impl Into<String>,
        shape: impl Into<String>,
        fields: I,
    ) -> Result<Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let fact_type = fact_type.into();
        if fact_type.is_empty() {
            return Err("fact identity type must not be empty".to_string());
        }
        let shape = shape.into();
        if shape.is_empty() {
            return Err("fact identity shape must not be empty".to_string());
        }
        Ok(Self {
            fact_type,
            shape,
            fields: canonical_fields("fact identity", fields)?,
        })
    }

    /// Build an identity from a statically known fact schema.
    pub fn of<I, K, V>(fact_type: impl Into<String>, shape: impl Into<String>, fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::new(fact_type, shape, fields).expect(
            "fact schemas use non-empty semantic types/shapes and unique static field names",
        )
    }

    /// The semantic fact-family identifier.
    pub fn fact_type(&self) -> &str {
        &self.fact_type
    }

    /// The semantic identity-shape identifier within the fact family.
    pub fn shape(&self) -> &str {
        &self.shape
    }

    /// Canonically name-ordered identity-bearing fact fields.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Project the fact identity into the temporary 0.2 finding-key JSON shape.
    ///
    /// The semantic `type` / `shape` machine projection is introduced only after all
    /// instruments migrate, so this additive expansion does not silently rewrite baselines.
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "namespace": self.fact_type,
            "code": self.shape,
            "fields": self.fields,
        })
    }

    pub(crate) fn semantic_json(&self) -> Value {
        serde_json::json!({
            "type": self.fact_type,
            "shape": self.shape,
            "fields": self.fields,
        })
    }

    pub(crate) fn from_semantic_json(value: &Value) -> Result<Self, String> {
        Self::new(
            required_string(value, "fact identity", "type")?,
            required_string(value, "fact identity", "shape")?,
            json_fields(value, "fact identity")?,
        )
    }

    /// Temporary 0.2 migration accessor for the former dimension namespace role.
    #[doc(hidden)]
    pub fn namespace(&self) -> &str {
        self.fact_type()
    }

    /// Temporary 0.2 migration accessor for the former fact-code role.
    #[doc(hidden)]
    pub fn code(&self) -> &str {
        self.shape()
    }
}
