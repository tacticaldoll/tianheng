//! The module-graph walk: from a crate's precomputed file list, resolve which modules Rust
//! actually compiles — the `mod`-declared graph reachable from the crate root — and select the
//! source files that belong to a governed module. An undeclared orphan file, an inline-only
//! shadow, and a `#[path]`-remapped module are all excluded, matching the compiler. Depends
//! downward on [`super::lexer`] (hygiene / token boundaries) and [`super::path_vocab`] (segment
//! canonicalization, containment, the `mod`-keyword test); reads files via `std::fs`.

use super::lexer::{is_ident_byte, strip_comments_and_strings, strip_macro_bodies};
use super::path_vocab::{canonical_segment, is_mod_declaration_keyword, path_within};
use std::path::{Path, PathBuf};

/// Every `(file, module path)` that belongs to the governed `module` (it *is* `module`
/// or sits beneath it) **and** is reachable in the crate's module graph (see
/// [`reachable_modules`]). An undeclared orphan file is not a governed source file even
/// when its path would map under the module, because Rust never compiles it — so its
/// imports must not be observed. Operates on a precomputed file list so the crate's
/// source is scanned once per boundary.
pub(crate) fn governed_files(
    src_dir: &Path,
    files: &[PathBuf],
    module: &str,
    reachable: &std::collections::BTreeSet<String>,
    inline_only: &std::collections::BTreeSet<String>,
    root_relative: Option<&Path>,
) -> Vec<(PathBuf, String)> {
    files
        .iter()
        .filter_map(|file| {
            let relative = file.strip_prefix(src_dir).ok()?;
            let module_path = module_path_of(relative, root_relative);
            // A conventional file whose path is claimed by an inline-only module is an orphan Rust
            // never compiles as that module (the inline body is the module), so it is not a
            // governed source file — the same "not compiled ⇒ not governed" rule as an undeclared
            // orphan, keyed on the inline shadow rather than mere unreachability.
            if inline_only.contains(&module_path) {
                return None;
            }
            if !reachable.contains(&module_path) {
                return None;
            }
            if path_within(&module_path, module) {
                Some((file.clone(), module_path))
            } else {
                None
            }
        })
        .collect()
}

/// The set of module paths reachable from the crate root via `mod` declarations — the
/// crate's real module graph as Rust compiles it, observed from source. The walk seeds at
/// `crate` (the crate-root file(s) `lib.rs` / `main.rs`), reads each reachable module's
/// file(s) for their top-level `mod name;` / `mod name { … }` declarations, and adds the
/// child `crate::…::name`.
///
/// A file's existence alone does **not** make it a module: an undeclared orphan file —
/// at the crate root (`src/serde.rs` with no `mod serde;`) or in a subtree
/// (`src/kernel/orphan.rs` that `kernel` never declares) — is never reached, so it is not
/// governed and its imports are not observed, matching the compiler (which never compiles
/// it). This realizes the spec's "a sibling `mod` is in scope" rule for the whole module
/// tree, not just the crate root. A reachable file that cannot be read is a scan error
/// ("cannot judge"), never a silent skip.
///
/// File-backed children only: an inline `mod name { … }` adds `crate::…::name` to the reachable
/// set, but its own file-backed sub-`mod`s sit at brace depth > 0 and are not walked — a
/// documented partial-coverage gap, the safe direction (a missed import, never a false one).
///
/// Returns `(reachable, inline_only)`. `inline_only` names every path declared **inline-only** —
/// declared with an inline body and NOT also declared file-form (`mod name;`) in its parent — so a
/// same-named conventional file (`name.rs` / `name/mod.rs`) beside it is an orphan Rust never
/// compiles as that module. The walk does not read such a file (nor mine it for phantom children),
/// and [`governed_files`] excludes it, so an inline target remains the self-describing inline
/// constitution error rather than silently governing the orphan. A path declared **both** ways —
/// which in valid source arises only under mutually-exclusive `#[cfg]` — is not inline-only and
/// keeps being observed through its conventional file (the existing cfg-blind lexical bound).
pub(crate) fn reachable_modules(
    src_dir: &Path,
    files: &[PathBuf],
    root_relative: Option<&Path>,
) -> Result<
    (
        std::collections::BTreeSet<String>,
        std::collections::BTreeSet<String>,
    ),
    String,
> {
    // Index files by their path-derived module path so a module's file(s) are found fast.
    let mut by_module: std::collections::BTreeMap<String, Vec<&PathBuf>> = Default::default();
    for file in files {
        if let Ok(relative) = file.strip_prefix(src_dir) {
            by_module
                .entry(module_path_of(relative, root_relative))
                .or_default()
                .push(file);
        }
    }

    let mut reachable = std::collections::BTreeSet::new();
    let mut inline_only = std::collections::BTreeSet::new();
    reachable.insert("crate".to_string());
    let mut queue = vec!["crate".to_string()];
    while let Some(module) = queue.pop() {
        // An inline-only module owns no source file of its own; its same-named conventional file
        // (if any) is an orphan Rust does not compile, so do not read it — reading it would both
        // scan the wrong body and mine phantom child modules from a file that is not the module.
        if inline_only.contains(&module) {
            continue;
        }
        let Some(module_files) = by_module.get(&module) else {
            continue; // an inline module owns no file of its own; nothing to read
        };
        // Classify each child across this module's file(s) before descending: a child seen with an
        // inline body but never a file declaration is inline-only. (A path seen both ways arises
        // only under mutually-exclusive `#[cfg]`; it is not inline-only — the cfg-blind bound.)
        let mut child_kinds: std::collections::BTreeMap<String, (bool, bool)> = Default::default();
        for file in module_files {
            let text = std::fs::read_to_string(file)
                .map_err(|err| format!("cannot read source file '{}': {err}", file.display()))?;
            for (child, is_inline) in declared_modules_with_kind(&text) {
                let seen = child_kinds.entry(child).or_default();
                if is_inline {
                    seen.0 = true;
                } else {
                    seen.1 = true;
                }
            }
        }
        for (child, (seen_inline, seen_file)) in child_kinds {
            let child_path = format!("{module}::{child}");
            if seen_inline && !seen_file {
                inline_only.insert(child_path.clone());
            }
            if reachable.insert(child_path.clone()) {
                queue.push(child_path);
            }
        }
    }
    Ok((reachable, inline_only))
}

/// Names of modules declared at the top level (brace depth 0) of `source`, each paired with
/// whether it is an **inline** declaration (`mod name { … }`, `true`) or a **file** declaration
/// (`mod name;`, `false`) — the distinction [`reachable_modules`] needs to tell a real
/// file-backed module from an inline body whose same-named conventional file is an orphan.
/// Declared at any visibility (`pub mod`, `pub(crate) mod`, …). Comments, string/char literals,
/// and macro bodies are stripped first, so a commented-out, quoted, or macro-generated `mod` is
/// not counted; a `mod` nested inside another item (depth > 0) declares a child module, not a
/// crate-root one, and is skipped. Names are canonicalized (`r#name` -> `name`). Robust over
/// malformed input: it never panics (the same tolerance as the `use` scanner).
fn declared_modules_with_kind(source: &str) -> Vec<(String, bool)> {
    // Strip macro bodies as well as comments/strings, the same hygiene the `use`
    // scanner applies: a `mod` written inside a macro body is macro-generated and out
    // of scope, so it must not be observed as a real declaration. (A `macro_rules!`
    // body is already excluded by brace depth; this also closes the `()`/`[]`-delimited
    // invocation gap, where `mod` would otherwise sit at brace depth 0.)
    let cleaned = strip_macro_bodies(&strip_comments_and_strings(source));
    let bytes = cleaned.as_bytes();
    let mut names = Vec::new();
    let mut depth: i32 = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
            }
            b'}' => {
                depth -= 1;
                i += 1;
            }
            b'm' if depth == 0 && is_mod_declaration_keyword(bytes, i) => {
                // Read the identifier after `mod`, then confirm a `;` or `{` follows so
                // only real declarations (not a stray `mod` token) are recorded.
                let mut j = i + 3;
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < bytes.len()
                    && !bytes[j].is_ascii_whitespace()
                    && bytes[j] != b';'
                    && bytes[j] != b'{'
                {
                    j += 1;
                }
                let ident = cleaned[start..j].trim();
                let mut k = j;
                while k < bytes.len() && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if !ident.is_empty() {
                    // The delimiter after the identifier tells inline (`{`) from file (`;`);
                    // anything else is a stray `mod` token, not a declaration. A `#[path = "…"]`
                    // attribute only remaps a FILE module (`mod name;`) off the conventional path
                    // (out of scope — a stated bound); on an INLINE `mod name { … }` it is a no-op
                    // for rustc (the body is the module), so an inline module is always recorded —
                    // applying the skip to it would drop a module Rust actually compiles.
                    match bytes.get(k) {
                        Some(b'{') => names.push((canonical_segment(ident).to_string(), true)),
                        Some(b';') if !has_path_attr_before_item(bytes, i) => {
                            names.push((canonical_segment(ident).to_string(), false))
                        }
                        _ => {}
                    }
                }
                i += 3;
            }
            _ => i += 1,
        }
    }
    names
}

/// The declared module names only, discarding the inline/file kind — a test-only convenience;
/// the reachability walk uses [`declared_modules_with_kind`] directly.
#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declared_modules_with_kind(source)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// Whether the top-level item prefix before a `mod` keyword contains a direct
/// `#[path = "..."]` attribute. The static scanner intentionally does not read
/// attributes in general, but this one is the stated coverage boundary: a path-remapped
/// module is not conventionally file-backed, so treating the `mod` token as reachable
/// would govern the wrong file if a same-named conventional file also exists.
fn has_path_attr_before_item(bytes: &[u8], mod_index: usize) -> bool {
    let mut start = 0;
    for i in (0..mod_index).rev() {
        if matches!(bytes[i], b';' | b'{' | b'}') {
            start = i + 1;
            break;
        }
    }
    attr_prefix_has_path(&bytes[start..mod_index])
}

fn attr_prefix_has_path(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes.get(i) != Some(&b'[') {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if bytes[i..].starts_with(b"path")
            && bytes.get(i + 4).is_none_or(|byte| !is_ident_byte(*byte))
        {
            return true;
        }
        // The combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
        // `#[cfg(<pred>)] #[path = "…"]`) is a conditional remap too — recognized cfg-blindly, the
        // same stated `#[path]` bound. Matched only on a genuine nested `path` meta, so a
        // `#[cfg_attr(<pred>, deprecated)]` on a normal file module is not mistaken for a remap.
        if bytes[i..].starts_with(b"cfg_attr")
            && bytes.get(i + 8).is_none_or(|byte| !is_ident_byte(*byte))
            && cfg_attr_prefix_has_path(&bytes[i + 8..])
        {
            return true;
        }
    }
    false
}

/// Whether a `cfg_attr(…)` attribute — `bytes` positioned just after the `cfg_attr` identifier —
/// carries a `path` meta among its **applied attributes**. `cfg_attr(<predicate>, <attr>, …)`: the
/// first meta is the cfg predicate (a condition, not an applied attribute), so it is **skipped**
/// before matching — mirroring hunyi's `is_path_remap` (`metas.iter().skip(1)`), so the two
/// dimensions agree. Scans the balanced parenthesis group and matches a depth-1 `path` identifier,
/// past the predicate, immediately followed by `=` (the `path = "…"` name-value form); it also
/// **recurses** into a nested applied `cfg_attr(…)`, so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
/// detected too. Conservative — a same-suffixed identifier (`target_path`), a `path` nested inside a
/// predicate group (`all(…)`), or a `path` in the predicate position is **not** matched — so a
/// non-remapping `cfg_attr` is never mistaken for a remap (which would drop a governed module — the
/// inverse false negative).
///
/// Input note: this runs on comment/string-stripped bytes (`declared_modules_with_kind` applies
/// `strip_comments_and_strings` first), so a `path` inside a string literal cannot reach here; the
/// `b'"'` arm below is defense-in-depth for that upstream invariant, not a live path.
fn cfg_attr_prefix_has_path(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return false;
    }
    i += 1;
    let mut depth = 1usize;
    // The first depth-1 meta is the cfg predicate, not an applied attribute; only match a `path`
    // meta AFTER the first depth-1 comma, so `#[cfg_attr(path = "…", …)]` (a `path` cfg key) is not
    // mistaken for a remap.
    let mut past_predicate = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b'"' => {
                // Strings are stripped upstream (see doc); defense-in-depth for the invariant.
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    if bytes[i] == b'\\' {
                        i += 1;
                    }
                    i += 1;
                }
                i += 1;
            }
            b'(' => {
                depth += 1;
                i += 1;
            }
            b')' => {
                depth -= 1;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                let start = i;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                if ident == b"path" {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        return true;
                    }
                } else if ident == b"cfg_attr" && cfg_attr_prefix_has_path(&bytes[i..]) {
                    // A nested `cfg_attr(<pred>, …)` applied meta: recurse into ITS group (which
                    // skips its own predicate), so `#[cfg_attr(a, cfg_attr(b, path = "…"))]` is
                    // detected too — matching hunyi's recursive `is_path_remap`.
                    return true;
                }
            }
            _ => i += 1,
        }
    }
    false
}

/// The module path of a source file, mapping the crate ROOT file to `crate` regardless of its
/// filename. Cargo permits a custom target root (`[lib] path = "src/core.rs"`,
/// `[[bin]] path = "src/app.rs"`), which [`file_module_path`] would otherwise map to
/// `crate::core` / `crate::app` — leaving `crate` empty so no submodule is ever reached (a false
/// negative / spurious exit-2). `root_relative` is that root file's path relative to `src_dir`
/// when known; for the conventional `lib.rs`/`main.rs` it coincides with what `file_module_path`
/// already returns, so passing `None` is safe for the common case.
fn module_path_of(relative: &Path, root_relative: Option<&Path>) -> String {
    if root_relative == Some(relative) {
        return "crate".to_string();
    }
    // A custom crate root (`[lib] path = "src/core.rs"`) is in effect when `root_relative` is known
    // and is NOT the conventional top-level `lib.rs`/`main.rs`. In that case a STRAY top-level
    // `lib.rs`/`main.rs` is not the crate root — the explicit `path` disables cargo's lib/bin
    // autodetection, so rustc never compiles it — and must not also claim the segment-less `crate`
    // module (which would union its declared modules into the real root and make them
    // phantom-reachable). It maps to `crate::lib` / `crate::main` like any other file and, being
    // undeclared from the true root, stays unreached — matching the compiler.
    let custom_root = root_relative.is_some_and(|r| !is_conventional_root(r));
    file_module_path(relative, custom_root)
}

/// Whether `relative` is a conventional top-level cargo target root — `lib.rs` or `main.rs`
/// directly under `src/` (no parent segment). These are the roots [`file_module_path`] already maps
/// to the segment-less `crate`; any other root file is a *custom* root set via an explicit
/// `[lib]`/`[[bin]]` `path`.
fn is_conventional_root(relative: &Path) -> bool {
    relative
        .file_name()
        .map(|n| matches!(n.to_string_lossy().as_ref(), "lib.rs" | "main.rs"))
        .unwrap_or(false)
        && relative.components().count() == 1
}

/// The module path of a source file from its path relative to `src/`:
/// `lib.rs`/`main.rs`/`mod.rs` contribute no segment; `kernel/foo.rs` ->
/// `crate::kernel::foo`.
fn file_module_path(relative: &Path, custom_root: bool) -> String {
    let components: Vec<String> = relative
        .components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect();
    let mut segments = vec![String::from("crate")];
    let last = components.len().saturating_sub(1);
    for (index, component) in components.iter().enumerate() {
        if index == last {
            let stem = component.strip_suffix(".rs").unwrap_or(component);
            // `mod.rs` names its directory at any depth. `lib.rs`/`main.rs` are segment-less ONLY at
            // the crate root of a conventional layout — they are the cargo *target* roots there, not
            // module names. When a CUSTOM root is in effect (`custom_root`), a top-level `lib.rs`/
            // `main.rs` is NOT the target root (cargo autodetection is off) and must keep its stem so
            // it does not masquerade as the segment-less `crate` alongside the true root. A declared
            // submodule file literally named `lib.rs`/`main.rs` (`mod lib;` inside a subdir →
            // `foo/lib.rs` = `crate::foo::lib`) contributes its stem like any other file; stripping
            // it at depth would mis-map it to its parent and drift from 渾儀's declaration-driven
            // descent (which resolves it correctly).
            let segmentless = stem == "mod"
                || (!custom_root && components.len() == 1 && matches!(stem, "lib" | "main"));
            if !segmentless {
                segments.push(canonical_segment(stem).to_string());
            }
        } else {
            segments.push(canonical_segment(component).to_string());
        }
    }
    segments.join("::")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::module_scan::rust_files;

    #[test]
    fn declared_modules_finds_only_top_level_declarations() {
        let source = r#"
            pub mod kernel;
            mod projection;
            pub(crate) mod runner;
            mod inline { mod nested_child; }   // nested_child is depth 1, not a root module
            // mod commented_out;
            fn f() { let _ = "mod string_literal;"; }
        "#;
        let mut mods = declared_modules(source);
        mods.sort();
        assert_eq!(
            mods,
            vec![
                "inline".to_string(),
                "kernel".to_string(),
                "projection".to_string(),
                "runner".to_string(),
            ],
            "only top-level mod declarations count; nested, commented, and quoted are excluded"
        );
    }

    #[test]
    fn reachable_modules_follows_mod_declarations_not_filenames() {
        // The crate root declares `mod kernel;`, but two orphan files exist that no `mod`
        // brings into scope: a root orphan (`serde.rs`) and a subtree orphan
        // (`kernel/orphan.rs`, which `kernel.rs` never declares). Only `crate` and
        // `crate::kernel` are reachable; the orphans are not — at the root OR in a subtree.
        let dir = std::env::temp_dir().join(format!("guibiao-reach-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(src.join("kernel")).expect("create temp src/kernel");
        std::fs::write(
            src.join("lib.rs"),
            "pub mod kernel;\nuse serde::Deserialize;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("kernel.rs"), "// kernel declares no submodule\n")
            .expect("write kernel.rs");
        std::fs::write(src.join("serde.rs"), "// root orphan, undeclared\n")
            .expect("write serde.rs");
        std::fs::write(
            src.join("kernel/orphan.rs"),
            "use crate::projection::Thing;\n",
        )
        .expect("write kernel/orphan.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(reachable.contains("crate"), "{reachable:?}");
        assert!(
            reachable.contains("crate::kernel"),
            "a declared `mod kernel;` is reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::serde"),
            "an undeclared root orphan is not reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::kernel::orphan"),
            "an undeclared subtree orphan is not reachable: {reachable:?}"
        );
    }

    #[test]
    fn a_stray_lib_beside_a_custom_root_is_not_a_second_crate_root() {
        // With a custom target root (`[lib] path = "src/core.rs"`), a
        // leftover top-level `lib.rs` is NOT the crate root — cargo never compiles it — so it must
        // not also claim the segment-less `crate` module. If both `core.rs` and `lib.rs` mapped to
        // `crate`, the stray file's `mod ghost;` would union into the real root and make
        // `crate::ghost` phantom-reachable (a spurious module-boundary violation on an uncompiled file).
        let dir = std::env::temp_dir().join(format!("guibiao-custom-root-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
        std::fs::write(src.join("core.rs"), "pub mod real;\n").expect("write core.rs");
        std::fs::write(
            src.join("real.rs"),
            "// real, declared from the true root\n",
        )
        .expect("write real.rs");
        std::fs::write(src.join("lib.rs"), "pub mod ghost;\n").expect("write stray lib.rs");
        std::fs::write(
            src.join("ghost.rs"),
            "// declared only by the uncompiled lib.rs\n",
        )
        .expect("write ghost.rs");

        let files = rust_files(&src).expect("list files");
        let root_relative = std::path::PathBuf::from("core.rs");
        let (reachable, _inline_only) =
            reachable_modules(&src, &files, Some(&root_relative)).expect("walk modules");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(
            reachable.contains("crate"),
            "the custom root seeds crate: {reachable:?}"
        );
        assert!(
            reachable.contains("crate::real"),
            "a module declared from the true root is reachable: {reachable:?}"
        );
        assert!(
            !reachable.contains("crate::ghost"),
            "a module declared only by the stray, uncompiled lib.rs is NOT reachable: {reachable:?}"
        );
    }

    #[test]
    fn path_remapped_modules_are_not_reachable() {
        let dir = std::env::temp_dir().join(format!("guibiao-path-remap-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("create temp src");
        std::fs::write(
            src.join("lib.rs"),
            "#[path = \"weird.rs\"]\npub mod kernel;\npub mod normal;\n",
        )
        .expect("write lib.rs");
        std::fs::write(src.join("weird.rs"), "use crate::projection::Thing;\n")
            .expect("write remapped file");
        std::fs::write(
            src.join("kernel.rs"),
            "use crate::wrong_file_if_observed::Thing;\n",
        )
        .expect("write conventional orphan");
        std::fs::write(src.join("normal.rs"), "// normal module\n").expect("write normal.rs");

        let files = rust_files(&src).expect("list files");
        let (reachable, _inline_only) =
            reachable_modules(&src, &files, None).expect("walk modules");
        let _ = std::fs::remove_dir_all(&dir);

        assert!(reachable.contains("crate::normal"), "{reachable:?}");
        assert!(
            !reachable.contains("crate::kernel"),
            "a #[path]-remapped module is out of scope and must not be mapped to a conventional file: {reachable:?}"
        );
    }

    #[test]
    fn path_attribute_detection_is_specific() {
        assert_eq!(
            declared_modules("#[pathology]\npub mod kernel;\n"),
            vec!["kernel".to_string()],
            "only the real `path` attribute is a remap marker"
        );
        assert!(
            declared_modules("# [ path = \"weird.rs\" ]\npub mod kernel;\n").is_empty(),
            "Rust permits whitespace in an outer attribute head; the remap still stays out of scope"
        );
    }

    #[test]
    fn a_cfg_attr_nested_path_is_a_remap() {
        // `#[cfg_attr(<pred>, path = "…")]` (== `#[cfg(<pred>)] #[path = "…"]`) is a
        // conditional remap — the FILE module is out of scope, the same stated `#[path]` bound. Not
        // recognizing it would scan the module against a wrong/absent conventional file (a silent
        // false negative in the static dimension).
        assert!(
            declared_modules("#[cfg_attr(windows, path = \"os/windows.rs\")]\npub mod os;\n")
                .is_empty(),
            "a cfg_attr-nested path remap puts the FILE module out of scope",
        );
        // The remap may sit after the predicate among several applied attrs, and whitespace varies.
        assert!(
            declared_modules("#[cfg_attr(all(unix), deprecated, path = \"p.rs\")]\nmod a;\n")
                .is_empty(),
        );
        // A NESTED `cfg_attr` remap (== `#[cfg(all(a,b))] #[path]`) is
        // detected too — the recursion must descend the applied `cfg_attr`, or guibiao would
        // silently govern nothing while hunyi failed loud (a cross-dimension divergence).
        assert!(
            declared_modules("#[cfg_attr(a, cfg_attr(b, path = \"secret.rs\"))]\npub mod m;\n")
                .is_empty(),
            "a nested cfg_attr path remap is detected",
        );
    }

    #[test]
    fn a_cfg_attr_without_a_path_meta_is_not_a_remap() {
        // The inverse false negative: a `cfg_attr` that carries NO `path` meta must not be mistaken
        // for a remap, or a normal file module would be dropped from scope and never governed.
        assert_eq!(
            declared_modules("#[cfg_attr(test, derive(Debug))]\npub mod real;\n"),
            vec!["real".to_string()],
            "a cfg_attr without a path meta is not a remap",
        );
        // A `path` substring inside a predicate's STRING value is not a `path` meta.
        assert_eq!(
            declared_modules("#[cfg_attr(feature = \"path\", deprecated)]\npub mod real;\n"),
            vec!["real".to_string()],
            "a `path` inside a predicate string is not a path meta",
        );
        // A same-suffixed identifier (`target_path`) is not the `path` meta.
        assert_eq!(
            declared_modules("#[cfg_attr(unix, target_path = \"x\")]\npub mod real;\n"),
            vec!["real".to_string()],
        );
        // A NESTED cfg_attr that carries no `path` meta must not be mistaken for a remap either.
        assert_eq!(
            declared_modules("#[cfg_attr(a, cfg_attr(b, deprecated))]\npub mod real;\n"),
            vec!["real".to_string()],
            "a nested cfg_attr without a path meta is not a remap",
        );
        // `path` in the PREDICATE position (first meta) is a cfg key, not an applied `path` attr —
        // must not be mistaken for a remap (would drop a normal module = inverse false negative).
        // Mirrors hunyi's `skip(1)`, keeping the two dimensions in agreement.
        assert_eq!(
            declared_modules("#[cfg_attr(path = \"x\", deprecated)]\npub mod real;\n"),
            vec!["real".to_string()],
            "a `path` cfg predicate key is not an applied path remap",
        );
    }

    #[test]
    fn a_cfg_attr_nested_path_on_an_inline_module_does_not_drop_it() {
        // As with a direct #[path], a cfg_attr(path) on an INLINE module is a rustc no-op, so the
        // module stays declared.
        assert_eq!(
            declared_modules(
                "#[cfg_attr(windows, path = \"x.rs\")]\npub mod a { pub mod inner; }\n"
            ),
            vec!["a".to_string()],
        );
    }

    #[test]
    fn a_path_attr_on_an_inline_module_does_not_drop_it() {
        // `#[path]` remaps only a FILE `mod name;` (out of scope); on
        // an INLINE `mod name { … }` it is a no-op for rustc (the body IS the module), so the module
        // must stay declared — dropping it would leave a compiled module unobserved.
        assert_eq!(
            declared_modules("#[path = \"x.rs\"]\npub mod a { pub mod inner; }\n"),
            vec!["a".to_string()],
            "an inline module with a (no-op) #[path] is still declared",
        );
        // Control: on a FILE mod, #[path] still puts it out of scope.
        assert!(
            declared_modules("#[path = \"x.rs\"]\npub mod a;\n").is_empty(),
            "a #[path]-remapped FILE module is out of scope",
        );
    }

    #[test]
    fn a_block_comment_before_a_mod_name_does_not_fuse_it() {
        // `mod/*c*/foo;` must not strip to `modfoo;` (which drops the
        // declaration); a block comment leaves a separator.
        assert_eq!(
            declared_modules("mod/*c*/foo;"),
            vec!["foo".to_string()],
            "a block comment after `mod` must not swallow the declaration",
        );
    }

    #[test]
    fn a_custom_crate_root_filename_maps_to_crate() {
        // A crate whose target root is a custom filename
        // (`[lib] path = "src/core.rs"`) must still have its submodules reachable. The root file's
        // relative path is passed as root_relative so it maps to `crate` (not `crate::core`).
        let dir = std::env::temp_dir().join(format!("guibiao-customroot-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let src = dir.join("src");
        std::fs::create_dir_all(&src).expect("mkdir src");
        std::fs::write(src.join("core.rs"), "pub mod sub;\n").expect("write core.rs");
        std::fs::write(src.join("sub.rs"), "// sub\n").expect("write sub.rs");
        let files = rust_files(&src).expect("list files");
        let (with_root, _) =
            reachable_modules(&src, &files, Some(std::path::Path::new("core.rs"))).expect("walk");
        let (without_root, _) = reachable_modules(&src, &files, None).expect("walk");
        let _ = std::fs::remove_dir_all(&dir);
        assert!(
            with_root.contains("crate::sub"),
            "with the custom root mapped to crate, its submodule is reachable: {with_root:?}"
        );
        assert!(
            !without_root.contains("crate::sub"),
            "without the root override, core.rs maps to crate::core and sub is unreachable: {without_root:?}"
        );
    }

    #[test]
    fn declared_modules_ignores_a_mod_inside_a_macro_invocation() {
        // A `mod` written inside a macro body is macro-generated and out of scope — the
        // same rule the `use` scanner already applies. `()`/`[]`-delimited invocations
        // were the gap (a `macro_rules!` body is already excluded by brace depth).
        assert!(declared_modules("some_macro!( mod ghost; );").is_empty());
        assert!(declared_modules("some_macro![ mod ghost; ];").is_empty());
        assert!(declared_modules("macro_rules! m { () => { mod ghost; }; }").is_empty());
        // A real top-level declaration is still found.
        assert_eq!(declared_modules("mod real;"), vec!["real".to_string()]);
    }
}
