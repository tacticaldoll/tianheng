//! Serialization and helper utilities for `xuanji`.

use serde_json::Value;

/// Serialize an owned [`Value`] to pretty JSON.
///
/// Infallible by construction: a `Value`'s `Serialize` impl does not fail for finite JSON data.
/// Fail-loud is reserved for observable misconfiguration, not unreachable states.
pub fn pretty_json(document: &Value) -> String {
    serde_json::to_string_pretty(document).expect("a serde_json::Value is always serializable")
}
