## Why

A declared **observation bound** — where an observation deliberately stops, so a shape that looks like
a defect is in fact governed policy — has no single declaration site and no machine-checkable link to
the test that pins it. Measured on this tree: 55 lines name a bound across `openspec/specs/*` — 25 of them
scenario headings that declare one, 29 prose or bare THEN clauses that do not — plus 69 occurrences across
`crates/*/src` rustdoc, 3 in `BACKLOG.md`'s ACCEPTED DEBT, and 21 tests following the
`*_is_a_stated_bound` / `*_is_a_documented_bound` naming convention. Those sets do not correspond: the
symlinked-directory bound's pinning test is cited by name in `BACKLOG.md` and is **not** among the 21,
so the naming convention cannot serve as the index either.

Two costs follow, and the 0.4.0-window sweep paid both. First, an auditor cannot tell a defect from a
declared bound without reading the whole surface — classifying one hypothesis about composite origin
shapes required finding and reading a specific `runtime-origin-assertion` paragraph, and that lookup is
not incidental, it is the tax for having no index. Second, and worse, **"the audit was thorough" is
unfalsifiable**: with no enumeration of what should have been checked, an audit can only report the
hypotheses someone happened to invent. A sweep that invents its own scope cannot be dry, only tired.

A bound is also the one claim class most exposed to drift: it asserts that a *reaction stops here*,
which is precisely the assertion no reaction defends. `PROJECT.md`'s drift law forbids a name without a
reaction; roughly a hundred bound claims are that shape today.

## What Changes

- A **bound register** built on the convention this repository already has, not a new one: a bound is
  declared as a `#### Scenario:` under its capability's `### Requirement: Observation bounds …`, and gains
  a citation bullet naming the **test that pins it**. Three specs carry that requirement today and 25
  scenarios already name themselves as bounds, so the register consolidates onto the existing shape; the
  remaining work is migrating the 29 unmarked prose statements into it. A bound's id is derived from
  `<capability>/<scenario-slug>` rather than allocated, so no ledger is introduced.
- A **reaction** over the register, with two directions:
  - every registered bound names a test that **exists in the tree** (a register entry pointing at a
    renamed or deleted test fails, rather than reading as covered);
  - every bound-prose occurrence a spec can be scanned for sits **inside** a declared bound scenario, so a
    bound stated in prose and left undeclared fails rather than being silently absent from the index —
    measured today as 11 specs stating bound prose with no Observation-bounds requirement at all.
- A **generated index projection** of the register, ordered and stale-checked the way
  `AGENTS.self-law.md` is: the document is derived, never hand-maintained, and a stale one fails.
- The index **states its own completeness bound**: the second reaction direction above is a floor over
  prose the scan can recognize, not a proof that no unstated bound exists. The projection says so in
  its own header, because an index that implies totality lies exactly where it is most trusted.
- Scope deliberately excluded from this change: bounds stated **only** in rustdoc (69 occurrences) and
  in `BACKLOG.md`'s ACCEPTED DEBT. Registering those means moving a claim between documents, which is a
  requirement-surface change per capability and earns its own change. Consolidating what specs already
  state claims nothing new.

Not breaking: additive, adopter-invisible, and requiring no adopter action — patch-class under
`AGENTS.md`'s *Versioning* definition.

## Capabilities

### New Capabilities
- `observation-bound-register`: how an observation bound is declared, what a declaration must carry (id,
  statement, pinning test), the two reaction directions over the register, the generated projection and
  its staleness reaction, and the register's own stated completeness bound.

### Modified Capabilities
<!-- None. Existing spec files gain register entries for bounds they already state, so no capability's
     requirements change; the register's own rules are owned by the new capability above. A bound whose
     only statement today is in rustdoc is out of this change's scope precisely because registering it
     WOULD change its capability's requirement surface. -->

## Impact

- **`openspec/specs/*`**: citation bullets added to the 25 scenarios that already declare a bound, and
  bound scenarios added to the 11 specs stating bound prose without an Observation-bounds requirement.
  Content grows; no capability's reactions change.
- **New**: the register reaction and the generated index projection, plus their own failure-direction
  test — a gate on a claim about coverage must be proven to fail when coverage is absent, or it is a
  restatement.
- **`AGENTS.md`**: the Definition of Done gains the register reaction, mirrored verbatim into
  `.github/workflows/ci.yml` (`check_dod_coherence.sh` enforces that mirroring).
- **`CHANGELOG.md`**: an `[Unreleased]` entry; the projection is a new adopter-readable surface even
  though no adopter action follows.
- **No crate API, wire format, identity shape, or manifest change.** No dimension gains or loses an
  observation.
- **Downstream**: this is the instrument the next audit is run against, so the audit's scope stops being
  invented per sweep. That consequence is why the change is worth its weight, and it is not itself in
  scope here.
