# Changelog

All notable changes to the 天衡 (Tianheng) crate family. This is the **adopter-facing**
projection of the release history; the per-change *why* lives in the squashed change commits and
their pull requests. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Versioning is **SemVer honesty** for a pre-1.0 line (see `AGENTS.md`): the family is
**experimental / pre-1.0**. It held at `0.1.x` deliberately until real adopters arrived; `0.2.0` is
the first deliberate minor past that hold. Pre-1.0, packaging/hygiene and **opt-in** depth on an
existing observation source are patch releases, a breaking change earns a minor, and no release
intentionally breaks the adopter-written builder (`Constitution` / boundary DSL / `run`). Depth that
reacts *by default* is not patch-class, whatever its diff size, because it follows the marking rule
below: it makes an adopter's recorded baseline stale, and that is work they did not choose. (`0.1.3`'s
default-on re-export exposure predates this and shipped as a patch.)

**What earns `**BREAKING**` in a release section.** A change that requires an adopter to *do* something —
including regenerating a recorded baseline — is marked, even when no public API, wire format, or identity
shape moves. Closing a false negative therefore counts: the reaction is additive, but the adopter's
baseline is not, and "the defect was ours" does not spare them the work. This is written down because the
0.4.0 window classified two same-shaped inbound-`Shallow` fixes two different ways before anyone compared
them.

## [Unreleased]

## [0.5.0] - 2026-08-22

### Documentation

- **A standalone-dimension adopter can now read a dimension's declared bounds through the contract that
  enumerates the promise.** `observation_bounds()` reached the public surface of `guibiao`, `hunyi` and `louke`
  in this window, advertised as the way to read what a dimension deliberately does not observe **without**
  implementing the protocol or composing a run — and no external compilation contract named it. Each dimension's
  `adopter_surface.rs` is a separate crate compiled against that surface, and each now calls it and uses what it
  returns, so the promise and the file that enumerates it are held together for this member as they already were
  for the protocol's other prelude members.

  Recorded because the cause is structural rather than an oversight: the three dimension contracts were tracked
  files that **no capability's declared subject claimed**, so the filing join that would otherwise have raised it
  never had them in scope. That was a declared false negative of the repository checks, with an owner, and it was
  the first time it was observed costing anything — filed in `BACKLOG.md` with its trigger rather than closed on
  the spot, since its stated reason (subjects need not tile the repository) had not moved. The trigger fired
  once, for this member, and the promoted `READY-PATCH` closed it: `adopter-surface`'s declared `## Subject`
  now names all four `adopter_surface.rs` files (the composed shell's alongside the three dimensions'), so a
  future change to any of the three no longer goes unfiled.

- **Every vocabulary type the observation protocol publishes was ruled on before it ships, by asking what a
  third party can own.** `0.5.0` is the first release in which an outside `Observer` can exist, so it is the
  last moment a vocabulary decision is free. The question applied to each: does an outsider occupy one of the
  values, or must they borrow one of this family's? `Owner` (`Engine` / `Inherited` / `Adopter`) names **roles**,
  so a third party's own engine is its `Engine`. `Extent` and `Reached` name positions any reaction can be in
  relative to its own observation; `Demonstrates` names directions any pinning test can take; `FactGranularity`
  names properties of a fact; `Defence` is their test or their tracker; and `BoundId::new` accepts any
  `Into<Cow<'static, str>>` so a computed id is expressible. All pass.

  `BoundaryKind` is the one that does not, and it stays as it is deliberately. `Crate` / `Module` / `Semantic` /
  `Runtime` name 三儀's own dimensions, so an outside participant labels its findings with a dimension that did
  not produce them — the example shipped beside the protocol reports `Module` and says so in a comment. **The
  correction that matters to an adopter:** the kind is the label a report and a SARIF render carry, and **not**
  a baseline. A recorded entry is `(target, rule_key, fact)` with no kind in it, and de-duplication is by that
  identity, so a borrowed kind misleads a consumer filtering by dimension without making anyone's recorded
  entries stale or forcing a regeneration. Earlier prose said a baseline carried it; the code says otherwise, and
  that half is what would have forced a decision before publication. Adding a variant later breaks no downstream
  match on a `#[non_exhaustive]` enum, so deferring is the reversible choice and a shipped variant is not.

- **The composed prelude promised the whole observation protocol, and the contract whose job is to name that
  promise named none of it.** `crates/tianheng/tests/adopter_surface.rs` is a separate crate that reaches every
  name through the same wildcard prelude an adopter uses, and its own header says it "deliberately names the
  whole promised surface" — but the protocol this window added reached the prelude without one member entering
  that contract: the `Observer` trait, `Run`, the three dimension observers, and the typed bound model
  (`BoundDecl`, `BoundId`, `Extent`, `Reached`, `Owner`, `Defence`, `Demonstrates`, `FactGranularity`). All of
  them are now named, in the form each kind admits, and the composition an outside participant writes — a run
  opened over a manifest, each dimension observer joining it, one verdict out — is type-checked through the
  wildcard prelude for the first time.

  Stated precisely, because a first draft of this entry overstated it: an external crate *had* compiled against
  part of the protocol. `examples/observer-participant` is its own workspace, depends on the shell, and reaches
  `Observer`, `Run`, `BoundDecl`, `BoundId`, `Extent`, `Reached` and `StaticObserver` — through a source patch,
  since the protocol is unpublished and no released consumer could. What had never happened is the thing the
  contract exists for: the promise growing and the contract that enumerates it standing still, with nothing
  holding the two together.

  **A trait cannot be named the way a type is**, which is why the contract gained a second helper rather than a
  longer list: `assert_public_type::<Observer>()` is `E0782: expected a type, found a trait`. Naming it in a
  bound proves more anyway — the trait is reachable *and* the observers re-exported beside it satisfy it, which
  is precisely what a third party writing their own participant needs to be true.

  Recorded because an adopter meets it: composing a run from a composed `Constitution` **clones per dimension**.
  Each observer's constructor takes its dimension's declarations by value while the `Constitution` lends them,
  so an outside caller owns a copy of each. That is the shape the shell's own documentation already prescribes;
  writing it into the external contract is what makes the cost visible from outside rather than only within.

- **A membership typed beside its enumerator went stale, and the sites that had not yet gone stale were the
  same shape.** `xuanji`'s `Reached::AsIntended` said *one* declared bound holds it while the extent projection
  rendered six, and the example it gave was the last of them; its sibling `NotAViolation` said *three* and was
  right only by luck of nothing having been added since. The rule against exactly this was already written on a
  third variant of the same enum — one rule, three places, two of them not following it — so it moves **up to
  the enum**, and every variant now documents the distinction that earns it a place instead. The distinction is
  the property a reader needs, and it is the one that does not move when the set does.

  Two internal doc comments carried the same shape and are repaired with it: one counted the boundary kinds a
  single rendering skeleton handles, the other the capabilities a single loop enumerates. In both the number was
  decoration and the point was the *oneness* — which is what they now say.

  `xuanji` is a published crate, so its variant documentation is adopter-facing. No signature, variant, outcome,
  identity, or exit class moves.

- Product vocabulary now follows the boundary already present in the manifests: **product** means publishable
  crates, **reaction** means their observable boundary behavior, Shengmo supplies dogfood gates that invoke
  those reactions, Kanhe supplies repository checks, and shell/CI only orchestrate them. The unpublished
  capability formerly called `rust-repository-reactions` is therefore `repository-checks`; its spec, bound ids,
  fixtures, and generated projections move together. No published API, outcome, report, exit class, manifest,
  or product law changes, so released adopters take no action.

- **`AGENTS.md` gains *A repair loop is a diagnosis, not a schedule*.** Three consecutive repair rounds on one
  text reader produced findings every time, and across all three **not one was a new code defect** — every
  finding was a sentence describing what the reader does, or a rule implemented twice. Each repair corrected
  the sentence review had named and wrote the next one. The rule says to sort a round's findings by kind before
  deciding what to do next: claims dominating means the property is stated where nothing can falsify it, and
  the answer is to change the shape rather than to add a round. It names the two moves that end those classes —
  a claim about a reaction becomes an executable case table, and one rule gets one implementation with
  exhaustive consumers — and it states outright that it has no reaction of its own, because deciding that a
  comment describes something a run could falsify is the prose judgement this repository measured and rejected
  three times.
- Giving each bound in `external-crate-confinement`'s overview its own `(bound: …)` reference created a hole,
  and a review proved it with a fixture. A `(bound: …)` reference was only ever resolved through a bound-prose
  record, and rewording that sentence dropped its line
  out of the scan's pattern — so the two references the repair added were **never resolved again**. Every
  reference is now resolved **wherever it sits**, independent of the wording around it: a Purpose paragraph, a
  requirement's prose, or inside a declared bound scenario. Every reference in the tree already resolved when
  the direction was adopted, so it is a free guard rather than a migration.
- `` `extern crate`-blind `` was an **undeclared and undefended** bound in the same capability, stated twice
  and invisible to the register both times — once with no trigger words, once as "a stated, inherited bound"
  whose comma breaks the adjacency the scan needs, which is the very wording the projection's first residual
  cites as its example. It is now a **declared bound** of that capability, pinned by
  `confine_ignores_an_extern_crate_declaration`: probed before declaring, and discriminating because the
  sibling test puts `use libc as c;` in the identical fixture shape and gets a violation. **Every bound the
  register holds is cited**, which the reaction requires rather than this note asserting a count.
- The projection's **content** is now asserted by the companion test, not only its freshness. Byte-for-byte
  staleness proves the document and the reaction agree; it can never prove either is right, because both come
  from one renderer. That is how `` `author\s:` `` — a shell quoting artifact where an apostrophe was meant —
  reached the tracked document and survived a review, alongside a sentence that did not parse. Both are fixed,
  and every disclosure the requirements oblige the header to make is now grepped literally, with a rendered
  backslash refused outright.
- A **retired bound was still alive** in `external-crate-confinement`'s overview paragraph: it listed
  `` `#[path]`-blind `` among the module scanner's inherited bounds while the resolution paragraph in the same
  file, corrected in this window, says a `#[path]`-remapped module is followed and its imports observed. One
  file said both, and the overview reads as permission — the failure this register exists to end, committed
  inside it. The stale name is deleted and each remaining bound in that sentence, and in the resolution
  paragraph, now carries its own `(bound: …)` reference.
- `docs/observation-bounds.md`'s header now enumerates **every** residual of the undeclared-prose direction
  and the one exemption, instead of stating the first and leaving the rest in the reaction's comments: a
  reference clears the prose it sits with regardless of how many bounds that prose states (the mechanism that
  let the bound above survive two sweeps), the scan is line-oriented, unrecognized wording is invisible, and
  prose under a requirement whose heading names bounds is exempt at the price of that requirement declaring at
  least one bound scenario. Scanning paragraphs instead of lines was measured against that defect and would
  not have caught it, so it is recorded as rejected rather than left to be re-proposed.
- Three measurements were stated in the present tense and had gone stale within this same unreleased window:
  the count of bounds sitting under the requirement they qualify, the number of specs carrying an
  Observation-bounds requirement, and a `CHANGELOG.md` entry whose bound count contradicted a sibling five
  lines away. Each is now number-free or reacted to rather than restated here.
- The register's shared-bound requirement claimed that one behaviour change cannot leave several specs
  stale. Its reaction reaches one shape — a single pinning test cited by two capabilities — so the claim
  is narrowed to that and the residual is stated in `docs/observation-bounds.md`'s header beside the
  prose floor already there: two declarations of one behaviour citing two *different* tests are invisible.
  Telling those apart from two genuine bounds over sibling shapes is a semantic judgment nothing can
  observe, so it is stated rather than declared as a bound — two operand dimensions here declare
  identically-worded bounds over `dyn` and `impl Trait`, each defended by its own test, and each must
  declare its own. The record of the two historical restatements is corrected in the same edit, since it
  was doing the overclaiming: the `#[path]`-remap bound was prose in one capability and a scenario in the
  other, so the undeclared-prose direction is what reached it. For a reader of the register the
  consequence is the point — it is a floor in two directions now, and says so.
- `observation-bound-register`'s `Purpose` was still the placeholder its archive step generated, the only
  specification that carried one.
- Two specs claimed that a `#[path]`-remapped module stays outside the scanner's observation, long after
  it stopped being true. `inline-symbol-path-confinement` carried it as a **declared bound** ("the system
  does not claim to observe it"), and `external-crate-confinement` listed it among the resolution's
  out-of-scope bounds. Measured on this tree, both forms react: a call inside a `#[path]`-remapped child
  of a clock-confined module is reported against that module, whether the attribute is written directly
  or wrapped in `cfg_attr`, and so is an import of a confined external crate. The scanner began following
  an unconditional remap in `0.2.2` and union-scanning a `cfg_attr`-wrapped one in `0.3.x`; the prose
  outlived the behaviour on both counts. Both claims are corrected and the behaviour is now pinned by
  `inline_path_remapped_child_is_observed` and
  `confine_observes_an_import_inside_a_path_remapped_module`. No reaction changed — only what the specs
  said about it, which for an adopter is the difference between reading a real escape as a defect and
  dismissing it as governed policy.

### Added

- `guibiao`, `hunyi`, and `louke` now expose the same observation-bound and observer vocabulary
  from each dimension root: `BoundDecl`, `BoundId`, `Defence`, `Demonstrates`, `Extent`,
  `FactGranularity`, `Observer`, `Owner`, and `Reached` — beside `Outcome`, which each root already
  re-exported. A standalone-dimension adopter
  no longer needs a direct `xuanji` dependency merely to name the protocol its dimension exposes.
- **A publish made through this repository's sanctioned path is refused at the source unless it runs on the
  signed, annotated-tagged `release: X.Y.Z` commit at the live tip of `main`** — refused before `cargo publish`
  is reached, rather than audited afterwards.
  `cargo publish` records the commit it ran on in each tarball's `.cargo_vcs_info.json`, a version can
  never be re-uploaded, and the `0.4.0` family recorded `f1dba52` — the tip of the since-archived
  `release/0.4.0` branch — rather than `e645a549`, the `release: 0.4.0` commit that `v0.4.0` tags.
  **The published content is unaffected**: the two trees are byte-identical, every shipped file in all
  six `0.4.0` tarballs matches `main`, and nothing needs re-fetching or re-resolving. Adopters
  verifying a `0.4.0` tarball against this repository should expect its recorded commit to name that
  release branch's tip rather than the tag.

  **The guarantee is over that path, and its edge is stated rather than implied.** A `cargo publish` invoked
  outside it meets no check at all; this repository declares that as an observation bound of its own, since
  reaching further would mean observing an operator's shell rather than this repository. Taking the sanctioned
  path stays a human's act, as an irreversible publish requires — what changed is that the path now refuses a
  wrong source instead of trusting the operator to have checked.
- `docs/history/published-artifact-provenance.md` inventories what commit **every** published version
  records, audited across all 96 published tarballs. Two versions disagree with their tag for two
  different reasons — `0.4.0` was published from the release branch, and `0.2.2` was published from
  `main` correctly and then force-pushed away an hour later — and neither disagreement affects any
  published crate's content. The eleven `0.1.x` releases point at commits the 2026-07-17 history
  rewrite dissolved, so whether they came from `main` is no longer knowable.
- An **observation participant** is now a public protocol: `Observer` (in `xuanji`, re-exported through
  the dimensions and `tianheng::prelude`) asks a participant to observe a workspace and to **declare what it does
  not observe**. Neither method has a default body, so a participant that says nothing about its limits cannot be
  written — and adding a stage later is deliberately a **breaking change**, because a declaration written before
  a question existed has not answered it. `tianheng::Run` composes participants with an **eager** fold:
  `Run::over(manifest).observe(a).observe(b).verdict()`, so no trait object appears in any signature and a
  participant composed onto an accumulator that already cannot judge is not evaluated at all. Assembly order is
  part of the contract — it decides which cannot-judge is reported. 圭表, 渾儀 and 漏刻 each implement it, so the
  protocol is dogfooded rather than offered: `StaticObserver`, `SemanticObserver` and `RuntimeObserver`, all three
  re-exported through `tianheng` and its prelude. `RuntimeObserver` is behind 漏刻's **`audit` feature** — the
  `tianheng` shell enables it, so it is always present there, and a crate depending on `louke` directly must ask
  for the feature. Each dimension also exports **`observation_bounds()`** as a plain library item — `guibiao`,
  `hunyi` and `louke` — so an adopter can read a dimension's declared bounds directly,
  without implementing the protocol or composing a run. That is what `observation-bound-model`'s requirement *A
  dimension SHALL export its declarations as library items* obliges, and 漏刻's set depends on its `audit` feature
  for the reason recorded below. **Nothing to migrate**: `check_constitution`, `run` and the CLI keep
  their exact path and behaviour, coverage included, and the protocol is an additional entry rather than a
  replacement.

- Every declared **observation bound** now carries a typed classification of *where the measure stops*:
  `xuanji::Extent` (re-exported through the dimensions and `tianheng::prelude`) says whether the observation
  source never reached the shape, or reached it and then refused to judge, deliberately declined to refuse,
  over-reacted, under-reacted, was correctly silent, or reacted exactly with only fact granularity bounded. An
  under-reaction — a declared false negative — must name who owns closing it: the engine, a named layer beneath
  it, or the adopter. `docs/observation-bound-extents.md` projects the result and leads with the count of
  declared false negatives and their owners, a figure that was previously spread across sixteen adjective
  phrasings and could not be counted at all. Read it beside `docs/observation-bounds.md`: one says what the
  specs declare, the other says what kind of stop each is. **Nothing to migrate** — no existing signature,
  `Constitution`, baseline format or report shape changed, and the new surface is opt-in to read.

  A declaration's strings are **owned-or-borrowed** (`Cow<'static, str>`), so a bound whose id, shape or rationale
  is computed can be declared at all: `Observer::bounds` carries no default body, and an implementor whose bounds
  are discovered rather than written — an observer over a plugin set, or over roots it scanned — was otherwise
  mandated to declare limits it had no way to name. A literal declaration is written exactly as before and still
  borrows, which is asserted by pointer identity rather than intended; every one of the family's own
  declarations is a literal, which a reaction now holds over all of them. Found in this window's pre-release review, and it earns no **BREAKING** mark for a
  measured reason rather than a remembered one: none of these types exists in `0.4.0`, so no adopter has the
  narrower form.

- `docs/observation-bounds.md` projects every **observation bound** the family declares — each claim that a
  reaction deliberately stops at a named shape — with the test that defends it or the tracker that owns
  closing the gap. Generated from the specs and staleness-checked, it leads with the count of bounds nothing
  yet defends rather than burying it. This entry deliberately carries **no** figure of its own: the register's
  census reaction greps every tracked Markdown file for `N bounds across M capabilities` and compares it with
  what the register holds **now**, with no exemption for a released section — so that phrasing, frozen into a
  dated entry, would fail the gate the first time the register grows. Read it before
  reporting a behaviour as a defect: a declared bound means the shape is governed policy, not an escape.
  Assembling it retired two bounds that had outlived their behaviour and added six tests for bounds nothing
  had defended. The list is a **floor rather than a proof**, and the projection's own header enumerates every
  residual: unrecognized wording, a line-oriented scan, a reference that clears prose stating more bounds than
  it names, and the exemption for requirements whose headings name bounds.
- The bound register refuses a **restatement**: one behaviour has one defence, so a test cited by declared
  bounds in more than one capability now fails, and the bound is declared once with the others referencing
  it. Two behaviours were declared three times each when the register first projected, which is why the
  register listed 41 bounds rather than the 44 it first projected — and a behaviour change can no longer leave
  several specs stale at once.
- `BoundDecl::borrows_every_string()` answers whether every string a declaration carries borrows rather than
  owning — the id, the shape, the pin, the extent's rationale, and an inherited owner's layer name, including
  the ones nested two levels down. The family's own declarations are literals and the specification said so,
  with **nothing measuring it**: every constructor accepts anything convertible, so a family declaration
  rewritten as `format!(…)` compiled, allocated on every pass of the register and the extent projection, and
  was named by no reaction. A reaction now holds it over all of them, and a `xuanji` test shows the answer can
  be `false` for a computed string in **each position independently** — a discriminant returning a constant
  `true` would satisfy every declaration, and one written as a short-circuiting chain can pass while examining
  only its first field. An adopter auditing whether a declaration owns any string value can ask the same
  question; it does not claim to measure allocations by non-string storage or the surrounding governance run.
  The deliberate counter-example lives outside the workspace: `examples/observer-participant`'s declarations
  are computed on purpose, which is what the owned form is for.

  Two scenarios of the protocol's own requirements gained the reactions they were written without, both found by
  the closing review's seventh round. *An observer's bounds method cannot be found where the reaction looks* is now
  asserted, with the discriminator that an **empty** body is found and judged — so absence and emptiness are
  distinguished rather than both reading as nothing to do. And *joining a run would require no new export* stops
  being a claim in prose: the example measures its own reach, one dependency and no import of a family crate past
  the shell, so a future edit importing around the public surface fails instead of quietly making the example
  prove the opposite of its point.

- `xingbiao::audit_corpus_and_anchor` derives a workspace's audit corpus **and** the directory every observed
  file's identity label is made relative to, in one call and from one `cargo metadata` read. 漏刻's observer and
  the composed shell both need that pair, and computing it twice is a twin derivation of **baseline identity** —
  the one thing 星表 exists to keep single, since a label that shifts makes an adopter's recorded baseline match
  nothing. Public because two crates share it and a crate cannot lend a private item; it takes the manifest path
  and returns the member root files with the resolved `workspace_root`, falling back to the manifest's own
  directory only for metadata carrying no such field — and, where even that resolves to nothing, **returning an
  error rather than an invented anchor**. That third state is the point of the error channel: the anchor *is*
  baseline identity, so a guessed one mislabels every observed file at once, silently, which is worse than
  declining to answer.

- `tianheng::testing::assert_projection_matches` is the bless-and-diff rule for **any** generated document, not
  only a rendered `Constitution`. A free function rather than a method, because blessing an unrelated document has
  nothing to do with a `Constitution` and requiring one would be a dependency invented by the API's shape.
  `GovernanceTest::assert_projection_fresh` delegates to it, so an adopter holding their own generated document —
  a register, a table, a report — gets the same `BLESS=1` behaviour and the same diff instead of writing a second
  copy of it.

### Changed

- **A Markdown boundary heading now names the boundary, not one field of it.** `tianheng list --format
  markdown` rendered each boundary as ``### `{target}` ``. For a crate boundary the target is the crate name
  and is unique; for a module, semantic, or runtime boundary it is the module path alone, and `.module("crate")`
  is the ordinary subtree-wide form — so every such boundary rendered the identical ``### `crate` ``, with the
  crate that told them apart three lines lower. In this repository's own projection that was 5 of 13 headings,
  four of them consecutive, three with bodies differing only by crate name. Headings are now
  ``### `{crate}::{module}` ({kind})``, and ``### `{crate}` ({kind})`` where a boundary has no crate field.

  Adopters rendering the Markdown projection will see every heading change, including crate boundaries, which
  gain their kind. That is deliberate rather than incidental: kind is rendered as **data**, never as a `match`
  arm, so a boundary kind added later is identified without the renderer being touched — the shape that
  produced this defect was a heading derived from a subset of what distinguishes a boundary. The subset is now
  as wide as construction can make it, and what construction cannot close — two boundaries alike in crate,
  module and kind but differing in rule — is held by `markdown_headings_are_pairwise_distinct`, over the
  rendered output rather than over an identity function, because it is the rendering that collapses.

- **BREAKING** — **`Outcome::Clean` now carries the subject it was measured over.** `Subject` says what the
  observation was asked to enforce, and how much of its own corpus it reached.
  `Subject::of` refuses the one combination that is a lie — *declared something, reached nothing* — so a
  participant can no longer report a sound workspace over an observation that never happened. Reaching nothing
  stays legitimate on its own: an empty semantic bundle is a static-only adoption, and
  `Subject::nothing_declared()` names that shape. Every other outcome already carried its evidence — a
  violation names eleven things about itself, a constitution error names a reason, a bound declaration names
  what a reaction does not see — and `Clean` was the one public value in the family that asserted the result
  of work while carrying none of it. It was also the missing dual of `Observer::bounds`, which has no default
  body precisely so a participant must declare what it does not observe. The text rendering and the JSON
  projection now carry the figures, so a reader — or an agent reading a CI log as context — can tell a
  workspace found sound from one that was never reached. **What to expect on upgrade**: every
  `Outcome::Clean` pattern stops compiling and gains a binding, which the Migration section below states in
  full. No exit code, report shape, or recorded baseline identity moves — this is a source-level break and
  nothing else.

- **The bounds-method reader's anchor rule is one typed decision, and its behaviour is a table that runs.**
  Two callers each re-derived the rule and drifted twice — once counting trimmed-start lines against
  occurrences, once when a line-start condition reached the reader and not its diagnostic. They match on one
  function's return now, so a fifth case forces every consumer to answer it or the build fails, and the doc
  comment that enumerated the decline classes is gone because the type holds them. Beside it, every shape the
  reader can meet is a row carrying the decision it makes — **including the shapes it gets wrong**, which is
  where `observer-protocol`'s declared bound over this reader is now read from. Writing the sentence three
  repair rounds kept writing ("a whole-line copy in a comment declines") as a row makes the row fail.
- **The shell's semantic arm now invokes `SemanticObserver` instead of calling 渾儀's composed entry point
  beside it.** What that buys is stated exactly, because review measured the looser claim false. It makes the
  two composition paths' **equality** for the semantic dimension construction-held — the built-in path obtains
  that outcome by invoking the dimension's observer, so there is no second call for a verdict to differ at, and
  the dimension joins runtime in the construction-held list. It does **not** close the delegation obligation:
  a guard deciding semantic emptiness above that call compiles, passes the whole suite and passes every gate,
  so `observer-protocol`'s bound on it stays declared and its `BACKLOG.md` entry stays open. The runtime arm's
  delegation was a different case and is not a precedent for this one — that arm held a second implementation
  of the corpus derivation, the audit call and the `cannot read workspace` message, which delegating collapsed
  into one, whereas the semantic arm always had one implementation with two callers.

  The requirement gains what keeps a construction-held dimension honest: the reaction SHALL still observe that
  the fixture's boundary for it **reacts at all**, so an arm that quietly went vacuous cannot leave the whole
  comparison resting on the dimensions that did not. Verified by perturbation — emptying the semantic
  observer's verdict fails that assertion by name. The **static** dimension is unaffected and its equality is
  still measured: the built-in path calls `check_and_cover`, whose coverage advisory the protocol cannot carry,
  while the observer calls `check`, and emptying *its* verdict fails the equality assert itself.

  No adopter action. `check_constitution` and `run` return what they returned; `SemanticObserver::observe` is
  `check_all` and always was, so the verdict for every declaration is unchanged.
- The `tianheng` shell no longer declares an unused direct normal dependency on `xingbiao`;
  its enforced self-law now limits the shell to the three dimensions it composes plus
  `serde_json`. Dimension crates continue to use the shared metadata substrate unchanged.
- `BoundDecl`'s new typed surface now carries a `Defence`: `PinnedBy { first, additional }`, with at
  least one pinning-test slot, or `Unpinned { tracker }`. Scenarios with several `PINNED-BY`
  citations retain every test instead of silently keeping only the last one.
  `BoundDecl::new` is replaced by `BoundDecl::pinned`, `BoundDecl::pinned_by_many`, and
  `BoundDecl::unpinned`, making the register's existing `PINNED-BY` / `UNPINNED` states mutually
  exclusive and both expressible in code. This refines an API added in this same unreleased window:
  `BoundDecl` does not exist in `0.4.0`, so an adopter upgrading from the shipped release has no former
  constructor call to migrate.
- **The census direction now judges tracked content and every figure on a line**, closing three ways the
  direction added one change earlier missed or overreached. It walked the filesystem, so an untracked scratch
  note and an ignored vendored tree each failed the reaction — a local file breaking a developer's run while a
  clean checkout passed, which is the checkout-dependence this family repairs wherever it appears; it now
  reads tracked Markdown through the same `git ls-files` every other direction here uses. Its matcher guarded
  the written number against a preceding digit, and that guard cannot match at a line's first column, so a
  **line-initial** figure was silently skipped while the identical figure mid-line was caught — and reflowed
  Markdown puts a number there routinely. A greedy match additionally examined only the **last** of two
  figures on one line, the same partial check the reference direction was already repaired for. A
  longest-match extraction replaces both, so a longer number is read whole rather than sliced into a false
  agreement, and the reaction is placed after the cannot-judge guard so a register it cannot read no longer
  reports every written census as disagreeing with `0 across 0` ahead of the one diagnosis that is true.
- The printed figure is **`pinning citations`, not `citations`**. This specification defines a citation as
  either form — `PINNED-BY` or `UNPINNED` — so an unqualified figure names two different numbers depending on
  which sense a reader carries, and that ambiguity is the actual cause of the four disagreeing counts the
  entry below records. A reaction emitting an unqualified figure would have become a fifth answer rather than
  the arbiter; labelled, the printed pinning-citation and unpinned counts sum to the requirement's sense with
  nothing left to infer.
- **A hand-written census of the register no longer lives in prose, and the one figure that must stay is now
  reacted to.** Adding a single declared bound invalidated fourteen written figures across four files —
  including the *Documentation* note whose own subject is that three measurements had gone stale in this same
  window.
  The cause is not carelessness: four independent, deliberate counts of this tree produced four different
  answers for the number of citations, and an ad-hoc count of unpinned bounds contradicted the generated
  projection. So a clean run now **prints** what it counted — the bound, capability, pinning-citation,
  unpinned, and reference figures, on one line after a clean verdict — five prose figures that carried no
  information are number-free, and a tracked Markdown document writing `N bounds across M capabilities`
  **fails** when either number disagrees with the count. The matched shape is deliberately narrow: a general
  number-in-prose matcher would refuse unrelated figures, which is how a gate earns the false positives that
  get it disabled. The printed figures are described rather than quoted here on purpose: a worked example
  carrying the reacted shape would itself become a second census to keep in step, which is the maintenance
  this entry removes.
- One matcher now decides the `(bound: …)` syntax. Two independently written ones did — an `awk` regex and a
  shell `grep -qE` whose whitespace classes differed — and clearing disagreeing with resolution about which
  references exist is the divergence that cost this window a review round.

- Every `(bound: …)` reference on a line is now resolved, not just one of them. The extraction was greedy, so
  only the **last** reference was ever examined and an earlier dangling one passed while the line reported
  clean. This does not close the residual noted above — that reference resolved — and is not offered as its fix.
- **The test harness now decides whether a bound-register citation names a test that runs**, replacing a
  source-text judgment that three reviews defeated. A `#[test]` neutralised by `#[cfg(any())]`, a `#[test] fn`
  inside an uninvoked `macro_rules!` body, and a definition inside a raw string or a block comment were all
  accepted; none of them registers a test. Enumeration is per package (`cargo test -p <pkg> -- --list`),
  because `--list` carries no crate label and this repository already has one test name registered in two
  crates — a workspace-wide match would let a citation qualified to a crate whose test was disabled be
  satisfied by the other crate's live test. The text scan keeps the two jobs the enumeration cannot do, the
  definition **site** and the duplicate direction, and remains a declared fallback where no manifest exists,
  printing which direction decided so a clean result names its own strength. An enumeration that cannot be
  produced at all is now *cannot judge* rather than a quiet fallback. **The register's two gate lines now
  depend on a built workspace** and are documented to run after `cargo test`; CI's step moved accordingly, so
  the enumeration is warm (≈1s) rather than a duplicate compile.
- The **third floor stated in `docs/observation-bounds.md` one release-window change ago is retired**: a
  `pinned by` line could be satisfied by a definition that never ran. It was declared a residual because the
  harness enumeration had been rejected twice on an unmeasured premise — that a throwaway fixture repository
  could not carry a manifest. Measured, such a crate enumerates cold in 107ms. The weakness survives only in
  the source-text fallback, which the spec describes, and `BACKLOG.md`'s entry about a shell-gated capability
  being unable to pin its own bound is demoted to WATCH with the measurement that dissolved its instance.
- A citation may name a **raw identifier** (`r#name`), which is a Rust identifier the register had refused; the
  contract is narrowed to ASCII identifiers with the reason stated rather than implied. A citation the harness
  registers but the definition scan cannot locate now reports the **line shape** the scan requires — `fn` and
  the name on one line — instead of reporting the test absent.
- A bound-register citation is now **validated before it is resolved**, closing two ways it could resolve to
  something other than the test it names. The cited name is interpolated into the search pattern, so a
  regular-expression metacharacter resolved a citation for a test that does not exist to a differently-named
  function — defeating the renamed-or-deleted direction the register exists for. The crate qualifier is
  joined to a filesystem path, so `../` resolved a citation against a function outside `crates/`. A name must
  now be a Rust identifier and a qualifier a crate-directory name, with at most one `::`; every citation in
  the tree already satisfies both. Validated rather than escaped, so a malformed citation is named as
  malformed instead of reported stale.
- Test recognition stops at a **block-comment delimiter**, so a `#[test]` written inside `/* … */` no longer
  satisfies the attribute run. It deliberately neither strips nor tracks comments: comment state is a
  forward property of a file that an upward walk cannot know, and stripping needs string-literal lexing —
  this tree's own lexer suites nest `/*` inside string literals, so a delimiter-counting scan would swallow
  real definitions. No `#[test]` run in the tree contains a block
  comment, so nothing existing is refused. The walk also lost its 12-line cap, which had refused a
  legitimate test whose attribute run was longer; the stop conditions were already the boundary the cap
  stood in for.
- `docs/observation-bounds.md` gains a **third floor**: a citation matches a line's form, not its comment
  state, so a function definition sitting inside a block comment satisfies one. Closing that needs the same
  Rust lexing the walk rejects, so it is stated where a register reader sees it and pinned by a fixture that
  records the accepted behaviour, and what blocks *declaring* it as a bound — `PINNED-BY` names a Rust test
  while this reaction's own defences are shell fixtures — is filed in `BACKLOG.md`.
- The bound register's own citations can no longer read as coverage while defending nothing. `PINNED-BY`
  resolved against **any** Rust function definition, so a production entry point of the right name
  satisfied it; it must now resolve to a **test**, read from the attribute run above the definition rather
  than the line before it. `UNPINNED` accepted any non-empty text as a tracker, so `no test exists`
  satisfied the very requirement forbidding it; a tracker must now name a path the repository tracks.
  Which section of that document owns the debt stays deliberately unchecked — that is prose, and a gate
  guessing at prose earns the false positives that get gates disabled. Every citation in
  this repository already satisfies both, so neither is a migration. Nothing about a pinning test's
  **name** is required, which is what keeps the suite's three naming variants working.
- Regenerating the projection (`BLESS=1`) now carries the same exit contract as judging it. It exited `0`
  after printing a register's offenses, reporting the family's "clean" over a register it had just
  refused; it writes the document and then fails, because "the document was rewritten" and "the register
  it describes is valid" are different claims. A register the gate cannot judge at all now fails *before*
  the write, so it leaves behind no `0 of 0` document reading as a complete register. CI always ran the
  judging line, so no violation ever passed this way — the misleading signal was local.
- What earns a minor is now stated once, in `AGENTS.md`'s *Versioning* section: any change an adopter
  has to act on, a stale recorded baseline included. Three documents had grown three answers — the rule
  above marks a false-negative closure `**BREAKING**`, `BACKLOG.md`'s version horizons listed such
  closures as patch-class, and this preamble called additive depth patch-class without distinguishing
  opt-in from default-on. For adopters the operative consequence: a false-negative closure or new
  default-on depth arrives as a **minor**, however small its diff, while packaging and hygiene, prose,
  opt-in depth, performance, and unchanged-outcome diagnostics arrive as patches.
- Every release tag is now a signed annotated tag. `v0.1.0` through `v0.1.10` were lightweight and
  unsigned; they were re-created in place on 2026-08-05, each still pointing at its own unchanged
  release commit, so the whole `v0.1.0`–`v0.4.0` range is uniform and GitHub-verified. A clone that
  already fetched the old refs needs `git fetch --tags --force` to see the new tag objects; no commit
  moved and no history was rewritten.
- The bound register resolves a `PINNED-BY` citation against **tracked** `.rs` files, not a filesystem
  walk. An untracked or ignored file decided a citation, so a scratch copy of a test file — the likeliest
  such artifact, tests being what citations name — resolved a name twice and the gate refused with
  "defined 2 times" locally while a clean checkout passed. Its cargo enumeration also keeps cargo's own
  stderr, so an exit-2 refusal names its cause instead of leaving a compile error, an absent package, and
  a held lock indistinguishable. A tracked spec absent from the worktree is now `cannot judge` **before**
  the projection is written: judging it already failed on projection staleness, but *blessing* rewrote the
  projection to describe a partial register and exited 0, leaving a document that reads as complete.
- The bound register refuses a **failed tracked-file enumeration** instead of reading it as an empty
  repository. Its four `git ls-files` enumerations were each consumed by `mapfile` from a process
  substitution, whose exit status reaches no one — `pipefail` does not follow into a subshell nobody
  reads — so a failed enumeration returned exactly what a repository holding nothing returns. Three
  wrong answers followed from that one reading: the census direction examined no document and reported
  **clean** over a stale figure, while the tracker and citation directions refused **every** bound in the
  register, blaming it for a `git` failure that was not the register's. One enumerator now checks the
  status where the reaction can act on it, buffering the NUL stream in a trap-owned file because command
  substitution strips NUL bytes and would have defeated the `-z` silently. The requirement states the
  direction, and states the absent-tracked-spec refusal alongside it — a reaction refusing where the
  declared law names no refusal is the inverse of the drift this capability ends. The **citation** path
  reached that enumerator through a function the caller consumed with its own process substitution, so the
  refusal exited that subshell and the parent still read an empty site list; the scan runs in the parent
  now. `grep`'s read failure is separated from its no-match exit for the same reason — a citation reported
  as defined nowhere because a file could not be opened is a violation invented from an IO failure. The
  remaining reads of the observation source are checked with them — the harness listing's parse, the
  attribute-run read, and both census `grep`s, the last of which discarded `grep`'s exit **>1** behind
  `|| true`, so a tracked document the census direction claims to cover went unexamined behind a clean
  report. What is left in a process substitution is computation over data this run already materialized, and
  the reaction says which rather than leaving the scope to be inferred.
- **The feature-gated export now has a reaction, and `cargo test -p louke` joins the Definition of Done.**
  `observation-bound-model`'s scenario *A build compiles none of the reaction a declaration describes* was added
  with none: the gating had been proved once by a throwaway crate reading `observation_bounds().len()`, and that
  crate was deleted, so the property rested on nobody changing the `cfg`. Each configuration now asserts **its
  own** answer — verified by removing the gate, which fails the audit-OFF direction naming every bound that leaked
  — and the audit-OFF library is *tested* rather than only linted, because the clippy pass beside it catches an
  unused item and never a declaration that should have been compiled out. Every `--workspace` or `--all-features`
  run has `audit` **on**, so this is the only place that answer is observed at all.

- CI checks out with **`actions/checkout@v5`** in all eight jobs. GitHub reports `@v4` as targeting the deprecated
  Node.js 20 and is already forcing it onto Node 24, so nothing was broken — the bump is ahead of the date that
  forcing stops. Verified as far as it can be locally: the eight occurrences are the only `checkout` references, the
  workflow still parses to its eight jobs, `release-coherence`'s `fetch-depth: 0` is untouched (a shallow checkout
  cannot see the release spine, and `v5` keeps that input), and the runner in use is well past `v5`'s minimum. Its
  real verification is a CI run. `EmbarkStudios/cargo-deny-action@v2` is deliberately unchanged — GitHub's warning
  does not name it, and the DoD-coherence gate's exemption comment cites that version.

- **漏刻's audit-scoped bound declarations are gated with the reaction they describe.** `mod bounds` and its
  `pub use observation_bounds` were unconditional while `mod observer` immediately beneath them is behind the
  non-default `audit` feature, with a comment explaining exactly why — reasoning that applies to `bounds` and had
  not been applied to it. Every declaration but one describes `audit_probe_coverage`, the scanner an audit-OFF
  build compiles none of — among them two declared false negatives owned by the engine — while the composite-shape
  bound describes the always-present origin derivation on the hot path. So an audit-OFF dependent read **six**
  declared bounds for a reaction its build did not contain, and the accessor's own doc comment was false in that
  configuration. **No adopter reads six**: `louke::observation_bounds` is absent at `v0.4.0`, so the six were an intra-window
  state and this is a correction rather than a change anyone upgrades through. Under the non-default feature an
  audit-OFF dependent now reads one, and the
  `tianheng` shell enables `audit`, so nothing changes for the composed entry. The defect arrived through the
  *export* rather than through a declaration, which is why no reaction saw it — the bijection runs with
  `--all-features` — and `observation-bound-model` now carries the rule: where a reaction is behind a Cargo
  feature, the declarations describing it are gated with it.

- The bound register's **exit contract now binds every path**, not the ones a wrapper remembered. `set -e`
  with `pipefail` carries a failing utility's own status out of the process, so a stubbed `sed` made the gate
  exit **4** printing nothing at all — a status the contract does not define, which no consumer can act on
  and no operator can read. An `ERR` trap maps any unhandled failure to `cannot judge` with the line that
  failed; it reports *where*, never what, and a failure worth naming keeps its own refusal beneath it (the
  spec read now names the spec). Its interaction with this gate's many deliberate non-zero returns was
  measured before being adopted: under `errtrace` a failure inside `if`, `||`, `&&`, an arithmetic guard, or
  a captured pipeline with its own handler does not fire it, even inside a function.
- The register's **package enumeration comes from tracked manifests** rather than a `find` walk whose
  pipeline status the parent never saw. The previous guard caught only a *totally* empty result, so an
  enumeration that emitted some entries and then failed left a short list reading as authoritative:
  measured on this repository, `find` printing one directory and exiting 3 produced `24 registered test
  names across 1 package(s)` and a cascade of false violations against citations in the five packages it
  never enumerated. A directory with no tracked `Cargo.toml` is not a package, which `cargo test -p` would
  have discovered one step later.
- The reference-integrity gate refuses a **failed extraction** instead of reporting clean. The per-file
  normalization ran inside a process substitution, where a failing `sed` or `sort` reports nothing to the
  parent — `pipefail` does not reach a subshell whose status no one reads — so the stream came back empty,
  every reference in that file went unexamined, and the file still counted as inspected. The gate had
  already been repaired for this exact shape one step earlier, where `grep`'s status is captured so an
  unreadable tracked file refuses to judge.
- **BREAKING** — an operand-scoped `dyn` or `impl Trait` boundary now reacts to a **bare** principal trait
  the governed module itself declares. In `0.4.0` a bare single-segment principal did not resolve at all, so
  `pub trait Port {}` beside `Box<dyn Port>` in the same module was passed over even under
  `must_not_expose_dyn_of(["crate::m::Port"])` — a **false negative**, now closed. The name is canonicalized
  before it is matched, so a local `pub trait r#type` reacts against the canonical `crate::m::type` an
  adopter writes. **What to expect on upgrade**: new findings wherever a governed module declares a trait,
  uses it bare in a public `dyn`/`impl Trait` position, and your boundary forbids that module-qualified
  path — so a recorded baseline needs regenerating (`--write-baseline`), which is why this is marked.
  A bare name the module does **not** declare — a prelude trait such as `Iterator` or `Fn`, a
  glob-imported trait, a name the file never mentions — is still dropped: that resolver-coverage bound is
  unchanged, and both capabilities' bound scenarios and pinning tests now state and observe it against the
  module-qualified spelling rather than a bare one that no ladder step can produce. Only the two
  operand-scoped rules are affected; signature-coupling, forbidden-marker, trait-impl locality, and unsafe
  confinement resolve through a different entry point and are unchanged.

  *Two intra-window states are deliberately not described as adopter effects, because neither shipped: the
  first cut of this fallback resolved every bare segment into the module (a false positive over a fabricated
  operand) and left a raw identifier unmatched. `BACKLOG.md`'s closed entry carries that history, where it
  explains the shape; a release note written from an unreleased baseline would describe an upgrade nobody
  performs.*

### Migration
- **Match `Outcome::Clean(_)` rather than `Outcome::Clean`.** The compiler names every site; the pattern gains
  a binding and nothing else. Where a test asserted equality against `Outcome::Clean`, assert
  `matches!(outcome, Outcome::Clean(_))` instead — pinning a specific subject asserts more than the test
  meant. An `Observer` implementation returns `Outcome::Clean(subject)`, building it from the two figures it
  already holds: `Subject::of(declared, reached)` where `declared` is what it was asked to enforce and
  `reached` is how much of its corpus it opened, or `Subject::nothing_declared()` where it was configured with
  nothing to enforce. `Subject::of` returning `None` is not a clean workspace — it is boundaries declared over
  a corpus that was never reached, which is a constitution error.
- **Regenerate any recorded baseline.** The operand-scoped `dyn`/`impl Trait` boundaries now resolve a **bare**
  principal trait the governed module itself declares, which closes a false negative and therefore adds
  findings: run `tianheng check --write-baseline <file>` wherever a baseline is kept, and re-apply any
  `owner` / `tracker` annotations onto the newly observed facts. Only those two rules are affected —
  signature-coupling, forbidden-marker, trait-impl locality, and unsafe confinement resolve through a different
  entry point and are unchanged.
- **Nothing else requires action**, including the one *published crate's* manifest that changed: `crates/tianheng` dropped its
  direct `xingbiao` dependency, and `tianheng` re-exported nothing from `xingbiao` at `v0.4.0`, so no path an
  adopter could name is gone. Every other entry either adds a surface absent at `v0.4.0` (the observation-bound
  vocabulary, the three concrete observers, `Run`, each dimension's `observation_bounds()`), corrects a
  repository-internal reaction that ships in no crate, or is documentation. No feature, `rust-version`, or
  dependency **requirement** moves.

### Fixed

- **`list`'s check-only-flag rejection was a hand-written array beside `ParsedArgs`, not derived from
  it.** A new check-only field added to `ParsedArgs` without a matching update to `dispatch_list`'s
  array would reach `list` unrejected instead of failing loud — measured directly: an added
  `strict_mode: bool` field, wired everywhere except this one array, compiled clean and `list
  --strict-mode` exited `0`. `dispatch_list` now exhaustively destructures `ParsedArgs` (no `..`), so
  a field added without an arm here fails to **compile**, naming the missing field, instead of
  silently reaching `list`. Only the failure mode for a field not yet added changes; every flag `list`
  rejects today is unchanged.

- **Composing two clean verdicts can no longer wrap into a clean verdict nobody reached.** The observer fold
  summed each participant's `Subject` with unchecked `usize` addition. `Subject::of` admits any pair where
  something declared also reached something, so `usize::MAX` declared is a value a participant can hand the
  fold through the published surface alone — and two of those overflow. Measured on the code this replaces,
  both ways: a debug build reported `attempt to add with overflow`, a release build returned
  `Clean(Subject { declared: 18446744073709551614, reached: 2 })`. The wrap is the dangerous half, because the
  wrapped total still satisfies `Subject::of` and the run states a clean verdict carrying a figure that is the
  sum of nothing.

  Both figures are now checked, and a sum that cannot be represented is an `Outcome::ConstitutionError` — exit
  class 2, because an aggregate this fold cannot state is not a boundary anything violated. Only inputs that
  previously panicked or wrapped behave differently, so nothing an adopter could rely on has moved.

- **A field added to `Reached::AsIntended` would have escaped `borrows_every_string` silently.** That measure
  reaches every string a declaration carries, and its own documentation rested the guarantee on in-crate
  exhaustiveness — *"a variant added with a new string of its own fails to compile here rather than being
  silently unmeasured"*. True for a **variant**; the one arm carrying a second field elided it as `..`, and a
  **field** is not a variant. Measured both ways: with `..`, adding a string-carrying field to that variant
  compiles clean and the string is never reached; naming it `bounded: _` makes the same addition
  `error[E0027]: pattern does not mention field`.

  Nothing observable moves — `bounded` is a `Copy` enum carrying no string, so every answer this measure gives
  is the answer it gave before. What changes is that the next field cannot be added without answering for it,
  which is the guarantee the documentation already claimed. The doc now separates the two obligations rather
  than letting the variant case stand for both.

- **A dimension's dependency allowlist could name a sibling and nothing reacted** — 三儀 ⊥ 三儀 quoted in the
  `because` while the allowlist beneath it permitted the opposite. Reproduced: widening `guibiao`'s allowlist to
  name `hunyi` left **every** test binary in this workspace green, and `AGENTS.self-law.md` regenerated to print
  `only: serde_json, xuanji, xingbiao, hunyi` directly beneath the reason that forbids it. Neither reaction a
  reader would expect to catch it can: the staleness check pins the projection against the *declaration*, so a
  blessed projection of a widened allowlist is **fresh** — and freshness is not truth — while the dependency
  reaction cannot fire on a *widened* allowlist, because permitting more than the tree uses produces no
  violation. The reaction now asserts membership beside the clause, names the sibling it found, and
  `self-law-projection` states why that assertion is the sole guard. The clause check's own limit is unchanged
  and still limited — and every limit of this reaction is now **declared**, four of them, each extent read off a
  run of that limit's own WHEN. The two wording limits go in **opposite directions**: paraphrasing the clause
  makes the reaction *fire* (over-reacting, the safe direction), while a `because` carrying the literal clause
  and then *negating* the law **passes**, so `AGENTS.self-law.md` can project the law's opposite to every agent
  that loads it — the serious one. The two enumeration limits are the reaction's hand-kept dimension list, which
  the set-coverage assertion structurally cannot hold because the set it compares is produced by filtering on
  that same list, and a rule variant the filter never reaches although it governs workspace-member edges
  specifically.

  All four are unpinned against one `BACKLOG.md` tracker. Two drafts got this wrong in opposite ways: one
  declared the first as a false *negative*, which a single run of its WHEN falsified, and one then deferred all
  four on the ground that declaring needed a pin. It does not — a bound may be declared unpinned — and
  withholding them kept a measured false negative out of the register a reader is told to consult before calling
  a behaviour a defect. Pinning is what needs the reaction run over a supplied declaration; the tracker owns
  that.

  Found by three withdrawn attempts at one `PROJECT.md` paragraph, the third of which tried to stop restating
  the law and **cite** the generated projection instead. Review showed the citation bought freshness rather than
  truth. The paragraph kept coming out wrong because the law it described was only half reacted to.
- **The source-shape reaction over the shell's composition body is retired, and the obligation it claimed is
  now a declared, unpinned bound.** It read the characters of one function body while the requirement is about
  what the shell *does*. Four review rounds narrowed it and each narrowing was defeated: by name resolution (a
  `use` shadowing 渾儀's entry point, the body byte-identical), by the binding site (the parameter renamed, or a
  second one added — the parameter list sits outside the read extent), by which definition is the subject (a raw
  identifier, leaving a commented copy as the only signature occurrence), by the caller frame (the guard moved
  into `check_constitution`), and by **execution** — a delegation bound to `let _`, written inside a
  never-invoked `macro_rules!`, or placed in a conditionally-called closure satisfies every textual rule while
  the shell decides for itself. That last group is the one no widening reaches, because it is not a property of
  text. Every defeat was measured end-to-end against the tracked composition function with the suite green and
  the formatter and linter silent.
  Along the way it closed real things — seven spellings of a reach for the constitution, a decoy anchor, a
  comment read as code — and cost two false-positive classes, one of them fired by `rustfmt` reformatting a
  conforming body. Retiring it rather than narrowing a fifth time is the honest disposition: what a text reader
  can still say truthfully is that the body reaches its constitution only through the declared accessors, and
  that is not the obligation. `observer-protocol` now declares the gap as a false negative this repository owns,
  **unpinned**, tracked for closure by construction. Which construction is deliberately not named here: the
  route this entry first pointed at — the one the runtime dimension's equality takes — was tried later in this
  same window and measured to close nothing, because that arm had a second implementation to collapse and this
  one never did. `BACKLOG.md` carries what would actually close it.
  The register leads with the count of bounds nothing yet defends, so the debt is visible rather than implied.
- The same decoy defeated the **bounds-method** reader, whose bound records the moved extent as *over-reacting*
  and therefore safe. A commented-out conforming copy above a divergent `bounds()` made the exact one-statement
  equality pass on text that was not the method — the safe direction inverted by the anchor rather than by an
  in-body brace. Both readers used the recognizer, so the repair covered both — and the delegation reader was
  retired in this same window, so what remains is the bounds-method reader and the one decoy pin that defends it.
  The pin sits with that reader rather than with the recognizer because the claim being defended is the reader's
  own error direction.
- **A bound was typed for the harmless one of its two readers.**
  `a-brace-inside-a-block-comment-or-a-string-literal-moves-the-read-body-extent` recorded the moved extent as
  *over-reacting*, read off the bounds-method comparison, where an exact one-statement equality cannot survive a
  moved extent and so refuses a conforming body. The shell-delegation reaction arrived later with a comparison
  that survives one intact, and the rationale was never re-derived against it — leaving a bound on the safe side
  of the false-negative line describing a reader on the unsafe side, which reads as permission. The
  over-reaction survives on the bounds-method comparison, which is the reader it was read off; the second reader
  was retired rather than given a bound of its own (see the retirement entry in this section). No adopter action:
  `tianheng::observation_bounds()` is absent at `v0.4.0`, so no identity here has ever shipped.
- `docs/observation-bound-extents.md` asserted that **refuses to judge** carried no bound while rendering one
  under that heading two sections below. The claim was a literal in the generator's template, which is the one
  place a freshness check cannot catch a falsehood — the comparison is the generator's own text against itself.
  The lesson it carried does not need the membership claim, so the claim is gone rather than re-typed — which
  is what kept the document honest when the retirement later removed that value's only instance.
- **A `~~~` fenced block counted as prose, so a path inside one satisfied "reachable from where a reader is
  sent".** `Prose` recognized only the backtick fence, and `projection-register` requires every generated
  document's path to appear in `AGENTS.md` prose — a fence is where a command lives, not where a reader is sent.
  Markdown fences with either character, so the tilde form was read as ordinary text. A fence now opens on a run
  of three or more backticks **or** tildes and closes only on a run of the same character, at least as long.
  A closing fence also carries no info string, so a run followed by text is content: without that, an inner
  `` ```rust `` closed the block — its contents counted as prose — and the bare run beneath re-opened a fence that
  never closed, excluding everything after it. One construct erring in both directions at once.
  The obvious repair — toggle on either marker — reopens the hole from the other side, letting a `~~~` displayed
  inside a backtick block close it and turning the rest of that block into prose; that is measured and pinned,
  and it matters here because `AGENTS.md` is documentation about documentation. Latent rather than live: no tracked Markdown has a line that *opens* a `~~~`
  fence — this entry mentions the delimiter, which is not the same thing — and that is the state in which the
  hole is cheapest to close and least likely to be noticed. What block structure is not modelled is now recorded by
  direction rather than lumped together: an unpaired indented fence, a leading inline code span, and a fence
  inside an open HTML comment span over-exclude; a fence opened on a blockquote or list-marker line
  under-excludes. Handling the blockquote form by stripping its prefix was tried and reverted — the strip cannot
  know whether a fence is already open, so a quoted run displayed *inside* a fence closed it and a path shown in
  a Markdown sample became prose, a worse instance of the fault being fixed.
- **Three text recognizers read past the region they observe, each in the direction its own comment said it did
  not.** One class, found by a review of this window and repaired together. The bound register's attribute walk
  stops at a blank line so an attribute cannot cover the item beneath it — but it read the preceding lines
  through a command substitution, which strips every trailing newline, so the blank *directly* above a
  definition was deleted before the walk began: `#[test]`, blank, `pub fn cited()` was accepted, and the gate
  reported clean over a citation defending nothing. The observer-protocol delegation reaction counted braces
  through comments, so `observation_bounds(); // }` closed the body at the comment and a second list beneath it
  was never presented to the comparison — the one thing that reaction exists to refuse, passing as the
  delegation; the comment tail is now removed **before** the braces are counted. And the projection register's
  prose reader dropped a whole line carrying an HTML comment, where the requirement is that a path appearing
  *only* inside one is not a mention — so a path a reader plainly sees was discarded and a conforming document
  refused. The comment **span** is now excised and the visible text kept. Each is pinned by the direction that
  fails without it, and the first two were confirmed against the unrepaired code. Adopter-facing effect: none —
  all three are this repository's own governance reactions, and what they change is an additive read-only
  surface no baseline or verdict depends on.
- What the brace count still cannot separate is now a **declared bound** of `observer-protocol` rather than an
  unwritten limit: a `{` or `}` inside a block comment or a string literal still moves the body extent, because
  telling one from the other needs the string-literal lexing this tree defeats, its own lexer suites putting
  comment delimiters inside string literals. It is declared with the direction measured rather than assumed:
  for the bounds-method reader's exact one-statement comparison, no brace-carrying construct survives, so a
  moved extent refuses a **conforming** body instead of accepting a divergent one — an over-reaction an author
  argues with and never a silent pass. Its pin carries that control. (Later in this window that direction was
  found to belong to the *comparison* rather than to the extent, and the bound split accordingly; see the
  entries above.)
- The publish-source gate now owns cleanup before acquiring its temporary signature workspace, so a partial
  acquisition that creates a directory and then fails cannot leave that directory behind.
- The publish-source gate now reports cannot-judge when Git's extracted signature is not the tag object's exact
  suffix, instead of silently verifying an unreconstructed payload and misreporting the tag as wrong source.
- The reference-integrity gate's required governance-document set is now immune to ambient environment overrides;
  its narrowed zero-corpus policy is an explicit, validated fixture-only argument that cannot target this workspace.
- The observation-bound register now reports cannot-judge when its target repository becomes unavailable before
  the written-census scan, instead of collapsing `cd` failure into grep's ordinary no-match exit and reporting 1.
- The observer-protocol trait-object guard now scans Tianheng's Rust sources recursively, so moving a public item
  into a private nested module cannot remove it from the corpus when a public re-export could still expose it.
- Repository governance reactions now select Rust or shell executed-source regions explicitly, so Rust
  attributes are not discarded as shell comments and shell recognizers do not inherit Rust comment semantics.
- Empty semantic composition now returns `Clean` in `hunyi::check_all` before reading workspace metadata, so
  `SemanticObserver` and the shell delegate empty and non-empty bundles to the same behavior owner.
- **The pre-publish gate now verifies the release tag's signature instead of matching its shape.** It grepped the
  whole tag object — message included — so an annotated-but-unsigned tag whose message quoted a
  `-----BEGIN SSH SIGNATURE-----` block, a pasted verification log, satisfied it. `cargo publish` stamps a
  permanent, non-re-uploadable commit pointer and this is the last automated check before it, so an adopter
  auditing a tarball's provenance is the reader who cares. Verification now runs through
  `ssh-keygen -Y check-novalidate`, which needs no configuration; `git verify-tag` was measured and cannot be
  used, reporting an identical `allowedSignersFile needs to be configured` for a genuinely signed tag and an
  unsigned one alike. A signature the gate cannot read, and a tag-object read failure, are both cannot-judge
  rather than a wrong source.

  The gate's own stated bound had the cause **backwards** — it said verification needs an allowed-signers file, so
  the gate could only match a shape. Verification does not; **attribution** does. That is corrected, and what
  remains is a declared observation bound: the gate does not judge whether the signer is authorized, owned by the
  verification environment rather than by this family, because giving CI an allowed-signers file is what would
  close it. `publish-source-integrity` is the new capability that finally states the gate's contract — until now
  it lived only in the script's own header, in no specification at all.

- **`list`'s refusal now names the check-only flag that was supplied**, instead of one sentence naming none of
  them. Every other refusal on this surface already named what the invocation did — all twenty malformed
  value-flag cells, and the `--format` *value* refusal which names the value and explains why `sarif` is not a
  `list` format — so this was the one place the surface moved the search onto the reader. It mattered most for
  `--manifest-path`, which is in the rejected set and is the flag typed by habit: told only that "list takes only
  `--format`", someone who passed both was being shown the flag they got right. Every supplied flag is named, not
  the first. No invocation's verdict changes: what was refused is still refused, with the same exit `2`.

  Found by sweeping the CLI surface against an enumeration rather than against invented shapes, and the defect was
  in the **requirements**, not the code: `list`'s own requirement asked only for usage guidance and exit 2, while
  the requirement covering the same conflict inside `check` cites it as "the rule the `list` requirement above
  states" and requires the flag to be named. Each implementation satisfied its own. The same sweep found that
  requirement enumerating four check-only flags while the runner rejected five, so the set is now derived rather
  than listed in prose.

- **`observer-protocol`'s equality reaction now covers all three dimensions, and two assertions that could not
  fail were replaced by ones that can.** The reaction promises the trait-driven fold and the built-in path cannot
  disagree silently; it proved that for one dimension. Its fixture declared only a violated static boundary, and
  an empty declaration is clean on this workspace, so the semantic and runtime arms compared clean against clean
  — measured, replacing `SemanticObserver::observe`'s body with `Clean` left the suite passing. The fixture now
  violates a boundary in **each** dimension and the reaction asserts each one reacted, so a fixture that goes
  vacuous because the workspace changed under it fails naming the dimension to repair rather than quietly proving
  less. Verified by short-circuiting each of the three observers in turn.

  The bound-set half was worse: it asserted `observer.bounds() == dimension::observation_bounds()` while every
  `bounds()` **is** that call, so it compared a function with itself — drifting a declaration's extent with its id
  untouched left it passing. Nothing was unguarded (that drift fails the extent projection, which is where the
  content is held), but an assertion that cannot fail reads as a guarantee. What the requirement refuses is a
  *second, divergent list*, which is something written in a body, so the reaction is now over each `bounds()`
  body's shape — exactly the delegation, recognized by position within the method — and it fails when a body
  holds a list of its own.

  Behind both: `check_constitution`'s runtime arm was a hand-copied twin of `RuntimeObserver::observe`, down to a
  duplicated `cannot read workspace` message, so equality for that dimension rested on nobody editing one of the
  copies. It now delegates. **Nothing to migrate** — no public API moves and both paths keep their exact
  behaviour, which is what the reaction proves.

- **`Observer::bounds` has a consumer, and the protocol has a third-party implementor.** The method is required
  precisely so a participant cannot join a run without declaring what it does not observe — and nothing read the
  answer: measured, no call site anywhere outside a comment, so a dimension could have answered anything without
  moving a verdict. Three implementors and a register full of classified bounds made it look answered, because it
  reached those through each dimension's free function while the trait method was a parallel door nobody walked
  through. The bijection now reads each dimension **through `Observer::bounds`**; returning the wrong set from one
  fails it, naming every id left unclassified. The shell's own declarations keep coming from its free function,
  because the shell composes dimensions rather than being one.

  Nothing in the repository had ever been a **third party** to the protocol either — all three implementors are
  family crates returning literal lists. `examples/observer-participant` is a crate outside the family that
  implements `Observer` and joins a composed run: its house rule (every module file opens with a `//!` header) is
  one no dimension of 三儀 has a DSL for, its violations carry its own structured identity, a subtree it was told
  to read and could not is exit `2`, and its bounds are **computed** — id, shape, reason and pin all built with
  `format!` from what it was configured to read. That is the first caller of `BoundId`'s owned-or-borrowed form
  that is not a literal; it shipped in this window for a caller that did not yet exist. The example needed **no
  addition to any crate's public API**, which is the load-bearing result: the public surface is enough to join
  a run. A `COOKBOOK.md` entry shows the same shape at teaching size.

  What it found, recorded in `BACKLOG.md` rather than worked around: `BoundaryKind` has no value a participant
  owns, so an outsider's violation must claim one of the family's four kinds.

  The participant declares **two extents**, not only two bounds: a shape it never reads (it lists a subtree one
  level deep and never descends) and a shape it reads and judges **too harshly** — its rule tests a file's first
  line, so a real module header below a licence comment reads as absent though a reader of that file learns
  exactly what the rule says it should. The over-reaction was found by the window's closing review, in the one
  artefact whose whole subject is declaring what you do not observe, and it is declared rather than closed:
  skipping a leading comment block would trade the edge for a block comment and an inner attribute, and would
  leave the rule's wording saying something other than what the code does.

- **`Polarity` now says when a violation carries none.** It is an `Option` on `Violation`, and nothing stated the
  rule, so a reader — this window's own review included — reads the absence as a rule kind missing its repair
  direction. Measured across every production emission site: 圭表's crate and module rules answer through
  **exhaustive matches returning `Polarity`** and 渾儀 emits every finding through a context carrying a
  **non-optional** one, so in those dimensions a new rule variant cannot compile without declaring a direction —
  a stronger guard than a reaction, and one a reaction could only duplicate and disagree with. The single path
  carrying `None` is 漏刻's **probe audit**, where it is correct: a declared seam with no probe is repaired by
  probing it *or* by dropping the declaration, which is neither a deny breach nor an allowlist gap. Documentation
  and a `runtime-origin-assertion` requirement — **and a reaction**, because the requirement was first written with
  none: the closing review's sixth round found it, in the window whose whole subject was closing that class. Both
  observable directions are now asserted on the one fixture that produces them together, so neither is checked on a
  report that lacks it, and attaching a polarity to the audit's violation builder fails it naming both findings. The
  by-construction half stays unreacted deliberately: an exhaustive match is the stronger guard, and a second copy of
  a fact the compiler holds can disagree with it.

- **The lexical trait-object guard states where it stops, and checks the premise it rested on.** The reaction
  keeping the composed shell free of trait objects has to be lexical — 渾儀 governs no module of `tianheng`, and
  the `dyn` DSL has no allow-except form — but it read only top-level `src/*.rs` without saying so. Every file
  under `src/runner/` went unopened, and an injected `pub fn … -> Option<Box<dyn Debug>>` among them left it
  passing. Not an exposure, because those modules are private — but the soundness rested on an **unchecked
  premise**, and `pub mod runner;` would have removed every file beneath it from the reaction's reach in
  silence. That
  premise is now asserted, and making the module public fails the reaction telling you to recurse. What genuinely
  remains is declared as a bound in the register: the recognizer is handed one line at a time, so a trait object
  on a wrapped signature's continuation line is text it is never presented with. Pinned by a test that feeds the
  recognizer the one-line control and the wrapped form.

- **A declared bound was narrowed after its stated cause was found wrong about its own history.**
  `gate-shape-contract`'s `1-versus-2` bound said the judgment it declines to make is what let a
  `return`-instead-of-`exit` inversion ride green. Read back against that inversion, it produced **both** of the
  bound's directions in one gate — every refusal was `1`, so a shallow clone reported *"the release surfaces
  disagree"*, while the exit-contract backstop turned every genuine incoherence into `2` — but what let it pass CI
  was the matrix asserting a **non-zero status rather than a code**, in that commit's own words. That mechanism is
  closed: the `exit codes` property requires the exact code from every twin and cites this very instance. So the
  residual is only the semantic judgment, and the bound now says so at all **three** sites it was written — the
  spec's THEN clause, the typed declaration's rationale, and the projections derived from them. A bound reads as
  *permission*, so one that overstates what is unobserved misleads exactly as much as one that understates it.

- **A latent under-reaction in the process-substitution property is declared.** `gate-shape-contract` permits a
  producer that is a shell builtin over data already in memory, on the stated reason that it has *no I/O to fail
  at* — and the recognizer applies that permission by reading the producer's **first word**. So
  `< <(printf '%s\n' "$rows" | sort)` passes while `sort` is an external process whose failure the parent never
  sees. Latent, not live: every process substitution in the gate surface was read, and the two real ones are
  `printf` over memory. Declared rather than closed because the repair was measured first — refusing a producer
  that contains a pipe **also refuses both live sites**, whose pipe sits inside a parameter expansion
  (`${b//|/$'\n'}`), and separating a pipe operator from a pipe inside `${…}` needs shell parsing rather than
  text. Its pin asserts that false positive alongside the bound, so the reason for declaring rather than fixing is
  executable: if a future parameter expansion loses its pipe, that leg fails and the bound is worth re-examining.

- **The repair that removed relative window references named the wrong window at every site it did not
  write.** `this window` is a positional reference: it means whichever window the reader is in, and a doc
  comment carrying it becomes false the moment the window closes. The 0.5.0 window replaced every one of them
  with an absolute `the 0.5.0 window` — correct inside `kanhe` and `shengmo`, which are new crates whose
  every sentence was written in this window, and **false at all four sites in crates that already
  shipped**. `louke`'s injectivity paragraph (twice), `tianheng`'s misdiagnosis-class note and `xingbiao`'s
  checkout-dependence note all appeared between v0.3.0 and v0.4.0, so their `this window` meant the 0.4.0
  window. They now say so. `hunyi`'s `which gained them this release` is the same shape read against the same
  test and is dated with them.

  The four are in **published crates**, so the false attribution shipped to an adopter's `cargo doc` rather
  than staying inside the repository machinery. The test that separates the two groups is mechanical and is
  what should have run with the substitution: a sentence carrying `this window` means the window it was
  written in, so ask whether the sentence exists in the previous release's tree, and take the window from the
  answer rather than from the window doing the editing.

  **Correction, of this entry's own first draft.** It said the window replaced `sixteen` of them and that
  `twelve` were correct. Counting the substitution's own diff answers **seventeen**: the seventeenth line
  sits in a generated document, produced from a literal the same commit changed, so it is a projection of
  another site rather than a site. Sixteen was right only under a convention the sentence never stated, and
  the squash `0bded84` and its pull request still carry it. The counts are gone rather than reconciled — the
  four are named individually in the paragraph above, so that set is enumerated by the text that uses it,
  which is the only figure here anything produces.

- **The requirement that the law is amended only with a human's acceptance was prose, and it never fired.**
  `.github/CODEOWNERS` says the review requirement **is** the reaction and that a merge cannot relax the law
  without a human accepting it, then says in its own last paragraph that designation alone only auto-requests
  review. Measured, `main` answers `require_code_owner_reviews: false` and `required_approving_review_count:
  0` — and enabling it would not close this, because a pull request's author cannot approve their own, so for
  a repository whose steward and author are one person the rule cannot fire at all. A prose prescription with
  no backstop, standing on the law itself, which is the shape the reason rule refuses everywhere else.

  What it cost is recorded above: two crate boundaries reached the projection under a commit body saying the
  law did not change, and nothing refused them. They were found by reading the history months later.

  The law's boundary set is declared as text and held against the projection **in both directions**. The
  declared identity carries more than the heading, because relaxing a law widens a boundary far more often
  than it deletes one — keying on the target alone would pass a 璇璣 that had quietly gained 圭表. The entry
  below widens that identity again and states what it establishes: editing the declaration is what **naming**
  an amendment consists of, in a file CODEOWNERS routes to the steward, and naming is not accepting.

  **The gap was measured before the check was written, and it was real.** With 璇璣's allowlist widened to
  permit 圭表 and the projection re-blessed, all **nine** existing self-governance assertions pass and only
  this one fails, naming `` `xuanji` (crate) `` with `only: serde_json` declared against `only: serde_json,
  guibiao` projected. Neither of the two assertions that come closest reaches it:
  `dimension_boundaries_declare_the_mutual_independence_law` reads only the three dimension crates'
  allowlists, and `every_workspace_member_is_self_governed` asks whether a member has a boundary, never what
  that boundary permits.

  The other direction is run too — a declared boundary the projection does not render fails naming it, which
  is the entry a one-directional comparison would go on certifying. The reader answers *unreadable* rather
  than *no boundaries* for every shape of projection it cannot parse, so the check's own failure mode is not
  the silence it exists to refuse. The subject is the projection's tracked text rather than
  `constitution()`, which `kanhe` can reach: a check calling the law it judges compares the law against
  itself and cannot fail.

  `repository-checks` gains the requirement with it — the two-way property, an identity carrying more than
  the target, the text-not-`constitution()` subject, and a scenario for each way the set can move. The entry
  below adds to all three.

  No published API, outcome, report, exit class, or manifest moves; `kanhe` ships in no package.

- **The amendment check's identity omitted the field the cheapest relaxation moves.** It carried each
  boundary's heading and rule. Lowering a boundary from `enforce` to `warn` changes neither — and it turns a
  run-failing violation into an advisory, which is the most dangerous amendment there is per character moved.
  Run against that form with 璇璣 lowered to `warn` and the projection re-blessed, **all nine self-governance
  assertions and the amendment check passed**. The check was one window old and had already been given a
  falsifier it could not survive.

  The identity now carries the boundary's **heading, reason, rule and severity**. `reason` for the same defect
  from the other side: this window moved three of them, and a check blind to reasons would report an amendment
  named while the sentence a reader takes the law's meaning from is exactly what changed. Both are run:
  lowering 璇璣 to `warn` now fails naming `enforce` against `warn`, and rewriting 星表's reason fails naming
  `metadata substrate` against `metadata layer`. A repeat on either side is refused before the comparison,
  because the earlier form built a set and a set folds a duplicate away.

  **And its claim was stronger than what it observes.** `PROJECT.md` said editing the declaration is what
  accepting an amendment consists of. It is not: one actor can change the law, re-bless the projection and
  edit the declaration in a single commit and pass everything, which shows the amendment was **named**. What
  the check establishes is that a structural delta produces a second explicit artifact in a steward-routed
  file. Acceptance rests on a steward decision, and a single-steward repository has no mechanical second party
  to carry it, since a pull request's author cannot approve their own. That is recorded as a judgement
  boundary rather than renamed — a same-author constant edit called acceptance is the overclaim the reason
  rule refuses.

- **Two governance rules were measured as un-reacted in one window and only one was answered.**
  `.github/CODEOWNERS`'s *a merge cannot relax the law without a human accepting it* was measured against
  `main`'s protection, and its **naming** half was given the reaction it lacked — the acceptance half is a
  judgement boundary, recorded as one two entries above. The reason-perimeter falsifier is in the same state
  and was not: applied by hand across four rounds it produced **eight** corrections and moved no allowlist,
  which is `a repair loop is a diagnosis, not a schedule`'s third class dominating, whose stated remedy is to
  change the shape rather than add a round. The two were noticed as one only when a review put them side by
  side, so `AGENTS.md` gains the binding rule: a governance rule measured as un-reacted is given a reaction or
  filed, in the same change, and the change says which.

  Filed, because the one decidable subset was measured and refuses. *A reason naming a family crate that is
  neither its own target nor in its own allowlist* was implemented against the current projection before being
  adopted: **8 of 13 boundaries fire, all 8 false positives, 0 true positives.** Every hit is a legitimate
  prohibition the allowlist entails — 繩墨's and 勘合's *no edge to 圭表, 渾儀, 漏刻 or 璇璣 can exist*, 圭表's
  *must not depend on the 天衡 shell* — or a module boundary naming the crate it must resolve through. It also
  reaches only one of the three shapes the fourth pass moved: 天衡's *remains the outward composition layer*
  names no crate, and 渾儀's *quarantined* names `syn`, which **is** in its allowlist. What would end the class
  is constructing the entailed half of a reason instead of writing it, which is a design change rather than a
  detector — filed with that shape, its cost, and its promotion trigger.

  `repository-checks` gains the widened identity, the duplicate refusal, the naming-not-acceptance limit, and
  three scenarios.

  No published API, outcome, report, exit class, or manifest moves; `kanhe` ships in no package.

- **The retraction reached three of its four sites, and the fourth is the one a reader reads.** The claim
  that editing the boundary declaration *is* accepting an amendment was withdrawn from `PROJECT.md`, from the
  check's own doc comment and from `repository-checks` — and left standing in the entry above it, which is
  the document an adopter opens. Two typed figures in the same entry had gone stale the same way: the
  identity it describes as carrying the rule now carries four fields, and *four shapes* of unreadable
  projection are six. All three are repaired where they stand, and the figures are replaced by the structure
  rather than re-typed, because a later change in the same window will move them again.

  **How it was found is the part worth keeping.** Every reaction built in this window had a defect found in
  the next round — the membership repair's guard did not reach its consumers, the census removal typed two
  figures of its own, the amendment check's identity omitted severity. Three of three. The entry that fixed
  the third was the only one nothing had read, so it was swept from its own base rather than waited on, and
  the class it carried is the one this window has closed most often: a repair that sweeps the sites it
  remembers rather than the sites a grep finds.

  No published API, outcome, report, exit class, or manifest moves; a record only.

- **A reason that spans a line was read to its first line, and a rendered field was not read at all.** The
  amendment check's identity has now omitted the field the next relaxation moved **twice**. The first form
  carried heading and rule, and lowering a boundary from `enforce` to `warn` moved neither. The second added
  reason and kind — and the projection writes a reason as `> {reason}` on one line only, while `because`
  places no restriction on newlines, so the second line of a two-line reason arrived unmarked and was
  dropped: half a sentence held, the other half unnamed. The same reader ignored `- **anchor**:`, a field the
  renderer emits, so an anchor could be added or changed with nothing moving.

  The identity now carries the `- **…**:` lines **as written, in order**, and reads a reason to the blank
  line before them. A field the renderer gains later enters the identity by itself, which is what stops a
  third form from omitting a third thing — the two omissions were not two oversights but one shape, choosing
  fields by name.

  Negative run, from the terminal: a newline inside 星表's reason renders as `> 星表 is the shared metadata
  substrate: it depends on no workspace member at all.` followed by an unmarked `serde_json only`, and the
  check fails. The parser's own direction reads a two-line reason whole and carries an `- **anchor**:` line
  no boundary currently has.

- **Two citations in live governance text named things that do not exist.** `BACKLOG.md`'s Shape clause — the
  actionable half of a READY-PATCH entry — told an implementer to mirror a test retired with
  `gate-shape-contract` in this same window, so the instruction was unactionable as written; the clause
  already describes the shape in words, so the pointer is dropped rather than repointed. And a live WATCH
  entry named the self-law generator `tianheng_constitution()`, which has never existed, two files from the
  projection header that spells it correctly.

  Both survived a full pre-release review and four review rounds, and the cause is that nothing resolves a
  bare identifier: `reference_integrity` resolves paths, `bound_register` resolves pinning-test names, and a
  backticked name in prose is resolved by nothing. Measured over the live documents: 369 such tokens, six
  unresolved, and all six legitimate — four explicitly past-tense, one in `docs/history/`, one framed by the
  line above it as a kept reproduction record. Filed rather than built, because that exemption is *a name
  cited as history*, which is the prose judgement this repository has rejected three times; what would make
  it buildable is exemption by declaration, a change to how entries are written.

- **The reaction the amendment control got was its naming half, and two documents said otherwise.**
  `PROJECT.md` and the check's own doc comment record the distinction correctly; `AGENTS.md`'s new rule and
  the entry above it both said `.github/CODEOWNERS`'s *a merge cannot relax the law without a human accepting
  it* got the reaction it was missing. It did not: the naming half did, and acceptance is a judgement
  boundary. Both now say so.

  A third site of the same retraction, after the previous entry repaired the fourth — which is the class
  itself, one more time: a repair sweeps the sites it remembers.

- **The rejected detector's figures are anchored to the run that produced them.** The entry filing the
  reason-perimeter class carried `8 of 13`, `8 false positives`, `0 true positives` as live text and used them
  as a promotion baseline, in an entry whose own subject is that a census with no producer drifts. The
  prototype was not retained, so they are anchored to `1fa86d1` on 2026-08-19 with the criterion that produced
  them written out, and the trigger no longer depends on the numbers staying true: it is now a decidable
  subset whose first run over the projection produces a true positive.

  No published API, outcome, report, exit class, or manifest moves; `kanhe` ships in no package.

- **A reason spanning more than one line fell out of its own blockquote.** The Markdown projection wrote a
  boundary's reason as `> {reason}` on a single line, while `because` places no restriction on newlines — so
  every continuation landed outside the quote, rendering as body text, and the projection carried less than
  the constitution declares. That is a defect in a **published crate**, independent of anything reading the
  projection: an adopter running `list --format markdown` over a constitution with a two-paragraph reason got
  broken Markdown. Every line is quoted now, an empty one as a bare `>` so the quote stays unbroken without
  trailing whitespace. A reason with no newline renders exactly as it did, so `AGENTS.self-law.md` is
  byte-identical.

  `constitution-projection` gains the fidelity requirement with a scenario — stated as fidelity, not layout,
  because that capability deliberately leaves the blockquote choice free to evolve and pinning it as a machine
  contract is a standing prohibition.

  **And the reader that consumes it stops guessing.** The amendment check had been recovering a reason's
  extent from a blank line and a `- **` prefix, which cannot be lossless over free text: a reason with a blank
  line inside it lost its second paragraph silently, and a reason line beginning `- **note**:` was filed as a
  boundary field. With every line quoted the reason is one unbroken run, and anything unquoted inside a
  section is refused rather than guessed at. Negative run, from the terminal: 星表's reason split into two
  paragraphs renders `> 星表 is the shared metadata substrate.` / `>` / `> It depends on no workspace member
  at all; serde_json only`, and the check fails; the whitespace gate passes on the bare `>`.

- **Two readers took the first candidate where the input can hold two.** `backlog_classification` anchored on
  the sentence *Classify live work by its* with `nth(1)`, deriving from it the class list the entire check
  compares `BACKLOG.md` against — `AGENTS.md` is hand-edited prose that can hold that sentence twice, and a
  second clause would have been dropped without a word. And `bound_register` took the first backticked name in
  a tracker, so a tracker naming two documents had only the first held against the tracked set: a second,
  untracked name passed, in the check whose whole subject is that debt filed where nobody looks is debt nobody
  owns.

  Both are the `nth(1)` habit this repository already records two live defects from, and both now ask the
  input how many candidates it holds: `the_only` refuses a duplicated anchor as a cannot-judge, and `all_of`
  holds every tracker name against the tracked set. Negative runs: a second *Classify live work by its*
  sentence in `AGENTS.md` fails with `expected exactly one … and found 2`, and a tracker gaining a second name
  fails with `UNPINNED against \`NOTHING-TRACKS-THIS.md\``.

  Found by a review widening its corpus rather than reading harder — its reader sweeps had run over
  `crates/*/src/` and bucketed the test targets as triage-only, and in this repository every repository check
  *is* a test target. The corpus was narrower than the claim, in the review rather than in the code.

  No published API, outcome, report, exit class, or manifest moves. The projection renderer's output changes
  only for a reason containing a newline, which no declared boundary has.

- **A reason about what ANOTHER crate does was the shape the second perimeter pass missed.** The falsifier
  the law's own header states took **three** passes to apply to eight boundaries, and each pass missed a
  different shape. The first read *entailed by the allowlist* generously; the second caught phrases about
  edges pointing **at** a crate and left three phrases about edges pointing **away from** other crates. 圭表's
  *the observation dimensions are composed only by the 天衡 shell* and 天衡's *remains the outward composition
  layer* both describe the shell's behaviour: stop composing, keep every declared edge, and both boundaries
  stay green while both sentences turn false. 渾儀's *quarantined* says `syn` appears in no sibling allowlist
  — add it to 圭表's and 渾儀's boundary is green while the word is false.

  The third was left standing on the second pass with a note saying it sat inside the law's perimeter without
  sitting inside this rule's. That is true, and it is not the test: the test is about this rule, and a reason
  is attached to this rule. The note was a way of keeping a sentence the falsifier had already refused, and
  recording it is worth more than the edit — an accurate observation used to overrule a decidable test is the
  harder failure to see. The header now names both missed shapes rather than showing one by example, since an
  example teaches its own shape and nothing else.

  No target, rule, or allowlist changes; the projection moves with the reasons.

- **`check_and_cover`'s published contract said `None` had one cause after it was given a second.** The
  membership repair earlier in this window added `Members::Unreadable → None` to the coverage arm, and the
  doc comment above it still said coverage is `None` *only when the metadata itself could not be read*. That
  is a public API's stated semantics for its own `Option`, in a crate an adopter compiles against, made false
  by a change three functions below it. It now names both facts and says that each is also reported as a
  constitution error, so `None` never travels alone.

  **And the guard for that repair did not reach the consumers it was written for.** It held
  `workspace_member_names`, whose `enum` already forces every caller to match — but matching is not choosing
  the right arm, and a later edit mapping `Unreadable` to an empty membership or to a fabricated coverage
  would have left it green. The coverage arm was inline in a function whose only entry point spawns
  `cargo metadata`, so no direction could hand it a membership to fail on; it is now `coverage_of`, named and
  reachable, and a new direction calls both consumers over metadata that cannot be read.

  Negative runs, from the terminal: mapping the coverage arm to a fabricated coverage fails with `no
  \`packages\` array: coverage over a membership that was never read is coverage over nothing`, and mapping
  the outcome arm to an empty membership fails with `the error must name what could not be read, got: a
  boundary must govern a real crate or it silently never reacts: target crate 'core' is not a member of the
  target workspace` — which is the pre-repair defect exactly: the right refusal, about the wrong thing.

  No signature, outcome kind, exit class, identity shape, or manifest moves; `coverage_of` is private.

### Self-governance

- **A `case` opened inside a wrapper's parser truncated the arm set, and the arm most likely to go was the
  catch-all.** `wrapper_parser::parser_arms` bounds itself to the parser's own `case` with a boolean, and
  shell's `case` nests — so an inner `esac` closed the outer read and every arm after that block left the map
  unannounced. Nothing downstream could report it: `gate_exit_classes` compares *takes* against *judged* and
  `publish_workflow` compares *asking* against *consuming*, and an arm dropped before either set is built is
  missing from both, which is two sets agreeing over a subject neither read.

  The existing direction did not cover it and could not be read as covering it. `the_scan_reads_the_parsers_case_and_no_other`
  places its second `case` **after** `esac`, where the flag is already clear — a sibling, not a nest — so it
  measured the over-inclusion that had already been repaired and said nothing about this direction.

  Two spellings, because the second is the one a check written for the first would have missed. Measured
  against the unrepaired reader, on a parser declaring `--subject`, `--body-file` and `*`: the differently
  spelled nest (`case $conclusion in`) answered `Some(["--subject"])`, and the parser's own spelling
  (`case $1 in`) answered `Some(["--nested", "--subject"])`. Written inline rather than in a fenced block,
  because a fence in this file is the shape `the_corpora_of_the_bare_str_markdown_readers_carry_no_fence_or_comment_span`
  reports as live — this entry drafted one and that direction refused it, which is the declared latent
  misread doing exactly what it was declared to do.

  The first truncates. The second is worse and is why the refusal sits **above** the `PARSER_CASE` arm rather
  than below it: spelled as the parser's own opener, the nested block takes that arm's `continue`, so its arms
  enter the map as the wrapper's — admitting an arm the wrapper does not have — and the inner `esac` then drops
  the real ones. A check placed below that arm closes the first spelling and leaves the second.

  Repaired as a refusal rather than as nesting analysis. Neither wrapper nests a `case`, and this family does
  not build machinery for a state that does not occur; the read stops rather than shrinking, which is the answer
  the unattributable-guard arm beside it already gives. A wrapper that needs a nested `case` is the reason to
  build the counter, and is not one this repository has.

- **The paragraph is the unit that pairs, and the previous repair's "over-reaction in the safe direction" was
  inert.** A Markdown code span wraps a line freely, so a per-line reader joins one span's closer to the next
  line's opener and judges the prose between them — measured: 503 lines across 88 tracked files carry an odd
  number of backticks. The repair before this one scanned such a line entire and called that safe. It caught
  nothing and could not: the span is then the line, the line contains a backtick, so the coordinate shape's
  left half is neither a tracked path nor empty. The hole was declared closed and was not.

  A code span cannot contain a blank line, so the paragraph is where single-backtick spans close. The same
  corpus has 59 odd paragraphs across 21 files, and those are fenced blocks and doubled markers rather than
  wrapped spans; each is judged whole, carrying the line it starts on. Measured: a coordinate standing after
  a wrapped span in the same paragraph is read at its own line, which per-line pairing missed.

  `reading::backticked` had one `Err` that four consumers read two ways — three as a refusal, one as *judge
  the block whole* — kept apart by a `match` arm at the call site. The second reading is
  `backticked_by_paragraph` now, so a fifth consumer picks by name.

- **The unreachable-branch reader decides the consumer by what it does, and it used to decide by four
  spellings.** The list held `.unwrap_or`, `.unwrap_or_default`, `.unwrap_or_else` and `?`, so the same dead
  default written `.map_or(d, f)` was invisible — and two live sites used a fifth and a sixth:
  `guibiao`'s `symbol_scan` read `.rsplit("::").next().is_some_and(…)`, in a published crate, and
  `one_spelling` read `.rsplit('/').next() == Some(…)`. Thirteen consumers now, every one of them total over
  `Option` and a no-op on an always-`Some` one; `.map(`, `.and_then(`, `.filter(` and `.expect(` stay out for
  reasons the constant states. Both sites are repaired.

  It also reported code that was correct. `if let Some(x) = lookup() { let f = v.split('/').next(); }` was
  called a dead branch, because the reader asked whether the text *before* the call contained the marker
  rather than whether the construct consumes that expression. Between a construct and the expression it
  consumes there is no statement boundary, so a `;`, `{` or `}` in between now says it consumes something
  else. `match` and the `let … else` binding joined the constructs at the same time.

  The backtick rule had the same shape one file over: `reading`'s own doc names **two** primitives it
  replaced — a `find` twice in a loop and a `split` with `step_by(2)` — and the reaction closed only the
  second, so pasting the loop that had just been deleted anywhere else swept clean. Both are refused now, and
  the five other primitives that reach a backtick literal are a declared bound: all five are in live use for
  reading one delimited value, where they are correct.

- **A guard that could never fire stood in a repository check, written in the same batch as the reaction for
  that class.** The wrapped-span repair put a `line.matches('`').count() % 2 == 0` test in front of
  `reading::backticked` — the same predicate the reader itself decides — so the `Err` arm behind it was
  unreachable, the vector it filled was always empty, and the assertion holding that vector empty was a
  guard nothing could make fire. The parity decision lives in one place now and the two arms are the two
  answers. Two leftover bare blocks from the earlier mechanical rewrite went with it, and a comment
  describing the policy that was rejected — a reader met two comments stating opposite policies and the code
  did the second.

- **The pipeline reaction's corpus is a parameter now, and the two places the class mattered most were
  outside it.** The reaction was written for `ci.yml` and both wrappers ran
  `printf '%s' "$output" | grep -qE 'test result: ok\. 1 passed'` under `set -Eeuo pipefail`, in front of
  `cargo publish` and `gh pr merge`. If it had fired, a passing gate would have reported *the gate did not
  run — its invocation selected no passing test* and stopped the act: a refusal about a renamed test for a
  fact about a closed pipe, immediately before something that cannot be undone.

  It is not live, and why it is not live is the point. Measured over a 405 KB stream: with the token at the
  end — where a `cargo test` summary sits — 0 of 8 runs returned non-zero, because `grep` must read
  essentially everything before matching; with the same token near the start, 8 of 8 did. Both wrappers were
  holding by where the token happened to sit, which nothing declares and nothing keeps true. Both read a
  here-string now.

  The reader was narrow in two more ways. It took only the segment after the **last** `|`, so a `grep -q`
  standing mid-pipeline was invisible; it now tests every stage fed by a pipe, and only those — a
  `grep -q … <<< "$value"` opening a line closes nothing, and testing every segment reported the SARIF
  assertion this rule exists to have repaired. And it matched three literal prefixes, so `grep -Eq` — the
  same request with the cluster written the other way round — matched none of them; the consumer is decided
  by what its flags ask for, long forms included.

- **A derivation stands where its status is visible, and the entry stating that added a second one.** The
  commit that converted `packaged-selftest`'s `mapfile -t crates < <(…)` wrote *the tool's absence is now the
  tool's own error* and added a new `mapfile -t declared < <(…)` to the `msrv` job in the same diff.
  `pipefail` cannot see inside `<(…)`: measured, an absent `jq` gives exit 0 and zero lines through the
  process substitution, against exit 127 through a command substitution. The `!= 1` floor caught it and named
  the workspace for a fact about the tool — the exact misdirection the sibling job's comment records as
  removed. `no_step_reads_a_value_through_a_process_substitution` refuses the shape now, over the same
  three-file corpus.

- **Every backtick pair now goes through the one reader, and a check that reads spans by line says which line
  it cannot decide.** The entry two below extracted `reading::backticked` and converted the three sites a
  review had named. Sweeping this session's own output found `split('`').skip(1).step_by(2)` still standing in
  three more — `reference_integrity`, `bound_register`, `backlog_classification` — the exact shape, in the
  window whose subject was that shape. `no_source_outside_the_shared_reader_pairs_backticks_by_hand` makes a
  seventh impossible.

  Converting `reference_integrity` measured something its per-line scan had been getting wrong all along: a
  Markdown code span **wraps a line** freely, and hundreds of tracked lines carry an odd number of single
  backticks. On each of those the old scan paired the halves with whatever came next, so a gate whose whole
  subject is what backticked text names was reading the prose between one span's closer and the next opener.
  A whole-document pairing is not the answer either — fenced blocks and `` `` `` spans make an even count
  wrong for a document — so a line this reader cannot decide was scanned entire. **That was called an
  over-reaction in the safe direction and was inert; the entry above replaces it and says why.**

  The three `trim_matches('`')` readers of a citation name stay as they are, measured rather than assumed: an
  unbalanced marker on a `PINNED-BY` line is accepted and still names the right test, so that tolerance
  repairs a typo instead of shifting a pairing. It is a different shape and it is left alone.

- **A pipeline that reads a value stands alone, and that is held rather than remembered.** Adopting
  workflow-level `pipefail` in the previous entry made a consumer that stops before its producer finishes fail
  the whole pipeline, and the three sites were repaired by hand — which left the rule living in whoever
  remembered it. `no_step_reads_a_value_through_a_pipeline_that_stops_early` refuses the shape by name now.

  Two of those three had not tripped, and not for a reason belonging to the pipeline: `cargo metadata` emits
  one line of JSON, so `sed … | head -n1` printed once and reached EOF. A latent SIGPIPE waiting on an output
  shape is worse than a live one, so the reaction refuses those too.

  The reader recognises its consumer from a list of three — `grep -q`, `grep -m`, `head` — which is an
  approximation and is declared as one: the set of programs that exit early is not closed, and the question
  behind it, *does this stage read its input to EOF*, is not one a reader over shell text can answer. That
  residue is a declared observation bound with a `BACKLOG.md` entry rather than a sentence in a doc comment,
  and the two strictness scenarios that were already observed now say which direction observes them.

- **A figure the census sweep cannot represent refuses, and the answer it used to give was not silence but a
  fabricated number.** `number_at` sent an overflowing digit run to `parse().ok()?`, which is the same answer
  as *there is nothing numeric here* — the conflation `reading`'s module doc names as the one bug this
  repository forbids. But `figures_in` advances one byte and retries, so the next attempt saw the same run one
  digit shorter, and eventually one that fits: measured, a 26-digit run read as `9999999999999999999`, and
  that figure — which no document wrote — was compared against the declared census. In the module whose whole
  subject is a declared figure disagreeing with a produced one.

  The three states travel from `number_at` through `match_from` and `figures_in` to `sweep`, which reports
  `repository-checks#census-figure-unreadable` naming the document, the line, and the run. Restoring the old
  answer no longer compiles at the call site; the typed perturbation that does compile reproduces the
  truncation exactly.

- **A backticked name has one reader, and a marker that closes nothing refuses.** Three sites paired markers
  as they arrived — `find('`')` twice in a loop, and `split('`').skip(1).step_by(2)` — so one unpaired marker
  shifted every pair after it. Measured at both sites that had the defect: a `## Capabilities` section listing
  `` `alpha` ``, a stray marker, then `` `beta` `` answered `{" here\n- ", "alpha"}`, admitting the prose
  between the stray marker and `beta`'s opener as a capability name and dropping `beta`; an admitted-types
  clause reading `` `feat`, `fix` and `chore `` answered `["feat", "fix", "chore"]`, taking the unterminated
  tail as a type. Neither could report the condition, because a shifted pairing is readable — it yields names,
  just not the document's.

  `reading::backticked` decides the count before it takes a pair, and the three sites the review named call
  it. One of them was the site that refused correctly, and it refused by a shape check rather than by
  counting, so the correct half of the module could tell the other two nothing. **Three more sites of the
  same shape were still standing when that was written, and the entry below closes them** — the sentence
  claiming the rule was itself the round's residue.

  `proposal_capabilities` answered `Result<BTreeSet<String>, usize>` — an error channel with room for *how many
  sections* and none for *unreadable*, which is why the state was skipped in the module whose sibling reader
  refuses it. It answers `Named` now, the sibling of `Declared`, and its consumer names the third state
  rather than treating a section it cannot read as a section naming fewer capabilities.

- **A branch a guard has already made unreachable is refused now, and the measurement that would have caught
  it was already in the tree.** `str::split` and `str::rsplit` always yield at least one item, so a `.next()`
  on either is always `Some` — `merge_message_gate` measured exactly this, wrote it down, and repaired its own
  site. Twenty-four sibling sites across three crates kept the shape, two of them written *after* that
  paragraph existed. One of them carried a comment saying the fallback names no state any input can reach,
  with the `unwrap_or_default()` still standing beneath it.

  `unreachable_branch` is the reaction, and it found four sites a line-at-a-time `grep` could not: `rustfmt`
  breaks a long chain so `.next()` and its consumer land on their own lines. Every site moved to
  `split_once`/`rsplit_once`, whose `None` is reachable and means something — the separator is absent. The
  sweep's own positions come from the region reader rather than from re-counting, because `numbered_lines`
  drops a whole-line comment and enumerating what `lines` returns numbers the remainder; the first run
  reported four offences at lines holding something else.

  `AGENTS.md`'s minimalism bound now names its one decidable instance and says plainly that the rest of the
  bound is judgement with no reaction.

- **A key is identified exactly, and it used to be identified by its prefix.** `manifest::publishable` reached
  the `publish` field with `strip_prefix("publish")`, so every `[package]` key beginning with those seven
  letters went down the value path: `publish-lockfile = true` — a key cargo itself once accepted — standing
  before a real `publish = false` read as *unreadable manifest* and refused the whole member, while cargo
  treats a key it does not know as unused and carries on.

  The same reader recognised one spelling of the key and one of the table header, and that direction is the
  worse one. Measured on cargo 1.96.0: `"publish" = false`, `'publish' = false`, `[ package ]` and
  `["package"]` all report `publish=[]`, and the reader answered **publishable** for every one of them —
  a crate cargo refuses to publish, called publishable, because a spelling left the key or the whole table
  unread. Two `publish` keys under one `[package]` now refuse rather than answering from the first, since
  cargo refuses such a manifest outright. Eleven rows joined the matrix and all seventeen spellings agree
  with cargo.

  The Cargo.lock reader carried the same shape and two more: each arm asked `starts_with(key)` and
  `contains('=')` and then split again behind an `unwrap_or_default()` the `contains` had already made
  unreachable — two decisions about one character, and a default nothing could reach. It reads the key once
  and matches it exactly. Two `if let Some(base) = path.rsplit('/').next()` guards in the same file were
  conditions nothing could fail, which is the fact `merge_message_gate` had already measured and recorded;
  they are gone.

  A newly public `cargo_metadata` kept the private function's doc summary glued in front of its replacement,
  pointing at machinery a reader of the public surface does not see. And the coverage floor added with the
  both-ways check counted the loop's own iterations, which could only fail if the loop had a `continue` it
  does not have — `f() == f()` reading as a coverage claim. It compares cargo's package set with the member
  directories on disk now, an enumerator cargo did not produce.

- **Shell strictness is the workflow's property now, not each step author's.** One CI step set
  `set -euo pipefail` and another set nothing, so a three-stage derivation ending in `tr` reported only its
  last stage: measured on all three failure paths — `cargo metadata` failing, malformed JSON, `jq` absent —
  the pipeline gave exit 0 and an empty set, every crate was treated as publishable, and the job said *kanhe
  is missing LICENSE-MIT* for a fact about an absent interpreter. Loud only by circumstance, since those two
  crates carry no license texts. `defaults.run.shell: bash -euo pipefail {0}` makes it one decision, and
  `shell_strictness_is_declared_once_for_the_whole_workflow` refuses a step that takes it back.

  Adopting it has a cost, paid here rather than discovered: `pipefail` makes a consumer that exits early fail
  the whole pipeline. `printf | grep -q` over the SARIF document measured 141 on five of five runs, so that
  assertion reads a here-string. The same sweep found two shell readers picking one of several and saying
  nothing about the rest — the MSRV read one of eight `rust_version` values through a greedy `sed` and
  `head -n1`, and the packaged self-test read one of however many `.crate` files sat in `target/package`
  through `ls | head -1`, where a stale tarball from an earlier version could be the one tested. Both answer
  the count first now, which is `selection::the_only`'s question asked in shell.

- **The two readers of that fact are held against each other now, and one of them was wrong by a whole
  spelling.** The previous entry gave *is this crate published* one criterion and left two deliberate readers
  — `manifest::publishable` over text, cargo in the workflow — with nothing between them. What connected them
  was a hand-kept six-crate literal in `doc_provenance`, a third owner, and `publishable`'s own matrix, which
  is `f(literal) == expected` over strings no manifest in the tree contains: it encoded a belief about cargo
  rather than cargo's answer.

  The belief was wrong. `publish = [ ]` — one space, legal TOML, and refused by `cargo publish` exactly as
  `[]` is (measured on cargo 1.96.0: `cargo metadata` reports `[]`, the dry run errors) — was answered
  **publishable**, because the arms matched one spelling of the empty array and sent every other bracketed
  value to `Yes`. In the function written because text readers called the empty array published. The verdict
  follows the array's contents now, the literal is retired in favour of a derivation, and
  `the_text_reader_agrees_with_cargo_about_every_member` compares every member's text verdict with cargo's own
  report in both directions.

- **A workflow step uses a host tool this repository declares.** The previous entry's repair reached for
  `python3` in two CI jobs — in neither `AGENTS.md`'s host-tool list nor any reaction, while `jq`, already on
  that list, does the same filter in one expression. Its absence would have failed in the shape that
  paragraph exists for: `mapfile` reads zero lines from a failed process substitution and neither `set -e`
  nor `pipefail` sees inside `<(…)`, so the empty-set floor fires saying *no publishable crate was derived
  from cargo metadata* — a sentence about cargo for a fact about an absent interpreter, which is verbatim the
  `jq` measurement that paragraph records. Measured: exit 0, array length 0. Both jobs use `jq`, whose
  `null == []` is false, so the semantics stay cargo's.

- **One fact about a manifest has one reader per language, and cargo's own semantics reach all of them.**
  *Is this crate published* had four readers on two criteria, and the previous round's repair added two of
  them: two CI jobs grepped `^\s*publish\s*=\s*false`, `doc_provenance` asked
  `starts_with("publish") && contains("false")`, and only `shengmo`'s self-governance read cargo's report.
  Measured on cargo 1.96.0: `cargo publish --dry-run` refuses `publish = []` **exactly as** it refuses
  `false`, and `cargo metadata` reports `[]` for both — so a crate spelling its exclusion as the empty
  registry list was classified *published* by all three text readers. `publish.workspace = true` is honoured
  by cargo and invisible to every one of them.

  `kanhe::manifest::publishable` is the text-side owner, with a third state for the value a manifest alone
  cannot decide — the workspace inheritance is a real instance of it, not a defensive arm. The two workflow
  jobs ask **cargo**, which owns the semantics and needs no build to answer. Six shapes are pinned, including
  that a `[workspace.package]` default is not a member's verdict and a commented-out key is not a key.

- **The interpreter pin's third leg is read now, and the prose that rested on it was the finding.** The pin is
  `node-version`, the `NOT-BEYOND` declaration beside it, and `engines.node` — one commitment written three
  times, of which two were held against each other. The module header rested on *`package.json` declares
  `">=24 <25"`* and the date refusal tells the operator *the three move together*, while the reaction saw two.
  Widening `engines.node` to `">=24"` passed every reaction and let a local Definition of Done take a
  different Node major than CI's `24.16.0`, silently. The reaction now holds that the range admits **exactly**
  the pinned major — lower bound the major, upper bound its successor — and eight constructed directions cover
  no upper bound, an upper bound a major too high, a lower bound below the pin, a different major, an exact
  version, none, and two.

- **Two rules this repository already owned were broken by the code that repaired them, and both are
  repaired at the rule rather than the instance.** `wrapper_parser::value_guard` reduced a set of guard names
  with `.next()` — the habit `selection` exists to end, in a module whose own header forbids shrinking a set
  two paragraphs up. It answers `selection::the_only` over the distinct names now, so several guards in one
  wrapper is a finding rather than a silent pick. And `parser_arms` read every `case` in the script: correct
  only because `merge-pr.sh`'s inner `case $conclusion in` writes each body on its pattern line, so no line
  there ends in `)`. Reformatting it would have collided that `*)` with the parser's own `*` and
  `BTreeMap::insert` would have dropped one — a dropped catch-all being the arm every refusal rests on. The
  scan is bounded to `case $1 in … esac` now, removing a dependency on someone else's formatting.

  **Both got a direction of their own, and the first perturbation of the boundary proved nothing.** Removing
  the boundary and reformatting the wrapper's inner case left the suite **green**: the wrapper's two
  catch-alls carry identical properties today, so the collision is real and its consequence there is not. A
  constructed script gives the second `case` arms whose properties differ from the parser's — a `--subject`
  that consumes without guarding, a catch-all that consumes — and removing the boundary then reports the
  parser's own `--subject` as unguarded. That is the difference between a guard and a restatement, and only
  running it says which.

- **A specification's prose and its scenarios are edited in one pass, and two measured repairs are declined
  rather than attempted.** The requirement added last commit had both halves of one seam wrong: *The scope
  SHALL be stated rather than implied* is a `SHALL` about that specification's own prose, which nothing can
  hold, and the third direction (`the_published_set_is_the_one_the_manifests_declare`) was pinned by a
  scenario the requirement never declared a property for. The first is now prose, the second has its sentence,
  and `AGENTS.md` names the reverse direction its existing rule did not own: a scenario must name its
  reaction, and requirement prose must not gain a clause with no scenario.

  **A Rust identifier named in a doc comment's prose is resolved by nothing, and that is now a declared bound
  rather than an open question.** Found by a grep run for another purpose: `uses_by_file` sat in published-crate
  rustdoc against **0** occurrences anywhere in the tree. `reference_integrity` matches paths and
  `gate_identity` matches `--exact` citations in scripts, so neither sees it.

  Two repairs were designed, measured, and declined — which is the whole content of the bound:

  - *A text check resolving each backticked name against the tree's declarations.* Measured over every
    published crate's `src`: 2,373 such tokens, **859** matching no declaration — and the 30 most frequent of
    those are 19 Rust keywords (`use`, `mod`, `dyn`, `fn`, `impl`), attribute names (`cfg_attr`) and std
    method names (`create_new`, `strip_prefix`, `remove_dir_all`). Separating them needs type information
    about a receiver, which `inline-symbol-path-confinement` already declares unobserved.
  - *Rewriting such a token as `[`name`]`, making `rustdoc -D warnings` the reaction.* That half is real —
    measured, an unresolvable link form fails with `unresolved link to …` while the prose form is silent. But
    of 8 candidates sampled as *a name declared in the same crate*, **8 of 8** were parameters, fields or
    locals whose link form correctly fails. A rule asking for the link form is wrong for the majority of
    prose backticks.

  So `reference-integrity` declares it, `BACKLOG.md` tracks it, and the register's own figure moves from 92
  bounds to 93 with the unpinned count from 20 to 21.

  **`doc_provenance`'s vacuity floor now states the property it stood for.** A `> 50` threshold sat against an
  actual 131 — a number answering nothing, and blind to a single crate dropping out of the enumeration. It
  asserts instead that every published crate contributed at least one source, which is what a lost `ls-files`
  argument or a renamed directory would actually break.

- **A doc comment no longer indexes its own provenance by review round, and a reaction holds it.** `AGENTS.md`
  settles the disposition — *a review round number, a pull request number → provenance* — and 28 doc lines
  across five published crates carried one anyway. Eleven carried it as the index to *see `PROJECT.md`'s
  Decisions*, and `PROJECT.md` holds **no** entry organised by round, so those eleven pointed at a structure
  that does not exist. Every one of the 28 carried its invariant alongside the round, so the repair kept all
  28 invariants and dropped only the moment-names; `crates/kanhe/tests/doc_provenance.rs` refuses the next
  one.

  **The scope is narrower than a first reading suggests, and that is measured rather than softened.** None of
  the 28 attached to a `pub` item — 10 private, 8 `pub(crate)`, one private-module `//!` — so
  `cargo doc --no-deps` generates none of them and docs.rs showed none. This was never adopter-facing. The
  reader it cost is whoever opens the source, an agent with the file in context included, and the round number
  names *when* rather than *what*.

  **The corpus stops at doc comments and the stop is a run, not a sentence.** 27 `//` inner comments carry a
  round number and sit outside by construction; a direction gives the reader both forms and requires it to
  separate them, alongside a `rounds to 3 decimal places` case for the token boundary. `BACKLOG.md` carries
  the 27 as a WATCH with the trigger that would widen the corpus.

  **And the ordering rule the two review passes kept tripping over is now stated.** Both proposed replacements
  that would have deleted an invariant while reaching for its provenance — because the two sat in one
  sentence. `AGENTS.md`'s table now asks for **invariant first, observation second, in separate sentences**,
  and the two passages that prompted it (`is_semver`, `Declared`) are written that way.

- **A multiline basic string is refused too, and it reached the same silence with no backslash in it.** The
  escape branch below closed one door and left the neighbouring one open. TOML admits `"""…"""` wherever it
  admits `"…"`, and cargo reads it — measured on cargo 1.96.0 against a scratch workspace:
  `path = """crates/xuanji"""` resolves the member and `name = """xuanji"""` / `version = """0.5.0"""` read as
  `xuanji` and `0.5.0`. This reader stripped the opening quote, found the next one immediately, and answered
  `Value("")`: an empty path, an empty identity, an empty version, each of which its consumer compares and
  passes over. Measured before the repair: `Value("")` where `Unreadable` was required. The check is three
  quotes — two after the opening one is stripped — and it is a check of its own because the backslash branch
  cannot see a shape that carries no backslash. An ordinary empty `""` is still a value it reads, and that
  boundary has a direction of its own.

  **Its position is not the property, and this entry's first draft said it was.** It reads the text before the
  split rather than the value after it, so it answers the same either side of the backslash branch. Measured
  by moving it past the body read: the direction over it stayed green. Recorded because a claim about ordering
  that nothing holds is the shape this repository removes on sight — what the direction pins is that the check
  exists, not where it sits.

  **Two mechanisms, two directions.** The escape matrix and the multiline one were one test for a moment, and
  its name said only the first — so they are separate now, each naming what it refuses.

  **And one of the escaped-path positions was never a false negative, which this entry's first draft said it
  was.** Measured by removing the backslash branch and running the direction: an escape **inside** the
  `crates/` prefix was not selected, took the `continue`, and reached a release clean — that is the
  regression direction. An escape **after** the prefix was still selected and still compared, and the old
  reader answered `internal-pin-disagrees` naming the stale pin. Not clean, and not a missed check. The
  direction keeps both, because the fail-closed rule is uniform, but only the first is evidence of the old
  silence — and `require_internal_pins` never resolves a crate identity from a path, so the claim that one
  was "compared against a name no crate has" described nothing.

  `release-coherence`'s renamed-package scenario said the ordinary sibling sits in *another example*, while
  the fixture deliberately puts both in one manifest — the only arrangement the per-example counter cannot
  see. The scenario now says so, so its `PINNED-BY` is not a completion claim about a different
  configuration.

- **A TOML escape is a value the manifest reader refuses now, not one it answers undecoded.** `quoted_value`
  took the text up to the first `"` and returned it as a `Quoted::Value`, so a legal TOML basic string
  carrying an escape was reported as one this reader had read. Measured on cargo 1.96.0 against a scratch
  workspace, which is the half that makes it a defect rather than a limitation: `path = "crates/\u0078uanji"`
  resolves the member at `crates/xuanji`, `name = "xuan\u006Ai"` reads as `xuanji`, and
  `version = "0.\u0035.0"` reads as `0.5.0`. **Cargo decodes; this reader decoded nothing and said it had.**

  What that cost is silence rather than a wrong answer. Every consumer compares the undecoded text — against a
  `crates/` prefix, against the family crate list, against a version — and a comparison that fails takes a
  `continue`. So an internal dependency with a stale pin, or a renamed family dependency with one, stopped
  being checked. The per-manifest and per-example vacuity guards could not see it either: one escaped entry
  beside one ordinary one leaves their counters non-zero, which is exactly the partial case those guards were
  added for one level up.

  **Where those guards do and do not reach is now measured rather than assumed, and it cost a direction.**
  `requirements_here` is counted **per example**, so an escaped entry alone in its own example leaves that
  counter at zero and the vacuity guard catches it. The first cut of the renamed-package direction put the
  escaped entry and the ordinary one in two different examples — and it passed under the perturbation, which
  is a restatement rather than a guard. Both entries now sit in one manifest, and the direction asserts the
  refusal's **site** rather than only its kind, so a refusal the vacuity guard produced can no longer read as
  this direction's evidence.

  **The repair is one branch, and deliberately not a decoder.** A backslash in a basic string always opens an
  escape, so `quoted_value` answers `Unreadable` on any — and every consumer already had an `Unreadable` arm
  that refuses as a cannot-judge, so nothing downstream needed changing. Writing a TOML escape decoder here
  would be a second hand-rolled grammar, which is the defect class `BACKLOG.md` already carries for these
  readers. Measured over `git ls-files '*.toml'`: no tracked manifest carries a backslash in a quoted value,
  so this refuses nothing this repository writes.

  It also closes the narrower shape the same read missed — `"a\"b"` split at the **escaped** quote and
  answered `a\`, an identity no manifest declares.

  `release-coherence` already required this: *every enumeration SHALL distinguish absent from unreadable, and
  SHALL refuse as a cannot-judge on the second*. Its scenarios reached a file that cannot be read; the value
  level is where the requirement was not being met, and it now has scenarios of its own. **An earlier review
  of this window saw the escaped-quote half and filed it as latent** on the ground that no package name, path
  or version can legally contain a `"` — true, and the wrong perimeter: the class is every escape, and
  `\uXXXX` is legal in all three.

- **Two backlog entries carried an unmeasured premise, and each now carries the measurement instead.** Both
  were found by the `openspec/specs/**` review pass — the one the two earlier passes had declared their
  largest coverage gap — which found no defect inside that scope and these two outside it.

  *The un-reacted-SHALL entry said the binding is written nowhere.* Its `First step` rested on that: *the
  binding must be **derived**, not declared — which test defends a requirement is nowhere written*. Measured
  at `36152c0`: **161** `PINNED-BY` citations cover **62 of 364** requirements, every one names a test that
  exists, `bound_register` resolves each through `cargo test -p <member> -- --list` per package while refusing
  an empty enumeration as the vacuity direction, and `pin_bites` runs 13 of them against a perturbed tree
  requiring each to fail. The absolute was true of the other 302 and false of exactly the half that would make
  the first step cheap. The step is now two, in order: extend the citation where the reaction already exists
  and is uncited — sampled in `repository-checks`, 18 of 32 requirements carry one and all five sampled
  uncited requirements have a reaction — then design a derivation for what a citation cannot reach.

  *The host-tool-fixture entry's promotion trigger had never been run.* It reads *a second reaction fixture
  found taking a host tool it does not declare*, and the decidable half is enumerable by a criterion sharper
  than a tool-name list, which cannot be known complete: a stub is executed only where a test puts its
  directory on `PATH`, and exactly **two** targets do. `merge_workflow`'s `gh` stub pipes through `jq`, which
  is the entry; `publish_workflow`'s `cargo` stub reaches no host tool at all. Every other host-tool name under
  `crates/*/tests` is prose or a fixture's own *subject* — `echo tail # cut` and `curl "$url#frag"` are text
  the region classifier reads. So the class has one instance by measurement rather than by nobody having
  looked, and the entry records the `PATH` criterion for whoever re-runs it.

- **Both wrappers refuse a flag-shaped value now, and a reaction holds the property across them.**
  `repository-checks` already required it — *a value position SHALL NOT be a place a refused argument may
  sit*, *this SHALL hold for every value-taking arm rather than the one measured to leak* — under a
  requirement about any sanctioned irreversible act. `scripts/publish.sh` implemented it with a paragraph
  arguing for it; `scripts/merge-pr.sh` kept a guard that checked only that *something* followed. So the
  requirement had an unsatisfied instance, not a missing clause. The consequences differ, which is why
  stating the rule at one wrapper read as covering both: where the tool does not consume a flag-shaped value
  the refused argument reaches it, and where the caller **does** — `gh`'s does — the admitted flag is
  swallowed as text and the gate then reports a subject disagreeing with the title. Measured:
  `--subject --admin` made the subject the literal string `--admin`, the operator's flag never reached `gh`,
  and the refusal was about the wrong thing one step before a record that cannot be repaired.

  **The repair is a reaction over both, not a second copy of the guard.** Two implementations of one rule
  agree by maintenance. `each_wrapper_refuses_a_flag_shaped_value_in_every_value_position` reads both
  wrappers from `gate_exit_classes`'s existing `WRAPPERS` pair and finds each guard by **the shape of its
  call** — three arguments: the count, the flag, the value — rather than by its name, because the two spell
  it differently (`require_value`, `require_a_value`) and a literal pair of names would be a third thing to
  keep in step. A third wrapper is covered on the day it is written.

  **The first cut of that reaction was not enough, and its own negative run is what said so.** It asserted
  that *some* guard call carried a value and that the guard judged the shape. Perturbed by shortening one
  arm's call to two arguments, it stayed green — the other arm still carried a value, so the guard was still
  found and still checked. A reader whose input is narrower than the claim it makes, in the direction written
  to close exactly that. The reading is now **arm-level**: an arm that consumes the following argument must
  hand its guard that argument, held in both directions, and the assertion names the arm that does not.

  **So the reader moved to `crates/kanhe/src/wrapper_parser.rs`,** where it is one implementation with two
  consumers rather than the copy the sibling direction would have been. It gained a third property in the
  move — `guards_with_value` beside `guards` and `consumes` — because two could not tell an arm asking for
  nothing from an arm handing its guard nothing to judge: the shortened call still opened with the guard's
  name, so it still read as guarded. `publish_workflow` now reads through it and keeps using `guards`, which
  is the property its own assertion was written for.

- **The set a job claims to cover is the one it derives.** `packaged-selftest` is named *every publishable
  crate's tests pass from its packaged tarball* and then hand-listed them twice — once for the resolve
  patches and once for the packaging loop — with nothing holding either list. The `license-files` job beside
  it derives the same set from the same two facts (`crates/*/Cargo.toml` and `publish = false`), and
  `release_coherence_gate` derives it from `cargo metadata` for the recorded reason that a path-shaped corpus
  went stale, so a seventh publishable crate would have been caught there and silently skipped here while
  this job's name went on saying *every*. Both lists are now one derivation from those two facts. Measured
  against the lists it replaces: the same members, and `cargo package --no-verify` exits 0 with every derived
  member patched, so the second hand-written fact a partial derivation would have needed — *which of them is
  the shell nobody depends on* — is not needed either. An unused patch is a warning, not a failure.

  **The derivation states its own floor, because a derived set can be empty where a hand-written one cannot.**
  With the array empty, `for crate in "${crates[@]}"` expands to nothing, the loop body never runs, and the
  job reports success over zero crates — which is the shape `gate-shape-contract` was retired for reaching,
  enumerating zero gates and reporting clean over all of it. The lists this replaces could not do that, so
  removing them without the floor would have traded one silent failure for another. Both directions of the
  guard were run standalone, and the job now prints the set it derived.

- **A fixture's premise moved into the fixture where it could, and is written down where it could not.** Two
  absent-reference probes named real files this window deleted while their neighbours use `zzz` sentinels.
  One was the neighbours' shape and became one. The other **cannot be**, and that asymmetry is now recorded
  at the row rather than left for the next reader: `bare_basename_offence` reports a bare run only when
  `deleted_outside_changes` holds it, because otherwise a bare word is a word and not a reference — so a
  sentinel leaves that row inert, measured directly by renaming it and watching the direction report it seen
  by nothing. The concern that re-adding either script would *silently* repoint a probe does not hold in
  either case: both directions assert that every planted form is seen, so a re-added file turns one red and
  names the form. What is genuinely fragile is the explanation — a comment naming the deleted path is itself
  a stale reference to it, and the row survives only because a backtick inside a string literal is written
  `\``, which the reference form does not match. The comment therefore does not spell the name it explains.

- **A `SHALL` that nothing observed is a convention now, and says so.** `repository-checks` required *the
  declaration SHALL sit beside the pin it bounds* for the interpreter support window, and nothing held it:
  the reader takes the workflow's whole text and a day and never computes a line index. Measured — the
  declaration moved 92 lines from the pin and outside every job, and the reaction stayed green 3 of 3. It was
  written in the same change as the reaction and neither reacted to nor filed, which is the class
  `BACKLOG.md`'s *Every normative SHALL either has a reaction or is a declared bound* entry tracks, recurring
  once more in a window whose own subject was closing it. The repair is not a reaction: defining how near
  counts as *beside* would put a threshold on a property whose whole value is that a reader meets the two
  together. So it takes the disposition a branch name already has — a convention for humans and agents,
  stated as one rather than as law — and the requirement now says what the reaction *does* hold, which is
  that the major and the date are read together and compared with the pin.

- **The Definition of Done names the host tools it assumes.** The list is *a corpus with a name* — the phrase
  is a completeness claim about a set someone can count — and its prerequisites were counted by nobody:
  `cargo`, `git`, `npm` with a Node in `engines` range, `cargo-deny`, `jq` and `gh` must be on the path, and
  only the Node range refuses when unsatisfied, through `.npmrc`'s `engine-strict`. `AGENTS.md` now says so,
  and says in the same breath that this half has **no reaction** behind it, because a list of prerequisites
  reads like a guarantee. What earns it the place is that a missing tool does not present as one: measured
  2026-08-23 with `jq` absent, 15 of `merge_workflow`'s 30 cases failed reporting
  `bin/gh: line 77: jq: command not found`, so an operator reads fifteen findings about the subject when the
  state met was one absent binary. The half with teeth stays where it was — `BACKLOG.md`'s WATCH entry, whose
  repair is the fixture declaring what it needs and stopping before the subject.

- **The window's breaking-change classification has one owner again.** `BACKLOG.md`'s version-horizon
  paragraph classified the window itself — naming the bare-principal resolver closure as the one item that
  earns the minor, and calling the rest of the window's public surface additive and therefore free. Both
  halves went stale inside the window: another entry here earned the `**BREAKING**` mark, `Outcome::Clean`
  gaining the subject it was measured over, and it is part of the very surface that sentence called additive
  — with a `### Migration` bullet reading *the compiler names every site*. Two owners for one fact, and the
  one with no producer drifted. The paragraph now states the version **consequence** (pre-1.0, an adopter
  having to act earns a minor whatever the diff size) and points here for which entries those are, rather
  than re-deriving the answer in prose. Nothing about the release changes: `0.5.0` was already right, and the
  marks and migration steps it rests on were already complete.

- **Two ways the interpreter support window read a date it could not read are closed, and both were
  accepted before.** The reaction's date reading was `filter_map(|part| part.parse::<i64>().ok())` followed
  by a destructure of three and a `1..=12`/`1..=31` range check, and each half admitted an input it then
  answered for. Measured against the released reaction: `# NOT-BEYOND: 24 2028--4-30` passed, the empty
  field between the doubled delimiter dropped and the three survivors read as `2028-04-30`; and
  `# NOT-BEYOND: 24 2028-02-31` passed, in range and off the calendar, which the civil-calendar arithmetic
  answered for as `2028-03-02`. The refusal for a malformed date said *names no day* while checking neither.
  `kanhe::reading::date` now answers both as one question of one input: the field count comes from a
  division that does not collapse a repeated delimiter, the widths are held at four-two-two so one date has
  one spelling, and the day is held against its month's length with the leap rule in all three directions.
  `Civil`'s fields are private so `date` is the only way in — a struct literal would otherwise build
  `2028-02-31` and answer `days_from_epoch` for it, which is the defect made unconstructible rather than
  caught. This also **narrows** one accepted spelling: `2028-4-30` was read as April 30th and is now
  refused, because `YYYY-MM-DD` is the declared form and admitting both makes the reader accept two
  spellings while its own message names one. The civil-calendar arithmetic moved to
  `crates/kanhe/src/reading.rs` with the awkward-days table that caught a real off-by-one in it.

- **A registered refusal site names one branch, which cost a repair mid-change and is worth recording.**
  The first cut of the date reader gave two branches the same identity twice over —
  `#date-not-the-declared-shape` for both a width failure and an unreachable parse arm, and
  `#date-names-no-day` for both a month and a day out of range. `refusal_register` refused it in so many
  words: *a direction citing one vouches for the others*. The month now carries
  `#date-names-no-month`, and the unreachable parse arm is **gone** rather than re-identified — three runs
  of ASCII digits of established width are read rather than parsed, so there is no failure arm to answer
  for. An unreachable arm is either a fail-loud over an impossible state, which the minimalism bound
  forbids, or a registered site no direction can observe, which is a declared gap where there is no gap.

- **A reader that cannot understand its input now refuses it, and the field count is the refusal.**
  `crates/kanhe/src/reading.rs` is the sibling of `selection` for the other half of one measured class:
  `selection` answers *how many candidates are there*, and this answers *could this be read at all*. They
  stay apart because they are different mechanisms, and one instrument for two of them is the shape this
  repository removes on sight. `fields::<N>` divides a text and refuses a count it was not asked for, so
  the survivors of a `filter_map` can no longer destructure as if nothing was dropped; `Sep` separates a
  delimiter whose repetition is a **defect** from whitespace a writer spaces freely, which is the
  distinction that made a doubled `-` invisible. The refusal is a cannot-judge naming the count that
  arrived, and the form to write instead comes from the caller — a shared reader does not know what form
  its caller wanted. `interpreter_support_window` reads its declaration's two fields through it, replacing
  three `next()` calls that read two and checked a third was absent: the same reading spelled longer, and
  making no claim about how many arrived. Exit classes and the set of shapes that react are unchanged; only
  the refusal's wording moves, now naming the count. The date half of the same reader is a separate change.

- **The interpreter pin carries the window it is good for, and a reaction holds it.** Pinning the interpreter
  exactly bought the property the step claims and created a hand-maintained pin nobody refreshes — filed
  beside the action SHAs under an entry whose own promotion trigger reads *a second ecosystem arriving whose
  pinning would want the same answer*. That trigger fired in the commit that added the Node pin to the entry
  naming it, and was not read. The half with teeth is not lagging within a major, which `engines` plus
  `engine-strict` already bound; it is running an interpreter past the point its major is maintained, which
  nothing reacted to. `.github/workflows/ci.yml` now declares `NOT-BEYOND: <major> <date>` beside the pin and
  `interpreter_support_window` refuses when that declaration is absent, doubled, unreadable, speaks for a
  major the workflow no longer pins, or has been reached. The declaration is a **commitment of this
  repository** rather than an assertion about Node's calendar: nothing offline can hold a vendor's release
  schedule, and a claim about what this tree will do needs only the file it is written in. The reading is a
  pure function of the workflow text and the day, so every refusal is constructed against a supplied date
  rather than waited for — a bound written to sit dormant for years is otherwise one nobody has seen work.
- **The interpreter running the pinned validator is pinned to a version, not a major, and the bound around it
  stops rather than warns.** `node-version: '24'` resolved whatever 24.x the runner's mirror carried that day
  — a repointable dependency inside the step that exists to close repointable dependencies, one field
  narrower — while `engines.node: ">=24"` left the upper end open and npm, measured at 11.13.0, warns past an
  unsatisfiable `engines` and exits 0. It is `'24.16.0'`, `">=24 <25"`, and `.npmrc`'s `engine-strict=true`;
  the hand-maintained half is filed with the action SHAs under the same accepted debt. The comment claiming
  the major was chosen by what the lock requires is corrected: the lock's floors are `>=20.19.0` and
  `^20.20.0 || >=22.22.0`, both of which Node 22 satisfies, so the major is a lifecycle choice and now says so.
  `.npmrc` is classified in `FORMATS` as a `#`-commented format, which the format sweep required before it
  would let the file exist — a new tracked format is read by nothing until it is named there.
- **The two arm properties now scan disjoint surfaces, so neither testifies for the other.** The guard's own
  call carries the token the consumption scan reads — `require_a_value "$#" "$1" "${2-}"` satisfies both tests
  on one line — so for any arm using that form the two agreed by construction and the over-refusal direction
  was dead: a non-value arm given a guard by mistake refuses the *following flag*, and nothing saw it.
- **The builder's own entry in the hermetic listing is matched exactly and derived from its constant.** Asking
  whether a line *contained* `core.excludesfile=/dev/null` admitted an ambient `core.excludesfile=/dev/null-evil`
  arriving on the channel the builder itself writes to — measured, with the case passing green while an
  unnamed channel set the one key this surface exists to hold. The exactness rule reached the repository's
  origin last round and not the builder's; it now covers the whole entry of both.
- **A reading that cannot attribute what it found now fails rather than shrinking its set.** The reader that
  takes the wrapper's value-taking arms from `scripts/publish.sh` used the guard request as its sole marker
  and dropped, silently, any call it could not attribute to a pattern line — the test required a leading
  `--`. Measured: an arm spelled `-j)` asks to be guarded, is dropped here, is absent from the declared
  literal too, and the both-ways equality holds over two sets that agree by both missing it, with that arm's
  value-position refusal never run. Each arm is now read for two properties that must agree and are not each
  other's evidence — it asks to be guarded, and it consumes the following argument — and a guard request
  belonging to no arm stops the reader. Three directions, each naming its own half.
- **Both members of the hermetic builder's configuration channels are now delivered, not only listed.**
  `GIT_CONFIG` was added to the cleared set with nothing constructing it: measured, deleting it from that set
  left the whole suite green. The ambient environment now carries both channels — one control each, because
  `GIT_CONFIG` replaces the listing and suppresses `GIT_CONFIG_PARAMETERS` entirely — so removing either from
  the builder makes it appear in what is classified.
- **The hermetic listing's classification matches an origin exactly rather than by a fragment the ambient
  side chooses.** A substring test for the repository's own config admitted every line of a file `GIT_CONFIG`
  named `…/foreign.git/config` — that variable's most ordinary use — `core.excludesFile` included, the one
  setting the whole surface exists to own. The case then fell through to the absence assertion and reported
  what was missing instead of what arrived. git renders the repository's own origin relative to the directory
  it runs in, so exactness costs nothing.
- **The release date is written at the cut, and the ritual did not say so.** The dated section for the version
  under preparation carries a date nothing compares until the `release: X.Y.Z` commit exists, and it drifted
  twice: one release was prepared four days behind its cut, and this section stood six days behind the branch
  tip. `AGENTS.md` now states the step where the value stops being a guess — the date is the last edit before
  the cut, not one made during preparation.

  **The earlier check is declined rather than missing**, and `BACKLOG.md` carries why. Comparing the dated
  section against `HEAD`'s date during preparation would have caught both instances the day they appeared — a
  cut happens at or after `HEAD`, so an earlier date is already wrong. But preparation spans days, so that
  reaction turns red on the first commit of every new day and asks for an edit whose value is a guess: right
  at one moment and wrong the rest of the time, which is the trade this repository already declined for the
  Definition of Done's untracked-file guard. What holds the property is
  `release-coherence#release-date-disagrees-with-its-commit` at the snapshot, so a stale date cannot reach a
  release — it fails the cut, and the cost is one retry rather than a wrong published date.

- **The one row of the doc-comment table a sweep can enumerate had no sweep.** `AGENTS.md` classes a relative
  anchor — a passage pointing at *when* by counting rather than by naming a moment — as neither an observation
  source nor provenance, *stale the moment the window closes*, and says in the same breath that this is the
  one row of that table a sweep can enumerate; the others need the criterion applied per site, which is the
  prose instrument this repository has declined three times. The row a sweep could have had did not have one,
  and nine passages carried anchors across `.rs`, `.sh` and `CODEOWNERS`.

  **Each instrument found what the one before it could not, and the sweep was no exception.** An adversarial
  review named six passages and a `git grep` found a seventh; the first form of the reaction found eight; a
  second review found the ninth, in the reaction's own blind spot. `.github/CODEOWNERS` was missed by both
  humans — it is not a format a reader thinks to grep — and `scripts/publish.sh`'s instance is invisible to any
  per-line search, because the phrase wraps: `for a` ends one line and `window` opens the next.

  **The blind spot was one glyph wide.** `FORMATS` declares `.rs` as a line comment opening with `//`, which is
  correct for what that declaration owns — and `///` and `//!` both open with `//`, so stripping the declared
  marker leaves `/` or `!` at the front of the line's contribution. That glyph lands inside the joined phrase,
  which is verbatim the failure the same function's comment describes for an unstripped `#`. Measured:
  `crates/kanhe/tests/publish_source.rs` carried a wrapped `one commit` / `ago` and was reported clean, in a
  tracked file inside the corpus. The marker's extension is trimmed after it, so a Rust doc comment reads like
  any other — a trim rather than a longest-first table, since the table would be a second declaration of what
  `FORMATS` already owns.

  **And the sweep shipped claiming a negative fixture it did not have.** Its own doc said it was split from the
  check *so a negative fixture can call it*; nothing called it, so the only thing holding the wrap was the tree
  happening to contain a shell instance while the corpus's majority shape went unread.
  `a_wrapped_anchor_reacts_in_every_marker_shape` now holds all three marker forms and the executed line that
  must not react — with the extension unstripped it reports one of three, which is the state that shipped.

  The offence names the line the phrase ends on rather than the line the passage began on — otherwise a wrapped
  file header reports line 1, and a shell script's `#!` opens the run, so every offence in one would have named
  the shebang.

  **And it found one in its own documentation.** The first form of the declaring constant explained why a
  member was admitted by writing that member out, which put a live instance in the corpus — verbatim what
  `projection-register` records about a check whose subject is text. The list is now the single owner and the
  prose does not restate it.

- **The pinning argument was applied to every dependency except the one that runs them.** The workflow's own
  header argues that a repointable dependency resolves fresh executable code on every run: hence every `uses:`
  naming a commit, and the npm tree reproduced from a committed lock with `--ignore-scripts --no-install`. The
  **Node** executing that tree was whatever the runner image carried — no `actions/setup-node`, no
  `engines` field, and nothing in the specs, `BACKLOG.md` or `AGENTS.md` covering it. GitHub changing the
  image's Node major changes the bytes that run the validator, which is the claim-wider-than-its-reaction shape
  this repository removes on sight.

  The major is chosen by what the lock requires rather than by preference: `@fission-ai/openspec` declares
  `>=20.19.0` and `posthog-node` `^20.20.0 || >=22.22.0`, and Node 20 left support before this was written.
  `package.json` gains `engines.node` naming the same floor, so a local run that cannot satisfy it says so
  rather than differing silently from CI.

  **A pin whose comment is wrong is not a pin**, and the first form of this one had that defect: the
  `actions/setup-node` release is `v7.0.0`, and the SHA first written was `v5`'s. Every `uses:` in the file
  was then re-resolved against its own comment — `checkout` at `v5.1.0`, `cargo-deny-action` at `v2.1.1`,
  `setup-node` at `v7.0.0` — and all three agree. Nothing holds that agreement, deliberately: resolving a tag
  needs the network, and `BACKLOG.md` already carries refreshing these as a human act.

- **The case built to find unnamed channels could not name the next one.** It filtered git's report to
  command-line entries and required each to be the builder's — which reads a channel that **adds** to the
  listing and is blind to one that **replaces** it. Measured with `GIT_CONFIG` naming a file: git lists that
  file alone, command-line entries number zero, and the case failed on emptiness saying *this read is not
  about the builder* — naming neither the setting that arrived nor where to close it, which is exactly the
  half of its own scenario that does the work.

  Every line is classified now against the two origins the builder admits — what it wrote, and the
  repository's own config — and the absence of the builder's setting is asserted **after** that, so a
  replacing channel is named by its content rather than by what went missing. Negative run: it reports
  `["file:/tmp/…\tprobe.key=INJECTED"]` and where to close it.

  `GIT_CONFIG` is cleared alongside, for a reason recorded rather than folded in: it does **not** move a
  judgement's reads — measured, `status --porcelain --untracked-files=all` reports an excluded file with and
  without it — but it redirects a **write**, so `git config` in a fixture builder lands outside the fixture and
  the commit after it fails for want of an identity. Fail-loud, like the object-directory pair.

- **A configuration channel parallel to the one the builder occupies was open for every caller.**
  `hermetic` empties git's config files, occupies index 0 of `GIT_CONFIG_COUNT` — which is what closes the
  indexed channel and, through it, the ignore channel — and clears the three repository selectors.
  `GIT_CONFIG_PARAMETERS` is none of those: git parses it independently of the count, so occupying index 0
  does nothing to it.

  Measured on git 2.53.0 against the builder's full environment: `'core.excludesFile=/tmp/x'` on that channel
  makes `config --get core.excludesFile` answer `/tmp/x`, and `status --porcelain --untracked-files=all` stops
  reporting a file that path excludes — **the read `publish-source-integrity#worktree-is-not-clean` rests on**,
  in front of an upload that can be yanked and never replaced. And it is ambient in the ordinary sense: git
  exports it itself, measured — a `pre-commit` hook under `git -c probe.key=SET commit` sees
  `GIT_CONFIG_PARAMETERS=['probe.key'='SET']`, and sees it unset without the `-c`.

  **The list was the defect, not the missing name.** Three rounds widened this builder by name — the config
  files, the selectors, the object pair — each after someone measured that variable, and a list grown that way
  is as complete as the last person's memory. `no_ambient_configuration_reaches_a_hermetic_command` asks the
  other way round: it runs the builder in a **child that inherits** an ambient environment and requires every
  configuration git reports from the command line to be one the builder wrote. A channel nobody has named
  shows up as an extra line. Inheritance is the delivery because setting the variable on the builder's own
  command would overwrite the removal and test the case's last statement.

  One interaction worth recording: the ambient settings deliberately do **not** name `core.excludesFile`. The
  sibling ignore sweep decides whether a file closed that channel by whether the file names the setting, so an
  attack string spelling it reads as this file having neutralised it — and takes the channel-control exception
  down with it. Measured; the sweep says so.

- **The refusal that closed that explained cargo's mechanism, and the mechanism differs per flag.** It told
  the operator *its value is X, which cargo reads as an argument of its own* — true of
  `--package --no-verify`, and false of most of what it stops. Measured: `--jobs --allow-dirty` has cargo
  consume the value and fail later with `could not parse --allow-dirty`; `--registry --config` is refused by
  clap with `a value is required for '--registry <REGISTRY>'`. Three mechanisms, one sentence, so the sentence
  was wrong twice — in a repository whose own commit reads *the refusal says what to do, not what is true*.

  It states the wrapper's own rule now, which holds for every arm and does not expire with a cargo version:
  this script does not accept a value beginning with `-`.

  **And the rule's sacrifice is declared rather than silent.** cargo documents a negative job count and
  measured, `cargo publish --jobs -1 --dry-run` packages and verifies normally; this wrapper refuses it.
  Admitting it needs a per-arm rule — a leading digit means nothing for `--package` — and a rule that differs
  per arm is what one shape check exists to avoid. `repository-checks` carries it as a stated bound with its
  own scenario, so a refusal of a call cargo documents is on record rather than discovered.

  Declaring it exercised the machinery that exists for exactly this, and the chain is worth recording:
  `bound_register` refused the prose until a scenario carried it; the scenario's `PINNED-BY` named a direction
  that did not construct `--jobs -1`, so the case gained the row that makes the citation true;
  `observation_bound_model` then refused a bound classified nowhere, so `bounds.rs` carries its extent —
  `OverReacts`, the safe direction, with the reason; and `census` refused `BACKLOG.md`'s typed `91 bounds`
  against an enumerator now producing 92. Four reactions, none of which had to be remembered.

  **The cross product's own input was a hand-copied list.** Its six value-taking arms sat beside the parser
  with an independent owner, so adding an arm without adding it there left the reaction green over a set that
  no longer described the wrapper — the shape the cross product exists to close, one level up in its own
  inputs. The arms are read from the wrapper now, by the marker that decides it there (`require_a_value`), and
  held against the literal both ways. Negative run: a seventh arm reports `unexpected ["--token-file"]`.

- **A refused flag reached `cargo publish` by sitting in the one selector the wrapper admits.** The
  `--package` arm checked that *something* followed and nothing else, and cargo does not consume a
  flag-shaped value. Measured on cargo 1.96.0: `cargo publish --package --no-verify` packages **without
  verifying** — byte-identical to passing `--no-verify` alone — and exits 0 with no complaint about a package
  by that name. So the two flags this script's header argues at length for refusing, `--no-verify` and
  `--allow-dirty`, were both reachable, in front of an upload that can be yanked and never replaced.

  Held for **every** value-taking arm rather than the one measured to leak. cargo's handling differs per flag
  — some consume the value and fail later, some are refused by clap — but that is a fact about one version's
  error paths, and a wrapper in front of an irreversible act does not rest on the tool failing correctly. The
  check is by **shape**: a leading `-` is what makes cargo read a token as its own argument, so a flag nobody
  has classified is refused for the same reason as a named one. One helper, both arms.

  **The axis, not the arm, is what was missing.** Both wrappers' directions read single arguments — each
  refused flag alone, each admitted flag with a well-formed value — so the interaction between arguments had
  no owner. `a_refused_flag_cannot_sit_in_an_admitted_arguments_value_position` crosses them: six value-taking
  arms against five classes of refused argument, including one nobody has classified.

- **The severity that repair set was right for three keys and rested on a file count for two.** Withdrawing
  the false-negative framing said *a job acquiring `if:` reports `SKIPPED`, the silent arm refuses, nothing
  reaches a merge either way*. True for `if:`, `needs:` and `continue-on-error:`, which move a **check's
  conclusion**. `paths:` and `paths-ignore:` work the other way: a workflow-level filter stops the workflow
  **triggering**, so its checks are absent from the rollup rather than reported skipped.

  That still refuses today, and by arithmetic rather than design — `ci.yml` is the only file in
  `.github/workflows/`, so a workflow that does not trigger leaves the rollup empty and the *no workflow has
  claimed this head* arm stops the merge. Add a second workflow and a filtered-out `ci.yml` contributes
  nothing to a rollup the other makes non-empty and green: the merge proceeds with none of its checks having
  run, and a missed `paths:` filter is a false negative again.

  So a claim about the wrapper was resting on the number of files in a directory — in the paragraph that
  prices everything else. The severity is split per mechanism where the keys are declared, and the count is
  held by `a_missed_path_filter_costs_a_delay_only_while_one_workflow_exists` rather than assumed:
  `.github/workflows/` gaining a second file fails, naming what changes and where the severity is stated.
  `BACKLOG.md`'s promotion trigger becomes that second file rather than a sixth position in the reader.

- **A refusal asserted a fact about the tree, and holding that fact cost seven review rounds.** The
  no-evidence refusal told an operator *no job in this repository's workflow carries `if:`, `needs:`, `paths:`
  or `continue-on-error:`* — true when written, false the moment anyone adds one, at which point the wrapper
  says something false about the tree the operator is standing in. It was justified on the ground that the
  classification filtered on the same claim.

  **It did not.** The `case` in `require_ci_green` reads a check's conclusion and nothing else, and
  `scripts/merge-pr.sh` named the workflow in exactly one place: that sentence. The premise was accepted from
  a review and never checked against the script, and it is what put a line reader over YAML in front of a Core
  Contract obligation. Five positions in that reader across seven rounds — two indentation widths, a key
  scope, and the flow form at two levels — two of them failing open, every one correct to repair and none of
  them the thing that was wrong.

  The refusal states what to do now: look at why the check did not run, and if a job may legitimately skip,
  move that conclusion back beside `SUCCESS` recording which job and why — the way the `EXPECTED`
  classification beside it already states its own reason. A claim about what to do next needs nothing holding
  it and buys the same thing.

  **The reader is kept and reframed rather than deleted.** It decides *when* an operator learns a job may
  skip, not *whether*: a skipping job reports `SKIPPED` and the wrapper refuses regardless, so what it buys is
  that the local Definition of Done says so first, with the key and its line, instead of a round trip through
  CI. Its remaining blind spots are recorded at that severity — minutes, not false negatives — and
  `BACKLOG.md`'s promotion trigger changes accordingly: a sixth position no longer earns a parser, since a
  sixth position costs a delay.

- **The rule was general and the change was one branch.** *Entering a block and reading it are not exclusive*
  was stated of the top-level key and applied there, leaving the same open direction one level down: a
  block-form `on:` whose event is written in flow form — `push: {branches: [main], paths: ['src/**']}` — which
  is the **more ordinary** of the two spellings. The reader now asks *what does this line open, and what does
  it still carry* through one implementation used wherever that question arises.

  **And the key was recognised as a substring in one of three spellings.** `starts_with` twice, `contains`
  once — and the `contains` form reacted to a trailing comment, so `# no paths: filter here` named a filter
  that is a word in a sentence. Splitting a flow body on its separators puts every key at the start of its own
  segment, and a block-form line is the degenerate case of the same rule, so one function answers for both
  forms at both levels. That is the same distinction `merge_message_gate` already draws between a line that
  carries a trailer and a sentence that names one.

  Fifteen fixture rows, six asserting no reaction; both negative runs land on their own row.

  `BACKLOG.md` records what seven rounds of this measured: the reader has yielded one more position per round,
  two of them failing open, and the entry that already carries this class for the TOML readers now carries it
  for this one — on the **cheap** side of its own dividing line, since `workflow_shape` is a test and a YAML
  parser lands in the `[dev-dependencies]` table `syn` already occupies rather than crossing the self-law
  boundary its TOML half is blocked on.

- **Entering a block and reading it were treated as exclusive, and that one was open.** The reader set its
  scope from a top-level key and moved to the next line, so the rest of that line was seen by nothing — and
  YAML's flow form puts the whole block there. Measured: with `on: {push: {paths: ['src/**']}}` the direction
  reported the premise intact over a workflow carrying a real path filter, while `require_ci_green` went on
  telling an operator that no job here can legitimately skip.

  **This is the first of the reader's defects that failed open.** The depth assumptions and the scope
  assumption each refused too much — a false refusal an operator meets and argues with. This one refuses too
  little, silently, which is the direction the whole guard exists to close. Narrow: `on: [push, pull_request]`
  is the common flow form and carries no filter, and GitHub's own documentation shows the block form for
  `paths`. Narrow and open is still open.

  **The job side keeps the asymmetry deliberately.** A flow-form `jobs: {alpha: {…}}` leaves no line ending in
  a colon at the name depth, so no job is found and the set equality says so — measured, `missing
  ["examples"]`. Reading that line the same way would turn a loud failure into a quiet pass unless the flow
  body were parsed, which is a YAML parser rather than a line reader. It is pinned as a fixture row so the
  asymmetry is on record rather than an omission someone later closes into silence.

  Thirteen fixture rows now, five of which assert no reaction.

- **The last of the reader's three assumptions about the file was a content claim, not a width.** Two were
  indentation and are gone; the third read `paths:` and `paths-ignore:` at any depth on any line, before the
  position gate the other three keys pass through, justified as *those two keys have no other meaning anywhere
  in it*. That is a claim about one file's current content rather than about the keys — the same kind of
  assumption the widths had just been rewritten to remove, one member later in the same list.

  Measured: a step input named `paths`, the shape `dorny/paths-filter` and `tj-actions/changed-files` take,
  made the direction refuse and tell a maintainer *a job can now legitimately skip* about an input that moves
  no job's conclusion. The reaction fails closed, so the cost was a false refusal rather than a merge — which
  is why the pair is scoped rather than dropped. A path filter is a **trigger** condition: it lives under
  `on:`, quoted or not, since YAML 1.1 reads a bare `on` as a boolean and both spellings name the block.

  Three fixture rows cover it — a filter under `on:` reacting, the same under a quoted `"on":` reacting, and a
  step input not reacting — and both negative runs land on their own row: matching at any depth reports the
  step input as an offence, and recognising only the bare spelling misses the quoted block. `repository-checks`
  now states the two key classes and their positions rather than describing one of them.

- **The reader holding that premise assumed two indentation widths, and one of them lost keys silently.**
  The direction's own documentation named three ways it could lose a job; two were closed in the round that
  wrote them and the third was not. It is the one the set equality cannot see: a job key at a depth other than
  the assumed four loses a **key**, not a **name**, so the job is still found, the equality still holds, and
  the forbidden key is simply never examined. Measured on a document `pyyaml` accepts — with `if:` among a
  job's six-space keys the reader found the job, held the equality, examined no key, and reported the premise
  intact, while `require_ci_green` went on telling an operator that no job here can legitimately skip.

  Both widths are derived from the document now rather than declared, which removes a literal instead of
  adding one: the job-name depth is whatever the first structural line under `jobs:` sits at, and each job's
  key depth is whatever its own first deeper non-sequence line sits at. YAML fixes neither — only consistency
  within a mapping — so this reads the file instead of assuming a house style.

  The reader is split out so a fixture can hand it shapes the tracked workflow does not have, which is the
  half that was missing: one real file cannot ask the question. Seven rows, including the two that must
  **not** react — a `steps:` entry's own `if:`, and a sequence item written at a job key's depth. Both
  negative runs land on their own row: assuming the key depth reports `carried [], expected 1`, and assuming
  the name depth reports `read {}, expected 2 job(s)`. The second row was absent when the fixture was first
  written, which is the same omission one axis over.

- **The no-evidence refusal asserted a property of the workflow, and nothing held it.** The classification
  and the diagnostic both filter on *no job in this repository's workflow carries `if:`, `needs:`, `paths:` or
  `continue-on-error:`* — a claim measured once and then load-bearing in two directions at once. Add one of
  those keys and the sentence is false at the moment it is printed, while the refusal it justifies goes on
  happening: a legitimate skip refused with a message asserting legitimate skips are impossible. `AGENTS.md`
  names the rule — *something downstream filters on the claim, so declare it and hold it to the producer both
  ways* — and the producer was a tracked file a sibling direction already reads.

  `no_workflow_job_can_legitimately_skip` reads the **job** level rather than the whole file, because a
  `steps:` entry may carry `if:` without the job's conclusion moving and refusing that would refuse correct
  code. It refuses a corpus in which no job was read: a renamed block or a re-indented file otherwise
  satisfies *none of them carries a forbidden key* while having parsed nothing. Both directions were run —
  an `if:` on one job names it by line, and renaming `jobs:` trips the vacuity guard.

- **A skipped check was read as agreement, and the fixture standing for a green suite carried one.**
  `require_ci_green` classified `NEUTRAL` and `SKIPPED` beside `SUCCESS` with no measurement, while the
  `EXPECTED` classification was reasoned onto the unfinished side at length — *reading it as agreement would
  merge past a required status that never arrived*. The identical argument covers a check that did not run: it
  produced no evidence, so agreement merges past whatever it would have said. Measured on this repository
  rather than argued from GitHub's vocabulary — no job in `.github/workflows/ci.yml` carries `if:`, `needs:`,
  `paths:`, `paths-ignore:` or `continue-on-error:`, so a skip here cannot mean *legitimately not applicable*;
  it can only mean the workflow changed or the run was interfered with.

  **The premise was invisible because a fixture held it.** The default rollup body — the one standing for a
  green suite, used by every success-path direction over this wrapper — carried a `SKIPPED` beside a
  `SUCCESS`. So those directions asserted the classification as an unwritten premise, and withdrawing it
  failed four of them at once, none about CI, each having reached its own subject only because the fixture
  agreed on the way past. A fixture that encodes the property under test makes the suite agree with itself
  in place of the subject; `repository-checks` now states that as a requirement.

  Four states rather than three, and the fourth has its own refusal because the operator action differs: an
  unfinished check is waited for and a skipped one is investigated. Negative run, with only the wrapper
  reverted: the case fails on `left: Some(0)` against `right: Some(2)` — the old wrapper does not merely
  misclassify, it reaches `gh pr merge` and returns success.

- **A repair for an undeclared stop left an undeclared stop.** The selector repair recorded that
  `GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES` had been measured only against a ref read, and
  that the object read — the tag body the signature check reconstructs — was *filed rather than guessed at*.
  Nothing was filed: zero occurrences in `BACKLOG.md`, in any spec, in either register. So a private doc
  comment was again the only carrier of a stop, which is verbatim the defect that repair had just closed,
  arriving through the repair for it.

  Measured rather than filed, against two repositories whose tag bodies differ: `GIT_OBJECT_DIRECTORY`
  **replaces** the object store, so this repository's own tag object goes missing and the command exits 128 —
  it refuses rather than answering wrongly; `GIT_ALTERNATE_OBJECT_DIRECTORIES` **appends** one, so the local
  object still answers and nothing moves. Neither is admitted, and both reasons are recorded, because a
  negative measurement kept is what stops the set growing by resemblance and stops the pair being
  re-measured every review. Reading a different body through either would need the ref to resolve to an
  object id whose content differs, and the refs come from this repository because `GIT_DIR` is cleared — so
  it needs a collision, not a variable.

  `repository-checks` also now carries the direction's own reach: a construction case establishes that the
  builder marks the variables for removal and cannot establish the composition, since making the variable
  arrive as it really would needs this process's environment mutated. That residue reads where the register
  reads rather than only beside the case.

- **The isolation builder left open the channel that moves which repository git answers about, and the stop
  was declared nowhere.** `hermetic` neutralises git's config files and names `core.excludesFile`, and its own
  table recorded `GIT_DIR` / `GIT_WORK_TREE` / `GIT_INDEX_FILE` as **not** closed — justified by *nothing in
  this tree sets them, zero occurrences repository-wide*. That corpus cannot decide it: the channel is
  ambient, so the variable arrives from outside the tree the sweep read. It is the reader-narrower-than-its-
  claim shape this repository spends four rules closing, sitting in the justification for leaving a channel
  open. And the stop reached no `openspec/specs/*` scenario, so it appeared in neither observation register —
  a reader following this repository's own instruction to check the register before reporting a behaviour as
  a defect would have found nothing to read.

  The consequence sat in front of the one act that cannot be undone. `publish_source_gate::judge` reads the
  worktree's cleanliness, `HEAD`'s subject, the release tag's presence and the tag object through this
  builder; with `GIT_DIR` naming another checkout, every one of those answers about that checkout while
  `cargo publish` packages the directory on disk. Measured against two repositories whose `HEAD` subjects and
  tags differ: `log -1 --format=%s` and `for-each-ref refs/tags` both answered the second, and
  `status --porcelain` reported the second tree against this index under `GIT_WORK_TREE`.

  Closed rather than declared, because closing costs an `env_remove` and refuses no caller. The set is what
  measurement admits rather than every `GIT_*`: `GIT_NAMESPACE` was measured in the same run and
  `for-each-ref refs/tags` still answered this repository, so it is not in it — an entry that closes nothing
  reads as a defence that was never there. `repository-checks` now carries the requirement and its scenario,
  and `a_repository_selector_cannot_reach_a_hermetic_command` holds it: a behaviour control proving the
  channel is real, then the removal read off the builder. That split is stated rather than glossed — making
  the variable arrive as it really would needs this process's environment mutated, which is unsafe in this
  edition and racy against a parallel run, and setting it on the builder's own `Command` would override the
  `env_remove` and test the case's own last statement.

- **A premise its own gate had falsified sat where an operator reads the branching rule first.**
  `AGENTS.md` closed *Branching and release* with *Like the self-describing-commit rule above, this is a
  convention for humans and agents, not a Tianheng boundary or repository check: a branching pattern is not an
  observable architectural fact, so the drift law keeps it out of the constitution.* The reason given reaches
  the **constitution** and stops there, while the conclusion also claimed **repository check** — two things
  this same file distinguishes in so many words, since `crates/kanhe` governs this repository without being
  the product running on itself. And the comparison had gone stale: `merge_message_gate` refuses a subject
  carrying `(#N)` and a body that is a bare commit list, which are the two decidable halves of the
  self-describing rule, so that rule has a gate where its squash instance is concerned while a branch name has
  none. This is the shape `scripts/merge-pr.sh`'s own header already records paying for — *a premise its own
  new code had falsified, left standing where an operator reads it first*.

  Repaired to say what the reason supports, with the branch-name question filed rather than answered.
  Measured for that entry over every merged pull request into `release/0.5.0`: 84 head branches open with
  `change/`, the role `AGENTS.md` records as retired, and seven carry no `/` at all.

- **Thirteen closed entries sat under headings a reader consults to ask what is left to do.**
  `BACKLOG.md`'s governance says a closed item *leaves the live class it was filed under; it does not stay
  there struck through*, and gives the reason — *a class heading is read as a queue and an entry that carries a
  question and its answer at once is a reader trap*. The closed-records section states the same rule from the
  other side. Two statements, and nothing held either: ten entries accumulated under `### READY-PATCH` and
  three under `### WATCH / ACCEPTED / DECLINED / BUILT`, each struck through and each carrying its own answer,
  in the index a reader consults for promotable work.

  `backlog_classification` was the check that could not see them, and deliberately: `live_entries` skips a
  struck-through bullet because a closed entry keeps the `*Class:*` line it had when written, and holding a
  record to today's headings is a falsification this repository refuses generally. Skipping the **comparison**
  is right; that the entry belongs where it sits does not follow.
  `a_closed_entry_does_not_stay_under_a_live_class` asks the other question — not *does its class match the
  heading* but *is it under a class heading at all* — which needs none of the record's own text.

  All thirteen are moved verbatim and in order; the file's line count is unchanged and its lines compare equal
  as a set, so only position moved. The closed-records preamble no longer says each entry was live *when the
  window opened*, which was true of the entries it was written for and not of the ones that closed after it.

- **A wrapper chose its exit class at four sites while the rule says one, and nothing counted them.**
  `repository-checks` requires the classification to be chosen once per wrapper rather than at each site.
  `gate_exit_classes` compared the class *constants* and the presence of `require_one_pass` across both
  wrappers in both directions — so the **identities** could not drift while the **sites** could, and they had:
  `scripts/merge-pr.sh` spelled `printf … >&2; exit 2` inline at the positional selector, the pull-request-URL
  refusal, `require_value` and the body-file guard. Two of the four predated the helpers they should have
  called, which is why they survived the round that converged every `case` arm. Two more carried no
  `merge message:` prefix at all, so the refusals firing on the two commonest misinvocations — no pull request,
  no body file — were invisible to an operator or a log filter keyed on it, and neither had a direction of any
  kind.

  Both wrappers now delegate every stop to a single `cannot_judge`; `scripts/publish.sh` had two helpers each
  choosing the class, which is the same shape one degree smaller and is converged with it. Each file spelling
  its diagnostic prefix once is a **consequence** of that and not a property anything holds — what is held is
  the count of `exit` statements, and a second `printf` above a delegated exit would break no rule. Two directions replace the reading: `each_wrapper_chooses_its_exit_class_in_one_place`
  counts the `exit` statements in executed shell text — exactly two may say `exit 2` (the class helper and the
  gate's own verdict arm) and exactly one `exit 1` — and `every_usage_refusal_carries_the_wrappers_own_diagnostic`
  holds each misconfigured invocation against the pair *(class, diagnostic form)* as a table, so a stop added
  later lands as a row rather than as a finding in the next review. Negative runs, with only the wrappers
  reverted: the first names all seven sites in `scripts/merge-pr.sh` with their line numbers, the second fails
  on the zero-argument row with `got "usage: merge-pr.sh …"` and no prefix.

  Executed text for the counting direction, because both wrappers' comments now discuss `exit 2` in prose and
  a check reading the whole file would count the sentence describing the rule as an instance of breaking it.

- **An ignore file on the machine running the gate could excuse a stale path reference, and silently omit a
  file from a fixture.** `reference_integrity::ignored` asked `git check-ignore` through a bare
  `Command::new("git")`, on the real repository, on the verdict path — and *ignored* there means the offence is
  **not** reported. So an entry in whoever's global `core.excludesFile` or `$XDG_CONFIG_HOME/git/ignore`
  matching a referenced path made the gate report clean over a reference it should have refused: an
  under-refusal whose answer depended on who ran it, in the capability whose stated Purpose is that a
  checkout's verdict does not depend on ambient process state.

  **Repairing that one read is what showed it was never one read.** The shared command builder neutralises the
  global and system config *files*, and `$XDG_CONFIG_HOME/git/ignore` is the default excludes path git uses
  when **no** config file names one — so emptying the files leaves the default in force, and the builder was
  not closing this channel for anyone. Measured, both directions, on a fixture whose only exclusion came from
  an XDG ignore file: the query answers *ignored*, and `git add -A` leaves the matching file **untracked**, so
  every fixture repository in the crate could be built without a file it named, by a file on the machine
  judging it. The builder names the setting now, through the one ambient channel that cannot be closed and
  therefore can carry a setting no config file names; the two judgements whose verdict turns on the answer
  keep the explicit flag as the narrower statement. Pinned by a case comparing an isolated command against an
  unisolated control — the control must leave the channel open, or it compares a value against itself.

  Held crate-wide by `no_judgement_reads_an_ambient_ignore_file`, over every tracked `.rs` under `crates`: a
  file running such a subcommand must either name the setting or start no process of its own, so that every
  command it runs is the builder's. Requiring the flag *on top of* the builder was the first form and it
  refused the pinning control itself — a rule that manufactures a redundancy and calls it a repair. Negative
  runs: the original defect restored verbatim is refused by name; with every named subcommand out of reach the
  vacuity guard fails rather than reporting clean; with the builder's setting removed the pinning case fails;
  and with the excused control's direction renamed away the exception fails rather than going on excusing the
  file. That last one was itself a repair — the exception was first held against the file's continuing to
  spawn, which a *different* test in the same file satisfied, so the guard stayed green with the control
  converted away.

  The subcommand is recognized as a **complete argument literal**, which is both why a diagnostic string
  reading `"check-ignore exploded"` and two doc comments are not mistaken for calls, and why the direction does
  not judge itself by its own marker array — escaped quotes on disk are not the quotes a call site carries.
  Not marked `**BREAKING**`: the mark is for a change that makes an adopter *do* something, and repository
  checks ship in no package, so no recorded baseline anywhere moves.

  What it does not reach, each measured: file granularity rather than per-call, because a per-call rule would
  refuse the one site that was already right, where one wrapper closes the channel for every judgement in that
  file; a subcommand composed at run time; `.git/info/exclude`, which is inside the repository so no setting
  reaches it; and one named spelling losing its reach individually, since the guard holds that *some* marker
  still matches, not each — a per-marker rule is not adoptable while the array admits a form git accepts and
  nothing uses.

- **The refusal reader read `b'\"'` as an opening quote and swallowed the next declaration.** Its
  string-literal scanner knew a `b` prefix on a *string* and not on a *char*, so the `b` before the quote read
  as an identifier, the char-literal arm declined the literal, and the `"` inside it opened a span that ran to
  the next quote several lines down — taking a `fn` with it. Every figure the reader produces holds only by
  what a swallowed span happened to contain, which is why the corpus-wide guard exists; it caught this the
  moment a shared helper was extracted and the swallow started crossing a declaration, having been latent
  before that purely by placement. Pinned by a case in the reader's own corpus, where a swallow shows up as a
  construction counted zero. Negative run: with the arm removed, the case and the corpus guard both fail.

  **Three lists that had to agree, in the direction that reads that corpus.** Each arm spelled its cases
  inline and the completeness join at the end spelled all eight again, so a case could be answered by an arm
  and left out of the join, or joined and answered by nothing. The join derives from three named sets now,
  each used once by the arm that answers it.

  Found by adding a fourth list — a set-equality guard the direction already had, one screenful further down.
  The duplicate reported five cases as unanswered and every one of them was answered, by an arm that gives a
  verdict rather than a count; what disagreed was the new list's membership rule, not the tree. So the finding
  was not the one it looked like: writing a guard for a property already held is how the three-way duplication
  became visible, and the guard itself was the wrong repair and was removed.

- **A test target spawning only through the shared fixture builder was invisible to the guard that names
  spawners.** The detector knew `Command::new(` and `hermetic(`; `hermetic_git::fixture` became a call site's
  spelling and matched neither, so `release_coherence.rs` — every one of whose fixture repositories it builds —
  was never in the declared set. That is the **fourth** narrower form in a row for a requirement that was
  already correct, and the first one to be live rather than caught while widening: the file's own doc comment
  had predicted it in those words. The detector names the process **module** now instead of the functions it
  exports, so every entry point it has and every one it gains is covered, and the missing target is declared
  with what it spawns.

- **The command-runner rule now holds where it is stated.** `bound_register_parse::{search, must}` took the
  program inside the argument list — the shape `hermetic_git`'s own doc calls admissible only where a caller
  chooses the program at run time — and every one of their twelve call sites named a literal, `"git"` or
  `"cargo"`. They take the program as a parameter now, so the list form survives at exactly the two runners
  that genuinely compose it, `pin_bites::run` and `gate_identity::run`, and a rule with an exception nothing
  needed is no longer there for the next caller to read as permission.

- **Three typed counts left where a typed count had just been deleted.** The repair that removed
  *seventeen times* from `refusal::Site`'s doc replaced it with *four call sites, twelve textual occurrences,
  twenty-six counting both constructors* — accurate the day they were written, which is exactly what
  *seventeen* was, standing on the same nothing: `census::sweep` reads tracked Markdown and a Rust doc comment
  is in no reaction's reach. The figures are gone; what stands is the shape, plus the record that the first
  repair reproduced the defect one sentence before concluding where such a figure belongs. A measurement of a
  repair belongs in the dated entry that repair writes — this one.

- **A struct-literal field sat four columns left of its sibling, and `cargo fmt --all --check` was green
  over it.** The `a-workspace-dependency-allowlist-is-not-examined` declaration in
  `crates/shengmo/src/bounds.rs` indented `owner:` at 12 where its own `because:` and the sibling
  `UnderReacts` declaration in the same list both sit at 16. rustfmt declines to reformat that `vec![]`
  element, so the formatter the Definition of Done runs to own `.rs` layout does not reach it — measured,
  not assumed: the check exits 0 both before and after the repair.

  Deliberately **no reaction is added**. A structural scan over `git ls-files '*.rs'`, comparing the
  indentation of struct-literal fields within one brace depth, found this declaration and one false
  positive — a `word:` inside a string continuation, where the real fields agree — so the class has a
  single live instance against a formatter that already governs the rest. Building an indentation check
  beside rustfmt for that is the defensive over-foolproofing the drift law forbids, and the scan is
  described here rather than counted so the next reader can re-run it instead of trusting a figure.

- **The marking rule that says which changes are breaking is paired per section, not per entry.** Filed to
  `BACKLOG.md` as `READY-PATCH` rather than repaired: `require_section_shape` is satisfied by one marked
  entry plus one `### Migration` heading anywhere in the same section, which is how an unmarked breaking
  entry stood beside a marked one in this very release's notes with the gate green. The direction that
  matters — *should this have been marked?* — asks whether a change requires adopter action, which no
  reaction can decide, so the entry works around it and separates the candidates by whether each rule is a
  theorem and whether it catches the instance that produced the entry. Only one candidate is both, it is
  a **named join** rather than a prose comparison — each migration bullet naming the entry it migrates, run
  both ways, the shape the bound register already uses — and it is a capability rather than a tightening,
  which is why it is filed with its trigger instead of built inside this window.

- **A dependency is read in either form cargo writes it, and only where cargo writes one.** The example-pin
  reader looked at no table heading, which cost it both directions at once. `[dependencies.alias]` — a
  detailed table with its own `package` and `version` lines — names no family crate on any single line, so
  the whole declaration was invisible, **renamed or not**: the rename repair one commit earlier closed the
  inline form and left this one open. And a `[features]` key spelled after a family crate was read as a
  version requirement, because nothing said which tables hold dependencies. Tracking the heading closes both
  with one change.

  The pin itself is now a typed four-state answer — declared, absent, unreadable, several — so the compiler
  asks each consumer rather than one of the three collapsing them. It had: `_ => None` reported an **absent**
  `version` as one this reader *could not read*, over the legal path-only form, which is the distinction its
  sibling had just been repaired to make.

  **And the field beside it stayed a sentinel through that same repair.** Which crate a dependency names was
  a `String` with the empty string standing for *a `package` value this reader cannot read* **and** for
  *several `package` keys* — one field typed, its sibling in the same struct left to collapse exactly the
  distinction the typing was for, and not injective either, since a literal `package = ""` is a third fact
  that read as the same state. It is `Named`, `Unreadable` or `Several` now, resolved in one function
  instead of the two byte-identical arms the inline and detailed forms each carried.

  **Writing a direction over that state found a wider one underneath it.** `quoted_value` took the first
  pair of double quotes anywhere in the text it was given, so a value that is not a string borrowed the next
  key's: `alias = { package = xuanji, version = "0.2.0" }` read its package as `0.2.0`, matched no family
  crate, and was skipped in silence. `Unreadable` is the state that type exists for and it was reachable
  only when nothing else on the line was quoted. The quote has to open the value now — which needed one
  contract rather than two, so the two `Cargo.lock` readers that passed a whole line where every other
  caller passed the value split on the `=` like everyone else.

  Four refusals this window shipped with no direction over them — an absent pin, several pins, and both
  package-identity failures — now have one each. The two that already worked were shown to fail by
  neutralising their arm and running the direction against the mutated reader.

  **The migration that reader belonged to had covered one of its two call sites.** `require_internal_pins`
  kept a line-oriented scan while its sibling moved to the shared reader, and the two then disagreed
  observably: against `[workspace.dependencies.xuanji]` with `path` and `version` on their own lines — what
  cargo writes — the scan selected the **path** line, because it carries `path`, `"crates/` and `=`, split
  it at its `=`, and took `path` for the dependency's name. The `version` line carried neither marker and
  was never read. `internal dependency path has no version pin`: a false refusal in front of the release
  gate over a manifest cargo reads correctly. The loop is gone rather than repaired, because two readers of
  one rule is the thing that failed, and `[workspace.dependencies]` is a dependency table with a context in
  front of it exactly as a target table is. Which dependencies are internal is now each one's own `path`
  value rather than the shape of the line it sits on — the same correction the sibling made when it stopped
  keying on the dependency's name.

  **That migration then shipped four refusals with no direction over them**, in the change immediately
  after the one whose own entry above records adding four such directions. A `path` this reader cannot
  read, several `path` keys, a `version` it cannot read and several `version` keys were all reachable and
  all unexercised. They have one each now, shown to fail by neutralising each arm and running against the
  mutated reader — where the unreadable-path half reported the vacuity refusal instead, which is the
  failure that branch exists to prevent.

  The recurrence is the finding rather than the four branches. *A guard is not a guard until it has been
  seen to fail* is stated in `AGENTS.md` and required by `repository-checks`, whose scenarios held only the
  clause about where a check may live; the clause about every refusal having been run against a tree
  carrying the shape it refuses had no scenario and no reaction, and was carried by attention — which is
  what the next entry replaces.

- **A refusal names the site that produced it, and a register holds that a direction observed it.** The
  clause above now has a reaction.

  **Why nothing held it before.** A refusal's identity lived only in its message, and a message is a
  *template* while a direction asserts a *rendering* of it. Five textual predicates were written against
  that gap and measured against the whole corpus; each was wrong in a different direction, over-reporting
  or under-reporting by a different mechanism, and none could tell a branch that was never exercised from
  one whose wording had moved. Whether a branch was observed is a question about running a program, which
  is what `pin_bites` already says about whether a test bites. So the site travels in the value:
  `refusal::violation_at` and `cannot_judge_at` take it, and a direction names the same identity through
  `refusal::expect`.

  Four things are held. A registered site no direction observes refuses — registering is the commitment
  that one does. A citation naming a site nothing produces refuses, because a one-way check is satisfiable
  by doing nothing in whichever direction it does not look. Two branches sharing one identity refuse, since
  one direction's citation would vouch for a branch it never reached. And the count of sites **not yet**
  registered is produced into a projection whose every change has to be blessed, so it falls as modules
  migrate and cannot rise quietly.

  **The migration is visible rather than instantaneous.** Rewriting every site in one change would be one
  diff nobody could read, so the unregistered constructors stand beside their registered siblings while
  modules cross over. Two constructors for one rule is a shape this repository closes, and it is carried
  here deliberately: it is counted, it is named in the projection, and it is deleted when the count reaches
  zero. `merge_message_gate` is the first module across, all ten of its sites registered and observed, with
  `selection` and `census` following, then `gate_identity` and `capability_subjects` — twenty-two sites
  across five modules, each already observed by a direction that now names it. The projection carries what
  remains.

  **The publish gate is the first module where the register found something.** Of its thirty refusal sites,
  twenty-three are observed and **seven are not**: the root that is no worktree, an absent workspace version,
  and five of the signature path — `ssh-keygen` unavailable, the signing mechanism failing its own
  round-trip, a signature block that cannot be read, an extracted signature that is not the tag object's
  suffix, and a signature that cannot be written for checking. They stay unregistered and counted rather
  than registered on a promise, because registering a site is the commitment that a direction observes it.
  This is the gate that stands in front of `cargo publish`.

  Three of the twenty-three were caught being cited wrongly, by the citation itself, at run time. The
  sharpest: an **unsigned annotated tag** does not reach *carries no signature* — it reaches *does not
  verify*, because a tag message quoting a signature block is text rather than a signature. Reading the
  messages said otherwise, which is the failure mode the register exists to end.

  **The release gate is the last module, and the whole ledger is now measured.** Of its sixty-one sites,
  thirty-nine are observed and twenty-two are not, so **twenty-nine refusal branches in this repository have
  never been seen to fail** — seven in the gate before `cargo publish` and twenty-two in the gate before a
  release. That figure is produced by running, and it is the first time it has existed: the question was
  asked five times of the text and answered differently every time.

  Four more citations were caught wrong by the citation itself. Two were a swapped pair — the `[Unreleased]`
  comparison link and the dated release link say nearly the same sentence — and two were sites whose
  identities this change had named backwards: a `release:` subject with no space is a malformed **subject**,
  while `release: next` is a malformed **version**, and both render the same message. A message-shaped
  assertion cannot tell those apart, and every one of them was found by running rather than by reading.

  **Seven of the twenty-nine are closed**, each with a fixture and a mutation run: a workspace version that
  is present and not a version, a crate manifest declaring no package name and one declaring a name this
  reader cannot take, an example pin it cannot take, and all three vacuity guards — no internal path
  dependency, no example manifest at all, and examples requiring no family crate. The three vacuity guards
  were the ones worth having: with each replaced by `Ok(())`, its fixture passed, which is a release
  reported clean over a check that judged nothing.

  **A site is now held or declared, and nothing is untriaged.** The fifteen refusals no direction reaches
  are declared unheld in a table carrying, for each, why a direction over it would test something other than
  that branch, an owner, and a tracker. The register holds the table and the sites in a bijection, refuses a
  declaration naming a site nothing produces, refuses a declared site a direction *does* observe — that one
  is held, and saying otherwise understates the coverage — and requires the count of sites carrying no
  identity to be **zero**.

  **A second was misclassified, and re-reading the reasons against the rule is what found it.** *No tracked
  file was found for any member* was declared with the reason *a repository shape rather than a release
  surface* — an argument about what is worth reacting to, where the table's criterion is that only a broken
  tool reaches the branch. A workspace cargo loads from paths git does not track is a repository, and one
  `git rm --cached` reaches it. It is held by a direction now. A reason that does not fit the category it is
  filed under is the shape a table of declarations is least able to see about itself, since every entry
  reads as a reason.

  Two other reasons were checked the same way and hold. A member manifest outside the workspace root is
  refused by cargo outright — *is not hierarchical*, measured — so no workspace cargo loads can produce it.

  **One of the fifteen was misclassified, and the residual found it.** `cargo metadata failed` was declared
  reachable only through a broken tool. It is not: the line readers judge version surfaces, pins and the
  lock, and none of them resolves a member's path dependencies — so a member declaring `{ path = "../nope" }`
  passes every one of them and cargo refuses the workspace. Measured against a real workspace before it was
  claimed. That is a defect in the judged repository, so the refusal is about the subject, its fixture is
  the defect it names, and it is held by a direction now rather than declared. With the branch disabled the
  gate blamed the tool — *cargo metadata is not JSON* — for a repository that is wrong, which is the
  diagnosis this branch exists to give correctly.

  The escape hatch is deliberately expensive: typed, counted, projected, owned. **And half of what looked
  unmechanizable turned out not to be.** A refusal that refuses as a *violation* may not be declared unheld
  at all: the declaration exists because a refusal about the reading failing can only be reached by breaking
  the machine, and a refusal about the subject has no such excuse — its fixture is the defect it names, and
  a shape that cannot be built is one the branch is not about. Every declared site being a cannot-judge was
  a measurement first; it is a rule now, which is what stops *declare it* from being available to any branch
  whose fixture is merely inconvenient. What remains a reviewer's obligation is narrower: which
  cannot-judge, not whether.

  That rule shipped inert on its first run. The edit recording each site's kind never landed — a mechanical
  replacement written without asserting that it applied, over a chain the formatter had since rewrapped — so
  the map it reads was always empty and the check always passed. The mutation run is what said so.

  **A tracked fixture corpus for exactly this reader had been sitting unread since 10 August, and running
  it found three more holes.** `crates/kanhe/tests/fixtures/refusal_scan/` holds fourteen cases that nothing
  referenced. Three name holes this window had already hit the hard way — a constructor taken by name, a
  longer identifier, a comment. Running the reader over the rest reported **a definition as a construction**
  in three cases: the constructor's own `fn cannot_judge(…)`, its multi-line sibling, and a function that
  merely shares the name `violation` while building something else entirely. A module that constructs
  nothing was being counted as constructing one.

  **The same class was found in a second gate by sweeping the corpus no review had swept.** Five rounds of
  findings all came from reading the code the previous repair produced, while the integration tests — where
  several of these gates keep both their judgement and their cases — had never been read as a corpus. The
  first sweep of it found `workspace_isolation` refusing over a manifest it could not read: mapped through a
  default the file declares no `[workspace]` table, so an unread file is reported as a **violation**,
  measured. Its vacuity guard, which refuses a repository carrying none of the manifests it judges, had no
  case either. Both have one now.

  Two more in the same corpus: building the claimed set refuses over a subject it cannot read — an
  unparseable bullet, or a second `## Subject` section — and neither branch had a case, because all three
  call sites pass the repository's own well-formed specs. Collapsed to an empty claim, which is the
  `unwrap_or_default()` shape that reader's own comment records as the defect it replaced, the claimed set
  comes back short and a change touching that capability's subject reads as filed.

  **An input the wrapper never supplied was judged as a message that disagrees.** The merge gate's harness
  respected the subject's absence — no subject, no merge being made — and read the other three judged inputs
  with a default, so absence arrived as emptiness. The gate answers emptiness on its own terms: an empty
  title and an empty commit list are cannot-judge, and an empty **body** is a violation. A body that was
  never supplied was therefore reported at exit 1, the class this repository reserves for a gate that ran
  and disagreed, and the two sites named *unavailable* could be reached by an empty value and never by the
  absence they name.

  `scripts/merge-pr.sh` carries the other half of this same defect in its own comment and closed it — an
  unreadable body file left its variable empty and the gate judged an empty body. The wrapper's half was
  repaired; this half was not. The process boundary was the last place in this crate where *absent* and
  *empty* were one fact, with `Quoted`, `WorkspaceVersion`, `PackageName`, `Declared`, `Package`, `Tracked`,
  `Failure` and `Site` all drawing it inside `src`.

  **The repair for that put the reader on the other side of its own asymmetry.** `imports_and_rest`
  recognised a `use` item by one textual prefix, so `pub use` and `pub(crate) use` — imports, constructing
  nothing — stayed in the executed text, and the constructor names they carry counted as calls. The register
  refused a module over an import. That is an **over**-reaction, on the side whose failures were supposed to
  be the loud harmless ones, and it is outside the bound declared for the other side: a gate that refuses
  correct source is a defect, not a limit. A `use` item is a visibility and the keyword now, and the space
  after `use` is load-bearing, because `impl Iterator<…> + use<'a>` is precise capturing and this repository
  writes it.

  **And the reader's own limit is declared, because patching shapes cannot terminate.** Every round of
  review has found another Rust shape this reading does not reach — a constructor taken by name, an aliased
  import, an alias the formatter wrapped, a raw literal, a site arriving as a parameter, a wrapped import, a
  struct literal. The case corpus grew from fourteen to nineteen in this window, five of them arriving as
  defects rather than as foresight, with no fall in the rate. Rust's grammar is not enumerable by a scanner,
  so the honest statement is a bound rather than a claim that the next patch is the last.

  What that bound says is narrower than *the reader is incomplete*, because the two sides of it fail in
  opposite directions. A missed **citation** reports a site as unobserved and fails loud. A missed
  **construction** reports clean over a site nothing holds. Only the second is unsafe, and closing it means
  the compiler enumerating the sites instead of a reader — a change to what a site *is*, filed with the
  trigger that names it. Bounded meanwhile by the corpus: `kanhe` ships in no package, so the worst case is
  a refusal site in this repository going untriaged.

  **The repair for those shapes then put two of this repository's own gates in contradiction.** The reader
  that strips imports dropped the line that *opens* a `use` statement and kept every continuation, so an
  import naming `cannot_judge_at` on a line of its own counted as a call with nothing to parse — and the
  register refused a module that constructs nothing. That shape is what `cargo fmt` produces the moment an
  import list grows too wide, and `cargo fmt --all --check` is in the Definition of Done: formatting and
  this gate would have demanded opposite things of the same source. Latent, since every such import in the
  tree is one line today.

  **The wrapper standing in front of the irreversible act declared no window, while its sibling declared
  one.** The publish wrapper runs the source gate, then `cd`s and `exec`s `cargo publish`; between those the
  repository can be altered and the gate's verdict is about the tree as it was. The merge wrapper declares
  exactly this class for its own title and pins its other two inputs by construction — `cargo publish` takes
  no argument naming the commit it must package, so the same pin does not exist here. Found by asking which
  limits are declared on one wrapper and not on its sibling; `publish-source-integrity` had declared exactly
  one bound, about the tag's signer.

  Bounded rather than assumed: `cargo publish` refuses a dirty worktree, so reaching it needs the tree
  amended *and* committed, and the wrapper's window is two statements wide rather than a whole `cargo test`.
  Narrowing is the only available move and it was already taken; what was missing was saying so.

  **And the entry that predicted the stolen doc had its own trigger fire unnoticed.** It recorded the class,
  two instances, the risk and the condition for re-deciding — *a third instance, or any instance on a
  private item after this entry* — and both halves arrived without anything acting on them. The decision was
  re-taken and stands, on a measurement rather than on the count it had been bound to: the only lint that
  closes the private half reports 785 undocumented private items over `kanhe` alone.

  A sweep of every trigger phrased as a count or an instance followed, run on purpose. Two already carried a
  recorded *Not fired.*, one had fired, and the rest had not — a third scanner of the scenario grammar is
  still three, a third governance member has not arrived, a second orphaned corpus has not, and the
  ecosystem the entry names as unrefreshable is the only one of its kind. That sweep is now an entry of its
  own, because what failed is not any single trigger: nothing evaluates them, so an entry states the
  condition for re-deciding and the condition arrives silently. It is the shape this whole window began
  with — a clause with no reaction — one document over.

  The merge harness's own repair then collapsed two facts one layer beneath the one it separated.
  `env::var` answers *not set* and *set but not UTF-8* with the same `Err`, and the wrapper passes the body
  as `$(cat -- "$body_file")` — whatever bytes that file holds. A body that is not UTF-8 was reported as one
  the wrapper never supplied, which sends an operator to look for a variable they did set. Both are
  cannot-judge, so the class was right and the sentence was not. `var_os` separates them, measured against a
  real non-UTF-8 value.

  The doc for that shared walker then spent one change attached to the wrong function. Inserting the
  visibility helper above it put the helper between `imports_and_rest` and its own explanation, so the
  explanation described a function two items away and `imports_and_rest` had none — two valid doc blocks
  fused into one, which is why `cargo doc -D warnings` passes over it and why nothing mechanical caught it.
  Inserting before an item whose doc sits above it is a hazard of the edit rather than of the code.

  The cause was one repair short of its own lesson. The same commit taught the *alias* detector to
  accumulate a statement to its `;` and left its neighbour — fifteen lines up, reading the same input —
  splitting on lines. Where a `use` statement ends is a fact about Rust, and it now has one implementation
  that both readers ask.

  **And a struct literal walked past all of it.** With every field public, `Refusal { site:
  Site::Registered("x#y"), … }` builds a registered refusal without calling anything, and the register
  counts calls: the site would be produced, unheld, undeclared and unreported, while the projection said no
  other construction exists. Detecting literals in text was the alternative; the field the register is about
  is private instead, so the compiler refuses the shape — measured, `field `site` of struct `Refusal` is
  private`.

  **The reader had three blind shapes, and each was invisible to both of its readings.** A constructor taken
  by name and called through the binding, a site arriving as a parameter, and a site written as a raw
  literal are none of them a direct call with an ordinary quoted literal. The parser saw no site; the
  untriaged counter reads the *site-less* constructors and saw nothing either. A real refusal site was
  neither held, nor declared, nor reported missing, and the register said clean. The parse is counted
  against the calls now, so a shape it cannot read is answered as *cannot answer for this module* — and each
  of the three is a case in the corpus.

  **And the field's own doc was the one claim in this window that nothing could falsify.** It said a second
  pair of constructors had been removed once the count reached zero, and that nothing could any longer
  construct a refusal unable to say which branch produced it. Neither was true: the constructors were
  restored in the change that wrote it, `violation("…")` constructs exactly that, and seventeen call sites
  do, while the constructor's own doc in that same file correctly calls the pair a deliberate corpus
  boundary. One file,
  two mutually exclusive statements, and the false one on the field a reader meets first. `Site` is two
  states now rather than a string with the empty string standing for *outside the corpus* — the same repair
  a sibling struct's `package` field was given one cycle earlier, and the type refuses to hold the sentence.
  What is actually true is narrower and is held by a run: no construction under `crates/kanhe/src` lacks an
  identity.

  Reviewing that repair found the same class inside it. The alias detector read **lines**, and a `use` list
  long enough is wrapped by the formatter — the line carrying the alias then begins with the alias rather
  than with `use`, so the reader answered *no alias here* over the exact shape it exists to catch. It reads
  statements now, in executed Rust and opening a line; the first draft of that repair scanned raw text and
  took a sentence containing the word *use* as a statement running to the next semicolon, which then carried
  an alias and a constructor name from two different paragraphs. The wrapped form is a case in the corpus.

  The fourteenth is not a count. An **aliased import** — `use crate::refusal::cannot_judge as cj;` — makes
  every later call invisible to a reader that matches names, and invisible reads as *this module constructs
  no refusal*. The register refuses such a module now rather than counting it, in the class this repository
  reserves for a source it could not read. The direction also holds the corpus and the cases it answers in a
  bijection, so a case added and forgotten is reported rather than sitting unread the way all fourteen did.

  The register reads `crates/kanhe/src`. Several gates live in `crates/kanhe/tests`, where the judgement and
  its directions share a file, and their constructions carry no identity because *which direction observes
  this branch* has no answer when every direction in the file can see it. That is now a **declared
  observation bound** with its own tracker rather than a sentence in a doc comment — a limit stated in prose
  and held by nothing is the shape this whole change exists to end, and it would have been one more.

  The orphaned corpus got a tracker too, with the measurement that keeps it one instance rather than a
  class: every tracked fixture corpus in this repository is referenced by at least one reader, eight of
  eight, swept on the day it was written. A second orphan changes the shape from a sweep run on purpose to a
  reaction that enumerates them.

  Three more, all about the subject rather than about the machine: a release snapshot naming one version
  while the surfaces declare another, a member manifest that is not text, and a workspace version the publish
  gate finds genuinely absent. Fourteen of the twenty-nine are closed and fifteen remain.

  The compiler then said something about the fifteen that no one had asked it. With the last of these
  registered, `violation` became an unused import in the release gate: **every branch still unobserved
  there is a cannot-judge**. That is the split named when this work was planned — a refusal about the
  *subject* against one about the *reading of it failing* — arriving as a fact rather than as a judgement.
  A branch that can only say *this could not be read* is one whose fixture must simulate a broken tool, and
  a fixture that simulates a tool tests the simulation.

  Two more are closed. One needed no new direction at all: the lock reader's unreadable-name refusal was
  already observed, by a direction whose assertion was the phrase *cannot read* — too generic for any needle
  map to attribute, which is the same weakness in message-shaped assertions the register replaces. It cites
  the site now. Its sibling, an unreadable lock **version**, had no direction and has one.

  **And two of the twenty-nine turned out to be branches nothing can reach.** The lock reader re-read every
  workspace manifest's `[package]` name and carried its own refusals for a name that is absent or
  unreadable — over a list that exists only because the example-pin reader resolved every one of those names
  first and refused otherwise. Trying to register them is what exposed it: a branch no input reaches can
  have no direction, and the register asks for one. They are deleted rather than declared, and the
  duplication under them is gone with them — the example-pin reader returns what it resolved, and the lock
  reader consumes it instead of asking the same question of the same input a second time.

  Two holes in the register's own reader surfaced while it was doing this, both making its figure smaller
  than the truth. It counted `violation(` rather than the identifier, so `map_err(cannot_judge)` — a
  constructor used as a value — was a live site invisible to the register built to count sites; found only
  because migrating the module made the compiler object to the import. And counting the identifier over the
  whole file then counted every doc comment naming a constructor, and every `use` list. It reads executed
  Rust with imports excluded now, through the same `region` module the gates use.

  Negative runs: each of the register's four directions was run against a tree carrying exactly the shape it
  refuses — a citation renamed to a site nothing produces, two sites given one identity, a site given a
  capability no spec declares — and each failed on its own perturbation while its siblings stayed green.

  A context cargo writes in front of a dependency table is stripped before the heading is classified, so
  `[target.<triple>.dependencies]` and its `.NAME` form are read like any other. That reading is narrower
  than the bound this window first declared, which named the whole target corpus: the reason it gave — *a
  quoted cfg expression is the grammar a line-oriented reader is likeliest to be wrong about* — was written
  from the hard case and then used to skip the easy one, and a bare triple is two bare TOML keys with
  nothing to guess, because a bare key cannot carry the dot that separates keys. `[target.'cfg(…)'.…]` is
  still left out and **declared as a bound** with its own tracker; the bound and its tracker are narrowed to
  say so.

- **A refusal nothing could produce.** `admitted_types` guarded `str::split(…).next()` with an `else`
  saying *the clause has no sentence after its anchor* — and splitting any string yields at least one item,
  `"".split(". ").next()` being `Some("")`, measured on a real compiler rather than reasoned about. The
  branch was unreachable, so a diagnostic the reader appeared to draw was one it could never render, and the
  input it named — an anchor ending the contract — fell to *names no backticked type*, which is true of it.
  `split_once` with an explicit fallback replaces it, and that input now has a direction.

  Recorded rather than fixed, because looking for its siblings found one that is not this: the three
  `stated(…)` applications on the built-in composition path also guard a shape no concrete family observer
  can build, since all four production constructions of `Outcome::Violations` are the `else` of an
  `if violations.is_empty()`. They stay. Removing them would apply one rule at one arrival out of four and
  rest the other three on a whole-family invariant nothing checks — the argument for selective application
  would have to stay true by review, which is what a rule applied where an outcome arrives exists to avoid.
  The distinction is written where the rule lives: on that path it is construction-held, and on the protocol
  path — which takes any `Observer` — it is observable.

- **A dated heading's fields are ranged, not merely digits.** The repair that replaced a length test with a
  digit test carried its own standard only one level: *a length test is a parse without its guarantee*
  applies again to a digit test, and `2026-99-99` — and `0000-00-00` — satisfied *carries dated release
  notes*. A month is `1..=12`, a day `1..=31`. Whether that day exists in that month needs a calendar this
  crate's declared dependency surface does not carry; that residue is a date written wrong rather than a
  shape that reads as one.

- **A sentence about the key recogniser was refuted by the code it described.** It claimed each half of the
  recogniser is needed because *the first alone still admits `/version`, the second alone still admits a key
  ending in `version`* — and the delimiter half rejects **both** of those, so the pair of examples
  established nothing about either half. Replaced with one case per half, as runs rather than as a
  description: the delimiter half is what rejects `rust-version` and a path component; the `=`-follows half
  is what rejects a delimiter-preceded occurrence inside a string value. They live beside the reader in
  `src/tests/`, because the shape the second needs is one `cargo metadata` rejects outright — the first
  draft tried it end-to-end and failed on a duplicate-key parse error rather than on what it meant to show.

- **A git failure says which failure it was, and a source whose tracking could not be read is not
  "untracked".** Two diagnostics that answered a question they had never asked.

  `hermetic_git::run` folded *git could not be run at all* and *git ran and refused* into one `Err(String)`
  carrying only stderr, and two callers then discarded even that with `map_err(|_| …)`. On a machine without
  git, the operator standing in front of `cargo publish` was told **`repository root X is not a git
  worktree`** — a sentence about the repository, for a fact about the machine. The failure is typed now, and
  both callers say which one they met.

  The sharper half is `tracks()`. `ls-files --error-unmatch` exits non-zero for *this path is untracked* —
  the question — and also when git cannot be run at all; `.is_ok()` made the second answer the first. Every
  exclusion source then read as untracked and the gate refused with *hidden by X, which this repository does
  not track*: an **exit 1**, a disagreement, over a fact it never established, in front of the one act that
  cannot be undone. This repository reserves `1` for a source that disagrees and `2` for one that could not
  be read. It is three-state now, and the unreadable answer refuses as a cannot-judge.

  **And that three-state repair stopped one door short of its own argument.** It separated *git could not be
  started* from *git ran*, and git running is not git answering: `--error-unmatch` answers the question with
  exit `1`, and reserves the rest for declining to read the repository at all — `128`, measured, both for a
  directory that is no repository and for an index that cannot be parsed. Every one of those reached
  `Tracked::No` and refused as a disagreement, which is the same defect one exit status over, surviving the
  fix written against it. `Failure::Exit` carries its status now, so a caller can ask which non-zero it met,
  and `tracks()` reads exactly one of them as the answer.

- **A mark that asked nothing of anyone, and a refusal that would not say which way.** Two corrections to
  this window's own entries.

  The release-gate entry carried `**BREAKING**` and, in its own next sentence, *nothing an adopter runs*.
  *Closing a false negative earns a minor* is a rule about a **product reaction**, and the reason it gives is
  that the adopter's recorded baseline goes stale. A repository check ships in no package and produces no
  finding that reaches anyone's baseline, so nothing is owed. The mark read the rule by its first clause and
  not by the reason that clause gives — under `### Self-governance`, the heading the adopter-narrative check
  exempts precisely because it is not adopter-facing.

  And `admitted_types` collapsed `the_only`'s refusal with `.ok()?`, so **no anchor** and **two anchors**
  reached the caller as one absence. A maintainer who restated the rule in `AGENTS.md` — the shape prose
  acquires the moment a rule is worth repeating — was sent to look for a missing anchor while there were two
  of them. The sibling reader repaired in this same window argued the opposite for the identical distinction:
  *none and several are different facts*. One rule, two readers, one of them following it. The refusal now
  travels, and the direction that had **pinned the collapse** asserts the message instead of the absence.

- **The repair that routed a reader through `selection` built its candidates twice.** Two sites handed
  `the_only` an enumeration and then rebuilt the same one inside the `Err` arm, to tell *none* from *several*
  — `the_only` reports both as one refusal, and here they are different facts. The answer was correct and the
  shape was the one that module exists to discourage: the candidates become a value **once**, and both
  questions are asked of that value. Nothing observable moves, so there is no negative run to record and none
  is claimed; the existing failure matrix is what pins the behaviour that had to stay still.

  The third time in this window that a repair carried the class it was written beside, and the reason it is
  recorded rather than folded in silently.

- **The title guard shipped without a run that could tell it from its absence.** The wrapper's controlled
  `gh` answered the same title on every call, and no direction anywhere exercised a moved one — so the guard
  added for the third judged input had no negative run, in a repository whose rule is that *a guard is not a
  guard until it has been seen to fail*. The stub now answers differently on its second `--json title` call,
  a counter on disk making *second* mean second across two processes, and two directions hold it: a moved
  title stops before the merge and exits `2`, an unchanged one still reaches `gh pr merge`. Without the
  re-read the first reports exit `0` and an empty stderr — the merge simply happening.

  Its bound also cited a tracker that could never close it: *a merge or publish made outside the wrapper*,
  whose trigger is an act reaching either **without** the wrapper, for a race reached only by going
  **through** it. The debt had an owner on paper and no condition that discharges it. It has its own entry
  now, whose trigger is `gh` gaining a `--match-title` or a second judged input that can only be re-read.

- **The merge wrapper pinned two of its three judged inputs and left the third captured once.** It reads a
  subject, a body and a commit set, judges them, and then merges — and it had a reason written beside each
  treatment. The body travels as the **value** the gate judged, so a rewrite in between cannot reach the
  record. The commit set is one end of *body equals their concatenation*, so a push in between makes the
  judged relation false and `--match-head-commit` refuses. `subject == title` is a relation too, and the
  title was captured once and never looked at again: an edit during the gate left the merge recording a
  subject that is no longer the title, which is exactly the disagreement the rule exists to prevent. Sorted
  by the wrapper's own criterion, it was filed on the wrong side.

  The title is now re-read after the gate. **That narrows the window and does not close it**, which is
  declared rather than implied: `--match-head-commit` is decided by the server atomically, `gh` offers no
  `--match-title`, so a client-side re-read shrinks the exposure from a whole `cargo test` to one API call.
  The residue is a declared bound of `repository-checks`, beside the one for a merge made outside the
  wrapper.

  **A moved title is a cannot-judge, and the exit-class check refused the first draft for saying otherwise.**
  The gate did not find the subject wrong — it found it right, against a title that no longer exists, so what
  the wrapper holds is a verdict about a vanished input. The construction reserving exit `1` for the gate's
  own verdict arm is what caught the misfiling.

- **Two occurrences of the admitted-types anchor read as one.** The clause naming the narrowest honest commit
  type is the anchor the contract-agreement direction reads from, and it was taken with `.nth(1)`. A second
  occurrence — the shape prose acquires the moment a rule is restated, quoted or exampled — was dropped in
  silence, so the gate's list would have been compared against half its subject while reporting agreement.
  Two anchors are now a contract this reader may not choose between.

- **A wrapper scalar that was declared, pinned, and never read.** `GATE_VERDICT_ENV` sat in both wrappers
  beside `GATE_VIOLATION_CLASS`, which *is* read. It never could be: the invocation writes the variable name
  literally, because a shell cannot expand one into an environment-assignment prefix. So it was a second
  spelling of a token the exit-class check already pins against `verdict_channel::ENV` at the invocation
  itself — dead in the shell, and held alive by an assertion demanding it exist. Both are gone; the pin that
  does the work stays.

- **The subject check moved to where an outcome enters a run, because a one-observer run never reached the
  fold it first guarded.** `Run::observe` stores the first outcome verbatim — `None => next` — so
  `Run::over(m).observe(o).verdict()` still answered a violation-free `Outcome::Violations`: exit code `0`,
  no subject, no refusal. The repair below closed the composed path and left the commonest one open, and the
  scenario written with it said *and it is folded with a participant that did observe*, which described the
  half that was covered rather than the property.

  Every outcome now passes the check exactly once, on both ways in — `Run::observe` for the protocol's path,
  `evaluate_constitution` for the built-in one. The fold's own guard stays as a **consequence**: it takes two
  outcomes and nothing in its signature says they were checked, so the claim is held by the check rather than
  by a caller's discipline — but no input can now reach its refusal.

- **A composed run no longer states a subject on a participant's behalf.** `Report::empty()` is public and
  `Outcome::Violations` of it exits `0`, so a participant can return an outcome that is violation-free and
  names no subject — and `0.5.0` is the first release in which an outside `Observer` can be that participant.
  The fold answered such an outcome with `Subject::nothing_declared()`, which is not an absence but a
  **claim**: *this participant was configured with nothing to enforce*. Folded beside a participant that had
  observed a whole workspace, the composed verdict came back clean carrying only the other side's figure —
  the run under-reporting its own reach, with nothing to notice it. The declared bound covers a participant
  reporting a subject *larger* than it observed; this was the engine forgetting rather than the declarer
  lying, which no bound covers and no `Subject` invariant catches, because the sum stays representable.

  It is now a constitution error naming what the participant did. `observer-protocol` states why that class
  is shared with a boundary that could not be evaluated rather than earning a fourth `Outcome` variant: all
  of them say *this run reached no verdict you may act on*, and a public breaking distinction for a
  difference an operator does not act on differently is not one worth drawing. The message names which
  occurred, since the class does not.

- **The release gate stopped reading the section the release is about, and three readers beside it were
  fixed with it.** Four defects, each with a negative run recorded against the unfixed code.

  *The adopter-narrative check had no subject during release preparation.* It read `## [Unreleased]` only —
  and release-ready state requires that section to be **empty**. Preparation dates the new section and then
  keeps writing into it, so hundreds of lines of adopter-facing prose passed unexamined every time. The
  exemption's own reason is that rewriting a dated section would falsify the record, and where the section is
  still being written there is no record to falsify. It now covers `[Unreleased]` always and, in release-ready
  and snapshot state, the section dated for the workspace version. **The state decides it, not a version
  comparison**: in development the workspace version equals the released one, so that section is genuine
  record and stays exempt — a rule phrased as *strictly below the workspace version* would refuse it.

  *A renamed family dependency was skipped entirely.* Cargo admits `alias = { package = "xuanji", version =
  "0.0.1" }`; `alias` matched no family crate, so the entry was never examined, while the aggregate counter
  stayed satisfied by other examples. Which crate a dependency names is now its `package` field where it has
  one. The sibling reader over the root manifest never had this hole because it keys on the **path**, which a
  rename cannot move.

  *A dated heading was counted, not parsed.* `## [X.Y.Z] - notadate!!` is exactly ten characters after the
  prefix, so it satisfied *carries dated release notes*. The suffix is now read as three `-`-separated
  all-digit fields of widths four, two and two.

  *A dependency whose own name carries `version` was refused.* The pin was read from the first occurrence of
  the word on the whole line — the name and the path included — so `version-utils = { path =
  "crates/version-utils", version = "…" }` produced *has no version pin*, a false refusal over a correct
  manifest. A `version` assignment is now recognised as a table key.

  **No adopter action, and no `**BREAKING**` mark** — which the first draft of this entry carried. *Closing a
  false negative earns a minor* is a rule about a **product reaction**, and its stated reason is that the
  adopter's recorded baseline goes stale. These checks ship in no package and produce no finding that reaches
  anyone's baseline, so nothing is owed. Marking them read the rule by its first clause and not by the reason
  the clause gives, in an entry sitting under `### Self-governance` — the heading the adopter-narrative check
  exempts precisely because it is not adopter-facing — and saying in its own next sentence that nothing an
  adopter runs changes. Three documents contradicting each other.

  Two commit subjects in this window carry `!` and a `BREAKING CHANGE:` footer for the same over-application.
  They are merged squashes, and a merged squash is record: amending one changes its hash and decouples it
  from the pull request whose merge record cites it. The correction lives here, where the adopter reads.

- **The refresh recipe written beside the pins was wrong for one of the two actions it documents.** The
  comment told a reader to resolve a tag with `git/ref/tags/<tag> --jq .object.sha`. Measured against both
  pinned actions: a lightweight tag's ref answers `object.type=commit`, but an **annotated** tag's answers
  `object.type=tag`, and its `.object.sha` is the tag object rather than the commit —
  `cargo-deny-action@v2` is annotated, so anyone following the recipe would paste a SHA that is not a commit.
  `commits/<tag>` dereferences either kind, and its answers are byte-equal to the two SHAs pinned in the file.

  Nothing reacts to this class, and the honest reason is that the defect is semantic rather than referential:
  the command resolved, ran, and returned the wrong thing. It is the third instance in this window of *out of
  corpus is not permission* — a gate exists, its corpus was chosen deliberately, and the sweep is what has to
  cover the rest.

- **The workflow made the pinning argument for npm and never applied it to itself.** Every `uses:` in
  `.github/workflows/ci.yml` named a mutable major tag, while the same file's `every spec validates` step
  carried the comment explaining why `npx` was replaced by `npm ci` against a committed lock — *"`npx` resolved the TRANSITIVE
  tree fresh on every run with no lock and no integrity check"* — which is exactly what a repointable tag
  does for the runner's own executable dependencies. Cargo was pinned by `Cargo.lock`, npm by
  `package-lock.json`, and the actions by nothing. Each now names the commit its tag pointed at, with the
  release in a trailing comment: this changed which bytes run, not which version.

  `permissions: contents: read` already bounded what a substituted action could reach, and bounding is not
  closing — it limits the blast radius of code this repository never chose to run rather than stopping it
  from running. The dual the pin opens, a SHA nobody refreshes, is filed as accepted debt with the reason
  Dependabot was not adopted in the same breath.

- **Declaring the `specs` mode retired a vocabulary that other paragraphs went on using.** The commit
  that adopted OpenSpec's `specs` half replaced the whole `## OpenSpec lifecycle` section — the one that had
  defined `explore → propose → apply → sync` — and touched nothing else. The section it deleted ended by
  pointing at *Branching and release*, and that section still named `change/<openspec-name>` as one of two
  **fixed** branch roles "matching an OpenSpec change directory", against a directory the same file declares
  nothing has ever been written to. Two sections of one document, one routing by a word the other had
  retired.

  This is the class `AGENTS.md`'s own *Retiring a capability requires the same sweep* rule describes, caught
  by the sweep that rule prescribes rather than by the retiring commit — which correctly described what it
  had just done and had no reason to know what it had orphaned. The sweep found **six** sites where a read of
  the diff alone found two: the amendment step still routed through "an OpenSpec change", the branching
  section carried both the retired role and the retired lifecycle vocabulary, and `BACKLOG.md` both told a
  reader to promote work to "an OpenSpec change" and narrated `AGENTS.md` in the present tense as describing
  a lifecycle it no longer describes.

  Records are left as records: a backlog entry citing the `change/*` branch a past measurement ran on, and a
  dated release section describing the branching rule as it then stood, are measurements of their own moment.
  Only the live instruction is corrected.

- **A reaction refusing positional references was green over five of them, because its direction vocabulary
  was two words.** `reference-integrity` forbids a reference by *counted offset* and names no direction; the
  recognizer looked for `above` and `below` alone, so `five lines down`, `one line up` and `three lines lower`
  were invisible. The recognizer's own doc records fixing this exact shape one dimension over — a list beside
  the counted branch was *a second list beside the rule, joined to nothing and necessarily narrower than it* —
  and left the direction in that state.

  The widened list is not the naive one, because the naive one was written first and measured: 20 offences,
  most of them not references at all. `up` matched inside `group`, so the direction must be a whole word — the
  same test the branch already applied to its count and not to its direction. And `one level up` or `a layer
  down` names a relation between two rules rather than a place to look, so the relation-capable directions are
  admitted only over a positional unit; reporting the rest would be the false refusal this family forbids.
  Writing the reaction row for a widened direction then found `POSITIONAL_UNITS` half-plural — `line` carried
  `lines` while `paragraph` and `sentence` carried nothing.

- **The shell region's rule had four owners and two of them overclaimed.** `region`'s token-start rule was
  documented as *the shell's own* rather than an approximation of it, and the string-literal residue was
  scoped to the Rust region while both regions run the identical rule. Measured on bash rather than reasoned
  about: a marker after an unquoted metacharacter opens a comment that this rule does not cut, and a
  whitespace-preceded marker inside quotes is string content that it does. The first lets commentary satisfy a
  property about executed text; the second deletes executed text, the direction the Core Contract forbids.

  Both are latent — no tracked script carries either shape on an executed line — and both are now declared
  bounds of `repository-checks` with their pinning tests, so the register carries them. The rule itself has
  one owner and the other three point at it, because correcting two sentences of four would have left the
  class open. The shell region's whole decision table is an executable case rather than a paragraph.

- **The restriction that widening earned was written as a conjunct of one branch, and reached the readings
  that happened to have a noun.** Admitting the relation-capable directions only over a positional unit is
  what keeps a relation from being reported as a place to look — but it was a condition inside the *counted*
  reading, so the *adverb* reading, which supplies the noun's place itself and therefore has no noun for a
  list to admit, kept firing on any word before it. An intensifier before such a direction was read as a
  position. Latent, since no tracked comment carried the shape, and invisible to the case table because no
  row paired an adverb with a widened direction — the sentence claiming the restriction dropped every
  relation phrase was held by nothing.

  The pairing now travels beside the adverb rather than being decided in a branch, the way the attribution
  marks already do, and the flag is named for the direction property it is rather than for the one branch that
  first consumed it. Both readings gained rows, with the control that keeps the repair from over-refusing.

- **A figure warranting a design decision was typed into live source with nothing to reconstruct it, and the
  rule against that lived in three separate paragraphs.** The measurement behind the direction restriction was
  written as a bare count: reproducing it needs both the matcher and the tree, and the commit recording it had
  also repaired comments the sweep reads. It is now anchored to the commit and the invocation that produced
  it, which makes it an external fact any reader can re-run rather than the state of the session that wrote
  it — and the **judgement** over that set is marked as the half an anchor cannot bind, pointing at the rows
  where part of it has been made inspectable.

  `AGENTS.md` states the whole disposal as one decision instead of three correct paragraphs sitting apart: a
  figure in a live document is declared by a census, anchored to its commit and command, or dropped because
  the sentence says the same thing without it; record carriers stand outside by construction; and at review
  the disposal is those same three rather than an argument about which rule applies. Drafting it produced four
  violations of itself, none of which anything could have caught. `BACKLOG.md` carries the reachable remainder
  as a `WATCH`: the figures whose producer exists and never declared them, to be found by enumerating the
  producers and never the prose.

- **`PROJECT.md` narrated a retired capability in the present tense, and the sweep written to catch that named
  two files.** `gate-shape-contract` was retired in this window; `AGENTS.md` and `BACKLOG.md` annotate it and
  `PROJECT.md`'s Decisions section did not. The retirement sweep said to grep `CHANGELOG.md` and `BACKLOG.md`,
  which is a corpus narrower than its claim inside the paragraph about corpora narrower than their claims. It
  now names every tracked live document, with a dated changelog section exempt by construction.

- **The one manifest reader both irreversible-act gates share was still reading raw lines, and the workspace
  version now answers in three states.** `manifest::workspace_version` decided a property over executed TOML
  without taking its corpus from the shared region classifier, which `repository-checks` requires in so many
  words — so one root `Cargo.toml` was being scanned under two different region decisions inside a single
  judgement, the sibling `package_name` having been repaired that way already.

  Both directions were live and both were false refusals over legal, cargo-accepted TOML.
  `[workspace.package] # …` failed the heading equality, then matched *starts with `[`* and closed the table
  before it opened, so the version read as **absent**; `version = "X.Y.Z"  # bumped` carried its comment into
  the value, so the version read as **malformed**. The second is the spelling an author reaches for while
  bumping, which is exactly when the release gate and the publish gate both run.

  The reader also answered in two states where the domain has three. A single-quoted value is legal TOML this
  reader does not take, and reporting it as a version that is *missing* sends an operator to look for a key
  sitting in front of them — the conflation `Quoted` was introduced to end one reader over, left standing in
  its sibling. `WorkspaceVersion` is now `Declared` / `Absent` / `Unreadable`, both callers match
  exhaustively, and each names the judgement **it** could not reach rather than sharing one sentence about the
  fact. `Quoted` moved into `manifest` alongside it, since both facts that module owns need it.

  Found by a static review rather than by anything running: this is the first instance of the declared class
  *a check that never wrote a region decision is invisible*, and it falsified that entry's risk sentence,
  which had assumed the class errs toward reporting more. `BACKLOG.md` carries the correction.

- **Two section readers took the first of several candidates without answering how many there were.**
  `capability_subjects::subject_globs` and `proposal_capabilities` each took the text after the *first*
  `## Subject` / `## Capabilities` marker. A spec or proposal carrying two sections had the second one's
  globs and capability names dropped, so a capability governed less than it says while reading as a complete
  declaration, and the filing join then missed every file those globs claim — the identical silent narrowing
  the bullet reader inside the same function already refuses, one level up from it. Both now make the
  candidates a value and answer the count: `Declared::SeveralSections` and an `Err(count)`, each with its own
  refusal wording, because the count is what an author acts on and the bullet message would send them looking
  for a bullet that parses fine. Latent — no tracked spec carries two — so running the check could not have
  found it, and it was correct only while a second section happened not to exist.

- **The repair that gave the manifest region TOML's comment rule dropped a whitespace class on the way
  through.** Routing the inherit check into `region::toml()` replaced `line.trim()` then
  `replace(' ', "")` with `replace(' ', "")` alone. TOML's `wschar` is `%x20` **and** `%x09`, so
  `\tversion.workspace = true` stopped matching and its member was refused with *must inherit
  version.workspace = true* — a false refusal in front of the release gate over a legal, cargo-accepted
  manifest.

  **The same class and the same direction as the defect that repair had just closed, reintroduced by the
  repair.** Latent: no tracked manifest is tab-indented, and no direction covered a tab spelling, so nothing
  would have said so. Of the five manifest readers in that file the other four trim; this is the only one
  comparing a whole line, which is the predicate an omitted whitespace class hurts most.

  Restoring the `trim()` would have fixed the indent and left the tab *before a comment*, which the region
  correctly leaves in the head. The predicate asks its own question now — this line with its whitespace
  gone, `%x20` and `%x09` and nothing else. `split_whitespace()` would also have closed it and is not used:
  it removes every Unicode whitespace character, which is wider than the grammar and would accept a line
  TOML rejects. Reading the language's own rule rather than a wider borrowed one is the entire subject of
  the repair this regressed out of.

  Two spellings are held, the indent and the gap before a comment, because only one of them was reachable
  from the comment work that introduced the defect.

- **A test doc went on defending a decision that had been reversed.** The direction for the glued inherit
  comment was written to justify keeping two manifest readers out of `region`, on the ground that `toml()`'s
  token-start rule read that `#` as content. Three commits later `toml()` stopped using that rule and both
  readers were converted.

  The reversing commit swept `CHANGELOG.md` for the superseded conclusion and **did not sweep the test whose
  own subject the reversal changed** — the same supersession, one file over. The direction itself was never
  wrong and still passes; what expired is the reason it gave for existing, and a stale reason is what a
  future reader reasons from. It now records what happened to it, and names the edit that turns it red under
  the rule the reader actually has: reverting `toml()` to the token-start rule, run rather than asserted.

  Found by a review that re-read the prose the fix had made false, which is the third site of this shape in
  this window and the first found by someone other than the sweep.

- **The prelude reader entered the module and never left it.** `promised_members` found the promise by
  splitting on `pub mod prelude {` and taking what followed — which is everything to end of file. So a
  `pub use super::{ … };` written *after* the module closed became a promised member.

  Its own doc asserted the stronger property, that entering the module made the sibling distinction true **by
  construction rather than by circumstance**. Entering is half of it. The half that was missing is the one
  that fails: entering alone excludes a sibling above the module and absorbs one below it, and every fixture
  placed the sibling above — the one arrangement the unbounded reader answers correctly. A property asserted
  in the only position where it cannot fail was measured no more than the prose was.

  The direction that matters is widening. A name absorbed from outside the module enters the promise, so
  `adopter_surface.rs` is then required to name something `tianheng::prelude::*` does not export — a demand no
  compiler can settle, since the contract mentions identifiers rather than importing them.

  The block now ends at its matching close, walked over executed Rust and refused as `UnclosedPrelude` if that
  close is never found. The two brace directions are not symmetric and the walk is built around which one is
  dangerous: a stray `}` inside a comment would end the block early and drop every member after it — the
  promise narrowing silently, the one failure this check exists to catch — while a stray `{` leaves the walk
  unbalanced and is refused rather than guessed. Cutting line comments removes the first; the region's
  declared block-comment residue remains, and errs into the second.

  One fixture had to be repaired rather than the walk: the member-form refusal wrote `a::{B` and `C}` as
  separate entries, each carrying an unbalanced brace of its own. That went unnoticed while the read ran to
  end of file. It is one nested group now, and the same test passes against the *unrepaired* reader, so the
  repair took no coverage with it.

- **The manifest region borrowed the shell's comment rule, and TOML's is not the shell's.** `region::toml()`
  and `region::shell()` shared the marker `#` and, until now, one rule: a marker opens a comment only where it
  begins a token. That rule is the shell's own — `echo a#b` prints `a#b` — and it is not TOML's, which admits
  zero whitespace before `#`.

  **It was wrong in both directions at once, which is why no adjustment of it was the answer.**
  `xuanji = { path = "crates/xuanji" }#, version = "0.2.0"` declares a dependency with **no** version. The
  region read the commented text as content, `require_internal_pins` found a pin in it, and the release gate
  certified a manifest the registry would then reject — a false **pass** in front of `cargo publish`. Running
  the other way, `version.workspace = true#c` is a legal comment on a line that still inherits, so cutting
  nothing there was a false refusal. An earlier entry in this window measured that second direction and
  concluded the affected readers had to stay raw; the direction that mattered was never measured.

  `toml()` tracks strings now and cuts where TOML cuts: `#` outside a string, wherever it sits, across every
  string form — the multi-line ones included, since `"""` and `'''` cross the line boundary a per-line scan
  assumes. That keeps a `"https://…#anchor"` value whole by knowing it is a string rather than by hoping no
  space precedes the fragment, which is what the token-start rule was reaching for and could only
  approximate. `shell()` keeps the token-start rule, because it *is* the shell's rule and not an
  approximation of one.

  With a rule that is actually the language's, nothing was left protecting the readers held out of the
  region. Both are converted, and the last hand-rolled cut over TOML text outside `region` — a bare
  `split('#')` inside the inherit check — is gone with the exception it served. That count was itself checked
  after an earlier wording called it *a fourth spelling of this same rule*: four `split('#')`-shaped sites
  existed, and the other three read a Markdown heading, a shell command and a URL fragment. `package_name` gained something in the
  move, at the table **heading**: `[package] # the repository checks` fails `trimmed == "[package]"`, so the
  table never opened, no `name` was found, and `require_example_pins` answered `cannot_judge` over a legal
  manifest.

  This entry first claimed that benefit at the `name` **value** — `name = "kanhe" # …` supposedly reaching
  `quoted_value` as `Unreadable`. It never did: `quoted_value` takes the text between the first pair of
  quotes and discards what follows, before these commits as after them. **A reviewer refuted it by reading
  the function.** The correction is recorded rather than quietly swapped, because a claimed benefit is
  evidence to anyone auditing why a conversion was worth making, and one that dissolves on inspection is the
  same defect as a direction that cannot go red — asserted, never run.

- **A defence was written as a restatement of the rule it defends.** The direction protecting two raw
  manifest readers held its own copy of both predicates and called nothing in the gate — five string
  assertions about themselves, which no edit to the product could turn red. Two sites cited it as a guard.

  The decision it recorded held against the rule as it then stood: converting those readers would have
  **narrowed** them, because `version.workspace = true#c` is legal TOML that `region`'s token-start rule read
  as content. What was wrong is that the artifact recording the measurement was a copy of the rule rather
  than a reader of it — and the measurement was half a rule, since the same blindness ran the other way as a
  false pass. Both readers are converted now that `region::toml()` cuts where TOML cuts.

  Replaced by two directions through `judge`, and the one that matters was run against the change it exists
  to prevent: converting the inherit reader to `region::toml()` turns it red. The cheap way to tell the two
  apart, worth writing down because this is the second time it has been needed — **before writing a
  direction, ask which edit to the product makes it red. If the answer is none, it is documentation with
  `#[test]` on it.**

- **Three CHANGELOG and BACKLOG corrections were described in commit bodies and never written.** A scripted
  edit matched its anchor loosely — an em dash where the script expected a colon, a sentence that had grown a
  clause — and returned the text unchanged. Nothing failed: the staging step found no diff, the commit
  succeeded, and its body reported the correction as done. Two release-window entries and one `BACKLOG.md`
  paragraph were lost that way.

  **The commit body asserting a repair that never happened is the worse half.** A reader auditing this window
  takes it as evidence, and the paragraph it claimed to fix — *"the workspace version is still `0.4.0`"* —
  went on being false while the record said it had been corrected. Found by a review that checked
  `git show --stat` against the body rather than reading the body.

  The method changed, not just the text: a scripted record edit now fails loudly when its anchor misses. It
  caught a third instance within the same sitting.

- **The release gate read manifests as raw text, so a commented-out pin was refused as a live one.**
  `require_internal_pins` filtered on `path`, `"crates/` and `=` with no comment exclusion, so
  `# xuanji = { path = "crates/xuanji" }` satisfied every predicate, counted toward the non-vacuity guard,
  and was refused as `internal dependency # xuanji has no version pin` — a false refusal in front of the
  release gate, over text that declares nothing.

  The three readers a comment could satisfy go through `kanhe::region` now, which this file had imported
  nowhere. Two were left raw at this point, because `region`'s token-start rule would have refused the legal
  `version.workspace = true#c`. The exception did not outlast the window: that rule was not TOML's in either
  direction, and once `toml()` was given TOML's own, both readers were converted too.

  `region` gains a `toml()` accessor rather than reusing `shell()`: they agree on `#` by coincidence of
  syntax, not by sharing a decision. Which is what later made it possible to correct one language's rule
  without touching the other's.

- **An `ssh-keygen` was left unreaped on one failure path, in front of `cargo publish`.** `pipe_into` was
  extracted to own write-then-reap and its comment claimed reaping on every path; `sign_probe` hand-rolled
  the same three steps and returned on a failed write **before** dropping stdin and before any wait. One
  copy stood as the counterexample to the other's documentation. Both consume one `deliver_and_reap` now.

- **The limit that let a boundary's reason overreach is a declared bound.** `crate-dependency-boundary`
  declared none while the register held 84, and its reader observes no optionality at all — an
  `optional = true` edge sits in cargo's declared set like any other, so a dependency rule governs it as
  though it were unconditional. A **product** limit: any adopter writing `restrict_dependencies_to` on a
  crate with optional dependencies gets the optional ones counted. Declared through the full path — spec
  scenario, typed declaration, a pinning direction, both projections — and the register is 85 bounds across
  25 capabilities.

- **Smaller:** the CHANGELOG section-name derivation was written twice byte-identical and is now one
  function, whose doc comment was then found sitting on the wrong item — the extraction had displaced
  `section_shape`'s, a class no reaction can catch and one this repository has paid for before; `npm ci`
  gains `--ignore-scripts`, closing the half a lockfile does not, since pinning *which* packages install
  never stopped 80 of them running install scripts on a release-branch checkout.

- **A boundary's reason asserted structure its own rule cannot see, in the text most designed to be
  imitated.** 漏刻's `because` said the hot path depends on 璇璣 only *with xingbiao audit-gated for CI probe
  coverage*. Its rule is `restrict_dependencies_to(["xuanji", "xingbiao"])`, which reads the declared
  dependency set — it cannot tell a hot path from an audit path, and it observes an optional edge exactly as
  it observes an ordinary one, which `crate_dependency`'s own matrix pins as *optional/version/inherited
  included*.

  **Demonstrated, not argued**: removing `optional = true` from louke's manifest leaves the entire
  self-governance suite green, `tianheng_governs_itself` included, while that clause becomes false. That is
  the test `AGENTS.md` sets — *a reason must never assert structure the law does not react to* — and this
  reason failed it.

  It matters more here than in an ordinary comment because it projects verbatim into `AGENTS.self-law.md`,
  the Layer-2 text whose stated job is to condition an agent's continuations by example. An open loop was
  being modelled where it is most copied. The sibling reason one boundary down already gets this right by
  saying *"direct normal edges"* — naming exactly what its own rule sees — which is what makes this a miss
  rather than a house style.

  Trimmed to the perimeter, and **not** re-declared as a boundary: no supported observation surface
  expresses a feature-gated edge. The hot-path/audit split is Layer 3 and already lives in louke's own
  `[features]` comment. The property itself was never unguarded — CI compiles the audit-off configuration
  through `cargo clippy -p louke` and `cargo test -p louke` — so what was open was the law's claim about
  itself, not the invariant.

- **Two further review rounds closed the last four findings, and one of them corrected a record.**

  **The counted-offset detector read four unit words where its requirement names none.** `positional_reference`
  recognised an offset as a count followed by `lines`/`line`/`paragraph`/`sentence`; the requirement forbids
  *a counted offset* and names no vocabulary, so those four were a second list beside the rule and could only
  be narrower. Sixteen live positional references stood in tracked comments, four of them in the gate's own
  crate — clean over its own requirement. All repaired by naming the construct.

  The **article** branch keeps the list, because its clause turns on the noun: *a definite article naming no
  thing*. A line is pure position; a construct named and located is a reference the rule permits. Dropping
  the list from both branches was measured and took the tree an order of magnitude further, almost entirely
  onto phrases the requirement allows.

  Two conditions keep the widened branch from its opposite error, neither adding a vocabulary: the count must
  be a whole word (`round-9 finding above` names a finding), and it must be adjacent to the noun. A third was
  written, **measured to change no verdict, and removed** — it would have introduced a false negative of its
  own. A discriminator that changes no verdict is a claim, not a guard.

  **`workspace_version` and `semver` lived twice, in the same two files whose command builder was extracted
  this window — and unlike that pair, these had diverged.** One accepted a `[package]` table where the other
  did not; one was a digit check that admitted a version too large to order, the other a parse that refused
  it. Two readers of one fact reaching different verdicts, in front of `cargo publish`. Now one module, with
  the overflow boundary pinned. The `[package]` fallback is **not** carried forward: measured unreachable for
  every subject either gate has, so keeping it would preserve an untested branch settling a disagreement no
  input could produce.

  **`CLASSES` restated `AGENTS.md`'s vocabulary with nothing joining them.** Its doc explained why the set is
  not derived from the *headings* and said nothing about the document where the vocabulary is stated, so a
  class dropped from the contract would go on being admitted. Joined both ways, in the shape
  `merge_message_gate` already uses for its commit types against the same document — including the loud
  refusal when the anchor sentence cannot be parsed.

  **The capability-subject filing join observes nothing under the declared mode**, and now says so. It
  enumerates `openspec/changes/*/proposal.md`, which the `specs`-mode decision makes permanently empty — the
  join and the declaration landed four commits apart and neither noticed the other. Not re-pointed: under
  `specs` mode there is no independent capability declaration to compare a diff against. Recorded with its
  trigger instead, because an inert reaction reads as coverage.

  **A record said something untrue.** Both `BACKLOG.md` and this file listed `unbounded` among the words that
  would be wrongly read as bounds-named. The check strips an optional `s` and then refuses a following
  letter, so `bound` + `ed` does not match — and `unbounded` occurs live in this tree, so a reader verifying
  the claim would find it false. The residual is real for the other three; only the example was wrong.

- **Two more static reviews landed, and eight findings survived verification.** Both were run against the
  merged tree, so most of what they raised was already closed; these are the ones that were not.

  **A fixture helper was about to become permanent published API.** `xingbiao::claim_scratch` is
  `#[doc(hidden)]` now. Measured: zero call sites outside a test target or a `#[cfg(test)]` module, while
  星表's domain is declared workspace data — it lives there for a dependency-graph reason, which is not a
  domain fit. `0.5.0` is the release that first publishes it, so this was the last moment the decision was
  free; the same argument this section already makes for `BoundaryKind`, applied to the item that still had
  the choice.

  **The repository checks had no code owner.** `.github/CODEOWNERS` states its own criterion — *the
  protected file must be the boundary whose edit turns CI green after drift* — and this window moved every
  repository check into `crates/kanhe/` and put two wrappers in `scripts/` in front of `cargo publish` and
  `gh pr merge`. CODEOWNERS was updated for the one move it noticed and not for the class, so the file
  guarding against an agent weakening a boundary to pass CI protected the self-law and left every gate
  unowned. Both paths are named now, and `scripts/` for a reason of its own: an edit there does not relax a
  boundary, it removes the requirement to consult one.

  **`deny.toml`'s licence comment kept a hand-written copy of the dependency graph** and claimed it was
  exhaustive. It was wrong four ways — one crate named that no graph here contains, three in the graph and
  unnamed — in the supply-chain gate, where a reviewer doing a licence audit would trust it. Removed rather
  than corrected: `cargo deny list` produces it, and fixing the entries repairs the instance while leaving
  the next drift to be found by hand. What survives is what a list cannot carry — why an SPDX `AND` clause
  is accepted explicitly.

  **The family-coverage gate's path denylist is replaced by cutting comments.** Excluding the gate's own
  file closed the instance; any corpus file could credit a family from a doc comment. The rule is
  `kanhe::region`'s, replicated and cited because `kanhe` depends on `shengmo` — and the family side was
  re-measured to confirm the cut narrowed nothing.

  **The reference participant separates its walk, its rule and its report.** Nine siblings of the same shape
  stay filed as `WATCH`; this one moved because `COOKBOOK.md` sends adopters to it as the runnable version
  of its recipe, so a design whose layers cannot be tested apart is being taught rather than merely lived
  with. The rule now takes the file's text rather than its path, which is what makes it decidable from a
  string.

  Two smaller: `create_dir`'s threat model had two owners and now has one, and a scenario in
  `reference-integrity` says which reaction it means — two independent reviews read *the reaction fails* as
  requiring an integration nothing states, so the sentence was the defect rather than the code.

- **The family-coverage reaction had its own source file in the corpus it judges, and credited families to
  itself for naming them.** `crates/shengmo/tests/family_coverage.rs` lives under `crates/shengmo/`, which
  its own owner rule admitted — so a boundary type mentioned in its prose counted as an adopter-shaped owner
  of that family. A reaction whose corpus contains its own text can be satisfied by *describing* the thing it
  checks for.

  Found by a probe that should have failed and did not. Renaming the one type
  `examples/sans-io-pure/tests/reaction.rs` spells should have made that family read as unowned; it stayed
  owned, by a comment. With the judge excluded, the same probe now reports `["AsyncExposureBoundary"]`
  exactly as it should.

  A residual is declared alongside: **ownership is credited by type name, so a profile is invisible.**
  `Constitution::sans_io_pure` constructs a `ModuleBoundary` and an `AsyncExposureBoundary` internally, so a
  file declaring a family that way never spells the type. `examples/sans-io-pure` declares async exposure on
  its imitable surface exactly that way and is credited only because a separate test names the type outright.
  Stated in the reaction's own documentation and filed as `WATCH`, not closed by widening the reader — the
  honest closure asks what a profile *expands to*, which means evaluating constructor bodies rather than
  reading declarations.

  A claim this review made two rounds earlier is retired by the same reading: `AsyncExposureBoundary` is
  **not** missing from `sans-io-pure`'s `src/governance.rs`. It is declared there through the profile. The
  asymmetry was an artifact of grepping for a type name, which is the same mistake the reaction was making.

- **The OpenSpec adoption mode is declared, after being chosen from the beginning and written down nowhere.**
  OpenSpec has two halves — `specs/` as the per-capability requirement truth, and `changes/` as a proposal
  workflow. This project has always used the first and never the second: zero change directories have ever
  existed, while seven whole capabilities were specified in this window alone, four of them under `feat:`.
  `openspec/config.yaml` declared neither half, and `AGENTS.md` described the `changes` workflow in the
  present indicative — so the document a reader is told to consult first described the mode not in use, with
  nothing saying it was a choice.

  The cost was observed rather than argued: an agent reading `AGENTS.md` first, exactly as that file
  instructs, planned work through the unused mode three times in one session before anyone measured.

  The call now sits in `PROJECT.md`'s Decisions with the condition that would change it — a change too large
  to land as one reviewable pull request — and `AGENTS.md` states how a capability change is worked in the
  mode actually in use: the requirement is written onto its spec, the reaction is written in the same branch,
  and both land as one squash. **The spec and the reaction move together because they are one change**, not
  because a later sync step merges them.

  **Two things deliberately not done.** `openspec/changes/archive/.gitkeep` stays, its stated job changed
  from archive hygiene — which described pruning that never happens here — to **optionality**: the directory
  exists so adopting the other half needs no setup and no exception. And nothing enforces the mode, because a
  check that fired when a change directory appeared would prevent exactly that adoption. Encoding the choice
  into the tree, by deletion or by reaction, is the freezing the declaration exists to avoid.

  Three diagnoses were written before the right one. *Unrealized aspiration* and *prose contradicted by
  practice* both read the gap as a failure to follow a process; it was an undeclared decision, and the
  correction came from the challenge rather than from the review.

- **`BACKLOG.md` classified work twice and the two disagreed, in the file whose job is classification.**
  Every entry sits under a `### ` class heading *and* declares its own `*Class:*`. Seven of twelve live
  entries under `### READY-PATCH` declared `*Class:* WATCH`, and one declared no class at all.

  Not a formatting complaint: the classification exists so a reader can ask *what is ready to work on*, and
  the heading is what they read. Measured, the heading answered **twelve** where the truth was **four** — the
  question was asked in that form and got the wrong answer before this was found. The seven moved to the
  class they declare.

  **This one earned a reaction where its siblings did not, and the difference is worth stating.** The
  residuals recorded elsewhere in this window — whether a requirement outlived its mechanism, whether a
  figure reads as a live claim or a record — need a judgement over what a sentence means, which this
  repository has designed, measured and rejected three times. This one is decidable: a heading is a literal,
  a `*Class:*` line is a literal, and whether they agree is a comparison. Where the class is decidable it
  gets a reaction, and `crates/kanhe/tests/backlog_classification.rs` now holds both directions — an entry
  under the wrong heading, and an entry declaring nothing, which the first direction cannot see because a
  missing line disagrees with nothing.

  Building it also found that the file has **two** legal ways to declare a class — a `*Class:*` line in the
  classified sections, a title prefix in the combined one — and that both abbreviate (`ACCEPTED` for
  `ACCEPTED DEBT`). Both are read, rather than one being made to conform: requiring both forms everywhere
  would put the fact in two places inside each entry, which is the shape being removed one level up.

  The `READY-PATCH` definition gained a clause instead of one entry being misfiled to fit it. It classifies
  **evidence and compatibility, not how much design the correction still needs** — the entry that declared no
  class had measured pressure and broke no API, and was left unclassified only because the heading sounds
  like "next" and its fix is a capability yet to be designed.

- **Three spec requirements described mechanisms the shell-to-Rust migration had deleted, and are closed.**
  Each stated a live `SHALL` that nothing ran — the failure this whole family exists to end, committed inside
  its own governance, where a reader consulting a spec is told a rule is enforced when it is not.

  `governance-dogfood`'s focused-matrix ordering requirement is **removed**: its three shell scripts became
  one Rust test that owns its ordering internally, so there is no sequence of separate commands left to order.
  The decision was taken two windows ago and recorded in three places; it reached the spec in none of them.

  `reference-integrity`'s fixture-policy requirement is **restated as what the port does**. It demanded an
  explicit fixture-only governance-document set, refused on the real workspace. Nothing implements it — and
  the replacement is stronger: `offences_in` takes the corpus as arguments so a fixture and the real workspace
  run the same code, and the required set is a compile-time `const`, so the narrowing the old wording refused
  is now not expressible rather than refused by a check.

  `governance-dogfood`'s boundary-family coverage requirement is **rebuilt as a derivation**. It asked for an
  inventory of thirteen families anchored to the `0.2.x` surface; a literal beside an enumerator is the shape
  that lets a family be added and never re-examined. Neither side is written down now: families are the
  boundary types the composed shell re-exports, owners are the tracked files under `examples/` and
  `crates/shengmo/` naming one, and the reaction fails in both directions. Two of the thirteen were
  **profiles, not families** — `sans_io_pure`'s own documentation says it "adds no new reaction" — so counting
  them would have asserted coverage of a reaction that does not exist.

  **How they were found, and what does not catch them.** Not by a reaction: by reading each of the twelve
  shell files the migration deleted and asking which requirement had implemented it. Nine were correctly
  swept when they were deleted; three were not. A first draft of this work promised a `## Subject`-resolution
  guard alongside — which already exists, was green throughout, and would not have caught these anyway, since
  all three subjects resolve while the mechanisms under them were gone. No reaction is proposed: deciding
  whether a prose-described mechanism still exists is the instrument this repository has measured and
  rejected three times. The residual is stated instead.

- **The second defect queue is gone, and with it the convention that kept making one.** `BACKLOG.md` promised
  every sweep its own dated `docs/audit/*.md` file. That was two mechanisms for one job: a closed finding's
  substance is its closing PR and this file, an open one is a `READY-PATCH` or `WATCH` entry with the
  observation source and trigger `BACKLOG.md` already asks for, and the separate file held a second copy of
  both.

  It also regenerated a hazard each campaign, which is what made this worth removing rather than tidying. The
  queue written one commit earlier carried **hand-written line counts of live functions**, held by nothing —
  produced by the same review round that was removing that exact class elsewhere in the tree.

- **A live line count is not written, and the rule now names it.** `AGENTS.md` already said a count of
  something this repository does not produce is not written; a line count is the sharpest case, because it
  counts nothing the repository enumerates and moves on every edit to its subject — a rename, a rewrap, a
  comment. A sweep found one live instance in a tracked document: correct at that moment, held by nothing,
  and decoration by the rule's own test — removing it took one clause out of a sentence that still means what
  it meant.

  A line count of something **gone** stays fine and the rule says so, with the tree's own example as the
  form: `repository-checks`'s figure for the deleted shell library says when it was taken and that the set no
  longer exists, so nothing can drift out from under it.

  **No detector, deliberately.** Telling a live count from a record is reading what a sentence means — the
  prose instrument this repository designed, measured three times and rejected — while the decidable
  alternative, refusing every line count, would refuse the correct record above. This half stays a rule the
  reviewer holds and a `git grep` run on purpose. Recorded so the next reader does not re-derive the choice.

- **Two independent full-campaign reviews were merged, and their intersection was empty.** One was run
  here; the other was contributed as a 27-agent parallel pass. Neither found anything the other found — 7
  findings from one, 4 from the other — which is the most useful thing either produced, and the reason both
  are recorded rather than the union being reported as one review's output.

  **Two of the incoming review's grades were reversed on measurement, and both reversals mattered more than
  the findings they replaced.** Its `CONTRACT-VIOLATED` on `BACKLOG.md`'s spec-corpus figures ("30-75%
  stale") was wrong: measured at `ee15665`, the commit that wrote them, requirements **310** and scenarios
  **1177** are exact, and the sentence says the specs *held* them. Acting on that grade would have destroyed
  a correct record and replaced it with a live-looking claim this repository's own census policy forbids in
  prose. What was actually wrong was smaller — no measurement anchor, and `1048` counted *lines carrying*
  `SHALL` while the sentence called them occurrences. Its `VIOLATED` on the mutation harness's target
  directory was likewise a real hardening described at the wrong grade: the isolation held, by cargo's
  default, and what was missing was anything asking for it.

  The other direction is recorded too: the incoming review's gate ladder blocked Gates 5-6 for two files on
  Gate 4 length findings, and **both of this window's actual correctness defects live at Gate 6 in exactly
  those two files** — the lockfile table boundary and the scenario-reader divergence below. Length is style;
  a wrong verdict is not.

- **A `Cargo.lock` table that is not `[[package]]` no longer absorbs the package block above it.**
  `[[patch.unused]]`, which cargo writes whenever a `[patch]` section exists, carries its own `name`,
  `version` and `source`; those overwrote the still-open block's, so the last member's version was replaced
  before it was filed and the release gate reported `Cargo.lock is missing workspace package …` for a lock
  that records it. A false accusation in front of `cargo publish`. Any table header now closes the record and
  only `[[package]]` reopens one — skipping the foreign tables by name would have been a list of the ones
  someone had thought of.

- **The three readers of the scenario grammar stop at the same line.** `bounds_in` and `citations_in` ended a
  scenario at `## `/`### `/`#### ` while `undeclared_prose_offences` ended one at any line starting with `#`,
  so a `##### ` sub-heading left the first two inside a declared bound and the third outside it — reporting
  prose as an undeclared bound the register had registered. One `ends_scenario` predicate now, and the
  direction holds it as *agreement between the readers* rather than as three separate expectations.

- **The census word reader's ceiling is a declared bound.** `number_at` reads the units, the tens and one
  compound of the two, stopping at ninety-nine, while every sibling residual in that requirement was
  declared. A word reader that silently stops matching reads as covered. Declared rather than extended:
  the figures this repository writes in words are the small ones. The register is 85 bounds now.

- **The pinned OpenSpec validator is reproduced from a committed lock rather than resolved fresh.** The exact
  version pin bound the direct package and nothing below it, so `npx` resolved the transitive tree anew on
  every CI run and every local Definition of Done. `npm ci` against `package-lock.json` pins all 80 packages
  by integrity digest and `--no-install` refuses the network. `/node_modules/` is ignored rather than merely
  untracked, because the publish gate reads `--untracked-files=all`.

- **Smaller, in one campaign:** the reference gate's dated-section exemption now states how wide it is while a
  release is being prepared (measured: 41 references in the unreleased section, all resolving — filed WATCH);
  the mutation checkout asks for its own target directory instead of inheriting one; `git_metadata` is called
  `cargo_metadata`, which is what it runs; the identifier tokenizer that lived twice lives once, and
  deliberately stays separate from the prose tokenizer it resembles; a CI step that named a defence it did not
  exercise no longer selects it; both wrappers stop depending on bash 4.4 for an empty passthrough; and
  `COOKBOOK.md`'s two elided names stop looking like ones its block binds.

- **A sixth adversarial review round, of the fifth round's own fix, found the identical
  case-sensitivity gap survived in the one sibling recognizer that fix did not touch.**
  `bound_register_parse::marks_a_bound` (which decides whether a *scenario heading* — not prose —
  marks itself a bound) still compared `"a stated bound"`/`"a documented bound"` case-sensitively,
  the same exact-case bug `states_a_bound_in_prose`/`negates_bound_in_prose` were just fixed for one
  entry above. Currently latent — every tracked `#### Scenario:` heading is lowercase — but the
  asymmetry itself was live: with the prose functions now case-folded and this one not, a heading
  written Title Case (`"... - A Stated Bound"`) would open no bound in `bounds_in` while the
  now-fixed prose scan correctly flagged the surrounding text as stated-but-undeclared — the
  opposite-direction confusion from what the fifth round's fix closed, for the one function in the
  same file it didn't reach. Fixed the same way: case-fold before matching.

  Every code fix here also carries a committed regression test verified in both directions.

- **A fifth adversarial review round, this time scoped to the whole `v0.4.0..HEAD` remediation
  campaign again rather than only the latest fix, found a live, currently-active defect in the
  observation-bound register's own prose scan — not merely a latent one.**
  `bound_register_parse::states_a_bound_in_prose`/`negates_bound_in_prose` compared every token
  case-sensitively against lowercase literals (`"stated"`, `"documented"`, `"not"`, `"never"`,
  `"a"`, `"an"`), so ordinary sentence-initial capitalization never matched. Measured against a real
  tracked spec: `semantic-dyn-trait-boundary/spec.md`'s "**Stated** renderer-granularity bounds MAY
  coalesce the same subject at the same seam..." was read past silently by
  `every_bound_stated_in_prose_is_declared_as_a_scenario` — a production gate that runs on every
  `cargo test -p kanhe` — reporting the spec clean over a bound genuinely stated in prose and
  declared nowhere. Fixed by case-folding both functions before tokenizing.

  Fixing the detector surfaced the real violation it had been masking: with the fix in place, the
  gate correctly failed against that exact sentence. Investigated rather than silenced — the
  sentence is a normative design statement about `hunyi`'s own violation-identity model (how a
  renderer's declared fact-granularity may coalesce findings), not a description of an unclosed
  observation gap in this repository's own governance tooling, so declaring it as a formal
  `#### Scenario: ... - a stated bound` would misrepresent it as a residual this repository cannot
  close.
  Reworded the sentence instead, preserving its exact normative meaning while removing the
  "stated ... bounds" phrase that triggered the (now-correct) detector: "A renderer MAY declare a
  coarser granularity that coalesces the same subject at the same seam, but traversal position
  SHALL NOT be used to claim injectivity." Verified: the gate and `openspec validate --specs
  --strict` both pass with the fix and the reword together.

  Filed rather than fixed, both currently latent (verified against every tracked spec, zero live
  instances): `negates_bound_in_prose`'s one-interposed-word budget is measured independently from
  `states_a_bound_in_prose`'s own, so a sentence stacking both qualifiers
  (`"this is not a documented residual bound"`) reads as a declaration instead of the denial it is;
  and `requirement_heading_is_bounds_named` checks only the character *after* a `bound`/`bounds`
  match, not before, so a heading containing `outbound`/`rebound`/`abound` as a substring would be
  wrongly classified as bounds-named. (`unbounded` was listed here and in `BACKLOG.md` and is **wrong**:
  the check strips an optional `s` and then refuses a following letter, so `bound` + `ed` does not match.
  Corrected where both records stated it — a closed record carrying a false claim is worse than no
  record, and `unbounded` occurs live in this tree, so a reader checking the claim would find it false.)
  Both recorded as `WATCH` entries in `BACKLOG.md`.

  Also fixed in this round: `crates/xingbiao/src/tests.rs`'s own `TempDir::new` — the crate that
  shipped `claim_scratch` this window still used `remove_dir_all`/`create_dir_all` for its own test
  fixture root, the exact vulnerable idiom the migration existed to close, a few dozen lines from
  `claim_scratch`'s own regression tests. Now uses `claim_scratch`.

  Every code fix here carries a committed regression test verified in both directions.

- **A fourth adversarial review round, of the third round's own fix, found one cosmetic issue and
  raised one question already settled by investigation.** `dispatch`'s exhaustive `ParsedArgs`
  destructure bound `manifest_path` (`Option<String>`) and later shadowed it 45 lines on with a
  differently-typed local of the same name (`PathBuf`, the resolved path) — legal, but a reader
  scanning between the two must track which binding is in scope. Renamed the first to
  `manifest_path_arg`. The review also asked, at low confidence, whether an indented `#### Scenario:`
  heading could make `citations_in`'s newly-widened close-check disagree with `bounds_in`'s; verified
  directly (not merely argued) that both readers treat the case identically — an indented heading
  opens no bound in either (both check the raw, unindented line to open one) and closes whichever
  bound was open in both (both now check the trimmed line to close one) — so this was already correct,
  not a live gap.

  This round's fix carries no new regression test — the one change is a pure rename with no behavior
  difference, and the investigated question resolved to "already correct," verified with a throwaway
  probe rather than a permanent test since there is no behavior to pin.

- **A third adversarial review round, of the second round's own fix, found a small correctness gap the
  extraction inherited and a stale BACKLOG residue from a fix that had already landed.**
  - `citations_in` (extracted last round from `pinning_citations`) reset its tracked bound only on a
    `### `/`## ` heading, not on a bare `#### ` heading spelled something other than `Scenario:` —
    `bounds_in`'s own body scan stops on any of the three. A citation following such a heading was
    silently attributed to whichever bound scenario opened above it. Inherited verbatim from
    `pinning_citations`'s original code, not introduced by the extraction, but now shared by two
    checks instead of one. No tracked spec currently has a bare `#### ` heading, so this was latent;
    fixed to match `bounds_in` exactly, with a regression test proving the disagreement first.
  - Simplified `cited_bounds`'s citation-to-name accumulation to `.extend(citation.bound)` (an
    `Option<T>` is 0 or 1 items) instead of a named intermediate and a branch.
  - Widened `dispatch`'s `ParsedArgs` destructure (moved earlier last round) from matching by
    reference to **consuming by value**: every remaining use of a non-`Copy` field
    (`manifest_path`/`baseline_path`/`write_baseline_path`) now reads the bound local, and the
    compiler refuses `parsed.<field>` for any of them, naming the field, if a later line tries it.
    (A `Copy` field — `format`, `warn_uncovered`, `disallow_stale` — has no such backstop, since
    copying a place doesn't consume it; every field in this function is read through the bound local
    by convention for those three, not by a guarantee the type system enforces.)
  - `BACKLOG.md`'s own *"census::figures_in returns the first match on a line"* residue note was
    already false: the fix that closed it (`39c8856`) landed the day after the note was written
    (`ca437abb`), and nobody removed the note. Corrected.
  - Filed rather than fixed: `observation_bound_model.rs`'s `spec_bounds`/`spec_defence` is a third,
    untouched implementation of the same `#### Scenario:`/`PINNED-BY` grammar `bounds_in`/
    `citations_in` now share, and it has already diverged from them in two latent edge cases
    (requires a literal space after `Scenario:`; checks an untrimmed line for a closing heading).
    Recorded as a `WATCH` entry in `BACKLOG.md` rather than unified here — it touches a file this
    window's fixes did not otherwise reach, and reworking it deserves its own scoped review.

  Every fix here also carries a committed regression test verified in both directions.

- **A second adversarial review round, this time of the first round's own fix, found the first
  round had reinvented logic that already existed and left one sibling flag-conflict check out of a
  hardening its own comment said was "the one place this could least afford to recur unnoticed."**
  Reviewing a review's own fix for the same drift is the same discipline one level deeper:
  - `pin_bites`'s `cited_bounds()` fix (below) added a second, hand-written line scanner to recognize
    a `PINNED-BY` citation outside a bound scenario — duplicating `bound_register_parse`'s own
    existing `pinning_citations`, which already resolves a citation to its bound wherever it appears,
    just from the worktree rather than from `HEAD`. Extracted `pinning_citations`'s per-text scan into
    a new `citations_in(capability, spec, text)`, reused by both: `pinning_citations` for the
    worktree, `cited_bounds` for `HEAD`. One recognizer instead of two, and `cited_bounds` no longer
    scans each spec's text twice.
  - `dispatch`'s exhaustive `ParsedArgs` destructure (below) covered only `--write-baseline`'s own two
    conflict checks, leaving the `--baseline`/`--write-baseline` mutual-exclusion check and the
    `--disallow-stale`-requires-`--baseline` check three lines above still reading `parsed.<field>`
    directly — the identical asymmetry, one level up in the same function. Moved the destructure
    earlier so it covers every flag-conflict check `dispatch` makes, not only the last two.
  - Clarified `every_declared_mutation_s_name_resolves_to_a_real_bound_id`'s doc comment: `cited_bounds`
    mapping a non-bound citation to an empty id list (below) stops the *existence* check from
    mistaking a real citation for a fabricated one, but this test still refuses a `pin_mutations.tsv`
    record for such a name, by design — a declared mutation is a claim about a bound's defence, and a
    citation with no bound to defend is not one. Not a behavior change; the prior comment did not say
    so and a reviewer read the CHANGELOG entry below as claiming otherwise.

  Every fix here also carries a committed regression test verified in both directions.

- **An adversarial review of this whole remediation window's own work (items above, back to the
  `v0.4.0..HEAD` audit tracker) found five defects in the fixes and checks it produced, and all five are
  closed.** Reviewing a campaign's own output for the same class of drift it exists to catch is the same
  discipline the campaign already applies to the rest of the repository, turned on itself:
  - `dispatch`'s `--write-baseline` conflict check read `parsed.write_baseline_path` /
    `parsed.warn_uncovered` / `parsed.format` by field access, while `dispatch_list`'s equivalent guard just
    above it was exhaustively destructured so a `ParsedArgs` field added later fails to compile rather than
    reaching `list` unconsidered — the asymmetry a Contract-violated finding earlier this window was
    specifically about, recurring at the one other site the same struct is read from a live conflict check.
    Now exhaustively destructured too.
  - `pin_bites`'s `cited_bounds()` (introduced this window to map a citation to the bound id it defends) read
    citations through `bound_register_parse::bounds_in`, which is scoped to scenarios `marks_a_bound`
    accepts — the observation-bound register's own job, not this check's. A `PINNED-BY` citation under an
    ordinary requirement scenario (this window's own
    `the_construction_held_list_matches_the_built_in_composition_path` among them: 7 of 76 tracked citations)
    was silently invisible to it, where the `cited_names()` it replaced had recognized every `PINNED-BY` line
    regardless of heading. Masked today because no `pin_mutations.tsv` record names one of the seven, but the
    first one added would have panicked reporting a real citation as cited by no declared bound. Broadened to
    recognize every citation, mapping a name found only outside a bound scenario to an empty id list rather
    than dropping it.
  - `reference_integrity`'s `ignored()` (the trailing-slash retry fixed earlier this window) forces the
    directory reading regardless of what is on disk, which its own doc comment claimed "can only widen …
    never narrow" what counts as ignored. Measured false: `git check-ignore -q -- build/` matches a
    directory-only `/build/` pattern even when `build` exists on disk as an ordinary file, so a real,
    trackable file sharing a name with a directory-only pattern was misclassified as deliberately ignored —
    a stale reference to it would have gone unreported. The retry is now skipped whenever the candidate
    already exists on disk as something that is not a directory.
  - `bound_register_parse::undeclared_prose_offences`'s own doc comment claimed it recognizes a declared
    bound scenario "mirroring `bounds_in`'s own recognition exactly, so the two cannot come to disagree" —
    false for an indented `#### Scenario:` heading: `bounds_in`'s opening check is untrimmed and does not
    recognize it, while this function trimmed the line first and did. An indented bound-marked scenario
    would vanish from the register entirely (never a declared bound, and never reported as prose stating one
    outside a declared scenario either) with neither check disagreeing loud enough to say so. No tracked
    spec is currently indented; fixed to check the untrimmed line, matching `bounds_in` exactly as claimed.
  - `census.rs`'s `figures_in` overlap protection (added earlier this window) skipped past only the *first*
    placeholder's own number after a match, closing the exposure where a start offset inside that number's
    tail re-triggered a spurious rematch. The identical exposure survives one placeholder later: a phrase of
    two or more placeholders with a short or empty literal after a non-first one lets *that* figure's digits
    restart a match, reusing text a prior occurrence already consumed (`"{} of {}"` against `"3 of 53 of
    9"` returned `[[3, 53], [53, 9]]`, double-counting `53`). Both currently-declared census phrases have
    long literal tails and never triggered it. Simplified rather than special-cased further: `match_from` now
    reports the whole match's consumed length, and the scan skips past all of it, which closes both the
    original and this exposure with less code than tracking the first number's boundary alone did.

  Each fix carries a committed regression test verified in both directions: reverting the fix reproduces the
  exact reported defect, and restoring it passes.

- **A fixture scratch root is now claimed the same way everywhere, not just at the one production site a
  prior fix closed.** `kanhe::publish_source_gate::claim_scratch` closed the exposure for the signature
  workflow standing in front of `cargo publish`: `create_dir_all` silently adopts a pre-existing symlink and
  writes through it, while `create_dir` cannot follow one and refuses. Roughly forty **test-fixture** scratch
  roots across `guibiao`, `hunyi`, `louke`, `tianheng`, and `kanhe` still used the wider call, each one a
  developer-machine-only race window rather than a path to a release artefact. Added `xingbiao::claim_scratch`
  — the shared helper lives in the lightest-weight crate already reachable from every dimension, so it
  closes the gap without amending `kanhe`'s dependency law — and migrated 37 call sites across 24 files.
  Sites that only build a subdirectory *within* an already-claimed root were left untouched, since building
  structure inside a root the fixture already owns was never the exposure.

- **Whether `observer-protocol`'s declared construction-held dimension list matches the built-in composition
  path is now held to that, by a cheaper route than the one originally recorded for it.** The recorded plan
  was a perturbed build: empty a dimension's observer and see which assertion fails. Reading the built-in
  path's own source directly answers the same question without paying for one — for a construction-held
  dimension the built-in path does not call some function that happens to agree with the observer today, it
  *constructs that dimension's own `Observer` and calls `.observe()` on it*, so there is exactly one
  implementation to find rather than two runs to compare. `evaluate_constitution`'s semantic and runtime arms
  each construct `SemanticObserver`/`RuntimeObserver` directly; the static arm still calls `check_and_cover`
  without ever constructing `StaticObserver`. Added
  `the_construction_held_list_matches_the_built_in_composition_path` and re-pointed the spec's scenario from
  `UNPINNED` to `PINNED-BY` it. Verified both negative directions: moving the semantic construction out of
  `evaluate_constitution`'s own body fails the new test, and adding a static construction inside it also
  fails — restoring the real implementation passes both.

- **Two declared residues over the same class were cross-referenced instead of left as two separate problems
  for a future reader to reconcile.** The bounds-method reader's whole-line-copy stated bound and
  `kanhe::region`'s `Executed` abstraction both declare the identical gap for the identical reason — a block
  comment and a string literal both need nested-span lexing this tree has "defeated repeatedly," which
  neither closes. The citation was one-directional (`region.rs` already pointed at the reader's bound);
  `observer-protocol`'s spec now points back. Also recorded, checked against the real corpus rather than
  argued abstractly: the "require the anchor preceded by `impl Observer for`" candidate closure was measured
  against the three real dimension files and refuted as literally stated (the real definitions sit several
  lines and one sibling method below their `impl` line), and a looser scope-containment version only raises
  the bar rather than closing the class. If this is ever closed, it needs one shared nested-span lexer
  serving both sites, not a point patch to either.

- **`openspec validate --specs --strict` now runs in CI and the Definition of Done, instead of only being
  claimed by a spec's own scenario.** `observation-bound-register/spec.md` states this command passes over
  every spec; it was never wired into `.github/workflows/ci.yml`, `AGENTS.md`'s Definition of Done, or any
  script — a prior CHANGELOG entry had said this gap was closed by adding the command to both, and it was
  not. Wiring it surfaced a genuine failure the command's own passing claim had been masking: `repository-checks`'s
  "The prelude promise SHALL be held against the contract compiled from outside" requirement failed strict
  validation, because its opening sentence — the only line the validator's parsed `text` field reads — carried
  no SHALL/MUST keyword, even though the rest of the paragraph did. Reworded the opening sentence to lead
  with the SHALL; all 36 tracked specs now validate clean. The invocation is pinned
  (`npx --yes @fission-ai/openspec@1.4.1 validate --specs --strict`) so a future validator release cannot
  silently change what "passes" means here.

- **The 20-item partial-claims batch from the `v0.4.0..HEAD` audit is closed.** Each carried real evidence
  but an incomplete verification artifact — usually a real fix whose permanent regression test was actually
  a manual, one-time terminal check. Verified each individually; where a genuine gap survived, closed it
  with a committed test rather than recording the claim as good enough:
  - `bound_register.rs` gained `search_and_must_panic_on_a_genuine_failure_not_only_on_a_clean_miss` (the
    shell era's `cd`-failure-collapsed-into-grep's-no-match-exit regression, and the general exit>1
    distinction), `tracked_specs_refuses_a_repository_with_no_spec_md_rather_than_reporting_it_empty`, and
    `a_raw_identifier_citation_resolves_to_its_definition` (extracting the shared `definition_pattern` so
    the test and the real check cannot diverge).
  - `shengmo/tests/self_governance.rs` gained `the_publish_field_s_four_states_are_read_correctly`,
    extracting `excluded_by_publish_field` so the dimension enumerator's four-state claim is pinned against
    cargo's actual reported shapes rather than only against whichever states this workspace's manifests
    happen to carry.
  - `dod_coherence.rs` gained `a_dod_block_with_no_commands_is_refused_not_reported_clean` — its own
    zero-commands guard existed in code but no test exercised it, unlike its `whitespace_hygiene.rs` sibling
    fixture, which was already present and correct.
  - Two CHANGELOG claims were themselves corrected on recount: "the regenerated projection differs by
    exactly three lines" undercounted a diff that also added two whole new crate sections (19 insertions, 3
    deletions, not 3 lines), and a stale reference to a deleted script and its test twin was reworded.
  - The remainder verified clean on inspection: existing hand-written test coverage already matched claimed
    shapes just not literally "table-driven" wording, a shell-era hardening concern (ambient environment
    override of a governance-document list) is structurally impossible against the Rust port's `const`-based
    design, `cargo metadata`-sourced identity reads cannot be defeated by a manifest-level alias by
    construction, and several fixture/durability claims (the anti-restatement reaction, the publish
    wrapper's allowlist and `--manifest-path` refusal, the lexical trait-object continuation-line bound, the
    third text-recognizer's HTML-comment-span fix) were confirmed still currently passing rather than
    re-argued from scratch.
  - A `scripts/` line-count discrepancy (6,827 vs. 6,841) turned out not to be an error: the two figures
    answer different, both-legitimate questions about what counts as a "library" (whether a 14-line
    mutation-fixture data table is one).

- **A DoD-coherence requirement carried over from the shell era was investigated rather than assumed
  still applicable, and retired.** The deleted shell predecessor of `dod_coherence` additionally required
  three named "focused example matrix" scripts and a positive driver script to appear as one contiguous,
  ordered sequence in both `AGENTS.md`'s Definition of Done and CI, and required the driver to never name
  a matrix script directly — guarding against the matrices and the driver silently reordering or nesting.
  None of those scripts exist anymore: the shell-to-Rust migration consolidated them into one Rust test,
  `examples_suite`, which owns its own example table and ordering internally and is named on a single
  DoD/CI line the existing membership check already covers. There is no longer a sequence of separate
  commands to order and no separate driver script that could recurse into a matrix script, so rebuilding
  the check would have nothing left to react to. Recorded the decision in `dod_coherence`'s own doc
  comment rather than leaving the gap silent.

- **A confinement test's name claimed to guard against a lib/bin coincident-path conflation its fixture
  never created.** `external_confinement`'s
  `confine_external_crate_conflates_coincident_lib_and_bin_conventional_paths` declared only a single
  library target — the test helper it used has no way to declare a second — so it never exercised any
  lib/bin interaction at all, and was otherwise byte-for-byte the same fixture as an existing test
  immediately above it in the same file, adding no coverage under a name that promised otherwise. Built a
  real fixture with a `lib` and a `bin` target sharing one `src/` directory, each with its own conventional
  `mod` declaration resolving to the identical physical file, and confirmed `check_module_boundary`'s
  per-root evaluation already reports each unit's own copy of a leak once, for its own unit — neither
  merged into one nor dropped for either. Deleted the redundant test and added
  `confine_external_crate_evaluates_each_unit_at_a_coincident_conventional_path`, which pins that real,
  previously-unpinned behavior. Verified both negative directions: reproducing the historical false
  negative (only the first root evaluated) drops the fixture from two reported violations to one, and the
  restored per-root evaluation reports both, each carrying its own unit label.

- **A reference-integrity exemption for a deliberately git-ignored path depended on whether the ignored
  directory happened to exist on disk.** `reference_integrity`'s `ignored()` helper exempts a
  prose-referenced path from the stale-reference check when git itself deliberately ignores it (a
  generated directory, for instance). A directory-only `.gitignore` pattern (one ending in `/`) only
  matches a candidate `git check-ignore` can see is a directory, and it can only see that by `lstat`ing
  a path that exists — so a generated directory a fresh checkout has not built yet read as "not a
  directory," silently failing to match, and whether the check fired depended on which examples happened
  to be built on the machine running it rather than on the repository's own declared ignore rules. Fixed
  by also querying the candidate with a trailing slash forced on, which reads as a directory regardless
  of what is on disk; this can only widen what counts as ignored, never narrow it, since the original
  bare query still runs first. Verified both negative directions: a fixture git repository with a
  directory-only pattern and no directory ever created reproduces the missed match under the old
  single-query form, and the new query catches it.

- **A census sweep read only the first figure-match on a line, missing a stale one following a correct
  one.** `figures_in` returned on its first successful match, so a line writing the current figures first and
  a stale earlier draft's figures later, in the same declared phrasing, reported clean — the trailing stale
  occurrence was never examined. Fixed to collect every occurrence. Fixing this surfaced a second, sharper
  defect adversarially: trying every byte offset means a start one digit into a multi-digit number reads its
  tail digit as its own separate number (matching the same phrase again), and a start inside a hyphenated
  compound number word reads its tail word as its own separate number — both producing a spurious extra
  match on an ordinary, single-occurrence line. `match_from` now reports how far the first number's own token
  reached, and the scan skips past it rather than retrying one byte later, closing both the digit and the
  compound-word overlap the same way. Verified both negative directions: a naive "collect every offset with
  no skip" reintroduces the spurious extra match on a correctly-written line, and reverting the fix entirely
  reintroduces the original missed-stale-figure defect.

- **The register's own projection was only staleness-checked, never content-checked.** The companion test
  compared `render_projection`'s output against the tracked `docs/observation-bounds.md` byte-for-byte — proof
  the document and the renderer agree, never proof either is right, since both come from the same format
  string. A typo baked into that string literal (the `` `author\s:` `` mangled-apostrophe class this
  repository has hit before) would regenerate byte-identically under `BLESS=1` and pass. Added
  `the_projection_s_disclosures_are_asserted_not_only_its_freshness`: reads the blessed document directly,
  independent of the renderer, and asserts each disclosure the requirements oblige the header to make is
  literally present, refusing a rendered backslash outright. Verified both negative directions: a missing
  disclosure and a stray backslash injected into the live document each fail the new test with the exact
  message this fix adds; the byte-diff freshness test alone would have passed both.

- **A bound stated in prose but not declared as a scenario now fails, closing a gap the shell-to-Rust
  migration silently dropped.** `observation-bound-register`'s requirement of the same name has been fully
  specified — four scenarios, three residuals, a bounds-named-requirement exemption — since before this
  window, but nothing reacted to it: the shell era's `BOUND_PROSE` scan (the bound-register gate script)
  was deleted by `64ed18c` and never reimplemented. Ported directly (`states_a_bound_in_prose`,
  `negates_bound_in_prose`, and a single-pass requirement/scenario state-machine walk in
  `bound_register_parse.rs`, reusing `marks_a_bound` and `bare_references` rather than re-deriving either;
  no regex crate, since `kanhe`'s dependency law admits none). Wired into the ordinary suite as
  `every_bound_stated_in_prose_is_declared_as_a_scenario`.

  Running the new scan against the live corpus surfaced one real hit:
  `runtime-origin-assertion`'s "Root-aware audit excludes unreachable source files" requirement stated a
  doubly-nested `#[cfg_attr(a, cfg_attr(b, path = "…"))]` as "a stated, undetected bound of this hand-rolled
  scanner." Measured directly (a fixture with the nested target holding the only real probe): **false** —
  the scanner's `path`-value search is a linear scan over the whole attribute span, indifferent to nesting
  depth, so a doubly-nested target resolves exactly like a single-level one already does. The requirement
  now states this as a SHALL with a new scenario, the matching stale comment in
  `crates/louke/src/audit/scan/lexer.rs` is corrected, and a new regression test pins the now-confirmed
  behavior in `crates/louke/src/audit/tests.rs`.

- **A pin surviving its declared mutation named the citation, never the bound it defended.** `pin_bites`'s
  `cited_names()` read every `PINNED-BY` citation from `HEAD` as a flat list, discarding which bound each
  one belonged to — so the one message this whole check exists to earn (a mutation a cited test does not
  catch) could say only the bare test name. Replaced with `cited_bounds()`, which reuses the canonical
  `bound_register_parse::bounds_in` parser (the same one `bound_register.rs` itself uses — a second
  implementation of "which bound cites this name" is the twin-drift class this repository keeps closing)
  to map each cited name to the bound id(s) it defends, still read from `HEAD` rather than the worktree.
  The survival message now names the citation, the mutation (file and the anchor replaced), and the bound.
  Pinned by a new, ungated regression (`every_declared_mutation_s_name_resolves_to_a_real_bound_id`, cheap
  enough to run on every `cargo test -p kanhe`) verified to fail when the bound-id mapping is dropped.

- **A citation resolving to more than one definition named the count, not where to look.** The bound
  register's duplicate-definition offence said `defined 2 times under crates/` without naming either
  site, even though `git grep -n`'s own output (`path:line:content`) was already sitting in hand. Extracted
  the offence-formatting into `definition_count_offence`, directly unit-tested with a synthetic two-site
  input rather than only through the live `cargo test --list` enumeration, and it now lists every
  definition site under the count. The zero-sites direction (a citation the harness registers but no
  definition matches) is unchanged — there is no site to name.

- **`hermetic_git`'s own `git()`-wrapper output/success/failure mapping lived twice, byte-identical past
  the leading flags, in `publish_source_gate` and `release_coherence_gate` — the exact class this module
  already exists to have closed for the `hermetic` builder underneath it.** The earlier fix unified
  `hermetic()` but left the wrapper built on top of it duplicated in the same two files. Moved into
  `hermetic_git::run(repo, flags, args)`, taking a leading-flags slice (`&[]` for
  `release_coherence_gate`, `&["-c", "core.excludesFile=/dev/null"]` for `publish_source_gate`); both
  gate files' own `git()` now delegate to it in one line. No behavior change, confirmed by all 30
  `publish_source`, 1 `publish_source_integrity`, and 51 `release_coherence` tests passing unchanged.

- **`scripts/merge-pr.sh` refused `gh`'s short spelling of `--delete-branch` without naming why.** `-d`
  fell through to the generic catch-all message instead of the dedicated one explaining that deleting a
  branch auto-closes any pull request stacked on it — unlike every other admitted-consequence flag
  family in this wrapper, each of which catches its own short/glued spelling in the same arm (`-t*` with
  `--subject`, `-F*`/`-b*` with `--body-file`/`--body`, and so on). Safety held either way (`-d` was
  still refused, exit 2), but the operator wasn't told the actual consequence. `-d` now shares the
  `--delete-branch` arm. Pinned by a new assertion in `only_an_allowlisted_flag_reaches_the_merge`; run
  against the prior script, it failed with the generic message this fix replaces.

- **The no-trait-object reaction scanned its own test file's crate, not the composed shell it names.**
  `composition_introduces_no_trait_object` used `CARGO_MANIFEST_DIR` to find the corpus to scan; since
  the test lives under `crates/kanhe/tests/`, that resolved to `crates/kanhe/src` — a directory with no
  coupling to `Observer`/`Run` at all, confirmed by a single doc-string mention and nothing else. The
  actual composed shell (`crates/tianheng/src`, where `Run` folds observers together) was never read.
  The reaction now resolves the workspace root and scans `crates/tianheng/src` explicitly. A probe
  function returning `Box<dyn Observer>`, injected into `crates/tianheng/src/runner.rs` and reverted
  immediately after, confirmed the fixed reaction catches it and the prior one did not.

- **The pre-publish source gate folded a read failure into "version missing," right before an
  irreversible act.** `workspace_version` read `Cargo.toml` with `.ok()?`, so a real `io::Error`
  (permission denied, a broken symlink, non-UTF8 bytes) and a manifest that reads fine but genuinely
  lacks a version key produced the identical `workspace version is missing or malformed: <missing>`
  refusal — the one distinction this gate exists to draw, at the one moment (`cargo publish`) it matters
  most to draw it. `workspace_version` now returns the read failure instead of discarding it, and
  `judge` reports it by name rather than folding it into the generic message. Pinned by a test that
  writes invalid UTF-8 in place of the manifest and asserts the refusal names the read failure and never
  reads `<missing>`; run against the prior code, it failed exactly as expected.

- **Five entries above describe a mechanism this same window later deleted, and none of them says so.** An
  adversarial contract review read every entry in this section — and the ones above it — against `HEAD`,
  not against the commit that closed each one, and found: the projection content-assertion test that
  caught `` `author\s:` `` lived in the shell era's bound-register companion test, deleted whole by
  `64ed18c` along with the rest of `scripts/`; the
  block-comment-delimiter stop and its dropped 12-line cap were the same shell fallback's logic, deleted
  with it; the second **third floor** (a citation matching a block comment's line, not its comment state)
  was superseded 33 minutes later by `68cd5dd2`, before this entry — or `HEAD` — ever carried it in this
  form; the `gate-shape-contract` `1-versus-2` bound's three-site correction and its process-substitution
  property both went with the whole capability, three and two days after each landed, in `8d44e77` and
  `5abda1f` respectively. Each entry was true when written and none is wrong about what it once did — a
  capability's retirement, unlike a rename, leaves no term for a grep to find, so nothing swept these when
  the deleting commit landed in the same window. `AGENTS.md`'s vocabulary-sweep rule is widened to name
  this case, since the sweep it already stated did not.

- **A declared bound outlived the behaviour it described, and the register said so to anyone who consulted it.**
  The dimension list's false negative — *a dimension absent from the check's own list is not examined* — was
  closed days earlier when that list began being read from cargo and compared both ways. The scenario, the typed
  declaration and both generated projections still carried it, and every check passed: the bijection compares
  the spec side with the code side, and both were stale together, so they agreed.

  That is the failure this register exists to end, committed inside it. A reader is told to consult the register
  **before** reporting a behaviour as a defect, so a retired bound left standing reads as permission for
  something that is now refused.

  Retired against a run of its **own WHEN** on the post-change tree rather than against the argument that it
  should be: a dimension removed from the literal, and a dimension crate the literal never named, are both
  refused now. Its own text had named its closing condition — *the dimension set derived from something that
  enumerates it rather than typed beside it* — and that is what the reaction does.

  The sibling bound stays: the rule variant the filter does not reach is still open, and the requirement's
  count of what it declares moves from four to three with the retirement recorded rather than the number
  quietly edited. The census direction caught that count in the same run.

- **The dimension enumerator matched a dependency in any table, and this family treats the tables as different
  observation surfaces.** Cargo reports `kind` as null for a normal dependency and as `dev` or `build` for the
  others; `crate-dependency-boundary` says a boundary observes exactly one table and that dev and build are
  ignored by default. A dimension is defined by the architectural edge, which is the normal one.

  Matching every kind meant a published crate reaching 璇璣 only from `[dev-dependencies]` — a test using the
  reaction model, an ordinary thing to write — entered the set and turned the dogfood gate red for a reason
  unrelated to 三儀 ⊥ 三儀. The normal table only, now.

  **This one is a false positive, and the direction is the news.** The four revisions before it were false
  negatives: a hand-kept literal, a pathspec whose `*` crossed the separator, a dependency matched by manifest
  text rather than identity, and a `publish` array read as though every array meant unpublishable. One
  enumerator has now been wrong in both directions, which says the fault was never the direction — it was
  answering a question about *identity* with whatever evidence was nearest.

- **Every `publish` array was read as unpublishable, and one of them means the opposite.** Cargo reports the
  field as null when it is absent or `true`, as the empty array for `publish = false`, and as a **non-empty
  list** for `publish = ["registry"]` — a crate that publishes, to a named registry. Excluding every array
  excluded that third case, so a dimension restricted to a private registry left the enumerated set and its
  allowlist went unchecked.

  Only the empty array excludes now. Measured across all four states: a private-registry dimension is caught,
  `publish = false` is still correctly excluded, an ordinary publishable dimension is still caught, and the
  aliased dependency the previous revision closed is still caught.

  This is the fourth revision of one enumerator and the fourth time in the same direction. The first kept a
  literal by hand; the second read a pathspec whose `*` crossed the separator; the third matched a dependency
  by manifest text rather than identity; this one read the right field and got its **semantics** wrong. Each
  returned the correct set for this workspace while resting on evidence that did not support it — which is
  what makes a green result no evidence at all about a corpus.

- **The dimension enumerator judged manifest text where the question is dependency identity.** It tested
  whether a line began with the model crate's name, which a legal rename defeats: a dependency written
  `model = { package = "xuanji", … }` declares the same edge and begins with `model`. Measured — a crate
  written that way vanished from the enumerated set, the comparison passed, and its allowlist went unchecked.
  That is the exact false negative the enumeration was added to close, reintroduced one layer down.

  It reads `cargo metadata` now. Cargo reports a dependency under its real `name` with the alias in `rename`,
  so identity is read rather than inferred, and `publish` arrives as structured state instead of one of several
  manifest spellings. `--no-deps` restricts the answer to workspace members, which is what a dimension must be
  to be built and governed at all.

  The crate's own manifest comment already said this, beside the dependency that makes it possible: *a
  reaction's corpus comes from the build, never from a list anyone maintains*. Two revisions of this enumerator
  read text instead — first a pathspec that crossed the separator, then a name that was not an identity — and
  both returned the right answer while resting on the wrong evidence.

- **The dimension enumerator's corpus was every manifest beneath `crates/`, not the crate manifests.** git's
  default pathspec is `fnmatch` **without** `FNM_PATHNAME`, so `crates/*/Cargo.toml` crosses the separator —
  measured, 14 tracked paths where 8 are crate manifests and the other six are test fixtures. It returned the
  right set only because no fixture happened to name 璇璣.

  That is one edit away, and the precedent is already in the tree: a fixture carries a workspace-member
  dependency written precisely so it violates the edge under test. The next fixture needing 璇璣 for the same
  reason would have turned the dogfood gate red naming a fixture path, for a reason unrelated to 三儀 ⊥ 三儀 —
  reproduced by adding that dependency and watching the comparison gain a fourth "dimension" whose name was a
  fixture path.

  `:(glob)` makes `*` stop at the separator, and the enumerated set is unchanged. A crate name carrying no
  separator is now asserted as well, so loosening the pathspec again fails loudly on the shape rather than
  admitting a fixture as a dimension.

- A doubled article introduced by the previous commit, where an old sentence's article survived the edit that
  rewrote what followed it.

- **The dogfood gate's dimension list is held against the workspace, and it was the fourth of that class — left
  behind when three siblings were closed.** A dimension born and not added to that literal has its dependency
  allowlist unchecked while the gate stays green, so 三儀 ⊥ 三儀 goes unenforced in the one gate `PROJECT.md`
  calls the product running on itself. The check's own comment carried the measurement — removing `guibiao`
  from the literal leaves a `guibiao` allowlist naming `hunyi` green — and ended by saying it was neither held
  nor declared. It sat in `BACKLOG.md` and not in the register, which are different places: the register is
  where a deliberate stop lives.

  A dimension is now **enumerated**: a published crate depending directly on 璇璣, which is the architecture
  `PROJECT.md` already states — the reaction model every dimension sits above, and one a new dimension cannot
  avoid, since a crate expressing findings in some other vocabulary would not be a dimension. The comparison
  runs both ways.

  It was not left open as a declared bound, and the reason is that the choice was already made: the requirement
  added days earlier says a list a check judges by is held against its enumerator **wherever such an enumerator
  exists**. One exists here. Declaring a bound would have been exempting a requirement from itself.

- **The count of subjects carrying a merge serial is anchored to when it was taken.** Six live places — a
  wrapper comment, a gate's module documentation, two directions, `AGENTS.md`, and a specification — stated
  *nine* in the present tense for a set nothing produces and that can still grow through a declared bound,
  where a merge made outside the wrapper is not observed. `AGENTS.md`'s own rule is that such a figure is
  produced, or anchored to a past moment so a reader cannot mistake it for current.

  Anchored rather than declared as a census, deliberately: the number's whole job is *this happened repeatedly
  before the gate existed*, which is a claim about a moment. Making it track a live set would point it at
  something that should never grow — and if it ever did, the right response is a defect report, not an updated
  figure. The specification now says that outright, in the wording its sibling requirement already uses for a
  record.

- **A scenario stated a classification where a reader would take it for an observation.** The region
  requirement carried *two scans of one file disagreeing about comments* as a scenario whose `THEN` said the
  disagreement **is a defect** — a definition, not something anything looks for. Nothing detects it, and nothing
  can: seeing it needs the reaction that same requirement records as designed, measured and rejected.

  It is requirement prose now, beside the rule it defines, and the declared bound widens to name both shapes:
  a region decision never written and one a neighbouring scan contradicts are invisible for the same reason,
  so declaring one while giving the other a scenario said two different things about one silence.

  Found by re-running this window's own scenario-to-reaction sweep on the final tree rather than by reading —
  which is the point of the sweep. It is the second instance of this class in this window, and both were the
  same mistake in the same direction: a scenario is a claim that something looks, and a property nothing can
  look at belongs in the prose that says so.

- **Three constants sat beside something this repository already enumerates, with nothing holding them equal.**
  All three agreed, which is what made them worth closing before one drifted rather than after.

  The squash gate's Conventional Commit types were a second copy of the list `AGENTS.md` states in prose;
  diverge them and the wrapper standing in front of an unamendable record refuses a subject the contract admits,
  or admits one it forbids. The dogfood suite's example list was a second copy of `examples/`, and a directory
  absent from it is exercised by **neither** of that suite's directions nor by the workflow job that runs them —
  a false negative in the gate that runs the product against itself, which is the gate whose silence is least
  likely to be questioned. The publish wrapper's arrival matrix proved five of the thirteen arguments its parser
  forwards actually reach cargo, while the specification requires each admitted argument to be measured against
  the tool rather than reasoned about.

  Each is now held against its enumerator **in both directions**, because the two failures differ: a member the
  enumerator has and the list does not is something unexercised, and a member the list has and the enumerator
  does not is a declaration that outlived its subject and reads as coverage while defending nothing.

  The rule is stated once rather than as three guards. The identical shape was already held for one list — the
  wrappers — by a direction whose own documentation names this exact risk, which made these the third, fourth
  and fifth instance rather than three coincidences. Where a constant genuinely has no enumerator it now says
  so, as the attribution list beside the types already did; a silence that is written is not the same as one
  nobody considered.

  Two readers refuse rather than shrink: an unparsed contract clause is a cannot-judge instead of an empty set,
  and an unreadable parser arm stops the comparison instead of narrowing the enumerator — because an empty or
  shortened enumerator is a subset of anything, and would report agreement while holding nothing.

- **The coordinate reader missed the spelling every compiler prints, and skipped a file it could not read.**
  Both are corrections to the direction added one commit earlier, and both are the shapes that direction was
  written to refuse.

  It split on the **last** colon, so `path:line:column` left `path:line` on the left — neither empty nor a
  tracked path, so nothing matched. That is what `rustc` and `clippy` print, which makes it the form most
  likely to be pasted into a document. Splitting on the **first** colon and requiring everything after it to be
  digits, optionally separated by further colons, makes `path:N`, `path:N:M` and the elided `:N` one shape
  rather than three cases.

  And an unreadable tracked file was skipped silently while **another direction in the same file** refuses one
  outright, saying that a file a check claims to have inspected must have been read. Two sites in one file
  disagreeing about one question is the shape a requirement written in this same window names as a defect
  regardless of whether either currently admits a wrong answer. It refuses now, and the count of files actually
  read is asserted non-zero, so clean-over-nothing cannot pass as clean.

- **The sweep meant to watch every acquisition reported clean over a wrapper it read nothing from.** The repair
  that widened it one commit earlier was inert: it stripped environment assignments from the physical line the
  command substitution opens, and both wrappers put the tool three lines further down, so what remained was the
  continuation backslash and the tool test failed exactly as before. The continuation walk that would have
  reached the tool already existed in the same function — it ran *after* the corpus decision it needed to inform.

  The consequence was worse than the miss. `publish.sh` contributed **no** acquisition at all, and the direction
  asserted only that the unguarded list was empty — which an empty corpus satisfies exactly as a clean one does.
  Those are opposite facts, and one of the two wrappers standing in front of an irreversible act was being judged
  by an empty reading.

  Lines are now joined into one logical statement before the corpus decision, and each wrapper asserts that it
  contributed at least one acquisition. Both gate invocations — the one thing these scripts exist to run — are
  examined for the first time, and both are guarded.

  **The earlier repair shipped because its negative run used the wrong shape.** The probe was a single-line
  acquisition, which the broken reader handles correctly; the defect lives in the multi-line form, and nothing
  exercised it. A guard seen to fail against a shape the defect does not take has not been seen to fail. The
  negative run now takes the shape the wrappers actually use.

- **Two scans of one file disagreed about what counts, and the sweep meant to watch every acquisition never saw
  the one the wrappers exist for.** The exit-class check excluded comment lines when finding a violation-class
  exit and did **not** when assembling the window proving that exit sits inside the verdict branch — five lines
  apart, the same file, the same question. And its acquisition sweep tested the text immediately after a command
  substitution opens, so `var=$(NAME=value tool …)` fell outside its corpus entirely; that is the shape the gate
  invocation takes in **both** sanctioned wrappers, so the central acquisition was the unwatched one. Both
  guarded today, so neither was a live defect — the check's stated subject simply did not reach them.

  Both now read one classified region from `kanhe::region`, so a future third scan inherits the decision instead
  of re-making it, and the acquisition test looks past the environment prefix because the tool is what the
  property is about.

  **The mechanization was designed and rejected, which is recorded rather than deferred.** Refusing the inline
  comment marker inside the checks would refuse more legitimate sites than defects: some select commentary on
  purpose — a check asserting a doc comment directs its reader somewhere must recognize comment lines — and
  others parse a data format whose own syntax marks comments. So the requirement carries the rule, the
  classifier carries the adoption, and what is left is declared: a check that never wrote a region decision at
  all is invisible, because nothing can scan for an absence.

  **A third reported defect was refuted by measurement and no code changed for it.** A Definition-of-Done
  comparison was reported as satisfiable by a command sitting only in a workflow comment. It is not — the
  comparison is exact equality and a comment keeps its marker, so no comment can equal a command. Measured by
  commenting a real Definition-of-Done line out and running the unmodified check, which refused. Filtering there
  would have been defensive over-foolproofing of an unreachable state.

- **The publish wrapper's allowlist is named by its parser, not half-listed in prose.** `AGENTS.md` enumerated
  five forwarded arguments and five refused ones as though they were the sets; the parser admits more on both
  sides, and a second list that must agree with the first is the shape an allowlist exists to avoid. The
  document now points at the owner and marks its arms as examples.

- **A coordinate is refused wherever it is written now, not only in source.** The rule that a reference names a
  thing rather than a position reacted over every line-comment format, and whole-document prose sat outside it
  by construction — a positional *phrase* in a record narrates a past state, and separating that from a live
  reference is a judgement over prose this repository has designed, measured and declined.

  That reasoning covers phrases and stops there. A backticked path with a line number is not a phrase: it is
  decidable by **shape**, exactly as a bound id is, so refusing it reads nothing around it and reopens no
  declined judgement. And the argument against a coordinate never depended on the format — a position is not a
  name, and it is not one in any tense, so a record citing one serves its reader no better than a live clause
  does. Neither can be checked; both rot on any edit above them.

  It is **refused rather than resolved**, which is the difference from the other reference kinds. Those land on
  an identity and fail when the identity is absent. A coordinate cannot fail that way: a path with a line number
  is *valid* — the file does have such a line — while naming nothing anyone meant. Validity is the trap.

  Two existed, both in one backlog clause, both correct when written and both since landed mid-paragraph in
  unrelated entries; one was displaced by the very window that repaired the source-side instances. They now name
  the entries they meant. A claim in this same window that such citations were gone from *tracked content* is
  narrowed to *tracked source*, which is the scope that reaction actually held — the two in prose are exactly
  what the wider word walked past.

  Two shapes are worth recording. The elided form — a colon and a number, citing whatever file was named before
  it — escaped the first draft, which required a tracked path on the left; it is a coordinate whose reader has to
  carry the file in their head as well as the position, and it is matched now. And the direction reported its own
  explanation twice before the explanation stopped spelling a real coordinate, which is the rule this window
  already wrote for a reaction whose subject is text.

- **A bound id resolved inside a `(bound: …)` wrapper and nowhere else.** Written bare it resolved against
  nothing, and three occurrences across two crates — one of them published — had been naming a bound that
  exists under an id it does not have. They sat in the two places the bijection cannot look: a doc comment
  citing a bound, and a unit-test fixture constructing a `BoundDecl` that mirrors one, with the assertion
  round-tripping it. Neither is a *declaration*, and the bijection compares the two declaration sides.

  The fixture is the sharper case. Its pin name and its shape string both carried the bound's full wording
  while its id carried an abbreviated one, so a declaration had drifted from itself **inside a single
  constructor** — three fields describing one bound, one of them disagreeing, and nothing able to say so.

  The register now resolves a bare `<capability>/<slug>` wherever tracked Rust or Markdown carries it.
  Recognition is by **shape against the enumerated capability set** — a maximal run of path characters that is
  exactly that pair, with the capability read from the tracked spec directories — so a capability added later
  is covered without the reaction being edited, and a path is excluded by construction rather than by an
  exception list: read whole, `openspec/specs/<capability>/spec.md` is one run carrying three slashes and is
  not the pair at all.

  This is the fourth kind of reference in this repository and the only one nothing resolved; paths, `--exact`
  identifiers and wrapped bound references already did. It is **not** the detector over prose designed,
  measured three times and rejected: a bound id has a fixed shape and the set it must land in is *produced* by
  the declarations, so nothing here decides what a sentence means.

  Two things are worth recording because they cost a round each. The direction's first run reported this
  repository's own **test fixtures** as dangling references, since a fixture needing an invented slug had hung
  it off a real capability name; the fixtures now name a capability no spec directory carries. Its second run
  then reported the **doc comment explaining that fix**, which had spelled the offending token. Both are the
  class this repository already records — a text reaction reading its own text — and both were closed the way
  it settled on: recognize by shape, and describe the shape rather than quoting it.

  A typed census rode along: the bound-model reaction's doc comment said it read *the family's four sets* while
  it composes five. Four was right before the repository catalogs moved out of the shell. It carries no
  reaction of its own — the census direction reads tracked Markdown and this is a Rust doc comment — so it is
  repaired as prose and the number is gone rather than corrected.

- **A tracked script that deferred its verdict to nothing was invisible.** The direction enumerating this
  repository's scripts folded every `--exact` gate citation into one list and asserted that *list* was
  non-empty, so a script citing nothing passed as long as any sibling cited something. A script that names no
  gate is a script rendering its own verdict — the shape this window deleted 1562 lines of, `check_*.sh` gates
  paired with `test_*.sh` twins over a shared shell library, which once had a whole capability describing it.
  The way back was open and the enumeration that would have seen it was already running.

  Each enumerated script is now asked its own question and named in its own refusal. Per script rather than by
  counting, deliberately: asserting that the citation total reaches the script count passes for two scripts
  where one cites twice and the other not at all, which is the aggregate reading that failed one level up.

  **The consequence is stated rather than discovered: `scripts/` is now a closed category.** A tracked script
  that is not a wrapper cannot be added there while this holds — which is exactly what the capability already
  claimed about itself when it said `git ls-files scripts/` names only wrappers. The claim is held now instead
  of written. A convenience script belongs elsewhere, or the requirement is amended deliberately.

  What it buys is the **shape**, and the limit is written down beside it: citing a gate is necessary, not
  sufficient. A script could cite one and still judge afterwards. Closing that needs a judgement over source
  prose, the instrument this repository has designed, measured three times and rejected — while citing a gate
  is a *reference*, and reference resolution here is already mechanical.

- **The squash wrapper judged a body it held and merged a body it re-read.** It reads the body once, guarded,
  and hands that value to the gate; its final invocation handed the merge the *path* instead, so the two were
  the same only while nothing touched the file in between. What sits between them is a whole `cargo test` run —
  minutes of it on a cold target directory — and what lands cannot be repaired: a squash commit's hash is cited
  by the pull request's merge record, so amending the commit afterwards decouples the two. An editor autosave or
  a typo fix started after the wrapper was launched would have been recorded by a merge that judged something
  else.

  The value now travels to the merge, and the obligation is stated over **every** judged input rather than as a
  repair to this one. Three of the four already satisfied it — the subject travelled as a value, the repository
  was resolved once and named on every call, the head was captured before the commit set and pinned, and the
  live commit subjects were pinned through that head — and nothing said they were one set, which is how the
  fourth sat there through the rounds that built the wrapper. It is the local half of a pin the requirement
  already made remotely: a pull request that moved is refused, and now an input that moved on disk is never read
  a second time.

  A wrapper-owned temporary file would close the same race and is refused for a reason this window already paid
  for: it must outlive the `exec` for the tool to read it, so it could not be removed beforehand and no `EXIT`
  trap survives an `exec` — the leak fixed one commit earlier, reintroduced to fix something else. A value in
  `argv` carries an `ARG_MAX` ceiling a path does not, and that ceiling fails loud before the merge rather than
  recording the wrong thing.

  The guard asserts the body the merge would **record**, never which flag was spelled: a wrapper spelling
  `--body "$(cat …)"` at merge time re-reads and must still fail. Its controlled tool resolves a body the way
  the real one does, and its fixture body is deliberately multi-line, so the newline a curated body actually
  carries is exercised rather than assumed away.

- **Both wrappers cleaned up on every path except the one that finishes the job.** The verdict file added this
  window was removed by `trap 'rm -f …' EXIT`, and an EXIT trap does not run when `exec` replaces the shell
  image — measured, `bash -c 'trap "echo T" EXIT; exec true'` prints nothing while the same script without `exec`
  prints `T`. So the trap fired wherever nothing had happened and was skipped wherever the merge or the publish
  actually completed: against an isolated `TMPDIR`, three successful runs left three empty files and a failing run
  left none.

  Removed immediately before each `exec`, where the file's purpose is spent. The trap stays — it is what covers the
  failure paths — and `exec` stays, because the tool's exit status becoming the script's is deliberate. The cost was
  hygiene rather than disclosure: the file is written only when the gate has a refusal, and a refusal exits through
  the trap, so every leaked file was empty.

  Held by a direction per wrapper that observes an **isolated temporary directory as a whole** rather than one
  known name, so a temporary file added later is covered without the direction being touched. Both halves, since
  dropping the trap would satisfy the completing path while reopening every failing one — and the two negative runs
  are exactly that pair.

- **What the wrappers cannot see in the environment is now a declared bound rather than an aside.** The
  allowlists classify **arguments**, and cargo takes the same configuration from the environment: measured on
  cargo 1.96.0, `--target not-a-real-triple` and `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical
  `failed to run cargo's rustc probe` failure. So a value the publish wrapper refuses as an argument reaches cargo
  unexamined when it is exported instead.

  Declared as an under-reaction owned by the engine, not inherited: the wrapper *could* scrub the environment
  before invoking cargo. It does not, because doing so needs an allowlist **over the environment** and legitimate
  setups export `CARGO_HOME` and `CARGO_TARGET_DIR` — which set to admit is a decision the bound records instead
  of guessing. Pinned by a direction carrying both halves on one configuration key: the argument is refused, and
  the exported value passes *and arrives*, because a bound whose silence is not contrasted with a reaction is
  indistinguishable from a wrapper that refuses nothing.

  **The pin's own harness hid the second half at first.** It logged each invocation's arguments and environment on
  separate lines, and the gate's `cargo test` runs before the publish and inherits the same environment — so a
  search for the exported value matched the gate's line and passed even with the publish scrubbed. The negative
  run did not fire until arguments and environment were recorded together, per invocation.

- **The attribution rule missed the spelling the tools actually write.** `AGENTS.md` forbids a co-authored
  trailer, a generated-with footer, and a robot glyph — and the gate implemented that as three case-**sensitive**
  substrings. Git writes the trailer with only its first letter capitalised and GitHub renders it that way, so the
  one form most likely to appear was the one form not caught. Measured: both canonical lowercase spellings were
  accepted.

  Matched without regard to ASCII case now, and **by the shape each mark has** — with the recognition travelling
  beside the mark rather than as one rule over all three, because they are not the same kind of thing. A trailer is
  recognised at the start of a line, so a body that *names* one inside a sentence is not carrying it; without that,
  the gate would refuse the commit message of any change about this rule, which is the false refusal
  `repository-checks` already forbids it. A glyph is recognised wherever it appears, because reading it by position
  would let a subject carrying it mid-line through.

  **The first draft read all three by position and would have opened a miss** — a subject with the glyph after its
  Conventional Commit head was refused by the substring it replaced and would have passed. Caught by writing that
  case into the failure matrix before trusting the reading, and it is one of the two recorded negative runs.

  The gate holds the marks `AGENTS.md` names. That document also forbids "any other tool-authorship mark", which is
  not enumerable, so the open clause is stated as a reviewer's obligation rather than implied by a list that reads
  as complete.

- **A gate's verdict class now travels on a channel of its own, and the check that was supposed to hold it was
  green over the hole.** The wrappers grepped the gate's output for `(Violation)`, which put the parentheses in
  the shell and the variant name in Rust — two owners for one token. The check pinned the rendering's *arguments*
  and never the rendering: measured, changing a gate's format string to `merge message: {:?} — {}` left all five
  directions green while the wrapper's pattern matched nothing, so **every violation would have reported as the
  unjudged class**. That is verbatim the failure the check's own prose said it existed to prevent.

  The gate now writes its class to a file the wrapper names, and the wrapper reads that file. Nothing is spelled
  twice — the variable name and the class are `kanhe::verdict_channel`'s, compared against both wrappers — and a
  run that reached no verdict writes nothing, so *absent* means unjudged by construction. That also closes a
  latent second hole in the old channel: the stream it searched carries arbitrary tooling output, where a class
  could be read from text no judgement wrote. Recorded as latent rather than live, because inducing a compile
  error on the rendering line produced no match — the difference between a defect and a shape.

  **An unreadable body file was reported as a body the operator had written wrongly.** `-f` says a regular file is
  there; it does not say this process may read it. The read sat inside the gate's own invocation as
  `TIANHENG_MERGE_BODY=$(cat …)`, unguarded, so an unreadable file made the variable empty — and the gate refuses
  an empty body as a disagreement, *the squash body is empty*. Measured against the controlled harness the
  pre-change wrapper exits `0` there, reaching the merge with an empty body and only `cat`'s stderr to show for
  it. The body is now read once, guarded, before the gate, and the gate is handed the value rather than the path.

  Two guards were narrower than the rules they held. The violation-class count matched only lines whose trimmed
  text was exactly `exit 1`, so `if …; then exit 1; fi` escaped both it and the window check — which itself only
  examined the first site found. Both widened: the statement is matched at word boundaries, and every site must
  sit inside the branch that read the verdict.

  And a direction that skipped on the wrapper's own success swallowed the very defect it was written for: a
  wrapper that merges with an empty body succeeds, so the escape hatch for a root user read it as an environment
  that could not fail. Whether the condition is producible is now asked of a probe of the direction's own.

- **The wrappers reported as disagreements the very facts their own gates call unjudgeable.** Five
  could-not-read conditions in `scripts/merge-pr.sh` were split across both exit classes with no stated rule: an
  unresolvable repository exited `2` while an unreadable body file, an unreadable head, an unresolvable
  pull-request number and an unreadable commit set exited `1`. Two of those are facts `merge_message_gate::judge`
  types the other way — cannot-judge for an unavailable title and for unavailable commit subjects, *which is not
  the same fact as a subject that disagrees*. The contract was stated twice in the tree, in `law.rs`'s outcomes
  and in the sibling wrapper's own header, and neither reached the sites.

  One rule now, chosen in one place per wrapper: `1` only where a gate ran and reported a disagreement,
  `2` for everything else — a misconfigured invocation, an input that could not be read, and **a gate that did not
  run**, which is the sharpest case of the class since its own message says so in as many words.

  Where a wrapper reports the class of a failing gate it now reads the class the gate rendered, and treats
  anything unrecognised — a compile failure included — as unjudged. The token it matches on is held against
  `refusal::Kind`'s own rendering by a new check, because a wrapper grepping a string a gate prints is two places
  that must agree.

  **Two further defects came out of making the verification able to see the class.** The helper covering three of
  those sites asserted only that the wrapper failed, which cannot see `1` from `2` — the split survived by being
  invisible to its own guards. With the class pinned, a failing commits read turned out to exit **91**: an
  unguarded `var=$(gh …)` under `set -e` exits with the *tool's* status and only the tool's stderr, so the class
  reported was neither of the wrapper's two and the operator got gh's words for a fact about the wrapper. Four
  acquisitions were unguarded; all four now report in the wrapper's own words and class.

  The new check found one more on its first run: an unused helper carrying an `exit 1` that the author had just
  added. Negative runs against the previous tree name all six mis-split sites and all four unguarded
  acquisitions.

- **The merge is now pinned to the head the gate read its evidence from.** The gate judges the squash body
  against the pull request's live commit subjects as they are while it runs; the merge happens afterwards. A
  commit pushed in between changed the set the body had to equal, and nothing noticed — `gh pr merge` would
  record the approved body over a commit set that had moved. The window was small and the consequence permanent,
  which is the same trade every other guard in front of this act was written for.

  The wrapper obtains the head and supplies `--match-head-commit` itself. A caller's is refused: gh takes the last
  spelling of a repeated flag, so a chosen SHA would replace exactly the link the pin exists to make. That is the
  same reasoning that refuses a caller's subject and body.

  **The head is read before the commit set, and the order is the guarantee.** Read first, a push in between
  leaves the commit set ahead of the pinned head and the merge is refused — it fails closed. Read after, the pin
  would carry the new commit while the gate judged the older set, so the merge would proceed and record a body
  missing it — it fails open. Two calls, either order, opposite guarantees; the direction asserts the order in the
  log rather than only the presence of the flag.

  A head that cannot be read stops the wrapper before the gate and the merge. An unreadable head is not a head
  that has not moved.

  Three negative runs, and the third one's own fixture was the first thing it found: dropping the flag, reversing
  the read order, and removing the unreadable-head refusal each fail their direction — but the last initially
  failed because the controlled `gh` did not know that mode and exited 92, which is a fixture reporting a guard it
  never exercised. Corrected, it reports the wrapper running to a successful merge with no head read at all.

- **The positional-reference rule kept a second extension list, one change after the declaration that ended
  them.** It swept `.rs` and `.sh` by extension while its sibling path check derived its corpus from the format
  declaration — so `.toml`, `.yml`, `Cargo.lock`, `CODEOWNERS` and `.gitignore` went unswept, every one of them
  source where a positional phrase rots exactly as it does in Rust, and nothing in the Markdown reasoning reached
  any of them. The floor was clean, so it read as covered.

  The scope now derives from the same declaration: a format admitted to the corpus is swept for **both**
  properties or for neither. Markdown stays out **by construction** rather than by omission — it is the format
  classified as whole-document prose, so it is not a line-comment format and no exclusion has to be written for
  it. The fixture direction gained a `.toml` comment beside the Rust one, so the corpus is shown to be derived
  rather than to be the two extensions it listed before.

- **The allowlists asked whether an argument moves what the gate judged, and never whether the tool honours
  it.** Both wrappers had an instance, and the publish one had already shipped inside this window.

  `scripts/publish.sh` admitted `--package` while writing `--workspace` unconditionally. Cargo maps that
  combination to *all packages* and emits no warning: measured on cargo 1.96.0 with the identical selection
  flags, `--workspace --package xuanji` selects **8** and `--package xuanji` selects **1**. So the selector
  admitted precisely so a partly completed publish could resume — crates.io accepts the six one at a time —
  instead re-attempted the whole workspace, in front of the one act that cannot be undone. `--workspace` is now
  the **default** selection rather than a constant written over the caller's, and `--package` replaces it and may
  be repeated.

  `scripts/merge-pr.sh` admitted `--auto` and `--disable-auto`, both of which pass gh's own argument validation.
  `--auto` merges *after* the gate has read the evidence — gh: "Automatically merge only after necessary
  requirements are met" — so a commit pushed before the deferred merge lands changes the pull request's commit
  set while the captured subject and body do not, and what gets recorded is no longer what was judged.
  `--disable-auto` is not a merge at all: the wrapper would run its gate, reach gh, and exit `0` having recorded
  nothing. Both refused. What stays admitted is `--admin`, `--delete-branch`, and `--match-head-commit` — the
  last being the only one that *strengthens* the claim, since it refuses the merge if the head moved.

  **The direction guarding the publish selector was itself the defect.** It asserted the string the wrapper typed
  — `publish --workspace --package xuanji` — against a controlled `cargo` that only logs its arguments, so it
  could not see what cargo discards, and it pinned as intended the exact invocation that voided the flag. Its
  name said the reason out loud: *the workspace is always named*. It now asserts the selection cargo would
  honour, and the absence of `--workspace` as much as the presence of the selector. The measurement a controlled
  executable cannot make lives beside the classification instead, against a named tool version.

  Admitting an argument now takes two questions, and the specification carries the second: does the tool honour
  it as the wrapper composes the invocation, and does it perform the act at the moment the gate read its
  evidence. Both directions were widened to assert that **every** admitted argument arrives rather than one of
  them.

- **A reference that names a position is now refused in tracked source, and the sweep that found them was
  narrower than the rule.** There is a ladder: an intra-doc link is checked by the compiler, a path by the
  reference gate, a path with a line number by nothing — and a reference naming only a position is not even a
  name. Two were measured in this window off by 86 and 98 lines, the second written after the first had been
  corrected, which is the criterion `scripts/publish.sh` states for itself: a rule stated and then missed needs a
  check rather than another sentence.

  Seven instances were found by hand and repaired — a signing-probe citation now an intra-doc link, and six
  rephrased to name the construct. The check's first tree-wide run then found an eighth the hand sweep had
  missed, because that pattern was written with the singular article only while the instance used the plural.
  **The instrument that declared the class handled was the one thing not measured**, which is how this class
  keeps returning.

  The corpus is comment lines, decided by the same rule as the sibling path sweep. That settles the check's own
  self-reading: every shape it refuses lives as a string literal in its directions, on an executed line, where it
  cannot be read as a reference — a position rather than an exemption, since nothing can hide a comment inside an
  executed line. Markdown stays outside, stated in the requirement: in a record a positional phrase narrates a
  past state, and separating that from a live reference is a judgement over prose this repository has measured
  and declined.

- **The reference gate read every format but the ones the wrappers and CI are written in — because it kept two
  lists.** An extension filter decided which files to open; a marker rule decided which of their lines to read.
  A format could sit in one and not the other, and shell did: the marker rule had known `#` all along while the
  extension filter never admitted `.sh`. The files that left unread are `scripts/merge-pr.sh` and
  `scripts/publish.sh`, whose comments cite the Rust gates they sequence **by path**, where a renamed test target
  is exactly what rots a citation. `.yml` was unread the same way, and CI is where this repository's own gate list
  is duplicated.

  Adding one extension per discovery would have been the third turn of the same handle — this window replaced two
  argument denylists with allowlists for exactly this reason. So there is now **one declaration**: every format
  the repository tracks is named as whole-document prose, as prose behind a stated line-comment marker, or as
  carrying no prose at all, and both the corpus and which lines get read derive from it. A format the repository
  holds and the declaration does not name **fails**, naming that format. Defaulting either way is the trap: a
  silent *no prose* reads a new format as having none, and a guessed marker asserts one it may not have.

  A shebang reaches the reader and names `/usr/bin/env`, an absolute path outside every prefix this gate
  recognizes, so it is not a reference — the shell probe carries one to say so. Measured when this landed:
  neither shell nor YAML named an absent path, so both were silences rather than backlogs — the kind that only
  report clean because nobody looked.

- **The publish wrapper had the same denylist, no direction holding it, and real arguments walking past.** One
  window after the merge wrapper's argument handling was rebuilt as an allowlist, `scripts/publish.sh` still
  forwarded everything except `--manifest-path` — and carried the same sentence, *a guard catching one would be a
  guard catching neither*, while arguments walked past it. Measured against a controlled `cargo` on the tree
  before this change, each of these reached `cargo publish --workspace` with the wrapper exiting `0`:

  - `--no-verify`, dropping cargo's own build of the packaged tarballs at the one moment nothing can be undone;
  - `--allow-dirty`, packaging content no commit holds while `.cargo_vcs_info.json` still names a commit that
    does not contain it;
  - `--exclude <spec>`, spliced after the `--workspace` the script writes itself, so the invocation reads as the
    whole workspace while publishing less;
  - `--config <KEY=VALUE|PATH>`, an arbitrary configuration override that can name a whole file and reach every
    other refusal here — while the script's own comment claimed every destination-changing argument was *written
    down rather than implied*;
  - `-Z`, the feature and target selectors, and a flag no cargo has.

  Now an allowlist, by the same question its sibling asks: does the argument move what the gate judged, or what
  the act records? Admitted are the arguments that change only whether and how the publish proceeds — `--dry-run`,
  `--package` (how a partly completed publish resumes, and it records which crates it named), `--locked`,
  `--offline`, `--frozen`, `--keep-going`, `--jobs`, `--color`, `--target-dir`, `--verbose`, `--quiet`, and the
  destination-side `--registry` and `--index`, keeping the reasoning that admitted those two. One spelling each,
  values as the argument after the flag. Two decisions changed rather than carried: `--allow-dirty` was forwarded
  on the ground that the source gate refuses a dirty tree upstream anyway, which makes it inert rather than safe;
  and `--token` is refused pointing where cargo 1.96.0 points, having deprecated it in favour of `cargo login` and
  environment variables. A misconfigured invocation now exits `2`, the usage-error class, rather than `1` — which
  is what a gate that ran and refused exits.

  **And the refusal it did have was held by nothing.** No test had ever run this script. The new controlled
  direction replaces `cargo` on `PATH`, so no upload and no build can occur, and runs both ways: a sample of
  arguments is refused with the cargo log empty — before the gate, since the gate is itself a `cargo` invocation
  that would appear there — and an admitted argument still reaches the publish with `--workspace` always named.
  The specification's allowlist requirement now covers both wrappers as one law rather than being restated per
  script.

  A doc comment carrying the corrected claim was found by the same sweep: the merge wrapper's
  repository-selector direction said all three spellings were covered *because a guard catching one is a guard
  catching neither*, which is a claim about a class made by enumerating a sample of it. What that direction holds
  is the diagnostic an operator reads; the class belongs to the allowlist.

- **What may reach the merge is now an allowlist, and the three leaks are one defect.** The sanctioned merge
  wrapper forwarded any argument it did not recognise. Three separate holes came out of that one shape: a
  `--repo` flag, a positional pull-request URL, and every short spelling of the flags its long-form arms named.
  The last is the sharpest — `gh` accepts `-t` for `--subject` and `-F` for `--body-file`, the wrapper splices
  forwarded arguments after its own, and `gh` reads the **last** occurrence of a repeated flag (measured on gh
  2.95.0: `--body-file A -F B` and `-F A --body-file B` both take B). One unlisted spelling would have replaced
  the very message the gate had just approved, at the one moment nothing can be undone.

  Patching a fourth spelling would have bought the fourth spelling. The wrapper now forwards only what it names,
  so an argument it does not know — including one a future `gh` adds — is refused by default. The admitted set is
  decided by one question: does the argument move what the gate judged? A flag that changes whether the merge may
  proceed may pass; a flag that changes the message, the strategy, the repository, or the author it records may
  not. Each is accepted in one spelling with its value as a separate argument, because parsing a tool's glued and
  equals forms is exactly what a denylist has to get exhaustively right and an allowlist does not. This family
  already argued it in its own law — an allowlist is always stricter than a denylist — and the wrapper was the
  place still taking the other side.

  Held by a direction that runs both ways against a controlled `gh`: a sample of spellings is refused with
  nothing executed, and an admitted flag still arrives at the merge — without that second half the first is satisfied by
  a wrapper that refuses everything. The property under test is the default, not the list, so the refused set
  includes a flag no `gh` has: on the tree before this change, the known spellings were still caught by the arms
  that name them while the unknown one ran the merge to completion and exited 0.

Changes to this repository's own governance machinery, which ships in no package and which
no adopter runs. They are here rather than under the adopter headings above because
`CHANGELOG.md` is the adopter's document; the rigour they carry is unchanged.

- **An independent review of this window found five, and the blocking one told an operator the wrong rule.**
  `merge_message_gate`'s attribution refusal served two recognition shapes with one sentence and stated only
  the trailer's: *naming the mark inside a sentence is not carrying it; a line that begins with it is*. A
  glyph is matched wherever it sits, mid-line included — which the shape's own doc says — so an operator
  refused for `fix(x): 🤖 wrote this` was handed the rule that would have permitted it, in front of a record
  no rerun amends. The rule now travels with the recognizer that refuses by it, in the same array as the mark,
  and a direction asserts each shape's own sentence and the absence of the other's. Negative run: with both
  shapes returning the trailer rule, it fails.

  **A figure typed into a doc comment had drifted past every reading.** `refusal.rs` said `violation("…")`
  constructs an identity-less refusal *seventeen times*. It was about right when written and nothing could
  re-measure it: `census::sweep` reads tracked Markdown, and this is Rust. The figure is gone and the shape is
  stated — *at every site outside `src`* — with why no reaction reached it.

  The first repair replaced the figure with three fresh ones and stated them in the present tense, without the
  counting rule that would let anyone reproduce them — so they read as facts about the tree and were checkable
  against nothing. They are gone too. A count that no reaction can produce does not become safe by moving to a
  dated entry; what a dated entry can carry is what was measured and how, and *seventeen* is here as the thing
  that drifted rather than as a figure anyone should trust.

  **`region`'s governing sentence was absolute and false inside its own crate.** *A corpus is never handed to
  a recognizer as `&str`* — while Markdown readers here still take one: three in `release_coherence_gate`
  and `restatement::document_offences`. Narrowed to what is true, executed text, with the residue stated where
  it was claimed away and measured: 0 fenced blocks and 0 HTML comment spans in `CHANGELOG.md`, 0 of each in
  every spec, and no fenced block in tracked Markdown names a crate together with its whole allowlist. Filed WATCH rather than declared a bound, because it is a debt about which reader a call site
  chose rather than a claim a reaction makes about its own limits — and closing it needs `Prose` to carry
  positions first, since `document_offences` reports the line a block starts at.

  **`Source::header` searched for a newline the first line does not have.** A document opening with `## `
  matched nothing and came back whole as its own header, and both consumers err toward a false pass from
  there. Closed with a position test rather than declared, and pinned by a direction that asserts the offset
  form beside it so it cannot pass by the search having been removed. Negative run: with the branch removed,
  it fails.

  **Four command runners for one job, two of them the shape this crate documents as not kept.**
  `bound_register_parse::search`, `bound_register_parse::must`, `gate_identity::run` and `pin_bites::run` each
  took the program inside `args[0]`, none routed through `hermetic`, and *a failed read is not an empty result*
  stood verbatim at four sites in three files, already diverged in what each printed beside it. The two that
  share a contract are one implementation in `hermetic_git` now; the sentence has one owner; and the list form
  is admitted where a caller composes the program at run time — `pin_bites` chooses `cargo` for a mutation
  build and `git` for a record read, `gate_identity` chooses `git` to enumerate and `cargo` to list a target's
  tests — unpacked in one place that turns an unstated index panic into a stated one. The doc that declared one
  shape universally kept now names both and says which is for which.

  This paragraph first said *three* runners and named two of the four, and the doc comment beside it said
  *three* and *one that cannot do otherwise*. Both counts were taken from inside the repair, over the set that
  had just been edited, instead of from the base commit — which is the same authoring failure as the typed
  census below, at two sites in one change. Corrected here and replaced with the enumeration there.

  Recorded because it is the machinery working on this entry's own work: `reference_integrity` refused two
  positional references — *paragraphs up*, *sentence above* — written while explaining the four repairs above
  it. Both name the item now.

  No published API, outcome, report, exit class, or manifest moves; every repaired site is in a crate that
  ships in no package.

- **The prelude's promise now has a reaction, and the requirement it holds did not exist.** The compile
  contract's claim to name the whole promised surface was a header comment: the one requirement about prelude
  additions names three specific types and nothing generalised it, so a window's worth of additions could
  reach the prelude unchallenged. `crates/kanhe/tests/prelude_promise.rs` reads the promise from the prelude's
  own block — not from any sibling `pub use super::{…}`, of which the shell carries several — and refuses when a
  promised member is mentioned nowhere in the contract. The relation is **containment, not equality**: the
  contract legitimately names root imports and its own helpers, and demanding equality would refuse it for
  being a test. Both cannot-judge directions are seen to refuse rather than assumed to: a promise that parses
  to nothing, and a contract that yields no identifier.

  Where the check stops is declared rather than left to be discovered: it asks whether a promised member is
  *mentioned*, so a mention that compiles nothing counts. Deciding that a mention is load-bearing is the
  judgement over text this repository has designed, measured and rejected, and what makes a mention bite is the
  compiler.

- **A publish claim promised more than any mechanism delivers.** It read *"Every crates.io publish is now made
  only from the signed, annotated-tagged commit at the live tip of `main`"* — but a bare `cargo publish` invoked
  outside the sanctioned path meets no check at all, and this repository declares exactly that as an observation
  bound. The entry now claims what the mechanism does: a publish **made through that path** is refused at the
  source unless it runs on the right commit, and the edge is stated beside it rather than left for a reader to
  discover.

  Repairing it tripped the reaction that reads adopter-facing entries for this repository's own machinery, whose
  advice was the right one — *state the guarantee and drop the filename*. An adopter never runs the wrapper; what
  they need is which commit a published tarball records, and that survives naming nothing internal.

- **Three claims a reader could not follow to their evidence.** The census reader justified reading counts as
  words with *"two of the four censuses declared here"* and cited two example phrases — and the live declared set
  holds **two** censuses, carrying **neither** phrase, so a reader following the comment to the declaration finds
  nothing it describes. The count was true when written and both declarations it counted have since been retired;
  it now carries the historical qualifier its own sibling reason two screens down already had, and cites the
  phrase the live set does carry. The *need* is unchanged, and it survives the instances.

  Two entries elsewhere described `tianheng::observation_bounds()` as present — one listing the shell beside the
  three dimensions that export it, one saying it returns an additional entry. It was retired before its first
  published appearance, and the two entries that say so are accurate and untouched; these two now say what the
  tree holds. Measured before editing: `guibiao`, `hunyi` and `louke` each define it, `crates/tianheng/src/`
  neither defines nor re-exports it.

- **A set was computed on every run and read by nothing, and `-D warnings` could not see it.** The
  release-coherence gate's document grammar collected the release-section names beside the headings and the
  breaking set; `judge` consumes the latter two and never the first. `dead_code` does not fire, because
  `insert` counts as a use of the field — so a workspace that refuses every warning passed over a collection
  built and discarded. It is deleted, with the `continue` beside it kept and its own reason written down: a
  section heading carries no `### …` and marks no break, so the arms below must not see it.

  This is the same class as the five module-level `allow(dead_code)` retired earlier in this window, reached the
  other way round: there the suppression was explicit and inert; here nothing was suppressed and the lint was
  simply blind.

- **A tail comment was executed text, in the reader whose whole subject is that a comment is not.** `Executed`
  dropped a line only when the comment marker was its **first** token, so placement decided the verdict: a bare
  marker line naming a document did not satisfy *this holder names its document*, while the same name written
  after `let n = 1;` did. The region reader's own module doc opens by naming that class, and one call site had
  already noticed and stripped tails **by hand** — one rule, two implementations, the absent one in the file that
  names the defect.

  **The obvious repair was measured and refused.** Cutting at the first marker corrupts twenty-six lines here,
  among them `"https://…"` constants, a string carrying `"/// …"`, and the region reader's own `comment: "//"`
  field — a false negative in the direction the Core Contract forbids, bought to close a false positive. The
  marker is recognised **preceded by whitespace or at line start** instead, so a marker glued to what precedes it
  begins no token and is not a comment. A third candidate, requiring the surviving head to carry non-space
  content, was measured too and separates nothing here, so it is not adopted.

  The hand-rolled strip is deleted, and the residue is declared beside the region's others: a marker preceded by
  whitespace **inside a string literal** is cut, because telling one from the other needs the string-literal
  lexing this tree has defeated repeatedly — a direction `observer-protocol` already declares.

- **The gate-identity join compared a name `libtest` never compares.** The check reads a target's registered
  tests from `--list` and asks whether the wrapper's `--exact <ident>` names exactly one of them — but it
  truncated each listed name to its last `::` segment, while `--exact` matches the **whole** path. Inexact in
  both directions: a gate moved into a module lists as `inner::the_gate`, truncated to `the_gate`, matched the
  citation and read as registered — while `--exact the_gate` selects nothing and `libtest` exits 0 over it,
  which is the condition the check exists to catch. And a leaf shared by two modules truncated to one name twice,
  refusing a citation `--exact` resolves to exactly one test.

  The doc above it asserted the opposite — *"`--list` is the set `--exact` filters against, which makes the join
  exact"* — and a unit test named `a_registered_name_is_its_last_segment` **pinned the truncation as intended
  behaviour**. It read clean because both live citations sit at file scope, where truncation is the identity
  function and the comparison is `f() == f()`.

  The listed name is now carried whole, both directions have a direction of their own, and the shared fixture
  gained a genuine duplicate so *registered twice* is tested by a name really registered twice rather than by two
  names collapsing. **No live verdict moves**: the real citations are unqualified, which is exactly why nothing
  had noticed.

- **Line-number citations are gone from tracked source, and the reason is that nothing was checking them.** A
  path is held by the reference gate; an intra-doc link is held by the documentation pass; a path with a line
  number is held by nothing, and rots on any edit above it. Three existed. One had rotted twice over — the line
  it named holds an unrelated statement, and the comment it attributed to that file no longer exists anywhere in
  the code, surviving only in the documents about it. The other two still resolved, which is the honest result
  rather than a tidy one: the class is not uniformly rotten, it is uniformly unwatched.

  Each moved up as far as the tooling allows. The doc comment now links the module, which `rustdoc` resolves
  under `-D warnings`, and names the function in prose — the function itself was tried first and rustdoc declines
  a link to a private one. The plain comment keeps its path, which the reference gate does hold, and gains the
  function's name in place of the coordinate. And the backlog entry stops citing a file for a record that file no
  longer carries: the rejection it describes was written in the check's own comments, and those comments did not
  survive the change that overturned them — which is that entry's own argument for a durable record, made by the
  disappearance rather than by assertion.

- **A repair of an unverifiable claim was itself unverifiable, and the second attempt is measured.** The census
  reader's word-parsing reason said *"two of the four censuses declared here"*; the repair that followed implied
  the declared set had once held four and shrunk. Measured across the whole history of the declaring test, it has
  **never held more than three** — so the correction replaced one figure a reader cannot check with another.

  No count replaces it. How many were declared at that moment is a figure about a past state that nothing
  produces, and it has now been wrong twice; the sentence says what happened without it. What the reason cites
  instead is evidence still in the tree: `{} of them in` **was** declared here and retired for matching an
  unrelated sentence — the phrase-specificity assertion in the sweep records that retirement, three lines of code
  away from the reason that had lost track of it — and the document sentence it existed for survives verbatim in
  two tracked documents.

  The one positional reference the draft carried is gone too, replaced by a link the compiler resolves. There is
  a ladder here: an intra-doc link is checked by the documentation pass, a path by the reference gate, and a line
  number by nothing at all.

- **Two guards over one condition, and only one of them could ever fire.** The self-law's shell-boundary
  selector asserted that exactly one allowlist matched, then extracted it with an `expect` standing over the
  state the assertion six lines above had just made unreachable. Two messages for one fact, one of them dead —
  the defensive over-foolproofing of an impossible state the minimalism bound forbids, and the shape this window
  already retired from the baseline writer's temp-file guard.

  They are one check now, and the arity is the **type's** obligation rather than a second reader's: converting
  the matches into a one-element array makes the extraction and the guard structurally incapable of disagreeing.
  The diagnostic got better rather than merely shorter — the failed conversion hands the vector back, so the
  refusal says how many were declared, which neither of the two it replaces could.

- **The same hole, through the positional selector this time.** Refusing a `--repo` flag did not close it: the
  wrapper's first argument was checked only for being non-empty and not flag-shaped, so a **pull-request URL**
  passed — and `gh pr view` and `gh pr merge` follow a URL to its own repository while the live-commits endpoint
  was built from a placeholder `gh` expands from the working directory. The gate would judge one pull request's
  commits and the merge would record another's.

  A URL is refused, and a number or a branch name still is not: neither names a repository, so both resolve
  against the one this checkout is. Requiring the selector to be numeric was declined — the wrapper's own
  contract says it *resolves an accepted selector* to a canonical identity, and a branch name is one it can
  resolve without moving what is judged.

  **And the four references now name one repository explicitly.** The endpoint always named one, implicitly;
  the three `gh pr` calls named whichever the selector resolved to. Four references defaulting to the same place
  is agreement by circumstance, and a fifth call added later would inherit the circumstance rather than the rule.
  The identity is resolved once and passed to every call, so the wrapper owns it and a caller cannot supply one.

- **The merge wrapper could have judged one repository and merged another.** A repository selector — `--repo`,
  `--repo=…` or `-R` — fell through to the passthrough array, which reaches only the final `gh pr merge`, while
  the title, the canonical pull-request number, the live commit subjects and the gate are all read from the
  repository the wrapper runs in. One argument would therefore have it judge pull request N here and merge pull
  request N somewhere else, at the one moment nothing can be undone. Measured rather than reasoned: with the
  selector passed through, the wrapper ran to completion and **exited 0**.

  Refused before any evidence is read, and the direction that holds it asserts the **order** rather than the
  exit code — the controlled `gh` logs every invocation and the log must be empty, because a refusal printed
  after the title had already been fetched would still exit 2 while having read the wrong repository's evidence.
  What the refusal covers is no longer this entry's to state: it named the long spellings it had added arms for,
  which left gh's glued and equals forms of the same flag open, and the allowlist entry above is what settles
  the question by not asking it per spelling.

- **The same migration, finished — and the sweep that declared it done was the reason it was not.** The pattern
  used to find retired exit-code prose required the word *exit* and a digit **adjacent**, and the vocabulary had
  already half-migrated: statements saying `SHALL exit cannot-judge` carry no digit, and one saying `SHALL
  likewise be` `2`, `not` `1` carries no *exit*. Every such statement escaped, in both directions, and the
  residual grep that reported zero was reporting on the wrong instrument.

  Swept again by each half **independently** rather than by their co-occurrence: six retired statements, four of
  them in a file the previous pass had already edited — including a whole requirement binding regeneration to
  "the same exit contract as judgment — 0 clean, 1 violation, 2 cannot judge", and another binding "the exit
  contract" to every path out of the reaction. Those are the verdict contract, and they now say so.

  **The spelling was split, and that is part of why the miss happened.** The same verdict appeared as both
  `cannot-judge` and `cannot judge`, so a sweep written against one form was blind to the other; the noun is now
  spelled one way throughout, while the ordinary verb phrase — *the reaction cannot judge* — is left alone,
  because it is a sentence rather than a value. Kept for the same reason as before: `libtest`'s own `0`, the
  product's live `0`/`1`/`2`, and the historical sentence explaining why a shell gate's status had to move into
  a type at all.

- **Two specifications and `AGENTS.md` still stated a gate's contract in exit codes it no longer has.** Both the
  publish-source gate and the bound register run as `cargo test`, so their process status is libtest's; the
  distinction they actually carry — a source that **disagrees** versus one that **cannot be judged** — lives in
  their shared result type. The normative prose said `exits 1` and `exits 2` anyway, which tells a reader to look
  for a status the run does not produce. Seven statements now name the two verdicts, and each specification's
  Subject says how its gate is invoked, so the vocabulary has somewhere to be anchored rather than inferred.

  **What was deliberately left alone**, because a blanket rewrite would have broken it: a nearby sentence
  describing `check-ignore`'s exit 1, and another describing `grep`'s ordinary no-match exit — those *are* process
  statuses, of other programs, and correct. One line held both kinds and only its first half moved. The product
  specifications keep every exit code they state: `tianheng check` really does answer `0` clean, `1` violation,
  `2` constitution error, and that is the adopter's contract rather than a retired one.

- **The promise reader dropped a member it could not parse, which narrowed the promise by exactly what it failed
  to read.** Measured on a mixed list: `{Alpha, runner::Format, Foo as Bar, a::{B, C}, Beta}` parsed to
  `{Alpha, Beta}` — three of five members gone. And dropping does not merely lose a member: with the survivors
  both mentioned, the check reports the promise **kept**, so a would-be disagreement becomes a clean verdict. The
  prelude is a flat list of identifiers today, so nothing was wrong; what was wrong is that nothing would have
  said so, in the check whose whole subject is a promise narrowing unobserved.

  Neither declared as a bound nor closed by widening. Declaring would put a false negative in that check;
  widening means new extraction rules — last path segment, post-`as` name — in a hand-rolled reader, with no
  pressure asking for them. The member is **refused** instead, named in a cannot-judge: no new rule, nothing that
  can narrow, and an author who adds such a form meets a refusal rather than silence. That is the same repair
  this window already applied to three other reads, arriving at a fourth that was written after them.

- **A declared bound was pinned by a test that defends a different direction, and nothing could have said so.**
  The bound says a promised member named **only in a comment** counts as named — `UnderReacts`, whose
  `demonstrates()` is `DoesNotReact`, a direction whose evidence is *silence*. Its citation named a test with no
  comment anywhere in it, asserting that a substring of an identifier is not a mention: a **reacting**
  distinction, true and useful and about something else. One perturbation separates them — making the check
  strip comments, the behaviour the bound says it does not have, kills only the new pin while the old one passes
  untouched.

  The citation now names a test that puts a promised member in a comment and nothing else, with a control
  showing the same contract without the comment is reported. The tokenizer test stays, as what it always was: the
  contract test for identifier extraction.

  **The blind spot that allowed it is declared rather than left for the next one.** `Extent::demonstrates()`
  names the direction a defence must show and reaches the projection label and the contradiction classification
  beside it; nothing compares that prediction with what the cited test asserts. Deciding what a test demonstrates
  from its source is a judgement over code of the kind measured and rejected over prose — and unlike a citation
  that never runs or never bites, there is no reaction here whose gap a fixture could exhibit, so it is declared
  unpinned with a tracker. It is the sibling of *a rationale that contradicts its extent*, one step over: the
  prose beside an extent was already free to disagree with it, and so, silently, was the test beneath it.

- **Two observation limits were real and undeclared, and each is now declared instead of removed.** The census
  sweep reads tracked **Markdown**, while its requirement said "every tracked document" — the requirement was the
  outlier, since the register spec and this changelog both already said Markdown. Widening the corpus was measured
  rather than argued: this repository's Rust sources carry census phrases **as fixture input**, where the figures
  are a parser's expected output and deliberately arbitrary, so admitting them would report a test asserting its
  own parser as a drifted document. The requirement now says what reacts, and the narrow corpus is a declared
  bound with a pinning test that contrasts the silence against a reaction on the same content.

  The release-coherence classifier reads whether a section *contains* `**BREAKING**`, so a section that merely
  **discusses** the marker is required to carry a `### Migration` it does not owe. **The reach is kept
  deliberately.** Recognising the marker at an entry's start would remove a refusal an author can argue with and
  buy a false negative in the floor: a real break whose marker sits anywhere but the first token would stop being
  observed. The Core Contract forbids exactly one bug and it is the false negative, so the over-reaction is
  declared with a fixture that shows it — a body announcing nothing, refused anyway.

- **One bound was projected in two wordings, and only the typed one was number-free.** The subject-coverage
  bound's declaration in Rust already said its blindness would cost *a claim per capability*; its spec scenario
  said *thirty-six claims*, and that scenario's body is quoted verbatim into a generated projection — so one
  hand-typed word travelled into a document whose own header says it is generated. The figure was right the day
  it was written (`git ls-files 'openspec/specs/*/spec.md'` returns exactly that many) and held by nothing, in a
  window that added eight capability specs. All three renderings now say what the declaration says, and the
  projection is re-blessed from them.

- **The sanctioned merge wrapper stopped silently when a flag was given no value.** `--subject` or `--body-file`
  passed as the final argument made `shift 2` return non-zero, `set -e` took that as the exit, and the wrapper
  ended with **no output at all** — in a script where every other refusal prints `merge message: …` and points at
  what to do instead. A missing flag value is an observable misconfiguration, which is exactly what the
  minimalism bound says to fail loud on, and the operator meets it at the one moment before a record lands and
  stops being repairable. Both arms now validate before shifting, name the flag, show the usage, and exit `2` —
  the same class the wrapper already uses for a malformed pull-request number.

- **Three reads in the enforcement floor treated "could not read" as "nothing to report", and one of them had
  the correct sibling seven lines below it.** The census sweep skipped a tracked document it could not read, so a
  clean verdict could rest on a corpus it never examined — while its sibling reference gate refuses that outright
  ("a file this check claims to have inspected must have been read"). The subject-glob read swallowed a failure,
  which silently shrinks what a capability claims and lets the filing join report a change clean over the very
  capability whose subject it touched. And the payload delivery in front of release-tag signature verification
  discarded the write result, so a short or failed write reached the child's exit status as if the payload had
  arrived — the signing probe in the same file already checks its own write and explains why, an unnoticed empty
  payload making the tool sign nothing and "reporting the mechanism broken when only the harness was".

  The sweep's return type now carries the distinction rather than losing it, the glob read propagates a
  cannot-judge, and the delivery reports whether the payload actually arrived. Each is a different operator
  action, which is the whole reason this repository has one typed refusal.

  **Five module-level `#![allow(dead_code)]` are gone from the floor's own crate.** Measured: deleting all five
  and running `cargo clippy -p kanhe --all-targets -- -D warnings` produces nothing, so they suppressed no
  live warning. What they *could* suppress is an orphaned private helper inside the checks that judge this
  repository — the one place in a `-D warnings` workspace where enforcement code can rot unobserved.

- **The rule shape this project governs itself with most is finally one an adopter meets by running an
  example.** A crate-level dependency allowlist carries more of `AGENTS.self-law.md` than any other rule, and it
  appeared nowhere under `examples/` — so an adopter working through the dogfood met every other shape and not
  that one. `examples/guibiao-standalone` now declares one beside its module boundary.

  The landing site was not a free choice. That example's manifest already claimed its one-dependency footprint
  **is** the 圭表 pitch, "demonstrated here rather than asserted" — a claim living in a comment with no reaction,
  which is the open loop this whole project exists to close. Declaring it as an allowlist makes the pitch react:
  a second dependency added to that manifest now names itself and gates.

  **A boundary that holds needs different evidence from one that reacts**, and the example says so rather than
  leaving it implied. A passing allowlist is indistinguishable from one reading the wrong thing, so
  `tests/reaction.rs` points the same shape at the same real manifest with the dependency excluded and requires
  it to name that dependency and exit 1. The example now teaches both granularities 圭表 observes — a module
  boundary that reacts by design, and a crate boundary that holds for a demonstrated reason.

- **`### Removed` is gone, because neither thing it announced ever existed in a release.** One was
  `tianheng::observation_bounds()`, retired before its first published appearance; the other was Kanhe's
  refusal instrumentation, which ships in no package at all. Both were built and retired inside this same
  unreleased window, so the adopter delta for each is **zero** — and announcing the removal of something nobody
  ever received is worse than silence once the section is dated, because a dated section is a record and a
  record is never repaired. Verified before deleting rather than assumed: both retirements are settled calls
  (`PROJECT.md` states that the shell exposes no repository catalog; the instrumentation went when constructor
  locations were reclassified as implementation coverage), so neither returns, and nothing in the tree ever
  consumed the shell entrypoint. Their provenance stays where provenance lives — the pull requests, and
  `BACKLOG.md`'s closed records.

- **A closing sweep of this window found four wrong claims and three of them were written by the sweep's own
  changes, hours earlier.** Sorted the way *A repair loop is a diagnosis* asks: not one was a code defect. Every
  one was a sentence about the tree, and the class recurring **under an author actively watching for it** is the
  finding rather than the four repairs.

  What the mechanical passes caught, by class rather than by reading. An **absolute claim**: "no external crate
  had ever compiled against any of it" was false — `examples/observer-participant` is its own workspace and
  reaches part of the protocol through a source patch. What had never happened is narrower and is what the new
  check holds: the promise growing while the file that enumerates it stood still. A **claim about the tree**: a
  justification said the shell carries several sibling re-exports of the prelude's form, and it carries none, so
  the looser reader it argued against would have agreed exactly and could not have been caught by running it —
  the correctness now rests on entering the module rather than on that absence, which is the point the false
  count was reaching for. A **claim invalidated by its own change**: a fixture's comment said the contract never
  names `Run`, written before the same change made it name `Run`. And an **ordinal**: an entry called a bound its
  capability's fourth, which is true and held by nothing.

  The first repair of the count class then wrote a fresh one — "carries exactly one … today" — and the second
  pass caught that too. A figure anchored to *today* is a figure; the property it was reaching for is that
  entering the module removes the dependency on any absence, and stating the property is what ends the loop.

- **Three of this repository's own documents counted a set instead of pointing at what enumerates it.** The
  bound register's projection typed its residual tally three lines below the figure it *computes* — the one
  place a freshness check structurally cannot see, because it compares the generator's text with itself, so a
  further residual would have arrived silently. The projection register said how many generating mechanisms it
  recognizes rather than naming the one it does. And `repository-checks`'s Purpose said `git ls-files scripts/`
  names one unit while citing the very command that reports two, both of them wrappers; it now states the
  property that carries the weight — only wrappers there, no gate — since a gate returning is a real change of
  state while the arithmetic never was.

- **The bare-filename form was inert for every Rust file this repository has ever deleted, and the extension
  list was why.** `crates/kanhe/tests/reference_integrity.rs` recognized a bare filename only when it carried a
  governance extension, so a reference to a deleted Rust module never reached the branch that decides
  deleted-versus-never-tracked — the branch was there and correct, and nothing arrived at it. Admitting the Rust
  extension reported stale references this repository had carried unseen: an `[Unreleased]` entry describing a
  retired mechanism, a doc comment and a test comment both naming a module the 0.4.0 collector split replaced,
  and a comment justifying its own rule by citing two recognizers in a test file this window deleted. Only the
  first had been reported by anyone.

  **The discriminator that makes the extension safe already existed.** A bare name reacts only when this
  repository once tracked it outside a change directory and tracks it no longer; an illustrative fixture name —
  the kind this repository's prose invents freely to describe a shape rather than to name a file — was never
  tracked and cannot enter. Measured before admitting the extension, and both directions now carry a scenario
  and a direction that holds them, the positive one as the control so the silence proves something.

  One repair went further than the reaction asked. The doc comment among the four named its sibling by file
  path, which this gate checks for *existence* and nothing checks for holding what the sentence claims; it now
  names the function by an intra-doc link instead, so `rustdoc` resolves it and the Definition of Done's
  `-D warnings` documentation pass refuses a break — observed by breaking it, which reports `unresolved link`.
  A reference the compiler resolves is a strictly stronger instrument than one a text gate can only confirm
  points at some file.

  The `[Unreleased]` entry it found was **deleted rather than annotated**, which is the sharper half. The
  mechanism it described in the present tense was built and retired inside this window, so no release ever had
  it and the adopter delta is zero: `BACKLOG.md` keeps the closed entry with what the sweep measured at closure,
  and the method it established — that swapping a refusal's kind and replacing its message find *different*
  defects — belongs where lessons belong rather than in the adopter's document. Annotating it instead would have
  frozen a description of a mechanism nobody received into a section that becomes a record on the day it is
  dated, and a record is never repaired. The retirement notice went the same way for the same reason, one entry
  later in this window.

- **The two unpublished members carry a front page of their own.** `繩墨` and `勘合` are what a reader meets
  first when browsing `crates/`, and they are the two most likely to be misread as a seventh and eighth
  product crate — which is the confusion this window exists to end. Each now says what it is, what it is
  **not** (no 儀, no 司), and that `cargo publish` packages neither; the root README says the same where it
  already pointed at the self-law. Hand-written in the family's style rather than generated: no other crate's
  README is a projection, and inventing that mechanism for these two would replace one asymmetry with another.

- **The squash gate refused two legal shapes.** Any `!` in a subject was read as a breaking marker, so
  `fix(tianheng): preserve bang! in summaries` was required to carry a `BREAKING CHANGE:` footer; it is now
  read from the Conventional Commit head, which the same judgement already parses five lines above. And any
  all-bullet body was read as GitHub's commit list, so a self-contained `- Why: …` / `- Contract: …` body was
  refused for its formatting. The wrapper now supplies the pull request's own commit subjects and a body is a
  bare list when every bullet **is** one of them — the exact question rather than a tighter resemblance, since
  every commit here is conventional and a shape rule would refuse a hand-written `- fix: …` body while missing
  a branch carrying one non-conventional subject. The wrapper reads the live pull request's paginated commits,
  taking the first line of each full message because GitHub's `messageHeadline` truncates long subjects; local
  remote-tracking refs can lag the pull request or carry no fork head at all. Without a live set the workflow
  stops rather than substituting a local subset or falling back to the shape.

- **A failed publish remote read keeps its cause.** The publish-source gate previously defaulted a failed
  `git ls-remote` to an empty response, making an unavailable remote and a repository with no `main` ref emit
  the same cannot-judge. The two remain fail-loud, but now name different facts: the failed read includes Git's
  error, while a successful empty read names the absent `refs/heads/main`.

- **The Definition-of-Done reaction now observes cargo-deny's effective CI command.** It previously skipped
  `cargo deny check` because CI expresses it through `EmbarkStudios/cargo-deny-action`; removing or
  misconfiguring that action therefore stayed green. The reaction now projects the action's declared
  `with.command` into the same command set as `run:` lines, while deliberately interpreting no other action.

- **Reference integrity now reads the comment-bearing corpus it names.** Tracked TOML and `.gitignore`
  comments were outside the reader, and every Rust test source was skipped wholesale, so current pointers to
  deleted workflow scripts survived. Markdown document text and admitted line-comment regions now share one
  tracked corpus; fixture strings remain code rather than repository claims, and each inspected file is counted
  once. The capability also drops process exit and stdout promises a cargo-test check cannot provide, retaining
  read-only pass/fail behavior and fail-loud observation errors.

- **Four judgements answered about something other than what they read.** The publish gate asked
  `check-ignore` about git's **quoted** spelling of a path — measured, a file named `ignored-普通` ignored by
  a *tracked* `.gitignore` is listed as `"ignored-\346\231\256\351\200\232"`, that literal matches
  nothing, and the gate refused a file the repository itself ignores. Every path is now carried in git's `-z`
  form, so the bytes it asks about are the bytes it was given. The same function read that classifier's
  *failure* as an empty classification; it now refuses. The bound register's package enumeration read the
  working directory and dropped failed entries — which its own capability already forbade in writing — and now
  reads tracked manifests, refusing rather than shortening. And the corpus anchor invented `/` where its
  signature already carried an error channel; an invented anchor mislabels every observed file, because the
  anchor *is* baseline identity.

  Two things surfaced in the repair: git's pathspec `*` crosses directory separators unlike the shell's, so
  the first enumeration also matched fixture manifests nested inside a member; and a classifier that cannot
  run is not a state a fixture repository can be put into while `ls-files` still answers, so the judgement now
  takes its classifier as an argument and both directions — failed, and matched nothing — are constructed.

- **Both wrappers standing in front of an irreversible act could be disarmed by a rename, silently.** They
  reach their gate through `cargo test … -- --exact <name>`, and `libtest` exits `0` when that filter selects
  nothing — measured, and `0` again for an `#[ignore]`d test. So the exit status they read answers *did the
  selected tests pass* while the question they ask is *did the gate judge this act*. Each now requires the run
  to report exactly one passing test, and a repository check pins every `--exact <ident>` a tracked script cites to
  the `--test` target of the same invocation. Both are needed and measurement says so: `--list` includes an
  ignored test, so the check cannot see a silenced gate, while the wrapper's `1 passed` can.

- **The filing join accepted one claimant, and that could not catch the defect it was written from.** Where
  two capabilities' subjects overlap — `scripts/publish.sh` is claimed both by `publish-source-integrity` and,
  through `scripts/*.sh`, by `repository-checks` — naming either satisfied the join, so the original
  misfiling would have passed. The direction asserting otherwise was built on a **constructed** claim map
  rather than the declared subjects, which is why it was green. It now reads this repository's own
  declarations, and every claimant must be accounted for: named as modified, or named with the reason its
  requirements do not change.

- **Which capability a requirement belongs to is now checked where it is decided.** It was chosen once, in a
  proposal, and held by nothing — and it went wrong twice in one window, both times caught by a reader. Every
  capability declares a `## Subject` (the tracked-path globs it governs, resolved by `git ls-files` so
  membership is produced rather than modelled), and a repository check joins each active change's **diff against its
  base** to the capabilities its proposal names. Reading the touched set from the change's own prose would
  compare the list against something written by the same decision. Two bounds are declared with it: subjects
  do not tile the repository, and which governance member a check belongs to is unobserved.

- **The capability holding this repository's checks is named `repository-checks`.** Its earlier name
  claimed self-governance for a population that mostly collates a record and reaches no product contract, and
  a name is what a reader consults before filing a requirement. Its repository-only bound ids follow the
  capability identity; the product no longer exports this unpublished catalog, so released adopters take no
  action.

- **The governance apparatus shipped to every adopter, and that is why it was misnamed.** `cargo package
  --list -p tianheng` carried all 50 files under `tests/`: every check judging this repository's changelog,
  squash messages, specs and documents reached every adopter, where it could only detect no workspace and
  return. The capability governing those checks gives shipping in **zero** packages as its own criterion
  for governance rather than product, so by its own test they were product. Sharing a directory is also how a
  governance document came to state that every Rust integration test "runs Tianheng's own reactions against
  the workspace" — false for 20 of the 25 then present, none of which reached the shipped API at all.

  Two unpublished members now hold it, split by what each judges. **繩墨 (shengmo)** — the inked line —
  carries the law 天衡 declares over itself, as library code rather than a function at line 69 of a 668-line
  test file, and the reactions that run the delivered product against this workspace. **勘合 (kanhe)** — the
  split tally — carries the rest: one record made in two halves, proven genuine by fitting them together. The published
  `tianheng` crate goes from 25 test targets in its tarball to 8, and the 8 are the ones whose subject is the
  packaged code. Nothing an adopter could run has left, and no API, binary, or behaviour moves.

  No pre-existing dimension's own law changed: the projection's three pre-existing content lines that move
  touched are the preamble's own self-reference (which crate and test now generate it), and the rest of the
  diff is additive — two new crate sections declaring `shengmo`'s and this crate's own boundaries (named
  `jiaochou` at the commit that moved it, `kanhe` today), one each for the two members created to hold what
  moved. The refusal-site census is identical to the figure recorded before anything moved — 67 enumerated,
  60 defended, 7 declared out of reach, 0 undistinguished — which is what a move should
  produce.

- **Four documents restated a declaration the law owns, and the rule forbidding it was already written.** A
  comment block naming every member of a live dependency allowlist is refused, with a pointer to
  `AGENTS.self-law.md` as its repair — but the reaction read one crate's line comments against one dimension's
  allowlist. It now reads every declared allowlist against every tracked governance document, and found four
  restatements, including one inside the very requirement that calls a restatement there the second source of
  truth it forbids.

- **Fourteen reactions carried their own copy of the workspace probe, and eleven carried an identical copy of
  its absent-layout direction.** Diffing the copies against each other before merging them showed the
  duplication is in the scaffolding rather than the judgement: each differed in exactly one place, the
  artifact its own reaction needs present. The scaffolding now has one definition and the prerequisite stays
  with its caller; `TIANHENG_WORKSPACE_TESTS` goes from 53 occurrences to 26, and from fourteen definitions of
  what it means to one.

- **A written requirement's own scenario was refusing the proposals it excludes.** `reference-integrity`
  states that the gate inspects tracked content *outside* active `openspec/changes/` plans and carries a
  scenario for it; the reaction passed every tracked path as its corpus, so the exclusion existed only in
  prose. Nothing had held it because no plan had yet named a path that did not exist — and then one did, and
  the gate reported five offences against a proposal for describing its own deliverable.

- **A spec asserted the outcome of a command nothing ran.** `observation-bound-register` states that
  `openspec validate --specs --strict` passes over every spec. Measured, two did not, and `git grep` found
  that command in the scenario asserting its result and nowhere else — not in CI, not in the Definition of
  Done. **That gap is not yet closed**, corrected here rather than left for a later reader to re-measure:
  `git grep -n "openspec validate"` over the tracked tree at `HEAD` still returns the one hit, in the
  scenario's own prose, and nothing in `ci.yml`, `AGENTS.md`'s Definition of Done, or any script invokes
  it. The spec's claim about the command's outcome is still asserted without ever running it; filed in
  `BACKLOG.md` rather than restated here as done.

- **Three judgements could report clean over something they did not read.** All three were reported by review,
  verified against the code, and one reproduced against a running git before anything was changed.

  **The publish gate's cleanliness answer depended on the machine.** The gate defined a hermetic command
  builder and used it for its *fixtures*; the judgement ran through a bare `git`. A `core.excludesFile` outside
  the repository made the cleanliness read return empty for an untracked file. The obvious repair is not the
  repair: routing through the existing builder closes one of three ambient sources and leaves the
  `$XDG_CONFIG_HOME/git/ignore` default — which no configuration names — still hiding files, measured. So the
  neutralisation is explicit, and what no configuration can neutralise is classified by **source** instead:
  the excluded set is the difference between an unexcluded listing and an excluded one, and each path's source
  file is readable. A source counts as repository content only if it is **tracked**, because an untracked
  `.gitignore` reports a repository-looking source while being no more part of the repository than the clone's
  own exclude file. Classification rather than a blanket refusal, because refusing whenever a clone carries an
  exclude file trades a false clean for a false alarm on the gate standing before an irreversible act.

  `clean` is now defined in the specification before anything relies on it: ignored by tracked repository
  content is clean, because `cargo publish` applies the same exclusion; hidden by this clone or this machine is
  not, because the same commit must not be judged differently in different places. That amendment had to come
  first — the requirement read *any untracked file*, and an ignored file is untracked, so the control this
  change needs would otherwise have asserted the opposite of the live rule.

  **Two enumerations dropped I/O failures.** A failed directory entry was discarded, and an example manifest
  that existed but could not be read was skipped *identically to one that is absent* — so the remaining
  readable examples satisfied the counters the judgement then reasons from. Absent is now distinguished from
  unreadable, one read site serves both enumerations so neither can shadow the other, and the entry failure a
  fixture cannot schedule is declared out of reach with a slug of its own.

  **A bound carrying two `UNPINNED` citations kept the last.** Several pinning tests are several defences of
  one bound; several trackers are several owners of one gap, and the declaration holds one — so the register
  recorded whichever line came last. Repeated `UNPINNED` is now an invalid citation state naming the bound,
  with a control keeping repeated `PINNED-BY` accepted, since flattening that asymmetry would break a live
  declaration.

  One repair refused one of this repository's own fixtures — it made a worktree read clean with an untracked
  `.gitignore`, the exact hole being closed — and then made that direction unconstructible: once `clean` is
  defined by the repository, a clean worktree with an unresolvable HEAD cannot exist. Measured across every
  route, so the read joined the shared site rather than keeping a refusal no input can take.

- **The squash-message rule gains a reaction, and a wrapper to reach it through.** `AGENTS.md` said the squash
  subject is the pull request's title with **no auto-appended `(#N)`**, and nothing held it: nine subjects in
  this repository's history carry that serial, the most recent on the commit that landed a reaction for a
  requirement enforced by nothing.

  The rule cannot be held where rules are usually held here, and that was measured rather than assumed. A
  squash merge runs on GitHub's servers, so no local commit exists and no `commit-msg` hook runs. Both values
  of `squash_merge_commit_title` append the serial, so the repository setting cannot suppress it. The serial
  is GitHub's rather than the title's, so a check on the title changes nothing. What remains is one string
  passed at merge time — at the moment a record lands and stops being repairable, since amending a merged
  squash changes the hash its pull request cites.

  So `scripts/merge-pr.sh` stands in front of `gh pr merge` the way `scripts/publish.sh` stands in front of
  `cargo publish`, and carries no verdict of its own: the judgement is a Rust reaction with a failure matrix,
  returning the shared kinded refusal so its own construction sites are swept like every other. It holds the
  subject to being the title, to carrying no serial, and to Conventional Commit shape; a breaking subject to
  naming its migration; and the body to being non-empty, free of agent attribution, and not the concatenated
  commit list the rule exists to replace. A merge made in a browser reaches no wrapper, and that is declared
  as a bound rather than implied.

- **A census is now DECLARED, and the half nothing can hold says so.** The rule had one reaction covering
  exactly one sentence, and adversarial review found **eight** figures wrong in a single change — an entry
  population corrected twice, each correction breaking the next; a block header counting four where there were
  seven; a ratio that drifted inside its own pull request; and a file count the commit typing it had just made
  stale. The reaction that enumerates a set now names the one sentence its figures are written in and produces
  them, and one sweep holds every tracked document to every declaration. Adding a census means declaring it,
  which is what makes it enumerable.

  Two of the first four declarations were **inert against the very documents they are for**, and both were
  found by running them rather than reading them. The matcher read digits, and this repository's prose writes
  counts as words — `twenty entries`, `eleven of them` — so it matched nothing. And one phrase spanned lines
  against a line-oriented sweep: declared, enumerable and silent. Both are now guarded on the declaration
  itself, along with a phrase too generic to name its own set, which had matched an unrelated sentence in a
  specification.

  Two shapes stay outside by construction and are stated rather than approximated. A figure about a **past
  state** is a record — holding it to today's enumeration would demand the record change every time the tree
  does — and a figure in a phrasing no census declares is unheld, because reaching it needs the prose detector
  measured three times and rejected. `AGENTS.md` carries that half as a rule with no reaction: **a count of
  something this repository does not produce is not written.** Each of the three wrong figures it would have
  caught was decoration — the sentence said the same thing without the number, right up until the number
  stopped being true.

- **A census is produced, never typed, and `AGENTS.md` now says so.** Hand-written figures drifted in every kind of
  place they can live during this window — a code doc, two backlog entries, a changelog sentence, three files at
  once, the version-horizon paragraph that assigns the release number, and a **generated projection's template**.
  That last one is why the rule names its instrument rather than only its prohibition:
  the retired gate-shape projection is generated and staleness-checked and its bound disclosure still went wrong,
  because the figure and the list were string literals in the generator — the freshness check compares the
  generator's own text with itself, so a bound declared in this same window went unlisted. The projection now
  **derives** that disclosure from the specification and is held to it in both directions, with the figure as the
  list's length; a single array carries only the explanations. The rule also records why a detector over prose was
  measured three times and rejected: widening the phrasing false-positives on both projections' headers, on the
  register's own diagnostic, and on its matrix's expected-output literals; widening the corpus to `scripts/`
  false-positives on the fixture censuses that matrix writes deliberately; and the one instance that occurred in a
  code doc was spelled in words. Most numbers here describe a *shape*, not a census.
- **The class that produced three withdrawn attempts is recorded, and it grew when the paragraph landed.**
  `PROJECT.md` restates architectural facts the generated projection already carries, and states others —
  locations, counts — that nothing carries. `crates/kanhe/tests/reference_integrity.rs` holds that a cited path exists and
  is tracked; nothing holds that the thing described lives there, which is exactly the half a withdrawn
  attempt got wrong. Closing the crate question added **five** more location claims to that file, so the
  entry is filed larger than the branch it came from filed it, not smaller. Its trigger is a claim found
  false *after* filing — the three found before are the control and cannot be evidence for themselves.

- **The crate question about the three governance surfaces is closed, and `PROJECT.md` says why.** It had been
  left open by one sentence — *crate-or-convention as their nature dictates* — which answered nothing and was
  consulted three times; all three answers were withdrawn, each restating a law that was then only half reacted
  to. Making `restrict_dependencies_to` naming no sibling an asserted fact rather than a quoted one removed the
  blocker, so the argument now stands on something: a crate is the boundary the self-law reacts to, and a
  governance surface has no boundary to be — each crosses every crate it touches and one lives outside
  `crates/` altogether. That says nothing about `xuanji` or `tianheng`, which are crates and are not
  instruments.

  The product decision had already been made by measurement rather than by argument: none of 垂象, 實錄 or 校讎
  is a crate name or a `description`, and none appears in any published crate's public surface — zero
  occurrences across the six publishable crates' `///` and `//!` documentation, and one `//` comment inside
  `tianheng`'s private `render` module. Where they do appear — that comment, `kanhe`'s manifest comments,
  `shengmo`'s source, and this repository's own governance documents and this entry — it is commentary about
  this repository's vocabulary rather than a name an adopter uses, which is why the naming was never a product
  question. Each is now stated with where it is,
  which is what the earlier attempts got wrong — one location table was false three times out of three, and one
  retirement rested on a grep that never swept `.github/`, where `CODEOWNERS`' first line reads *"The amendment
  reaction"*. The set name 三司 goes with them; 司 keeps a definition in place, because the 潛移 paragraph's
  contrast against it still needs an antecedent.
- **This heading exists, and a reaction refuses the leak back into the others.** `scripts/` and `docs/` ship
  in zero packages, and before this change twenty entries named that machinery under `### Added`, `### Changed`, `### Fixed` and
  `### Documentation`, because the document offered no heading that was not an adopter's vocabulary. What is
  separated is **publication, not rigour**: no reaction, twin, observation bound or exit-contract obligation
  was removed, and the test is that deleting an adopter-facing entry about a gate changes nothing about that
  gate. An entry under an adopter-facing heading of `[Unreleased]` may not name a path under `scripts/`, a
  bare basename `git ls-files scripts/` resolves, or a directory derived from that enumeration. Adopter-facing
  is the **complement** of this one heading, so a heading nobody anticipated reacts rather than being exempt.
  Dated sections are record and stay untouched.

  Two rules were falsified before they were kept. *Cite no path that ships in no package* died on enumerating
  the real citation population: every path `[Unreleased]` cited ships in no package, `COOKBOOK.md` and
  `docs/*.md` among them, and those are adopter surface anyway. Comparing whole backticked spans died on
  three shapes this document already uses — a span carrying a command, a padded double-backtick span, and an
  inline span wrapped across a source line. A name is a **word**: a maximal run of path characters required to
  equal a tracked name. Seven limits are declared, six pinned.

- **The repository checks and self-governance dogfood gates are Rust tests, and the repair of that move is the larger half of this
  entry.** `scripts/` held seven `check_*.sh` gates, eleven `test_*.sh` twins and seven libraries — 6,827 lines,
  4,000 of them once blanks and comments are dropped, 56% of those twin matrices that existed only because a gate was a shell script. In Rust a
  test's failure mode is asserted inline, so the twin obligation buys nothing and the pairing retires with it;
  `gate-shape-contract`, the capability specifying that pairing, is retired for the same reason, its own
  projection having reached `0 gates, 11 properties each` and reported clean over it.

  **The move landed with six of the checks inert, and each was measured rather than reasoned about.** The
  bound register collected citations into a list and asserted only that the list was non-empty — it never read
  `PINNED-BY` once, while its doc comment claimed it did; its projection check read the document and compared
  it to itself, an assertion that cannot fail and that under `BLESS` wrote the file back to itself. Pin-bites
  counted tab-separated lines. Release coherence kept only its bound pins, which assert *silence* and passed
  trivially against a check that no longer existed. Reference integrity read markdown links only, and read
  the worktree rather than the index. The examples suite ran `cargo check` and discarded the result. The
  publish-source gate asserted that `Cargo.toml` exists and that `git status` runs — in front of an
  irreversible act — while `scripts/publish.sh` still called a file the move had deleted, exiting **127**.

  The commit that made the move also carried `BREAKING CHANGE:` in its body, for a directory that ships in
  **zero** packages. No adopter ever had those entry points, so nothing an adopter can observe changed. That
  marker is wrong and is recorded here rather than rewritten out of the branch's history; this release is not
  breaking on its account.

  All six are restored, each holding what its capability specifies and each refusal seen to fail. Test-ness is
  decided by the harness rather than the source text, the mutation suite checks out a detached worktree with
  hooks disabled and re-runs its control after the restore, the publish judgement separates a violation from a
  cannot-judge in its **type**, and the two reactions too costly for every suite are named on their own
  lines in the Definition of Done and in CI — a third, the publish gate, is asked for by `scripts/publish.sh`
  at the one moment a release snapshot exists to judge — a reaction that runs only when someone remembers is worse than
  one that costs.

- **The publish wrapper refuses `--manifest-path`** (either spelling) before the source gate runs. The gate
  judges this repository; that argument moves cargo's workspace root to another tree, so the two would
  disagree about what is being published, one argument away from an act that cannot be corrected. The
  registry-side arguments (`--registry`, `--index`, `--token`) change the publish's destination rather than
  its source and stay forwarded.

- **The one merge-gate input whose absence means "no merge is being made" was the one still reading absence
  and unreadability as the same fact.** `scripts/merge-pr.sh` takes the squash subject from `argv` — where
  arbitrary bytes are expressible — and hands it to the gate as environment. Read with `env::var`, *not set*
  and *set but not UTF-8* are one `Err`, and the arm answering it returns **clean**: the gate printed "not
  judged", the run exited `0`, the wrapper's `require_one_pass` saw `1 passed`, and `exec gh pr merge`
  recorded a subject no judgement had read. Every other stop in that wrapper fails closed; this one failed
  open, in front of a record that cannot be amended, because a merged squash's hash is what the pull
  request's merge record cites.

  The repair that closed this collapse for the other three inputs landed one commit earlier and did not
  reach the fourth. Two spellings of one rule is how: the three went through `var_os` and a match, the
  subject kept `var`. So the rule now has **one implementation** — `kanhe::supplied` answers *absent*, *the
  value*, and *set to bytes this gate cannot read* as three states, and all four inputs go through it. A
  fifth input cannot be added under a different rule, because there is no second rule to add it under.

  Measured rather than argued, and the control is what makes it a measurement: the direction re-executes the
  test binary with all four inputs supplied and asserts the class on the verdict channel. With a readable
  subject the child reaches a clean verdict; with `\xff\xfe` in the subject it exited `0` and wrote nothing
  before this change, and refuses as a cannot-judge after it. The two runs differ only in the subject's
  bytes.

  No published API, outcome, report, exit class, or manifest moves — `kanhe` ships in no package — so
  released adopters take no action.

- **Three twins, and the reaction built to find them was deleted in the same change.** Two extractions in
  `crates/kanhe` each closed a duplicated helper and each left a sibling behind, and both say so in their own
  headers — `hermetic_git`'s names a command builder that "lived twice, byte-identical, in
  `publish_source_gate` and `release_coherence_gate`", `manifest`'s names "two more twins left behind in that
  extraction". The same pair of files, twice, because the corpus of an extraction was the function someone had
  noticed rather than what those two modules share.

  Converged here: the fixture-side `run()`, which lived in both gates differing only in whether the program
  was its own argument or `args[0]` — the second spelling also panicking on an empty slice where the first
  could not, so the twin had begun to diverge. Then, exposed by that convergence, an `add`-then-`commit`
  helper `release_coherence_gate` had written for itself and `publish_source_gate` had not. And the three
  sentences both gates tell an operator about a workspace version that is absent, unreadable, or not a
  semantic version.

  **The version arms themselves stay twinned, and by a constraint rather than an oversight.**
  `refusal_register` registers a site by the string literal that *opens* the constructor's argument list, so a
  site id arriving as a variable is a construction it cannot parse — and it holds that count at zero.
  Collapsing the two arms would instead fold each gate's own identity into a shared one, which is what the
  register exists to prevent. So what moved is what was genuinely duplicable: the sentences have one owner and
  each gate keeps its own literal identity.

  **A reaction was built for the class and did not earn its place**, and that is recorded because the
  measurement is the useful part. Over `crates/{kanhe,shengmo}/src`, a window of four executed lines carrying
  at least two executed statements reports the two structural twins and nothing else — the corpus rule that
  makes it usable being *executed statements, not item declarations*, without which the same window also
  reports `#[cfg(test)] mod tests { use super::*;` across four modules. Two other instruments were measured
  and rejected: normalized function-body similarity puts the live twin at `0.62` against a noise floor of
  `0.56`, and restricting to same-named functions widens that margin to `0.20` while going blind to the
  `WorkspaceVersion` pair, a 25-line block inside two 200-line functions that dilutes to `0.30`.

  It was deleted anyway. It sees neither token-level instance the same review found; it reports a call's
  arguments as statements wherever rustfmt wrapped one, and every tightening measured against that false
  positive removed a true one; and the convergence it asks for is the one `refusal_register` forbids. An
  instrument covering one instance of four, needing three declared bounds, and demanding a repair another gate
  refuses is not the available option — so `AGENTS.md` gains *An extraction's corpus is the pair of modules,
  not the function you came for*, which covers all four shapes and states outright that it has no reaction.

  No published API, outcome, report, exit class, or manifest moves; `kanhe` ships in no package.

- **A gate that ran, passed, and judged nothing was indistinguishable from one that agreed.** Both wrappers
  stood in front of an irreversible act and asked `require_one_pass` whether the gate had spoken — a question
  it cannot answer, because it reads *did the selected test pass*, and a harness that returns without judging
  satisfies that. One did: a subject supplied as bytes the gate could not read printed "not judged" and
  returned, so `1 passed` was true and nothing had been judged. The channel was written only from the refusal
  arm, so the success path had no positive evidence at all.

  The verdict is now a **value**. `kanhe::verdict_channel::Verdict` has three arms — the act is not being made,
  a verdict was reached and holds, a verdict was reached and refuses — and `deliver` is the single exit of both
  gate harnesses. A clean verdict writes `Clean` to the channel, so *absent on success* means unjudged by
  construction, and each wrapper reads that rather than inferring it from an exit status. The two guards catch
  different states and both stay: `require_one_pass` sees a renamed test, where nothing ran; the new one sees a
  test that ran, passed, and reached no verdict.

  **The direction that used to guard this could only reach one exit.** It located the harness's
  `Err(refusal) => {` arm by substring and asserted the report preceded the panic *within it*, so every other
  exit owed nothing — which is how the unreadable subject left through a clean `return`. The pairing is now a
  property of the type, held over the whole enum: *every verdict that fails the run reached the channel first*.
  What is left to a text reading is the half a type cannot carry, that each gate delegates rather than deciding
  for itself, and it also refuses a harness writing the channel directly.

  Recorded because it constrained the repair: the `Verdict` failure matrix lives in `crates/kanhe/tests/`
  rather than beside the other matrices in `src/tests/`, because a `Refusal` can only be built by a constructor
  and the refusal register reads every construction under `crates/kanhe/src` as a site that must carry an
  identity. A fixture's construction is indistinguishable from a branch nobody observes, and the register is
  right to refuse it — so the matrix takes its refusals from judgements that already register their own sites.

  No published API, outcome, report, exit class, or manifest moves; `kanhe` ships in no package and
  `scripts/` reaches no tarball.

- **Two constants existed so that one token would have one owner, and both were written out again as
  literals.** `kanhe::region::DO_NOT_EDIT` is the marker a generated document declares itself with, and
  `shengmo::workspace::MARKER` names the variable saying a run must find a repository. Both were written out
  again as literals in every module that could reach them — the producers and the recognizer of the first on
  opposite sides of it, which is the shape
  `verdict_channel` spent a window closing between a shell script and Rust and which had stayed open between
  Rust and Rust.

  **The marker's second spellings included every projection's regeneration command**, and that half fails
  quietly rather than loudly: a renamed variable leaves each generated document telling a reader a command
  that runs the gate with the marker unset, so the gate skips and the reader takes the skip for freshness.
  The document-marker half fails loudly instead — the recognizer stops recognizing and the freshness gate
  refuses — which is why it was the smaller risk of the two despite being the more visible duplication.

  Both are converged within the reach of their constant, held by a check whose corpus is exactly that reach.
  What lies outside it is named where a reader meets it rather than declared as a bound: `shengmo`'s own law
  projection cannot reach `kanhe`, and `MARKER` spellings live in `tianheng`, `louke` and `xuanji`, which are
  published crates that cannot depend on `shengmo` without closing a cycle. A rename still has to reach every
  one of them, and the ones outside the constant's reach are the ones no check can see — a fact about the
  dependency graph rather than a site anyone declined to fix.

  The projections are byte-identical after the change, which is what makes it a convergence rather than an
  edit: the interpolated constants render exactly what the literals did.

  No published API, outcome, report, exit class, or manifest moves; neither crate ships in a package.

- **One governance vocabulary named two structures, with two of its three terms inverted between them.**
  `AGENTS.md` opened *Three-Layer Architecture* with the reaction backstop at Layer 1; `COOKBOOK.md` opened a
  paragraph under the same name with the reaction backstop at Layer 3; and `self-law-projection`'s own `SHALL`
  defines *Three-Layer Agent Law* as preamble, generated body, Rust law source — agreeing with `COOKBOOK.md`
  and not with `AGENTS.md`. Two documents, one name, `Layer 1` and `Layer 3` swapped.

  It surfaced from an ordering question rather than from a review. Two external reports argued that historical
  prose should leave doc comments **by citing Layer 3**, and the ruling against them cited Layer 3 too — three
  parties reasoning from a term with two live referents, and five review rounds over this window had not named
  it. The ordinals now have one owner: the `SHALL` keeps them, `COOKBOOK.md`'s paragraph is renamed to the
  structure it was already teaching, and `AGENTS.md`'s three carriers are **named** rather than numbered,
  because they were never an ordering.

- **`AGENTS.md` gains *What earns a place in a doc comment*.** The test is whether a passage carries an
  **observation source or a falsifier** for a claim the item makes — not whether its verbs are past tense.
  `Measured: bash -c 'printf a;#b' prints a` is the observation source for a declared over-inclusion, and
  deleting it leaves an assertion nothing can falsify; `fixed in round 6` names when rather than what. The
  rule tabulates the shapes a review actually files, so the same categories do not have to be adjudicated
  again.

  **No reaction, and the measurement behind that decision is recorded rather than left to be re-taken.** The
  decidable subset is small and collides with itself — `crates/` carries 5 `PR #N` and 7 bare `#NNN`, two of
  the latter being the **fixture** for the squash-serial check, so a detector for the shape would refuse the
  check that forbids it. The rest is not decidable: 20 `round N` and 16 `this window`, each needing the
  criterion applied per site, which is the judgement over prose this repository has designed, measured three
  times and rejected. One row of the table *is* enumerable — a relative anchor names a moving reference and is
  stale the moment its window closes — and that sweep follows separately.

  No published API, outcome, report, exit class, or manifest moves.

- **Correction: the verification recorded for the constant-owner change did not happen the way it is
  written.** The squash commit `f62a7b9` and its pull request say the check "was written first and run against
  the tree before any convergence", that it "named all seven `DO_NOT_EDIT` sites", and that "the count went
  from nine to fifteen and then to zero". None of the three holds. The check was written **after** the
  `DO_NOT_EDIT` convergence and its first run named five sites, all `MARKER`. Run against `e0d8500` its own
  criterion names **six** `DO_NOT_EDIT` sites — the seventh is `crates/shengmo/src/law.rs`, which that check's
  corpus **excludes by design** — and **seven** `MARKER` sites, thirteen in total. No run produced nine or
  fifteen.

  The correction lives here because a merged squash is a record: amending it changes its hash, and the pull
  request's merge record cites that hash. What the entry says about the change itself stands — the
  convergences landed and the projections are byte-identical.

  **The cause is worth more than the correction.** Every other negative run in that window was pasted verbatim
  from the terminal and every one of them is accurate. This one was composed from the plan — *the check goes
  first, it will name the twins* — and it is the only one that is false. `AGENTS.md` gains the rule in the
  same window: a negative run's record is pasted from the run's output, never written from the intention.

  Two figures typed in the same window are corrected with it: `seven owners and nine` mixed two denominators
  (one counting literal spellings, the other counting those plus the constant), and `Six of the MARKER
  spellings` was seven when it was typed. Both are removed rather than repaired — nothing enumerates those
  sets, so a figure there is a census with no producer.

- **The check that holds one token to one owner had a hand-written corpus and exempted a file where it meant
  a declaration.** Both are the shape it exists to refuse, in itself. Its corpus was a typed list of
  directories while its own header argued from the dependency graph, so a member added to the graph — or
  `shengmo` gaining a `kanhe` edge — would have put new sites outside the corpus while the check reported
  clean. And skipping the owner's whole **file** exempted more than the owner **declaration**: a second
  constant carrying the same value beside the first read as clean.

  The corpus is now derived from the manifests and the declared list is held against it **in both
  directions**, which `repository-checks` already required of any constant a check judges by — a requirement
  whose own text records that this class had come through three doors before. This was the fourth. Exemption
  is now by declaration: the value must appear exactly once in the whole reachable corpus, and that once must
  be the declaration.

  The check also gains the requirement it shipped without. `repository-checks` now states the unique-spelling
  property, the derived corpus, exemption-by-declaration, the dependency-based residue, and the refusal on an
  unreadable enumeration — with a scenario each.

  Both new assertions were run against the code without them: adding `shengmo` to the corpus of a constant
  `shengmo` cannot reach fails with `left: {"kanhe", "shengmo"} right: {"kanhe"}`, and a second constant of
  the same value in `region.rs` fails naming both lines, `2` against `1`.

  No published API, outcome, report, exit class, or manifest moves.

- **Every reference to *this window* named a window that closes.** Sixteen doc comments across ten modules,
  plus one inside a generated projection, said *this window* — a moving reference, stale the moment the
  release cuts and meaningless to a reader who arrives after it. They now name the window: `the 0.5.0 window`,
  which is an immutable historical fact and the one form the versioning rule admits in long-lived prose.

  This is the single enumerable row of the doc-comment criterion added in the same window. The rest of that
  criterion is applied by a reader; a relative anchor is not, because what makes it wrong is structural
  rather than a matter of judgement.

- **`BACKLOG.md`'s promotion-trigger annotations carry the moment they were evaluated.** They were written as
  present-tense assertions — a bare *not fired* — so a reader could not tell one evaluated today from one
  written when the entry was filed and unread since. The entry that records *a promotion trigger is evaluated
  by whoever next reads it* declares its own escalation on **a second trigger found to have fired unnoticed**,
  and *unnoticed* had no observable meaning while the annotations carried no time: the condition could arrive
  and nothing could show it had. Every annotation now reads `**Not fired** (evaluated <date>)`, so the next
  sweep compares dates rather than re-reading every entry, and the entry's own trigger becomes decidable.

  No published API, outcome, report, exit class, or manifest moves.

- **Two self-law boundaries carried reasons a dependency rule cannot observe, and neither had been seen to
  bite its own target.** `restrict_dependencies_to` observes one thing: which packages a crate declares. The
  reasons on the `shengmo` and `kanhe` boundaries also asserted that 繩墨 *is an adopter rather than a member
  of the family it governs*, that it *exercises exactly the surface an adopter has*, that 勘合 *governs no
  product contract*, and that keeping the two apart *stops a claim about the law being read as a claim about
  document hygiene*. Those are roles, uses and reader cognition — prose prescription with no backstop, which
  the reason rule forbids outright.

  Both reasons now state only their perimeter: the allowlist, and the dimension edges it forbids. The roles
  move to `PROJECT.md`'s Decisions, which is where the reason rule sends them.

  **And each boundary is now seen to bite its own target.** The self-governance suite proved the machinery;
  it had never shown these two formations refusing anything. Four perturbations, run and recorded: a
  forbidden dimension edge on `shengmo` refuses with `target: "shengmo", finding: "guibiao"`, the same on
  `kanhe` with `target: "kanhe", finding: "guibiao"`, and removing a legal member from each allowlist refuses
  the **existing** edge — `finding: "serde_json"` for 繩墨, `finding: "shengmo"` for 勘合 — which is what
  shows the rule reads each target's real edges rather than passing over them.

  **Correction to how those four were classified.** The pull request called the last two "the precision half".
  They are not: a precision witness shows the rule declining to fire on something legal and adjacent, and both
  of those fire — under a *perturbed law*, which makes them a second pair of violating witnesses observed from
  the other side. They establish that the rule reads each target's real edges, which is worth having and is
  not what the word means. The precision witness is the clean run with the law unchanged, where 繩墨's real
  `tianheng` and `serde_json` edges and 勘合's real `shengmo`, `tianheng` and `serde_json` edges all pass at
  exit `0`. It exists; it was mislabelled, not missing. A witness is classified by what it establishes, never
  by the intent it was run with.

  **What remains for a human**: these two boundaries were added under a commit stating "the law itself did not
  change". They are formations, not a relocation, and formation carries its own acceptance. The evidence
  above is what that acceptance would rest on; the acceptance itself is not something a repair can supply.

  **The evidence is replayable, and here it is in full.** A review read the record above and found it
  summarised rather than reproducible: no exact command, no process status per run, no source delta, and a
  partial reaction identity. All four perturbations were re-run for that reason. The command is the same one
  every time — `TIANHENG_WORKSPACE_TESTS=1 cargo test -p shengmo --test self_governance
  tianheng_governs_itself` — and every perturbed run exits **101**, the clean run **0**.

  | delta applied | `target` | `finding` | `rule_key.allowed` |
  |---|---|---|---|
  | 繩墨's allowlist loses `serde_json` | `shengmo` | `serde_json` | `["tianheng"]` |
  | 勘合's allowlist loses `shengmo` | `kanhe` | `shengmo` | `["serde_json","tianheng"]` |
  | `guibiao = { path = "../guibiao" }` added to 繩墨's manifest | `shengmo` | `guibiao` | `["serde_json","tianheng"]` |
  | the same line added to 勘合's manifest | `kanhe` | `guibiao` | `["serde_json","shengmo","tianheng"]` |

  Every one carries the same identity shape, quoted from the run: `rule: "restrict dependencies to"`,
  `rule_type: "tianheng.rule/guibiao/restrict-dependencies-to"`,
  `fact_type: "tianheng.fact/guibiao/dependency"`, `shape: "dependency-edge"` with
  `fields: {"kind": "normal", "package": <the finding>}`, `severity: Enforce`, `baselined: false`,
  `polarity: Some(AllowlistGap)`, and the boundary's own `reason` verbatim. The `reason` field is what makes
  these witnesses about **these two formations** rather than about the rule in general: it is the sentence
  each boundary declares, carried into the violation the boundary produced.

  The two kinds are not the same witness. Removing a legal member shows the rule reading a **real, existing**
  edge of that target; adding a forbidden edge shows it reading a **new** one. Both fire under a perturbed
  subject, so both are violating witnesses; the precision witness remains the clean run at exit `0`, where
  繩墨's real `tianheng` and `serde_json` edges and 勘合's real `shengmo`, `tianheng` and `serde_json` edges
  all pass with the law unchanged.

  The tree is byte-identical after the replay, `Cargo.lock` included.

  **Accepted, and recorded in `PROJECT.md`'s Decisions.** 繩墨 may depend on 天衡 and serde_json only; 勘合 on
  繩墨, 天衡 and serde_json only; both at `enforce`; both crates ship in no package, so no adopter is reached
  either way. The cost accepted with them is the ordinary one — a later edge from either to a dimension is an
  amendment rather than a commit.

  **Correction to `09c29d0`'s own body.** It reads *the law itself did not change: the regenerated projection
  differs by exactly three lines, all of them the preamble's own self-reference*. Three of the projection's
  changed lines are the preamble's; the projection changed by nineteen, and fourteen of them are these two
  boundary entries with their own targets, rules and severities. The correction lives here because a merged
  squash is a record: amending it changes the hash its pull request's merge record cites.

  No published API, outcome, report, exit class, or manifest moves.

- **A repair reintroduced the class it repaired, one call further along.** The fail-loud directory enumerator
  that replaced two `filter_map(|entry| entry.ok())` returned `PathBuf`, which turns the infallible
  `DirEntry::file_name` into `Path::file_name`'s `Option` — and both callers then defaulted it, encoding an
  absent name as an empty member. That is the encoding the enumerator exists to remove. It returns the
  entries now, so the infallible accessor survives the call.

  Latent rather than live: `read_dir` gives every entry a name. Recorded because the shape is what matters —
  a repair that moves an encoding rather than removing it reads as a repair and is not one.

- **Correction: `3c679af` claims no `Three-Layer Architecture` remains in the tree, and one does.** It sits in
  that same commit's own `CHANGELOG` entry, which cannot describe the retired vocabulary without naming it.
  The sentence was true when the `grep` ran and false by the time the commit landed. The tree is right; the
  claim was the wrong shape — a completeness claim over a sweep must name the corpus it swept and exclude the
  record describing it, and `AGENTS.md` now says so beside the rule about composed verification.

  No published API, outcome, report, exit class, or manifest moves.

- **A verdict the gate reached and could not write left the same absent file a verdict never reached leaves.**
  `deliver` called the channel writer and discarded its outcome, so *absent means unjudged* — which this
  module and `repository-checks` both claim **by construction** — had two causes and observed only one. A
  `Refused` whose write failed arrived at the wrapper as exit `2`, the class for what could not be judged,
  where the gate had found exit `1`. That is the class collapse the channel exists to end, reached through a
  different door.

  It is a guarantee that was **removed rather than missing**. The predecessor this routing replaced returned
  the write's outcome and said why in its own doc — *"Returning the outcome lets a direction assert that,
  rather than inferring it from a file that is missing for either reason"* — and folding every gate into one
  exit dropped the return. The writer now answers three states like everything else in this crate: no channel
  opened, written, or opened and failed. The third fails loudly, naming the path and the error.

  Held by a direction that re-executes the gate with the channel under a directory that does not exist.
  Permissions were the obvious fixture and are the wrong one: `publish-source-integrity#signature-unwritable`
  is declared unheld precisely because a permission-based fixture answers differently for root, which makes
  the direction's own result depend on who runs it. A missing parent fails the same way for everyone.

- **An in-band sentinel occupied the same type as a real exclusion source.** `"<unshown>"` stood for *the
  classifier named no source*, was tested back by string comparison twice, and reached an operator diagnostic
  as `hidden by <unshown>, which this repository does not track` — reading as though a file of that name were
  the ignore source. A gitignore literally named `<unshown>` is a legal filename, so the placeholder was also
  a shape the judged tree could forge. It is an `Option` now, and the diagnostic states the fact instead of
  rendering a stand-in as a filename.

  No published API, outcome, report, exit class, or manifest moves.

- **The test that decides whether a reason is inside its perimeter lived in one boundary's own annotation, and
  had never been applied to the other seven — including the boundary hosting it.** `AGENTS.md` states the rule;
  what makes it operable is a falsifier, and 漏刻's annotation had written it: *delete the thing the clause
  asserts, and if the boundary stays green while the clause turns false, the clause is outside*. Applied to
  `restrict_dependencies_to`, which sees the declared dependency set and nothing else, *what a dependency is
  used for* always fails and *what the allowlist entails* always passes.

  The falsifier moves to `constitution()`'s header, where every `because(...)` is written, and all eight
  boundaries were run against it. Three carried use clauses: 繩墨's *serde_json reads cargo's own message
  stream*, 星表's *reading cargo metadata*, and — the sharpest — 漏刻's own *it reacts in prod independently
  of the 天衡 shell*, in the same boundary whose annotation states the test. Stop reacting in prod, keep the
  edges, and each boundary is green while its sentence is false.

  The other five pass, and 天衡's is the model the annotation already named: *direct normal edges end at
  observation dimensions and projection serialization* names exactly what its own rule sees. 渾儀's
  *quarantined syn* is left as written and noted here: the quarantine is entailed by the **sibling**
  allowlists rather than by this boundary's own, so it is inside the law's perimeter without being inside
  this rule's — a distinction worth stating rather than resolving by edit.

  The reasons project into `AGENTS.self-law.md`, so the projection moves with them. No target, rule or
  allowlist changes, so no recorded baseline is affected.

- **A dependency edge was recognised by one string layout.** The check holding one token to one owner derives
  its corpus from the manifests, and read an edge as `text.contains("path = \"../{dep}\"")` — one spelling of
  one form. `path="../x"` declares the same edge with no spaces; `alias = { package = "x", … }` is cargo's
  rename, which this repository's release gate already reads for exactly that reason. Both were invisible, and
  an edge missed on **both** sides of the two-way comparison is a corpus that shrinks while agreeing with
  itself. The graph is read as TOML now, through the same region reader every other manifest question here
  uses, across both dependency tables and their target-scoped forms.

  **The negative run is over the reader, and that is forced rather than chosen.** The two constants are owned
  by the two crates at the top of this workspace's dependency DAG, so no member can be made to reach either
  without closing a cycle — measured: adding `shengmo` to `xuanji` produces `cyclic package dependency:
  package guibiao depends on itself` and cargo refuses to build before any direction runs. The declared side
  of the comparison is perturbable and is held that way; the graph side is not, so five spellings are given to
  the reader directly and the substring form fails three of them.

- **Two figures with no producer, and one annotation a mechanical replacement did not reach.**
  `shengmo::workspace::MARKER`'s own documentation still counted the sites outside its reach after the same
  figures had been removed from the check that reads it — the carrier was cleaned and the token's owner was
  not. `AGENTS.md`'s doc-comment rule recorded four counts to justify having no reaction, and one of them was
  reduced to zero by a sweep in the same window, from the same run that typed it. Both are removed: the
  arguments do not rest on the numbers, and a census belongs where something enumerates it.

  And a claim that *every* annotation now carries an evaluation date was false when it was written: the
  replacement matched `**Not fired.**` and one entry spelled it `**not fired.**`. It carries a date now, and
  the claim names the corpus it swept.

  No published API, outcome, report, exit class, or manifest moves.

- **Pull-request tokens leave the tree, and the relative anchor's sweep reaches the corpus it had missed.**
  Five `PR #N` references sat in test headers where the provenance led and the observation followed — *closed
  by PR #159: `is_self_dependency` matched by NAME ALONE, so a package depending on a different,
  externally-sourced package sharing its name was wrongly swallowed*. The lead clause names when; the rest is
  the falsifier, and only the lead is gone. One kept a sibling suite's path, which is a cross-reference rather
  than a serial and stays.

  And two `this window` references in `openspec/specs/` now name the window. The sweep that removed them from
  `crates/` stated the corpus it swept — the rule this window added — so the claim was not false, but the
  class was wider than the sweep. It is zero across the whole tree now.

  **The review-round token is measured and left, which is the part worth recording.** A sweep proposal put it
  at 88 sites; split by shape, **7 are parenthetical provenance and 68 are grammatically embedded** — *round 9
  closed*, *the round-12 fix that closed the identical gap* — where the round is the sentence's subject.
  Removing those is 68 sentences rewritten by hand, each while keeping the observation source it carries,
  which is the per-site prose judgement this repository has designed, measured three times and rejected. The
  split is in `AGENTS.md` so the next proposal starts from it rather than from the count.

  **Two categories of that proposal are declined against rules this window landed**, and recorded here rather
  than re-adjudicated later. Removing prose by verb tense contradicts the criterion — the test is whether a
  passage carries an observation source or a falsifier, and the proposal's own largest target,
  `hunyi::exposure`'s branch-index derivation, is a worked counterexample carrying both. And removing version
  literals contradicts the versioning rule, which admits them for an immutable historical fact — one of the
  named targets is a `PROJECT.md` Decision's own subject.

  No published API, outcome, report, exit class, or manifest moves.

- **Correction: "Definition of Done all green" was written ten times in this window over runs that were a
  subset of it, and the subset shrank while the wording did not.** The Definition of Done is a named list of
  fifteen commands. Audited across the window's fourteen pull requests, the command **kinds** each record
  names run 12, 9, 8, 8, 6, 6, 3, 5, 3, 4, 1, 2, 1, 2 — and ten of the fourteen call that *the Definition of
  Done*. Two of them name a single command while claiming the whole list; both actually ran six of fifteen.

  **No merge was red.** The full list is green on the release branch, all fifteen commands, and the one commit
  deliberately committed red — the twin detector, red on its own by design — was discarded by its squash and
  is not in the branch's history. What is wrong is the record, not the tree, and a green tree does not make
  those sentences true after the fact.

  It is the same class this window named repeatedly and the sharpest instance of it, because the rule
  forbidding it was added mid-window by the same run: *a completeness claim over a sweep names the corpus it
  swept*. The commands were named, which is the only reason the audit was possible — and the phrase
  *Definition of Done* is itself the completeness claim, over a corpus that has a name and can be counted.
  `AGENTS.md` now says so where the list is defined: run the whole list and say so, or name what ran and drop
  the phrase.

  No published API, outcome, report, exit class, or manifest moves.

- **Correction: the "7 mechanically removable" review-round tokens were a regex artifact, and the real count
  is zero.** The split recorded with the doc-comment criterion measured 88 sites by **shape** — 7 parenthetical
  or comma-set, 68 grammatically embedded. Read in place, the 7 sit in paragraphs that also carry an embedded
  token: removing `(round 9's fix)` leaves *the identical failure mode **round 9** closed* three lines on,
  and removing `(round 6)` leaves *the **round-6/7** grouping* on the next. Taking only the parenthetical
  orphans the reference and produces worse text than leaving both.

  **The shape a regex sees is not the shape a removal has to respect**, and the first measurement reported the
  former as the latter. The correction moves the decline from "mostly not mechanical" to "not mechanical
  anywhere", which is a stronger statement than the one it replaces — the instrument was wrong in the
  direction that made the class look easier than it is.

  This is the fourth instrument of this run to be wrong before it was right: a brace counter that read `{` inside
  a `panic!` string, a commented-out-code detector that matched English sentences opening with `let` and
  `impl`, a test-file filter that missed `tests.rs` and inflated a panic census from 16 to 347, and now a
  shape split read as a separability split.

  No published API, outcome, report, exit class, or manifest moves.

- **The workspace membership reader answered three different facts with one empty set, in shipped code.**
  `workspace_member_names` returned an empty `Vec` when `cargo metadata` carried no `packages` array, when a
  package's `name` could not be read (dropped one at a time by a `filter_map`), and when a workspace genuinely
  declares no member. Both consumers read empty as *nothing to govern*.

  **The two halves fail differently.** Coverage computed `total = 0` with an empty uncovered list and rendered
  it as **complete coverage over a membership it never read**. Evaluation refused — `Subject::of` already
  declines a positive boundary count against zero members — but with the sentence for *a workspace that
  declares no member to observe them over*, which is the wrong fact about the wrong thing when the metadata is
  what could not be read. The first is a silent pass; the second is the absent-versus-unreadable collapse this
  window drew eight times in the repository machinery.

  This crate states the rule itself, on `workspace_member_src_dirs`: *an unreadable workspace is a constitution
  error, never a silent empty set*. The membership reader is the same question about the same metadata and did
  not follow it. It now answers `Members::{Read, Unreadable}`; a package whose `name` cannot be read refuses
  rather than being dropped, and coverage is withheld rather than fabricated when the membership is unreadable.

  **Recorded because of where it was, not what it was.** Every three-state reader this window built —
  `Quoted`, `WorkspaceVersion`, `PackageName`, `Declared`, `Package`, `Tracked`, `Failure`, `Site`, `Supplied`,
  `Delivery` — is in the unpublished repository machinery. This is the first of the class found in code an
  adopter runs, and it was found by first entering that corpus rather than by re-reading a repair.

  Observable only for metadata cargo does not produce: `cargo metadata` emits a `name` for every package and a
  `packages` array for every workspace it can load. No published signature, outcome kind, exit class, identity
  shape, or manifest moves, and no recorded baseline is affected.

- **Correction: the verdict-channel repair did not end the exit-class collapse it said it ended.** `01fbf93`
  records that a `Refused` whose channel write failed "arrived at the wrapper as exit `2` … where the gate had
  found exit `1`", and that the repair closes that. It does not. The channel is absent either way, so the
  wrapper reads unjudged either way and exits `2` before and after. What the repair buys is that the gate now
  **names the channel and the error** instead of failing on the refusal alone — the operator is told which of
  the two facts they have, and the exit class is not what carries it.

  The direction that shipped with it used a clean subject, so it proved the clean arm and was cited for the
  refused one. A direction for the refused arm lands here, asserting what actually happens rather than what was
  claimed, and `repository-checks` gains both scenarios — the gate failing loudly on an unwritable channel, and
  the explicit statement that a refused verdict's exit class is unchanged by it.

- **A dependency table is read as a table.** The manifest reader that derives the constant-owner corpus read
  `key = value` lines and looked for the word `package` anywhere in the value. That missed
  `[dependencies.alias]`, where the heading names the dependency and no key does — and it **deleted** a real
  edge from `{ path = "…", features = ["package"] }`, because the word appears as a feature name and the rename
  branch then found no `=` after it. Both returned an empty set, and an edge lost on both sides of a two-way
  comparison is a corpus that shrinks while agreeing with itself. Nine spellings are now held, including the
  two that broke it.

- **The test helper for the verdict channel named one path and read another.** Its env and its read-back were
  two expressions, so a caller supplying a channel had the child write there while the helper read the default
  scratch file. Every caller passed an **unwritable** path, where empty is also the correct answer — so the
  mismatch was invisible and the assertion over it passed for the wrong reason. One path, decided once, and a
  direction with a writable channel is the one input that tells the two apart.

  No published API, outcome, report, exit class, or manifest moves.

- **Correction: the four figures in the doc-comment rule were never removed, and by the time two reviews
  measured them three were wrong.** An entry in this window records them as removed. The replacement that was
  supposed to do it silently matched nothing and the result was not checked, so the paragraph kept saying
  `crates/` carries 5 `PR #N`, 7 bare `#NNN`, 20 `round N` and 16 `this window`. Measured now: **0, 2, 18 or
  88 depending on whether the hyphenated form counts, and 0** — and three of the four moved *because sweeps in
  the same window, from the same run that typed them*, changed what they counted.

  They are gone, and the argument is unchanged without them: the decidable subset collides with the fixture for
  the squash-serial check, and the rest needs the criterion applied per site. The paragraph's own subject is
  that hand-written figures drift; it enumerated nothing, so it should never have carried any.

- **Correction: the relative-anchor sweep claimed the whole tree and had swept two directories.** An entry
  records `this window` as "zero across the whole tree". It was zero across `crates/` and `openspec/`, which is
  what the sweep covered; thirteen remained in `PROJECT.md` and `BACKLOG.md`. They now name the window. The one
  in `AGENTS.md` stays: it is the rule quoting the shape it forbids, the same way the promotion-trigger rule
  quotes the annotation form.

  This is the second time in one window that a completeness claim outran its sweep, and the rule against it was
  added between the two.

- **The rule about the Definition of Done's name carried its own unanchored census.** It recorded how many
  times the phrase had been misused, over how many commands, eroding from how many kinds to one — four figures
  with no producer, in a rule about naming a corpus honestly. The rule says what it needs without them; the
  measurement stays in the `CHANGELOG` entry that took it.

  No published API, outcome, report, exit class, or manifest moves.

- **The perimeter test applied strictly moved five of eight reasons, not three.** The first pass read
  *entailed by the allowlist* generously and cleared three clauses that do not survive the test the
  constitution's own header states. Applied literally — *delete the thing the clause asserts, and if the
  boundary stays green while the clause turns false, the clause is outside*:

  **A phrase about edges pointing at this crate fails too.** 璇璣 *below every dimension* and 星表 *beneath
  every dimension* name positions the **dimensions'** own allowlists determine, not these boundaries'. Delete
  every edge into them and each stays green while its sentence turns false. Both now say only what their own
  rule sees: they depend on no workspace member.

  And two use clauses survived the pass that was written to remove use clauses. 繩墨 *declares this law through
  the shell* and *every boundary here is stated through the surface the shell publishes*; 勘合 *a repository
  check reads the record through the same published surface an adopter has*. Stop declaring, stop reading, keep
  the edges — green, and both sentences false. Both reasons now state their allowlist and the dimension edges
  it forbids, and nothing else.

  渾儀's *quarantined syn* is the one left standing, with its reason recorded rather than edited: the quarantine
  is real and reacted to, but by the **sibling** allowlists, none of which names `syn`. It is inside the law's
  perimeter without being inside this rule's — the one case where the two come apart, and an edit would blur
  which of them holds it.

  The reasons project into `AGENTS.self-law.md`, so the projection moves with them. No target, rule, or
  allowlist changes; no recorded baseline is affected. No published API, outcome, report, exit class, or
  manifest moves.

- **Four typed censuses were removed, one of them a figure this window had already ruled false.** A census is
  produced, never typed, and the correction recorded above for `seven owners and nine` removed the two phrases
  it quoted rather than the claim: `kanhe::region::DO_NOT_EDIT`'s doc comment still said `seven owners for one
  token` — the seventh was the constant's own declaration — and the entry for that change still carried the
  `MARKER` spelling counts in two sentences the correction had not read. `six per-capability identities into
  two` mixed two denominators a third time, in `manifest.rs`, in `AGENTS.md` and in that entry: the two
  version arms are four identities folding into two, while the whole workspace-version block is six folding
  into three.

  Each is removed rather than repaired, which is the disposition the earlier correction already recorded and
  the reason it gives applies unchanged — nothing enumerates those sets, so any figure there is a census with
  no producer. What each passage argues is unchanged; only the number is gone, and in every case the sentence
  beside it already names the check whose run counts the sites.

- **The suite nobody ran was red for nineteen consecutive merges.** `01fbf93` wrote
  `if let Some(source) = source && !answered.contains_key(source)` — a let-chain the default toolchain
  accepts and the workspace's declared MSRV, 1.85, refuses. Every merge after it failed CI on exactly one job,
  `MSRV (rust-version)`, and every one was merged. Nineteen runs, one cause, all red on the same line.

  **Every local gate was green each time, and that is not a contradiction.** `AGENTS.md` says the Definition
  of Done is the local pre-flight list and that CI runs a **superset** of it; the MSRV job is in the superset
  because it installs a toolchain and rebuilds the workspace, which a pre-flight list should not carry. The
  process was working as documented. What was missing is that nothing read CI's answer before the merge — an
  earlier audit in this window asked whether any red pull request had been merged, checked the local
  Definition of Done, and answered no. It was reading the wrong suite.

  The line is filtered rather than chained, so 1.85 builds and tests the whole workspace again — measured, 68
  suites green on `cargo +1.85`. **Only `kanhe` was affected**, it is the workspace's one let-chain, and it
  ships in no package: no published crate's MSRV moves, and `v0.4.0` builds clean on 1.85 as it always did.

  **`scripts/merge-pr.sh` now reads CI's verdict the way it already reads its own.** The wrapper stands in
  front of `gh pr merge` and refused to reach it without a gate verdict; it now equally refuses a pull request
  whose checks disagree, and refuses one whose checks have not finished — three states, because a run still in
  flight is not a run that failed, and merging on *not success* would refuse a pull request that simply has
  not been answered yet. A head no workflow has claimed is its own cannot-judge rather than a pass.

  **Recorded because of how it was found.** Seven rounds of static review could not see it — the reviewer said
  so in the round that found nothing, naming the unrun suite as the evidence static reading structurally
  cannot supply. Two of the four things it named were CI-only jobs. The packaged-tarball self-test was run at
  the same time and passes for all six publishable crates; this one did not.

  No published API, outcome, report, exit class, or manifest moves; the repaired line is in a crate that ships
  in no package.

- **The guard written for the nineteen red merges could not reach its own third state.** `require_ci_green`
  asked two independent filters about the check rollup, and a pull request with **no checks at all** is a
  value neither can produce: the disagreement filter answers the empty string it gives for *nothing
  disagreed*, and the unfinished filter answers the zero it gives for *nothing is pending*. Neither refused,
  and the merge ran — the same false-negative direction the guard was written to close, arriving through the
  guard. Its own comment asserted the opposite, and the fake answered `''` and `0` for a clean pull request
  too, so the two cases were byte-identical and no direction could have told them apart.

  All three states are derived from **one** read now, and the fake answers nothing for a mode of its own. Two
  more defects in that function go with it: a failed read of the pending count was inferred as *no unfinished
  checks* while every other acquisition in the wrapper routes failure to a cannot-judge, and the disagreement
  read folded the tool's stderr into the value it then tested for emptiness — so a notice on a **successful**
  call would have been reported as a check that disagreed, naming one that does not exist. One stderr
  discipline for the read, and the tool's own cause reaches the operator.

  `repository-checks` gains the one-read requirement, the stderr discipline, and the third scenario.

- **The wrapper chose its exit class at every argument arm, against its own clause.** `repository-checks` says
  the classification SHALL be chosen in one place per wrapper; `publish.sh` routes every refusal through a
  helper and `merge-pr.sh` spelled `printf … >&2; exit 2` inline at each arm. It has that helper now, and its
  arms go through it. The pair is filed with what a review measured of it — most of `publish.sh`'s code
  byte-identical to its sibling, every named construct they share among it — and with why the
  extraction waits: the two wrappers stand in front of the two acts that cannot be undone, and a refactor's
  own risk outweighs the drift it removes until the release is out.

- **A crate's boundary paragraph outlived the edge it denied.** `kanhe`'s module header said what lives there
  *reaches no product contract at all*, in the same paragraph arguing that keeping product and record apart
  is the point. Its manifest declares a `tianheng` edge, four modules under `src/` reach `tianheng::`, and a
  test target composes the observer protocol against it. The paragraph predates the edge and nothing re-read
  it. It now draws the distinction the manifest comment already drew — `kanhe` **reads** the product's
  declared surface where a record is held against it, and runs no product reaction against this workspace,
  which is 繩墨's — and says that it was wrong, since the drift happened in the paragraph declaring the split.

- **A refusal named a fact the rule does not test, and a projection asserted the negation of its own source.**
  The example-pin diagnostic said a pin *the workspace version does not satisfy* while the rule is string
  equality against two spellings, so `= "^0.5"` — which `0.5.0` genuinely satisfies — was refused with a
  sentence sending a maintainer to check semver instead of changing the spelling. It now says what was
  measured, and the spec scenario's heading matches its own WHEN.

  The refusal register's header claimed every refusal is constructed through the site-carrying forms *since
  nothing else exists*. `refusal::violation` and `refusal::cannot_judge` do exist — `Site::OutsideRegister`
  exists for them. Every figure in that document is computed from the register value and this one sentence
  was a literal, which is why it was the one that was false. It is computed too now, and reads as the floor
  its sibling projections carry — and that first computation was itself wrong, corrected in the entry below.

- **Git listings were read without `-z`.** `machinery_names` builds the set `adopter_cited_machinery`
  refuses CHANGELOG citations against, and `core.quotePath` defaults on while `hermetic()` neutralises the
  config that could turn it off — so a tracked path carrying non-ASCII bytes would enter that set in its
  escaped spelling, its real name absent, and a record citing it would go unrecognised. Latent: no tracked
  path needs quoting today. Closed rather than declared because the sibling capability already raises the
  class to a SHALL, measured on a fixture named `ignored-普通`. The one `status` call without `-z` is closed
  with them, where the consequence was confined to how a diagnostic renders.

  No published API, outcome, report, exit class, or manifest moves; every repaired site is in a crate or a
  script that ships in no package.

- **A repair that removed four false sentences wrote three more, and one of them was worse than what it
  replaced.** The round's own subject was *four sentences that outlived their subject*; an adversarial
  review of the repair found it had reproduced the class three times, each time by removing a false claim
  and asserting a new one in the same breath without sweeping from its own base.

  **The figure was produced from the wrong region.** The refusal register's new floor counted constructions
  over the **raw text** of its corpus and rendered a number — every one of which was the English word
  *violation* or *cannot-judge* in a doc comment, and not one a call. The correct reader sits in the same
  file, under a comment naming this exact defect: *counting the bare identifier over the whole text counted
  every doc comment naming a constructor*. On executed Rust that corpus holds **none**, which the count
  beside it already said; the constructions are in the test targets the corpus excludes, and the floor names
  those now. A false figure in a document whose header says every number is produced is worse than the false
  prose it replaced, because it carries the authority of a measurement.

  So `calls` takes a corpus that has already had its comments and imports removed, as a type. Three sites
  passed one and the fourth passed the file, and the difference was a caller remembering — which is the
  failure `region`'s own header was written to make impossible: *a corpus is never handed to a recognizer as
  a string*. The compiler found the fourth site immediately.

  **The boundary clause was false in the same way its predecessor was.** `kanhe` *runs no product reaction
  against this workspace* — except `observer_protocol` calls `tianheng::check_constitution` over this
  workspace's own manifest. The true line is what each call **asks**: `kanhe` runs one with the workspace as
  its *subject*, comparing two composition paths for equality over a non-trivial input, never to enforce the
  law over it. The paragraph now records that it has been wrong twice, in the paragraph declaring the split.

  **A flag changed a stream's shape and its reader did not change with it.** Adding `-z` to `status` made the
  records NUL-separated, and the diagnostic interpolated them raw — so a dirty worktree printed its entries
  run together on one line, where the plain form printed one each, and `trim_end` left the trailing
  separator because NUL is not white space. The `-z` change was made *for* how that diagnostic renders a
  quoted path, and then left the diagnostic alone.

  **And the scope claim described what the commit touched rather than the class.** A fourth listing in the
  same crate reads `git ls-files openspec/specs` and derives each capability's name from the path, so a
  quoted path fails the prefix strip and that whole capability's declared bounds are never registered — the
  same direction, one layer worse. Closed with the others.

  **Declined — and the reason first recorded for declining it was itself false.** The review asked for two
  rows to leave the Definition of Done's list because the workspace test run already covers them. It does,
  and the rows still belong: CI names both as their own steps while the two suites the list's prose says need
  *no line of its own* are named nowhere in CI, so the list is doing exactly what it says. What was recorded
  instead was that `dod_coherence` would refuse the removal. It would not — that check holds one direction,
  every listed command appears in CI, and a shorter list cannot fail it. The mirror runs one way and nothing
  says so, which is why `cargo test -p louke` — the one named CI step the workspace run does not cover — is
  in the list because someone remembered.

  **Corrected with them:** a direction titled for an ordering asserted only that two pure functions agree.
  The ordering — that the channel is written before the panic — is held by construction, and the title said
  it was observed. It now says which.

  No published API, outcome, report, exit class, or manifest moves; every repaired site is in a crate or a
  script that ships in no package.

- **A reaction's reach has three axes, and the rounds closed them one at a time.** What it looks for, what
  it counts as a hit, and **where it looks**. The first two were closed by construction — the capability
  instead of its spellings, and no verb list at all — while the third had never been examined: the corpus was
  one crate's test directory, from the form where the finding happened to sit there, and every widening since
  asked *what to look for* rather than *where to look*.

  The requirement said *every test target* throughout. Four outside that directory spawn a process and two
  run `git` directly, in the crates whose own gates this guard protects — `shengmo`'s examples suite spawns
  cargo and enumerates with git, its family-coverage direction enumerates with git, and two more run a
  program passed as a value. The corpus is now every integration test target the workspace compiles, matched
  by the shape cargo builds as its own binary rather than by a pathspec glob, whose `*` crosses `/` and would
  also take a nested fixture. Twenty-two members.

  Negative run: a spawn added to a `guibiao` test target — outside the old corpus entirely — now fails naming
  it.

- **A detector keyed on spellings was one form short for the third round running.** `hermetic(`, then
  `Command::new("git")`, then `Command::new(args[0])` — the program-as-value form, which the shared
  builder's own header had already recorded as one of the two variants it converged, **before this guard was
  written**. Each round the requirement was right and the reach was not, and `gate_identity.rs` ran
  `git ls-files` twice through the third spelling while the set equality passed over a set that could not
  contain it.

  Three instances are one defect. The question is now the one with a single syntactic form and no knowledge
  of the program — **does this target spawn a process itself** — so the guard no longer has to know how
  `git` is spelled, or that it is `git`. The earlier form's own argument, that an allowlist is stricter than
  a denylist, applied to the set and left the detector a denylist of known spellings; it applies to both
  now. The cost is three more members out of eighteen files.

  **A probe that measured nothing found the boundary defect.** Renaming a member's `Command::new(` to
  `PhantomCommand::new(` did not move the verdict — because the marker is a substring of the rename. The
  recognizer excluded a preceding quote and not a preceding identifier character, so a different type's
  constructor read as a spawn. Both boundaries now, and the perturbation moves the verdict.

  Three of the fifteen purposes were also wrong or short against what their file invokes: one named uses
  that go through the invisible spelling while omitting the `worktree remove` it does directly, one omitted
  its fixture initialisation, and one attributed a sibling direction's enumeration to itself. All eighteen
  are rewritten from a per-file reading, and the entry says what it is: prose with no producer, a reader's
  aid. What the direction holds is membership.

- **The guard held a denylist, and both wrappers already argue why that is the weaker direction.** Its
  membership was *does this target write with git* and it carried a typed list of write subcommands — so it
  was complete only while a prose sentence about the tree stayed true, with nothing holding it. A fixture
  gaining `git rebase` would have made the sentence false and dropped that write out of reach, **together
  and silently**: the claim and the coverage fail as one.

  Membership is now the question with no list to keep — *does this target run `git` itself* — so an
  unfamiliar use is a member by default, which is a file to look at rather than a gap. That is the argument
  both irreversible-act wrappers make in their own headers, applied to the guard that was holding the other
  kind. The declared set is fifteen entries, each saying what that target uses git for; most only enumerate
  what the repository tracks, and the two that create commits or tags route those through the shared builder
  and keep only reads.

  **Three forms, and each was corrected on the axis the last one moved.** The first was wider than its
  detector; the second met its detector and bought completeness with a claim; the third removes the claim.
  What ends this shape is not a better list but no list — the same move that ended the lexer thread five
  entries above, where a sequence of filters became one pass over the grammar.

  Both directions run red without the guard: an unnamed target that gains a direct run fails naming it, and
  a named target that stops running git fails too.

- **Two reasons written beside a change described the tree the change started in.** The converged commit
  site said the delegation *keeps this file out of the set of targets that write with git directly* — and
  the file is in that set, named in the declaration, because membership is per file and it still holds a
  direct read beside the write verb it now delegates. The rule has an owner and the site restated a
  consequence of it backwards; the site says why it delegates now and leaves the set to the declaration.

  And the guard's exclusion note credited the wrong mechanism. It said recognition is by position *because
  this file holds every marker as a literal* — measured over the executed text the check actually reads,
  `hermetic(` is the only marker whose own spelling produces the sequence it searches for, and its single
  executed occurrence is exactly the quote-preceded one, so the lookbehind is load-bearing there and nowhere
  else. The five write markers produce no executed sequence at all: the quotes inside them are escaped in
  the array that declares them. What keeps this file out of its own set is holding no executed write plus
  that escaping — and the escaping is the fragile half, which is worth naming because a rewrite of the
  array that stopped escaping would make the lookbehind load-bearing where it is not.

  **The distinction bit while the correction was being written.** Naming a write marker in the new paragraph
  put a real one in the file; the comment strip removes it and a raw reading would not have. Measured after
  the edit rather than before it, which is the only order that catches this.

  `-qm` and `-am` are recorded as belt-and-braces — a `commit -qm` already carries `"commit"` and a
  `tag -am` already carries `"tag"`, so neither can fire alone — with the sweep that says the verb list is
  complete for this corpus: no `rebase`, `cherry-pick`, `revert`, `commit-tree`, `stash` or `filter-branch`
  anywhere in `crates/`.

- **The guard's requirement was wider than its detector, which puts the gap in the floor.** It said *test
  targets that run `git` without the shared builder* and detected one spelling of the invocation —
  `hermetic(` — while **fourteen** targets ran git through a bare `Command::new("git")`, this guard's own
  file among them. The declared set named one file and the equality passed, because the detector could not
  produce the others. A set comparison is only as wide as what reaches it.

  The requirement is narrowed to the class the fixture-date discipline is about — a run that **writes**,
  since a write is what carries a date — and the detector is widened to every spelling of an invocation in
  executed text. Membership is per file: a direct invocation anywhere in it and a write verb anywhere in it,
  which over-includes a file whose direct runs are reads. That is the safe direction, and each member's entry
  says which it is.

  It found a real one immediately: `capability_subjects.rs` created a commit through a bare
  `Command::new("git")`, taking both dates from the clock. That commit goes through the builder now. The two
  members that remain hold reads beside writes that already delegate.

  Both directions run red without the guard. Recorded with them: the first attempt at the second negative run
  reported green from a **stale binary** — a blanket replacement had also changed a sibling constant's length
  and the target did not compile, and the passing line came from the previous build. Read what the run
  actually was, not that a run was green.

- **The convergence took one of two copies, and the day split between two constants.** Both are the same
  finding at two scales: one helper with two implementations, one fact with two owners — and each repair
  closed the instance in the file it was already editing.

  `publish_source.rs` held the same `hermetic("git")`-plus-assert runner its twin in `release_coherence.rs`
  did, byte-identical past a doc comment, and fifteen commit- and tag-creating calls went through it —
  three of them fresh `release: 9.9.9` commits taking **both** dates from the clock. It delegates now. The
  two direct runs that remain there are reads, which ask the repository a question and write no commit.

  And `FIXTURE_DATE` spelled the day a second time six lines under `FIXTURE_DAY`. The `concat!` around it
  reads as a derivation and is not one — measured with rustc, that macro takes literals and not a constant's
  name — so a second literal wore the appearance of the first. The constant is gone: `Command::env` takes
  anything `AsRef<OsStr>`, so the value that was needed was a `String` and the `const` was never required.
  The day is written once.

  **The reading that finds this is not the reading that finds a file's own defects.** Reading a file finds
  the copy being edited; the copy that is not is found by reading the *pair*. So the set of test targets
  running `git` outside the shared builder is declared and held against the tree in both directions — a
  target that gains one is named with why it may, and a name that outlives its reason fails too. It
  recognizes the call by position rather than by the bare marker, because the check's own source is in the
  corpus it reads and holds the marker as a literal.

  `repository-checks` gains the requirement with a scenario for each direction.

  Recorded with them: the previous round's own repair of this constant was reverted before it landed. A
  `git checkout -- <file>` restoring a temporary perturbation took the refactor with it, and the run
  afterwards was green because the reverted tree is internally consistent — a restore verified by a passing
  suite rather than by its content. That is the second time in this window; the rule that failed was staging
  before the *real* edit rather than before the *temporary* one.

- **The fixture date got a constant from the half that needed it, not from the fact.** The commit half was
  given `FIXTURE_DATE`; the other half — the changelog section that same check compares it against — stayed a
  literal in the generator and in four directions. One fact, two consumers, an owner available and unused.

  The day is the owner now and the instant is built from it. A first attempt reached for `concat!`, which
  takes literals and not a constant's name, so the day would have been written twice under a macro that
  reads as though it were derived — worse than two plain literals, and caught by running the pair rather
  than by reading it: changing the day alone left the commits on the old one and the changelog on the new,
  and the gate refused naming both. Changing it now moves both halves and the suite stays green.

- **A second command helper stood beside the shared one, and the date extraction walked past it.**
  `release_coherence`'s directions had their own `git()` — `hermetic("git")` plus an assert, which is
  `hermetic_git::fixture` minus the fixed dates — and four commit-creating paths went through it, including
  the one the date direction adds. So every amended fixture HEAD carried a wall-clock committer date while
  the constant beside it existed to stop exactly that. The fourth instance of the class that module's own
  header is about; every `git` in that file goes through the builder now.

  **And the reason recorded for setting the committer date was false of the tree it was written into.** It
  said a fixture setting only the author date would record a wall-clock committer date *which a later
  direction would read* — nothing reads `%cd`, the spine reads `%ad`. It is set for symmetry rather than for
  a consumer, and the note says which, so the next reader does not go looking for the one that does not
  exist.

- **A release section's date is held against the commit that makes the release.** `is_iso_date` was
  hardened twice — parsed rather than counted, then ranged rather than digit-tested — and each step asked a
  sharper question about the **shape**. The value was never asked, and the value is what a reader takes the
  release to have happened on. Three releases carried a section date equal to their `release: X.Y.Z` commit's
  date because a person remembered; the fourth was prepared four days behind the day it would be cut on, and
  nothing would have said so.

  The comparison runs **only at the snapshot**, which is the first moment the answer exists: before the
  release commit there is nothing to date against, and a date written during preparation is an intent rather
  than a claim. So the check stays silent through development and release-ready and speaks at the one commit
  whose date is the answer, naming both dates so an operator can see which to change. The spine already read
  the commit; it reads its date from the same log line.

  **The fixtures commit at a fixed instant now**, which the new direction needs and which every fixture
  benefits from: with the date taken from the clock, a literal release date and its commit agree only until
  midnight, and the fixture would be asserting the machine rather than the subject. Both the author and the
  committer date are pinned, because git takes them from different variables and a fixture setting only the
  first would still record a wall-clock committer date.

  `release-coherence` gains the requirement with its scenario, and the control is the existing coherent
  snapshot — the same fixture with the date left agreeing.

- **A debt entry counted the tree before its own repair, and named the invalidation two sentences later.**
  The WATCH entry filing the two wrappers as one lifecycle written twice recorded the overlap as a figure and
  a list of five shared named constructs — and the same paragraph records `merge-pr.sh` gaining `refuse`,
  which is the sixth. The repair that closed the idiom divergence added a shared construct in the commit that
  filed the debt at the pre-repair count. Nothing catches it: the census reaction reaches the phrases a census
  declares, and an entry's arithmetic declares none.

  Both figures are gone rather than corrected. The entry's promotion trigger reads *a second divergence
  between the two wrappers' shared constructs*, which is a shape; the arithmetic is what goes stale, and this
  entry is written to be re-read later by someone who will re-measure anyway.

  The discipline is the one already applied to a repair's own doc comments, moved to a governance record: an
  entry describing a repair is written from the repaired tree, not from the tree the repair started in.

- **The dirty-worktree diagnostic is observed, after six rounds of being the finding nothing holds.** Its
  direction asserted the refusal's opening sentence and nothing else, so the render broke when `-z` arrived —
  the NUL-separated records ran together on one line — and was repaired again with neither change failing
  anything. It now asserts that each path is named as the repository holds it, unescaped, one per line, and
  that no separator reaches the operator.

  Two dirty paths, one of them non-ASCII, because with a single record a run-together render and a
  one-per-line render are the same string. Both negative runs, from the terminal: without `-z` the non-ASCII
  path arrives in git's quoted spelling and the first assertion fails; without the split the message reads
  `?? plain.txt ?? 普通.txt` on one line and the separator assertion fails.

  `publish-source-integrity` gains the requirement with its scenario.

- **A closing pipe is not a parameter position, and the difference was one real construction.** The
  introduction test — *a closure parameter or a `let` binding introduces the name; everything else
  references it* — is a statement about **position**, and it was implemented as the character standing before
  the name. A parameter list's closing `|` looks exactly like its opening one to that test, so
  `.map_err(|err| cannot_judge(…))` was excluded: a construction that references and calls, missed in the
  false-negative direction this count exists to catch. One instance in the corpus today, in a test target;
  none in the crate the published figure reads, where one idiomatic `.map_err(|e| cannot_judge(…))` would
  leave a real site-less construction unreported while the projection states none exists.

  The count is positional now — an odd number of pipes before the name on its line means inside a binder,
  and the closing pipe makes it even again, which is exactly the boundary the character test could not see.
  `||` is even whether it opens a zero-argument closure or means boolean or.

  **The direction's shapes were chosen from the defect and not from the predicate's edges**, which is the
  same shape as the round before it: three cases each asserted that a binder is excluded and none asserted
  that a construction *adjacent* to one is kept. Three more join them, including the two-on-one-line case
  where the parameter is excluded and the reference in the body is not.

  **And the floor named one obstacle where there are two.** It said the census has no producer because a
  bare reference is undecidable from text — true, and a floor. It did not say that the introduction test is
  an approximation of a position, which is a debt with a fix. A reader acting on the floor as written would
  have concluded nothing could be done. It now names both and says which is which.

- **Two ordered passes cannot close a mutual dependency, and the ordering was the defect rather than its
  direction.** Comments and string literals each delimit the other. Stripping comments first truncates a `//`
  inside a literal and leaves an unmatched quote; stripping literals first lets a lone `"` inside a comment
  open a string that runs into real code. The previous round chose the second, documented why, and closed
  every token that can hold a quote **in code** — and four files of this repository's own corpus carry
  comments with an odd number of double quotes. Measured on `region.rs`, whose comments hold 43: three `pub
  fn` declarations were being swallowed, today, in the corpus feeding the figure the register refuses a
  non-zero answer for.

  One pass now, tracking both states: inside a comment no quote opens a string, inside a string no `//` opens
  a comment, with nested block comments and byte-prefixed raw strings. The ordering question disappears
  rather than being answered, and the shared reader is no longer in this path at all. An escaped newline
  inside a string is emitted rather than consumed, without which every line index after a `\`-continued
  string shifted — which is how the corpus direction below first reported a declaration lost that the reader
  had not touched.

  **The direction that would have caught it is over the corpus, not over fragments.** Eleven declared shapes
  were each a well-formed Rust fragment; none was a comment carrying an unbalanced quote, which is what the
  corpus actually holds. A declaration is code, the reader removes no code, so any file where a line that
  declared one no longer does names a span the reader lost. It is asserted at the same line index in both
  texts, because these files embed Rust in literals on purpose and an embedded fragment never starts its own
  line.

- **A reader named for calls counted occurrences.** `calls` tested a word boundary on each side and nothing
  else — and `(` is a boundary, but so are `|`, `.` and `,`, so a closure parameter, a field access and a
  binding all read as constructions. Live: `.any(|violation| (dimension.reacted)(violation.kind))` contributed
  two to a module that constructs none there.

  **Requiring a following `(` is the obvious repair and the corpus written for this reader refuses it.** Its
  fixture `a_constructor_taken_by_name` is `let build = violation;` — a constructor taken by name and called
  through the alias — declared as one construction. So the question is not whether the name is called but
  whether this occurrence *introduces* it: a closure parameter or a `let` binding introduces, a field access
  projects a value that merely shares it, and everything else references. Three shapes join the direction,
  and one of them records what the reader cannot tell apart rather than asserting it away — a bare reference
  is a constructor taken by name or a local sharing its spelling, and nothing in the text says which.

  **The floor's reason is restated on the property rather than the history.** It said no count is given
  because three readers had disagreed; two readers now agree and are both wrong for the same reason. What
  prevents the census is that this reader tells one region from another and does not tell one token role from
  another — which a reader can act on, where a fact about the past cannot.

- **The literal scanner had no arm for the tokens that can also hold a quote.** It read `"…"` and `r#"…"#`
  and nothing else, so a char literal spelling a quote — `if c == '"'` — opened a string at that quote and
  ran to the next one anywhere after it. Both directions fail from there: a construction between two of them
  is swallowed, which is the false negative the unregistered count exists to catch, and a literal's contents
  after one are exposed as code. Five modules of the reader's own corpus carry the shape, and a byte-prefixed
  raw string, `br#"…"#`, was read as an ordinary string because the `r` sat behind a `b`.

  The arms are enumerated now rather than added one per defect — char literal, raw string with or without a
  byte prefix, string with or without one — and a lifetime is told from a char literal by looking for the
  close before consuming anything. Five shapes join the direction: a construction between two quote char
  literals, a literal after one, an escaped quote char literal, a lifetime, and a byte string raw or plain.

  **The two figures this reader does produce were true, and the claim that they had become true by
  construction was not.** That sentence is corrected in the entry below: the repair closed the arms for
  tokens that can hold a quote **in code** and left the mirror open — a comment can hold an unbalanced quote
  too, and this reader ran on raw text.

  **And publishing no count was what contained it.** Had a figure been rendered from the previous reader the
  projection would carry a fourth wrong number. The recorded reason — three readers, three answers, no two
  agreeing — now has its cause: no literal handling, literals stripped after comments, and literals stripped
  before comments but with no char-literal arm.

- **The sentence that caused all of it was still there.** `executed_rust`'s doc said it removed *comments,
  string literals and imports* while its body removed two of the three, and the type introduced beside it
  said the opposite one screen down. It survived two repairs aimed at exactly that half, because both read
  the body to find the defect and neither re-read the sentence. It now says what it does and points at the
  corpus that removes literals.

- **A direction asserted an exit class and not the message, and the two refusals it separates collapse
  without it.** `(( changed == 0 ))` is *true* for `not-a-number` — bash arithmetic resolves an unset
  identifier to zero — so deleting the shape guard left that direction green while the wrapper told the
  operator the pull request changes no file, which is the other refusal about a different fact. The message
  is asserted now, as its five siblings already did, and the negative run fails without the guard.

  Two readability repairs go with them: the changed-file guard had been inserted between `require_ci_green`
  and its own header, so three paragraphs about reading CI read as documentation for it and the function they
  belong to had none; and a `Vec<char>` was named `bytes` in the one function whose correctness turns on that
  distinction.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate or a script that
  ships in no package.

- **A record reached the release branch and the work did not.** `4cd8bee` asserts seven repairs across five
  files and its tree is byte-identical to its parent's. The content had been committed onto the release
  branch itself while the branch the pull request named still pointed at an already-merged commit, so the
  pull request's diff was empty — and every guard the merge wrapper has was satisfied: the live commit set
  was non-empty, the message gate judged it against that set, CI was green because nothing had changed, and
  the head pin named a real commit. The only copy of the work was then discarded by a reset to the remote,
  and it survived as a dangling object.

  It is recovered whole and lands here, so the seven claims are true of the tree. `4cd8bee` is left standing:
  it is on the remote and a merged squash is a record, which is the same reasoning this window has applied
  four times — the correction lives beside it rather than in it.

  **And the wrapper now reads what it had been inferring.** A pull request's changed-file count is the one
  question it did not ask, and nothing anywhere holds a merged squash's tree against its parent. 勘合's own
  name is a document made in two halves proven genuine by fitting them together; this is the case where one
  half is empty. A count of zero refuses, and a count it cannot read refuses separately — never as a count of
  some. `repository-checks` gains the requirement with both scenarios.

  The message being curated apart from the tree is deliberate and stays: it is what lets a squash message be
  the record. The pull request's diff is the only thing tying the two together, and now something reads it.

- **The region defect closed; the figure was wrong three times, so it is gone.** `calls` takes a corpus by
  type now and the compiler holds it — that repair was right. What it did not reach is one layer inside the
  guarantee: the reader's own doc claimed it removed *comments, string literals and imports* and its body
  removed two of the three, which `region`'s header already declares as a residue of the Rust rule. Inert
  while the corpus was `crates/kanhe/src`, where the executed count is zero; load-bearing the moment the
  corpus moved to the test targets, in the same commit that moved it.

  Implementing the third removal moves the figure and does not settle it: three readers have now produced
  three answers and no two agree. Calibration says the implementation is right where it can be checked —
  over a module whose real constructions are countable by hand it answers exactly that number, and a new
  direction holds six shapes (a call, a literal, a raw literal, adjacent literals in one expression, an
  import, a comment). It still disagrees with an independent reading over the whole corpus, and the
  disagreement is concentrated in the register's own module, whose source sits in the corpus it reads.

  So no count is given. That is the disposition this window has reached four times and it applies to its own
  instrument: a figure produced by a reader two independent measurements disagree about is a census this
  register cannot produce, and the floor says so rather than rendering a third guess. What the floor asserts
  is unchanged and is what matters — those constructions carry no site identity, they are outside this
  corpus, and none is registered, held or declared here.

- **A direction's name outlived the correction to its own summary.** The summary line was fixed to say the
  body asserts two pure functions agree rather than an ordering; the function name still said
  *reaches the channel*, and the name is what appears in test output where the doc comment does not.

- **The reason recorded for declining a finding was false, and it asserted a protection that does not
  exist.** `dod_coherence` holds one direction — every listed command appears in CI — so removing a row from
  the Definition of Done cannot fail it. The decline stands on the reason that does hold: CI names both
  suites as their own steps, while the two the list's prose says need no line are named nowhere in CI. The
  one-way mirror is filed with its observation source: three named CI steps sit outside the list, covered by
  a listed command, and `cargo test -p louke` — the one the workspace run does not cover — is listed because
  someone remembered.

  No published API, outcome, report, exit class, or manifest moves; every repaired site is in a crate that
  ships in no package.

- **A brace-counting extent reader is closed by a real parser, and the anchor bug it shared a table with
  survived narrower.** `observer_protocol.rs`'s bounds-method extent step counted braces by eye and mistook
  one inside a block comment or a string literal for the body's own — a declared bound, safe in its one
  comparison because a moved extent could only refuse a conforming body, never accept a divergent one.
  `syn_body_span` replaces the count: kanhe's dev-dependency on syn tokenizes a comment or a literal as what
  it is, so no brace inside either is ever available to be miscounted, and the reader declines rather than
  guesses where a real parse, or a real match at the anchor's own offset, is unavailable.

  Two of `ANCHOR_CASES`'s rows demonstrated this bound by hiding a decoy definition inside a comment or a
  literal, and closing the extent bug forces both from `ReadsTheWrongBody` to `Declines` — a mathematically
  forced consequence, since no parser finds a function inside a comment's or a string's own token. The anchor
  step itself (which occurrence is the definition) is untouched, and its own bound — a whole-line occurrence
  anchors the read whether or not it is the definition — stays open in `observer-protocol/spec.md`, exactly as
  written before this change. What no longer demonstrates it is replaced with what still does: a same-named
  `bounds` method on an unrelated `impl`, real and parseable, which `anchor()` cannot tell from the intended
  definition for the identical reason the closed cases could not — a case the scenario's own wording already
  covered before this change gave it a row.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **The refusal register reads Rust, not text, and its stated bound narrows to the residue a parser cannot
  close.** `refusal_register.rs`'s character-by-character scanner — `code_only`, `calls`, `first_literal_args`,
  `imports_and_rest` — is replaced by a reader built on `syn::parse_str` and `syn::visit::Visit`. Exclusions
  that used to be text heuristics (pipe-counting for a closure parameter, a `.` check for field access, an
  `fn`-suffix check for a definition) are now free consequences of AST node type: a bound `Pat`, a `UseTree`,
  or a `Signature::ident` is simply never visited as an `Expr::Path`. A byte char literal, a raw string, and a
  closure parameter list wrapped across two lines — three shapes that desynchronised the old scanner the first
  time each was found — are closed by construction rather than by an arm added per shape.

  `openspec/specs/repository-checks/spec.md`'s "A construction shape the register's reader does not model"
  bound narrows to what a parse tree genuinely cannot decide: whether a bare reference to a registered
  constructor's name is the constructor taken by value or a local variable sharing its spelling, which needs
  name resolution rather than syntax. `crates/kanhe/src/bounds.rs`'s matching declaration and `BACKLOG.md`'s
  watch entry are narrowed in lockstep; `docs/observation-bounds.md` and `docs/observation-bound-extents.md`
  are regenerated.

  A raw string literal used as a registered site's identity, previously unparseable and filed under
  `UNREADABLE_SITE_CASES`, is now read like any other string — `syn::LitStr::value()` decodes a raw string
  exactly as it decodes a plain one, so there is no special case left to write for it. `docs/refusal-register.md`
  is byte-identical to what it was before this change: this repository's own tracked source carries no
  registered site the old and new readers disagree about.

  `code_only`'s own doc comment, and the one test that reads its output, both assumed a string or char
  literal's interior is blanked so an embedded `pub fn`-shaped fragment is never read back as a real
  declaration — adversarial review found the first implementation instead kept every literal's span as
  in-code, benign only because no tracked file happens to open such a fragment at column zero of its own line.
  Corrected before landing rather than left as a doc comment the code did not yet match.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **An unreadable re-export statement is refused, not read as a promise of nothing.** `prelude_promise`'s
  reader returned `Ok` of an empty member set when a `pub use super::{` statement reached no `};`, which the
  vacuity guard one level up then reported as *the promise parsed to no member* — an input the check cannot
  read wearing the diagnostic of a prelude that genuinely holds nothing, and discarding whatever earlier
  statements in the same block had already contributed. The two demand different repairs: one is a malformed
  statement to fix, the other a prelude with no members. `Unreadable` gains a fourth variant carrying the
  statement it could not terminate, and `judge` reports it as itself.

  The direction was always safe — it refused either way — so what moved is the diagnostic and not the exit
  class. The shape is reachable rather than hypothetical: `pub use super::{A, B} ;` is legal Rust and carries
  no `};` substring. `openspec/specs/repository-checks/spec.md` gains the requirement clause and its scenario.
  The pin gives the reader **two** statements with the second unterminated, because one on its own returns an
  empty set under both readings and cannot tell them apart; without the repair it reports `Ok({})`, naming the
  `Alpha` that the statement before it had already declared.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **The OpenSpec adoption mode now says which mode this is.** `PROJECT.md`'s decision recorded `specs` half
  only, on the measurement *zero change directories have ever existed*. That was false in the plainest sense:
  `git log --all` reaches dozens of them, `c52cd1d` and `9fa3ee5` each committed one on 2026-08-21, and the two
  commits on that day's tip say in their own bodies that the work was proposed and synced through one. The
  claim was true of the **release spine**, which is the corpus it never named — the completeness failure
  `AGENTS.md` names one section earlier, in the document that names it.

  What the tree actually does is now what both documents say: **both halves, one requirement truth.** A
  requirement is read from `openspec/specs/*` and nowhere else; `openspec/changes/<slug>/` plans one open
  change, is committed on the development branch, and is stripped before the squash, so `main` and every
  `release/*` track nothing there but `archive/.gitkeep`. What is transient is the plan, not the requirement.

  **Four repository checks already encoded this, which is how the claim was caught.** `reference_integrity`
  excludes a change directory from its corpus and treats a basename tracked only there as lifecycle
  vocabulary; `law_restatement` excludes it beside `docs/history/` and `CHANGELOG.md`; `capability_subjects`
  takes it as a corpus. One of those exclusions is a **requirement with a scenario** in
  `openspec/specs/reference-integrity/spec.md` — so a capability spec and a `PROJECT.md` decision had been
  contradicting each other for the whole window, which is the conflict `AGENTS.md`'s document-authority rule
  says to fix before building on it.

  Two sentences that over-reached the same way are corrected with it. `AGENTS.md`'s *the
  `change/<openspec-name>` branch role went with the half* retired the role correctly and said the half went
  too; the role stays retired and a change directory rides a `<type>/<scope>-<slug>` branch.
  `capability_subjects`'s own comment said its corpus *can never be non-empty* — true where CI runs it, false
  where the direction actually runs. `BACKLOG.md`'s closed entry and its live `capability-subjects` WATCH each
  carry a trailing note rather than a rewrite, per this repository's idiom for an entry a later change
  orphans; the WATCH's trigger loses the half that had already fired.

  No published API, outcome, report, exit class, or manifest moves. No reaction changes behaviour: the one
  code edit is a comment.

- **`syn`'s perimeter is stated at the table the law observes, and the class behind it is filed.** The
  `[workspace.dependencies]` comment read *quarantined to 渾儀 (hunyi) alone — never the 圭表 core or 璇璣 model
  (enforced by `crates/shengmo/src/law.rs`)*, unchanged since `2f903fb`, and `alone` had been false since this
  same window added `syn` to `crates/kanhe/Cargo.toml`'s `[dev-dependencies]` for 勘合's test-only Rust
  scanners. The parenthetical over-reached in exactly the direction the new edge sits:
  `restrict_dependencies_to` observes one table and `Normal` is the default every self-law boundary takes — a
  specified default with its own scenario in `openspec/specs/crate-dependency-boundary/spec.md`, which is why
  the edge needed no amendment and why the sentence could not cover it. The comment now separates what is
  reacted (among `[dependencies]`, 渾儀 alone) from what is not (`[dev-dependencies]` is outside every
  allowlist, and 勘合 occupies it), and the crate-family header above it says *the only packaged crate* rather
  than naming the whole tree.

  **Two claims of one shape in one window, so the class is filed rather than only repaired.** `BACKLOG.md`
  gains a `READY-PATCH` for it — a claim about an enumerable property of this tree, written with no producer
  and an edge wider than anything that reacts — with the design that shape admits: `census` already declares
  the sentence a **figure** is written in, and the missing sibling declares one whose held value is a produced
  **set**, so *only 渾儀 names syn* is compared against the enumerator that answers it. A detector over prose
  stays refused, on the measurement that refused it three times. What the declaration cannot cover is written
  into the entry rather than left to be found: coverage is an author's act, and a role description no check
  enumerates a set for stays a reviewer's. A sibling `WATCH` records that filing the class does not close it.

  **And one filing that is an observation rather than a defect.** The drift law — *no target or name without a
  reaction* — has never been turned on the repository checks themselves: every one reasons well in its header,
  and nothing enumerates which have an observation source in the sense this repository demands of a backlog
  entry. Measured at `HEAD` with the one pinned enumeration the entry carries: the packaged product's logic
  gained +2230 lines in this window while the machinery gained +28687. Both readings of that ratio
  are stated in the entry, because the evidence does not choose between them, and the audit that would is
  named and deliberately not attempted — it is the enumerated-surface cycle `PROJECT.md` records, and a window's
  work rather than a review round's.

  No published API, outcome, report, exit class, or manifest moves; no reaction changes behaviour, and no code
  was edited.

- **A dependency key this reader cannot decode is refused, not skipped — the release gate's own false
  negative.** Where an example declares no `package`, its key *is* the identity, and the key was taken as the
  raw text before the `=`. TOML admits a quoted key and cargo decodes it — measured, `"serde_json" = "1"`
  resolves to a dependency named `serde_json` — so `"xuanji" = "0.0.1"` is a real family requirement whose raw
  spelling matches no family member. The `!family.contains(…)` filter dropped it by `continue`, and the
  requirement counter is an **aggregate** over every example, so one example's quoted pins contribute zero
  while the others keep the total non-zero and the guard stays silent.

  That is the one class the Core Contract forbids, and it is the **second door** into a class this file's own
  comment already describes verbatim for the first: *cargo renames with `alias = { package = "xuanji", version
  = "stale" }`, `alias` is in no family, and the entry was skipped entirely — while the aggregate
  `requirements` counter stayed non-zero on the strength of the other examples.* `Package` gains
  `KeyUnreadable`, `Package::of` judges the key where it already resolves the identity, and the one consumer
  the compiler names answers it with a cannot-judge.

  Refused rather than decoded, deliberately: decoding is TOML string parsing, which `BACKLOG.md` already files
  as its own entry with its own trigger, and a reader that refuses what it cannot decode cannot narrow the set
  it judges. Measured before writing — no tracked manifest carries a non-bare dependency key, so this refuses
  nothing the tree has.

  The negative run is at the **verdict** level rather than at the reader's: with the key check reverted,
  `require_example_pins` returns `Ok([("crates/xuanji/Cargo.toml", "xuanji")])` over a stale `"0.0.1"` — the
  gate reporting a clean release. The pin carries a second, correct example for that reason; with only the
  quoted one present the counter reaches zero and the existing vacuity guard refuses for a different reason.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **The example requirement count is per example, and the aggregate guard is gone rather than kept beside
  it.** The counter lived outside the loop over `examples/`, so it answered *did any example declare a family
  requirement* when the question it was guarding is *was every example examined*. Seven parsing kept it
  non-zero while an eighth went unexamined — and both identity doors closed in this window reached a clean
  release through exactly that: a renamed key, whose repair records it verbatim, and a key this reader cannot
  decode, closed one change earlier.

  One example is one subject: whatever the reader failed to see there is invisible to every other example's
  success. An example declaring no family requirement is now its own cannot-judge, **naming that example**,
  and the site id moves from the aggregate `#no-family-requirement-in-examples` to the per-example
  `#example-requires-no-family-crate` — the singular form its `#example-pin-absent` and
  `#example-pin-unreadable` siblings already use.

  The aggregate guard is **deleted, not kept**: with every example refusing on its own zero, a run past the
  loop has each example contributing at least one requirement and a run with none is the guard above it, so
  no input reaches it. Keeping it would be the dead branch this same function refuses one read earlier — *a
  branch no input can take, which is dead code rather than a guard*. Its WHEN moved rather than vanished: the
  fixture that reached it now reaches the per-example refusal, and the direction that pinned it is rewritten
  onto the new site rather than deleted. A third direction, the one enumerating every vacuity guard, was
  updated by being run.

  The negative run is the aggregate shape itself: hoist the counter back out of the loop and the gate answers
  `ok release coherence (development: 0.2.0)` over an example it never examined, beside a sibling whose pin is
  correct. That two-example fixture is what the pre-existing one-example direction could not be — with a
  single example the aggregate and the per-example guard agree, which is why the hole survived a direction
  that already covered the site.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **The merge wrapper now says what it holds, reads both rollup shapes, and its filter is finally under
  test.** Three findings in one file, and the third is why the first two were invisible.

  `scripts/merge-pr.sh`'s header said *whether CI is green remains a human's call* while `require_ci_green`
  refused a red or unfinished rollup unconditionally — a premise the wrapper's own newer code had falsified,
  left standing where an operator reads it first. `require_ci_green` landed 204 commits after the `--admin`
  arm was reasoned **from that premise**, which is how a stale premise spreads rather than merely sits.
  `--admin` stays admitted for what it still does — bypass required **reviews**, which a single-steward
  repository needs because a pull request's author cannot approve their own — and its arm no longer claims it
  reaches past CI.

  **The rollup is a union of two node shapes and the filter read one.** `StatusCheckRollupContext` is
  `CheckRun | StatusContext`; a commit status carries `.state`/`.context` and neither `.conclusion` nor
  `.name`, so `(.conclusion // "")` answered `""` for every one of them and a **failed** status classified as
  *unfinished*, reported as `these checks have not finished: ?`. That is verbatim the wrong sentence the
  stderr-capture rule already forbids, returned through the shape the filter never covered. Fail-closed
  either way, so latent — this repository produces no commit statuses — and latent is not fixed: a check
  reported under a name nobody can find is what sends an operator after the wrong thing. `EXPECTED` is
  classified **unfinished rather than agreeing**, a deliberate departure from the review that found the
  union: GitHub's meaning is *required and not yet posted*, so agreement would merge past a status that never
  arrived.

  **The filter was executed by no direction at all**, and that is the finding behind the other two. The `gh`
  stub printed the already-transformed `<conclusion>\t<name>` lines, so it stood exactly where the filter
  ran: a filter reading half the union was unobservable, and so would its repair have been. The stub now
  emits raw JSON and applies the `-q` filter **it was handed**, so the filter keeps one owner — copying it
  into the stub would be two places that must agree with nothing comparing them — and every one of the
  target's existing directions passes unchanged through real `jq`.

  Three directions added. Two carry negative runs; the third pins the contract and says so, because it does
  not move behaviour — `require_ci_green` already refused a red suite with `--admin` passed, and what was
  missing was any direction that looked, which is exactly how the arm's prose went on describing a withdrawn
  premise. The `EXPECTED` negative run had to move **only** the classification: reverted together with the
  filter it passed, because the old filter answered `""` for a commit status and so already read as
  unfinished — accidentally right for the wrong reason.

  `reference_integrity` refused the change once on the way in: a new comment said *the paragraph below*, and
  a positional reference names nothing checkable. It names the rule it means instead.

  No published API, outcome, report, exit class, or manifest moves; the wrapper ships in no package.

- **A tag read that declines is not a snapshot that was never tagged.** The publish gate's tag check was
  `git rev-parse --verify` plus `.is_err()`, so a directory that is no repository, a ref store it could not
  read, and an absent tag were one fact — reported as *there is no tag*, **as a violation**, in front of an
  upload that can be yanked and never replaced.

  `publish-source-integrity` states the rule over the class: every git read whose answer is an exit status
  reads the status that IS the answer and treats every other non-zero as a refusal to answer. The spec
  records that it was generalized from `check-ignore` to the class because it arrived through a second door,
  `ls-files --error-unmatch`. This was the third, and the machinery for it already existed —
  `hermetic_git::Failure::Exit` carries the status, and its own doc records the measurement that put it
  there. `TagPresence` is `Tracked`'s sibling, built from the same split.

  **`--quiet` is what makes the split exist**, and it is not cosmetic: measured, a bare `rev-parse --verify`
  exits `128` for an absent ref *and* for a directory that is no repository, so the answer and the refusal
  are one status until it is passed. With it, an unresolvable ref exits `1` and git keeps `128` for declining
  — the contract `ls-files --error-unmatch` already had, and the reason its reader could be split at all.

  The control is built without a commit: `hermetic` closes the config that would carry an identity, so the
  fixture tags a written blob instead. A tag ref may name any object and what this reads is whether the ref
  RESOLVES, so the object's kind is not the subject.

  **What stays unsplit is filed rather than declared.** A ref file holding garbage exits `1` exactly as an
  absent ref does, so *missing* and *corrupt* are one status at this layer; `BACKLOG.md` carries it as a
  `WATCH` with the commands that measured it. It is deliberately **not** an observation bound: a bound is
  pinned by a direction over its own WHEN, and this WHEN produces the same answer as the case beside it, so
  the pin would compare a value with itself. A neighbouring case is separated and recorded so the two are not
  confused — a ref holding a well-formed sha with no object exits `0`, reads as present, and the tag-object
  read downstream refuses it as unreadable.

  Negative run: with the classifier reverted to *every failure is absent*, the refusal direction reports
  `Absent` for a directory git declined to read, while the control and the sibling `tracks` direction both
  pass unchanged in the same run.

  No published API, outcome, report, exit class, or manifest moves; the gate ships in no package.

- **The ambient-ignore guard reaches staging, two fixtures stop bypassing it, and the table it enforces was
  wrong in two rows.** `AMBIENT_IGNORE_READS` held four markers and every one was a *query* — `check-ignore`,
  `--others`, `--untracked-files`. The source of truth names the other direction **first**: `hermetic_git`'s
  ambient table records `git add -A` with the three file variables set leaving the matching file untracked,
  *a fixture silently built without a file it named*. Querying misreports; staging **corrupts the subject**,
  and the guard could not see it.

  Adding `"add"` reported exactly the two files a review had named: `bound_register.rs` and
  `workspace_isolation.rs`, both spawning a bare `Command::new("git")` with neither `hermetic` nor the
  neutraliser. Both are routed through `hermetic_git::{fixture, run}` now — and `bound_register`'s block stood
  **byte-identical twice**, so it converged into one helper on the way through. `workspace_isolation`'s third
  git site, an `ls-files` enumeration, is not in that channel and went through the shared reader anyway: one
  file spawning git two ways is the twin this crate keeps converging, and the builder's failure type already
  separates *git could not run* from *git ran and refused*, which the hand-rolled form folded into one
  sentence.

  **Two table rows were wrong, both measured rather than reasoned.** `GIT_CONFIG_*` was recorded as **open** —
  *any key reaches git* — and is closed: `Command::env` overrides `GIT_CONFIG_COUNT` to `1`, git reads index
  `0` only, and this builder owns index `0`. Measured with `GIT_CONFIG_COUNT=2` and
  `GIT_CONFIG_KEY_1=user.name` ambient: under the builder `git config --get user.name` exits `1` with no
  output, and the same pair without it answers the ambient value. A row claiming a channel is open is not a
  conservative error — it reads as governed policy and would send the next fixture author to build isolation
  they already have. And `GIT_DIR` / `GIT_WORK_TREE` / `GIT_INDEX_FILE` were **missing**: they are not an
  ignore channel, they move which repository git acts on, reaching past `current_dir` entirely. Measured, with
  `GIT_DIR` naming another repository, `rev-parse --git-dir` under the builder answers that other repository.
  Nothing in this tree sets them, so it is a stated limit rather than a live defect — stated because the
  table's own column is *ambient source* and a fixture author reads it as the set.

  The correction invalidated the sentence under it, which said that channel *cannot be closed*. Occupying
  index `0` is what closes both rows at once: the setting reaches git from here, and the count this builder
  writes is what makes an ambient key unreachable. The channel is **used**, not open.

  The new direction asserts the **construction** — the count is written and index `0` is taken — rather than
  simulating an ambient environment, and says why: `Command::env` *overrides* the inherited value, so a
  direction setting `GIT_CONFIG_COUNT=2` on the command overrides the builder rather than standing in for
  ambient. The first draft did exactly that and failed, which is the trap and not a finding; constructing the
  real case needs the test process's own environment mutated or a child to carry it. `EXCLUDES_SETTING` gives
  the setting name one owner, which that direction needs for a second reason: a file that *spells* the setting
  is read by this very guard as having closed the channel itself, and the guard's control-file assertion
  caught that on the way in.

- **One enumerator for the capability specs, which is the third of three and the one the extraction left
  behind.** `observation_bound_model` had already converged the marker predicate and the slug rule into
  `bound_register_parse` and imported both — and kept enumerating the corpus itself, with
  `std::fs::read_dir(openspec/specs)` over the **worktree**, while the register enumerated the same set from
  `git ls-files -z` over **tracked content**, where `-z` is load-bearing because a quoted path would silently
  drop a whole capability's bounds.

  Two enumerators of one set, in one crate, with nothing comparing them. The consequence is not abstract: an
  untracked or gitignored `spec.md` entered the bijection gate and could never enter the register's, so the
  two gates judged different corpora and neither could say so. Measured, with an untracked
  probe capability's `spec.md` planted in the worktree and the walk restored — the bijection reported it as
  *declared in a spec and classified nowhere* while `bound_register` stayed green, both corpora visible in one
  run.

  `spec_bounds` calls `tracked_specs` now, and the duplicate vacuity assert goes with the walk: `tracked_specs`
  refuses its own empty enumeration, so the second one was a guard a call away from itself. The *other* assert
  stays and is a different fact — no declared bound found is not no capability spec found.

  `openspec/specs/observation-bound-model/spec.md` already forbade a second implementation of the predicate and
  the slug rule, and stopped exactly where this defect was; the corpus is named there now as the third, with
  the scenario. The pin asserts the corpus **by identity** rather than by re-deriving a set to compare against
  — a comparison would need the second walk this removes.

  No published API, outcome, report, exit class, or manifest moves; every site is in a crate that ships in no
  package.

- **An attribution mark ends where it ends, and the two line marks end differently.** `carries` matched a
  trailer with `starts_with`, so a line beginning `Co-authored-bystander …` was refused while carrying no
  attribution at all. That is a **false refusal**, and it is the very thing the line-start rule was written to
  prevent — *this gate would otherwise refuse the commit message of any change about this rule* — reached from
  the other end.

  The two line marks are not the same shape and lumping them is what let the prefix run on. `co-authored-by`
  is a trailer **key** and ends at its `:`; `generated with` is a footer **phrase** with no colon at all, so
  demanding one would stop refusing the real mark, and it ends at a word boundary instead. `Shape::Trailer`
  splits into `TrailerKey` and `Footer`, each carrying its own boundary and its own refusal sentence — the
  array already carried the recognizer beside the mark, and now it carries the right two.

  **A review reported this as the gate failing to establish a `Key: Value` shape, and that part is refuted
  rather than adopted.** `repository-checks` asks for case-insensitive recognition *at the start of a line* and
  states no shape requirement; and one of the marks is not a `Key: Value` — the requirement's own word for it
  is *footer*. What was wrong is the boundary, not the shape, and citing a requirement that does not exist
  would have justified the repair on a premise the spec contradicts.

  Direction and controls run together: the three admitted lines carry no attribution, and the three refused
  ones are the marks in the forms that actually appear — including `co-authored-by : …`, since git's own
  trailer reader accepts a space before the colon. Without the controls the direction would hold for a gate
  that refuses nothing. Negative run, with the boundary reverted and the two shapes kept so it still compiles:
  the `Co-authored-bystander` line is refused.

  No published API, outcome, report, exit class, or manifest moves; the gate ships in no package.

## [0.4.0] - 2026-08-04

### Documentation
- Retired a `BACKLOG.md` WATCH line that had survived its own promotion. It asked whether
  `xingbiao::crate_root_file` collapses a multi-root package, with the trigger "confirm `cargo metadata`
  actually emits multiple root files for one package before treating this as more than speculative" —
  which was confirmed in this window, promoted the item into the DESIGN-BREAKING entry, and closed with
  the per-target corpus, while the WATCH line stayed. An index that carries a question and its answer at
  once is a reader trap, so the lesson is recorded with it: retire the WATCH line in the same change that
  promotes it.
- `structured-violation-identity` now derives identity completeness rather than leaving it to review: a
  fact carries every coordinate of **where** the observation was made that can vary — the governing
  declaration, the compilation unit, the module, the owner or item, and the position-free discriminator
  within that item — and carries none that cannot. Every identity collision this system has had was a
  missing coordinate found one adversarial review at a time, seven of them in this window alone. The
  requirement also records why pre-emptive schema widening is not the answer: an identity is its *values*,
  measured on the real type (no field ≠ an empty field ≠ a real value), so declaring a field early re-keys
  every baseline once for the field and again when the value arrives. Each dimension's
  published-identity-schema test is the enforcement point, and it earned that immediately — adding the
  assertion found 渾儀's unsafe-site fact carrying no compilation unit, where `crate::m` in a library and
  `crate::m` in the `bin` beside it would otherwise have shared one identity.
- Swept every repository document for drift against the code, mechanically rather than by reading: each
  referenced repo path exists, each referenced script exists, each cross-referenced capability has a
  spec directory, `PROJECT.md`'s architecture section describes all six crates, and every crate carries
  a README. Two items were stale and are fixed — `BACKLOG.md` claimed an "18-test `baseline_cli` suite"
  (it now holds twenty, and the count carried no argument, so it is dropped rather than corrected to a
  number that will drift again), and it named a reproduction test without saying the closure had since
  removed it, leaving a reader to grep for a symbol that is gone. No behaviour changes.
- Stated 漏刻's legacy **directory** corpus bound and retired the WATCH hypothesis that guessed at it. A
  probe behind a symlinked subdirectory is seen when the audit is given a **target root file** (the
  module graph reaches it; reading a file follows symlinks) and unseen when it is given a **directory**
  (the walk classifies entries with `file_type()`, which does not, so a symlinked directory is not
  recognized as one — deliberately, since following it admits an unbounded walk on a cyclic symlink).
  Measured on one fixture and pinned in both directions rather than described. The hypothesis it replaces
  guessed a bypassed cycle guard and did not distinguish the two corpora, which is where the whole answer
  lives: the input the 天衡 shell actually passes has no gap, and the directory input exists for source
  compatibility. Also retired a stale `ACCEPTED DEBT` line — "file-granular un-auditable-probe identity"
  — which the 0.4.0 owner-qualified identity had already superseded, so it recorded a bound that no longer
  exists. No behaviour changes.
- Corrected a coverage claim three surfaces made and the code never had: `module-boundary` stated that
  a lib+bin package's "both crate roots (`lib.rs` and `main.rs`) resolve to `crate`", `module_check.rs`
  repeated it to justify an inbound dedup step, and a unit test named for it could not have caught the
  premise being wrong — its synthetic metadata declared only one target, so "two roots deduplicated" and
  "one root scanned" produced the identical assertion (the outcome-unaltered trap `AGENTS.md` names).
  Measured through three lenses: `xingbiao::crate_root_file` returns one root by construction; a minimal
  `src/lib.rs` + `src/main.rs` package with the same offending construct in both reports one violation,
  in `lib.rs`; and `src/bin/*.rs`, a `[[bin]] path` inside `src/`, and one outside it are all unobserved
  when a library root exists. What survived of this entry is the *correction of a false claim*: three
  surfaces asserted coverage the code did not have, and the measurement is what established the real
  scope — the ONE resolved crate root and the modules reachable from it — as a recorded bound rather
  than a silent pass. **That scope did not survive this window.** It was promoted in `BACKLOG.md` to
  DESIGN-BREAKING with per-target module graphs as the trigger, and the trigger fired inside the same
  window: see "圭表 and 渾儀 now govern **every** compiled root of a package" under *Fixed*, which is the
  state 0.4.0 ships. The tests written here as a bound in both directions (real manifest, real `cargo
  metadata`) are now `crates/guibiao/tests/per_target_corpus.rs`, inverted rather than deleted, since
  the transition they were written to detect is exactly what happened. This entry is kept for the
  false-claim correction and the measurements; it is not the shipped scope, and the note is here so a
  reader meeting it first does not take it for one.
- Retired the last of the window's stale version and provenance claims, and completed the Migration
  collection. Eight sites attributed shipped behaviour to a release **0.3.1** that has no tag and no
  CHANGELOG section — it was the *branch* `release/0.3.1`, renamed to `release/0.4.0` — so fixes an adopter
  reads as already released are in fact shipping now; they name 0.4.0. The sweep-provenance mentions
  (`0.3.1 sweep`, `0.3.1 audit trigger`) are left alone: those name where a finding came from, which is
  accurate, and `BACKLOG.md` spells the range branch-qualified. Four adopter-facing self-descriptions still
  named `0.3.0` as the current line, one of them actively misleading ("0.3.0 spends a deliberate breaking
  window on reaction identity" — that is this window). Two were not load-bearing and now name no version at
  all, following the rule applied to `.gitignore` and `scripts/test_examples.sh`; two are compatibility
  statements and moved to 0.4.0. The Migration section drops another drifting count and gains the steps it
  omitted: new entries from causes outside the inbound `Shallow` cell (a second compiled root, an outbound
  rule's second importing module), and the shapes that now refuse to judge (exit 2) where they previously
  answered — enumerated in that section rather than counted here, since counting them is what kept going
  stale. Also restores a doc comment this window displaced: `declaration_text` was
  inserted between `strip_macro_bodies` and its doc, so the macro-body documentation described the wrong
  function — and described it self-contradictorily, since the text says "runs on already
  comment/string-stripped text" while `declaration_text` performs that stripping itself. `cargo doc -D
  warnings` cannot see this: it is a misattached comment, not a broken link.
- Swept the supersession this window's own reversals left behind, and moved closed work out of the live
  decision queue. `[Unreleased]` had carried two entries describing **incompatible** states of one
  capability — a *Documentation* entry establishing "the ONE resolved crate root" as the stated
  requirement, and a *Fixed* entry 150 lines later reporting that every compiled root is governed — so an
  adopter met the superseded scope first and as fact. The Documentation entry now keeps what survived of
  it (a false coverage claim corrected, and the measurements that did it) and says plainly that its scope
  did not survive the window, pointing at the entry that ships. `BACKLOG.md`'s six closed
  DESIGN-BREAKING items no longer sit struck-through inside that class heading while still carrying
  present-tense `Class:` / `Risk:` / `Promotion trigger:` lines: their reproduction records move to a
  *Closed in the 0.4.0 window* section, the class heading is now honestly empty, and the governance
  vocabulary states the rule so the next closure does not repeat it. Three refuted references are
  corrected — a test file renamed by this window's own reversal, a regression test that was replaced by
  its inverse, and a claim that two doc comments pin a bound they now contradict. `scripts/test_examples.sh`
  stops naming the family version in prose, the fix already applied to `.gitignore`'s comment for the same
  reason.
- Made "a guard is not a guard until it has been seen to fail" an explicit rule in `AGENTS.md`'s
  adversarial-review stance, and required the negative run per new guard in the PR body's
  `## Verification`. The trap it names is the change whose *outcome* is unaltered: when a fix improves
  a diagnostic while the exit code stays identical, a test bound to the exit code passes equally
  before and after, pinning the surrounding contract rather than the change. That was hit twice in
  this window — once as a conclusion that no test was possible, once as a test that stayed green with
  its rule removed — so the discipline is written down rather than left to be re-derived. A test kept
  for the contract rather than the change now earns a comment saying which it is.
- Recorded the baseline directory flush's absence of a reacting test as `ACCEPTED DEBT` in
  `BACKLOG.md`, with its measurement (`cargo mutants` reports both of its mutants MISSED, while the
  rules beside it are caught) and the reason the alternatives are worse. It is unobservable by
  construction — an infallible best-effort step has no behavior to bind — so the bound is stated
  instead of implied by a green suite.
- Stopped tracking `examples/capability-catalog/Cargo.lock`, the only committed example lockfile.
  `.gitignore` had matched it since the examples were introduced — tracking simply overrode the rule —
  and the rule's own comment stated that the examples "carry no lock". It pinned all six family
  crates to the published 0.3.0, so an example run without the examples gate's local-source patch
  demonstrated a release other than the tree it was cloned from, and no gate could notice because
  the gate patches resolution to local source. The same comment also still described the examples'
  dependency form as `= "0.1"` when all six commit `= "0.3"`; it now points at each example's own
  manifest instead of naming the version a second place.
- Replaced the root manifest's per-package `exclude` list with the two directory prefixes that
  contain them (`crates/tianheng/tests/fixtures`, `examples`). The list had drifted to 3 of 5
  fixtures and 3 of 6 examples while its comments claimed to cover all of both. No membership change
  (`cargo metadata` reports the same six members): what actually keeps a fixture's deliberate faults
  out of this workspace is each one's own `[workspace]` table, with `members` an explicit
  glob-free list. The exclusion is the second line of that defence, for a future fixture or example
  added without its own `[workspace]` — verified load-bearing by adding one and observing cargo
  operate inside it when excluded and refuse when not — and it now covers all eleven rather than six,
  as a prefix that cannot fall behind what it contains.
- Derived the branch role from the Conventional Commit type instead of an enumerated prefix list.
  `AGENTS.md` listed `refactor/` and `docs/` but not `fix/` or `test/`, both long-established, and
  declared a `polish/X.Y.Z/<slug>` role no release has ever used — the contributor rule most likely
  to be read first was the one least matching practice. Outside `change/` and `release/`, a branch is
  now `<type>/<scope>-<slug>` for the type its work lands as, so branch role and squash subject cannot
  disagree and the rule cannot rot the way a blessed-prefix list did. Pre-release polish takes the
  type its own work lands as; the unused release-staging role is gone.
- Ignored `.github/skills/`, openspec's per-clone generated skills directory, alongside its
  already-ignored `.github/prompts/` sibling.
- Specified 圭表's plain-`mod` conventional-file resolution outcomes in `module-boundary`, which had
  shipped and been tested since 0.2.3 with no requirement of their own: both forms present is an
  ambiguity constitution error (ahead of the absent-file tolerance, so a `#[cfg]`-gated-off
  declaration still reacts even though the crate compiles), an unconditionally absent file is a
  constitution error, and a bare `#[cfg]` tolerates absence. No behavior change — the requirement
  truth catches up to the reaction. (A `#[cfg_attr]` wrapper's own tolerance is specified by the
  fix below in this same window, so it is stated there instead of restated here.)

### Added

New public surface, enumerated by diffing each crate's public items between `v0.3.0` and this branch
rather than by recalling what was added — two of these had reached the branch inside a commit typed
`docs`, so no narrative entry named them.

- `xingbiao::path_label` — a path as a canonical identity label: `/` as its only component separator,
  every byte preserved. The one answer both label sites now share; see *Fixed*.
- `xingbiao::crate_root_files` — every compiled crate root of one package, so a dimension can govern each
  as its own corpus.
- `xingbiao::compilation_unit_label` — a compilation unit's stable identity label, its root path relative
  to the package's own manifest directory.
- `xingbiao::workspace_root` — the workspace root Cargo resolved, read for its stability as an anchor to
  label observed files against.
- `guibiao::ModuleBoundary::reason()` and `ModuleBoundary::module()` — the declared reason and governed
  module, readable from a boundary rather than only from a violation.
- `guibiao::CrateTarget::as_str()`, and a `crate_package()` accessor on each rule-DSL boundary type
  (generated by the shared `boundary_common!` macro rather than hand-repeated eight times).
- `louke::dropped_sink_events() -> u64` — the count of events the default sink dropped, a single
  lock-free atomic read.
- `xuanji::Violation::is_active_enforce()` — whether a violation is an unbaselined `Enforce`, the
  gating predicate the projections previously each spelled out.

`louke::OriginEntry::__from_register_origin` is also new and is deliberately **not** adopter-facing API:
it is `#[doc(hidden)]`, exists only as `register_origin!`'s expansion target, and is public solely
because a macro's expansion runs in the caller's crate. It replaces `OriginEntry::new`, which is removed
— both recorded under *Fixed*, where the forgery gap it closes is described.

### Changed
- **BREAKING**: renamed 渾儀's `SemanticBoundary` (the signature-coupling DSL's boundary type,
  `dsl/signature.rs`) to `SignatureBoundary`, along with its draft chain
  (`SemanticCrateDraft`/`SemanticModuleDraft`/`SemanticBoundaryDraft` →
  `SignatureCrateDraft`/`SignatureModuleDraft`/`SignatureBoundaryDraft`). `SemanticBoundary` read as
  if it were the DSL's umbrella type, unlike its 7 siblings (`AsyncExposureBoundary`,
  `DynTraitBoundary`, `ForbiddenMarkerBoundary`, `ImplTraitBoundary`, `TraitImplBoundary`,
  `UnsafeBoundary`, `VisibilityBoundary`), which all name their own capability. No behavior, rule
  string, JSON wire, or CLI change — only the Rust type names an adopter's `Constitution`
  construction code references. `SemanticBoundaries` (the per-dimension aggregate struct
  `hunyi::SemanticBoundaries` holding one `Vec` per capability) and every `semantic_*` dimension
  label are unrelated and unchanged.
- Test-only, no production code change: 渾儀's `every_public_seam_shape_is_named_and_identity_injective`
  now derives its coverage check from an exhaustive `seam_kind` match over `PublicSeam` instead of
  comparing the hand-written fixture's length against itself. `published_seam_fields` and
  `assert_semantic_fact_is_cataloged` already forced a new `PublicSeam` variant to gain a schema arm
  (a compile error otherwise), but nothing forced an *instance* of it into the fixture — the old
  `keys.len() == seams.len()` check would have stayed green even with a variant silently unrepresented.
  `seam_kind`'s own exhaustive match now fails to compile on a new variant too, and the fixture's
  distinct-kind count is asserted against it directly. Coverage was already complete (all 11 kinds
  present); only the enforcement was hand-maintained.
- Test-only follow-up, no production code change: that same coverage check no longer rests on a
  hand-maintained integer. `PUBLIC_SEAM_KIND_COUNT: usize = 11` sat beside the `seam_kind` mapping
  while the fixture it described sat a hundred lines below, and the compiler forced only the match
  arm — so adding a `PublicSeam` variant and its arm while forgetting both the integer and the
  fixture representative left the check green with the new shape uncovered, the same
  hand-maintained-enforcement gap the entry above set out to close, one link further along.
  `seam_kind` now returns a closed `SeamKind` enum whose shapes are compared **as a set** against
  `SeamKind::ALL`, so a missing representative fails by name rather than by two integers differing,
  and the shape-to-published-label mapping is asserted to be a bijection against
  `published_seam_fields` — production schema truth — so a new variant folded into an existing shape
  cannot read as already covered. That bijection is checked in **both** directions, because neither
  count catches the other's failure: with every shape represented, the distinct (shape, label) pair
  count rises above the shape count only when one shape is published under two labels, while two
  shapes *sharing* a label leaves it untouched and is caught by the distinct label count instead.
  Verified by three probes: adding a twelfth `PublicSeam` variant (the compiler demands arms at four
  sites, and with those satisfied but the fixture entry omitted the check fails naming the shape,
  where the previous version passed), publishing one shape under two labels, and publishing two
  shapes under one label — each failing on its own assertion.
- Test-only, no production code change: 天衡's two baseline temp-path guards no longer rest on
  winning a race. Both plant an obstruction (a stale temp file, a symlink to a victim) at the
  `<target>.tmp-<pid>` path a spawned CLI run will predict — which the parent can only compute after
  `spawn`, so a loaded runner can let the child open its own temp file first and never reach the
  collision. Neither guard could tell that had happened from its own assertions: the stale-temp one
  demanded exit 2 and failed spuriously (observed once in CI), while the symlink one passed
  **vacuously**, since an untouched victim and a non-symlinked baseline are exactly what an
  unexercised run leaves behind too. Both now go through one helper that re-races until the run's
  refusal names the planted path, and fails loud naming the attempt count if it never does — so a
  verdict is only ever reported by a run that earned it. Verified by injecting the lost race
  deliberately: the guards still pass through three lost races, and the helper fails with its own
  "never exercised" message when the plant can never land.
- Internal refactor: 渾儀's three call sites that compose transparent-macro flattening with
  const/fn-body-nested-impl recovery (`scan::flatten_for_walk`,
  `module_resolve::resolve_module_items_with_files`, `module_resolve::resolve_module_items_with_cfg_tags`)
  now share one crate-private helper, `syn_util::flatten_with_body_nested_impls`, instead of each
  hand-composing the identical sequence. No public API, wire format, or observable behavior change.
- Internal refactor: 渾儀's four call sites that guard a forbidden-operand list against a malformed
  `::`-path entry (`exposure.rs`, `forbidden_marker.rs`, `shape_scan.rs`, `impl_trait.rs`) now share
  one crate-private helper, `resolve::validate_path_operands`, instead of each repeating the
  identical inline check. No public API, wire format, or observable behavior change at these four
  sites.
- Internal refactor, no public API/wire/behavior change: a large structural-clarity and
  deduplication pass across 圭表, 渾儀, 漏刻, 星表, and 璇璣 — splitting oversized functions
  (`module_check::check_module_boundary`, `module_scan/lexer.rs`'s comment/string skipper,
  `module_resolve::descend`, `scan::resolve_child_modules`/`walk_module`,
  `exposure::module_findings`, `forbidden_marker_findings`, `finding::into_finding_with_text`,
  `audit/scan.rs`'s `collect_scope_modules`/`fn_scopes`, `audit::audit_probe_coverage_with_markers`,
  `runner::dispatch`) and factoring repeated shapes into shared helpers (a `boundary_common!` macro
  for the 8 rule-DSL files' identical accessors, a shared `ViolationContext`/`push_violation`, a
  `CapabilitySet` trait replacing 3 independent per-capability enumerations, `delimiter_group_end`
  for 3 near-identical balanced-delimiter scanners, `Violation::is_active_enforce`, and several
  smaller extractions). No test count changed in any crate.
- Internal refactor: 圭表's `cargo_metadata.rs::matching_dependency_edges` now delegates to
  `governed_dependencies(package, kind, true)` plus its own name filter, instead of hand-repeating
  the identical `kind_matches`/`!is_self_dependency` conjunction `governed_dependencies` already
  encapsulates. No public API, wire format, or observable behavior change.
- Internal refactor: 渾儀's `file_scope.rs` drops `resolve_crate`, the single-root preamble
  `resolve_crate_units` replaced at all of its call sites earlier in this window. It had been kept
  compiling by an `#[allow(dead_code)]` — the only one in non-test product source — which suppressed
  the one signal that would have reported it; with the attribute removed the compiler names it
  directly (`function resolve_crate is never used`). Its removal takes three stale surfaces with it:
  the module header named it as the preamble every `check_*_boundary` opens with, and the doc comment
  above `resolve_crate_units` still opened with `resolve_crate`'s own text, so the first sentence a
  reader met described a `(package, crate-root file, source dir)` return that function does not have.
  The two sentences of that comment still true of the surviving function — one home for the
  constitution errors resolution can raise, and each `src_dir` owned rather than borrowing its root
  file — are kept and corrected to name all three errors, the out-of-package-root case having been
  added since. The `CompilationUnit` alias also moves below the imports it was inserted between. No
  public API, wire format, or observable behavior change.
- Internal refactor: 渾儀's `resolve_direct_path_child`/`resolve_conventional_child`
  (`module_resolve.rs`) now share `load_child_file` for the canonicalize → descent-path
  cycle-check → crate-wide dedup-guard → `read_parse` sequence 3 near-identical call sites
  repeated; each caller keeps its own distinct child-directory/tuple-assembly logic, which
  genuinely differs per call site. No public API, wire format, or observable behavior change.
- Internal refactor: 漏刻's `audit/scan.rs::collect_directory_probes` now reads and scans a source
  file through a new `scan_rust_file` helper instead of an inline `read_to_string` call in its
  recursive dispatch loop — the same I/O-isolation shape `read_dir_entries_sorted` already applies
  to directory listing, one level deeper. `scan_rust_file` also dedupes an identical read+scan
  sequence in `collect_reachable_probes` (the file-input mode's reachable-module walker), which
  needs the read source text back afterward to walk the file's own further module references, so
  the helper returns it rather than discarding it. No public API, wire format, or observable
  behavior change.

### Fixed
- **BREAKING**: 圭表 and 渾儀 now govern **every** compiled root of a package — a `main.rs` beside a
  `lib.rs`, any `src/bin/*.rs`, any `[[bin]] path` — each as its own module graph, and every module and
  semantic fact carries the **compilation unit** it came from as an identity role (the root's source path
  relative to the package's own directory). Previously only the first library-kind target, else the first
  `bin`, was observed, so a violation written in any other root of the same package passed silently — the
  forbidden false negative, on the most ordinary Rust package shape. 漏刻 already governed every root, so
  the three dimensions now agree on which of a package's source Cargo actually compiles.
  The identity role is what keeps the fix from trading one defect for another: every root denotes the
  module path `crate` and shares the package name, so without it the same violation in two roots would
  carry one identity and a baseline accepting it in one root would silently mask the other's.
  A target's **name** is not that role — a package may build a library and a `bin` of the same name, as
  this repository does — and a root whose path lies outside the package's own directory is a **constitution
  error** (exit 2) rather than being labeled by the checkout's location, which would make the identity
  differ between two clones of one commit. That refusal is narrow: a root inside the package but outside
  its source directory is governed normally.
  Also **BREAKING**: an outbound rule's finding now carries its **importing module**. Two different modules
  of a governed subtree importing the same forbidden path used to collapse into one finding, so accepting
  it in a baseline masked the other; the inbound rules had always qualified by importer, and the two
  families are now symmetric. The dedup key is the (importing module, import path) pair.
  **Migration**: every module and semantic baseline entry re-keys, absorbed by the single regeneration this
  release already requires. Expect **new** entries too, not only relabeled ones: a violation in a second
  crate root, or in a second module importing an already-recorded path, was never reported before.
- **BREAKING**: 漏刻 no longer relativizes the identity label of a file reached through an **absolute**
  `#[path = "/…"]` literal — it keeps the path the literal wrote, in every checkout. This closes the last
  open identity gap in the window. Relativizing it *was* the gap: prefix-stripping succeeds by pure text
  match wherever such a target happens to be nested under a given checkout's anchor and fails everywhere
  else, so one identical committed literal produced a relative-looking label in one clone and the raw
  absolute path in another — a baseline recorded in one going stale and re-firing as new in the other, on
  a commit that changed nothing. An absolute literal does not move with the checkout, so its label no
  longer does either, and "does this target happen to sit under this anchor" no longer reaches the
  identity. The property is inherited by the files that target reaches in turn, since they resolve from
  its own directory and the same coincidence applies to them. Every other file — a conventional child, a
  relative `#[path]`, the legacy directory walk — is labeled relative to the anchor exactly as before,
  which is what keeps the realistic sibling-share idiom checkout-independent. **The "last open identity
  gap" claim in this entry was premature**: inheritance was threaded down the file chain only, and an
  inline `mod`'s own absolute `#[path]` base introduces the same non-portability *within* a file — closed
  separately below.

  The recorded promotion trigger had described this as threading a fact through four functions, which
  read as a broad refactor and is why it stayed open; it is not, because `Path::join` discards its
  receiver *exactly when* the joinee is absolute, so the fact is knowable at the one line that resolves
  the literal.
  **Migration**: an `unauditable-probe` baseline entry whose `file` was a relative-looking label produced
  from an absolute `#[path]` literal re-keys once — regenerate with `--write-baseline`. No other entry is
  affected, since no other labeling changed.
- 渾儀 now follows a `#[cfg_attr(pred, path = "dir")]` remap on an **inline** `mod x { mod y; }` to the
  base its file-form children resolve from, the same rule 圭表 gained just above and 漏刻's own
  specification already stated. Before, 渾儀 reported a missing-module constitution error (exit 2) on
  source that compiles cleanly, while its own **file-form** resolution followed the identical
  attribute — an inconsistency inside one crate, and the third dimension diverging on a rule the other
  two now share. Two of 渾儀's walkers needed it (the item scan that loads a child's contents and the
  branch walk that resolves a boundary's target module), and each was proven load-bearing by reverting
  it alone and watching the conformance case fail. Found by adding the shape to
  `cfg_attr_path_only_module_conformance.rs` — the cross-dimension ledger that exists for exactly this
  divergence — which the 圭表 fix had not done, closing the instance while leaving the class open. That
  suite now feeds this shape to all three dimensions' real entry points, so the next drift of it is a
  failing test rather than an adopter's exit 2.
- 圭表 now follows a `#[cfg_attr(pred, path = "dir")]` remap on an **inline** `mod x { mod y; }` to the
  base directory that body's file-form children resolve from. Previously only the *unconditional*
  `#[path]` form was followed, so the walk looked for the conventional `src/x/y.rs` and reported a
  missing-module constitution error (exit 2) on a crate that compiles cleanly under real rustc — 圭表
  refused to judge the crate rather than judging it, and an adopter using the idiom could not run
  `check` on it at all. Reproduced against the real entry point with the unconditional form as the
  control before the fix. Every `cfg_attr` target is now a **candidate** base unioned with the
  conventional directory, cfg-blind (the scanner does not evaluate `cfg`, so preferring one would drop
  every child beneath the other); a candidate is descended only when it exists as a directory, and when
  none does the conventional base is descended anyway, so a child reference broken on every platform
  still fails loud. This is 漏刻's own already-stated rule for the identical shape, implemented
  independently (三儀 ⊥ 三儀). `module-boundary` gains the requirement and three scenarios.
  Not breaking: no public API, violation identity, or baseline shape changes — but a crate that
  previously exited 2 is now judged, so its first run may report violations that were never reported
  before rather than relabeled ones. This closes a `BACKLOG.md` WATCH item whose recorded risk class
  (a false negative) the reproduction **refuted**: the behaviour was fail-loud throughout.
- **BREAKING**: `--write-baseline` now records a fresh snapshot over a **zero-length** existing
  baseline instead of refusing it (exit 2 → exit 0 for that input, reporting what it found). The
  refusal exists to stop an overwrite from destroying hand-authored owner/tracker annotations, which
  no rerun can reconstruct — and zero bytes cannot hold any, so it protected nothing while telling
  the adopter to "preserve any desired annotations" that were not there and to move a file by hand to
  recover. It is also precisely the shape an interrupted create leaves, since the create path
  publishes its directory entry before its first byte, so the one state a crash can produce was the
  one needing manual repair. The recovery is announced on stderr rather than silent. Deliberately
  bounded to *zero* length: whitespace-only and truncated content might have held annotations before
  being damaged, and stay refused, byte-for-byte untouched — pinned by the same test. Gate mode
  (`--baseline`) does **not** share the tolerance: a declared baseline it cannot parse remains exit 2,
  because recording may regenerate a snapshot it owns while gating consumes a declaration the adopter
  wrote, and reading a corrupt one as "nothing is accepted" would silently discard their
  accepted-violation record. `violation-baseline` gains the exception and three scenarios.
- A baseline write now flushes its bytes to stable storage before reporting success, closing the one
  gap between the overwrite path's documented crash guarantee and what it implemented. Temp-then-
  rename made the swap atomic *for other observers*, but `rename` orders only the directory entry —
  never the temp file's still-dirty data pages — so a crash shortly after a successful rename could
  leave the baseline path present and **empty**, losing both the previous document and the
  owner/tracker annotations just merged into it, which no rerun can reconstruct. The overwrite path
  now fsyncs the staged temp file before the rename, and the create path fsyncs its file before
  reporting the write. ext4's `auto_da_alloc` heuristic happens to mask this for the
  replace-via-rename pattern, but it is disabled by `noauto_da_alloc` and absent on other
  filesystems, and this crate ships to adopters on filesystems it does not choose. Each path
  additionally *attempts* to flush the containing directory, so the published name survives a crash
  too — but that attempt is best-effort and never fails the write: it strengthens a write that has
  already landed, and it can be unavailable for capability reasons rather than storage faults (a
  directory that is writable but not readable answers `EACCES` to the open; some FUSE and network
  mounts answer `EINVAL`/`ENOSYS` to the fsync), where reporting "cannot write baseline" for a
  baseline sitting correctly on disk would be the worse outcome. The strict guarantee is therefore
  the file flush; the directory flush is unix-only besides (`std` exposes no portable way to open a
  directory handle on Windows). `create_baseline_file`'s own doc no longer claims a crash there
  "simply leaves no file": it publishes its directory entry before its first byte, so a crash
  mid-create can leave a file with no bytes in it, or with some — and the two are not the same state.
  The zero-length exception recorded above in this same window is what separates them: an **empty**
  residue is recorded afresh on the next run (exit 0, announced), since zero bytes hold none of the
  annotations the refusal protects, while a **partial** or whitespace-only residue stays refused
  (exit 2, remedy named), because it may have held annotations before being damaged. This entry's
  first cut said the next run refuses the empty file too — true when it was written, superseded within
  the window by that exception, and corrected here so the two entries read as one behavior rather than
  two. `violation-baseline` gains the requirement and its three scenarios,
  and the tolerance is pinned by a test that first proves its own
  precondition — a directory that is genuinely unreadable to the running process — so it reports a
  vacuous run instead of passing through one.
- A value-taking flag (`--manifest-path`, `--baseline`, `--write-baseline`, `--format`) whose next
  argument is itself a `--`-prefixed flag is now a usage error that exits 2 and names the token
  found, instead of consuming that flag as its value. The absent-value case already failed loud; the
  value-is-a-flag case silently ate the following flag. For `--write-baseline` that reached a silent
  *success*: `check --manifest-path <ws> --write-baseline --warn-uncovered` wrote a baseline file
  literally named `--warn-uncovered` into the working directory and exited **0**, with
  `--warn-uncovered` dropped and no diagnostic — a misconfiguration passing as clean, which
  PROJECT.md forbids. The other three flags did reach a non-zero exit, but reported it as a
  downstream unreadable path or unknown format rather than as the malformed invocation it was. The
  `--flag=<value>` form is unchanged and remains the way to pass a value that legitimately begins
  with `--`, since it carries its value in the same token; that escape hatch is pinned end-to-end,
  distinguishing the two forms by their diagnostics rather than by an exit code they share.
  An **empty** value is now that same usage error in *both* forms: `--baseline=` and `--baseline ""`
  are the same mistake as `--baseline` with nothing after it, and used to be carried onward — an
  empty path answers `NotFound` at the filesystem, so a malformed invocation was reported as
  `cannot read baseline ` against a path nobody typed, complete with the dangling space where the
  path would be. The exit code was already 2 in every one of these cases, so what changes is which
  mistake the diagnostic names; the equals form is the only way to reach an empty value for a path
  flag, so the rule is shared by both forms rather than living in the space form alone.
  `cli-check-runner` gains the requirement and its four scenarios.
  A value-taking flag is now also required to be given its value **once**: `--baseline a --baseline b`
  (in either form, or one of each) is a usage error naming the flag instead of the second value
  silently overwriting the first. The invocation named two files and the runner acted on one, saying
  nothing about the other — the same dropped-flag mistake one token further out, and which value a
  repeat means is not inferable, so neither is chosen. The exit code was already 2 for the three path
  flags (the surviving value's own downstream failure), so what changes there is which mistake the
  diagnostic names — `--baseline first --baseline second` reported `cannot read baseline second`; for
  `--manifest-path` given two valid paths it changes 0 → 2. A repeated **boolean**
  (`--warn-uncovered --warn-uncovered`) is deliberately still accepted: the second occurrence asks
  for exactly what the first set, so nothing is dropped and there is nothing to report.
- A flag that `check` recognizes but whose effect the requested action cannot produce is now a usage
  error naming the flag (exit 2), instead of being accepted and dropped: `--write-baseline` records a
  snapshot and emits no report, so `check --manifest-path <ws> --write-baseline out.json
  --warn-uncovered --format sarif` used to record the baseline, exit **0**, and discard both flags
  with no diagnostic — an adopter could believe they had coverage advisories or a SARIF document and
  receive neither. A rejected invocation now writes no baseline either. This is the rule `list`
  already applied across commands ("a flag that is recognized by `check` but inapplicable to `list`
  SHALL be rejected rather than accepted as a silent no-op"), and that `--disallow-stale requires
  --baseline` already applied in the other direction, finally applied *within* `check` between its
  two actions — the one place it did not hold. `--format text` is rejected alongside `sarif`/`json`,
  since the value asked for is irrelevant to an action that can honor no format at all. The line is
  deliberately "the action produces nothing this flag could affect", not "this flag changes nothing
  observable": `--warn-uncovered` under `--format json` stays accepted, because the JSON report's
  `coverage` object already names every uncovered crate whether or not the flag is given, so the flag
  is redundant there rather than dropped. `cli-check-runner` gains a requirement stating both halves
  of that line, with three scenarios.
- **BREAKING**: 圭表's inbound module-boundary rules (`must_not_be_imported_by`,
  `must_only_be_imported_by`) now react to an item-form import (`use m::Item;`) of the anchored
  module under `ScanDepth::Shallow`, not only a bare import of the module itself. The Shallow
  target match compared the import's full path string — including any item leaf — directly against
  the anchored module, so `within_scan_depth("crate::internal::Secret", "crate::internal",
  Shallow)` demanded exact string equality and silently failed: a real, released-since-0.3.0 false
  negative in exactly the boundary PROJECT.md's core contract forbids reacting silently in. The
  same import in bare-module form (`use crate::internal;`), and the identical item-form import
  under `ScanDepth::Subtree`, both already reacted correctly — only the Shallow + item-import cell
  was silent. Fixed by resolving the import path to the module it actually denotes (itself, or its
  longest reachable-module prefix) before the depth comparison, using the same reachable-module
  set `ScanContext` already carries — so an item declared directly in the anchored module reacts,
  while an item in a descendant module correctly does not. Closing that target-match precision also
  surfaced a latent, adjacent false positive: the importer-side self-import exemption (a file
  within the protected module's own subtree is never an inbound importer) was itself gated to
  `Subtree` only. Fixed alongside it, so the self-import exemption now holds identically at both
  depths, matching `module-boundary`'s own unconditional wording — narrowing to `Shallow` scopes
  what counts as *reaching* the protected module, never who counts as *inside* it. Any existing
  `--write-baseline` output for an inbound rule declared at `ScanDepth::Shallow` may need
  regeneration: an import that previously passed silently may now correctly react.
  That exemption fix reached the per-import check but left the **file-level pre-filter** still gated
  to `Subtree`, which was dead logic held correct only by a non-local argument — and not inert: at
  `Shallow` a file the exemption excuses was still read and scanned, so an unreadable one, or one
  whose `use` tree nests past the scanner's brace-nesting cap, turned a `Shallow` inbound rule into
  exit 2 where `Subtree` exits 0. Both sites now call one depth-free predicate
  (`is_inside_protected_module`), so the pre-filter and the rule cannot drift, and what the exemption
  excuses can no longer decide the exit code.
  That target-match resolution's own namespace-blindness was recorded here as a **stated bound**: `mod
  foo` and `fn foo` resolve in different namespaces, so both can be declared in one module and a single
  `use m::foo;` binds both (verified against rustc), while the path-only resolver returns the module
  reading and leaves the value reading unobserved under `Shallow`. **That bound did not survive this
  window** — it was a false negative, and it is closed: see "圭表's inbound module rules now observe the
  **value namespace**" under *Fixed*, which is the state 0.4.0 ships. What this entry recorded that still
  holds is the reasoning that shaped the fix, not the fix's absence: reacting on both readings was
  rejected then and is still rejected now, because it would make every ordinary `use m::child;` react
  under `Shallow`, so the closure consults the value namespace instead of unioning the readings. The test
  it named was inverted with the bound and is now
  `shallow_inbound_target_match_observes_the_value_namespace`; `rule-model-surface` states the closure
  with two scenarios; and the `BACKLOG.md` entry is closed rather than READY-PATCH. The helper the two
  families used to share is split:
  external-crate confinement keeps a depth-sensitive pre-filter under a name that says so
  (`hosts_only_permitted_importers`), because there the skip is only sound when every importer the
  file can host is permitted — never under `Shallow`, where an inline `mod` inside the permitted file
  lies outside the anchored module. `rule-model-surface` gains both requirements and a scenario.
- 天衡's `--write-baseline` now overwrites an existing, supported baseline durably: the merged
  document is written to a sibling temp path first, then an atomic `rename` swaps it into place,
  instead of a bare truncating write. A crash, interrupt, or full disk mid-write previously left the
  baseline truncated — destroying exactly the owner/tracker annotations the metadata-preserving
  merge exists to carry forward, contradicting the function's own stated intent. The swap targets
  the file's symlink-resolved real path and carries over its existing permissions, set at creation
  rather than narrowed afterward: `rename` unconditionally replaces whatever sits at its
  destination, so writing straight to the given path would otherwise replace a symlinked baseline
  with a plain file (orphaning whatever it pointed at), and creating the temp file at the process
  umask default before narrowing it would briefly widen permissions an adopter had deliberately
  restricted. The temp file itself is opened with `create_new` (`O_EXCL`): its name is predictable
  (`<target>.tmp-<pid>`), so a plain create-or-truncate would follow whatever already sat at that
  path — a symlink included — letting anything pre-planted there redirect the write onto an
  arbitrary file; `create_new` refuses outright instead. Its path is built from the resolved
  target's raw bytes rather than through lossy display formatting, so a target reached through a
  non-UTF-8-named directory component no longer fails the overwrite outright. A stale temp file
  left behind by an interrupted prior run (a killed process, or a pid reused across a fresh
  container) is a real, reachable case `create_new` also reports — now with its own specific
  message naming the actual colliding temp path and explaining why it is there, rather than a bare
  `cannot write baseline <path>: File exists` that names nothing the adopter can act on. A baseline
  path that is a symlink to a deleted target is reported by its own cause too, not misattributed to
  the sibling "it appeared while the new snapshot was being prepared" race message: `O_EXCL` fails
  on a dangling symlink exactly as it does on a genuine concurrent creation, but the two are not the
  same state — a dangling symlink is permanent, so "rerun the command" (that message's own remedy)
  could never have succeeded. The create-new path (writing a baseline where none existed) is
  otherwise unaffected: it has no pre-existing content to protect, and already fails loud rather
  than clobbering if the file appears concurrently.
- Bounded native recursion depth across four recursive walkers in three crates, closing the same
  false-negative-adjacent bug class in every observation dimension — a pathologically (but
  genuinely acyclic) nested module tree, `use` tree, or block/macro-arm structure could overflow
  the native stack (an uncontrolled process abort) instead of the contract's own exit-2 "cannot
  judge". Three of the four (圭表's two, 渾儀's) had an existing cap that silently returned an
  empty/partial result past it instead of erroring — `Outcome::Clean` when a real violation
  exists, the exact false negative PROJECT.md's core contract names as the one forbidden bug; the
  fourth (漏刻's) had no cap at all. Each bound was *measured* against a real crash, never guessed:
  an initial 512 guess for 渾儀's walkers crashed a real test process. The settled bounds: 32
  (渾儀's `walk_module`/`collect_subtree`/`walk_unsafe`, clear of a measured 80–90-level crash line,
  and independently clear of `syn::parse_file`'s own ~300–350-level parser-recursion crash line on
  the same fixture shape); 128 (圭表's `use_scan::expand_use_tree_depth`); 64 (圭表's
  `symbol_scan.rs`'s `glob_bases` and `expand_use_leaves`'s inner `go`, feeding the glob-hazard
  pass and alias resolution for `ConfineInlineSymbolPath` — the identical silent-truncation shape
  `use_scan` was fixed for, never carried over to this sibling scanner until now); and 300 (漏刻's
  `audit/scan.rs::collect_scope_modules`, which recurses through transparent-macro arms, inline
  `mod` bodies, and arbitrary blocks — measured safe at depth 1100 and a reproducible SIGABRT at
  depth 1105+ under a 2MB test-thread stack). Each fix added tests proving both directions:
  nesting comfortably under the bound is still fully observed, and nesting past it is a scan
  error, never a crash or a silent pass.
- **BREAKING**: `PublicSeam::InherentMethod`/`InherentAssoc` now carry the impl **block's own**
  declaring module, distinct from the self type's canonical `owner` path. `owner` names what the
  impl is *for*, not where it is *written* — Rust's coherence rules let an inherent `impl` for one
  type be written in any module of the same crate, a real, common platform-conditional idiom
  (`impl Conn { … }` once in `plat_unix`, once in `plat_win`, both for a `Conn` declared in
  `common`). Two such impl blocks declaring a same-named public method/associated item previously
  resolved to the identical `{owner, name}` seam and collapsed to one violation: measured on the
  real `hunyi::check_impl_trait`'s `including_submodules()` subtree scan, the second module's real
  violation was silently dropped by the fact-only dedup, not merely deduplicated against an
  equivalent finding — the false negative PROJECT.md's Core Contract forbids outright. dyn-trait and
  signature-coupling build the identical seam through the same constructors but cannot currently
  observe more than one module per evaluation, so they close the identical structural gap
  pre-emptively rather than a second live false negative. **Any existing `--write-baseline` output
  for an `InherentMethod`/`InherentAssoc`-seam finding is now stale** (the fact gained a required
  field) and must be regenerated; every previously accepted violation reappears as new exactly once.
  Rendered finding text is unchanged (the module is identity-only, matching
  `AsyncInherentMethod`'s own already-shipped precedent). No DSL, builder, or CLI surface change —
  only the identity `fact` payload gains a field, the identical shape this same `[Unreleased]`
  window's own `governing_package` fix already took (below).
- **BREAKING**: `PublicSeam::InherentGenerics` now also carries the **bounded thing** each exposure
  sits on — a generic parameter's own name, or a where-predicate's rendered bounded type. Module and
  owner were not enough for this one seam: module says where an impl block is written and owner says
  what it is for, and neither says which block, so two inherent impl blocks on one type in ONE module —
  `impl<T: Secret, U> Conn<T, U>` beside `impl<T, U: Secret> Conn<T, U>`, each exposing the same
  forbidden subject through a different bound — produced identical facts and collapsed to one violation,
  letting a baseline accepting the first suppress the second's never-accepted one. Unlike a method or an
  associated item, this seam has no per-item name to fall back on, which is why it needed a role of its
  own. The role is not invented: it is keyed exactly like a trait `impl`'s existing `where` position,
  and both now come from one shared walk over an impl block's generics positions, so the two
  vocabularies cannot drift and the third collector (dyn-trait) gets the same keys for free. An
  impl-block **ordinal** was explicitly not used — `semantic-signature-coupling` forbids identity
  resting on scan order or item ordinal, so a positional key would have traded one defect for a rule
  violation. Walking positions instead of the whole generics node loses nothing, which rests on a
  language rule checked against a real `rustc` rather than assumed: an `impl`'s generic parameters
  cannot carry defaults, so a parameter contributes only its bounds (or, for a const parameter, its
  type), and lifetime positions name no type. Two blocks whose bounds are textually identical still
  resolve to one seam — a limit rather than a gap, since nothing structural distinguishes them and two
  blocks bounding the same parameter to the same forbidden type state one fact twice, the same reason
  one import on two lines is one violation; stated in the seam's own doc. **Rendered finding text
  changes** for this seam, deliberately: `impl <Owner> (generics)` becomes
  `impl <Owner> (generics: T)`, because two distinct violations that render identically make a report
  unreadable even when identity is correct. **Any existing `--write-baseline` output for an
  `inherent_generics`-seam finding is stale** (the fact gained a required field) and must be
  regenerated. `semantic-signature-coupling` gains the rule, its stated limit, and a scenario.
- **BREAKING**: 渾儀's trait-impl-locality violation identity now uses the **resolved** trait anchor
  for both its `target` and its rule key, instead of the constitution's declared spelling. Matching
  already resolved both sides — the declared anchor through the crate's own `pub use` closure, and each
  impl site's trait path — so only identity kept the raw declaration, and renaming a boundary from a
  facade spelling to the trait's defining path (a pure refactor: no code change, the same impls still
  misplaced) gave every affected violation a new `ViolationId`. Each accepted violation re-fired as new
  while its recorded entry reported stale — the baseline-defeating churn this window has closed twice
  elsewhere, here reached by editing a declaration rather than by moving a checkout. Two equivalent
  spellings now converge on one identity. The multi-candidate question that made this design work is
  answered by refusing rather than picking: a declared anchor whose re-export closure reaches more than
  one distinct local trait DEFINITION (two mutually-exclusive `#[cfg]` branches re-exporting different
  traits under one facade) is a constitution error naming both candidates and pointing at the defining
  path, because the ambiguity is in the declaration and choosing one would make the governed target
  arbitrary. `allowed_locations` deliberately stays inside the rule key — it is what keeps two
  boundaries governing the same trait with different allowed sets from collapsing onto one identity for
  one misplaced impl — and the in-code comment that claimed the opposite ("not part of the violation's
  identity — so editing the allowed set does not turn a still-misplaced impl into a new violation") is
  corrected: `ViolationId` compares `rule_key` in full, so editing the allowed set does re-fire
  still-misplaced impls as new. Loud churn, never masking, and now stated rather than denied.
  **Any existing `--write-baseline` output for a trait-impl-locality boundary declared through a
  re-export spelling is stale** and must be regenerated; a boundary that already named the defining
  path is unaffected. `semantic-trait-impl-locality` gains all three rules with two scenarios.
- **BREAKING**: 渾儀 now refuses to name an owner whose self-type head two mutually-exclusive
  `#[cfg]` branches bind to different targets, instead of rendering both sites onto whichever
  candidate came first. `#[cfg(unix)] use crate::a::Foo as X; #[cfg(not(unix))] use crate::b::Bar as
  X;` with an `impl` for `X` in each branch is two genuinely different types; resolution is cfg-blind
  by design, so both bindings are live candidates and the owner renderer took the first — giving the
  two sites the identical owner. Owner is a dedup key, so they collapsed into one violation and a
  baseline accepting the first suppressed the second's never-accepted one: the false negative
  PROJECT.md's Core Contract forbids outright, across trait-impl-locality, forbidden-marker,
  unsafe-confinement, and signature-coupling at once. Neither candidate can be preferred (only one
  compiles, and which one is a `cfg` evaluation this dimension deliberately does not perform), and the
  candidate SET is identical for both sites so it cannot separate them either — so the reaction is to
  refuse: the ambiguity reaches the same fail-loud identity gate an unrenderable self type already
  hits, a constitution error (exit 2) whose diagnostic names the `#[cfg]` collision as the cause
  without publishing the internal sentinel that carries it. "Cannot judge" over a silent collapse is
  the Core Contract's own ordering. The structural half: the single-candidate `resolve_path` that fed
  every such label was **deleted** rather than bypassed, leaving `resolve_path_all` the only resolver,
  so a caller needing one value must now decide what to do about more than one instead of receiving an
  arbitrary pick — the defect class is unrepresentable rather than fixed at three call sites. Its own
  doc had claimed those identity callers "have no audit-verified need for cfg-blind multi-candidate
  resolution"; that claim was the bug, stated. A single alias binding — the ordinary case — resolves
  exactly as before, pinned by its own control test. **Any existing `--write-baseline` output is
  unaffected** (no identity shape changed; what changes is that an ambiguous one is now refused rather
  than recorded under an arbitrary label). `semantic-signature-coupling` gains the requirement and its
  two scenarios, and the shared gate's diagnostic now names which role failed and why, keeping its
  previous sentence verbatim as the prefix so the fail-loud tests that pin it stay meaningful.
- **BREAKING**: `PublicSeam::InherentGenerics` and `PublicSeam::ExternCrate` now carry their
  declaring module too — the two seam shapes the fix above left behind, found by re-reviewing that
  fix's own reasoning against every shape in the vocabulary rather than only the ones it touched.
  They were the only two of the eleven that carried no module at all, and both are reachable by two
  distinct source sites with identical identity: an impl block's own generics seam
  (`impl<T: crate::infra::Secret> Conn<T>` written once in `plat_unix` and once in `plat_win`) is
  distinguished by nothing but its owner, since — unlike a method or an associated item — it has no
  item name to fall back on, and `pub extern crate <dep>;` is legal in more than one module of one
  crate, where the republished crate's name was the whole identity. Either shape collapsed two real
  violations into one, so a baseline accepting the first silently suppressed the second's
  never-accepted violation: the false negative PROJECT.md's Core Contract forbids outright. Like the
  fix above, this closes a structural gap rather than a second live report — signature-coupling and
  dyn-trait, the two capabilities that build these seams, still observe one module per evaluation —
  and it is closed now for the same reason: the exclusion that makes it unreachable is an accident of
  today's walkers, not a property of the identity. A trait `impl` seam deliberately stays
  module-free, and that is now stated as the coherence argument it is rather than left silent: its
  trait reference and owner both carry their rendered generic arguments, and Rust rejects two impls of
  the same trait — same arguments — for one self type anywhere in a crate, so two coexisting blocks
  already differ in one of those roles. **Any existing `--write-baseline` output for an
  `inherent_generics`- or `extern_crate`-seam finding is now stale** (each fact gained a required
  field) and must be regenerated; every previously accepted violation reappears as new exactly once.
  Rendered finding text is unchanged in both cases (identity-only, the same precedent), and no DSL,
  builder, or CLI surface changes. The seam-coverage fixture now carries a colliding pair for each
  shape, so the existing injectivity assertion — not a new hand-maintained rule — is what fails if
  either module role is dropped again, and each of the three construction sites has its own live
  two-module reaction test, since the fixture alone would let a site pass a module-blind constant.
- 渾儀 now rejects a forbidden/allowed operand whose `::`-delimited spelling has an empty segment
  (a leading `::`, a trailing `::`, a doubled `::`, or the empty string) as a constitution error,
  across `must_not_expose`/`and_not_expose`, `must_not_expose_dyn_of`, `must_not_expose_impl_trait_of`
  (module- and subtree-scoped alike), and `must_not_acquire`/`and_not_acquire`. `extern_verbatim_renamed`
  builds a resolved canonical path purely from `syn::Path` segments — it never carries a leading `::`
  regardless of how the scanned source is spelled — so an operand like `must_not_expose("::serde")`
  could never equal or prefix-contain a real resolved path and silently, permanently never reacted;
  `must_not_acquire`'s leaf-identifier matching has the mirror-image gap for a trailing `::`. No
  existing usage in this repo used the malformed spelling, so this is a strict tightening of an
  already-inert configuration, not an adopter-visible behavior change for any working boundary.
- 渾儀's `only_implemented_in`/`and_in` (`allowed_locations`) and `only_under` (unsafe-confinement's
  own `allowed_locations`) now reject the identical malformed `::`-path shape as the forbidden-operand
  fix above, sharing its guard (`resolve::validate_path_operands`). Unlike the forbidden-operand
  direction, the previous behavior here already failed loud rather than silently passing — a
  malformed allowed entry never matched any real module location in `matches_allowed`, so a
  genuinely-in-place impl or `unsafe` site was reported as a spurious violation instead of a
  named constitution error. No existing usage in this repo used the malformed spelling, so this
  is a diagnosis improvement on an already-broken configuration, not an adopter-visible behavior
  change for any working boundary.
- 漏刻's CI probe audit now reads the arms of a `cfg_if!` invocation as real code, in both of its
  passes, completing the family: all three dimensions now share one transparency rule and are pinned
  on one shared fixture (`cfg_if_transparency_conformance.rs`). Skipping such a body like any foreign
  macro broke two of the audit's three reaction directions, in both error directions at once. **Two
  false negatives close:** a probe naming a mis-typed seam inside an arm escaped the
  probed-but-undeclared reaction entirely (at runtime it asserts against a seam nobody declared), and
  an un-auditable probe (a non-literal seam argument) inside an arm was silently skipped —
  contradicting `audit_probe_coverage`'s own documented promise that a silent skip never happens. **A
  false alarm retires:** a seam whose only production probe lived inside an arm was reported unprobed,
  failing an adopter's CI over coverage they actually had; the same held for every probe beneath a
  `mod` declared only inside an arm, since that module never entered the reachable corpus. An
  arm-declared module is now also treated as cfg-conditional, so an absent conventional file is
  tolerated exactly as under a bare `#[cfg]` (圭表's rule, adopted), while a resolution ambiguity stays
  a constitution error under every gate. Bounds unchanged and now uniform across the three: only
  `cfg_if` is transparent, and observation stays cfg-blind. A newly caught typo'd or un-auditable probe
  is a real finding and absorbable by baseline.
- 渾儀 now reads the arms of a `cfg_if!` invocation as real code, in every walk it performs. Closes an
  exposure false negative measured on ordinary, compilable source: a `pub fn` returning a forbidden
  type reacted at a module's top level and **passed** when the identical function sat inside a
  `cfg_if!` arm, because `syn` parses the invocation as one opaque macro item and no capability
  handled that variant. A `mod` declared only inside an arm was equally invisible, so its file's
  `unsafe` sites, forbidden markers, trait impls, and re-exports went unobserved with it, and the
  module could not be named as an anchor at all. 圭表 has read these bodies since 0.2.3, so an adopter
  using `cfg_if!` was already seeing the static half of these findings — this adds the semantic half,
  and the two dimensions are now pinned on one shared fixture
  (`cfg_if_transparency_conformance.rs`). Three properties come with it, stated rather than implied:
  an arm-declared module is cfg-conditional, so an absent conventional file is tolerated exactly as
  under a bare `#[cfg]` (圭表's rule, adopted); both conventional forms present is still an ambiguity
  constitution error under arm membership; and arms are unioned **cfg-blind**, so a violation in an
  arm this build does not compile still reacts. Only `cfg_if!` is transparent — a body-wrapping macro
  under any other name stays unobserved, which is load-bearing rather than cautious: reading an
  arbitrary macro's braces as arms recovers items from a nested `impl` block that the macro may never
  emit, a false positive. Transparency also covers **item position** only: an invocation written
  inside an `impl` or `trait` body still goes unobserved, a measured gap left stated and owned by its
  own change rather than half-closed here. New violations are ordinary findings and absorbable by
  baseline.
- 圭表 now treats a `mod` declared inside a `cfg_if!` arm as cfg-conditional, so an absent
  conventional file (or an absent unconditional `#[path]` target) is tolerated exactly as it already
  is for a bare `#[cfg]`-gated declaration. Completes the 0.2.3 transparency carve-out, which made arm
  bodies observable but left the absent-file tolerance keyed on an attribute preceding the item — a
  `mod` inside an arm carries none, because the predicate lives in the macro's `if #[cfg(..)]` header.
  The two spellings of one per-platform shim therefore gave opposite verdicts: with only one arm's file
  committed, the bare-attribute form exited 0 while the `cfg_if!` form exited 2, reporting the absence
  as unconditional — on source that compiles, since rustc strips the non-selected arm. Adopters who saw
  that exit 2 now get a real verdict, which may surface violations in modules the aborted walk never
  reached and therefore need baselining. Tolerating an absence cannot hide anything: a file that does
  not exist holds no code. An arm module whose file exists is still reached and governed, and both
  conventional forms present at once is still an ambiguity constitution error under every gate.
- 渾儀 now reacts with a constitution error (exit 2) when a plain `mod name;` is backed by BOTH
  conventional forms at once (`name.rs` AND `name/mod.rs`), instead of silently resolving to the
  first form it probes and never reading the other. Closes an exposure false negative: with the two
  files present, moving a forbidden exposure from `name.rs` into `name/mod.rs` turned a reaction into
  a clean pass, so whether the module was governed at all depended on which file its author wrote the
  item in. 圭表 and 漏刻 have both reacted to this shape since 0.2.3 and earlier, and the composed
  `tianheng check` therefore already exited 2 on it — the gap was reachable by a **standalone 渾儀**
  consumer. Two trigger shapes, stated plainly: a live declaration of this kind is a rustc compile
  error (E0761), but a `#[cfg]`-gated-off one is stripped before module resolution and **compiles**,
  and it also now reacts, because cfg-blind observation cannot know which arm is live (the ordering
  圭表 and 漏刻 each already apply). A constitution error never enters a baseline; the repair is to
  delete whichever of the two files is not the module. All four outcomes of the lookup are now pinned
  across all three dimensions in `dual_backed_module_conformance.rs`.
- **BREAKING**: 圭表's and 渾儀's violation identity now carries the crate a boundary was declared
  against. Neither dimension's fact construction previously named the declaring crate — only a bare
  module path — so two workspace members declaring the identical rule against the identical module
  path collapsed into one `ViolationId`: the composed report silently dropped the second crate's real
  violation, and a baseline accepted for one crate could suppress the other's never-accepted one (the
  false negative PROJECT.md's Core Contract forbids outright). Every `ModuleFact` and `SemanticFact`
  variant now carries a `governing_package` identity field equal to the boundary's declared crate
  (`unsafe_confinement` excepted — its identity already varies by crate through `target`). **Any
  existing `--write-baseline` output for a module or semantic boundary is now stale** (identity
  gained a required field) and must be regenerated; every previously accepted violation reappears as
  new exactly once. No DSL, builder, or CLI surface change — only the identity `fact` payload gains a
  field.
- 圭表's lexical hygiene no longer panics on a governed source file ending in an unterminated block
  comment that swallows a multi-byte UTF-8 character. The comment-stripping pass could leave exactly
  one trailing byte unconsumed when a comment never closed before EOF; if that byte was the orphaned
  tail of a multi-byte character whose lead byte(s) were already dropped inside the comment, it was
  re-scanned as code and pushed alone into the stripped buffer — an invalid UTF-8 fragment that
  `String::from_utf8_lossy` then lengthened (one byte becomes the 3-byte U+FFFD replacement),
  desynchronizing the position map from the string it indexes into and panicking the next stage's
  lookup. An unterminated comment is now treated as extending through end-of-file, so nothing is left
  to re-scan. Not a behavior an adopter could have depended on — a crash is none of PROJECT.md's Core
  Contract outcomes (0 clean / 1 violation / 2 constitution error) — so no **BREAKING** marker.
- 圭表 no longer silently passes a forbidden import when a non-ASCII char literal sits immediately
  adjacent to a `'{'` char literal (e.g. `['«','{']`, no space) — the false negative the Core
  Contract forbids outright. The lexer's "simple char literal" check assumed every char literal's
  payload is exactly one byte, which holds for `'x'` but not a multi-byte UTF-8 scalar (`'«'` is 2
  bytes, `'未'` is 3); for a non-ASCII literal the check failed and the scalar's raw bytes leaked
  into the cleaned text as ordinary code. When a second literal followed closely enough, the misread
  literal's real closing quote, an intervening comma, and the next literal's real opening quote could
  coincidentally match the old one-byte assumption exactly, swallowing that opening quote too — which
  left the next literal's own payload (here, `{`) unprotected, leaking it into the cleaned text as a
  spurious structural brace and throwing off the reachability walker's brace-depth tracking for every
  `mod` declared after it. The check now measures a char literal's real UTF-8 byte length from its
  lead byte rather than assuming one. Not breaking — this closes a false negative against
  `module-boundary`'s already-stated import-detection contract; no baseline identity shape changes.
- 渾儀's signature-coupling query now observes a `pub fn`/`pub static` declared inside an `extern`
  block — the FFI declaration is a real, callable item in the enclosing module's own namespace,
  exactly as public as a same-shaped ordinary item, but the exposure collector had no
  `ForeignMod` handling at all, so a forbidden type named only there escaped the query entirely
  (exit 0 Clean on source with a real, callable public API leak). Reuses the existing seam/path-
  collection machinery verbatim — no new seam kind, since Rust cannot declare both an ordinary item
  and a foreign one under the same name in one module, so there is no identity collision to design
  around. Not breaking — closes a false negative; no baseline identity shape changes.
- 渾儀's visibility-boundary query (`must_not_declare_pub` / `max_visibility`) now observes a `pub
  fn`, `pub static`, or `pub type` declared inside an `extern` block — the sibling gap the
  signature-coupling fix above did not touch, since the two capabilities collect a module's direct
  items through entirely independent per-item logic (`collect_item_exposures` vs.
  `item_observation_parts`), sharing only the underlying module-item enumerator. A bare-`pub` foreign
  item is exactly as visible as a same-shaped ordinary one, but `item_observation_parts` had no
  `ForeignMod` arm at all, so an `extern` block's declarations were silently absent from the
  module's observed direct items regardless of their declared visibility (exit 0 Clean on a module
  whose only bare-`pub` item sat inside an `extern` block). `pub type` (an extern type declaration)
  is included here though it carried no exposable signature and so was out of the
  signature-coupling fix's own scope — this capability cares about the declared keyword, not a
  type-signature leak. Reuses the existing `Fn`/`Static`/`Type` visibility kinds verbatim, no new
  kind, for the identical no-identity-collision reason as the sibling fix. `item_observation`
  widens from `Option` to `Vec` (an `extern` block can hold more than one independently-visible
  foreign item, unlike every other observed item kind), with its one call site updated accordingly.
  Not breaking — closes a false negative; no baseline identity shape changes.
- 渾儀's shared `use`-map and re-export closure no longer silently drop one candidate when two
  mutually-exclusive `#[cfg]` branches (bare `#[cfg]` or `cfg_if!` arms alike) declare `use ... as
  Name;` (or `pub use ... as Name;`) for the identical name with different targets. Both were
  single-valued (`HashMap<String, String>`), so the second declaration always overwrote the first —
  the verdict for a real forbidden-type exposure depended on which mutually-exclusive branch was
  written last, not on whether either branch's binding was genuinely forbidden. Both maps are now
  multi-valued (mirroring the crate's existing type-alias map). Every matcher that consumes them now
  checks every candidate and reacts if any is forbidden, not only signature-coupling's exposure
  resolution and dyn-trait's/impl-trait's shared operand-scoped principal-trait resolver (discovered
  to have the identical gap while fixing this): an adversarial review of the fix itself found the
  same order-dependent silent pass still reachable through forbidden-marker's derive and impl-form
  leaf matching, its self-type/marker-acquisition landing (through a third, previously single-valued
  type-alias map), and trait-impl-locality's anchor resolution — each independently reproduced before
  being closed here too. Not breaking — closes false negatives; no baseline identity shape changes.
- 渾儀's crate-wide scan no longer drops a module reached only through a `cfg_attr`-wrapped `#[path]`
  remap. `cfg_attr` never removes the `mod` item the way a bare `#[cfg]` does, so the module is
  present on every configuration and needs SOME file to back it — treating the attribute as a blanket
  skip bound dropped the whole subtree, not just the alternate target its predicate might select. Two
  shapes: an **inline** module's body is unaffected by `#[path]` at all (rustc ignores it there; the
  body always compiles) and is now always descended; a **file** module's conventional file and its
  `cfg_attr` target are both read when they exist on disk, unioned rather than either being silently
  preferred — matching 圭表's own already-fixed union-scan policy for the identical shape. Neither
  candidate existing, with no other cfg-conditional gate, remains a genuine scan error. Since the
  crate-wide scan backs signature-coupling's own alias/re-export closure and dyn-trait's/impl-trait's
  shared operand-scoped principal-trait resolver, not only forbidden-marker, trait-impl-locality, and
  unsafe-confinement (the two capabilities the discovering findings measured against), and since
  async-exposure's and impl-trait's own subtree-scope opt-in (`including_submodules()`) shares the
  identical walker one hop further out (found on adversarial review), all seven were independently
  reproduced and confirmed fixed by this one change. `module_resolve.rs`'s separate single-module-
  anchor resolution (signature-coupling's own anchor, visibility, dyn-trait's shape-only module-scoped
  resolution, and trait-impl-exposure) gets the identical fix: a third adversarial review disproved
  this change's own earlier claim that the function was "already correct, fails loud" — a mutually-
  exclusive sibling declaration for the same module name silently absorbed the branch count, so the
  `cfg_attr` target's own file vanished with exit 0 whenever ANY sibling resolved, and even a LONE
  such declaration never followed an existing target file at all. Now it does, the same union as the
  crate-wide walk. A fourth review then found one more gap shared by both walkers: a module stacking
  more than one SEPARATE `cfg_attr`-wrapped `#[path]` attribute (one per platform predicate — the
  natural 3+-way per-platform shim) only ever had its first-declared candidate tried; every other
  platform's target silently never was. Every stacked candidate is now read. Not breaking — closes
  false negatives; no baseline identity shape changes.
- 漏刻's CI-face audit scanner no longer drops a module whose declaration carries a comment between
  the `mod` keyword and its name (or between the name and its terminator) — trivia to rustc, but a
  bare whitespace-only skip stopped at the comment's leading `/`, so the declaration was never
  recognized as a `mod` at all: the module and its whole subtree, and every probe beneath it,
  silently vanished from the corpus. It also now descends into every function/block/match-arm body
  looking for a nested `mod`, not only the scopes it specifically recognized — the only legal
  non-inline module form there, `#[path] mod name;`, was previously invisible with no loud signal at
  all. And `mod_preamble_attrs`'s documented `cfg_attr(path)` tolerance is now actually implemented:
  the attribute match previously checked for the exact identifier `cfg`, so `cfg_attr` — a different
  identifier — matched neither the `path` arm nor the `cfg` arm, and a module stacking two
  `cfg_attr`-wrapped `#[path]` declarations that together cover every platform (both targets present,
  compiling cleanly everywhere) was reported a hard constitution error instead of being scanned — a
  false positive on entirely valid code. Every `cfg_attr` target that exists on disk is now read,
  unioned with the conventional file, matching the crate-wide walk 圭表 and 渾儀 both already apply to
  the identical shape; the same union now also applies to a `cfg_attr`-wrapped `#[path]` on an
  *inline* `mod x { … }` (governing the base directory x's own nested items resolve from, descended
  only when it exists as a directory, falling back to the conventional base when none does) — an
  adversarial review round found the first cut had wired the union into only the external-`mod x;`
  consumer. A doubly-nested `cfg_attr(cfg_attr(path))` remains a stated, undetected bound of this
  hand-rolled scanner. Not breaking — closes false negatives and one false positive, not an identity
  shape; no baseline identity shape changes.
- 圭表's own module-boundary reachability walk no longer requires a plain conventional file
  (`name.rs` / `name/mod.rs`) for a declaration backed only by one or more `cfg_attr(path)`
  remaps. A resolved `cfg_attr(path)` candidate was already union-scanned for governance, but the
  separate plain-file requirement ran unconditionally regardless: a module stacking two
  `cfg_attr`-wrapped `#[path]` attributes that together cover every platform (both targets
  present, no plain file ever needed) was reported a hard constitution error — "source file could
  not be located" — instead of being governed, a false positive on entirely valid code, and not
  specific to "stacked": a single `cfg_attr(path)` target with no plain fallback hit the same
  error. A resolved candidate is now treated as legitimate grounds for the plain file's own
  absence, the same "might legitimately be absent on this build" signal a bare `#[cfg]` or a
  `cfg_if!` arm already carries — matching 渾儀's/漏刻's own `has_backing_source` rule for the
  identical shape (三儀 ⊥ 三儀: the same rule, not the same function). Two outcomes stay exactly as
  strict as before: both conventional forms present is still an unconditional ambiguity error
  regardless of any resolved candidate, and a declaration whose every candidate is absent (no
  plain file, no resolved `cfg_attr(path)` target, no bare `#[cfg]`) is still a genuine
  constitution error. Not breaking — closes a false negative; no baseline identity shape changes.
- `deny.toml`'s `[advisories]` table now sets `yanked = "deny"` explicitly: the field's own unset
  default is `"warn"`, so `cargo deny check` was printing `warning[yanked]: detected yanked crate`
  and still exiting 0 (`advisories ok`) — reproduced against a real yanked crate pinned into the
  lockfile — directly contradicting the section's own stated claim that yanked crates are denied.
  `scripts/test_examples.sh` now asserts (`cargo tree -p <crate> --depth 0`) that every example's
  `patch.crates-io` override actually resolved to local source, for every family crate it patches:
  reproduced against a version-bumped scratch copy of the workspace, Cargo was silently dropping an
  incompatible patch (`patch ... was not used in the crate graph`) and falling back to the last
  published crate, so the dogfood gate stayed green while silently testing stale, already-published
  code instead of the in-development tree it exists to exercise. Not breaking — strengthens two CI
  gates to enforce what they already claimed; neither the yanked crate nor the incompatible patch is
  present in the current workspace, so this has no effect on the present green build.
- **BREAKING for a recorded baseline**: 圭表 now reads a value's name past a `static mut` modifier,
  closing the last false negative of that shape. The name was taken as the identifier following the item
  keyword, which for `static mut foo` is `mut` — so the module recorded a value named `mut`, never `foo`,
  and an import binding the `static mut foo` declared beside a `mod foo` passed silently. rustc compiles
  that pair and one `use m::foo;` binds both. The four modifier spellings that already worked
  (`extern "C" fn`, `const fn`, `async fn`, `unsafe fn`) worked for a reason that does not generalize:
  `fn` is itself an item keyword, so the walk's next step reaches the real name — `mut` is not one, so
  nothing recovered. By the grammar (`static [mut] NAME: TYPE`) this is the only item of that shape, and
  all six spellings are now pinned together so the distinction is visible rather than rediscovered. The
  token is skipped **unraw'd only**: `pub static r#mut: u8` legally names the item `mut`, so skipping that
  spelling would attribute the following token to it — trading the fixed false negative for a false
  positive. That bound is pinned as its own case.
- **BREAKING for a recorded baseline**: 圭表 now observes a value declared inside an `extern` block as a
  value of the **enclosing** module, closing a false negative — the class `PROJECT.md` forbids outright.
  An extern block's `{` opens a brace but no naming scope: `unsafe extern "C" { pub fn foo(); }` declares
  `foo` in the module containing the block, and it legally coexists with `mod foo` because the two live in
  different namespaces. Verified against rustc — the pair compiles, and one `use m::foo;` binds both, so
  `unsafe { foo() }` and `foo::INSIDE` resolve from that single import. The definition collector treated
  that brace like any other and recorded only items at the module's own depth, so the value was invisible
  and a real import of a protected module passed silently under an inbound `Shallow` boundary. 渾儀 had been
  corrected for this exact shape earlier in this window; 圭表's newer reader had not, which is the
  cross-dimension divergence the conformance ledgers exist to catch and this one slipped past. Three
  spellings are pinned (`unsafe extern "C"`, bare `extern`, and an `extern static`), and the counterweight
  is pinned beside them: an inline `mod` body's brace *does* re-scope, so a value declared there stays the
  submodule's and still does not react — otherwise this would trade a false negative for a false positive.
  The transparency also reaches the strict-external local-precedence ladder, which shares the walk, and
  correctly: a bare `rand()` call resolves to a local `extern "C" { pub fn rand(); }` exactly as it would
  to a plain local `fn rand()`, so treating the extern one as absent had been reading a local call as an
  external dependency.
- Closed a **third** import form of that same class: `use m::foo::{self};` binds the module `foo`, never a
  `fn foo` beside it, and was reacting. `use_scan` records a `{self}` leaf as its prefix module, so it
  arrives at the reaction byte-identical to a bare `use m::foo;` — the same collapse the glob condition
  had just been added for, one form over, in the same function. Verified against rustc: with both
  declared, `use m::foo::{self};` then `foo()` is `error[E0423]: expected function, found module 'foo'`
  while `foo::INSIDE` compiles. All four spellings were measured reacting — bare, `{self as f}`, nested in
  an outer brace group, and beside a sibling leaf — because each takes a different path through the
  use-tree expansion. The fix does not add a third condition: the language rule now lives on
  `ImportedPath::can_bind_a_value`, one question with the two ruled-out forms behind it, so the next form
  is one place to look rather than a third ad-hoc test. `rule-model-surface` is restated the same way —
  naming only the glob is what let this through, since the requirement read as complete while mandating a
  false positive in the same cell.
- Bounded that value-namespace reaction to what actually binds a value, closing two false positives it
  shipped with — both in the very cell it was written to make correct, and both contradicting its own
  central claim that "a narrow false negative must not be traded for a broad false positive". A **glob**
  import reached the reaction byte-identical to a bare one, because the scan stores a glob at its base
  module with `::*` removed, so `use m::foo::*;` satisfied every condition and reported an import that
  reaches only the descendant. It cannot bind a value at all — verified against rustc: with both `mod foo`
  and `pub fn foo` declared, `use m::foo;` compiles using `foo()` and `foo::INSIDE`, while
  `use m::foo::*;` fails `error[E0425]: cannot find function 'foo' in this scope`, a glob importing the
  module's *contents* and never the name. Separately, the value collector was handed the **raw** file
  against its own stated precondition, so a `fn foo` written only in a comment, a string literal, or a
  macro body counted as a declaration; the declaration-cleaning pipeline the dimension's four other
  readers each spelled out by hand is now one named function (`declaration_text`) that all five compose.
  `rule-model-surface` states both bounds with a scenario each. No public API or identity shape change.
- 漏刻 inherits absolute-reached provenance through an **inline** `mod`'s absolute `#[path]` base, which
  the earlier closure missed. `Path::join` discards its receiver when the joinee is absolute, so
  `#[path = "/abs/dir"] mod thing { mod child; }` makes `/abs/dir` the base the body's file-form children
  resolve against — and those bases were returned as bare paths carrying no record of how they were
  reached. The walk's own inheritance could not recover it: it threads provenance down the **file** chain,
  and this base is introduced within one file, so a conventionally declared child of the body was queued
  as not-absolute and its label relativized whenever the target happened to sit under the checkout's
  anchor. Measured across two checkouts of one committed literal, the identities differed —
  `crates/foo/src/inline_remapped/child.rs` in the checkout whose anchor contains the base, the absolute
  path in the one whose anchor does not — which is exactly the coincidence a label must not encode. The
  flag now rides with each inline base and accumulates through nesting. This was **reported once and
  wrongly refuted**: the refutation checked the file-chain inheritance, found it correct, and did not
  check the second path. Baseline effect is confined to that already-non-portable construct: an
  `unauditable-probe` entry for a file reached through an inline absolute `#[path]` base re-keys once.
- 圭表's inbound module rules now observe the **value namespace**, closing a recorded false negative.
  Rust resolves `mod foo` and `fn foo` in different namespaces, so both may be declared in one module and
  a single `use m::foo;` binds **both** — verified against rustc. The target match read only the import
  path, so it resolved to the module reading (`m::foo`, a descendant) and under `Shallow` anchored at `m`
  did not react, while the value reading reaches `m` itself and must: a real import of a protected module
  passed silently, the one class the Core Contract forbids. It is closed by consulting the value
  namespace, **not** by reacting on both readings — that would make every ordinary `use m::child;` react
  under `Shallow`, contradicting `rule-model-surface`'s exact-seam scenario, and the narrow false negative
  must not be traded for a broad false positive. An import now additionally reacts when its whole path
  resolves to a single-segment child module of the anchored module *and* that module itself declares a
  `fn`/`const`/`static` of the same name; when it declares only the module, nothing changes. Recorded
  honestly: the promotion trigger asked for "a value-namespace item observation guibiao does not have",
  and that premise was false — `symbol_scan`'s definition collector already read exactly those names, per
  module, at module top level, with the true-inline-module and top-level-only disciplines already worked
  out. The trigger was a missing connection, not a missing capability, which is the same misjudged cost as
  the absolute-`#[path]` entry earlier in this window. The residual is now the observation rather than the
  resolution and is stated in `rule-model-surface` with two scenarios: a value declared inside a macro
  body or arriving through a re-export is unobserved, matching every other declaration reader in the
  dimension, and directs the reaction toward the module reading. **BREAKING for a recorded baseline**: a
  constitution with an inbound `Shallow` boundary over a module that declares a `mod` and a value of one
  name will now report imports it previously missed, so such a baseline needs regeneration. No public API
  and no identity shape change.

  The marking follows the rule stated once at the head of this section rather than this entry's own
  reading of it. Both readings are defensible — "a false negative being paid back" and "an adopter's
  recorded baseline changes" — and an earlier entry in this same window took the second for the identical
  change to the identical rule family at the identical depth. Two same-shaped changes classified two ways,
  ~500 lines apart, is worse than either choice: the next contributor copies whichever they read first.
- **BREAKING (Windows only)**: every identity-bearing path label is now canonical rather than as the
  observing platform renders it. 星表 gains `path_label` — `/` as a path's only component separator,
  and every byte preserved — and both label sites are built on it: 圭表/渾儀's compilation unit
  (`compilation_unit_label`) and 漏刻's observed file (`audit::scan::probes::labeled`, whose private
  `encoded` is retired into the shared function). Previously each rendered its stripped path verbatim,
  so one commit yielded `unit: "src/lib.rs"` on Linux and `unit: "src\\lib.rs"` on Windows: a baseline
  recorded by CI matched nothing for a Windows contributor and every entry re-fired as new. That is the
  checkout-dependence class this window closed five times, along the one axis none of those five
  covered — not *where* the repository sits, but *which platform read it*. Separator interpretation is
  delegated to `Path::components()` rather than performed by substituting characters, and that is the
  whole of why it is correct: on unix `\` is a legal byte *within* a name (`library/std/src/sys/path/
  unix.rs` declares `path_separator_bytes!(b'/')`), so substituting it would map the single file `a\b`
  and the file `b` inside directory `a` onto one label — destroying the injectivity the label exists
  for — while on Windows both `\` and `/` separate (`.../windows.rs`: `path_separator_bytes!(b'\\',
  b'/')`), so substituting one is incomplete. **No unix baseline re-keys**: measured, every shape that
  occurs — `src/lib.rs`, `src/bin/x.rs`, `tools/outside.rs`, an absolute path, a `%`-bearing path, a
  non-UTF-8 path, and a unix backslash inside one file name — labels byte-identically to before. The
  Windows behaviour is argued from std's own source rather than executed: there is no Windows runner
  and no wine here, and the two tests covering it say so instead of letting a green unix suite imply
  coverage.
- Review also reported that `compilation_unit_label` conflated two `None` causes and so refused a
  non-UTF-8 crate root inside its package with a diagnostic naming a cause that is factually false.
  **Recorded as refuted rather than fixed**, measured four ways: `cargo metadata` under a non-UTF-8
  directory fails outright (`error: path contains invalid UTF-8 characters`, exit 101); an
  auto-discovered target whose file name is not valid UTF-8 is silently omitted from the target list; a
  `Cargo.toml` is UTF-8, so a `[[bin]] path` literal cannot spell such a path; and decisively,
  `src_path` and `manifest_path` reach 星表 as JSON **strings**, so any path built from them is valid
  UTF-8 by construction and `to_str()` cannot fail. The `None`-means-one-thing property still arrives,
  as a free consequence of the shared primitive being total. The byte-injectivity half of that
  primitive keeps its reason for existing in 漏刻, whose labels come from filesystem walks where such a
  name *is* reachable — which is why the rule is shared rather than dropped: the dimension where it
  cannot trigger can no longer drift from the one where it can.
- `xingbiao::crate_root_files` is now unique by root rather than by adjacency. Its doc promised roots
  "in Cargo's reported order, deduplicated" and it called `Vec::dedup` on an unsorted vector, which
  removes only *consecutive* duplicates — unlike `member_root_files`, which sorts first and is
  therefore total by construction. Two targets may legitimately name the same `path`: Cargo accepts it
  and builds both, and it reports targets sorted by **name**, so the two reports are adjacent only when
  no third target's name sorts between them. Measured against a real three-`[[bin]]` manifest, the
  function returned `[shared.rs, between.rs, shared.rs]`, and that root was scanned once per report.
  The consequence is narrower than a duplicated corpus suggests, and is recorded here rather than
  overstated: **no duplicate finding was reachable**, because both static dimensions already collapse
  violations by identity before reporting (`guibiao/src/lib.rs`, `hunyi/src/driver.rs`) — each for its
  own unrelated stated reason, two identical boundaries declared on one constitution. That is also why
  the duplication went unnoticed: a corpus defect was being underwritten by a dedup that exists for
  something else. The sibling-root exclusion was checked for the worse failure and does not have it —
  it keeps the current root explicitly, so a root appearing twice never excludes itself. Not breaking —
  no API signature, no identity shape, and no reported violation changes.
- Narrowed that dangling classification to genuine **absence**, and gated its test to the platform that
  can construct it. The refined condition was `std::fs::metadata(path).is_err()` — any metadata failure,
  not "the target does not resolve" — so a symlink whose target **exists but cannot be reached** (`EACCES`
  on a component of its path, or `ELOOP`) still reported "it is a symlink to X, which does not exist" and
  prescribed recreating a file already there. That is the same misdiagnosis one error kind further in, in
  the branch narrowed to fix it. Only `NotFound` is dangling now; anything else falls through to the
  arm that prints the real cause. Unlike the race the previous refinement documented, this one needs no
  race to construct: with the target inside a `chmod 000` directory, `lstat` reports a symlink, the
  `O_EXCL` open fails `EEXIST`, and `metadata` fails `EACCES` — so it is pinned by a third test arm rather
  than argued. Separately, only the test's `symlink` calls carried `#[cfg(unix)]` while its assertions ran
  everywhere, so on Windows it asserted about paths nothing had created and panicked on its first arm; the
  whole test is now `#[cfg(unix)]`, because a test that cannot construct its subject must not run rather
  than run and assert about something else.
- `--write-baseline` calls a symlink at the baseline path **dangling** only when its target really does
  not resolve. `create_baseline_file` is reached only when reading the path returned `NotFound`, so for a
  symlink the target was absent when the path was read — but it can come back before the `O_EXCL` open (a
  restored file, or the link replaced), and the branch classified on symlink-ness alone. It then told the
  adopter "it is a symlink to X, which does not exist" about a target that does exist, and prescribed a
  remedy ("recreate the target") already satisfied. Refusing was always safe; only the reason was false —
  the misdiagnosis class this window corrected twice elsewhere. Falling through loses no diagnostic:
  the `create_new && AlreadyExists` arm already reports that the baseline "appeared while the new
  snapshot was being prepared", which is exactly what happened. Not breaking — the exit code is `2`
  either way; only the sentence changes.
- `--write-baseline`'s atomic write applies the preserved mode to the **open descriptor**
  (`File::set_permissions`, an `fchmod`) rather than to the temp path. The temp file is opened with
  `create_new` (`O_EXCL`) precisely so nothing pre-planted at the predictable `<target>.tmp-<pid>`
  name can receive the write — and the next step then re-opened a variant of that same race by
  chmod'ing *by path*: anything able to write the baseline's directory (the access the `O_EXCL`
  reasoning already assumes) could unlink the temp file and plant a symlink at that name in between,
  and `chmod` follows a symlink, stamping the baseline's mode onto a file the attacker names.
  Measured in both halves: `chmod 0666` through a planted symlink moves the victim's mode to 0666,
  while an `O_EXCL` open against a planted name is refused outright — so the descriptor always names
  the inode this process created and there is no second name lookup to win. The mode application
  itself cannot simply be dropped: `O_CREAT`'s mode is masked by the process umask, so it can only
  narrow, and a 0666 baseline under umask 022 would otherwise be silently published at 0644 (verified
  under umask 0077, where the 0666 mode is still preserved). The resulting mode is identical either
  way, so the change was verified at the syscall — `chmod("<target>.tmp-<pid>", 0100666)` before,
  `fchmod(4, 0100666)` after. The failure-path `remove_file(&tmp_path)` is deliberately unchanged and
  is not the same exposure: `unlink` does not follow symlinks, so a planted symlink is itself what
  gets removed, never its target (verified). Not breaking — no API, no identity shape, and the
  published mode is byte-for-byte what it was.
- A 14th Definition-of-Done gate, `scripts/check_reference_integrity.sh`, asserts that every
  in-repository path a document or a source comment points at exists. This class has now been fixed by
  hand twice — the second sweep covered `*.md` only — and a module split landing after that sweep
  reintroduced it in **nine** places across documents and `.rs` comments, two of which no review found.
  A sweep is a snapshot; the next rename re-opens what it closed. The gate's two scope rules are drawn
  from what the repository actually contains rather than from a hand-written allowlist: a `crates/<name>/`
  reference is checked only when `<name>` is a real workspace member, so deliberately illustrative
  packages (`crates/a/src/lib.rs`, `crates/foo/src`) are skipped and a crate genuinely added under one of
  those names starts being checked the day it becomes a member; and scripts are out of scope because a
  script legitimately *constructs* paths that do not exist in the repository. The nine references are
  corrected, including `tests/dogfood.rs`, a file that exists nowhere and had been cited as the home of
  圭表's black-box tests. Existence is judged against **tracked content**, not the working directory, and
  a reference git deliberately ignores is skipped — the gate's first version tested the filesystem,
  passed locally, and failed in CI on three paths that exist in a developer's clone and in no fresh
  checkout, which is the same checkout-dependence class this window's identity labels were fixed for,
  landing inside the gate meant to catch drift. The ignore check passes the path with a trailing slash
  for the same reason: `git check-ignore` reads the filesystem to decide directory-ness, so for a
  directory-only pattern the bare form answers differently in a clone where the directory happens to
  exist (measured in a fresh clone — bare `.github/prompts` is not ignored, `.github/prompts/` is).
- The two new gates drop their remaining **GNU-sed** dependencies, which had the same failure shape the
  `realpath` fix removed from one of these files a commit earlier. `check_reference_integrity.sh` marked a
  markdown link target with `\x01` in a `sed` script; `\xHH` is a GNU extension that busybox and BSD `sed`
  emit **literally**, so under a POSIX `sed` the gate reported **9 stale references that do not exist** —
  measured, a portability failure surfacing as a repository defect. The marker is now a literal control
  byte built with `printf` and interpolated, which every implementation agrees on. A companion dead filter
  went with it: `grep -v '^\x01?$'` could never fire, because without `-E` the `?` is literal, so the
  pattern meant "SOH followed by a question mark" — its intent was already served by the guard inside the
  loop. `check_whitespace_hygiene.sh`'s `sed 's/\r$//'` likewise now uses a literal CR byte, since BSD
  `sed` does not interpret that escape on the left-hand side. `tr -d '\r'` was rejected as the fix despite
  being portable: it deletes mid-line CRs too, so `text\r\r\n` would stop being an offense at all —
  trading a portability bug for a false negative. Both gates were verified to give identical verdicts under
  GNU and busybox userlands, and — the check that matters more — to still *catch* a planted broken relative
  link, a planted broken prose path, and a planted trailing-whitespace offense under both.
- `scripts/check_reference_integrity.sh` normalizes a link target with portable shell instead of
  `realpath -m --relative-to`, whose `-m` and `--relative-to` are GNU coreutils extensions that BSD and
  macOS `realpath` reject. The script sits in a Definition of Done that states no platform restriction,
  and under `set -e` the unrecognized option would have exited with realpath's own status — landing on
  **1**, which in this gate's contract means "stale references found", so a portability failure would have
  been reported as a repository defect. Lexical normalization is not a compromise but the correct
  semantics: a markdown link resolves by path text, the target need not exist (a broken one must still be
  reported with its resolved form), and symlinks must not be followed — `realpath -m` was chosen for
  exactly those properties, and pure text has them by construction. Verified as a drop-in: the replacement
  agrees with GNU `realpath -m --relative-to` on all 35 markdown link targets in the repository and on 13
  adversarial shapes (repeated `..`, `..` escaping the root, `a//b`, `a/./b`, a bare `.`), and runs
  unchanged under `dash`. The first attempt did **not** agree — a `/`-joined accumulator silently stopped
  popping after the first `..`, so a doubly-ascending link came out with a doubled separator
  (`…//PROJECT.md`) instead of resolving to the root document — which is what the equivalence comparison
  was written to catch, and it differed on eight of forty-eight cases.
- `scripts/check_reference_integrity.sh` covers root-level references and stops swallowing read errors.
  Its extraction regex recognized only paths under `crates/`, `scripts/`, `openspec/`, `docs/`,
  `examples/`, and `.github/`, so 259 references to `PROJECT.md`, `AGENTS.self-law.md`, `BACKLOG.md`,
  `Cargo.toml` and their siblings never entered the check — a false negative against the gate's own
  stated purpose, one release-branch commit after it was added to prevent that class. Two unambiguous
  forms are added rather than "any bare filename", because a bare filename in prose is usually generic:
  a **markdown link target** is checked unconditionally (it is a link by syntax, resolved relative to the
  referring file as markdown means it), and a **bare filename** only when its basename is tracked at the
  repository root and nowhere else (`PROJECT.md`, `deny.toml` — but not `Cargo.toml`, `README.md`, or
  `spec.md`, which name the generic thing). Neither form can judge a reference to a document renamed
  *away*, since its basename is then tracked nowhere, so the repository's governance documents are
  asserted to exist as a **required set** — safe to write down where an allowlist would not be, because a
  required set fails the moment it goes stale rather than quietly excusing something. Separately, `grep`
  ran with `2>/dev/null` inside a process substitution, which hid exit 2 (cannot read) behind exit 1 (no
  match) where `set -e` could not see it: an unreadable tracked file counted as inspected and the run
  reported clean. The two exits are now distinguished and the unreadable case refuses to judge.
- `scripts/check_release_coherence.sh` now also requires every example's committed family-crate version
  requirement to be satisfiable by the workspace version. The script claimed workspace/dependency version
  alignment and never read `examples/*/Cargo.toml`, where all seven requirements pin the previous minor.
  The failure was already caught — `test_examples.sh` asserts each `patch.crates-io` override took
  effect — but by the dogfood job rather than by the gate that claims alignment, and it surfaced as a
  resolution puzzle rather than as "the release bump left the examples behind". Cargo *silently drops* a
  patch whose local version no longer satisfies the requirement, so those examples would resolve the last
  published family from crates.io instead of the tree under development. Both dependency forms are read
  (plain and `{ version = "…" }`), so one example moving to the table form is not skipped while the
  set-level vacuity guard stays satisfied by its siblings, and the family package names are read from the
  workspace so a seventh crate is covered the day it becomes a member. Three failure directions are added
  to `test_release_coherence.sh`'s matrix: a stale example pin, a missing `examples/` directory, and a
  table-form requirement that is stale.
- A 13th Definition-of-Done gate, `scripts/check_whitespace_hygiene.sh`, asserts over every tracked
  text file that no line carries trailing whitespace, no file ends on a blank line, and every file
  ends with a newline. `cargo fmt` governs `.rs` only, so nothing in the repository checked `.md`,
  `.toml`, `.sh`, or `.yml` — and three blank lines at EOF reached this release branch through 23
  touched spec files and two independent full-range adversarial reviews, one of which ran `cargo fmt
  --all --check` and reported it passing. Neither review had anything to consult, the property being
  stated nowhere and reacted nowhere; the three sites are corrected here. The gate then caught its
  first regression on its own first real use, and in doing so corrected the attribution: the blank line
  is appended by `openspec archive`'s sync step, observed directly when this window's next delta was
  synced. So the source recurs on every sync rather than being a one-off slip, which is exactly the kind
  of thing prose cannot hold and a reaction can. `git diff --check` was the
  obvious candidate and is the wrong shape for a gate — it answers about a *diff*, so its verdict
  moves with the base it is given, and run locally with no argument it sees only unstaged work
  (observed: with the three fixes staged but uncommitted, `git diff --check v0.3.0..HEAD` still
  reported all three). This asserts the invariant over the checkout instead. It reads the **worktree**
  with a line-terminating `\r` normalized away, and both halves are load-bearing: reading the index
  instead would tell a contributor who just added trailing whitespace that the tree is clean, since
  the offence is unstaged — a false negative, which no amount of platform-independence buys back —
  while reading the worktree raw reports every line of every file as trailing whitespace under
  `core.autocrlf`, because `\r` is a `[[:space:]]` character (measured: 7 offenses against the same
  content that yields 3 on Linux, a flood that gets a gate disabled rather than obeyed). Exit 0
  clean, 1 violation, 2 cannot judge — refusing to judge rather than skipping when a tracked path's
  own name would defeat the parse that finds it. Not breaking — no library surface, no identity
  shape, no adopter-visible behaviour.
- 圭表's crate rules no longer exempt a **same-named but externally-sourced** dependency as if it were
  the crate's own self-reference. The self-dependency exemption — which exists for Cargo's legal
  `main = { path = "." }` doctest/dogfooding idiom, a null-source edge naming no OTHER crate — matched
  by **name alone**, so a package `foo` declaring `foo = { git = "…" }` was swallowed by it. That is a
  real wrapper/fork/self-comparison pattern, and cargo reports it as an ordinary edge
  (`{"name":"foo","source":"git+…"}`, verified against a real `cargo metadata` read), so a genuine
  external dependency vanished from every rule reading the shared self-excluding observation:
  `forbid_dependency_on`, `restrict_dependencies_to`, `restrict_workspace_dependencies_to` (through
  `dependencies()`), `restrict_dependency_sources_to` (through `dependencies_with_disallowed_source()`),
  and the feature-granularity rules (through the dependency-edge selector) all passed it silently —
  the false negative PROJECT.md's core contract forbids outright. `deny_external_dependencies` was the
  one rule that already reacted, since its own observation never applied the exemption; that
  divergence between siblings reading the same table is what the fix removes. The exemption now
  additionally requires a **null declared `source`**, so the path idiom stays exempt while a
  same-named external edge reacts exactly as any other dependency of that name would.
  `crate-dependency-boundary`'s self-reference scenario is narrowed to the path form and gains its
  "same-named but externally-sourced dependency is NOT exempted" sibling, and the rule families that
  read the exempting observation are pinned against this edge by a test. Not breaking — closes a
  false negative; no identity shape changes and no baseline regeneration is needed. An adopter
  carrying this dependency shape sees a workspace that passed silently begin to react (exit 0 → 1),
  which is the reaction it should always have had.
- **BREAKING**: 漏刻's un-auditable-probe identity no longer embeds a raw absolute filesystem path.
  Reproduced directly: scanning the byte-identical file at two different absolute locations (the
  same relocation a different clone path or CI runner produces) yielded two DIFFERENT
  `unauditable-probe` identities, differing only in the `file` field's absolute prefix — a baseline
  recorded in one checkout matched nothing in another, so the accepted violation re-fired as new
  while the recorded entry was simultaneously reported stale. `file` is now labeled relative to an
  **anchor the caller supplies** — `audit_probe_coverage` and `audit_probe_coverage_with_markers` each
  take one, and the `tianheng` shell passes the workspace root Cargo itself resolves (the new
  `xingbiao::workspace_root`), the same directory whichever member manifest `--manifest-path` names.
  The anchor MUST be absolute: a relative or empty one is a constitution error (exit 2) naming the
  failed precondition and the value to pass instead, never accepted with a silently degraded label.
  Stripping a relative prefix from an absolute source path cannot succeed, and stripping an empty one
  succeeds while removing nothing, so either would leave every label in its raw absolute form —
  measured through the public entry point, anchors `"."`, `"crates"`, and `""` each returned the full
  absolute path. That is the checkout-dependent identity this whole change closes, reached through an
  argument that merely *looked* accepted, so it reacts rather than degrades. (An earlier revision of
  this entry documented the empty anchor as a deliberate "no anchor" opt-out; its effect is precisely
  the defect, so it is refused by the same rule instead.) Absoluteness is what the audit can check;
  being a true ancestor of the observed roots stays the caller's responsibility, and a file outside an
  absolute anchor still keeps its path as observed — the per-file fallback the absolute-`#[path]` bound
  below depends on. **Any existing `--write-baseline` output naming
  an `unauditable-probe` violation is now stale** (its `file` field's value changed shape) and must be
  regenerated; every previously accepted one reappears as new exactly once.
  That `file` label is also no longer produced by `Path::display()`, which is **lossy**: a platform path
  is a byte string, and `display()` replaces each undecodable byte with one replacement character — so
  two source files differing only in such bytes collapsed to one label, one identity, and one violation.
  Reproduced: two roots named `lib<0xFF>.rs` and `lib<0xFE>.rs`, each holding a non-literal probe,
  produced a single `unauditable-probe` finding labeled `crates/foo/src/lib\u{FFFD}.rs`, so baselining it
  suppressed a violation that was never accepted — the same injectivity class this window closed at five
  other identity sites, reached through an identity *component* rather than through the scan. The label
  is now an injective encoding: an undecodable byte becomes `%XX`, and a literal `%` becomes `%25` so no
  escaped label can be spelled by a path that needed no escaping (both directions pinned by tests). A
  path that is valid UTF-8 and contains no `%` — every realistic source path — is labeled byte-identically
  to before, so this widens nothing beyond the re-key already announced above; only a path containing a
  literal `%` changes.
  The anchor is a parameter rather than something the audit derives from the roots it is given, and
  that distinction is the fix rather than a detail of it: this entry's first cut computed the longest
  common prefix of every `source_inputs` root, which is checkout-independent but not
  **member-set**-independent, and so reopened the same loss through a second door. Reproduced
  directly: with every member under `crates/` the derived anchor is `<root>/crates` and a file is
  labeled `a/src/lib.rs`; adding one member outside that prefix — a tool, an example, a fixture crate
  — drops the anchor to `<root>` and relabels that identical file `crates/a/src/lib.rs`, so every
  entry recorded against the old label goes stale and re-fires as new at once, on a change that
  touched none of the observed files. An identity must be a function of the observed source and a
  stable anchor only, never of which other roots happened to be scanned alongside it, and only a
  caller knows a directory that stays put across both — so the derivation is gone rather than kept as
  a fallback, and `common_ancestor` with it. A single-crate scan's labels therefore name the file's
  place in the checkout (`src/lib.rs`) rather than its bare filename (`lib.rs`), which is also what
  keeps two members' same-named `lib.rs` apart.
  Stated bound: an ABSOLUTE `#[path = "/…"]` literal is a known residual gap, not fully closed by this
  fix — when its target does not lie under the anchor, the label falls back to the raw absolute path
  (`Path::join` discards its receiver for an absolute joinee); when it happens to lie under the
  anchor, the label becomes relative-looking instead, so the SAME hardcoded literal can still
  disagree across two checkouts. The workspace-root anchor does narrow that gap in one direction,
  pinned rather than assumed: an absolute literal pointing INSIDE the checkout is now labeled
  relative to it, where the derived per-file anchor left it absolute. An absolute literal is already
  non-portable on its own; the realistic relative sibling-share idiom this fix targets is
  unaffected either way.
- 渾儀's trait-impl-exposure `where`-clause bounded-type seam no longer keys an unrenderable bound
  (a complex const-generic argument, e.g. `Arr<{ N + 1 }>`) to the bare literal `_`. Reproduced
  directly: one impl block with two where-clause bounds that both fail to render
  (`Arr<{ N + 1 }>: AsRef<crate::infra::Secret>` and `Arr<{ N + 2 }>: AsRef<crate::infra::Secret>`)
  both fell back to `_`, so the two bounds' facts — identical kind, subject, and seam — collapsed
  to one; the two-bound fixture and either bound alone produced the byte-identical finding, meaning
  the second bound's violation left no trace a baseline could ever distinguish from the first. The
  fallback is now an internal positional sentinel (never itself published), caught by the existing
  fail-loud gate this capability already applies to its structurally identical cases (an
  unrenderable Self type or trait path), so an impl with this shape now reports a constitution error
  instead of silently under-counting its violations. A where-clause bound that renders cleanly (the
  ordinary case) is unaffected.
- 渾儀 now observes an `impl` block written as a direct statement of a `const` initializer's or a
  `fn`'s own body — the "const-eval trick" idiom (`const _: () = { impl Foo { … } };`, commonly used
  for a compile-time trait assertion or a doctest/dogfooding scratch impl) and its fn-body-nested
  sibling — instead of treating the whole body as opaque. Closes a false negative measured directly
  on ordinary, compilable source across six capabilities: signature-coupling, async-exposure,
  dyn-trait, and impl-trait all missed an inherent impl's method the moment its enclosing `impl`
  moved into such a body, and trait-impl-locality and forbidden-marker's hand-impl form both missed
  a trait impl the same way — the identical method or impl that reacted at a module's top level
  produced zero findings once wrapped, even though Rust binds an `impl` to its self type's coherence
  set regardless of where it is lexically written, so the wrapped impl was always real, externally
  callable public API. Three bounds are stated rather than left silent: only an `impl` block is
  recovered (a plain `fn`/`struct`/`mod` nested the same way stays exactly as unobserved as the
  existing body-nested-module bound already treats it); only an `impl` that is a DIRECT statement of
  the body's own outermost block is recovered, never one nested a level further (inside an
  `if`/`loop`/closure/nested `fn`); and only a `const` initializer or a `fn` body is inspected, never
  a `static` initializer. New violations are ordinary findings and absorbable by baseline.
- 漏刻's shipped default sink no longer silently discards a failed stderr write. An adopter who
  never calls `set_sink` — the exact adopter the default sink exists for — lost an enforce-severity
  `Violation` with zero trace whenever the write failed (a closed pipe after `myapp 2>&1 | consumer`
  exits, a daemon with closed inherited fds, or plainly `myapp 2>&-`): the process correctly did not
  crash, but nothing recorded that the reaction had even fired. A failed write is now counted by a
  new public `louke::dropped_sink_events() -> u64`, a single lock-free atomic add that cannot itself
  fail or panic, so an adopter can poll it into their own health check or diagnostics endpoint to
  detect the loss from outside the process. Scope stays narrow: a custom sink's own success or
  failure is opaque to the system (`set_sink` takes a `Fn(&Violation)` returning nothing) and is
  never counted. Additive, non-breaking — the only public surface change is the new function.
- **BREAKING**: 漏刻's runtime **origin is now derived from the registered type**, and its expansion
  target `OriginEntry::__from_register_origin` (renamed from `OriginEntry::new` in this same window)
  takes **no arguments** — it is generic over the type and derives the type identity, the origin, and
  the type name from it.
  This closes a false negative, not a wording problem. The previous constructor took three ordinary
  values, so a hand-written call fabricated all three at once and registered a rogue type under an
  allowlisted origin; the crossing then passed with no reaction. Reproduced in-tree before the fix and
  re-derived by an independent review. With nothing left to pass, an origin a type does not have is
  **unrepresentable** rather than detected.
  An origin is therefore the module the type is **defined** in, not the module the registration call
  sits in. Earlier in this window the same entry described the gap as a deliberate, cooperative
  process trust boundary and pinned it with a test; that description is retired here, along with the
  test. What made the closure possible was giving up on finding a *macro* that can pass something
  hand-written code cannot — a proc-macro is expanded into its caller's crate and resolved there
  exactly as a `macro_rules!` is (`E0603` at the consumer's own call), so no macro form has privilege
  its caller lacks. The origin had to stop coming from the caller at all.
  **Migration is narrower than it sounds.** `register_origin!(MyType)`'s spelling does not change, so
  no adopter edits source. A registration written **inside the type's own module** — the documented
  idiom, and what `examples/composed` does — derives the *byte-identical* origin string it derived
  before, so its `only_origins(...)` entries need no edit; this is machine-checked by
  `a_registration_can_only_name_its_own_types_defining_module`, which asserts the derived origin equals
  `module_path!()` at the defining module. A registration written **outside** the type's module now
  names the type's own module, so an `only_origins(...)` entry naming the registration site must change
  to the type's defining module; until it does, the seam reacts fail-closed and the finding names both
  the observed origin and the concrete type. A type from another crate now carries that crate's own
  origin: to give an adapter your layer's origin, define the adapter in your layer (a newtype), which
  is also what actually crosses the seam.
  **No baseline is affected.** An observed origin never reaches a `Report` or a baseline — the prod
  reaction's `Violation` goes to the sink, every CI-face fact carries no origin, and the runtime
  `RuleKey` is built from the **declared** allowlist, which this does not touch. The probe hot path is
  unchanged: `&'static str` origins resolved once at startup, no lock, no allocation per crossing, no
  new dependency. The CI face (`audit_probe_coverage`) audits seams and probes and never observed an
  origin, so it is untouched.
  Stated bounds, measured rather than assumed: a type defined in another crate reports that crate's own
  defining path, which may be a private internal module (`std::collections::hash::map`) rather than the
  public re-export path; a type defined inside a function body reports a path qualified by that
  function, which is not a module path; a generic type's arguments are not part of its origin (the
  argument list is cut before the final separator is sought, because an argument can itself contain
  separators) — which in the direction that matters means a generic **defined in an allowed module
  carries that origin whatever it wraps**, so an instantiation whose argument comes from a forbidden
  module crosses as allowed; that is the bound of observing an origin as a *module* rather than a type,
  it is stated rather than left to be inferred from the deduplication reading, and it is pinned by a
  test. A type alias reports the aliased type's defining path, so an alias cannot relabel an
  origin; a shape carrying no path at all (`u8`, `&str`) yields its own rendering, there being no `::`
  to find; and a **composite** shape wrapping a pathed type (`&m::Foo`, a tuple, an array, a pointer, a
  fn pointer) yields a *truncated* rendering, the argument cut being delimiter-aware for `<…>` alone.
  None of these is a new fail-loud class: each matches no allowlist entry and therefore reacts
  fail-closed, which is the safe direction and needs no separate gate.
  For the record, the residual this replaces was **CI-preventable** for a Tianheng-governed workspace
  and the prose never said so: 圭表's `must_not_call_inline("louke::OriginEntry").strict_external()`
  reacts to the hand-written bypass — measured across the plain path, the leading-`::` spelling, and an
  alias import — though `.strict_external()` is required, since the default resolver does not classify
  an external crate's paths. That is history now, not a recipe; and it never covered a third-party
  dependency, which is why the crate's own guarantee could not rest on it.
  `runtime-origin-assertion` replaces its origin-observation requirement (the premise changed, not the
  wording) and adds three: the derivation, its stated shape bounds, and an explicit requirement that
  the allowlist match stays **equality** — pairing this with prefix matching would let a type defined
  under a descendant module newly pass a seam that reacts today, the same forbidden bug arriving
  through the matcher instead of the observation.

### Migration

Every step below is already stated by the entry it comes from; this section only collects them, so an
adopter reads the work in one place instead of assembling it from the `**BREAKING**` entries above. The
count is deliberately not named: it said "five" while the window grew to fourteen, which is the same
"a number in prose beside a growing thing drifts by construction" failure this window cited when it
dropped rather than corrected a test-suite size elsewhere. Dropping it is the fix, not recounting it.

- **Regenerate any recorded baseline.** Violation identity gained a required field at several sites this
  window — the count is not named, for the reason the section intro gives: `governing_package` on every 圭表
  and 渾儀 module/semantic fact; the compilation unit on every one of them too; the declaring module on
  inherent method/associated-item seams; the declaring module on impl-generics and extern-crate seams; the
  **bounded thing** on impl-generics seams; the importing module on outbound module facts; and the
  checkout-relative `file` label on every 漏刻 un-auditable probe. `--write-baseline` carries
  `owner`/`tracker` annotations forward only where an entry's identity still **matches** (the merge is
  keyed on identity), and each of these identities changed shape, so annotations cannot cross
  automatically: preserve them externally, rewrite the baseline, then restore them onto the newly
  observed facts. Every previously accepted violation reappears as new exactly once. A sixth case
  affects only some adopters: a trait-impl-locality boundary declared through a **facade `pub use`
  spelling** now identifies by the trait's resolved defining path, so its entries move too — a boundary
  that already named the defining path is untouched.
- **Expect new entries, not only relabeled ones, for an inbound rule at `ScanDepth::Shallow`.** That
  depth now reacts to imports it silently passed before, from **two** independent causes, so a regenerated
  baseline can legitimately be larger than a relabeling would explain: an item-form import, whose path is
  resolved to the module it denotes before the depth comparison; and an import binding a **value** the
  protected module declares under a name it also gives a child module (`mod foo` beside `fn foo`), which
  the path alone cannot distinguish from an import of the child. Both are false negatives being paid
  back.
- **Expect new entries from causes outside `Shallow` too.** Changes beyond that cell enlarge a regenerated
  baseline for **every** rule family: a violation written in a
  package's second compiled root (a `main.rs` beside a `lib.rs`, any `src/bin/*.rs`) is now observed at
  all, and an outbound rule now reports each importing module separately where it previously collapsed
  several into one. Both are false negatives being paid back, so the new entries are findings that were
  always true and never reported.
- **Your build may now refuse to judge where it previously answered.** Several shapes became constitution
  errors (exit 2) rather than silent or collapsed answers — enumerated rather than counted, for the reason
  the intro gives — and each names its own remedy: an owner label whose head admits more than one distinct
  type under mutually-exclusive `#[cfg]`; a package target rooted **outside** its own package directory,
  which has no checkout-independent label; a 漏刻 audit anchor that is relative or empty; a
  forbidden/allowed operand carrying an empty `::` segment (a leading, trailing, or doubled `::`, or the
  empty string) across the six DSL methods that take one — `must_not_expose("::serde")` previously exited
  0 and never reacted, and now refuses; and, for a **standalone 渾儀** consumer, a plain `mod name;` backed
  by both `name.rs` and `name/mod.rs`, where the composed `tianheng check` already exited 2. Exit 2 is
  "cannot judge", not "violation" — a gate that hits one needs the declaration or layout corrected, not a
  baseline.
- **Some recorded entries go stale rather than move, and `--disallow-stale` fails on them.** Every step
  above concerns an entry appearing or re-keying; this window also **narrowed** reactions, so a violation an
  adopter legitimately baselined may now be gone. 圭表's strict-external local-precedence ladder now sees a
  value the collector previously missed and lets it claim the local bare head: one declared inside an
  `extern` block, and one declared `static mut`. So a bare `rand()` call resolving to a local
  `extern "C" { pub fn rand(); }` **or** to a local `static mut rand` is no longer read as reaching a `rand`
  dependency — each measured, one violation before and clean after. And the inbound self-import exemption
  now holds at `Shallow` as well as `Subtree`, removing inbound violations from files inside the protected
  subtree. A stale entry is not an error by itself — but `--baseline … --disallow-stale` turns an otherwise
  clean run into **exit 1** on one. Prune the entries the fresh snapshot no longer produces, which the
  regenerate step above already does if you take it wholesale.
- **On Windows, regenerate even if nothing else applies to you.** Every identity-bearing path label is now
  canonical — `/` as its only separator, whatever the observing platform uses — so a baseline recorded on
  Windows before this release carries `src\lib.rs` where the label is now `src/lib.rs`, and every entry
  re-fires as new. On unix no label changes and this step is a no-op. This is the one **BREAKING** entry
  whose step this section previously omitted, which is exactly the failure the completeness claim above
  invites: an adopter following the section alone would have regenerated nothing and met a wholly
  new-looking baseline.
- **If you build a 漏刻 `OriginEntry` by hand**, switch to `register_origin!`. The constructor is renamed,
  hidden, and now takes no arguments at all; the macro is, and always was, the supported path.
- **If you write `register_origin!` outside the type's own module, check your `only_origins(...)`.** An
  origin is now the module the type is **defined** in, so an entry naming the registration site must
  change to the type's defining module. A registration inside the type's own module — the documented
  idiom — needs no edit at all: the origin string is byte-identical. Registering a type from another
  crate now carries that crate's origin; wrap it in a newtype defined in your own layer instead.
- **If you call 漏刻's audit directly**, pass the new anchor argument to `audit_probe_coverage` /
  `audit_probe_coverage_with_markers`: the workspace root, via the new
  `xingbiao::workspace_root(&metadata)` for a Cargo workspace. It must be **absolute** — a relative or
  empty anchor is refused with a constitution error (exit 2) rather than silently producing
  checkout-dependent labels, so a caller that passed a path built relative to the working directory
  needs to make it absolute rather than adjust to a new label shape.
- **If your constitution names 渾儀's signature-coupling builder types**, rename `SemanticBoundary` and
  its draft chain to `SignatureBoundary` (`SignatureCrateDraft` / `SignatureModuleDraft` /
  `SignatureBoundaryDraft`). No rule string, wire, or CLI name changes with it.
- **If a CI invocation combines `--write-baseline` with `--warn-uncovered` or `--format`, drop the
  inapplicable flag**, and likewise keep only one occurrence of a value-taking flag passed twice. Both
  were previously accepted and silently ignored; both are now usage errors (exit 2), so an invocation
  that appeared to work will now fail loud until the dropped flag is removed.

## [0.3.0] - 2026-07-26

### Documentation
- Archived historical 0.1.x–0.3.0 shipped backlog ledgers into `docs/history/0.1.0-0.3.0-built-ledger.md` and pruned `BACKLOG.md` to optimize context gravity.
- Reconciled the 0.3.0 migration guide with the separately shipped testing harness and restored
  the deferred baseline debt-ratchet WATCH decision after backlog pruning.

### Added
- Semantic `RuleKey` and `StructuredFactIdentity` inspection across 圭表, 渾儀, 漏刻, 璇璣, and
  `tianheng::prelude::*`; all three instruments remain directly adoptable and return the same
  structured reaction model.
- Explicit machine-contract formats: `tianheng.baseline/structured-facts`,
  `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries`.
- `tianheng::testing::GovernanceTest`: a reusable fluent architecture-test harness for clean
  reactions, complete workspace-member coverage, projection freshness with explicit
  `BLESS=1`/`BLESS=true` regeneration, and negative fixture checks. Tianheng's own self-law
  dogfoods the same public projection gate.
- `ScanDepth::{Shallow, Subtree}` and explicit `.depth(...)` controls on supporting 圭表 and 渾儀
  boundaries. Legacy module boundaries retain subtree evaluation and identity; shallow scope is
  projected explicitly.
- `check --disallow-stale` turns any stale baseline entry into a gate failure while preserving
  constitution-error precedence and consistent text, JSON, and SARIF exit semantics.
- `ImplTraitBoundary::including_submodules()`: an opt-in subtree scope for the impl-trait
  (existential RPIT) boundary, mirroring `AsyncExposureBoundary`'s existing depth. Defaults off;
  an existing boundary projects and reacts byte-identically.
- `NoExistentialLeak` / `Constitution::no_existential_leak(...)`: a composed profile folding
  impl-trait's written `-> impl Trait` and async-exposure's implicit `impl Future` — the two
  existential-leak signals — into one declaration, mirroring `SansIoPure`. Each composed boundary
  keeps its own separate identity; adds no new reaction.
- `louke::audit_probe_coverage_with_markers(...)`: CI probe-coverage audits can recognize
  adopter-defined probe macro names while `audit_probe_coverage(...)` preserves
  `assert_boundary!` as the compatible default.

### Fixed
- 圭表 now union-scans every physically existing path candidate when one module declaration mixes
  direct `#[path = "…"]` and conditional `cfg_attr(..., path = "…")` remaps, independent of
  attribute order. A candidate selected by a real rustc configuration can no longer disappear
  behind the scanner's former direct-path early return.
- 漏刻's un-auditable-probe identity now includes the full enclosing lexical function chain, so
  byte-identical probes in same-named nested functions or local contexts under different outer
  functions remain distinct and baselining one cannot suppress another.
- Module-boundary constitution projection now omits the legacy/default subtree scan depth and emits
  the non-legacy shallow depth, preserving old projection bytes while exposing the real opt-in.
- 漏刻's un-auditable-probe finding identity is no longer file-granular: it is now qualified by the
  offending non-literal seam expression's own source text and its owner-qualified enclosing item
  (module path plus `fn`/`impl`/`trait` context), so two distinct non-literal probes in the same
  file react as distinct findings and baselining one can no longer mask another. A false-negative
  closure (a patch, per the standing v0.1.3 re-export-exposure precedent): an existing baseline
  with an un-auditable-probe entry goes stale and needs `--write-baseline`, never silently
  reinterpreted. Two byte-identical expressions in the same file and the same enclosing item still
  collapse to one finding — a stated bound.
- 圭表 now preserves `cfg_if!` bodies as transparent control-flow wrappers, so enclosed imports,
  module declarations, and inline symbol calls remain observable instead of being stripped as
  macro-generated code.
- 圭表's `must_not_import` now fails closed on ancestor glob hazards such as `use crate::a::*;`
  when `crate::a::b` is forbidden, while unrelated and non-glob ancestor imports remain clean.
- 渾儀's signature-coupling alias resolver now walks nested nominal targets in non-generic tuple,
  array, slice, reference, raw-pointer, group, and parenthesized aliases.
- 圭表 now normalizes embedded `self` and `super` segments throughout grouped imports and inline
  symbol paths before evaluating module boundaries.

### Changed
- **Breaking:** violation and baseline identity is now exactly governed target + semantic rule key
  + structured fact identity. Rule/finding wording and all diagnostics remain available but cannot
  affect matching, ordering, or SARIF fingerprints.
- **Breaking:** SARIF partial fingerprints now use `tianheng/structured-fact-identity`, derived
  solely from canonical semantic identity; `tianhengViolationId/v1` is no longer emitted.

### Removed
- **Breaking:** `FindingKey`, presentation-derived `ViolationId` construction, numeric baseline
  generations, legacy text matching, and automatic baseline upgrade behavior.

### Migration
- Preserve desired `owner` / `tracker` annotations externally, move or delete the old baseline,
  run `tianheng check --write-baseline <file>`, then restore annotations onto the newly observed
  facts. Unsupported existing files are never overwritten. There is no automatic adapter.
- Architecture tests should call an existing standalone `check*` function or
  `check_constitution`, then assert on `Violation::target()`, `Violation::rule_key()`, and
  `Violation::fact()`. The identity migration adds no plugin protocol; the separately specified
  `tianheng::testing::GovernanceTest` harness is available for repository architecture tests.

### Compatibility evidence
- Pacta `d3e24df`'s unpublished `pacta-governance` consumer compiled against this checkout's local
  `tianheng` and `guibiao` crates (`cargo check -p pacta-governance`) from a temporary copy; no
  Pacta source migration was required. This is recorded historical external evidence, not a
  sibling-repository dependency of Tianheng's required CI. Ongoing local reaction is provided by
  the external-view `tianheng` and `guibiao` `adopter_surface` tests; those fixtures protect the
  corresponding public call shapes without claiming to re-verify that external commit.

- Refined core project documentation density (`PROJECT.md`, `BACKLOG.md`) to archive verbose
  historical post-mortems and prune redundant release ledgers, reducing context token overhead.

## [0.2.3] - 2026-07-22

### Fixed
- 渾儀's forbidden-marker self-type resolver (`resolve_self_type`) now routes through the crate's
  own hop-capped alias/re-export fixpoint instead of a second, hand-rolled loop guarded only by an
  exact-repeat check — closing a real unbounded-loop gap (a divergent, non-cycling alias rewrite
  chain the exact-repeat guard alone cannot catch) and, as a side effect, an alias-resolution false
  negative (a member reached through an aliased *prefix*, not just an exact alias key, now lands).
- 圭表 now reacts (a constitution error) when a plain `mod x;` resolves to BOTH `x.rs` and
  `x/mod.rs` at once — a genuine `rustc` compile error (E0761) it previously accepted silently as
  two separate sources, dual-governing one module path. Matches 漏刻's own probe scanner, which
  already reacted on this exact shape.
- 渾儀's single-module-anchored resolver (`descend`) now tolerates a `#[cfg]`-gated `mod x;` with
  no backing file, matching its own crate-wide walker's (`resolve_child_modules`) existing policy —
  the two previously disagreed, so a boundary anchored directly at a `#[cfg]`-gated module hard-
  failed even when a mutually-exclusive per-platform sibling (e.g. an inline arm) legitimately
  resolved it.
- 漏刻's CI probe-coverage scanner now canonicalizes its module-cycle dedup guard (via a new,
  additive `xingbiao` dependency gated behind the non-default `audit` feature — never reaches the
  production hot path), matching 圭表/渾儀's own guards. Previously deduped on the literal path
  only, so a symlinked directory or circular `#[path]` chain reached via two distinct literal paths
  to the same real file could make the scan misbehave instead of terminating cleanly.
- 漏刻's CI probe-coverage scanner no longer tolerates a missing conventional module file merely
  because the item carries ANY `#[cfg]`/`#[cfg_attr]` attribute. Verified against a real `rustc`
  build: unlike a bare `#[cfg(pred)]` (which genuinely removes the item when `pred` is false),
  `#[cfg_attr(pred, …)]` never removes the item — only conditionally applies its wrapped
  attribute — so a `#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a real,
  unconditional compile error (E0583) that was previously silently skipped by the audit.
- 圭表 and 渾儀 now tolerate a missing unconditional `#[path]` target when the item also carries a
  co-occurring bare `#[cfg(pred)]` — a standard per-platform shim (`#[cfg(windows)] #[path =
  "windows_impl.rs"] mod imp;`) that previously hard-failed on any platform whose target file
  wasn't committed, even though rustc itself strips the whole item, `#[path]` included, before
  ever resolving it when `pred` is false (verified against a real build).
- 圭表 now reacts (a constitution error), rather than silently dropping the module from
  `reachable`, when a plain `mod x;` with no backing file carries no `#[cfg]` at all — closing a
  longstanding cross-dimension coverage gap (渾儀 already hard-erred on the identical shape). A
  `#[cfg]`-gated missing file is still tolerated, matching 渾儀. A boundary anchored directly at a
  module whose sole declaration was `#[cfg]`-tolerated away now reacts as an unknown module
  (never a vacuous clean pass), matching 渾儀's own resolver's identical precedent — unless an
  inline sibling arm of the same name exists, in which case the self-describing inline-target
  error still applies (never misreported as a generic "unknown module, check the path" error).
- 圭表's and 漏刻's independent `#[path]`-string decoders now handle backslash-newline line
  continuation (`"a\` + newline + `b"` decoding to `"ab"`), matching `syn` (used by 渾儀) and real
  `rustc` behavior. Previously 圭表 silently dropped such a remapped module from `reachable` with
  no error, and 漏刻 fell back to (or hard-errored on) the conventional location instead of
  following the real target.

### Changed
- Internal refactor: modularized crate internals across `xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, and the `tianheng` runner's projection layer (deduplicated JSON/text boundary-projection rendering) — no public API, JSON wire format, or self-governance boundary changed.

## [0.2.2] - 2026-07-22

### Fixed
- 圭表 module reachability now walks into an inline `mod parent { … }` body to find its own
  file-backed declarations, so a child reached only through an inline parent (`mod parent { mod
  child; }`, compiling `parent/child.rs`) is observed and its imports are checked.
- 圭表 now follows an unconditional, direct `#[path = "…"]` module declaration to its real target
  (matching 渾儀 and 漏刻), so a relocated module's imports are observed by all three observation
  dimensions. A `cfg_attr`-wrapped `#[path]` remains excluded (cfg-conditional, never followed
  cfg-blind).
- Every declared source for a module name is now observed, cfg-blind: an inline module body's own
  nested declarations, a plain conventional file, and an unconditional `#[path]` remap of the same
  name under mutually-exclusive `#[cfg]` arms (the standard per-platform shim) are all governed,
  regardless of attribute order or which source is scanned first. A plain (`#[path]`-free) `mod
  child;` declared inside a file reached through an unconditional `#[path]` remap is now governed
  under its logical path.
- A `#[path]` inside one mutually-exclusive `#[cfg]` arm's target — or inside a plain child of that
  arm — that legitimately references a sibling arm's own target (the two are never simultaneously
  open in any real build) is no longer misreported as a module cycle. Plain-child resolution now
  tracks each source's own directory context (where a `#[path]` written in it resolves, and
  separately, where its own plain/inline children live) instead of resolving through a shared
  structural index.
- A plain child reached only through a **symlinked directory** component, and an inline module
  preceded by an unconditional `#[path]` header (which relocates the base its own file-form
  children resolve from), are both now followed and governed correctly.
- 渾儀's single-module resolver (backing signature-coupling, visibility, dyn/impl-trait, and
  async-exposure anchors) now unions every mutually-exclusive `#[cfg]` variant of a module — inline
  and file-form alike — instead of stopping at the first match, and resolves a segment nested
  beneath a split point, or a `#[path]`-loaded module's own conventional children, from that
  variant's own directory rather than a name-derived or shared one. Two `#[cfg]` arms plainly
  declaring the identical name (resolving to one real file) are deduped by canonical path so they
  never inflate one violation into two.
- A `use`-map, and the child-module/re-export/rename tables it depends on, are now computed **per
  branch** of a `#[cfg]`-split module rather than once over the flattened cross-branch union —
  closing false negatives where one branch's own `use` alias or genuine re-export was silently
  shadowed or overwritten by an unrelated, mutually-exclusive sibling branch. Two purely-inline
  `#[cfg]` siblings sharing one enclosing file are split into their own branches for this purpose,
  not just file-form ones.
- A finding's reported `file` is now attributed **at collection time**, carried from the exact
  `#[cfg]` branch that produced it, rather than re-resolved afterward from a module-path string —
  so a violation written in a non-first branch is reported at its own file, never an innocent
  sibling's.
- The subtree walker backing `.including_submodules()` now descends every surviving `#[cfg]` branch
  independently, each from its own resolved `#[path]` base, instead of collapsing several branches
  to one shared directory pair for further descent.
- A self type that resolves to the enclosing `impl`'s own declared generic type parameter —
  written as a bare identifier, a projection (`T::Assoc`), or a qualified path (`<T>::Item`) — is
  no longer resolved through a same-named `use` alias, in both the forbidden-marker acquisition
  gate and the trait-impl-locality owner label. This closes a false-positive marker finding and a
  dedup-collapse false negative where two distinct `MisplacedImpl` violations were silently
  reported as one.
- `async_exposure`'s subtree scan now assigns a continuously-incrementing ordinal across the whole
  walk, never reset per module — closing a dedup-collapse false negative where two
  mutually-exclusive `#[cfg]` branches of one async fn, each carrying an unrenderable const-generic
  self type, collided on the same fallback identity and were reported as a single finding.
- 漏刻's probe-coverage scanner now locates a `mod` declaration's own attribute preamble with a
  forward, literal- and attribute-group-aware scan, replacing a backward raw-byte scan that could
  desync on a bare `;`/`{`/`}` inside an earlier attribute's string value or a brace-delimited
  attribute argument — closing false hard-fails and wrong-file substitutions on valid, compiling
  code.
- 圭表's crate-boundary rules (`forbid_dependency_on`, `restrict_dependencies_to`,
  `restrict_workspace_dependencies_to`, `restrict_dependency_sources_to`, and the
  feature-granularity rules) no longer observe a crate's own self-referential dependency on
  itself — a real, Cargo-legal pattern (e.g. a `[dev-dependencies]` path dependency on `.`, used
  for doctest/dogfooding) that names no other crate at all, so it can never be the cross-crate
  concern any of these rules exist to govern. The exclusion lives in the shared dependency
  observation itself, so every crate rule is covered at once.

## [0.2.1] - 2026-07-21

### Changed
- Published finding schemas and their dimension-local canonicalizers are now exhaustively pinned as
  compatibility reactions. Human finding wording remains presentation and is deliberately not
  snapshot-frozen.
- The baseline guide now documents the existing `--write-baseline` operation as the bounded,
  explicit V1-to-V2 upgrade path, including metadata carry-forward and stale-entry removal.
- 圭表 `must_not_import` now documents a stated partial-coverage bound: a `use`-glob of an
  *ancestor* of the forbidden module (`use crate::*;` while forbidding `crate::secret`) is observed
  at the glob's base, not as the forbidden descendant edge, so it does not react — forbid or confine
  the parent. The narrow `use crate::secret;` / `use crate::secret::*;` forms are caught as before.

### Fixed
- 渾儀 unsafe-confinement now qualifies a **trait-impl** `unsafe fn` by `<trait for self>`
  (`unsafe fn <A for Foo>::m`), not its self type alone: on one self type, an inherent `unsafe fn m`,
  `impl A for Foo { unsafe fn m }`, and `impl B for Foo { unsafe fn m }` are three distinct sites and
  now stay three findings. Previously all collapsed to `unsafe fn Foo::m`, so a baseline of one
  silently accepted a later-added trait-impl `unsafe fn` on a safe trait — a false negative, the
  trait-impl case 0.2.0's notes already claimed owner-qualified. *Baseline note:* this changes the
  `finding_key` of a trait-impl `unsafe fn`, so a 0.2.0 baseline entry for one resurfaces on upgrade
  and must be re-accepted (`--write-baseline`); unsafe-confinement is one release old, so the
  affected surface is minimal.
- Baseline `owner` / `tracker` metadata now rejects non-string JSON values instead of silently
  erasing malformed governance data; the CLI gate fails as a constitution error and explicit
  rewrite retains its warning-before-recovery behavior.
- Runtime probe coverage now starts from every exact Cargo library and binary target root and walks
  only module-reachable source, so an orphan `.rs` file can no longer satisfy a seam it never
  enforces. Direct callers that pass a directory retain the legacy recursive corpus.
- 渾儀 and 漏刻 now **follow** an unconditional `#[path = "…"] mod x;` to its author-chosen file,
  closing a coverage false negative: a relocated module's `unsafe` sites, trait impls, and
  `assert_boundary!` probes were previously dropped, so a disallowed impl or an undeclared-seam probe
  in a relocated module passed unobserved (semantic single-module boundaries on such a module errored
  loudly rather than governing it). The target is resolved with rustc fidelity — relative to the
  containing file's own directory, accumulating each enclosing inline-`mod` name as a directory
  component (so `mod inline { #[path="p.rs"] mod inner; }` reads `inline/p.rs`), with the path
  literal's escapes decoded as rustc and syn do; the two independent dimensions resolve the same
  file, and two declarations sharing one target (or a conventional `mod` plus a `#[path]` alias to
  it) are governed under each path rather than misread as a module cycle. A `#[path]`-loaded file is
  mod-rs-like, so its own children resolve from its directory. A `cfg_attr`-wrapped `#[path]` stays a
  stated bound — not followed cfg-blind, since it could observe a file rustc does not compile in this
  configuration — and an absent unconditional target is a fail-loud constitution error. Both
  dimensions detect the attribute structurally, so an incidental `path` substring in a comment or a
  `#[cfg(feature = "fastpath")]` gate is never mistaken for a relocation. As with any false-negative
  closure, a downstream carrying a real violation inside a relocated module may see green CI turn red
  on upgrade — adopt via `warn` / `Baseline` (the same patch-level precedent as the v0.1.3 re-export
  closure).
- The probe-coverage walker now tolerates a `#[cfg(...)]`/`#[cfg_attr(...)]`-gated module whose file
  is absent in the current configuration (an off feature or another platform), skipping it instead
  of failing the audit — matching the semantic dimension, so a cross-platform workspace no longer
  hard-errors on a platform-specific module. A non-cfg missing module and a resolution ambiguity
  remain fail-loud.

## [0.2.0] - 2026-07-20

The first **breaking** window since `0.1.0` — a deliberate `0.2.0` minor (the `0.1.x` hold ended
when real adopters arrived). The break is quarantined to internal identity/model surfaces; the
adopter-written builder is a drop-in swap (see **Compatibility**).

### Added
- **`tianheng::check_constitution`** — one inspectable composed reaction over the static (圭表),
  semantic (渾儀), and runtime (漏刻) dimensions in a single call, sharing the runner's evaluator
  (static-first error precedence, runtime orphan-probe auditing) without going through the CLI.
- **Adopter surface contract.** The composed wildcard `prelude` is now an explicit,
  compile-checked external compatibility promise, with a symmetric `ModuleRule` inspection path;
  hidden granular checks stay outside the promise.

### Changed
- **BREAKING — structured violation identity.** Violation matching moved from rendered finding
  *text* to dimension-owned **structured keys**: `Violation::new` now takes a typed `ViolationId`,
  and newly-written baselines use version-2 `finding_key`s (fact-specific named fields) instead of a
  rendered descriptor. 渾儀's semantic findings derive both their diagnostic text and their key from
  one typed fact model. Reports stay byte-identical.
- **BREAKING — 圭表 rule model surface narrowed.** `Rule` / `ModuleRule` are now
  builder-constructed only — downstream can no longer construct or exhaustively destructure their
  data-carrying variants (open-ended *inspection* stays available through the boundary accessors);
  `InlineExternalStrict` is folded into `Inline`. Reaction, projection, polarity, and violation
  identity are unchanged.

### Fixed
- 渾儀 unsafe-confinement: `unsafe fn` findings are now **owner-qualified** (`unsafe fn {owner}::{m}`)
  for inherent, trait-declaration, and trait-impl methods, so two same-named `unsafe fn`s on
  different owners in one out-of-subtree module no longer collapse to one finding — closing a
  baseline-masking false negative (the `unsafe fn` sibling of 0.1.8's `unsafe impl` closure).
- 圭表 inline-symbol-path confinement (`must_not_call_inline`): a `use`-group member whose name
  merely *starts with* the substring `self` (e.g. `use chrono::{self_utc as clk}`) is now resolved
  rather than dropped, so a confined inline call through such an alias reacts — closing a false
  negative.
- 渾儀 single-module resolution: a module split across `#[cfg(…)] mod x { … }` **inline variants** now
  has every variant governed (matching the crate-wide scan's observe-all), so a forbidden exposure
  in a non-source-first variant reacts — closing a `mod`-resolution false negative.

### Compatibility
- The **adopter-written builder** (`Constitution`, `CrateBoundary`, `ModuleBoundary`, the boundary
  DSL, `run`, `prelude`) is a **drop-in swap** — the break is quarantined to the internal
  `Violation` / `ViolationId` / baseline wire and 圭表's rule-model surface.
- **Baseline migration.** Version-1 baselines are still read (exact-text match), so existing
  baselines keep grandfathering; a baseline rewritten under this release upgrades to the version-2
  structured form.

## [0.1.10] - 2026-07-15

### Added
- 圭表 **feature-granularity crate-dependency boundary** — `CrateBoundary::crate_(…)`'s
  `restrict_features_of(C, […])` / `forbid_features_of(C, […])` / `forbid_feature(C, f)` govern
  which features a crate *declares* on a dependency `C`: its explicit `features` list plus the
  `default` pseudo-feature (so `forbid_feature(C, "default")` ≡ requiring `default-features =
  false`), matched by package name and unioned across the target's dependency edges. It observes
  the **declared** request only — never expanding `C`'s own `[features]` graph and never reading
  `cargo metadata`'s resolved `resolve.nodes[].features` — so it is stable under Cargo feature
  unification and builds under the existing `--no-deps` metadata read with no new dependency.
  Findings are `C/feature` (kind-qualified when the dependency kind is not `Normal`), injective
  across the two polarities; severity, baseline, dependency-kind selection, and the text/JSON
  projection reuse the existing crate-rule machinery. Transitive/unification-enabled features are
  an explicit non-goal (declared-not-resolved, at the altitude of the existing dependency rules).
  Additive and non-breaking; existing constitutions and baselines are unaffected. See
  `COOKBOOK.md`.

### Changed
- Contributor-facing docs only: `AGENTS.md` makes the project's practised conventions explicit
  (document authority, OpenSpec lifecycle, adversarial review, single-source Definition of Done,
  branch prefixes, subject-only release commits); `BACKLOG.md` records the `0.1.x → 0.2.0` trigger
  and the install-vs-constitution decision; the `README.md` license section links to its files.

## [0.1.9] - 2026-07-11

### Added
- 圭表 `must_not_call_inline(…).strict_external()` — **opt-in**: also catch a *fully-qualified
  external-crate* call (e.g. a bare `chrono::Utc::now()` with no `use chrono`), closing the
  asymmetry where a sysroot head (`std::time::…`) was caught but a fully-qualified external head was
  silently resolved as local. A bare head matching a declared dependency is resolved as that crate,
  after a local-precedence ladder so a genuinely-local item of the same name stays local at any
  nesting depth. Composes with `.ending_with` / `.strict_prefix_only`; with the flag off the default
  is **byte-identical**, so existing constitutions and baselines are unaffected. Carried as a new
  `#[non_exhaustive]` rule variant (patch-safe; identity-parity, no baseline churn), and 圭表 grows
  its own rename-aware dependency-name reader — no dependency on 渾儀 (三儀 ⊥ 三儀), still `syn`-free.
  Stated bounds (an `extern crate … as` rename; and, under a single-segment prefix, a local binding
  or a definition site that reads as a call) are declared, never a silent pass.
- Adopter cookbook recipes (`COOKBOOK.md`): test that a boundary reacts, gate workspace coverage in
  CI, why exposure rules are deny-shaped (not a "may only expose" allowlist), and the
  `strict_external` recipe. `README.md` gains a "what the instruments do **not** see" note, so a
  reader does not over-infer a dimension's reach (渾儀 reads a signature's types/traits, never a
  call site).

### Changed
- Internal refinement, behavior-preserving and no public-API change: 渾儀's whole-crate-scan
  capabilities share one violation-emission helper; the text projection shares a module-block
  helper; idiom/consistency cleanups; and `xingbiao` now carries `#![deny(missing_docs)]` like its
  five sibling crates.

## [0.1.8] - 2026-07-11

### Added
- 圭表 inline-symbol-path confinement — forbid a crate from *calling* a fully-qualified path inline
  (e.g. `std::time::SystemTime::now()`), resolving `use` renames / aliases / re-exports and the
  glob-danger shapes. The syn-free static complement to observing a `use`-import.
- 渾儀 `UnsafeBoundary` — declare that a crate's `unsafe` (blocks, `unsafe fn`/`impl`/`trait`,
  `unsafe extern`) may appear **only under** a declared subtree
  (`UnsafeBoundary::in_crate("app").only_under(["crate::ffi"])`): the auditability boundary of a
  layered crate, the confinement complement of `#![forbid(unsafe_code)]`.
- 渾儀 visibility ceiling — `max_visibility(Crate | Super | Module)`, generalizing the binary
  `must_not_declare_pub` into a rank ceiling (an item declared above the ceiling reacts; the prior
  rule is now the `max_visibility(Crate)` sugar, byte-stable in findings).
- 渾儀 async-exposure opt-in **subtree** scope — `.including_submodules()` descends the anchored
  module's whole subtree, so a "this seam is synchronous" boundary governs a pure kernel throughout,
  not only at its own seam.
- Every crate declares `#![forbid(unsafe_code)]` — the family is `unsafe`-free and says so at
  compile time.
- `examples/` gained `unsafe-confinement` and `sans-io-pure`, plus a `max_visibility` demo in
  `hunyi-standalone`.

### Fixed
- 渾儀 unsafe-confinement: the finding is owner-qualified (`unsafe impl {trait} for {self type}`), so
  two `unsafe impl`s of one trait for different self types in a module no longer collapse to one
  finding — closing a baseline-masking false negative.
- 渾儀 / 圭表: a nested `#[cfg_attr(pred, path = "…")]` module remap is recognized in both dimensions,
  closing a silent false negative in the static scanner and the semantic subtree walk.
- 圭表 type-alias resolution skips a defaulted generic parameter's `=`
  (`type Clock<Tz = LocalTz> = std::time::SystemTime;` now resolves to its real target), closing a
  false negative where a confined type reached through the alias passed unobserved.

### Changed
- modou is no longer framed as superseded. It is a living, independently-developed sibling project;
  Tianheng's static core (圭表) is *derived from* it, and Tianheng keeps all three dimensions
  (README / PROJECT).
- README gained a Phase-0 one-line on-ramp (lock one seam, enforce, pipe SARIF into CI) above the
  full multi-dimension example.

## [0.1.7] - 2026-07-08

### Added
- 圭表 `confine_external_crate` — confine an **external** crate's `use` imports to one declared
  module subtree (FFI / platform-vocabulary confinement): `ModuleBoundary::in_crate("app")
  .module("crate::ffi").confine_external_crate("libc")` reacts when any module outside
  `crate::ffi`'s subtree imports `libc`. The first static rule to *observe* external-crate imports
  (every other rule ignores them), source-observed — not a `cargo metadata` dependency-table rule.
  The confined crate is the violation target, so confinements of different crates on one module stay
  distinct in the baseline. A package name written with a `-` (e.g. `windows-sys`) matches its
  underscore import identifier (`windows_sys`).
- `COOKBOOK.md` — a cookbook of common governance intents expressed as declared boundaries (圭表 /
  渾儀 / 漏刻 recipes), the imitable surface an adopter or agent copies rather than translating a
  foreign policy format.
- Coloured, reason-first terminal output for the human `check` report — a severity-coloured header
  (red for an enforced violation, yellow for an advisory) over the emphasised reason. Presentation
  only: gated to an interactive terminal (honours `NO_COLOR`), so a pipe, a redirect, or a CI log
  stays byte-identical, and `--format json` / `sarif` are never coloured.
- `examples/` — three runnable, self-checking examples: `guibiao-standalone` (the syn-free static
  import linter), `hunyi-standalone` (the semantic public-API exposure linter), and `composed`
  (the `tianheng` shell governing one app with all three instruments, in a CI-time `check` mode and
  a runtime `run` mode).
- Per-instrument GitHub issue templates (圭表 / 渾儀 / 漏刻).

## [0.1.6] - 2026-07-07

### Changed
- Extracted the `cargo metadata` substrate into a new `xingbiao` crate — a `serde_json`-only base
  beneath the dimensions — so the static and semantic dimensions read the workspace through one
  source of truth instead of two hand-copied twins.

### Fixed
- 渾儀 forbidden-marker: closed two false negatives — a hand `impl` whose self-type is spelled
  through a `pub use` re-export, and a locally-renamed (`use … as`) trait/derive leaf.

## [0.1.5] - 2026-07-07

### Added
- 圭表 `must_only_be_imported_by` — the closed inbound dual of `must_not_be_imported_by`
  ("only `crate::facade` may import `crate::internal`").

### Fixed
- 漏刻 probe-coverage audit: a probe inside a `macro_rules!` body no longer counts as coverage.
- Recorded a documented robustness bound in the `use`/`mod` lexer around multibyte char literals
  (no confirmed false negative).

## [0.1.4] - 2026-07-05

### Fixed
- 圭表 module-source hardening: module boundaries use Cargo's observed `src_path`, and
  `#[path]`-remapped and inline-only orphan modules are excluded rather than governed through a
  same-named conventional file.
- Packaging: every publishable crate now physically bundles its `LICENSE-MIT` / `LICENSE-APACHE`
  texts (`cargo publish` ships only crate-local files; 0.1.0–0.1.1 shipped without them). Guarded
  by a CI reaction.

## [0.1.3] - 2026-07-02

### Added
- 渾儀 semantic depth: public re-export exposure and trait-impl exposure.

## [0.1.2] - 2026-07-02

### Added
- 圭表 `restrict_dependency_sources_to` — govern the declared dependency source kind
  (git / registry / path).
- 渾儀 `dyn`-trait and `impl Trait` boundaries, and async-exposure.

## [0.1.1] - 2026-06-30

### Fixed
- Early packaging and metadata hygiene.

## [0.1.0] - 2026-06-29

### Added
- Initial release of the crate family: the `xuanji` reaction model, the three observation
  instruments — 圭表 (`guibiao`, static), 渾儀 (`hunyi`, semantic), 漏刻 (`louke`, runtime) — and
  the 天衡 (`tianheng`) shell that composes them into one `check` with a `0` / `1` / `2` exit
  contract and `--format json` / `sarif` projections.

[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v0.5.0...HEAD
[0.5.0]: https://github.com/tacticaldoll/tianheng/compare/v0.4.0...v0.5.0
[0.4.0]: https://github.com/tacticaldoll/tianheng/compare/v0.3.0...v0.4.0
[0.3.0]: https://github.com/tacticaldoll/tianheng/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/tacticaldoll/tianheng/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/tacticaldoll/tianheng/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/tacticaldoll/tianheng/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tacticaldoll/tianheng/compare/v0.1.10...v0.2.0
[0.1.10]: https://github.com/tacticaldoll/tianheng/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/tacticaldoll/tianheng/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/tacticaldoll/tianheng/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/tacticaldoll/tianheng/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/tacticaldoll/tianheng/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/tacticaldoll/tianheng/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/tacticaldoll/tianheng/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/tacticaldoll/tianheng/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/tacticaldoll/tianheng/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/tacticaldoll/tianheng/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/tacticaldoll/tianheng/releases/tag/v0.1.0
