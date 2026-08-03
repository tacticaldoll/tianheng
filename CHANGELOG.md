# Changelog

All notable changes to the 天衡 (Tianheng) crate family. This is the **adopter-facing**
projection of the release history; the per-change *why* lives in the squashed change commits and
their pull requests. The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).

Versioning is **SemVer honesty** for a pre-1.0 line (see `AGENTS.md`): the family is
**experimental / pre-1.0**. It held at `0.1.x` deliberately until real adopters arrived; `0.2.0` is
the first deliberate minor past that hold. Pre-1.0, additive depth on an existing observation source
and packaging/hygiene are patch releases, a breaking change earns a minor, and no release
intentionally breaks the adopter-written builder (`Constitution` / boundary DSL / `run`).

## [Unreleased]

### Documentation
- Stopped tracking `examples/capability-catalog/Cargo.lock`, the only committed example lockfile.
  `.gitignore` had matched it since the examples were introduced — tracking simply overrode the rule —
  and the rule's own comment stated that the examples "carry no lock". It pinned all six family
  crates to the published 0.3.0, so an example run without the examples gate's local-source patch
  demonstrated a release other than the tree it was cloned from, and no gate could notice because
  the gate patches resolution to local source. The same comment also still described the examples'
  dependency form as `= "0.1"` when all six commit `= "0.3"`; it now points at each example's own
  manifest instead of naming the version a second place.
- Replaced the root manifest's per-package `exclude` list with the two directory prefixes that
  contain them (`crates/tianheng/tests/fixtures`, `examples`). The list had drifted to 3 of 5
  fixtures and 3 of 6 examples while its comments claimed to cover all of both. No membership change
  (`cargo metadata` reports the same six members): what actually keeps a fixture's deliberate faults
  out of this workspace is each one's own `[workspace]` table, with `members` an explicit
  glob-free list. The exclusion is the second line of that defence, for a future fixture or example
  added without its own `[workspace]` — verified load-bearing by adding one and observing cargo
  operate inside it when excluded and refuse when not — and it now covers all eleven rather than six,
  as a prefix that cannot fall behind what it contains.
- Derived the branch role from the Conventional Commit type instead of an enumerated prefix list.
  `AGENTS.md` listed `refactor/` and `docs/` but not `fix/` or `test/`, both long-established, and
  declared a `polish/X.Y.Z/<slug>` role no release has ever used — the contributor rule most likely
  to be read first was the one least matching practice. Outside `change/` and `release/`, a branch is
  now `<type>/<scope>-<slug>` for the type its work lands as, so branch role and squash subject cannot
  disagree and the rule cannot rot the way a blessed-prefix list did. Pre-release polish takes the
  type its own work lands as; the unused release-staging role is gone.
- Ignored `.github/skills/`, openspec's per-clone generated skills directory, alongside its
  already-ignored `.github/prompts/` sibling.
- Specified 圭表's plain-`mod` conventional-file resolution outcomes in `module-boundary`, which had
  shipped and been tested since 0.2.3 with no requirement of their own: both forms present is an
  ambiguity constitution error (ahead of the absent-file tolerance, so a `#[cfg]`-gated-off
  declaration still reacts even though the crate compiles), an unconditionally absent file is a
  constitution error, and a bare `#[cfg]` tolerates absence. No behavior change — the requirement
  truth catches up to the reaction. (A `#[cfg_attr]` wrapper's own tolerance is specified by the
  fix below in this same window, so it is stated there instead of restated here.)

### Changed
- **BREAKING**: renamed 渾儀's `SemanticBoundary` (the signature-coupling DSL's boundary type,
  `dsl/signature.rs`) to `SignatureBoundary`, along with its draft chain
  (`SemanticCrateDraft`/`SemanticModuleDraft`/`SemanticBoundaryDraft` →
  `SignatureCrateDraft`/`SignatureModuleDraft`/`SignatureBoundaryDraft`). `SemanticBoundary` read as
  if it were the DSL's umbrella type, unlike its 7 siblings (`AsyncExposureBoundary`,
  `DynTraitBoundary`, `ForbiddenMarkerBoundary`, `ImplTraitBoundary`, `TraitImplBoundary`,
  `UnsafeBoundary`, `VisibilityBoundary`), which all name their own capability. No behavior, rule
  string, JSON wire, or CLI change — only the Rust type names an adopter's `Constitution`
  construction code references. `SemanticBoundaries` (the per-dimension aggregate struct
  `hunyi::SemanticBoundaries` holding one `Vec` per capability) and every `semantic_*` dimension
  label are unrelated and unchanged.
- Test-only, no production code change: 渾儀's `every_public_seam_shape_is_named_and_identity_injective`
  now derives its coverage check from an exhaustive `seam_kind` match over `PublicSeam` instead of
  comparing the hand-written fixture's length against itself. `published_seam_fields` and
  `assert_semantic_fact_is_cataloged` already forced a new `PublicSeam` variant to gain a schema arm
  (a compile error otherwise), but nothing forced an *instance* of it into the fixture — the old
  `keys.len() == seams.len()` check would have stayed green even with a variant silently unrepresented.
  `seam_kind`'s own exhaustive match now fails to compile on a new variant too, and the fixture's
  distinct-kind count is asserted against it directly. Coverage was already complete (all 11 kinds
  present); only the enforcement was hand-maintained.
- Test-only follow-up, no production code change: that same coverage check no longer rests on a
  hand-maintained integer. `PUBLIC_SEAM_KIND_COUNT: usize = 11` sat beside the `seam_kind` mapping
  while the fixture it described sat a hundred lines below, and the compiler forced only the match
  arm — so adding a `PublicSeam` variant and its arm while forgetting both the integer and the
  fixture representative left the check green with the new shape uncovered, the same
  hand-maintained-enforcement gap the entry above set out to close, one link further along.
  `seam_kind` now returns a closed `SeamKind` enum whose shapes are compared **as a set** against
  `SeamKind::ALL`, so a missing representative fails by name rather than by two integers differing,
  and the shape-to-published-label mapping is asserted to be a bijection against
  `published_seam_fields` — production schema truth — so a new variant folded into an existing shape
  cannot read as already covered. That bijection is checked in **both** directions, because neither
  count catches the other's failure: with every shape represented, the distinct (shape, label) pair
  count rises above the shape count only when one shape is published under two labels, while two
  shapes *sharing* a label leaves it untouched and is caught by the distinct label count instead.
  Verified by three probes: adding a twelfth `PublicSeam` variant (the compiler demands arms at four
  sites, and with those satisfied but the fixture entry omitted the check fails naming the shape,
  where the previous version passed), publishing one shape under two labels, and publishing two
  shapes under one label — each failing on its own assertion.
- Internal refactor: 渾儀's three call sites that compose transparent-macro flattening with
  const/fn-body-nested-impl recovery (`scan::flatten_for_walk`,
  `module_resolve::resolve_module_items_with_files`, `module_resolve::resolve_module_items_with_cfg_tags`)
  now share one crate-private helper, `syn_util::flatten_with_body_nested_impls`, instead of each
  hand-composing the identical sequence. No public API, wire format, or observable behavior change.
- Internal refactor: 渾儀's four call sites that guard a forbidden-operand list against a malformed
  `::`-path entry (`exposure.rs`, `forbidden_marker.rs`, `shape_scan.rs`, `impl_trait.rs`) now share
  one crate-private helper, `resolve::validate_path_operands`, instead of each repeating the
  identical inline check. No public API, wire format, or observable behavior change at these four
  sites.
- Internal refactor, no public API/wire/behavior change: a large structural-clarity and
  deduplication pass across 圭表, 渾儀, 漏刻, 星表, and 璇璣 — splitting oversized functions
  (`module_check::check_module_boundary`, `module_scan/lexer.rs`'s comment/string skipper,
  `module_resolve::descend`, `scan::resolve_child_modules`/`walk_module`,
  `exposure::module_findings`, `forbidden_marker_findings`, `finding::into_finding_with_text`,
  `audit/scan.rs`'s `collect_scope_modules`/`fn_scopes`, `audit::audit_probe_coverage_with_markers`,
  `runner::dispatch`) and factoring repeated shapes into shared helpers (a `boundary_common!` macro
  for the 8 rule-DSL files' identical accessors, a shared `ViolationContext`/`push_violation`, a
  `CapabilitySet` trait replacing 3 independent per-capability enumerations, `delimiter_group_end`
  for 3 near-identical balanced-delimiter scanners, `Violation::is_active_enforce`, and several
  smaller extractions). No test count changed in any crate.
- Internal refactor: 圭表's `cargo_metadata.rs::matching_dependency_edges` now delegates to
  `governed_dependencies(package, kind, true)` plus its own name filter, instead of hand-repeating
  the identical `kind_matches`/`!is_self_dependency` conjunction `governed_dependencies` already
  encapsulates. No public API, wire format, or observable behavior change.
- Internal refactor: 渾儀's `resolve_direct_path_child`/`resolve_conventional_child`
  (`module_resolve.rs`) now share `load_child_file` for the canonicalize → descent-path
  cycle-check → crate-wide dedup-guard → `read_parse` sequence 3 near-identical call sites
  repeated; each caller keeps its own distinct child-directory/tuple-assembly logic, which
  genuinely differs per call site. No public API, wire format, or observable behavior change.
- Internal refactor: 漏刻's `audit/scan.rs::collect_directory_probes` now reads and scans a source
  file through a new `scan_rust_file` helper instead of an inline `read_to_string` call in its
  recursive dispatch loop — the same I/O-isolation shape `read_dir_entries_sorted` already applies
  to directory listing, one level deeper. `scan_rust_file` also dedupes an identical read+scan
  sequence in `collect_reachable_probes` (the file-input mode's reachable-module walker), which
  needs the read source text back afterward to walk the file's own further module references, so
  the helper returns it rather than discarding it. No public API, wire format, or observable
  behavior change.

### Fixed
- A baseline write now flushes its bytes to stable storage before reporting success, closing the one
  gap between the overwrite path's documented crash guarantee and what it implemented. Temp-then-
  rename made the swap atomic *for other observers*, but `rename` orders only the directory entry —
  never the temp file's still-dirty data pages — so a crash shortly after a successful rename could
  leave the baseline path present and **empty**, losing both the previous document and the
  owner/tracker annotations just merged into it, which no rerun can reconstruct. The overwrite path
  now fsyncs the staged temp file before the rename, and the create path fsyncs its file before
  reporting the write. ext4's `auto_da_alloc` heuristic happens to mask this for the
  replace-via-rename pattern, but it is disabled by `noauto_da_alloc` and absent on other
  filesystems, and this crate ships to adopters on filesystems it does not choose. Each path
  additionally *attempts* to flush the containing directory, so the published name survives a crash
  too — but that attempt is best-effort and never fails the write: it strengthens a write that has
  already landed, and it can be unavailable for capability reasons rather than storage faults (a
  directory that is writable but not readable answers `EACCES` to the open; some FUSE and network
  mounts answer `EINVAL`/`ENOSYS` to the fsync), where reporting "cannot write baseline" for a
  baseline sitting correctly on disk would be the worse outcome. The strict guarantee is therefore
  the file flush; the directory flush is unix-only besides (`std` exposes no portable way to open a
  directory handle on Windows). `create_baseline_file`'s own doc no longer claims a crash there
  "simply leaves no file": it publishes its directory entry before its first byte, so a crash
  mid-create can leave an empty file that the next run refuses to overwrite (exit 2, remedy named) —
  stated instead of overclaimed. `violation-baseline` gains the requirement and its three scenarios,
  and the tolerance is pinned by a test that first proves its own
  precondition — a directory that is genuinely unreadable to the running process — so it reports a
  vacuous run instead of passing through one.
- A value-taking flag (`--manifest-path`, `--baseline`, `--write-baseline`, `--format`) whose next
  argument is itself a `--`-prefixed flag is now a usage error that exits 2 and names the token
  found, instead of consuming that flag as its value. The absent-value case already failed loud; the
  value-is-a-flag case silently ate the following flag. For `--write-baseline` that reached a silent
  *success*: `check --manifest-path <ws> --write-baseline --warn-uncovered` wrote a baseline file
  literally named `--warn-uncovered` into the working directory and exited **0**, with
  `--warn-uncovered` dropped and no diagnostic — a misconfiguration passing as clean, which
  PROJECT.md forbids. The other three flags did reach a non-zero exit, but reported it as a
  downstream unreadable path or unknown format rather than as the malformed invocation it was. The
  `--flag=<value>` form is unchanged and remains the way to pass a value that legitimately begins
  with `--`, since it carries its value in the same token; that escape hatch is pinned end-to-end,
  distinguishing the two forms by their diagnostics rather than by an exit code they share.
  `cli-check-runner` gains the requirement and its two scenarios.
- **BREAKING**: 圭表's inbound module-boundary rules (`must_not_be_imported_by`,
  `must_only_be_imported_by`) now react to an item-form import (`use m::Item;`) of the anchored
  module under `ScanDepth::Shallow`, not only a bare import of the module itself. The Shallow
  target match compared the import's full path string — including any item leaf — directly against
  the anchored module, so `within_scan_depth("crate::internal::Secret", "crate::internal",
  Shallow)` demanded exact string equality and silently failed: a real, released-since-0.3.0 false
  negative in exactly the boundary PROJECT.md's core contract forbids reacting silently in. The
  same import in bare-module form (`use crate::internal;`), and the identical item-form import
  under `ScanDepth::Subtree`, both already reacted correctly — only the Shallow + item-import cell
  was silent. Fixed by resolving the import path to the module it actually denotes (itself, or its
  longest reachable-module prefix) before the depth comparison, using the same reachable-module
  set `ScanContext` already carries — so an item declared directly in the anchored module reacts,
  while an item in a descendant module correctly does not. Closing that target-match precision also
  surfaced a latent, adjacent false positive: the importer-side self-import exemption (a file
  within the protected module's own subtree is never an inbound importer) was itself gated to
  `Subtree` only. Fixed alongside it, so the self-import exemption now holds identically at both
  depths, matching `module-boundary`'s own unconditional wording — narrowing to `Shallow` scopes
  what counts as *reaching* the protected module, never who counts as *inside* it. Any existing
  `--write-baseline` output for an inbound rule declared at `ScanDepth::Shallow` may need
  regeneration: an import that previously passed silently may now correctly react.
- 天衡's `--write-baseline` now overwrites an existing, supported baseline durably: the merged
  document is written to a sibling temp path first, then an atomic `rename` swaps it into place,
  instead of a bare truncating write. A crash, interrupt, or full disk mid-write previously left the
  baseline truncated — destroying exactly the owner/tracker annotations the metadata-preserving
  merge exists to carry forward, contradicting the function's own stated intent. The swap targets
  the file's symlink-resolved real path and carries over its existing permissions, set at creation
  rather than narrowed afterward: `rename` unconditionally replaces whatever sits at its
  destination, so writing straight to the given path would otherwise replace a symlinked baseline
  with a plain file (orphaning whatever it pointed at), and creating the temp file at the process
  umask default before narrowing it would briefly widen permissions an adopter had deliberately
  restricted. The temp file itself is opened with `create_new` (`O_EXCL`): its name is predictable
  (`<target>.tmp-<pid>`), so a plain create-or-truncate would follow whatever already sat at that
  path — a symlink included — letting anything pre-planted there redirect the write onto an
  arbitrary file; `create_new` refuses outright instead. Its path is built from the resolved
  target's raw bytes rather than through lossy display formatting, so a target reached through a
  non-UTF-8-named directory component no longer fails the overwrite outright. A stale temp file
  left behind by an interrupted prior run (a killed process, or a pid reused across a fresh
  container) is a real, reachable case `create_new` also reports — now with its own specific
  message naming the actual colliding temp path and explaining why it is there, rather than a bare
  `cannot write baseline <path>: File exists` that names nothing the adopter can act on. A baseline
  path that is a symlink to a deleted target is reported by its own cause too, not misattributed to
  the sibling "it appeared while the new snapshot was being prepared" race message: `O_EXCL` fails
  on a dangling symlink exactly as it does on a genuine concurrent creation, but the two are not the
  same state — a dangling symlink is permanent, so "rerun the command" (that message's own remedy)
  could never have succeeded. The create-new path (writing a baseline where none existed) is
  otherwise unaffected: it has no pre-existing content to protect, and already fails loud rather
  than clobbering if the file appears concurrently.
- Bounded native recursion depth across four recursive walkers in three crates, closing the same
  false-negative-adjacent bug class in every observation dimension — a pathologically (but
  genuinely acyclic) nested module tree, `use` tree, or block/macro-arm structure could overflow
  the native stack (an uncontrolled process abort) instead of the contract's own exit-2 "cannot
  judge". Three of the four (圭表's two, 渾儀's) had an existing cap that silently returned an
  empty/partial result past it instead of erroring — `Outcome::Clean` when a real violation
  exists, the exact false negative PROJECT.md's core contract names as the one forbidden bug; the
  fourth (漏刻's) had no cap at all. Each bound was *measured* against a real crash, never guessed:
  an initial 512 guess for 渾儀's walkers crashed a real test process. The settled bounds: 32
  (渾儀's `walk_module`/`collect_subtree`/`walk_unsafe`, clear of a measured 80–90-level crash line,
  and independently clear of `syn::parse_file`'s own ~300–350-level parser-recursion crash line on
  the same fixture shape); 128 (圭表's `use_scan::expand_use_tree_depth`); 64 (圭表's
  `symbol_scan.rs`'s `glob_bases` and `expand_use_leaves`'s inner `go`, feeding the glob-hazard
  pass and alias resolution for `ConfineInlineSymbolPath` — the identical silent-truncation shape
  `use_scan` was fixed for, never carried over to this sibling scanner until now); and 300 (漏刻's
  `audit/scan.rs::collect_scope_modules`, which recurses through transparent-macro arms, inline
  `mod` bodies, and arbitrary blocks — measured safe at depth 1100 and a reproducible SIGABRT at
  depth 1105+ under a 2MB test-thread stack). Each fix added tests proving both directions:
  nesting comfortably under the bound is still fully observed, and nesting past it is a scan
  error, never a crash or a silent pass.
- **BREAKING**: `PublicSeam::InherentMethod`/`InherentAssoc` now carry the impl **block's own**
  declaring module, distinct from the self type's canonical `owner` path. `owner` names what the
  impl is *for*, not where it is *written* — Rust's coherence rules let an inherent `impl` for one
  type be written in any module of the same crate, a real, common platform-conditional idiom
  (`impl Conn { … }` once in `plat_unix`, once in `plat_win`, both for a `Conn` declared in
  `common`). Two such impl blocks declaring a same-named public method/associated item previously
  resolved to the identical `{owner, name}` seam and collapsed to one violation: measured on the
  real `hunyi::check_impl_trait`'s `including_submodules()` subtree scan, the second module's real
  violation was silently dropped by the fact-only dedup, not merely deduplicated against an
  equivalent finding — the false negative PROJECT.md's Core Contract forbids outright. dyn-trait and
  signature-coupling build the identical seam through the same constructors but cannot currently
  observe more than one module per evaluation, so they close the identical structural gap
  pre-emptively rather than a second live false negative. **Any existing `--write-baseline` output
  for an `InherentMethod`/`InherentAssoc`-seam finding is now stale** (the fact gained a required
  field) and must be regenerated; every previously accepted violation reappears as new exactly once.
  Rendered finding text is unchanged (the module is identity-only, matching
  `AsyncInherentMethod`'s own already-shipped precedent). No DSL, builder, or CLI surface change —
  only the identity `fact` payload gains a field, the identical shape this same `[Unreleased]`
  window's own `governing_package` fix already took (below).
- 渾儀 now rejects a forbidden/allowed operand whose `::`-delimited spelling has an empty segment
  (a leading `::`, a trailing `::`, a doubled `::`, or the empty string) as a constitution error,
  across `must_not_expose`/`and_not_expose`, `must_not_expose_dyn_of`, `must_not_expose_impl_trait_of`
  (module- and subtree-scoped alike), and `must_not_acquire`/`and_not_acquire`. `extern_verbatim_renamed`
  builds a resolved canonical path purely from `syn::Path` segments — it never carries a leading `::`
  regardless of how the scanned source is spelled — so an operand like `must_not_expose("::serde")`
  could never equal or prefix-contain a real resolved path and silently, permanently never reacted;
  `must_not_acquire`'s leaf-identifier matching has the mirror-image gap for a trailing `::`. No
  existing usage in this repo used the malformed spelling, so this is a strict tightening of an
  already-inert configuration, not an adopter-visible behavior change for any working boundary.
- 渾儀's `only_implemented_in`/`and_in` (`allowed_locations`) and `only_under` (unsafe-confinement's
  own `allowed_locations`) now reject the identical malformed `::`-path shape as the forbidden-operand
  fix above, sharing its guard (`resolve::validate_path_operands`). Unlike the forbidden-operand
  direction, the previous behavior here already failed loud rather than silently passing — a
  malformed allowed entry never matched any real module location in `matches_allowed`, so a
  genuinely-in-place impl or `unsafe` site was reported as a spurious violation instead of a
  named constitution error. No existing usage in this repo used the malformed spelling, so this
  is a diagnosis improvement on an already-broken configuration, not an adopter-visible behavior
  change for any working boundary.
- 漏刻's CI probe audit now reads the arms of a `cfg_if!` invocation as real code, in both of its
  passes, completing the family: all three dimensions now share one transparency rule and are pinned
  on one shared fixture (`cfg_if_transparency_conformance.rs`). Skipping such a body like any foreign
  macro broke two of the audit's three reaction directions, in both error directions at once. **Two
  false negatives close:** a probe naming a mis-typed seam inside an arm escaped the
  probed-but-undeclared reaction entirely (at runtime it asserts against a seam nobody declared), and
  an un-auditable probe (a non-literal seam argument) inside an arm was silently skipped —
  contradicting `audit_probe_coverage`'s own documented promise that a silent skip never happens. **A
  false alarm retires:** a seam whose only production probe lived inside an arm was reported unprobed,
  failing an adopter's CI over coverage they actually had; the same held for every probe beneath a
  `mod` declared only inside an arm, since that module never entered the reachable corpus. An
  arm-declared module is now also treated as cfg-conditional, so an absent conventional file is
  tolerated exactly as under a bare `#[cfg]` (圭表's rule, adopted), while a resolution ambiguity stays
  a constitution error under every gate. Bounds unchanged and now uniform across the three: only
  `cfg_if` is transparent, and observation stays cfg-blind. A newly caught typo'd or un-auditable probe
  is a real finding and absorbable by baseline.
- 渾儀 now reads the arms of a `cfg_if!` invocation as real code, in every walk it performs. Closes an
  exposure false negative measured on ordinary, compilable source: a `pub fn` returning a forbidden
  type reacted at a module's top level and **passed** when the identical function sat inside a
  `cfg_if!` arm, because `syn` parses the invocation as one opaque macro item and no capability
  handled that variant. A `mod` declared only inside an arm was equally invisible, so its file's
  `unsafe` sites, forbidden markers, trait impls, and re-exports went unobserved with it, and the
  module could not be named as an anchor at all. 圭表 has read these bodies since 0.2.3, so an adopter
  using `cfg_if!` was already seeing the static half of these findings — this adds the semantic half,
  and the two dimensions are now pinned on one shared fixture
  (`cfg_if_transparency_conformance.rs`). Three properties come with it, stated rather than implied:
  an arm-declared module is cfg-conditional, so an absent conventional file is tolerated exactly as
  under a bare `#[cfg]` (圭表's rule, adopted); both conventional forms present is still an ambiguity
  constitution error under arm membership; and arms are unioned **cfg-blind**, so a violation in an
  arm this build does not compile still reacts. Only `cfg_if!` is transparent — a body-wrapping macro
  under any other name stays unobserved, which is load-bearing rather than cautious: reading an
  arbitrary macro's braces as arms recovers items from a nested `impl` block that the macro may never
  emit, a false positive. Transparency also covers **item position** only: an invocation written
  inside an `impl` or `trait` body still goes unobserved, a measured gap left stated and owned by its
  own change rather than half-closed here. New violations are ordinary findings and absorbable by
  baseline.
- 圭表 now treats a `mod` declared inside a `cfg_if!` arm as cfg-conditional, so an absent
  conventional file (or an absent unconditional `#[path]` target) is tolerated exactly as it already
  is for a bare `#[cfg]`-gated declaration. Completes the 0.2.3 transparency carve-out, which made arm
  bodies observable but left the absent-file tolerance keyed on an attribute preceding the item — a
  `mod` inside an arm carries none, because the predicate lives in the macro's `if #[cfg(..)]` header.
  The two spellings of one per-platform shim therefore gave opposite verdicts: with only one arm's file
  committed, the bare-attribute form exited 0 while the `cfg_if!` form exited 2, reporting the absence
  as unconditional — on source that compiles, since rustc strips the non-selected arm. Adopters who saw
  that exit 2 now get a real verdict, which may surface violations in modules the aborted walk never
  reached and therefore need baselining. Tolerating an absence cannot hide anything: a file that does
  not exist holds no code. An arm module whose file exists is still reached and governed, and both
  conventional forms present at once is still an ambiguity constitution error under every gate.
- 渾儀 now reacts with a constitution error (exit 2) when a plain `mod name;` is backed by BOTH
  conventional forms at once (`name.rs` AND `name/mod.rs`), instead of silently resolving to the
  first form it probes and never reading the other. Closes an exposure false negative: with the two
  files present, moving a forbidden exposure from `name.rs` into `name/mod.rs` turned a reaction into
  a clean pass, so whether the module was governed at all depended on which file its author wrote the
  item in. 圭表 and 漏刻 have both reacted to this shape since 0.2.3 and earlier, and the composed
  `tianheng check` therefore already exited 2 on it — the gap was reachable by a **standalone 渾儀**
  consumer. Two trigger shapes, stated plainly: a live declaration of this kind is a rustc compile
  error (E0761), but a `#[cfg]`-gated-off one is stripped before module resolution and **compiles**,
  and it also now reacts, because cfg-blind observation cannot know which arm is live (the ordering
  圭表 and 漏刻 each already apply). A constitution error never enters a baseline; the repair is to
  delete whichever of the two files is not the module. All four outcomes of the lookup are now pinned
  across all three dimensions in `dual_backed_module_conformance.rs`.
- **BREAKING**: 圭表's and 渾儀's violation identity now carries the crate a boundary was declared
  against. Neither dimension's fact construction previously named the declaring crate — only a bare
  module path — so two workspace members declaring the identical rule against the identical module
  path collapsed into one `ViolationId`: the composed report silently dropped the second crate's real
  violation, and a baseline accepted for one crate could suppress the other's never-accepted one (the
  false negative PROJECT.md's Core Contract forbids outright). Every `ModuleFact` and `SemanticFact`
  variant now carries a `governing_package` identity field equal to the boundary's declared crate
  (`unsafe_confinement` excepted — its identity already varies by crate through `target`). **Any
  existing `--write-baseline` output for a module or semantic boundary is now stale** (identity
  gained a required field) and must be regenerated; every previously accepted violation reappears as
  new exactly once. No DSL, builder, or CLI surface change — only the identity `fact` payload gains a
  field.
- 圭表's lexical hygiene no longer panics on a governed source file ending in an unterminated block
  comment that swallows a multi-byte UTF-8 character. The comment-stripping pass could leave exactly
  one trailing byte unconsumed when a comment never closed before EOF; if that byte was the orphaned
  tail of a multi-byte character whose lead byte(s) were already dropped inside the comment, it was
  re-scanned as code and pushed alone into the stripped buffer — an invalid UTF-8 fragment that
  `String::from_utf8_lossy` then lengthened (one byte becomes the 3-byte U+FFFD replacement),
  desynchronizing the position map from the string it indexes into and panicking the next stage's
  lookup. An unterminated comment is now treated as extending through end-of-file, so nothing is left
  to re-scan. Not a behavior an adopter could have depended on — a crash is none of PROJECT.md's Core
  Contract outcomes (0 clean / 1 violation / 2 constitution error) — so no **BREAKING** marker.
- 圭表 no longer silently passes a forbidden import when a non-ASCII char literal sits immediately
  adjacent to a `'{'` char literal (e.g. `['«','{']`, no space) — the false negative the Core
  Contract forbids outright. The lexer's "simple char literal" check assumed every char literal's
  payload is exactly one byte, which holds for `'x'` but not a multi-byte UTF-8 scalar (`'«'` is 2
  bytes, `'未'` is 3); for a non-ASCII literal the check failed and the scalar's raw bytes leaked
  into the cleaned text as ordinary code. When a second literal followed closely enough, the misread
  literal's real closing quote, an intervening comma, and the next literal's real opening quote could
  coincidentally match the old one-byte assumption exactly, swallowing that opening quote too — which
  left the next literal's own payload (here, `{`) unprotected, leaking it into the cleaned text as a
  spurious structural brace and throwing off the reachability walker's brace-depth tracking for every
  `mod` declared after it. The check now measures a char literal's real UTF-8 byte length from its
  lead byte rather than assuming one. Not breaking — this closes a false negative against
  `module-boundary`'s already-stated import-detection contract; no baseline identity shape changes.
- 渾儀's signature-coupling query now observes a `pub fn`/`pub static` declared inside an `extern`
  block — the FFI declaration is a real, callable item in the enclosing module's own namespace,
  exactly as public as a same-shaped ordinary item, but the exposure collector had no
  `ForeignMod` handling at all, so a forbidden type named only there escaped the query entirely
  (exit 0 Clean on source with a real, callable public API leak). Reuses the existing seam/path-
  collection machinery verbatim — no new seam kind, since Rust cannot declare both an ordinary item
  and a foreign one under the same name in one module, so there is no identity collision to design
  around. Not breaking — closes a false negative; no baseline identity shape changes.
- 渾儀's visibility-boundary query (`must_not_declare_pub` / `max_visibility`) now observes a `pub
  fn`, `pub static`, or `pub type` declared inside an `extern` block — the sibling gap the
  signature-coupling fix above did not touch, since the two capabilities collect a module's direct
  items through entirely independent per-item logic (`collect_item_exposures` vs.
  `item_observation_parts`), sharing only the underlying module-item enumerator. A bare-`pub` foreign
  item is exactly as visible as a same-shaped ordinary one, but `item_observation_parts` had no
  `ForeignMod` arm at all, so an `extern` block's declarations were silently absent from the
  module's observed direct items regardless of their declared visibility (exit 0 Clean on a module
  whose only bare-`pub` item sat inside an `extern` block). `pub type` (an extern type declaration)
  is included here though it carried no exposable signature and so was out of the
  signature-coupling fix's own scope — this capability cares about the declared keyword, not a
  type-signature leak. Reuses the existing `Fn`/`Static`/`Type` visibility kinds verbatim, no new
  kind, for the identical no-identity-collision reason as the sibling fix. `item_observation`
  widens from `Option` to `Vec` (an `extern` block can hold more than one independently-visible
  foreign item, unlike every other observed item kind), with its one call site updated accordingly.
  Not breaking — closes a false negative; no baseline identity shape changes.
- 渾儀's shared `use`-map and re-export closure no longer silently drop one candidate when two
  mutually-exclusive `#[cfg]` branches (bare `#[cfg]` or `cfg_if!` arms alike) declare `use ... as
  Name;` (or `pub use ... as Name;`) for the identical name with different targets. Both were
  single-valued (`HashMap<String, String>`), so the second declaration always overwrote the first —
  the verdict for a real forbidden-type exposure depended on which mutually-exclusive branch was
  written last, not on whether either branch's binding was genuinely forbidden. Both maps are now
  multi-valued (mirroring the crate's existing type-alias map). Every matcher that consumes them now
  checks every candidate and reacts if any is forbidden, not only signature-coupling's exposure
  resolution and dyn-trait's/impl-trait's shared operand-scoped principal-trait resolver (discovered
  to have the identical gap while fixing this): an adversarial review of the fix itself found the
  same order-dependent silent pass still reachable through forbidden-marker's derive and impl-form
  leaf matching, its self-type/marker-acquisition landing (through a third, previously single-valued
  type-alias map), and trait-impl-locality's anchor resolution — each independently reproduced before
  being closed here too. Not breaking — closes false negatives; no baseline identity shape changes.
- 渾儀's crate-wide scan no longer drops a module reached only through a `cfg_attr`-wrapped `#[path]`
  remap. `cfg_attr` never removes the `mod` item the way a bare `#[cfg]` does, so the module is
  present on every configuration and needs SOME file to back it — treating the attribute as a blanket
  skip bound dropped the whole subtree, not just the alternate target its predicate might select. Two
  shapes: an **inline** module's body is unaffected by `#[path]` at all (rustc ignores it there; the
  body always compiles) and is now always descended; a **file** module's conventional file and its
  `cfg_attr` target are both read when they exist on disk, unioned rather than either being silently
  preferred — matching 圭表's own already-fixed union-scan policy for the identical shape. Neither
  candidate existing, with no other cfg-conditional gate, remains a genuine scan error. Since the
  crate-wide scan backs signature-coupling's own alias/re-export closure and dyn-trait's/impl-trait's
  shared operand-scoped principal-trait resolver, not only forbidden-marker, trait-impl-locality, and
  unsafe-confinement (the two capabilities the discovering findings measured against), and since
  async-exposure's and impl-trait's own subtree-scope opt-in (`including_submodules()`) shares the
  identical walker one hop further out (found on adversarial review), all seven were independently
  reproduced and confirmed fixed by this one change. `module_resolve.rs`'s separate single-module-
  anchor resolution (signature-coupling's own anchor, visibility, dyn-trait's shape-only module-scoped
  resolution, and trait-impl-exposure) gets the identical fix: a third adversarial review disproved
  this change's own earlier claim that the function was "already correct, fails loud" — a mutually-
  exclusive sibling declaration for the same module name silently absorbed the branch count, so the
  `cfg_attr` target's own file vanished with exit 0 whenever ANY sibling resolved, and even a LONE
  such declaration never followed an existing target file at all. Now it does, the same union as the
  crate-wide walk. A fourth review then found one more gap shared by both walkers: a module stacking
  more than one SEPARATE `cfg_attr`-wrapped `#[path]` attribute (one per platform predicate — the
  natural 3+-way per-platform shim) only ever had its first-declared candidate tried; every other
  platform's target silently never was. Every stacked candidate is now read. Not breaking — closes
  false negatives; no baseline identity shape changes.
- 漏刻's CI-face audit scanner no longer drops a module whose declaration carries a comment between
  the `mod` keyword and its name (or between the name and its terminator) — trivia to rustc, but a
  bare whitespace-only skip stopped at the comment's leading `/`, so the declaration was never
  recognized as a `mod` at all: the module and its whole subtree, and every probe beneath it,
  silently vanished from the corpus. It also now descends into every function/block/match-arm body
  looking for a nested `mod`, not only the scopes it specifically recognized — the only legal
  non-inline module form there, `#[path] mod name;`, was previously invisible with no loud signal at
  all. And `mod_preamble_attrs`'s documented `cfg_attr(path)` tolerance is now actually implemented:
  the attribute match previously checked for the exact identifier `cfg`, so `cfg_attr` — a different
  identifier — matched neither the `path` arm nor the `cfg` arm, and a module stacking two
  `cfg_attr`-wrapped `#[path]` declarations that together cover every platform (both targets present,
  compiling cleanly everywhere) was reported a hard constitution error instead of being scanned — a
  false positive on entirely valid code. Every `cfg_attr` target that exists on disk is now read,
  unioned with the conventional file, matching the crate-wide walk 圭表 and 渾儀 both already apply to
  the identical shape; the same union now also applies to a `cfg_attr`-wrapped `#[path]` on an
  *inline* `mod x { … }` (governing the base directory x's own nested items resolve from, descended
  only when it exists as a directory, falling back to the conventional base when none does) — an
  adversarial review round found the first cut had wired the union into only the external-`mod x;`
  consumer. A doubly-nested `cfg_attr(cfg_attr(path))` remains a stated, undetected bound of this
  hand-rolled scanner. Not breaking — closes false negatives and one false positive, not an identity
  shape; no baseline identity shape changes.
- 圭表's own module-boundary reachability walk no longer requires a plain conventional file
  (`name.rs` / `name/mod.rs`) for a declaration backed only by one or more `cfg_attr(path)`
  remaps. A resolved `cfg_attr(path)` candidate was already union-scanned for governance, but the
  separate plain-file requirement ran unconditionally regardless: a module stacking two
  `cfg_attr`-wrapped `#[path]` attributes that together cover every platform (both targets
  present, no plain file ever needed) was reported a hard constitution error — "source file could
  not be located" — instead of being governed, a false positive on entirely valid code, and not
  specific to "stacked": a single `cfg_attr(path)` target with no plain fallback hit the same
  error. A resolved candidate is now treated as legitimate grounds for the plain file's own
  absence, the same "might legitimately be absent on this build" signal a bare `#[cfg]` or a
  `cfg_if!` arm already carries — matching 渾儀's/漏刻's own `has_backing_source` rule for the
  identical shape (三儀 ⊥ 三儀: the same rule, not the same function). Two outcomes stay exactly as
  strict as before: both conventional forms present is still an unconditional ambiguity error
  regardless of any resolved candidate, and a declaration whose every candidate is absent (no
  plain file, no resolved `cfg_attr(path)` target, no bare `#[cfg]`) is still a genuine
  constitution error. Not breaking — closes a false negative; no baseline identity shape changes.
- `deny.toml`'s `[advisories]` table now sets `yanked = "deny"` explicitly: the field's own unset
  default is `"warn"`, so `cargo deny check` was printing `warning[yanked]: detected yanked crate`
  and still exiting 0 (`advisories ok`) — reproduced against a real yanked crate pinned into the
  lockfile — directly contradicting the section's own stated claim that yanked crates are denied.
  `scripts/test_examples.sh` now asserts (`cargo tree -p <crate> --depth 0`) that every example's
  `patch.crates-io` override actually resolved to local source, for every family crate it patches:
  reproduced against a version-bumped scratch copy of the workspace, Cargo was silently dropping an
  incompatible patch (`patch ... was not used in the crate graph`) and falling back to the last
  published crate, so the dogfood gate stayed green while silently testing stale, already-published
  code instead of the in-development tree it exists to exercise. Not breaking — strengthens two CI
  gates to enforce what they already claimed; neither the yanked crate nor the incompatible patch is
  present in the current workspace, so this has no effect on the present green build.
- **BREAKING**: 漏刻's un-auditable-probe identity no longer embeds a raw absolute filesystem path.
  Reproduced directly: scanning the byte-identical file at two different absolute locations (the
  same relocation a different clone path or CI runner produces) yielded two DIFFERENT
  `unauditable-probe` identities, differing only in the `file` field's absolute prefix — a baseline
  recorded in one checkout matched nothing in another, so the accepted violation re-fired as new
  while the recorded entry was simultaneously reported stale. `file` is now labeled relative to the
  common ancestor of every `source_inputs` root passed to one `audit_probe_coverage` call (the real
  caller's actual checkout root, by construction — every workspace member's root shares it, whatever
  the invocation's working directory), falling back to the previous absolute form only when no
  shared ancestor exists at all. No public function signature changed. **Any existing
  `--write-baseline` output naming an `unauditable-probe` violation is now stale** (its `file` field's
  value changed shape) and must be regenerated; every previously accepted one reappears as new
  exactly once. Stated bound: an ABSOLUTE `#[path = "/…"]` literal is a known residual gap, not fully
  closed by this fix — when its target does not lie under the scanning checkout's own anchor, the
  label falls back to the raw absolute path (`Path::join` discards its receiver for an absolute
  joinee); when it happens to lie under the anchor, the label becomes relative-looking instead, so
  the SAME hardcoded literal can still disagree across two checkouts. An absolute literal is already
  non-portable on its own; the realistic relative sibling-share idiom this fix targets is
  unaffected either way.
- 渾儀's trait-impl-exposure `where`-clause bounded-type seam no longer keys an unrenderable bound
  (a complex const-generic argument, e.g. `Arr<{ N + 1 }>`) to the bare literal `_`. Reproduced
  directly: one impl block with two where-clause bounds that both fail to render
  (`Arr<{ N + 1 }>: AsRef<crate::infra::Secret>` and `Arr<{ N + 2 }>: AsRef<crate::infra::Secret>`)
  both fell back to `_`, so the two bounds' facts — identical kind, subject, and seam — collapsed
  to one; the two-bound fixture and either bound alone produced the byte-identical finding, meaning
  the second bound's violation left no trace a baseline could ever distinguish from the first. The
  fallback is now an internal positional sentinel (never itself published), caught by the existing
  fail-loud gate this capability already applies to its structurally identical cases (an
  unrenderable Self type or trait path), so an impl with this shape now reports a constitution error
  instead of silently under-counting its violations. A where-clause bound that renders cleanly (the
  ordinary case) is unaffected.
- 渾儀 now observes an `impl` block written as a direct statement of a `const` initializer's or a
  `fn`'s own body — the "const-eval trick" idiom (`const _: () = { impl Foo { … } };`, commonly used
  for a compile-time trait assertion or a doctest/dogfooding scratch impl) and its fn-body-nested
  sibling — instead of treating the whole body as opaque. Closes a false negative measured directly
  on ordinary, compilable source across six capabilities: signature-coupling, async-exposure,
  dyn-trait, and impl-trait all missed an inherent impl's method the moment its enclosing `impl`
  moved into such a body, and trait-impl-locality and forbidden-marker's hand-impl form both missed
  a trait impl the same way — the identical method or impl that reacted at a module's top level
  produced zero findings once wrapped, even though Rust binds an `impl` to its self type's coherence
  set regardless of where it is lexically written, so the wrapped impl was always real, externally
  callable public API. Three bounds are stated rather than left silent: only an `impl` block is
  recovered (a plain `fn`/`struct`/`mod` nested the same way stays exactly as unobserved as the
  existing body-nested-module bound already treats it); only an `impl` that is a DIRECT statement of
  the body's own outermost block is recovered, never one nested a level further (inside an
  `if`/`loop`/closure/nested `fn`); and only a `const` initializer or a `fn` body is inspected, never
  a `static` initializer. New violations are ordinary findings and absorbable by baseline.
- 漏刻's shipped default sink no longer silently discards a failed stderr write. An adopter who
  never calls `set_sink` — the exact adopter the default sink exists for — lost an enforce-severity
  `Violation` with zero trace whenever the write failed (a closed pipe after `myapp 2>&1 | consumer`
  exits, a daemon with closed inherited fds, or plainly `myapp 2>&-`): the process correctly did not
  crash, but nothing recorded that the reaction had even fired. A failed write is now counted by a
  new public `louke::dropped_sink_events() -> u64`, a single lock-free atomic add that cannot itself
  fail or panic, so an adopter can poll it into their own health check or diagnostics endpoint to
  detect the loss from outside the process. Scope stays narrow: a custom sink's own success or
  failure is opaque to the system (`set_sink` takes a `Fn(&Violation)` returning nothing) and is
  never counted. Additive, non-breaking — the only public surface change is the new function.

## [0.3.0] - 2026-07-26

### Documentation
- Archived historical 0.1.x–0.3.0 shipped backlog ledgers into `docs/history/0.1.0-0.3.0-built-ledger.md` and pruned `BACKLOG.md` to optimize context gravity.
- Reconciled the 0.3.0 migration guide with the separately shipped testing harness and restored
  the deferred baseline debt-ratchet WATCH decision after backlog pruning.

### Added
- Semantic `RuleKey` and `StructuredFactIdentity` inspection across 圭表, 渾儀, 漏刻, 璇璣, and
  `tianheng::prelude::*`; all three instruments remain directly adoptable and return the same
  structured reaction model.
- Explicit machine-contract formats: `tianheng.baseline/structured-facts`,
  `tianheng.reaction/structured-facts`, and `tianheng.constitution/declared-boundaries`.
- `tianheng::testing::GovernanceTest`: a reusable fluent architecture-test harness for clean
  reactions, complete workspace-member coverage, projection freshness with explicit
  `BLESS=1`/`BLESS=true` regeneration, and negative fixture checks. Tianheng's own self-law
  dogfoods the same public projection gate.
- `ScanDepth::{Shallow, Subtree}` and explicit `.depth(...)` controls on supporting 圭表 and 渾儀
  boundaries. Legacy module boundaries retain subtree evaluation and identity; shallow scope is
  projected explicitly.
- `check --disallow-stale` turns any stale baseline entry into a gate failure while preserving
  constitution-error precedence and consistent text, JSON, and SARIF exit semantics.
- `ImplTraitBoundary::including_submodules()`: an opt-in subtree scope for the impl-trait
  (existential RPIT) boundary, mirroring `AsyncExposureBoundary`'s existing depth. Defaults off;
  an existing boundary projects and reacts byte-identically.
- `NoExistentialLeak` / `Constitution::no_existential_leak(...)`: a composed profile folding
  impl-trait's written `-> impl Trait` and async-exposure's implicit `impl Future` — the two
  existential-leak signals — into one declaration, mirroring `SansIoPure`. Each composed boundary
  keeps its own separate identity; adds no new reaction.
- `louke::audit_probe_coverage_with_markers(...)`: CI probe-coverage audits can recognize
  adopter-defined probe macro names while `audit_probe_coverage(...)` preserves
  `assert_boundary!` as the compatible default.

### Fixed
- 圭表 now union-scans every physically existing path candidate when one module declaration mixes
  direct `#[path = "…"]` and conditional `cfg_attr(..., path = "…")` remaps, independent of
  attribute order. A candidate selected by a real rustc configuration can no longer disappear
  behind the scanner's former direct-path early return.
- 漏刻's un-auditable-probe identity now includes the full enclosing lexical function chain, so
  byte-identical probes in same-named nested functions or local contexts under different outer
  functions remain distinct and baselining one cannot suppress another.
- Module-boundary constitution projection now omits the legacy/default subtree scan depth and emits
  the non-legacy shallow depth, preserving old projection bytes while exposing the real opt-in.
- 漏刻's un-auditable-probe finding identity is no longer file-granular: it is now qualified by the
  offending non-literal seam expression's own source text and its owner-qualified enclosing item
  (module path plus `fn`/`impl`/`trait` context), so two distinct non-literal probes in the same
  file react as distinct findings and baselining one can no longer mask another. A false-negative
  closure (a patch, per the standing v0.1.3 re-export-exposure precedent): an existing baseline
  with an un-auditable-probe entry goes stale and needs `--write-baseline`, never silently
  reinterpreted. Two byte-identical expressions in the same file and the same enclosing item still
  collapse to one finding — a stated bound.
- 圭表 now preserves `cfg_if!` bodies as transparent control-flow wrappers, so enclosed imports,
  module declarations, and inline symbol calls remain observable instead of being stripped as
  macro-generated code.
- 圭表's `must_not_import` now fails closed on ancestor glob hazards such as `use crate::a::*;`
  when `crate::a::b` is forbidden, while unrelated and non-glob ancestor imports remain clean.
- 渾儀's signature-coupling alias resolver now walks nested nominal targets in non-generic tuple,
  array, slice, reference, raw-pointer, group, and parenthesized aliases.
- 圭表 now normalizes embedded `self` and `super` segments throughout grouped imports and inline
  symbol paths before evaluating module boundaries.

### Changed
- **Breaking:** violation and baseline identity is now exactly governed target + semantic rule key
  + structured fact identity. Rule/finding wording and all diagnostics remain available but cannot
  affect matching, ordering, or SARIF fingerprints.
- **Breaking:** SARIF partial fingerprints now use `tianheng/structured-fact-identity`, derived
  solely from canonical semantic identity; `tianhengViolationId/v1` is no longer emitted.

### Removed
- **Breaking:** `FindingKey`, presentation-derived `ViolationId` construction, numeric baseline
  generations, legacy text matching, and automatic baseline upgrade behavior.

### Migration
- Preserve desired `owner` / `tracker` annotations externally, move or delete the old baseline,
  run `tianheng check --write-baseline <file>`, then restore annotations onto the newly observed
  facts. Unsupported existing files are never overwritten. There is no automatic adapter.
- Architecture tests should call an existing standalone `check*` function or
  `check_constitution`, then assert on `Violation::target()`, `Violation::rule_key()`, and
  `Violation::fact()`. The identity migration adds no plugin protocol; the separately specified
  `tianheng::testing::GovernanceTest` harness is available for repository architecture tests.

### Compatibility evidence
- Pacta `d3e24df`'s unpublished `pacta-governance` consumer compiled against this checkout's local
  `tianheng` and `guibiao` crates (`cargo check -p pacta-governance`) from a temporary copy; no
  Pacta source migration was required. This is recorded historical external evidence, not a
  sibling-repository dependency of Tianheng's required CI. Ongoing local reaction is provided by
  the external-view `tianheng` and `guibiao` `adopter_surface` tests; those fixtures protect the
  corresponding public call shapes without claiming to re-verify that external commit.

- Refined core project documentation density (`PROJECT.md`, `BACKLOG.md`) to archive verbose
  historical post-mortems and prune redundant release ledgers, reducing context token overhead.

## [0.2.3] - 2026-07-22

### Fixed
- 渾儀's forbidden-marker self-type resolver (`resolve_self_type`) now routes through the crate's
  own hop-capped alias/re-export fixpoint instead of a second, hand-rolled loop guarded only by an
  exact-repeat check — closing a real unbounded-loop gap (a divergent, non-cycling alias rewrite
  chain the exact-repeat guard alone cannot catch) and, as a side effect, an alias-resolution false
  negative (a member reached through an aliased *prefix*, not just an exact alias key, now lands).
- 圭表 now reacts (a constitution error) when a plain `mod x;` resolves to BOTH `x.rs` and
  `x/mod.rs` at once — a genuine `rustc` compile error (E0761) it previously accepted silently as
  two separate sources, dual-governing one module path. Matches 漏刻's own probe scanner, which
  already reacted on this exact shape.
- 渾儀's single-module-anchored resolver (`descend`) now tolerates a `#[cfg]`-gated `mod x;` with
  no backing file, matching its own crate-wide walker's (`resolve_child_modules`) existing policy —
  the two previously disagreed, so a boundary anchored directly at a `#[cfg]`-gated module hard-
  failed even when a mutually-exclusive per-platform sibling (e.g. an inline arm) legitimately
  resolved it.
- 漏刻's CI probe-coverage scanner now canonicalizes its module-cycle dedup guard (via a new,
  additive `xingbiao` dependency gated behind the non-default `audit` feature — never reaches the
  production hot path), matching 圭表/渾儀's own guards. Previously deduped on the literal path
  only, so a symlinked directory or circular `#[path]` chain reached via two distinct literal paths
  to the same real file could make the scan misbehave instead of terminating cleanly.
- 漏刻's CI probe-coverage scanner no longer tolerates a missing conventional module file merely
  because the item carries ANY `#[cfg]`/`#[cfg_attr]` attribute. Verified against a real `rustc`
  build: unlike a bare `#[cfg(pred)]` (which genuinely removes the item when `pred` is false),
  `#[cfg_attr(pred, …)]` never removes the item — only conditionally applies its wrapped
  attribute — so a `#[cfg_attr(unix, allow(dead_code))] mod x;` with no backing file is a real,
  unconditional compile error (E0583) that was previously silently skipped by the audit.
- 圭表 and 渾儀 now tolerate a missing unconditional `#[path]` target when the item also carries a
  co-occurring bare `#[cfg(pred)]` — a standard per-platform shim (`#[cfg(windows)] #[path =
  "windows_impl.rs"] mod imp;`) that previously hard-failed on any platform whose target file
  wasn't committed, even though rustc itself strips the whole item, `#[path]` included, before
  ever resolving it when `pred` is false (verified against a real build).
- 圭表 now reacts (a constitution error), rather than silently dropping the module from
  `reachable`, when a plain `mod x;` with no backing file carries no `#[cfg]` at all — closing a
  longstanding cross-dimension coverage gap (渾儀 already hard-erred on the identical shape). A
  `#[cfg]`-gated missing file is still tolerated, matching 渾儀. A boundary anchored directly at a
  module whose sole declaration was `#[cfg]`-tolerated away now reacts as an unknown module
  (never a vacuous clean pass), matching 渾儀's own resolver's identical precedent — unless an
  inline sibling arm of the same name exists, in which case the self-describing inline-target
  error still applies (never misreported as a generic "unknown module, check the path" error).
- 圭表's and 漏刻's independent `#[path]`-string decoders now handle backslash-newline line
  continuation (`"a\` + newline + `b"` decoding to `"ab"`), matching `syn` (used by 渾儀) and real
  `rustc` behavior. Previously 圭表 silently dropped such a remapped module from `reachable` with
  no error, and 漏刻 fell back to (or hard-errored on) the conventional location instead of
  following the real target.

### Changed
- Internal refactor: modularized crate internals across `xuanji`, `xingbiao`, `guibiao`, `hunyi`, `louke`, and the `tianheng` runner's projection layer (deduplicated JSON/text boundary-projection rendering) — no public API, JSON wire format, or self-governance boundary changed.

## [0.2.2] - 2026-07-22

### Fixed
- 圭表 module reachability now walks into an inline `mod parent { … }` body to find its own
  file-backed declarations, so a child reached only through an inline parent (`mod parent { mod
  child; }`, compiling `parent/child.rs`) is observed and its imports are checked.
- 圭表 now follows an unconditional, direct `#[path = "…"]` module declaration to its real target
  (matching 渾儀 and 漏刻), so a relocated module's imports are observed by all three observation
  dimensions. A `cfg_attr`-wrapped `#[path]` remains excluded (cfg-conditional, never followed
  cfg-blind).
- Every declared source for a module name is now observed, cfg-blind: an inline module body's own
  nested declarations, a plain conventional file, and an unconditional `#[path]` remap of the same
  name under mutually-exclusive `#[cfg]` arms (the standard per-platform shim) are all governed,
  regardless of attribute order or which source is scanned first. A plain (`#[path]`-free) `mod
  child;` declared inside a file reached through an unconditional `#[path]` remap is now governed
  under its logical path.
- A `#[path]` inside one mutually-exclusive `#[cfg]` arm's target — or inside a plain child of that
  arm — that legitimately references a sibling arm's own target (the two are never simultaneously
  open in any real build) is no longer misreported as a module cycle. Plain-child resolution now
  tracks each source's own directory context (where a `#[path]` written in it resolves, and
  separately, where its own plain/inline children live) instead of resolving through a shared
  structural index.
- A plain child reached only through a **symlinked directory** component, and an inline module
  preceded by an unconditional `#[path]` header (which relocates the base its own file-form
  children resolve from), are both now followed and governed correctly.
- 渾儀's single-module resolver (backing signature-coupling, visibility, dyn/impl-trait, and
  async-exposure anchors) now unions every mutually-exclusive `#[cfg]` variant of a module — inline
  and file-form alike — instead of stopping at the first match, and resolves a segment nested
  beneath a split point, or a `#[path]`-loaded module's own conventional children, from that
  variant's own directory rather than a name-derived or shared one. Two `#[cfg]` arms plainly
  declaring the identical name (resolving to one real file) are deduped by canonical path so they
  never inflate one violation into two.
- A `use`-map, and the child-module/re-export/rename tables it depends on, are now computed **per
  branch** of a `#[cfg]`-split module rather than once over the flattened cross-branch union —
  closing false negatives where one branch's own `use` alias or genuine re-export was silently
  shadowed or overwritten by an unrelated, mutually-exclusive sibling branch. Two purely-inline
  `#[cfg]` siblings sharing one enclosing file are split into their own branches for this purpose,
  not just file-form ones.
- A finding's reported `file` is now attributed **at collection time**, carried from the exact
  `#[cfg]` branch that produced it, rather than re-resolved afterward from a module-path string —
  so a violation written in a non-first branch is reported at its own file, never an innocent
  sibling's.
- The subtree walker backing `.including_submodules()` now descends every surviving `#[cfg]` branch
  independently, each from its own resolved `#[path]` base, instead of collapsing several branches
  to one shared directory pair for further descent.
- A self type that resolves to the enclosing `impl`'s own declared generic type parameter —
  written as a bare identifier, a projection (`T::Assoc`), or a qualified path (`<T>::Item`) — is
  no longer resolved through a same-named `use` alias, in both the forbidden-marker acquisition
  gate and the trait-impl-locality owner label. This closes a false-positive marker finding and a
  dedup-collapse false negative where two distinct `MisplacedImpl` violations were silently
  reported as one.
- `async_exposure`'s subtree scan now assigns a continuously-incrementing ordinal across the whole
  walk, never reset per module — closing a dedup-collapse false negative where two
  mutually-exclusive `#[cfg]` branches of one async fn, each carrying an unrenderable const-generic
  self type, collided on the same fallback identity and were reported as a single finding.
- 漏刻's probe-coverage scanner now locates a `mod` declaration's own attribute preamble with a
  forward, literal- and attribute-group-aware scan, replacing a backward raw-byte scan that could
  desync on a bare `;`/`{`/`}` inside an earlier attribute's string value or a brace-delimited
  attribute argument — closing false hard-fails and wrong-file substitutions on valid, compiling
  code.
- 圭表's crate-boundary rules (`forbid_dependency_on`, `restrict_dependencies_to`,
  `restrict_workspace_dependencies_to`, `restrict_dependency_sources_to`, and the
  feature-granularity rules) no longer observe a crate's own self-referential dependency on
  itself — a real, Cargo-legal pattern (e.g. a `[dev-dependencies]` path dependency on `.`, used
  for doctest/dogfooding) that names no other crate at all, so it can never be the cross-crate
  concern any of these rules exist to govern. The exclusion lives in the shared dependency
  observation itself, so every crate rule is covered at once.

## [0.2.1] - 2026-07-21

### Changed
- Published finding schemas and their dimension-local canonicalizers are now exhaustively pinned as
  compatibility reactions. Human finding wording remains presentation and is deliberately not
  snapshot-frozen.
- The baseline guide now documents the existing `--write-baseline` operation as the bounded,
  explicit V1-to-V2 upgrade path, including metadata carry-forward and stale-entry removal.
- 圭表 `must_not_import` now documents a stated partial-coverage bound: a `use`-glob of an
  *ancestor* of the forbidden module (`use crate::*;` while forbidding `crate::secret`) is observed
  at the glob's base, not as the forbidden descendant edge, so it does not react — forbid or confine
  the parent. The narrow `use crate::secret;` / `use crate::secret::*;` forms are caught as before.

### Fixed
- 渾儀 unsafe-confinement now qualifies a **trait-impl** `unsafe fn` by `<trait for self>`
  (`unsafe fn <A for Foo>::m`), not its self type alone: on one self type, an inherent `unsafe fn m`,
  `impl A for Foo { unsafe fn m }`, and `impl B for Foo { unsafe fn m }` are three distinct sites and
  now stay three findings. Previously all collapsed to `unsafe fn Foo::m`, so a baseline of one
  silently accepted a later-added trait-impl `unsafe fn` on a safe trait — a false negative, the
  trait-impl case 0.2.0's notes already claimed owner-qualified. *Baseline note:* this changes the
  `finding_key` of a trait-impl `unsafe fn`, so a 0.2.0 baseline entry for one resurfaces on upgrade
  and must be re-accepted (`--write-baseline`); unsafe-confinement is one release old, so the
  affected surface is minimal.
- Baseline `owner` / `tracker` metadata now rejects non-string JSON values instead of silently
  erasing malformed governance data; the CLI gate fails as a constitution error and explicit
  rewrite retains its warning-before-recovery behavior.
- Runtime probe coverage now starts from every exact Cargo library and binary target root and walks
  only module-reachable source, so an orphan `.rs` file can no longer satisfy a seam it never
  enforces. Direct callers that pass a directory retain the legacy recursive corpus.
- 渾儀 and 漏刻 now **follow** an unconditional `#[path = "…"] mod x;` to its author-chosen file,
  closing a coverage false negative: a relocated module's `unsafe` sites, trait impls, and
  `assert_boundary!` probes were previously dropped, so a disallowed impl or an undeclared-seam probe
  in a relocated module passed unobserved (semantic single-module boundaries on such a module errored
  loudly rather than governing it). The target is resolved with rustc fidelity — relative to the
  containing file's own directory, accumulating each enclosing inline-`mod` name as a directory
  component (so `mod inline { #[path="p.rs"] mod inner; }` reads `inline/p.rs`), with the path
  literal's escapes decoded as rustc and syn do; the two independent dimensions resolve the same
  file, and two declarations sharing one target (or a conventional `mod` plus a `#[path]` alias to
  it) are governed under each path rather than misread as a module cycle. A `#[path]`-loaded file is
  mod-rs-like, so its own children resolve from its directory. A `cfg_attr`-wrapped `#[path]` stays a
  stated bound — not followed cfg-blind, since it could observe a file rustc does not compile in this
  configuration — and an absent unconditional target is a fail-loud constitution error. Both
  dimensions detect the attribute structurally, so an incidental `path` substring in a comment or a
  `#[cfg(feature = "fastpath")]` gate is never mistaken for a relocation. As with any false-negative
  closure, a downstream carrying a real violation inside a relocated module may see green CI turn red
  on upgrade — adopt via `warn` / `Baseline` (the same patch-level precedent as the v0.1.3 re-export
  closure).
- The probe-coverage walker now tolerates a `#[cfg(...)]`/`#[cfg_attr(...)]`-gated module whose file
  is absent in the current configuration (an off feature or another platform), skipping it instead
  of failing the audit — matching the semantic dimension, so a cross-platform workspace no longer
  hard-errors on a platform-specific module. A non-cfg missing module and a resolution ambiguity
  remain fail-loud.

## [0.2.0] - 2026-07-20

The first **breaking** window since `0.1.0` — a deliberate `0.2.0` minor (the `0.1.x` hold ended
when real adopters arrived). The break is quarantined to internal identity/model surfaces; the
adopter-written builder is a drop-in swap (see **Compatibility**).

### Added
- **`tianheng::check_constitution`** — one inspectable composed reaction over the static (圭表),
  semantic (渾儀), and runtime (漏刻) dimensions in a single call, sharing the runner's evaluator
  (static-first error precedence, runtime orphan-probe auditing) without going through the CLI.
- **Adopter surface contract.** The composed wildcard `prelude` is now an explicit,
  compile-checked external compatibility promise, with a symmetric `ModuleRule` inspection path;
  hidden granular checks stay outside the promise.

### Changed
- **BREAKING — structured violation identity.** Violation matching moved from rendered finding
  *text* to dimension-owned **structured keys**: `Violation::new` now takes a typed `ViolationId`,
  and newly-written baselines use version-2 `finding_key`s (fact-specific named fields) instead of a
  rendered descriptor. 渾儀's semantic findings derive both their diagnostic text and their key from
  one typed fact model. Reports stay byte-identical.
- **BREAKING — 圭表 rule model surface narrowed.** `Rule` / `ModuleRule` are now
  builder-constructed only — downstream can no longer construct or exhaustively destructure their
  data-carrying variants (open-ended *inspection* stays available through the boundary accessors);
  `InlineExternalStrict` is folded into `Inline`. Reaction, projection, polarity, and violation
  identity are unchanged.

### Fixed
- 渾儀 unsafe-confinement: `unsafe fn` findings are now **owner-qualified** (`unsafe fn {owner}::{m}`)
  for inherent, trait-declaration, and trait-impl methods, so two same-named `unsafe fn`s on
  different owners in one out-of-subtree module no longer collapse to one finding — closing a
  baseline-masking false negative (the `unsafe fn` sibling of 0.1.8's `unsafe impl` closure).
- 圭表 inline-symbol-path confinement (`must_not_call_inline`): a `use`-group member whose name
  merely *starts with* the substring `self` (e.g. `use chrono::{self_utc as clk}`) is now resolved
  rather than dropped, so a confined inline call through such an alias reacts — closing a false
  negative.
- 渾儀 single-module resolution: a module split across `#[cfg(…)] mod x { … }` **inline variants** now
  has every variant governed (matching the crate-wide scan's observe-all), so a forbidden exposure
  in a non-source-first variant reacts — closing a `mod`-resolution false negative.

### Compatibility
- The **adopter-written builder** (`Constitution`, `CrateBoundary`, `ModuleBoundary`, the boundary
  DSL, `run`, `prelude`) is a **drop-in swap** — the break is quarantined to the internal
  `Violation` / `ViolationId` / baseline wire and 圭表's rule-model surface.
- **Baseline migration.** Version-1 baselines are still read (exact-text match), so existing
  baselines keep grandfathering; a baseline rewritten under this release upgrades to the version-2
  structured form.

## [0.1.10] - 2026-07-15

### Added
- 圭表 **feature-granularity crate-dependency boundary** — `CrateBoundary::crate_(…)`'s
  `restrict_features_of(C, […])` / `forbid_features_of(C, […])` / `forbid_feature(C, f)` govern
  which features a crate *declares* on a dependency `C`: its explicit `features` list plus the
  `default` pseudo-feature (so `forbid_feature(C, "default")` ≡ requiring `default-features =
  false`), matched by package name and unioned across the target's dependency edges. It observes
  the **declared** request only — never expanding `C`'s own `[features]` graph and never reading
  `cargo metadata`'s resolved `resolve.nodes[].features` — so it is stable under Cargo feature
  unification and builds under the existing `--no-deps` metadata read with no new dependency.
  Findings are `C/feature` (kind-qualified when the dependency kind is not `Normal`), injective
  across the two polarities; severity, baseline, dependency-kind selection, and the text/JSON
  projection reuse the existing crate-rule machinery. Transitive/unification-enabled features are
  an explicit non-goal (declared-not-resolved, at the altitude of the existing dependency rules).
  Additive and non-breaking; existing constitutions and baselines are unaffected. See
  `COOKBOOK.md`.

### Changed
- Contributor-facing docs only: `AGENTS.md` makes the project's practised conventions explicit
  (document authority, OpenSpec lifecycle, adversarial review, single-source Definition of Done,
  branch prefixes, subject-only release commits); `BACKLOG.md` records the `0.1.x → 0.2.0` trigger
  and the install-vs-constitution decision; the `README.md` license section links to its files.

## [0.1.9] - 2026-07-11

### Added
- 圭表 `must_not_call_inline(…).strict_external()` — **opt-in**: also catch a *fully-qualified
  external-crate* call (e.g. a bare `chrono::Utc::now()` with no `use chrono`), closing the
  asymmetry where a sysroot head (`std::time::…`) was caught but a fully-qualified external head was
  silently resolved as local. A bare head matching a declared dependency is resolved as that crate,
  after a local-precedence ladder so a genuinely-local item of the same name stays local at any
  nesting depth. Composes with `.ending_with` / `.strict_prefix_only`; with the flag off the default
  is **byte-identical**, so existing constitutions and baselines are unaffected. Carried as a new
  `#[non_exhaustive]` rule variant (patch-safe; identity-parity, no baseline churn), and 圭表 grows
  its own rename-aware dependency-name reader — no dependency on 渾儀 (三儀 ⊥ 三儀), still `syn`-free.
  Stated bounds (an `extern crate … as` rename; and, under a single-segment prefix, a local binding
  or a definition site that reads as a call) are declared, never a silent pass.
- Adopter cookbook recipes (`COOKBOOK.md`): test that a boundary reacts, gate workspace coverage in
  CI, why exposure rules are deny-shaped (not a "may only expose" allowlist), and the
  `strict_external` recipe. `README.md` gains a "what the instruments do **not** see" note, so a
  reader does not over-infer a dimension's reach (渾儀 reads a signature's types/traits, never a
  call site).

### Changed
- Internal refinement, behavior-preserving and no public-API change: 渾儀's whole-crate-scan
  capabilities share one violation-emission helper; the text projection shares a module-block
  helper; idiom/consistency cleanups; and `xingbiao` now carries `#![deny(missing_docs)]` like its
  five sibling crates.

## [0.1.8] - 2026-07-11

### Added
- 圭表 inline-symbol-path confinement — forbid a crate from *calling* a fully-qualified path inline
  (e.g. `std::time::SystemTime::now()`), resolving `use` renames / aliases / re-exports and the
  glob-danger shapes. The syn-free static complement to observing a `use`-import.
- 渾儀 `UnsafeBoundary` — declare that a crate's `unsafe` (blocks, `unsafe fn`/`impl`/`trait`,
  `unsafe extern`) may appear **only under** a declared subtree
  (`UnsafeBoundary::in_crate("app").only_under(["crate::ffi"])`): the auditability boundary of a
  layered crate, the confinement complement of `#![forbid(unsafe_code)]`.
- 渾儀 visibility ceiling — `max_visibility(Crate | Super | Module)`, generalizing the binary
  `must_not_declare_pub` into a rank ceiling (an item declared above the ceiling reacts; the prior
  rule is now the `max_visibility(Crate)` sugar, byte-stable in findings).
- 渾儀 async-exposure opt-in **subtree** scope — `.including_submodules()` descends the anchored
  module's whole subtree, so a "this seam is synchronous" boundary governs a pure kernel throughout,
  not only at its own seam.
- Every crate declares `#![forbid(unsafe_code)]` — the family is `unsafe`-free and says so at
  compile time.
- `examples/` gained `unsafe-confinement` and `sans-io-pure`, plus a `max_visibility` demo in
  `hunyi-standalone`.

### Fixed
- 渾儀 unsafe-confinement: the finding is owner-qualified (`unsafe impl {trait} for {self type}`), so
  two `unsafe impl`s of one trait for different self types in a module no longer collapse to one
  finding — closing a baseline-masking false negative.
- 渾儀 / 圭表: a nested `#[cfg_attr(pred, path = "…")]` module remap is recognized in both dimensions,
  closing a silent false negative in the static scanner and the semantic subtree walk.
- 圭表 type-alias resolution skips a defaulted generic parameter's `=`
  (`type Clock<Tz = LocalTz> = std::time::SystemTime;` now resolves to its real target), closing a
  false negative where a confined type reached through the alias passed unobserved.

### Changed
- modou is no longer framed as superseded. It is a living, independently-developed sibling project;
  Tianheng's static core (圭表) is *derived from* it, and Tianheng keeps all three dimensions
  (README / PROJECT).
- README gained a Phase-0 one-line on-ramp (lock one seam, enforce, pipe SARIF into CI) above the
  full multi-dimension example.

## [0.1.7] - 2026-07-08

### Added
- 圭表 `confine_external_crate` — confine an **external** crate's `use` imports to one declared
  module subtree (FFI / platform-vocabulary confinement): `ModuleBoundary::in_crate("app")
  .module("crate::ffi").confine_external_crate("libc")` reacts when any module outside
  `crate::ffi`'s subtree imports `libc`. The first static rule to *observe* external-crate imports
  (every other rule ignores them), source-observed — not a `cargo metadata` dependency-table rule.
  The confined crate is the violation target, so confinements of different crates on one module stay
  distinct in the baseline. A package name written with a `-` (e.g. `windows-sys`) matches its
  underscore import identifier (`windows_sys`).
- `COOKBOOK.md` — a cookbook of common governance intents expressed as declared boundaries (圭表 /
  渾儀 / 漏刻 recipes), the imitable surface an adopter or agent copies rather than translating a
  foreign policy format.
- Coloured, reason-first terminal output for the human `check` report — a severity-coloured header
  (red for an enforced violation, yellow for an advisory) over the emphasised reason. Presentation
  only: gated to an interactive terminal (honours `NO_COLOR`), so a pipe, a redirect, or a CI log
  stays byte-identical, and `--format json` / `sarif` are never coloured.
- `examples/` — three runnable, self-checking examples: `guibiao-standalone` (the syn-free static
  import linter), `hunyi-standalone` (the semantic public-API exposure linter), and `composed`
  (the `tianheng` shell governing one app with all three instruments, in a CI-time `check` mode and
  a runtime `run` mode).
- Per-instrument GitHub issue templates (圭表 / 渾儀 / 漏刻).

## [0.1.6] - 2026-07-07

### Changed
- Extracted the `cargo metadata` substrate into a new `xingbiao` crate — a `serde_json`-only base
  beneath the dimensions — so the static and semantic dimensions read the workspace through one
  source of truth instead of two hand-copied twins.

### Fixed
- 渾儀 forbidden-marker: closed two false negatives — a hand `impl` whose self-type is spelled
  through a `pub use` re-export, and a locally-renamed (`use … as`) trait/derive leaf.

## [0.1.5] - 2026-07-07

### Added
- 圭表 `must_only_be_imported_by` — the closed inbound dual of `must_not_be_imported_by`
  ("only `crate::facade` may import `crate::internal`").

### Fixed
- 漏刻 probe-coverage audit: a probe inside a `macro_rules!` body no longer counts as coverage.
- Recorded a documented robustness bound in the `use`/`mod` lexer around multibyte char literals
  (no confirmed false negative).

## [0.1.4] - 2026-07-05

### Fixed
- 圭表 module-source hardening: module boundaries use Cargo's observed `src_path`, and
  `#[path]`-remapped and inline-only orphan modules are excluded rather than governed through a
  same-named conventional file.
- Packaging: every publishable crate now physically bundles its `LICENSE-MIT` / `LICENSE-APACHE`
  texts (`cargo publish` ships only crate-local files; 0.1.0–0.1.1 shipped without them). Guarded
  by a CI reaction.

## [0.1.3] - 2026-07-02

### Added
- 渾儀 semantic depth: public re-export exposure and trait-impl exposure.

## [0.1.2] - 2026-07-02

### Added
- 圭表 `restrict_dependency_sources_to` — govern the declared dependency source kind
  (git / registry / path).
- 渾儀 `dyn`-trait and `impl Trait` boundaries, and async-exposure.

## [0.1.1] - 2026-06-30

### Fixed
- Early packaging and metadata hygiene.

## [0.1.0] - 2026-06-29

### Added
- Initial release of the crate family: the `xuanji` reaction model, the three observation
  instruments — 圭表 (`guibiao`, static), 渾儀 (`hunyi`, semantic), 漏刻 (`louke`, runtime) — and
  the 天衡 (`tianheng`) shell that composes them into one `check` with a `0` / `1` / `2` exit
  contract and `--format json` / `sarif` projections.

[Unreleased]: https://github.com/tacticaldoll/tianheng/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/tacticaldoll/tianheng/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/tacticaldoll/tianheng/compare/v0.2.2...v0.2.3
[0.2.2]: https://github.com/tacticaldoll/tianheng/compare/v0.2.1...v0.2.2
[0.2.1]: https://github.com/tacticaldoll/tianheng/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/tacticaldoll/tianheng/compare/v0.1.10...v0.2.0
[0.1.10]: https://github.com/tacticaldoll/tianheng/compare/v0.1.9...v0.1.10
[0.1.9]: https://github.com/tacticaldoll/tianheng/compare/v0.1.8...v0.1.9
[0.1.8]: https://github.com/tacticaldoll/tianheng/compare/v0.1.7...v0.1.8
[0.1.7]: https://github.com/tacticaldoll/tianheng/compare/v0.1.6...v0.1.7
[0.1.6]: https://github.com/tacticaldoll/tianheng/compare/v0.1.5...v0.1.6
[0.1.5]: https://github.com/tacticaldoll/tianheng/compare/v0.1.4...v0.1.5
[0.1.4]: https://github.com/tacticaldoll/tianheng/compare/v0.1.3...v0.1.4
[0.1.3]: https://github.com/tacticaldoll/tianheng/compare/v0.1.2...v0.1.3
[0.1.2]: https://github.com/tacticaldoll/tianheng/compare/v0.1.1...v0.1.2
[0.1.1]: https://github.com/tacticaldoll/tianheng/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/tacticaldoll/tianheng/releases/tag/v0.1.0
