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

Where the law stops is written down too: [`docs/observation-bounds.md`](docs/observation-bounds.md)
projects every **observation bound** — each claim that a reaction deliberately stops at a named shape —
with the test that defends it, or the tracker that owns closing the gap. Read it before reporting a
behaviour as a defect: a declared bound means the shape is governed policy, and the projection leads with
the count of bounds nothing yet defends.

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

Governance follows a **Three-Layer Architecture**:
1. **Layer 1: Reaction Backstop (Code)** — Functional boundaries (`restrict_dependencies_to`, `must_not_call_inline`) enforce hard invariants in code. Minimalism forbids redundant reactions (do not add a denylist for a prohibition an allowlist already enforces).
2. **Layer 2: Qiányí Gravity Pull (Prose Reason & Projection)** — `because(...)` reasons project into `AGENTS.self-law.md` to condition LLM continuations. Write reasons strictly in a **forward voice** ("the kernel depends inward only"), never as a backward justification or historical debrief ("we once hit a cycle in 0.2.2"): **provenance belongs in `PROJECT.md` decisions and git history, not in the live context reason.**
3. **Layer 3: Provenance & History (Doc)** — Historical rationale, lessons learned, and decision context stay in `PROJECT.md` decisions and commit history, keeping live context dense and noise-free.

Keep every reason **within the boundary's observable perimeter** — a reason must never assert structure the law does not react to (that is prose prescription, an open loop with no backstop). Forward voice, bounded to what reacts, minimal in reactions.

## Document authority & provenance

Each document has one job, so a fact lives in one place. `PROJECT.md` is the contract — the *why*
and the invariants, with significant calls recorded in its Decisions section.
[`AGENTS.self-law.md`](AGENTS.self-law.md) is the enforced self-law, projected from
`self_governance.rs` (never hand-edited). `openspec/specs/*` is the per-capability requirement
truth. `BACKLOG.md` records deferred work and explicit non-goals. This file is the operating
protocol for humans and agents. **Provenance — why a change was made — lives in its commit body and
PR, not a separate ADR file class.** When two documents conflict, fix the conflict (an OpenSpec
change, or a doc PR) before building on it.

Backlog entries are decision inputs, not an undifferentiated wish list. Classify live work by its
evidence and next trigger (`READY-PATCH`, `DESIGN-BREAKING`, `WATCH`, or `ACCEPTED DEBT`), keep
rejected directions under `DECLINED`, and move shipped work to `BUILT / HISTORY`. Promotion into
implementation requires the entry to name its observation source, risk, compatibility class, and
authority in `BACKLOG.md`; a breaking candidate does not promise a minor release until its recorded
trigger fires.

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

**A guard is not a guard until it has been seen to fail.** Every new test that claims to protect a
change must be run against the code *without* that change, and the observed failure recorded in the
PR's `## Verification`. A test written from the same understanding as the fix inherits its blind
spots, so passing afterwards proves nothing on its own; only the negative run distinguishes a guard
from a restatement.

The trap this exists for is the change whose outcome is unaltered. When a fix improves a
**diagnostic** while the exit code, return value, or wire output stays identical, a test bound to
that outcome passes equally before and after — it pins the surrounding contract, not the change.
Choose the observation level the change actually moved (stderr text, the emitted document, a
syscall sequence), and where a test genuinely cannot reach it, say so in the PR and state what
evidence stands in its place instead of leaving the reader to assume a green suite covered it. A
test kept for the contract rather than the change earns a comment saying which it is.

A vocabulary- or identity-level breaking change additionally requires grepping every touched spec
and doc for the retired term across its *whole* file, not only the new diff: sync bolting on a
correctly-worded requirement while the same file's older prose still names the retired shape is
itself an undetected drift, invisible to a diff-only read (the 0.3.0 `finding_key` lesson).

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
  commands and external consumers actually checked — never an unqualified "tests pass" — and, for
  each new guard, the failure observed without the change (see *Adversarial review stance* above).
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

Branch names encode role and intent. Two roles are fixed: `change/<openspec-name>` exactly matches
an OpenSpec change directory, and `release/X.Y.Z` is the first squash target. All other work uses
`<type>/<scope>-<slug>`, where `<type>` is the *Conventional Commit type the work will land as* (the
same set *Commits & PRs* above admits — `fix`, `test`, `refactor`, `docs`, `feat`, `ci`, and so on),
so a branch's role and its squash subject cannot disagree. Deriving the role from the commit type is
deliberate: an enumerated list of blessed prefixes drifts from what the repository does, and a
governance rule that has drifted is read as license rather than law. Pre-release polish therefore
takes the type its own work lands as; there is no separate release-staging role, because a branch's
role is what it does, not when it happens. Slugs are lowercase kebab-case, describe the outcome
without an issue number, and never use a placeholder such as `spike` after intent is known. `main`
takes no direct work — it is release-only.

Both squashes are performed by a GitHub pull request's "Squash and merge", not a local merge. The
release-branch-to-`main` squash is the sole message exception: its subject is `release: X.Y.Z` and
its body is deliberately empty. A release snapshot's change is the whole tree; per-change why lives
in the curated commits and PRs below it. A PR that touches a steward-owned path
(`.github/CODEOWNERS`) is merged by the steward. A release branch is archived once it merges; it
carries no further work and is never a source of record for anything downstream.

**`main` is also the only publish source.** After the release squash and the signed `vX.Y.Z` tag, the
crates.io publish runs from a checkout of *that tagged `main` commit* — never from the release
branch. `cargo publish` stamps the sha1 of whatever `HEAD` it ran on into every tarball's
`.cargo_vcs_info.json`, and a published version can never be re-uploaded, so the pointer is permanent
from the moment it lands. An identical tree does not make a release branch's tip an acceptable
source: cargo records the **commit**, not the content, and the commit it would record belongs to a
branch the ritual archives. `bash scripts/publish.sh` is that path — it runs
`scripts/check_publish_source.sh` (worktree clean; `HEAD` the `release: X.Y.Z` snapshot for the
workspace version; `vX.Y.Z` annotated, signed, and pointing at it; `HEAD` the live tip of
`origin/main`, read from the remote rather than a possibly-stale `refs/remotes/`) and only then
`cargo publish --workspace`. The gate reads `0` publishable, `1` wrong source, `2` cannot judge. The
wrapper forwards extra arguments to cargo but **refuses `--manifest-path`** (either spelling) before
the gate runs: it would move cargo's workspace root away from the tree the gate judged, which is the
wrapper's whole claim undone by one argument. The registry-side arguments (`--registry`, `--index`,
`--token`) change the destination rather than the source and stay forwarded.

**A published release snapshot is immutable.** Once a version is on crates.io, its `release: X.Y.Z`
commit must never be amended or force-pushed away: the published artifact points at that sha1
permanently, so replacing it orphans the pointer just as surely as publishing from the wrong branch
does. `0.2.2` was published from `main` correctly and then force-pushed away an hour later, which the
publish-source gate cannot foresee — at publish time it would have passed — so this half stays a
convention. What each published version actually records, and the two mechanisms that produced the
disagreements, is inventoried in
[`docs/history/published-artifact-provenance.md`](docs/history/published-artifact-provenance.md).

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
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --document-private-items
                                           # --document-private-items is NOT optional: nearly every item in
                                           # these crates is crate-private, so without it a broken intra-doc
                                           # link in that majority is invisible locally (17 had accumulated
                                           # before CI gained the flag, every one pointing at something a
                                           # module split had moved or renamed)
cargo deny check
bash scripts/test_whitespace_hygiene.sh  # prove every refusal of whitespace hygiene: this gate had no matrix,
                                           # and it is where the shared exit-contract backstop first misfired —
                                           # printing cannot-judge once per clean file while still exiting 0, so
                                           # every check reading only the code reported it passing
bash scripts/check_whitespace_hygiene.sh # `cargo fmt` governs .rs only; nothing checked .md/.toml/.sh/.yml,
                                           # so three blank lines at EOF reached a release branch through 23
                                           # touched spec files and two full-range adversarial reviews
bash scripts/test_reference_integrity.sh # prove every refusal of in-repo path reference checking:
                                           # fixture-based failure matrix proves broken links/paths fail loud
bash scripts/check_reference_integrity.sh # every in-repo path a document or comment points at must exist:
                                           # this class was hand-swept twice (once for .md only) and a module
                                           # split landing after that sweep reintroduced it in nine places
bash scripts/test_dod_coherence.sh       # prove every refusal of the coherence gate that binds this list to
                                           # CI: it was the last gate with no matrix, so the claim this block
                                           # makes about itself rested on a reaction nobody had watched refuse
bash scripts/check_dod_coherence.sh     # this list is a subset of CI's — checked, not promised
bash scripts/test_release_coherence.sh # prove every release state and failure direction
bash scripts/check_release_coherence.sh # react against this checkout (requires release history)
bash scripts/test_publish_source.sh     # prove the publish-source gate refuses every wrong source. The
                                           # gate itself runs at publish time (see Branching and release),
                                           # not here — no development checkout is a release snapshot. Its
                                           # matrix is: 0.4.0's six tarballs recorded the release branch's
                                           # tip instead of main's tagged release commit, and that stamp
                                           # can never be re-uploaded
bash scripts/test_bound_register.sh     # prove every refusal of the observation-bound register: a gate whose
                                           # subject is absence can refuse nothing and still read as protection
bash scripts/check_bound_register.sh    # every declared observation bound names the test that defends it or the
                                           # tracker that owns closing it, every bound stated in prose is declared,
                                           # docs/observation-bounds.md matches the specs, and a tracked Markdown
                                           # document writing "N bounds across M capabilities" agrees with what the
                                           # run counted — a clean run prints the figures, so prose is written from
                                           # a measurement rather than from memory. These two lines sit
                                           # AFTER cargo test deliberately: whether a citation names a test that
                                           # RUNS is decided by the harness's own enumeration (cargo test -p … --
                                           # --list), because three reviews defeated deciding it from source text
                                           # (a cfg-removed #[test], an uninvoked macro body, a definition inside
                                           # a string or comment). Run on a cold checkout they compile the
                                           # workspace; run in this order the enumeration is warm (≈1s)
bash scripts/test_published_family_coverage.sh # prove the published-family ledger refuses: a family with no
                                           # fulfilled owner, and an owner claiming a family the inventory does
                                           # not list. The ledger runs inside test_examples.sh below; these are
                                           # the proofs that it still refuses, and nothing ran them
bash scripts/test_example_quality_gate.sh # prove a real isolated-workspace warning stops the gate before
                                           # reaction acceptance
bash scripts/test_example_suite.sh       # prove example ownership and invocation-local artifact cleanup
bash scripts/test_examples.sh            # every dogfood example still reacts as declared
```

The self-governance gate (`self_governance.rs`, run under `cargo test`) and its projection
(`self_law_projection_is_fresh`) must stay green — never weaken the law to pass it. So must
`observation_bound_model.rs`, which holds every declared observation bound's spec scenario and its typed
classification in a bijection and projects `docs/observation-bound-extents.md`; it needs no line of its own
above because it runs under that same `cargo test`. Beyond the list
above, CI also runs a **default-features** `clippy`/`doc` pass (catching an unused item or a broken
intra-doc link when the `audit` feature is off), the declared-MSRV build and test, license-text
bundling, the packaged-tarball self-test, and the reaction on the clean/violating fixtures (see
[`.github/workflows/ci.yml`](.github/workflows/ci.yml)).

## Versioning — SemVer honesty (the modou lesson)

- Pre-1.0 and at `0.0.x`: **no inter-release compatibility is promised**; any release may
  break. Do not vanity-bump the minor for a non-breaking change.
- Graduate to `0.1.0` only when the public API has settled enough to promise
  `0.1.x`-patch compatibility. After that: non-breaking → patch, breaking → minor.
- **Breaking means the adopter has to act**, which is wider than a moved API. A changed public
  signature, wire format, or identity shape breaks — and so does a change that leaves a recorded
  baseline no longer describing the adopter's tree, because regenerating it is work they did not
  choose. **Closing a false negative therefore earns a minor**, however small its diff: the reaction
  is additive, the baseline is not, and "the defect was ours" does not spare them the work. So does
  new depth that reacts by default. Patch-class is what an adopter can take without doing anything —
  packaging and hygiene, prose and specs, opt-in depth, performance, and a diagnostic whose exit code
  and emitted documents are unchanged. `CHANGELOG.md` states the same fact as its `**BREAKING**`
  marking rule, which is that projection for adopters; the version consequence is here, because a
  release number is decided before the notes are written.

## Drift law & minimalism (inherited, non-negotiable)

- **No drift type without an observation source; no target or name without a reaction** —
  at module, crate, and dimension granularity. Do not pre-create empty `semantic`/`runtime`
  crates or stub modules; a dimension's crate is born when it is built.
- **Fail loud only on observable misconfiguration.** No defensive over-foolproofing of
  impossible states.

## Outward / irreversible actions — confirm first

Merging to `main`, tagging, publishing to crates.io, force-pushing, and deleting a repo
are confirm-first: get explicit human sign-off even if a permission rule would auto-allow
it. (crates.io publishes are permanent — only yankable, never deletable, and a version's recorded
source commit is permanent with it — so *where* the publish runs from is gated rather than
remembered; see *Branching and release*.) A local
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
