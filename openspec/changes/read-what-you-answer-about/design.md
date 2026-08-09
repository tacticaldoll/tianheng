## Context

Four judgements answer about something other than what they read. Three are one substitution away from a
false negative; the fourth invents a value where its signature already has a channel for refusing.

## Goals / Non-Goals

**Goals:** every path asked about is the path that was given; a failed classification refuses; the package
enumeration reads tracked content; the anchor is refused rather than invented.

**Non-Goals:** changing what any of these judge, or any public API — `audit_corpus_and_anchor` already returns
`Result`.

## Decisions

### `-z` everywhere the gate handles a path

`ls-files -z`, `status -z`, and `check-ignore -z -v --stdin`. Measured before choosing it: the same fixture
under `-z` returns `ignored-普通` raw, and `check-ignore --stdin -z -v` answers
`.gitignore|1|ignored-*|ignored-普通`.

Alternatives considered: **unquoting the printed form ourselves** reimplements git's C-style escaping — a
third hand-rolled unescaper in a judgement that decides whether a publish may proceed. **Comparing quoted
forms on both sides** works for the `ls-files` / `status` comparison, where both quote identically, but not
for `check-ignore`, which is asked about the path rather than shown it.

`status --porcelain=v1 -z` emits `XY <path>` records NUL-separated, and a rename record carries two paths;
untracked records (`?? `) never do, and untracked is all this comparison reads.

### The classifier's failure is a refusal, not an empty answer

`check-ignore` exits 1 when it matched nothing, which for a path this function computed as excluded is a
disagreement between two listings rather than an answer — that case already resolves conservatively. What
changes is a **failure to run**: a non-zero status that is not the no-match code becomes a cannot-judge naming
the paths left unclassified.

### The package enumeration is `git ls-files`, not `read_dir`

`git ls-files -- 'crates/*/Cargo.toml'`, with a non-zero status refusing. Tracked content is what the
capability's own requirement names, and it settles the untracked-directory case for free.

`cargo metadata` was considered — it is the set `-p` accepts — and **not** chosen: this reaction's requirement
says *tracked content like every other read*, and a member present in the worktree and in no commit is exactly
what that phrasing excludes. Where the two disagree, the disagreement is a defect the reaction should report
rather than absorb.

### The anchor refuses instead of inventing

The innermost fallback returns `Err`. The manifest-directory fallback stays: it exists for synthetic metadata
carrying no `workspace_root`, which a unit test constructs and a real `cargo metadata` read always carries.
What is removed is the literal `/`, which is the defensive over-foolproofing of an impossible state the
minimalism bound forbids — with the error channel sitting unused in the signature.

## Risks / Trade-offs

- **[`-z` parsing is hand-rolled]** → it is a split on NUL, not an unescaper: the reason for choosing it is
  that it removes the escaping question rather than answering it.
- **[The fixture needs a name git quotes]** → non-ASCII is enough and is measured; a non-UTF-8 name would
  overlap the fixture that already exists for unreadable bytes.
- **[Refusing a failed enumeration is louder than shortening]** → that is the requirement, and the alternative
  is the false negative it was written against.
