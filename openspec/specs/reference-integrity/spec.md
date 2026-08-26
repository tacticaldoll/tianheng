# reference-integrity Specification

## Purpose

Keep tracked in-repository path references and Tianheng's required governance surface honest under a hermetic
policy, so a checkout's verdict does not depend on ambient process state.

## Subject

- `crates/kanhe/tests/reference_integrity.rs`

## Requirements

### Requirement: The real governance-document policy SHALL be hermetic

The reference-integrity gate SHALL carry Tianheng's required governance-document set literally. Ambient
environment variables SHALL NOT replace or narrow that set, so the same checkout receives the same required-surface
judgment regardless of its parent process.

**Ambient git configuration is a second channel, and this requirement never covered it.** The Purpose above
says a checkout's verdict does not depend on ambient process state; this requirement is scoped to environment
variables and to the document set, and git config is neither — which is how the gate's own ignore query went
a window reading whoever's personal `core.excludesFile`, quietly excusing a stale reference. That channel is
held by `repository-checks`' requirement that a judgement closes the ambient channel of any read whose answer
an ignore file changes, whose corpus includes this gate's own source. A Purpose wider than every requirement
under it is not a claim anything defends.

#### Scenario: Ambient state names a smaller set

- **WHEN** a required governance document is absent and the process environment names a smaller document set
- **THEN** the reaction still fails, naming the absent required document

### Requirement: A fixture corpus SHALL be supplied as an argument, never as a narrowed policy

A direction judging a repository other than Tianheng's own SHALL supply that repository's corpus and tracked
set **as arguments to the judgement**, so a fixture and the real workspace run the same code over different
inputs. The required governance-document set SHALL NOT be narrowable at all — not by an option, not by the
environment, not for a fixture.

**This requirement was rewritten to describe what the port does.** It used to require the gate to *accept an
explicit fixture-only governance-document set*, refused on the real workspace, with an unreadable or surplus
input naming what could not be read — a CLI-shaped option the shell-era gate carried. The shell-to-Rust
migration removed it and gave the fixtures a stronger shape instead: `offences_in` takes the corpus root, the
tracked paths and the corpus as parameters, and `GOVERNANCE_DOCUMENTS` is a compile-time `const` no caller can
reach. The three scenarios below replace three that described the option's behaviour, which nothing had
implemented for two windows.

The direction the old wording was protecting survives, and is stronger: a narrowing that cannot be expressed
cannot be requested on the real workspace, so the refusal it demanded is unreachable by construction rather
than by a check.

#### Scenario: A fixture corpus is judged by the same code as the real workspace

- **WHEN** a direction supplies a throwaway repository's root, tracked set and corpus to the judgement
- **THEN** it is evaluated by the same function the real workspace is, so the fixture demonstrates the
  reaction rather than a copy of it

#### Scenario: The required governance-document set cannot be narrowed

- **WHEN** any caller, on any repository, attempts to supply a smaller required governance-document set
- **THEN** there is no parameter, option or environment variable that carries one; the set is a compile-time
  constant, so the narrowing this requirement once refused is not expressible

#### Scenario: A fixture corpus that inspects nothing is refused

- **WHEN** the supplied corpus yields no inspectable source
- **THEN** the judgement fails rather than reporting clean, because a verdict over nothing is not a verdict —
  the refusal the fixture-only option existed to make observable

### Requirement: Tracked checkout content is the reference evidence

The reference-integrity gate SHALL judge repository paths against Git-tracked content and tracked ancestor
directories, never untracked filesystem state. Its workspace-member classification SHALL be derived from tracked
`crates/<name>/Cargo.toml` paths, so an untracked crate manifest cannot make an illustrative crate reference
enforceable.

**Which formats carry prose SHALL be one declaration, and every tracked format SHALL be classified.** Each format
this repository tracks SHALL be named as whole-document prose, as prose on the lines whose first non-whitespace
token is a stated line-comment marker, or as carrying no prose at all. A format the repository holds and the
declaration does not name SHALL fail, naming that format — not default either way, since a silent *no prose*
reads a new format as having none and a guessed marker asserts one it may not have. The corpus, and which of a
file's lines are read, SHALL both derive from that one declaration. Outside active `openspec/changes/` plans, the
gate SHALL inspect every classified format's prose, including Rust rustdoc forms. A Rust test source SHALL NOT be
excluded wholesale; its admitted comment lines are judged through the same region rule as other Rust.

**Two lists is the shape that failed.** An extension filter decided what to open while a marker rule decided
which lines to read, so a format could sit in one and not the other — and shell did, for a whole window, while
the marker rule had known `#` all along. The files that left unread are the sanctioned merge and publish
wrappers, which cite the Rust gate they sequence *by path*, where a renamed test target is exactly what rots a
citation; YAML, where this repository's own gate list is duplicated, was unread the same way. Adding one
extension per discovery is the denylist shape the 0.5.0 window replaced twice elsewhere. A tracked script's shebang
SHALL NOT be a reference: it names an absolute path outside every prefix this gate recognizes. Before judging references it SHALL require the repository's
governance-document surface, at least one tracked workspace member under `crates/`, and at least one inspected
source; absence of any prerequisite SHALL fail loudly rather than read as clean.

#### Scenario: A complete tracked checkout is inspectable

- **WHEN** the repository contains the required governance documents, a tracked workspace member, and a tracked
  source of any format the declaration classifies as carrying prose
- **THEN** the gate builds its tracked-path evidence and evaluates the corpus without consulting untracked files

#### Scenario: Required evidence is absent

- **WHEN** a required governance document, every tracked workspace member, or every inspectable source is absent
- **THEN** the reaction fails, naming the missing prerequisite instead of reporting clean. **The reaction is
  the gate — the whole test target — not each direction inside it.** A prerequisite may therefore be held by
  a sibling direction rather than by the resolution walk calling it, and two independent reviews read this
  line as requiring the second before it said so. What the requirement forbids is the gate reporting clean
  over absent evidence; how the directions divide that work between them is not a claim this makes

#### Scenario: An untracked manifest cannot create a workspace member

- **WHEN** tracked prose names a missing path under an illustrative crate and only an untracked `crates/<name>/Cargo.toml` gives that crate member shape
- **THEN** the gate leaves the reference outside its existence judgment and retains the verdict produced from tracked evidence alone

#### Scenario: An active OpenSpec plan names future paths

- **WHEN** a tracked file under `openspec/changes/` references a path the plan intends to create
- **THEN** that transient plan is excluded from the inspected corpus and does not produce a stale-reference verdict

#### Scenario: A comment names an absent path, in any classified format

- **WHEN** a tracked shell script's or CI workflow's comment names a repository path no commit holds
- **THEN** the reaction fails and names it, rather than leaving that format's citations unread

#### Scenario: The repository holds an unclassified format

- **WHEN** a tracked file's format is not named by the declaration
- **THEN** the reaction fails naming that format, because a format read by nothing leaves every sweep here
  reporting clean over prose it never opened

#### Scenario: A test source names a deleted live path

- **WHEN** a tracked Rust test comment names a recognized path this repository deleted rather than a path the
  test fixture constructs
- **THEN** the reaction fails and names the stale reference instead of excluding the whole test source

### Requirement: Reference syntax determines path resolution

The gate SHALL recognize repository-relative paths under Tianheng's own top-level directories, Markdown
link targets, bare `tests/*.rs` references written inside member crates, and bare filenames carrying a
governance or Rust extension. Markdown links SHALL resolve lexically relative to the referring file. Bare
test references SHALL be satisfied by the matching tracked test under any workspace member. A
`crates/<name>/...` reference SHALL be judged only when `<name>` is a real workspace member; illustrative
non-member crates and glob patterns SHALL remain outside the existence judgment.

A bare filename SHALL react only when this repository once tracked that name outside a change directory and
tracks it no longer. A name any tracked file still carries resolves; a name no tracked file has ever carried
is not a path but an illustrative shape, which is what admits the Rust extension without judging every
fixture name this repository's prose invents.

#### Scenario: A stale repository-relative prose path reacts

- **WHEN** tracked prose names a recognized repository-relative path that is not tracked
- **THEN** the reaction fails and names the stale reference and referring file

#### Scenario: A stale Markdown link reacts

- **WHEN** a Markdown link resolves lexically from its referring document to a path that is not tracked
- **THEN** the reaction fails and names the stale link target

#### Scenario: A bare member-test reference is absent everywhere

- **WHEN** Rust source in a workspace member names `tests/*.rs` and no workspace member tracks that test path
- **THEN** the reaction fails and reports that the reference is tracked under no workspace member

#### Scenario: A bare filename names something this repository deleted

- **WHEN** live prose or a source comment names a bare filename this repository once tracked outside a change
  directory and tracks no longer
- **THEN** the reaction fails and reports that this repository deleted it

#### Scenario: A bare filename no tracked file has ever carried is not a path

- **WHEN** prose names a bare filename this repository has never tracked, as an illustrative name inside an
  explanation of a shape
- **THEN** the reaction is silent, because such a name describes a shape rather than naming a file

### Requirement: A reference SHALL name a thing, not a position

The comment lines of every format the declaration classifies as carrying line comments SHALL NOT reference an
item by its position — a counted offset, a definite article naming no thing, or an adverb standing in for one. A
reference SHALL name the item: an intra-doc link where the documentation can reach it, otherwise the identifier
or the path. A direction word following a named construct is a reference to a thing and SHALL NOT react.

**The scope SHALL be derived from that one declaration, not listed again.** This rule was written over `.rs` and
`.sh` by extension — a second list beside the declaration, which is the defect the declaration was introduced to
end, and it left `.toml`, `.yml`, `Cargo.lock`, `CODEOWNERS` and `.gitignore` unswept while the reasoning here
covers every one of them. A format admitted to the corpus SHALL be swept for both properties or for neither.

**The ladder this sits at the bottom of.** An intra-doc link is checked by the compiler; a path is checked by the
sweep above; a path with a line number is checked by nothing; a position is not even a name. Measured on this
repository, two such references were off by 86 and 98 lines, and the second was written after the first had been
corrected — the criterion `scripts/publish.sh` states for itself, that a rule stated and then missed needs a
check rather than another sentence.

The corpus SHALL be comment lines, by the same rule that decides the sibling sweep's corpus, so a specimen
written as a string literal sits on an executed line and cannot be read as a reference. That is a position rather
than a marker: nothing can hide a comment inside an executed line, and the check's own explanation of the shapes
it refuses would otherwise be the corpus it judges.

Markdown SHALL be outside this requirement, **by construction rather than by omission**: it is the format
classified as whole-document prose, so it is not a line-comment format and no exclusion has to be written for it.
In a record — a `CHANGELOG.md` entry, a `BACKLOG.md` history — a positional phrase narrates a past state, and
separating that from a live reference is a judgement over prose, which this repository has designed, measured,
and declined. In source there is no such reading: a comment describes the file it is in.

**A structured coordinate SHALL be refused in every tracked format, whole-document prose included.** The
exclusion above is about a positional *phrase*, and it stands. A backticked `` `<tracked-path>:<line>` `` is not
a phrase: it is decidable by **shape**, exactly as a bound id is, so refusing it reads no prose and reopens no
declined judgement. The ladder's own argument reaches it without help — a position is not a name, and it is not
one in any tense, so a record citing a coordinate serves its reader no better than a live reference does. Naming
the entry costs a clause and cannot rot.

Measured before this was written: two existed, both in one `BACKLOG.md` clause, both correct when written and
both since landed mid-paragraph in unrelated entries — one of them moved by the very window that repaired the
source instances. After their repair the reaction holds an empty set, which is kept rather than pruned, by the
same rule that keeps a recognizer asserting its own emptiness elsewhere here.

#### Scenario: A coordinate in whole-document prose

- **WHEN** a tracked Markdown file carries a backticked path with a line number, in a record or in a live clause
- **THEN** the reaction fails, naming the file, the line and the coordinate, because a position names no thing
  in any tense — while a positional *phrase* in the same file stays outside, unread

#### Scenario: A line whose code-span membership cannot be decided

- **WHEN** a tracked line carries an odd number of single backticks, because a Markdown code span wraps across
  it
- **THEN** the whole line is scanned for the coordinate shape rather than paired. A per-line pairing reads the
  prose between one span's closer and the next opener as the document's own backticked text, which is the
  spans this check would have judged. Scanning entire over-reacts in the safe direction — a coordinate outside
  a span on such a line is refused too — and none of those lines carries the shape today
- **PINNED-BY** `no_reference_names_a_line_number`

#### Scenario: A comment names a position rather than a thing

- **WHEN** a comment in any line-comment format references an item by counted offset, bare article, or adverb
- **THEN** the reaction fails, naming the file, the line, and the shape, and says to name the item instead

#### Scenario: A named construct followed by a direction

- **WHEN** a comment names a construct and gives a direction to find it
- **THEN** nothing reacts, because the reference is to a thing

#### Scenario: A specimen of a refused shape

- **WHEN** the check's own directions carry the shapes they refuse, as string literals on executed lines
- **THEN** they are outside the corpus by position, not by an exemption the corpus could also claim

### Requirement: An anchor SHALL name a moment, not a moving reference

The comment lines of every format the declaration classifies as carrying line comments SHALL NOT anchor a
passage to a reference that moves with time. `AGENTS.md`'s table of what earns a place in a doc comment gives
this row its verdict — *neither* an observation source nor provenance, because it names a moving reference and
is stale the moment that reference moves — and says of the same table that this is the one row a sweep can
enumerate. A passage SHALL anchor to the moment instead: a version, a date, a commit; or drop the clause,
since the sentence almost always means the same without it.

**This is the sibling of naming a position, on the other axis.** That requirement refuses a reference that
points *where* by counting; this one refuses a passage that points *when* by counting. Both are decidable
without reading what the sentence means, which is what separates them from the rest of the table — those need
the criterion applied per site, and that is prose judgement this repository has designed, measured three times
and declined.

The recognised phrases SHALL be declared in one place in the reaction and admitted **by instance or by the
rule's own text**, never on the strength of sounding similar: an entry that closes nothing reads as a defence
that was never there. A phrase that can be anchored by what precedes it SHALL stay outside, because deciding it
means reading what the sentence points back to.

**Runs of comment lines SHALL be joined, with the marker stripped first, before matching.** A wrapped comment
splits a phrase across lines, and both halves of that are load-bearing: matching per line misses a wrapped
instance outright, and joining without stripping the marker leaves the marker inside the phrase and matches
nothing either. **The marker's own extensions SHALL go with it** — a doc comment opens with the declared
marker and adds a glyph, so stripping only what the format declaration owns leaves that glyph inside the
joined phrase, which is the same failure by a different character. The offence SHALL name the line the phrase ends on rather than the line its passage
began on — a wrapped file header otherwise reports line 1, and a shell script's `#!` opens the run, so every
offence in one would have named the shebang.

Markdown SHALL be outside this requirement by the same construction the sibling names: it is whole-document
prose rather than a line-comment format. The second ground — that a relative phrase narrates a past state —
holds for the **record** set, `CHANGELOG.md`'s dated sections and `docs/history/`, and SHALL NOT be read as
covering the rest. `BACKLOG.md`, `AGENTS.md` and the specifications carry no dated sections and are read
later by design, so the rule stated for prose generally reaches them and no reaction does. That residue is
declared as a bound below rather than closed, and the reason is measured rather than asserted.

#### Scenario: A relative phrase in non-record Markdown is not observed — a stated bound

- **WHEN** a tracked Markdown document outside the record set writes one of the declared phrases without
  anchoring it
- **THEN** nothing reacts. Extending the sweep to whole-document prose was measured against the tree it would
  judge, and most of what it would report is not an offence: some occurrences are `AGENTS.md`'s own row
  **declaring** the phrases, some are duration rather than pointer — *admitted it for a window*, which
  narrates how long something lasted — some are a generated projection's copy of either, and some are already
  anchored, by a commit or by naming the release. A reader over text separates none of those groups: telling
  a phrase that points at a moving window from one that measures a span is a judgement about the sentence,
  which is the prose instrument `AGENTS.md` records as designed, measured three times and rejected
- **AND** how many fall in each group SHALL NOT be written here, because **this passage is itself in the
  corpus it describes**: a bound about a phrase has to quote the phrase, so stating the breakdown moves it,
  and its two generated projections move it again. The measurement belongs in the tracker's *Observation
  source*, where it is dated by the entry that carries it
- **AND** the rule is wider than its reaction and SHALL stay so rather than being narrowed to fit: a comment
  format is where the reaction can decide, and prose is where a reviewer must
- **UNPINNED** `BACKLOG.md` — *a relative phrase in non-record Markdown*

#### Scenario: A comment anchors to a moving reference

- **WHEN** a comment in any line-comment format carries one of the declared phrases
- **THEN** the reaction fails, naming the file, the line the phrase ends on, and the phrase, and says to anchor
  it to the moment or drop it
- **PINNED-BY** `no_tracked_source_names_a_relative_anchor`

#### Scenario: A phrase wrapped across two comment lines

- **WHEN** a phrase's words fall on either side of a line break in a wrapped comment
- **THEN** the reaction still names it, because the run is joined with its markers stripped before matching

### Requirement: A live document SHALL anchor a citation to something a fresh clone can reach

A live governance document SHALL NOT cite a commit object or a hosting serial. The anchor SHALL be the
release window, which survives because `main` is made of releases, or the change's own name, which is text.

`main` carries one commit per release: a whole development window squashes into a single `release: X.Y.Z`
commit, so no development commit is reachable from it — not eventually, but by construction. A citation to
such a commit in a document that is read *later, against the tree* is therefore dead the moment its window
closes, and a hosting platform's serial was never in the tree at all. `AGENTS.md` already dispositions both
as **provenance** — they name *when*, not *what*, and nothing downstream reads them.
Where a citation is load-bearing as the provenance of a decision it belongs in a **record** — a commit
message, a dated `CHANGELOG.md` section, or `docs/history/` — which is a measurement of its moment and is
read as one.

**This is a reaction where the sibling relative-anchor rule is not, and the difference is that nothing here
is a judgement about a sentence.** A token is shaped like an abbreviated object or it is not; a `#` is
followed by a digit or it is not. Fenced blocks and HTML comment spans are outside it, read through the same
prose reader the rest of this capability uses, because a fence is where a command lives and a command may
carry an object name legitimately.

#### Scenario: A live governance document cites a commit object or a hosting serial

- **WHEN** tracked Markdown outside the record set names an abbreviated commit object in a code span, or a
  `#` immediately followed by a digit
- **THEN** the reaction fails, naming the file, the line and the citation, and says to name the release
  window or move the citation into a record
- **PINNED-BY** `no_live_document_cites_a_moment_a_fresh_clone_cannot_reach`

#### Scenario: An abbreviation carrying no letter, or no digit, is not observed — a stated bound

- **WHEN** a live document cites an abbreviated commit object whose characters are all digits, or all
  letters
- **THEN** nothing reacts. The reader requires **both** a letter and a digit, and the two directions it
  refuses without that requirement are both live in this tree: a specification writes a long run of digits as
  the figure a fabricating reader produced, and English carries words spelled entirely from the hex alphabet
  at this length. Admitting either would refuse a passage that cites nothing
- **AND** the residue is computed rather than estimated: over uniformly random seven-character
  abbreviations it is 3.8%, and the direction is chosen deliberately — this repository's Core Contract
  forbids a false refusal more strictly than it forbids a miss, so the reader gives up that fraction rather
  than refuse prose that names no commit
- **UNPINNED** `BACKLOG.md` — *an abbreviation carrying no letter or no digit*

### Requirement: Deliberate absence does not become a stale-reference finding

The gate SHALL skip a recognized target when Git reports that target ignored, because prose may
deliberately describe an absent generated or local artifact. It SHALL ask Git with directory semantics so
the answer does not depend on whether an ignored directory happens to exist in the checkout.

#### Scenario: Prose names an ignored path

- **WHEN** a recognized untracked reference is covered by the repository's ignore rules
- **THEN** the gate emits no stale-reference finding for that path

### Requirement: Observation failures fail loudly rather than reading as clean

The gate SHALL **fail, naming the observation it could not make**, when it cannot build the tracked-path
index, enumerate the inspected corpus, read an inspected source, or read the deletion history. A failed read
is not an empty result, and reporting one as the other is the vacuity direction the Core Contract forbids.

A Rust test passes or fails, so every observation failure fails loudly and says which read it was. That is the
safe direction: the alternative is a check reporting clean over content it never read.

#### Scenario: The tracked-path index cannot be built

- **WHEN** the Git enumeration that owns every tracked-path answer fails
- **THEN** the gate fails, naming the tracked-path index failure

#### Scenario: Extracted references cannot be normalized

- **WHEN** the normalization pipeline fails for references extracted from an inspected file
- **THEN** the gate fails, naming that file instead of silently examining an empty stream

#### Scenario: An unhandled command fails

- **WHEN** an unwrapped command fails while the gate is running
- **THEN** the gate fails, naming the read it could not make rather than reporting a clean corpus

### Requirement: The gate SHALL be read-only and fail observably

The reference-integrity gate SHALL be read-only. A clean judgment SHALL pass. One or more stale references SHALL
be aggregated into a failing assertion with remediation. An invalid invocation, missing prerequisite, or
observation failure SHALL fail loudly and name what could not be judged. No verdict SHALL alter tracked,
untracked, or commit state in the repository being judged.

#### Scenario: A clean repository passes

- **WHEN** every judged reference resolves or falls within a declared exclusion
- **THEN** the reaction passes without requiring a particular stdout or process-exit vocabulary

#### Scenario: Stale references are an enforced failure

- **WHEN** one or more judged references do not resolve to tracked or deliberately ignored paths
- **THEN** the reaction reports every offence with remediation and fails

#### Scenario: Judging a repository does not mutate it

- **WHEN** the gate evaluates a fixture it has not previously inspected
- **THEN** the fixture's tracked tree, untracked state, and HEAD remain unchanged

### Requirement: A dated record SHALL keep the paths it named then

A **dated** CHANGELOG section — one whose heading carries `] - <date>` — SHALL NOT be held to today's tree.
Its paths are part of a record of what was true when it was written, and requiring them to name what is true
now is the falsification `release-coherence` refuses for the same sections. Measured the hard way: a sweep
that did not know this rewrote eight hunks inside the released `[0.4.0]`, leaving it saying a Rust test
"normalizes a link target with portable shell". Measured again when the exemption was narrowed: those
sections carry eight unresolved paths, and every one is a shell gate that genuinely existed at `0.4.0` and
was deleted when it migrated to Rust.

**The exemption SHALL be exactly this, and SHALL NOT extend to a document because of where it lives.**
`docs/history/` was exempt as a whole directory, and the exemption was declared nowhere — not in this
specification, not as a scenario, not as a bound. Measured, it hid exactly one reference: a present-tense
pointer at a gate that had moved crates inside the `0.5.0` window, in the document the CHANGELOG advertises
to adopters as the provenance authority for verifying published tarballs. Fourteen of that directory's
fifteen path references already resolved. The facts a record must keep are shas, dates, versions and counts,
and none of those is a path — so a record document is judged like any other, and only a dated section within
one is not.

Both directions SHALL be held by one reaction. A reaction asserting only the silence is satisfied by a check
that reads no CHANGELOG at all.

#### Scenario: A stale path inside a dated section

- **WHEN** a dated `## [X.Y.Z] - <date>` section names a path this repository no longer tracks
- **THEN** nothing reacts, because the section records what was true then

#### Scenario: The same path in an undated section

- **WHEN** an undated section — `## [Unreleased]` — names that same path
- **THEN** it reacts, because an undated section is not yet a record and claims what is true now

#### Scenario: A path already wrong when a dated record was written is not observed — a stated bound

- **WHEN** a dated section names a path that resolved to nothing at the moment it was written
- **THEN** nothing reacts, and nothing ever will. The exemption is by section, not by whether the path was
  once right, and distinguishing the two needs the tree as it stood at that date — a per-section historical
  checkout this check does not make and whose cost is not proportionate to a mistyped path in a frozen
  record. The engine owns the narrowing: it is this check's rule that declines to look, not a limit an
  adopter chose
- **PINNED-BY** `a_dated_changelog_section_keeps_its_paths_and_an_undated_one_does_not`

#### Scenario: A Rust identifier named in prose is not resolved — a stated bound

- **WHEN** a doc comment writes a backticked snake_case name in prose rather than as an intra-doc link
- **THEN** nothing reacts, and no reader of text can make it. Such tokens routinely match no declaration
  anywhere in the tree, and the most frequent of those are Rust keywords (`use`, `mod`, `dyn`, `fn`, `impl`),
  attribute names (`cfg_attr`) and std method names (`create_new`, `strip_prefix`, `remove_dir_all`). Telling
  a name that should resolve from one that should not needs type information about a receiver, which
  `inline-symbol-path-confinement` already declares unobserved.
- **AND** the natural alternative was measured and is worse: rewriting such a token as `[`name`]` makes
  `rustdoc -D warnings` the reaction, and it does refuse an unresolvable one — but 8 of 8 sampled candidates,
  selected as *a name declared in the same crate*, were parameters, fields or locals whose link form
  correctly fails to resolve. A rule asking for the link form would be wrong for the majority of prose
  backticks.
- **UNPINNED** `BACKLOG.md` — *a Rust identifier named in prose is resolved by no reaction*
