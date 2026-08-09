## Context

Two wrappers stand in front of an act that cannot be undone, and both read only an exit status that answers
*did the selected tests pass* while the question they are asking is *did the gate judge this act*. Those differ
exactly when the name is wrong, which is exactly when a rename has quietly happened.

## Goals / Non-Goals

**Goals:**
- A wrapper stops before the irreversible command when its gate did not run, and says so.
- A rename that would disarm a wrapper is red in an ordinary `cargo test`, before any wrapper is invoked.

**Non-Goals:**
- Observing a publish or a merge made without the wrapper. That bound is declared and stays declared.
- Any change to what either gate judges. Only whether it is known to have judged.

## Decisions

### The wrapper requires one passing test, not a zero exit

Capture the run's output and require `test result: ok. 1 passed`; on anything else print it and exit 1.

Alternatives considered: **checking the exit status only** is the defect itself; **asking `-- --list` first**
runs two invocations and checks the one that did not decide. **Asserting inside the gate** cannot work — a
renamed or `#[ignore]`d test cannot report that it did not run.

If `libtest`'s summary format ever changes the match fails and the wrapper refuses. A wrapper that stops when
it cannot read the result is right; one that proceeds is the bug being fixed.

### The reaction resolves the identifier through the harness, not through a path

For `--exact <ident>` bound to `--test <target>` in package `<pkg>`, ask
`cargo test -p <pkg> --test <target> -- --list` and require `<ident>` exactly once among the registered names'
last segments.

Mapping `--test <target>` to a source path would reimplement cargo's target resolution in string form. This
repository has paid for one such reimplementation already — a `#[path]` closure reasoned rather than measured
shipped a false negative, and the refusal corpus was rebuilt on rustc's own dep-info. The harness's `--list`
*is* the set `--exact` filters against, which makes the join exact rather than approximate, and it settles
duplicates for free.

**What `--list` cannot see**: an `#[ignore]`d test is still listed. That direction is the wrapper's
`1 passed`, which is the concrete form of "neither substitutes for the other".

### The invocation is a logical line, not a physical one

The gate call spans several physical lines joined by trailing `\`. Join continued lines first, then look for
`--exact` and `--test` within one logical line — the shell's own rule applied once, rather than a pattern
hoping to span newlines. This is the mistake `refusal_sites` made and corrected: a line-oriented search missed
a constructor whose call was wrapped.

An `--exact` in a logical line carrying no `--test` is a **cannot-judge**. The reaction cannot bind it to a
target, and an unbound identifier is one it could not resolve rather than one it resolved as fine.

### Scope: tracked scripts, enumerated by git

`git ls-files scripts/` — the enumeration `release-coherence` already uses for the same directory, with its
non-zero status a cannot-judge.

## Risks / Trade-offs

- **[The reaction shells out to cargo]** → `bound_register` already runs `cargo test … -- --list` per member,
  so the pattern and its cost are established. Two targets are listed here.
- **[`test result:` format drift]** → fails closed in the wrapper, and the reaction is the second holder.
- **[A future wrapper uses a different flag]** → the reaction holds `--exact`, the only form in the tree.
  The requirement is written over *the identifier a wrapper cites*, so the gap is visible when it appears.
