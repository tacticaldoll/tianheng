## 1. Validate the citation before resolving it

- [ ] 1.1 Probe first: confirm on a fixture that `PINNED-BY \`<name>.\`` resolves to a differently-named
      function with exit 0, and that a `../` qualifier resolves outside `crates/` with exit 0.
- [ ] 1.2 Confirm all 36 cited names are plain Rust identifiers and every `crates/` directory is a plain
      name, so validation refuses nothing that exists.
- [ ] 1.3 Reject a cited name that is not a Rust identifier, a qualifier that is not a crate-directory name,
      and a citation with more than one `::`. Fail naming the bound id and the rejected citation.
- [ ] 1.4 Add the refusal fixtures: a metacharacter name, a traversing qualifier, a two-`::` citation.
- [ ] 1.5 Confirm the crate-qualified passing fixture and all 41 real citations still pass.

## 2. Stop the attribute walk at a block-comment delimiter

- [ ] 2.1 Probe first: confirm `/*`, `#[test]`, `*/`, `pub fn cited()` currently passes with exit 0.
- [ ] 2.2 Confirm no `#[test]` run in the tree contains a block comment, and no cited test's preceding lines
      carry one, so the stop refuses nothing that exists.
- [ ] 2.3 Add `/*` and `*/` to the walk's stop set.
- [ ] 2.4 Add the refusal fixture for the block-commented attribute.

## 3. Walk to the item boundary rather than a line cap

- [ ] 3.1 Probe first: confirm a `#[test]` above 13 further attributes is currently refused with exit 1.
- [ ] 3.2 Read the preceding lines once and walk to the stop conditions with no line cap.
- [ ] 3.3 Add the passing fixture for an attribute run longer than the old cap.

## 4. State the commented-definition residual, and file what blocks declaring it

- [ ] 4.1 Probe: confirm a whole definition inside a block comment currently satisfies a citation, and that
      this predates the test-ness check.
- [ ] 4.2 Add the third floor to `render_projection`'s header.
- [ ] 4.3 Add the fixture recording the accepted residual, commented as a residual record rather than a
      desired behaviour.
- [ ] 4.4 File a `BACKLOG.md` observation: the register cannot pin a bound of its own capability, because
      `PINNED-BY` names a Rust test under `crates/` while its own defences are shell fixtures. Record the
      reproduction, not a cost estimate.
- [ ] 4.5 Regenerate `docs/observation-bounds.md` and confirm the non-blessing run is clean.

## 5. Verification and lifecycle

- [ ] 5.1 Run every new guard against the code **without** the change and record the observed acceptance or
      refusal for each.
- [ ] 5.2 Run the full Definition of Done from `AGENTS.md`.
- [ ] 5.3 Sync the delta, run `openspec validate --specs --strict`, update `CHANGELOG.md`.
- [ ] 5.4 Archive, prune the dated copy, and open one squash PR into `release/0.4.1`.
