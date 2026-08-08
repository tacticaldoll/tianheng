## ADDED Requirements

### Requirement: A dimension's declared allowlist SHALL obey the law its reason quotes

`tianheng_constitution()` gives each dimension a `restrict_dependencies_to` allowlist whose `because` quotes
三儀 ⊥ 三儀 — a dimension must never learn from a sibling. The reaction over those declarations SHALL assert
**both** statements separately: that the clause is present in the reason, and that the allowlist **names no
sibling dimension**. They are different statements, and only the first was asserted.

That gap was a false negative, reproduced rather than reasoned about: widening `guibiao`'s allowlist to name
`hunyi` left **every** test binary in this workspace green, and `AGENTS.self-law.md` regenerated to print
`only: serde_json, xuanji, xingbiao, hunyi` **directly beneath** the reason that forbids it.

Neither of the two reactions a reader would expect to catch it can. The staleness check pins the projection
against the declaration, so a blessed projection of a widened allowlist is *fresh* — freshness is not truth. And
the dependency reaction cannot fire on a **widened** allowlist, because permitting more than the tree uses
produces no violation. So this assertion is the sole guard, which is why it is a requirement rather than an
implementation detail.

The clause check SHALL remain. Its limits are **stated and not yet declared as bounds**, and the requirement says
which is which rather than leaving a reader to assume: paraphrasing the clause makes the reaction **fire**, an
over-reaction refusing a reason that genuinely states the law; a `because` carrying the literal clause while
*negating* it passes, and the projection then teaches the negation, which is the under-reaction. Both were
measured by writing them into the tree. Declaring either SHALL wait for a pin that runs **the reaction over a
declaration**, not its predicate over a string — a draft of this change declared the first as a false negative
and pinned it with the predicate, and one run of the bound's own WHEN falsified both the extent and the pin.
`BACKLOG.md` carries them meanwhile, together with the two enumeration limits of the membership half: a
hand-kept dimension list nothing holds, and a rule variant the filter does not reach.

Membership is the structural half; wording is the half that is not.

#### Scenario: A dimension's allowlist names a sibling

- **WHEN** a dimension's `restrict_dependencies_to` allowlist contains another dimension's package name
- **THEN** the reaction fails, naming the target, the sibling, and that the boundary now permits exactly what
  its own reason forbids

#### Scenario: The projection of a widened allowlist is fresh

- **WHEN** an allowlist is widened and the projection is regenerated
- **THEN** the staleness check is clean, because it compares the projection with the declaration — so freshness
  cannot stand in for this assertion
