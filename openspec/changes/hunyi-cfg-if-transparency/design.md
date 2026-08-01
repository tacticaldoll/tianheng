## Context

`syn` parses `cfg_if::cfg_if! { … }` in item position as `syn::Item::Macro`, whose `mac.tokens` is an
opaque `TokenStream`. 渾儀 handles no `Item::Macro` anywhere (`collect.rs` matches only concrete item
variants; `scan.rs` and `module_resolve.rs` match `Item::Mod`), so arm contents are invisible.

`cfg_if!`'s grammar is `if #[cfg(a)] { items } else if #[cfg(b)] { items } else { items }`.

## Feasibility spike — measured, ten shapes

A throwaway probe walked `mac.tokens`, filtered top-level brace groups, and `syn::parse2::<syn::File>`
each. Results, all as hoped:

| shape | outcome |
| --- | --- |
| if/else, exposure in each arm | both arms' items recovered |
| if-only arm (no `else`) | recovered |
| `else if` chain, three arms | all three recovered |
| nested `cfg_if!` inside an arm | recursion recovers inner arms |
| `mod x;` and inline `mod x { … }` inside arms | both recovered |
| paren-delimited invocation `cfg_if!( … )` | recovered — the outer delimiter is irrelevant |
| `cfg_if!` inside an inline `mod` | recovered once the walk descends `Item::Mod` content |
| generative macro whose body does not parse | nothing recovered, no panic |

Two results carry the design:

**No `MacroScope` machine is needed.** `cfg_if!` written inside a function body is **not**
`Item::Macro` — `syn` places it as a statement, so 渾儀's item walk never reaches it. The five 0.2.3
follow-up fixes 圭表 needed (`MacroScope`, `brace_stack`, `MacroBody`/`Arm`/`Regular` brace kinds,
`transparent_count`, `inherited_top_level`) exist because a byte scanner must reconstruct item context
by hand. A parser supplies it. Combined with `syn::File`'s own item structure, the arm-versus-item-body
problem cannot arise here.

**Name gating is load-bearing.** Applied to an arbitrary macro, arm extraction is wrong:

```
generate_wrapper! { impl Foo { pub fn hidden() -> crate::forbidden::Thing { … } } }
                         └──── the impl body's braces are a top-level brace group ────┘
                                              ↓
                                    recovered as "arm0: fn hidden"
```

That invents an item the macro may never emit verbatim — a false positive. So restricting the
mechanism to the `cfg_if` name is what keeps it sound, not a hedge.

## Goals / Non-Goals

**Goals:**
- Arm items enter item collection; arm `mod` declarations enter the module walkers.
- Adopt 圭表's arm-membership cfg-conditional rule for absence tolerance rather than inventing one.
- Declare the two residual bounds in the spec.

**Non-Goals:**
- Structural (name-independent) transparency. Withdrawn: it rests on an unevidenced population of other
  transparent macros (`cfg_match!` is nightly-only; no adopter usage shown for hand-rolled equivalents)
  and carries the measured false positive above. The class is effectively a singleton, so a general
  mechanism buys nothing for a real cost.
- Evaluating `cfg`. Arms are unioned cfg-blind, as everywhere else in 渾儀.
- An invocation inside an `impl` or `trait` body. Measured during apply: `syn` gives an
  `ImplItem::Macro` there, whose arms parse as impl items, not items — a parallel flattening for two
  more item kinds threaded through ~10 `impl`/`trait` body walkers in `collect.rs` plus the `unsafe`
  visitor. A real residual false negative, kept **stated** rather than silently absorbed: pinned by
  `a_cfg_if_inside_an_impl_body_is_a_stated_bound`, declared in the requirement, and left to its own
  change with its own review. Widening this change's blast radius to reach it would have shipped the
  ~10 call-site edits without the review they deserve.
- 漏刻's equivalent. Its `foreign_macro_body_end` is called in **two** passes —
  `collect_scope_modules` (module declarations) and `scan_source_with_markers` (probes) — and having no
  parser it would need 圭表's brace-kind model in both. That is a different cost class and gets its own
  change and its own spike.

## Decisions

### Decision 1: One flattening helper, applied where items enter

A single function turns `&[syn::Item]` into an item list with transparent-macro arms spliced in,
recursing for nested invocations. Parsing yields owned items, so the result is owned. Applying it at
the walk entry points keeps every downstream matcher unchanged — no capability learns about macros.

### Decision 2: Arm membership is cfg-conditional, per 圭表's settled rule

An arm-declared module whose conventional file is absent is tolerated, not a missing-file constitution
error, because the arm's predicate gates it. 圭表 reached this in `a567211`; adopting the same rule
keeps the two dimensions from disagreeing on the shape the way their missing-file policies once did
(the 0.2.2 lesson). The ambiguity reaction is unaffected: two conventional forms present stays a
constitution error under every gate, arm membership included.

### Decision 3: Name-gated, and the gate is stated in the spec

`cfg_if` only, matching 圭表's `is_transparent_macro_name`. The bound is written into the requirement
rather than left implicit, so an adopter reading the spec learns that another body-wrapping macro is
not covered — today that is a silent hole in all three dimensions.

## Risks / Trade-offs

- **[Trade-off] New violations for `cfg_if!` adopters.** A false-negative closure, the class
  0.2.2/0.2.3 shipped as patches, and absorbable by baseline. 圭表 already surfaced its half in 0.2.x,
  so the static findings are not new to such an adopter; this adds the semantic ones.
- **[Risk] Flattening allocates.** Owned items are cloned per walk. Acceptable for a build-time scanner
  and unmeasured; if it ever matters, the fix is laziness, not a different observation model.
- **[Risk] A malformed `cfg_if! { … }` with no `if` header** has no real gate yet its contents would be
  treated as arm contents. That source does not compile, so the over-observation can only reach a crate
  that already cannot build — a stated bound, matching 圭表's own.
- **[Bound, stated not silent] Only `cfg_if` is transparent, and arms are unioned cfg-blind.** Both are
  irreducible: name-independence is measurably unsound (above), and knowing which arm compiles requires
  evaluating the whole feature/target resolution — cargo's job, not a scanner's.
