## Why

Every gate installs the shared backstop with a label it writes by hand:

```bash
exit_contract_backstop 'bound register'
```

That label is the gate's self-identification in the one diagnostic a reader gets when the shell aborts a gate
rather than the gate refusing — the failure mode the backstop exists for, where before it existed six gates
exited 131, 130, 9, 7, 4 and 3 **with no output at all**. Nothing ties the label to the gate that wrote it.

The damage a wrong label does is **bounded, and worth stating as bounded**: the same message prints
`${BASH_SOURCE[0]}:$LINENO`, which expands in the failing gate's own frame, so the path is right even when the
prefix is not. A mislabeled gate therefore produces a *contradiction* rather than a blank —

```
bound register: cannot judge: an unhandled command failed (exit 7) at scripts/check_whitespace_hygiene.sh:120
```

— and a contradiction is read in whichever direction the reader trusts first. Someone grepping CI output for
the gate they changed finds nothing; someone reading the prefix opens the wrong file.

Why now, and why this is not urgency dressed up: the hazard is pure copy-paste, and copy-paste is exactly how
this gate surface came to exist — six gates carrying one shape, each written by reading a sibling. The
`gate-shape-contract` reaction that landed one change ago already reads every gate's text and already requires
the backstop's *installation*; adding what its argument must be is one entry in an array. Measured on the
tree: **6 of 6 gates already agree with their basenames**, so this requirement describes what is there rather
than migrating anything.

## What Changes

**One property added to `gate-shape-contract`'s exit-contract requirement**, making the surface's checkable
properties ten rather than nine.

- The label a gate passes to `exit_contract_backstop` SHALL be the gate's own name, derived from its basename:
  `check_` and `.sh` removed, underscores read as spaces. `scripts/check_bound_register.sh` therefore names
  itself `bound register`.
- The reaction gains the tenth entry, the projection gains a column, and a failure names the gate, the label it
  wrote, and the label its own basename asks for — the repair is then the message rather than a search.
- No gate changes. This is a describes-the-tree requirement.

**Counts of the properties are removed from prose rather than incremented.** Four documents and several doc
comments carried "nine", and a set this reaction enumerates and prints should not also be counted by hand — the
class that has already produced four different answers to one question in this repository. The projection prints
the figure; prose points at the projection.

## Capabilities

### Modified Capabilities

- `gate-shape-contract`: the requirement *Every enumerated gate SHALL hold the family's exit contract in a
  checkable form* gains the label's correspondence to the gate's own name.

## Impact

- **Modified**: `crates/tianheng/tests/gate_shape_contract.rs` (one array entry, one recognizer), and its
  projection `docs/gate-shape-contract.md` (one column).
- **Modified**: prose carrying a hand-written count of the properties — `AGENTS.md`, `BACKLOG.md`, and the
  reaction's own doc comments.
- **Not affected**: no gate, no twin, no crate's public API, no `Constitution`, no baseline format. Version
  class **PATCH**; repository-internal governance.
