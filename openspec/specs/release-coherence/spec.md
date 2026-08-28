# Release Coherence Specification

## Purpose

Define the read-only repository reaction that keeps Tianheng's release commit spine, Cargo version
surfaces, lock snapshot, and adopter-facing changelog coherent without time-based release policy.

## Subject

- `CHANGELOG.md`
- `crates/kanhe/tests/release_coherence.rs`
- `crates/kanhe/src/release_coherence_gate.rs`

## Requirements

### Requirement: Repository state determines the release phase

The repository SHALL classify its release phase solely from the latest exact `release: X.Y.Z`
commit in git history, the position of `HEAD`, and the current workspace version. A later commit at
the same version SHALL be development; a strictly newer numeric `X.Y.Z` current version SHALL be
release-ready; and the exact latest release commit SHALL be a release snapshot. A current version
older than the latest release, or missing or malformed release history, SHALL fail as an observable
repository misconfiguration. Classification SHALL NOT depend on branch names, tags, wall-clock
time, warning windows, or hosted-CI-only variables.

#### Scenario: Post-release work is development

- **WHEN** `HEAD` is later than the latest exact release commit and the workspace version is
  unchanged
- **THEN** the repository is checked as active development

#### Scenario: A newer workspace version is release-ready

- **WHEN** `HEAD` is later than the latest exact release commit and the numeric `X.Y.Z` workspace
  version is strictly newer
- **THEN** the repository is checked as release-ready

#### Scenario: A version regression fails loud

- **WHEN** the workspace version is older than the latest exact release commit
- **THEN** the coherence check fails and names the current and latest release versions

#### Scenario: The release commit is a snapshot

- **WHEN** `HEAD` is the latest exact `release: X.Y.Z` commit
- **THEN** the repository is checked as a release snapshot for `X.Y.Z`

#### Scenario: Shallow or absent history fails loud

- **WHEN** no exact release commit is observable in the available git history
- **THEN** the coherence check fails and identifies release history as unavailable

**The workspace version SHALL be read from the manifest's executed region, and a value the reader cannot
read SHALL be a distinct refusal from a key that is absent.** TOML opens a comment wherever a string is not
open, so `[workspace.package] # …` and `version = "X.Y.Z"  # …` both declare a version and cargo accepts
both. Read as raw lines the first closed the table before it opened and the second carried its comment into
the value, so a legal manifest was refused for declaring no version at all — the second spelling being the
one an author reaches for while bumping, which is exactly when this classification runs. The corpus is
therefore the shared TOML region `repository-checks` requires of a check deciding a property over executed
text, and the same reader serves `publish-source-integrity`, so both refusals moved together.

The same holds of the **key**, and it did not: the heading side of that reader decoded while the key side
compared raw text, so spellings cargo accepts answered *absent*. The scenario below records what was
measured. A value the reader cannot read — a single-quoted or literal string, which cargo accepts and this
reader does not take — names a different operator action from a missing key: one is a key to add, the other a spelling
this check has never met. Each refusal SHALL name the value as written, and SHALL say which judgement it
blocked rather than only which fact was unreadable, because the two gates sharing the reader cannot decide
different things.

#### Scenario: A comment on either version surface

- **WHEN** the root manifest carries a comment on the `[workspace.package]` heading, or after the `version`
  value, or both
- **THEN** the version is read as declared and classification proceeds

#### Scenario: A version value the reader cannot read

- **WHEN** `[workspace.package]` declares a version in a form this reader does not take
- **THEN** the check refuses as a cannot-judge, quoting the value as written and naming what it could not
  decide — never as a version that is absent

#### Scenario: A key spelling cargo accepts, and a table cargo writes as a value

- **WHEN** the key is written in a spelling cargo accepts and this reader's heading side already decoded —
  `"version" = "0.5.0"`, `'version' = "0.5.0"`, or the same spellings of `[package]`'s `name`
- **THEN** the key is read and the value judged. Measured under cargo 1.96.0, through a member inheriting
  `version.workspace = true`: each resolves the member at `0.5.0`, and `[package]` with `"name" = "m"` names
  `m`. The heading side decoded and the key side matched raw text, so each answered *the key is absent* — the
  state reserved for a key that is not there — and the gates then said *workspace version is missing or
  malformed*, and *declares no `[package]` name*, about manifests that declare both. **Every reader asking
  whether a line assigns a named key asks one reader** — the workspace version, the package name, and
  publishability — rather than each deciding it over raw text
- **AND** the dependency reader and the lock reader ask a different question and are not that reader's: they
  ask *which* key a line assigns, with the key unknown, where this one is asked about a key it is given.
  A first statement of this requirement said *one reader owns the question for every table body*, which was
  wider than the code by those two; the general form that would unify them is filed with its trigger rather
  than asserted here
- **AND** a **dotted** head naming the sought key assigns a field of it rather than the key —
  `version.workspace = true`, the line every member of this workspace writes — so the reader reports **which
  field**, with every segment of the tail decoded. A reader wanting the key's own value refuses on it, because
  taking `true` as a version would not be visible; a reader asking about that named field compares the field's
  name. A dotted head naming any other key is another key's business, since refusing on those would refuse
  every member manifest in the tree
- **AND** *decoded* is a property of the whole answer, not of its first segment. Three review rounds each moved
  that boundary one segment right and left the next raw: the heading decoded while the key did not, then the
  key while one recogniser compared whole lines, then the head while the tail was joined raw. Measured under
  cargo 1.96.0, the last of those refused `version."workspace" = true`, `version.'workspace' = true`,
  `version."\u0077orkspace" = true` and `version = { "workspace" = true }` — all four inherit — and read
  `xuanji."path" = "xuanji"` as a dependency with no path, which is the **false negative** recorded below
- **AND** where the table is written as a **value** inside its parent — `[workspace]` with
  `package.version = "0.5.0"`, or with `package = { version = "0.5.0" }`, both of which cargo resolves — the
  check refuses naming the line. Composing a table out of a dotted key path or an inline table is structure
  this reader does not build; refusing names what it met, where reporting absence names a declaration nobody
  made
- **AND** a dependency whose classification the reader could not read is **refused, not treated as external**.
  Which crate a dependency names decides whether it is internal; one whose path could not be read names a
  family crate whose source is undecided, and *might be this workspace's* is not an answer. Measured: `xuanji."path" = "xuanji"` beside `xuanji.version = "0.5"` is a path
  dependency at `^0.5` to cargo, and reading the tail raw answered *no path* — so the stale pin left the
  internal check's subject in silence while one correct pin elsewhere satisfied its non-vacuity floor, and the
  release reported clean. That is the aggregate-counter shape the per-example check records having fixed, in its
  sibling
- **PINNED-BY** `a_key_spelling_cargo_accepts_is_read_and_a_table_written_as_a_value_is_refused`
- **PINNED-BY** `a_stale_internal_pin_behind_a_quoted_tail_is_refused`
- **PINNED-BY** `every_inherit_spelling_cargo_honours_is_read_as_inheriting`
- **PINNED-BY** `a_member_whose_package_name_is_quoted_is_read_under_that_name`

### Requirement: Development carries adopter-facing release narrative

Active development SHALL retain the current released workspace version, at least one changelog list
item under `[Unreleased]`, and an `[Unreleased]` comparison link from that version to `HEAD`.
Workspace crate manifests SHALL inherit the common version and internal workspace dependency pins
SHALL equal it. Development SHALL NOT require old generated lock entries to be rewritten solely to
pass this gate. `[Unreleased]` may name the intended release in adopter-facing narrative before mechanical
release preparation advances the workspace version. The reaction SHALL judge the mutable version-bearing
surfaces it enumerates; it SHALL NOT require a version literal in `[Unreleased]` prose to equal the still-released
workspace version.

#### Scenario: Development with release notes is coherent

- **WHEN** post-release commits retain the released version and `[Unreleased]` contains an item and
  the matching comparison link
- **THEN** release coherence passes without requiring a release-prep version or lock rewrite

#### Scenario: A different intended release literal precedes mechanical version preparation

- **WHEN** `[Unreleased]` prose names a future version different from the current released workspace version,
  while workspace and example manifests, internal pins, lock entries, and the comparison link retain that current version
- **THEN** development coherence passes because prose narrative is not one of the enumerated version-bearing surfaces

#### Scenario: Empty development notes fail

- **WHEN** post-release commits exist but `[Unreleased]` contains no list item
- **THEN** the coherence check fails and names the missing adopter-facing release narrative

### Requirement: A release section SHALL be coherent with itself

The reaction SHALL read each release section's **internal** consistency as well as the changelog's **state** —
which version, which sections exist, whether the comparison link is right. Internal consistency is a different
question and was unasked until two defects of that shape landed in one window: an `[Unreleased]` grew a second `### Changed`
heading three hundred lines from the first, and a prose claim about which prior releases carry a `### Migration`
section was wrong under every reading.

A heading SHALL NOT appear twice within one release section. Two blocks of one name split what belongs
together, and a reader of the second never learns the first exists.

A section marking a change `**BREAKING**` SHALL carry a `### Migration` section. The obligation is one-way: a
section MAY carry a migration for a break marked some other way, which this repository's own `[0.3.0]` does.

The vacuity guard SHALL be over **sections**, not headings. A changelog whose sections carry bullets directly
and no `###` sub-headings is an ordinary small changelog — this repository's own early releases are that shape —
so guarding on headings would refuse them; a changelog with no `## [` section at all is the undecidable one.

What this requirement SHALL NOT reach is the **content** of an entry: whether it is accurate, whether "no
adopter action" is true, whether a named symbol exists. Those are judgements over prose, and the detector they
would need is the one `AGENTS.md` records as designed, measured three times and rejected. The line drawn here is
between the document's grammar and its claims, and only the grammar is decidable — which is why the two defects
above were reachable and the sentence about them was not.

#### Scenario: A release section repeats a heading

- **WHEN** one `## [` section carries two `### ` headings of the same name
- **THEN** the reaction fails, naming the section and the heading

#### Scenario: A break is marked with nowhere to read what to do

- **WHEN** a release section contains a `**BREAKING**` marker and no `### Migration` heading
- **THEN** the reaction fails, naming the section

#### Scenario: A break is marked and the migration is there

- **WHEN** the same section carries both
- **THEN** the reaction is clean, so the refusal above is about the missing migration rather than about the
  marker

#### Scenario: A changelog with no release section at all

- **WHEN** the structure read from the changelog holds no `## [` section
- **THEN** the reaction refuses to judge rather than reporting every property of zero sections satisfied

### Requirement: Release-ready and snapshot surfaces agree

A release-ready repository SHALL carry an empty `[Unreleased]` section, a dated changelog section
for the current workspace version, a comparison link for that version, matching internal workspace
dependency pins, and matching `Cargo.lock` entries for every Tianheng workspace package. A release
snapshot SHALL additionally have the exact subject `release: <workspace-version>`. Any divergence
SHALL fail and name the surface and expected version. The check SHALL observe repository state only
and SHALL NOT perform a version bump, commit, merge, tag, or publish action.

#### Scenario: A coherent release candidate passes

- **WHEN** the workspace version is newer than the latest release and every changelog, pin, and
  lock surface names the new version
- **THEN** release coherence passes as release-ready

#### Scenario: An example pins a family crate at neither the workspace version nor its minor series

- **WHEN** a manifest under `examples/` requires a family crate at a version the workspace version neither
  equals nor is the minor series of
- **THEN** the coherence check fails and names the example, the crate, and the version found

#### Scenario: A dependency key this reader cannot decode

- **WHEN** an example declares a family crate under a key that is not a bare TOML key — `"xuanji" = "0.0.1"`,
  which cargo decodes to a dependency named `xuanji` — and no `package` value names the crate explicitly
- **THEN** the check refuses as a cannot-judge naming the key it could not decode, rather than passing the
  entry over. A key whose decoded value is not its text names some crate and this reader cannot say which, so
  it can neither be matched against the family nor skipped: skipping it is what let a stale pin reach a
  release as clean, while the aggregate requirement counter stayed non-zero on the strength of the other
  examples. It is the same false negative as a renamed dependency, through a second door, and the identity
  rule that closed the first — a dependency names the crate its `package` field names, and its key only
  otherwise — is where the key's own spelling has to be judged
- **PINNED-BY** `a_dependency_key_this_reader_cannot_decode_is_refused_rather_than_skipped`

#### Scenario: A dependency table whose heading cargo decodes

- **WHEN** an example declares a family crate inside a table whose heading spells its name with a TOML escape —
  `[target.<triple>."\u0064ependencies"]` or `["dep\u0065ndencies"]`, which cargo decodes and reads as a
  dependencies table — at a version the workspace version does not satisfy, beside an ordinary family
  dependency keeping the aggregate requirement counter non-zero
- **THEN** the check reads that table's entries as pins and fails naming the crate, rather than classifying the
  heading as some other table and passing every entry inside it over
- **AND** the escapes are **decoded** rather than answered as undecidable. Measured against cargo: it reads
  `serde` under both spellings above, so a reader refusing on a backslash left those pins unread while the
  ordinary pin beside them kept the guard satisfied — a silent false negative, where decoding is both what
  cargo does and the answer that needs no third state carried to a heading's consumers. The readers of the
  `[package]` and `[workspace.package]` tables decode by the same rule, which is what stops an escaped
  `publish` key from answering *publishable* for a crate cargo refuses to publish
- **AND** an escape cargo itself **rejects** — `["\q"]`, `["\uD800"]` — leaves which table the heading names
  undecided, and the readers whose answer turns on a table being absent refuse rather than reporting nothing
  declared. That is a file `cargo metadata` fails on, so the refusal stands for a manifest nothing builds from
- **PINNED-BY** `an_escaped_dependency_table_heading_is_read_as_the_table_cargo_reads`

#### Scenario: A dependency is read in either form cargo writes it

- **WHEN** an example declares a family crate as a detailed table — `[dependencies.alias]` with its own
  `package` and `version` lines, or `[dependencies.xuanji]` with a `version` line — at a version the
  workspace version does not satisfy
- **THEN** the check fails naming the crate. A declaration's form is cargo's to choose; a reader keyed on
  `<crate> = …` entries saw no family crate on any line of such a table and skipped the whole declaration,
  renamed or not
- **PINNED-BY** `a_detailed_dependency_table_is_read_renamed_or_not`

#### Scenario: Both pin readers read dependencies the same way

- **WHEN** the root manifest writes an internal pin as a detailed table — `[workspace.dependencies.xuanji]`
  with `path` and `version` on their own lines
- **THEN** the check reads it and passes, and refuses a stale one in that same form. Which dependencies a
  manifest declares SHALL have one reader: the example-pin check was migrated to it while the internal-pin
  check kept a line-oriented scan, and the two then disagreed over a manifest cargo reads correctly — the
  scan selected any line carrying `path`, `"crates/` and `=`, so it took `path` for the dependency's name
  and never read the `version` line, refusing with *internal dependency path has no version pin*. Which
  dependencies are internal is read from each one's own **identity** — the crate it names — rather than from
  the shape of a line or from where it points
- **PINNED-BY** `an_internal_pin_written_as_a_detailed_table_is_read`
- **PINNED-BY** `a_stale_internal_pin_in_a_detailed_table_is_a_violation`

#### Scenario: A family crate the catalog offers from anywhere but this workspace

- **WHEN** the root manifest offers a crate the workspace itself declares — `xuanji = "0.4.0"`, a `git`
  source, or a `path` that is not the member's own directory — rather than by a path to that member
- **THEN** the check fails, naming the crate it could not hold and where that member actually is. The subject
  SHALL be selected by the crate a dependency **names**, not by where it points: measured under cargo 1.96.0,
  a catalog entry `xuanji = "0.4.0"` beside a local member `xuanji 0.9.0` gives the inheriting member
  `registry+…#xuanji@0.4.0` with the member unused, and `cargo package` on a `git` dependency carrying a
  `version` drops the source and records the version alone. Either way the published requirement is that
  version, so a stale family requirement reached `cargo publish` through a line a path-selected subject never
  contained, and deleting one `path = …` is the whole of the edit that gets there
- **AND** the path SHALL be compared against **that member's own directory**, not against a prefix, and both
  sides SHALL be read through `std::path::Component` so they share one representation: a `.` component, a
  repeated separator and a trailing separator are dropped, because cargo resolves `crates/xuanji`,
  `./crates/xuanji`, `crates//xuanji` and `crates/xuanji/` to one directory, measured. A prefix decided
  neither direction correctly — `./crates/xuanji` names the member and was refused, `crates/../vendor/xuanji`
  resolves to `vendor/xuanji` and passed — and the member's directory cannot be derived from its package name,
  which this repository's own fixture holds apart. Splitting one side on `/` while the other came from a
  `Path` agrees only where the platform's separator is `/`, so the comparison SHALL NOT be made on text
- **AND** a path this reader will **not** name a directory for is a cannot-judge rather than a collapsed
  guess, and the refusal SHALL say **which** of the reasons it met rather than enumerating the others. There
  are three: a **rooted** path, including a drive prefix, which this reader cannot make relative because it is
  handed no repository; a **`..` segment**, applied after symlink resolution, so `crates/../vendor` is
  `vendor` only while `crates` is not a link and this reader touches no filesystem to find out; and a value
  whose components are all `.` or separators, which **names no directory** beneath the manifest's own —
  measured under cargo 1.96.0, `path = "."` fails resolution outright. Enumerating two of the three sent an
  operator reading the third's refusal to look for a `..` that was not there
- **AND** the drive-prefix case is a **coverage limitation and not a bound**: the classifier reacts to it, and
  correctly — a drive prefix is rooted — so nothing is declined and the bound register has nothing to hold.
  What is absent is a run: the component is produced only on Windows and this repository's CI is Ubuntu. It
  shares its answer with the rooted case, which is exercised
- **PINNED-BY** `a_family_crate_offered_with_no_path_is_a_violation`
- **PINNED-BY** `a_family_crate_path_is_compared_against_the_members_own_directory`
- **PINNED-BY** `a_path_value_is_read_through_its_components`

#### Scenario: A case alias of a member directory — a stated bound

- **WHEN** the catalog offers a family crate through a path whose components differ from the member's only in
  case — `CRATES/TIANHENG` for `crates/tianheng` — on a **case-insensitive** filesystem, where cargo resolves
  the two to one directory
- **THEN** the check reports a violation, naming a directory that is the member's. The comparison is
  component-wise and case-sensitive on every host, so the answer is the same everywhere and is only *right*
  where the filesystem is case-sensitive. On this repository's CI it is right: the path names a directory that
  does not exist. On a case-insensitive host it over-reacts
- **AND** the reach is kept deliberately. Closing it means asking the filesystem, because case folding is the
  volume's rule and not the string's — and a release gate whose verdict over one tree differs by the machine
  it runs on is worse than a refusal an author can read and argue with. This reader is also handed no
  repository to ask
- **AND** an earlier wording of this clause claimed that canonicalizing would make `..` resolvable and move
  three other verdicts with it. Review showed otherwise: canonicalization can be confined to the branch that
  already produced a directory, after the rooted, traversal and no-directory refusals have been returned, so
  every one of them survives. That reason was false and is not what keeps the bound
- **AND** the direction that observes this shape runs where the refusal is **correct**, so it demonstrates a
  real violation rather than the over-reaction declared here — which is why this bound is unpinned. It was
  declared pinned for one round, and that was the second instance of the tracked class below
- **UNPINNED** `BACKLOG.md` — *a pin may defend a direction its bound does not declare*

#### Scenario: An internal dependency this reader cannot resolve is not one it may skip

- **WHEN** the root manifest declares an internal dependency whose `path` or `version` is not a
  double-quoted string, or declares more than one of either key
- **THEN** the check refuses as a cannot-judge saying which key and which of the two it met. An unreadable
  **path** is the one that cannot be answered by skipping the entry, because the entry names a family crate
  and whether members inherit *this workspace's* copy of it is what could not be read
- **PINNED-BY** `a_path_or_a_version_this_reader_cannot_read_is_a_cannot_judge`
- **PINNED-BY** `several_paths_or_several_versions_in_one_dependency_are_not_chosen_between`

#### Scenario: A key outside a dependency table is not a dependency

- **WHEN** a manifest carries a key spelled after a family crate in a table that declares no dependencies —
  `[features]`, whose values are arrays
- **THEN** the check reads it as nothing. Which tables hold dependencies is read from the heading, so the
  same repair that admits the detailed form closes the direction where a non-dependency was read as one
- **PINNED-BY** `a_feature_named_after_a_family_crate_is_not_a_pin`

#### Scenario: A dependency under a cfg target is read, whatever the expression contains

- **WHEN** a family dependency is declared under `[target.'cfg(…)'.dependencies]` or its `.NAME` form, at a
  version the workspace version does not satisfy — including a cfg expression carrying a **dot**, in either
  spelling cargo accepts
- **THEN** the check reads the pin and fails naming the crate. Measured: cargo reads the dependency under
  `[target.'cfg(target_os = "l.x")'.dependencies]` and under the basic-quoted spelling of the same expression,
  reporting it with that target
- **AND** this was a declared bound and is retired rather than reworded. It said such a pin went unobserved
  because stepping past the target context split the heading at its first dot, landing inside the expression —
  true of a reader holding the heading as one dotted string, and not of one holding the keys as segments, where
  the expression is a single key whatever it contains. The bound's own WHEN was written into the tree after the
  change: it reacts, and with the quote-aware cut removed it returns *ok release coherence* exactly as the
  bound described
- **PINNED-BY** `a_pin_under_a_cfg_target_carrying_a_dot_is_read`

#### Scenario: A renamed family dependency is resolved by the package it names

- **WHEN** an example requires a family crate under another key — `alias = { package = "xuanji", version =
  "0.0.1" }`, the rename form cargo admits — and that version does not satisfy the workspace version
- **THEN** the check fails naming the package **and** the key it was given, rather than passing over an
  entry whose key matches no family crate. Which crate a dependency names is its `package` field where it
  has one, and its key only otherwise
- **PINNED-BY** `a_renamed_family_dependency_is_resolved_by_its_package_field`

#### Scenario: A dependency identity the reader cannot read is not one it can

- **WHEN** an example declares a dependency whose `package` value is not a double-quoted string, or declares
  more than one `package` key for one dependency
- **THEN** the check refuses as a cannot-judge saying which of the two it met. Which crate a dependency
  names is `Named`, `Unreadable` or `Several` — the distinction its sibling `version` field already carried,
  where an identity held as a string with the empty string for both failures reported *several* as
  *unreadable*, and read a literal `package = ""` as both
- **PINNED-BY** `an_example_whose_package_value_is_unreadable_is_not_judged`
- **PINNED-BY** `an_example_declaring_several_package_keys_is_not_judged`

#### Scenario: An example declaring no family requirement is refused on its own, not counted against its siblings

- **WHEN** one manifest under `examples/` declares no family dependency this reader could see, while other
  examples declare theirs correctly
- **THEN** the check refuses as a cannot-judge **naming that example**, rather than passing it over because
  the other examples supplied requirements. The requirement count SHALL be per example: an aggregate over
  every example cannot distinguish *no example declares one* from *this example went unexamined while its
  siblings parsed*, and the second is the shape a reader that misses a declaration produces. Both closed
  identity doors — a renamed key, and a key this reader cannot decode — reached a clean release through this
  counter, so the two are one requirement and the count is where it is enforced
- **PINNED-BY** `an_example_requiring_no_family_crate_reports_over_nothing`
- **PINNED-BY** `an_example_declaring_nothing_is_refused_though_its_sibling_is_fine`

#### Scenario: A workspace table is not a dependency of the package whose manifest carries it

- **WHEN** a manifest under `examples/` carries `[workspace.dependencies]`, `[workspace.dev-dependencies]` or
  `[workspace.target.<selector>.dependencies]` naming a family crate, at a version the workspace version does
  not satisfy
- **THEN** nothing in that table is read as a dependency of that example: no pin is refused over it, and it
  counts toward no example's family-requirement. Measured against cargo: a package declaring
  `[workspace.dependencies] xuanji = "0.5"` beside `[dependencies] serde_json = "1"` reports exactly one
  dependency and it is not `xuanji`; a member inheriting from either of the other two fails to load, because
  inheritance reads `[workspace.dependencies]` alone
- **AND** an example whose *only* family mention is such a table therefore declares no family requirement, and
  is refused by the scenario above rather than counted as satisfying it. One reader answered both this check
  and the root's internal-pin check with one unqualified list, and the catalog belongs only to the second: a
  table offering a version to members is not the package requiring it
- **AND** the root's own use of the catalog is unaffected — a version this manifest **pins**, required here or
  offered to members, is still read where the subject is the repository's own pins
- **PINNED-BY** `a_workspace_table_is_not_a_dependency_of_the_package_carrying_it`

#### Scenario: A value that is not a string does not borrow the next one

- **WHEN** a dependency's `package` or `version` value is not a double-quoted string while a later key on
  the same line is — `alias = { package = xuanji, version = "0.2.0" }`
- **THEN** the value reads as unreadable. The quote SHALL open the value; taking the first pair of quotes
  anywhere in the text answered one key with the next key's string, which made the unreadable state
  reachable only when nothing else on the line was quoted
- **PINNED-BY** `a_value_that_is_not_a_string_does_not_borrow_the_next_one`

#### Scenario: An example requires a family crate with no version at all

- **WHEN** an example declares a family crate with no `version` key — the legal path-only or git-only form —
  or with more than one
- **THEN** the check refuses: a violation for the absent pin, because nothing holds it to the workspace
  version, and a cannot-judge for several, because which one is required is not this reader's to choose
- **AND** a dependency declaring `workspace = true` is **not** this case: it declares no `version` of its own
  and is held to the one the catalog offers, which the scenario below reads
- **PINNED-BY** `an_example_requiring_a_family_crate_with_no_version_is_refused`
- **PINNED-BY** `an_example_declaring_several_version_keys_is_not_judged`

#### Scenario: An example taking the offer in its own catalog is held to what the catalog offers

- **WHEN** an example declares a family crate with `workspace = true` — inline, as a dotted key, or as a
  `workspace = true` line in a detailed table — beside a `[workspace.dependencies]` entry naming that crate
- **THEN** the requirement judged is the **catalog's**, not absent. Every example in this repository is its own
  workspace root, so the catalog is in the same manifest; measured, cargo resolves all three spellings to the
  catalog's requirement, and resolves to it even when a local `version` sits in the same inline table — so the
  catalog is the answer rather than one of two. A stale catalog is therefore a stale pin, and a catalog at the
  workspace version or its minor series passes
- **AND** where the catalog beside it names no such crate, or names it through an identity this reader cannot
  resolve, or offers a requirement that itself takes the offer, the check refuses as a cannot-judge saying
  which of the three it met. Each is a manifest `cargo metadata` refuses to parse, and a refusal that stops in
  front of an operator is the answer for a file nothing builds — never a pin read past
- **AND** the root's own catalog is a different subject: the manifest that *declares* the catalog cannot
  inherit from it, so a pin there taking the offer is refused as undecidable rather than reported absent
- **PINNED-BY** `an_example_inheriting_from_its_own_catalog_is_held_to_the_catalog_version`
- **PINNED-BY** `an_example_inheriting_what_no_catalog_offers_is_not_judged`
- **PINNED-BY** `a_catalog_entry_whose_identity_is_unresolvable_stops_the_inheriting_example`
- **PINNED-BY** `a_catalog_entry_that_itself_inherits_is_named_rather_than_followed`
- **PINNED-BY** `an_internal_pin_taking_the_workspace_offer_is_refused`

#### Scenario: Two sections claim one version

- **WHEN** a release-ready repository carries more than one `## [X.Y.Z]` section for the version being
  released, whichever suffixes they carry — two dates, the same date twice, a suffix that is not a date, or
  none at all
- **THEN** the check refuses to judge, naming every claiming heading, because which one records the release
  is not this reader's to choose. The count SHALL be taken over the sections that claim the version and not
  over the ones whose suffix already parsed: a malformed sibling above a well-formed one is the likelier
  mistake, and counting after the date filter left exactly one candidate and reported clean
- **PINNED-BY** `two_dated_sections_for_one_version_are_not_judged`

#### Scenario: A dated heading whose suffix is not a date

- **WHEN** a release-ready repository carries `## [X.Y.Z] - ` followed by ten characters that are not an
  ISO date
- **THEN** the check fails as missing dated release notes, because the suffix is parsed as three
  `-`-separated all-digit fields of widths four, two and two, **each ranged** — a month in `1..=12`, a day
  in `1..=31`. A digit test is a parse without a date's guarantee exactly as a length test is a parse
  without a digit's, so `2026-99-99` is refused alongside `notadate!!`. Whether a day exists in its month
  needs a calendar, which this crate's declared dependency surface does not carry, and that residue is a
  date written wrong rather than a shape that reads as one
- **PINNED-BY** `a_dated_heading_whose_suffix_is_not_a_date_is_a_violation`
- **PINNED-BY** `a_dated_heading_whose_fields_are_out_of_range_is_a_violation`
- **PINNED-BY** `a_dated_suffix_is_a_date_and_not_only_three_digit_runs`

#### Scenario: A dependency whose own name carries the word `version`

- **WHEN** an internal path dependency's name or path contains `version` and the entry is correctly pinned
- **THEN** the check reads the pin and passes, because a `version` assignment is recognised as a table key —
  preceded by a delimiter, followed by `=` — rather than as the first occurrence of the word on the line
- **PINNED-BY** `a_member_whose_name_carries_version_still_reads_its_pin`

#### Scenario: A stale lock entry fails release readiness

- **WHEN** any Tianheng workspace package lock entry names a version other than the release-ready
  workspace version
- **THEN** the coherence check fails and names that package and expected version

#### Scenario: A mismatched release subject fails the snapshot

- **WHEN** `HEAD` is an exact release commit whose subject version differs from the workspace
  version
- **THEN** the coherence check fails and names both versions

#### Scenario: The check performs no release action

- **WHEN** release coherence is evaluated in any phase
- **THEN** repository files, commits, tags, packages, and external release state remain unchanged

### Requirement: Adopter narrative SHALL NOT name this repository's own machinery

An entry under an adopter-facing heading of a section **still being written** SHALL NOT name this
repository's own machinery in any of three forms: a path under `scripts/`, a bare basename that
`git ls-files scripts/` resolves to a tracked file there, or a **directory** under `scripts/` written with
its trailing slash. All three are derived from the one enumeration — the directories by stripping components
from each enumerated path — so none is a list written beside it.

**Still being written** is `[Unreleased]` always, and — in release-ready and snapshot state — the section
dated for the current workspace version. Release preparation dates that section and then keeps writing into
it, so it is prose under review rather than record; in development state the workspace version equals the
latest released one, so the section carrying it is genuinely record and stays exempt. The state decides it
and not a version comparison: a rule phrased as *versions strictly below the workspace version stay exempt*
would refuse the development case the exemption exists for.

`CHANGELOG.md` is the adopter's document. It carries nine kinds of heading — `### Added`, `### Changed`,
`### Fixed`, `### Migration`, `### Documentation`, `### Removed`, `### Compatibility`,
`### Compatibility evidence` and `### Self-governance` — and every one of the first eight is an adopter's vocabulary. It offered none that was
not, so every
change to this repository's own governance machinery has been written into one of them. Measured **before this rule existed**, in the window that introduced it: twenty entries named it, spread
across four adopter headings, for a directory that ships in **zero** packages. That figure is a record of a
past state rather than a census — no reaction produces it now, the section it counted has been collapsed, and
holding a record to today's enumeration would demand the record change every time the tree does.

The `[Unreleased]` section SHALL be permitted a `### Self-governance` heading,
under which naming that machinery is what belongs; a heading is adopter-facing when it is any `### `
heading **other than** that one, so a heading nobody anticipated is adopter-facing rather than exempt.

The basename form SHALL be decided against the enumerator and never against a list of gate names written
beside it. A hand-kept list lets a new script be added and never measured, which is the register's own
prohibition rather than a stylistic call. For the same reason no count of that enumeration SHALL be written
into the reaction: a census is produced, never typed, and the first draft of this reaction carried one that
the commit adding it made stale.

A name SHALL be recognised as a **word** — a maximal run of path characters, required to equal a tracked
path, basename or derived directory. That is exact matching of a lexical token rather than substring
matching: a sentence
merely containing the characters cannot match, because the run is delimited by the first character a path
cannot hold. The rule was first written to compare whole **backticked spans**, and adversarial review
reproduced three false negatives against that reading, every one a shape this repository's own changelog
already uses — a span carrying anything besides the bare path, a double-backtick span, and an inline span
wrapped across a source line. Reading words closes all three and reaches a markdown link target the span
reading never could.

This sits on the **decidable** side of the line this capability already draws for itself: a path citation
is a reference, and reference resolution over `CHANGELOG.md` is already mechanical. Whether an entry's
*subject* is adopter-facing is a judgement over prose — the instrument `AGENTS.md` records as designed,
measured three times and rejected — and is declared as a bound below rather than approximated.

What the rule forces is a **rewrite**, not a move, wherever the adopter-relevant fact is genuinely
present. A publish-provenance entry states the guarantee an adopter gets; naming the gate file that
enforces it is the leak. If a fact matters to an adopter, state the fact.

#### Scenario: An adopter heading names a gate

- **WHEN** an entry under `### Fixed` in `[Unreleased]` names a path under `scripts/`
- **THEN** the reaction fails, naming the section, the heading and the path

#### Scenario: The same entry under the self-governance heading

- **WHEN** that entry moves under `### Self-governance` in the same section
- **THEN** the reaction is clean, so the refusal above is about the heading it sat under rather than
  about the path being named at all

#### Scenario: A bare basename the enumerator resolves

- **WHEN** an entry under an adopter-facing heading names `publish.sh` with no directory
- **THEN** the reaction fails, because the machinery enumeration resolves that basename to a tracked file

#### Scenario: A gate named inside a longer span, or as unquoted prose

- **WHEN** an entry under an adopter-facing heading writes `` `bash crates/kanhe/tests/pin_bites.rs --fix` ``, or
  `` `` `crates/kanhe/tests/pin_bites.rs` `` ``, or a span wrapped across a source line, or a markdown link whose
  target is the gate, or the bare name with no backticks at all
- **THEN** the reaction fails in every one of those, because the word is the unit rather than the span

The machinery set SHALL be **produced from the workspace manifests**: every tracked path under a member the
workspace does not publish, plus `scripts/`. It SHALL NOT be a location. The set was `git ls-files scripts/`,
which was right while the machinery *was* fourteen shell gates and stopped being right in the window that
deleted them and moved it into unpublished crates — leaving `scripts/` naming two wrappers, and this
requirement's own scenario naming a path the enumeration could not resolve. `publish = false` is the same
criterion the refusal states, read from the build rather than from a path.

A **basename**, and an ancestor directory the enumeration derives, SHALL enter the set only where it names
machinery alone. Measured when the corpus widened: 78 machinery paths against 182 published ones, with five
basenames on both sides — and `crates/`, an ancestor of machinery and of every published crate, which made
the first run of the widened corpus refuse this repository's own changelog. A full path is unambiguous and
always enters; a convenience has to earn its place.

#### Scenario: A bare basename the enumerator does not resolve

- **WHEN** an entry under an adopter-facing heading names `check_something_that_does_not_exist.sh`
- **THEN** the reaction is clean, so the rule is held to the enumerator rather than to the `check_`
  prefix

#### Scenario: Prose about the marker is read as a marker — a stated bound

- **WHEN** a release section discusses `**BREAKING**` without marking anything — an entry saying a change
  *earns no* such mark, or one describing how the marking rule works
- **THEN** the section is classified as breaking and required to carry a `### Migration` it does not owe. The
  reaction reads the marker's presence rather than its position, and that reach is kept deliberately:
  over-reaction is the safe direction, while a positional matcher would stop observing a real break whose marker
  sits anywhere but an entry's first token — buying a false negative in the floor to remove a refusal an author
  can argue with. The Core Contract forbids exactly one bug and it is the false negative
- **PINNED-BY** `prose_about_the_marker_is_read_as_a_marker_a_stated_bound`

#### Scenario: A dated release section names a gate — a stated bound

- **WHEN** a dated `## [X.Y.Z] - DATE` section for an **already released** version — one this repository is
  no longer writing into — carries an entry naming a path under `scripts/`
- **THEN** nothing reacts, and the leak is real: an adopter reading `[0.4.0]` meets nine entries naming
  files they can never run. What is refused is the **repair**, not the diagnosis — rewriting a dated section
  to satisfy a rule written afterwards would falsify the record, the same reason `docs/history/` is left
  alone — so this is a declared false negative with an owner rather than a shape that is harmless. Closing it
  needs a repair that adds to the record instead of editing it.

  **The WHEN was wider than the reason, and the gap was live.** It read *a dated section*, so the section the
  current release is still being written into was exempt too — where no record exists to falsify, and where
  release-ready state requires `[Unreleased]` to be empty, leaving the check with no subject at all during
  preparation. Narrowed to what the reason actually covers
- **PINNED-BY** `a_dated_section_naming_a_gate_is_a_stated_bound`

#### Scenario: Machinery the judged repository tracks by nothing — a stated bound

- **WHEN** an entry under an adopter-facing heading names a file under `scripts/` that exists in the
  worktree and in no commit
- **THEN** nothing reacts. The enumeration is `git ls-files scripts/`, so an untracked `scripts/` reads
  as absent; closing this means judging worktree content, which this repository's gates are held not to
  do — the larger error, so the blindness is declared instead
- **PINNED-BY** `machinery_tracked_by_nothing_is_a_stated_bound`

#### Scenario: An entry about self-governance that names no machinery — a stated bound

- **WHEN** an entry under an adopter-facing heading describes this repository's own governance without
  naming any path under `scripts/`
- **THEN** nothing reacts. Reaching it needs a judgement over the entry's subject rather than over its
  references, and that instrument is the one this repository measured three times and rejected;
  widening the matcher toward it — heading keywords, phrase lists — would trade a declared, bounded
  blindness for an undeclared false-positive surface
- **UNPINNED** `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*

#### Scenario: The enumeration cannot be read

- **WHEN** `git ls-files scripts/` fails rather than returning nothing
- **THEN** the reaction refuses to judge, because a failed read is not an empty result and treating it
  as one reports a verdict over content that was never read

#### Scenario: A repository tracking no machinery at all

- **WHEN** the enumeration succeeds and names no file, and an entry under an adopter-facing heading
  names a path under `scripts/`
- **THEN** the reaction is clean, because a repository tracking no machinery has nothing an entry could
  leak — and it SHALL reach that verdict by having nothing to match. Keying the parser on the record
  number rather than on the input file makes an empty enumeration consume the changelog itself, after
  which the section vacuity guard refuses a document the reaction never read

#### Scenario: The members the machinery set is drawn from contribute nothing

- **WHEN** the workspace declares unpublished members and **none of them** contributes a tracked file, while a
  published member does
- **THEN** the check refuses to judge, naming the members that were expected to contribute. The floor SHALL
  count what the machinery set is **built from** — the unpublished members — and not every member's tracked
  paths: the set is drawn from the unpublished ones plus `scripts/`, so one tracked file under any published
  crate kept a wider counter non-zero while the set was `scripts/` alone, and the check then ran against
  `scripts/` and reported clean over a nearly-empty subject. This is the third guard in this reaction written
  over a wider set than the one it protects
- **AND** a workspace declaring **no** unpublished members legitimately has `scripts/` alone as its machinery,
  so the condition is *declared and contributed nothing* rather than *the set is empty*
- **AND** enumerating **nothing at all**, across every member, is a different fact and carries its own
  refusal: cargo and git are then describing different trees, which is not a statement about this subject's
  size
- **PINNED-BY** `unpublished_members_contributing_nothing_is_refused`
- **PINNED-BY** `a_workspace_whose_members_are_untracked_reports_over_nothing`

#### Scenario: A basename an entry writes for another reason — a stated bound

- **WHEN** an adopter-facing entry names a file of its own whose basename the judged repository also tracks
  under `scripts/`
- **THEN** the reaction **fails**, refusing an innocent entry. The direction is the safe one — an author
  meets a refusal to argue with — and narrowing it means deciding which of two files a bare name meant,
  a judgement about the sentence rather than about the reference
- **PINNED-BY** `a_colliding_basename_is_a_stated_bound`

#### Scenario: A name reached only through a URL — a stated bound

- **WHEN** an adopter-facing entry names machinery only inside a URL
- **THEN** nothing reacts. A word is a maximal run of path characters, so a scheme and host fuse with the
  path into one run that equals no tracked name; splitting a URL into its path would make the reaction judge
  a foreign host's layout as though it were this repository's
- **PINNED-BY** `a_name_reached_only_through_a_url_is_a_stated_bound`

#### Scenario: A heading inside a fenced code block

- **WHEN** a `### ` line sits inside a fenced code block, followed by entries that name machinery
- **THEN** the reaction reports those entries, because a fenced line sets no heading in force and cannot name
  the one exempt heading. This was a declared bound whose stated cost was a second, stateful reading of a
  document read once; the reaction reads a prose region now, whose fence tracking is shared with every other
  reader in the crate, so that reading is one that already existed
- **PINNED-BY** `a_heading_inside_a_fenced_block_does_not_reattribute_a_later_entry`

#### Scenario: The directory itself, and a derived ancestor

- **WHEN** an entry under an adopter-facing heading names `` `scripts/` ``, or `` a shared shell library `` where the
  judged repository tracks a file two levels below it
- **THEN** the reaction fails in both, because every ancestor directory is derived from the enumeration by
  stripping one component at a time

#### Scenario: A directory named without its trailing slash — a stated bound

- **WHEN** an adopter-facing entry names `scripts` or a shared shell library with no trailing slash
- **THEN** nothing reacts. Directories are derived slash-terminated, and the unslashed form is a word
  indistinguishable from ordinary prose — `scripts` is an English plural this document already uses as one.
  Admitting it for deeper names only would make the reaction judge which of its own keys read as English
- **PINNED-BY** `a_directory_named_without_its_slash_is_a_stated_bound`

#### Scenario: The enumeration fails rather than returning nothing

- **WHEN** `git ls-files scripts/` exits non-zero while an adopter-facing entry names a gate
- **THEN** the reaction refuses to judge. This is the direction whose absence is a false **negative** rather
  than a downgrade: with the refusal replaced by a plain redirect, the parser reads the empty capture cleanly
  and reports a document naming a gate as coherent

### Requirement: An enumeration SHALL NOT pass over content it failed to read

Every enumeration this judgement makes SHALL distinguish **absent** from **unreadable**, and SHALL refuse as a
cannot-judge on the second. Skipping is reserved for what genuinely is not there.

A directory entry that fails to yield SHALL be propagated rather than dropped, and a manifest that exists and
cannot be read SHALL refuse rather than be skipped. Collapsing the two lets the remaining readable members
satisfy the counters, so the run reports clean over the very file it could not read — and the counters are what
the judgement then reasons from.

Where propagating produces a refusal no fixture can construct, it SHALL be declared out of reach with a slug of
its own. A slug shared between two sites excuses whichever one was looked at.

#### Scenario: A manifest exists and cannot be read

- **WHEN** an example manifest is present but is not readable as text
- **THEN** the judgement refuses as a cannot-judge naming the path, rather than skipping it as though the
  directory held no manifest

#### Scenario: A directory holds no manifest at all

- **WHEN** a directory under the enumerated root has no `Cargo.toml`
- **THEN** it is skipped, because absence is not a failed read

#### Scenario: A directory entry cannot be yielded

- **WHEN** iterating an enumerated directory fails part-way
- **THEN** the judgement refuses rather than continuing over the entries it did receive

#### Scenario: A value cargo decodes and this reader does not

The scenarios above reach a **file** that cannot be read. This reaches a **value**, which is where the
requirement was not being met: a quoted value carrying a TOML escape was answered as one this reader could
read, and the undecoded source then decided a comparison.

- **WHEN** a `path`, a `package` or a `version` is written as a TOML basic string carrying an escape — legal
  TOML, which cargo decodes — and an ordinary sibling entry keeps the vacuity counters non-zero
- **THEN** the judgement refuses as a cannot-judge, rather than comparing the undecoded source, failing to
  match a `crates/` prefix or a family crate, and passing the entry over with its stale pin unchecked
- **AND** refusing is a choice between two answers this reader can now give, not the only one it has. A table
  **heading** carrying the same escape is decoded, because `manifest::decoded` exists — the reason first
  written for refusing values was *no decoder, and hand-rolling a TOML grammar is a filed backlog entry*, and
  half of that expired the moment a heading needed one. What separates them: a key decides *which table or
  which key this is*, so misreading one drops a whole table's contents with nothing said, while a value is the
  thing being judged and refusing it stops the judgement in front of an operator with nothing skipped
- **PINNED-BY** `an_escaped_path_is_refused_and_an_ordinary_sibling_does_not_cover_for_it`

#### Scenario: An escaped renamed package

- **WHEN** a renamed dependency's `package` carries an escape, beside an ordinary family dependency **in the
  same example manifest** — which is the configuration the guards cannot see, because `requirements_here` is
  counted per example and an escaped entry alone in its own example leaves that counter at zero
- **THEN** the judgement refuses, rather than reading the entry as naming no family crate at all
- **PINNED-BY** `an_escaped_renamed_package_is_refused_and_an_ordinary_sibling_does_not_cover_for_it`

### Requirement: A release section is dated on the day its release commit was made

At the release snapshot the dated section for the workspace version SHALL carry the date of the
`release: X.Y.Z` commit itself. The check SHALL compare the value, not only the shape: a reader takes that
date for the day the release happened, and `is_iso_date` answers whether the field is a calendar date and
never which one.

It SHALL hold **only at the snapshot**. Before the release commit exists there is nothing to date against,
and a date written during preparation is an intent rather than a claim — so the check stays silent through
development and release-ready and speaks at the one commit whose date is the answer.

Three releases carried a section date equal to their release commit's date because a person remembered; a
fourth was prepared with a date four days behind the day it would be cut on, and nothing said so.

#### Scenario: The dated section disagrees with its release commit

- **WHEN** the workspace is at the release snapshot and the dated section for its version carries a date
  other than the release commit's
- **THEN** release coherence fails naming both dates, so an operator can see which to change
- **PINNED-BY** `a_release_section_dated_away_from_its_commit_is_a_violation`
