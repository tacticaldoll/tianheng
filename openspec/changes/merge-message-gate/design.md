## Context

`AGENTS.md` carries the squash-message rules; nothing holds them. Nine subjects in this repository's history
carry a trailing `(#N)`.

Measured before designing, because the obvious answers do not work:

| surface | why it cannot hold this |
|---|---|
| `commit-msg` / `pre-commit` hook | a squash merge runs on GitHub's servers; no local commit exists, so no hook runs |
| `squash_merge_commit_title` setting | both values append `(#N)`; the repository setting cannot suppress it |
| a CI check on the PR title | the serial is appended by GitHub, not carried by the title, so a clean title changes nothing |
| a `push` workflow on the release branch | the record has already landed, and a commit message is not rewritten |

What remains is the one string passed as `--subject`. That is a compliance point, not an enforcement surface —
so the design is a **wrapper at the point the act is launched**, exactly as `scripts/publish.sh` is.

## Goals / Non-Goals

**Goals:**

- A squash subject that disagrees with its PR title, or carries a serial, cannot reach a release branch through
  the sanctioned path.
- The judgement is a Rust reaction with a failure matrix, like every other reaction judging this repository.
- The wrapper is the only sanctioned way to reach `gh pr merge`, so the check is not a step to remember.

**Non-Goals:**

- Repairing the nine subjects already in history. They are records.
- Judging whether a body is *good*. The reaction holds shape — attribution, emptiness, and GitHub's
  concatenated commit list — not prose quality.
- Preventing a merge made in the GitHub web UI. A wrapper cannot reach the browser, and that residual is
  declared rather than implied.

## Decisions

### D1: The judgement is Rust; only the wrapper is shell

`rust-self-governance-gates` requires every reaction judging this repository to be a `#[test]` under
`crates/tianheng/tests/`. A judgement written in bash would violate the live specification this change amends.
So `scripts/merge-pr.sh` gathers inputs and runs `cargo test -p tianheng --test merge_message`, exactly as
`publish.sh` runs the publish-source gate — the shell carries no verdict.

### D2: The verdict is the shared `Refusal`

The gate separates *the message disagrees* from *the message could not be read* — an unavailable PR title is
not a wrong subject. Using the shared kinded `Refusal` means its construction sites are enumerated and
perturbed by `refusal_bites` like every other, so this reaction arrives already subject to the contract it is
an instance of.

### D3: The most specific refusal fires first

A subject carrying `(#N)` also differs from its PR title, and is also still conventional-shaped. Checked in
order — serial, then equality, then shape — so the message names the thing that is actually wrong. Reporting
"differs from the PR title" for an appended serial would send a reader to compare two strings that differ by
exactly the thing the rule already names.

### D4: What the body is held to, and what it is not

Three shapes, all mechanical: it must not be empty (*bodies carry provenance*), it must not carry agent
attribution, and it must not be GitHub's concatenated commit list — a body whose every non-blank line begins
with `* ` is the default the rule says to replace. Anything beyond that is a judgement over prose, which this
repository has measured and rejected three times.

## Risks / Trade-offs

- **A merge made in the web UI bypasses the wrapper** → declared, not implied. The wrapper makes the
  sanctioned path the easy one; it cannot reach a browser. The same is true of `publish.sh`.
- **The reaction reads its inputs from the environment** → so it can be run over any proposed message, which is
  what makes the failure matrix possible. The wrapper is the only caller that supplies real ones.
- **Nine offences stay in history** → a reaction over past subjects would be permanently red, and the repair it
  would demand is rewriting records. The reaction judges what is about to be written.
