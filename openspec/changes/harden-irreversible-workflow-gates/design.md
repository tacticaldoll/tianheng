## Context

`scripts/publish.sh` and `scripts/merge-pr.sh` are workflow orchestrators immediately before acts that cannot be repaired in place. Their verdicts remain in Rust, but the evidence handed to those verdicts must itself be trustworthy. Today the publish gate collapses `git ls-remote` failure into an empty response, and the merge wrapper reads commit subjects from local remote-tracking refs that can be stale or cannot represent a fork head.

## Goals / Non-Goals

**Goals:**

- Preserve the cause and classification of a failed live-remote read.
- Make squash-message judgment independent of local remote-tracking-ref freshness and head-repository location.
- Prove both fixes at the observation level that changes.
- Keep shell limited to workflow input acquisition and orchestration; keep message and source judgment in Rust.

**Non-Goals:**

- Do not replace either workflow script with a Rust executable.
- Do not change publish eligibility, squash-message policy, public crate APIs, or release ritual.
- Do not address unrelated repository-check findings or the later reaction-vocabulary amendment.

## Decisions

### Match the live-remote command result before parsing it

The publish-source gate will match `git ls-remote` as a `Result`. A command failure becomes cannot-judge with the original Git error; only a successful response is parsed. A successful response without `refs/heads/main` remains cannot-judge, but receives a distinct message naming the absent ref.

The alternative—retaining one generic message for both cases—keeps the verdict kind safe but leaves an operator unable to distinguish an unavailable remote from a repository whose branch is absent.

### Read pull-request commits from the live PR API

The merge wrapper will first resolve the accepted `gh pr` selector to the pull request's canonical numeric identity, then use `gh api --paginate` against that pull request's commits endpoint. It will derive each subject from the first line of the API's full `commit.message`, preserving subjects longer than the `messageHeadline` projection and supporting fork heads without requiring a local remote. A selector that cannot resolve to one numeric pull-request identity stops before endpoint construction.

No local-ref fallback will be used. A fallback would reintroduce the same silent-subset direction whenever the live read failed.

### Treat missing commit evidence as workflow failure

An API failure or an empty derived subject set will stop the wrapper before the Rust gate and before `gh pr merge`. The wrapper does not classify the merge message; it only refuses to present incomplete evidence as the pull request's commits.

### Exercise the wrapper with controlled external commands

A repository test will invoke the wrapper with controlled `gh` and `cargo` executables. The fixture will make local refs stale or irrelevant, return a live commit set containing a subject absent locally, and require that the Rust-gate invocation receives that complete set. Separate directions will make API failure and an empty result stop before the merge command.

The publish test will assert the failed read's diagnostic cause, because the refusal kind is unchanged and cannot prove a diagnostic-only repair.

## Risks / Trade-offs

- **GitHub API pagination or response-shape drift** → Exercise more than one returned page in the controlled-command test and fail loudly when no subjects are derived.
- **Commit messages contain bodies** → Take only the first line of each full message, matching `git log --format=%s` without the API headline truncation.
- **The wrapper test accidentally exercises the real network or merge command** → Put controlled executables first on `PATH` and assert every received argument; no real repository mutation or remote call occurs.
- **A more explicit publish diagnostic becomes brittle** → Assert the command's stable failure cause and the remote identity, not incidental Git formatting beyond what the operator needs.
