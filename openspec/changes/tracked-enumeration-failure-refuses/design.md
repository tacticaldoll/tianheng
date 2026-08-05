## Context

`tracked_files` wraps the one `git ls-files -z` this gate uses, and four call sites consume it:

| Site | Direction | What an empty result produces |
|---|---|---|
| `definitions_of` | resolves a `PINNED-BY` citation | every citation "defines no function under crates/" — exit 1 |
| `build_tracked_path_index` | judges an `UNPINNED` tracker | every tracker "names no tracked path" — exit 1 |
| the census scan | compares a written figure | no document examined — exit 0 over a stale census |
| the spec-file list | the register itself | caught by the vacuity guard, but diagnosed as "matched no spec.md" |

Each consumes it as `mapfile -d '' -t arr < <(tracked_files …)`. The process substitution runs in a
subshell whose exit status the parent never reads, and `pipefail` does not reach it, so a `git` failure
and an empty repository are the same observation.

## Goals / Non-Goals

**Goals:**

- Tell a failed enumeration from an empty one, at every site, and refuse rather than judge.
- Keep one enumerator, so the discipline cannot be applied to three sites and missed on the fourth.
- Prove the direction on a fixture, as every other refusal here is proven.

**Non-Goals:**

- Auditing every process substitution in the file. Two read already-materialized data — the attribute-run
  `sed` over a file `grep` just located, and the `awk` over the id table this run wrote — which are
  computations over what was read, not enumerations of the observation source. They are named as the
  residual rather than swept in, because a scope a reader has to infer is one the next change gets wrong.
- Retrying a failed enumeration, or degrading to a filesystem walk. The spec forbids the walk and the
  exit contract has a value for "cannot decide".

## Decisions

- **The status is checked in the parent, which means the output must be buffered.** A status and a NUL
  stream cannot both come back from a process substitution, and command substitution cannot carry NUL
  bytes at all — bash strips them, which would silently defeat the `-z` this enumerator exists for. So
  the enumeration writes a trap-owned temp file, the parent checks its status, and `mapfile` reads the
  file. This is the discipline the file already applies to its other lazily-created temp files.
- **One file, reused, joined to the EXIT trap through the same conditional expansion `rendered` uses.**
  A per-call `mktemp` with its own `rm` leaks on any abort between the two.
- **A nameref, so the call site cannot forget the ordering.** `read_tracked_files <array> <pathspec…>`
  fills the caller's array; the alternative — a function that fills a shared buffer the caller must
  `mapfile` immediately — is a contract a future edit can break silently. `local -n` needs bash 4.3 and
  `mapfile -d` needs 4.4, which `test_examples.sh` already requires, so the floor does not move.
- **`cannot_judge` is called from the function**, which is legitimate only because the function runs in
  the parent shell. Calling it from inside a process substitution would exit the subshell and leave the
  parent reading an empty list — the exact bug being fixed, wearing the fix's clothes.
- **The spec-file enumeration routes through the same helper** even though its vacuity guard already
  stops a silent pass, because its diagnosis was a claim about the repository rather than the
  enumeration, and because two idioms for one query is how the first three copies happened.

## Risks / Trade-offs

- **Probability is low; direction is what earns the fix.** These enumerations fail on a corrupted index,
  a missing `git`, or resource exhaustion — not on ordinary input. But one direction reports *clean* over
  content it never read, in the gate whose subject is a coverage claim, and two others blame the register
  for a failure elsewhere. The repair is a status check.
- **A stubbed `git` is a strong fixture instrument.** The matrix stubs `git` to fail for the census
  enumeration alone and pass everything else through, so the case proves the enumeration's own direction
  rather than "the gate needs git". The same technique was used one change ago for `sed`, and it is what
  makes the negative run meaningful: without the fix the fixture exits 0 over a stale census.
- **The absent-spec scenario declares behaviour that already shipped in this window.** It is added here
  rather than left implicit because the register's requirements enumerate the reaction's refusals, and a
  reaction refusing in a direction the law does not name is the inverse of the drift this capability
  exists to end.
