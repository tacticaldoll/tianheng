## Context

`scripts/check_publish_source.sh` stands before `cargo publish`, which stamps a permanent commit pointer into
every tarball. Its contract is written in its own header and nowhere in the specification surface. Its signature
assertion greps the whole tag object, so a quoted signature block satisfies it — measured on a fixture.

## Goals / Non-Goals

**Goals.** Give the gate's contract a home in the claim surface. Verify a signature rather than match its shape.
Keep every environment's verdict identical. Separate what this gate can judge from what it cannot, and declare
the remainder.

**Non-Goals.** Judging *who* signed — that needs an allowed-signers configuration CI has not, and requiring it
would make the same tag judged differently by where the gate ran. Verifying published content (the gate asserts
committed state only). Changing any other release requirement.

## Decisions

### D1 — `ssh-keygen -Y check-novalidate`, because it is the only mechanism that separates the two questions

Four mechanisms were measured against two fixtures, with no allowed-signers file anywhere. Three fail:

- the current `grep` accepts the quoted block — the defect;
- `%(contents:signature)` **also** returns the quoted block, so the extraction is right and the assertion is not
  (this was one review's proposed fix, and the measurement refutes it);
- `git verify-tag` cannot run at all: exit 1 with `allowedSignersFile needs to be configured` for **both**
  fixtures, and `--raw` gives the same. Adopting it would make the gate always exit 2 on CI — the check disabled
  while looking strengthened. This was another review's proposed fix.

`ssh-keygen -Y check-novalidate -n git -s <sig>` over the payload accepts the signed fixture and refuses the
quoted one, with no configuration. It is cryptographic verification with the attribution step omitted, which is
precisely the split this gate needs.

### D2 — The refuted cause is corrected in the open, not quietly

The header's stated bound says verification needs allowed-signers. It does not; attribution does. That
conflation is why the gate matched a shape for as long as it did, and the correction is the substance of this
change rather than a footnote to it.

It is also a live instance of `observation-bound-model`'s declared bound that a declaration's stated cause is
not observed — the second instance found today. A bound whose *cause* is wrong is worse than one with no cause,
because the cause is what a later author reasons from: this one said "you cannot check this", and so nobody did.

### D3 — What this gate cannot verify is cannot-judge, never a violation

A non-SSH signature is verifiable in principle and unreadable by this mechanism. Reporting it as a wrong source
would be a false refusal before an irreversible act, which is the direction that costs a release. Exit 2 says
what is true: the gate cannot judge this shape.

The same reasoning moves the tag-object read failure from 1 to 2. It is currently folded into `|| fail`, so a
gate that could not read the tag reports a wrong source.

### D4 — The bound is owned by the environment, not the engine

`Extent::Reached(Reached::UnderReacts { owner: Owner::Inherited { from: "the verification environment" } })`.
Not `Engine`: no change to this gate closes it, because attribution needs a configuration that exists on a
maintainer's machine and not in CI. Not `Adopter`: no adopter declaration affects it. Naming the environment is
what makes the owner actionable — the way to close it is to give CI an allowed-signers file, which is a
repository decision rather than a code change.

### D5 — The five existing assertions are described, not designed

Each already has a direction in `scripts/test_publish_source.sh`, so writing them into a requirement describes
the tree. That is deliberate: a capability whose first act was to change five behaviours would be a redesign
wearing a specification. The one behaviour that changes is the signature assertion, and it changes because a
measurement refuted the reason it was weak.

## Risks / Trade-offs

**`ssh-keygen` becomes load-bearing for the gate, not just its twin.** The twin already refuses to run without
it (`command -v ssh-keygen || exit 2`), so the dependency is declared; the gate now needs it too and must refuse
the same way rather than treating its absence as an unsigned tag.

**The payload reconstruction was wrong in the first draft of this design, and the review that found it is the
reason it is not shipping.** It said to strip from the first `-----BEGIN SSH SIGNATURE-----` line. Measured on a
**genuinely signed** tag whose message also quotes a verification log, that reconstructs a truncated payload and
**refuses a real signature** — a false refusal before an irreversible act, introduced by the very change meant to
harden it.

The mechanism is therefore suffix removal of the exact `%(contents:signature)` value, which is the trailing block
by construction: `payload=${obj%"$sig"}`. Measured on the same tag, that verifies. A quote earlier in the message
stays part of the payload, where it belongs, and the unsigned fixture still refuses because the block it trails
with is not a signature.

The residual that remains is narrow and named: the payload is reconstructed textually rather than asked for, so a
future git that changes how a tag object serialises its trailer would break the reconstruction. It would break it
in the refusing direction, and the twin's signed-tag direction is what fails first.
