## Context

潛移's foregrounding so far touches the **law projection** (`list --format markdown`). But an agent blocked by a reaction reads the **reaction's output** — `check`'s human text report (`report_violations` in `runner.rs`). There the reason is the 4th field (Boundary → Rule → Found → Reason → Reaction), and the offending `file` — added to the JSON by the offending-file change — is never printed, so the human report silently drops the "where to repair." This change extends the same foregrounding principle to the reaction output.

## Goals / Non-Goals

**Goals:**
- Lead each violation's text block with the reason (the principle and repair direction).
- Surface the offending `file` in the text report (project already-observed data the report was dropping).
- Group violations by boundary so the agent reads the maxim once per boundary.

**Non-Goals:**
- No JSON change (the machine contract stays byte-stable).
- No `repair_hint` or any derived/rewritten reason field (would be a second source of truth — the rejected open-loop prose).
- No good/bad reason library, no repair cookbook, no making the report a machine contract.

## Decisions

### Foreground in the reaction output, mirroring the law projection

The order becomes **Reason → Boundary → Rule → Found → (File) → Reaction**. Reason leads because it is both the imitation principle and the repair direction; the mechanical fields follow; `File` (when present) sits with the mechanical "where", just before the Reaction verdict. This is the text-report analogue of the markdown foregrounding, and like it pins an *ordering/presence invariant*, not the exact layout (wording/spacing free).

### Surface `file`, omit it faithfully when absent

The offending-file change made `file` a faithful byproduct of observation (present for module-import and un-auditable-probe violations, `null` otherwise). The text report simply projects it when present and omits the element when absent — never a fabricated location. This finishes surfacing a feature that until now only reached the JSON.

### Group by `(target, rule)` in the presentation layer only

Sorting happens **inside `report_violations`** (a stable sort by `(target, rule)`), not by mutating `Report`. So the JSON projection (`report_json`) keeps its existing order and content untouched — the machine contract is byte-stable; only the human report clusters. Baselined-violation accounting is unchanged.

### No JSON change — the bright line

The JSON is the machine contract; this change must not touch it. A scenario explicitly asserts the JSON is byte-stable. The improvements are presentation of the text report over already-observed fields — consistent with the rule that the imitable/agent-facing surfaces may evolve while the machine projection stays fixed.

## Risks / Trade-offs

- **[Reordering the text report could surprise a script scraping stderr]** → stderr text is a human surface, never a documented machine contract (JSON is); the reorder is non-breaking by that line. A scenario pins JSON stability so the contract consumers are unaffected.
- **[Grouping in the renderer diverges from JSON order]** → intended: JSON keeps detection order (stable machine contract), text clusters for human/agent reading. The scenario makes this explicit.
- **[Scope creep into repair_hint]** → explicitly a non-goal; the reason is shown as declared, never rewritten or derived.

## Open Questions

- None blocking. The `constitution_markdown` doctest (E) rides along as a small task — it locks the previous change's adopter recipe as a CI-run example; it is independent of the report changes.
