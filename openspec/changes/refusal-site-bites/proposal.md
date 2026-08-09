## Why

`rust-self-governance-gates` already requires that a reaction reaching both outcomes "carry the distinction in
its return type" and that "its directions assert the kind rather than merely that it refused". Nothing holds
that requirement. Four adversarial rounds over one window each found the class the previous round had
repaired, and the decisive difference was the **method**: disabling an `if` and re-running finds an
unreachable branch; swapping `violation`↔`cannot_judge` at each construction site **and** replacing the
message finds a direction that asserts a refusal happened without asserting *which*. That sweep was a review
technique, not a reaction — so it stopped when the reviewer stopped, and its findings were recorded as a
count rather than as a floor.

Measured at `22ec98e` over the two kinded gates: **60 refusal construction sites, 24 surviving both
perturbations**. That number is not one fact. A perturbation kills nothing for two different reasons — no
direction *distinguishes* the site, and no direction *reaches* it — and a sweep run by hand cannot tell them
apart. Both are refusals that can silently change kind or message in front of `cargo publish`, which is
irreversible and whose kind is what an operator acts on, but they are closed by different work, and reporting
them as one number hides which.

## What Changes

- **One `Refusal` type instead of two.** `publish_source_gate.rs` and `release_coherence_gate.rs` each define
  their own `Kind`, `Refusal`, `violation`, and `cannot_judge`. Two constructions of one concept is the
  twin-drift class this repository keeps closing; they collapse into `crates/tianheng/tests/support/refusal.rs`
  and both gates use it.
- **A refusal site becomes perturbable at run time.** The two constructors take `#[track_caller]`, read
  `Location::caller()` **in their own body** — inside the propagation chain, so the location is the site and
  not the shared constructor's interior — and consult one injection point that can swap the site's kind or
  replace its message.
- **A new reaction, `crates/tianheng/tests/refusal_bites.rs`.** It enumerates every refusal site statically,
  records which sites the suite actually reaches, and then, for each reached site, runs the test targets that
  can observe it twice: once with that site's kind swapped and once with its message replaced. Some direction
  must fail under **each** perturbation, because the kind and the message are two independent contracts — a
  site observed only in kind lets its message rot into a sentence about something else.
- **Survivors are red, in two named classes.** A reached site that no direction distinguishes fails. A site
  the suite never reaches fails too, unless it is declared out of reach — and it is reported as its own class,
  because the two are closed by different work and one number hides which.
- **The enumeration is total by refusal, not by cleverness.** The site search runs over the whole text so a
  call may wrap; a wrapped call no direction reaches would otherwise be invisible to the static enumeration
  and to the reach recording at once. Two forms that would still evade a search for a name — an import that
  renames a constructor, and a constructor taken as a value — **fail** rather than being followed, because
  following either means resolving names, which is the compiler's job.
- **A residual is closed in order: construct it, or delete it, before declaring it.** A refusal branch that a
  preceding check makes logically unreachable is dead code, and deleting it is the smaller change than
  declaring a bound about it.
- **An out-of-reach site is declared in source and joined to a bound.** Sites that genuinely cannot be
  constructed — `ssh-keygen` absent, the signature mechanism failing its own round trip — use a named
  constructor form carrying a stable slug. A **repository-local typed registry** in test support joins each
  slug to the `BoundId` covering it; `Extent` is a shipped public type and is not widened to carry exemption
  membership. The reaction refuses a slug no site carries, a slug two sites share, a slug no registry entry
  covers, a registry entry naming a `BoundId` the live bound set does not contain, and a slug whose site turns
  out to be reachable after all.
- **The corpus is what the compiler read, and it must be tracked.** Targets and their source lists come from
  `cargo test --no-run --message-format=json` and the dep-info beside each executable — not from a
  reimplementation of module resolution, which misses `mod`, `include!` and conditional paths and admits
  `cfg`-excluded files. HEAD is not the source either: it would compare one tree's text against another
  tree's run, leaving a new uncommitted unreachable site invisible to the enumeration and the recording at
  once. A file in that corpus that `git ls-files` does not name fails separately.
- **Re-declaring the shared refusal vocabulary outside the shared module fails** — by exact name. A gate that
  renames its vocabulary is outside, and that is a **declared bound**, not a claim of coverage: recognising a
  vocabulary by intent is a judgement over source, and nothing compile-time reaches a gate not yet written.
- **`scripts/publish.sh` scrubs the instrumentation.** It invokes the publish-source gate under
  `env -u TIANHENG_REFUSAL_MUTANT -u TIANHENG_REFUSAL_RECORD -u TIANHENG_REFUSAL_BITES`, so a stray variable
  in an operator's environment cannot make the gate that stands before an irreversible act report clean, nor
  run the reaction's machinery where only its verdict was asked for.
- **Env-gated and named where the run is decided**, on its own line in the `AGENTS.md` Definition of Done and
  in CI, exactly as the mutation suite and the examples suite are.

**Compatibility.** The test machinery ships in no package. One packaged file does change: `crates/tianheng/src/bounds.rs`,
whose `observation_bounds()` is `pub` at `crates/tianheng/src/lib.rs:52`. The change there is one added element
in the returned set — no signature change, no type change, no migration for any caller. The exemption registry
that joins slugs to that `BoundId` is deliberately **not** put in `Extent`, precisely so the public type does
not grow a field to serve a test.

## Capabilities

### New Capabilities

None. The authority already exists.

### Modified Capabilities

- `rust-self-governance-gates`: the requirement that a reaction's directions assert **which** outcome a shape
  produces gains a reaction that holds it, the rule that a site out of reach is declared rather than assumed,
  and the identity rule that a perturbation selector is not an exemption identity.

## Impact

- **Rewritten**: `crates/tianheng/tests/support/publish_source_gate.rs`,
  `crates/tianheng/tests/support/release_coherence_gate.rs` — their refusal vocabulary is removed and imported;
  every site keeps its message and kind.
- **New**: `crates/tianheng/tests/support/refusal.rs`, `crates/tianheng/tests/refusal_bites.rs`.
- **Amended**: `crates/tianheng/tests/publish_source.rs`, `publish_source_integrity.rs`,
  `release_coherence.rs` — they gain the shared module and lose their local `Kind` imports.
- **Amended**: whichever directions the reaction finds missing, in those three targets. The population is
  measured, not assumed — the 24 counted on the earlier tree bounds neither class, because it merges them, so
  both figures come from the reaction's first run rather than from an estimate here.
- **Amended**: `crates/tianheng/src/bounds.rs` (one added declaration), `crates/tianheng/tests/census.rs` (the
  exempt-count census, produced by the shared site enumerator), `scripts/publish.sh`, `AGENTS.md`,
  `.github/workflows/ci.yml`, `CHANGELOG.md`, `BACKLOG.md` (the `WATCH` entry this change closes),
  `docs/observation-bounds.md` (generated).
- **Cost**: measured, not guessed — 794ms + 137ms + 255ms per target run, no rebuild between perturbations,
  so the whole sweep is roughly one minute of process time. It is gated anyway, so the ordinary suite is
  unaffected.
