//! Shared path vocabulary for the source scanner — the small foundation every reader above the token tree
//! stands on: raw-identifier canonicalization, `::`-delimited containment, `self`/`super` folding, the spelling
//! of a block's path segment, and the prefix spelling a confinement is declared in. Pure string processing, an
//! identifier read by Unicode's `XID_Start` and `XID_Continue` and compared in NFC, and no model type.

use super::token_tree::Edition;
use unicode_normalization::UnicodeNormalization;

/// Canonicalize one path segment by stripping a leading raw-identifier marker
/// (`r#name` -> `name`). Rust resolves `mod r#type;` to the source file `type.rs`,
/// so the file-derived path, the `mod` declaration, and a `use r#type::…` path must
/// all reduce to the same module identity; this is the single place that reduction
/// lives. A segment with no `r#` prefix is returned unchanged.
pub(super) fn canonical_segment(segment: &str) -> &str {
    segment.strip_prefix("r#").unwrap_or(segment)
}

/// Canonicalize a whole `::`-joined module path segment-by-segment (see
/// [`canonical_segment`]), in Unicode Normalization Form C, so a boundary's declared path and an observed path compare
/// in one vocabulary regardless of which uses the raw-identifier form or which composition of a character: the token
/// tree reads every identifier in NFC, as rustc compares them.
pub(crate) fn canonical_module_path(path: &str) -> String {
    let joined = path
        .split("::")
        .map(canonical_segment)
        .collect::<Vec<_>>()
        .join("::");
    if joined.is_ascii() {
        joined
    } else {
        joined.nfc().collect()
    }
}

/// Whether `segment` is exactly one identifier, written with nothing around it.
///
/// Read as the Rust Reference's identifier grammar reads it — a first character that is `_` or Unicode
/// `XID_Start`, then `XID_Continue` characters, behind at most one `r#` — with one exception: `_` alone, which the
/// Reference reads as no identifier, passes here and is refused by each caller. A module path `crate::_` names no
/// module, so its existence check refuses it, as 渾儀's does; a symbol prefix has no existence check past its head,
/// so [`is_symbol_path_spelling`] refuses it. Behind `r#` the five names a raw identifier cannot spell — `crate`,
/// `self`, `super`, `Self` and `_` — are refused, as rustc refuses them. Whitespace, a soft hyphen, a word joiner or a dash is no `XID_Continue` character, so `std::process` followed
/// by a left-to-right mark is refused rather than accepted as a prefix no path can spell. Keywords pass: a module path
/// names a module whatever its spelling, and the existence check that follows every crate-rooted path is what
/// refuses one that names nothing.
fn is_identifier(segment: &str) -> bool {
    let (raw, name) = match segment.strip_prefix("r#") {
        Some(name) => (true, name),
        None => (false, segment),
    };
    let mut characters = name.chars();
    let lexes = characters
        .next()
        .is_some_and(|first| first == '_' || unicode_ident::is_xid_start(first))
        && characters.all(unicode_ident::is_xid_continue);
    lexes && !(raw && matches!(name, "crate" | "self" | "super" | "Self" | "_"))
}

/// A crate the toolchain ships, which a path may name without the package declaring it.
///
/// Each variant is a distinct answer to *which scope holds it*: [`Sysroot::Prelude`] is in every crate's extern
/// prelude, [`Sysroot::ProcMacro`] is in a proc-macro crate's alone, and [`Sysroot::Test`] in none, so a crate names
/// it only through an `extern crate`, which the scope table reads as a binding. Every path each publishes is ASCII.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Sysroot {
    /// `std`, `core` or `alloc`.
    Prelude,
    /// `proc_macro`.
    ProcMacro,
    /// `test`.
    Test,
}

/// The sysroot crate `head` names, written bare: the one list every reader of a sysroot head matches on.
pub(crate) fn sysroot_crate(head: &str) -> Option<Sysroot> {
    match head {
        "std" | "core" | "alloc" => Some(Sysroot::Prelude),
        "proc_macro" => Some(Sysroot::ProcMacro),
        "test" => Some(Sysroot::Test),
        _ => None,
    }
}

/// `crate`, or `crate::` followed by `::`-separated identifiers.
fn is_canonical_spelling(written: &str) -> bool {
    let mut segments = written.split("::");
    segments.next() == Some("crate") && segments.all(is_identifier)
}

/// The canonical spelling of a written module path, or the spelling it most plausibly meant.
///
/// A module path has exactly one accepted spelling — `crate`, or `crate::` followed by
/// `::`-separated identifiers — so that one module is one identity: a rule's module paths enter its
/// [`RuleKey`](xuanji::RuleKey), and a baseline entry recorded under one spelling would not suppress
/// the same finding declared under another. The one equivalence folded is rustc's own, `r#x` and `x`
/// being the same identifier, so the accepted form carries no raw prefix. Every other spelling is
/// refused rather than rewritten, and the refusal carries the suggestion: the written path's
/// non-empty, trimmed segments rooted at `crate`, or `None` when that is no canonical spelling
/// either, or when the path starts at `self` or `super` — relative to a module a declaration does
/// not have.
pub(crate) fn canonical_module_spelling(written: &str) -> Result<String, Option<String>> {
    if is_canonical_spelling(written) {
        return Ok(canonical_module_path(written));
    }
    let segments: Vec<&str> = written
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    let candidate = match segments.first().map(|head| canonical_segment(head)) {
        None => return Err(Some("crate".to_string())),
        Some("self" | "super") => return Err(None),
        Some("crate") => std::iter::once("crate")
            .chain(segments[1..].iter().copied())
            .collect::<Vec<_>>()
            .join("::"),
        Some(_) => format!("crate::{}", segments.join("::")),
    };
    Err(is_canonical_spelling(&candidate).then_some(candidate))
}

/// Whether a symbol path's first segment, as written, can never name a crate or module.
///
/// The set is the Rust Reference's identifier grammar rather than a keyword list: `_` is not an
/// identifier, so neither `_` nor `r#_` names anything; `crate`, `self`, `super` and `Self` cannot be
/// written raw; bare `self`, `super` and `Self` are relative to a module or type a declaration does not
/// have; and `crate` stands only at the start of a path, so after a leading `::` it names nothing. Bare
/// `crate` is the crate-root form. Every other head — a keyword in some edition or not — names the crate
/// or module of that name, written bare or raw.
fn is_disallowed_symbol_head(head: &str, is_global: bool) -> bool {
    match head.strip_prefix("r#") {
        Some(name) => matches!(name, "_" | "crate" | "self" | "super" | "Self"),
        None => matches!(head, "_" | "self" | "super" | "Self") || (is_global && head == "crate"),
    }
}

/// Whether `written` is `::`-separated identifiers (optionally starting with `::` for external
/// crates) whose first segment can name a crate or module ([`is_disallowed_symbol_head`]).
///
/// Under a sysroot head ([`sysroot_crate`]) every segment is ASCII, since every path a sysroot crate publishes is,
/// and nothing after this holds a sysroot prefix to what it names: `std::pr` followed by a Cyrillic `о` and
/// `cess` is an identifier to [`is_identifier`] and names no path, so it is refused here. Under any other
/// head a segment is held to [`is_identifier`] alone, since a crate's own names may lie past ASCII.
fn is_symbol_path_spelling(written: &str) -> bool {
    let is_global = written.starts_with("::");
    let raw = if is_global {
        let after = &written[2..];
        if after.is_empty() || after.starts_with(':') {
            return false;
        }
        after
    } else {
        written
    };
    if raw.ends_with(':') {
        return false;
    }
    let mut segments = raw.split("::");
    let Some(head) = segments.next() else {
        return false;
    };
    let under_sysroot = sysroot_crate(canonical_segment(head)).is_some();
    is_identifier(head)
        && !is_disallowed_symbol_head(head, is_global)
        && segments.all(|segment| {
            is_identifier(segment) && segment != "_" && (!under_sysroot || segment.is_ascii())
        })
}

/// Whether an inline-call prefix was written from the extern-crate root (`::std::time`) or bare
/// (`std::time`). Both name one crate, so the root form is not part of a prefix's identity; it is kept
/// for the readers that judge what the written first segment may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum PrefixRoot {
    Bare,
    Global,
}

/// An inline-call prefix in its one canonical form: `::`-separated segments with no leading `::` and
/// no `r#`, and the root form it was written in. `std::time` and `::std::time` are one `path`, which is
/// what a prefix's rule key, its violations' target and the call matcher all read, so the three cannot
/// disagree about which prefix a finding belongs to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SymbolPrefix {
    pub path: String,
    pub root: PrefixRoot,
}

impl SymbolPrefix {
    /// The canonical form of any written prefix. Total, so the model can key a rule on it before the
    /// spelling is judged; [`canonical_symbol_path_spelling`] is what refuses a spelling.
    pub(crate) fn of(written: &str) -> Self {
        match written.strip_prefix("::") {
            Some(rest) => SymbolPrefix {
                path: canonical_module_path(rest),
                root: PrefixRoot::Global,
            },
            None => SymbolPrefix {
                path: canonical_module_path(written),
                root: PrefixRoot::Bare,
            },
        }
    }
}

/// The canonical spelling of a written symbol path, or the spelling it most plausibly meant.
///
/// A symbol path — an inline-call prefix — is compared with resolved paths segment by segment, so it
/// has one accepted spelling: `::`-separated identifiers, optionally starting with `::` to explicitly
/// name an external crate, each read by the [`is_identifier`] a module path is read by, with a first
/// segment outside the finite set that can never name a crate or module
/// ([`is_disallowed_symbol_head`]). Which of the remaining first segments name something is the
/// caller's question, since it needs what the crate declares. `r#x` and `x` are one identifier, so the
/// accepted form is a [`SymbolPrefix`], raw prefixes and the leading `::` removed. Every other spelling
/// is refused, and the refusal carries the written path's non-empty, trimmed segments, raw prefixes
/// removed, as the suggestion — or `None` when that is no accepted spelling either.
pub(crate) fn canonical_symbol_path_spelling(
    written: &str,
) -> Result<SymbolPrefix, Option<String>> {
    if is_symbol_path_spelling(written) {
        return Ok(SymbolPrefix::of(written));
    }
    let is_global = written.trim().starts_with("::");
    let segments: Vec<&str> = written
        .split("::")
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .collect();
    if segments.is_empty() {
        return Err(None);
    }
    let first = segments[0];
    let head = canonical_segment(first);
    if is_disallowed_symbol_head(head, is_global) {
        return Err(None);
    }
    let candidate_path = segments
        .iter()
        .map(|segment| canonical_segment(segment))
        .collect::<Vec<_>>()
        .join("::");
    let candidate = if is_global {
        format!("::{candidate_path}")
    } else {
        candidate_path
    };
    Err(is_symbol_path_spelling(&candidate).then_some(candidate))
}

/// Fold a Cargo package name to its Rust import identifier: `-` → `_` (`windows-sys` →
/// `windows_sys`). Cargo maps a hyphenated package name to an underscore identifier in source, and
/// a `use` path can never contain `-`, so every site matching a declared package name against an
/// observed import head needs this fold. The single home of it, so a dependency-name-matching site
/// and a confined-crate-name site cannot silently diverge on the rule.
pub(crate) fn package_name_to_import_ident(name: &str) -> String {
    name.replace('-', "_")
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

/// The path segment standing for block `id` of table `table` in the path of a module declared in that block. It
/// begins with `{`, which no identifier does, so no path written in source and no prefix names a module through
/// it, and it names the block by its file's table and its scope, so two blocks' modules of one name are two
/// modules — in one file, or in two cfg-exclusive files of one module, which each number their blocks from their
/// own start. This and [`is_block_segment`] are the one owner of the spelling.
pub(super) fn block_segment(table: usize, id: u32) -> String {
    format!("{{block#{table}.{id}}}")
}

/// The segment naming the `count`th module of one name the blocks of one module declare, in source order: `{block}`
/// for the first and `{block N}` for the Nth after it, so two are two modules. A block names nothing a path outside it
/// can write, so the segment is the block's readable form rather than a path.
pub(super) fn block_label(count: usize) -> String {
    if count == 1 {
        "{block}".to_string()
    } else {
        format!("{{block {count}}}")
    }
}

/// Whether `segment` is a [`block_segment`].
pub(super) fn is_block_segment(segment: &str) -> bool {
    segment.starts_with('{')
}

/// `path` as a refusal shows it: each [`block_segment`] written `{block}`, since its table and scope numbers name
/// nothing a reader can find in the source. A segment already readable — `{block}`, or the `{block 2}` a second
/// block-declared module of one name is governed as — is kept.
pub(super) fn readable_module(path: &str) -> String {
    path.split("::")
        .map(|segment| {
            if segment.starts_with("{block#") {
                "{block}"
            } else {
                segment
            }
        })
        .collect::<Vec<_>>()
        .join("::")
}

/// Each module path of one file as an identity, in the order the file declares them: [`readable_module`]'s form,
/// and where two modules read alike — two functions each declaring a `mod m` — each after the first has its last
/// block segment numbered, `{block 2}`, so two modules are two identities. The number counts only modules that
/// read alike, so a block holding no module, or one holding a module of another name, moves none of them.
/// [`readable_module`] is for a refusal a reader repairs from; this is for a value a baseline keys on.
pub(super) fn identity_modules<'a>(
    modules: impl IntoIterator<Item = &'a str>,
) -> std::collections::BTreeMap<String, String> {
    let mut seen: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    let mut identities = std::collections::BTreeMap::new();
    for module in modules {
        if identities.contains_key(module) {
            continue;
        }
        let readable = readable_module(module);
        let count = seen.entry(readable.clone()).or_default();
        *count += 1;
        let identity = match readable.rfind("{block}") {
            Some(at) if *count > 1 => {
                format!(
                    "{}{{block {count}}}{}",
                    &readable[..at],
                    &readable[at + "{block}".len()..]
                )
            }
            _ => readable,
        };
        identities.insert(module.to_string(), identity);
    }
    identities
}

/// Whether `path` runs through a [`block_segment`]: it names an item of a module declared in a block, which
/// no path outside that block names.
pub(super) fn names_a_block_item(path: &str) -> bool {
    path.split("::").any(is_block_segment)
}

/// `path` with the block segments it ends in removed: the module a block-declared module's parent is.
pub(super) fn strip_block_segments(path: &str) -> &str {
    let mut path = path;
    while let Some((parent, last)) = path.rsplit_once("::") {
        if !is_block_segment(last) {
            break;
        }
        path = parent;
    }
    path
}

/// Canonicalize and fold module path segments, resolving embedded `self` and `super`
/// segments anywhere in the path (e.g. `["crate", "a", "b", "super", "c"]` -> `["crate", "a", "c"]`).
/// Returns `None` if `super` over-pops past the `crate` root or if the path is not crate-rooted.
///
/// A `super` in a module declared in a block names the module the block stands in: the block is not a
/// module, so the block segments a `super` leaves last are popped with it. Measured on rustc 1.96.0,
/// edition 2021: `fn g() { struct Y; mod m { fn h() { super::Y; } } }` is refused with `E0433`, while
/// `super::Z` for a `Z` of the enclosing module compiles.
pub(super) fn fold_canonical_segments(segments: &[&str]) -> Option<String> {
    let mut stack: Vec<&str> = Vec::new();
    for &raw_seg in segments {
        let seg = canonical_segment(raw_seg.trim());
        if seg.is_empty() {
            continue;
        }
        match seg {
            "self" => continue,
            "super" => {
                if stack.last() == Some(&"crate") || stack.is_empty() {
                    return None;
                }
                stack.pop();
                while stack
                    .last()
                    .is_some_and(|segment| is_block_segment(segment))
                {
                    stack.pop();
                }
            }
            other => stack.push(other),
        }
    }
    if stack.first() == Some(&"crate") {
        Some(stack.join("::"))
    } else {
        None
    }
}

/// Resolve a `self::…` / `super::…` relative path against `current_module` into a crate-rooted
/// absolute path. `parts` is the already-canonicalized, `::`-split path whose first segment is
/// `self` or `super`. Returns `None` when a `super` chain **over-pops** past the crate root (more
/// `super`s than ancestors): the result would not be crate-rooted, names no internal module (and
/// the source does not compile), so it must never be mistaken for an outward edge. Any other head
/// (`parts[0]` not `self`/`super`, or empty) also returns `None` — the caller resolves those.
///
/// The single home of the `super`-pop loop and its over-pop guard, which the import scan ([`super::use_scan`]), the
/// resolver ([`super::resolve`]) and the visibility reading ([`super::item_head`]) share — so a fix to that subtle edge
/// cannot silently diverge across them. guibiao-internal; crosses no dimension boundary.
pub(super) fn resolve_self_super(current_module: &str, parts: &[&str]) -> Option<String> {
    let first = parts.first().copied()?;
    if first != "self" && first != "super" {
        return None;
    }
    let mut full: Vec<&str> = current_module
        .split("::")
        .filter(|s| !s.is_empty())
        .collect();
    full.extend(parts);
    fold_canonical_segments(&full)
}

/// Where a written path stands: a `use` path — a `use` leaf, a `pub use` or a glob — or any other path:
/// an expression, or a `type` alias's target.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub(super) enum PathSite {
    Use,
    Expr,
}

/// What a written path's first segment roots it at, before any scope is read: the one classification
/// every written path — a `use` leaf, a glob, a `type` alias's target, an occurrence — starts from.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum WrittenRoot {
    /// `crate::…`, or a `self`/`super` path folded against its module: crate-rooted.
    Crate(String),
    /// `::head::…` in edition 2018 and later: the crate named `head`, a sysroot crate or a dependency.
    Extern { head: String, rest: Vec<String> },
    /// In edition 2015, a `use` path or a path beginning with `::`: `head` is looked up in the crate
    /// root's scope, where an `extern crate` is itself an item, and names a crate where nothing there
    /// binds it.
    FromCrateRoot { head: String, rest: Vec<String> },
    /// A bare head, looked up in the scope the path is written in — in edition 2018 and later a `use`
    /// path's too, as a uniform path.
    Bare { head: String, rest: Vec<String> },
    /// Nothing a path can start from: no segment, a `super` past the crate root, or `::self`,
    /// `::super`, `::Self`.
    Invalid,
}

/// Classify the path `written` at `module`. Only the root is decided here; what a bare head names is
/// the resolver's lookup, never a guess from the head's spelling.
///
/// A bare `::`, the base of a `use ::*;` glob, names the crate root in edition 2015, where a `::`-rooted path starts
/// there, and nothing in later editions, where it starts at the extern prelude a glob cannot read.
pub(super) fn written_root(
    written: &str,
    module: &str,
    site: PathSite,
    edition: Edition,
) -> WrittenRoot {
    let raw = written.trim();
    let global = raw.starts_with("::");
    let parts: Vec<String> = raw
        .trim_start_matches("::")
        .split("::")
        .map(|s| canonical_module_path(s.trim()))
        .filter(|s| !s.is_empty())
        .collect();
    let Some((head, rest)) = parts.split_first() else {
        return if global && edition == Edition::Rust2015 {
            WrittenRoot::Crate("crate".to_string())
        } else {
            WrittenRoot::Invalid
        };
    };
    let parts_str: Vec<&str> = parts.iter().map(String::as_str).collect();
    let (head, rest) = (head.clone(), rest.to_vec());
    match head.as_str() {
        "crate" => {
            fold_canonical_segments(&parts_str).map_or(WrittenRoot::Invalid, WrittenRoot::Crate)
        }
        "self" | "super" | "Self" if global => WrittenRoot::Invalid,
        "self" | "super" => {
            resolve_self_super(module, &parts_str).map_or(WrittenRoot::Invalid, WrittenRoot::Crate)
        }
        _ if edition == Edition::Rust2015 && (global || site == PathSite::Use) => {
            WrittenRoot::FromCrateRoot { head, rest }
        }
        _ if global => WrittenRoot::Extern { head, rest },
        _ => WrittenRoot::Bare { head, rest },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fold_canonical_segments_resolves_embedded_super_and_self() {
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "b", "super", "c", "D"]),
            Some("crate::a::c::D".to_string())
        );
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "self", "b", "C"]),
            Some("crate::a::b::C".to_string())
        );
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "b", "super", "super", "secret"]),
            Some("crate::secret".to_string())
        );
    }

    #[test]
    fn fold_canonical_segments_over_pop_returns_none() {
        assert_eq!(
            fold_canonical_segments(&["crate", "a", "super", "super"]),
            None
        );
        assert_eq!(fold_canonical_segments(&["super", "a"]), None);
    }
}
