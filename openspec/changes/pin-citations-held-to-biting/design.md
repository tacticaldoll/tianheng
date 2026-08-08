# Design — a pinning citation held to biting

## The question this cannot be answered by reading text

The register already refuses to decide test-ness from source text. Its own header records why: a `#[test]`
removed by a `cfg`, a definition trapped in an uninvoked `macro_rules!`, a definition written inside a string
or a comment. Three reviews defeated the textual answer, and the gate now asks the harness instead.

*Biting* is strictly harder than *running*. Whether a test would fail if the reaction changed is a question
about executing a program, and no arrangement of characters answers it. This repository has just paid five
review rounds for a reaction that tried the textual route on a smaller question — whether one function body
delegates — and retired it, because execution is not a property of text. Nothing here repeats that: the gate
**runs the test**, against a tree where the reaction has been changed, and reads the exit status.

## Isolation, and the false clean it prevents

The obvious implementation is to mutate a scratch copy and reuse the repository's warm `target/`, since a cold
build looks expensive. Measured, that arrangement reports every pin as biting:

```
$ git archive HEAD | tar -x -C $scratch
$ (cd $scratch && CARGO_TARGET_DIR=<repo>/target cargo test … -- --exact <pin>)   # after mutating $scratch
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.01s
    test result: ok. 16 passed
```

Cargo resolved the fingerprint against the sources the artifacts were first built from — the real worktree —
so the mutated file was never compiled and the binary under test was the unmutated one. A gate whose whole
subject is a defence that is not really defending would have been exactly that.

The gate therefore owns its target directory. Warming it costs **5.2s** once for the test target measured;
each mutation afterwards recompiles only the crate it touched, **under a second**. Cost scales with the number
of declared mutations, not with the register's 60 citations.

Mutating the real worktree in place was rejected for a second reason as well as speed: a gate that edits
tracked files must restore them, and a gate interrupted between edit and restore has destroyed work. The
family's own law already says a reaction judges **tracked content**, never the working directory, and
`git archive HEAD` is that content by construction.

## The declaration

A mutation is four fields — the cited test name, a tracked path, `from`, `to` — kept in
`scripts/lib/pin_mutations.tsv`, one per line, tab-separated, with `\n` and `\t` escapes so a perturbation
spanning lines is still one record.

`from` SHALL occur **exactly once** in the named file. This is the anchor-uniqueness rule the observer
protocol's reader learned the expensive way in the same window: an anchor that matches twice names a set rather
than a site, and substituting the first occurrence silently mutates something other than what the author meant.
Zero occurrences and more than one are both **cannot judge** — the mutation could not be applied, which is a
different fact from the pin not biting, and collapsing them would let a rotted mutation read as a passing gate.

The test name is held against the register's own citations in both directions: a mutation naming a test no
bound cites is refused, because it is a perturbation of something this register does not claim to defend, and
its passing would read as coverage of a citation that does not exist.

## Why the file rather than the spec

The register parses `PINNED-BY` out of spec Markdown, and a `from` string is code — braces, backslashes,
backticks, and the delimiters Markdown itself uses. Putting an exact substring inside prose makes the record's
fidelity depend on escaping, and a mutation that fails to apply because a backtick was consumed is the
cannot-judge above, arriving for a reason that has nothing to do with the pin. A TSV holds exact bytes.

The cost is a second file to keep in step with the specs, which is the drift this repository refuses to accept
on trust — so the cross-check above is bidirectional and runs on every gate invocation, not a convention.

## What stays uncovered, and why the figure is printed

Authoring a mutation that genuinely perturbs the pinned point is expert work per bound, and a mutation that
misses reports a biting pin as a dead one. That direction is safe — an author meets a false alarm and answers it
by writing a better mutation — but it means coverage grows one considered entry at a time rather than by a
sweep.

So the gate prints the uncovered count on a clean run, exactly as `check_bound_register.sh` prints its figures
and `docs/observation-bounds.md` leads with the unpinned one. The alternative — reporting the mutations that
passed and saying nothing about the citations with none — is a gate reading as coverage, which is the class this
change exists to close.
