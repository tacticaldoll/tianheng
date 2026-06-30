## Context

潛移 (gravity) made the law/reaction imitable and in-context. 垂象 (reaction visibility) is the complementary office: land the reaction where humans and agents already look — the PR diff and CI. `check` today emits machine output only as `--format json` (a tool must parse it). **SARIF** (OASIS, vendor-neutral) is the standard GitHub code-scanning and other tools/editors ingest. This projects the same `Violation` measure into SARIF — additive, non-breaking, no change to law/reaction/exit-code.

## Goals / Non-Goals

**Goals:**
- `check --format sarif` → SARIF 2.1.0, projecting the same non-baselined violations and the same exit code as JSON.
- Keep it a check-only reaction projection (symmetric to markdown being list-only).
- Be honest about location precision: file-level, no line (line is not observed).

**Non-Goals:**
- A GitHub-specific `--format github` (`::error::`) — vendor coupling; see the decision below.
- editor/LSP shift-left (a large, born-when-built integration → 0.2.0).
- Any change to JSON/text/markdown, exit codes, reaction semantics, or baseline identity.
- Line/column precision (would require new observation); a `repair_hint` or any field that rewrites the reason.

## Decisions

### SARIF only — a vendor-specific `--format github` is deliberately excluded

The first cut considered both SARIF and a GitHub `--format github` (`::error::` workflow commands). The github format is dropped: SARIF is an **open, vendor-neutral** interchange standard (GitHub code-scanning, GitLab, Azure, editors all consume it), whereas `::error::` is **one CI vendor's proprietary protocol**. Baking it into the tool couples Tianheng to GitHub and invites an open-ended `gitlab`/`azure`/… format set — against the minimalism bound. And it is largely redundant: GitHub already inlines SARIF on the PR. Turning the neutral output into vendor annotations (upload the SARIF, or `jq` the JSON report into `::error::` lines) is a **harness/CI-step convention**, not a tool feature — the same line Tianheng draws for 校讎 and the branching rules ("convention, not constitution"). A short README recipe carries it. This keeps 垂象's machine surfaces vendor-neutral.

### Generalize `json: bool` to a `ReportFormat` enum

The check path collapses `--format` to `json: bool`, threaded into `gate()` and the final output block. Replace it with `ReportFormat { Text, Json, Sarif }` threaded the same way; each output point matches on it. `gate()` has **two** dispatch sites (the constitution-error early-return and the main violations branch) — both widen. `report_json`/`report_violations`/coverage/text are unchanged — only the dispatch widens.

### SARIF 2.1.0, built from the same Outcome with `serde_json`

`report_sarif(&Outcome) -> String` builds a SARIF `Value` (no new dependency — `serde_json` is already sanctioned; it takes only the outcome, since stale/coverage are not projected to SARIF). `tool.driver.name = "tianheng"`; one `results[]` entry per **non-baselined** violation: `ruleId` = rule, `level` = `error` (Enforce) / `warning` (Warn), `message.text` = reason + finding (the rule is in `ruleId`, not the message). A constitution error becomes `runs[0].invocations[0]` with `executionSuccessful = false` (required on any SARIF invocation) plus a tool-execution notification at `error` level. Clean → empty `results`. Baselined violations are excluded (they do not fail), consistent with the human report.

### Location honesty — file-level, no line, no fabrication

A `Violation` observes `file` (some dimensions) but **never a line**. So a SARIF result's location carries only `physicalLocation.artifactLocation.uri = file` with **no `region`**; a violation with `file: None` emits **no** `locations` — a faithful absence, never a fabricated line/file. This file-level bound is stated in the spec.

### Exit code and outcome are untouched

`sarif` is presentation only: the composed `Outcome` and `outcome.exit_code()` are computed exactly as today; the format choice changes only what is written, never the verdict (0/1/2). A scenario pins that the SARIF run exits identically to JSON.

## Risks / Trade-offs

- **[SARIF schema correctness]** → emit the minimal valid SARIF 2.1.0 shape (version, `runs[].tool.driver.name`, `results[]` with `ruleId`/`level`/`message`, and `invocations[].executionSuccessful` on the error path — a required property); validated by a test asserting the key structure (not a byte snapshot). We do not claim full SARIF feature coverage — just a document GitHub code-scanning ingests.
- **[File-level locations look imprecise in a PR]** → honest: the line is not observed. Stated as a bound; fabricating a line would be the rejected false-precision.
- **[Refactor blast radius — the `json: bool` thread]** → contained: a mechanical enum substitution through `gate()` (both sites) and one output block; existing `report_json`/`report_violations`/coverage untouched; covered by the existing json/text tests plus new sarif tests.

## Open Questions

- None blocking. Coverage is deliberately not in SARIF (it is an observation, not a reaction; the SARIF surface is for the reaction).
