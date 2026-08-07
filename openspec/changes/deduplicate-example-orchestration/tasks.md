## 1. Orchestration ownership

- [x] 1.1 Record the current nested matrix markers emitted by the positive driver
- [x] 1.2 Remove the nested matrix invocations from `test_examples.sh`
- [x] 1.3 Correct DoD commentary to name top-level ownership

## 2. Verification

- [x] 2.1 Prove each focused matrix and the positive driver still pass in canonical order
- [x] 2.2 Run OpenSpec validation and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- Before the refactor, `bash scripts/test_examples.sh` itself emitted the published-family, repository-example,
  and isolated-warning matrix success markers before running positive examples.
- After the refactor, each focused matrix and then the positive driver pass in canonical order; the driver emits
  no focused-matrix marker. OpenSpec validation, repository hygiene, gate-shape, DoD coherence, and the complete
  Definition of Done pass.
