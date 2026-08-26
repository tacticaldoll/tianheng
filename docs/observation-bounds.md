# Observation bounds

Every **observation bound** this family declares: a claim that a reaction deliberately stops at a
named shape, so that shape is governed policy rather than a defect.

**25 of 96 declared bounds have no pinning test.** That figure is the register's
audit backlog and leads the document because a number in a footnote is not read. Each such bound names
the tracker that owns closing it.

Generated from `openspec/specs/*/spec.md` by `crates/kanhe/tests/bound_register.rs`. **Do not edit by hand** —
regenerate with `BLESS=1 TIANHENG_WORKSPACE_TESTS=1 cargo test -p kanhe --test bound_register`. A stale projection fails that gate.

**What this document does not claim.** It lists the bounds the specs *state in a recognizable form*: a
scenario whose heading marks it a bound. The undeclared-prose direction that keeps this list honest has known
residuals and a deliberate exemption, each stated in this document rather than left in the check's
comments, because a residual a reader cannot see is one the register is lying about:

1. **Unrecognized wording.** A bound worded outside the scanned form — "out-of-scope", "does not claim
   to observe", "a stated, inherited bound" — is invisible to the scan.
2. **The scan is line-oriented.** A statement whose bound names continue onto the next line is examined
   only on the line carrying the trigger words.
3. **A reference clears more than it names.** `(bound: …)` clears the prose it sits with regardless of
   how many bounds that prose states, or whether the bound it names is one of them. This is how a
   retired `#[path]` bound survived two sweeps inside a sentence listing four inherited bounds behind
   one reference to a fifth. The discipline is one reference per stated bound, and it is the author's:
   closing it would mean reading which bounds a sentence lists, which no repository check can do. Scanning
   paragraphs instead of lines was measured against that defect and would not have caught it, because
   the paragraph carries the same clearing reference.

The **exemption**: prose under a requirement whose heading names bounds is not reported, because several
such requirements state their bounds as a numbered list, and requiring each item to become its own
scenario would restructure them and read worse. Its price
is charged — such a requirement must declare at least one bound scenario — but the other items of its
list are unregistered, which is why this list is a floor rather than a proof of completeness.

The second floor is the same shape. A bound declared twice is caught only when both declarations cite
the **same pinning test**, which is a fact rather than a heuristic; two declarations of one behaviour
citing two different tests are invisible. Telling those apart from two genuine bounds over sibling
shapes is a semantic judgment — two operand dimensions here declare identically-worded bounds over
`dyn` and `impl Trait`, each defended by its own test, and each must declare its own — so nothing
observes it and no bound of the register capability claims it.

A third floor was stated here for one change and is **retired**: a `pinned by` line could be satisfied
by a definition that never ran — commented out, inside a string, removed by a `cfg`, or trapped in an
uninvoked macro — because the scan read only the form of a line. Test-ness is now decided by the test
harness enumeration, which registers none of those. The weakness survives only in the source-text
fallback used where no manifest exists, which the register spec describes.


## crate-dependency-boundary

### `crate-dependency-boundary/an-optional-dependency-edge-is-observed-as-a-declared-one-a-stated-bound`

> that edge is governed exactly as an unconditional one, a stated bound: the rules read the **declared** dependency set, cargo reports an optional edge in that set like any other, and no rule can express *depends on this only when that feature is on*. A boundary therefore cannot say that an edge is confined to a non-default feature, and a reason that claims it would be asserting structure no rule observes. Where that distinction matters, the reaction is a build of the feature-off configuration — outside this capability — and the split belongs in the manifest's own `[features]` prose

- **pinned by**: `an_optional_dependency_edge_is_observed_as_a_declared_one`

## crate-source-boundary

### `crate-source-boundary/a-git-plus-version-dependency-is-flagged-though-it-would-publish-a-stated-bound`

> the system classifies it as `Git` (its declared source is `git+…`) and emits a violation — even though such a dependency would `cargo publish` successfully — because the rule governs the declared source kind, a stated conservative bound, not publish-eligibility

- **pinned by**: `source_rule_flags_every_git_source_outside_a_registry_or_path_allowlist`

## external-crate-confinement

### `external-crate-confinement/cfg-gated-code-is-observed-as-written-a-stated-bound`

> the system observes it as written rather than evaluating the predicate, so the reaction is cfg-blind — inherited from the module scanner and stated here, never a silent claim about which branch is live

- **pinned by**: `confine_external_crate_is_cfg_blind_to_unenabled_cfg_arms`

### `external-crate-confinement/a-confined-crate-use-inside-a-string-or-macro-body-is-not-observed-a-stated-bound`

> the system reports no violation, because comments, string literals, and macro bodies are stripped before scanning, matching the scanner's stated bounds

- **pinned by**: `confine_ignores_a_use_inside_a_string_literal`

### `external-crate-confinement/an-extern-crate-declaration-is-not-observed-a-stated-bound`

> the system reports no violation, because the rule is use-only and observes `use` imports rather than `extern crate` declarations

- **pinned by**: `confine_ignores_an_extern_crate_declaration`

## inline-symbol-path-confinement

### `inline-symbol-path-confinement/a-future-read-verb-outside-the-declared-set-is-a-documented-bound`

> the system does NOT react (a false negative the adopter owns by narrowing), rather than the engine silently guessing which verbs are reads

- **pinned by**: `inline_a_verb_outside_the_declared_set_is_a_bound`

### `inline-symbol-path-confinement/a-receiver-method-read-is-a-documented-bound`

> the system does not claim to observe it (no type inference on the receiver) — a stated bound, not a silent assertion of cleanliness

- **pinned by**: `inline_receiver_method_read_is_a_bound`

### `inline-symbol-path-confinement/a-path-taken-as-a-value-is-a-documented-bound-under-the-default`

> the system does not react (value-position mention is a stated bound under the default; `.strict_prefix_only()` catches it) — declared, not silent

- **pinned by**: `inline_value_capture_is_a_bound_under_the_default`

### `inline-symbol-path-confinement/an-external-crate-re-export-is-a-documented-bound`

> the system does not claim to observe it (foreign AST is not scanned) — a stated bound

- **pinned by**: `inline_foreign_reexport_of_the_confined_path_is_a_bound`

### `inline-symbol-path-confinement/an-extern-crate-rename-is-a-stated-bound-under-strict-external`

> the system does not claim to observe the call through the `chr` alias head (the use-map reads `use` only; the `extern crate … as` rename is a stated bound even under strict-external), never a silent assertion of cleanliness

- **pinned by**: `inline_strict_external_extern_crate_rename_is_a_stated_bound`

### `inline-symbol-path-confinement/the-fully-qualified-external-call-is-a-stated-bound-under-the-default`

> the system does NOT react (the fully-qualified un-`use`d external call is a stated non-observation under the default; behavior is unchanged from before this capability)

- **pinned by**: `inline_strict_external_absent_fully_qualified_call_is_a_bound`

## observation-bound-model

### `observation-bound-model/whether-a-declaration-s-stated-cause-is-the-real-cause-is-not-observed-a-stated-bound`

> the model does not claim to observe it, a stated bound: the extent is typed and checkable, the rationale is prose, and requiring the prose to match would trade a fact for a heuristic

- **pinned by**: `a_rationale_that_contradicts_its_extent_is_a_stated_bound`

### `observation-bound-model/an-answer-that-depends-on-the-corpus-entry-point-has-no-extent-of-its-own-a-stated-bound`

> it is declared as under-reacting with the entry point as the inherited owner rather than carrying an extent of its own, a stated bound: one live instance does not earn a value in a set every other member has several of, and the direction that matters (a seam reported covered when it is not) is recorded either way

- **pinned by**: `an_entry_dependent_bound_is_declared_as_under_reacting`

### `observation-bound-model/a-bound-both-out-of-reach-and-granularity-limited-cannot-be-expressed-a-stated-bound`

> the model cannot express it, a stated bound: no declared bound exhibits the pair, and offering granularity on every extent would invite a combination nothing shows while weakening the nesting that makes the contradiction above impossible

- **pinned by**: `granularity_is_carried_only_by_the_as_intended_extent`

## observation-bound-register

### `observation-bound-register/which-member-holds-a-check-is-a-judgement-a-stated-bound`

> nothing observes whether it landed in the right one. The split is by what a check judges — the law and the delivered product on one side, this repository's record on the other — and two mechanical rules were each measured unreliable: a text scan reads a comment naming `AGENTS.md` as governance while a check scanning every tracked file names nothing, and the workspace marker means both "this needs the repository as its subject" and "this needs a fixture". Position is the declaration; the join below catches a **capability** named wrongly, never a member chosen wrongly

- **unpinned**, tracked by: `BACKLOG.md` — *which governance member a check belongs to is unobserved*

### `observation-bound-register/whether-a-citation-demonstrates-the-direction-its-bound-declares-a-stated-bound`

> nothing reacts. `Extent::demonstrates()` names the direction a defence must show, and it reaches the projection's label and the contradiction classification beside it; no reader compares that prediction with what the cited test asserts. Reaching further means deciding what a test demonstrates from its source, which is a judgement over code of the same kind this repository has designed, measured and rejected over prose — and unlike a citation that never runs or never bites, there is no reaction here whose gap a fixture could exhibit. This is the sibling of *a rationale that contradicts its extent*, one step over: the prose beside an extent is already declared free to disagree with it, and so, until now silently, was the test beneath it

- **unpinned**, tracked by: `BACKLOG.md` — *a pin may defend a direction its bound does not declare*

### `observation-bound-register/what-code-executed-inside-the-checkout-does-outside-it-is-not-observed-a-stated-bound`

> nothing reacts. Running the cited test is the reaction's whole method, so code execution inside the checkout is granted unconditionally, and neither consequence is separable from it: the shared common directory is what makes a git-reading citation reachable at all, and re-checking a resolved path after the build would re-check the same window that defeated it. Hooks are the one case that IS closed, because those run without any citation asking for them

- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-cited-test-s-outcome-depends-on-its-run-count-is-not-observed-beyond-one-period-a-stated-bound`

> the citation is reported as exercised by a perturbation that did nothing. The reaction runs the test a fixed number of times, so any period matching that sequence escapes it, and the number is readable in the reaction's own source. Closing it needs each run to be unable to observe how many times the test has run — a separate checkout per run — whose cost grows with the coverage this capability exists to grow

- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-pin-gutted-but-not-committed-still-bites-is-not-observed-a-stated-bound`

> nothing reacts, because the checkout under test is HEAD's content. The sibling gates read the worktree for exactly this reason, and this reaction cannot: mutating the author's checkout is what a separate checkout exists to avoid, so the two properties are in tension and this one is given up deliberately

- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-record-perturbs-the-check-or-the-pin-s-own-assertions-is-not-observed-a-stated-bound`

> the pin fails and the citation is counted as exercised, because a killed pin does not say what killed it. Separating the two by refusing a record that edits its pin's file was measured against this tree's own first record, which legitimately edits the file its pin lives in, so the rule would refuse a conforming shape

- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

### `observation-bound-register/whether-a-citation-carrying-no-declared-mutation-is-defended-is-not-observed-a-stated-bound`

> the reaction does not decide whether that pin bites, and says how many such citations there are on every clean run rather than leaving the gap to be inferred

- **unpinned**, tracked by: `BACKLOG.md` — *most pinning citations have never been seen to fail*

## observer-protocol

### `observer-protocol/whether-the-shell-makes-an-independent-semantic-decision-is-not-observed-a-stated-bound`

> nothing reacts — a stated bound, and a declared false negative this repository owns. A text reader over the composition body was built, hardened across four review rounds and defeated at every level: name resolution, the parameter's binding site, the identity of the definition, the caller frame, and execution, which no reading of text reaches at all. Invoking the observer made the two paths' *equality* construction-held and left this untouched: a guard written above that call compiles, passes the whole suite, and passes every gate — measured on the tree that invokes the observer, not on the one that did not. The bound carries no pinning test because there is no reaction left to demonstrate a gap in; it is tracked instead

- **unpinned**, tracked by: `BACKLOG.md` — *the shell's semantic delegation, held by construction*

### `observer-protocol/a-whole-line-occurrence-that-is-not-the-definition-anchors-the-read-a-stated-bound`

> the reaction reads that copy's body and reports it as the method's. Both anchor conditions are satisfied — one occurrence, at a line start — and the reader knows nothing of comments or literals, so the class is "the unique whole-line occurrence is not the definition" rather than any one syntactic position. What passes is a **second, hand-maintained path that agrees today**: a *divergent* list does not, because `observation-bound-model` reads every dimension's declarations through `Observer::bounds` and holds them in a bijection with the specs, which fails on any difference of membership or content. Measured both ways. So the residual is narrower than a divergent list slipping through, and wider than a comment. **Not a defect unique to this reader.** `kanhe::region`'s own `Executed` abstraction declares the identical residue for the same reason — a block comment and a string literal both need nested-span lexing to tell from executed text, which this tree has defeated repeatedly and left declared rather than approximated. Closing either needs the same instrument; closing one without the other would leave the class recorded twice under two names for a reader to reconcile. This bound SHALL be **shown rather than described**: the reaction enumerates every shape it decides together with the decision, the reader is run against that table, and the rows where it reads a body that is not the method's are this bound. A sentence here that the table contradicts fails, which is what the three repair rounds preceding this scenario could not do

- **unpinned**, tracked by: `BACKLOG.md` — *the bounds-method reader anchors on a whole-line occurrence that is not the definition*

### `observer-protocol/a-trait-object-on-a-wrapped-signature-s-continuation-line-is-not-seen-a-stated-bound`

> the reaction does not see it, a stated bound: the recognizer is handed one line at a time, so the continuation is never a candidate it declined — it is text the observation never presents. Closing it needs 渾儀 watching this crate, which the same measurement above found unavailable. Multi-line public signatures exist here, so the shape is live even where no instance names a trait object

- **pinned by**: `a_trait_object_on_a_continuation_line_is_not_recognized`

### `observer-protocol/whether-an-observer-s-declared-bounds-are-complete-is-not-observed-a-stated-bound`

> the protocol does not claim to observe the omission, a stated bound: the trait compels a declaration, never a complete one, and no reaction can enumerate the limits of a reaction it did not write

- **pinned by**: `an_observer_may_under_declare_its_bounds`

### `observer-protocol/whether-an-observer-s-own-verdict-is-correct-is-not-observed-a-stated-bound`

> the fold merges it as given, a stated bound: it composes verdicts and does not adjudicate them, and a protocol that second-guessed each participant would need a second implementation of every dimension

- **pinned by**: `the_fold_does_not_adjudicate_a_participant_s_verdict`

### `observer-protocol/what-a-subject-does-not-establish-a-stated-bound`

> nothing reacts. The constructor is public because an implementor must be able to return the outcome, so the type converts an omission into a commission and stops there; distinguishing a reported subject from an observed one would need the engine to walk each dimension's corpus itself, which is the shared scanner 三儀 ⊥ 三儀 forbids. The engine owns this narrowing: it is a limit of what a protocol can ask of its participants, not a limit an adopter chose

- **pinned by**: `a_subject_is_declared_by_the_participant_and_not_verified`

## projection-register

### `projection-register/whether-a-stated-regeneration-command-regenerates-its-document-is-not-observed-a-stated-bound`

> the check does not observe it, a stated bound: verifying the claim means running the command, and both mechanisms refuse — a Rust `BLESS=1 cargo test` re-enters the harness already running, and the shell one **writes** the projection, which would make the check mutate the tree it judges

- **pinned by**: `a_regeneration_command_is_registered_and_never_run`

### `projection-register/a-document-generated-by-an-unrecognized-mechanism-is-not-observed-a-stated-bound`

> the check does not observe it, a stated bound: it is absent from both sides of the correspondence, which then holds over a surface missing a member. This is a false negative rather than a limit of the corpus — the third mechanism's source sits in the tree the check already reads — so it is owned by the engine

- **pinned by**: `a_third_generation_mechanism_is_not_recognized`

## publish-source-integrity

### `publish-source-integrity/the-tree-changing-after-the-gate-passed-is-not-observed-a-stated-bound`

> nothing reacts. The gate is one process and the act is another, and the wrapper holds no handle that ties them: `cargo publish` takes no argument naming the commit it must package, so there is no `--match-head-commit` to pin what was judged, which is what closes the equivalent window for the commit set on the merge path. What narrows it is `cargo publish`'s own refusal of a dirty worktree, which is a weaker property than the gate's — a tree amended and committed is clean again and packages a different commit under the same tag. The wrapper `cd`s once and `exec`s, so the window is two statements wide rather than a whole `cargo test`, and narrowing is all it can do

- **unpinned**, tracked by: `BACKLOG.md` — *the window the publish wrapper can only narrow*

### `publish-source-integrity/whether-the-tag-s-signer-is-authorized-is-not-observed-a-stated-bound`

> the gate accepts it, a stated bound: validity is verifiable without configuration and **attribution is not**, needing an allowed-signers file that exists on a maintainer's machine and not in CI. The ownership is inherited from the verification environment rather than held by this engine, because no change to this gate closes it — giving CI an allowed-signers file is what would

- **pinned by**: `a_valid_signature_from_an_unauthorized_key_is_accepted`

## reference-integrity

### `reference-integrity/a-relative-phrase-in-non-record-markdown-is-not-observed-a-stated-bound`

> nothing reacts. Extending the sweep to whole-document prose was measured against the tree it would judge: of the tracked non-record Markdown occurrences, one is `AGENTS.md`'s own row **declaring** the phrases, three are duration rather than pointer — *admitted it for a window*, which narrates how long something lasted — one of those three is a generated projection's copy of another, and two more are already anchored, one by a commit and one by naming the release. Two are genuine. A reader over text separates none of those groups: telling a phrase that points at a moving window from one that measures a span is a judgement about the sentence, which is the prose instrument `AGENTS.md` records as designed, measured three times and rejected

- **unpinned**, tracked by: `BACKLOG.md` — *a relative phrase in non-record Markdown*

### `reference-integrity/a-path-already-wrong-when-a-dated-record-was-written-is-not-observed-a-stated-bound`

> nothing reacts, and nothing ever will. The exemption is by section, not by whether the path was once right, and distinguishing the two needs the tree as it stood at that date — a per-section historical checkout this check does not make and whose cost is not proportionate to a mistyped path in a frozen record. The engine owns the narrowing: it is this check's rule that declines to look, not a limit an adopter chose

- **pinned by**: `a_dated_changelog_section_keeps_its_paths_and_an_undated_one_does_not`

### `reference-integrity/a-rust-identifier-named-in-prose-is-not-resolved-a-stated-bound`

> nothing reacts, and no reader of text can make it. Such tokens routinely match no declaration anywhere in the tree, and the most frequent of those are Rust keywords (`use`, `mod`, `dyn`, `fn`, `impl`), attribute names (`cfg_attr`) and std method names (`create_new`, `strip_prefix`, `remove_dir_all`). Telling a name that should resolve from one that should not needs type information about a receiver, which `inline-symbol-path-confinement` already declares unobserved.

- **unpinned**, tracked by: `BACKLOG.md` — *a Rust identifier named in prose is resolved by no reaction*

## release-coherence

### `release-coherence/a-dependency-declared-under-a-quoted-cfg-target-is-not-observed-a-stated-bound`

> nothing reacts. That heading's second key is a **quoted** cfg expression, and reading which configurations it selects is a grammar of its own rather than a context in front of a dependency table. The sibling form is read: a bare target triple is two bare TOML keys, and a bare key carries no dot, so the triple runs to the next dot with nothing guessed. This bound named both forms until that was measured — the reason it gives is about the quoted grammar, and it had been used to skip the whole target corpus

- **unpinned**, tracked by: `BACKLOG.md` — *a dependency declared under a quoted cfg target is not observed*

### `release-coherence/prose-about-the-marker-is-read-as-a-marker-a-stated-bound`

> the section is classified as breaking and required to carry a `### Migration` it does not owe. The reaction reads the marker's presence rather than its position, and that reach is kept deliberately: over-reaction is the safe direction, while a positional matcher would stop observing a real break whose marker sits anywhere but an entry's first token — buying a false negative in the floor to remove a refusal an author can argue with. The Core Contract forbids exactly one bug and it is the false negative

- **pinned by**: `prose_about_the_marker_is_read_as_a_marker_a_stated_bound`

### `release-coherence/a-dated-release-section-names-a-gate-a-stated-bound`

> nothing reacts, and the leak is real: an adopter reading `[0.4.0]` meets nine entries naming files they can never run. What is refused is the **repair**, not the diagnosis — rewriting a dated section to satisfy a rule written afterwards would falsify the record, the same reason `docs/history/` is left alone — so this is a declared false negative with an owner rather than a shape that is harmless. Closing it needs a repair that adds to the record instead of editing it. **The WHEN was wider than the reason, and the gap was live.** It read *a dated section*, so the section the current release is still being written into was exempt too — where no record exists to falsify, and where release-ready state requires `[Unreleased]` to be empty, leaving the check with no subject at all during preparation. Narrowed to what the reason actually covers

- **pinned by**: `a_dated_section_naming_a_gate_is_a_stated_bound`

### `release-coherence/machinery-the-judged-repository-tracks-by-nothing-a-stated-bound`

> nothing reacts. The enumeration is `git ls-files scripts/`, so an untracked `scripts/` reads as absent; closing this means judging worktree content, which this repository's gates are held not to do — the larger error, so the blindness is declared instead

- **pinned by**: `machinery_tracked_by_nothing_is_a_stated_bound`

### `release-coherence/an-entry-about-self-governance-that-names-no-machinery-a-stated-bound`

> nothing reacts. Reaching it needs a judgement over the entry's subject rather than over its references, and that instrument is the one this repository measured three times and rejected; widening the matcher toward it — heading keywords, phrase lists — would trade a declared, bounded blindness for an undeclared false-positive surface

- **unpinned**, tracked by: `BACKLOG.md` — *the self-governance residual is a judgement over an entry's subject*

### `release-coherence/a-basename-an-entry-writes-for-another-reason-a-stated-bound`

> the reaction **fails**, refusing an innocent entry. The direction is the safe one — an author meets a refusal to argue with — and narrowing it means deciding which of two files a bare name meant, a judgement about the sentence rather than about the reference

- **pinned by**: `a_colliding_basename_is_a_stated_bound`

### `release-coherence/a-name-reached-only-through-a-url-a-stated-bound`

> nothing reacts. A word is a maximal run of path characters, so a scheme and host fuse with the path into one run that equals no tracked name; splitting a URL into its path would make the reaction judge a foreign host's layout as though it were this repository's

- **pinned by**: `a_name_reached_only_through_a_url_is_a_stated_bound`

### `release-coherence/a-directory-named-without-its-trailing-slash-a-stated-bound`

> nothing reacts. Directories are derived slash-terminated, and the unslashed form is a word indistinguishable from ordinary prose — `scripts` is an English plural this document already uses as one. Admitting it for deeper names only would make the reaction judge which of its own keys read as English

- **pinned by**: `a_directory_named_without_its_slash_is_a_stated_bound`

## repository-checks

### `repository-checks/a-construction-shape-the-register-s-reader-does-not-model-a-stated-bound`

> the reference is read as a construction, whichever it actually names. **This bound used to be wider.** The register's reader was text over Rust and not exhaustive over the language: a byte char literal, a raw string, or a closure whose parameter list spanned two lines could desynchronise a character-by-character scan entirely, producing a site the reader neither parsed nor counted as unparseable — invisible to both of its readings at once, which was the unsafe direction this bound named, since a missed citation fails loud while a missed construction reports clean over a site nothing holds. Reading this repository's own Rust with a real parser instead of scanning it closes that floor: every syntactically valid construction is seen by construction, not by an arm added the day a shape was found wrong. **What remains is not lexical.** Whether a bare reference names the constructor taken by value or a local variable that happens to share its spelling is not written down anywhere a parse tree carries — answering it needs name resolution, which a reader of syntax alone does not have

- **unpinned**, tracked by: `BACKLOG.md` — *a bare reference to a registered constructor's name cannot be told from a local variable sharing its spelling without name resolution*

### `repository-checks/a-refusal-constructed-outside-the-register-s-corpus-is-not-triaged-a-stated-bound`

> nothing triages it. The register reads `crates/kanhe/src`, and a construction there is either held by a direction or declared unheld; a construction beside its own directions is neither, because *which direction observes this branch* has no answer when every direction in the file can see it. Reaching further means deciding what a file that is both judgement and test is being asked, which is a question about where those gates should live rather than about this register

- **unpinned**, tracked by: `BACKLOG.md` — *a gate that is its own test is outside the refusal register*

### `repository-checks/a-marker-is-reached-through-some-other-primitive-a-stated-bound`

> the reaction reports nothing. Those five are in live use for reading a *single* delimited value rather than pairing a sequence, and none of their live uses is a pairing — so refusing them would refuse the honest use, and telling the two apart needs the expression's shape rather than the primitive's name

- **unpinned**, tracked by: `BACKLOG.md` — *the backtick primitives the pairing reader names*

### `repository-checks/the-consumer-stands-on-a-later-statement-a-stated-bound`

> the reaction reports nothing. It joins a chain `rustfmt` broke and reads one logical line, so a consumer reached through a binding is outside what a line-scoped reader can decide; following the binding is name resolution, which no reader over text performs. Measured when this was written: no site in the tree binds a `split(…).next()` and consumes it later

- **unpinned**, tracked by: `BACKLOG.md` — *the always-Some consumer reached through a binding*

### `repository-checks/a-consumer-that-stops-early-is-neither-head-nor-grep-a-stated-bound`

> the reaction reports nothing. Naming programs is the instrument: the set that exits early is not closed, and the question behind it — *does this stage read its input to EOF* — is not one a reader over shell text can decide. This closes the door that was open, not every door

- **unpinned**, tracked by: `BACKLOG.md` — *the early-exit consumers the pipeline reader names*

### `repository-checks/a-count-written-in-a-sentence-no-census-declares-a-stated-bound`

> no repository check fires. The declaration is the coverage; reaching further needs a judgement over prose, which is the instrument measured three times and rejected. `AGENTS.md` carries the other half as a rule with no check: a count of something this repository does not produce is not written

- **pinned by**: `a_count_in_an_undeclared_phrasing_is_a_stated_bound`

### `repository-checks/a-figure-written-in-words-at-one-hundred-or-above-is-not-matched-a-stated-bound`

> the sweep does not match it, a stated bound: the word reader covers the units, the tens, and one compound of the two, which stops at ninety-nine. Extending it upward buys nothing measurable — the figures this repository writes in words are the small ones, and a set large enough to need three-digit words is one whose prose writes digits. The residual is stated rather than closed because a word reader that silently stops matching reads as covered, which is the failure this requirement's own sweep exists to refuse

- **pinned by**: `a_word_form_at_one_hundred_or_above_is_a_stated_bound`

### `repository-checks/a-census-written-outside-markdown-is-not-observed-a-stated-bound`

> the sweep does not see it, a stated bound: the corpus is tracked Markdown, and widening it was measured rather than reasoned about. This repository's Rust sources carry census phrases **as fixture input**, where the figures are a parser's expected output and deliberately arbitrary; admitting them would report a test asserting its own parser as a drifted document. The narrow corpus is what keeps the sweep's every report actionable, and the residual is a figure in a code comment, which `AGENTS.md` measured and left to the reviewer

- **pinned by**: `a_census_outside_markdown_is_a_stated_bound`

### `repository-checks/a-hook-is-proposed-for-this-rule-a-stated-bound`

> neither holds it: a squash merge runs on GitHub's servers so no local commit exists and no hook runs, and both values of that setting append the serial. Nor can a merge made in the browser be reached by a wrapper. The compliance point is one string passed at merge time, and this check guards the sanctioned path to it rather than every path

- **pinned by**: `a_merge_made_outside_the_wrapper_is_not_observed`

### `repository-checks/a-negative-value-cargo-documents-is-refused-by-the-shape-rule-a-stated-bound`

> the wrapper refuses it, and nothing will admit it short of a per-arm rule. A leading digit means a job count for one arm and nothing for `--package` or `--registry`, so admitting it means the shape question is asked differently per arm — which is the arrangement one shape check exists to replace, and the arrangement whose per-arm reasoning the refusal wording above was just corrected for repeating. The engine owns the narrowing: the caller passes the count instead, one arithmetic step

- **pinned by**: `a_refused_flag_cannot_sit_in_an_admitted_arguments_value_position`

### `repository-checks/files-no-capability-claims-a-stated-bound`

> no repository check fires. Subjects are declared where a capability has something to say, and requiring them to tile the tree would buy coverage with a claim per capability nobody could defend. The blindness is declared so that a clean report is not read as a complete one, and the check prints how many tracked paths went unclaimed rather than leaving the reader to assume none did

- **pinned by**: `files_no_capability_claims_are_reported_rather_than_implied_judged`

### `repository-checks/a-check-that-should-distinguish-a-region-and-does-not-a-stated-bound`

> no reaction sees either. An absence is not a shape, and nothing can scan for a filter that was never written; a disagreement between two scans is visible only to something that can already recognize what a region decision is, which is the same reaction. A reaction refusing an inline region decision was designed and measured against this repository: of the sites carrying the marker, only some are this class — the rest select commentary deliberately or parse a data format whose syntax marks comments — so it would refuse more legitimate sites than defects, which is how a gate earns being turned off. The classifier's adoption is what narrows this, and the narrowing is not a closure. The bound is declared on the construction rather than on an instance: a candidate one was reported and **refuted by measurement**, and a bound resting on a refuted instance would be worse than none

- **unpinned**, tracked by: `BACKLOG.md` — *a check that never wrote a region decision is invisible*

### `repository-checks/a-shell-comment-opened-by-a-metacharacter-stays-in-the-executed-region-a-stated-bound`

> the region keeps it, a stated bound: the rule tests for whitespace or line start, so commentary can satisfy a property about executed text. The direction is the over-including one and closing it needs word-splitting the classifier does not do; no tracked script carries the shape on an executed line

- **pinned by**: `a_shell_marker_after_a_metacharacter_stays_in_the_region`

### `repository-checks/a-whitespace-preceded-shell-marker-inside-quotes-is-cut-a-stated-bound`

> the region cuts it, a stated bound: executed text is deleted, which is the direction the Core Contract forbids, and the residue was for a window recorded as reaching the Rust region alone. Closing it needs the quote tracking the TOML region has, rewritten for the shell's own quoting; no tracked script carries the shape on an executed line

- **pinned by**: `a_shell_marker_inside_quotes_is_cut_from_the_region`

### `repository-checks/a-tool-configuration-set-in-the-environment-is-not-observed-a-stated-bound`

> the wrapper does not see it, a stated bound: the allowlist classifies **arguments**, and cargo takes the same configuration from the environment — measured on cargo 1.96.0, `--target not-a-real-triple` and `CARGO_BUILD_TARGET=not-a-real-triple` produce the identical rustc-probe failure. Closing it is ordinary work here rather than another layer's, since the wrapper could scrub the environment before invoking the tool; it needs an allowlist **over the environment**, and legitimate setups export `CARGO_HOME` and `CARGO_TARGET_DIR`, so which set to admit is a decision this bound records instead of guessing

- **pinned by**: `a_tool_configuration_set_in_the_environment_is_a_stated_bound`

### `repository-checks/a-gate-reached-without-the-wrapper-a-stated-bound`

> no repository check fires. Both assertions guard the sanctioned path; reaching further would mean observing the operator's shell or GitHub's servers rather than this repository. The pinning check narrows this without closing it: it keeps the sanctioned path sanctioned, so what is left is choosing not to use it rather than using it unguarded

- **unpinned**, tracked by: `BACKLOG.md` — *a merge or publish made outside the wrapper is not observed*

### `repository-checks/a-title-edited-inside-the-re-read-itself-a-stated-bound`

> nothing observes it, and the merge records the subject the gate approved against a title that has since moved. The wrapper judges three inputs and pins two of them by construction — the body travels as the value the gate judged, and the commit set is pinned through `--match-head-commit`, which GitHub decides atomically. `gh` offers no `--match-title`, so the third can only be re-read, which shrinks the exposure from a whole `cargo test` to one API call rather than closing it. Closing it needs a server-decided precondition this tool does not offer

- **unpinned**, tracked by: `BACKLOG.md` — *the title race the wrapper can only narrow*

### `repository-checks/whether-a-mention-compiles-anything-is-not-observed-a-stated-bound`

> the check counts it as named, a stated bound: deciding that a mention is load-bearing is a judgement over text, the instrument this repository has designed, measured and rejected, and what makes a mention bite is the compiler rather than this check. A comment-only mention still fails the reviewer reading the diff, which is the layer that owns it

- **pinned by**: `a_member_named_only_in_a_comment_is_counted_as_named`

## runtime-origin-assertion

### `runtime-origin-assertion/source-outside-a-member-s-library-or-binary-target-subtree-is-out-of-scope-a-stated-bound`

> the audit does not read it, its corpus being the member's library and binary targets — a stated bound shared with the semantic dimension, never a silent claim of coverage

- **pinned by**: `source_outside_lib_or_bin_target_subtree_is_out_of_scope_corpus_bound`

### `runtime-origin-assertion/a-production-probe-behind-a-non-production-cfg-is-still-counted-a-stated-bound`

> the audit counts it as coverage, being cfg-blind, so a seam whose production probe lives there is reported as probed — a stated bound, never a silent pass

- **pinned by**: `production_probe_behind_non_production_cfg_is_counted_as_coverage`

### `runtime-origin-assertion/identical-expression-repeated-in-the-same-function-collapses-to-one-finding-a-stated-bound`

> `audit_probe_coverage` emits one un-auditable-probe violation for that site — a stated bound, since no further source content distinguishes the two occurrences

- **pinned by**: `identical_expression_repeated_in_the_same_function_collapses_to_one_violation`

### `runtime-origin-assertion/an-absolute-path-literal-s-target-outside-the-anchor-keeps-its-absolute-label-a-stated-bound`

> `audit_probe_coverage` still emits the un-auditable-probe violation, naming the site with the raw absolute path — a stated bound, since the literal has no textual relationship to the anchor

- **pinned by**: `an_absolute_path_literal_outside_the_anchor_keeps_the_path_the_literal_wrote`

### `runtime-origin-assertion/a-composite-shape-yields-a-truncated-origin-a-stated-bound`

> the derived origin is a truncated rendering that equals no module name, so it matches no allowlist entry and in particular never the wrapped type's own defining module; the crossing reacts fail-closed rather than being admitted through the wrapper

- **pinned by**: `the_derived_origin_honors_its_stated_shape_bounds`

### `runtime-origin-assertion/a-probe-behind-a-symlinked-subdirectory-is-seen-from-the-root-and-not-from-the-directory-a-stated-bound`

> the root-file run reports the seam covered, while the directory run reports it unprobed — the stated bound of the legacy corpus, recorded rather than presented as equivalent coverage

- **pinned by**: `a_symlinked_subdirectory_is_descended_from_a_root_file_and_not_from_a_directory`

## self-law-projection

### `self-law-projection/a-doc-example-of-the-dependency-dsl-is-refused-a-stated-bound`

> the reaction refuses it anyway, a stated bound: it reads a comment's text and never its purpose, and the shell publishes that DSL, so the shape is live even with no instance in the tree today

- **pinned by**: `a_doc_example_of_the_dependency_dsl_is_refused`

### `self-law-projection/a-comment-naming-every-member-for-another-reason-is-refused-a-stated-bound`

> the reaction refuses it anyway, a stated bound: it asks whether the members all appear and never why, so a block naming them for another reason reads the same as a copied census

- **pinned by**: `a_comment_naming_every_member_for_another_reason_is_refused`

### `self-law-projection/a-reason-that-paraphrases-the-law-is-refused-a-stated-bound`

> the reaction **fires**, refusing a reason that genuinely states the law. Measured by writing that WHEN into the tree: paraphrasing `guibiao`'s clause produces *"dimension boundary for `guibiao` dropped the `三儀 ⊥ 三儀` clause"*. The direction is the safe one — an author meets a refusal to argue with — and closing it needs the check to decide two wordings state one law, a judgement over prose measured and rejected here

- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

### `self-law-projection/a-reason-carrying-the-clause-while-negating-the-law-is-not-observed-a-stated-bound`

> nothing reacts, and `AGENTS.self-law.md` projects the negation to every agent that loads it. This is the serious direction of the pair: the teaching surface can carry the law's opposite while satisfying the check that exists to keep the law taught. Measured, with the projection blessed and the whole suite green

- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

### `self-law-projection/a-workspace-dependency-allowlist-is-not-examined-a-stated-bound`

> the reaction never examines it, though that rule governs workspace-member edges specifically and is the more natural one for this law. Measured: a second `guibiao` boundary of that variant naming `hunyi` is green, and set coverage still reads the three dimensions

- **unpinned**, tracked by: `BACKLOG.md` — *four limits of the mutual-independence check*

## semantic-async-exposure-boundary

### `semantic-async-exposure-boundary/a-body-nested-module-is-a-stated-bound`

> the system does not observe `hidden` — a `mod` inside a fn body is not public API (not reachable as `crate::…`), a stated bound, never a silent claim about it

- **pinned by**: `async_subtree_does_not_observe_a_body_nested_module`

## semantic-dyn-trait-boundary

### `semantic-dyn-trait-boundary/a-public-item-naming-such-an-alias-is-not-expanded-a-stated-bound`

> the system reacts at the alias declaration but emits **no additional** reaction for `make` via alias expansion — the `dyn` is already caught at the alias, and `type` aliases are not expanded (a stated bound)

- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-boundary/a-private-alias-hiding-a-dyn-in-a-public-position-is-a-stated-bound`

> the system does not claim to observe the hidden `dyn` (a stated coverage bound — the resolver does not expand `type` aliases), rather than silently asserting the boundary is clean

- **pinned by**: `a_private_alias_hiding_a_dyn_is_a_stated_bound`

### `semantic-dyn-trait-boundary/a-macro-generated-dyn-is-a-documented-bound`

> the system does not claim to observe it (the universal 渾儀 macro-expansion bound), rather than silently asserting the boundary is clean

- **pinned by**: `a_macro_generated_dyn_is_a_documented_coverage_bound`

### `semantic-dyn-trait-boundary/an-unrenderable-sub-node-is-a-stated-bound`

> the system does not claim to distinguish them: they share a canonical `subject` field and key at the same seam (each still *reacts* on first occurrence; only baseline-dedup granularity is bounded). This is a **stated subject-rendering bound** — the same `(target, rule_key, fact)` granularity bound `semantic-trait-impl-locality`'s `(impl for <self_ty>)` fact carries — declared here, never a silent claim of cleanliness

- **pinned by**: `an_unrenderable_sub_node_is_a_stated_rendering_bound`

## semantic-dyn-trait-operand-boundary

### `semantic-dyn-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> the system does not resolve the principal and reports no violation — a stated resolver-coverage bound (the oracle does not over-reach a single bare segment), never a silent claim of cleanliness over a resolvable operand

- **pinned by**: `dyn_operand_genuinely_unresolvable_bare_principal_is_a_bound`

## semantic-forbidden-marker

### `semantic-forbidden-marker/an-unresolvable-hand-impl-self-type-is-a-documented-bound`

> the system does not claim to observe it (a stated coverage bound), rather than silently asserting cleanliness — the co-located, `use`-imported, re-export-spelled, and type-alias cases (the common ones) do resolve and react

- **pinned by**: `an_unresolvable_glob_self_type_is_a_documented_bound`

## semantic-impl-trait-operand-boundary

### `semantic-impl-trait-operand-boundary/a-genuinely-unresolvable-bare-principal-is-a-documented-bound`

> the system does not resolve the principal and reports no violation — a stated resolver-coverage bound, never a silent claim over a resolvable operand

- **pinned by**: `impl_trait_operand_genuinely_unresolvable_bare_principal_is_a_bound`

## semantic-reexport-exposure

### `semantic-reexport-exposure/an-underscore-rename-is-a-documented-bound`

> the system does not react — `as _` binds no nameable path a consumer can reach — and this is a stated bound, not a silent claim of cleanliness

- **pinned by**: `restricted_and_private_and_underscore_reexports_do_not_react`

### `semantic-reexport-exposure/a-sibling-root-glob-is-a-documented-bound`

> the system does not claim to observe the transitively re-exported leaf (the inherited glob bound — the glob's leaves are not enumerable here), rather than silently asserting the boundary is clean

- **pinned by**: `sibling_root_glob_does_not_react`

### `semantic-reexport-exposure/an-ancestor-root-glob-spanning-a-deeper-forbidden-prefix-is-a-documented-bound`

> the system treats it as a stated bound (it cannot enumerate whether `crate::infra` publicly re-exports the forbidden `db` subtree), documented as the sharper ancestor-root sub-case rather than lumped with the innocent sibling glob or silently claimed clean

- **pinned by**: `ancestor_root_glob_over_a_deeper_forbidden_prefix_does_not_react`

### `semantic-reexport-exposure/a-facade-hop-re-exporting-a-privately-used-bare-name-is-a-stated-bound`

> the system does not follow that hop (the closure captures only inline `pub use` paths) and this is a documented inherited bound, not silently claimed clean

- **pinned by**: `facade_hop_reexporting_a_privately_used_bare_name_is_a_stated_bound`

### `semantic-reexport-exposure/a-non-forbidden-root-external-glob-is-a-documented-bound`

> the system does not claim to observe the glob's individual leaves, rather than silently asserting the boundary is clean

- **pinned by**: `extern_glob_nonforbidden_root_is_a_stated_bound`

### `semantic-reexport-exposure/a-re-export-renamed-through-a-foreign-module-is-a-documented-bound`

> the system matches only the written path (`worklane_core::prelude::Foo`, not in/under the forbidden set) and does not silently claim to have followed the foreign chain

- **pinned by**: `foreign_prelude_rename_is_a_stated_bound`

### `semantic-reexport-exposure/a-module-scoped-extern-crate-rename-is-a-documented-bound`

> the system does not resolve `wc` to `worklane_core` (only crate-root renames are collected, since a module-scoped alias binds only locally) and this is a documented bound, distinct from the handled crate-root rename

- **pinned by**: `module_scoped_extern_crate_rename_is_a_stated_bound`

## semantic-signature-coupling

### `semantic-signature-coupling/an-invocation-inside-an-impl-body-is-a-stated-bound`

> the system reports no exposure — transparency covers item position, where an arm's contents are items; an `impl`-body invocation's arms are impl items, observed through different walkers, and that remains a declared gap rather than a claimed reaction

- **pinned by**: `a_cfg_if_inside_an_impl_body_is_a_stated_bound`

### `semantic-signature-coupling/a-macro-under-another-name-is-not-treated-as-transparent-a-stated-bound`

> the system reports no exposure — the invocation is not transparent, so its body stays a stated coverage bound; extracting from it would read the `impl` body's braces as an arm and report an item the macro may never emit

- **pinned by**: `an_arbitrary_macro_body_is_not_read_as_transparent_arms`

### `semantic-signature-coupling/a-plain-item-nested-the-same-way-stays-a-stated-bound`

> the system reports no exposure — only an `impl` block is recovered from a body this way; a plain item nested the same way is genuinely scoped to that body and unreachable as `crate::…`, exactly like the existing body-nested-module bound, so it stays unobserved rather than a new, unaudited claim

- **pinned by**: `a_plain_fn_directly_in_a_const_body_stays_a_stated_bound`

### `semantic-signature-coupling/an-impl-nested-one-level-further-or-static-wrapped-is-a-stated-bound`

> the system reports no exposure for that impl — a stated coverage bound rather than a silent claim of cleanliness

- **pinned by**: `an_impl_nested_one_level_further_stays_a_stated_bound`
- **pinned by**: `a_static_wrapped_impl_stays_a_stated_bound`

## semantic-trait-impl-exposure

### `semantic-trait-impl-exposure/a-glob-imported-type-in-an-impl-position-is-a-documented-bound`

> the system does not claim to observe it (inherited glob bound), rather than silently asserting the boundary is clean

- **pinned by**: `a_glob_imported_type_in_an_impl_position_is_a_documented_coverage_bound`

## semantic-trait-impl-locality

### `semantic-trait-impl-locality/a-macro-generated-impl-is-a-documented-bound`

> the system does not claim to observe it (out of scope, the same nature as the existing macro bound), rather than silently asserting the boundary is clean

- **pinned by**: `a_macro_generated_impl_is_a_documented_bound`

### `semantic-trait-impl-locality/a-cfg-gated-module-with-an-absent-file-is-skipped-not-a-scan-error-a-stated-bound`

> the whole-crate walk skips the module (a stated coverage bound) rather than failing the gate with a scan error (exit 2)

- **pinned by**: `hunyi::a_cfg_gated_module_with_no_file_is_skipped_not_errored`

## semantic-unsafe-confinement

### `semantic-unsafe-confinement/macro-generated-unsafe-is-a-documented-bound`

> the system does not claim to observe it (out of scope, the dimension's macro bound), rather than silently asserting the module is unsafe-free

- **pinned by**: `unsafe_in_a_macro_body_is_a_stated_bound`

## semantic-visibility-boundary

### `semantic-visibility-boundary/a-macro-generated-item-is-a-documented-bound`

> the system does not claim to observe it (out of scope, the same nature as the dimension's existing macro bound), rather than silently asserting the module is clean

- **pinned by**: `a_macro_invocation_pub_item_is_a_documented_bound`

### `semantic-visibility-boundary/a-pub-in-narrow-path-item-may-over-react-under-a-tight-ceiling-a-stated-bound`

> the system MAY react (the conservative `Crate` rank exceeds the `Module` ceiling), a stated over-reaction bound, never a silent pass

- **pinned by**: `a_pub_in_narrow_path_over_reacts_under_a_module_ceiling`
