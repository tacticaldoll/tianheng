## Context

Two measurements, taken before anything was written:

| Injected failure | Today | Contract |
|---|---|---|
| `sed` exits 4 inside `parse_spec` | gate exits **4**, printing nothing | 2, with a reason |
| `git ls-files` exits 3 in the sibling gate's index build | gate exits **3**, printing nothing | 2, with a reason |
| `find` prints one member then exits 3 | gate exits **1**, reporting `24 registered test names across 1 package(s)` and refusing citations in five unenumerated packages | 2 |

The first two are the same shape: `set -e` with `pipefail` aborts on an unhandled failure and the process
carries the failing utility's status out. The third is the swallowed-status class the previous two changes
closed for reads of the observation source — this call site was classified as *guarded*, and the guard
(`${#members[@]} -eq 0`) only catches a totally empty result.

## Goals / Non-Goals

**Goals:**

- The reaction exits `0`, `1`, or `2` on every path, whether or not the failure was anticipated.
- A failure that has a useful name keeps its own diagnosis; the structural rule is a floor, not a
  replacement.
- The package list stops being a filesystem walk whose failure is invisible.

**Non-Goals:**

- Wrapping every command individually. That is what has been tried twice and has twice left a site behind;
  the count of unwrapped commands is not the property to manage.
- Recovering from a failed read. There is a defined value for "cannot decide" and it is the honest answer.
- Auditing `check_release_coherence.sh`, already filed with its own entry — a different gate, a separate
  read.

## Decisions

- **An `ERR` trap, with `set -E`.** One mechanism covers every command in the file, including ones a later
  change adds. Its interaction with the shapes this file depends on was **measured, not reasoned**: a
  failure inside `if`, `||`, `&&`, an arithmetic guard, or a captured pipeline with its own handler does
  not fire it, even inside a function under `errtrace` — which is exactly the property that makes it safe
  to install over code full of deliberate non-zero returns (`grep -q` misses, `[[ ]] && continue`,
  `((status <= 1)) || cannot_judge`).
- **`$LINENO` in the trap, not a guess at intent.** The trap cannot know what failed; it reports where, and
  says so. A trap that invented a cause would be worse than the raw status it replaces.
- **`parse_spec` keeps an explicit refusal.** "An unhandled command failed at line 610" is a worse answer
  than "the spec at `openspec/specs/x/spec.md` could not be read", and the read has a name worth giving.
  The trap is what catches the sites nobody named.
- **Packages come from tracked `crates/*/Cargo.toml`, through `read_tracked_files`.** This replaces `find`
  rather than checking it: the enumerator already checks its status in the parent, already reads tracked
  content — which is what the requirement says this reaction does — and one fewer external tool is one
  fewer status to propagate. The behavioural difference is that a directory with no tracked manifest is no
  longer offered to `cargo test -p`, which would have failed on it one step later; a directory that *is* a
  package necessarily carries a tracked manifest.

## Risks / Trade-offs

- **A trap that fires where it should not would break every run.** Hence the probe before the design, and
  hence the matrix keeps its full set of passing directions: if the trap misfired on a `grep -q` miss or an
  `|| continue`, the passing cases would fail immediately rather than subtly.
- **The trap makes the contract hold without making each site's diagnosis good.** That is a real limit and
  is stated in the requirement rather than papered over: a named refusal is better, and the two the review
  found get named refusals. What the trap guarantees is that the *code* is inside the contract.
- **Deriving packages from manifests changes what "a package" means** for a malformed tree. The old
  behaviour reached `cargo test -p <dirname>` and turned its failure into `cannot judge`; the new one
  simply does not consider a manifest-less directory a package. Both refuse to invent a package; the new
  one refuses earlier and for a stateable reason.
