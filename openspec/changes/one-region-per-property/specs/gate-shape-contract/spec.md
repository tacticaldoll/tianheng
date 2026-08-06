## MODIFIED Requirements

### Requirement: Both files SHALL be reachable from the Definition of Done, except the publish-time gate

Each enumerated gate and its twin SHALL be **invoked** by `AGENTS.md`'s Definition of Done block — appearing in
command position, not merely mentioned in it. That block is the single source for the local pre-flight list.

Mentioned-is-not-invoked is a measured defect, not a hypothetical: `test -f scripts/check_whitespace_hygiene.sh` in
the block satisfied the membership check while executing nothing. A gate the block names and never runs is the
"matrix present but unrun" class wearing the appearance of coverage, which is worse than the absence, because the
projection then reports it reachable. A gate present in the tree and absent from that block runs nowhere by
default, which is the "matrix present but unrun" class this window recorded three times.

`scripts/check_publish_source.sh` is exempt from the gate half of this requirement and SHALL be declared as
such: it runs from `scripts/publish.sh` at publish time, because no development checkout is a release
snapshot. Its twin is in the block; the gate is not.

The exemption SHALL be checked **live**, not merely honoured. A hand-written exception that has stopped
applying is an exception that rots silently, and this one rots in the flattering direction: were the
publish-time gate ever added to the Definition of Done, an exemption that only ever *permits* would keep
permitting, and the next reader would inherit a licence with no live instance behind it.

#### Scenario: A gate or twin is absent from the Definition of Done

- **WHEN** an enumerated gate or its twin does not appear in the Definition of Done block
- **THEN** the reaction fails, naming the file, because a gate nothing invokes is a comment

#### Scenario: The publish-time gate's absence from the Definition of Done is excused by name

The exemption is deliberately **not** declared through the observation-bound mechanism. A bound says a reaction
stops at a shape; this says one named instance is excused from a requirement. Declaring it as a bound would put
something that is not an observation limit into the register whose leading figure counts exactly those — and
`observation-bound-model` would then demand a typed extent for it, which no value in that model honestly fits.

- **WHEN** the reaction reaches `scripts/check_publish_source.sh`, which the Definition of Done deliberately
  omits because it runs at publish time
- **THEN** the reaction accepts its absence as declared policy rather than reporting a violation, while still
  requiring its twin's membership

#### Scenario: The membership exemption has stopped applying

- **WHEN** `scripts/check_publish_source.sh` appears in the Definition of Done block
- **THEN** the reaction fails, saying the exemption is stale and must be retired, because an exception with
  no live instance behind it reads as licence to the next author

### Requirement: Every enumerated gate SHALL have a companion failure matrix holding five properties

Each gate SHALL have a twin, and each twin SHALL: assert expected exit **codes** rather than merely non-zero;
hold at least one passing direction and at least one refusing direction; assert that the gate left the
repository it judged unchanged; and assert that a clean run prints nothing on stderr.

Each is a class this window observed, not a checklist assembled for symmetry. Asserting non-zero rather than
the code let a genuine incoherence collapse from 1 into 2 and ride green through CI. A matrix with no passing
direction cannot distinguish a working gate from one that refuses everything. The silent-clean-run assertion
is the only one that catches the shape where the shared backstop printed cannot-judge once per clean file
while the exit code stayed 0 — invisible to every check reading only the code.

Every property over a twin or a gate SHALL be read from the **region of the text it is about**: executed text for a
property about what runs, the header for a property about what a file declares of itself, prose for a property about
what a reader is sent to. Two properties once read the whole file while their own helper's documentation said they
were about executed text, so `expected_status` in a header comment satisfied a property about an assertion — the
wrong check rather than a loose one.

Requiring the twins' helper form (`expect_pass` / `expect_fail`) is legitimate where requiring a *product*
test-name convention would not be, and the difference is ownership: these twins are authored in this
repository for this purpose, so this capability may require their shape, exactly as the bound register may
require a scenario heading's form while declining to require a pinning test's name.

Two of the five are recognized through an authored form on that same argument, and the forms are named here so
an author meets the requirement rather than discovering it as an invented violation. The silent-clean-run
assertion SHALL capture stderr alone — a redirection of the form `2>&1 >/dev/null` — and SHALL test the
variable that capture assigned for emptiness. The unchanged-repository assertion SHALL name itself in its
refusal, saying the gate `mutated` what it judged; the comparison behind it cannot be recognized mechanically,
since the twins compare a porcelain listing, a `HEAD`, a tag list and a directory walk in four combinations.

#### Scenario: A gate has no twin

- **WHEN** an enumerated gate has no companion twin beside it, named by substituting `test_` for `check_`
- **THEN** the reaction fails, naming the gate, because a gate nobody has watched refuse is protection
  claimed rather than protection observed

#### Scenario: A twin asserts non-zero instead of the code

- **WHEN** a twin's refusing directions assert only that the gate exited non-zero
- **THEN** the reaction fails, naming the twin, because a violation reported as cannot-judge — or the reverse
  — is then indistinguishable from the verdict the gate owed

#### Scenario: A twin has no passing direction

- **WHEN** a twin holds refusing directions only
- **THEN** the reaction fails, naming the twin, because a gate that refuses everything satisfies such a
  matrix completely

#### Scenario: A twin does not assert a silent clean run

- **WHEN** a twin never asserts that a clean run's stderr is empty
- **THEN** the reaction fails, naming the twin, because a gate can print cannot-judge on every clean input
  while exiting 0, and no assertion on the code can see it

#### Scenario: A twin does not assert the gate is read-only

- **WHEN** a twin never asserts the judged repository is unchanged after the gate runs
- **THEN** the reaction fails, naming the twin, because a gate that edits what it judges makes its own next
  verdict unreproducible
