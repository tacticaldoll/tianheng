## Context

The 0.2.0 identity conversion is intentionally dimension-owned: `CrateFact`/`ModuleFact`,
`SemanticFact`, and `RuntimeFact` produce `FindingKey`s without exposing their typed schemas through
璇璣. Existing tests establish envelope validation, presentation independence, and selected
distinctness properties, but they do not form an exhaustive compatibility reaction over the
published schemas. Whole report snapshots are unsuitable because they also freeze diagnostic
presentation that the structured identity design deliberately freed.

The reaction must span three crates without creating a 三儀-to-三儀 dependency. It must also make a
new fact variant an explicit compatibility decision instead of relying on a reviewer to remember a
separate hand-maintained index.

## Goals / Non-Goals

**Goals:**

- Pin the version-2 identity schema of every currently shipped fact family at its owning dimension.
- Make adding a fact variant require an explicit catalog decision at compile/test time.
- Pin representative canonical values wherever a renderer or canonicalizer feeds a key.
- Keep the assertions focused on identity-bearing roles, including the outer target and rule.

**Non-Goals:**

- Changing any production key, baseline version, finding text, rule label, or public API.
- Moving typed fact schemas into 璇璣 or exposing private dimension enums.
- Freezing complete report JSON, ordering unrelated to identity, or human presentation.
- Resolving the 0.3.0 candidates for unsafe-site, async-seam, or rule identity.

## Decisions

### Keep one exhaustive catalog in each owning dimension

Each dimension's private fact tests will enumerate its variants and assert exact `FindingKey`
namespace, code, canonical field names, and representative values. Test-only exhaustive matches over
the fact enums and every nested enum that changes a code, field set, or canonical value make a newly
added identity shape fail compilation until its schema is consciously added. This includes 圭表's
dependency kind and 渾儀's exposure kind, public-seam kinds, item/member/associated kinds, and
trait-impl positions. 圭表 keeps separate coverage for its crate and module fact enums; 漏刻's
audit-only variants remain under the existing `audit` feature.

This is preferred over a shell-level integration catalog. A central catalog would either expose
private fact vocabulary or duplicate it outside the observing dimension, both violating ownership;
it would also make one dimension depend on another merely for tests.

### Assert key objects, not presentation documents

Catalog assertions compare exact `FindingKey` values or their key-only JSON object. They do not
snapshot `Finding::text`, a `Violation` document, or a report. A small representative assertion per
dimension inspects a violation produced by that dimension's real boundary reaction—not an arbitrary
direct constructor call—and proves target/rule remain the outer identity roles while the observed
fact remains the key.

This is preferred over repeating every capability fixture or using golden baseline files: the former
would duplicate the full behavior suite merely to re-prove one shared wiring convention, while a
golden would also freeze finding wording and baseline presentation, obscuring whether a failure is a
wire break or harmless output polish.

### Treat canonical key renderers as wire code

Documentation on helpers whose output enters a key will say that their byte form is version-2 wire,
even when the same helper also serves readable output. The catalog supplies representative byte
pins; comments alone are not the backstop.

No attempt is made to replace string canonicalization with a richer shared value model. That remains
the recorded pressure trigger, not a requirement of this compatibility-only change.

## Risks / Trade-offs

- **[Catalog duplicates expected constants from production]** → Keep duplication test-only and
  literal: the point is an independent published-wire expectation, not another production source.
- **[A broad snapshot freezes presentation]** → Assert only namespace, code, canonical fields,
  values, target, and rule; explicitly exclude finding text and report JSON.
- **[Audit-only runtime facts escape default-feature testing]** → Place the exhaustive runtime
  catalog behind `feature = "audit"`; the repository DoD and CI run all-feature tests, while the
  production-light default build remains unchanged.
- **[A legitimate future identity change makes the test noisy]** → Require the change to declare
  whether it is additive and patch-safe or a baseline migration that earns a breaking window; the
  test is intended to fail before that decision is hidden in a refactor.

## Migration Plan

There is no adopter migration: production bytes remain unchanged. If implementation reveals that a
current key differs from the recorded 0.2.0 wire, stop and treat it as a design issue rather than
updating the expected value. Rollback is removal of test/documentation changes only.

## Open Questions

None. The 0.3.0 identity questions are deliberately outside this change.
