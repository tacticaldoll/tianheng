## 1. Hermetic governance policy

- [x] 1.1 Prove ambient `GOVERNANCE_DOCUMENTS` cannot weaken a real run
- [x] 1.2 Replace the environment override with an explicit fixture-only argument
- [x] 1.3 Guard own-workspace, empty-value, and unknown-argument refusals
- [x] 1.4 Record the gate correction under `[Unreleased]`

## 2. Verification

- [x] 2.1 Observe every new matrix direction fail against the ambient override implementation
- [x] 2.2 Run the reference-integrity matrix, OpenSpec validation, and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- Before the explicit option existed, the own-workspace fixture invocation was ignored and exited 0.
- With argument validation present but the ambient override retained, a missing `PROJECT.md` under a poisoned
  `GOVERNANCE_DOCUMENTS` value exited 1 as a stale reference rather than exit 2 as a missing required document.
- Removing the non-empty check made the final matrix fail because empty fixture policy exited 0; removing the
  unknown-argument refusal likewise made an unknown option exit 0.
- Removing the exact-arity check made a surplus fixture-policy value exit 0, which its matrix guard refused.
- The final reference-integrity matrix, live gate, OpenSpec validation, and whitespace hygiene passed.
- The complete repository Definition of Done passed, including all Rust, dependency-policy, release-state,
  governance, and isolated example reactions.
