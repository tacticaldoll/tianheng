## Context

`Baseline::from_json` already rejects malformed required fields and unknown versions, but optional
`owner` and `tracker` use `Value::as_str`, conflating a missing key, explicit `null`, and every other
JSON type. The first two forms reasonably mean "no annotation"; the last is observable malformed
governance data. The CLI gate surfaces parser errors, while `--write-baseline` deliberately warns
and writes a fresh snapshot when the prior file cannot be parsed.

## Goals / Non-Goals

**Goals:**

- Give optional metadata a precise `missing | null | string` input contract.
- Make wrong-typed metadata fail through the same parser and CLI reactions as other malformed
  baseline fields.
- Keep generated output canonical: absent metadata remains omitted rather than serialized as null.

**Non-Goals:**

- Changing version-1 matching, version-2 structured identity, or metadata preservation rules.
- Adding metadata schemas, validation policy for string contents, or new migration commands.
- Warning on every version-1 read or combining legacy migration documentation into this behavior
  change.

## Decisions

### Treat explicit null as absence

JSON `null` is the natural explicit form of an unset optional value and round-trips semantically to
the same `Option::None` as an omitted key. Rejecting null would add strictness without protecting
information. Generated baselines continue to omit unset keys for stable diffs.

### Validate optional fields in the shared baseline parser

The parser is the single source used by the composed CLI and standalone dimension adopters. A small
field helper will distinguish missing/null from string and return an entry-local error for every
other type. Validating only in `tianheng` would let public `xuanji::Baseline::from_json` retain the
silent-loss bug and duplicate policy in the shell.

### Preserve the write path's explicit-loss escape hatch

`--write-baseline` is an explicit snapshot operation. It already warns when the previous file is
unreadable and then writes fresh state; wrong-typed metadata follows that same path. Turning this
case into a refusal would be a separate behavioral decision and would conflict with the existing
recovery contract.

## Risks / Trade-offs

- Existing hand-edited files with accidental numeric, boolean, array, or object metadata become
  invalid. This is intentional fail-loud behavior; replacing the value with a string, null, or
  omission repairs the file.
- Treating null and omission alike is not byte-preserving on rewrite because generated output omits
  null. They are both explicitly absence, so no governance information is lost.
