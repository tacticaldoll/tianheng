## ADDED Requirements

### Requirement: A check SHALL take the region it judges from the shared classifier

A repository check deciding a property about **executed** text SHALL obtain its corpus from `kanhe::region`,
which classifies a format once and carries the decision in the type. It SHALL NOT re-decide the region at the
call site by filtering comment markers inline.

The rule exists because the shape has now cost nine defects. Six were recorded when the classifier was written,
all one shape — *the corpus was taken to be the whole blob when the property was about a distinguished part of
it*. Three more were found afterwards in the two checks that never adopted it, and one of those three is two
scans of a single file disagreeing about the same question five lines apart. A helper was the first answer and
reached most callers but not all; the type is the second, and adoption is what this requirement adds.

**An acquisition SHALL be recognized past whatever precedes the tool name.** A sweep testing the text
immediately after a command substitution opens is blind to an environment-prefixed acquisition, which is the
form the central gate invocation takes in both sanctioned wrappers. The tool is what the property is about; the
assignments in front of it are not.

**Selecting comments is not re-deciding a region, and SHALL NOT be read as a violation of this.** A check whose
subject *is* the commentary — that a doc comment directs a reader somewhere — necessarily recognizes comment
lines, and so does a check parsing a data format whose own syntax marks comments. The rule is about a property
over executed text being decided on unclassified text, not about the marker appearing.

#### Scenario: An acquisition prefixed by environment assignments

- **WHEN** a wrapper acquires a value as `var=$(NAME=value tool …)`
- **THEN** the sweep recognizes it as an acquisition of `tool`, because what precedes the tool name is not what
  the property is about

#### Scenario: Two scans of one file disagree about comments

- **WHEN** one scan of a file excludes comment lines and a neighbouring scan of the same file does not
- **THEN** the disagreement is a defect regardless of whether either currently admits a wrong answer, because
  the region is a property of the format and not of the scan

#### Scenario: A check whose subject is the commentary

- **WHEN** a check recognizes comment lines in order to judge what a comment says, or to parse a data format
  whose syntax marks comments
- **THEN** nothing reacts: the region was not re-decided, it is the subject

#### Scenario: A check that should distinguish a region and does not — a stated bound

- **WHEN** a check judges a property over executed text on unclassified text, having written no region decision
  at all
- **THEN** no reaction sees it. An absence is not a shape, and nothing can scan for a filter that was never
  written. A reaction refusing an inline region decision was designed and measured against this repository:
  of the sites carrying the marker, only some are this class — the rest select commentary deliberately or parse
  a data format — so it would refuse more legitimate sites than defects, which is how a gate earns being turned
  off. The classifier's adoption is what narrows this, and the narrowing is not a closure
- **UNPINNED** `BACKLOG.md` — *a check that never wrote a region decision is invisible*
