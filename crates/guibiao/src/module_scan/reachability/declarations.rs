//! Lexical extraction of top-level `mod` declarations and their cfg/path attributes.

#[cfg(test)]
use super::super::lexer::clean_with_positions;
use super::super::lexer::{balanced_group_end, is_ident_byte, transparent_macro_body_at};
use super::super::path_vocab::{canonical_segment, is_mod_declaration_keyword};

/// One `mod` declared at the top level of a byte range within already-cleaned (comment/string/
/// macro-body-stripped) text: its canonical name, whether it is inline (`{ … }`, `true`) or file
/// (`;`, `false`), and — for an inline declaration — the byte range of its body's *content*
/// (excluding the enclosing braces), so a caller can re-scan just that span to find further
/// declarations nested inside it. `direct_path_eq` is the cleaned-text position of the `=` in an
/// **unconditional** `#[path = "…"]` preceding a FILE declaration — cleaning has already dropped
/// the quoted value itself, so a caller resolves it by mapping this position back to the
/// original source (see [`super::super::lexer::clean_with_positions`]) and reading from there.
pub(super) struct DeclaredModule {
    pub(super) name: String,
    pub(super) is_inline: bool,
    pub(super) body: Option<(usize, usize)>,
    pub(super) direct_path_eq: Option<usize>,
    pub(super) conditional_path_eqs: Vec<usize>,
    /// Whether this declaration may legitimately have no source file in the current configuration —
    /// the "might legitimately be absent on this build" signal. Only meaningful for a non-inline
    /// (file-form) declaration with no resolvable file, where it decides between a tolerated skip and
    /// a constitution error, for a plain conventional file and for a `#[path]` remap target alike.
    ///
    /// Two sources, treated identically because they express one intent:
    /// - a BARE `#[cfg(...)]` attribute (never `cfg_attr`) precedes the item — see
    ///   [`has_bare_cfg_attr_before_item`];
    /// - the declaration sits directly inside a transparent control-flow macro arm (`cfg_if!`), whose
    ///   predicate lives in the macro's `if #[cfg(..)]` header rather than on the item. Every arm is
    ///   conditionally compiled by construction, the trailing `else` on its predicate's negation.
    ///
    /// Deliberately NOT the same signal as 渾儀's `has_cfg_attr`, which reads only the item's own
    /// attributes: the semantic dimension does not observe arm declarations at all yet, so it has
    /// nothing here to agree or disagree with until it does.
    pub(super) is_cfg_conditional: bool,
}

struct MacroScope {
    open_pos: usize,
    close_pos: usize,
    macro_depth: usize,
    inherited_top_level: bool,
}

enum ScanPosition {
    TransparentMacroStart,
    Source { is_top_level: bool },
}

#[derive(Default)]
struct TopLevelTracker {
    macro_scopes: Vec<MacroScope>,
    file_depth: usize,
}

impl TopLevelTracker {
    fn advance(&mut self, bytes: &[u8], index: usize) -> ScanPosition {
        while self
            .macro_scopes
            .last()
            .is_some_and(|active| index >= active.close_pos)
        {
            self.macro_scopes.pop();
        }

        if let Some((open_pos, close_pos)) = transparent_macro_body_at(bytes, index) {
            let inherited_top_level = self
                .macro_scopes
                .last()
                .map_or(self.file_depth == 0, |parent| {
                    parent.inherited_top_level && parent.macro_depth == 1
                });
            self.macro_scopes.push(MacroScope {
                open_pos,
                close_pos,
                macro_depth: 0,
                inherited_top_level,
            });
            return ScanPosition::TransparentMacroStart;
        }

        let is_top_level = self
            .macro_scopes
            .last()
            .map_or(self.file_depth == 0, |active| {
                active.inherited_top_level && active.macro_depth == 1
            });
        match bytes[index] {
            b'{' | b'(' | b'[' => {
                if let Some(active) = self.macro_scopes.last_mut() {
                    if index > active.open_pos {
                        active.macro_depth += 1;
                    }
                } else if bytes[index] == b'{' {
                    self.file_depth += 1;
                }
            }
            b'}' | b')' | b']' => {
                if let Some(active) = self.macro_scopes.last_mut() {
                    if index > active.open_pos {
                        active.macro_depth = active.macro_depth.saturating_sub(1);
                    }
                } else if bytes[index] == b'}' {
                    self.file_depth = self.file_depth.saturating_sub(1);
                }
            }
            _ => {}
        }
        ScanPosition::Source { is_top_level }
    }

    /// Whether a transparent control-flow macro (`cfg_if!`) body is open at the position last passed
    /// to [`Self::advance`] — which, combined with the `is_top_level` gate that already restricts
    /// `mod` observation to a declaration directly in an arm brace, is arm membership.
    ///
    /// Read rather than re-derived: scanning backward for a `cfg_if!` header would duplicate this
    /// scope model and have to re-solve arm-versus-item-body depth, the exact problem the model owns.
    fn in_transparent_macro(&self) -> bool {
        !self.macro_scopes.is_empty()
    }
}

/// The test-only `declared_modules_with_kind` generalized to scan `cleaned[range]` instead of a
/// whole file, so it can be re-applied to an inline module's own body — the byte span between its
/// braces — to find the `mod` declarations nested inside it. `path_attr_before_item` scans backward
/// from a candidate unbounded by `range.start`, which stays correct here: the nearest preceding
/// `;`/`{`/`}` it finds is either an earlier sibling's terminator within the range or the range's
/// own enclosing `{`, never a byte outside the declaration it is checking.
/// The direct/conditional `#[path]` remap pair for the `mod` declaration at `mod_index`, or
/// `(None, [])` when there is none — the shared shape both the inline (`{`) and file (`;`) forms
/// below extract identically from [`path_attr_before_item`], so a fix to one cannot silently
/// diverge from the other.
fn path_attr_pair(bytes: &[u8], mod_index: usize) -> (Option<usize>, Vec<usize>) {
    path_attr_before_item(bytes, mod_index).map_or_else(|| (None, Vec::new()), Remap::pair)
}

pub(super) fn declared_modules_in(
    cleaned: &str,
    range: std::ops::Range<usize>,
) -> Vec<DeclaredModule> {
    let bytes = cleaned.as_bytes();
    let end = range.end.min(bytes.len());
    let mut declared = Vec::new();
    let mut i = range.start.min(end);

    let mut top_level = TopLevelTracker::default();

    while i < end {
        let is_top_level = match top_level.advance(bytes, i) {
            ScanPosition::TransparentMacroStart => {
                i += 1;
                continue;
            }
            ScanPosition::Source { is_top_level } => is_top_level,
        };

        match bytes[i] {
            b'{' | b'(' | b'[' | b'}' | b')' | b']' => i += 1,
            b'm' if is_top_level && is_mod_declaration_keyword(bytes, i) => {
                let mut j = i + 3;
                while j < end && bytes[j].is_ascii_whitespace() {
                    j += 1;
                }
                let start = j;
                while j < end
                    && !bytes[j].is_ascii_whitespace()
                    && bytes[j] != b';'
                    && bytes[j] != b'{'
                {
                    j += 1;
                }
                let ident = cleaned[start..j].trim();
                let mut k = j;
                while k < end && bytes[k].is_ascii_whitespace() {
                    k += 1;
                }
                if !ident.is_empty() {
                    match bytes.get(k) {
                        Some(b'{') => {
                            // Skip the whole body in one jump — its content is re-scanned only if
                            // this module turns out to be inline-only, from `body` below. The
                            // module itself is always declared regardless of a preceding
                            // `#[path]` (rustc's `path` attribute never relocates an inline
                            // body's OWN content — the body already IS the module). It is NOT a
                            // no-op, though: it relocates the base directory THIS body's own
                            // file-form children resolve from (verified against a real rustc
                            // build: `#[path = "d"] mod x { mod y; }` compiles `y` at `d/y.rs`,
                            // never `<parent's child_base>/x/y.rs`) — an unconditional direct
                            // value is captured here (`direct_path_eq`) for exactly that reason;
                            // a `cfg_attr`-wrapped one stays the same stated, cfg-conditional skip
                            // bound as the file-form case (never followed cfg-blind).
                            let (direct_path_eq, conditional_path_eqs) = path_attr_pair(bytes, i);
                            let close = balanced_group_end(bytes, k).unwrap_or(bytes.len());
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: true,
                                body: Some((k + 1, close.saturating_sub(1))),
                                direct_path_eq,
                                conditional_path_eqs,
                                is_cfg_conditional: false,
                            });
                            i = close;
                            continue;
                        }
                        Some(b';') => {
                            // Either source makes an absent file legitimate: a bare `#[cfg]` on the
                            // item, or membership in a `cfg_if!` arm (whose predicate sits in the
                            // macro header, not on the item). Without the second, the two spellings
                            // of one per-platform shim get opposite verdicts on the same tree.
                            let is_cfg_conditional = has_bare_cfg_attr_before_item(bytes, i)
                                || top_level.in_transparent_macro();
                            let (direct_path_eq, conditional_path_eqs) = path_attr_pair(bytes, i);
                            declared.push(DeclaredModule {
                                name: canonical_segment(ident).to_string(),
                                is_inline: false,
                                body: None,
                                direct_path_eq,
                                conditional_path_eqs,
                                is_cfg_conditional,
                            });
                        }
                        _ => {}
                    }
                }
                i += 3;
            }
            _ => i += 1,
        }
    }
    declared
}

/// Names of modules declared at the top level (brace depth 0) of `source`, each paired with
/// whether it is an **inline** declaration (`mod name { … }`, `true`) or a **file** declaration
/// (`mod name;`, `false`) — the distinction [`reachable_modules`] needs to tell a real
/// file-backed module from an inline body whose same-named conventional file is an orphan.
/// Declared at any visibility (`pub mod`, `pub(crate) mod`, …). Comments, string/char literals,
/// and macro bodies are stripped first, so a commented-out, quoted, or macro-generated `mod` is
/// not counted; a `mod` nested inside another item (depth > 0) declares a child module, not a
/// crate-root one, and is skipped. Names are canonicalized (`r#name` -> `name`). Robust over
/// malformed input: it never panics (the same tolerance as the `use` scanner). Test-only: the
/// reachability walk itself calls [`declared_modules_in`] directly (over both whole files and
/// inline body spans), so production code no longer goes through this whole-file convenience.
#[cfg(test)]
fn declared_modules_with_kind(source: &str) -> Vec<(String, bool)> {
    // Strip macro bodies as well as comments/strings, the same hygiene the `use`
    // scanner applies: a `mod` written inside a macro body is macro-generated and out
    // of scope, so it must not be observed as a real declaration. (A `macro_rules!`
    // body is already excluded by brace depth; this also closes the `()`/`[]`-delimited
    // invocation gap, where `mod` would otherwise sit at brace depth 0.)
    let (cleaned, _positions) = clean_with_positions(source);
    let len = cleaned.len();
    declared_modules_in(&cleaned, 0..len)
        .into_iter()
        .map(|declared| (declared.name, declared.is_inline))
        .collect()
}

/// The declared module names only, discarding the inline/file kind — a test-only convenience
/// wrapping [`declared_modules_with_kind`] (itself test-only; see its doc).
#[cfg(test)]
pub(super) fn declared_modules(source: &str) -> Vec<String> {
    declared_modules_with_kind(source)
        .into_iter()
        .map(|(name, _)| name)
        .collect()
}

/// A `#[path]` remap the prefix before a `mod` keyword carries — which, being one, remaps something.
///
/// The static scanner intentionally does not read attributes in general, but `path` is a stated coverage
/// concern either way: an unconditional, direct `#[path = "…"]` is followed, carrying the cleaned-text
/// position of its `=` so the real value can be read from the untouched original source, and a
/// `cfg_attr`-wrapped one is a conditional candidate beside it. Every candidate physically written is
/// unioned, because this scanner is deliberately cfg-blind and neither attribute order nor the active
/// predicate may silently remove governed source.
///
/// **A remap that remaps nothing is unconstructible, and it used to be a runtime check.** The shape was a
/// three-variant enum whose `Remaps` held an `Option` and a `Vec` — so `direct: None` with an empty
/// `conditional` was a value the type admitted and only an `if` before the constructor kept out. Its
/// siblings were `None` and an `Excluded` that no consumer acted on: the sole match folded the two into one
/// arm, making the variant behaviourally identical to its neighbour while its doc claimed the opposite.
/// `first` keeps the conditional state non-empty in the type, the way `xuanji::bound::Defence::PinnedBy`
/// does for the same reason.
///
/// Measured by planting the construction the old shape admitted:
///
/// ```text
/// error[E0308]: mismatched types
///         let _ = Remap::Conditional { first: None, rest: Vec::new() };
///                                             ^^^^ expected `usize`, found `Option<_>`
/// ```
///
/// There is no negative *run* for this, and the reason is the property: a state the type cannot hold has no
/// value to assert about. The compiler's refusal is the evidence, and the two rows this change adds to
/// `both_readers_take_the_attribute_name_from_one_position` are what hold the behaviour that used to reach
/// the deleted variant.
///
/// **What `Excluded` was for is not legal Rust.** It stood for a `path`-named attribute with no followable
/// value, and its doc said such a module is excluded from conventional file backing. Measured against rustc
/// 1.96.0, edition 2021, `--crate-type lib`: `#[path] mod m;` and `#[path("m.rs")] mod m;` are both
/// `error: malformed 'path' attribute input`. A shape no configuration compiles is outside what a cfg-blind
/// union of compilable candidates governs, so the answer is the same as no remap at all — which is what the
/// fold already did, and is now what the type says.
enum Remap {
    /// An unconditional `#[path = "…"]`, with every cfg-conditional candidate written beside it.
    Direct { at: usize, conditional: Vec<usize> },
    /// Only cfg-conditional candidates. `first` is what keeps the state non-empty in the type.
    Conditional { first: usize, rest: Vec<usize> },
}

impl Remap {
    /// The direct/conditional pair a consumer reads, which is the shape both `mod` forms extract.
    fn pair(self) -> (Option<usize>, Vec<usize>) {
        match self {
            Remap::Direct { at, conditional } => (Some(at), conditional),
            Remap::Conditional { first, mut rest } => {
                rest.insert(0, first);
                (None, rest)
            }
        }
    }
}

fn path_attr_before_item(bytes: &[u8], mod_index: usize) -> Option<Remap> {
    let start = attribute_prefix_start(bytes, mod_index);
    let shift = |positions: Vec<usize>| positions.into_iter().map(|rel| start + rel).collect();
    match attr_prefix_remap(&bytes[start..mod_index])? {
        Remap::Direct { at, conditional } => Some(Remap::Direct {
            at: start + at,
            conditional: shift(conditional),
        }),
        Remap::Conditional { first, rest } => Some(Remap::Conditional {
            first: start + first,
            rest: shift(rest),
        }),
    }
}

/// Start of the top-level attribute prefix immediately before the item at `item_index`.
///
/// A preceding item or block delimiter ends the candidate prefix; punctuation inside literals and
/// comments has already been stripped by the reachability scan's cleaned source.
fn attribute_prefix_start(bytes: &[u8], item_index: usize) -> usize {
    (0..item_index)
        .rev()
        .find(|&i| matches!(bytes[i], b';' | b'{' | b'}'))
        .map_or(0, |i| i + 1)
}

/// Where the attribute's name begins, given the `#` at `hash` — or `None` where that `#` opens no attribute.
///
/// **Two readers ask this, and the position is one fact.** `attr_prefix_remap` looks for `path` here and
/// `attr_prefix_has_bare_cfg` looks for `cfg`, and what each looks *at* has to be the same byte or the two
/// disagree about the same source. Written out per site the preamble stood twice, byte-identical for
/// twenty-three lines and diverging only at the terminal word — which is the shape where one copy gets a
/// repair and the other keeps the defect. It got one: the raw-identifier skip below was added to both by
/// hand, and a hand is what would have to add the next one.
///
/// **A raw identifier is ONE segment**, at the attribute's own name position as much as inside a
/// `cfg_attr`'s argument list, where `cfg_attr_group_path_eqs` already consumes the prefix with the segment
/// it belongs to. `r#` changes a lexical spelling and not the name it spells, so `#[r#path = "…"]` IS the
/// built-in remap — measured against rustc 1.96.0, edition 2021, `--crate-type lib`, which compiles the
/// remapped file for it even with the conventional file present. Reading the name as written left this
/// scanner governing a file the build does not contain.
///
/// The `#` is not consumed on a miss: the caller advances by one and reads the next byte itself, so
/// `##[path = "…"]` reaches the attribute its second `#` opens.
fn attr_name_start(bytes: &[u8], hash: usize) -> Option<usize> {
    let mut i = hash + 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'[') {
        return None;
    }
    i += 1;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes[i..].starts_with(b"r#") && bytes.get(i + 2).is_some_and(|byte| is_ident_byte(*byte)) {
        i += 2;
    }
    Some(i)
}

fn attr_prefix_remap(bytes: &[u8]) -> Option<Remap> {
    let mut i = 0;
    let mut direct = None;
    let mut conditional_eqs = Vec::new();
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        let Some(name) = attr_name_start(bytes, i) else {
            i += 1;
            continue;
        };
        i = name;
        // **The two matching branches advance the cursor themselves, and the miss deliberately does not.**
        // All three terminated before this, but by the loop head rather than locally: the cursor sits on
        // the attribute's *name* from here rather than on the `#` that opened it, so `bytes[i] != b'#'`
        // stepped it forward on the next iteration and no branch could re-enter for the same attribute.
        // That invariant is true and it is not visible at the `continue` — four independent readings of
        // these lines called this scanner non-terminating, each reading the `continue` and not the loop
        // head. So where an advance is provable at the branch it is written there: `j` starts at `i + 4`
        // and a matched `cfg_attr` is eight bytes, and `starts_with` guarantees no `#` among the bytes
        // either one steps over, which is what makes the two identical to the walk they replace. The miss
        // is the one case where that does not hold, and the comment below it says why.
        if bytes[i..].starts_with(b"path")
            && bytes.get(i + 4).is_none_or(|byte| !is_ident_byte(*byte))
        {
            // Retain an unconditional direct path alongside every cfg-conditional candidate.
            // rustc currently gives multiple path-bearing attributes textual precedence (and
            // warns that accepting the shape will become an error), while this scanner is
            // deliberately cfg-blind. Unioning every physically existing written candidate is
            // therefore the only false-negative-safe observation: neither attribute order nor
            // the active predicate may silently remove governed source.
            let mut j = i + 4;
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if bytes.get(j) == Some(&b'=') {
                direct = Some(j);
                i = j + 1;
                continue;
            }
            // A bare `#[path]`/`#[path(...)]` is not valid remap syntax, and measured against rustc it is
            // not valid Rust either — `error: malformed 'path' attribute input` for both spellings. So it
            // contributes no candidate, and the scan continues rather than returning, because a later
            // unconditional `#[path = "…"]` on the same item still wins. `j` is the first byte that is
            // not whitespace after the name, so resuming there both advances and re-reads nothing.
            i = j;
            continue;
        }
        // The combined `#[cfg_attr(<pred>, …, path = "…")]` spelling (equivalent to
        // `#[cfg(<pred>)] #[path = "…"]`) is a conditional remap. Collect candidate path = "..."
        // positions across all cfg_attr occurrences. An unconditional `#[path = "…"]` elsewhere
        // on the same item still wins (above), so this keeps scanning instead of returning immediately.
        if bytes[i..].starts_with(b"cfg_attr")
            && bytes.get(i + 8).is_none_or(|byte| !is_ident_byte(*byte))
        {
            cfg_attr_prefix_collect_path_eqs(&bytes[i + 8..], i + 8, &mut conditional_eqs);
            i += 8;
            continue;
        }
        // No advance here, and it is not an omission. The name position can itself be a `#` —
        // `attr_name_start` skips `[` and whitespace and stops, so `#[#[path = "x.rs"]` answers the inner
        // `#` — and the loop head reads it as an attribute opener and reaches the remap inside. Stepping
        // over it would skip that attribute, which is a false negative in a scanner whose whole
        // construction is the false-negative-safe union. A direction holds this shape.
    }
    match (direct, conditional_eqs) {
        (Some(at), conditional) => Some(Remap::Direct { at, conditional }),
        (None, mut conditional) if !conditional.is_empty() => Some(Remap::Conditional {
            first: conditional.remove(0),
            rest: conditional,
        }),
        (None, _) => None,
    }
}

fn cfg_attr_prefix_collect_path_eqs(bytes: &[u8], base_offset: usize, eqs: &mut Vec<usize>) {
    let mut pending = vec![(bytes, base_offset)];
    while let Some((bytes, base_offset)) = pending.pop() {
        cfg_attr_group_path_eqs(bytes, base_offset, eqs, &mut pending);
    }
    // The explicit work stack may visit a nested group after a later sibling in its parent.
    // Source offsets restore rustc-facing textual order without returning to native recursion.
    eqs.sort_unstable();
}

fn cfg_attr_group_path_eqs<'a>(
    bytes: &'a [u8],
    base_offset: usize,
    eqs: &mut Vec<usize>,
    pending: &mut Vec<(&'a [u8], usize)>,
) {
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    if bytes.get(i) != Some(&b'(') {
        return;
    }
    i += 1;
    let mut depth = 1usize;
    let mut past_predicate = false;
    // Whether the previous significant token was a path separator. The attribute admitting applied metas
    // is the BUILT-IN `cfg_attr`, whose path is exactly that one segment: `foo::cfg_attr(a, path = "…")`
    // ends in the same word while being somebody else's attribute, and descending into it reads a target
    // no build compiles. Measured against this reader before the test existed: the span
    // `(any(), foo::cfg_attr(a, path = "bogus"), path = "real.rs")` yielded two positions where one is
    // right, and the raw spelling `foo::r#cfg_attr` yielded two as well.
    let mut after_path_sep = false;
    while i < bytes.len() && depth > 0 {
        match bytes[i] {
            b':' if bytes.get(i + 1) == Some(&b':') => {
                after_path_sep = true;
                i += 2;
            }
            b'"' => {
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
                after_path_sep = false;
                i += 1;
            }
            b')' => {
                depth -= 1;
                after_path_sep = false;
                i += 1;
            }
            b',' if depth == 1 => {
                past_predicate = true;
                after_path_sep = false;
                i += 1;
            }
            byte if depth == 1 && past_predicate && is_ident_byte(byte) => {
                // A raw identifier is ONE segment: `r#` changes a lexical spelling and not a name, so
                // `r#cfg_attr` names `cfg_attr`. Reading `r`, `#` and `cfg_attr` as separate events would
                // clear a qualification the separator had set.
                let mut start = i;
                if bytes[i] == b'r' && bytes.get(i + 1) == Some(&b'#') {
                    let after = i + 2;
                    if after < bytes.len() && is_ident_byte(bytes[after]) {
                        start = after;
                    }
                }
                i = start;
                while i < bytes.len() && is_ident_byte(bytes[i]) {
                    i += 1;
                }
                let ident = &bytes[start..i];
                let qualified = after_path_sep;
                after_path_sep = false;
                // **The narrowing belongs to the target as much as to the wrapper.** `qualified` was
                // computed here and spent on the `cfg_attr` arm alone, so `foo::path = "bogus.rs"` — an
                // attribute belonging to somebody else — was collected as a module remap. Measured
                // against rustc 1.96.0, edition 2021, `--crate-type lib`:
                // `#[cfg_attr(any(), foo::path = "bogus.rs", path = "real.rs")] mod plat;` compiles,
                // because a false predicate means no applied attribute is expanded and `foo::path` is
                // never resolved. This scanner is cfg-blind, so it unions every candidate on disk — and a
                // file named by nobody's `path` is not one. Reading it reports a violation against source
                // the governed tree does not compile.
                if ident == b"path" && !qualified {
                    let mut j = i;
                    while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                        j += 1;
                    }
                    if bytes.get(j) == Some(&b'=') {
                        eqs.push(base_offset + j);
                    }
                } else if ident == b"cfg_attr" && !qualified {
                    pending.push((&bytes[i..], base_offset + i));
                }
            }
            _ => {
                if !bytes[i].is_ascii_whitespace() {
                    after_path_sep = false;
                }
                i += 1;
            }
        }
    }
}

/// Whether a BARE `#[cfg(...)]` attribute (never `cfg_attr`) is among the attribute prefix
/// immediately preceding an item — the same "might legitimately be absent on this build" signal
/// hunyi's `has_cfg_attr` checks via `syn` (`crate::syn_util::has_cfg_attr`), hand-rolled here for
/// this crate's syn-free scanner. Deliberately narrow: this detects mere PRESENCE of the `cfg`
/// identifier, never evaluates a predicate — the same syntactic-identifier-only shape already used
/// above to detect `path`/`cfg_attr`, not a new capability tier or a step toward general attribute
/// evaluation. `cfg_attr` is deliberately excluded (verified against a real `rustc` build): unlike
/// a bare `#[cfg(pred)]`, which removes the whole item when `pred` is false, `#[cfg_attr(pred, …)]`
/// never removes the item — it only conditionally applies its wrapped attribute — so it must never
/// grant this tolerance (`#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a
/// genuine compile error, E0583, on every platform).
fn has_bare_cfg_attr_before_item(bytes: &[u8], mod_index: usize) -> bool {
    let start = attribute_prefix_start(bytes, mod_index);
    attr_prefix_has_bare_cfg(&bytes[start..mod_index])
}

fn attr_prefix_has_bare_cfg(bytes: &[u8]) -> bool {
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'#' {
            i += 1;
            continue;
        }
        let Some(name) = attr_name_start(bytes, i) else {
            i += 1;
            continue;
        };
        i = name;
        // The byte immediately after `cfg` must not continue the identifier (excludes `cfg_attr`,
        // whose next byte is `_`).
        if bytes[i..].starts_with(b"cfg")
            && bytes.get(i + 3).is_none_or(|byte| !is_ident_byte(*byte))
        {
            return true;
        }
        i += 1;
    }
    false
}

#[cfg(test)]
mod tests {
    use super::{
        attr_name_start, attr_prefix_has_bare_cfg, attr_prefix_remap,
        cfg_attr_prefix_collect_path_eqs,
    };

    /// The two readers of an attribute prefix agree about where the name is, because they ask once.
    ///
    /// The position is what the two shared before it was extracted: twenty-three byte-identical lines in
    /// each, diverging only at `path` versus `cfg`. This direction is over the position itself and over both
    /// verdicts that stand on it, so a change to one reader's approach shows up as a disagreement here
    /// rather than as a `cfg` scanner that learned a spelling the `path` scanner did not.
    #[test]
    fn both_readers_take_the_attribute_name_from_one_position() {
        // (prefix, where the name starts, does it remap, is it a bare cfg)
        for (prefix, name_at, remaps, bare_cfg) in [
            (&b"#[path = \"x.rs\"]"[..], Some(2), true, false),
            (&b"#[cfg(unix)]"[..], Some(2), false, true),
            // Whitespace on either side of the bracket, which the preamble skips in two separate loops.
            (&b"# [ path = \"x.rs\"]"[..], Some(4), true, false),
            (&b"# [ cfg(unix)]"[..], Some(4), false, true),
            // The raw spelling names the built-in, at the attribute's own name position.
            (&b"#[r#path = \"x.rs\"]"[..], Some(4), true, false),
            (&b"#[r#cfg(unix)]"[..], Some(4), false, true),
            // `cfg_attr` is not a bare `cfg`, raw-spelled or not — the byte after `cfg` continues the
            // identifier.
            (
                &b"#[r#cfg_attr(unix, path = \"x.rs\")]"[..],
                Some(4),
                true,
                false,
            ),
            // A `path`-named attribute with no followable value contributes no candidate. Measured
            // against rustc 1.96.0, edition 2021, `--crate-type lib`: both spellings are
            // `error: malformed 'path' attribute input`, so no configuration compiles either, and a
            // cfg-blind union of compilable candidates has nothing to union. The name position is still
            // read, which is what keeps this row about the remap verdict rather than about the reader
            // failing to find a name.
            (&b"#[path]"[..], Some(2), false, false),
            (&b"#[path(\"x.rs\")]"[..], Some(2), false, false),
            // A lone `r#` is not a raw identifier, so the name starts at the `r`.
            (&b"#[r#]"[..], Some(2), false, false),
            // The `#` that opens nothing is stepped over, and the one after it is read.
            (&b"##[path = \"x.rs\"]"[..], Some(3), true, false),
            // A `#` at the NAME position, which is the one case where the scan must not step off a name
            // it did not match. `attr_name_start` skips `[` and whitespace and stops, so it answers the
            // inner `#` here — and the remap is inside the attribute that `#` opens. The union is
            // false-negative-safe by construction, so a candidate physically written in the source counts
            // whether or not any configuration compiles this spelling.
            (&b"#[#[path = \"x.rs\"]"[..], Some(2), true, false),
        ] {
            let spelling = String::from_utf8_lossy(prefix).into_owned();
            let hash = prefix
                .iter()
                .position(|byte| *byte == b'#')
                .expect("every row opens with a hash");
            let start = match attr_name_start(prefix, hash) {
                Some(start) => Some(start),
                None => attr_name_start(prefix, hash + 1),
            };
            assert_eq!(start, name_at, "the name position in {spelling}");

            assert_eq!(
                attr_prefix_remap(prefix).is_some(),
                remaps,
                "the path reader's verdict on {spelling}"
            );
            assert_eq!(
                attr_prefix_has_bare_cfg(prefix),
                bare_cfg,
                "the cfg reader's verdict on {spelling}"
            );
        }
    }

    #[test]
    fn deeply_nested_cfg_attr_paths_use_a_bounded_native_stack() {
        const DEPTH: usize = 4096;
        let mut nested = String::new();
        for _ in 0..DEPTH {
            nested.push_str("(predicate, cfg_attr");
        }
        nested.push_str("(predicate, path = \"target.rs\")");
        for _ in 0..=DEPTH {
            nested.push(')');
        }

        let mut eqs = Vec::new();
        cfg_attr_prefix_collect_path_eqs(nested.as_bytes(), 0, &mut eqs);

        assert_eq!(
            eqs.len(),
            1,
            "the deepest path candidate remains observable"
        );
        assert_eq!(nested.as_bytes()[eqs[0]], b'=');
    }

    #[test]
    fn iterative_cfg_attr_collection_preserves_textual_candidate_order() {
        let nested =
            b"(a, cfg_attr(b, path = \"first.rs\"), path = \"second.rs\", cfg_attr(c, path = \"third.rs\"))";
        let mut eqs = Vec::new();

        cfg_attr_prefix_collect_path_eqs(nested, 0, &mut eqs);

        assert_eq!(eqs.len(), 3);
        assert!(eqs.windows(2).all(|pair| pair[0] < pair[1]), "{eqs:?}");
    }

    /// The attribute admitting applied metas is the BUILT-IN `cfg_attr`, whose path is exactly one
    /// segment.
    ///
    /// **A qualified look-alike ends in the same word while being somebody else's attribute**, so
    /// descending into it collects a target no build compiles — a module read that rustc never reads,
    /// and any violation found there is reported over source the governed tree does not have. `r#`
    /// changes an identifier's lexical spelling and not its name, so it must not split one segment into
    /// separate events either.
    ///
    /// Negative run, against the reader that matched the bare identifier: both spellings yielded **two**
    /// positions where one is right.
    ///
    /// The control is the other half: an unqualified `cfg_attr`, raw-spelled or not, IS the built-in, so
    /// this must not cost a genuine nested group its applied metas.
    #[test]
    fn a_qualified_look_alike_is_not_the_built_in_cfg_attr() {
        for (label, span) in [
            (
                "plain",
                &b"(any(), foo::cfg_attr(a, path = \"bogus\"), path = \"real.rs\")"[..],
            ),
            (
                "raw identifier",
                &b"(any(), foo::r#cfg_attr(a, path = \"bogus\"), path = \"real.rs\")"[..],
            ),
        ] {
            let mut eqs = Vec::new();
            super::cfg_attr_prefix_collect_path_eqs(span, 0, &mut eqs);
            assert_eq!(
                eqs.len(),
                1,
                "{label}: only the applied target is a module path, got {eqs:?}"
            );
        }

        // Control: an unqualified nested `cfg_attr` still contributes, in either spelling.
        for (label, span) in [
            ("plain", &b"(any(), cfg_attr(a, path = \"nested.rs\"))"[..]),
            (
                "raw identifier",
                &b"(any(), r#cfg_attr(a, path = \"nested.rs\"))"[..],
            ),
        ] {
            let mut eqs = Vec::new();
            super::cfg_attr_prefix_collect_path_eqs(span, 0, &mut eqs);
            assert_eq!(
                eqs.len(),
                1,
                "{label}: an unqualified nested group keeps its applied metas, got {eqs:?}"
            );
        }
    }
}
