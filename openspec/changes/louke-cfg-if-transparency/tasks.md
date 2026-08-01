# tasks: Louke Observes cfg_if Arm Contents Implementation Plan

Implementation notes carried from the spike (measurements and rationale in `design.md`, so none of
this needs re-deriving):

- The two call sites are `collect_scope_modules` (`scan.rs:138`) and `scan_source_with_markers`
  (`scan.rs:666`), both routing through `foreign_macro_body_end`.
- The name test can reuse `preceding_ident_is(b, name_end, b"cfg_if")`, the helper that already
  recognizes `macro_rules` — including its whitespace-before-`!` handling (`cfg_if ! { … }` is valid
  Rust and must match).
- The module pass needs a **positive** descent, not merely a removed skip: its catch-all
  `if bytes[i] == b'{' { i = balanced_brace_end(…) }` would otherwise swallow the arm as an opaque
  block.
- An arm's sub-range is walked with the caller's own `child_base` / `file_dir` (an arm adds no
  directory component), unlike the inline-`mod` branch which accumulates the module name.

- [ ] Add the louke-local transparent-macro name test beside `foreign_macro_body_end`, documenting the name gate as load-bearing and pointing at the sibling dimensions' identical rule. <!-- id: 0 -->
- [ ] Probe pass: do not skip a transparent invocation's body; scan into it so a probe, a typo'd seam, and an un-auditable probe inside an arm are observed exactly as at top level. <!-- id: 1 -->
- [ ] Module pass: walk a transparent invocation's body, descending each top-level brace group (its arms) as a sub-range with the ENCLOSING bases, so an arm-declared `mod` enters the reachable corpus; recursion covers a nested `cfg_if!` for free. <!-- id: 2 -->
- [ ] Thread `in_transparent_arm` through `collect_scope_modules` and OR it into the absent-file tolerance beside `attrs.cfg`, leaving the resolution-ambiguity error unconditional; do not inherit the flag into an inline `mod` body descended from an arm (matching the other two dimensions). <!-- id: 3 -->
- [ ] Regression coverage in `crates/louke/src/audit/tests.rs` for the five measured shapes — probe in an arm, arm-declared `mod`, typo'd seam in an arm, un-auditable probe in an arm, and the top-level control for each — plus if-only, else-if chain, nested `cfg_if!`, paren-delimited invocation, and the unqualified `cfg_if!` spelling. <!-- id: 4 -->
- [ ] Add the controls without which an assertion could pass vacuously: the identical constructs at top level still react, and a clean `cfg_if!` arm stays clean (transparency observes contents, it does not react to the macro). <!-- id: 5 -->
- [ ] Resolve the un-auditable probe's **lexical owner** inside an arm, and decide it on a measurement rather than by assumption. `fn_scopes` deliberately does not skip macro bodies, justified in its own doc by "a probe is never found inside one" — a premise this change invalidates, since a transparent body now yields probes. Measure the rendered owner for an un-auditable probe inside an arm; if the arm's braces contribute anonymous `block …` scopes, decide between accepting them (structural, stable, but adopter-visible in a baseline) and making the transparent invocation's own braces transparent there too (matching the other dimensions' "as if written at the invocation's position"), then pin the chosen owner string in a test and correct that doc's stale justification either way. <!-- id: 6 -->
- [ ] Cover the absence rules: an arm-declared module with no conventional file is tolerated, the same declaration outside an arm still fails loud, and an arm-declared dual-backed module is still a constitution error. <!-- id: 7 -->
- [ ] Extend `crates/tianheng/tests/cfg_if_transparency_conformance.rs` to all three dimensions on the SAME fixtures — add 漏刻's declared seam and its probe to the shared arm — and retire the module doc's "漏刻 is deliberately absent" paragraph. <!-- id: 8 -->
- [ ] MODIFY the two `runtime-origin-assertion` requirements (CI face: the macro-body exclusion gains the transparent carve-out; root-aware audit: arm-declared modules and arm membership as a cfg-conditional source), reproducing every existing paragraph and scenario, and state the two bounds (name-gated, cfg-blind). <!-- id: 9 -->
- [ ] After `openspec archive`, verify the synced main spec against the delta by character count and `#### Scenario` count. Measured on this change: `archive` writes the delta's raw markdown and is **lossless** (a verbatim MODIFIED round-tripped byte-identical apart from a trailing newline), so multi-paragraph descriptions survive; the first-paragraph-only truncation lives in the parsed `text` that `validate` and `show` read, which is why SHALL must sit on the description's first line. The real MODIFIED hazard is authoring — a delta omits a paragraph or scenario and the replacement drops it — hence the verbatim-extract-then-edit method used here. <!-- id: 10 -->
- [ ] Sweep 漏刻's own prose for the retired claim — `audit.rs`'s `audit_probe_coverage` doc, `crates/louke/README.md`, and any test comment asserting macro bodies are uniformly skipped — so no file keeps stating the pre-change rule (the whole-file grep AGENTS.md requires for a vocabulary-level change). <!-- id: 11 -->
- [ ] Add the adopter-facing `CHANGELOG.md` `[Unreleased]` → `### Fixed` entry, naming both closed false negatives, the retired false alarm, and that all three dimensions now share one rule. <!-- id: 12 -->
- [ ] Run the full Definition of Done from the workspace root and report actual output, including the two isolated clippy passes, both release-coherence scripts, and `test_examples.sh`. <!-- id: 13 -->
