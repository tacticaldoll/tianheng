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

    // **Every field that distinguishes a boundary is rendered, and none of them is a branch on kind.**
    //
    // The heading was `target` alone, which is unique for a crate boundary and is not for the others:
    // `.module("crate")` is the ordinary subtree-wide form, so five boundaries rendered the identical
    // ``### `crate` `` while the half that told them apart — the crate — sat three lines lower. Four of them
    // were consecutive. This is the surface `AGENTS.md` step 1 points an agent at to learn the shape it must
    // not drift, and the same renderer is the shipped `tianheng list --format markdown`.
    //
    // Kind is rendered **data**, never a `match` arm, so a new `BoundaryKind` is identified here without this
    // function being touched. The one branch is on whether a `crate` field exists — which decides how the
    // path is joined, not how much identity is shown; every kind shows all of it.
    //
    // This narrows collisions and cannot close them: `rule` distinguishes two boundaries as well, and is not
    // in the heading. What construction cannot guarantee is held by
    // `markdown_headings_are_pairwise_distinct`.
    let target = field("target");
    let qualified = match boundary.get("crate").and_then(Value::as_str) {
        Some(krate) => format!("{krate}::{target}"),
        None => target.to_string(),
    };
    let mut out = format!("\n### `{qualified}` ({})\n", field("kind"));

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

    if let Some(anchor) = boundary.get("anchor").and_then(Value::as_str) {
        out.push_str(&format!("- **anchor**: {anchor}\n"));
    }

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
/// (kind/target/crate/rule/severity/reason/anchor) — rendered inline. `pub(in crate::runner)` so a
/// projection test can pin `STRUCTURAL` against `boundary_json_base`'s emitted keys (guarding the
/// hand-maintained list from drift).
pub(in crate::runner) fn boundary_params(boundary: &Value) -> String {
    const STRUCTURAL: [&str; 7] = [
        "kind", "target", "crate", "rule", "severity", "reason", "anchor",
    ];
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
