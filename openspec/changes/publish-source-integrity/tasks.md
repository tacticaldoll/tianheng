## 1. First, confirm the five described assertions really are described

- [ ] 1.1 For each of the five committed-state assertions, find its direction in `scripts/test_publish_source.sh`
      and record which test proves it. Any assertion with **no** twin direction is not described but invented, and
      must either gain a direction in this change or leave the requirement.
- [ ] 1.2 Specifically check the remote-`main`-tip assertion: the twin builds a bare origin and pushes, so the
      direction may exist — confirm by reading rather than assuming.

## 2. The signature verification

- [ ] 2.1 Replace the whole-object `grep` with: extract `%(contents:signature)`; refuse when empty; reconstruct
      the payload by **suffix removal** (`payload=${obj%"$sig"}`); verify with
      `ssh-keygen -Y check-novalidate -n git -s <sig>`.
- [ ] 2.2 `ssh-keygen` absent SHALL be cannot-judge (2), the same refusal the twin already makes for itself —
      never read as an unsigned tag.
- [ ] 2.3 A signature `check-novalidate` cannot decode: distinguish *not a signature at all* (exit 1, the tag is
      unsigned) from *a signature this mechanism cannot read*, e.g. a non-SSH one (exit 2). Decide the
      discriminator from the tool's own output and pin it, because this is the false-refusal direction.
- [ ] 2.4 The tag-object read moves out of `|| fail` so its failure exits 2.

## 3. Observed failing, every direction

- [ ] 3.1 The quoted-signature fixture: annotated, unsigned, message containing a signature block → exit 1. This
      is the defect; the fixture is already built and passes today.
- [ ] 3.2 A genuinely signed tag with **no** allowed-signers file (`GIT_CONFIG_GLOBAL=/dev/null`) → exit 0.
      Without this direction the fix could be a `verify-tag` in disguise.
- [ ] 3.3 A genuinely signed tag whose message **also** quotes a signature block → exit 0. This is the
      false-refusal the first draft of the mechanism produced; it must be a fixture, not a memory.
- [ ] 3.4 A tag-object read failure → exit 2, not 1.
- [ ] 3.5 `ssh-keygen` unavailable → exit 2 naming it.
- [ ] 3.6 Perturb the verification to always succeed and confirm 3.1 fails, so the guard discriminates rather
      than passing by construction.

## 4. The bound

- [ ] 4.1 `a_valid_signature_from_an_unauthorized_key_is_accepted` — demonstrate: a tag signed by a key no
      allowed-signers file names is accepted. A fixture, not prose.
- [ ] 4.2 Typed declaration in `tianheng::observation_bounds()`: `UnderReacts` with
      `Owner::Inherited { from: "the verification environment" }` (design D4 — not `Engine`, because no change to
      this gate closes it).
- [ ] 4.3 Confirm it resolves under `bash scripts/check_bound_register.sh` after sync, and take the register's
      new figures (54 bounds across 20 capabilities) from the run rather than by counting.

## 5. Coherence

- [ ] 5.1 The gate's header: the stated bound's **cause was refuted** — verification does not need
      allowed-signers, attribution does. Correct it in the header too, not only in the spec, and say so.
- [ ] 5.2 `CHANGELOG.md`: a **Fixed** entry. An adopter verifying a tarball's provenance is the reader who cares.
- [ ] 5.3 `BACKLOG.md`: record that `observation-bound-model`'s *stated cause is not the real cause* bound has now
      produced its second live instance today. Two instances is a trigger worth naming.
- [ ] 5.4 Sweep for prose this invalidates: the gate's header, and anything in `CHANGELOG.md` claiming the gate
      checks a signature's presence.

## 6. Verification and sync

- [ ] 6.1 Every observation from task 3 in the pull request's `## Verification`.
- [ ] 6.2 Full Definition of Done, then again from a clean clone.
- [ ] 6.3 `openspec validate` strict before and after sync; archive pruned.
