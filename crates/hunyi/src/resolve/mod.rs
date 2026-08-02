//! 渾儀's shared **name-resolution** layer — the dimension-internal facility both
//! semantic capabilities turn on.
//!
//! Resolution is *observation*, not reaction: it reads source structure to map a path
//! as written into a canonical `crate::…` path. It therefore lives **here, in the
//! semantic dimension** — not in 璇璣 (`xuanji`), the dimension-agnostic reaction model that
//! renders no verdict (the measure, not an observing dimension); and not shared with 圭表
//! (`guibiao`), whose token scanner must stay `syn`-free to keep the dependency-light core. The two
//! resolvers are intentionally separate (a PROJECT.md decision); this one is `syn`-based.
//!
//! It resolves a name three ways and bounds the rest honestly:
//! - an in-scope `use` (including renamed and path-qualified), and `crate::`/`self`/`super`;
//! - a **bare or relative name against the current module** (a same-module item needs no
//!   `use`) — opt-in via [`BareFallback`], because exposure-governance wants a bare local
//!   name *ignored* while impl-locality must resolve it (the bare name *is* the anchor);
//! - following **local `pub use` re-export chains**, so a path reached through a facade
//!   matches the item it denotes.
//!
//! Out of scope (stated bounds, never a silent claim): glob imports, macro-generated
//! names, and cross-crate re-exports.

use std::collections::{HashMap, HashSet};

mod shape;
pub(crate) use shape::*;

/// Each name a `use` brings into a module's scope mapped to **every** written full path declared
/// for that name — almost always exactly one, but two mutually-exclusive `#[cfg]`-gated `use ...
/// as Name;` declarations in the same file are never compiled together, so both candidate targets
/// are kept (cfg-blind: observation cannot know which is live) rather than the later declaration
/// silently overwriting the earlier one. Mirrors [`AliasMap`]'s existing multi-valued shape.
pub(crate) type UseMap = HashMap<String, Vec<String>>;

/// A `pub use` re-export closure: an alias's canonical path → **every** canonical path it
/// re-exports (mirrors [`UseMap`]'s multi-valued shape for the identical cfg-blind reason: two
/// mutually-exclusive `pub use ... as X;` targets are both kept). Following it to a fixpoint
/// canonicalizes a facade path to the item(s) it denotes.
pub(crate) type ReexportMap = HashMap<String, Vec<String>>;

/// A type-alias closure: a `type X = <path>;` alias's canonical path (`{module}::X`) → the
/// canonical paths of its nominal targets. Followed together with the re-export closure to a
/// fixpoint, so a forbidden type reached through an alias resolves to its defining path.
pub(crate) type AliasMap = HashMap<String, Vec<String>>;

/// Whether a bare/relative name (not in the `use`-map, not `crate`/`self`/`super`)
/// resolves against the current module, or is left unresolved.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum BareFallback {
    /// Leave a bare name unresolved (`None`) — exposure-governance's choice: a bare
    /// local name is not the cross-module forbidden type, and resolving it risks a
    /// same-module false positive.
    Ignore,
    /// Resolve a bare name against the current module (`crate::module::Name`) — impl-
    /// locality's choice: the bare name *is* the anchored trait, so leaving it
    /// unresolved would be a false negative.
    CurrentModule,
}

/// Strip a raw-identifier prefix so `r#type` compares as `type`.
pub(crate) fn strip_raw(ident: &str) -> String {
    ident.strip_prefix("r#").unwrap_or(ident).to_string()
}

/// Canonicalize a `::`-delimited path so each raw-identifier segment compares as its
/// plain form.
pub(crate) fn canonical_path_str(path: &str) -> String {
    path.split("::")
        .map(strip_raw)
        .collect::<Vec<_>>()
        .join("::")
}

/// Map each name a `use` brings into the module's scope to its full written path
/// (`use a::b::C` → `C → a::b::C`; `use a::b::C as D` → `D → a::b::C`; `use a::b` →
/// `b → a::b`). Glob imports bring no nameable leaf (a stated bound). Only the module's
/// own `use`s are collected — Rust modules do not inherit ancestor `use`s.
pub(crate) fn collect_uses(items: &[syn::Item]) -> UseMap {
    let mut map = UseMap::new();
    for item in items {
        if let syn::Item::Use(use_item) = item {
            collect_use_tree(&use_item.tree, String::new(), &mut map);
        }
    }
    map
}

/// Add `value` as a candidate for `key`, skipping an exact duplicate (two mutually-exclusive `#[cfg]`
/// branches declaring the byte-identical `use` produce nothing new to react to). Never overwrites —
/// the whole point of [`UseMap`]/[`ReexportMap`] being multi-valued is that a later declaration for
/// the same name never silently discards an earlier one.
fn push_candidate(map: &mut HashMap<String, Vec<String>>, key: String, value: String) {
    let candidates = map.entry(key).or_default();
    if !candidates.contains(&value) {
        candidates.push(value);
    }
}

fn collect_use_tree(tree: &syn::UseTree, prefix: String, map: &mut UseMap) {
    let join = |prefix: &str, ident: &str| {
        if prefix.is_empty() {
            ident.to_string()
        } else {
            format!("{prefix}::{ident}")
        }
    };
    match tree {
        syn::UseTree::Path(path) => {
            let ident = strip_raw(&path.ident.to_string());
            collect_use_tree(&path.tree, join(&prefix, &ident), map);
        }
        syn::UseTree::Name(name) => {
            let ident = strip_raw(&name.ident.to_string());
            if ident == "self" {
                // `use a::b::{self}` binds the prefix module itself under its final segment
                // (never the literal `self`) — mirror `walk_reexport_tree`'s reaction side so the
                // closure and the direct walk agree. A `self` under no prefix cannot arise from a
                // legal `use`.
                if let Some(last) = prefix.rsplit("::").next().filter(|s| !s.is_empty()) {
                    push_candidate(map, last.to_string(), prefix.clone());
                }
            } else {
                push_candidate(map, ident.clone(), join(&prefix, &ident));
            }
        }
        syn::UseTree::Rename(rename) => {
            let ident = strip_raw(&rename.ident.to_string());
            let alias = strip_raw(&rename.rename.to_string());
            if alias == "_" {
                // `as _` binds no nameable path — mirror `walk_reexport_tree`'s stated bound.
            } else if ident == "self" {
                // `use a::b::{self as x}` binds the prefix module itself, renamed.
                if !prefix.is_empty() {
                    push_candidate(map, alias, prefix.clone());
                }
            } else {
                push_candidate(map, alias, join(&prefix, &ident));
            }
        }
        // A glob brings no nameable leaf into the map — a documented out-of-scope bound.
        syn::UseTree::Glob(_) => {}
        syn::UseTree::Group(group) => {
            for item in &group.items {
                collect_use_tree(item, prefix.clone(), map);
            }
        }
    }
}

/// Resolve `crate::`/`self`/`super`-rooted segments to an absolute `crate::…` path,
/// relative to `module` (e.g. `crate::domain`). `None` when the head is not one of those
/// (a `use`-head or bare name — resolved elsewhere). Over-popping past the crate root is
/// unresolvable.
fn resolve_crate_relative(segs: &[String], module: &str) -> Option<String> {
    let head = segs.first()?;
    match head.as_str() {
        "crate" => Some(segs.join("::")),
        "self" | "super" => {
            let mut parts: Vec<&str> = module.split("::").collect();
            let mut i = 0;
            while i < segs.len() {
                match segs[i].as_str() {
                    "self" => i += 1,
                    "super" => {
                        if parts.len() <= 1 {
                            return None;
                        }
                        parts.pop();
                        i += 1;
                    }
                    _ => break,
                }
            }
            let rest = &segs[i..];
            if rest.is_empty() {
                Some(parts.join("::"))
            } else {
                Some(format!("{}::{}", parts.join("::"), rest.join("::")))
            }
        }
        _ => None,
    }
}

/// Resolve a path as written (in a signature or an `impl` header) to a canonical crate
/// path, using the module's in-scope `use`s, `crate::`/`self`/`super` relative to
/// `module`, and — per `bare` — a bare/relative name against the current module. `None`
/// when not resolvable (a glob/external/primitive name under [`BareFallback::Ignore`]) —
/// a stated bound, never a silent claim. Takes the **first** use-map candidate when the head
/// resolves to more than one (a mutually-exclusive `#[cfg]` collision) — the identity/anchor
/// callers of this function (impl-locality, trait-impl anchoring, marker acquisition) have no
/// audit-verified need for cfg-blind multi-candidate resolution; [`resolve_path_all`] is the
/// exposure-matching sibling that checks every candidate.
pub(crate) fn resolve_path(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    bare: BareFallback,
) -> Option<String> {
    resolve_path_all(path, uses, module, bare)
        .into_iter()
        .next()
}

/// [`resolve_path`]'s cfg-blind sibling: returns **every** candidate canonical path the head could
/// resolve to, instead of only the first. Exposure-matching callers (signature-coupling and the
/// shared existential-exposure pipeline) must check every candidate and react if any is forbidden —
/// observation cannot know which of two mutually-exclusive `#[cfg]` branches' `use` declarations is
/// live, so neither may be silently dropped in favor of the other.
pub(crate) fn resolve_path_all(
    path: &syn::Path,
    uses: &UseMap,
    module: &str,
    bare: BareFallback,
) -> Vec<String> {
    let segs: Vec<String> = path
        .segments
        .iter()
        .map(|s| strip_raw(&s.ident.to_string()))
        .collect();
    let Some(head) = segs.first() else {
        return Vec::new();
    };

    if let Some(canonical) = resolve_crate_relative(&segs, module) {
        return vec![canonical];
    }
    match uses.get(head) {
        Some(candidates) => candidates
            .iter()
            .map(|full| {
                let rest = &segs[1..];
                let combined = if rest.is_empty() {
                    full.clone()
                } else {
                    format!("{full}::{}", rest.join("::"))
                };
                // The use-target may itself be `crate`/`self`/`super`-relative (e.g.
                // `use super::x::Y`); canonicalize it against the module so it compares as an
                // absolute path. A bare-headed target (an external crate, edition 2018+) is
                // left as written — it cannot match a local anchor/forbidden path anyway.
                let combined_segs: Vec<String> = combined.split("::").map(strip_raw).collect();
                resolve_crate_relative(&combined_segs, module).unwrap_or(combined)
            })
            .collect(),
        None => match bare {
            BareFallback::Ignore => Vec::new(),
            // A name needs no `use` in its own module: resolve against `module`.
            BareFallback::CurrentModule => {
                vec![if module.is_empty() {
                    format!("crate::{}", segs.join("::"))
                } else {
                    format!("{module}::{}", segs.join("::"))
                }]
            }
        },
    }
}

/// If the first segment of a written path names an **external crate** — a declared
/// dependency or a sysroot crate, per the `externs` set — the path denotes an external item
/// and canonicalizes to **itself**, verbatim. This is the exposure pipeline's bounded oracle
/// for "is this bare head extern?": it is applied only *after* `use`-map and
/// `crate`/`self`/`super` resolution have declined (so a local `use … as <dep>` alias still
/// wins), and only by the exposure resolve and the re-export closure — never by
/// [`resolve_path`]'s other callers. An `externs` that is empty (the non-exposure callers)
/// makes it inert, so the closure behaves exactly as before.
pub(crate) fn extern_verbatim_segs(segs: &[String], externs: &HashSet<String>) -> Option<String> {
    let head = segs.first()?;
    externs.contains(head).then(|| segs.join("::"))
}

/// A source-level `extern crate X as Y;` rename closure: a crate-root alias `Y` → the real crate
/// `X`. Read from the local AST (unlike a Cargo-manifest `package =` rename, which the extern set
/// already folds in via `.rename`), so a renamed head resolves to the real crate.
pub(crate) type ExternRenameMap = HashMap<String, String>;

/// [`extern_verbatim_segs`] with a source-level crate-root `extern crate X as Y;` rename applied to the
/// head. A head `Y` known in `renames` is rewritten to the real crate `X` and returned **verbatim,
/// without the extern-set membership check**: a rename alias is, by grammar, never also a local
/// child module (two items named `Y` in one scope do not compile), so the caller's per-module
/// child-module shadow can never apply to it, and its target `X` is by definition a declared extern
/// crate. Checking the *renamed* head against a (possibly shadowed) set would wrongly drop it when a
/// child module happens to share `X`'s name. A head **not** in `renames` keeps the normal membership
/// check against `externs`, so this is identical to [`extern_verbatim_segs`] when `renames` is empty.
pub(crate) fn extern_verbatim_renamed(
    path: &syn::Path,
    externs: &HashSet<String>,
    renames: &ExternRenameMap,
) -> Option<String> {
    let mut segs: Vec<String> = path
        .segments
        .iter()
        .map(|s| strip_raw(&s.ident.to_string()))
        .collect();
    if let Some(real) = segs.first().and_then(|h| renames.get(h)).cloned() {
        segs[0] = real;
        return Some(segs.join("::"));
    }
    extern_verbatim_segs(&segs, externs)
}

/// Rewrite the **crate-relative spelling** of a crate-root `extern crate X as Y;` rename:
/// `crate::Y::rest` → `X::rest`. `crate::Y` unambiguously names the crate-root extern rename — a
/// crate-root `mod Y` cannot coexist with `extern crate … as Y` (E0260) — so no local shadow
/// applies and the rewrite is unconditional. Only the segment **immediately** after `crate` is the
/// alias: a deeper `crate::m::Y::…` is a submodule item, left unchanged. A canonical whose head is
/// not `crate`, or whose second segment is not a rename alias, is returned unchanged. Applied to the
/// **final** canonical (after alias/re-export closure), so `crate::Y::…` reached directly, through a
/// `type` alias, or through a `pub use` target is rewritten alike.
pub(crate) fn apply_crate_root_rename(canonical: String, renames: &ExternRenameMap) -> String {
    let segs: Vec<&str> = canonical.split("::").collect();
    if segs.len() >= 2 && segs[0] == "crate" {
        if let Some(real) = renames.get(segs[1]) {
            let mut out = vec![real.as_str()];
            out.extend_from_slice(&segs[2..]);
            return out.join("::");
        }
    }
    canonical
}

/// Rewrite a **bare** crate-root `extern crate X as Y;` alias head on a final canonical:
/// `Y::rest → X::rest` when the head `Y` is a rename alias in `renames`. The sibling of
/// [`apply_crate_root_rename`] for the *bare* spelling: a forbidden type imported by a private
/// `use Y::…;` resolves through the use-map to `Y::…` verbatim (the use-map never consults the
/// rename map), so without this the aliased import spelling is a false negative that the direct
/// type-position spelling (rewritten by the extern oracle) avoids. Callers pass the
/// child-mod-shadowed map (`renames_bare`), so a head shadowed by a local `mod Y` is not rewritten.
/// A `crate`/`self`/`super`-rooted or non-alias head is returned unchanged; applied to the **final**
/// canonical (after the alias/re-export closure) so an aliased or facaded `Y::…` is rewritten alike.
pub(crate) fn apply_bare_alias_rename(canonical: String, renames: &ExternRenameMap) -> String {
    let mut segs: Vec<&str> = canonical.split("::").collect();
    if let Some(real) = segs.first().and_then(|h| renames.get(*h)) {
        segs[0] = real.as_str();
        return segs.join("::");
    }
    canonical
}

/// The `extern crate X as Y;` rename map with any alias `Y` shadowed by a same-named child `mod Y`
/// removed. A crate-root `extern crate X as Y;` binds `Y` crate-wide, but a module that declares its
/// own child `mod Y` shadows the alias there (rustc resolves a **bare** `Y::…` to the local module),
/// so a bare head must not be rewritten to the crate under such a module. The crate-relative
/// (`crate::Y::…`) and leading-`::` forms are not shadowable and keep the full map. With `child_mods`
/// empty this is byte-identical to `renames`. One source for the three sites that must not drift:
/// [`module_findings`](crate::module_findings)' bare-head rewrite, [`collect_reexports`]' facade
/// closure, and the operand principal resolver.
pub(crate) fn renames_shadowed(
    renames: &ExternRenameMap,
    child_mods: &HashSet<String>,
) -> ExternRenameMap {
    renames
        .iter()
        .filter(|(alias, _)| !child_mods.contains(*alias))
        .map(|(a, b)| (a.clone(), b.clone()))
        .collect()
}

/// Collect the **local** `pub use` (and `pub(crate)`/`pub(in …)`) re-exports declared in
/// `items` (which live in `module`) into `out`, keyed by the alias's canonical path. A
/// glob contributes no local hop; a bare-headed target re-exporting an **external** crate
/// (head ∈ `externs`) is retained verbatim, so a local facade chain terminating at an
/// extern type canonicalizes to it. A private `use` is not collected — it is invisible from
/// other modules, so it can only be a same-module name already in that module's [`UseMap`].
///
/// A **bare** re-export head is shadowed by a same-named child module **of this defining module**
/// (rustc resolves `pub use dep::X;` under a child `mod dep` to the local module — E0432 if absent),
/// so it resolves against the extern set and the crate-root rename map with `child_mods` removed
/// (`externs − child_mods` / `renames − child_mods`) — closing the facade-closure sibling of the
/// direct head-shadow FP, since this crate-wide map is followed by every module's query. A
/// **leading-`::`** head (`pub use ::dep::X;`) is an unambiguous extern that no local module
/// shadows, so it keeps the raw sets: `collect_use_tree` walks `use_item.tree`, which carries no
/// leading colon, so the flag is read from the `ItemUse` here (mirroring the direct walker's
/// `push_reexport`, which preserves it for the same escape-hatch reason). This split mirrors
/// `module_findings`' `externs_reexport` / `renames_bare`; with `child_mods` empty the map is
/// byte-identical to the raw-set behavior.
pub(crate) fn collect_reexports(
    items: &[syn::Item],
    module: &str,
    externs: &HashSet<String>,
    child_mods: &HashSet<String>,
    renames: &ExternRenameMap,
    out: &mut ReexportMap,
) {
    let externs_bare: HashSet<String> = externs.difference(child_mods).cloned().collect();
    let renames_bare = renames_shadowed(renames, child_mods);
    for item in items {
        if let syn::Item::Use(use_item) = item {
            if matches!(use_item.vis, syn::Visibility::Inherited) {
                continue;
            }
            let mut local = UseMap::new();
            collect_use_tree(&use_item.tree, String::new(), &mut local);
            // A leading `::` on the `use` item marks an unambiguous extern head — unshadowed by any
            // same-named child `mod`, so it keeps the raw sets; a bare head uses the child-excluded
            // sets. The flag lives on `ItemUse`, not the `UseTree` `collect_use_tree` walked.
            let (head_externs, head_renames) = if use_item.leading_colon.is_some() {
                (externs, renames)
            } else {
                (&externs_bare, &renames_bare)
            };
            // `local` maps each name this ONE use item's tree brings in to its single written
            // target — Rust rejects two same-named leaves within one item (E0252), so `written`
            // never actually holds more than one candidate here; `collect_use_tree`'s multi-valued
            // shape only matters once `out` accumulates across items/branches below.
            for (name, written) in local {
                let alias = format!("{module}::{name}");
                for written in &written {
                    if let Some(target) =
                        canonicalize_use_target(written, module, head_externs, head_renames)
                    {
                        // Skip a self-referential entry (`target == alias`) and — critically — one
                        // whose alias key is a strict `::`-prefix of its own target (`pub use
                        // self::x::x;` → `crate::x -> crate::x::x`, a same-name value re-export
                        // nested under a same-named module). The latter is meaningless for
                        // type-path canonicalization (the module path `crate::x` still denotes the
                        // module; rewriting would fabricate a nonexistent `crate::x::x::…`) and,
                        // left in the map, makes `rewrite_longest_prefix` re-fire on its own
                        // monotonically-growing output forever — the exact-repeat `seen` guard
                        // cannot catch a never-repeating sequence.
                        if target != alias && !is_strict_path_prefix(&alias, &target) {
                            push_candidate(out, alias.clone(), target);
                        }
                    }
                }
            }
        }
    }
}

/// Canonicalize a `pub use` target written as `crate::`/`self`/`super`-rooted to an
/// absolute path; a bare-headed target whose head names an external crate (per `externs`), or a
/// crate-root `extern crate … as` rename alias (per `renames`), canonicalizes to the extern path
/// verbatim — so a facade chain terminating at an extern type, incl. one reached through a source
/// rename, canonicalizes to it. Any other bare head (a pre-2018 crate-root-relative local module,
/// an unknown name) is out of scope for the local closure — a stated bound.
fn canonicalize_use_target(
    written: &str,
    module: &str,
    externs: &HashSet<String>,
    renames: &ExternRenameMap,
) -> Option<String> {
    let segs: Vec<String> = written.split("::").map(strip_raw).collect();
    if let Some(real) = segs.first().and_then(|h| renames.get(h)).cloned() {
        let mut renamed = segs.clone();
        renamed[0] = real;
        return Some(renamed.join("::"));
    }
    resolve_crate_relative(&segs, module).or_else(|| extern_verbatim_segs(&segs, externs))
}

/// Whether `prefix` is a strict `::`-boundary prefix of `path` — `crate::a` of `crate::a::b`, but
/// not of the unrelated `crate::ab` (segment-boundary aware) nor of itself. Used to refuse a
/// re-export map entry that would let `rewrite_longest_prefix` re-fire on its own growing output.
fn is_strict_path_prefix(prefix: &str, path: &str) -> bool {
    path.len() > prefix.len() && path.starts_with(prefix) && path[prefix.len()..].starts_with("::")
}

fn rewrite_longest_alias_prefixes(path: &str, map: &AliasMap) -> Option<Vec<String>> {
    let segments: Vec<&str> = path.split("::").collect();
    for end in (1..=segments.len()).rev() {
        let prefix = segments[..end].join("::");
        if let Some(targets) = map.get(&prefix) {
            let tail = if end == segments.len() {
                ""
            } else {
                &path[prefix.len()..]
            };
            return Some(targets.iter().map(|t| format!("{t}{tail}")).collect());
        }
    }
    None
}

pub(crate) fn expand_canonical_paths(
    path: &str,
    aliases: &AliasMap,
    reexports: &ReexportMap,
) -> Vec<String> {
    if aliases.is_empty() && reexports.is_empty() {
        return vec![path.to_string()];
    }
    // Iterative post-order DFS over the alias/re-export graph.
    //
    // Each stack entry is `(node, returning, depth)`. On the first visit (`returning = false`) we push the
    // node's children (unresolved dependencies) and then push a `returning = true` sentinel that
    // fires only after all children have been resolved. Cycle detection uses `in_stack` (for exact
    // node cycles) and `depth >= max_steps` (where `max_steps = aliases.len() + reexports.len() + 1`,
    // the mathematical maximum path length in a non-looping rewrite system) to terminate self-growing
    // prefix loops (such as `crate::a -> crate::a::b`).
    let max_steps = aliases.len() + reexports.len() + 1;
    let mut memo: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();
    let mut in_stack: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut work: Vec<(String, bool, usize)> = vec![(path.to_string(), false, 0)];

    while let Some((current, returning, depth)) = work.pop() {
        if returning {
            in_stack.remove(&current);
            if memo.contains_key(&current) {
                continue;
            }
            let mut results = Vec::new();
            if let Some(targets) = rewrite_longest_alias_prefixes(&current, aliases) {
                for t in &targets {
                    let child = memo
                        .get(t.as_str())
                        .cloned()
                        .unwrap_or_else(|| vec![t.clone()]);
                    for r in child {
                        if !results.contains(&r) {
                            results.push(r);
                        }
                    }
                }
            } else if let Some(targets) = rewrite_longest_alias_prefixes(&current, reexports) {
                for t in &targets {
                    let child = memo
                        .get(t.as_str())
                        .cloned()
                        .unwrap_or_else(|| vec![t.clone()]);
                    for r in child {
                        if !results.contains(&r) {
                            results.push(r);
                        }
                    }
                }
            } else {
                results.push(current.clone());
            }
            memo.insert(current, results);
            continue;
        }

        // Pre-visit.
        if memo.contains_key(&current) {
            continue;
        }
        if in_stack.contains(&current) || depth >= max_steps {
            // Active-path cycle or self-growing prefix loop cap reached.
            continue;
        }

        in_stack.insert(current.clone());

        // Push the returning sentinel first (processed after all children).
        work.push((current.clone(), true, depth));

        // Push unresolved children (reversed so the first target is processed first).
        if let Some(targets) = rewrite_longest_alias_prefixes(&current, aliases) {
            for target in targets.into_iter().rev() {
                if !memo.contains_key(&target) {
                    work.push((target, false, depth + 1));
                }
            }
        } else if let Some(targets) = rewrite_longest_alias_prefixes(&current, reexports) {
            for target in targets.into_iter().rev() {
                if !memo.contains_key(&target) {
                    work.push((target, false, depth + 1));
                }
            }
        }
    }

    memo.remove(path).unwrap_or_else(|| vec![path.to_string()])
}

/// The alias's targets as **bare nominal paths** — collects all `syn::Path` targets contained in
/// `ty`, iteratively walking non-generic compound type constructors (`Type::Reference`,
/// `Type::Ptr`, `Type::Tuple`, `Type::Slice`, `Type::Array`, `Type::Group`, `Type::Paren`). Any
/// path with `qself` or generic arguments (`Vec<T>`) is skipped — a stated coverage bound, never a
/// silent claim.
pub(crate) fn alias_nominal_targets<'a>(ty: &'a syn::Type, acc: &mut Vec<&'a syn::Path>) {
    let mut pending = vec![ty];
    while let Some(ty) = pending.pop() {
        match ty {
            syn::Type::Path(tp) => {
                if tp.qself.is_none()
                    && tp
                        .path
                        .segments
                        .iter()
                        .all(|s| matches!(s.arguments, syn::PathArguments::None))
                {
                    acc.push(&tp.path);
                }
            }
            syn::Type::Reference(tr) => pending.push(&tr.elem),
            syn::Type::Ptr(tp) => pending.push(&tp.elem),
            syn::Type::Tuple(tt) => pending.extend(tt.elems.iter().rev()),
            syn::Type::Slice(ts) => pending.push(&ts.elem),
            syn::Type::Array(ta) => pending.push(&ta.elem),
            syn::Type::Group(tg) => pending.push(&tg.elem),
            syn::Type::Paren(tp) => pending.push(&tp.elem),
            _ => {}
        }
    }
}

/// A bare, single-segment exposed path (`H`) that names a local `type` alias in `module`
/// (`{module}::H` ∈ `aliases`) resolves to that alias's canonical path, so the alias fixpoint
/// can expand it. `None` for any other shape — a multi-segment path (a type alias cannot be a
/// path prefix), a leading-`::` or generic-argument-bearing path, or a name that is not a known
/// alias — leaving the existing extern / re-export resolution unchanged. Ordered **before**
/// `extern_verbatim` at the call site so a local alias shadows a same-named extern crate (Rust's
/// own resolution); `extern_verbatim` stays meaningful for a multi-segment `dep::Foo`.
pub(crate) fn bare_local_alias(
    path: &syn::Path,
    module: &str,
    aliases: &AliasMap,
) -> Option<String> {
    bare_single_segment_ident(path)
        .map(|n| format!("{module}::{n}"))
        .filter(|key| aliases.contains_key(key))
}

/// The raw-stripped ident of a bare, single-segment, non-generic path (`H`, `r#H`) — the shared
/// syntactic guard behind [`bare_local_alias`] and the scan's alias-target resolver. `None` for a
/// leading-`::`, multi-segment, or generic-argument-bearing path. Each caller forms its own
/// `{module}::{ident}` key and applies its own (deliberately different) membership check.
pub(crate) fn bare_single_segment_ident(path: &syn::Path) -> Option<String> {
    if path.leading_colon.is_some() || path.segments.len() != 1 {
        return None;
    }
    let seg = &path.segments[0];
    if !matches!(seg.arguments, syn::PathArguments::None) {
        return None;
    }
    Some(strip_raw(&seg.ident.to_string()))
}

#[cfg(test)]
mod tests {
    use super::alias_nominal_targets;

    #[test]
    fn deeply_nested_alias_targets_use_a_bounded_native_stack() {
        const DEPTH: usize = 32_768;
        let mut ty: syn::Type = syn::parse_quote!(Leaf);
        for _ in 0..DEPTH {
            ty = syn::Type::Paren(syn::TypeParen {
                paren_token: syn::token::Paren::default(),
                elem: Box::new(ty),
            });
        }
        // Leak the adversarially deep fixture so its recursive syn-owned drop cannot become the
        // native-stack behavior under test; production AST lifetime is owned by the parsed file.
        let ty = Box::leak(Box::new(ty));
        let mut targets = Vec::new();

        alias_nominal_targets(ty, &mut targets);

        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].segments[0].ident, "Leaf");
    }
}
