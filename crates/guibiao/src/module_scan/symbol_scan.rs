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
    SymbolPrefix, canonical_module_path, canonical_segment, fold_canonical_segments,
    is_crate_root_shadow, path_within, resolve_self_super, scan_inline_modules,
};
use super::scope_graph::{
    CrateScopes, Head, Namespace, PathRoots, ScopeTable, expand_use_leaves, extern_block_brace_at,
    glob_bases, resolve_written_path, skip_angles, skip_ws,
};

/// The crate-wide resolution context, built once from every reachable file: the local definition
/// closure (`type` aliases and `pub use` re-exports, keyed by their fully-qualified local name →
/// target path) and the glob re-exports (a module that `pub use`-globs another path). Each file's
/// lexical scopes are a [`ScopeTable`], built once per file.
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
/// forbidden). `prefix` is the confined prefix in its one canonical form; `ending_with` narrows to read verbs;
/// `strict` reacts on any mention, not only calls; `external` opts in the strict-external head
/// ladder (a fully-qualified un-`use`d head matching a declared dependency reclassifies as
/// external); `dependency_names` are the rename-aware declared-dependency import identifiers that
/// ladder matches against (unused when `external` is false); `edition_2015` makes a `use` path and a
/// `::`-rooted path start at the crate root. Returns findings sorted + deduped by `finding`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn inline_symbol_findings(
    all_files: &[(std::path::PathBuf, String)],
    governed: &[(std::path::PathBuf, String)],
    root_modules: &[String],
    prefix: &SymbolPrefix,
    ending_with: Option<&[String]>,
    strict: bool,
    external: bool,
    dependency_names: &[String],
    edition_2015: bool,
) -> Result<Vec<InlineFinding>, String> {
    let roots = PathRoots {
        root_modules,
        edition_2015,
    };
    let prefix = prefix.path.as_str();
    let verbs: Option<Vec<String>> =
        ending_with.map(|vs| vs.iter().map(|v| canonical_module_path(v)).collect());

    let mut ctx = ResolveCtx {
        defs: HashMap::new(),
        glob_reexports: Vec::new(),
    };
    let mut file_text: HashMap<std::path::PathBuf, String> = HashMap::new();
    let mut tables: HashMap<std::path::PathBuf, ScopeTable> = HashMap::new();
    let mut use_maps: HashMap<std::path::PathBuf, HashMap<(String, String), String>> =
        HashMap::new();
    let mut module_paths: HashSet<String> = HashSet::new();
    let mut item_defs: HashSet<String> = HashSet::new();
    for (file, module) in all_files {
        let raw = std::fs::read_to_string(file)
            .map_err(|err| crate::errors::unreadable_governed_file_error(file, &err.to_string()))?;
        let call_text = strip_comments_and_strings(&raw);
        let decl_text = strip_macro_bodies(&call_text);
        let table = ScopeTable::build(&call_text, module, roots)?;
        let use_map = table.module_bindings();
        collect_defs(&decl_text, module, &table, roots, &use_map, &mut ctx)?;
        module_paths.insert(module.clone());
        collect_item_definition_names(module, &decl_text, &mut item_defs);
        use_maps.insert(file.clone(), use_map);
        tables.insert(file.clone(), table);
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
    let crate_scopes = CrateScopes::new(tables.values(), item_defs.clone());

    let mut findings: Vec<InlineFinding> = Vec::new();
    for (file, module) in governed {
        let raw = &file_text[file];
        let use_map = &use_maps[file];
        let table = &tables[file];
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(raw));
        let mut chase_defs = ctx.defs.clone();
        for ((use_module, alias), target) in use_map {
            chase_defs
                .entry(format!("{use_module}::{alias}"))
                .or_insert_with(|| target.clone());
        }

        for (glob_path, occurrence_module) in glob_import_paths(&decl_text, module)? {
            let reaches = resolve_head(
                &glob_path,
                &occurrence_module,
                None,
                use_map,
                roots,
                external_vocab.as_ref(),
            )
            .iter()
            .any(|resolved| glob_reaches_prefix(resolved, prefix, &ctx, &mut HashSet::new()));
            if reaches {
                findings.push(InlineFinding {
                    fact: ModuleFact::InlineGlob {
                        path: glob_path,
                        module: module.clone(),
                    },
                    file: file.display().to_string(),
                });
            }
        }

        let call_text = strip_comments_and_strings(raw);
        for occurrence in path_occurrences(&call_text, module) {
            for resolved in resolve_head(
                &occurrence.segments,
                &occurrence.module,
                Some((table, table.scope_at(occurrence.at), &crate_scopes)),
                use_map,
                roots,
                external_vocab.as_ref(),
            ) {
                let resolved = chase_closure(&resolved, &chase_defs, &mut HashSet::new());
                if !path_within(&resolved, prefix) {
                    continue;
                }
                if should_react_on_occurrence(
                    strict,
                    occurrence.is_call,
                    verbs.as_deref(),
                    &resolved,
                ) {
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
    /// The byte, in the scanned text, where the path starts: what the [`ScopeTable`] places it by.
    at: usize,
    segments: String,
    is_call: bool,
    module: String,
}

/// Scan all call and path-mention occurrences in `source`.
///
/// Every occurrence carries its true (inline) module, tracked by inline
/// `mod name { … }` nesting exactly as [`super::use_scan`]'s walk does (non-`mod` braces move the
/// depth but never touch the stack, so a call anywhere inside `mod tests { … }` attributes to
/// `…::tests`). The caller's `external` mode remains a resolution policy, not a lexical-module
/// attribution policy.
fn path_occurrences(source: &str, base_module: &str) -> Vec<PathOccurrence> {
    let bytes = source.as_bytes();
    let mut out = Vec::new();
    let inline_modules = scan_inline_modules(source, base_module);
    let mut i = 0;
    while i < bytes.len() {
        if matches!(bytes[i], b'{' | b'}') {
            i += 1;
            continue;
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
        let mut segments = normalize_segments(&bytes[start..end]);
        match root_form(bytes, i) {
            RootForm::Plain => {}
            RootForm::Rooted => segments = format!("::{segments}"),
            RootForm::Qualified => {
                i = end.max(i + 1);
                continue;
            }
        }
        let is_call = is_call_application(bytes, end) && !names_a_fn_definition(bytes, i);
        if segments.contains("::") || is_call {
            let module = inline_modules.modules[inline_modules.contexts[i] as usize].clone();
            out.push(PathOccurrence {
                at: start,
                segments,
                is_call,
                module,
            });
        }
        i = end.max(i + 1);
    }
    out
}

/// How a path starting at an identifier is rooted, read from what precedes it.
enum RootForm {
    /// No `::` before the head: the head is looked up from the path's scope.
    Plain,
    /// A `::` standing where a path begins: the head names a crate.
    Rooted,
    /// A `::` after the `>` closing a `<…>` group — the tail of `<T>::f` or `<T as Trait>::f`, whose item the
    /// type chooses. It has no crate-root head, and resolving it needs the type inference the scanner does not
    /// perform, so it stays unresolved under the receiver-method bound.
    Qualified,
}

/// The [`RootForm`] of a path whose head starts at `i`. A `::` directly before the head roots the path unless it
/// follows an identifier (a continuation this walk reads as the same path) or a `>` that closes a `<…>` group in
/// the same statement; the `>` of `->` and `=>`, and a `>` with no `<` to close, are comparisons or arrows before
/// which a path begins.
fn root_form(bytes: &[u8], i: usize) -> RootForm {
    let mut p = i;
    while p > 0 && bytes[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    if p < 2 || bytes[p - 1] != b':' || bytes[p - 2] != b':' {
        return RootForm::Plain;
    }
    let mut before = p - 2;
    while before > 0 && bytes[before - 1].is_ascii_whitespace() {
        before -= 1;
    }
    if before == 0 {
        return RootForm::Rooted;
    }
    match bytes[before - 1] {
        byte if is_ident_byte(byte) => RootForm::Plain,
        b'>' if closes_an_angle_group(bytes, before - 1) => RootForm::Qualified,
        _ => RootForm::Rooted,
    }
}

/// Whether the `>` at `close` closes a `<…>` group opened earlier in the same statement. The `>` of `->` and
/// `=>` closes nothing.
fn closes_an_angle_group(bytes: &[u8], close: usize) -> bool {
    let is_arrow = |k: usize| k > 0 && matches!(bytes[k - 1], b'-' | b'=');
    if is_arrow(close) {
        return false;
    }
    let mut depth = 0usize;
    let mut k = close + 1;
    while k > 0 {
        k -= 1;
        match bytes[k] {
            b'>' if !is_arrow(k) => depth += 1,
            b'<' => {
                depth -= 1;
                if depth == 0 {
                    return true;
                }
            }
            b';' | b'{' | b'}' => return false,
            _ => {}
        }
    }
    false
}

/// Whether the identifier at `i` is the name a `fn` item declares: its `name(` defines rather than calls.
fn names_a_fn_definition(bytes: &[u8], i: usize) -> bool {
    let mut p = i;
    while p > 0 && bytes[p - 1].is_ascii_whitespace() {
        p -= 1;
    }
    p >= 2 && super::lexer::keyword_starts_at(bytes, p - 2, b"fn")
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

/// Every glob import path in already-declaration-cleaned source: a `use <path>::*;` (bare) or a
/// grouped `use <path>::{ … * … };` (the `*` among the group members). Returns the module-path
/// `<path>` (without the trailing `::*`) and the true inline module enclosing the `use`.
fn glob_import_paths(source: &str, base_module: &str) -> Result<Vec<(String, String)>, String> {
    use super::lexer::UseStatementScan;
    let bytes = source.as_bytes();
    let inline_modules = scan_inline_modules(source, base_module);
    let mut paths = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"use") {
            match super::lexer::scan_use_statement(bytes, source, i) {
                UseStatementScan::Statement { body, next } => {
                    let mut bases = Vec::new();
                    glob_bases(&body, &mut bases, 0)?;
                    let module =
                        inline_modules.modules[inline_modules.contexts[i] as usize].clone();
                    paths.extend(bases.into_iter().map(|path| (path, module.clone())));
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
    Ok(paths)
}

/// Collect the `type`-alias and `pub use` re-export definitions of a file into the crate-wide
/// context, keyed by their fully-qualified local name. Only a `type` alias written directly in a
/// module body enters it — the table's [`ScopeTable::module_aliases`] — since one written in a block
/// or as an associated type is named by no path outside its block or its type. Targets are resolved
/// module-relative (through the file's `use_map`), so `type B = A;` targets the sibling
/// `crate::mod::A` and `use std::time::SystemTime; type Clock = SystemTime;` targets
/// `std::time::SystemTime`.
fn collect_defs(
    source: &str,
    module: &str,
    table: &ScopeTable,
    roots: PathRoots<'_>,
    use_map: &HashMap<(String, String), String>,
    ctx: &mut ResolveCtx,
) -> Result<(), String> {
    for (name, target, def_module) in table.module_aliases() {
        if let Some(canonical) = resolve_target(target, def_module, use_map, roots) {
            ctx.defs.insert(
                format!("{def_module}::{}", canonical_module_path(name)),
                canonical,
            );
        }
    }
    for (def_module, tree) in pub_use_statements(source, module) {
        let mut globs = Vec::new();
        glob_bases(&tree, &mut globs, 0)?;
        for base in globs {
            if let Some(canonical) = resolve_written_path(&base, &def_module, roots) {
                ctx.glob_reexports.push((def_module.clone(), canonical));
            }
        }
        for (alias, path) in expand_use_leaves(&tree)? {
            if let Some(canonical) = resolve_written_path(&path, &def_module, roots) {
                ctx.defs.insert(format!("{def_module}::{alias}"), canonical);
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
/// `strict_external`'s rustdoc and not re-argued here. A `fn` item's own name is never read as a call,
/// so an associated or nested `fn` named like the crate is not one of them.
fn collect_item_definition_names(module: &str, source: &str, out: &mut HashSet<String>) {
    const KEYWORDS: [&[u8]; 9] = [
        b"mod", b"struct", b"enum", b"union", b"trait", b"type", b"fn", b"const", b"static",
    ];
    collect_definition_names(module, source, &KEYWORDS, true, out);
}

/// Every item the files of one compilation unit define at their modules' top level, keyed
/// `{true_module}::{name}` — the set the strict-external ladder reads, gathered by the same
/// [`collect_item_definition_names`] over the same declaration-cleaned text.
///
/// It is what an inline-call prefix naming an item of the crate is held to. An item written inside a
/// macro body, or generated by one, is not in it, as it is not in the ladder's set.
pub(crate) fn local_item_definitions(
    files: &[(std::path::PathBuf, String)],
) -> Result<HashSet<String>, String> {
    let mut items = HashSet::new();
    for (file, module) in files {
        let raw = std::fs::read_to_string(file)
            .map_err(|err| crate::errors::unreadable_governed_file_error(file, &err.to_string()))?;
        let decl_text = strip_macro_bodies(&strip_comments_and_strings(&raw));
        collect_item_definition_names(module, &decl_text, &mut items);
    }
    Ok(items)
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

/// Collect item definition names declared in `source` for the given module.
///
/// The shared walk behind [`collect_item_definition_names`] and [`value_namespace_item_names`]:
/// module-top-level definitions introduced by any of `keywords`, keyed `{true_module}::{name}`.
/// `capture_inline_mod_names` decides whether an inline `mod x { … }`'s own name is itself recorded as
/// a definition of the enclosing module — wanted for the local-precedence ladder, not for the
/// value-namespace query, whose whole point is to distinguish the two namespaces.
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
    let inline_modules = scan_inline_modules(source, module);
    let mut i = 0;
    let mut depth = 0usize;
    let mut extern_opens: Vec<usize> = Vec::new();
    let mut site_index = 0usize;
    while i < bytes.len() {
        let module_top = inline_modules.module_tops[i];
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
        if inline_modules
            .sites
            .get(site_index)
            .is_some_and(|site| site.at == i)
        {
            let site = &inline_modules.sites[site_index];
            if capture_inline_mod_names && depth == site.module_top && !site.name.is_empty() {
                out.insert(format!(
                    "{}::{}",
                    inline_modules.modules[site.enclosing as usize], site.name
                ));
            }
            site_index += 1;
            i = site.brace;
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
                        out.insert(format!(
                            "{}::{name}",
                            inline_modules.modules[inline_modules.contexts[i] as usize]
                        ));
                    }
                }
            }
        }
        i = end_of_ident(bytes, i);
    }
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
fn pub_use_statements(source: &str, base_module: &str) -> Vec<(String, String)> {
    let bytes = source.as_bytes();
    let inline_modules = scan_inline_modules(source, base_module);
    let mut trees = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if super::lexer::keyword_starts_at(bytes, i, b"pub") {
            let module = inline_modules.modules[inline_modules.contexts[i] as usize].clone();
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
                        trees.push((module, body));
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

/// Resolve the head of a written path occurrence (its `::`-joined `segments`) to every canonical path
/// it can name — empty when it names none a prefix can reach.
///
/// A `std`/`core`/`alloc` head is literal and a `crate`/`self`/`super` head is local. Any other
/// `::`-rooted head names an external crate, except in edition 2015, where it starts at the crate root;
/// an external crate is observed under `.strict_external()` only, the same answer its bare spelling
/// gets, so `dep::f()` and `::dep::f()` are reported in the same mode. Any other head
/// is looked up first in `scoped` — the file's [`ScopeTable`], the scope the occurrence stands in, and
/// the crate's [`CrateScopes`] its globs are followed through — which answers with every binding of
/// the nearest scope that binds it; a block-local item is named by no path a prefix can reach, so it
/// resolves to nothing. A head the
/// table leaves unbound, or one read without a scope (a glob's own path, read from its module), is
/// looked up in `use_map`, the module-scope `use` bindings, and otherwise treated as a local item of
/// the occurrence's module (so a `type`/`pub use` closure can then rewrite it). Leaf-only matching of
/// an unresolved head is deliberately NOT done.
///
/// `external` is `Some` ONLY under `.strict_external()`: an un-`use`d bare head that matches a
/// declared dependency (and is not locally shadowed — see [`head_is_external_dependency`]) is then
/// kept as the literal external path (`chrono::Utc::…`) instead of the fake-local
/// `{module}::chrono::Utc::…`, closing the fully-qualified-external false negative. When `external`
/// is `None` the default path is unchanged: every non-`use` bare head falls to the load-bearing
/// `{module}::…` fallback the `type`-alias / re-export closure depends on.
///
/// `occurrence_module` is the occurrence's true (inline) module (`{file_module}::inner…`): relative
/// `self`/`super` resolution, the module-scope lookup, the fallback and the strict-external
/// local-shadow ladder all read from it, so a file-top item cannot mask an external call in an inline
/// submodule, and a submodule-local item shadows only its own module.
fn resolve_head(
    segments: &str,
    occurrence_module: &str,
    scoped: Option<(&ScopeTable, u32, &CrateScopes)>,
    use_map: &HashMap<(String, String), String>,
    roots: PathRoots<'_>,
    external: Option<&ExternalVocab>,
) -> Vec<String> {
    let raw = segments.trim();
    let global = raw.starts_with("::");
    let raw = raw.trim_start_matches("::");
    let parts: Vec<String> = raw
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let Some((head, rest)) = parts.split_first() else {
        return Vec::new();
    };
    let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
    let base: Option<String> = match head.as_str() {
        "std" | "core" | "alloc" => Some(parts.join("::")),
        "crate" => fold_canonical_segments(&parts_str),
        "self" | "super" if !global => resolve_self_super(occurrence_module, &parts_str),
        other if global => {
            if roots.edition_2015 && roots.names_root_module("crate", other) {
                Some(format!("crate::{}", parts.join("::")))
            } else {
                external.map(|_| parts.join("::"))
            }
        }
        other => {
            let namespace = if rest.is_empty() {
                Namespace::Value
            } else {
                Namespace::Type
            };
            let lexical = scoped.map_or(Head::Unbound, |(table, scope, crate_scopes)| {
                table.resolve(scope, other, rest, namespace, crate_scopes)
            });
            match lexical {
                Head::Paths(paths) => return paths,
                Head::Local => None,
                Head::Unbound => Some(
                    if let Some(target) =
                        use_map.get(&(occurrence_module.to_string(), other.to_string()))
                    {
                        let mut base = target.clone();
                        for seg in rest {
                            base.push_str("::");
                            base.push_str(seg);
                        }
                        base
                    } else if external.is_some_and(|v| {
                        head_is_external_dependency(other, occurrence_module, roots.root_modules, v)
                    }) {
                        parts.join("::")
                    } else {
                        format!("{occurrence_module}::{}", parts.join("::"))
                    },
                ),
            }
        }
    };
    base.into_iter().collect()
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

/// Resolve a `type`-alias / re-export **target** module-relative: a bare head is looked up in the
/// file's `use_map`; a `std`/`core`/`alloc`/`crate` head is literal; `self`/`super` resolve as a
/// path; any other bare head is a **local** item of the current module (so `type B = A;` targets
/// `{module}::A`, chaining through the closure). Contrast [`resolve_written_path`], which treats a
/// bare `use`-path head as an external crate.
fn resolve_target(
    target: &str,
    module: &str,
    use_map: &HashMap<(String, String), String>,
    roots: PathRoots<'_>,
) -> Option<String> {
    let raw = target.trim();
    if raw.starts_with("::") {
        return resolve_written_path(raw, module, roots);
    }
    let parts: Vec<String> = raw
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let (head, rest) = parts.split_first()?;
    if let Some(mapped) = use_map.get(&(module.to_string(), head.to_string())) {
        let mut base = mapped.clone();
        for seg in rest {
            base.push_str("::");
            base.push_str(seg);
        }
        return Some(base);
    }
    match head.as_str() {
        "std" | "core" | "alloc" | "crate" => Some(parts.join("::")),
        "self" | "super" => resolve_written_path(raw, module, roots),
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
