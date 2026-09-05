use super::probes::*;
use std::collections::HashMap;

/// Read the string-literal value of a `#[path = "…"]` starting just past the `=` (`start`), bounded
/// by `end`. Handles a normal `"…"` (with the standard escapes) and a raw `r"…"` / `r#…"…"#` string
/// (content verbatim). Returns `None` when no string literal follows (a non-literal `path` argument
/// is not a valid remap) — the caller then treats the module as non-relocated (conventional
/// resolution or a loud missing-file error, never a silent skip). Bytes accumulate so a UTF-8
/// filename round-trips.
pub(crate) fn read_path_string(bytes: &[u8], start: usize, end: usize) -> Option<String> {
    // Advance past whitespace and comments to the value — but NOT over a string literal, which is
    // exactly what we are here to read (`skip_preamble_trivia` would skip the literal as trivia).
    let mut i = start;
    while i < end {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b'/' && matches!(bytes.get(i + 1), Some(&b'/') | Some(&b'*')) {
            if let Some(next) = skip_literal_or_comment(bytes, i) {
                i = next.min(end);
                continue;
            }
        }
        break;
    }
    if bytes.get(i) == Some(&b'r') {
        // Raw string `r#…"content"#…`: no escapes; the closing is `"` then the same `#` count.
        let mut hashes = 0usize;
        let mut j = i + 1;
        while bytes.get(j) == Some(&b'#') {
            hashes += 1;
            j += 1;
        }
        if bytes.get(j) != Some(&b'"') {
            return None;
        }
        j += 1;
        let content_start = j;
        while j < end {
            if bytes[j] == b'"' {
                let mut k = j + 1;
                let mut seen = 0usize;
                while seen < hashes && bytes.get(k) == Some(&b'#') {
                    k += 1;
                    seen += 1;
                }
                if seen == hashes {
                    return String::from_utf8(bytes[content_start..j].to_vec()).ok();
                }
            }
            j += 1;
        }
        return None;
    }
    if bytes.get(i) != Some(&b'"') {
        return None;
    }
    i += 1;
    let content_start = i;
    while i < end {
        match bytes[i] {
            // Decode the literal's escapes through the crate's full decoder — the same set rustc
            // and syn accept (incl. `\x` / `\u{}` / `\'`) — so 漏刻's `#[path]` value matches 渾儀's
            // syn-derived `s.value()` on the same input (twin-drift parity). A residually
            // undecodable form (e.g. a backslash-newline line continuation) yields `None` and the
            // module falls back to non-relocated handling — fail-safe, never a mis-decoded path.
            b'"' => return decode_str_escapes(&bytes[content_start..i]),
            // Skip the escaped byte so an escaped quote `\"` (or `\\`) does not end the literal early.
            b'\\' => i += 2,
            _ => i += 1,
        }
    }
    None
}

pub(crate) fn is_mod_keyword(bytes: &[u8], i: usize) -> bool {
    bytes.get(i..i + 3) == Some(b"mod")
        && (i == 0 || !is_ident_byte(bytes[i - 1]))
        && bytes.get(i + 3).is_none_or(|b| !is_ident_byte(*b))
}

pub(crate) fn preceding_token_is_ident(bytes: &[u8], bang: usize) -> bool {
    let mut end = bang;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    end > 0 && is_ident_byte(bytes[end - 1])
}

pub(crate) fn skip_ascii_space(bytes: &[u8], mut i: usize) -> usize {
    while i < bytes.len() && bytes[i].is_ascii_whitespace() {
        i += 1;
    }
    i
}

/// Advance past whitespace AND any interleaved comments — used between the `mod` keyword and its
/// name, and between the name and its terminator (`;`/`{`), where a comment is trivia to rustc
/// (`pub mod /* relocated */ child;` compiles identically to `pub mod child;`) but a bare
/// whitespace-only skip stops at the comment's leading `/`. There, the following identifier-run
/// scan finds no valid identifier at that position, so the whole declaration was never recognized
/// as a `mod` at all — not a graceful skip, a silent corpus drop: the module and its entire
/// subtree, and every probe beneath it, vanished from the scan (found on adversarial review; a
/// comment in this position is legal and unremarkable Rust, not a stated bound).
pub(crate) fn skip_space_and_comments(bytes: &[u8], mut i: usize) -> usize {
    loop {
        i = skip_ascii_space(bytes, i);
        if i >= bytes.len() {
            return i;
        }
        match skip_literal_or_comment(bytes, i) {
            Some(next) => i = next,
            None => return i,
        }
    }
}

/// The **interior** byte ranges of each arm of a transparent macro invocation whose `!` is at `bang`
/// and whose balanced body ends at `body_end` (as [`foreign_macro_body_end`] reports it, one past the
/// closing delimiter).
///
/// `cfg_if!`'s grammar is `if #[cfg(a)] { items } else if #[cfg(b)] { items } else { items }`, so the
/// body's top-level **brace** groups are exactly the arms. A `#[cfg(…)]` predicate is a `#` plus a
/// *bracket* group and `if` / `else` are bare identifiers, so those bytes are simply walked over: the
/// only way a `{` could hide inside a predicate is within a string literal, which
/// `skip_literal_or_comment` consumes first. The invocation's own outer delimiter is irrelevant —
/// `cfg_if!( … )` works the same as `cfg_if! { … }`.
pub(crate) fn transparent_arm_ranges(
    b: &[u8],
    bang: usize,
    body_end: usize,
) -> Vec<(usize, usize)> {
    let mut arms = Vec::new();
    let open = skip_trivia(b, bang + 1);
    if !matches!(b.get(open), Some(b'{') | Some(b'(') | Some(b'[')) {
        return arms;
    }
    // Just inside the invocation's own delimiter, up to (not including) its closer.
    let limit = body_end.saturating_sub(1);
    let mut i = open + 1;
    while i < limit {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next.min(limit);
            continue;
        }
        if b[i] == b'{' {
            let close = balanced_brace_end(b, i, limit);
            arms.push((i + 1, close.saturating_sub(1)));
            i = close;
            continue;
        }
        i += 1;
    }
    arms
}

/// Find the index just past the matching closer for a delimiter group opened at `open`
/// (`(`/`)`, `[`/`]`, `{`/`}`), scanning `[open, limit)` with nesting depth tracked so an inner
/// delimiter of the SAME kind does not prematurely close the group; literals/comments are skipped
/// so a delimiter-like byte inside one never miscounts. Returns `limit` if the group never closes
/// within it — a caller-bound scan limit, never a hang. Shared by [`balanced_brace_end`],
/// [`paren_group_end`], and [`attr_group_end`], which each pick one delimiter pair.
pub(crate) fn delimiter_group_end(
    bytes: &[u8],
    open: usize,
    limit: usize,
    open_b: u8,
    close_b: u8,
) -> usize {
    let mut depth = 0usize;
    let mut i = open;
    while i < limit {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(limit);
            continue;
        }
        if bytes[i] == open_b {
            depth += 1;
        } else if bytes[i] == close_b {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return i + 1;
            }
        }
        i += 1;
    }
    limit
}

pub(crate) fn balanced_brace_end(bytes: &[u8], open: usize, limit: usize) -> usize {
    delimiter_group_end(bytes, open, limit, b'{', b'}')
}

/// Outer attributes on the `mod name;` at `mod_index` that steer the walker.
pub(crate) struct ModPreambleAttrs {
    /// The target of an **unconditional** `#[path = "..."]` relocation (the direct string form): the
    /// module lives at this author-chosen file, which the walker now *follows* to count its probes
    /// (closing the relocated-module coverage gap). `None` when there is no such attribute.
    pub(crate) path: Option<String>,
    /// Every `path = "…"` target found inside a `#[cfg_attr(<pred>, …, path = "…")]` wrapper — a
    /// module may carry more than one, one per platform predicate. `cfg_attr` never removes the
    /// `mod` item the way a bare `#[cfg]` does, so cfg-blind observation must union EVERY candidate
    /// (found on adversarial review: earlier code matched only the exact identifier `cfg`, so
    /// `cfg_attr` — a different identifier — matched neither the `path` arm above nor the bare-`cfg`
    /// arm below, and this field did not exist at all; a `cfg_attr`-wrapped `#[path]` target was
    /// therefore never followed, contradicting this very doc's own prior claim that it "reads as
    /// cfg"). Each candidate is resolved the identical way the unconditional `path` above is
    /// (relative to the containing file's own directory), not the conventional-child base.
    pub(crate) cfg_attr_paths: Vec<String>,
    /// A **bare** `#[cfg(...)]` gate: the module may legitimately have no file in the current
    /// configuration (an off feature / another platform), so an absent file is tolerated rather than
    /// a scan error — the same cfg-tolerance 渾儀 applies, reimplemented louke-locally (三儀 ⊥ 三儀).
    /// This is not `cfg` evaluation: a resolvable cfg-gated module is still scanned and its probes
    /// still counted; only an *absent* file for a cfg-gated declaration is tolerated. `cfg_attr` does
    /// NOT set this — see `cfg_attr_paths` above for its own, additive absence tolerance.
    pub(super) cfg: bool,
}

/// Scan a `mod name;`'s preamble (the bytes since the previous item boundary) for the outer
/// attributes that steer the walker. Detection is **structural, not a raw substring**: comments and
/// string literals are skipped, and only an *outer attribute whose meta name is exactly* `path`
/// (followed by `=`), `cfg`, or `cfg_attr` matches. A comment or unrelated attribute that merely
/// contains the text (`// fast path`, `#[cfg(feature = "fastpath")]`) MUST NOT be read as a `path`
/// relocation — a false match would drop a reachable module and every probe under it (a silent
/// coverage false negative, the worst outcome under FN-first). A `#[cfg_attr(.., path = ..)]`
/// conditional relocation's own `path = "…"` target is extracted separately (`cfg_attr_paths`, every
/// candidate the item carries) and unioned with the conventional file by the caller — never treated
/// as equivalent to a bare `#[cfg]`'s absence tolerance, since `cfg_attr` never removes the item.
///
/// `scope_start` bounds the search for the preamble's own start: it is the enclosing scope's own
/// start (a real item/scope boundary, never inside a literal or comment), so scanning **forward**
/// from it — skipping literals/comments exactly like the rest of this file's walkers — to find the
/// last `;`/`}` outside of any literal/comment/attribute-group is well-defined. A backward raw-byte
/// scan (the original implementation) is NOT well-defined this way: it cannot tell whether a
/// `;`/`{`/`}` byte it meets while walking backward sits inside a string/char literal or comment
/// without first knowing where that literal started — so an EARLIER attribute's own string value
/// containing one of those bytes (e.g. `#[doc = "Handles A; falls back to B."]`) stopped the old
/// backward scan mid-literal, desyncing the subsequent forward attribute walk and silently losing a
/// later `#[path = "…"]` on the same preamble.
///
/// The forward scan is not merely literal-aware but **attribute-group-aware**: an entire `#[…]` /
/// `#![…]` is skipped as one atomic unit via [`attr_group_end`], the identical primitive the
/// second (attribute-matching) pass below already uses. Attribute syntax permits an arbitrary
/// token-tree argument, including a brace-delimited one (`#[foo({ 1 })]`) that is not a string
/// literal — literal-awareness in the first pass alone still let such a brace be mistaken for a
/// top-level item terminator, resetting `start` to a point AFTER an earlier, real
/// `#[path = "…"]` attribute and silently losing it: the identical failure mode above, reached
/// through a different vector. A non-attribute `{…}`
/// (a preceding sibling item's own block body, or a macro invocation's body) is likewise skipped
/// as one atomic unit via [`balanced_brace_end`], landing on its own matching `}` — the real
/// boundary — rather than treating the interior's own bytes as candidates.
pub(crate) fn mod_preamble_attrs(
    bytes: &[u8],
    scope_start: usize,
    mod_index: usize,
) -> ModPreambleAttrs {
    let mut start = scope_start;
    let mut i = scope_start;
    while i < mod_index {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(mod_index);
            continue;
        }
        if bytes[i] == b'#' {
            let mut open = i + 1;
            if bytes.get(open) == Some(&b'!') {
                open += 1;
            }
            if bytes.get(open) == Some(&b'[') {
                // The whole attribute group is opaque here — its own `;`/`{`/`}` bytes (inside a
                // token-tree argument) are content, never a boundary. Left in the scanned range
                // for the second pass below, which is what actually matches it.
                i = attr_group_end(bytes, open, mod_index);
                continue;
            }
        }
        if bytes[i] == b'{' {
            i = balanced_brace_end(bytes, i, mod_index);
            start = i;
            continue;
        }
        if bytes[i] == b';' {
            start = i + 1;
        }
        i += 1;
    }
    let mut attrs = ModPreambleAttrs {
        path: None,
        cfg_attr_paths: Vec::new(),
        cfg: false,
    };
    let mut i = start;
    while i < mod_index {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(mod_index);
            continue;
        }
        if bytes[i] == b'#' {
            let mut open = i + 1;
            if bytes.get(open) == Some(&b'!') {
                open += 1;
            }
            if bytes.get(open) == Some(&b'[') {
                // The attribute's meta name is the first identifier inside the brackets.
                let name_start = skip_preamble_trivia(bytes, open + 1, mod_index);
                let mut name_end = name_start;
                while name_end < mod_index && is_ident_byte(bytes[name_end]) {
                    name_end += 1;
                }
                match &bytes[name_start..name_end] {
                    b"path" => {
                        let eq = skip_preamble_trivia(bytes, name_end, mod_index);
                        if bytes.get(eq) == Some(&b'=') {
                            attrs.path = read_path_string(bytes, eq + 1, mod_index);
                        }
                    }
                    // A BARE `#[cfg(pred)]` genuinely removes the whole item when `pred` is false
                    // — the file may legitimately be absent. `cfg_attr` does NOT: it only
                    // conditionally applies its wrapped attribute(s); the `mod` item itself always
                    // exists regardless of the predicate (verified against a real `rustc` build:
                    // `#[cfg_attr(unix, allow(dead_code))] mod x;` with no `x.rs` is E0583 on every
                    // platform) — this bare-`cfg` scope is only for the plain-missing-file
                    // tolerance, so a `cfg_attr` sighting must never grant it (its own absence
                    // tolerance is additive, via `cfg_attr_paths` below, not this flag).
                    b"cfg" => attrs.cfg = true,
                    // `#[cfg_attr(<pred>, …, path = "…")]`: extract EVERY `path = "…"` value from
                    // WITHIN this attribute's own argument list.
                    //
                    // The predicate of each group is skipped, so all three dimensions agree about which
                    // positions are applied targets — see [`path_meta_values`] for what reading it cost.
                    //
                    // Two axes, and only one of them was covered. A module may carry more than one
                    // SEPARATE `cfg_attr`-wrapped `#[path]`, one per platform predicate — this arm
                    // fires once per occurrence of the outer loop, so every one is collected. But ONE
                    // such attribute may also carry several, nested under one predicate, and reading
                    // the first alone made the rest invisible. [`path_meta_values`] records what that
                    // cost, measured against rustc.
                    //
                    // The reader tracks nesting, but only enough to answer two questions per open
                    // group: is this group a `cfg_attr`'s own argument list, and has that group's
                    // predicate been passed. A doubly- (or deeper-) nested
                    // `#[cfg_attr(a, cfg_attr(b, path = "…"))]` therefore resolves the same way a
                    // single-level one does — measured directly
                    // (`a_doubly_nested_cfg_attr_path_is_followed_the_same_as_a_single_nesting`) —
                    // while a predicate's own group answers neither. It said *anywhere in the argument
                    // span rather than parsing nesting structure*, which is what read predicates as
                    // targets.
                    b"cfg_attr" => {
                        let paren_open = skip_preamble_trivia(bytes, name_end, mod_index);
                        if bytes.get(paren_open) == Some(&b'(') {
                            let paren_close = paren_group_end(bytes, paren_open, mod_index);
                            attrs.cfg_attr_paths.extend(path_meta_values(
                                bytes,
                                paren_open + 1,
                                paren_close,
                                mod_index,
                            ));
                        }
                    }
                    _ => {}
                }
                i = attr_group_end(bytes, open, mod_index);
                continue;
            }
        }
        i += 1;
    }
    attrs
}

/// Index just past the `)` closing the paren group opened at `open` (which indexes the `(`),
/// tracking nested `()` and skipping string/char literals and comments so a `)` inside a
/// `#[cfg_attr(unix, path = "a)b.rs")]` literal does not close the group early. Mirrors
/// [`attr_group_end`]'s `[]`-tracking for a `cfg_attr`'s own argument list.
pub(crate) fn paren_group_end(bytes: &[u8], open: usize, limit: usize) -> usize {
    delimiter_group_end(bytes, open, limit, b'(', b')')
}

/// **Every** `path = "…"` name-value meta in `[start, paren_close)` — the interior of a `cfg_attr`'s
/// own argument list, bounded separately by `mod_index` for `read_path_string`'s own trivia skip — in
/// the textual order they appear. Empty if none is present.
///
/// Scans identifier-by-identifier rather than by a raw substring search, so a predicate that merely
/// contains the text (`target_os = "path_os"`, a doc comment) is never mistaken for the applied `path`
/// meta.
///
/// **A `path` in a predicate position is a cfg key, not a target** — before a `cfg_attr`'s own first comma,
/// or anywhere inside a compound predicate such as `all(…)`. `#[cfg_attr(path = "bogus",
/// path = "real.rs")] mod plat;` is legal source whatever cfg flags are set, and reading `bogus` as a
/// target scanned a file rustc does not compile — a probe inside it then counted as coverage and the audit
/// reported clean over a seam nothing probes on any real build. Each open group carries **two** facts —
/// what kind of group it is, and whether its own predicate has been passed — because a phase alone read a
/// compound predicate's comma as a `cfg_attr`'s. 圭表's own byte scanner draws the same distinction, so a
/// comment here claiming the shape could not be met in a real tree and that closing it needed a nesting
/// parser was wrong twice over.
///
/// **It returned only the first, and one attribute may carry several.** Measured against rustc
/// (edition 2021, `--crate-type lib`), this compiles cleanly on Linux with only `linux.rs` on disk and
/// neither `mac.rs` nor `plat.rs` present:
///
/// ```text
/// #[cfg_attr(unix, cfg_attr(target_os = "macos", path = "mac.rs"),
///                  cfg_attr(target_os = "linux", path = "linux.rs"))]
/// pub mod plat;
/// ```
///
/// Reading the first alone answered `mac.rs`, which resolves to nothing, so a module with no
/// conventional file behind it reported *cannot resolve reachable module* — **exit 2 over valid code**.
/// The union is not a hypothesis here: 圭表 already collects every `path =` across nested groups, so
/// answering the first was the one dimension of the three disagreeing about a shape rustc accepts.
pub(crate) fn path_meta_values(
    bytes: &[u8],
    start: usize,
    paren_close: usize,
    mod_index: usize,
) -> Vec<String> {
    let mut found = Vec::new();
    // One frame per open group: whether the group is a `cfg_attr`'s own argument list, and whether that
    // group's PREDICATE has been passed.
    //
    // Both halves are needed and the first repair had only the second. `cfg_attr` takes its predicate
    // first and its applied metas after the first comma AT THAT LEVEL — but a compound predicate is a
    // parenthesised group too, so `all(unix, path = "bogus")` has a comma of its own, and a phase kept
    // per group read `bogus` as applied. A comma inside `all(…)`, `any(…)` or `not(…)` belongs to the
    // predicate grammar and says nothing about the surrounding `cfg_attr`.
    //
    // So a group is an applied-meta position only where its `(` follows the identifier `cfg_attr`.
    // Anything else carries no module target — a compound predicate, and equally another attribute taking
    // a `path` argument of its own.
    struct Group {
        applies_metas: bool,
        past_predicate: bool,
    }
    // The span handed in IS a `cfg_attr`'s argument list, by this arm's own construction.
    let mut groups = vec![Group {
        applies_metas: true,
        past_predicate: false,
    }];
    let mut last_ident_was_cfg_attr = false;
    // Whether the previous SIGNIFICANT token was a path separator. Tracked forward through trivia rather
    // than reconstructed by looking behind: a look-behind over whitespace alone stopped at the `/` of
    // `foo::/**/cfg_attr` and read the segment as unqualified, which is the same false coverage through a
    // third spelling. A comment is trivia and must not change what a path IS.
    let mut after_path_sep = false;
    let mut i = start;
    while i < paren_close {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(paren_close);
            continue;
        }
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if bytes[i] == b':' && bytes.get(i + 1) == Some(&b':') {
            after_path_sep = true;
            i += 2;
            continue;
        }
        match bytes[i] {
            b'(' => {
                groups.push(Group {
                    applies_metas: last_ident_was_cfg_attr,
                    past_predicate: false,
                });
                last_ident_was_cfg_attr = false;
                after_path_sep = false;
                i += 1;
                continue;
            }
            b')' => {
                groups.pop();
                last_ident_was_cfg_attr = false;
                after_path_sep = false;
                if groups.is_empty() {
                    // Unbalanced past this span's own group: nothing further is this attribute's.
                    return found;
                }
                i += 1;
                continue;
            }
            b',' => {
                if let Some(group) = groups.last_mut() {
                    group.past_predicate = true;
                }
                last_ident_was_cfg_attr = false;
                after_path_sep = false;
                i += 1;
                continue;
            }
            _ => {}
        }
        if is_ident_byte(bytes[i]) && (i == start || !is_ident_byte(bytes[i - 1])) {
            // **A raw identifier is ONE path segment, not three scanner events.** `r#` changes an
            // identifier's lexical spelling and not its name, so `r#cfg_attr` names `cfg_attr` — it is not
            // escaping a keyword here, since `cfg_attr` is not one, and the form is admitted for any
            // identifier. A reader that saw `r`, then `#`, then `cfg_attr` cleared the qualification twice
            // and read `foo::r#cfg_attr` as unqualified. The prefix is consumed with the segment it
            // belongs to, and the name compared is the one it spells. This file already draws the same
            // distinction for a macro name; the reader below did not.
            let mut name_start = i;
            if bytes[i] == b'r' && bytes.get(i + 1) == Some(&b'#') {
                let after = i + 2;
                if after < paren_close && is_ident_byte(bytes[after]) {
                    name_start = after;
                }
            }
            let mut name_end = name_start;
            while name_end < paren_close && is_ident_byte(bytes[name_end]) {
                name_end += 1;
            }
            let name = &bytes[name_start..name_end];
            // **The built-in is the SINGLE-segment path**, and matching the last identifier alone is not
            // that: `foo::cfg_attr(a, path = "…")` ends in the same word while being somebody else's
            // attribute, and reopening applied-meta scanning inside it restored the false coverage this
            // reader had just closed. A segment reached through `::` carries no module target — and
            // *reached through* is decided by the token before it, tracked forward past trivia, because a
            // look-behind over whitespace alone read `foo::/**/cfg_attr` as unqualified.
            last_ident_was_cfg_attr = name == b"cfg_attr" && !after_path_sep;
            after_path_sep = false;
            let applied = groups
                .last()
                .is_some_and(|group| group.applies_metas && group.past_predicate);
            if name == b"path" && applied {
                let eq = skip_preamble_trivia(bytes, name_end, paren_close);
                if bytes.get(eq) == Some(&b'=') {
                    found.extend(read_path_string(bytes, eq + 1, mod_index));
                }
            }
            i = name_end;
            continue;
        }
        after_path_sep = false;
        i += 1;
    }
    found
}

/// Advance past whitespace, comments, and string/char literals to the next significant byte
/// (bounded by `end`). Shared by the attribute walk so a comment or literal inside a preamble
/// never derails the meta-name match.
pub(crate) fn skip_preamble_trivia(bytes: &[u8], mut i: usize, end: usize) -> usize {
    while i < end {
        if let Some(next) = skip_literal_or_comment(bytes, i) {
            i = next.min(end);
            continue;
        }
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        break;
    }
    i
}

/// Index just past the `]` closing the attribute-bracket group opened at `open` (which indexes the
/// `[`), tracking nested `[]` and skipping string/char literals and comments so a `]` inside a
/// `#[path = "a]b.rs"]` literal does not close the group early. Mirrors [`balanced_brace_end`].
pub(crate) fn attr_group_end(bytes: &[u8], open: usize, limit: usize) -> usize {
    delimiter_group_end(bytes, open, limit, b'[', b']')
}

/// Skip a (possibly nested) block comment whose opening `/*` is at `i`, returning the index just
/// past its outermost `*/`. Rust block comments nest, so depth is tracked; an unterminated comment
/// runs to EOF. Shared by [`scan_source_with_markers`] (and its `#[cfg(test)]` `scan_source`
/// wrapper) and [`skip_trivia`] so the two cannot drift — the
/// original non-nested bug existed in *both* precisely because they were independent copies.
pub(crate) fn skip_block_comment(b: &[u8], mut i: usize) -> usize {
    let mut depth = 1usize;
    i += 2; // past the opening `/*`
    while i + 1 < b.len() && depth > 0 {
        if b[i] == b'/' && b[i + 1] == b'*' {
            depth += 1;
            i += 2;
        } else if b[i] == b'*' && b[i + 1] == b'/' {
            depth -= 1;
            i += 2;
        } else {
            i += 1;
        }
    }
    if depth > 0 { b.len() } else { i }
}

/// Walk source skipping comments / string & char literals, and when the `assert_boundary!`
/// probe marker appears in code, record whether its seam argument is a string literal
/// (auditable) or not (un-auditable). Declarations come from the passed `RuntimeBoundary` objects.
/// `file` labels an un-auditable probe so the reaction is actionable.
#[cfg(test)]
pub(crate) fn scan_source(source: &str, file: &str, probes: &mut Vec<Probe>) {
    scan_source_with_markers(source, file, DEFAULT_MARKERS, probes);
}

pub(crate) fn scan_source_with_markers(
    source: &str,
    file: &str,
    markers: &[&str],
    probes: &mut Vec<Probe>,
) {
    let b = source.as_bytes();
    // Resolved once per file: every `fn` body's byte range, owner-qualified (never a bare name —
    // see `fn_scopes`), so an un-auditable probe's enclosing item is looked up by position below.
    let scopes = fn_scopes(b);
    let mut i = 0;
    while i < b.len() {
        // Comments and string/char literals are skipped whole (one shared definition below), so
        // a marker or delimiter inside them is never mis-read.
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        // A left word boundary: `my_assert_boundary!` / `xassert_boundary!` are unrelated user
        // macros, not our probe. Require the preceding byte to be a non-identifier char so a
        // marker embedded in a longer identifier is not mis-counted as a probe.
        let left_boundary = i == 0 || !is_ident_byte(b[i - 1]);
        if left_boundary {
            if let Some((rest, marker)) = match_probe_marker(b, i, markers) {
                let owner = owner_for(&scopes, i);
                let (probe, next) = capture_probe(b, rest, marker, file, &owner);
                if let Some(probe) = probe {
                    probes.push(probe);
                }
                i = next;
                continue;
            }
        }
        // A foreign macro invocation / `macro_rules!` definition body is macro-generated or dead
        // code: a probe lexically inside it must not count as coverage (the 圭表 strip_macro_bodies
        // rule, reimplemented louke-locally — 三儀 ⊥ 三儀 forbids importing it). `assert_boundary!`'s
        // own `!` is consumed by the marker branch above (and `capture_probe` advances past it), so
        // a `!`-preceded-by-identifier reached here is always a FOREIGN macro; skip its balanced
        // body (and any probe nested in it) in one jump.
        if b[i] == b'!' {
            // A foreign macro's `!` may be separated from its name by whitespace (`some_macro !(…)`
            // is valid Rust), mirroring the probe marker's own gap tolerance — so look back past
            // whitespace for the name's last identifier byte before deciding this opens a macro
            // body. (A comment between the name and `!` stays a documented bound: rustfmt removes
            // it, and scanning back over a block comment is not worth the cost.)
            let mut name_end = i;
            while name_end > 0 && b[name_end - 1].is_ascii_whitespace() {
                name_end -= 1;
            }
            let mut name_start = name_end;
            while name_start > 0 && is_ident_byte(b[name_start - 1]) {
                name_start -= 1;
            }
            // A raw identifier `r#keyword` (e.g. a macro named `r#async`) escapes the keyword and IS
            // a valid macro name — its body must still be skipped. The ident-run stops at the `#`
            // (not an ident byte), so detect a preceding `r#` at a word boundary and exempt it from
            // the keyword test below.
            let is_raw_ident = name_start >= 2
                && b[name_start - 1] == b'#'
                && b[name_start - 2] == b'r'
                && (name_start == 2 || !is_ident_byte(b[name_start - 3]));
            // Otherwise the name before `!` must be a real identifier that is NOT a keyword. A
            // keyword there is unary negation in expression position (`return !(x)`, `if !(cond) {…}`,
            // `match !(x)`), never a macro — treating its parenthesized operand as a macro body would
            // skip real code (and drop any probe inside it). `macro_rules` is not a keyword, so it
            // still reaches `foreign_macro_body_end`'s name-skip.
            // The one transparent macro is NOT skipped: its arms hold real, compiled code, so the
            // scan walks into the body and observes a probe (or a typo'd seam, or an un-auditable
            // probe) there exactly as at top level. Ordering matters — a transparent invocation
            // written inside a `macro_rules!` definition is never reached, because that outer body
            // is skipped first, so the macro-definition exclusion is unaffected.
            if name_start < name_end
                && !is_transparent_macro_name(b, name_end)
                && (is_raw_ident || !is_rust_keyword(&b[name_start..name_end]))
            {
                if let Some(end) = foreign_macro_body_end(b, i) {
                    i = end;
                    continue;
                }
            }
        }
        i += 1;
    }
}

/// If `i` begins a comment or a string/char literal, return the index just past it; else `None`.
/// One shared definition for the main scan and the macro-body skip, so their literal/comment
/// handling can never drift apart (the independent-copy drift `skip_block_comment` warns about).
/// Raw/byte strings are tested before plain strings (an inner `"` would otherwise desync), and a
/// lifetime (`'a`) is deliberately NOT a literal (left to be walked as code).
pub(crate) fn skip_literal_or_comment(b: &[u8], i: usize) -> Option<usize> {
    // line comment
    if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'/' {
        let mut j = i;
        while j < b.len() && b[j] != b'\n' {
            j += 1;
        }
        return Some(j);
    }
    // block comment (nesting + drift rationale in `skip_block_comment`)
    if b[i] == b'/' && i + 1 < b.len() && b[i + 1] == b'*' {
        return Some(skip_block_comment(b, i));
    }
    // raw / byte string literal (r"…", r#"…"#, b"…", br#"…"#) — before the plain-string case
    if let Some(end) = raw_or_byte_string_end(b, i) {
        return Some(end);
    }
    // plain string literal
    if b[i] == b'"' {
        let mut j = i + 1;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    // char literal vs lifetime: only a clear char ('x' or '\n'); a lifetime ('a) is not a literal.
    if b[i] == b'\'' {
        let is_char =
            (i + 1 < b.len() && b[i + 1] == b'\\') || (i + 2 < b.len() && b[i + 2] == b'\'');
        if is_char {
            let mut j = i + 1;
            while j < b.len() && b[j] != b'\'' {
                if b[j] == b'\\' {
                    j += 1;
                }
                j += 1;
            }
            return Some((j + 1).min(b.len()));
        }
    }
    None
}

/// The identifier run ending immediately before `end` equals `target`. Used to recognize a
/// `macro_rules` keyword before its `!` (the only stable form taking a `name` between `!` and the
/// body delimiter) without a false match on `my_macro_rules` (the maximal run differs).
pub(crate) fn preceding_ident_is(b: &[u8], end: usize, target: &[u8]) -> bool {
    let mut start = end;
    while start > 0 && is_ident_byte(b[start - 1]) {
        start -= 1;
    }
    &b[start..end] == target
}

/// The identifier run ending immediately before `end` (whitespace already stepped over by the
/// caller) is the one **transparent control-flow macro**, `cfg_if!`. See
/// `runtime-origin-assertion`'s "CI face — every declared seam is probed" requirement for why
/// scanning into its arms (rather than skipping them as macro-generated) closes a coverage false
/// negative, and why the gate is on the macro **name** rather than any body-wrapping macro. Matches
/// 圭表's `is_transparent_macro_name` and 渾儀's own test — the same rule in three hand-written
/// copies, never a shared scanner (三儀 ⊥ 三儀), with `cfg_if_transparency_conformance.rs` as the
/// drift reaction.
pub(crate) fn is_transparent_macro_name(b: &[u8], end: usize) -> bool {
    preceding_ident_is(b, end, b"cfg_if")
}

/// Given `bang` where `b[bang] == b'!'` and the preceding byte is an identifier byte, return the
/// index past a foreign macro's balanced body, or `None` when this `!` does not open one — `!=`,
/// unary `!expr`, or a keyword glued to `!` (`if!cond {…}` / `while!x {…}` / `match!x {…}`), none of
/// which is a macro. `macro_rules! name {…}` is the sole form with an identifier between `!` and the
/// delimiter, so the name-skip is gated on the preceding identifier being exactly `macro_rules`;
/// treating any `ident! ident {` as a macro would swallow a real `if`/`while`/`match` block and drop
/// a probe inside it (a reintroduced false negative). The balanced walk reuses
/// `skip_literal_or_comment`, so a delimiter inside a string/char/comment never closes early; an
/// unterminated body at EOF returns `Some(len)`.
pub(crate) fn foreign_macro_body_end(b: &[u8], bang: usize) -> Option<usize> {
    let mut i = skip_trivia(b, bang + 1);
    // The name may be separated from `!` by whitespace (`macro_rules ! foo {…}` is valid Rust),
    // exactly as the caller tolerates when deciding this `!` opens a macro. Skip back over that
    // whitespace before the keyword test — anchoring at `bang` would miss the spaced form, leaving
    // the body (and any probe inside it) unskipped and wrongly counted as coverage (a false negative).
    let mut name_end = bang;
    while name_end > 0 && b[name_end - 1].is_ascii_whitespace() {
        name_end -= 1;
    }
    if preceding_ident_is(b, name_end, b"macro_rules") {
        let name_start = i;
        while i < b.len() && (is_ident_byte(b[i]) || b[i] == b'#') {
            i += 1;
        }
        if i == name_start {
            return None; // `macro_rules!` with no name — malformed, not a body to skip
        }
        i = skip_trivia(b, i);
    }
    if !matches!(b.get(i), Some(b'{') | Some(b'(') | Some(b'[')) {
        return None;
    }
    // One depth counter over all three delimiter kinds: correct because the audit scans compilable
    // Rust, whose token trees are properly nested (a `)` never closes a `{`). Literals/comments are
    // skipped first each iteration, so a delimiter inside a string/char never perturbs the count.
    let mut depth = 0usize;
    while i < b.len() {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        match b[i] {
            b'{' | b'(' | b'[' => depth += 1,
            b'}' | b')' | b']' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return Some(i + 1);
                }
            }
            _ => {}
        }
        i += 1;
    }
    Some(b.len())
}

/// Detect a raw or byte string literal starting at `i` (`r"…"`, `r#"…"#`, `b"…"`,
/// `br"…"`, `br#"…"#`) and return the index past its end, or `None` if `i` is not such a
/// literal. Rust syntax guarantees `r`/`b` immediately before `"`/`#` is a literal prefix
/// (no identifier can precede a string), so no token-boundary check is needed.
pub(crate) fn raw_or_byte_string_end(b: &[u8], i: usize) -> Option<usize> {
    let mut j = i;
    let byte = j < b.len() && b[j] == b'b';
    if byte {
        j += 1;
    }
    let raw = j < b.len() && b[j] == b'r';
    if raw {
        j += 1;
        let mut hashes = 0;
        while j < b.len() && b[j] == b'#' {
            hashes += 1;
            j += 1;
        }
        if j >= b.len() || b[j] != b'"' {
            return None;
        }
        j += 1;
        // scan to the closing `"` followed by `hashes` `#`s
        while j < b.len() {
            if b[j] == b'"' {
                let mut k = j + 1;
                let mut h = 0;
                while k < b.len() && h < hashes && b[k] == b'#' {
                    k += 1;
                    h += 1;
                }
                if h == hashes {
                    return Some(k);
                }
            }
            j += 1;
        }
        return Some(b.len());
    }
    // a `b"…"` byte string (escaped like a normal string) — only when a `b` prefix was
    // consumed and a quote immediately follows.
    if byte && j < b.len() && b[j] == b'"' {
        j += 1;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        return Some((j + 1).min(b.len()));
    }
    None
}

/// Match a configured probe marker at `i`: the identifier at a word boundary, then — as
/// `ident ! (…)` with whitespace/comments between the name and `!` is valid Rust (`println !("x")`
/// compiles) — its `!`. Returns the index just past the `!`, whence [`capture_probe`] skips trivia
/// to the opening delimiter; `None` otherwise. The right word boundary rejects a longer identifier
/// like `assert_boundaryx`; the caller checks the left boundary. Identifier matching delegates to
/// [`match_keyword`] so probe markers and structural keywords share one right-boundary rule.
/// Tolerating the gap closes a false negative: a probe written `assert_boundary !("seam")` was
/// silently dropped by a contiguous match.
pub(crate) fn match_probe_marker<'a>(
    b: &[u8],
    i: usize,
    markers: &[&'a str],
) -> Option<(usize, &'a str)> {
    for &marker in markers {
        let name = marker.as_bytes();
        if let Some(after_name) = match_keyword(b, i, name) {
            let bang = skip_trivia(b, after_name);
            if b.get(bang) == Some(&b'!') {
                return Some((bang + 1, marker));
            }
        }
    }
    None
}

/// An identifier byte — ASCII `[A-Za-z0-9_]` or any UTF-8 non-ASCII byte (`>= 0x80`). Used for the
/// marker's word boundary: a multi-byte Unicode identifier char (`Ω` in `Ωassert_boundary`) is XID
/// and must keep the boundary, so a foreign macro whose name merely *ends* in `assert_boundary` is
/// not mis-read as our probe. ASCII-only would treat the `Ω` continuation bytes as a boundary and
/// falsely match (a false coverage / fabricated probed-but-undeclared reaction).
pub(crate) fn is_ident_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte >= 0x80
}

/// Whether `marker` is one valid Rust macro identifier accepted by the configurable probe scan.
///
/// Plain keywords and invalid identifier spellings are rejected; raw identifiers may escape a
/// keyword except for Rust's non-escapable path/self names.
pub(crate) fn is_valid_macro_marker(marker: &str) -> bool {
    if marker.is_empty() {
        return false;
    }
    let (ident_str, is_raw) = if let Some(stripped) = marker.strip_prefix("r#") {
        (stripped, true)
    } else {
        (marker, false)
    };

    if ident_str.is_empty() {
        return false;
    }

    if is_raw {
        if matches!(ident_str, "self" | "Self" | "super" | "crate" | "_") {
            return false;
        }
    } else {
        if ident_str == "_" {
            return false;
        }
        if is_rust_keyword(ident_str.as_bytes()) {
            return false;
        }
    }

    let bytes = ident_str.as_bytes();
    let first = bytes[0];
    if !first.is_ascii_alphabetic() && first != b'_' {
        return false;
    }

    bytes
        .iter()
        .all(|&b| b.is_ascii_alphanumeric() || b == b'_')
}

/// Whether the identifier run `word` is a Rust keyword (strict or reserved). A macro name is a real
/// identifier and never a keyword, so a keyword immediately before `!` is unary negation
/// (`return !(x)`, `if !(cond) {…}`), not a macro invocation — its operand must not be skipped as a
/// macro body. `macro_rules` is deliberately absent (it is not a keyword and must reach the
/// name-skip). A non-ASCII / non-UTF-8 run is never a keyword.
pub(crate) fn is_rust_keyword(word: &[u8]) -> bool {
    let Ok(word) = std::str::from_utf8(word) else {
        return false;
    };
    matches!(
        word,
        "as" | "break"
            | "const"
            | "continue"
            | "crate"
            | "dyn"
            | "else"
            | "enum"
            | "extern"
            | "false"
            | "fn"
            | "for"
            | "if"
            | "impl"
            | "in"
            | "let"
            | "loop"
            | "match"
            | "mod"
            | "move"
            | "mut"
            | "pub"
            | "ref"
            | "return"
            | "self"
            | "Self"
            | "static"
            | "struct"
            | "super"
            | "trait"
            | "true"
            | "type"
            | "unsafe"
            | "use"
            | "where"
            | "while"
            | "async"
            | "await"
            // reserved / edition keywords
            | "abstract"
            | "become"
            | "box"
            | "do"
            | "final"
            | "macro"
            | "override"
            | "priv"
            | "typeof"
            | "unsized"
            | "virtual"
            | "yield"
            | "try"
            | "gen"
    )
}

/// Skip ASCII whitespace and `//` / `/* */` comments, returning the next code index. Mirrors
/// the comment handling in [`scan_source_with_markers`] so a comment between the `!` and `(`, or before the
/// seam argument, does not desync probe capture (which would silently drop a real probe).
pub(crate) fn skip_trivia(b: &[u8], mut i: usize) -> usize {
    loop {
        while i < b.len() && b[i].is_ascii_whitespace() {
            i += 1;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'/') {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b.get(i) == Some(&b'/') && b.get(i + 1) == Some(&b'*') {
            i = skip_block_comment(b, i);
            continue;
        }
        return i;
    }
}

/// After a configured probe marker, classify the probe by its first argument and return
/// `(probe, next_index)`. Skip trivia, expect a macro opening delimiter (`(`, `{`, or `[`),
/// skip trivia; a plain or raw string first argument is an auditable [`Probe::Literal`] (its
/// value); any other first token (a `const`, an expression, a byte string) is
/// [`Probe::Unauditable`] — never a silent skip. `None` (with `next` past the marker) only when
/// the marker is not actually a probe call (no opening delimiter follows). `owner` is the
/// caller-supplied, already-resolved owner-qualified enclosing item (see `fn_scopes`), threaded
/// straight into an `Unauditable` probe's identity.
pub(crate) fn capture_probe(
    b: &[u8],
    i: usize,
    marker: &str,
    file: &str,
    owner: &str,
) -> (Option<Probe>, usize) {
    let i = skip_trivia(b, i);
    // Rust macros accept `( )`, `{ }`, or `[ ]` interchangeably; a probe written
    // `assert_boundary!{"s", o}` or `["s", o]` is a real probe. Accept any of the three
    // opening delimiters so a non-`()` probe is not silently dropped — a silent drop would let
    // a typo'd seam escape the undeclared-seam check, a false negative.
    if !matches!(b.get(i), Some(&b'(') | Some(&b'{') | Some(&b'[')) {
        return (None, i);
    }
    let i = skip_trivia(b, i + 1);
    if i >= b.len() {
        return (None, i);
    }
    // The offending expression's own trimmed source text, captured once regardless of which
    // un-auditable branch below is taken — this is the identity discriminator (never a byte
    // offset), so two textually distinct non-literal probes never collapse to one finding.
    let unauditable = |b: &[u8]| -> Probe {
        let end = first_macro_arg_end(b, i);
        let expr = String::from_utf8_lossy(trim_bytes(&b[i..end])).into_owned();
        Probe::Unauditable {
            marker: marker.to_string(),
            file: file.to_string(),
            owner: owner.to_string(),
            expr,
        }
    };
    // A raw string `r"…"` / `r#"…"#` is a traceable literal — parse its value rather than
    // rejecting it as un-auditable (which would mis-flag a legitimate probe and double-report).
    if b[i] == b'r' && matches!(b.get(i + 1), Some(b'"') | Some(b'#')) {
        if let Some((seam, next)) = raw_string_value(b, i) {
            return (Some(Probe::Literal(seam)), next);
        }
        return (Some(unauditable(b)), i);
    }
    // A plain string literal. Find its end (the `\\`-skip only keeps a `\"` from ending the
    // string early), then DECODE its escapes to the value the compiler produces — the declared
    // seam set is compiler-decoded (`RuntimeBoundary::seam()`), so comparing the raw source bytes
    // would let an escape-bearing seam diverge between the two faces (a false pair of reactions,
    // and a false negative when two spellings decode to the same bytes). An escape the decoder
    // cannot reproduce exactly reacts as un-auditable (loud), never a silently mismatched literal.
    if b[i] == b'"' {
        let mut j = i + 1;
        let start = j;
        while j < b.len() && b[j] != b'"' {
            if b[j] == b'\\' {
                j += 1;
            }
            j += 1;
        }
        if j >= b.len() {
            return (None, j);
        }
        return match decode_str_escapes(&b[start..j]) {
            Some(seam) => (Some(Probe::Literal(seam)), j + 1),
            None => (Some(unauditable(b)), j + 1),
        };
    }
    // Anything else (a const, an expression, a byte string) cannot be traced to a declared seam.
    (Some(unauditable(b)), i)
}

/// Find the end of a macro's first argument starting at `open` (just past the opening delimiter
/// and any leading trivia): the index of a top-level comma, or the matching close delimiter if no
/// comma precedes it. Tracks nesting over all three delimiter kinds — the same one-depth-counter
/// model `foreign_macro_body_end` uses for a whole macro body — so a nested call or index in the
/// seam expression (`assert_boundary!(some_fn(a, b), obj)`, `assert_boundary!(TABLE[i], obj)`) is
/// not mistaken for the argument's own end.
pub(crate) fn first_macro_arg_end(b: &[u8], open: usize) -> usize {
    let mut depth = 0usize;
    let mut angle_depth = 0usize;
    let mut last_token_was_double_colon = false;
    let mut last_token_was_ident_or_gt = false;
    let mut last_token_was_as = false;
    let mut i = open;
    while i < b.len() {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if i + 1 < b.len() && b[i] == b':' && b[i + 1] == b':' {
            last_token_was_double_colon = true;
            last_token_was_ident_or_gt = false;
            last_token_was_as = false;
            i += 2;
            continue;
        }
        if is_ident_byte(b[i]) {
            let start = i;
            last_token_was_ident_or_gt = true;
            last_token_was_double_colon = false;
            while i < b.len() && is_ident_byte(b[i]) {
                i += 1;
            }
            last_token_was_as = &b[start..i] == b"as";
            continue;
        }
        match b[i] {
            b'(' | b'{' | b'[' => depth += 1,
            b')' | b'}' | b']' => {
                if depth == 0 {
                    return i;
                }
                depth = depth.saturating_sub(1);
            }
            b'<' => {
                if depth == 0 {
                    let is_qualified_start = is_unary_prefix_span(&b[open..i]);
                    let is_turbofish = last_token_was_double_colon;
                    let is_inner_generic = angle_depth > 0 && last_token_was_ident_or_gt;
                    if is_qualified_start || is_turbofish || is_inner_generic || last_token_was_as {
                        angle_depth += 1;
                    }
                }
            }
            b'>' => {
                if depth == 0 && angle_depth > 0 {
                    let prev = if i > open { b[i - 1] } else { b'\0' };
                    if prev != b'-' {
                        angle_depth -= 1;
                    }
                }
                last_token_was_ident_or_gt = true;
            }
            b',' if depth == 0 && angle_depth == 0 => return i,
            _ => {}
        }
        last_token_was_double_colon = false;
        last_token_was_as = false;
        if b[i] != b'>' {
            last_token_was_ident_or_gt = false;
        }
        i += 1;
    }
    b.len()
}

/// Whether `b` consists only of ASCII whitespace, comments, and unary / prefix operator tokens
/// (`&`, `*`, `!`, `-`, `+`, `mut`, `ref`). Value literals (strings, chars, numbers) return false.
pub(crate) fn is_unary_prefix_span(b: &[u8]) -> bool {
    let mut i = 0;
    while i < b.len() {
        let next = skip_trivia(b, i);
        if next != i {
            i = next;
            continue;
        }
        if matches!(b[i], b'&' | b'*' | b'!' | b'-' | b'+') {
            i += 1;
            continue;
        }
        if b[i..].starts_with(b"mut") && (i + 3 == b.len() || !is_ident_byte(b[i + 3])) {
            i += 3;
            continue;
        }
        if b[i..].starts_with(b"ref") && (i + 3 == b.len() || !is_ident_byte(b[i + 3])) {
            i += 3;
            continue;
        }
        return false;
    }
    true
}

/// Trim ASCII whitespace from both ends of a byte slice (a `str::trim` that stays on raw bytes,
/// since the captured text is not yet known to be valid UTF-8 at the trim point).
pub(crate) fn trim_bytes(b: &[u8]) -> &[u8] {
    let start = b
        .iter()
        .position(|c| !c.is_ascii_whitespace())
        .unwrap_or(b.len());
    let end = b
        .iter()
        .rposition(|c| !c.is_ascii_whitespace())
        .map_or(start, |p| p + 1);
    &b[start..end]
}

/// An `impl`'s owner context: an inherent impl carries only its `Self` type; a trait impl also
/// carries the trait path. Qualifies a nested `fn`'s owner (never a bare method name — two owners
/// may share one), mirroring `hunyi`'s `owner`/`trait_ref` qualification for the identical
/// same-named-item collision (`semantic-unsafe-confinement`).
pub(crate) enum ImplOrTraitContext {
    Impl {
        trait_ref: Option<String>,
        self_ty: String,
    },
    Trait(String),
}

/// Render a `fn`'s owner-qualified identity string from the accumulated inline-`mod` path, the
/// innermost enclosing `impl`/`trait` context (if any), and the fn's own name. Never a bare method
/// or fn name alone: the module path additionally disambiguates two same-named free `fn`s (or two
/// same-named local types) declared in different inline `mod { … }` blocks of the *same* file —
/// two same-named items in *different files* are already distinguished by the outer `file` field,
/// so this only needs to cover same-file `mod` nesting, not cross-file module identity.
pub(crate) fn render_owner(
    module_path: &str,
    enclosing_fn: Option<&str>,
    context_stack: &[(usize, ImplOrTraitContext)],
    fn_name: &str,
) -> String {
    let prefix = if module_path.is_empty() {
        String::new()
    } else {
        format!("{module_path}::")
    };
    let body = match context_stack.last() {
        Some((
            _,
            ImplOrTraitContext::Impl {
                trait_ref: Some(trait_ref),
                self_ty,
            },
        )) => format!("impl {trait_ref} for {self_ty}::{fn_name}"),
        Some((
            _,
            ImplOrTraitContext::Impl {
                trait_ref: None,
                self_ty,
            },
        )) => {
            format!("impl {self_ty}::{fn_name}")
        }
        Some((_, ImplOrTraitContext::Trait(name))) => format!("trait {name}::{fn_name}"),
        None => format!("fn {fn_name}"),
    };
    match enclosing_fn {
        Some(enclosing) => format!("{enclosing}::{body}"),
        None => format!("{prefix}{body}"),
    }
}

pub(crate) fn anonymous_scope_header(b: &[u8], start: usize, brace: usize) -> String {
    let header = String::from_utf8_lossy(trim_bytes(&b[start..brace]));
    let normalized = header.split_whitespace().collect::<Vec<_>>().join(" ");
    if normalized.is_empty() {
        "<block>".to_string()
    } else {
        normalized
    }
}

pub(crate) fn enclosing_owner(
    named_owner: Option<&str>,
    anonymous_stack: &[(usize, String)],
) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(named_owner) = named_owner {
        parts.push(named_owner.to_string());
    }
    parts.extend(anonymous_stack.iter().map(|(_, scope)| scope.clone()));
    (!parts.is_empty()).then(|| parts.join("::"))
}

/// Match a bare keyword identifier at `i` (e.g. `fn`, `impl`, `trait`), requiring a right word
/// boundary so `implx`/`fnx` is not mistaken for the keyword — mirrors [`match_probe_marker`]'s
/// own boundary discipline. The caller checks the left boundary.
pub(crate) fn match_keyword(b: &[u8], i: usize, name: &[u8]) -> Option<usize> {
    if i + name.len() > b.len() || &b[i..i + name.len()] != name {
        return None;
    }
    let after = i + name.len();
    if b.get(after).is_some_and(|&c| is_ident_byte(c)) {
        return None;
    }
    Some(after)
}

/// Find `keyword` (`for` or `where`) at top-level depth (outside any `<…>`/`(…)`/`[…]`) in
/// `header`, respecting word boundaries and skipping string/char/comment content, returning its
/// start index. Used to split an `impl` header without being fooled by a `for`/`where` nested in a
/// generic bound (e.g. an HRTB `for<'a>` inside `<…>`). `>` only closes a generic level when not
/// preceded by `-` (excluding a `->` return-arrow), matching `skip_to_item_body`'s own rule.
pub(crate) fn find_top_level_keyword(header: &[u8], keyword: &[u8]) -> Option<usize> {
    let mut depth = 0usize;
    let mut i = 0;
    while i < header.len() {
        if let Some(next) = skip_literal_or_comment(header, i) {
            i = next;
            continue;
        }
        match header[i] {
            b'(' | b'[' | b'<' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b'>' if i == 0 || header[i - 1] != b'-' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth == 0 {
            let left_boundary = i == 0 || !is_ident_byte(header[i - 1]);
            if left_boundary && match_keyword(header, i, keyword).is_some() {
                return Some(i);
            }
        }
        i += 1;
    }
    None
}

/// Skip from `i` (just past a `fn`/`trait` name, or the start of an `impl` header) past any
/// generics/parameter-list/return-type/where-clause to the item's own opening `{`, returning the
/// index just past that brace — or `None` if a top-level `;` is reached first (a body-less trait
/// method declaration, which contributes no scope). Tracks `(`/`)`/`[`/`]`/`<`/`>` nesting (not
/// `{`/`}`, which is exactly what the caller is deciding whether it has reached) so a generic
/// bound or parameter type containing any of these does not false-trigger the terminator search.
/// Stated bound: a const-generic default expression using a shift operator (`<<`/`>>`) before the
/// item's own body is not specially handled — vanishingly rare in a bare `fn`/`impl`/`trait`
/// header and not attempted here.
pub(crate) fn skip_to_item_body(b: &[u8], mut i: usize) -> Option<usize> {
    let mut depth = 0usize;
    while i < b.len() {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        match b[i] {
            b'(' | b'[' | b'<' => depth += 1,
            b')' | b']' => depth = depth.saturating_sub(1),
            b'>' if i == 0 || b[i - 1] != b'-' => depth = depth.saturating_sub(1),
            b'{' if depth == 0 => return Some(i + 1),
            b';' if depth == 0 => return None,
            _ => {}
        }
        i += 1;
    }
    None
}

/// After the `fn`/`trait` keyword (`after_keyword` just past it), parse the item's name and skip
/// to its opening `{`. Returns `None` for a malformed/nameless item or a body-less declaration.
pub(crate) fn parse_named_item_header(b: &[u8], after_keyword: usize) -> Option<(String, usize)> {
    let name_start = skip_trivia(b, after_keyword);
    let ident_start = if b.get(name_start..name_start + 2) == Some(b"r#") {
        name_start + 2
    } else {
        name_start
    };
    let mut name_end = ident_start;
    while name_end < b.len() && is_ident_byte(b[name_end]) {
        name_end += 1;
    }
    if name_end == ident_start {
        return None;
    }
    let name = String::from_utf8_lossy(&b[ident_start..name_end]).into_owned();
    let body_start = skip_to_item_body(b, name_end)?;
    Some((name, body_start))
}

/// After the `impl` keyword (`after_impl` just past it), parse the header up to its opening `{`,
/// splitting an optional `Trait for Self` header into `(Some(trait), self)` and an inherent
/// `impl Self` header into `(None, self)`. The split searches for a top-level `for` only before
/// any top-level `where` (a `where`-clause `for<'a>` HRTB must never be mistaken for the impl's
/// own `for`). Returns `None` only on malformed/truncated input (an `impl` always has a body).
pub(crate) fn parse_impl_header(
    b: &[u8],
    after_impl: usize,
) -> Option<(ImplOrTraitContext, usize)> {
    let header_start = skip_trivia(b, after_impl);
    let body_start = skip_to_item_body(b, header_start)?;
    let header = &b[header_start..body_start - 1];
    let search_region = match find_top_level_keyword(header, b"where") {
        Some(w) => &header[..w],
        None => header,
    };
    let ctx = match find_top_level_keyword(search_region, b"for") {
        Some(for_at) => ImplOrTraitContext::Impl {
            trait_ref: Some(String::from_utf8_lossy(trim_bytes(&header[..for_at])).into_owned()),
            self_ty: String::from_utf8_lossy(trim_bytes(&header[for_at + 3..])).into_owned(),
        },
        None => ImplOrTraitContext::Impl {
            trait_ref: None,
            self_ty: String::from_utf8_lossy(trim_bytes(header)).into_owned(),
        },
    };
    Some((ctx, body_start))
}

/// Mutable scan state threaded through [`fn_scopes`]'s single byte-walk: the open
/// mod/impl-or-trait/fn/anonymous-block stacks (each paired with the depth it opened at, so a `}`
/// pops exactly the frames it closes), the sibling-numbering map for anonymous blocks, the running
/// brace depth / current-header start the walk advances, and the completed `(body_start, body_end,
/// owner)` triples.
struct FnScopeState {
    depth: usize,
    // Accumulated inline `mod name { … }` nesting — an external `mod name;` (no body in this
    // file) contributes nothing here, since its content is scanned separately, as its own file,
    // where the outer `file` identity field already disambiguates it.
    mod_stack: Vec<(usize, String)>,
    context_stack: Vec<(usize, ImplOrTraitContext)>,
    fn_stack: Vec<(usize, usize, String)>,
    anonymous_stack: Vec<(usize, String)>,
    anonymous_siblings: HashMap<(String, String), usize>,
    // Start of the current code header, advanced only by code delimiters observed by this
    // literal/comment-aware walk. Punctuation inside skipped literals/comments never becomes an
    // anonymous-scope boundary.
    anonymous_header_start: usize,
    out: Vec<(usize, usize, String)>,
}

impl FnScopeState {
    fn new() -> Self {
        Self {
            depth: 0,
            mod_stack: Vec::new(),
            context_stack: Vec::new(),
            fn_stack: Vec::new(),
            anonymous_stack: Vec::new(),
            anonymous_siblings: HashMap::new(),
            anonymous_header_start: 0,
            out: Vec::new(),
        }
    }

    /// Try each header keyword (`mod`/`impl`/`trait`/`fn`) at `i` in `b`; on a match, push the
    /// opened scope's frame and return the walk's next position (the body start) — the caller
    /// `continue`s the outer loop from there without falling through to the brace/`;` handling.
    /// `None` when no keyword matches at `i`.
    fn try_dispatch_keyword(&mut self, b: &[u8], i: usize) -> Option<usize> {
        if let Some(rest) = match_keyword(b, i, b"mod") {
            if let Some((name, body_start)) = parse_named_item_header(b, rest) {
                self.mod_stack.push((self.depth, name));
                self.depth += 1;
                self.anonymous_header_start = body_start;
                return Some(body_start);
            }
        } else if let Some(rest) = match_keyword(b, i, b"impl") {
            if let Some((ctx, body_start)) = parse_impl_header(b, rest) {
                self.context_stack.push((self.depth, ctx));
                self.depth += 1;
                self.anonymous_header_start = body_start;
                return Some(body_start);
            }
        } else if let Some(rest) = match_keyword(b, i, b"trait") {
            if let Some((name, body_start)) = parse_named_item_header(b, rest) {
                self.context_stack
                    .push((self.depth, ImplOrTraitContext::Trait(name)));
                self.depth += 1;
                self.anonymous_header_start = body_start;
                return Some(body_start);
            }
        } else if let Some(rest) = match_keyword(b, i, b"fn") {
            if let Some((name, body_start)) = parse_named_item_header(b, rest) {
                let module_path = self
                    .mod_stack
                    .iter()
                    .map(|(_, name)| name.as_str())
                    .collect::<Vec<_>>()
                    .join("::");
                let enclosing = enclosing_owner(
                    self.fn_stack.last().map(|(_, _, owner)| owner.as_str()),
                    &self.anonymous_stack,
                );
                let owner = render_owner(
                    &module_path,
                    enclosing.as_deref(),
                    &self.context_stack,
                    &name,
                );
                self.fn_stack.push((self.depth, body_start, owner));
                self.depth += 1;
                self.anonymous_header_start = body_start;
                return Some(body_start);
            }
        }
        None
    }

    /// Open a new anonymous-block frame at `i`, headed by the code since
    /// `anonymous_header_start` and numbered against its siblings under the same parent+header.
    fn open_brace(&mut self, b: &[u8], i: usize) {
        let header = anonymous_scope_header(b, self.anonymous_header_start, i);
        let parent = enclosing_owner(
            self.fn_stack.last().map(|(_, _, owner)| owner.as_str()),
            &self.anonymous_stack,
        )
        .unwrap_or_else(|| "<module scope>".to_string());
        let sibling = self
            .anonymous_siblings
            .entry((parent, header.clone()))
            .and_modify(|count| *count += 1)
            .or_insert(1);
        self.anonymous_stack
            .push((self.depth, format!("block {header}#{sibling}")));
        self.depth += 1;
        self.anonymous_header_start = i + 1;
    }

    /// Close the brace at `i`: pop every stack frame opened at the depth this brace closes,
    /// recording a completed `(body_start, i, owner)` triple for a closed `fn` scope.
    fn close_brace(&mut self, i: usize) {
        self.depth = self.depth.saturating_sub(1);
        if self
            .fn_stack
            .last()
            .is_some_and(|&(open_depth, _, _)| open_depth == self.depth)
        {
            let (_, body_start, owner) = self.fn_stack.pop().expect("checked Some above");
            self.out.push((body_start, i, owner));
        }
        if self
            .context_stack
            .last()
            .is_some_and(|&(open_depth, _)| open_depth == self.depth)
        {
            self.context_stack.pop();
        }
        if self
            .mod_stack
            .last()
            .is_some_and(|&(open_depth, _)| open_depth == self.depth)
        {
            self.mod_stack.pop();
        }
        if self
            .anonymous_stack
            .last()
            .is_some_and(|&(open_depth, _)| open_depth == self.depth)
        {
            self.anonymous_stack.pop();
        }
        self.anonymous_header_start = i + 1;
    }
}

/// Every owner-qualified `fn` body in this source file, as `(body_start, body_end, owner)` byte
/// ranges — `body_start`/`body_end` bound just inside the fn's own `{ … }` (excluding the braces
/// themselves). Looked up by [`owner_for`] so an un-auditable probe's identity is qualified by a
/// real structural discriminator, never a bare name or a position.
///
/// Deliberately does not skip macro-invocation/`macro_rules!` bodies the way `scan_source` does.
/// The two cases this leaves, both fine:
///
/// - A **non-transparent** macro body still yields no probe (`scan_source` skips it before a probe
///   is ever captured), and a phantom `fn`/`impl`/`trait` mis-parsed out of its macro-template text
///   lies wholly inside that body's balanced braces, so its range can never overlap a real probe's
///   position — inert, not a correctness risk. (This was once the *whole* justification, resting on
///   "a probe is never found inside a macro body". Transparency retired that premise, so it is now
///   only half the story.)
/// - A **transparent** `cfg_if!` body does now yield probes, and this walk reads it as ordinary code
///   — which is exactly right: a `fn` inside an arm becomes a real scope. The invocation's own body
///   braces and the arm braces are counted as the anonymous block scopes they lexically are, so such
///   a probe's owner renders as `block cfg_if::cfg_if!#1::block if #[cfg(unix)]#1::fn f`. That is
///   this function's existing rule for any anonymous scope (a real `if` block reads the same way),
///   applied unchanged rather than special-cased for arms, and it names the arm in the violation
///   message — which an adopter reading it wants. Pinned by
///   `an_unauditable_probe_inside_a_cfg_if_arm_reacts_with_its_lexical_owner`.
pub(crate) fn fn_scopes(b: &[u8]) -> Vec<(usize, usize, String)> {
    let mut state = FnScopeState::new();
    let mut i = 0;
    while i < b.len() {
        if let Some(next) = skip_literal_or_comment(b, i) {
            i = next;
            continue;
        }
        let left_boundary = i == 0 || !is_ident_byte(b[i - 1]);
        if left_boundary {
            if let Some(next) = state.try_dispatch_keyword(b, i) {
                i = next;
                continue;
            }
        }
        match b[i] {
            b'{' => state.open_brace(b, i),
            b'}' => state.close_brace(i),
            b';' => state.anonymous_header_start = i + 1,
            _ => {}
        }
        i += 1;
    }
    state.out
}

/// Look up the innermost owner-qualified `fn` scope containing byte position `pos` (the smallest
/// enclosing range, in case of nested `fn`s), or a stated fallback if `pos` falls inside no known
/// `fn` body (a probe outside any function — not a realistic `assert_boundary!` call site, but
/// handled rather than panicking).
pub(crate) fn owner_for(scopes: &[(usize, usize, String)], pos: usize) -> String {
    scopes
        .iter()
        .filter(|(start, end, _)| *start <= pos && pos < *end)
        .min_by_key(|(start, end, _)| end - start)
        .map(|(_, _, owner)| owner.clone())
        .unwrap_or_else(|| "<module scope>".to_string())
}

/// Parse a raw string literal `r"…"` / `r#…"…"#…` starting at `i`, returning `(value, next)`.
/// `None` if it is not a well-formed raw string.
pub(crate) fn raw_string_value(b: &[u8], i: usize) -> Option<(String, usize)> {
    let mut j = i + 1; // past `r`
    let mut hashes = 0;
    while b.get(j) == Some(&b'#') {
        hashes += 1;
        j += 1;
    }
    if b.get(j) != Some(&b'"') {
        return None;
    }
    j += 1;
    let start = j;
    while j < b.len() {
        if b[j] == b'"' {
            let mut k = j + 1;
            let mut h = 0;
            while h < hashes && b.get(k) == Some(&b'#') {
                k += 1;
                h += 1;
            }
            if h == hashes {
                return Some((String::from_utf8_lossy(&b[start..j]).into_owned(), k));
            }
        }
        j += 1;
    }
    None
}

/// Decode a plain-string literal's inner bytes (between the quotes, escapes still present) to the
/// exact `&str` value the Rust compiler produces — see `runtime-origin-assertion`'s "CI face —
/// every declared seam is probed" requirement for the full decoded-value-matching and
/// un-auditable-on-failure rationale (including backslash-newline line continuation). Also used
/// for a `#[path]` value (the other caller, below), matching 渾儀's syn-derived `s.value()` on the
/// same input. No real seam name spans lines, so this never meaningfully changes the
/// seam-name caller's behavior. The escape set is the `&str` string-literal set only;
/// byte-string-only escapes never reach here (byte strings are already un-auditable).
pub(crate) fn decode_str_escapes(inner: &[u8]) -> Option<String> {
    // The surrounding source compiled, so it is valid UTF-8; escapes are all ASCII, so iterating
    // by `char` reconstructs any multi-byte content faithfully.
    let s = std::str::from_utf8(inner).ok()?;
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match chars.next()? {
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            't' => out.push('\t'),
            '\\' => out.push('\\'),
            '0' => out.push('\0'),
            '\'' => out.push('\''),
            '"' => out.push('"'),
            // Backslash-newline line continuation (`\n` or `\r\n`): strip it and every
            // subsequent leading whitespace character on the continued line.
            '\r' | '\n' => {
                while matches!(chars.peek(), Some(' ' | '\t' | '\n' | '\r')) {
                    chars.next();
                }
            }
            // `\xHH`: exactly two hex digits, and (for a `&str`) a value in `0x00..=0x7F`.
            'x' => {
                let hi = chars.next()?.to_digit(16)?;
                let lo = chars.next()?.to_digit(16)?;
                let v = hi * 16 + lo;
                if v > 0x7F {
                    return None;
                }
                out.push(char::from_u32(v)?);
            }
            // `\u{ H..H }`: 1..=6 hex digits (`_` permitted as separators), a valid `char`.
            'u' => {
                if chars.next()? != '{' {
                    return None;
                }
                let mut value: u32 = 0;
                let mut digits = 0;
                loop {
                    match chars.next()? {
                        '}' => break,
                        // A leading `_` is "invalid start of unicode escape" in rustc; only
                        // internal/trailing separators are legal, so match rustc exactly here.
                        '_' if digits == 0 => return None,
                        '_' => continue,
                        d => {
                            let hd = d.to_digit(16)?;
                            digits += 1;
                            if digits > 6 {
                                return None;
                            }
                            value = value * 16 + hd;
                        }
                    }
                }
                if digits == 0 {
                    return None;
                }
                out.push(char::from_u32(value)?);
            }
            // An unrecognized escape: react loud.
            _ => return None,
        }
    }
    Some(out)
}
