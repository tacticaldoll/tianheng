//! The inline-symbol-path scan: the observation source for `ConfineInlineSymbolPath`
//! (`must_not_call_inline`). Unlike the `use`-scan, it observes **call expressions** (and, under
//! strict, any path mention) in function bodies — INCLUDING macro-invocation bodies — resolving a
//! path's head through an alias-carrying use-map, local `type` aliases, and the local `pub use`
//! re-export closure to a fixpoint. A glob that can bring a prefix-resolving name into scope reacts
//! fail-closed. Pure string / path processing over [`super::lexer`] and [`super::path_vocab`]; no
//! model type. The declared stated bounds (receiver-method reads, in-macro-body aliases,
//! fragment/proc-macro construction, external-crate re-exports, value-position captures under the
//! default, and the inherited file-scope scanner bounds) are non-observations, never silent passes.

use std::collections::{HashMap, HashSet};

use crate::finding::ModuleFact;

use super::lexer::{is_ident_byte, strip_comments_and_strings, strip_macro_bodies};
use super::path_vocab::{
    canonical_module_path, canonical_segment, effective_module, inline_mod_at,
    is_crate_root_shadow, path_within, resolve_self_super,
};

/// The crate-wide resolution context, built once from every reachable file: the local definition
/// closure (`type` aliases and `pub use` re-exports, keyed by their fully-qualified local name →
/// target path) and the glob re-exports (a module that `pub use`-globs another path). Per-file
/// `use`-maps are built on demand during the call scan.
struct ResolveCtx {
    /// Fully-qualified local name (`crate::mod::Name`) → its target path (canonicalized in the
    /// defining module's context). Covers `type Name = Target;` and `pub use Target as Name;`
    /// (and `pub use Target;`, whose name is the last segment).
    defs: HashMap<String, String>,
    /// `(module, resolved-glob-path)` for each `pub use <path>::*;` — feeds the recursive
    /// local-module glob-hazard test.
    glob_reexports: Vec<(String, String)>,
}

/// One inline offence: the `finding` string (per the identity requirement) and the source file.
pub(crate) struct InlineFinding {
    pub fact: ModuleFact,
    pub file: String,
}

/// Scan the crate for inline-symbol-path offences against a `ConfineInlineSymbolPath` or
/// `ConfineInlineSymbolPathExternal` boundary (both route here via `inline_payload`).
/// `all_files` is every reachable `(file, module)` pair (crate-wide, for the def closure);
/// `governed` is the subset whose module is within the governed subtree (where calls are
/// forbidden). `prefix` is the confined module-path prefix; `ending_with` narrows to read verbs;
/// `strict` reacts on any mention, not only calls; `external` opts in the strict-external head
/// ladder (a fully-qualified un-`use`d head matching a declared dependency reclassifies as
/// external); `dependency_names` are the rename-aware declared-dependency import identifiers that
/// ladder matches against (unused when `external` is false). Returns findings sorted + deduped by
/// `finding`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn inline_symbol_findings(
    all_files: &[(std::path::PathBuf, String)],
    governed: &[(std::path::PathBuf, String)],
    root_modules: &[String],
    prefix: &str,
    ending_with: Option<&[String]>,
    strict: bool,
    external: bool,
    dependency_names: &[String],
) -> Result<Vec<InlineFinding>, String> {
    let prefix = canonical_module_path(prefix);
    // Verbs are matched leaf-exact on the terminal segment; canonicalize raw-identifier forms.
    let verbs: Option<Vec<String>> =
        ending_with.map(|vs| vs.iter().map(|v| canonical_module_path(v)).collect());

    // Pass 1 — build the crate-wide def / glob-reexport closure from every reachable file.
    let mut ctx = ResolveCtx {
        defs: HashMap::new(),
        glob_reexports: Vec::new(),
    };
    let mut file_text: HashMap<std::path::PathBuf, String> = HashMap::new();
    let mut use_maps: HashMap<std::path::PathBuf, HashMap<String, String>> = HashMap::new();
    // The FULL local vocabulary backing the strict-external local-precedence ladder, built ONLY
    // under the flag: (a) the complete crate module-path set — every reachable `(_, module)`, so a
    // DEEP local `mod` (not just a crate-root child) is visible (rung iii); (b) item-definition
    // names across all namespaces — a local `struct`/`fn`/plain `mod`/… named like a dependency
    // (rung iv). `ctx.defs`/`root_modules` alone cannot back these rungs (defs holds only type-alias
    // + `pub use`; root_modules only crate-root children), the first-cut flaw.
    let mut module_paths: HashSet<String> = HashSet::new();
    let mut item_defs: HashSet<String> = HashSet::new();
    for (file, module) in all_files {
        let raw = std::fs::read_to_string(file)
            .map_err(|err| crate::errors::unreadable_governed_file_error(file, &err.to_string()))?;
        // Declarations (`type` / `pub use`) are read from macro-stripped text: an alias declared
        // inside a macro body is a stated bound (in-macro-body alias), not observed here.
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(&raw));
        // The per-file `use`-map (alias-carrying): head identifier → target path. A `type`-alias
        // or `pub use` target is resolved through it, so `use std::time::SystemTime; type Clock =
        // SystemTime;` chases correctly.
        let use_map = collect_use_map(&decl_text, module, root_modules);
        collect_defs(&decl_text, module, root_modules, &use_map, &mut ctx);
        if external {
            module_paths.insert(module.clone());
            collect_item_definition_names(module, &decl_text, &mut item_defs);
        }
        use_maps.insert(file.clone(), use_map);
        file_text.insert(file.clone(), raw);
    }
    // The declared-dependency import-identifier set (rung v).
    let dep_names: HashSet<String> = if external {
        dependency_names.iter().cloned().collect()
    } else {
        HashSet::new()
    };
    // The head resolver consults this ONLY under `.strict_external()`; `None` keeps head resolution
    // byte-identical to the default path (the local rungs act purely as a guard on whether the
    // dependency match fires, still emitting the load-bearing `{module}::…` fallback otherwise).
    let external_vocab = external.then_some(ExternalVocab {
        module_paths: &module_paths,
        item_defs: &item_defs,
        dep_names: &dep_names,
    });

    // Pass 2 — scan each governed file for offending calls / mentions and hazardous globs.
    let mut findings: Vec<InlineFinding> = Vec::new();
    for (file, module) in governed {
        let raw = &file_text[file];
        let use_map = &use_maps[file];
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(raw));
        // The chase map is the crate-wide def closure PLUS this file's own `use`-map (keyed by the
        // alias's fully-qualified local name), so a two-hop `use`-re-alias resolves: `use
        // std::time::SystemTime; use self::SystemTime as Clock;` → `Clock` → `crate::m::SystemTime`
        // (use-map) → `std::time::SystemTime` (this file's other `use`, now in the chase map).
        let mut chase_defs = ctx.defs.clone();
        for (alias, target) in use_map {
            chase_defs
                .entry(format!("{module}::{alias}"))
                .or_insert_with(|| target.clone());
        }

        // (i) Glob-hazard: a glob import that can bring a prefix-resolving name into scope reacts
        // fail-closed. Read from decl_text (a glob is a `use`, not a call).
        for glob_path in glob_import_paths(&decl_text) {
            if let Some(resolved) = resolve_head(
                &glob_path,
                module,
                module,
                use_map,
                root_modules,
                external_vocab.as_ref(),
            ) {
                if glob_reaches_prefix(&resolved, &prefix, &ctx, &mut HashSet::new()) {
                    findings.push(InlineFinding {
                        fact: ModuleFact::InlineGlob {
                            path: glob_path,
                            module: module.clone(),
                        },
                        file: file.display().to_string(),
                    });
                }
            }
        }

        // (ii) Call / mention scan: from comment/string-stripped text WITH macro bodies kept
        // (real reads hide in `cfg_if!` / logging / async DSL bodies — scanned, never skipped).
        let call_text = strip_comments_and_strings(raw);
        for occurrence in path_occurrences(&call_text, module, external) {
            // A glob has no call terminal segment; narrowing / call-vs-mention apply to paths only.
            let Some(resolved) = resolve_head(
                &occurrence.segments,
                module,
                &occurrence.module,
                use_map,
                root_modules,
                external_vocab.as_ref(),
            ) else {
                continue; // unresolved head — not matched by leaf (would be a false positive)
            };
            let resolved = chase_closure(&resolved, &chase_defs, &mut HashSet::new());
            if !path_within(&resolved, &prefix) {
                continue;
            }
            let react = if strict {
                true // strict: any path under the prefix, call or not
            } else if !occurrence.is_call {
                false // default: only calls (a mention / value capture passes — a stated bound)
            } else if let Some(verbs) = &verbs {
                // narrowed: the terminal segment must be a declared read verb (leaf-exact)
                resolved
                    .rsplit("::")
                    .next()
                    .is_some_and(|leaf| verbs.iter().any(|v| v == leaf))
            } else {
                true // default call, no narrowing: every call under the prefix
            };
            if react {
                findings.push(InlineFinding {
                    fact: ModuleFact::InlinePath {
                        path: resolved,
                        module: module.clone(),
                    },
                    file: file.display().to_string(),
                });
            }
        }
    }

    // One violation per distinct finding (per-(canonical path / glob, module)); keep the first
    // file after a deterministic sort, so a subtree spanning several files does not double-count.
    findings.sort_by(|a, b| a.fact.cmp(&b.fact).then(a.file.cmp(&b.file)));
    findings.dedup_by(|a, b| a.fact == b.fact);
    Ok(findings)
}

/// A path occurrence in call/mention position: its `::`-joined segments, whether it is applied as a
/// call (`path(...)` or `path::<...>(...)`), and the true (inline) module that lexically encloses it
/// (`{file_module}::inner…`). The `module` feeds ONLY the strict-external local-shadow check in
/// [`resolve_head`]; the finding text and default resolution stay keyed on the file module.
struct PathOccurrence {
    segments: String,
    is_call: bool,
    module: String,
}

/// Extract path occurrences from already comment/string-stripped source. A path is an identifier
/// head (not preceded by an identifier byte or `.` — so a method / field access `x.now()` is not a
/// head; a leading `::` or a struct-field `:` before the head is fine, the head is captured) with
/// zero or more `::`-joined segments, tolerating interior whitespace and mid-path turbofish
/// `::<…>`. A trailing `(` (after whitespace / a turbofish) marks a call. Interior `::`
/// continuations are consumed greedily so a mid-path segment is never independently re-scanned.
///
/// Under `external` each occurrence carries its true (inline) module, tracked by inline
/// `mod name { … }` nesting exactly as [`super::use_scan`]'s walk does (non-`mod` braces move the
/// depth but never touch the stack, so a call anywhere inside `mod tests { … }` attributes to
/// `…::tests`). When `external` is `false` the tracking is skipped entirely and every occurrence is
/// keyed to `base_module`.
fn path_occurrences(source: &str, base_module: &str, external: bool) -> Vec<PathOccurrence> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    // Inline-`mod` nesting, populated ONLY under a strict-external boundary (each entry is `(name,
    // enclosing brace depth)`).
    let mut depth = 0usize;
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    while i < bytes.len() {
        if external {
            if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
                mod_stack.push((
                    canonical_segment(&normalize_segments(&bytes[name_start..name_end]))
                        .to_string(),
                    depth,
                ));
                i = brace; // let the `{` arm below increment the depth
                continue;
            }
            match bytes[i] {
                b'{' => {
                    depth += 1;
                    i += 1;
                    continue;
                }
                b'}' => {
                    depth = depth.saturating_sub(1);
                    while mod_stack.last().is_some_and(|(_, d)| *d == depth) {
                        mod_stack.pop();
                    }
                    i += 1;
                    continue;
                }
                _ => {}
            }
        }
        if !is_ident_byte(bytes[i]) || bytes[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        // A head must not sit mid-identifier or after `.` (a method / field access — a stated
        // bound). A preceding `:` (a leading `::` global path, or a struct-field colon) is fine.
        let prev = i.checked_sub(1).map(|p| bytes[p]);
        if prev.is_some_and(is_ident_byte) || prev == Some(b'.') {
            i = end_of_ident(bytes, i);
            continue;
        }
        // Collect the path: ident ( ws? :: ws? (ident | turbofish `<…>`) )*.
        let start = i;
        let mut end = end_of_ident(bytes, i);
        loop {
            let j = skip_ws(bytes, end);
            if bytes.get(j) == Some(&b':') && bytes.get(j + 1) == Some(&b':') {
                let after = skip_ws(bytes, j + 2);
                if bytes
                    .get(after)
                    .is_some_and(|b| is_ident_byte(*b) && !b.is_ascii_digit())
                {
                    end = end_of_ident(bytes, after);
                    continue;
                }
                if bytes.get(after) == Some(&b'<') {
                    // a mid-path turbofish `::<…>` — skip the balanced generics, continue the path
                    end = skip_angles(bytes, after);
                    continue;
                }
            }
            break;
        }
        // Build the canonical `::`-joined segments, dropping interior whitespace and turbofish
        // `<…>` groups (generic args, not path segments).
        let segments = normalize_segments(&bytes[start..end]);
        let is_call = is_call_application(bytes, end);
        if segments.contains("::") || is_call {
            let module = if external {
                effective_module(base_module, &mod_stack)
            } else {
                base_module.to_string()
            };
            out.push(PathOccurrence {
                segments,
                is_call,
                module,
            });
        }
        i = end.max(i + 1);
    }
    out
}

/// Index just past the identifier starting at `i`, tolerating a leading raw-identifier `r#`.
fn end_of_ident(bytes: &[u8], i: usize) -> usize {
    let mut j = i;
    if bytes.get(j) == Some(&b'r') && bytes.get(j + 1) == Some(&b'#') {
        j += 2;
    }
    while j < bytes.len() && is_ident_byte(bytes[j]) {
        j += 1;
    }
    j.max(i + 1)
}

/// Index of the next non-whitespace byte at or after `i`.
fn skip_ws(bytes: &[u8], i: usize) -> usize {
    let mut j = i;
    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
        j += 1;
    }
    j
}

/// Index just past the balanced `<…>` group opening at `start` (`bytes[start] == '<'`); the end of
/// input if unbalanced (never panics).
fn skip_angles(bytes: &[u8], start: usize) -> usize {
    let mut depth = 0usize;
    let mut k = start;
    while k < bytes.len() {
        match bytes[k] {
            b'<' => depth += 1,
            b'>' => {
                depth -= 1;
                if depth == 0 {
                    return k + 1;
                }
            }
            _ => {}
        }
        k += 1;
    }
    bytes.len()
}

/// Reduce a captured path span to its `::`-joined identifier segments, dropping interior
/// whitespace, `::` separators, and balanced turbofish `<…>` groups; a raw-identifier `r#name`
/// segment is canonicalized to `name`.
fn normalize_segments(span: &[u8]) -> String {
    let mut segs: Vec<String> = Vec::new();
    let mut k = 0;
    while k < span.len() {
        if span[k] == b'<' {
            k = skip_angles(span, k);
        } else if is_ident_byte(span[k]) || (span[k] == b'r' && span.get(k + 1) == Some(&b'#')) {
            let s = end_of_ident(span, k);
            let seg = String::from_utf8_lossy(&span[k..s]).into_owned();
            segs.push(seg.strip_prefix("r#").unwrap_or(&seg).to_string());
            k = s;
        } else {
            k += 1;
        }
    }
    segs.join("::")
}

/// Whether a call application `(` follows the path ending at `end`, skipping whitespace and an
/// optional (trailing) turbofish `::<…>`.
fn is_call_application(bytes: &[u8], end: usize) -> bool {
    let mut j = skip_ws(bytes, end);
    if bytes.get(j) == Some(&b':') && bytes.get(j + 1) == Some(&b':') {
        let after = skip_ws(bytes, j + 2);
        if bytes.get(after) == Some(&b'<') {
            j = skip_ws(bytes, skip_angles(bytes, after));
        }
    }
    bytes.get(j) == Some(&b'(')
}

/// Every glob import path in already-declaration-cleaned source: a `use <path>::*;` (bare) or a
/// grouped `use <path>::{ … * … };` (the `*` among the group members). Returns the module-path
/// `<path>` (without the trailing `::*`), for each glob.
fn glob_import_paths(source: &str) -> Vec<String> {
    let mut paths = Vec::new();
    for tree in use_statements(source) {
        glob_bases(&tree, &mut paths, 0);
    }
    paths
}

/// Collect every glob base path in a use tree, recursing into groups (so a **nested** glob member
/// `use std::{time::*, io::Write}` yields `std::time`, not just a top-level `use std::time::*`).
/// A bare tail `a::b::*` → `a::b`; a group member `*` → the group prefix. Brace handling goes
/// through [`brace_inner`] / [`split_top_commas`] (char-based, never a byte slice), so a malformed
/// `}`-before-`{` cannot panic.
fn glob_bases(tree: &str, out: &mut Vec<String>, depth: usize) {
    if depth > 64 {
        return;
    }
    let tree = tree.trim();
    match tree.find('{') {
        Some(open) => {
            let prefix = tree[..open].trim();
            for part in split_top_commas(&brace_inner(&tree[open..])) {
                let part = part.trim();
                if part.is_empty() {
                    continue;
                }
                if part == "*" {
                    let base = prefix.trim_end_matches(':').trim();
                    if !base.is_empty() {
                        out.push(base.to_string());
                    }
                } else {
                    glob_bases(&format!("{prefix}{part}"), out, depth + 1);
                }
            }
        }
        None => {
            if let Some(base) = tree.strip_suffix("::*") {
                let base = base.trim();
                if !base.is_empty() {
                    out.push(base.to_string());
                }
            }
        }
    }
}

/// The raw `use …` statement bodies (the text between `use` and `;`), from declaration-cleaned
/// source. A lightweight cousin of the `use`-scan's walk, sufficient for glob detection.
fn use_statements(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"use") {
            let start = i + 3;
            let mut p = start;
            while p < bytes.len() && bytes[p].is_ascii_whitespace() {
                p += 1;
            }
            if bytes.get(p) == Some(&b'<') {
                i = start; // a `use<…>` precise-capturing bound, not an import
                continue;
            }
            if let Some(rel) = source[start..].find(';') {
                trees.push(source[start..start + rel].trim().to_string());
                i = start + rel + 1;
                continue;
            }
            break;
        }
        i += 1;
    }
    trees
}

/// Build the per-file alias-carrying use-map: the head identifier a `use` introduces → the target
/// path it names (canonicalized). `use std::time::SystemTime as SysT;` → `SysT` →
/// `std::time::SystemTime`; `use std::time;` → `time` → `std::time`; `use std::time::SystemTime;`
/// → `SystemTime` → `std::time::SystemTime`. Grouped forms are expanded. Glob (`::*`) entries name
/// no head, so they are skipped here (the glob-hazard rule handles them).
fn collect_use_map(
    source: &str,
    current_module: &str,
    root_modules: &[String],
) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for tree in use_statements(source) {
        for (alias, path) in expand_use_leaves(&tree) {
            if let Some(canonical) = resolve_written_path(&path, current_module, root_modules) {
                map.insert(alias, canonical);
            }
        }
    }
    map
}

/// Expand a use tree into `(introduced-head-identifier, written-path)` leaves. `a::{b, c as d}` →
/// `(b, a::b)`, `(d, a::c)`. A `self`/glob leaf introduces no simple head and is skipped.
fn expand_use_leaves(tree: &str) -> Vec<(String, String)> {
    fn go(tree: &str, out: &mut Vec<(String, String)>, depth: usize) {
        if depth > 64 {
            return;
        }
        let tree = tree.trim();
        match tree.find('{') {
            Some(open) => {
                let prefix = tree[..open].trim();
                let inner = brace_inner(&tree[open..]);
                for part in split_top_commas(&inner) {
                    let part = part.trim();
                    if part.is_empty() || part == "*" || part.starts_with("self") {
                        continue;
                    }
                    go(&format!("{prefix}{part}"), out, depth + 1);
                }
            }
            None => {
                if tree.ends_with("::*") || tree.is_empty() {
                    return;
                }
                let (path, alias) = match tree.split_once(" as ") {
                    Some((p, a)) => (p.trim().to_string(), a.trim().to_string()),
                    None => {
                        let leaf = tree.rsplit("::").next().unwrap_or(tree).trim();
                        (tree.to_string(), leaf.to_string())
                    }
                };
                let alias = canonical_module_path(&alias);
                if !alias.is_empty() {
                    out.push((alias, path));
                }
            }
        }
    }
    let mut out = Vec::new();
    go(tree, &mut out, 0);
    out
}

/// Collect the `type`-alias and `pub use` re-export definitions of a file into the crate-wide
/// context, keyed by their fully-qualified local name. Targets are resolved module-relative
/// (through the file's `use_map`), so `type B = A;` targets the sibling `crate::mod::A` and
/// `use std::time::SystemTime; type Clock = SystemTime;` targets `std::time::SystemTime`.
fn collect_defs(
    source: &str,
    module: &str,
    root_modules: &[String],
    use_map: &HashMap<String, String>,
    ctx: &mut ResolveCtx,
) {
    // `type Name = Target;`
    for (name, target) in type_aliases(source) {
        if let Some(canonical) = resolve_target(&target, module, use_map, root_modules) {
            ctx.defs.insert(
                format!("{module}::{}", canonical_module_path(&name)),
                canonical,
            );
        }
    }
    // `pub use …` — named re-exports feed `defs`; glob re-exports (incl. nested) feed
    // `glob_reexports`, so a `pub use std::time::*;` inside a locally-globbed module is reachable
    // by the recursive glob-hazard test.
    for tree in pub_use_statements(source) {
        let mut globs = Vec::new();
        glob_bases(&tree, &mut globs, 0);
        for base in globs {
            if let Some(canonical) = resolve_written_path(&base, module, root_modules) {
                ctx.glob_reexports.push((module.to_string(), canonical));
            }
        }
        for (alias, path) in expand_use_leaves(&tree) {
            if let Some(canonical) = resolve_written_path(&path, module, root_modules) {
                ctx.defs.insert(format!("{module}::{alias}"), canonical);
            }
        }
    }
}

/// Collect the **true-module-qualified** names of every reachable module's own item definitions —
/// `mod`, `struct`, `enum`, `union`, `trait`, `type`, `fn`, `const`, `static` — from
/// declaration-cleaned source into `out` as `{true_module}::{name}`, where `{true_module}` is the
/// file's `module` extended by the inline `mod name { … }`s enclosing the item. Backs rung (iv) of
/// the strict-external local-precedence ladder: a bare head naming a local item **of the calling
/// module** is NOT reclassified as an external dependency (so a local `fn rand()` under a `rand`
/// dependency, or a local `struct`/`type`/plain `mod` named like a dep, stays clean). The
/// value-namespace items (`fn`/`const`/`static`) are included beyond
/// `hunyi::crate_scope::local_type_namespace_names` because a bare *call* head (`rand()`) binds to a
/// local `fn`.
///
/// Two disciplines keep this from *over*-suppressing (an external call silently read as local — the
/// one forbidden bug, a false negative):
/// - **True-module-qualified.** Names are keyed `{true_module}::{name}` and matched against
///   `{occurrence_module}::head` (mirroring rung iii), so a same-named item of another module never
///   cross-suppresses and a file-top item does not mask a call inside `mod tests { … }`.
/// - **Module top level only.** Only an item at its own module's top level enters that module's
///   bare-head scope (brace depth == the enclosing module's body-open depth); associated / block-
///   local items sit deeper and are skipped (capturing them would over-suppress a same-named
///   external call). An inline `mod`'s own name is itself such a top-level item. (Comments, strings,
///   and char literals are pre-stripped from `source`, so a `'}'` cannot miscount the depth.)
///
/// Residual stated bound: the full single-segment over-reaction (a local `let` / param / closure
/// binding, `must_not_call_inline("rand")` only; `chrono::Utc` is immune) is canonical in
/// `strict_external`'s rustdoc and not re-argued here. One corollary specific to this fn's
/// module-top-level-only discipline: the definition site of an associated / nested `fn` named like
/// the crate (whose `name(` reads as a call) may likewise false-positive under a single-segment prefix.
fn collect_item_definition_names(module: &str, source: &str, out: &mut HashSet<String>) {
    const KEYWORDS: [&[u8]; 9] = [
        b"mod", b"struct", b"enum", b"union", b"trait", b"type", b"fn", b"const", b"static",
    ];
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    // Inline `mod name { … }` nesting, mirroring `use_scan::use_trees_with_modules`: each entry is
    // `(name, enclosing brace depth)`. A submodule item is keyed by its true (inline) module
    // (`{module}::inner…::name`), so a submodule-local `fn rand` is `…::tests::rand`, not file-top.
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    while i < bytes.len() {
        // Body-open depth of the CURRENT module (0 at file top, +1 per open inline `mod`); only
        // items at exactly this depth are module-top-level bare-head names (see doc).
        let top = mod_stack.last().map_or(0, |(_, d)| d + 1);
        // An inline `mod name {`: its name is a top-level item of the CURRENT (enclosing) module —
        // captured when the `mod` sits at that module's top level — and its body opens a new scope.
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            let name = normalize_segments(&bytes[name_start..name_end]);
            if depth == top && !name.is_empty() {
                out.insert(format!("{}::{name}", effective_module(module, &mod_stack)));
            }
            mod_stack.push((canonical_segment(name.trim()).to_string(), depth));
            i = brace; // let the `{` arm below increment the depth
            continue;
        }
        match bytes[i] {
            b'{' => {
                depth += 1;
                i += 1;
                continue;
            }
            b'}' => {
                depth = depth.saturating_sub(1);
                while mod_stack.last().is_some_and(|(_, d)| *d == depth) {
                    mod_stack.pop();
                }
                i += 1;
                continue;
            }
            _ => {}
        }
        if !is_ident_byte(bytes[i]) {
            i += 1;
            continue;
        }
        // Only a module-top-level item keyword introduces a bare-head name into the CURRENT inline
        // module's scope; deeper keywords are associated / block-local items (see doc).
        if depth == top {
            if let Some(kw) = KEYWORDS
                .iter()
                .find(|kw| super::lexer::keyword_starts_at(bytes, i, kw))
            {
                // The declared name is the identifier following the keyword (across whitespace),
                // tolerating a raw-identifier `r#name`. A non-identifier there (e.g. `const _:` or a
                // `const fn` where `fn` is itself a keyword) simply captures nothing useful — the
                // subsequent keyword scan still reaches the real name.
                let name_start = skip_ws(bytes, i + kw.len());
                if bytes.get(name_start).is_some_and(|b| is_ident_byte(*b)) {
                    let name =
                        normalize_segments(&bytes[name_start..end_of_ident(bytes, name_start)]);
                    if !name.is_empty() {
                        out.insert(format!("{}::{name}", effective_module(module, &mod_stack)));
                    }
                }
            }
        }
        i = end_of_ident(bytes, i);
    }
}

/// Every `type Name = Target;` in declaration-cleaned source, as `(Name, Target)`.
fn type_aliases(source: &str) -> Vec<(String, String)> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"type") {
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            let name_start = j;
            while j < bytes.len() && is_ident_byte(bytes[j]) {
                j += 1;
            }
            let name = source[name_start..j].to_string();
            // Advance to the aliasing `=`, skipping a generic parameter list `<…>` whole (via the
            // shared angle walker) so a defaulted parameter's own `=` (`type C<T = Default> = …`) is
            // not mistaken for the alias `=`.
            while j < bytes.len() && bytes[j] != b'=' && bytes[j] != b';' {
                if bytes[j] == b'<' {
                    j = skip_angles(bytes, j);
                } else {
                    j += 1;
                }
            }
            if !name.is_empty() && bytes.get(j) == Some(&b'=') {
                // The target runs to the top-level `;`, honoring `[]`/`()`/`{}` nesting and `<…>`
                // groups so an inner `;` (an array type `[T; N]`) does not truncate it.
                if let Some(end) = alias_target_end(bytes, j + 1) {
                    let target = source[j + 1..end].trim();
                    // Take the leading path of the target (drop generic args / where-ish tails).
                    let target_path = leading_path(target);
                    if !target_path.is_empty() {
                        out.push((name, target_path));
                    }
                    i = end + 1;
                    continue;
                }
            }
            i = j.max(i + 1);
            continue;
        }
        i += 1;
    }
    out
}

/// Index of the top-level `;` that terminates a `type … = <target>;`, starting at `from` (just past
/// the aliasing `=`); the end of input if none. Tracks `[]`/`()`/`{}` nesting and skips `<…>` groups
/// whole (via [`skip_angles`], so a `->` return arrow's `>` is never miscounted), so a `;` inside an
/// array/tuple type (`[T; N]`) does not prematurely end the target.
fn alias_target_end(bytes: &[u8], from: usize) -> Option<usize> {
    let mut depth = 0i32;
    let mut k = from;
    while k < bytes.len() {
        match bytes[k] {
            b'<' => {
                k = skip_angles(bytes, k);
                continue;
            }
            b'[' | b'(' | b'{' => depth += 1,
            b']' | b')' | b'}' => depth -= 1,
            b';' if depth <= 0 => return Some(k),
            _ => {}
        }
        k += 1;
    }
    None
}

/// The leading `::`-path of a type expression (`std::time::SystemTime<T>` → `std::time::SystemTime`,
/// `&Foo` → `Foo`). Stops at the first byte that is neither an identifier byte nor `:`.
fn leading_path(expr: &str) -> String {
    let expr = expr.trim_start_matches(['&', ' ', '*']);
    let bytes = expr.as_bytes();
    let mut j = 0;
    while j < bytes.len() && (is_ident_byte(bytes[j]) || bytes[j] == b':') {
        j += 1;
    }
    expr[..j].trim_end_matches(':').to_string()
}

/// The `pub use …` statement bodies (only `pub` re-exports feed the crate-wide closure; a private
/// `use` is local to its file and already handled per-file by the use-map).
fn pub_use_statements(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"pub") {
            let mut j = i + 3;
            // optional `(crate)` / `(super)` / `(in path)` visibility qualifier
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes.get(j) == Some(&b'(') {
                let mut depth = 0usize;
                while j < bytes.len() {
                    match bytes[j] {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                j += 1;
                                break;
                            }
                        }
                        _ => {}
                    }
                    j += 1;
                }
                while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
            }
            if super::lexer::keyword_starts_at(bytes, j, b"use") {
                let start = j + 3;
                if let Some(rel) = source[start..].find(';') {
                    trees.push(source[start..start + rel].trim().to_string());
                    i = start + rel + 1;
                    continue;
                }
                break;
            }
        }
        i += 1;
    }
    trees
}

/// Content inside the first `{ … }` of `s` (which starts at `{`), honoring nesting.
fn brace_inner(s: &str) -> String {
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
    let mut depth = 0i32;
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

/// The extra crate vocabulary [`resolve_head`] consults ONLY under `.strict_external()`: the
/// complete crate module-path set (rung iii), the module-qualified top-level item-definition names
/// (rung iv), and the declared-dependency import identifiers (rung v). Absent (`None`) on the
/// default path.
struct ExternalVocab<'a> {
    module_paths: &'a HashSet<String>,
    item_defs: &'a HashSet<String>,
    dep_names: &'a HashSet<String>,
}

/// Under `.strict_external()`, whether a bare `head` (already declined by the per-file use-map,
/// rung i) reclassifies as an **external crate** — i.e. it matches a declared dependency name AND
/// is not claimed by local precedence. `occurrence_module` is the true (inline) module the call
/// occurs in (inline-`mod`-aware — see `path_occurrences`), not necessarily the file module. Local
/// precedence
/// (first match wins) suppresses the dependency match: (ii) a crate-root module shadow; (iii) a
/// local module `{occurrence_module}::head` (at ANY depth, from the full crate module-path set —
/// not only crate-root children); (iv) a local top-level item definition `{occurrence_module}::head`
/// of the true (inline) module (module-qualified, mirroring iii — a same-named item of another module
/// never suppresses, which would be a false negative). Only when none of these claim the head does
/// the dependency match fire.
fn head_is_external_dependency(
    head: &str,
    occurrence_module: &str,
    root_modules: &[String],
    vocab: &ExternalVocab,
) -> bool {
    let locally_shadowed = is_crate_root_shadow(occurrence_module, head, root_modules)
        || vocab
            .module_paths
            .contains(&format!("{occurrence_module}::{head}"))
        || vocab
            .item_defs
            .contains(&format!("{occurrence_module}::{head}"));
    !locally_shadowed && vocab.dep_names.contains(head)
}

/// Resolve the head of a written path occurrence (its `::`-joined `segments`) to a canonical path,
/// via the per-file use-map, then treating a `std`/`core`/`alloc` head as literal, a
/// `crate`/`self`/`super` head as local, and any other bare head as a local item of the current
/// module (so a `type`/`pub use` closure can then rewrite it). Returns `None` only for an
/// empty/degenerate path. Leaf-only matching of an unresolved head is deliberately NOT done.
///
/// `external` is `Some` ONLY under `.strict_external()`: an un-`use`d bare head that matches a
/// declared dependency (and is not locally shadowed — see [`head_is_external_dependency`]) is then
/// kept as the literal external path (`chrono::Utc::…`) instead of the fake-local
/// `{module}::chrono::Utc::…`, closing the fully-qualified-external false negative. When `external`
/// is `None` the default path is unchanged: every non-`use` bare head falls to the load-bearing
/// `{module}::…` fallback the `type`-alias / re-export closure depends on.
///
/// `occurrence_module` is the occurrence's true (inline) module (`{file_module}::inner…`) and is used
/// ONLY inside the `external` branch, for the local-shadow ladder — so a file-top item cannot mask
/// an external call in an inline submodule, and a submodule-local item shadows only its own module.
/// Everything else (the `{current_module}::…` fallback, `self`/`super`, the finding's module) keeps
/// using the FILE module `current_module`; on the default path (`external` `None`) the parameter is
/// unread (the caller passes the file module there anyway).
fn resolve_head(
    segments: &str,
    current_module: &str,
    occurrence_module: &str,
    use_map: &HashMap<String, String>,
    root_modules: &[String],
    external: Option<&ExternalVocab>,
) -> Option<String> {
    let raw = segments.trim().trim_start_matches("::");
    let parts: Vec<String> = raw
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let (head, rest) = parts.split_first()?;
    let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
    let base: String = match head.as_str() {
        "std" | "core" | "alloc" => parts.join("::"),
        "crate" => parts.join("::"),
        // `self`/`super` relative resolution (incl. the `super` over-pop guard, whose `None` is
        // `?`-propagated) lives once in `path_vocab::resolve_self_super`.
        "self" | "super" => resolve_self_super(current_module, &parts_str)?,
        other => {
            if let Some(target) = use_map.get(other) {
                // (i) alias / imported head → its target, then the remaining segments.
                let mut base = target.clone();
                for seg in rest {
                    base.push_str("::");
                    base.push_str(seg);
                }
                base
            } else if external.is_some_and(|v| {
                // (ii)-(iv) local-shadow ladder vs the occurrence's true (inline) module, then
                // (v) the declared-dependency match (see `head_is_external_dependency`).
                head_is_external_dependency(other, occurrence_module, root_modules, v)
            }) {
                // (v) strict-external: a fully-qualified, un-`use`d external head not shadowed by
                // any local rung — keep the literal external path so it prefix-matches.
                parts.join("::")
            } else {
                // an un-imported bare head is a local item of the current module (a local `type`
                // alias / definition); the closure rewrites it if it re-exports under the prefix.
                // (Also every local rung under the flag lands here, staying local — no FP.)
                format!("{current_module}::{}", parts.join("::"))
            }
        }
    };
    Some(base)
}

/// Chase a candidate path through the `type`-alias / `pub use` closure to a fixpoint: repeatedly
/// replace the longest local-name prefix that is a `defs` key with its target. Cycle-safe via the
/// visited set.
fn chase_closure(
    path: &str,
    defs: &HashMap<String, String>,
    visited: &mut HashSet<String>,
) -> String {
    let mut current = path.to_string();
    // A step cap in addition to the visited set: a self-referential def (`type A = A::B;`, which
    // does not compile but is observable) rewrites to a strictly longer, never-repeating path each
    // round, which the visited set cannot catch — the cap guarantees termination (never hang).
    for _ in 0..256 {
        if !visited.insert(current.clone()) {
            return current; // cycle — stop
        }
        // Longest-prefix def match: try the full path, then successively shorter `::` prefixes.
        let mut matched: Option<(String, String)> = None;
        let segments: Vec<&str> = current.split("::").collect();
        for take in (1..=segments.len()).rev() {
            let key = segments[..take].join("::");
            if let Some(target) = defs.get(&key) {
                let remainder = &segments[take..];
                let mut next = target.clone();
                for seg in remainder {
                    next.push_str("::");
                    next.push_str(seg);
                }
                matched = Some((key, next));
                break;
            }
        }
        match matched {
            Some((_, next)) => current = next,
            None => return current,
        }
    }
    current // step cap reached (pathological self-referential defs) — stop, never hang
}

/// Whether a glob whose resolved module path is `glob` can bring a name resolving under `prefix`
/// into scope — the fail-closed hazard test, applied recursively to local-module re-export
/// closures. (a) `glob` is the prefix or beneath it; (b) `glob` is an ancestor of the prefix; (c)
/// `glob` is a local module whose named re-exports/`type`s reach under the prefix, or which itself
/// glob-re-exports a path that (recursively) reaches the prefix.
fn glob_reaches_prefix(
    glob: &str,
    prefix: &str,
    ctx: &ResolveCtx,
    visited: &mut HashSet<String>,
) -> bool {
    if !visited.insert(glob.to_string()) {
        return false; // cycle guard
    }
    // (a) prefix or beneath, and (b) ancestor of the prefix.
    if path_within(glob, prefix) || path_within(prefix, glob) {
        return true;
    }
    // (c) a local module whose named defs reach under the prefix.
    for (name, target) in &ctx.defs {
        if path_within(name, glob) {
            let resolved = chase_closure(target, &ctx.defs, &mut HashSet::new());
            if path_within(&resolved, prefix) {
                return true;
            }
        }
    }
    // (c, recursive) a glob re-export in a module within `glob` whose own path reaches the prefix.
    let inner: Vec<String> = ctx
        .glob_reexports
        .iter()
        .filter(|(module, _)| path_within(module, glob))
        .map(|(_, gp)| gp.clone())
        .collect();
    for gp in inner {
        if glob_reaches_prefix(&gp, prefix, ctx, visited) {
            return true;
        }
    }
    false
}

/// Resolve a *written* module path (from a `use` / `type` / `pub use`) to a canonical absolute
/// form, for the def closure and use-map. A `std`/`core`/`alloc` head or any external head stays
/// as written (canonicalized); `crate`/`self`/`super` resolve against `current_module`; a bare
/// head naming a crate-root module resolves to `crate::…` only at the crate root (the shadow rule).
fn resolve_written_path(
    path: &str,
    current_module: &str,
    root_modules: &[String],
) -> Option<String> {
    let raw = path.trim();
    let global = raw.starts_with("::");
    let parts: Vec<String> = raw
        .trim_start_matches("::")
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let (head, _rest) = parts.split_first()?;
    let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
    match head.as_str() {
        "crate" | "std" | "core" | "alloc" => Some(parts.join("::")),
        _ if global => Some(parts.join("::")), // `::name::…` — global/external, kept as written
        // `self`/`super` relative resolution — incl. the `super` over-pop guard — lives once in
        // `path_vocab::resolve_self_super`.
        "self" | "super" => resolve_self_super(current_module, &parts_str),
        other => {
            if is_crate_root_shadow(current_module, other, root_modules) {
                let mut out = vec!["crate".to_string()];
                out.extend(parts.iter().cloned());
                Some(out.join("::"))
            } else {
                // an external head (e.g. `std`-alike from another crate) — kept as written so the
                // closure can still compare it literally against the prefix.
                Some(parts.join("::"))
            }
        }
    }
}

/// Resolve a `type`-alias / re-export **target** module-relative: a bare head is looked up in the
/// file's `use_map`; a `std`/`core`/`alloc`/`crate` head is literal; `self`/`super` resolve as a
/// path; any other bare head is a **local** item of the current module (so `type B = A;` targets
/// `{module}::A`, chaining through the closure). Contrast [`resolve_written_path`], which treats a
/// bare `use`-path head as an external crate.
fn resolve_target(
    target: &str,
    module: &str,
    use_map: &HashMap<String, String>,
    root_modules: &[String],
) -> Option<String> {
    let raw = target.trim();
    if raw.starts_with("::") {
        return resolve_written_path(raw, module, root_modules);
    }
    let parts: Vec<String> = raw
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let (head, rest) = parts.split_first()?;
    if let Some(mapped) = use_map.get(head) {
        let mut base = mapped.clone();
        for seg in rest {
            base.push_str("::");
            base.push_str(seg);
        }
        return Some(base);
    }
    match head.as_str() {
        "std" | "core" | "alloc" | "crate" => Some(parts.join("::")),
        "self" | "super" => resolve_written_path(raw, module, root_modules),
        _ => Some(format!("{module}::{}", parts.join("::"))),
    }
}
