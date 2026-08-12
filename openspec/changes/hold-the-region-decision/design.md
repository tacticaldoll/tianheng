## Context

`region::Source` classifies a format once and hands back `Executed`, `Header` or `Prose`. Four checks use it;
twelve read raw. The two carrying this window's three defects are among the twelve.

## Goals / Non-Goals

**Goals:**

- The two checks holding the defects take their corpus from the classifier.
- The residual — an absence — is declared rather than implied covered.

**Non-Goals:**

- Migrating all twelve. A check reading a data file, or one whose subject is the commentary, is not this class;
  moving it would be motion rather than repair.
- A reaction refusing the inline marker. Measured and rejected — see below.

## Decisions

**The reaction was designed, measured and rejected, and that is recorded rather than dropped.** The obvious
mechanization is to refuse `starts_with('#')` / `starts_with("//")` inside the checks. Measured against this
repository, the sites carrying that marker split: some are this class, and the rest select commentary on
purpose — a check asserting a doc comment directs its reader somewhere must recognize comment lines — or parse
a data format whose own syntax marks comments. A gate refusing more legitimate sites than defects is how one
earns being turned off, which this repository has said of the prose detector it rejected three times. So the
requirement carries the rule and the classifier carries the adoption, with no scan over the checks themselves.

**The absence is the residual, and it is a bound rather than a gap.** The Definition-of-Done defect is not a
wrong filter — it is *no* filter. Nothing can scan for something never written, so the honest statement is a
declared bound naming what no reaction sees, tracked where debt is tracked.

**An acquisition is recognized past its environment prefix, not by listing prefixes.** The sweep splits on the
command-substitution opener and then tests what follows; leading `NAME=value` tokens are stripped before that
test rather than enumerated. What precedes the tool name is not what the property is about.

**Two scans of one file must agree by construction, not by maintenance.** Both scans in the exit-class check
now read the same `Executed` region, so a future third scan inherits the decision instead of re-making it. That
is the whole difference between a helper and a type, and the reason the helper was replaced.

## Risks / Trade-offs

**YAML is classified with the shell's line-comment rule** → Both use `#` at line start. A `#` inside a quoted
YAML scalar would be treated as a comment; no workflow line in this repository has that shape, and the
alternative is a YAML parser for a comparison over command strings. Recorded here rather than assumed away.

**Ten checks still read raw** → Deliberate. This change moves the two that carry defects; a sweep of the rest
would be motion without a measured pressure, which is what the backlog's promotion discipline exists to refuse.

## Migration Plan

None. No published surface, no version movement.

## Open Questions

- **Should `region` gain a comments-only accessor?** Two checks select commentary by hand today and both are
  legitimate. If a third appears, that is the trigger — named here so the next instance is recognized as one.
