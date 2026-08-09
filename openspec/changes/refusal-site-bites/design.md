## Context

Two reactions carry a kinded refusal: `crates/tianheng/tests/support/publish_source_gate.rs` (25 construction
sites) and `crates/tianheng/tests/support/release_coherence_gate.rs` (35). Each defines its own `Kind`,
`Refusal`, `violation` and `cannot_judge`. `rust-self-governance-gates` already requires their directions to
assert *which* outcome a shape produces; nothing holds that requirement, and a review sweep at `22ec98e`
measured **24 of the 60 sites** as surviving both a kind swap and a message replacement.

The measured ground this design rests on, taken before designing rather than after:

| | |
|---|---|
| Sites | 60 — no line carries two, so `file:line` identifies a site uniquely |
| `Refusal { … }` literals outside the two constructors | 0 — the constructors are the only entry |
| Targets that can observe a site | `publish_source` + `publish_source_integrity` (publish), `release_coherence` (release) |
| Target run times | 794ms / 137ms / 255ms |

Because no line carries two constructor calls and no literal bypasses them, both the static enumeration and
the run-time interception are total. Neither fact is assumed; both are re-checked by the reaction, so a future
edit that breaks either one fails rather than silently shrinking coverage.

## Goals / Non-Goals

**Goals:**

- Every reached refusal site is distinguished by some test direction in **both** of its contracts — its kind
  and its message.
- A site that no direction distinguishes, and a site the suite never reaches, both **fail**.
- A site that genuinely cannot be constructed is declared at the site itself, cited by one bound, and its
  membership is produced rather than typed.
- The perturbation costs no rebuild, so the reaction is measured in seconds rather than in builds.

**Non-Goals:**

- Judging the *content* of a refusal message. This holds that some direction depends on the message, not that
  the message is a good sentence.
- Reaching beyond kinded refusals. `reference_integrity` and the census sweep return offence strings, not a
  kinded `Refusal`; extending to them is a separate subject. A future gate re-declaring the **same names**
  outside the shared module fails (D5b); one that renames its vocabulary is outside, and is declared as a
  bound rather than claimed as covered.
- Replacing `pin_bites`. That reaction answers a different question — whether a *cited* test dies under a
  *declared* source mutation — and its records stay as they are.

## Decisions

### D1: Perturb at run time through the caller's location, not by mutating source

`pin_bites`' machinery is checkout → edit → rebuild → run → restore. Applied here it would be 120 rebuilds,
each of a support file `#[path]`-included into three targets, so each rebuild is close to full. That is tens
of minutes and a worktree that holds an edit if the run is killed.

Instead the two constructors take `#[track_caller]` and consult a single injection point:

```rust
#[track_caller]
pub fn violation(message: impl Into<String>) -> Refusal {
    let site = std::panic::Location::caller();          // read HERE — see D2
    poison(site, Refusal { kind: Kind::Violation, message: message.into() })
}
```

`poison` reads `TIANHENG_REFUSAL_MUTANT=<file>:<line>:<mode>` and, only when the site matches, swaps the kind
or replaces the message. The consequences:

- **No rebuild between perturbations.** One `--no-run` build, then N process runs of already-built binaries.
- **Restore is free.** Nothing on disk changed, so there is no window in which a killed run leaves the tree
  edited — the reason `pin_bites` needs a detached worktree at all.
- **Cost:** 25 sites × 2 perturbations × 931ms + 35 × 2 × 255ms ≈ **65 seconds**, plus controls.

*Alternative rejected — source mutation.* Same answers, two orders of magnitude more expensive, and it
reintroduces the destructive-restore window this repository has already been bitten by.

*Alternative rejected — a compile-time switch (`cfg`) per site.* It cannot select one site without a rebuild,
which is the cost being avoided.

### D2: `Location::caller()` is read inside the `#[track_caller]` chain, and that is falsifiable

`#[track_caller]` propagates only through annotated frames. Reading `Location::caller()` inside a helper such
as `.poisoned_here()` that is *not* itself annotated measures the shared constructor's own interior — every
site would report the same one or two lines, the reaction would enumerate 60 sites and intercept two, and it
would read as coverage. So the location is read **in the annotated constructor's own body** and passed down as
a value; any helper that reads it instead must carry `#[track_caller]` too.

This is not left to review. A direction builds two refusals from two different lines of a fixture and requires
the recorded locations to **differ**. If the chain is broken, both record the constructor's interior and are
equal, and that direction fails.

### D3: `file:line` is a selector, never an identity

A selector is valid only for the duration of one build: inserting a line above a site moves it. So `file:line`
is used for exactly one thing — naming which site to poison in one run — and never for anything that outlives
the run.

In particular **an exemption is not keyed on `file:line`**. A site that genuinely cannot be constructed is
written with a named form carrying a stable slug:

```rust
cannot_judge_out_of_reach("ssh-keygen-unavailable", format!("…"))
```

The slug is a compile-checked argument at the site, so it moves with the site.

*Alternative rejected — a marker comment above the site.* It carries no compile-time force and can be moved
away from any site; the argument form cannot.

*Alternative rejected — keying the exemption on the message text.* Messages are operator-facing prose, so
rewording one would silently move an exemption.

### D3a: The slug↔bound join needs a data model, and it is repository-local

`BoundDecl` carries a `BoundId`, a subject, an `Extent` and a pinning test name. Nothing in it can answer
*which slugs does this bound cover*, and `Extent::OutOfReach { because }` is a **shipped public type** —
`observation_bounds()` is `pub` at `crates/tianheng/src/lib.rs:52` and `bounds.rs` is packaged. Widening it to
carry exemption membership would change a product API to serve a test.

So the join lives in a repository-local typed registry in test support:

```rust
pub struct Exemption {
    pub slug: &'static str,   // carried at the site, compile-checked
    pub bound: &'static str,  // the BoundId this exemption is covered by
    pub because: &'static str,
}
```

and the reaction holds a **three-way join**, each edge required in both directions:

| Edge | Refusal when it is missing |
|---|---|
| site slug → registry | a site declares itself out of reach under a slug nothing covers |
| registry → site slug | a registry entry names a slug no site carries — a dead exemption |
| registry → `observation_bounds()` | an entry names a `BoundId` the live bound set does not contain |
| registry ← `observation_bounds()` | the exemption-class bound is present while the registry is empty — permanent residue |
| slug uniqueness | a slug carried by two sites excuses whichever member happened to be looked at |
| reachability | a slug whose site is observed being constructed — a stale exemption |

The registry↔bound edge is a **biconditional**, not one direction. There is exactly one exemption-class bound,
so "registry non-empty ⇒ bound present" alone lets the last exemption disappear while the bound survives as
permanent residue — a declared false negative about a set with no members, which reads as a limit the
reaction still has. Both directions are required: registry non-empty ⇔ the bound is declared.

The registry is a hand-written table, which is the form this repository distrusts — so it is never the
authority for anything. Both of its columns are joined against sets that are **produced** (the site
enumeration, the live bound set), so it cannot drift silently in either direction; and its size is a census
rather than a number anyone types. It is a join table, not a declaration.

### D4: Both perturbations must kill, because kind and message are independent contracts

| Perturbation | What it catches |
|---|---|
| `kind` — swap `Violation`↔`CannotJudge` | a direction asserting only the message, so the kind an operator acts on before an irreversible act is unheld |
| `message` — **replace** the whole message with a sentinel | a direction asserting only `Kind`, and **shadowing**: two sites producing one needle, where the assertion cannot say which fired |

The message perturbation replaces rather than prefixes. A prefix leaves `contains(needle)` passing, so it
finds neither of the two things above.

Requiring *either* would let a site be observed in one contract and rot in the other. Requiring *both* is the
stronger reading and is what the sweep that produced the 24 actually did in each half.

### D5: Every site falls into exactly one of five classes, and three of them are red

A recording run (`TIANHENG_REFUSAL_RECORD=<path>`, appending each construction's location) gives the set the
suite actually reaches. One file per target run, so no two processes share it, and the appends within a
process are serialised behind a mutex; a failed write or an unparseable line **panics** rather than being
skipped. That is not tidiness. A lost line makes a site look unreached, and an unreached site is not
automatically red — if it happens to be a declared exemption, the loss lands it in a *legal* class and the
run goes green. Recording integrity is therefore load-bearing for a whole class of verdict, not just for
counting.

Combined with the static enumeration and the perturbation runs:

| Site | Verdict |
|---|---|
| reached, both perturbations kill a direction | ✅ defended |
| reached, some perturbation kills nothing | ❌ undefended |
| never reached, declared out of reach | ✅ declared, and counted in the residual |
| never reached, not declared | ❌ unreachable and unclaimed |
| declared out of reach, but reached | ❌ stale exemption |

The classification is total, which is the property that keeps the reaction from having a silent category. Only
reached sites need poisoning, so the perturbation cost scales with what the suite constructs rather than with
the enumeration.

### D5a: The corpus is what rustc read, taken from dep-info, and tracked-ness is a separate refusal

`pin_bites` reads HEAD, because it checks HEAD out into a worktree and judges that. This reaction has no
worktree: it perturbs the binaries built from the **current tree**. Enumerating from HEAD would therefore
compare one tree's text against another tree's run — and a refusal site added but not yet committed would be
invisible to the static enumeration *and*, if it is unreachable, invisible to the recording too. Both halves
blind at once is a genuine false clean.

The first draft said "the transitive `#[path]` include closure". That is a **reimplementation of rustc's
module resolution**, and it is wrong in four directions at once: it misses conventional `mod foo;`, `include!`,
and `#[cfg_attr(…, path = …)]`, and it *includes* files `cfg` excluded from the build actually run. This
repository has already shipped a false negative from exactly this pattern — mimicking rustc resolution by
reasoning instead of measuring against a real build.

So the corpus is **rustc's own answer**. `cargo test --no-run --message-format=json` names every test target
of the package and its executable; `<executable>.d` beside it is the dep-info rustc emitted, listing precisely
the source files it read. Both facts were verified against a real build before being designed around, not
reasoned about:

```
release_coherence-b7fa18453b28aa0d.d:
  crates/tianheng/tests/release_coherence.rs crates/tianheng/tests/support/release_coherence_gate.rs Cargo.toml
```

This also settles where the **roots** come from: the same JSON enumerates every integration-test target in the
worktree, so a newly added third gate target is inside the corpus without anyone remembering to add it. A
hard-coded root list is exactly the shape where a whole target falls outside.

**The residual, declared rather than hidden.** Dep-info records what rustc read *under the feature set built*.
The reaction builds with `--all-features`, which is the set it also runs, so enumeration and run agree; a file
compiled only under some other feature combination is outside, and that is a declared bound rather than a
silently narrower claim.

Tracked-ness does not disappear; it becomes its own refusal. Any file in that closure that `git ls-files` does
not name **fails**, saying that a judged target compiles content no one else has and no review can see. This
is the complement of the tracked-content rule rather than a departure from it: a gate over *shipped* content
judges what is tracked, and a gate correlating text with a **run** judges what ran, then requires it to be
tracked.

### D5b: Scope derivation must find the gate that does not use the shared module

"Which targets declare the shared refusal module" finds only the gates already playing by the rule. The gate
that matters — a third one defining its own `Kind` and `Refusal` — is precisely the one that would not appear,
and the spec states that shape as a refusal while nothing would enumerate it.

Two derivations, from the same corpus:

- **Foreign vocabulary — and only what is enumerable.** Every file in the corpus is scanned for the **exact**
  syntax the shared module defines: an item named `Refusal`, an enum carrying a `CannotJudge` variant, or an
  `fn violation` / `fn cannot_judge` definition, anywhere outside `support/refusal.rs`. Any hit **fails**,
  naming the file.

  "A `Kind`-like enum" is not a mechanical predicate. A gate that names its type `Decision` with variants
  `Disagrees` and `Unreadable` carries the same contract and matches none of the above, and no widening of the
  scan reaches it — recognising a vocabulary by intent is a judgement over source, which is the instrument
  this repository has measured and rejected three times. Nor is there a compile-time construction that forces
  a gate not yet written to return the shared type: any "every gate SHALL" is a convention until something
  enumerates gates, and what a gate *is* has no mechanical definition either.

  So the claim is narrowed to what is checked, and **a renamed vocabulary is a declared bound**. The
  over-claim in the first draft — "a future kinded gate cannot fall outside quietly" — was false: it can, by
  choosing other names.

- **Per-site targets.** A site's observers are the targets whose dep-info lists the site's *file*, not the
  targets that happen to include the shared module. Deriving from the shared module would run publish targets
  for release sites: not wrong, since a target that does not compile the site cannot be affected by poisoning
  it, but it would triple the sweep and it would stop being a statement about who can observe the site.

### D6: One `Refusal`, at the test root, not nested `#[path]`

`Kind`, `Refusal`, the constructors, the injection point and the recording go to
`crates/tianheng/tests/support/refusal.rs`. The three test roots declare `#[path = "support/refusal.rs"] mod
refusal;` alongside their gate module, and the gate modules refer to `crate::refusal::*`. No support file
includes another today, and a nested `#[path]` is the resolution shape this repository has a whole capability
about; keeping the inclusion flat avoids it.

This also collapses the twin definition that `publish_source_gate.rs`'s own header calls "the twin-drift class
this repository keeps closing".

### D7: The reaction's own falsifiers

A reaction asserting agreement needs a shown disagreement, or its silence says nothing. The controls:

1. **Per-site control** — the target set runs clean *unpoisoned* before any site is poisoned, so a failure
   under poison is attributable to the poison and not to a test that fails on its own.
2. **Injection is wired** — one run with `TIANHENG_REFUSAL_MUTANT=ALL:kind` must **fail**. If it passes, the
   injection point is not reached at all and every per-site verdict below is meaningless.
3. **Injection is not spurious** — one run naming a `file:line` no site occupies must be **green**. If it
   fails, the poison is firing where it was not aimed and no per-site attribution holds.
4. **The classifier discriminates** — a fixture refusal no assertion distinguishes must classify as
   undefended, *and* the same fixture with a distinguishing direction must classify as defended.
5. **The propagation chain is intact** — D2's location-distinctness direction.

**Each guard is falsified by its own defect, not by one blanket perturbation.** Disabling the injection point
kills guard 2 and leaves 3 passing (a poison that never fires is exactly a poison that does not fire where it
was not aimed) and can leave 4 passing too. A single negative run that only one guard notices would report the
other guards as exercised when nothing tested them — the reads-as-coverage failure, one level up, inside the
reaction built to end it. So:

| Guard | The defect it is run against |
|---|---|
| 2 — injection wired | the injection point disabled |
| 3 — not spurious | the site-match condition forced true, so the poison fires everywhere |
| 4 — classifier discriminates | its own pair: the undefended fixture must be named, the defended one must not |
| 5 — propagation chain | `#[track_caller]` removed from a fixture constructor |

Each of these is run once, its failure observed, and that observation recorded in the change — the obligation
`rust-self-governance-gates` already places on every refusal.

### D8: The one way this can lie, and where it is closed

`TIANHENG_REFUSAL_MUTANT` weakens a gate rather than adding a check, and `publish_source` stands in front of
`cargo publish`, which is irreversible. A stray variable in an operator's environment could poison the one site
that would have fired.

It is closed where the irreversible act is actually launched: `scripts/publish.sh` invokes the gate under
`env -u TIANHENG_REFUSAL_MUTANT -u TIANHENG_REFUSAL_RECORD -u TIANHENG_REFUSAL_BITES`. All three are
instrumentation and none of them belongs in a publish run — `RECORD` writes a file from inside the gate, which
is not a lie but is still the reaction's machinery running where only its verdict was asked for. The
environment is scrubbed at the one point
that matters rather than guarded by a check inside the thing being poisoned. The variables are also only ever
read by test-support code, which ships in no package.

### D9: Gated, and named where the run is decided

`TIANHENG_REFUSAL_BITES`, on its own line in the `AGENTS.md` Definition of Done and in CI, exactly as the
mutation suite and the examples suite are. Sixty-five seconds is cheap against a build but not against an
ordinary `cargo test --workspace`, and it is the class of thing that must run before something is called done
rather than on every save.

## Risks / Trade-offs

- **The undefended population is measured, not known** → 24 is the floor under "either perturbation"; under
  "both" it can only be larger, up to 60. The first run of the reaction produces the number, and the scope of
  the direction-writing follows from that measurement. Estimating it here and then writing to the estimate is
  the failure mode this repository has recorded twice.
- **Exemptions become the escape hatch** → an exemption is a compile-checked slug, joined to one declared
  bound, and its count is a census produced by the reaction, so growth is visible in a document rather than
  invisible in a habit. A slug whose site becomes reachable fails.
- **The recording file is written from concurrently running tests** → one file per target run so no two
  processes share it, appends serialised behind a mutex within the process, and a failed write or an
  unparseable line panics. A lost line does **not** reliably show up as red: for a site that is a declared
  exemption it lands in the legal "declared, unreached" class and the run goes green. Recording integrity is
  load-bearing for a verdict class, so it fails loudly rather than degrading.
- **A direction might fail under poison for an unrelated reason** → the unpoisoned control run must be green
  first, and the reaction requires the poisoned run to have executed at least one test rather than merely
  having exited non-zero.
- **A future gate adds a third kind, or a `Refusal` literal** → both are checked statically by the reaction
  itself, so either fails rather than shrinking the enumeration.
- **65 seconds is a floor that grows with sites** → it grows linearly and with no rebuild, so a gate twice the
  size costs two minutes. If it ever stops being affordable, the lever is running only the target that can
  observe each site, which the design already does.
