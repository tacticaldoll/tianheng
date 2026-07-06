//! The source scanner: the functional core's observation source for module
//! boundaries. Given a crate's `src/`, it lists `.rs` files, walks the `mod`-declared
//! module graph reachable from the crate root, and extracts the `crate::…` module
//! paths a file imports via `use` — comments, string literals, and macro bodies
//! stripped, so only real, file-based, reachable imports are observed (PROJECT.md).
//! Pure string and path processing: it depends on no model type, only `std`. Its lexical
//! hygiene (comment/string/macro stripping and token-boundary primitives) lives in the
//! [`lexer`] submodule.

use std::path::{Path, PathBuf};

mod lexer;
use lexer::{is_ident_byte, keyword_starts_at, strip_comments_and_strings, strip_macro_bodies};

/// Internal module paths imported by `source`, normalized to absolute `crate::…`
/// form. `current_module` is the importing file's module; a `use` inside an inline
/// `mod name { … }` is attributed to that submodule, so `self`/`super` resolve against
/// the real enclosing module, not the file's. Only `use` declarations are observed;
/// grouped and glob forms are expanded; raw identifiers (`r#name`) are canonicalized;
/// paths whose first segment is an external crate are ignored. Bare path expressions
/// and macro-generated imports are out of scope (PROJECT.md): comments and string
/// literals are stripped, and macro bodies are removed, so a `use` written inside one
/// is a macro-generated import and is not observed. Returns sorted, de-duplicated paths.
pub(crate) fn imported_module_paths(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> Vec<String> {
    let mut paths: Vec<String> = imports_with_importers(source, current_module, root_modules)
        .into_iter()
        .map(|(_importer, import)| import)
        .collect();
    paths.sort();
    paths.dedup();
    paths
}

/// Each internal import paired with the **module that actually declares it** — inline-aware, so a
/// `use` inside an inline `mod inner { … }` is attributed to `{current_module}::inner`, not the
/// file's module. The inbound rules (`MustNotBeImportedBy` / `MustOnlyBeImportedBy`) test the
/// *importer's* identity, so they need this pair; [`imported_module_paths`] keeps only the absolute
/// import path (right for the outbound rules, which test the import), discarding the importer an
/// inbound rule would otherwise mis-attribute to the file's module. Sorted + deduped by
/// `(importer, import)`.
pub(crate) fn imports_with_importers(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> Vec<(String, String)> {
    let cleaned = strip_macro_bodies(&strip_comments_and_strings(source));
    let mut pairs = Vec::new();
    for (module, tree) in use_trees_with_modules(&cleaned, current_module) {
        for leaf in expand_use_tree(&tree) {
            if let Some(absolute) = normalize_module_path(&leaf, &module, root_modules) {
                pairs.push((module.clone(), absolute));
            }
        }
    }
    pairs.sort();
    pairs.dedup();
    pairs
}

/// Each `use … ;` statement paired with the module that lexically encloses it. The
/// walk tracks inline `mod name { … }` nesting by brace depth, so a `use` inside an
/// inline submodule is attributed to that submodule (e.g. `crate::a::inner`) rather
/// than the file's module (`crate::a`); `self`/`super` then resolve against the real
/// enclosing module, and a bare first segment inside an inline submodule is external
/// even when the file is the crate root. A `mod name;` with no inline body encloses
/// nothing. The text is already comment/string/macro-stripped, so every brace is
/// structural; a `use … ;` is consumed whole, so its own group braces
/// (`use a::{b, c};`) never perturb the depth.
fn use_trees_with_modules(source: &str, base_module: &str) -> Vec<(String, String)> {
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    // (inline module name, brace depth at which its body opened).
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    let mut depth = 0usize;
    let mut i = 0;
    while i < bytes.len() {
        if keyword_starts_at(bytes, i, b"use") {
            let start = i + 3;
            // A precise-capturing bound `-> impl Trait + use<'a, T>` (stable Rust) puts a `use`
            // token inside a type bound: it is followed by `<`, whereas a `use` *statement* is
            // always followed by a path (ident / `{` / `*` / `::` / `crate`/`self`/`super`).
            // So a `<` here means this is a bound, not an import — skip past the `use` token and
            // continue (letting the `<…>` be walked as ordinary bytes) rather than scanning to the
            // next `;`, which would swallow the following real `use` (a false negative). A comment
            // between `use` and `<` is already removed by the upstream comment/string strip.
            let mut p = start;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            if bytes.get(p) == Some(&b'<') {
                i = start;
                continue;
            }
            match source[start..].find(';') {
                Some(rel) => {
                    trees.push((
                        effective_module(base_module, &mod_stack),
                        source[start..start + rel].trim().to_string(),
                    ));
                    i = start + rel + 1;
                    continue;
                }
                None => break,
            }
        }
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            mod_stack.push((
                canonical_segment(source[name_start..name_end].trim()).to_string(),
                depth,
            ));
            i = brace; // let the `{` arm below increment the depth
            continue;
        }
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth = depth.saturating_sub(1);
                while mod_stack.last().is_some_and(|(_, d)| *d == depth) {
                    mod_stack.pop();
                }
            }
            _ => {}
        }
        i += 1;
    }
    trees
}

/// The module path enclosing a `use`, formed from the file's `base` module and the
/// names of the inline `mod`s currently open around it.
fn effective_module(base: &str, mod_stack: &[(String, usize)]) -> String {
    let mut module = base.to_string();
    for (name, _) in mod_stack {
        module.push_str("::");
        module.push_str(name);
    }
    module
}

/// If an inline module declaration `mod <ident> {` begins at `i` (a standalone `mod`
/// keyword whose name is followed, after optional whitespace, by `{`), return
/// `(name_start, name_end, index_of_opening_brace)`; otherwise `None` — a `mod name;`
/// with no body, or not a declaration. Only an inline body encloses a `use`.
fn inline_mod_at(bytes: &[u8], i: usize) -> Option<(usize, usize, usize)> {
    if !is_mod_declaration_keyword(bytes, i) {
        return None;
    }
    let mut j = i + 3;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    let name_start = j;
    while j < bytes.len() && !bytes[j].is_ascii_whitespace() && bytes[j] != b';' && bytes[j] != b'{'
    {
        j += 1;
    }
    let name_end = j;
    if name_end == name_start {
        return None;
    }
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    if bytes.get(j) == Some(&b'{') {
        Some((name_start, name_end, j))
    } else {
        None
    }
}

/// Canonicalize one path segment by stripping a leading raw-identifier marker
/// (`r#name` -> `name`). Rust resolves `mod r#type;` to the source file `type.rs`,
/// so the file-derived path, the `mod` declaration, and a `use r#type::…` path must
/// all reduce to the same module identity; this is the single place that reduction
/// lives. A segment with no `r#` prefix is returned unchanged.
fn canonical_segment(segment: &str) -> &str {
    segment.strip_prefix("r#").unwrap_or(segment)
}

/// Canonicalize a whole `::`-joined module path segment-by-segment (see
/// [`canonical_segment`]), so a boundary's declared path and an observed path compare
/// in one vocabulary regardless of which uses the raw-identifier form.
pub(crate) fn canonical_module_path(path: &str) -> String {
    path.split("::")
        .map(canonical_segment)
        .collect::<Vec<_>>()
        .join("::")
}

/// Expand a use tree into leaf paths: `a::{b, c::d}` -> `a::b`, `a::c::d`; drop
/// `::*` and ` as alias`; `{self}` resolves to the prefix module.
fn expand_use_tree(tree: &str) -> Vec<String> {
    expand_use_tree_depth(tree, 0)
}

/// A brace-nesting depth cap so a pathologically nested `use a::{b::{c::{ … }}}` cannot overflow the
/// stack — a DoS backstop set far beyond any real or lint-clean source (rustfmt-formatted `use`s
/// nest a handful of levels). Past the cap the sub-tree is not expanded; this is the only place the
/// scanner bounds coverage, so it is logged as a stated limit, not a silent hole for real code.
const MAX_USE_NEST_DEPTH: usize = 128;

fn expand_use_tree_depth(tree: &str, depth: usize) -> Vec<String> {
    if depth >= MAX_USE_NEST_DEPTH {
        return Vec::new();
    }
    let tree = tree.trim();
    match tree.find('{') {
        Some(open) => {
            let prefix = tree[..open].trim();
            let inner = brace_content(&tree[open..]);
            let mut out = Vec::new();
            for part in split_top_commas(&inner) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if part == "self" {
                    let module = prefix.trim_end_matches(':').trim();
                    if !module.is_empty() {
                        out.push(module.to_string());
                    }
                } else {
                    out.extend(expand_use_tree_depth(&format!("{prefix}{part}"), depth + 1));
                }
            }
            out
        }
        None => {
            let leaf = match tree.find(" as ") {
                Some(idx) => &tree[..idx],
                None => tree,
            };
            let leaf = leaf.trim().strip_suffix("::*").unwrap_or(leaf.trim());
            let leaf = leaf.trim_end_matches(':');
            if leaf.is_empty() {
                Vec::new()
            } else {
                vec![leaf.to_string()]
            }
        }
    }
}

/// Content inside the first `{ … }` of `s` (which must start with `{`), honoring
/// nesting.
fn brace_content(s: &str) -> String {
    let mut depth = 0;
    let mut out = String::new();
    for ch in s.chars() {
        match ch {
            '{' => {
                depth += 1;
                if depth == 1 {
                    continue;
                }
            }
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        out.push(ch);
    }
    out
}

/// Split on commas at brace depth 0.
fn split_top_commas(s: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut depth = 0;
    let mut current = String::new();
    for ch in s.chars() {
        match ch {
            '{' => {
                depth += 1;
                current.push(ch);
            }
            '}' => {
                depth -= 1;
                current.push(ch);
            }
            ',' if depth == 0 => parts.push(std::mem::take(&mut current)),
            _ => current.push(ch),
        }
    }
    if !current.trim().is_empty() {
        parts.push(current);
    }
    parts
}

/// Resolve a use path to an absolute `crate::…` module path, or `None` if it refers
/// to an external crate. A first segment of `crate`/`self`/`super` resolves as usual.
/// A first segment that names a crate-root module (`root_modules`) resolves to
/// `crate::…` **only when the importing file is the crate root** (`current_module ==
/// "crate"`): there a sibling `mod` is in scope and shadows the extern prelude, so a
/// bare `use foo::…` is the local module. In a submodule a bare first segment reaches
/// only the extern prelude (a bare crate-root-module path there is either an external
/// crate or a compile error), so it is external. A path written with a leading `::`
/// (`use ::foo::…`) is explicitly the external/global crate and is always external. Any
/// other first segment is external (`None`).
fn normalize_module_path(
    path: &str,
    current_module: &str,
    root_modules: &[String],
) -> Option<String> {
    // A leading `::` is an explicit external/global path (`use ::serde::…`): it bypasses
    // the local-module shadow, so it is external even if a crate-root module shares the
    // name. Checked before segments are split so the marker is not lost.
    if path.trim_start().starts_with("::") {
        return None;
    }
    let segments: Vec<&str> = path
        .split("::")
        .map(|segment| canonical_segment(segment.trim()))
        .filter(|segment| !segment.is_empty())
        .collect();
    let (first, rest) = segments.split_first()?;
    match *first {
        "crate" => Some(segments.join("::")),
        "self" => {
            let mut out: Vec<&str> = current_module
                .split("::")
                .filter(|s| !s.is_empty())
                .collect();
            out.extend(rest);
            Some(out.join("::"))
        }
        "super" => {
            let mut out: Vec<&str> = current_module
                .split("::")
                .filter(|s| !s.is_empty())
                .collect();
            let mut tail = &segments[..];
            while let Some((segment, next)) = tail.split_first() {
                if *segment != "super" {
                    break;
                }
                out.pop();
                tail = next;
            }
            // An over-popped `super` (more `super`s than ancestors) drops the `crate`
            // root, leaving a path that is not crate-rooted — it names no internal module
            // (and the source does not compile). Return `None` rather than a malformed
            // root-less path, which would otherwise be mistaken for an outward edge.
            if out.first() != Some(&"crate") {
                return None;
            }
            out.extend(tail.iter().copied());
            Some(out.join("::"))
        }
        other => {
            // A bare first segment names a crate-root module only at the crate root,
            // where a sibling `mod` is in scope and shadows the extern prelude — there a
            // bare `use foo::…` is the local module, so resolve to `crate::…`. In a
            // submodule the same bare path reaches only the extern prelude (it is an
            // external crate, or a compile error), so it is external. Gating on
            // `current_module == "crate"` keeps a submodule's `use serde::…` external
            // even when a `mod serde;` exists at the crate root. (Explicit external is
            // written `::name`, handled above.)
            if current_module == "crate" && root_modules.iter().any(|module| module == other) {
                let mut out = vec!["crate"];
                out.extend(segments.iter().copied());
                Some(out.join("::"))
            } else {
                None
            }
        }
    }
}

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
) -> Vec<(PathBuf, String)> {
    files
        .iter()
        .filter_map(|file| {
            let relative = file.strip_prefix(src_dir).ok()?;
            let module_path = file_module_path(relative);
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

/// Sibling-safe `::`-delimited path containment: `path` is `prefix` itself or lies strictly
/// beneath it (`crate::a` contains `crate::a::b`, never the prefix-colliding sibling
/// `crate::ab`). The single home of the containment rule every module boundary's inbound /
/// outbound predicate and the file selector share, so no copy can drift to a bare
/// `starts_with` — which would admit a sibling (a false positive on the allowed side) or,
/// inverted, miss a subtree (a false negative on the forbidden side). The 圭表 twin of 渾儀's
/// `path_within`; the two dimensions cannot share code (三儀 ⊥ 三儀), so they agree by using the
/// same rule, not the same function.
pub(crate) fn path_within(path: &str, prefix: &str) -> bool {
    path == prefix || path.starts_with(&format!("{prefix}::"))
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
                .entry(file_module_path(relative))
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
                if has_path_attr_before_item(bytes, i) {
                    i += 3;
                    continue;
                }
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
                    // anything else is a stray `mod` token, not a declaration.
                    match bytes.get(k) {
                        Some(b'{') => names.push((canonical_segment(ident).to_string(), true)),
                        Some(b';') => names.push((canonical_segment(ident).to_string(), false)),
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
fn declared_modules(source: &str) -> Vec<String> {
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
    }
    false
}

/// Whether a standalone `mod` keyword begins at `i` (bounded by non-identifier bytes) —
/// the head of a possible module declaration, not a substring like `module`.
fn is_mod_declaration_keyword(bytes: &[u8], i: usize) -> bool {
    keyword_starts_at(bytes, i, b"mod")
}

/// The module path of a source file from its path relative to `src/`:
/// `lib.rs`/`main.rs`/`mod.rs` contribute no segment; `kernel/foo.rs` ->
/// `crate::kernel::foo`.
fn file_module_path(relative: &Path) -> String {
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
            // the crate root — they are the cargo *target* roots there, not module names. A declared
            // submodule file literally named `lib.rs`/`main.rs` (`mod lib;` inside a subdir →
            // `foo/lib.rs` = `crate::foo::lib`) contributes its stem like any other file; stripping
            // it at depth would mis-map it to its parent and drift from 渾儀's declaration-driven
            // descent (which resolves it correctly).
            let segmentless =
                stem == "mod" || (components.len() == 1 && matches!(stem, "lib" | "main"));
            if !segmentless {
                segments.push(canonical_segment(stem).to_string());
            }
        } else {
            segments.push(canonical_segment(component).to_string());
        }
    }
    segments.join("::")
}

/// All `.rs` files under `dir`, recursively. A directory that cannot be read (or an
/// entry that cannot be resolved) is a scan error, never a silent skip: a skipped
/// subtree could hide a real module-boundary violation — "cannot judge", not "nothing
/// to judge", the same rule as an unreadable governed file.
pub(crate) fn rust_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
    let mut found = Vec::new();
    let entries = std::fs::read_dir(dir).map_err(|err| {
        format!(
            "cannot read governed source directory '{}': {err}",
            dir.display()
        )
    })?;
    for entry in entries {
        let entry = entry.map_err(|err| {
            format!(
                "cannot read an entry in governed source directory '{}': {err}",
                dir.display()
            )
        })?;
        // Recurse only into a real directory: `file_type()` does NOT follow symlinks (unlike
        // `path.is_dir()`, which stats the target), so a symlinked directory (a cyclic
        // `src/loop -> .`) is not entered — avoiding an unbounded recursion → stack overflow.
        // Matches louke's probe scanner, which guards the same hazard the same way.
        let file_type = entry.file_type().map_err(|err| {
            format!(
                "cannot stat an entry in governed source directory '{}': {err}",
                dir.display()
            )
        })?;
        let path = entry.path();
        if file_type.is_dir() {
            found.extend(rust_files(&path)?);
        } else if path.extension().and_then(|ext| ext.to_str()) == Some("rs") {
            // A `.rs` file — including a symlink whose target is a real file: rustc compiles such a
            // `mod`-declared source (and `read_to_string` follows the symlink to it), so it is
            // governed. Not gated on `file_type.is_file()`, which would drop a symlinked source and
            // silently miss its imports; a symlinked *directory* is already excluded above.
            found.push(path);
        }
    }
    // Sort so the governed-file order — and hence module-violation order in the report —
    // is deterministic, independent of the filesystem's `read_dir` order.
    found.sort();
    Ok(found)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanner_expands_groups_and_resolves_relative_imports() {
        let source = r#"
            // a line comment mentioning use crate::ignored::me;
            use crate::a::{b, c::d};
            use super::sibling::X;
            use self::inner::Y;
            use serde::Deserialize;
            use crate::z::*;
        "#;
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(imports.contains(&"crate::a::b".to_string()), "{imports:?}");
        assert!(
            imports.contains(&"crate::a::c::d".to_string()),
            "{imports:?}"
        );
        // `super` from crate::kernel resolves to crate.
        assert!(
            imports.contains(&"crate::sibling::X".to_string()),
            "{imports:?}"
        );
        // `self` resolves against the current module.
        assert!(
            imports.contains(&"crate::kernel::inner::Y".to_string()),
            "{imports:?}"
        );
        // glob keeps the module prefix.
        assert!(imports.contains(&"crate::z".to_string()), "{imports:?}");
        // external first segment is ignored, and commented-out imports are not seen.
        assert!(!imports.iter().any(|p| p.contains("serde")), "{imports:?}");
        assert!(
            !imports.iter().any(|p| p.contains("ignored")),
            "{imports:?}"
        );
    }

    #[test]
    fn super_past_the_crate_root_is_not_an_internal_module() {
        // `crate::a` has one ancestor (`crate`); `super::super` over-pops past the root.
        // Such a path names no internal module (and does not compile), so it must not be
        // observed — never a malformed root-less path like "other::X".
        let over = "use super::super::other::X;\n";
        assert!(
            imported_module_paths(over, "crate::a", &[]).is_empty(),
            "over-popped super must yield no import: {:?}",
            imported_module_paths(over, "crate::a", &[])
        );
        // A single `super` from `crate::a` still resolves to `crate::other::X`.
        let ok = "use super::other::X;\n";
        assert_eq!(
            imported_module_paths(ok, "crate::a", &[]),
            vec!["crate::other::X".to_string()],
        );
    }

    #[test]
    fn scanner_ignores_comments_and_string_literals() {
        // A `//` inside a string must not eat a real `use` later on the same line.
        let url = r#"let u = "http://example.com"; use crate::real::A;"#;
        let imports = imported_module_paths(url, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::A".to_string()),
            "{imports:?}"
        );

        // A `use …;` written inside a string is not a real import.
        let in_string = r#"let s = "use crate::ghost::Z;";"#;
        assert!(
            imported_module_paths(in_string, "crate::kernel", &[]).is_empty(),
            "a use inside a string must not be observed"
        );

        // A quote-bearing char literal must not open a spurious string and swallow code.
        let quote_char = r#"let q = '"'; use crate::real::B;"#;
        let imports = imported_module_paths(quote_char, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::B".to_string()),
            "{imports:?}"
        );

        // A lifetime must not break use detection or produce a spurious path.
        let lifetime = "fn f<'a>(x: &'a str) {} use crate::a::b;";
        let imports = imported_module_paths(lifetime, "crate::kernel", &[]);
        assert_eq!(imports, vec!["crate::a::b".to_string()], "{imports:?}");
    }

    #[test]
    fn scanner_handles_nested_block_comments() {
        // Rust nests block comments. Commenting out code that itself contains a
        // `/* */` must not let the inner `*/` re-expose the rest as live code: a
        // `use` inside the (nested) comment must not be observed, while a real `use`
        // after the outer close still is.
        let source = r#"
            /*
            fn old() {
                /* tweak later */
                use crate::legacy::Thing;
            }
            */
            use crate::current::A;
        "#;
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::current::A".to_string()),
            "the real use after the nested comment must be observed: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("legacy")),
            "a use inside a nested block comment must not be observed: {imports:?}"
        );
    }

    #[test]
    fn scanner_resolves_root_relative_bare_use() {
        // A root-relative bare `use kernel::…` (legal only at the crate root) names a
        // crate-root module, not an external crate, so it resolves to `crate::kernel::…`.
        // An unknown first segment is still external. With no root modules known, the
        // bare path stays external (the conservative pre-fix behavior).
        let source = "use kernel::Thing; use serde::Deserialize;";
        let roots = vec!["kernel".to_string()];
        let imports = imported_module_paths(source, "crate", &roots);
        assert!(
            imports.contains(&"crate::kernel::Thing".to_string()),
            "a bare use of a crate-root module must resolve to crate::…: {imports:?}"
        );
        assert!(
            !imports.iter().any(|p| p.contains("serde")),
            "an unknown first segment stays external: {imports:?}"
        );
        assert!(
            imported_module_paths(source, "crate", &[])
                .iter()
                .all(|p| !p.contains("kernel")),
            "with no root modules known, the bare path is treated as external"
        );
    }

    #[test]
    fn scanner_treats_leading_colon_path_as_external() {
        // `use ::serde::…` is an explicit external/global path: the leading `::` bypasses
        // the local-module shadow, so it is external even when a crate-root module shares
        // the name. (Before the fix the leading `::` was dropped and the path was
        // mis-resolved as the internal `crate::serde::…`.)
        let roots = vec!["serde".to_string()];
        assert!(
            imported_module_paths("use ::serde::Deserialize;", "crate", &roots).is_empty(),
            "a leading-:: path must be external even when its head matches a root module"
        );
        // Sanity: without the leading `::`, the same head IS the local module, because a
        // crate-root module shadows the extern prelude in a bare `use` path.
        assert!(
            imported_module_paths("use serde::Deserialize;", "crate", &roots)
                .contains(&"crate::serde::Deserialize".to_string()),
            "a bare head matching a root module resolves locally (shadowing rule)"
        );
    }

    #[test]
    fn scanner_resolves_root_relative_use_only_at_crate_root() {
        // The crate-root-module shadow holds ONLY at the crate root. In a submodule a
        // bare `use serde::…` reaches the extern prelude (external), even when `serde`
        // is a crate-root module — matching the compiler. Resolving it to
        // `crate::serde::…` would be a false positive that fails a module boundary.
        let roots = vec!["serde".to_string()];
        assert!(
            imported_module_paths("use serde::Value;", "crate", &roots)
                .contains(&"crate::serde::Value".to_string()),
            "at the crate root, a bare crate-root-module path resolves locally"
        );
        assert!(
            imported_module_paths("use serde::Value;", "crate::sub", &roots).is_empty(),
            "in a submodule, a bare first segment is external even if it matches a root module"
        );
    }

    #[test]
    fn scanner_preserves_non_ascii_module_paths() {
        // strip is UTF-8 safe: a non-ASCII module path survives stripping intact.
        let source = "use crate::café::Item;";
        let imports = imported_module_paths(source, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::café::Item".to_string()),
            "{imports:?}"
        );
    }

    #[test]
    fn scanner_ignores_raw_and_byte_strings() {
        // A `use …;` inside a raw string (any hash count) is not an import.
        for src in [
            r##"let s = r"use crate::ghost::Z;";"##,
            r##"let s = r#"use crate::ghost::Z;"#;"##,
            r##"let s = br#"use crate::ghost::Z;"#;"##,
            r#"let s = b"use crate::ghost::Z;";"#,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel", &[]).is_empty(),
                "a use inside a (raw/byte) string must not be observed: {src}"
            );
        }

        // A `//` and an inner `"#` inside a raw string must not eat a following use.
        // (Two outer hashes so the inner `"#` does not close it.)
        let tricky = r####"let s = r##"http://x "# inside"##; use crate::real::C;"####;
        let imports = imported_module_paths(tricky, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::C".to_string()),
            "{imports:?}"
        );

        // `r` / `b` as ordinary identifiers (not raw-string prefixes) are unaffected.
        let idents = "let r = 1; let b = 2; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel", &[]),
            vec!["crate::real::D".to_string()]
        );
    }

    #[test]
    fn scanner_ignores_raw_c_strings() {
        // A `use …;` inside a raw C-string (`cr"…"` / `cr#"…"#`, stable Rust 1.79) is not observed.
        for src in [
            r##"let s = cr"use crate::ghost::Z;";"##,
            r##"let s = cr#"use crate::ghost::Z;"#;"##,
        ] {
            assert!(
                imported_module_paths(src, "crate::kernel", &[]).is_empty(),
                "a use inside a raw C-string must not be observed: {src}"
            );
        }

        // The desync guard: a raw C-string with an **odd** number of inner unescaped `"` (raw
        // strings do not escape) must not swallow a following `use`. Before the `cr#` prefix was
        // recognized, the leading `c` made `raw_string_prefix` decline (and the following `r` sat
        // after an identifier byte, so it declined too), so `#"a"b"#` was scanned as code + plain
        // strings: `"a"` closed as one plain string, then the `"` before `#` opened an unterminated
        // one that ran to EOF, silently dropping the import (a false negative in FFI-flavoured
        // code). An odd inner-quote count is required to leave that final `"` unpaired; an even
        // count re-pairs into balanced plain strings and does not desync — so the test uses an odd
        // count (one inner `"` here) to actually fail on unfixed code, not merely pass on it.
        let tricky = r##"let s = cr#"a"b"#; use crate::real::C;"##;
        let imports = imported_module_paths(tricky, "crate::kernel", &[]);
        assert!(
            imports.contains(&"crate::real::C".to_string()),
            "a use after a raw C-string with an odd inner-quote count must be observed: {imports:?}"
        );

        // A non-raw `c"…"` C-string (ordinary escaping) also hides an inner use and does not
        // desync — handled by the plain-string branch, the `c` prefix byte emitted as code.
        let cstr = r#"let s = c"use crate::ghost::Z;"; use crate::real::E;"#;
        assert_eq!(
            imported_module_paths(cstr, "crate::kernel", &[]),
            vec!["crate::real::E".to_string()]
        );

        // `c` as an ordinary identifier (not a C-string prefix) is unaffected.
        let idents = "let c = 1; use crate::real::D;";
        assert_eq!(
            imported_module_paths(idents, "crate::kernel", &[]),
            vec!["crate::real::D".to_string()]
        );
    }

    #[test]
    fn scanner_does_not_panic_on_odd_input() {
        // Truncated / malformed inputs must never panic (robustness over precision).
        for src in [
            "r#\"unterminated raw string",
            "\"unterminated string",
            "/* unterminated block",
            "'",
            "r",
            "use ",
            "use crate::",
            "mod ",
            "mod foo",
            "}",
            "",
        ] {
            let _ = imported_module_paths(src, "crate::kernel", &[]);
            let _ = declared_modules(src);
            let _ = strip_macro_bodies(src);
        }
    }

    #[test]
    fn use_inside_a_macro_body_is_not_observed() {
        // A `use` written inside a macro — a `macro_rules!` definition OR a macro
        // invocation — is a macro-generated import (out of scope): it must not be
        // observed. A real `use` outside the macro still is.
        let source = r#"
            macro_rules! m {
                () => { use crate::ghost::Thing; };
                ($x:tt) => {{ use crate::ghost::Other; }};
            }
            with_imports! { use crate::ghost::FromInvocation; }
            use crate::real::A;
        "#;
        let imports = imported_module_paths(source, "crate", &[]);
        assert_eq!(
            imports,
            vec!["crate::real::A".to_string()],
            "macro definition and invocation bodies are skipped; the real use is kept: {imports:?}"
        );
        // Every body delimiter form ({}, (), []) is skipped, for both definitions and
        // invocations.
        for body in [
            "macro_rules! m { () => { use crate::ghost::T; }; }",
            "macro_rules! m ( () => { use crate::ghost::T; }; )",
            "macro_rules! m [ () => { use crate::ghost::T; }; ]",
            "some_macro! { use crate::ghost::T; }",
            "some_macro!( use crate::ghost::T; )",
            "some_macro![ use crate::ghost::T; ]",
        ] {
            assert!(
                imported_module_paths(body, "crate", &[]).is_empty(),
                "no import observed from a macro body: {body}"
            );
        }
        // `!=` and unary `!` are not macro invocations — a following real use is kept.
        let not_macros = "let _ = a != b; let _ = !flag; use crate::real::B;";
        assert!(
            imported_module_paths(not_macros, "crate", &[]).contains(&"crate::real::B".to_string()),
            "`!=` / unary `!` must not be treated as a macro invocation"
        );
    }

    #[test]
    fn a_raw_identifier_macro_name_does_not_leak_its_body() {
        // A `macro_rules!` with a raw-identifier name (`r#try`): `#` is not an identifier byte, so
        // the name scan must tolerate the `r#` prefix — otherwise it stops at `r`, fails to locate
        // the body, leaves it unstripped, and wrongly observes the `use`/`mod` inside the
        // never-invoked definition (a false positive). Both scans share the macro-body stripping.
        let with_use = r#"
            macro_rules! r#try {
                () => { use crate::ghost::Thing; };
            }
            use crate::real::A;
        "#;
        assert_eq!(
            imported_module_paths(with_use, "crate", &[]),
            vec!["crate::real::A".to_string()],
            "a `use` inside a raw-identifier macro definition is not observed"
        );
        let with_mod = "macro_rules! r#try { () => { mod ghost; }; }\nmod real;";
        assert_eq!(
            declared_modules(with_mod),
            vec!["real".to_string()],
            "a `mod` inside a raw-identifier macro definition is not a declared module"
        );
    }

    #[test]
    fn keyword_detection_does_not_fire_inside_a_unicode_identifier() {
        // Rust allows non-ASCII identifiers. `use貓` / `mod貓` are single identifiers, so
        // the leading `use` / `mod` is NOT a keyword: the byte after it (the lead byte of
        // `貓`, >= 0x80) is an identifier byte. A regression guard for ASCII-only
        // `is_ident_byte`, which used to split the identifier and fire a false keyword.
        assert!(!keyword_starts_at("use貓;".as_bytes(), 0, b"use"));
        assert!(!keyword_starts_at("mod貓 {}".as_bytes(), 0, b"mod"));
        // The genuine keyword (followed by whitespace) is still detected.
        assert!(keyword_starts_at("use 貓;".as_bytes(), 0, b"use"));
        // And nothing is observed as an import or a declared module from the identifier.
        assert!(imported_module_paths("fn use貓() {}", "crate", &[]).is_empty());
        assert!(declared_modules("fn mod貓() {}").is_empty());
    }

    #[test]
    fn a_precise_capturing_use_bound_does_not_swallow_the_following_use() {
        // `-> impl Trait + use<…>` (stable Rust) is a precise-capturing bound, not an import.
        // The scanner must not treat it as a `use` statement and consume to the next `;`, which
        // would swallow the real `use` that follows — a false negative that silently disables the
        // module boundary. Cover the empty, parameterized, whitespace, and comment forms.
        for header in [
            "fn iter() -> impl Iterator<Item = u8> + use<> { std::iter::empty() }",
            "fn iter<'a, T>() -> impl Iterator<Item = &'a T> + use<'a, T> { loop {} }",
            "fn iter() -> impl Iterator<Item = u8> + use <> { std::iter::empty() }",
            "fn iter() -> impl Iterator<Item = u8> + use /*c*/ <> { std::iter::empty() }",
        ] {
            let src = format!("{header}\nuse crate::forbidden::Thing;");
            assert_eq!(
                imported_module_paths(&src, "crate", &[]),
                vec!["crate::forbidden::Thing".to_string()],
                "the `use<…>` bound must be skipped so the following real use is observed: {header:?}"
            );
        }
    }

    #[test]
    fn a_use_bound_as_the_last_token_neither_panics_nor_drops_a_preceding_use() {
        // Control: a plain `use` is unaffected. And a `use<>` bound as the file's final token
        // (no trailing `;`) must not panic (bounds-safe peek) and must not drop the real `use`
        // that precedes it.
        assert_eq!(
            imported_module_paths("use crate::x::Y;", "crate", &[]),
            vec!["crate::x::Y".to_string()],
            "a plain use is unaffected by the bound-skip"
        );
        assert_eq!(
            imported_module_paths(
                "use crate::x::Y;\nfn f() -> impl Sized + use<>",
                "crate",
                &[],
            ),
            vec!["crate::x::Y".to_string()],
            "a trailing use<> bound must not drop the preceding real use or panic"
        );
    }

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
        let (reachable, _inline_only) = reachable_modules(&src, &files).expect("walk modules");
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
        let (reachable, _inline_only) = reachable_modules(&src, &files).expect("walk modules");
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
    fn raw_identifiers_are_canonicalized() {
        // `mod r#type;` compiles to `type.rs`, so the `mod` token, the `use` path, and
        // the file path must all reduce to the same identity (`type`).
        assert_eq!(canonical_segment("r#type"), "type");
        assert_eq!(canonical_segment("type"), "type");
        assert_eq!(
            canonical_module_path("crate::r#type::r#mod"),
            "crate::type::mod"
        );
        assert_eq!(
            declared_modules("pub mod r#type;"),
            vec!["type".to_string()]
        );
        assert_eq!(
            imported_module_paths("use crate::r#type::Thing;", "crate", &[]),
            vec!["crate::type::Thing".to_string()],
            "a raw-identifier use path is canonicalized to its plain form"
        );
    }

    #[test]
    fn use_inside_an_inline_module_is_attributed_to_that_module() {
        // A `self`/`super` import inside an inline `mod inner { … }` resolves against the
        // inline submodule, not the file's module: `self` -> crate::a::inner::…, and
        // `super` from crate::a::inner -> crate::a.
        let source = "mod inner { use self::leaf::Thing; use super::sibling::X; }";
        let imports = imported_module_paths(source, "crate::a", &[]);
        assert!(
            imports.contains(&"crate::a::inner::leaf::Thing".to_string()),
            "self must resolve against the inline submodule: {imports:?}"
        );
        assert!(
            imports.contains(&"crate::a::sibling::X".to_string()),
            "super from the inline submodule resolves to crate::a: {imports:?}"
        );
        // A bare first segment inside an inline submodule is external even when the file
        // IS the crate root (the enclosing module is crate::inner, not crate).
        let bare = imported_module_paths(
            "mod inner { use kernel::Thing; }",
            "crate",
            &["kernel".to_string()],
        );
        assert!(
            bare.is_empty(),
            "a bare use inside an inline submodule is external: {bare:?}"
        );
        // A top-level use (no inline module) is unchanged.
        assert_eq!(
            imported_module_paths("use self::leaf::Thing;", "crate::a", &[]),
            vec!["crate::a::leaf::Thing".to_string()]
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

    #[test]
    fn escaped_quote_char_literal_is_consumed_whole() {
        // `'\''` must be skipped as a whole so the following string's fake `use` is still
        // stripped and the real `use` after it is observed (a regression guard for the
        // escaped-char skip that used to leak the closing quote).
        let src = r#"let _q = '\''; let _s = "use crate::ghost::Z;"; use crate::real::A;"#;
        assert_eq!(
            imported_module_paths(src, "crate::kernel", &[]),
            vec!["crate::real::A".to_string()],
            "an escaped-quote char literal must not leak and expose a fake use"
        );
    }
}
