## Context

The DoD lists commands as shell invocations, while CI may express the same invocation through a GitHub Action. The existing reaction normalizes `run:` lines but special-cases `cargo deny check` out of comparison because CI uses `EmbarkStudios/cargo-deny-action@v2`. That exemption makes removal or misconfiguration of the action invisible.

## Goals / Non-Goals

**Goals:**

- Compare `cargo deny check` with the effective command declared by the existing cargo-deny action.
- Prove the action must be present and configured with `command: check`.
- Keep the reaction read-only and local to repository text.

**Non-Goals:**

- Do not implement a general GitHub Actions interpreter.
- Do not change the DoD, CI workflow, or cargo-deny configuration.
- Do not combine unrelated reference-integrity or vocabulary repairs.

## Decisions

### Project the one supported action shape into the command set

The reaction will recognize an `EmbarkStudios/cargo-deny-action` step and read its `with.command` value. `check` contributes `cargo deny check` to the same effective-command set as normalized `run:` lines. A missing or different value contributes no matching command, so ordinary set comparison reports the DoD entry missing from CI.

This is deliberately narrow. Treating every `uses:` step as a shell command would require action-specific semantics the repository does not own; silently exempting the command preserves the current false negative.

### Test the comparison through controlled document strings

The extraction and comparison will be callable with fixture AGENTS and CI text. A negative direction removes the cargo-deny step from an otherwise matching CI document; another changes its command. Both must name `cargo deny check` as missing. The real-workspace test remains the production entry point.

## Risks / Trade-offs

- **Action YAML shape changes** → The controlled directions pin the supported `uses` plus `with.command` shape; an unrecognized shape fails by making the DoD command missing.
- **Loose indentation associates a command with the wrong step** → Scope action parsing to one step and stop at the next list item at the same indentation.
- **Generic action interpretation grows accidentally** → Keep the projection specific to cargo-deny and document that boundary in the requirement.
