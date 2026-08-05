## Context

The observation-bound register's citation resolution is a text scan: a `grep -rnE` for the definition form,
then an upward read of the attribute run. Three inputs defeat it, each reproduced on a throwaway repository
before being accepted as a finding:

| input | before |
|---|---|
| `PINNED-BY \`a_probe_bound_is_pinne.\`` | exit 0 — resolves to `a_probe_bound_is_pinned` |
| `PINNED-BY \`../outside::a_fn\`` | exit 0 — resolves outside `crates/` |
| `/*`, `#[test]`, `*/`, `pub fn cited()` | exit 0 — a non-test satisfies the citation |
| `#[test]` above 13 further attributes | exit 1 — a real test refused |

The first three are false coverage, the class this capability exists to end. The fourth runs the other way and
is a loud refusal.

The review filed the first two as a security alert. They are not treated as one: the input is
repository-controlled prose in a tracked spec, so whoever can write it can edit the gate script beside it and
no privilege boundary is crossed. Reclassifying them as false coverage raises their priority rather than
lowering it — a citation naming a test that does not exist, passing, is precisely the failure the register
was built to catch.

## Goals / Non-Goals

**Goals:**

- A citation cannot resolve to a function it does not name, or to one outside `crates/`.
- A commented-out attribute cannot satisfy test recognition.
- No legitimate attribute run is refused for being long.
- The residual that remains is stated where a register reader sees it and pinned by a fixture.

**Non-Goals:**

- Lexing Rust. Not deferred — measured as the wrong tool for this gate; see the decision below.
- Distinguishing a commented-out *definition* from a real one. Same reason. It becomes a stated residual.
- Escaping the cited name for safe interpolation. Validation is strictly better: it refuses the citation
  instead of silently searching for something odd.

## Decisions

**1. Validate, do not escape.**

Two candidate fixes for the metacharacter direction: escape the name before interpolation, or refuse a name
that is not an identifier. Escaping makes `a_probe.` search for a literal `a_probe.`, find nothing, and report
"no function under crates/ defines it" — technically correct and diagnostically wrong, because the citation is
malformed rather than stale. Validation names the actual defect. It also generalizes: the same validation of
the qualifier closes the traversal direction, since a crate-directory name cannot contain `/` or `.`.

Measured before specifying: all 36 cited names are plain identifiers, and every directory under `crates/` is
a plain name, so the validation refuses nothing that exists. Raw identifiers (`r#type`) would be refused; none
is cited, and the refusal would be loud.

**2. Stop the attribute walk at a block-comment delimiter; do not strip or track comments.**

The review's correction was to strip or track block comments while walking. Neither works here:

- *Tracking while walking upward* cannot work in principle. Whether a line sits inside a block comment is a
  property of everything **before** it, and the walk moves backwards from the definition with no knowledge of
  the file above.
- *Stripping* requires knowing which `/*` opens a comment and which is text inside a string literal. This
  tree makes that concrete rather than theoretical: 49 `/*` occurrences live **inside string literals**,
  several nested, because `louke`'s lexer and the workspace's lexical-conformance suite test exactly that
  (`crates/tianheng/tests/lexical_conformance.rs:72`, `crates/louke/src/audit/tests.rs:673`). A
  delimiter-counting stripper would open a phantom comment at the first of them and swallow every definition
  until the next `*/`, so the gate would start refusing real citations in this repository on the first run.

Stopping at the delimiter needs neither. It treats `/*` and `*/` as boundaries — the same role the existing
walk already gives a blank line and the previous item's end. Verified before adopting: no `#[test]` run in the
tree contains a block comment, and none of the 36 cited tests is affected. Its error direction is loud — a
test whose run genuinely contains a block comment is refused, which an author sees.

**3. Walk to the item boundary, not to a line count.**

The 12-line cap was a backstop against walking to the top of a file. The stop conditions already provide that
boundary — a blank line, or a line ending `{`, `}`, or `;` — so the cap only ever removes correct behaviour.
Removing it needs the preceding lines read once rather than one `sed` per line, which is also cheaper.

**4. The commented-out definition becomes a stated residual, not a declared bound.**

It is observable — a fixture demonstrates it — so the register's own rule for unobservable residuals does not
apply. What blocks a declaration is the **citation form**: `PINNED-BY` names a Rust test under `crates/`,
while every defence of this reaction is a shell fixture in `scripts/test_bound_register.sh`. Declaring the
bound would force `UNPINNED` against a tracker owning something already measured as out of reach — permanent
debt wearing an owner's name, which the unpinned requirement forbids.

So the residual is stated in the requirement and in the projection's header (its third floor), and pinned by a
fixture asserting the accepted behaviour, so a later repair fails that fixture instead of being absorbed
silently. That the register cannot pin a bound of its own capability is filed as a `BACKLOG.md` observation
with its reproduction, not solved by widening the citation grammar in a patch responding to a review.

## Risks / Trade-offs

- **[Validation refuses a citation form someone legitimately wants — a raw identifier, or a nested module
  path]** → loud refusal naming the citation, never silent coverage, and nothing in the tree uses either.
  Widening the grammar later is additive.
- **[Stopping at `/*` refuses a test whose attribute run carries a block comment]** → measured absent from
  this tree; the refusal is loud and the repair is to move the comment. Accepting such a run would require the
  lexing this design rejects.
- **[Removing the line cap lets the walk read to the top of a small file]** → bounded by the same stop
  conditions; a file whose first item has an unbroken attribute run to line 1 is a file where that run really
  is the attribute run.
- **[The commented-definition fixture pins a weakness, so it reads as endorsement]** → the requirement and the
  projection header both state it as a floor, and the fixture carries a comment saying it records an accepted
  residual rather than a desired behaviour.

## Migration Plan

None. Repository gate only; no crate, API, or adopter-visible behaviour. Every citation in the tree already
satisfies the tightened form, and `docs/observation-bounds.md` is regenerated in the same change.

## Open Questions

None.
