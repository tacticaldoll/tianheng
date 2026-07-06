/// Gate a checked-in projection against the live one — the 潛移 staleness reaction, reusable by
/// adopters. Pair it with [`super::constitution_markdown`]: generate a Markdown projection of your
/// constitution into a checked-in file (so your agents read the reacted law in-context, the
/// **projection** track), keep a short hand-written spine (`AGENTS.md`-style, the **prose** track),
/// and gate the projection's freshness in a `cargo test` so the file can never drift from the
/// declared law — the same mechanism Tianheng runs on its own `AGENTS.self-law.md`.
///
/// - `live` — the freshly generated projection (e.g. `constitution_markdown(&c)`).
/// - `path` — the checked-in artifact.
/// - `regenerate` — the command echoed in the error (e.g. `"BLESS=1 cargo test --test law"`).
/// - `bless` — when `true`, overwrite `path` with `live` (creating any missing parent directories);
///   when `false`, compare. The **caller** supplies this (typically
///   `std::env::var_os("BLESS").is_some()`), so this function reads no environment and is a pure
///   function of its arguments — no process-global state, safe under parallel tests.
///
/// Returns `Ok(())` when the file byte-matches `live` (or was blessed); `Err` — naming both the path
/// and `regenerate` — when the file differs, is missing, or is unreadable (a projection that cannot
/// be confirmed fresh is "cannot judge", never a silent pass), or when a bless write fails.
///
/// ```no_run
/// use tianheng::prelude::*;
/// let c = Constitution::new("my-project"); // … your declared boundaries
/// let live = tianheng::constitution_markdown(&c);
/// let bless = std::env::var_os("BLESS").is_some();
/// // Call this inside a `#[test]`:
/// tianheng::projection_gate(
///     &live,
///     std::path::Path::new("AGENTS.my-law.md"),
///     "BLESS=1 cargo test",
///     bless,
/// )
/// .unwrap();
/// ```
pub fn projection_gate(
    live: &str,
    path: &std::path::Path,
    regenerate: &str,
    bless: bool,
) -> Result<(), String> {
    if bless {
        // A first bless may target a not-yet-existing subdir; `fs::write` does not create parents.
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent).map_err(|e| {
                    format!(
                        "cannot create {}: {e} — regenerate with `{regenerate}`",
                        parent.display()
                    )
                })?;
            }
        }
        return std::fs::write(path, live).map_err(|e| {
            format!(
                "cannot write {}: {e} — regenerate with `{regenerate}`",
                path.display()
            )
        });
    }
    // "Cannot confirm fresh" (missing/unreadable) is a reaction, never a silent pass.
    let checked_in = std::fs::read_to_string(path).map_err(|e| {
        format!(
            "cannot read {}: {e} — regenerate it with `{regenerate}`",
            path.display()
        )
    })?;
    if checked_in == live {
        Ok(())
    } else {
        Err(format!(
            "{} is stale; regenerate it with `{regenerate}`",
            path.display()
        ))
    }
}
