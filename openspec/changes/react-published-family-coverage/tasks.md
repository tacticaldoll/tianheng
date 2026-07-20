## 1. Executable Family Ledger

- [x] 1.1 Add the repository-only published-family inventory and helpers that reject unknown
  claims, deduplicate fulfilled owners, and name missing inventory members.
- [x] 1.2 Attach each family fulfillment to an existing example block only after its real evaluator
  and family-specific structured reaction assertions succeed.
- [x] 1.3 Add focused negative checks proving that an unknown claim and an unfulfilled inventory
  member each fail loudly with the relevant family identity.

## 2. Governance Contract

- [x] 2.1 Update the capability-catalog commentary and repository governance documentation to
  identify the executable ledger as contract coverage, not an automatic builder-method registry.
- [x] 2.2 Verify the normal examples dogfood run fulfills every published 0.2.x family without
  changing focused tutorial scope or production/public surfaces.

## 3. Verification

- [x] 3.1 Run shell syntax/static checks and the focused positive and negative ledger checks.
- [x] 3.2 Run the complete Definition of Done from `AGENTS.md`, including self-governance and all
  isolated examples, before checking off implementation.
- [x] 3.3 Confirm the diff changes no manifests, package versions, public Rust APIs, identity or
  baseline wire, evaluator behavior, or serialized reports.
