# Tasks — Clean carries its subject

Each task names the signal that says it landed. A guard is not a guard until it has been seen to
fail, so every new refusal records its negative run. The whole Definition of Done runs before each
commit — not a subset; this window has twice shipped a defect through a partial one.

## 1. The type

- [ ] `xuanji`: `Subject`, `#[non_exhaustive]`, private fields, accessors for `declared` and
      `reached`, and a fallible constructor that refuses declared-without-reached.
      **Signal**: a unit direction constructs `(0, 0)`, `(3, 7)` and `(1, 0)`, and only the third is
      refused. Negative run: with the refusal removed, the third row passes.
- [ ] `xuanji`: `Outcome::Clean` carries it.
      **Signal**: the workspace no longer compiles until every construction site supplies one — the
      compiler is the enumerator, and the count of sites it names is recorded in the PR.

## 2. The dogfood — this is the adversary, not a formality

Each dimension supplies both numbers from the function that already classifies its outcome. If any
dimension cannot, **stop, revert, and report**: a number threaded from elsewhere or invented to fit
is the design failing, not the dimension.

- [ ] `guibiao::evaluate` — `constitution.boundaries()` and `workspace_member_names(metadata)`.
      **Signal**: an existing clean fixture still reports clean, now carrying both figures.
- [ ] `hunyi::check_all` — `boundaries`, and the member count from the metadata it already reads.
      **Signal**: the empty bundle still returns clean **without reading the manifest**, carrying
      `(0, 0)`. The existing direction that holds the no-metadata-read property must stay green;
      if supplying `reached` forces a read, the design is wrong for this dimension.
- [ ] `louke::audit_probe_coverage` — the `declared` and `roots` parameters.
      **Signal**: an existing clean audit still reports clean, carrying both.
- [ ] `louke/tests/adopter_surface.rs`'s `ExternalObserver` — the third-party shape.
      **Signal**: it compiles and returns a subject without reaching into any dimension's internals.
      This is the row that says the shape is usable from outside; if it needs anything not public,
      that is a finding about the surface rather than about the fixture.

## 3. The composition

- [ ] `tianheng`: the composed run's own clean verdict carries a subject.
      **Signal**: a run over two clean observers reports the aggregate; a run over none still reports
      `ConstitutionError`, unchanged.
- [ ] `tianheng`: `Subject` joins the prelude.
      **Signal**: `prelude_promise` names it, and the external compilation contract mentions it —
      both already fail on an unnamed promised member, so this is held by an existing check.

## 4. What the operator and the agent receive

- [ ] The text rendering states the subject.
      **Signal**: a clean run's stderr names what was declared and reached, rather than only
      `clean — no boundary violated`.
- [ ] The JSON and SARIF projections carry it.
      **Signal**: the projection directions assert the field; a clean run's document is readable by
      an agent without parsing prose.

## 5. The bound

- [ ] Declare the stated bound: a participant may report a subject larger than it observed, and
      nothing reacts.
      **Signal**: the typed catalog carries it, the bijection holds, and both projections regenerate.
      Its pin is the direction asserting the constructor is public and the value unverified — a bound
      whose pin asserts a silence needs a control, so that direction also shows the constructor
      refusing the one combination it does refuse.

## 6. Migration

- [ ] `CHANGELOG.md` `[Unreleased]` records the break with `**BREAKING**` and a `### Migration`
      section naming the mechanical fix.
      **Signal**: `release_coherence` holds a section carrying `**BREAKING**` to carrying
      `### Migration`; it is already green and must stay so.
- [ ] Sweep the window's own prose for claims this change falsifies.
      **Signal**: nothing in `[Unreleased]`, the specs, or the doc comments still describes `Clean`
      as carrying nothing.
