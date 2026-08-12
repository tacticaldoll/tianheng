## MODIFIED Requirements

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

#### Scenario: A comment names a position rather than a thing

- **WHEN** a comment in any line-comment format references an item by counted offset, bare article, or adverb
- **THEN** the reaction fails, naming the file, the line, and the shape, and says to name the item instead

#### Scenario: A named construct followed by a direction

- **WHEN** a comment names a construct and gives a direction to find it
- **THEN** nothing reacts, because the reference is to a thing

#### Scenario: A specimen of a refused shape

- **WHEN** the check's own directions carry the shapes they refuse, as string literals on executed lines
- **THEN** they are outside the corpus by position, not by an exemption the corpus could also claim
