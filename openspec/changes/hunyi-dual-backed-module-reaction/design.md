## Context

`locate_module_file(child_dir, seg)` (`crates/hunyi/src/module_resolve.rs:309`) answers "which file
backs `mod seg;`" by probing the two conventional forms in order and returning the first hit:

```rust
let flat = child_dir.join(format!("{seg}.rs"));
if flat.is_file() { return Some(flat); }
let nested = child_dir.join(seg).join("mod.rs");
if nested.is_file() { return Some(nested); }
None
```

Its input space has four states, but the return type (`Option<PathBuf>`) can express only three. The
fourth — both forms present, a rustc E0761 compile error — collapses into `One(flat)`.

Two call sites consume it, both already in `Result<_, String>` context, both already tolerating a
cfg-gated absent file and erroring on an unconditional one:

- `module_resolve.rs:272` — `descend`, the single-module **anchor** resolution behind
  `resolve_module_items_with_files` (signature-coupling, visibility, dyn/impl-trait, async-exposure).
- `scan.rs:348` — `resolve_child_modules`, the **crate-wide** walk behind the whole-crate
  capabilities (trait-impl locality, unsafe confinement, forbidden marker).

圭表 (`module_scan/reachability/walk.rs:234`) and 漏刻 (`audit/scan.rs:239`) each independently
detect the four-state distinction and error. 渾儀 is the only dimension that does not.

## Goals / Non-Goals

**Goals:**
- Make the ambiguous state representable, and react to it with a constitution error at both call
  sites.
- Word the error to name both resolved paths and the exactly-one-file rule, without claiming parity
  with either sibling's existing text (Decision 4).
- Pin the three-dimension agreement in `dual_backed_module_conformance.rs`, exhaustively over the
  four states.

**Non-Goals:**
- Extracting the shared convention into `xingbiao` (see Decision 3).
- Promising byte-identical error text across dimensions (see Decision 4).
- Evaluating `cfg` predicates. Two present files are an ambiguity under every predicate value.
- Documenting 圭表's own already-shipped reaction in `module-boundary`'s spec (see Risks).

## Decisions

### Decision 1: Make the four states representable, keep the lookup pure

Replace `Option<PathBuf>` with a three-variant crate-internal enum rather than threading an error
out of the lookup:

```rust
pub(crate) enum ModuleFile {
    /// Neither conventional form exists.
    Absent,
    /// Exactly one conventional form exists.
    One(PathBuf),
    /// Both forms exist at once — a rustc E0761 ambiguity.
    Ambiguous { flat: PathBuf, nested: PathBuf },
}
```

The lookup stays a pure filesystem question and names no module path, so each call site builds the
error from **its own** module identity (`module` in `descend`, `child_module` in
`resolve_child_modules`) — the identity each site already has, and already uses for its
missing-file and cycle errors. Returning `Result<_, String>` from the lookup instead would force it
to take a module path and a crate name purely to phrase an error, coupling a filesystem probe to
diagnostic construction.

### Decision 2: `Ambiguous` bypasses cfg-tolerance at both sites

Both call sites currently tolerate `Absent` when the declaration carries a `#[cfg]`/`#[cfg_attr]`
gate. `Ambiguous` MUST NOT route through that tolerance: cfg-tolerance exists because a predicate
that is off legitimately leaves a module with no file, whereas no predicate value makes two present
files compile as one module.

This is genuine parity, verified in both siblings rather than inferred: 圭表's
`resolve_plain_sources` tests `flat.is_file() && nested.is_file()` and returns `Err` **before** its
`has_bare_cfg` guard, which covers only the neither-present case
(`crates/guibiao/src/module_scan/reachability/walk.rs:234`); 漏刻's `resolve_external_module` returns
`Err` from its `(true, true)` match arm before any cfg handling, and its spec states the ambiguity
"remains a constitution error regardless of any gate"
(`crates/louke/src/audit/scan.rs:239`, `openspec/specs/runtime-origin-assertion/spec.md:267`).

The consequence must be stated plainly rather than discovered: a **gated-off** dual-backed
declaration is stripped by rustc before module resolution, so it raises no E0761 and the crate
compiles. All three dimensions nevertheless refuse to judge it. That is the correct call for cfg-blind
observation — it cannot know which arm is live, and treating one arm's ambiguity as resolvable would
require evaluating `cfg` — but it means the reaction is **not** confined to uncompilable source.

### Decision 3: Do not sink the convention into 星表

The obvious tidying — extract `locate_module_file` into `xingbiao` so all three dimensions share one
implementation — is declined:

- The 0.2.3 precedent for this exact shape did **not** sink anything. 圭表 gained its own independent
  check and the conformance ledger pinned the agreement. Sinking would be a departure from the
  established pattern, needing its own justification.
- The ledger already **is** the drift reaction here. Its coverage claim must be stated honestly,
  though: the four-state exhaustiveness is a property of `locate_module_file` itself, not of the
  ledger, which exercises the dimensions through their public surfaces where `#[cfg]`, `#[path]`, and
  nesting interact — a larger space it samples rather than exhausts. Sinking would buy unconditional
  agreement across that larger space; what it buys over an exhaustive four-state ledger on the
  lookup's own decision is nothing. This is the weakest of the four reasons here, not the strongest.
- The genuinely shared kernel is the four-state decision (~8 lines). Everything around it diverges:
  圭表 tracks source positions and `#[path]` targets, 渾儀 returns `own_dir`/`path_base`, 漏刻 owns
  its own error type. Extraction would leave the divergent parts in place and add a permanent public
  API to a published crate.
- 星表 is a *table* — declared workspace data and the IO that reads it. Admitting arbitrary shared
  machinery is the god-crate failure mode the crate family split exists to avoid.

### Decision 4: The ledger pins exit codes, not wording

`Outcome::ConstitutionError` carries a `String`, and each dimension's error builders are
`pub(crate)`. Cross-dimension wording agreement is therefore claimable in doc comments but
verifiable only black-box through fixtures — and `errors_conformance.rs` already records one twin it
could not pin, because reaching fixtures kept landing on unrelated per-dimension errors.

There is nothing to be a twin *of*, besides: the two existing messages already diverge three ways.

```
圭表  module '{child_path}' resolves to both '{}' and '{}' — a plain `mod {child}` must be backed by exactly one file
漏刻  module `{name}` resolves to both '{}' and '{}'
```

Single-quoted full module path vs backticked bare name; a trailing rule clause present in one and
absent in the other. Declaring 渾儀's message a "parallel twin" of either would be picking a side in
an existing divergence while using the language of agreement. It is worded well on its own terms
instead. This change pins **"all three dimensions refuse to judge with exit 2"** and claims nothing
about text. Wording parity — including whether the existing two should converge — stays
`errors_conformance.rs`'s concern.

### Decision 5: ADDED, not MODIFIED, on `Anchor resolution`

The existing `Requirement: Anchor resolution` governs an anchor that **cannot be resolved**. A
dual-backed anchor resolves — ambiguously. That is a new concern rather than changed behavior, which
is the documented condition for `ADDED`; `MODIFIED` would require restating a ~60-line requirement
with nine scenarios, whose partial restatement is the documented way to lose detail at archive time.

## Risks / Trade-offs

- **[Risk] The crate-wide call site has a wider blast radius than the headline.** `descend` aborts
  only a boundary whose anchor is dual-backed; `resolve_child_modules` walks every module, so a
  dual-backed module **anywhere** in the crate will now exit 2 even when the boundary anchors
  elsewhere. This is deliberate and matches 圭表, whose `resolve_plain_sources` likewise returns `Err`
  out of the whole reachability walk rather than skipping one module — verified, not assumed. Stated
  rather than discovered, since the requirement's own scenarios must say so.
- **[Trade-off] Adopter-facing effect is a new exit 2, not a new violation.** A constitution error
  never enters a baseline, so an adopter cannot absorb it — the scan refuses to judge. Measured today,
  a standalone 渾儀 consumer on a dual-backed crate gets exit 0 or exit 1 depending on which file the
  violation sits in; afterwards it gets exit 2 in both cases. Per Decision 2 the reaction is **not**
  confined to uncompilable source: a `#[cfg]`-gated-off dual-backed declaration compiles and still
  reacts. That widens the reachable population beyond the E0761 case, which is the honest reason the
  change is worth making rather than recording as accepted debt — and it is also the sharpest thing a
  reviewer should weigh, since it is the only way a *working* build newly stops being judged.
- **[Closed] No existing fixture or example is dual-backed.** Checked across `crates/` and
  `examples/`: no directory `x/mod.rs` sits beside a sibling `x.rs` anywhere in the repo, so the
  Definition of Done should not require a fixture edit. If one turns out to be needed, that is a
  finding to report rather than quietly fix.
- **[Declined scope] 圭表's own reaction has no spec requirement.** `module-boundary/spec.md` never
  states the dual-backed rule, although 圭表 has implemented and tested it since 0.2.3 — only
  漏刻's spec states it. Documenting shipped 圭表 behavior is independently valuable and independently
  reviewable, and no honest `fix(hunyi)` squash subject covers it, so it stays out of this change.
  It is raised for a separate `docs/` branch or a `BACKLOG.md` entry rather than silently absorbed
  or silently dropped.
