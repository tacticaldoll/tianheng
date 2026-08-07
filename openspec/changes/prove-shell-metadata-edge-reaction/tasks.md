## 1. Permanent Negative Fixture

- [ ] 1.1 Add an isolated `tianheng` fixture whose only forbidden normal dependency is the local `xingbiao` crate.
- [ ] 1.2 Add a self-governance test that selects exactly one live shell dependency boundary and evaluates it against the fixture without copying the allowlist.
- [ ] 1.3 Remove the fixture edge temporarily, run the focused test to observe the expected-violation guard fail on `Clean`, then restore the edge.
- [ ] 1.4 Run the focused self-governance target with the restored fixture and confirm projection freshness remains green.

## 2. Verification And Lifecycle

- [ ] 2.1 Run the repository Definition of Done and post-change self-governance reaction.
- [ ] 2.2 Sync the fixture-backed requirement and archive the completed change without retaining a dated archive copy.
