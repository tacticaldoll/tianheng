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

#### Scenario: Ambient state names a smaller set

- **WHEN** a required governance document is absent and the process environment names a smaller document set
- **THEN** the reaction still fails, naming the absent required document

### Requirement: Fixture policy narrowing SHALL be explicit and confined

The gate SHALL accept an explicit fixture-only governance-document set when judging a repository other than
Tianheng's own physical workspace. The set SHALL be non-empty. The option SHALL be refused on the real workspace,
and an unreadable or incomplete input SHALL fail, naming what could not be read.

#### Scenario: The zero-corpus fixture narrows its prerequisite set

- **WHEN** the failure matrix explicitly supplies a non-empty fixture set for a throwaway repository
- **THEN** the gate uses it, allowing the later zero-inspected-files refusal to be observed

#### Scenario: Fixture policy targets the real workspace

- **WHEN** fixture-only policy narrowing is requested for Tianheng's own physical workspace
- **THEN** the reaction fails rather than weakening the required set

#### Scenario: Fixture policy is empty, surplus, or an argument is unknown

- **WHEN** the fixture option has no non-empty value or has surplus values, or an unknown argument is supplied
- **THEN** the reaction fails, naming the invalid invocation

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
extension per discovery is the denylist shape this window replaced twice elsewhere. A tracked script's shebang
SHALL NOT be a reference: it names an absolute path outside every prefix this gate recognizes. Before judging references it SHALL require the repository's
governance-document surface, at least one tracked workspace member under `crates/`, and at least one inspected
source; absence of any prerequisite SHALL fail loudly rather than read as clean.

#### Scenario: A complete tracked checkout is inspectable

- **WHEN** the repository contains the required governance documents, a tracked workspace member, and a tracked
  source of any format the declaration classifies as carrying prose
- **THEN** the gate builds its tracked-path evidence and evaluates the corpus without consulting untracked files

#### Scenario: Required evidence is absent

- **WHEN** a required governance document, every tracked workspace member, or every inspectable source is absent
- **THEN** the reaction fails, naming the missing prerequisite instead of reporting clean

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

Tracked Rust and shell comment lines SHALL NOT reference an item by its position — a counted offset, a definite
article naming no thing, or an adverb standing in for one. A reference SHALL name the item: an intra-doc link
where the documentation can reach it, otherwise the identifier or the path. A direction word following a named
construct is a reference to a thing and SHALL NOT react.

**The ladder this sits at the bottom of.** An intra-doc link is checked by the compiler; a path is checked by the
sweep above; a path with a line number is checked by nothing; a position is not even a name. Measured on this
repository, two such references were off by 86 and 98 lines, and the second was written after the first had been
corrected — the criterion `scripts/publish.sh` states for itself, that a rule stated and then missed needs a
check rather than another sentence.

The corpus SHALL be comment lines, by the same rule that decides the sibling sweep's corpus, so a specimen
written as a string literal sits on an executed line and cannot be read as a reference. That is a position rather
than a marker: nothing can hide a comment inside an executed line, and the check's own explanation of the shapes
it refuses would otherwise be the corpus it judges.

Markdown SHALL be outside this requirement. In a record — a `CHANGELOG.md` entry, a `BACKLOG.md` history — a
positional phrase narrates a past state, and separating that from a live reference is a judgement over prose,
which this repository has designed, measured, and declined. In source there is no such reading: a comment
describes the file it is in.

#### Scenario: A comment names a position rather than a thing

- **WHEN** a tracked Rust or shell comment references an item by counted offset, bare article, or adverb
- **THEN** the reaction fails, naming the file, the line, and the shape, and says to name the item instead

#### Scenario: A named construct followed by a direction

- **WHEN** a comment names a construct and gives a direction to find it
- **THEN** nothing reacts, because the reference is to a thing

#### Scenario: A specimen of a refused shape

- **WHEN** the check's own directions carry the shapes they refuse, as string literals on executed lines
- **THEN** they are outside the corpus by position, not by an exemption the corpus could also claim

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
