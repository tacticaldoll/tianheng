# Changelog

All notable changes to the 天衡 (Tianheng) crate family. This is the **adopter-facing**
projection of the release history; the per-change *why* lives in the squashed change commits and
their pull requests. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

**This history begins at `0.6.0`.** This repository was re-founded at the `0.6.0` snapshot: `main` carries
that commit as its root, so no commit, tag, or release section here reaches `0.1.0`-`0.5.0`. Those versions
remain published and installable on crates.io, and each of their tarballs names this repository's URL
permanently — the package metadata is written at upload and a version can never be re-uploaded — so the link
crates.io renders for them no longer reaches the history they were cut from. **What a published crate
contains is in its own tarball**, which is complete on its own: every publishable crate carries its license
texts and passes its tests from the packaged tarball, and CI holds both. What is absent here is the commit
each of those versions records.

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

## [0.6.0] - 2026-09-09

### Governance

- **The by-directory record carrier is retired, and with it the document it was carrying.** `AGENTS.md`
  enumerated three record carriers; it now enumerates two, a commit message and a dated `CHANGELOG.md`
  section. The retired one was a directory, and the specification that governs the path direction already
  argued against it in so many words — *SHALL NOT extend to a document because of where it lives*, for a
  reason it states holds without an example — while the classifier reading it survived in the citation and
  census directions. With the directory gone the prefix matched nothing, which is a name without a reaction.

  **What it held was a second copy.** The document inventoried what commit each published version's tarballs
  name, and the two properties it carried are ones `AGENTS.md` states itself: that `cargo publish` records
  the commit rather than the content, and that a snapshot amended after a clean publish orphans a pointer the
  gate cannot foresee. Its table's other two columns were repository facts, and a root-commit `main` resolves
  neither, so the rows asserted relations nothing here can inspect.

  **The `by path` requirement is unaffected and that was checked rather than assumed.** It exists so a live
  document carrying a dated `## [X.Y.Z] - date` heading is not exempted by shape alone, and the path half it
  names is the `CHANGELOG.md` test, which is untouched. Two dead exclusions went with the directory, in the
  law-restatement corpus and in the Markdown anchor reader.

- **This repository was re-founded at the `0.6.0` snapshot, and `main` carries that commit as its root.** No
  commit, tag, or release section reaches `0.1.0`-`0.5.0`. Those versions remain published and installable,
  and what a published crate contains is in its own tarball, which the `License texts bundled` and `Packaged
  crate self-test` jobs hold complete on its own. What is absent here is the commit each of them records.
  This file's own header carries the same statement, which is the first thing a reader arriving from
  crates.io meets.

  The `[0.6.0]` comparison link takes the first-release form, which is what `release-coherence` requires of a
  spine whose latest snapshot has no predecessor. `BACKLOG.md`'s closed reproduction records and the
  `0.1.0`-`0.3.0` capability ledger are not carried.

- **Release snapshots now obey the same Conventional Commit grammar as every other commit on `main`.**
  Their canonical subject is `chore(release): X.Y.Z`; only the empty body remains exceptional because a
  snapshot records the whole release tree. The merge-message and publish-source gates accept only that form.

  The release-spine reader also recognizes the retired `release: X.Y.Z` form as a historical predecessor.
  That compatibility is directional: it lets an existing release branch classify itself against the old
  spine, while a retired subject at `HEAD` is refused as a new snapshot. The three gates derive the canonical
  rendering from one source so the merge, history, and publish boundaries cannot spell it differently.

- **A defect's codename from the process that found it was standing in the code that fixed it.**
  `Round-9 finding:`, `Round-2 fix:`, `F2`, `F6`, `fix #6`, `fix #9`, `Apply-review finding 1` and
  `propose-review` opened comments across nine files. Each names *when* a defect was seen and by which pass,
  which is the one thing nothing downstream reads: the round has no artifact in this repository, the letter
  codes index a list that never existed here, and a reader who wanted the provenance has the commit. What
  follows each of them is the invariant, and that is what the comment is for — so the codename came off and
  the invariant stayed, twenty-three sites, no other wording touched.

  **They lived where the reaction for exactly this shape declares it does not look.** `doc_provenance`
  refuses the `round` token in a published crate's **doc** comments and says so in its header — ordinary
  `//` comments are its stated stop. Every one of the twenty-three was an ordinary `//` comment. The sweep
  is
  `git ls-files '*.rs' | xargs grep -inE '^\s*//' | grep -E 'Round[- ]?[0-9]+|\bF[0-9]+[: ]|fix #[0-9]+|[Aa]pply-review finding|propose-review'`,
  the reader's own file excluded, and it now answers nothing.

  **The corpus did not widen, and the reason is that one token carries two dispositions.** A round number
  used as an index is provenance. A **count** of rounds can be the observation itself: *four rounds of
  findings in this file were one cause* is what says a positional reading cannot close the class, and *three
  rounds of widening each added the variable someone had just measured* is what says a name list is not a
  defence. Delete those and the invariant beside them has nothing to falsify it — the substitution
  `AGENTS.md`'s *Invariant first, observation second* was written to stop. No reader separates the two;
  an allowlist of the measurement sites is two lists that must agree; a shape matching only a comment that
  *opens* with a codename covers seventeen of the twenty-three and reads as though it covered the class.
  So `AGENTS.md`'s disposition table now carries the split on the two rows that already implied it, and
  `BACKLOG.md` carries the un-reacted half as a `WATCH` whose reopening condition is a property: the corpus
  widens with no allowlist the moment no comment in the tree carries a round count.

- **A gate and the fixture that demonstrates it were one module, twice, with a banner comment where the
  boundary belonged.** `release_coherence_gate` and `publish_source_gate` each held both jobs behind a
  `// --- the fixture ---` line, and `hermetic_git` held a third piece of the same job beside its invocation
  forms. A banner is a reader's note about a boundary; a module is the boundary. Both builders, and the
  commit helper the two shared, now live under `kanhe::fixture`, with a submodule per gate and the shared
  primitives in the parent.

  **The compiler stated the thing the banner could only assert.** With the section gone,
  `release_coherence_gate`'s `use crate::hermetic_git::fixture as run` became an unused import — so the
  judgement half had been holding a fixture runner, and now holds none. That is the measurement this change
  rests on, and it is the whole of it: a relocation preserves behaviour, so no direction was added and none
  was owed. What exercises the moved builders is the three targets that consume them, unchanged: 128, 38 and
  1 directions, green before and after.

  Two things the move surfaced rather than caused. The two builders wrote a fixture file two ways — one
  through a helper that creates the directories above it, one through a bare `std::fs::write` that did not —
  so a fixture naming a nested path depended on some earlier call having made the directory; they share the
  helper now. And `cargo doc` under `-D warnings` caught the one claim the move falsified: a doc link that
  resolved only through a re-export, in a sentence saying this file used the builder for its fixtures.

  **This has no reaction, and the reason is that the decidable half is not the class.** A reader could refuse
  a `*_gate.rs` that imports the fixture runner — the seed both gates actually used — but a gate can equally
  construct a fixture through the command builder directly, so the instrument would defend a fraction of the
  rule while reading as though it defended the rule. That is the shape this repository withdrew once already,
  and the criterion stays the instrument here.

- **The file that records what a pin proves carried a hand-written count of its own subject.** Its header
  named how many bounds the window had declared and how many bounds one target holds. Both were true when
  typed and both were understatements of the same sets before the window closed — a new bound and two more
  bounds arrived after the figures did — in the one file whose whole argument is that a citation proves a
  name resolves and only a record proves a pin bites.

  A census belongs where something enumerates it, and something already does: `pin_bites` prints
  `N declared mutation(s) covering N of M cited test(s)` on every clean run. So the figures are gone and
  neither is replaced by another figure — what stands in their place is the property they were standing in
  for. **What a window owes is its own bounds, not the standing set**: paying a record for every citation
  the repository already carries is not affordable and paying one per bound a window declares is, so the
  obligation rides the act that creates the debt. And the target with unrecorded bounds names them instead
  of counting them, since a denominator there is a count that moves while the sentence does not.

- **A scenario stated a refusal and nothing reached it.** `repository-checks` declares that a tracked Rust
  file the lexer refuses is a cannot-judge naming the path, and the reaction was there — but the
  repeated-paragraph target's twelve directions covered an unreadable file and a path that is not UTF-8 and
  never a file that does not lex. A reaction no direction reaches is a clause with the shape of evidence and
  none of the substance, which is the seam `AGENTS.md` asks a scenario to close in the direction it was not
  looked at.

  It was reachable the whole time: the target's own unreadable-file direction calls `offences` with a
  hand-built listing, so the probe is a file written into a scratch tree. **The fixture is chosen so the
  guard's absence accuses rather than merely skips.** An unterminated string literal wrapping a doubled
  comment-shaped paragraph reaches both halves at once — `proc_macro2` refuses it, and a lexer that could
  not run shadows nothing, so every comment-shaped line inside the literal is read as a comment. Measured
  with the lex question removed and the fixture untouched: `Violation — unterminated.rs:4: a 1-line comment
  paragraph is written twice in a row`, a file's own literal text reported back at its author as a
  repetition nobody wrote. The terminated spelling is the control and answers clean in both runs, so the
  direction is about the refusal and not about the paragraph being out of reach either way.

- **`xingbiao`'s charter says what it carries, and states the criterion instead of a list.** Its module doc
  enumerated the path-identity primitives it holds *beside* the metadata reader — an enumeration that stopped
  covering the module when the filesystem-answer policy was added to it this window, for the ordinary reason
  a list goes stale. It also said it sits beneath the static and semantic dimensions, while the runtime
  dimension's CI face has depended on it since before that.

  The widening was not wrong and it was not declared, which is the defect. It went in because three
  dimensions needed one answer — a reason that decides nothing on its own, since it is equally the reason for
  a consolidation this repository refuses elsewhere. **The criterion now written down is what separates
  them:** a fact belongs there when every dimension must agree on it before observing AND it is a fact about
  the tree rather than a reading of what the tree contains. What is there, as against what it says.

  That line refuses something concrete, which is why it is worth having: an interpretation of source is a
  dimension's own observation. `cargo metadata` says which file is a crate root; what the tokens in that
  file mean belongs to whichever dimension is asking — so the lexical-boundary question `BACKLOG.md` carries
  cannot be answered by putting a tokenizer here, and that is now a reading of the charter rather than a
  matter of taste.

  **Nothing reacted to any of this.** The self-law constrains what `xingbiao` may depend on and says nothing
  about what it may become, so a charter widening is prose against prose. `PROJECT.md`'s decision states the
  criterion for the same reason.

- **A mark is what encloses a phrase, and the parity of the markers before it is a different question.** The
  Markdown relative-anchor sweep decided *is this phrase a quotation* by counting the marks preceding it and
  reading an odd count as *inside one*. Parity and membership agree only where every marker pairs: after one
  that closes nothing, parity answers *marked* for every remaining phrase in the paragraph, so one stray
  marker suppresses every finding after it. Measured over this repository's own Markdown — a paragraph
  opening a fenced block carries an odd count by construction, and one live relative anchor in `BACKLOG.md`
  was standing behind exactly that while the reaction was green. `kanhe::reading::marked_spans` answers the
  membership question with the spans themselves, over both marker classes this repository defines a phrase
  with, and an unpairable passage answers *undecidable* — which the caller reads as marking nothing, the
  direction that over-reacts rather than the one that goes quiet. The same run over the whole corpus reports
  nothing else, so the over-reaction is unrealised rather than tolerated.

  **The reader existed and a declared bound is why it was not asked.** `reading` already owned the pairing,
  and `no_source_outside_the_shared_reader_pairs_markers_by_hand` already refused a `split` or a `find` on a
  marker literal outside it. Neither reached a `matches` count taken modulo two — that primitive sat inside
  an observation bound whose measured ground was that none of its live uses is a pairing, true when it was
  written and falsified by the site above. The tracking entry named its own promotion: *the question is the
  expression's shape rather than the primitive's name*. So the reaction now refuses a marker count taken
  modulo two for both classes, the bound keeps the four primitives whose live uses read one delimited value,
  and the direction is renamed for the marker classes it reads rather than the one it was written for.

- **A record carrier is decided by path as well as by section shape.** The same sweep exempted the span under
  any dated `## [X.Y.Z] - date` heading in any tracked Markdown file, while `AGENTS.md` enumerates exactly
  three carriers and a level-2 heading in a live document is none of them — so a live file gained a record's
  exemption by writing a record's heading. Measured: no tracked Markdown outside `CHANGELOG.md` carries that
  shape today, so this closes a latent false negative rather than a standing one. The shape reader is
  unchanged and the carrier is now the caller's answer.

- **A projected reason and a baseline clause say what holds, not how they came to say it.** `AGENTS.md` puts
  provenance in a third carrier — a commit message, a dated `CHANGELOG.md` section, `docs/history/` — and
  requires a boundary's reason in a **forward voice**, because that reason is projected into an agent's
  context and imitated there. Over a specification the test is sharper still: a baseline is atemporal, so
  *reproducible now, or not at all*. Twelve passages narrated their own prior wording instead: a bound whose
  reason said what it used to cover and until when, four more correcting a ground they had once stated, and
  seven spec clauses recording what a requirement previously required or superseded. Each is restated to the
  property that holds, keeping every measurement and rejected alternative the passage carried — those are
  observation sources, and the disposition table says they stay. The two generated projections carrying the
  reasons are regenerated with them.

  **Swept as a class, and the sweep is not a reaction, which is measured rather than asserted.** The corpus
  is the projected reasons and the specification clauses — the two carriers the forward-voice rule names —
  read for a passage narrating its own prior text. A marker list over that corpus (`until`, `previously`,
  `used to`, `no longer`, `was false`) reports 6 of 58 reasons and some 60 spec lines, and most of those are
  correct forward voice: a `WHEN` clause saying a baseline entry is *no longer* present names a current
  condition, and *cannot be published until the path is renamed* is a forward consequence. `AGENTS.md`'s own
  live text uses *this rule used to carry an exception* load-bearingly, with the invariant the exception
  violated. So the repair the detector would name depends on reading the sentence, which is what
  `AGENTS.md` states as the criterion for an appeal rather than a reaction. It stays an appeal, now with the
  corpus measurement behind it.

### Static

- **A `#` at an attribute's name position reaches the attribute it opens, and a direction now holds that.**
  `attr_name_start` skips the opening `#`, the `[` and the whitespace and stops, so it can answer a position
  that is itself a `#` — `#[#[path = "x.rs"]` answers the inner one, and the remap lies inside the attribute
  that `#` opens. The scanner reaches it because a name it did not match is left for the loop head to read as
  an opener, and nothing pinned that: measured, stepping over an unmatched name passes every other row of the
  fixture table and loses this candidate, which is a false negative in a scanner whose whole construction is
  the false-negative-safe union. The row is added, and it fails against the tree without the property.

- **The attribute scanner's termination is provable where a reader is looking.** The two matching branches
  advance the cursor themselves — `j` from `i + 4` past the name and its whitespace, and eight bytes for a
  matched `cfg_attr`, with `starts_with` guaranteeing no `#` among the bytes either steps over, which is what
  makes them identical to the single-byte walk they replace. They terminated before this too, by the loop
  head: the cursor sits on an attribute's *name* from that point and never on the `#` that opened it, so the
  next iteration stepped it forward. That invariant is true and it is not visible at the `continue` — four
  independent readings of these three lines called the scanner non-terminating, each reading the `continue`
  and not the loop head. The unmatched-name case keeps the loop head as its owner, deliberately, for the
  reason the entry above gives.

- **The instrument that says an axis does something was built for one axis, which is the shape the change
  before it repaired.** A `Value` spelling collapsed onto its sibling had been closed by asserting the two
  spellings differ; `Position` was owed the same and did not have it. A shape's `position` comes from the
  loop variable and its `attribute` from the match arm, so the two can part — measured, a `Direct` arm
  emitting `#[cfg_attr(unix, {meta})]` satisfies the set agreement, both value assertions and the
  per-position floor, reports the same 90 spellings, and leaves the direct attribute position unexercised.
  The axis the corpus grew to cover, covered by nothing. It is matched rather than iterated now, so a new
  position is a compile error there as it is in the builder.

  **The three obligations are stated where the next axis reads them.** Set agreement, per-variant coverage,
  variant distinctness — each catches a shrink the other two cannot, and the third is measurably the one
  that gets missed: twice in this window, once per axis, each time passing every assertion that existed when
  it was tried. A principle in a commit body is not a principle an axis inherits, so it sits on the axis
  definitions.

  **And it was attached to one axis while governing both**, which is the annexed doc this repository names
  elsewhere — a passage describing one thing hanging off its neighbour. Fifteen lines of *both axes owe all
  three* sat at the head of `Position`'s item doc, displacing `Position`'s own summary line out of first
  position, while `Value` said nothing about the obligations. That inverts the reason for writing them down:
  a third axis modelled on `Value` — the per-variant shape, which is the one that generalises — would be
  read beside the sibling that carried no rule. It is a `//` region comment above both definitions now, so
  it is not an item doc at all and neither axis owns a rule that governs the pair.

  **Three instances of one discipline failure, and the class is filed rather than closed.** Each time the
  missing instrument was the one for the axis added *beside* the one being fixed. A trait demanding all
  three pieces per axis would make the compiler ask for them, and it misdescribes what is there:
  `Value::spell` is a pure per-variant spelling while `Position` selects a whole sub-product, which is the
  asymmetry the sum-over-positions structure exists to express. `BACKLOG.md` carries it as a `WATCH` whose
  trigger is a **third** axis — two cannot answer whether the asymmetry generalises.

  The set-agreement message also now says what it holds. It compares the rendered sequences, because the two
  axes are different types and one loop carries both, so a **reorder** fails it too — and the message said
  *trimmed*, which would have named the wrong cause at the one moment someone reads it.

- **Two axes were added and one was floored.** The coverage floor that arrived with `Position` and `Value`
  iterated `Position::ALL` and asserted nothing about the axis added beside it, in the same commit whose own
  comment says *a branch that stops emitting is a corpus that shrank, and a shrinking corpus passes*. Both
  axes are floored now, and `Shape` carries its value spelling for the same reason it carries its position:
  the floor reads the axis value rather than the label.

  **A floor iterating the array the corpus was built from cannot see that array trimmed**, which is the
  wider hole and the reason the expectation is now declared. Measured: `Value::ALL` cut to `[Ordinary]`
  dropped 43 of 90 shapes and every assertion passed, because the floor asked about the set the corpus was
  built from rather than the set the corpus is supposed to cover. The expectation is declared here and held
  to each enumerator **both ways**, which is the shape `AGENTS.md` prescribes for a claim something
  downstream filters on — and the precedent it cites is exactly this: a coverage assertion filtering on a
  literal stayed green when a dimension was removed from the list it filtered against.

  **And an axis whose variants coincide is not an axis.** A `spell` that stopped distinguishing the two
  would satisfy every coverage floor while generating one form twice under two labels, because a floor reads
  the value a shape was *labelled* with. The assertion that the two spellings differ is independent of both
  arrays, and is what says the axis does something.

- **A generated corpus claimed the language and enumerated a cross-product.** The attribute-spelling
  differential's doc read *the corpus is every lexical form Rust admits, which no inspection enumerates* over
  a flat `wrapper × predicate × meta` product — a claim no cross-product can make, and one that was false
  along three axes at once. Every shape was `cfg_attr`-headed, so the **direct** attribute position had no
  row; every value was an ordinary literal, so `path = r#"target.rs"#` had none; and the bare-`cfg` spelling
  the doc names as the defect that motivated the whole file had none either.

  Two of the three are axes now, `Position` and `Value`, each with its variants beside it, and the corpus is
  a **sum over positions** rather than one product — because the axes are not orthogonal, which measuring
  them is what showed. A direct attribute carries no wrapper and no predicate; and it admits no look-alike,
  since `#[foo::path = "target.rs"] mod m;` is `error[E0433]: cannot find module or crate 'foo'` with no
  false predicate to keep the qualified path from resolving. A decoy has to compile while naming nothing, so
  decoys live only where a predicate carries them. The count is printed on every clean run — 90 spellings,
  80 governed and 10 look-alikes — rather than written into the prose that describes it.

  **The third is a different question and is filed as one.** A bare `#[cfg(pred)]` removes the whole item
  when `pred` is false, where `#[cfg_attr(pred, …)]` never removes the item, so what a reader does with a
  bare `cfg` is *absence tolerance* — whether a missing backing file is an error — and its probe is a file
  that does not exist rather than an item that resolves. `Answer` has no value for it. Rows added to the
  existing corpus would have declared an answer the corpus cannot check; `BACKLOG.md` carries it as a
  second corpus with the shape it needs.

  **Widening found no disagreement**, which is worth saying plainly rather than leaving to be inferred: all
  three dimensions already answer the direct position and the raw value correctly. What the window bought is
  that the corpus now covers them and a floor keeps it covering them — every declared position must produce
  a shape, read off the axis value rather than a label, because a branch that stops emitting is a corpus that
  shrank and a shrinking corpus passes.

- **A variant whose doc and behaviour disagreed, beside a remap that could remap nothing.**
  `PathAttrKind` held `None`, `Remaps { direct: Option, conditional: Vec }` and `Excluded`. Its sole
  consumption folded `Excluded` into the same arm as `None`, so the variant was behaviourally identical to
  its neighbour while its doc said the opposite — that such a module is excluded from conventional file
  backing. And `Remaps` admitted `direct: None` with an empty `conditional`: a remap that remaps nothing,
  kept out by an `if` before the constructor rather than by the type.

  **What the deleted variant stood for is not legal Rust**, which is what decided the repair rather than an
  argument about intent. Measured against rustc 1.96.0, edition 2021, `--crate-type lib`: `#[path] mod m;`
  and `#[path("m.rs")] mod m;` are both `error: malformed 'path' attribute input`. The 圭表 scanner is
  deliberately cfg-blind and unions every candidate a build *could* compile; a shape no configuration
  compiles is outside that, so implementing the doc's claimed exclusion would have governed source that
  cannot exist. The answer is the same as no remap at all — which is what the fold already produced, and is
  now what the type says.

  `Remap` is two variants, `Direct { at, conditional }` and `Conditional { first, rest }`, where `first`
  keeps the conditional state non-empty the way `xuanji::bound::Defence::PinnedBy` does for the same reason.
  There is no negative run, and the reason is the property: a state the type cannot hold has no value to
  assert about. Planting the construction the old shape admitted answers `expected 'usize', found
  'Option<_>'`, and the two rows added to
  `both_readers_take_the_attribute_name_from_one_position` hold the behaviour that used to reach the
  deleted variant.

- **BREAKING** — **圭表 requires the built-in `cfg_attr`'s exact path, where it matched the bare
  identifier.** `foo::cfg_attr(a, path = "bogus")` ends in the same word while being somebody else's
  attribute, so the collector descended into it and took `bogus` as a module remap. Measured over the span
  `(any(), foo::cfg_attr(a, path = "bogus"), path = "real.rs")`: two positions where one is right, and the
  raw spelling `foo::r#cfg_attr` the same. A module rustc never reads was then scanned, and any violation
  found there reported over source the governed tree does not have.

  The qualification is decided by the token before the segment, and `r#` changes a lexical spelling
  without changing a name, so one segment stays one. The control holds the other side: an unqualified
  nested `cfg_attr`, raw-spelled or not, keeps its applied metas.

  **This was found by measuring a claim rather than by a report.** An entry in `BACKLOG.md` said 圭表
  answers this question correctly and was written without measuring it; 漏刻's equivalent repairs made
  that claim worth checking, and it was false. What 圭表 does close, by a `depth == 1` guard, is the
  compound predicate that defeated 漏刻 — so neither reader was the correct one, and each hand-rolling of
  the same lexical decisions missed a different subset.

  **Why a minor:** this **removes** findings rather than adding them. A recorded baseline holding a
  violation from a target no longer read no longer describes the adopter's tree, and `--disallow-stale`
  reports it. Regenerating is work they did not choose.

### Release

- **A path handed to git as a pathspec was read as an instruction.** `--` separates revisions from paths
  and nothing else: git parses pathspec *magic* after the separator too. Measured on this machine's git with
  a directory literally named `:(exclude)odd` tracked in a fixture, `ls-files -- ':(exclude)odd'` answers the
  **whole** repository — the pathspec stopped restricting and became an exclusion of something else, so the
  machinery corpus for an unpublished member was an answer to a different question.

  **Two wider fixes were measured and rejected, and the third is where the knowledge is.**
  `GIT_LITERAL_PATHSPECS=1` on the shared builder makes `check-ignore` fail outright — `fatal: pathspec
  magic not supported by this command: 'literal'`, exit 128 — so the exclusion classifier cannot take it.
  `--literal-pathspecs` on the shared `tracked_records` breaks the callers whose pathspec **is** a pattern:
  `capability_subjects` asks for `openspec/specs/*/spec.md` and `law_restatement` for `*.md`, and the flag
  disables glob magic with the rest. That premise — *this accessor never takes a caller's glob* — was
  asserted and then falsified by running the suite, which is why the literalization sits on
  `repository_path`, the owner of how this repository spells a path.

  The direction carries its own control: without the un-literalized listing beside the literal one, the
  assertion would pass on any git that never parsed magic there and report clean for an unrelated reason.
  What is **not** held is the choice to use the owner at each call site — dropping it compiles and nothing
  goes red, because the type that knew the value was a path is gone by then, and a reader deciding which
  `&str` came from a path is the judgement over source this repository declines.

- **A refusal an operator reads before an irreversible act carried twenty-six spaces mid-sentence.** The
  message for a member manifest with no parent directory read *there is no member&nbsp;… directory to
  enumerate*. `AGENTS.md`'s carrier table records this shape as one with **no reaction** deliberately —
  measured across every `violation_at`/`cannot_judge_at` message in the tree, the only remaining run was
  column alignment a reader wants — so nothing was going to catch it. The tooling that produced it is worth
  naming: a Rust `\`-continued string literal written through a Python heredoc collapses, because Python
  treats the same backslash as its own line continuation and keeps the following indentation.

- **The reader that enumerates crate manifests told absent from unreadable, and its twin in the same file
  did not.** `require_example_pins` was given `std::fs::metadata` with `NotFound` separated from every other
  answer, and a comment saying why: *absent is not unreadable; skipping both alike let the remaining
  readable examples satisfy the counters below*. `workspace_manifests` kept `manifest.is_file()`, which
  answers one `false` for a manifest that is not there, one that cannot be stat'd, and a directory named
  `Cargo.toml` — so a member whose manifest could not be read left the enumeration silently. The repair for
  the class landed without a sweep for its sibling.

  It is one construction with three reasons now, because the register holds a site identity to exactly one
  branch and three arms reaching three calls would be one identity vouching for branches no direction
  reached. The direction puts a directory where a crate manifest was, and the negative run shows what the
  fold produced: not a clean verdict but a refusal blaming the **declaration form** for a manifest nothing
  opened. Whether a silent clean is reachable instead depends on whether every judgement standing on the
  enumeration would notice a missing member, which is not established here and is not claimed.

- **This release line is `0.6.0`, decided from what the window's changes do.** It carries `**BREAKING**`
  markings — false-negative closures, a rule-key identity change, a module-resolution widening — and this
  repository's rule past `0.1.0` is that breaking earns a **minor**. What decides the number is that any such
  marking is present, so the classes are named and the tally is left to whatever enumerates them. The line was
  opened as a patch and reclassified when the markings were read against it; the working number it carried
  before that is not recorded here, because a version this project never released names nothing an adopter
  can resolve, and this document is the adopter-facing projection.

  **Nothing mechanical said so, which is why it is written here.** `release_coherence` compares version
  literals to each other — `Cargo.toml`, the dated changelog heading, the lock — and never to the change
  class the entries declare, so the release would have passed every gate and shipped a patch requiring
  baseline regeneration. It was found by an independent review reading the markings against what the window
  had built.

### Migration

- **Regenerate any recorded baseline if a module in your tree carries more than one `path` inside a single
  `cfg_attr` attribute, beside a conventional file.** Those extra targets were not read before and are read
  now, so a tree that was green may report new findings: run `tianheng check --write-baseline <file>`
  wherever a baseline is kept, and re-apply any `owner` / `tracker` annotations onto the newly observed
  facts. `#[cfg_attr(unix, cfg_attr(a, path = "x.rs"), cfg_attr(b, path = "y.rs"))]` is the shape; several
  SEPARATE `cfg_attr` attributes carrying one `path` each were already unioned and are unaffected.
- **Regenerate any recorded baseline if a boundary declares a module path, allowlist entry, or confined
  crate name with a raw identifier (`r#name`).** Those rules keyed on the spelling as written while the
  evaluation folded it, so the two forms were one boundary to the matcher and two identities to the
  baseline. They are now one: run `tianheng check --write-baseline <file>` wherever a baseline is kept and
  re-apply any `owner` / `tracker` annotations. A declaration carrying no `r#` anywhere is unaffected —
  every key is byte-identical.
- **Regenerate any recorded baseline if a module in your tree is remapped through an attribute whose path
  merely ENDS in `cfg_attr` — `foo::cfg_attr(…, path = "…")`.** That target was read and is not any more, so
  a baseline entry recorded from it describes a file the governed tree does not have. Run
  `tianheng check --write-baseline <file>` wherever a baseline is kept. A declaration using the built-in
  `cfg_attr` unqualified, raw-spelled or not, is unaffected.
- **Regenerate any recorded baseline if a module in your tree gates a `pub use` and a same-named child
  `mod` on `r#not` rather than `not`.** That pair reported nothing before and reacts now, so a tree that
  was green may report a new exposure: run `tianheng check --write-baseline <file>` wherever a baseline is
  kept, and re-apply any `owner` / `tracker` annotations onto the newly observed facts. The plain `not`
  spelling already reacted and is unaffected.
- **Make every module target readable, or expect exit `2` naming it.** A module file or directory that any
  of the three dimensions cannot open is no longer tolerated as an absent one. Where a `#[cfg]`-gated declaration sat over an
  unreadable subtree, the audit previously reported the seam inside it as unprobed — or passed clean where
  that seam was probed elsewhere — and now refuses, naming the path and the reason. Fix the permission, or
  exclude the path from the audited roots.
- **Nothing to do where a multi-`path` `cfg_attr` declaration previously stopped the run.** Where no conventional file backed it,
  the check reported the module unresolvable and exited `2`. That was a false refusal over code rustc
  accepts; it now resolves, and no baseline existed to move.

### Semantic and runtime

- **渾儀's built-in-name predicate is named for its reach.** It was named for attributes alone while one call
  site asks it about `not`, which is a `cfg` predicate's combinator and not an attribute at all — a name
  wider than the thing it names, in the family that holds every other name to that standard. `pub(crate)`,
  so no published surface moves. Found by the same review.


- **BREAKING** — **渾儀 reads the `not` wrapper as the name it spells, where it matched the identifier as
  written.** The cfg-aware carve-out decides whether a same-named child `mod` genuinely shadows a `pub
  use`'s bare head: where the two are provably mutually exclusive, the `mod` never wins name resolution
  for that `pub use`'s own build and must not suppress it. `Path::is_ident` compares an ident as written,
  so `#[cfg(r#not(unix))]` was not the negation of `#[cfg(unix)]` — while the reader of the predicates
  that wrapper encloses already stripped the prefix off every segment. One comparison family answered two
  ways about one grammar.

  The outcome is the forbidden one rather than a noisy one. A negation read as no negation leaves the
  shadow standing, so the re-export is never resolved against the extern prelude at all and its exposure
  is dropped. Measured on `#[cfg(unix)] mod serde;` beside `#[cfg(r#not(unix))] pub use serde::Value;`
  under `must_not_expose("serde")`: the finding set was **empty** where the `not(unix)` spelling emits
  `serde::Value exposed by pub use crate::api::Value`.

  That the two spellings are one predicate is rustc's answer, not an inference. Measured against rustc
  1.96.0, edition 2021, `--crate-type lib`, with the module's file absent: `#[cfg(r#not(unix))] pub mod
  a;` compiles, because the predicate is false on this host and the item is removed;
  `#[cfg(r#not(windows))] pub mod a;` fails `E0583`, and `#[cfg(not(windows))] pub mod a;` fails it
  verbatim. A reader that separates the spellings governs a configuration nobody compiles.

  **Why a minor:** this **adds** findings. A recorded baseline that was green over such a pair no longer
  describes the adopter's tree, and regenerating it is work they did not choose.

- **The owner's own header stated a defect in the present tense, after the defect was gone.** The module
  written to end three spellings of one identity opened with a table of the three and a verdict naming
  which of them agreed — true when it was written, false the moment it landed, and read as current state
  by anyone arriving afterwards. The class its own repository refuses: a past defect described
  where the invariant belongs.

  The invariant is kept and the debrief is tensed: two of the three readers are the two **sides of one
  comparison** held as strings, which is what makes a shared spelling load-bearing rather than tidy, and the
  disagreement that falsified it is recorded as closed. The enum's doc is split the same way — the reason it
  is not an `Option` first, the two live readings that made it one second, in separate sentences, so
  trimming the provenance cannot take the falsifier with it.

  Found by re-reading this window's own repairs rather than by running anything, which is where a claim about
  code sits when the code is right.

- **An extraction removed the reach and left the visibility behind, in two crates.** A name's visibility is
  a promise about who may call it; an extraction that collapses the callers into one place ends the promise
  without touching the line that makes it, and nothing complains — the name still compiles, still resolves,
  and now says something about this tree that stopped being true.

  `xingbiao::is_absence` was added `pub` this window on a crate that ships, and its callers are
  `is_regular_file` and `is_directory` **in its own file** — a published name is a promise this crate would
  have to keep, made for nobody. `hunyi`'s `is_anchor_absent_from_unit` is the same shape from the other
  direction: it was called from eight files at `0.5.0`, the `over_each_unit` extraction removed every one of
  those call sites, and the `pub(crate)` stayed. Both are now as narrow as their reach — crate-private and
  module-private respectively. Neither name was ever released public, so there is nothing to regenerate and
  nothing for an adopter to do.

  **Nothing reacted, and nothing here is proposed to.** Reach is a whole-crate fact, and the visibility a
  name *should* carry is not decidable from one file — the shape a reaction would need is a call graph, which
  is a reading of source and therefore each dimension's own, not the substrate's. Found by review.

  Narrowing `is_absence` broke two rustdoc links to it, which is the bound `reference-integrity` declares
  rather than a surprise: a link to a private item is exactly what it refuses. Both were demoted to prose, as
  `repository_path`'s own module doc already records doing for the same reason.

- **The widened reader was still narrower than the clause written for it, and its scenario pinned a
  direction that could not fail.** Two independent reviews arrived at the same pair, and both hold.

  The clause says a citation is refused *wherever it stands inside a code span*. The reader split on
  whitespace — and a citation sits beside **punctuation** far more often than beside a space, because a
  revision expression glues it to `..`, `^`, `~` or `{`. Widened to every maximal hex run whose neighbours
  are not alphanumeric, it immediately reported two live sites the previous widening could not reach: a
  `<object>..release/X.Y.Z` range in `BACKLOG.md`, and — the reader catching its own explanation — the
  three example spellings written into the comment that introduced the previous widening. The examples are
  placeholders now, which is the shape working rather than an inconvenience.

  **And the scenario was pinned to a direction that scans the tracked corpus**, which is kept clean — so
  reverting the widening left it green, and a scenario was resting on a guard that had stopped guarding.
  `a_citation_glued_to_punctuation_is_read` supplies the offending spans itself, one per narrowing this
  reader has had: the whole-span predicate reads **0 of 3**, whitespace tokens read **1 of 3**, and the
  delimiter-bounded runs read all three. Both negative runs are recorded against the fixture untouched.

  The object is assembled from pieces none of which is object-shaped, because the file it is written in is
  inside the corpus the live sweep reads — a literal one there would be an offence of the class under test.

  `BACKLOG.md`'s citation is now `v0.4.0..v0.5.0`: two tags, so the range resolves from any clone and
  neither end can be renamed. Verified the same range — one commit either way.

- **An import sat 200 lines below its first call site, with its reason attached to a `use` that renders
  nowhere.** 漏刻's probe scanner takes `is_directory` / `is_regular_file` from the substrate; the `use`
  stood at line 502 between two functions while the file's import block is lines 1–3 and the first call is
  line 299. The two mid-file imports this workspace already has each express a reason by their placement —
  one pairs with a `pub use` re-export, one is `#[cfg(test)]`-gated — and this one had none. A `///`
  attached to a private `use` is documented nowhere, so its explanation reached only a reader already
  standing at line 502.

  Moved to the block, with the explanation kept as a `//` note beside it.

- **The reader that forbids a commit object could not see one cited inside a longer span, which is why
  three of them stood.** `is_abbreviated_object` asks whether a code span **is** the hex. `git show <object>`
  is a single span carrying spaces, so the predicate answered *no* while the citation was plainly in it —
  and this reader had run over those very lines on every push. The corpus was narrower than the claim, in
  the reader whose whole subject is a claim's reachability.

  It now tests every whitespace-separated token of a span; the shape predicate is unchanged and is what keeps
  the noise out. Over the repaired tree it reports **zero**, and the negative run is the shape that escaped:
  with the citation restored, the widened reader fails naming `f41b3b9` at its line, and the whole-span
  reader passes on the identical tree.

  `reference-integrity` states it as a clause — *the reader's corpus SHALL be the claim's corpus* — with its
  own scenario, rather than leaving the widening to be inferred from the code.

- **This window widened three rules and swept none of the corpora the widenings named.** Each rule was
  broadened here, and each broadening brought new text into scope that nobody then read. Reported by an
  outside review and verified against the tree before acting:

  - **Specification prose became atemporal** — *reproducible now, or not at all* over `openspec/specs/*`.
    **Seven clauses added this window are historical narrative**: a floor that *was once* aggregate, an
    answer that *is no longer* available, *an earlier version of this requirement*, a rule *first written
    for the wrapper alone*. Two of the seven were written by the sweep that widened the rule. Each is now
    the mechanism stated as a property, which is the disposition the rule itself names.
  - **Commit objects went from *live prose* to *anywhere in tracked content*.** Three remained, on two
    lines, and each was resolved against `origin/main` rather than assumed: none is contained in it, so none
    resolves in a fresh clone. `AGENTS.md`'s own census bound was one of them — and the sentence carrying it
    claimed *naming the commit inside the command keeps that answer checkable after the tree moves on*, four
    lines below the row that says why a development object cannot. **The claim asserted the property its own
    file denies.** Re-anchored to `v0.5.0`, where the same command answers **137**, the figure it already
    carried.
  - **Carriers went from prose to every tracked live file.** Three relative anchors — `this window` — stand
    in live `BACKLOG.md` sections, one of them in a line this window's own version sweep edited and left.
    Named instead. **The claim about the remainder was wrong and is corrected below**: it said the remaining
    hits were the rule quoting the phrase to define it *and one inside a record section, which is exempt* —
    and `AGENTS.md` grants no record exemption in the relative-anchor row, whose whole disposition is *anchor
    it to the moment, or name the item*. The three carriers it does enumerate, one section over, are a commit
    message, a dated `CHANGELOG.md` section and `docs/history/`; a `BACKLOG.md` section is not one of them.

  **What connects them is not carelessness but shape**: a rule widened in the same change that repairs its
  old corpus leaves the new corpus unread, because the sweep was written before the widening. `AGENTS.md`
  already schedules a pre-cut reading for `BACKLOG.md`'s triggers; nothing schedules one for a rule that
  just grew.

- **The pre-cut trigger reading cannot be verified by a reader, and four attempts to verify it are the
  evidence.** The rule landed in this window: `BACKLOG.md`'s promotion triggers are read against the window
  before the cut, and an evaluation records **what it checked**, because a bare *not fired* is
  indistinguishable from nobody having looked. Asking *has that been done?* looked mechanical. It is not.

  Four readers over the live sections, each correcting the last: looking for a capitalised *Not fired*
  inside a nine-day date window answered 47 entries unevaluated; widened to the window's real start,
  26; widened to verdicts spelled *fired*, *swept*, *witness-only*, 11; widened again to read the **whole**
  entry rather than only the text after the trigger sentence — because one entry's reading sits *above* its
  trigger, phrased *re-derived the same day* — 8. **Each number is a property of the reader, not of the
  tree.** A verdict in this file has no canonical spelling, which is the same conclusion the scenario-clause
  entry reached one round earlier about a different subject, reached again here by a different route.

  So the rule's own framing is the correct one and needs no repair: *this is a step someone performs, listed
  here because a discipline nobody is asked for is one nobody does*. It never claimed to be checkable, and
  the attempt to check it is what failed.

  What the attempt did produce: one entry filed this window carried no verdict and now does — the
  scenario-clause seam is **unfired at filing**, and says what that rests on. And a reading was performed
  in full for the one-spelling corpus reader, which had one already; both agree, which is the only
  cross-check available here.

- **This window quoted three prior states of its own prose that no release carried, against the row it
  added for exactly that.** The rule is `CHANGELOG.md`'s own: `main` carries one snapshot per release, so a
  sentence written and corrected inside one window existed in **no** artifact an adopter has, and quoting it
  resolves from nowhere. Swept with the rule's own test at the last tag: an evidence clause quoted as what
  it *read*, a module header's verdict quoted the same way, and a doc comment quoted as what it *said*.
  None is at `v0.5.0`. Each is now the **defect stated** rather than the dead sentence quoted, which is the
  repair the row itself names — *its doc claimed two where the file held four* resolves from any clone; the
  sentence it claimed it with does not.

  **And the row shipped a test that over-reports.** `git grep -F "<the quote>" <last tag>` is line-oriented
  and a quotation is not: measured, a quote this repository *does* carry at `v0.5.0` reported absent because
  the source wraps it across two lines. A sweep run as written would accuse a legitimate quotation. The row
  now says to join lines — the same wrapped-instance trap the sweep rule three sections above already
  records, met on this row's own instrument.

- **A baseline carried a measurement nobody can re-run, and a count of a set nothing enumerates.**
  `reference-integrity`'s whole-directory-exemption clause narrated the defect that produced it — *it hid a
  present-tense pointer at a relocated gate* — and then typed how many of that directory's path references
  had already resolved. Both belong to a record rather than a baseline: the pointer is repaired, so the
  first cannot be re-run, and nothing in the tree produces the second, which sits in the document everything
  else is checked **against** while the directory it counts has since changed. `AGENTS.md` sets the sharper
  test for exactly this carrier — over `openspec/specs/*`, *reproducible now, or not at all*.

  The invariant was already in the next sentence, so the repair is the disposition the table names — keep
  the invariant, drop the debrief — and it is now stated as holding without an example: the facts a record
  must keep are shas, dates, versions and counts, none of which is a path, so exempting the directory buys
  a record nothing it needs and costs every path reference in it.

- **A reclassified entry was still called by its old class, against a rule this repository had already
  written down.** `BACKLOG.md` says *retire the WATCH line in the same change*, and a previous window
  recorded why: an index carrying a question and its answer at once is a reader trap. The entry that moved
  from `WATCH` to `ACCEPTED DEBT` this window left a live sentence elsewhere in this section still naming it
  a `WATCH` — the retirement sweep `AGENTS.md` requires before closing a change that moves a mechanism,
  unrun by the change that moved it. The sentence now says what it was and where the class lives.

- **The seam that has no owner is now priced, and both instruments are declined on measurements rather
  than on an impression.** `AGENTS.md` states the rule and says the direction is unowned; the repair below
  fixed the three instances; this asks whether the rule can react at all. Two readers were built and
  measured, and the answer is no — for different reasons, which is why both are recorded.

  **The lexical reader** — a term in two or more of a requirement's scenarios and in none of its prose —
  produces thousands of candidate refusals over the spec corpus, every sample an ordinary word a scenario
  used and its requirement did not repeat. Every narrowing that quietens it (hyphenated terms only, a floor
  of three scenarios, the `THEN` clause only) reaches single digits **and stops catching the known
  instance**. Under the noise is the reason it cannot work: the property was written *both* `raw identifier`
  and `raw-identifier` inside one spec, so there is no canonical spelling to key on.

  **The structural reader** — a requirement gaining a scenario while its own prose block is unchanged — is
  decidable and spelling-independent, and was measured rather than dismissed. Per commit it fires on about a
  third of all requirement-level scenario additions in this repository's history, **including every release
  snapshot**, because a squash collapses the development commit where the prose did move. At the window
  level, which is what a branch gate sees, it fires on **27 of 107** across every window from `0.1.0`
  onward — a quarter of all legitimate spec work, whose repair on a false positive is a clause the author
  did not need. `PROJECT.md`'s recorded rule decides it: *never add a permanent authoring tax to close a
  bounded, visible failure.*

  So the discipline stays a reviewer's, and the entry carries both measurements with the method, so the
  next person prices it from this rather than rebuilding it. Its trigger is the property that would change
  the answer: an instance reading does **not** find.

- **Eleven scenarios pinned a property no requirement declared, and the seam that names this had no owner
  in exactly that direction.** `AGENTS.md` states it: requirement prose gaining a clause with no
  scenario, or a scenario gaining a `PINNED-BY` with no clause declaring what it pins — the seam had no
  owner in either direction, and both halves of it once landed in a single commit. This window then wrote raw-identifier scenarios into `module-boundary`,
  `semantic-forbidden-marker` and `semantic-signature-coupling` without a clause in any of the three
  requirements saying an attribute, a derive or a macro is the name it spells. `runtime-origin-assertion`
  got its clause, which is what made the omission visible — the same corpus-narrower-than-the-claim shape,
  one level up, in the repairs for it.

  Each requirement now declares the property its scenarios pin, and **the keyword half travels with it**:
  `r#mut` is an identifier named `mut` and is precisely *not* the keyword, so a reader matching Rust
  keywords compares as written. A clause stating only the first half is the one a later reader
  over-applies.

- **A load-bearing evidence clause, falsified by the window that cited it.** The lexical-scanner entry's evidence
  clause — the one it marks as not neutral — claimed two things: that 渾儀 had never carried one of these
  shapes, and that what failed was hand-rolled lexing specifically, in both crates that do it. Both are
  false as of this window, and the clause was still standing when a
  decision citing it was recorded.

  渾儀 carried four shapes, and in three it was the **only** wrong reader — a raw identifier spelling `path`
  or `cfg_attr` inside a `cfg_attr`, the same on `derive` and its wrapper, and a raw spelling of the
  transparent macro's own name. None is a lexing defect: `syn` does the lexing, and what failed is
  `Path::is_ident` comparing an identifier **as written**. The error runs the other way too — 圭表 and 漏刻
  alone read a qualified applied `path` — and once, on a raw bare `cfg`, all three were wrong together.

  So the class is not a property of hand-rolling, which is what the clause turned into an argument for a
  token boundary. **It does not reopen the decision; it removes the one argument that pointed at a
  boundary**, since neither a token boundary nor a shared substrate would have reached the `syn`-side
  failures. The differential does, and did: reverting that comparison reports eighteen rows. The null
  option's own cost line loses its count with it — the shapes are named in the entries that closed them,
  and a figure maintained beside them would drift from those.

- **The instrument built to end a class committed that class, in its first round.** The differential's
  rustc step asked one question with one arm: compile the shape *with* a reference to an item defined only
  in the remap target, and report a failure as *rustc rejects the generated spelling … the corpus claims a
  shape Rust does not admit*. That arm fires on two facts. Measured:
  `#[cfg_attr(unix, allow(dead_code))]` declared governed reported exactly that sentence over source rustc
  **accepts** — rustc's own first line, `cannot find function only_in_target in module imp`, was about the
  reference rather than the spelling. Two facts an author repairs in opposite places, and the message named
  the wrong one.

  Three questions now, three arms, three messages: *rustc will not take this spelling* is the generator's
  legality claim; *rustc takes it but the build does not contain the named target* is the declared answer;
  and for a look-alike, *with its predicate made live it DOES apply a remap* is the corpus calling a real
  module target a decoy.

  **The third arm is the part that was not merely a repair.** A look-alike's declaration — *no configuration
  compiles this file* — rested on the generator's word. It is now measured: each look-alike's predicate is
  made live and rustc is asked, and *no remap applies* is established either by rustc refusing the attribute
  outright or by it compiling while the target's item does not resolve. Seven look-alikes, each shown rather
  than asserted. What rustc still cannot decide is said where it applies: a governed shape under a dead
  predicate has no live configuration on this host, so question one is all it can answer and the live rows
  are what keep that declaration honest.

  All three arms were checked for bite, each reporting the fact it owns rather than a neighbour's.

- **The lexical-scanner question is decided, and the option chosen was not one of the three recorded.**
  The entry's *what would decide it* rested on a premise: that separating lexing from interpretation is
  *how* enumeration becomes possible. It is not. The enumeration was built over the readers **as they
  stand** — what makes a differential possible is exactly the property the boundary option defends, that
  the three implementations are independent and can be asked the same question separately. A shared
  substrate is the one thing that would have made it harder.

  **Two parties are not enough, which is what the instrument adds over a reading.** A differential over the
  three dimensions alone answers *do they agree*, and agreement is not correctness: the raw bare-`cfg`
  spelling was missed by all three at once, and a reader comparing only the three would have called that
  clean. So rustc is the third party. `attribute_spelling_differential` generates the corpus over wrapper ×
  predicate × meta axes plus the look-alikes, compiles every shape, refuses one rustc will not take as the
  **generator's** defect rather than a dimension's, resolves an item defined only in the remap target
  wherever the predicate is live, and holds all three dimensions to the declared answer.

  Forty-three spellings on the first run: every one legal Rust, every one answered identically. **The null
  result was checked for bite before it was believed** — reverting 圭表's qualification narrowing reports
  two rows, 漏刻's reports two, 渾儀's raw-identifier comparison reports eighteen, and a deliberately
  malformed shape is reported against the generator. It is env-gated behind
  `TIANHENG_SPELLING_DIFFERENTIAL` and named on its own line in the Definition of Done and in CI, the trade
  `pin_bites` already makes for a direction that compiles things.

  What it does not buy is stated with it: the corpus is the generator's axes, not every lexical form Rust
  admits. That is strictly better than what someone thought to *read*, it can be grown cheaply, and its
  coverage is inspectable in one file — so **growing an axis is now the repair a new spelling earns**,
  rather than a patch to two scanners and a hope about the third. The backlog entry moves from `WATCH` to
  `ACCEPTED DEBT` on that bound.

- **A requirement narrower than the reaction it governs, in the capability that owns the reader.**
  `runtime-origin-assertion` states the single-segment rule for the `cfg_attr` **wrapper** — a qualified
  look-alike carries no module target — and said nothing about the applied `path` meta itself. The repair
  below moved the reaction; the requirement it is checked against still described the narrower shape, which
  is the direction this window has already corrected once for the same spec.

  The clause and its scenario now cover the target, with the consequence stated in this capability's own
  terms: a probe inside a file no build compiles counted as coverage, so the audit reported clean over a
  seam nothing probes.

  **What found it was a differential probe rather than a reading.** Sixteen `cfg_attr` spellings rustc
  accepts were run through all three dimensions, then nine more whose target must **not** be read, with the
  violations placed only in the file that must not be reached. Current tree: no disagreement and no
  over-read. The instrument was checked for bite before that null result was believed — reverting the
  qualified-path repair turns two of the nine red, `圭表=1 渾儀=0 漏刻=1`, and one of those two is a
  raw-and-qualified combination nobody had written a direction for. The probe is not kept here; what it
  produced is evidence for the standing architectural entry, which is where the question of enumerating this
  reader's corpus waits.

- **BREAKING** — **The narrowing that closed a qualified `cfg_attr` was never applied to the target it
  wraps.** Both byte scanners compute whether a segment was reached through `::` and both spend it on the
  `cfg_attr` decision alone — 圭表 captures the flag and tests it in that arm only, 漏刻 clears it before
  the `path` arm can read it. `path_meta_values`' own doc comment states the rule its `path` arm does
  not apply: *the built-in is the SINGLE-segment path*. A file disagreeing with itself, in the reader whose
  subject that rule is.

  Measured under rustc 1.96.0, edition 2021, `--crate-type lib`:
  `#[cfg_attr(any(), foo::path = "bogus.rs", path = "real.rs")] mod plat;` compiles, because a false
  predicate expands no applied attribute and `foo::path` is never resolved. Both dimensions are cfg-blind
  by construction and union every candidate on disk — and a file named by nobody's `path` is not one.

  **Both halves of the class, one perturbation each.** 圭表 answered `1`: a violation reported against
  source the governed tree does not compile. 漏刻 answered `0` over a seam whose only probe sat in that
  file — coverage fabricated by a spelling, which is the Core Contract's forbidden bug. 渾儀 takes the
  same question through `get_ident`, which declines a multi-segment path, and was correct.

  The `path_meta_values` entry records exactly this trigger — *a Rust-valid spelling of any of those four
  properties that the pinned corpus does not already contain* — and it has **fired**. The entry carries what
  fired it and says plainly that the local repair is not the decision it waits for. It was a `WATCH` when
  this was written; a later change in this same window decided it and moved it to `ACCEPTED DEBT`, so the
  class is named there rather than here.

  **Why breaking:** an adopter whose source carries a qualified applied attribute ending in `path` has 圭表
  and 漏刻 baselines that no longer describe their tree — the entries that named the wrongly-read file are
  gone.

- **A repair that was not made, and the measurement that stopped it.** 渾儀 answers a `cfg_attr` whose
  applied metas do not parse two ways: `cfg_attr_path_values` drops the whole attribute's `path` candidates
  with `.ok()`, while `scan::items::extract_derives` answers the identical failure on the identical
  attribute with a scan error — its own doc saying *"cannot judge" is never a silent skip*. Both use the
  same parser, so the difference is the answer and nothing else, and the Core Contract states the policy the
  second follows.

  Replacing `.ok()` with a refusal is one line. It was not made, because *a violation is a rule, and a rule
  needs a reachable instance*: the attempt to reach the silent arm through source rustc accepts used
  `#[cfg_attr(unix, path = "imp_unix.rs", unsafe(no_mangle))] pub mod imp;` — ordinary source, since the 2024
  edition requires `unsafe(no_mangle)` in place of the bare spelling. Measured under rustc 1.96.0, edition
  2021, `--crate-type lib`: it compiles, the remap applies, syn parses `unsafe(no_mangle)` as a `Meta`, and
  渾儀 answers `1` alongside the other two.

  So the asymmetry is filed in `BACKLOG.md` as a `WATCH` with that measurement and a trigger stated as a
  property — a `cfg_attr` whose applied metas rustc accepts and `Punctuated<syn::Meta, Comma>` rejects,
  which a syn upgrade can fire as readily as a spelling. The fixture is kept as a conformance direction,
  pinning a shape nothing covered — a `path` remap standing beside a sibling applied attribute — and saying
  in its own doc that it pins the contract rather than a change.

- **BREAKING** — **The class has two halves that point opposite ways, and enumerating it is what showed
  that.** Three rounds each closed one instance of *an identifier compared as written where rustc compares
  the name it spells*. Rather than a fourth instance, the class was enumerated — every comparison of an
  identifier against a literal name across all three dimensions — and it splits:

  - **A non-keyword name**: `r#` escapes nothing and only changes the spelling, so the two are one name.
    `path`, `cfg`, `cfg_attr`, `derive`, `cfg_if` are all this half.
  - **A Rust keyword**: `r#` is what makes the identifier *not* the keyword. Measured under rustc 1.96.0,
    edition 2021, `--crate-type lib`: `pub fn r#mut() -> u8` is a function named `mut` and compiles. So the
    readers matching `static`, `mut`, `ref` and `as` compare **as written**, correctly, and applying the
    first half's rule to them would break each one.

  **The one instance the enumeration found was `cfg_if`**, the transparent-macro name, read in all three
  dimensions. 渾儀 compares through `syn`, where a raw `Ident` is not equal to the plain string, so
  `r#cfg_if! { … }` was an opaque macro and everything declared in its arms went unobserved — measured, it
  answers `0` where 圭表 and 漏刻 both answer `1` on the same tree. Both byte scanners accept it and by
  accident: each walks **backwards** over identifier bytes from the `!` and `#` is not one, so the walk
  stops after the prefix. That property now says so at both sites, because a forward reader there would have
  to consume the prefix explicitly and nothing would have noticed.

  `semantic-signature-coupling` gains the scenario, pinned, and states the keyword half beside it so the
  next reader does not apply the wrong one.

  **Why breaking:** an adopter invoking `cfg_if!` through a raw spelling has a 渾儀 baseline that no longer
  describes their tree.

- **BREAKING** — **A third reader of attribute names, found because the sweep that closed the first two
  took one file as its corpus and the claim was about the crate.** `scan::items::extract_derives` reads
  `#[derive(…)]` and the `cfg_attr` that wraps one, and it compared both names as written. Measured
  against rustc 1.96.0, edition 2021, `--crate-type lib`: `#[r#derive(Clone)]`,
  `#[r#cfg_attr(unix, derive(Clone))]` and `#[cfg_attr(unix, r#derive(Clone))]` each apply the derive — a
  `.clone()` on all three types compiles. Read as written, all three were invisible.

  A derive this reader does not see is a marker `semantic-forbidden-marker` cannot refuse. Perturbed, the
  direction returns `[]`: three forbidden `serde::Serialize` derives, none of them found, on a subtree that
  declares it forbidden. That is the false negative the Core Contract names, in a capability whose whole
  subject is refusing a marker.

  **What found it was the corpus, not another round of reading.** The two repairs before this one swept
  `crates/hunyi/src/syn_util.rs` while claiming a property of 渾儀's attribute reading; the crate holds a
  second file that reads attribute names, and it was never in the corpus. The seed that reaches the class is
  the identifier rather than the wording, and the search is `git grep -n 'is_ident(' crates/hunyi/src`. With
  this change landed it returned **one** site — a `cfg` predicate combinator (`not`), a different grammar
  and one **this sweep did not measure** — so the command doubled as the check that nothing else compares an
  attribute name as written.

  **That clause once disposed of the remaining site as well as describing it, and the disposing half was
  reached without measuring.** The describing half is true: a predicate combinator is not an attribute's
  name, which is why this entry's own completeness claim held with that site untouched. What a corpus
  argument cannot decide is whether the grammar it excluded may compare a name as written — and the
  measurement that followed says it may not, the `not` wrapper's own entry in this section carrying it. The
  sweep returns nothing now. A statement about what a sweep measured survives the next measurement; a
  disposition resting on a site the sweep did not measure does not, so this one says which of the two it is.

  **Why breaking:** an adopter whose source spells a derive or its `cfg_attr` wrapper with a raw identifier
  has a 渾儀 baseline that no longer describes their tree.

- **BREAKING** — **The attribute's own name position takes a raw identifier too, and closing it in one
  dimension is how the second position was found.** The repair below reached the spelling inside a
  `cfg_attr`'s argument list, where both byte scanners already consumed the `r#` prefix with its segment.
  The attribute's **own** name is a separate position and none of the three read it: 圭表 matched `path`
  against the bytes after `[`, 漏刻 lexed the name with an identifier reader that stops at `#`, and 渾儀
  compared through `syn`. Fixing 渾儀 alone therefore turned a shared miss into a disagreement, which is the
  thing the cross-dimension ledger exists to prevent — found by re-reading the repair rather than by running
  anything.

  Measured under rustc 1.96.0, edition 2021, `--crate-type lib`: `#[r#path = "imp_unix.rs"] pub mod imp;`
  compiles with only `imp_unix.rs` on disk, and with a conventional `imp.rs` present as well it is the
  remapped file that is compiled. Perturbed one dimension at a time, each of the three answered `0` — clean
  over a file the build does not contain, while the file it does contain went unobserved.

  **And the same spelling on a bare `cfg` costs a refusal rather than a miss.** `#[r#cfg(target_os = "none")]
  pub mod gone;` compiles with no `gone.rs` anywhere, because `r#cfg` is the built-in `cfg` and a false
  predicate removes the item. All three withheld that tolerance from the raw spelling and reported a
  missing-file constitution error over source that compiles — consistently, which made it invisible to a
  ledger that only asks whether the three agree.

  `module-boundary` gains both scenarios, each pinned. Neither carries a declared mutation, for the reason
  `pin_bites` states: a declared mutation is a claim about a **bound's** defence, and these are ordinary
  scenarios rather than declared observation bounds.

  **Why breaking:** an adopter whose source spells an attribute name with a raw identifier has 圭表 and 漏刻
  baselines that no longer describe their tree, for the remap half; the `cfg` half only removes refusals.

- **BREAKING** — **A raw-identifier spelling of a path remap is the same remap, and 渾儀 alone read it as a
  different one.** `r#` changes an identifier's lexical spelling and not the name it spells, so `r#path`
  names the built-in `path` attribute and `r#cfg_attr` names `cfg_attr`. Measured under rustc 1.96.0,
  edition 2021, `--crate-type lib`: `#[cfg_attr(unix, r#path = "imp_unix.rs")] pub mod plat;` compiles with
  only `imp_unix.rs` on disk, the nested `#[cfg_attr(unix, r#cfg_attr(target_os = "linux", path =
  "imp_linux.rs"))]` compiles with only `imp_linux.rs`, and with a conventional `plat.rs` present **as well**
  it is the remapped file rustc compiles.

  圭表 and 漏刻 each say this in their own byte scanners, in nearly the same words — *a raw identifier is ONE
  segment*. 渾儀 reads its attribute names through `syn`, whose `Path::is_ident` compares the ident as
  written: proc-macro2's `PartialEq<str>` requires the compared string to carry `r#` when the ident is raw,
  so `r#path` was not `path` to it. The crate already had `strip_raw` and already applied it to module
  identifiers; it was never applied to attribute names.

  **Two outcomes, and only one of them was loud.** With no conventional file, 渾儀 exited `2` on source
  rustc compiles cleanly. With a conventional file present beside the remap, it governed *that* file — one
  the build does not contain — reported **clean**, and left the file the build does contain unobserved. A
  false negative produced by a spelling of the governed code, which is what *no spelling, alias, re-export,
  `cfg` arm, or macro form escapes observation* forbids.

  `module-boundary` gains both scenarios, each pinned, each with its declared mutation.
  `cfg_attr_path_only_module_conformance` gains three directions: the two spellings, and the conventional
  file standing beside the remap.

  **Why breaking:** an adopter whose source spells a remap with `r#` has a 渾儀 baseline that no longer
  describes their tree — the reaction is additive, the baseline is not, and regenerating it is work they did
  not choose.

  **What is deliberately not changed:** `r#cfg` in a bare `#[cfg]` position. All three dimensions miss that
  spelling equally today, and closing it in one alone would create the divergence this change removes. The
  direction is also the safe one — a missed bare `#[cfg]` withholds the absent-file tolerance, so it refuses
  where it might have passed.

- **A path-QUALIFIED look-alike reopened the same hole, twice.** Group kind was decided by the last
  identifier before a `(`, and `foo::cfg_attr(a, path = "bogus")` ends in that word while being somebody
  else's attribute — so applied-meta scanning resumed inside it and the false coverage came back through a
  different spelling. The first repair asked whether the segment was qualified by looking **behind** it over
  whitespace, and `foo::/**/cfg_attr` stopped that scan at the comment's `/`: the same over-read, a spelling
  later. Measured, before each: `Clean(Subject { declared: 2, reached: 1 })`.

  A third spelling followed: `foo::r#cfg_attr`, where a raw identifier became three scanner events — `r`,
  `#`, `cfg_attr` — and cleared the qualification twice. A raw identifier is ONE segment, and `r#` changes
  a lexical spelling without changing the name, so the prefix is consumed with its segment and the name
  compared is the one it spells. The control matters as much: an **unqualified** `r#cfg_attr` IS the built-in, so the narrowing
  must not cost a genuine nested group its applied metas.

  The built-in is the **single-segment** path, and whether a segment is reached through `::` is a fact about
  the token before it — tracked forward past trivia, because a comment is trivia and must not change what a
  path IS. `runtime-origin-assertion` gains the clause and the pin, since the reaction moved and the
  requirement had said nothing about qualified attribute paths at all. All three spellings are one
  direction now, because replacing one with another is how the second was lost while the third was closed.

  **The shape is filed rather than answered one spelling at a time.** Every property this reader's decision
  rests on is lexical — where a predicate ends, whether a group is a `cfg_attr`'s, whether a segment is
  qualified, what one identifier is — and each is answered from bytes. `BACKLOG.md` carries that with the
  directions that pin the corpus as it stands: what a further Rust-valid spelling would decide is whether
  the corpus can be enumerated, not which shape comes next.

  Two carriers of the previous repair went with it: the call-site comment still said the reader searches
  *anywhere in the argument span rather than parsing nesting structure*, which is the shape that read
  predicates as targets, and the entry below still said one flag per open group closes it — true of the
  repair before last, and half the rule since. Closing it took a phase **and** a kind, and each repair
  carried one.

- **A `path` inside a COMPOUND predicate was still read as a target.** The repair below tracked a predicate
  phase per parenthesised group — and `all(…)` is one, with a comma of its own — so
  `#[cfg_attr(all(unix, path = "bogus"), path = "real.rs")]` set the phase past `all`'s comma and collected
  `bogus`. The same false clean, one nesting level in: measured,
  `Clean(Subject { declared: 1, reached: 1 })` with only the group-kind half reverted.

  A comma inside `all(…)`, `any(…)` or `not(…)` belongs to the predicate grammar and says nothing about the
  surrounding `cfg_attr`. So a group is an applied-meta position only where its `(` follows the identifier
  `cfg_attr` — which also excludes another attribute taking a `path` argument of its own. The phase alone
  was half the rule; the kind is the other half.

  `runtime-origin-assertion` required the shape that produced this — *the scanner locates the `path` value
  anywhere within the outer attribute's argument span* — so the requirement is amended rather than the
  reaction left disagreeing with it, and the two predicate positions are pinned by a scenario.

- **A `cfg_attr` PREDICATE spelled `path` was read as an applied module target.** 漏刻's scanner walked the
  whole argument span, so `#[cfg_attr(path = "bogus", path = "real.rs")] mod plat;` recorded **both**. That
  declaration is legal source whatever cfg flags are set, and reading `bogus` scanned a file rustc does not
  compile: a probe inside it counted as coverage, and the audit reported **clean** over a seam nothing
  probes on any real build. Measured, before: `Clean(Subject { declared: 1, reached: 1 })`.

  Closing it took two rules, and this repair carried one: a phase per open group, so a `path` before that
  level's first comma is part of the condition and names nothing. The entry above carries the other — the
  group's KIND — after a phase alone read a compound predicate's comma as a `cfg_attr`'s. 圭表 already
  waits for the top-level comma and 渾儀 skips the predicate at each level, so this was the third dimension
  disagreeing about which positions are applied targets, on the same shape the cfg_attr union found them
  disagreeing about.

  **The comment that recorded the disagreement and left it open was wrong three times**: the shape needs no
  special build to appear in a tree, reading it is not harmless, and closing it does not need a nesting
  parser — 圭表 does the same kind of scanning without one, though not, it turns out, without the same
  holes. Left open on the strength of that comment, in the same window
  that wrote it.

  Two carriers of the earlier title/base/head widening also survived its sweep: a positive control whose
  assertion still spoke of an unchanged title alone, and a tracker saying `gh` offers no precondition *for
  either* where the set is three. The sweep that missed them named identifiers and prose; assertion
  messages and cardinality words are the classes it did not carry.

- **BREAKING** — **One owner decides what a failed `metadata` means, and all three dimensions ask it.**
  `Path::is_file` answers `false` both for a target that is not there and for one this reader could not
  stat, and the `#[cfg]` tolerance is what an ABSENCE is owed — so an unreadable subtree was swallowed by
  it, with whatever it holds going unobserved. 圭表, 渾儀 and 漏刻 each carried the collapse, at twelve
  reads between them.

  `xingbiao::is_absence` carries the criterion and `is_regular_file` / `is_directory` carry the reads. It
  lives in the substrate because all three ask the same question and none may ask another — the same
  reason `canonicalize_or_fail` is there. The criterion, not a list: an error saying *this path cannot name
  anything* is an absence; one saying *this reader could not find out* is not. `FilesystemLoop` is
  deliberately loud.

  Measured, 渾儀 with only its own reader reverted: `left: 0, right: 2` — clean, over a subtree it could
  not open. 漏刻's two directions hold its half.

  **圭表's arm has no falsifier, and that is declared rather than implied.** Measured on the same fixture it
  exits `2` before and after — loud for a reason this change does not touch — so an assertion over it would
  pass either way. Its repair travels on symmetry with the two that were seen to fail; the change can only
  convert a silent tolerance into a loud refusal, never the reverse, so what is unknown is whether that arm
  was reachable at all rather than whether the repair is safe. `BACKLOG.md` carries the search.

  **Why a minor:** a tree with an unreadable module subtree was green and now exits `2`, in two more
  dimensions than *A module target this reader cannot read is refused* already said.

- **A repair of the absence class refused a tree rustc compiles, and three more defects rode in with it.**
  An independent adversarial review of this window found four in changes that had already merged, three of
  them self-inflicted by the two commits above and every one passing a green Definition of Done and eight
  CI jobs. Only the last is a corrected claim; the first refuses source an adopter can legally write.

  *`NotFound` was taken for the only absence.* Measured: with `src/gated` a plain file,
  `fs::metadata("src/gated/mod.rs")` answers `NotADirectory` — the target cannot exist, so it is absent, and
  routing it to the loud channel refused a `#[cfg]`-gated declaration over a tree rustc compiles cleanly.
  That is the class the same window closed one change earlier. `is_absence` carries the criterion now: an
  error saying *this path cannot name anything* is an absence, one saying *this reader could not find out*
  is not. `FilesystemLoop` stays loud deliberately, matching `read_dir_entries_sorted`'s own policy.

  *A three-arm distinction had two reachable arms.* `OwnerUnnameable::Unresolved` named a path resolving to
  no candidate, which no producer can construct — `resolve_path_all` with the current-module fallback always
  yields at least one. Measured: with that arm's clause replaced by a panic, the whole suite stays green.
  A name with no reaction is what the drift law forbids, so it is gone, and *three arms rather than two,
  deliberately* is withdrawn with it.

  *And the causes reached the wrong refusals.* A trait impl whose **trait** would not render, with a self
  type rendering perfectly, was told *its syntax has no supported rendering* about the owner — which is
  exactly the defect the causes exist to close, produced by the change that closed it. Both trait refusals
  take the trait sentence now.

  The `list` projection guard's summary is narrowed to what a fixture-derived corpus can hold; a Migration
  bullet's antecedent, a hand-written count of one live set, a false claim about a scanner skipping its
  predicate, and two ordinals for one event are corrected with it.

- **An owner that cannot be named says which of three things was met.** A self type is unnameable because
  its path resolves to no candidate, because two mutually-exclusive `#[cfg]` branches bind one alias to
  different types, or because its syntax has no supported rendering — and all three emitted
  *cannot identify … without a positional fallback*, which names the **policy** rather than the fact. The
  refusal is right in every case; the sentence sent an adopter to the wrong place in two of them, with
  nothing in the emitted text to grep for the one they had. Measured, before: a `#[cfg]`-collided alias
  reported `cannot identify unsafe impl self type in crate::net without a positional fallback`, not
  mentioning the collision at all.

  `OwnerUnnameable` carries the cause to the callers that refuse. Three arms rather than two, deliberately:
  collapsing *unresolved* into *unrenderable* would say a path's syntax has no supported rendering about a
  path that renders perfectly and simply resolves nowhere — a smaller instance of the defect this closes.
  The collector's one refusal that is **not** about a self type, an impl whose trait could not be named,
  keeps its own sentence.

  `current_owner` gains the same distinction: `Ok` where the enclosing impl's self type could be named,
  `Err` carrying why where it could not, and absent where there is no enclosing impl — three facts that had
  two representations between them.

  **Patch-class:** no verdict, exit code, finding identity or emitted document moves. Refusing was correct
  before and is correct now; what changes is whether the refusal can be acted on.

- **BREAKING** — **A rule's key canonicalizes every module path it carries, as its evaluation already did.**
  `module_check` maps `canonical_module_path` over an allowlist so a boundary may be written with `r#name`
  or `name` and match either way, and folds a confined crate the same way. Two key arms did not:
  `canonical_module_set` sorted and deduped the spellings it was given without deciding which spellings are
  one path, and `ConfineExternalCrate` keyed on `package_name_to_import_ident` alone, which replaces `-`
  with `_` and nothing else — so a raw prefix survived it.

  Measured, each taken alone because the first assertion to fail hides the rest:
  `allowed: "[\"crate::r#type\"]"` against `"[\"crate::type\"]"`, and `crate: "r#gen"` against `"gen"`.
  So a declaration rewritten between the two spellings — a rename to a reader and to the evaluator — moved
  the identity every recorded violation is filed under. That is the class the trait-impl-locality entry in
  `BACKLOG.md` closed by re-keying, in two arms it did not reach.

  Both specifications already ended their canonicalization scenarios with *having been canonicalized to one
  identity*, and both spoke only of **matching**. They now say the recorded identity too.

  **Why a minor:** an adopter whose declaration carries `r#` anywhere has a baseline that no longer
  describes their tree. One carrying none is unaffected — every key is byte-identical.

- **BREAKING** — **A module target this reader cannot read is refused, never tolerated as an absent one.**
  `is_file()` answers `false` for two different facts. Measured as uid 1000: a directory at mode `000`
  holding `mod.rs` gives `Path::is_file("gated/mod.rs") == false` with
  `fs::metadata(..).err().kind() == PermissionDenied`, while `Path::is_dir("gated") == true`. Read through
  `is_file()` alone the target is *absent* — which is exactly what a `#[cfg]`-gated declaration is allowed
  to have, so the whole subtree behind it was tolerated and never audited.

  The visible half is worse than a silent skip. Measured: the audit exited **1** reporting
  *declared seam 'gated' has no configured probe marker* — over a file that declares one. The probe is
  there and the directory could not be opened, so an adopter is told their correct code violates, with a
  cause that sends them to the wrong place. Where the same seam is probed elsewhere the verdict is `0`
  instead and the subtree is simply unaudited.

  The three states were already present: `Ok(Some(..))`, `Ok(None)` and `Err` are what the resolvers
  return, so the repair is the **routing** rather than a new type — *unreadable* belongs on the channel
  that fails loud, beside the ambiguity it already carries, not on the one a caller may tolerate.

  **Why a minor:** a tree whose seams are probed elsewhere was green over an unreadable subtree and now
  exits `2`. That is work the adopter did not choose, and *"the defect was ours" does not spare them
  the work*.

  **All three dimensions take it now**, through one owner in the substrate — see *One owner decides what a
  failed `metadata` means*.

- **BREAKING** — **Every `path` in one `cfg_attr` span is read, where two dimensions took the first.**
  Measured against rustc (edition 2021, `--crate-type lib`), this compiles cleanly on Linux with only
  `linux.rs` on disk and neither `mac.rs` nor `plat.rs` present:

  ```rust
  #[cfg_attr(unix, cfg_attr(target_os = "macos", path = "mac.rs"),
                   cfg_attr(target_os = "linux", path = "linux.rs"))]
  pub mod plat;
  ```

  渾儀 and 漏刻 each answered `mac.rs` and stopped. With no conventional file behind the declaration
  that resolves to nothing, so both reported the module unresolvable — **exit 2 over valid code**,
  measured at `left: 2, right: 1` for each, run separately. With a conventional file present they
  governed it and never looked at the rest.

  圭表 already unioned every `path =` across nested groups, so this was one shape on which the three
  dimensions disagreed. The union has two axes — several `cfg_attr` attributes each carrying one
  `path`, and one attribute carrying several — and 渾儀's own doc records the first axis being repaired
  after a `find_map` *silently dropped every candidate but the first-declared*. A second `find_map` one
  level in did the same thing, and the repair had stopped there.

  The conformance suite's stacked direction covers two ATTRIBUTES with one `path` each, which a
  first-only reader passes: each span holds exactly one.
  `all_three_dimensions_read_every_path_nested_in_one_cfg_attr` is the axis it did not have, with the
  violating target declared **second** on purpose.

  **Why this earns a minor rather than a patch, decided from what the change does rather than from its
  shape.** An adopter stuck at exit 2 is merely unblocked, which costs them nothing. But where a
  conventional file exists beside such a declaration, the additional targets were previously unseen and
  are now governed: a tree that was green can report new violations, and its recorded baseline no
  longer describes it. That is depth reacting by default, and *"the defect was ours" does not spare
  them the work*.

### Documentation

- **The per-unit fan-out policy had two halves and only one of them had a home.** `is_anchor_absent_from_unit`
  decided what an absence *means*; what to *do* about it — govern where the anchor is, refuse only where it
  is nowhere — was written out at each of the seven boundary checkers, head and tail identical in all seven
  and differing only in the body between them. `over_each_unit` now holds it beside the half that already
  lived there. What was genuinely per-boundary stays at its call site: `trait_impl`'s note that a governed
  trait is as unit-varying as a governed module is why *that* boundary fans out at all, and is kept.

- **A gate's label was also its dispatch key, so renaming it would have dropped `-D warnings` in silence.**
  The examples suite decided two independent properties — whether warnings fail the build, and whether the
  family patch applies — by comparing a label against a literal written twelve and sixteen lines away in the
  same expression. Measured: renaming `"clippy"` to `"lint"` in the table, which reads as a wording change,
  leaves clippy running without `-D warnings` and the suite green. Nothing in the tree references those
  labels, so nothing would have said so — in the suite whose own module doc says *the reaction it
  demonstrates could be gone entirely*. The gates are a declared struct now; the label names the gate in an
  assertion message and decides nothing.

- **The CLI usage block was restated in a module doc, and the copy had drifted.** `main.rs` carried the
  invocation shapes beside the ones `usage()` prints, and the runner rejected `--disallow-stale` while the
  copy did not name it — the same flag and the same direction as the instance `BACKLOG.md` records for
  `list`'s requirement, which was closed by deriving the set rather than by correcting the prose. A doc
  comment cannot derive anything, so the copy is deleted and its owner named. What stays is what only that
  file says: what the binary is, and what its exit codes mean.

- **The governable-crate-root predicate was spelled twice in one module.** `crate_root_files` and
  `member_root_files` each carried the same `LIBRARY_KINDS`-or-`bin` filter over the same `src_path` read,
  so admitting a target kind would have moved one and not the other. `member_root_files` is now the
  per-package reader flat-mapped across the workspace; what stays its own is the scope and the ordering —
  `crate_root_files` dedups within a package by first appearance, and this sorts across packages so the
  corpus is deterministic whatever order cargo lists them in. `target_has_kind`'s doc named the pair it is
  shared by and now names how each reaches it.

- **The third reader of a `use` statement's body now shares the one home the other two do.**
  `lexer::scan_use_statement` declares itself *the one place both interpret what is a `use` statement's body
  identically* — and `pub_use_statements` re-spelled it, so it carried neither the shared guard nor the
  shared shape. The guard matters where a `use` is followed by `<`: that is a precise-capturing bound and
  not an import, and scanning to the next `;` there swallows the following real `use`. The extraction that
  made the reader shared converged two sites and walked past the third, which is the extraction-corpus class
  this repository already records. It is unreachable from this site — `pub` cannot precede a type bound —
  and is now answered rather than assumed, so the loop owns no arm the shared reader does not. What
  genuinely differs, the `pub` and visibility-qualifier walk, is what the function keeps.

- **The workspace-isolation check decided a TOML question on raw lines, and its own doc said otherwise.**
  Its reader walked `manifest.lines()` while its doc claimed a line was the right unit *because of* the two
  cases a line gets wrong — `[workspace]` inside a string, and `[workspace]` after a `#` — and the reader
  got both of them wrong. Measured,
  each run alone against that reader: `[workspace]` on its own line inside a `"""` block read as a declared
  root, and `[workspace] # root` read as no root at all. `repository-checks` states the corpus rule as a
  SHALL over exactly this kind of reader, and the declared bound beside it explains why nothing reacted —
  an absence is not a shape, so nothing can scan for a filter that was never written. It takes the executed
  TOML region now, which crosses the multi-line string forms because a line is not a unit TOML respects.

- **The TOML parse refusal had five copies and a helper whose doc said there were two.** `toml_edit` renders
  an error over several lines and a refusal message is one, so every reader flattened it — four by hand and
  two through a helper whose own doc claimed the file held **two** such sites while it held four. A claim
  about the code, wrong while the code was right. One owner
  now holds the flatten and the sentence; each caller keeps its own state or constructor, which is what
  differs. Deleting the helper is what enumerated its callers: a count of them said two, and the compiler named a
  third.

  Two annexed doc comments moved with it — the paragraph describing `declared_dependencies` had been sitting
  above the helper, and `DEPENDENCY_KINDS`'s one-line doc above the enum before it. And `workspace_version`'s
  doc claimed it read *from the shared region* while its body parses with `toml_edit` and its module imports
  no region: the region read it names was replaced by a parser, which answers the same requirement by
  construction.

- **The twin came back inside the module that exists to end twins, and it had already drifted.**
  `hermetic_git::run` and `run_exact` were the same eight statements with one `.map` inserted — the
  `Failure::Spawn` literal and the `Failure::Exit` construction character-identical, the two
  `Failure::Unreadable` sentences **not**. One fact reached an operator two ways depending on which accessor
  a caller reached for, and nothing compared them: the sentence occurs nowhere but its two constructions and
  one doc line, so no direction could see the divergence. `run` is now the trim over `run_exact` and the
  fuller sentence moved into the shared body rather than being dropped. A new direction holds that the two
  answer an undecodable read **in the same words** — not that the words are any particular ones, which would
  refuse an improvement to them.

  A doc comment annexed onto the wrong item moved with it: the extraction history of `run` opened
  `run_exact`'s doc, which the delegation would have made worse rather than better. The repository's own
  relative-anchor sweep refused the first repair of it for writing *the paragraph below*, which names a
  position rather than a thing.

- **The `list` projection's coverage guard was blind to a dimension, and a typed count agreed with it.**
  The guard says a capability added to `list_document` without a `list_markdown` section fails CI rather
  than silently under-projecting. `unsafe_confinement_boundaries` was in neither its enumerated table nor
  its fixture, so it was never looked at — and the parity half compared the **fixture's** document against
  a literal `9`. The fixture is what decides that document's keys, so the number and the document could
  only ever agree on the one input incapable of exposing the gap, while `list_document` had emitted that
  dimension the whole time. Measured: with `list_markdown`'s unsafe row deleted — a real under-projection —
  the guard passed. The fixture now populates all ten dimensions and parity is **set equality**, which
  names which member is missing where a count cannot. `constitution-projection`'s scenario requires the
  fixture to declare "one boundary of every semantic capability the JSON document can emit", so this is a
  conformance repair rather than hygiene; no specification text changes.

- **The head-branch re-read's own carriers, swept by the rule the same window widened.** The re-read landed
  and the summary comment over it still named **two** of the three inputs it now re-reads; the shared positive control was still
  named `an_unchanged_title_still_reaches_the_merge` while controlling three guards, and that title-only name
  was carried by `PINNED-BY` into the head-branch scenario as its evidence; the bound's rationale still said
  declaring it per input would be *two records*, a count the set had already outgrown; and the backlog said
  `gh` offers no equivalent precondition for *either title or base*, omitting the endpoint just added and the
  direction pinning it. The control is renamed for what it controls, and the rest state the criterion. Found
  by an independent review, in the window whose own subject is that identifiers carry superseded claims
  furthest — the rule was widened and then not applied to the change sitting beside it.

- **The retirement sweep's corpus says where the residue actually was.** The rule read *every live document*
  — corrected once already, from naming two files to naming the class — and that still reads as the files a
  narration sits in. Measured across this window, a superseded claim survived in a test function name, in
  doc comments beside two directions, in an inline comment in a shell script, in a scenario heading and in
  the `PINNED-BY` names under it. One test file carried a `///` stating the opposite of the assertions twelve
  lines below it, and a review reading the specification alone called that file coherent. The corpus is now
  every tracked live **file**, and the seeds are the wording **and** the identifiers carrying it. No check is
  added: this file already records a prose detector as designed, measured three times and rejected, and the
  bound rule already said the same thing for its own subject — what was missing was that it is the general
  case.

- **A claim the parser migration superseded is corrected wherever it was still written down.** Replacing the
  hand-rolled readers changed what this repository can decide, and the reaction moved while the carriers
  stating the old answer did not. `release-coherence` still required a **cannot-judge** for a quoted
  dependency key — `"xuanji" = "0.0.1"` — that the parser decodes and the cited pin judges, and still required
  a **refusal** for `[workspace]` carrying `package.version` as a dotted key or an inline table, which the
  reader returns as the declared version. Measured under cargo 1.96.0, every one of those spellings resolves,
  so refusing them was a false refusal over legal Cargo syntax. A cannot-judge is a claim about the *reader*;
  where the reader decides, saying otherwise stops an operator in front of a manifest cargo reads without
  difficulty.

  The residue was not only in the requirement. The pin held the superseded claim in its own **function name**
  and in the last paragraph of its **doc comment**, twelve lines above assertions expecting the opposite and
  an inline comment recording that the refusal had been false — a file disagreeing with itself, which a review
  reading only the specification reported as coherent code. Sweeping for the retired test name then reached a
  third carrier: a `BACKLOG.md` entry deferring this exact conversion, whose trigger had fired and whose open
  paragraph named a function, a state and a direction that no longer exist. Annotated in that file's idiom
  rather than rewritten.

  The dated section below states the old behaviour and is left as it stands: a record is a measurement of the
  moment it was taken.

- **Provenance trimmed from the published crates' rustdoc.** Doc comments that named the window or the review
  round a behaviour came from now state the behaviour itself: a canonical path label says why one separator
  makes a baseline portable rather than which release found that out, and the cycle guards say they share one
  canonicalize-failure policy rather than recounting how they once disagreed. **No documented guarantee, API,
  outcome, identity shape or exit class moves** — every changed line in every published crate is a comment,
  and the compiled behaviour is the `0.5.0` behaviour.

### Self-governance

- **Three directions proved their own reach by reading this repository's accumulated content, and each now
  reads a fixture instead.** A guard whose corpus is the host's deletion history, or its own document's
  marks, draws its colour from the host rather than from the reader under test — the wrong-grain failure this
  repository already records for a sibling direction inside one test binary. At the root commit all three
  went silent while the readers they defend were unchanged: two over `git log --diff-filter=D`, which answers
  nothing where nothing has been deleted, and one over the struck-through bullet, every instance of which the
  closed reproduction records held.

  The repair is one shape rather than three: the direction proving reach takes a fixture, and the direction
  judging the repository takes the repository and is allowed to be clean over an empty corpus. Each half was
  seen to fail. Dropping `.rs` from the bare form's extension list reports *a bare Rust basename — planted in
  probe-rust-basename.md and seen by nothing* and leaves the shell row seen, which is the defect that row
  exists for; making the bare form unconditional reports both rows; perturbing the closed-entry mark reports
  `left: 0, right: 1` while the direction judging `BACKLOG.md` stays clean, which is the split working.

  **No specification moved, and two candidates were measured rather than assumed.** The bare-filename
  requirement already states the discriminator, so the reach limit is a precondition with no instances rather
  than an undeclared stop. `Form::Legacy` is likewise specified, pinned and reachable in its fixtures, and
  `unreachable_branch` is green over it — so neither is drift a re-founding may retire, and declaring a bound
  for either would claim the law stops where it does not.

- **The direction that held the rollup's position asserted a weaker property than its own doc claimed, and
  its negative run wore the right colour for the wrong reason.** It listed the wrapper's five other reads
  and required each to come before the rollup, taking each one's **first** occurrence — and three of them
  occur twice, an early capture and a post-gate re-read, so the comparison was against the early capture.
  Measured: with the rollup moved to after the changed-file count but before the three re-reads — one of
  the positions the ordering exists to exclude — that form answers `ok`, while three `pr view` calls still
  follow the rollup. The run recorded when it was written moved the rollup a slot further back and went red
  on the changed-file count, which is a weaker property producing the same red.

  It asserts adjacency now: the rollup is the call immediately before the merge, and it is read once. That
  says what the requirement says, needs no list to keep in step with the wrapper's reads, and puts every
  other read before the rollup by construction rather than by enumeration. The negative run is at the
  position the enumeration passed, and the refusal names the count it found between them — three.

- **Three unreachable recovery arms, one of them holding a hang.** `run_with_stdin` took its three pipe
  handles one at a time, each with a refusal that had to unwind whatever the ones before it had started —
  and the second joined a reader while stdin was still held, which waits on a stdout that cannot reach EOF
  until the child sees one. Unreachable, since `Stdio::piped()` two lines above makes all three `Some`, so
  no arm ran and nothing hung. What it cost is a reader working out that three recovery paths are dead.
  They are taken together now, before any reader exists, so there is nothing started to unwind and *the
  child did not give the pipes it was built with* is one fact rather than three. A refusal rather than an
  `expect`: this module answers in `Failure` everywhere outside its fixture side, and a construction
  invariant is not a reason to add the first panic. Behaviour is unchanged, measured on the same probe.

- **A bound whose subject enumerated its inputs while its reason said the count is not written.** The
  post-gate re-read bound named *title, base branch or head branch* in the scenario's WHEN and in the typed
  `BoundDecl`, and said one clause later that it is *one bound rather than one per input… reached through
  whichever inputs are re-read* and that the count is deliberately unwritten. Adding what CI said to the
  re-read set left the enumeration short by one and the two halves disagreeing about their own subject. The
  subject is the class now — an input the wrapper re-reads after the gate — which is what the reason was
  already arguing for, and both projections are regenerated from it.

  Found by two independent reviews of this window, one per defect, each naming the line.

- **What CI said was read among the values the merge records, and it is one of the relations the merge is
  judged against.** The merge wrapper sorts its inputs by one question: does the merge RECORD this, or is it
  JUDGED AGAINST it? A recorded value travels as the value the gate saw; a judged relation has to still hold
  when the merge happens, which is why the title, the base and the head branch each gained a post-gate
  re-read. The rollup was filed with the recorded values — and it records nothing, since its value is local
  to the guard that reads it and nothing downstream sees it.

  What that leaves is a window in which a required check re-run on the **same head** turns the rollup red
  while every guard after it still passes: the head object has not moved, so `--match-head-commit` is
  satisfied, and none of the three branch names has moved. The read is the last guard now, once rather than
  twice — there being no value to record from an earlier one — so the window is this block's own remaining
  API calls rather than those plus a whole `cargo test`. `repository-checks` carries the ordering as a
  requirement with a direction pinning it, and the residual is the post-gate bound already declared: a
  client-side read cannot be atomic with the act it precedes, and that stop is reached through whichever
  inputs are read that way rather than through any one of them.

  **This is the third time that sorting has been got wrong in this file**, and the second time a sentence in
  it rested the claim on an ordering. The `--admin` arm said that passing the flag to force a red merge
  *does not work* because the rollup is refused **before `gh` is reached** — true of the read and not of the
  merge, with the read where it was. The arm now says what it is about the flag, and the ordering is where
  the property lives.

  **And where that flag reaches at all is measured, because a review read it as a privilege escalation.**
  Asked of this repository on 2026-09-08: the development base every squash lands on answers `404 Branch
  not protected`, so there are no required checks there to bypass, and the release base is protected with
  seven required checks and `enforce_admins` **enabled**, so GitHub refuses an administrator's bypass of
  them. The flag reaches required reviews, which is the use its arm states, and reaches no check on either
  base — so the stale reading was a race against this wrapper's own claim rather than a way past GitHub's.
  The measurement is dated and re-asked rather than trusted, being a fact about settings.

  Found by an independent review of this window, which named the ordering and the line; the security half of
  its verdict is the half the protection settings answer.

- **The runner that holds a conversation with `git` drained one of its two output pipes, and the other was
  the same hazard it had already paid for.** `run_with_stdin` gives stdout a reader of its own because the
  header above it records the measurement: 73,670 excluded paths, 9.1 MB in and 11.0 MB out against a 64 KB
  pipe, `git check-ignore` in `pipe_wait`, and `scripts/publish.sh` hung indefinitely. `stderr` was piped
  too and read only by the collection that runs *after* the delivery loop — so a child that fills the stderr
  buffer mid-conversation stops reading stdin, the parent blocks on a full stdin, and neither moves. One pipe
  over, in the one runner standing in front of `cargo publish`, and left to each caller to bet its
  subcommand's stderr is small.

  Both pipes now have a reader running with the delivery. **No negative run stands behind that half, and the
  reason is the argument set rather than an omission**: measured, `check-ignore` writes to stderr only to be
  fatal, in about a hundred and sixty bytes, and it exits there — so the buffer is not reachable through the
  one production call site. What stands in its place is the property. With both readers running, the shape is
  not writable at the runner level, which is what the header's own argument asks for: a list of what is safe
  is as complete as the last person's memory, and the case is the defence.

  **The other half of the repair is reachable, and it was measured.** A delivery that fails because the child
  has already refused is the child's refusal. Reported as a write failure it read *cannot write records to
  git […]: Broken pipe*, a sentence about this process for a fact about git's — the fold this module's own
  `Failure` doc records paying for one level up, where a machine without git was told something about the
  repository. Measured through the runner, with a record `check-ignore` is fatal about first and fifty
  thousand ordinary ones behind it: `Err(Spawn("cannot write records to git […]: Broken pipe (os error
  32)"))`, git's `fatal:` and its exit status both discarded. The answer is now
  `Err(Exit { code: Some(128), stderr: "fatal: … is outside repository at …" })`, and
  `publish-source-integrity` gains the scenario with a direction pinning it.

  Found by an independent review of this window, which named the pipe and the line.

- **A rule whose subject is every tracked live file had a reader over comment lines, so Markdown was
  unread — and it is read now.** `prose_of` classifies Markdown as `Prose::Whole` and the relative-anchor
  sweep filtered to `Prose::LineComment`, so `AGENTS.md`, `BACKLOG.md`, `PROJECT.md` and every specification
  sat outside the corpus entirely. Four review rounds found anchors there one at a time, by hand, and a fifth
  found the one this entry's own predecessor had claimed exempt.

  **The bound that covered the gap was already declared, and it narrows rather than retires.** Its reason was
  that a widened sweep would report four groups a reader over text cannot separate: the rule's own declaring
  row, duration rather than pointer, a generated projection's copy of either, and phrases already anchored.
  Three of those are separable **by shape**: a record carrier by path and by dated section; a marked
  quotation — the phrase in backticks or single emphasis, which is how this repository quotes a phrase to
  define it; and a projection's copy of either, which is a marked quotation by the same test. What is left is
  duration, and it is one phrase — so the bound is now *for a window* alone, and the clause saying the rule
  must stay wider than its reaction because *prose is where a reviewer must* is gone, being false for the
  other three.

  **The reader found the class in a shape a hand sweep could not, which is the argument for building it.** A
  per-line measurement of the same corpus reported one offence; the paragraph-joining reader reported three,
  because two had `this window` wrapped across a line break — verbatim the failure `AGENTS.md` records for
  the comment sweep before it was line-joined. All three are named now: two by restating the mechanism as a
  property, one by naming the rule it points at.

  **The measurement that shaped this was written three times wrong before it was right**, which is why the
  reaction earns its place over another round of reading. A `startswith("## ")` boundary matched `### `
  subheadings; an `elif` that reopened a dated span without closing the previous one left the largest section
  unexempted and reported 144 offences in a file holding three; and the per-line pass above missed two of
  three. The reader's own span logic is the fourth spelling, and its doc says so.

- **The merge wrapper's repository-identity guard was defeated by three environment variables, and the
  wrapper ran to completion.** `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` move which repository a git
  command acts on, past `current_dir` and past `-C`. `hermetic_git::hermetic` removes them for every git this
  repository builds in Rust, with a doc saying why; `scripts/merge-pr.sh`, standing in front of the one
  irreversible act, inherited them.

  What they defeat is the guard rather than a read. It compares the worktree holding the wrapper's gate
  against the worktree its evidence comes from, so that a wrapper invoked by absolute path from another
  checkout cannot judge one repository's pull request by another repository's law. Measured with the two
  pointed at a third repository, both reads answer the decoy — so the comparison **passes** in exactly the
  arrangement it exists to refuse, vouching for an equality about a tree that is neither the gate's nor the
  evidence's. The negative run shows what follows: exit `0`. Not a passed guard and a later refusal — the
  wrapper merged.

  The hazard was already written down in the direction that guards the same seam: *a fixture built under an
  ambient `GIT_DIR` is not the worktree this direction believes it made*. The fixture builder was made
  hermetic and the wrapper under test was not, in the same file. The selectors are cleared before anything
  reads a repository now, and a direction sets them at a decoy. The configuration channels are out of scope
  deliberately: this script's git calls are `rev-parse --show-toplevel`, which configuration does not move.

- **A red rollup named the wrong gate.** The Definition of Done job ran `whitespace_hygiene`,
  `repeated_paragraph` and `hermetic_invocations` under one step called *whitespace hygiene across every
  tracked text file*, so a refusal from either of the other two reported under a name that misidentifies it
  — the same class as *a refusal says which refusal*, one level out in the workflow. The two gates added
  alongside them each got their own step; these two were appended to an existing one. Each has its own named
  step now.

  **`dod_coherence` cannot see this, and the two instruments that would were measured and declined.** It
  holds each Definition-of-Done command against the workflow as a flat line set, so a gate appended under an
  unrelated name passes by construction. Generating the job's steps from the `AGENTS.md` list would need a
  *fragment* projection: `projection_register` recognizes a generated document by a marker in its header, so
  a partially generated file must either declare the whole of `ci.yml` generated or carry a comment-delimited
  region — and `region.rs` exists because *a comment made a file count as holding a projection*. A wholly
  generated composite action avoids that and reports its inner steps under the calling step's name, which is
  the finding. The decidable check — one Definition-of-Done command per step — is declined on cost: it turns
  every present and future command into a step of its own, which is a permanent authoring tax against a
  bounded, visible failure. `BACKLOG.md` carries the class as a `WATCH`, whose trigger is a second instance,
  because one is a typo and two is a shape.

- **A repair that typed three answers apart left a fold at the `Path::parent` call in front of it.**
  `repository_path` answers `Below`, `Outside` or `NotUtf8` precisely so a consumer cannot read a missing
  value its own way — and the site that asks it about a *member directory* reached `Path::parent` first,
  taking its `None` through an `else` arm that returns `member-manifest-outside-workspace-root`, whose
  message asserts the manifest is not under the workspace root. That is false of a path with no parent, and
  the site's own comment argues against exactly this shape: *a `let … else` reads every answer
  that is not the one it wants as the one fact its `else` names.*

  The question moved to the owner, so there is no longer an `Option` in front of any caller. What the extra
  answer is *not* is a fourth `RepositoryPath` variant: the compiler refused that, because the plain path
  question would then have to match a state it can never produce, and the only arms available there are the
  fold being repaired or an `unreachable!` that `unreachable_branch` refuses. A second enum restating the
  three answers would be two lists that must agree. `DirectoryOf` **wraps** instead — `Directory(…)` plus
  `HasNoDirectory` — which costs one `match` at the single site asking the question and nothing anywhere
  else.

  The new refusal site is **declared unheld** rather than held by a fixture, with the reason its sibling
  already uses for the same shape: `cargo metadata --no-deps` reports absolute manifest paths and every one
  has a parent, so a manifest with no directory is not a shape cargo produces. The function's own behaviour
  *is* held, at the unit level where the state is reachable. `docs/refusal-register.md` moves to 148 sites
  with 20 declared unheld.

- **A bound claimed a stop in both string-literal forms and cited the evidence for one.** Its subject said
  *either form* and its `because` said *one stop remains, in both forms*, while its citation named only
  `a_construction_inside_an_ordinary_string_literal_is_not_read`. The direction holding the raw form was
  cited nowhere — not by the declaration, not by either projection — so it could have been deleted with the
  register, both projections and every gate unchanged, leaving the bound claiming a stop it had half the
  evidence for. It cites both now, through `BoundDecl::pinned_by_many`.

  **Two directions defending one bound, not two bounds.** Reading what the second direction demonstrates
  settled that: a lexer answers both literal forms the same way, so the raw form's earlier over-report is
  *closed* rather than declared, and there is one stop reached two ways. Splitting it would have declared two
  stops where there is one, which is a worse fault than the under-citation — a bound register is only as
  honest as the count of stops it names. The scenario says so in its own clause now, so the next reader does
  not have to re-derive it from the directions.

  The scenario's `WHEN` said *inside an **ordinary** string literal* while the declaration's subject said *of
  any form* — a third instance of the same class in one bound, and one nothing could hold: the spec-to-code
  bijection is over bound **ids**, not over the shape text either side spells. Both now say *either form,
  ordinary or raw*.

- **A deliberate stop was argued in a doc comment and declared nowhere.** The publish gate refuses rather
  than judging the cleanliness of a worktree holding a path that is a legal filename and not UTF-8, and the
  reason for that — `xingbiao::path_identity` keeps two identities for two paths differing only in
  undecodable bytes, so collapsing them here would contradict the product's own rule — sat in a `///` where
  no register could see it. Two independent reviews read the same code and reached opposite conclusions
  about whether it was policy or a defect, which is what an undeclared stop costs.

  It is an observation bound now, pinned by a direction that builds the tree rather than reasoning about it:
  measured, `git ls-files -z --others` answers a `stray\xff` file as `stray\377\0`, verbatim, so the worktree
  read stops before any cleanliness question is asked. Its declared mutation makes that read swallow the
  failure instead, and the gate then reports `ok publish source` over a tree it never read — a clean pass in
  front of an act that cannot be undone, reached by a legal filename.

  It is also this family's **first** `Reached::RefusesToJudge` bound. The variant was declared for the
  distinction it draws — a fail-loud refusal is not the silent false negative a backlog entry once predicted
  for it — and until now nothing in the tree held it. It is not a declared false negative and the projection
  counts it as neither: `docs/observation-bounds.md` moves to 105 declared bounds with the false-negative and
  unpinned figures unchanged.

- **One gate read git's answer under two policies.** The publish gate's exclusion classifier spelled its own
  `from_utf8_lossy` while every other read in the same gate refused an undecodable answer through the shared
  runner — and the strict one is the policy `hermetic_git::answered` states, from `xingbiao::path_identity`.
  Nothing downstream could tell, which is why no direction caught it: the classifier's paths arrive
  already strict-decoded, so the lossy call had no reachable effect at the site it was reached from. A policy
  that is right by accident everywhere it is reached from is still two policies.

  The conversation — NUL-separated records fed on stdin, the answer drained while the question is still being
  asked — now sits in `hermetic_git` beside the other two accessors, and one internal `answered` decides both
  the exit-status disposition and the decode for all three. The deadlock property that shaped that
  orchestration is unchanged, and its direction is what guards the move rather than a new test written for it.

  **Converging them found a third fact the gate had been folding.** The strict decode is reachable through the
  classifier with an all-text question, which the fold's reasoning had ruled out: `check-ignore -v` answers
  with the *pattern* that matched, and a `.gitignore` is arbitrary bytes. Measured — a `.gitignore` holding
  `f[o\xff]o` against an untracked `foo` answers `.gitignore\01\0f[o\xff]o\0foo\0` at exit `0`. Read as *the
  classifier could not run*, that told an operator `An unusable classifier is not one that found nothing`
  about a classifier that ran, answered, and exited `0`. It has its own cannot-judge site now. Both states
  are cannot-judge, so no exit class ever separated them; what separates them is the subject an operator is
  sent to — a machine, or a `.gitignore` in the repository under judgement.

- **Two refusals shared one message, and two output streams shared one line.** Neither makes a gate report
  clean; both make it name the wrong fact when it does not, which is the class this repository refuses in
  its own words.

  `summary` answered `None` for *no summary produced* and for *more than one*, and the report said the
  first — so a run that produced two summaries was reported as producing none. It answers which:

  ```
  left: Err(Missing)   right: Err(Multiple(2))
  ```

  And a failed run's two streams were trimmed and concatenated, so a non-empty stdout's last line ran into
  a non-empty stderr's first. They carry labels and a newline now.

  **Scoped deliberately.** Two narrow repairs, no spec change — the contract was already right — and no
  proof-run framework, which would add a reviewing surface to close a diagnostic gap. What a check reports
  when it refuses is worth being exact about; what it would take to make that exactness general is not, at
  one caller.


- **Three places where the evidence's type still admitted a state its contract calls impossible.** All three
  are the same shape one level down from the last change: a token where a parse belongs, a list where a set
  belongs, a path where an owner belongs.

  **`contains("1 passed")` is a token, not a result.** Any output carrying those words satisfied it — a build
  line, another target's summary — and it cannot tell one passing test from one passing test beside a
  failing one. A libtest summary opens its line and carries both counts, so it is parsed, and there must be
  exactly one of it: two summaries are two runs, and which carried the proof is not decidable from here.
  Negative run, with the meta-run's filter pointed at a name no target declares:

  ```
  running `no_ambient_channel_moves_…` reported 0 passed and 0 failed, where the proof is exactly one test passing
  ```

  **The baseline was a `Vec` where the contract says a set.** *Two sets, not one* was written into the
  inventory and into the spec, and a variable repeated in it was accepted and kept. The type refuses it now,
  at ingestion:

  ```
  the inventory clears GIT_CONFIG_NOSYSTEM twice; a baseline is a set, and a repeated row is a row nobody can act on differently
  ```

  **And the owner did not own its own evidence.** `shengmo::hermetic_probe` held the inventory's path as
  `crates/kanhe/tests/fixtures/…` — reaching into a crate that *depends on it* for the file it is built
  around. `kanhe` could move or delete it, and the two consumers that cannot reach `kanhe` at all were
  reading a path inside it. The inventory sits beside its owner.


- **Two weaker sets had been standing in for two stronger ones, and both are now separate things.**
  *Registered tests* is not *tests that ran*, and *channels worth attacking* is not *variables that must be
  emptied first*. Each had been one list doing two jobs.

  **A proof is run, not listed.** Four readers said otherwise in turn: `sig.ident` alone admitted an
  ordinary function; `#[test]` plus a refusal of `#[ignore]` admitted `#[cfg(any())] #[test]` and an
  `ignore` reached through `cfg_attr`; and asking the harness to *list* the direction admitted an ignored
  test, under whichever features the listing was taken with rather than the `--all-features` CI runs. The
  direction is executed now — `--all-features`, `--include-ignored`, and the run must report exactly one
  test, because a filter matching nothing also exits zero. Negative run, on a proof that runs and fails:

  ```
  crates/shengmo/tests/family_coverage.rs: `no_ambient_channel_moves_…` did not pass:
  assertion `left == right` failed: a proof that fails
  ```

  And the review's own falsifier is closed rather than merely detected: a proof left `#[ignore]` **still
  runs**, because `--include-ignored` is what the meta-run passes.

- **The baseline was derived from the attack cases, and so missed a variable no case attacks.** The builder
  sets `GIT_CONFIG_NOSYSTEM`; nothing injects it; so nothing cleared it, and a host that carries it
  suppresses the system-config channel — collapsing the `GIT_CONFIG_SYSTEM` case's control. The inventory
  carries both sets now, and a channel attacked without being cleared is refused. Negative runs:

  ```
  the inventory attacks GIT_CONFIG_SYSTEM and does not clear it first, so what a case demonstrates could be an inherited channel rather than the injected one

  a bare `Command` read the same under GIT_CONFIG_SYSTEM as without it   left: "isolated"   right: "isolated"
  ```

  The second is the review's finding, demonstrated: with `GIT_CONFIG_NOSYSTEM` out of the baseline and set
  in the host, the case proves nothing and says so.

- **One owner holds the evidence; each site holds only its builder.** Three runners parsed the inventory,
  validated it their own way and assembled the child's environment by hand — and what drifted was not the
  builders but the evidence: one baseline missed a variable another's had, and a count each accepted stood
  in for a set none of them held. `shengmo::hermetic_probe` owns the parse, the validation, the injection,
  the report's shape and the judgement.

  It lives in `shengmo` because that is the **only** member both consumers reach without a new edge or a
  published promise: measured across the workspace, `kanhe` depends on `shengmo`, the two copies are
  `shengmo`'s own test targets, and every other crate reachable from both — `tianheng` and what it carries —
  ships. `shengmo` ships in no package, and shared test support is the role it already had.


- **Whether a proof runs is an execution result, and three attribute readers had said otherwise.** The
  citation compared `sig.ident` alone, so an ordinary function of the same name satisfied it; then it
  required `#[test]` and refused `#[ignore]`, and `#[cfg(any())] #[test]` — or an `ignore` reached through
  `cfg_attr` — satisfied it while never entering the harness. Each repair closed the spelling a review
  brought and left the next, because *whether a test runs* is decided by cfg evaluation and the harness
  registry rather than by attributes a reader can see.

  It is asked of the harness now: the target is listed and the direction must be in what it lists. Negative
  run, using the review's own falsifier:

  ```
  crates/shengmo/tests/family_coverage.rs: the harness does not list `no_ambient_channel_moves_what_the_family_coverage_builder_reads`, so nothing runs it
  ```

- **A row count is not a set.** The inventory was held at `len() >= 8`, so a row could be replaced by a
  second row for a channel already listed — the count intact, the channel it displaced asked about by
  nobody. All three consumers refuse a repeated channel now. Negative run, replacing `GIT_CONFIG` with a
  second `GIT_DIR` row:

  ```
  the channel inventory names GIT_DIR twice; a repeated row makes the count without making the case, so the channel it displaced is asked about by nobody
  ```

- **"One channel per case" was an addition, not a baseline.** Each case injected its channel onto the
  environment this test binary inherited, so what its control demonstrated was *a* channel rather than *the*
  channel. Every governed channel and the two indexed helpers are cleared before one is injected.

  **It has no negative run, and that is stated rather than left as a gap.** Every governed channel is closed
  by the builder, so an inherited one cannot flip a verdict: the fix is to what a passing case *attributes*,
  not to whether it passes. Measured with `GIT_DIR` set in the host and the baseline removed — no case
  changed. A property a single edit cannot falsify is one this repository records as carried by reading.

- **The module header still described the model this file no longer holds.** It named an `Isolated` state
  that no longer exists and claimed a comparison of environment operations that was deleted. It states the
  two questions the file actually answers: which tracked files construct their own `git`, and whether a
  declared site names a direction the harness lists.


- **The behavioural matrix was written out per site, and a matrix per site is a matrix that diverges per
  site.** Four cases stood in each of three files — the three repository selectors and
  `GIT_CONFIG_PARAMETERS` — while `GIT_CONFIG`, the two configuration-file channels and the indexed channel
  were in none of them. The previous change called that *the whole matrix*; it was half of one, and the
  claim is corrected here rather than left standing. It is the transcription failure one level above the one
  the probes were built to end.

  `crates/kanhe/tests/fixtures/hermetic_channels.tsv` holds the cases once — channel, injection, the
  observation it moves, the isolated reading — and the owner and both boundary-forced copies consume it, so
  a channel added there is a case every builder starts owing. Eight cases now, each with its own
  bare-command control. Negative run, on a copy whose old matrix never had the channel:

  ```
  this builder followed GIT_CONFIG   left: "ambient-probe"   right: "isolated"
  ```

  **The first observation was the wrong one, and the control said so.** Reading `user.name` let the fixture's
  own repository config outrank the file channels, so `GIT_CONFIG_GLOBAL` moved nothing and the case would
  have passed for the wrong reason. It reads a key no repository sets, with `--default`, so an isolated
  command answers and exits zero.

- **A `git` that failed was read as an isolated answer.** The probe folded the child's exit into stdout, and
  the worktree case's isolated value **is** the empty string — so a `git` that could not run passed as
  isolation. Each reading now carries its status and its stderr:

  ```
  the builder `git` under GIT_WORK_TREE exited 129, so its reading is a failure rather than an answer: error: unknown option `no-such-flag'
  ```

- **A name is not a proof, and neither is a `#[test]` that is ignored.** `ProvenBy` compared `sig.ident`
  alone, so an ordinary function of the same name — or the same test with its attribute removed — satisfied
  a citation nothing runs. The attribute is required now, `#[ignore]` refused, and a name declared twice
  refused as a citation naming a set.

- **A `crate::std` is not the standard library, and stripping the root said it was.** The alias reader
  normalised `self::` and `crate::` away before comparing the path, merging a module this repository could
  define with the external crate — a false positive in the direction that reports a construction where there
  is none. Those roots are refused; `std::process` and its absolute spelling are admitted.

- **Deleting one guard at a time said which line carries which channel, and twice it was not the one the
  builder claimed.** Measured:

  ```
  GIT_CONFIG_SYSTEM alone removed    still green   — NOSYSTEM carries it
  GIT_CONFIG_NOSYSTEM alone removed  still green   — SYSTEM carries it
  both removed                       BITES         — they are redundant with each other
  GIT_CONFIG_COUNT alone removed     still green
  GIT_CONFIG_KEY_0 alone removed     BITES         — occupying index 0 is what closes the channel
  ```

  `hermetic`'s header said its defence against the indexed channel is **occupying the count**; it is
  occupying the **key**. Both are corrected in place and recorded in the inventory, redundancy included:
  redundancy that has been measured is a different fact from redundancy nobody checked.


- **The responsibility is split: a reader of syntax answers ownership, and a run answers isolation.** Ten
  rounds walked one source model through a rename, a macro, a literal compared by its rendering, a constant
  kept while its loop was deleted, a removal made on a decoy receiver, a nested macro token, and a
  `foo::process::Command`. Each was real, each was closed, each was followed by the next — because the
  question being asked of syntax was *whether the command this program builds is isolated*, and that is a
  question about running it.

  **The environment modelling is deleted, not extended.** `Operation`, `Environment`, `IteratedRemoval`,
  `expected_operations` and `environment_operations_of` are gone. What remains of the reader is the
  ownership question it can answer: which tracked files construct their own `git`, held two-directionally
  against a declared set, and — for a site that cannot reach the builder — that it **names** the direction
  proving its isolation and that the direction exists. An item's name is what a parser reports.

  **Every channel is injected alone, with a reading it moves.** The first probe set all three repository
  selectors and read `git log`, which is sensitive to `GIT_DIR` alone — measured:

  ```
  kept selector      git log -1 --format=%s   git ls-files   git status --porcelain
  GIT_DIR            decoy                    decoy.txt      D decoy.txt
  GIT_WORK_TREE      judged                   judged.txt     D judged.txt
  GIT_INDEX_FILE     judged                   decoy.txt      RD judged.txt -> decoy.txt
  ```

  So a builder clearing one of three passed. Each channel now arrives alone with a reading it moves —
  `log` for `GIT_DIR`, `status` for `GIT_WORK_TREE`, `ls-files` for `GIT_INDEX_FILE`, `config --get` for
  `GIT_CONFIG_PARAMETERS` — each carrying the bare-command control, without which a pass proves only that
  the channel never arrived. Both boundary-forced copies carry the whole matrix, configuration included,
  which the source comparison could never have covered. Negative runs, one per channel:

  ```
  a command this builder made followed GIT_WORK_TREE   left: "D judged.txt"   right: ""
  a command this builder made followed GIT_INDEX_FILE  left: "decoy.txt"      right: "judged.txt"
  this builder followed GIT_CONFIG_PARAMETERS          left: "ambient-probe"  right: "judged"
  ```

  The two syntactic gaps this round found are closed at the size the narrowed responsibility makes right: a
  macro body's tokens are walked to **any depth**, so a construction nested inside a delimiter is not read
  as nothing; and an alias binds on the **whole path** being `std::process`, not on a segment named
  `process` occurring somewhere, which had made somebody else's `Command` this one.


- **The property is asked of a run now, because nine rounds proved the reader could not be finished.** Every
  round a review supplied a spelling the source reader missed — a rename, a macro, a literal compared by its
  rendering, a constant kept while its loop was deleted, and now a removal made on a **decoy receiver** — and
  every one is a different way to write the same program. `pin_bites` already says the thing this needed:
  *whether a test bites is a question about running a program and no reading of text answers it.* Whether a
  `git` inherits the ambient environment is that same kind of question.

  A child process inherits a decoy `GIT_DIR` / `GIT_WORK_TREE` / `GIT_INDEX_FILE`, runs the builder against a
  known repository, and reports what it read beside what a bare `Command` read in the same environment. The
  bare reading is the control: without it a pass proves only that the selectors never arrived. Both
  boundary-forced copies carry the same probe, so each proves its own isolation by running rather than by
  spelling.

  **The decoy-receiver falsifier is where the two readings part, and it is measured.** With `hermetic`'s
  removals redirected to a decoy `Command` while the returned one stays ambient:

  ```
  the syntactic check:  test result: ok
  the run:              left: "decoy"   right: "judged"
  ```

  The reader is not retired — it is early warning, and it catches transcription drift before a run does. It
  is no longer the only thing standing between a copy and an ambient `git`.

- **Two more spellings, closed as far as syntax reaches.** A `use std::process::Command as Cmd` **inside a
  function or an inline module** bound nothing, because aliases were collected from top-level items alone;
  they are collected by a visitor now, which reaches every scope. And the path is checked rather than only
  the rename — binding on the segment `Command` alone would have made `use foo::Command as Cmd` a
  `std::process::Command`, widening the reaction rather than the bound.

  A **statement-oriented** macro body — `passthrough!(let _ = Command::new("git");)` — is not an expression
  list, and discarding that parse failure let it carry a construction past the reader. Statements are parsed
  too; and a body neither grammar accepts that **names something bound to `Command`** is now
  `Reading::Undecidable` rather than silence, which is the safe direction where the alternative is reporting
  a file clean over tokens nothing classified.


- **A constant stood in for an operation the builder no longer performed.** The two arrays `hermetic`
  iterates were expanded into `env_remove` operations because the arrays still existed — so deleting
  `for selector in REPOSITORY_SELECTORS { command.env_remove(selector) }` while keeping the array left this
  check green over a builder that had stopped clearing every repository selector. Membership is not an
  operation; the loop is what performs one. What is read now is a `for` loop in `hermetic` whose iterable
  **is** that constant and whose body calls `env_remove` on the element it binds. Negative run, using the
  review's falsifier:

  ```
  `hermetic` no longer iterates both arrays into `env_remove`, so this reader would derive a set from whichever half it still performs
    left: 1
   right: 2
  ```

- **A rename and a macro each carried a construction past the reader.** `use std::process::Command as Cmd;
  Cmd::new("git")` was missed, because only the segment `Command` was read; `dbg!(Command::new("git"))` was
  missed, because a macro's tokens are not expressions until something parses them. Both are valid Rust in
  an undeclared file that this check reported as constructing nothing. Renames the file itself binds are
  bound now, and a macro body is parsed as an expression list where it is one.

  **The floor that leaves is declared rather than walked toward again.** What a name means when it is bound
  *somewhere else* — another module's rename, a type alias, a re-export — is not written down anywhere a
  parse tree carries, and answering it needs name resolution. This repository's other reader of its own Rust
  reached that same floor and declared it; this one now cites it by the same road, pinned by the control
  that a name the file does not bind is not read, and carrying the mutation record a new bound owes.


- **Reading tokens was not the repair; reading *syntax* is.** The round before replaced a substring over a
  trimmed line with a token walk, and the readers stayed **positional** — `trees[index - 3]` for the owner
  segment, a literal compared by `to_string`, a value read at `inner.get(2)` or dropped. Three findings came
  straight back, all in that arithmetic:

  - **A literal was compared by its rendering.** `r"git"` and `"\x67it"` both decode to `git` and both
    rendered unequal to `"git"`, so a construction written either way was not read. A parser decodes.
  - **An operation was modelled as a variable name.** The set could not tell `.env_remove("GIT_DIR")` from
    `.env("GIT_DIR", "/tmp/other")` — a copy that *points* the selector somewhere satisfying a requirement
    that it *clear* it — nor `GIT_CONFIG_COUNT` pinned to `"1"` from the same variable set to `"0"`, which
    reopens the ambient-key channel the builder's own header spends a paragraph closing. Both are the
    review's own falsifiers, and both now fail:

    ```
    crates/shengmo/tests/family_coverage.rs: makes no `.env_remove("GIT_DIR")` call
    crates/shengmo/tests/family_coverage.rs: makes no `.env("GIT_CONFIG_COUNT", "1")` call
    ```
  - **A file it could not parse was reported clean.** The tokeniser's failure arm fell back to an exact
    substring, which answers *no construction* for one split across lines — silently, which is the one
    direction the Core Contract forbids. The reading is three-state now and the corpus direction names the
    path it could not decide.

  **`refusal_register`'s own header records this repository learning this once already**: a reader that is
  *text over Rust* is not exhaustive over the language, and reading its own Rust with a real parser is what
  closed that floor. This check re-derived the same mistake in two stages. It asks `syn` now — a call's
  callee, its argument count and each argument's decoded value are what a parse gives, and none of them is
  an offset from something else. The `item_body` token scanner is gone with it: an item is named by the
  parser, so there is no terminator to pick.

  **Measured, for the sibling question this raises.** Twenty-five checks read this repository's own Rust;
  four ask a parser. Most of the rest judge Markdown or TOML rather than Rust syntax, but one is the same
  shape: `gate_exit_classes`' spawn detector reads lines and excludes only a preceding quote, so a
  `Command::new(` inside a comment would count as a spawn — and its own header already records being *one
  form short, three rounds running*. Measured across every test target: **zero** files reach it by prose
  alone today, so it is latent rather than live, and it is left as a separate subject rather than folded in
  here.


- **Three rounds of findings in one file were one cause: it read Rust as lines where the question is about
  code.** Each round repaired an instance and left the cause — a substring over a trimmed line, a span
  sliced to a searched-for terminator, a variable name looked for anywhere in non-comment text. This round
  the reader asks `proc_macro2` and `syn`'s own item shapes instead, and three findings close together
  because they were never three:

  - **A block comment was read as a construction.** The test was on the line's opening `//`, so
    `/* … */` carrying the spelling passed straight through — the prose bound saying the opposite, two files
    away. Comments are what a lexer discards, so every comment form goes at once.
  - **A construction rustfmt split across lines was not read.** Neither the opening nor the argument carried
    the whole spelling, so a file whose construction wrapped was absent from the constructing set and the
    check's own requirement was wider than its reader. A line break is not a token.
  - **An inert string satisfied the isolation check.** `line.contains(variable)` over non-comment text is
    satisfied by `let _ = "GIT_DIR";` — it asked whether a **name occurs** where the property is whether a
    **call happens**. The line filter it carried was itself a repair of this same shape one round earlier,
    when the paragraph explaining the isolation named every variable it removes. Reading the `.env` and
    `.env_remove` calls closes both. Negative run, using the review's own falsifier:

    ```
    crates/shengmo/tests/family_coverage.rs: makes no `env`/`env_remove` call naming GIT_DIR
    ```

  The first spelling of the token reader carried a `let` chain, which this crate's MSRV predates and the
  workspace toolchain compiles without a word — the MSRV job is what said so, which is the job's whole
  reason for standing on its own line.

  **And the span slicing is gone rather than patched again.** A `const`'s value is the group after its `=`
  and a `fn`'s body is its braces; a group carries its own end, so there is no terminator to pick and
  nothing to get wrong. The `.min()` repair of the round before was the right answer to the instance and
  this is the answer to the class.

- **Reading tokens closed an over-report that had just been declared.** The raw-string half of the
  string-literal stop — a fixture written `r#"…"#` reported as constructing a `git` — was measured, stated
  as behaviour and pinned one round earlier. A literal is one token, so both literal forms now answer the
  same way and the over-report is **closed rather than carried**. One stop remains, in both forms, and the
  bound says so.

  **Two of that file's three bounds now have no mutation record, and the table says why rather than leaving
  two silent gaps.** *A `git` named in prose is not read* and *a `git` constructed inside a string literal is
  not read* are what a token stream **is** — comments are discarded, and a literal's contents are not a
  stream — so perturbing either means giving the reader a text path back, which is the defect rather than a
  perturbation of it. Coverage reads **7 declared mutations covering 7 of 242 cited tests**: down from nine
  because two claims that could not be proven were withdrawn rather than left standing.


- **A span reader took the next function's closing brace for a one-line constant, and the arm written for
  that case was a branch no input could reach.** `environment_operations_of` searched `"\n}\n"` first and
  `"];\n"` only as a fallback — but a single-line `const` has a closing brace after it too, the next
  function's, so the fallback never ran. Measured: `CONFIG_CHANNELS` got an **87-line** span where its
  subject is one line and `REPOSITORY_SELECTORS` a **45-line** one, both swallowing the whole of
  `run_exact`. The dead arm is separately the shape this repository refuses in its own words.

  Latent, and only because the per-line comment filter kept the builder's own doc table — which names
  `GIT_OBJECT_DIRECTORY` and `GIT_ALTERNATE_OBJECT_DIRECTORIES`, variables `hermetic` deliberately does not
  clear — out of the derived set. Negative run, with an unrelated `GIT_PAGER` read inserted into
  `run_exact`, showing the reader report the builder as disagreeing with a check about a function that had
  not changed:

  ```
  the environment operations `hermetic` makes differ from the set this check requires of a copy
    left:  {…, "GIT_INDEX_FILE", "GIT_PAGER", "GIT_WORK_TREE"}
   right:  {…, "GIT_INDEX_FILE", "GIT_WORK_TREE"}
  ```

  The end is the **earlier** of the two offsets now, not the first that matches. The same insertion passes.

- **The third stop was mis-stated twice, in opposite directions, and both halves are real.** It was written
  first as *a construction inside a string literal is not read either* — false for a raw string, which
  carries the spelling verbatim and **is** reported. Correcting that, it was then written as **not a stop at
  all**, which threw away the half that is one: an **ordinary** literal spells the construction with its
  quotes escaped, so the file drops out on its own, and a file that emits Rust and compiles it carries
  exactly that.

  The two halves point opposite ways, so they are dispositioned differently rather than together. The
  ordinary-literal half is the silent one and is declared —
  `repository-checks/a-git-constructed-inside-a-string-literal-is-not-read-a-stated-bound`, with the reason
  the sibling requirement already carries: separating a literal from the code around it needs a lexer, which
  `repeated_paragraph` carries and this check does not. The raw-string half is an over-report — visible and
  arguable — so it is pinned as behaviour under its own name rather than declared as a bound.

  The bound arrives with its mutation record, which is the rule the previous change installed rather than a
  courtesy: coverage moves to **9 declared mutations covering 9 of 242 cited tests**, and the record was run
  and seen to kill the pin it names.


- **A pin was proven to resolve and almost never to bite, and that is where six rounds of findings came
  from.** Measured across this window: **61 `PINNED-BY` citations added, and no mutation record** — the count
  stood at four before it and four after. `bound_register` decides that a citation names a test the harness
  registers; only `pin_bites` decides that the test would fail if the thing it defends changed. So a bound
  could ship citing a test about a **different stop**, twice, and both times a review found it rather than a
  reaction: first `a-paragraph-repeated-out-of-line-is-not-read`, then
  `a-git-constructed-through-a-program-value-is-not-read`, whose pin exercised the prose stop instead.

  **Four bounds is what this window declared, and proving four is affordable where proving 300 is not.**
  Each now carries a record that was run and seen to kill the pin it names: coverage moves from *4 declared
  mutations covering 4 of 241 cited tests* to **8 of 241**, and the run costs 4 seconds more. The asymmetry
  that produced the drift is that a citation costs one line and its proof costs a build; the affordable
  correction is to pay it where the claims are new, which is exactly the population the reviews kept finding
  defects in.

- **Three declaration defects in the check built one round earlier, and one of them was a claim that was
  simply false.**

  **A constant compared one way where its own doc said two.** `ENVIRONMENT_OPERATIONS` was written out by
  hand and only checked *into* the builder, so it caught the builder dropping a variable and never the
  builder gaining one — a twelfth `env_remove` in `hermetic` would have been required of no copy, and both
  boundary-forced copies would have fallen silently behind. That is the partial-transcription class the file
  was written to close, in the file's own constant, twelve lines below a sibling constant compared both ways
  with the reason written out. It is derived from `hermetic`'s own span plus the two arrays it iterates —
  not the whole file, since the fixture-side `commit` names `GIT_AUTHOR_DATE` and `GIT_COMMITTER_DATE`, which
  are no part of what a read inherits. Negative run, with a `GIT_CEILING_DIRECTORIES` added to the builder:

  ```
  the environment operations `hermetic` makes differ from the set this check requires of a copy
    left:  {"GIT_CEILING_DIRECTORIES", "GIT_CONFIG", …}
   right:  {"GIT_CONFIG", …}
  ```

  **A bound pinned by a test about a different stop.** `constructs_git` makes two stops and one test held
  both, so the program-value bound cited a name about prose. Split, each pinned by a direction named for it,
  and the prose stop is declared as a bound of its own rather than living only in a `///` the register
  cannot reach.

  **And the third stop was not a stop.** The doc claimed a construction inside a string literal is unread;
  measured, the direction is the opposite for a **raw** string, which carries the spelling verbatim and *is*
  reported. An ordinary literal escapes the quotes and so drops out on its own rather than by any decision.
  That is an over-report — visible and arguable where a miss would be silent — so it is stated as behaviour
  and pinned, not declared as a bound that does not exist. Writing the fixture out made this file report
  itself, which is how the correction was measured; both fixtures are assembled, because declaring this file
  exempt to hold one would blind the check to a real construction added here later.


- **An example directory the release gate could not stat was skipped as one holding no
  example, and the stale pin behind it reached `cargo publish` unjudged.** `is_dir()` answers `false` for
  *not a directory* and for *this reader could not stat it*, so the entry it could not reach was passed over
  — the identical collapse the manifest read in the same loop goes to length to avoid, one step before it.

  **The count floor does not catch this, which is why it survived.** `example_manifests == 0` catches every
  example being unreachable; the dangerous shape is *one of several*, where the readable examples carry the
  count, the floor is satisfied and the gate reports clean. Negative run with the collapse restored, over a
  fixture holding one readable example beside one entry that cannot be stated:

  ```
  an entry this reader cannot stat is not one holding no example: "ok release coherence (development: 0.2.0)"
  ```

  The three-arm `metadata` match the manifest already uses now stands here too: `NotFound` is the absence the
  loop may skip, anything else refuses and names the entry, and a `Cargo.toml`-less file such as a README
  still passes over.

  **The fixture's first spelling measured the wrong thing.** A mode-stripped `examples/` makes *every* entry
  unstatable, so the negative run showed the floor firing — the case that was already closed — rather than
  the silent skip. A symlink loop fails `stat` for **one** entry while its siblings stay readable, which is
  the shape the finding is about. An absent target answers `NotFound` and is the absence the loop may
  legitimately skip, so the loop rather than a dangling link is what makes the fixture perturb anything.

  Found by opening the last unopened reader, not by re-running a sweep — five review passes had gone by
  without `require_example_pins` being read.


- **A copy inherits nothing, so it holds whatever was carried across by hand — and two of three were.** The
  enumeration owner states three properties a caller of `ls-files` must not decide for itself: `-z`, a strict
  decode, and the hermetic builder, *because a verdict must not move with configuration outside the
  repository being judged*. Two sites cannot reach that owner — `shengmo`'s test targets, since `kanhe`
  depends on `shengmo` and the edge would close a cycle — and when they were converged the first two
  properties were transcribed and **the third was not**, in both copies, unmentioned in either comment.
  `GIT_DIR`, `GIT_WORK_TREE` and `GIT_INDEX_FILE` take precedence over discovery from `current_dir`, so a set
  variable has those two checks enumerate a different repository and answer.

  **And the sweep that gave `ls-files` one owner converged the callers past a helper rather than converging
  the helper.** `capability_subjects`' private `git()` still ran a bare `Command::new("git")` with a lossy
  decode, serving five reads that decide a verdict — `rev-parse --abbrev-ref @{upstream}`, `for-each-ref`,
  `merge-base`, `rev-list --count`, and the `diff --name-only` that selects which capability subjects a
  change touches. There is no boundary reason for that one: it is `kanhe`, and the owner is one call away.
  `merge_workflow`'s fixture `git init` is the same shape on the other side, where `hermetic_git::fixture`
  exists for exactly it.

  Both are repaired, and the two boundary-forced copies now make every environment operation the builder
  makes, with each comment naming the property as **three** parts rather than two.

  **The class is closed rather than its instances.** `crates/kanhe/tests/hermetic_invocations.rs` holds every
  tracked file constructing its own `git` against a declared set — the two-directional comparison
  `gate_exit_classes` uses — and adds the column that shape was missing: whether the site *claims* the
  builder's environment properties. A site that claims them must make every operation the builder makes, and
  the required set is read from the builder's own text, so an operation it starts making is one a copy starts
  owing. Nothing had said what a copy owed, which is why nothing noticed two of three. Negative runs:

  ```
  a site declared to hold the builder's environment properties does not hold all of them:
    crates/shengmo/tests/family_coverage.rs: does not handle GIT_DIR
    crates/shengmo/tests/family_coverage.rs: does not handle GIT_WORK_TREE
    crates/shengmo/tests/family_coverage.rs: does not handle GIT_INDEX_FILE

  the files constructing a `git` differ from the set named here. 294 tracked Rust file(s) were read
    left:  {…hermetic_git.rs, …examples_suite.rs, …family_coverage.rs}
   right:  {…hermetic_git.rs, …census.rs, …examples_suite.rs, …family_coverage.rs}
  ```

  **The first spelling of that guard counted its own explanation.** Written against the whole file text, it
  passed with the three `env_remove` calls deleted, because the paragraph saying why they are there names
  every variable it removes. It reads code lines only now — the class `repeated_paragraph` met from the other
  side, in the round after that one closed it.

  The stop is declared: a `git` constructed as `Command::new(<value>)` is not read, because whether a value
  names `git` is not decidable from the line that constructs it. Measured, the two such sites in this tree
  are the builder itself and an `ssh-keygen` signature verifier — and `gate_exit_classes` already requires
  any target that spawns a process to be declared, so what this stop leaves unclassified is *which program*
  a spawn is, not that it happens.


- **Nineteen readers answered *which paths does git track*, and each decided three things for itself.**
  Every one is correct on a tree whose paths are all ASCII, which is what a latent class looks like from
  inside a green run — and the property was independently discovered and written down **three separate
  times** before it had an owner: `release_coherence_gate`'s walk, `projection_register`'s reader and
  `repeated_paragraph`'s enumeration each carry their own sentence about `core.quotePath`. Measured across
  the invocations rather than the files: **8 were line-oriented, 11 decoded lossily, 9 bypassed the hermetic
  builder.**

  **The shape is one edit away, not hypothetical.** Measured on a scratch repository holding `圭表.md`:

  ```
  git ls-files      →  "\345\234\255\350\241\250.md"   (opens nothing)
  git ls-files -z   →  圭表.md
  ```

  This repository's whole vocabulary is those characters and its crates are named for them, so a tracked
  file named that way is one commit, not a thought experiment.

  `kanhe::hermetic_git::tracked_paths` owns the question now, and every enumeration in the workspace asks
  it — nineteen call sites converged, and the `ls-files` spellings that remain are the ones that are not
  enumerations: membership probes (`--error-unmatch`), and directions whose subject **is** the runner. The
  owner's own direction holds the property against the alternative rather than asserting it alone: it builds
  a repository with a non-ASCII path and requires the line-oriented read to disagree.

  Two sites hold the property in place instead of sharing it, and say why: `shengmo`'s test targets cannot
  depend on `kanhe`, because `kanhe` depends on `shengmo` and the edge would close a cycle. A fact about the
  dependency graph rather than a site anyone declined to converge, which is the disposition this repository
  already gives `MARKER` and `DO_NOT_EDIT`.

  **No reaction holds this, and the requirement says so.** *Is this invocation an enumeration* is not
  decidable from the argument list — the same command answers membership and is the subject of directions
  about the runner itself — so a reader keyed on the spelling would refuse those. It is carried by review,
  like the rows `AGENTS.md` disposes the same way.


- **A comment-shaped line inside a string literal was read as a comment, and the workaround was the
  evidence.** The repeated-paragraph check decided *is this a comment* by reading the trimmed line, which
  cannot tell a comment from a line of a multi-line string — and this repository holds Rust fixtures as Rust
  strings everywhere. The class was met the day the check was written: its own six-line fixture *was* such a
  string, the live sweep reported this file at its own line 200, and that was got out of the way by
  assembling the fixture at run time. The defect stayed, and the workaround became a tax on whoever wrote
  the next fixture.

  **Comments are what a lexer discards**, so the classification is taken from one. `proc_macro2` was already
  a dev-dependency here — for the register that reads this repository's own Rust with a real parser instead
  of scanning it — so a literal's span says which lines are its text, and a comment-shaped line strictly
  inside one is not a comment. A file that does not lex is refused as a cannot-judge: a comment it cannot
  separate from a string is a question it did not decide. Measured: all 293 tracked Rust files lex.

  **The risk was entirely in the other direction, and it is pinned.** A doc comment reaches the lexer as a
  synthesised `#[doc = "…"]` whose literal spans the comment's own line, so shadowing literals naively would
  have stopped this check reading `///` paragraphs **at all** — most of this repository's prose, and a
  silent false negative rather than a tax. They are told apart by the only thing that distinguishes them: a
  literal whose own first line is comment-shaped is a doc comment. One direction holds both halves, because
  getting either wrong breaks the other. Negative run with the shadow removed:

  ```
  assertion `left == right` failed: a duplicated comment paragraph inside a string literal is that string's text
    left: [(4, 1)]
   right: []
  ```

  **The fixture is written out again**, which is the second direction: it is now a duplicated comment
  paragraph inside a string literal, in a file the live sweep reads — a live instance of the class rather
  than a description of one. Written first as a `fn`, it tripped a sibling: the refusal register asserts that
  its span reader loses no declaration, and its own comment states the convention embedded Rust here follows
  — behind a call, never opening a line. It is a `mod` now. One guard caught what another had just been
  taught to ignore, which is the arrangement working.

  Also in this change: the enumeration takes `run_exact` rather than `run`. That module assigns the two by
  what the caller does with the answer — `run` trims for a caller reading a value, `run_exact` keeps the
  bytes for one comparing content — and a NUL-separated path list is the second. The trim was harmless here,
  since `\0` is not ASCII whitespace, but taking the accessor whose stated criterion fits is what keeps the
  criterion true of its callers. And `repetitions`' own first line still promised a *byte-identical*
  comparison after the module, the specification and its fixtures had all moved to line content.


- **The exception added for a third party's pin was wider than the sanction it was added for, three ways.**
  Two independent reviews arrived at the same unit and each found what the other did not.

  **It never asked whose repository the pin names.** Read by shape alone, the sanction covered
  `<this repository's owner>/<its name>@<sha>` — GitHub's canonical cross-reference for a commit of this
  tree, which is precisely what the requirement exists to refuse. The hatch was recorded and accepted on the
  ground that *the alternative is a list of third parties somebody has to keep*; that reason does not
  survive, because excluding this repository needs no list — only the `repository` field the workspace
  manifest already declares. It is read from there now, and a direction refuses an exclusion that resolves
  to nothing, since that is indistinguishable in a green run from one that works.

  **A suffix made a revision expression wear a pin's prefix.** `owner/action@<sha>^{commit}` and
  `owner/action@<sha>..HEAD` each name a commit *reached from* the pin, which is a citation of a moment —
  the prohibited form itself. What may follow the object is an allowlist of sentence-closing punctuation
  now, in which a single `.` counts only where a second does not follow it.

  **And an empty path segment resolved for nobody.** `owner//action@<sha>` passed a reader that counted
  slashes; segments are read rather than counted now, each required non-empty.

  The control fixture holds the whole boundary at once — two sanctioned spans and six refused — and each
  narrowing has its own negative run, so what each buys is stated separately rather than as one figure:

  ```
  own-repository exclusion removed:  reported 9, 11, 13, 15, 17   — 7  escapes (this repository's own commit)
  terminator allowlist removed:      reported 7, 13, 15, 17       — 9, 11 escape (`^{commit}`, `..HEAD`)
  segment check removed:             reported 7, 9, 11, 15, 17    — 13 escapes (`owner//action`)
  ```

  The `useless_format` the tightened control first carried was caught by `cargo clippy --all-targets`, not
  by the suite: the assertion hard-coded the two sanctioned line numbers where its sibling loop derived
  them. It derives them now, from the same fixture layout.


- **The citation reader refused a third party's pin, which governance sanctions by name.** `AGENTS.md`'s
  commit-object row says it in a sentence of its own: *a third party's object is not this row — an action
  pinned as `owner/action@<sha>` is correct supply-chain practice*. The reader classified every
  delimiter-bounded hex run without first classifying the syntax containing it, so the sha after an `@` was
  read as a citation of a commit of this tree. Nothing had gone red because the workflow carrying the real
  pins is not prose and never reaches this sweep: the rule and its reaction disagreed with **no instance
  between them**, which is the state a control fixture exists to end.

  **The criterion governance states is not one a reaction can run.** That row says the same criterion
  excludes a third party's object *without a list — that sha does not resolve here*. A development commit of
  this tree does not resolve in a fresh clone either, which is the whole reason the rule exists, so a reader
  keyed on resolution would report **clean in CI over exactly the citations it is there to find**, and loud
  on the author's own machine. Resolution is the criterion a person applies; the shape is what a reader can.
  So the containing syntax is classified first: `<owner>/<name>[/<path>]@<forty lowercase hex>`.

  **The exception is bounded, and the control says by what.** The forty is required — a pin shortened is not
  the practice the rule sanctions — and the fixture holds three shapes at once: the pin passes, a bare object
  in the same document is still reported, and a shortened pin is reported too. Negative run with the
  exception removed, showing the pin reported twice as this repository's object:

  ```
  a third party's pinned sha is sanctioned by name and must not be reported: {
      "  GUIDE.md:3 cites the commit object `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09`, and live text anchors to a release. …",
      "  GUIDE.md:5 cites the commit object `fbc6f3992d24b796d5a048ff273f7fcc4a7b6c09`, and live text anchors to a release. …",
      "  GUIDE.md:7 cites the commit object `f41b3b9c`, …",
      "  GUIDE.md:9 cites the commit object `f41b3b9c`, …",
  }
  ```

  It is an escape hatch, and the requirement says which: a citation of this repository's own object spelled
  `foo/bar@<sha>` would pass. Every sanctioned form is an escape hatch that way, and the alternative is a
  list of third parties somebody has to keep.


- **The repeated-paragraph check's declarations did not match its reader, in three places.** All three are
  declaration defects rather than reader defects, which is what an external review found in the round after
  the check landed — the pattern this repository records about itself, that a new reaction carries a defect
  into the next round.

  **A stop the reader had and nothing declared.** The span floor sat at two lines, so a one-line comment
  paragraph written twice was not read, while the module doc claimed *no comment paragraph is written twice
  in a row* — and a one-line paragraph is a paragraph. The check's two other stops each carry a `BoundDecl`
  with a reason and a pinning direction; this third one was written nowhere, so the reader claimed more than
  it had. Measured over the whole tracked Rust corpus at both floors — **zero either way** — so the floor
  came down to one rather than being declared, and the content requirement is what keeps a repeated
  paragraph break out. Negative run with the floor restored:

  ```
  assertion `left == right` failed: a single comment line repeated is the shortest paste there is
    left: []
   right: [(3, 1)]
  ```

  **A precision the requirement claimed and the reader does not have.** The requirement said *byte-identical
  copy*; `str::lines` drops `\r\n` and `\n` alike, so the comparison is over line content and a block
  terminated one way is one repetition with a copy terminated the other. Reacting there is right — a paste
  an editor re-terminated is still a paste, and requiring the terminators to agree would let it through,
  which is a silent false negative — so the requirement now says *line content, not line bytes* and a
  direction supplies the shape, since every tracked file is `i/lf` and there is no `.gitattributes` for the
  corpus to acquire it from. Negative run with the terminators required to agree:

  ```
  assertion `left == right` failed: the terminators differ and the paragraph is the same, which is the paste this reads
    left: []
   right: [(3, 2)]
  ```

  **A pin whose name stated the other of the two facts its direction held.** The out-of-line bound cited
  `identical_code_lines_are_not_read`, which does hold the fact — in its second assertion — but a reader
  following the citation landed on a name about repeated *code*. Split, so the bound cites
  `a_repetition_split_by_code_is_not_read` and each name states its own subject. An identifier is a carrier
  of a claim, which is this repository's own rule about names, applied here to the change that wrote it.


- **A new reader spelled its own `git` call and renamed the paths it enumerated.** The repeated-paragraph
  check ran `git ls-files -z` through its own `Command` and took the output through `from_utf8_lossy` — the
  exact decode `hermetic_git::run` exists to refuse, in a header that names **this command** as its reason:
  `-z` promises nothing about encoding, so a tracked path carrying an undecodable byte arrives as a
  different path than the one on disk. Every read below it is then made against that name, and the resulting
  *tracked file could not be read* is honest about the wrong file. `xingbiao::path_identity` exists for the
  opposite property and `repository_path` refuses the same thing in the same words; this reader, written
  after both, reached past them.

  Routed through `hermetic_git::run`, which refuses. A direction builds a repository holding
  `probe-\xff.rs`, tracks it, and requires the enumeration to stop. Negative run with the lossy decode
  restored verbatim:

  ```
  a path this reader cannot represent must stop the enumeration, not be decoded into another name: ["plain.rs", "probe-\u{fffd}.rs"]
  ```

  **The first spelling of that direction was itself the defect it was written for.** Held over
  `hermetic_git::run` rather than over the enumeration, it passed with the lossy decode restored — because
  what had gone wrong was the caller reaching past that runner, which a direction observing the runner
  cannot see. It is held over the enumeration now, and the negative run above is what that change bought.


- **A direction made the same assertion twice, and one of them was the whole direction's second half.**
  `a_tag_with_no_signature_block_is_named_as_such` called `refusal::expect` on the same identifier and the
  same refusal twice in a row, byte-identical. It shipped in `0.5.0`. Nothing was wrong with the direction's
  verdict — the second call asks exactly what the first asked — but a reader counting what a direction holds
  counts two facts where there is one, which is the same currency as a paragraph written twice.

  Found by the measurement taken while bounding the repeated-paragraph reader above: a rule over identical
  adjacent **lines** rather than identical adjacent **comment** lines reports three sites, two of them
  deliberate and this one not. Reported here rather than reacted to, because the reaction that would hold it
  is the one that measurement ruled out.


- **Two readers of one position each held their own copy of how to find it.** In 圭表's declaration scanner,
  `attr_prefix_path_kind` looks for `path` at an attribute's name position and `attr_prefix_has_bare_cfg`
  looks for `cfg` at the same one. Written out per site, the walk to that position stood twice —
  twenty-three byte-identical lines, diverging only at the terminal word.

  **It had already been repaired twice by hand**, which is the finding rather than the duplication. The
  raw-identifier skip (`#[r#path = "…"]` names the built-in `path`) was added to each copy separately; the
  hand that added it to both is the hand that would have to add the next one, and the failure mode of that
  arrangement is not a compile error — it is the `cfg` scanner learning a spelling the `path` scanner does
  not, and the two then disagreeing about the same source.

  `attr_name_start` now owns the answer, with the measurement and the `##[` resumption rule stated once
  where the answer is computed. Behaviour-preserving: the extraction changes only *where* the walk is
  written, and 74 suites pass unchanged. A direction holds both readers against the shared position over
  nine spellings — whitespace on either side of the bracket, the raw spelling, `r#cfg_attr` against bare
  `cfg`, a lone `r#`, and a `#` that opens nothing. Negative run, with the raw-identifier skip deleted from
  the one owner:

  ```
  assertion `left == right` failed: the name position in #[r#path = "x.rs"]
    left: Some(2)
   right: Some(4)
  ```


- **One env-gated direction returned green without running and said nothing.**
  `every_example_passes_its_isolated_quality_gates` skipped when `TIANHENG_EXAMPLES` was unset and printed
  nothing, so a local run reported `ok` for a direction that had not looked at anything. The reader who most
  needs to know is exactly the one who set no variable and read the `ok`. It now announces, in the words its
  own sibling four screens above already used.

  **The census, since a fix to one site is worth nothing if the class is wider.** Four directions in this
  repository skip on an environment variable, and the other three already say so: `pin_bites` and
  `every_example_reacts_as_declared` through `eprintln!`, the spelling differential through `println!`, and
  `publish_source`'s pre-flight in the strongest form of all — it returns a typed `Verdict::NotAsked`
  carrying the reason, so the skip is a value the caller must handle rather than a line someone might read.

  **No reaction is proposed, and the census is why.** Announcing has three correct forms here and one of
  them is a typed return; a detector would have to recognise all three, which is the shape
  `gate_exit_classes`' own header records being one form short of, three rounds running. Against four sites
  that is more machinery than subject. The honest guard is the census above, re-run by reading.


- **A comment paragraph stood twice, byte-identical, and every tool in the chain read it as deliberate.**
  In `release_coherence_gate`, the six lines naming why presence is asked by `ls-tree` rather than `show`
  were pasted twice. It compiled, formatted, linted and passed every gate this repository runs, because a
  repeated paragraph is well-formed text — and it was invisible to the person who made it for the same
  reason it was made: the eye that skips a paragraph it has already read is the eye that pasted it. Found by
  review.

  It matters more here than a missing final newline, which this repository does refuse, because of what the
  prose is **for**. The rules are carried by weight rather than by enforcement — what sits in an agent's
  context is what gets imitated — so a paragraph standing twice is that weight doubled by accident, in a
  file whose whole purpose is to be read.

  **A reaction now holds the class**, `crates/kanhe/tests/repeated_paragraph.rs`: no tracked Rust file
  carries two or more comment lines immediately followed by a byte-identical copy of them. Negative run over
  the tree before the repair, verbatim:

  ```
  293 tracked Rust file(s) inspected; a comment paragraph is written twice, or a tracked file could not be read:
    Violation: crates/kanhe/src/release_coherence_gate.rs:533: a 6-line comment paragraph is written twice in a row; the second copy begins here
  ```

  **Two stops are declared rather than assumed.** The rule reads *adjacent* repetition only, and only in
  Rust. Measured over the tracked corpus at three states — `v0.5.0`, the tree carrying the defect, and the
  repaired tree — a rule keyed on identical adjacent **lines** rather than identical adjacent **comment**
  lines reports three more sites, and each is deliberate: a test passing `--manifest-path` twice to assert
  the duplicate flag exits `2`, a function type spelling the same parameters twice, and a duplicated
  assertion. That is the authoring tax `PROJECT.md` refuses, so the corpus is comments; and Markdown repeats
  identical adjacent lines for its own reasons, so prose — the corpus this check most exists to protect — is
  outside it until a shape with no false positive is found. Both are pinned bounds, projected into
  `docs/observation-bounds.md`.

  The check reads itself, which is where its first finding came from: written out, the fixture holding the
  repeated paragraph *was* a repeated paragraph in a tracked file, and the live sweep reported this file at
  its own line 200. The fixture is assembled at run time instead — one paragraph repeated by the direction,
  leaving no two identical adjacent lines in the source.


- **The Definition of Done said staging is enough, and it is not for a gate whose corpus is `HEAD`.**
  `AGENTS.md` told an operator to `git add` a created file and then, in the same paragraph, that *committing
  is not required*. That is true of a gate taking its path list from `git ls-files` and its content from
  disk, which is how the paragraph's own measurement was taken — and false of `pin_bites`, which reads both
  halves of its subject through `git show HEAD:…`: the declared-mutation records, and the source each
  mutation perturbs. An uncommitted record is no record, so the direction reports green over an **empty
  set**.

  Measured while landing the raw-identifier repair: two mutations added and staged, `pin_bites` green
  locally, and both CI's Definition of Done and the MSRV job red on the same tree the moment it was
  committed — `all_three_dimensions_read_a_raw_identifier_spelling_of_a_cfg_attr_path` *is cited but
  resolves to no bound id*. The local pass was not merely weaker than CI's; it was a pass over nothing.

  The paragraph now states the criterion — the read, not the gate — and names the search that finds them:
  `git grep -nE '"HEAD:|ls-tree", "HEAD' -- crates/kanhe`. It returns one other pair of sites, and that one
  is not this case: `release_coherence` compares HEAD's `CHANGELOG.md` against the worktree's, so the
  difference is its subject rather than a blind spot. There is no reaction, for the reason the surrounding
  paragraph already gives: in CI nothing is ever uncommitted, so a check would be vacuous exactly where it
  runs.

- **The branch written to tolerate a member sitting at the workspace root could not carry one.** A root
  declaring `[workspace]` and `[package]` together is a shape cargo accepts — measured on cargo 1.96.0, it
  reports that member's `manifest_path` as the root's own `Cargo.toml` — so the member's directory strips to
  nothing. The empty string was then handed to `git ls-files` as a pathspec, and git refuses one:
  `fatal: empty string is not a valid pathspec. please use . instead if you meant to match all paths`. The
  branch reached the directory-unreadable refusal with an empty subject, saying `could not enumerate : `.

  `.` is what git suggests and is not what this check means. The machinery set is *the tracked files under a
  member the workspace does not publish*, and for a member that is the root, `.` is every tracked file in the
  repository — every other member's source included. So the shape is refused rather than tolerated, in the
  words `manifest`'s own reader already uses of a single-crate root: not a shape this check judges, and
  saying so beats guessing. `machinery_names` is `pub(crate)` for the direction that meets it, which is
  narrower than the `pub` its neighbour already carries for the same reason.

- **A path that is not UTF-8 kept its own identity at one end of a comparison and not at the other.**
  `hermetic_git` refuses git output it cannot decode, in so many words — *a path that is not UTF-8 keeps its
  own identity, and reporting a replaced one would compare something the repository does not hold*. The
  reader that spells this repository's own side of that comparison decoded lossily, so a component the
  operating system holds as bytes came back as U+FFFD per byte: a name resolving to nothing, and two
  distinct names collapsing onto one spelling.

  **One of its three call sites can reach it, and stating which is half the repair.** `machinery_names` and
  the member-enumeration comparison are handed `manifest_path` as a `&str` out of cargo's JSON, so the
  parser has already made every component UTF-8 and no arm of the decode can fire for them — an earlier
  reading of this defect claimed a silent false negative in the machinery set on exactly that path, and it
  was wrong. `workspace_manifests` takes its paths from `read_dir`, where the bytes are the operating
  system's, and a crate directory whose name is not UTF-8 is legal on Unix.

  `RepositoryPath` gains a third state, so *not under the root* and *not spellable at all* stay apart —
  two facts an operator repairs in opposite directions — and `machinery_names`' `let … else`, which had
  folded the new state into the first of them, is a `match`. The walk's arm is held by
  `a_crate_directory_that_is_not_utf8_is_refused_by_the_walk` against a real fixture directory; the
  JSON-fed arm is declared unheld, naming the direction that observes the same shape where a walk feeds it.

- **A path below the root had three spellings, and the two that had to agree did not.** Turning an absolute
  path cargo reports into the repository-relative identity git uses was written out at three sites.
  `machinery_names` and the member-enumeration comparison both stripped the root component-wise and joined
  the result with `/`; `workspace_manifests` stripped it with `Path::strip_prefix` too but spelled the answer
  with `Path::display`, the host's own separator — and on a failed strip carried the **absolute** path
  forward through `unwrap_or`, as though it were relative.

  The second of those is the side the comparison reads. Both sets are compared as strings, so wherever the
  host's separator is not `/` they share no member at all and the direction reports every member as both
  unwalked and undeclared. It is latent — this repository's CI is `ubuntu-latest` throughout — and its own
  comment claimed the opposite, saying it compared *through components, as the gate this direction stands
  beside does*. That sentence was true of one side.

  `kanhe::repository_path` is now the one owner: the component-wise strip, the `/` join, and a `RepositoryPath`
  whose `Outside` variant every consumer has to answer. The absolute-path fallback is gone, and where the
  answer cannot be reached by construction — the walk joins its manifests onto the very root it strips — the
  site is registered and declared unheld beside the sibling that already records the same shape for the
  answer cargo gives.

  Its failure matrix carries the two arms a host with a `/` separator can still observe: a path outside the
  root, whose negative run returns `Below("/elsewhere/kanhe/Cargo.toml")` — an absolute path presented as one
  below the root — and a root that is a text prefix of a sibling (`/r/crates` against `/r/crates-extra`),
  whose negative run returns `Below("-extra/kanhe/Cargo.toml")`. The separator half is unobservable here and
  is said so rather than counted as covered.

- **An unreleased version has no admissible form, and the exception was the defect.** The rule below
  repaired 24 sites by keeping each dead number and qualifying it — *the same sentence says what it became* —
  and exempted the sites that **declare** the class. The steward rejected both halves on 潛移: what sits in
  an agent's context is what gets imitated, and a number carrying a clause explaining that it never shipped
  is still that number in the context, with the clause read by a human and skipped by the continuation. A
  rule enforced by reverse prose is enforced on the wrong reader.

  So the exception is gone. `AGENTS.md`'s carrier taxonomy row states the rule with **no instance**, which
  the class's own shape allows — a version literal below the workspace version with neither a dated section
  nor a tag is decidable without an example — and a check built for it would need no allowlist, which is the
  cheaper design as well as the honest one. Where a mechanism was carried only by such a number, it is
  restated as a **property**: a measurement addressed at a release branch *by name* rather than the command
  that ran it, and a row that reaches `main` *when the next release is cut* rather than when a particular
  number is.

  Every live document now names the version each window shipped as. Measured after the sweep,
  `git grep` over tracked `*.md`, `*.rs` and `*.toml` outside `CHANGELOG.md` and `docs/history/` returns
  **nothing** for the three. What is deliberately left is `CHANGELOG.md`'s own **released** sections: a
  dated section is a measurement of its moment and an adopter holds that text, so rewriting one would
  falsify a record rather than clean a context. `AGENTS.md`'s row now says so at the point of use.

- **A version this repository never released was written into its own governance, three times.** The sweep
  had been treating this as 24 stale references to one window. It is one class, and a different fault
  from the relative anchor beside it in `AGENTS.md`'s table: `this window` names a **moving** reference,
  while an unreleased number names one that **never came to rest**. Release class is decided from what a
  window's changes do, so a window's number is not knowable until its cut — and a number written into prose
  beforehand becomes a pointer to nothing when the class moves.

  **It has moved three times.** Measured over every `X.Y.Z` literal in tracked live Markdown outside
  `CHANGELOG.md` and `docs/history/`: three distinct numbers appear and none has a dated section
  or a tag. The trend runs the wrong way — 15 occurrences, then 4, then 30 — and the reason it will continue
  is that **reclassifying upward is this repository's SemVer honesty working correctly**, so windows will
  keep being renumbered while their prose is written before the number is earned.

  **The repair first proposed was the wrong one, and the steward said so.** It kept the number and qualified
  it — every site naming what the window became in the same clause — on the reasoning that a site declaring
  the class is exempt. 潛移 refutes that: what sits in an agent's context is what gets imitated, and a dead
  number carrying a clause explaining that it is dead is still that number in the context, with the clause
  read by a human and skipped by the continuation. So there is **no exempt site and no qualified form**.
  Every live document names the version each window shipped as, and `git grep` for the three finds nothing
  outside `docs/history/` and this file's own **released** sections, which are records of their moment and
  stand. `AGENTS.md`'s carrier taxonomy row states the rule without an instance, which the class's own shape
  allows: a version literal below the workspace version with neither a dated section nor a tag needs no
  example to be decidable.

  **The last holdout was a measurement, and keeping the name there was wrong for a reason better than
  tidiness.** That figure was addressed at the release branch **by name**, defended on the ground that a
  query's argument is part of its observation. It is — but the observation was the wrong one: a branch is renameable
  and a version that never shipped resolves through nothing, so the corpus was fragile from the start.
  Re-addressed to the window's commit range, which resolves from any clone, the same question answers **103
  pull requests and three offences** where the branch-addressed corpus answered 22 and one. The two it had
  never seen are `fix/tianheng-…` landing as `docs(tianheng): …` and `gov/spec-prose-discipline`, whose type
  is not in the admitted set — the type disagreement the naming rule exists to prevent, and a second retired
  role, which is a clause of that entry's trigger never before seen to fire. **The dead name was not only
  unresolvable; it was holding a figure three times too small.**

  **A reaction is reachable here, which is unusual for this file's prose classes and is why it is filed
  rather than waved at.** A version literal below the workspace version with neither a dated section nor a
  tag is decidable with no judgement over meaning: above the workspace version is a plan, with a section or
  tag is a release, and what is left is neither. The exemption — the same paragraph naming an earned version
  — is positional. What is not taken is the design decision inside that exemption, which is whether a
  governance file may narrate its own reclassification, because choosing it alone is the shape this sweep
  kept finding wrong.

- **The reading finished, and the last thing it found was a typed list inside the paragraph reasoning from
  it.** The workflow-pin entry bounds its own risk by naming which actions run and under what permission —
  *the actions are `actions/checkout` and `EmbarkStudios/cargo-deny-action`*, two, while the workflow pins
  **three**. `actions/setup-node` arrived with the Node interpreter pin **this same entry narrates**, and the
  enumeration its disposition rests on did not move with it. What actually bounds the risk is the permission
  grant, which is a property of the workflow rather than of which actions it happens to name, so the entry
  states that criterion and the count is left to whatever reads `uses:`. *A census is produced, never typed*
  — inside an `ACCEPTED DEBT` whose acceptance was argued from the typed half.

  Three negatives close the pass, each recording what it checked. *The scanner count is three, not four* —
  measured by which files `starts_with` or `strip_prefix` the markers rather than which mention them, a
  distinction worth the care because counting mentions answers seven. *No scratch-directory helper reached
  the wildcard prelude*, so the fixture that would migrate on it still cannot. And *`gh` 2.46.0 offers no
  new server-decided precondition* — `--match-head-commit` pins the head object, which the wrapper already
  uses and which is not one of the three names its re-read races are about.

- **The first reading under the new rule, and a `WATCH` moved because its proposal turned out to be already
  running.** Four more entries evaluated, each recording what it checked rather than only what it concluded.

  *A Shape that had been run without anyone noticing.* The entry watching `shengmo::workspace::MARKER`'s
  out-of-reach literal copies proposes a comparison rather than a convergence — `kanhe` sees both the
  constant and the text of the crates that cannot depend on it. Swept for its trigger, no mistype exists;
  what does exist is a **second** constant of the same shape, `kanhe::verdict_channel::ENV`, whose
  out-of-reach copies are the two shell wrappers — and whose copies are already **held**, by
  `gate_exit_classes` asserting each wrapper's text against the constant. The trigger asked for a second
  *unheld* constant and found a second *held* one, which answers a different and better question: the
  proposal is not a hypothesis. The entry moves to `READY-PATCH` on this file's own definitions, `WATCH`
  being for pressure without enough correctness evidence.

  *And one trigger cannot fire yet, which is now something a verdict may say.* The prose-claim entry's
  condition is a window **after** a `READY-PATCH` that has not landed, so no window has begun under the state
  it measures from. It is recorded as **unevaluable** rather than *not fired* — the distinction the rule
  adopted in this window requires, since a trigger whose precondition is unmet has neither been observed nor
  passed.

  Two clean negatives with their measurements: every tracked fixture corpus is still read by something
  outside `fixtures/`, eight of eight; and the `bash <path>` sweep finds five occurrences, one live command
  tracked executable and four that are a declaration, a fixture string, a constructed shell example and this
  entry's own recorded instance.

  *A guard caught the reading, for the class the reading was about.* The first draft of that last sentence
  spelled the two untracked names out, and `reference_integrity` refused it — a reference to a path this
  repository does not have. A sweep confirming that commands name targets which exist, writing a path that
  does not. The repair is the one this window keeps arriving at: describe the thing, do not reproduce the
  token that resolves to nothing.

- **`BACKLOG.md`'s promotion triggers are read against the window before the cut, and that is a step someone
  performs rather than a thing built.** A live entry names what would promote it and nothing evaluated that,
  which this file already recorded from the inside: an entry whose trigger had fired on **both** halves sat
  unchanged through a full round of review *because nothing evaluates a promotion trigger*. Read on purpose,
  fifteen entries produced eight defects sitting behind triggers already written — including one entry whose
  figure was three times too small and one document refuted by its own next decision.

  **The reachable half is the occasion, not an instrument.** Deciding whether a trigger's set is its
  residue's set means knowing what the entry meant, which is the judgement-over-text this repository has
  measured and rejected three times. So `AGENTS.md`'s release ritual now carries the reading, next to the
  rule that the dated section's date is the last edit before the cut — a discipline nobody is asked for
  being one nobody does.

  **Two rules make the hour worth spending, and both constrain the verdict rather than the entry.** An
  evaluation records what it **checked**, because a bare *Not fired* cannot be told from nobody having
  looked. And a verdict no sweep could have reached **says so** instead of carrying a date: measured, an
  entry read `Not fired, swept` over an observable its own lifecycle destroys before the squash, and a dated
  verdict reads as *settled* where the honest word is *unobserved*.

  Per-entry labels sorting triggers into swept and witnessed were the other candidate and are **declined**,
  on the objection the entry proposing them had already recorded against itself: a field on sixty entries is
  a form to maintain. Two sentences reach the same failure at the point where it happens and cost nothing to
  keep.

- **The sweep's third pass, and the sweep wrote an instance of the class it was measuring.** Four more
  entries evaluated. The one that matters is the relative-anchor bound, whose corpus command was re-run: it
  answers three live offences against ten at the previous reading, and **one of the three had been written
  minutes earlier by the sweep's own repair** — inside the paragraph whose subject is a figure attached to
  the wrong corpus. All three are anchored to `0.6.0` now. The count falling is not the finding; *the class
  is produced by the hand repairing it* was already recorded once and has now held for a second window
  running, at an interval of minutes rather than days. Nothing observed any of it, which is what the declared
  bound says will happen.

  The other three are negatives with their measurements, recorded because a trigger nobody re-reads is a
  stale answer whichever way it points. *Every commit on the release branch came through a pull request* —
  101 of 101, asked of the API per commit rather than matched against a listing, which is the direction that
  cannot miss one a listing's limit truncated. *The two irreversible-act wrappers have not diverged* —
  `merge-pr.sh` grew 58 lines while `publish.sh` was untouched, which is the shape a divergence would arrive
  in, and every named shared construct is still present in both with the structural ones matching in count;
  the growth is the pull-request re-reads, which the publish side has no counterpart for. *And no stolen doc
  changed what a reader did* — but that entry had moved its trigger off a count on the ground that it was
  *a count nothing produces*, and this window produced two more instances of the class, both found by review
  and repaired where they were made. The count is produced; what nothing produces is a reaction that
  produces it, and the entry now says the second rather than the first.

- **The sweep continued over the rest of the live entries, and `PROJECT.md` was refuted by its own next
  decision.** Three more findings, each a claim rather than a reaction — nothing behaved differently, which is
  why nothing caught them.

  *A trigger fired on a document contradicting itself.* `PROJECT.md`'s `xingbiao` description read *the
  static and semantic dimensions read the workspace through one source of truth* — two dimensions, where
  all three carry it. **The falsifier was on the same page**: `PROJECT.md`'s own runtime-identity decision
  records the cargo feature 漏刻 reaches it through. The first repair proved the point by restating the
  membership from the manifests, and `law_restatement` refused it — the dependency rules are declared and
  `AGENTS.self-law.md` renders them, so the entry cites the projection and keeps only the half no projection
  carries, which is that one of the three arrives through a feature rather than unconditionally. A guard
  caught a repair for the defect the repair was about. It was found by re-reading the file after
  `xingbiao`'s module doc was corrected for the identical omission earlier in this window — one carrier
  repaired and this one not swept, which is the retirement-sweep class whose corpus is *every tracked live
  file*. The parenthesised list of three exports went with it: enumerating three functions of a substrate
  that now holds path identity and the filesystem-answer policy is the same defect one level down.

  *A governance figure was recorded over an unfinished window.* The ratio entry's numbers were taken at 42 of
  this window's 100 landed changes. Re-derived with the command the entry carries: the classifier as written
  answers `34/100`, by carrier `100/100`, and **`21/100` touched a published crate's `src/`** — above
  `0.5.0`'s 18%, where the reading at 42 had none, because the window's second half is where the lexical
  false-negative closures landed and those are product. The entry's own observation source says the figure is
  run *after the release was cut*; a mid-window reading in the same sentence shape is a figure attached to an
  incomplete corpus, and nothing disagreed with it because nobody re-ran it. Both readings are kept, each
  stating its count.

  *And the trigger-corpus entry fired on both halves at once.* It watches for *a trigger found to have fired
  unnoticed for a whole window* — the positional-reference trigger fired four times across this window and
  surfaced only when the triggers were read on purpose — and for *a fifth instance* of a trigger written over
  a set narrower than the residue it guards, which the swept-versus-witnessed entry supplied. Its four
  original instances were each found by a reader who happened to hold both halves; these two were found by
  asking every trigger the question at once, so **what is reachable is the occasion rather than an
  instrument**. The re-decision is the steward's.

- **A promotion-trigger sweep of the entries this window touched, and the sweep this window already ran had
  moved a measurement off its own corpus.** `BACKLOG.md` asks every live entry for a promotion trigger, and a
  trigger nobody re-reads after the work that would fire it is a decision surface stating a stale answer. Eight
  entries were evaluated against the 99 landed changes — chosen by whether this window plausibly touched them,
  which is stated because it is **not** the whole live set.

  *A rename rewrote a base inside a measurement.* The branch-name entry's evidence was a count of merged pull
  requests addressed by the branch they landed on, and it named one instance —
  `test-kanhe-a-fixture-sha-that-resolves-nowhere` — which that branch's merge records do not contain. GitHub retargets a
  renamed branch's **open** pull requests and leaves a **merged** one's recorded base alone, so a branch this
  window renamed left the figure attached to a set that does not contain its own evidence. The rename's own
  sweep did it, one window after the commit that wrote the figure — whose subject is *a trigger is a
  predicate, and nothing holds one to its corpus*, and whose corpus was a branch. The figure is re-addressed
  to the window's commit range and carries the command that produces it; *A version this repository never
  released was written into its own governance* records what that re-addressing then found.

  *A trigger fired.* *A positional reference appearing again after this sweep* — whose own control was the
  0.5.0 sweep — fired four times in this section. Read out, two resolve, one is ambiguous, and
  `see the entry below` pointed at the wrong entry: the one it means was written later and, entries being
  newest-first, landed **above** it. That is the entry's recorded mechanism, unobserved because a group
  merge's verification compares the multiset of lines and is blind to whether an entry still points at what
  it meant. Both are named rather than positioned now.

  *And a verdict was written in the wrong register.* The OpenSpec-lifecycle entry carried
  `Not fired, swept 2026-09-01` on the ground that `openspec/changes/**` is untouched — but step 5 of that
  lifecycle is *strip the change directory before the squash*, and the squash collapses the branch that would
  hold the propose commit. **Both observables are erased by the ritual they were read as observing**, so the
  emptiness is what either state looks like and *whether the event has happened is decidable* was false. It
  is a witness-only trigger, and a dated verdict made it read as settled rather than unobserved. The entry
  that sorts triggers into swept and witnessed named only the direction where a witness-only one turns out
  cheap; it is widened to the direction where a swept one turns out impossible, which is the more dangerous
  half, and it fires on that instance. A malformed heading fragment splitting an entry from its own closing
  note goes with them — it begins with a space, so the classification check that reads `### ` never saw it
  while a reader meets it as a section break.

  No reaction is proposed for any of this. What would decide whether a version name in a sentence is a label
  or an observation is a judgement about meaning, which is the prose detector this file records as designed,
  measured and rejected three times.

- **The head branch is re-read too, and the premise that excused it was false.** It was left out of the
  post-gate re-reads on the ground that GitHub offers no way to change an existing pull request's head, so a
  guard could only ever refuse against a fixture — which is not a guard. An independent review refuted it:
  GitHub cannot **repoint** an open pull request, but renaming a branch retargets the pull requests on it, so
  `headRefName` moves while `headRefOid` does not and `--match-head-commit` pins the object without observing
  the name. A `release/X.Y.Z` -> `main` squash approved with an empty body, whose head branch is then renamed,
  lands that body from a branch that is no longer a release branch. The exception can only be **lost** this
  way and never gained, so it is not a false negative — but an omission defended by a false premise is worse
  than the omission, and the negative run was constructible all along.

- **Claims this window made about its own work, corrected where the code refuses them.** An independent
  adversarial review of the three changes above found each of these, and each is a claim rather than a
  behaviour — nothing reacted differently, which is why nothing caught them.

  *The load-bearing measurement was mis-attributed.* Five sites and a commit body said cargo **refuses** a
  `package` written beside `workspace = true`. It does not: measured, cargo accepts it, warns
  `unused manifest key: dependencies.alias.package`, ignores the local value and resolves the catalog's crate
  anyway. The probe that produced the original claim had moved two variables at once — it renamed the catalog
  key *and* added the `package`, so the refusal it observed was the key mismatch. The conclusion is unchanged
  and the correct measurement argues for it more strongly, but a specification clause is *reproducible now or
  not at all*, and re-running that one refuted it.

  *A sweep claimed a corpus it had not swept*, and *two test doc comments stated a false reason for their own
  fixture* — both tests pass with the second example deleted, because the per-example counter is incremented
  before the pin is compared, so the vacuity guard they named is unreachable. One of the two was new, having
  copied the reason from its sibling rather than checking it.

  *A negative run's record was composed rather than pasted*, losing the `left`/`right` pair that is the
  evidence, while its sibling in the same window was verbatim. *A five-line paragraph was left duplicated*
  inside one function, the first copy describing a statement the same change had deleted. And *`offered`'s
  doc comment said `Offered::Missing` is a refusal rather than a fallback*, which held until the commit
  carrying that sentence made it one.

- **The base the squash lands on is re-read after the gate, as the title already was.** The wrapper's judged
  inputs divide by one question — what the merge **records** travels as the value the gate saw, what the merge
  is **judged against** has to still hold when the merge happens — and the base had been filed on the wrong
  side of it, exactly as the title once was. `gh pr merge` takes no base of its own and lands wherever the
  pull request points at merge time, so a base edited during the gate left an approved empty-body release
  message landing on a destination nothing judged, and carried the one message exception to a squash that is
  not one. The negative run reaches `pr merge` and exits `0`.

  The **head branch** is deliberately not re-read: GitHub offers no way to change an existing pull request's
  head and `--match-head-commit` already pins the head object, so a guard for it could be made to refuse only
  against the fixture and never against the tool. The declared bound is **widened rather than duplicated** —
  the stop is a property of a client-side re-read not being atomic with the act it precedes, reached through
  whichever inputs are re-read, and one stop declared twice is two records that must then agree. Its backlog
  entry's own trigger named this arrival: *another judged input that can only be re-read would make this a
  shape rather than an instance*.

  One sentence said the wrapper *judges three inputs* — written when it judged three, and left standing when
  the base and head branch were added to the gate in this same window. Replaced by the criterion, which does
  not go stale as the set grows.

  **The first repair of it claimed a corpus it had not swept**, saying the wording stood in four places and
  naming them. An independent review then found the same claim alive in nine more, two of them a `///` saying
  *four judged inputs* directly above a function that supplies six, and one of them in the wrapper itself
  sixty-six lines below the block that repair had rewritten to remove exactly this count. Every live site is
  now converted — the corpus is tracked `.rs`, `.md` and `.sh`, excluding `CHANGELOG.md` and `docs/`, the
  record carrier and the generated projections. The surviving hits there are dated entries and this
  paragraph's own italicised quotation of the retired wording, which is what a finished conversion looks
  like rather than an unfinished one.

- **A family crate the workspace catalog renames is judged, where the local key alone decided membership.**
  An inherited dependency's key is a **lookup key** into the catalog, never a name: measured under cargo
  1.96.0, `alias = { package = "realdep", version = "0.0.1" }` beside `alias = { workspace = true }` resolves
  to `realdep` at `^0.0.1`; a `package` written beside `workspace = true` is accepted and **ignored** with an
  `unused manifest key` warning, and inheriting under the crate's name rather than the catalog's key is
  refused outright — so neither shape makes the local key an identity. Asking
  the identity question first passed the entry over, and the example was then reported as declaring no family
  requirement — a different fact about a different manifest, while the stale requirement went unread. Where a
  sibling example keeps the per-example counter non-zero, that is a false negative in front of the release.

  The catalog search moved with it. Matching entries by resolved identity refused on **any** unreadable entry
  in the table, so a catalog carrying both an unreadable entry and a stale family pin answered cannot-judge
  about the entry nothing took while the pin the example did take went unread behind it — the same false
  negative reached through a refusal rather than a pass. One fixture had been passing on that over-breadth
  rather than on its own subject: it wrote the unreadable entry under a key nothing inherited.

- **git answering in bytes no `String` holds is refused, never replaced.** The shared git runner decoded
  stdout with `from_utf8_lossy`, and what it mostly carries is **paths**: `ls-files -z` avoids git's own
  quoting and promises nothing about encoding, so a tracked path that is not UTF-8 arrived as a different path
  than the one on disk and every comparison downstream was made against that. The reaction model carries a
  path-identity rule for the opposite property — two paths differing only in undecodable bytes keep two
  identities — so a reader here that collapses them contradicts it.

  The runner now separates *git failed* from *git answered and this reader cannot represent the answer*, and
  every caller answers the new state rather than inheriting one: the compiler named all five. stderr stays
  lossy, deliberately — it is a sentence for an operator, not a value anything compares. The direction builds
  a filename of one invalid byte in a real repository; its negative run returns the replacement character
  where the repository's byte was.

- **An example manifest that is there and is not a regular file is no longer read as absent.** `is_file()`
  answered both with one `false`, so a directory named `Cargo.toml` — or any path that exists and is not a
  regular file — read as *this example declares none*, and the remaining readable examples satisfied the
  counters that follow. Asking for the metadata separates them, in one construction whose message carries
  which it met. The subject is now stated too: an example is a **directory**, and `examples/` holds files of
  its own, so an entry that is not a directory holds no example rather than an unreadable one.

- **A path is compared as written before any punctuation comes off it.** The machinery-name reader stripped
  **every** trailing dot from a token, so a path legitimately ending in one was rewritten before it could
  match — an identity normalised to suit a sentence. A Markdown sentence ends in one period, so one is what
  comes off, and only where the name as written matches nothing.

- **A release commit whose own tree carries no changelog is refused, where it read as the next cycle.**
  `git show HEAD:<path>` exits the same status for a path the tree does not name and for a tree git cannot
  read, so one arm had to mean one thing for both — and meaning *the worktree differs* let a release commit
  that shipped without the document its release is narrated in pass on the worktree's copy alone. Presence is
  asked by `ls-tree`, whose exit status answers it: an empty listing for absent, non-zero only for a tree it
  could not read. Absence anywhere but the release commit stays unremarkable, and the two facts are separated
  by where the commit is rather than by a read failing.

- **A member's directory is taken from its manifest path component-wise.** Both readers built a `"{root}/"`
  string and stripped it as text, and cargo reports **native** paths — so on a host whose separator is not
  `/` no member sits under the prefix and every one reaches the refusal for describing a different tree.
  `Path::strip_prefix` compares component by component, which is the rule the sibling reader for a
  dependency's `path` already carries as a requirement: a rule enforced at one site and not its neighbour is
  a rule about the site. The identity is joined back with `/` deliberately — it is compared against git's
  paths and cited in this repository's own prose, both of which spell a separator that way whatever the host
  does.

- **Two specifications disagreed about a TOML escape, and the implementation followed the other one.**
  `repository-checks` says the value reads as cargo reads it; `release-coherence` still required a
  cannot-judge, with an `AND` clause arguing why refusing was the better of two answers. The parser decodes,
  so the second was false — and **both of its scenarios were cited by tests whose names say the opposite**,
  one of which also disagreed with its own first doc line. The scenarios state what happens, and the test is
  renamed to what it asserts.

- **Three unreadable-value states said *not in double quotes*, and one also said *declared twice*.** The
  parser reads a literal string as cargo does, and a key declared twice is a document it refuses whole — so
  both shapes name conditions that no longer reach those states, in the type documentation a maintainer reads
  before the specification. Each now says *not a string at all*, and says what it is **not**, so the old
  reading cannot be re-derived from the new sentence.

- **A catalog search is handed a parsed catalog, so its *nothing names it* state carries one fact.** It
  parsed the manifest itself, once per inherited dependency, and mapped the parse failure onto the state
  meaning *no entry names it* — two facts under one name, and different things to tell an operator. The
  catalog is parsed once for the manifest now, the refusal belongs to the caller that meets it, and the
  repeated parse goes with it.

- **A changelog git cannot answer for is no longer read as a modified worktree.** One `is_ok_and` collapsed
  three causes into *not a snapshot*: the path missing from `HEAD`, which is the only one that means it; git
  failing to start; and git answering in bytes no `String` holds. The third came into existence when the
  runner learned to refuse those bytes rather than replace them — a predicate that was narrow when written
  widened underneath it. Each cause is answered separately now, and the two that leave the comparison unmade
  refuse rather than picking a state.

- **A checkout edited only in its trailing whitespace is edited.** The comparison trimmed both sides, so a
  worktree differing from the release commit by a newline read as unmodified and the gate answered *snapshot*
  over a tree that is not the one released. The trim was never a judgement about content: it compensated for
  the git runner trimming its own output. Reading git's answer exactly removes the compensation and the
  residue with it.

- **The one squash-message exception is one version across three positions.** `AGENTS.md` fixes the branch's
  role as `release/X.Y.Z` against a subject reading `release: X.Y.Z`. A `release/` **prefix** admitted
  `release/not-a-version`, and admitted `release/0.4.0` carrying `release: 0.5.0` — a branch whose whole
  purpose is one version, squashing a message about another. The gate now requires the role and the equality,
  and each of the three narrower readings this clause has had is recorded beside it: the subject alone
  admitted any branch, the subject and the destination admitted any source, the destination and a prefix
  admitted a branch that is not the role.

- **The one squash-message exception names both endpoints, because the contract does.** It is the
  **release-branch-to-`main`** squash. Taking only the destination left it reachable from any branch: a
  `fix/…` pull request onto `main` whose subject read `release: X.Y.Z` claimed a release branch's exception.
  Naming one endpoint of a two-endpoint contract narrows the door without closing it. The gate takes the head
  branch as evidence beside the base, and a head it cannot read stops the wrapper before the gate and the
  merge.

- **The one squash-message exception is identified by where the squash lands, not only by what it says.**
  `AGENTS.md` states it as the **release-branch-to-`main`** squash; the gate decided it on the subject alone,
  so a message reading `release: X.Y.Z` with an empty body claimed the exception on any base. The law was
  branch-scoped and the reaction was not.

  The gate now takes the base as evidence, exactly as it takes the title, and the wrapper supplies it from the
  pull request. A base it cannot read stops it before the gate and the merge: not knowing where a squash lands
  is not the same fact as knowing it lands somewhere ordinary, and a wrapper that guessed would decide the
  exception by default. Both directions carry their negative run.

- **A pinning citation now arrives with its mutation, or with the reason it has none, in the same change.**
  A `PINNED-BY` is held to its name resolving to one registered test; whether the test would fail if the
  behaviour it defends changed is decided only where a mutation is declared, and the biting check reports the
  uncovered part on every clean run. Citing is cheap and authoring a mutation is not, so a change that cites
  without mutating enlarges the population that check exists to report — measured when six requirements gained
  citations and none declared a mutation.

  **What was declined is the campaign.** Authoring mutations for the standing citation set was costed on its
  rate rather than its size: the set grows faster than mutations can be written, so chasing the numerator
  loses to stopping the denominator. The obligation costs nothing now and changes the trajectory; the grind
  costs a window and does not.

- **Two normative clauses said the semantic dimension's crate is the only one permitted to depend on `syn`,
  and it is not.** A `publish = false` member of this workspace names `syn` in `[dev-dependencies]`, which is
  permitted — the dependency allowlists observe the normal table only, a specified default with its own
  scenario — and the root manifest's own comment states that and names the occupant.
  `semantic-signature-coupling`'s requirement and `semantic-dyn-trait-boundary`'s aside now say *the only
  **packaged** crate that depends on `syn`*, the wording that manifest already reached. **No published
  surface, guarantee or reaction moves**: the requirement's own scenarios were already narrower than its
  prose — they assert that the static core does not acquire `syn` — so what changed is the sentence above
  them and not what anything reacts to.

- **The declared-set instrument for prose claims is not built, and the reason is measured rather than
  deferred.** The backlog entry *A claim about this tree, written as prose, is held only where its author
  declared it* proposed extending the census idiom: a declared phrase whose held value is a produced **set**,
  so *only 渾儀 names syn* is compared against the enumerator that answers it. Its stated floor was that
  coverage stays opt-in, declaring being an author's act.

  That floor was measured, and it decides the shape. The two live instances found in this change are claims
  their authors plainly believed — one of them a normative `SHALL` whose own scenarios were narrower than
  its prose. **Nobody declares a sentence they think is true**, so a declaration-armed instrument holds the
  claims someone already doubted and not the ones that go wrong.

  What found them was a sweep of the absolute-quantifier vocabulary — *only*, *alone*, *the one place* —
  within a line of a named enumerable subject. That is not the prose detector this repository has designed,
  measured and rejected three times: it decides nothing and starts from a subject the tree enumerates rather
  than from a sentence's meaning. It produces a review queue, which is the interim-instrument form already
  stated for the corpus-narrowing class.

- **Sixteen requirements-to-reaction citations added, and the half of the backlog entry that called itself
  cheap turns out to have a third state.** The entry *Every normative SHALL either has a reaction or is a
  declared bound* splits into extending the existing citation where a reaction exists (no capability needed)
  and deriving the binding for the rest (a capability). The first half was run against the sample the entry
  names.

  Measured: across every tracked spec, 371 requirements, and 297 carried no citation anywhere under them.
  `PINNED-BY` attaches to a scenario and never to a requirement, so the unit is a requirement with no
  citation under any of its scenarios. In `repository-checks`, six of the fourteen uncited requirements were
  citable and are cited, each pairing verified by reading the test against the scenario's `THEN` rather than
  matched by name. A planted citation naming nothing fails
  `every_pinning_citation_resolves_to_one_registered_test`, so the new ones are held.

  *The eight that remain are not "a reaction nobody cited".* Three of them are **gate-only**: the reaction
  asserts the clean tree while the requirement's `THEN` describes the refusal — `one_spelling` asserts that
  no second spelling of a token exists, where the requirement says the check refuses *naming every site and
  the constant that owns the token*, which nothing observes. One is a gate that **returns early where CI
  runs it**, and says so in its own comment. One is **undefendable by construction**, telling a reader that a
  projection is a view rather than an authority.

  *One limit of the citation grammar, found by using it.* A defence defined in two **targets** of one crate
  cannot be cited: the register refuses a name defined twice, and the recorded disambiguation is a crate
  prefix, which answers two crates and not two targets. That scenario stays uncited rather than pointing at
  half its defence.

  *Extending this to the rest is declined, on what a citation is held to rather than on its size.* The
  register decides that a citation's name resolves to exactly one registered test — proven by planting a name
  that resolves to nothing — and nothing decides the **pairing**. `pin_bites` closes the neighbouring half
  only where a mutation is declared, and even a biting pin ties its test to an author-chosen perturbation
  rather than to the requirement. So extending part one to the remaining requirements would produce
  hand-authored claims on the order of the requirement count, each held by name resolution — the drifting
  artifact the entry's own second half refuses. The citations added are kept: they are correct, and a renamed
  test turns them red.

  *And a cost this entry owed its sibling.* None of the new citations declares a mutation, so each landed in
  the part of the citation set `pin_bites` reports as uncovered on every clean run. Citing a pin is cheap and
  authoring its mutation is not, so work that cites more pins enlarges *most pinning citations have never
  been seen to fail* unless it authors the mutations too. That is booked in that entry rather than left in
  the change that caused it.

- **The workspace-version inherit question is parsed, and the hand-rolled TOML layer it was the last caller
  of is deleted.** Net 130 insertions against 548 deletions, more than half of
  `crates/kanhe/src/manifest.rs` among them, and no production code calls `region::Source::toml()` any more.

  *The defect that ended the approach rather than extending it.* A member may inherit through a sub-table
  heading — `[package.version]` with `workspace = true` — and cargo 1.96.0 resolves it, measured in a scratch
  workspace whose `cargo metadata` reports the inherited version. The reader asked each **line** whether it
  assigned `version`; a table heading assigns nothing, so no line answered and the member was refused for not
  inheriting a version it does inherit. That is the fourth round of one defect, each earlier round having
  moved the boundary of *decoded* one segment right — and the first found by measuring cargo rather than by
  reading a review. There was no segment left to move: the spelling is a heading, and a line-oriented reader
  cannot represent it at all.

  One expression over a parsed document answers every spelling the three recorded rounds fixed one clause at
  a time, and the sub-table form with them. With the inherit read migrated, `manifest::assignment`,
  `assigned`, `Assignment`, `Assigned`, `table_heading`, `TableHeading`, `dotted`, `unquoted`,
  `split_outside`, `outside_strings` and the escape decoder beneath them had no production caller left, along
  with the gate's own `inline_fields`, `assignments` and `offer_value` — whose doc comment already said it
  "survives for the one caller left". The compiler named each as it fell rather than a grep guessing.

  *One behaviour narrowed, in the direction that agrees with cargo.* The line walk took a
  `version.workspace` assignment in **any** table; cargo honours it under `[package]` and nowhere else, so
  the parsed read is scoped there.

  *One refusal gained an identity of its own.* A member manifest the parser cannot read is now a cannot-judge
  naming which member, rather than a member reported as not inheriting. It is registered separately from the
  dependency reader's site for the same condition, because the refusal register compares identities — a
  second construction of a held one would report the new branch as observed by a direction that never reaches
  it. Negative run: mapped to *does not inherit*, it answered `Violation` where `CannotJudge` was expected.

  The spec scenario carrying this requirement described **which reader owns the question**, and that prose
  had gone stale before this change — the readers it named were migrated earlier in the `0.6.0` window. It now
  states the behaviour and keeps the measurements, which is one fewer hand-maintained structural claim.

- **Six more promotion triggers evaluated, and one entry's evidence had gone false under this window's own
  work.** None of the six had fired; the finding is in what one of them rested on.

  *The one-spelling corpus reader* argued its safety from two premises. One still holds — the eight members'
  directory basenames equal their package names, re-derived — and the other does not: *the eight declare no
  registry dependency at all* is false. `serde_json` is declared by seven of them, `syn` by 渾儀, and
  `toml_edit` by 勘合 since this window's self-law amendment, which is the entry's own evidence overtaken by
  a change made two hundred lines away in the same document. What actually keeps the spurious edge absent is
  that **no such name matches any member's directory basename** — the property the trigger turns on, and the
  one the entry should have rested on from the start.

  *The bounds-named heading reader* is still latent, and the annotation says how narrowly: no
  `### Requirement:` heading carries a word ending in `bound`, which is the only corpus reaching it — but
  `Inbound` and `Outbound` now appear in `#### Scenario:` headings, so the vocabulary is in the tree and a
  rename away rather than hypothetical.

  The other four are annotated with what was checked: two governance members and no third, both tracked
  fixture corpora read, this repository's own commit objects absent outside the closed record, and the
  citation reader green.

- **The one merge the ritual cares about most was the one act the wrapper could not perform.** Sweeping the
  promotion triggers turned up one that had fired, on `release: 0.5.0` itself: *a merge or publish made
  outside the wrapper is not observed*. The release squash onto `main` was made with `gh pr merge` directly —
  and not because anyone slipped past the gate. **The wrapper could not do it.**

  `AGENTS.md` names that squash as the **sole message exception**: subject `release: X.Y.Z`, body deliberately
  empty. The merge gate refused both — as a non-Conventional subject, and as an empty body. It encoded every
  rule of the ritual except the one the ritual itself names, so the release snapshot had to go around it.
  `0.4.0`'s release commit carries an empty body too, so this had been true of **every** release.

  The gate learns that exception now, narrowly: exactly `release: X.Y.Z` with a well-formed version, which is
  the same line the release-history reader accepts — a malformed one is still refused, and by the
  conventional-subject rule rather than the empty-body one. An ordinary subject's empty body is still a
  violation.

  This is **more** observation, not a relaxation: the subject shape, the attribution marks and the title
  match are all judged on that merge now, where before none of them were because the merge never reached the
  gate at all. What remains of the entry is the half no repository can reach — a `cargo publish` run
  directly, or a merge made in the browser.

- **The OpenSpec lifecycle is retained, and the entry that asked read a rule as a claim about the tree.** The
  trigger was *a human call about intent* and the steward made it: a new capability must go through OpenSpec,
  so the section stays. The absence of instances is explained rather than unexplained — the recent windows
  were patches and prose, and neither is a capability change.

  The correction is the part worth keeping. The entry called the present-indicative *a capability change
  moves through OpenSpec* "prose stating a fact about the tree that the tree contradicts", and counted zero
  instances as the contradiction. **A rule is not falsified by having no instances; it is left
  unexercised.** The evidence it gathered was real and its reading of that evidence was the error — the same
  distinction this window wrote into `AGENTS.md`'s bar, where a rule needs a reachable instance to earn a
  *reaction*, not to be true.

  What survives is narrower than the entry and is filed as its own watch: the lifecycle is retained and has
  never been run, so the first capability change is the first thing that will tell anyone whether it still
  works. Half of it is already exercised — the sync-evidence rule is followed and the tracked
  `archive/.gitkeep` is exactly as described — and rehearsing the rest would mean inventing a capability to
  have one, when the lifecycle's whole subject is a change someone actually needs.

- **The promotion-trigger sweep was run, which is the thing its own entry says nothing runs.** Seven `Not
  fired` annotations carried evaluation dates of 2026-08-18/19 — **thirteen days and one shipped release**
  stale — and it was the dates that made that visible, which is exactly what they were added for.

  All seven were re-evaluated against the tree rather than re-dated. **None had fired.** Each now carries
  what was checked beside its date, because a date alone asserts an evaluation without saying what it looked
  at — the same shape as a figure without its instrument, one document over.

  What the checks were: `openspec/changes/` unchanged across the whole window; `examples/` moved 32 times and
  `sans-io-pure` still declares through the one profile with no bare boundary beside it; the capability
  filing join passes and the scenarios rewritten this window stayed under the capabilities already holding
  them; the sites named by the length entry were worked in heavily and **shrank**; `PROJECT.md` moved eight
  times and its two decidable graph claims were re-derived true.

  One is not a defect and is not mine to decide: the OpenSpec lifecycle entry's trigger is *a human call
  about intent* — whether the lifecycle is being restored or acknowledged as abandoned — and it has now gone
  unmade through a full release.

- **The governance-ratio entry's own instrument was a pipeline with an ellipsis in it, and it was caught by
  trying to run it.** `git log … | …` describes a measurement rather than being one — the same shape this
  window removed from `AGENTS.md`'s census rule, filed by the same hand, in the entry whose entire purpose is
  to make a figure re-derivable. The classifier is written out now.

  Re-running it also found a second thing the entry had not said: the range must be taken against the
  **release branch's own history**, never against `main`, which squashes a whole window into one commit. The
  same range on `main` answers `0/1`, which is what the first re-run returned.

  **First reading: 8/17 (47%)** for `0.6.0` so far against `0.5.0`'s 52%, with `crates/kanhe` down from
  37,154 lines to 36,460 — the first window in which it has shrunk. Neither number carries much yet, and the
  entry says so: seventeen changes is a sample two differently-classified commits would move ten points, and
  nearly all of them are one hand working on governance, so it measures who was working more than whether the
  bar changed anything.

- **The dependency grammar is parsed, and the spellings cargo accepts are decided rather than refused.**
  `declared_dependencies` asked a heading's text which table it opened, matched it segment by segment against
  every admitted form, then collected a detailed table's fields across lines and filed the record when the
  *next* heading proved the table over. It walks the document now: `[dependencies.xuanji]` and
  `xuanji = { … }` are one entry in one table, and there is no boundary to find.

  **Four states became unconstructible and six refusal sites retired with them**, because what they refused is
  now either decided or refused by the parser. A quoted key names its crate, so `"xuanji" = "0.0.1"` is a
  stale family requirement that is **judged**. An escaped path is decoded and compared against the member's
  own directory. A `package` written `xuan\u006ai` is matched against the family as `xuanji`. And a key
  declared twice is a document **cargo itself will not load**, so the honest answer is the parse error, which
  names the key and its position rather than a count this reader had to keep.

  **Two fixtures were themselves invalid TOML, which the hand-rolled reader tolerated.** Both appended a
  second declaration of `xuanji` beside the example's own, and a line reader never met the collision. They
  compose one declaration now, and assert that the edit landed — a fixture that stopped being about the thing
  it names is the shape a reader of lines cannot see.

  Ten integration directions moved with the reader, each to what a parser answers rather than by
  substitution: three to the parse refusal, three to a value that is no string, two to a judgement where they
  had asserted a refusal, and two to a fixture that composes correctly. Four unit directions were deleted with
  their subject — they pinned a hand-rolled lexer — and one moved onto the new reader, keeping the property
  that a value is not a key.

  Negative runs: with a rename left unresolved, four directions bite; with a parse failure read as declaring
  nothing, five.

- **The `READY-PATCH` queue is dispositioned, and none of the five remaining closes by patching now.** That
  is what the class says it means: it grades **evidence and compatibility, not how much design the correction
  still needs**. Reading it as *small* or *next* is the misreading the class definition itself warns about,
  and it is the misreading this pass started with.

  - *The breaking-marker pairing* — the shape is decided (a named join with explicit handles), and
    dispositioning it surfaced a collision the design has to answer: backfilling handles would rewrite
    `[0.5.0]`'s **released** Migration bullets, which this repository refuses as a stated bound. Resolved by
    holding the join over sections still being written; recorded in the entry. Its own trigger is a **second**
    unmarked entry and one is observed, so it waits.
  - *Pinning citations seen to fail* — ongoing, not closable: `pin_bites` prints 4 declared mutations covering
    4 of 186 cited tests, and the entry states that authoring one is per-bound expert work. Adding one would
    read as progress and close nothing.
  - *The shell's semantic delegation* — closing it needs the semantic outcome to be unreachable except through
    the observer, *a design step and not a call-site swap*, in the entry's own words. Tracked by a declared
    unpinned bound.
  - *Every normative SHALL reacted or bounded* — its trigger has fired **six times**, three of them in
    requirements written by the window that was closing the class. The obvious instrument, a citation per
    SHALL, is refused by this family's own rule against hand-maintained pointers on the order of the thing
    counted.
  - *A prose claim held only where declared* — two instances; the trigger is a third, and the instrument is
    sketched as a sibling of `Census` holding a produced **set** rather than a figure.

  The first draft of this entry spelled the breaking marker literally while describing it, and the reaction
  refused the release for a section that marks a change and carries no migration — which is the declared
  over-reaction that entry names in its own words: *the classifier reads the marker's presence rather than
  its position*. The bound fired on prose about itself, and the entry is written without the token instead.

- **Four of the manifest readers are parsed; the fifth is scoped rather than attempted again.**
  `workspace_version`, `package_name`, `require_lock_versions` and `publishable` ask a real parser now.
  `declared_dependencies` does not, and `BACKLOG.md` records why with the measurement from the attempt
  rather than an estimate: **287 insertions against 1,168 deletions**, four enum variants made
  unconstructible, six refusal sites retired, four unit directions deleted with their subject, two spec
  scenarios rewritten, and ten integration directions each needing a judgement about the right new answer —
  one of whose fixtures is itself invalid TOML the hand-rolled reader tolerated.

  It does not split. The four variants die together the moment the reader stops constructing them, and
  staging it by keeping the hand-rolled reader alongside the parser would be two readers of one question,
  which is the defect being repaired.

  The attempt is also recorded because of how it went wrong: a failure check that grepped for test names
  could not see a test module that had **stopped compiling**, so several steps ran on a false green. The
  check used since asks for a `test result` line per binary and treats a compile error as not green.

- **The publication reader is parsed, and the key it could not decode is now decided rather than deferred.**
  `publishable` walked its own table and asked a shared key reader whether a line assigned `publish`. A key
  spelling it could not decode answered `Unreadable` — the safe answer, and not the right one: measured
  against cargo, `"\u0070ublish" = false` reports `publish=[]`, so the crate does **not** publish. The parser
  decodes the key and the verdict follows.

  The negative run is the part worth keeping. With an undecoded key passed over, `publishable` answers
  **`Yes`** for a crate cargo refuses to publish — which is what the old `Unreadable` was standing in front
  of. Two rows now hold it: the decoded key, and `publish.workspace = true`, which defers to the workspace
  manifest and is still no verdict this text carries.

  `classify` went with it — the value classifier is what `as_bool` and `as_array` answer, so it became dead
  code the moment the parse landed.

- **The lock reader takes `[[package]]` from the parser, which removes an ordering premise it had to know.**
  The block boundary was the literal string `[[package]]`, and beneath it sat a fact about cargo's output
  order: `source` is written *after* `version`, so filing an entry before the body ended recorded every one
  as source-less — and source is what tells a workspace member from a registry entry sharing its name. An
  array of tables has neither question. Each element **is** one entry, and the order its keys were written in
  is not something this reader has to know.

  **A diagnosis it could not give before.** An unparseable lock produced no blocks, so the reader reported
  *Cargo.lock is missing workspace package …* — a violation naming a package that may be sitting right there,
  in a file cargo cannot read either. It is now a cannot-judge saying which fact was met, with its own
  direction; the negative run that folds the parse failure into an empty document reports the old
  misdiagnosis verbatim.

  Two more sites had their WHEN moved by the same cause — both used a single-quoted value the parser now
  takes — and what reaches them is a `name` or a `version` that is no string. One message drifted in the
  rewrite and was restored to what it said before: a direction asserts it, and changing a refusal's words for
  no reason is churn a reader has to re-learn.

- **The second whole-document manifest reader is parsed rather than hand-split, and its improvement is
  guarded because a negative run said it was not.** `package_name` now asks a real parser for
  `[package].name`; 46 lines of hand-rolled table walking are gone.

  The two directions that observed its unreadable state both used a **single-quoted** name — legal TOML that
  cargo resolves and the old reader declined — so both had their WHEN moved to a `name` that is no string at
  all, which is what still reaches the site. Then the negative run that restores the old double-quote-only
  rule **broke nothing**: with both WHENs moved, nobody was left observing the new answer, and the
  improvement could have been reverted in silence. A direction now judges a stale example pin behind a
  single-quoted member name, and the same perturbation fails it.

  That is the second time in this window a reader's *improvement* went unobserved while its *refusal* stayed
  pinned. Moving a WHEN keeps the site honest; it does not carry the new answer, and nothing says so.

- **The manifest grammar is parsed rather than hand-split, and the amendment that allows it was measured
  before it was proposed.** `toml_edit` enters `kanhe`'s dependencies through the self-law amendment ritual —
  the allowlist in `crates/shengmo/src/law.rs`, the regenerated `AGENTS.self-law.md`, and the boundary
  declared verbatim in `self_law_amendment.rs`, all three of which must move together or the check refuses.

  **The case, measured rather than argued.** Of the 0.5.0 window's 540 landed changes, 89 touched
  `manifest.rs` or `release_coherence_gate.rs` and **69 of those 89 were typed `fix`** — one hand-rolled
  grammar produced the largest run of repairs in the repository. Every shape those repairs were about was
  then put to a real parser, which answered all of them: a dotted pin as one dependency rather than two, a
  quoted tail as a path rather than none, a key literally named `version.extra` as one key, an escaped dot in
  a heading not overwriting the real table, three inherit spellings as one answer, a comment glued to a value,
  a commented-out pin, and `[lib]` before `[package]` not becoming the package name.

  **`toml_edit` rather than `toml`, because both halves are needed.** The structure says which dependency
  carries which field; the original spelling is required because `release-coherence` holds an internal pin to
  equal the workspace version *as written*, and a normalising reader loses that. `cargo metadata` was weighed
  and does not serve: it reports a requirement normalised to `^0.5.0`, and whether a member inherits its
  version or hardcodes it is invisible in resolved data.

  **Two declared limitations turn out to have been false refusals, and both are gone.** A single-quoted
  version, and `[workspace]` with the table composed as `package.version`, `package = { version = … }` or
  `package."version"` — measured under cargo 1.96.0, **all four resolve at the declared version**, and the
  reader refused them. One of the two was written into this specification as a limitation the reader has.

  **Two refusal sites kept their place by having their WHEN rerun**, which is the ritual this repository
  states for retiring a bound: their old WHEN was the single-quoted value, and what reaches them now is a
  `version` that is not a string at all — the catalog declaring that it inherits — plus a manifest the parser
  cannot read, which is one cargo cannot read either. A duplicate `version` key is now answered by the parser,
  which names the key and says *duplicate key*.

  Scoped deliberately to `workspace_version`. The line-oriented readers (`assigned`, `assignment`,
  `table_heading`) are reached from thirty-one call sites and each carries its own bound accounting; migrating
  them is separate work, not a larger version of this change.

- **`AGENTS.md`'s carrier taxonomy answers which binding a carrier admits and never whether the claim earns
  one, and that missing question is what the 0.5.0 window answered by default.** It built `crates/kanhe` from
  nothing — 142 refusal call sites, 98 declared bounds, about 37,229 lines of Rust replacing 1,562 lines of
  shell — while over half its landed changes touched that machinery and no published source.

  The bar now stated has two halves, and the split is measured rather than asserted. **A cannot-judge is not
  a rule**: a reader declining to answer where it cannot see is never the forbidden direction, since the
  alternative is a silent skip. **A violation is a rule and needs a reachable instance** — a shape a
  maintainer would plausibly write, demonstrated against the real toolchain, not a fixture built to display
  the gap and not a gap derived by reading the code. Otherwise it takes prose and a trigger, the form two
  rules took the same day and which cost a sentence each.

  **Two discriminators for retiring existing rules were tried and neither survives.** *Only ever fired on
  fixtures* answers **all** of them, because every direction here builds a synthetic fixture — that is the
  method. *Named by no release note* answers 135 of 140, and measures how release notes are written: **70 of
  those are cannot-judge**, and retiring one trades a refusal for silence. So no retirement list is offered.
  The reducible weight is elsewhere and was measured too — tests are 26,568 of the crate's 37,229 lines —
  and none of it shrinks by retiring refusal sites.

  The bar is on the **rate**, and that is the reason it is the answer rather than a cut. Retiring rules once
  leaves the mechanism that produced them, and this window's reviews recorded it: every reaction built had a
  defect found in the next round, three of three.

- **A relative anchor was a vaguer restatement of a sentence three lines above it, so it is deleted rather
  than dated.** The `InherentGenerics` entry closed its cross-module half and said exactly what closed it —
  *the module role was added in the 0.4.0 window* — then described the remaining risk as *one step narrower
  than when this entry was written*. That phrase names a moving reference for a narrowing the entry had
  already named by its **item**, which is what `AGENTS.md`'s disposition table asks for. Nothing is lost by
  removing it; giving it a date would have added a second, weaker carrier of one fact.

  Found by a sweep run for a different repair and reported rather than folded into it, then done on its own.

- **The workspace's members had two enumerators and nothing asked whether they agree.** `cargo` names its
  members in `[workspace] members`; the release gate reaches them by walking `crates/*/Cargo.toml`. Both
  answer *which crates are the family*, and the premise that they answer the same held by **layout** — which
  is not something anyone declared.

  The direction that matters is the false negative: a member declared outside `crates/` is invisible to the
  walk, so it is held to no inherited version, enters no family the catalog's pins are judged against, and is
  read in no lock. Its stale pin would reach `cargo publish` through a subject that never contained it — the
  same shape as a family crate offered without a `path`, one layer up, in the **enumerator** rather than the
  selector. Both directions are now asserted, the other being a crate cargo does not build held to this
  workspace's version.

  **Asked of this repository rather than inside the gate.** The gate's phases are a sequence whose order is
  observable and whose failure matrix asserts which refusal a repository meets first; adding a
  `cargo metadata` call to that sequence would move that order for every repository it judges. The premise is
  about *this* workspace's layout, so the question is put to this workspace — and it goes through the gate's
  own walker rather than restating it, since a third enumerator is the defect itself.

  Negative runs, both measured: a member added at `tools/probe` is reported as unreached by the walk; a
  manifest at `crates/stray` that `members` does not name is reported as held to a version cargo never builds
  it at.

- **A closed entry called a consequence *structural*, and that is how its premise escaped being examined.**
  The `BACKLOG.md` record of the `0.5.0` provenance row reasoned that the row is an audit of the tarballs and
  so cannot precede them, that the branch carrying it is archived at the release squash, and that `main` takes
  nothing except through a release branch — therefore the one-cycle lag was **structural and remains**. Every
  step below the premise was accurate.

  The premise was that a commit sha1 is kept in this repository at all, and the entry never put that
  question. Once it was put, the answer removed the lag instead of accommodating it. The entry now records
  where it reasoned wrongly rather than only what it got right.

- **The one instance the widened rule forbade is gone, and the tree now agrees with the rule.**
  `crates/kanhe/tests/merge_workflow.rs` used a real commit of this tree as its fixture head — made and
  squashed away inside the `0.5.0` window, contained in no branch, resolving only in a clone that still
  happened to hold it. The fixture needs an **opaque token**, not a reference: what the assertions require is
  that the wrapper pins the head its evidence came from. It now uses a value that resolves to no object in
  any repository, and the stub says why the value is what it is.

  Swept afterwards rather than assumed: every 40-hex literal in tracked content put to `git cat-file -t`.
  Seven still resolve, and all seven are rows of `docs/history/published-artifact-provenance.md` — the record
  the rule exempts, and one that is now closed, so the exemption has no way to grow.

- **The rule against citing this repository's own commits covered live prose only, and one instance sat
  outside it.** `AGENTS.md`'s disposition table said *a commit object in live text*; the reaction behind it
  reads `.rs` as comments only, on the coherent ground that prose carries citations while code carries
  values. `crates/kanhe/tests/merge_workflow.rs` carries one as an opaque fixture token — a real commit of this tree,
  made and squashed away inside the `0.5.0` window, contained in **no branch**, which resolves only in a
  clone that still happens to hold it. A reader cannot tell that from a reference, and neither can a fresh clone.

  The row now reads *anywhere in tracked content*, and carries what makes the rule correct rather than merely
  strict: a **third party's** object is not this shape. An action pinned as `owner/action@<sha>` is right, and
  the same criterion excludes it without maintaining a list — that sha does not resolve here.

  **Stated as prose with its reaction's reach named, which is a first-class form here.** The census rule
  already declares that it has no repository check rather than leaving it to be discovered. `BACKLOG.md`
  carries the upgrade path with a trigger of a second instance outside live prose, and the test it would use,
  which is decidable today: `git cat-file -t` on every 40-hex literal decides whether it is an object of this
  repository.

  Not built now, and the reason is the measurement in this same section: over half of the last window's
  landed work fed the machinery that judges this repository. One instance does not earn a reaction there —
  but it does earn a sentence, and the difference between those two was the mistake in the first reading of
  this question.

- **The published-artifact inventory is closed: a commit sha1 does not belong in this repository.** Its
  timing and its price are both wrong. The sha1 exists only *after* the upload that records it, so a
  committed copy cannot be written before the act — and the branch that would carry it is archived at the
  release squash, so it arrives one release behind the thing it describes. The price of writing forty
  hexadecimal characters is a branch, a pull request, a full CI run and a squash merge; `0.5.0`'s row was
  paid for exactly that way, one cycle late.

  What it bought was a **second copy of a permanent record**. The document's own premise is that
  `cargo publish` writes the sha1 into the tarball and a version can never be re-uploaded — the registry
  holds it unalterably for as long as the crate exists. The committed copy is the only half that can go
  stale.

  The existing rows stay. They are a real audit of the era before the publish-source reaction existed, and
  their *verdicts* — which disagreement, and which mechanism produced it — are judgements an audit
  established rather than figures anyone can look up. What stops is the growth.

  Nothing replaces the table because the question was always answerable without it: the record's command
  asks the tarball and the tag directly, for any version, at any time. The section that carried it is
  renamed from *Reproducing the audit* to what it actually is.

  Stated rather than left to be discovered: the one mechanism the reaction **cannot** prevent is a release
  snapshot force-pushed away after a clean publish, as `0.2.2` was an hour later. A row appended at publish
  time never catches that — at publish time the gate has just passed — so per-release rows were never its
  instrument. Re-running the audit later is.

  `AGENTS.md`'s pointer is made consistent in the same change. The sentence in the dated `[0.5.0]` section
  that calls the document an inventory of *every* published version is left alone: rewriting a dated section
  to satisfy a rule written afterwards falsifies the record, which this repository holds as a stated bound.

- **A release checkout being edited was judged as a release checkout, and no tree could satisfy the state it
  was put in.** `State::Snapshot` was decided by `head == release_commit` alone — a fact about the
  **commit** — while every other reader in the reaction takes its content from the worktree through
  `std::fs::read_to_string`. The first change of a new cycle falls between those two sources: sitting on the
  release commit, an author writes the `[Unreleased]` entry that **development requires**, and it is judged
  in **snapshot**, where `[Unreleased]` must be **empty**. Two real rules, no tree satisfying both, and the
  only escape is to commit — the act that moves `HEAD`. Measured on this repository: `release/0.6.0`'s first
  change could not pass the Definition of Done until it was committed, and passed immediately afterwards
  unaltered.

  A snapshot is a **checkout**, not a commit. The state now also asks whether the `CHANGELOG.md` being judged
  is still the one that commit carries, so the state and the content come from one source.

  **Two wider spellings were tried and refuted by the corpus, which is the part worth keeping.** Asking
  whether *anything* tracked was modified made a release checkout whose `Cargo.lock` had been replaced by a
  directory classify as development — that is a broken release checkout, not a new cycle — so it reported a
  missing `[Unreleased]` entry instead of the lockfile it could not read. And asking `git status` read the
  **index**, which intercepted a corrupt-index fixture another guard uses to reach its own refusal: measured,
  that guard stopped reaching it, taking a WHEN that was not this reader's. `git show HEAD:CHANGELOG.md`
  takes the object database instead, and the same corruption is left to the reader that owns it. Both were
  caught by existing directions rather than by review.

  This repair made the reaction **quieter** and relaxed nothing: the failure it removes was a
  **misclassification**, not a rule that was too strict. Downgrading the verdict would have kept the
  misdiagnosis and lowered its volume, in the direction the Core Contract orders above every other.

- **The 0.5.0 window spent more than half its landed work on the machinery that judges this repository.**
  Measured over the window's own history, one landed change per squash: `284/540 (52%)` touched
  `crates/kanhe` or `crates/shengmo` and no published crate's source; `100/540 (18%)` touched a published
  source at all. Two reviewers separately recorded the other side of it — the published surface moved by two
  lines of a private doc comment across sixteen rounds.

  Filed in `BACKLOG.md` with the command that produced it, so the next window's figure is derived rather than
  recalled, and with what the figure argues for: the weight is in how much of the tree is under reaction at
  all, not in how loudly a reaction speaks, so the lever is retiring rules rather than softening verdicts. No
  reaction is proposed for the ratio itself — it rests on which crates are the product, which is a judgement,
  and the remedy it points at is removal, which no reaction performs.

- **A record stated the gate's current answer, and was wrong three times running — the fourth correction
  would have been the same mistake.** A *Version horizons* paragraph in `BACKLOG.md` ended by naming what
  the release-coherence gate reports. It said `development: 0.4.0`; that was corrected to
  `release-ready: 0.5.0`; and that was **already false at the commit that froze it**, because the release
  squash makes HEAD's subject `release: X.Y.Z`, which is `State::Snapshot` — the tree it shipped in reports
  `snapshot: 0.5.0`. Measured at `v0.5.0` itself.

  The defect was never the label. A gate's answer is a **live state**, and a record cannot hold one: the
  paragraph's own two earlier drifts are its evidence, and the paragraph already prescribed the remedy for
  its commit counts one screen earlier — *re-derive rather than trusting a figure here*, with the command to
  do it. It simply had not applied that to itself. It now states the property that survives — the gate reads
  versions, never a branch name — and carries the command that prints the state for any checkout.

  Two smaller carriers went with it: *has since done **all four*** was a hand-written count of the list
  beside it, and *when this entry was written* was a relative anchor in a paragraph that names an absolute
  date two sentences earlier.

  It could not be fixed before the release. The only way was to rewrite `main`'s release snapshot — the act
  that orphaned `0.2.2`'s tarballs — over one sentence in a document that ships in no crate and fails no
  reaction. A BACKLOG entry is also a record, and this repository's own stated bound refuses rewriting a
  record to satisfy a rule written afterwards.

- **`0.5.0`'s published tarballs name the commit its tag names, and the record now says so from measurement.**
  All six were pulled from `static.crates.io` on their publication day and their `.cargo_vcs_info.json` read:
  one distinct sha1 across the six, and it is the commit `v0.5.0` names — so
  `docs/history/published-artifact-provenance.md` records *agrees with the tag*, the verdict `0.4.0` could not
  earn (it was published from the release branch) and `0.2.2` lost afterwards (its snapshot was force-pushed
  away). This is the first release the publish-source gate stood in front of, and it is the first whose
  provenance was audited from the artifacts rather than reconstructed later.

  The row could not be written in `0.5.0` itself: it is an audit **of the tarballs**, so it cannot precede
  them, and the branch that would have carried it is archived at the release squash while `main` takes nothing
  except through a release branch. `BACKLOG.md` carried the pointer across that gap and is retired with it.
  The lag is structural and remains — this row reaches `main` only when the next release is cut, whatever
  number that window earns.

  The audit's scope line was restated rather than having its number bumped: the 2026-08-05 audit covered the
  96 tarballs then on the books, and the six new ones were audited on 2026-08-28, so the sentence says which
  audit covered what instead of letting one date stand for both.

[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v0.6.0...HEAD
[0.6.0]: https://github.com/tacticaldoll/tianheng/releases/tag/v0.6.0
