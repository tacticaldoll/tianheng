## Context

`crates/tianheng/tests/` holds two unrelated populations: 17 targets that judge **this repository** — its
changelog, its specs, its scripts, its documents, and its own declared law — and 8 that test **the crate** —
the cross-dimension conformance matrices, the CLI, and the adopter-surface compile contract. Only the second
kind has anything to do with the package they are all inside, and all 25 ship in its tarball.

## Goals / Non-Goals

**Goals:**
- The governance apparatus ships in zero packages, which is the criterion its own capability already states.
- The two populations are told apart by **position**, so a claim about one cannot silently cover the other.
- The packaged self-test runs tests that exercise the packaged crate.

**Non-Goals:**
- Renaming anything. The vocabulary rectification is the next change, written against the new locations.
- Changing what any reaction judges. Every refusal, every bound, every citation keeps its meaning.
- A new mechanism to keep the split honest — the change that follows declares each capability's subject and
  joins a proposal to what it touches, which is exactly that mechanism. Building a second one here would be
  the redundant reaction the minimalism bound forbids.

## Decisions

### The member is 繩墨 (`shengmo`), and it is deliberately not an instrument

The family's published crates are astronomical instruments: 璇璣, 星表, 圭表, 渾儀, 漏刻, 天衡. This member is
not one of them and must not read as a seventh. 繩墨 — the carpenter's inked line — is a measuring tool of a
different kind: everything is judged straight against it, and the line is not part of the furniture. That last
clause is the property being fixed.

`publish = false`, and no `LICENSE-MIT` / `LICENSE-APACHE` in its directory: `cargo publish` packages only
crate-local files and never packages this one, and CI's `license-files` job already skips members declaring
`publish = false`. Its manifest still inherits `license.workspace = true` so its metadata matches its
siblings.

### The split is a judgement, and the location becomes the declaration

Two mechanical classifiers were tried and **both were measured unreliable**:

- *Does the target's text name a repository artifact?* — `baseline_cli` names `AGENTS.md` in a comment and is
  a crate test; `whitespace_hygiene` scans every tracked file and names none.
- *Does the target consult `TIANHENG_WORKSPACE_TESTS`?* — the marker means two different things today: "this
  needs the repository as its subject" and "this needs a fixture, or a real workspace to run the product
  against". `baseline_cli` and `observer_protocol` both consult it, for the two different reasons.

So the split is stated, not derived. What keeps it from being a claim is that after the move the **location is
the declaration** — there is no separate list to drift from. What keeps a new reaction from landing on the
wrong side is the following change's subject declarations and its proposal join.

**Moving (17)** — subject is this repository: `self_governance`, `bound_register`, `census`, `dod_coherence`,
`examples_suite`, `merge_message`, `observation_bound_model`, `observer_protocol`, `pin_bites`,
`projection_register`, `publish_source`, `publish_source_integrity`, `reference_integrity`, `refusal_bites`,
`release_coherence`, `source_regions`, `whitespace_hygiene`.

**Staying (8)** — subject is the code in the tarball: `adopter_surface`, `baseline_cli`, and the six
`*_conformance` matrices.

### `support/` splits where its users do

Nine of the ten support modules serve only moving targets and move with them: `bound_register_parse`,
`census`, `merge_message_gate`, `publish_source_gate`, `refusal`, `refusal_exemptions`, `refusal_sites`,
`region`, `release_coherence_gate`.

`support/mod.rs` — the `TempFixture` plumbing the conformance matrices share — stays, and drops its
`pub mod region;`. Measured rather than assumed: the only users of `region::Source` are `observer_protocol`,
`projection_register`, and `source_regions`, all three of which move.

### The support modules stay `#[path]`-included rather than becoming the member's library

`crates/shengmo/src/lib.rs` carries documentation and no code; the tests keep including their support modules
through `#[path = "support/…"]` exactly as today.

The alternative — making the support modules the member's actual library and replacing every `#[path]` with
`use shengmo::…` — is the better long-term shape and is **deliberately deferred**. `refusal_sites` builds its
corpus from what rustc reports having read, and `refusal_bites` runs every judged binary against that corpus;
changing both the location and the inclusion mechanism in one step would make a failure in either
indistinguishable from a failure in the other. The move is verified first.

## Risks / Trade-offs

- **[A package whose only real content is tests]** → `src/lib.rs` exists and is documented rather than empty,
  so the member has a target and a reader is told what it is for. Whether the repository's own constitution
  reacts to a doc-only crate is verified during apply, not assumed.
- **[`refusal_sites` enumerates its corpus from dep-info of test targets]** → the corpus follows the targets
  because it is produced from the build rather than from a list. Confirmed by running `refusal_bites` after
  the move and comparing its census against the pre-move figures.
- **[Every invocation of a moved target must follow]** → they are enumerable: `--test <name>` in `ci.yml`,
  `AGENTS.md`, and `scripts/`. `dod_coherence` holds the DoD-to-CI correspondence, and `reference_integrity`
  holds every path reference, so a missed one is red rather than silent — which is also why those two move
  last in the task order.
- **[The published crate loses 16 test targets from its tarball]** → nothing an adopter could run. The
  packaged self-test is strengthened by it, not weakened: what remains is tests whose subject is the packaged
  code.
