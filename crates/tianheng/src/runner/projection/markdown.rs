use serde_json::Value;

/// The `list --format markdown` projection: an agent-readable summary of the *whole* declared
/// law. It is rendered from the very [`Value`] `list_document` emits, so it provably carries
/// no information absent from the JSON and covers exactly the same dimensions.
pub(in crate::runner) fn list_markdown(document: &Value) -> String {
    let name = document
        .get("constitution")
        .and_then(Value::as_str)
        .unwrap_or("(unnamed)");
    let mut out = format!("# Constitution: {name}\n");
    // The dimension sections in projection order; each key matches `list_document`'s, and a
    // section absent or empty there is skipped here, so the two projections stay in lockstep.
    for (key, heading) in [
        ("boundaries", "Static boundaries"),
        (
            "semantic_boundaries",
            "Semantic boundaries (signature-coupling)",
        ),
        ("trait_impl_boundaries", "Trait-impl-locality boundaries"),
        ("visibility_boundaries", "Visibility boundaries"),
        ("forbidden_marker_boundaries", "Forbidden-marker boundaries"),
        ("dyn_trait_boundaries", "Dyn-trait boundaries"),
        ("impl_trait_boundaries", "Impl-trait boundaries"),
        ("async_exposure_boundaries", "Async-exposure boundaries"),
        (
            "unsafe_confinement_boundaries",
            "Unsafe-confinement boundaries",
        ),
        ("runtime_boundaries", "Runtime boundaries"),
    ] {
        let Some(Value::Array(items)) = document.get(key) else {
            continue;
        };
        if items.is_empty() {
            continue;
        }
        out.push_str(&format!("\n## {heading}\n"));
        for item in items {
            out.push_str(&boundary_markdown(item));
        }
    }
    out
}

/// One boundary as a Markdown block, with the declared `reason` **foregrounded**.
pub(super) fn boundary_markdown(boundary: &Value) -> String {
    let field = |key: &str| boundary.get(key).and_then(Value::as_str).unwrap_or("");
    let mut out = format!("\n### `{}`\n", field("target"));

    let reason = field("reason");
    if !reason.is_empty() {
        out.push_str(&format!("\n> {reason}\n\n"));
    }

    out.push_str(&format!("- **rule**: {}", field("rule")));
    let params = boundary_params(boundary);
    if !params.is_empty() {
        out.push_str(&format!(" ({params})"));
    }
    out.push('\n');

    let mut context = format!("- **kind**: {}", field("kind"));
    let severity = field("severity");
    if !severity.is_empty() {
        context.push_str(&format!(" · **severity**: {severity}"));
    }
    if let Some(krate) = boundary.get("crate").and_then(Value::as_str) {
        context.push_str(&format!(" · **crate**: {krate}"));
    }
    out.push_str(&context);
    out.push('\n');
    out
}

/// The rule parameters of a boundary — every JSON field that is not one of the structural keys
/// (kind/target/crate/rule/severity/reason) — rendered inline. `pub(in crate::runner)` so a
/// projection test can pin `STRUCTURAL` against `boundary_json_base`'s emitted keys (guarding the
/// hand-maintained list from drift).
pub(in crate::runner) fn boundary_params(boundary: &Value) -> String {
    const STRUCTURAL: [&str; 6] = ["kind", "target", "crate", "rule", "severity", "reason"];
    let Some(object) = boundary.as_object() else {
        return String::new();
    };
    object
        .iter()
        .filter(|(key, _)| !STRUCTURAL.contains(&key.as_str()))
        .map(|(key, value)| format!("{key}: {}", inline_value(value)))
        .collect::<Vec<_>>()
        .join("; ")
}

/// Render a JSON value compactly for a Markdown parameter.
pub(super) fn inline_value(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(inline_value)
            .collect::<Vec<_>>()
            .join(", "),
        Value::Bool(boolean) => boolean.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => "null".to_string(),
        Value::Object(_) => value.to_string(),
    }
}
