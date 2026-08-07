## 1. Cleanup ownership

- [x] 1.1 Add a partial-acquisition cleanup matrix direction
- [x] 1.2 Install inert cleanup before acquiring the signature directory
- [x] 1.3 Record the cleanup correction under `[Unreleased]`

## 2. Verification

- [x] 2.1 Observe the new matrix direction fail against the post-acquisition trap
- [x] 2.2 Run the publish-source matrix, OpenSpec validation, and repository hygiene gates
- [x] 2.3 Run the complete repository Definition of Done

### Verification evidence

- With cleanup installed after acquisition, `bash scripts/test_publish_source.sh` exited 1 and reported the
  partially acquired signature workspace still present.
- With cleanup preinstalled, the publish-source matrix, OpenSpec validation, repository hygiene gates, and the
  complete Definition of Done all pass.
