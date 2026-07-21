# AGENTS.md — 天衡 (Tianheng)

Working agreement for humans and AI agents. `PROJECT.md` is the contract (the *why* and
the invariants); this file is the *how* of contributing. Keep both short.

## Agent workflow — read the law, react against it, repair toward the reason

When you (human or agent) change code in a Tianheng-governed project, work *with* the
reaction, not around it.

**AI context order** — entering this repo, read in this order, then stop: `PROJECT.md` (the
contract and the 潛移 thesis) → [`AGENTS.self-law.md`](AGENTS.self-law.md) (the enforced self-law,
in imitable form) → the relevant `openspec/specs/*` (the capability you are touching) → the code.
`PROJECT.md` and this file stay short on purpose; the law's per-boundary detail lives in the
generated projection, and requirement detail in the specs — read those, do not inflate these.

1. **Before changing code — read the declared law.** `tianheng list --format markdown`
   (or `--format json`) projects the whole constitution: every boundary's target, what it
   forbids or restricts, and its declared reason. Read it so you know the architectural
   shape you must not drift. (The published binary's `list` projects the *demo* constitution;
   for Tianheng's **own** enforced self-law, read [`AGENTS.self-law.md`](AGENTS.self-law.md) — a
   projection generated from `self_governance.rs` and staleness-checked by `cargo test`.)
2. **After changing code — react.** `tianheng check --format json` evaluates the
   constitution against the workspace. Exit `0` is clean (or warn-only / fully baselined),
   `1` is an enforced violation, `2` is a constitution/scan/usage error.
3. **On a violation — repair toward the declared reason.** Each violation carries its
   `reason` — the intent the boundary protects. In any projection (text report, `--format
   json`, `--format sarif`), **read the `reason` first** — it is the repair direction — then
   `file` (where), then `finding` / `rule` (what tripped). Repair the code so the reason holds
   again; do not weaken the boundary to make the reaction pass.
4. **To change the law itself — amend it deliberately.** A boundary is wrong only by a
   human-reviewed amendment (an OpenSpec change / steward review), never by quietly editing
   the constitution so CI turns green. Before proposing an OpenSpec change, read the law
   projection (step 1) so the proposal reasons against the declared shape, not a guess.

This SOP is **orientation, not the binding mechanism**: the reaction (a failed `check`, a
runtime probe) is what binds: reading the law first does not *grant* compliance, it just
saves a round-trip. It is convention, not constitution — an observable architectural fact
belongs in the declared law and reacts; a working agreement like this one does not, so the
drift law keeps it here, not in `Constitution`.

## Writing a boundary's `reason` — for 潛移 (gravity)

A boundary's `because(...)` is read twice: once by a human, and — projected into an agent's
context by `list` — once by an autoregressive model that *imitates* it (see PROJECT.md, 潛移).
Write it as the **forward shape the boundary protects** ("the kernel depends inward only"),
not a backward justification ("we once hit a cycle"): the forward voice conditions original
generation, not just repair. But keep it **within the boundary's observable perimeter** — a
reason must never assert structure the law does not react to (that is prose prescription, an
open loop with no backstop). Forward voice, bounded to what reacts.

## Document authority & provenance

Each document has one job, so a fact lives in one place. `PROJECT.md` is the contract — the *why*
and the invariants, with significant calls recorded in its Decisions section.
[`AGENTS.self-law.md`](AGENTS.self-law.md) is the enforced self-law, projected from
`self_governance.rs` (never hand-edited). `openspec/specs/*` is the per-capability requirement
truth. `BACKLOG.md` records deferred work and explicit non-goals. This file is the operating
protocol for humans and agents. **Provenance — why a change was made — lives in its commit body and
PR, not a separate ADR file class.** When two documents conflict, fix the conflict (an OpenSpec
change, or a doc PR) before building on it.

## OpenSpec lifecycle

A capability change moves through OpenSpec: **explore → propose → apply → sync**. Each committed
phase is self-describing and follows *Commits & PRs* below. Propose and sync are documentation
changes — `docs(openspec): propose <change>` and `docs(openspec): sync <change>` — while apply names
the actual product effect (`feat(xuanji)!: …`, `fix(hunyi): …`, `refactor(guibiao)!: …`, and so on).
The lifecycle phase stays explicit without pretending that `propose` / `apply` / `sync` are
Conventional Commit types:

1. **explore** — investigate and shape intent; write no feature code outside a change.
2. **propose** — write `proposal.md` / `design.md` / `specs/**` / `tasks.md`.
3. **apply** — implement against the delta specs; check off a task only after verification (the
   Definition of Done below).
4. **sync** — merge the delta into `openspec/specs/*` (agent-driven).

A completed change is **not** retained as a persistent dated copy. The OpenSpec CLI folds sync
into its `archive` command, whose default *moves* the change under
`changes/archive/YYYY-MM-DD-<name>/`; once the delta is synced into the specs, Tianheng removes
**that dated copy**, while **keeping the `changes/archive/` directory itself as a tracked empty
placeholder (a single `.gitkeep`)** — the archive home is stable but never accumulates
completed-change scaffolding. Its record then lives in the main specs and git history. (Pruning
the dated copy each sync is the guardrail against the archive silently accumulating those copies;
that one placeholder also keeps `openspec/changes/` present, so no second `.gitkeep` is needed.)
These lifecycle commits never land on `main` individually — they squash up per *Branching and
release* below.

## Adversarial review stance

Work is gated by adversarial review, not performed agreement. At **propose**, challenge the design
before it is accepted: does it earn its weight against the drift law and minimalism; does it push
`xuanji` or a dimension past measure-only, or breach 三儀 ⊥ 三儀; is it a name without a reaction?
At **apply**, challenge the implementation: does the declared reaction still *bite* the boundary the
prose claims, or has the code drifted so the law passes without protecting its reason? Prefer an
independent reviewer, and verify each finding against the code before acting on it; reject or
redesign a change rather than let it pass diluted (the no-weakening-to-pass rule itself is
*Self-governance*, below). (`propose` / `apply` here are the OpenSpec phases above.)

## Commits & PRs

- **Conventional Commits.** Every non-release subject is
  `<type>(<scope>)!?: <imperative summary>` using a lowercase type and, when present, a lowercase
  package or workflow scope. Use the narrowest honest type: `feat`, `fix`, `refactor`, `docs`,
  `test`, `build`, `ci`, `perf`, or `chore`. Append `!` for a breaking change and name the migration
  in a `BREAKING CHANGE:` footer. Do not use lifecycle phases, branch roles, issue numbers, or a
  vague `update` as the type.
- **Bodies carry provenance.** Except for the release snapshot below, every commit has a concise
  body that explains why the change exists and what contract or reaction it preserves. Separate it
  from the subject with one blank line; do not merely repeat the diff or rely on a PR number.
- **PR title and body are merge inputs.** A PR title is the exact Conventional Commit subject
  intended for the squash commit. Its body uses `## Why`, `## What changed`,
  `## Adversarial review`, `## Verification`, and `## Compatibility`; the last section states the
  public/migration effect and whether manifests or package versions changed. Verification names the
  commands and external consumers actually checked — never an unqualified "tests pass".
- **Curated squash message.** For a development PR into a release branch, set the squash subject
  exactly to the PR title with no auto-appended `(#N)`. Replace GitHub's concatenated commit list
  with a self-contained body distilled from the PR's why, reaction, and compatibility result;
  retain any `BREAKING CHANGE:` footer. The branch's fine-grained commits remain review provenance,
  not the release branch's message body.
- **No AI/agent attribution.** Commit messages and PR descriptions must NOT contain a
  `Co-Authored-By: Claude` trailer, a "Generated with Claude Code" footer, a "🤖" line, or
  any other tool-authorship mark. The history records *what changed and why*, not what
  typed it. This is a project rule, not a personal preference.
- **Self-describing style.** A message says what changed and why, in its own words — not
  an issue/PR number as a crutch. A reader should understand the change from the message
  alone.

## Branching and release

`main` is release-only: it carries nothing but linear, non-merge `release: X.Y.Z` snapshot
commits, each tagged `vX.Y.Z`. The fine-grained lifecycle commits (propose / apply / sync)
never land on `main` individually — they collapse through two squash stages on the
way up: a change branch is squash-merged into `release/X.Y.Z`, and that release branch is
squash-merged into `main`.

Branch names encode role and intent: `change/<openspec-name>` exactly matches an OpenSpec change
directory; `release/X.Y.Z` is the first squash target; `polish/X.Y.Z/<slug>` carries pre-release
polish; and `refactor/<scope>-<slug>` / `docs/<scope>-<slug>` carry work outside OpenSpec. Slugs are
lowercase kebab-case, describe the outcome without an issue number, and never use a placeholder such
as `spike` after intent is known. `main` takes no direct work — it is release-only.

Both squashes are performed by a GitHub pull request's "Squash and merge", not a local merge. The
release-branch-to-`main` squash is the sole message exception: its subject is `release: X.Y.Z` and
its body is deliberately empty. A release snapshot's change is the whole tree; per-change why lives
in the curated commits and PRs below it. A PR that touches a steward-owned path
(`.github/CODEOWNERS`) is merged by the steward.

`bash scripts/check_release_coherence.sh` is the release-state reaction. During development it
requires an adopter-facing `[Unreleased]` entry and aligned workspace/internal dependency versions,
but deliberately tolerates historical lockfile drift. Once the workspace version moves forward for
release preparation—and at the exact `release: X.Y.Z` snapshot—the dated CHANGELOG section,
internal pins, and every workspace package entry in `Cargo.lock` must all name that version. The
check is read-only and needs full git history; it never bumps, commits, tags, or publishes.

Like the self-describing-commit rule above, this is a convention for humans and agents, not a
Tianheng reaction: a branching pattern is not an observable architectural fact, so the drift law
keeps it out of the constitution.

## Self-governance — don't weaken the law to make CI pass

Tianheng governs itself: `crates/tianheng/tests/self_governance.rs` runs Tianheng's own
reaction against the workspace as a `cargo test` gate. Its live invariants are declared in
`self_governance.rs` and projected into [`AGENTS.self-law.md`](AGENTS.self-law.md); do not
hand-maintain a second list here.

If a change makes this test fail, **fix the change**, not the test. A boundary is altered
only by a deliberate, human-reviewed amendment to `self_governance.rs` — never by quietly
weakening it so CI turns green.

## Definition of Done

Run these from the workspace root before checking off an apply task, syncing, or reporting a change
done. This is the single source for the local pre-flight gate list (so other docs need not restate
it); CI runs a superset of it:

```bash
cargo build --workspace
cargo clippy --all-targets --all-features -- -D warnings
cargo clippy --workspace -- -D warnings   # shipped lib/bins only (no --all-targets, default features):
                                           # catches dead code that ships in the crate but is masked by
                                           # the --all-targets passes above (a test constructs an item
                                           # that is dead in the library — e.g. a feature-gated variant)
cargo clippy -p louke -- -D warnings       # louke's audit-OFF library on its own: every --workspace pass
                                           # feature-unifies louke/audit ON (the tianheng shell enables it),
                                           # so only an isolated louke build sees the prod-light config where
                                           # an unused audit-gated item would otherwise hide until publish
cargo fmt --all --check
TIANHENG_WORKSPACE_TESTS=1 cargo test --workspace --all-features
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features
cargo deny check
bash scripts/test_release_coherence.sh # prove every release state and failure direction
bash scripts/check_release_coherence.sh # react against this checkout (requires release history)
bash scripts/test_examples.sh            # every dogfood example still reacts as declared
```

The self-governance gate (`self_governance.rs`, run under `cargo test`) and its projection
(`self_law_projection_is_fresh`) must stay green — never weaken the law to pass it. Beyond the list
above, CI also runs a **default-features** `clippy`/`doc` pass (catching an unused item or a broken
intra-doc link when the `audit` feature is off), the declared-MSRV build and test, license-text
bundling, the packaged-tarball self-test, and the reaction on the clean/violating fixtures (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)).

## Versioning — SemVer honesty (the modou lesson)

- Pre-1.0 and at `0.0.x`: **no inter-release compatibility is promised**; any release may
  break. Do not vanity-bump the minor for a non-breaking change.
- Graduate to `0.1.0` only when the public API has settled enough to promise
  `0.1.x`-patch compatibility. After that: non-breaking → patch, breaking → minor.

## Drift law & minimalism (inherited, non-negotiable)

- **No drift type without an observation source; no target or name without a reaction** —
  at module, crate, and dimension granularity. Do not pre-create empty `semantic`/`runtime`
  crates or stub modules; a dimension's crate is born when it is built.
- **Fail loud only on observable misconfiguration.** No defensive over-foolproofing of
  impossible states.

## Outward / irreversible actions — confirm first

Merging to `main`, tagging, publishing to crates.io, force-pushing, and deleting a repo
are confirm-first: get explicit human sign-off even if a permission rule would auto-allow
it. (crates.io publishes are permanent — only yankable, never deletable.) A local
`.claude/settings.local.json` `permissions.ask` rule on `gh pr merge` is a recommended way to
mirror this in a dev environment, but the confirm-first rule binds regardless of local settings.

Before publishing, confirm every publishable crate **bundles its license texts**: `cargo
publish` packages only files inside each crate's own directory, so the workspace-root
`LICENSE-*` and the inherited SPDX `license` field are not enough — each crate must physically
carry `LICENSE-MIT` and `LICENSE-APACHE`, or it ships without them (as 0.1.0/0.1.1 did, before
this was caught). `cargo package --list -p <crate>` shows exactly what a crate would ship. This
is release/packaging hygiene, not architectural drift, so it is a **CI reaction** (the
`License texts bundled` job), never a Tianheng constitution boundary — the same reason the
branching/release ritual above stays convention rather than a reaction.
