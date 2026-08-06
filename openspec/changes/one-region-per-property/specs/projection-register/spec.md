## MODIFIED Requirements

### Requirement: Every generated document and the reaction holding it fresh SHALL correspond, in both directions

Each enumerated document SHALL name the unit that generates it, and that unit SHALL be tracked. **The number of
projections a unit blesses SHALL equal the number of enumerated documents naming it** — the correspondence is
counted per blessing call site, never per file.

Per file was a measured defect: a second `assert_projection_matches` added to an existing holder, blessing a tracked
document that carries no marker, was accepted in silence. The file was already paired with its first document and
nothing asked about the second. A holder blessing two documents and registering one is exactly the state the register
exists to make impossible.

The correspondence is by **count**, and what that does not reach is stated rather than implied: *which* call blesses
*which* document is not resolved, because the path is a constant in the source and reading it would mean evaluating
Rust rather than reading it. Two holders that each bless one document and swapped which one they name would satisfy
this count. The measured defect — a blessing nothing registers — is caught; a permutation is not, and calling the
count a per-pair correspondence would have been the overclaim this capability exists to refuse.

The reaction SHALL enumerate the holders **independently of the documents' own claims**, and SHALL recognize both
mechanisms this repository uses: a Rust call to the shared blessing rule, and a `check_*` gate writing its
projection under `BLESS`. The shell holder SHALL be recognized as a **gate**, not by mentioning `BLESS`: a twin
that proves the blessing behaves mentions it too and writes no projection, so the looser rule has a false positive
among the units this repository already has. A document naming its generator is a claim by the document; the call site is the fact, and an
inventory that trusted the claim would be an inventory of claims.

Both directions are required because each catches a different defect. A document with no holder is a
hand-maintained file wearing a generated file's warning — worse than plain prose, because a reader trusts it more
and no reaction defends it. A holder with no document is a projection whose freshness is asserted and whose
existence no reader has been told.

#### Scenario: A document claims generation that nothing asserts

- **WHEN** an enumerated document names a generator that holds no projection, or names none at all
- **THEN** the reaction fails, naming the document, because its warning not to edit rests on nothing

#### Scenario: A reaction holds a projection no document registers

- **WHEN** a unit holds a projection fresh and no enumerated document names it
- **THEN** the reaction fails, naming the unit and the path it blesses, because the document exists and the
  register does not know it

#### Scenario: The shell-generated projection is registered like any other

- **WHEN** the holder is a shell gate writing its projection under `BLESS` rather than a Rust call
- **THEN** it is enumerated and paired identically, because a reaction that recognized only the Rust mechanism
  would report a perfect correspondence over three quarters of the surface

### Requirement: Every generated document SHALL be reachable from where a reader is sent

Each enumerated document's path SHALL appear in `AGENTS.md`, which is the document a reader is told to open first.
A projection nothing points at is a file a reader finds by accident.

A path appearing only inside a fenced code block, or only inside an **HTML comment**, SHALL NOT count as a mention.
An HTML comment is invisible to the reader the requirement is about, so `<!-- docs/x.md -->` satisfying it was the
requirement being met in appearance and failed in substance — measured, not supposed. Prose is where a reader is sent; a
fence is where a command lives. This is live rather than hypothetical: one projection's path appears both in prose
and in a comment inside the Definition of Done fence, so a rule counting either would accept a document that only
a gate's comment mentions — the shape the Definition-of-Done membership check was already bitten by once.

#### Scenario: A projection is named nowhere a reader is sent

- **WHEN** an enumerated document's path does not appear in `AGENTS.md`
- **THEN** the reaction fails, naming the document, because the register knowing it exists is not the same as a
  reader being able to find it
