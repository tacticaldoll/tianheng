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
    brace_content, canonical_module_path, canonical_segment, effective_module,
    fold_canonical_segments, inline_mod_at, is_crate_root_shadow, path_within, resolve_self_super,
    split_top_commas,
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

/// Scan the crate for inline-symbol-path offences against a `ConfineInlineSymbolPath` boundary;
/// its default and strict-external forms both route here via `inline_payload`.
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
    let verbs: Option<Vec<String>> =
        ending_with.map(|vs| vs.iter().map(|v| canonical_module_path(v)).collect());

    let mut ctx = ResolveCtx {
        defs: HashMap::new(),
        glob_reexports: Vec::new(),
    };
    let mut file_text: HashMap<std::path::PathBuf, String> = HashMap::new();
    let mut use_maps: HashMap<std::path::PathBuf, HashMap<String, String>> = HashMap::new();
    let mut module_paths: HashSet<String> = HashSet::new();
    let mut item_defs: HashSet<String> = HashSet::new();
    for (file, module) in all_files {
        let raw = std::fs::read_to_string(file)
            .map_err(|err| crate::errors::unreadable_governed_file_error(file, &err.to_string()))?;
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(&raw));
        let use_map = collect_use_map(&decl_text, module, root_modules)?;
        collect_defs(&decl_text, module, root_modules, &use_map, &mut ctx)?;
        if external {
            module_paths.insert(module.clone());
            collect_item_definition_names(module, &decl_text, &mut item_defs);
        }
        use_maps.insert(file.clone(), use_map);
        file_text.insert(file.clone(), raw);
    }
    let dep_names: HashSet<String> = if external {
        dependency_names.iter().cloned().collect()
    } else {
        HashSet::new()
    };
    let external_vocab = external.then_some(ExternalVocab {
        module_paths: &module_paths,
        item_defs: &item_defs,
        dep_names: &dep_names,
    });

    let mut findings: Vec<InlineFinding> = Vec::new();
    for (file, module) in governed {
        let raw = &file_text[file];
        let use_map = &use_maps[file];
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(raw));
        let mut chase_defs = ctx.defs.clone();
        for (alias, target) in use_map {
            chase_defs
                .entry(format!("{module}::{alias}"))
                .or_insert_with(|| target.clone());
        }

        for glob_path in glob_import_paths(&decl_text)? {
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

        let call_text = strip_comments_and_strings(raw);
        for occurrence in path_occurrences(&call_text, module, external) {
            let Some(resolved) = resolve_head(
                &occurrence.segments,
                module,
                &occurrence.module,
                use_map,
                root_modules,
                external_vocab.as_ref(),
            ) else {
                continue;
            };
            let resolved = chase_closure(&resolved, &chase_defs, &mut HashSet::new());
            if !path_within(&resolved, &prefix) {
                continue;
            }
            if should_react_on_occurrence(strict, occurrence.is_call, verbs.as_deref(), &resolved) {
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

/// Scan all call and path-mention occurrences in `source`.
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
                i = brace;
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
        let prev = i.checked_sub(1).map(|p| bytes[p]);
        if prev.is_some_and(is_ident_byte) || prev == Some(b'.') {
            i = end_of_ident(bytes, i);
            continue;
        }
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
                    end = skip_angles(bytes, after);
                    continue;
                }
            }
            break;
        }
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
            segs.push(canonical_segment(&seg).to_string());
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

/// A brace-nesting depth cap for this file's two hand-rolled `use`-tree walkers ([`glob_bases`],
/// [`expand_use_leaves`]'s inner `go`), so a pathologically nested `use` cannot overflow the
/// stack — a DoS backstop set far beyond any real or lint-clean source. Past the cap, fail loud
/// (a scan error) rather than silently dropping the sub-tree: a real, compilable `use` nested
/// past this depth would otherwise vanish from observation with no report — the false negative
/// PROJECT.md's core contract forbids. Mirrors `use_scan::MAX_USE_NEST_DEPTH`'s identical
/// rationale for the same shape of walker.
const MAX_SYMBOL_NEST_DEPTH: usize = 64;

/// Every glob import path in already-declaration-cleaned source: a `use <path>::*;` (bare) or a
/// grouped `use <path>::{ … * … };` (the `*` among the group members). Returns the module-path
/// `<path>` (without the trailing `::*`), for each glob.
fn glob_import_paths(source: &str) -> Result<Vec<String>, String> {
    let mut paths = Vec::new();
    for tree in use_statements(source) {
        glob_bases(&tree, &mut paths, 0)?;
    }
    Ok(paths)
}

/// Collect every glob base path in a use tree, recursing into groups (so a **nested** glob member
/// `use std::{time::*, io::Write}` yields `std::time`, not just a top-level `use std::time::*`).
/// A bare tail `a::b::*` → `a::b`; a group member `*` → the group prefix. Brace handling goes
/// through [`brace_content`] / [`split_top_commas`] (char-based, never a byte slice), so a malformed
/// `}`-before-`{` cannot panic.
fn glob_bases(tree: &str, out: &mut Vec<String>, depth: usize) -> Result<(), String> {
    if depth > MAX_SYMBOL_NEST_DEPTH {
        return Err(format!(
            "cannot judge a `use` tree nested past {MAX_SYMBOL_NEST_DEPTH} brace levels: '{tree}'"
        ));
    }
    let tree = tree.trim();
    match tree.find('{') {
        Some(open) => {
            let prefix = tree[..open].trim();
            for part in split_top_commas(&brace_content(&tree[open..])) {
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
                    glob_bases(&format!("{prefix}{part}"), out, depth + 1)?;
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
    Ok(())
}

/// The raw `use …` statement bodies (the text between `use` and `;`), from declaration-cleaned
/// source. A lightweight cousin of the `use`-scan's walk, sufficient for glob detection.
fn use_statements(source: &str) -> Vec<String> {
    use super::lexer::UseStatementScan;
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"use") {
            match super::lexer::scan_use_statement(bytes, source, i) {
                UseStatementScan::Statement { body, next } => {
                    trees.push(body);
                    i = next;
                    continue;
                }
                UseStatementScan::NotAStatement { resume_at } => {
                    i = resume_at;
                    continue;
                }
                UseStatementScan::Unterminated => break,
            }
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
) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    for tree in use_statements(source) {
        for (alias, path) in expand_use_leaves(&tree)? {
            if let Some(canonical) = resolve_written_path(&path, current_module, root_modules) {
                map.insert(alias, canonical);
            }
        }
    }
    Ok(map)
}

/// Expand a use tree into `(introduced-head-identifier, written-path)` leaves. `a::{b, c as d}` →
/// `(b, a::b)`, `(d, a::c)`. A `self`/glob leaf introduces no simple head and is skipped.
fn expand_use_leaves(tree: &str) -> Result<Vec<(String, String)>, String> {
    fn go(tree: &str, out: &mut Vec<(String, String)>, depth: usize) -> Result<(), String> {
        if depth > MAX_SYMBOL_NEST_DEPTH {
            return Err(format!(
                "cannot judge a `use` tree nested past {MAX_SYMBOL_NEST_DEPTH} brace levels: '{tree}'"
            ));
        }
        let tree = tree.trim();
        match tree.find('{') {
            Some(open) => {
                let prefix = tree[..open].trim();
                let inner = brace_content(&tree[open..]);
                for part in split_top_commas(&inner) {
                    let part = part.trim();
                    let head = match part.find(" as ") {
                        Some(idx) => part[..idx].trim(),
                        None => part,
                    };
                    if part.is_empty() || part == "*" || head == "self" {
                        continue;
                    }
                    go(&format!("{prefix}{part}"), out, depth + 1)?;
                }
            }
            None => {
                if tree.ends_with("::*") || tree.is_empty() {
                    return Ok(());
                }
                let (path, alias) = match tree.split_once(" as ") {
                    Some((p, a)) => (p.trim().to_string(), a.trim().to_string()),
                    None => {
                        let leaf = tree.rsplit_once("::").map_or(tree, |(_, leaf)| leaf).trim();
                        (tree.to_string(), leaf.to_string())
                    }
                };
                let alias = canonical_module_path(&alias);
                if !alias.is_empty() {
                    out.push((alias, path));
                }
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    go(tree, &mut out, 0)?;
    Ok(out)
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
) -> Result<(), String> {
    for (name, target) in type_aliases(source) {
        if let Some(canonical) = resolve_target(&target, module, use_map, root_modules) {
            ctx.defs.insert(
                format!("{module}::{}", canonical_module_path(&name)),
                canonical,
            );
        }
    }
    for tree in pub_use_statements(source) {
        let mut globs = Vec::new();
        glob_bases(&tree, &mut globs, 0)?;
        for base in globs {
            if let Some(canonical) = resolve_written_path(&base, module, root_modules) {
                ctx.glob_reexports.push((module.to_string(), canonical));
            }
        }
        for (alias, path) in expand_use_leaves(&tree)? {
            if let Some(canonical) = resolve_written_path(&path, module, root_modules) {
                ctx.defs.insert(format!("{module}::{alias}"), canonical);
            }
        }
    }
    Ok(())
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
///   One brace is transparent to this rule: an `extern` block's. Its `fn`/`static` items are declared in
///   the module that CONTAINS the block, not in a scope of their own, so they are module-top-level
///   despite sitting one depth deeper — see [`extern_block_brace_at`]. That is right for this ladder as
///   well as for the value-namespace query: a bare `rand()` call resolves to a local
///   `extern "C" { pub fn rand(); }` exactly as it would to a plain local `fn rand()`, so treating the
///   extern one as absent read a local call as an external dependency.
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
    collect_definition_names(module, source, &KEYWORDS, true, out);
}

/// The **value-namespace** names a module declares at its own top level: `fn`, `const`, `static`.
///
/// Rust resolves a `mod` in the TYPE namespace, so the only names that can legally collide with
/// `mod foo` are these — `struct foo` beside `mod foo` would be a duplicate type-namespace
/// definition and does not compile. One `use m::foo;` then binds **both**, which is why an inbound
/// module boundary anchored at `m` must consult this: the module reading alone resolves the import to
/// the descendant `m::foo` and misses that it also reaches `m` itself (see
/// `module_check::resolve_import_module`).
///
/// Shares [`collect_item_definition_names`]'s walk, and with it both disciplines that keep the
/// answer honest: names are keyed by their **true** (inline-`mod`-qualified) module, and only items at
/// their own module's top level are captured, so an associated or block-local `fn` of the same name
/// does not count. Inline `mod` names are deliberately NOT captured here — they are the type-namespace
/// side of the very collision this exists to detect.
///
/// "Top level" includes an item inside an `extern` block opened at that level, because such a block opens
/// no naming scope: `unsafe extern "C" { pub fn foo(); }` declares `foo` in the enclosing module, and it
/// coexists with `mod foo` for exactly the namespace reason this function exists to observe. Treating that
/// brace like any other made the value invisible and a real import of the governed module pass silently —
/// the class `PROJECT.md` forbids outright, and the shape 渾儀 had already been corrected for.
///
/// `source` must be declaration-cleaned (comments, strings, and macro bodies stripped), like every
/// other reader in this module: an item declared inside a macro body is not observed, a stated bound.
pub(crate) fn value_namespace_item_names(module: &str, source: &str) -> HashSet<String> {
    const VALUE_KEYWORDS: [&[u8]; 3] = [b"fn", b"const", b"static"];
    let mut out = HashSet::new();
    collect_definition_names(module, source, &VALUE_KEYWORDS, false, &mut out);
    out
}

/// The shared walk behind [`collect_item_definition_names`] and [`value_namespace_item_names`]:
/// module-top-level definitions introduced by any of `keywords`, keyed `{true_module}::{name}`.
/// `capture_inline_mod_names` decides whether an inline `mod x { … }`'s own name is itself recorded as
/// a definition of the enclosing module — wanted for the local-precedence ladder, not for the
/// value-namespace query, whose whole point is to distinguish the two namespaces.
/// The `{` of an `extern` block starting at `i`, if one starts there: `extern {`, `extern "C" {`, and the
/// `unsafe extern "C" {` form Rust 2024 requires (the `unsafe` sits before the keyword, so matching on
/// `extern` alone reaches all three).
///
/// An extern block's brace opens no naming scope — its `fn`/`static` items are declared in the module
/// that CONTAINS the block, and can legally coexist with a `mod` of the same name because the two live in
/// different namespaces. `extern crate foo;` is deliberately not matched: it has no brace, so the `{`
/// requirement excludes it without a special case.
fn extern_block_brace_at(bytes: &[u8], i: usize) -> Option<usize> {
    if !super::lexer::keyword_starts_at(bytes, i, b"extern") {
        return None;
    }
    let mut cursor = skip_ws(bytes, i + b"extern".len());
    if bytes.get(cursor) == Some(&b'"') {
        cursor += 1;
        while cursor < bytes.len() && bytes[cursor] != b'"' {
            cursor += 1;
        }
        cursor = skip_ws(bytes, cursor.saturating_add(1));
    }
    (bytes.get(cursor) == Some(&b'{')).then_some(cursor)
}

/// Collect item definition names declared in `source` for the given module.
///
/// Tracks inline `mod name { … }` nesting so items are qualified under their true enclosing
/// module (`{module}::inner…::name`). An `extern` block's brace does not open a naming scope;
/// its items remain top-level items of the enclosing module. Modifiers like `static mut` skip the
/// unraw `mut` keyword to read the true declared name.
fn collect_definition_names(
    module: &str,
    source: &str,
    keywords: &[&[u8]],
    capture_inline_mod_names: bool,
    out: &mut HashSet<String>,
) {
    let bytes = source.as_bytes();
    let mut i = 0;
    let mut depth = 0usize;
    let mut mod_stack: Vec<(String, usize)> = Vec::new();
    let mut extern_opens: Vec<usize> = Vec::new();
    while i < bytes.len() {
        let module_top = mod_stack.last().map_or(0, |(_, d)| d + 1);
        let top = if extern_opens.last() == Some(&module_top) {
            module_top + 1
        } else {
            module_top
        };
        if let Some(brace) = extern_block_brace_at(bytes, i) {
            extern_opens.push(depth);
            i = brace;
            continue;
        }
        if let Some((name_start, name_end, brace)) = inline_mod_at(bytes, i) {
            let name = normalize_segments(&bytes[name_start..name_end]);
            if capture_inline_mod_names && depth == module_top && !name.is_empty() {
                out.insert(format!("{}::{name}", effective_module(module, &mod_stack)));
            }
            mod_stack.push((canonical_segment(name.trim()).to_string(), depth));
            i = brace;
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
                while extern_opens.last().is_some_and(|d| *d >= depth) {
                    extern_opens.pop();
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
        if depth == top {
            if let Some(kw) = keywords
                .iter()
                .find(|kw| super::lexer::keyword_starts_at(bytes, i, kw))
            {
                let mut name_start = skip_ws(bytes, i + kw.len());
                if kw == b"static" && super::lexer::keyword_starts_at(bytes, name_start, b"mut") {
                    name_start = skip_ws(bytes, name_start + b"mut".len());
                }
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
            while j < bytes.len() && bytes[j] != b'=' && bytes[j] != b';' {
                if bytes[j] == b'<' {
                    j = skip_angles(bytes, j);
                } else {
                    j += 1;
                }
            }
            if !name.is_empty() && bytes.get(j) == Some(&b'=') {
                if let Some(end) = alias_target_end(bytes, j + 1) {
                    let target = source[j + 1..end].trim();
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
///
/// **The body is read by [`super::lexer::scan_use_statement`], which is the declared single home for that
/// question.** This loop re-spelled it — `start = j + 3`, then `find(';')` — and so did not carry the
/// guard that home has: a `use` followed by `<` is a precise-capturing bound rather than an import, and
/// scanning to the next `;` there swallows the following real `use`. The extraction that made the reader
/// shared converged two of the three sites and walked past this one, which is the twin-drift class this
/// module's own siblings record. What differs here is the surrounding `pub` and visibility-qualifier
/// walk, and that is what this function keeps.
fn pub_use_statements(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut trees = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"pub") {
            let mut j = i + 3;
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
                match super::lexer::scan_use_statement(bytes, source, j) {
                    super::lexer::UseStatementScan::Statement { body, next } => {
                        trees.push(body);
                        i = next;
                        continue;
                    }
                    super::lexer::UseStatementScan::NotAStatement { resume_at } => {
                        i = resume_at;
                        continue;
                    }
                    super::lexer::UseStatementScan::Unterminated => break,
                }
            }
        }
        i += 1;
    }
    trees
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
        "crate" => fold_canonical_segments(&parts_str)?,
        "self" | "super" => resolve_self_super(current_module, &parts_str)?,
        other => {
            if let Some(target) = use_map.get(other) {
                let mut base = target.clone();
                for seg in rest {
                    base.push_str("::");
                    base.push_str(seg);
                }
                base
            } else if external.is_some_and(|v| {
                head_is_external_dependency(other, occurrence_module, root_modules, v)
            }) {
                parts.join("::")
            } else {
                format!("{current_module}::{}", parts.join("::"))
            }
        }
    };
    Some(base)
}

/// Chase a candidate path through the `type`-alias / `pub use` closure to a fixpoint: repeatedly
/// replace the longest local-name prefix that is a `defs` key with its target. Cycle-safe via the
/// visited set, capped at 256 iterations to guarantee termination on self-referential definitions.
fn chase_closure(
    path: &str,
    defs: &HashMap<String, String>,
    visited: &mut HashSet<String>,
) -> String {
    let mut current = path.to_string();
    for _ in 0..256 {
        if !visited.insert(current.clone()) {
            return current;
        }
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
    current
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
        return false;
    }
    if path_within(glob, prefix) || path_within(prefix, glob) {
        return true;
    }
    for (name, target) in &ctx.defs {
        if path_within(name, glob) {
            let resolved = chase_closure(target, &ctx.defs, &mut HashSet::new());
            if path_within(&resolved, prefix) {
                return true;
            }
        }
    }
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
        "std" | "core" | "alloc" => Some(parts.join("::")),
        "crate" => fold_canonical_segments(&parts_str),
        _ if global => Some(parts.join("::")),
        "self" | "super" => resolve_self_super(current_module, &parts_str),
        other => {
            if is_crate_root_shadow(current_module, other, root_modules) {
                let mut out = vec!["crate".to_string()];
                out.extend(parts.iter().cloned());
                Some(out.join("::"))
            } else {
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

/// Decision helper for whether an occurrence (call or path mention) triggers a boundary reaction.
///
/// Order of evaluation:
/// 1. `strict`: any path under the prefix reacts (whether call or mention).
/// 2. Default (!strict): only calls react; a non-call mention passes.
/// 3. If narrowed by read verbs (`verbs`), the terminal segment must match a declared verb leaf-exact.
/// 4. Otherwise, every call under the prefix reacts.
fn should_react_on_occurrence(
    strict: bool,
    is_call: bool,
    verbs: Option<&[String]>,
    resolved: &str,
) -> bool {
    if strict {
        true
    } else if !is_call {
        false
    } else if let Some(verbs) = verbs {
        let leaf = resolved
            .rsplit_once("::")
            .map_or(resolved, |(_, leaf)| leaf);
        verbs.iter().any(|v| v == leaf)
    } else {
        true
    }
}
