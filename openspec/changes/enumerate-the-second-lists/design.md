## Context

One list of this shape is already held: `every_gate_running_wrapper_is_named` compares the wrapper constant
against the tracked scripts, and its documentation names the risk. Three more carry no such guard.

## Goals / Non-Goals

**Goals:** each list held against its enumerator, both directions, with the rule stated once.

**Non-Goals:** a general reaction over *every* constant in the crate. Deciding which constants have enumerators
is a judgement, and one made per instance is honest where one made by a matcher would guess.

## Decisions

**Both directions, always.** A one-directional check catches the omission and misses the entry that outlived
its subject. The examples list has exactly the second risk: a directory removed while its entry stays would
read as coverage while defending nothing.

**The arrival matrix enumerates the parser, not the reverse.** The parser is the allowlist — the specification
says so, and `AGENTS.md` was corrected to point at it this window. So the matrix is compared *against what the
parser accepts*, and a flag the parser stops accepting must leave the matrix too.

**A constant with no enumerator states that.** The attribution constant beside `TYPES` already does; `TYPES`
stated nothing while being fully enumerable, which reads as though nothing could hold it.

**Tracked content, not the working directory**, for the example enumeration — the rule every sibling here
follows, so an untracked scratch directory is neither a failure nor an example.

## Risks / Trade-offs

**Parsing a prose sentence for the type list** → `dod_coherence` already parses `AGENTS.md`, so the mechanism is
precedented. The anchor is the backticked run inside the *narrowest honest type* clause; if the anchor is gone
the check must refuse loudly rather than match nothing, since a parse that silently finds zero types would make
the comparison vacuous in the direction that matters.

**Reading the parser's `case` arms** → a second reader of the wrapper's syntax. An arm it cannot parse must
fail loud rather than be skipped, for the same reason.

## Migration Plan

None. All three lists currently agree, so every comparison lands green.

## Open Questions

- **Are there constants beyond these four?** Not swept for. Each was found by review rather than by
  enumeration, which is itself the weaker method — named here so a future sweep is recognized as worth doing
  rather than assumed done.
