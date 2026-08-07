## Context

The gate accepts an optional repository path so its failure matrix can judge fixtures. Most operations prefix that value when addressing tracked files and the generated projection. The written-census direction later changes the process working directory to the same value, leaving a relative argument with two meanings: it is first relative to the caller and then relative to itself.

## Goals / Non-Goals

**Goals:**

- Give every later path operation one stable physical repository root.
- Preserve the checked `cd` that distinguishes an unavailable repository from an ordinary grep miss.
- Exercise the defect with a relative fixture path that contains tracked Markdown and a generated projection.

**Non-Goals:**

- Change the register grammar, projection format, exit contract, or blessing semantics.
- Generalize repository-path handling across unrelated gates.

## Decisions

- Canonicalize the repository argument once, before any observation. `cd "$repo" && pwd -P` uses the shell tools this gate already owns and resolves both relative spelling and symlinks to the directory subsequent commands actually enter.
- Keep the later checked directory transition. Root normalization prevents path drift; it does not replace the cannot-judge direction for a repository that disappears between enumeration and census scanning.
- Add the regression to `test_bound_register.sh` as a real repository fixture invoked from its parent with a relative path. A unit over string concatenation would not exercise the state change that causes the defect.

## Risks / Trade-offs

- **Physical-path normalization changes diagnostic spelling** → Diagnostics may name the absolute judged root, but verdicts and generated paths become deterministic and no public document format changes.
- **A fixture can pass before reaching projection access** → The regression includes a tracked census and an existing projection so execution crosses the `cd` and reaches the path that previously doubled the root.
