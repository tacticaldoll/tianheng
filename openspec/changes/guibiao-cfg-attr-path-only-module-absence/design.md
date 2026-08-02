## Context

`crates/guibiao/src/module_scan/reachability/walk.rs`'s `collect_children` classifies each
declared `mod name;` into `ChildSources.plain` (a plain, conventionally-file-backed declaration)
unless it carries a direct, unconditional `#[path = "…"]` (which routes to `.direct` instead).
`resolve_plain_sources` then requires exactly one of `name.rs` / `name/mod.rs` to exist for every
`PlainSource`, unless that source's `is_cfg_conditional` flag is set — today true only from a bare
`#[cfg(...)]` attribute or `cfg_if!` arm membership (`declarations.rs`'s `DeclaredModule`).

A declaration's `cfg_attr(path)` candidates are collected separately, into
`declared.conditional_path_eqs`, and only reach `ChildSources.conditional` — never influencing
`is_cfg_conditional` or the plain-file requirement. So a declaration backed ONLY by one or more
`cfg_attr(path)` remaps (no plain file, no direct `#[path]`) is *always* required to also have a
plain conventional file, even when a remap candidate resolves to a real file that would make the
declaration compile cleanly on every real configuration. `crates/hunyi/src/scan.rs`'s crate-wide
walk already avoids this: it computes `has_backing_source` — true iff at least one
`cfg_attr_targets` candidate `is_file()` — and only raises the missing-file error when
`!has_backing_source && !cfg_conditional`.

## Goals / Non-Goals

**Goals:**
- Tolerate an absent plain conventional file for a declaration when at least one of its
  `cfg_attr(path)` candidates resolves to a real on-disk file — matching 渾儀/漏刻's existing rule
  for the identical shape (三儀 ⊥ 三儀).
- Preserve every existing outcome the current tests and spec already pin: the dual-conventional-
  forms ambiguity error, the fully-absent-everything error, and the direct-`#[path]`
  precedence/bypass.
- Keep the fix inside `guibiao`'s existing per-source, cfg-blind, additive aggregation model
  (`ScanSource`, `ChildSources`, per-source `ancestors`) — no new data model, no cross-arm merging.

**Non-Goals:**
- Proving that a set of stacked `cfg_attr` predicates is jointly exhaustive (a SAT/tautology
  problem over opaque predicate expressions `guibiao` is deliberately cfg-blind about). The fix
  does not attempt this — it only asks "does at least one candidate physically exist on disk?",
  the same blind, existence-only test the plain-file check and the bare-`#[cfg]` tolerance already
  use. This mirrors, rather than extends, the family's existing risk posture.
- Changing what counts as cfg-conditional at the lexical level
  (`declarations.rs::DeclaredModule::is_cfg_conditional`, which stays scoped to bare `#[cfg]` /
  `cfg_if!` arm membership). The new tolerance is filesystem-dependent (does a candidate resolve?)
  and therefore belongs in `walk.rs`, where `path_base` and disk access are already available —
  not in the purely lexical `declarations.rs` scanner.

## Decisions

- **Resolve `conditional_path_eqs` before deciding the plain branch, within the same
  `collect_children` iteration over one `declared: DeclaredModule`.** Both the existing
  conditional-resolution loop and the plain/direct branch already run per-`declared`, per-source,
  inside the same `for declared in declared_modules_in(...)` iteration — reordering so the
  conditional resolution runs first, and folding its result into the `PlainSource` pushed for that
  *same* declaration, requires no new struct and no cross-iteration state. Considered: adding a
  fourth `ChildSources` bucket or a post-hoc reconciliation pass after the whole `children` map is
  built — rejected as more code for no behavioral gain, since the information needed
  (`declared.conditional_path_eqs`, `loaded.path_base`) is already in scope at the point the
  `PlainSource` is constructed.
- **Widen `PlainSource.is_cfg_conditional` to `declared.is_cfg_conditional || has_backing_conditional_target`
  rather than adding a separate field.** `resolve_plain_sources` already treats
  `is_cfg_conditional` as exactly the "this plain source's absence may be legitimate" signal
  `resolve_plain_sources` needs; a resolved `cfg_attr(path)` candidate is a third source of that
  same signal, not a materially different one from the caller's perspective. Considered: a
  dedicated `plain_absence_tolerated: bool` — rejected as a distinction without a difference at
  the one call site that reads it (`resolve_plain_sources`'s `if is_cfg_conditional { continue; }`);
  it would only add a second field to keep synchronized with the first.
- **Compute `has_backing_conditional_target` strictly per-declaration, never merged across other
  `PlainSource`/`ConditionalPathSource` entries for the same child name.** Two mutually-exclusive
  `#[cfg]` arms declaring the same module name (the standard per-platform shim) must keep
  independent ancestor sets and independent absence signals — merging them was the exact defect
  the existing per-source `ancestors: HashSet<PathBuf>` design already guards against elsewhere in
  this file (see its module-level doc comment). The new check reuses the same `declared` value's
  own `conditional_path_eqs` and the same `loaded.path_base`/`loaded.ancestors` already scoped to
  this one source — it introduces no new merging.
- **Do not touch the dual-conventional-forms ambiguity branch.** `resolve_plain_sources` checks
  `flat.is_file() && nested.is_file()` and errors unconditionally, before ever consulting
  `is_cfg_conditional`. The fix only touches how `is_cfg_conditional` itself is computed, so this
  branch's behavior is unchanged by construction, not merely by intent — pinned by a dedicated
  regression scenario rather than left to review alone.

## Risks / Trade-offs

- **[Risk] A `cfg_attr(path)` target that resolves to a file for the WRONG reason (e.g. a stray
  file coincidentally sitting at the remap's relative path, unrelated to any real build) could
  now silently excuse a genuinely missing plain file.** → Mitigation: this is the same blind,
  existence-only trust the union-scan (`656dc111`) and the crate-wide walkers (`hunyi`/`louke`)
  already extend to every `cfg_attr(path)` candidate; a resolved candidate is *governed* (scanned
  for imports) exactly like today, so a stray file would surface as a governed module with
  unexpected imports rather than a silent pass — no new blind spot, only a narrower one than the
  status quo's "always error."
- **[Trade-off] The fix cannot distinguish "this platform's real build never needs the plain
  file" from "the author forgot to remove a stale conventional file that also happens to exist."**
  → Accepted: this is the same trade-off the existing "both conventional forms present is always
  an ambiguity" and "bare `#[cfg]` tolerates absence without evaluating the predicate" rules
  already make; a cfg-blind scanner cannot do better without adopting a real predicate evaluator
  (an explicit, larger amendment PROJECT.md already declines elsewhere for the identical reason).
