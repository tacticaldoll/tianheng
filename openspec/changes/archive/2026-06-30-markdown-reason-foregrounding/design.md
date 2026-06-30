## Context

The self-law projection (capability `self-law-projection`) put the enforced law into agent context, and reserved **Contract B**: the Markdown layout is a human/agent surface that may evolve in a compatible release; JSON is the machine contract; no test pins the Markdown bytes. This change is the **first layout evolution under that freedom** (Contract B itself landed in the preceding change): foreground the `reason`, which the 潛移 thesis identifies as the gravity-bearing content (the generative principle an agent imitates), instead of rendering it last as a peer metadata bullet.

## Goals / Non-Goals

**Goals:**
- Make the `reason` lead each boundary block in the Markdown projection — read as a principle, not a trailing metadata field.
- Improve the imitable surface without touching law, reaction, or the JSON machine contract.
- Lock the *foregrounding invariant* as a tested requirement, while keeping the exact layout free (Contract B intact).

**Non-Goals:**
- No JSON / text-projection change; no reaction/law change; no SemVer-identity change.
- No golden/byte snapshot of the Markdown (that is the freeze Contract B forbids).
- No generator, no `list-self` CLI, no new boundary/dimension.

## Decisions

### Shape B2 — reason as a leading blockquote, ordered reason → rule → kind/severity

Three shapes were weighed:
- **B1 (reorder bullets, reason first)** — rejected: the reason stays a peer `- **reason**:` bullet; the imitation gain is weak (it still reads as metadata).
- **B2 (reason as a leading blockquote)** — **chosen**: the reason is lifted out of the metadata list into a blockquote between the target heading and the mechanical bullets. It reads as the block's principle ("hear the maxim first, then the mechanical constraints"); the target stays the scannable heading.
- **B3 (rule promoted into the heading, reason as subtitle)** — rejected: stuffing the rule into the `### ` heading reads fine for a short rule but turns into a dragging, hard-to-scan heading for an allowlist / allowed-origins / semantic forbidden-set.

Order is **reason → rule → kind/severity**: the reason is the principle, the rule is the reaction's mechanical shape, kind/severity is classification. The existing `if !reason.is_empty()` guard stays, so a reason-less boundary emits no blockquote.

### Lock the invariant, not the format (Contract B stays the global guard)

The new test asserts **only** that the reason precedes the rule and the classification — an order predicate over byte-indices, which by construction does not pin spacing, wording, the blockquote choice, or added fields. So the foregrounding test is genuinely *not* a format freeze; that is the precise line between *requiring a principle* (legitimate, tested) and *freezing a format* (forbidden).

Honesty about enforcement strength: this change does **not** make its spec the global guard against byte-snapshotting the Markdown. That prohibition lives in `self-law-projection`'s Contract B (unchanged, review-verified) — "No automated test SHALL pin the helper's exact Markdown layout as a contract." The foregrounding requirement is a deliberate, narrow *exception* under Contract B (one ordering invariant is committed), and it names Contract B normatively rather than silently restating or weakening it. Nothing here promises to *prevent* a future golden test; Contract B remains that (review-verified) guard.

### Spec delta is MODIFIED "List honors the format flag", not a new ADDED requirement

The requirement that already owns the Markdown projection is **"List honors the format flag"** (it enumerates the Markdown field set — target/rule/reason — order-agnostically). Foregrounding is an *ordering attribute of that same Markdown contract*, not a separate capability concern, so adding it as a second ADDED requirement would leave two requirements governing one renderer's output (one order-silent, one order-mandating) — a duplicated-concern split. We therefore **MODIFY** "List honors the format flag", folding in the foregrounding sentence and an ordering scenario, so one requirement owns the Markdown contract. (The separate "List command projects the declared constitution" requirement governs the default text projection and is untouched.)

### AGENTS.self-law.md regenerates, it does not drift

The self-law projection uses the same renderer, so changing `boundary_markdown` changes its output; the staleness test (`self_law_projection_is_fresh`) fails until `BLESS=1` regenerates the artifact. That failure-then-regenerate is the **expected, correct** ripple — exactly the Contract A discipline working: the artifact follows the law by regeneration, never by hand.

## Risks / Trade-offs

- **[A future layout change could be read as breaking by an adopter who scraped the markdown]** → Contract B's doc-comment (reaffirmed here) states the layout may evolve and directs machine consumers to JSON; no byte test pins it. The risk is documented away, not frozen in.
- **[The foregrounding test accidentally becomes a de-facto format freeze]** → Mitigated by construction: the test asserts byte-index ordering (reason < rule < classification), never equality to a literal. Reviewed as such.
- **[D (adopter recipe in README) leaks repo-internal story into the product entry]** → Keep it to one line, framed as the library primitive (`constitution_markdown`), not a workflow; no generator/CLI promise.

## Open Questions

- None blocking. D's exact home (README vs crate-level docs) is a placement nit settled at apply time; the recipe content is fixed.
