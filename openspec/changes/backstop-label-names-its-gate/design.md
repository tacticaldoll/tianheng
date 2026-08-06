## Context

`scripts/lib/exit_contract.sh` takes one argument and interpolates it into the trap's message as a prefix. It is
the gate's name as the gate declares it, and it is the only part of that diagnostic the gate chooses — the exit
status, the source path and the line all come from the shell at fire time.

The reaction added by `gate-shape-contract` already reads every gate's text and already asserts that
`exit_contract_backstop` is both sourced and invoked. What it does not read is the argument.

## Goals / Non-Goals

**Goals.** Make a gate's self-identification derivable from its filename, so a copy-paste cannot produce a
diagnostic that contradicts its own source path. Keep the failure message enough to repair from.

**Non-Goals.** Governing the gates' *other* self-identifications. Five of the six prefix their ordinary output
with the same words (`bound register ok`, `reference integrity ok`), and `check_dod_coherence.sh` does not
(`ok: every local Definition of Done command (23 parsed) is run by CI`). Requiring that too would be a
migration on one gate for a message no reader has been misled by, and it is a different property — how a gate
speaks when it *works*, not how it is identified when the shell kills it. Not declared as a bound either,
because a bound says a reaction stops at a shape it might have been expected to cover, and nothing about the
exit contract suggests this one.

## Decisions

### D1 — The label is derived from the basename, not from a hand-kept table

The alternative is a map from gate to label, which is a second declaration of the gate's name and rots exactly
as the thing it would be checking. The basename already names the gate, and the transformation is total:
`check_` prefix and `.sh` suffix removed, `_` read as space. Measured, all six gates already satisfy it, so
there is no gate whose natural name the rule cannot express.

The rule's weakness is stated rather than hidden: a gate whose good name is not its basename with underscores
respaced — one wanting an acronym in caps, say — must rename the file or argue the requirement. That is the same
trade the surface's `check_` naming already makes, and the required-set-over-allowlist argument applies
unchanged: an allowlist of label exceptions rots silently.

### D2 — The label must be a literal, and an unreadable one is refused as unreadable

The invocation is matched on a line whose first word is `exit_contract_backstop`; the remainder is trimmed and
stripped of one surrounding pair of `'` or `"`. Unquoted is accepted because the shell accepts it for a
single-word label, and refusing it would be a rule about quoting rather than about naming.

A label built by expansion is **not** resolved — the reaction reads text and does not evaluate it. The first
draft of this design said such a label is "reported as a mismatch", and that was wrong in two ways caught on
review. It states something false: a gate writing `exit_contract_backstop "$name"` wrote neither the label the
message would accuse it of nor the derived one, so "you wrote X, the basename asks for Y" is a fabricated
comparison. And it hides the sharper point — `exit_contract_backstop "$(basename "$0" .sh)"` is a **better**
implementation than any literal, since it cannot disagree with the filename at all, and a reaction reporting it
as a naming violation is the invented-violation direction this capability least affords.

So the requirement is explicit that the authored form is a literal, and the refusal says *that*: the label could
not be read as a literal. The form is required because the property has to be checkable by reading, and admitting
derivations would make the rule about which spellings of a derivation the reaction recognizes. That the required
form is not the better form is stated in the requirement rather than left for a reader to notice.

A bound is not the right shape for this: the reaction does look at the argument and does reach a verdict, refusing
where it cannot confirm. A bound would claim it stops there silently, which is the opposite.

### D2a — A gate with no invocation reports two absences, not one

The label property is measured from the invocation, so a gate that never invokes the backstop has no label. Both
properties then fail, and both offences are printed. That follows the precedent already set for an absent twin,
which reports the four matrix properties it cannot hold: each names a real absence, and suppressing the dependent
would need a third value of "held" meaning *not applicable*, which is a claim about the gate that neither
`yes` nor `no` is making.

The per-property fixture test therefore expects **two** offences when the backstop is withheld — the same shape
as the twin case expecting five.

### D3 — A failure prints both labels

`scripts/check_x.sh: backstop label — the label passed to exit_contract_backstop is 'y', and this gate's
basename asks for 'x'`. The remedy is then the message. This matters more than it sounds: the property exists
because of copy-paste, so the reader who trips it is by construction someone who has just copied a sibling and
is not looking at either name.

### D4 — The counts leave prose instead of being incremented

"Nine" appears in `AGENTS.md`, in `BACKLOG.md`'s closed entry, and in several of the reaction's own doc
comments, including one line — "three per gate, five per twin, one over `AGENTS.md`" — sitting directly above
the array it counts. Incrementing them all is the maintenance this repository has already been bitten by four
times in one tree, and the projection prints the figure from the array. So prose points at the projection and
states no number.

## Risks / Trade-offs

**The property is cosmetic in the worst case and this change says so.** A wrong label costs a reader one wrong
turn, not a wrong verdict; the gate still exits 2 and CI still fails. It is here because the cost of checking is
one array entry against a surface that already holds it, not because it is dangerous.

**It is the tenth property, and a tenth invites an eleventh.** The guard against a checklist growing for its own
sake is the same one the original nine were held to: each must be a class this repository *observed*. This one
is observed as a hazard rather than as an incident — no gate has ever carried a wrong label — and that is a
weaker warrant than the nine had, stated here so a future addition is measured against the nine and not against
this.
