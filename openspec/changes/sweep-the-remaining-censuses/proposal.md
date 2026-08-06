# Change: sweep the remaining censuses, and name the export an adopter reads

## Why

`AGENTS.md`'s new rule — *a census is produced, never typed* — has its first live test, and the sweep found more
sites than the review reported.

**The stale figure, in four places.** *"the `Observation bounds` requirement **three** specs carry"* and its
variants. Measured now: **5** specs carry that exact heading and **8** carry a heading beginning with it.

| Site | What it says |
| --- | --- |
| `openspec/specs/observation-bound-register/spec.md:51` | "three specs carry" |
| `scripts/check_bound_register.sh:18` | the same sentence, copied into the gate |
| `scripts/check_bound_register.sh:52` | "because **three** such requirements state their bounds as numbered lists" |
| `scripts/check_bound_register.sh:901` → `docs/observation-bounds.md:30` | the same figure as a **template literal inside a generated projection** |

The last two were not in the review; the third and the register's own header were found by grepping the phrasings
after fixing the two that were. The template literal is the shape `AGENTS.md` names as the one place a projection
cannot self-correct — the freshness check compares the generator's own text with itself — and it also **contradicts
its own specification**, which was corrected to "several" at `spec.md:300`.

**A second census in the same sentence the review flagged.** `spec.md:296` reads *"3 of 30 specs carry an
Observation-bounds requirement today while **11** more state bound prose without one"*. The review caught the
`3 of 30`; the `11` is the same defect in the same sentence. And the history is the sharpest part: `8df7ed9` swept
that denominator `29 → 30` and left the numerator at `3` — a fourth sweep, which is exactly what the rule was
written to forbid.

**A word-form census in the changelog.** *"every one of the family's own **fifty-three** declarations is a literal"*
— counted now: **56** (11 + 25 + 6 + 14), matching what the register projects. It is invisible to
`check_bound_register.sh`'s census direction because it is spelled in words, the blind spot `AGENTS.md` records —
reproduced four lines from the entry announcing the model.

**And an export the Added section never named.** `observation_bounds()` is new public API on four published crates
and is what `observation-bound-model`'s *A dimension SHALL export its declarations as library items* obliges. Its
only changelog occurrence was inside a narrative about a test.

## What Changes

- Every stale figure loses its number rather than gaining a fresher one — the rule's own guidance, and the only
  option that cannot go stale again. The generator's template literal becomes `several`, matching the specification
  it had drifted from.
- `spec.md:296` says the size is measured **by the reaction, which prints it**, instead of carrying two figures.
- The `Added` entry names `observation_bounds()` and what an adopter can do with it.

## Impact

- Affected specs: `observation-bound-register`
- Affected code: `scripts/check_bound_register.sh`, `docs/observation-bounds.md` (regenerated)
- Affected docs: `CHANGELOG.md`
- No public API change, no version bump. Nothing about any verdict moves: every edit is prose or a comment, plus one
  string in a projection's template.
