## Context

Three reports, all of the shape *a judgement passes over something it did not read*. Verified before planning;
the publish one was reproduced against a running git rather than reasoned about.

## Goals / Non-Goals

**Goals:** a cleanliness verdict that depends on the repository and not on the checkout; two enumerations that
cannot pass over an unread file; a citation answered twice that fails by name.

**Non-Goals:** changing what `cargo publish` packages; repairing history; widening any judgement into prose.

## Decisions

### D1: `hermetic()` alone does not close the exclusion class — measured

| ambient source | `hermetic()` as written | `-c core.excludesFile=/dev/null` |
|---|---|---|
| global / system `core.excludesFile` | closed | closed |
| XDG default `$XDG_CONFIG_HOME/git/ignore` | **survives** | closed |
| `.git/info/exclude` | **survives** | **survives** |

Routing the judgement through the existing builder would have read as a repair while the XDG default still hid
files. The neutralisation is therefore explicit, and the third source needs something else entirely.

### D2: Classify the hidden set by source; do not refuse it wholesale

`git ls-files --others` applies no exclusion; `git status --untracked-files=all` applies all of them. The
difference is the ignored-untracked set, and `git check-ignore -v --no-index <path>` names the **source file**
for each — measured:

```
by-repo.txt      .gitignore:1:by-repo.txt
by-clone.txt     .git/info/exclude:1:by-clone.txt
```

A source is legitimate only if it is **tracked**. An *untracked* `.gitignore` reports a repository-looking
source while being no more part of the repository than `.git/info/exclude` is — found by measuring, not by
reading, and it is why the rule is "tracked" rather than "named `.gitignore`".

Refusing whenever `.git/info/exclude` is non-empty was the simpler alternative and is rejected: it trades a
false clean for a false alarm on the gate guarding an irreversible act, which is the worse trade.

### D3: The specification has to say what *clean* means first

The requirement reads *any modified or untracked file present → exit 1*, and an ignored file **is** untracked.
The control this change needs — a tracked `.gitignore` must not block a release — asserts the opposite of that
sentence. So the clarification is a prerequisite, not closing paperwork: without it the control encodes a
preference rather than a requirement.

### D4: An unreadable file is not an absent one

`let Ok(text) = read_to_string(&manifest) else { continue }` collapses two facts. The member-manifest read one
function above already separates them, and its cannot-judge has a direction. The example scan gets the same
shape, and skips only where `Cargo.toml` genuinely does not exist.

### D5: A failed directory entry is propagated even if no fixture can produce one

Dropping it is the false clean the report names. Propagating adds refusal sites that may be unconstructible,
which is what construct → delete → declare is for — and if they are declared, **one slug per site**, since a
slug shared by two sites excuses whichever was looked at. The exemption registry already refuses a shared slug,
so injectivity is machine-checked rather than remembered.

### D6: `PINNED-BY` is plural and `UNPINNED` is singular, deliberately

`observation-bound-model` says several `PINNED-BY` lines are all retained; `Unpinned(String)` holds one
tracker. Two trackers therefore have no representation and one is silently chosen. The new state names the
bound, and a control keeps two `PINNED-BY` accepted — that asymmetry is stated, and a repair that flattened it
would break a live declaration.

## Risks / Trade-offs

- **The repair could over-refuse a legitimate publish** → which is why D2 classifies rather than refuses, and
  why the accept-control lands in the same increment as the hole-closing one.
- **The XDG source cannot be exercised in-process** → it needs process-global environment every parallel
  direction shares. Its closure rests on the measurement in D1, recorded at the call site rather than asserted
  by a test that cannot run.
- **New refusal sites turn `refusal_bites` red until they are reached or declared** → each site lands with its
  direction in the same increment, never deferred.
