## Context

`scripts/merge-pr.sh` reads the body once, guarded, at line 205 and hands the **value** to the gate as
`TIANHENG_MERGE_BODY`. Its final `exec` at line 336 hands `gh pr merge` the **path**. Between those two lines
sits `cargo test -p kanhe --test merge_message`, which on a cold target directory is minutes rather than
seconds. Anything that rewrites the file in that interval — an editor autosave, a "quick typo fix" started
after the wrapper was launched — is recorded by the merge and was judged by nothing.

The wrapper already treats this exact split as the thing it exists to prevent. Its allowlist refuses a
caller's `--body`, `--body-file=`, `-F*`, `-b*` in every spelling, and the refusal text says why: *"the message
this wrapper hands to the gate is the message the merge records, and gh takes the last spelling of a repeated
flag — so this would have the gate judge one message and the merge write another."* The requirement it serves
already pins the remote side: the head is captured before the commit set and supplied as
`--match-head-commit`, so a pull request that moved is refused. The local side had no equivalent.

Measured for this change: `gh version 2.95.0` — the version the allowlist is classified against — accepts
`-b, --body text` on `pr merge`. The substitution is available at the named version rather than assumed.

## Goals / Non-Goals

**Goals:**

- The bytes the gate judged are the bytes the merge records, with no second read of any source.
- The obligation is stated over **every** judged input, so the next input added is answered rather than
  discovered.
- The direction that holds it observes what the act would record, not what the wrapper's source text looks
  like.

**Non-Goals:**

- Reaching a merge made outside the wrapper. That is already a declared bound
  (`repository-checks/a-hook-is-proposed-for-this-rule-a-stated-bound`, pinned by
  `a_merge_made_outside_the_wrapper_is_not_observed`) and nothing here narrows or widens it.
- Closing the same question for `scripts/publish.sh`. That wrapper hands its gate no inputs at all — the gate
  reads git itself — so the property is vacuous there, and a rule with one live site and one vacuous site is
  the name-without-a-reaction the drift law refuses. It is stated as a requirement over judged inputs, which
  is empty for a wrapper that supplies none.
- Any change to the read itself. Reading once, guarded, with a cannot-judge refusal distinct from a body that
  disagrees, is already right and is what makes the value available to pass.

## Decisions

**Pass the value through `argv`, not through a wrapper-owned temporary file.**
The alternative — write `$body` to a `mktemp` the wrapper owns and pass that path — also closes the race, since
nothing outside the wrapper knows the name. It is rejected because the file must survive until `gh` reads it,
which is *after* `exec` has replaced the shell image: it could not be removed before the `exec`, and an `EXIT`
trap does not run across one. That is precisely the leak this window closed one commit ago — three successful
runs left three empty files in `TMPDIR` — and reintroducing it to fix a different defect trades one for the
other.

**The ceiling this introduces is fail-loud, so it earns no bound.**
A value through `argv` is bounded by `ARG_MAX` where a path is not. A body large enough to exceed it makes the
`exec` fail with `E2BIG` **before** the merge, which is a refusal rather than a false negative — the direction
this family always prefers. It is also far from reachable: the tool's own body limit is orders of magnitude
below a typical `ARG_MAX`. A declared bound records where a measure silently stops; this stops loudly, so it
is a note rather than a declaration.

**The wrapper's own `--body` cannot be overridden by a later occurrence.**
`gh` takes the last spelling of a repeated flag, and the wrapper splices `"${passthrough[@]}"` after its own
arguments. That is safe only because every caller spelling of a body flag is already refused by the allowlist,
so `passthrough` can never contain one. The safety is a property of the allowlist rather than of the argument
order, which is worth stating because reordering the splice would not break it and removing an allowlist arm
would.

**State the obligation over all four inputs rather than as a body repair.**
Three of the four already satisfied it and nothing said they were one set, which is exactly why the fourth
could sit there through the review rounds that produced this wrapper. A requirement naming the property makes
the next input answer it; a requirement naming the body makes the next input a fresh discovery.

**Observe what the act would record, by having the controlled `gh` resolve its body.**
The direction gives the wrapper a body file, has the controlled `cargo` — which stands where the gate runs —
rewrite that file, and then asserts the body the merge invocation carries is the original. The controlled `gh`
must therefore resolve `--body-file P` by reading `P` and `--body V` by taking `V`, as the real tool does, so
the assertion is about the recorded body rather than about which flag was spelled. Asserting the flag name
instead would pass for a wrapper that passed `--body "$(cat "$body_file")"` at merge time, which re-reads and
is the defect.

**The negative run is the same direction against the unchanged wrapper.** With `--body-file`, the controlled
`gh` reads the mutated file and the assertion fails on the recorded body. That is the observation this change
must record in `## Verification`.

## Risks / Trade-offs

**The harness logs `"$*"`, so a body with newlines splits one invocation across log lines** → Existing
directions locate the merge with `gh_log.lines().find(|line| line.starts_with("pr merge"))` and then assert
`--match-head-commit` on that line. A multi-line body would push later arguments onto following lines and
break assertions that have nothing to do with this change. The controlled `gh` must log newline-safely, and
every existing `merge_workflow.rs` direction must still pass unchanged — that is the acceptance condition, not
a hope.

**Log-shape changes can silently weaken a sibling direction** → Any change to how the controlled executable
records its arguments is a change to the observation source of a dozen directions at once. Whatever form is
chosen, the existing assertions keep their current text where possible; where one must move, it is named in
the pull request rather than adjusted quietly.

**The direction proves the wrapper's behaviour, not `gh`'s** → The controlled executable stands in for the
real tool, so the direction shows what the wrapper hands over and what a faithful tool would record. Nothing
here observes the real `gh`. That limit is not introduced by this change — it is the shape of every direction
in `merge_workflow.rs` — so it is recorded here rather than declared as this change's bound.

## Migration Plan

None. `scripts/` ships in zero packages, no published crate, signature, wire format, exit class, baseline or
manifest is touched, and the workspace version does not move. The changelog entry belongs under
`### Self-governance`; the adopter-narrative reaction refuses a `scripts/` path under any of the eight adopter
headings, and there is no adopter-visible guarantee to restate.

## Open Questions

- **Does the controlled-executable substitution deserve a declared bound of its own?** No bound in
  `crates/kanhe/src/bounds.rs` covers it today, and it is the observation limit of every direction in
  `merge_workflow.rs` — not of this one. Filing it belongs to a change whose subject is that harness, and it is
  named here so the omission is deliberate rather than missed.
