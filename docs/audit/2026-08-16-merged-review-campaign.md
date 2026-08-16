# Merged-review campaign — `v0.4.0..release/0.5.0`

A defect queue, not a report: every unchecked box here is work the project still owes itself.
Check the box in the same commit that closes it. This file is retained only while it has unchecked
boxes — `BACKLOG.md`'s own rule is that a sweep's working queue is not kept once fully drained,
because its substance then lives in the closing commits and in `CHANGELOG.md`.

## Provenance

Two independent full-range reviews over `v0.4.0..HEAD` (298 commits, 246 files), merged. One was run
in-session against the tree; the other was contributed as a 27-agent parallel pass with its own
gate ladder and against-contract track. **Their intersection was empty** — 7 findings from one, 4 from
the other, and neither found anything the other found.

Two of the contributed review's grades were reversed on measurement rather than acted on, and both
reversals are recorded in the items below (items 5 and 12). One of them would have destroyed a correct
historical record.

An orphaned-requirement sweep was run to decide item 1's shape: the twelve shell files deleted by the
shell-to-Rust migration were mapped to the spec requirements they implemented. Nine were correctly
swept when they were deleted; **three were not**, and all three are item 1.

## Phase A — spec↔reaction integrity (one OpenSpec change)

- [ ] **A1: three requirements the shell-to-Rust migration orphaned.** One `change/` branch, one squash
      PR, because all three need the same decision made per orphan rather than three separate repairs.
  - [ ] `governance-dogfood` §120-126 + 3 scenarios (focused-matrix ordering) — **REMOVE**. The decision
        was already made and recorded in `dd6e1f8`, `dod_coherence.rs` and the retired audit doc; it never
        reached the spec.
  - [ ] `reference-integrity` §25-43 + 3 scenarios (fixture-policy narrowing) — **REVISE**. The capability
        did not vanish, it changed shape: the port parameterises `offences_in` directly instead of accepting
        a fixture-set option, and `GOVERNANCE_DOCUMENTS` is a compile-time `const`.
  - [ ] `governance-dogfood` §14 + 2 scenarios (boundary-family coverage inventory) — **REVISE + BUILD**.
        Measured: all 13 named families do have an adopter-shaped owner today, so the substance holds and
        the gap is bookkeeping — nothing would notice a family losing its owner. Build it *derived both
        ways* (families from the boundary types, owners from the examples and self-law), never as a
        hand-kept inventory. Drop the stale "published 0.2.x" anchor and the unrelated `GovernanceTest`
        clause. One asymmetry the inventory would surface immediately: `AsyncExposureBoundary` is owned only
        in `sans-io-pure`'s `tests/reaction.rs`, not in its `src/governance.rs` like every sibling.
  - [ ] **A2 (part of A1, not a separate change):** a direction holding every spec's `## Subject` to paths
        that resolve. Two of the three orphans are of exactly that shape, and AGENTS.md's sync rule requires
        a revised scenario to carry its observation evidence in the same change.

## Phase B — correctness

- [x] **B1: a lock table that is not `[[package]]` ate the block above it** —
      `release_coherence_gate::require_lock_versions`. False accusation in front of `cargo publish`.
      Red-first with `Violation: Cargo.lock is missing workspace package machinery-under-another-name`.
- [x] **B2: three readers of the scenario grammar disagreed** — `bound_register_parse`. One
      `ends_scenario` predicate; the direction holds agreement between the readers, not each one's answer.
- [x] **B3: the reference gate's dated-section exemption is widest while a release is being prepared** —
      judged as designed (dating is the freeze act) and filed WATCH with its trigger, rather than narrowed.
      Narrowing would make a reference verdict depend on the release spine.

## Phase C — figures and documents

- [x] **C1+C2: the census word reader's ceiling, and an anchored record.** Merged, and the merge falsified
      the original plan: C1 was going to update `1048/310/1177` to today's counts. Measured at `ee15665`,
      those figures were exact — the sentence is a record, not a stale claim. Anchored and the noun
      corrected instead. The ≥100 refusal C2 originally proposed would have refused the first legitimate
      large figure; the ceiling is a declared bound instead.
- [x] **C3: `COOKBOOK.md`'s two elided names looked like ones the block binds.**

## Phase D — hardening

- [x] **D1: the mutation checkout asks for its own target directory.**
- [x] **D2: the pinned validator is reproduced from a committed lock.** `npm ci` + `--no-install`;
      `/node_modules/` ignored because the publish gate reads `--untracked-files=all`.
- [x] **D3: an empty passthrough no longer depends on bash 4.4.**

## Phase E — naming and duplication

- [x] **E1: `git_metadata` runs `cargo metadata`.**
- [x] **E2: the identifier tokenizer that lived twice lives once** — and stays separate from the prose
      tokenizer it resembles, which reads unicode and a different character class.
- [x] **E3: a CI step that named a defence it did not exercise.**

## Phase F — structure

- [x] **F1: `offences_in`'s four resolution rules are four functions** (260 → 167 lines).
- [x] **F2: `judge` is the sequence of its phases** (241 → 48 lines).
- [ ] The other ten Gate 4 findings are **deliberately not** in this campaign. They are pure structural
      changes to gate code that stabilised in this same window, and the risk is asymmetric. File them as
      `READY-PATCH` in `BACKLOG.md` with their measurements. The `observer.rs` one must name `COOKBOOK.md`
      as a co-edit site: the recipe points at that file as its runnable version.

## Phase G — this directory

- [x] **G: the 0.5.0 remediation queue is retired, drained.** 43/43 boxes checked, and `BACKLOG.md`'s own
      rule is that such a file is not retained. This file replaces it for the current campaign and goes the
      same way once its own boxes are checked.
