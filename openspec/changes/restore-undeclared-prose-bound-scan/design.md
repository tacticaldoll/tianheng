## Context

`bound_register_parse.rs` already parses `#### Scenario:` headings into `Bound` (via `bounds_in`/`marks_a_bound`)
and already resolves bare `<capability>/<slug>` references anywhere in text (via `bare_references`). What's
missing is the third leg the spec requires: scanning **every line** of tracked spec prose for
bound-declaring language and failing on any occurrence that sits outside a declared bound scenario, is not
exempted by a bounds-named requirement heading, and carries no resolvable reference.

The shell-era implementation (`scripts/check_bound_register.sh`, deleted by `64ed18c`) is the reference
algorithm — read in full via `git show e26d2ea:scripts/check_bound_register.sh`. It is a single-pass `awk`
state machine over each spec file's lines, tracking: the current `### Requirement:` heading and whether its
own wording names bounds, and the current `####` block and whether it is a declared bound scenario. This
design ports that state machine into Rust rather than inventing a new algorithm, since the shell version's
comments already record several measured, non-obvious tolerances (negation adjacency, one-interposed-word
tolerance, why paragraph-level scanning was rejected) that a fresh design would have to re-discover.

## Goals / Non-Goals

**Goals:**
- Implement the requirement exactly as already specified: fail on bound-declaring prose outside a declared
  scenario, per the four existing scenarios.
- Reuse `marks_a_bound` (scenario-heading detection) and `bare_references` (reference resolution) rather
  than re-deriving either.
- Pass cleanly over the live `openspec/specs/*` corpus with no new false positives, or — if the scan
  surfaces a genuine pre-existing undeclared bound — fix that spec's prose (declare it or reference it) as
  part of this change, since a new gate that immediately needs `TIANHENG_WORKSPACE_TESTS`-style suppression
  is the same "found a gap, didn't close it" pattern the audit exists to end.
- No new dependency: hand-written matching only, since `kanhe`'s dependency law forbids adding a regex crate.

**Non-Goals:**
- Not changing the requirement's wording or its four scenarios — they are already correct.
- Not switching the scan to paragraph-level matching. The shell era already measured this and rejected it
  (residual 3: a `(bound: ...)` reference clears the *paragraph*, not the *sentence*, so paragraph-scanning
  buys nothing against the defect that motivated the residual, at the cost of new false positives). Ported
  as-is, including this rejection.
- Not touching `docs/observation-bounds.md`'s projection *content*-assertion gap (a separate finding,
  Contract #7 in the same audit) — this change only needs the projection to keep matching what the reaction
  now actually does, not to gain a new independent content check.

## Decisions

**Port the shell state machine's four states directly, as a small enum-driven walk over `text.lines()`.**
Alternative considered: reuse `bounds_in`'s existing line-walk and extend it in place. Rejected — `bounds_in`
already does one job (parse declared bounds) and mixing in "flag undeclared prose" would make one function
answer two questions the shell script kept apart with `flush()`/`req_stated` bookkeeping. A second function
walking the same lines is not the twin-parse the register's own doctrine forbids, because it isn't a second
opinion about *bound identity* — that's still owned solely by `bounds_in`. It's a different question
("does this line simply say something outside any declared shape") that only needs `bounds_in`'s heading
detection (`marks_a_bound`) as a shared primitive, not its full parse.

**The prose trigger keeps the shell era's one-interposed-word tolerance independently of `marks_a_bound`,
which no longer has it.** `marks_a_bound` matches only the two bare phrases `a stated bound`/`a documented
bound` (`bound_register_parse.rs:94-98`) — the interposed-word slot it once admitted was deliberately
removed, because a qualifier there doubled as an unclosed classification feeding a bound's derived id
(`observation-bound-register/spec.md`'s "no qualifier" requirement). The prose scan's match feeds no id —
it only decides whether to flag a line for the declaration/exemption/reference check below — so that reason
does not apply here, and the shell era's own `BOUND_PROSE` tolerance is the right precedent to port, not
`marks_a_bound`'s now-stricter one.

**Hand-written trigger/negation matching, tokenized on whitespace with punctuation trimmed at token
boundaries.** The shell regex `(stated|documented)( [A-Za-z-]+)? bounds?` operates on raw bytes; a
line-then-whitespace-split walk is not byte-identical to POSIX ERE but is measured against the live corpus
(the actual acceptance test for this change) rather than assumed equivalent. Alternative considered: a
tiny hand-rolled character-level automaton mirroring the ERE exactly. Rejected as more code for no
behavioral gain here — the pattern has no lookahead/backreference the token walk can't express, and the
corpus is the real arbiter either way.

**Requirement-heading bounds-exemption and scenario-heading tracking share one state machine pass**, matching
the shell script's single-pass design (`req`/`req_is_bounds`/`req_stated`/`open` variables), rather than two
separate passes (one to find exemption windows, one to scan prose). One pass keeps "which lines are inside
the current exemption window" a property the walk *is*, not a separate range computation that could disagree
with the first pass's own idea of where a section starts and ends.

**A negation check is a fixed adjacency test, not a proximity/distance heuristic.** Ported verbatim from the
shell's own measured lesson: only `(rather than|not|never) an?( [A-Za-z-]+)? bounds?` — the negation directly
on the noun — is excluded, because a wider "anywhere nearby" rule was measured to hide three real declarations
in this repository's own specs while catching none of the intended cases.

## Risks / Trade-offs

- **[Risk]** The live corpus may already contain genuine undeclared bound-prose once the scan actually runs
  (the shell era caught real instances before it was deleted; nothing has swept the specs since).
  → **Mitigation**: run the new scan against the current tree as the acceptance step. Any real hit gets
  fixed in this same change (declare the bound as a scenario, or add a `<capability>/<slug>` reference) —
  the tasks below budget for this rather than assuming zero hits.
- **[Risk]** Hand-written tokenization could diverge from the shell regex's exact semantics on an edge case
  the shell era never hit (e.g. punctuation-adjacent triggers, mid-word matches).
  → **Mitigation**: the corpus scan is the acceptance test, not a unit-test-only claim; a divergence that
  matters will surface as either a missed real case (caught by intentionally reintroducing the shell era's
  own historical example sentences as synthetic tests) or a spurious one (caught by the full-corpus run).
- **[Trade-off]** This does not close residual 2 (line-oriented scan misses a statement whose bound name
  continues onto the next line) or residual 3 (a reference clears the whole prose it sits with, not just the
  bound it names) — both are already declared, already-accepted residuals in the spec's own text, ported
  as declared rather than solved, matching the shell era's own judgment call.

## Migration Plan

Repository-internal, `kanhe` is `publish = false` — no external migration. Rollout is: implement, run against
the live corpus, fix any real hits found, land as one PR. No flag or staged rollout needed; a false positive
found after merge is fixed the same way any other repository-check bug is (a following commit), not something
requiring a rollback path.

## Open Questions

- None outstanding; the shell-era implementation and the current spec's four scenarios fully constrain the
  behavior. The only unknown — how many real hits the live corpus produces — resolves by running the scan
  during apply, not by a design decision made in advance.
