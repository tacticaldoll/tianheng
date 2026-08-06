# Change: a permitted builtin piped onward is a stated bound

## Why

Round 4 of this window's closing review, on the process-substitution property.

`gate-shape-contract` refuses `done < <(producer)` when the producer can fail, and permits one exception, which its
own requirement states as a *reason* rather than a list:

> A producer that is a **shell builtin over data already in memory** — `printf` or `echo` re-splitting a variable —
> SHALL be permitted, **having no I/O to fail at**.

The recognizer reads the producer's **first word**. So `< <(printf '%s\n' "$rows" | sort)` is permitted — first word
`printf` — while `sort` is an external process with I/O that can fail, and the permission's stated reason does not
hold for it. A producer that emits rows and then fails in its second stage would be judged as a whole read, which
is the exact defect this property exists to refuse.

**Latent, not live.** Every process substitution in the enumerated gate surface was read: two are
`< <(printf '%s\n' "${…}")` over memory, and the rest sit in comments, which the executed region already excludes.
Nothing in the tree exhibits the shape.

## Why this is declared rather than fixed

The obvious repair — also refuse a producer containing `|` — was tried against the tree before being believed, and
it **false-positives on both live sites**:

```
first='printf'   contains_pipe=True   done < <(printf '%s\n' "${b//|/$'\n'}")   ← legitimate, in the tree twice
first='printf'   contains_pipe=True   done < <(printf '%s\n' "$rows" | sort)    ← the defect
```

The live sites carry a `|` **inside a parameter expansion**, and a pipe operator cannot be told from a pipe inside
`${…}` by matching text — that needs shell parsing. So the honest instrument is a declared bound, and the
alternative is recorded as measured-and-rejected rather than left for someone to re-propose.

## What Changes

- A bound scenario in `gate-shape-contract`, and the matching typed declaration: extent
  `Reached(UnderReacts)`, owner **engine**, because the recognizer *is* handed the producer's text and stops at its
  first word.
- A pin that demonstrates all three legs by feeding the recognizer text: the under-reaction is accepted, an
  external producer is still refused (so the pin cannot hold for a recognizer that never fires), and the naive
  pipe rule's false positive on the live shape is asserted, so the reason for declaring rather than fixing is
  executable rather than prose.

## Impact

- Affected specs: `gate-shape-contract`
- Affected code: `crates/tianheng/tests/gate_shape_contract.rs`, `crates/tianheng/src/bounds.rs`
- No public API change, no version bump. The register gains one bound; both projections gain that row.
