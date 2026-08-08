# The three offices are vocabulary, and the crate question is closed

## Why

`PROJECT.md` leaves the shape of 垂象, 實錄 and 校讎 an open question, in the sentence a reader consults
when asking whether one of them should become a crate:

> Both are **crate-or-convention as their nature dictates**, never named before their reaction exists.

That answers nothing, and it has been asked three times. Three changes were written to close it and all three
were withdrawn (#435, #436, #437) — the diagnosis was that each restated a law that was only **half** reacted
to, so the paragraph had nothing to stand on.

**That blocker is gone.** The law the argument rests on is `三儀 ⊥ 三儀` — a dimension must never learn from a
sibling — and until `mutual-independence-reacts-to-membership` merged, that law was *quoted* by each
dimension's `because` and asserted by nothing. Widening `guibiao`'s allowlist to name `hunyi` left the whole
workspace green. The reaction now inspects allowlist membership, so "a crate is the boundary the self-law
reacts to" is a fact rather than a claim.

## The product decision, which the measurement already made

None of the three names appears in any shipped public item, crate name, manifest, `description`, or
adopter-facing document. Every occurrence outside `docs/history/`:

```
PROJECT.md                                the paragraph itself
AGENTS.self-law.md                        the generated agent preamble
crates/tianheng/tests/self_governance.rs  the preamble constant, and two doc comments
crates/tianheng/src/runner/render.rs      one doc comment
```

**They never reached the product.** So the question "should one become a crate?" was never a product question
open for decision — it was a sentence keeping a settled matter unsettled. This change closes it and says why,
so it is not re-minted.

## Where each surface actually is

Measured, because the withdrawn attempts died on this: #436's location table was wrong three times out of
three, and #435's retirement of 校讎 rested on a grep that never swept `.github/`.

| surface | referent |
|---|---|
| 垂象 | `crates/guibiao/src/projection.rs` assembles the report and constitution documents; `crates/tianheng/src/runner/render.rs` renders text and SARIF; `crates/xuanji` serializes a `Violation` |
| 實錄 | `crates/xuanji/src/baseline.rs` holds the model, and every crate above it consumes the baseline its own verdicts fold into |
| 校讎 | `.github/CODEOWNERS` — whose first line reads *"The amendment reaction"* — routes a change to the law to the steward; `AGENTS.md` owns the OpenSpec lifecycle; `crates/tianheng/src/constitution.rs` names the routing in shipped source; and `self-law-projection`'s spec makes `CODEOWNERS` normative for Layer 3 |

## Why none is a crate, stated as boundaries rather than importance

> 三儀 are orthogonal — a dimension must never learn from a sibling — so each needs a boundary the self-law can
> react to, and every dimension's `restrict_dependencies_to` naming no sibling **is** that reaction. A
> governance surface has no boundary to be: each crosses every crate it touches, and one lives outside
> `crates/` altogether. A crate there would enclose nothing, so the name would mark nothing — which is the
> drift law's own prohibition rather than a stylistic call.

This does **not** say "only a 儀 is a crate" — an argument #437 was withdrawn for, since `xuanji` and
`tianheng` are crates and neither is a 儀. It says a surface with no boundary earns no crate.

## What this does not do

- It does not retire the three names. Each has a referent, `校讎`'s in four places, and a name with a referent
  is vocabulary rather than drift.
- It does not make them modules of the `tianheng` facade either. `實錄` is `xuanji::baseline` — **below**
  `tianheng` — so folding it upward would invert the dependency the boundary law exists to hold.
- It does not touch `docs/history/0.1.0-0.3.0-built-ledger.md`, which records what was true at 0.1.0–0.3.0.
