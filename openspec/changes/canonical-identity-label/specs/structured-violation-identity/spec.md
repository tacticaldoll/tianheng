## ADDED Requirements

### Requirement: An identity-bearing path is labeled canonically, not as the platform renders it

A path that becomes part of a violation's identity SHALL be recorded through one canonical labeling
rule, shared by every dimension that records one, rather than by each site rendering the path as its
own platform and string type happen to.

The rule has two parts, and each closes a defect the other does not:

- **The separator is the label's, not the platform's.** A label SHALL use `/` as its only component
  separator, whatever separator the observing platform uses. A component cannot contain `/` on any
  supported platform, so `/` in a label unambiguously means a component boundary. Separator
  interpretation SHALL be delegated to the platform's own path-component semantics rather than
  performed by substituting characters: on some platforms a backslash is a legal byte *within* a
  component, so substituting it would map two distinct paths onto one label, and on others more than
  one character separates, so substituting one is incomplete.
- **Every byte survives.** A label SHALL preserve information the observed path carried, so two paths
  differing only in bytes that are not valid UTF-8 keep two labels. A lossy rendering that replaces
  undecodable bytes with a replacement character SHALL NOT be used, since a baseline accepting the
  first such path would silently suppress the second's never-accepted violation.

A path whose bytes are not valid UTF-8 SHALL therefore be **judged**, not refused: it has a canonical
label like any other. Refusing it would trade a governed package for nothing, and would leave two
dimensions disagreeing about what the same input is.

#### Scenario: The same commit labels a compilation unit identically on either platform

- **GIVEN** a package whose crate root is at `src/lib.rs` relative to its manifest directory
- **WHEN** the compilation unit's label is recorded on a platform whose path separator is `/`, and
  again on one whose separator is `\`
- **THEN** both record `src/lib.rs`, so a baseline written by either is matched by the other, and the
  entry does not re-fire as new for a contributor on the other platform

#### Scenario: A separator byte that is legal inside a component is not treated as a separator

- **GIVEN** a platform on which a backslash is an ordinary byte within a file name
- **WHEN** a single file literally named `a\b` is labeled, and separately a file `b` inside a directory
  `a` is labeled
- **THEN** the two labels differ, because the labeling asks the platform which characters separate
  rather than substituting a fixed one

#### Scenario: Two paths differing only in undecodable bytes keep two identities

- **WHEN** two observed paths differ only in bytes that are not valid UTF-8
- **THEN** their labels differ, and neither is refused; a baseline accepting one does not suppress the
  other

#### Scenario: A path outside the anchor it is labeled against is refused for that reason alone

- **GIVEN** a labeling that is defined relative to an anchor, and an observed path not under it
- **WHEN** the label cannot be formed
- **THEN** the refusal reports that the path lies outside the anchor, and SHALL NOT be reachable for
  any other cause — in particular not for the path's bytes, which are labelable — so the diagnostic
  an adopter reads names the condition that actually holds
