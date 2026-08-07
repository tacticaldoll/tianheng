## Context

`check_reference_integrity.sh` already carries a detailed behavioral contract in its source header and
implementation, and `test_reference_integrity.sh` exercises its clean state and refusal directions. The
capability spec was introduced later from a narrow change and currently records only hermetic governance
policy and fixture-only narrowing.

## Goals / Non-Goals

**Goals:**

- Recover the stable observable contract from the gate and its existing matrix.
- Group the inventory by responsibility rather than mirror shell control flow line by line.
- Ensure every added scenario names a state already exercised by the matrix or positive repository gate.

**Non-Goals:**

- Change extraction syntax, path resolution, exit behavior, or the test matrix.
- Turn implementation details such as temporary-file placement into independent requirements.
- Claim reference forms or language corpora the gate does not inspect.

## Decisions

Add requirements for five established responsibilities: tracked checkout evidence, syntax-aware path
resolution, deliberate exclusions, fail-loud observation, and the read-only 0/1/2 contract. Keep the two
existing policy-isolation requirements intact.

Use the matrix's existing fixture directions as the scenario inventory: clean, missing governance,
missing members, empty corpus, stale prose/link/test references, ignored paths, active OpenSpec plans,
failed extraction/index/unhandled commands, repository immutability, and silent clean stderr. The positive
gate supplies coverage for the real tracked corpus and its bounded illustrative forms.

Do not add a new detector or edit scripts. N5 is a specification inventory defect; behavior and reaction
already agree.

## Risks / Trade-offs

- A requirement could accidentally promise more syntax than the regex observes → name only the concrete
  forms and bounds stated in the gate header.
- A scenario could become inert prose → tie each scenario to an existing matrix direction or the positive
  gate in PR verification.
- Mirroring every implementation branch would make the spec brittle → specify verdict-bearing behavior,
  not command arrangement.
