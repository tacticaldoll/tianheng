## Context

The census direction captures `(cd "$repo" && grep ...)` and later accepts statuses 0 and 1 because grep uses 1
for no match. Shell composition erases which command produced that 1. The shared ERR backstop cannot recover the
meaning because the status is deliberately captured and handled.

## Goals / Non-Goals

**Goals:**

- distinguish failure to enter the judged repository from a successful census scan with no match;
- preserve grep's 0/1 result contract;
- prove the distinction at the census boundary in the existing failure matrix.

**Non-Goals:**

- change census wording or matching;
- alter other bound-register directions;
- broaden the shared exit-contract helper.

## Decision

Run `cd "$repo"` before the grep capture and route its failure directly through `cannot_judge`. Once the working
directory is established, only grep contributes to `census_status`, so exit 1 again has one meaning.

The matrix will wrap `git` and move the fixture after the final tracked-Markdown enumeration. All earlier reads
therefore succeed, while the separately checked directory transition observes the exact unavailable-root state.

## Verification

The new matrix direction must exit 1 against the old combined capture, demonstrating the clean/violation result,
and exit 2 with the dedicated diagnostic after the repair. The full gate matrix and repository DoD then run.
