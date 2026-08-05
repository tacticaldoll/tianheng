## Why

A third review produced three more inputs where a `PINNED-BY` citation is satisfied by something that never
runs, each reproduced before being accepted:

- `#[test]` followed by `#[cfg(any())]` — the attribute run says test, and Rust removes the item before
  registration. **exit 0.**
- `#[test] fn cited()` inside an uninvoked `macro_rules!` body — real tokens, no test. **exit 0.**
- `#[test] fn cited()` inside a multi-line raw string. **exit 0.**

That is the same class the previous change answered by *declaring a residual*: the gate matches text shape, so
it cannot answer whether a test registers. Declaring it was the wrong call, and the reason is on record — the
harness enumeration was rejected twice in that change's own comments as needing "a compiled workspace, and the
whole failure matrix is throwaway repositories holding one `lib.rs` and no manifest". Measured rather than
assumed, that premise is false:

| measurement | result |
|---|---|
| `cargo test --list` over this workspace, warm | **1s**, 1251 tests |
| all 36 cited names present in it | **36 / 36** |
| the same per package, for crate precision | **746ms** for all six |
| a throwaway fixture crate with a 6-line manifest, **cold** | **107ms** |
| `#[cfg(any())]` test / uninvoked macro-body test in that fixture | **not listed** — refused exactly |

So the residual was not out of reach; the cost of reaching it was estimated from inside the code instead of
measured. Enumerating more sub-cases instead is unbounded: `cfg`, `cfg_attr`, feature gates, a cfg-gated
`mod`, comments, strings, macros — the previous change stated one of them and this review found three more.

Two smaller defects from the same review:

- `r#type` is refused, though the requirement says a name SHALL be a Rust identifier and the reaction adds no
  other restriction. Non-ASCII identifiers are refused too.
- A definition whose `fn` and name sit on different lines is reported as **absent**, an undeclared line-shape
  limitation.

## What Changes

- **The test harness becomes the authority on whether a citation names a test that runs.** The reaction
  enumerates each workspace member's registered tests (`cargo test -p <pkg> --all-features -- --list`) and
  requires the cited name to appear in the cited crate's set. Per package rather than workspace-wide, because
  `--list` carries no crate label and this repository already has one test name live in two crates.
- **The text scan becomes the fallback, and the degradation is loud.** A repository with no root manifest
  cannot be enumerated, so the attribute-run walk still answers test-ness there — and the reaction says on
  stdout that it did, because a gate that silently drops its strongest direction reports a weaker clean than
  it claims.
- **The block-comment-definition residual is retired**, along with the projection's third floor and its
  `BACKLOG.md` entry: an unregistered definition — commented, stringified, cfg-removed, or macro-trapped —
  now fails. The retirement is recorded rather than quietly deleted, because it was declared one change ago.
- A raw identifier (`r#name`) is accepted; the contract narrows to **ASCII** identifiers with the reason
  stated, since the search pattern is byte-oriented and no cited name needs otherwise.
- A citation the harness registers but the definition scan cannot locate is reported as the **line-shape
  limitation** it is, naming what the scan requires, instead of as an absent test.
- CI runs the register step **after** the build, so the enumeration is warm rather than a duplicate compile.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `observation-bound-register`: test-ness is decided by the harness with the text scan as a declared
  fallback; the citation grammar accepts a raw identifier and is narrowed to ASCII; the line-shape limitation
  is named in a diagnostic; the definition-form residual is retired.

## Impact

- `scripts/check_bound_register.sh` — the harness index, the verdict order, the fallback notice, the
  projection header.
- `scripts/test_bound_register.sh` — manifest-bearing fixtures for the harness directions, a fixture for the
  degradation notice, and the retired residual's fixture inverted to a refusal.
- `openspec/specs/observation-bound-register/spec.md` — the resolution requirement.
- `.github/workflows/ci.yml` — step order.
- `AGENTS.md` — the Definition of Done comment for the register lines, which now depend on a built workspace.
- `BACKLOG.md`, `docs/observation-bounds.md`, `CHANGELOG.md`.
- No crate, no public API, no adopter action.
