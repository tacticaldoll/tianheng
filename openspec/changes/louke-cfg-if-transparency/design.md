## Context

漏刻's audit is a byte scanner with no parser, and `foreign_macro_body_end` skips a macro
invocation's balanced body in one jump. It is called from two independent passes:

- `collect_scope_modules` (`scan.rs:138`) — the reachable-module walk.
- `scan_source_with_markers` (`scan.rs:666`) — the probe scan.

Both therefore treat a `cfg_if!` body as macro-generated dead code.

## Feasibility spike — measured

A throwaway test drove the real `audit_probe_coverage` over five fixtures (see `proposal.md` for the
table). All five behaved as feared: the probe pass loses probes, typo'd seams, and un-auditable
probes written inside an arm; the module pass loses the whole subtree beneath an arm-declared `mod`.

**The spike's load-bearing finding overturns this change's own earlier cost estimate.** When 渾儀's
change was written, its `design.md` recorded that 漏刻 "would need 圭表's brace-kind model in both"
passes. Reading the two passes disproves that:

| pass | its item-context model | what transparency costs |
| --- | --- | --- |
| probe scan | **none** — it counts a marker anywhere in the file, and skipped macro bodies wholesale | do not skip; scan into the body. Three lines. |
| module walk | **range recursion** — descends an inline `mod x { … }` body as a sub-range, and skips every other `{ … }` block via `balanced_brace_end` | recurse into each arm as a sub-range with the *enclosing* bases |

圭表 needed `MacroScope` / `brace_stack` / brace kinds / `transparent_count` because it reconstructs
item nesting by hand from a flat byte walk. 漏刻's module pass already *is* that model, expressed as
recursion over ranges — so an arm is simply one more range to descend, and a nested `cfg_if!` inside
an arm falls out of the same recursion for free. The one thing the recursion does not carry by itself
is arm membership, which the absent-file tolerance needs; that is one boolean parameter.

The line that makes the module pass wrong today is not the macro skip alone. Even with the skip
removed, the walk would reach the arm's `{` and hit its catch-all `if bytes[i] == b'{' { i =
balanced_brace_end(…) }` — skipping the arm as an opaque block. Transparency therefore has to be
positive (descend the arm), not merely the absence of a skip.

## Goals / Non-Goals

**Goals:**
- Both passes observe arm contents; an arm-declared `mod` enters the reachable corpus.
- Arm membership is a cfg-conditional source for absent-file tolerance, matching the other two
  dimensions rather than a third rule.
- 漏刻 joins `cfg_if_transparency_conformance.rs`, retiring its stated absence.

**Non-Goals:**
- Evaluating `cfg`. The scan stays lexical and cfg-blind, as its contract already states.
- Transparency for any other macro name. Same bound as 圭表 and 渾儀 — and here the byte scanner has
  no way at all to tell a body-wrapping macro's arms from an arbitrary macro's nested blocks.
- Expression-position `cfg_if!`. 渾儀 gets this free (a fn-body invocation is a statement its item
  walk never reaches); 漏刻's probe pass, having no nesting model, reads *into* every `cfg_if!` body
  regardless of position — which is the FN-safe direction and is what its cfg-blind, lexical contract
  already promises.

## Decisions

### Decision 1: One name test, two call sites, louke-local

A byte-level "the identifier before this `!` is exactly `cfg_if`" test, reusing the existing
`preceding_ident_is` helper that already recognizes `macro_rules`. No new scanning machinery, and
nothing imported from 圭表 (三儀 ⊥ 三儀).

### Decision 2: The module pass descends arms with the ENCLOSING bases

An arm is not a module: rustc adds no directory component for it. So the arm's sub-range is walked
with the caller's own `child_base` and `file_dir`, unlike the inline-`mod` branch which accumulates
the module name. Getting this wrong would resolve an arm-declared `mod net;` under
`<dir>/cfg_if/net.rs` and silently drop every probe beneath it — the coverage false negative this
change exists to close, reintroduced one layer down.

### Decision 3: Arm membership is one boolean, OR-ed with the attribute gate

`collect_scope_modules` gains an `in_transparent_arm: bool`, true only for an arm recursion, and the
absent-file tolerance reads `attrs.cfg || in_transparent_arm`. It is *not* inherited into an inline
`mod` body descended from within an arm — matching how a bare `#[cfg]` on an outer `mod` already
fails to tolerate an absent file for an inner one, in all three dimensions.

### Decision 4: The spec change is MODIFIED, and verified after sync

Two requirements must be rewritten rather than supplemented, because the CI-face requirement's
macro-body sentence currently mandates the broken behavior. Both have multi-paragraph descriptions
(4 and 3 paragraphs) and 17 and 11 scenarios.

Measured before authoring them, correcting a belief this repo had been carrying: `openspec archive`
writes the delta's **raw markdown** and is lossless — a verbatim `## MODIFIED Requirements` copy of
the 4-paragraph, 17-scenario CI-face requirement round-tripped byte-identical apart from one trailing
newline, with the CLI reporting `~ 1 modified` so it genuinely rewrote it. The first-paragraph-only
truncation people (including this project's own notes) attribute to archive is in the parsed `text`
field that `validate` and `show` consume — which is why `SHALL` must sit on a description's first
line, and nothing more. The real MODIFIED hazard is therefore **authoring**: the delta replaces the
whole requirement, so any paragraph or scenario missing from the delta is deleted. Both deltas here
are built by extracting the requirement verbatim from the main spec and editing it, never retyped,
and the sync is still verified by count.

### Decision 5: The un-auditable probe's owner keeps its anonymous arm scopes (measured, then decided)

`fn_scopes` resolves an un-auditable probe's lexical owner and deliberately does not skip macro
bodies, justified in its own doc by "a probe is never found inside one" — a premise transparency
retires. Measured after the probe pass landed: a probe inside an arm renders as
`block cfg_if::cfg_if!#1::block if #[cfg(unix)]#1::fn f`, against `fn f` at top level.

Kept as-is rather than made transparent there too. It is this function's **existing** rule for any
anonymous scope — a real `if` block's braces read the same way — so accepting it keeps 漏刻 internally
consistent instead of special-casing arms; the rendering names the arm, which an adopter reading the
violation wants; and the alternative needs the name test plus arm-brace tracking in a third place for
a cosmetic gain. The owner string is pinned by a test and the stale justification is rewritten to
state both cases (transparent bodies now yield probes and are read as ordinary code; other macro
bodies stay inert because the probe scan still skips them).

The spec is left silent on the rendering: this is an existing rule applied unchanged, not a new one,
and the requirements that own owner identity already cover "enclosing items and anonymous scopes".
Writing the exact string into a requirement would over-specify an incidental rendering.

## Risks / Trade-offs

- **[Trade-off] Newly caught findings for `cfg_if!` adopters.** A typo'd or un-auditable probe inside
  an arm now reacts. That is the point; both are absorbable by baseline. The opposite direction — a
  false "unprobed" alarm — becomes *quieter*, so the net adopter effect is fewer failures.
- **[Risk] The probe pass reads into a `cfg_if!` in expression position too.** Its contract is
  lexical and cfg-blind, so counting such a probe is consistent with counting one behind
  `#[cfg(test)]` — already a stated bound. FN-safe: it can only make coverage *more* visible, never
  less.
- **[Bound, stated not silent] Only `cfg_if` is transparent, and the scan stays cfg-blind.** Identical
  to the bounds the other two dimensions declare, so the three now state one rule rather than three.
- **[Risk] A malformed `cfg_if!` with no `if` header** has its brace groups descended as arms anyway.
  That source does not compile, so the over-observation cannot reach a buildable crate — the same
  stated bound 圭表 and 渾儀 carry.
