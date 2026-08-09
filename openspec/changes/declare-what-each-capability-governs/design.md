## Context

Two things are wrong at once, and the second is the reason the first matters.

**The name.** Of the 25 top-level reaction targets under `crates/tianheng/tests/`, 5 reach Tianheng's shipped
API and 20 do not. The capability holding the twenty is called `rust-self-governance-gates`, and `AGENTS.md`
states that all of them "run Tianheng's own reactions against the workspace". Self-governance — governing this
repository with the delivered product — is what the five do, and `governance-dogfood` already owns it.

**The filing.** Which capability a requirement belongs to is decided once, in a proposal, and checked by
nothing. The misnomer plus the unchecked decision already produced one defect: a requirement about what
`scripts/publish.sh` must do before `cargo publish` was filed under a capability whose own Purpose says
`publish.sh` "is a wrapper rather than a gate".

## Goals / Non-Goals

**Goals:**
- The two governances have different names, and a reaction keeps them apart by measurement rather than by
  prose.
- Each capability says what it governs, in a form a machine reads.
- A change that touches files owned by a capability it did not name is red **at proposal time**.

**Non-Goals:**
- Tiling the repository with subjects. Files no capability claims stay unjudged; that blindness is declared.
- Holding every requirement to a subject through its text. That option was considered and not taken — see
  below.
- Any change to what the twenty reactions judge. Only what they are called and where their requirements live.

## Decisions

### The new name is `rust-repository-reactions`, and the `rust-` prefix is load-bearing

The requirement set moves **verbatim**, so review reads a rename rather than a rewrite.

The prefix is not decoration. The constraint that broke was *this capability's subject is Rust test files*, and
a name that carries the language is the one a reader consults before filing. `repository-reactions` was
considered and rejected for dropping exactly the word whose absence caused the defect.

### Renaming the capability renames four published bound ids, and that is not avoidable

`crates/tianheng/src/bounds.rs` carries four ids beginning `rust-self-governance-gates/`, exported through
`pub use bounds::observation_bounds` at `lib.rs:52` and already published in 0.4.0. So this rename changes
data an adopter can observe.

Keeping the old id strings while renaming the directory was considered and **rejected**: the register requires
a bound id to be *derived*, `<capability>/<scenario-slug>`, "requiring no lookup table and no allocation step".
Pinning the strings would convert every one of them into an assignment and defeat the requirement that makes
them checkable. The rename is therefore announced in `CHANGELOG.md` under `[Unreleased]`, in the 0.5.0 window
where a `0.x` minor already signals an observable change. No version is bumped here; that is release prep.

### A capability declares its subject as git pathspecs, resolved by git

A `## Subject` section between `## Purpose` and `## Requirements`, holding backticked globs:

```markdown
## Subject

- `crates/tianheng/tests/**/*.rs`
```

Membership is `git ls-files -- <glob>`. **No glob matcher is written.** Git's pathspec is both the matcher and
the definition of "tracked", so subject membership is one produced answer rather than a reimplementation —
the same reason the refusal corpus was rebuilt on rustc's own dep-info instead of a text model of `#[path]`.

Measured before choosing the format: `openspec validate` accepts the section, and both spec-parsing reactions
(`observation_bound_model`, `projection_register`) pass with one present.

Guard against a declaration drifting into fiction: every declared glob SHALL match at least one tracked path.
A dead glob is a subject claim about nothing.

### The join is against the change's produced diff, not against its prose

For each **active** change directory: the touched set is `git diff --name-only <base>...HEAD`, minus that
change's own directory. For each touched file, if any capability's subject claims it, then some capability
claiming it SHALL appear in the proposal's Capabilities section.

Alternatives considered:

- **Join the proposal's list to the delta-spec directories present under `specs/`** — cheap, and it would not
  have caught this defect: a wrong capability choice writes the delta in the wrong place too. Both sides come
  from the same decision, so the comparison is `f() == f()`.
- **Scan requirement text for paths outside the subject** — rejected as the weaker of the two. It needs a
  marked form for legitimate rationale (one exists today: `scripts/` is named in a requirement about what
  ships in zero packages), and a requirement that names no path at all slips past. The scenarios written for
  the parked change said "a wrapper", not a path — they would have passed.

**Overlapping subjects are allowed**, and the join then requires only that *one* claiming capability be
listed. Two capabilities may legitimately govern one file; demanding all of them would refuse honest
proposals.

**Base determination is the fragile part.** It is resolved from the branch's upstream and the merge-base with
the tracked release and main refs; when it cannot be resolved the reaction is a **cannot-judge**, never clean.
With no active change directory the reaction is clean, so an ordinary checkout is not made noisy by a branch
question it has no reason to ask.

### The two governances are held apart by set equality, not by a label

One declared list of the targets that govern this repository with the delivered product, beside the
enumeration that measures which targets reach the shipped API. The two SHALL be equal **in both directions**:
a target cannot carry the name without the reach, nor reach without being named.

The measurement blanks comment lines in place before scanning, reusing what `refusal_sites` already does, so a
doc comment mentioning `tianheng::Boundary` does not make a target a dogfood reaction.

One list, not two — a declared set beside a hand-kept mirror is how a member gets to exist and never be
measured.

## Risks / Trade-offs

- **[The rename sweeps ~126 vocabulary occurrences across ~26 files]** → records are excluded by rule, not by
  care: dated `CHANGELOG.md` sections and `docs/history/` are not rewritten, and `release-coherence` requires
  the `### Self-governance` heading text, which stays. The mechanical part — the 23 occurrences of the
  capability id — is joined by `bound_register`, which fails if any is missed.
- **[36 subject declarations are 36 new claims that can drift]** → each is held by the dead-glob guard, and
  by the join itself: a subject too narrow makes honest changes red, a subject too wide claims files the
  capability does not govern and shows up when those files are touched.
- **[Base determination fails in a shallow CI clone]** → cannot-judge, which stops the run and says why. The
  failure direction is deliberate; reading an unresolvable base as "nothing touched" would report clean over
  every change.
- **[Subjects do not tile the tree]** → declared as a bound. A file no capability claims is not judged, and
  the reaction says so rather than implying coverage it does not have.
