//! Dimension-agnostic structured identity and human finding text.

use serde_json::Value;
use std::collections::BTreeMap;

/// A dimension-agnostic identity for one observed fact.
///
/// The observation dimension owns the meaning of namespace, code, and fields.
/// `xuanji` maintains the canonical validated envelope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FindingKey {
    namespace: String,
    code: String,
    fields: BTreeMap<String, String>,
}

impl FindingKey {
    /// Build a structured finding key.
    ///
    /// Namespaces and codes must be non-empty. Field names must be unique and non-empty.
    pub fn new<I, K, V>(
        namespace: impl Into<String>,
        code: impl Into<String>,
        fields: I,
    ) -> Result<Self, String>
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        let namespace = namespace.into();
        if namespace.is_empty() {
            return Err("finding key namespace must not be empty".to_string());
        }
        let code = code.into();
        if code.is_empty() {
            return Err("finding key code must not be empty".to_string());
        }
        let mut canonical = BTreeMap::new();
        for (name, value) in fields {
            let name = name.into();
            if name.is_empty() {
                return Err("finding key field name must not be empty".to_string());
            }
            if canonical.insert(name.clone(), value.into()).is_some() {
                return Err(format!("finding key field `{name}` is duplicated"));
            }
        }
        Ok(Self {
            namespace,
            code,
            fields: canonical,
        })
    }

    /// Build a key from a statically-known fact schema.
    pub fn of<I, K, V>(namespace: impl Into<String>, code: impl Into<String>, fields: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        Self::new(namespace, code, fields)
            .expect("fact schemas use non-empty, unique static field names")
    }

    /// The observation dimension namespace.
    pub fn namespace(&self) -> &str {
        &self.namespace
    }

    /// The dimension-owned fact code.
    pub fn code(&self) -> &str {
        &self.code
    }

    /// Canonically name-ordered identity fields.
    pub fn fields(&self) -> impl Iterator<Item = (&str, &str)> {
        self.fields
            .iter()
            .map(|(name, value)| (name.as_str(), value.as_str()))
    }

    /// Project key into canonical JSON object (`namespace`, `code`, `fields`).
    pub fn to_json(&self) -> Value {
        serde_json::json!({
            "namespace": self.namespace,
            "code": self.code,
            "fields": self.fields,
        })
    }

    pub(crate) fn from_json(value: &Value) -> Result<Self, String> {
        let string = |name: &str| {
            value[name]
                .as_str()
                .ok_or_else(|| format!("finding key is missing string `{name}`"))
        };
        let fields = value["fields"]
            .as_object()
            .ok_or_else(|| "finding key `fields` must be an object".to_string())?
            .iter()
            .map(|(name, value)| {
                value
                    .as_str()
                    .map(|value| (name.as_str(), value))
                    .ok_or_else(|| format!("finding key field `{name}` must be a string"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Self::new(string("namespace")?, string("code")?, fields)
    }
}

/// Pair human-readable finding text with stable structured identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub(crate) text: String,
    pub(crate) key: FindingKey,
}

impl Finding {
    /// Pair human finding text with dimension-owned stable key.
    pub fn new(text: impl Into<String>, key: FindingKey) -> Self {
        Self {
            text: text.into(),
            key,
        }
    }

    /// Human-readable finding text.
    pub fn text(&self) -> &str {
        &self.text
    }

    /// Stable structured identity key.
    pub fn key(&self) -> &FindingKey {
        &self.key
    }
}
