## 1. Payload reconstruction

- [x] 1.1 Add an extracted-signature mismatch matrix direction
- [x] 1.2 Refuse suffix mismatch as cannot-judge before verification
- [x] 1.3 Record the corrected exit classification under `[Unreleased]`

## 2. Verification

- [x] 2.1 Observe the matrix direction fail against silent suffix removal
- [x] 2.2 Run the publish-source matrix, OpenSpec validation, and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- With the new matrix and silent suffix removal, `bash scripts/test_publish_source.sh` exited 1: the gate reported
  that the real tag's signature did not verify after only the extracted block was altered, rather than exit 2.
- With the suffix assertion restored, the publish-source matrix, OpenSpec validation, repository hygiene gates,
  and the complete Definition of Done all pass.
