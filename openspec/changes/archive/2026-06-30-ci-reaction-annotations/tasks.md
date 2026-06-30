## 1. Format parsing and the ReportFormat enum

- [x] 1.1 Add `sarif` to the `--format` parse in `runner.rs` (both `--format X` / `--format=X`); the unknown-format usage message lists `text|json|markdown|sarif`.
- [x] 1.2 Introduce `enum ReportFormat { Text, Json, Sarif }`. In the `check` path, replace the `json: bool` derivation with a `ReportFormat`; `markdown` for `check` stays a usage error (exit 2).
- [x] 1.3 `list` rejects `sarif` (usage error, exit 2) — it projects the reaction, not the law (symmetric to markdown being list-only). Update the `list` format match accordingly.
- [x] 1.4 Update the `check` usage line to `check ... [--format text|json|sarif]`; leave the `list` usage line as `text|json|markdown`.

## 2. Thread ReportFormat through the output paths

- [x] 2.1 Change `gate(...)`'s `json: bool` parameter to `ReportFormat`. `gate()` has **two** `if json` sites — the constitution-error early-return AND the main violations branch; **both** become a `match` (Json → report_json; Sarif → report_sarif; Text → the existing human path). The constitution-error arm dispatches to sarif too.
- [x] 2.2 Change the final non-gate output block the same way (Json/Sarif print the rendered string to stdout; Text → `report(&outcome)` + coverage). Exit code is `outcome.exit_code()` unchanged in every arm.

## 3. SARIF renderer

- [x] 3.1 Add `report_sarif(outcome: &Outcome) -> String` building a SARIF 2.1.0 `serde_json::Value`: `version` `"2.1.0"`, `runs[0].tool.driver.name = "tianheng"`, and `runs[0].results[]` one per **non-baselined** violation — `ruleId` = rule, `level` = `error`(Enforce)/`warning`(Warn), `message.text` = reason + finding (the rule is in `ruleId`, not the message).
- [x] 3.2 Location: when the violation has `Some(file)`, add `locations[0].physicalLocation.artifactLocation.uri = file` with **no `region`**; when `None`, emit no `locations`. Never fabricate a line.
- [x] 3.3 Clean outcome → valid SARIF with empty `results`. ConstitutionError → `runs[0].invocations[0]` with **`executionSuccessful = false`** (required by SARIF for any invocation) and a `toolExecutionNotifications[]` entry at `level` `error` carrying the message. Pretty-print via `serde_json::to_string_pretty`.

## 4. No GitHub-specific format (vendor-neutrality)

- [x] 4.1 Do NOT add a `--format github` (`::error::`) built-in — it would couple the tool to one CI vendor. Leave a short comment in `runner.rs` recording why (SARIF is the neutral surface; vendor annotations are a harness convention). Add a README recipe: for GitHub PR inline annotations, upload the SARIF (code-scanning), or in a CI step convert `--format json` to `::error::` lines with `jq`.

## 5. Tests

- [x] 5.1 `report_sarif` on an enforced violation (pure function) → parse as JSON; assert `version` `2.1.0`, `runs[0].tool.driver.name` `tianheng`, a result with `ruleId`, `level` `error`, message containing the reason. Assert structure, not a byte snapshot.
- [x] 5.2 Location honesty: a file-bearing violation → SARIF result has `artifactLocation.uri` and NO `region`; a file-less violation → SARIF result has no `locations`.
- [x] 5.3 Clean → empty `results`; constitution error → `runs[0].invocations[0].executionSuccessful = false` + a notification carrying the message.
- [x] 5.4 `list --format sarif` → exit 2 (check-only). Unknown format still exit 2.
- [x] 5.5 Exit-code parity: the same enforced-violation outcome exits 1 under sarif exactly as under json; a clean workspace exits 0 under sarif.
- [x] 5.6 Confirm JSON/text/markdown outputs and exit codes are unchanged by this change (existing tests still pass).

## 6. Verify (Definition of Done)

- [x] 6.1 `TIANHENG_WORKSPACE_TESTS=1 cargo test -p tianheng --all-features` passes; `cargo fmt --all --check`, `cargo clippy --all-targets --all-features -- -D warnings`, `RUSTDOCFLAGS=-Dwarnings cargo doc --workspace --no-deps --all-features` all clean (verify by exit code).
- [x] 6.2 Confirm no new external dependency (SARIF built with existing `serde_json`); no `Cargo.toml` version change (bump deferred to the release commit); reaction/exit-code/baseline semantics unchanged.
